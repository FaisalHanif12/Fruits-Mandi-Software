//! D7 (plan 59, item C): several figures asked in one question, on the rich test year.
//!
//! The app answers a question by itself only when it can answer EVERY part of it. These tests hold the rules
//! that decide that, each with an oracle figure of the year (`robustness_year/oracle.rs`), never a typed one:
//!
//! 7. a question word, a duration or a count of times ("kab se", "kis kis ka", "kitne din se", "kitni baar")
//!    asks something a figure alone leaves out, so the model answers; "kis rate par" and "kis qadar" do not;
//! 8. a second figure that the tiers, a repeat, a buying or selling verb or a word of a screen's name would
//!    drop without a word is never left out: the model answers;
//! 9. two to four different figures of one scope are one line each, in the order asked, and the answer is given
//!    only when every figure has a line of its own, the period is one for all of them, and no figure is the
//!    same thing as another.
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
use mandi_assistant::understand::{Reader, Understanding, understand};
use mandi_reports::reports::Cell;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use std::collections::BTreeMap;

fn production_harness(y: year::seed::Year) -> Harness {
    let h = Harness::over(y.w);
    h.runtime.set_read_first(true);
    h.runtime.set_local(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

/// The books with the several-measure answers ON (lead decision D333 keeps them OFF in the app until Umar says
/// otherwise): the path that one line of `MULTI_MEASURE_INSTANT` would switch on.
fn multi_harness(y: year::seed::Year) -> Harness {
    let h = production_harness(y);
    h.runtime.set_multi_measure(true);
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

/// The side of an oracle balance ("Dr" or "Cr"), as the oracle writes it after the rupees.
fn oracle_side(facts: &BTreeMap<String, String>, key: &str) -> String {
    facts
        .get(key)
        .and_then(|t| t.split_whitespace().nth(1))
        .unwrap_or_else(|| panic!("oracle fact {key} has no side"))
        .to_owned()
}

/// What the app understood of a question, with today the day of the rich year.
fn understood(h: &Harness, question: &str) -> Understanding {
    common::set_today(&h.world.env, 2026, 11, 5);
    let language = if question
        .chars()
        .any(|c| ('\u{0600}'..='\u{06FF}').contains(&c))
    {
        Language::Ur
    } else {
        Language::En
    };
    understand(
        &Reader {
            ctx: &h.world.env.ctx,
            actor: &h.world.owner,
            language,
            today: BusinessDate::from_ymd(2026, 11, 5).expect("a date"),
            partner_locked: false,
        },
        question,
    )
}

/// A question the app answers by itself: no request to the model, an answer, and these figures in this order.
fn instant(h: &Harness, question: &str, expected: &[i64]) {
    let (reply, requests) = h.ask(question, None, Vec::new());
    assert!(requests.is_empty(), "{question}: the model was called");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{question}");
    assert_eq!(shown(&reply), expected, "{question}");
}

/// A question the model answers: it is asked (it has no reply here), and the app gave no "not found".
fn goes_to_the_model(h: &Harness, question: &str) {
    let (reply, requests) = h.ask(question, None, Vec::new());
    assert!(!requests.is_empty(), "{question}: the app answered alone");
    assert_ne!(
        reply.outcome,
        AskOutcome::NotFound,
        "{question}: refused as not found"
    );
}

#[test]
fn guard_7_a_question_word_or_a_duration_that_asks_more_than_a_figure_goes_to_the_model() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = multi_harness(y);
    // "kis rate par" asks the rate, which is the figure: the "kis" of it is no second ask.
    instant(
        &h,
        "kele ka aaj ka aakhri kis rate par bika",
        &[oracle(&f, "ix.today.banana.last_rate")],
    );
    // The figure alone is answered, as before, with the side the books give it (بنام or جمع).
    instant(
        &h,
        "akram toll tax ka bakaya kitna hai",
        &[oracle_balance(&f, "buyer.akram_toll_tax.balance")],
    );
    let (reply, _) = h.ask("akram toll tax ka bakaya kitna hai", None, Vec::new());
    let side = figures(answer_of(&reply))
        .iter()
        .find_map(|(cell, _)| match cell {
            Cell::Balance { side, .. } => Some(format!("{side:?}")),
            _ => None,
        })
        .expect("a balance cell");
    let want = oracle_side(&f, "buyer.akram_toll_tax.balance");
    assert!(
        side.to_lowercase().starts_with(&want.to_lowercase()[..1]),
        "side {side} against the oracle's {want}"
    );
    instant(
        &h,
        "how much is the dead amount",
        &[oracle(&f, "tiles.dead")],
    );
    instant(
        &h,
        "aaj ki bikri kitni hui",
        &[oracle(&f, "tiles.sales_today")],
    );
    // The same figure with a second ask: a date, a person, a place, a reason, a duration, a count of times.
    for question in [
        "akram toll tax ka bakaya kitna hai aur kab se",
        "akram toll tax ka bakaya kitna hai aur kitne din se",
        "akram toll tax ka bakaya kitna purana hai",
        "dead amount kitna hai aur kis kis ka",
        "dead amount kitna hai aur kin logon ka",
        "aaj ki bikri kitni hui aur kahan se",
        "aaj kitni baar bikri hui",
        "what is the cash in hand and who counted it",
        "how long has akram toll tax owed",
        "how many days has akram toll tax owed",
        "akram toll tax ka bakaya aur aakhri wasooli ki tareekh",
        // A negation turns the figure into its opposite: today's sale is not what did NOT sell.
        "aaj kitna maal nahi bika",
        "aaj kitni wasooli nahi hui",
        "اکرم ٹول ٹیکس کا بقایا کتنا ہے اور کب سے",
        "ڈوبی رقم کتنی ہے اور کس کس کی",
    ] {
        goes_to_the_model(&h, question);
    }
    h.scan_requests();
}

#[test]
fn guard_8_a_second_figure_that_a_tier_a_repeat_a_verb_or_a_screen_word_would_drop_is_never_left_out()
 {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = multi_harness(y);
    // The expense alone, and the total dues alone, are answered.
    instant(
        &h,
        "pichle mahine dukan ka kharcha kitna hua",
        &[oracle(&f, "mx.last_month.r29.total.expense")],
    );
    // A tier-1 figure (money received, collecting) beside a plain one is a second ask.
    for question in [
        "pichle mahine dukan ka kharcha aur wasooli kitni hui",
        "last month's shop expenses and money received",
        "aaj kitna commission bana aur paise aaye",
    ] {
        goes_to_the_model(&h, question);
    }
    // The same figure named twice with an "and": two asks that one kind would hide.
    for question in [
        "akram toll tax ka bakaya aur kul bakaya",
        "aaj kele ki bikri aur kul bikri",
    ] {
        goes_to_the_model(&h, question);
    }
    // Collected and still to collect in one question.
    goes_to_the_model(&h, "wasooli kitni hui aur baqi");
    // A word of a screen's name that names no figure ("backup", "index", "rules") leaves its ask out.
    goes_to_the_model(&h, "kul agrahi aur doobi raqam kitni hai");
    goes_to_the_model(&h, "aaj ki bikri kitni hui aur backup");
    goes_to_the_model(&h, "aaj ki bikri aur index kitna hai");
    goes_to_the_model(&h, "aaj ki bikri kitni hui aur rules");
    // A verb of selling beside another figure is an ask of its own.
    goes_to_the_model(&h, "maal bika aur bacha kitna");
    goes_to_the_model(&h, "aaj jo bika uska commission aur roqar kitni hai");
    // A verb of buying beside another figure is an ask of its own.
    goes_to_the_model(
        &h,
        "akram toll tax ki is mahine ki khareed aur kul commission kitna",
    );
    // Both sides named and one figure: the other side is left out.
    for question in [
        "gahakon ka bakaya aur byopariyon ko kitna dena hai",
        "gahakon ka bakaya aur byopariyon ka dena kitna hai",
        "buyers aur sellers ka bakaya kitna hai",
        "gahakon aur byopariyon ka bakaya kitna hai",
        "buyers and sellers dues how much",
    ] {
        goes_to_the_model(&h, question);
    }
    // The one word "bakaya" must not become the sellers' payable for both sides.
    goes_to_the_model(&h, "gahakon ka bakaya aur byopariyon ki peshgi kitni hai");
    // A balance dropped beside the cash word, a collecting ask after an "and", the stock under a "baqi".
    goes_to_the_model(&h, "roqar kitni hai aur kitna lena hai");
    goes_to_the_model(&h, "akram toll tax ka bakaya kitna hai aur wasooli kitni");
    goes_to_the_model(&h, "aaj ki bikri aur maal kitna baqi hai");
    // The same figure of a party and of the shop.
    goes_to_the_model(&h, "akram toll tax ka aur kul bakaya kitna hai");
    goes_to_the_model(&h, "akram toll tax ka bakaya, kul bakaya");
    // A seller's commission is two sentences but one asked figure: the second ask still counts.
    goes_to_the_model(
        &h,
        "rahim ka is mahine commission kitna aur wasooli kitni hui",
    );
    goes_to_the_model(&h, "rahim ka is saal ka commission kitna hai aur kitne ka");
    h.scan_requests();
}

#[test]
fn several_figures_of_one_scope_are_one_line_each_in_the_order_asked() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = multi_harness(y);
    let sales = oracle(&f, "tiles.sales_today");
    let bills = oracle(&f, "tiles.count_today");
    let commission = oracle(&f, "tiles.commission_today");
    let cash = oracle(&f, "tiles.cash");
    let dues = oracle(&f, "tiles.buyers_dues");
    let payable = oracle(&f, "tiles.sellers_payable");
    // D120, D121: the shop's day, three figures, each from its own tile.
    instant(
        &h,
        "today's sales, the number of bills and the commission",
        &[sales, bills, commission],
    );
    instant(
        &h,
        "aj ki kul bikri, kitne bil bane aur kamishan kitna hoa",
        &[sales, bills, commission],
    );
    // The order asked is the order shown.
    instant(
        &h,
        "today's commission, sales and number of bills",
        &[commission, sales, bills],
    );
    instant(&h, "today's sale and the number of bills", &[sales, bills]);
    // Figures of NOW beside a figure of today: each line names its own scope.
    instant(&h, "the cash in hand and today's sales", &[cash, sales]);
    instant(&h, "aaj ki bikri aur roqar kitni hai", &[sales, cash]);
    instant(
        &h,
        "roqar kitni hai aur gahakon ka bakaya kitna hai",
        &[cash, dues],
    );
    instant(
        &h,
        "buyers dues and sellers payable how much",
        &[dues, payable],
    );
    // The buyers' dues leave the dead amount out, so the two are separate figures.
    instant(
        &h,
        "buyers dues and the dead amount, how much",
        &[dues, oracle(&f, "tiles.dead")],
    );
    // One item and one day: the item's own row of the view.
    instant(
        &h,
        "today's sales and commission of banana",
        &[
            oracle(&f, "ix.today.banana.gross"),
            oracle(&f, "ix.today.banana.comm_total"),
        ],
    );
    h.scan_requests();
}

#[test]
fn several_figures_are_refused_when_one_is_unsafe_or_has_no_line() {
    let y = year::seed::build();
    let h = multi_harness(y);
    for question in [
        // A figure of NOW beside a sale of last week would be read as the closing cash of that week.
        "pichle hafte ki bikri aur roqar kitni hai",
        "last week's sales and the cash in hand",
        // "so far" folded into "today": which period belongs to which figure is not known.
        "the value of the fruit taken so far and today's sales",
        "fruit taken so far and today's sales",
        "today's sales and the commission so far",
        // A now-figure with a period, or a total word beside a named scope.
        "yesterday's sales and the dead amount",
        "akram toll tax ki is mahine ki bikri aur kul commission",
        // The balance holds the advance and the payable: three lines would read as three sums of money.
        "rahim ka bakaya, dena aur peshgi kitni hai",
        // The total dues already hold the dead amount.
        "total dues and the dead amount, how much",
        "the dead amount and total dues",
        // Receipts of the buyers only is no line (R23 lists every receipt): the sale alone would be partial.
        "aaj ki bikri aur gahakon se wasooli kitni hui",
        // The fruit taken is given only when its value is asked: a dropped line would leave an ask out.
        "today's sales and the fruit taken",
        // The same figure named twice is one line twice, which would read as two sums of money.
        "kasur ke gahakon ka bakaya aur dues kitna hai",
        // A commission and a laga together: the commission holds the laga.
        "aaj ki bikri aur commission aur laga",
        // Two how-much words beside the totals of a report: a line the app keeps back would leave one out.
        "is mahine bikri kitni aur kharcha kitna",
        // Two periods.
        "today's sales and last month's commission",
        // A period folded away: "kal" read as total, a month and days made one, "so far" taken into the day.
        "is mahine ka commission aur kal bikri kitni hui",
        "1 se 10 tak ki bikri aur is mahine ka commission",
        // A name or a period between the figures belongs to the second one only.
        "aaj ki bikri aur akram toll tax ka commission",
        "kul bikri aur aaj ka commission",
        // A word for the whole of something beside a named scope, in every spelling the app knows.
        "kele ki aaj ki bikri aur saara commission kitna hua",
        "kele ki aaj ki bikri aur tamam commission kitna hua",
    ] {
        goes_to_the_model(&h, question);
    }
    h.scan_requests();
}

#[test]
fn each_guard_sets_its_own_flag_on_its_own_wording() {
    let y = year::seed::build();
    let h = production_harness(y);
    // (question, which flag must be set). A guard that goes dead stays red here even when another guard
    // happens to refuse the same question.
    let open_ask = |u: &Understanding| u.open_ask;
    let lower_ask = |u: &Understanding| u.lower_ask;
    let merged = |u: &Understanding| u.period_merged;
    let shop_word = |u: &Understanding| u.shop_word;
    let shop_after_and = |u: &Understanding| u.shop_after_and;
    let stray_screen = |u: &Understanding| u.stray_screen_word;
    let period_between = |u: &Understanding| u.period_between;
    let both_sides = |u: &Understanding| u.buyer_word && u.seller_word;
    type Flag = fn(&Understanding) -> bool;
    let table: Vec<(&str, &str, Flag)> = vec![
        (
            "akram toll tax ka bakaya kitna hai aur kab se",
            "open_ask",
            open_ask,
        ),
        (
            "akram toll tax ka bakaya kitna hai aur kitne din se",
            "open_ask",
            open_ask,
        ),
        (
            "dead amount kitna hai aur kin logon ka",
            "open_ask",
            open_ask,
        ),
        ("aaj kitna maal nahi bika", "open_ask", open_ask),
        (
            "how many days has akram toll tax owed",
            "open_ask",
            open_ask,
        ),
        (
            "akram toll tax ka bakaya aur aakhri wasooli ki tareekh",
            "open_ask",
            open_ask,
        ),
        (
            "pichle mahine dukan ka kharcha aur wasooli kitni hui",
            "lower_ask",
            lower_ask,
        ),
        (
            "akram toll tax ka bakaya aur kul bakaya",
            "lower_ask",
            lower_ask,
        ),
        (
            "akram toll tax ka bakaya, kul bakaya",
            "lower_ask",
            lower_ask,
        ),
        ("wasooli kitni hui aur baqi", "lower_ask", lower_ask),
        (
            "akram toll tax ka bakaya kitna hai aur wasooli kitni",
            "lower_ask",
            lower_ask,
        ),
        (
            "aaj ki bikri aur maal kitna baqi hai",
            "lower_ask",
            lower_ask,
        ),
        ("roqar kitni hai aur kitna lena hai", "lower_ask", lower_ask),
        (
            "gahakon ka bakaya aur byopariyon ki peshgi kitni hai",
            "lower_ask",
            lower_ask,
        ),
        (
            "akram toll tax ki is mahine ki khareed aur kul commission kitna",
            "lower_ask",
            lower_ask,
        ),
        ("maal bika aur bacha kitna", "lower_ask", lower_ask),
        (
            "the value of the fruit taken so far and today's sales",
            "period_merged",
            merged,
        ),
        (
            "is mahine ka commission aur kal bikri kitni hui",
            "period_merged",
            merged,
        ),
        (
            "1 se 10 tak ki bikri aur is mahine ka commission",
            "period_merged",
            merged,
        ),
        (
            "kele ki aaj ki bikri aur saara commission kitna hua",
            "shop_word",
            shop_word,
        ),
        (
            "akram toll tax ka aur kul bakaya kitna hai",
            "shop_after_and",
            shop_after_and,
        ),
        (
            "aaj ki bikri kitni hui aur backup",
            "stray_screen_word",
            stray_screen,
        ),
        (
            "aaj ki bikri aur index kitna hai",
            "stray_screen_word",
            stray_screen,
        ),
        (
            "kul bikri aur aaj ka commission",
            "period_between",
            period_between,
        ),
        (
            "gahakon ka bakaya aur byopariyon ko kitna dena hai",
            "buyer and seller words",
            both_sides,
        ),
    ];
    for (question, name, flag) in table {
        let u = understood(&h, question);
        assert!(flag(&u), "{question}: {name} is not set");
    }
    // And none of the flags is set by the plain questions the app answers. (`period_between` is set for "the cash
    // in hand and today's sales" and refuses only a period figure that comes first, so it is not read here.)
    for question in [
        "akram toll tax ka bakaya kitna hai",
        "aaj ki bikri kitni hui",
        "today's sales, the number of bills and the commission",
        "the cash in hand and today's sales",
        "akram toll tax ka bakaya aur arshad ka bakaya",
        "galle me abhi kitna cash hai",
        "what is the cash balance of the cash book this month",
        "tahir ne kitni peshgi li hui hai",
        "kele ka aaj ka aakhri kis rate par bika",
    ] {
        let u = understood(&h, question);
        assert!(
            !(u.open_ask
                || u.lower_ask
                || u.period_merged
                || u.shop_after_and
                || u.stray_screen_word),
            "{question}: a guard refused a plain question"
        );
    }
}

#[test]
fn plain_questions_that_a_word_could_wrongly_refuse_are_still_answered() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = multi_harness(y);
    // "li" is no buying: it is the verb of "peshgi li hui hai" (an advance taken), asked as the seller's advance.
    instant(
        &h,
        "tahir ne kitni peshgi li hui hai",
        &[oracle(&f, "r26.tahir.open")],
    );
    // Two parties, each with the same figure: one line for each, not a repeat.
    instant(
        &h,
        "akram toll tax ka bakaya aur arshad ka bakaya",
        &[
            oracle_balance(&f, "buyer.akram_toll_tax.balance"),
            oracle_balance(&f, "buyer.arshad.balance"),
        ],
    );
    // Two words of one noun phrase are one figure.
    instant(
        &h,
        "galle me abhi kitna cash hai",
        &[oracle(&f, "tiles.cash")],
    );
    h.scan_requests();
}

#[test]
fn with_the_gate_off_two_measures_go_to_the_model_and_one_measure_for_several_names_is_instant() {
    // Lead decision D333: PRD FR-7.90 sends two-part questions to the model. The gate
    // `MULTI_MEASURE_INSTANT` is OFF, which is the default of the harness too.
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // The harness starts with the app's own gate; this test is about the OFF path, whatever the gate is.
    assert_eq!(
        h.runtime.multi_measure(),
        mandi_assistant::understand::MULTI_MEASURE_INSTANT
    );
    h.runtime.set_multi_measure(false);
    for question in [
        "today's sales, the number of bills and the commission",
        "today's sales and commission",
        "kal ki bikri aur commission",
        "the cash in hand and today's sales",
        "roqar kitni hai aur gahakon ka bakaya kitna hai",
        "buyers dues and sellers payable how much",
        "today's sales and commission of banana",
        "faisal ka bakaya aur commission",
    ] {
        goes_to_the_model(&h, question);
    }
    // One measure for several named parties or items is not a two-measure question: one line for each.
    instant(
        &h,
        "akram toll tax ka bakaya aur arshad ka bakaya",
        &[
            oracle_balance(&f, "buyer.akram_toll_tax.balance"),
            oracle_balance(&f, "buyer.arshad.balance"),
        ],
    );
    h.scan_requests();
}
