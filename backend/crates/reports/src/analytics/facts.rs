//! Sums of the sale facts (`analytics_facts`, Architecture 15.6), the one source of every sales
//! figure in analytics (PRD FR-18.6, FR-18.7, money agent Section 9).
//!
//! A fact is a stored sale line value with a sign: +1 when the line was saved and −1 on the row a
//! reversal writes. A reversal row copies the date and every dimension of the row it reverses
//! (`mandi_db::bikri::edit`), so inside any group the −1 rows can never be more than the +1 rows.
//!
//! Each column is summed twice in SQL, once over the +1 rows and once over the −1 rows, with
//! `SUM()` on the stored `INTEGER` columns and no other arithmetic in SQL. The net is worked out
//! in Rust with checked maths. A net below 0 is a bug signal and an error, never shown or hidden.
//! A group whose net quantity and amounts are all 0 (a cancelled or fully edited-away sale) has
//! no row (FR-18.7).
//!
//! Example (PRD 18.4, view A, کیلا): +1 rows qty 50, seller amount 29,400, seller commission
//! 1,764, buyer commission 1,764, لاگا 400; no −1 rows. The net is the same; the total
//! commission is 1,764 + 1,764 + 400 = 3,928.

use mandi_core::money::Paisa;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::filters::Typed;
use super::model::{AnalyticsGoods, AnalyticsKey, AnalyticsSide};
use crate::error::ReportsError;
use crate::reports::common::{Conds, add, paisa};

/// The summed columns, in query order.
const COLUMNS: [&str; 6] = [
    "qty",
    "seller_amount_paisa",
    "seller_comm_paisa",
    "buyer_value_paisa",
    "buyer_comm_paisa",
    "laga_paisa",
];

/// The net figures of one group: the +1 rows less the −1 rows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Sums {
    /// نگ.
    pub(crate) qty: i64,
    /// خام بکری: the seller amounts.
    pub(crate) gross: Paisa,
    /// The seller commission.
    pub(crate) seller_comm: Paisa,
    /// The value the buyers were charged before commission and لاگا.
    pub(crate) buyer_value: Paisa,
    /// The buyer commission.
    pub(crate) buyer_comm: Paisa,
    /// لاگا.
    pub(crate) laga: Paisa,
}

fn net(plus: i64, minus: i64, what: &str) -> Result<i64, ReportsError> {
    match plus.checked_sub(minus) {
        Some(value) if value >= 0 => Ok(value),
        _ => Err(ReportsError::Data(format!(
            "the net of {what} in a fact group is below 0"
        ))),
    }
}

impl Sums {
    /// The net of the twelve sums of a group (six columns, +1 rows then −1 rows).
    fn from_raw(raw: &[i64; 12]) -> Result<Self, ReportsError> {
        let at = |index: usize| raw.get(index).copied().unwrap_or(0);
        let pair = |index: usize| -> Result<i64, ReportsError> {
            net(
                at(index),
                at(index.saturating_add(6)),
                COLUMNS.get(index).copied().unwrap_or("a column"),
            )
        };
        Ok(Self {
            qty: pair(0)?,
            gross: paisa(pair(1)?)?,
            seller_comm: paisa(pair(2)?)?,
            buyer_value: paisa(pair(3)?)?,
            buyer_comm: paisa(pair(4)?)?,
            laga: paisa(pair(5)?)?,
        })
    }

    /// True when nothing is left of the group (FR-18.7).
    pub(crate) fn is_zero(&self) -> bool {
        self.qty == 0
            && self.gross == Paisa::ZERO
            && self.seller_comm == Paisa::ZERO
            && self.buyer_value == Paisa::ZERO
            && self.buyer_comm == Paisa::ZERO
            && self.laga == Paisa::ZERO
    }

    /// Seller commission plus buyer commission plus لاگا ("total comm", PRD 18.2), or the part
    /// of it on the side asked for.
    pub(crate) fn total_comm(&self, side: AnalyticsSide) -> Result<Paisa, ReportsError> {
        let mut total = Paisa::ZERO;
        match side {
            AnalyticsSide::Both => {
                add(&mut total, self.seller_comm)?;
                add(&mut total, self.buyer_comm)?;
                add(&mut total, self.laga)?;
            }
            AnalyticsSide::Seller => add(&mut total, self.seller_comm)?,
            AnalyticsSide::Buyer => {
                add(&mut total, self.buyer_comm)?;
                add(&mut total, self.laga)?;
            }
        }
        Ok(total)
    }

    /// Adds another group to this one, with checked maths.
    pub(crate) fn add(&mut self, other: &Sums) -> Result<(), ReportsError> {
        self.qty = self
            .qty
            .checked_add(other.qty)
            .ok_or_else(|| ReportsError::Data("the quantity total does not fit".to_owned()))?;
        add(&mut self.gross, other.gross)?;
        add(&mut self.seller_comm, other.seller_comm)?;
        add(&mut self.buyer_value, other.buyer_value)?;
        add(&mut self.buyer_comm, other.buyer_comm)?;
        add(&mut self.laga, other.laga)?;
        Ok(())
    }
}

/// One group of a query: the values of its dimensions and its net figures.
#[derive(Debug, Clone)]
pub(crate) struct Group {
    /// The dimension values, in the order asked for.
    pub(crate) dims: Vec<Value>,
    /// The net figures.
    pub(crate) sums: Sums,
}

impl Group {
    /// An integer dimension.
    pub(crate) fn int(&self, index: usize) -> Option<i64> {
        match self.dims.get(index) {
            Some(Value::Integer(value)) => Some(*value),
            _ => None,
        }
    }

    /// A text dimension.
    pub(crate) fn text(&self, index: usize) -> Option<String> {
        match self.dims.get(index) {
            Some(Value::Text(value)) => Some(value.clone()),
            _ => None,
        }
    }
}

/// The conditions on the facts for a run: the date range given (the run's or the compared one),
/// the picked records, and the extra narrowing of a drill-down key.
pub(crate) fn conds(
    typed: &Typed,
    range: (Option<&str>, Option<&str>),
    key: &AnalyticsKey,
) -> Conds {
    let mut c = dated(range);
    c.text("f.entry_date >= ?", key.from.as_deref())
        .text("f.entry_date <= ?", key.to.as_deref())
        .id("f.item_id = ?", typed.item_id)
        .id("f.item_id = ?", key.item_id)
        .not_in("f.item_id", &key.not_item_ids)
        .id("f.seller_account_id = ?", typed.seller_id)
        .id("f.seller_account_id = ?", key.seller_id)
        .id("f.buyer_account_id = ?", typed.buyer_id)
        .id("f.buyer_account_id = ?", key.buyer_id)
        .text("f.pukhta_person_key = ?", typed.person_key.as_deref())
        .text("f.pukhta_person_key = ?", key.person_key.as_deref())
        .id("f.city_id = ?", typed.city_id)
        .id("f.adda_id = ?", typed.adda_id)
        .id("f.venture_id = ?", typed.trip_id)
        // The goods of one garden: the lots whose seller is its account (FR-12.36).
        .id("f.seller_account_id = ?", typed.garden_account_id)
        .in_list("f.purchase_id", &key.purchase_ids);
    match typed.goods {
        AnalyticsGoods::All => {}
        // Consigned goods have no venture and are not a garden's (PRD 18.1).
        AnalyticsGoods::Consigned => {
            c.always(
                "f.venture_id IS NULL AND f.seller_account_id NOT IN
                 (SELECT account_id FROM ventures WHERE kind = 'garden' AND account_id IS NOT NULL)",
            );
        }
        AnalyticsGoods::Trip => {
            c.always("f.venture_id IN (SELECT id FROM ventures WHERE kind = 'trip')");
        }
        AnalyticsGoods::Garden => {
            c.always(
                "(f.venture_id IN (SELECT id FROM ventures WHERE kind = 'garden')
                  OR f.seller_account_id IN
                     (SELECT account_id FROM ventures WHERE kind = 'garden' AND account_id IS NOT NULL))",
            );
        }
    }
    c
}

/// The date range view J reads its commission in: none. J picks a purchase by the day it was
/// bought, and its sales can fall on any later day (R33 reads them the same way).
pub(crate) const TRIPS_RANGE: (Option<&str>, Option<&str>) = (None, None);

/// The filters view J reads the sale facts of its purchases under, and the lines behind a row of
/// J open them under: the run's filters without the item. The item picks the purchases; the
/// commission of a purchase is of all its sales (money agent M1, M2).
pub(crate) fn for_trips(typed: &Typed) -> Typed {
    Typed {
        item_id: None,
        ..typed.clone()
    }
}

/// The conditions of a date range alone, found through the date index of the facts (Code
/// Standards B18).
pub(crate) fn dated(range: (Option<&str>, Option<&str>)) -> Conds {
    let mut c = Conds::new();
    c.text("f.entry_date >= ?", range.0)
        .text("f.entry_date <= ?", range.1);
    c
}

/// The date range of the run itself.
pub(crate) fn run_range(typed: &Typed) -> (Option<&str>, Option<&str>) {
    (typed.from.as_deref(), typed.to.as_deref())
}

/// The date range of the compared period.
pub(crate) fn compare_range(typed: &Typed) -> (Option<&str>, Option<&str>) {
    typed
        .compare
        .as_ref()
        .map_or((None, None), |(from, to)| (Some(from), Some(to)))
}

/// The query of [`groups`]: the dimension columns, then the +1 sums and the −1 sums of the six
/// columns, grouped by the dimensions.
pub(crate) fn groups_sql(conds: &Conds, dims: &[&'static str]) -> String {
    let dimension_list = dims.join(", ");
    let mut sums_sql = String::new();
    for sign in ["1", "-1"] {
        for column in COLUMNS {
            sums_sql.push_str(&format!(
                ", COALESCE(SUM(CASE WHEN f.sign = {sign} THEN f.{column} END), 0)"
            ));
        }
    }
    let group_by = if dims.is_empty() {
        String::new()
    } else {
        format!("GROUP BY {dimension_list}")
    };
    let select = if dims.is_empty() {
        sums_sql.trim_start_matches(", ").to_owned()
    } else {
        format!("{dimension_list}{sums_sql}")
    };
    format!(
        "SELECT {select} FROM analytics_facts f WHERE {} {group_by}",
        conds.sql()
    )
}

/// The groups of the facts under `conds`, one for each combination of the dimension columns
/// (constants of this crate such as `f.item_id`). Groups with nothing left are left out.
///
/// # Errors
/// A database error; `ReportsError::Data` for a net below 0; `ReportsError::Money`.
pub(crate) fn groups(
    conn: &Connection,
    conds: &Conds,
    dims: &[&'static str],
) -> Result<Vec<Group>, ReportsError> {
    let sql = groups_sql(conds, dims);
    let mut statement = conn.prepare(&sql)?;
    let width = dims.len();
    let raw = statement
        .query_map(params_from_iter(conds.args().iter()), |r| {
            let mut dim_values = Vec::with_capacity(width);
            for index in 0..width {
                dim_values.push(r.get::<_, Value>(index)?);
            }
            let mut sums = [0_i64; 12];
            for (slot, index) in sums.iter_mut().zip(width..) {
                *slot = r.get(index)?;
            }
            Ok((dim_values, sums))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut found = Vec::with_capacity(raw.len());
    for (dim_values, sums) in raw {
        let sums = Sums::from_raw(&sums)?;
        if !sums.is_zero() {
            found.push(Group {
                dims: dim_values,
                sums,
            });
        }
    }
    Ok(found)
}

/// The sums over all facts under `conds`, with no grouping.
///
/// # Errors
/// As [`groups`].
pub(crate) fn total(conn: &Connection, conds: &Conds) -> Result<Sums, ReportsError> {
    let mut all = groups(conn, conds, &[])?;
    Ok(all.pop().map_or_else(Sums::default, |g| g.sums))
}
