//! The types of the sales page (plan 83, PRD FR-9.9, FR-9.1, FR-19.11).
//!
//! One page reads a period (one day, a month, a year or a range of days) and, when asked, the sales
//! of one party, one item or some goods in it. The shapes follow the commission page (plan 76 and
//! 77): a row for each day (a row for each month in a year), a totals row over the whole period, and
//! the figures the screen shows, never worked out again on the screen (CLAUDE.md rule 1).

use mandi_core::accounts::RecordRef;
use mandi_core::auth::Language;
use mandi_core::bikri::LotName;
use mandi_core::money::Paisa;
use mandi_core::printing::Paper;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::analytics::AnalyticsGoods;
use crate::dashboard::commission::{
    CommissionParty, CommissionPartyShown, CommissionPeriod, CommissionPeriodShown,
};

/// Input of `dashboard_sales_period`.
///
/// Example: `{ period: { kind: "year", year: 2026 }, goods: "garden" }`.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct SalesPeriodInput {
    /// The period: a day, a month, a year or a range. Null is the current month.
    #[serde(default)]
    #[ts(optional)]
    pub period: Option<CommissionPeriod>,
    /// Only this party's lines (a buyer or a seller). Null is the whole shop.
    #[serde(default)]
    #[ts(optional)]
    pub party: Option<CommissionParty>,
    /// Only this item. Null is every item.
    #[serde(default)]
    #[ts(type = "number | null", optional)]
    pub item_id: Option<i64>,
    /// Which goods are counted: all (the default), consigned goods, trip goods or garden goods, as the
    /// goods filter of analytics (PRD 18.1).
    #[serde(default)]
    #[ts(optional)]
    pub goods: Option<AnalyticsGoods>,
    /// Rows to skip: only a range is paged.
    #[serde(default)]
    #[ts(type = "number | null", optional)]
    pub offset: Option<u32>,
    /// Rows to return: only a range is paged. Null or 0 gives the default page of 100.
    #[serde(default)]
    #[ts(type = "number | null", optional)]
    pub limit: Option<u32>,
}

/// The figures of one day, of one month of a year, or of the whole period (the totals row, `date`
/// empty). Every money figure is a sum of stored sale-line values with their signs.
///
/// Example (10-10-26, after Events 1 to 5): 2 بکری, 60 نگ, خام بکری 41,400, buyer total 44,364, cash
/// 12,800, credit 31,564.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SalesPeriodRow {
    /// The day, `YYYY-MM-DD`; the month, `YYYY-MM`, in a year. Empty on the totals row.
    pub date: String,
    /// How many بکری have sales in this row. A بکری with sales on two days of a month counts once
    /// on each day row and once in the month row of a year, so the totals row is the sum of the rows
    /// above it, as the other columns.
    #[ts(type = "number")]
    pub bikri_count: i64,
    /// نگ sold.
    #[ts(type = "number")]
    pub qty: i64,
    /// خام بکری: the seller amounts (qty × seller rate).
    #[ts(type = "Paisa")]
    pub gross_paisa: Paisa,
    /// What the buyers were charged: value + buyer commission + لاگا of every line.
    #[ts(type = "Paisa")]
    pub buyer_total_paisa: Paisa,
    /// نقدی: paid on the spot (PRD FR-12.44).
    #[ts(type = "Paisa")]
    pub cash_paisa: Paisa,
    /// ادھار: left on the buyers' accounts, the buyer total less the cash.
    #[ts(type = "Paisa")]
    pub credit_paisa: Paisa,
}

/// What `dashboard_sales_period` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SalesPeriod {
    /// The month of the first day of the period, `YYYY-MM` (the month asked for, for a month).
    pub month: String,
    /// The year of the first day of the period.
    pub year: i32,
    /// The period the rows are for.
    pub period: CommissionPeriodShown,
    /// The party the rows are for; null for the whole shop.
    pub party: Option<CommissionPartyShown>,
    /// The item the rows are for; null for every item.
    pub item: Option<RecordRef>,
    /// The goods the rows are for.
    pub goods: AnalyticsGoods,
    /// The first month with a sale, or null when there is none. The back button stops here.
    pub first_month: Option<String>,
    /// The current month, where the next button stops.
    pub current_month: String,
    /// Today, `YYYY-MM-DD`: the row of this date is marked on the screen.
    pub today: String,
    /// The rows of this page, oldest first: one for each day (a day, a month, a range) or each month
    /// (a year) that has a sale.
    pub rows: Vec<SalesPeriodRow>,
    /// How many rows the whole period has.
    #[ts(type = "number")]
    pub total_count: i64,
    /// Rows skipped: the first row of this page is row `offset` of the period.
    #[ts(type = "number")]
    pub offset: u32,
    /// The page size used (a range is paged; the other kinds send every row).
    #[ts(type = "number")]
    pub limit: u32,
    /// The totals row over the whole period, never a sum of the page.
    pub totals: SalesPeriodRow,
}

/// Input of `dashboard_sales_day`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SalesDayInput {
    /// The day, `YYYY-MM-DD` (as the rows give it), `DD-MM-YYYY` or a shortcut (PRD FR-7.19).
    pub date: String,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return, at most 1,000. 0 gives 100.
    #[serde(default)]
    pub limit: u32,
    /// Only this party's lines. Null is the whole day.
    #[serde(default)]
    #[ts(optional)]
    pub party: Option<CommissionParty>,
    /// Only this item. Null is every item.
    #[serde(default)]
    #[ts(type = "number | null", optional)]
    pub item_id: Option<i64>,
    /// Only these goods. Null is all goods.
    #[serde(default)]
    #[ts(optional)]
    pub goods: Option<AnalyticsGoods>,
}

/// One بکری of a day with its figures of that day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SalesDayRow {
    /// The بکری, to open it.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub bikri_number: i64,
    /// The sellers of the lots sold that day, each once, in lot order (under a filter, of the
    /// matching lines only).
    pub sellers: Vec<RecordRef>,
    /// The names of those lots (PRD FR-12.45): seller, item and counter.
    pub lots: Vec<LotName>,
    /// نگ sold that day.
    #[ts(type = "number")]
    pub qty: i64,
    /// خام بکری of that day.
    #[ts(type = "Paisa")]
    pub gross_paisa: Paisa,
    /// What the buyers were charged that day.
    #[ts(type = "Paisa")]
    pub buyer_total_paisa: Paisa,
    /// Paid on the spot that day.
    #[ts(type = "Paisa")]
    pub cash_paisa: Paisa,
    /// Left on the buyers' accounts that day.
    #[ts(type = "Paisa")]
    pub credit_paisa: Paisa,
}

/// What `dashboard_sales_day` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SalesDay {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The party the rows are for; null for the whole day.
    pub party: Option<CommissionPartyShown>,
    /// The item the rows are for; null for every item.
    pub item: Option<RecordRef>,
    /// The goods the rows are for.
    pub goods: AnalyticsGoods,
    /// One page of the بکری of the day, by number.
    pub rows: Vec<SalesDayRow>,
    /// How many بکری the day has: the `bikri_count` of its row on the period page.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The day's figures: the row of the period page for this day.
    pub totals: SalesPeriodRow,
}

/// Input of `dashboard_sales_print_data` and of the sheet of kind `sales_period`: the period, the
/// filters and the print language.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct SalesPrintInput {
    /// The period; null is the current month.
    #[serde(default)]
    #[ts(optional)]
    pub period: Option<CommissionPeriod>,
    /// One party; the print header names the side and the party.
    #[serde(default)]
    #[ts(optional)]
    pub party: Option<CommissionParty>,
    /// One item; the print header names it.
    #[serde(default)]
    #[ts(type = "number | null", optional)]
    pub item_id: Option<i64>,
    /// Which goods; null is all goods. The header names a goods filter.
    #[serde(default)]
    #[ts(optional)]
    pub goods: Option<AnalyticsGoods>,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    pub paper: Option<Paper>,
}
