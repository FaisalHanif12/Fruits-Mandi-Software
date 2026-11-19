//! The quality (grade) of a sale line in the scope of an answer, on the rich test year, and the
//! reads the app makes by itself for two multi-period questions (live run 2, the lead's list).
//!
//! 1. Q1700 "اول کیلا کی sale آج": the first quality of the banana of the day is 75,980 rupees; the
//!    whole banana of the day is 183,640. The app plans view A with `grade: first` for today. A
//!    model that reads view A again with no quality and cites that is refused (`scope`) and the
//!    shown figure is 75,980, never 183,640.
//! 2. "how did onion sales go in August, September and October" and "is potato selling more this
//!    month than last month" are understood for sure, so their reads are made before the first call
//!    (H348 and H371 ended in `time_limit` at 25 s with the model reading by itself).
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
#[path = "robustness_year/mod.rs"]
mod year;

use harness::{Harness, answer_of, figures, money};
use mandi_assistant::ask::AskOutcome;
use serde_json::{Value, json};

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_200, "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 7_000},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

fn tool_reply(name: &str, args: &Value) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null,
            "tool_calls": [{"id": "call_0", "type": "function",
                            "function": {"name": name, "arguments": args.to_string()}}]}}],
        "usage": usage(),
    })
}

fn answer(text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

/// The ids and tools the model was shown as already read in a request.
fn read_before(request: &Value) -> Vec<(String, String)> {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .filter_map(|v| Some((v["id"].as_str()?.to_owned(), v["tool"].as_str()?.to_owned())))
        .collect()
}

/// The words of the `check_failed` messages the model was sent.
fn told(request: &Value) -> Vec<Value> {
    let mut out = Vec::new();
    for m in request["messages"].as_array().unwrap() {
        if m["role"] != "user" {
            continue;
        }
        if let Some(v) = m["content"]
            .as_str()
            .and_then(|t| serde_json::from_str::<Value>(t).ok())
        {
            out.extend(v["check_failed"].as_array().cloned().unwrap_or_default());
        }
    }
    out
}

fn year_harness() -> Harness {
    let y = year::seed::build();
    let h = Harness::over(y.w);
    // The app's own reads on, its own answers off, so the question reaches the model.
    h.runtime.set_read_first(true);
    h.runtime.set_local(false);
    h.runtime.set_instant(false);
    h.runtime.set_low_effort(true);
    h
}

#[test]
fn q1700_the_first_quality_of_the_banana_of_the_day_is_75980_and_the_whole_item_is_refused() {
    let h = year_harness();
    let question = "اول کیلا کی sale آج";
    // What the app planned and read before the first call: view A with the quality.
    let (_, probe) = h.ask(question, None, vec![answer("")]);
    let planned: Vec<String> = read_before(&probe[0])
        .into_iter()
        .filter(|(_, tool)| tool == "analytics")
        .map(|(id, _)| id)
        .collect();
    assert_eq!(planned.len(), 1, "one view A was planned");
    let planned_id = planned[0].clone();

    // The model reads view A for the same item and day with NO quality, and cites it.
    let own = json!({
        "view": "A", "from": "r1.from", "to": "r1.to", "item": "r2.matches.0", "grade": null,
        "adda": null, "buyer": null, "city": null, "compare": null, "garden": null,
        "goods": null, "group_by": null, "item_group": null, "partner": null, "rows": null,
        "seller": null, "side": null, "sort": null, "trip": null
    });
    let h = year_harness();
    let (reply, requests) = h.ask(
        question,
        None,
        vec![
            tool_reply("analytics", &own),
            answer("آج {{r2.matches.0.name}} کی خام بکری {{r4.totals.gross}} رہی۔"),
            answer(&format!(
                "آج {{{{r2.matches.0.name}}}} کی خام بکری {{{{{planned_id}.totals.gross}}}} رہی۔"
            )),
        ],
    );
    let told_all: Vec<Value> = requests.iter().flat_map(told).collect();
    assert!(
        told_all.iter().any(|f| f["rule"] == "scope"),
        "the read with no quality was refused: {told_all:?}"
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let shown: Vec<i64> = figures(answer_of(&reply))
        .iter()
        .map(|(cell, _)| money(cell))
        .collect();
    assert_eq!(shown, [75_980], "the first quality only, not 183,640");
}

#[test]
fn the_months_list_and_the_two_months_compare_are_read_before_the_first_call() {
    // H348 and H371: the app understood neither for sure, so it read nothing and the model read by
    // itself, one tool a call, until the 25 s limit. Now the reads are in before its first call.
    let h = year_harness();
    for (question, views) in [
        ("how did onion sales go in August, September and October", 3),
        ("is potato selling more this month than last month", 2),
    ] {
        let (_, requests) = h.ask(question, None, vec![answer("")]);
        let read = read_before(&requests[0]);
        let views_read = read.iter().filter(|(_, tool)| tool == "analytics").count();
        assert_eq!(views_read, views, "{question}: {read:?}");
        assert!(
            read.iter().any(|(_, tool)| tool == "find"),
            "{question}: the item was found first: {read:?}"
        );
    }
}
