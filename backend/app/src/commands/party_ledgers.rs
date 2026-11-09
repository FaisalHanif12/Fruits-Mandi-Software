//! The party ledgers screen (پارٹی کھاتے, plan 66, PRD FR-12.43, D8 Feature 6): the balance of every
//! buyer or seller, and one party's statement with a row for each date. Both are read only (Code
//! Standards B29) and need the signed-in Owner (NFR-SEC-4). The third level is the R35 day slip of
//! the party and the date, read with `reports_run` and printed with `reports_print_data`.

use mandi_services::core::party_ledgers::{
    PartyLedgerStatement, PartyLedgerStatementInput, PartyLedgersList, PartyLedgersListInput,
};
use mandi_services::{auth, party_ledgers};

use super::run;
use crate::error::UiError;

/// A page of buyers or sellers by code, active or not, each with his balance now (the figure of his
/// party card and the closing balance of R13) and a mark for the dead amount (PRD FR-10.10). The
/// shared چکتا account and نقدی منڈی are not listed (PRD FR-13.6). `text` filters like the account
/// list: part of the code, or of a name in either script. `count` and the footer are over every
/// party the filter matches, not the page (Code Standards B31). Read only.
#[tauri::command]
pub async fn party_ledgers_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartyLedgersListInput,
) -> Result<PartyLedgersList, UiError> {
    run(app, "party_ledgers_list", move |ctx| {
        auth::require(ctx)?;
        party_ledgers::list(ctx, &input)
    })
    .await
}

/// One party's statement, newest date first: for each date with an entry, his balance before the
/// day, what was charged and paid, and his balance after it, with the totals over every date the
/// filter matches. With no `to`, `closing` is his balance now. Read only. Errors: `ERR_NOT_FOUND`
/// on `account_id` (not a buyer or a seller), `ERR_DATE_INVALID` and `ERR_DATE_IN_FUTURE` on `from`
/// or `to`, `ERR_DATE_RANGE` on `to`.
#[tauri::command]
pub async fn party_ledgers_statement<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartyLedgerStatementInput,
) -> Result<PartyLedgerStatement, UiError> {
    run(app, "party_ledgers_statement", move |ctx| {
        auth::require(ctx)?;
        party_ledgers::statement(ctx, &input)
    })
    .await
}
