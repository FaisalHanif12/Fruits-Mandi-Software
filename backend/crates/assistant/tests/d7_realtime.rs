//! D7 (plan 59): real-time figures that are exactly the books, on the rich test year. Every expected
//! figure is read from the oracle of the year (`robustness_year/oracle.rs`), never typed here.
//!
//! F1, the two CRITICAL wrong figures of the live run of 07-11-2026:
//!
//! 1. Q2062 "pichle mahine Faisal se kele par kitna commission liya": the commission taken FROM a buyer
//!    is his buyer-side column, 7,586 (`ix.last_month.banana.faisal.comm_buyer`). The model cited the
//!    `total_comm` of the same row, 15,716 = seller 7,586 + buyer 7,586 + Laga 544.
//! 2. Q2436 "سیب کا اس ہفتے کا آخری ریٹ کیا تھا": the rate of the last sale line, 3,940
//!    (`ix.week.apple.last_rate`). The register's newest-first order kept the first line of the
//!    same بکری and day (3,830). The empty date ("مورخہ ۔") of that live answer was the runner's
//!    text dump, which does not print date parts; the answer carries the date part.
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
#[path = "robustness_year/mod.rs"]
mod year;

use harness::{Harness, answer_of, figures, money};
use mandi_assistant::answer::AnswerPart;
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

fn answer(text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

/// The books as the shop runs them: the app's own reads, answers and speed-ups on.
fn production_harness(y: year::seed::Year) -> Harness {
    let h = Harness::over(y.w);
    h.runtime.set_read_first(true);
    h.runtime.set_local(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

/// The model path with the app's own reads on: the planned reads are made, the model writes the answer,
/// and no instant answer is given (the guards of the model's answer are what the test is about).
fn model_harness(y: year::seed::Year) -> Harness {
    let h = production_harness(y);
    h.runtime.set_instant(false);
    h
}

/// The first money figure an answer shows, in whole rupees.
fn first_money(reply: &mandi_assistant::ask::AskReply) -> i64 {
    let shown = figures(answer_of(reply));
    money(shown.first().expect("a figure").0)
}

fn oracle_rupees(facts: &std::collections::BTreeMap<String, String>, key: &str) -> i64 {
    facts
        .get(key)
        .unwrap_or_else(|| panic!("no oracle fact {key}"))
        .parse()
        .unwrap_or_else(|_| panic!("oracle fact {key} is not a number"))
}

/// A question whose reads the app made: its results as the model saw them in the first request, by
/// the id of each (`r1` ...).
fn tool_results(request: &Value) -> Vec<Value> {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .collect()
}

// ---- Q2062: the commission taken from a buyer is his column, never the total ------------------------

#[test]
fn q2062_commission_taken_from_a_buyer_is_the_buyer_side_7586_never_the_total_15716() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let buyer_side = oracle_rupees(&facts, "ix.last_month.banana.faisal.comm_buyer");
    let total = oracle_rupees(&facts, "ix.last_month.banana.faisal.comm_total");
    assert_eq!(buyer_side, 7_586);
    assert_eq!(total, 15_716);
    let h = model_harness(y);
    let question = "pichle mahine Faisal se kele par kitna commission liya";

    // The model cites the total of both sides and Laga, as the live model did: the app cites the
    // buyer's column of the same row instead, with no second call.
    let (reply, requests) = h.ask(
        question,
        None,
        vec![answer(
            "پچھلے مہینے اس خریدار سے کمیشن {{r4.totals.total_comm}} لیا گیا۔",
        )],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered);
    assert_eq!(requests.len(), 1, "no rewrite call");
    assert_eq!(
        first_money(&reply),
        buyer_side,
        "the buyer side, not 15,716"
    );

    // The right cell cited straight away stays as it is.
    let (reply, requests) = h.ask(
        question,
        None,
        vec![answer(
            "پچھلے مہینے اس خریدار سے کمیشن {{r4.totals.buyer_comm}} لیا گیا۔",
        )],
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(first_money(&reply), buyer_side);

    // A question that asks for the whole keeps the total: 15,716 is right for it.
    let (reply, _) = h.ask(
        "pichle mahine Faisal ke kele par kul commission kitna bana",
        None,
        vec![answer("پچھلے مہینے کل کمیشن {{r4.totals.total_comm}} بنا۔")],
    );
    assert_eq!(
        first_money(&reply),
        total,
        "the total of both sides and Laga"
    );
    h.scan_requests();
}

#[test]
fn q2063_the_same_for_potato_in_september_and_for_the_word_buyer() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let buyer_side = oracle_rupees(&facts, "ix.m2026_09.potato.faisal.comm_buyer");
    let h = model_harness(y);
    let (reply, requests) = h.ask(
        "ستمبر میں فیصل سے آلو پر کتنا کمیشن لیا",
        None,
        vec![answer(
            "ستمبر میں اس خریدار سے کمیشن {{r4.totals.total_comm}} لیا گیا۔",
        )],
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(first_money(&reply), buyer_side);
    // "buyer commission on banana last month" (no name): the app planned no read for it, the model reads
    // view A itself and cites the total. The figure shown is the buyer side of the banana.
    let item_side = oracle_rupees(&facts, "ix.last_month.banana.comm_buyer");
    let view_a = json!({
        "view": "A", "from": "2026-10-01", "to": "2026-10-31", "group_by": null, "item": "Banana",
        "item_group": null, "seller": null, "buyer": null, "city": null, "adda": null,
        "garden": null, "trip": null, "goods": null, "partner": null, "side": null,
        "compare": null, "sort": null, "rows": null, "grade": null
    });
    let question = "pichle mahine kele par gahak se kitna commission mila";
    let (reply, requests) = h.ask(
        question,
        None,
        vec![
            tool_reply("analytics", &view_a),
            answer("پچھلے مہینے گاہک سے کمیشن {{r2.totals.total_comm}} ملا۔"),
        ],
    );
    assert_eq!(requests.len(), 2, "one read, one answer, no rewrite");
    assert_eq!(first_money(&reply), item_side);
    // The seller's column cited for the buyer's question is refused, never turned into another number
    // under the same words (the money review): the model writes it again, once, and is told which cell.
    let (reply, requests) = h.ask(
        question,
        None,
        vec![
            tool_reply("analytics", &view_a),
            answer("پچھلے مہینے گاہک سے کمیشن {{r2.totals.seller_comm}} ملا۔"),
            answer("پچھلے مہینے گاہک سے کمیشن {{r2.totals.buyer_comm}} ملا۔"),
        ],
    );
    assert_eq!(requests.len(), 3, "one rewrite");
    assert_eq!(first_money(&reply), item_side);
    let told: Vec<Value> = requests[2]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .filter_map(|v| v.get("check_failed").cloned())
        .collect();
    assert_eq!(told.len(), 1, "the rewrite is told what failed");
    assert_eq!(told[0][0]["rule"], "measure");
    assert_eq!(told[0][0]["figure"], "r2.totals.buyer_comm");
    // "Faisal se kitna commission mila": "mila" means received, not the whole (money review).
    let (reply, _) = h.ask(
        "pichle mahine Faisal se kele par kitna commission mila",
        None,
        vec![answer(
            "پچھلے مہینے اس خریدار سے کمیشن {{r4.totals.total_comm}} ملا۔",
        )],
    );
    assert_eq!(
        first_money(&reply),
        oracle_rupees(&facts, "ix.last_month.banana.faisal.comm_buyer")
    );
    // A seller named and the seller side asked: the figure charged to him, not the total.
    let rahim = oracle_rupees(&facts, "d7.sale.rahim_banana_year.comm_seller");
    let (reply, _) = h.ask(
        "rahim ke kele par is saal seller commission kitna hai",
        None,
        vec![answer(
            "اس سال بیوپاری سے کمیشن {{r4.totals.total_comm}} لیا گیا۔",
        )],
    );
    assert_eq!(first_money(&reply), rahim);
    // The total cited in an answer that fails another rule too (a typed digit), and cited again in the
    // rewrite, is never shown (money review): each answer is put right, the one shown is the buyer's.
    let (reply, requests) = h.ask(
        "pichle mahine Faisal se kele par kitna commission liya",
        None,
        vec![
            answer("اس سے {{r4.totals.total_comm}} اور 5 دیا۔"),
            answer("پچھلے مہینے اس خریدار سے کمیشن {{r4.totals.total_comm}} لیا گیا۔"),
        ],
    );
    assert_eq!(
        requests.len(),
        2,
        "the failures of both rules in one rewrite"
    );
    assert_eq!(
        first_money(&reply),
        oracle_rupees(&facts, "ix.last_month.banana.faisal.comm_buyer")
    );
}

// ---- Q2436: the last rate is the rate of the last line entered --------------------------------------

/// The date parts of an answer.
fn dates(reply: &mandi_assistant::ask::AskReply) -> Vec<String> {
    answer_of(reply)
        .parts
        .iter()
        .filter_map(|p| match p {
            AnswerPart::Date { iso } => Some(iso.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn q2436_the_planned_read_gives_the_last_line_of_the_day_3940_not_the_first_3830() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let last = oracle_rupees(&facts, "ix.week.apple.last_rate");
    assert_eq!(last, 3_940);
    let h = model_harness(y);
    let question = "سیب کا اس ہفتے کا آخری ریٹ کیا تھا";
    // The reads the app made before the call: R2 for apple this week, newest first, one row.
    let (_, probe) = h.ask(question, None, vec![answer("x")]);
    let results = tool_results(&probe[0]);
    let r2 = results
        .iter()
        .find(|r| r["tool"] == "report" && r["source"] == "R2")
        .expect("the register was read");
    assert_eq!(r2["rows"].as_array().unwrap().len(), 1);
    assert_eq!(
        r2["rows"][0]["rate"]["rs"], last,
        "the last line, not line 1"
    );
    // The answer cites that row's rate and its date: the date is a part of the answer.
    let id = r2["id"].as_str().unwrap();
    let key = r2["rows"][0]["key"].as_str().unwrap();
    let (reply, _) = h.ask(
        question,
        None,
        vec![answer(&format!(
            "اس ہفتے کا آخری ریٹ {{{{{id}.rows.{key}.rate}}}} ہے، مورخہ {{{{{id}.rows.{key}.date}}}}۔"
        ))],
    );
    assert_eq!(first_money(&reply), last);
    assert_eq!(
        dates(&reply),
        vec!["2026-11-04".to_owned()],
        "the date is not empty"
    );
}

#[test]
fn q2436_a_model_that_reads_the_register_newest_first_gets_the_last_line_too() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let last = oracle_rupees(&facts, "ix.week.apple.last_rate");
    let h = Harness::over(y.w);
    h.runtime.set_read_first(false);
    h.runtime.set_local(false);
    h.runtime.set_instant(false);
    h.runtime.set_low_effort(true);
    // One sort key, the date, newest first, one row: the report engine's own sort.
    let args = json!({
        "id": "R2", "from": "2026-11-02", "to": "2026-11-05", "as_of": null, "party": null,
        "item": "Apple", "city": null, "adda": null, "collector": null, "partner": null,
        "trip": null, "garden": null, "bikri": null, "seller": null, "zamindar": null,
        "status": null, "voucher_type": null, "layout": null, "group_by": null, "where": null,
        "sort": [{"column": "date", "order": "desc"}], "rows": 1, "item_group": null
    });
    let (_, requests) = h.ask(
        "سیب کا اس ہفتے کا آخری ریٹ کیا تھا",
        None,
        vec![tool_reply("report", &args), answer("x")],
    );
    let results = tool_results(&requests[1]);
    let r2 = results
        .iter()
        .find(|r| r["tool"] == "report")
        .expect("the register was read");
    assert_eq!(r2["rows"][0]["rate"]["rs"], last);
    // The date and then the entry, both newest first (the form the model used to send): the same row,
    // read in one SQL page, not the whole register.
    let mut two = args.clone();
    two["sort"] = json!([
        {"column": "date", "order": "desc"},
        {"column": "entry", "order": "desc"}
    ]);
    let (_, requests) = h.ask(
        "سیب کا اس ہفتے کا آخری ریٹ کیا تھا",
        None,
        vec![tool_reply("report", &two), answer("x")],
    );
    let results = tool_results(&requests[1]);
    let r2 = results
        .iter()
        .find(|r| r["tool"] == "report")
        .expect("the register was read");
    assert_eq!(r2["rows"][0]["rate"]["rs"], last);
}

// ---- instant lines: exact against the oracle, with no model call ---------------------------------------

/// The money figures of an answer, in whole rupees, in order.
fn all_money(reply: &mandi_assistant::ask::AskReply) -> Vec<i64> {
    figures(answer_of(reply))
        .iter()
        .map(|(cell, _)| money(cell))
        .collect()
}

/// A question the app answers by itself: no request to the model, an answer, and these figures.
fn instant(h: &Harness, question: &str, expected: &[i64]) -> mandi_assistant::ask::AskReply {
    let (reply, requests) = h.ask(question, None, Vec::new());
    assert!(requests.is_empty(), "{question}: the model was called");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{question}");
    assert_eq!(all_money(&reply), expected, "{question}");
    reply
}

#[test]
fn a_buyer_item_commission_laga_seller_item_and_the_last_rate_are_instant_and_exact() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let o = |key: &str| oracle_rupees(&facts, key);
    let h = production_harness(y);

    // Q2062: the commission taken from a buyer on one item is his side, in no model call.
    instant(
        &h,
        "pichle mahine Faisal se kele par kitna commission liya",
        &[o("ix.last_month.banana.faisal.comm_buyer")],
    );
    // Q2063: the same in Urdu letters, for a month named.
    instant(
        &h,
        "ستمبر میں فیصل سے آلو پر کتنا کمیشن لیا",
        &[o("ix.m2026_09.potato.faisal.comm_buyer")],
    );
    // The Laga charged to the buyer on the item is its own figure.
    instant(
        &h,
        "pichle mahine Faisal par kele ka laga kitna tha",
        &[o("ix.last_month.banana.faisal.laga")],
    );
    // A seller and an item: two labelled lines, the one charged to him and the total on his goods.
    instant(
        &h,
        "rahim ke kele ka is saal ka commission kitna hai",
        &[
            o("d7.sale.rahim_banana_year.comm_seller"),
            o("d7.sale.rahim_banana_year.comm_total"),
        ],
    );
    // Q2436: the last rate of apple this week, with its date, in no model call.
    let reply = instant(
        &h,
        "سیب کا اس ہفتے کا آخری ریٹ کیا تھا",
        &[o("ix.week.apple.last_rate")],
    );
    assert_eq!(
        dates(&reply).last().map(String::as_str),
        Some("2026-11-04"),
        "the date of the last line ends the sentence"
    );
    // Q2444, Q2447, Q2449: other items and days.
    instant(
        &h,
        "انگور کا 1 سے 15 اکتوبر تک last ریٹ کیا تھا",
        &[o("ix.r2026_10_01_2026_10_15.grapes.last_rate")],
    );
    instant(
        &h,
        "last rate of potatoe on 14 October",
        &[o("ix.d2026_10_14.potato.last_rate")],
    );
    instant(
        &h,
        "آلو کا اس مہینے last ریٹ کیا تھا",
        &[o("ix.month.potato.last_rate")],
    );
    // The last rate a buyer paid for an item.
    instant(
        &h,
        "Faisal ko pichle mahine kele ka aakhri rate kya tha",
        &[o("ix.last_month.banana.faisal.last_rate")],
    );
    h.scan_requests();
}

// ---- the last rate on a day with no sale (Umar, 08-11-2026) ---------------------------------------------

/// A buyer with a name of his own and an item he bought before today but NOT today, in one packing, with the
/// newest line he had before today: (buyer name, item name, the rate charged, the day). Worked out from the
/// books by SQL, never from the app's own reads.
fn buyer_item_with_no_sale_today(w: &world::World) -> (String, String, i64, String) {
    let (buyer, item, buyer_id, item_id): (String, String, i64, i64) = w
        .env
        .ctx
        .db()
        .read(|c| {
            c.query_row(
                "SELECT a.name_en, i.name_en, a.id, i.id FROM bikri_sales s \
                 JOIN bikri_lots l ON l.id = s.lot_id JOIN items i ON i.id = l.item_id \
                 JOIN accounts a ON a.id = s.buyer_account_id \
                 WHERE s.is_active = 1 AND s.entry_date < '2026-11-05' AND a.is_system = 0 \
                   AND a.is_pukhta_khata = 0 AND a.is_pukhta_party = 0 \
                   AND (SELECT COUNT(*) FROM accounts b WHERE b.name_en LIKE \
                        substr(a.name_en, 1, instr(a.name_en || ' ', ' ') - 1) || '%') = 1 \
                   AND NOT EXISTS (SELECT 1 FROM bikri_sales t JOIN bikri_lots m ON m.id = t.lot_id \
                        WHERE t.is_active = 1 AND t.entry_date = '2026-11-05' \
                          AND t.buyer_account_id = s.buyer_account_id AND m.item_id = l.item_id) \
                   AND (SELECT COUNT(DISTINCT m.packing_id) FROM bikri_sales t \
                        JOIN bikri_lots m ON m.id = t.lot_id WHERE t.is_active = 1 \
                          AND t.buyer_account_id = s.buyer_account_id AND m.item_id = l.item_id) = 1 \
                 ORDER BY a.id, i.id LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
        })
        .expect("a buyer and an item with no sale today");
    let (rate, day): (i64, String) = w
        .env
        .ctx
        .db()
        .read(|c| {
            c.query_row(
                "SELECT s.buyer_rate_paisa, s.entry_date FROM bikri_sales s \
                 JOIN bikri_lots l ON l.id = s.lot_id JOIN bikri_entries e ON e.id = s.bikri_id \
                 WHERE s.is_active = 1 AND s.entry_date < '2026-11-05' AND s.buyer_account_id = ?1 \
                   AND l.item_id = ?2 \
                 ORDER BY s.entry_date DESC, e.number DESC, s.lot_id DESC, s.line_no DESC, s.id DESC \
                 LIMIT 1",
                [buyer_id, item_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
        })
        .expect("the newest line before today");
    (buyer, item, rate / 100, day)
}

#[test]
fn a_last_rate_on_a_day_with_no_sale_says_so_and_gives_the_last_rate_with_its_date() {
    let y = year::seed::build();
    let (buyer, item, rate, day) = buyer_item_with_no_sale_today(&y.w);
    let h = production_harness(y);
    // The buyer and the item had no sale today: one answer, in no model call, that says there was none and then
    // the last rate before today with the day of that line.
    let question = format!("what was the last rate of {item} to {buyer} today");
    let reply = instant(&h, &question, &[rate]);
    assert_eq!(
        dates(&reply).last().map(String::as_str),
        Some(day.as_str()),
        "{question}: the day of the last line ends the sentence"
    );
    // The sentence is the one of "no sale" (its words say so), not the plain last-rate one.
    let said = |reply: &mandi_assistant::ask::AskReply| -> String {
        answer_of(reply)
            .parts
            .iter()
            .filter_map(|p| match p {
                AnswerPart::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let words = said(&reply);
    assert!(
        words.contains("نہیں ہوئی") || words.contains("No sale"),
        "{question}: {words}"
    );
    // On a day the pair did sell, the plain sentence is still the answer (no "no sale" words).
    let reply = h
        .ask("last rate of potatoe on 14 October", None, Vec::new())
        .0;
    let words = said(&reply);
    assert!(
        !(words.contains("نہیں ہوئی") || words.contains("No sale")),
        "{words}"
    );
    h.scan_requests();
}

// ---- R3, R4: a number of a بکری and a quality are never dropped -------------------------------------

#[test]
fn a_bikri_number_or_a_quality_is_never_dropped_from_a_figure() {
    let y = year::seed::build();
    let h = production_harness(y);
    // R3: "kal bikri number 5 ka commission kitna tha" is no figure of the whole shop for yesterday: the
    // app plans no read and gives no instant answer, so the model is asked (and it has R1 for that بکری).
    let (_, requests) = h.ask(
        "kal bikri number 5 ka commission kitna tha",
        None,
        vec![answer("x")],
    );
    assert!(!requests.is_empty(), "the model is asked");
    let read: Vec<String> = tool_results(&requests[0])
        .iter()
        .filter_map(|r| r["tool"].as_str().map(str::to_owned))
        .collect();
    assert!(
        !read.iter().any(|t| t == "tiles" || t == "analytics"),
        "no figure of the whole shop was read for it: {read:?}"
    );
    // R4: the last rate of ONE quality is read with that quality, and has no instant answer (the
    // sentence would not say which quality).
    let (reply, requests) = h.ask("awwal seb ka aakhri rate kya tha", None, vec![answer("x")]);
    assert!(!requests.is_empty(), "no instant answer for a quality");
    let r2 = tool_results(&requests[0])
        .into_iter()
        .find(|r| r["tool"] == "report")
        .expect("the register was read");
    assert_eq!(r2["filters"]["grade"], "first", "only the first quality");
    let _ = reply;
}

// ---- the last rate is the SELLER rate; the order of a day is posting date, بکری number, line -----------

fn save_bikri(w: &world::World, draft: &mandi_services::core::bikri::BikriDraft) {
    mandi_services::bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

#[test]
fn the_last_rate_is_the_seller_rate_unless_the_buyers_is_asked_and_a_later_bikri_is_the_last() {
    // The worked day (10-10-26) and a third بکری of the day: 5 banana to Akram at the seller's rate 590
    // and the buyer's rate 610 (ریٹ مختلف, PRD FR-12.3).
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let mut lot = world::bare_lot(&w, "lot-3", w.rahim, w.banana, "5");
    let mut sale = world::sale("s-3", w.akram, "5", "590");
    sale.rate_differs = true;
    sale.buyer_rate = Some("610".into());
    lot.sales = vec![sale];
    save_bikri(&w, &world::draft(vec![lot]));
    let h = production_harness_world(w);
    // The seller rate, the auction rate: 590, with no model call.
    instant(&h, "kele ka last rate aaj", &[590]);
    // The rate charged TO a named buyer is the buyer's: 610 (lead ruling).
    instant(&h, "Akram ko kele ka last rate aaj", &[610]);
    // The buyer's rate only when the question says so.
    instant(&h, "last buyer rate of banana today", &[610]);
    // No period: the books so far, and the third بکری is still the last of the newest day.
    let (reply, requests) = h.ask("kele ka last rate", None, Vec::new());
    assert!(requests.is_empty());
    assert_eq!(
        all_money(&reply),
        vec![590],
        "the third بکری is the last of the day"
    );
}

fn model_harness_world(w: world::World) -> Harness {
    let h = production_harness_world(w);
    h.runtime.set_instant(false);
    h
}

fn production_harness_world(w: world::World) -> Harness {
    let h = Harness::over(w);
    h.runtime.set_read_first(true);
    h.runtime.set_local(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

#[test]
fn a_last_rate_over_more_than_one_packing_is_not_instant_and_the_register_names_the_packing() {
    // The rate of an item in a peti is not the rate of the same item in a crate (PRD FR-7.53): with two
    // packings in the filter the register names the packing, and the app gives no instant sentence
    // that names none.
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let crate_id = world::add_packing(&w.env, &w.owner, "کریٹ", "Crate");
    let mut lot = world::bare_lot(&w, "lot-3", w.rahim, w.banana, "5");
    lot.packing_id = Some(crate_id);
    lot.sales = vec![world::sale("s-3", w.akram, "5", "590")];
    save_bikri(&w, &world::draft(vec![lot]));
    let h = production_harness_world(w);
    let (reply, requests) = h.ask("kele ka last rate aaj", None, vec![answer("x")]);
    assert!(!requests.is_empty(), "the model says which packing");
    let r2 = tool_results(&requests[0])
        .into_iter()
        .find(|r| r["tool"] == "report" && r["source"] == "R2")
        .expect("the register was read");
    let columns: Vec<&str> = r2["columns"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(columns.contains(&"packing"), "{columns:?}");
    let _ = reply;
}

#[test]
fn the_commission_columns_of_report_r7_and_a_laga_beside_the_commission_are_guarded_too() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let buyer_side = oracle_rupees(&facts, "ix.last_month.banana.faisal.comm_buyer");
    let h = model_harness(y);
    // The model reads R7 (the item register) for Faisal and the banana, and cites its total.
    let r7 = json!({
        "id": "R7", "from": "2026-10-01", "to": "2026-10-31", "as_of": null, "party": "Faisal",
        "item": "Banana", "city": null, "adda": null, "collector": null, "partner": null,
        "trip": null, "garden": null, "bikri": null, "seller": null, "zamindar": null,
        "status": null, "voucher_type": null, "layout": null, "group_by": null, "where": null,
        "sort": null, "rows": null, "item_group": null
    });
    let question = "pichle mahine Faisal se kele par kitna commission liya";
    let (_, probe) = h.ask(
        question,
        None,
        vec![tool_reply("report", &r7), answer("x"), answer("x")],
    );
    let results = tool_results(&probe[1]);
    let r7_result = results
        .iter()
        .find(|r| r["tool"] == "report" && r["source"] == "R7")
        .expect("R7 was read");
    let id = r7_result["id"].as_str().unwrap().to_owned();
    let columns = r7_result["columns"].to_string();
    assert!(columns.contains("total_comm"), "{columns}");
    // The total of R7 is refused or put right; it is never shown as the buyer's commission.
    let (reply, _) = h.ask(
        question,
        None,
        vec![
            tool_reply("report", &r7),
            answer(&format!(
                "پچھلے مہینے اس خریدار سے کمیشن {{{{{id}.totals.total_comm}}}} لیا گیا۔"
            )),
            answer(&format!(
                "پچھلے مہینے اس خریدار سے کمیشن {{{{{id}.totals.buyer_comm}}}} لیا گیا۔"
            )),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered);
    assert_eq!(first_money(&reply), buyer_side);
}

// ---- the newest line: posting date, then بکری number, then line number (lead ruling, PRD FR-7.90) ------

/// Event 1 with 45 of the 50 sold: Akram 20 at 600 (line 1) and Arshad 25 at 580 (line 2), 5 left.
fn part_sold(w: &world::World) {
    let mut draft = world::event1(w);
    draft.lots[0].sales[1].qty = "25".into();
    draft.lots[0].sales[1].seller_rate = "580".into();
    save_bikri(w, &draft);
}

fn later_sale(w: &world::World, buyer: i64, qty: &str, rate: &str, date: &str) {
    use mandi_services::core::bikri::{LaterSaleSaveInput, StockListInput};
    let stock = mandi_services::bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    let mut line = world::sale("s-later", buyer, qty, rate);
    line.sale_id = None;
    mandi_services::bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id: stock.rows[0].lot_id,
            date: date.into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

#[test]
fn the_last_line_is_by_posting_date_then_bikri_number_then_line_not_by_the_order_entered() {
    // (a) The same day: بکری 1 (45 sold, 5 left), then بکری 2 (banana at 700), then a later sale of the
    // 5 left of بکری 1 at 650 on the same day. The last line of the day is بکری 2's (number 2), not the
    // later sale, which was only entered last.
    let w = worked::worked_day();
    part_sold(&w);
    let mut lot = world::bare_lot(&w, "lot-2", w.karim, w.banana, "10");
    lot.sales = vec![world::sale("s-2", w.akram, "10", "700")];
    save_bikri(&w, &world::draft(vec![lot]));
    later_sale(&w, w.arshad, "5", "650", "10-10-2026");
    let h = production_harness_world(w);
    instant(&h, "kele ka last rate aaj", &[700]);

    // (b) The later sale on its OWN, later date counts as the newest, though it belongs to an older بکری.
    let w = worked::worked_day();
    part_sold(&w);
    let h = production_harness_world(w);
    instant(&h, "kele ka last rate aaj", &[580]);
    common::set_today(&h.world.env, 2026, 10, 11);
    later_sale(&h.world, h.world.arshad, "5", "560", "11-10-2026");
    instant(&h, "kele ka last rate aaj", &[560]);
    instant(&h, "kele ka last rate", &[560]);
}

#[test]
fn an_edited_line_keeps_its_place_in_the_order_of_the_day() {
    // بکری 1: line 1 Akram 20 at 600, line 2 Arshad 25 at 580. Line 1 is edited to 610: it is saved again
    // with a new id but it is still line 1, so the last line of the day stays line 2 (580).
    use mandi_services::core::bikri::{EditOpenInput, EditSaveInput};
    let w = worked::worked_day();
    part_sold(&w);
    let one: i64 = w
        .env
        .ctx
        .db()
        .read(|c| {
            c.query_row(
                "SELECT id FROM bikri_entries ORDER BY id LIMIT 1",
                [],
                |r| r.get(0),
            )
        })
        .unwrap();
    let entry: i64 = w
        .env
        .ctx
        .db()
        .read(|c| {
            c.query_row(
                "SELECT id FROM journal_entries WHERE source_type = 'bikri' AND source_id = ?1 \
                 AND posting_type = 'bikri_first' ORDER BY id LIMIT 1",
                [one],
                |r| r.get(0),
            )
        })
        .unwrap();
    let opened = mandi_services::bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: one,
            entry_id: entry,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    mandi_services::bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("rate".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let h = production_harness_world(w);
    instant(&h, "kele ka last rate aaj", &[580]);
}

#[test]
fn the_last_rate_of_an_item_sold_by_weight_is_not_an_instant_line() {
    // The rate of such an item is per kg and the instant sentence does not say so (lead ruling): the
    // question keeps the model, with the register read for it.
    let w = worked::worked_day();
    let mango = world::add_item(&w.env, &w.owner, "آم", "Mango", "fruit", "weight");
    let mut lot = world::bare_lot(&w, "lot-w", w.rahim, mango, "10");
    lot.weight_received = Some("100".into());
    let mut sale = world::sale("s-w", w.akram, "10", "50");
    sale.weight = Some("100".into());
    lot.sales = vec![sale];
    save_bikri(&w, &world::draft(vec![lot]));
    let h = production_harness_world(w);
    let (_, requests) = h.ask("last rate of mango today", None, vec![answer("x")]);
    assert!(!requests.is_empty(), "no instant answer for a rate per kg");
}

#[test]
fn a_side_question_never_takes_its_figure_from_a_total_commission_and_a_laga_is_not_folded_in() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let buyer_side = oracle_rupees(&facts, "ix.last_month.banana.faisal.comm_buyer");
    let laga = oracle_rupees(&facts, "ix.last_month.banana.faisal.laga");
    let h = model_harness(y);
    // The tile of today's commission is the total of both sides and Laga, whatever the question: it is
    // refused for a buyer-side question, the model writes again from the view (money review).
    let (reply, requests) = h.ask(
        "pichle mahine Faisal se kele par kitna commission liya",
        None,
        vec![
            tool_reply("tiles", &json!({})),
            answer("اس خریدار سے کمیشن {{r5.today_commission.amount}} لیا گیا۔"),
            answer("پچھلے مہینے اس خریدار سے کمیشن {{r4.totals.buyer_comm}} لیا گیا۔"),
        ],
    );
    assert_eq!(requests.len(), 3, "one rewrite");
    assert_eq!(first_money(&reply), buyer_side);

    // The same for the day total sheet R34 (its total commission line is on another row for each day of
    // a range) and for the views that hold only the total (D, E, G, H, I, J): a source that has no figure
    // of one side is refused whole.
    let r34 = json!({
        "id": "R34", "from": "2026-11-03", "to": "2026-11-04", "as_of": null, "party": null,
        "item": null, "city": null, "adda": null, "collector": null, "partner": null,
        "trip": null, "garden": null, "bikri": null, "seller": null, "zamindar": null,
        "status": null, "voucher_type": null, "layout": null, "group_by": null, "where": null,
        "sort": null, "rows": null, "item_group": null
    });
    let (reply, requests) = h.ask(
        "pichle mahine Faisal se kele par kitna commission liya",
        None,
        vec![
            tool_reply("report", &r34),
            answer("اس خریدار سے کمیشن {{r5.rows.x6.amount}} لیا گیا۔"),
            answer("پچھلے مہینے اس خریدار سے کمیشن {{r4.totals.buyer_comm}} لیا گیا۔"),
        ],
    );
    assert_eq!(requests.len(), 3, "one rewrite");
    assert_eq!(first_money(&reply), buyer_side);

    // Commission and Laga asked together: the total of the row stands for both sides, so it is refused
    // and not rewritten to the commission alone; the rewrite gives the two figures.
    let view_a = json!({
        "view": "A", "from": "2026-10-01", "to": "2026-10-31", "group_by": null, "item": "Banana",
        "item_group": null, "seller": null, "buyer": "Faisal", "city": null, "adda": null,
        "garden": null, "trip": null, "goods": null, "partner": null, "side": null,
        "compare": null, "sort": null, "rows": null, "grade": null
    });
    let (reply, requests) = h.ask(
        "Faisal se kele ka commission aur laga pichle mahine",
        None,
        vec![
            tool_reply("analytics", &view_a),
            answer("اس خریدار سے کمیشن اور لاگا {{r2.totals.total_comm}} لیا گیا۔"),
            answer(
                "اس خریدار سے کمیشن {{r2.totals.buyer_comm}} اور لاگا {{r2.totals.laga}} لیا گیا۔",
            ),
        ],
    );
    assert!(
        requests.len() >= 3,
        "refused, then written again: {} {:?} {:?}",
        requests.len(),
        reply.outcome,
        reply.failure
    );
    assert_eq!(all_money(&reply), vec![buyer_side, laga]);
}

/// The rules of the `check_failed` messages the model was sent in a request.
fn told_rules(request: &Value) -> Vec<String> {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .filter_map(|v| v.get("check_failed").cloned())
        .flat_map(|list| list.as_array().cloned().unwrap_or_default())
        .filter_map(|f| f["rule"].as_str().map(str::to_owned))
        .collect()
}

#[test]
fn a_figure_made_of_a_total_commission_source_is_refused_for_a_side_and_not_for_no_side() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let buyer_side = oracle_rupees(&facts, "ix.last_month.banana.faisal.comm_buyer");
    let h = model_harness(y);
    let question = "pichle mahine Faisal se kele par kitna commission liya";
    let good = answer("پچھلے مہینے اس خریدار سے کمیشن {{r4.totals.buyer_comm}} لیا گیا۔");

    // The tile of today's commission: a measure failure (the rule, not a wrong period), then the rewrite.
    let (reply, requests) = h.ask(
        question,
        None,
        vec![
            tool_reply("tiles", &json!({})),
            answer("اس خریدار سے کمیشن {{r5.today_commission.amount}} لیا گیا۔"),
            good.clone(),
        ],
    );
    assert_eq!(requests.len(), 3);
    assert!(told_rules(&requests[2]).contains(&"measure".to_owned()));
    assert_eq!(first_money(&reply), buyer_side);

    // A calculation made over that tile is refused the same way.
    let (reply, requests) = h.ask(
        question,
        None,
        vec![
            tool_reply("tiles", &json!({})),
            tool_reply(
                "calculate",
                &json!({"op": "sum", "inputs": ["r5.today_commission.amount"]}),
            ),
            answer("اس خریدار سے کمیشن {{c1.result}} لیا گیا۔"),
            good.clone(),
        ],
    );
    assert_eq!(requests.len(), 4);
    assert!(told_rules(&requests[3]).contains(&"measure".to_owned()));
    assert_eq!(first_money(&reply), buyer_side);

    // A control: a question that names no side takes the tile as it always did.
    let (reply, requests) = h.ask(
        "aaj ka commission kitna hai",
        None,
        vec![answer("آج کا کمیشن {{r1.today_commission.amount}} ہے۔")],
    );
    assert!(!requests.is_empty());
    assert_eq!(reply.outcome, AskOutcome::Answered);
}

#[test]
fn an_estimate_of_another_commission_and_the_register_r9_are_refused_for_a_side_question() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let buyer_side = oracle_rupees(&facts, "ix.last_month.banana.faisal.comm_buyer");
    let h = model_harness(y);
    let question = "pichle mahine Faisal se kele par kitna commission liya";
    let good = answer("پچھلے مہینے اس خریدار سے کمیشن {{r4.totals.buyer_comm}} لیا گیا۔");
    // An estimate of the commission of both sides is no figure of the buyer's side (money review).
    let estimate = json!({
        "measure": "commission_all", "item": null, "seller": null, "buyer": null, "garden": null,
        "trip": null, "head": null, "partner": null, "party": null, "ahead": 6, "until": null,
        "what_if": null, "percent": null, "target": null
    });
    let (reply, requests) = h.ask(
        question,
        None,
        vec![
            tool_reply("estimate", &estimate),
            answer("اس خریدار سے کمیشن {{e1.same_pace.total}} لیا گیا۔"),
            good.clone(),
        ],
    );
    assert_eq!(requests.len(), 3, "one rewrite");
    assert!(told_rules(&requests[2]).contains(&"measure".to_owned()));
    assert_eq!(first_money(&reply), buyer_side);

    // R9 holds each row's commission on that row's side, which is not known here: refused.
    let r9 = json!({
        "id": "R9", "from": "2026-10-01", "to": "2026-10-31", "as_of": null, "party": null,
        "item": null, "city": null, "adda": null, "collector": null, "partner": null,
        "trip": null, "garden": null, "bikri": null, "seller": null, "zamindar": null,
        "status": null, "voucher_type": null, "layout": null, "group_by": null, "where": null,
        "sort": null, "rows": null, "item_group": null
    });
    let (reply, requests) = h.ask(
        question,
        None,
        vec![
            tool_reply("report", &r9),
            answer("اس خریدار سے کمیشن {{r5.totals.commission}} لیا گیا۔"),
            good,
        ],
    );
    assert_eq!(requests.len(), 3, "one rewrite");
    assert!(told_rules(&requests[2]).contains(&"measure".to_owned()));
    assert_eq!(first_money(&reply), buyer_side);
}

#[test]
fn a_buyer_side_never_takes_a_view_with_laga_in_its_side_and_a_wildcard_calculation_is_refused() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let buyer_side = oracle_rupees(&facts, "ix.last_month.banana.faisal.comm_buyer");
    let h = model_harness(y);
    let question = "pichle mahine Faisal se kele par kitna commission liya";
    let good = answer("پچھلے مہینے اس خریدار سے کمیشن {{r4.totals.buyer_comm}} لیا گیا۔");
    // View G read with the side of the buyers holds the buyer commission PLUS Laga: not the figure of the
    // buyer's commission (money review, 7th).
    let view_g = json!({
        "view": "G", "from": "2026-10-01", "to": "2026-10-31", "group_by": "month", "item": null,
        "item_group": null, "seller": null, "buyer": null, "city": null, "adda": null,
        "garden": null, "trip": null, "goods": null, "partner": null, "side": "buyer",
        "compare": null, "sort": null, "rows": null, "grade": null
    });
    let (reply, requests) = h.ask(
        question,
        None,
        vec![
            tool_reply("analytics", &view_g),
            answer("اس خریدار سے کمیشن {{r5.totals.commission}} لیا گیا۔"),
            good.clone(),
        ],
    );
    assert_eq!(requests.len(), 3, "one rewrite");
    assert!(told_rules(&requests[2]).contains(&"measure".to_owned()));
    assert_eq!(first_money(&reply), buyer_side);

    // A sum over every row of the total commission column.
    let (reply, requests) = h.ask(
        question,
        None,
        vec![
            tool_reply(
                "calculate",
                &json!({"op": "sum", "inputs": ["r4.rows.*.total_comm"]}),
            ),
            answer("اس خریدار سے کمیشن {{c1.result}} لیا گیا۔"),
            good,
        ],
    );
    assert_eq!(requests.len(), 3, "one rewrite");
    assert!(told_rules(&requests[2]).contains(&"measure".to_owned()));
    assert_eq!(first_money(&reply), buyer_side);
}

#[test]
fn a_commission_plus_laga_calculation_is_refused_when_no_laga_was_asked() {
    // The buyer's commission and his Laga added up are not "the commission taken from him" (money review
    // 8): the sum 7,586 + 544 is refused, the rewrite cites the commission alone.
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let buyer_side = oracle_rupees(&facts, "ix.last_month.banana.faisal.comm_buyer");
    let h = model_harness(y);
    let question = "pichle mahine Faisal se kele par kitna commission liya";
    let (reply, requests) = h.ask(
        question,
        None,
        vec![
            tool_reply(
                "calculate",
                &json!({"op": "sum", "inputs": ["r4.totals.buyer_comm", "r4.totals.laga"]}),
            ),
            answer("اس خریدار سے کمیشن {{c1.result}} لیا گیا۔"),
            answer("پچھلے مہینے اس خریدار سے کمیشن {{r4.totals.buyer_comm}} لیا گیا۔"),
        ],
    );
    assert_eq!(requests.len(), 3, "one rewrite");
    assert!(told_rules(&requests[2]).contains(&"measure".to_owned()));
    assert_eq!(first_money(&reply), buyer_side);
}

// ---- F1b: O1, the total dues are the tile کل اگراہی (PRD 9.1, glossary, FR-19.9, FR-7.76) --------------

#[test]
fn o1_total_dues_are_the_tile_with_the_sellers_and_the_dead_amount_not_the_buyers_dues() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let total = oracle_rupees(&facts, "tiles.total_dues");
    let buyers = oracle_rupees(&facts, "tiles.buyers_dues");
    assert!(
        total > buyers,
        "the rich year has sellers who owe and a dead amount"
    );
    let h = production_harness(y);
    for question in [
        "what is the total dues of the shop",
        "kul ugrahi kitni banti hai",
        "kul ugrahi kitni hai",
        "total dues",
        "کل اگراہی کتنی ہے",
    ] {
        instant(&h, question, &[total]);
    }
    // The buyers' dues are still the buyers' tile, and a list by collector is the buyers' list.
    instant(&h, "buyers ke dues kitne hain", &[buyers]);
}

#[test]
fn o2_to_o5_an_answer_by_the_app_never_leaves_out_a_part_of_the_question() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let h = production_harness(y);
    // O2: a figure question with an unknown word is not answered with the guide of a screen.
    // O3: the count AND the total. O4: the bills AND the sales and the commission. O5: which sold more
    // AND by how much. Each goes to the model (with its reads), never to a partial instant answer.
    for question in [
        "what is Partner As totel in the shop",
        "kitne bayopariyon ko abhi paise dene hain aur kul kitne",
        "today's sales, the number of bills and the commission",
        "this month which sold more, banana or apple, and by how much",
    ] {
        let (reply, requests) = h.ask(question, None, vec![answer("x"), answer("x")]);
        assert!(
            !requests.is_empty(),
            "{question}: answered by the app alone"
        );
        assert_ne!(
            reply.outcome,
            AskOutcome::NotFound,
            "{question}: taken for a missing name"
        );
    }
    // The plain figures stay instant: an amount asked with "kitne" or "kitni" is not a count.
    instant(
        &h,
        "kitne paise dene hain bayopariyon ko",
        &[oracle_rupees(&facts, "tiles.sellers_payable")],
    );
}

#[test]
fn f2_the_laga_verb_after_a_commission_and_a_roman_kal_before_a_figure_noun() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let h = production_harness(y);
    // "kitna commission laga": laga is the verb, not a Laga asked, so the buyer's commission is the
    // whole question (money review 9): 7,586, instant.
    instant(
        &h,
        "pichle mahine Faisal se kele par kitna commission laga",
        &[oracle_rupees(
            &facts,
            "ix.last_month.banana.faisal.comm_buyer",
        )],
    );
    // D7 R13b: Roman "kal" straight before a figure noun, with no verb, may be "kul" (total): the app is
    // not sure and the model asks; "kal ka commission" (the genitive) is a day and stays instant.
    let (_, requests) = h.ask(
        "kal commission kitna hai",
        None,
        vec![answer("x"), answer("x")],
    );
    assert!(!requests.is_empty(), "no instant answer for a bare kal");
    let (reply, requests) = h.ask("kal ka commission kitna hai", None, Vec::new());
    assert!(requests.is_empty(), "the genitive is a day");
    assert_eq!(reply.outcome, AskOutcome::Answered);
}

#[test]
fn r7_a_voucher_number_is_never_dropped_from_a_receipts_figure() {
    // "aaj wasooli number 3 kitni thi": R23 for today holds ALL of today's receipts; the number 3 is a
    // filter no read has, so the app reads nothing for it and gives no instant figure (D7 R7).
    // The number is named without the word "number" too (money review 10): "wasooli 3", "voucher 3",
    // the voucher's own mark "و-3".
    let y = year::seed::build();
    let h = production_harness(y);
    for question in [
        "aaj wasooli number 3 kitni thi",
        "aaj wasooli 3 kitni thi",
        "voucher 3 ki raqam kitni thi",
        "aaj وصولی و-3 کتنی تھی",
    ] {
        let (_, requests) = h.ask(question, None, vec![answer("x"), answer("x")]);
        assert!(!requests.is_empty(), "{question}: the model is asked");
        let read: Vec<String> = tool_results(&requests[0])
            .iter()
            .filter_map(|r| r["source"].as_str().map(str::to_owned))
            .collect();
        assert!(
            !read.iter().any(|s| s == "R23"),
            "{question}: R23 was read for it: {read:?}"
        );
    }
}

#[test]
fn the_register_titles_its_rate_as_the_buyers_only_beside_the_sellers_rate() {
    // The worked day with a rate-difference line: R2 has `rate` (titled the buyer's) and `seller_rate`;
    // without one the register is exactly as it was (a plain `rate`, no `seller_rate`).
    use mandi_reports::reports::{ReportFilters, ReportId, ReportRunInput};
    let titles = |w: &world::World| -> Vec<(String, String)> {
        let page = mandi_reports::reports::run(
            w.env.ctx.db(),
            &ReportRunInput {
                id: ReportId::R2,
                filters: ReportFilters::default(),
                offset: 0,
                limit: 0,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap();
        page.columns
            .iter()
            .map(|c| (c.key.clone(), c.title_id.clone()))
            .collect()
    };
    let plain = worked::worked_day();
    worked::events_1_to_5(&plain);
    let plain_cols = titles(&plain);
    assert!(plain_cols.contains(&("rate".into(), "report-col-rate".into())));
    assert!(!plain_cols.iter().any(|(k, _)| k == "seller_rate"));

    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let mut lot = world::bare_lot(&w, "lot-3", w.rahim, w.banana, "5");
    let mut sale = world::sale("s-3", w.akram, "5", "590");
    sale.rate_differs = true;
    sale.buyer_rate = Some("610".into());
    lot.sales = vec![sale];
    save_bikri(&w, &world::draft(vec![lot]));
    let cols = titles(&w);
    assert!(cols.contains(&("rate".into(), "report-col-buyer-rate".into())));
    assert!(cols.contains(&("seller_rate".into(), "report-col-seller-rate".into())));
}

#[test]
fn r6_a_packing_word_beside_an_item_is_never_dropped_from_a_figure() {
    // "aaj kele ki peti ki bikri kitni hui": the sales of banana in one PACKING. No read has a packing
    // filter, so the plain view of the banana would be a real figure of something wider: the app plans no
    // read and gives no instant answer for it (D7 R6).
    let y = year::seed::build();
    let h = production_harness(y);
    let (_, requests) = h.ask(
        "aaj kele ki peti ki bikri kitni hui",
        None,
        vec![answer("x"), answer("x")],
    );
    assert!(
        !requests.is_empty(),
        "the model is asked, not the app alone"
    );
    let read: Vec<String> = tool_results(&requests[0])
        .iter()
        .filter_map(|r| r["tool"].as_str().map(str::to_owned))
        .collect();
    assert!(
        !read.iter().any(|t| t == "analytics" || t == "tiles"),
        "a figure of the whole banana was read ahead: {read:?}"
    );
    // The packing is a filter with a group, with no item, and for a rate too (money review 11): the
    // average rate of all the packings is not the rate of the peti.
    for question in [
        "aaj sabzi ki bori ki bikri kitni hui",
        "aaj peti ki bikri kitni hui",
        "aaj kele ki peti ka rate kya hai",
    ] {
        let (_, requests) = h.ask(question, None, vec![answer("x"), answer("x")]);
        assert!(!requests.is_empty(), "{question}: the model is asked");
        let read: Vec<String> = tool_results(&requests[0])
            .iter()
            .filter_map(|r| r["tool"].as_str().map(str::to_owned))
            .collect();
        assert!(
            !read.iter().any(|t| t == "analytics" || t == "tiles"),
            "{question}: a figure of all the packings was read ahead: {read:?}"
        );
    }
    // The same question with no packing word keeps its planned read of the banana.
    let (_, requests) = h.ask(
        "aaj kele ki bikri kitni hui",
        None,
        vec![answer("x"), answer("x")],
    );
    assert!(!requests.is_empty());
    let read: Vec<String> = tool_results(&requests[0])
        .iter()
        .filter_map(|r| r["tool"].as_str().map(str::to_owned))
        .collect();
    assert!(
        read.iter().any(|t| t == "analytics"),
        "the plain question keeps its read: {read:?}"
    );
}

#[test]
fn r8_an_unsure_question_with_a_filter_gets_no_tile_read_ahead() {
    // "akram" and "usman" each name two accounts, so the app is not sure whom the question means and
    // the model answers it.
    // The whole-day tile must not be read ahead for it when a party or an item narrows it: the model
    // could quote the wider figure (D7 R8, money review 10). The first control proves the tile IS read
    // ahead for the same shape of question with no filter, so the guard is what keeps it away.
    // ("udhari" is not used: that word sends the question to general advice, which reads nothing.)
    let y = year::seed::build();
    let h = production_harness(y);
    let tiles_read = |question: &str| -> bool {
        let (_, requests) = h.ask(question, None, vec![answer("x"), answer("x")]);
        assert!(!requests.is_empty(), "{question}: the model is asked");
        tool_results(&requests[0])
            .iter()
            .any(|r| r["tool"].as_str() == Some("tiles"))
    };
    assert!(
        tiles_read("aaj ki bikri zzqx"),
        "control: a plain unsure tile question keeps its tile"
    );
    assert!(
        !tiles_read("aaj ki bikri akram ki"),
        "an unsure question with a party named is narrowed too"
    );
    assert!(
        !tiles_read("aaj faisal ki bikri zzqx"),
        "a party narrows it"
    );
    assert!(!tiles_read("aaj kele ki bikri zzqx"), "an item narrows it");
}

#[test]
fn r5_a_name_in_urdu_letters_asks_which_one_when_the_tag_tells_two_accounts_apart() {
    // اکرم ٹول ٹیکس (0020) and اکرم K (0022, no entries): "akram" asks which one. In Urdu letters it
    // took اکرم K alone, the whole name, and answered 0 without a question (D7 R5, FR-7.77).
    use mandi_services::core::accounts::AccountType;
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let k = world::add_account_tagged(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        ("اکرم", "Akram"),
        (true, ""),
        "K",
    );
    let h = production_harness_world(w);
    // The app is not sure whom the question means, so it writes no answer of its own: the model is
    // asked and finds both (the "کون سا؟" of E2E-52 comes from that call). Before the fix the Urdu
    // question was answered by the app alone: اکرم K, 0, no question.
    for question in ["اکرم کا بقایا کتنا ہے", "akram ka bakaya kitna hai"] {
        let (reply, requests) = h.ask(question, None, vec![answer("x"), answer("x")]);
        assert!(!requests.is_empty(), "{question}: the app guessed alone");
        assert_ne!(reply.outcome, AskOutcome::Answered, "{question}");
    }
    // Two accounts of the very same name told apart only by the tag (PRD FR-10.7, E38): "اسلم" and
    // "اسلم BA" are two people, so neither is "the whole name" and the app asks, in both scripts
    // (money review 11: with the tag added only to the longer name, the plain one won alone).
    let w2 = worked::worked_day();
    worked::events_1_to_5(&w2);
    world::add_account(
        &w2.env,
        &w2.owner,
        AccountType::Buyer,
        "اسلم",
        "Aslam",
        true,
        "",
    );
    world::add_account_tagged(
        &w2.env,
        &w2.owner,
        AccountType::Buyer,
        ("اسلم", "Aslam"),
        (true, ""),
        "BA",
    );
    let h2 = production_harness_world(w2);
    for question in ["اسلم کا بقایا کتنا ہے", "aslam ka bakaya kitna hai"] {
        let (reply, requests) = h2.ask(question, None, vec![answer("x"), answer("x")]);
        assert!(!requests.is_empty(), "{question}: the app guessed alone");
        assert_ne!(reply.outcome, AskOutcome::Answered, "{question}");
    }
    // With the tag typed, the one account is meant, and the app answers it alone.
    let (reply, requests) = h.ask("اکرم K کا بقایا کتنا ہے", None, Vec::new());
    assert!(requests.is_empty(), "{reply:#?}");
    assert_eq!(reply.outcome, AskOutcome::Answered);
    let _ = k;
}

#[test]
fn r11_a_quality_word_beside_a_stock_question_reads_no_sales() {
    // "doyam kele kitne bache hain" asks the goods left, which have no quality. The plan for a quality
    // read the SALES of the second quality for it (D7 R11 a). The control: the sales of the second
    // quality are still read for a sales question.
    let y = year::seed::build();
    let h = production_harness(y);
    let read = |question: &str| -> Vec<String> {
        let (_, requests) = h.ask(question, None, vec![answer("x"), answer("x")]);
        assert!(!requests.is_empty(), "{question}: the model is asked");
        tool_results(&requests[0])
            .iter()
            .filter_map(|r| r["tool"].as_str().map(str::to_owned))
            .collect()
    };
    assert!(
        read("doyam kele ki bikri aaj kitni hui")
            .iter()
            .any(|t| t == "analytics" || t == "report"),
        "control: the sales of the quality are read"
    );
    let stock = read("doyam kele kitne bache hain");
    assert!(
        !stock.iter().any(|t| t == "analytics"),
        "sales were read for a stock question: {stock:?}"
    );
}

#[test]
fn r13_an_answer_that_only_gives_the_owners_own_amount_back_is_refused() {
    // "kya akram ka bakaya 5000 hai": "ہاں، 5000" with no read would confirm his own number as if the
    // books had said it (D7 R13). An answer that cites only his amount, or no figure at all, is
    // refused and written again, and the rewrite is told which results were read to cite.
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let h = model_harness_world(w);
    let ask = |replies: Vec<Value>| {
        let (reply, requests) = h.ask("kya akram ka bakaya 5000 hai", None, replies);
        let told: Vec<Value> = requests
            .iter()
            .flat_map(|r| r["messages"].as_array().unwrap().iter())
            .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
            .filter_map(|v| v.get("check_failed").cloned())
            .collect();
        (reply, requests.len(), told)
    };
    // His own amount alone: refused once.
    let (_, calls, told) = ask(vec![
        answer("ہاں، {{q.n1}} روپے ہے۔"),
        answer("{{r1.matches.0.name}}"),
    ]);
    assert!(calls >= 2, "refused and written again");
    assert!(
        told.iter().any(|t| t.to_string().contains("own_number")),
        "{told:?}"
    );
    // A yes with no figure at all is refused the same way.
    let (_, calls, told) = ask(vec![
        answer("جی ہاں، یہی رقم ہے۔"),
        answer("{{r1.matches.0.name}}"),
    ]);
    assert!(calls >= 2, "refused and written again");
    assert!(
        told.iter().any(|t| t.to_string().contains("own_number")),
        "{told:?}"
    );
}

#[test]
fn r5b_a_pair_of_one_name_never_hands_the_tie_to_a_third_account() {
    // Money review 13: seller "Ali", buyer "Ali" with the tag KHAN, buyer "Ali Khan" (no tag). "ali khan"
    // fits Ali KHAN and Ali Khan alike, and they read the same on screen: the app asks. Taking the
    // whole-name bonus from the twin of the seller only had let Ali Khan win alone.
    use mandi_services::core::accounts::AccountType;
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    world::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "علی",
        "Ali",
        true,
        "",
    );
    world::add_account_tagged(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        ("علی", "Ali"),
        (true, ""),
        "KHAN",
    );
    world::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "علی خان",
        "Ali Khan",
        true,
        "",
    );
    let h = production_harness_world(w);
    let (reply, requests) = h.ask(
        "ali khan ka bakaya kitna hai",
        None,
        vec![answer("x"), answer("x")],
    );
    assert!(!requests.is_empty(), "the app guessed alone");
    assert_ne!(reply.outcome, AskOutcome::Answered);

    // The tag typed with a twin present still finds the one: seller "Aslam" and buyer "Aslam BA".
    let w2 = worked::worked_day();
    worked::events_1_to_5(&w2);
    world::add_account(
        &w2.env,
        &w2.owner,
        AccountType::Seller,
        "اسلم",
        "Aslam",
        true,
        "",
    );
    world::add_account_tagged(
        &w2.env,
        &w2.owner,
        AccountType::Buyer,
        ("اسلم", "Aslam"),
        (true, ""),
        "BA",
    );
    let h2 = production_harness_world(w2);
    let (reply, requests) = h2.ask("aslam ba ka bakaya kitna hai", None, Vec::new());
    assert!(requests.is_empty(), "{reply:#?}");
    assert_eq!(reply.outcome, AskOutcome::Answered);
}

#[test]
fn d331_a_capital_answer_opens_the_partner_overview_never_the_statement_page() {
    // "usman ka sarmaya kitna hai": the capital 200,000 is read from his capital history and the partner
    // overview. The R31 button opens his statement, which does not show 200,000 as a cell: the button of
    // a capital figure is the partner overview screen (PRD 17.3), which does (D7 D331, plan 59 C1).
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let capital = oracle_rupees(&facts, "partner.usman.capital");
    let h = production_harness(y);
    let (reply, requests) = h.ask("usman ka sarmaya kitna hai", None, Vec::new());
    assert!(requests.is_empty(), "the app answers it alone");
    assert_eq!(reply.outcome, AskOutcome::Answered);
    assert_eq!(all_money(&reply), [capital], "{reply:#?}");
    let answer = answer_of(&reply);
    assert!(
        answer.sources.iter().all(|s| !matches!(
            s,
            mandi_assistant::answer::AnswerSource::Report {
                id: mandi_reports::reports::ReportId::R31,
                ..
            }
        )),
        "an R31 button was offered for the capital: {:#?}",
        answer.sources
    );
    let overview = answer
        .sources
        .iter()
        .filter(|s| {
            matches!(
                s,
                mandi_assistant::answer::AnswerSource::Screen {
                    route: mandi_assistant::answer::ScreenRoute::PartnerOverview,
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        overview, 1,
        "one partner overview button: {:#?}",
        answer.sources
    );
}

#[test]
fn r5c_a_short_name_that_another_account_starts_with_asks_which_one() {
    // PRD FR-7.77 (lead ruling): "faisal" is the whole name of one account and the first word of
    // Faisal Traders, so the app does not take it for the shorter one: it asks, in both scripts. With
    // only one Faisal in the books it answers alone (the control). Before, the whole name won the
    // tie-break and the app answered Faisal's figure without a question.
    use mandi_services::core::accounts::AccountType;
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    world::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "فیصل",
        "Faisal",
        true,
        "",
    );
    let h = production_harness_world(w);
    let (reply, requests) = h.ask("faisal ka bakaya kitna hai", None, Vec::new());
    assert!(requests.is_empty(), "control: one Faisal is answered alone");
    assert_eq!(reply.outcome, AskOutcome::Answered);

    let w2 = worked::worked_day();
    worked::events_1_to_5(&w2);
    world::add_account(
        &w2.env,
        &w2.owner,
        AccountType::Buyer,
        "فیصل",
        "Faisal",
        true,
        "",
    );
    world::add_account(
        &w2.env,
        &w2.owner,
        AccountType::Buyer,
        "فیصل ٹریڈرز",
        "Faisal Traders",
        true,
        "",
    );
    let h2 = production_harness_world(w2);
    for question in ["faisal ka bakaya kitna hai", "فیصل کا بقایا کتنا ہے"] {
        let (reply, requests) = h2.ask(question, None, vec![answer("x"), answer("x")]);
        assert!(!requests.is_empty(), "{question}: the app guessed alone");
        assert_ne!(reply.outcome, AskOutcome::Answered, "{question}");
    }
    // The full name typed is the one account.
    let (reply, requests) = h2.ask("faisal traders ka bakaya kitna hai", None, Vec::new());
    assert!(requests.is_empty(), "{reply:#?}");
    assert_eq!(reply.outcome, AskOutcome::Answered);
}

#[test]
fn r5d_a_role_word_settles_a_tie_only_between_accounts_of_one_name() {
    // Money review 15, M2: "faisal gahak ka bakaya" with a seller Faisal and a buyer Faisal Traders.
    // Both fit "faisal" alike now, and "gahak" names a buyer: the role word used to take the buyer,
    // Faisal Traders, alone. A tie of different names is not settled by a role word: the app asks. The
    // same name for a seller and a buyer (one person, E10) is still settled by the role word.
    use mandi_services::core::accounts::AccountType;
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    world::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "فیصل",
        "Faisal",
        true,
        "",
    );
    world::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "فیصل ٹریڈرز",
        "Faisal Traders",
        true,
        "",
    );
    let h = production_harness_world(w);
    let (reply, requests) = h.ask(
        "faisal gahak ka bakaya kitna hai",
        None,
        vec![answer("x"), answer("x")],
    );
    assert!(!requests.is_empty(), "the app guessed alone: {reply:#?}");
    assert_ne!(reply.outcome, AskOutcome::Answered);

    // One person who is both a seller and a buyer under one name: the role word settles it.
    let w2 = worked::worked_day();
    worked::events_1_to_5(&w2);
    world::add_account(
        &w2.env,
        &w2.owner,
        AccountType::Seller,
        "نعیم",
        "Naeem",
        true,
        "",
    );
    world::add_account(
        &w2.env,
        &w2.owner,
        AccountType::Buyer,
        "نعیم",
        "Naeem",
        true,
        "",
    );
    let h2 = production_harness_world(w2);
    let (reply, requests) = h2.ask("naeem gahak ka bakaya kitna hai", None, Vec::new());
    assert!(requests.is_empty(), "{reply:#?}");
    assert_eq!(reply.outcome, AskOutcome::Answered);
}
