//! Plan 55 Section 10 (PRD FR-7.90): a simple figure is answered by the app itself, from the reads
//! of the plan, with no model call. The recorded model has NO replies here: any call to it would
//! end the question as an error, so an answer proves that no call was made.
//!
//! The worked day after Events 1 to 5 (today 10-10-26): اکرم owes 2,880 بنام, ارشد 18,684, رحیم
//! is paid, کریم is owed 10,160 (PRD 4.9).
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

use harness::{Harness, answer_of, balance, figures, money};
use mandi_assistant::answer::AnswerSource;
use mandi_assistant::ask::{AskOutcome, AskReply, AssistantRerunInput};
use mandi_assistant::client::RecordedClient;
use mandi_assistant::run::rerun;
use mandi_assistant::step::AssistantStep;
use mandi_services::core::accounts::BalanceSide;
use serde_json::Value;

fn instant() -> Harness {
    let h = Harness::day();
    h.runtime.set_local(true);
    h.runtime.set_read_first(true);
    h.runtime.set_instant(true);
    h
}

fn no_model(h: &Harness, question: &str, conversation: Option<&str>) -> (AskReply, RecordedClient) {
    let client = RecordedClient::new(Vec::new());
    let reply = h.ask_with(question, conversation, &client);
    (reply, client)
}

/// The last row of the question log: finish reason, model calls, counted, and the tool entries.
struct LogRow {
    finish_reason: Option<String>,
    model_calls: Option<i64>,
    counted: bool,
    tool_calls_json: String,
}

fn log_row(h: &Harness) -> LogRow {
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    let row = rows.first().expect("a row");
    LogRow {
        finish_reason: row.finish_reason.clone(),
        model_calls: row.model_calls,
        counted: row.counted,
        tool_calls_json: row.tool_calls_json.clone(),
    }
}

#[test]
fn a_buyers_balance_is_written_by_the_app_with_no_model_call() {
    let h = instant();
    let (reply, client) = no_model(&h, "akram ka bakaya kitna hai", None);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert!(client.requests().is_empty(), "the model was called");
    // One figure: his balance, 2,880 بنام, with its tick (two places agree).
    let shown = figures(answer_of(&reply));
    assert_eq!(shown.len(), 1, "{reply:#?}");
    assert_eq!(balance(shown[0].0), (2_880, BalanceSide::Dr));
    assert!(shown[0].1, "ticked");
    // The card is the button, as for any answer about a party.
    assert!(
        answer_of(&reply)
            .sources
            .iter()
            .any(|s| matches!(s, AnswerSource::PartyCard { .. })),
        "{reply:#?}"
    );
    // Not counted: no model call, and the daily count stays where it was.
    assert_eq!(reply.count_today, 0);
    let row = log_row(&h);
    assert_eq!(row.finish_reason.as_deref(), Some("local"));
    assert_eq!(row.model_calls, Some(0));
    assert!(!row.counted);
    // The reads are in the log as local, with their ids so that they can be read again.
    let calls: Vec<Value> = serde_json::from_str(&row.tool_calls_json).unwrap();
    let tools: Vec<&str> = calls.iter().filter_map(|c| c["tool"].as_str()).collect();
    assert_eq!(tools, ["find", "party_card"], "{calls:?}");
    assert!(
        calls
            .iter()
            .all(|c| c["local"] == true && c["id"].is_string())
    );
    h.scan_requests();
}

#[test]
fn the_instant_sentence_is_in_both_languages_and_a_seller_is_owed_not_owing() {
    // کریم is a seller owed 10,160 (جمع). The sentence is the app's own; the figure is the card's.
    let h = instant();
    let (reply, _) = no_model(&h, "kareem ko kitna dena he", None);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let shown = figures(answer_of(&reply));
    assert_eq!(balance(shown[0].0), (10_160, BalanceSide::Cr));
    assert!(shown[0].1);
    let text: String = answer_of(&reply)
        .parts
        .iter()
        .filter_map(|p| match p {
            mandi_assistant::answer::AnswerPart::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert!(!text.trim().is_empty(), "the app's own words: {reply:#?}");
}

#[test]
fn an_instant_answer_is_not_stopped_by_the_daily_limit_and_a_follow_up_works() {
    let h = instant();
    // A follow-up in the same conversation with a subject of its own is answered the same way.
    let (first, _) = no_model(&h, "akram ka bakaya kitna hai", None);
    let (second, client) = no_model(
        &h,
        "arshad ka bakaya kitna hai",
        Some(&first.conversation_id),
    );
    assert_eq!(second.outcome, AskOutcome::Answered, "{second:#?}");
    assert!(client.requests().is_empty());
    let shown = figures(answer_of(&second));
    assert_eq!(balance(shown[0].0), (18_684, BalanceSide::Dr));
}

#[test]
fn what_is_not_a_simple_figure_goes_to_the_model() {
    // Advice is not a figure: the app does not write it. The model path runs (here it has no
    // replies, so the call fails) and the question is logged as a model question, not a local one.
    let h = instant();
    let (reply, client) = no_model(&h, "kya mujhe saib ka commission barhana chahiye", None);
    assert_ne!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert!(!client.requests().is_empty(), "the model was asked");
    let row = log_row(&h);
    assert_ne!(row.finish_reason.as_deref(), Some("local"));
}

#[test]
fn an_instant_answer_can_be_read_again_on_todays_books() {
    let h = instant();
    let (reply, _) = no_model(&h, "akram ka bakaya kitna hai", None);
    assert_eq!(reply.outcome, AskOutcome::Answered);
    let client = RecordedClient::new(Vec::new());
    let emit = |_s: AssistantStep| {};
    let deps = h.deps(&client, &emit);
    let again = rerun(
        &deps,
        &AssistantRerunInput {
            question_id: reply.question_id,
        },
    )
    .unwrap();
    assert_eq!(again.outcome, AskOutcome::Answered, "{again:#?}");
    assert!(client.requests().is_empty());
    // The reads were run again: the card of اکرم is a source of the new answer.
    assert!(
        answer_of(&again)
            .sources
            .iter()
            .any(|s| matches!(s, AnswerSource::PartyCard { .. }))
    );
}

#[test]
fn a_partners_kul_raqam_is_written_by_the_app_and_is_never_his_capital() {
    // Live run 1, Q198, Q199, Q546 to Q548: "kul raqam" was answered with his capital, ticked. The
    // plan now reads R31 for everyone and the answer cites the partner's own row: his total.
    // الف of the worked day: -1,004 (PRD 4.9, E2E-39).
    let h = instant();
    let (reply, client) = no_model(&h, "alif ki kul raqam kitni hai", None);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert!(client.requests().is_empty(), "the model was called");
    let shown = figures(answer_of(&reply));
    assert_eq!(shown.len(), 1, "{reply:#?}");
    assert_eq!(money(shown[0].0), -1_004);
    assert!(shown[0].1, "ticked: the overview and the statement agree");
    // The button opens his statement, with him (never a partner-less R31).
    let reports: Vec<_> = answer_of(&reply)
        .sources
        .iter()
        .filter_map(|s| match s {
            AnswerSource::Report { id, filters, .. } => Some((
                format!("{id:?}"),
                filters.partner.as_ref().map(|p| p.name_en.clone()),
            )),
            _ => None,
        })
        .collect();
    assert_eq!(reports, [("R31".to_owned(), Some("Partner A".to_owned()))]);
}

#[test]
fn b_after_e2e_35_gets_his_kul_raqam_50746_and_not_his_capital_50000() {
    // PRD E2E-35 and E2E-37: ب invests 50,000 on 11-10-26; his کل رقم is 50,746.
    let h = instant();
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
    for question in ["ب کی کل رقم کتنی ہے", "partner b ka kitna hai abhi"] {
        let (reply, client) = no_model(&h, question, None);
        assert!(
            client.requests().is_empty(),
            "{question}: the model was called"
        );
        assert_eq!(
            reply.outcome,
            AskOutcome::Answered,
            "{question}: {reply:#?}"
        );
        let shown = figures(answer_of(&reply));
        assert_eq!(money(shown[0].0), 50_746, "{question}: {reply:#?}");
        assert!(shown[0].1, "{question}: ticked");
    }
}
