//! The request body of one model call (Tech Stack 4.8, D6-system-prompt Section 5.3, Code
//! Standards Q10, Q11).
//!
//! Built here and nowhere else, so a test can scan the one place every byte sent to the model
//! comes from (E2E-54). The fixed part comes first (instructions, then tools in the `tools` field,
//! then the answer schema), so the provider can cache it. After it come the last 3 questions of the
//! conversation, with their earlier tool results cut down, and the question being answered.
//!
//! The provider block is a constant: only the Azure endpoint of the model, no fallback, zero data
//! retention, no data collection (Umar, 02-11-2026). Because it asks `require_parameters`, every
//! top-level key of the body must be one the Azure endpoints take (`BODY_KEYS_ALLOWED`). A key
//! they do not list makes OpenRouter drop every Azure endpoint and answer 404 (plan 51 Section 3).
//!
//! Example: a first question gives `messages` = the system message and one user message
//! `{"today":"2026-10-10","language":"ur","question":"ikram ke total bakaya kitne hai",
//! "question_numbers":{}}`.

use mandi_services::core::auth::Language;
use mandi_services::domain::figures::{NumberUnit, QuestionNumber};
use serde_json::{Map, Value, json};

use crate::conversation::{Conversation, Turn};
use crate::limits::CONVERSATION_QUESTIONS;
use crate::prompt::{SYSTEM, answer_schema, tools};
use crate::results::{ToolResult, Words, fit, tool_result_json};

/// The name of the answer schema in `response_format`.
pub const ANSWER_SCHEMA_NAME: &str = "answer";
/// How hard the model thinks (Tech Stack 4.8 [A]).
pub const REASONING_EFFORT: &str = "medium";
/// The fixed seed of every call.
pub const SEED: i64 = 20_261_102;
/// `tool_choice` while the model may call tools.
pub const TOOL_CHOICE_AUTO: &str = "auto";
/// `tool_choice` of the last call of a question and of the rewrite: no tool may be called.
pub const TOOL_CHOICE_NONE: &str = "none";
/// The marker of a tool message that holds the id of its result. Taken out before sending.
pub const RESULT_MARK: &str = "_result";

/// The top-level keys the body may hold (plan 51 Section 3, read from OpenRouter's endpoint list of
/// `openai/gpt-6-luna` on 02-11-2026). First the parameters its Azure endpoints list, then the keys
/// OpenRouter always takes: `model`, `messages` and `provider` (the routing block).
///
/// `stream` is not on the list and is never sent. Its default is false, which is what D6 Section 5
/// wants (an answer is shown only when whole and checked, FR-7.76), so leaving it out changes
/// nothing and cannot cost an Azure endpoint.
///
/// With `require_parameters: true`, a key outside this list drops every Azure endpoint, and the
/// call fails with HTTP 404. A new key is added here only after the Azure endpoints are checked.
pub const BODY_KEYS_ALLOWED: [&str; 13] = [
    "include_reasoning",
    "max_completion_tokens",
    "reasoning",
    "reasoning_effort",
    "response_format",
    "seed",
    "structured_outputs",
    "tool_choice",
    "tools",
    "verbosity",
    "model",
    "messages",
    "provider",
];

/// The provider rules of every call.
pub fn provider() -> Value {
    json!({
        "only": ["azure"],
        "allow_fallbacks": false,
        "zdr": true,
        "data_collection": "deny",
        "require_parameters": true,
    })
}

fn unit_word(unit: NumberUnit) -> &'static str {
    match unit {
        NumberUnit::Months => "months",
        NumberUnit::Days => "days",
        NumberUnit::Years => "years",
        NumberUnit::Percent => "percent",
        NumberUnit::Amount => "amount",
        NumberUnit::Plain => "plain",
    }
}

/// The user message of a question (D6-system-prompt 5.3).
pub fn user_message(
    today: &str,
    language: Language,
    question: &str,
    numbers: &[QuestionNumber],
) -> Value {
    let mut map = Map::new();
    for (i, n) in numbers.iter().enumerate() {
        map.insert(
            format!("n{}", i.saturating_add(1)),
            json!({"number": n.number, "text": n.text, "unit": unit_word(n.unit)}),
        );
    }
    let content = json!({
        "today": today,
        "language": match language { Language::Ur => "ur", Language::En => "en" },
        "question": question,
        "question_numbers": Value::Object(map),
    });
    json!({"role": "user", "content": content.to_string()})
}

/// An earlier result cut down: its id, source, filters, party, count, totals, summary and matched,
/// plus only the rows its answer referred to (D6-system-prompt 5.3).
fn cut_down(full: &Value, referenced: &[(String, String)], id: &str) -> Value {
    let Value::Object(fields) = full else {
        return full.clone();
    };
    if !matches!(
        fields.get("tool").and_then(Value::as_str),
        Some("report" | "analytics")
    ) {
        return full.clone();
    }
    let keep = [
        "id",
        "tool",
        "source",
        "title",
        "filters",
        "party",
        "count",
        "totals",
        "summary",
        "matched",
        "compare",
        "partner_parts",
    ];
    let mut out = Map::new();
    for key in keep {
        if let Some(v) = fields.get(key) {
            out.insert(key.to_owned(), v.clone());
        }
    }
    if let Some(Value::Array(rows)) = fields.get("rows") {
        let kept: Vec<Value> = rows
            .iter()
            .filter(|row| {
                row.get("key")
                    .and_then(Value::as_str)
                    .is_some_and(|k| referenced.iter().any(|(rid, rk)| rid == id && rk == k))
            })
            .cloned()
            .collect();
        out.insert("rows".to_owned(), Value::Array(kept));
    }
    Value::Object(out)
}

fn result_content(
    conversation: &Conversation,
    words: &Words,
    id: &str,
    current: bool,
    locked: bool,
    referenced: &[(String, String)],
) -> String {
    let Some(result) = conversation.results.iter().find(|r| r.id == id) else {
        return json!({"id": id, "dropped": "not kept"}).to_string();
    };
    // Once the section is locked again, an earlier partner result is not sent (E2E-54).
    if !current && locked && result.partner_data {
        return json!({"id": id, "tool": result.tool, "dropped": "partner section locked"})
            .to_string();
    }
    let mut result: ToolResult = result.clone();
    fit(&mut result, words);
    let full = tool_result_json(&result, words);
    if current {
        full.to_string()
    } else {
        cut_down(&full, referenced, id).to_string()
    }
}

fn message_for_send(
    message: &Value,
    conversation: &Conversation,
    words: &Words,
    current: bool,
    locked: bool,
    referenced: &[(String, String)],
) -> Value {
    let Value::Object(fields) = message else {
        return message.clone();
    };
    let mut out = fields.clone();
    if let Some(Value::String(id)) = out.remove(RESULT_MARK) {
        out.insert(
            "content".to_owned(),
            Value::String(result_content(
                conversation,
                words,
                &id,
                current,
                locked,
                referenced,
            )),
        );
    }
    Value::Object(out)
}

/// Everything that builds one request.
pub struct Parts<'a> {
    /// The model.
    pub model: &'a str,
    /// The conversation: its finished questions (the last 3 are sent) and every result so far.
    pub conversation: &'a Conversation,
    /// The question being answered.
    pub current: &'a Turn,
    /// The words of the model's JSON.
    pub words: &'a Words,
    /// True while the partner section is locked.
    pub partner_locked: bool,
    /// Output tokens this call may use. Sent as `max_completion_tokens`, the name the Azure
    /// endpoints take (`max_tokens` is not on their list).
    pub max_completion_tokens: u32,
    /// True for the last call a question may make and for the rewrite: `tool_choice` is `none`,
    /// so the model must answer from what it has read (plan 52 item 1, PRD E96). The `tools`
    /// list is still sent, so the fixed part stays the same and stays cached.
    pub tools_off: bool,
}

/// The body of one call.
pub fn request_body(parts: &Parts<'_>) -> Value {
    let earlier = &parts.conversation.turns;
    let first_earlier = earlier.len().saturating_sub(CONVERSATION_QUESTIONS);
    let mut messages = vec![json!({"role": "system", "content": SYSTEM})];
    let sent = earlier
        .iter()
        .skip(first_earlier)
        .map(|turn| (turn, false))
        .chain(std::iter::once((parts.current, true)));
    for (turn, current) in sent {
        for message in &turn.messages {
            messages.push(message_for_send(
                message,
                parts.conversation,
                parts.words,
                current,
                parts.partner_locked,
                &turn.referenced,
            ));
        }
    }
    json!({
        "model": parts.model,
        "messages": messages,
        "tools": tools(),
        "tool_choice": if parts.tools_off { TOOL_CHOICE_NONE } else { TOOL_CHOICE_AUTO },
        "response_format": {
            "type": "json_schema",
            "json_schema": {"name": ANSWER_SCHEMA_NAME, "strict": true, "schema": answer_schema()},
        },
        "reasoning": {"effort": REASONING_EFFORT},
        "max_completion_tokens": parts.max_completion_tokens,
        "seed": SEED,
        "provider": provider(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(max_completion_tokens: u32) -> Value {
        body_with(max_completion_tokens, false)
    }

    fn body_with(max_completion_tokens: u32, tools_off: bool) -> Value {
        let turn = Turn {
            question: "ikram ka bakaya".to_owned(),
            messages: vec![user_message(
                "2026-10-10",
                Language::Ur,
                "ikram ka bakaya",
                &[],
            )],
            ..Turn::default()
        };
        let conversation = Conversation::new("c1");
        let words = Words::new().expect("the Fluent files");
        request_body(&Parts {
            model: "openai/gpt-6-luna",
            conversation: &conversation,
            current: &turn,
            words: &words,
            partner_locked: true,
            max_completion_tokens,
            tools_off,
        })
    }

    /// Plan 51 Section 3: with `require_parameters: true`, a top-level key that the Azure endpoints
    /// do not list drops every Azure endpoint and the call answers 404.
    #[test]
    fn every_top_level_key_is_one_the_azure_endpoints_take() {
        let body = body(1_500);
        let Value::Object(fields) = &body else {
            panic!("the body is an object");
        };
        for key in fields.keys() {
            assert!(
                BODY_KEYS_ALLOWED.contains(&key.as_str()),
                "the key {key:?} is not on the Azure list"
            );
        }
        // The two keys that caused the 404 are gone.
        assert!(!fields.contains_key("max_tokens"));
        assert!(!fields.contains_key("parallel_tool_calls"));
        // `stream` is not sent either: the default is false, and Azure does not list it.
        assert!(!fields.contains_key("stream"));
    }

    #[test]
    fn the_output_budget_is_sent_as_max_completion_tokens() {
        assert_eq!(body(1_500)["max_completion_tokens"], 1_500);
        assert_eq!(body(1)["max_completion_tokens"], 1);
    }

    #[test]
    fn the_provider_block_is_azure_only_with_zero_retention() {
        let body = body(1_500);
        assert_eq!(body["provider"], provider());
        assert_eq!(
            body["provider"],
            json!({
                "only": ["azure"],
                "allow_fallbacks": false,
                "zdr": true,
                "data_collection": "deny",
                "require_parameters": true,
            })
        );
        assert_eq!(body["tool_choice"], "auto");
        assert_eq!(body["seed"], SEED);
        assert_eq!(body["reasoning"], json!({"effort": REASONING_EFFORT}));
    }

    /// Plan 52 item 1: the last call has the tools switched off; the tool list stays in the body,
    /// so the fixed part is the same and stays cached.
    #[test]
    fn the_last_call_goes_out_with_tool_choice_none() {
        let on = body_with(1_500, false);
        let off = body_with(1_500, true);
        assert_eq!(on["tool_choice"], "auto");
        assert_eq!(off["tool_choice"], "none");
        assert_eq!(on["tools"], off["tools"]);
        assert_eq!(on["messages"][0], off["messages"][0]);
        assert_eq!(on["response_format"], off["response_format"]);
    }

    #[test]
    fn the_allowed_list_has_no_repeat() {
        let mut keys = BODY_KEYS_ALLOWED.to_vec();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), BODY_KEYS_ALLOWED.len());
    }
}
