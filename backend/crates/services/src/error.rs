//! The one error type of `mandi-services` (Code Standards B9).
//!
//! A service either rejects the input with codes and fields the screen can show, or fails
//! for a reason the user cannot fix (database, random source, hashing). The app turns the
//! first into field errors and the second into `ERR_INTERNAL` with a reference number.

use std::collections::BTreeMap;

use mandi_db::DbError;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use thiserror::Error;

/// One problem to show to the user: a code, the field it belongs to (if any) and the values
/// its message needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    /// The error code.
    pub code: ErrorCode,
    /// Path of the field, for example `password_repeat`. `None` for a whole-form error.
    pub field: Option<String>,
    /// Values for the message, for example `{"tries_left": 3}`.
    pub values: BTreeMap<String, ErrorValue>,
}

impl From<FieldError> for Rejection {
    fn from(error: FieldError) -> Self {
        Self {
            code: error.code,
            field: Some(error.field),
            values: error.values,
        }
    }
}

/// What a service can return instead of a result.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// The input or the state does not allow this. The user can act on it.
    #[error("rejected: {0:?}")]
    Rejected(Vec<Rejection>),
    /// The database failed.
    #[error(transparent)]
    Db(#[from] DbError),
    /// The password hasher failed (a bad parameter or hash). Not the user's mistake.
    #[error("password hashing failed: {0}")]
    Hash(String),
    /// The system has no random source.
    #[error("no random source: {0}")]
    Random(String),
    /// Stored data is not in the shape this code wrote (for example a settings value).
    #[error("stored data is wrong: {0}")]
    Data(String),
}

impl ServiceError {
    /// A rejection with a code only.
    pub fn reject(code: ErrorCode) -> Self {
        Self::Rejected(vec![Rejection {
            code,
            field: None,
            values: BTreeMap::new(),
        }])
    }

    /// A rejection that points at a field.
    pub fn reject_field(code: ErrorCode, field: &str) -> Self {
        Self::Rejected(vec![Rejection {
            code,
            field: Some(field.to_owned()),
            values: BTreeMap::new(),
        }])
    }

    /// A rejection with a code and one number for its message.
    pub fn reject_with(code: ErrorCode, field: Option<&str>, key: &str, value: i64) -> Self {
        let mut values = BTreeMap::new();
        values.insert(key.to_owned(), ErrorValue::Number(value));
        Self::Rejected(vec![Rejection {
            code,
            field: field.map(str::to_owned),
            values,
        }])
    }

    /// The code the screen sees. For several rejections, the first one.
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Rejected(items) => items
                .first()
                .map_or(ErrorCode::ErrInternal, |item| item.code),
            Self::Db(error) => error.code(),
            Self::Hash(_) | Self::Random(_) | Self::Data(_) => ErrorCode::ErrInternal,
        }
    }

    /// True if this is a rejection with this code (tests and callers use it).
    pub fn has_code(&self, code: ErrorCode) -> bool {
        match self {
            Self::Rejected(items) => items.iter().any(|item| item.code == code),
            other => other.code() == code,
        }
    }
}

impl From<FieldError> for ServiceError {
    fn from(error: FieldError) -> Self {
        Self::Rejected(vec![error.into()])
    }
}

impl From<Vec<FieldError>> for ServiceError {
    fn from(errors: Vec<FieldError>) -> Self {
        Self::Rejected(errors.into_iter().map(Rejection::from).collect())
    }
}

impl From<mandi_db::rusqlite::Error> for ServiceError {
    fn from(error: mandi_db::rusqlite::Error) -> Self {
        Self::Db(DbError::from(error))
    }
}
