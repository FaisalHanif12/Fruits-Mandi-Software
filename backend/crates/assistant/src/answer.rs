//! The answer the panel shows (PRD FR-7.74, FR-7.75, D6 Section 2).
//!
//! Everything in an [`Answer`] is built by the app from tool results. The model only chose the
//! order of the text, which results to show, and the next questions; it never typed a figure, a
//! name or a date (Code Standards Q4). The screen shows these values and nothing else: it never
//! sums, rounds, or builds a figure (Code Standards F2, M12).
//!
//! Money is always a report [`Cell`] (paisa, whole rupees), so the screen draws a figure with the
//! same cell renderer the report screens use.
//!
//! Example (worked day, "ikram ke total bakaya kitne hai"): parts are
//! `name اکرم ٹول ٹیکس (0020)`, `text " کے ذمے "`, `figure balance 288000 paisa, side dr,
//! checked`, `text " باقی ہیں۔"`, and one table of R13 with its last rows.

use mandi_reports::analytics::{AnalyticsKey, AnalyticsView};
use mandi_reports::reports::{Cell, Column, Names, ReportId, Row};
use serde::Serialize;
use ts_rs::TS;

use crate::source::{AnswerRecord, SourceAnalyticsFilters, SourceReportFilters};

/// The direction word the app puts next to a difference or a change: زیادہ (more), کم (less) or
/// برابر (the same). The model never chooses it (PRD FR-7.75).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum FigureDirection {
    /// More than before: زیادہ.
    Up,
    /// Less than before: کم.
    Down,
    /// No change: برابر.
    Same,
}

/// A record's name in both scripts, for `displayName` (the tag goes after it), and its code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnswerName {
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The tag after the name (PRD 10.2). Null for none.
    pub tag: Option<String>,
    /// The account code, for example `0020`. Null for a record with no code (an item, a month).
    pub code: Option<String>,
}

/// What a calculation step does. The panel writes the inputs with the sign of the operation and
/// the result after `=`; it works nothing out (PRD FR-7.43).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum WorkingOp {
    /// `a + b + c`.
    Sum,
    /// `first − second`.
    Difference,
    /// `(a + b + c) ÷ n`.
    Average,
    /// `first ÷ second × 100`, a percent.
    PercentOf,
    /// `(second − first) ÷ first × 100`, a percent.
    ChangePercent,
    /// `per month × months` (the total of an estimate).
    Times,
    /// `first ÷ second` (the change for each month of the same growth: 15,000 ÷ 5 = 3,000).
    DividedBy,
}

/// One step of a working: its inputs in order and its result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct WorkingStep {
    /// What the step does.
    pub op: WorkingOp,
    /// The inputs: money, balance, number, percent or month cells.
    pub inputs: Vec<Cell>,
    /// The result, in the kind of the first input (or percent).
    pub result: Cell,
}

/// How a calculator or estimator figure was worked out, built by the same function that works the
/// figure out (Code Standards B30). `title_id` is a Fluent message that names it, for example
/// `assistant-working-sum`; an estimate has one step for the average and one for the total.
///
/// Example: sum of two balances: one step, `sum`, inputs 288000 and 1868400 paisa, result 2156400.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnswerWorking {
    /// Fluent message ID of the heading of the working.
    pub title_id: String,
    /// The steps, in order.
    pub steps: Vec<WorkingStep>,
}

/// One piece of the answer text, in order (D6 Section 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnswerPart {
    /// The model's own words, shown as they are. Never drawn as a number (D6 Section 2).
    Text {
        /// The words.
        text: String,
    },
    /// A figure the app filled in: money, a balance with its side, a count, a percent.
    Figure {
        /// The value. A balance has its side inside the cell.
        cell: Cell,
        /// For a difference or a change: its direction word. Null for any other figure.
        dir: Option<FigureDirection>,
        /// True when the figure's two places agree: the screen shows ✓ (PRD FR-7.76). False for
        /// a figure that has no second place, and for a number read from the question.
        checked: bool,
        /// The working, when the figure came from the calculator or the estimator (ⓘ).
        working: Option<AnswerWorking>,
    },
    /// A record's name, drawn through `displayName` with its tag.
    Name {
        /// The name.
        name: AnswerName,
    },
    /// A date, `YYYY-MM-DD`, drawn as DD-MM-YY.
    Date {
        /// The date.
        iso: String,
    },
    /// A month, drawn in the language of the app, for example ستمبر 2026.
    Month {
        /// The year, for example 2026.
        #[ts(type = "number")]
        year: i64,
        /// The month, 1 to 12.
        #[ts(type = "number")]
        month: i64,
    },
    /// An entry's label, for example بکری 1 or وصولی 4: its kind as a Fluent message ID and its
    /// number, as a report `doc` cell.
    Doc {
        /// Fluent message ID of the kind, for example `report-doc-bikri-first`.
        type_id: String,
        /// The number. Null if the source has none.
        #[ts(type = "number | null")]
        number: Option<i64>,
    },
}

/// A source the app read for this question. It gives the source line and the button "پوری رپورٹ
/// کھولیں" with the same filters (PRD FR-7.74). A report or an analytics source can also be printed
/// with the same filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnswerSource {
    /// The party card of FR-7.38. The button opens the party's ledger (R13).
    PartyCard {
        /// The result of this question it came from, for example `r2`.
        result_id: String,
        /// The account, as a whole record (a picker value).
        party: AnswerRecord,
    },
    /// A report R1 to R38.
    Report {
        /// The result of this question it came from, for example `r3`.
        result_id: String,
        /// Fluent message ID of the report's title.
        label_id: String,
        /// The report.
        id: ReportId,
        /// The filters it was read with; each record is a whole record (id, code, names, tag), so
        /// the screen opens the report with the pickers already filled.
        filters: SourceReportFilters,
    },
    /// An analytics view A to K.
    Analytics {
        /// The result of this question it came from.
        result_id: String,
        /// Fluent message ID of the view's title.
        label_id: String,
        /// The view.
        view: AnalyticsView,
        /// The filters it was read with; each record is a whole record.
        filters: SourceAnalyticsFilters,
    },
    /// The dashboard tiles (PRD 9.1).
    Tiles {
        /// The result of this question it came from.
        result_id: String,
    },
}

/// One row of an answer table: the report's own [`Row`] (cells, the record it opens, how it is
/// drawn), and for a row of an analytics view the key that opens the lines behind it (PRD
/// FR-18.2). The screen draws `row.cells` with the report screens' own cell renderer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnswerRow {
    /// The report's own row. An analytics row is given in this shape too, with `source` null.
    pub row: Row,
    /// The lines behind the row (a row of an analytics view). Null for a report row.
    pub analytics_key: Option<AnalyticsKey>,
}

/// A table the model chose to show: at most 7 rows of a report or view with that source's own
/// columns, and its totals over the whole filter (PRD FR-7.74, FR-19.11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnswerTable {
    /// The result of this question it came from, for example `r3`. It links to an
    /// [`AnswerSource`].
    pub result_id: String,
    /// Fluent message ID of the table's title.
    pub title_id: String,
    /// The columns.
    pub columns: Vec<Column>,
    /// The rows shown, at most 7.
    pub rows: Vec<AnswerRow>,
    /// How many rows the whole filter has.
    #[ts(type = "number")]
    pub count: i64,
    /// Fluent message ID of the totals row label.
    pub totals_label_id: String,
    /// The totals row over the whole filter, one cell for each column. Never a sum of the rows
    /// shown.
    pub totals: Vec<Cell>,
}

/// A month of a year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct AnswerMonth {
    /// The year, for example 2026.
    #[ts(type = "number")]
    pub year: i64,
    /// The month, 1 to 12.
    #[ts(type = "number")]
    pub month: i64,
}

/// What a line of a chart is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ChartSeriesKind {
    /// The months of the books, as they were.
    Actual,
    /// An estimate by the same pace.
    SamePace,
    /// An estimate by the same growth.
    SameGrowth,
    /// An estimate by the same season.
    SameSeason,
}

/// One line of a chart. `values` has one cell for each month of the chart: money or number, or
/// `empty` where the line has no value (an estimate line has none for the past months).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ChartSeries {
    /// What the line is.
    pub kind: ChartSeriesKind,
    /// The values, in the order of the chart's months.
    pub values: Vec<Cell>,
}

/// A small chart: a monthly series from the books, or an estimate with the months ahead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnswerChart {
    /// The result of this question it came from, for example `r4` or `e1`.
    pub result_id: String,
    /// Fluent message ID of what is plotted, for example `assistant-measure-commission-all`.
    pub measure_id: String,
    /// The months on the axis, oldest first.
    pub months: Vec<AnswerMonth>,
    /// The lines.
    pub series: Vec<ChartSeries>,
    /// True for an estimate chart: the amber band shows with it (PRD FR-7.86).
    pub is_estimate: bool,
}

/// How one method of PRD FR-7.86 worked out the months ahead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EstimateMethodKind {
    /// اسی رفتار سے: the average of the last 3 full months.
    SamePace,
    /// اسی اضافے سے: the last full month plus the average change over the last 6 full months.
    SameGrowth,
    /// پچھلے سال جیسا: the same months of last year (needs 12 full months).
    SameSeason,
}

/// One method of an estimate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EstimateMethod {
    /// Which method.
    pub kind: EstimateMethodKind,
    /// How many full months it used.
    #[ts(type = "number")]
    pub months_used: i64,
    /// The first of the months it used.
    pub used_from: AnswerMonth,
    /// The last of the months it used.
    pub used_to: AnswerMonth,
    /// The figure for each month (same pace). Null for a method with a different step.
    pub per_month: Option<Cell>,
    /// The change for each month (same growth), with its direction in `step_dir`.
    pub step: Option<Cell>,
    /// The direction of `step`.
    pub step_dir: Option<FigureDirection>,
    /// The total of the months ahead.
    pub total: Cell,
    /// With a target: the month the running total reaches it. Null when it does not within the
    /// months ahead (and no target was asked: see [`AnswerEstimate::target`]).
    pub target_month: Option<AnswerMonth>,
    /// The working, with ⓘ.
    pub working: AnswerWorking,
}

/// What an estimate was and how it was worked out (PRD FR-7.86). Present only when an estimate
/// figure is shown. The amber band "اندازہ، یقینی نہیں" shows with it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnswerEstimate {
    /// The result of this question it came from, for example `e1`.
    pub result_id: String,
    /// Fluent message ID of what is estimated, for example `assistant-measure-commission-all`.
    pub measure_id: String,
    /// How many full months the books gave.
    #[ts(type = "number")]
    pub full_months: i64,
    /// The first month ahead.
    pub first: AnswerMonth,
    /// The last month ahead.
    pub last: AnswerMonth,
    /// The target the Owner typed ("3 lakh"), as a plain amount. Null when none was asked.
    pub target: Option<Cell>,
    /// The methods that could be worked out, in the order same pace, same growth, same season.
    pub methods: Vec<EstimateMethod>,
}

/// One value of a figure in one of its two places.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PlaceValue {
    /// Fluent message ID of the place, for example `assistant-place-party-card` or
    /// `assistant-place-r13-closing`.
    pub place_id: String,
    /// The figure in that place.
    pub cell: Cell,
}

/// A figure whose two places differ (PRD FR-7.76 point 2). The answer is refused; the panel shows
/// "ریکارڈ میں فرق ملا، رپورٹ کھول کر دیکھیں" with the two values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FigureDifference {
    /// Fluent message ID of what the figure is, for example `assistant-figure-party-balance`.
    pub figure_id: String,
    /// The party, when it is a party's balance.
    pub name: Option<AnswerName>,
    /// Its values, one for each place.
    pub places: Vec<PlaceValue>,
}

/// A suggestion or a refusal route: the entry screen that opens with the party chosen and every
/// amount empty (PRD Section 7.7, FR-7.79). The assistant never writes. A help answer (PRD FR-7.89)
/// may point to any main screen instead: the screen opens as it is, and `party` is null. The routes
/// of the main screens: `dashboard` `/`, `settings` `/settings`, `masters` `/index`, `accounts`
/// `/index/accounts`, `commission_rules` `/index/rules`, `today_sales` `/bikri/today`,
/// `unsold_stock` `/bikri/stock`, `pukhta_khata` `/bikri/pukhta`, `collection_list`
/// `/bikri/collection`, `cash_book` `/rokar`, `gardens` `/gardens`, `market_trips` `/trips`,
/// `partners` `/partners`, `month_close` `/month`, `day_close` `/day`, `reports` `/reports`,
/// `dead_amount` `/reports/dead-amount`, `analytics` `/analytics`, `backup` `/backup`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EntryScreen {
    /// بکری بک.
    Bikri,
    /// وصولی.
    Wasooli,
    /// ادائیگی.
    Adaigi,
    /// پیشگی.
    Peshgi,
    /// خرچہ.
    Kharcha,
    /// ادھار رقم.
    UdharRaqam,
    /// الگ کھاتہ.
    AlagKhata,
    /// سرمایہ.
    Sarmaya,
    /// پھل.
    Phal,
    /// جنرل.
    General,
    /// The account form.
    Account,
    /// ڈیش بورڈ.
    Dashboard,
    /// ترتیبات.
    Settings,
    /// انڈکس.
    Masters,
    /// The accounts list.
    Accounts,
    /// کمیشن اصول.
    CommissionRules,
    /// آج کی بکری.
    TodaySales,
    /// باقی مال.
    UnsoldStock,
    /// پختہ کھاتہ.
    PukhtaKhata,
    /// اگراہی فہرست.
    CollectionList,
    /// روکڑ بک.
    CashBook,
    /// باغات.
    Gardens,
    /// مارکیٹ سفر.
    MarketTrips,
    /// شراکت دار.
    Partners,
    /// مہینہ بند.
    MonthClose,
    /// دن بند کریں.
    DayClose,
    /// رپورٹ.
    Reports,
    /// ڈوبی رقم.
    DeadAmount,
    /// تجزیات.
    Analytics,
    /// بیک اپ.
    Backup,
}

/// The screen a refused change request opens (outcome `change_refused`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EntryRoute {
    /// The entry screen.
    pub screen: EntryScreen,
    /// The party to choose on it, as a whole record (a picker value: id, code, both names, tag).
    /// Null for none.
    ///
    /// Which record it is depends on the screen:
    /// - `kharcha`, `sarmaya` and `phal`: a PARTNER record. `id` is `partners.id` and the names are
    ///   the partner's own; it is never his capital, drawings or 0604 account. A partner has no
    ///   code or tag, so those two are null.
    /// - Every other screen (`bikri`, `wasooli`, `adaigi`, `peshgi`, `udhar_raqam`, `alag_khata`,
    ///   `general`, `account`): the ACCOUNT record, with `id` = `accounts.id`.
    ///
    /// A record of the other kind is never sent: the party is then null and the screen opens with
    /// nothing chosen.
    pub party: Option<AnswerRecord>,
}

/// An answer (PRD FR-7.74). The order on the screen is: the parts, the tables and charts, the
/// bands, the buttons, the source line, the next questions.
///
/// For outcomes whose words the panel owns (`which_one`, `need_dates`, `need_party`,
/// `not_found`, `not_understood`, `limit_reached`, `offline`, `error`, `locked`) the whole
/// [`Answer`] is null in the reply. For `checks_failed` it holds the tool results as plain tables
/// and their sources. For `records_differ` it holds the sources and the differences. For
/// `outside_books` it holds the nearest source, if any.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Answer {
    /// The answer text, in order. Empty for the outcomes above.
    pub parts: Vec<AnswerPart>,
    /// Up to 3 tables and charts together (D6 Section 2); a table has at most 7 rows.
    pub tables: Vec<AnswerTable>,
    /// Charts.
    pub charts: Vec<AnswerChart>,
    /// The estimate behind any estimate figure or chart. The amber band shows when it is set.
    pub estimate: Option<AnswerEstimate>,
    /// True when the advice band "مشورہ، فیصلہ آپ کا" shows (PRD FR-7.88).
    pub advice: bool,
    /// Every source read, for the source line and the buttons.
    pub sources: Vec<AnswerSource>,
    /// Figures whose two places differ (only with outcome `records_differ`).
    pub differences: Vec<FigureDifference>,
    /// Up to 3 suggested questions, as plain text in the app language. A click sends one as a
    /// new question.
    pub next: Vec<String>,
    /// When the books were read, UTC, `YYYY-MM-DDTHH:MM:SSZ`.
    pub read_at: String,
}

/// A record offered after "کون سا؟" or when a name is not found (PRD FR-7.77).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnswerOption {
    /// The record's key, for example `a20`. `assistant_pick` sends it back.
    pub key: String,
    /// His name with the tag and code.
    pub name: AnswerName,
    /// His city. Null for none.
    pub city: Option<Names>,
    /// His balance with its side (a `balance` cell). Null for a record with no balance.
    pub balance: Option<Cell>,
    /// True when both places of the balance agree (✓).
    pub checked: bool,
    /// True for a person in the dead amount (PRD FR-10.10).
    pub dead: bool,
}
