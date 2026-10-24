//! Month close commands (plan 22 round 7, PRD FR-17.6, FR-21.2, NFR-SEC-4). Every command needs
//! the signed-in Owner. A command checks sign-in, then calls one service function in
//! `mandi_services::month` (one transaction for a close or a reversal).
//!
//! `month_close_preview` shows what a close would post and is refused for the same reasons as
//! `month_close_save`, so the screen can say why before the Owner presses the button.
//!
//! Example: `month_close_save { month: "2026-10" }` after the worked day answers
//! `{ month: "2026-10", entry_id, total_paisa: 548800 }` (5,488 rupees).

use mandi_services::auth;
use mandi_services::month::{
    self, MonthClosed, MonthInput, MonthList, MonthPreview, MonthReverseInput, MonthReversed,
};

use super::run;
use crate::error::UiError;

/// The months from the first entry to this month, each with where it stands. Read only.
#[tauri::command]
pub async fn month_list<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<MonthList, UiError> {
    run(app, "month_list", |ctx| {
        auth::require(ctx)?;
        month::list(ctx)
    })
    .await
}

/// What closing a month would post. Read only.
#[tauri::command]
pub async fn month_close_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: MonthInput,
) -> Result<MonthPreview, UiError> {
    run(app, "month_close_preview", move |ctx| {
        auth::require(ctx)?;
        month::preview(ctx, &input)
    })
    .await
}

/// Closes a finished month (PRD FR-17.6). The screen asks the Owner to confirm first (FR-7.8).
#[tauri::command]
pub async fn month_close_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: MonthInput,
) -> Result<MonthClosed, UiError> {
    run(app, "month_close_save", move |ctx| {
        let actor = auth::require(ctx)?;
        month::close(ctx, &actor, &input)
    })
    .await
}

/// Reverses the close in force for a month, with a reason (PRD FR-21.2).
#[tauri::command]
pub async fn month_close_reverse<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: MonthReverseInput,
) -> Result<MonthReversed, UiError> {
    run(app, "month_close_reverse", move |ctx| {
        let actor = auth::require(ctx)?;
        month::reverse(ctx, &actor, &input)
    })
    .await
}
