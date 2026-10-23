//! The two listings of the readable pack that no report covers yet: the shop expenses and the
//! partners' personal expenses, read from the posted journal lines of their accounts (PRD
//! FR-20.5, FR-16.2, Code Standards B20).
//!
//! A line is one posted debit or credit on an account of the group, so a cancelled expense shows
//! with its reversal and nets to nothing, a salary deduction (posted to the تنخواہ head) and a
//! جنرل line on the group are included, and the total equals the journal exactly. Month-close
//! entries are left out: they only move a month's balance to capital (FR-17.6).
//!
//! The month-close filter is on the source of the entry, as R28 and R30 do it, so a reversed
//! month close is left out too.
//!
//! Amounts are stored `INTEGER` paisa columns summed with `SUM()`; the difference of the two
//! sums is one checked subtraction.

use mandi_core::money::Paisa;
use mandi_db::rusqlite::{Connection, params};

use crate::error::ReportsError;

/// The accounts of a group, as a half-open range of codes (`0501` to `0502`). The unique index
/// on `accounts.code` answers it without scanning.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CodeRange {
    pub(crate) from: &'static str,
    pub(crate) to: &'static str,
}

/// Shop expense heads (PRD FR-10.4: control 0501).
pub(crate) const SHOP_EXPENSES: CodeRange = CodeRange {
    from: "0501",
    to: "0502",
};

/// Each partner's drawings account (PRD FR-10.4: control 0603).
pub(crate) const PERSONAL_EXPENSES: CodeRange = CodeRange {
    from: "0603",
    to: "0604",
};

/// One posted line of the listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListLine {
    pub(crate) date_iso: String,
    pub(crate) posting_type: String,
    pub(crate) account_code: String,
    pub(crate) name_ur: String,
    pub(crate) name_en: String,
    pub(crate) narration: String,
    /// Debit minus credit of the line.
    pub(crate) amount: Paisa,
}

/// The lines of a group between two days, and their total from the books.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Listing {
    pub(crate) lines: Vec<ListLine>,
    /// `SUM(debit) - SUM(credit)` of the same lines.
    pub(crate) total: Paisa,
}

const WHERE: &str = "e.status = 'posted' AND e.source_type <> 'month_close'
     AND a.code >= ?1 AND a.code < ?2 AND e.entry_date >= ?3 AND e.entry_date <= ?4";

/// The SQL of the lines, for the query plan test (Code Standards B18).
#[doc(hidden)]
pub fn lines_sql() -> String {
    format!(
        "SELECT e.entry_date, e.posting_type, a.code, a.name_ur, a.name_en,
                COALESCE(NULLIF(l.narration, ''), e.narration, ''), l.dr_paisa, l.cr_paisa
         FROM accounts a
         JOIN journal_lines l ON l.account_id = a.id
         JOIN journal_entries e ON e.id = l.entry_id
         WHERE {WHERE}
         ORDER BY e.entry_date, e.id, l.line_no"
    )
}

fn totals_sql() -> String {
    format!(
        "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM accounts a
         JOIN journal_lines l ON l.account_id = a.id
         JOIN journal_entries e ON e.id = l.entry_id
         WHERE {WHERE}"
    )
}

/// Reads the posted lines of a group of accounts between two days (both included).
///
/// # Errors
/// A database or money error; `ReportsError::Data` if the lines do not add up to the total (a
/// bug signal).
pub(crate) fn read(
    conn: &Connection,
    range: CodeRange,
    from_iso: &str,
    to_iso: &str,
) -> Result<Listing, ReportsError> {
    let mut statement = conn.prepare(&lines_sql())?;
    let rows = statement.query_map(params![range.from, range.to, from_iso, to_iso], |row| {
        Ok((
            ListLine {
                date_iso: row.get(0)?,
                posting_type: row.get(1)?,
                account_code: row.get(2)?,
                name_ur: row.get(3)?,
                name_en: row.get(4)?,
                narration: row.get(5)?,
                amount: Paisa::ZERO,
            },
            row.get::<_, i64>(6)?,
            row.get::<_, i64>(7)?,
        ))
    })?;
    let mut lines = Vec::new();
    let mut check = Paisa::ZERO;
    for found in rows {
        let (mut line, dr, cr) = found?;
        line.amount = Paisa::try_from_paisa(dr)?.checked_sub(Paisa::try_from_paisa(cr)?)?;
        check = check.checked_add(line.amount)?;
        lines.push(line);
    }
    let (dr_total, cr_total): (i64, i64) = conn.query_row(
        &totals_sql(),
        params![range.from, range.to, from_iso, to_iso],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let total = Paisa::try_from_paisa(dr_total)?.checked_sub(Paisa::try_from_paisa(cr_total)?)?;
    if total != check {
        return Err(ReportsError::Data(
            "the lines of a pack listing do not add up to its total".to_owned(),
        ));
    }
    Ok(Listing { lines, total })
}
