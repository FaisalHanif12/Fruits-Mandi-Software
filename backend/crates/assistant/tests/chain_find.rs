//! Find and read in one reply (plan 52 item 8, PRD FR-7.87, Umar 02-11-2026): the model may send a
//! `find` and the tool that reads the party it found in the same reply. The app reads that party
//! only when the find had exactly one match; with none or several it ends the question at once
//! with "not found" or "کون سا؟", and no model call is spent on it. One round trip less for every
//! named-party question.
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

use harness::{Harness, answer_of, balance, figures};
use mandi_assistant::ask::{AskOutcome, PickChoice};
use mandi_services::core::accounts::BalanceSide;
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

fn final_reply(kind: &str, text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": kind, "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

fn card_after_find(text: &str) -> Value {
    tool_reply(&[
        ("find", json!({"text": text, "role": "buyer"})),
        ("party_card", json!({"party": "r1.matches.0"})),
    ])
}

#[test]
fn one_match_is_read_in_the_same_reply_so_the_question_takes_two_calls_not_three() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "ikram ke total bakaya kitne hai",
        None,
        vec![
            card_after_find("akram"),
            final_reply(
                "answer",
                "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔",
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 2, "the find and its read in one reply");
    assert_eq!(
        balance(figures(answer_of(&reply))[0].0),
        (2_880, BalanceSide::Dr)
    );
    h.scan_requests();
}

#[test]
fn several_matches_end_the_question_with_which_one_and_no_second_model_call() {
    let h = Harness::day();
    // "a" is in Akram and Arshad: several buyers.
    let (reply, requests) = h.ask(
        "a ka khata dikhao",
        None,
        vec![card_after_find("a"), final_reply("answer", "غلط")],
    );
    assert_eq!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
    assert_eq!(
        requests.len(),
        1,
        "the app asks, the model is not called again"
    );
    assert!(reply.options.len() >= 2);
    // The read was not run: the tool message of the party card says why.
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    let log: Vec<Value> = serde_json::from_str(&rows[0].tool_calls_json).unwrap();
    assert_eq!(log.len(), 1, "only the find ran: {log:?}");
    assert_eq!(log[0]["tool"], "find");
    // The Owner picks one: the question goes on from his choice, with one more model call.
    let key = reply.options[0].key.clone();
    let (picked, requests) = h.pick(
        reply.question_id,
        PickChoice::Match { key },
        vec![
            tool_reply(&[("party_card", json!({"party": "r1.matches.0"}))]),
            final_reply(
                "answer",
                "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔",
            ),
        ],
    );
    assert_eq!(picked.outcome, AskOutcome::Answered, "{picked:#?}");
    // The messages the model gets stay valid: every tool call has its tool message.
    let first = requests[0]["messages"].as_array().unwrap();
    let calls = first
        .iter()
        .filter_map(|m| m["tool_calls"].as_array())
        .map(Vec::len)
        .sum::<usize>();
    let answers = first.iter().filter(|m| m["role"] == "tool").count();
    assert_eq!(calls, answers, "{first:?}");
    h.scan_requests();
}

#[test]
fn no_match_ends_the_question_with_not_found() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "zzzparty ka khata",
        None,
        vec![card_after_find("zzzparty"), final_reply("answer", "غلط")],
    );
    assert_eq!(reply.outcome, AskOutcome::NotFound, "{reply:#?}");
    assert_eq!(requests.len(), 1);
    assert!(reply.options.is_empty());
}

#[test]
fn a_find_from_an_earlier_reply_is_not_guarded_the_model_has_read_it() {
    // Round one: the find alone, with several matches. The model chooses to read the first itself
    // in round two (it saw the result): the app does not step in.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "a ka khata dikhao",
        None,
        vec![
            tool_reply(&[("find", json!({"text": "a", "role": "buyer"}))]),
            tool_reply(&[("party_card", json!({"party": "r1.matches.0"}))]),
            final_reply(
                "answer",
                "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔",
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 3);
}
