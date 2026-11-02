//! The question log, page by page (PRD FR-7.84, Architecture 15.3, Code Standards B31).
//!
//! The log is append-only and part of every backup. It stores the question, the tools called with
//! their inputs, the answer as the model wrote it (with references, not figures), the outcome and
//! the tokens. The figures shown are not stored: `assistant_rerun` runs the tools again on today's
//! books.
//!
//! Example: the worked question is one row: source `model`, outcome `answered`, counted, tools
//! `find` and `party_card` and `report R13`, 3 model calls.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ask::AskOutcome;

/// What made the row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum LogSource {
    /// A quick question (a chip). No model; not counted.
    Chip,
    /// A question to the model. Counted.
    Model,
    /// A continuation after "کون سا؟" or the partner code. Not counted.
    Continue,
    /// An old answer read again. Not counted.
    Rerun,
    /// A connection test. Counted.
    Test,
}

/// One tool the model called (or a chip's fixed call).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LogTool {
    /// The tool: `find`, `party_card`, `report`, `analytics`, `tiles`, `period`, `calculate` or
    /// `estimate`.
    pub tool: String,
    /// The report id or view letter it read, for example `R13` or `A`. Null for the others.
    pub source: Option<String>,
}

/// One row of the log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AssistantLogRow {
    /// The row's id. `assistant_rerun` takes it.
    #[ts(type = "number")]
    pub id: i64,
    /// When it was asked, UTC, as `YYYY-MM-DDTHH:MM:SS.mmmZ`.
    pub asked_at: String,
    /// The Pakistan date it counts for, `YYYY-MM-DD`.
    pub asked_day: String,
    /// The question as typed. For a chip, the chip's name, for example `today_sales`.
    pub question: String,
    /// What made the row.
    pub source: LogSource,
    /// How it ended.
    pub outcome: AskOutcome,
    /// True when it counts in the daily 20.
    pub counted: bool,
    /// The question this row continues, or null.
    #[ts(type = "number | null")]
    pub parent_id: Option<i64>,
    /// The tools used, in order.
    pub tools: Vec<LogTool>,
    /// The model. Null for a chip.
    pub model: Option<String>,
    /// How many model calls it made. Null for a chip.
    #[ts(type = "number | null")]
    pub model_calls: Option<i64>,
    /// Input tokens, cached ones included. Null for a chip.
    #[ts(type = "number | null")]
    pub tokens_in: Option<i64>,
    /// Of those, the cached ones. Null for a chip.
    #[ts(type = "number | null")]
    pub tokens_cached: Option<i64>,
    /// Output tokens, reasoning included. Null for a chip.
    #[ts(type = "number | null")]
    pub tokens_out: Option<i64>,
    /// Of those, the reasoning ones. Null for a chip.
    #[ts(type = "number | null")]
    pub tokens_reasoning: Option<i64>,
    /// How long this run took in milliseconds, from the start to the reply (plan 51 B2). Null for
    /// a row written before the times were kept.
    #[ts(type = "number | null")]
    pub total_millis: Option<i64>,
    /// Of those, the time spent waiting for the model. Null for a chip and for an old row.
    #[ts(type = "number | null")]
    pub model_millis: Option<i64>,
    /// Of those, the app's own work in the tools. Null for a chip and for an old row.
    #[ts(type = "number | null")]
    pub tool_millis: Option<i64>,
    /// True when the row can be read again with `assistant_rerun`: it has tool calls.
    pub can_rerun: bool,
}

/// A page of the log, newest first.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AssistantLogListInput {
    /// Where the page starts. 0 for the first page.
    #[ts(type = "number")]
    pub offset: i64,
    /// How many rows. 0 means the default, 100. At most 100.
    #[ts(type = "number")]
    pub limit: i64,
}

/// A page of the log with the count over the whole log (Code Standards B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AssistantLogPage {
    /// The rows of this page, newest first.
    pub rows: Vec<AssistantLogRow>,
    /// Where this page starts.
    #[ts(type = "number")]
    pub offset: i64,
    /// How many rows were asked for.
    #[ts(type = "number")]
    pub limit: i64,
    /// How many rows the whole log has.
    #[ts(type = "number")]
    pub total_count: i64,
}
