//! Whole calendar months for the pages that show one month at a time (plan 76 points 7 and 11,
//! the month window of PRD 19.11). A month is written `YYYY-MM`.
//!
//! Example: `2026-10` runs from 2026-10-01 to 2026-10-31; the month before is `2026-09`.

use crate::dates::BusinessDate;

/// A calendar month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Month {
    year: i32,
    month: u32,
}

impl Month {
    /// The month of a date.
    pub fn of(date: BusinessDate) -> Month {
        Month {
            year: date.year(),
            month: date.month(),
        }
    }

    /// Reads `YYYY-MM`. `None` for anything else, a month outside 1 to 12 or a year outside the
    /// calendar.
    pub fn parse(text: &str) -> Option<Month> {
        let (year, month) = text.trim().split_once('-')?;
        if year.len() != 4 || month.len() != 2 {
            return None;
        }
        let (year, month) = (year.parse::<i32>().ok()?, month.parse::<u32>().ok()?);
        BusinessDate::from_ymd(year, month, 1)?;
        Some(Month { year, month })
    }

    /// `YYYY-MM`.
    pub fn key(self) -> String {
        format!("{:04}-{:02}", self.year, self.month)
    }

    /// The first day.
    pub fn first_day(self) -> Option<BusinessDate> {
        BusinessDate::from_ymd(self.year, self.month, 1)
    }

    /// The last day.
    pub fn last_day(self) -> Option<BusinessDate> {
        self.next()
            .first_day()
            .and_then(BusinessDate::previous_day)
            .or_else(|| BusinessDate::from_ymd(self.year, self.month, 31))
    }

    /// The month after.
    pub fn next(self) -> Month {
        if self.month == 12 {
            Month {
                year: self.year.saturating_add(1),
                month: 1,
            }
        } else {
            Month {
                year: self.year,
                month: self.month.saturating_add(1),
            }
        }
    }

    /// The month before.
    pub fn previous(self) -> Month {
        if self.month == 1 {
            Month {
                year: self.year.saturating_sub(1),
                month: 12,
            }
        } else {
            Month {
                year: self.year,
                month: self.month.saturating_sub(1),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_month_has_its_first_and_last_day() {
        let october = Month::parse("2026-10").unwrap();
        assert_eq!(october.first_day().unwrap().to_iso(), "2026-10-01");
        assert_eq!(october.last_day().unwrap().to_iso(), "2026-10-31");
        let february = Month::parse("2028-02").unwrap();
        assert_eq!(february.last_day().unwrap().to_iso(), "2028-02-29");
        assert_eq!(Month::parse("2026-12").unwrap().next().key(), "2027-01");
        assert_eq!(Month::parse("2026-01").unwrap().previous().key(), "2025-12");
    }

    #[test]
    fn a_bad_month_is_refused() {
        for bad in ["2026-13", "2026-0", "26-10", "", "2026/10", "2026-1a"] {
            assert!(Month::parse(bad).is_none(), "{bad}");
        }
    }
}
