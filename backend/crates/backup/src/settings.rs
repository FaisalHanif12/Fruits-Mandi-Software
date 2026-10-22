//! The settings of the backup (PRD 21 "Backup": cloud account details, local backup folder, USB
//! on or off, closing time). The non-secret part is one row of `settings`, written with an audit
//! row in the same transaction (Code Standards S7). The cloud keys are secrets and live only in
//! the Credential Manager (`secrets`).

use mandi_db::Db;
use mandi_db::audit::AuditEntry;
use mandi_db::rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use crate::error::{BackupCode, BackupError};

/// The `settings.key` of this group.
pub const SETTINGS_KEY: &str = "backup";
/// When the cloud copy is made each day if the Owner set nothing (FR-20.3).
pub const DEFAULT_CLOSING_TIME: &str = "21:00";

/// The backup settings that are not secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct BackupSettings {
    /// Folder of the 15-minute local copies. Empty means the `backups/local` folder of the app
    /// data. Must be a full path.
    pub local_dir: String,
    /// True when a copy goes to a USB drive (FR-20.4).
    pub usb_enabled: bool,
    /// The folder on the USB drive, for example `E:\MandiBackup`. The drive counts as plugged in
    /// when the folder, or the drive it is on, is there.
    pub usb_folder: String,
    /// True when a copy goes to the cloud (FR-20.3).
    pub cloud_enabled: bool,
    /// The S3 endpoint, for example `https://<account>.r2.cloudflarestorage.com`.
    pub cloud_endpoint: String,
    /// The bucket.
    pub cloud_bucket: String,
    /// The region. Cloudflare R2 uses `auto`.
    pub cloud_region: String,
    /// A folder inside the bucket for all objects of this shop, for example `mandi`.
    pub cloud_prefix: String,
    /// The time of day (Pakistan time, `HH:MM`) when the daily USB and cloud copy is made
    /// (FR-20.3). Shared with the Day settings group.
    pub closing_time: String,
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            local_dir: String::new(),
            usb_enabled: false,
            usb_folder: String::new(),
            cloud_enabled: false,
            cloud_endpoint: String::new(),
            cloud_bucket: String::new(),
            cloud_region: "auto".to_owned(),
            cloud_prefix: "mandi".to_owned(),
            closing_time: DEFAULT_CLOSING_TIME.to_owned(),
        }
    }
}

/// The closing time as minutes since midnight.
pub fn closing_minutes(text: &str) -> Option<u32> {
    let (hours, minutes) = text.split_once(':')?;
    if hours.len() != 2 || minutes.len() != 2 {
        return None;
    }
    let (hours, minutes): (u32, u32) = (hours.parse().ok()?, minutes.parse().ok()?);
    (hours < 24 && minutes < 60).then_some(hours * 60 + minutes)
}

fn out_of_range(field: &str, detail: &str) -> BackupError {
    BackupError::new(BackupCode::ErrOutOfRange, detail).on_field(field)
}

fn required(field: &str) -> BackupError {
    BackupError::new(BackupCode::ErrRequired, "required").on_field(field)
}

fn is_full_path(text: &str) -> bool {
    std::path::Path::new(text).is_absolute()
}

impl BackupSettings {
    /// Checks every field and returns all problems together.
    ///
    /// # Errors
    /// `ERR_REQUIRED` and `ERR_OUT_OF_RANGE`, each on its field.
    pub fn validate(&self) -> Result<(), BackupError> {
        let mut problems = Vec::new();
        if closing_minutes(&self.closing_time).is_none() {
            problems.push(out_of_range("closing_time", "closing time is not HH:MM"));
        }
        if self.local_dir.chars().any(char::is_control) || self.local_dir.len() > 260 {
            problems.push(out_of_range("local_dir", "local folder"));
        } else if !self.local_dir.is_empty() && !is_full_path(&self.local_dir) {
            problems.push(out_of_range(
                "local_dir",
                "local folder must be a full path",
            ));
        }
        if self.usb_enabled {
            if self.usb_folder.trim().is_empty() {
                problems.push(required("usb_folder"));
            } else if !is_full_path(&self.usb_folder)
                || self.usb_folder.chars().any(char::is_control)
            {
                problems.push(out_of_range("usb_folder", "usb folder must be a full path"));
            }
        }
        if self.cloud_enabled {
            let endpoint = self.cloud_endpoint.trim();
            if endpoint.is_empty() {
                problems.push(required("cloud_endpoint"));
            } else if !endpoint.starts_with("https://") {
                problems.push(out_of_range(
                    "cloud_endpoint",
                    "endpoint must start with https://",
                ));
            }
            let bucket = self.cloud_bucket.trim();
            if bucket.is_empty() {
                problems.push(required("cloud_bucket"));
            } else if bucket.len() < 3
                || bucket.len() > 63
                || !bucket
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
            {
                problems.push(out_of_range("cloud_bucket", "bucket name"));
            }
            if self.cloud_region.trim().is_empty() {
                problems.push(required("cloud_region"));
            }
        }
        let prefix = &self.cloud_prefix;
        if prefix.len() > 64
            || prefix.contains("..")
            || !prefix
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
        {
            problems.push(out_of_range("cloud_prefix", "folder in the bucket"));
        }
        BackupError::from_many(problems).map_or(Ok(()), Err)
    }
}

/// Reads the settings. A missing row gives the defaults.
///
/// # Errors
/// `ERR_INTERNAL` if the row cannot be read or is not the expected JSON.
pub fn load(conn: &Connection) -> Result<BackupSettings, BackupError> {
    let text: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [SETTINGS_KEY],
            |row| row.get(0),
        )
        .optional()?;
    match text {
        None => Ok(BackupSettings::default()),
        Some(text) => Ok(serde_json::from_str(&text)?),
    }
}

/// Checks and saves the settings, with an audit row, in one transaction (Code Standards B3, S7).
///
/// # Errors
/// The errors of [`BackupSettings::validate`]; `ERR_DISK_FULL`, `ERR_DB_BUSY` and the other
/// database codes.
pub fn save(db: &Db, user_id: Option<i64>, new: &BackupSettings) -> Result<(), BackupError> {
    new.validate()?;
    let value = serde_json::to_string(new)?;
    db.write(|tx| {
        let old = load(tx)?;
        tx.execute(
            "INSERT INTO settings (key, value_json, created_by, updated_by) VALUES (?1, ?2, ?3, ?3) \
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, \
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), updated_by = excluded.updated_by, \
             version = version + 1",
            params![SETTINGS_KEY, value, user_id],
        )?;
        if old != *new {
            db.auditor().write(
                tx,
                &AuditEntry {
                    user_id,
                    action: "edit".to_owned(),
                    entity: "backup_settings".to_owned(),
                    entity_id: None,
                    old_json: Some(json!(old)),
                    new_json: Some(json!(new)),
                    reason: None,
                },
            )?;
        }
        Ok::<(), BackupError>(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_time_is_hh_mm() {
        assert_eq!(closing_minutes("21:00"), Some(21 * 60));
        assert_eq!(closing_minutes("00:05"), Some(5));
        for bad in ["9:00", "24:00", "12:60", "12-00", "", "ab:cd"] {
            assert_eq!(closing_minutes(bad), None, "{bad}");
        }
    }

    #[test]
    fn defaults_are_valid() {
        BackupSettings::default().validate().unwrap();
    }

    #[test]
    fn all_problems_come_together_each_on_its_field() {
        let settings = BackupSettings {
            usb_enabled: true,
            usb_folder: String::new(),
            cloud_enabled: true,
            cloud_endpoint: "http://insecure".into(),
            cloud_bucket: String::new(),
            closing_time: "25:00".into(),
            ..BackupSettings::default()
        };
        let error = settings.validate().unwrap_err();
        let found: std::collections::BTreeSet<(String, BackupCode)> = error
            .all()
            .into_iter()
            .map(|e| (e.field.unwrap_or_default(), e.code))
            .collect();
        let expected: std::collections::BTreeSet<(String, BackupCode)> = [
            ("cloud_bucket", BackupCode::ErrRequired),
            ("usb_folder", BackupCode::ErrRequired),
            ("closing_time", BackupCode::ErrOutOfRange),
            ("cloud_endpoint", BackupCode::ErrOutOfRange),
        ]
        .into_iter()
        .map(|(field, code)| (field.to_owned(), code))
        .collect();
        assert_eq!(found, expected);
    }
}
