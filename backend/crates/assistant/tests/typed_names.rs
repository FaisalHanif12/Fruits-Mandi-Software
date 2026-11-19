//! A typed name in a filter is looked up by the app (plan 52 item 6). The model is a recorded
//! transcript. Before, `item: "apple"` was refused and cost a round (Q054 and Q090 of the live run
//! of 02-11-2026); now one match is taken, and none or several end the question as `not_found` or
//! `which_one` at once.
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
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

fn tool_reply(name: &str, args: &Value) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null,
            "tool_calls": [{"id": "call_0", "type": "function",
                "function": {"name": name, "arguments": args.to_string()}}]}}],
        "usage": usage(),
    })
}

fn final_reply(kind: &str, text: &str, choose: &Value) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": kind, "text": text, "show": [], "next": [],
            "choose": choose, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

fn tool_messages(request: &Value) -> Vec<String> {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .map(|m| m["content"].as_str().unwrap().to_owned())
        .collect()
}

fn analytics(extra: Value) -> Value {
    let mut base = json!({
        "view": "A", "from": "2026-10-01", "to": "2026-10-31", "group_by": null, "item": null,
        "seller": null, "buyer": null, "city": null, "adda": null, "garden": null, "trip": null,
        "goods": null, "partner": null, "side": null, "compare": null, "sort": null, "rows": null
    });
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

#[test]
fn a_typed_item_with_one_match_is_taken_and_the_note_says_so() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "saib par kitna commission",
        None,
        vec![
            tool_reply("analytics", &analytics(json!({"item": "apple"}))),
            final_reply(
                "answer",
                "اس مال پر کمیشن {{r1.totals.total_comm}} بنا۔",
                &Value::Null,
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 2, "no round was lost to a refusal");
    let message = &tool_messages(&requests[1])[0];
    // The model's own result keeps its id r1; the app's find is f1.
    assert!(message.contains("\"id\":\"r1\""), "{message}");
    assert!(
        message.contains("was taken as the only match, f1.matches.0"),
        "{message}"
    );
    // The figure is Apple's: 720 + 720 + 80 (PRD 18.4).
    assert_eq!(money(figures(answer_of(&reply))[0].0), 1_520);
    // The log keeps the find with its id, and the call with the reference.
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    let log: Vec<Value> = serde_json::from_str(&rows[0].tool_calls_json).unwrap();
    assert_eq!(log[0]["id"], "f1");
    assert_eq!(log[0]["tool"], "find");
    assert_eq!(log[1]["input"]["item"], "f1.matches.0");
    h.scan_requests();
}

#[test]
fn a_typed_name_with_no_match_is_refused_and_the_question_ends_as_not_found() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "zzzfruit par kitna commission",
        None,
        vec![
            tool_reply("analytics", &analytics(json!({"item": "zzzfruit"}))),
            final_reply("not_found", "", &json!("f1")),
        ],
    );
    // The app ends the question itself (plan 52 item 8): no second model call.
    assert_eq!(reply.outcome, AskOutcome::NotFound, "{reply:#?}");
    assert_eq!(requests.len(), 1);
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    assert!(rows[0].tool_calls_json.contains("ERR_UNKNOWN_REFERENCE"));
}

#[test]
fn a_typed_name_with_several_matches_is_refused_and_the_owner_is_asked_which() {
    let h = Harness::day();
    // "a" is in both Akram and Arshad: several accounts.
    let party = json!({
        "id": "R13", "from": null, "to": null, "as_of": null, "party": "a", "item": null,
        "city": null, "adda": null, "collector": null, "partner": null, "trip": null,
        "garden": null, "bikri": null, "group_by": null, "where": null, "sort": null, "rows": null
    });
    let (reply, requests) = h.ask(
        "a ka khata",
        None,
        vec![
            tool_reply("report", &party),
            final_reply("which_one", "", &json!("f1")),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
    assert!(reply.options.len() >= 2, "{:?}", reply.options);
    assert_eq!(requests.len(), 1, "no model call is spent on the choice");
}

#[test]
fn a_reference_is_never_looked_up_again() {
    // A real find result is used as it is: no f result, no note.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "saib par kitna commission",
        None,
        vec![
            tool_reply("find", &json!({"text": "apple", "role": "item"})),
            tool_reply("analytics", &analytics(json!({"item": "r1.matches.0"}))),
            final_reply(
                "answer",
                "اس مال پر کمیشن {{r2.totals.total_comm}} بنا۔",
                &Value::Null,
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert!(
        !requests[2]
            .to_string()
            .contains("was taken as the only match")
    );
}

#[test]
fn a_direction_word_the_model_writes_after_a_difference_is_not_printed_twice() {
    // Q081 of the live run of 02-11-2026: the panel prints the word after the difference, and the
    // model wrote it again ("3,000 زیادہ زیادہ"). The second one is dropped.
    use mandi_assistant::answer::{AnswerPart, FigureDirection};
    let h = Harness::day();
    let (reply, _) = h.ask(
        "arshad akram se kitna zyada deta hai",
        None,
        vec![
            tool_reply("find", &json!({"text": "arshad", "role": "buyer"})),
            tool_reply("find", &json!({"text": "akram", "role": "buyer"})),
            tool_reply(
                "calculate",
                // The app read Arshad's card ahead as r2, so the second find is r3.
                &json!({"op": "difference",
                        "inputs": ["r1.matches.0.balance", "r3.matches.0.balance"]}),
            ),
            final_reply(
                "answer",
                "ارشد کا بقایا اکرم سے {{c1.result}} زیادہ ہے۔",
                &Value::Null,
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let parts = &answer_of(&reply).parts;
    let has_direction = parts.iter().any(|p| {
        matches!(
            p,
            AnswerPart::Figure {
                dir: Some(FigureDirection::Up),
                ..
            }
        )
    });
    assert!(has_direction, "{parts:?}");
    let words: String = parts
        .iter()
        .filter_map(|p| match p {
            AnswerPart::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert!(!words.contains("زیادہ"), "{words}");
    assert!(words.contains("ہے"), "{words}");
}

#[test]
fn a_name_that_only_a_close_spelling_found_is_not_taken_without_the_owner() {
    // Money review of plan 52 item 2: "Faisal" typed, only "Faizal" exists. One close match is
    // kept for the Owner to confirm, never taken silently. Here "akrm" is close to Akram.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "akrm ka commission",
        None,
        vec![
            tool_reply("analytics", &analytics(json!({"buyer": "akrm"}))),
            final_reply("which_one", "", &json!("f1")),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
    assert_eq!(reply.options.len(), 1);
    assert_eq!(requests.len(), 1);
    // The word that names him ("akram", a word of his name) is taken.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "akram ka commission",
        None,
        vec![
            tool_reply("analytics", &analytics(json!({"buyer": "akram"}))),
            final_reply(
                "answer",
                "اس گاہک پر کمیشن {{r1.totals.total_comm}} بنا۔",
                &Value::Null,
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert!(tool_messages(&requests[1])[0].contains("taken as the only match"));
}

#[test]
fn a_typed_roman_urdu_item_is_taken_through_the_word_list() {
    // "kela" is not in the item's names (کیلا, Banana); the word list gives its Urdu spelling, and
    // the one item with that name is taken: Banana, 3,928 of commission (PRD 18.4).
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "kela se kitna comission mila",
        None,
        vec![
            tool_reply("analytics", &analytics(json!({"item": "kela"}))),
            final_reply(
                "answer",
                "اس مال سے کمیشن {{r1.totals.total_comm}} ملا۔",
                &Value::Null,
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 2, "no round lost");
    assert_eq!(money(figures(answer_of(&reply))[0].0), 3_928);
    assert!(tool_messages(&requests[1])[0].contains("taken as the only match"));
}
