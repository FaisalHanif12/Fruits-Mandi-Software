//! The conversation the panel keeps (PRD FR-7.78) and what a question that waits keeps.
//!
//! A conversation lives in memory while the app runs: the earlier questions with their messages
//! and tool results, so "aur pichle mahine?" can follow a question about اکرم. Only the last 3
//! questions are sent with a new one. A question that waits (for "کون سا؟" or for the partner code)
//! keeps its messages and what is left of its limits, and goes on from there; after a restart it
//! is gone and the continuation answers `ERR_ASSISTANT_NOT_WAITING`.
//!
//! The log (`assistant_questions`) is the permanent record; this is only the working memory.

use serde_json::Value;

use crate::client::ToolCall;
use crate::results::ToolResult;

/// The numbers one question has used so far.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counters {
    /// Model calls made, not counting the rewrite.
    pub model_calls: u32,
    /// The rewrite, 0 or 1.
    pub rewrites: u32,
    /// Tool calls made.
    pub tool_calls: u32,
    /// Input tokens, cached ones included.
    pub tokens_in: i64,
    /// Of those, cached.
    pub tokens_cached: i64,
    /// Output tokens, reasoning included.
    pub tokens_out: i64,
    /// Of those, reasoning.
    pub tokens_reasoning: i64,
}

/// One tool call of a model call, with the time it took (plan 51 B2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolTiming {
    /// The tool's name (a fixed word, never a figure or a name).
    pub name: String,
    /// Milliseconds the app's own work took.
    pub millis: i64,
    /// False when the tool refused its input.
    pub ok: bool,
}

/// One model call with the time it took and the tools it asked for (plan 51 B2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallTiming {
    /// Milliseconds spent waiting for the model: the call, from sending to the whole reply.
    pub model_millis: i64,
    /// Input tokens, cached ones included.
    pub tokens_in: i64,
    /// Of those, cached.
    pub tokens_cached: i64,
    /// Output tokens, reasoning included.
    pub tokens_out: i64,
    /// Of those, reasoning.
    pub tokens_reasoning: i64,
    /// True for the one rewrite after a failed check.
    pub rewrite: bool,
    /// True when the call itself failed (no internet, a timeout, a bad key ...): its time is how
    /// long the app waited before it gave up.
    pub failed: bool,
    /// The tools this call asked for, in order.
    pub tools: Vec<ToolTiming>,
}

/// Where the time of one run of a question went. A question that waits for the partner code and
/// goes on is two runs, each with its own timing and its own row of the log.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Timing {
    /// One entry for each model call of this run, in order.
    pub calls: Vec<CallTiming>,
}

impl Timing {
    /// Milliseconds waiting for the model.
    pub fn model_millis(&self) -> i64 {
        self.calls
            .iter()
            .fold(0_i64, |sum, c| sum.saturating_add(c.model_millis))
    }

    /// Milliseconds of the app's own work in its tools.
    pub fn tool_millis(&self) -> i64 {
        self.calls
            .iter()
            .flat_map(|c| c.tools.iter())
            .fold(0_i64, |sum, t| sum.saturating_add(t.millis))
    }

    /// The calls as the JSON the log keeps: counts and times only, no name, no figure.
    pub fn to_json(&self) -> serde_json::Value {
        let calls: Vec<serde_json::Value> = self
            .calls
            .iter()
            .enumerate()
            .map(|(i, c)| {
                serde_json::json!({
                    "n": i.saturating_add(1),
                    "model_ms": c.model_millis,
                    "tokens_in": c.tokens_in,
                    "tokens_cached": c.tokens_cached,
                    "tokens_out": c.tokens_out,
                    "tokens_reasoning": c.tokens_reasoning,
                    "rewrite": c.rewrite,
                    "failed": c.failed,
                    "tools": c.tools.iter()
                        .map(|t| serde_json::json!({"name": t.name, "ms": t.millis, "ok": t.ok}))
                        .collect::<Vec<_>>(),
                })
            })
            .collect();
        serde_json::Value::Array(calls)
    }
}

/// One question of the conversation.
#[derive(Debug, Clone, Default)]
pub struct Turn {
    /// The question as typed.
    pub question: String,
    /// The messages of the question in order: the user message, then each assistant message and its
    /// tool messages, then the final answer. A tool message carries `_result`, the id of its
    /// result, which is taken out before sending.
    pub messages: Vec<Value>,
    /// The rows its answer referred to: (result id, row key). Only these rows are resent.
    pub referenced: Vec<(String, String)>,
    /// Each tool called: `{"id","tool","input"}`, for the log and for reading it again.
    pub tool_log: Vec<Value>,
    /// What it used so far.
    pub counters: Counters,
    /// Where the time of this run went (plan 51 B2).
    pub timing: Timing,
}

/// What a waiting question waits for.
#[derive(Debug, Clone)]
pub enum Wait {
    /// "کون سا؟": the matches of this `find` result.
    Which {
        /// The id of the `find` result, for example `r1`.
        find_id: String,
    },
    /// The partner code: these tool calls were paused, in order.
    Partner {
        /// The calls not yet run, the one that paused first.
        pending: Vec<ToolCall>,
    },
}

/// A question that waits.
#[derive(Debug, Clone)]
pub struct Waiting {
    /// The row of the log that is waiting.
    pub question_id: i64,
    /// What for.
    pub wait: Wait,
}

/// One conversation.
#[derive(Debug, Clone, Default)]
pub struct Conversation {
    /// The id the panel holds.
    pub id: String,
    /// The questions, oldest first. The last one is the one being answered.
    pub turns: Vec<Turn>,
    /// Every tool result, with the question (turn index) it belongs to. Ids run on.
    pub results: Vec<ToolResult>,
    /// The question that waits, if any.
    pub waiting: Option<Waiting>,
}

impl Conversation {
    /// A new, empty conversation.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_owned(),
            ..Self::default()
        }
    }
}
