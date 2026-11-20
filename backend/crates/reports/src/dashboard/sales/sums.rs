//! The sums of the sales page (PRD FR-9.9, FR-18.6, FR-18.7, FR-12.44): every figure is read from
//! the posted records, never worked out again.
//!
//! One grouped query reads the sale facts (`analytics_facts`) of a range, joined to their sale line
//! (`bikri_sales`) by its key: for each day (or month) and each بکری, `SUM(sign × value)` of the
//! quantity, the seller amount, the buyer total and the money paid on the spot, and `SUM(sign)`.
//!
//! * A fact is a stored sale line value with a sign: +1 when the line was saved, −1 on the row a
//!   reversal writes. A reversal row copies the date, the dimensions and the sale line of the row it
//!   reverses, so a cancelled or edited-away line nets to 0 and a line that replaces it adds its own
//!   values on the same day (PRD FR-18.7). The stored `buyer_total_paisa` and `paid_paisa` of a sale
//!   line never change after the save (a trigger freezes them), so the sign trick is exact.
//! * A بکری counts on a day when its facts of that day add up to a sign above 0, the same rule as the
//!   tile آج کی بکری (`sales_by_day`): a later sale of an old بکری and a carried lot count on the day
//!   they were sold, and a cancelled بکری counts nowhere.
//! * The credit is the buyer total less the cash, worked out here with checked maths.
//!
//! Example (10-10-26, after Events 1 to 5): بکری 1: 50 نگ, 29,400, 31,564, 0; بکری 2: 10 نگ, 12,000,
//! 12,800, 12,800. The day: 2 بکری, 60 نگ, 41,400, 44,364, 12,800, credit 31,564.

use std::collections::BTreeMap;

use mandi_core::money::Paisa;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::model::SalesPeriodRow;
use crate::analytics::AnalyticsGoods;
use crate::analytics::facts::goods_condition;
use crate::dashboard::commission::CommissionParty;
use crate::dashboard::commission::CommissionSide;
use crate::error::ReportsError;
use crate::reports::common::{add, paisa};

/// Characters of a day, `YYYY-MM-DD`.
pub(super) const DAY_LEN: usize = 10;

/// The figures of a bucket (a day, a month, or the whole period) while they are read. Also the
/// figures of one بکری in a bucket, where `count` is 1 or 0.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct Figures {
    /// بکری counted.
    pub(super) count: i64,
    /// نگ sold.
    pub(super) qty: i64,
    /// خام بکری.
    pub(super) gross: Paisa,
    /// What the buyers were charged.
    pub(super) buyer_total: Paisa,
    /// Paid on the spot.
    pub(super) cash: Paisa,
}

fn data(what: &str) -> ReportsError {
    ReportsError::Data(format!("{what} does not fit"))
}

impl Figures {
    /// Adds another bucket's figures into this one, with checked maths.
    pub(super) fn merge(&mut self, other: &Figures) -> Result<(), ReportsError> {
        self.count = self
            .count
            .checked_add(other.count)
            .ok_or_else(|| data("the count of بکری"))?;
        self.qty = self
            .qty
            .checked_add(other.qty)
            .ok_or_else(|| data("the quantity"))?;
        add(&mut self.gross, other.gross)?;
        add(&mut self.buyer_total, other.buyer_total)?;
        add(&mut self.cash, other.cash)?;
        Ok(())
    }

    /// True when the bucket has no sale at all after the signs: nothing is shown for it.
    pub(super) fn is_zero(&self) -> bool {
        self.count == 0
            && self.qty == 0
            && self.gross == Paisa::ZERO
            && self.buyer_total == Paisa::ZERO
            && self.cash == Paisa::ZERO
    }

    /// The row of this bucket.
    ///
    /// # Errors
    /// `ReportsError::Data` if the cash is more than the buyer total (it never is: a line's paid
    /// amount is at most its total, PRD FR-12.44, so this is a bug signal and is never hidden);
    /// `ReportsError::Money` on overflow.
    pub(super) fn row(&self, date: &str) -> Result<SalesPeriodRow, ReportsError> {
        let credit = self.buyer_total.checked_sub(self.cash)?;
        if credit.as_paisa() < 0 {
            return Err(ReportsError::Data(
                "the money paid on the spot is more than the buyer total".to_owned(),
            ));
        }
        Ok(SalesPeriodRow {
            date: date.to_owned(),
            bikri_count: self.count,
            qty: self.qty,
            gross_paisa: self.gross,
            buyer_total_paisa: self.buyer_total,
            cash_paisa: self.cash,
            credit_paisa: credit,
        })
    }
}

/// The grouping of a bucket: the day itself, or the first `bucket_len` characters of the day (a
/// month, a year).
fn bucket_sql(bucket_len: usize) -> String {
    if bucket_len == DAY_LEN {
        "f.entry_date".to_owned()
    } else {
        format!("substr(f.entry_date, 1, {bucket_len})")
    }
}

/// The narrowing of the facts to one party, one item and some goods: the SQL text (constants of this
/// crate) and its values, numbered from `first` (`?3` after two date values). The facts table is
/// named `f`, as in the goods text of analytics.
pub(super) fn narrow_sql(
    party: Option<&CommissionParty>,
    item_id: Option<i64>,
    goods: AnalyticsGoods,
    first: usize,
) -> (String, Vec<Value>) {
    let mut sql = String::new();
    let mut args: Vec<Value> = Vec::new();
    let mut next = first;
    if let Some(party) = party {
        let column = match party.side {
            CommissionSide::Seller => "f.seller_account_id",
            CommissionSide::Buyer => "f.buyer_account_id",
        };
        sql.push_str(&format!(" AND {column} = ?{next}"));
        args.push(Value::Integer(party.account_id));
        next = next.saturating_add(1);
        if let Some(key) = party
            .chukta_person_key
            .as_deref()
            .map(str::trim)
            .filter(|k| !k.is_empty())
        {
            sql.push_str(&format!(" AND f.chukta_person_key = ?{next}"));
            args.push(Value::Text(key.to_owned()));
            next = next.saturating_add(1);
        }
    }
    if let Some(item) = item_id {
        sql.push_str(&format!(" AND f.item_id = ?{next}"));
        args.push(Value::Integer(item));
    }
    if let Some(goods) = goods_condition(goods) {
        sql.push_str(" AND ");
        sql.push_str(goods);
    }
    (sql, args)
}

/// The facts query of a range: for each bucket and each بکری, the net sign and the sums. `?1` and
/// `?2` are the first and last day; `narrow` is the narrowing, numbered from `?3` ([`narrow_sql`]).
///
/// Only `SUM()` on stored `INTEGER` columns and the sign; the rest is worked out in Rust. The join is
/// a `CROSS JOIN` so the facts are read first, through their date index, and each fact finds its sale
/// line by the key: the app's file has no table statistics, so the planner is not left to choose.
pub(super) fn facts_sql(bucket_len: usize, narrow: &str) -> String {
    let bucket = bucket_sql(bucket_len);
    format!(
        "SELECT {bucket}, f.bikri_id, SUM(f.sign), SUM(f.sign * f.qty),
                SUM(f.sign * f.seller_amount_paisa), SUM(f.sign * s.buyer_total_paisa),
                SUM(f.sign * s.paid_paisa)
         FROM analytics_facts f CROSS JOIN bikri_sales s ON s.id = f.sale_id
         WHERE f.entry_date >= ?1 AND f.entry_date <= ?2{narrow}
         GROUP BY {bucket}, f.bikri_id"
    )
}

/// One بکری in one bucket, as the query gives it.
#[derive(Debug, Clone, Copy)]
pub(super) struct BikriInBucket {
    /// The بکری.
    pub(super) bikri_id: i64,
    /// Its figures in the bucket (`count` is 1 when its signs add up to more than 0).
    pub(super) figures: Figures,
}

/// Reads the facts of a range, a row for each bucket and بکری, in bucket order.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does not
/// fit.
pub(super) fn bikri_in_buckets(
    conn: &Connection,
    from: &str,
    to: &str,
    bucket_len: usize,
    narrow: &str,
    narrow_args: &[Value],
) -> Result<Vec<(String, BikriInBucket)>, ReportsError> {
    let mut args = vec![Value::Text(from.to_owned()), Value::Text(to.to_owned())];
    args.extend(narrow_args.iter().cloned());
    let mut statement = conn.prepare(&facts_sql(bucket_len, narrow))?;
    let raw = statement
        .query_map(params_from_iter(args.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(raw.len());
    for (bucket, bikri_id, sign, qty, gross, buyer_total, cash) in raw {
        out.push((
            bucket,
            BikriInBucket {
                bikri_id,
                figures: Figures {
                    count: i64::from(sign > 0),
                    qty,
                    gross: paisa(gross)?,
                    buyer_total: paisa(buyer_total)?,
                    cash: paisa(cash)?,
                },
            },
        ));
    }
    Ok(out)
}

/// The sums of each bucket of a range (`bucket_len` 10 is a day, 7 is a month): the rows of
/// [`bikri_in_buckets`] folded by bucket.
///
/// # Errors
/// As [`bikri_in_buckets`].
pub(super) fn buckets_in(
    conn: &Connection,
    from: &str,
    to: &str,
    bucket_len: usize,
    narrow: &str,
    narrow_args: &[Value],
) -> Result<BTreeMap<String, Figures>, ReportsError> {
    let mut buckets: BTreeMap<String, Figures> = BTreeMap::new();
    for (bucket, one) in bikri_in_buckets(conn, from, to, bucket_len, narrow, narrow_args)? {
        buckets.entry(bucket).or_default().merge(&one.figures)?;
    }
    Ok(buckets)
}
