//! A partner's کل رقم is not his capital (live run 1, 05-11-2026: the wrong figure, with a tick).
//!
//! "alif ki kul raqam kitni hai" was answered 0 (Q198, Q199) and "usman ka kitna hai abhi" was
//! answered with Usman's capital, 200,000, when his کل رقم is 2,506,856 (Q546 to Q548). The model
//! was shown the partner's statement with a `capital` block beside it, whose `balance` it cited.
//! Now the capital is shown only when the question asks for it.
//!
//! The worked day (today 10-10-26): الف has earned 2,196, paid 1,200 of the shop's expense and
//! taken 2,000, so his کل رقم is −1,004 (PRD 4.9, E2E-39); his capital is 0.
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
use mandi_assistant::ask::AskReply;
use serde_json::{Value, json};

fn usage() -> Value {
    json!({"prompt_tokens": 1_000, "completion_tokens": 20})
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
    json!({"model": "m", "choices": [{"finish_reason": "tool_calls", "message": {
        "role": "assistant", "content": null, "tool_calls": calls}}], "usage": usage()})
}

fn final_reply(text: &str) -> Value {
    json!({"model": "m", "choices": [{"finish_reason": "stop", "message": {
        "role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null}).to_string()}}], "usage": usage()})
}

/// R31 read for the partner the first `find` returned.
fn statement() -> Value {
    json!({"id": "R31", "adda": null, "as_of": null, "bikri": null, "city": null,
           "collector": null, "from": null, "garden": null, "group_by": null, "item": null,
           "partner": "r1.matches.0", "party": null, "rows": null, "sort": null, "to": null,
           "trip": null, "where": null})
}

/// The `report` result the model was shown in the last request.
fn report_result(requests: &[Value]) -> Value {
    requests
        .last()
        .and_then(|r| r["messages"].as_array())
        .into_iter()
        .flatten()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .find(|v| v["tool"] == "report")
        .expect("the report result")
}

/// Asks a question about a partner: find him, read his statement, answer with `text`.
fn ask_partner(question: &str, name: &str, text: &str) -> (AskReply, Vec<Value>) {
    let h = Harness::day();
    h.ask(
        question,
        None,
        vec![
            tool_reply(&[("find", json!({"text": name, "role": "partner"}))]),
            tool_reply(&[("report", statement())]),
            final_reply(text),
        ],
    )
}

#[test]
fn a_question_for_the_kul_raqam_is_not_shown_the_capital() {
    for (question, name) in [
        ("alif ki kul raqam kitni hai", "alif"),
        ("الف کی کل رقم کتنی ہے", "الف"),
        ("alif ka kitna hai abhi", "alif"),
        ("how much does the partner alif have now?", "alif"),
    ] {
        let (reply, requests) = ask_partner(question, name, "{{r2.totals.running}}");
        let result = report_result(&requests);
        assert!(
            result.get("capital").is_none(),
            "{question}: the capital is not shown: {result}"
        );
        // The one closing figure is his کل رقم: −1,004, and that is what the answer shows.
        assert_eq!(result["totals"]["running"]["rs"], -1_004, "{question}");
        let shown = figures(answer_of(&reply));
        assert_eq!(money(shown[0].0), -1_004, "{question}: {reply:#?}");
    }
}

#[test]
fn a_question_for_the_capital_is_shown_the_capital() {
    for (question, name) in [
        ("alif ka sarmaya kitna hai", "alif"),
        ("الف کا سرمایہ کتنا ہے", "الف"),
        ("what is partner alif's capital", "alif"),
    ] {
        let (_reply, requests) = ask_partner(question, name, "{{r2.capital.balance}}");
        let result = report_result(&requests);
        assert_eq!(
            result["capital"]["balance"]["rs"], 0,
            "{question}: the capital is shown: {result}"
        );
    }
}

/// R31 for everyone: no partner.
fn overview() -> Value {
    json!({"id": "R31", "adda": null, "as_of": null, "bikri": null, "city": null,
           "collector": null, "from": null, "garden": null, "group_by": null, "item": null,
           "partner": null, "party": null, "rows": null, "sort": null, "to": null,
           "trip": null, "where": null})
}

/// The row of one partner (`s1`) in the keyed rows the model is shown.
fn row_of<'a>(result: &'a Value, key: &str) -> &'a Value {
    result["rows"]
        .as_array()
        .and_then(|rows| rows.iter().find(|r| r["key"] == key))
        .unwrap_or_else(|| panic!("no row {key}: {result}"))
}

fn ask_overview(question: &str, text: &str) -> (AskReply, Vec<Value>) {
    let h = Harness::day();
    h.ask(
        question,
        None,
        vec![tool_reply(&[("report", overview())]), final_reply(text)],
    )
}

#[test]
fn the_row_of_every_partner_has_the_kul_raqam_and_the_capital_only_when_asked() {
    // The overview of the worked day: الف −1,004, ب 746, ج 746 (PRD 4.9, E2E-9).
    let (reply, requests) = ask_overview("sab partner ki kul raqam", "{{r1.rows.s1.total}}");
    let result = report_result(&requests);
    for key in ["s1", "s2", "s3"] {
        assert!(
            row_of(&result, key).get("capital").is_none(),
            "{key}: no capital for a kul raqam question: {result}"
        );
    }
    assert_eq!(row_of(&result, "s1")["total"]["rs"], -1_004);
    assert_eq!(money(figures(answer_of(&reply))[0].0), -1_004);
    // A capital question gets the capital of every partner.
    let (_reply, requests) =
        ask_overview("sab partner ka sarmaya", "{{r1.rows.s2.capital.balance}}");
    let result = report_result(&requests);
    for key in ["s1", "s2", "s3"] {
        assert!(
            row_of(&result, key).get("capital").is_some(),
            "{key}: the capital is shown for a capital question: {result}"
        );
    }
}

/// The report buttons of an answer as (report, partner name) in order.
fn buttons(reply: &AskReply) -> Vec<(String, Option<String>)> {
    answer_of(reply)
        .sources
        .iter()
        .filter_map(|s| match s {
            mandi_assistant::answer::AnswerSource::Report { id, filters, .. } => Some((
                format!("{id:?}"),
                filters.partner.as_ref().map(|p| p.name_en.clone()),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn the_button_of_a_partner_overview_answer_opens_the_partners_it_names_and_never_none() {
    // Q052: three partners cited, three buttons, each with its partner. Before: one R31 button
    // with no partner, which the report screen refused with ERR_REQUIRED.
    let (reply, _) = ask_overview(
        "partner ki kiya total hain abhi tak",
        "{{r1.rows.s1.total}} {{r1.rows.s2.total}} {{r1.rows.s3.total}}",
    );
    assert_eq!(
        buttons(&reply),
        [
            ("R31".to_owned(), Some("Partner A".to_owned())),
            ("R31".to_owned(), Some("Partner B".to_owned())),
            ("R31".to_owned(), Some("Partner C".to_owned())),
        ]
    );
    // Q400: one partner's capital. His capital is read from the capital history and the partner
    // overview, and the R31 statement page does not hold it, so there is NO button (D7 D331, lead
    // ruling: a button opens only the page the figure was read from). Before, one R31 button with him.
    let (reply, _) = ask_overview(
        "Partner B ka capital kitna laga hua hai",
        "{{r1.rows.s2.capital.invested}}",
    );
    assert_eq!(buttons(&reply), []);
}

// ---- the money review of 05-11-2026: the closing is ticked, a dated read is dated, ب after E2E-35

/// R31 for the partner the first `find` returned, up to `to`.
fn statement_to(to: &str) -> Value {
    let mut call = statement();
    call["to"] = json!(to);
    call
}

fn ask_with_calls(question: &str, name: &str, read: Value, text: &str) -> (AskReply, Vec<Value>) {
    let h = Harness::day();
    h.ask(
        question,
        None,
        vec![
            tool_reply(&[("find", json!({"text": name, "role": "partner"}))]),
            tool_reply(&[("report", read)]),
            final_reply(text),
        ],
    )
}

#[test]
fn the_closing_of_a_partners_statement_is_ticked_and_the_newest_line_comes_first() {
    // His کل رقم is read in two places (the statement closing and the partner overview), as the
    // capital is: without a tick on it the capital looked checked and was cited for a کل رقم
    // question (Q198 to Q199). The rows come newest first, so the first row's `running` is his
    // total now and the 20 rows shown are the latest.
    let (_reply, requests) = ask_partner("alif ki kul raqam kitni hai", "alif", "x");
    let result = report_result(&requests);
    assert_eq!(result["totals"]["running"]["rs"], -1_004);
    assert_eq!(result["totals"]["running"]["checked"], true, "{result}");
    assert_eq!(
        result["summary"]["partners_statement_closing"]["rs"],
        -1_004
    );
    assert_eq!(
        result["summary"]["partners_statement_closing"]["checked"],
        true
    );
    assert_eq!(result["rows"][0]["running"]["rs"], -1_004, "newest first");
}

#[test]
fn a_dated_statement_is_not_ticked_and_a_dated_capital_question_is_told_there_is_no_capital() {
    // To an earlier day there is no second place, so no tick. His capital is worked out for now only.
    let (_reply, requests) = ask_with_calls(
        "alif ka 09-10 tak ka sarmaya",
        "alif",
        statement_to("2026-10-09"),
        "x",
    );
    let result = report_result(&requests);
    assert!(
        result["totals"]["running"].get("checked").is_none(),
        "{result}"
    );
    assert!(result["capital"].get("balance").is_none(), "{result}");
    assert!(result["capital"]["note"].is_string(), "{result}");
}

#[test]
fn the_overview_to_an_earlier_day_gives_that_day_and_not_todays_total_with_a_tick() {
    // Money review BUG 1: R31 for everyone ignored `to` and showed today's totals, ticked.
    // To 09-10-26 nobody had earned anything yet: 0 for each, no tick, and the day is echoed.
    let mut call = overview();
    call["to"] = json!("2026-10-09");
    let h = Harness::day();
    let (_reply, requests) = h.ask(
        "09-10 tak sab partner ki kul raqam",
        None,
        vec![
            tool_reply(&[("report", call)]),
            final_reply("{{r1.rows.s1.total}}"),
        ],
    );
    let result = report_result(&requests);
    for key in ["s1", "s2", "s3"] {
        let total = &row_of(&result, key)["total"];
        assert_eq!(total["rs"], 0, "{key}: {result}");
        assert!(
            total.get("checked").is_none(),
            "{key}: no tick for an earlier day"
        );
    }
    assert_eq!(result["filters"]["to"], "2026-10-09");
    assert!(
        result["note"].is_string(),
        "the note says it is not checked"
    );
    // And today's is ticked: الف −1,004 (the test above).
}

#[test]
fn b_after_e2e_35_has_a_capital_of_50000_and_a_kul_raqam_of_50746_and_each_question_gets_its_own() {
    // PRD E2E-35 and E2E-37: ب invests 50,000 on 11-10-26; his کل رقم is 50,746 (746 of profit
    // share), his capital 50,000. The live run answered the کل رقم question with the capital.
    let h = Harness::day();
    common::set_today(&h.world.env, 2026, 10, 11);
    mandi_services::vouchers::save_capital_in(
        &h.world.env.ctx,
        &h.world.owner,
        &mandi_services::core::capital::CapitalInput {
            date: "11-10-2026".into(),
            partner_id: Some(h.world.partners[1]),
            amount: "50,000".into(),
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let read = |question: &str, text: &str| {
        h.ask(
            question,
            None,
            vec![
                tool_reply(&[("find", json!({"text": "Partner B", "role": "partner"}))]),
                tool_reply(&[("report", statement())]),
                final_reply(text),
            ],
        )
    };
    // The کل رقم question: no capital block; the closing 50,746, ticked.
    let (reply, requests) = read("ب کی کل رقم کتنی ہے", "{{r2.totals.running}}");
    let result = report_result(&requests);
    assert!(result.get("capital").is_none(), "{result}");
    assert_eq!(result["totals"]["running"]["rs"], 50_746);
    assert_eq!(result["totals"]["running"]["checked"], true);
    assert_eq!(money(figures(answer_of(&reply))[0].0), 50_746);
    // The capital question: his capital 50,000, ticked, and the closing is still there, 50,746.
    let (reply, requests) = read("ب کا سرمایہ کتنا ہے", "{{r2.capital.balance}}");
    let result = report_result(&requests);
    assert_eq!(result["capital"]["balance"]["rs"], 50_000);
    assert_eq!(result["capital"]["balance"]["checked"], true);
    assert_eq!(result["totals"]["running"]["rs"], 50_746);
    assert_eq!(money(figures(answer_of(&reply))[0].0), 50_000);
}
