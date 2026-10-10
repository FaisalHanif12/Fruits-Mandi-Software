//! Business dates and the open-day rule (PRD FR-6.6, FR-7.19, Code Standards G9 and 4.2).
//!
//! A business date is a plain date with no time and no time zone. It is stored as
//! `YYYY-MM-DD` and shown as `DD-MM-YYYY`. Core never reads the clock: "today" is always
//! a parameter (Code Standards B1).

use chrono::{Datelike, NaiveDate};

use crate::error::ErrorCode;

/// A plain business date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BusinessDate(NaiveDate);

impl BusinessDate {
    /// A date from year, month and day. `None` if it does not exist (31-02-2026).
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Option<BusinessDate> {
        NaiveDate::from_ymd_opt(year, month, day).map(BusinessDate)
    }

    /// Reads the storage form `YYYY-MM-DD`.
    pub fn parse_iso(text: &str) -> Option<BusinessDate> {
        NaiveDate::parse_from_str(text, "%Y-%m-%d")
            .ok()
            .filter(|d| (1000..=9999).contains(&d.year()))
            .map(BusinessDate)
    }

    /// The storage form `YYYY-MM-DD`.
    pub fn to_iso(self) -> String {
        self.0.format("%Y-%m-%d").to_string()
    }

    /// Year, for example 2026.
    pub fn year(self) -> i32 {
        self.0.year()
    }

    /// Month, 1 to 12.
    pub fn month(self) -> u32 {
        self.0.month()
    }

    /// Day of the month, 1 to 31.
    pub fn day(self) -> u32 {
        self.0.day()
    }
}

/// Reads a date typed by the user (PRD FR-7.19). Accepts:
/// - `DD-MM-YYYY`, for example `10-10-2026` (day and month may have one digit);
/// - `10`: day 10 of today's month and year;
/// - `10-9`: 10 September of today's year.
///
/// `/` and `.` also work as separators. Urdu digits are taken as 0-9.
/// Empty text is `ERR_REQUIRED`; anything else that is not a real date is `ERR_DATE_INVALID`.
pub fn parse_ddmmyyyy(text: &str, today: BusinessDate) -> Result<BusinessDate, ErrorCode> {
    let cleaned: String = text
        .trim()
        .chars()
        .map(|c| match c {
            '\u{06F0}'..='\u{06F9}' | '\u{0660}'..='\u{0669}' => {
                let zero = if c >= '\u{06F0}' {
                    '\u{06F0}'
                } else {
                    '\u{0660}'
                };
                char::from_digit(u32::from(c).saturating_sub(u32::from(zero)), 10).unwrap_or(c)
            }
            '/' | '.' => '-',
            other => other,
        })
        .collect();
    if cleaned.is_empty() {
        return Err(ErrorCode::ErrRequired);
    }
    let parts: Vec<&str> = cleaned.split('-').collect();
    let number = |s: &str, max_len: usize| -> Option<u32> {
        let ok = (1..=max_len).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit());
        if ok { s.parse().ok() } else { None }
    };
    let (day, month, year) = match parts.as_slice() {
        [d] => (number(d, 2), Some(today.month()), Some(today.year())),
        [d, m] => (number(d, 2), number(m, 2), Some(today.year())),
        [d, m, y] if y.len() == 4 => (
            number(d, 2),
            number(m, 2),
            number(y, 4).and_then(|y| i32::try_from(y).ok()),
        ),
        _ => (None, None, None),
    };
    match (day, month, year) {
        (Some(d), Some(m), Some(y)) if y >= 1000 => {
            BusinessDate::from_ymd(y, m, d).ok_or(ErrorCode::ErrDateInvalid)
        }
        _ => Err(ErrorCode::ErrDateInvalid),
    }
}

/// The display form `DD-MM-YYYY`, the same in both languages (NFR-URD-16).
pub fn format_ddmmyyyy(date: BusinessDate) -> String {
    date.0.format("%d-%m-%Y").to_string()
}

/// The open-day rule (PRD FR-6.6): a posting date must be after the "closed up to" date
/// and not after today. `closed_up_to` is `None` when no day has ever been closed.
///
/// Example: closed up to 09-10-2026, today 10-10-2026: 10-10-2026 is fine, 09-10-2026 is
/// `ERR_DATE_CLOSED`, 11-10-2026 is `ERR_DATE_IN_FUTURE`.
pub fn check_posting_date(
    date: BusinessDate,
    closed_up_to: Option<BusinessDate>,
    today: BusinessDate,
) -> Result<(), ErrorCode> {
    if closed_up_to.is_some_and(|closed| date <= closed) {
        return Err(ErrorCode::ErrDateClosed);
    }
    if date > today {
        return Err(ErrorCode::ErrDateInFuture);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> BusinessDate {
        BusinessDate::from_ymd(y, m, day).unwrap()
    }

    const TODAY: (i32, u32, u32) = (2026, 10, 10);

    fn today() -> BusinessDate {
        d(TODAY.0, TODAY.1, TODAY.2)
    }

    #[test]
    fn full_dates() {
        assert_eq!(parse_ddmmyyyy("10-10-2026", today()), Ok(d(2026, 10, 10)));
        assert_eq!(parse_ddmmyyyy(" 1-9-2026 ", today()), Ok(d(2026, 9, 1)));
        assert_eq!(parse_ddmmyyyy("01/09/2026", today()), Ok(d(2026, 9, 1)));
        assert_eq!(parse_ddmmyyyy("01.09.2026", today()), Ok(d(2026, 9, 1)));
        assert_eq!(parse_ddmmyyyy("۱۰-۱۰-۲۰۲۶", today()), Ok(d(2026, 10, 10)));
        assert_eq!(parse_ddmmyyyy("١٠-١٠-٢٠٢٦", today()), Ok(d(2026, 10, 10)));
        assert_eq!(parse_ddmmyyyy("29-02-2028", today()), Ok(d(2028, 2, 29)));
    }

    #[test]
    fn shortcuts_of_fr_7_19() {
        // "10" = day 10 of today's month and year.
        assert_eq!(parse_ddmmyyyy("10", today()), Ok(d(2026, 10, 10)));
        assert_eq!(parse_ddmmyyyy("3", today()), Ok(d(2026, 10, 3)));
        // "10-9" = 10 September of today's year.
        assert_eq!(parse_ddmmyyyy("10-9", today()), Ok(d(2026, 9, 10)));
    }

    #[test]
    fn invalid_dates() {
        for text in [
            "31-02-2026",
            "29-02-2026",
            "32",
            "0",
            "10-13",
            "10-10-26",
            "10-10-0999",
            "abc",
            "10--2026",
            "1-1-1-2026",
            "100-1-2026",
            "1-100-2026",
            "-",
            "10-10-20266",
        ] {
            assert_eq!(
                parse_ddmmyyyy(text, today()),
                Err(ErrorCode::ErrDateInvalid),
                "{text}"
            );
        }
        assert_eq!(parse_ddmmyyyy("  ", today()), Err(ErrorCode::ErrRequired));
    }

    #[test]
    fn format_and_iso() {
        let date = d(2026, 9, 1);
        assert_eq!(format_ddmmyyyy(date), "01-09-2026");
        assert_eq!(date.to_iso(), "2026-09-01");
        assert_eq!(BusinessDate::parse_iso("2026-09-01"), Some(date));
        assert_eq!(BusinessDate::parse_iso("2026-02-30"), None);
        assert_eq!(BusinessDate::parse_iso("01-09-2026"), None);
        assert_eq!((date.year(), date.month(), date.day()), (2026, 9, 1));
    }

    #[test]
    fn open_day_rule_fr_6_6() {
        let closed = Some(d(2026, 10, 9));
        assert_eq!(check_posting_date(d(2026, 10, 10), closed, today()), Ok(()));
        assert_eq!(
            check_posting_date(d(2026, 10, 9), closed, today()),
            Err(ErrorCode::ErrDateClosed)
        );
        assert_eq!(
            check_posting_date(d(2026, 1, 1), closed, today()),
            Err(ErrorCode::ErrDateClosed)
        );
        assert_eq!(
            check_posting_date(d(2026, 10, 11), closed, today()),
            Err(ErrorCode::ErrDateInFuture)
        );
        assert_eq!(check_posting_date(d(2000, 1, 1), None, today()), Ok(()));
    }
}
