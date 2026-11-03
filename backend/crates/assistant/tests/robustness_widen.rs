//! Attacks on the widened find (plan 52 P52-S1): an empty buyer or seller search is widened to the
//! other role, an empty any-party search to the partners. The result says so (`widened`, and each
//! match its own `type`), and a widened match must never be taken for something it is not: not by a
//! typed filter, and never past the lock on the partner section.
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
use mandi_assistant::ask::AskOutcome;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::partner_code::PartnerCodeSetInput;
use mandi_services::partner_code;
use serde_json::{Value, json};

fn usage() -> Value {
    json!({"prompt_tokens": 7_200, "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 7_000},
        "completion_tokens_details": {"reasoning_tokens": 20}})
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
    json!({"model": "openai/gpt-6-luna",
           "choices": [{"finish_reason": "tool_calls", "message": {
               "role": "assistant", "content": null, "tool_calls": calls}}],
           "usage": usage()})
}

fn final_reply(kind: &str, text: &str, choose: Value) -> Value {
    json!({"model": "openai/gpt-6-luna",
           "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
               "kind": kind, "text": text, "show": [], "next": [],
               "choose": choose, "entry": null, "nearest": null
           }).to_string()}}],
           "usage": usage()})
}

fn find(text: &str, role: &str) -> (&'static str, Value) {
    ("find", json!({"text": text, "role": role}))
}

fn analytics_args(view: &str, set: Value) -> Value {
    let mut call = json!({
        "view": view, "from": null, "to": null, "group_by": null, "item": null, "seller": null,
        "buyer": null, "city": null, "adda": null, "garden": null, "trip": null, "goods": null,
        "partner": null, "side": null, "compare": null, "sort": null, "rows": null
    });
    for (k, v) in set.as_object().unwrap() {
        call[k] = v.clone();
    }
    call
}

/// The tool messages of a request as parsed results, in order.
fn results_of(body: &Value) -> Vec<Value> {
    body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .map(|m| serde_json::from_str(m["content"].as_str().unwrap_or("{}")).unwrap_or(Value::Null))
        .collect()
}

#[test]
fn a_find_in_the_wrong_role_says_it_widened_and_gives_the_real_type_of_each_match() {
    // Arshad is a buyer: a seller search for him is widened and says so.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "arshad ka bakaya",
        None,
        vec![
            tool_reply(&[find("arshad", "seller")]),
            tool_reply(&[("party_card", json!({"party": "r1.matches.0"}))]),
            final_reply(
                "answer",
                "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔",
                Value::Null,
            ),
        ],
    );
    let found = &results_of(&requests[1])[0];
    assert!(
        found["widened"].as_bool().unwrap_or(false)
            || found["widened"].is_string()
            || found["widened"].is_object(),
        "the find says it widened: {found}"
    );
    let first = &found["matches"][0];
    assert!(
        first["type"].to_string().to_lowercase().contains("buyer"),
        "the match is a buyer: {first}"
    );
    // The answer is about him, with his checked balance.
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    assert_eq!(balance(figs[0].0), (18_684, BalanceSide::Dr));
    h.scan_requests();
}

#[test]
fn a_typed_filter_never_takes_a_widened_match() {
    // `analytics seller: "arshad"` typed by the model, with Arshad a buyer: the seller filter must
    // not quietly take the buyer. The question stops at once asking which one (no further model
    // call), the buyer is only offered by name, and no seller figure is shown.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "arshad ke maal par commission",
        None,
        vec![tool_reply(&[(
            "analytics",
            analytics_args("B", json!({"seller": "arshad"})),
        )])],
    );
    assert_eq!(
        requests.len(),
        1,
        "the app asked, the model was not called again"
    );
    assert_eq!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
    assert_eq!(
        reply
            .options
            .iter()
            .map(|o| o.name.name_en.as_str())
            .collect::<Vec<_>>(),
        ["Arshad Multani"]
    );
    assert!(
        reply.answer.is_none(),
        "no figure of a seller who is not one"
    );
    h.scan_requests();
}

#[test]
fn a_widened_partner_search_shows_no_figure_of_a_partner_behind_the_lock() {
    // The partner section is locked. An any-party search for a partner is widened to the partners
    // and must give no figure of his (the partners' money is behind the code): the find, and every
    // request after it, holds no partner total.
    // The control: with the section open the same search finds the partner (so it is the lock
    // that hides him below).
    let open = Harness::day();
    let (_, open_requests) = open.ask(
        "alif ki kul raqam",
        None,
        vec![
            tool_reply(&[find("alif", "any_party")]),
            final_reply("not_understood", "", Value::Null),
        ],
    );
    let opened = results_of(&open_requests[1]).remove(0);
    assert!(
        opened["matches"].as_array().is_some_and(|m| !m.is_empty()),
        "with the section open the partner is found: {opened}"
    );
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
    let (reply, requests) = h.ask(
        "alif ki kul raqam",
        None,
        vec![
            tool_reply(&[find("alif", "any_party")]),
            final_reply("not_understood", "", Value::Null),
        ],
    );
    let found = results_of(&requests[1]).remove(0);
    let text = found.to_string().to_lowercase();
    for word in ["balance", "total", "amount", "capital", "profit", "share"] {
        assert!(
            !text.contains(word),
            "the widened find shows {word} of a partner behind the lock: {found}"
        );
    }
    // The worked day's partner totals (A: -1,004; B and C: 746) are nowhere in any request.
    for body in &requests {
        let all = body.to_string();
        for figure in ["-100400", "74600"] {
            assert!(
                !all.contains(figure),
                "a partner total reached the model: {figure}"
            );
        }
    }
    assert_ne!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    h.scan_requests();
}
