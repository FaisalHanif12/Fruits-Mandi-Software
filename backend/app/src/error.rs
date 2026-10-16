//! The one error shape every command returns (plan 01 Section 4.2, Code Standards 4.3).
//!
//! The frontend turns each `code` into a message in the app language through
//! `errors.ftl`. No error text is written here (CLAUDE.md rule 7).
//!
//! Example of the JSON a screen receives:
//!
//! ```json
//! { "ok": false,
//!   "errors": [ { "code": "ERR_NOT_IMPLEMENTED", "field": null, "values": {} } ],
//!   "ref": "E-20261010-0001" }
//! ```

use std::collections::BTreeMap;

use serde::{Serialize, Serializer};
use ts_rs::TS;

use crate::refs::next_reference;
use mandi_services::db::DbError;
use mandi_services::{Rejection, ServiceError};

/// The one list of error codes and message values lives in `mandi-core` (plan 02 Section 4.6).
pub use mandi_services::core::error::{ErrorCode, ErrorValue, FieldError};

/// One problem, pointing at the field or grid cell it belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct UiErrorItem {
    /// The error code, for example `ERR_QTY_EXCEEDS_LOT`.
    pub code: ErrorCode,
    /// Path of the field, for example `lots[0].sales[2].qty`. Null for a whole-form error.
    pub field: Option<String>,
    /// Values for the message, for example `{"available": 5}`.
    #[ts(type = "Record<string, string | number>")]
    pub values: BTreeMap<String, ErrorValue>,
}

impl From<FieldError> for UiErrorItem {
    fn from(e: FieldError) -> Self {
        Self {
            code: e.code,
            field: Some(e.field),
            values: e.values,
        }
    }
}

/// Always serialises as `false`, so `ok` can never be `true` on an error.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NotOk;

impl Serialize for NotOk {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(false)
    }
}

/// The error every command returns. Several problems travel together, so the
/// user can fix all of them at once (Code Standards 4.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct UiError {
    /// Always `false`.
    #[ts(type = "false")]
    pub ok: NotOk,
    /// One or more problems.
    pub errors: Vec<UiErrorItem>,
    /// Reference number shown to the user and written to the log (NFR-OBS-3).
    #[serde(rename = "ref")]
    pub reference: String,
}

impl UiError {
    /// An error with one problem.
    pub fn single(code: ErrorCode, field: Option<&str>) -> Self {
        Self::from_items(vec![UiErrorItem {
            code,
            field: field.map(str::to_owned),
            values: BTreeMap::new(),
        }])
    }

    /// An error with several problems, each with its own field.
    ///
    /// The reference number is written to the log with the codes and field paths, never
    /// the values (NFR-OBS-2, NFR-OBS-3).
    pub fn from_items(errors: Vec<UiErrorItem>) -> Self {
        let error = Self::build(errors);
        let codes = error
            .errors
            .iter()
            .map(|e| format!("{:?}", e.code))
            .collect::<Vec<_>>();
        let fields = error
            .errors
            .iter()
            .filter_map(|e| e.field.clone())
            .collect::<Vec<_>>();
        tracing::warn!(
            reference = %error.reference,
            codes = %codes.join(","),
            fields = %fields.join(","),
            "command returned an error"
        );
        error
    }

    /// An internal failure, for example a background task that stopped. The detail is not
    /// sent to the screen. It is written to the log with the reference number.
    pub fn internal<E: std::fmt::Display>(detail: E) -> Self {
        Self::coded(ErrorCode::ErrInternal, &detail)
    }

    /// An error with one code and no field, whose cause goes to the log (Code Standards B12).
    fn coded(code: ErrorCode, detail: &dyn std::fmt::Display) -> Self {
        let error = Self::build(vec![UiErrorItem {
            code,
            field: None,
            values: BTreeMap::new(),
        }]);
        tracing::error!(
            reference = %error.reference,
            code = ?code,
            detail = %detail,
            "command failed"
        );
        error
    }

    fn build(errors: Vec<UiErrorItem>) -> Self {
        Self {
            ok: NotOk,
            errors,
            reference: next_reference(),
        }
    }
}

impl From<Rejection> for UiErrorItem {
    fn from(item: Rejection) -> Self {
        Self {
            code: item.code,
            field: item.field,
            values: item.values,
        }
    }
}

impl From<ServiceError> for UiError {
    /// A rejection keeps its codes, fields and values. A database error keeps its code. Any
    /// other failure is `ERR_INTERNAL` with the cause in the log.
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::Rejected(items) if !items.is_empty() => {
                Self::from_items(items.into_iter().map(UiErrorItem::from).collect())
            }
            ServiceError::Db(error) => Self::from(error),
            other => Self::internal(other),
        }
    }
}

impl From<DbError> for UiError {
    /// The code comes from the database error; the cause goes to the log.
    fn from(error: DbError) -> Self {
        Self::coded(error.code(), &error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serialises_to_the_plan_shape() {
        let error = UiError::single(ErrorCode::ErrNotImplemented, None);
        let value = serde_json::to_value(&error).unwrap();
        let reference = value["ref"].as_str().unwrap().to_owned();
        assert_eq!(
            value,
            json!({
                "ok": false,
                "errors": [ { "code": "ERR_NOT_IMPLEMENTED", "field": null, "values": {} } ],
                "ref": reference,
            })
        );
    }

    #[test]
    fn carries_field_and_values() {
        let mut values = BTreeMap::new();
        values.insert("available".to_owned(), ErrorValue::Number(5));
        values.insert("item".to_owned(), ErrorValue::Text("کیلا".to_owned()));
        let error = UiError::from_items(vec![UiErrorItem {
            code: ErrorCode::ErrInternal,
            field: Some("lots[0].sales[2].qty".to_owned()),
            values,
        }]);
        let value = serde_json::to_value(&error).unwrap();
        assert_eq!(
            value["errors"][0],
            json!({
                "code": "ERR_INTERNAL",
                "field": "lots[0].sales[2].qty",
                "values": { "available": 5, "item": "کیلا" },
            })
        );
    }

    #[test]
    fn field_errors_become_items() {
        let field_error = FieldError::new(ErrorCode::ErrQtyTooLarge, "lots[0].qty")
            .with("max", ErrorValue::Number(1_000_000));
        let error = UiError::from_items(vec![field_error.into()]);
        let value = serde_json::to_value(&error).unwrap();
        assert_eq!(
            value["errors"][0],
            json!({
                "code": "ERR_QTY_TOO_LARGE",
                "field": "lots[0].qty",
                "values": { "max": 1_000_000 },
            })
        );
    }

    #[test]
    fn reference_has_date_and_counter() {
        let reference = UiError::internal("detail").reference;
        // E-YYYYMMDD-NNNN
        assert_eq!(reference.len(), "E-20261010-0001".len());
        assert!(reference.starts_with("E-"));
        let (date, counter) = reference[2..].split_once('-').unwrap();
        assert_eq!(date.len(), 8);
        assert!(date.chars().all(|c| c.is_ascii_digit()));
        assert!(counter.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn database_errors_keep_their_code() {
        let disk = UiError::from(DbError::DiskFull {
            available_bytes: 10,
            needed_bytes: 50,
        });
        assert_eq!(disk.errors[0].code, ErrorCode::ErrDiskFull);
        assert_eq!(
            UiError::from(DbError::Busy).errors[0].code,
            ErrorCode::ErrDbBusy
        );
        let value =
            serde_json::to_value(UiError::from(DbError::DataCheckFailed("page 3".into()))).unwrap();
        assert_eq!(value["errors"][0]["code"], "ERR_DATA_CHECK_FAILED");
        assert!(
            !value.to_string().contains("page 3"),
            "the cause stays in the log"
        );
    }

    #[test]
    fn internal_detail_never_reaches_the_screen() {
        let value = serde_json::to_value(UiError::internal("secret path C:\\x")).unwrap();
        assert!(!value.to_string().contains("secret"));
        assert_eq!(value["errors"][0]["code"], "ERR_INTERNAL");
    }
}
