//! What a screen remembers for the Owner: the choices and filters of a report, an analytics view
//! or a list (PRD FR-7.45, Architecture 15.3 `user_prefs` key `filters.<screen>`).
//!
//! The service keeps the screen's own text and does not read it: the value is JSON text the screen
//! wrote (for example `{"from":"01-10-2026","shortcut":"this_month"}`), checked only to be valid
//! JSON of a small size. It is a convenience: it is never a source of truth for the books, and a
//! value the screen cannot read any more is simply ignored by the screen. Nothing about money or
//! the books is stored here, so there is no audit row (Code Standards S7 covers money, rules,
//! shares, the password and settings only).
//!
//! The last open screen and the print choices have their own keys (`last_screen` through
//! `set_pref`, `print_after_save` through the printing settings), so they are not repeated here.
//!
//! Example: the screen `report.r20` saves `{"city_id":3}`; after a restart the same call with the
//! screen `report.r20` gives the same text back.

use mandi_db::rusqlite::params;
use mandi_domain::core::error::ErrorCode;
use serde_json::Value;

use crate::context::Context;
use crate::error::ServiceError;
use crate::store;

/// The part of the preference key before the screen name.
const KEY_PREFIX: &str = "filters.";
/// The longest screen name.
pub const MAX_SCREEN_CHARS: usize = 40;
/// The longest value, in characters.
pub const MAX_VALUE_CHARS: usize = 4_000;
/// The most screens one user may keep a value for, so the table cannot grow without end.
pub const MAX_SCREENS: i64 = 200;

/// The key of a screen, or `ERR_OUT_OF_RANGE` on `screen` for a name that is empty, too long or
/// holds anything but lower-case letters, digits, `_`, `-` and `.`.
fn key_of(screen: &str) -> Result<String, ServiceError> {
    let name = screen.trim();
    let valid = !name.is_empty()
        && name.chars().count() <= MAX_SCREEN_CHARS
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '.'));
    if !valid {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrOutOfRange,
            "screen",
        ));
    }
    Ok(format!("{KEY_PREFIX}{name}"))
}

/// What the signed-in Owner saved for a screen, as the JSON text he saved. `None` when nothing was
/// saved. Read only.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` (`screen`); `ServiceError::Db`; `ServiceError::Data` for a stored value
/// that is not JSON.
pub fn get(ctx: &Context, user_id: i64, screen: &str) -> Result<Option<String>, ServiceError> {
    let key = key_of(screen)?;
    ctx.db
        .read(|conn| Ok::<_, ServiceError>(store::pref(conn, user_id, &key)?))
        .map(|value| value.filter(|v| !v.is_null()).map(|v| v.to_string()))
}

/// Saves what a screen wants to remember. An empty `value_text` forgets it (the row is kept with
/// `null`, so the count of screens only grows by a new screen name).
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `screen`, or on `value` for text that is not JSON or is longer than
/// [`MAX_VALUE_CHARS`], or for a new screen when [`MAX_SCREENS`] are kept already;
/// `ServiceError::Db`.
pub fn set(
    ctx: &Context,
    user_id: i64,
    screen: &str,
    value_text: &str,
) -> Result<(), ServiceError> {
    let key = key_of(screen)?;
    let trimmed = value_text.trim();
    let value: Value = if trimmed.is_empty() {
        Value::Null
    } else if trimmed.chars().count() > MAX_VALUE_CHARS {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrOutOfRange,
            "value",
        ));
    } else {
        serde_json::from_str(trimmed)
            .map_err(|_| ServiceError::reject_field(ErrorCode::ErrOutOfRange, "value"))?
    };
    ctx.db.write(|tx| {
        let known: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM user_prefs WHERE user_id = ?1 AND key = ?2)",
            params![user_id, key],
            |r| r.get(0),
        )?;
        if !known {
            let kept: i64 = tx.query_row(
                "SELECT COUNT(*) FROM user_prefs WHERE user_id = ?1 AND key LIKE 'filters.%'",
                [user_id],
                |r| r.get(0),
            )?;
            if kept >= MAX_SCREENS {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrOutOfRange,
                    "screen",
                ));
            }
        }
        store::set_pref(tx, user_id, &key, &value)
    })
}
