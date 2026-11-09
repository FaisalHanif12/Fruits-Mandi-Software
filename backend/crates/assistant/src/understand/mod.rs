//! Step 1 of plan 55, "understand": what a question is about, before any model is called.
//!
//! The app reads the Owner's question itself, in Roman Urdu, Urdu letters, English or a mix, with
//! misspellings, and answers with a typed [`Understanding`]:
//!
//! - the **intent**: figure, list, print, how-to, navigate, forecast, what-if, advice, change
//!   request, not in the app, or unclear;
//! - the **parties**, each found in his real role (seller, buyer, partner, garden, trip, khata,
//!   staff), or not found (with the role that was asked), or found in another role, or several;
//! - the **items**, with plurals and spellings (apples, kele, saib, کیلے, mausami);
//! - the **period** as exact dates (aaj, kal, parson, is hafte, pichle mahine, October, "1 se 15
//!   tak", is saal, ab tak);
//! - the **figure** asked (balance, sale, commission, dues, payable, stock, cash ...) and whether
//!   two were asked together ("commission aur sale");
//! - for a print or an open-the-screen request, the **report** and its filters (plan 54 Section 6.2
//!   and 6.4), with a second report where the plan gives one; for a how-to, the **guide topic**;
//! - a **confidence** for each part, and one overall `sure` flag.
//!
//! `sure` means: the app understood the whole question, including a name that does not exist. It does
//! not decide who answers. The caller routes by intent (plan 55 Section 2). Not sure is a valid, safe
//! result: the model path takes over.
//!
//! The module only reads. It writes nothing, calls no model and never touches a key. It uses the
//! search index, the item and partner lists and the transliteration words of the services, through
//! their public functions (Code Standards B29). No money or quantity is worked out here (Code
//! Standards Q5).
//!
//! Example (worked day, today 10-10-2026): "faisal ki aaj ki bikri ka print" with Faisal a buyer is
//! `Print`, party Faisal as a buyer, period 10-10-2026, report R35 with `party` and `as_of`.

use chrono::NaiveDate;
use mandi_services::Context;
use mandi_services::auth::Actor;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;

mod change;
mod compose_change;
mod compose_common;
mod compose_help;
mod compose_missing;
mod compose_outside;
mod compose_print;
mod compose_route;
mod figure;
mod function_words;
mod items;
mod lexicon;
mod plan;
mod reads;
mod scan;
mod what;
mod when;
mod who;

pub use compose_change::compose_change;
pub use compose_common::Composed;
pub use compose_help::{compose_help, compose_navigate};
pub use compose_missing::compose_missing;
pub use compose_outside::{compose_outside, outside_message};
pub use compose_print::compose_print;
pub use compose_route::compose;
pub use figure::{FigureAnswer, Sentence, compose_figure};
pub use plan::{PlanFilters, ReportPlan};
pub use reads::{PlannedRead, plan_pre_reads, plan_reads};

/// How sure the app is of one part of the understanding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    /// A guess, or the parts disagree.
    Low,
    /// Read from the words, with one weaker sign.
    Medium,
    /// Read from clear words, and checked against the books where that applies.
    High,
}

/// One part of the understanding and how sure the app is of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part<T> {
    /// What was understood.
    pub value: T,
    /// How sure.
    pub confidence: Confidence,
}

/// What the question wants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    /// One figure (or two): "faisal ka bakaya kitna hai".
    Figure,
    /// A list or a table: "faisal ne kya kya khareeda", "top 5 buyers".
    List,
    /// A printout: "faisal ki aaj ki bikri ka print".
    Print,
    /// How to do something in the app, or what a screen is: "wasooli kaise likhun".
    HowTo,
    /// Open a screen or a report: "dashboard kholo", "faisal ka khata kholo".
    Navigate,
    /// A figure ahead in time: "agle mahine commission kitna hoga".
    Forecast,
    /// A figure under a condition: "agar raftar double ho to".
    WhatIf,
    /// Advice: "sale kaise barhayen", "kisko udhar na dun".
    Advice,
    /// An instruction to write: "5000 jama karo", "wasooli likh do".
    ChangeRequest,
    /// Something the app does not keep: weather, GST, a bank balance, the rate in Lahore.
    NotInApp,
    /// The app could not tell.
    Unclear,
}

/// What a party, a record or a person is in the books.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartyRole {
    /// A seller (بیوپاری).
    Seller,
    /// A buyer (گاہک).
    Buyer,
    /// A partner (شراکت دار).
    Partner,
    /// A garden (باغ).
    Garden,
    /// A market trip (مارکیٹ سفر).
    Trip,
    /// A الگ کھاتہ.
    Khata,
    /// A staff member or any person who borrows (ادھار رقم).
    Staff,
}

impl PartyRole {
    /// The role as the reply of the ask commands names it (`AskReply.asked_role`, `FoundAs.role`).
    /// A khata or a person who borrowed is "any other record" there.
    ///
    /// Example: `PartyRole::Buyer.asked_role()` is `AskedRole::Buyer`.
    pub fn asked_role(self) -> crate::ask::AskedRole {
        use crate::ask::AskedRole;
        match self {
            PartyRole::Seller => AskedRole::Seller,
            PartyRole::Buyer => AskedRole::Buyer,
            PartyRole::Partner => AskedRole::Partner,
            PartyRole::Garden => AskedRole::Garden,
            PartyRole::Trip => AskedRole::Trip,
            PartyRole::Khata | PartyRole::Staff => AskedRole::Any,
        }
    }
}

/// A record found in the books. The key is the one `find` gives: `a12` for an account, `s3` for a
/// partner, `g2` for a garden, `t5` for a trip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyHit {
    /// The key.
    pub key: String,
    /// The id of the record (the account, the partner, the garden or the trip).
    pub id: i64,
    /// What he is in the books.
    pub role: PartyRole,
    /// His code, empty for a partner.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The tag after the name, if any.
    pub tag: Option<String>,
}

/// An item found in the books.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemHit {
    /// The key, `i7`.
    pub key: String,
    /// The id.
    pub id: i64,
    /// The code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// True for an item sold by weight: its rate is per kg, not per نگ.
    pub sold_by_weight: bool,
}

/// What a name led to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome<T> {
    /// Exactly one record, in the role asked (or in no particular role).
    Found(T),
    /// Exactly one record, but in another role than the one asked: a buyer asked as a seller.
    FoundAs(T),
    /// More than one record. The list is the first five.
    Several(Vec<T>),
    /// No record has this name.
    NotFound,
}

impl<T> Outcome<T> {
    /// The one record, whether found in the role asked or in another.
    pub fn hit(&self) -> Option<&T> {
        match self {
            Outcome::Found(hit) | Outcome::FoundAs(hit) => Some(hit),
            Outcome::Several(_) | Outcome::NotFound => None,
        }
    }
}

/// A person or a place the question names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyRef {
    /// The words as typed (normalised).
    pub text: String,
    /// The first and the after-last word of the name in the question.
    pub words: (usize, usize),
    /// The role the question asked for, from its words ("seller", "becha", "khareeda"), if it said.
    pub asked_role: Option<PartyRole>,
    /// What the books say.
    pub outcome: Outcome<PartyHit>,
    /// How sure.
    pub confidence: Confidence,
}

/// The kinds of record besides parties and items that a question can name and the app resolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    /// An expense head (بجلی).
    ExpenseHead,
    /// A city.
    City,
    /// A truck stand (اڈا).
    Adda,
    /// A staff member who has no account of his own (a collector, a munshi, a labourer): `st5`.
    Staff,
}

/// A record of another master list found in the books. The key is the one `find` gives: `eh3`
/// for an expense head, `ct4` for a city, `ad5` for an اڈا.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordHit {
    /// Which list.
    pub kind: RecordKind,
    /// The key.
    pub key: String,
    /// The id.
    pub id: i64,
    /// The code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
}

/// An expense head, a city or an اڈا the question names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordRef {
    /// The words as typed (normalised).
    pub text: String,
    /// The first and the after-last word of the name in the question.
    pub words: (usize, usize),
    /// What the books say.
    pub outcome: Outcome<RecordHit>,
    /// How sure.
    pub confidence: Confidence,
}

/// What a question outside the books is about: the app says plainly that it does not keep it, with
/// the nearest thing it does keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutsideKind {
    /// The weather.
    Weather,
    /// GST or any tax.
    Tax,
    /// A bank balance.
    Bank,
    /// The rate of goods in another market ("lahore mein kele ka rate", "mandi ka rate").
    MarketRate,
    /// The dollar, gold, petrol, the stock market.
    Prices,
    /// Anything else the app does not keep (news, cricket, a joke).
    Other,
}

/// What the app chose because the question did not say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assumed {
    /// The measure of a ranking: total commission (view H, PRD 18.2).
    Measure,
    /// The period of a ranking: this year.
    Period,
}

/// An item the question names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRef {
    /// The words as typed (normalised).
    pub text: String,
    /// The first and the after-last word of the name in the question.
    pub words: (usize, usize),
    /// What the books say.
    pub outcome: Outcome<ItemHit>,
    /// How sure.
    pub confidence: Confidence,
}

/// The kinds of period a question can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodKind {
    /// آج.
    Today,
    /// کل in a past question.
    Yesterday,
    /// پرسوں in a past question.
    DayBeforeYesterday,
    /// کل in a future question.
    Tomorrow,
    /// پرسوں in a future question.
    DayAfterTomorrow,
    /// This week, Monday to today.
    ThisWeek,
    /// Last week, Monday to Sunday.
    LastWeek,
    /// Next week.
    NextWeek,
    /// The 1st to today.
    ThisMonth,
    /// The whole of last month.
    LastMonth,
    /// The whole of next month.
    NextMonth,
    /// 1 January to today.
    ThisYear,
    /// The whole of last year.
    LastYear,
    /// The whole of next year.
    NextYear,
    /// A month by name.
    Month {
        /// The year.
        year: i32,
        /// 1 to 12.
        month: u32,
    },
    /// One day by its date.
    Date,
    /// From one date to another.
    Range,
    /// The last N days, up to today.
    LastDays(u32),
    /// The last N weeks, up to today.
    LastWeeks(u32),
    /// The last N months, up to today.
    LastMonths(u32),
    /// The last N years, up to today.
    LastYears(u32),
    /// This month and the following ones, N months in all.
    NextMonths(u32),
    /// This month and the following ones, N years in all.
    NextYears(u32),
    /// From today for N days (or N weeks as days).
    NextDays(u32),
    /// A whole year by its number (2025), or up to today for this one.
    Year(i32),
    /// From the first day of the books to today.
    UpToNow,
    /// From the first day of the books up to and including a day the question named ("kal tak",
    /// "30 september tak").
    UpToDay,
    /// From today to 31 December.
    YearEnd,
}

/// A period as exact dates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeriodRef {
    /// Which kind.
    pub kind: PeriodKind,
    /// The first day. `None` only when the books hold nothing yet (`UpToNow`, `UpToDay`).
    pub from: Option<NaiveDate>,
    /// The last day.
    pub to: NaiveDate,
    /// How sure.
    pub confidence: Confidence,
    /// True when the period lies after today.
    pub future: bool,
    /// The words as typed (normalised).
    pub words: String,
}

impl PeriodRef {
    /// The first day as `YYYY-MM-DD`, or empty.
    pub fn from_iso(&self) -> String {
        self.from
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default()
    }

    /// The last day as `YYYY-MM-DD`.
    pub fn to_iso(&self) -> String {
        self.to.format("%Y-%m-%d").to_string()
    }
}

/// The figure a question asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FigureKind {
    /// A party's balance (bakaya, lena, dena).
    Balance,
    /// Sales, the amount (bikri).
    Sale,
    /// How many بکری (bikriyan).
    SaleCount,
    /// Commission.
    Commission,
    /// لاگا.
    Laga,
    /// What a buyer bought (khareeda).
    Purchases,
    /// The shop's dues from buyers (ugrahi).
    Dues,
    /// Money received (wasooli hui).
    Receipts,
    /// What the shop owes sellers (adaigi baqi).
    Payable,
    /// Advances to sellers (peshgi).
    Advance,
    /// Goods not sold (baqi maal).
    Stock,
    /// Cash in hand (roqar).
    Cash,
    /// A partner's capital (sarmaya).
    Capital,
    /// A partner's total (kul raqam).
    PartnerTotal,
    /// A partner's share (hissa).
    PartnerShare,
    /// Profit or loss (munafa).
    Profit,
    /// Expenses (kharcha).
    Expense,
    /// Fruit taken (phal).
    FruitTaken,
    /// Money lent (udhar raqam).
    MoneyLent,
    /// The dead amount (doobi raqam).
    DeadAmount,
    /// A rate (rate, bhao).
    Rate,
    /// The summary of a day (khulasa).
    DaySummary,
    /// A party's ledger (khata, hisab).
    Ledger,
    /// A party's statement (fard hisab).
    Statement,
    /// The day book (roznamcha).
    DayBook,
    /// The trial balance.
    TrialBalance,
    /// The result of a garden or a trip.
    Result,
    /// The money the partners took for themselves, or gave back (ذاتی خرچہ, "li hui raqam").
    PartnerTaken,
    /// A report named by its title that no figure above names (PRD 19.2): the report code, for
    /// example `R4` for "udhari bikri". Only a print or an open request uses it.
    Report(&'static str),
    /// A question about the app's own state or lists, by the read it needs: `rules`, `attention`,
    /// `days`, `staff_munshi`, `staff_collector`, `staff_labour`, `staff_all`, `heads`.
    Ask(&'static str),
}

/// What a list is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListOver {
    /// Items.
    Items,
    /// Buyers.
    Buyers,
    /// Sellers.
    Sellers,
    /// Parties of any side.
    Parties,
}

/// Highest or lowest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    /// Sab se zyada, top.
    Most,
    /// Sab se kam, lowest.
    Least,
}

/// What a list question asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ListAsk {
    /// What it is over, if it said.
    pub over: Option<ListOver>,
    /// Highest or lowest, if it said.
    pub rank: Option<Rank>,
    /// "top 5": how many, if it said.
    pub n: Option<u32>,
}

/// Why the app is not (fully) sure. Each is a reason for the model path to take over, or for the
/// caller to ask the Owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Doubt {
    /// The question is empty, or too long to read.
    NothingToRead,
    /// A name matched more than one record.
    SeveralMatches,
    /// A name is close to a record but not the same.
    CloseSpelling,
    /// A name that may be a party or an item matched both.
    NameOrItem,
    /// Words the app does not know stand where a name should be.
    UnknownWords,
    /// More than one period was named (a comparison).
    SeveralPeriods,
    /// A print or report needs a day or a range and the question gave none.
    NoPeriod,
    /// A print or figure needs a party and the question named none.
    NoParty,
    /// No figure, screen or topic could be read.
    NoFigure,
    /// No topic of the guide fits.
    NoTopic,
    /// No report fits this request.
    NoReport,
    /// Two kinds of question are mixed.
    MixedIntent,
    /// The question compares two things, or asks for a share or a change.
    Comparison,
    /// A partner is named while the partner section is locked.
    PartnerLocked,
    /// A read of the books failed.
    LookupFailed,
    /// The question was cut for length.
    Cut,
    /// The meaning of کل / kal depends on a verb that is not clear.
    TenseUnclear,
    /// A time word stands that no period took ("pichli", "hafte mein", "din"): when the question
    /// means is not clear.
    TimeWord,
    /// A name is not a party or an item but another record (an expense head, a city, an اڈا).
    OtherRecord,
    /// Several things and several figures were asked at once ("kela ke kitne commission or sale,
    /// saib ke kitne"): a table of answers, for the model path.
    TwoParts,
}

/// Whether a word of the question names another measure than the one the reads hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measure {
    /// The measure the reads hold, or none is named.
    Same,
    /// A quantity (nag): the same views hold it, so the reads are the same, but the amount is no answer.
    Quantity,
    /// A breakdown ("kis kis din kitna commission diya"): the same views hold it, so the reads are
    /// the same, but one total is no answer to it.
    Breakdown,
    /// Another number altogether: the model's, with no reads made for it.
    Other,
}

/// A group of items a word names: fruit (phal) or vegetables (sabzi).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupWord {
    /// پھل, phal, fruit.
    Fruit,
    /// سبزی, sabzi, vegetables.
    Vegetable,
}

impl GroupWord {
    /// The word the tools take for the group (`fruit` or `vegetable`).
    pub fn tool_word(self) -> &'static str {
        match self {
            GroupWord::Fruit => "fruit",
            GroupWord::Vegetable => "vegetable",
        }
    }
}

/// The quality of a sale line a word names: awwal (first), doyam (second), soyam (third); PRD FR-12.42.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradeWord {
    /// اول, awwal, first quality, A grade.
    First,
    /// دوم, doyam, second quality, B grade.
    Second,
    /// سوم, soyam, third quality, C grade.
    Third,
}

impl GradeWord {
    /// The word the tools take for the quality (`first`, `second` or `third`).
    pub fn tool_word(self) -> &'static str {
        match self {
            GradeWord::First => "first",
            GradeWord::Second => "second",
            GradeWord::Third => "third",
        }
    }
}

/// Which side's commission a question asks (PRD 18.2: the seller side, `seller_comm`, or the buyer side,
/// `buyer_comm`, of a line). A question that names no side, or both, has none and the total is its
/// figure. D7 (plan 59): "Faisal se kele par kitna commission liya" is the buyer side, 7,586, and not
/// the total of both sides and Laga, 15,716.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommissionSide {
    /// The commission charged to the seller (`seller_comm`).
    Seller,
    /// The commission charged to the buyer (`buyer_comm`), without Laga.
    Buyer,
}

/// How a sale was paid, when a question asks the sales of one payment type ("udhaar me kitni bikri",
/// "naqdi bikri"): the cash sales (R3) or the credit sales (R4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayKind {
    /// نقدی: paid on the spot.
    Cash,
    /// ادھاری: paid later.
    Credit,
}

/// The way capital moves in a question about it: put into the shop, or taken back out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapitalFlow {
    /// سرمایہ جمع: invested, put in.
    In,
    /// سرمایہ واپس: taken back, withdrawn.
    Out,
}

/// A side of the trial balance a question asks for (D7, plan 59 F3b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialSide {
    /// بنام: the total debit of R18.
    Debit,
    /// جمع: the total credit of R18.
    Credit,
}

/// One part of the cash book a question asks for: what came in, what went out, what was left (D7, plan 59).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CashDir {
    /// The cash received over the period (R12 `cash_in`).
    In,
    /// The cash paid out over the period (R12 `cash_out`).
    Out,
    /// The cash left at the end of the period (R12 closing).
    Closing,
    /// The cash the period started with. Not answered by the app alone yet.
    Opening,
}

/// What the app understood of one question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Understanding {
    /// What the question wants.
    pub intent: Part<Intent>,
    /// Every person or place named, in the order of the question.
    pub parties: Vec<PartyRef>,
    /// Every item named.
    pub items: Vec<ItemRef>,
    /// Every expense head, city or اڈا named.
    pub records: Vec<RecordRef>,
    /// Every period named. One for most questions.
    pub periods: Vec<PeriodRef>,
    /// The figures asked, in the order of the question. Empty when none.
    pub figures: Part<Vec<FigureKind>>,
    /// What the app chose because the question did not say (a ranking with no measure or period).
    /// The answer says so: "is saal, کل کمیشن کے حساب سے".
    pub assumed: Vec<Assumed>,
    /// For `NotInApp`: what the question is about.
    pub outside: Option<OutsideKind>,
    /// True when two figures were asked in one breath ("commission aur sale").
    pub two_figures: bool,
    /// True when a figure asked is dropped before the plan sees it: a cue of tier 1 that names ANOTHER figure
    /// than the plainest cue ("dukan ka kharcha aur wasooli"), the same figure named twice with an "and"
    /// ("akram ka bakaya aur kul bakaya"), or collected and still to collect together. The app gives no instant
    /// answer and the model answers (D7, plan 59 C).
    pub lower_ask: bool,
    /// True when a "so far" was folded into another period that ends today ("fruit so far and today's sales"):
    /// with several figures asked, which period belongs to which figure is not known, so the model answers.
    pub period_merged: bool,
    /// True when a word for the whole shop or a total stands in it ("kul", "total", "dukan", "sab"): beside a
    /// named party or item and several figures, it would be read as that party's figure.
    pub shop_word: bool,
    /// True when ONE figure for several things was asked ("dono ka mila kar").
    pub sum_asked: bool,
    /// True when the question compares, or asks for a share or a change.
    pub compare: bool,
    /// For a list: what it is over and how it is ranked.
    pub list: Option<ListAsk>,
    /// For a print or an open-the-report request: the report and its filters (plan 54 6.2). Always
    /// `None` when `sure` is false: a question that is not sure never has a button (a wrong, wider date
    /// on a button is worse than none).
    pub report: Option<ReportPlan>,
    /// For a how-to: the topic of the guide.
    pub topic: Option<Part<&'static str>>,
    /// For navigate: the screen name of the answer schema, when one fits. For a change request: the
    /// entry screen that does it (`wasooli`, `peshgi`, `kharcha` ...), when the question says
    /// plainly which; `None` when it could be two or none. The amount is never kept.
    pub screen: Option<&'static str>,
    /// True when the question also asks how to do it ("kaise nikalun"): the steps are wanted.
    pub wants_steps: bool,
    /// True when the question names a garden, a trip, own goods or the guests by word. The reads
    /// of [`plan_reads`] have no filter for them, so such a question is the model's.
    pub scoped: bool,
    /// True when the question says plainly that the parts are to be added ("mila kar", "dono ka",
    /// "combined"), not only a bare "kul" or "total".
    pub sum_strong: bool,
    /// True when the question holds an amount (a target: "5 lakh tak kab pohanchenge"). An estimate
    /// with a target is the model's to set up.
    pub amount_named: bool,
    /// The group of items the question names with a word ("sabzi ki bikri", "phal ka commission"),
    /// when no item is named. The views and reports of the app have no filter for a group yet, so
    /// such a question is the model's, except the rules of a group (`app_state` takes the word).
    pub item_group: Option<GroupWord>,
    /// True when a packing word stands as a filter of a flow ("kele ki peti ki bikri"): no read has a
    /// packing filter, so such a question is the model's.
    pub packing_filter: bool,
    /// True when the question names a document by its number ("wasooli number 3"): no read of a figure has
    /// a filter for it, so the question is the model's.
    pub document_number: bool,
    /// The بکری number the question names ("bikri 5 mein kisne kya badla").
    pub bikri_number: Option<i64>,
    /// A truck number the question holds, written `LES-1234`: the sales of that truck are found by
    /// `find` with the role `truck`, then read with the `report` tool (R1 and its `bikri` number).
    pub truck: Option<String>,
    /// True when the question names the shop's guests (mehman): the fruit register has a read for
    /// them (R37 with the partner `guests`).
    pub guests: bool,
    /// True when the question names the quality of a sale line (awwal, doyam, soyam, A grade, first
    /// quality; PRD FR-12.42). The figure of an item or a party adds all its qualities, so it is no
    /// answer for one of them: such a question has no instant figure, and its read is the lines of the
    /// sales (R2), which carry the quality.
    pub quality: bool,
    /// Which quality it names, when one word does (awwal, doyam, soyam and their spellings). `None`
    /// with `quality` true asks all the qualities ("kis darje mein bike"): the lines carry each one.
    pub grade: Option<GradeWord>,
    /// For a question about capital: whether it asks what came in or what was taken back.
    pub capital_flow: Option<CapitalFlow>,
    /// The parts of the cash book the question names, in the order it names them ("cash came in, went out
    /// and was left": In, Out, Closing). Empty when it names none.
    pub cash_dirs: Vec<CashDir>,
    /// The number of the بکری the question names ("bikri number 100 me kitna maal bika"): the model reads R1 of
    /// it; the app reads it ahead (D7, plan 59 D). No figure is given from it by the app.
    pub sale_number: Option<i64>,
    /// The sides of the trial balance the question names, in the order it names them ("trial balance ka banaam
    /// total": Debit). Empty when it names none.
    pub trial_sides: Vec<TrialSide>,
    /// True when the question names the gardens' goods with no garden named ("baghat ki bikri"): the
    /// goods filter `garden` of view A reads them.
    pub garden_goods: bool,
    /// True when the question asks for the last one ("aakhri", "last", "pichla") and names no period:
    /// the last rate of an item sold to a buyer is the newest line of his sales (R2).
    pub last_one: bool,
    /// True when the question asks in how many months something sold ("mooli kitne mahine biki hai"):
    /// the item's sales by month (view D) hold the answer.
    pub months_asked: bool,
    /// For that question: the first day of the books and today, ISO. "Kitne mahine" with no period asks
    /// all the books, so view D is read for these days, never for the tool's own default period.
    pub months_range: Option<(String, String)>,
    /// True when the question asks whom to collect from first ("kis se pehle wasooli karon"): the
    /// buyers' dues, oldest first (R20 sorted by days), are the answer.
    pub oldest: bool,
    /// True when the question asks what was given, paid or received (an event of a period), not what
    /// is open now ("rahim ko kitni peshgi di"). A party's card holds only what is open, so such a
    /// question is the model's.
    pub happened: bool,
    /// Whether a word of the question names another measure than the reads hold (a nag count, a net
    /// sale, a percent, a partner's خالص, "udhar" for credit): the plain figure would be a real
    /// number for the wrong thing, so the question is the model's (money review, 05-11-2026).
    pub other_measure: Measure,
    /// The side of the commission the question asks, when it names one (a buyer or a seller word beside
    /// the commission, "gahak se commission", or one named buyer): the figure is that side's column, never
    /// the total. `None` when no side is named, both are, or the question holds a Laga too.
    pub commission_side: Option<CommissionSide>,
    /// True when the question asks how many of something ("the number of bills", "kitne bayopariyon ko"):
    /// an instant figure that holds no count leaves that part out, so it goes to the model.
    pub asks_count: bool,
    /// True when the question says it is the BUYER's rate ("gahak ka rate", "buyer rate"). A rate or a last
    /// rate is the seller's rate (ریٹ بیوپاری, the auction rate) unless it says this (lead ruling, D7).
    pub buyer_rate: bool,
    /// How many "how much" and "how many" cues stand in the question, each place once ("kitna maal liya,
    /// kitne ka, aur kitna commission" is 3). An instant answer with fewer figures than cues leaves a part
    /// out, so it goes to the model (D7, plan 59 F3b, the independent check).
    pub how_much_cues: usize,
    /// How many counts the question asks ("kitne bill aur kitne gahak" is 2). An instant answer that holds
    /// fewer count lines leaves a part out.
    pub count_asks: usize,
    /// True when the question asks the quantity of an item, a party or the shop in { نگ } ("kitne nag bika",
    /// "how many units"): the sale figure is read as its `nag` column. Never set when a packing or a weight
    /// word is beside it (D7, plan 59 S-A7).
    pub qty_asked: bool,
    /// True when the question says the AVERAGE ("ausat", "average"): an item's rate is then view A's average
    /// rate, never the last rate (D7, plan 59 S-A1).
    pub average_asked: bool,
    /// The payment type of the sales asked ("udhaar", "naqdi"), when the question names ONE (D7, S-A8).
    pub pay_kind: Option<PayKind>,
    /// True when a payment-type word stands in the question (cash, credit, naqdi, udhari beside a sale),
    /// even when it names both. The plain figures of the reads cannot answer such a question: only the
    /// sales of one payment type can (`pay_kind`), else the model does.
    pub pay_asked: bool,
    /// True when a word of the question is explained by nothing: it is in no table, names no party, item or
    /// record and is no function word. The figures that the reports hold as totals (an expense, a receipt)
    /// are not given by the app alone then, because the word may be a filter the reads do not have ("chai"
    /// of "chai ka kharcha").
    pub unexplained: bool,
    /// True when a word for buyers or sellers stands in the question ("gahakon se wasooli", "beopariyon
    /// ko"): a report that lists every party cannot be split by it, so its total is no answer.
    pub role_word: bool,
    /// True when a SELLERS word stands in it (بیوپاری, byopari, seller): the buyers of a city are not his figure.
    pub seller_word: bool,
    /// True when a BUYERS word stands in it (گاہک, gahak, buyer). With a sellers word too and one figure, the
    /// app would answer for one side and leave the other out.
    pub buyer_word: bool,
    /// True when a question word that asks a date, a person, a place, a reason or a way stands in the question
    /// ("kab se", "kis kis ka", "kahan se", "since when", "who"): a figure alone leaves that ask out, so the app
    /// gives no instant answer and the model answers (D7, plan 59 C).
    pub open_ask: bool,
    /// True when a word stands in the question that is known only as a part of a screen's name ("agrahi" of
    /// "agrahi list") and no screen is named: the word names no figure, so a figure answered beside it would
    /// leave it out ("kul agrahi aur doobi raqam"). The app gives no instant answer then.
    pub stray_screen_word: bool,
    /// The word position of each figure of `figures`, in the same order.
    pub figure_at: Vec<usize>,
    /// The word spans of the names that were found (parties, items, records).
    pub name_spans: Vec<(usize, usize)>,
    /// True when a period's words stand between the first and the last figure asked ("kul bikri aur aaj ka
    /// commission"): the period may be the second figure's only.
    pub period_between: bool,
    /// True when a word for the shop or a total stands after the last found name and after an "and" ("akram ka
    /// aur kul bakaya"): the figure asked is the shop's, not the named party's.
    pub shop_after_and: bool,
    /// True when several different measures asked in one question may be answered by the app alone: the gate
    /// [`MULTI_MEASURE_INSTANT`], which the caller may override (a test switch).
    pub multi_measure: bool,
    /// True when two "how much" words stand in the question ("kharcha kitna hua aur sharkat daron ne kitna
    /// liya"): it asks two things, and the figures of a report's totals answer one.
    pub how_much_twice: bool,
    /// True when the question asks the VALUE of the fruit taken ("phal ki qeemat"), not its quantity.
    pub value_asked: bool,
    /// True when the dues asked are the total dues ("kul ugrahi", "total dues": the tile کل اگراہی,
    /// buyers' dues plus sellers' dues plus the dead amount), not the buyers' dues alone.
    pub total_dues: bool,
    /// True when the app understood the whole question. The caller routes by intent.
    pub sure: bool,
    /// Why the app is not fully sure, or what it noted.
    pub doubts: Vec<Doubt>,
}

impl Understanding {
    /// The first period, if any.
    pub fn period(&self) -> Option<&PeriodRef> {
        self.periods.first()
    }

    /// The first figure, if any.
    pub fn figure(&self) -> Option<FigureKind> {
        self.figures.value.first().copied()
    }

    /// The first party named, if any.
    pub fn party(&self) -> Option<&PartyRef> {
        self.parties.first()
    }

    /// True when a name was asked for and no record has it.
    pub fn has_not_found(&self) -> bool {
        self.parties
            .iter()
            .any(|p| matches!(p.outcome, Outcome::NotFound))
    }
}

/// Whether a question that asks two or more DIFFERENT measures ("aaj ki bikri aur commission", "bakaya aur last
/// bikri") is answered by the app alone, one labelled line for each (plan 59, item C).
///
/// OFF by lead decision D333: PRD FR-7.90 sends two-part questions to the model. The code stays, behind this
/// gate, so that Umar's yes is a change of this one line. One measure for several named parties, items or
/// periods ("akram aur arshad ka bakaya") is not a two-measure question and is not gated.
pub const MULTI_MEASURE_INSTANT: bool = false;

/// What the understanding reads: the books through the services, in the Owner's session.
pub struct Reader<'a> {
    /// The services context (the database, the search index).
    pub ctx: &'a Context,
    /// The signed-in Owner.
    pub actor: &'a Actor,
    /// The app language (it decides nothing about the words; the question may be in any language).
    pub language: Language,
    /// Today, the Pakistan date.
    pub today: BusinessDate,
    /// True while the partner section is locked (PRD FR-7.82): no partner is looked up.
    pub partner_locked: bool,
}

impl Reader<'_> {
    fn today_date(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.today.to_iso(), "%Y-%m-%d").ok()
    }
}

/// Understands one question. Only reads. See the module documentation.
///
/// Example: "faisal ka bakaya kitna hai" is `Figure`, the figure `Balance`, the party Faisal.
pub fn understand(reader: &Reader<'_>, question: &str) -> Understanding {
    let Some(today) = reader.today_date() else {
        return what::nothing(Doubt::LookupFailed);
    };
    let scan = scan::Scan::new(question, today);
    if scan.words.is_empty() {
        return what::nothing(Doubt::NothingToRead);
    }
    let read = when::read_periods(&scan, today);
    let mut periods = read.periods;
    fill_start_of_books(reader, &mut periods);
    let names = who::resolve(reader, &scan, &periods);
    // A cue inside a name that was found ("Imran Fruit", "Akram Toll Tax") is part of the name.
    let scan = scan.without_spans(&names.spans);
    what::assemble(
        reader,
        &scan,
        names,
        periods,
        read.stray,
        read.so_far_merged,
    )
}

/// Builds the word table and the typing word list now, so the first question of the Owner is not
/// the slow one (about 85 ms in a debug build, once). Call it at start-up. Reads only.
pub fn warm_up(reader: &Reader<'_>) {
    let _ = understand(reader, "faisal ka bakaya kitna hai");
}

/// Builds the tables of words that need no database: the cues, the period words, the guide and the
/// function words, so the first question of the Owner is not the slow one. Needs no signed-in
/// session, so the app calls it at start-up, next to `translit::warm_up` (which loads the typing
/// word list).
pub fn warm_words() {
    let Some(day) = NaiveDate::from_ymd_opt(2026, 1, 1) else {
        return;
    };
    let sample = "faisal ka bakaya kitna hai pichle mahine ki wasooli kaise likhun";
    let scan = scan::Scan::new(sample, day);
    let _ = when::periods(&scan, day);
    let _ = crate::prompt::guide();
    let _ = crate::prompt::words();
    let _ = crate::help_guard::topic_of_question(sample, &[]);
}

/// "Ab tak" starts at the first day of the books, when there is one.
fn fill_start_of_books(reader: &Reader<'_>, periods: &mut [PeriodRef]) {
    for period in periods
        .iter_mut()
        .filter(|p| matches!(p.kind, PeriodKind::UpToNow | PeriodKind::UpToDay))
    {
        period.from = mandi_services::day::first_book_day(reader.ctx)
            .ok()
            .flatten()
            .and_then(|d| NaiveDate::parse_from_str(&d.to_iso(), "%Y-%m-%d").ok());
    }
}
