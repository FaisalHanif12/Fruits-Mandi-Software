//! The commands that ask a question and the reply they all give (PRD FR-7.72 to FR-7.82, D6
//! Section 2, plan 48 Section 4).
//!
//! `assistant_ask` sends a typed question to the model. `assistant_quick` answers a chip with no
//! model (FR-7.80). `assistant_pick` goes on after "کون سا؟" or after the partner code (FR-7.77,
//! FR-7.82). `assistant_rerun` reads an old answer again on today's books (FR-7.84). All four
//! give an [`AskReply`] and none writes to the books; each writes one row of the question log.
//!
//! The app language is not sent with a question: the backend reads the saved language
//! preference (`prefs_set`, PRD FR-7.30), the same one the panel's language switch writes.
//!
//! Example (worked day): `assistant_ask {"text": "ikram ke total bakaya kitne hai",
//! "conversation_id": null}` gives outcome `answered`, one `name`, one `text` and one `figure`
//! part (288000 paisa, side `dr`, checked), the R13 table, and `count_today` 1 of `limit` 20.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::answer::{Answer, AnswerOption, EntryRoute};

/// How a question ended (PRD FR-7.84). The log stores the same word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AskOutcome {
    /// A checked answer.
    Answered,
    /// An answer with an estimate figure (the amber band shows).
    Estimate,
    /// An answer with advice (the advice band shows).
    Advice,
    /// Two to five names fit: the panel shows a button for each (`options`).
    WhichOne,
    /// The period is not one the app knows: ask for the dates.
    NeedDates,
    /// The question needs a party and names none.
    NeedParty,
    /// No name fits: the panel shows up to 3 closest names (`options`).
    NotFound,
    /// The question is outside the books.
    OutsideBooks,
    /// The Owner asked for a change; the assistant only tells (`entry_route`).
    ChangeRefused,
    /// The model's reply could not be used, or it said it could not tell what was asked, or a limit
    /// stopped the question (then `stop_reason` says which).
    NotUnderstood,
    /// The answer failed its checks twice: the panel shows the tool results as plain tables.
    ChecksFailed,
    /// A figure's two places differ: the answer is refused (final, no rewrite).
    RecordsDiffer,
    /// A partner figure is needed and the section is locked: the question waits for the code.
    Locked,
    /// 20 questions are done today.
    LimitReached,
    /// There is no internet.
    Offline,
    /// The model could not answer (see `failure`).
    Error,
}

/// Why a model call or the connection test failed. The panel has one reviewed message for each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    /// No internet (the request could not be sent).
    Offline,
    /// No reply within 15 seconds, or the question went past 45.
    Timeout,
    /// HTTP 401: the key is wrong.
    BadKey,
    /// HTTP 402: no credit left on the key.
    NoCredit,
    /// HTTP 404: no Azure endpoint matches the provider rules (zero retention, no fallback), for
    /// example a model that Azure does not serve.
    NoEndpoint,
    /// HTTP 429: busy, try again.
    Busy,
    /// HTTP 5xx: the service failed.
    Server,
    /// A reply the app cannot use: bad JSON, a cut-off reply (`finish_reason` length), a reply
    /// too big.
    BadReply,
    /// Anything else.
    Other,
}

/// The failure behind outcome `error` or `offline`, and behind a failed connection test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct AssistantFailure {
    /// What kind of failure.
    pub kind: FailureKind,
    /// The HTTP status when there was a reply (401, 402, 404, 429, 500 ...). Null otherwise.
    #[ts(type = "number | null")]
    pub http_status: Option<i64>,
}

/// Why a question was stopped before it was answered (plan 51 B2). Only with outcome
/// `not_understood`. The panel then says the books could not be read in the time allowed and asks
/// the Owner to ask in smaller parts, instead of "I did not understand".
///
/// Example: the model used its 4 calls on tool refusals: `stop_reason` is `calls`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// The question used its model calls (4, and the one rewrite).
    Calls,
    /// The question used its token limit (input or output).
    Tokens,
}

impl StopReason {
    /// The word the question log keeps in `finish_reason`.
    pub fn word(self) -> &'static str {
        match self {
            StopReason::Calls => "call_limit",
            StopReason::Tokens => "token_limit",
        }
    }
}

/// What every ask command gives back (plan 48 Section 4).
///
/// Which fields are set for each outcome:
///
/// | outcome | `answer` | `options` | `entry_route` | `failure` |
/// |---|---|---|---|---|
/// | answered, estimate, advice | set | empty | null, or set when a change was also asked | null |
/// | which_one, not_found | null | the matches | null | null |
/// | need_dates, need_party, not_understood | null | empty | null | null |
/// | not_understood with `stop_reason` | null, or the plain tables of the results that were read | empty | null | null |
/// | outside_books | set (the nearest source) or null | empty | null | null |
/// | change_refused | set or null | empty | set | null |
/// | checks_failed, records_differ | set (tables, sources, differences) | empty | null | null |
/// | locked | null | empty | null | null (`locked` is true) |
/// | limit_reached | null | empty | null | null |
/// | offline, error | null | empty | null | set |
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AskReply {
    /// The row of the question log this reply is. `assistant_pick` and `assistant_rerun` take it.
    #[ts(type = "number")]
    pub question_id: i64,
    /// The conversation the panel keeps (PRD FR-7.78). The panel sends it with its next question.
    pub conversation_id: String,
    /// How the question ended.
    pub outcome: AskOutcome,
    /// The answer, or null (see the table above).
    pub answer: Option<Answer>,
    /// For `which_one` and `not_found`: up to 5 records.
    pub options: Vec<AnswerOption>,
    /// True when more than 5 names fit: the panel adds "اور بھی ہیں، کوڈ لکھیں" (PRD FR-7.77).
    pub more_options: bool,
    /// For `change_refused`: the entry screen to open, with the party chosen and no amount.
    pub entry_route: Option<EntryRoute>,
    /// True when the question waits for the partner code (outcome `locked`). After the code is
    /// given (`partners_code_unlock`), `assistant_pick` with `partner_code` goes on.
    pub locked: bool,
    /// For `error` and `offline`: why.
    pub failure: Option<AssistantFailure>,
    /// For `not_understood` after a limit: which one (plan 51 B2). Null for every other reply,
    /// including a `not_understood` where the model could not tell what was asked.
    pub stop_reason: Option<StopReason>,
    /// Today's counted questions, after this one (PRD FR-7.81). The panel shows "آج 5 / 20".
    #[ts(type = "number")]
    pub count_today: i64,
    /// The daily limit, 20.
    #[ts(type = "number")]
    pub limit: i64,
}

/// A typed question.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
pub struct AssistantAskInput {
    /// The question as typed, raw text in any script (no Roman-to-Urdu conversion). 1 to 500
    /// characters after trimming. Empty is `ERR_REQUIRED`, longer is `ERR_TEXT_TOO_LONG`, on
    /// `text`.
    pub text: String,
    /// The conversation of the panel, or null to start a new one.
    pub conversation_id: Option<String>,
}

/// The quick questions that need no internet and no model (PRD FR-7.80).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum QuickChip {
    /// آج کی بکری.
    TodaySales,
    /// آج کا کمیشن.
    TodayCommission,
    /// موجود روکڑ.
    CashInHand,
    /// گاہکوں کے ذمے.
    BuyersDues,
    /// کل اگراہی.
    TotalDues,
    /// ڈوبی رقم.
    DeadAmount,
    /// بیوپاریوں کو ادائیگی باقی.
    SellersPayable,
    /// پیشگی باقی.
    AdvancesOpen,
    /// باقی مال.
    StockLeft,
    /// کسی پارٹی کا کھاتہ: needs `party_id`.
    PartyLedger,
}

/// A quick question.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
pub struct AssistantQuickInput {
    /// Which chip.
    pub chip: QuickChip,
    /// The party for `party_ledger`, chosen with the account picker. Null for the other chips.
    /// A missing party for `party_ledger` is `ERR_REQUIRED` on `party_id`.
    #[ts(type = "number | null")]
    pub party_id: Option<i64>,
    /// The conversation of the panel, or null to start a new one. An addition to the plan, so a
    /// quick answer is part of the conversation and "aur pichle mahine?" can follow it
    /// (PRD FR-7.78).
    pub conversation_id: Option<String>,
}

/// How a paused question goes on.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PickChoice {
    /// The Owner chose one of the `options` ("کون سا؟", PRD FR-7.77).
    Match {
        /// The `key` of the option, for example `a20`.
        key: String,
    },
    /// The partner code was given (`partners_code_unlock` worked): run the paused tools and go
    /// on (PRD FR-7.82). If the section is still locked: `ERR_PARTNER_LOCKED`.
    PartnerCode,
}

/// A continuation of a question that is waiting.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
pub struct AssistantPickInput {
    /// The `question_id` of the reply that is waiting (outcome `which_one`, `not_found` or
    /// `locked`). `ERR_NOT_FOUND` on `question_id` if there is no such question;
    /// `ERR_ASSISTANT_NOT_WAITING` if it is not waiting for this choice.
    #[ts(type = "number")]
    pub question_id: i64,
    /// The choice.
    pub choice: PickChoice,
}

/// Reading an old answer again.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
pub struct AssistantRerunInput {
    /// The row of the log. `ERR_NOT_FOUND` on `question_id` if there is none.
    #[ts(type = "number")]
    pub question_id: i64,
}

/// Starting a new conversation (PRD FR-7.78).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
pub struct AssistantNewConversationInput {
    /// The conversation the panel leaves. Its memory is dropped at once. Null, or an id the app
    /// does not know, drops nothing; the new id is given either way.
    pub conversation_id: Option<String>,
}

/// What `assistant_new_conversation` gives: the panel forgets the old conversation (PRD FR-7.78).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ConversationStarted {
    /// The new conversation.
    pub conversation_id: String,
}
