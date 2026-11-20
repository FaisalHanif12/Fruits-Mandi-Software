//! The warm-up call (plan 82, NFR-SEC-9 v1.22, Tech Stack 4.8).
//!
//! The first call of a question after a pause was the slowest one of all: 14 to 32 s in the live
//! check of 20-11-2026, against 3 to 5 s for the next question. It paid for a new TCP and TLS
//! connection to the provider and for the provider's prompt cache, which had gone cold (the fixed
//! part of a request is about 11,700 tokens). When the question panel opens, or its box gets the
//! first key, the app sends ONE small request that holds only that fixed part, so the connection
//! and the cache are ready when the Owner presses Enter.
//!
//! What it holds: the instructions, the tools and the answer schema, the same as every request, and
//! one neutral user message ([`WARM_TEXT`]). No books data, no question, no conversation. The model
//! may write at most [`WARM_MAX_OUTPUT_TOKENS`] tokens, and its reply is thrown away. It is never a
//! question: no log row, not counted in the daily 20, no step line. It is sent at most once in
//! [`WARM_MIN_GAP_SECS`] and [`WARM_PER_DAY`] times a day, only when the assistant is on and a key
//! is saved, and not while a question is running (its connection is warm already).
//!
//! Example: the Owner presses `F1` at 9:00; the app warms the line; he types for 5 s and presses
//! Enter; the question's first call finds an open connection and a hot cache.

use std::time::{Duration, Instant};

use mandi_services::assistant;
use serde_json::Value;

use crate::conversation::{Conversation, Turn};
use crate::request::{Parts, request_body, user_message};
use crate::results::Words;
use crate::run::Deps;

/// The least time between two warm-ups.
pub const WARM_MIN_GAP_SECS: u64 = 300;
/// The most warm-ups in one day (the Pakistan day).
pub const WARM_PER_DAY: u32 = 12;
/// The most output tokens the warm-up asks for.
pub const WARM_MAX_OUTPUT_TOKENS: u32 = 16;
/// How long the warm-up may take at the most. Nothing waits for it.
pub const WARM_TIMEOUT_SECS: u64 = 15;
/// The one neutral line the model is given. It is not a question and holds no data.
pub const WARM_TEXT: &str = "warm-up";

/// How a call of [`warm`] ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarmOutcome {
    /// The request went out and came back (its reply is not used).
    Sent,
    /// The request went out and failed. Nothing is wrong: the question will make its own.
    Failed,
    /// The assistant is off or no key is saved.
    Off,
    /// A question is running: its connection is warm.
    Busy,
    /// The last warm-up was less than [`WARM_MIN_GAP_SECS`] ago.
    TooSoon,
    /// [`WARM_PER_DAY`] warm-ups were sent today.
    DayFull,
}

/// When a warm-up may go out. Kept in the [`crate::runtime::Runtime`].
#[derive(Debug, Default)]
pub struct WarmGate {
    last: Option<Instant>,
    day: String,
    count: u32,
}

impl WarmGate {
    /// Asks for a warm-up at `now` on the Pakistan day `day`. When it may go out, it is written
    /// down at once (a warm-up that fails still counts, so a bad line is not hammered).
    pub fn allow(&mut self, now: Instant, day: &str) -> Result<(), WarmOutcome> {
        if self.day != day {
            self.day = day.to_owned();
            self.count = 0;
        }
        if self.count >= WARM_PER_DAY {
            return Err(WarmOutcome::DayFull);
        }
        if let Some(last) = self.last
            && now.saturating_duration_since(last) < Duration::from_secs(WARM_MIN_GAP_SECS)
        {
            return Err(WarmOutcome::TooSoon);
        }
        self.last = Some(now);
        self.count = self.count.saturating_add(1);
        Ok(())
    }
}

/// The body of the warm-up: the same fixed part as every request, with one neutral user message and
/// the tools switched off, so the model has nothing to do but stop.
pub fn warm_body(
    model: &str,
    today: &str,
    language: mandi_services::core::auth::Language,
) -> Value {
    let words = match Words::new() {
        Ok(words) => words,
        Err(_) => return Value::Null,
    };
    let turn = Turn {
        question: WARM_TEXT.to_owned(),
        messages: vec![user_message(today, language, WARM_TEXT, &[])],
        ..Turn::default()
    };
    let conversation = Conversation::new("warm-up");
    request_body(&Parts {
        model,
        conversation: &conversation,
        current: &turn,
        words: &words,
        partner_locked: true,
        max_completion_tokens: WARM_MAX_OUTPUT_TOKENS,
        tools_off: true,
        low_effort: true,
    })
}

/// Sends the warm-up when it is allowed. It runs on the caller's thread and takes at most
/// [`WARM_TIMEOUT_SECS`]; the command that calls it does not make the screen wait.
pub fn warm(d: &Deps<'_>) -> WarmOutcome {
    let Ok(config) = assistant::config(d.ctx) else {
        return WarmOutcome::Off;
    };
    if !config.enabled {
        return WarmOutcome::Off;
    }
    let Ok(Some(key)) = d.runtime.keys().get() else {
        return WarmOutcome::Off;
    };
    if d.runtime.is_busy() {
        return WarmOutcome::Busy;
    }
    let day = assistant::pakistan_day(d.ctx);
    if let Err(why) = d.runtime.allow_warm(Instant::now(), &day) {
        return why;
    }
    let language =
        assistant::language(d.ctx, d.actor).unwrap_or(mandi_services::core::auth::Language::Ur);
    let model = config
        .model
        .unwrap_or_else(|| crate::settings::DEFAULT_MODEL.to_owned());
    let body = warm_body(&model, &day, language);
    if body.is_null() {
        return WarmOutcome::Failed;
    }
    match d
        .client
        .call(&key, &body, Duration::from_secs(WARM_TIMEOUT_SECS))
    {
        Ok(_) => WarmOutcome::Sent,
        Err(_) => WarmOutcome::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_in_five_minutes_and_twelve_a_day() {
        let mut gate = WarmGate::default();
        let t0 = Instant::now();
        assert_eq!(gate.allow(t0, "2026-11-20"), Ok(()));
        assert_eq!(
            gate.allow(t0 + Duration::from_secs(299), "2026-11-20"),
            Err(WarmOutcome::TooSoon)
        );
        let mut at = t0;
        let mut sent = 1;
        for _ in 0..30 {
            at += Duration::from_secs(WARM_MIN_GAP_SECS);
            if gate.allow(at, "2026-11-20").is_ok() {
                sent += 1;
            }
        }
        assert_eq!(sent, WARM_PER_DAY, "the day is full after 12");
        assert_eq!(
            gate.allow(at + Duration::from_secs(1_000), "2026-11-20"),
            Err(WarmOutcome::DayFull)
        );
        // The next day starts again.
        assert_eq!(
            gate.allow(at + Duration::from_secs(1_000), "2026-11-21"),
            Ok(())
        );
    }
}
