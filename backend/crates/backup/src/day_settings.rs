//! The Day group of the settings (PRD Section 21, FR-20.3, FR-21.1): the closing time and whether
//! the day closes by itself.
//!
//! - **Closing time** is the time of the daily USB and cloud copy, one value with the backup
//!   settings (`BackupSettings::closing_time`); this group reads and writes that same field, so
//!   there is one source and one check (`HH:MM`, `ERR_OUT_OF_RANGE` with `min` and `max`).
//! - **Auto day close**: the `settings` row `auto_day_close` holds `true` or `false`, starting
//!   `false`. When it is `true`, the app closes today by itself at the closing time (the thread
//!   of `app/src/day_auto.rs`, plan 19), through the same close the screen uses.
//!
//! A change writes an audit row with the old and the new value in the same transaction (Code
//! Standards S7), and asking for the values it has writes nothing.
//!
//! Example: closing time `21:00` and auto close off at the start; the Owner sets `22:30` and on.

use mandi_db::Db;
use mandi_db::audit::AuditEntry;
use mandi_db::rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use crate::error::BackupError;
use crate::settings::{self, closing_time_problem};

/// The `settings.key` of the auto day close switch.
pub const AUTO_DAY_CLOSE_KEY: &str = "auto_day_close";

/// The Day group of the settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct DaySettings {
    /// The time of day (Pakistan time, `HH:MM`) when the daily copy is made. Shared with the
    /// backup settings.
    pub closing_time: String,
    /// True when the day is closed by itself at the closing time (plan 19: the app does it, but
    /// never while the Owner has an unsaved draft).
    pub auto_day_close: bool,
}

fn load_auto(conn: &Connection) -> Result<bool, BackupError> {
    let text: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [AUTO_DAY_CLOSE_KEY],
            |row| row.get(0),
        )
        .optional()?;
    match text {
        None => Ok(false),
        Some(text) => Ok(serde_json::from_str(&text)?),
    }
}

/// Reads the Day group. A missing row gives closing time `21:00` and auto close off.
///
/// # Errors
/// `ERR_INTERNAL` if a row cannot be read.
pub fn load(conn: &Connection) -> Result<DaySettings, BackupError> {
    Ok(DaySettings {
        closing_time: settings::load(conn)?.closing_time,
        auto_day_close: load_auto(conn)?,
    })
}

/// Checks and saves the Day group, in one transaction with its audit rows.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `closing_time` (values `min`, `max`); `ERR_DISK_FULL`, `ERR_DB_BUSY`
/// and the other database codes.
pub fn save(db: &Db, user_id: Option<i64>, new: &DaySettings) -> Result<DaySettings, BackupError> {
    if let Some(problem) = closing_time_problem(&new.closing_time) {
        return Err(problem);
    }
    db.write(|tx| {
        let old = load(tx)?;
        if old.closing_time != new.closing_time {
            let mut backup = settings::load(tx)?;
            backup.closing_time.clone_from(&new.closing_time);
            settings::save_in(db, tx, user_id, &backup)?;
        }
        if old.auto_day_close != new.auto_day_close {
            tx.execute(
                "INSERT INTO settings (key, value_json, created_by, updated_by) VALUES (?1, ?2, ?3, ?3) \
                 ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, \
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), updated_by = excluded.updated_by, \
                 version = version + 1",
                params![AUTO_DAY_CLOSE_KEY, json!(new.auto_day_close).to_string(), user_id],
            )?;
            db.auditor().write(
                tx,
                &AuditEntry {
                    user_id,
                    action: "edit".to_owned(),
                    entity: "day_settings".to_owned(),
                    entity_id: None,
                    old_json: Some(json!({ "auto_day_close": old.auto_day_close })),
                    new_json: Some(json!({ "auto_day_close": new.auto_day_close })),
                    reason: None,
                },
            )?;
        }
        Ok::<(), BackupError>(())
    })?;
    Ok(new.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bad_closing_time_is_refused_with_its_bounds() {
        let problem = closing_time_problem("25:00").unwrap();
        assert_eq!(problem.field.as_deref(), Some("closing_time"));
        assert!(closing_time_problem("22:30").is_none());
    }
}
