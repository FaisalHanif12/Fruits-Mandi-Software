//! Plan 82 (PRD FR-7.90 v1.42, FR-7.77, FR-9.3): a named party's commission or sale over a period is
//! answered by the app itself, with NO model call, in the words the Owner really types.
//!
//! The model here is a client that counts every call and fails it, so an answer proves that no call was
//! made. The books are the client's demo of D8 (today 08-11-2026), whose figures are worked by hand in
//! `support/worked.rs`:
//!
//! - Mani is a BUYER: 20 mango at 1,000 on 03-11-2026. Value 20,000; his commission 6% = 1,200; his Laga
//!   20 x 8 = 160; the two together 1,360.
//! - Rahim is a SELLER: 8 lines, 66,500 in all (45,500 this year, 22,500 in November, 0 in October).
//!   Commission charged to him 6% (3,990 in all, 2,730 this year, 1,350 in November). Commission on his
//!   goods = his 6% + the buyers' 6% + Laga 8 a نگ (100 نگ in all, 70 this year, 35 in November):
//!   8,780 in all, 6,020 this year, 2,980 in November.
//!
//! The first three questions are the ones of the question log (ids 42 to 44) that took 22 to 25 seconds and
//! ended as an error.
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

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use harness::{Harness, answer_of, figures, money};
use mandi_assistant::answer::{AnswerPart, AnswerSource};
use mandi_assistant::ask::{
    AskOutcome, AskReply, AssistantAskInput, AssistantPickInput, FailureKind, PickChoice,
};
use mandi_assistant::client::{CallError, ModelClient, Reply};
use mandi_assistant::run::{ask, pick};
use mandi_assistant::step::AssistantStep;
use mandi_services::assistant;
use mandi_services::core::accounts::AccountType;
use serde_json::Value;

/// A model that must not be called: every call is counted and fails.
#[derive(Default)]
struct NoModel {
    calls: AtomicUsize,
}

impl NoModel {
    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ModelClient for NoModel {
    fn retry_wait(&self) -> Duration {
        Duration::ZERO
    }

    fn call(&self, _key: &str, _body: &Value, _timeout: Duration) -> Result<Reply, CallError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(CallError::new(FailureKind::Other))
    }
}

/// The books as the shop runs them: the app's own reads, answers and speed-ups on.
fn production(w: world::World) -> Harness {
    let h = Harness::over(w);
    h.runtime.set_local(true);
    h.runtime.set_read_first(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

fn ask_with(h: &Harness, client: &NoModel, question: &str) -> (AskReply, Duration) {
    let emit = |_s: AssistantStep| {};
    let deps = h.deps(client, &emit);
    let started = Instant::now();
    let reply = ask(
        &deps,
        &AssistantAskInput {
            text: question.to_owned(),
            conversation_id: None,
        },
    )
    .expect("ask");
    (reply, started.elapsed())
}

/// The dates the answer names, in order, `YYYY-MM-DD`.
fn dates_of(reply: &AskReply) -> Vec<String> {
    answer_of(reply)
        .parts
        .iter()
        .filter_map(|p| match p {
            AnswerPart::Date { iso } => Some(iso.clone()),
            _ => None,
        })
        .collect()
}

fn shown(reply: &AskReply) -> Vec<i64> {
    figures(answer_of(reply))
        .into_iter()
        .map(|(cell, _)| money(cell))
        .collect()
}

/// The last row of the question log: finish reason, model calls and whether it counted.
fn last_row(h: &Harness) -> (Option<String>, Option<i64>, bool) {
    let (rows, _, _) = assistant::list_questions(&h.world.env.ctx, 0, 100).unwrap();
    let row = rows.into_iter().max_by_key(|r| r.id).expect("a row");
    (row.finish_reason, row.model_calls, row.counted)
}

/// One wording: the question, the figures it must show in order (whole rupees) and the dates it names
/// (`""` for a day or "today", which the sentence writes in words).
struct Case {
    question: &'static str,
    figures: &'static [i64],
    from: &'static str,
    to: &'static str,
}

const ALL_FROM: &str = "2025-11-20";
const TODAY: &str = "2026-11-08";
const YEAR_FROM: &str = "2026-01-01";
const MONTH_FROM: &str = "2026-11-01";

/// Mani's three figures: his commission, his Laga, the two together.
const MANI: &[i64] = &[1_200, 160, 1_360];
const MANI_NONE: &[i64] = &[0, 0, 0];

#[rustfmt::skip]
const CASES: &[Case] = &[
    // ---- the questions of the log ----
    Case { question: "mani ne abhi tak shop ko kitni commission banwa kr di ?", figures: MANI, from: ALL_FROM, to: TODAY },
    Case { question: "mani ne abhi tak shop ko kitni commission banwa kr di ?", figures: MANI, from: ALL_FROM, to: TODAY },
    Case { question: "mani gahaq se hum ne kitni commission hasil ki abhi tak ?", figures: MANI, from: ALL_FROM, to: TODAY },
    // ---- a buyer: every period word ----
    Case { question: "mani se kitna commission mila abhi tak", figures: MANI, from: ALL_FROM, to: TODAY },
    Case { question: "mani se kitni commission li abhi tak", figures: MANI, from: ALL_FROM, to: TODAY },
    Case { question: "mani sy kitna commission mila abhi tak", figures: MANI, from: ALL_FROM, to: TODAY },
    Case { question: "mani ka commission kitna hua abhi tak", figures: MANI, from: ALL_FROM, to: TODAY },
    Case { question: "mani ki taraf se kitna commission bana is mahine", figures: MANI, from: MONTH_FROM, to: TODAY },
    Case { question: "mani ki tarf se kitni commission aai is mahine", figures: MANI, from: MONTH_FROM, to: TODAY },
    Case { question: "mani gahak se is mahine hamen kitna commission mila", figures: MANI, from: MONTH_FROM, to: TODAY },
    Case { question: "mani k commission kitna hua is saal", figures: MANI, from: YEAR_FROM, to: TODAY },
    Case { question: "mani ne is saal humein kitna commission diya", figures: MANI, from: YEAR_FROM, to: TODAY },
    Case { question: "mani se pichle mahine kitna commission mila", figures: MANI_NONE, from: "2026-10-01", to: "2026-10-31" },
    Case { question: "mani se october mein kitna commission mila", figures: MANI_NONE, from: "2026-10-01", to: "2026-10-31" },
    Case { question: "mani ki kitni bikri hui is mahine", figures: &[20_000], from: MONTH_FROM, to: TODAY },
    // ---- a buyer: English ----
    Case { question: "commission from mani this month", figures: MANI, from: MONTH_FROM, to: TODAY },
    Case { question: "how much commission did we get from mani so far", figures: MANI, from: ALL_FROM, to: TODAY },
    Case { question: "how much commission have we earned from mani till now", figures: MANI, from: ALL_FROM, to: TODAY },
    Case { question: "how much commission did we get from mani in october", figures: MANI_NONE, from: "2026-10-01", to: "2026-10-31" },
    // ---- a buyer: Urdu letters ----
    Case { question: "مانی سے اب تک کتنا کمیشن ملا", figures: MANI, from: ALL_FROM, to: TODAY },
    Case { question: "مانی نے دکان کو کتنا کمیشن دیا ابھی تک", figures: MANI, from: ALL_FROM, to: TODAY },
    // ---- a seller: commission charged to him, then on his goods ----
    Case { question: "rahim se is mahine kitna commission mila", figures: &[1_350, 2_980], from: MONTH_FROM, to: TODAY },
    Case { question: "rahim se is saal kitna commission mila", figures: &[2_730, 6_020], from: YEAR_FROM, to: TODAY },
    Case { question: "rahim ne shop ko is saal kitni commission banwa kr di", figures: &[2_730, 6_020], from: YEAR_FROM, to: TODAY },
    Case { question: "رحیم نے آج تک کتنی کمیشن بنوا کر دی", figures: &[3_990, 8_780], from: ALL_FROM, to: TODAY },
    Case { question: "rahim se aaj kitna commission mila", figures: &[0, 0], from: "", to: "" },
    // ---- a seller: sales ----
    Case { question: "rahim ki kitni bikri hui abhi tak", figures: &[66_500], from: ALL_FROM, to: TODAY },
    Case { question: "rahim ki is saal ki bikri", figures: &[45_500], from: YEAR_FROM, to: TODAY },
    Case { question: "rahim ne pichle mahine kitni bikri ki", figures: &[0], from: "2026-10-01", to: "2026-10-31" },
    Case { question: "how much did rahim sell this month", figures: &[22_500], from: MONTH_FROM, to: TODAY },
    Case { question: "رحیم کی اس مہینے کتنی بکری ہوئی", figures: &[22_500], from: MONTH_FROM, to: TODAY },
    // ---- a party's commission and sale together (PRD FR-7.90, v1.42) ----
    Case { question: "rahim se november mein kitna commission mila aur kitni bikri hui", figures: &[1_350, 2_980, 22_500], from: MONTH_FROM, to: TODAY },
];

#[test]
fn a_partys_commission_or_sale_is_answered_by_the_app_with_no_model_call() {
    assert!(CASES.len() >= 23, "the three logged questions and 20 more");
    let h = production(worked::d8().w);
    for case in CASES {
        let client = NoModel::default();
        let (reply, took) = ask_with(&h, &client, case.question);
        assert_eq!(
            reply.outcome,
            AskOutcome::Answered,
            "{}\n{reply:#?}",
            case.question
        );
        assert_eq!(
            client.calls(),
            0,
            "the model was called for {}",
            case.question
        );
        assert!(
            took < Duration::from_secs(1),
            "{} took {took:?}",
            case.question
        );
        // The checked figures of the commission page or the analytics view, in whole rupees.
        assert_eq!(shown(&reply), case.figures, "{}", case.question);
        // The dates it says are the dates it read.
        if !case.from.is_empty() {
            let dates = dates_of(&reply);
            assert_eq!(
                dates.first().map(String::as_str),
                Some(case.from),
                "{}",
                case.question
            );
            assert_eq!(
                dates.last().map(String::as_str),
                Some(case.to),
                "{}",
                case.question
            );
        }
        // Not counted in the daily limit, and logged as the app's own answer.
        assert_eq!(reply.count_today, 0, "{}", case.question);
        let (finish, model_calls, counted) = last_row(&h);
        assert_eq!(finish.as_deref(), Some("local"), "{}", case.question);
        assert_eq!(model_calls, Some(0), "{}", case.question);
        assert!(!counted, "{}", case.question);
        h.scan_requests();
    }
}

#[test]
fn a_commission_answer_has_the_button_of_the_commission_page_of_that_party() {
    let h = production(worked::d8().w);
    // A buyer: the commission page of the buyer Mani, over the dates of the answer (FR-9.3).
    let client = NoModel::default();
    let (reply, _) = ask_with(
        &h,
        &client,
        "mani ne abhi tak shop ko kitni commission banwa kr di ?",
    );
    let sources = &answer_of(&reply).sources;
    assert!(
        sources.iter().any(|s| matches!(
            s,
            AnswerSource::Commission { party, from, to, .. }
                if party.record.name_en == "Mani" && from == ALL_FROM && to == TODAY
        )),
        "{sources:#?}"
    );
    // A seller: the page of the seller Rahim.
    let (reply, _) = ask_with(&h, &client, "rahim se is mahine kitna commission mila");
    assert!(
        answer_of(&reply).sources.iter().any(|s| matches!(
            s,
            AnswerSource::Commission { party, from, to, .. }
                if party.record.name_en == "Rahim" && from == MONTH_FROM && to == TODAY
        )),
        "{:#?}",
        answer_of(&reply).sources
    );
    assert_eq!(client.calls(), 0);
}

#[test]
fn what_the_app_is_not_sure_of_still_goes_to_the_model() {
    // Never guess. No period: "so far" would be the app's guess, and the model asks or reads.
    // "aur hasil": a word beside an "and" may be a second name, so it is not dropped.
    // A noun nothing explains ("bijli") may be a filter the figure would ignore.
    let h = production(worked::d8().w);
    for question in [
        "mani se kitna commission mila",
        "mani aur hasil ki commission abhi tak",
        "mani ka bijli par commission abhi tak banwa kr di",
    ] {
        let client = NoModel::default();
        let (reply, _) = ask_with(&h, &client, question);
        assert!(
            client.calls() >= 1,
            "{question} was answered by the app: {reply:#?}"
        );
    }
}

#[test]
fn a_name_nobody_has_is_not_found_with_no_model_call() {
    let h = production(worked::d8().w);
    let client = NoModel::default();
    let (reply, _) = ask_with(
        &h,
        &client,
        "qqzzx ne abhi tak shop ko kitni commission banwa kr di",
    );
    assert_eq!(client.calls(), 0, "{reply:#?}");
    assert_eq!(reply.outcome, AskOutcome::NotFound, "{reply:#?}");
    assert_eq!(reply.count_today, 0);
}

/// The worked day after Events 1 to 5 (today 10-10-2026) with a second buyer named like the first:
/// Akram Toll Tax (0020, bought 12,000 on 10-10) and Akram K (no sales).
fn two_akrams() -> Harness {
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    world::add_account_tagged(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        ("اکرم", "Akram"),
        (true, ""),
        "K",
    );
    production(w)
}

#[test]
fn several_records_of_a_name_ask_which_one_with_no_model_call_and_the_click_is_answered_the_same_way()
 {
    let h = two_akrams();
    let client = NoModel::default();
    let (reply, took) = ask_with(&h, &client, "akram se is mahine kitna commission mila");
    assert_eq!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
    assert_eq!(reply.options.len(), 2, "{reply:#?}");
    assert_eq!(client.calls(), 0);
    assert!(took < Duration::from_secs(1), "{took:?}");
    // Not counted, and logged as the app's own question.
    assert_eq!(reply.count_today, 0);
    let (finish, model_calls, counted) = last_row(&h);
    assert_eq!(finish.as_deref(), Some("local"));
    assert_eq!(model_calls, Some(0));
    assert!(!counted);
    // The click on Akram Toll Tax: his commission 720, his Laga 160, the two together 880 (PRD 4.4,
    // E2E-33), over 01-10-2026 to 10-10-2026, with no model call.
    let toll = reply
        .options
        .iter()
        .find(|o| o.name.name_en == "Akram Toll Tax")
        .expect("Akram Toll Tax is an option")
        .key
        .clone();
    let emit = |_s: AssistantStep| {};
    let deps = h.deps(&client, &emit);
    let clicked = pick(
        &deps,
        &AssistantPickInput {
            question_id: reply.question_id,
            choice: PickChoice::Match { key: toll },
        },
    )
    .expect("pick");
    assert_eq!(clicked.outcome, AskOutcome::Answered, "{clicked:#?}");
    assert_eq!(client.calls(), 0, "the click called the model");
    assert_eq!(shown(&clicked), [720, 160, 880]);
    assert_eq!(
        dates_of(&clicked).first().map(String::as_str),
        Some("2026-10-01")
    );
    assert_eq!(clicked.count_today, 0);
    let (finish, model_calls, counted) = last_row(&h);
    assert_eq!(finish.as_deref(), Some("local"));
    assert_eq!(model_calls, Some(0));
    assert!(!counted);
    h.scan_requests();
}

#[test]
fn the_other_record_of_the_name_is_answered_for_himself() {
    let h = two_akrams();
    let client = NoModel::default();
    let (reply, _) = ask_with(
        &h,
        &client,
        "how much commission did we get from akram this month",
    );
    assert_eq!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
    let tagged = reply
        .options
        .iter()
        .find(|o| o.name.tag.as_deref() == Some("K"))
        .expect("Akram K is an option")
        .key
        .clone();
    let emit = |_s: AssistantStep| {};
    let deps = h.deps(&client, &emit);
    let clicked = pick(
        &deps,
        &AssistantPickInput {
            question_id: reply.question_id,
            choice: PickChoice::Match { key: tagged },
        },
    )
    .expect("pick");
    // He has bought nothing: three zeros, never his namesake's 880.
    assert_eq!(clicked.outcome, AskOutcome::Answered, "{clicked:#?}");
    assert_eq!(shown(&clicked), [0, 0, 0]);
    assert_eq!(client.calls(), 0);
}

#[test]
fn a_balance_with_several_records_is_still_asked_by_the_model_path() {
    // Only a commission or a sale is asked locally. The golden question E2E-52 ("akram ka bakaya")
    // keeps the model's way and counts as a question.
    let h = two_akrams();
    let client = NoModel::default();
    let (_, _) = ask_with(&h, &client, "akram ka bakaya kitna hai");
    assert!(client.calls() >= 1);
}
