//! A call to the model with a TRUE deadline and a second copy when the first is slow (plan 55
//! Section 3; the lead's decisions of 05-11-2026 after live run 1).
//!
//! Two faults of the plain request, both seen in the live run:
//!
//! - The HTTP client's timeout restarts when the headers arrive, so a call could take twice its
//!   cut (Q244: 23.8 s against a 12 s cut, and six questions ended past 25 s). Here each request
//!   runs on its own thread and the caller waits for the first result until the real deadline, so
//!   [`hedged`] returns at `timeout` at the latest, whatever the request does.
//! - About 4% of the calls got no byte at all for 12 s and then again for 11 s (Azure stalls).
//!   Short calls end in 6 s in 90% of the cases and in 8 s in 94%. So when nothing has come by
//!   [`HEDGE_AFTER_SECS`], a second identical request is sent, and whichever answers first is
//!   used. The other is dropped (its thread ends at its own deadline and its answer is ignored).
//!   The question counts it once; both requests are written to the log.
//!
//! Example: a call with 12 s: the first request is sent at 0 s; no reply at 7 s, so a copy is sent
//! with 5 s; the copy answers at 9 s: the call returns that reply at 9 s, and the note says
//! "copy won, first cancelled after 9 s".

use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use crate::ask::FailureKind;
use crate::client::{CallError, Reply};

/// Which of the two requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The request sent first.
    First,
    /// The identical request sent when the first was slow.
    Copy,
}

/// How one request ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    /// Its reply was used.
    Won,
    /// It was still waiting when the other one answered; its answer, if it comes, is ignored.
    Cancelled,
    /// It failed on its own (or ran into the deadline).
    Failed(FailureKind),
}

/// One request of a hedged call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attempt {
    /// Which request.
    pub side: Side,
    /// Milliseconds from its own start to its end (or to the moment it was given up).
    pub millis: i64,
    /// How it ended.
    pub end: End,
}

/// What a hedged call did: the requests, in the order they were sent, and the time of the whole
/// call. Only present when the copy was sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hedge {
    /// One or two requests; two when the copy was sent.
    pub attempts: Vec<Attempt>,
    /// Milliseconds from the first send to the end of the call.
    pub wall_millis: i64,
}

fn millis(d: Duration) -> i64 {
    i64::try_from(d.as_millis()).unwrap_or(i64::MAX)
}

/// A call to `send` that is over at `timeout` at the latest, with a second copy of it after
/// `hedge_after` when at least `min_copy` is left for it.
///
/// `send(budget)` is one whole request that gives up by itself after `budget`. A failure of the
/// first request before `hedge_after` is returned at once, with no copy: the caller's own retry
/// rule handles it. A failure after the copy was sent waits for the other request. When both fail,
/// the later failure is returned.
pub fn hedged_with<F>(
    send: F,
    timeout: Duration,
    hedge_after: Duration,
    min_copy: Duration,
) -> (Result<Reply, CallError>, Option<Hedge>)
where
    F: Fn(Duration) -> Result<Reply, CallError> + Send + Sync + 'static,
{
    let send = Arc::new(send);
    let (tx, rx) = mpsc::channel::<(Side, Result<Reply, CallError>, Duration)>();
    let begin = Instant::now();
    let start = |side: Side, budget: Duration| {
        let send = Arc::clone(&send);
        let tx = tx.clone();
        std::thread::spawn(move || {
            let at = Instant::now();
            let result = send(budget);
            // The receiver is gone when the other request already won: nothing to tell.
            let _ = tx.send((side, result, at.elapsed()));
        });
    };
    start(Side::First, timeout);
    let may_copy = timeout.saturating_sub(hedge_after) >= min_copy;
    // When the copy was sent, from the first send.
    let mut copy_at: Option<Duration> = None;
    // The requests that failed by themselves, with the time each took.
    let mut failed: Vec<(Side, CallError, Duration)> = Vec::new();
    loop {
        let elapsed = begin.elapsed();
        if elapsed >= timeout {
            let hedge = hedge_note(None, elapsed, copy_at, &failed);
            return (Err(CallError::new(FailureKind::Timeout)), hedge);
        }
        let left = timeout.saturating_sub(elapsed);
        let wait = if may_copy && copy_at.is_none() {
            hedge_after.saturating_sub(elapsed).min(left)
        } else {
            left
        };
        match rx.recv_timeout(wait) {
            Ok((side, Ok(reply), _)) => {
                let now = begin.elapsed();
                return (Ok(reply), hedge_note(Some(side), now, copy_at, &failed));
            }
            Ok((side, Err(error), took)) => {
                failed.push((side, error, took));
                if copy_at.is_none() {
                    // It failed before the copy was due: the caller's retry rule takes it.
                    return (Err(error), None);
                }
                if failed.len() >= 2 {
                    let now = begin.elapsed();
                    return (Err(error), hedge_note(None, now, copy_at, &failed));
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if may_copy && copy_at.is_none() && begin.elapsed() >= hedge_after {
                    let at = begin.elapsed();
                    copy_at = Some(at);
                    start(Side::Copy, timeout.saturating_sub(at));
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return (Err(CallError::new(FailureKind::Dropped)), None);
            }
        }
    }
}

/// The note of a hedged call, when the copy was sent (`copy_at` is when). `winner` is the side
/// whose reply was used, if any; a request that failed by itself shows its own failure; one that
/// was still waiting shows as cancelled when the other won, and as timed out at the deadline.
fn hedge_note(
    winner: Option<Side>,
    now: Duration,
    copy_at: Option<Duration>,
    failed: &[(Side, CallError, Duration)],
) -> Option<Hedge> {
    let copy_at = copy_at?;
    let mut attempts = Vec::new();
    for side in [Side::First, Side::Copy] {
        let started = match side {
            Side::First => Duration::ZERO,
            Side::Copy => copy_at,
        };
        let own = failed.iter().find(|(s, _, _)| *s == side);
        let (took, end) = match (winner, own) {
            (Some(w), _) if w == side => (now.saturating_sub(started), End::Won),
            (_, Some((_, error, took))) => (*took, End::Failed(error.kind)),
            (Some(_), None) => (now.saturating_sub(started), End::Cancelled),
            // The deadline came while it was still waiting: it timed out.
            (None, None) => (
                now.saturating_sub(started),
                End::Failed(FailureKind::Timeout),
            ),
        };
        attempts.push(Attempt {
            side,
            millis: millis(took),
            end,
        });
    }
    Some(Hedge {
        attempts,
        wall_millis: millis(now),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use serde_json::json;

    use super::*;

    fn reply(tag: &str) -> Reply {
        Reply::from_json(&json!({
            "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": tag}}],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1}
        }))
        .expect("a reply")
    }

    const SHORT: Duration = Duration::from_millis(150);
    const NO_MIN: Duration = Duration::ZERO;

    #[test]
    fn a_reply_before_the_copy_is_due_sends_no_copy() {
        let sent = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&sent);
        let (result, hedge) = hedged_with(
            move |_| {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(reply("first"))
            },
            Duration::from_secs(2),
            SHORT,
            NO_MIN,
        );
        assert_eq!(result.unwrap().content.as_deref(), Some("first"));
        assert!(hedge.is_none());
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(sent.load(Ordering::SeqCst), 1, "one request only");
    }

    #[test]
    fn a_slow_first_request_loses_to_the_copy_which_answers_at_once() {
        let sent = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&sent);
        let started = Instant::now();
        let (result, hedge) = hedged_with(
            move |_| {
                if counter.fetch_add(1, Ordering::SeqCst) == 0 {
                    std::thread::sleep(Duration::from_millis(1_500));
                    Ok(reply("first, too late"))
                } else {
                    std::thread::sleep(Duration::from_millis(20));
                    Ok(reply("copy"))
                }
            },
            Duration::from_secs(3),
            SHORT,
            NO_MIN,
        );
        let took = started.elapsed();
        assert_eq!(result.unwrap().content.as_deref(), Some("copy"));
        assert!(took < Duration::from_millis(600), "{took:?}");
        let hedge = hedge.expect("the copy was sent");
        let ends: Vec<(Side, End)> = hedge.attempts.iter().map(|a| (a.side, a.end)).collect();
        assert_eq!(
            ends,
            [(Side::First, End::Cancelled), (Side::Copy, End::Won)]
        );
        assert!(
            hedge.wall_millis >= 150 && hedge.wall_millis < 600,
            "{hedge:?}"
        );
    }

    #[test]
    fn the_first_request_can_still_win_after_the_copy_was_sent() {
        let sent = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&sent);
        let (result, hedge) = hedged_with(
            move |_| {
                if counter.fetch_add(1, Ordering::SeqCst) == 0 {
                    std::thread::sleep(Duration::from_millis(300));
                    Ok(reply("first"))
                } else {
                    std::thread::sleep(Duration::from_millis(1_000));
                    Ok(reply("copy, too late"))
                }
            },
            Duration::from_secs(3),
            SHORT,
            NO_MIN,
        );
        assert_eq!(result.unwrap().content.as_deref(), Some("first"));
        let ends: Vec<(Side, End)> = hedge
            .unwrap()
            .attempts
            .iter()
            .map(|a| (a.side, a.end))
            .collect();
        assert_eq!(
            ends,
            [(Side::First, End::Won), (Side::Copy, End::Cancelled)]
        );
    }

    #[test]
    fn a_failure_before_the_copy_is_due_is_returned_at_once_with_no_copy() {
        let sent = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&sent);
        let started = Instant::now();
        let (result, hedge) = hedged_with(
            move |_| {
                counter.fetch_add(1, Ordering::SeqCst);
                Err(CallError::new(FailureKind::Busy))
            },
            Duration::from_secs(2),
            Duration::from_millis(500),
            NO_MIN,
        );
        assert_eq!(result.unwrap_err().kind, FailureKind::Busy);
        assert!(hedge.is_none());
        assert!(started.elapsed() < Duration::from_millis(300));
        assert_eq!(sent.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn when_the_first_fails_after_the_copy_the_copy_is_waited_for() {
        let sent = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&sent);
        let (result, hedge) = hedged_with(
            move |_| {
                if counter.fetch_add(1, Ordering::SeqCst) == 0 {
                    std::thread::sleep(Duration::from_millis(300));
                    Err(CallError::new(FailureKind::Dropped))
                } else {
                    std::thread::sleep(Duration::from_millis(400));
                    Ok(reply("copy"))
                }
            },
            Duration::from_secs(3),
            SHORT,
            NO_MIN,
        );
        assert_eq!(result.unwrap().content.as_deref(), Some("copy"));
        let ends: Vec<(Side, End)> = hedge
            .unwrap()
            .attempts
            .iter()
            .map(|a| (a.side, a.end))
            .collect();
        assert_eq!(
            ends,
            [
                (Side::First, End::Failed(FailureKind::Dropped)),
                (Side::Copy, End::Won)
            ]
        );
    }

    #[test]
    fn a_request_that_never_ends_is_over_at_the_deadline_and_both_are_logged() {
        // The requests ignore their budget, as a stuck connection does: the caller must not wait.
        let started = Instant::now();
        let (result, hedge) = hedged_with(
            |_| {
                std::thread::sleep(Duration::from_secs(5));
                Ok(reply("never used"))
            },
            Duration::from_millis(600),
            SHORT,
            NO_MIN,
        );
        let took = started.elapsed();
        assert_eq!(result.unwrap_err().kind, FailureKind::Timeout);
        assert!(
            took >= Duration::from_millis(600) && took < Duration::from_millis(900),
            "{took:?}"
        );
        let hedge = hedge.expect("the copy was sent");
        assert!(
            hedge
                .attempts
                .iter()
                .all(|a| a.end == End::Failed(FailureKind::Timeout))
        );
        assert_eq!(hedge.attempts.len(), 2);
    }

    #[test]
    fn no_copy_is_sent_when_too_little_time_is_left_for_it() {
        let sent = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&sent);
        let (result, hedge) = hedged_with(
            move |_| {
                counter.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(500));
                Ok(reply("first"))
            },
            Duration::from_secs(1),
            Duration::from_millis(300),
            Duration::from_millis(900),
        );
        assert_eq!(result.unwrap().content.as_deref(), Some("first"));
        assert!(hedge.is_none());
        assert_eq!(sent.load(Ordering::SeqCst), 1);
    }
}
