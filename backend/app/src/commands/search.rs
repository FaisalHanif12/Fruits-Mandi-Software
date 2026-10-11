//! Search for the pickers (plan 06, Tech Stack 4.7). Read only, except `search_touch`, which
//! writes the user's own preference and nothing else (Code Standards B29).

use mandi_services::core::search::{
    SearchHit, SearchMastersInput, SearchRecentInput, SearchTouchInput,
};
use mandi_services::{auth, search};

use super::run;
use crate::error::UiError;

/// The active records of a list that match the typed text, best first: exact code, then names
/// that start with the text, then records used before, then close spellings. Both scripts are
/// searched; the text is never converted (PRD FR-7.40).
#[tauri::command]
pub async fn search_masters<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SearchMastersInput,
) -> Result<Vec<SearchHit>, UiError> {
    run(app, "search_masters", move |ctx| {
        let actor = auth::require(ctx)?;
        search::search(ctx, &actor, input.scope, &input.text, input.limit)
    })
    .await
}

/// The last 10 records used in a list, then the rest (PRD FR-7.41).
#[tauri::command]
pub async fn search_recent<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SearchRecentInput,
) -> Result<Vec<SearchHit>, UiError> {
    run(app, "search_recent", move |ctx| {
        let actor = auth::require(ctx)?;
        search::recent(ctx, &actor, input.scope)
    })
    .await
}

/// Records that a record was chosen. Errors: `ERR_NOT_FOUND`.
#[tauri::command]
pub async fn search_touch<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SearchTouchInput,
) -> Result<(), UiError> {
    run(app, "search_touch", move |ctx| {
        let actor = auth::require(ctx)?;
        search::touch(ctx, &actor, input.scope, input.id)
    })
    .await
}
