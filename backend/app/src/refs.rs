//! Reference numbers of errors: `E-<yyyymmdd>-<4-digit counter per day>` (plan 03 Section 4.6,
//! NFR-OBS-3). The user reads the number from the screen, and support finds the same number
//! in the log line.
//!
//! The counter restarts each day. It is also kept in a small file in the log folder, so a
//! restart on the same day goes on counting and two errors never share a number.

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use chrono::NaiveDate;

/// Name of the counter file inside the log folder.
pub const COUNTER_FILE: &str = "error-ref-counter";

/// The counter of one day. The day is Pakistan's local day, as the user sees it.
#[derive(Debug, Default)]
pub struct ErrorRefs {
    day: Option<NaiveDate>,
    counter: u32,
    file: Option<PathBuf>,
}

impl ErrorRefs {
    /// A counter that lives only in memory.
    pub const fn new() -> Self {
        Self {
            day: None,
            counter: 0,
            file: None,
        }
    }

    /// A counter that keeps its state in `file`, and goes on from it if it is from `today`.
    pub fn with_file(file: PathBuf, today: NaiveDate) -> Self {
        let mut refs = Self {
            day: None,
            counter: 0,
            file: Some(file),
        };
        if let Some((day, counter)) = refs.load()
            && day == today
        {
            refs.day = Some(day);
            refs.counter = counter;
        }
        refs
    }

    /// The next number for `today`, for example `E-20261010-0007`.
    pub fn next(&mut self, today: NaiveDate) -> String {
        if self.day != Some(today) {
            self.day = Some(today);
            self.counter = 0;
        }
        self.counter = self.counter.saturating_add(1);
        self.save(today);
        format!("E-{}-{:04}", today.format("%Y%m%d"), self.counter)
    }

    fn load(&self) -> Option<(NaiveDate, u32)> {
        let text = std::fs::read_to_string(self.file.as_ref()?).ok()?;
        let (day, counter) = text.trim().split_once(' ')?;
        Some((
            NaiveDate::parse_from_str(day, "%Y%m%d").ok()?,
            counter.parse().ok()?,
        ))
    }

    /// Best effort: a number is still given if the file cannot be written.
    fn save(&self, today: NaiveDate) {
        if let Some(file) = &self.file {
            let text = format!("{} {}", today.format("%Y%m%d"), self.counter);
            if let Err(error) = std::fs::write(file, text) {
                tracing::warn!("cannot save the error reference counter: {error}");
            }
        }
    }
}

/// The one counter of the running app.
static REFS: Mutex<ErrorRefs> = Mutex::new(ErrorRefs::new());

/// Today's date as the user sees it.
fn local_today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

/// Starts keeping the counter in `file`. Called once at start-up, after the log folder exists.
pub fn init(file: PathBuf) {
    let refs = ErrorRefs::with_file(file, local_today());
    *REFS.lock().unwrap_or_else(PoisonError::into_inner) = refs;
}

/// The next reference number of the running app.
pub fn next_reference() -> String {
    REFS.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .next(local_today())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn counts_up_and_restarts_each_day() {
        let mut refs = ErrorRefs::new();
        assert_eq!(refs.next(day(2026, 10, 10)), "E-20261010-0001");
        assert_eq!(refs.next(day(2026, 10, 10)), "E-20261010-0002");
        assert_eq!(refs.next(day(2026, 10, 11)), "E-20261011-0001");
        assert_eq!(refs.next(day(2026, 10, 11)), "E-20261011-0002");
    }

    #[test]
    fn goes_on_after_a_restart_on_the_same_day_only() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(COUNTER_FILE);
        let mut first = ErrorRefs::with_file(file.clone(), day(2026, 10, 10));
        first.next(day(2026, 10, 10));
        first.next(day(2026, 10, 10));
        let mut same_day = ErrorRefs::with_file(file.clone(), day(2026, 10, 10));
        assert_eq!(same_day.next(day(2026, 10, 10)), "E-20261010-0003");
        let mut next_day = ErrorRefs::with_file(file.clone(), day(2026, 10, 11));
        assert_eq!(next_day.next(day(2026, 10, 11)), "E-20261011-0001");
        // A broken file starts the day again at 1.
        std::fs::write(&file, "garbage").unwrap();
        let mut broken = ErrorRefs::with_file(file, day(2026, 10, 10));
        assert_eq!(broken.next(day(2026, 10, 10)), "E-20261010-0001");
    }

    #[test]
    fn a_file_that_cannot_be_written_still_gives_numbers() {
        let mut refs =
            ErrorRefs::with_file(PathBuf::from("/no/such/folder/counter"), day(2026, 10, 10));
        assert_eq!(refs.next(day(2026, 10, 10)), "E-20261010-0001");
    }

    #[test]
    fn many_threads_never_share_a_number() {
        let threads: Vec<_> = (0..8)
            .map(|_| std::thread::spawn(|| (0..50).map(|_| next_reference()).collect::<Vec<_>>()))
            .collect();
        let mut all: Vec<String> = threads
            .into_iter()
            .flat_map(|t| t.join().unwrap())
            .collect();
        let total = all.len();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), total);
    }
}
