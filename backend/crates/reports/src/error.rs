//! The one error type of `mandi-reports` (Code Standards B9). The app turns it into an error
//! code for the screen. No error text for the user is written here (CLAUDE.md rule 7).

use mandi_core::error::ErrorCode;
use mandi_core::money::MoneyError;
use mandi_core::urdu_words::WordsError;
use mandi_db::DbError;
use thiserror::Error;

/// Something went wrong while building a report or print data.
#[derive(Debug, Error)]
pub enum ReportsError {
    /// The record asked for does not exist (`ERR_NOT_FOUND`, field `bikri_id`).
    #[error("record not found")]
    NotFound,
    /// The database failed.
    #[error(transparent)]
    Db(#[from] DbError),
    /// A sum of stored amounts does not fit. Stored data is checked, so this is a bug signal.
    #[error("money: {0}")]
    Money(#[from] MoneyError),
    /// A stored amount cannot be written in words (outside 0 to 10,00,00,00,000).
    #[error("amount in words: {0}")]
    Words(#[from] WordsError),
    /// Stored data is not in the shape this code wrote.
    #[error("stored data is wrong: {0}")]
    Data(String),
}

impl ReportsError {
    /// The code the screen sees. Everything except a missing record is an internal failure.
    pub fn code(&self) -> ErrorCode {
        match self {
            ReportsError::NotFound => ErrorCode::ErrNotFound,
            ReportsError::Db(error) => error.code(),
            // A sum that does not fit is `ERR_OVERFLOW`; the money bug variants keep their own
            // codes (`MoneyError::code`). Words only fail for a page total above
            // 10,00,00,00,000 rupees, which is also a clear refusal and not a bug.
            ReportsError::Money(error) => error.code(),
            ReportsError::Words(_) => ErrorCode::ErrOverflow,
            ReportsError::Data(_) => ErrorCode::ErrInternal,
        }
    }
}

impl From<mandi_db::rusqlite::Error> for ReportsError {
    fn from(error: mandi_db::rusqlite::Error) -> Self {
        Self::Db(DbError::from(error))
    }
}
