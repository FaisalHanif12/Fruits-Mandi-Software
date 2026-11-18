//! The sums of the commission page (PRD FR-9.3, FR-18.6, FR-18.7, FR-6.5): every figure is read
//! from the posted records, never worked out again.
//!
//! * seller commission, buyer commission, لاگا and the sale of a day: `SUM(sign × value)` of
//!   `analytics_facts`. A cancelled بکری has reversal rows with sign −1, so it nets to 0. Under
//!   a party the facts are narrowed to his lines, the same numbers as analytics view B (a seller)
//!   and view C (a buyer);
//! * shared income and each partner's share: the income share rows of every partner on that day.
//!   They belong to whole postings, so they are read for the whole shop only.
//!
//! Example (10-10-26, seller رحیم): sale 29,400, seller commission 1,764, buyer commission 1,764,
//! لاگا 400. As the commission on his goods: 1,764 + 1,764 + 400 = 3,928.

use std::collections::BTreeMap;

use mandi_core::money::Paisa;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::model::{
    CommissionDayRow, CommissionPartner, CommissionParty, CommissionSide, PartnerShare,
};
use crate::analytics::AnalyticsGoods;
use crate::analytics::facts::goods_condition;
use crate::error::ReportsError;
use crate::reports::common::{add, paisa};

/// What a row of the page is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shape {
    /// The whole shop (plan 76).
    Shop,
    /// One seller.
    Seller,
    /// One buyer.
    Buyer,
}

impl Shape {
    /// The shape of the page for the party asked for, if any.
    pub(super) fn of(party: Option<&CommissionParty>) -> Shape {
        match party.map(|p| p.side) {
            None => Shape::Shop,
            Some(CommissionSide::Seller) => Shape::Seller,
            Some(CommissionSide::Buyer) => Shape::Buyer,
        }
    }
}

/// One bucket's sums (a day, a month, or a posting) while they are read.
#[derive(Default, Clone)]
pub(super) struct Day {
    /// The sale: the seller's خام بکری, or the buyer's purchase value. 0 for the shop.
    pub(super) sale: Paisa,
    pub(super) seller: Paisa,
    pub(super) buyer: Paisa,
    pub(super) laga: Paisa,
    pub(super) shares: BTreeMap<i64, Paisa>,
}

impl Day {
    pub(super) fn shared(&self) -> Result<Paisa, ReportsError> {
        let mut sum = Paisa::ZERO;
        for share in self.shares.values() {
            add(&mut sum, *share)?;
        }
        Ok(sum)
    }

    /// True when the bucket has no figure at all: no commission, no لاگا, no sale and no partner
    /// share (a day whose shares net to 0 as a whole but not for each partner still shows).
    pub(super) fn is_zero(&self) -> Result<bool, ReportsError> {
        Ok(self.sale == Paisa::ZERO
            && self.seller == Paisa::ZERO
            && self.buyer == Paisa::ZERO
            && self.laga == Paisa::ZERO
            && self.shared()? == Paisa::ZERO
            && self.shares.values().all(|share| *share == Paisa::ZERO))
    }

    /// The row of this bucket, in the shape of the page.
    pub(super) fn row(
        &self,
        date: &str,
        partners: &[CommissionPartner],
        shape: Shape,
    ) -> Result<CommissionDayRow, ReportsError> {
        let mut total = Paisa::ZERO;
        let (seller, partner_shares, shared, sale, goods) = match shape {
            Shape::Shop => {
                add(&mut total, self.seller)?;
                add(&mut total, self.buyer)?;
                add(&mut total, self.laga)?;
                let shares = partners
                    .iter()
                    .map(|p| PartnerShare {
                        partner_id: p.id,
                        share_paisa: self.shares.get(&p.id).copied().unwrap_or(Paisa::ZERO),
                    })
                    .collect();
                (self.seller, Some(shares), self.shared()?, None, None)
            }
            Shape::Seller => {
                add(&mut total, self.seller)?;
                add(&mut total, self.buyer)?;
                add(&mut total, self.laga)?;
                (self.seller, None, Paisa::ZERO, Some(self.sale), Some(total))
            }
            Shape::Buyer => {
                add(&mut total, self.buyer)?;
                add(&mut total, self.laga)?;
                (Paisa::ZERO, None, Paisa::ZERO, Some(self.sale), None)
            }
        };
        Ok(CommissionDayRow {
            date: date.to_owned(),
            seller_comm_paisa: seller,
            buyer_comm_paisa: self.buyer,
            laga_paisa: self.laga,
            total_comm_paisa: total,
            shared_income_paisa: shared,
            partner_shares,
            sale_paisa: sale,
            goods_comm_paisa: goods,
        })
    }

    pub(super) fn merge(&mut self, other: &Day) -> Result<(), ReportsError> {
        add(&mut self.sale, other.sale)?;
        add(&mut self.seller, other.seller)?;
        add(&mut self.buyer, other.buyer)?;
        add(&mut self.laga, other.laga)?;
        for (id, share) in &other.shares {
            add(self.shares.entry(*id).or_default(), *share)?;
        }
        Ok(())
    }
}

/// The partners to show: every active partner and every partner with an income share in the
/// range, by id (created first first).
pub(super) fn partners_in(
    conn: &Connection,
    from: &str,
    to: &str,
) -> Result<Vec<CommissionPartner>, ReportsError> {
    let mut statement = conn.prepare(
        "SELECT p.id, p.name_ur, p.name_en FROM partners p
         WHERE p.is_active = 1 OR EXISTS (
             SELECT 1 FROM partner_share_rows r
             WHERE r.partner_id = p.id AND r.kind = 'income'
               AND r.entry_date >= ?1 AND r.entry_date <= ?2)
         ORDER BY p.id",
    )?;
    let rows = statement
        .query_map([from, to], |r| {
            Ok(CommissionPartner {
                id: r.get(0)?,
                name_ur: r.get(1)?,
                name_en: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// The narrowing of the facts to one party: the SQL text (constants of this crate) and its
/// values. The values are numbered from `first` (`?3` after two date values).
pub(super) fn party_sql(party: Option<&CommissionParty>, first: usize) -> (String, Vec<Value>) {
    let Some(party) = party else {
        return (String::new(), Vec::new());
    };
    let mut args = vec![Value::Integer(party.account_id)];
    let column = match party.side {
        CommissionSide::Seller => "seller_account_id",
        CommissionSide::Buyer => "buyer_account_id",
    };
    let mut sql = format!(" AND {column} = ?{first}");
    if let Some(key) = party
        .chukta_person_key
        .as_deref()
        .map(str::trim)
        .filter(|k| !k.is_empty())
    {
        sql.push_str(&format!(
            " AND chukta_person_key = ?{}",
            first.saturating_add(1)
        ));
        args.push(Value::Text(key.to_owned()));
    }
    (sql, args)
}

/// The narrowing of the facts to one party and to some goods: the SQL text (constants of this crate)
/// and its values. The facts table is named `f`, the name of the goods text of analytics (plan 78
/// D4). The party values are numbered from `first`.
pub(super) fn narrow_sql(
    party: Option<&CommissionParty>,
    goods: AnalyticsGoods,
    first: usize,
) -> (String, Vec<Value>) {
    let (mut sql, args) = party_sql(party, first);
    if let Some(goods) = goods_condition(goods) {
        sql.push_str(" AND ");
        sql.push_str(goods);
    }
    (sql, args)
}

/// The sale column of a shape: a seller's خام بکری or a buyer's purchase value.
const fn sale_column(shape: Shape) -> &'static str {
    match shape {
        Shape::Buyer => "buyer_value_paisa",
        Shape::Seller => "seller_amount_paisa",
        Shape::Shop => "0",
    }
}

/// The grouping of a bucket: the day itself, so the rows come in day order from the covering index
/// with no sort (migration 24), or the first `bucket_len` characters of the day (a month, a year).
fn bucket_sql(bucket_len: usize) -> String {
    if bucket_len == DAY_LEN {
        "entry_date".to_owned()
    } else {
        format!("substr(entry_date, 1, {bucket_len})")
    }
}

/// Characters of a day, `YYYY-MM-DD`.
const DAY_LEN: usize = 10;

/// The facts query of a range: the sums of each bucket. `?1` and `?2` are the first and last day;
/// `narrow` is the party's narrowing, numbered from `?3` ([`party_sql`]).
pub(super) fn facts_sql(shape: Shape, bucket_len: usize, narrow: &str) -> String {
    let bucket = bucket_sql(bucket_len);
    // The whole shop has no sale column: a constant 0 spares one multiplication and one sum for each
    // of the million facts of ten years.
    let sale = match shape {
        Shape::Shop => "0".to_owned(),
        _ => format!("SUM(sign * {})", sale_column(shape)),
    };
    format!(
        "SELECT {bucket}, {sale},
                SUM(sign * seller_comm_paisa), SUM(sign * buyer_comm_paisa),
                SUM(sign * laga_paisa)
         FROM analytics_facts f WHERE entry_date >= ?1 AND entry_date <= ?2{narrow}
         GROUP BY {bucket}"
    )
}

/// The income share rows of a range: each partner's sum for each bucket. `?1` and `?2` are the
/// first and last day.
pub(super) fn shares_sql(bucket_len: usize) -> String {
    let bucket = bucket_sql(bucket_len);
    format!(
        "SELECT {bucket}, partner_id, SUM(amount_paisa)
         FROM partner_share_rows
         WHERE kind = 'income' AND entry_date >= ?1 AND entry_date <= ?2
         GROUP BY {bucket}, partner_id"
    )
}

/// The sums of each bucket of a range (`bucket_len` 10 is a day, 7 is a month), from the facts
/// and, for the whole shop, the income share rows.
///
/// A range of days is read from two covering indexes, in day order (migration 24), so ten years of
/// the whole shop are read in about a second.
///
/// # Errors
/// A database error; a money error.
pub(super) fn buckets_in(
    conn: &Connection,
    from: &str,
    to: &str,
    bucket_len: usize,
    party: Option<&CommissionParty>,
    goods: AnalyticsGoods,
) -> Result<BTreeMap<String, Day>, ReportsError> {
    let shape = Shape::of(party);
    let mut days: BTreeMap<String, Day> = BTreeMap::new();
    let (narrow, narrow_args) = narrow_sql(party, goods, 3);
    let sql = facts_sql(shape, bucket_len, &narrow);
    let mut args = vec![Value::Text(from.to_owned()), Value::Text(to.to_owned())];
    args.extend(narrow_args);
    let mut facts = conn.prepare(&sql)?;
    let rows = facts.query_map(params_from_iter(args.iter()), |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, i64>(4)?,
        ))
    })?;
    for row in rows {
        let (date, sale, seller, buyer, laga) = row?;
        let day = days.entry(date).or_default();
        day.sale = paisa(sale)?;
        day.seller = paisa(seller)?;
        day.buyer = paisa(buyer)?;
        day.laga = paisa(laga)?;
    }
    // Partner shares belong to whole postings, not to one party's lines or to some goods.
    if shape != Shape::Shop || goods != AnalyticsGoods::All {
        return Ok(days);
    }
    let mut shares = conn.prepare(&shares_sql(bucket_len))?;
    let rows = shares.query_map([from, to], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
        ))
    })?;
    for row in rows {
        let (date, partner, amount) = row?;
        days.entry(date)
            .or_default()
            .shares
            .insert(partner, paisa(amount)?);
    }
    Ok(days)
}
