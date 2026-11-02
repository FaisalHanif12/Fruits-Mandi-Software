//! The limits of one question, as named constants (Code Standards Q9, G8, NFR-AI-5).
//!
//! Example: a question may use 6 model calls and 1 rewrite, 12 tool calls, 4,000 output tokens
//! (1,500 a call) and 90,000 input tokens, within 30 seconds a call and 90 seconds in all. The
//! last call always answers: it goes out with the tools switched off (plan 52 item 1).

/// Counted questions a day, a hard stop (PRD FR-7.81). The day starts at midnight, Pakistan time.
pub const DAILY_LIMIT: i64 = 20;
/// Model calls of one question, not counting the rewrite. 6 since Umar's answer of 02-11-2026
/// (plan 52, NFR-AI-5): the last of them has the tools switched off, so it always answers.
pub const MAX_MODEL_CALLS: u32 = 6;
/// Extra model calls allowed to rewrite an answer that failed its checks (PRD FR-7.76).
pub const MAX_REWRITES: u32 = 1;
/// Tool calls of one question (12 since plan 52).
pub const MAX_TOOL_CALLS: u32 = 12;
/// Output tokens of one question, reasoning tokens included (4,000 since plan 52).
pub const MAX_OUTPUT_TOKENS: u32 = 4_000;
/// Output tokens of one model call.
pub const MAX_CALL_OUTPUT_TOKENS: u32 = 1_500;
/// Input tokens of one question, cached tokens included (90,000 since plan 52).
pub const MAX_INPUT_TOKENS: u32 = 90_000;
/// The fixed part of a request (tools, instructions, answer schema), in tokens, as the `o200k_base`
/// tokenizer counts it (the model counts about 3.4% more). 11,000 since D6-system-prompt v1.3
/// (Umar, 02-11-2026, plan 52): 6 calls and 12 tools, forgiving tools, typed names, general advice,
/// one-call help and the whole-year estimate. It was 9,500 in v1.2: the tools list the sort and
/// where words of every report and view, and the tool `app_help` comes with the help button.
pub const MAX_PREFIX_TOKENS: u32 = 11_000;
/// Seconds one model call may take. 30 since Umar's answer of 02-11-2026 (plan 51 B3): the model
/// takes 7 to 9 seconds a call on his questions, and 15 cut off a slow one while it was answering.
pub const CALL_TIMEOUT_SECS: u64 = 30;
/// Seconds one whole question may take (NFR-AI-3: 90 since plan 52; 60 before).
pub const QUESTION_TIMEOUT_SECS: u64 = 90;
/// Milliseconds to wait before the one retry of a call that failed on the network, on 429 or on a
/// 5xx (plan 52 item 6). The client may say another time (the tests say none).
pub const RETRY_WAIT_MILLIS: u64 = 1_500;
/// Seconds of the question that must be left for a retry to be tried. A retry that cannot finish
/// is not started.
pub const RETRY_MIN_LEFT_SECS: u64 = 20;
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
