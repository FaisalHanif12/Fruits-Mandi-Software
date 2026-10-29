//! View G, trend, رجحان (PRD 18.2, FR-17.5, FR-18.6, C9, C12, C21, C25).
//!
//! One row for each day, month or year (`group_by`, month by default), oldest first, today last.
//! The rows are the series of the chart. For each period:
//!
//! * خام بکری and commission: the sale facts with their sign (FR-18.6), under the sale filters;
//!   commission is seller commission + buyer commission + لاگا, narrowed by the side filter. Each
//!   of the columns is summed apart and added with checked maths.
//! * Shared income, shop expenses and profit: the share rows of all partners (income rows,
//!   expense rows shown as a positive number, and their sum, C9, C12), or the rows of the
//!   partner picked. Share rows are kept for each posting, not for each lot, so when a sale
//!   filter is set (item, seller, buyer, city, اڈا, trip or goods) these three columns are empty.
//! * Trips: with no partner picked, the result of the trips by the day each event happened:
//!   the money the sales brought (the partners' credits of the own lots, by the day of their
//!   posting, and the money of the sales in other shops) less the cost of the goods sold (the
//!   sale facts' cost and the cost of the sales in other shops) and the cost written off (by the
//!   day of the move) (C21). With a partner picked: his money from trips in the period, the very
//!   figure of the trips column of view F (the credits less the debits on his 0604 account), so
//!   the periods of G add up to F for the same range. The two are not the same number: the
//!   partners' figures leave out their pocket parts, and the result leaves out the cost of
//!   goods still on hand. A buyer, seller, city or اڈا filter leaves the trips column empty,
//!   because the sales in other shops and the write-offs have none.
//!
//! A row opens the sale lines of its period (FR-18.2). The totals row adds the rows.
//!
//! Example (the worked day, view G by day): 10-10-26 خام بکری 41,400, commission 5,448, shared
//! income 5,488, shop expenses 3,000, profit 2,488.

use std::collections::BTreeMap;

use mandi_core::money::Paisa;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::facts::{self, conds, run_range};
use super::filters::Typed;
use super::ids;
use super::model::{AnalyticsGoods, AnalyticsGroupBy, AnalyticsKey};
use super::table::{Table, col, count_of, row};
use crate::error::ReportsError;
use crate::reports::common::{Conds, add, paisa};
use crate::reports::{Cell, ColumnKind, PageSpec};

/// The first and the last day the period may have, when it is open.
const FIRST_DAY: &str = "0000-01-01";
const LAST_DAY: &str = "9999-12-31";

/// The text length of a bucket: `YYYY-MM-DD`, `YYYY-MM` or `YYYY`.
fn bucket_len(group_by: AnalyticsGroupBy) -> usize {
    match group_by {
        AnalyticsGroupBy::Day => 10,
        AnalyticsGroupBy::Month => 7,
        AnalyticsGroupBy::Year => 4,
    }
}

/// The SQL that cuts a date column down to its bucket. The column is a constant of this crate.
pub(super) fn bucket_sql(group_by: AnalyticsGroupBy, column: &str) -> String {
    format!("substr({column}, 1, {})", bucket_len(group_by))
}

/// The first and last day of a bucket.
fn bucket_days(group_by: AnalyticsGroupBy, bucket: &str) -> Result<(String, String), ReportsError> {
    use mandi_core::dates::BusinessDate;
    let bad = || ReportsError::Data(format!("a period bucket is wrong: {bucket}"));
    match group_by {
        AnalyticsGroupBy::Day => Ok((bucket.to_owned(), bucket.to_owned())),
        AnalyticsGroupBy::Month => {
            let (year, month) = bucket.split_once('-').ok_or_else(bad)?;
            let (year, month): (i32, u32) = (
                year.parse().map_err(|_| bad())?,
                month.parse().map_err(|_| bad())?,
            );
            let first = BusinessDate::from_ymd(year, month, 1).ok_or_else(bad)?;
            let next = if month == 12 {
                BusinessDate::from_ymd(year.checked_add(1).ok_or_else(bad)?, 1, 1)
            } else {
                BusinessDate::from_ymd(year, month.checked_add(1).ok_or_else(bad)?, 1)
            }
            .ok_or_else(bad)?;
            let last = next.previous_day().ok_or_else(bad)?;
            Ok((first.to_iso(), last.to_iso()))
        }
        AnalyticsGroupBy::Year => {
            let year: i32 = bucket.parse().map_err(|_| bad())?;
            Ok((format!("{year:04}-01-01"), format!("{year:04}-12-31")))
        }
    }
}

/// The cell that names a period.
fn period_cell(group_by: AnalyticsGroupBy, bucket: &str) -> Result<Cell, ReportsError> {
    let bad = || ReportsError::Data(format!("a period bucket is wrong: {bucket}"));
    Ok(match group_by {
        AnalyticsGroupBy::Day => Cell::Date {
            iso: bucket.to_owned(),
        },
        AnalyticsGroupBy::Month => {
            let (year, month) = bucket.split_once('-').ok_or_else(bad)?;
            Cell::Month {
                year: year.parse().map_err(|_| bad())?,
                month: month.parse().map_err(|_| bad())?,
            }
        }
        AnalyticsGroupBy::Year => Cell::Number {
            value: bucket.parse().map_err(|_| bad())?,
        },
    })
}

/// True when a filter of the sales is set. The share rows and the partners' money are not kept
/// for each lot, so they cannot follow these filters.
fn sale_filter_set(typed: &Typed) -> bool {
    typed.item_id.is_some()
        || typed.seller_id.is_some()
        || typed.buyer_id.is_some()
        || typed.city_id.is_some()
        || typed.adda_id.is_some()
        || typed.trip_id.is_some()
        || typed.garden_id.is_some()
        || typed.goods != AnalyticsGoods::All
}

/// The sums of two columns by bucket.
type ByBucket = BTreeMap<String, Paisa>;

/// Runs a query that gives (bucket, amount) rows with `SUM()` of a stored `INTEGER` column.
fn sums_by_bucket(conn: &Connection, sql: &str, args: &[Value]) -> Result<ByBucket, ReportsError> {
    let mut statement = conn.prepare(sql)?;
    let rows = statement
        .query_map(params_from_iter(args.iter()), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = ByBucket::new();
    for (bucket, amount) in rows {
        add(out.entry(bucket).or_insert(Paisa::ZERO), paisa(amount)?)?;
    }
    Ok(out)
}

fn add_into(into: &mut ByBucket, from: &ByBucket, negate: bool) -> Result<(), ReportsError> {
    for (bucket, amount) in from {
        let slot = into.entry(bucket.clone()).or_insert(Paisa::ZERO);
        *slot = if negate {
            slot.checked_sub(*amount)?
        } else {
            slot.checked_add(*amount)?
        };
    }
    Ok(())
}

/// The conditions on a date column and the item and trip of a trip purchase.
fn trip_conds(
    typed: &Typed,
    date: &'static str,
    date_to: &'static str,
    item: &'static str,
    trip: &'static str,
) -> Conds {
    let mut c = Conds::new();
    c.text(date, typed.from.as_deref())
        .text(date_to, typed.to.as_deref())
        .id(item, typed.item_id)
        .id(trip, typed.trip_id);
    c
}

/// The movement of the running balances of the gardens by period: the credits less the debits on
/// their accounts dated in the period (the closing and its undo left out), less the installments
/// and costs dated in it. Over the life of a garden the periods add up to its running balance.
/// The gardens are picked by the item (their main fruit) and the garden filter, whatever the
/// period, because the period here is the day of each movement.
fn gardens_result(conn: &Connection, typed: &Typed) -> Result<ByBucket, ReportsError> {
    let group_by = typed.group_by;
    let recs = crate::gardens::pick_recs(
        conn,
        &crate::gardens::Pick {
            item_id: typed.item_id,
            garden_id: typed.garden_id,
            ..crate::gardens::Pick::default()
        },
    )?;
    let mut result = ByBucket::new();
    let first = typed.from.clone().unwrap_or_else(|| FIRST_DAY.to_owned());
    let last = typed.to.clone().unwrap_or_else(|| LAST_DAY.to_owned());
    let bucket = bucket_sql(group_by, "e.entry_date");
    let paid_bucket = bucket_sql(group_by, "x.entry_date");
    for chunk in recs.chunks(crate::reports::common::IN_CHUNK) {
        let accounts: Vec<Value> = chunk.iter().map(|r| Value::Integer(r.account.id)).collect();
        let gardens: Vec<Value> = chunk.iter().map(|r| Value::Integer(r.id)).collect();
        let marks = vec!["?"; chunk.len()].join(", ");
        let money_sql = format!(
            "SELECT {bucket}, COALESCE(SUM(l.cr_paisa), 0), COALESCE(SUM(l.dr_paisa), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE l.account_id IN ({marks}) AND e.status = 'posted' AND {}
               AND e.entry_date >= ? AND e.entry_date <= ? GROUP BY 1",
            mandi_db::garden_figures::NOT_A_CLOSING
        );
        let mut args = accounts;
        args.push(Value::Text(first.clone()));
        args.push(Value::Text(last.clone()));
        let mut statement = conn.prepare(&money_sql)?;
        let rows = statement.query_map(params_from_iter(args.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in rows {
            let (key, credits, debits) = row?;
            let slot = result.entry(key).or_insert(Paisa::ZERO);
            *slot = slot
                .checked_add(paisa(credits)?)?
                .checked_sub(paisa(debits)?)?;
        }
        let paid_sql = format!(
            "SELECT {paid_bucket}, COALESCE(SUM(x.amount_paisa), 0)
             FROM venture_entries x
             WHERE x.venture_id IN ({marks}) AND x.status = 'saved'
               AND x.kind IN ('installment', 'cost')
               AND x.entry_date >= ? AND x.entry_date <= ? GROUP BY 1"
        );
        let mut args = gardens;
        args.push(Value::Text(first.clone()));
        args.push(Value::Text(last.clone()));
        let mut statement = conn.prepare(&paid_sql)?;
        let rows = statement.query_map(params_from_iter(args.iter()), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (key, paid) = row?;
            let slot = result.entry(key).or_insert(Paisa::ZERO);
            *slot = slot.checked_sub(paisa(paid)?)?;
        }
    }
    Ok(result)
}

/// The result of the trips for each period by the day each event happened (see the module doc).
fn trips_result(conn: &Connection, typed: &Typed) -> Result<ByBucket, ReportsError> {
    let group_by = typed.group_by;
    let mut result = ByBucket::new();

    // Money from sales in our shop: the partners' credits of the own lots, on the posting day.
    let day = bucket_sql(group_by, "e.entry_date");
    let c = trip_conds(
        typed,
        "e.entry_date >= ?",
        "e.entry_date <= ?",
        "p.item_id = ?",
        "p.venture_id = ?",
    );
    let sql = format!(
        "SELECT {day}, COALESCE(SUM(c.amount_paisa), 0)
         FROM lot_credit_parts c JOIN journal_entries e ON e.id = c.journal_entry_id
         JOIN bikri_lots l ON l.id = c.lot_id JOIN purchases p ON p.id = l.purchase_id
         WHERE {} GROUP BY 1",
        c.sql()
    );
    add_into(&mut result, &sums_by_bucket(conn, &sql, c.args())?, false)?;

    // Money and cost of the sales in other shops, on their day.
    let day = bucket_sql(group_by, "x.entry_date");
    let c = trip_conds(
        typed,
        "x.entry_date >= ?",
        "x.entry_date <= ?",
        "p.item_id = ?",
        "x.venture_id = ?",
    );
    for (column, negate) in [("x.amount_paisa", false), ("x.cost_paisa", true)] {
        let sql = format!(
            "SELECT {day}, COALESCE(SUM({column}), 0)
             FROM venture_entries x JOIN purchases p ON p.id = x.purchase_id
             WHERE x.kind = 'other_sale' AND x.status = 'saved' AND {} GROUP BY 1",
            c.sql()
        );
        add_into(&mut result, &sums_by_bucket(conn, &sql, c.args())?, negate)?;
    }

    // The cost of the goods sold in our shop, from the sale facts: the +1 rows less the -1 rows.
    let day = bucket_sql(group_by, "f.entry_date");
    let c = trip_conds(
        typed,
        "f.entry_date >= ?",
        "f.entry_date <= ?",
        "f.item_id = ?",
        "f.venture_id = ?",
    );
    for (sign, negate) in [("1", true), ("-1", false)] {
        let sql = format!(
            "SELECT {day}, COALESCE(SUM(f.cost_paisa), 0)
             FROM analytics_facts f
             WHERE f.purchase_id IS NOT NULL AND f.sign = {sign} AND {} GROUP BY 1",
            c.sql()
        );
        add_into(&mut result, &sums_by_bucket(conn, &sql, c.args())?, negate)?;
    }

    // The cost written off, on the day of the move; an undone write-off is the opposite move.
    let day = bucket_sql(group_by, "m.move_date");
    let c = trip_conds(
        typed,
        "m.move_date >= ?",
        "m.move_date <= ?",
        "p.item_id = ?",
        "p.venture_id = ?",
    );
    let sql = format!(
        "SELECT {day}, COALESCE(SUM(m.cost_paisa), 0)
         FROM stock_moves m JOIN purchases p ON p.id = m.purchase_id
         WHERE (m.kind = 'written_off'
                OR (m.kind = 'reversal' AND EXISTS (
                      SELECT 1 FROM stock_moves o
                      WHERE o.id = m.reverses_move_id AND o.kind = 'written_off')))
           AND {} GROUP BY 1",
        c.sql()
    );
    add_into(&mut result, &sums_by_bucket(conn, &sql, c.args())?, true)?;
    Ok(result)
}

/// The income and the expense share rows by bucket (all partners, or the partner picked).
fn share_rows(conn: &Connection, typed: &Typed) -> Result<(ByBucket, ByBucket), ReportsError> {
    let day = bucket_sql(typed.group_by, "entry_date");
    let mut c = Conds::new();
    c.text("entry_date >= ?", typed.from.as_deref())
        .text("entry_date <= ?", typed.to.as_deref())
        .id("partner_id = ?", typed.partner_id);
    let one = |kind: &str| -> Result<ByBucket, ReportsError> {
        let sql = format!(
            "SELECT {day}, COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows
             WHERE kind = '{kind}' AND {} GROUP BY 1",
            c.sql()
        );
        sums_by_bucket(conn, &sql, c.args())
    };
    Ok((one("income")?, one("expense")?))
}

/// The dimension SQL of the facts query, by grouping.
fn fact_dim(group_by: AnalyticsGroupBy) -> &'static str {
    match group_by {
        AnalyticsGroupBy::Day => "f.entry_date",
        AnalyticsGroupBy::Month => "substr(f.entry_date, 1, 7)",
        AnalyticsGroupBy::Year => "substr(f.entry_date, 1, 4)",
    }
}

/// One period being built.
#[derive(Default)]
struct Period {
    gross: Paisa,
    commission: Paisa,
    income: Option<Paisa>,
    expenses: Option<Paisa>,
    profit: Option<Paisa>,
    trips: Option<Paisa>,
    gardens: Option<Paisa>,
}

impl Period {
    fn cells(&self, lead: Cell) -> Vec<Cell> {
        let maybe = |value: Option<Paisa>| value.map_or(Cell::Empty, |paisa| Cell::Money { paisa });
        vec![
            lead,
            Cell::Money { paisa: self.gross },
            Cell::Money {
                paisa: self.commission,
            },
            maybe(self.income),
            maybe(self.expenses),
            maybe(self.profit),
            maybe(self.trips),
            maybe(self.gardens),
        ]
    }
}

fn plus(total: &mut Option<Paisa>, amount: Option<Paisa>) -> Result<(), ReportsError> {
    if let Some(amount) = amount {
        let slot = total.get_or_insert(Paisa::ZERO);
        *slot = slot.checked_add(amount)?;
    }
    Ok(())
}

/// Builds view G.
///
/// # Errors
/// A database error; `ReportsError::Money`; `ReportsError::Data` for a fact net below 0.
pub(crate) fn run(conn: &Connection, typed: &Typed, spec: PageSpec) -> Result<Table, ReportsError> {
    let group_by = typed.group_by;
    let mut periods: BTreeMap<String, Period> = BTreeMap::new();

    // Sales and commission.
    let none = AnalyticsKey::default();
    let c = conds(typed, run_range(typed), &none);
    for group in facts::groups(conn, &c, &[fact_dim(group_by)])? {
        let Some(bucket) = group.text(0) else {
            continue;
        };
        let slot = periods.entry(bucket).or_default();
        slot.gross = group.sums.gross;
        slot.commission = group.sums.total_comm(typed.side)?;
    }

    // Shared income, expenses and profit: not when a sale filter is set.
    let plain = !sale_filter_set(typed);
    if plain {
        let (income, expense) = share_rows(conn, typed)?;
        for (bucket, amount) in &income {
            let slot = periods.entry(bucket.clone()).or_default();
            slot.income = Some(*amount);
        }
        for (bucket, amount) in &expense {
            let slot = periods.entry(bucket.clone()).or_default();
            slot.expenses = Some(Paisa::ZERO.checked_sub(*amount)?);
        }
        for slot in periods.values_mut() {
            let income = slot.income.unwrap_or(Paisa::ZERO);
            let expenses = slot.expenses.unwrap_or(Paisa::ZERO);
            slot.income = Some(income);
            slot.expenses = Some(expenses);
            slot.profit = Some(income.checked_sub(expenses)?);
        }
    }

    // Trips.
    let trips_allowed = typed.buyer_id.is_none()
        && typed.garden_id.is_none()
        && typed.seller_id.is_none()
        && typed.city_id.is_none()
        && typed.adda_id.is_none()
        && !matches!(
            typed.goods,
            AnalyticsGoods::Consigned | AnalyticsGoods::Garden
        );
    if let Some(partner_id) = typed.partner_id {
        // His money from trips by period: only when no sale filter narrows it.
        if plain && let Some(partner) = mandi_db::partners::get(conn, partner_id)? {
            // One query for all periods (Code Standards B22): the credits less the debits of his
            // trip lines, by the same bucket as the other columns. A period with sales and no
            // trip line of his shows 0.
            for slot in periods.values_mut() {
                slot.trips = Some(Paisa::ZERO);
            }
            let first = typed.from.as_deref().unwrap_or(FIRST_DAY);
            let last = typed.to.as_deref().unwrap_or(LAST_DAY);
            for (bucket, amount) in
                super::partner::trips_net_by_bucket(conn, &partner, first, last, group_by)?
            {
                periods.entry(bucket).or_default().trips = Some(amount);
            }
        }
    } else if trips_allowed {
        for (bucket, amount) in trips_result(conn, typed)? {
            periods.entry(bucket).or_default().trips = Some(amount);
        }
    }

    // Gardens: the movement of the running balances in the period. Not when a sale filter or the
    // goods or a trip leave the gardens out.
    let gardens_allowed = typed.buyer_id.is_none()
        && typed.seller_id.is_none()
        && typed.city_id.is_none()
        && typed.adda_id.is_none()
        && typed.trip_id.is_none()
        && !matches!(
            typed.goods,
            AnalyticsGoods::Consigned | AnalyticsGoods::Trip
        );
    if let Some(partner_id) = typed.partner_id {
        // His money from gardens by period (as view F's gardens column), when nothing narrows it.
        if plain
            && gardens_allowed
            && typed.garden_id.is_none()
            && let Some(partner) = mandi_db::partners::get(conn, partner_id)?
        {
            for slot in periods.values_mut() {
                slot.gardens = Some(Paisa::ZERO);
            }
            let first = typed.from.as_deref().unwrap_or(FIRST_DAY);
            let last = typed.to.as_deref().unwrap_or(LAST_DAY);
            for (bucket, amount) in
                super::partner::gardens_net_by_bucket(conn, &partner, first, last, group_by)?
            {
                periods.entry(bucket).or_default().gardens = Some(amount);
            }
        }
    } else if gardens_allowed {
        for slot in periods.values_mut() {
            slot.gardens = Some(Paisa::ZERO);
        }
        for (bucket, amount) in gardens_result(conn, typed)? {
            periods.entry(bucket).or_default().gardens = Some(amount);
        }
    }

    // A period with nothing in it has no row.
    periods.retain(|_, p| {
        p.gross != Paisa::ZERO
            || p.commission != Paisa::ZERO
            || p.income.is_some_and(|v| v != Paisa::ZERO)
            || p.expenses.is_some_and(|v| v != Paisa::ZERO)
            || p.trips.is_some_and(|v| v != Paisa::ZERO)
            || p.gardens.is_some_and(|v| v != Paisa::ZERO)
    });

    let period_kind = match group_by {
        AnalyticsGroupBy::Day => ColumnKind::Date,
        AnalyticsGroupBy::Month => ColumnKind::Month,
        AnalyticsGroupBy::Year => ColumnKind::Number,
    };
    let columns = vec![
        col("period", ids::COL_PERIOD, period_kind),
        col("gross", ids::COL_GROSS, ColumnKind::Money),
        col("commission", ids::COL_TOTAL_COMM, ColumnKind::Money),
        col("shared_income", ids::COL_SHARED_INCOME, ColumnKind::Money),
        col("shop_expenses", ids::COL_SHOP_EXPENSES, ColumnKind::Money),
        col("profit", ids::COL_PROFIT, ColumnKind::Money),
        // With a partner picked the column is his money from trips, as in view F, and is named
        // so; otherwise it is the result of the trips (money agent S5).
        col(
            "trips_result",
            if typed.partner_id.is_some() {
                ids::COL_TRIPS
            } else {
                ids::COL_TRIPS_RESULT
            },
            ColumnKind::Money,
        ),
        // The movement of the running balances of the gardens in the period (money in less what
        // was paid); with a partner picked, his money from gardens, as in view F.
        col(
            "gardens_result",
            if typed.partner_id.is_some() {
                ids::COL_GARDENS
            } else {
                ids::COL_GARDENS_RESULT
            },
            ColumnKind::Money,
        ),
    ];
    let mut grand = Period::default();
    let mut all_rows = Vec::with_capacity(periods.len());
    for (bucket, period) in &periods {
        add(&mut grand.gross, period.gross)?;
        add(&mut grand.commission, period.commission)?;
        plus(&mut grand.income, period.income)?;
        plus(&mut grand.expenses, period.expenses)?;
        plus(&mut grand.profit, period.profit)?;
        plus(&mut grand.trips, period.trips)?;
        plus(&mut grand.gardens, period.gardens)?;
        let (from, to) = bucket_days(group_by, bucket)?;
        all_rows.push(row(
            period.cells(period_cell(group_by, bucket)?),
            AnalyticsKey {
                from: Some(from),
                to: Some(to),
                ..AnalyticsKey::default()
            },
        ));
    }
    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(all_rows.len()),
        rows: spec.slice(&all_rows),
        totals: grand.cells(Cell::Empty),
    })
}
