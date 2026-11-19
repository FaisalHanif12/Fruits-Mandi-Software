//! Offline, an error from OpenRouter, a timeout (PRD FR-7.81, E89, NFR-AI-3, NFR-AI-7).
//!
//! `robustness_assistant.rs` attack 6 runs all 12 failure kinds on a first call, a failure in the
//! middle of a question, the 12 and 25 second limits and the retry. This file adds: a failure in
//! the call that rewrites an answer, a failure when a "which one" question goes on, the questions
//! the app answers itself when the model cannot be reached, the reason an error reply carries
//! (it must hold no key), and the quick questions after each ending.
#![allow(dead_code)]

use mandi_assistant::ask::{
    AskOutcome, AskReply, AssistantAskInput, FailureKind, PickChoice, QuickChip,
};
use mandi_assistant::client::{CallError, ModelClient, RecordedClient};
use mandi_assistant::provider_reason::reason_of;
use mandi_assistant::run;
use mandi_services::core::accounts::AccountType;
use serde_json::{Value, json};

use crate::harness::{Harness, TEST_KEY, answer_of, figures, money};
use crate::kit::*;
use crate::world;

fn ask_client(h: &Harness, text: &str, client: &dyn ModelClient) -> AskReply {
    let before = row_counts(&h.world.dir);
    let steps = h.steps.clone();
    let emit = move |s| steps.lock().unwrap().push(s);
    let deps = h.deps(client, &emit);
    let reply = run::ask(
        &deps,
        &AssistantAskInput {
            text: text.to_owned(),
            conversation_id: None,
        },
    )
    .unwrap();
    assert_only_log_grew(&before, &row_counts(&h.world.dir), 1);
    reply
}

fn chips_work(h: &Harness) {
    let chip = h.chip(QuickChip::CashInHand);
    assert_eq!(money(figures(answer_of(&chip))[0].0), 58_484);
    assert!(h.runtime.try_begin().is_some(), "the assistant is busy");
}

const FAILURES: [(FailureKind, Option<i64>); 8] = [
    (FailureKind::Offline, None),
    (FailureKind::Timeout, None),
    (FailureKind::Dropped, None),
    (FailureKind::BadKey, Some(401)),
    (FailureKind::NoCredit, Some(402)),
    (FailureKind::NoEndpoint, Some(404)),
    (FailureKind::Busy, Some(429)),
    (FailureKind::Server, Some(503)),
];

fn error(kind: FailureKind, status: Option<i64>) -> CallError {
    CallError {
        kind,
        http_status: status,
        reason: None,
    }
}

/// The model answers once with a bad answer, and the call that rewrites it fails. The Owner must
/// not lose what was read: the tables of the reads are shown, nothing the model typed is, the
/// question is counted once (the model did reply) and everything else is free.
#[test]
fn t5_a_failure_in_the_rewrite_call_shows_the_tables_and_nothing_of_the_model() {
    let h = Harness::day();
    let typed = answer("answer", "{{r1.matches.0.name}} کے ذمے 2,800 باقی ہیں۔");
    for (kind, status) in FAILURES {
        let error = error(kind, status);
        let tries = if error.retryable() { 2 } else { 1 };
        let mut results = vec![
            Ok(tool_reply(&[find("ikram", "buyer")])),
            Ok(tool_reply(&[card(AKRAM)])),
            Ok(final_reply(&typed)),
        ];
        results.extend(std::iter::repeat_n(Err(error), tries));
        let client = RecordedClient::with_results(results);
        let before = h.runtime.try_begin().is_some();
        assert!(before);
        let reply = ask_client(&h, "ikram ke total bakaya kitne hai", &client);
        // The typed figure never reaches the screen, whatever the ending.
        let text = serde_json::to_string(&reply).unwrap();
        assert!(
            !text.contains("2,800"),
            "{kind:?}: the model's figure was shown"
        );
        assert!(
            matches!(
                reply.outcome,
                AskOutcome::ChecksFailed
                    | AskOutcome::Error
                    | AskOutcome::Offline
                    | AskOutcome::TimeLimit
            ),
            "{kind:?}: {:?}",
            reply.outcome
        );
        chips_work(&h);
    }
}

fn two_akrams() -> Harness {
    let h = Harness::day();
    world::add_account_tagged(
        &h.world.env,
        &h.world.owner,
        AccountType::Buyer,
        ("اکرم ٹول ٹیکس", "Akram Toll Tax"),
        (true, ""),
        "B",
    );
    h
}

/// "کون سا؟" is waiting, the Owner clicks, and the model cannot be reached: the click ends in a
/// reply that says so, with the kind of failure, and loses nothing else.
#[test]
fn t5_a_click_on_which_one_with_no_line_is_a_reply_with_its_failure() {
    for (kind, status) in FAILURES {
        let h = two_akrams();
        let which = answer_full(
            "which_one",
            "",
            json!([]),
            json!([]),
            json!("r1"),
            Value::Null,
        );
        let (asked, _) = ask(
            &h,
            "akram ka bakaya",
            None,
            vec![tool_reply(&[find("akram", "buyer")]), final_reply(&which)],
        );
        assert_eq!(asked.outcome, AskOutcome::WhichOne);
        let key = asked.options[0].key.clone();
        let error = error(kind, status);
        let tries = if error.retryable() { 2 } else { 1 };
        let client = RecordedClient::with_results(vec![Err(error); tries]);
        let steps = h.steps.clone();
        let emit = move |s| steps.lock().unwrap().push(s);
        let deps = h.deps(&client, &emit);
        let reply = run::pick(
            &deps,
            &mandi_assistant::ask::AssistantPickInput {
                question_id: asked.question_id,
                choice: PickChoice::Match { key },
            },
        )
        .unwrap_or_else(|e| panic!("{kind:?}: the pick was lost: {e:?}"));
        assert!(
            matches!(
                reply.outcome,
                AskOutcome::Offline | AskOutcome::Error | AskOutcome::TimeLimit
            ),
            "{kind:?}: {:?}",
            reply.outcome
        );
        assert_eq!(reply.failure.map(|f| f.kind), Some(kind));
        chips_work(&h);
    }
}

/// FINDING T5-6: the message for a failure is "جواب نہیں آ سکا، دوبارہ کوشش کریں" (No answer
/// came; try again), but a failed continuation has cleared the waiting question (`pick` sets
/// `waiting = None` before the call and `conclude` sets it from `Finished::wait`, which a failure
/// leaves empty). The second click is refused with `ERR_ASSISTANT_NOT_WAITING` and the Owner has to
/// type the question again. The same for the partner code: after the code, a failed call loses the
/// paused question.
#[test]
fn t5_6_a_which_one_question_can_be_clicked_again_after_a_failure() {
    let h = two_akrams();
    let which = answer_full(
        "which_one",
        "",
        json!([]),
        json!([]),
        json!("r1"),
        Value::Null,
    );
    let (asked, _) = ask(
        &h,
        "akram ka bakaya",
        None,
        vec![tool_reply(&[find("akram", "buyer")]), final_reply(&which)],
    );
    let key = asked.options[0].key.clone();
    let client = RecordedClient::with_results(vec![Err(error(FailureKind::BadKey, Some(401)))]);
    let steps = h.steps.clone();
    let emit = move |s| steps.lock().unwrap().push(s);
    let deps = h.deps(&client, &emit);
    run::pick(
        &deps,
        &mandi_assistant::ask::AssistantPickInput {
            question_id: asked.question_id,
            choice: PickChoice::Match { key: key.clone() },
        },
    )
    .unwrap();
    // The line is back: the same question goes on.
    let (done, _) = h.pick(
        asked.question_id,
        PickChoice::Match { key },
        vec![
            tool_reply(&[card("r1.matches.0")]),
            final_reply(&good_akram()),
        ],
    );
    assert_eq!(done.outcome, AskOutcome::Answered, "{done:#?}");
}

/// With the model out of reach, the questions the app answers itself still get their answer, and
/// the ones that need the model say so (NFR-AI-7: nothing else waits). The app's own paths are on
/// as they are in the shop.
#[test]
fn t5_with_no_line_the_apps_own_answers_still_come_and_the_rest_say_offline() {
    let h = Harness::day();
    h.runtime.set_local(true);
    h.runtime.set_read_first(true);
    h.runtime.set_instant(true);
    h.runtime.set_prefind(true);
    // The model is always out of reach.
    let dead = RecordedClient::with_results(vec![Err(error(FailureKind::Offline, None)); 40]);
    // A plain figure: the app writes it from the books, no call.
    let reply = ask_client(&h, "aaj ka commission", &dead);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(money(figures(answer_of(&reply))[0].0), 5_448);
    assert!(dead.requests().is_empty(), "the app answered with no call");
    assert_eq!(reply.count_today, 0, "not counted");
    // A question that needs the model ends as offline, counted as nothing.
    let reply = ask_client(
        &h,
        "kya mujhe akram ko aur udhar dena chahiye ya nahi",
        &dead,
    );
    assert!(
        matches!(reply.outcome, AskOutcome::Offline | AskOutcome::Error),
        "{reply:#?}"
    );
    assert_eq!(reply.count_today, 0);
    chips_work(&h);
}

/// The reason an error reply carries is shown to the Owner and logged (NFR-OBS-2, NFR-SEC-15):
/// whatever a provider writes, the key of the call is never in it.
#[test]
fn t5_the_reason_of_an_error_reply_never_holds_the_key() {
    let key = TEST_KEY;
    let half = &key[..key.len() / 2];
    let bodies: Vec<(&str, String)> = vec![
        (
            "the key in the message",
            format!(r#"{{"error":{{"code":401,"message":"Invalid key {key}"}}}}"#),
        ),
        (
            "the key with a json escape",
            format!(r#"{{"error":{{"message":"bad sk-or-v1-{}"}}}}"#, &key[9..]),
        ),
        (
            "the key in a bearer line",
            format!("Authorization: Bearer {key}"),
        ),
        (
            "the key in a url",
            format!("see https://openrouter.ai/api/v1/chat/completions?key={key}"),
        ),
        (
            "the key in the metadata raw",
            format!(
                r#"{{"error":{{"message":"x","metadata":{{"raw":"{key}","provider_name":"Azure"}}}}}}"#
            ),
        ),
        ("the key in lower case", key.to_lowercase()),
        ("the key in upper case", key.to_uppercase()),
        ("the key twice", format!("{key} and {key}")),
        (
            "the key cut by spaces into words",
            format!("{} {}", half, &key[key.len() / 2..]),
        ),
        (
            "the key after a zero-width space",
            format!("sk-or-v1-\u{200b}{}", &key[9..]),
        ),
        (
            "the key in html",
            format!("<html><body>Forbidden: {key}</body></html>"),
        ),
        ("the key as a header value", format!("x-api-key: {key}")),
        (
            "the key inside a long body",
            format!("{} {key} {}", "a ".repeat(5_000), "b ".repeat(5_000)),
        ),
    ];
    for (name, body) in bodies {
        let reason = reason_of(body.as_bytes(), key).unwrap_or_default();
        assert!(!reason.contains(key), "{name}: {reason}");
        assert!(
            !reason.to_lowercase().contains(&key.to_lowercase()),
            "{name}: {reason}"
        );
        assert!(
            !reason.contains(&key[9..]),
            "{name}: the secret part: {reason}"
        );
        assert!(
            reason.chars().count() <= mandi_assistant::provider_reason::MAX_REASON_CHARS,
            "{name}"
        );
    }
}

/// Every failure ends with the key in no reply, no log row and no step the panel was sent.
#[test]
fn t5_no_failure_leaves_the_key_in_a_reply_a_step_or_the_log() {
    let h = Harness::day();
    for (kind, status) in FAILURES {
        let mut e = error(kind, status);
        e.reason = Some(format!("provider said: {}", "x".repeat(40)));
        let tries = if e.retryable() { 2 } else { 1 };
        let client = RecordedClient::with_results(vec![Err(e); tries]);
        let reply = ask_client(&h, "aaj ka commission", &client);
        let text = serde_json::to_string(&reply).unwrap();
        assert!(!text.contains(TEST_KEY), "{kind:?}");
        for request in client.requests() {
            // The key travels in a header the client adds, never in the body of the request.
            assert!(
                !request.to_string().contains(TEST_KEY),
                "{kind:?}: the key is in a body"
            );
        }
    }
    let conn = crate::common::raw(&h.world.dir);
    let rows: Vec<String> = conn
        .prepare("SELECT question || answer_json || tool_calls_json || sources_json || calls_json FROM assistant_questions")
        .unwrap()
        .query_map([], |r| r.get::<_, Option<String>>(0))
        .unwrap()
        .map(|r| r.unwrap().unwrap_or_default())
        .collect();
    assert!(
        rows.iter().all(|r| !r.contains(TEST_KEY)),
        "the key is in the log"
    );
    let steps = format!("{:?}", h.steps.lock().unwrap());
    assert!(!steps.contains(TEST_KEY));
}
