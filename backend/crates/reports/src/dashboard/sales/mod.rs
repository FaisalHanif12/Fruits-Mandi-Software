//! The sales page (plan 83, PRD FR-9.9, FR-9.1, FR-19.11): the tile آج کی بکری opens one row for each
//! day of a period, built like the commission page (plan 76 and 77).
//!
//! A period is one day, a month, a year (a row for each month) or a range of days (paged). The page
//! is for the whole shop or narrowed to one party (a buyer or a seller), one item or some goods.
//! A day opens in place into the بکری sold on it.
//!
//! Every figure is read from the posted records, never worked out again:
//!
//! * بکری, نگ, خام بکری, buyer total and cash of a day: `SUM(sign × value)` of the sale facts
//!   (`analytics_facts`) joined to their sale line, with the stored `buyer_total_paisa` and
//!   `paid_paisa`. A cancelled بکری has reversal rows with sign −1, so it nets to 0;
//! * a day counts the sales posted that day, later sales and carried lots included, by the rule of
//!   the tile (`sales_by_day`), so the row of today is the tile;
//! * credit = buyer total − cash (PRD FR-12.44), worked out with checked maths;
//! * the month total of the page is the total of R2 for the same days (the sales register).
//!
//! A bucket whose figures are all 0 after the signs is left out. The totals are checked sums of the
//! bucket rows over the whole period.
//!
//! Example (E2E, 10-10-26): 2 بکری, 60 نگ, خام بکری 41,400, buyer total 44,364, cash 12,800, credit
//! 31,564. Its day opens بکری 1 (رحیم، کیلا-1: 50 نگ, 29,400, 31,564, 0) and بکری 2 (کریم، سیب-1:
//! 10 نگ, 12,000, 12,800, 12,800).

mod model;
mod print;
mod read;
mod sums;

pub use model::{
    SalesDay, SalesDayInput, SalesDayRow, SalesPeriod, SalesPeriodInput, SalesPeriodRow,
    SalesPrintInput,
};
pub use print::{sales_period_print_data, sales_period_spec};
pub use read::{sales_day, sales_period};

/// The SQL of the sales page for the query plan test (Code Standards B18): the grouped read of a
/// range for the whole shop, and the lots of one day, with the values to bind.
#[doc(hidden)]
pub mod plans {
    use mandi_db::rusqlite::types::Value;

    use super::sums::{DAY_LEN, facts_sql};

    /// The facts query of the whole shop, a sum for each day and بکری.
    pub fn facts_by_day(from: &str, to: &str) -> (String, Vec<Value>) {
        (
            facts_sql(DAY_LEN, ""),
            vec![Value::Text(from.to_owned()), Value::Text(to.to_owned())],
        )
    }

    /// The facts query narrowed to one seller, one item and one kind of goods, a sum for each month.
    pub fn facts_narrowed_by_month(
        from: &str,
        to: &str,
        seller: i64,
        item: i64,
    ) -> (String, Vec<Value>) {
        use crate::analytics::AnalyticsGoods;
        use crate::dashboard::commission::{CommissionParty, CommissionSide};
        let party = CommissionParty {
            side: CommissionSide::Seller,
            account_id: seller,
            chukta_person_key: None,
        };
        let (narrow, narrow_args) =
            super::sums::narrow_sql(Some(&party), Some(item), AnalyticsGoods::Consigned, 3);
        let mut args = vec![Value::Text(from.to_owned()), Value::Text(to.to_owned())];
        args.extend(narrow_args);
        (facts_sql(7, &narrow), args)
    }
}
