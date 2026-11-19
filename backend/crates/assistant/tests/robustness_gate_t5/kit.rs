//! Helpers shared by the T5 modules: recorded replies, readers of a request body, the read-only
//! check of attack 10, and a `Reader` that runs the tools directly (the same functions the loop
//! calls) and keeps every JSON text the model would be sent.
#![allow(dead_code)]

use std::collections::BTreeSet;

use mandi_assistant::ask::AskReply;
use mandi_assistant::results::{Words, fit, tool_result_json};
use mandi_assistant::tools::{self, Env, State, Stop};
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use serde_json::{Value, json};
use tempfile::TempDir;

use crate::common;
use crate::harness::Harness;
use crate::world;

// ---- recorded replies ------------------------------------------------------------------------

/// The first find of almost every transcript: the result `r1` has Akram as its first match.
pub const AKRAM: &str = "r1.matches.0";

pub fn usage() -> Value {
    json!({
        "prompt_tokens": 5200,
        "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 0},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

pub fn tool_reply(calls: &[(&str, Value)]) -> Value {
    let raw: Vec<(&str, String)> = calls.iter().map(|(n, a)| (*n, a.to_string())).collect();
    raw_tool_reply(&raw)
}

pub fn raw_tool_reply(calls: &[(&str, String)]) -> Value {
    let calls: Vec<Value> = calls
        .iter()
        .enumerate()
        .map(|(i, (name, arguments))| {
            json!({"id": format!("call_{i}"), "type": "function",
                   "function": {"name": name, "arguments": arguments}})
        })
        .collect();
    json!({"model": "openai/gpt-6-luna",
           "choices": [{"finish_reason": "tool_calls",
                        "message": {"role": "assistant", "content": null, "tool_calls": calls}}],
           "usage": usage()})
}

pub fn text_reply(finish_reason: &str, content: &str) -> Value {
    json!({"model": "openai/gpt-6-luna",
           "choices": [{"finish_reason": finish_reason,
                        "message": {"role": "assistant", "content": content}}],
           "usage": usage()})
}

pub fn final_reply(content: &str) -> Value {
    text_reply("stop", content)
}

pub fn answer(kind: &str, text: &str) -> String {
    answer_full(kind, text, json!([]), json!([]), Value::Null, Value::Null)
}

pub fn answer_full(
    kind: &str,
    text: &str,
    show: Value,
    next: Value,
    choose: Value,
    entry: Value,
) -> String {
    json!({"kind": kind, "text": text, "show": show, "next": next,
           "choose": choose, "entry": entry, "nearest": null})
    .to_string()
}

pub fn answer_nearest(kind: &str, text: &str, nearest: Value) -> String {
    json!({"kind": kind, "text": text, "show": [], "next": [],
           "choose": null, "entry": null, "nearest": nearest})
    .to_string()
}

pub fn find(text: &str, role: &str) -> (&'static str, Value) {
    ("find", json!({"text": text, "role": role}))
}

pub fn card(party: &str) -> (&'static str, Value) {
    ("party_card", json!({"party": party}))
}

pub fn report_args(id: &str, extra: Value) -> Value {
    let mut base = json!({"id": id, "from": null, "to": null, "as_of": null, "party": null,
        "item": null, "item_group": null, "city": null, "adda": null, "collector": null,
        "partner": null, "trip": null, "garden": null, "bikri": null, "seller": null,
        "zamindar": null, "status": null, "voucher_type": null, "layout": null,
        "group_by": null, "where": null, "sort": null, "rows": null});
    merge(&mut base, extra);
    base
}

pub fn analytics_args(view: &str, extra: Value) -> Value {
    let mut base = json!({"view": view, "from": null, "to": null, "group_by": null, "item": null,
        "item_group": null, "grade": null, "seller": null, "buyer": null, "city": null,
        "adda": null, "garden": null, "trip": null, "goods": null, "partner": null,
        "side": null, "compare": null, "sort": null, "rows": null});
    merge(&mut base, extra);
    base
}

pub fn merge(base: &mut Value, extra: Value) {
    if let (Some(b), Value::Object(e)) = (base.as_object_mut(), extra) {
        for (k, v) in e {
            b.insert(k, v);
        }
    }
}

pub fn estimate_args(measure: &str, ahead: Option<i64>, extra: Value) -> Value {
    let mut base = json!({"measure": measure, "item": null, "seller": null, "buyer": null,
        "garden": null, "trip": null, "ahead": ahead, "target": null, "until": null,
        "what_if": null, "percent": null, "head": null, "partner": null, "party": null});
    merge(&mut base, extra);
    base
}

pub fn period_args(kind: &str, extra: Value) -> Value {
    let mut base = json!({"kind": kind, "month": null, "day": null, "from": null, "to": null,
        "n": null});
    merge(&mut base, extra);
    base
}

/// The transcript of "find Akram, read his card", then the given last replies.
pub fn akram_then(last: Vec<Value>) -> Vec<Value> {
    let mut replies = vec![
        tool_reply(&[find("ikram", "buyer")]),
        tool_reply(&[card(AKRAM)]),
    ];
    replies.extend(last);
    replies
}

pub fn good_akram() -> String {
    answer(
        "answer",
        "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔",
    )
}

pub fn gives_up() -> Vec<Value> {
    vec![final_reply(&answer("not_understood", ""))]
}

// ---- reading what was sent ----------------------------------------------------------------

pub fn messages(body: &Value) -> &Vec<Value> {
    body["messages"].as_array().expect("messages")
}

pub fn user_of(body: &Value) -> Value {
    let m = messages(body)
        .iter()
        .find(|m| m["role"] == "user")
        .expect("a user message");
    serde_json::from_str(m["content"].as_str().expect("text")).expect("the user message is JSON")
}

/// The error codes of the tool messages of a request, in order (`None` for a tool that worked).
pub fn tool_errors(body: &Value) -> Vec<Option<String>> {
    messages(body)
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter(|m| {
            !m["tool_call_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("pre_p"))
        })
        .map(|m| {
            let v: Value =
                serde_json::from_str(m["content"].as_str().unwrap_or("{}")).unwrap_or(Value::Null);
            v["error"]["code"].as_str().map(str::to_owned)
        })
        .collect()
}

pub fn failed_rules(body: &Value) -> BTreeSet<String> {
    let mut rules = BTreeSet::new();
    for m in messages(body).iter().filter(|m| m["role"] == "user") {
        let Some(v) = m["content"]
            .as_str()
            .and_then(|t| serde_json::from_str::<Value>(t).ok())
        else {
            continue;
        };
        for f in v["check_failed"].as_array().into_iter().flatten() {
            if let Some(r) = f["rule"].as_str() {
                rules.insert(r.to_owned());
            }
        }
    }
    rules
}

pub fn all_text(requests: &[Value]) -> String {
    requests
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

/// The text of the messages of a request that are not tool results (the instructions, the
/// question, the history): where an injected name must never be found.
pub fn non_tool_text(body: &Value) -> String {
    messages(body)
        .iter()
        .filter(|m| m["role"] != "tool")
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

// ---- the read-only check (attack 10) --------------------------------------------------------

pub fn row_counts(dir: &TempDir) -> Vec<(String, i64)> {
    let conn = common::raw(dir);
    let mut names: Vec<String> = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' \
             ORDER BY name",
        )
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let n: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM \"{name}\""), [], |r| {
                    r.get(0)
                })
                .unwrap();
            (name, n)
        })
        .collect()
}

pub fn assert_only_log_grew(before: &[(String, i64)], after: &[(String, i64)], new_rows: i64) {
    assert_eq!(before.len(), after.len(), "a table appeared or went");
    for ((name, was), (_, now)) in before.iter().zip(after.iter()) {
        let expected = if name == "assistant_questions" {
            was + new_rows
        } else {
            *was
        };
        assert_eq!(*now, expected, "table {name} changed");
    }
}

/// Asks one question with recorded replies, and checks that only the log grew (by one row).
pub fn ask(
    h: &Harness,
    text: &str,
    conv: Option<&str>,
    replies: Vec<Value>,
) -> (AskReply, Vec<Value>) {
    let before = row_counts(&h.world.dir);
    let out = h.ask(text, conv, replies);
    assert_only_log_grew(&before, &row_counts(&h.world.dir), 1);
    out
}

// ---- a reader that runs the tools directly -------------------------------------------------

/// Runs the tools the way the loop does (`tools::call`, then `fit` and `tool_result_json`), over
/// one results list, and keeps every JSON text it made. The texts are exactly what a request to
/// the model would carry for these calls.
pub struct Reader<'a> {
    pub env: Env<'a>,
    pub state: State,
    pub words: &'a Words,
    /// Every result and every refusal as the model would read it.
    pub seen: Vec<String>,
    /// Tool calls that gave a result.
    pub ok: usize,
    /// Tool calls that were refused (an error result).
    pub refused: usize,
    /// Tool calls that stopped for the partner code.
    pub paused: usize,
    /// Results by what was read: `report:R13`, `analytics:F`, `tiles`, `find:partner` ...
    pub by: std::collections::BTreeMap<String, usize>,
    /// Refusals by error code.
    pub codes: std::collections::BTreeMap<String, usize>,
}

impl Reader<'_> {
    /// One call. Gives the id of the result (`r4`), or `None` for a refusal or a pause.
    pub fn call(&mut self, name: &str, args: Value) -> Option<String> {
        let arguments = args.to_string();
        let what = match name {
            "report" => format!("report:{}", args["id"].as_str().unwrap_or("?")),
            "analytics" => format!("analytics:{}", args["view"].as_str().unwrap_or("?")),
            "find" => format!("find:{}", args["role"].as_str().unwrap_or("?")),
            "app_state" => format!("app_state:{}", args["what"].as_str().unwrap_or("?")),
            other => other.to_owned(),
        };
        match tools::call(&self.env, &self.state, name, &arguments) {
            Ok(draft) => {
                *self.by.entry(what).or_default() += 1;
                let prefix = match draft.tool {
                    "calculate" => 'c',
                    "estimate" => 'e',
                    _ => 'r',
                };
                let id = self.state.next_id(prefix);
                let mut result = tools::stored(draft, id.clone(), 0);
                fit(&mut result, self.words);
                self.seen
                    .push(tool_result_json(&result, self.words).to_string());
                self.state.results.push(result);
                self.ok += 1;
                Some(id)
            }
            Err(Stop::Error(error)) => {
                *self.codes.entry(error.code.clone()).or_default() += 1;
                self.seen.push(error.to_json().to_string());
                self.refused += 1;
                None
            }
            Err(Stop::Locked) => {
                self.paused += 1;
                None
            }
        }
    }

    /// A find, then the path of its first match (`r3.matches.0`), or `None` when it matched nobody.
    pub fn first_match(&mut self, text: &str, role: &str) -> Option<String> {
        let id = self.call("find", json!({"text": text, "role": role}))?;
        let result = self.state.find(&id)?;
        let has = matches!(
            result.node.get(&["matches"]),
            Some(mandi_assistant::results::Node::List(items)) if !items.is_empty()
        );
        has.then(|| format!("{id}.matches.0"))
    }

    pub fn text(&self) -> String {
        self.seen.join("\n")
    }
}

/// Runs `f` with a reader over the world. `locked` is the state of the partner section the tools
/// are told (the app reads it with `assistant::partner_locked` at the start of a question).
pub fn with_reader<R>(
    w: &world::World,
    language: Language,
    locked: bool,
    f: impl FnOnce(&mut Reader<'_>) -> R,
) -> R {
    let words = Words::new().expect("the Fluent words");
    let today_iso = mandi_services::assistant::pakistan_day(&w.env.ctx);
    let today = BusinessDate::parse_iso(&today_iso).expect("today");
    let snap = w.env.ctx.snapshot().expect("a read snapshot");
    let env = Env {
        ctx: &snap,
        actor: &w.owner,
        words: &words,
        language,
        today,
        partner_locked: locked,
        numbers: &[],
        question: "",
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };
    let mut reader = Reader {
        env,
        state: State::default(),
        words: &words,
        seen: Vec::new(),
        ok: 0,
        refused: 0,
        paused: 0,
        by: std::collections::BTreeMap::new(),
        codes: std::collections::BTreeMap::new(),
    };
    f(&mut reader)
}
