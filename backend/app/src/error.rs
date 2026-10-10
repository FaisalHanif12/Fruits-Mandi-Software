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
use std::sync::atomic::{AtomicU32, Ordering};

use serde::{Serialize, Serializer};
use ts_rs::TS;

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

/// Counter for reference numbers in this run of the app.
static NEXT_REF: AtomicU32 = AtomicU32::new(1);

/// Makes a reference number like `E-20261010-0001`.
///
/// Plan 01 uses today's local date and a counter. Plan 03 ties it to the log line.
fn next_reference() -> String {
    let counter = NEXT_REF.fetch_add(1, Ordering::Relaxed);
    let today = chrono::Local::now().format("%Y%m%d");
    format!("E-{today}-{counter:04}")
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
    pub fn from_items(errors: Vec<UiErrorItem>) -> Self {
        Self {
            ok: NotOk,
            errors,
            reference: next_reference(),
        }
    }

    /// An internal failure, for example a background task that stopped.
    /// The detail is not sent to the screen. Plan 03 writes it to the log.
    pub fn internal<E: std::fmt::Display>(_detail: E) -> Self {
        Self::single(ErrorCode::ErrInternal, None)
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
    fn internal_detail_never_reaches_the_screen() {
        let value = serde_json::to_value(UiError::internal("secret path C:\\x")).unwrap();
        assert!(!value.to_string().contains("secret"));
        assert_eq!(value["errors"][0]["code"], "ERR_INTERNAL");
    }
}
