//! The combined balance of two linked accounts on R13 and R14 (PRD E10, FR-10.2, gate finding T4-2).
//!
//! One person can be both a seller and a buyer. His two accounts are linked, so the ledger heading
//! can show what he owes or is owed in all. The line is read only: nothing is posted, and it is
//! worked out here from both ledgers with checked integer maths, never by the screen.
//!
//! Example: a buyer account closes at 2,880 بنام and the linked seller account is 10,160 جمع on the
//! same day. The line "both accounts together" reads 7,280 جمع.

use mandi_core::accounts::Balance;
use mandi_db::accounts as account_repo;
use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;

use crate::error::ReportsError;
use crate::reports::common::{balance_cell, paisa};
use crate::reports::ids;
use crate::reports::model::SummaryLine;

/// The summary line "both accounts together", or `None` when the account has no link, when the
/// other account is missing, or when it is چکتا کھاتہ, which has no ledger (PRD FR-13.6). `closing`
/// is this ledger's closing balance, `as_of` the last day of the ledger (`None` for up to now).
/// The other account counts up to the same day, so the two balances are of one moment. An inactive
/// other account still counts: its balance is real.
///
/// # Errors
/// A database error; `ReportsError::Money`.
pub(super) fn combined_line(
    conn: &Connection,
    account_id: i64,
    closing: Balance,
    as_of: Option<&str>,
) -> Result<Option<SummaryLine>, ReportsError> {
    let Some(other_id) = account_repo::linked_account_of(conn, account_id)? else {
        return Ok(None);
    };
    let Some(other) = account_repo::get(conn, other_id)? else {
        return Ok(None);
    };
    if other.id == account_id || other.is_chukta_khata {
        return Ok(None);
    }
    let totals = ledger_repo::totals_of(conn, other.id, None, as_of)?;
    let other_balance = Balance::from_totals(paisa(totals.dr_paisa)?, paisa(totals.cr_paisa)?)?;
    let combined = closing.combined_with(other_balance)?;
    Ok(Some(SummaryLine {
        label_id: ids::LINKED_COMBINED.to_owned(),
        cell: balance_cell(combined),
    }))
}
