//! Fewer model calls and fewer rewrites (plan 52 P52-S1, the time analysis of the full live run of
//! 03-11-2026). One section for each cut of that analysis; the worked day is the data: اکرم ٹول
//! ٹیکس owes 2,880 بنام, ارشد ملتانی 18,684.
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

fn answer_with(kind: &str, text: &str, next: &[&str], show: Value) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": kind, "text": text, "show": show, "next": next,
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

fn final_reply(text: &str) -> Value {
    answer_with("answer", text, &[], json!([]))
}

fn find(text: &str, role: &str) -> (&'static str, Value) {
    ("find", json!({"text": text, "role": role}))
}

/// The tool messages of a request, as (tool_call_id, parsed content).
fn tool_messages(body: &Value) -> Vec<(String, Value)> {
    body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .map(|m| {
            (
                m["tool_call_id"].as_str().unwrap_or("").to_owned(),
                serde_json::from_str(m["content"].as_str().unwrap_or("{}")).unwrap_or(Value::Null),
            )
        })
        .collect()
}

fn result_ids(body: &Value) -> Vec<String> {
    tool_messages(body)
        .iter()
        .map(|(_, v)| v["id"].as_str().unwrap_or("").to_owned())
        .collect()
}

fn log_calls(h: &Harness) -> Vec<Value> {
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    serde_json::from_str(rows[0].calls_json.as_deref().unwrap()).unwrap()
}

// ---- 1. a path may go on below a value by the value's own field names ---------------------------

#[test]
fn a_reference_that_goes_on_below_a_value_by_its_field_names_is_the_value() {
    // The model reads {"rs": 2880} and writes `.rs`, or a name's `.ur`: the same figure and the same
    // name, no rewrite.
    for text in [
        "{{r1.matches.0.name}} کے ذمے {{r1.matches.0.balance}} باقی ہیں۔",
        "{{r1.matches.0.name.ur}} کے ذمے {{r1.matches.0.balance.rs}} باقی ہیں۔",
        "{{r1.matches.0.name}} کے ذمے {{r1.matches.0.balance.side}} {{r1.matches.0.balance.rs}} باقی ہیں۔",
    ] {
        let h = Harness::day();
        let (reply, requests) = h.ask(
            "ikram ke total bakaya kitne hai",
            None,
            vec![tool_reply(&[find("akram", "buyer")]), final_reply(text)],
        );
        assert_eq!(reply.outcome, AskOutcome::Answered, "{text}: {reply:#?}");
        // The find with one match is followed by the app's read of the card, then the answer.
        assert_eq!(requests.len(), 2, "{text}: no rewrite call");
        let answer = answer_of(&reply);
        assert!(
            matches!(&answer.parts[0], AnswerPart::Name { .. }),
            "{text}"
        );
        let figs = figures(answer);
        assert_eq!(balance(figs[0].0), (2_880, BalanceSide::Dr), "{text}");
        assert!(figs[0].1, "{text}: still checked");
        h.scan_requests();
    }
}

#[test]
fn only_the_fields_of_a_value_are_forgiven_not_any_word_after_it() {
    // `.rupees` is not a field of the value; `.rs.rs.rs` is too deep; a path to nothing stays refused.
    for text in [
        "{{r1.matches.0.balance.rupees}} باقی ہیں۔",
        "{{r1.matches.0.balance.rs.rs.rs}} باقی ہیں۔",
        "{{r1.matches.0.nothing.rs}} باقی ہیں۔",
    ] {
        let h = Harness::day();
        let (reply, requests) = h.ask(
            "ikram ke total bakaya kitne hai",
            None,
            vec![
                tool_reply(&[find("akram", "buyer")]),
                final_reply(text),
                final_reply(text),
            ],
        );
        assert_eq!(
            reply.outcome,
            AskOutcome::ChecksFailed,
            "{text}: {reply:#?}"
        );
        assert_eq!(requests.len(), 3, "{text}: the one rewrite");
    }
}

// ---- 2. what the app read ahead has the normal numbers, and is not read twice -------------------

#[test]
fn the_card_and_the_ledger_read_ahead_are_r2_and_r3_and_a_repeated_call_gets_them_back() {
    // "akram ka khata dikhao": the app reads the card (r2) and the ledger R13 (r3) after the find
    // (r1), the numbers the prompt's example gives them. A recorded model that asks for the very
    // same two reads next (the transcript of E2E-47 does) is given r2 and r3 again, and its
    // answer, which cites r2.balance and r3.rows, still replays. Nothing is read twice.
    let h = Harness::day();
    let r13 = json!({"id": "R13", "party": "r1.matches.0", "from": null, "to": null, "as_of": null,
        "item": null, "city": null, "adda": null, "collector": null, "partner": null,
        "trip": null, "garden": null, "bikri": null, "group_by": null, "where": null,
        "sort": null, "rows": null});
    let (reply, requests) = h.ask(
        "akram ka khata dikhao",
        None,
        vec![
            tool_reply(&[find("akram", "buyer")]),
            tool_reply(&[
                ("party_card", json!({"party": "r1.matches.0"})),
                ("report", r13.clone()),
            ]),
            answer_with(
                "answer",
                "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔\n• ان سے {{r3.rows.je2.credit}} وصول ہوئے۔",
                &[],
                json!([{"result": "r3", "as": "table", "rows": 7}]),
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 3);
    // The third request holds the app's two reads and the model's two calls, answered by the same
    // two results: four tool messages, only the ids r1, r2, r3.
    let ids = result_ids(&requests[2]);
    assert_eq!(ids, ["r1", "r2", "r3", "r2", "r3"], "{ids:?}");
    let calls = log_calls(&h);
    assert!(
        calls[1]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .all(|t| t.get("prefetch").is_none())
    );
    // The log lists each read once: the find and the app's two reads.
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    let logged: Vec<Value> = serde_json::from_str(&rows[0].tool_calls_json).unwrap();
    assert_eq!(logged.len(), 3, "{logged:?}");
    assert_eq!(
        balance(figures(answer_of(&reply))[0].0),
        (2_880, BalanceSide::Dr)
    );
    h.scan_requests();

    // A different read of the same ledger (100 rows) is a new one: the next number.
    let h = Harness::day();
    let mut more = r13;
    more["rows"] = json!(100);
    let (_, requests) = h.ask(
        "akram ka khata dikhao",
        None,
        vec![
            tool_reply(&[find("akram", "buyer")]),
            tool_reply(&[("report", more)]),
            final_reply("{{r1.matches.0.name}} {{r4.totals.balance}}"),
        ],
    );
    assert_eq!(result_ids(&requests[2]), ["r1", "r2", "r3", "r4"]);
}

// ---- 6. an estimate needs no `ahead` the question already gives -------------------------------

/// April to September 2026: the commission of each month is 36,000 ... 51,000 (E2E-59).
const HISTORY: [(u32, i64, i64); 6] = [
    (4, 180, 1_600),
    (5, 150, 2_100),
    (6, 120, 2_850),
    (7, 225, 1_600),
    (8, 150, 2_600),
    (9, 173, 2_390),
];

fn estimate_args(extra: Value) -> Value {
    let mut input = json!({
        "measure": "commission_all", "item": null, "seller": null, "buyer": null,
        "garden": null, "trip": null, "ahead": null, "until": null, "what_if": null,
        "percent": null, "target": null
    });
    for (k, v) in extra.as_object().unwrap() {
        input[k] = v.clone();
    }
    input
}

/// Asks one question whose first reply is an `estimate` call and returns that tool's result.
fn estimate_result(question: &str, args: Value) -> Value {
    let h = Harness::over(worked::history_world(&HISTORY));
    let (_, requests) = h.ask(
        question,
        None,
        vec![
            tool_reply(&[("estimate", estimate_args(args))]),
            final_reply("غلط"),
            final_reply("غلط"),
        ],
    );
    tool_messages(&requests[1]).remove(0).1
}

#[test]
fn an_estimate_without_ahead_takes_the_rule_of_the_tool_and_no_call_is_lost_to_an_error() {
    // A target and no months: 12 months. No target and no months: 6. One months number: that one.
    let v = estimate_result(
        "commission ka 3 lakh ka target kab tak pura hoga",
        json!({"target": "q.n1"}),
    );
    assert!(v.get("error").is_none(), "{v}");
    assert_eq!(v["ahead_count"]["n"], 12);
    let v = estimate_result("agle mahine ka commission kitna hoga", json!({}));
    assert!(v.get("error").is_none(), "{v}");
    assert_eq!(v["ahead_count"]["n"], 6);
    let v = estimate_result("agle 3 mahine ka commission kitna hoga", json!({}));
    assert_eq!(v["ahead_count"]["n"], 3, "{v}");
    // Two months numbers: the Owner is asked.
    let v = estimate_result("agle 3 mahine ya agle 6 mahine ka commission", json!({}));
    assert_eq!(v["error"]["code"], "ERR_REQUIRED", "{v}");
    // The model's own number still has to be the question's (or the default).
    let v = estimate_result("agle mahine ka commission", json!({"ahead": 9}));
    assert_eq!(v["error"]["code"], "ERR_OUT_OF_RANGE", "{v}");
}

#[test]
fn the_end_of_the_year_in_the_question_is_enough_and_inflected_scenario_words_count() {
    // No `until` sent, but the question says to the end of the year: October to December.
    let v = estimate_result("saal ke aakhir tak commission kitna hoga", json!({}));
    assert!(v.get("error").is_none(), "{v}");
    assert_eq!(v["ahead_count"]["n"], 3);
    assert_eq!(v["year"]["so_far"]["rs"], 261_000);
    // "if the pace doubles ... by the end of the year" is a what-if: "doubles" is "double".
    let v = estimate_result(
        "if the pace doubles how much commission by the end of the year",
        json!({"until": "year_end", "what_if": "double"}),
    );
    assert!(v.get("error").is_none(), "{v}");
    assert_eq!(v["what_if_pace"]["total"]["rs"], 288_000);
    // And the same without a target on a year-end question with a target is still asked.
    let v = estimate_result(
        "saal ke aakhir tak 4 lakh ka target pura hoga",
        json!({"target": "q.n1"}),
    );
    assert_eq!(v["error"]["code"], "ERR_OUT_OF_RANGE", "{v}");
}

// ---- 9. the log says what the refused piece was (never a name, never a digit) -------------------

#[test]
fn the_rewrite_row_says_which_number_word_letter_or_path_was_refused() {
    let h = Harness::day();
    let bad = final_reply(
        "{{r1.matches.0.name}} کے ذمے ہزار Owed {{r1.matches.0.balance}} باقی ہیں، 2 اور اکرم ٹول ٹیکس۔",
    );
    let (reply, _) = h.ask(
        "ikram ke total bakaya kitne hai",
        None,
        vec![tool_reply(&[find("akram", "buyer")]), bad.clone(), bad],
    );
    assert_eq!(reply.outcome, AskOutcome::ChecksFailed, "{reply:#?}");
    let calls = log_calls(&h);
    let rewrite = calls.iter().find(|c| c["rewrite"] == true).unwrap();
    let at: Vec<&str> = rewrite["check_at"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(at.contains(&"number_word:ہزار"), "{at:?}");
    assert!(at.iter().any(|a| a.starts_with("script:")), "{at:?}");
    // The typed name was put back by the app (so it is no failure of the rewrite), and neither it
    // nor the digit is ever in `check_at`.
    assert!(
        !rewrite["check_failed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r == "name")
    );
    assert!(
        calls.iter().any(|c| c["repaired"] == json!(["name"])),
        "{calls:?}"
    );
    assert!(
        at.iter()
            .all(|a| !a.contains("اکرم") && !a.starts_with("digit") && !a.starts_with("name")),
        "{at:?}"
    );
}

// ---- 3. a help answer that broke the words rules is written from the guide itself ----------------

#[test]
fn a_help_answer_that_retyped_the_guide_is_written_from_the_guide_without_a_rewrite() {
    // "bikri kaise add karun": the app reads the guide (h1) before the first call. The model retypes
    // it in its own words with a key name (Latin letters) and a number word: the words rules fail,
    // and the app writes the answer from the guide's summary, steps and keys: one model call.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "bikri kaise add karun",
        None,
        vec![answer_with(
            "answer",
            "بکری بک کھولیں، Alt+B دبائیں اور دو لائنیں لکھیں۔",
            &[],
            json!([]),
        )],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 1, "no rewrite call");
    let answer = answer_of(&reply);
    // Only the guide's own words: text parts of the app, no figure, no table.
    assert!(!answer.parts.is_empty());
    assert!(
        answer
            .parts
            .iter()
            .all(|p| matches!(p, AnswerPart::Text { .. })),
        "{:?}",
        answer.parts
    );
    let calls = log_calls(&h);
    assert_eq!(calls[0]["repaired"], json!(["help"]), "{calls:?}");
    // The log keeps what the model wrote.
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    assert!(rows[0].answer_json.as_deref().unwrap().contains("Alt+B"));
    h.scan_requests();
}

#[test]
fn the_guide_is_not_written_over_a_wrong_reference_a_data_question_or_a_name() {
    // A reference that does not exist is not a words rule: the one rewrite is asked.
    let h = Harness::day();
    let bad = final_reply("{{h9.summary}}");
    let (reply, requests) = h.ask("bikri kaise add karun", None, vec![bad.clone(), bad]);
    assert_eq!(reply.outcome, AskOutcome::ChecksFailed, "{reply:#?}");
    assert_eq!(requests.len(), 2);

    // A question that also read the books (a report) is not a help answer: its figures are not
    // the guide's.
    let h = Harness::day();
    let typed = final_reply("بکری Alt+B دو بار۔");
    let (reply, requests) = h.ask(
        "what is each partner's total now",
        None,
        vec![tool_reply(&[("tiles", json!({}))]), typed.clone(), typed],
    );
    assert_eq!(reply.outcome, AskOutcome::ChecksFailed, "{reply:#?}");
    assert_eq!(
        requests.len(),
        3,
        "the rewrite was asked for, not the guide"
    );
}

// ---- 8. the Owner's own number written in words is his number -----------------------------------

#[test]
fn a_number_word_that_echoes_the_owners_number_is_put_back_without_a_rewrite() {
    // "agle 6 mahine": the model writes "چھ مہینوں". The number-word rule would refuse it; the app
    // writes `{{q.n1}}` (his 6) and checks again: one model call fewer.
    let h = Harness::over(worked::history_world(&HISTORY));
    let (reply, requests) = h.ask(
        "agle 6 mahine ka commission kitna hoga",
        None,
        vec![
            tool_reply(&[("estimate", estimate_args(json!({"ahead": 6})))]),
            answer_with(
                "estimate",
                "اگلے چھ مہینوں میں اسی رفتار سے تقریباً {{e1.same_pace.total}} بنے گا۔",
                &[],
                json!([{"result": "e1", "as": "chart", "rows": null}]),
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Estimate, "{reply:#?}");
    assert_eq!(requests.len(), 2, "no rewrite call");
    let calls = log_calls(&h);
    assert_eq!(calls[1]["repaired"], json!(["number"]), "{calls:?}");
    // His number is shown as a number (6), the estimate as an estimate.
    let answer = answer_of(&reply);
    let figs = figures(answer);
    assert!(
        figs.iter()
            .any(|(cell, _)| matches!(cell, mandi_reports::reports::Cell::Number { value: 6 })),
        "{:?}",
        answer.parts
    );
    h.scan_requests();

    // A number word that is not his number is still refused (3 months asked, "six" written).
    let h = Harness::over(worked::history_world(&HISTORY));
    let bad = answer_with(
        "estimate",
        "اگلے چھ مہینوں میں {{e1.same_pace.total}}",
        &[],
        json!([{"result": "e1", "as": "chart", "rows": null}]),
    );
    let (reply, requests) = h.ask(
        "agle 3 mahine ka commission kitna hoga",
        None,
        vec![
            tool_reply(&[("estimate", estimate_args(json!({"ahead": 3})))]),
            bad.clone(),
            bad,
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::ChecksFailed, "{reply:#?}");
    assert_eq!(requests.len(), 3);
}

// ---- 4 and 5. the period a question names and the tiles it is about are read before the first call ----

fn view_a_args(extra: Value) -> Value {
    let mut input = json!({
        "view": "A", "from": null, "to": null, "group_by": null, "item": null, "seller": null,
        "buyer": null, "city": null, "adda": null, "garden": null, "trip": null, "goods": null,
        "partner": null, "side": null, "compare": null, "sort": null, "rows": null
    });
    for (k, v) in extra.as_object().unwrap() {
        input[k] = v.clone();
    }
    input
}

#[test]
fn one_clear_period_is_read_first_so_the_first_call_asks_for_the_report() {
    // "is mahine ki bikri": the period (this month) is r1 before the model is called; its first
    // call is the analytics view with `from: r1.from`; two calls, not three. A recorded model that
    // asks for the period itself (E2E-49 does) is given r1 back.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "is mahine ki bikri",
        None,
        vec![
            tool_reply(&[(
                "analytics",
                view_a_args(json!({"from": "r1.from", "to": "r1.to"})),
            )]),
            answer_with(
                "answer",
                "{{r1.label}} میں خام بکری {{r2.totals.gross}} ہوئی۔",
                &[],
                json!([]),
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 2);
    // The first request already holds the period, with its own tool call and tool message.
    assert_eq!(result_ids(&requests[0]), ["r1"]);
    let first = tool_messages(&requests[0]);
    assert_eq!(first[0].0, "pre_period");
    assert_eq!(first[0].1["from"]["date"], "2026-10-01");
    assert_eq!(harness::money(figures(answer_of(&reply))[0].0), 41_400);
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    let logged: Vec<Value> = serde_json::from_str(&rows[0].tool_calls_json).unwrap();
    assert_eq!(logged[0]["prefetch"], true);
    h.scan_requests();

    // The model asking for the very same period gets r1 back, and its own next read is r2.
    let h = Harness::day();
    let period = json!({"kind": "this_month", "month": null, "day": null, "from": null, "to": null, "n": null});
    let (_, requests) = h.ask(
        "is mahine ki bikri",
        None,
        vec![
            tool_reply(&[("period", period)]),
            tool_reply(&[(
                "analytics",
                view_a_args(json!({"from": "r1.from", "to": "r1.to"})),
            )]),
            final_reply("{{r2.totals.gross}}"),
        ],
    );
    assert_eq!(
        result_ids(&requests[2]),
        ["r1", "r1", "r2"],
        "no second period"
    );
}

#[test]
fn two_periods_a_number_the_end_of_the_year_or_another_period_are_left_to_the_model() {
    for question in [
        "is mahine ka commission pichle mahine se zyada hai",
        "pichle 3 mahine ka commission",
        "is saal ke aakhir tak commission kitna hoga",
        "september ka commission kitna tha",
        "ikram ke total bakaya kitne hai",
    ] {
        let h = Harness::day();
        let (_, requests) = h.ask(
            question,
            None,
            vec![
                tool_reply(&[("tiles", json!({}))]),
                final_reply("غلط"),
                final_reply("غلط"),
            ],
        );
        // Nothing before the first call: the user message only.
        assert!(tool_messages(&requests[0]).is_empty(), "{question}");
    }
}

#[test]
fn a_tile_question_has_its_tiles_before_the_first_call_and_takes_one_model_call() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "aaj ki bikri kitni hui",
        None,
        vec![answer_with(
            "answer",
            "آج کی خام بکری {{r1.today_sales.amount}} ہوئی۔",
            &[],
            json!([]),
        )],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 1, "one model call");
    assert_eq!(result_ids(&requests[0]), ["r1"]);
    assert_eq!(harness::money(figures(answer_of(&reply))[0].0), 41_400);
    h.scan_requests();

    // A party's question that holds the same words is not a tile question ("ikram ke total
    // bakaya"), nor a help question about the screen.
    for question in [
        "ikram ke total bakaya kitne hai",
        "aaj ki bikri wali screen kahan hai",
    ] {
        let h = Harness::day();
        let (_, requests) = h.ask(
            question,
            None,
            vec![
                tool_reply(&[find("akram", "buyer")]),
                final_reply("غلط"),
                final_reply("غلط"),
            ],
        );
        let ids = result_ids(&requests[0]);
        assert!(
            !ids.contains(&"r1".to_owned()) || question.contains("screen"),
            "{question}: {ids:?}"
        );
    }
}
