//! The limits of one question, as named constants (Code Standards Q9, G8, NFR-AI-5).
//!
//! Example: a question may use 4 model calls and 1 rewrite, 8 tool calls, 3,000 output tokens
//! (1,500 a call) and 60,000 input tokens, within 30 seconds a call and 60 seconds in all.

/// Counted questions a day, a hard stop (PRD FR-7.81). The day starts at midnight, Pakistan time.
pub const DAILY_LIMIT: i64 = 20;
/// Model calls of one question, not counting the rewrite.
pub const MAX_MODEL_CALLS: u32 = 4;
/// Extra model calls allowed to rewrite an answer that failed its checks (PRD FR-7.76).
pub const MAX_REWRITES: u32 = 1;
/// Tool calls of one question.
pub const MAX_TOOL_CALLS: u32 = 8;
/// Output tokens of one question, reasoning tokens included.
pub const MAX_OUTPUT_TOKENS: u32 = 3_000;
/// Output tokens of one model call.
pub const MAX_CALL_OUTPUT_TOKENS: u32 = 1_500;
/// Input tokens of one question, cached tokens included.
pub const MAX_INPUT_TOKENS: u32 = 60_000;
/// The fixed part of a request (tools, instructions, answer schema), in tokens. 9,500 since
/// D6-system-prompt v1.2 (Umar, 02-11-2026): the tools list the sort and where words of every
/// report and view, and the tool `app_help` and its rules come with the schema of the help button.
pub const MAX_PREFIX_TOKENS: u32 = 9_500;
/// Seconds one model call may take. 30 since Umar's answer of 02-11-2026 (plan 51 B3): the model
/// takes 7 to 9 seconds a call on his questions, and 15 cut off a slow one while it was answering.
pub const CALL_TIMEOUT_SECS: u64 = 30;
/// Seconds one whole question may take (NFR-AI-3: 60 since 02-11-2026).
pub const QUESTION_TIMEOUT_SECS: u64 = 60;
/// Bytes of JSON of one tool result; rows are dropped from the end to fit.
pub const MAX_RESULT_BYTES: usize = 16_000;
/// Words of an answer, counted after the references are filled, each reference as one word.
pub const MAX_ANSWER_WORDS: usize = 130;
/// Rows of a table in an answer (PRD A82).
pub const MAX_TABLE_ROWS: usize = 7;
/// Tables and charts one answer may show.
pub const MAX_SHOW: usize = 3;
/// Suggested next questions of one answer.
pub const MAX_NEXT: usize = 3;
/// Matches offered after "کون سا؟" (PRD FR-7.77).
pub const MAX_OPTIONS: usize = 5;
/// Months ahead an estimate may look (PRD FR-7.86).
pub const MAX_MONTHS_AHEAD: u32 = 12;
/// Earlier questions of the conversation that are sent with a new one (PRD FR-7.78).
pub const CONVERSATION_QUESTIONS: usize = 3;
/// Rows of one page of the question log (Code Standards B31).
pub const LOG_PAGE_ROWS: i64 = 100;
/// Longest question, in characters after trimming (command input, Code Standards 4.2).
pub const MAX_QUESTION_CHARS: usize = 500;
