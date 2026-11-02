//! Both forms of the day queries of R34, for the equality test of plan 48 P48-S3 only.
//!
//! The day summary reads the lines of the buyers and the sellers for some days. For the whole book
//! it starts from the entries of the days; the older form starts from the accounts. The first is
//! the one the app uses, because the app's file has no table statistics. `services/tests/
//! speed_rewrites_equal.rs` reads each query in both forms on the same books and holds the rows to
//! the same list, in the same order.

use mandi_db::rusqlite::Connection;

use super::sheet;
use crate::error::ReportsError;

/// The rows of one query in both forms, as text.
pub struct FormRows {
    /// The entries of the days first (the form the app uses for the whole book).
    pub entries_first: Vec<String>,
    /// The lines of the accounts first (the form before plan 48 P48-S3).
    pub lines_first: Vec<String>,
}

fn rows(forms: sheet::Forms) -> FormRows {
    FormRows {
        entries_first: forms.entries_first,
        lines_first: forms.lines_first,
    }
}

/// The ledger by party and day, whole book.
///
/// # Errors
/// A database error.
pub fn ledger_rows(conn: &Connection, from: &str, to: &str) -> Result<FormRows, ReportsError> {
    Ok(rows(sheet::ledger_forms(conn, from, to)?))
}

/// The sale lines by the seller of the lot, whole book.
///
/// # Errors
/// A database error.
pub fn seller_lines(conn: &Connection, from: &str, to: &str) -> Result<FormRows, ReportsError> {
    Ok(rows(sheet::seller_forms(conn, from, to)?))
}

/// The charges of the lots of the first postings, whole book.
///
/// # Errors
/// A database error.
pub fn lot_charges(conn: &Connection, from: &str, to: &str) -> Result<FormRows, ReportsError> {
    Ok(rows(sheet::lot_forms(conn, from, to)?))
}

/// The movements of the buyer accounts by day, account and person.
///
/// # Errors
/// A database error.
pub fn buyer_moves(conn: &Connection, from: &str, to: &str) -> Result<FormRows, ReportsError> {
    let text = |entries_first: bool| -> Result<Vec<String>, ReportsError> {
        Ok(super::day::moves_during(conn, from, to, entries_first)?
            .iter()
            .map(|m| format!("{m:?}"))
            .collect())
    };
    Ok(FormRows {
        entries_first: text(true)?,
        lines_first: text(false)?,
    })
}
