//! Input checks for sign-in, users and settings (Code Standards 4.2, NFR-SEC-1, plan 04).
//!
//! Each function takes the field name, so the error points at the right field on the screen.

use mandi_core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_core::validate::{self, FieldErrors};

/// Shortest password, in characters (NFR-SEC-1).
pub const MIN_PASSWORD_CHARS: usize = 6;
/// Longest password, in characters. A limit keeps one request from asking for huge work.
pub const MAX_PASSWORD_CHARS: usize = 128;
/// Shortest sign-in name.
pub const MIN_USERNAME_CHARS: usize = 3;
/// Longest sign-in name.
pub const MAX_USERNAME_CHARS: usize = 32;
/// Fewest minutes of idle lock (PRD FR-8.4).
pub const MIN_IDLE_LOCK_MINUTES: u32 = 1;
/// Most minutes of idle lock.
pub const MAX_IDLE_LOCK_MINUTES: u32 = 120;
/// Idle lock minutes when nothing was set (PRD FR-8.4).
pub const DEFAULT_IDLE_LOCK_MINUTES: u32 = 15;
/// Smallest text size, in percent (NFR-USE-18).
pub const MIN_TEXT_SIZE: u32 = 80;
/// Largest text size, in percent.
pub const MAX_TEXT_SIZE: u32 = 150;
/// Text size steps, in percent.
pub const TEXT_SIZE_STEP: u32 = 10;
/// Text size when nothing was set.
pub const DEFAULT_TEXT_SIZE: u32 = 100;
/// Longest screen route kept as the last screen.
pub const MAX_ROUTE_CHARS: usize = 200;

fn error(code: ErrorCode, field: &str) -> FieldError {
    FieldError::new(code, field)
}

/// A sign-in name: trimmed, lower case, 3 to 32 of `a-z 0-9 . _ -`. Empty is `ERR_REQUIRED`.
/// Returns the stored form.
///
/// ```
/// use mandi_domain::auth::validate::username;
/// assert_eq!(username("username", "  Munshi.Ali ").unwrap(), "munshi.ali");
/// assert!(username("username", "ab").is_err());
/// assert!(username("username", "ali khan").is_err());
/// ```
pub fn username(field: &str, input: &str) -> Result<String, FieldError> {
    let cleaned = input.trim().to_lowercase();
    if cleaned.is_empty() {
        return Err(error(ErrorCode::ErrRequired, field));
    }
    let length = cleaned.chars().count();
    let chars_ok = cleaned
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'));
    if !(MIN_USERNAME_CHARS..=MAX_USERNAME_CHARS).contains(&length) || !chars_ok {
        return Err(error(ErrorCode::ErrUsernameInvalid, field));
    }
    Ok(cleaned)
}

/// A password: not empty, 6 to 128 characters. Spaces count. It is not trimmed or changed.
pub fn password(field: &str, input: &str) -> Result<(), FieldError> {
    if input.is_empty() {
        return Err(error(ErrorCode::ErrRequired, field));
    }
    let length = input.chars().count();
    if length < MIN_PASSWORD_CHARS {
        return Err(error(ErrorCode::ErrPasswordTooShort, field).with(
            "min",
            ErrorValue::Number(i64::try_from(MIN_PASSWORD_CHARS).unwrap_or(i64::MAX)),
        ));
    }
    if length > MAX_PASSWORD_CHARS {
        return Err(error(ErrorCode::ErrTextTooLong, field).with(
            "max",
            ErrorValue::Number(i64::try_from(MAX_PASSWORD_CHARS).unwrap_or(i64::MAX)),
        ));
    }
    Ok(())
}

/// A new password and its repeat. Adds every problem to `errors`: the password rules on
/// `field`, and `ERR_PASSWORDS_DIFFER` on `repeat_field`.
pub fn password_pair(
    errors: &mut FieldErrors,
    field: &str,
    repeat_field: &str,
    password_text: &str,
    repeat_text: &str,
) {
    let valid = errors.take(password(field, password_text)).is_some();
    // The repeat is only compared when the first password is usable, so the user fixes one
    // thing at a time.
    if valid && password_text != repeat_text {
        errors.push(error(ErrorCode::ErrPasswordsDiffer, repeat_field));
    }
}

/// The name shown in the top bar (see `core::validate::person_name`).
pub fn display_name(field: &str, input: &str) -> Result<String, FieldError> {
    validate::person_name(field, input)
}

/// Whole minutes of idle lock as typed, 1 to 120. Not a number is `ERR_NOT_A_NUMBER`; outside
/// the range is `ERR_OUT_OF_RANGE` with `min` and `max`.
pub fn idle_lock_minutes(field: &str, input: &str) -> Result<u32, FieldError> {
    let value = whole_number(field, input)?;
    in_range(field, value, MIN_IDLE_LOCK_MINUTES, MAX_IDLE_LOCK_MINUTES)
}

/// A text size in percent as typed: 80 to 150 in steps of 10.
pub fn text_size(field: &str, input: &str) -> Result<u32, FieldError> {
    let value = whole_number(field, input)?;
    let value = in_range(field, value, MIN_TEXT_SIZE, MAX_TEXT_SIZE)?;
    if value.checked_rem(TEXT_SIZE_STEP) != Some(0) {
        return Err(range_error(field, MIN_TEXT_SIZE, MAX_TEXT_SIZE));
    }
    Ok(value)
}

/// The route of the screen to open next time, for example `/bikri`: starts with `/`, at most
/// 200 characters, no control characters.
pub fn last_screen(field: &str, input: &str) -> Result<String, FieldError> {
    let cleaned = input.trim();
    if cleaned.chars().count() > MAX_ROUTE_CHARS {
        return Err(error(ErrorCode::ErrTextTooLong, field).with(
            "max",
            ErrorValue::Number(i64::try_from(MAX_ROUTE_CHARS).unwrap_or(i64::MAX)),
        ));
    }
    if !cleaned.starts_with('/') || cleaned.chars().any(char::is_control) {
        return Err(error(ErrorCode::ErrOutOfRange, field));
    }
    Ok(cleaned.to_owned())
}

fn whole_number(field: &str, input: &str) -> Result<u32, FieldError> {
    let text = input.trim();
    if text.is_empty() {
        return Err(error(ErrorCode::ErrRequired, field));
    }
    if !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(error(ErrorCode::ErrNotANumber, field));
    }
    // Digits only, so a failure here means the number is too big for any range.
    text.parse::<u32>()
        .map_err(|_| error(ErrorCode::ErrOutOfRange, field))
}

fn range_error(field: &str, min: u32, max: u32) -> FieldError {
    error(ErrorCode::ErrOutOfRange, field)
        .with("min", ErrorValue::Number(i64::from(min)))
        .with("max", ErrorValue::Number(i64::from(max)))
}

fn in_range(field: &str, value: u32, min: u32, max: u32) -> Result<u32, FieldError> {
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(range_error(field, min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(result: Result<impl Sized, FieldError>) -> Option<ErrorCode> {
        result.err().map(|e| e.code)
    }

    #[test]
    fn usernames() {
        assert_eq!(username("u", "Owner").unwrap(), "owner");
        assert_eq!(username("u", "a.b_c-9").unwrap(), "a.b_c-9");
        assert_eq!(username("u", "abc").unwrap(), "abc");
        assert_eq!(username("u", &"a".repeat(32)).unwrap().len(), 32);
        assert_eq!(code(username("u", "")), Some(ErrorCode::ErrRequired));
        assert_eq!(code(username("u", "   ")), Some(ErrorCode::ErrRequired));
        for bad in [
            "ab",
            &"a".repeat(33),
            "ali khan",
            "ali@x",
            "علی",
            "al\tex",
            "ali/x",
            "ali+1",
        ] {
            assert_eq!(
                code(username("u", bad)),
                Some(ErrorCode::ErrUsernameInvalid),
                "{bad}"
            );
        }
    }

    #[test]
    fn passwords() {
        assert!(password("p", "123456").is_ok());
        assert!(password("p", "pass word with spaces").is_ok());
        assert!(password("p", "پاسورڈ12").is_ok());
        assert!(password("p", &"x".repeat(128)).is_ok());
        assert_eq!(code(password("p", "")), Some(ErrorCode::ErrRequired));
        let short = password("p", "12345").unwrap_err();
        assert_eq!(short.code, ErrorCode::ErrPasswordTooShort);
        assert_eq!(short.values.get("min"), Some(&ErrorValue::Number(6)));
        assert_eq!(
            code(password("p", &"x".repeat(129))),
            Some(ErrorCode::ErrTextTooLong)
        );
        // Six spaces are six characters: allowed, the Owner chose it.
        assert!(password("p", "      ").is_ok());
    }

    #[test]
    fn password_pairs() {
        let mut errors = FieldErrors::default();
        password_pair(
            &mut errors,
            "password",
            "password_repeat",
            "secret1",
            "secret1",
        );
        assert!(errors.is_empty());
        let mut errors = FieldErrors::default();
        password_pair(
            &mut errors,
            "password",
            "password_repeat",
            "secret1",
            "secret2",
        );
        let list = errors.finish().unwrap_err();
        assert_eq!(list.len(), 1);
        assert_eq!(
            (list[0].code, list[0].field.as_str()),
            (ErrorCode::ErrPasswordsDiffer, "password_repeat")
        );
        // A too short password is the only error shown.
        let mut errors = FieldErrors::default();
        password_pair(&mut errors, "password", "password_repeat", "abc", "xyz");
        let list = errors.finish().unwrap_err();
        assert_eq!(list.len(), 1);
        assert_eq!(
            (list[0].code, list[0].field.as_str()),
            (ErrorCode::ErrPasswordTooShort, "password")
        );
    }

    #[test]
    fn idle_lock_and_text_size() {
        assert_eq!(idle_lock_minutes("m", "15").unwrap(), 15);
        assert_eq!(idle_lock_minutes("m", " 1 ").unwrap(), 1);
        assert_eq!(idle_lock_minutes("m", "120").unwrap(), 120);
        for bad in ["0", "121", "99999999999"] {
            assert_eq!(
                code(idle_lock_minutes("m", bad)),
                Some(ErrorCode::ErrOutOfRange),
                "{bad}"
            );
        }
        for bad in ["abc", "1.5", "-5", "1e2", "+5", "١٥"] {
            assert_eq!(
                code(idle_lock_minutes("m", bad)),
                Some(ErrorCode::ErrNotANumber),
                "{bad}"
            );
        }
        assert_eq!(
            code(idle_lock_minutes("m", "")),
            Some(ErrorCode::ErrRequired)
        );
        let range = idle_lock_minutes("m", "0").unwrap_err();
        assert_eq!(range.values.get("min"), Some(&ErrorValue::Number(1)));
        assert_eq!(range.values.get("max"), Some(&ErrorValue::Number(120)));

        for good in ["80", "90", "100", "110", "120", "130", "140", "150"] {
            assert!(text_size("t", good).is_ok(), "{good}");
        }
        for bad in ["70", "85", "155", "160", "0"] {
            assert_eq!(
                code(text_size("t", bad)),
                Some(ErrorCode::ErrOutOfRange),
                "{bad}"
            );
        }
    }

    #[test]
    fn last_screen_routes() {
        assert_eq!(last_screen("v", " /bikri ").unwrap(), "/bikri");
        assert_eq!(last_screen("v", "/").unwrap(), "/");
        assert_eq!(
            code(last_screen("v", "bikri")),
            Some(ErrorCode::ErrOutOfRange)
        );
        assert_eq!(
            code(last_screen("v", "/a\nb")),
            Some(ErrorCode::ErrOutOfRange)
        );
        assert_eq!(
            code(last_screen("v", &format!("/{}", "x".repeat(200)))),
            Some(ErrorCode::ErrTextTooLong)
        );
    }

    #[test]
    fn display_names() {
        assert_eq!(display_name("n", " Maher Adnan ").unwrap(), "Maher Adnan");
        assert_eq!(display_name("n", "ماہر عدنان").unwrap(), "ماہر عدنان");
        assert_eq!(display_name("n", "Ali علی").unwrap(), "Ali علی");
        assert_eq!(code(display_name("n", "")), Some(ErrorCode::ErrRequired));
        assert_eq!(
            code(display_name("n", "1234")),
            Some(ErrorCode::ErrRequired)
        );
        assert_eq!(
            code(display_name("n", &"x".repeat(101))),
            Some(ErrorCode::ErrNameTooLong)
        );
        assert_eq!(
            code(display_name("n", "a\u{0007}b")),
            Some(ErrorCode::ErrControlChars)
        );
    }
}
