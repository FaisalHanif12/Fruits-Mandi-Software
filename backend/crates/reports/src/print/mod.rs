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
//!   numbers and codes are as typed (FR-7.25); a phone is printed `03XX-XXXXXXX` (plain digits).
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
mod collection;
pub(crate) mod format;
mod frame;
mod header;
mod ids;
pub(crate) mod load;
mod pages;
mod voucher;

use mandi_core::auth::Language;
use mandi_core::printing::{Paper, PrintDoc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use build::{BuildOptions, build};
pub use collection::{
    CollectionBuildOptions, CollectionPrintData, CollectionPrintDataInput, CollectionPrintGroup,
    CollectionPrintRow, build_collection, collection_print_data,
};
pub use frame::{PrintFrame, PrintFrameInput, print_frame};
pub use header::{PrintContact, PrintFooter, PrintHeader};
pub use load::{FirmNames, bikri_print_data};
pub use voucher::voucher_print_data;

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

/// Input of `reports_voucher_print_data` (plan 15, PRD FR-15.2).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct VoucherPrintDataInput {
    /// The saved voucher: a وصولی, an ادائیگی or a پیشگی.
    #[ts(type = "number")]
    pub voucher_id: i64,
    /// The print language (PRD FR-7.36): names, words and digits follow it.
    pub language: Language,
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
    /// Phone, printed `03XX-XXXXXXX` with plain digits. Empty if none.
    pub phone: String,
}

/// One fact in the header: a label ID and its value, for example the بکری number or the date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
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
    /// The short code of a seller or buyer (PRD FR-7.39, FR-19.13), for example `1` for
    /// 03040001. Absent for a one-time person and for an account that has no short code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub short_code: Option<String>,
    /// The name in the print language.
    pub name: String,
    /// The tag (نشان) of the account, printed after the name. Absent if none, and always absent
    /// for a one-time person (PRD 10.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag: Option<String>,
    /// The phone, printed `03XX-XXXXXXX` with plain digits. Null if none.
    pub phone: Option<String>,
}

/// One line of goods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintLine {
    /// The day of the posting of this line, `DD-MM-YYYY`. A later sale has a later day.
    pub date: String,
    /// The buyer's name, on the seller bill. Null on a slip (the page is the buyer).
    pub name: Option<String>,
    /// The tag (نشان) of that buyer, printed after his name. Absent if none, on a slip, and for
    /// a one-time buyer (PRD 10.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name_tag: Option<String>,
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
    /// The line written the client's way for R5 and the buyer slips: `(item qty . rate)`, for
    /// example `(کیلا 20 . 600)` (PRD R5, plan 17e). Built here from the item name in the print
    /// language and the formatted quantity and rate, so the template only lays it out.
    pub detail: String,
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
    /// The row number of this page's party in the document: 1, 2, 3 ... (the serial column of
    /// R5, PRD 19.2), formatted for print.
    pub serial: String,
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
    /// The header: the firm, its contacts and trade mark, the title and the facts (number, date,
    /// bill date when it differs, truck). Plan 17e moved these here from the top level
    /// (`firm`, `title_id`, `facts`).
    pub header: PrintHeader,
    /// The footer: the print time and the labels of "Page x of y". Plan 17e moved `printed_at`
    /// here.
    pub footer: PrintFooter,
    /// The بکری or voucher number, formatted for print.
    pub number: String,
    /// True for a cancelled بکری; the template stamps the pages.
    pub cancelled: bool,
    /// The pages. Empty when there is nothing to print (no one-time buyer for the پختہ slip).
    pub pages: Vec<PrintPage>,
}
