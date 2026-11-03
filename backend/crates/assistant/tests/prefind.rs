//! The party of a plain question is found by the app before the first model call (plan 52 P52-S1,
//! item 10), only when there is no doubt: one word finds exactly one account in the role the question
//! implies, with the close spellings included, and no other word finds another single account.
//! Every doubtful case is left to the model's own `find`. The worked day: اکرم ٹول ٹیکس (buyer, owes
//! 2,880), ارشد ملتانی (buyer, 18,684), رحیم (seller, 0) and کریم (seller, 10,160 جمع).
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
use mandi_assistant::answer::AnswerPart;
use mandi_assistant::ask::AskOutcome;
use mandi_services::core::accounts::BalanceSide;
use serde_json::{Value, json};

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

fn find(text: &str, role: &str) -> (&'static str, Value) {
    ("find", json!({"text": text, "role": role}))
}

fn harness() -> Harness {
    let h = Harness::day();
    h.runtime.set_prefind(true);
    h
}

fn result_ids(body: &Value) -> Vec<String> {
    body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .map(|m| {
            let v: Value = serde_json::from_str(m["content"].as_str().unwrap_or("{}")).unwrap();
            v["id"].as_str().unwrap_or("").to_owned()
        })
        .collect()
}

#[test]
fn a_buyer_named_in_a_plain_question_is_found_and_read_before_the_first_call() {
    // "ikram ke total bakaya kitne hai": "bakaya" says a buyer, "ikram" finds Akram only (a close
    // spelling), no other word finds anyone. His find (r1) and card (r2) are there before the
    // model is called; its first call is the answer: ONE model call.
    let h = harness();
    let (reply, requests) = h.ask(
        "ikram ke total bakaya kitne hai",
        None,
        vec![final_reply(
            "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔",
        )],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 1, "one model call");
    assert_eq!(result_ids(&requests[0]), ["r1", "r2"]);
    let figs = figures(answer_of(&reply));
    assert_eq!(balance(figs[0].0), (2_880, BalanceSide::Dr));
    assert!(figs[0].1, "checked against his ledger");
    h.scan_requests();
}

#[test]
fn a_seller_is_found_by_the_words_that_imply_a_seller() {
    let h = harness();
    let (reply, requests) = h.ask(
        "kareem ko kitna dena he",
        None,
        vec![final_reply(
            "{{r1.matches.0.name}} کو {{r2.balance}} دینے ہیں۔",
        )],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 1);
    assert_eq!(
        balance(figures(answer_of(&reply))[0].0),
        (10_160, BalanceSide::Cr)
    );
    h.scan_requests();
}

#[test]
fn the_name_of_the_found_party_always_shows_in_an_answer_about_him() {
    // The model leaves his name out; the app puts it first.
    let h = harness();
    let (reply, _) = h.ask(
        "ikram ke total bakaya kitne hai",
        None,
        vec![final_reply("{{r2.balance}} باقی ہیں۔")],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let parts = &answer_of(&reply).parts;
    assert!(
        matches!(&parts[0], AnswerPart::Name { name } if name.name_en == "Akram Toll Tax"),
        "{parts:?}"
    );
    assert!(
        matches!(&parts[1], AnswerPart::Text { text } if text == ": "),
        "{parts:?}"
    );
}

#[test]
fn a_ledger_question_reads_the_card_and_the_ledger_and_the_models_find_is_answered_from_r1() {
    // "akram ka khata dikhao": any party (khata fits both); the find r1, the card r2 and the ledger
    // R13 r3 are read. A model that still asks for its own find of the same word and role gets r1.
    let h = harness();
    let (reply, requests) = h.ask(
        "akram ka khata dikhao",
        None,
        vec![
            tool_reply(&[find("akram", "any_party")]),
            final_reply("{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(result_ids(&requests[0]), ["r1", "r2", "r3"]);
    assert_eq!(
        result_ids(&requests[1]),
        ["r1", "r2", "r3", "r1"],
        "no second find"
    );
}

#[test]
fn doubt_leaves_the_find_to_the_model() {
    // Two parties, a seller named as a buyer, a name nobody has, a how-to, a number, a follow-up:
    // no find and no read before the first call (the request holds the user message only).
    for question in [
        "akram aur arshad ka bakaya kitna hai",
        "kareem ka bakaya kitna hai",
        "zzzparty ka bakaya kitna hai",
        "akram ka khata kaise dekhun",
        "akram ki 5000 wasooli kitni",
    ] {
        let h = harness();
        let (_, requests) = h.ask(
            question,
            None,
            vec![
                tool_reply(&[find("akram", "buyer")]),
                final_reply("{{r1.matches.0.name}}"),
                final_reply("{{r1.matches.0.name}}"),
            ],
        );
        assert!(result_ids(&requests[0]).is_empty(), "{question}");
    }

    // A follow-up in the same conversation is never pre-found: it leans on the earlier answer.
    let h = harness();
    let (first, _) = h.ask(
        "ikram ke total bakaya kitne hai",
        None,
        vec![final_reply("{{r1.matches.0.name}} {{r2.balance}}")],
    );
    let (_, requests) = h.ask(
        "arshad ka bakaya?",
        Some(&first.conversation_id),
        vec![
            tool_reply(&[find("arshad", "buyer")]),
            final_reply("{{r3.matches.0.name}} {{r4.balance}}"),
        ],
    );
    let ids = result_ids(&requests[0]);
    assert_eq!(
        ids,
        ["r1", "r2"],
        "only the first question's reads: {ids:?}"
    );
}

#[test]
fn a_party_found_by_the_app_who_is_not_the_one_asked_about_is_not_named_in_another_answer() {
    // The model decides the question is about arshad instead, finds him itself and answers about
    // him: Akram's name is not put in front of Arshad's answer.
    let h = harness();
    let (reply, _) = h.ask(
        "ikram ke total bakaya kitne hai",
        None,
        vec![
            tool_reply(&[find("arshad", "buyer")]),
            final_reply("{{r3.matches.0.name}} کے ذمے {{r4.balance}} باقی ہیں۔"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let parts = &answer_of(&reply).parts;
    let names: Vec<&str> = parts
        .iter()
        .filter_map(|p| match p {
            AnswerPart::Name { name } => Some(name.name_en.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["Arshad Multani"], "{parts:?}");
}
