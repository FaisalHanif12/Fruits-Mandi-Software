//! Lockout after wrong tries (PRD FR-8.2, NFR-SEC-2).
//!
//! Per user name, counting wrong tries in a row:
//!
//! | Wrong tries | Result |
//! |---|---|
//! | 1 to 9 | wrong password, 9, 8 ... 1 tries left |
//! | 10 | locked 60 seconds |
//! | 11 to 19 (after the wait) | wrong password, 9, 8 ... 1 tries left |
//! | 20 | locked 300 seconds |
//! | 30, 40, 50 ... | locked 900 seconds each time |
//!
//! A right password sets the count back to 0. The same table is used for the partner code (PRD
//! FR-17.11, NFR-SEC-14). Umar, 12-11-2026: 10 tries in each step (it was 5).

/// Wrong tries before each lock.
pub const TRIES_PER_STEP: u32 = 10;
/// The first lock, in seconds (after one step of wrong tries).
pub const FIRST_LOCK_SECONDS: u64 = 60;
/// The second lock, in seconds (after two steps of wrong tries).
pub const SECOND_LOCK_SECONDS: u64 = 300;
/// Every later lock, in seconds (after three, four ... steps of wrong tries).
pub const LATER_LOCK_SECONDS: u64 = 900;

/// What a wrong try leads to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrongTry {
    /// Still allowed to try: this many tries until the next lock.
    TriesLeft(u32),
    /// Locked for this many seconds.
    Locked {
        /// How long the sign-in waits.
        seconds: u64,
    },
}

/// The length of the lock that starts at this count of wrong tries in a row, if one starts.
/// `None` when the count is 0 or not a multiple of [`TRIES_PER_STEP`].
pub fn lock_seconds_at(failed_attempts: u32) -> Option<u64> {
    if failed_attempts == 0 || failed_attempts.checked_rem(TRIES_PER_STEP) != Some(0) {
        return None;
    }
    Some(match failed_attempts {
        TRIES_PER_STEP => FIRST_LOCK_SECONDS,
        n if n == TRIES_PER_STEP.saturating_mul(2) => SECOND_LOCK_SECONDS,
        _ => LATER_LOCK_SECONDS,
    })
}

/// The length of the most recent lock for a count of wrong tries (the lock that began at the
/// last multiple of [`TRIES_PER_STEP`]). 0 when no lock was reached yet. A stored lock end time is never
/// trusted for longer than this (NFR-REL-11).
pub fn current_step_seconds(failed_attempts: u32) -> u64 {
    let past_multiple = failed_attempts.checked_rem(TRIES_PER_STEP).unwrap_or(0);
    let last_multiple = failed_attempts.saturating_sub(past_multiple);
    lock_seconds_at(last_multiple).unwrap_or(0)
}

/// What happens after a wrong try, given the count INCLUDING this try.
///
/// ```
/// use mandi_domain::auth::lockout::{WrongTry, after_wrong_try};
/// assert_eq!(after_wrong_try(1), WrongTry::TriesLeft(9));
/// assert_eq!(after_wrong_try(10), WrongTry::Locked { seconds: 60 });
/// assert_eq!(after_wrong_try(17), WrongTry::TriesLeft(3));
/// assert_eq!(after_wrong_try(20), WrongTry::Locked { seconds: 300 });
/// assert_eq!(after_wrong_try(30), WrongTry::Locked { seconds: 900 });
/// ```
pub fn after_wrong_try(failed_attempts: u32) -> WrongTry {
    match lock_seconds_at(failed_attempts) {
        Some(seconds) => WrongTry::Locked { seconds },
        None => {
            let past_multiple = failed_attempts.checked_rem(TRIES_PER_STEP).unwrap_or(0);
            WrongTry::TriesLeft(TRIES_PER_STEP.saturating_sub(past_multiple))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The worked table of plan 04 Section 3, with 10 tries in each step (Umar, 12-11-2026).
    #[test]
    fn the_plan_table() {
        let mut expected = Vec::new();
        for count in 1..=9_u32 {
            expected.push((count, WrongTry::TriesLeft(10 - count)));
        }
        expected.push((10, WrongTry::Locked { seconds: 60 }));
        for count in 11..=19_u32 {
            expected.push((count, WrongTry::TriesLeft(20 - count)));
        }
        expected.extend([
            (20, WrongTry::Locked { seconds: 300 }),
            (21, WrongTry::TriesLeft(9)),
            (29, WrongTry::TriesLeft(1)),
            (30, WrongTry::Locked { seconds: 900 }),
            (40, WrongTry::Locked { seconds: 900 }),
            (50, WrongTry::Locked { seconds: 900 }),
            (1000, WrongTry::Locked { seconds: 900 }),
        ]);
        for (count, want) in expected {
            assert_eq!(after_wrong_try(count), want, "count {count}");
        }
    }

    #[test]
    fn the_step_in_force() {
        assert_eq!(current_step_seconds(0), 0);
        assert_eq!(current_step_seconds(9), 0);
        assert_eq!(current_step_seconds(10), 60);
        assert_eq!(current_step_seconds(19), 60);
        assert_eq!(current_step_seconds(20), 300);
        assert_eq!(current_step_seconds(29), 300);
        assert_eq!(current_step_seconds(30), 900);
        assert_eq!(current_step_seconds(u32::MAX), 900);
    }

    #[test]
    fn no_lock_at_zero_and_never_a_panic() {
        assert_eq!(lock_seconds_at(0), None);
        assert_eq!(after_wrong_try(0), WrongTry::TriesLeft(10));
        // u32::MAX ends in 5, so it is not a multiple of 10; the one 5 below it is.
        assert_eq!(after_wrong_try(u32::MAX), WrongTry::TriesLeft(5));
        assert_eq!(
            after_wrong_try(u32::MAX - 5),
            WrongTry::Locked { seconds: 900 }
        );
        assert_eq!(after_wrong_try(u32::MAX - 6), WrongTry::TriesLeft(1));
    }
}
