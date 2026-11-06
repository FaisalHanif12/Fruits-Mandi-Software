//! The loop of one question (Tech Stack 4.8, PRD FR-7.72 to FR-7.82, Code Standards Q1 to Q10).
//!
//! `ask` takes a typed question; `pick` goes on with a question that waits ("کون سا؟" or the
//! partner code); `rerun` reads an old answer again on today's books; `test_connection` sends one
//! fixed question to see that the key and the model work. Chips are in `quick.rs`.
//!
//! The loop, for one question: build the request (the fixed instructions, the last 3 questions,
//! this question), call the model, run the tools it asks for (a partner source while the section
//! is locked pauses the question with no model call), call again with the results, and repeat until
//! it gives its answer: at most 6 model calls and 12 tool calls, the last call with the tools
//! switched off. The answer is checked
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
    AskOutcome, AskReply, AskedRole, AssistantAskInput, AssistantFailure, AssistantPickInput,
    AssistantRerunInput, FailureKind, FoundAs, PickChoice, StopReason,
};
use crate::check::{Check, Failure, NameBook, NameSet, Verdict};
use crate::client::{CallError, ModelClient, Reply, ToolCall};
use crate::conversation::{CallTiming, Conversation, Timing, ToolTiming, Turn, Wait, Waiting};
use crate::echo;
use crate::fill;
use crate::hedge::{Attempt, End, Side};
use crate::help_guard;
use crate::limits::{
    DAILY_LIMIT, FIRST_BYTE_SECS, MAX_CALL_OUTPUT_TOKENS, MAX_INPUT_TOKENS, MAX_MODEL_CALLS,
    MAX_OUTPUT_TOKENS, MAX_QUESTION_CHARS, MAX_REWRITES, MAX_TOOL_CALLS, QUESTION_TIMEOUT_SECS,
    RETRY_MIN_LEFT_SECS, TOOLS_OFF_AT_SECS,
};
use crate::model_answer::{AnswerKind, EntryScreenName, ModelAnswer, Nearest};
use crate::prefetch;
use crate::prefind;
use crate::request::{Parts, RESULT_MARK, request_body, user_message};
use crate::results::{Node, ToolResult, Words, fit, tool_result_json};
use crate::route;
use crate::runtime::Runtime;
use crate::scope_guard::{self, AskScope};
use crate::settings::{AssistantTestResult, DEFAULT_MODEL};
use crate::source::{AnswerRecord, SourceParty, SourceReportFilters};
use crate::step::AssistantStep;
use crate::tools::{self, Env, State, Stop, ToolError, cues, typed};

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
    /// For `not_understood` after a limit: which one.
    pub stop_reason: Option<StopReason>,
    /// The answer as the model wrote it, for the log.
    pub model_text: Option<String>,
    pub wait: Option<Wait>,
    pub finish_reason: Option<String>,
    /// For `not_found`: the kind of record that was asked for (plan 52 Section 8).
    pub asked_role: Option<AskedRole>,
    /// For `not_found`: the name as the record of another role.
    pub found_as: Option<FoundAs>,
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
            stop_reason: None,
            model_text: None,
            wait: None,
            finish_reason: None,
            asked_role: None,
            found_as: None,
        }
    }

    /// A limit stopped the question. The reason goes to the log (`finish_reason`) and to the panel
    /// (`stop_reason`); the plain tables of the results that were read, if any, are shown.
    pub(crate) fn stopped(reason: StopReason, tables: Option<Answer>) -> Self {
        Self {
            stop_reason: Some(reason),
            answer: tables,
            finish_reason: Some(reason.word().to_owned()),
            ..Self::plain(AskOutcome::NotUnderstood)
        }
    }

    /// The question ran out of time (or the line dropped late) after the books were read: what
    /// was read is shown as plain tables (plan 55 Section 3). The log keeps the word `time_limit`.
    pub(crate) fn time_limited(tables: Answer) -> Self {
        Self {
            answer: Some(tables),
            finish_reason: Some("time_limit".to_owned()),
            ..Self::plain(AskOutcome::TimeLimit)
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
        AskOutcome::TimeLimit => "time_limit",
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
    /// Every account and partner name, built the first time an answer is checked (P52-S1).
    book: std::cell::OnceCell<std::sync::Arc<NameBook>>,
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
            book: std::cell::OnceCell::new(),
        })
    }

    /// Every account and partner name of the books, for the name check of an answer. Built from
    /// the search index in memory the first time (the index is warm: NFR-SPD-6) and kept for the
    /// rest of the question. When the names cannot be read the book is empty, so the check of the
    /// results' own names stands alone, as before.
    fn name_book(&self, exempt: &[String]) -> std::sync::Arc<NameBook> {
        self.book
            .get_or_init(|| {
                let names = assistant::name_book(self.d.ctx).unwrap_or_default();
                std::sync::Arc::new(NameBook::new(&names, exempt))
            })
            .clone()
    }

    pub(crate) fn env<'e>(&'e self, numbers: &'e [QuestionNumber], question: &'e str) -> Env<'e> {
        Env {
            ctx: &self.snap,
            actor: self.d.actor,
            words: &self.words,
            language: self.language,
            today: self.today,
            partner_locked: self.partner_locked,
            numbers,
            question,
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
        let question = turn.question.clone();
        let mut state = State {
            results: std::mem::take(&mut conv.results),
            turn: index,
        };
        let mut paused = None;
        // The finds made in this reply: the ones a later call of it may read the match of.
        let mut batch_finds: Vec<String> = Vec::new();
        // True while every call of the reply is a `find`: only then does the app read ahead.
        let mut only_finds = true;
        let mut iter = calls.into_iter();
        while let Some(call) = iter.next() {
            only_finds &= call.name == "find";
            if turn.counters.tool_calls >= MAX_TOOL_CALLS {
                turn.messages.push(tool_message(
                    &call.id,
                    ToolError::new(
                        "ERR_TOO_MANY_CALLS",
                        "tool",
                        &format!("{MAX_TOOL_CALLS} tool calls are the most"),
                    )
                    .to_json()
                    .to_string(),
                    None,
                ));
                continue;
            }
            // A call that reads a match of a find made in this same reply is run only when that
            // find had exactly one match; otherwise the Owner is asked at once (plan 52 item 8).
            if let Some(find_id) =
                unclear_find_read(&state, &batch_finds, &turn.unclear, &call.arguments)
            {
                turn.messages.push(tool_message(
                    &call.id,
                    ToolError::new(
                        "ERR_NOT_ONE_MATCH",
                        "party",
                        &format!(
                            "{find_id} did not match exactly one record, so this was not read; the \
                             Owner is asked which one"
                        ),
                    )
                    .to_json()
                    .to_string(),
                    None,
                ));
                turn.ask_which.get_or_insert(find_id);
                continue;
            }
            // The very read the app made ahead: the model gets that result back, nothing is read
            // twice and no new number is spent (plan 52 P52-S1).
            if let Some(read_id) =
                prefetch::already_read(&turn.prefetched, &call.name, &call.arguments)
                && let Some(result) = state.find(&read_id)
            {
                let content = tool_result_json(result, &self.words).to_string();
                if let Some(last) = turn.timing.calls.last_mut() {
                    last.tools.push(ToolTiming {
                        name: call.name.clone(),
                        millis: 0,
                        ok: true,
                        prefetch: false,
                    });
                }
                turn.counters.tool_calls = turn.counters.tool_calls.saturating_add(1);
                turn.messages
                    .push(tool_message(&call.id, content, Some(&read_id)));
                continue;
            }
            let env = self.env(&numbers, &question);
            // The step line goes out as the tool starts, so a slow report shows what it is doing.
            (self.d.emit)(step_of(&call, &env, &state));
            let began = Instant::now();
            // A typed name in a filter is looked up for the model (plan 52 item 6).
            let mut looked_up = Vec::new();
            let mut notes = Vec::new();
            let mut asked = None;
            let resolved = typed::resolve(
                &env,
                &mut state,
                index,
                &call.name,
                &call.arguments,
                &mut looked_up,
                &mut asked,
            );
            turn.tool_log.extend(looked_up);
            if let Some(find_id) = asked {
                turn.unclear.push(find_id.clone());
                turn.ask_which.get_or_insert(find_id);
            }
            let outcome = match resolved {
                Ok(r) => {
                    notes = r.notes;
                    tools::call(&env, &state, &call.name, &r.arguments)
                }
                Err(stop) => Err(stop),
            };
            let millis = millis_since(began);
            let timed = |ok: bool| ToolTiming {
                name: call.name.clone(),
                millis,
                ok,
                prefetch: false,
            };
            match outcome {
                Ok(mut draft) => {
                    if let Some(last) = turn.timing.calls.last_mut() {
                        last.tools.push(timed(true));
                    }
                    turn.counters.tool_calls = turn.counters.tool_calls.saturating_add(1);
                    if !notes.is_empty() {
                        typed::add_note(&mut draft.node, &notes.join("; "));
                    }
                    // What this report holds and where the nearby figure is (plan 52 P52-S1).
                    if let Some(note) = route::scope_note(&call.name, &call.arguments) {
                        typed::add_note(&mut draft.node, note);
                    }
                    let id = state.next_id(prefix_of(draft.tool));
                    let mut result = tools::stored(draft, id.clone(), index);
                    fit(&mut result, &self.words);
                    if result.tool == "find" {
                        batch_finds.push(id.clone());
                    }
                    let content = tool_result_json(&result, &self.words).to_string();
                    turn.tool_log.push(json!({
                        "id": id,
                        "tool": result.tool,
                        "input": result.input,
                    }));
                    state.results.push(result);
                    turn.messages
                        .push(tool_message(&call.id, content, Some(&id)));
                    // The only parties found are of the other role than the question's verb gives
                    // ("Faisal ne kya becha" and he is a buyer): the question will end as not found
                    // with him as `found_as`, and nothing of his is read as the other role's.
                    if turn.wrong_role.is_none()
                        && batch_finds.last() == Some(&id)
                        && let Some(role) = route::wrong_role_found(&question, &state, &id)
                    {
                        turn.wrong_role = Some((id.clone(), role));
                    }
                }
                Err(Stop::Error(error)) => {
                    if let Some(last) = turn.timing.calls.last_mut() {
                        last.tools.push(timed(false));
                    }
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
        // The model sent only finds and they matched one party each: read his card (and his
        // ledger when the question names one) now, so the next call can answer (P52-S1).
        if paused.is_none()
            && only_finds
            && turn.ask_which.is_none()
            && turn.wrong_role.is_none()
            && !batch_finds.is_empty()
        {
            self.prefetch(turn, &mut state, index, &numbers, &question, &batch_finds);
        }
        conv.results = state.results;
        paused
    }

    /// Reads what a find with exactly one party match leads to, as the model would have
    /// (`party_card`, and R13, R2 or a commission view when the question names one, see
    /// `prefetch`). The results get their own ids
    /// (`p1` ...), the question log marks them, and they count as no tool call of the model. They
    /// are added to the conversation as a call of the assistant and its tool messages, after the
    /// ones of the find.
    fn prefetch(
        &self,
        turn: &mut Turn,
        state: &mut State,
        index: u32,
        numbers: &[QuestionNumber],
        question: &str,
        batch_finds: &[String],
    ) {
        let mut calls: Vec<Value> = Vec::new();
        let mut answers: Vec<Value> = Vec::new();
        for find_id in batch_finds.iter().take(prefetch::MAX_FINDS) {
            let Some(party) = prefetch::single_party_match(state, find_id) else {
                continue;
            };
            for (name, arguments) in prefetch::reads(question, &party) {
                let call = ToolCall {
                    id: String::new(),
                    name: name.to_owned(),
                    arguments,
                };
                let env = self.env(numbers, question);
                (self.d.emit)(step_of(&call, &env, state));
                let began = Instant::now();
                let outcome = tools::call(&env, state, &call.name, &call.arguments);
                let millis = millis_since(began);
                let Ok(draft) = outcome else {
                    // A read the app could not do is the model's to ask for; nothing is said.
                    continue;
                };
                if let Some(last) = turn.timing.calls.last_mut() {
                    last.tools.push(ToolTiming {
                        name: call.name.clone(),
                        millis,
                        ok: true,
                        prefetch: true,
                    });
                }
                // The normal numbers, as if the model had asked in its next reply: the example of
                // the prompt (find r1, card r2, ledger r3) then holds.
                let id = state.next_id(prefix_of(draft.tool));
                let mut result = tools::stored(draft, id.clone(), index);
                fit(&mut result, &self.words);
                let content = tool_result_json(&result, &self.words).to_string();
                turn.tool_log.push(json!({
                    "id": id,
                    "tool": result.tool,
                    "input": result.input,
                    "prefetch": true,
                }));
                state.results.push(result);
                if let Some(input) = prefetch::canonical(&call.name, &call.arguments) {
                    turn.prefetched.push((call.name.clone(), input, id.clone()));
                }
                let call_id = format!("pre_p{}", turn.prefetched.len());
                calls.push(json!({"id": call_id, "type": "function",
                                  "function": {"name": call.name, "arguments": call.arguments}}));
                answers.push(tool_message(&call_id, content, Some(&id)));
            }
        }
        if !calls.is_empty() {
            turn.messages
                .push(json!({"role": "assistant", "content": null, "tool_calls": calls}));
            turn.messages.extend(answers);
        }
    }

    /// The results this question read as plain tables, or none when it read no table.
    fn tables_read(&self, conv: &Conversation) -> Option<Answer> {
        let state = State {
            results: conv.results.clone(),
            turn: u32::try_from(conv.turns.len()).unwrap_or(u32::MAX),
        };
        let answer = fill::plain_tables(&state, &self.read_at());
        (!answer.tables.is_empty()).then_some(answer)
    }

    /// What the app understood of the question, when it understood the whole of it (plan 55
    /// Section 2). The progress line "understood" says of what kind.
    ///
    /// In a follow-up (`follow_up`), a question that leans on the earlier ones is left to the model,
    /// which sees them (money review R-c): see [`leans_on_earlier_questions`].
    fn understand_question(
        &self,
        text: &str,
        follow_up: bool,
    ) -> Option<crate::understand::Understanding> {
        use crate::understand::{self, Intent};
        let reader = understand::Reader {
            ctx: &self.snap,
            actor: self.d.actor,
            language: self.language,
            today: self.today,
            partner_locked: self.partner_locked,
        };
        let understood = understand::understand(&reader, text);
        if !understood.sure || (follow_up && leans_on_earlier_questions(&understood)) {
            return None;
        }
        let kind = match understood.intent.value {
            Intent::Print => "print",
            Intent::HowTo | Intent::Navigate => "help",
            Intent::List => "list",
            Intent::Advice | Intent::Forecast | Intent::WhatIf => "advice",
            _ => "figure",
        };
        (self.d.emit)(AssistantStep {
            id: "assistant-step-understood".to_owned(),
            args: vec![MessageArg {
                name: "kind".to_owned(),
                value: ArgValue::Text {
                    text: kind.to_owned(),
                },
            }],
        });
        Some(understood)
    }

    /// Reads, before the first call, what the model would have chosen to read (plan 54 S3, plan 55
    /// Section 2). True when something was read. The reads are the tools themselves, with the
    /// filters `understand::plan_reads` gives; a read the tool refuses is dropped, and the model
    /// carries on from what was read.
    fn read_first(
        &self,
        conv: &mut Conversation,
        turn: &mut Turn,
        understood: &crate::understand::Understanding,
    ) -> bool {
        let Some(reads) = crate::understand::plan_reads(understood) else {
            return false;
        };
        let mut read_any = false;
        for read in reads {
            let done = self.preread_tool(conv, turn, read.tool, &read.input.to_string());
            read_any |= done.is_some();
        }
        read_any
    }

    /// The views and reports the app planned for this question, read now or found already read: their
    /// result ids, for the scope guard to name to the model.
    fn read_planned(&self, conv: &mut Conversation, turn: &mut Turn) -> Vec<String> {
        let mut ids = Vec::new();
        for (tool, input) in turn.planned.clone() {
            let arguments = input.to_string();
            let id = prefetch::already_read(&turn.prefetched, &tool, &arguments)
                .or_else(|| self.preread_tool(conv, turn, &tool, &arguments));
            if let Some(id) = id
                && (tool == "analytics" || tool == "report")
            {
                ids.push(id);
            }
        }
        ids
    }

    /// An answer the app writes by itself, with no model call (plan 54 S2, plan 55 Section 2):
    /// only when the app understood the whole question and the answer is already written in the
    /// app. The one door is `understand::compose` (backend2's composers): a print request opens its
    /// report with the party and the dates filled and gives the steps, a how-to answers from the
    /// guide, a screen opens, a question outside the books gets its words, and a name that no
    /// record has, or that only a record of the other role has ("Faisal ne kya becha" and he is a
    /// buyer), ends as not found with the kind of record asked for. Anything else, and anything the
    /// app is not sure of, goes to the model as before. The log keeps `local` as its finish reason.
    ///
    /// Only one thing is added here: for a name found in the other role, the `find` runs as the
    /// model would send it, so "show him as a buyer" is one click (it needs the options and the
    /// wait of the record that was found, which a pure composer cannot read).
    fn local_answer(
        &self,
        conv: &mut Conversation,
        turn: &mut Turn,
        understood: &crate::understand::Understanding,
    ) -> Option<Finished> {
        use crate::understand::{Outcome, PartyRole};
        let read_at = self.read_at();
        let composed =
            crate::understand::compose(understood, self.language, &self.words, &read_at)?;
        // The question log shows what the answer used (the guide topic, the report and its
        // filters), marked `local`: no tool was run for it.
        let logged = local_log(understood, &composed);
        turn.tool_log.extend(logged);
        let local = |mut finished: Finished| {
            finished.finish_reason = Some("local".to_owned());
            Some(finished)
        };
        if composed.outcome == AskOutcome::NotFound && composed.found_as.is_some() {
            let party = understood
                .parties
                .iter()
                .find(|p| matches!(p.outcome, Outcome::FoundAs(_)))?;
            let asked = composed.asked_role.unwrap_or(AskedRole::Any);
            let word = match party.asked_role {
                Some(PartyRole::Seller) => "seller",
                Some(PartyRole::Buyer) => "buyer",
                _ => return local(Self::finished_of(composed)),
            };
            let input = json!({"text": party.text, "role": word});
            let id = self.preread_tool(conv, turn, "find", &input.to_string())?;
            // The model has not been asked yet. When the Owner clicks the name (a pick), the model
            // goes on from these messages, so they must start with his question (money review
            // BUG-6). The find wrote nothing before it, so the question goes first.
            let question = turn.question.clone();
            let numbers = figures::question_numbers(&question);
            turn.messages.insert(
                0,
                user_message(&self.today_iso, self.language, &question, &numbers),
            );
            return local(self.not_found_in_other_role(conv, &id, asked));
        }
        local(Self::finished_of(composed))
    }

    /// The ending of a question from what a local composer made.
    fn finished_of(composed: crate::understand::Composed) -> Finished {
        let mut finished = Finished::plain(composed.outcome);
        finished.answer = composed.answer;
        finished.asked_role = composed.asked_role;
        finished.found_as = composed.found_as;
        finished.entry_route = composed.entry_route;
        finished
    }

    /// The instant answer of a simple figure (PRD FR-7.90, plan 55 Section 10): the planned reads
    /// are run, `understand::compose_figure` writes the answer from them in the app's own reviewed
    /// sentences (the figures are references into the reads), and the same checks as for the
    /// model's answer run on it (FR-7.76: the figure in its two places, the side words, the words
    /// around each reference). Nothing is added or rounded here.
    ///
    /// `None` when the question has no planned read, when the reads gave nothing, when no sentence
    /// can be written, or when a check refuses the answer. After the reads, `turn.read_first` is
    /// true, so the model path does not read again. Each read is logged with `"local": true` and
    /// still replays on a re-read; the question is logged as local, with no model call.
    fn instant_answer(
        &self,
        conv: &mut Conversation,
        turn: &mut Turn,
        understood: &crate::understand::Understanding,
    ) -> Option<Finished> {
        let start = turn.tool_log.len();
        if !self.read_first(conv, turn, understood) {
            return None;
        }
        turn.read_first = true;
        let index = u32::try_from(conv.turns.len()).unwrap_or(u32::MAX);
        let read: Vec<ToolResult> = conv
            .results
            .iter()
            .filter(|r| r.turn == index)
            .cloned()
            .collect();
        let figure =
            crate::understand::compose_figure(understood, &read, self.language, &self.words)?;
        let numbers = figures::question_numbers(&turn.question);
        let question = turn.question.clone();
        let env = self.env(&numbers, &question);
        let state = State {
            results: conv.results.clone(),
            turn: index,
        };
        let exempt = assistant::exempt_names(&self.snap).unwrap_or_default();
        let names = NameSet::with_book(&state, &exempt, self.name_book(&exempt));
        let verdict = Check {
            env: &env,
            state: &state,
            names: &names,
        }
        .run(&figure.answer);
        if !matches!(verdict, Verdict::Ok) {
            return None;
        }
        for entry in turn.tool_log.iter_mut().skip(start) {
            if let Some(map) = entry.as_object_mut() {
                map.insert("local".to_owned(), json!(true));
            }
        }
        let content = json!({
            "kind": "answer", "text": figure.answer.text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        })
        .to_string();
        turn.messages
            .push(json!({"role": "assistant", "content": content}));
        turn.referenced = referenced_rows(&figure.answer);
        let mut f = self.conclude_answer(&env, &state, &figure.answer, &self.read_at());
        f.model_text = Some(json_text(&content));
        f.finish_reason = Some("local".to_owned());
        Some(f)
    }

    /// What was read for this question, to show when time runs out (plan 55 Section 3): the
    /// tables, and a party card's name and checked balance. `None` if nothing that shows was read.
    fn read_so_far(&self, conv: &Conversation, question: &str) -> Option<Answer> {
        let state = State {
            results: conv.results.clone(),
            turn: u32::try_from(conv.turns.len()).unwrap_or(u32::MAX),
        };
        fill::read_so_far(&state, &self.read_at(), self.card_field(question))
    }

    /// The field of a party card that holds the figure the question asks for, so that a slow line
    /// never shows his balance for a question about his advance (money review R-b). It reads the
    /// question again with the same understanding, only when time has run out, so it costs
    /// nothing on the way to an answer. `None` when the figure is not one a card holds.
    fn card_field(&self, question: &str) -> Option<&'static str> {
        use crate::understand::{self, FigureKind};
        let reader = understand::Reader {
            ctx: &self.snap,
            actor: self.d.actor,
            language: self.language,
            today: self.today,
            partner_locked: self.partner_locked,
        };
        match understand::understand(&reader, question).figure()? {
            FigureKind::Balance | FigureKind::Dues => Some(fill::CARD_BALANCE),
            FigureKind::Advance => Some(fill::CARD_ADVANCE),
            FigureKind::Payable => Some(fill::CARD_PAYABLE),
            _ => None,
        }
    }

    /// How the question ends when it has no time left, or when a call failed late (plan 55
    /// Section 3): with what was read, shown as plain tables (`time_limit`), or, when nothing was
    /// read, with the failure itself. Only a slow or dropped line is a matter of time: a wrong
    /// key, no credit or no internet end as they always did.
    fn out_of_time(&self, conv: &Conversation, turn: &Turn, error: CallError) -> Box<Finished> {
        let slow = matches!(
            error.kind,
            FailureKind::Timeout | FailureKind::Dropped | FailureKind::Busy | FailureKind::Server
        );
        match self.read_so_far(conv, &turn.question) {
            Some(read) if slow => Box::new(Finished::time_limited(read)),
            _ => Box::new(Finished::failed(error)),
        }
    }

    /// One model call, within the limits of the question (plan 54 S5, plan 55 Section 3). The
    /// reply of the model comes whole, so a try is cut when no reply has come in
    /// [`FIRST_BYTE_SECS`]. A call that failed on the network, on 429, on a 5xx or by that cut is
    /// tried once more after a short wait, when enough of the question is left (plan 52 item 6).
    /// The whole question has [`QUESTION_TIMEOUT_SECS`]: from [`TOOLS_OFF_AT_SECS`] the call goes
    /// out with the tools off, and at the limit the question ends with what was read. The failed
    /// try is in the timing, and only the reply that came is counted.
    fn call_model(
        &self,
        conv: &Conversation,
        turn: &mut Turn,
        rewrite: bool,
    ) -> Result<Reply, Box<Finished>> {
        let total = Duration::from_secs(QUESTION_TIMEOUT_SECS);
        if self.started.elapsed() >= total {
            return Err(self.out_of_time(conv, turn, CallError::new(FailureKind::Timeout)));
        }
        let c = &turn.counters;
        let over_calls = !rewrite && c.model_calls >= MAX_MODEL_CALLS;
        let over_rewrite = rewrite && c.rewrites >= MAX_REWRITES;
        let out_used = u32::try_from(c.tokens_out).unwrap_or(u32::MAX);
        let in_used = u32::try_from(c.tokens_in).unwrap_or(u32::MAX);
        if over_calls
            || over_rewrite
            || out_used >= MAX_OUTPUT_TOKENS
            || in_used >= MAX_INPUT_TOKENS
        {
            // Not "I did not understand": the books could not be read within the limits. What was
            // read before the stop is shown as plain tables (plan 51 B2).
            let reason = if over_calls || over_rewrite {
                StopReason::Calls
            } else {
                StopReason::Tokens
            };
            return Err(Box::new(Finished::stopped(reason, self.tables_read(conv))));
        }
        let Ok(Some(key)) = self.d.runtime.keys().get() else {
            return Err(Box::new(Finished::failed(CallError::new(
                FailureKind::BadKey,
            ))));
        };
        let max_completion_tokens = MAX_OUTPUT_TOKENS
            .saturating_sub(out_used)
            .min(MAX_CALL_OUTPUT_TOKENS);
        let mut retried = false;
        let reply = loop {
            let elapsed = self.started.elapsed();
            let left = total.saturating_sub(elapsed);
            if left.is_zero() {
                return Err(self.out_of_time(conv, turn, CallError::new(FailureKind::Timeout)));
            }
            // From 20 s the call must answer from what has been read.
            let late = elapsed >= Duration::from_secs(TOOLS_OFF_AT_SECS);
            let tools_already_off = tools_off(turn, rewrite);
            if late && !tools_already_off {
                (self.d.emit)(step("assistant-step-tools-off"));
            }
            let body = request_body(&Parts {
                model: &self.model,
                conversation: conv,
                current: turn,
                words: &self.words,
                partner_locked: self.partner_locked,
                max_completion_tokens,
                tools_off: tools_already_off || late,
                low_effort: turn.read_first && self.d.runtime.low_effort(),
            });
            // The first try waits FIRST_BYTE_SECS; the one retry has all the time that is left.
            let first_try = Duration::from_secs(FIRST_BYTE_SECS).min(left);
            let timeout = if retried { left } else { first_try };
            let sent = Instant::now();
            let (result, hedge) = self.d.client.call_noted(&key, &body, timeout);
            let model_millis = millis_since(sent);
            match result {
                Ok(reply) => {
                    // A call that was sent twice: the request that lost goes to the log first, and
                    // the one whose reply was used last (the tools of the call attach to it).
                    let winner = hedge.as_ref().and_then(|h| {
                        h.attempts
                            .iter()
                            .find(|a| a.end == End::Won)
                            .map(|a| side_word(a.side))
                    });
                    if let Some(h) = &hedge {
                        for lost in h.attempts.iter().filter(|a| a.end != End::Won) {
                            turn.timing.calls.push(hedge_entry(lost, rewrite, true));
                        }
                    }
                    turn.timing.calls.push(CallTiming {
                        model_millis,
                        tokens_in: reply.usage.prompt_tokens,
                        tokens_cached: reply.usage.cached_tokens,
                        tokens_out: reply.usage.completion_tokens,
                        tokens_reasoning: reply.usage.reasoning_tokens,
                        rewrite,
                        failed: false,
                        tools: Vec::new(),
                        error: None,
                        check_failed: Vec::new(),
                        repaired: Vec::new(),
                        check_paths: Vec::new(),
                        check_at: Vec::new(),
                        hedged: winner,
                        hedge_extra: false,
                    });
                    break reply;
                }
                Err(e) => {
                    // The time of a call that failed is the most useful one: a timeout shows here.
                    // When the copy was sent too, the first request carries the time the question
                    // waited and the copy is a second entry (left out of the sums).
                    turn.timing.calls.push(CallTiming {
                        model_millis,
                        tokens_in: 0,
                        tokens_cached: 0,
                        tokens_out: 0,
                        tokens_reasoning: 0,
                        rewrite,
                        failed: true,
                        tools: Vec::new(),
                        error: failure_word(e.kind),
                        check_failed: Vec::new(),
                        repaired: Vec::new(),
                        check_paths: Vec::new(),
                        check_at: Vec::new(),
                        hedged: hedge.as_ref().map(|_| "first"),
                        hedge_extra: false,
                    });
                    if let Some(h) = &hedge {
                        for copy in h.attempts.iter().filter(|a| a.side == Side::Copy) {
                            turn.timing.calls.push(hedge_entry(copy, rewrite, true));
                        }
                    }
                    let left = total.saturating_sub(self.started.elapsed());
                    if !retried && e.retryable() && left >= Duration::from_secs(RETRY_MIN_LEFT_SECS)
                    {
                        retried = true;
                        (self.d.emit)(step("assistant-step-retrying"));
                        std::thread::sleep(self.d.client.retry_wait());
                        continue;
                    }
                    return Err(self.out_of_time(conv, turn, e));
                }
            }
        };
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
        if !pending.is_empty() {
            let log_start = turn.tool_log.len();
            if let Some(rest) = self.run_calls(conv, turn, pending) {
                return paused(rest);
            }
            if let Some((find_id, asked)) = turn.wrong_role.take() {
                return self.not_found_in_other_role(conv, &find_id, asked);
            }
            if let Some(find_id) = turn.ask_which.take()
                && !read_data_since(turn, log_start)
            {
                return self.ask_which_one(conv, &find_id);
            }
        }
        loop {
            // The books were read before the first call: this call only writes the answer, and
            // the panel says so (plan 55 Section 2).
            if turn.read_first && turn.counters.model_calls == 0 {
                (self.d.emit)(step("assistant-step-writing"));
            }
            let off = tools_off(turn, false);
            let reply = match self.call_model(conv, turn, false) {
                Ok(r) => r,
                Err(f) => return *f,
            };
            if !matches!(reply.finish_reason.as_str(), "stop" | "tool_calls") {
                // `length` and anything else: the reply cannot be used. What was read before is
                // shown as plain tables (PRD E96); with nothing read it is "not understood".
                return match self.tables_read(conv) {
                    Some(tables) if !conv.results.is_empty() => {
                        let mut f = Finished::stopped(StopReason::Tokens, Some(tables));
                        f.finish_reason = Some(reply.finish_reason);
                        f
                    }
                    _ => {
                        let mut f = Finished::plain(AskOutcome::NotUnderstood);
                        f.finish_reason = Some(reply.finish_reason);
                        f
                    }
                };
            }
            if reply.tool_calls.is_empty() {
                (self.d.emit)(step("assistant-step-checking"));
                return self.finish(conv, turn, reply);
            }
            if off {
                // The tools were switched off for this call and the model asked for one anyway:
                // nothing more may be read, so the question stops with what it has (PRD E96).
                return Finished::stopped(StopReason::Calls, self.tables_read(conv));
            }
            turn.messages.push(reply.message.clone());
            let log_start = turn.tool_log.len();
            if let Some(rest) = self.run_calls(conv, turn, reply.tool_calls) {
                return paused(rest);
            }
            if let Some((find_id, asked)) = turn.wrong_role.take() {
                return self.not_found_in_other_role(conv, &find_id, asked);
            }
            // A name that matched none or several ends the question with "which one" at once, with
            // no model call. Not when the same reply also read the books for another name: the
            // model asked for two things (two items in one question), and it must see both
            // results to answer them (Q111 of the live run 2).
            if let Some(find_id) = turn.ask_which.take()
                && !read_data_since(turn, log_start)
            {
                return self.ask_which_one(conv, &find_id);
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
        // A local entry that has an id is a read of an instant answer: it is read again. One
        // without an id only says what a local answer used, and nothing is run for it.
        for entry in turn
            .tool_log
            .iter()
            .filter(|e| e.get("local").is_none() || e.get("id").is_some())
        {
            let (Some(id), Some(tool), Some(input)) = (
                entry.get("id").and_then(Value::as_str),
                entry.get("tool").and_then(Value::as_str),
                entry.get("input"),
            ) else {
                continue;
            };
            let env = self.env(&numbers, &turn.question);
            if let Ok(draft) = tools::call(&env, &state, tool, &input.to_string()) {
                let mut result = tools::stored(draft, id.to_owned(), index);
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
        let mut repaired = false;
        let mut help_repaired = false;
        let mut echoed = false;
        let mut digits_echoed = false;
        let mut refs_fixed = false;
        // What the model wrote before the app put a name back: the log keeps the model's own words
        // (the call marks `repaired`), so the next live run shows what it typed.
        let mut written_by_model: Option<String> = None;
        loop {
            let Ok(written) = ModelAnswer::parse(&content) else {
                let mut f = Finished::plain(AskOutcome::NotUnderstood);
                f.model_text = Some(json_text(&content));
                f.finish_reason = Some(finish_reason);
                return f;
            };
            // Help is never a change request (plan 52 item 4): a "how do I" question that the
            // model ended as a change request is answered with the guide of its topic.
            let helped = self.help_instead(conv, turn, &written);
            let swapped = helped.is_some();
            let mut parsed = helped.unwrap_or_else(|| written.clone());
            // A general advice question answered as advice that reads nothing is general advice.
            if turn.general_advice
                && parsed.kind == AnswerKind::Advice
                && parsed.show.is_empty()
                && !parsed.text.contains("{{")
            {
                parsed.kind = AnswerKind::GeneralAdvice;
            }
            let numbers = figures::question_numbers(&turn.question);
            let state = State {
                results: conv.results.clone(),
                turn: u32::try_from(conv.turns.len()).unwrap_or(u32::MAX),
            };
            let question = turn.question.clone();
            let env = self.env(&numbers, &question);
            let exempt = assistant::exempt_names(&self.snap).unwrap_or_default();
            let names = NameSet::with_book(&state, &exempt, self.name_book(&exempt));
            let verdict = Check {
                env: &env,
                state: &state,
                names: &names,
            }
            .run(&parsed);
            // An answer must rest on views and reports with the scope the question named (live run
            // 2, CRITICAL): the period, the item, the group, the party of the planned read.
            let verdict = if matches!(verdict, Verdict::Ok) && !swapped {
                let mut wrong = match AskScope::of_planned(&turn.planned) {
                    Some(scope) => scope_guard::failures(&scope, &state, &parsed, &[]),
                    None => Vec::new(),
                };
                // And an answer of the guide alone, when a figure was asked and read (G3).
                wrong.extend(scope_guard::guide_only(
                    &state,
                    &parsed,
                    help_guard::is_how_to(&turn.question, &numbers),
                ));
                if wrong.is_empty() {
                    verdict
                } else {
                    Verdict::Retry(wrong)
                }
            } else {
                verdict
            };
            let read_at = self.read_at();
            let mut f = match verdict {
                // The guide text is the app's own; if it somehow fails a check, the change
                // refused answer stands, as before.
                Verdict::Retry(_) | Verdict::Differ(_) if swapped => {
                    self.conclude_answer(&env, &state, &written, &read_at)
                }
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
                    // A name typed in the words that the results hold exactly is put back as a
                    // reference by the app, with every rule checked again, instead of a rewrite
                    // call of 3 to 6 seconds (plan 52 P52-S1).
                    if !repaired
                        && failures.iter().any(|x| x.rule == "name")
                        && let Some(fixed) = repaired_content(&content, &state, &names)
                    {
                        repaired = true;
                        if let Some(last) = turn.timing.calls.last_mut() {
                            last.repaired.push("name".to_owned());
                        }
                        written_by_model.get_or_insert_with(|| content.clone());
                        content = fixed;
                        continue;
                    }
                    // The Owner's own number written in words ("چھ مہینوں" for his "6 mahine") is his
                    // number: put back as `{{q.nK}}`, then everything is checked again.
                    if !echoed
                        && failures.iter().any(|x| x.rule == "number_word")
                        && let Some(fixed) = echoed_content(&content, &numbers)
                    {
                        echoed = true;
                        if let Some(last) = turn.timing.calls.last_mut() {
                            last.repaired.push("number".to_owned());
                        }
                        written_by_model.get_or_insert_with(|| content.clone());
                        content = fixed;
                        continue;
                    }
                    // The Owner's own number written in digits ("3 مہینوں" for his "3 mahine") is his
                    // number too: put back as `{{q.nK}}` (hidden round 4).
                    if !digits_echoed
                        && failures.iter().any(|x| x.rule == "digit")
                        && let Some(fixed) =
                            map_texts(&content, &|text| echo::put_digits_back(text, &numbers))
                    {
                        digits_echoed = true;
                        if let Some(last) = turn.timing.calls.last_mut() {
                            last.repaired.push("digit".to_owned());
                        }
                        written_by_model.get_or_insert_with(|| content.clone());
                        content = fixed;
                        continue;
                    }
                    // A reference the model mistyped where the right one is certain: `n1` for
                    // `q.n1`, and `rows.0` for the one row of a result (hidden round 4, H363).
                    if !refs_fixed
                        && failures.iter().any(|x| x.rule == "reference")
                        && let Some(fixed) = map_texts(&content, &|text| {
                            echo::fix_references(text, &state, numbers.len())
                        })
                    {
                        refs_fixed = true;
                        if let Some(last) = turn.timing.calls.last_mut() {
                            last.repaired.push("reference".to_owned());
                        }
                        written_by_model.get_or_insert_with(|| content.clone());
                        content = fixed;
                        continue;
                    }
                    // A help answer that retyped the guide in its own words and broke the words rules
                    // (Latin key names, number words, a digit, too long) is written from the guide's
                    // own references instead of a rewrite call (plan 52 P52-S1).
                    if !help_repaired
                        && let Some(fixed) =
                            help_repaired_content(&content, &state, &parsed, &failures)
                    {
                        help_repaired = true;
                        if let Some(last) = turn.timing.calls.last_mut() {
                            last.repaired.push("help".to_owned());
                        }
                        written_by_model.get_or_insert_with(|| content.clone());
                        content = fixed;
                        continue;
                    }
                    if rewrote {
                        let mut f = Finished::plain(AskOutcome::ChecksFailed);
                        f.answer = Some(fill::plain_tables(&state, &read_at));
                        f
                    } else {
                        rewrote = true;
                        (self.d.emit)(step("assistant-step-rewriting"));
                        turn.messages
                            .push(json!({"role": "assistant", "content": content}));
                        // A view or report with another scope than the question's: the app reads
                        // the planned ones (or finds them already read) and names them, so the one
                        // rewrite can cite the right results (the rewrite has no tools).
                        let mut failures = failures;
                        if failures.iter().any(|x| x.rule == "scope") {
                            let right = self.read_planned(conv, turn);
                            for failure in failures.iter_mut().filter(|x| x.rule == "scope") {
                                failure.instead = right.clone();
                            }
                        }
                        let list: Vec<Value> = failures.iter().map(|x| x.to_json()).collect();
                        turn.messages.push(json!({
                            "role": "user",
                            "content": json!({"check_failed": list}).to_string(),
                        }));
                        let rewritten = self.call_model(conv, turn, true);
                        if let Some(last) = turn.timing.calls.last_mut() {
                            last.check_failed =
                                failures.iter().map(|x| x.rule.to_owned()).collect();
                            last.check_paths = failures
                                .iter()
                                .filter(|x| matches!(x.rule, "reference" | "earlier_question"))
                                .map(|x| x.at.clone())
                                .collect();
                            last.check_at = failures
                                .iter()
                                .filter(|x| {
                                    matches!(
                                        x.rule,
                                        "number_word"
                                            | "script"
                                            | "never_word"
                                            | "month"
                                            | "side_words"
                                            | "show"
                                            | "choose"
                                            | "next"
                                            | "too_long"
                                    )
                                })
                                .map(|x| format!("{}:{}", x.rule, x.at))
                                .collect();
                        }
                        match rewritten {
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
            // The party the app found is always named in an answer about him.
            if matches!(
                f.outcome,
                AskOutcome::Answered | AskOutcome::Estimate | AskOutcome::Advice
            ) && let (Some(answer), Some(found)) = (f.answer.as_mut(), turn.prefound.as_ref())
            {
                prefind::ensure_name(answer, found, &prefind::cited_ids(&content));
            }
            f.model_text = Some(json_text(written_by_model.as_deref().unwrap_or(&content)));
            f.finish_reason = Some(finish_reason);
            return f;
        }
    }

    /// Ends the question with "کون سا؟" (several or a close spelling) or "not found" (none) for
    /// this find, with no model call (plan 52 item 8). The Owner's choice goes on as after the
    /// model's own `which_one`.
    fn ask_which_one(&self, conv: &Conversation, find_id: &str) -> Finished {
        let state = State {
            results: conv.results.clone(),
            turn: u32::try_from(conv.turns.len()).unwrap_or(u32::MAX),
        };
        let found = state.find(find_id);
        let count = match found.and_then(|r| r.node.get(&["matches"])) {
            Some(Node::List(items)) => items.len(),
            _ => 0,
        };
        // A name that exists only in another role than the one asked (a buyer in a seller's field)
        // is not found as that role, and says what he is (plan 52 Section 8).
        let widened = route::is_widened(&state, find_id);
        let mut f = Finished::plain(if count == 0 || widened {
            AskOutcome::NotFound
        } else {
            AskOutcome::WhichOne
        });
        f.options = fill::options_of(&state, find_id);
        f.more_options = found
            .and_then(|r| r.node.get(&["more"]))
            .is_some_and(|n| matches!(n, Node::Bool(true)));
        if f.outcome == AskOutcome::NotFound {
            f.asked_role = route::asked_role_of_find(&state, find_id);
            f.found_as = route::found_as_of(&state, find_id, f.asked_role);
        }
        if count > 0 {
            f.wait = Some(Wait::Which {
                find_id: find_id.to_owned(),
            });
        }
        f
    }

    /// The role and the typed name of a click on a name found in another role: `Some(("buyer",
    /// "faisal"))` when `key` is the one party the widened `find` found, and he is a buyer or a
    /// seller. `None` for any other click.
    fn found_in_other_role(
        &self,
        conv: &Conversation,
        find_id: &str,
        key: &str,
    ) -> Option<(&'static str, String)> {
        let state = State {
            results: conv.results.clone(),
            turn: u32::try_from(conv.turns.len()).unwrap_or(u32::MAX),
        };
        if !route::is_widened(&state, find_id) {
            return None;
        }
        let asked = route::asked_role_of_find(&state, find_id);
        let found = route::found_as_of(&state, find_id, asked)?;
        if found.key != key {
            return None;
        }
        let word = match found.role {
            AskedRole::Buyer => "buyer",
            AskedRole::Seller => "seller",
            _ => return None,
        };
        let text = state.find(find_id)?.input.get("text")?.as_str()?.to_owned();
        Some((word, text))
    }

    /// Ends the question as "not found" for a name that exists in the other role than the question's
    /// verb gives ("Faisal ne kya becha", and he is a buyer): no model call, `asked_role` the role
    /// of the verb, `found_as` the party, and his name offered, so that "show his purchases" is one
    /// click (plan 52 Section 8).
    fn not_found_in_other_role(
        &self,
        conv: &Conversation,
        find_id: &str,
        asked: AskedRole,
    ) -> Finished {
        let state = State {
            results: conv.results.clone(),
            turn: u32::try_from(conv.turns.len()).unwrap_or(u32::MAX),
        };
        let mut f = Finished::plain(AskOutcome::NotFound);
        f.options = fill::options_of(&state, find_id);
        f.asked_role = Some(asked);
        f.found_as = route::found_as_of(&state, find_id, Some(asked));
        f.wait = Some(Wait::Which {
            find_id: find_id.to_owned(),
        });
        f
    }

    /// The guide topic of a help question that the model ended as a change request, as an answer
    /// of the app's own words (plan 52 item 4). `None` when the question is not help: it has a
    /// number to write, or does not ask how, where or what, or the answer is not a change request.
    /// The guide is the one the model read; when it read none, the app reads the topic of the
    /// screen the model named.
    fn help_instead(
        &self,
        conv: &mut Conversation,
        turn: &mut Turn,
        written: &ModelAnswer,
    ) -> Option<ModelAnswer> {
        if written.kind != AnswerKind::ChangeRequest {
            return None;
        }
        let numbers = figures::question_numbers(&turn.question);
        if !help_guard::is_how_to(&turn.question, &numbers) {
            return None;
        }
        let index = u32::try_from(conv.turns.len()).unwrap_or(u32::MAX);
        let read = conv
            .results
            .iter()
            .rev()
            .find(|r| r.tool == "app_help" && r.turn == index)
            .map(|r| r.id.clone());
        let id = match read {
            Some(id) => id,
            None => {
                let screen = written.entry.as_ref()?.screen;
                self.read_help(conv, turn, help_guard::topic_of(screen))?
            }
        };
        let result = conv.results.iter().find(|r| r.id == id)?;
        let steps = match result.node.get(&["steps"]) {
            Some(Node::List(items)) => items.len(),
            _ => 0,
        };
        let mut text = format!("{{{{{id}.summary}}}}");
        for step in 0..steps {
            text.push_str(&format!("\n• {{{{{id}.steps.{step}}}}}"));
        }
        Some(ModelAnswer {
            kind: AnswerKind::Answer,
            text,
            show: Vec::new(),
            next: Vec::new(),
            choose: None,
            entry: written.entry.clone(),
            nearest: None,
        })
    }

    /// Puts the guide of a topic into the conversation as if the model had asked for it, with the
    /// id `h1` (its own letter, so the ids `r1`, `r2` ... of the model's own calls never move).
    /// Nothing counts against the model's tool calls (plan 52 item 7).
    fn preread_help(&self, conv: &mut Conversation, turn: &mut Turn, topic: &str) {
        let index = u32::try_from(conv.turns.len()).unwrap_or(u32::MAX);
        let numbers = figures::question_numbers(&turn.question);
        let question = turn.question.clone();
        let env = self.env(&numbers, &question);
        let mut state = State {
            results: std::mem::take(&mut conv.results),
            turn: index,
        };
        let arguments = json!({"topic": topic}).to_string();
        if let Ok(draft) = tools::call(&env, &state, "app_help", &arguments) {
            let id = state.next_id('h');
            let mut result = tools::stored(draft, id.clone(), index);
            fit(&mut result, &self.words);
            let content = tool_result_json(&result, &self.words).to_string();
            turn.tool_log
                .push(json!({"id": id, "tool": result.tool, "input": result.input}));
            state.results.push(result);
            turn.messages.push(json!({
                "role": "assistant", "content": null,
                "tool_calls": [{"id": "pre_help", "type": "function",
                                "function": {"name": "app_help", "arguments": arguments}}]
            }));
            turn.messages
                .push(tool_message("pre_help", content, Some(&id)));
        }
        conv.results = state.results;
    }

    /// Reads a tool for the model before its first call, as if it had asked for it: the period a
    /// question names, or the tiles a question is about. The result has the normal number (`r1`),
    /// is in the log with `prefetch: true`, and a later identical call of the model gets it back
    /// (plan 52 P52-S1: 62 questions began with a call that only asked for a period, 43 with the
    /// tiles). Nothing is said when the tool cannot be read.
    fn preread_tool(
        &self,
        conv: &mut Conversation,
        turn: &mut Turn,
        tool: &str,
        arguments: &str,
    ) -> Option<String> {
        let index = u32::try_from(conv.turns.len()).unwrap_or(u32::MAX);
        let numbers = figures::question_numbers(&turn.question);
        let question = turn.question.clone();
        let env = self.env(&numbers, &question);
        let mut state = State {
            results: std::mem::take(&mut conv.results),
            turn: index,
        };
        let call = ToolCall {
            id: String::new(),
            name: tool.to_owned(),
            arguments: arguments.to_owned(),
        };
        (self.d.emit)(step_of(&call, &env, &state));
        let mut read = None;
        if let Ok(draft) = tools::call(&env, &state, tool, arguments) {
            let id = state.next_id(prefix_of(draft.tool));
            let mut result = tools::stored(draft, id.clone(), index);
            fit(&mut result, &self.words);
            let content = tool_result_json(&result, &self.words).to_string();
            turn.tool_log.push(json!({
                "id": id, "tool": result.tool, "input": result.input, "prefetch": true,
            }));
            state.results.push(result);
            if let Some(input) = prefetch::canonical(tool, arguments) {
                turn.prefetched.push((tool.to_owned(), input, id.clone()));
            }
            let call_id = format!("pre_{tool}_{}", turn.messages.len());
            turn.messages.push(json!({
                "role": "assistant", "content": null,
                "tool_calls": [{"id": call_id, "type": "function",
                                "function": {"name": tool, "arguments": arguments}}]
            }));
            turn.messages
                .push(tool_message(&call_id, content, Some(&id)));
            read = Some(id);
        }
        conv.results = state.results;
        read
    }

    /// Finds the party a plain lookup question names, before the first call, when there is no
    /// doubt (see `prefind`): one word finds exactly one account in the role the question implies,
    /// no other word finds a different single account. Then the find, his card and what the
    /// question names are read as r1, r2 ... and the model's first call can answer.
    fn prefind(&self, conv: &mut Conversation, turn: &mut Turn) {
        let numbers = figures::question_numbers(&turn.question);
        if !conv.turns.is_empty() || !prefind::plain_lookup(&turn.question, numbers.is_empty()) {
            return;
        }
        let words = prefind::name_words(&turn.question);
        if words.is_empty() || words.len() > prefind::MAX_WORDS {
            return;
        }
        let role = prefind::role_of(&turn.question);
        let index = u32::try_from(conv.turns.len()).unwrap_or(u32::MAX);
        let question = turn.question.clone();
        let env = self.env(&numbers, &question);
        let scratch = State {
            results: Vec::new(),
            turn: index,
        };
        // The words that find exactly one account, with its key.
        let mut singles: Vec<(String, String)> = Vec::new();
        for word in &words {
            let arguments = json!({"text": word, "role": role}).to_string();
            let Ok(draft) = tools::call(&env, &scratch, "find", &arguments) else {
                continue;
            };
            if let Some(key) = prefind::single_key(&draft.node) {
                singles.push((word.clone(), key));
            }
        }
        let mut keys: Vec<&String> = singles.iter().map(|(_, k)| k).collect();
        keys.dedup();
        let [only] = keys.as_slice() else {
            return;
        };
        let Some((word, _)) = singles.iter().find(|(_, k)| k == *only) else {
            return;
        };
        let arguments = json!({"text": word, "role": role}).to_string();
        let Some(find_id) = self.preread_tool(conv, turn, "find", &arguments) else {
            return;
        };
        let mut state = State {
            results: std::mem::take(&mut conv.results),
            turn: index,
        };
        if prefetch::single_party_match(&state, &find_id).is_some() {
            turn.prefound = state
                .find(&find_id)
                .and_then(|r| r.node.get(&["matches", "0", "name"]).cloned())
                .and_then(|n| match n {
                    Node::Val(crate::results::Val::Name(name)) => Some(name.answer_name()),
                    _ => None,
                });
            self.prefetch(turn, &mut state, index, &numbers, &question, &[find_id]);
        }
        conv.results = state.results;
    }

    /// Reads one topic of the guide for the app itself (the model asked for none). The read is in
    /// the question's log. It counts as no tool call of the model.
    fn read_help(&self, conv: &mut Conversation, turn: &mut Turn, topic: &str) -> Option<String> {
        let index = u32::try_from(conv.turns.len()).unwrap_or(u32::MAX);
        let numbers = figures::question_numbers(&turn.question);
        let question = turn.question.clone();
        let env = self.env(&numbers, &question);
        let mut state = State {
            results: std::mem::take(&mut conv.results),
            turn: index,
        };
        let arguments = json!({"topic": topic}).to_string();
        let found = tools::call(&env, &state, "app_help", &arguments)
            .ok()
            .map(|draft| {
                let id = state.next_id(prefix_of(draft.tool));
                let mut result = tools::stored(draft, id.clone(), index);
                fit(&mut result, &self.words);
                turn.tool_log
                    .push(json!({"id": id, "tool": result.tool, "input": result.input}));
                state.results.push(result);
                id
            });
        conv.results = state.results;
        found
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
                // A main screen (a help answer, PRD FR-7.89) opens as it is: it takes no party.
                party: e
                    .party
                    .as_deref()
                    .filter(|_| e.screen.is_entry_screen())
                    .and_then(|p| tools::record(env, state, p, "party").ok())
                    .filter(|r| r.prefix == wanted)
                    .and_then(|r| {
                        tools::records::answer_record(state, &format!("{}{}", r.prefix, r.id))
                    }),
            }
        });
        match parsed.kind {
            AnswerKind::Answer
            | AnswerKind::Estimate
            | AnswerKind::Advice
            | AnswerKind::GeneralAdvice => {
                let mut f = Finished::plain(match parsed.kind {
                    AnswerKind::Estimate => AskOutcome::Estimate,
                    AnswerKind::Advice | AnswerKind::GeneralAdvice => AskOutcome::Advice,
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
                if parsed.kind == AnswerKind::NotFound {
                    f.asked_role = route::asked_role_of_find(state, &find_id);
                    f.found_as = route::found_as_of(state, &find_id, f.asked_role);
                }
                if parsed.kind == AnswerKind::WhichOne
                    || (parsed.kind == AnswerKind::NotFound && !f.options.is_empty())
                {
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
                        general_advice: false,
                        sources: vec![source],
                        how: Vec::new(),
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
            // The time of this run: a question that waited for the partner code and went on is
            // two runs and two rows, each with its own time (plan 51 B2).
            total_millis: Some(millis_since(self.started)),
            model_millis: (source != Source::Chip && source != Source::Rerun)
                .then(|| turn.timing.model_millis()),
            tool_millis: (source != Source::Chip && source != Source::Rerun)
                .then(|| turn.timing.tool_millis()),
            calls_json: (source != Source::Chip && source != Source::Rerun)
                .then(|| turn.timing.to_json().to_string()),
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
            stop_reason: f.stop_reason,
            count_today: assistant::count_today(self.d.ctx)?,
            limit: DAILY_LIMIT,
            asked_role: f.asked_role,
            found_as: f.found_as,
        })
    }
}

/// The id of a find made in this same reply whose match a call reads (`r1.matches.0`), when that
/// find did not match exactly one record (or has more than it showed), or of a typed-name find kept
/// for the Owner's choice (`kept`, this question's earlier replies too). `None` when the call reads
/// no such match or the find had one.
fn unclear_find_read(
    state: &State,
    batch_finds: &[String],
    kept: &[String],
    arguments: &str,
) -> Option<String> {
    fn walk(value: &Value, found: &mut Vec<String>) {
        match value {
            Value::String(text) => {
                if let Some((root, rest)) = text.split_once('.')
                    && rest.starts_with("matches")
                {
                    found.push(root.to_owned());
                }
            }
            Value::Array(items) => items.iter().for_each(|v| walk(v, found)),
            Value::Object(map) => map.values().for_each(|v| walk(v, found)),
            _ => {}
        }
    }
    let args: Value = serde_json::from_str(arguments).ok()?;
    let mut roots = Vec::new();
    walk(&args, &mut roots);
    roots
        .into_iter()
        .filter(|root| batch_finds.contains(root) || kept.contains(root))
        .find(|root| {
            // A typed-name find that was kept for the Owner's choice is never read (Q111: a model
            // that went on after a failed name could read `f1.matches.0` of an unclear find).
            if kept.contains(root) {
                return true;
            }
            let Some(result) = state.find(root) else {
                return false;
            };
            let count = match result.node.get(&["matches"]) {
                Some(Node::List(items)) => items.len(),
                _ => 0,
            };
            let more = matches!(result.node.get(&["more"]), Some(Node::Bool(true)));
            count != 1 || more
        })
}

/// The model's answer with each text of it (`text` and every `next` question) changed by `fix`, as
/// the JSON text it was. `None` when `fix` changed nothing or the content is not JSON.
fn map_texts(content: &str, fix: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    let mut value: Value = serde_json::from_str(content).ok()?;
    let mut changed = false;
    if let Some(Value::String(text)) = value.get_mut("text")
        && let Some(fixed) = fix(text)
    {
        *text = fixed;
        changed = true;
    }
    if let Some(Value::Array(next)) = value.get_mut("next") {
        for item in next {
            if let Value::String(text) = item
                && let Some(fixed) = fix(text)
            {
                *text = fixed;
                changed = true;
            }
        }
    }
    changed.then(|| value.to_string())
}

/// The model's answer with the names it typed put back as references (`NameSet::put_back`).
fn repaired_content(content: &str, state: &State, names: &NameSet) -> Option<String> {
    map_texts(content, &|text| names.put_back(state, text))
}

/// The model's answer with the Owner's own numbers, written in words, put back as `{{q.nK}}`
/// (`echo::put_numbers_back`).
fn echoed_content(content: &str, numbers: &[QuestionNumber]) -> Option<String> {
    map_texts(content, &|text| echo::put_numbers_back(text, numbers))
}

/// The answer of a help question written from the guide the model read: its summary, its steps and
/// its keys, each as a reference to the guide's own text and nothing else (plan 52 P52-S1). Used
/// when the model retyped the guide in its own words and only the words rules failed. `None` when
/// the question read anything but guides, or the model's answer is not a plain answer, or a rule
/// other than the words rules failed (a wrong reference, a name, a figure ...).
fn help_repaired_content(
    content: &str,
    state: &State,
    parsed: &ModelAnswer,
    failures: &[Failure],
) -> Option<String> {
    const WORD_RULES: [&str; 6] = [
        "script",
        "digit",
        "number_word",
        "too_long",
        "never_word",
        "check_mark",
    ];
    if parsed.kind != AnswerKind::Answer
        || failures.is_empty()
        || !failures.iter().all(|f| WORD_RULES.contains(&f.rule))
    {
        return None;
    }
    let guides: Vec<&ToolResult> = state
        .results
        .iter()
        .filter(|r| r.turn == state.turn)
        .collect();
    if guides.is_empty() || guides.iter().any(|r| r.tool != "app_help") {
        return None;
    }
    // The guide of the screen the model named, else the last one it read.
    let wanted = parsed
        .entry
        .as_ref()
        .map(|e| help_guard::topic_of(e.screen));
    let guide = guides
        .iter()
        .rev()
        .find(|r| r.input.get("topic").and_then(Value::as_str) == wanted)
        .or(guides.last())?;
    let steps = match guide.node.get(&["steps"]) {
        Some(Node::List(items)) => items.len(),
        _ => 0,
    };
    let id = &guide.id;
    let mut text = format!("{{{{{id}.summary}}}}");
    for step in 0..steps {
        text.push_str(&format!("\n• {{{{{id}.steps.{step}}}}}"));
    }
    if guide.node.get(&["keys"]).is_some() {
        text.push_str(&format!("\n• {{{{{id}.keys}}}}"));
    }
    let entry = serde_json::from_str::<Value>(content)
        .ok()
        .and_then(|v| v.get("entry").cloned())
        .unwrap_or(Value::Null);
    Some(
        json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": entry, "nearest": null
        })
        .to_string(),
    )
}

/// True when the next call must answer and may call no tool (plan 52 item 1, PRD E96): it is the
/// last model call the question may make, or the tool calls are used up, or it is the rewrite
/// (which may not read anything new, D6-system-prompt 5.3).
/// The log entries of a local answer (one the app wrote itself, with no tool and no model): what
/// it used, so the Owner's question log and the scorer can see it. Each entry is marked
/// `"local": true`, because nothing was run: [`Run::refresh`] and [`rerun`] skip such entries.
///
/// - the guide topic of a how-to or a screen: `{tool: "app_help", local: true, input: {topic}}`;
/// - each report button, one entry in the order of the buttons: `{tool: "report", local: true,
///   input: {id, ...the filters of the button}}`, only the filters that are set; a tiles button:
///   `{tool: "tiles", local: true, input: {}}`;
/// - a name no record has: `{tool: "find", local: true, input: {text, role}}`.
///
/// Example: "akram ki aaj ki bikri ka print" logs `{tool: "report", local: true, input: {id:
/// "R35", as_of: "2026-10-10", party: {...}}}`.
fn local_log(
    understood: &crate::understand::Understanding,
    composed: &crate::understand::Composed,
) -> Vec<Value> {
    use crate::understand::{Outcome, PartyRole};
    let mut entries: Vec<(&str, Value)> = Vec::new();
    if let Some(topic) = &understood.topic
        && composed.outcome == AskOutcome::Answered
    {
        entries.push(("app_help", json!({"topic": topic.value})));
    }
    if let Some(answer) = &composed.answer {
        for source in &answer.sources {
            match source {
                AnswerSource::Report { id, filters, .. } => {
                    let mut input = serde_json::Map::new();
                    input.insert(
                        "id".to_owned(),
                        serde_json::to_value(id).unwrap_or(Value::Null),
                    );
                    if let Value::Object(set) =
                        compact_json(serde_json::to_value(filters).unwrap_or(Value::Null))
                    {
                        input.extend(set);
                    }
                    entries.push(("report", Value::Object(input)));
                }
                AnswerSource::Tiles { .. } => entries.push(("tiles", json!({}))),
                AnswerSource::Screen { route, .. } => {
                    entries.push(("screen", json!({"route": route})));
                }
                AnswerSource::PartyCard { .. } | AnswerSource::Analytics { .. } => {}
            }
        }
    }
    if composed.outcome == AskOutcome::NotFound && composed.found_as.is_none() {
        for party in &understood.parties {
            if matches!(party.outcome, Outcome::NotFound) {
                let role = match party.asked_role {
                    Some(PartyRole::Seller) => "seller",
                    Some(PartyRole::Buyer) => "buyer",
                    _ => "any",
                };
                entries.push(("find", json!({"text": party.text, "role": role})));
            }
        }
    }
    entries
        .into_iter()
        .map(|(tool, input)| json!({"tool": tool, "local": true, "input": input}))
        .collect()
}

/// A JSON value without its empty parts: no null, no empty text, no `false`, no empty list or
/// object. For the filters of a button in the log, where only the ones that were set matter.
fn compact_json(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, compact_json(v)))
                .filter(|(_, v)| !is_empty_json(v))
                .collect(),
        ),
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(compact_json)
                .filter(|v| !is_empty_json(v))
                .collect(),
        ),
        other => other,
    }
}

fn is_empty_json(value: &Value) -> bool {
    match value {
        Value::Null | Value::Bool(false) => true,
        Value::String(text) => text.is_empty(),
        Value::Array(items) => items.is_empty(),
        Value::Object(map) => map.is_empty(),
        _ => false,
    }
}

/// True when a question that reads the books takes something from the questions before it, so the
/// app, which reads only the new words, would read the wrong thing (money review R-c).
///
/// Two cases. A figure that depends on a period, with no period named: after "akram ka pichle mahine
/// ka commission", "arshad ka commission?" means last month too, and the app alone would read all
/// time. A question that names no party, item, record or period, and is not about the shop now:
/// "uska print nikalo" or "aur batao" has no subject of its own.
///
/// Example: after any question, "cash kitna hai" does not lean (cash now needs nothing before it),
/// and "faisal ki aaj ki bikri ka print" does not lean (a party and a day).
fn leans_on_earlier_questions(u: &crate::understand::Understanding) -> bool {
    use crate::understand::{FigureKind, Intent};
    if !matches!(
        u.intent.value,
        Intent::Figure | Intent::List | Intent::Print
    ) {
        return false;
    }
    let has_period = !u.periods.is_empty();
    let has_subject =
        !u.parties.is_empty() || !u.items.is_empty() || !u.records.is_empty() || u.list.is_some();
    let depends_on_period = matches!(
        u.figure(),
        Some(
            FigureKind::Sale
                | FigureKind::SaleCount
                | FigureKind::Commission
                | FigureKind::Laga
                | FigureKind::Purchases
                | FigureKind::Receipts
                | FigureKind::Profit
                | FigureKind::Expense
                | FigureKind::FruitTaken
                | FigureKind::Rate
        )
    );
    let about_the_shop_now = matches!(
        u.figure(),
        Some(
            FigureKind::Cash
                | FigureKind::Dues
                | FigureKind::Payable
                | FigureKind::Advance
                | FigureKind::Stock
                | FigureKind::MoneyLent
                | FigureKind::DeadAmount
                | FigureKind::DaySummary
        )
    );
    (depends_on_period && !has_period) || (!has_subject && !has_period && !about_the_shop_now)
}

fn tools_off(turn: &Turn, rewrite: bool) -> bool {
    let counters = &turn.counters;
    rewrite
        // General advice reads nothing: the one call answers with no tool.
        || turn.general_advice
        || counters.model_calls.saturating_add(1) >= MAX_MODEL_CALLS
        || counters.tool_calls >= MAX_TOOL_CALLS
}

/// The word of a failure kind for the log (`offline`, `timeout`, `busy` ...).
/// The word the log uses for a side of a hedged call.
fn side_word(side: Side) -> &'static str {
    match side {
        Side::First => "first",
        Side::Copy => "copy",
    }
}

/// The log entry of one request of a hedged call: its own time, no tokens (a reply that was not
/// used is not counted), and how it ended: `hedge_lost` when the other answered first, or its own
/// failure (`timeout` ...). `extra` leaves it out of the time sums.
fn hedge_entry(attempt: &Attempt, rewrite: bool, extra: bool) -> CallTiming {
    let error = match attempt.end {
        End::Failed(kind) => failure_word(kind),
        End::Cancelled | End::Won => Some("hedge_lost".to_owned()),
    };
    CallTiming {
        model_millis: attempt.millis,
        tokens_in: 0,
        tokens_cached: 0,
        tokens_out: 0,
        tokens_reasoning: 0,
        rewrite,
        failed: true,
        tools: Vec::new(),
        error,
        check_failed: Vec::new(),
        repaired: Vec::new(),
        check_paths: Vec::new(),
        check_at: Vec::new(),
        hedged: Some(side_word(attempt.side)),
        hedge_extra: extra,
    }
}

fn failure_word(kind: FailureKind) -> Option<String> {
    serde_json::to_value(kind)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
}

/// Whole milliseconds since a moment, for the log (plan 51 B2).
fn millis_since(moment: Instant) -> i64 {
    i64::try_from(moment.elapsed().as_millis()).unwrap_or(i64::MAX)
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
/// True when a tool that reads the books (a report, a view, a card, the tiles, an estimate, a
/// calculation or the state of the app: rules, attention, closed days) ran without an error since this point of the question's log.
fn read_data_since(turn: &Turn, start: usize) -> bool {
    turn.tool_log.iter().skip(start).any(|entry| {
        entry.get("error").is_none()
            && matches!(
                entry.get("tool").and_then(Value::as_str),
                Some(
                    "analytics"
                        | "report"
                        | "party_card"
                        | "tiles"
                        | "estimate"
                        | "calculate"
                        | "app_state"
                )
            )
    })
}

fn party_prefix(screen: EntryScreen) -> &'static str {
    match screen {
        EntryScreen::Kharcha | EntryScreen::Sarmaya | EntryScreen::Phal => "s",
        EntryScreen::GardenInstallment
        | EntryScreen::GardenCost
        | EntryScreen::GardenSale
        | EntryScreen::GardenClose => "g",
        EntryScreen::TripPurchase | EntryScreen::TripCost | EntryScreen::TripSale => "t",
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
        EntryScreenName::Dashboard => EntryScreen::Dashboard,
        EntryScreenName::Settings => EntryScreen::Settings,
        EntryScreenName::Masters => EntryScreen::Masters,
        EntryScreenName::Accounts => EntryScreen::Accounts,
        EntryScreenName::CommissionRules => EntryScreen::CommissionRules,
        EntryScreenName::TodaySales => EntryScreen::TodaySales,
        EntryScreenName::UnsoldStock => EntryScreen::UnsoldStock,
        EntryScreenName::PukhtaKhata => EntryScreen::PukhtaKhata,
        EntryScreenName::CollectionList => EntryScreen::CollectionList,
        EntryScreenName::CashBook => EntryScreen::CashBook,
        EntryScreenName::Gardens => EntryScreen::Gardens,
        EntryScreenName::MarketTrips => EntryScreen::MarketTrips,
        EntryScreenName::Partners => EntryScreen::Partners,
        EntryScreenName::MonthClose => EntryScreen::MonthClose,
        EntryScreenName::DayClose => EntryScreen::DayClose,
        EntryScreenName::Reports => EntryScreen::Reports,
        EntryScreenName::DeadAmount => EntryScreen::DeadAmount,
        EntryScreenName::Analytics => EntryScreen::Analytics,
        EntryScreenName::Backup => EntryScreen::Backup,
        EntryScreenName::GardenInstallment => EntryScreen::GardenInstallment,
        EntryScreenName::GardenCost => EntryScreen::GardenCost,
        EntryScreenName::GardenSale => EntryScreen::GardenSale,
        EntryScreenName::GardenClose => EntryScreen::GardenClose,
        EntryScreenName::TripPurchase => EntryScreen::TripPurchase,
        EntryScreenName::TripCost => EntryScreen::TripCost,
        EntryScreenName::TripSale => EntryScreen::TripSale,
        EntryScreenName::BikriLaterSale => EntryScreen::BikriLaterSale,
        EntryScreenName::BikriEdit => EntryScreen::BikriEdit,
        EntryScreenName::BikriCancel => EntryScreen::BikriCancel,
    }
}

/// Where the record `key` is among the matches of the `find` result `find_id`.
fn match_position(results: &[ToolResult], find_id: &str, key: &str) -> Option<usize> {
    let result = results.iter().find(|r| r.id == find_id)?;
    match result.node.get(&["matches"]) {
        Some(Node::List(items)) => items
            .iter()
            .position(|m| matches!(m.get(&["key"]), Some(Node::Str(k)) if k == key)),
        _ => None,
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
    // The app writes the answer itself when it understood the whole question and the answer is
    // already in the app. It costs no model call, so it is not counted and the daily limit does
    // not stop it (FR-7.81).
    let understood = if d.runtime.local() || d.runtime.read_first() {
        run.understand_question(text, !conv.turns.is_empty())
    } else {
        None
    };
    turn.planned = understood
        .as_ref()
        .and_then(crate::understand::plan_reads)
        .map(|reads| {
            reads
                .into_iter()
                .map(|r| (r.tool.to_owned(), r.input))
                .collect()
        })
        .unwrap_or_default();
    if d.runtime.local()
        && let Some(u) = &understood
        && let Some(finished) = run.local_answer(&mut conv, &mut turn, u)
    {
        return run.conclude(conv, turn, finished, Source::Model, None);
    }
    let numbers = figures::question_numbers(text);
    turn.messages
        .push(user_message(&run.today_iso, run.language, text, &numbers));
    // A simple figure whose reads already hold it is written by the app (PRD FR-7.90, plan 55
    // Section 10). It costs no model call, so it is not counted and the daily limit does not stop
    // it, like the other local answers. When it cannot be written or a check refuses it, the
    // reads stay and the question goes on to the model, which only has to write.
    if d.runtime.instant()
        && let Some(u) = &understood
        && let Some(finished) = run.instant_answer(&mut conv, &mut turn, u)
    {
        return run.conclude(conv, turn, finished, Source::Model, None);
    }
    if assistant::count_today(d.ctx)? >= DAILY_LIMIT {
        return run.conclude(
            conv,
            turn,
            Finished::plain(AskOutcome::LimitReached),
            Source::Model,
            None,
        );
    }
    // A how-to question about one screen: the guide of that topic is read for the model before
    // its first call, so help takes one call (plan 52 item 7).
    if help_guard::general_advice_asked(text, &numbers) && conv.turns.is_empty() {
        // How to do better at something the shop does: no tool, one call (PRD FR-7.88).
        turn.general_advice = true;
    } else if let Some(topic) = help_guard::topic_of_question(text, &numbers) {
        run.preread_help(&mut conv, &mut turn, topic);
        turn.read_first = true;
    } else if turn.read_first {
        // The instant attempt already read the books for this question: the call only writes.
    } else if d.runtime.read_first()
        && let Some(u) = &understood
        && run.read_first(&mut conv, &mut turn, u)
    {
        // The books were read for the question before the first call (plan 54 S3): the call has
        // only to write the answer.
        turn.read_first = true;
    } else if let Some(kind) = cues::period_kind(text) {
        // One clear period in the question: read it before the first call, so that call can
        // already ask for the report (plan 52 P52-S1).
        let input = json!({"kind": kind, "month": null, "day": null, "from": null,
                           "to": null, "n": null});
        let period = run.preread_tool(&mut conv, &mut turn, "period", &input.to_string());
        // "How many بکری" in a plural word: the number is the bikri_count of view B, not the lines
        // of R2 (Q152 of the live run of 03-11-2026: 122 lines read as 122 بکری, the books hold
        // 32). View B is read for the same period, so the first call can answer.
        if let Some(id) = period
            && route::bikri_count_asked(text)
        {
            let view = json!({
                "view": "B", "from": format!("{id}.from"), "to": format!("{id}.to"),
                "group_by": null, "item": null, "seller": null, "buyer": null, "city": null,
                "adda": null, "garden": null, "trip": null, "goods": null, "partner": null,
                "side": null, "compare": null, "sort": null, "rows": null
            });
            run.preread_tool(&mut conv, &mut turn, "analytics", &view.to_string());
        }
    } else if cues::tile_question(text) {
        run.preread_tool(&mut conv, &mut turn, "tiles", "{}");
    } else if d.runtime.prefind() {
        // A plain question about one party the app can name without a doubt (item 10).
        run.prefind(&mut conv, &mut turn);
    }
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
            if match_position(&conv.results, find_id, key).is_none() {
                Err(ServiceError::reject_field(ErrorCode::ErrNotFound, "key"))
            } else {
                // A name found in another role ("Faisal ne kya becha", and he is a buyer): the
                // Owner chose to see him as what he is. The find the model has in front of it was
                // made as the other role and ended the question as "not found" twice in the live
                // run (Q427 to Q429, Q431), so the app finds him again in his real role, and the
                // click points at that find.
                let other = run.found_in_other_role(&conv, find_id, key);
                let (use_id, as_role) = match other {
                    Some((word, text)) => {
                        let input = json!({"text": text, "role": word});
                        match run.preread_tool(&mut conv, &mut turn, "find", &input.to_string()) {
                            Some(new_id)
                                if match_position(&conv.results, &new_id, key).is_some() =>
                            {
                                (new_id, Some(word))
                            }
                            _ => (find_id.clone(), None),
                        }
                    }
                    None => (find_id.clone(), None),
                };
                let at = match_position(&conv.results, &use_id, key).unwrap_or(0);
                let mut picked = json!({"picked": format!("{use_id}.matches.{at}")});
                if let (Some(word), Some(map)) = (as_role, picked.as_object_mut()) {
                    map.insert("as".to_owned(), json!(word));
                    map.insert(
                        "note".to_owned(),
                        json!(if word == "buyer" {
                            "the Owner chose to see him as a buyer: answer for what he bought, not for sales"
                        } else {
                            "the Owner chose to see him as a seller: answer for what he sold, not for purchases"
                        }),
                    );
                }
                turn.messages.push(json!({
                    "role": "user",
                    "content": picked.to_string(),
                }));
                Ok(Vec::new())
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
    // This run has its own time; the counters of the question go on from the first run.
    turn.timing = Timing::default();
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
    // An entry marked `local` says what a local answer used; nothing was run, so nothing is read.
    for call in calls
        .iter()
        .filter(|c| c.get("error").is_none() && (c.get("local").is_none() || c.get("id").is_some()))
    {
        let (Some(tool), Some(input)) =
            (call.get("tool").and_then(Value::as_str), call.get("input"))
        else {
            continue;
        };
        let env = run.env(&numbers, &row.question);
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
