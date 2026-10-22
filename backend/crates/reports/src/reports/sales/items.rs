//! R7 قسم مال وار بکری (item-wise sales) with the commission columns of analytics view A (PRD 19.2,
//! 18.2, FR-18.6, FR-18.7).
//!
//! The figures are the stored values of each sale line, read from `analytics_facts` (never
//! worked out again with today's rules). A fact carries a sign: +1 for a sale, −1 for the row a
//! reversal writes, so a cancelled or edited posting adds nothing. A column total is the sum of
//! the +1 rows less the sum of the −1 rows: two `SUM()` calls on stored `INTEGER` columns, and
//! the difference in Rust with checked maths (money agent Section 9; no multiplication or
//! subtraction in SQL).
//!
//! The average rate is for display only (PRD C18): `avg_rate(gross, qty)`.
//!
//! Example (PRD 18.4, view A, the worked day): کیلا 50 نگ, خام 29,400, average 588, seller
//! commission 1,764, buyer commission 1,764, لاگا 400, total commission 3,928; سیب 10 نگ, خام
//! 12,000, average 1,200, total commission 1,520; the total row 60 نگ, 41,400, 5,448.

use mandi_core::money::{Paisa, Qty, avg_rate};
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{Conds, add, col, count_of, names_cell, paisa};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{Cell, ColumnKind, ReportId, ReportPage, Row, RowKind};

/// The conditions of R7, with only the filters that are set (Code Standards B18): the date range
/// is found through the date index of the facts.
fn conds(typed: &Typed) -> Conds {
    let mut c = Conds::new();
    c.text("f.entry_date >= ?", typed.from.as_deref())
        .text("f.entry_date <= ?", typed.to.as_deref())
        .id("f.seller_account_id = ?", typed.seller_id)
        .id("f.item_id = ?", typed.item_id)
        .id("f.buyer_account_id = ?", typed.party_id)
        .text("f.pukhta_person_key = ?", typed.person_key.as_deref())
        .id("f.city_id = ?", typed.city_id)
        .id("f.adda_id = ?", typed.adda_id);
    c
}

/// The query of R7: one row for each item with facts in the filter. Each sum is split into the
/// +1 rows and the −1 rows, so the difference is worked out in Rust with checked maths.
pub(crate) fn query(typed: &Typed) -> (String, Vec<Value>) {
    let c = conds(typed);
    let sql = format!(
        "SELECT f.item_id, i.name_ur, i.name_en,
        COALESCE(SUM(CASE WHEN f.sign = 1 THEN f.qty END), 0),
        COALESCE(SUM(CASE WHEN f.sign = -1 THEN f.qty END), 0),
        COALESCE(SUM(CASE WHEN f.sign = 1 THEN f.seller_amount_paisa END), 0),
        COALESCE(SUM(CASE WHEN f.sign = -1 THEN f.seller_amount_paisa END), 0),
        COALESCE(SUM(CASE WHEN f.sign = 1 THEN f.seller_comm_paisa END), 0),
        COALESCE(SUM(CASE WHEN f.sign = -1 THEN f.seller_comm_paisa END), 0),
        COALESCE(SUM(CASE WHEN f.sign = 1 THEN f.buyer_comm_paisa END), 0),
        COALESCE(SUM(CASE WHEN f.sign = -1 THEN f.buyer_comm_paisa END), 0),
        COALESCE(SUM(CASE WHEN f.sign = 1 THEN f.laga_paisa END), 0),
        COALESCE(SUM(CASE WHEN f.sign = -1 THEN f.laga_paisa END), 0)
        FROM analytics_facts f JOIN items i ON i.id = f.item_id
        WHERE {}
        GROUP BY f.item_id ORDER BY i.code",
        c.sql()
    );
    (sql, c.args().to_vec())
}

/// The net of one fact column: the +1 rows less the −1 rows.
fn net(plus: i64, minus: i64) -> Result<i64, ReportsError> {
    plus.checked_sub(minus)
        .ok_or_else(|| ReportsError::Data("a fact total does not fit".to_owned()))
}

struct Item {
    names: (String, String),
    qty: i64,
    amounts: [Paisa; 4],
}

/// Builds R7.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a total
/// does not fit.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let (sql, args) = query(typed);
    let mut statement = conn.prepare(&sql)?;
    let raw = statement
        .query_map(params_from_iter(args.iter()), |r| {
            let mut sums = [0_i64; 10];
            for (slot, index) in sums.iter_mut().zip(3_usize..) {
                *slot = r.get(index)?;
            }
            Ok((r.get::<_, String>(1)?, r.get::<_, String>(2)?, sums))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut items = Vec::with_capacity(raw.len());
    let mut total_qty = 0_i64;
    let mut totals = [Paisa::ZERO; 4];
    for (name_ur, name_en, s) in raw {
        let qty = net(s[0], s[1])?;
        let amounts = [
            paisa(net(s[2], s[3])?)?,
            paisa(net(s[4], s[5])?)?,
            paisa(net(s[6], s[7])?)?,
            paisa(net(s[8], s[9])?)?,
        ];
        // An item whose every posting was reversed adds nothing (FR-18.7): no row.
        if qty == 0 && amounts.iter().all(|a| *a == Paisa::ZERO) {
            continue;
        }
        total_qty = total_qty
            .checked_add(qty)
            .ok_or_else(|| ReportsError::Data("the quantity total does not fit".to_owned()))?;
        for (total, amount) in totals.iter_mut().zip(amounts) {
            add(total, amount)?;
        }
        items.push(Item {
            names: (name_ur, name_en),
            qty,
            amounts,
        });
    }

    let mut all_rows = Vec::with_capacity(items.len());
    for item in &items {
        let [gross, seller_comm, buyer_comm, laga] = item.amounts;
        let mut total_comm = seller_comm;
        add(&mut total_comm, buyer_comm)?;
        add(&mut total_comm, laga)?;
        all_rows.push(Row {
            cells: vec![
                names_cell(Some(item.names.0.clone()), Some(item.names.1.clone())),
                Cell::Number { value: item.qty },
                Cell::Money { paisa: gross },
                average(gross, item.qty)?,
                Cell::Money { paisa: seller_comm },
                Cell::Money { paisa: buyer_comm },
                Cell::Money { paisa: laga },
                Cell::Money { paisa: total_comm },
            ],
            source: None,
            kind: RowKind::Normal,
        });
    }
    let [gross, seller_comm, buyer_comm, laga] = totals;
    let mut total_comm = seller_comm;
    add(&mut total_comm, buyer_comm)?;
    add(&mut total_comm, laga)?;

    Ok(ReportPage {
        id: ReportId::R7,
        title_id: ids::title_id("R7"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("item", ids::COL_ITEM, ColumnKind::Names),
            col("qty", ids::COL_QTY, ColumnKind::Number),
            col("gross", ids::COL_GROSS, ColumnKind::Money),
            col("avg_rate", ids::COL_AVG_RATE, ColumnKind::Money),
            col("seller_comm", ids::COL_SELLER_COMM, ColumnKind::Money),
            col("buyer_comm", ids::COL_BUYER_COMM, ColumnKind::Money),
            col("laga", ids::COL_LAGA, ColumnKind::Money),
            col("total_comm", ids::COL_TOTAL_COMM, ColumnKind::Money),
        ],
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count_of(all_rows.len()),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Number { value: total_qty },
            Cell::Money { paisa: gross },
            Cell::Empty,
            Cell::Money { paisa: seller_comm },
            Cell::Money { paisa: buyer_comm },
            Cell::Money { paisa: laga },
            Cell::Money { paisa: total_comm },
        ],
        summary: Vec::new(),
    })
}

/// The average rate of a row (PRD C18), or an empty cell when there is no quantity.
fn average(gross: Paisa, qty: i64) -> Result<Cell, ReportsError> {
    if qty <= 0 || gross.is_negative() {
        return Ok(Cell::Empty);
    }
    let qty = Qty::new(qty)?;
    Ok(Cell::Money {
        paisa: avg_rate(gross, qty)?,
    })
}
