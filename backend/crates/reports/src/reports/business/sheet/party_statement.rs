//! One party's statement on paper (plan 74 T1, PRD FR-12.47, FR-19.5, FR-19.7, FR-19.13).
//!
//! The print holds every date with an entry in a period, oldest first: the goods of the day, the
//! commission and لاگا (a seller's deductions) on one line, the day's bill, what was paid on the
//! spot, each وصولی or ادائیگی, and the balance at the end of the day; then the totals of the
//! period, the closing balance and its amount in words.
//!
//! One calculation path (Code Standards B5): the dates, the balance before and after each date and
//! the totals of the period are the statement of the screen (`mandi_services::party_ledgers`), and
//! the figures of each day are the R35 day slip's own (`figures::load`). Nothing is worked out
//! again with a new rule. What this file adds is checked against them before anything is returned,
//! and a mismatch is an error, never a printed wrong statement (money agent rule 3):
//!
//! * the balance before a day and the balance after it are the statement's, and a day starts where
//!   the one before it ended;
//! * the vouchers listed for the day are the receipts and payments the slip counts, and the rest of
//!   the day's entries are the slip's "other entries";
//! * the totals printed, each with its sign, carry the opening balance to the closing balance.
//!
//! A date with no posted line is not a date of the statement. The one case is a seller's lot whose net
//! is exactly 0 with no نقد: it posts nothing on him, so it is not listed, and the sold and deduction
//! totals are each short by the same amount; the balances are right (PRD FR-12.7).
//!
//! Example (PRD FR-12.47): a buyer, 05-11-2026 bought 30,000 and paid 0; 06-11-2026 bought 21,360 and
//! paid 21,360 on the spot. Opening 0. The first day closes at 30,000 بنام, the second at 30,000
//! بنام. Totals: bought 51,360, paid on the spot 21,360; closing 30,000 بنام, in words.

use std::collections::HashMap;

use mandi_core::accounts::{Balance, BalanceSide};
use mandi_core::auth::Language;
use mandi_core::bikri::SaleGrade;
use mandi_core::money::Paisa;
use mandi_core::party_ledgers::{LedgerSide, PartyLedgerDay, PartyLedgerStatementWhole};
use mandi_core::printing::Paper;
use mandi_db::Db;
use mandi_db::rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::activity::SAME_DAY_PAIR;
use super::figures::{BuyerDay, SellerDay, Who, load};
use crate::error::ReportsError;
use crate::print::load::load_grouping;
use crate::print::{AmountRole, PrintAmount, PrintFact, PrintParty, PrintWords, TextDir};
use crate::reports::common::{add, load_party, paisa};
use crate::reports::ids;
use crate::reports::model::Names;
use crate::reports::print_data::{Fmt, PrintOptions, fact};

/// Fluent message ID of the title of the print and of its header.
pub const TITLE_ID: &str = "print-statement-title";
/// The day's bill of a buyer, and the day's net sale of a seller. A seller whose deductions are more
/// than his sales (PRD FR-12.7, E28) has the label `print-seller-owes` and the amount without a sign,
/// as on his bill.
const BILL_BUYER: &str = "print-statement-bill-buyer";
const BILL_SELLER: &str = "print-statement-bill-seller";
const SELLER_OWES: &str = "print-seller-owes";
/// What was entered that day, one label for each kind (the sign is in the label).
const ENTRY_RECEIPT: &str = "print-statement-entry-receipt";
const ENTRY_PAYMENT: &str = "print-statement-entry-payment";
const ENTRY_ADVANCE: &str = "print-statement-entry-advance";
const ENTRY_OTHER_DEBIT: &str = "print-statement-entry-other-debit";
const ENTRY_OTHER_CREDIT: &str = "print-statement-entry-other-credit";
/// The totals of the period, in the order they are printed.
const TOTAL_BOUGHT: &str = "print-statement-total-bought";
const TOTAL_SOLD: &str = "print-statement-total-sold";
/// A buyer's commission and لاگا, which are already inside "bought". A seller's deductions (commission,
/// freight, labour, munshiana, phone, store rent), which are taken off "sold"; the نقد is not one.
const TOTAL_CHARGES: &str = "print-statement-total-charges";
const TOTAL_DEDUCTIONS: &str = "print-statement-total-deductions";
/// What a buyer paid on the spot, and the نقد given to a seller with his lots.
const TOTAL_ON_SPOT: &str = "print-statement-total-paid-on-spot";
const TOTAL_CASH: &str = "print-statement-total-cash";
const TOTAL_RECEIVED: &str = "print-statement-total-received";
/// ادائیگی and پیشگی are separate totals: an advance is not a payment.
const TOTAL_PAID: &str = "print-statement-total-paid";
const TOTAL_ADVANCED: &str = "print-statement-total-advanced";
const TOTAL_OTHER_DEBIT: &str = "print-statement-total-other-debit";
const TOTAL_OTHER_CREDIT: &str = "print-statement-total-other-credit";
/// The charges of a day, as the bills write them (the ids of `print::ids`).
const BUYER_COMMISSION: &str = "print-buyer-commission";
const LAGA: &str = "print-laga";
const COMMISSION: &str = "print-commission";
const FREIGHT: &str = "print-freight";
const LABOUR: &str = "print-labour";
const MUNSHIANA: &str = "print-munshiana";
const PHONE_CHARGE: &str = "print-phone-charge";
const STORE_RENT: &str = "print-store-rent";
/// The role words of the party block.
const PARTY_BUYER: &str = "print-party-buyer";
const PARTY_SELLER: &str = "print-party-seller";

/// Input of `party_ledgers_statement_print_data`: the party, the period as typed (`DD-MM-YYYY` or a
/// date shortcut, both required), the print language and the paper (A4 when left out).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct StatementPrintDataInput {
    /// The party: a buyer or a seller.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The first day of the period.
    pub from: String,
    /// The last day of the period.
    pub to: String,
    /// The print language (PRD FR-7.36): names, words and digits follow it.
    pub language: Language,
    /// The paper. Missing means A4.
    #[serde(default)]
    #[ts(optional)]
    pub paper: Option<Paper>,
}

/// The side a balance stands on: بنام (he owes the shop), جمع (the shop owes him), or none at 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum StatementSide {
    /// بنام.
    Debit,
    /// جمع.
    Credit,
    /// The balance is 0.
    None,
}

impl From<BalanceSide> for StatementSide {
    fn from(side: BalanceSide) -> Self {
        match side {
            BalanceSide::Dr => StatementSide::Debit,
            BalanceSide::Cr => StatementSide::Credit,
            BalanceSide::Zero => StatementSide::None,
        }
    }
}

/// A balance as printed: the amount and its side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementBalance {
    /// Whole rupees, formatted for print.
    pub amount: String,
    /// Its side.
    pub side: StatementSide,
}

/// Whose statement it is: it picks the words (bought or sold, the day's bill or the net sale).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum StatementKind {
    /// A buyer.
    Buyer,
    /// A seller.
    Seller,
}

/// One line of goods of a day: "آم 20 × 1,000 = 20,000". Lines of the same item, quality and rate are
/// one line (their quantities and stored amounts add up); a line of an item sold by weight is never
/// joined to another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementItem {
    /// The item in the print language.
    pub name: String,
    /// The quality of the goods (PRD FR-12.42): the template writes its Fluent word after the item.
    pub grade: Option<SaleGrade>,
    /// Quantity in نگ.
    pub qty: String,
    /// The weight in kg of an item sold by weight, with its rate for one kilo. Null otherwise.
    pub weight_kg: Option<String>,
    /// The rate.
    pub rate: String,
    /// The amount: a buyer's goods value, or a seller's seller amount (the stored value).
    pub amount: String,
}

/// A وصولی, an ادائیگی, a پیشگی or another entry of the day. The label says which, and with it the
/// side: a receipt is a credit, a payment and an advance are debits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementEntry {
    /// Fluent message ID of the kind.
    pub label_id: String,
    /// The amount in whole rupees, formatted for print.
    pub amount: String,
    /// The narration of the voucher as typed. Null if none.
    pub note: Option<String>,
}

/// One date with an entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementDay {
    /// The day, `DD-MM-YYYY`.
    pub date: String,
    /// The lines of goods.
    pub items: Vec<StatementItem>,
    /// A buyer's commission and لاگا, a seller's commission and other deductions: only those that are
    /// not 0, in this order.
    pub charges: Vec<PrintAmount>,
    /// The day's bill (a buyer) or net sale (a seller). Null when the day has no sale line.
    pub bill: Option<PrintAmount>,
    /// What he paid on the spot (a buyer), or the نقد given to him with his lots (a seller). Null when
    /// 0.
    pub paid_on_spot: Option<String>,
    /// Each وصولی, ادائیگی and پیشگی of the day, then the other entries (one for each side).
    pub entries: Vec<StatementEntry>,
    /// The balance at the end of the day.
    pub closing: StatementBalance,
}

/// Everything a template needs to print one party's statement for a period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementPrintData {
    /// The print language.
    pub language: Language,
    /// The writing direction.
    pub dir: TextDir,
    /// The paper class.
    pub paper: Paper,
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The period and the party, the facts under the title (FR-19.10). They go to
    /// `reports_print_header` with the title.
    pub filters: Vec<PrintFact>,
    /// A buyer or a seller.
    pub kind: StatementKind,
    /// Who the statement is for.
    pub party: PrintParty,
    /// The first day of the period, `DD-MM-YYYY`.
    pub period_from: String,
    /// The last day of the period, `DD-MM-YYYY`.
    pub period_to: String,
    /// The balance before the first day of the period.
    pub opening: StatementBalance,
    /// Every date with an entry, oldest first.
    pub days: Vec<StatementDay>,
    /// The totals of the period, in the order they are printed: only those that are not 0, and the
    /// first (bought or sold) always.
    pub totals: Vec<PrintAmount>,
    /// The balance at the end of the period.
    pub closing: StatementBalance,
    /// The closing amount in words (PRD FR-19.7). Null when the closing is 0.
    pub words: Option<PrintWords>,
}

fn data_error(text: &str) -> ReportsError {
    ReportsError::Data(text.to_owned())
}

/// A stored balance (an amount and its side) as the signed ledger net: debit positive.
fn signed(amount: Paisa, side: BalanceSide) -> Result<Paisa, ReportsError> {
    Ok(match side {
        BalanceSide::Dr => amount,
        BalanceSide::Cr => amount.checked_neg()?,
        BalanceSide::Zero => Paisa::ZERO,
    })
}

fn balance_of(
    fmt: &Fmt,
    amount: Paisa,
    side: BalanceSide,
) -> Result<StatementBalance, ReportsError> {
    Ok(StatementBalance {
        amount: fmt.style.money(amount)?,
        side: side.into(),
    })
}

/// One posted line of the party that is not a sale posting and not half of a same-day pair.
#[derive(Debug, Clone)]
struct EntryLine {
    date: String,
    posting_type: String,
    dr: Paisa,
    cr: Paisa,
    note: Option<String>,
}

/// The lines of the party from `from` to `to` that the day's sale part does not own: the vouchers
/// (receipts, payments, advances) and everything else posted on him, without the two halves of a
/// same-day cancel or edit (the same exclusion as the R35 slip, `SAME_DAY_PAIR`). Oldest first.
fn entry_lines(
    conn: &Connection,
    account_id: i64,
    from: &str,
    to: &str,
) -> Result<Vec<EntryLine>, ReportsError> {
    let mut statement = conn.prepare_cached(&entry_lines_sql())?;
    let rows = statement
        .query_map(params![account_id, from, to], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (date, posting_type, dr, cr, note) in rows {
        out.push(EntryLine {
            date,
            posting_type,
            dr: paisa(dr)?,
            cr: paisa(cr)?,
            note,
        });
    }
    Ok(out)
}

/// The query of [`entry_lines`], for the query plan test (Code Standards B18).
#[doc(hidden)]
#[must_use]
pub fn entry_lines_sql() -> String {
    format!(
        "SELECT e.entry_date, e.posting_type, l.dr_paisa, l.cr_paisa,
                COALESCE(NULLIF(TRIM(v.narration), ''), NULLIF(TRIM(l.narration), ''))
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         LEFT JOIN vouchers v ON e.source_type = 'voucher' AND v.id = e.source_id
         WHERE l.account_id = ?1 AND e.status = 'posted'
           AND e.entry_date >= ?2 AND e.entry_date <= ?3
           AND NOT (e.source_type = 'bikri' AND e.posting_type IN ('bikri_first', 'bikri_later'))
           AND NOT ({SAME_DAY_PAIR})
         ORDER BY e.entry_date, e.id, l.line_no"
    )
}

/// What the entries of one day add up to, by kind.
#[derive(Default)]
struct EntrySums {
    /// Credits of receipts.
    receipts: Paisa,
    /// Debits of payments.
    payments: Paisa,
    /// Debits of advances.
    advances: Paisa,
    other_debit: Paisa,
    other_credit: Paisa,
}

impl EntrySums {
    /// Debits less credits of everything except the receipts' credits: what the slip calls "other
    /// entries" of a buyer (a receipt is counted as paid).
    fn net_without_receipts(&self) -> Result<Paisa, ReportsError> {
        Ok(self
            .payments
            .checked_add(self.advances)?
            .checked_add(self.other_debit)?
            .checked_sub(self.other_credit)?)
    }
}

/// The entries of one day: each voucher on its own line, then the other lines as at most one entry
/// for each side.
fn day_entries(
    fmt: &Fmt,
    lines: &[&EntryLine],
) -> Result<(Vec<StatementEntry>, EntrySums), ReportsError> {
    let mut entries = Vec::new();
    let mut sums = EntrySums::default();
    for line in lines {
        let one = |label: &str, amount: Paisa| -> Result<StatementEntry, ReportsError> {
            Ok(StatementEntry {
                label_id: label.to_owned(),
                amount: fmt.style.money(amount)?,
                note: line.note.clone(),
            })
        };
        // A line has one side only (a CHECK of the table).
        match (
            line.posting_type.as_str(),
            line.dr > Paisa::ZERO,
            line.cr > Paisa::ZERO,
        ) {
            ("receipt", false, true) => {
                add(&mut sums.receipts, line.cr)?;
                entries.push(one(ENTRY_RECEIPT, line.cr)?);
            }
            ("payment", true, false) => {
                add(&mut sums.payments, line.dr)?;
                entries.push(one(ENTRY_PAYMENT, line.dr)?);
            }
            ("advance", true, false) => {
                add(&mut sums.advances, line.dr)?;
                entries.push(one(ENTRY_ADVANCE, line.dr)?);
            }
            (_, true, false) => add(&mut sums.other_debit, line.dr)?,
            (_, false, true) => add(&mut sums.other_credit, line.cr)?,
            _ => return Err(data_error("a journal line has no amount, or both sides")),
        }
    }
    for (label, amount) in [
        (ENTRY_OTHER_DEBIT, sums.other_debit),
        (ENTRY_OTHER_CREDIT, sums.other_credit),
    ] {
        if amount != Paisa::ZERO {
            entries.push(StatementEntry {
                label_id: label.to_owned(),
                amount: fmt.style.money(amount)?,
                note: None,
            });
        }
    }
    Ok((entries, sums))
}

/// One sale line as the statement writes it. Lines are joined only within one packing, so the
/// quantities of a joined line are of one unit.
struct GoodsLine<'a> {
    item_id: i64,
    packing_id: i64,
    item: &'a Names,
    grade: Option<SaleGrade>,
    qty: i64,
    weight_grams: Option<i64>,
    rate: Paisa,
    amount: Paisa,
}

/// The lines of goods of a day: lines of one item, packing, quality and rate are joined (a line of an item
/// sold by weight is never joined), in the order they first appear. The amounts are the stored
/// amounts added up; nothing is multiplied again.
fn goods_items(fmt: &Fmt, lines: &[GoodsLine<'_>]) -> Result<Vec<StatementItem>, ReportsError> {
    struct Joined {
        item_id: i64,
        packing_id: i64,
        name: String,
        grade: Option<SaleGrade>,
        rate: Paisa,
        qty: i64,
        weight_grams: Option<i64>,
        amount: Paisa,
    }
    let mut joined: Vec<Joined> = Vec::new();
    for line in lines {
        let same = joined.iter_mut().find(|j| {
            j.weight_grams.is_none()
                && line.weight_grams.is_none()
                && j.item_id == line.item_id
                && j.packing_id == line.packing_id
                && j.grade == line.grade
                && j.rate == line.rate
        });
        match same {
            Some(one) => {
                one.qty = one
                    .qty
                    .checked_add(line.qty)
                    .ok_or_else(|| data_error("the quantities do not fit"))?;
                add(&mut one.amount, line.amount)?;
            }
            None => joined.push(Joined {
                item_id: line.item_id,
                packing_id: line.packing_id,
                name: fmt.names(line.item),
                grade: line.grade,
                rate: line.rate,
                qty: line.qty,
                weight_grams: line.weight_grams,
                amount: line.amount,
            }),
        }
    }
    joined
        .into_iter()
        .map(|j| {
            Ok(StatementItem {
                name: j.name,
                grade: j.grade,
                qty: fmt.style.count(j.qty),
                weight_kg: j
                    .weight_grams
                    .map(|grams| fmt.style.weight(grams))
                    .transpose()?,
                rate: fmt.style.money(j.rate)?,
                amount: fmt.style.money(j.amount)?,
            })
        })
        .collect()
}

fn charge(
    fmt: &Fmt,
    out: &mut Vec<PrintAmount>,
    label: &str,
    amount: Paisa,
) -> Result<(), ReportsError> {
    if amount != Paisa::ZERO {
        out.push(PrintAmount {
            label_id: label.to_owned(),
            amount: fmt.style.money(amount)?,
            role: AmountRole::Line,
        });
    }
    Ok(())
}

/// What one day adds to the totals of the period.
#[derive(Default)]
struct DaySums {
    /// A buyer's bills; a seller's gross sales.
    sale: Paisa,
    /// A buyer's commission and لاگا; a seller's deductions.
    charges: Paisa,
    /// Paid on the spot (a buyer); نقد (a seller).
    on_spot: Paisa,
    received: Paisa,
    /// ادائیگی: money the shop paid out on the party's account.
    paid: Paisa,
    /// پیشگی: advances given to a seller.
    advanced: Paisa,
    other_debit: Paisa,
    other_credit: Paisa,
}

impl DaySums {
    fn plus(&mut self, other: &DaySums) -> Result<(), ReportsError> {
        add(&mut self.sale, other.sale)?;
        add(&mut self.charges, other.charges)?;
        add(&mut self.on_spot, other.on_spot)?;
        add(&mut self.received, other.received)?;
        add(&mut self.paid, other.paid)?;
        add(&mut self.advanced, other.advanced)?;
        add(&mut self.other_debit, other.other_debit)?;
        add(&mut self.other_credit, other.other_credit)?;
        Ok(())
    }
}

/// The money paid out (payments), the advances given, and the receipts of one day.
fn entry_sums(entries: &EntrySums) -> (Paisa, Paisa, Paisa) {
    (entries.payments, entries.advances, entries.receipts)
}

/// A buyer's day on paper, checked against the slip's figures.
fn buyer_print_day(
    fmt: &Fmt,
    date: &str,
    row: &PartyLedgerDay,
    figure: &BuyerDay,
    lines: &[&EntryLine],
) -> Result<(StatementDay, DaySums), ReportsError> {
    let goods: Vec<GoodsLine<'_>> = figure
        .lines
        .iter()
        .map(|l| GoodsLine {
            item_id: l.goods.item_id,
            packing_id: l.goods.packing_id,
            item: &l.goods.item,
            grade: l.grade,
            qty: l.qty,
            weight_grams: l.weight_grams,
            rate: l.rate,
            amount: l.value,
        })
        .collect();
    let mut on_spot = Paisa::ZERO;
    for line in &figure.lines {
        add(&mut on_spot, line.paid)?;
    }
    let (entries, entry) = day_entries(fmt, lines)?;
    // The slip counts a buyer's receipts as paid, and everything else posted on him as "other".
    if figure.paid != on_spot.checked_add(entry.receipts)? {
        return Err(data_error(
            "the receipts of a day are not what the slip counts",
        ));
    }
    if figure.other != entry.net_without_receipts()? {
        return Err(data_error(
            "the other entries of a day are not what the slip counts",
        ));
    }
    let mut charges = Vec::new();
    charge(fmt, &mut charges, BUYER_COMMISSION, figure.commission)?;
    charge(fmt, &mut charges, LAGA, figure.laga)?;
    let has_sale = !figure.lines.is_empty();
    let bill = has_sale
        .then(|| {
            Ok::<_, ReportsError>(PrintAmount {
                label_id: BILL_BUYER.to_owned(),
                amount: fmt.style.money(figure.total)?,
                role: AmountRole::Total,
            })
        })
        .transpose()?;
    let (paid, advanced, received) = entry_sums(&entry);
    let sums = DaySums {
        sale: figure.total,
        charges: figure.commission.checked_add(figure.laga)?,
        on_spot,
        received,
        paid,
        advanced,
        other_debit: entry.other_debit,
        other_credit: entry.other_credit,
    };
    let day = StatementDay {
        date: fmt.style.date(date)?,
        items: goods_items(fmt, &goods)?,
        charges,
        bill,
        paid_on_spot: (on_spot != Paisa::ZERO)
            .then(|| fmt.style.money(on_spot))
            .transpose()?,
        entries,
        closing: balance_of(fmt, row.balance_paisa, row.balance_side)?,
    };
    Ok((day, sums))
}

/// A seller's day on paper, checked against the slip's figures.
fn seller_print_day(
    fmt: &Fmt,
    date: &str,
    row: &PartyLedgerDay,
    figure: &SellerDay,
    lines: &[&EntryLine],
) -> Result<(StatementDay, DaySums), ReportsError> {
    let goods: Vec<GoodsLine<'_>> = figure
        .lines
        .iter()
        .map(|l| GoodsLine {
            item_id: l.goods.item_id,
            packing_id: l.goods.packing_id,
            item: &l.goods.item,
            grade: l.grade,
            qty: l.qty,
            weight_grams: l.weight_grams,
            rate: l.rate,
            amount: l.amount,
        })
        .collect();
    let (entries, entry) = day_entries(fmt, lines)?;
    // The slip counts a seller's payments as paid, and everything else posted on him as "other",
    // his receipts (an advance he returns) among them. The نقد is what is left of what the slip
    // calls paid.
    let on_spot = figure.paid.checked_sub(entry.payments)?;
    if on_spot.is_negative() {
        return Err(data_error(
            "the payments of a day are more than the slip's paid",
        ));
    }
    if figure.other
        != entry
            .net_without_receipts()?
            .checked_sub(entry.payments)?
            .checked_sub(entry.receipts)?
    {
        return Err(data_error(
            "the other entries of a day are not what the slip counts",
        ));
    }
    let mut deductions = Paisa::ZERO;
    let mut charges = Vec::new();
    for (label, amount) in [
        (COMMISSION, figure.commission),
        (FREIGHT, figure.freight),
        (LABOUR, figure.labour),
        (MUNSHIANA, figure.munshiana),
        (PHONE_CHARGE, figure.phone),
        (STORE_RENT, figure.store_rent),
    ] {
        add(&mut deductions, amount)?;
        charge(fmt, &mut charges, label, amount)?;
    }
    // `sale` is the negative of the net sale, so the identity holds in the ledger's own sign.
    if figure.sale != deductions.checked_sub(figure.gross)? {
        return Err(data_error(
            "the net sale of a day is not its sales less its deductions",
        ));
    }
    let has_sale = !figure.lines.is_empty() || figure.sale != Paisa::ZERO;
    let bill = has_sale
        .then(|| {
            let (label, shown) = if figure.sale <= Paisa::ZERO {
                (BILL_SELLER, figure.sale.checked_neg()?)
            } else {
                (SELLER_OWES, figure.sale)
            };
            Ok::<_, ReportsError>(PrintAmount {
                label_id: label.to_owned(),
                amount: fmt.style.money(shown)?,
                role: AmountRole::Total,
            })
        })
        .transpose()?;
    let (paid, advanced, received) = entry_sums(&entry);
    let sums = DaySums {
        sale: figure.gross,
        charges: deductions,
        on_spot,
        received,
        paid,
        advanced,
        other_debit: entry.other_debit,
        other_credit: entry.other_credit,
    };
    let day = StatementDay {
        date: fmt.style.date(date)?,
        items: goods_items(fmt, &goods)?,
        charges,
        bill,
        paid_on_spot: (on_spot != Paisa::ZERO)
            .then(|| fmt.style.money(on_spot))
            .transpose()?,
        entries,
        closing: balance_of(fmt, row.balance_paisa, row.balance_side)?,
    };
    Ok((day, sums))
}

fn total(
    fmt: &Fmt,
    out: &mut Vec<PrintAmount>,
    label: &str,
    amount: Paisa,
    always: bool,
) -> Result<(), ReportsError> {
    if always || amount != Paisa::ZERO {
        out.push(PrintAmount {
            label_id: label.to_owned(),
            amount: fmt.style.money(amount)?,
            role: AmountRole::Line,
        });
    }
    Ok(())
}

/// The statement of one party for a period, laid out for a print language. Read only.
///
/// `whole` is the statement of the services for the period, every date of it. The rows are checked
/// against the R35 figures of each day and the totals against the balances before anything is
/// returned.
///
/// # Errors
/// A database error; `ReportsError::Data` when the statement, the slips of its days and the entries
/// of the party do not agree; `ReportsError::Money` if a sum does not fit.
pub fn party_statement_print_data(
    db: &Db,
    whole: &PartyLedgerStatementWhole,
    input: &StatementPrintDataInput,
) -> Result<StatementPrintData, ReportsError> {
    let paper = input.paper.unwrap_or(Paper::A4);
    let statement = &whole.statement;
    let account_id = statement.head.account_id;
    if input.account_id != account_id {
        return Err(data_error(
            "the statement is not the one of the party asked for",
        ));
    }
    let kind = match statement.head.kind {
        LedgerSide::Buyer => StatementKind::Buyer,
        LedgerSide::Seller => StatementKind::Seller,
    };
    // Oldest first, the order of the print.
    let rows: Vec<&PartyLedgerDay> = statement.rows.iter().rev().collect();
    let dates: Vec<String> = rows.iter().map(|r| r.date.clone()).collect();
    db.read(|conn| {
        let grouping = load_grouping(conn)?;
        let fmt = Fmt::new(&PrintOptions {
            language: input.language,
            grouping,
            paper,
        });
        let figures = load(
            conn,
            &dates,
            Some(&Who {
                account: account_id,
                person: None,
                buyer: kind == StatementKind::Buyer,
            }),
        )?;
        let lines = match (dates.first(), dates.last()) {
            (Some(first), Some(last)) => entry_lines(conn, account_id, first, last)?,
            _ => Vec::new(),
        };
        let mut by_day: HashMap<&str, Vec<&EntryLine>> = HashMap::new();
        for line in &lines {
            by_day.entry(line.date.as_str()).or_default().push(line);
        }
        let party_ref = load_party(conn, account_id)?
            .ok_or_else(|| data_error("the party of a statement is not on the books"))?;

        let opening = signed(statement.opening_paisa, statement.opening_side)?;
        let closing = signed(statement.closing_paisa, statement.closing_side)?;
        let mut days = Vec::with_capacity(rows.len());
        let mut sums = DaySums::default();
        let mut before = opening;
        for row in &rows {
            // A day starts where the one before it ended: nothing happens between two dates.
            if signed(row.previous_paisa, row.previous_side)? != before {
                return Err(data_error(
                    "a day of the statement does not start where the last ended",
                ));
            }
            let after = signed(row.balance_paisa, row.balance_side)?;
            let key = (row.date.clone(), account_id, String::new());
            let day_lines = by_day.get(row.date.as_str()).map_or(&[][..], Vec::as_slice);
            let (day, day_sums) = match kind {
                StatementKind::Buyer => {
                    let figure = figures
                        .buyers
                        .get(&key)
                        .ok_or_else(|| data_error("a date of the statement has no slip"))?;
                    if figure.previous != before || figure.new != after {
                        return Err(data_error(
                            "the statement and the day slip of a buyer differ",
                        ));
                    }
                    buyer_print_day(&fmt, &row.date, row, figure, day_lines)?
                }
                StatementKind::Seller => {
                    let figure = figures
                        .sellers
                        .get(&key)
                        .ok_or_else(|| data_error("a date of the statement has no slip"))?;
                    if figure.previous != before || figure.new != after {
                        return Err(data_error(
                            "the statement and the day slip of a seller differ",
                        ));
                    }
                    seller_print_day(&fmt, &row.date, row, figure, day_lines)?
                }
            };
            sums.plus(&day_sums)?;
            days.push(day);
            before = after;
        }
        if before != closing {
            return Err(data_error(
                "the statement does not end on its closing balance",
            ));
        }
        // The totals printed, each with its sign, carry the opening to the closing.
        let effect = match kind {
            StatementKind::Buyer => sums
                .sale
                .checked_sub(sums.on_spot)?
                .checked_sub(sums.received)?,
            StatementKind::Seller => sums
                .charges
                .checked_sub(sums.sale)?
                .checked_add(sums.on_spot)?
                .checked_sub(sums.received)?,
        }
        .checked_add(sums.paid)?
        .checked_add(sums.advanced)?
        .checked_add(sums.other_debit)?
        .checked_sub(sums.other_credit)?;
        if opening.checked_add(effect)? != closing {
            return Err(data_error(
                "the totals of a statement do not carry its opening to its closing",
            ));
        }

        let mut totals = Vec::new();
        let sale_label = match kind {
            StatementKind::Buyer => TOTAL_BOUGHT,
            StatementKind::Seller => TOTAL_SOLD,
        };
        total(&fmt, &mut totals, sale_label, sums.sale, true)?;
        let (charges_label, on_spot_label) = match kind {
            StatementKind::Buyer => (TOTAL_CHARGES, TOTAL_ON_SPOT),
            StatementKind::Seller => (TOTAL_DEDUCTIONS, TOTAL_CASH),
        };
        total(&fmt, &mut totals, charges_label, sums.charges, false)?;
        total(&fmt, &mut totals, on_spot_label, sums.on_spot, false)?;
        total(&fmt, &mut totals, TOTAL_RECEIVED, sums.received, false)?;
        total(&fmt, &mut totals, TOTAL_PAID, sums.paid, false)?;
        total(&fmt, &mut totals, TOTAL_ADVANCED, sums.advanced, false)?;
        total(
            &fmt,
            &mut totals,
            TOTAL_OTHER_DEBIT,
            sums.other_debit,
            false,
        )?;
        total(
            &fmt,
            &mut totals,
            TOTAL_OTHER_CREDIT,
            sums.other_credit,
            false,
        )?;

        let closing_balance = Balance::from_net(closing)?;
        let words = (closing_balance.amount_paisa != Paisa::ZERO)
            .then(|| fmt.words(closing_balance.amount_paisa))
            .transpose()?;
        let party = PrintParty {
            role_id: match kind {
                StatementKind::Buyer => PARTY_BUYER,
                StatementKind::Seller => PARTY_SELLER,
            }
            .to_owned(),
            code: Some(party_ref.code.clone()),
            short_code: party_ref.short_code.clone(),
            name: fmt.name(&party_ref.name_ur, &party_ref.name_en),
            tag: party_ref.tag.clone(),
            phone: None,
        };
        let filters = vec![
            fact(ids::FILTER_FROM, fmt.style.date(&whole.from)?),
            fact(ids::FILTER_TO, fmt.style.date(&whole.to)?),
            fact(ids::FILTER_PARTY, fmt.party_text(&party_ref)),
        ];
        Ok(StatementPrintData {
            language: input.language,
            dir: TextDir::of(input.language),
            paper,
            title_id: TITLE_ID.to_owned(),
            filters,
            kind,
            party,
            period_from: fmt.style.date(&whole.from)?,
            period_to: fmt.style.date(&whole.to)?,
            opening: balance_of(&fmt, statement.opening_paisa, statement.opening_side)?,
            days,
            totals,
            closing: balance_of(&fmt, statement.closing_paisa, statement.closing_side)?,
            words,
        })
    })
}
