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
    /// True when the app ran the tool itself, after a `find` with exactly one match (the
    /// model did not ask for it, plan 52 P52-S1).
    pub prefetch: bool,
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
    /// For a call that failed: what kind of failure (`offline`, `timeout`, `server` ...), so a
    /// run of the real model shows which stage failed (plan 52 item 6).
    pub error: Option<String>,
    /// For the rewrite: the rules of the checks that the first answer failed (the words of
    /// `check.rs`, no text and no figure), so the log shows why a rewrite was needed.
    pub check_failed: Vec<String>,
    /// For an answer whose typed names the app put back as references instead of asking for a
    /// rewrite (plan 52 P52-S1): the rules it repaired (the word `name`), so the log shows it.
    pub repaired: Vec<String>,
    /// For the rewrite: the paths of the references that failed (`reference`, `earlier_question`).
    /// A path holds no name and no figure; it shows why the first answer was refused.
    pub check_paths: Vec<String>,
    /// For the rewrite: what the other refused rules were about, as `rule:piece`: the number word,
    /// the Latin letter, the never-use word, the month name, the path of a `show`, `choose` or
    /// `next` (plan 52 P52-S1, item 9). Never a name and never a digit.
    pub check_at: Vec<String>,
    /// For a call that was sent twice (the hedge, plan 55 Section 3): which request this entry
    /// describes, `first` or `copy`. `None` for a call sent once.
    pub hedged: Option<&'static str>,
    /// True for the entry of a hedged call that the question did not wait for: the request that
    /// lost, or the copy when both failed. It is in the log, and left out of the time sums, since
    /// its time is inside the time of the other one.
    pub hedge_extra: bool,
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
            .filter(|c| !c.hedge_extra)
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
                let mut entry = serde_json::json!({
                    "n": i.saturating_add(1),
                    "model_ms": c.model_millis,
                    "tokens_in": c.tokens_in,
                    "tokens_cached": c.tokens_cached,
                    "tokens_out": c.tokens_out,
                    "tokens_reasoning": c.tokens_reasoning,
                    "rewrite": c.rewrite,
                    "failed": c.failed,
                    "tools": c.tools.iter()
                        .map(|t| {
                            let mut tool = serde_json::json!({"name": t.name, "ms": t.millis, "ok": t.ok});
                            if let (true, Some(map)) = (t.prefetch, tool.as_object_mut()) {
                                map.insert("prefetch".to_owned(), serde_json::json!(true));
                            }
                            tool
                        })
                        .collect::<Vec<_>>(),
                });
                // Only when there is something to say, so a normal call's entry stays as it was.
                if let (Some(error), Some(map)) = (&c.error, entry.as_object_mut()) {
                    map.insert("error".to_owned(), serde_json::json!(error));
                }
                if let (Some(side), Some(map)) = (c.hedged, entry.as_object_mut()) {
                    map.insert("hedged".to_owned(), serde_json::json!(side));
                    if c.hedge_extra {
                        map.insert("hedge_extra".to_owned(), serde_json::json!(true));
                    }
                }
                if !c.check_failed.is_empty()
                    && let Some(map) = entry.as_object_mut()
                {
                    map.insert("check_failed".to_owned(), serde_json::json!(c.check_failed));
                }
                if !c.check_paths.is_empty()
                    && let Some(map) = entry.as_object_mut()
                {
                    map.insert("check_paths".to_owned(), serde_json::json!(c.check_paths));
                }
                if !c.check_at.is_empty()
                    && let Some(map) = entry.as_object_mut()
                {
                    map.insert("check_at".to_owned(), serde_json::json!(c.check_at));
                }
                if !c.repaired.is_empty()
                    && let Some(map) = entry.as_object_mut()
                {
                    map.insert("repaired".to_owned(), serde_json::json!(c.repaired));
                }
                entry
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
    /// A find whose matches the Owner must choose from (none, several or only a close spelling),
    /// set while a reply's tools run so that the question ends at once with "کون سا؟" or "not
    /// found" and no model call is spent on it (plan 52 item 8, PRD FR-7.87).
    pub ask_which: Option<String>,
    /// What the app read ahead after a find (plan 52 P52-S1): the tool, its input as the model
    /// would send it (without the empty fields), and the id of the result. A later call of the
    /// model with the same tool and input gets that result back and reads nothing again.
    pub prefetched: Vec<(String, Value, String)>,
    /// The reads the app planned for this question when it understood it (`understand::plan_reads`):
    /// the tool and its input. The scope guard compares the views and reports an answer cites with
    /// them, and reads them again when an answer cited others.
    pub planned: Vec<(String, Value)>,
    /// The party the app found itself before the first call (plan 52 P52-S1, item 10). The answer
    /// always shows his name when it is about him.
    pub prefound: Option<crate::answer::AnswerName>,
    /// True when the question is general mandi advice (`help_guard::general_advice_asked`): its
    /// calls go out with the tools off and an answer of kind `advice` that reads nothing is
    /// general advice.
    pub general_advice: bool,
    /// The typed-name finds (`f1` ...) that matched none, several, or only a close spelling: they
    /// are kept for the Owner's choice and are never read, in this reply or a later one.
    pub unclear: Vec<String>,
    /// A find that found only parties of the other role than the question's verb gives ("Faisal ne
    /// kya becha", and he is a buyer): the find's id and the role the verb asks for. The question
    /// ends as not found with `found_as` (plan 52 Section 8).
    pub wrong_role: Option<(String, crate::ask::AskedRole)>,
    /// True when the app read the books before the first call (plan 54 S3) or gave the guide
    /// (help): the data is in the call, so the model has only to write, and thinks less (S4).
    pub read_first: bool,
    /// The side of the commission the question asks, when the app understood that (D7, plan 59 F1): a
    /// commission figure of the other column is not shown (`measure_guard`).
    pub commission_side: Option<crate::understand::CommissionSide>,
    /// True when the question also asks a Laga (with a commission side): the total of both sides and
    /// Laga is never rewritten to one side's figure.
    pub commission_laga: bool,
    /// The lot the question is about (plan 71 L9, PRD FR-7.77), as its key (`l7`): the only lot the
    /// app found for a seller and an item the question names, or the one the Owner picked. The
    /// model's own `report` calls of that seller and item are narrowed to it.
    pub lot: Option<String>,
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
