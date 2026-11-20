//! Plan 82, NFR-SEC-9 v1.22: the warm-up call. One small request with only the fixed part of a
//! request, sent when the panel opens, so the connection and the provider's cache are ready.
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

use std::time::Instant;

use harness::Harness;
use mandi_assistant::client::RecordedClient;
use mandi_assistant::step::AssistantStep;
use mandi_assistant::warm::{
    WARM_MAX_OUTPUT_TOKENS, WARM_PER_DAY, WARM_TEXT, WarmGate, WarmOutcome, warm,
};
use mandi_services::assistant;
use serde_json::{Value, json};

fn stop_reply() -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "length", "message": {"role": "assistant", "content": ""}}],
        "usage": {"prompt_tokens": 12_000, "completion_tokens": 16,
                  "prompt_tokens_details": {"cached_tokens": 0},
                  "completion_tokens_details": {"reasoning_tokens": 0}},
    })
}

fn answer_reply(text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": {"prompt_tokens": 7_200, "completion_tokens": 60,
                  "prompt_tokens_details": {"cached_tokens": 7_000},
                  "completion_tokens_details": {"reasoning_tokens": 20}},
    })
}

fn warm_with(h: &Harness, client: &RecordedClient) -> WarmOutcome {
    let emit = |_s: AssistantStep| {};
    let deps = h.deps(client, &emit);
    warm(&deps)
}

#[test]
fn the_warm_up_sends_one_small_request_with_only_the_fixed_part() {
    let h = Harness::day();
    let client = RecordedClient::new(vec![stop_reply()]);
    assert_eq!(warm_with(&h, &client), WarmOutcome::Sent);
    let requests = client.requests();
    assert_eq!(requests.len(), 1);
    let body = &requests[0];
    // Small: at most 16 tokens, and the tools are switched off so the model can only stop.
    assert_eq!(body["max_completion_tokens"], WARM_MAX_OUTPUT_TOKENS);
    assert_eq!(body["tool_choice"], "none");
    // The same provider block as every request: Azure only, zero retention, no fallback.
    assert_eq!(body["provider"], mandi_assistant::request::provider());
    // One system message and ONE neutral user message: no question, no conversation, no books data.
    let messages = body["messages"].as_array().expect("messages");
    assert_eq!(messages.len(), 2, "{messages:#?}");
    assert_eq!(messages[0]["role"], "system");
    assert_eq!(messages[1]["role"], "user");
    let user: Value =
        serde_json::from_str(messages[1]["content"].as_str().expect("text")).expect("json");
    assert_eq!(user["question"], WARM_TEXT);
    assert_eq!(user["question_numbers"], json!({}));
    // Nothing of the books: the user message is the whole of what is not the fixed part, and it is
    // today's date, the language and the neutral line. (The fixed instructions themselves name
    // example parties, so the names of the worked day are not a test of absence.)
    assert_eq!(
        user.as_object()
            .map(|o| o.keys().cloned().collect::<Vec<_>>().len()),
        Some(4)
    );
    let text = messages[1].to_string();
    for word in ["Rahim", "Karim", "0300", "رحیم", "کریم"] {
        assert!(!text.contains(word), "{word} is in the warm-up message");
    }
    h.seen.lock().unwrap().extend(requests);
    h.scan_requests();
}

#[test]
fn the_fixed_part_is_the_same_as_the_one_of_a_question_so_the_cache_serves_both() {
    let h = Harness::day();
    let client = RecordedClient::new(vec![stop_reply(), answer_reply("جی")]);
    assert_eq!(warm_with(&h, &client), WarmOutcome::Sent);
    let _ = h.ask_with("kya haal hai", None, &client);
    let requests = client.requests();
    assert_eq!(requests.len(), 2);
    for key in ["tools", "response_format", "provider", "model", "seed"] {
        assert_eq!(requests[0][key], requests[1][key], "{key}");
    }
    assert_eq!(requests[0]["messages"][0], requests[1]["messages"][0]);
}

#[test]
fn it_is_no_question_no_log_row_and_no_count() {
    let h = Harness::day();
    let before = assistant::count_today(&h.world.env.ctx).expect("count");
    let client = RecordedClient::new(vec![stop_reply()]);
    assert_eq!(warm_with(&h, &client), WarmOutcome::Sent);
    assert_eq!(
        assistant::count_today(&h.world.env.ctx).expect("count"),
        before
    );
    let (rows, _, _) = assistant::list_questions(&h.world.env.ctx, 0, 10).expect("log");
    assert!(rows.is_empty(), "no log row");
    assert!(h.steps.lock().unwrap().is_empty(), "no step line");
}

#[test]
fn once_in_five_minutes_and_not_when_the_assistant_is_off_or_busy() {
    // Twice in a row: the second goes nowhere.
    let h = Harness::day();
    let client = RecordedClient::new(vec![stop_reply(), stop_reply()]);
    assert_eq!(warm_with(&h, &client), WarmOutcome::Sent);
    assert_eq!(warm_with(&h, &client), WarmOutcome::TooSoon);
    assert_eq!(client.requests().len(), 1);
    // Off: nothing is sent.
    let h = Harness::day();
    assistant::set_config(
        &h.world.env.ctx,
        &h.world.owner,
        false,
        "openai/gpt-6-luna",
        true,
    )
    .expect("switch off");
    let client = RecordedClient::new(vec![stop_reply()]);
    assert_eq!(warm_with(&h, &client), WarmOutcome::Off);
    assert!(client.requests().is_empty());
    // No key: nothing is sent.
    let h = Harness::day();
    h.runtime.keys().delete().expect("remove the key");
    let client = RecordedClient::new(vec![stop_reply()]);
    assert_eq!(warm_with(&h, &client), WarmOutcome::Off);
    assert!(client.requests().is_empty());
    // A question is running: its connection is warm, so nothing is sent.
    let h = Harness::day();
    let guard = h.runtime.try_begin().expect("the place");
    let client = RecordedClient::new(vec![stop_reply()]);
    assert_eq!(warm_with(&h, &client), WarmOutcome::Busy);
    assert!(client.requests().is_empty());
    drop(guard);
}

#[test]
fn a_failed_warm_up_is_no_failure_and_still_counts_against_the_gap() {
    let h = Harness::day();
    let client = RecordedClient::with_results(vec![Err(mandi_assistant::client::CallError::new(
        mandi_assistant::ask::FailureKind::Timeout,
    ))]);
    assert_eq!(warm_with(&h, &client), WarmOutcome::Failed);
    // The line is bad: the next try waits for the gap, so it is not hammered.
    assert_eq!(warm_with(&h, &client), WarmOutcome::TooSoon);
}

#[test]
fn the_day_holds_twelve_and_the_next_day_starts_again() {
    let mut gate = WarmGate::default();
    let t = Instant::now();
    let mut sent = 0;
    for n in 0..40_u64 {
        let at = t + std::time::Duration::from_secs(n * 301);
        if gate.allow(at, "2026-11-20").is_ok() {
            sent += 1;
        }
    }
    assert_eq!(sent, WARM_PER_DAY);
}
