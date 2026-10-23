//! The روکڑ بک (PRD 16.1, plan 17). Needs the signed-in Owner. Read only.

use mandi_services::core::cashbook::{Cashbook, CashbookInput};
use mandi_services::{auth, cashbook};

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
