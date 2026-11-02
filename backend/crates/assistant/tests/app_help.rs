//! The tool `app_help` (plan 51 P51-B3, PRD FR-7.89): how-to and "what is this" questions are
//! answered from the app's written guide. The model asks for a topic and cites its texts by
//! reference; the app puts the guide's own words into the answer, so a key or a step is never
//! retyped by the model. The model is a recorded transcript; nothing calls the network.
//!
//! The four questions of Umar's list, each in Roman Urdu, in Urdu letters and in English:
//! "market safar kya hai" (market_trips), "bikri kaise add karun" (bikri_book), "wasooli kaise
//! likhun" (receipt) and "partner ka hisaab kaise dekhun" (partners).
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

use harness::Harness;
use mandi_assistant::answer::AnswerPart;
use mandi_assistant::ask::AskOutcome;
use mandi_assistant::prompt::{GuideText, guide};
use mandi_services::core::auth::{Language, PrefKey};
use serde_json::{Value, json};

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_800, "completion_tokens": 90,
        "prompt_tokens_details": {"cached_tokens": 7_600},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

fn help_call(topic: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null,
            "tool_calls": [{"id": "call_0", "type": "function",
                "function": {"name": "app_help", "arguments": json!({"topic": topic}).to_string()}}]}}],
        "usage": usage(),
    })
}

fn answer_of(text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

fn english(h: &Harness) {
    mandi_services::settings::set_pref(&h.world.env.ctx, &h.world.owner, PrefKey::Language, "en")
        .unwrap();
}

/// The text the model writes: the summary and each step by reference.
fn help_text(steps: usize) -> String {
    let mut text = "{{r1.summary}}".to_owned();
    for i in 0..steps {
        text.push_str(&format!("\n• {{{{r1.steps.{i}}}}}"));
    }
    text
}

fn texts(parts: &[AnswerPart]) -> Vec<&str> {
    parts
        .iter()
        .filter_map(|p| match p {
            AnswerPart::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

fn the(language: Language, topic: &str) -> &'static GuideText {
    let t = guide().topics.get(topic).expect("a topic of the guide");
    match language {
        Language::Ur => &t.ur,
        Language::En => &t.en,
    }
}

/// (question, language of the app, topic).
const QUESTIONS: [(&str, Language, &str); 12] = [
    (
        "is main market safar kiya hai iski mujhay samaj nahi aa rahi",
        Language::Ur,
        "market_trips",
    ),
    ("مارکیٹ سفر کیا ہوتا ہے؟", Language::Ur, "market_trips"),
    ("what is a market trip?", Language::En, "market_trips"),
    ("bikri kaise add karun", Language::Ur, "bikri_book"),
    ("بکری کیسے شامل کروں؟", Language::Ur, "bikri_book"),
    ("how do I add a sale?", Language::En, "bikri_book"),
    ("wasooli kaise likhun", Language::Ur, "receipt"),
    ("وصولی کیسے لکھوں؟", Language::Ur, "receipt"),
    ("how do I write a receipt?", Language::En, "receipt"),
    ("partner ka hisaab kaise dekhun", Language::Ur, "partners"),
    ("شراکت دار کا حساب کیسے دیکھوں؟", Language::Ur, "partners"),
    (
        "where do I see a partner's account?",
        Language::En,
        "partners",
    ),
];

#[test]
fn a_how_to_question_is_answered_from_the_guide_in_two_calls_in_every_wording() {
    for (question, language, topic) in QUESTIONS {
        let h = Harness::day();
        if language == Language::En {
            english(&h);
        }
        let g = the(language, topic);
        let steps = g.steps.len().min(4);
        let (reply, requests) = h.ask(
            question,
            None,
            vec![help_call(topic), answer_of(&help_text(steps))],
        );
        assert_eq!(
            reply.outcome,
            AskOutcome::Answered,
            "{question}: {reply:#?}"
        );
        assert_eq!(
            requests.len(),
            2,
            "{question}: one call for the topic, one for the answer"
        );
        assert!(
            reply.entry_route.is_none(),
            "{question}: a how-to is not a change request"
        );
        assert_eq!(reply.stop_reason, None);
        let answer = reply.answer.as_ref().expect("an answer");
        // The words of the answer are the guide's own, in the language of the app, in order.
        let got = texts(&answer.parts);
        assert_eq!(got[0], g.summary, "{question}: the summary");
        for (i, step) in g.steps.iter().take(steps).enumerate() {
            assert!(
                got.iter().any(|t| t == step),
                "{question}: step {i} is in the answer"
            );
        }
        // No figure of the books is in a help answer, and it has no table or source button.
        assert!(
            answer
                .parts
                .iter()
                .all(|p| matches!(p, AnswerPart::Text { .. })),
            "{question}: only the guide's text and the model's own words"
        );
        assert!(answer.tables.is_empty() && answer.sources.is_empty());
        h.scan_requests();
    }
}

#[test]
fn the_guide_of_a_topic_reaches_the_model_in_the_app_language() {
    for language in [Language::Ur, Language::En] {
        let h = Harness::day();
        if language == Language::En {
            english(&h);
        }
        let (_, requests) = h.ask(
            "bikri kaise add karun",
            None,
            vec![help_call("bikri_book"), answer_of(&help_text(2))],
        );
        let tool_message = requests[1]["messages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["role"] == "tool")
            .unwrap()["content"]
            .as_str()
            .unwrap()
            .to_owned();
        let result: Value = serde_json::from_str(&tool_message).unwrap();
        let g = the(language, "bikri_book");
        assert_eq!(result["tool"], "app_help");
        assert_eq!(result["topic"], "bikri_book");
        assert_eq!(result["title"], g.title);
        assert_eq!(result["summary"], g.summary);
        assert_eq!(result["steps"].as_array().unwrap().len(), g.steps.len());
        assert_eq!(result["keys"], g.keys);
        // The other language is not sent.
        let other = the(
            if language == Language::Ur {
                Language::En
            } else {
                Language::Ur
            },
            "bikri_book",
        );
        assert!(!tool_message.contains(&other.summary));
    }
}

#[test]
fn a_topic_the_guide_does_not_have_is_refused_with_the_list_and_the_retry_works() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "bikri kaise add karun",
        None,
        vec![
            help_call("sale_entry"),
            help_call("bikri_book"),
            answer_of(&help_text(2)),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let first_refusal = requests[1]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "tool")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    let refusal: Value = serde_json::from_str(first_refusal).unwrap();
    assert_eq!(refusal["error"]["field"], "topic");
    let allowed: Vec<&str> = refusal["error"]["allowed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect();
    assert!(
        allowed.contains(&"bikri_book") && allowed.contains(&"market_trips"),
        "{allowed:?}"
    );
}

#[test]
fn help_never_waits_for_the_partner_code_and_reads_no_books() {
    // The partner section is locked on the worked day; the guide for partners is the app's own
    // text and holds no figure, so the question does not pause.
    let h = Harness::day();
    let (reply, _) = h.ask(
        "partner ka hisaab kaise dekhun",
        None,
        vec![help_call("partners"), answer_of(&help_text(3))],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert!(!reply.locked);
    // The log row names the tool, with no source.
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 100).unwrap();
    let row = rows.into_iter().max_by_key(|r| r.id).unwrap();
    assert!(row.tool_calls_json.contains("app_help"));
}

#[test]
fn a_help_answer_follows_the_word_limit_and_is_short_when_it_is_short() {
    // Five points of the guide and the summary: six references, six words for the limit, however
    // long each text is (a reference counts as one word, the rule of the check).
    let h = Harness::day();
    let text = "{{r1.summary}}\n• {{r1.steps.0}}\n• {{r1.steps.1}}\n• {{r1.steps.2}}\n• {{r1.steps.3}}\n• {{r1.keys}}";
    let (reply, _) = h.ask(
        "bikri kaise add karun",
        None,
        vec![help_call("bikri_book"), answer_of(text)],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    // The model's own words are limited to 130 words in all: more than that is refused, and the
    // one rewrite is asked for.
    let long = format!("{} {{{{r1.summary}}}}", "بکری ".repeat(140));
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "bikri kaise add karun",
        None,
        vec![
            help_call("bikri_book"),
            answer_of(&long),
            answer_of(&help_text(2)),
        ],
    );
    assert_eq!(
        requests.len(),
        3,
        "the long answer was sent back once: {reply:#?}"
    );
    let check = requests[2]["messages"].as_array().unwrap().last().unwrap()["content"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(check.contains("too_long"), "{check}");
    assert_eq!(reply.outcome, AskOutcome::Answered);
}
