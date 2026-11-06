//! Plan 55 gate 6: slow and lost lines. A scripted model on a bad line: each call can be delayed or
//! dropped. Offline: no key, no network, the model is a recorded transcript behind [`SlowLine`].
//!
//! The hard limits of plan 55 Section 3 (Umar: "over 30 s has no purpose"):
//!
//! - no question goes over 25 seconds;
//! - at 20 seconds the last call goes out with the tools off and answers from what has been read;
//! - at 25 seconds the panel shows what was read (the checked tables), not an error: no answer is
//!   ever lost;
//! - a slow line is a slow line, never an error.
//!
//! Three tests really wait (about 21 to 25 seconds each, side by side): the hard limits are built
//! (backend, S1 and S5), so they run with the rest. A drop or a hang after something was read ends as
//! `time_limit` with what was read (a report, a party card, the tiles); after a find alone it ends
//! with the failure.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/harness.rs"]
mod harness;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use harness::{Harness, answer_of};
use mandi_assistant::ask::{AskOutcome, AskReply, AssistantAskInput, FailureKind};
use mandi_assistant::client::{CallError, ModelClient, RecordedClient, Reply};
use mandi_assistant::run::ask;
use mandi_assistant::step::AssistantStep;
use serde_json::{Value, json};

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_200, "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 7_000},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

fn tool_reply(calls: &[(&str, Value)]) -> Value {
    let calls: Vec<Value> = calls
        .iter()
        .enumerate()
        .map(|(i, (name, args))| {
            json!({"id": format!("call_{i}"), "type": "function",
                   "function": {"name": name, "arguments": args.to_string()}})
        })
        .collect();
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null, "tool_calls": calls}}],
        "usage": usage(),
    })
}

fn final_reply(text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

fn find(text: &str, role: &str) -> (&'static str, Value) {
    ("find", json!({"text": text, "role": role}))
}

/// What the line did to one call.
#[derive(Debug, Clone)]
struct Note {
    /// When the call began, from the start of the question.
    at: Duration,
    /// The `tool_choice` of the request (`auto`, or `none` for the last call).
    tool_choice: String,
    /// The time limit the app gave this call.
    timeout: Duration,
    /// True when the line dropped it.
    dropped: bool,
}

/// A bad line in front of a recorded model. Call `n` (from 0, every attempt counts, a retry too)
/// waits `delays[n]` and fails when `drops[n]` is set. A call whose delay is longer than the time
/// the app gave it times out at that time, as a real client does.
struct SlowLine {
    inner: RecordedClient,
    delays: Vec<Duration>,
    drops: Vec<Option<FailureKind>>,
    notes: Mutex<Vec<Note>>,
    begin: Instant,
}

impl SlowLine {
    fn new(replies: Vec<Value>, delays: Vec<Duration>, drops: Vec<Option<FailureKind>>) -> Self {
        Self {
            inner: RecordedClient::new(replies),
            delays,
            drops,
            notes: Mutex::new(Vec::new()),
            begin: Instant::now(),
        }
    }

    fn notes(&self) -> Vec<Note> {
        self.notes.lock().expect("the notes").clone()
    }
}

impl ModelClient for SlowLine {
    fn retry_wait(&self) -> Duration {
        Duration::ZERO
    }

    fn call(&self, key: &str, body: &Value, timeout: Duration) -> Result<Reply, CallError> {
        let n = self.notes.lock().expect("the notes").len();
        let dropped = self.drops.get(n).copied().flatten();
        self.notes.lock().expect("the notes").push(Note {
            at: self.begin.elapsed(),
            tool_choice: body["tool_choice"].as_str().unwrap_or("").to_owned(),
            timeout,
            dropped: dropped.is_some(),
        });
        if let Some(kind) = dropped {
            return Err(CallError::new(kind));
        }
        let delay = self.delays.get(n).copied().unwrap_or(Duration::ZERO);
        if delay > timeout {
            std::thread::sleep(timeout);
            return Err(CallError::new(FailureKind::Timeout));
        }
        std::thread::sleep(delay);
        self.inner.call(key, body, timeout)
    }
}

fn ask_on(h: &Harness, line: &SlowLine, text: &str) -> (AskReply, Duration) {
    let emit = |_s: AssistantStep| {};
    let deps = h.deps(line, &emit);
    let started = Instant::now();
    let reply = ask(
        &deps,
        &AssistantAskInput {
            text: text.to_owned(),
            conversation_id: None,
        },
    )
    .expect("ask");
    (reply, started.elapsed())
}

const QUESTION: &str = "ikram ke total bakaya kitne hai";
const GOOD: &str = "{{r1.matches.0.name}} کا بقایا {{r2.balance}} ہے۔";

/// A `report` call with every field the schema holds.
fn report(id: &str) -> (&'static str, Value) {
    (
        "report",
        json!({
            "id": id, "adda": null, "as_of": null, "bikri": null, "city": null, "collector": null,
            "from": null, "garden": null, "group_by": null, "item": null, "partner": null,
            "party": null, "rows": null, "sort": null, "to": null, "trip": null, "where": null
        }),
    )
}

/// A question that reads a report with a table (the dues, R20), then answers.
fn dues_replies() -> Vec<Value> {
    vec![
        tool_reply(&[report("R20")]),
        final_reply("{{r1.totals.balance}}"),
    ]
}

/// A normal question: find the buyer, read his card, answer.
fn akram_replies() -> Vec<Value> {
    vec![
        tool_reply(&[find("ikram", "buyer")]),
        tool_reply(&[("party_card", json!({"party": "r1.matches.0"}))]),
        final_reply(GOOD),
    ]
}

fn seconds(n: u64) -> Duration {
    Duration::from_secs(n)
}

// ---- drops: fast, every run -------------------------------------------------------------------

#[test]
fn a_dropped_first_call_is_tried_once_more_and_the_question_is_answered() {
    for kind in [
        FailureKind::Offline,
        FailureKind::Dropped,
        FailureKind::Server,
    ] {
        let h = Harness::day();
        let line = SlowLine::new(akram_replies(), vec![], vec![Some(kind)]);
        let (reply, took) = ask_on(&h, &line, QUESTION);
        assert_eq!(reply.outcome, AskOutcome::Answered, "{kind:?}: {reply:#?}");
        assert!(took < seconds(25), "{kind:?}: {took:?}");
        let notes = line.notes();
        assert_eq!(
            notes.len(),
            4,
            "{kind:?}: the drop, then the three calls: {notes:?}"
        );
        assert!(notes[0].dropped && !notes[1].dropped);
    }
}

#[test]
fn a_dropped_call_in_the_middle_is_tried_once_more_and_the_question_is_answered() {
    for dropped_at in [1_usize, 2] {
        let h = Harness::day();
        let mut drops = vec![None; dropped_at + 1];
        drops[dropped_at] = Some(FailureKind::Dropped);
        let line = SlowLine::new(akram_replies(), vec![], drops);
        let (reply, _) = ask_on(&h, &line, QUESTION);
        assert_eq!(
            reply.outcome,
            AskOutcome::Answered,
            "call {dropped_at} dropped: {reply:#?}"
        );
    }
}

#[test]
fn a_line_that_drops_every_try_ends_with_the_reason_and_the_question_is_kept() {
    // Both tries of the first call drop: nothing was read, so there is nothing to show. The Owner is
    // told the line is down (not that the question is wrong), and the question is in the log, so it
    // can be asked again.
    let h = Harness::day();
    let line = SlowLine::new(
        akram_replies(),
        vec![],
        vec![Some(FailureKind::Offline), Some(FailureKind::Offline)],
    );
    let (reply, took) = ask_on(&h, &line, QUESTION);
    assert!(
        matches!(reply.outcome, AskOutcome::Offline | AskOutcome::Error),
        "{reply:#?}"
    );
    assert!(reply.failure.is_some(), "the reason is given: {reply:#?}");
    assert!(took < seconds(25), "{took:?}");
    let logged = mandi_services::assistant::get_question(&h.world.env.ctx, reply.question_id)
        .expect("the log")
        .expect("the question is logged");
    assert_eq!(logged.question, QUESTION);
}

#[test]
fn a_line_that_drops_after_a_report_was_read_shows_what_was_read_as_a_time_limit() {
    // Plan 55 Section 3: no answer is ever lost. The dues report was read and checked; then the line
    // dropped on the last call, twice. The Owner sees the table that was read, as a slow line
    // (`time_limit`), not an error.
    let h = Harness::day();
    let line = SlowLine::new(
        dues_replies(),
        vec![],
        vec![None, Some(FailureKind::Dropped), Some(FailureKind::Dropped)],
    );
    let (reply, took) = ask_on(&h, &line, "gahakon ke kul bakaya");
    assert!(took < seconds(25), "{took:?}");
    assert_eq!(reply.outcome, AskOutcome::TimeLimit, "{reply:#?}");
    assert!(reply.failure.is_none(), "a slow line is not a failure");
    let answer = answer_of(&reply);
    assert!(
        !answer.tables.is_empty(),
        "the table that was read: {reply:#?}"
    );
}

#[test]
fn a_line_that_drops_after_the_tiles_were_read_shows_them_as_a_time_limit() {
    // "aaj ki bikri kitni hui": the dashboard tiles were read (checked figures), then the last call
    // drops twice. The tiles count as read: the Owner sees them and the line is shown as slow.
    let h = Harness::day();
    let line = SlowLine::new(
        vec![
            tool_reply(&[("tiles", json!({}))]),
            final_reply("{{r1.today_sales.amount}}"),
        ],
        vec![],
        vec![None, Some(FailureKind::Dropped), Some(FailureKind::Dropped)],
    );
    let (reply, took) = ask_on(&h, &line, "aaj ki bikri kitni hui");
    assert!(took < seconds(25), "{took:?}");
    assert_eq!(reply.outcome, AskOutcome::TimeLimit, "{reply:#?}");
    assert!(reply.failure.is_none(), "a slow line is not a failure");
    let answer = answer_of(&reply);
    assert!(
        !answer.sources.is_empty() || !answer.parts.is_empty() || !answer.tables.is_empty(),
        "the tiles that were read are shown: {reply:#?}"
    );
}

#[test]
fn a_line_that_drops_after_only_a_find_ends_with_the_failure_and_not_with_a_guess() {
    // A find alone is a list of names, not a figure. The item banana was found (an item has no
    // card, so the app reads nothing more for it), then the last call drops twice. There is no
    // checked record to show, so the question ends with the failure (the line is down), counted
    // once, and is kept to ask again. Nothing is guessed from a name.
    let h = Harness::day();
    let question = "kele ka commission kitna hai";
    let line = SlowLine::new(
        vec![
            tool_reply(&[find("banana", "item")]),
            final_reply("{{r1.matches.0.name}}"),
        ],
        vec![],
        vec![None, Some(FailureKind::Dropped), Some(FailureKind::Dropped)],
    );
    let (reply, took) = ask_on(&h, &line, question);
    assert!(took < seconds(25), "{took:?}");
    assert!(
        matches!(reply.outcome, AskOutcome::Error | AskOutcome::Offline),
        "{reply:#?}"
    );
    assert_eq!(
        reply.failure.as_ref().map(|f| f.kind),
        Some(FailureKind::Dropped),
        "{reply:#?}"
    );
    assert!(reply.answer.is_none(), "nothing is shown for a find alone");
    assert_eq!(
        reply.count_today, 1,
        "the model answered once: counted once"
    );
    let logged = mandi_services::assistant::get_question(&h.world.env.ctx, reply.question_id)
        .expect("the log")
        .expect("the question is logged");
    assert_eq!(logged.question, question);
}

#[test]
fn a_line_that_drops_after_a_party_card_was_read_still_shows_what_was_read() {
    // The most common question: "ikram ke total bakaya kitne hai". The buyer was found and his card
    // (his checked balance) was read; the line dropped on the last call, twice. A card is a checked
    // record, so the Owner must see it, but a card is not a table.
    let h = Harness::day();
    let line = SlowLine::new(
        akram_replies(),
        vec![],
        vec![
            None,
            None,
            Some(FailureKind::Dropped),
            Some(FailureKind::Dropped),
        ],
    );
    let (reply, took) = ask_on(&h, &line, QUESTION);
    assert!(took < seconds(25), "{took:?}");
    let shown = reply
        .answer
        .as_ref()
        .map_or(0, |a| a.tables.len() + a.sources.len());
    assert!(
        shown > 0,
        "what was read is shown, not lost: {:?} {reply:#?}",
        reply.outcome
    );
}

// ---- slow: really wait ------------------------------------------------------------------------

#[test]
fn at_20_seconds_the_last_call_goes_out_with_the_tools_off() {
    // The first two calls take 10 and 11 seconds and each asks for one more read. The question has
    // used 21 s: the third call may not read anything, and answers from what was read.
    let h = Harness::day();
    let line = SlowLine::new(
        vec![
            tool_reply(&[report("R20")]),
            tool_reply(&[report("R24")]),
            final_reply("{{r1.totals.balance}}"),
        ],
        vec![seconds(10), seconds(11)],
        vec![],
    );
    let (reply, took) = ask_on(&h, &line, "gahakon ke kul bakaya");
    let notes = line.notes();
    assert!(took <= seconds(25), "no question over 25 s: {took:?}");
    assert_eq!(notes.len(), 3, "{notes:?}");
    assert_eq!(
        notes.last().map(|n| n.tool_choice.as_str()),
        Some("none"),
        "the call after 20 s has the tools off: {notes:?}"
    );
    assert!(notes[2].at >= seconds(20), "{notes:?}");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
}

#[test]
fn at_25_seconds_the_panel_shows_what_was_read_and_not_an_error() {
    // The dues report is read at once; then the model's next call never comes back (a line so slow
    // that the call hangs, and its one retry hangs too). At 25 s the question ends with the table
    // that was read, as a slow line.
    let h = Harness::day();
    let line = SlowLine::new(
        dues_replies(),
        vec![Duration::ZERO, seconds(100), seconds(100)],
        vec![],
    );
    let (reply, took) = ask_on(&h, &line, "gahakon ke kul bakaya");
    assert!(took <= seconds(26), "no question over 25 s: {took:?}");
    assert_eq!(
        reply.outcome,
        AskOutcome::TimeLimit,
        "a slow line is shown as a slow line, never as an error: {reply:#?}"
    );
    let answer = answer_of(&reply);
    assert!(
        !answer.tables.is_empty(),
        "what was read is shown: {reply:#?}"
    );
    for note in line.notes() {
        assert!(
            note.at + note.timeout <= seconds(25) + Duration::from_millis(500),
            "a call may not run past the 25 s limit: {note:?}"
        );
    }
}

#[test]
fn a_slow_first_call_never_pushes_the_question_past_25_seconds() {
    // Nothing is read yet and the first call hangs, and so does its one retry. The question ends in
    // at most 25 s with the reason, and is kept so it can be asked again. No call is given more
    // time than the question has left.
    let h = Harness::day();
    let line = SlowLine::new(akram_replies(), vec![seconds(100), seconds(100)], vec![]);
    let (reply, took) = ask_on(&h, &line, QUESTION);
    assert!(took <= seconds(26), "no question over 25 s: {took:?}");
    assert!(
        reply.failure.is_some() || reply.answer.is_some(),
        "{reply:#?}"
    );
    for note in line.notes() {
        assert!(
            note.at + note.timeout <= seconds(25) + Duration::from_millis(500),
            "a call may not run past the 25 s limit: {note:?}"
        );
    }
    assert!(
        line.notes().iter().any(|n| n.timeout <= seconds(12)),
        "the first try is cut at 12 s: {:?}",
        line.notes()
    );
}
