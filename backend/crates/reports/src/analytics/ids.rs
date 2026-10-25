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
pub(crate) const COL_SHARE_PERCENT: &str = "analytics-col-share-percent";
pub(crate) const COL_SHARE_KIND: &str = "analytics-col-share-kind";
pub(crate) const COL_SHARE_BASE: &str = "analytics-col-share-base";
pub(crate) const COL_SHARE_AMOUNT: &str = "analytics-col-share-amount";
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

// Column headings of views F, G and J.
pub(crate) const COL_PARTNER: &str = "report-col-partner";
pub(crate) const COL_INCOME_SHARE: &str = "report-col-income-share";
pub(crate) const COL_EXPENSE_SHARE: &str = "report-col-expense-share";
pub(crate) const COL_PROFIT_SHARE: &str = "report-col-profit-share";
pub(crate) const COL_GARDENS: &str = "analytics-col-gardens";
pub(crate) const COL_TRIPS: &str = "analytics-col-trips";
pub(crate) const COL_FRUIT: &str = "analytics-col-fruit";
pub(crate) const COL_OTHER_MONEY: &str = "analytics-col-other-money";
pub(crate) const COL_NET_CHANGE: &str = "analytics-col-net-change";
pub(crate) const COL_SHARED_INCOME: &str = "analytics-col-shared-income";
pub(crate) const COL_SHOP_EXPENSES: &str = "analytics-col-shop-expenses";
pub(crate) const COL_PROFIT: &str = "analytics-col-profit";
pub(crate) const COL_TRIPS_RESULT: &str = "analytics-col-trips-result";
pub(crate) const COL_TRIP: &str = "report-col-trip";
pub(crate) const COL_PURCHASE: &str = "report-col-purchase";
pub(crate) const COL_BOUGHT_QTY: &str = "report-col-bought-qty";
pub(crate) const COL_BOUGHT_COST: &str = "report-col-bought-cost";
pub(crate) const COL_SOLD_QTY: &str = "report-col-sold";
pub(crate) const COL_SALES_MONEY: &str = "report-col-sales-money";
pub(crate) const COL_COST_SOLD: &str = "report-col-cost-sold";
pub(crate) const COL_WRITTEN_OFF: &str = "report-col-written-off";
pub(crate) const COL_WRITTEN_OFF_COST: &str = "report-col-written-off-cost";
pub(crate) const COL_LEFT_QTY: &str = "report-col-left-qty";
pub(crate) const COL_LEFT_COST: &str = "report-col-left-cost";
pub(crate) const COL_RESULT: &str = "report-col-result";
pub(crate) const COL_PARTNER_SHARE: &str = "analytics-col-partner-share";
pub(crate) const COL_PARTNER_PART: &str = "analytics-col-partner-part";

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
pub(crate) const FILTER_PARTNER: &str = "report-filter-partner";
pub(crate) const FILTER_TRIP: &str = "analytics-filter-trip";
pub(crate) const FILTER_GARDEN: &str = "analytics-filter-garden";
pub(crate) const COL_GARDEN: &str = "analytics-col-garden";
pub(crate) const COL_PARTNER_RESULT: &str = "analytics-col-partner-result";
pub(crate) const COL_OTHER_IN: &str = "analytics-col-other-in";
pub(crate) const COL_GARDEN_STATUS: &str = "analytics-col-garden-status";
pub(crate) const COL_PAID_SHOP: &str = "analytics-col-paid-shop";
pub(crate) const COL_PAID_POCKET: &str = "analytics-col-paid-pocket";
pub(crate) const COL_SALES_OUR_SHOP: &str = "analytics-col-sales-our-shop";
pub(crate) const COL_SALES_OTHER_SHOPS: &str = "analytics-col-sales-other-shops";
pub(crate) const COL_GARDEN_RESULT: &str = "analytics-col-garden-result";
pub(crate) const COL_GARDENS_RESULT: &str = "analytics-col-gardens-result";
pub(crate) const FILTER_GOODS_CONSIGNED: &str = "analytics-filter-goods-consigned";
pub(crate) const FILTER_GOODS_TRIP: &str = "analytics-filter-goods-trip";
pub(crate) const FILTER_GOODS_GARDEN: &str = "analytics-filter-goods-garden";
pub(crate) const FILTER_TRIP_BY_TRIP: &str = "report-filter-trip-by-trip";
pub(crate) const FILTER_TRIP_BY_PURCHASE: &str = "analytics-filter-trip-by-purchase";
pub(crate) const FILTER_TRIP_BY_ITEM: &str = "report-filter-trip-by-item";
pub(crate) const FILTER_TRIP_BY_DAY: &str = "report-filter-trip-by-day";
pub(crate) const FILTER_COMPARE_FROM: &str = "analytics-filter-compare-from";
pub(crate) const FILTER_COMPARE_TO: &str = "analytics-filter-compare-to";

/// The footer of the shared print layout.
pub(crate) const PAGE_FOOTER: &str = "print-page-label";
