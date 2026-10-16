//! The بکری entry screen (PRD 12, plan 12). Every command needs the signed-in Owner. They only
//! read: nothing here saves (Code Standards B29). The save is `bikri_save` in plan 13.

use mandi_services::core::bikri::{
    BikriPreview, BikriPreviewInput, FreightPart, SplitFreightInput, split_freight,
};
use mandi_services::{auth, bikri};

use super::run;
use crate::error::UiError;

/// Works out a whole typed بکری for the live summary: every line, commission and لاگا with its
/// working, the seller summary, the buyer bills, the live partner split, the cash note and all
/// errors and warnings, each at its field. Nothing is saved. The draft is text; the answer is
/// money in paisa.
#[tauri::command]
pub async fn bikri_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BikriPreviewInput,
) -> Result<BikriPreview, UiError> {
    run(app, "bikri_preview", move |ctx| {
        auth::require(ctx)?;
        bikri::preview(ctx, &input.draft)
    })
    .await
}

/// Splits the header's total freight over the lots by نگ received (PRD C17). Read only.
/// Errors: the money and quantity codes on `total_freight` and `lots[i].qty_received`,
/// `ERR_NO_LOTS` (`lots`).
#[tauri::command]
pub async fn bikri_split_freight<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SplitFreightInput,
) -> Result<Vec<FreightPart>, UiError> {
    run(app, "bikri_split_freight", move |ctx| {
        auth::require(ctx)?;
        Ok(split_freight(&input)?)
    })
    .await
}
