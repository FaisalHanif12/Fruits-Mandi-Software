//! Plan 82: a slow model on a FAKE clock, and the effort of every call.
//!
//! The recorded client says each call took 15 seconds (`with_latency`). Nothing waits for real: the
//! loop adds the fake time to its own clock, so the limits of the question (20 s a try, 45 s in all,
//! the tools off from 30 s) are seen working with the real code of the loop. The real-time tests of
//! the deadline are `true_deadline.rs` (a local server) and `robustness_slow_line.rs`.
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

use std::time::Duration;

use harness::Harness;
use mandi_assistant::ask::AskOutcome;
use mandi_assistant::client::{CallError, ModelClient, RecordedClient};
use mandi_assistant::limits::{
    FIRST_BYTE_SECS, HEDGE_AFTER_SECS, QUESTION_TIMEOUT_SECS, TOOLS_OFF_AT_SECS,
};
use serde_json::{Value, json};

const QUESTION: &str = "ikram ke total bakaya kitne hai";

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

fn answer_reply(text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

/// The model finds the buyer; the app reads his card itself (plan 52 P52-S1).
fn find_akram() -> Value {
    tool_reply(&[("find", json!({"text": "akram", "role": "buyer"}))])
}

/// The model reads the buyers' dues (R20).
fn read_dues() -> Value {
    let report = json!({
        "id": "R20", "adda": null, "as_of": null, "bikri": null, "city": null, "collector": null,
        "from": null, "garden": null, "group_by": null, "item": null, "partner": null,
        "party": null, "rows": null, "sort": null, "to": null, "trip": null, "where": null
    });
    tool_reply(&[("report", report)])
}

/// The seconds the fake clock has run.
fn skew(client: &RecordedClient) -> u64 {
    client.clock_skew().as_secs()
}

/// True when two times differ by less than a second: the loop's own clock runs a few milliseconds
/// beside the fake one.
fn near(a: Duration, b: Duration) -> bool {
    a.abs_diff(b) < secs(1)
}

fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

fn effort(request: &Value) -> &str {
    request["reasoning"]["effort"].as_str().unwrap_or("")
}

#[test]
fn the_limits_are_the_ones_of_plan_82() {
    assert_eq!(FIRST_BYTE_SECS, 20);
    assert_eq!(QUESTION_TIMEOUT_SECS, 45);
    assert_eq!(HEDGE_AFTER_SECS, 7);
    // The last call keeps time to answer: a call takes 4 to 10 s, and the try is 20.
    const { assert!(QUESTION_TIMEOUT_SECS - TOOLS_OFF_AT_SECS >= 10) };
    const { assert!(TOOLS_OFF_AT_SECS < QUESTION_TIMEOUT_SECS) };
}

#[test]
fn a_model_that_takes_15_seconds_a_call_still_finishes_a_two_call_question() {
    let client = RecordedClient::new(vec![find_akram(), answer_reply("جی")]).with_latency(secs(15));
    let h = Harness::day();
    let reply = h.ask_with(QUESTION, None, &client);
    // Two calls, 15 s each, are 30 s of the 45: answered, not cut short.
    assert_eq!(client.requests().len(), 2, "{reply:#?}");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(skew(&client), 30);
    // Each call was given the 20 s of a try (the second has 30 s left, so still 20).
    let waits = client.waits();
    assert!(
        near(waits[0], secs(20)) && near(waits[1], secs(20)),
        "{waits:?}"
    );
}

#[test]
fn from_30_seconds_the_next_call_goes_out_with_the_tools_off() {
    // Read the dues (16 s), find the buyer (31 s), then the third call has to answer.
    let client = RecordedClient::new(vec![read_dues(), find_akram(), answer_reply("جی")])
        .with_latencies(vec![secs(16), secs(15), secs(10)]);
    let h = Harness::day();
    let reply = h.ask_with(QUESTION, None, &client);
    let requests = client.requests();
    assert_eq!(requests.len(), 3, "{reply:#?}");
    assert_eq!(requests[0]["tool_choice"], "auto");
    assert_eq!(
        requests[1]["tool_choice"], "auto",
        "at 16 s the tools are on"
    );
    assert_eq!(
        requests[2]["tool_choice"], "none",
        "at 31 s the call must answer from what was read"
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
}

#[test]
fn a_call_slower_than_its_try_is_cut_at_20_seconds_and_tried_once_more() {
    // The first call takes 25 s: it is cut at 20 s. The retry has the 25 s that are left, and its
    // reply comes after 15 s: the question ends at 35 s with its answer.
    let client =
        RecordedClient::new(vec![answer_reply("جی")]).with_latencies(vec![secs(25), secs(15)]);
    let h = Harness::day();
    let reply = h.ask_with("kya haal hai", None, &client);
    let waits = client.waits();
    assert!(
        near(waits[0], secs(20)) && near(waits[1], secs(25)),
        "{waits:?}"
    );
    assert_eq!(skew(&client), 35);
    assert_ne!(reply.outcome, AskOutcome::Error, "{reply:#?}");
    assert_ne!(reply.outcome, AskOutcome::TimeLimit, "{reply:#?}");
}

#[test]
fn a_model_that_is_always_too_slow_ends_at_45_seconds_with_what_was_read() {
    // The first call (15 s) finds the buyer and the app reads his card. Every call after it takes
    // 25 s: cut at 20 s, the retry has 10 s, and it is cut too. The question ends at 45 s and shows
    // the card that was read, never nothing and never past 45 s.
    let client = RecordedClient::with_results(vec![
        Ok(find_akram()),
        Err(CallError::new(mandi_assistant::ask::FailureKind::Timeout)),
    ])
    .with_latencies(vec![secs(15), secs(25)]);
    let h = Harness::day();
    let reply = h.ask_with(QUESTION, None, &client);
    assert_eq!(reply.outcome, AskOutcome::TimeLimit, "{reply:#?}");
    assert!(reply.answer.is_some(), "the card that was read is shown");
    assert!(skew(&client) <= QUESTION_TIMEOUT_SECS);
    assert_eq!(
        client.waits().len(),
        3,
        "the first call, the second and its retry"
    );
    assert!(near(client.waits()[1], secs(20)), "{:?}", client.waits());
    assert!(near(client.waits()[2], secs(10)), "{:?}", client.waits());
}

#[test]
fn every_call_thinks_low_but_the_rewrite_of_a_refused_answer() {
    // The model finds the buyer, writes his balance as digits (refused: every figure is a
    // reference), and writes it again once. The rewrite is the one call that thinks "medium".
    let client = RecordedClient::new(vec![
        find_akram(),
        answer_reply("اکرم کے ذمے 2,880 باقی ہیں"),
        answer_reply("جی"),
    ]);
    let h = Harness::day();
    h.runtime.set_low_effort(true);
    let _ = h.ask_with(QUESTION, None, &client);
    let requests = client.requests();
    assert_eq!(requests.len(), 3);
    assert_eq!(
        effort(&requests[0]),
        "low",
        "the first call, nothing read yet"
    );
    assert_eq!(
        effort(&requests[1]),
        "low",
        "the call that writes the answer"
    );
    assert_eq!(effort(&requests[2]), "medium", "the rewrite");
}

#[test]
fn with_the_switch_off_every_call_thinks_medium() {
    let client = RecordedClient::new(vec![find_akram(), answer_reply("جی")]);
    let h = Harness::day();
    h.runtime.set_low_effort(false);
    let _ = h.ask_with(QUESTION, None, &client);
    for request in client.requests() {
        assert_eq!(effort(&request), "medium");
    }
}
