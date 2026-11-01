//! Ask the app (PRD Section 7.7, FR-7.72 to FR-7.88, plan 48): thin commands. Every command needs
//! the signed-in Owner, checks it first, and never writes to the books; the question log is the
//! only thing written (FR-7.84). Until plan 48 T3 each command answers `ERR_NOT_IMPLEMENTED`
//! after the sign-in check.
//!
//! The event `assistant://step` (`mandi_assistant::step::STEP_EVENT`) carries the step lines
//! while a question runs.

use mandi_assistant::ask::{
    AskReply, AssistantAskInput, AssistantPickInput, AssistantQuickInput, AssistantRerunInput,
    ConversationStarted,
};
use mandi_assistant::log::{AssistantLogListInput, AssistantLogPage};
use mandi_assistant::settings::{
    AssistantKeyRemoveInput, AssistantKeySaveInput, AssistantSettingsSetInput, AssistantStatus,
    AssistantTestResult,
};
use mandi_services::{ServiceError, auth};

use super::run;
use crate::error::{ErrorCode, UiError};

/// The refusal every stub gives, after the sign-in check.
fn not_implemented<T>() -> Result<T, ServiceError> {
    Err(ServiceError::reject(ErrorCode::ErrNotImplemented))
}

/// On or off, whether a key is saved (with its last 4 characters only), the model, today's count
/// and the limit of 20 (PRD FR-7.81, FR-7.85). Read only.
#[tauri::command]
pub async fn assistant_status<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<AssistantStatus, UiError> {
    run(app, "assistant_status", |ctx| {
        auth::require(ctx)?;
        not_implemented()
    })
    .await
}

/// A typed question to the model (PRD FR-7.72). One question, one log row; it counts once in the
/// 20 a day when the model's first reply arrives (FR-7.81). Errors: `ERR_ASSISTANT_OFF`;
/// `ERR_ASSISTANT_KEY_MISSING`; `ERR_ASSISTANT_BUSY` (a question is running); `ERR_REQUIRED` or
/// `ERR_TEXT_TOO_LONG` on `text`. No internet, a full day, a locked partner section and a failed
/// model call are not errors: they are outcomes of the reply (`offline`, `limit_reached`,
/// `locked`, `error`).
#[tauri::command]
pub async fn assistant_ask<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AssistantAskInput,
) -> Result<AskReply, UiError> {
    run(app, "assistant_ask", move |ctx| {
        auth::require(ctx)?;
        let _ = &input;
        not_implemented()
    })
    .await
}

/// A quick question (a chip): fixed tool calls and fixed words, no model, no internet, not counted
/// (PRD FR-7.80). Errors: `ERR_REQUIRED` on `party_id` for the party chip; `ERR_NOT_FOUND` on
/// `party_id`.
#[tauri::command]
pub async fn assistant_quick<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AssistantQuickInput,
) -> Result<AskReply, UiError> {
    run(app, "assistant_quick", move |ctx| {
        auth::require(ctx)?;
        let _ = &input;
        not_implemented()
    })
    .await
}

/// Goes on with a question that waits: after "کون سا؟" (the chosen match) or after the partner
/// code was given (PRD FR-7.77, FR-7.82). Not counted again. Errors: `ERR_NOT_FOUND` on
/// `question_id`; `ERR_ASSISTANT_NOT_WAITING` on `question_id`; `ERR_ASSISTANT_OFF`;
/// `ERR_ASSISTANT_KEY_MISSING`; `ERR_ASSISTANT_BUSY`; `ERR_PARTNER_LOCKED` when the code was
/// not given.
#[tauri::command]
pub async fn assistant_pick<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AssistantPickInput,
) -> Result<AskReply, UiError> {
    run(app, "assistant_pick", move |ctx| {
        auth::require(ctx)?;
        let _ = &input;
        not_implemented()
    })
    .await
}

/// Reads an old answer again: the same tool calls on today's books, shown as plain tables with a
/// note that figures may have changed. No model call; not counted (PRD FR-7.84). Errors:
/// `ERR_NOT_FOUND` on `question_id`.
#[tauri::command]
pub async fn assistant_rerun<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AssistantRerunInput,
) -> Result<AskReply, UiError> {
    run(app, "assistant_rerun", move |ctx| {
        auth::require(ctx)?;
        let _ = &input;
        not_implemented()
    })
    .await
}

/// A new conversation: the panel forgets the old one (PRD FR-7.78). Writes nothing.
#[tauri::command]
pub async fn assistant_new_conversation<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<ConversationStarted, UiError> {
    run(app, "assistant_new_conversation", |ctx| {
        auth::require(ctx)?;
        not_implemented()
    })
    .await
}

/// Switches the assistant on or off and sets the model name; written to the audit log (PRD
/// FR-7.85). Errors: `ERR_ASSISTANT_KEY_MISSING` on `enabled`; `ERR_REQUIRED` or
/// `ERR_FORMAT_INVALID` on `model`.
#[tauri::command]
pub async fn assistant_settings_set<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AssistantSettingsSetInput,
) -> Result<AssistantStatus, UiError> {
    run(app, "assistant_settings_set", move |ctx| {
        auth::require(ctx)?;
        let _ = &input;
        not_implemented()
    })
    .await
}

/// Saves or replaces the OpenRouter key in Credential Manager, proved by the Owner's password;
/// written to the audit log without the key (PRD FR-7.85, NFR-SEC-15). The reply never holds the
/// key. Errors: `ERR_REQUIRED` or `ERR_FORMAT_INVALID` on `key`; `ERR_WRONG_PASSWORD` on
/// `password`; `ERR_LOCKED_WAIT` (value `seconds`) after too many wrong tries.
#[tauri::command]
pub async fn assistant_key_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AssistantKeySaveInput,
) -> Result<AssistantStatus, UiError> {
    run(app, "assistant_key_save", move |ctx| {
        auth::require(ctx)?;
        let _ = &input;
        not_implemented()
    })
    .await
}

/// Removes the key and switches the assistant off, proved by the Owner's password; written to the
/// audit log (PRD FR-7.85). Errors: `ERR_WRONG_PASSWORD` on `password`; `ERR_LOCKED_WAIT`.
#[tauri::command]
pub async fn assistant_key_remove<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AssistantKeyRemoveInput,
) -> Result<AssistantStatus, UiError> {
    run(app, "assistant_key_remove", move |ctx| {
        auth::require(ctx)?;
        let _ = &input;
        not_implemented()
    })
    .await
}

/// Sends one fixed test question and says whether it worked and how long it took. It counts as a
/// question and writes a log row (PRD FR-7.85). A failed connection is a result (`worked` false
/// with its `failure`), not an error. Errors: `ERR_ASSISTANT_OFF`; `ERR_ASSISTANT_KEY_MISSING`;
/// `ERR_ASSISTANT_BUSY`.
#[tauri::command]
pub async fn assistant_test<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<AssistantTestResult, UiError> {
    run(app, "assistant_test", |ctx| {
        auth::require(ctx)?;
        not_implemented()
    })
    .await
}

/// The question log, page by page, newest first, with the count over the whole log (PRD FR-7.84,
/// Code Standards B31). Read only. Errors: `ERR_OUT_OF_RANGE` on `offset` or `limit`.
#[tauri::command]
pub async fn assistant_log_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AssistantLogListInput,
) -> Result<AssistantLogPage, UiError> {
    run(app, "assistant_log_list", move |ctx| {
        auth::require(ctx)?;
        let _ = &input;
        not_implemented()
    })
    .await
}
