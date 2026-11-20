//! Plan 82: what the app repairs by itself, so that a refused answer does not cost a whole model
//! call (live check of 20-11-2026: of 123 calls, 7 were rewrites). Recorded transcripts, no network.
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

use harness::{Harness, answer_of, figures, money};
use mandi_assistant::ask::AskOutcome;
use serde_json::{Value, json};

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_200, "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 7_000},
        "completion_tokens_details": {"reasoning_tokens": 0}
    })
}

fn tool_reply(name: &str, args: Value) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null, "tool_calls": [
                {"id": "call_0", "type": "function",
                 "function": {"name": name, "arguments": args.to_string()}}]}}],
        "usage": usage(),
    })
}

fn answer_reply(text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

/// View C (by buyer) for the worked day, every field given as the strict schema wants.
fn view_c() -> Value {
    json!({
        "view": "C", "from": "2026-10-01", "to": "2026-10-10", "group_by": null, "item": null,
        "seller": null, "buyer": null, "city": null, "adda": null, "garden": null, "trip": null,
        "goods": null, "partner": null, "side": null, "compare": null, "sort": null, "rows": null
    })
}

/// Live check M11: the model cited `{{r1.purchase_value}}` where the figure is `r1.totals`'
/// `purchase_value` (the question here asks for the purchase value itself, not a bill with its
/// commission and laga, which the measure guards would refuse). Only the place is wrong, so the app writes it right and the answer needs no
/// rewrite: two model calls, not three, and the same figure as with the right path.
#[test]
fn a_figure_named_on_the_result_that_only_its_totals_hold_needs_no_rewrite() {
    let h = Harness::day();
    let (right, requests_right) = h.ask(
        "total purchase value of all the buyers",
        None,
        vec![
            tool_reply("analytics", view_c()),
            answer_reply("{{r1.totals.purchase_value}}"),
        ],
    );
    assert_eq!(right.outcome, AskOutcome::Answered, "{right:#?}");
    assert_eq!(requests_right.len(), 2);
    let h = Harness::day();
    let (wrong, requests_wrong) = h.ask(
        "total purchase value of all the buyers",
        None,
        vec![
            tool_reply("analytics", view_c()),
            answer_reply("{{r1.purchase_value}}"),
        ],
    );
    assert_eq!(wrong.outcome, AskOutcome::Answered, "{wrong:#?}");
    assert_eq!(
        requests_wrong.len(),
        2,
        "the wrong place was written right: no rewrite call"
    );
    assert_eq!(
        money(figures(answer_of(&wrong))[0].0),
        money(figures(answer_of(&right))[0].0)
    );
    // A figure that no place of the result holds is still refused (and the rewrite is told).
    let h = Harness::day();
    let (nothing, requests_nothing) = h.ask(
        "total purchase value of all the buyers",
        None,
        vec![
            tool_reply("analytics", view_c()),
            answer_reply("{{r1.no_such_figure}}"),
            answer_reply("{{r1.totals.purchase_value}}"),
        ],
    );
    assert_eq!(requests_nothing.len(), 3, "{nothing:#?}");
}
