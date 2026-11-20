//! The commission page (plan 76 point 7, plan 77, PRD 9.1, FR-9.3 and FR-19.11 v1.41, v1.42): the
//! tile آج کا کمیشن opens one row for each day of a period.
//!
//! A period is one day, a month, a year (a row for each month) or a range of days (paged). The
//! page is for the whole shop or for one party, a buyer or a seller.
//!
//! Every figure is read from the posted records, never worked out again:
//!
//! * seller commission, buyer commission and لاگا of a day: `SUM(sign × value)` of
//!   `analytics_facts` on that day. A cancelled بکری has reversal rows with sign −1, so it nets to
//!   0 (PRD FR-18.7);
//! * total commission = seller + buyer + لاگا, the same figure as the tile and the analytics view A;
//! * shared income of a day = the income share rows of every partner on that day (PRD FR-6.5), so
//!   it can be more than the total commission (فون, ریٹ فرق, a جنرل entry on an income account);
//! * a partner's share = his income share rows on that day. The partner columns add up to the
//!   shared income, not to the total commission;
//! * under a party: the facts of his lines only, the same numbers as analytics views B and C. A
//!   party page has no partner columns, because shares belong to whole postings.
//!
//! A bucket whose columns are all 0 after the signs is left out. The totals are checked sums of the
//! bucket rows over the whole period. Expense share rows are never read: they are not income.
//!
//! Example (E2E, 10-10-26): seller 2,484, buyer 2,484, لاگا 480, total 5,448, shared income 5,488
//! (5,448 + فون 40), الف 2,196, ب 1,646, ج 1,646. For the seller رحیم: sale 29,400, seller
//! commission 1,764, commission on his goods 3,928.

mod model;
mod period;
mod print;
mod read;
mod sums;

pub use model::{
    CommissionDay, CommissionDayInput, CommissionDayRow, CommissionEntry, CommissionMonth,
    CommissionMonthInput, CommissionPartner, CommissionParty, CommissionPartyShown,
    CommissionPeriod, CommissionPeriodKind, CommissionPeriodShown, CommissionPrintInput,
    CommissionSide, PartnerShare,
};
pub use print::{commission_month_print_data, commission_month_spec};
pub(in crate::dashboard) use period::{Resolved, resolve as resolve_period};
pub use read::{commission_day, commission_month};
pub(in crate::dashboard) use read::{narrowed, resolve_party};

/// The SQL of the commission page for the query plan test (Code Standards B18): the sums of each
/// day of a range for the whole shop, with the values to bind.
#[doc(hidden)]
pub mod plans {
    use mandi_db::rusqlite::types::Value;

    use super::sums::{Shape, facts_sql, shares_sql};

    const DAY: usize = 10;

    fn range(from: &str, to: &str) -> Vec<Value> {
        vec![Value::Text(from.to_owned()), Value::Text(to.to_owned())]
    }

    /// The facts query of the whole shop, a sum for each day.
    pub fn facts_by_day(from: &str, to: &str) -> (String, Vec<Value>) {
        (facts_sql(Shape::Shop, DAY, ""), range(from, to))
    }

    /// The income share rows, a sum for each day and partner.
    pub fn shares_by_day(from: &str, to: &str) -> (String, Vec<Value>) {
        (shares_sql(DAY), range(from, to))
    }
}
