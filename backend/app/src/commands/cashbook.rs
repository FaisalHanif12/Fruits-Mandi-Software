//! The روکڑ بک (PRD 16.1, plan 17). Needs the signed-in Owner. Read only.

use mandi_reports::reports::{TablePrintData, capital_card_print_data};
use mandi_services::core::cashbook::{
    CapitalPrintInput, CapitalWhere, CapitalWhereInput, Cashbook, CashbookInput,
};
use mandi_services::{Context, ServiceError};
use mandi_services::{auth, cashbook, partner_code};

use super::reports_engine::engine_error;
use super::run;
use crate::error::UiError;

/// The cash book of a range, one page: the opening and closing cash, every cash in and out from
/// all modules with the running cash, and the totals over the whole range (PRD FR-19.11). There is
/// no cash count command (FR-16.1). Errors: the date codes on `from` and `to`.
#[tauri::command]
pub async fn cashbook_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CashbookInput,
) -> Result<Cashbook, UiError> {
    run(app, "cashbook_get", move |ctx| {
        auth::require(ctx)?;
        cashbook::get(ctx, &input)
    })
    .await
}

/// Where the partners' money is now and whose it is (PRD FR-16.14 v1.43 and v1.50, plans 78 E and 87):
/// the card on the روکڑ page, as of a day. The input may be left out (as of today). Behind the partner
/// code (PRD FR-17.11, Code Standards S8). Read only.
/// Errors: `ERR_PARTNER_LOCKED`; `ERR_DATE_FORMAT`, `ERR_DATE_IN_FUTURE` (`as_of`).
#[tauri::command]
pub async fn cashbook_capital_where<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: Option<CapitalWhereInput>,
) -> Result<CapitalWhere, UiError> {
    run(app, "cashbook_capital_where", move |ctx| {
        auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        cashbook::capital_where_at(ctx, &input.unwrap_or_default())
    })
    .await
}

/// The print data of the card: the same read as `cashbook_capital_where` (Code Standards B5), laid out
/// as a table in the print language (PRD FR-16.14 v1.50, FR-19.10, plan 87). Behind the partner code.
/// The print window reads it. Errors: `ERR_PARTNER_LOCKED`; the date codes on `as_of`.
#[tauri::command]
pub async fn cashbook_capital_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CapitalPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "cashbook_capital_print_data", move |ctx| {
        auth::require(ctx)?;
        capital_print_data(ctx, &input)
    })
    .await
}

/// The card as a print table, after the partner code is checked: one function for the data command
/// and the print job.
pub(crate) fn capital_print_data(
    ctx: &Context,
    input: &CapitalPrintInput,
) -> Result<TablePrintData, ServiceError> {
    partner_code::require_unlocked(ctx)?;
    let card = cashbook::capital_where_at(
        ctx,
        &CapitalWhereInput {
            as_of: input.as_of.clone(),
        },
    )?;
    capital_card_print_data(ctx.db(), input.language, input.paper, &card).map_err(engine_error)
}
