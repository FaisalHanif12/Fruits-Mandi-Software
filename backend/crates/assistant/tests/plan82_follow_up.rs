//! Plan 82, the live check of 21-11-2026 (D26b, Y21b, Y31b): a follow-up that changes only the subject is the
//! earlier question with that subject.
//!
//! "rahim ki is saal ki bikri kitni hai ?" then "aur ehsan ki ?" is "ehsan ki is saal ki bikri kitni hai ?":
//! the app answers it by itself, with the period of the question before, and no model call is made. Before, the
//! model read the party and then cited a result of the earlier question (refused), or read a head with no dates.
//!
//! The books are the client's demo of D8 (today 08-11-2026; `support/worked.rs`): Rahim and Ehsan are sellers,
//! Mani is a buyer.
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

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use harness::{Harness, answer_of, figures, money};
use mandi_assistant::ask::{AskOutcome, AskReply, AssistantAskInput, FailureKind};
use mandi_assistant::client::{CallError, ModelClient, Reply};
use mandi_assistant::run::ask;
use mandi_assistant::step::AssistantStep;
use serde_json::Value;

/// A model that must not be called: every call is counted and fails.
#[derive(Default)]
struct NoModel {
    calls: AtomicUsize,
}

impl ModelClient for NoModel {
    fn retry_wait(&self) -> Duration {
        Duration::ZERO
    }

    fn call(&self, _key: &str, _body: &Value, _timeout: Duration) -> Result<Reply, CallError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(CallError::new(FailureKind::Other))
    }
}

fn production() -> Harness {
    let h = Harness::over(worked::d8().w);
    h.runtime.set_local(true);
    h.runtime.set_read_first(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

fn ask_in(h: &Harness, client: &NoModel, question: &str, conversation: Option<&str>) -> AskReply {
    let emit = |_s: AssistantStep| {};
    let deps = h.deps(client, &emit);
    ask(
        &deps,
        &AssistantAskInput {
            text: question.to_owned(),
            conversation_id: conversation.map(str::to_owned),
        },
    )
    .expect("ask")
}

fn shown(reply: &AskReply) -> Vec<i64> {
    figures(answer_of(reply))
        .into_iter()
        .map(|(cell, _)| money(cell))
        .collect()
}

#[test]
fn aur_a_name_is_the_question_before_for_that_name() {
    let h = production();
    // What the direct question gives.
    let direct = NoModel::default();
    let want = shown(&ask_in(
        &h,
        &direct,
        "ehsan ki is saal ki bikri kitni hai",
        None,
    ));
    assert!(!want.is_empty());
    let client = NoModel::default();
    let first = ask_in(&h, &client, "rahim ki is saal ki bikri kitni hai", None);
    assert_eq!(first.outcome, AskOutcome::Answered, "{first:#?}");
    let second = ask_in(&h, &client, "aur ehsan ki ?", Some(&first.conversation_id));
    assert_eq!(second.outcome, AskOutcome::Answered, "{second:#?}");
    assert_eq!(shown(&second), want, "{second:#?}");
    assert_eq!(client.calls.load(Ordering::SeqCst), 0, "no model call");
    // A third follow-up goes on from the second: the subject is replaced again, the period stays.
    let third = ask_in(&h, &client, "aur rahim ki ?", Some(&first.conversation_id));
    assert_eq!(shown(&third), shown(&first), "{third:#?}");
    assert_eq!(client.calls.load(Ordering::SeqCst), 0, "no model call");
}

#[test]
fn a_follow_up_with_its_own_period_is_not_rewritten() {
    let h = production();
    let client = NoModel::default();
    let first = ask_in(&h, &client, "rahim ki is saal ki bikri kitni hai", None);
    // It names a period of its own: it is its own question, answered by the app with that period.
    let own = ask_in(
        &h,
        &client,
        "ehsan ki aaj ki bikri",
        Some(&first.conversation_id),
    );
    let direct = ask_in(&h, &NoModel::default(), "ehsan ki aaj ki bikri", None);
    assert_eq!(shown(&own), shown(&direct), "{own:#?}");
}

/// A `report` call with every field of the schema.
fn report_call(id: &str) -> Value {
    serde_json::json!({
        "id": id, "adda": null, "as_of": null, "bikri": null, "city": null, "collector": null,
        "from": null, "garden": null, "group_by": null, "item": null, "partner": null,
        "party": null, "rows": null, "sort": null, "to": null, "trip": null, "where": null,
        "seller": null, "zamindar": null, "status": null, "voucher_type": null, "layout": null,
        "item_group": null
    })
}

fn tool_reply(name: &str, args: &Value) -> Value {
    serde_json::json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null, "tool_calls": [{"id": "call_0", "type": "function",
                "function": {"name": name, "arguments": args.to_string()}}]}}],
        "usage": {"prompt_tokens": 7200, "completion_tokens": 60,
            "prompt_tokens_details": {"cached_tokens": 7000},
            "completion_tokens_details": {"reasoning_tokens": 20}},
    })
}

#[test]
fn a_read_of_a_follow_up_with_no_dates_is_for_the_period_of_the_question_before() {
    // Y31b: "and on tea?" read the expense report with no dates: all the books, not this year.
    let h = production();
    let first = h
        .ask("rahim ki is saal ki bikri kitni hai", None, Vec::new())
        .0;
    assert_eq!(first.outcome, AskOutcome::Answered, "{first:#?}");
    let (_, requests) = h.ask(
        "kuch aur batao ?",
        Some(&first.conversation_id),
        vec![
            tool_reply("report", &report_call("R2")),
            tool_reply("report", &report_call("R2")),
        ],
    );
    let said: String = requests.iter().skip(1).map(Value::to_string).collect();
    assert!(
        said.contains("names no period") && said.contains("2026-01-01 to 2026-11-08"),
        "{said}"
    );
}

#[test]
fn an_expense_head_found_by_find_is_the_head_filter_of_r29() {
    // Y04: the model passed the head it had found as `party`: refused as another kind of record, six calls
    // and an empty apology. The head's own row is picked, and its expense is `matched`.
    let h = Harness::day();
    let find = serde_json::json!({"role": "expense_head", "text": "electricity"});
    let mut with_head = report_call("R29");
    with_head["party"] = serde_json::json!("r1.matches.0");
    let (_, requests) = h.ask(
        "How much did the shop spend on electricity?",
        None,
        vec![
            tool_reply("find", &find),
            tool_reply("report", &with_head),
            tool_reply("report", &report_call("R29")),
        ],
    );
    let tools: Vec<String> = requests
        .iter()
        .skip(2)
        .flat_map(|r| r["messages"].as_array().cloned().unwrap_or_default())
        .filter(|m| m["role"] == "tool")
        .map(|m| m["content"].to_string())
        .collect();
    let head_read = tools
        .iter()
        .find(|t| t.contains("\\\"source\\\":\\\"R29\\\""))
        .expect("the report was read");
    assert!(!head_read.contains("ERR_"), "{head_read}");
    assert!(head_read.contains("matched"), "{head_read}");
    // Only the head's row: Electricity 3,000, and not the partner rows.
    assert!(head_read.contains("Electricity"), "{head_read}");
    assert!(!head_read.contains("Partner A"), "{head_read}");
}

#[test]
fn the_gross_beside_the_bills_has_view_a_read_before_the_model_answers() {
    // M18: "khaam bikri aur gahakon ke kul bill alag alag" read R2 only; R2 holds no gross, so the guard refused
    // the answer. View A is read ahead for the same days.
    let h = production();
    let (_, requests) = h.ask(
        "is mahine ki khaam bikri aur gahakon ke kul bill alag alag batao",
        None,
        vec![serde_json::json!({
            "model": "openai/gpt-6-luna",
            "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": "{}"}}],
            "usage": {"prompt_tokens": 7200, "completion_tokens": 60},
        })],
    );
    let first = requests
        .first()
        .expect("a request")
        .to_string()
        .replace("\\\"", "\"");
    assert!(first.contains("\"view\":\"A\""), "{first}");
    assert!(first.contains("\"id\":\"R2\""), "{first}");
}

#[test]
fn probe_m() {
    let h = Harness::day();
    let args = serde_json::json!({
        "view": "M", "from": "2026-09-01", "to": "2026-10-31", "group_by": "month", "item": null, "item_group": null,
        "seller": null, "buyer": null, "city": null, "adda": null, "garden": null, "trip": null, "goods": null,
        "partner": null, "side": null, "compare": null, "sort": null, "rows": null, "grade": null
    });
    let (_, requests) = h.ask("expenses", None, vec![tool_reply("analytics", &args)]);
    for r in requests.iter().skip(1) {
        for m in r["messages"].as_array().unwrap().iter().filter(|m| m["role"] == "tool") {
            eprintln!("TOOLM {}", m["content"].to_string().chars().take(2500).collect::<String>());
        }
    }
}
