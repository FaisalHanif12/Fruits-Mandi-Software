//! The partner statement (R31 data, PRD FR-17.9, FR-17.10, plan 17): one dated line for every
//! amount that reaches a partner, with the running کل رقم (C22). Every line is made from posted
//! records (share rows, journal lines, vouchers, trip parts), so nothing is typed twice.
//!
//! Command `partners_statement`: input `PartnersStatementInput`, answer `PartnerStatement`.
//!
//! The words of a line (the heading, the detail) come from Fluent on the screen, picked by
//! `heading` and `detail`; `values` holds the numbers the message needs. Gardens and fruit come
//! in D4R (`heading` already names them).
//!
//! Worked numbers (PRD D4-A33, the E2E-34 run): الف's lines are −4,160 (`trip`, `trip_paid`: his
//! shop part of purchase خ-1), +480 (`bikri`, `income_share`: his part of the commission 1,200),
//! +3,384 (`trip`, `trip_credit`: his 40% of the seller credit 8,460), ending at −296.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::error::ErrorValue;
use crate::money::Paisa;
use crate::paging::{MonthSubtotal, SortInput};

/// Input of `partners_statement`. Dates are `DD-MM-YYYY` or a shortcut; null or empty means no
/// limit. The running total at the start of the range is the balance before it.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartnersStatementInput {
    /// The partner. `ERR_NOT_FOUND` on `partner_id`.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// First day.
    #[serde(default)]
    pub from: Option<String>,
    /// Last day.
    #[serde(default)]
    pub to: Option<String>,
    /// How many lines to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many lines to return. 0 gives the default page of 100 lines (PRD FR-19.11).
    #[serde(default)]
    pub limit: u32,
    /// The column to sort the whole range by: `entry_date`, `amount_paisa` or `running_paisa`. Null is the natural order (plan 38
    /// R4). A sorted row still shows its true running figure of the natural order.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// True to give the subtotal of each month in `months`. Used only in date order.
    #[serde(default)]
    pub group_by_month: bool,
}

/// Where a line came from (PRD FR-17.10 "headings").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum StatementHeading {
    /// بکری: his share of the commission, with the بکری number.
    Bikri,
    /// خرچہ: his share of a shop expense.
    Expense,
    /// سرمایہ: opening capital, invested, taken back, month close.
    Capital,
    /// خرچہ: money taken or returned (ذاتی خرچہ), with its reason.
    Drawings,
    /// سفر: his part paid from the shop's cash, the money credited to him, notes.
    Trip,
    /// باغ (D4R): installments, running costs, the closing.
    Garden,
    /// پھل (D4R): fruit taken, with item, qty and rate.
    Fruit,
    /// کھاتہ اور ادھار (plan 76 points 5 and 10, PRD FR-16.6, FR-16.13): money that reached him
    /// through a separate khata or a loan linked to him or to the shop. `subject` is the khata or
    /// the person.
    Linked,
}

/// Which kind of line it is inside its heading. It picks the Fluent message. The note lines
/// (`month_close`, `trip_write_off`, `trip_closed`) have no amount and leave the running total
/// unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum StatementDetail {
    /// `bikri`: his share of the income of a posting. `number` is the بکری number.
    IncomeShare,
    /// `expense`: his share of a shop expense (negative). `number` is the voucher number; `subject`
    /// is the expense head.
    ExpenseShare,
    /// `capital`: his opening capital (PRD FR-10.9, C22).
    CapitalOpening,
    /// `capital`: invested. `number` is the voucher number.
    CapitalIn,
    /// `capital`: taken back (negative).
    CapitalOut,
    /// `capital`, note: month close moved profit into capital. No amount.
    MonthClose,
    /// `drawings`: money he took (negative); `reason` says why. `number` is the voucher number.
    Taken,
    /// `drawings`: money he returned.
    Returned,
    /// `drawings`: an opening debit on his drawings account, dated its opening date (FR-10.9).
    DrawingsOpening,
    /// `trip`: his part of a purchase or trip cost paid from the shop's cash (negative). `number`
    /// is the trip number, `secondary_number` the purchase or cost number.
    TripPaid,
    /// `trip`: his part of a seller credit of a trip lot, or of money from a sale in another shop.
    /// `number` is the trip number, `secondary_number` the بکری number (0 or null for another
    /// shop); `subject` is the item; `values`: `qty`, `sales_paisa`, `cost_paisa`, `result_paisa`,
    /// `share_bps` and `part_paisa` (his part of the result: `split_by_bps` of it by the trip's
    /// shares, plan 76 point 11, PRD FR-17.12; absent when the trip's shares do not total 100%).
    TripCredit,
    /// `trip`, note: trip goods were written off. No amount.
    TripWriteOff,
    /// `trip`, note: the trip closed. No amount; `values`: `cost_paisa`, `sales_paisa`,
    /// `result_paisa`, `share_bps`, `net_paisa` (his money back less his money in, plan 16b
    /// finding 3).
    TripClosed,
    /// `garden` (D4R): his part paid from the shop's cash.
    GardenPaid,
    /// `garden` (D4R): the closing line.
    GardenClosed,
    /// `fruit` (D4R): fruit taken.
    FruitTaken,
    /// `linked`: his part of money paid out to a khata (negative). `number` is the voucher number.
    KhataPaid,
    /// `linked`: his part of money that came back from a khata.
    KhataReceived,
    /// `linked`: money given to a person linked to him or to the shop (negative).
    LentGiven,
    /// `linked`: money a person returned.
    LentReturned,
    /// `linked`: any other entry on a linked khata (the credit of its goods sold, fruit, a جنرل
    /// line), with its sign. `number` is the بکری or voucher number when it has one.
    LinkedOther,
    /// `linked`, note: a loan linked to the shop was given or returned. No amount, because the money
    /// is still owed to the shop and his total does not change. `values`: `kind` (`given` or
    /// `returned`), `amount_paisa` (the loan), `part_paisa` (his % of it, whole rupees; absent when
    /// the shares of that day did not total 100%).
    LentShareNote,
}

/// One line of the statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementLine {
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// Where it came from.
    pub heading: StatementHeading,
    /// Which kind of line, for the message.
    pub detail: StatementDetail,
    /// The journal entry behind it, or null for a note line.
    #[ts(type = "number | null")]
    pub entry_id: Option<i64>,
    /// The main number: a بکری, voucher or trip number.
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The code of the garden or trip the line belongs to (plan 34): `0080`, `0070`. Null on a
    /// line of any other heading. For a garden or trip line `number` is the number in its series
    /// and this is the text to show.
    pub venture_code: Option<String>,
    /// A second number: a purchase, a trip cost or a بکری inside a trip line.
    #[ts(type = "number | null")]
    pub secondary_number: Option<i64>,
    /// What the line is about: the expense head, or the item of a trip sale or of fruit taken. Null
    /// if none.
    pub subject: Option<RecordRef>,
    /// The packing of the item, for a fruit line (PRD FR-17.10): "2 پیٹی مالٹا @ 1,400". Null for
    /// any other line.
    pub packing: Option<RecordRef>,
    /// The user's reason. Empty if none.
    pub reason: String,
    /// The signed amount, or null for a note line.
    #[ts(type = "Paisa | null")]
    pub amount_paisa: Option<Paisa>,
    /// His کل رقم after this line.
    #[ts(type = "Paisa")]
    pub running_paisa: Paisa,
    /// Numbers for the message. A name ending in `_paisa` is money, `_bps` a share.
    #[ts(type = "Record<string, string | number>")]
    pub values: BTreeMap<String, ErrorValue>,
}

/// What a statement adds up to, over the whole range (B31). Notes are not counted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementTotals {
    /// Σ of the lines that add (plus).
    #[ts(type = "Paisa")]
    pub plus_paisa: Paisa,
    /// Σ of the lines that take away, as a positive number.
    #[ts(type = "Paisa")]
    pub minus_paisa: Paisa,
}

/// A partner's statement, one page (PRD FR-17.9, FR-17.10, FR-19.11). Read only. The last running
/// total of the whole range equals his کل رقم at the end of the range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartnerStatement {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His کل رقم at the start of the range.
    #[ts(type = "Paisa")]
    pub opening_total_paisa: Paisa,
    /// His کل رقم at the end of the range.
    #[ts(type = "Paisa")]
    pub closing_total_paisa: Paisa,
    /// The page, in date order.
    pub lines: Vec<StatementLine>,
    /// How many lines there are in the range.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over the whole range.
    pub totals: StatementTotals,
    /// The count and subtotals of each month of the whole range, in the order of the list, when
    /// asked with `group_by_month` and the list is in date order. Figures: `plus_paisa` and `minus_paisa`.
    pub months: Vec<MonthSubtotal>,
}

// ---- One month at a time (plan 76 point 11, PRD FR-17.9 v1.41) ----

/// Input of `partners_statement_month`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartnersStatementMonthInput {
    /// The partner. `ERR_NOT_FOUND` on `partner_id`.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// The month, `YYYY-MM`. Null or empty is the current month. A month after the current one is
    /// refused (`ERR_DATE_IN_FUTURE` on `month`).
    #[serde(default)]
    pub month: Option<String>,
}

/// The money of a day or a month by what it was, signed: plus raises his کل رقم, minus lowers it.
/// The columns add up to the net.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementColumns {
    /// His share of the income of the day's postings (commission, لاگا, فون, ریٹ فرق).
    #[ts(type = "Paisa")]
    pub commission_paisa: Paisa,
    /// His share of the shop expenses (negative).
    #[ts(type = "Paisa")]
    pub expense_paisa: Paisa,
    /// His gardens and trips: his parts paid from the shop's cash and the money credited to him.
    #[ts(type = "Paisa")]
    pub gardens_trips_paisa: Paisa,
    /// Fruit he took (negative).
    #[ts(type = "Paisa")]
    pub fruit_paisa: Paisa,
    /// Money taken and returned: his drawings (and, from plan 76 points 5 and 10, the linked الگ
    /// کھاتہ and ادھار رقم).
    #[ts(type = "Paisa")]
    pub taken_returned_paisa: Paisa,
    /// Capital in and out, with his opening capital.
    #[ts(type = "Paisa")]
    pub capital_paisa: Paisa,
    /// The six columns added up.
    #[ts(type = "Paisa")]
    pub net_paisa: Paisa,
}

/// One day of the month that has a movement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StatementMonthDay {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// What came in and went out that day.
    pub columns: StatementColumns,
    /// His کل رقم at the end of the day.
    #[ts(type = "Paisa")]
    pub running_paisa: Paisa,
    /// True when the day also has a note line (a trip closing, a write-off, a month close).
    pub has_notes: bool,
}

/// What `partners_statement_month` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartnerStatementMonth {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// The month shown, `YYYY-MM`.
    pub month: String,
    /// The first month with a movement of his, or the current month when he has none. The back
    /// button stops here.
    pub first_month: String,
    /// The current month, where the next button stops.
    pub current_month: String,
    /// His کل رقم at the start of the month.
    #[ts(type = "Paisa")]
    pub opening_total_paisa: Paisa,
    /// The days of the month that have a movement, oldest first.
    pub days: Vec<StatementMonthDay>,
    /// The month's columns added up.
    pub totals: StatementColumns,
    /// His کل رقم at the end of the month: the partner overview and R31 for the current month.
    #[ts(type = "Paisa")]
    pub closing_total_paisa: Paisa,
}

/// Input of `partners_statement_day`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartnersStatementDayInput {
    /// The partner.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// The day, `YYYY-MM-DD` (as the month rows give it) or `DD-MM-YYYY`.
    pub date: String,
}

/// What `partners_statement_day` gives back: every movement of one day with its source, in the
/// order of the statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartnerStatementDay {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// His کل رقم at the start of the day.
    #[ts(type = "Paisa")]
    pub opening_total_paisa: Paisa,
    /// The lines, with their entry ids and headings so each opens its entry.
    pub lines: Vec<StatementLine>,
    /// His کل رقم after the day.
    #[ts(type = "Paisa")]
    pub after_total_paisa: Paisa,
}

/// Input of `partners_statement_month_print_data`: the month as the page shows it and the print
/// language.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartnersStatementMonthPrintInput {
    /// The partner.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// The month, `YYYY-MM`; null is the current month.
    #[serde(default)]
    pub month: Option<String>,
    /// The print language (PRD FR-7.36).
    pub language: crate::auth::Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    pub paper: Option<crate::printing::Paper>,
}
