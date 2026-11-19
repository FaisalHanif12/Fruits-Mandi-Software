//! The list of what was posted on a day (PRD FR-12.46, FR-9.1, FR-6.1; Umar, 19-11-2026, plan 80
//! R5-1). A row is one بکری on one day it was posted: the first save, a later sale, or a lot carried
//! into a newer بکری. Its figures are the stored figures of that day's postings only, so the rows of a
//! day add up to the dashboard tile آج کی بکری, the commission page and R34, which read the same
//! analytics facts (PRD FR-6.1). The صافی of a row is worked out here, in Rust, with checked
//! operations: its day's خام, less its day's seller commission, less the lot charges of the day the
//! lots were first posted (a later sale has none, PRD FR-12.17).
//!
//! A saved بکری is on a day when its sale lines of that day add up to more than 0 (the rule of the
//! tile). A cancelled بکری has facts that add up to 0 on every day it was posted, so it is listed on
//! those days and shows its lines as they stood when it was cancelled (`ended = 'cancelled'`); the
//! sums over a filter leave it out, like the totals of the list.
//!
//! The dates of the filter are on the postings (`analytics_facts.entry_date`, one covering index,
//! `idx_analytics_facts_views`), grouped once; the بکری are then joined by id (Code Standards B18,
//! B22, NFR-SPD-24). Only `SUM()` on INTEGER columns is used for money.
//!
//! Example (PRD E2E-16 and E2E-16b): 45 of 50 are sold on 10-10-26, the last 5 on 11-10-26. The list
//! of 10-10-26 has بکری 1 with خام 26,500 and صافی 16,590; the list of 11-10-26 has it again with
//! خام 2,800 and صافی 2,632 (the seller credit of the later sale). The two صافی add up to 19,222,
//! the صافی of the lot.

use mandi_core::bikri::BikriListRow;
use mandi_core::edit::EntryFlags;
use rusqlite::types::Value;
use rusqlite::{Connection, params_from_iter};

use super::saved::{
    Cols, DayRec, ListFilter, ListTotals, MonthRec, fill_sellers_and_items, where_of,
};

/// A row of the list, with the day the بکری itself was first posted. The edit flags of a row follow
/// that day (the Edit button works on the whole بکری), not the day of the row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostedRow {
    /// The row. `entry_date` is the day of its postings.
    pub row: BikriListRow,
    /// The بکری's own date (`bikri_entries.entry_date`), `YYYY-MM-DD`.
    pub own_date: String,
}

/// The columns a list of postings can be sorted by: the field of the row and its SQL. They are
/// the output columns of [`list_query`].
pub const SORT_COLUMNS: [(&str, &str); 9] = [
    ("number", "number"),
    ("entry_date", "entry_date"),
    ("truck_no", "truck_no"),
    ("status", "status"),
    ("qty_received", "qty_received"),
    ("qty_sold", "qty_sold"),
    ("gross", "gross"),
    ("net", "(gross - comm - charges)"),
    ("buyer_total", "buyer_total"),
];

/// A figure of a cancelled بکری: the sum of its lines as they stood when it was cancelled, on the
/// day of the row; of any other بکری: the sum of the day's facts.
fn figure(from_lines: &str, from_facts: &str) -> String {
    format!(
        "CASE WHEN e.status = 'cancelled' THEN COALESCE((SELECT SUM(s.{from_lines}) FROM bikri_sales s
              WHERE s.bikri_id = e.id AND s.entry_date = p.day AND s.ended = 'cancelled'), 0)
         ELSE p.{from_facts} END"
    )
}

/// The rows of the list before the sort: one for each بکری and day the filter matches, with their
/// figures, and the values the query binds (the two dates, then the conditions on the بکری).
fn inner_query(filter: &ListFilter) -> (String, Vec<Value>) {
    let mut binds: Vec<Value> = Vec::new();
    let mut dates: Vec<&str> = Vec::new();
    if let Some(from) = &filter.from {
        dates.push("f.entry_date >= ?");
        binds.push(Value::Text(from.clone()));
    }
    if let Some(to) = &filter.to {
        dates.push("f.entry_date <= ?");
        binds.push(Value::Text(to.clone()));
    }
    let on_postings = if dates.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", dates.join(" AND "))
    };
    let (condition, more) = where_of(filter, false);
    binds.extend(more);
    // The conditions on the بکری follow the one that keeps a day it was really sold on.
    let and = condition.replacen(" WHERE ", " AND ", 1);
    let sql = format!(
        "WITH p AS (
            SELECT f.bikri_id AS bikri_id, f.entry_date AS day,
                   SUM(f.sign * f.seller_amount_paisa) AS gross,
                   SUM(f.sign * f.seller_comm_paisa) AS comm,
                   SUM(f.sign * f.qty) AS qty_sold,
                   SUM(f.sign * (f.buyer_value_paisa + f.buyer_comm_paisa + f.laga_paisa)) AS buyer_total,
                   SUM(f.sign) AS signs
            FROM analytics_facts f{on_postings}
            GROUP BY f.bikri_id, f.entry_date)
         SELECT id, number, entry_date, created_at, truck_no, status, own_date, qty_received,
                qty_sold, gross, comm, charges, buyer_total
         FROM (
            SELECT e.id AS id, e.number AS number, p.day AS entry_date, e.created_at AS created_at,
                   e.truck_no AS truck_no, e.status AS status, e.entry_date AS own_date,
                   CASE WHEN p.day = e.entry_date
                        THEN (SELECT COALESCE(SUM(l.qty_received), 0) FROM bikri_lots l WHERE l.bikri_id = e.id AND l.removed_at IS NULL)
                        ELSE 0 END AS qty_received,
                   {qty_sold} AS qty_sold, {gross} AS gross, {comm} AS comm,
                   CASE WHEN p.day = e.entry_date
                        THEN (SELECT COALESCE(SUM(l.deductions_paisa - l.seller_comm_paisa), 0)
                              FROM bikri_lots l WHERE l.bikri_id = e.id AND l.removed_at IS NULL)
                        ELSE 0 END AS charges,
                   {buyer_total} AS buyer_total
            FROM p JOIN bikri_entries e ON e.id = p.bikri_id
            WHERE (e.status = 'cancelled' OR p.signs > 0){and})",
        qty_sold = figure("qty", "qty_sold"),
        gross = figure("seller_amount_paisa", "gross"),
        comm = figure("seller_comm_paisa", "comm"),
        buyer_total = figure("buyer_total_paisa", "buyer_total"),
    );
    (sql, binds)
}

/// The page query of [`list`] and the values it binds, so a test can read its plan (Code Standards
/// B18). Newest day first, then the number.
pub fn list_query(filter: &ListFilter) -> (String, Vec<Value>) {
    let (inner, mut binds) = inner_query(filter);
    binds.push(Value::Integer(filter.limit));
    binds.push(Value::Integer(filter.offset));
    (
        format!(
            "{inner} ORDER BY {order} LIMIT ? OFFSET ?",
            order = filter.order_by.as_deref().map_or_else(
                || "entry_date DESC, number DESC".to_owned(),
                |o| format!("{o}, entry_date DESC, number DESC"),
            )
        ),
        binds,
    )
}

/// One page of the list: a row for each بکری and day of the filter, newest day first.
///
/// # Errors
/// Any SQLite error.
pub fn list(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<Vec<PostedRow>> {
    let (sql, binds) = list_query(filter);
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(binds.iter()), |row| {
        let mut c = Cols::new(row);
        let id = c.get()?;
        let number = c.get()?;
        let entry_date = c.get()?;
        let created_at = c.get()?;
        let truck_no = c.get()?;
        let status = c.status()?;
        let own_date: String = c.get()?;
        let qty_received = c.get()?;
        let qty_sold = c.get()?;
        let gross_paisa: i64 = c.get()?;
        let comm: i64 = c.get()?;
        let charges: i64 = c.get()?;
        let buyer_total = c.paisa()?;
        let whole = |value: i64| {
            mandi_core::money::Paisa::try_from_paisa(value).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Integer,
                    "bikri: an amount that is not a whole rupee".into(),
                )
            })
        };
        let gross = whole(gross_paisa)?;
        let net = whole(net_of(gross_paisa, comm, charges)?)?;
        Ok(PostedRow {
            row: BikriListRow {
                id,
                number,
                entry_date,
                created_at,
                truck_no,
                sellers: Vec::new(),
                items: Vec::new(),
                lots: Vec::new(),
                qty_received,
                qty_sold,
                gross,
                net,
                buyer_total,
                status,
                // Set by the service, which knows the closed-up-to day and the audit stamp.
                flags: EntryFlags::unmarked(),
            },
            own_date,
        })
    })?;
    let mut out = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    // The sellers, items and lots are of the whole بکری, the same for each of its days.
    let mut plain: Vec<BikriListRow> = out.iter().map(|r| r.row.clone()).collect();
    fill_sellers_and_items_of_rows(conn, &mut plain)?;
    for (posted, filled) in out.iter_mut().zip(plain) {
        posted.row = filled;
    }
    Ok(out)
}

/// Adds the sellers, items and lots to rows that may repeat a بکری (once for each of its days).
fn fill_sellers_and_items_of_rows(
    conn: &Connection,
    rows: &mut [BikriListRow],
) -> rusqlite::Result<()> {
    // The shared reader gives each id one row; a بکری of two days is read for the first and the
    // answer copied to the other.
    let mut unique: Vec<BikriListRow> = Vec::new();
    for row in rows.iter() {
        if !unique.iter().any(|u| u.id == row.id) {
            unique.push(row.clone());
        }
    }
    fill_sellers_and_items(conn, &mut unique)?;
    for row in rows.iter_mut() {
        if let Some(done) = unique.iter().find(|u| u.id == row.id) {
            row.sellers = done.sellers.clone();
            row.items = done.items.clone();
            row.lots = done.lots.clone();
        }
    }
    Ok(())
}

/// The sums a query over the rows gives, written once: only the saved rows count in a sum. The
/// صافی is not summed in SQL: its three parts are, and Rust subtracts them.
const SUMS: &str = "COALESCE(SUM(CASE WHEN status = 'saved' THEN qty_received END), 0),
        COALESCE(SUM(CASE WHEN status = 'saved' THEN qty_sold END), 0),
        COALESCE(SUM(CASE WHEN status = 'saved' THEN gross END), 0),
        COALESCE(SUM(CASE WHEN status = 'saved' THEN comm END), 0),
        COALESCE(SUM(CASE WHEN status = 'saved' THEN charges END), 0),
        COALESCE(SUM(CASE WHEN status = 'saved' THEN buyer_total END), 0)";

/// Gross less commission less charges, in whole paisa, checked.
fn net_of(gross: i64, comm: i64, charges: i64) -> rusqlite::Result<i64> {
    gross
        .checked_sub(comm)
        .and_then(|rest| rest.checked_sub(charges))
        .ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Integer,
                "bikri: a صافی that does not fit".into(),
            )
        })
}

/// The query of [`totals`] and the values it binds, for a test that reads its plan.
pub fn totals_query(filter: &ListFilter) -> (String, Vec<Value>) {
    let (inner, binds) = inner_query(filter);
    (format!("SELECT COUNT(*), {SUMS} FROM ({inner})"), binds)
}

/// The count and the sums over every row a filter matches, not only one page (PRD FR-19.11, B31).
/// The count has the cancelled rows the filter shows; the sums leave them out.
///
/// # Errors
/// Any SQLite error.
pub fn totals(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<ListTotals> {
    let (sql, binds) = totals_query(filter);
    conn.query_row(&sql, params_from_iter(binds.iter()), |r| {
        Ok(ListTotals {
            count: r.get(0)?,
            qty_received: r.get(1)?,
            qty_sold: r.get(2)?,
            gross_paisa: r.get(3)?,
            net_paisa: net_of(r.get(3)?, r.get(4)?, r.get(5)?)?,
            buyer_total_paisa: r.get(6)?,
        })
    })
}

/// The query of [`months`] and the values it binds.
pub fn months_query(filter: &ListFilter) -> (String, Vec<Value>) {
    let (inner, binds) = inner_query(filter);
    (
        format!(
            "SELECT substr(entry_date, 1, 7), COUNT(*), {SUMS} FROM ({inner}) GROUP BY 1 ORDER BY 1"
        ),
        binds,
    )
}

/// The same sums as [`totals`], one row for each month of the filter, oldest month first.
///
/// # Errors
/// Any SQLite error.
pub fn months(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<Vec<MonthRec>> {
    let (sql, binds) = months_query(filter);
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(binds.iter()), |r| {
        Ok(MonthRec {
            month: r.get(0)?,
            count: r.get(1)?,
            qty_received: r.get(2)?,
            qty_sold: r.get(3)?,
            gross_paisa: r.get(4)?,
            net_paisa: net_of(r.get(4)?, r.get(5)?, r.get(6)?)?,
            buyer_total_paisa: r.get(7)?,
        })
    })?;
    rows.collect()
}

/// The query of [`days`] and the values it binds.
pub fn days_query(filter: &ListFilter) -> (String, Vec<Value>) {
    let (inner, binds) = inner_query(filter);
    (
        format!(
            "SELECT entry_date, COUNT(*),
                    COALESCE(SUM(CASE WHEN status = 'saved' THEN gross END), 0)
             FROM ({inner}) GROUP BY entry_date ORDER BY entry_date"
        ),
        binds,
    )
}

/// The days of a filter that have at least one بکری posted on them, oldest first, with their count
/// and خام بکری. A day with none is not in the answer; the service adds it.
///
/// # Errors
/// Any SQLite error.
pub fn days(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<Vec<DayRec>> {
    let (sql, binds) = days_query(filter);
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(binds.iter()), |r| {
        Ok(DayRec {
            date: r.get(0)?,
            count: r.get(1)?,
            gross_paisa: r.get(2)?,
        })
    })?;
    rows.collect()
}
