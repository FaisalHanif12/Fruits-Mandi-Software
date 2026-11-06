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
/// tokenizer counts it (the model counts about 3.4% more). 11,600 since NFR v1.16 NFR-AI-5 and Code
/// Standards v1.20 Q9 (Umar, 06-11-2026): the item group filter of the report and view tools. It was
/// 11,500 since NFR v1.15 and Code Standards v1.19 (D6-system-prompt v1.5: the tenth tool
/// `app_state`, the estimate with `head`, `partner` and `party`, ten more entry screens and the
/// `audit` topic; 11,494 measured). It was 11,000 in v1.3 (Umar, 02-11-2026, plan 52): 6 calls and
/// 12 tools, forgiving tools, typed names, general advice, one-call help and the whole-year
/// estimate. It was 9,500 in v1.2: the tools list the sort and where words of every report and
/// view, and the tool `app_help` comes with the help button.
pub const MAX_PREFIX_TOKENS: u32 = 11_600;
/// Seconds one try of a model call may wait for the reply (plan 54 S5, plan 55 §3): the first try
/// is cut at 12 s, as the model takes 4 to 9 s a call on a good line, and it is tried once more.
/// The reply of the model comes whole, so this is the wait for the first byte of it.
pub const FIRST_BYTE_SECS: u64 = 12;
/// Seconds after which the last call of a question goes out with the tools off, and answers from
/// what has been read (plan 55 §3).
pub const TOOLS_OFF_AT_SECS: u64 = 20;
/// Seconds one whole question may take (plan 55 §3: 25; it was 90, NFR-AI-3). At 25 s the panel
/// shows what was read as plain tables (outcome `time_limit`), and no model call is made.
pub const QUESTION_TIMEOUT_SECS: u64 = 25;
/// Milliseconds to wait before the one retry of a call that failed on the network, on 429 or on a
/// 5xx (plan 52 item 6). The client may say another time (the tests say none).
pub const RETRY_WAIT_MILLIS: u64 = 1_500;
/// Seconds of the question that must be left for a retry to be tried. A retry that cannot finish
/// is not started.
pub const RETRY_MIN_LEFT_SECS: u64 = 6;

/// When no reply has come by now, a second identical request is sent and the first reply to come
/// is used (the hedge, plan 55 Section 3; live run 1: 90% of short calls end in 6 s, 94% in 8 s).
pub const HEDGE_AFTER_SECS: u64 = 7;

/// The copy is sent only when at least this long is left for it.
pub const HEDGE_MIN_COPY_SECS: u64 = 3;
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
