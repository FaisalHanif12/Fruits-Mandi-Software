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

// ==== Plan 52 item 14: the defects of live run 2 (03-11-2026, 369 questions) =====================

/// E2E-59: banana sales April to September 2026, their commission 36,000 ... 51,000.
fn e2e_59() -> Harness {
    Harness::over(worked::history_world(&[
        (4, 180, 1_600),
        (5, 150, 2_100),
        (6, 120, 2_850),
        (7, 225, 1_600),
        (8, 150, 2_600),
        (9, 173, 2_390),
    ]))
}

// ---- 4. Q143, Q144: the month with the most commission ------------------------------------------

#[test]
fn q143_q144_the_top_month_is_named_with_its_amount() {
    // The live model wrote a path on a month key (`series.m202606.month`) and it was refused: the
    // answer ended as checks failed, or as "see the chart". The series is keyed by month now, and
    // `peak` and `lowest` name the month with the most and the least.
    for text in [
        "{{r2.peak.month}} میں سب سے زیادہ کمیشن رہا: {{r2.peak.value}}۔",
        "{{r2.series.m202609.month}} میں سب سے زیادہ کمیشن رہا: {{r2.series.m202609.value}}۔",
        // The old guess: a path that goes on below the month.
        "{{r2.series.m202609.month.month}} میں {{r2.series.m202609.value}}۔",
    ] {
        let h = e2e_59();
        let (reply, requests) = h.ask(
            "kis mahine mein sab se zyada commission aaya",
            None,
            vec![
                tool_reply(&[
                    ("period", this_year()),
                    analytics(
                        "D",
                        json!({"group_by": "month", "from": "r1.from", "to": "r1.to"}),
                    ),
                ]),
                final_reply(text),
            ],
        );
        let result = result_of(&requests[1], "r2");
        assert_eq!(result["peak"]["key"], "m202609", "{result}");
        assert_eq!(result["lowest"]["key"], "m202604", "{result}");
        assert_eq!(result["peak"]["value"]["rs"], 51_000, "{result}");
        assert_eq!(result["lowest"]["value"]["rs"], 36_000, "{result}");
        assert_eq!(reply.outcome, AskOutcome::Answered, "{text}: {reply:#?}");
        let parts = &answer_of(&reply).parts;
        assert!(
            parts.iter().any(|p| matches!(
                p,
                mandi_assistant::answer::AnswerPart::Month {
                    year: 2026,
                    month: 9
                }
            )),
            "{parts:?}"
        );
        assert_eq!(harness::money(figures(answer_of(&reply))[0].0), 51_000);
    }
}

// ---- 8. Q115, Q090, Q242: an item with one clear match does not ask "which one" ------------------

#[test]
fn q115_q090_q242_an_item_typed_as_an_alias_or_a_plural_is_taken_as_the_one_item() {
    let h = Harness::day();
    world::add_item(
        &h.world.env,
        &h.world.owner,
        "موسمی",
        "Mosambi",
        "fruit",
        "unit",
    );
    for (typed, sold) in [
        ("mausami", None),
        ("apples", Some("Apple")),
        ("Apples", Some("Apple")),
        ("bananas", Some("Banana")),
    ] {
        let (reply, requests) = h.ask(
            "is it worth raising the commission on this item",
            None,
            vec![
                tool_reply(&[analytics("A", json!({"item": typed}))]),
                final_reply("ٹھیک ہے۔"),
            ],
        );
        // Not asked which one: the model got the result and answered.
        assert_eq!(requests.len(), 2, "{typed}: {reply:#?}");
        assert_ne!(reply.outcome, AskOutcome::WhichOne, "{typed}");
        let result = result_of(&requests[1], "r1");
        assert!(
            note_of(&result).contains("taken as the only match"),
            "{typed}: {result}"
        );
        if let Some(name) = sold {
            assert_eq!(result["rows"][0]["name"]["en"], name, "{typed}: {result}");
        }
    }
    // A word that only looks like a plural names nothing: the Owner is asked or told.
    let (reply, _) = h.ask(
        "commission on this item",
        None,
        vec![
            tool_reply(&[analytics("A", json!({"item": "applesauces"}))]),
            final_reply("ٹھیک ہے۔"),
        ],
    );
    assert_ne!(reply.outcome, AskOutcome::Answered);
}

// ---- 7. Q158: "which one" written with braces ----------------------------------------------------

fn answer_with_choose(kind: &str, choose: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": kind, "text": "", "show": [], "next": [],
            "choose": choose, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

#[test]
fn q158_a_which_one_with_the_find_id_in_braces_still_asks_the_owner() {
    // Two sellers fit "rahim": the model asked which one but wrote `"choose":"{{r1}}"`. That failed
    // the check and the question ended as plain tables. The braces are taken off now.
    let h = Harness::day();
    world::add_account(
        &h.world.env,
        &h.world.owner,
        mandi_services::core::accounts::AccountType::Seller,
        "رحیم خان",
        "Rahim Khan",
        true,
        "",
    );
    let (reply, requests) = h.ask(
        "rahim ko kitne paise dene hen",
        None,
        vec![
            tool_reply(&[find("rahim", "seller")]),
            answer_with_choose("which_one", "{{r1}}"),
        ],
    );
    assert_eq!(requests.len(), 2);
    assert_eq!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
    assert_eq!(reply.options.len(), 2, "{reply:#?}");
}

// ---- 5. Q111, Q112: two items in one question ---------------------------------------------------

#[test]
fn q111_a_name_that_matches_nothing_does_not_end_a_question_that_also_read_the_books() {
    // The live model asked for two items in one reply and typed one of them wrongly: the app ended
    // the question as "not found" at once, though the other item was read. Now the model sees both
    // results and answers.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "kela aur saib ka is saal ka commission",
        None,
        vec![
            tool_reply(&[
                analytics("A", json!({"item": "khira"})),
                analytics("A", json!({"item": "apple"})),
            ]),
            final_reply("{{r2.rows.i2.name}} کا کمیشن {{r2.rows.i2.total_comm}} رہا۔"),
        ],
    );
    assert_eq!(requests.len(), 2, "the model was called again: {reply:#?}");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(harness::money(figures(answer_of(&reply))[0].0), 1_520);

    // With nothing read in that reply, the question still ends at once.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "khira ka commission",
        None,
        vec![
            tool_reply(&[analytics("A", json!({"item": "khira"}))]),
            final_reply("ٹھیک ہے۔"),
        ],
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(reply.outcome, AskOutcome::NotFound, "{reply:#?}");
}

#[test]
fn q112_two_items_asked_for_each_are_not_summed_into_one_figure() {
    // Banana 3,928 and apple 1,520 (E2E-11). "kele or saib ka commission" asks for each; the sum
    // 5,448 is refused unless the question asks for the two together.
    let calls = |question: &str, answer: &str| {
        let h = Harness::day();
        h.ask(
            question,
            None,
            vec![
                tool_reply(&[
                    analytics("A", json!({"item": "banana"})),
                    analytics("A", json!({"item": "apple"})),
                ]),
                tool_reply(&[(
                    "calculate",
                    json!({"op": "sum", "inputs": ["r1.totals.total_comm", "r2.totals.total_comm"]}),
                )]),
                final_reply(answer),
            ],
        )
    };
    let (reply, requests) = calls(
        "kele or saib ka comision aur bikri btao",
        "{{r1.rows.i1.name}} {{r1.rows.i1.gross}} {{r1.rows.i1.total_comm}} {{r2.rows.i2.name}} {{r2.rows.i2.gross}} {{r2.rows.i2.total_comm}}",
    );
    assert_eq!(tool_error_codes(&requests[2]), ["ERR_OUT_OF_RANGE"]);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    // Banana: sales 29,400, commission 3,928; apple: sales 12,000, commission 1,520.
    assert_eq!(
        [0, 1, 2, 3].map(|i| harness::money(figs[i].0)),
        [29_400, 3_928, 12_000, 1_520]
    );
    // Asked together, the sum is worked out.
    let (reply, requests) = calls(
        "kele or saib ka mila kar commission kitna hai",
        "{{c1.result}}",
    );
    assert!(tool_error_codes(&requests[2]).is_empty());
    assert_eq!(harness::money(figures(answer_of(&reply))[0].0), 5_448);
}

// ---- 6. Q154, Q155: both figures asked, both given -----------------------------------------------

#[test]
fn q154_a_question_for_commission_and_sales_gets_both_figures() {
    // E2E-11: رحیم's goods: sales 29,400, commission 3,928 (view B, his row). The first answer
    // gave the commission only: sent back once; the rewrite gives both.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "rahim ke mal ka commission aur sale kitna he",
        None,
        vec![
            tool_reply(&[
                find("rahim", "seller"),
                analytics("B", json!({"seller": "r1.matches.0"})),
            ]),
            final_reply("{{r1.matches.0.name}} کے مال پر کمیشن {{r2.totals.total_comm}} رہا۔"),
            final_reply(
                "{{r1.matches.0.name}} کے مال کی بکری {{r2.totals.gross}} اور کمیشن {{r2.totals.total_comm}} رہا۔",
            ),
        ],
    );
    assert_eq!(requests.len(), 3, "one rewrite: {reply:#?}");
    let failed = &requests[2]["messages"].as_array().unwrap().last().unwrap()["content"];
    assert!(failed.as_str().unwrap().contains("asked"), "{failed}");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    assert_eq!(
        (harness::money(figs[0].0), harness::money(figs[1].0)),
        (29_400, 3_928)
    );
    // A question for the commission only is not sent back for lacking the sales.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "rahim ke mal ka commission kitna he",
        None,
        vec![
            tool_reply(&[
                find("rahim", "seller"),
                analytics("B", json!({"seller": "r1.matches.0"})),
            ]),
            final_reply("{{r1.matches.0.name}} کے مال پر کمیشن {{r2.totals.total_comm}} رہا۔"),
        ],
    );
    assert_eq!(requests.len(), 2);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    // A question for the number of بکری and commission does not ask for the sales.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "rahim ki kitni bikriyan hui aur commission",
        None,
        vec![
            tool_reply(&[
                find("rahim", "seller"),
                analytics("B", json!({"seller": "r1.matches.0"})),
            ]),
            final_reply(
                "{{r1.matches.0.name}} کی بکریاں {{r2.totals.bikri_count}} اور کمیشن {{r2.totals.total_comm}} رہا۔",
            ),
        ],
    );
    assert_eq!(requests.len(), 2);
}

// ---- 9. Q250, Q251: how to collect dues faster is general advice ----------------------------------

#[test]
fn q250_q251_a_how_to_do_better_question_is_general_advice_with_no_tool() {
    for question in ["wasooli jaldi kaise karen", "وصولی جلدی کیسے کریں"] {
        let h = Harness::day();
        // The live model read the dues list and answered with ordinary advice. Now the call goes
        // out with the tools off, and advice that reads nothing is general advice.
        let (reply, requests) = h.ask(
            question,
            None,
            vec![answer_with_kind(
                "advice",
                "• ہر ہفتے پرانے بقایا کی فہرست دیکھیں۔\n• ادھار پر ادائیگی کی تاریخ طے کریں۔",
            )],
        );
        assert_eq!(requests.len(), 1, "{question}");
        assert_eq!(requests[0]["tool_choice"], "none", "{question}");
        assert_eq!(reply.outcome, AskOutcome::Advice, "{question}: {reply:#?}");
        let answer = answer_of(&reply);
        assert!(answer.general_advice, "{question}: {answer:#?}");
        assert!(answer.sources.is_empty());
    }
    // A question that names a party, asks a figure or holds a number is not general advice.
    for question in [
        "akram se wasooli kaise karen",
        "wasooli kitni hui",
        "5 din mein wasooli kaise karen",
        "ikram ke total bakaya kitne hai",
    ] {
        assert!(
            !mandi_assistant::help_guard::general_advice_asked(
                question,
                &mandi_services::figures::question_numbers(question)
            ),
            "{question}"
        );
    }
}

fn answer_with_kind(kind: &str, text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": kind, "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

// ---- 3. Q129, Q130: one item, this year against last year ---------------------------------------

#[test]
fn q129_q130_a_comparison_left_out_says_how_to_answer_anyway_and_the_difference_is_worked_out() {
    // The books of the history world start in April 2026, so last year has nothing: the comparison
    // is left out. The note says to read the other period and use calculate; September (51,000)
    // against August (48,000) is the same two-reads-and-a-difference path: 3,000 more.
    let h = e2e_59();
    let (_, requests) = h.ask(
        "kele ka commission pichle saal se kitna zyada hai",
        None,
        vec![
            tool_reply(&[
                ("period", this_year()),
                analytics(
                    "A",
                    json!({"from": "r2.from", "to": "r2.to", "compare": "last_year"}),
                ),
            ]),
            final_reply("ٹھیک ہے۔"),
        ],
    );
    // "pichle saal" is read as a period first (r1, last year); this year is r2, the view r3.
    let left_out = result_of(&requests[1], "r3");
    assert!(left_out["compare"].is_null(), "{left_out}");
    let note = note_of(&left_out);
    assert!(note.contains("the comparison was left out"), "{note}");
    assert!(
        note.contains("calculate") && note.contains("last_year"),
        "{note}"
    );

    // The difference of two totals read separately: September against August.
    let h = e2e_59();
    let (reply, requests) = h.ask(
        "ستمبر اور اگست کے کمیشن میں کتنا فرق ہے",
        None,
        vec![
            tool_reply(&[
                analytics("A", json!({"from": "2026-09-01", "to": "2026-09-30"})),
                analytics("A", json!({"from": "2026-08-01", "to": "2026-08-31"})),
            ]),
            tool_reply(&[(
                "calculate",
                json!({"op": "difference", "inputs": ["r1.totals.total_comm", "r2.totals.total_comm"]}),
            )]),
            final_reply("فرق {{c1.result}} رہا۔"),
        ],
    );
    let calc = result_of(&requests[2], "c1");
    assert_eq!(calc["result"]["rs"], 3_000, "{calc}");
    assert_eq!(calc["result"]["dir"], "up", "{calc}");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(harness::money(figures(answer_of(&reply))[0].0), 3_000);
}

// ---- 1. Q203, Q204: a partner's capital ---------------------------------------------------------

/// The worked day after Events 1 to 5, then ب puts in 500,000 and ج puts in 300,000 and takes back
/// 100,000 (PRD 17.4, FR-17.7). Cash stays above 0, so no confirm is needed.
fn capital_world() -> Harness {
    use mandi_services::core::capital::CapitalInput;
    let h = Harness::day();
    let [_, b, c] = h.world.partners;
    let input = |partner: i64, amount: &str| CapitalInput {
        date: "10-10-2026".into(),
        partner_id: Some(partner),
        amount: amount.into(),
        narration: None,
        confirm_cash_below_zero: false,
    };
    let (ctx, owner) = (&h.world.env.ctx, &h.world.owner);
    mandi_services::vouchers::save_capital_in(ctx, owner, &input(b, "500,000")).unwrap();
    mandi_services::vouchers::save_capital_in(ctx, owner, &input(c, "300,000")).unwrap();
    mandi_services::vouchers::save_capital_out(ctx, owner, &input(c, "100,000")).unwrap();
    h
}

#[test]
fn q203_a_partners_capital_is_read_from_his_statement_report() {
    let h = capital_world();
    let (reply, requests) = h.ask(
        "be ka sarmaya kitna laga hua hai",
        None,
        vec![
            tool_reply(&[find("ب", "partner")]),
            tool_reply(&[report(
                "R31",
                json!({"partner": "r1.matches.0", "rows": 20}),
            )]),
            final_reply("{{r1.matches.0.name}} کا سرمایہ {{r2.capital.balance}} ہے۔"),
        ],
    );
    let result = result_of(&requests[2], "r2");
    let capital = &result["capital"];
    assert_eq!(capital["balance"]["rs"], 500_000, "{result}");
    assert_eq!(capital["invested"]["rs"], 500_000, "{result}");
    assert_eq!(capital["taken_back"]["rs"], 0, "{result}");
    assert_eq!(capital["month_close"]["rs"], 0, "{result}");
    // Read in two places that agree, and the parts make the balance.
    assert_eq!(capital["balance"]["checked"], true, "{result}");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    assert_eq!(harness::money(figs[0].0), 500_000);
    assert!(figs[0].1, "checked");

    // The statement of a month that ended before today carries no capital: it is today's balance.
    let h = capital_world();
    let (_, requests) = h.ask(
        "be ka sarmaya pichle mahine",
        None,
        vec![
            tool_reply(&[find("ب", "partner")]),
            tool_reply(&[report(
                "R31",
                json!({"partner": "r2.matches.0", "from": "2026-08-01", "to": "2026-08-31"}),
            )]),
            final_reply("ٹھیک ہے۔"),
        ],
    );
    let result = requests[2]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .find(|v| v["source"] == "R31")
        .expect("the statement");
    assert!(result.get("capital").is_none(), "{result}");
}

#[test]
fn q204_after_a_partner_took_capital_back_what_is_left_is_read_for_everyone() {
    let h = capital_world();
    let (reply, requests) = h.ask(
        "jeem ne sarmaya wapas liya tha, ab uska sarmaya kitna reh gaya",
        None,
        vec![
            tool_reply(&[report("R31", json!({"rows": 100}))]),
            final_reply("ج کا سرمایہ {{r1.rows.s3.capital.balance}} رہ گیا۔"),
        ],
    );
    let result = result_of(&requests[1], "r1");
    let rows = result["rows"].as_array().unwrap();
    let of = |key: &str| rows.iter().find(|r| r["key"] == key).expect(key)["capital"].clone();
    assert_eq!(of("s2")["balance"]["rs"], 500_000);
    let c = of("s3");
    assert_eq!(c["balance"]["rs"], 200_000, "{result}");
    assert_eq!(c["invested"]["rs"], 300_000);
    assert_eq!(c["taken_back"]["rs"], 100_000);
    assert_eq!(c["balance"]["checked"], true);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(harness::money(figures(answer_of(&reply))[0].0), 200_000);
}

// ---- 2. Q215: the fruit a partner took home -----------------------------------------------------

#[test]
fn q215_the_fruit_taken_home_is_totalled_for_each_partner_and_the_guests() {
    use mandi_services::core::fruit::{FruitInput, FruitSource};
    let h = Harness::over(worked::part_sold_world());
    let [a, b, _] = h.world.partners;
    let nazeer = world::add_account(
        &h.world.env,
        &h.world.owner,
        mandi_services::core::accounts::AccountType::Seller,
        "حاجی نذیر",
        "Haji Nazeer",
        true,
        "",
    );
    let take = |partner: Option<i64>, qty: &str, rate: &str| FruitInput {
        for_guests: partner.is_none(),
        partner_id: partner,
        date: "10-10-2026".into(),
        source: FruitSource::OtherShop,
        lot_id: None,
        agent_account_id: Some(nazeer),
        item_id: Some(h.world.apple),
        packing_id: Some(h.world.box_),
        qty: qty.into(),
        weight: None,
        rate: rate.into(),
        note: None,
        editing_id: None,
    };
    let (ctx, owner) = (&h.world.env.ctx, &h.world.owner);
    // الف takes 20 box at 1,800 (36,000), ب 4 at 500 (2,000), the guests 3 at 1,000 (3,000).
    mandi_services::fruit::save(ctx, owner, &take(Some(a), "20", "1,800")).unwrap();
    mandi_services::fruit::save(ctx, owner, &take(Some(b), "4", "500")).unwrap();
    mandi_services::fruit::save(ctx, owner, &take(None, "3", "1,000")).unwrap();
    let key = format!("s{a}");
    let (reply, requests) = h.ask(
        "alif ne is sal ghar ke liye kitna fal liya",
        None,
        vec![
            tool_reply(&[report("R37", json!({"rows": 100}))]),
            final_reply(&format!(
                "{{{{r2.by_for.{key}.name}}}} نے {{{{r2.by_for.{key}.home_qty}}}} نگ لیے، {{{{r2.by_for.{key}.home_value}}}}۔"
            )),
        ],
    );
    // "is saal" is read as a period first (r1); the register is r2.
    let result = result_of(&requests[1], "r2");
    let by_for = result["by_for"].as_array().expect("by_for");
    let row = |k: &str| by_for.iter().find(|r| r["key"] == k).expect(k).clone();
    assert_eq!(row(&key)["home_qty"]["n"], 20, "{result}");
    assert_eq!(row(&key)["home_value"]["rs"], 36_000, "{result}");
    assert_eq!(row(&format!("s{b}"))["home_value"]["rs"], 2_000);
    assert_eq!(row("guests")["value"]["rs"], 3_000);
    assert_eq!(row("guests")["qty"]["n"], 3);
    // The parts add up to the register's own total (27 box, 41,000).
    assert_eq!(result["totals"]["qty"]["n"], 27);
    assert_eq!(result["totals"]["value"]["rs"], 41_000);
    assert!(note_of(&result).contains("shared equally"), "{result}");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    assert_eq!(harness::count(figs[0].0), 20);
    assert_eq!(harness::money(figs[1].0), 36_000);

    // One partner asked for: the register is filtered, and there is no by_for beside its totals.
    let (_, requests) = h.ask(
        "alif ke fal ka total",
        None,
        vec![
            tool_reply(&[
                find("الف", "partner"),
                report("R37", json!({"partner": "r1.matches.0"})),
            ]),
            final_reply("ٹھیک ہے۔"),
        ],
    );
    let filtered = result_of(&requests[1], "r2");
    assert!(filtered.get("by_for").is_none(), "{filtered}");
    assert_eq!(filtered["totals"]["value"]["rs"], 36_000);
}

#[test]
fn q143_the_peak_month_of_the_commission_is_not_the_peak_month_of_the_sales() {
    // View G has several series; its first is the gross. April: 10 box at 10,000 (gross 100,000,
    // commission 12,080). May: 100 box at 990 (gross 99,000, commission 12,680). The month with
    // the most commission is May; the month with the most sales is April. A bare `value` would
    // have been the gross.
    let h = Harness::over(worked::months_world(&[
        (4, true, 10, 10_000),
        (5, true, 100, 990),
    ]));
    let (reply, requests) = h.ask(
        "kis mahine mein sab se zyada commission aaya",
        None,
        vec![
            tool_reply(&[
                ("period", this_year()),
                analytics(
                    "G",
                    json!({"group_by": "month", "from": "r1.from", "to": "r1.to"}),
                ),
            ]),
            final_reply(
                "{{r2.peak.commission.month}} میں سب سے زیادہ کمیشن: {{r2.peak.commission.value}}۔",
            ),
        ],
    );
    let result = result_of(&requests[1], "r2");
    assert_eq!(result["peak"]["commission"]["key"], "m202605", "{result}");
    assert_eq!(
        result["peak"]["commission"]["value"]["rs"], 12_680,
        "{result}"
    );
    assert_eq!(result["peak"]["gross"]["key"], "m202604", "{result}");
    assert_eq!(result["lowest"]["commission"]["key"], "m202604", "{result}");
    // No bare `value` on a month of a chart with several series.
    let entry = &result["series"][0];
    assert!(entry.get("value").is_none(), "{entry}");
    assert!(
        entry.get("commission").is_some() && entry.get("gross").is_some(),
        "{entry}"
    );
    assert!(
        note_of(&result).contains("months that have activity"),
        "{result}"
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(harness::money(figures(answer_of(&reply))[0].0), 12_680);
}

#[test]
fn a_month_close_moves_profit_into_capital_and_a_reversed_close_leaves_no_trace() {
    // October's profit 2,488 is split 996 / 746 / 746 (E2E-9). ب's capital after the close is
    // 500,000 + 746; the close is shown as month_close, not as invested. Reversed, it is gone.
    let h = capital_world();
    let b = h.world.partners[1];
    common::set_today(&h.world.env, 2026, 11, 2);
    let ctx = &h.world.env.ctx;
    let owner = &h.world.owner;
    mandi_services::day::close(
        ctx,
        owner,
        &mandi_services::core::vouchers::DayCloseInput {
            up_to: "31-10-2026".into(),
        },
    )
    .unwrap();
    mandi_services::month::close(
        ctx,
        owner,
        &mandi_services::month::MonthInput {
            month: "2026-10".into(),
        },
    )
    .unwrap();
    let read = |h: &Harness| {
        let (_, requests) = h.ask(
            "be ka sarmaya kitna hai",
            None,
            vec![
                tool_reply(&[
                    find("ب", "partner"),
                    report("R31", json!({"partner": "r1.matches.0", "rows": 20})),
                ]),
                final_reply("ٹھیک ہے۔"),
            ],
        );
        result_of(&requests[1], "r2")["capital"].clone()
    };
    let capital = read(&h);
    assert_eq!(capital["balance"]["rs"], 500_746, "{capital}");
    assert_eq!(capital["invested"]["rs"], 500_000, "{capital}");
    assert_eq!(capital["taken_back"]["rs"], 0, "{capital}");
    assert_eq!(capital["month_close"]["rs"], 746, "{capital}");
    assert_eq!(capital["balance"]["checked"], true, "{capital}");
    // The close is reversed: the profit leaves capital again, and the two places still agree.
    mandi_services::month::reverse(
        ctx,
        owner,
        &mandi_services::month::MonthReverseInput {
            month: "2026-10".into(),
            reason: "غلطی".into(),
        },
    )
    .unwrap();
    let capital = read(&h);
    assert_eq!(capital["balance"]["rs"], 500_000, "{capital}");
    assert_eq!(capital["month_close"]["rs"], 0, "{capital}");
    assert_eq!(capital["balance"]["checked"], true, "{capital}");
    let _ = b;
}

#[test]
fn a_typed_name_kept_for_the_owner_is_never_read_in_a_later_reply() {
    // Two apples fit "apple": the typed name is kept as f1 for the Owner's choice. A model that
    // goes on and reads `f1.matches.0` in its next reply is refused, not given an apple of its
    // own choosing.
    let h = Harness::day();
    world::add_item(
        &h.world.env,
        &h.world.owner,
        "سیب کالا",
        "Apple Black",
        "fruit",
        "unit",
    );
    let (reply, requests) = h.ask(
        "kela aur apple ka commission",
        None,
        vec![
            tool_reply(&[
                analytics("A", json!({"item": "apple"})),
                analytics("A", json!({"item": "banana"})),
            ]),
            tool_reply(&[analytics("A", json!({"item": "f1.matches.0"}))]),
            final_reply("ٹھیک ہے۔"),
        ],
    );
    // The second reply was refused and the Owner is asked at once: no third model call.
    assert_eq!(requests.len(), 2);
    // The Owner is asked which apple, with both options.
    assert_eq!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
    assert_eq!(reply.options.len(), 2);
}

#[test]
fn every_combining_operation_refuses_figures_of_different_items_unless_the_question_asks_for_it() {
    // Banana 3,928, apple 1,520, all items 5,448 (E2E-11). The ops and the words that allow them.
    let run = |question: &str, op: &str, inputs: [&str; 2]| {
        let h = Harness::day();
        let (reply, requests) = h.ask(
            question,
            None,
            vec![
                tool_reply(&[
                    analytics("A", json!({"item": "banana"})),
                    analytics("A", json!({"item": "apple"})),
                    analytics("A", json!({})),
                ]),
                tool_reply(&[("calculate", json!({"op": op, "inputs": inputs}))]),
                final_reply("{{r1.rows.i1.name}} {{r1.rows.i1.total_comm}}"),
            ],
        );
        (tool_error_codes(&requests[2]), reply)
    };
    // sum and average: a together word; an item plus the unfiltered total is another scope.
    for op in ["sum", "average"] {
        for pair in [
            ["r1.totals.total_comm", "r2.totals.total_comm"],
            ["r1.totals.total_comm", "r3.totals.total_comm"],
        ] {
            let (codes, _) = run("kela aur saib ka commission", op, pair);
            assert_eq!(codes, ["ERR_OUT_OF_RANGE"], "{op} {pair:?}");
        }
    }
    let (codes, _) = run(
        "kele aur saib ka total commission",
        "sum",
        ["r1.totals.total_comm", "r2.totals.total_comm"],
    );
    assert!(codes.is_empty());
    // difference: a comparing word.
    let (codes, _) = run(
        "kela aur saib ka commission",
        "difference",
        ["r1.totals.total_comm", "r2.totals.total_comm"],
    );
    assert_eq!(codes, ["ERR_OUT_OF_RANGE"]);
    let (codes, reply) = run(
        "kele aur saib ke commission mein farq",
        "difference",
        ["r1.totals.total_comm", "r2.totals.total_comm"],
    );
    assert!(codes.is_empty());
    let _ = reply;
    // percent_of and change_percent: a share or change word.
    for op in ["percent_of", "change_percent"] {
        let (codes, _) = run(
            "kela ka commission",
            op,
            ["r1.totals.total_comm", "r3.totals.total_comm"],
        );
        assert_eq!(codes, ["ERR_OUT_OF_RANGE"], "{op}");
    }
    let (codes, _) = run(
        "kele ka hissa kitna hai",
        "percent_of",
        ["r1.totals.total_comm", "r3.totals.total_comm"],
    );
    assert!(codes.is_empty());
    // The same item in two periods is one scope: no word is needed.
    let h = Harness::over(worked::months_world(&[
        (8, true, 10, 1_000),
        (9, true, 20, 1_000),
    ]));
    let (_, requests) = h.ask(
        "ستمبر اگست کمیشن",
        None,
        vec![
            tool_reply(&[
                analytics("A", json!({"from": "2026-09-01", "to": "2026-09-30"})),
                analytics("A", json!({"from": "2026-08-01", "to": "2026-08-31"})),
            ]),
            tool_reply(&[(
                "calculate",
                json!({"op": "difference", "inputs": ["r1.totals.total_comm", "r2.totals.total_comm"]}),
            )]),
            final_reply("ٹھیک ہے۔"),
        ],
    );
    assert!(tool_error_codes(&requests[2]).is_empty());
}
