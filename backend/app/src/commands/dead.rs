//! The dead amount (ڈوبی رقم, PRD FR-10.10, plan 40). Every command needs the signed-in Owner. The
//! preview and the report only read (Code Standards B29). Nothing here posts: a move makes the
//! account inactive and keeps the balance where it is (PRD A65).

use mandi_services::core::dead::{
    DeadAmountInput, DeadAmountPage, DeadMarkInput, DeadMarked, DeadPreview, DeadPreviewInput,
    DeadUndoInput, DeadUndone,
};
use mandi_services::{auth, dead};

use super::reports_engine::engine_error;
use super::run;
use crate::error::UiError;

/// What moving a person to the dead amount would do: what he owes now, whether he can be moved
/// (and the code that says why not), the day of his last posting and his linked account with its
/// balance. Read only. Errors: `ERR_NOT_FOUND` on `account_id`; `ERR_PERSON_REQUIRED` and
/// `ERR_NOT_FOUND` on `pukhta_person_key`. A person who cannot be moved is not an error: `allowed`
/// is false and `reason_not_allowed` is the code.
#[tauri::command]
pub async fn masters_dead_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DeadPreviewInput,
) -> Result<DeadPreview, UiError> {
    run(app, "masters_dead_preview", move |ctx| {
        auth::require(ctx)?;
        dead::preview(ctx, &input)
    })
    .await
}

/// Moves a person to the dead amount (PRD FR-10.10). One transaction: the account is made inactive
/// (a پختہ کھاتہ person only gets his key marked), the move is written with what he owes that day,
/// and the audit row. Nothing posts. Errors: `ERR_NOT_FOUND` (`account_id`); `ERR_NOTHING_OWED`,
/// `ERR_DEAD_NOT_ALLOWED`, `ERR_ALREADY_DEAD` (`account_id`); `ERR_PERSON_REQUIRED` and
/// `ERR_PERSON_IN_DEAD_AMOUNT` (`pukhta_person_key`); `ERR_NOTE_REQUIRED` (`note`); the date codes
/// on `entry_date` and `ERR_DATE_BEFORE_LAST_POSTING`.
#[tauri::command]
pub async fn masters_dead_mark<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DeadMarkInput,
) -> Result<DeadMarked, UiError> {
    run(app, "masters_dead_mark", move |ctx| {
        let actor = auth::require(ctx)?;
        dead::mark(ctx, &actor, &input)
    })
    .await
}

/// Makes a person active again, with a reason ("دوبارہ فعال کریں"). One transaction: the move ends,
/// the account (or the person key) is active, and the audit row. Errors: `ERR_NOT_FOUND`
/// (`mark_id`); `ERR_REASON_REQUIRED` (`reason`).
#[tauri::command]
pub async fn masters_dead_undo<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DeadUndoInput,
) -> Result<DeadUndone, UiError> {
    run(app, "masters_dead_undo", move |ctx| {
        let actor = auth::require(ctx)?;
        dead::undo(ctx, &actor, &input)
    })
    .await
}

/// R38 ڈوبی رقم رپورٹ: one page of the moves, oldest first, with the row count and the totals over
/// the whole filter (Code Standards B31). Read only. Errors: `ERR_OUT_OF_RANGE` on `offset` or
/// `limit`.
#[tauri::command]
pub async fn reports_dead_amount<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DeadAmountInput,
) -> Result<DeadAmountPage, UiError> {
    run(app, "reports_dead_amount", move |ctx| {
        auth::require(ctx)?;
        mandi_reports::dead::page(ctx.db(), &input).map_err(engine_error)
    })
    .await
}
