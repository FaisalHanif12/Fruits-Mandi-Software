//! Print data of a saved بکری (plan 14, PRD FR-12.12, FR-7.36, FR-19.3 to FR-19.7, R1 and R5).
//!
//! The backend builds everything that is printed. A template only lays out what it receives and
//! calculates nothing (CLAUDE.md rule 1, FR-19.3):
//!
//! - Every number is a **stored value** of the saved بکری, formatted for print here: grouping by
//!   the number grouping setting, English digits unless the Urdu digits setting is on and the
//!   print language is Urdu (NFR-URD-14). Sums are sums of stored amounts (Code Standards B20).
//! - Every title and label is a **Fluent message ID**. The template resolves it with the bundle
//!   of the print language (CLAUDE.md rule 7). The only sentences the backend builds are the
//!   amounts in words (FR-19.7).
//! - Names are already in the print language (`name_ur` or `name_en`, FR-7.36). Notes, truck
//!   numbers, phones and codes are as typed (FR-7.25).
//!
//! The documents (PRD 4.4, worked day):
//!
//! | Document | Pages | One page |
//! |---|---|---|
//! | `seller_bill` (R1) | one per seller | his lines, the deductions, the net sale and its words |
//! | `buyer_slips` (R5) | one per buyer account | his lines, commission, لاگا, the total and its words |
//! | `pukhta_slip` | one per one-time buyer | the same as a buyer slip, with his name and phone |
//!
//! Example (بکری 1, seller bill, Urdu): the page of رحیم has the amounts gross 29,400,
//! freight 3,000, commission 1,764, labour 250, munshiana 50, cash paid 5,000, store rent 0,
//! phone 20, total deductions 10,084 and net sale 19,316, and the words
//! "انیس ہزار تین سو سولہ روپے".

mod build;
mod format;
mod ids;
mod load;
mod pages;

use mandi_core::auth::Language;
use mandi_core::printing::{Paper, PrintDoc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use build::{BuildOptions, build};
pub use load::{FirmNames, bikri_print_data};

/// Input of `reports_bikri_print_data` (plan 14).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct BikriPrintDataInput {
    /// The saved بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Which document.
    pub doc: PrintDoc,
    /// The print language (PRD FR-7.36): names, words and digits follow it.
    pub language: Language,
    /// Only the page of this buyer account (`buyer_slips` only). Missing means every buyer.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub only_buyer_id: Option<i64>,
}

/// Writing direction of the print.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TextDir {
    /// Right to left (Urdu).
    Rtl,
    /// Left to right (English).
    Ltr,
}

impl TextDir {
    /// The direction of a print language.
    pub const fn of(language: Language) -> Self {
        match language {
            Language::Ur => TextDir::Rtl,
            Language::En => TextDir::Ltr,
        }
    }
}

/// The firm header in the print language (PRD Section 21 "Firm").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintFirm {
    /// Firm name in the print language.
    pub name: String,
    /// Mandi name in the print language.
    pub mandi: String,
    /// Address as typed. Empty if none.
    pub address: String,
    /// Phone as typed. Empty if none.
    pub phone: String,
}

/// One fact in the header: a label ID and its value, for example the بکری number or the date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintFact {
    /// Fluent message ID of the label.
    pub label_id: String,
    /// The value, formatted for print.
    pub value: String,
}

/// The party a page is for: the seller, a buyer or a one-time buyer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintParty {
    /// Fluent message ID of the role label (seller, buyer, one-time buyer).
    pub role_id: String,
    /// The account code. Null for a one-time person.
    pub code: Option<String>,
    /// The name in the print language.
    pub name: String,
    /// The phone as typed. Null if none.
    pub phone: Option<String>,
}

/// One line of goods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintLine {
    /// The day of the posting of this line, `DD-MM-YYYY`. A later sale has a later day.
    pub date: String,
    /// The buyer's name, on the seller bill. Null on a slip (the page is the buyer).
    pub name: Option<String>,
    /// The item in the print language.
    pub item: String,
    /// The packing in the print language.
    pub packing: String,
    /// Quantity in نگ.
    pub qty: String,
    /// Weight in kg, for an item sold by weight. Null otherwise.
    pub weight_kg: Option<String>,
    /// The rate: the seller rate on the seller bill, the buyer rate on a slip.
    pub rate: String,
    /// The amount: the seller amount on the seller bill, the buyer value on a slip.
    pub amount: String,
    /// The line note as typed. Null if none.
    pub note: Option<String>,
}

/// How a printed amount line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AmountRole {
    /// A normal line.
    Line,
    /// A sum of the lines above it (gross sale, total deductions, value of goods).
    Subtotal,
    /// The amount the page is about (net sale, buyer pays). The amount in words is for this one.
    Total,
}

/// One amount line: a label ID and its value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintAmount {
    /// Fluent message ID of the label.
    pub label_id: String,
    /// The amount in whole rupees, formatted for print.
    pub amount: String,
    /// How it is drawn.
    pub role: AmountRole,
}

/// An amount in words (PRD FR-19.7), built by the backend from `core::urdu_words` or
/// `core::english_words`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintWords {
    /// Fluent message ID of the label ("in words").
    pub label_id: String,
    /// The sentence, in the print language.
    pub text: String,
}

/// One page of a document: one seller, one buyer or one one-time buyer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintPage {
    /// Who the page is for.
    pub party: PrintParty,
    /// The lines of goods, in order.
    pub lines: Vec<PrintLine>,
    /// The amount lines below the goods, in the order to print them.
    pub amounts: Vec<PrintAmount>,
    /// Fluent message ID of how the buyer pays: `print-payment-cash` when every line is cash
    /// (paid), `print-payment-credit` when every line is on credit. Null on the seller bill
    /// and when the lines differ.
    pub payment_id: Option<String>,
    /// The `Total` amount in words. Null when there is no total.
    pub words: Option<PrintWords>,
}

/// Everything a template needs to print one document of one saved بکری (plan 14).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintData {
    /// Which document.
    pub doc: PrintDoc,
    /// The print language.
    pub language: Language,
    /// The writing direction, from the language.
    pub dir: TextDir,
    /// The paper class, from the Printing settings. The template uses the matching `@page`.
    pub paper: Paper,
    /// True when Urdu digits are used on this print (an Urdu print with the setting on), so the
    /// template puts the Naskh font first for numbers (Tech Stack 4.1).
    pub urdu_digits: bool,
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The firm header.
    pub firm: PrintFirm,
    /// The بکری number, formatted for print.
    pub number: String,
    /// The facts of the header, in order: number, date, bill date (when it differs), truck.
    pub facts: Vec<PrintFact>,
    /// The time of this print in Pakistan time, `DD-MM-YYYY HH:MM`.
    pub printed_at: String,
    /// True for a cancelled بکری; the template stamps the pages.
    pub cancelled: bool,
    /// The pages. Empty when there is nothing to print (no one-time buyer for the پختہ slip).
    pub pages: Vec<PrintPage>,
}
