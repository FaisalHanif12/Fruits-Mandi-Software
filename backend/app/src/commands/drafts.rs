//! Unsaved work of a screen (PRD FR-12.10, NFR-REL-2, plan 13). The screen sends its draft every
//! 5 seconds (`DRAFT_SAVE_INTERVAL_MS = 5_000`) and asks for it after a crash. A draft belongs to
//! the signed-in user and the screen. It is not a business record, so it has no audit row; a
//! saved بکری removes its own draft inside its transaction.

use mandi_services::core::drafts::{
    DraftDiscardInput, DraftGetInput, DraftRow, DraftSaveInput, DraftSaved,
};
use mandi_services::{auth, drafts};

use super::run;
use crate::error::UiError;

/// Keeps the draft of a screen (replaces the earlier one). Errors: `ERR_DRAFT_INVALID`
/// (`payload_json`, value `max_bytes`) for JSON that is not valid or is over 1 MB;
/// `ERR_FORMAT_INVALID` on `screen`.
#[tauri::command]
pub async fn drafts_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DraftSaveInput,
) -> Result<DraftSaved, UiError> {
    run(app, "drafts_save", move |ctx| {
        let actor = auth::require(ctx)?;
        drafts::save(ctx, &actor, &input)
    })
    .await
}

/// The kept draft of a screen, or null. Read only. Errors: `ERR_FORMAT_INVALID` on `screen`.
#[tauri::command]
pub async fn drafts_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DraftGetInput,
) -> Result<Option<DraftRow>, UiError> {
    run(app, "drafts_get", move |ctx| {
        let actor = auth::require(ctx)?;
        drafts::get(ctx, &actor, &input)
    })
    .await
}

/// Throws the draft of a screen away. Nothing happens if there is none. Errors:
/// `ERR_FORMAT_INVALID` on `screen`.
#[tauri::command]
pub async fn drafts_discard<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DraftDiscardInput,
) -> Result<(), UiError> {
    run(app, "drafts_discard", move |ctx| {
        let actor = auth::require(ctx)?;
        drafts::discard(ctx, &actor, &input)
    })
    .await
}
