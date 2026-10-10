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

/// Every error code the backend can send. Serialised as `ERR_` + SCREAMING_SNAKE_CASE.
/// Each code needs a message in `frontend/src/i18n/{ur,en}/errors.ftl`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// The command exists in the contract but is not built yet (task T1 stubs).
    ErrNotImplemented,
    /// Something failed inside the backend that the user cannot fix by changing input.
    ErrInternal,
}

/// One value that goes into an error message, for example `{"available": 5}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum ErrorValue {
    /// A text value, for example a code or a name.
    Text(String),
    /// A whole number, for example a quantity.
    Number(i64),
}

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
