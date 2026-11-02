//! The 8 read-only tools (PRD FR-7.87, Code Standards Q1 to Q3, D6-system-prompt Section 3).
//!
//! The model reaches the books only through these. Each tool:
//!
//! 1. reads its input **strictly** (an unknown field, a wrong type or an unknown name is an error
//!    result and nothing runs: Code Standards Q1);
//! 2. resolves the records and dates the model names from results it was shown, never from typed
//!    names (D6-system-prompt "Rules for the inputs");
//! 3. calls the same function as its screen (Code Standards Q3: `reports::run`, the analytics view
//!    functions, the dashboard tiles, `assist::party_card`, the search index);
//! 4. builds a [`Node`] tree of typed values for the model and for the checks (`results.rs`).
//!
//! No tool writes anything. Partner sources pause the question while the section is locked (PRD
//! FR-7.82): [`Stop::Locked`], with no model call made with a locked result.

use mandi_reports::dashboard::BackupFacts;
use mandi_reports::partner_guard::{PartnerNeed, analytics_need, report_need};
use mandi_services::auth::Actor;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::error::ErrorCode;
use mandi_services::domain::figures::QuestionNumber;
use mandi_services::{Context, ServiceError};
use serde_json::{Value, json};

use crate::answer::{AnswerSource, AnswerWorking};
use crate::results::{Node, Shown, ToolResult, Val, Words};

pub mod analytics;
pub mod calc;
pub mod cells;
pub mod find;
pub mod help;
pub mod party;
pub mod period;
pub mod records;
pub mod report;
pub mod teach;
pub mod tiles;

/// Everything a tool reads from outside its input.
pub struct Env<'a> {
    /// The services context (the database, the partner lock, the search index).
    pub ctx: &'a Context,
    /// The signed-in Owner.
    pub actor: &'a Actor,
    /// The Fluent words of the model's JSON.
    pub words: &'a Words,
    /// The app language.
    pub language: Language,
    /// Today, the Pakistan date.
    pub today: BusinessDate,
    /// True while the partner section is locked.
    pub partner_locked: bool,
    /// The numbers the Owner typed in this question.
    pub numbers: &'a [QuestionNumber],
    /// What the backup service says about the cloud copy (for the tile `last_backup`).
    pub backup: BackupFacts,
}

/// The results of the conversation so far, oldest first. Ids run on through the conversation.
#[derive(Debug, Default, Clone)]
pub struct State {
    /// Every result, with its question's `turn`.
    pub results: Vec<ToolResult>,
    /// The question being answered.
    pub turn: u32,
}

impl State {
    /// The result with this id.
    pub fn find(&self, id: &str) -> Option<&ToolResult> {
        self.results.iter().find(|r| r.id == id)
    }

    /// The next id with this letter: `r1`, `r2` ... `c1` ... `e1`.
    pub fn next_id(&self, prefix: char) -> String {
        let used = self
            .results
            .iter()
            .filter(|r| r.id.starts_with(prefix))
            .count();
        format!("{prefix}{}", used.saturating_add(1))
    }

    /// True when a record key (`a20`) appears in a result of the conversation: the model may pass
    /// a key it was shown (D6-system-prompt "Rules for the inputs").
    pub fn knows_key(&self, key: &str) -> bool {
        self.results.iter().any(|r| node_has_key(&r.node, key))
    }
}

fn node_has_key(node: &Node, key: &str) -> bool {
    match node {
        Node::Obj(fields) => fields.iter().any(|(k, v)| {
            (k == "key" && matches!(v, Node::Str(s) if s == key)) || node_has_key(v, key)
        }),
        Node::List(items) => items.iter().any(|n| node_has_key(n, key)),
        Node::Keyed(rows) => rows.iter().any(|(k, v)| k == key || node_has_key(v, key)),
        _ => false,
    }
}

/// A tool refused its input or could not run. Sent to the model as an error result, never shown
/// to the Owner; the model may correct its input once. So the refusal says what to fix: the field
/// as the tool's own input spells it, a reason in words and, where the input has a short list of
/// right words (columns, views, filters), that list (plan 51 B2, `teach.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolError {
    /// An `ERR_` code.
    pub code: String,
    /// The input field it belongs to.
    pub field: Option<String>,
    /// A short reason for the model. Never empty in the JSON the model reads.
    pub detail: String,
    /// The words that are allowed in the field, when there is a short list of them.
    pub allowed: Vec<String>,
}

impl ToolError {
    /// A refusal.
    pub fn new(code: &str, field: &str, detail: &str) -> Self {
        Self {
            code: code.to_owned(),
            field: Some(field.to_owned()),
            detail: detail.to_owned(),
            allowed: Vec::new(),
        }
    }

    /// The same refusal with the words that are allowed.
    #[must_use]
    pub fn allowed(mut self, words: Vec<String>) -> Self {
        self.allowed = words;
        self
    }

    /// A refusal from a service error code. The field is renamed as the tool's input spells it
    /// (`filters.from` is `from`) and the reason is in words.
    pub fn from_code(code: ErrorCode, field: Option<&str>) -> Self {
        let text = serde_json::to_value(code)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_else(|| "ERR_INTERNAL".to_owned());
        let field = field.map(teach::tool_field);
        let detail = teach::detail_for(&text, field.as_deref());
        Self {
            code: text,
            field,
            detail,
            allowed: Vec::new(),
        }
    }

    /// The error result the model reads.
    pub fn to_json(&self) -> Value {
        let detail = if self.detail.is_empty() {
            teach::detail_for(&self.code, self.field.as_deref())
        } else {
            self.detail.clone()
        };
        if self.allowed.is_empty() {
            json!({"error": {"code": self.code, "field": self.field, "detail": detail}})
        } else {
            json!({"error": {"code": self.code, "field": self.field, "detail": detail,
                             "allowed": self.allowed}})
        }
    }
}

/// Why a tool did not give a result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stop {
    /// Refused: an error result for the model.
    Error(ToolError),
    /// A partner source while the section is locked: the question pauses (PRD FR-7.82).
    Locked,
}

impl From<ToolError> for Stop {
    fn from(error: ToolError) -> Self {
        Stop::Error(error)
    }
}

/// A service failure as a stop: a rejection keeps its code and field; anything else is internal.
pub(crate) fn service_stop(error: ServiceError) -> Stop {
    match error {
        ServiceError::Rejected(items) => {
            let first = items.into_iter().next();
            match first {
                Some(item) if item.code == ErrorCode::ErrPartnerLocked => Stop::Locked,
                Some(item) => Stop::Error(ToolError::from_code(item.code, item.field.as_deref())),
                None => Stop::Error(ToolError::from_code(ErrorCode::ErrInternal, None)),
            }
        }
        _ => Stop::Error(ToolError::from_code(ErrorCode::ErrInternal, None)),
    }
}

/// What a tool built, before it gets its id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    /// The tool.
    pub tool: &'static str,
    /// The input as the model sent it.
    pub input: Value,
    /// What the model reads and cites.
    pub node: Node,
    /// What the screen can draw.
    pub shown: Option<Shown>,
    /// The source for the buttons.
    pub source: Option<AnswerSource>,
    /// True when partner parts were left out.
    pub partner_parts_locked: bool,
    /// True when the result holds partner figures.
    pub partner_data: bool,
    /// The working of a calculator result.
    pub working: Option<AnswerWorking>,
}

impl Draft {
    /// A draft with a node only.
    pub fn new(tool: &'static str, input: Value, node: Node) -> Self {
        Self {
            tool,
            input,
            node,
            shown: None,
            source: None,
            partner_parts_locked: false,
            partner_data: false,
            working: None,
        }
    }
}

/// A node found by a reference: a node of a result, or a number of the question.
pub enum Found<'s> {
    /// A node inside a result.
    Ref(&'s Node),
    /// A number of the question (`q.n1`).
    Owned(Node),
}

impl Found<'_> {
    /// The node.
    pub fn node(&self) -> &Node {
        match self {
            Found::Ref(n) => n,
            Found::Owned(n) => n,
        }
    }
}

/// Follows a reference path (`r2.balance`, `q.n1`) in this question's results and numbers.
///
/// # Errors
/// `ERR_UNKNOWN_REFERENCE` for a path that does not exist, and for a result of an earlier question
/// (every figure is read for this question, PRD FR-7.76).
pub fn lookup<'s>(
    env: &Env<'_>,
    state: &'s State,
    path: &str,
    field: &str,
) -> Result<Found<'s>, ToolError> {
    let unknown = || {
        // The numbers of the question and the results of this question are what a path may start
        // with, so the model can see what it was allowed to name (plan 51 B2).
        let mut allowed: Vec<String> = (1..=env.numbers.len()).map(|n| format!("q.n{n}")).collect();
        allowed.extend(
            state
                .results
                .iter()
                .filter(|r| r.turn == state.turn)
                .map(|r| r.id.clone()),
        );
        ToolError::new(
            "ERR_UNKNOWN_REFERENCE",
            field,
            &format!(
                "{path}: no such path. A number comes from the question (q.n1); a figure from a \
                 result of this question, such as r2.balance"
            ),
        )
        .allowed(allowed)
    };
    let parts: Vec<&str> = path.split('.').collect();
    let (first, rest) = parts.split_first().ok_or_else(unknown)?;
    if *first == "q" {
        let which = rest.first().ok_or_else(unknown)?;
        let n: usize = which
            .strip_prefix('n')
            .and_then(|d| d.parse().ok())
            .ok_or_else(unknown)?;
        let number = n
            .checked_sub(1)
            .and_then(|i| env.numbers.get(i))
            .ok_or_else(unknown)?;
        return Ok(Found::Owned(Node::Val(Val::Question {
            number: number.number,
        })));
    }
    let result = state.find(first).ok_or_else(unknown)?;
    if result.turn != state.turn {
        return Err(ToolError::new(
            "ERR_EARLIER_QUESTION",
            field,
            "a result of an earlier question; read it again",
        ));
    }
    result.node.get(rest).map(Found::Ref).ok_or_else(unknown)
}

/// A record the model names: its key prefix (`a`, `i`, `s` ...) and its id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rec {
    /// The letters of the key.
    pub prefix: String,
    /// The id.
    pub id: i64,
}

/// Splits `a20` into its letters and its id.
pub fn parse_key(key: &str) -> Option<Rec> {
    let digits_at = key.find(|c: char| c.is_ascii_digit())?;
    let (letters, digits) = key.split_at(digits_at);
    if letters.is_empty() || !letters.chars().all(|c| c.is_ascii_lowercase()) {
        return None;
    }
    Some(Rec {
        prefix: letters.to_owned(),
        id: digits.parse().ok()?,
    })
}

/// Reads a record from a find match (`r1.matches.0`) or from a key the model was shown (`a20`).
///
/// # Errors
/// `ERR_UNKNOWN_REFERENCE` for a path or a key the conversation never showed.
pub fn record(env: &Env<'_>, state: &State, text: &str, field: &str) -> Result<Rec, ToolError> {
    let bad = || ToolError::new("ERR_UNKNOWN_REFERENCE", field, text);
    if text.contains('.') {
        let found = lookup(env, state, text, field)?;
        let key = match found.node() {
            Node::Obj(fields) => fields.iter().find_map(|(k, v)| match (k.as_str(), v) {
                ("key", Node::Str(s)) => Some(s.clone()),
                _ => None,
            }),
            Node::Str(s) => Some(s.clone()),
            _ => None,
        };
        return key.as_deref().and_then(parse_key).ok_or_else(bad);
    }
    if !state.knows_key(text) {
        return Err(bad());
    }
    parse_key(text).ok_or_else(bad)
}

/// True when the path is the `from` of a result of the `period` tool, for example `r1.from`.
fn is_period_from(state: &State, path: &str) -> bool {
    let mut parts = path.split('.');
    let (Some(id), Some("from"), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    state.find(id).is_some_and(|r| r.tool == "period")
}

/// A date the model passes: a period result field (`r2.from`) or `YYYY-MM-DD`.
///
/// `None` is "no date": a period field that is empty, such as the `from` of `up_to_now` (from the
/// first day of the books), stays open, as an empty `from` does in the report screen (plan 51 B2).
///
/// # Errors
/// `ERR_DATE_INVALID` for a date that does not exist; `ERR_UNKNOWN_REFERENCE`.
pub fn date_arg(
    env: &Env<'_>,
    state: &State,
    text: &str,
    field: &str,
) -> Result<Option<String>, ToolError> {
    if text.starts_with(|c: char| c.is_ascii_alphabetic()) {
        let found = lookup(env, state, text, field)?;
        return match found.node() {
            Node::Val(Val::Date(d)) => Ok(Some(d.clone())),
            // Only the `from` of a period result may be empty (`up_to_now`: from the first day of
            // the books). Any other empty field is not a date: `as_of: "r1.from"` would otherwise
            // quietly become today (money review, plan 51 B2).
            Node::Null if field == "from" && is_period_from(state, text) => Ok(None),
            Node::Null => Err(ToolError::new(
                "ERR_UNKNOWN_REFERENCE",
                field,
                "this path has no date; a period such as up_to_now has no start date, so only \
                 from may use it, and to and as_of need a real date",
            )),
            _ => Err(ToolError::new(
                "ERR_UNKNOWN_REFERENCE",
                field,
                "this path is not a date; use the from or to of a period result, or YYYY-MM-DD",
            )),
        };
    }
    BusinessDate::parse_iso(text)
        .map(|d| Some(d.to_iso()))
        .ok_or_else(|| {
            ToolError::new(
                "ERR_DATE_INVALID",
                field,
                "not a date; write YYYY-MM-DD or use a period result",
            )
        })
}

/// Reads a tool's input strictly: an unknown field or a wrong type is a refusal.
///
/// # Errors
/// `ERR_FORMAT_INVALID`.
pub fn input<T: serde::de::DeserializeOwned>(args: &Value) -> Result<T, ToolError> {
    serde_json::from_value(args.clone())
        .map_err(|e| ToolError::new("ERR_FORMAT_INVALID", "input", &e.to_string()))
}

/// Whether a report or a view needs the partner code, and if the section is locked what to do
/// (PRD FR-7.82): refuse and pause, or run without the partner parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartnerRule {
    /// No partner figure in it.
    Free,
    /// Run it, leaving out the partner parts.
    HideParts,
    /// Pause the question.
    Pause,
}

/// The rule for a report.
pub fn report_rule(
    id: mandi_reports::reports::ReportId,
    filters: &mandi_reports::reports::ReportFilters,
) -> PartnerRule {
    rule(report_need(id, filters))
}

/// The rule for an analytics view.
pub fn view_rule(
    view: mandi_reports::analytics::AnalyticsView,
    filters: &mandi_reports::analytics::AnalyticsFilters,
) -> PartnerRule {
    rule(analytics_need(view, filters))
}

fn rule(need: PartnerNeed) -> PartnerRule {
    match need {
        PartnerNeed::None => PartnerRule::Free,
        PartnerNeed::HideParts => PartnerRule::HideParts,
        PartnerNeed::Refuse => PartnerRule::Pause,
    }
}

/// Runs one tool call.
///
/// # Errors
/// [`Stop::Error`] with the refusal for the model; [`Stop::Locked`] for a partner source.
pub fn call(env: &Env<'_>, state: &State, name: &str, arguments: &str) -> Result<Draft, Stop> {
    let args: Value = serde_json::from_str(arguments).map_err(|_| {
        Stop::Error(ToolError::new(
            "ERR_FORMAT_INVALID",
            "input",
            "the input is not JSON",
        ))
    })?;
    match name {
        "find" => find::run(env, state, &args),
        "party_card" => party::run(env, state, &args),
        "report" => report::run_report(env, state, &args),
        "analytics" => analytics::run(env, state, &args),
        "tiles" => tiles::run(env, state, &args),
        "period" => period::run(env, state, &args),
        "calculate" => calc::run_calculate(env, state, &args),
        "estimate" => calc::run_estimate(env, state, &args),
        "app_help" => help::run(env, state, &args),
        _ => Err(Stop::Error(ToolError::new(
            "ERR_UNKNOWN_TOOL",
            "name",
            name,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_splits_into_letters_and_id() {
        assert_eq!(
            parse_key("a20"),
            Some(Rec {
                prefix: "a".to_owned(),
                id: 20
            })
        );
        assert_eq!(
            parse_key("je4518"),
            Some(Rec {
                prefix: "je".to_owned(),
                id: 4518
            })
        );
        assert_eq!(parse_key("20"), None);
        assert_eq!(parse_key("a"), None);
        assert_eq!(parse_key("A20"), None);
        assert_eq!(parse_key("a2x"), None);
    }

    #[test]
    fn ids_run_on_through_the_conversation() {
        let mut state = State::default();
        assert_eq!(state.next_id('r'), "r1");
        state.results.push(ToolResult {
            id: "r1".to_owned(),
            tool: "find",
            input: Value::Null,
            node: Node::Null,
            shown: None,
            source: None,
            partner_parts_locked: false,
            partner_data: false,
            turn: 0,
            working: None,
        });
        assert_eq!(state.next_id('r'), "r2");
        assert_eq!(state.next_id('c'), "c1");
    }

    #[test]
    fn an_error_result_carries_its_code_and_field() {
        let e = ToolError::new("ERR_UNKNOWN_REFERENCE", "party", "r9.matches.0");
        assert_eq!(e.to_json()["error"]["code"], "ERR_UNKNOWN_REFERENCE");
        assert_eq!(e.to_json()["error"]["field"], "party");
        let from = ToolError::from_code(ErrorCode::ErrNoBase, None);
        assert_eq!(from.code, "ERR_NO_BASE");
    }
}
