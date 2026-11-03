//! The time of a question is kept in the log (plan 51 P51-B2, migration 17): the whole run, the
//! time waiting for the model, the app's own time in the tools, and each model call with its tools.
//! The model is a recorded transcript that sleeps a fixed time on every call, so the numbers can be
//! checked. Counts, times and the fixed tool names only: no question, name or figure.
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

use std::sync::{Arc, Mutex};
use std::time::Duration;

use harness::{Harness, transcript};
use mandi_assistant::ask::{AskOutcome, AssistantAskInput, FailureKind, QuickChip};
use mandi_assistant::client::{CallError, ModelClient, RecordedClient, Reply};
use mandi_assistant::run;
use mandi_services::assistant;
use serde_json::Value;

/// A recorded model that takes `latency` on every call, or fails after it.
struct SlowClient {
    inner: RecordedClient,
    latency: Duration,
    fail: Option<FailureKind>,
}

impl ModelClient for SlowClient {
    fn call(&self, key: &str, body: &Value, timeout: Duration) -> Result<Reply, CallError> {
        std::thread::sleep(self.latency);
        if let Some(kind) = self.fail {
            return Err(CallError::new(kind));
        }
        self.inner.call(key, body, timeout)
    }
}

fn ask(h: &Harness, client: &SlowClient, text: &str) -> mandi_assistant::ask::AskReply {
    let steps = Arc::new(Mutex::new(Vec::new()));
    let sink = steps.clone();
    let emit = move |s: mandi_assistant::step::AssistantStep| sink.lock().unwrap().push(s);
    let deps = h.deps(client, &emit);
    run::ask(
        &deps,
        &AssistantAskInput {
            text: text.to_owned(),
            conversation_id: None,
        },
    )
    .unwrap()
}

fn last_row(h: &Harness) -> mandi_services::db::assistant::QuestionRow {
    let (rows, _, _) = assistant::list_questions(&h.world.env.ctx, 0, 100).unwrap();
    rows.into_iter().max_by_key(|r| r.id).unwrap()
}

#[test]
fn a_question_keeps_its_total_its_model_time_its_tool_time_and_each_call() {
    let h = Harness::day();
    let client = SlowClient {
        inner: RecordedClient::new(transcript(include_str!("fixtures/model/e2e-47.json"))),
        latency: Duration::from_millis(120),
        fail: None,
    };
    let reply = ask(&h, &client, "ikram ke total bakaya kitne hai");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let row = last_row(&h);
    let (total, model, tools) = (
        row.total_millis.unwrap(),
        row.model_millis.unwrap(),
        row.tool_millis.unwrap(),
    );
    // Three calls of at least 120 ms each; the tools are the app's few milliseconds.
    assert!(model >= 360, "model {model}");
    assert!(total >= model, "total {total} model {model}");
    assert!(
        total < model + 5_000,
        "the app's own time is small: total {total} model {model}"
    );
    assert!((0..2_000).contains(&tools), "tools {tools}");
    // The calls: 3, each at least 120 ms; the second asks for party_card and report.
    let calls: Vec<Value> = serde_json::from_str(row.calls_json.as_deref().unwrap()).unwrap();
    assert_eq!(calls.len(), 3);
    for (i, call) in calls.iter().enumerate() {
        assert_eq!(call["n"], i + 1);
        assert!(call["model_ms"].as_i64().unwrap() >= 120, "{call}");
        assert_eq!(call["failed"], false);
        assert_eq!(call["rewrite"], false);
        assert!(call["tokens_in"].as_i64().unwrap() > 0);
    }
    // The tools the model asked for; the ones the app read ahead are marked `prefetch` (P52-S1).
    let names = |call: &Value| -> Vec<String> {
        call["tools"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|t| t.get("prefetch").is_none())
            .map(|t| t["name"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(names(&calls[0]), ["find"]);
    // The find matched one buyer: the app read his card ahead, in the same call's tools.
    assert_eq!(calls[0]["tools"][1]["name"], "party_card");
    assert_eq!(calls[0]["tools"][1]["prefetch"], true);
    assert_eq!(names(&calls[1]), ["party_card", "report"]);
    assert!(names(&calls[2]).is_empty(), "the last call is the answer");
    // The sum of the calls is the model time of the row; the tools' sum is the tool time.
    let sum = |key: &str| -> i64 { calls.iter().map(|c| c[key].as_i64().unwrap()).sum() };
    assert_eq!(sum("model_ms"), model);
    let tool_sum: i64 = calls
        .iter()
        .flat_map(|c| c["tools"].as_array().unwrap().iter())
        .map(|t| t["ms"].as_i64().unwrap())
        .sum();
    assert_eq!(tool_sum, tools);
    // Counts, times and tool names only: nothing of the question or the books.
    let text = row.calls_json.unwrap();
    for private in ["ikram", "اکرم", "akram", "bakaya", "2880", "288000"] {
        assert!(!text.contains(private), "{private} in {text}");
    }
}

#[test]
fn a_call_that_fails_keeps_the_time_the_app_waited() {
    let h = Harness::day();
    let client = SlowClient {
        inner: RecordedClient::new(Vec::new()),
        latency: Duration::from_millis(150),
        fail: Some(FailureKind::Timeout),
    };
    let reply = ask(&h, &client, "ikram ke total bakaya kitne hai");
    assert_eq!(reply.outcome, AskOutcome::Error);
    let row = last_row(&h);
    assert!(row.total_millis.unwrap() >= 150);
    assert!(row.model_millis.unwrap() >= 150);
    let calls: Vec<Value> = serde_json::from_str(row.calls_json.as_deref().unwrap()).unwrap();
    // A timeout is tried once more when 20 seconds are left (Q237 and Q313 of the live run of
    // 03-11-2026): both tries are in the log, each with the time the app waited.
    assert_eq!(calls.len(), 2);
    for call in &calls {
        assert_eq!(call["failed"], true);
        assert!(call["model_ms"].as_i64().unwrap() >= 150);
        assert_eq!(call["tokens_in"], 0);
    }
}

#[test]
fn a_chip_keeps_only_its_total() {
    let h = Harness::day();
    let reply = h.chip(QuickChip::CashInHand);
    assert_eq!(reply.outcome, AskOutcome::Answered);
    let row = last_row(&h);
    assert!(row.total_millis.is_some());
    assert_eq!(row.model_millis, None);
    assert_eq!(row.tool_millis, None);
    assert_eq!(row.calls_json, None);
}
