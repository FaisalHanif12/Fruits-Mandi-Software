//! The Fluent message IDs the analytics commands send to the screen and the print template
//! (CLAUDE.md rule 7). The frontend defines them in `analytics.ftl`, in Urdu and in English, with
//! the glossary words for the business terms. Columns that mean the same as in the reports reuse
//! the `report-col-*` IDs of the report engine. Keep this list in step with `analytics.ftl`. No
//! sentence is built here.

use super::model::AnalyticsView;

/// The title of a view: `analytics-view-a-title` and so on.
pub(crate) fn title_id(view: AnalyticsView) -> String {
    format!("analytics-view-{}-title", view.code().to_lowercase())
}

// The title of the lines behind a number.
pub(crate) const LINES_TITLE: &str = "analytics-lines-title";

// The totals row.
pub(crate) const TOTAL: &str = "report-total";

// Column headings of analytics.
pub(crate) const COL_ITEM: &str = "report-col-item";
pub(crate) const COL_SELLER: &str = "report-col-seller";
pub(crate) const COL_BUYER: &str = "report-col-buyer";
pub(crate) const COL_QTY: &str = "report-col-qty";
pub(crate) const COL_GROSS: &str = "report-col-gross";
pub(crate) const COL_AVG_RATE: &str = "report-col-avg-rate";
pub(crate) const COL_SELLER_COMM: &str = "report-col-seller-comm";
pub(crate) const COL_BUYER_COMM: &str = "report-col-buyer-comm";
pub(crate) const COL_LAGA: &str = "report-col-laga";
pub(crate) const COL_TOTAL_COMM: &str = "report-col-total-comm";
pub(crate) const COL_NET: &str = "report-col-net";
pub(crate) const COL_DATE: &str = "report-col-date";
pub(crate) const COL_DOC: &str = "report-col-doc";
pub(crate) const COL_SHARE: &str = "analytics-col-share";
pub(crate) const COL_BIKRI_COUNT: &str = "analytics-col-bikri-count";
pub(crate) const COL_PURCHASE_VALUE: &str = "analytics-col-purchase-value";
pub(crate) const COL_DUES_NOW: &str = "analytics-col-dues-now";
pub(crate) const COL_AVG_DAYS: &str = "analytics-col-avg-days-to-pay";
pub(crate) const COL_COMPARE: &str = "analytics-col-compare";
pub(crate) const COL_CHANGE: &str = "analytics-col-change";
pub(crate) const COL_RANK: &str = "analytics-col-rank";
pub(crate) const COL_NAME: &str = "analytics-col-name";
pub(crate) const COL_PERIOD: &str = "analytics-col-period";
pub(crate) const COL_ITEM_COLUMN: &str = "analytics-col-item-column";
pub(crate) const COL_ROW_TOTAL: &str = "analytics-col-row-total";

// Section headings of view H.
pub(crate) const TOP_ITEMS: &str = "analytics-top-items";
pub(crate) const TOP_SELLERS: &str = "analytics-top-sellers";
pub(crate) const TOP_BUYERS: &str = "analytics-top-buyers";

// Header facts of a print: the label is the whole fact, so the value is empty (or a date).
pub(crate) const FILTER_FROM: &str = "report-filter-from";
pub(crate) const FILTER_TO: &str = "report-filter-to";
pub(crate) const FILTER_ITEM: &str = "report-filter-item";
pub(crate) const FILTER_SELLER: &str = "report-filter-seller";
pub(crate) const FILTER_BUYER: &str = "analytics-filter-buyer";
pub(crate) const FILTER_CITY: &str = "report-filter-city";
pub(crate) const FILTER_ADDA: &str = "report-filter-adda";
pub(crate) const FILTER_SIDE_SELLER: &str = "analytics-filter-side-seller";
pub(crate) const FILTER_SIDE_BUYER: &str = "analytics-filter-side-buyer";
pub(crate) const FILTER_BY_DAY: &str = "report-filter-by-day";
pub(crate) const FILTER_BY_MONTH: &str = "report-filter-by-month";
pub(crate) const FILTER_BY_YEAR: &str = "analytics-filter-by-year";
pub(crate) const FILTER_COMPARE_FROM: &str = "analytics-filter-compare-from";
pub(crate) const FILTER_COMPARE_TO: &str = "analytics-filter-compare-to";

/// The footer of the shared print layout.
pub(crate) const PAGE_FOOTER: &str = "print-page-label";
