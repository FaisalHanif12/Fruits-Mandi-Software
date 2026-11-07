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
