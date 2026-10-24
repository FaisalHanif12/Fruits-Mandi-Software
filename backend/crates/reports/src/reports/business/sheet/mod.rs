//! R35 پارٹی کی روزانہ پرچی (the party day sheet) (PRD 19.2, FR-19.12, FR-19.10, E2E-43).
//!
//! The collector takes a slip for each party to the shops. For one day (the "as of" day, today
//! when none is given) the report has two layouts:
//!
//! * **All parties, one table** (no party picked). Two tables on one page, buyers first and
//!   sellers second, each with a heading row that carries its totals. One row for each party
//!   who has a balance or an entry that day, sorted by his Urdu name: the short code, the party
//!   (name with tag), سابقہ (the balance at the start of the day), بنام and جمع (everything
//!   debited and credited that day), بقایا (the balance at the end), and an empty column for
//!   the collector's note. Every row, every heading and the total row hold
//!   `سابقہ + بنام − جمع = بقایا` (signed: a debit balance is positive).
//! * **One party's slip** (a party picked). A buyer's slip lists the lines of the day (تعداد,
//!   قسم مال, ریٹ and the amount with commission and لاگا, C5) and, below, the lines of PRD
//!   FR-19.12: total نگ, previous balance, what he was charged that day, what he paid on the
//!   spot (the cash part of a نقدی line), what he paid that day (وصولی), and the balance that
//!   remains. A seller's slip lists the credit lines of his بکری that day with their lot's item
//!   and quantity, and below: previous, sold, نقد given on the spot, paid that day, remaining.
//!
//! The figures are the posted ledger, the same sums as R20, R13 and the trial balance: the
//! balance at the start of the day is every posted line dated before it, and the debits and
//! credits of the day are the posted lines dated that day. The two entries of a same-day cancel
//! or edit (the posting and its reversal) are both left out of the day's columns, because they
//! cancel each other and a cancelled bill is not what the party bought. The balances do not
//! change by this, so the identity above always holds. Where a slip names a charge or a payment
//! it is worked out from the stored sale lines and the ledger; nothing is calculated again with
//! today's rules (Code Standards B20). A month close writes no line on a buyer or a seller.
//!
//! Example (E2E-43, 10-10-26): اکرم previous 0, bought 12,880 (کیلا 20 پیٹی @ 600), paid that day
//! 10,000, remaining 2,880 بنام; the buyers' remaining total is 21,564, the total of R20.

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{
    add, balance_cell, col, count_of, load_parties, names_cell, paisa, pukhta_party, text_cell,
};
use crate::reports::filters::Typed;
use crate::reports::grouped_sums::pukhta_names;
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, PartyRef, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind,
    SummaryLine,
};
use mandi_core::accounts::Balance;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_core::text::UrduCollator;
use mandi_db::rusqlite::{Connection, params};
use std::collections::HashMap;

mod activity;
mod lines;
mod slip;
mod table;

/// The control codes of the two kinds of party the sheet is for (PRD FR-10.2).
const BUYER_CONTROL: &str = "0304";

const SELLER_CONTROL: &str = "03011";

/// A line of the footer.
fn line(label: &str, cell: Cell) -> SummaryLine {
    SummaryLine {
        label_id: label.to_owned(),
        cell,
    }
}

/// A money line that is left blank when it is 0 (the collector fills it in by hand).
fn blank_if_zero(amount: Paisa) -> Cell {
    if amount == Paisa::ZERO {
        Cell::Empty
    } else {
        Cell::Money { paisa: amount }
    }
}

/// Builds R35.
///
/// # Errors
/// `ReportsError::Rejected` for a party that is neither a buyer nor a seller, or a one-time
/// account without his person; a database error; `ReportsError::Money` if a stored amount is not
/// whole rupees; `ReportsError::Data` if the sale lines or credit lines of the day do not agree
/// with the ledger of the party.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let today = typed
        .today
        .ok_or_else(|| ReportsError::Data("the report has no today".to_owned()))?;
    let day = typed.as_of.clone().unwrap_or_else(|| today.to_iso());
    match typed.party_id {
        Some(account) => slip::slip(conn, typed, spec, &day, account),
        None => table::table(conn, typed, spec, &day),
    }
}
