//! Builds the print data of a saved بکری from its stored values (plan 14, PRD FR-12.12, R1, R5,
//! FR-13.1 to FR-13.3).
//!
//! Nothing is worked out again (Code Standards B20). A line shows its stored rate and amount.
//! A sum is a sum of stored amounts, the same sums `bikri_get` sends (`SavedTotals`), so a
//! printed total equals the screen and the journal to the rupee.

use chrono::{DateTime, Utc};
use mandi_core::auth::Language;
use mandi_core::bikri::{BikriDetail, BikriStatus};
use mandi_core::english_words::english_words;
use mandi_core::money::{Grouping, Paisa};
use mandi_core::printing::{PrintDoc, PrintSetup};
use mandi_core::urdu_words::urdu_words;

use super::format::NumberStyle;
use super::header::{footer, header, pick};
use super::ids::*;
use super::load::FirmNames;
use super::pages::{buyer_pages, pukhta_pages, seller_pages};
use super::{AmountRole, PrintAmount, PrintData, PrintFact, PrintWords, TextDir};
use crate::error::ReportsError;

/// Everything besides the saved بکری that decides what is printed.
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Which document.
    pub doc: PrintDoc,
    /// The print language.
    pub language: Language,
    /// The number grouping setting (PRD FR-19.7).
    pub grouping: Grouping,
    /// The Printing settings: paper per document and the Urdu digits switch.
    pub setup: PrintSetup,
    /// The firm header.
    pub firm: FirmNames,
    /// The time of the print.
    pub now_utc: DateTime<Utc>,
    /// Only the page of this buyer account (buyer slips only).
    pub only_buyer_id: Option<i64>,
}

/// Builds the print data of one document from a saved بکری. Pure: it reads nothing else.
///
/// Example (بکری 1, PRD 4.4): the seller bill has one page, for رحیم, with gross 29,400 and net
/// 19,316; the buyer slips have two pages (اکرم 12,880 and ارشد 18,684).
///
/// # Errors
/// `ReportsError::Money` if a sum does not fit, `ReportsError::Words` if an amount cannot be
/// written in words, `ReportsError::Data` for a stored date that is not a date.
pub fn build(detail: &BikriDetail, options: &BuildOptions) -> Result<PrintData, ReportsError> {
    let ctx = Ctx {
        language: options.language,
        grouping: options.grouping,
        style: NumberStyle::new(
            options.language,
            options.grouping,
            options.setup.urdu_digits,
        ),
    };
    let (title_id, pages) = match options.doc {
        PrintDoc::SellerBill => (TITLE_SELLER_BILL, seller_pages(&ctx, detail)?),
        PrintDoc::BuyerSlips => (
            TITLE_BUYER_SLIP,
            buyer_pages(&ctx, detail, options.only_buyer_id)?,
        ),
        PrintDoc::PukhtaSlip => (TITLE_PUKHTA_SLIP, pukhta_pages(&ctx, detail)?),
        // The voucher slip is built from a voucher (plan 15) and the collection list from a
        // day (plan 17e), not from a بکری.
        PrintDoc::ReceiptSlip | PrintDoc::CollectionList => {
            return Err(ReportsError::Data(format!(
                "{} is not a document of a بکری",
                options.doc.as_str()
            )));
        }
    };
    let language = options.language;
    Ok(PrintData {
        doc: options.doc,
        language,
        dir: TextDir::of(language),
        paper: options.setup.paper_of(options.doc),
        urdu_digits: ctx.style.urdu_digits(),
        header: header(language, &options.firm, title_id, facts(&ctx, detail)?),
        footer: footer(ctx.style.printed_at(options.now_utc)),
        number: ctx.style.count(detail.number),
        cancelled: detail.status == BikriStatus::Cancelled,
        pages,
    })
}

/// What every page builder needs.
pub(super) struct Ctx {
    pub(super) language: Language,
    pub(super) grouping: Grouping,
    pub(super) style: NumberStyle,
}

impl Ctx {
    pub(super) fn money(&self, amount: Paisa) -> Result<String, ReportsError> {
        self.style.money(amount)
    }

    /// One amount line.
    pub(super) fn amount(
        &self,
        label_id: &str,
        amount: Paisa,
        role: AmountRole,
    ) -> Result<PrintAmount, ReportsError> {
        Ok(PrintAmount {
            label_id: label_id.to_owned(),
            amount: self.money(amount)?,
            role,
        })
    }

    /// The amount in words, in the print language (PRD FR-19.7).
    pub(super) fn words(&self, amount: Paisa) -> Result<PrintWords, ReportsError> {
        let text = match self.language {
            Language::Ur => urdu_words(amount)?,
            Language::En => english_words(amount, self.grouping)?,
        };
        Ok(PrintWords {
            label_id: IN_WORDS.to_owned(),
            text,
        })
    }

    pub(super) fn name(&self, name_ur: &str, name_en: &str) -> String {
        pick(self.language, name_ur, name_en)
    }
}

/// The header facts: the number, the day, the bill day when it differs, the truck.
fn facts(ctx: &Ctx, detail: &BikriDetail) -> Result<Vec<PrintFact>, ReportsError> {
    let mut facts = vec![
        PrintFact {
            label_id: FACT_NUMBER.to_owned(),
            value: ctx.style.count(detail.number),
        },
        PrintFact {
            label_id: FACT_DATE.to_owned(),
            value: ctx.style.date(&detail.entry_date)?,
        },
    ];
    if detail.bill_date != detail.entry_date {
        facts.push(PrintFact {
            label_id: FACT_BILL_DATE.to_owned(),
            value: ctx.style.date(&detail.bill_date)?,
        });
    }
    // The truck number is as typed (PRD FR-7.25).
    if let Some(truck) = detail.truck_no.as_deref().filter(|t| !t.trim().is_empty()) {
        facts.push(PrintFact {
            label_id: FACT_TRUCK.to_owned(),
            value: truck.to_owned(),
        });
    }
    Ok(facts)
}
