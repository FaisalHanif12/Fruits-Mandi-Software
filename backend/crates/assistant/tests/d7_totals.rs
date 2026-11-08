//! D7 (plan 59, F3a): the figures that a report or a view holds as a total, answered by the app alone and
//! exactly, on the rich test year. Every expected figure is an oracle fact of the year
//! (`robustness_year/oracle.rs`), never typed here; every question is one of the D7 bank.
//!
//! - the quantity of a sale (`nag`), an item's average rate, the shop's expenses (all, or one head),
//!   money received, the cash book's closing balance, the sales of one payment type, a seller's net
//!   sale and the value of the fruit taken are cells of their report or view, with no model call;
//! - a figure that needs a filter the app has not understood is never given: "chai ka kharcha"
//!   (the head was not found) and "shop rent so far" go to the model, and so do a question with two
//!   asks ("expense and what the partners took") and "sold for cash and on credit";
//! - a bare "کل" or "kal" before a "how much" word is "total" when the question has a period of its own,
//!   or when a now-figure (dues, payables, cash) is asked and no verb makes it yesterday.
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

use harness::{Harness, answer_of, count, figures, money};
use mandi_assistant::ask::{AskOutcome, AskReply};
use mandi_reports::reports::Cell;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The books as the shop runs them: the app's own reads, answers and speed-ups on.
fn production_harness(y: year::seed::Year) -> Harness {
    let h = Harness::over(y.w);
    h.runtime.set_read_first(true);
    h.runtime.set_local(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

fn facts_of(y: &year::seed::Year) -> BTreeMap<String, String> {
    year::oracle::compute(y).facts
}

fn oracle(facts: &BTreeMap<String, String>, key: &str) -> i64 {
    facts
        .get(key)
        .unwrap_or_else(|| panic!("no oracle fact {key}"))
        .parse()
        .unwrap_or_else(|_| panic!("oracle fact {key} is not a number"))
}

/// An oracle balance: its rupees, with the side word the oracle writes after them left off.
fn oracle_balance(facts: &BTreeMap<String, String>, key: &str) -> i64 {
    let text = facts
        .get(key)
        .unwrap_or_else(|| panic!("no oracle fact {key}"));
    text.split_whitespace()
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("oracle fact {key} is not a balance: {text}"))
}

/// The figures of an answer in whole rupees or counts, in order.
fn shown(reply: &AskReply) -> Vec<i64> {
    figures(answer_of(reply))
        .iter()
        .map(|(cell, _)| match cell {
            Cell::Number { .. } => count(cell),
            Cell::Balance { paisa, .. } => paisa.rupees(),
            other => money(other),
        })
        .collect()
}

/// A question the app answers by itself: no request to the model, an answer, and these figures.
fn instant(h: &Harness, question: &str, expected: &[i64]) -> AskReply {
    let (reply, requests) = h.ask(question, None, Vec::new());
    assert!(requests.is_empty(), "{question}: the model was called");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{question}");
    assert_eq!(shown(&reply), expected, "{question}");
    reply
}

/// A question the app does NOT answer by itself: the model is asked (it has no reply here).
fn not_instant(h: &Harness, question: &str) {
    let (_, requests) = h.ask(question, None, Vec::new());
    assert!(!requests.is_empty(), "{question}: the app answered alone");
}

#[test]
fn the_quantity_of_a_sale_is_the_nag_column_in_every_wording() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // D324, D325: an item over this year; D402, D403: another item; D406, D407: last month.
    for question in [
        "is saal aam kitni qty bika",
        "how many units of mangoe were sold this year",
    ] {
        instant(&h, question, &[oracle(&f, "ix.year.mango.qty")]);
    }
    for question in [
        "is saal moosami kitni qty bik gayi",
        "how many units of mosambi were sold this year",
    ] {
        instant(&h, question, &[oracle(&f, "ix.year.mosambi.qty")]);
    }
    for question in [
        "pichle mahine tamatar kitne nag bika",
        "how many units of tomato were sold last month",
    ] {
        instant(&h, question, &[oracle(&f, "ix.last_month.tomato.qty")]);
    }
    // A packing word beside it is a filter the views do not have: the model answers.
    not_instant(&h, "pichle mahine aloo ki peti me kitni bikri hui");
    h.scan_requests();
}

#[test]
fn an_items_average_rate_is_named_as_the_average_and_the_last_rate_stays_the_last() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // D326, D327: this month; D408, D409: last month.
    for question in [
        "اس مہینے کیلے کا اوسط ریٹ کیا ہے",
        "what is the average banana rate this month",
    ] {
        instant(&h, question, &[oracle(&f, "ix.month.banana.avg_rate")]);
    }
    for question in [
        "پچھلے مہینے سیب کا اوسط ریٹ کیا رہا",
        "pichle mahine seb ka ausat rate kya tha",
    ] {
        instant(&h, question, &[oracle(&f, "ix.last_month.apple.avg_rate")]);
    }
    // The last rate is another figure and has its own sentence (F1).
    instant(
        &h,
        "last rate of potatoe on 14 October",
        &[oracle(&f, "ix.d2026_10_14.potato.last_rate")],
    );
    h.scan_requests();
}

#[test]
fn shop_expenses_are_the_total_of_r29_or_the_row_of_one_head_and_an_unfound_head_is_not_guessed() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // D424, D425: the shop's whole expense this year ("kal" before a figure noun is "total").
    for question in [
        "is saal dukaan ka kul kharcha kitna hua",
        "اس سال دکان کا کل خرچہ کتنا ہوا",
    ] {
        instant(&h, question, &[oracle(&f, "r29.year.total.expense")]);
    }
    // D266, D267: one head, this year.
    for question in [
        "اس سال چائے کا خرچہ کتنا ہوا",
        "how much was spent on tea this year",
    ] {
        instant(&h, question, &[oracle(&f, "r29.year.tea.expense")]);
    }
    // D422, D423: one head, so far. D333: the same for tea.
    for question in [
        "اب تک تنخواہوں پر کتنا خرچ ہوا",
        "how much has been spent on salaries so far",
    ] {
        instant(&h, question, &[oracle(&f, "r29.all.salaries.expense")]);
    }
    // D063, D064: electricity so far (the "kal" of "کل کتنا" is total beside "اب تک").
    for question in [
        "اب تک بجلی کے بل پر کل کتنا خرچ ہوا",
        "how much has been spent on electricity so far",
    ] {
        instant(&h, question, &[oracle(&f, "r29.all.electricity.expense")]);
    }
    // The head word was not understood ("chai", "shop rent"): the whole expense would be another
    // figure, so the model answers. Two asks in one question are not answered by one of them.
    not_instant(&h, "is saal chai ka kharcha kitna hua");
    not_instant(&h, "how much has been spent on shop rent so far");
    not_instant(
        &h,
        "pichle mahine dukan ka kharcha kitna hua aur sharkat daron ne kitna liya",
    );
    h.scan_requests();
}

#[test]
fn money_received_and_the_cash_book_closing_are_cells_of_their_reports() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // D338, D339: last month's receipts ("کل کتنی" beside "پچھلے مہینے" is total, not yesterday).
    for question in [
        "پچھلے مہینے کل کتنی وصولی ہوئی",
        "pichle mahine kul wasuli kitni hoi",
    ] {
        instant(
            &h,
            question,
            &[oracle(&f, "mx.last_month.r23.total.amount")],
        );
    }
    // D450, D451 (this week's, "from the buyers") have a buyers word: see the next test.
    // D336, D337: the cash book at the end of yesterday; D438, D439: at the end of this month.
    for question in [
        "kal rat ko roknad kitni thi",
        "what was the cash at the end of yesterday",
    ] {
        instant(
            &h,
            question,
            &[oracle_balance(
                &f,
                "r12.yesterday.sum.report_closing_balance",
            )],
        );
    }
    for question in [
        "is mahine ke akhir me roknad kitni hai",
        "what is the cash balance of the cash book this month",
    ] {
        instant(
            &h,
            question,
            &[oracle_balance(&f, "r12.month.sum.report_closing_balance")],
        );
    }
    h.scan_requests();
}

#[test]
fn sales_of_one_payment_type_a_sellers_net_sale_and_the_fruit_value_are_report_cells() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // D250 to D252: today's credit sales (R4 total).
    for question in [
        "aaj udhaar me kitni bikri hui",
        "آج ادھار میں کتنی بکری ہوئی",
        "how much was sold on credit today",
    ] {
        instant(&h, question, &[oracle(&f, "d7.sale.credit_today.gross")]);
    }
    // Both payment types in one question: the model answers (the plain figures would be other things).
    not_instant(
        &h,
        "how much was sold for cash today and how much on credit",
    );
    // D454, D455: a seller's net sale. "khalis" and "net" name a partner's figure for a partner only.
    for question in [
        "bashir ki pichle mahine ki khalis raqam kitni bani",
        "what was Bashir's net last month",
    ] {
        instant(&h, question, &[oracle(&f, "b.last_month.bashir.net")]);
    }
    // D437: the value of the fruit taken so far; its quantity is another question.
    instant(
        &h,
        "what is the total value of the fruit taken so far",
        &[oracle(&f, "r37.all.total.value")],
    );
    not_instant(
        &h,
        "how much fruit did Partner A take for home and what is its total value",
    );
    h.scan_requests();
}

#[test]
fn a_bare_kal_beside_a_now_figure_is_total_and_a_past_verb_keeps_it_yesterday() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // D081, D083, D342: the dues and the payables now.
    let buyers = oracle(&f, "tiles.buyers_dues");
    instant(&h, "گاہكوں كے ذمے كل كتنا باقي ہے", &[buyers]);
    instant(
        &h,
        "كل اگراہی كتنی بنتی ہے",
        &[oracle(&f, "tiles.total_dues")],
    );
    h.scan_requests();
}

#[test]
fn a_ledger_question_is_the_balance_with_the_newest_rows_and_the_urdu_may_is_a_month() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // D362, D376: "his hisab" is his balance (read in two places) with a table of the newest ledger rows.
    for (question, key) in [
        ("naseer ka hisab kya he", "seller.naseer.balance"),
        ("imran froot ka hisab kya hai", "buyer.imran.balance"),
    ] {
        let reply = instant(&h, question, &[oracle_balance(&f, key)]);
        let answer = answer_of(&reply);
        assert_eq!(answer.tables.len(), 1, "{question}: the ledger rows");
        assert!(
            answer.tables[0].rows.len() <= 7,
            "{question}: the newest rows only"
        );
        assert!(
            answer.sources.iter().any(|s| matches!(
                s,
                mandi_assistant::answer::AnswerSource::Report {
                    id: mandi_reports::reports::ReportId::R13,
                    ..
                }
            )),
            "{question}: the button of the full ledger"
        );
    }
    // Before: "مئی" was no month, the question read no period and the model was given the books of all
    // time. D253: May, the buyer's side (one named buyer).
    instant(
        &h,
        "اکرم ٹول ٹیکس کا مئی کا کمیشن",
        &[oracle(&f, "d7.sale.akram_toll_tax_may.comm_buyer")],
    );
    h.scan_requests();
}

#[test]
fn bills_are_bikri_entries_the_day_before_yesterday_is_one_day_and_kata_is_the_seller_side() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // D444, D445: how many bills today is the number of بکری of the day.
    for question in ["aaj kitne bill bane", "آج کتنے بل بنے"] {
        instant(&h, question, &[oracle(&f, "tiles.count_today")]);
    }
    // Two counts asked, one answered: the model gives both ("kitne bill aur kitne gahak").
    not_instant(&h, "aaj kitne bill bane aur kitne gahak aaye");
    // D259, D261: the day before yesterday is 03-11-26, never yesterday.
    for question in [
        "parso ki bikri kitni thi",
        "how much was sold the day before yesterday",
    ] {
        instant(&h, question, &[oracle(&f, "ix.d2026_11_03.all.gross")]);
    }
    // D024, D025, D218 to D220: "commission deducted" is what was taken off the seller, his side only.
    for question in [
        "rahim se pichle mahine kitna kamishan kata",
        "how much commission did we deduct from Rahim last month",
    ] {
        instant(
            &h,
            question,
            &[oracle(&f, "b.last_month.rahim.seller_comm")],
        );
    }
    for question in [
        "rahim ke kele ka pichle mahine kitna kamishan kata",
        "رحیم کے کیلے کا پچھلے مہینے کتنا کمیشن کٹا",
        "how much commission was deducted on Rahim's banana last month",
    ] {
        instant(
            &h,
            question,
            &[oracle(&f, "d7.sale.rahim_banana_last_month.comm_seller")],
        );
    }
    h.scan_requests();
}

// ---- the money review of F3a: what must NOT be answered by the app alone ---------------------------------

#[test]
fn a_receipts_figure_with_a_buyers_or_sellers_word_goes_to_the_model() {
    // R23 lists every receipt, a seller's return of advance money too (PRD FR-15.12), so "gahakon se" and
    // "beopariyon se" cannot be answered by its total. The plain question still is.
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    instant(
        &h,
        "is hafte kitni wasooli hui",
        &[oracle(&f, "mx.week.r23.total.amount")],
    );
    not_instant(&h, "is hafte gahakon se kitni wasooli hui");
    not_instant(&h, "اس ہفتے گاہکوں سے کتنی وصولی ہوئی");
    not_instant(&h, "beopariyon se is hafte kitni wasooli hui");
}

#[test]
fn a_filter_the_figure_has_no_read_for_is_never_dropped_from_a_net_sale_a_credit_sale_or_a_month() {
    let y = year::seed::build();
    let h = production_harness(y);
    // A group, a truck, a quality, or two asks beside the figure.
    not_instant(
        &h,
        "bashir ki pichle mahine sabzi ki khalis raqam kitni bani",
    );
    not_instant(&h, "LES-1234 ki aaj udhaar bikri");
    not_instant(&h, "awwal seb ki aaj udhaar bikri kitni hui");
    // "mei" is "in": no month. "rahim ke maal mei kitna commission bana" is not the commission of May.
    // The planned reads carry the period, so a May would show in the register the model is given.
    for question in [
        "rahim ke maal mei kitna commission bana",
        "maal mei ki gayi bikri kitni hui",
        "in mei se kis ka commission zyada hai",
    ] {
        let (_, requests) = h.ask(question, None, Vec::new());
        assert!(!requests.is_empty(), "{question}: the app answered alone");
        let sent = requests[0].to_string();
        assert!(!sent.contains("2026-05"), "{question}: mei was read as May");
    }
    // Two asks in one question: yesterday is not dropped for the last month.
    not_instant(&h, "kal kitni bikri hui aur pichle mahine kitni");
}

#[test]
fn credit_sales_with_a_rate_difference_line_are_not_answered_by_the_app_alone() {
    // The worked day: the credit sales of 10-10-26 are Akram 12,000 and Arshad 17,400 (PRD 4.4), 29,400.
    // Then a line sold to Akram at the seller's rate 590 and the buyer's rate 610 (ریٹ مختلف, FR-12.3):
    // the register carries both rates, R4's total is the buyer's value, and the model words it.
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let h = production_harness_world(w);
    let (reply, requests) = h.ask("aaj udhaar me kitni bikri hui", None, Vec::new());
    assert!(requests.is_empty(), "control: no rate difference, instant");
    assert_eq!(shown(&reply), [29_400]);

    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let mut lot = world::bare_lot(&w, "lot-3", w.rahim, w.banana, "5");
    let mut sale = world::sale("s-3", w.akram, "5", "590");
    sale.rate_differs = true;
    sale.buyer_rate = Some("610".into());
    lot.sales = vec![sale];
    mandi_services::bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: world::draft(vec![lot]),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let h = production_harness_world(w);
    not_instant(&h, "aaj udhaar me kitni bikri hui");
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
fn the_cash_book_closing_is_a_debit_balance() {
    let y = year::seed::build();
    let h = production_harness(y);
    let (reply, requests) = h.ask(
        "what was the cash at the end of yesterday",
        None,
        Vec::new(),
    );
    assert!(requests.is_empty());
    let sides: Vec<_> = figures(answer_of(&reply))
        .iter()
        .map(|(cell, _)| match cell {
            Cell::Balance { side, .. } => Some(*side),
            _ => None,
        })
        .collect();
    assert_eq!(
        sides,
        [Some(mandi_services::core::accounts::BalanceSide::Dr)],
        "cash in hand is a debit balance"
    );
}

// ---- the continuation after "which one?" runs the same guards as any answer (F3b) --------------------

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

/// One reply with several tool calls (the find and the read of its first match, as the model sends them).
fn tool_replies(calls: &[(&str, Value)]) -> Value {
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

fn view_c(from: Option<&str>, to: Option<&str>) -> Value {
    json!({
        "view": "C", "from": from, "to": to, "group_by": null, "item": null, "item_group": null,
        "seller": null, "buyer": "r2.matches.0", "city": null, "adda": null, "garden": null,
        "trip": null, "goods": null, "partner": null, "side": null, "compare": null, "sort": null,
        "rows": null, "grade": null
    })
}

#[test]
fn an_answer_after_which_one_is_kept_to_the_period_of_the_question() {
    // "akram" is two buyers: the app asks which one. The Owner picks one, and the model reads his view C
    // for ALL time and cites its commission. The question said last month, so the scope guard refuses it
    // (before, the continuation had no understanding of the question and showed the figure).
    let y = year::seed::build();
    // The loop as the recorded transcripts run it: no read ahead, the model chooses its tools.
    let h = Harness::over(y.w);
    let question = "akram ka pichle mahine ka commission";
    let (reply, _) = h.ask(
        question,
        None,
        vec![
            tool_replies(&[
                ("find", json!({"text": "akram", "role": "buyer"})),
                ("analytics", view_c(Some("2026-10-01"), Some("2026-10-31"))),
            ]),
            answer("x"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
    assert!(reply.options.len() >= 2);
    let key = reply.options[0].key.clone();
    let (picked, requests) = h.pick(
        reply.question_id,
        mandi_assistant::ask::PickChoice::Match { key },
        vec![
            tool_reply("analytics", &view_c(None, None)),
            answer("پچھلے مہینے کمیشن {{r3.totals.buyer_comm}} رہا۔"),
            answer("x"),
        ],
    );
    assert!(
        requests.len() >= 3,
        "the all-time figure was refused and the model was asked again: {} requests, {picked:#?}",
        requests.len()
    );
    h.scan_requests();
}

// ---- F3b: a group of items, a seller with an open advance -------------------------------------------

#[test]
fn a_group_of_items_is_the_totals_of_view_a_for_the_group() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // The vegetables and the fruit, last month: the sale, the commission with Laga, the Laga, the nag.
    instant(
        &h,
        "pichle mahine sabzi ki bikri kitni hui",
        &[oracle(&f, "ix.last_month.vegetable.gross")],
    );
    instant(
        &h,
        "pichle mahine phal ka commission kitna bana",
        &[oracle(&f, "ix.last_month.fruit.comm_total")],
    );
    instant(
        &h,
        "is mahine sabzi ka laga kitna laga",
        &[oracle(&f, "ix.month.vegetable.laga")],
    );
    instant(
        &h,
        "is mahine phal ki kitni qty bika",
        &[oracle(&f, "ix.month.fruit.qty")],
    );
    // A truck, or "kitna maal" (nag or amount?), is no filter or figure the group read has.
    not_instant(&h, "LES-1234 ki pichle mahine sabzi ki bikri kitni hui");
    not_instant(&h, "is mahine sabzi ka kitna maal bika");
    h.scan_requests();
}

#[test]
fn a_seller_with_an_open_advance_gets_one_sentence_with_payable_advance_and_balance() {
    // Tahir has 30,000 of advance open (R26). His ledger balance already includes it and is not what he is
    // owed now, so the answer is ONE sentence: what is payable now, the open advance, the ledger balance.
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    let reply = instant(
        &h,
        "tahir ka bakaya kitna hai",
        &[
            oracle(&f, "r24.tahir.payable"),
            oracle(&f, "r26.tahir.open"),
            oracle_balance(&f, "seller.tahir.balance"),
        ],
    );
    assert_eq!(figures(answer_of(&reply)).len(), 3);
    h.scan_requests();
}

#[test]
fn an_average_rate_of_an_item_sold_by_weight_is_the_models_never_a_per_nag_rate() {
    // An item sold by weight: the average rate of a view is gross over nag, a rate per nag, which is not the
    // rate per kg the Owner means. The model, with the weights, answers (money review minor 1). The year's
    // own items are all sold by nag, so a pear sold by weight is added to the books for this test.
    let y = year::seed::build();
    let f = facts_of(&y);
    world::add_item(&y.w.env, &y.w.owner, "ناشپاتی", "Pear", "fruit", "weight");
    let h = production_harness(y);
    for question in [
        "what is the average pear rate this month",
        "اس مہینے ناشپاتی کا اوسط ریٹ کیا ہے",
        "is mahine pear ka ausat rate kya hai",
    ] {
        not_instant(&h, question);
    }
    // The same question of an item sold by nag is the view's average (the other side of the rule).
    instant(
        &h,
        "what is the average banana rate this month",
        &[oracle(&f, "ix.month.banana.avg_rate")],
    );
    h.scan_requests();
}

#[test]
fn the_shop_expense_is_given_with_the_partner_section_locked_and_shows_no_partner_figure() {
    // R29 holds partner parts behind the partner code (PRD FR-17.11, FR-7.82). The shop's expense total
    // is not one of them: with the section locked the app still answers it, from the total, and no
    // partner figure is in the answer or in a request (money review minor 2).
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // The shop sets its partner code and the section locks, as it does at the end of every visit.
    let ctx = &h.world.env.ctx;
    mandi_services::partner_code::set(
        ctx,
        &h.world.owner,
        &mandi_services::core::partner_code::PartnerCodeSetInput {
            password: common::OWNER_PASSWORD.to_owned(),
            code: "4826".to_owned(),
            code_again: "4826".to_owned(),
        },
    )
    .expect("the code is set");
    mandi_services::partner_code::lock(ctx).expect("the section locks");
    assert!(
        mandi_services::assistant::partner_locked(ctx).expect("the lock state"),
        "the section is locked"
    );
    let expense = oracle(&f, "r29.year.total.expense");
    for question in [
        "is saal dukaan ka kul kharcha kitna hua",
        "how much was the shop's expense this year",
    ] {
        let reply = instant(&h, question, &[expense]);
        assert_eq!(
            shown(&reply),
            [expense],
            "{question}: one figure, the shop's total"
        );
    }
    // What the partners took out of it is behind the code: the model is asked, and the section stays
    // locked (the app pauses for the code, it never reads the partner parts).
    let (reply, _) = h.ask(
        "pichle mahine dukan ka kharcha kitna hua aur sharkat daron ne kitna liya",
        None,
        Vec::new(),
    );
    assert_ne!(reply.outcome, AskOutcome::Answered);
    h.scan_requests();
}

#[test]
fn the_cash_a_period_took_in_and_paid_out_are_the_totals_of_the_cash_book() {
    // "how much cash came in today", "kal kitna cash gya": the `cash_in` and `cash_out` totals of R12 for the
    // period, one sentence each, in the order asked (D7 D447, D100, plan 59 F3b). The cash in hand now is
    // another figure and keeps its own question.
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // Yesterday of the rich year has one entry and no reversal: given alone. Today has reversals (the
    // year's edits and cancels are on its last day, lead ruling of 08-11-2026), so the totals of the cash
    // book would hold some entries twice: the model answers, and so for a month that holds today.
    instant(
        &h,
        "how much cash went out yesterday",
        &[oracle(&f, "r12.yesterday.total.cash_out")],
    );
    for question in [
        "how much cash came in today",
        "aaj galle se kitna cash nikala",
        "is mahine kitna cash aya",
        "pichle mahine kitna cash gaya",
        "aaj kitna cash aya",
        "aaj ka cash in kitna hai",
    ] {
        not_instant(&h, question);
    }
    // Two and three parts: one sentence for each, in the order asked.
    instant(
        &h,
        "kal kitna cash aya aur kitna cash gaya",
        &[
            oracle(&f, "r12.yesterday.total.cash_in"),
            oracle(&f, "r12.yesterday.total.cash_out"),
        ],
    );
    instant(
        &h,
        "yesterday how much cash came in, how much went out and how much was left",
        &[
            oracle(&f, "r12.yesterday.total.cash_in"),
            oracle(&f, "r12.yesterday.total.cash_out"),
            oracle_balance(&f, "r12.yesterday.sum.report_closing_balance"),
        ],
    );
    // The cash in hand, a party's cash and a head's cash are other questions.
    instant(&h, "cash in hand", &[oracle(&f, "tiles.cash")]);
    for question in [
        "how much cash in the drawer today",
        "akram se kitna cash aya",
        "cash went out for chai yesterday",
        "how much cash came in this month and last month",
        // "aa gaya" and "ho gaya" are not "went out" (money review of the cash book).
        "aaj kitna cash aa gaya",
        "aaj galle mein kitna cash ho gaya",
        // A group of parties, a partner or the staff: the cash of THEM, not of the whole book.
        "kal byopariyon ko kitna cash gaya",
        "aaj byopari ko kitna naqd gaya",
        "aaj gahakon se kitna cash aya",
        "how much cash received from buyers today",
        "aaj partner ne galle se kitna cash nikala",
        "aaj munshi ko kitna cash gaya",
        // The drawer, in any of its words.
        "how much cash in gala today",
        "how much cash in my galla today",
        // Another action beside the direction word.
        "aaj kitna cash diya gaya",
    ] {
        not_instant(&h, question);
    }
    h.scan_requests();
}

/// The worked day (PRD 4.9: cash in 22,800, cash out 14,316, closing 58,484), as the shop runs it.
fn production_over(w: world::World) -> Harness {
    let h = Harness::over(w);
    h.runtime.set_read_first(true);
    h.runtime.set_local(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

#[test]
fn the_cash_book_totals_are_given_alone_only_on_a_day_with_no_reversal() {
    // Lead ruling of 08-11-2026 (the cash book totals hold a cancelled or edited entry twice): on E2E-8's day
    // the cash in is 12,800 + 10,000 = 22,800 and the cash out 5,000 + 4,316 + 3,000 + 2,000 = 14,316, given
    // alone. After بکری 2 is cancelled the same day (E2E-15) its reversal is a row of the day: cash in and
    // cash out go to the model, while the closing balance (45,684 = 58,484 - 12,800) is still given alone.
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let h = production_over(w);
    instant(&h, "aaj kitna cash aya", &[22_800]);
    instant(&h, "aaj kitna cash gaya", &[14_316]);
    instant(&h, "aaj ki roknad ka closing kitna hai", &[58_484]);
    h.scan_requests();

    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let second: i64 = w
        .env
        .ctx
        .db()
        .read(|c| {
            c.query_row(
                "SELECT id FROM bikri_entries ORDER BY id LIMIT 1 OFFSET 1",
                [],
                |r| r.get(0),
            )
        })
        .expect("the second بکری");
    let detail = mandi_services::bikri::get(&w.env.ctx, second).expect("its detail");
    mandi_services::bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::CancelInput {
            id: second,
            version: detail.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: true,
        },
    )
    .expect("the cancel");
    let h = production_over(w);
    not_instant(&h, "aaj kitna cash aya");
    not_instant(&h, "aaj kitna cash gaya");
    not_instant(&h, "aaj kitna cash aya aur kitna cash gaya");
    instant(&h, "aaj ki roknad ka closing kitna hai", &[45_684]);
    h.scan_requests();

    // E2E-13: a same-day edit (اکرم's rate 600 to 610) reverses the first posting and posts it again.
    use mandi_services::core::bikri::{EditOpenInput, EditSaveInput};
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
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
        .expect("the first بکری");
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
        .expect("its first posting");
    let opened = mandi_services::bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: one,
            entry_id: entry,
        },
    )
    .expect("opened for edit");
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
    .expect("the edit");
    let h = production_over(w);
    not_instant(&h, "aaj kitna cash aya");
    not_instant(&h, "aaj kitna cash gaya");
    h.scan_requests();
}

#[test]
fn the_trial_balance_total_debit_and_credit_are_the_totals_of_r18() {
    // D440, D441: the total debit (بنام) of the trial balance as of now; the credit (جمع) the same; both
    // sides one line each, in the order asked. The difference, and the trial balance at the start (R19), are
    // other figures: the model's.
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    let debit = oracle(&f, "r18.total.debit");
    let credit = oracle(&f, "r18.total.credit");
    for question in [
        "ٹرائل بیلنس کا بنام کل کتنا ہے",
        "what is the total debit of the trial balance",
    ] {
        instant(&h, question, &[debit]);
    }
    for question in [
        "ٹرائل بیلنس کا جمع کل کتنا ہے",
        "what is the total credit of the trial balance",
    ] {
        instant(&h, question, &[credit]);
    }
    instant(
        &h,
        "trial balance me banaam aur jama kitna hai",
        &[debit, credit],
    );
    for question in [
        "is there any difference in the trial balance, how much",
        "shuru ke trial balance me kitna banaam aur jama tha",
    ] {
        not_instant(&h, question);
    }
    h.scan_requests();
}

#[test]
fn what_the_buyers_of_a_city_owe_is_the_city_row_of_r21_and_a_part_of_it_is_never_dropped() {
    // D292 to D294: the dues of the buyers of Kasur, the city's own row of R21 (`d7.city.kasur.dues`). The
    // number of its buyers and its dues of one age are other figures: the model's.
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    let dues = oracle(&f, "d7.city.kasur.dues");
    for question in [
        "kasur ke gahakon ke zimme kul kitna baqi hai",
        "قصور کے گاہکوں کے ذمے کل کتنا باقی ہے",
        "how much do the buyers of Kasur owe in all",
    ] {
        instant(&h, question, &[dues]);
    }
    for question in [
        "kasur shehar ke kitne gahakon ka bakaya hai",
        "قصور شہر کے کتنے گاہکوں کا بقایا ہے",
        "shehar kasur ka bakaya aur usme se 8 se 30 din purana kitna hai",
        "the dues of the city Kasur and how much of it is 8 to 30 days old",
    ] {
        not_instant(&h, question);
    }
    h.scan_requests();
}
