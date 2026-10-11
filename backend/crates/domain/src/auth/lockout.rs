//! Lockout after wrong tries (PRD FR-8.2, NFR-SEC-2).
//!
//! Per user name, counting wrong tries in a row:
//!
//! | Wrong tries | Result |
//! |---|---|
//! | 1 to 4 | wrong password, 4, 3, 2, 1 tries left |
//! | 5 | locked 60 seconds |
//! | 6 to 9 (after the wait) | wrong password, 4, 3, 2, 1 tries left |
//! | 10 | locked 300 seconds |
//! | 15, 20, 25 ... | locked 900 seconds each time |
//!
//! A right password sets the count back to 0.

/// Wrong tries before each lock.
pub const TRIES_PER_STEP: u32 = 5;
/// The first lock, in seconds (after 5 wrong tries).
pub const FIRST_LOCK_SECONDS: u64 = 60;
/// The second lock, in seconds (after 10 wrong tries).
pub const SECOND_LOCK_SECONDS: u64 = 300;
/// Every later lock, in seconds (after 15, 20, 25 ... wrong tries).
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
/// `None` when the count is 0 or not a multiple of 5.
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
/// last multiple of 5). 0 when no lock was reached yet. A stored lock end time is never
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
/// assert_eq!(after_wrong_try(1), WrongTry::TriesLeft(4));
/// assert_eq!(after_wrong_try(5), WrongTry::Locked { seconds: 60 });
/// assert_eq!(after_wrong_try(7), WrongTry::TriesLeft(3));
/// assert_eq!(after_wrong_try(10), WrongTry::Locked { seconds: 300 });
/// assert_eq!(after_wrong_try(15), WrongTry::Locked { seconds: 900 });
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

    /// The worked table of plan 04 Section 3.
    #[test]
    fn the_plan_table() {
        let expected = [
            (1, WrongTry::TriesLeft(4)),
            (2, WrongTry::TriesLeft(3)),
            (3, WrongTry::TriesLeft(2)),
            (4, WrongTry::TriesLeft(1)),
            (5, WrongTry::Locked { seconds: 60 }),
            (6, WrongTry::TriesLeft(4)),
            (7, WrongTry::TriesLeft(3)),
            (8, WrongTry::TriesLeft(2)),
            (9, WrongTry::TriesLeft(1)),
            (10, WrongTry::Locked { seconds: 300 }),
            (11, WrongTry::TriesLeft(4)),
            (14, WrongTry::TriesLeft(1)),
            (15, WrongTry::Locked { seconds: 900 }),
            (20, WrongTry::Locked { seconds: 900 }),
            (25, WrongTry::Locked { seconds: 900 }),
            (1000, WrongTry::Locked { seconds: 900 }),
        ];
        for (count, want) in expected {
            assert_eq!(after_wrong_try(count), want, "count {count}");
        }
    }

    #[test]
    fn the_step_in_force() {
        assert_eq!(current_step_seconds(0), 0);
        assert_eq!(current_step_seconds(4), 0);
        assert_eq!(current_step_seconds(5), 60);
        assert_eq!(current_step_seconds(9), 60);
        assert_eq!(current_step_seconds(10), 300);
        assert_eq!(current_step_seconds(14), 300);
        assert_eq!(current_step_seconds(15), 900);
        assert_eq!(current_step_seconds(u32::MAX), 900);
    }

    #[test]
    fn no_lock_at_zero_and_never_a_panic() {
        assert_eq!(lock_seconds_at(0), None);
        assert_eq!(after_wrong_try(0), WrongTry::TriesLeft(5));
        // u32::MAX ends in 5, so it is a multiple of 5.
        assert_eq!(after_wrong_try(u32::MAX), WrongTry::Locked { seconds: 900 });
        assert_eq!(after_wrong_try(u32::MAX - 1), WrongTry::TriesLeft(1));
    }
}
