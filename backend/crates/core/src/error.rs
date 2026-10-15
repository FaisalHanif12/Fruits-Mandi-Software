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
    /// Not enough free disk space to save safely. Nothing was written (NFR-REL-10).
    ErrDiskFull,
    /// The data file failed its start-up check, so saving is stopped (NFR-REL-7).
    ErrDataCheckFailed,
    /// A database upgrade failed. The data was put back from the copy taken before it (NFR-REL-13).
    ErrMigrationFailed,
    /// The data file is busy with another write and did not free up in time.
    ErrDbBusy,
    /// First-run setup was asked for, but a user already exists (plan 04).
    ErrAlreadySetUp,
    /// A password shorter than 6 characters (NFR-SEC-1). Values: `min`.
    ErrPasswordTooShort,
    /// The password and its repeat differ.
    ErrPasswordsDiffer,
    /// A user name that is not 3 to 32 of `a-z 0-9 . _ -`.
    ErrUsernameInvalid,
    /// Wrong user name or password (the same code for an unknown name). Values: `tries_left`.
    ErrWrongPassword,
    /// Too many wrong tries; sign-in waits (PRD FR-8.2). Values: `seconds` left.
    ErrLockedWait,
    /// The user was made inactive by the Owner.
    ErrUserInactive,
    /// The recovery code, or the user name it belongs to, is wrong (counts as a wrong try).
    ErrRecoveryCodeWrong,
    /// Someone is already signed in, so a second sign-in is refused (PRD FR-2.3).
    ErrAlreadySignedIn,
    /// The session is not locked, so there is nothing to unlock.
    ErrNotLocked,
    /// A setting outside its allowed range, for example idle lock 0 or 500 minutes.
    ErrOutOfRange,
    /// The command needs a signed-in user.
    ErrNotLoggedIn,
    /// The session is locked (idle lock). Unlock with the password first (PRD FR-8.4).
    ErrSessionLocked,
    /// The record asked for does not exist.
    ErrNotFound,
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
    /// A code that is already used by another record of the same list (plan 06).
    ErrCodeTaken,
    /// The record was changed by someone else since the screen read it. Open it again (plan 06).
    ErrVersionChanged,
    /// No free code is left: the 10-digit codes of a master list, or the serials 1 to 8999 of an
    /// account series (PRD FR-10.3, E29). For an account series, `values.control` is the control
    /// code.
    ErrSeriesFull,
    /// A journal entry whose debits do not equal its credits was refused; nothing was saved
    /// (PRD FR-6.2). This is a bug signal, not the user's mistake.
    ErrJournalUnbalanced,
    /// A code in the reserved series: serial 0000 or 9000 to 9999 belong to system accounts
    /// (PRD FR-10.3).
    ErrCodeReserved,
    /// An opening with both a debit and a credit (PRD FR-10.9). Only one side is allowed.
    ErrOpeningBothSides,
    /// "Opening is an advance" on an account that is not a seller, or on an opening without a
    /// debit (PRD FR-15.8).
    ErrAdvanceNotSeller,
    /// This account cannot take an opening balance: an income or shop expense account (it would
    /// need partner share rows, PRD FR-6.5), ابتدائی فرق itself, or a پختہ کھاتہ account (it has
    /// no person to hold it). Waiting for Umar's decision.
    ErrOpeningNotAllowed,
    /// The account is not a seller (a پیشگی goes to a seller only), or is the پختہ کھاتہ account,
    /// which has no person to hold the advance.
    ErrNotASeller,
    /// The account is hidden from pickers, so a new posting cannot use it (PRD FR-10.2).
    ErrAccountInactive,
    /// A reason is required (reopening a day, PRD FR-21.2).
    ErrReasonRequired,
    /// The trial balance does not balance, so the day cannot be closed (NFR-REL-8).
    ErrTrialNotBalanced,
    /// The day is not closed, so there is nothing to reopen.
    ErrDayNotClosed,
    /// Every day up to today is already closed, so there is no open day to close.
    ErrNothingToClose,
    /// The month was closed (PRD FR-17.6); its month close must be reversed first.
    ErrMonthClosed,
    /// A rule that PRD 11.2 or 11.3 does not allow: a method the charge does not have, a charge
    /// the side does not have, a party on the wrong side, or a group together with a party or an
    /// item. The field says which part.
    ErrRuleNotAllowed,
    /// The last day of a rule is before its first day.
    ErrDateRange,
    /// A value that is not in the right form, for example a code with letters or a choice that
    /// is not offered (plan 06).
    ErrFormatInvalid,
    /// A name holds a word that only the spelling rules could have made and the user has not
    /// confirmed it (PRD FR-7.66, E49). Values: `words` (the words, separated by one space).
    ErrSpellingUnconfirmed,
    /// Active partner shares do not total 100% on the posting date (PRD FR-17.1, E9).
    #[serde(rename = "ERR_SHARES_NOT_100")]
    ErrSharesNot100,
    /// One line of a journal entry, or the total of the entry, is above what one entry may hold
    /// (10^12 rupees). A single typed amount is held to a lower limit by `ErrAmountTooLarge`, so
    /// this means the amounts of several lines add up past the limit. Values: `max` (rupees).
    ErrEntryTooLarge,
    /// A system account (روکڑ, ابتدائی فرق, commission income, نقدی منڈی, the two پختہ کھاتہ
    /// accounts ...) cannot be made inactive: the app posts to it by itself (PRD FR-10.4).
    /// Field `id`. The Active box of such an account is shown ticked and cannot be changed.
    ErrSystemAccount,
}

impl ErrorCode {
    /// Every code, in the order above. Tests use it to check the exported list.
    pub const ALL: [ErrorCode; 64] = [
        ErrorCode::ErrNotImplemented,
        ErrorCode::ErrInternal,
        ErrorCode::ErrDiskFull,
        ErrorCode::ErrDataCheckFailed,
        ErrorCode::ErrMigrationFailed,
        ErrorCode::ErrDbBusy,
        ErrorCode::ErrAlreadySetUp,
        ErrorCode::ErrPasswordTooShort,
        ErrorCode::ErrPasswordsDiffer,
        ErrorCode::ErrUsernameInvalid,
        ErrorCode::ErrWrongPassword,
        ErrorCode::ErrLockedWait,
        ErrorCode::ErrUserInactive,
        ErrorCode::ErrRecoveryCodeWrong,
        ErrorCode::ErrAlreadySignedIn,
        ErrorCode::ErrNotLocked,
        ErrorCode::ErrOutOfRange,
        ErrorCode::ErrNotLoggedIn,
        ErrorCode::ErrSessionLocked,
        ErrorCode::ErrNotFound,
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
        ErrorCode::ErrCodeTaken,
        ErrorCode::ErrVersionChanged,
        ErrorCode::ErrSeriesFull,
        ErrorCode::ErrJournalUnbalanced,
        ErrorCode::ErrCodeReserved,
        ErrorCode::ErrOpeningBothSides,
        ErrorCode::ErrAdvanceNotSeller,
        ErrorCode::ErrOpeningNotAllowed,
        ErrorCode::ErrNotASeller,
        ErrorCode::ErrAccountInactive,
        ErrorCode::ErrReasonRequired,
        ErrorCode::ErrTrialNotBalanced,
        ErrorCode::ErrDayNotClosed,
        ErrorCode::ErrNothingToClose,
        ErrorCode::ErrMonthClosed,
        ErrorCode::ErrRuleNotAllowed,
        ErrorCode::ErrDateRange,
        ErrorCode::ErrFormatInvalid,
        ErrorCode::ErrSpellingUnconfirmed,
        ErrorCode::ErrOverflow,
        ErrorCode::ErrSharesNot100,
        ErrorCode::ErrEntryTooLarge,
        ErrorCode::ErrSystemAccount,
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
