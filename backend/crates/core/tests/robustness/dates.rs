//! Dates at the edges (PRD FR-6.6, FR-7.19, Code Standards 4.2). Rules: leap years follow
//! the calendar (2000 yes, 2100 no); impossible dates and shortcuts that land on one are
//! `ERR_DATE_INVALID`; the open-day rule holds on its boundaries; nothing panics.

use mandi_core::dates::{BusinessDate, check_posting_date, format_ddmmyyyy, parse_ddmmyyyy};
use mandi_core::error::{ErrorCode, ErrorValue};
use mandi_core::validate;

use super::{hostile_strings, no_panic};

fn d(y: i32, m: u32, day: u32) -> BusinessDate {
    BusinessDate::from_ymd(y, m, day).unwrap()
}

const INVALID: Result<BusinessDate, ErrorCode> = Err(ErrorCode::ErrDateInvalid);

#[test]
fn leap_days() {
    let today = d(2026, 10, 10);
    for (text, expected) in [
        ("29-02-2024", Ok(d(2024, 2, 29))),
        ("29-02-2028", Ok(d(2028, 2, 29))),
        ("29-02-2000", Ok(d(2000, 2, 29))),
        ("29-02-2025", INVALID),
        ("29-02-2026", INVALID),
        ("29-02-2100", INVALID),
        ("29-02-1900", INVALID),
        ("28-02-2026", Ok(d(2026, 2, 28))),
        ("30-02-2024", INVALID),
    ] {
        assert_eq!(parse_ddmmyyyy(text, today), expected, "{text}");
    }
}

#[test]
fn impossible_dates() {
    let today = d(2026, 10, 10);
    for text in [
        "31-04-2026",
        "31-06-2026",
        "31-09-2026",
        "31-11-2026",
        "32-01-2026",
        "00-00-0000",
        "00-01-2026",
        "01-00-2026",
        "01-13-2026",
        "0-0-0",
        "01-01-0000",
        "01-01-0999",
        "01-01-10000",
        "-1",
        "+1-10-2026",
        "1-+1-2026",
        "10 10 2026",
        "10-10-2026-",
        "10/10-",
        "99999999999999999999",
        "١٠-١٠-٢٠٢٦-١",
        "10-10-٢٠٢٦x",
        "2026-10-10",
        "1e1",
        "",
    ] {
        let got = no_panic(text, || parse_ddmmyyyy(text, today));
        let expected = if text.is_empty() {
            Err(ErrorCode::ErrRequired)
        } else {
            INVALID
        };
        assert_eq!(got, expected, "{text:?}");
    }
    assert_eq!(parse_ddmmyyyy("31-12-9999", today), Ok(d(9999, 12, 31)));
    assert_eq!(parse_ddmmyyyy("01-01-1000", today), Ok(d(1000, 1, 1)));
}

#[test]
fn shortcuts_at_month_ends() {
    // FR-7.19: "31" is day 31 of today's month; in a 30-day month or February it is invalid.
    assert_eq!(parse_ddmmyyyy("31", d(2026, 10, 1)), Ok(d(2026, 10, 31)));
    assert_eq!(parse_ddmmyyyy("31", d(2026, 4, 30)), INVALID);
    assert_eq!(parse_ddmmyyyy("30", d(2026, 2, 28)), INVALID);
    assert_eq!(parse_ddmmyyyy("29", d(2026, 2, 28)), INVALID);
    assert_eq!(parse_ddmmyyyy("29", d(2028, 2, 1)), Ok(d(2028, 2, 29)));
    // "29-2" takes today's year.
    assert_eq!(parse_ddmmyyyy("29-2", d(2026, 10, 10)), INVALID);
    assert_eq!(parse_ddmmyyyy("29-2", d(2028, 10, 10)), Ok(d(2028, 2, 29)));
    assert_eq!(parse_ddmmyyyy("31-4", d(2026, 10, 10)), INVALID);
    assert_eq!(parse_ddmmyyyy("31-12", d(2026, 1, 1)), Ok(d(2026, 12, 31)));
    assert_eq!(parse_ddmmyyyy("0", d(2026, 1, 1)), INVALID);
    assert_eq!(parse_ddmmyyyy("1-1", d(2026, 12, 31)), Ok(d(2026, 1, 1)));
    assert_eq!(parse_ddmmyyyy("۳۱", d(2026, 12, 1)), Ok(d(2026, 12, 31)));
}

#[test]
fn open_day_rule_boundaries() {
    let today = d(2026, 10, 10);
    let closed = Some(d(2026, 10, 9));
    assert_eq!(check_posting_date(today, closed, today), Ok(()));
    assert_eq!(
        check_posting_date(d(2026, 10, 9), closed, today),
        Err(ErrorCode::ErrDateClosed)
    );
    assert_eq!(
        check_posting_date(d(2026, 10, 11), closed, today),
        Err(ErrorCode::ErrDateInFuture)
    );
    // Closed up to today: nothing can be posted.
    assert_eq!(
        check_posting_date(today, Some(today), today),
        Err(ErrorCode::ErrDateClosed)
    );
    // Year boundaries.
    let ny = d(2027, 1, 1);
    assert_eq!(check_posting_date(ny, Some(d(2026, 12, 31)), ny), Ok(()));
    assert_eq!(
        check_posting_date(d(2026, 12, 31), Some(d(2026, 12, 31)), ny),
        Err(ErrorCode::ErrDateClosed)
    );
    // The error carries the limiting date for the message.
    let e = validate::posting_date("entry_date", "09-10-2026", closed, today).unwrap_err();
    assert_eq!(
        e.values.get("closed_up_to"),
        Some(&ErrorValue::Text("09-10-2026".to_owned()))
    );
    let e = validate::posting_date("entry_date", "11", closed, today).unwrap_err();
    assert_eq!(e.code, ErrorCode::ErrDateInFuture);
    assert_eq!(
        e.values.get("today"),
        Some(&ErrorValue::Text("10-10-2026".to_owned()))
    );
}

#[test]
fn iso_storage_form_rejects_hostile_text() {
    for text in [
        "",
        "2026-02-30",
        "0999-01-01",
        "10000-01-01",
        "2026-10-10T00:00",
        "2026/10/10",
        "' OR 1=1 --",
    ] {
        assert_eq!(
            no_panic(text, || BusinessDate::parse_iso(text)),
            None,
            "{text:?}"
        );
    }
    let date = d(2026, 1, 9);
    assert_eq!(BusinessDate::parse_iso(&date.to_iso()), Some(date));
    assert_eq!(format_ddmmyyyy(date), "09-01-2026");
}

#[test]
fn every_date_parser_survives_every_hostile_string() {
    let today = d(2026, 10, 10);
    for s in hostile_strings() {
        let short: String = s.chars().take(20).collect();
        no_panic(&short, || {
            let _ = parse_ddmmyyyy(&s, today);
            let _ = BusinessDate::parse_iso(&s);
            let _ = validate::posting_date("d", &s, None, today);
        });
    }
}
