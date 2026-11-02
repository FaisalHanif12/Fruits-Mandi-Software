//! The loop of one question (Tech Stack 4.8, PRD FR-7.72 to FR-7.82, Code Standards Q1 to Q10).
//!
//! `ask` takes a typed question; `pick` goes on with a question that waits ("کون سا؟" or the
//! partner code); `rerun` reads an old answer again on today's books; `test_connection` sends one
//! fixed question to see that the key and the model work. Chips are in `quick.rs`.
//!
//! The loop, for one question: build the request (the fixed instructions, the last 3 questions,
//! this question), call the model, run the tools it asks for (a partner source while the section
//! is locked pauses the question with no model call), call again with the results, and repeat until
//! it gives its answer: at most 4 model calls and 8 tool calls. The answer is checked
//! (`check.rs`); if a check fails the model writes it once more; if that fails too the tool
//! results are shown as plain tables. A figure whose two places differ ends the question at once.
//! Every figure on the screen comes from a tool result (`fill.rs`). The question is written to the
//! log after the answer, in its own save.
//!
//! One moment (NFR-AI-6): a question opens a read-only copy of the books at its start
//! (`Context::snapshot`, a read transaction on a connection of its own; SQLite is in WAL mode, so it
//! blocks no writer and no other screen) and runs ALL its tools on that copy. Whatever is saved
//! while the model thinks is not seen by any tool of the question, so no two figures of one answer
//! are from two moments. A question that goes on after a click or the partner code opens a new copy
//! and reads its earlier tool calls again on it (`refresh`), so it too has one moment.
//!
//! Example (worked day, "ikram ke total bakaya kitne hai"): call 1 gives `find`; call 2 gives
//! `party_card` and `report R13` together; call 3 gives the answer with `{{r2.balance}}`; it passes
//! the checks; the reply is `answered` with the figure 2,880 بنام ✓ and the R13 table.

use std::time::{Duration, Instant};

use chrono::SecondsFormat;
use mandi_reports::analytics::catalog as analytics_catalog;
use mandi_reports::dashboard::BackupFacts;
use mandi_reports::reports::{ArgValue, MessageArg, Names, ReportId, catalog as report_catalog};
use mandi_services::auth::Actor;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::error::ErrorCode;
use mandi_services::domain::figures::QuestionNumber;
use mandi_services::{Context, ServiceError, assistant, figures};
use serde_json::{Value, json};

use crate::answer::{Answer, AnswerOption, AnswerSource, EntryRoute, EntryScreen};
use crate::ask::{
    AskOutcome, AskReply, AssistantAskInput, AssistantFailure, AssistantPickInput,
    AssistantRerunInput, FailureKind, PickChoice,
};
use crate::check::{Check, NameSet, Verdict};
use crate::client::{CallError, ModelClient, Reply, ToolCall};
use crate::conversation::{Conversation, Turn, Wait, Waiting};
use crate::fill;
use crate::limits::{
    CALL_TIMEOUT_SECS, DAILY_LIMIT, MAX_CALL_OUTPUT_TOKENS, MAX_INPUT_TOKENS, MAX_MODEL_CALLS,
    MAX_OUTPUT_TOKENS, MAX_QUESTION_CHARS, MAX_REWRITES, MAX_TOOL_CALLS, QUESTION_TIMEOUT_SECS,
};
use crate::model_answer::{AnswerKind, EntryScreenName, ModelAnswer, Nearest};
use crate::request::{Parts, RESULT_MARK, request_body, user_message};
use crate::results::{ToolResult, Words, fit, tool_result_json};
use crate::runtime::Runtime;
use crate::settings::{AssistantTestResult, DEFAULT_MODEL};
use crate::source::{AnswerRecord, SourceParty, SourceReportFilters};
use crate::step::AssistantStep;
use crate::tools::{self, Env, State, Stop, ToolError};

/// What the loop needs from outside.
pub struct Deps<'a> {
    /// The services context.
    pub ctx: &'a Context,
    /// The signed-in Owner.
    pub actor: &'a Actor,
    /// The assistant's memory and key store.
    pub runtime: &'a Runtime,
    /// Where model calls go.
    pub client: &'a dyn ModelClient,
    /// Where step lines go (the `assistant://step` event).
    pub emit: &'a dyn Fn(AssistantStep),
    /// What the backup service says about the cloud copy (for the tile `last_backup`).
    pub backup: BackupFacts,
}

/// How a question ended, before it is written to the log.
pub(crate) struct Finished {
    pub outcome: AskOutcome,
    pub answer: Option<Answer>,
    pub options: Vec<AnswerOption>,
    pub more_options: bool,
    pub entry_route: Option<EntryRoute>,
    pub locked: bool,
    pub failure: Option<AssistantFailure>,
    /// The answer as the model wrote it, for the log.
    pub model_text: Option<String>,
    pub wait: Option<Wait>,
    pub finish_reason: Option<String>,
}

impl Finished {
    pub(crate) fn plain(outcome: AskOutcome) -> Self {
        Self {
            outcome,
            answer: None,
            options: Vec::new(),
            more_options: false,
            entry_route: None,
            locked: false,
            failure: None,
            model_text: None,
            wait: None,
            finish_reason: None,
        }
    }

    fn failed(error: CallError) -> Self {
        let outcome = if error.kind == FailureKind::Offline {
            AskOutcome::Offline
        } else {
            AskOutcome::Error
        };
        Self {
            failure: Some(error.failure()),
            ..Self::plain(outcome)
        }
    }
}

/// The kind of row written to the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Source {
    Model,
    Continue,
    Rerun,
    Chip,
    Test,
}

impl Source {
    fn word(self) -> &'static str {
        match self {
            Source::Model => "model",
            Source::Continue => "continue",
            Source::Rerun => "rerun",
            Source::Chip => "chip",
            Source::Test => "test",
        }
    }

    fn counts(self) -> bool {
        matches!(self, Source::Model | Source::Test)
    }
}

fn reject(code: ErrorCode) -> ServiceError {
    ServiceError::reject(code)
}

fn outcome_word(outcome: AskOutcome) -> &'static str {
    match outcome {
        AskOutcome::Answered => "answered",
        AskOutcome::Estimate => "estimate",
        AskOutcome::Advice => "advice",
        AskOutcome::WhichOne => "which_one",
        AskOutcome::NeedDates => "need_dates",
        AskOutcome::NeedParty => "need_party",
        AskOutcome::NotFound => "not_found",
        AskOutcome::OutsideBooks => "outside_books",
        AskOutcome::ChangeRefused => "change_refused",
        AskOutcome::NotUnderstood => "not_understood",
        AskOutcome::ChecksFailed => "checks_failed",
        AskOutcome::RecordsDiffer => "records_differ",
        AskOutcome::Locked => "locked",
        AskOutcome::LimitReached => "limit_reached",
        AskOutcome::Offline => "offline",
        AskOutcome::Error => "error",
    }
}

fn prefix_of(tool: &str) -> char {
    match tool {
        "calculate" => 'c',
        "estimate" => 'e',
        _ => 'r',
    }
}

fn text_arg(name: &str, text: &str) -> MessageArg {
    MessageArg {
        name: name.to_owned(),
        value: ArgValue::Text {
            text: text.to_owned(),
        },
    }
}

/// A name argument in both scripts, with the tag after each (like `displayName`), so a language
/// switch while a question runs shows the right script (CLAUDE.md rule 8, Code Standards F38).
fn names_arg(name: &str, record: &AnswerRecord) -> MessageArg {
    let with_tag = |text: &str| match &record.tag {
        Some(tag) => format!("{text} {tag}"),
        None => text.to_owned(),
    };
    MessageArg {
        name: name.to_owned(),
        value: ArgValue::Names {
            names: Names {
                ur: with_tag(&record.name_ur),
                en: with_tag(&record.name_en),
            },
        },
    }
}

/// The step line of a tool call, in the panel's own words (Fluent id and arguments). A party's
/// name goes as `ArgValue::Names`; what the Owner typed (`find`) and the report id go as text.
fn step_of(call: &ToolCall, env: &Env<'_>, state: &State) -> AssistantStep {
    let input: Value = serde_json::from_str(&call.arguments).unwrap_or(Value::Null);
    let field = |k: &str| input.get(k).and_then(Value::as_str).unwrap_or("");
    // The party is a path of an earlier result (`r1.matches.0`): found as the call itself finds it.
    let party_arg = |path: &str| {
        tools::record(env, state, path, "party")
            .ok()
            .and_then(|r| tools::records::answer_record(state, &format!("{}{}", r.prefix, r.id)))
            .map(|r| names_arg("name", &r))
            .into_iter()
            .collect::<Vec<_>>()
    };
    let (id, args) = match call.name.as_str() {
        "find" => (
            "assistant-step-finding",
            vec![text_arg("text", field("text"))],
        ),
        "party_card" => ("assistant-step-reading-card", party_arg(field("party"))),
        "report" if field("id") == "R13" => {
            ("assistant-step-reading-ledger", party_arg(field("party")))
        }
        "report" => (
            "assistant-step-reading-report",
            vec![text_arg("report", field("id"))],
        ),
        "analytics" => ("assistant-step-reading-analytics", Vec::new()),
        "tiles" => ("assistant-step-reading-tiles", Vec::new()),
        "period" => ("assistant-step-period", Vec::new()),
        "calculate" => ("assistant-step-calculating", Vec::new()),
        "estimate" => ("assistant-step-estimating", Vec::new()),
        _ => ("assistant-step-generic", Vec::new()),
    };
    AssistantStep {
        id: id.to_owned(),
        args,
    }
}

fn step(id: &str) -> AssistantStep {
    AssistantStep {
        id: id.to_owned(),
        args: Vec::new(),
    }
}

/// The state of one run: the words, the language, today and the limits already used.
pub(crate) struct Run<'a> {
    pub d: &'a Deps<'a>,
    pub words: Words,
    pub language: Language,
    pub today: BusinessDate,
    pub today_iso: String,
    pub model: String,
    pub partner_locked: bool,
    /// The books at the moment the question began; every tool reads this, never `d.ctx`.
    pub snap: Context,
    started: Instant,
}

impl<'a> Run<'a> {
    pub(crate) fn new(d: &'a Deps<'a>) -> Result<Self, ServiceError> {
        // The moment of the question: fixed before anything else is read.
        let snap = d.ctx.snapshot()?;
        let words = Words::new().map_err(|e| ServiceError::Data(format!("fluent: {e}")))?;
        let today_iso = assistant::pakistan_day(d.ctx);
        let today = BusinessDate::parse_iso(&today_iso)
            .ok_or_else(|| ServiceError::Data("the date is wrong".to_owned()))?;
        let config = assistant::config(d.ctx)?;
        Ok(Self {
            d,
            words,
            language: assistant::language(d.ctx, d.actor)?,
            today,
            today_iso,
            model: config.model.unwrap_or_else(|| DEFAULT_MODEL.to_owned()),
            partner_locked: assistant::partner_locked(d.ctx)?,
            snap,
            started: Instant::now(),
        })
    }

    pub(crate) fn env<'e>(&'e self, numbers: &'e [QuestionNumber]) -> Env<'e> {
        Env {
            ctx: &self.snap,
            actor: self.d.actor,
            words: &self.words,
            language: self.language,
            today: self.today,
            partner_locked: self.partner_locked,
            numbers,
            backup: self.d.backup,
        }
    }

    pub(crate) fn read_at(&self) -> String {
        self.d
            .ctx
            .now_utc()
            .to_rfc3339_opts(SecondsFormat::Secs, true)
    }

    /// Runs tool calls in order and puts their results into the conversation. A partner source
    /// while the section is locked stops the run: the call and the ones after it wait.
    pub(crate) fn run_calls(
        &self,
        conv: &mut Conversation,
        turn: &mut Turn,
        calls: Vec<ToolCall>,
    ) -> Option<Vec<ToolCall>> {
        let index = u32::try_from(conv.turns.len()).unwrap_or(u32::MAX);
        let numbers = figures::question_numbers(&turn.question);
        let mut state = State {
            results: std::mem::take(&mut conv.results),
            turn: index,
        };
        let mut paused = None;
        let mut iter = calls.into_iter();
        while let Some(call) = iter.next() {
            if turn.counters.tool_calls >= MAX_TOOL_CALLS {
                turn.messages.push(tool_message(
                    &call.id,
                    ToolError::new("ERR_TOO_MANY_CALLS", "tool", "8 tool calls are the most")
                        .to_json()
                        .to_string(),
                    None,
                ));
                continue;
            }
            let env = self.env(&numbers);
            // The step line goes out as the tool starts, so a slow report shows what it is doing.
            (self.d.emit)(step_of(&call, &env, &state));
            match tools::call(&env, &state, &call.name, &call.arguments) {
                Ok(draft) => {
                    turn.counters.tool_calls = turn.counters.tool_calls.saturating_add(1);
                    let id = state.next_id(prefix_of(draft.tool));
                    let mut result = ToolResult {
                        id: id.clone(),
                        tool: draft.tool,
                        input: draft.input,
                        node: draft.node,
                        shown: draft.shown,
                        source: draft.source,
                        partner_parts_locked: draft.partner_parts_locked,
                        partner_data: draft.partner_data,
                        turn: index,
                        working: draft.working,
                    };
                    fit(&mut result, &self.words);
                    let content = tool_result_json(&result, &self.words).to_string();
                    turn.tool_log.push(json!({
                        "id": id,
                        "tool": result.tool,
                        "input": result.input,
                    }));
                    state.results.push(result);
                    turn.messages
                        .push(tool_message(&call.id, content, Some(&id)));
                }
                Err(Stop::Error(error)) => {
                    turn.counters.tool_calls = turn.counters.tool_calls.saturating_add(1);
                    // A refused call is in the log too, with its code (it has no result id).
                    turn.tool_log.push(json!({
                        "tool": call.name,
                        "input": serde_json::from_str::<Value>(&call.arguments).unwrap_or(Value::Null),
                        "error": error.code,
                    }));
                    turn.messages
                        .push(tool_message(&call.id, error.to_json().to_string(), None));
                }
                Err(Stop::Locked) => {
                    let mut rest = vec![call];
                    rest.extend(iter);
                    paused = Some(rest);
                    break;
                }
            }
        }
        conv.results = state.results;
        paused
    }

    /// One model call, within the limits of the question.
    fn call_model(
        &self,
        conv: &Conversation,
        turn: &mut Turn,
        rewrite: bool,
    ) -> Result<Reply, Box<Finished>> {
        let elapsed = self.started.elapsed();
        let total = Duration::from_secs(QUESTION_TIMEOUT_SECS);
        let c = &turn.counters;
        if elapsed >= total {
            return Err(Box::new(Finished::failed(CallError::new(
                FailureKind::Timeout,
            ))));
        }
        let over_calls = !rewrite && c.model_calls >= MAX_MODEL_CALLS;
        let over_rewrite = rewrite && c.rewrites >= MAX_REWRITES;
        let out_used = u32::try_from(c.tokens_out).unwrap_or(u32::MAX);
        let in_used = u32::try_from(c.tokens_in).unwrap_or(u32::MAX);
        if over_calls
            || over_rewrite
            || out_used >= MAX_OUTPUT_TOKENS
            || in_used >= MAX_INPUT_TOKENS
        {
            return Err(Box::new(Finished::plain(AskOutcome::NotUnderstood)));
        }
        let Ok(Some(key)) = self.d.runtime.keys().get() else {
            return Err(Box::new(Finished::failed(CallError::new(
                FailureKind::BadKey,
            ))));
        };
        let max_tokens = MAX_OUTPUT_TOKENS
            .saturating_sub(out_used)
            .min(MAX_CALL_OUTPUT_TOKENS);
        let body = request_body(&Parts {
            model: &self.model,
            conversation: conv,
            current: turn,
            words: &self.words,
            partner_locked: self.partner_locked,
            max_tokens,
        });
        let timeout = Duration::from_secs(CALL_TIMEOUT_SECS).min(total.saturating_sub(elapsed));
        let reply = self
            .d
            .client
            .call(&key, &body, timeout)
            .map_err(|e| Box::new(Finished::failed(e)))?;
        if rewrite {
            turn.counters.rewrites = turn.counters.rewrites.saturating_add(1);
        } else {
            turn.counters.model_calls = turn.counters.model_calls.saturating_add(1);
        }
        let c = &mut turn.counters;
        c.tokens_in = c.tokens_in.saturating_add(reply.usage.prompt_tokens);
        c.tokens_cached = c.tokens_cached.saturating_add(reply.usage.cached_tokens);
        c.tokens_out = c.tokens_out.saturating_add(reply.usage.completion_tokens);
        c.tokens_reasoning = c
            .tokens_reasoning
            .saturating_add(reply.usage.reasoning_tokens);
        Ok(reply)
    }

    /// The loop. `pending` are tool calls that waited for the partner code.
    pub(crate) fn drive(
        &self,
        conv: &mut Conversation,
        turn: &mut Turn,
        pending: Vec<ToolCall>,
    ) -> Finished {
        if !pending.is_empty()
            && let Some(rest) = self.run_calls(conv, turn, pending)
        {
            return paused(rest);
        }
        loop {
            let reply = match self.call_model(conv, turn, false) {
                Ok(r) => r,
                Err(f) => return *f,
            };
            if !matches!(reply.finish_reason.as_str(), "stop" | "tool_calls") {
                // `length` and anything else: the reply cannot be used.
                let mut f = Finished::plain(AskOutcome::NotUnderstood);
                f.finish_reason = Some(reply.finish_reason);
                return f;
            }
            if reply.tool_calls.is_empty() {
                (self.d.emit)(step("assistant-step-checking"));
                return self.finish(conv, turn, reply);
            }
            turn.messages.push(reply.message.clone());
            if let Some(rest) = self.run_calls(conv, turn, reply.tool_calls) {
                return paused(rest);
            }
        }
    }

    /// Runs this question's tool calls again, in order, keeping their ids.
    pub(crate) fn refresh(&self, conv: &mut Conversation, turn: &Turn) {
        let index = u32::try_from(conv.turns.len()).unwrap_or(u32::MAX);
        let numbers = figures::question_numbers(&turn.question);
        let old = std::mem::take(&mut conv.results);
        let mut state = State {
            results: old.iter().filter(|r| r.turn != index).cloned().collect(),
            turn: index,
        };
        for entry in &turn.tool_log {
            let (Some(id), Some(tool), Some(input)) = (
                entry.get("id").and_then(Value::as_str),
                entry.get("tool").and_then(Value::as_str),
                entry.get("input"),
            ) else {
                continue;
            };
            let env = self.env(&numbers);
            if let Ok(draft) = tools::call(&env, &state, tool, &input.to_string()) {
                let mut result = ToolResult {
                    id: id.to_owned(),
                    tool: draft.tool,
                    input: draft.input,
                    node: draft.node,
                    shown: draft.shown,
                    source: draft.source,
                    partner_parts_locked: draft.partner_parts_locked,
                    partner_data: draft.partner_data,
                    turn: index,
                    working: draft.working,
                };
                fit(&mut result, &self.words);
                state.results.push(result);
            }
        }
        conv.results = state.results;
    }

    /// Checks the final answer, lets the model write it once more if a check fails, and fills it.
    fn finish(&self, conv: &mut Conversation, turn: &mut Turn, reply: Reply) -> Finished {
        let mut content = reply.content.clone().unwrap_or_default();
        let finish_reason = reply.finish_reason.clone();
        let mut rewrote = false;
        loop {
            let Ok(parsed) = ModelAnswer::parse(&content) else {
                let mut f = Finished::plain(AskOutcome::NotUnderstood);
                f.model_text = Some(json_text(&content));
                f.finish_reason = Some(finish_reason);
                return f;
            };
            let numbers = figures::question_numbers(&turn.question);
            let state = State {
                results: conv.results.clone(),
                turn: u32::try_from(conv.turns.len()).unwrap_or(u32::MAX),
            };
            let env = self.env(&numbers);
            let exempt = assistant::exempt_names(&self.snap).unwrap_or_default();
            let names = NameSet::new(&state, &exempt);
            let verdict = Check {
                env: &env,
                state: &state,
                names: &names,
            }
            .run(&parsed);
            let read_at = self.read_at();
            let mut f = match verdict {
                Verdict::Ok => {
                    turn.messages
                        .push(json!({"role": "assistant", "content": content}));
                    turn.referenced = referenced_rows(&parsed);
                    self.conclude_answer(&env, &state, &parsed, &read_at)
                }
                Verdict::Differ(paths) => {
                    let mut f = Finished::plain(AskOutcome::RecordsDiffer);
                    f.answer = Some(fill::differ_answer(&env, &state, &paths, &read_at));
                    f
                }
                Verdict::Retry(failures) => {
                    if rewrote {
                        let mut f = Finished::plain(AskOutcome::ChecksFailed);
                        f.answer = Some(fill::plain_tables(&state, &read_at));
                        f
                    } else {
                        rewrote = true;
                        (self.d.emit)(step("assistant-step-rewriting"));
                        turn.messages
                            .push(json!({"role": "assistant", "content": content}));
                        let list: Vec<Value> = failures.iter().map(|x| x.to_json()).collect();
                        turn.messages.push(json!({
                            "role": "user",
                            "content": json!({"check_failed": list}).to_string(),
                        }));
                        match self.call_model(conv, turn, true) {
                            Ok(next) if next.tool_calls.is_empty() => {
                                content = next.content.unwrap_or_default();
                                continue;
                            }
                            // A new tool call in the rewrite is not allowed (D6-system-prompt 5.3).
                            Ok(_) => {
                                let mut f = Finished::plain(AskOutcome::ChecksFailed);
                                f.answer = Some(fill::plain_tables(&state, &read_at));
                                f
                            }
                            Err(f) => return *f,
                        }
                    }
                }
            };
            f.model_text = Some(json_text(&content));
            f.finish_reason = Some(finish_reason);
            return f;
        }
    }

    fn conclude_answer(
        &self,
        env: &Env<'_>,
        state: &State,
        parsed: &ModelAnswer,
        read_at: &str,
    ) -> Finished {
        let entry_route = parsed.entry.as_ref().map(|e| {
            let screen = entry_screen(e.screen);
            let wanted = party_prefix(screen);
            EntryRoute {
                screen,
                party: e
                    .party
                    .as_deref()
                    .and_then(|p| tools::record(env, state, p, "party").ok())
                    .filter(|r| r.prefix == wanted)
                    .and_then(|r| {
                        tools::records::answer_record(state, &format!("{}{}", r.prefix, r.id))
                    }),
            }
        });
        match parsed.kind {
            AnswerKind::Answer | AnswerKind::Estimate | AnswerKind::Advice => {
                let mut f = Finished::plain(match parsed.kind {
                    AnswerKind::Estimate => AskOutcome::Estimate,
                    AnswerKind::Advice => AskOutcome::Advice,
                    _ => AskOutcome::Answered,
                });
                f.answer = Some(fill::fill(env, state, parsed, read_at));
                // PRD E98 with D6-system-prompt `entry`: a change request carries the route, and
                // so may an answer or advice to a question that also asks for a change (the panel
                // then adds the change refused line and its button). An estimate carries none.
                f.entry_route = match parsed.kind {
                    AnswerKind::Estimate => None,
                    _ => entry_route,
                };
                f
            }
            AnswerKind::WhichOne | AnswerKind::NotFound => {
                let find_id = parsed.choose.clone().unwrap_or_default();
                let mut f = Finished::plain(if parsed.kind == AnswerKind::WhichOne {
                    AskOutcome::WhichOne
                } else {
                    AskOutcome::NotFound
                });
                f.options = fill::options_of(state, &find_id);
                f.more_options = state
                    .find(&find_id)
                    .and_then(|r| r.node.get(&["more"]).cloned())
                    .is_some_and(|n| matches!(n, crate::results::Node::Bool(true)));
                if parsed.kind == AnswerKind::WhichOne {
                    f.wait = Some(Wait::Which { find_id });
                }
                f
            }
            AnswerKind::NeedDates => Finished::plain(AskOutcome::NeedDates),
            AnswerKind::NeedParty => Finished::plain(AskOutcome::NeedParty),
            AnswerKind::NotUnderstood => Finished::plain(AskOutcome::NotUnderstood),
            AnswerKind::OutsideBooks => {
                let mut f = Finished::plain(AskOutcome::OutsideBooks);
                f.answer = parsed
                    .nearest
                    .as_ref()
                    .and_then(|n| nearest_source(env, state, n))
                    .map(|source| Answer {
                        parts: Vec::new(),
                        tables: Vec::new(),
                        charts: Vec::new(),
                        estimate: None,
                        advice: false,
                        sources: vec![source],
                        differences: Vec::new(),
                        next: Vec::new(),
                        read_at: read_at.to_owned(),
                    });
                f
            }
            AnswerKind::ChangeRequest => {
                let mut f = Finished::plain(AskOutcome::ChangeRefused);
                f.entry_route = entry_route;
                f
            }
        }
    }

    /// Writes the log row and builds the reply. The conversation is kept for the next question.
    pub(crate) fn conclude(
        &self,
        mut conv: Conversation,
        turn: Turn,
        f: Finished,
        source: Source,
        parent_id: Option<i64>,
    ) -> Result<AskReply, ServiceError> {
        let counted = source.counts() && turn.counters.model_calls > 0;
        let sources_json = f.answer.as_ref().map_or_else(
            || "[]".to_owned(),
            |a| serde_json::to_string(&a.sources).unwrap_or_else(|_| "[]".to_owned()),
        );
        let spec = assistant::QuestionSpec {
            question: turn.question.clone(),
            language: self.language,
            conversation_id: conv.id.clone(),
            parent_id,
            source: source.word(),
            counted,
            sources_json,
            outcome: outcome_word(f.outcome),
            tool_calls_json: Value::Array(turn.tool_log.clone()).to_string(),
            answer_json: f.model_text.clone(),
            model: (source != Source::Chip && source != Source::Rerun).then(|| self.model.clone()),
            model_calls: (source != Source::Chip && source != Source::Rerun).then(|| {
                i64::from(
                    turn.counters
                        .model_calls
                        .saturating_add(turn.counters.rewrites),
                )
            }),
            tokens_in: (source != Source::Chip && source != Source::Rerun)
                .then_some(turn.counters.tokens_in),
            tokens_cached: (source != Source::Chip && source != Source::Rerun)
                .then_some(turn.counters.tokens_cached),
            tokens_out: (source != Source::Chip && source != Source::Rerun)
                .then_some(turn.counters.tokens_out),
            tokens_reasoning: (source != Source::Chip && source != Source::Rerun)
                .then_some(turn.counters.tokens_reasoning),
            finish_reason: f.finish_reason.clone(),
        };
        let question_id = assistant::record_question(self.d.ctx, self.d.actor, &spec)?;
        conv.waiting = f.wait.clone().map(|wait| Waiting { question_id, wait });
        let conversation_id = conv.id.clone();
        conv.turns.push(turn);
        self.d.runtime.keep(conv);
        Ok(AskReply {
            question_id,
            conversation_id,
            outcome: f.outcome,
            answer: f.answer,
            options: f.options,
            more_options: f.more_options,
            entry_route: f.entry_route,
            locked: f.locked,
            failure: f.failure,
            count_today: assistant::count_today(self.d.ctx)?,
            limit: DAILY_LIMIT,
        })
    }
}

/// The model's text as JSON for the log: the log column holds valid JSON only, so a reply that is
/// not JSON is kept as a JSON string.
fn json_text(content: &str) -> String {
    if serde_json::from_str::<Value>(content).is_ok() {
        content.to_owned()
    } else {
        Value::String(content.to_owned()).to_string()
    }
}

fn tool_message(call_id: &str, content: String, result: Option<&str>) -> Value {
    let mut message = json!({"role": "tool", "tool_call_id": call_id, "content": content});
    if let (Some(id), Some(map)) = (result, message.as_object_mut()) {
        map.insert(RESULT_MARK.to_owned(), Value::String(id.to_owned()));
    }
    message
}

fn paused(rest: Vec<ToolCall>) -> Finished {
    let mut f = Finished::plain(AskOutcome::Locked);
    f.locked = true;
    f.wait = Some(Wait::Partner { pending: rest });
    f
}

/// The key prefix of the record a screen takes as its party: a partner (`s`) on the screens of
/// partner money, an account (`a`) on every other (see [`EntryRoute::party`]).
fn party_prefix(screen: EntryScreen) -> &'static str {
    match screen {
        EntryScreen::Kharcha | EntryScreen::Sarmaya | EntryScreen::Phal => "s",
        _ => "a",
    }
}

fn entry_screen(screen: EntryScreenName) -> EntryScreen {
    match screen {
        EntryScreenName::Bikri => EntryScreen::Bikri,
        EntryScreenName::Wasooli => EntryScreen::Wasooli,
        EntryScreenName::Adaigi => EntryScreen::Adaigi,
        EntryScreenName::Peshgi => EntryScreen::Peshgi,
        EntryScreenName::Kharcha => EntryScreen::Kharcha,
        EntryScreenName::UdharRaqam => EntryScreen::UdharRaqam,
        EntryScreenName::AlagKhata => EntryScreen::AlagKhata,
        EntryScreenName::Sarmaya => EntryScreen::Sarmaya,
        EntryScreenName::Phal => EntryScreen::Phal,
        EntryScreenName::General => EntryScreen::General,
        EntryScreenName::Account => EntryScreen::Account,
    }
}

/// The rows an answer referred to: `r3.rows.je4518.debit` is (`r3`, `je4518`).
fn referenced_rows(answer: &ModelAnswer) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for piece in crate::model_answer::pieces(&answer.text) {
        if let crate::model_answer::Piece::Ref(r) = piece {
            let parts: Vec<&str> = r.path.split('.').collect();
            if let [result, "rows", key, ..] = parts.as_slice() {
                out.push(((*result).to_owned(), (*key).to_owned()));
            }
        }
    }
    out
}

/// The source the model named for a question outside the books.
fn nearest_source(env: &Env<'_>, state: &State, nearest: &Nearest) -> Option<AnswerSource> {
    let party: Option<AnswerRecord> = nearest
        .party
        .as_deref()
        .and_then(|p| tools::record(env, state, p, "party").ok())
        .and_then(|r| tools::records::answer_record(state, &format!("{}{}", r.prefix, r.id)));
    let id = nearest.source.as_str();
    if id == "tiles" {
        return Some(AnswerSource::Tiles {
            result_id: String::new(),
        });
    }
    if id == "party_card" {
        return party.map(|party| AnswerSource::PartyCard {
            result_id: String::new(),
            party,
        });
    }
    if let Ok(report) = serde_json::from_value::<ReportId>(Value::String(id.to_owned())) {
        let entry = report_catalog().into_iter().find(|e| e.id == report)?;
        let filters = SourceReportFilters {
            party: party.map(|record| SourceParty {
                record,
                person_key: None,
            }),
            ..SourceReportFilters::default()
        };
        return Some(AnswerSource::Report {
            result_id: String::new(),
            label_id: entry.title_id,
            id: report,
            filters,
        });
    }
    let view = serde_json::from_value::<mandi_reports::analytics::AnalyticsView>(Value::String(
        id.to_owned(),
    ))
    .ok()?;
    let entry = analytics_catalog().into_iter().find(|e| e.id == view)?;
    Some(AnswerSource::Analytics {
        result_id: String::new(),
        label_id: entry.title_id,
        view,
        filters: crate::source::SourceAnalyticsFilters::default(),
    })
}

// ---- the commands ----------------------------------------------------------------------------

fn require_on(d: &Deps<'_>) -> Result<(), ServiceError> {
    if !assistant::config(d.ctx)?.enabled {
        return Err(reject(ErrorCode::ErrAssistantOff));
    }
    require_key(d)
}

fn require_key(d: &Deps<'_>) -> Result<(), ServiceError> {
    let key = d
        .runtime
        .keys()
        .get()
        .map_err(|e| ServiceError::Data(format!("credential store: {}", e.0)))?;
    if key.is_none() {
        return Err(reject(ErrorCode::ErrAssistantKeyMissing));
    }
    Ok(())
}

/// A typed question to the model (`assistant_ask`).
///
/// # Errors
/// `ERR_REQUIRED` / `ERR_TEXT_TOO_LONG` on `text`; `ERR_ASSISTANT_OFF`;
/// `ERR_ASSISTANT_KEY_MISSING`; `ERR_ASSISTANT_BUSY`.
pub fn ask(d: &Deps<'_>, input: &AssistantAskInput) -> Result<AskReply, ServiceError> {
    let text = input.text.trim();
    if text.is_empty() {
        return Err(ServiceError::reject_field(ErrorCode::ErrRequired, "text"));
    }
    if text.chars().count() > MAX_QUESTION_CHARS {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrTextTooLong,
            "text",
        ));
    }
    require_on(d)?;
    let _guard = d
        .runtime
        .try_begin()
        .ok_or_else(|| reject(ErrorCode::ErrAssistantBusy))?;
    let run = Run::new(d)?;
    let id = input
        .conversation_id
        .clone()
        .filter(|c| !c.trim().is_empty())
        .unwrap_or_else(|| d.runtime.new_conversation_id(d.ctx));
    let mut conv = d
        .runtime
        .take(&id)
        .unwrap_or_else(|| Conversation::new(&id));
    conv.waiting = None;
    let mut turn = Turn {
        question: text.to_owned(),
        ..Turn::default()
    };
    if assistant::count_today(d.ctx)? >= DAILY_LIMIT {
        return run.conclude(
            conv,
            turn,
            Finished::plain(AskOutcome::LimitReached),
            Source::Model,
            None,
        );
    }
    let numbers = figures::question_numbers(text);
    turn.messages
        .push(user_message(&run.today_iso, run.language, text, &numbers));
    let finished = run.drive(&mut conv, &mut turn, Vec::new());
    run.conclude(conv, turn, finished, Source::Model, None)
}

/// Goes on with a question that waits (`assistant_pick`).
///
/// # Errors
/// `ERR_NOT_FOUND` on `question_id`; `ERR_ASSISTANT_NOT_WAITING`; `ERR_ASSISTANT_OFF`;
/// `ERR_ASSISTANT_KEY_MISSING`; `ERR_ASSISTANT_BUSY`; `ERR_PARTNER_LOCKED`.
pub fn pick(d: &Deps<'_>, input: &AssistantPickInput) -> Result<AskReply, ServiceError> {
    let row = assistant::get_question(d.ctx, input.question_id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "question_id"))?;
    require_on(d)?;
    let _guard = d
        .runtime
        .try_begin()
        .ok_or_else(|| reject(ErrorCode::ErrAssistantBusy))?;
    let not_waiting =
        || ServiceError::reject_field(ErrorCode::ErrAssistantNotWaiting, "question_id");
    let mut conv = d
        .runtime
        .take(&row.conversation_id)
        .ok_or_else(not_waiting)?;
    let Some(waiting) = conv.waiting.clone().filter(|w| w.question_id == row.id) else {
        d.runtime.keep(conv);
        return Err(not_waiting());
    };
    let run = Run::new(d)?;
    let Some(mut turn) = conv.turns.pop() else {
        return Err(not_waiting());
    };
    let outcome = match (&waiting.wait, &input.choice) {
        (Wait::Which { find_id }, PickChoice::Match { key }) => {
            let at = conv
                .results
                .iter()
                .find(|r| &r.id == find_id)
                .and_then(|r| {
                    match r.node.get(&["matches"]) {
                    Some(crate::results::Node::List(items)) => items.iter().position(|m| {
                        matches!(m.get(&["key"]), Some(crate::results::Node::Str(k)) if k == key)
                    }),
                    _ => None,
                }
                });
            match at {
                Some(i) => {
                    turn.messages.push(json!({
                        "role": "user",
                        "content": json!({"picked": format!("{find_id}.matches.{i}")}).to_string(),
                    }));
                    Ok(Vec::new())
                }
                None => Err(ServiceError::reject_field(ErrorCode::ErrNotFound, "key")),
            }
        }
        (Wait::Partner { pending }, PickChoice::PartnerCode) => {
            if run.partner_locked {
                Err(ServiceError::reject(ErrorCode::ErrPartnerLocked))
            } else {
                Ok(pending.clone())
            }
        }
        _ => Err(not_waiting()),
    };
    let pending = match outcome {
        Ok(p) => p,
        Err(e) => {
            conv.turns.push(turn);
            d.runtime.keep(conv);
            return Err(e);
        }
    };
    conv.waiting = None;
    // A new moment: what this question read before is read again on it, so all of it is one.
    run.refresh(&mut conv, &turn);
    let finished = run.drive(&mut conv, &mut turn, pending);
    run.conclude(conv, turn, finished, Source::Continue, Some(row.id))
}

/// Reads an old answer again on today's books (`assistant_rerun`): the same tool calls, shown as
/// plain tables. No model call; not counted.
///
/// # Errors
/// `ERR_NOT_FOUND` on `question_id`.
pub fn rerun(d: &Deps<'_>, input: &AssistantRerunInput) -> Result<AskReply, ServiceError> {
    let row = assistant::get_question(d.ctx, input.question_id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "question_id"))?;
    let run = Run::new(d)?;
    let calls: Vec<Value> = serde_json::from_str(&row.tool_calls_json).unwrap_or_default();
    let numbers = figures::question_numbers(&row.question);
    let mut state = State {
        results: Vec::new(),
        turn: 0,
    };
    let mut turn = Turn {
        question: row.question.clone(),
        ..Turn::default()
    };
    for call in calls.iter().filter(|c| c.get("error").is_none()) {
        let (Some(tool), Some(input)) =
            (call.get("tool").and_then(Value::as_str), call.get("input"))
        else {
            continue;
        };
        let env = run.env(&numbers);
        if let Ok(draft) = tools::call(&env, &state, tool, &input.to_string()) {
            let id = call
                .get("id")
                .and_then(Value::as_str)
                .map_or_else(|| state.next_id(prefix_of(draft.tool)), str::to_owned);
            let mut result = ToolResult {
                id,
                tool: draft.tool,
                input: draft.input,
                node: draft.node,
                shown: draft.shown,
                source: draft.source,
                partner_parts_locked: draft.partner_parts_locked,
                partner_data: draft.partner_data,
                turn: 0,
                working: draft.working,
            };
            fit(&mut result, &run.words);
            turn.tool_log.push(call.clone());
            state.results.push(result);
        }
    }
    let mut f = Finished::plain(AskOutcome::Answered);
    f.answer = Some(fill::plain_tables(&state, &run.read_at()));
    let conv = Conversation::new(&row.conversation_id);
    run.conclude(conv, turn, f, Source::Rerun, Some(row.id))
}

/// One fixed test question (`assistant_test`): it needs a saved key, not the on switch, counts as a
/// question and writes a log row.
///
/// # Errors
/// `ERR_ASSISTANT_KEY_MISSING`; `ERR_ASSISTANT_BUSY`.
pub fn test_connection(d: &Deps<'_>) -> Result<AssistantTestResult, ServiceError> {
    require_key(d)?;
    let _guard = d
        .runtime
        .try_begin()
        .ok_or_else(|| reject(ErrorCode::ErrAssistantBusy))?;
    let run = Run::new(d)?;
    let started = Instant::now();
    let conv = Conversation::new(&d.runtime.new_conversation_id(d.ctx));
    let mut turn = Turn {
        question: "test".to_owned(),
        ..Turn::default()
    };
    turn.messages
        .push(user_message(&run.today_iso, run.language, "test", &[]));
    let result = run.call_model(&conv, &mut turn, false);
    let millis = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);
    let (finished, failure) = match result {
        Ok(reply) => {
            let mut f = Finished::plain(AskOutcome::Answered);
            f.finish_reason = Some(reply.finish_reason);
            (f, None)
        }
        Err(f) => {
            let failure = f.failure;
            (*f, failure)
        }
    };
    let worked = failure.is_none() && finished.outcome == AskOutcome::Answered;
    let reply = run.conclude(conv, turn, finished, Source::Test, None)?;
    Ok(AssistantTestResult {
        worked,
        millis,
        model: run.model.clone(),
        failure,
        count_today: reply.count_today,
        limit: DAILY_LIMIT,
    })
}

#[cfg(test)]
mod name_tests {
    use super::*;

    #[test]
    fn a_name_argument_has_both_scripts_and_the_tag_after_each() {
        let record = AnswerRecord {
            id: 5,
            code: None,
            name_ur: "عاشق ظفر".to_owned(),
            name_en: "Ashiq Zafar".to_owned(),
            tag: Some("K".to_owned()),
        };
        let arg = names_arg("name", &record);
        assert_eq!(arg.name, "name");
        assert_eq!(
            arg.value,
            ArgValue::Names {
                names: Names {
                    ur: "عاشق ظفر K".to_owned(),
                    en: "Ashiq Zafar K".to_owned(),
                },
            }
        );
        let plain = AnswerRecord {
            tag: None,
            ..record
        };
        let ArgValue::Names { names } = names_arg("name", &plain).value else {
            panic!("names");
        };
        assert_eq!(
            (names.ur.as_str(), names.en.as_str()),
            ("عاشق ظفر", "Ashiq Zafar")
        );
    }
}
