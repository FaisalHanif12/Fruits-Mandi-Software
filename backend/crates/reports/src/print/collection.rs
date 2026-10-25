//! Print data of the collection list (اگراہی فہرست, PRD FR-15.6, FR-7.61, FR-19.13; plan 17e).
//!
//! The list is the buyers planned for one day, with their dues, by collector and then by city. It
//! is the fifth built print, so it has the same header and footer as the other four and the same
//! rules: every number is formatted for print here, every title and label is a Fluent message ID,
//! and names are in the print language.
//!
//! Plan 17e T1 holds the types only. The builder (`collection_print_data`) comes in T2, from the
//! same list the screen reads (`assist_collection_list`), so the print and the screen can never
//! disagree. The blank column for the collector's note is layout: the template draws it, there is
//! no data for it.
//!
//! Example (after Events 1 to 5, a collector with three buyers): one group, three rows, the
//! total of their dues, and the buyers' count.

use mandi_core::auth::Language;
use mandi_core::printing::{Paper, PrintDoc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::header::{PrintFooter, PrintHeader};
use super::{PrintAmount, TextDir};

/// Input of `reports_collection_print_data` (plan 17e): the same day and filters as
/// `assist_collection_list`, and the print language.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CollectionPrintDataInput {
    /// The day, `DD-MM-YYYY` or a shortcut (PRD FR-7.19). Buyers whose collection day is this
    /// weekday are on the list.
    pub date: String,
    /// Only this collector (a staff member). Missing means every collector.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub collector_staff_id: Option<i64>,
    /// Only this city. Missing means every city.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub city_id: Option<i64>,
    /// The print language (PRD FR-7.36): names, digits and dates follow it.
    pub language: Language,
}

/// One buyer on the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionPrintRow {
    /// The row number in the whole list, 1, 2, 3 ..., formatted for print.
    pub serial: String,
    /// The short code of the buyer (PRD FR-7.39): his serial, for example `1` for 03040001.
    pub code: String,
    /// His name in the print language.
    pub name: String,
    /// His tag (نشان), printed after the name. Absent if none (PRD 10.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag: Option<String>,
    /// His phone as typed (PRD FR-7.25). Empty if none.
    pub phone: String,
    /// What he owes (his debit balance), in whole rupees, formatted for print.
    pub dues: String,
    /// The day of his oldest open bill, `DD-MM-YYYY`. Null if he has none.
    pub oldest_bill_date: Option<String>,
    /// Its age in days, formatted for print. Null if he has no open bill.
    pub oldest_days: Option<String>,
}

/// The buyers of one collector in one city.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionPrintGroup {
    /// The collector's name in the print language. Null for buyers with no collector.
    pub collector: Option<String>,
    /// The city's name in the print language. Null for buyers with no city.
    pub city: Option<String>,
    /// The buyers, by code.
    pub rows: Vec<CollectionPrintRow>,
    /// Σ of their dues, formatted for print.
    pub total: String,
}

/// Everything a template needs to print the collection list of one day (plan 17e).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionPrintData {
    /// Always `collection_list`.
    pub doc: PrintDoc,
    /// The print language.
    pub language: Language,
    /// The writing direction, from the language.
    pub dir: TextDir,
    /// The paper class, from the Printing settings (starts as A4).
    pub paper: Paper,
    /// True when Urdu digits are used on this print.
    pub urdu_digits: bool,
    /// The shared header (PRD FR-19.13). Its facts are the day, and the collector and the city
    /// when the list is filtered by them.
    pub header: PrintHeader,
    /// The shared footer (PRD FR-19.13).
    pub footer: PrintFooter,
    /// By collector, then city.
    pub groups: Vec<CollectionPrintGroup>,
    /// Σ of all dues on the list: the totals row (PRD FR-19.5).
    pub total: PrintAmount,
    /// How many buyers are on the list, formatted for print.
    pub buyer_count: String,
}
