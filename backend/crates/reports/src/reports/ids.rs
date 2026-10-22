//! The Fluent message IDs the report engine sends to the screen and to the print template
//! (CLAUDE.md rule 7). The frontend defines them in `reports.ftl`, in Urdu and in English. Keep
//! this list in step with it. No sentence is built here.

pub(crate) const GROUP_SALES: &str = "report-group-sales";
pub(crate) const GROUP_ACCOUNTS: &str = "report-group-accounts";
pub(crate) const GROUP_COLLECTIONS: &str = "report-group-collections";
pub(crate) const GROUP_BUSINESS: &str = "report-group-business";

// Column headings.
pub(crate) const COL_DATE: &str = "report-col-date";
pub(crate) const COL_DOC: &str = "report-col-doc";
pub(crate) const COL_ACCOUNT: &str = "report-col-account";
pub(crate) const COL_NARRATION: &str = "report-col-narration";
pub(crate) const COL_DEBIT: &str = "report-col-debit";
pub(crate) const COL_CREDIT: &str = "report-col-credit";
pub(crate) const COL_BALANCE: &str = "report-col-balance";
pub(crate) const COL_DETAIL: &str = "report-col-detail";
pub(crate) const COL_CITY: &str = "report-col-city";
pub(crate) const COL_HEAD: &str = "report-col-head";
pub(crate) const COL_OPENING_DATE: &str = "report-col-opening-date";

// Labels of summary and total lines.
pub(crate) const TOTAL: &str = "report-total";
pub(crate) const OPENING_BALANCE: &str = "report-opening-balance";
pub(crate) const CLOSING_BALANCE: &str = "report-closing-balance";
pub(crate) const DIFFERENCE: &str = "report-difference";
pub(crate) const STATEMENT_BOUGHT: &str = "report-statement-bought";
pub(crate) const STATEMENT_SOLD: &str = "report-statement-sold";
pub(crate) const STATEMENT_PAID: &str = "report-statement-paid";
pub(crate) const STATEMENT_DEBIT: &str = "report-statement-debit";
pub(crate) const STATEMENT_CREDIT: &str = "report-statement-credit";

// The side of a balance.
pub(crate) const SIDE_DR: &str = "report-side-dr";
pub(crate) const SIDE_CR: &str = "report-side-cr";
pub(crate) const SIDE_ZERO: &str = "report-side-zero";

// The filters echoed in a print header.
pub(crate) const FILTER_FROM: &str = "report-filter-from";
pub(crate) const FILTER_TO: &str = "report-filter-to";
pub(crate) const FILTER_AS_OF: &str = "report-filter-as-of";
pub(crate) const FILTER_PARTY: &str = "report-filter-party";
pub(crate) const FILTER_SELLER: &str = "report-filter-seller";
pub(crate) const FILTER_ZAMINDAR: &str = "report-filter-zamindar";
pub(crate) const FILTER_ITEM: &str = "report-filter-item";
pub(crate) const FILTER_CITY: &str = "report-filter-city";
pub(crate) const FILTER_ADDA: &str = "report-filter-adda";
pub(crate) const FILTER_DETAIL: &str = "report-filter-detail";
pub(crate) const FILTER_BALANCE_ONLY: &str = "report-filter-balance-only";

/// The page footer message of the shared print layout ("Page x of y").
pub(crate) const PAGE_FOOTER: &str = "print-page-label";

/// The ID of the title of a report: `report-r11-title`.
pub(crate) fn title_id(code: &str) -> String {
    format!("report-{}-title", code.to_ascii_lowercase())
}

/// The posting types the journal can hold (Architecture 15.7, after migration 0006).
const POSTING_TYPES: &[&str] = &[
    "bikri_first",
    "bikri_later",
    "reversal",
    "receipt",
    "payment",
    "advance",
    "shop_expense",
    "personal_expense",
    "personal_return",
    "money_lent",
    "money_lent_return",
    "capital_in",
    "capital_out",
    "khata_out",
    "khata_in",
    "purchase",
    "trip_cost",
    "garden_payment",
    "venture_receipt",
    "garden_close",
    "fruit_taken",
    "general",
    "opening",
    "month_close",
];

/// The ID of the document label of a posting type: `report-doc-bikri-first`. A type this list
/// does not know gets `report-doc-other`, so a new type never shows an empty cell.
pub(crate) fn doc_id(posting_type: &str) -> String {
    if POSTING_TYPES.contains(&posting_type) {
        format!("report-doc-{}", posting_type.replace('_', "-"))
    } else {
        "report-doc-other".to_owned()
    }
}

/// Every posting type with a document label, for the Fluent check.
pub fn posting_types() -> &'static [&'static str] {
    POSTING_TYPES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_id_is_the_report_code_in_lower_case() {
        assert_eq!(title_id("R11"), "report-r11-title");
    }

    #[test]
    fn every_posting_type_has_a_document_label_and_an_unknown_one_a_fallback() {
        assert_eq!(doc_id("bikri_first"), "report-doc-bikri-first");
        assert_eq!(doc_id("money_lent_return"), "report-doc-money-lent-return");
        assert_eq!(doc_id("something_new"), "report-doc-other");
        for kind in posting_types() {
            assert!(doc_id(kind).starts_with("report-doc-"));
            assert_ne!(doc_id(kind), "report-doc-other");
        }
    }
}
