//! The clock is injected, so audit times are the same in every test (plan 03 Section 4.5).
//! Times are stored in UTC (Code Standards G9).

use std::sync::{Mutex, PoisonError};

use chrono::{DateTime, Duration, NaiveDate, SecondsFormat, Utc};

/// Pakistan time is UTC+5 all year (no summer time since 2009).
const PAKISTAN_OFFSET_HOURS: i64 = 5;

/// Gives the current time in UTC.
pub trait Clock: Send + Sync {
    /// The current time.
    fn now_utc(&self) -> DateTime<Utc>;

    /// Today's date on this computer, as the user sees it (business dates, PRD FR-6.6).
    /// The default is Pakistan time, UTC+5.
    fn local_today(&self) -> NaiveDate {
        (self.now_utc() + Duration::hours(PAKISTAN_OFFSET_HOURS)).date_naive()
    }
}

/// The real clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }

    /// The computer's own date, so a shop outside Pakistan time still sees its own day.
    fn local_today(&self) -> NaiveDate {
        chrono::Local::now().date_naive()
    }
}

/// A clock that always shows the same time. For tests.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub DateTime<Utc>);

impl Clock for FixedClock {
    fn now_utc(&self) -> DateTime<Utc> {
        self.0
    }
}

/// A clock a test can set and move, also backwards (NFR-REL-11).
#[derive(Debug)]
pub struct ManualClock(Mutex<DateTime<Utc>>);

impl ManualClock {
    /// A clock that shows `start` until it is moved.
    pub fn new(start: DateTime<Utc>) -> Self {
        Self(Mutex::new(start))
    }

    /// Sets the time, also to an earlier time.
    pub fn set(&self, time: DateTime<Utc>) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = time;
    }

    /// Moves the time on by `seconds` (a negative number moves it back).
    pub fn advance_seconds(&self, seconds: i64) {
        let mut time = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        *time += Duration::seconds(seconds);
    }
}

impl Clock for ManualClock {
    fn now_utc(&self) -> DateTime<Utc> {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The storage form of a time: `2026-10-10T07:30:12.345Z` (same as the column defaults).
pub fn format_utc(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn storage_form_has_milliseconds_and_z() {
        let time = Utc.with_ymd_and_hms(2026, 10, 10, 7, 30, 12).unwrap();
        assert_eq!(format_utc(time), "2026-10-10T07:30:12.000Z");
        assert_eq!(FixedClock(time).now_utc(), time);
    }
}
