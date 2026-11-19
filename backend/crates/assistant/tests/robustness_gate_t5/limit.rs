//! The daily limit of 20 (PRD FR-7.81, FR-7.85, NFR-AI-5, Code Standards Q9).
//!
//! `robustness_assistant.rs` attack 7 has: the 21st question refused with no request, 8 tools
//! counted once, two Enters at once, and the midnight Pakistan time boundary. This file adds the
//! rows of the table that are not there: what does not count (chips, a "which one" pick, the
//! partner code, the automatic rewrite, a call that failed before any reply, reading an answer
//! again, a connection test that is refused), what does (a click on a suggested next question, a
//! connection test), and the day a question that crosses midnight belongs to.
#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use chrono::TimeZone;
use mandi_assistant::ask::{AskOutcome, AssistantAskInput, FailureKind, PickChoice, QuickChip};
use mandi_assistant::client::{CallError, ModelClient, RecordedClient, Reply};
use mandi_assistant::run;
use mandi_services::assistant;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::partner_code::{PartnerCodeSetInput, PartnerCodeUnlockInput};
use mandi_services::partner_code;
use serde_json::{Value, json};

use crate::harness::Harness;
use crate::kit::*;
use crate::{common, world};

fn count(h: &Harness) -> i64 {
    assistant::count_today(&h.world.env.ctx).unwrap()
}

/// The `counted` flag of every row of the question log, oldest first.
fn counted_flags(h: &Harness) -> Vec<i64> {
    let conn = common::raw(&h.world.dir);
    conn.prepare("SELECT counted FROM assistant_questions ORDER BY id")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn fill_to(h: &Harness, n: i64) {
    while count(h) < n {
        ask(h, "aaj ka commission", None, gives_up());
    }
}

/// The 20th question is the last; a question that is a rewrite costs one, not two.
#[test]
fn t5_a_rewrite_is_not_a_second_question() {
    let h = Harness::day();
    fill_to(&h, 19);
    let typed = answer("answer", "{{r1.matches.0.name}} کے ذمے 2,800 باقی ہیں۔");
    // The 20th: the answer fails the checks, the one rewrite is good. Two replies, one question.
    let (reply, requests) = ask(
        &h,
        "ikram ke total bakaya kitne hai",
        None,
        akram_then(vec![final_reply(&typed), final_reply(&good_akram())]),
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 4, "find, card, answer, rewrite");
    assert_eq!(reply.count_today, 20);
    // The 21st is refused with no request.
    let (over, requests) = ask(&h, "aaj ka commission", None, gives_up());
    assert_eq!(over.outcome, AskOutcome::LimitReached);
    assert!(requests.is_empty());
    // Every row of the log is one question, and only model questions are counted.
    assert_eq!(counted_flags(&h).iter().filter(|c| **c == 1).count(), 20);
}

/// Chips never count and are never stopped, at 19, at 20 and past it.
#[test]
fn t5_chips_are_not_counted_and_never_stopped() {
    let h = Harness::day();
    for target in [0, 19, 20] {
        fill_to(&h, target);
        for chip in [
            QuickChip::CashInHand,
            QuickChip::TodaySales,
            QuickChip::TodayCommission,
            QuickChip::BuyersDues,
            QuickChip::TotalDues,
        ] {
            let before = count(&h);
            let reply = h.chip(chip);
            assert_eq!(reply.count_today, before, "{chip:?} at {target}");
            assert_ne!(
                reply.outcome,
                AskOutcome::LimitReached,
                "{chip:?} at {target}"
            );
        }
    }
    // The chip rows are in the log and none of them is counted.
    let flags = counted_flags(&h);
    assert_eq!(flags.iter().filter(|c| **c == 1).count(), 20, "{flags:?}");
    assert!(flags.len() > 20, "the chips are logged");
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

/// "کون سا؟": the click on a button answers with no new question counted, also at the limit.
#[test]
fn t5_a_click_on_a_which_one_button_is_not_counted_and_is_not_stopped_at_the_limit() {
    let h = two_akrams();
    fill_to(&h, 19);
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
    assert_eq!(asked.outcome, AskOutcome::WhichOne, "{asked:#?}");
    assert_eq!(asked.count_today, 20, "the 20th question");
    assert_eq!(asked.options.len(), 2);
    // At 20 the pick still goes on: it is not a new question.
    let choice = PickChoice::Match {
        key: asked.options[0].key.clone(),
    };
    let (done, requests) = h.pick(
        asked.question_id,
        choice,
        vec![
            tool_reply(&[card("r1.matches.0")]),
            final_reply(&good_akram()),
        ],
    );
    assert_ne!(done.outcome, AskOutcome::LimitReached, "{done:#?}");
    assert_eq!(done.count_today, 20, "the pick did not count");
    assert!(!requests.is_empty(), "the pick reached the model");
    let flags = counted_flags(&h);
    assert_eq!(
        *flags.last().unwrap(),
        0,
        "the pick's row is not counted: {flags:?}"
    );
}

/// The partner code: the paused question goes on after the code, with no new count.
#[test]
fn t5_the_partner_code_continuation_is_not_counted_and_is_not_stopped_at_the_limit() {
    let h = Harness::day();
    let ctx = &h.world.env.ctx;
    partner_code::set(
        ctx,
        &h.world.owner,
        &PartnerCodeSetInput {
            password: common::OWNER_PASSWORD.to_owned(),
            code: "4826".to_owned(),
            code_again: "4826".to_owned(),
        },
    )
    .unwrap();
    partner_code::lock(ctx).unwrap();
    fill_to(&h, 19);
    let (waiting, requests) = ask(
        &h,
        "partner ki kiya total hain abhi tak",
        None,
        vec![
            tool_reply(&[("report", report_args("R31", json!({})))]),
            final_reply(&answer("answer", "x")),
        ],
    );
    assert_eq!(waiting.outcome, AskOutcome::Locked);
    assert_eq!(requests.len(), 1);
    // A question that stops on the code made no model reply of its own (the first call replied
    // with the tool call, so it counts: the 20th).
    assert_eq!(waiting.count_today, 20, "{:?}", counted_flags(&h));
    partner_code::unlock(
        ctx,
        &h.world.owner,
        &PartnerCodeUnlockInput {
            code: "4826".into(),
        },
    )
    .unwrap();
    let transcript = harness_transcript();
    let (done, _) = h.pick(waiting.question_id, PickChoice::PartnerCode, transcript);
    assert_ne!(done.outcome, AskOutcome::LimitReached, "{done:#?}");
    assert_eq!(
        done.count_today, 20,
        "the continuation after the code did not count"
    );
}

fn harness_transcript() -> Vec<Value> {
    crate::harness::transcript(include_str!("../fixtures/model/e2e-50b.json"))
}

/// A call that fails before any reply is not counted, however many times it fails; the question
/// that then works counts once.
#[test]
fn t5_a_failed_first_call_is_never_counted() {
    let h = Harness::day();
    for kind in [
        FailureKind::Offline,
        FailureKind::Timeout,
        FailureKind::Server,
        FailureKind::Busy,
        FailureKind::BadKey,
        FailureKind::NoCredit,
        FailureKind::NoEndpoint,
        FailureKind::Dropped,
    ] {
        for _ in 0..3 {
            let error = CallError {
                kind,
                http_status: None,
                reason: None,
            };
            let tries = if error.retryable() { 2 } else { 1 };
            let client = RecordedClient::with_results(vec![Err(error); tries]);
            let steps = h.steps.clone();
            let emit = move |s| steps.lock().unwrap().push(s);
            let deps = h.deps(&client, &emit);
            let reply = run::ask(
                &deps,
                &AssistantAskInput {
                    text: "aaj ka commission".into(),
                    conversation_id: None,
                },
            )
            .unwrap();
            assert_eq!(reply.count_today, 0, "{kind:?}");
        }
    }
    assert_eq!(count(&h), 0, "{} failed questions", 8 * 3);
    // And the 20 that work are all allowed afterwards.
    fill_to(&h, 20);
    let (over, _) = ask(&h, "aaj ka commission", None, gives_up());
    assert_eq!(over.outcome, AskOutcome::LimitReached);
}

/// A first call that fails and is tried once more successfully is one question.
#[test]
fn t5_a_retry_that_works_is_one_question() {
    let h = Harness::day();
    let client = RecordedClient::with_results(vec![
        Err(CallError::new(FailureKind::Timeout)),
        Ok(final_reply(&answer("not_understood", ""))),
    ]);
    let steps = h.steps.clone();
    let emit = move |s| steps.lock().unwrap().push(s);
    let deps = h.deps(&client, &emit);
    let reply = run::ask(
        &deps,
        &AssistantAskInput {
            text: "aaj ka commission".into(),
            conversation_id: None,
        },
    )
    .unwrap();
    assert_eq!(reply.count_today, 1);
    assert_eq!(client.requests().len(), 2);
}

/// A click on a next question is a new question (FR-7.81): it counts and the limit stops it.
#[test]
fn t5_a_next_question_that_is_asked_counts_and_the_limit_stops_it() {
    let h = Harness::day();
    fill_to(&h, 20);
    let (reply, requests) = ask(&h, "pichle mahine ka?", None, gives_up());
    assert_eq!(reply.outcome, AskOutcome::LimitReached);
    assert!(requests.is_empty());
}

/// Reading an answer again (the log screen) is not a question: not counted, not stopped.
#[test]
fn t5_reading_an_answer_again_is_not_counted_and_not_stopped() {
    let h = Harness::day();
    let (first, _) = ask(
        &h,
        "ikram ke total bakaya kitne hai",
        None,
        akram_then(vec![final_reply(&good_akram())]),
    );
    fill_to(&h, 20);
    let client = RecordedClient::new(Vec::new());
    let emit = |_s| {};
    let deps = h.deps(&client, &emit);
    let again = run::rerun(
        &deps,
        &mandi_assistant::ask::AssistantRerunInput {
            question_id: first.question_id,
        },
    )
    .unwrap();
    assert_eq!(again.count_today, 20);
    assert!(client.requests().is_empty(), "no model call");
    let flags = counted_flags(&h);
    assert_eq!(*flags.last().unwrap(), 0, "{flags:?}");
}

/// FINDING T5-5: "کنکشن جانچیں" counts as a question (FR-7.85) but `test_connection` never reads
/// the count: at 20 it still calls the model, and the day goes past 20 (21, 22 ...). The limit is "a
/// hard stop" (FR-7.81, NFR-AI-5).
#[test]
fn t5_5_the_connection_test_is_stopped_at_the_limit() {
    let h = Harness::day();
    fill_to(&h, 20);
    let client = RecordedClient::new(vec![final_reply(&answer("not_understood", ""))]);
    let emit = |_s| {};
    let deps = h.deps(&client, &emit);
    let result = run::test_connection(&deps);
    let stopped = match &result {
        Err(_) => true,
        Ok(r) => !r.worked && client.requests().is_empty(),
    };
    assert!(
        stopped && client.requests().is_empty(),
        "the connection test went to the model at 20 questions: {result:?}, requests {}",
        client.requests().len()
    );
    assert_eq!(count(&h), 20, "no 21st counted question");
}

/// The test below the limit counts (FR-7.85), and a chip after it is still free.
#[test]
fn t5_the_connection_test_counts_as_a_question() {
    let h = Harness::day();
    let client = RecordedClient::new(vec![final_reply(&answer("not_understood", ""))]);
    let emit = |_s| {};
    let deps = h.deps(&client, &emit);
    let result = run::test_connection(&deps).unwrap();
    assert!(result.worked, "{result:?}");
    assert_eq!(result.count_today, 1);
}

/// A client that moves the clock past midnight Pakistan time while the model "thinks".
struct CrossesMidnight<'a> {
    h: &'a Harness,
    calls: AtomicUsize,
}

impl ModelClient for CrossesMidnight<'_> {
    fn call(&self, _key: &str, _body: &Value, _timeout: Duration) -> Result<Reply, CallError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        // 18:59:58 UTC is 23:59:58 in Pakistan; the reply comes at 19:00:03 UTC, 00:00:03.
        self.h.world.env.clock.set(
            chrono::Utc
                .with_ymd_and_hms(2026, 10, 10, 19, 0, 3)
                .single()
                .unwrap(),
        );
        Reply::from_json(&final_reply(&answer("not_understood", "")))
    }
}

/// A question that starts on one day and ends on the next is counted once, on one day, and never
/// on both: the day of the log row and the day the count reads are the same.
#[test]
fn t5_a_question_that_crosses_midnight_is_counted_once_on_one_day() {
    let h = Harness::day();
    h.world.env.clock.set(
        chrono::Utc
            .with_ymd_and_hms(2026, 10, 10, 18, 59, 58)
            .single()
            .unwrap(),
    );
    fill_to(&h, 19);
    let client = CrossesMidnight {
        h: &h,
        calls: AtomicUsize::new(0),
    };
    let steps = h.steps.clone();
    let emit = move |s| steps.lock().unwrap().push(s);
    let deps = h.deps(&client, &emit);
    let reply = run::ask(
        &deps,
        &AssistantAskInput {
            text: "aaj ka commission".into(),
            conversation_id: None,
        },
    )
    .unwrap();
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    let (rows, _, _) = assistant::list_questions(&h.world.env.ctx, 0, 100).unwrap();
    let on_day = |day: &str| {
        rows.iter()
            .filter(|r| r.asked_day == day && r.counted)
            .count()
    };
    let (first, second) = (on_day("2026-10-10"), on_day("2026-10-11"));
    assert_eq!(
        first + second,
        20,
        "every counted question is on one day: {first} + {second}"
    );
    // The day the count reads now is the day the row says.
    let today = assistant::pakistan_day(&h.world.env.ctx);
    assert_eq!(today, "2026-10-11");
    assert_eq!(reply.count_today, i64::try_from(second).unwrap());
}

/// The count of a day is the number of counted rows of that Pakistan day: a log with rows on
/// several days counts each day by itself, to the 20.
#[test]
fn t5_each_day_has_its_own_twenty() {
    let h = Harness::day();
    for (day, hour) in [(10u32, 9u32), (11, 9), (12, 9)] {
        h.world.env.clock.set(
            chrono::Utc
                .with_ymd_and_hms(2026, 10, day, hour, 0, 0)
                .single()
                .unwrap(),
        );
        fill_to(&h, 20);
        let (over, requests) = ask(&h, "aaj ka commission", None, gives_up());
        assert_eq!(over.outcome, AskOutcome::LimitReached, "day {day}");
        assert!(requests.is_empty());
    }
    let (rows, total, _) = assistant::list_questions(&h.world.env.ctx, 0, 100).unwrap();
    assert!(total >= 63, "{total}");
    assert_eq!(rows.iter().filter(|r| r.counted).count(), 60);
}
