//! Plan 55 Section 3 (the slow line) with the money review of 04-11-2026 (R-b): when time runs out
//! after a party card was read, the card shows the figure the Owner ASKED FOR, not always his
//! balance. A balance given for a question about his advance or his payable is a real number for
//! the wrong thing.
//!
//! The worked day (today 10-10-26): اکرم (a buyer) owes 2,880 بنام. The line drops after the card
//! was read, twice, so the question ends as `time_limit` with what was read, or with the failure
//! when what was read is not the figure asked.
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
use mandi_assistant::ask::{AskOutcome, AskReply, AssistantAskInput, FailureKind};
use mandi_assistant::client::{CallError, ModelClient, RecordedClient, Reply};
use mandi_assistant::run::ask;
use mandi_assistant::step::AssistantStep;
use mandi_reports::reports::Cell;
use mandi_services::core::accounts::BalanceSide;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

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
        "usage": {"prompt_tokens": 7_200, "completion_tokens": 60,
            "prompt_tokens_details": {"cached_tokens": 7_000},
            "completion_tokens_details": {"reasoning_tokens": 20}},
    })
}

/// The recorded replies, then a line that drops every call after them.
struct DropsAfter {
    inner: RecordedClient,
    calls: AtomicUsize,
    good_calls: usize,
}

impl ModelClient for DropsAfter {
    fn retry_wait(&self) -> Duration {
        Duration::ZERO
    }

    fn call(&self, key: &str, body: &Value, timeout: Duration) -> Result<Reply, CallError> {
        if self.calls.fetch_add(1, Ordering::SeqCst) >= self.good_calls {
            return Err(CallError::new(FailureKind::Dropped));
        }
        self.inner.call(key, body, timeout)
    }
}

/// Asks one question where the model finds the party, reads his card, and then the line drops.
fn card_then_drop(question: &str, name: &str, role: &str) -> AskReply {
    let h = Harness::day();
    let client = DropsAfter {
        inner: RecordedClient::new(vec![
            tool_reply(&[("find", json!({"text": name, "role": role}))]),
            tool_reply(&[("party_card", json!({"party": "r1.matches.0"}))]),
        ]),
        calls: AtomicUsize::new(0),
        good_calls: 2,
    };
    let emit = |_s: AssistantStep| {};
    let deps = h.deps(&client, &emit);
    ask(
        &deps,
        &AssistantAskInput {
            text: question.to_owned(),
            conversation_id: None,
        },
    )
    .expect("ask")
}

#[test]
fn a_balance_question_shows_his_checked_balance() {
    let reply = card_then_drop("ikram ke total bakaya kitne hai", "ikram", "buyer");
    assert_eq!(reply.outcome, AskOutcome::TimeLimit, "{reply:#?}");
    let shown = figures(answer_of(&reply));
    assert_eq!(shown.len(), 1, "{reply:#?}");
    assert_eq!(balance(shown[0].0), (2_880, BalanceSide::Dr));
    assert!(shown[0].1, "his balance carries the check");
}

#[test]
fn an_advance_question_never_shows_his_balance() {
    // Money review R-b. Before the fix the slow-line answer for "rahim ki peshgi kitni hai" showed
    // his balance, a checked cell, whatever the card held. Now it shows the card's open advance
    // (plain money, not the balance cell), or nothing when he has none.
    let reply = card_then_drop("rahim ki peshgi kitni hai", "rahim", "seller");
    if let Some(answer) = &reply.answer {
        for (cell, _) in figures(answer) {
            assert!(
                matches!(cell, Cell::Money { .. }),
                "only the open advance may show, never a balance: {cell:?}"
            );
        }
    } else {
        assert_eq!(reply.outcome, AskOutcome::Error, "{reply:#?}");
    }
    assert_ne!(reply.outcome, AskOutcome::Answered);
}

#[test]
fn a_question_the_card_does_not_hold_shows_no_figure_of_it() {
    // "How much did Akram pay": his receipts are not a field of his card. A balance would be a real
    // number for the wrong thing, so nothing from the card shows; the question ends with the
    // failure itself, and the Owner asks again.
    let reply = card_then_drop("ikram ki wasooli kitni hui", "ikram", "buyer");
    assert_eq!(reply.outcome, AskOutcome::Error, "{reply:#?}");
    assert!(reply.answer.is_none(), "{reply:#?}");
}
