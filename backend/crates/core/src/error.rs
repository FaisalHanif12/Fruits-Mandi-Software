//! Error codes and field errors shared by every crate (Code Standards 4.3, plan 02 Section 4.6).
//!
//! The backend sends codes, never text. The frontend turns each code into a message in the
//! app language through `errors.ftl` (CLAUDE.md rule 7). This is the one list of codes:
//! the app re-exports it and ts-rs exports it to `frontend/src/bindings/ErrorCode.ts`.
//!
//! Example: the qty cell of the third sale line holds `"abc"`. The command returns
//! `FieldError { code: ErrNotANumber, field: "lots[0].sales[2].qty", values: {} }`.

use std::collections::BTreeMap;

use serde::Serialize;
use ts_rs::TS;

/// Every error code the backend can send. Serialised as `ERR_` + SCREAMING_SNAKE_CASE.
/// Each code needs a message in `frontend/src/i18n/{ur,en}/errors.ftl`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// The command exists in the contract but is not built yet (task T1 stubs).
    ErrNotImplemented,
    /// Something failed inside the backend that the user cannot fix by changing input.
    ErrInternal,
    /// A required field is empty.
    ErrRequired,
    /// The text is not a number, or not a simple sum such as `20+25` (PRD FR-7.48).
    ErrNotANumber,
    /// Money or a rate was typed with a decimal point (PRD Section 22, E31).
    ErrDecimalsNotAllowed,
    /// A negative value, or a simple sum below 0 (PRD E42).
    ErrNegativeNotAllowed,
    /// Money above 10,00,00,00,000 rupees. Values: `max` (rupees).
    ErrAmountTooLarge,
    /// A rate, quantity or weight of 0 (PRD E24).
    ErrMustBeMoreThanZero,
    /// A percent with more than 2 decimals, or a weight with more than 3. Values: `max` (decimals).
    ErrTooManyDecimals,
    /// A percent above 100.
    #[serde(rename = "ERR_PERCENT_OVER_100")]
    ErrPercentOver100,
    /// A quantity above 1,000,000 نگ. Values: `max`.
    ErrQtyTooLarge,
    /// A quantity (نگ) with decimals, for example `2.5`. Money uses `ErrDecimalsNotAllowed`.
    ErrQtyNotWhole,
    /// A weight above 100,000 kg. Values: `max` (kg).
    ErrWeightTooLarge,
    /// Not a real date, or not in a form the date field takes (PRD FR-7.19).
    ErrDateInvalid,
    /// The date is on or before the "closed up to" date (PRD FR-6.6). Values: `closed_up_to`.
    ErrDateClosed,
    /// The date is after today (PRD FR-6.6). Values: `today`.
    ErrDateInFuture,
    /// An account code that is not 1 to 4 digits (short code) or 8 or 9 digits (full code).
    ErrCodeInvalid,
    /// The Urdu name has a Latin letter or a character that is not allowed (PRD FR-7.28).
    ErrNameUrScript,
    /// The English name has an Urdu letter or a character that is not allowed (PRD FR-7.28).
    ErrNameEnScript,
    /// A name longer than 100 characters. Values: `max`.
    ErrNameTooLong,
    /// A phone number that is not `03XXXXXXXXX` or `03XX-XXXXXXX`.
    ErrPhoneInvalid,
    /// Free text longer than 500 characters. Values: `max`.
    ErrTextTooLong,
    /// Text with control characters (for example a tab or a line break in a name).
    ErrControlChars,
    /// A money total grew past what the app can hold (Code Standards M7).
    ErrOverflow,
    /// Active partner shares do not total 100% on the posting date (PRD FR-17.1, E9).
    #[serde(rename = "ERR_SHARES_NOT_100")]
    ErrSharesNot100,
}

impl ErrorCode {
    /// Every code, in the order above. Tests use it to check the exported list.
    pub const ALL: [ErrorCode; 25] = [
        ErrorCode::ErrNotImplemented,
        ErrorCode::ErrInternal,
        ErrorCode::ErrRequired,
        ErrorCode::ErrNotANumber,
        ErrorCode::ErrDecimalsNotAllowed,
        ErrorCode::ErrNegativeNotAllowed,
        ErrorCode::ErrAmountTooLarge,
        ErrorCode::ErrMustBeMoreThanZero,
        ErrorCode::ErrTooManyDecimals,
        ErrorCode::ErrPercentOver100,
        ErrorCode::ErrQtyTooLarge,
        ErrorCode::ErrQtyNotWhole,
        ErrorCode::ErrWeightTooLarge,
        ErrorCode::ErrDateInvalid,
        ErrorCode::ErrDateClosed,
        ErrorCode::ErrDateInFuture,
        ErrorCode::ErrCodeInvalid,
        ErrorCode::ErrNameUrScript,
        ErrorCode::ErrNameEnScript,
        ErrorCode::ErrNameTooLong,
        ErrorCode::ErrPhoneInvalid,
        ErrorCode::ErrTextTooLong,
        ErrorCode::ErrControlChars,
        ErrorCode::ErrOverflow,
        ErrorCode::ErrSharesNot100,
    ];
}

/// One value that goes into an error message, for example `{"available": 5}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum ErrorValue {
    /// A text value, for example a code or a date.
    Text(String),
    /// A whole number, for example a quantity.
    Number(i64),
}

/// One problem with one input field. Same shape as the app's `UiErrorItem` (plan 01 Section 4.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldError {
    /// The error code.
    pub code: ErrorCode,
    /// Path of the field, for example `lots[0].sales[2].qty`.
    pub field: String,
    /// Values for the message, for example `{"max": 100}`.
    pub values: BTreeMap<String, ErrorValue>,
}

impl FieldError {
    /// A field error with no values.
    pub fn new(code: ErrorCode, field: &str) -> Self {
        Self {
            code,
            field: field.to_owned(),
            values: BTreeMap::new(),
        }
    }

    /// Adds one value for the message and returns the error.
    #[must_use]
    pub fn with(mut self, key: &str, value: ErrorValue) -> Self {
        self.values.insert(key.to_owned(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_serialise_as_err_names() {
        let text = serde_json::to_string(&ErrorCode::ErrNotANumber).unwrap();
        assert_eq!(text, "\"ERR_NOT_A_NUMBER\"");
        let text = serde_json::to_string(&ErrorCode::ErrPercentOver100).unwrap();
        assert_eq!(text, "\"ERR_PERCENT_OVER_100\"");
        let text = serde_json::to_string(&ErrorCode::ErrSharesNot100).unwrap();
        assert_eq!(text, "\"ERR_SHARES_NOT_100\"");
    }

    #[test]
    fn every_code_is_listed_once() {
        let mut seen = std::collections::BTreeSet::new();
        for code in ErrorCode::ALL {
            assert!(seen.insert(code), "{code:?} twice");
        }
        let decl = ErrorCode::decl(&ts_rs::Config::new());
        assert_eq!(
            decl.matches("\"ERR_").count(),
            ErrorCode::ALL.len(),
            "{decl}"
        );
    }

    #[test]
    fn field_error_shape() {
        let error = FieldError::new(ErrorCode::ErrQtyTooLarge, "lots[0].qty")
            .with("max", ErrorValue::Number(1_000_000));
        let value = serde_json::to_value(&error).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "code": "ERR_QTY_TOO_LARGE",
                "field": "lots[0].qty",
                "values": { "max": 1_000_000 },
            })
        );
    }
}
