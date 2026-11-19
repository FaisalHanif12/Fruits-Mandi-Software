//! General advice (plan 52 item 11, PRD FR-7.88 v1.19, Umar 02-11-2026): when the books have too
//! little for a suggestion question, or the question is general ("sale kaise barhayen"), the
//! model gives short practical mandi advice in ONE call. It is marked "عام مشورہ", names no figure
//! and no name, and shows nothing. The model is a recorded transcript.
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

use harness::{Harness, answer_of};
use mandi_assistant::answer::AnswerPart;
use mandi_assistant::ask::AskOutcome;
use serde_json::{Value, json};

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_800, "completion_tokens": 120,
        "prompt_tokens_details": {"cached_tokens": 7_600},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

fn reply(kind: &str, text: &str, show: Value) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": kind, "text": text, "show": show, "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

const GOOD: &str = "ریٹ مناسب اور مستقل رکھیں۔\n• پرانے بقایا کی وصولی ہر ہفتے کریں۔\n• جن گاہکوں کا بل پرانا ہو انہیں فون کریں۔\n• بیوپاریوں کی ادائیگی وقت پر کریں۔\n• ہر بکری اسی دن درج کریں۔";

#[test]
fn a_general_question_gets_general_advice_in_one_call_under_its_own_band() {
    let h = Harness::day();
    let (r, requests) = h.ask(
        "sale kaise barhayen",
        None,
        vec![reply("general_advice", GOOD, json!([]))],
    );
    assert_eq!(r.outcome, AskOutcome::Advice, "{r:#?}");
    assert_eq!(requests.len(), 1, "one model call");
    let a = answer_of(&r);
    assert!(
        a.advice && a.general_advice,
        "both flags: advice is general advice"
    );
    assert!(a.estimate.is_none() && a.tables.is_empty() && a.charts.is_empty());
    assert!(
        a.parts.iter().all(|p| matches!(p, AnswerPart::Text { .. })),
        "no figure, name or date of its own: {:?}",
        a.parts
    );
    assert_eq!(r.count_today, 1);
    // The plain advice kind is still advice and not general advice.
    let (r, _) = h.ask(
        "kis se pehle wasooli karun",
        None,
        vec![reply("advice", "پہلے وصولی کریں۔", json!([]))],
    );
    let a = answer_of(&r);
    assert!(a.advice && !a.general_advice);
    h.scan_requests();
}

#[test]
fn general_advice_may_not_hold_a_reference_or_show_a_result() {
    let h = Harness::day();
    // A reference is a figure or a name: refused, the model writes it again without it.
    let (r, requests) = h.ask(
        "sale kaise barhayen",
        None,
        vec![
            reply(
                "general_advice",
                "ریٹ {{r1.matches.0.name}} جیسا رکھیں۔",
                json!([]),
            ),
            reply("general_advice", GOOD, json!([])),
        ],
    );
    assert_eq!(r.outcome, AskOutcome::Advice, "{r:#?}");
    assert_eq!(requests.len(), 2, "one rewrite");
    assert!(answer_of(&r).general_advice);
    // A typed figure is refused as in every answer, and nothing may be shown.
    let (r, _) = h.ask(
        "sale kaise barhayen",
        None,
        vec![
            reply("general_advice", "بقایا 5,000 وصول کریں۔", json!([])),
            reply("general_advice", "بقایا 5,000 وصول کریں۔", json!([])),
        ],
    );
    assert_eq!(r.outcome, AskOutcome::ChecksFailed, "{r:#?}");
    let (r, _) = h.ask(
        "sale kaise barhayen",
        None,
        vec![
            reply(
                "general_advice",
                GOOD,
                json!([{"result": "r1", "as": "table", "rows": null}]),
            ),
            reply("general_advice", GOOD, json!([])),
        ],
    );
    assert_eq!(
        r.outcome,
        AskOutcome::Advice,
        "the rewrite shows nothing: {r:#?}"
    );
}
