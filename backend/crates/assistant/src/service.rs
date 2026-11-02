//! The commands that need no model: status, the setting, the key, the log and a new conversation
//! (plan 48 Section 4, PRD FR-7.81, FR-7.84, FR-7.85). Each takes the signed-in Owner; the thin
//! commands in the app check the sign-in first. None of them writes to the books: they write only
//! the two settings, the key (in Credential Manager) and audit rows.
//!
//! Example: with the key `…ab12` saved, the assistant on, and 9 counted questions today,
//! [`status`] gives `enabled` true, `key_saved` true, `key_last4` "ab12", `count_today` 9 and
//! `limit` 20.

use mandi_services::auth::Actor;
use mandi_services::core::error::ErrorCode;
use mandi_services::{Context, Secret, ServiceError, assistant};
use serde_json::Value;

use crate::ask::{AskOutcome, AssistantNewConversationInput, ConversationStarted};
use crate::keys::{KeyStoreError, is_well_formed, last4};
use crate::limits::DAILY_LIMIT;
use crate::log::{AssistantLogListInput, AssistantLogPage, AssistantLogRow, LogSource, LogTool};
use crate::runtime::Runtime;
use crate::settings::{
    AssistantKeyRemoveInput, AssistantKeySaveInput, AssistantSettingsSetInput, AssistantStatus,
    DEFAULT_MODEL,
};

/// A store failure is an internal error (the Owner cannot fix it); the text never holds the key.
fn store_error(error: KeyStoreError) -> ServiceError {
    ServiceError::Data(format!("credential store: {}", error.0))
}

/// On or off, the key (its last 4 characters only), the model, today's count and the limit (PRD
/// FR-7.81, FR-7.85).
///
/// # Errors
/// `ServiceError::Db`; the key store could not be reached (`ERR_INTERNAL`).
pub fn status(ctx: &Context, runtime: &Runtime) -> Result<AssistantStatus, ServiceError> {
    let config = assistant::config(ctx)?;
    let key = runtime.keys().get().map_err(store_error)?;
    Ok(AssistantStatus {
        enabled: config.enabled && key.is_some(),
        key_saved: key.is_some(),
        key_last4: key.as_deref().and_then(last4),
        model: config.model.unwrap_or_else(|| DEFAULT_MODEL.to_owned()),
        default_model: DEFAULT_MODEL.to_owned(),
        count_today: assistant::count_today(ctx)?,
        limit: DAILY_LIMIT,
    })
}

/// Switches the assistant on or off and sets the model (PRD FR-7.85). On needs a saved key.
///
/// # Errors
/// `ERR_ASSISTANT_KEY_MISSING` on `enabled`; `ERR_REQUIRED` or `ERR_FORMAT_INVALID` on `model`.
pub fn settings_set(
    ctx: &Context,
    runtime: &Runtime,
    actor: &Actor,
    input: &AssistantSettingsSetInput,
) -> Result<AssistantStatus, ServiceError> {
    let key_saved = runtime.keys().get().map_err(store_error)?.is_some();
    assistant::set_config(ctx, actor, input.enabled, &input.model, key_saved)?;
    status(ctx, runtime)
}

/// Saves or replaces the key, proved by the Owner's password, with an audit row that holds no key
/// (PRD FR-7.85, NFR-SEC-15).
///
/// # Errors
/// `ERR_REQUIRED` or `ERR_FORMAT_INVALID` on `key`; `ERR_WRONG_PASSWORD` on `password`;
/// `ERR_LOCKED_WAIT`.
pub fn key_save(
    ctx: &Context,
    runtime: &Runtime,
    actor: &Actor,
    input: &AssistantKeySaveInput,
) -> Result<AssistantStatus, ServiceError> {
    let key = input.key.trim();
    if key.is_empty() {
        return Err(ServiceError::reject_field(ErrorCode::ErrRequired, "key"));
    }
    if !is_well_formed(key) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "key",
        ));
    }
    assistant::prove_password(ctx, actor, &Secret::new(input.password.as_str()))?;
    let replaced = runtime.keys().get().map_err(store_error)?.is_some();
    runtime.keys().set(key).map_err(store_error)?;
    assistant::audit_key_change(
        ctx,
        actor,
        if replaced {
            "assistant_key_replace"
        } else {
            "assistant_key_save"
        },
    )?;
    status(ctx, runtime)
}

/// Removes the key and switches the assistant off, proved by the Owner's password (PRD FR-7.85).
///
/// # Errors
/// `ERR_WRONG_PASSWORD` on `password`; `ERR_LOCKED_WAIT`.
pub fn key_remove(
    ctx: &Context,
    runtime: &Runtime,
    actor: &Actor,
    input: &AssistantKeyRemoveInput,
) -> Result<AssistantStatus, ServiceError> {
    assistant::prove_password(ctx, actor, &Secret::new(input.password.as_str()))?;
    let had = runtime.keys().get().map_err(store_error)?.is_some();
    runtime.keys().delete().map_err(store_error)?;
    assistant::switch_off(ctx, actor)?;
    if had {
        assistant::audit_key_change(ctx, actor, "assistant_key_remove")?;
    }
    status(ctx, runtime)
}

/// A new conversation: the panel forgets the old one (PRD FR-7.78). The old conversation's memory
/// is dropped at once, so an ask with the old id starts fresh.
pub fn new_conversation(
    ctx: &Context,
    runtime: &Runtime,
    input: &AssistantNewConversationInput,
) -> ConversationStarted {
    if let Some(old) = &input.conversation_id {
        runtime.forget(old);
    }
    ConversationStarted {
        conversation_id: runtime.new_conversation_id(ctx),
    }
}

fn parse<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, ServiceError> {
    serde_json::from_value(Value::String(text.to_owned()))
        .map_err(|e| ServiceError::Data(format!("question log: {e}")))
}

/// The tools of one log row: the name, and the report id or view letter it read.
fn tools_of(tool_calls_json: &str) -> Vec<LogTool> {
    let calls: Vec<Value> = serde_json::from_str(tool_calls_json).unwrap_or_default();
    calls
        .iter()
        .filter_map(|call| {
            let tool = call.get("tool")?.as_str()?.to_owned();
            let input = call.get("input");
            let source = input
                .and_then(|i| i.get("id").or_else(|| i.get("view")))
                .and_then(Value::as_str)
                .map(str::to_owned);
            Some(LogTool { tool, source })
        })
        .collect()
}

/// One page of the question log, newest first (PRD FR-7.84, Code Standards B31).
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `offset` or `limit`.
pub fn log_list(
    ctx: &Context,
    input: &AssistantLogListInput,
) -> Result<AssistantLogPage, ServiceError> {
    let (rows, total, limit) = assistant::list_questions(ctx, input.offset, input.limit)?;
    let rows = rows
        .into_iter()
        .map(|row| {
            let tools = tools_of(&row.tool_calls_json);
            Ok(AssistantLogRow {
                id: row.id,
                asked_at: row.asked_at,
                asked_day: row.asked_day,
                question: row.question,
                source: parse::<LogSource>(&row.source)?,
                outcome: parse::<AskOutcome>(&row.outcome)?,
                counted: row.counted,
                parent_id: row.parent_id,
                total_millis: row.total_millis,
                model_millis: row.model_millis,
                tool_millis: row.tool_millis,
                can_rerun: !tools.is_empty(),
                tools,
                model: row.model,
                model_calls: row.model_calls,
                tokens_in: row.tokens_in,
                tokens_cached: row.tokens_cached,
                tokens_out: row.tokens_out,
                tokens_reasoning: row.tokens_reasoning,
            })
        })
        .collect::<Result<Vec<_>, ServiceError>>()?;
    Ok(AssistantLogPage {
        rows,
        offset: input.offset,
        limit,
        total_count: total,
    })
}
