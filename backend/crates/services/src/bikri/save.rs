//! Saves a بکری (PRD FR-12.8, FR-12.9, plan 13): ONE user action, ONE transaction, all or nothing
//! (Code Standards B3, NFR-REL-1).
//!
//! The draft is worked out by `work_out`, the very function of the live preview, so the saved
//! numbers are the previewed ones (Code Standards B5). Then, inside the same transaction:
//!
//! 1. every error of the draft is returned together and nothing is written; then the cash
//!    confirmation (PRD FR-6.7);
//! 2. the بکری number (the biggest ever given plus one) and the `bikri_entries` row;
//! 3. each lot, with its final stored totals in the INSERT (the table checks that deductions are
//!    the sum of the charges and net is gross less deductions);
//! 4. the journal entry of PRD FR-12.9 through `ledger::post` (draft, lines, one update to
//!    posted; the partner share rows are written by the hook inside it);
//! 5. each sale line with its snapshot (method, value, rule, amount; PRD FR-11.6), stock moves
//!    and analytics facts;
//! 6. matching: the pinned pair of each نقدی line, then the new bills of each buyer key matched to
//!    his unmatched credits, then the advance adjustment of each lot whose tick is on;
//! 7. the audit row, and the draft of the screen is removed.
//!
//! A failure anywhere rolls the whole transaction back: no row is left, and the number is not
//! used.
//!
//! Worked numbers (PRD 4.4): Event 1 saves بکری number 1 with the journal of 12 lines, sale part
//! 31,564 = 31,564, whole entry 36,564 = 36,564, share rows 1,580 / 1,184 / 1,184, and (tick on)
//! 15,000 of رحیم's advance adjusted.

use std::collections::{BTreeMap, BTreeSet};

use mandi_db::bikri::write::{
    self as repo, LotChargeColumns, NewEntry, NewFact, NewLot, NewSale, NewStockMove,
    PukhtaColumns, SaleChargeColumns, StockKind,
};
use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::bikri::{CleanPerson, clean_person};
use mandi_domain::core::bikri::{
    BikriSaveInput, BikriSaved, LineCalc, LineCharge, LotCharge, LotChargeMethod, LotPreview,
    PaymentType, PreparedLot, PreparedSale, SummaryTotals, prepare_draft,
};
use mandi_domain::core::money::{Grams, Paisa};
use mandi_domain::ledger::bikri::{EntryLayout, JournalLot, JournalSale};
use mandi_domain::ledger::{BalancedEntry, Side};
use mandi_domain::ledger::{PostingType, SourceType};
use serde_json::json;

use super::shared::{Worked, data, switches, work_out};
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash;
use crate::context::Context;
use crate::day;
use crate::drafts;
use crate::error::{ServiceError, money_error};
use crate::ledger::{self, PostingEvent};
use crate::matching::{self, CashSalePair, NewLine};
use crate::rules;

/// One worked sale line with the three views of it that the save needs.
struct RowShape<'a> {
    row: &'a PreparedSale,
    calc: &'a LineCalc,
    journal: &'a JournalSale,
}

/// One worked lot with the views of it that the save needs.
struct LotShape<'a> {
    lot: &'a PreparedLot,
    preview: &'a LotPreview,
    totals: &'a SummaryTotals,
    journal: &'a JournalLot,
    rows: Vec<RowShape<'a>>,
}

fn sum(mut values: impl Iterator<Item = Paisa>) -> Result<Paisa, ServiceError> {
    values
        .try_fold(Paisa::ZERO, |total, value| total.checked_add(value))
        .map_err(money_error)
}

/// A position as a 1-based number (lot number, line number).
fn one_based(index: usize) -> Result<i64, ServiceError> {
    i64::try_from(index)
        .ok()
        .and_then(|i| i.checked_add(1))
        .ok_or_else(|| data("too many lots or lines"))
}

/// Lines up the draft, the preview of each lot and the journal source, and checks that they
/// describe the same lots and rows. A draft without errors always does; if not, it is a bug in
/// the calculation and nothing is saved.
fn shape(worked: &Worked) -> Result<Vec<LotShape<'_>>, ServiceError> {
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

fn person_columns(person: Option<&CleanPerson>) -> Option<PukhtaColumns<'_>> {
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
fn stored_line_ids(
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
struct Posting<'a> {
    user_id: i64,
    now: &'a str,
    bikri_id: i64,
    entry_id: i64,
    date_iso: &'a str,
    year: i32,
    month: u32,
    adda_id: Option<i64>,
}

/// Saves a بکری in one transaction (PRD FR-12.8). See the module text for the order.
///
/// # Errors
/// Every error and code of the preview, each at its field, all together and with nothing written
/// (`lots[i].sales[j].qty`, `ERR_NO_LOTS`, `ERR_NO_SALE_LINES` on `lots`, `ERR_DATE_CLOSED` on
/// `header.entry_date`, `ERR_SHARES_NOT_100` on `split`, ...);
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`) when the entry takes روکڑ below
/// 0 and `confirm_cash_below_zero` is false; `ServiceError::Db` or `ServiceError::Data` for a
/// failure the user cannot fix, which rolls everything back.
pub fn save(
    ctx: &Context,
    actor: &Actor,
    input: &BikriSaveInput,
) -> Result<BikriSaved, ServiceError> {
    let (closed_up_to, today) = day::window(ctx)?;
    let prepared = prepare_draft(&input.draft, today, closed_up_to);
    // Fetched before the write: the cache reads through the read connection.
    let rule_set = rules::rule_set(ctx)?;
    let settings = rules::overrides(ctx)?;
    let switches = switches(&settings);
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let worked = work_out(tx, prepared, &rule_set, switches)?;
        if !worked.errors.is_empty() {
            return Err(ServiceError::from(worked.errors.clone()));
        }
        cash::require_confirmed(&worked.cash, input.confirm_cash_below_zero)?;
        let (entry, layout) = worked
            .entry
            .as_ref()
            .ok_or_else(|| data("a بکری without errors has no journal entry"))?;
        let lots = shape(&worked)?;
        let header = &worked.prepared.header;
        let date = header
            .entry_date
            .ok_or_else(|| data("a بکری without errors has no date"))?;
        let date_iso = date.to_iso();
        let bill_iso = header.bill_date.unwrap_or(date).to_iso();

        // 2. The بکری.
        let number = repo::next_number(tx)?;
        let bikri_id = repo::insert_entry(
            tx,
            &NewEntry {
                number,
                book: header.book.as_deref(),
                entry_date: &date_iso,
                bill_date: &bill_iso,
                truck_no: header.truck_no.as_deref(),
                zamindar_account_id: header.zamindar_account_id,
                adda_id: header.adda_id,
                driver_name: header.driver_name.as_deref(),
                driver_phone: header.driver_phone.as_deref(),
                total_freight_paisa: header.total_freight.map(Paisa::as_paisa),
                store_id: header.store_id,
                note: header.note.as_deref(),
            },
            actor.user_id,
            &now,
        )?;

        // 3. The lots, with their final totals.
        let mut lot_ids = Vec::with_capacity(lots.len());
        for (index, shaped) in lots.iter().enumerate() {
            lot_ids.push(insert_lot(
                tx,
                actor.user_id,
                &now,
                bikri_id,
                index,
                shaped,
            )?);
        }

        // 4. The journal entry, posted through the one function.
        let posted = ledger::post(
            ctx,
            tx,
            actor.user_id,
            &PostingEvent {
                date,
                posting_type: PostingType::BikriFirst,
                source_type: SourceType::Bikri,
                source_id: bikri_id,
                narration: None,
                reverses: None,
                entry: entry.clone(),
            },
        )?;
        let line_ids = stored_line_ids(tx, posted.entry_id, entry.lines().len())?;
        for lot_id in &lot_ids {
            repo::set_lot_first_entry(tx, *lot_id, posted.entry_id, actor.user_id, &now)?;
        }

        // 5. The sale lines, stock moves and analytics facts.
        let posting = Posting {
            user_id: actor.user_id,
            now: &now,
            bikri_id,
            entry_id: posted.entry_id,
            date_iso: &date_iso,
            year: date.year(),
            month: date.month(),
            adda_id: header.adda_id,
        };
        let lines_saved = insert_sales_and_facts(tx, &posting, &lots, &lot_ids)?;

        // 6. Matching.
        match_posting(tx, &posting, &lots, (entry, layout), &line_ids)?;

        // The shares were written by the hook inside `ledger::post`; they must add up to the
        // shared income the screen showed (PRD FR-6.5).
        check_shares(tx, posted.entry_id, worked.calc.summary.shared_income)?;

        // 7. Audit and the draft.
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "create", "bikri")
                .entity_id(bikri_id)
                .new_value(json!({
                    "number": number,
                    "entry_date": date_iso,
                    "entry_id": posted.entry_id,
                    "lots": lots.len(),
                    "lines": lines_saved,
                    "gross_paisa": worked.calc.summary.gross.as_paisa(),
                    "deductions_paisa": worked.calc.summary.deductions.as_paisa(),
                    "net_paisa": worked.calc.summary.net.as_paisa(),
                    "buyer_total_paisa": worked.calc.summary.buyer_total.as_paisa(),
                    "shared_income_paisa": worked.calc.summary.shared_income.as_paisa(),
                    "cash_after_paisa": worked.cash.cash_after_paisa.as_paisa(),
                    "cash_below_zero_confirmed": worked.cash.needs_confirm,
                }))
                .into_entry(),
        )?;
        drafts::discard_in(tx, actor.user_id, drafts::BIKRI_SCREEN)?;
        Ok(BikriSaved {
            bikri_id,
            number,
            entry_id: posted.entry_id,
            preview: worked.preview(),
        })
    })
}

/// Adds one lot with the totals `calc_posting` worked out (PRD FR-11.6, B20).
fn insert_lot(
    tx: &Connection,
    user_id: i64,
    now: &str,
    bikri_id: i64,
    index: usize,
    shaped: &LotShape<'_>,
) -> Result<i64, ServiceError> {
    let lot = shaped.lot;
    let totals = shaped.totals;
    let person = match &shaped.journal.person_key {
        Some(key) => Some(
            clean_person(&lot.pukhta)
                .filter(|p| &p.key == key)
                .ok_or_else(|| data("a one-time seller could not be cleaned to his key"))?,
        ),
        None => None,
    };
    let qty_received = lot
        .qty_received
        .ok_or_else(|| data("a lot without errors has no quantity"))?
        .get();
    let qty_sold = shaped
        .rows
        .iter()
        .try_fold(0_i64, |total, r| total.checked_add(r.calc.qty))
        .ok_or_else(|| data("the quantity sold does not fit"))?;
    let id = repo::insert_lot(
        tx,
        &NewLot {
            bikri_id,
            lot_no: one_based(index)?,
            seller_account_id: shaped.journal.seller_account_id.get(),
            pukhta: person_columns(person.as_ref()),
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
            adjust_advance: lot.adjust_advance,
            qty_sold,
            gross_paisa: totals.gross.as_paisa(),
            seller_comm_paisa: totals.seller_comm.as_paisa(),
            deductions_paisa: totals.deductions.as_paisa(),
            net_paisa: totals.net.as_paisa(),
        },
        user_id,
        now,
    )?;
    Ok(id)
}

/// Adds the sale lines (numbered 1 to n, blank rows left out), a stock move for each lot and
/// each line, and an analytics fact for each line. Returns how many lines were saved.
fn insert_sales_and_facts(
    tx: &Connection,
    posting: &Posting<'_>,
    lots: &[LotShape<'_>],
    lot_ids: &[i64],
) -> Result<usize, ServiceError> {
    let buyer_ids: Vec<i64> = lots
        .iter()
        .flat_map(|l| l.rows.iter().map(|r| r.journal.buyer_account_id.get()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let cities = repo::account_cities(tx, &buyer_ids)?;
    let mut saved = 0_usize;
    for (shaped, lot_id) in lots.iter().zip(lot_ids) {
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
            },
            posting.user_id,
            posting.now,
        )?;
        for (index, shaped_row) in shaped.rows.iter().enumerate() {
            let sale_id = insert_sale(tx, posting, *lot_id, one_based(index)?, shaped_row)?;
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
                },
                posting.user_id,
                posting.now,
            )?;
            saved = saved.saturating_add(1);
        }
    }
    Ok(saved)
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
        },
        posting.user_id,
        posting.now,
    )?;
    Ok(id)
}

/// Matching after the posting (PRD FR-15.4, FR-15.8, FR-12.23): the pinned pair of each نقدی
/// line, the matching of the new bills of each buyer key to his unmatched credits, and the
/// advance adjustment of each lot whose tick is on.
fn match_posting(
    tx: &Connection,
    posting: &Posting<'_>,
    lots: &[LotShape<'_>],
    (entry, layout): (&BalancedEntry, &EntryLayout),
    line_ids: &[i64],
) -> Result<(), ServiceError> {
    let line_id = |position: Option<usize>| -> Result<i64, ServiceError> {
        position
            .and_then(|p| line_ids.get(p).copied())
            .ok_or_else(|| data("a journal line of the posting is missing"))
    };
    // The new bills of each buyer key, in a fixed order.
    let mut credit_bills: BTreeMap<(i64, Option<String>), Vec<NewLine>> = BTreeMap::new();
    for (shaped, laid) in lots.iter().zip(&layout.lots) {
        for (row, row_layout) in shaped.rows.iter().zip(&laid.sales) {
            let account_id = row.journal.buyer_account_id.get();
            match row.calc.payment_type {
                PaymentType::Cash => {
                    // Settled at once by its own cash (PRD FR-12.9): pinned to its own bill.
                    matching::pin_cash_sale(
                        tx,
                        posting.user_id,
                        posting.now,
                        &CashSalePair {
                            account_id,
                            person_key: row.journal.person_key.as_deref(),
                            bill_line_id: line_id(row_layout.buyer_debit)?,
                            payment_line_id: line_id(row_layout.cash_credit)?,
                            amount: row.calc.buyer_total,
                        },
                    )?;
                }
                PaymentType::Credit => {
                    credit_bills
                        .entry((account_id, row.journal.person_key.clone()))
                        .or_default()
                        .push(NewLine {
                            line_id: line_id(row_layout.buyer_debit)?,
                            amount: row.calc.buyer_total,
                        });
                }
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
        if shaped.lot.adjust_advance && shaped.journal.person_key.is_none() {
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
                line_id(Some(position))?,
                line.amount,
                posting.date_iso,
            )?;
        }
    }
    Ok(())
}

/// The share rows of the posting add up to the shared income of the summary (PRD FR-6.5,
/// FR-17.4). They were written by the hook inside `ledger::post`.
fn check_shares(tx: &Connection, entry_id: i64, shared_income: Paisa) -> Result<(), ServiceError> {
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
