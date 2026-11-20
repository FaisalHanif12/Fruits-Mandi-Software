//! Plan 82, the widened scope (21-11-2026): more questions the app answers by itself, and questions it
//! must NEVER end as a local "not found" or answer in part.
//!
//! The model is a client that counts every call and fails it, so an answer proves that no call was made.
//! The books are the client's demo of D8 (today 08-11-2026, hand-worked in `support/worked.rs`):
//!
//! - Mani is a BUYER: 20 mango at 1,000 on 03-11-2026. Value 20,000; his commission 6% = 1,200; his Laga
//!   20 x 8 = 160.
//! - Rahim and Ehsan are SELLERS.
//!
//! What the lead found in the live check of the 98 complex questions:
//! - "kitna maal liya aur kitni commission di" was answered with the commission only (the purchase was left
//!   out), and in Urdu the word "خریداری" was taken for a party's name (a local "not found");
//! - "مدوں" (heads) and the role word "gahaq" were taken for names and ended as "not found" (the first trip,
//!   "pehle safar", is the trip whose code is 0070: checked on the dev copy of the books, which has a trip);
//! - "har gahak ne alag alag kitni bikri li aur kitni commission di" is a row for each buyer (view C), a
//!   table the model writes from the view, read ahead.
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
use std::time::{Duration, Instant};

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

impl NoModel {
    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
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

/// The books as the shop runs them: the app's own reads, answers and speed-ups on.
fn production() -> Harness {
    let h = Harness::over(worked::d8().w);
    h.runtime.set_local(true);
    h.runtime.set_read_first(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

fn ask_with(h: &Harness, client: &NoModel, question: &str) -> (AskReply, Duration) {
    let emit = |_s: AssistantStep| {};
    let deps = h.deps(client, &emit);
    let started = Instant::now();
    let reply = ask(
        &deps,
        &AssistantAskInput {
            text: question.to_owned(),
            conversation_id: None,
        },
    )
    .expect("ask");
    (reply, started.elapsed())
}

fn shown(reply: &AskReply) -> Vec<i64> {
    figures(answer_of(reply))
        .into_iter()
        .map(|(cell, _)| money(cell))
        .collect()
}

#[test]
fn what_a_buyer_bought_and_the_commission_he_gave_are_both_answered_by_the_app() {
    // Before: only the commission was shown, and the purchase (20,000) was left out of the answer.
    let h = production();
    for question in [
        "mani ne abhi tak kitna maal liya aur kitni commission di ?",
        "mani ne abhi tak kitni khareedari ki aur kitni commission di ?",
        "مانی نے اب تک کتنی خریداری کی اور کتنا کمیشن دیا؟",
    ] {
        let client = NoModel::default();
        let (reply, took) = ask_with(&h, &client, question);
        assert_eq!(client.calls(), 0, "{question}: the model was called");
        assert_eq!(
            reply.outcome,
            AskOutcome::Answered,
            "{question}: {reply:#?}"
        );
        let figures = shown(&reply);
        assert!(
            figures.contains(&20_000),
            "{question}: the purchase is missing: {figures:?}"
        );
        assert!(
            figures.contains(&1_200),
            "{question}: the commission is missing: {figures:?}"
        );
        assert!(took < Duration::from_secs(1), "{question}: took {took:?}");
    }
}

#[test]
fn the_shop_taking_commission_is_not_a_purchase() {
    // "commission li" is the shop taking it: a bare "li" is no word of buying, so no purchase is added.
    let h = production();
    let client = NoModel::default();
    let (reply, _) = ask_with(
        &h,
        &client,
        "shop ne mani se abhi tak kitni commission li ?",
    );
    let figures = shown(&reply);
    assert!(
        !figures.contains(&20_000),
        "the purchase was added to a commission question: {figures:?}"
    );
}

#[test]
fn a_name_that_matches_nothing_in_the_books_is_never_a_party_of_the_question() {
    // The words of a trip, of heads and of a role are no names: the question goes on (to the model or to a
    // read) and never ends as the app's own "name not found". The model client here fails, so a question
    // that reaches it is counted; a local "not found" would be counted 0.
    let h = production();
    for question in [
        "پچھلے مہینے دکان کا خرچہ مدوں کے حساب سے کتنا ہوا؟",
        "6 november ko ehsan ne kitna maal bheja aur kis kis gahaq ne kitna liya ?",
    ] {
        let client = NoModel::default();
        let (reply, took) = ask_with(&h, &client, question);
        assert_ne!(
            reply.outcome,
            AskOutcome::NotFound,
            "{question}: {reply:#?}"
        );
        assert!(client.calls() > 0, "{question}: the model was not asked");
        assert!(took < Duration::from_secs(2), "{question}: took {took:?}");
    }
}

#[test]
fn each_buyer_asked_for_sale_and_commission_has_view_c_read_before_the_model_answers() {
    // "har gahak ne alag alag kitni bikri li aur kitni commission di": one row for each buyer. The model
    // writes the table, so the view is read ahead and the first request already holds it.
    let h = production();
    let (_, requests) = h.ask(
        "is mahine har gahak ne alag alag kitni bikri li aur kitni commission di ?",
        None,
        Vec::new(),
    );
    assert!(!requests.is_empty(), "the model was not asked");
    let first = requests[0].to_string();
    assert!(
        first.contains("\\\"view\\\":\\\"C\\\"") || first.contains("\"view\":\"C\""),
        "view C was not read ahead: {first}"
    );
}

#[test]
fn a_few_names_asked_together_are_read_ahead_for_the_model() {
    // Two buyers and a commission: the app writes no sum, the model does, from the reads made ahead.
    let h = production();
    let (_, requests) = h.ask(
        "How much commission did the shop earn from Mani and Haroon together this month?",
        None,
        Vec::new(),
    );
    assert!(!requests.is_empty(), "the model was not asked");
    let tools = requests[0]["messages"]
        .as_array()
        .map_or(0, |m| m.iter().filter(|x| x["role"] == "tool").count());
    assert!(tools > 0, "no read ahead: {}", requests[0]);
}

#[test]
fn who_gave_the_most_commission_reads_the_buyers_own_column_not_the_total_of_both_sides() {
    // D04: view H ranks a buyer by the TOTAL commission of his lines, which holds the seller's commission on
    // them. The commission a buyer gave is his own column of view C (buyer commission, Laga beside it), so
    // the first request already holds view C, sorted by it.
    let h = production();
    for question in [
        "is mahine sab se zyada commission kis gahaq ne di ?",
        "which buyer gave the most commission this month ?",
    ] {
        let (_, requests) = h.ask(question, None, Vec::new());
        assert!(!requests.is_empty(), "{question}: the model was not asked");
        let first = requests[0].to_string();
        assert!(
            first.contains("\\\"view\\\":\\\"C\\\"") || first.contains("\"view\":\"C\""),
            "{question}: view C was not read ahead: {first}"
        );
        assert!(
            !first.contains("\\\"view\\\":\\\"H\\\"") && !first.contains("\"view\":\"H\""),
            "{question}: view H (the total of both sides) was read: {first}"
        );
    }
}
