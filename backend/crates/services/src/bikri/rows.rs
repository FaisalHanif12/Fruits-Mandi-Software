//! The rows that a posting of a بکری writes after its journal entry (PRD FR-12.8, plan 13 and
//! 16): the lots, the sale lines with their snapshots, the stock moves and the analytics facts,
//! and the matching of the new bills and credits. The save of a new بکری, a later sale and an edit
//! all use these functions, so a line is written one way only (Code Standards B5, G6).

use std::collections::{BTreeMap, BTreeSet};

use mandi_db::bikri::edit::{self as edit_repo, LotSums};
use mandi_db::bikri::write::{
    self as repo, LotChargeColumns, NewCreditPart, NewFact, NewLot, NewSale, NewStockMove,
    PukhtaColumns, SaleChargeColumns, StockKind,
};
use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::bikri::{CleanPerson, clean_person};
use mandi_domain::core::bikri::{
    LineCalc, LineCharge, LotCharge, LotChargeMethod, LotPreview, PaymentType, PreparedLot,
    PreparedSale, SummaryTotals,
};
use mandi_domain::core::money::{Grams, Paisa};
use mandi_domain::ledger::bikri::{EntryLayout, JournalLot, JournalSale};
use mandi_domain::ledger::{BalancedEntry, Side};

use super::shared::{Worked, data};
use crate::error::{ServiceError, money_error};
use crate::matching::{self, CashSalePair, NewLine};
use crate::purchases;

/// One worked sale line with the three views of it that the save needs.
pub(super) struct RowShape<'a> {
    pub row: &'a PreparedSale,
    pub calc: &'a LineCalc,
    pub journal: &'a JournalSale,
}

/// One worked lot with the views of it that the save needs.
pub(super) struct LotShape<'a> {
    pub lot: &'a PreparedLot,
    pub preview: &'a LotPreview,
    pub totals: &'a SummaryTotals,
    pub journal: &'a JournalLot,
    pub rows: Vec<RowShape<'a>>,
}

/// A checked sum of money.
pub(super) fn sum(mut values: impl Iterator<Item = Paisa>) -> Result<Paisa, ServiceError> {
    values
        .try_fold(Paisa::ZERO, |total, value| total.checked_add(value))
        .map_err(money_error)
}

/// A position as a 1-based number (lot number, line number).
pub(super) fn one_based(index: usize) -> Result<i64, ServiceError> {
    i64::try_from(index)
        .ok()
        .and_then(|i| i.checked_add(1))
        .ok_or_else(|| data("too many lots or lines"))
}

/// Lines up the draft, the preview of each lot and the journal source, and checks that they
/// describe the same lots and rows. A draft without errors always does; if not, it is a bug in
/// the calculation and nothing is saved.
pub(super) fn shape(worked: &Worked) -> Result<Vec<LotShape<'_>>, ServiceError> {
    let lots = &worked.prepared.lots;
    let previews = &worked.calc.lots;
    let journal = &worked.calc.journal.lots;
    if lots.len() != previews.len() || lots.len() != journal.len() {
        return Err(data("the lots of a بکری and its journal do not line up"));
    }
    let mut out = Vec::with_capacity(lots.len());
    for ((lot, preview), journal) in lots.iter().zip(previews).zip(journal) {
        let totals = preview
            .totals
            .as_ref()
            .ok_or_else(|| data("a lot of a بکری has no totals"))?;
        let mut rows = Vec::new();
        let mut journal_sales = journal.sales.iter();
        for (row, line) in lot.sales.iter().zip(&preview.lines) {
            if row.is_blank {
                continue;
            }
            let calc = line
                .calc
                .as_ref()
                .ok_or_else(|| data("a sale line of a بکری has no numbers"))?;
            let journal = journal_sales
                .next()
                .ok_or_else(|| data("a sale line of a بکری has no journal line"))?;
            rows.push(RowShape { row, calc, journal });
        }
        if journal_sales.next().is_some() {
            return Err(data("the journal of a lot has more lines than the lot"));
        }
        // The lot's stored totals are sums of the stored line values (money agent point 4b).
        let gross = sum(rows.iter().map(|r| r.calc.seller_amount))?;
        let comm = sum(rows.iter().map(|r| r.calc.seller_comm.amount))?;
        if gross != totals.gross || comm != totals.seller_comm {
            return Err(data("a lot's totals differ from the sum of its lines"));
        }
        out.push(LotShape {
            lot,
            preview,
            totals,
            journal,
            rows,
        });
    }
    Ok(out)
}

pub(super) fn person_columns(person: Option<&CleanPerson>) -> Option<PukhtaColumns<'_>> {
    person.map(|p| PukhtaColumns {
        name_ur: &p.name_ur,
        name_en: &p.name_en,
        phone: &p.phone,
        person_key: &p.key,
    })
}

/// A lot charge as the table keeps it. A typed (manual) charge keeps its amount only: the table
/// wants 0 in its value column (money agent point 3).
fn lot_charge_columns(
    charge: Option<&LotCharge>,
) -> Result<LotChargeColumns<'static>, ServiceError> {
    let charge = charge.ok_or_else(|| data("a lot has no worked lot charge"))?;
    Ok(LotChargeColumns {
        paisa: charge.amount.as_paisa(),
        method: charge.method.as_str(),
        value_paisa: if charge.method == LotChargeMethod::Manual {
            0
        } else {
            charge.value_paisa.as_paisa()
        },
        rule_id: charge.rule_id,
    })
}

fn sale_charge_columns(charge: &LineCharge) -> SaleChargeColumns<'static> {
    SaleChargeColumns {
        method: charge.method.as_str(),
        value_bps: charge.value_bps,
        value_paisa: charge.value_paisa.map(Paisa::as_paisa),
        rule_id: charge.rule_id,
        paisa: charge.amount.as_paisa(),
    }
}

/// The ids of the stored lines of the posted entry, by position (a position plus one is the
/// `line_no`).
pub(super) fn stored_line_ids(
    tx: &Connection,
    entry_id: i64,
    expected: usize,
) -> Result<Vec<i64>, ServiceError> {
    let stored = ledger_repo::get_entry(tx, entry_id)?
        .ok_or_else(|| data("the posted journal entry cannot be read back"))?;
    let ids: Vec<i64> = stored.lines.iter().map(|l| l.id).collect();
    let in_order = stored
        .lines
        .iter()
        .enumerate()
        .all(|(i, l)| usize::try_from(l.line_no).is_ok_and(|n| n == i.saturating_add(1)));
    if ids.len() != expected || !in_order {
        return Err(data(
            "the stored lines of the journal entry are not as built",
        ));
    }
    Ok(ids)
}

/// Everything about the posting that the rows after it need.
pub(super) struct Posting<'a> {
    pub user_id: i64,
    pub now: &'a str,
    pub bikri_id: i64,
    pub entry_id: i64,
    pub date_iso: &'a str,
    pub year: i32,
    pub month: u32,
    pub adda_id: Option<i64>,
}

/// The one-time seller of a lot, cleaned, if the lot has one.
pub(super) fn lot_person(shaped: &LotShape<'_>) -> Result<Option<CleanPerson>, ServiceError> {
    match &shaped.journal.person_key {
        Some(key) => Ok(Some(
            clean_person(&shaped.lot.pukhta)
                .filter(|p| &p.key == key)
                .ok_or_else(|| data("a one-time seller could not be cleaned to his key"))?,
        )),
        None => Ok(None),
    }
}

/// The charges of a lot that come off its seller once, in its first posting (PRD C6): the freight
/// the firm paid, مزدوری, منشیانہ, نقد, کرایہ سٹور and فون.
#[derive(Debug, Clone, Copy)]
pub(super) struct LotCharges {
    pub freight: Paisa,
    pub labour: Paisa,
    pub munshiana: Paisa,
    pub cash_paid: Paisa,
    pub store_rent: Paisa,
    pub phone: Paisa,
}

impl LotCharges {
    /// The charges of a worked first posting.
    pub(super) fn of_totals(totals: &SummaryTotals) -> Self {
        LotCharges {
            freight: totals.freight,
            labour: totals.labour,
            munshiana: totals.munshiana,
            cash_paid: totals.cash_paid,
            store_rent: totals.store_rent,
            phone: totals.phone,
        }
    }
}

/// The running totals of a lot (PRD C6 to C8, B20): خام بکری, the sum of the line commissions,
/// کل اخراجات and صافی, from the sums of its lines in force in ALL postings and the charges of the
/// first posting. The table checks the same formulas, so a number that differs from them cannot be
/// written.
///
/// Example (PRD E2E-16b): lines 26,500 and 2,800 give gross 29,300 and commission 1,590 + 168;
/// with freight 3,000, labour 250, munshiana 50, نقد 5,000 and فون 20, deductions are 10,078 and
/// net is 19,222.
pub(super) fn lot_running_totals(
    charges: &LotCharges,
    sums: LotSums,
) -> Result<edit_repo::LotTotals, ServiceError> {
    let gross = Paisa::try_from_paisa(sums.gross_paisa).map_err(money_error)?;
    let comm = Paisa::try_from_paisa(sums.comm_paisa).map_err(money_error)?;
    let deductions = sum([
        charges.freight,
        comm,
        charges.labour,
        charges.munshiana,
        charges.cash_paid,
        charges.store_rent,
        charges.phone,
    ]
    .into_iter())?;
    let net = gross.checked_sub(deductions).map_err(money_error)?;
    Ok(edit_repo::LotTotals {
        qty_sold: sums.qty_sold,
        gross_paisa: gross.as_paisa(),
        seller_comm_paisa: comm.as_paisa(),
        deductions_paisa: deductions.as_paisa(),
        net_paisa: net.as_paisa(),
    })
}

/// Whether the lot's seller is a garden's account (PRD FR-12.36).
pub(super) fn is_garden_lot(
    conn: &Connection,
    shaped: &LotShape<'_>,
) -> Result<bool, ServiceError> {
    Ok(mandi_db::gardens::of_account(conn, shaped.journal.seller_account_id.get())?.is_some())
}

/// The columns of a lot of the first posting, with its running totals. `person` is the lot's
/// one-time seller (see [`lot_person`]).
pub(super) fn lot_columns<'a>(
    conn: &Connection,
    bikri_id: i64,
    lot_no: i64,
    shaped: &'a LotShape<'a>,
    person: Option<&'a CleanPerson>,
    running: &edit_repo::LotTotals,
) -> Result<NewLot<'a>, ServiceError> {
    let lot = shaped.lot;
    let totals = shaped.totals;
    let garden = is_garden_lot(conn, shaped)?;
    let qty_received = lot
        .qty_received
        .ok_or_else(|| data("a lot without errors has no quantity"))?
        .get();
    Ok(NewLot {
        bikri_id,
        lot_no,
        seller_account_id: shaped.journal.seller_account_id.get(),
        // A trip lot names its purchase (PRD FR-12.26).
        purchase_id: lot.purchase_id,
        pukhta: person_columns(person),
        item_id: lot
            .item_id
            .ok_or_else(|| data("a lot without errors has no item"))?,
        packing_id: lot
            .packing_id
            .ok_or_else(|| data("a lot without errors has no packing"))?,
        marka_id: lot.marka_id,
        store_id: lot.store_id,
        qty_received,
        weight_received_grams: lot.weight_received.map(Grams::get),
        // The freight typed on the lot is kept even when the seller paid it (PRD E6); only
        // the deduction (`totals.freight`) is 0 then.
        freight_paisa: lot.freight.as_paisa(),
        freight_paid_by: lot.freight_paid_by.as_str(),
        labour: lot_charge_columns(shaped.preview.labour.as_ref())?,
        munshiana: lot_charge_columns(shaped.preview.munshiana.as_ref())?,
        phone: lot_charge_columns(shaped.preview.phone.as_ref())?,
        cash_paid_paisa: totals.cash_paid.as_paisa(),
        store_rent_paisa: totals.store_rent.as_paisa(),
        // A trip lot adjusts no advance: there is no seller (PRD FR-12.26). Nor does a garden's
        // lot: the garden's account holds sales money, never an advance (PRD FR-12.36). The
        // backend forces it off, whatever the screen sent.
        adjust_advance: lot.adjust_advance && lot.purchase_id.is_none() && !garden,
        qty_sold: running.qty_sold,
        gross_paisa: running.gross_paisa,
        seller_comm_paisa: running.seller_comm_paisa,
        deductions_paisa: running.deductions_paisa,
        net_paisa: running.net_paisa,
    })
}

/// The sums of the lines of a lot in this posting, for the running totals of a first save.
pub(super) fn posting_sums(shaped: &LotShape<'_>) -> Result<LotSums, ServiceError> {
    let qty_sold = shaped
        .rows
        .iter()
        .try_fold(0_i64, |total, r| total.checked_add(r.calc.qty))
        .ok_or_else(|| data("the quantity sold does not fit"))?;
    Ok(LotSums {
        qty_sold,
        gross_paisa: sum(shaped.rows.iter().map(|r| r.calc.seller_amount))?.as_paisa(),
        comm_paisa: sum(shaped.rows.iter().map(|r| r.calc.seller_comm.amount))?.as_paisa(),
    })
}

/// Adds the sale lines (numbered after the lot's last line), a stock move for each line, an
/// analytics fact for each line and, for a first posting (`with_received`), a stock move for each
/// lot. Returns the ids of the new lines, lot by lot in the order of the rows.
pub(super) fn insert_sales_and_facts(
    tx: &Connection,
    posting: &Posting<'_>,
    lots: &[LotShape<'_>],
    lot_ids: &[i64],
    with_received: bool,
) -> Result<Vec<Vec<i64>>, ServiceError> {
    let buyer_ids: Vec<i64> = lots
        .iter()
        .flat_map(|l| l.rows.iter().map(|r| r.journal.buyer_account_id.get()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let cities = repo::account_cities(tx, &buyer_ids)?;
    let mut ids = Vec::with_capacity(lots.len());
    for (shaped, lot_id) in lots.iter().zip(lot_ids) {
        if with_received {
            let qty_received = shaped
                .lot
                .qty_received
                .ok_or_else(|| data("a lot without errors has no quantity"))?
                .get();
            repo::insert_stock_move(
                tx,
                &NewStockMove {
                    lot_id: *lot_id,
                    entry_id: posting.entry_id,
                    move_date: posting.date_iso,
                    kind: StockKind::Received,
                    qty: qty_received,
                    // The lot's own "received" move; the purchase gives up its goods when they
                    // are sold (the sold move names the purchase).
                    purchase_id: None,
                },
                posting.user_id,
                posting.now,
            )?;
        }
        let first_line = edit_repo::max_line_no(tx, *lot_id)?;
        let purchase_id = shaped.lot.purchase_id;
        // The trip of an own lot, kept on every fact for the analytics filters (PRD FR-18.1).
        let venture_id = match shaped.preview.own.as_ref() {
            Some(own) => Some(own.trip.id),
            // A garden's lot: the garden, for the analytics filters (plan 17c, view I).
            None => mandi_db::gardens::of_account(tx, shaped.journal.seller_account_id.get())?
                .map(|(garden_id, _)| garden_id),
        };
        let (mut own_qty, mut own_cost) = (0_i64, Paisa::ZERO);
        let mut lot_sale_ids = Vec::with_capacity(shaped.rows.len());
        for (index, shaped_row) in shaped.rows.iter().enumerate() {
            let line_no = first_line
                .checked_add(one_based(index)?)
                .ok_or_else(|| data("too many lines"))?;
            let sale_id = insert_sale(tx, posting, *lot_id, line_no, shaped_row)?;
            let calc = shaped_row.calc;
            let buyer_id = shaped_row.journal.buyer_account_id.get();
            repo::insert_stock_move(
                tx,
                &NewStockMove {
                    lot_id: *lot_id,
                    entry_id: posting.entry_id,
                    move_date: posting.date_iso,
                    kind: StockKind::Sold,
                    qty: calc
                        .qty
                        .checked_neg()
                        .ok_or_else(|| data("the quantity does not fit"))?,
                    purchase_id,
                },
                posting.user_id,
                posting.now,
            )?;
            repo::insert_fact(
                tx,
                &NewFact {
                    journal_entry_id: posting.entry_id,
                    entry_date: posting.date_iso,
                    year: i64::from(posting.year),
                    month: i64::from(posting.month),
                    bikri_id: posting.bikri_id,
                    lot_id: *lot_id,
                    sale_id,
                    item_id: shaped
                        .lot
                        .item_id
                        .ok_or_else(|| data("a lot without errors has no item"))?,
                    seller_account_id: shaped.journal.seller_account_id.get(),
                    buyer_account_id: buyer_id,
                    pukhta_person_key: shaped_row.journal.person_key.as_deref(),
                    // A snapshot of the buyer's city on the day of the sale (PRD FR-18.6).
                    city_id: cities.get(&buyer_id).copied(),
                    adda_id: posting.adda_id,
                    qty: calc.qty,
                    seller_amount_paisa: calc.seller_amount.as_paisa(),
                    seller_comm_paisa: calc.seller_comm.amount.as_paisa(),
                    buyer_value_paisa: calc.buyer_value.as_paisa(),
                    buyer_comm_paisa: calc.buyer_comm.amount.as_paisa(),
                    laga_paisa: calc.laga.amount.as_paisa(),
                    sign: 1,
                    purchase_id,
                    venture_id,
                    cost_paisa: calc.cost_of_goods.as_paisa(),
                },
                posting.user_id,
                posting.now,
            )?;
            own_qty = own_qty
                .checked_add(calc.qty)
                .ok_or_else(|| data("the quantity does not fit"))?;
            own_cost = own_cost
                .checked_add(calc.cost_of_goods)
                .map_err(money_error)?;
            lot_sale_ids.push(sale_id);
        }
        // The trip's partners are credited the seller credit of the lot (PRD FR-12.26, C25). The
        // parts are kept here because the journal lines carry no lot.
        if let Some(own) = &shaped.preview.own {
            for credit in &own.credits {
                repo::insert_credit_part(
                    tx,
                    &NewCreditPart {
                        lot_id: *lot_id,
                        journal_entry_id: posting.entry_id,
                        partner_id: credit.partner.id,
                        share_bps: i64::from(credit.share_bps),
                        amount_paisa: credit.amount_paisa.as_paisa(),
                    },
                    posting.user_id,
                    posting.now,
                )?;
            }
        }
        // The purchase gives up what the own lot sold: نگ and cost (PRD FR-12.26, C21).
        if let Some(purchase_id) = purchase_id {
            purchases::take_sold(
                tx,
                purchase_id,
                own_qty,
                own_cost,
                posting.user_id,
                posting.now,
            )?;
        }
        ids.push(lot_sale_ids);
    }
    Ok(ids)
}

/// Adds one sale line with its snapshot (PRD FR-11.6).
fn insert_sale(
    tx: &Connection,
    posting: &Posting<'_>,
    lot_id: i64,
    line_no: i64,
    shaped: &RowShape<'_>,
) -> Result<i64, ServiceError> {
    let calc = shaped.calc;
    let person = match &shaped.journal.person_key {
        Some(key) => Some(
            clean_person(&shaped.row.pukhta)
                .filter(|p| &p.key == key)
                .ok_or_else(|| data("a one-time buyer could not be cleaned to his key"))?,
        ),
        None => None,
    };
    let id = repo::insert_sale(
        tx,
        &NewSale {
            bikri_id: posting.bikri_id,
            lot_id,
            line_no,
            journal_entry_id: posting.entry_id,
            entry_date: posting.date_iso,
            buyer_account_id: shaped.journal.buyer_account_id.get(),
            pukhta: person_columns(person.as_ref()),
            qty: calc.qty,
            weight_grams: calc.weight_grams,
            seller_rate_paisa: calc.seller_rate.as_paisa(),
            seller_amount_paisa: calc.seller_amount.as_paisa(),
            seller_comm: sale_charge_columns(&calc.seller_comm),
            buyer_rate_paisa: calc.buyer_rate.as_paisa(),
            buyer_value_paisa: calc.buyer_value.as_paisa(),
            buyer_comm: sale_charge_columns(&calc.buyer_comm),
            laga: sale_charge_columns(&calc.laga),
            buyer_total_paisa: calc.buyer_total.as_paisa(),
            rate_diff_paisa: calc.rate_diff.as_paisa(),
            payment_type: calc.payment_type.as_str(),
            note: shaped.row.note.as_deref(),
            cost_paisa: calc.cost_of_goods.as_paisa(),
        },
        posting.user_id,
        posting.now,
    )?;
    Ok(id)
}

/// The id of the journal line at a position of the layout.
fn line_id(line_ids: &[i64], position: Option<usize>) -> Result<i64, ServiceError> {
    position
        .and_then(|p| line_ids.get(p).copied())
        .ok_or_else(|| data("a journal line of the posting is missing"))
}

/// The pinned pair of each نقدی line: its bill and its own cash credit (PRD FR-12.9). Written
/// before any matching of the buyer, so the cash of today's sale never pays an older bill.
pub(super) fn pin_cash_sales(
    tx: &Connection,
    posting: &Posting<'_>,
    lots: &[LotShape<'_>],
    layout: &EntryLayout,
    line_ids: &[i64],
) -> Result<(), ServiceError> {
    for (shaped, laid) in lots.iter().zip(&layout.lots) {
        for (row, row_layout) in shaped.rows.iter().zip(&laid.sales) {
            if row.calc.payment_type == PaymentType::Cash {
                matching::pin_cash_sale(
                    tx,
                    posting.user_id,
                    posting.now,
                    &CashSalePair {
                        account_id: row.journal.buyer_account_id.get(),
                        person_key: row.journal.person_key.as_deref(),
                        bill_line_id: line_id(line_ids, row_layout.buyer_debit)?,
                        payment_line_id: line_id(line_ids, row_layout.cash_credit)?,
                        amount: row.calc.buyer_total,
                    },
                )?;
            }
        }
    }
    Ok(())
}

/// Matching after the posting of a NEW posting (PRD FR-15.4, FR-15.8, FR-12.23): the pinned pair
/// of each نقدی line, the matching of the new bills of each buyer key to his unmatched credits,
/// and the advance adjustment of each lot whose tick is on. Only for a posting that is the newest
/// line of its keys; an edit works the keys out from the start instead.
pub(super) fn match_posting(
    tx: &Connection,
    posting: &Posting<'_>,
    lots: &[LotShape<'_>],
    (entry, layout): (&BalancedEntry, &EntryLayout),
    line_ids: &[i64],
) -> Result<(), ServiceError> {
    pin_cash_sales(tx, posting, lots, layout, line_ids)?;
    // The new bills of each buyer key, in a fixed order.
    let mut credit_bills: BTreeMap<(i64, Option<String>), Vec<NewLine>> = BTreeMap::new();
    for (shaped, laid) in lots.iter().zip(&layout.lots) {
        for (row, row_layout) in shaped.rows.iter().zip(&laid.sales) {
            if row.calc.payment_type == PaymentType::Credit {
                credit_bills
                    .entry((
                        row.journal.buyer_account_id.get(),
                        row.journal.person_key.clone(),
                    ))
                    .or_default()
                    .push(NewLine {
                        line_id: line_id(line_ids, row_layout.buyer_debit)?,
                        amount: row.calc.buyer_total,
                    });
            }
        }
    }
    // A credit sale is a new bill: an unmatched credit of the buyer (a prepayment) settles it.
    for ((account_id, key), bills) in &credit_bills {
        matching::match_new_bills(
            tx,
            posting.user_id,
            posting.now,
            *account_id,
            key.as_deref(),
            posting.date_iso,
            bills,
        )?;
    }
    // Advances: only a seller credit LINE (C8) of an ordinary seller, with the tick on. A
    // negative credit is a debit line and a one-time seller has no advances.
    for (shaped, laid) in lots.iter().zip(&layout.lots) {
        let Some(position) = laid.seller_credit else {
            continue;
        };
        if shaped.lot.adjust_advance
            && shaped.journal.person_key.is_none()
            && shaped.lot.purchase_id.is_none()
            && !is_garden_lot(tx, shaped)?
        {
            // The amount adjusted is the credit of the journal line itself: both come from the
            // one calculation, and a drift would be caught here before the database trigger.
            let line = entry
                .lines()
                .get(position)
                .filter(|l| l.side == Side::Cr && l.amount == shaped.totals.seller_credit)
                .ok_or_else(|| data("the seller credit line is not the credit of its lot"))?;
            matching::adjust_advances(
                tx,
                posting.user_id,
                posting.now,
                shaped.journal.seller_account_id.get(),
                line_id(line_ids, Some(position))?,
                line.amount,
                posting.date_iso,
            )?;
        }
    }
    Ok(())
}

/// The share rows of the posting add up to the shared income of the summary (PRD FR-6.5,
/// FR-17.4). They were written by the hook inside `ledger::post`.
pub(super) fn check_shares(
    tx: &Connection,
    entry_id: i64,
    shared_income: Paisa,
) -> Result<(), ServiceError> {
    let rows = mandi_db::partners::share_rows_of_entry(tx, entry_id)?;
    let (mut income, mut expense) = (0_i64, 0_i64);
    for row in &rows {
        let slot = if row.kind == "income" {
            &mut income
        } else {
            &mut expense
        };
        *slot = slot
            .checked_add(row.amount_paisa)
            .ok_or_else(|| data("the share rows do not fit"))?;
    }
    if income != shared_income.as_paisa() || expense != 0 {
        tracing::error!(
            income,
            expense,
            shared = shared_income.as_paisa(),
            "the share rows of a بکری differ from its shared income"
        );
        return Err(data("the share rows differ from the shared income"));
    }
    Ok(())
}
