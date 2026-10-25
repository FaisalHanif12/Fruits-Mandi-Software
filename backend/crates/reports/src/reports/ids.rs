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

// Column headings of the Sales, Collections and Business groups (plan 20, step B3).
pub(crate) const COL_BUYER: &str = "report-col-buyer";
pub(crate) const COL_SELLER: &str = "report-col-seller";
pub(crate) const COL_ITEM: &str = "report-col-item";
pub(crate) const COL_QTY: &str = "report-col-qty";
pub(crate) const COL_RATE: &str = "report-col-rate";
pub(crate) const COL_VALUE: &str = "report-col-value";
pub(crate) const COL_BUYER_TOTAL: &str = "report-col-buyer-total";
pub(crate) const COL_GROSS: &str = "report-col-gross";
pub(crate) const COL_FREIGHT: &str = "report-col-freight";
pub(crate) const COL_COMMISSION: &str = "report-col-commission";
pub(crate) const COL_LABOUR: &str = "report-col-labour";
pub(crate) const COL_MUNSHIANA: &str = "report-col-munshiana";
pub(crate) const COL_CASH_PAID: &str = "report-col-cash-paid";
pub(crate) const COL_STORE_RENT: &str = "report-col-store-rent";
pub(crate) const COL_PHONE_CHARGE: &str = "report-col-phone-charge";
pub(crate) const COL_DEDUCTIONS: &str = "report-col-deductions";
pub(crate) const COL_NET: &str = "report-col-net";
pub(crate) const COL_AVG_RATE: &str = "report-col-avg-rate";
pub(crate) const COL_SELLER_COMM: &str = "report-col-seller-comm";
pub(crate) const COL_BUYER_COMM: &str = "report-col-buyer-comm";
pub(crate) const COL_LAGA: &str = "report-col-laga";
pub(crate) const COL_TOTAL_COMM: &str = "report-col-total-comm";
pub(crate) const COL_ADDA: &str = "report-col-adda";
pub(crate) const COL_TRUCKS: &str = "report-col-trucks";
pub(crate) const COL_LOTS: &str = "report-col-lots";
pub(crate) const COL_SIDE: &str = "report-col-side";
pub(crate) const COL_PERSON: &str = "report-col-person";
pub(crate) const COL_PHONE: &str = "report-col-phone";
pub(crate) const COL_OLDEST_DATE: &str = "report-col-oldest-date";
pub(crate) const COL_DAYS: &str = "report-col-days";
pub(crate) const COL_AGE_0_7: &str = "report-col-age-0-7";
pub(crate) const COL_AGE_8_30: &str = "report-col-age-8-30";
pub(crate) const COL_AGE_31_90: &str = "report-col-age-31-90";
pub(crate) const COL_AGE_91: &str = "report-col-age-91";
pub(crate) const COL_COLLECTOR: &str = "report-col-collector";
pub(crate) const COL_SECTION: &str = "report-col-section";
pub(crate) const COL_PAYABLE: &str = "report-col-payable";
pub(crate) const COL_ADVANCE_OPEN: &str = "report-col-advance-open";
pub(crate) const COL_OWES_FIRM: &str = "report-col-owes-firm";
pub(crate) const COL_LAST_DATE: &str = "report-col-last-date";
pub(crate) const COL_GIVEN: &str = "report-col-given";
pub(crate) const COL_RETURNED: &str = "report-col-returned";
pub(crate) const COL_ADJUSTED: &str = "report-col-adjusted";
pub(crate) const COL_OPEN: &str = "report-col-open";
pub(crate) const COL_BILL_DATE: &str = "report-col-bill-date";
pub(crate) const COL_PERIOD: &str = "report-col-period";
pub(crate) const COL_AMOUNT: &str = "report-col-amount";
pub(crate) const COL_INCOME_SHARE: &str = "report-col-income-share";
pub(crate) const COL_EXPENSE_SHARE: &str = "report-col-expense-share";
pub(crate) const COL_PROFIT_SHARE: &str = "report-col-profit-share";
pub(crate) const COL_TAKEN: &str = "report-col-taken";
pub(crate) const COL_PARTNER_NET: &str = "report-col-partner-net";
pub(crate) const COL_PARTNER: &str = "report-col-partner";

// Column headings and sections of R12, R29 and R32 (plan d5-late, round 7).
pub(crate) const COL_CASH_IN: &str = "report-col-cash-in";
pub(crate) const COL_CASH_OUT: &str = "report-col-cash-out";
pub(crate) const COL_NAME: &str = "report-col-name";
pub(crate) const COL_EXPENSE: &str = "report-col-expense";
pub(crate) const COL_STAFF: &str = "report-col-staff";
pub(crate) const COL_LENT: &str = "report-col-lent";
pub(crate) const COL_RETURNED_CASH: &str = "report-col-returned-cash";
pub(crate) const COL_RETURNED_SALARY: &str = "report-col-returned-salary";
pub(crate) const COL_RETURN_BY: &str = "report-col-return-by";
pub(crate) const COL_DAYS_PAST: &str = "report-col-days-past";
pub(crate) const STAFF_YES: &str = "report-staff-yes";
pub(crate) const R32_OVERDUE: &str = "report-r32-overdue";
// Column headings of R10, R33 and R34 (plan d5-late, round 8).
pub(crate) const COL_PACKING: &str = "report-col-packing";
pub(crate) const COL_RECEIVED: &str = "report-col-received";
pub(crate) const COL_SOLD: &str = "report-col-sold";
pub(crate) const COL_WRITTEN_OFF: &str = "report-col-written-off";
pub(crate) const COL_BALANCE_QTY: &str = "report-col-balance-qty";
// Fruit taken (plan d5-late, after D4R 17d): R10, R34 and R37.
pub(crate) const COL_FRUIT_TAKEN: &str = "report-col-fruit-taken";
pub(crate) const R34_FRUIT_TAKEN: &str = "report-r34-fruit-taken";
/// Words the fruit screen already has, used as they are by the register and its header.
pub(crate) const FRUIT_FOR_GUESTS: &str = "fruit-for-guests";
pub(crate) const FRUIT_SOURCE_OWN: &str = "fruit-source-own";
pub(crate) const FRUIT_SOURCE_OTHER: &str = "fruit-source-other";
pub(crate) const COL_FRUIT_FOR: &str = "fruit-filter-for";
pub(crate) const COL_FRUIT_SOURCE: &str = "fruit-col-source";
pub(crate) const COL_FRUIT_WEIGHT: &str = "fruit-col-weight";
pub(crate) const COL_DAYS_WAITING: &str = "report-col-days-waiting";
pub(crate) const COL_TRIP: &str = "report-col-trip";
pub(crate) const COL_PURCHASE: &str = "report-col-purchase";
pub(crate) const COL_BOUGHT_QTY: &str = "report-col-bought-qty";
pub(crate) const COL_BOUGHT_COST: &str = "report-col-bought-cost";
pub(crate) const COL_SALES_MONEY: &str = "report-col-sales-money";
pub(crate) const COL_COST_SOLD: &str = "report-col-cost-sold";
pub(crate) const COL_WRITTEN_OFF_COST: &str = "report-col-written-off-cost";
pub(crate) const COL_LEFT_QTY: &str = "report-col-left-qty";
pub(crate) const COL_LEFT_COST: &str = "report-col-left-cost";
pub(crate) const COL_RESULT: &str = "report-col-result";
pub(crate) const COL_SHARE: &str = "report-col-share";
pub(crate) const COL_MONEY_IN_SHOP: &str = "report-col-money-in-shop";
pub(crate) const COL_MONEY_IN_POCKET: &str = "report-col-money-in-pocket";
pub(crate) const COL_MONEY_BACK: &str = "report-col-money-back";
pub(crate) const COL_PART_OF_RESULT: &str = "report-col-part-of-result";
// Column headings and labels of R36, the gardens report (plan d5-late, round 11).
pub(crate) const COL_SERIAL: &str = "report-col-serial";
pub(crate) const FILTER_BIKRI: &str = "report-filter-bikri";
pub(crate) const COL_GARDEN: &str = "report-col-garden";
pub(crate) const COL_RENT: &str = "report-col-rent";
pub(crate) const COL_INSTALLMENTS: &str = "report-col-installments";
pub(crate) const COL_GARDEN_COSTS: &str = "report-col-garden-costs";
pub(crate) const COL_SALES_OUR_SHOP: &str = "report-col-sales-our-shop";
pub(crate) const COL_SALES_OTHER_SHOPS: &str = "report-col-sales-other-shops";
pub(crate) const COL_OTHER_IN: &str = "report-col-other-in";
pub(crate) const COL_MONEY_IN: &str = "report-col-money-in";
pub(crate) const COL_RUNNING_BALANCE: &str = "report-col-running-balance";
pub(crate) const COL_RENT_DUE: &str = "report-col-rent-due";
pub(crate) const GARDEN_OPEN: &str = "report-garden-open";
pub(crate) const GARDEN_CLOSED: &str = "report-garden-closed";
pub(crate) const FILTER_GARDEN: &str = "report-filter-garden";
pub(crate) const FILTER_GARDEN_BY_GARDEN: &str = "report-filter-garden-by-garden";
pub(crate) const FILTER_GARDEN_BY_PARTNER: &str = "report-filter-garden-by-partner";
pub(crate) const COL_LINE: &str = "report-col-line";
pub(crate) const COL_COUNT: &str = "report-col-count";
// R35, the party day sheet (PRD FR-19.12): column headings, the two tables and the footer lines.
pub(crate) const COL_CODE: &str = "report-col-code";
pub(crate) const COL_PREVIOUS: &str = "report-col-previous";
pub(crate) const R35_BUYERS: &str = "report-r35-buyers";
pub(crate) const R35_SELLERS: &str = "report-r35-sellers";
pub(crate) const R35_TOTAL_QTY: &str = "report-r35-total-qty";
pub(crate) const R35_PREVIOUS: &str = "report-r35-previous";
pub(crate) const R35_BOUGHT: &str = "report-r35-bought";
pub(crate) const R35_SOLD: &str = "report-r35-sold";
pub(crate) const R35_OTHER_DEBITS: &str = "report-r35-other-debits";
pub(crate) const R35_OTHER_CREDITS: &str = "report-r35-other-credits";
pub(crate) const R35_ON_SPOT: &str = "report-r35-on-spot";
pub(crate) const R35_CASH_GIVEN: &str = "report-r35-cash-given";
pub(crate) const R35_PAID: &str = "report-r35-paid";
pub(crate) const R35_RECEIVED: &str = "report-r35-received";
pub(crate) const R35_REMAINING: &str = "report-r35-remaining";
/// The last line of a seller's slip: what is left, "balance" (the word ٹوٹل رقم suits a buyer's bill
/// only: for a seller it would read as money he owes).
pub(crate) const R35_BALANCE: &str = "report-r35-balance";
/// The party column of the all-parties table (PRD FR-19.12 says پارٹی).
pub(crate) const COL_PARTY: &str = "report-col-party";
pub(crate) const R35_COLLECTION: &str = "report-r35-collection";
pub(crate) const R35_SELLERS_CREDIT: &str = "report-r35-sellers-credit";
pub(crate) const R35_SELLERS_DEBIT: &str = "report-r35-sellers-debit";
pub(crate) const R35_SELLERS_REMAINING: &str = "report-r35-sellers-remaining";

// The rows of R34, the day summary.
pub(crate) const R34_BIKRI: &str = "report-r34-bikri";
pub(crate) const R34_COMMISSION: &str = "report-r34-commission";
pub(crate) const R34_CASH_OPENING: &str = "report-r34-cash-opening";
pub(crate) const R34_CASH_IN: &str = "report-r34-cash-in";
pub(crate) const R34_CASH_OUT: &str = "report-r34-cash-out";
pub(crate) const R34_CASH_CLOSING: &str = "report-r34-cash-closing";
pub(crate) const R34_RECEIPTS: &str = "report-r34-receipts";
pub(crate) const R34_PAYMENTS: &str = "report-r34-payments";
pub(crate) const R34_ADVANCES: &str = "report-r34-advances";
pub(crate) const R34_EXPENSES: &str = "report-r34-expenses";
pub(crate) const R34_PARTNER_TAKEN: &str = "report-r34-partner-taken";
pub(crate) const R34_PARTNER_RETURNED: &str = "report-r34-partner-returned";
pub(crate) const R34_CAPITAL: &str = "report-r34-capital";
pub(crate) const R34_MONEY_LENT: &str = "report-r34-money-lent";
pub(crate) const R34_MONEY_LENT_BACK: &str = "report-r34-money-lent-back";
pub(crate) const R34_TRIPS_BOUGHT: &str = "report-r34-trips-bought";
pub(crate) const R34_TRIPS_SOLD: &str = "report-r34-trips-sold";
pub(crate) const R34_GARDENS_PAID: &str = "report-r34-gardens-paid";
pub(crate) const R34_GARDENS_IN: &str = "report-r34-gardens-in";
pub(crate) const R34_PARTNER_SHARES: &str = "report-r34-partner-shares";
pub(crate) const R34_BUYERS_DUES: &str = "report-r34-buyers-dues";
pub(crate) const R34_OPEN_LOTS: &str = "report-r34-open-lots";
pub(crate) const R34_OPEN_QTY: &str = "report-r34-open-qty";
pub(crate) const SECTION_SHOP_EXPENSES: &str = "report-section-shop-expenses";
pub(crate) const SECTION_PARTNER_MONEY: &str = "report-section-partner-money";

pub(crate) const COL_STATUS: &str = "report-col-status";

// The state of a line of R9.
pub(crate) const STATUS_PAID: &str = "report-status-paid";
pub(crate) const STATUS_DUE: &str = "report-status-due";
pub(crate) const STATUS_CANCELLED: &str = "report-status-cancelled";

// Labels of the lines of R9 and the sections of R24.
pub(crate) const SIDE_BUYER: &str = "report-side-buyer";
pub(crate) const SIDE_SELLER: &str = "report-side-seller";
pub(crate) const SECTION_PAYABLE: &str = "report-section-payable";
pub(crate) const SECTION_OWES_FIRM: &str = "report-section-owes-firm";

// Summary lines of R30.
pub(crate) const R30_INCOME: &str = "report-r30-income";
pub(crate) const R30_EXPENSES: &str = "report-r30-expenses";
pub(crate) const R30_PROFIT: &str = "report-r30-profit";

// Filters echoed in a print header, shown as the label alone.
pub(crate) const FILTER_PAID: &str = "report-filter-paid";
pub(crate) const FILTER_DUE: &str = "report-filter-due";
pub(crate) const FILTER_BY_DAY: &str = "report-filter-by-day";
pub(crate) const FILTER_BY_MONTH: &str = "report-filter-by-month";

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
/// The day of the party day sheet (R35): a sheet is for one day, not "as of" a day.
pub(crate) const FILTER_DAY: &str = "report-filter-day";
pub(crate) const FILTER_PARTY: &str = "report-filter-party";
pub(crate) const FILTER_SELLER: &str = "report-filter-seller";
pub(crate) const FILTER_ZAMINDAR: &str = "report-filter-zamindar";
pub(crate) const FILTER_ITEM: &str = "report-filter-item";
pub(crate) const FILTER_CITY: &str = "report-filter-city";
pub(crate) const FILTER_ADDA: &str = "report-filter-adda";
pub(crate) const FILTER_PARTNER: &str = "report-filter-partner";
// R33 says how it is laid out; the label is the whole fact.
pub(crate) const FILTER_TRIP_BY_TRIP: &str = "report-filter-trip-by-trip";
pub(crate) const FILTER_TRIP_BY_ITEM: &str = "report-filter-trip-by-item";
pub(crate) const FILTER_TRIP_BY_DAY: &str = "report-filter-trip-by-day";
pub(crate) const FILTER_TRIP_BY_MONTH: &str = "report-filter-trip-by-month";
pub(crate) const FILTER_TRIP_BY_PARTNER: &str = "report-filter-trip-by-partner";
pub(crate) const FILTER_DETAIL: &str = "report-filter-detail";
pub(crate) const FILTER_BALANCE_ONLY: &str = "report-filter-balance-only";

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
