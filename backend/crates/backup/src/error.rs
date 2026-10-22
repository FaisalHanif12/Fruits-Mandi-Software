//! The one error type of `mandi-backup` (Code Standards B9).
//!
//! Each error carries a code for the screen. No user text is written here (CLAUDE.md rule 7);
//! the `detail` is for the log only and never reaches the screen (Code Standards B12).
//!
//! The codes are this crate's own list, [`BackupCode`]. A few of them have the same text as a
//! code of `mandi-core` (for example `ERR_DISK_FULL`), so the screen finds one message for both.
//! Run B may not change `mandi-core`, so the backup codes live here until the lead moves them.

use std::collections::BTreeMap;
use std::fmt;

use mandi_core::error::{ErrorCode, ErrorValue};
use mandi_db::DbError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

/// Every error code the backup commands can send. Serialised as `ERR_` + SCREAMING_SNAKE_CASE.
/// Each code needs a message in `frontend/src/i18n/{ur,en}/errors.ftl`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BackupCode {
    /// Not enough free disk space for the copy. Nothing was written (NFR-REL-10). Same text as
    /// the `mandi-core` code of the same name.
    ErrDiskFull,
    /// Something failed that the user cannot fix by changing input. Same text as the
    /// `mandi-core` code of the same name.
    ErrInternal,
    /// The data file failed its check. Same text as the `mandi-core` code of the same name.
    ErrDataCheckFailed,
    /// The data file is busy. Same text as the `mandi-core` code of the same name.
    ErrDbBusy,
    /// A required field is empty. Same text as the `mandi-core` code of the same name.
    ErrRequired,
    /// A setting or an id is not in the allowed form. Same text as the `mandi-core` code of the
    /// same name. Field and values say which.
    ErrOutOfRange,
    /// The upgrade of a restored copy failed. Same text as the `mandi-core` code of the same
    /// name (NFR-REL-13).
    ErrMigrationFailed,
    /// The backup asked for does not exist (any more).
    ErrBackupNotFound,
    /// The backup key is not set up yet, so USB and cloud copies cannot be made (NFR-BAK-6).
    ErrBackupKeyNotSet,
    /// The key was already created. It is shown once and cannot be shown again (FR-20.10).
    ErrBackupKeyAlreadySet,
    /// The typed key is not 24 letters and digits. Field `key`.
    ErrBackupKeyFormat,
    /// The typed key does not open this backup, or is not the key of this computer. Field `key`.
    /// Nothing was changed.
    ErrBackupKeyWrong,
    /// The backup is encrypted and this computer has no key for it. Type the printed key.
    /// Field `key`.
    ErrBackupKeyRequired,
    /// The backup file is damaged or was changed: its checksum, its integrity check or its
    /// counts do not match. It is never restored (FR-20.7, NFR-BAK-5).
    ErrBackupCorrupt,
    /// A copy was made but failed its check right after (FR-20.7). The copy is marked bad.
    ErrBackupCheckFailed,
    /// The cloud copy is not set up: no account details, or the cloud copy is off.
    ErrBackupCloudNotConfigured,
    /// The cloud did not answer or refused (no internet, wrong keys). The copy waits in the
    /// queue and is tried again every 10 minutes (NFR-BAK-9). Value `reason`.
    ErrBackupCloudFailed,
    /// The USB folder is not there: the drive is not plugged in (FR-20.4).
    ErrBackupUsbNotPresent,
    /// Writing to the USB drive failed, for example the drive was pulled out half way. No
    /// half-written copy is left.
    ErrBackupUsbFailed,
    /// Writing the local copy failed. No half-written copy is left.
    ErrBackupLocalFailed,
    /// Another backup or restore is running. Try again in a moment.
    ErrBackupBusy,
    /// A restore was asked for without the confirmation (FR-20.9). Field `confirmed`.
    ErrBackupNotConfirmed,
    /// The backup was made by a newer version of the app than this one. It is not restored.
    ErrBackupNewerThanApp,
    /// The backup is not running: the data file did not open, so there is nothing to back up.
    ErrBackupNotStarted,
    /// The restore failed after the safety copy. The data file is as it was. The safety copy
    /// is kept.
    ErrBackupRestoreFailed,
}

/// A failure of the backup crate: a code, the field it belongs to, values for the message, and
/// the cause for the log.
#[derive(Debug, Error)]
#[error("{code:?}: {detail}")]
pub struct BackupError {
    /// The code for the screen.
    pub code: BackupCode,
    /// The field it belongs to, when it points at one.
    pub field: Option<String>,
    /// Values for the message, for example `{"reason": "offline"}`.
    pub values: BTreeMap<String, ErrorValue>,
    /// The cause, for the log only.
    pub detail: String,
    /// Further problems found together with this one, so the user can fix them all at once
    /// (Code Standards 4.3).
    pub also: Vec<BackupError>,
}

impl BackupError {
    /// An error with a code and the cause for the log.
    pub fn new(code: BackupCode, detail: impl fmt::Display) -> Self {
        Self {
            code,
            field: None,
            values: BTreeMap::new(),
            detail: detail.to_string(),
            also: Vec::new(),
        }
    }

    /// This error and the ones found with it, each on its own.
    pub fn all(mut self) -> Vec<BackupError> {
        let rest = std::mem::take(&mut self.also);
        let mut items = vec![self];
        items.extend(rest);
        items
    }

    /// One error from a list of problems found together. `None` for an empty list.
    pub fn from_many(mut items: Vec<BackupError>) -> Option<BackupError> {
        if items.is_empty() {
            return None;
        }
        let mut first = items.remove(0);
        first.also.extend(items);
        Some(first)
    }

    /// The same error, pointing at a field.
    #[must_use]
    pub fn on_field(mut self, field: &str) -> Self {
        self.field = Some(field.to_owned());
        self
    }

    /// The same error, with one more value for the message.
    #[must_use]
    pub fn with(mut self, name: &str, value: ErrorValue) -> Self {
        self.values.insert(name.to_owned(), value);
        self
    }

    /// An internal failure. The cause goes to the log.
    pub fn internal(detail: impl fmt::Display) -> Self {
        Self::new(BackupCode::ErrInternal, detail)
    }

    /// A file or folder problem that is not a full disk.
    pub fn io(context: &str, error: &std::io::Error) -> Self {
        Self::new(BackupCode::ErrInternal, format!("{context}: {error}"))
    }
}

impl From<DbError> for BackupError {
    fn from(error: DbError) -> Self {
        let code = match error.code() {
            ErrorCode::ErrDiskFull => BackupCode::ErrDiskFull,
            ErrorCode::ErrDataCheckFailed => BackupCode::ErrDataCheckFailed,
            ErrorCode::ErrDbBusy => BackupCode::ErrDbBusy,
            ErrorCode::ErrMigrationFailed => BackupCode::ErrMigrationFailed,
            _ => BackupCode::ErrInternal,
        };
        Self::new(code, error)
    }
}

impl From<mandi_db::rusqlite::Error> for BackupError {
    fn from(error: mandi_db::rusqlite::Error) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<std::io::Error> for BackupError {
    fn from(error: std::io::Error) -> Self {
        Self::io("file", &error)
    }
}

impl From<serde_json::Error> for BackupError {
    fn from(error: serde_json::Error) -> Self {
        Self::internal(format!("json: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_serialise_with_the_err_prefix() {
        let text = serde_json::to_string(&BackupCode::ErrBackupKeyWrong).unwrap();
        assert_eq!(text, "\"ERR_BACKUP_KEY_WRONG\"");
        let text = serde_json::to_string(&BackupCode::ErrDiskFull).unwrap();
        assert_eq!(text, "\"ERR_DISK_FULL\"", "same text as the core code");
    }

    #[test]
    fn database_errors_keep_their_meaning() {
        let error = BackupError::from(DbError::DiskFull {
            available_bytes: 1,
            needed_bytes: 2,
        });
        assert_eq!(error.code, BackupCode::ErrDiskFull);
        let error = BackupError::from(DbError::Busy);
        assert_eq!(error.code, BackupCode::ErrDbBusy);
    }

    #[test]
    fn the_cause_is_in_display_but_not_in_the_values() {
        let error = BackupError::new(BackupCode::ErrBackupCorrupt, "page 3")
            .on_field("backup")
            .with("reason", ErrorValue::Text("checksum".into()));
        assert!(error.to_string().contains("page 3"));
        assert_eq!(error.field.as_deref(), Some("backup"));
        assert!(!format!("{:?}", error.values).contains("page 3"));
    }
}
