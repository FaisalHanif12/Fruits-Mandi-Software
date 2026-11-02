//! When a limit stops a question (plan 51 P51-B2, fix 3). Umar's "apni sale ko ziyada kaise
//! karain" used its 4 calls on tool refusals and ended as "not understood", which told him to ask
//! again in other words although the problem was not his words. Now the reply says a limit stopped
//! it (`stop_reason`), the log keeps the reason in `finish_reason`, and the tables the question
//! did read are shown as they are.
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

use harness::Harness;
use mandi_assistant::ask::{AskOutcome, StopReason};
use mandi_services::assistant;
use serde_json::{Value, json};

fn usage(prompt: i64, completion: i64) -> Value {
    json!({
        "prompt_tokens": prompt, "completion_tokens": completion,
        "prompt_tokens_details": {"cached_tokens": 0},
        "completion_tokens_details": {"reasoning_tokens": 10}
    })
}

fn tool_reply(calls: &[(&str, Value)], tokens: (i64, i64)) -> Value {
    let tool_calls: Vec<Value> = calls
        .iter()
        .enumerate()
        .map(|(i, (name, args))| {
            json!({"id": format!("call_{i}"), "type": "function",
                   "function": {"name": name, "arguments": args.to_string()}})
        })
        .collect();
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls",
                     "message": {"role": "assistant", "content": null, "tool_calls": tool_calls}}],
        "usage": usage(tokens.0, tokens.1),
    })
}

fn r20() -> (&'static str, Value) {
    (
        "report",
        json!({"id": "R20", "from": null, "to": null, "as_of": null, "party": null, "item": null,
               "city": null, "adda": null, "collector": null, "partner": null, "trip": null,
               "garden": null, "bikri": null, "group_by": null, "where": null, "sort": null,
               "rows": null}),
    )
}

fn bad_period() -> (&'static str, Value) {
    // A period call the tool refuses: `n` needs a question number.
    (
        "period",
        json!({"kind": "last_n_days", "month": null, "day": null, "from": null, "to": null, "n": 7}),
    )
}

fn last_log_row(h: &Harness) -> mandi_services::db::assistant::QuestionRow {
    let (rows, _, _) = assistant::list_questions(&h.world.env.ctx, 0, 100).unwrap();
    rows.into_iter().max_by_key(|r| r.id).unwrap()
}

#[test]
fn four_calls_of_refusals_end_with_the_call_limit_and_nothing_to_show() {
    let h = Harness::day();
    let replies = vec![
        tool_reply(&[bad_period()], (7_000, 60)),
        tool_reply(&[bad_period()], (7_100, 60)),
        tool_reply(&[bad_period()], (7_200, 60)),
        tool_reply(&[bad_period()], (7_300, 60)),
    ];
    let (reply, requests) = h.ask("pichle 7 din ki sale", None, replies);
    assert_eq!(requests.len(), 4);
    assert_eq!(reply.outcome, AskOutcome::NotUnderstood, "{reply:#?}");
    assert_eq!(reply.stop_reason, Some(StopReason::Calls));
    assert!(
        reply.answer.is_none(),
        "nothing was read, so nothing is shown"
    );
    let row = last_log_row(&h);
    assert_eq!(row.outcome, "not_understood");
    assert_eq!(row.finish_reason.as_deref(), Some("call_limit"));
    assert_eq!(row.model_calls, Some(4));
}

#[test]
fn a_table_read_before_the_limit_is_shown_as_it_is() {
    let h = Harness::day();
    let replies = vec![
        // Call 1 reads R20 (a real table), then three calls of refusals use up the rest.
        tool_reply(&[r20()], (7_000, 60)),
        tool_reply(&[bad_period()], (7_100, 60)),
        tool_reply(&[bad_period()], (7_200, 60)),
        tool_reply(&[bad_period()], (7_300, 60)),
    ];
    let (reply, _) = h.ask("kis kis ke zimme kitna hai, pichle 7 din", None, replies);
    assert_eq!(reply.outcome, AskOutcome::NotUnderstood, "{reply:#?}");
    assert_eq!(reply.stop_reason, Some(StopReason::Calls));
    let answer = reply.answer.as_ref().expect("the table that was read");
    assert_eq!(answer.tables.len(), 1);
    assert_eq!(answer.tables[0].count, 2, "the two buyers with dues");
    assert!(
        answer.parts.is_empty(),
        "the model wrote nothing, so no text is invented"
    );
    assert!(
        !answer.sources.is_empty(),
        "the button that opens the report"
    );
}

#[test]
fn the_token_limit_ends_with_its_own_reason() {
    let h = Harness::day();
    // The first call already used the question's whole input budget (60,000).
    let replies = vec![tool_reply(&[r20()], (60_000, 60))];
    let (reply, requests) = h.ask("kis kis ke zimme kitna hai", None, replies);
    assert_eq!(requests.len(), 1);
    assert_eq!(reply.outcome, AskOutcome::NotUnderstood, "{reply:#?}");
    assert_eq!(reply.stop_reason, Some(StopReason::Tokens));
    assert_eq!(
        last_log_row(&h).finish_reason.as_deref(),
        Some("token_limit")
    );
    assert!(reply.answer.is_some(), "R20 was read");
}

#[test]
fn a_model_that_says_it_cannot_tell_what_was_asked_has_no_stop_reason() {
    let h = Harness::day();
    let reply = json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content":
            json!({"kind": "not_understood", "text": "", "show": [], "next": [], "choose": null,
                   "entry": null, "nearest": null}).to_string()}}],
        "usage": usage(7_000, 40),
    });
    let (reply, _) = h.ask("xyz abc", None, vec![reply]);
    assert_eq!(reply.outcome, AskOutcome::NotUnderstood);
    assert_eq!(
        reply.stop_reason, None,
        "this one really was not understood"
    );
}
