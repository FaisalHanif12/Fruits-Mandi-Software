//! Plan 82 (21-11-2026, the wrong-column fix): "safi bikri" is the NET column and "gahakon ke bill" the
//! BUYER TOTAL, never another real figure of the same books.
//!
//! The live check answered "ab tak ki safi bikri kitni hui" with the gross sale (382,000 against a net of
//! 332,780) and "safi bikri batao" with the buyers' bills (407,750). The model path is guarded by words
//! (`sale_guard`): a cited cell of another sale column is rewritten to the right cell of the same row, or
//! refused with the cells that hold the right column. The model here is a recorded transcript: no network.
//!
//! The books are the client's demo of D8 (today 08-11-2026; hand-worked in `support/worked.rs`): the shop's
//! gross sale is 260,500, the commission charged to the sellers 15,630 (6% of each line) and nothing else is
//! deducted, so the NET sale is 260,500 - 15,630 = 244,870. The buyers' bills are the goods 260,500 + the
//! buyers' commission 15,630 + Laga 310 nag x 8 = 2,480, so 278,610. Mani's bill is 21,360 (E2E-65).
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
use mandi_assistant::ask::{AskOutcome, AskReply};
use mandi_assistant::understand::{SaleMeasure, asked_sale_measure};
use serde_json::{Value, json};

const GROSS: i64 = 260_500;
const NET: i64 = 244_870;
const BILLS: i64 = 278_610;

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_200, "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 7_000},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

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
        "usage": usage(),
    })
}

fn final_reply(text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

/// A `report` call with every field of the schema.
fn report(id: &str) -> (&'static str, Value) {
    (
        "report",
        json!({
            "id": id, "adda": null, "as_of": null, "bikri": null, "city": null, "collector": null,
            "from": null, "garden": null, "group_by": null, "item": null, "partner": null,
            "party": null, "rows": null, "sort": null, "to": null, "trip": null, "where": null,
            "seller": null, "zamindar": null, "status": null, "voucher_type": null, "layout": null,
            "item_group": null
        }),
    )
}

/// An `analytics` call of view A, over all the books.
fn view_a() -> (&'static str, Value) {
    (
        "analytics",
        json!({
            "view": "A", "from": null, "to": null, "group_by": null, "item": null, "item_group": null,
            "seller": null, "buyer": null, "city": null, "adda": null, "garden": null, "trip": null,
            "goods": null, "partner": null, "side": null, "compare": null, "sort": null, "rows": null,
            "grade": null
        }),
    )
}

fn shown(reply: &AskReply) -> Vec<i64> {
    figures(answer_of(reply))
        .into_iter()
        .map(|(cell, _)| money(cell))
        .collect()
}

/// The text of every request after the first, for what the app told the model.
fn told(requests: &[Value]) -> String {
    requests
        .iter()
        .skip(1)
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_words_of_a_question_name_the_measure() {
    let net = |q: &str| asked_sale_measure(q).map(|a| a.measure);
    assert_eq!(
        net("abhi tak ki safi bikri kitni hai ?"),
        Some(SaleMeasure::Net)
    );
    assert_eq!(net("safi bikri batao"), Some(SaleMeasure::Net));
    assert_eq!(net("اب تک کی صافی بکری کتنی ہے؟"), Some(SaleMeasure::Net));
    assert_eq!(net("What is the net sale so far?"), Some(SaleMeasure::Net));
    assert_eq!(
        net("What is the total of all buyer bills so far?"),
        Some(SaleMeasure::Bills)
    );
    assert_eq!(
        net("mani ke bill ki kul raqam kitni hai (commission aur laga samet) ?"),
        Some(SaleMeasure::Bills)
    );
    assert_eq!(
        net("ہارون کے بلوں کی کل رقم کتنی ہے؟"),
        Some(SaleMeasure::Bills)
    );
    // The gross sale named by its word is its own column (the guard keeps R2's goods value out of it).
    assert_eq!(
        net("abhi tak ki khaam bikri kitni hui"),
        Some(SaleMeasure::Gross)
    );
    // The plain sale, a count of bills and a partner's net name no sale measure.
    assert_eq!(net("aaj ki bikri kitni hui"), None);
    assert_eq!(net("aaj kitne bill bane"), None);
    assert_eq!(net("alif ki khalis raqam kitni hai"), None);
    // A commission beside the bills is a second question, unless it is a part of the bill.
    let apart = asked_sale_measure("mani ke bill aur commission batao").unwrap();
    assert!(apart.commission_too);
    let inside = asked_sale_measure("mani ke bill ki raqam commission samet").unwrap();
    assert!(!inside.commission_too);
}

#[test]
fn the_gross_cited_for_the_net_sale_is_rewritten_to_the_net_of_the_same_report() {
    // R6 holds both columns, so the cell is changed by the app, and no second call is made.
    let h = Harness::over(worked::d8().w);
    let (reply, requests) = h.ask(
        "ab tak ki safi bikri kitni hui",
        None,
        vec![
            tool_reply(&[report("R6")]),
            final_reply("{{r1.totals.gross}}"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(shown(&reply), [NET]);
    assert_eq!(requests.len(), 2, "the app repaired the cell itself");
}

#[test]
fn the_gross_of_a_view_with_no_net_column_is_refused_with_the_cells_that_hold_it() {
    // View A has no net column. The answer cites its gross: refused, and the model is told which cell of
    // the R6 it read holds the net.
    let h = Harness::over(worked::d8().w);
    let (reply, requests) = h.ask(
        "ab tak ki safi bikri kitni hui",
        None,
        vec![
            tool_reply(&[view_a(), report("R6")]),
            final_reply("{{r1.totals.gross}}"),
            final_reply("{{r2.totals.net}}"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(shown(&reply), [NET]);
    assert_eq!(requests.len(), 3, "one rewrite");
    let said = told(&requests);
    assert!(said.contains("measure"), "{said}");
    assert!(said.contains("r2.totals.net"), "{said}");
}

#[test]
fn a_gross_never_shows_for_the_net_sale_but_the_plain_sale_keeps_its_gross() {
    let h = Harness::over(worked::d8().w);
    // The model keeps citing the gross: the second answer is refused too, and nothing wrong is shown.
    let (reply, _) = h.ask(
        "safi bikri batao",
        None,
        vec![
            tool_reply(&[view_a()]),
            final_reply("{{r1.totals.gross}}"),
            final_reply("{{r1.totals.gross}}"),
        ],
    );
    assert!(
        !shown(&reply).contains(&GROSS),
        "the gross was shown for the net sale: {reply:#?}"
    );
    // The plain sale, and both asked together, are no fault.
    let (reply, _) = h.ask(
        "abhi tak ki khaam bikri kitni hui",
        None,
        vec![tool_reply(&[view_a()]), final_reply("{{r1.totals.gross}}")],
    );
    assert_eq!(shown(&reply), [GROSS], "{reply:#?}");
    let (reply, _) = h.ask(
        "khaam bikri aur safi bikri dono batao",
        None,
        vec![
            tool_reply(&[view_a(), report("R6")]),
            final_reply("{{r1.totals.gross}} {{r2.totals.net}}"),
        ],
    );
    assert_eq!(shown(&reply), [GROSS, NET], "{reply:#?}");
}

#[test]
fn the_buyers_balances_cited_for_the_buyers_bills_are_refused() {
    // R20 holds what the buyers still owe (a balance), R2 the bills. "gahakon ke bill" is the bills.
    let h = Harness::over(worked::d8().w);
    let (reply, requests) = h.ask(
        "abhi tak gahakon ke kul bill kitne ke bane",
        None,
        vec![
            tool_reply(&[report("R20"), report("R2")]),
            final_reply("{{r1.totals.balance}}"),
            final_reply("{{r2.totals.buyer_total}}"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(shown(&reply), [BILLS]);
    assert_eq!(requests.len(), 3);
    assert!(told(&requests).contains("r2.totals.buyer_total"));
}

#[test]
fn the_goods_value_cited_for_the_bills_is_rewritten_to_the_bill_total_of_the_same_register() {
    let h = Harness::over(worked::d8().w);
    let (reply, requests) = h.ask(
        "abhi tak gahakon ke kul bill kitne ke bane",
        None,
        vec![
            tool_reply(&[report("R2")]),
            final_reply("{{r1.totals.value}}"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(shown(&reply), [BILLS]);
    assert_eq!(requests.len(), 2);
}
