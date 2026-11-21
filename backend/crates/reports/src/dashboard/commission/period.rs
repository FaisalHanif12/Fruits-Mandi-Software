//! Reading the period of the commission page (PRD FR-9.3, plan 77).
//!
//! A day, a month, a year or a range of days. Today's input, a month alone, stays a month. The
//! backend treats every date as untrusted text (Code Standards V0): a missing or bad value is an
//! error with its field, a date after today is refused, and a range that ends before it starts is
//! refused.
//!
//! Example (today 10-10-26): `{ kind: year, year: 2026 }` is 2026-01-01 to 2026-12-31;
//! `{ kind: range, from: "01-10-2026", to: "11-10-2026" }` is refused (`ERR_DATE_IN_FUTURE` on
//! `period.to`).

use mandi_core::dates::BusinessDate;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::months::Month;

use super::model::{CommissionPeriod, CommissionPeriodKind};
use crate::error::ReportsError;
use crate::reports::common::parse_day;

/// The first year the page accepts.
const FIRST_YEAR: i32 = 1;

/// A period that has been read and checked.
#[derive(Debug, Clone, Copy)]
pub(in crate::dashboard) struct Resolved {
    /// What it is.
    pub(in crate::dashboard) kind: CommissionPeriodKind,
    /// The first day.
    pub(in crate::dashboard) from: BusinessDate,
    /// The last day.
    pub(in crate::dashboard) to: BusinessDate,
}

impl Resolved {
    /// The length of the bucket text a row is grouped by: `YYYY-MM-DD` for a day, a month and a
    /// range, `YYYY-MM` for a year.
    pub(in crate::dashboard) const fn bucket_len(&self) -> usize {
        match self.kind {
            CommissionPeriodKind::Year => 7,
            _ => 10,
        }
    }
}

fn reject(code: ErrorCode, field: &str) -> ReportsError {
    ReportsError::Rejected(vec![FieldError::new(code, field)])
}

fn month_of(text: Option<&str>, current: Month, field: &str) -> Result<Resolved, ReportsError> {
    let month = match text.map(str::trim).filter(|m| !m.is_empty()) {
        None => current,
        Some(text) => {
            Month::parse(text).ok_or_else(|| reject(ErrorCode::ErrFormatInvalid, field))?
        }
    };
    if month > current {
        return Err(reject(ErrorCode::ErrDateInFuture, field));
    }
    let (Some(from), Some(to)) = (month.first_day(), month.last_day()) else {
        return Err(reject(ErrorCode::ErrFormatInvalid, field));
    };
    Ok(Resolved {
        kind: CommissionPeriodKind::Month,
        from,
        to,
    })
}

/// A date typed in the form: required, a real date, and not after today.
fn day_of(
    text: Option<&str>,
    today: BusinessDate,
    field: &str,
) -> Result<BusinessDate, ReportsError> {
    let text = text
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .ok_or_else(|| reject(ErrorCode::ErrRequired, field))?;
    let day = parse_day(text, today).map_err(|code| reject(code, field))?;
    if day > today {
        return Err(reject(ErrorCode::ErrDateInFuture, field));
    }
    Ok(day)
}

/// Reads the period of a request: `period` when sent, else `month` alone.
///
/// # Errors
/// `ERR_REQUIRED`, `ERR_FORMAT_INVALID`, `ERR_DATE_INVALID`, `ERR_DATE_IN_FUTURE` or
/// `ERR_DATE_RANGE` (a range that ends before it starts), each with its field.
pub(in crate::dashboard) fn resolve(
    today: BusinessDate,
    month: Option<&str>,
    period: Option<&CommissionPeriod>,
) -> Result<Resolved, ReportsError> {
    let current = Month::of(today);
    let Some(period) = period else {
        return month_of(month, current, "month");
    };
    match period.kind {
        CommissionPeriodKind::Month => month_of(period.month.as_deref(), current, "period.month"),
        CommissionPeriodKind::Day => {
            let day = day_of(period.date.as_deref(), today, "period.date")?;
            Ok(Resolved {
                kind: CommissionPeriodKind::Day,
                from: day,
                to: day,
            })
        }
        CommissionPeriodKind::Year => {
            let year = period
                .year
                .ok_or_else(|| reject(ErrorCode::ErrRequired, "period.year"))?;
            if year < FIRST_YEAR {
                return Err(reject(ErrorCode::ErrFormatInvalid, "period.year"));
            }
            if year > today.year() {
                return Err(reject(ErrorCode::ErrDateInFuture, "period.year"));
            }
            let (Some(from), Some(to)) = (
                BusinessDate::from_ymd(year, 1, 1),
                BusinessDate::from_ymd(year, 12, 31),
            ) else {
                return Err(reject(ErrorCode::ErrFormatInvalid, "period.year"));
            };
            Ok(Resolved {
                kind: CommissionPeriodKind::Year,
                from,
                to,
            })
        }
        CommissionPeriodKind::Range => {
            let from = day_of(period.from.as_deref(), today, "period.from");
            let to = day_of(period.to.as_deref(), today, "period.to");
            match (from, to) {
                (Ok(from), Ok(to)) => {
                    if from > to {
                        return Err(reject(ErrorCode::ErrDateRange, "period.to"));
                    }
                    Ok(Resolved {
                        kind: CommissionPeriodKind::Range,
                        from,
                        to,
                    })
                }
                // Both problems come back together (Code Standards 4.3).
                (from, to) => {
                    let mut problems = Vec::new();
                    for result in [from, to] {
                        if let Err(ReportsError::Rejected(found)) = result {
                            problems.extend(found);
                        }
                    }
                    Err(ReportsError::Rejected(problems))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> BusinessDate {
        BusinessDate::from_ymd(2026, 10, 10).unwrap()
    }

    fn codes(result: Result<Resolved, ReportsError>) -> Vec<(ErrorCode, String)> {
        match result {
            Err(ReportsError::Rejected(found)) => {
                found.into_iter().map(|e| (e.code, e.field)).collect()
            }
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    #[test]
    fn a_month_alone_is_a_month_and_none_is_this_month() {
        let r = resolve(today(), None, None).unwrap();
        assert_eq!(r.kind, CommissionPeriodKind::Month);
        assert_eq!(r.from.to_iso(), "2026-10-01");
        assert_eq!(r.to.to_iso(), "2026-10-31");
        let r = resolve(today(), Some("2026-02"), None).unwrap();
        assert_eq!(
            (r.from.to_iso(), r.to.to_iso()),
            ("2026-02-01".into(), "2026-02-28".into())
        );
        assert_eq!(
            codes(resolve(today(), Some("2026-11"), None)),
            vec![(ErrorCode::ErrDateInFuture, "month".to_owned())]
        );
        assert_eq!(
            codes(resolve(today(), Some("10-2026"), None)),
            vec![(ErrorCode::ErrFormatInvalid, "month".to_owned())]
        );
    }

    #[test]
    fn a_day_a_year_and_a_range_are_read_and_checked() {
        let day = CommissionPeriod {
            kind: CommissionPeriodKind::Day,
            date: Some("10-10-2026".into()),
            ..CommissionPeriod::default()
        };
        let r = resolve(today(), None, Some(&day)).unwrap();
        assert_eq!((r.from, r.to), (today(), today()));
        let future = CommissionPeriod {
            date: Some("2026-10-11".into()),
            ..day.clone()
        };
        assert_eq!(
            codes(resolve(today(), None, Some(&future))),
            vec![(ErrorCode::ErrDateInFuture, "period.date".to_owned())]
        );
        let missing = CommissionPeriod { date: None, ..day };
        assert_eq!(
            codes(resolve(today(), None, Some(&missing))),
            vec![(ErrorCode::ErrRequired, "period.date".to_owned())]
        );

        let year = CommissionPeriod {
            kind: CommissionPeriodKind::Year,
            year: Some(2026),
            ..CommissionPeriod::default()
        };
        let r = resolve(today(), None, Some(&year)).unwrap();
        assert_eq!(
            (r.from.to_iso(), r.to.to_iso()),
            ("2026-01-01".into(), "2026-12-31".into())
        );
        assert_eq!(r.bucket_len(), 7);
        let next = CommissionPeriod {
            year: Some(2027),
            ..year
        };
        assert_eq!(
            codes(resolve(today(), None, Some(&next))),
            vec![(ErrorCode::ErrDateInFuture, "period.year".to_owned())]
        );

        let range = |from: &str, to: &str| CommissionPeriod {
            kind: CommissionPeriodKind::Range,
            from: Some(from.into()),
            to: Some(to.into()),
            ..CommissionPeriod::default()
        };
        let r = resolve(today(), None, Some(&range("01-10-2026", "10-10-2026"))).unwrap();
        assert_eq!(
            (r.from.to_iso(), r.to.to_iso()),
            ("2026-10-01".into(), "2026-10-10".into())
        );
        assert_eq!(
            codes(resolve(
                today(),
                None,
                Some(&range("10-10-2026", "01-10-2026"))
            )),
            vec![(ErrorCode::ErrDateRange, "period.to".to_owned())]
        );
        // Both bad dates come back together.
        assert_eq!(
            codes(resolve(
                today(),
                None,
                Some(&range("31-02-2026", "2026-12-01"))
            ))
            .len(),
            2
        );
    }
}
