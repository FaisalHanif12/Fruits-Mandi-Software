//! Partner capital (PRD 17.4, plan 17): invested and taken back on one screen, with the partner's
//! capital history. Every command needs the signed-in Owner.

use mandi_services::core::capital::{CapitalHistory, CapitalHistoryInput, CapitalInput};
use mandi_services::core::vouchers::VoucherSaved;
use mandi_services::{auth, partners, vouchers};

use super::run;
use crate::error::UiError;

/// Saves capital invested (PRD FR-17.7): روکڑ بنام, the partner's capital account (0602) جمع. Never
/// split, never income. Errors: the date and money codes; `ERR_PARTNER_REQUIRED` and
/// `ERR_NOT_FOUND` on `partner_id`; the text code on `narration`.
#[tauri::command]
pub async fn vouchers_save_capital_in<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CapitalInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_capital_in", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_capital_in(ctx, &actor, &input)
    })
    .await
}

/// Saves capital taken back (PRD FR-17.7): the capital account بنام, روکڑ جمع. More than his
/// balance saves with the warning `ERR_CAPITAL_EXCEEDS_BALANCE` in the preview (FR-17.8). Errors: as
/// `vouchers_save_capital_in`, plus `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value
/// `cash_after_paisa`).
#[tauri::command]
pub async fn vouchers_save_capital_out<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CapitalInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "vouchers_save_capital_out", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::save_capital_out(ctx, &actor, &input)
    })
    .await
}

/// A partner's capital history and balance, oldest first, one page (PRD 17.4). Read only. Errors:
/// `ERR_NOT_FOUND` on `partner_id`.
#[tauri::command]
pub async fn capital_history<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CapitalHistoryInput,
) -> Result<CapitalHistory, UiError> {
    run(app, "capital_history", move |ctx| {
        auth::require(ctx)?;
        partners::capital_history(ctx, &input)
    })
    .await
}
