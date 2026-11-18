//! The روکڑ بک (PRD 16.1, plan 17). Needs the signed-in Owner. Read only.

use mandi_services::core::cashbook::{CapitalWhere, Cashbook, CashbookInput};
use mandi_services::{auth, cashbook, partner_code};

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

/// Where the partners' money is now and whose it is (PRD FR-16.14 v1.43, plan 78 E): the card on the
/// روکڑ page. Behind the partner code (PRD FR-17.11, Code Standards S8). Read only.
/// Errors: `ERR_PARTNER_LOCKED`.
#[tauri::command]
pub async fn cashbook_capital_where<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<CapitalWhere, UiError> {
    run(app, "cashbook_capital_where", move |ctx| {
        auth::require(ctx)?;
        partner_code::require_unlocked(ctx)?;
        cashbook::capital_where(ctx)
    })
    .await
}
