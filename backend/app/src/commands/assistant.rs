//! Ask the app (PRD Section 7.7, FR-7.72 to FR-7.88, plan 48): thin commands. Every command needs
//! the signed-in Owner, checks it first, and never writes to the books; the question log is the
//! only thing written (FR-7.84). The commands for status, the setting, the key, the log and a new
//! conversation, and the ones that ask a question (ask, quick, pick, rerun, test) are real: each
//! checks the sign-in first and then calls one function of `mandi_assistant::run`.
//!
//! The event `assistant://step` (`mandi_assistant::step::STEP_EVENT`) carries the step lines
//! while a question runs.

use mandi_assistant::ask::{
    AskReply, AssistantAskInput, AssistantNewConversationInput, AssistantPickInput,
    AssistantQuickInput, AssistantRerunInput, ConversationStarted,
};
use mandi_assistant::client::OpenRouterClient;
use mandi_assistant::log::{AssistantLogListInput, AssistantLogPage};
use mandi_assistant::run::Deps;
use mandi_assistant::runtime::Runtime;
use mandi_assistant::settings::{
    AssistantKeyRemoveInput, AssistantKeySaveInput, AssistantSettingsSetInput, AssistantStatus,
    AssistantTestResult,
};
use mandi_assistant::step::{AssistantStep, STEP_EVENT};
use mandi_assistant::{quick, run as asker, service};
use mandi_services::auth::Actor;
use mandi_services::{Context, auth};
use tauri::Emitter;

use super::run;
use crate::error::UiError;
use crate::state::AppState;

/// The one real client. The key is read from Credential Manager by the loop, just before a call.
static CLIENT: OpenRouterClient = OpenRouterClient;

/// Runs `body` with the dependencies of the loop: the context, the Owner, the memory and the step
/// event of the panel.
fn with_deps<R: tauri::Runtime, T>(
    app: &tauri::AppHandle<R>,
    ctx: &Context,
    actor: &Actor,
    runtime: &Runtime,
    body: impl FnOnce(&Deps<'_>) -> T,
) -> T {
    let emit = |step: AssistantStep| {
        // A lost step line is not a failure of the question.
        let _ = app.emit(STEP_EVENT, step);
    };
    let deps = Deps {
        ctx,
        actor,
        runtime,
        client: &CLIENT,
        emit: &emit,
        backup: super::dashboard::backup_facts(app),
    };
    body(&deps)
}

/// On or off, whether a key is saved (with its last 4 characters only), the model, today's count
/// and the limit of 20 (PRD FR-7.81, FR-7.85). Read only.
#[tauri::command]
pub async fn assistant_status<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<AssistantStatus, UiError> {
    let runtime = AppState::of(&app)?.assistant();
    run(app, "assistant_status", move |ctx| {
        auth::require(ctx)?;
        service::status(ctx, &runtime)
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
    let runtime = AppState::of(&app)?.assistant();
    let handle = app.clone();
    run(app, "assistant_ask", move |ctx| {
        let actor = auth::require(ctx)?;
        with_deps(&handle, ctx, &actor, &runtime, |deps| {
            asker::ask(deps, &input)
        })
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
    let runtime = AppState::of(&app)?.assistant();
    let handle = app.clone();
    run(app, "assistant_quick", move |ctx| {
        let actor = auth::require(ctx)?;
        with_deps(&handle, ctx, &actor, &runtime, |deps| {
            quick::quick(deps, &input)
        })
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
    let runtime = AppState::of(&app)?.assistant();
    let handle = app.clone();
    run(app, "assistant_pick", move |ctx| {
        let actor = auth::require(ctx)?;
        with_deps(&handle, ctx, &actor, &runtime, |deps| {
            asker::pick(deps, &input)
        })
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
    let runtime = AppState::of(&app)?.assistant();
    let handle = app.clone();
    run(app, "assistant_rerun", move |ctx| {
        let actor = auth::require(ctx)?;
        with_deps(&handle, ctx, &actor, &runtime, |deps| {
            asker::rerun(deps, &input)
        })
    })
    .await
}

/// A new conversation: the panel forgets the old one (PRD FR-7.78) and the app drops that
/// conversation's memory at once. Writes nothing.
#[tauri::command]
pub async fn assistant_new_conversation<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AssistantNewConversationInput,
) -> Result<ConversationStarted, UiError> {
    let runtime = AppState::of(&app)?.assistant();
    run(app, "assistant_new_conversation", move |ctx| {
        auth::require(ctx)?;
        Ok(service::new_conversation(ctx, &runtime, &input))
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
    let runtime = AppState::of(&app)?.assistant();
    run(app, "assistant_settings_set", move |ctx| {
        let actor = auth::require(ctx)?;
        service::settings_set(ctx, &runtime, &actor, &input)
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
    let runtime = AppState::of(&app)?.assistant();
    run(app, "assistant_key_save", move |ctx| {
        let actor = auth::require(ctx)?;
        service::key_save(ctx, &runtime, &actor, &input)
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
    let runtime = AppState::of(&app)?.assistant();
    run(app, "assistant_key_remove", move |ctx| {
        let actor = auth::require(ctx)?;
        service::key_remove(ctx, &runtime, &actor, &input)
    })
    .await
}

/// Sends one fixed test question and says whether it worked and how long it took. It counts as a
/// question and writes a log row (PRD FR-7.85). It needs a saved key but not the on switch: the
/// Owner tests a key before he turns the assistant on. A failed connection is a result (`worked`
/// false with its `failure`), not an error. Errors: `ERR_ASSISTANT_KEY_MISSING`;
/// `ERR_ASSISTANT_BUSY`.
#[tauri::command]
pub async fn assistant_test<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<AssistantTestResult, UiError> {
    let runtime = AppState::of(&app)?.assistant();
    let handle = app.clone();
    run(app, "assistant_test", move |ctx| {
        let actor = auth::require(ctx)?;
        with_deps(&handle, ctx, &actor, &runtime, asker::test_connection)
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
        service::log_list(ctx, &input)
    })
    .await
}
