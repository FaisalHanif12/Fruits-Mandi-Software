//! جنرل (PRD 16.4, plan 17): a journal entry of free lines. It saves only when Σ بنام = Σ جمع.
//! Every command needs the signed-in Owner.

use mandi_services::core::general::{GeneralInput, GeneralPreview, GeneralPreviewInput};
use mandi_services::core::vouchers::VoucherSaved;
use mandi_services::{auth, vouchers};

use super::run;
use crate::error::UiError;

/// Checks the lines as typed and gives the totals, the difference, whether the entry can be saved
/// and the partner split of income and expense lines. Nothing is saved. The same function as the
/// save (Code Standards B5).
#[tauri::command]
pub async fn general_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GeneralPreviewInput,
) -> Result<GeneralPreview, UiError> {
    run(app, "general_preview", move |ctx| {
        auth::require(ctx)?;
        vouchers::general_preview(ctx, &input)
    })
    .await
}

/// Saves a جنرل entry in one transaction (PRD 16.4). Errors: the date codes; the line codes at
/// `lines[i].<field>`; `ERR_GENERAL_TOO_FEW_LINES` and `ERR_GENERAL_NOT_BALANCED` (values `dr_paisa`,
/// `cr_paisa`) on `lines`, with nothing written; `ERR_OWN_GOODS_ACCOUNT_RESTRICTED` on
/// `lines[i].account_id`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`).
#[tauri::command]
pub async fn general_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GeneralInput,
) -> Result<VoucherSaved, UiError> {
    run(app, "general_save", move |ctx| {
        let actor = auth::require(ctx)?;
        vouchers::general_save(ctx, &actor, &input)
    })
    .await
}
