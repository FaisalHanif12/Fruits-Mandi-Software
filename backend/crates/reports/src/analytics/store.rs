//! Saved views (PRD FR-18.5): a named set of filters the Owner opens again from the dashboard.
//!
//! They are stored for the signed-in user in the `user_prefs` table under the key
//! `analytics.views`, as JSON, the whole list in one row (no migration). The pref commands of the
//! settings module take only three fixed keys, so these two commands read and write that row.
//! A saved view holds only filters; every figure is read again when it is opened, so a saved
//! view never keeps a number (FR-18.6).
//!
//! Every filter set is checked on save with the same rules as a run, so an item or a seller that
//! no longer exists is refused with the field `views[2].filters.item_id`. At most 50 views; a
//! name is 1 to 60 letters with no control characters. Two views may have the same name.
//!
//! Example: the Owner saves "کیلا کمیشن ماہانہ" (view D, item کیلا, 2026, by month) and opens it
//! from the list with one click.

use mandi_core::error::{ErrorCode, FieldError};
use mandi_db::Db;
use mandi_db::rusqlite::OptionalExtension;

use super::filters::resolve;
use super::model::{AnalyticsSavedView, AnalyticsViewsSaveInput};
use crate::error::ReportsError;
use crate::reports::common::today;

/// The key of the row in `user_prefs`.
pub const PREF_KEY: &str = "analytics.views";
/// Most saved views of one user.
pub const MAX_VIEWS: usize = 50;
/// Longest name of a saved view, in letters.
pub const MAX_NAME_LETTERS: usize = 60;

/// The saved views of a user, in the order he saved them.
///
/// # Errors
/// A database error; `ReportsError::Data` if the stored text is not a list of saved views.
pub fn views_get(db: &Db, user_id: i64) -> Result<Vec<AnalyticsSavedView>, ReportsError> {
    let text: Option<String> = db.read(|conn| {
        conn.query_row(
            "SELECT value_json FROM user_prefs WHERE user_id = ?1 AND key = ?2",
            mandi_db::rusqlite::params![user_id, PREF_KEY],
            |r| r.get(0),
        )
        .optional()
    })?;
    match text {
        None => Ok(Vec::new()),
        Some(text) => serde_json::from_str(&text)
            .map_err(|e| ReportsError::Data(format!("saved views are not readable: {e}"))),
    }
}

/// Replaces the saved views of a user with this list, after checking every name and every filter
/// set. Returns the list as stored.
///
/// # Errors
/// `ReportsError::Rejected` with every problem and its field (`views`, `views[i].name`,
/// `views[i].filters.*`); a database error.
pub fn views_save(
    db: &Db,
    user_id: i64,
    input: &AnalyticsViewsSaveInput,
) -> Result<Vec<AnalyticsSavedView>, ReportsError> {
    let mut problems: Vec<FieldError> = Vec::new();
    if input.views.len() > MAX_VIEWS {
        problems.push(FieldError::new(ErrorCode::ErrOutOfRange, "views"));
    }
    let now = today(db.clock().now_utc())?;
    let mut cleaned = Vec::with_capacity(input.views.len());
    db.read(|conn| {
        for (index, view) in input.views.iter().enumerate() {
            let name = view.name.trim();
            let field = format!("views[{index}].name");
            if name.is_empty() {
                problems.push(FieldError::new(ErrorCode::ErrRequired, &field));
            } else if name.chars().count() > MAX_NAME_LETTERS {
                problems.push(FieldError::new(ErrorCode::ErrNameTooLong, &field));
            } else if name.chars().any(char::is_control) {
                problems.push(FieldError::new(ErrorCode::ErrControlChars, &field));
            }
            match resolve(conn, now, view.view, &view.filters) {
                Ok(_) => {}
                Err(ReportsError::Rejected(items)) => {
                    for item in items {
                        let inner = item.field.clone();
                        let mut moved = item;
                        moved.field = format!("views[{index}].{inner}");
                        problems.push(moved);
                    }
                }
                Err(other) => return Err(other),
            }
            cleaned.push(AnalyticsSavedView {
                name: name.to_owned(),
                view: view.view,
                filters: view.filters.clone(),
            });
        }
        Ok::<(), ReportsError>(())
    })?;
    if !problems.is_empty() {
        return Err(ReportsError::Rejected(problems));
    }
    let json = serde_json::to_string(&cleaned)
        .map_err(|e| ReportsError::Data(format!("saved views cannot be written: {e}")))?;
    db.write(|tx| {
        tx.execute(
            "INSERT INTO user_prefs (user_id, key, value_json, created_by, updated_by)
             VALUES (?1, ?2, ?3, ?1, ?1)
             ON CONFLICT(user_id, key) DO UPDATE SET value_json = excluded.value_json,
                 updated_by = excluded.updated_by,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), version = version + 1",
            mandi_db::rusqlite::params![user_id, PREF_KEY, json],
        )?;
        Ok::<(), ReportsError>(())
    })?;
    Ok(cleaned)
}
