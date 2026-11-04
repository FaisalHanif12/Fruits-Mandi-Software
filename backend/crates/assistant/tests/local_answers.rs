//! Plan 54 S2 (Umar's addition of 04-11-2026) and plan 55 Section 2: a print request, a how-to and
//! a name that no record has are answered by the app itself, with no model call. The recorded model
//! has NO replies here: any call to it would end the question as an error, so an answer proves that
//! no call was made.
//!
//! The worked day (today 10-10-26): اکرم (20) and ارشد (21) are buyers, رحیم (18) and کریم are
//! sellers.
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

use std::path::{Path, PathBuf};

use harness::Harness;
use mandi_assistant::answer::{Answer, AnswerSource, HowStep};
use mandi_assistant::ask::{AskOutcome, AskReply, AskedRole, PickChoice};
use mandi_assistant::client::RecordedClient;
use mandi_reports::reports::{ArgValue, ReportId};

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Writes (with `MANDI_WRITE_FIXTURES=1`) or checks a fixture of the REAL answer of the app, for
/// the screens to copy byte for byte.
fn fixture(name: &str, value: &impl serde::Serialize) {
    let text = serde_json::to_string_pretty(value).unwrap() + "\n";
    let path = folder().join(name);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::write(&path, text).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{name} is missing: run with {WRITE_VARIABLE}=1 to write the fixtures")
    });
    assert!(
        on_disk == text,
        "{name} is not the real answer any more. If the change is on purpose, run with \
         {WRITE_VARIABLE}=1, read the diff and tell frontend."
    );
}

fn request(question: &str) -> serde_json::Value {
    serde_json::json!({ "text": question, "conversation_id": null })
}

/// The progress lines the panel got while the question ran, as fixtures show them.
fn steps_of(h: &Harness) -> Vec<mandi_assistant::step::AssistantStep> {
    h.steps.lock().unwrap().clone()
}

fn ask(question: &str) -> (Harness, AskReply, RecordedClient) {
    let h = Harness::day();
    h.runtime.set_local(true);
    let client = RecordedClient::new(Vec::new());
    let reply = h.ask_with(question, None, &client);
    (h, reply, client)
}

/// The one report source of an answer: its id and filters.
fn report(answer: &Answer, at: usize) -> (ReportId, mandi_assistant::source::SourceReportFilters) {
    match answer.sources.get(at) {
        Some(AnswerSource::Report {
            id,
            filters,
            result_id,
            ..
        }) => {
            assert!(result_id.is_empty(), "a link only: nothing was read");
            (*id, filters.clone())
        }
        other => panic!("not a report source: {other:?}"),
    }
}

fn text_of(step: &HowStep, name: &str) -> String {
    step.args
        .iter()
        .find(|a| a.name == name)
        .map(|a| match &a.value {
            ArgValue::Text { text } => text.clone(),
            other => format!("{other:?}"),
        })
        .unwrap_or_default()
}

fn ids(answer: &Answer) -> Vec<&str> {
    answer.how.iter().map(|s| s.id.as_str()).collect()
}

fn no_model_call(reply: &AskReply, client: &RecordedClient) {
    assert!(client.requests().is_empty(), "the model was called");
    assert_eq!(reply.count_today, 0, "a local answer is not counted");
}

#[test]
fn a_buyers_day_slip_opens_r35_with_him_and_the_day_and_gives_the_steps() {
    let (h, reply, client) = ask("akram ki aaj ki bikri ka print nikalna hai");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    no_model_call(&reply, &client);
    assert!(
        reply.entry_route.is_none(),
        "a print is never a change request"
    );
    let answer = reply.answer.as_ref().unwrap();
    let (id, filters) = report(answer, 0);
    assert_eq!(id, ReportId::R35);
    assert_eq!(
        filters.party.as_ref().unwrap().record.name_en,
        "Akram Toll Tax"
    );
    assert_eq!(filters.as_of, "2026-10-10");
    assert!(
        filters.from.is_empty() && filters.to.is_empty(),
        "one day, no range"
    );
    // The steps are made from the same values.
    assert_eq!(
        ids(answer),
        [
            "assistant-how-open-report",
            "assistant-how-set-party",
            "assistant-how-set-day",
            "assistant-how-press-print"
        ]
    );
    assert_eq!(text_of(&answer.how[0], "report"), "R35");
    assert_eq!(text_of(&answer.how[1], "role"), "buyer");
    assert_eq!(
        text_of(&answer.how[1], "code"),
        filters.party.unwrap().record.code.unwrap()
    );
    assert_eq!(text_of(&answer.how[2], "day"), "2026-10-10");
    fixture("assistant_reply_print_r35_day.json", &reply);
    fixture(
        "assistant_reply_print_r35_day_request.json",
        &request("akram ki aaj ki bikri ka print nikalna hai"),
    );
    fixture("assistant_steps_print.json", &steps_of(&h));
    // The log keeps it as a local answer with no model call and not counted.
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    assert_eq!(rows[0].finish_reason.as_deref(), Some("local"));
    assert_eq!(rows[0].model_calls, Some(0));
    assert!(!rows[0].counted);
    h.scan_requests();
}

#[test]
fn a_buyers_month_opens_r5_with_the_dates_and_offers_r13_as_the_second_button() {
    let (_h, reply, client) = ask("akram ki pichle mahine ki bikri ka print chahiye");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    fixture("assistant_reply_print_r5_range.json", &reply);
    fixture(
        "assistant_reply_print_r5_range_request.json",
        &request("akram ki pichle mahine ki bikri ka print chahiye"),
    );
    no_model_call(&reply, &client);
    let answer = reply.answer.as_ref().unwrap();
    let (id, filters) = report(answer, 0);
    assert_eq!(id, ReportId::R5);
    assert_eq!(
        (filters.from.as_str(), filters.to.as_str()),
        ("2026-09-01", "2026-09-30")
    );
    assert_eq!(
        filters.party.as_ref().unwrap().record.name_en,
        "Akram Toll Tax"
    );
    let (second, second_filters) = report(answer, 1);
    assert_eq!(second, ReportId::R13);
    assert_eq!(second_filters.from, "2026-09-01");
    let steps = ids(answer);
    assert_eq!(steps.last(), Some(&"assistant-how-press-print"));
    assert!(steps.contains(&"assistant-how-set-dates"));
    let dates = answer
        .how
        .iter()
        .find(|s| s.id == "assistant-how-set-dates")
        .unwrap();
    assert_eq!(text_of(dates, "from"), "2026-09-01");
    assert_eq!(text_of(dates, "to"), "2026-09-30");
}

#[test]
fn a_ledger_print_opens_r13_for_the_range_and_names_the_party_and_the_dates() {
    let question = "akram ke khata ka is mahine ka print nikalna hai";
    let (_h, reply, client) = ask(question);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    no_model_call(&reply, &client);
    let answer = reply.answer.as_ref().unwrap();
    let (id, filters) = report(answer, 0);
    assert_eq!(id, ReportId::R13);
    assert_eq!(
        filters.party.as_ref().unwrap().record.name_en,
        "Akram Toll Tax"
    );
    assert_eq!(
        (filters.from.as_str(), filters.to.as_str()),
        ("2026-10-01", "2026-10-10")
    );
    assert_eq!(ids(answer).last(), Some(&"assistant-how-press-print"));
    fixture("assistant_reply_print_r13_ledger.json", &reply);
    fixture(
        "assistant_reply_print_r13_ledger_request.json",
        &request(question),
    );
}

#[test]
fn a_sellers_sales_open_r6_with_him_as_the_seller() {
    let (_h, reply, client) = ask("rahim ki aaj ki bikri ka print nikalna hai");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    no_model_call(&reply, &client);
    let answer = reply.answer.as_ref().unwrap();
    let (id, filters) = report(answer, 0);
    assert_eq!(id, ReportId::R6);
    assert_eq!(filters.seller.as_ref().unwrap().name_en, "Rahim");
    assert!(filters.party.is_none(), "R6 takes `seller`, not `party`");
    assert_eq!(
        (filters.from.as_str(), filters.to.as_str()),
        ("2026-10-10", "2026-10-10")
    );
    assert_eq!(text_of(&answer.how[1], "role"), "seller");
}

#[test]
fn a_whole_day_with_no_party_opens_the_day_summary() {
    let (_h, reply, client) = ask("aaj ki bikri ka print nikalna hai");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    no_model_call(&reply, &client);
    let (id, filters) = report(reply.answer.as_ref().unwrap(), 0);
    assert_eq!(id, ReportId::R34);
    assert_eq!(
        (filters.from.as_str(), filters.to.as_str()),
        ("2026-10-10", "2026-10-10")
    );
}

#[test]
fn a_name_that_no_record_has_ends_as_not_found_with_the_kind_asked_and_no_model_call() {
    let (_h, reply, client) = ask("zubair ki aaj ki bikri ka print nikalna hai");
    assert_eq!(reply.outcome, AskOutcome::NotFound, "{reply:#?}");
    no_model_call(&reply, &client);
    assert!(reply.found_as.is_none());
    assert!(reply.asked_role.is_some());
    assert!(reply.answer.is_none());
}

#[test]
fn a_buyer_asked_as_a_seller_is_found_as_a_buyer_without_asking_the_model() {
    // "Arshad ne kya becha": Arshad is a buyer (plan 52 Section 8), and now no model call is spent.
    let (_h, reply, client) = ask("arshad ne kya becha");
    assert_eq!(reply.outcome, AskOutcome::NotFound, "{reply:#?}");
    no_model_call(&reply, &client);
    assert_eq!(reply.asked_role, Some(AskedRole::Seller));
    let found = reply.found_as.as_ref().expect("found as a buyer");
    assert_eq!(found.role, AskedRole::Buyer);
    assert_eq!(found.name.name_en, "Arshad Multani");
    assert_eq!(reply.options.len(), 1, "his name is offered: one click");
}

#[test]
fn a_click_on_the_name_found_in_the_other_role_gives_the_model_the_question_first() {
    // Money review BUG-6. The app ended "arshad ne kya becha" itself, with only the find in the
    // messages. When the Owner clicks Arshad, the model goes on from those messages: it must see
    // the question first, or it answers something else about him (his balance, or an old figure).
    let (h, reply, client) = ask("arshad ne kya becha");
    assert_eq!(reply.outcome, AskOutcome::NotFound, "{reply:#?}");
    no_model_call(&reply, &client);
    let key = reply.options[0].key.clone();
    let answer = serde_json::json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant",
            "content": serde_json::json!({"kind": "answer", "text": "جی", "show": [],
                "next": [], "choose": null, "entry": null, "nearest": null}).to_string()}}],
        "usage": {"prompt_tokens": 7_200, "completion_tokens": 60,
            "prompt_tokens_details": {"cached_tokens": 7_000},
            "completion_tokens_details": {"reasoning_tokens": 20}},
    });
    let (_picked, requests) = h.pick(reply.question_id, PickChoice::Match { key }, vec![answer]);
    let messages = requests[0]["messages"].as_array().expect("messages");
    let question_at = messages
        .iter()
        .position(|m| {
            m["role"] == "user"
                && m["content"]
                    .as_str()
                    .is_some_and(|c| c.contains("arshad ne kya becha"))
        })
        .expect("the model gets the question");
    let find_at = messages
        .iter()
        .position(|m| m["role"] == "assistant" && m["tool_calls"].is_array())
        .expect("the find the app ran");
    assert!(question_at < find_at, "the question comes before the find");
    let picked_at = messages
        .iter()
        .position(|m| {
            m["role"] == "user" && m["content"].as_str().is_some_and(|c| c.contains("picked"))
        })
        .expect("the click");
    assert!(find_at < picked_at);
}

#[test]
fn a_question_about_tax_says_the_app_does_not_keep_it_and_opens_the_sales_register() {
    // Plan 55 gate 5 and the lead's request of 04-11-2026: the real reply of the compose path for
    // a thing the app does not keep. `answer.parts` holds the app's own sentence (the Fluent
    // message `assistant-outside-tax`), and the nearest thing the app does keep is the button
    // (R2). The frontend copies this fixture to test the real shape.
    let question = "gst kitna lagta hai";
    let (h, reply, client) = ask(question);
    assert_eq!(reply.outcome, AskOutcome::OutsideBooks, "{reply:#?}");
    no_model_call(&reply, &client);
    let answer = reply.answer.as_ref().unwrap();
    assert_eq!(answer.parts.len(), 1, "one sentence, written by the app");
    assert!(
        matches!(&answer.parts[0], mandi_assistant::answer::AnswerPart::Text { text } if !text.is_empty()),
        "{answer:#?}"
    );
    let (id, filters) = report(answer, 0);
    assert_eq!(id, ReportId::R2, "the nearest thing the app keeps");
    assert!(filters.party.is_none());
    assert!(answer.how.is_empty() && answer.tables.is_empty());
    fixture("assistant_reply_outside_tax.json", &reply);
    fixture(
        "assistant_reply_outside_tax_request.json",
        &request(question),
    );
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    assert_eq!(rows[0].finish_reason.as_deref(), Some("local"));
    assert_eq!(rows[0].model_calls, Some(0));
    assert!(!rows[0].counted);
    h.scan_requests();
}

#[test]
fn a_how_to_the_guide_answers_needs_no_model() {
    let (_h, reply, client) = ask("print kaise nikalte hain");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    no_model_call(&reply, &client);
    let answer = reply.answer.as_ref().unwrap();
    assert!(!answer.parts.is_empty());
    assert!(answer.tables.is_empty() && answer.sources.is_empty());
    assert!(reply.entry_route.is_none());
}

#[test]
fn a_question_the_app_is_not_sure_of_still_goes_to_the_model() {
    // A figure with two parts is not a print or a how-to: the model path runs (here it has no
    // replies, so the call fails and the question ends as an error), and it is counted only when
    // a reply arrives.
    let (_h, reply, client) = ask("kela aur saib ka commission kitna hai aur kal ka farq");
    assert!(!client.requests().is_empty(), "the model was asked");
    assert_ne!(reply.outcome, AskOutcome::Answered);
}
