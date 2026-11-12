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
use mandi_assistant::ask::{AskOutcome, PickChoice};
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

/// A third بکری that gives رحیم a second کیلا lot (his کیلا-1 is بکری 1's lot).
fn with_a_second_lot(h: &Harness) {
    let w = &h.world;
    let d = world::draft(vec![world::with_sales(
        world::lot(w, "lot-9", w.rahim, w.banana, "10"),
        vec![world::sale("s-9", w.akram, "10", "600")],
    )]);
    mandi_services::bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: d,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
}

#[test]
fn a_seller_and_an_item_with_several_lots_ask_which_lot_with_no_model_call() {
    // Plan 71 L9, PRD FR-7.77: "rahim ke banana" and "rahim ke 200 peti banana" fit two lots of رحیم,
    // کیلا-2 and کیلا-1. The app finds them itself and asks "کون سا؟" with a button for each lot: its
    // name, its day and the quantity received. No model call, so nothing is counted.
    for question in ["rahim ke banana", "rahim ke 200 peti banana"] {
        let h = harness();
        with_a_second_lot(&h);
        let (reply, requests) = h.ask(question, None, Vec::new());
        assert_eq!(
            reply.outcome,
            AskOutcome::WhichOne,
            "{question}: {reply:#?}"
        );
        assert!(requests.is_empty(), "no model call");
        // Two lots and the last button, "all lots" (key `l*`), which the screen labels itself.
        assert_eq!(reply.options.len(), 3, "{reply:#?}");
        let names: Vec<&str> = reply
            .options
            .iter()
            .take(2)
            .map(|o| o.name.name_en.as_str())
            .collect();
        assert_eq!(names, ["Rahim, Banana-2", "Rahim, Banana-1"]);
        let all = reply.options.last().unwrap();
        assert_eq!(all.key, "l*");
        assert_eq!((all.date.as_deref(), all.qty), (None, None));
        assert!(reply.options[0].key.starts_with('l'));
        assert_eq!(reply.options[0].date.as_deref(), Some("2026-10-10"));
        assert_eq!(reply.options[0].qty, Some(10));
        assert_eq!(reply.options[1].qty, Some(50));
        assert_eq!(
            reply.count_today, 0,
            "a question with no model call is not counted"
        );
    }
}

#[test]
fn a_named_lot_is_found_and_read_before_the_first_call() {
    // "rahim banana-2" and "rahim-2 ke banana" name ONE lot: its find is r1 before the model is called,
    // and the first call answers.
    for question in ["rahim banana-2", "rahim-2 ke banana"] {
        let h = harness();
        with_a_second_lot(&h);
        let (reply, requests) = h.ask(question, None, vec![final_reply("{{r1.matches.0.name}}")]);
        assert_eq!(
            reply.outcome,
            AskOutcome::Answered,
            "{question}: {reply:#?}"
        );
        assert_eq!(requests.len(), 1, "one model call");
        assert_eq!(result_ids(&requests[0]), ["r1"]);
        h.scan_requests();
    }
}

#[test]
fn a_figure_question_of_a_seller_and_an_item_with_several_lots_asks_which_lot() {
    // Plan 71 L9, the H5 shape: the question names a seller and an item among other words about what
    // he wants to know. The read would be of that seller and item, so the Owner is asked which lot,
    // with no model call. Both lots of رحیم fit: کیلا-2 and کیلا-1.
    for question in [
        "rahim ke 200 peti banana kis kis ne liye aur kis ne kitna naqd diya",
        "rahim ke banana ki bikri kitni hui",
    ] {
        let h = harness();
        with_a_second_lot(&h);
        let (reply, requests) = h.ask(question, None, Vec::new());
        assert_eq!(
            reply.outcome,
            AskOutcome::WhichOne,
            "{question}: {reply:#?}"
        );
        assert!(requests.is_empty(), "no model call: {question}");
        let names: Vec<&str> = reply
            .options
            .iter()
            .take(2)
            .map(|o| o.name.name_en.as_str())
            .collect();
        assert_eq!(names, ["Rahim, Banana-2", "Rahim, Banana-1"], "{question}");
        assert_eq!(reply.options.last().unwrap().key, "l*", "{question}");
        assert_eq!(reply.options[0].qty, Some(10));
        assert_eq!(reply.options[1].qty, Some(50));
        assert_eq!(reply.count_today, 0);
    }
}

#[test]
fn one_lot_is_not_asked_and_other_words_do_not_name_a_seller() {
    // With ONE lot of the seller and item no question is asked: the model's first call answers, and
    // the lot is read as r1. A question that names an item and no seller is not a lot ask.
    let h = harness();
    let (reply, requests) = h.ask(
        "rahim ke banana kis kis ne liye aur kis ne kitna naqd diya",
        None,
        vec![final_reply("ٹھیک")],
    );
    assert_ne!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
    assert_eq!(requests.len(), 1, "one model call, no asking");
    assert_eq!(
        result_ids(&requests[0]).first().map(String::as_str),
        Some("r1")
    );
    let h = harness();
    with_a_second_lot(&h);
    let (reply, _) = h.ask("banana ki bikri kitni hui", None, vec![final_reply("ٹھیک")]);
    assert_ne!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
}

#[test]
fn the_lot_the_owner_picked_narrows_the_report_of_its_seller_and_item() {
    // After the pick the model reads R5 for رحیم and کیلا as it always does. The app narrows it to
    // the lot: کیلا-2 sold 10 to اکرم, and ارشد bought only from کیلا-1, so he is not in the result.
    let h = harness();
    with_a_second_lot(&h);
    let (asked, _) = h.ask(
        "rahim ke 200 peti banana kis kis ne liye aur kis ne kitna naqd diya",
        None,
        Vec::new(),
    );
    assert_eq!(asked.outcome, AskOutcome::WhichOne, "{asked:#?}");
    let lot_key = asked.options[0].key.clone();
    assert_eq!(asked.options[0].name.name_en, "Rahim, Banana-2");
    let w = &h.world;
    let (picked, requests) = h.pick(
        asked.question_id,
        PickChoice::Match {
            key: lot_key.clone(),
        },
        vec![
            tool_reply(&[(
                "report",
                json!({"id": "R5", "seller": format!("a{}", w.rahim), "item": format!("i{}", w.banana)}),
            )]),
            final_reply("{{r1.matches.0.name}}"),
        ],
    );
    assert_eq!(requests.len(), 2, "{picked:#?}");
    // The first call has the pick and its note; the second has the report, narrowed.
    let first = requests[0].to_string();
    assert!(first.contains("the Owner chose this lot"), "{first}");
    let second = requests[1].to_string();
    assert!(
        second.contains("only the lot the Owner chose is read"),
        "{second}"
    );
    assert!(second.contains("Akram Toll Tax"), "{second}");
    assert!(
        !second.contains("Arshad Multani"),
        "the other lot's buyer is not read: {second}"
    );
    // The log keeps the narrowed input, so the answer can be read again.
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    assert!(
        rows[0]
            .tool_calls_json
            .contains(&format!("\"lot\":\"{lot_key}\"")),
        "{}",
        rows[0].tool_calls_json
    );
    h.scan_requests();
}

#[test]
fn a_total_word_or_a_period_word_means_all_lots_and_nothing_is_asked() {
    // The lead's rule (plan 71 L9): when the question itself asks for all of them, a total word (kul,
    // sab, total, mila kar) or a period (is mahine, aaj), no lot is asked: the read is by seller and
    // item over the period, as before. Two lots of رحیم fit, yet the Owner is not asked.
    for question in [
        "rahim ke banana ki kul bikri kitni hui",
        "rahim ke banana ka total kitna hai",
        "rahim ke sab banana mila kar kitne bike",
        "rahim ke banana is mahine kitne bike",
        "rahim ke banana aaj kitne bike",
        "رحیم کے کیلا کل کتنے بکے",
    ] {
        let h = harness();
        with_a_second_lot(&h);
        let (reply, requests) = h.ask(question, None, vec![final_reply("ٹھیک")]);
        assert_ne!(
            reply.outcome,
            AskOutcome::WhichOne,
            "{question}: {reply:#?}"
        );
        assert!(requests.len() <= 1, "{question}");
        // Nothing narrows a read: no lot is set on the question.
        let (rows, _, _) =
            mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
        assert!(
            !rows[0].tool_calls_json.contains("\"lot\":\"l"),
            "{question}"
        );
    }
    // A named counter is a lot, whatever else the question says: "banana-2" is one lot.
    let h = harness();
    with_a_second_lot(&h);
    let (reply, _) = h.ask(
        "rahim banana-2 ka total",
        None,
        vec![final_reply("{{r1.matches.0.name}}")],
    );
    assert_ne!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
}

#[test]
fn the_all_lots_button_reads_the_seller_and_item_with_no_narrowing() {
    let h = harness();
    with_a_second_lot(&h);
    let (asked, _) = h.ask(
        "rahim ke 200 peti banana kis kis ne liye aur kis ne kitna naqd diya",
        None,
        Vec::new(),
    );
    assert_eq!(asked.outcome, AskOutcome::WhichOne, "{asked:#?}");
    let all = asked.options.last().unwrap();
    assert_eq!(all.key, "l*");
    // The fallback name is the seller and the item, with no counter.
    assert_eq!(all.name.name_en, "Rahim, Banana");
    let w = &h.world;
    let (picked, requests) = h.pick(
        asked.question_id,
        PickChoice::Match {
            key: "l*".to_owned(),
        },
        vec![
            tool_reply(&[(
                "report",
                json!({"id": "R5", "seller": format!("a{}", w.rahim), "item": format!("i{}", w.banana)}),
            )]),
            final_reply("{{r1.matches.0.name}}"),
        ],
    );
    assert_eq!(requests.len(), 2, "{picked:#?}");
    let first = requests[0].to_string();
    assert!(first.contains("chose ALL lots"), "{first}");
    assert!(!first.contains("the Owner chose this lot"), "{first}");
    let second = requests[1].to_string();
    // Both lots are read: ارشد bought only from کیلا-1, اکرم from both.
    assert!(second.contains("Akram Toll Tax"), "{second}");
    assert!(second.contains("Arshad Multani"), "{second}");
    assert!(!second.contains("only the lot the Owner chose"), "{second}");
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    assert!(
        !rows[0].tool_calls_json.contains("\"lot\":\"l"),
        "{}",
        rows[0].tool_calls_json
    );
    h.scan_requests();
}
