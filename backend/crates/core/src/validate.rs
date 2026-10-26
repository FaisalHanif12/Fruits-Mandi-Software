//! Input checks for every field type (Code Standards 4.1 layer 2, 4.2 and 4.3).
//!
//! The backend treats everything from the screen as untrusted (rule V0). Each function takes
//! the field path and the typed text and returns the parsed value or a `FieldError` with a
//! code, the field and any values for the message. `FieldErrors` collects several errors,
//! so a command can return them all at once.
//!
//! Example: `validate::qty("lots[0].sales[2].qty", "20+25")` is `Ok(Qty(45))`;
//! with `"abc"` it is `Err(FieldError { code: ErrNotANumber, field: "lots[0].sales[2].qty", .. })`.

use unicode_normalization::UnicodeNormalization;

use crate::dates::{self, BusinessDate};
use crate::error::{ErrorCode, ErrorValue, FieldError};
use crate::money::{self, Bps, Grams, MAX_QTY, MAX_RUPEES, MAX_WEIGHT_GRAMS, Paisa, Qty};
use crate::text;

/// The longest name, in characters (Code Standards 4.2).
pub const MAX_NAME_CHARS: usize = 100;
/// The longest free text, in characters (Code Standards 4.2).
pub const MAX_TEXT_CHARS: usize = 500;
/// The longest tag (نشان) of an account, in characters (PRD 10.2, Code Standards 4.2).
pub const MAX_TAG_CHARS: usize = 4;
/// Decimals a percent may have.
const PERCENT_DECIMALS: i64 = 2;
/// Decimals a weight may have.
const WEIGHT_DECIMALS: i64 = 3;
/// Grams in one kg, for the weight limit value.
const GRAMS_PER_KG: i64 = 1_000;

/// Simple sums in quantity fields (PRD FR-7.48).
pub use crate::money::parse_sum_qty;
/// Simple sums in money fields (PRD FR-7.48, Code Standards 4.2 `parse_sum`).
pub use crate::money::parse_sum_rupees as parse_sum;

/// Adds the values a message needs for a code: the maximum for "too large" codes.
fn error(code: ErrorCode, field: &str) -> FieldError {
    let max = match code {
        ErrorCode::ErrAmountTooLarge => Some(MAX_RUPEES),
        ErrorCode::ErrQtyTooLarge => Some(MAX_QTY),
        ErrorCode::ErrWeightTooLarge => MAX_WEIGHT_GRAMS.checked_div(GRAMS_PER_KG),
        ErrorCode::ErrTooManyDecimals => Some(PERCENT_DECIMALS),
        ErrorCode::ErrNameTooLong => i64::try_from(MAX_NAME_CHARS).ok(),
        ErrorCode::ErrTextTooLong => i64::try_from(MAX_TEXT_CHARS).ok(),
        ErrorCode::ErrTagInvalid => i64::try_from(MAX_TAG_CHARS).ok(),
        _ => None,
    };
    let e = FieldError::new(code, field);
    match max {
        Some(m) => e.with("max", ErrorValue::Number(m)),
        None => e,
    }
}

/// A required money field (Code Standards 4.2). Accepts a simple sum.
pub fn money(field: &str, input: &str) -> Result<Paisa, FieldError> {
    money::parse_sum_rupees(input).map_err(|c| error(c, field))
}

/// True if the text is empty once spaces and the invisible marks of number text are removed
/// (`text::is_ignorable_in_number`). The check comes after the marks are removed, so a field
/// holding only a pasted zero-width mark is empty, not `ERR_REQUIRED` (finding R02-5).
fn is_blank(input: &str) -> bool {
    input
        .chars()
        .all(|c| c.is_whitespace() || text::is_ignorable_in_number(c))
}

/// An optional money field: empty text, or text with only invisible marks, is `None`.
pub fn money_optional(field: &str, input: &str) -> Result<Option<Paisa>, FieldError> {
    if is_blank(input) {
        return Ok(None);
    }
    money(field, input).map(Some)
}

/// A rate per نگ or per kg: money that must be more than 0 (PRD E24).
pub fn rate(field: &str, input: &str) -> Result<Paisa, FieldError> {
    let value = money(field, input)?;
    if value == Paisa::ZERO {
        return Err(error(ErrorCode::ErrMustBeMoreThanZero, field));
    }
    Ok(value)
}

/// A percent, 0 to 100 with up to 2 decimals, as bps.
pub fn percent(field: &str, input: &str) -> Result<Bps, FieldError> {
    money::parse_bps(input).map_err(|c| error(c, field))
}

/// A quantity in نگ, 1 to 1,000,000. Accepts a simple sum (`20+25`).
pub fn qty(field: &str, input: &str) -> Result<Qty, FieldError> {
    money::parse_sum_qty(input).map_err(|c| error(c, field))
}

/// A weight in kg with up to 3 decimals, more than 0, as grams.
pub fn weight(field: &str, input: &str) -> Result<Grams, FieldError> {
    money::parse_weight(input).map_err(|c| match c {
        ErrorCode::ErrTooManyDecimals => {
            FieldError::new(c, field).with("max", ErrorValue::Number(WEIGHT_DECIMALS))
        }
        _ => error(c, field),
    })
}

/// A date typed as `DD-MM-YYYY` or a shortcut (PRD FR-7.19).
pub fn date(field: &str, input: &str, today: BusinessDate) -> Result<BusinessDate, FieldError> {
    dates::parse_ddmmyyyy(input, today).map_err(|c| error(c, field))
}

/// A posting date: a valid date after the "closed up to" date and not after today
/// (PRD FR-6.6). The error carries the date that limits it, as `DD-MM-YYYY`.
pub fn posting_date(
    field: &str,
    input: &str,
    closed_up_to: Option<BusinessDate>,
    today: BusinessDate,
) -> Result<BusinessDate, FieldError> {
    let value = date(field, input, today)?;
    dates::check_posting_date(value, closed_up_to, today).map_err(|code| {
        let limit = match (code, closed_up_to) {
            (ErrorCode::ErrDateClosed, Some(closed)) => ("closed_up_to", closed),
            _ => ("today", today),
        };
        FieldError::new(code, field)
            .with(limit.0, ErrorValue::Text(dates::format_ddmmyyyy(limit.1)))
    })?;
    Ok(value)
}

/// What the user typed in an account code field (Code Standards 4.2, PRD 1.8 R5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountCodeInput {
    /// A full code: a code of the scheme of plan 32 (`0010`, `00111`: the type number and a running
    /// number, 4 to 10 digits, starting with `00`), or the 8 or 9 digits of any other account
    /// (control code + 4-digit serial).
    Full(String),
    /// A short code: the running number alone, digits with no leading zero, up to 7 digits. Services
    /// resolve it by field type.
    Short(ShortCode),
}

/// A running number typed as a short code, 0 to 9,999,999. `"0"` is the first account of its type
/// (`0010` in a seller field) and `"11"` is `00111`. There is no padding: `"011"` is not a short
/// code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShortCode(u32);

impl ShortCode {
    /// The running number.
    pub fn serial(self) -> u32 {
        self.0
    }
}

/// Lengths of a full code of an account that is not a seller or a buyer.
const FULL_CODE_LENGTHS: [usize; 2] = [8, 9];

/// An account code. Whether it exists and is active is checked by the service.
pub fn account_code(field: &str, input: &str) -> Result<AccountCodeInput, FieldError> {
    let digits = digit_text(input);
    if digits.is_empty() {
        return Err(error(ErrorCode::ErrRequired, field));
    }
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(error(ErrorCode::ErrCodeInvalid, field));
    }
    // A running number never starts with 0 (except `0` itself) and a code of the scheme starts with
    // 00, so the two can never be mixed up.
    if let Some(running) = crate::codes::parse_running(&digits)
        && let Ok(serial) = u32::try_from(running)
    {
        return Ok(AccountCodeInput::Short(ShortCode(serial)));
    }
    if crate::codes::parse_code(&digits).is_some() || FULL_CODE_LENGTHS.contains(&digits.len()) {
        return Ok(AccountCodeInput::Full(digits));
    }
    Err(error(ErrorCode::ErrCodeInvalid, field))
}

/// Urdu (۰-۹) and Arabic-Indic (٠-٩) digits to 0-9; other characters unchanged.
fn ascii_digit(c: char) -> char {
    let zero = match c {
        '\u{06F0}'..='\u{06F9}' => '\u{06F0}',
        '\u{0660}'..='\u{0669}' => '\u{0660}',
        _ => return c,
    };
    char::from_digit(u32::from(c).saturating_sub(u32::from(zero)), 10).unwrap_or(c)
}

/// Text of a code or phone field as ASCII digits: the invisible marks of number text
/// (`text::is_ignorable_in_number`) are removed first, as in every number field, then the
/// text is trimmed and Urdu and Arabic-Indic digits become 0-9. RLM, ALM and the bidi
/// controls stay, so they make the code or phone invalid (finding R02-7).
fn digit_text(input: &str) -> String {
    let visible: String = input
        .chars()
        .filter(|&c| !text::is_ignorable_in_number(c))
        .collect();
    visible.trim().chars().map(ascii_digit).collect()
}

/// Trims, applies NFC and removes zero-width characters, as for every name.
fn clean_name(input: &str) -> String {
    input
        .trim()
        .nfc()
        .filter(|&c| !text::is_invisible(c))
        .collect::<String>()
        .trim()
        .to_owned()
}

/// Shared checks of both name fields: required, length, control characters, script.
fn name(
    field: &str,
    input: &str,
    script_ok: fn(&str) -> bool,
    script_error: ErrorCode,
) -> Result<String, FieldError> {
    let cleaned = clean_name(input);
    if cleaned.is_empty() {
        return Err(error(ErrorCode::ErrRequired, field));
    }
    if cleaned.chars().count() > MAX_NAME_CHARS {
        return Err(error(ErrorCode::ErrNameTooLong, field));
    }
    if cleaned.chars().any(char::is_control) {
        return Err(error(ErrorCode::ErrControlChars, field));
    }
    if !script_ok(&cleaned) {
        return Err(error(script_error, field));
    }
    Ok(cleaned)
}

/// An Urdu name: 1 to 100 characters, Urdu letters, digits, spaces and `. - ' ( ) / &`,
/// at least one Urdu letter (PRD FR-7.28). Returns the cleaned name to store as `name_ur`.
pub fn name_ur(field: &str, input: &str) -> Result<String, FieldError> {
    name(field, input, text::is_urdu_name, ErrorCode::ErrNameUrScript)
}

/// An English name: 1 to 100 characters, A-Z, a-z, digits, spaces and `. - ' ( ) / &`,
/// at least one letter (PRD FR-7.28). Returns the cleaned name to store as `name_en`.
pub fn name_en(field: &str, input: &str) -> Result<String, FieldError> {
    name(
        field,
        input,
        text::is_english_name,
        ErrorCode::ErrNameEnScript,
    )
}

/// The tag (نشان) of an account: empty, or 1 to 4 characters, English capital letters A to Z and
/// `.`, with at least one letter (PRD 10.2, FR-7.28, Code Standards 4.2). A lower-case English
/// letter is made capital. Returns `None` for an empty tag, else the tag to store in
/// `accounts.tag`. It is never inside a name and never converted to Urdu (PRD FR-7.25).
///
/// # Errors
/// `ERR_TAG_INVALID` (value `max` 4) for a digit, a space, an Urdu letter, any other sign,
/// more than 4 characters, or dots only.
///
/// ```
/// use mandi_core::validate::tag;
/// assert_eq!(tag("tag", "k").unwrap(), Some("K".to_owned()));
/// assert_eq!(tag("tag", " D.P ").unwrap(), Some("D.P".to_owned()));
/// assert_eq!(tag("tag", "").unwrap(), None);
/// for bad in ["1A", "ABCDE", "ب", "K-", "..", "K A"] {
///     assert!(tag("tag", bad).is_err(), "{bad}");
/// }
/// ```
pub fn tag(field: &str, input: &str) -> Result<Option<String>, FieldError> {
    let typed = input.trim();
    if typed.is_empty() {
        return Ok(None);
    }
    // Only ASCII letters change case; anything else stays as it is and is refused below.
    let capital = typed.to_ascii_uppercase();
    let letters_and_dots = capital.chars().all(|c| c.is_ascii_uppercase() || c == '.');
    let has_letter = capital.chars().any(|c| c.is_ascii_uppercase());
    if letters_and_dots && has_letter && capital.chars().count() <= MAX_TAG_CHARS {
        Ok(Some(capital))
    } else {
        Err(error(ErrorCode::ErrTagInvalid, field))
    }
}

/// The name of a person as shown in the top bar: 1 to 100 characters in any script, with at
/// least one letter. Unlike `name_ur` and `name_en` it is not checked for one script, because
/// it is typed once and not searched (plan 04). Returns the cleaned name.
pub fn person_name(field: &str, input: &str) -> Result<String, FieldError> {
    name(
        field,
        input,
        |n| n.chars().any(char::is_alphabetic),
        ErrorCode::ErrRequired,
    )
}

/// A Pakistani mobile number: `03XXXXXXXXX` or `03XX-XXXXXXX`. Returned as 11 digits.
/// A number with an Arabic-Indic digit (٠-٩) and a hyphen is `ERR_PHONE_INVALID`: bidi would
/// show the two digit runs in a different order from the saved digits.
pub fn phone(field: &str, input: &str) -> Result<String, FieldError> {
    let typed = digit_text(input);
    if typed.is_empty() {
        return Err(error(ErrorCode::ErrRequired, field));
    }
    // Arabic-Indic digits are bidi class AN and the hyphen is class ES, so the hyphen takes the
    // right-to-left direction and the two digit runs show in the other order than the saved
    // digits: "٠٣٠٠-١٢٣٤٥٦٧" could show as "١٢٣٤٥٦٧-٠٣٠٠". Same rule as sums (plan 02
    // Section 4.6, lead decision). Without the hyphen, or with Urdu digits, the order holds.
    if input.contains('-') && input.chars().any(text::is_arabic_indic_digit) {
        return Err(error(ErrorCode::ErrPhoneInvalid, field));
    }
    let bytes = typed.as_bytes();
    let digits: String = match typed.len() {
        11 => typed.clone(),
        12 if bytes.get(4) == Some(&b'-') => typed.replacen('-', "", 1),
        _ => String::new(),
    };
    if digits.len() == 11 && digits.starts_with("03") && digits.bytes().all(|b| b.is_ascii_digit())
    {
        Ok(digits)
    } else {
        Err(error(ErrorCode::ErrPhoneInvalid, field))
    }
}

/// An optional phone: empty text, or text with only invisible marks, is `None`.
pub fn phone_optional(field: &str, input: &str) -> Result<Option<String>, FieldError> {
    if is_blank(input) {
        return Ok(None);
    }
    phone(field, input).map(Some)
}

/// Free text such as a note or a narration: trimmed, NFC, up to 500 characters,
/// no control characters. Empty text is allowed and returned as "".
///
/// The bidi embeddings, overrides and isolates (U+202A to U+202E, U+2066 to U+2069) are
/// refused as `ERR_CONTROL_CHARS`: they change the order text is shown and printed in, so
/// a note could show digits that differ from the digits saved (Trojan Source, PRD FR-19.6,
/// finding R02-3).
pub fn free_text(field: &str, input: &str) -> Result<String, FieldError> {
    let cleaned: String = input.trim().nfc().collect();
    if cleaned.chars().count() > MAX_TEXT_CHARS {
        return Err(error(ErrorCode::ErrTextTooLong, field));
    }
    if cleaned
        .chars()
        .any(|c| c.is_control() || text::is_bidi_control(c))
    {
        return Err(error(ErrorCode::ErrControlChars, field));
    }
    Ok(cleaned)
}

/// Collects the errors of several fields, so the user can fix all of them at once
/// (Code Standards 4.3).
///
/// ```
/// use mandi_core::validate::{self, FieldErrors};
/// let mut errors = FieldErrors::default();
/// let qty = errors.take(validate::qty("qty", "20"));
/// let rate = errors.take(validate::rate("rate", "abc"));
/// assert_eq!(qty.map(|q| q.get()), Some(20));
/// assert!(rate.is_none());
/// assert_eq!(errors.finish().unwrap_err().len(), 1);
/// ```
#[derive(Debug, Default)]
pub struct FieldErrors(Vec<FieldError>);

impl FieldErrors {
    /// Keeps the error, if any, and returns the value, if any.
    pub fn take<T>(&mut self, result: Result<T, FieldError>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(e) => {
                self.0.push(e);
                None
            }
        }
    }

    /// Adds one error found by a later check.
    pub fn push(&mut self, error: FieldError) {
        self.0.push(error);
    }

    /// True when no error was found.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// `Ok` when no error was found, else every error found.
    pub fn finish(self) -> Result<(), Vec<FieldError>> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(self.0)
        }
    }
}

#[cfg(test)]
mod tag_tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn the_tag_rules_of_code_standards_4_2() {
        for (typed, saved) in [
            ("K", "K"),
            ("BA", "BA"),
            ("D.P", "D.P"),
            ("k", "K"),
            ("d.p", "D.P"),
            ("  gm  ", "GM"),
            ("A.B.", "A.B."),
        ] {
            assert_eq!(
                tag("tag", typed).unwrap().as_deref(),
                Some(saved),
                "{typed}"
            );
        }
        for empty in ["", "   ", "\t"] {
            assert_eq!(tag("tag", empty).unwrap(), None, "{empty:?}");
        }
        for bad in [
            "1A",
            "ABCDE",
            "ب",
            "K-",
            ".",
            "...",
            "K A",
            "K1",
            "K_",
            "ÄB",
            "ＫＡ",
            "K\u{200b}",
        ] {
            let error = tag("tag", bad).unwrap_err();
            assert_eq!(error.code, ErrorCode::ErrTagInvalid, "{bad}");
            assert_eq!(error.field, "tag");
        }
    }

    #[test]
    fn four_characters_fit_and_five_do_not() {
        assert!(tag("tag", "A.B.").is_ok());
        assert!(tag("tag", "A.B.C").is_err());
    }

    proptest! {
        #[test]
        fn a_saved_tag_always_follows_the_rule(input in ".{0,12}") {
            if let Ok(Some(saved)) = tag("tag", &input) {
                prop_assert!((1..=MAX_TAG_CHARS).contains(&saved.chars().count()));
                prop_assert!(saved.chars().all(|c| c.is_ascii_uppercase() || c == '.'));
                prop_assert!(saved.chars().any(|c| c.is_ascii_uppercase()));
                // Saving a saved tag again changes nothing.
                prop_assert_eq!(tag("tag", &saved).unwrap(), Some(saved.clone()));
            }
        }
    }
}
