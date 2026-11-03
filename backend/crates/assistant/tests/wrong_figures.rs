//! The wrong figures of the full live run of 03-11-2026 (plan 52 P52-S1, class 1): one recorded
//! test for each question. The recorded model takes the route the live model took; the tools now
//! lead it to the right source. The worked day is the data unless a test says otherwise.
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

use harness::{Harness, answer_of, figures};
use mandi_assistant::ask::AskOutcome;
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

/// An `analytics` call with every field the schema holds.
fn analytics(view: &str, set: Value) -> (&'static str, Value) {
    let mut call = json!({
        "view": view, "from": null, "to": null, "group_by": null, "item": null, "seller": null,
        "buyer": null, "city": null, "adda": null, "garden": null, "trip": null, "goods": null,
        "partner": null, "side": null, "compare": null, "sort": null, "rows": null
    });
    for (k, v) in set.as_object().unwrap() {
        call[k] = v.clone();
    }
    ("analytics", call)
}

/// A `report` call with every field the schema holds.
fn report(id: &str, set: Value) -> (&'static str, Value) {
    let mut call = json!({
        "id": id, "adda": null, "as_of": null, "bikri": null, "city": null, "collector": null,
        "from": null, "garden": null, "group_by": null, "item": null, "partner": null,
        "party": null, "rows": null, "sort": null, "to": null, "trip": null, "where": null
    });
    for (k, v) in set.as_object().unwrap() {
        call[k] = v.clone();
    }
    ("report", call)
}

/// The tool messages of a request, as parsed JSON, by result id.
fn result_of(body: &Value, id: &str) -> Value {
    body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .find(|v| v["id"] == id)
        .unwrap_or_else(|| panic!("no result {id} in {body}"))
}

fn percent(cell: &mandi_reports::reports::Cell) -> i64 {
    match cell {
        mandi_reports::reports::Cell::Percent { hundredths } => *hundredths,
        other => panic!("not a percent: {other:?}"),
    }
}

fn number(cell: &mandi_reports::reports::Cell) -> i64 {
    harness::count(cell)
}

fn note_of(result: &Value) -> String {
    result["note"].as_str().unwrap_or("").to_owned()
}

// ---- Q029: the share of banana in all commission ------------------------------------------------

#[test]
fn q029_a_view_filtered_to_one_item_gives_no_share_and_the_unfiltered_view_gives_72_10() {
    // The live model read view A with the item filter and cited its share, 100%. The filtered
    // table has one row, so the share column is left out and the note says where the share of ALL
    // commission is.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "what share of all commission is banana",
        None,
        vec![
            tool_reply(&[
                find("banana", "item"),
                analytics("A", json!({"item": "r1.matches.0"})),
            ]),
            tool_reply(&[analytics("A", json!({}))]),
            final_reply("{{r1.matches.0.name}} کا حصہ {{r3.rows.i1.share}} ہے۔"),
        ],
    );
    let filtered = result_of(&requests[1], "r2");
    assert!(filtered["totals"].get("share").is_none(), "{filtered}");
    assert!(filtered["rows"][0].get("share").is_none(), "{filtered}");
    assert!(note_of(&filtered).contains("view H"), "{filtered}");
    assert!(note_of(&filtered).contains("without the item filter"));
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    assert_eq!(percent(figs[0].0), 7_210, "72.10%");
}

#[test]
fn q029_a_view_filtered_by_a_seller_with_several_items_keeps_its_shares() {
    // Not trivial: a seller who sent two items has two rows, and their shares are of his commission.
    // Banana 10 × 1,000 in August (1,280), apple 20 × 1,000 in September (2,560): 33.33 and 66.67.
    let h = Harness::over(worked::months_world(&[
        (8, true, 10, 1_000),
        (9, false, 20, 1_000),
    ]));
    let (_, requests) = h.ask(
        "rahim ke maal mein kis item ka kitna hissa hai",
        None,
        vec![
            tool_reply(&[
                find("rahim", "seller"),
                analytics(
                    "A",
                    json!({"seller": "r1.matches.0", "from": "2026-08-01", "to": "2026-09-30"}),
                ),
            ]),
            final_reply("{{r1.matches.0.name}}"),
        ],
    );
    let result = result_of(&requests[1], "r2");
    assert_eq!(result["count"]["n"], 2, "{result}");
    assert!(result["rows"][0].get("share").is_some(), "{result}");
    assert!(result["totals"].get("share").is_some(), "{result}");
    // The share is kept, with a note that it is of THIS table, and where the share of all is.
    assert!(note_of(&result).contains("THIS filtered table"), "{result}");
}

// ---- Q061: the cost of a market trip ------------------------------------------------------------

#[test]
fn q061_r29_says_a_trips_cost_is_not_in_it_and_where_it_is() {
    let h = Harness::day();
    let (_, requests) = h.ask(
        "mandi safar mein kitna kharcha hua",
        None,
        vec![
            tool_reply(&[report("R29", json!({}))]),
            tool_reply(&[analytics("J", json!({}))]),
            final_reply("سفر کا کوئی اندراج نہیں۔"),
        ],
    );
    let r29 = result_of(&requests[1], "r1");
    let note = note_of(&r29);
    assert!(note.contains("R33") && note.contains("view J"), "{note}");
    // The second call, view J, is read: the model got to the right source.
    let j = result_of(&requests[2], "r2");
    assert_eq!(j["source"], "J", "{j}");
}

// ---- Q065: how much of a seller's goods is left -------------------------------------------------

#[test]
fn q065_a_seller_and_goods_left_reads_his_stock_not_only_his_payable() {
    // E2E-16: 5 پیٹی کیلا of رحیم are left. The card (his payable) is r2 as before, R10 for him r3.
    let h = Harness::over(worked::part_sold_world());
    let (reply, requests) = h.ask(
        "رحیم کے مال میں کتنا باقی ہے",
        None,
        vec![
            tool_reply(&[find("رحیم", "seller")]),
            final_reply("{{r1.matches.0.name}} کے {{r3.totals.balance}} نگ باقی ہیں۔"),
        ],
    );
    assert_eq!(requests.len(), 2, "find, then the answer");
    let stock = result_of(&requests[1], "r3");
    assert_eq!(stock["source"], "R10", "{stock}");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    assert_eq!(number(figs[0].0), 5);
}

#[test]
fn q065_a_balance_question_about_a_seller_reads_no_stock() {
    let h = Harness::over(worked::part_sold_world());
    let (_, requests) = h.ask(
        "rahim ka kitna baqi hai",
        None,
        vec![
            tool_reply(&[find("rahim", "seller")]),
            final_reply("{{r1.matches.0.name}} {{r2.balance}}"),
        ],
    );
    let ids: Vec<String> = requests[1]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .map(|m| {
            serde_json::from_str::<Value>(m["content"].as_str().unwrap_or("{}")).unwrap()["id"]
                .as_str()
                .unwrap_or("")
                .to_owned()
        })
        .collect();
    assert_eq!(ids, ["r1", "r2"], "the card only");
}

// ---- Q146: September against August -------------------------------------------------------------

#[test]
fn q146_a_whole_month_is_compared_with_the_whole_month_before() {
    // August 1,280 (10 × 1,000: 600 + 600 + 80), September 2,560 (20 × 1,000): 1,280 more. The
    // compared period of 01-09 to 30-09 is 01-08 to 31-08, not the 30 days before it.
    let h = Harness::over(worked::months_world(&[
        (8, true, 10, 1_000),
        (9, true, 20, 1_000),
    ]));
    let (reply, requests) = h.ask(
        "اگست کے مقابلے میں ستمبر کا کمیشن کتنا زیادہ یا کم رہا",
        None,
        vec![
            // September is a period the model reads first (r1), as in the live run.
            tool_reply(&[
                (
                    "period",
                    json!({"kind": "month", "month": 9, "day": null, "from": null, "to": null,
                           "n": null}),
                ),
                analytics(
                    "A",
                    json!({"from": "r1.from", "to": "r1.to", "compare": "previous"}),
                ),
            ]),
            final_reply("کمیشن پچھلے مہینے سے {{r2.compare.difference.total_comm}} زیادہ رہا۔"),
        ],
    );
    let result = result_of(&requests[1], "r2");
    assert_eq!(result["compare"]["from"]["date"], "2026-08-01", "{result}");
    assert_eq!(result["compare"]["to"]["date"], "2026-08-31", "{result}");
    assert_eq!(result["compare"]["totals"]["total_comm"]["rs"], 1_280);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(harness::money(figures(answer_of(&reply))[0].0), 1_280);
}

// ---- Q152: how many بکری ------------------------------------------------------------------------

#[test]
fn q152_how_many_bikriyan_is_read_from_view_b_in_one_model_call() {
    // The worked day has 2 بکری and 3 sale lines. The app reads the period and view B before the
    // first call; the answer is the bikri_count total, not the lines of R2.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "is mahine kitni bikriyan hui",
        None,
        vec![final_reply(
            "اس مہینے {{r2.totals.bikri_count}} بکریاں ہوئیں۔",
        )],
    );
    assert_eq!(requests.len(), 1, "one model call");
    let b = result_of(&requests[0], "r2");
    assert_eq!(b["source"], "B", "{b}");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(number(figures(answer_of(&reply))[0].0), 2);
}

#[test]
fn q152_a_question_with_the_singular_word_is_not_read_as_a_count() {
    // "aaj ki bikri kitni hui" asks for the amount: no view B is read ahead.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "is mahine ki bikri kitni hui",
        None,
        vec![final_reply("x"), final_reply("x")],
    );
    let ids: Vec<String> = requests[0]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .map(|m| {
            serde_json::from_str::<Value>(m["content"].as_str().unwrap_or("{}")).unwrap()["id"]
                .as_str()
                .unwrap_or("")
                .to_owned()
        })
        .collect();
    assert_eq!(ids, ["r1"], "the period only");
}

#[test]
fn q152_r2_says_its_count_is_sale_lines_and_where_the_bikri_count_is() {
    let h = Harness::day();
    let (_, requests) = h.ask(
        "show the register",
        None,
        vec![tool_reply(&[report("R2", json!({}))]), final_reply("x")],
    );
    let r2 = result_of(&requests[1], "r1");
    let note = note_of(&r2);
    assert!(
        note.contains("sale LINES") && note.contains("bikri_count"),
        "{note}"
    );
    assert_eq!(r2["count"]["n"], 3, "3 sale lines, 2 بکری");
}

// ---- Q194: the biggest expense head -------------------------------------------------------------

#[test]
fn q194_a_sorted_list_never_holds_the_heading_or_total_line_as_a_row() {
    // R29 has a heading line with the total of all shop expenses. Sorted by expense, the live
    // model read that line as the biggest head. Now the heading lines are given apart, in
    // `headings`, and the rows are the lines only.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "is saal kis kharche mein sab se zyada paisa gaya",
        None,
        vec![
            tool_reply(&[report(
                "R29",
                json!({"from": "2026-01-01", "to": "2026-10-10", "rows": 100,
                       "sort": [{"column": "expense", "order": "desc"}]}),
            )]),
            final_reply("{{r2.rows.a31.name}} میں سب سے زیادہ گئے: {{r2.rows.a31.expense}}۔"),
        ],
    );
    let result = requests[1]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .find(|v| v["source"] == "R29")
        .unwrap();
    let is_heading = |k: &str| k.starts_with('h') && k[1..].bytes().all(|b| b.is_ascii_digit());
    let keys: Vec<&str> = result["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["key"].as_str().unwrap())
        .collect();
    assert!(keys.iter().all(|k| !is_heading(k)), "{keys:?}");
    // The heading lines are still there, apart, with their sums.
    let headings = result["headings"].as_array().unwrap();
    assert!(!headings.is_empty(), "{result}");
    assert!(
        headings
            .iter()
            .all(|r| is_heading(r["key"].as_str().unwrap()))
    );
    assert!(note_of(&result).contains("`headings`"));
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    assert_eq!(harness::money(figs[0].0), 3_000);
}

// ---- Q202: R31 with dates -----------------------------------------------------------------------

#[test]
fn q202_a_partner_statement_with_dates_is_read() {
    // Every R31 call with a from or to date was refused with ERR_DATE_INVALID: the statement
    // service takes DD-MM-YYYY and was given the ISO dates of the filters.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "الف نے اس سال ذاتی خرچ کے لیے کتنا لیا اور کتنا واپس کیا",
        None,
        vec![
            tool_reply(&[
                find("الف", "partner"),
                report(
                    "R31",
                    json!({"partner": "r2.matches.0", "from": "r1.from", "to": "r1.to"}),
                ),
            ]),
            final_reply("{{r2.matches.0.name}} نے {{r3.rows.je9.amount}} لیے، کچھ واپس نہیں کیا۔"),
        ],
    );
    let statement = result_of(&requests[1], "r3");
    assert_eq!(statement["count"]["n"], 4, "{statement}");
    assert_eq!(statement["filters"]["from"], "2026-01-01");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    // The 2,000 he took is a line of his statement with its minus sign.
    assert_eq!(harness::money(figures(answer_of(&reply))[0].0), -2_000);
}

// ---- Q265, Q267: the pace doubles or triples ----------------------------------------------------

/// Six months of banana sales, April to September, for the estimate questions.
fn estimate_world() -> Harness {
    Harness::over(worked::history_world(&[
        (4, 10, 1_000),
        (5, 12, 1_000),
        (6, 14, 1_000),
        (7, 16, 1_000),
        (8, 18, 1_000),
        (9, 20, 1_000),
    ]))
}

fn estimate_args(set: Value) -> (&'static str, Value) {
    let mut call = json!({
        "measure": "commission_all", "item": null, "seller": null, "buyer": null,
        "garden": null, "trip": null, "ahead": null, "until": null, "what_if": null,
        "percent": null, "target": null
    });
    for (k, v) in set.as_object().unwrap() {
        call[k] = v.clone();
    }
    ("estimate", call)
}

#[test]
fn q265_and_q267_english_doubles_and_triples_are_worked_out() {
    for (question, word, factor) in [
        (
            "if the pace doubles how much commission by the end of the year",
            "double",
            2,
        ),
        (
            "if the pace triples how much commission by the end of the year",
            "triple",
            3,
        ),
    ] {
        let h = estimate_world();
        let (_, requests) = h.ask(
            question,
            None,
            vec![
                tool_reply(&[estimate_args(json!({"until": "year_end", "what_if": word}))]),
                final_reply("x"),
                final_reply("x"),
            ],
        );
        let result = result_of(&requests[1], "e1");
        assert!(result.get("error").is_none(), "{result}");
        let same = result["ahead"][0]["same_pace"]["rs"].as_i64().unwrap();
        let what_if = result["ahead"][0]["what_if_pace"]["rs"].as_i64().unwrap();
        assert_eq!(what_if, same * factor, "{word}");
    }
}

// ---- Q285, Q286: "ek saal" is 12 months ---------------------------------------------------------

#[test]
fn q285_and_q286_one_year_in_words_is_twelve_months_ahead() {
    for question in [
        "agle ek saal ka commission ka andaza lagao",
        "اگلے ایک سال کے کمیشن کا اندازہ لگائیں",
    ] {
        let h = estimate_world();
        let (_, requests) = h.ask(
            question,
            None,
            vec![
                tool_reply(&[estimate_args(json!({"ahead": 12}))]),
                final_reply("x"),
                final_reply("x"),
            ],
        );
        let result = result_of(&requests[1], "e1");
        assert!(result.get("error").is_none(), "{question}: {result}");
        assert_eq!(result["ahead_count"]["n"], 12, "{result}");
    }
    // Without the words, 12 months is still refused: the Owner said no such thing.
    let h = estimate_world();
    let (_, requests) = h.ask(
        "agle mahino ka commission ka andaza lagao",
        None,
        vec![
            tool_reply(&[estimate_args(json!({"ahead": 12}))]),
            final_reply("x"),
            final_reply("x"),
        ],
    );
    let refused = requests[1]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .any(|m| {
            m["content"]
                .as_str()
                .unwrap_or("")
                .contains("ERR_OUT_OF_RANGE")
        });
    assert!(refused);
}

// ---- Q290, Q291: a figure and a percent more ---------------------------------------------------

#[test]
fn q290_and_q291_a_figure_and_ten_percent_more() {
    // This year's commission so far is 5,448 (E2E-11); 10% more is 5,992.8, which is 5,993.
    for question in [
        "agar is saal commission 10 percent zyada hota to kitna hota",
        "if this year's commission had been 10 percent higher, what would it be",
    ] {
        let h = Harness::day();
        let (reply, requests) = h.ask(
            question,
            None,
            vec![
                tool_reply(&[
                    ("period", this_year()),
                    analytics("A", json!({"from": "r1.from", "to": "r1.to"})),
                ]),
                tool_reply(&[(
                    "calculate",
                    json!({"op": "percent_more", "inputs": ["r2.totals.total_comm", "q.n1"]}),
                )]),
                final_reply("{{r2.totals.total_comm}} {{c1.result}}"),
            ],
        );
        let calc = result_of(&requests[2], "c1");
        assert!(calc.get("error").is_none(), "{question}: {calc}");
        assert_eq!(
            reply.outcome,
            AskOutcome::Answered,
            "{question}: {reply:#?}"
        );
        let figs = figures(answer_of(&reply));
        assert_eq!(harness::money(figs[0].0), 5_448);
        assert_eq!(harness::money(figs[1].0), 5_993, "{question}");
    }
}

#[test]
fn q290_percent_more_needs_the_word_more_and_a_percent_of_the_question() {
    // "less" in the question and the op percent_more: refused. A percent that is not the Owner's
    // (a path) is refused too.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "agar is saal commission 10 percent kam hota to kitna hota",
        None,
        vec![
            tool_reply(&[
                ("period", this_year()),
                analytics("A", json!({"from": "r1.from", "to": "r1.to"})),
            ]),
            tool_reply(&[(
                "calculate",
                json!({"op": "percent_more", "inputs": ["r2.totals.total_comm", "q.n1"]}),
            )]),
            tool_reply(&[(
                "calculate",
                json!({"op": "percent_less", "inputs": ["r2.totals.total_comm", "q.n1"]}),
            )]),
            final_reply("{{r2.totals.total_comm}} {{c1.result}}"),
        ],
    );
    let refused = tool_error_codes(&requests[2]);
    assert!(
        refused.contains(&"ERR_OUT_OF_RANGE".to_owned()),
        "{refused:?}"
    );
    let less = result_of(&requests[3], "c1");
    assert!(less.get("error").is_none(), "{less}");
}

fn this_year() -> Value {
    json!({"kind": "this_year", "month": null, "day": null, "from": null, "to": null, "n": null})
}

fn tool_error_codes(body: &Value) -> Vec<String> {
    body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .filter_map(|v| v["error"]["code"].as_str().map(str::to_owned))
        .collect()
}

#[test]
fn q065_a_money_word_beside_goods_and_left_keeps_it_a_balance_question() {
    // "رحیم کے مال کے پیسے کتنے باقی ہیں" asks what he is owed: his card only, no stock.
    let h = Harness::over(worked::part_sold_world());
    let (_, requests) = h.ask(
        "رحیم کے مال کے پیسے کتنے باقی ہیں",
        None,
        vec![
            tool_reply(&[find("رحیم", "seller")]),
            final_reply("{{r1.matches.0.name}} {{r2.balance}}"),
        ],
    );
    let ids: Vec<String> = requests[1]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .map(|m| {
            serde_json::from_str::<Value>(m["content"].as_str().unwrap_or("{}")).unwrap()["id"]
                .as_str()
                .unwrap_or("")
                .to_owned()
        })
        .collect();
    assert_eq!(ids, ["r1", "r2"], "the card only");
}

// ---- Q055, Q070, Q130, Q209: finding the right record -------------------------------------------

#[test]
fn q055_a_partner_asked_for_as_a_party_is_found_as_a_partner() {
    let h = Harness::day();
    let (_, requests) = h.ask(
        "alif ka khaalis kitna hai",
        None,
        vec![tool_reply(&[find("alif", "any_party")]), final_reply("x")],
    );
    let found = result_of(&requests[1], "r1");
    assert_eq!(found["widened"], "partner", "{found}");
    assert_eq!(found["matches"][0]["key"], "s1", "{found}");
    assert!(
        note_of(&found).contains("no any_party has this name"),
        "{found}"
    );
}

#[test]
fn q070_two_buyers_asked_for_as_sellers_are_found_as_buyers() {
    let h = Harness::day();
    let (_, requests) = h.ask(
        "arshad akram se kitna zyada deta hai",
        None,
        vec![
            tool_reply(&[find("arshad", "seller"), find("akram", "seller")]),
            final_reply("x"),
        ],
    );
    for id in ["r1", "r2"] {
        let found = result_of(&requests[1], id);
        assert_eq!(found["widened"], "buyer", "{found}");
        assert_eq!(found["matches"][0]["type"], "buyer", "{found}");
    }
    // A name that is a seller found as a seller is not widened.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "rahim kitna dega",
        None,
        vec![tool_reply(&[find("rahim", "seller")]), final_reply("x")],
    );
    assert!(result_of(&requests[1], "r1").get("widened").is_none());
}

#[test]
fn q070_a_typed_seller_filter_never_takes_a_buyer_in_its_place() {
    // analytics view B with seller "akram" (a buyer): the typed name is looked up, found only as a
    // buyer, and the call is not run for the buyer: the Owner is asked which one, with no second
    // model call.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "akram ke maal ka commission",
        None,
        vec![
            tool_reply(&[analytics("B", json!({"seller": "akram"}))]),
            final_reply("x"),
        ],
    );
    assert_eq!(requests.len(), 1, "no second model call");
    assert_eq!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
}

#[test]
fn q130_the_oblique_plural_of_an_item_names_the_item() {
    // "کیلے کا کمیشن": کیلے names کیلا; the typed item is taken as the only match.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "کیلے کا کمیشن اس سال کتنا ہے",
        None,
        vec![
            tool_reply(&[
                ("period", this_year()),
                analytics(
                    "A",
                    json!({"item": "کیلے", "from": "r1.from", "to": "r1.to"}),
                ),
            ]),
            final_reply("x"),
            final_reply("x"),
        ],
    );
    assert!(
        tool_error_codes(&requests[1]).is_empty(),
        "{:?}",
        tool_error_codes(&requests[1])
    );
    let a = result_of(&requests[1], "r2");
    assert_eq!(a["rows"][0]["name"]["en"], "Banana", "{a}");
    assert!(note_of(&a).contains("taken as the only match"), "{a}");
}

// ---- Q237, Q313: a call that timed out is tried once more --------------------------------------

#[test]
fn q237_a_call_that_timed_out_is_tried_once_more_and_the_question_is_answered() {
    use mandi_assistant::ask::FailureKind;
    use mandi_assistant::client::{CallError, RecordedClient};
    let h = Harness::day();
    // The first call finds the buyer, the second times out, the retry answers.
    let client = RecordedClient::with_results(vec![
        Ok(tool_reply(&[find("akram", "buyer")])),
        Err(CallError::new(FailureKind::Timeout)),
        Ok(final_reply("{{r1.matches.0.name}} {{r2.balance}}")),
    ]);
    let reply = h.ask_with("how much does akram owe", None, &client);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(
        client.requests().len(),
        3,
        "the timed-out call was sent again"
    );
    assert_eq!(harness::balance(figures(answer_of(&reply))[0].0).0, 2_880);
    // Two timeouts in a row end the question: one retry only.
    let h = Harness::day();
    let client = RecordedClient::with_results(vec![
        Err(CallError::new(FailureKind::Timeout)),
        Err(CallError::new(FailureKind::Timeout)),
        Ok(final_reply("x")),
    ]);
    let reply = h.ask_with("how much does akram owe", None, &client);
    assert_eq!(reply.outcome, AskOutcome::Error, "{reply:#?}");
    assert_eq!(client.requests().len(), 2);
}

// ---- Q329: the shortcut keys ---------------------------------------------------------------------

#[test]
fn q329_a_long_keys_answer_with_number_words_is_written_from_the_guide() {
    // The live model listed the shortcuts in its own words: number words and too many words, twice.
    // The app writes the answer from the guide of the keys topic instead (one model call).
    let long = "کی بورڈ کی شارٹ کٹس یہ ہیں: ".to_owned()
        + &"ایک دو تین چار پانچ چھ سات آٹھ نو دس ".repeat(20);
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "which keyboard shortcuts are there",
        None,
        vec![final_reply(&long), final_reply(&long)],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 1, "no rewrite call");
    // Only the guide's own words; the app's checks passed it (references count as one word).
    let answer = answer_of(&reply);
    assert!(!answer.parts.is_empty());
    assert!(
        answer
            .parts
            .iter()
            .all(|p| matches!(p, mandi_assistant::answer::AnswerPart::Text { .. })),
        "{:?}",
        answer.parts
    );
}

// ---- Q247: how to grow the sales is advice, not a screen guide ----------------------------------

#[test]
fn q247_how_to_increase_the_sales_is_advice_and_reads_no_guide() {
    for question in [
        "بکری کیسے بڑھائیں",
        "bikri kaise barhayen",
        "bikri kaise barhao",
        "how do I increase the sales",
    ] {
        assert_eq!(
            mandi_assistant::help_guard::topic_of_question(question, &[]),
            None,
            "{question}"
        );
    }
    // A real how-to still reads the guide.
    assert_eq!(
        mandi_assistant::help_guard::topic_of_question("bikri kaise add karun", &[]),
        Some("bikri_book")
    );
}

// ---- Q205, Q206: the stock tile with a path that goes on below the value ------------------------

#[test]
fn q205_and_q206_a_path_that_goes_on_below_a_value_is_the_value() {
    // E2E-16: 5 پیٹی in 1 lot are left. The model wrote `.n` after the counts.
    for question in ["baki mal kitna he", "باقی مال کتنا ہے"] {
        let h = Harness::over(worked::part_sold_world());
        let (reply, requests) = h.ask(
            question,
            None,
            vec![final_reply(
                "{{r1.stock.lots.n}} لاٹ میں {{r1.stock.qty.n}} نگ باقی ہیں۔",
            )],
        );
        assert_eq!(requests.len(), 1, "{question}: the tiles were read ahead");
        assert_eq!(
            reply.outcome,
            AskOutcome::Answered,
            "{question}: {reply:#?}"
        );
        let figs = figures(answer_of(&reply));
        assert_eq!((number(figs[0].0), number(figs[1].0)), (1, 5), "{question}");
    }
}

// ---- the second money review of this step --------------------------------------------------------

#[test]
fn a_percent_more_needs_a_what_if_one_percent_and_the_word_right_after_it() {
    // A past fact and a reverse question ("this year is 10% more than last year, what was last
    // year"), and a rate ("if the commission were 7%, how much more would I get"): none of them is
    // a figure to scale, so the op is refused.
    for question in [
        "is saal ka commission pichle saal se 10 percent zyada hai to pichle saal kitna tha",
        "agar commission 7 percent hota to kitna zyada milta",
        "agar 6 percent ki jagah 10 percent zyada hota to kitna hota",
    ] {
        let h = Harness::day();
        let (_, requests) = h.ask(
            question,
            None,
            vec![
                tool_reply(&[
                    ("period", this_year()),
                    analytics("A", json!({"from": "r1.from", "to": "r1.to"})),
                ]),
                tool_reply(&[(
                    "calculate",
                    json!({"op": "percent_more", "inputs": ["r2.totals.total_comm", "q.n1"]}),
                )]),
                final_reply("x"),
                final_reply("x"),
            ],
        );
        let codes = tool_error_codes(&requests[2]);
        assert_eq!(codes, ["ERR_OUT_OF_RANGE"], "{question}: {codes:?}");
    }
}

#[test]
fn a_seller_field_never_takes_a_buyers_key_and_the_reverse() {
    // A buyer found by a seller search (widened) is passed on as the seller of a view: refused,
    // with the type said, never a seller's report of nothing.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "arshad ka maal kitna bika",
        None,
        vec![
            tool_reply(&[
                find("arshad", "seller"),
                analytics("B", json!({"seller": "r1.matches.0"})),
                analytics("C", json!({"buyer": "r1.matches.0"})),
            ]),
            final_reply("x"),
            final_reply("x"),
        ],
    );
    let messages: Vec<Value> = requests[1]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .collect();
    // The find, then the refused seller view, then the buyer view that works (he IS a buyer).
    assert_eq!(messages[0]["widened"], "buyer");
    assert_eq!(
        messages[1]["error"]["code"], "ERR_FORMAT_INVALID",
        "{:?}",
        messages[1]
    );
    assert!(
        messages[1]["error"]["detail"]
            .as_str()
            .unwrap()
            .contains("is a buyer and not a seller"),
        "{:?}",
        messages[1]
    );
    assert_eq!(messages[2]["source"], "C", "{:?}", messages[2]);
}

#[test]
fn nothing_is_read_ahead_for_a_widened_find() {
    // "arshad" is a buyer: a seller search is widened, and the app does not read his card ahead.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "arshad ko kitna dena hai",
        None,
        vec![tool_reply(&[find("arshad", "seller")]), final_reply("x")],
    );
    let ids: Vec<String> = requests[1]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .map(|m| {
            serde_json::from_str::<Value>(m["content"].as_str().unwrap_or("{}")).unwrap()["id"]
                .as_str()
                .unwrap_or("")
                .to_owned()
        })
        .collect();
    assert_eq!(ids, ["r1"], "the find only");
}

#[test]
fn a_sum_over_the_rows_of_a_sorted_report_is_not_refused_for_the_headings_kept_apart() {
    // R29 sorted by expense: its heading lines are in `headings`, so its count (which holds them)
    // is still met by the rows and the headings sent.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "kharche ka total",
        None,
        vec![
            tool_reply(&[report(
                "R29",
                json!({"rows": 100, "sort": [{"column": "expense", "order": "desc"}]}),
            )]),
            tool_reply(&[(
                "calculate",
                json!({"op": "sum", "inputs": ["r1.rows.*.taken"]}),
            )]),
            final_reply("x"),
            final_reply("x"),
        ],
    );
    let refused: Vec<String> = requests[2]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .filter_map(|v| v["error"]["detail"].as_str().map(str::to_owned))
        .collect();
    assert!(
        refused
            .iter()
            .all(|d| !d.contains("not every row was read")),
        "{refused:?}"
    );
}
