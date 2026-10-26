//! The master lists of PRD 10.3 (plan 06): list, save, make active or inactive, next code.
//!
//! One generic set of commands serves all ten masters; the kind travels inside the input. There
//! is no delete command: a master can only be made inactive (PRD E21).

use mandi_services::core::masters::{
    MasterInput, MasterRow, MastersList, MastersListInput, MastersNextCodeInput,
    MastersSetActiveInput,
};
use mandi_services::{auth, masters};

use super::run;
use crate::error::UiError;

/// One page of the masters of one kind, by code (names for kinds without a code), with the counts
/// over the whole list (PRD FR-19.11). Inactive ones only when asked.
#[tauri::command]
pub async fn masters_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: MastersListInput,
) -> Result<MastersList, UiError> {
    run(app, "masters_list", move |ctx| {
        auth::require(ctx)?;
        masters::list_page(ctx, &input)
    })
    .await
}

/// Adds a master (`id` null) or edits one (`id` and the `version` that was read). The Urdu and
/// English names are checked for their script and their spelling (PRD FR-7.28, FR-7.66).
/// Errors: the validate codes with their field paths, `ERR_CODE_TAKEN` (`code`),
/// `ERR_NOT_FOUND`, `ERR_VERSION_CHANGED`, `ERR_SPELLING_UNCONFIRMED` (`name_ur`, `name_en`).
#[tauri::command]
pub async fn masters_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: MasterInput,
) -> Result<MasterRow, UiError> {
    run(app, "masters_save", move |ctx| {
        let actor = auth::require(ctx)?;
        masters::save(ctx, &actor, &input)
    })
    .await
}

/// Makes a master active or inactive. Errors: `ERR_NOT_FOUND`.
#[tauri::command]
pub async fn masters_set_active<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: MastersSetActiveInput,
) -> Result<MasterRow, UiError> {
    run(app, "masters_set_active", move |ctx| {
        let actor = auth::require(ctx)?;
        masters::set_active(ctx, &actor, input.kind, input.id, input.is_active)
    })
    .await
}

/// The next free code of a master, for the form to suggest. An item gets `0040`, `0041` ..., a
/// packing `0050` ... and a staff member `0060` ... (plan 32 R2); the other kinds the largest number
/// plus one. Empty for a narration, which has no code.
#[tauri::command]
pub async fn masters_next_code<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: MastersNextCodeInput,
) -> Result<String, UiError> {
    run(app, "masters_next_code", move |ctx| {
        auth::require(ctx)?;
        masters::next_code(ctx, input.kind)
    })
    .await
}
