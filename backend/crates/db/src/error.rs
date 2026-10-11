//! The one error type of `mandi-db` (Code Standards B9).
//!
//! Each variant maps to an error code with [`DbError::code`]. The app turns the code into a
//! message in the app language. No error text is written for the user here (CLAUDE.md rule 7);
//! the `Display` text is for the log only.

use mandi_core::error::ErrorCode;
use rusqlite::ffi::ErrorCode as SqliteCode;
use thiserror::Error;

/// Something went wrong in the database layer.
#[derive(Debug, Error)]
pub enum DbError {
    /// Not enough free disk space. Nothing was written (NFR-REL-10).
    #[error("not enough free disk space: {available_bytes} bytes free, {needed_bytes} needed")]
    DiskFull {
        /// Free bytes on the disk of the data file.
        available_bytes: u64,
        /// The least free space a save needs.
        needed_bytes: u64,
    },
    /// The data file is damaged, or its start-up check failed (NFR-REL-7).
    #[error("data check failed: {0}")]
    DataCheckFailed(String),
    /// A database upgrade failed. The data was put back from the copy (NFR-REL-13).
    #[error("migration failed: {0}")]
    MigrationFailed(String),
    /// Another writer holds the data file and did not let go in time.
    #[error("database busy")]
    Busy,
    /// A journal entry whose debits do not equal its credits was posted (PRD FR-6.2).
    #[error("journal entry does not balance")]
    JournalUnbalanced,
    /// A change to a posted journal entry or its lines (PRD FR-6.3).
    #[error("posted journal entries cannot change")]
    JournalPosted,
    /// An update or delete on an append-only table (Code Standards B17).
    #[error("append-only table")]
    AppendOnly,
    /// A change to a system account that is not allowed (PRD FR-10.4).
    #[error("system account cannot change")]
    SystemAccount,
    /// Another CHECK, NOT NULL, UNIQUE or foreign key rule of the database failed.
    #[error("database rule failed: {0}")]
    Constraint(String),
    /// The audit chain key could not be read or created (Code Standards S3).
    #[error("audit key store: {0}")]
    KeyStore(String),
    /// Any other SQLite error.
    #[error("sqlite: {0}")]
    Sqlite(#[source] rusqlite::Error),
    /// A file or folder problem.
    #[error("io: {0}")]
    Io(#[source] std::io::Error),
    /// JSON that could not be written.
    #[error("json: {0}")]
    Json(#[source] serde_json::Error),
}

impl DbError {
    /// The code for the screen. A rule failure inside the database means the domain layer let
    /// bad data through, which the user cannot fix, so those are `ERR_INTERNAL` with the real
    /// cause in the log. (The ledger plan adds `ERR_JOURNAL_UNBALANCED` for its own check.)
    pub fn code(&self) -> ErrorCode {
        match self {
            DbError::DiskFull { .. } => ErrorCode::ErrDiskFull,
            DbError::DataCheckFailed(_) => ErrorCode::ErrDataCheckFailed,
            DbError::MigrationFailed(_) => ErrorCode::ErrMigrationFailed,
            DbError::Busy => ErrorCode::ErrDbBusy,
            DbError::JournalUnbalanced
            | DbError::JournalPosted
            | DbError::AppendOnly
            | DbError::SystemAccount
            | DbError::Constraint(_)
            | DbError::KeyStore(_)
            | DbError::Sqlite(_)
            | DbError::Io(_)
            | DbError::Json(_) => ErrorCode::ErrInternal,
        }
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(error: rusqlite::Error) -> Self {
        classify(error)
    }
}

impl From<std::io::Error> for DbError {
    fn from(error: std::io::Error) -> Self {
        DbError::Io(error)
    }
}

impl From<serde_json::Error> for DbError {
    fn from(error: serde_json::Error) -> Self {
        DbError::Json(error)
    }
}

/// Sorts a SQLite error into one of our variants. The triggers of `0001_init.sql` raise
/// messages that start with an `ERR_` token.
fn classify(error: rusqlite::Error) -> DbError {
    let rusqlite::Error::SqliteFailure(failure, message) = &error else {
        return DbError::Sqlite(error);
    };
    let text = message.clone().unwrap_or_default();
    match failure.code {
        SqliteCode::DatabaseBusy | SqliteCode::DatabaseLocked => DbError::Busy,
        SqliteCode::DatabaseCorrupt | SqliteCode::NotADatabase => DbError::DataCheckFailed(text),
        SqliteCode::DiskFull => DbError::DiskFull {
            available_bytes: 0,
            needed_bytes: crate::disk::MIN_FREE_DISK_BYTES,
        },
        SqliteCode::ConstraintViolation => {
            if text.contains("ERR_JOURNAL_UNBALANCED") {
                DbError::JournalUnbalanced
            } else if text.contains("ERR_JOURNAL_POSTED") {
                DbError::JournalPosted
            } else if text.contains("ERR_APPEND_ONLY") {
                DbError::AppendOnly
            } else if text.contains("ERR_SYSTEM_ACCOUNT") {
                DbError::SystemAccount
            } else {
                DbError::Constraint(text)
            }
        }
        _ => DbError::Sqlite(error),
    }
}
