//! Plan 82, the live check of 21-11-2026: three wrong figures the real model gave, each with a recorded
//! transcript (no network).
//!
//! - Y18: "this month against last month" worked out a change in percent the wrong way round and showed
//!   last month's rise ("28.66% more") for this month's fall (truth 22.27% less). A change in percent now goes
//!   from the earlier period to the later one, whatever order the model wrote the two figures in.
//! - Y31c: "what is the total of both?" after two figures of this year added up the figures of ALL the books
//!   (454,000 for 295,000 and 98,000). A follow-up that names no period is about the period of the questions
//!   before it.
//! - M18 (`src/sale_guard.rs`, unit tests): the gross sale is view A's `gross`, never the register's goods
//!   value.
//!
//! The books are the client's demo of D8 (today 08-11-2026, `support/worked.rs`): 239,500 of gross sale in
//! 2026 up to 08-11 and 21,000 in 2025 (Haroon 10 banana at 500 and 20 apple at 800).
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

use harness::{Harness, answer_of};
use mandi_assistant::answer::AnswerPart;
use mandi_assistant::ask::{AskOutcome, AskReply};
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::trips;
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

/// An `analytics` call of view A for these dates, every field of the schema.
fn view_a(from: Option<&str>, to: Option<&str>) -> (&'static str, Value) {
    (
        "analytics",
        json!({
            "view": "A", "from": from, "to": to, "group_by": null, "item": null, "item_group": null,
            "seller": null, "buyer": null, "city": null, "adda": null, "garden": null, "trip": null,
            "goods": null, "partner": null, "side": null, "compare": null, "sort": null, "rows": null,
            "grade": null
        }),
    )
}

fn calculate(op: &str, inputs: &[&str]) -> (&'static str, Value) {
    ("calculate", json!({"op": op, "inputs": inputs}))
}

/// What the answer shows for each figure: its debug text (the cell and its direction word).
fn figures_shown(reply: &AskReply) -> Vec<String> {
    answer_of(reply)
        .parts
        .iter()
        .filter_map(|p| match p {
            AnswerPart::Figure { cell, dir, .. } => Some(format!("{cell:?} {dir:?}")),
            _ => None,
        })
        .collect()
}

#[test]
fn a_change_in_percent_goes_from_the_earlier_period_to_the_later_one() {
    // Y18 of the live check. This year 239,500, last year 21,000. The model read this year first and asked
    // for the change FROM this year TO last year: -91.23%, "less", under the words "this year is less than
    // last". The change from last year to this year is +1,040.48%, "more".
    let h = Harness::over(worked::d8().w);
    let (reply, requests) = h.ask(
        "is saal ki khaam bikri pichle saal se kitne percent zyada ya kam hai",
        None,
        vec![
            tool_reply(&[
                view_a(Some("2026-01-01"), Some("2026-11-08")),
                view_a(Some("2025-01-01"), Some("2025-12-31")),
            ]),
            tool_reply(&[calculate(
                "change_percent",
                &["r1.totals.gross", "r2.totals.gross"],
            )]),
            final_reply("{{c1.result}}"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let shown = figures_shown(&reply);
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert!(
        shown[0].contains("Up") || shown[0].contains("More"),
        "this year is MORE than last year: {shown:?}"
    );
    assert!(
        !shown[0].contains("Down") && !shown[0].contains("Less"),
        "{shown:?}"
    );
    // The model is told that the inputs were put in time order.
    let said: String = requests.iter().skip(2).map(Value::to_string).collect();
    assert!(said.contains("earlier period to the later one"), "{said}");
}

#[test]
fn a_change_written_in_time_order_is_left_as_it_is() {
    let h = Harness::over(worked::d8().w);
    let (reply, requests) = h.ask(
        "is saal ki khaam bikri pichle saal se kitne percent zyada ya kam hai",
        None,
        vec![
            tool_reply(&[
                view_a(Some("2025-01-01"), Some("2025-12-31")),
                view_a(Some("2026-01-01"), Some("2026-11-08")),
            ]),
            tool_reply(&[calculate(
                "change_percent",
                &["r1.totals.gross", "r2.totals.gross"],
            )]),
            final_reply("{{c1.result}}"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let shown = figures_shown(&reply);
    assert!(
        shown[0].contains("Up") || shown[0].contains("More"),
        "{shown:?}"
    );
    let said: String = requests.iter().skip(2).map(Value::to_string).collect();
    assert!(
        !said.contains("earlier period to the later one"),
        "no note: {said}"
    );
}

#[test]
fn a_follow_up_with_no_period_adds_the_figures_of_the_period_already_shown() {
    // Y31c of the live check. Question 1 (this year, answered by the app): Rahim's sale. Question 2 (a model
    // question, the same year): Ehsan's. Question 3, "dono ka total batao", names no period: the model read
    // the whole books (no dates) and added figures that are not the ones shown (the ids go on from the
    // first question: r1 to r3 are its reads, so this question's are r4 and r5). The calculator refuses it
    // and says the dates to read again; the model then reads this year and adds the right figures.
    let h = Harness::over(worked::d8().w);
    h.runtime.set_local(true);
    h.runtime.set_read_first(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    let first = h
        .ask("rahim ki is saal ki bikri kitni hai", None, Vec::new())
        .0;
    assert_eq!(first.outcome, AskOutcome::Answered, "{first:#?}");
    let conv = first.conversation_id.clone();
    let (reply, requests) = h.ask(
        "dono ka total batao",
        Some(&conv),
        vec![
            // Call 1: the whole books, no dates, and the sum of two of its figures.
            tool_reply(&[view_a(None, None)]),
            tool_reply(&[calculate(
                "sum",
                &["r4.totals.gross", "r4.totals.seller_comm"],
            )]),
            // Told the dates, the model reads this year again and adds those.
            tool_reply(&[view_a(Some("2026-01-01"), Some("2026-11-08"))]),
            tool_reply(&[calculate(
                "sum",
                &["r5.totals.gross", "r5.totals.seller_comm"],
            )]),
            final_reply("{{c1.result}}"),
        ],
    );
    let said: String = requests.iter().skip(1).map(Value::to_string).collect();
    let shown: Vec<String> = requests
        .iter()
        .map(|r| {
            r["messages"]
                .as_array()
                .map(|m| {
                    m.iter()
                        .filter(|x| x["role"] != "system")
                        .rev()
                        .take(2)
                        .map(|x| {
                            format!(
                                "{}: {}",
                                x["role"],
                                x["content"]
                                    .to_string()
                                    .chars()
                                    .take(900)
                                    .collect::<String>()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" || ")
                })
                .unwrap_or_default()
        })
        .collect();
    assert!(
        said.contains("names no period") && said.contains("2026-01-01 to"),
        "the model was not told the dates: {shown:#?}"
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
}

/// The D8 books with three trips of the three partners (from their pockets, so no cash is needed): 01-10-26,
/// 03-10-26 and 05-10-26, whose codes are 0070, 0071 and 0072.
fn books_with_trips() -> Harness {
    let w = worked::d8().w;
    let [a, b, c] = w.partners;
    let pocket = |partner_id: i64, share: &str| VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Pocket,
    };
    for date in ["01-10-2026", "03-10-2026", "05-10-2026"] {
        trips::create(
            &w.env.ctx,
            &w.owner,
            &TripInput {
                date: date.into(),
                market: Some("Lahore".into()),
                sent_person: None,
                note: None,
                partners: vec![pocket(a, "40"), pocket(b, "30"), pocket(c, "30")],
            },
        )
        .unwrap();
    }
    let h = Harness::over(w);
    h.runtime.set_local(true);
    h.runtime.set_read_first(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

#[test]
fn the_first_second_and_last_trip_are_found_in_the_order_of_the_books() {
    // D13, Y13 and Y27 of the live check ended as "not found": the words "pehle", "doosre" were searched as the
    // text of a trip. The first trip is the one with the lowest code, the second the next one, the last the
    // highest; the app reads that trip's report (R33) before the model answers.
    let h = books_with_trips();
    for (question, date) in [
        (
            "pehle safar ka nateeja kya raha, kitna nafa ya nuksan hua ?",
            "01-10-2026",
        ),
        (
            "doosre safar mein abhi tak kitna maal khareeda aur kitna bik chuka ?",
            "03-10-2026",
        ),
        ("akhri safar ka nateeja kya raha ?", "05-10-2026"),
    ] {
        let (reply, requests) = h.ask(question, None, Vec::new());
        assert_ne!(
            reply.outcome,
            AskOutcome::NotFound,
            "{question}: {reply:#?}"
        );
        assert!(!requests.is_empty(), "{question}: the model was not asked");
        let first = requests[0].to_string();
        let tools: Vec<String> = requests[0]["messages"]
            .as_array()
            .map(|m| {
                m.iter()
                    .filter(|x| x["role"] == "tool")
                    .map(|x| {
                        x["content"]
                            .to_string()
                            .chars()
                            .take(300)
                            .collect::<String>()
                    })
                    .collect()
            })
            .unwrap_or_default();
        assert!(
            first.contains(date),
            "{question}: the report of the trip of {date} was not read ahead: {tools:#?}"
        );
    }
    // A trip number that is not there is said so, as before ("safar 99").
    let (reply, _) = h.ask("safar 99 ka nateeja kya raha", None, Vec::new());
    assert_eq!(reply.outcome, AskOutcome::NotFound, "{reply:#?}");
}

#[test]
fn the_commission_from_the_sellers_and_from_the_buyers_is_read_ahead_and_both_columns_may_be_cited()
{
    // Y28 of the live check ended as "checks failed" twice: "bikri walon" (the sellers) was read as a sale asked
    // and as the buyer side only, so the answer that cited the seller and the buyer commission was refused.
    // November on the D8 books: 6% of 216,500 each way, 12,990 charged to the sellers and 12,990 to the buyers.
    let h = production_books();
    let (reply, requests) = h.ask(
        "is mahine commission bikri walon se kitni li aur khareedaron se kitni ?",
        None,
        vec![final_reply(
            "{{r2.totals.seller_comm}} {{r2.totals.buyer_comm}}",
        )],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(
        requests.len(),
        1,
        "view A was read ahead and one call answered"
    );
    let shown: Vec<String> = figures_shown(&reply);
    assert_eq!(shown.len(), 2, "{shown:?}");
    assert!(
        shown
            .iter()
            .all(|s| s.contains("12990") || s.contains("1299000")),
        "{shown:?}"
    );
}

/// The D8 books with the app's own reads and answers on.
fn production_books() -> Harness {
    let h = Harness::over(worked::d8().w);
    h.runtime.set_local(true);
    h.runtime.set_read_first(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

/// The tool names and the first 200 characters of each tool result in the first request of a question.
fn first_reads(requests: &[Value]) -> Vec<String> {
    requests
        .first()
        .and_then(|r| r["messages"].as_array())
        .map(|m| {
            m.iter()
                .filter(|x| x["role"] == "tool")
                .map(|x| x["content"].to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn the_expenses_for_each_month_read_view_m_grouped_by_month_before_the_model_answers() {
    // Y03 of the live check: R29 holds one total for the period, so the model said the months were not there.
    // View M has a column for each month of its period.
    let h = production_books();
    let (_, requests) = h.ask(
        "is saal ka kharcha mahine ke hisab se batao",
        None,
        Vec::new(),
    );
    assert!(!requests.is_empty(), "the model was not asked");
    let reads = first_reads(&requests);
    assert!(
        reads.iter().any(|r| r.contains("\\\"source\\\":\\\"M\\\"")),
        "view M by month was not read ahead: {reads:#?}"
    );
}

#[test]
fn a_seller_and_the_buyers_of_his_goods_have_views_b_and_c_read_before_the_model_answers() {
    // D29 of the live check: "kis kis gahak ne kitna liya" after a seller's day: the answer said no buyer was named
    // although three bought. View C read with the seller's key lists them. D8: Ehsan's 200 mango of 03-11-26 went to
    // Mani, Anas and Haroon.
    let h = production_books();
    let (_, requests) = h.ask(
        "3 november ko ehsan ne kitna maal bheja aur kis kis gahak ne kitna liya ?",
        None,
        Vec::new(),
    );
    assert!(!requests.is_empty(), "the model was not asked");
    let reads = first_reads(&requests);
    let sources: Vec<&str> = ["B", "C"]
        .into_iter()
        .filter(|v| {
            reads
                .iter()
                .any(|r| r.contains(&format!("\\\"source\\\":\\\"{v}\\\"")))
        })
        .collect();
    assert_eq!(sources, ["B", "C"], "{reads:#?}");
    // The buyers are in the first request.
    let all = reads.join(" ");
    for buyer in ["Mani", "Anas", "Haroon"] {
        assert!(all.contains(buyer), "{buyer} is not in the reads: {all}");
    }
}

#[test]
fn the_share_of_a_buyer_in_the_shops_commission_is_no_partner_question() {
    // D25 of the live check ended as "locked": "share" was taken for a partner's share. It is a percent of the
    // shop's commission, a figure of the books that needs no partner code.
    let h = production_books();
    let (reply, requests) = h.ask(
        "What share of the shop's total commission came from Mani this month, in percent?",
        None,
        Vec::new(),
    );
    assert!(!requests.is_empty(), "the model was not asked");
    assert!(!reply.locked, "{reply:#?}");
}

#[test]
fn a_buyers_purchases_for_each_month_read_view_c_grouped_by_month_before_the_model_answers() {
    // Y11 of the live check: no read gave a buyer's purchases by month, so the model said it could not. View C
    // with `group_by` month gives one row for each month, each with its period. D8: Haroon bought 18,000 on
    // 15-03-26 and 117,000 on 03-11-26 (his 12,000 apple of 2025 is another year).
    let h = production_books();
    let (_, requests) = h.ask(
        "haroon ne is saal mahine ke hisab se kitna maal liya ?",
        None,
        Vec::new(),
    );
    assert!(!requests.is_empty(), "the model was not asked");
    let reads = first_reads(&requests);
    let view_c = reads
        .iter()
        .find(|r| r.contains("\\\"source\\\":\\\"C\\\""))
        .unwrap_or_else(|| panic!("view C was not read ahead: {reads:#?}"));
    for month in ["2026-03", "2026-11"] {
        assert!(view_c.contains(month), "no row for {month}: {view_c}");
    }
    assert!(
        view_c.contains("1800000") || view_c.contains("18000"),
        "{view_c}"
    );
}

#[test]
fn the_analytics_tool_groups_the_rows_of_views_a_b_c_and_e_and_reads_view_m() {
    // The model's own call is accepted too (D6 v1.14): group_by on A, B, C, E gives one set of rows for each
    // bucket and unique keys; view M is the shop's expenses by head with a column for each month.
    let h = production_books();
    let (_, requests) = h.ask(
        "is saal ki bikri mahine ke hisab se batao",
        None,
        vec![
            tool_reply(&[
                (
                    "analytics",
                    json!({
                        "view": "A", "from": "2026-01-01", "to": "2026-11-08", "group_by": "month",
                        "item": null, "item_group": null, "seller": null, "buyer": null, "city": null,
                        "adda": null, "garden": null, "trip": null, "goods": null, "partner": null,
                        "side": null, "compare": null, "sort": null, "rows": null, "grade": null
                    }),
                ),
                (
                    "analytics",
                    json!({
                        "view": "M", "from": "2026-01-01", "to": "2026-11-08", "group_by": "month",
                        "item": null, "item_group": null, "seller": null, "buyer": null, "city": null,
                        "adda": null, "garden": null, "trip": null, "goods": null, "partner": null,
                        "side": null, "compare": null, "sort": null, "rows": null, "grade": null
                    }),
                ),
            ]),
            final_reply("{{r2.totals.gross}}"),
        ],
    );
    let tools: Vec<String> = requests
        .last()
        .and_then(|r| r["messages"].as_array())
        .map(|m| {
            m.iter()
                .filter(|x| x["role"] == "tool")
                .map(|x| x["content"].to_string())
                .collect()
        })
        .unwrap_or_default();
    let all = tools.join(" ");
    assert!(
        all.contains("\\\"source\\\":\\\"M\\\""),
        "view M was refused: {all}"
    );
    assert!(!all.contains("ERR_OUT_OF_RANGE"), "{all}");
    assert!(
        all.contains("2026-03") && all.contains("2026-11"),
        "no monthly rows: {all}"
    );
}
