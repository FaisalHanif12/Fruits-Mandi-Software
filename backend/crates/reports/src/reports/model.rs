//! The types of the report engine (plan 20, PRD 19.1 to 19.5, 19.10, 19.11).
//!
//! One shape serves every report: a catalogue entry, a set of filters, and a page of typed rows
//! with the totals over the whole filter. The screen lays out what it receives and calculates
//! nothing (CLAUDE.md rule 1). Money travels as paisa (`Paisa`, whole rupees, a multiple of 100).
//! Titles and labels are Fluent message IDs; names travel in both scripts so the screen can use
//! `displayName` and a print can choose its language.

use mandi_core::accounts::BalanceSide;
use mandi_core::auth::Language;
use mandi_core::money::Paisa;
use mandi_core::printing::Paper;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The reports of the engine. The PRD IDs of 19.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub enum ReportId {
    /// Full sales register, مکمل بکری.
    #[serde(rename = "R2")]
    R2,
    /// Cash sales, نقدی.
    #[serde(rename = "R3")]
    R3,
    /// Credit sales, ادھاری.
    #[serde(rename = "R4")]
    R4,
    /// Seller summary, بکری تفصیل / روزنامچہ بکری.
    #[serde(rename = "R6")]
    R6,
    /// Item-wise sales, قسم مال وار بکری.
    #[serde(rename = "R7")]
    R7,
    /// Truck stand report, اڈا رپورٹ.
    #[serde(rename = "R8")]
    R8,
    /// One-time parties register, پختہ کھاتہ رپورٹ.
    #[serde(rename = "R9")]
    R9,
    /// Daily journal, روزنامچہ.
    #[serde(rename = "R11")]
    R11,
    /// Cash book, روکڑ.
    #[serde(rename = "R12")]
    R12,
    /// Party ledger, کھاتہ.
    #[serde(rename = "R13")]
    R13,
    /// Party statement, فرد حساب.
    #[serde(rename = "R14")]
    R14,
    /// Account index, انڈکس.
    #[serde(rename = "R16")]
    R16,
    /// Head balances, مدوں بیلنس.
    #[serde(rename = "R17")]
    R17,
    /// Trial balance, ٹرائل بیلنس.
    #[serde(rename = "R18")]
    R18,
    /// Opening trial, اوپن ٹرائل.
    #[serde(rename = "R19")]
    R19,
    /// Buyer dues, وصولی / اگراہی رپورٹ.
    #[serde(rename = "R20")]
    R20,
    /// Buyer dues by city, اگراہی بحساب شہر.
    #[serde(rename = "R21")]
    R21,
    /// Buyer dues by collector, اگراہی بحساب اگراہی والا.
    #[serde(rename = "R22")]
    R22,
    /// Seller payables, ادائیگی باقی.
    #[serde(rename = "R24")]
    R24,
    /// Advances, پیشگی کھاتہ.
    #[serde(rename = "R26")]
    R26,
    /// Bill series, بیجک سیریز.
    #[serde(rename = "R27")]
    R27,
    /// Income report, آمدن رپورٹ.
    #[serde(rename = "R28")]
    R28,
    /// Expense report, خرچہ رپورٹ.
    #[serde(rename = "R29")]
    R29,
    /// Profit and partner split, منافع و تقسیم.
    #[serde(rename = "R30")]
    R30,
    /// Partner statement, شراکت دار کھاتہ.
    #[serde(rename = "R31")]
    R31,
    /// Money lent, ادھار رقم رپورٹ.
    #[serde(rename = "R32")]
    R32,
}

impl ReportId {
    /// The PRD ID as text: `R11`.
    pub const fn code(self) -> &'static str {
        match self {
            ReportId::R2 => "R2",
            ReportId::R3 => "R3",
            ReportId::R4 => "R4",
            ReportId::R6 => "R6",
            ReportId::R7 => "R7",
            ReportId::R8 => "R8",
            ReportId::R9 => "R9",
            ReportId::R11 => "R11",
            ReportId::R12 => "R12",
            ReportId::R13 => "R13",
            ReportId::R14 => "R14",
            ReportId::R16 => "R16",
            ReportId::R17 => "R17",
            ReportId::R18 => "R18",
            ReportId::R19 => "R19",
            ReportId::R20 => "R20",
            ReportId::R21 => "R21",
            ReportId::R22 => "R22",
            ReportId::R24 => "R24",
            ReportId::R26 => "R26",
            ReportId::R27 => "R27",
            ReportId::R28 => "R28",
            ReportId::R29 => "R29",
            ReportId::R30 => "R30",
            ReportId::R31 => "R31",
            ReportId::R32 => "R32",
        }
    }

    /// The first part of a PDF file name: `report-r11`.
    pub fn file_stem(self) -> String {
        format!("report-{}", self.code().to_ascii_lowercase())
    }
}

/// The four groups of the catalogue (PRD 19.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ReportGroup {
    /// بکری reports.
    Sales,
    /// Journal, ledgers and trial balances.
    Accounts,
    /// Dues, payables and advances.
    Collections,
    /// Income, expenses and profit.
    Business,
}

/// Which filter boxes a report shows (PRD FR-19.1, FR-19.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename = "ReportFilterKind")]
pub enum FilterKind {
    /// From and to dates, with the date shortcuts (FR-19.2). Either may be empty.
    DateRange,
    /// One "as of" date, for a balance list (FR-19.10).
    AsOf,
    /// One account, and for a پختہ کھاتہ account one person. Required when the entry says so.
    Party,
    /// A seller.
    Seller,
    /// A زمیندار.
    Zamindar,
    /// An item.
    Item,
    /// A city.
    City,
    /// An اڈا.
    Adda,
    /// The "with detail" switch of the statement (R14): off shows balances only.
    Detail,
    /// Paid or due lines only (R9).
    Status,
    /// Group by day or by month (R28).
    GroupBy,
    /// One partner (R31).
    Partner,
}

/// One report in the catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportCatalogEntry")]
pub struct CatalogEntry {
    /// The PRD ID.
    pub id: ReportId,
    /// The group it is listed under.
    pub group: ReportGroup,
    /// Fluent message ID of the group name.
    pub group_title_id: String,
    /// Fluent message ID of the report title, for example `report-r11-title`.
    pub title_id: String,
    /// The filters it takes, in the order to show them.
    pub filters: Vec<FilterKind>,
    /// True when the party filter must be filled (R13, R14).
    pub party_required: bool,
}

/// The date shortcuts of FR-19.2. This week starts on Monday [A].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename = "ReportDateShortcut")]
pub enum DateShortcut {
    /// آج.
    Today,
    /// کل.
    Yesterday,
    /// یہ ہفتہ.
    ThisWeek,
    /// یہ مہینہ.
    ThisMonth,
    /// پچھلا مہینہ.
    LastMonth,
}

/// The party filter: one account, and for a پختہ کھاتہ account one person (PRD FR-13.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(rename = "ReportPartyFilter")]
pub struct PartyFilter {
    /// The account the user picked.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The person key inside a پختہ کھاتہ account. Missing means the whole account.
    #[serde(default)]
    pub person_key: Option<String>,
}

/// How the income report groups its lines (R28, PRD 19.2: "by day or month").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename = "ReportGroupBy")]
pub enum GroupBy {
    /// One period for each day.
    Day,
    /// One period for each month.
    #[default]
    Month,
}

/// The paid or due filter of the one-time parties register (R9, PRD FR-13.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename = "ReportPaidFilter")]
pub enum PaidFilter {
    /// Lines with nothing left to pay or collect.
    Paid,
    /// Lines with something left to pay or collect.
    Due,
}

/// What the user typed or picked in the filter boxes. Dates are text as typed
/// (`DD-MM-YYYY`, `10`, `10-9`, or the stored form `YYYY-MM-DD`); the backend reads them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, TS)]
#[serde(default)]
pub struct ReportFilters {
    /// A date shortcut. When set, `from` and `to` are ignored.
    pub shortcut: Option<DateShortcut>,
    /// First day, included. Empty means from the first day of data.
    pub from: String,
    /// Last day, included. Empty means up to the last day of data.
    pub to: String,
    /// The day of a balance list. Empty means up to now.
    pub as_of: String,
    /// The party.
    pub party: Option<PartyFilter>,
    /// A seller account.
    #[ts(type = "number | null")]
    pub seller_id: Option<i64>,
    /// A زمیندار account.
    #[ts(type = "number | null")]
    pub zamindar_id: Option<i64>,
    /// An item.
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// A city.
    #[ts(type = "number | null")]
    pub city_id: Option<i64>,
    /// An اڈا.
    #[ts(type = "number | null")]
    pub adda_id: Option<i64>,
    /// The statement with its detail lines (R14). Off shows balances only.
    pub detail: bool,
    /// The income report by day or by month (R28). Missing means month.
    #[ts(optional)]
    pub group_by: Option<GroupBy>,
    /// Paid or due lines only (R9). Missing means all lines.
    #[ts(optional)]
    pub status: Option<PaidFilter>,
    /// The partner of a partner statement (R31).
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
}

/// A name in both scripts. The screen picks one with `displayName`; a print picks by language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportNames")]
pub struct Names {
    /// Urdu name.
    pub ur: String,
    /// English name.
    pub en: String,
}

/// An account as a report shows it (PRD FR-19.13: short code and name with tag).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportParty")]
pub struct PartyRef {
    /// The account id, to open its ledger.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The full code, for example `03040001`.
    pub code: String,
    /// The short code of a seller or buyer (the serial, PRD FR-7.39). Null for other accounts.
    pub short_code: Option<String>,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The tag after the name (PRD 10.2). Null for none.
    pub tag: Option<String>,
    /// The person inside a پختہ کھاتہ account, as stored. Null for any other line.
    pub person_key: Option<String>,
}

/// The filters as the backend read them, echoed back (PRD FR-19.10). Dates are `YYYY-MM-DD`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportResolvedFilters")]
pub struct ResolvedFilters {
    /// The shortcut that was used, if any.
    pub shortcut: Option<DateShortcut>,
    /// First day, included. Null for open.
    pub from: Option<String>,
    /// Last day, included. Null for open.
    pub to: Option<String>,
    /// The day of a balance list. Null for up to now.
    pub as_of: Option<String>,
    /// The party, with names for the print header.
    pub party: Option<PartyRef>,
    /// The seller.
    pub seller: Option<PartyRef>,
    /// The زمیندار.
    pub zamindar: Option<PartyRef>,
    /// The item.
    pub item: Option<Names>,
    /// The city.
    pub city: Option<Names>,
    /// The اڈا.
    pub adda: Option<Names>,
    /// The partner of a partner statement (R31).
    pub partner: Option<Names>,
    /// The statement with its detail lines.
    pub detail: bool,
    /// The grouping of R28. Missing for any other report.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub group_by: Option<GroupBy>,
    /// The paid or due filter of R9. Missing for all lines.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub status: Option<PaidFilter>,
}

/// What kind of value a column holds. The screen picks the grid column type from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename = "ReportColumnKind")]
pub enum ColumnKind {
    /// Text as typed.
    Text,
    /// A Fluent message ID.
    Label,
    /// A document: its kind and number.
    Doc,
    /// A whole number.
    Number,
    /// Money in whole rupees.
    Money,
    /// Money with a side.
    Balance,
    /// A date.
    Date,
    /// A month of a year (the grouping of R28).
    Month,
    /// A share in hundredths of a percent (analytics, PRD 18.2): 7210 is 72.10%.
    Percent,
    /// An account.
    Party,
    /// A name in both scripts.
    Names,
    /// The items of a line.
    Detail,
    /// A sentence written from a Fluent message and its arguments (a partner statement line).
    Message,
}

/// One column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportColumn")]
pub struct Column {
    /// A stable key, for example `debit`.
    pub key: String,
    /// Fluent message ID of the heading.
    pub title_id: String,
    /// What it holds.
    pub kind: ColumnKind,
}

/// The goods or the reason behind a ledger line (PRD FR-19.8). Structured values, never a
/// sentence: the screen and the print put the words around them from Fluent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(rename = "ReportDetailPart")]
pub enum DetailPart {
    /// A sale line on a buyer's ledger: "کیلا 20 پیٹی @ 600 = 12,000، کمیشن 720، لاگا 160".
    Sale {
        /// The بکری number.
        #[ts(type = "number")]
        bikri_number: i64,
        /// The item.
        item: Names,
        /// The packing.
        packing: Names,
        /// The quantity, in نگ.
        #[ts(type = "number")]
        qty: i64,
        /// The buyer rate.
        #[ts(type = "Paisa")]
        rate_paisa: Paisa,
        /// The buyer value (quantity × rate).
        #[ts(type = "Paisa")]
        value_paisa: Paisa,
        /// The buyer commission.
        #[ts(type = "Paisa")]
        commission_paisa: Paisa,
        /// لاگا.
        #[ts(type = "Paisa")]
        laga_paisa: Paisa,
    },
    /// A seller credit: the lot's item, quantity, خام بکری and what was taken off it.
    Lot {
        /// The بکری number.
        #[ts(type = "number")]
        bikri_number: i64,
        /// The item.
        item: Names,
        /// The packing.
        packing: Names,
        /// The quantity sold in this posting.
        #[ts(type = "number")]
        qty: i64,
        /// خام بکری of this posting.
        #[ts(type = "Paisa")]
        gross_paisa: Paisa,
        /// The deductions: خام بکری less the seller credit posted (نقد is not in it).
        #[ts(type = "Paisa")]
        deductions_paisa: Paisa,
    },
    /// Cash lent to a buyer (PRD FR-15.11): "نقد ادھار" and its reason.
    CashLent {
        /// The reason as typed.
        reason: String,
    },
    /// A note or reason as typed.
    Text {
        /// The text.
        text: String,
    },
}

/// One cell of a row, in the kind of its column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(rename = "ReportCell")]
pub enum Cell {
    /// Nothing.
    Empty,
    /// Text as typed.
    Text {
        /// The text.
        text: String,
    },
    /// A Fluent message ID.
    Label {
        /// The ID.
        id: String,
    },
    /// A whole number.
    Number {
        /// The value.
        #[ts(type = "number")]
        value: i64,
    },
    /// Money in whole rupees.
    Money {
        /// The amount.
        #[ts(type = "Paisa")]
        paisa: Paisa,
    },
    /// Money with a side. A zero balance has the side `zero`.
    Balance {
        /// The amount, 0 or more.
        #[ts(type = "Paisa")]
        paisa: Paisa,
        /// The side.
        side: BalanceSide,
    },
    /// A date, `YYYY-MM-DD`.
    Date {
        /// The date.
        iso: String,
    },
    /// A month of a year, for a report grouped by month. The screen and the print name the
    /// month from Fluent in the language they show.
    Month {
        /// The year, for example 2026.
        #[ts(type = "number")]
        year: i64,
        /// The month, 1 to 12.
        #[ts(type = "number")]
        month: i64,
    },
    /// A share of a total in hundredths of a percent, for display only (analytics, PRD 18.2,
    /// money agent Section 9): 7210 is 72.10%. The screen writes it as `72.10` with the sign
    /// `%`; it never works anything out from it.
    Percent {
        /// Hundredths of a percent, 0 to 10,000.
        #[ts(type = "number")]
        hundredths: i64,
    },
    /// An account.
    Party {
        /// The account.
        party: PartyRef,
    },
    /// A name in both scripts.
    Names {
        /// The names.
        names: Names,
    },
    /// A document: its kind as a Fluent message ID and its number.
    Doc {
        /// Fluent message ID of the kind, for example `report-doc-bikri-first`.
        type_id: String,
        /// The document number (بکری number, voucher number). Null if the source has none.
        #[ts(type = "number | null")]
        number: Option<i64>,
    },
    /// The goods or reason behind a ledger line.
    Detail {
        /// The parts, in order.
        parts: Vec<DetailPart>,
    },
    /// A sentence the screen or the print writes from a Fluent message and its arguments, for
    /// lines whose words depend on what happened (a partner statement line, PRD FR-17.10).
    Message {
        /// The Fluent message ID.
        id: String,
        /// Its arguments.
        args: Vec<MessageArg>,
    },
}

/// One argument of a [`Cell::Message`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportMessageArg")]
pub struct MessageArg {
    /// The variable name in the message, for example `number` or `share`.
    pub name: String,
    /// Its value.
    pub value: ArgValue,
}

/// The value of a message argument. The screen and the print write numbers and money with the
/// number grouping setting, and a name in the language they are in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(rename = "ReportArgValue")]
pub enum ArgValue {
    /// Text as it is (a reason, a date, a kind word).
    Text {
        /// The text.
        text: String,
    },
    /// A whole number (a count, a document number).
    Number {
        /// The number.
        #[ts(type = "number")]
        value: i64,
    },
    /// Money in whole rupees.
    Money {
        /// The amount.
        #[ts(type = "Paisa")]
        paisa: Paisa,
    },
    /// A rate in hundredths of a percent as the percent a person reads (`3750` is `37.5`).
    Percent {
        /// Hundredths of a percent.
        #[ts(type = "number")]
        hundredths: i64,
    },
    /// A name in both scripts.
    Names {
        /// The names.
        names: Names,
    },
    /// Another message, for a word such as a number series (`E-5`) that has its own message.
    Message {
        /// The Fluent message ID.
        id: String,
        /// Its arguments.
        args: Vec<MessageArg>,
    },
}

/// What a row opens (PRD FR-19.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename = "ReportSourceKind")]
pub enum SourceKind {
    /// A saved بکری.
    Bikri,
    /// A saved voucher.
    Voucher,
    /// An account.
    Account,
    /// A month close entry.
    MonthClose,
    /// A trip purchase.
    Purchase,
    /// A trip or a garden.
    Venture,
    /// Fruit taken.
    Fruit,
    /// A journal entry that belongs to no screen of its own.
    Entry,
}

/// The record a row opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportRowSource")]
pub struct RowSource {
    /// What kind of record.
    pub kind: SourceKind,
    /// Its id.
    #[ts(type = "number")]
    pub id: i64,
}

/// How a row is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename = "ReportRowKind")]
pub enum RowKind {
    /// A normal line.
    Normal,
    /// A heading line that groups the lines under it (a head in R17).
    Group,
}

/// One row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportRow")]
pub struct Row {
    /// One cell for each column, in order.
    pub cells: Vec<Cell>,
    /// The record the row opens. Null if it opens nothing.
    pub source: Option<RowSource>,
    /// How it is drawn.
    pub kind: RowKind,
}

/// One line above the table: a label and a value, for example the opening balance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportSummaryLine")]
pub struct SummaryLine {
    /// Fluent message ID of the label.
    pub label_id: String,
    /// The value.
    pub cell: Cell,
}

/// A page of a report, with the totals over the whole filter (PRD FR-19.11, Code Standards B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ReportPage {
    /// The report.
    pub id: ReportId,
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The filters as read, echoed back.
    pub filters: ResolvedFilters,
    /// The columns.
    pub columns: Vec<Column>,
    /// The rows of this page.
    pub rows: Vec<Row>,
    /// Where this page starts.
    #[ts(type = "number")]
    pub offset: i64,
    /// How many rows were asked for. `0` for a page with every row (a print).
    #[ts(type = "number")]
    pub limit: i64,
    /// How many rows the whole filter has.
    #[ts(type = "number")]
    pub total_count: i64,
    /// Fluent message ID of the totals row label.
    pub totals_label_id: String,
    /// The totals row over the whole filter, one cell for each column. A cell of a column with no
    /// total is `empty`. Never a sum of the rows of the page.
    pub totals: Vec<Cell>,
    /// Lines above the table (opening balance, closing balance, the difference of a trial).
    pub summary: Vec<SummaryLine>,
}

/// Input of `reports_run`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ReportRunInput {
    /// The report.
    pub id: ReportId,
    /// The filters.
    #[serde(default)]
    pub filters: ReportFilters,
    /// Where the page starts. 0 for the first page.
    #[serde(default)]
    #[ts(type = "number")]
    pub offset: i64,
    /// How many rows. 0 means the default, 100. At most 1,000.
    #[serde(default)]
    #[ts(type = "number")]
    pub limit: i64,
}

/// Input of `reports_print_data`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ReportPrintInput {
    /// The report.
    pub id: ReportId,
    /// The filters.
    #[serde(default)]
    pub filters: ReportFilters,
    /// The print language (PRD FR-7.36): names, words and digits follow it.
    pub language: Language,
    /// The paper. Missing means A4.
    #[serde(default)]
    pub paper: Option<Paper>,
}
