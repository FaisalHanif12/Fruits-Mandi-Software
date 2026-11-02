//! Ask the app, the database side (plan 48, PRD FR-7.81, FR-7.84, FR-7.85, NFR-SEC-15).
//!
//! The model, the tools and the checks live in `mandi-assistant`. What needs the services' own
//! parts lives here: the two settings (on or off, the model), the question log and the daily
//! count (the Pakistan date), the proof of the Owner's password for the key, and the audit rows
//! of a change. Nothing here touches the books: the only table written is `assistant_questions`
//! (append-only) and the two settings keys.
//!
//! The OpenRouter key itself is never here. It lives in Credential Manager (`mandi-assistant`),
//! and no audit row, setting or log row holds it or any part of it (NFR-SEC-15).
//!
//! Example: at 19:30 UTC on 10-10-26 it is 00:30 on 11-10-26 in Pakistan, so a question asked then
//! counts for 11-10-26: the day starts at midnight, Pakistan time (PRD FR-7.81).

use chrono::{FixedOffset, SecondsFormat};
use mandi_db::assistant::{self as log, NewQuestion, QuestionRow};
use mandi_domain::core::auth::Language;
use mandi_domain::core::error::ErrorCode;
use serde_json::json;

use crate::audit::Event;
use crate::auth::{Actor, prove_owner};
use crate::context::Context;
use crate::error::ServiceError;
use crate::partner_code;
use crate::secret::Secret;
use crate::store;

/// The setting that holds on or off: `true` or `false`.
pub const KEY_ENABLED: &str = "assistant_enabled";
/// The setting that holds the model name.
pub const KEY_MODEL: &str = "assistant_model";
/// The preference that holds the app language (PRD FR-7.30).
const PREF_LANGUAGE: &str = "language";
/// Pakistan is UTC+5 all year (no daylight saving).
const PAKISTAN_OFFSET_SECONDS: i32 = 5 * 3600;
/// Longest model name.
const MAX_MODEL_CHARS: usize = 100;
/// Most rows of one page of the log (Code Standards B31).
pub const MAX_LOG_PAGE: i64 = 100;

/// The two settings of the assistant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// True when the assistant is switched on. It starts off.
    pub enabled: bool,
    /// The model name, or `None` before the Owner chooses one (the caller then uses its default).
    pub model: Option<String>,
}

/// A question to write to the log. The services fill the time and the Pakistan day.
#[derive(Debug, Clone)]
pub struct QuestionSpec {
    /// The question as typed, or a chip's name.
    pub question: String,
    /// The app language.
    pub language: Language,
    /// The conversation of the panel.
    pub conversation_id: String,
    /// The question a continuation belongs to.
    pub parent_id: Option<i64>,
    /// `chip`, `model`, `continue`, `rerun` or `test`.
    pub source: &'static str,
    /// True for a model question and a connection test.
    pub counted: bool,
    /// Each source read, as JSON text.
    pub sources_json: String,
    /// How it ended.
    pub outcome: &'static str,
    /// Each tool called with its inputs, as JSON text.
    pub tool_calls_json: String,
    /// The answer as the model wrote it, as JSON text. Null for a chip.
    pub answer_json: Option<String>,
    /// The model. Null for a chip.
    pub model: Option<String>,
    /// Model calls made.
    pub model_calls: Option<i64>,
    /// Input tokens.
    pub tokens_in: Option<i64>,
    /// Of those, cached.
    pub tokens_cached: Option<i64>,
    /// Output tokens.
    pub tokens_out: Option<i64>,
    /// Of those, reasoning.
    pub tokens_reasoning: Option<i64>,
    /// The last `finish_reason`, or the limit that stopped the question (`call_limit`,
    /// `token_limit`).
    pub finish_reason: Option<String>,
    /// Milliseconds this run took, from the start to the reply (plan 51 B2).
    pub total_millis: Option<i64>,
    /// Of those, the time spent waiting for the model.
    pub model_millis: Option<i64>,
    /// Of those, the time of the app's own work in its tools.
    pub tool_millis: Option<i64>,
    /// Each model call of this run as JSON: its time, its tokens and its tools (names and times
    /// only). Null for a chip.
    pub calls_json: Option<String>,
}

/// On or off and the model. Off, with no model, before the Owner sets them.
///
/// # Errors
/// `ServiceError::Db`.
pub fn config(ctx: &Context) -> Result<Config, ServiceError> {
    ctx.db.read(|conn| {
        let enabled = store::setting(conn, KEY_ENABLED)?
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let model = store::setting(conn, KEY_MODEL)?
            .and_then(|v| v.as_str().map(str::to_owned))
            .filter(|m| !m.is_empty());
        Ok(Config { enabled, model })
    })
}

/// A model name is 1 to 100 characters of letters, digits and `. _ : / -`, with no spaces
/// (an OpenRouter model name such as `openai/gpt-6-luna`).
fn check_model(model: &str) -> Result<(), ServiceError> {
    if model.is_empty() {
        return Err(ServiceError::reject_field(ErrorCode::ErrRequired, "model"));
    }
    let ok = model.chars().count() <= MAX_MODEL_CHARS
        && model
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '/' | '-'));
    if ok {
        Ok(())
    } else {
        Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "model",
        ))
    }
}

/// Switches the assistant on or off and sets the model, with an audit row of the change (PRD
/// FR-7.85). `key_saved` says whether a key is in Credential Manager: it can be switched on only
/// then.
///
/// # Errors
/// `ERR_ASSISTANT_KEY_MISSING` on `enabled`; `ERR_REQUIRED` or `ERR_FORMAT_INVALID` on `model`;
/// `ServiceError::Db`.
pub fn set_config(
    ctx: &Context,
    actor: &Actor,
    enabled: bool,
    model: &str,
    key_saved: bool,
) -> Result<Config, ServiceError> {
    let model = model.trim();
    check_model(model)?;
    if enabled && !key_saved {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrAssistantKeyMissing,
            "enabled",
        ));
    }
    ctx.db.write(|tx| {
        let was_enabled = store::setting(tx, KEY_ENABLED)?
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let was_model = store::setting(tx, KEY_MODEL)?.and_then(|v| v.as_str().map(str::to_owned));
        if was_enabled != enabled || was_model.as_deref() != Some(model) {
            store::set_setting(tx, KEY_ENABLED, &json!(enabled), Some(actor.user_id))?;
            store::set_setting(tx, KEY_MODEL, &json!(model), Some(actor.user_id))?;
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), "edit", "settings")
                    .old(json!({ "assistant_enabled": was_enabled, "assistant_model": was_model }))
                    .new_value(json!({ "assistant_enabled": enabled, "assistant_model": model }))
                    .into_entry(),
            )?;
        }
        Ok(Config {
            enabled,
            model: Some(model.to_owned()),
        })
    })
}

/// Switches the assistant off and leaves the model as it is, in the same transaction as nothing
/// else (used when the key is removed, PRD FR-7.85). Writes an audit row only if it was on.
///
/// # Errors
/// `ServiceError::Db`.
pub fn switch_off(ctx: &Context, actor: &Actor) -> Result<(), ServiceError> {
    ctx.db.write(|tx| {
        let was = store::setting(tx, KEY_ENABLED)?
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if was {
            store::set_setting(tx, KEY_ENABLED, &json!(false), Some(actor.user_id))?;
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), "edit", "settings")
                    .old(json!({ "assistant_enabled": true }))
                    .new_value(json!({ "assistant_enabled": false }))
                    .reason("key removed")
                    .into_entry(),
            )?;
        }
        Ok(())
    })
}

/// Proves the Owner's password before the key is saved, replaced or removed (PRD FR-7.85). A wrong
/// password counts like a wrong sign-in (PRD FR-8.2).
///
/// # Errors
/// `ERR_REQUIRED` or `ERR_WRONG_PASSWORD` on `password`; `ERR_LOCKED_WAIT`.
pub fn prove_password(ctx: &Context, actor: &Actor, password: &Secret) -> Result<(), ServiceError> {
    prove_owner(ctx, actor, password, "password", false)
}

/// Writes the audit row of a key change, **without the key** (PRD FR-7.85, NFR-SEC-15): only that
/// it was saved, replaced or removed.
///
/// # Errors
/// `ServiceError::Db`.
pub fn audit_key_change(
    ctx: &Context,
    actor: &Actor,
    change: &'static str,
) -> Result<(), ServiceError> {
    ctx.db.write(|tx| {
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), change, "assistant_key")
                .new_value(json!({ "assistant_key": change }))
                .into_entry(),
        )?;
        Ok(())
    })
}

/// The Pakistan date now, `YYYY-MM-DD`. The day starts at midnight, Pakistan time (PRD FR-7.81).
pub fn pakistan_day(ctx: &Context) -> String {
    let offset = FixedOffset::east_opt(PAKISTAN_OFFSET_SECONDS);
    match offset {
        Some(zone) => ctx
            .now_utc()
            .with_timezone(&zone)
            .format("%Y-%m-%d")
            .to_string(),
        // Cannot happen: 5 hours is a valid offset. Fall back to the UTC date.
        None => ctx.now_utc().format("%Y-%m-%d").to_string(),
    }
}

/// How many counted questions today has (PRD FR-7.81).
///
/// # Errors
/// `ServiceError::Db`.
pub fn count_today(ctx: &Context) -> Result<i64, ServiceError> {
    let day = pakistan_day(ctx);
    ctx.db.read(|conn| Ok(log::count_counted_on(conn, &day)?))
}

/// The app language of the Owner: his saved preference, Urdu before he chooses (PRD FR-7.30).
///
/// # Errors
/// `ServiceError::Db`.
pub fn language(ctx: &Context, actor: &Actor) -> Result<Language, ServiceError> {
    ctx.db.read(|conn| {
        Ok(store::pref(conn, actor.user_id, PREF_LANGUAGE)?
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(Language::Ur))
    })
}

fn language_text(language: Language) -> &'static str {
    match language {
        Language::Ur => "ur",
        Language::En => "en",
    }
}

/// Writes one row of the question log, in its own save, after the answer (PRD FR-7.84). The time
/// and the Pakistan day come from the clock of the database.
///
/// # Errors
/// `ServiceError::Db`.
pub fn record_question(
    ctx: &Context,
    actor: &Actor,
    spec: &QuestionSpec,
) -> Result<i64, ServiceError> {
    let asked_at = ctx.now_utc().to_rfc3339_opts(SecondsFormat::Millis, true);
    let asked_day = pakistan_day(ctx);
    let row = NewQuestion {
        asked_at: &asked_at,
        asked_day: &asked_day,
        asked_by: Some(actor.user_id),
        question: &spec.question,
        language: language_text(spec.language),
        conversation_id: &spec.conversation_id,
        parent_id: spec.parent_id,
        source: spec.source,
        counted: spec.counted,
        sources_json: &spec.sources_json,
        outcome: spec.outcome,
        tool_calls_json: &spec.tool_calls_json,
        answer_json: spec.answer_json.as_deref(),
        model: spec.model.as_deref(),
        model_calls: spec.model_calls,
        tokens_in: spec.tokens_in,
        tokens_cached: spec.tokens_cached,
        tokens_out: spec.tokens_out,
        tokens_reasoning: spec.tokens_reasoning,
        finish_reason: spec.finish_reason.as_deref(),
        total_millis: spec.total_millis,
        model_millis: spec.model_millis,
        tool_millis: spec.tool_millis,
        calls_json: spec.calls_json.as_deref(),
    };
    ctx.db.write(|tx| Ok(log::insert(tx, &row)?))
}

/// One row of the log.
///
/// # Errors
/// `ServiceError::Db`.
pub fn get_question(ctx: &Context, id: i64) -> Result<Option<QuestionRow>, ServiceError> {
    ctx.db.read(|conn| Ok(log::get(conn, id)?))
}

/// One page of the log, newest first, and the count of the whole log (Code Standards B31).
/// `limit` 0 means 100; at most 100.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `offset` (below 0) or `limit` (below 0 or above 100); `ServiceError::Db`.
pub fn list_questions(
    ctx: &Context,
    offset: i64,
    limit: i64,
) -> Result<(Vec<QuestionRow>, i64, i64), ServiceError> {
    if offset < 0 {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrOutOfRange,
            "offset",
        ));
    }
    if !(0..=MAX_LOG_PAGE).contains(&limit) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrOutOfRange,
            "limit",
        ));
    }
    let limit = if limit == 0 { MAX_LOG_PAGE } else { limit };
    ctx.db.read(|conn| {
        let rows = log::list(conn, offset, limit)?;
        let total = log::total(conn)?;
        Ok((rows, total, limit))
    })
}

/// True while the partner section is locked (PRD FR-7.82): the tools of `mandi-assistant` pause
/// at a partner source and send no partner figure to the model.
///
/// # Errors
/// `ServiceError::Db`.
pub fn partner_locked(ctx: &Context) -> Result<bool, ServiceError> {
    partner_code::is_locked(ctx)
}

/// The names of the seeded system accounts and the expense heads, for the name check of an answer
/// (see `mandi_db::assistant::exempt_names`). Read only.
///
/// # Errors
/// `ServiceError::Db`.
pub fn exempt_names(ctx: &Context) -> Result<Vec<String>, ServiceError> {
    ctx.db.read(|conn| Ok(log::exempt_names(conn)?))
}
