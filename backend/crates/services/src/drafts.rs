//! Unsaved work of a screen (PRD FR-12.10, NFR-REL-2, plan 13).
//!
//! The screen sends its draft every 5 seconds (`DRAFT_SAVE_INTERVAL_MS = 5_000`) and asks for it
//! after a crash or a power cut: at most the last 5 seconds are lost. A draft belongs to the
//! signed-in user and the screen. It is the user's own scratch copy, not a business record, so
//! it has no audit row (a draft every 5 seconds would bury the log), and the save of a بکری
//! removes its draft inside its own transaction (`discard_in`).
//!
//! The payload is the screen's own JSON. The backend only checks that it is valid JSON of at
//! most 1 MB (`ERR_DRAFT_INVALID`) and never reads inside it (Code Standards V0: untrusted input
//! is stored as text, never trusted).

use mandi_db::drafts as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::drafts::{
    DraftDiscardInput, DraftGetInput, DraftRow, DraftSaveInput, DraftSaved, MAX_DRAFT_BYTES,
    MAX_SCREEN_CHARS,
};
use mandi_domain::core::error::{ErrorCode, ErrorValue};

use crate::auth::Actor;
use crate::context::Context;
use crate::error::{Rejection, ServiceError};

/// The screen key of the بکری screen (the screen sends it; the save removes this draft).
pub const BIKRI_SCREEN: &str = "bikri";

/// A screen key is `a-z 0-9 _ . -` and 1 to 40 characters, so it is a plain name and nothing a
/// query or a log line could be confused by.
fn check_screen(screen: &str) -> Result<(), ServiceError> {
    let ok = !screen.is_empty()
        && screen.chars().count() <= MAX_SCREEN_CHARS
        && screen
            .chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-'));
    if ok {
        Ok(())
    } else {
        Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "screen",
        ))
    }
}

fn invalid_payload() -> ServiceError {
    let max = i64::try_from(MAX_DRAFT_BYTES).unwrap_or(i64::MAX);
    ServiceError::Rejected(vec![Rejection {
        code: ErrorCode::ErrDraftInvalid,
        field: Some("payload_json".to_owned()),
        values: std::collections::BTreeMap::from([(
            "max_bytes".to_owned(),
            ErrorValue::Number(max),
        )]),
    }])
}

/// True if the text is one valid JSON value. The value itself is not kept in memory.
fn is_valid_json(text: &str) -> bool {
    serde_json::from_str::<serde::de::IgnoredAny>(text).is_ok()
}

/// Keeps the draft of a screen for the Owner, replacing the earlier one.
///
/// # Errors
/// `ERR_FORMAT_INVALID` on `screen`; `ERR_DRAFT_INVALID` on `payload_json` (not JSON, or over
/// 1 MB, value `max_bytes`).
pub fn save(
    ctx: &Context,
    actor: &Actor,
    input: &DraftSaveInput,
) -> Result<DraftSaved, ServiceError> {
    check_screen(&input.screen)?;
    if input.payload_json.len() > MAX_DRAFT_BYTES || !is_valid_json(&input.payload_json) {
        return Err(invalid_payload());
    }
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        repo::upsert(tx, actor.user_id, &input.screen, &input.payload_json, &now).map_err(|error| {
            // The CHECK of the table (SQLite's own JSON check) agrees with ours; if it ever
            // does not, the payload is refused, not a failure of the app.
            if matches!(
                mandi_db::DbError::from(error),
                mandi_db::DbError::Constraint(_)
            ) {
                invalid_payload()
            } else {
                ServiceError::reject(ErrorCode::ErrInternal)
            }
        })
    })?;
    Ok(DraftSaved { saved_at: now })
}

/// The kept draft of a screen, if any.
///
/// # Errors
/// `ERR_FORMAT_INVALID` on `screen`.
pub fn get(
    ctx: &Context,
    actor: &Actor,
    input: &DraftGetInput,
) -> Result<Option<DraftRow>, ServiceError> {
    check_screen(&input.screen)?;
    ctx.db.read(|conn| {
        Ok(
            repo::get(conn, actor.user_id, &input.screen)?.map(|(payload_json, saved_at)| {
                DraftRow {
                    payload_json,
                    saved_at,
                }
            }),
        )
    })
}

/// Throws the draft of a screen away. Nothing happens if there is none.
///
/// # Errors
/// `ERR_FORMAT_INVALID` on `screen`.
pub fn discard(
    ctx: &Context,
    actor: &Actor,
    input: &DraftDiscardInput,
) -> Result<(), ServiceError> {
    check_screen(&input.screen)?;
    ctx.db.write(|tx| {
        repo::delete(tx, actor.user_id, &input.screen)?;
        Ok(())
    })
}

/// Removes a draft inside the caller's transaction (a saved بکری removes its own).
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn discard_in(
    tx: &Connection,
    user_id: i64,
    screen: &str,
) -> Result<usize, ServiceError> {
    Ok(repo::delete(tx, user_id, screen)?)
}
