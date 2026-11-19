//! Plan 54 S1 and S5, plan 55 Section 3: the provider is asked for the fastest Azure server, a try
//! of a call waits 12 seconds and is tried once more, and a question whose last call fails late
//! ends with what was read (`time_limit`), never with nothing.
//!
//! The limits are checked on the wait the app gives each call (the recorded client keeps it) and on
//! the reply, so no test waits for real. The real-time tests of the 20 and 25 second limits are
//! robustness's (`robustness_slow_line.rs`).
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
use mandi_assistant::ask::{AskOutcome, FailureKind};
use mandi_assistant::client::{CallError, RecordedClient};
use serde_json::{Value, json};

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_200, "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 7_000},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

/// A reply of the model that calls the tools.
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

/// The call that reads the buyers' dues (R20): a table the answer can fall back to.
fn read_dues() -> Value {
    let report = json!({
        "id": "R20", "adda": null, "as_of": null, "bikri": null, "city": null, "collector": null,
        "from": null, "garden": null, "group_by": null, "item": null, "partner": null,
        "party": null, "rows": null, "sort": null, "to": null, "trip": null, "where": null
    });
    tool_reply(&[("report", report)])
}

/// The call that finds the buyer; the app reads his card itself (plan 52 P52-S1).
fn find_akram() -> Value {
    tool_reply(&[("find", json!({"text": "akram", "role": "buyer"}))])
}

fn fail(kind: FailureKind) -> Result<Value, CallError> {
    Err(CallError::new(kind))
}

fn ask(client: &RecordedClient) -> (Harness, mandi_assistant::ask::AskReply) {
    let h = Harness::day();
    let reply = h.ask_with("buyers ka bakaya kitna hai", None, client);
    (h, reply)
}

fn step_ids(h: &Harness) -> Vec<String> {
    h.steps
        .lock()
        .unwrap()
        .iter()
        .map(|s| s.id.clone())
        .collect()
}

#[test]
fn every_request_asks_the_provider_for_the_fastest_azure_server() {
    let client = RecordedClient::with_results(vec![fail(FailureKind::BadKey)]);
    let _ = ask(&client);
    let provider = &client.requests()[0]["provider"];
    assert_eq!(provider["sort"], "latency");
    // The rest of the block is as it was: Azure only, no fallback, no retention, nothing collected.
    assert_eq!(provider["only"], json!(["azure"]));
    assert_eq!(provider["allow_fallbacks"], false);
    assert_eq!(provider["zdr"], true);
    assert_eq!(provider["data_collection"], "deny");
    assert_eq!(provider["require_parameters"], true);
}

#[test]
fn the_first_try_waits_twelve_seconds_and_the_one_retry_has_what_is_left() {
    let client =
        RecordedClient::with_results(vec![fail(FailureKind::Timeout), fail(FailureKind::Timeout)]);
    let (h, reply) = ask(&client);
    let waits = client.waits();
    assert_eq!(waits.len(), 2, "one try and one retry, no more");
    assert_eq!(waits[0], Duration::from_secs(12));
    // The retry has the rest of the 25 seconds: more than the first try's share is not possible,
    // and at least 20 seconds of it are left when the first try failed at once.
    assert!(waits[1] <= Duration::from_secs(25), "{waits:?}");
    assert!(waits[1] >= Duration::from_secs(20), "{waits:?}");
    // Nothing was read: it ends as the failure it is.
    assert_eq!(reply.outcome, AskOutcome::Error, "{reply:#?}");
    assert_eq!(reply.failure.map(|f| f.kind), Some(FailureKind::Timeout));
    assert!(reply.answer.is_none());
    assert!(step_ids(&h).contains(&"assistant-step-retrying".to_owned()));
}

#[test]
fn a_try_that_works_the_second_time_answers_as_if_it_had_worked_the_first() {
    let client = RecordedClient::with_results(vec![fail(FailureKind::Timeout), Ok(read_dues())]);
    let (_h, _reply) = ask(&client);
    // The retry's reply was used: the tool call it held was run and a third call was asked for.
    assert!(client.requests().len() >= 3, "{}", client.requests().len());
}

#[test]
fn a_last_call_that_times_out_after_the_books_were_read_shows_what_was_read() {
    let client = RecordedClient::with_results(vec![
        Ok(read_dues()),
        fail(FailureKind::Timeout),
        fail(FailureKind::Timeout),
    ]);
    let (h, reply) = ask(&client);
    assert_eq!(reply.outcome, AskOutcome::TimeLimit, "{reply:#?}");
    let answer = reply.answer.as_ref().expect("the tables that were read");
    assert!(!answer.tables.is_empty());
    assert!(answer.parts.is_empty(), "no word of the model's");
    assert!(reply.failure.is_none());
    // The log keeps the word, and the book of questions accepts it (migration 0018).
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    assert_eq!(rows[0].outcome, "time_limit");
    h.scan_requests();
}

#[test]
fn a_line_that_drops_after_the_books_were_read_shows_what_was_read_too() {
    for kind in [FailureKind::Dropped, FailureKind::Busy, FailureKind::Server] {
        let client = RecordedClient::with_results(vec![Ok(read_dues()), fail(kind), fail(kind)]);
        let (_h, reply) = ask(&client);
        assert_eq!(reply.outcome, AskOutcome::TimeLimit, "{kind:?}: {reply:#?}");
        assert!(reply.answer.is_some(), "{kind:?}");
    }
}

#[test]
fn a_wrong_key_or_no_internet_after_the_books_were_read_is_still_that_failure() {
    // Only a slow or dropped line is a matter of time: the other failures say what they are.
    for (kind, outcome) in [
        (FailureKind::BadKey, AskOutcome::Error),
        (FailureKind::NoCredit, AskOutcome::Error),
        (FailureKind::Offline, AskOutcome::Offline),
    ] {
        let client = RecordedClient::with_results(vec![Ok(read_dues()), fail(kind), fail(kind)]);
        let (_h, reply) = ask(&client);
        assert_eq!(reply.outcome, outcome, "{kind:?}: {reply:#?}");
        assert!(reply.answer.is_none(), "{kind:?}");
    }
}

#[test]
fn a_party_card_that_was_read_is_the_answer_when_the_last_call_fails() {
    // "ikram ke total bakaya kitne hai": the buyer is found, his card is read (2,880 بنام, checked),
    // and the last call hangs or drops. The one checked figure he asked for is never lost.
    for kind in [FailureKind::Timeout, FailureKind::Dropped] {
        let client = RecordedClient::with_results(vec![Ok(find_akram()), fail(kind), fail(kind)]);
        let h = Harness::day();
        let reply = h.ask_with("ikram ke total bakaya kitne hai", None, &client);
        assert_eq!(reply.outcome, AskOutcome::TimeLimit, "{kind:?}: {reply:#?}");
        let answer = reply.answer.as_ref().expect("the card");
        let name = answer
            .parts
            .iter()
            .find_map(|p| match p {
                mandi_assistant::answer::AnswerPart::Name { name } => Some(name.name_en.clone()),
                _ => None,
            })
            .expect("his name");
        assert_eq!(name, "Akram Toll Tax");
        let figures = harness::figures(answer);
        assert_eq!(harness::balance(figures[0].0).0, 2_880);
        assert!(figures[0].1, "his balance is checked (card = R13)");
        assert!(
            answer
                .sources
                .iter()
                .any(|s| matches!(s, mandi_assistant::answer::AnswerSource::PartyCard { .. }))
        );
    }
}

#[test]
fn the_tiles_that_were_read_show_as_their_source_and_a_find_alone_is_not_an_answer() {
    let tiles = tool_reply(&[("tiles", json!({}))]);
    let client = RecordedClient::with_results(vec![
        Ok(tiles),
        fail(FailureKind::Timeout),
        fail(FailureKind::Timeout),
    ]);
    let h = Harness::day();
    let reply = h.ask_with("cash kitna hai", None, &client);
    assert_eq!(reply.outcome, AskOutcome::TimeLimit, "{reply:#?}");
    assert!(reply.answer.as_ref().is_some_and(|a| {
        a.sources
            .iter()
            .any(|s| matches!(s, mandi_assistant::answer::AnswerSource::Tiles { .. }))
    }));
    // A name that was searched and not found read no figure: the failure is told as it is.
    let none = tool_reply(&[("find", json!({"text": "zzzparty", "role": "buyer"}))]);
    let client = RecordedClient::with_results(vec![
        Ok(none),
        fail(FailureKind::Timeout),
        fail(FailureKind::Timeout),
    ]);
    let h = Harness::day();
    let reply = h.ask_with("zzzparty ka bakaya", None, &client);
    assert_eq!(reply.outcome, AskOutcome::Error, "{reply:#?}");
    assert!(reply.answer.is_none());
}
