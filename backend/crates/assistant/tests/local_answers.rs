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
    let (picked, requests) = h.pick(reply.question_id, PickChoice::Match { key }, vec![answer]);
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
    // The finds the model has in front of it, in order: the one made as a seller (the local
    // answer's), and the one the click made in his real role, a buyer (live run 1: Q427 to Q429
    // and Q431 ended as "not found" again, because the model was left with the seller find only).
    let finds: Vec<(usize, serde_json::Value)> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m["role"] == "assistant" && m["tool_calls"].is_array())
        .map(|(at, m)| {
            let args = m["tool_calls"][0]["function"]["arguments"]
                .as_str()
                .unwrap_or("{}");
            (at, serde_json::from_str(args).unwrap_or_default())
        })
        .collect();
    assert_eq!(finds.len(), 2, "{finds:?}");
    assert!(
        question_at < finds[0].0,
        "the question comes before the finds"
    );
    assert_eq!(finds[0].1["role"], "seller");
    assert_eq!(
        finds[1].1["role"], "buyer",
        "he is found again as what he is"
    );
    assert_eq!(finds[1].1["text"], finds[0].1["text"]);
    // The click points at the new find, and says what he was chosen as.
    let click = messages
        .iter()
        .rev()
        .find_map(|m| {
            (m["role"] == "user")
                .then(|| serde_json::from_str::<serde_json::Value>(m["content"].as_str()?).ok())
                .flatten()
                .filter(|v| v.get("picked").is_some())
        })
        .expect("the click");
    assert_eq!(click["picked"], "r2.matches.0");
    assert_eq!(click["as"], "buyer");
    assert!(click["note"].as_str().is_some_and(|n| n.contains("bought")));
    assert!(finds[1].0 < messages.len() - 1, "the click is last");
    // The tool ids of the two finds differ, so every tool message has its own call.
    let ids: Vec<&str> = messages
        .iter()
        .filter(|m| m["role"] == "assistant" && m["tool_calls"].is_array())
        .filter_map(|m| m["tool_calls"][0]["id"].as_str())
        .collect();
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1]);
    // The model answered in its one call: not "not found" again.
    assert_ne!(picked.outcome, AskOutcome::NotFound, "{picked:#?}");
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

/// A model reply that finishes with this text.
fn answer_reply(text: &str) -> serde_json::Value {
    serde_json::json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": serde_json::json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": {
            "prompt_tokens": 7_200, "completion_tokens": 60,
            "prompt_tokens_details": {"cached_tokens": 7_000},
            "completion_tokens_details": {"reasoning_tokens": 20}
        },
    })
}

#[test]
fn a_question_about_the_attention_list_gets_a_screen_source_that_opens_the_dashboard() {
    // The lead's request of 06-11-2026 for frontend2: a REAL reply with a `screen` source. "aaj ke
    // kaam" is read before the first call (`app_state` `attention`); the model writes one sentence
    // that cites the count. The source is the screen the list is on: the dashboard (plan 55 11.1).
    let question = "aaj ke kaam";
    let find = |client: &RecordedClient| -> String {
        client.requests()[0]["messages"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["role"] == "tool")
            .filter_map(|m| serde_json::from_str::<serde_json::Value>(m["content"].as_str()?).ok())
            .find(|v| v["tool"] == "app_state")
            .and_then(|v| v["id"].as_str().map(str::to_owned))
            .expect("the attention list was read before the first call")
    };
    // Pass one: what the first call already holds.
    let h = Harness::day();
    h.runtime.set_read_first(true);
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with(question, None, &client);
    let id = find(&client);
    // Pass two: the model cites the count of the list.
    let h = Harness::day();
    h.runtime.set_read_first(true);
    let client = RecordedClient::new(vec![answer_reply(&format!(
        "آج کے کام: {{{{{id}.total}}}}"
    ))]);
    let reply = h.ask_with(question, None, &client);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(client.requests().len(), 1, "one model call");
    let answer = reply.answer.as_ref().unwrap();
    let screens: Vec<_> = answer
        .sources
        .iter()
        .filter_map(|s| match s {
            AnswerSource::Screen { result_id, route } => Some((result_id.as_str(), *route)),
            _ => None,
        })
        .collect();
    assert_eq!(
        screens,
        [(id.as_str(), mandi_assistant::answer::ScreenRoute::Dashboard)],
        "{answer:#?}"
    );
    fixture("assistant_reply_screen_source.json", &reply);
    fixture(
        "assistant_reply_screen_source_request.json",
        &request(question),
    );
    h.scan_requests();
}

#[test]
fn a_question_about_who_changed_a_bikri_gets_a_screen_source_that_opens_the_audit_log() {
    // The same for the audit log (the lead's request of 06-11-2026): the model asks `app_state`
    // `audit` for بکری 1 in its own call, and the source is the audit log tab of Settings. The
    // frontend copies this fixture next to the dashboard one.
    let question = "bikri 1 mein kisne badla";
    let ask_audit = serde_json::json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null,
            "tool_calls": [{"id": "call_0", "type": "function", "function": {
                "name": "app_state",
                "arguments": serde_json::json!(
                    {"what": "audit", "party": null, "item": null, "bikri": 1}).to_string()}}]}}],
        "usage": {
            "prompt_tokens": 7_200, "completion_tokens": 60,
            "prompt_tokens_details": {"cached_tokens": 7_000},
            "completion_tokens_details": {"reasoning_tokens": 0}
        },
    });
    let h = Harness::day();
    let client = RecordedClient::new(vec![
        ask_audit,
        answer_reply("اس بکری کے ریکارڈ میں {{r1.count}} اندراج ہیں"),
    ]);
    let reply = h.ask_with(question, None, &client);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let answer = reply.answer.as_ref().unwrap();
    let screens: Vec<_> = answer
        .sources
        .iter()
        .filter_map(|s| match s {
            AnswerSource::Screen { result_id, route } => Some((result_id.as_str(), *route)),
            _ => None,
        })
        .collect();
    assert_eq!(
        screens,
        [("r1", mandi_assistant::answer::ScreenRoute::Audit)],
        "{answer:#?}"
    );
    fixture("assistant_reply_screen_source_audit.json", &reply);
    fixture(
        "assistant_reply_screen_source_audit_request.json",
        &request(question),
    );
    h.scan_requests();
}

/// The tool entries of the log row of the last question, as the log keeps them.
fn logged(h: &Harness) -> Vec<serde_json::Value> {
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    serde_json::from_str(&rows[0].tool_calls_json).unwrap()
}

#[test]
fn a_local_answer_writes_what_it_used_into_the_question_log_marked_local() {
    // The lead's request of 05-11-2026: the Owner's log and the scorer must see what a local answer
    // used. Nothing was run, so each entry carries `"local": true` and `rerun` skips it.
    // A how-to: the guide topic.
    let (h, reply, _client) = ask("print kaise nikalte hain");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let entries = logged(&h);
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0]["tool"], "app_help");
    assert_eq!(entries[0]["local"], true);
    assert!(
        entries[0]["input"]["topic"]
            .as_str()
            .is_some_and(|t| !t.is_empty()),
        "{entries:?}"
    );
    // The log page names the topic as the source of the entry.
    let page = mandi_assistant::service::log_list(
        &h.world.env.ctx,
        &mandi_assistant::log::AssistantLogListInput {
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(
        page.rows[0].tools[0].source.as_deref(),
        entries[0]["input"]["topic"].as_str()
    );

    // A print: the report id and the filters of the button, only the ones set.
    let (h, reply, _client) = ask("akram ki aaj ki bikri ka print nikalna hai");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let entries = logged(&h);
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0]["tool"], "report");
    assert_eq!(entries[0]["local"], true);
    assert_eq!(entries[0]["input"]["id"], "R35");
    let filters = &entries[0]["input"];
    assert_eq!(filters["as_of"], "2026-10-10");
    assert_eq!(filters["party"]["record"]["name_en"], "Akram Toll Tax");
    assert!(
        filters.get("from").is_none() && filters.get("detail").is_none(),
        "only the filters that are set: {filters}"
    );

    // A month: two buttons, two entries, in the order of the buttons.
    let (h, _reply, _client) = ask("akram ki pichle mahine ki bikri ka print chahiye");
    let entries = logged(&h);
    let ids: Vec<&str> = entries
        .iter()
        .map(|e| e["input"]["id"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(ids, ["R5", "R13"], "{entries:?}");
    assert!(entries.iter().all(|e| e["local"] == true));

    // A name no record has: the find that came up empty.
    let (h, reply, _client) = ask("zubair ki aaj ki bikri ka print nikalna hai");
    assert_eq!(reply.outcome, AskOutcome::NotFound, "{reply:#?}");
    let entries = logged(&h);
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0]["tool"], "find");
    assert_eq!(entries[0]["local"], true);
    assert_eq!(entries[0]["input"]["text"], "zubair");
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
