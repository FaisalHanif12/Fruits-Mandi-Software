//! The party ledgers screen (پارٹی کھاتے, plan 66, PRD FR-12.43, D8 Feature 6): the balance of every
//! buyer or seller, and one party's statement with a row for each date. Both are read only (Code
//! Standards B29) and need the signed-in Owner (NFR-SEC-4). The third level is the R35 day slip of
//! the party and the date, read with `reports_run` and printed with `reports_print_data`.

use mandi_reports::reports::{
    StatementPrintData, StatementPrintDataInput, party_statement_print_data,
};
use mandi_services::core::party_ledgers::{
    PartyLedgerGoTo, PartyLedgerGoToInput, PartyLedgerStatement, PartyLedgerStatementInput,
    PartyLedgersList, PartyLedgersListInput,
};
use mandi_services::{auth, party_ledgers};

use super::reports_engine::engine_error;
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
/// on `account_id` (not a buyer or a seller), `ERR_DATE_INVALID` on `from` or `to`,
/// `ERR_DATE_RANGE` on `to`.
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

/// Goes to a date on one party's statement (PRD FR-12.47, plan 74 W5a): the 30-day window that ends on
/// the earlier of the day plus 15 days and today, as `from` and `to` (`YYYY-MM-DD`), with that
/// window's page of dates and totals in `statement` (the same answer as `party_ledgers_statement`
/// for it), `has_entry` for the asked day, and `row`, its place among `statement.rows` (null with no
/// entry). `date` is `DD-MM-YYYY` or a shortcut. Read only. Errors: `ERR_REQUIRED`,
/// `ERR_DATE_INVALID` and `ERR_DATE_IN_FUTURE` on `date`; `ERR_NOT_FOUND` on `account_id`.
#[tauri::command]
pub async fn party_ledgers_go_to_date<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartyLedgerGoToInput,
) -> Result<PartyLedgerGoTo, UiError> {
    run(app, "party_ledgers_go_to_date", move |ctx| {
        auth::require(ctx)?;
        party_ledgers::go_to_date(ctx, &input)
    })
    .await
}

/// One party's statement for a period, laid out for a print (PRD FR-12.47, plan 74): every date with
/// an entry, oldest first, each with its goods, its commission and لاگا (a seller's deductions), its
/// bill (a buyer's bill or a seller's net sale), what was paid on the spot, each وصولی, ادائیگی and
/// پیشگی, and the balance at the end of the day; then the totals of the period, the closing balance
/// and its amount in words. Every text is written in the print language (digits 0 to 9, dates
/// `DD-MM-YYYY`, names in that script); the labels are Fluent message ids. The figures are the
/// statement's and the R35 day slip's own, held against each other before anything is returned.
/// `from` and `to` are typed (`DD-MM-YYYY` or a shortcut) and both are required; `paper` is A4 when
/// left out. The header and footer come from `reports_print_header` with `title_id` and `filters`.
/// Read only. Errors: `ERR_REQUIRED`, `ERR_DATE_INVALID` on `from` or `to`; `ERR_DATE_RANGE` on `to`;
/// `ERR_NOT_FOUND` on `account_id` (not a buyer or a seller).
#[tauri::command]
pub async fn party_ledgers_statement_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: StatementPrintDataInput,
) -> Result<StatementPrintData, UiError> {
    run(app, "party_ledgers_statement_print_data", move |ctx| {
        auth::require(ctx)?;
        let whole = party_ledgers::statement_whole(ctx, input.account_id, &input.from, &input.to)?;
        party_statement_print_data(ctx.db(), &whole, &input).map_err(engine_error)
    })
    .await
}
