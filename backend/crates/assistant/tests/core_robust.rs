//! The robust core of plan 52 (items 1, 4 and 6): the last call answers with the tools switched
//! off, a call that failed on the network is tried once more, and help is never a change request.
//! The model is a recorded transcript; nothing calls the network.
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
use mandi_assistant::answer::{AnswerPart, EntryScreen};
use mandi_assistant::ask::{AskOutcome, FailureKind};
use mandi_assistant::client::{CallError, RecordedClient};
use mandi_services::assistant;
use serde_json::{Value, json};

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_800, "completion_tokens": 90,
        "prompt_tokens_details": {"cached_tokens": 7_600},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

fn tool_call(name: &str, args: &Value, id: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null,
            "tool_calls": [{"id": id, "type": "function",
                "function": {"name": name, "arguments": args.to_string()}}]}}],
        "usage": usage(),
    })
}

fn final_reply(kind: &str, text: &str, entry: &Value) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": kind, "text": text, "show": [], "next": [],
            "choose": null, "entry": entry, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

fn good_commission() -> Value {
    final_reply(
        "answer",
        "آج کا کمیشن {{r1.today_commission.amount}} بنا۔",
        &Value::Null,
    )
}

fn last_row(h: &Harness) -> mandi_services::db::assistant::QuestionRow {
    let (rows, _, _) = assistant::list_questions(&h.world.env.ctx, 0, 100).unwrap();
    rows.into_iter().max_by_key(|r| r.id).unwrap()
}

// ---- item 6: one retry ------------------------------------------------------------------------

#[test]
fn a_call_that_failed_on_the_network_is_tried_once_more_and_the_question_goes_on() {
    for error in [
        CallError::new(FailureKind::Offline),
        CallError::new(FailureKind::Dropped),
        CallError {
            kind: FailureKind::Busy,
            http_status: Some(429),
            reason: None,
        },
        CallError {
            kind: FailureKind::Server,
            http_status: Some(503),
            reason: None,
        },
    ] {
        let h = Harness::day();
        let client = RecordedClient::with_results(vec![
            Err(error.clone()),
            Ok(tool_call("tiles", &json!({}), "call_0")),
            Ok(good_commission()),
        ]);
        let reply = h.ask_with("aaj ka commission", None, &client);
        assert_eq!(reply.outcome, AskOutcome::Answered, "{error:?}: {reply:#?}");
        let requests = client.requests();
        assert_eq!(requests.len(), 3, "{error:?}: the same call, sent again");
        assert_eq!(requests[0], requests[1], "{error:?}");
        // The failed try is in the timing, with its kind; only replies that came are counted.
        let row = last_row(&h);
        assert_eq!(row.model_calls, Some(2), "{error:?}");
        let calls: Vec<Value> = serde_json::from_str(&row.calls_json.unwrap()).unwrap();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0]["failed"], true);
        assert!(calls[0]["error"].is_string(), "{error:?}");
        assert!(calls[1].get("error").is_none());
        assert_eq!(reply.count_today, 1, "one question, counted once");
        h.scan_requests();
    }
}

#[test]
fn only_one_retry_and_an_offline_ending_when_the_internet_is_really_gone() {
    let h = Harness::day();
    let client = RecordedClient::with_results(vec![
        Err(CallError::new(FailureKind::Offline)),
        Err(CallError::new(FailureKind::Offline)),
        Ok(good_commission()),
    ]);
    let reply = h.ask_with("aaj ka commission", None, &client);
    assert_eq!(reply.outcome, AskOutcome::Offline);
    assert_eq!(reply.failure.map(|f| f.kind), Some(FailureKind::Offline));
    assert_eq!(client.requests().len(), 2, "never a second retry");
    assert_eq!(reply.count_today, 0, "no reply came, nothing counted");
    // A connection that was made and dropped is not called "no internet".
    let client = RecordedClient::with_results(vec![
        Err(CallError::new(FailureKind::Dropped)),
        Err(CallError::new(FailureKind::Dropped)),
    ]);
    let reply = h.ask_with("aaj ka commission", None, &client);
    assert_eq!(reply.outcome, AskOutcome::Error);
    assert_eq!(reply.failure.map(|f| f.kind), Some(FailureKind::Dropped));
    assert_eq!(client.requests().len(), 2);
}

#[test]
fn what_one_more_try_cannot_cure_is_not_tried_again() {
    for error in [
        CallError {
            kind: FailureKind::BadKey,
            http_status: Some(401),
            reason: None,
        },
        CallError {
            kind: FailureKind::NoCredit,
            http_status: Some(402),
            reason: None,
        },
        CallError {
            kind: FailureKind::NoEndpoint,
            http_status: Some(404),
            reason: None,
        },
        CallError::new(FailureKind::BadReply),
        CallError {
            kind: FailureKind::Other,
            http_status: Some(400),
            reason: None,
        },
    ] {
        let h = Harness::day();
        let client = RecordedClient::with_results(vec![Err(error.clone()), Ok(good_commission())]);
        let reply = h.ask_with("aaj ka commission", None, &client);
        assert_eq!(reply.outcome, AskOutcome::Error, "{error:?}");
        assert_eq!(client.requests().len(), 1, "{error:?}");
    }
}

// ---- item 1: the last call answers ------------------------------------------------------------

#[test]
fn the_rewrite_goes_out_with_the_tools_switched_off_and_the_log_says_why_it_was_needed() {
    let h = Harness::day();
    // The first answer has a typed figure; the rewrite is a good one.
    let typed = final_reply("answer", "آج کا کمیشن 5,448 بنا۔", &Value::Null);
    let (reply, requests) = h.ask(
        "aaj ka commission",
        None,
        vec![
            tool_call("tiles", &json!({}), "call_0"),
            typed,
            good_commission(),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0]["tool_choice"], "auto");
    assert_eq!(requests[1]["tool_choice"], "auto");
    assert_eq!(
        requests[2]["tool_choice"], "none",
        "the rewrite reads nothing"
    );
    // The tool list is still sent, so the fixed part is the same and stays cached.
    assert_eq!(requests[1]["tools"], requests[2]["tools"]);
    // The log keeps the rule that failed, no text and no figure.
    let calls: Vec<Value> = serde_json::from_str(&last_row(&h).calls_json.unwrap()).unwrap();
    assert_eq!(calls[2]["rewrite"], true);
    let rules = calls[2]["check_failed"]
        .as_array()
        .expect("the failed rules");
    assert!(!rules.is_empty());
    assert!(rules.iter().all(|r| r.is_string()));
    assert!(!calls[2].to_string().contains("5,448"));
}

#[test]
fn a_question_that_reads_until_the_last_call_still_gets_its_answer() {
    let h = Harness::day();
    // Five calls read a tool each. The sixth has the tools off, and the model answers.
    let mut replies: Vec<Value> = (0..5)
        .map(|n| tool_call("tiles", &json!({}), &format!("call_{n}")))
        .collect();
    replies.push(good_commission());
    let (reply, requests) = h.ask("aaj ka commission", None, replies);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 6);
    assert_eq!(requests[4]["tool_choice"], "auto");
    assert_eq!(requests[5]["tool_choice"], "none");
}

// ---- item 4: help is never a change request ----------------------------------------------------

/// The model reads the guide and then, wrongly, ends the question as a change request (the four
/// misses of the live run of 02-11-2026).
#[test]
fn a_how_to_question_ended_as_a_change_request_is_answered_with_the_guide() {
    for (question, topic, screen, want) in [
        (
            "sale entry kese kartay hain",
            "bikri_book",
            "bikri",
            EntryScreen::Bikri,
        ),
        (
            "بکری کیسے شامل کروں؟",
            "bikri_book",
            "bikri",
            EntryScreen::Bikri,
        ),
        (
            "how do I add a sale?",
            "bikri_book",
            "bikri",
            EntryScreen::Bikri,
        ),
        (
            "where do I see a partner's account?",
            "partners",
            "partners",
            EntryScreen::Partners,
        ),
    ] {
        let h = Harness::day();
        let (reply, requests) = h.ask(
            question,
            None,
            vec![
                tool_call("app_help", &json!({"topic": topic}), "call_0"),
                final_reply(
                    "change_request",
                    "",
                    &json!({"screen": screen, "party": null}),
                ),
            ],
        );
        assert_eq!(
            reply.outcome,
            AskOutcome::Answered,
            "{question}: {reply:#?}"
        );
        assert_eq!(requests.len(), 2, "{question}: no extra call");
        // The words are the guide's own: its summary, then its steps, each as one part.
        let answer = answer_of(&reply);
        assert!(
            answer
                .parts
                .iter()
                .any(|p| matches!(p, AnswerPart::Text { .. })),
            "{question}"
        );
        assert!(answer.parts.len() >= 3, "{question}: {:?}", answer.parts);
        // The button that opens the screen stays, and takes no party.
        let route = reply.entry_route.expect("the route");
        assert_eq!(route.screen, want, "{question}");
        assert!(route.party.is_none());
        // The log keeps what the model wrote and the guide it read.
        let row = last_row(&h);
        assert_eq!(row.outcome, "answered");
        assert!(row.tool_calls_json.contains("app_help"));
        h.scan_requests();
    }
}

#[test]
fn a_help_question_the_model_ended_without_reading_the_guide_gets_the_guide_of_the_screen() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "bikri kaise add karun",
        None,
        vec![final_reply(
            "change_request",
            "",
            &json!({"screen": "bikri", "party": null}),
        )],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 1, "the app read the guide itself");
    assert!(answer_of(&reply).parts.len() >= 3);
    assert_eq!(reply.entry_route.expect("route").screen, EntryScreen::Bikri);
    let row = last_row(&h);
    assert!(
        row.tool_calls_json.contains("app_help"),
        "the read is in the log"
    );
    assert!(row.tool_calls_json.contains("bikri_book"));
    // The model did not call it: the question has no tool call of the model's.
    assert_eq!(row.model_calls, Some(1));
}

#[test]
fn a_real_change_request_is_still_refused() {
    // It names a number to write: not a how-to question.
    let h = Harness::day();
    let find = json!({"text": "akram", "role": "buyer"});
    let (reply, _) = h.ask(
        "akram ki 5000 wasooli likh do",
        None,
        vec![
            tool_call("find", &find, "call_0"),
            final_reply(
                "change_request",
                "",
                &json!({"screen": "wasooli", "party": "r1.matches.0"}),
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::ChangeRefused, "{reply:#?}");
    assert_eq!(
        reply.entry_route.expect("route").screen,
        EntryScreen::Wasooli
    );
    // No how, where or what word: also still refused, the guard needs evidence of help.
    let h = Harness::day();
    let (reply, _) = h.ask(
        "akram ka rate barha do",
        None,
        vec![
            tool_call("find", &find, "call_0"),
            final_reply(
                "change_request",
                "",
                &json!({"screen": "account", "party": "r1.matches.0"}),
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::ChangeRefused, "{reply:#?}");
}

#[test]
fn a_how_to_question_with_a_number_is_a_change_request() {
    let h = Harness::day();
    let find = json!({"text": "akram", "role": "buyer"});
    let (reply, _) = h.ask(
        "akram ki 5000 wasooli kaise likhun",
        None,
        vec![
            tool_call("find", &find, "call_0"),
            final_reply(
                "change_request",
                "",
                &json!({"screen": "wasooli", "party": "r1.matches.0"}),
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::ChangeRefused, "{reply:#?}");
}

// ---- item 7: help in one call -------------------------------------------------------------------

#[test]
fn a_how_to_question_about_one_screen_gets_the_guide_before_the_first_call_and_one_call_answers() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "bikri kaise add karun",
        None,
        vec![final_reply(
            "answer",
            "{{h1.summary}}\n• {{h1.steps.0}}\n• {{h1.steps.1}}",
            &json!({"screen": "bikri", "party": null}),
        )],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 1, "help takes one model call");
    // The guide is in that one request, as a tool result with its own id `h1`.
    let messages = requests[0]["messages"].as_array().unwrap();
    let tool = messages
        .iter()
        .find(|m| m["role"] == "tool")
        .expect("the guide as a tool result");
    let content = tool["content"].as_str().unwrap();
    assert!(
        content.contains("\"id\":\"h1\"") && content.contains("bikri_book"),
        "{content}"
    );
    // The answer is the guide's own words and carries the button of the screen.
    assert!(answer_of(&reply).parts.len() >= 3);
    assert_eq!(reply.entry_route.expect("route").screen, EntryScreen::Bikri);
    // The model's own tool calls were not used up, and its ids did not move: the guide is `h1`.
    let (rows, _, _) = assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    assert_eq!(rows[0].model_calls, Some(1));
    // A question that names no topic, or two, is not guessed: the model asks for the topic.
    let (_, requests) = h.ask(
        "ye kaise karun",
        None,
        vec![final_reply("not_understood", "", &Value::Null)],
    );
    assert!(!requests[0].to_string().contains("pre_help"));
    h.scan_requests();
}
