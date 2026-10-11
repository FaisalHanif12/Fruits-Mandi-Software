//! Lockout memory (PRD FR-8.2, NFR-SEC-2, NFR-REL-11).
//!
//! A lock is stored twice. In the database as `users.locked_until_at` (wall clock), so it
//! survives a restart. In memory as a monotonic deadline, so while the app runs, moving the
//! computer's clock back never ends a lock early or makes it longer, and moving it forward
//! never ends it early. After a restart only the wall clock is left, so the stored end time
//! is trusted for no longer than the length of the lock step the user is in.

use std::time::Duration;

use chrono::{DateTime, Utc};
use mandi_domain::auth::lockout::current_step_seconds;

use crate::context::{Context, MAX_UNKNOWN_NAMES, MemoryLock};
use crate::store::UserRecord;

/// The longest user name remembered for lockouts, in characters.
const MAX_KEY_CHARS: usize = 64;

/// The key a typed user name is counted under: trimmed, lower case, cut to 64 characters.
/// An unknown name and a known one are treated the same way.
pub(crate) fn key_for(typed: &str) -> String {
    typed
        .trim()
        .to_lowercase()
        .chars()
        .take(MAX_KEY_CHARS)
        .collect()
}

/// Whole seconds, rounded up, never 0 for a duration above 0.
pub(crate) fn ceil_seconds(duration: Duration) -> u64 {
    let whole = duration.as_secs();
    if duration.subsec_nanos() > 0 {
        whole.saturating_add(1)
    } else {
        whole
    }
}

/// Seconds the sign-in still waits for this name, or `None` if it may try.
pub(crate) fn remaining_seconds(
    ctx: &Context,
    key: &str,
    user: Option<&UserRecord>,
) -> Option<u64> {
    let now = ctx.monotonic.elapsed();
    let mut lockouts = ctx.lockouts_lock();
    if let Some(lock) = lockouts.locks.get(key) {
        // This run of the app knows the lock: its monotonic deadline decides, alone.
        return (lock.until > now).then(|| ceil_seconds(lock.until.saturating_sub(now)));
    }
    // Only the stored end time is left (a restart). Use it, but never for longer than the step.
    let user = user?;
    let until: DateTime<Utc> = DateTime::parse_from_rfc3339(user.locked_until_at.as_deref()?)
        .ok()?
        .with_timezone(&Utc);
    let wall_left = (until - ctx.now_utc()).to_std().ok()?;
    let cap = Duration::from_secs(current_step_seconds(user.failed_attempts));
    let left = wall_left.min(cap);
    if left.is_zero() {
        return None;
    }
    // From now on the monotonic deadline decides, so a later clock change cannot change it.
    lockouts.locks.insert(
        key.to_owned(),
        MemoryLock {
            until: now.saturating_add(left),
        },
    );
    Some(ceil_seconds(left))
}

/// Starts a lock of `seconds` from now.
pub(crate) fn start(ctx: &Context, key: &str, seconds: u64) {
    let until = ctx
        .monotonic
        .elapsed()
        .saturating_add(Duration::from_secs(seconds));
    ctx.lockouts_lock()
        .locks
        .insert(key.to_owned(), MemoryLock { until });
}

/// Forgets everything about this name (after a right password or a reset).
pub(crate) fn clear(ctx: &Context, key: &str) {
    let mut lockouts = ctx.lockouts_lock();
    lockouts.locks.remove(key);
    lockouts.unknown_tries.remove(key);
}

/// Counts one wrong try of a name that is not a user, and returns how many in a row.
pub(crate) fn count_unknown(ctx: &Context, key: &str) -> u32 {
    let mut lockouts = ctx.lockouts_lock();
    if lockouts.unknown_tries.len() >= MAX_UNKNOWN_NAMES
        && !lockouts.unknown_tries.contains_key(key)
    {
        // Bounded memory: forget the old names rather than grow without end.
        lockouts.unknown_tries.clear();
    }
    let count = lockouts.unknown_tries.entry(key.to_owned()).or_insert(0);
    *count = count.saturating_add(1);
    *count
}
