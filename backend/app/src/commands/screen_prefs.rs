//! What a screen remembers for the Owner: its filters and choices (plan 19, PRD FR-7.45,
//! Architecture 15.3 `user_prefs` key `filters.<screen>`). The last open screen is `prefs_set`
//! with `last_screen`, and the print choices are in the printing settings.
//!
//! The value is JSON text the screen wrote; the backend only checks that it is JSON and small.
//! It is a convenience and never a source of truth for the books.

use mandi_services::{auth, screen_prefs};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::run;
use crate::error::UiError;

/// The input of `prefs_screen_get`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ScreenStateGetInput {
    /// The screen: lower-case letters, digits, `_`, `-` and `.`, at most 40 characters, for
    /// example `report.r20`. `ERR_OUT_OF_RANGE` on `screen` otherwise.
    pub screen: String,
}

/// The input of `prefs_screen_set`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ScreenStateSetInput {
    /// The screen, as in [`ScreenStateGetInput`].
    pub screen: String,
    /// JSON text of at most 4,000 characters. Empty forgets the value. `ERR_OUT_OF_RANGE` on
    /// `value` for text that is not JSON or is too long.
    pub value: String,
}

/// What a screen remembered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ScreenState {
    /// The screen.
    pub screen: String,
    /// The JSON text it saved. Null when nothing is kept.
    pub value: Option<String>,
}

/// What the signed-in Owner saved for a screen. Read only.
#[tauri::command]
pub async fn prefs_screen_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ScreenStateGetInput,
) -> Result<ScreenState, UiError> {
    run(app, "prefs_screen_get", move |ctx| {
        let actor = auth::require(ctx)?;
        let value = screen_prefs::get(ctx, actor.user_id, &input.screen)?;
        Ok(ScreenState {
            screen: input.screen.trim().to_owned(),
            value,
        })
    })
    .await
}

/// Saves what a screen wants to remember, and answers with the saved state.
#[tauri::command]
pub async fn prefs_screen_set<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ScreenStateSetInput,
) -> Result<ScreenState, UiError> {
    run(app, "prefs_screen_set", move |ctx| {
        let actor = auth::require(ctx)?;
        screen_prefs::set(ctx, actor.user_id, &input.screen, &input.value)?;
        let value = screen_prefs::get(ctx, actor.user_id, &input.screen)?;
        Ok(ScreenState {
            screen: input.screen.trim().to_owned(),
            value,
        })
    })
    .await
}
