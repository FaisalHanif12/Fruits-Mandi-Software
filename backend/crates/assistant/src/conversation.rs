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
