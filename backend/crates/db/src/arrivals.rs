//! Goods that arrived from a garden (مال آیا, PRD FR-12.40, plan 33 v1.1): the rows of
//! `garden_arrivals` and the figures read about them. SQL only: every value is checked before it
//! comes here, and the CHECKs and triggers of `0009_date_to_sell_and_garden_arrivals.sql` are the
//! last guard (Code Standards 4.1 layer 4).
//!
//! An arrival is never deleted (Code Standards D3). A cancel sets its status; the running figures
//! `qty_sold` and `qty_written_off` move together with the sale or write-off that changes them. An
//! arrival has no money: no cost, no journal entry.

use std::collections::HashMap;

use mandi_core::accounts::RecordRef;
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Row, params, params_from_iter};

use crate::listing::{MonthSums, like_contains, like_prefix, month_sums};

/// The columns of a new arrival.
#[derive(Debug, Clone)]
pub struct NewArrival<'a> {
    /// The garden.
    pub venture_id: i64,
    /// The day it arrived, `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// The day to sell, `YYYY-MM-DD`. Never before `entry_date`.
    pub sell_date: &'a str,
    /// The item.
    pub item_id: i64,
    /// The packing.
    pub packing_id: i64,
    /// نگ arrived.
    pub qty: i64,
    /// Weight arrived, grams.
    pub weight_grams: Option<i64>,
    /// A note.
    pub note: Option<&'a str>,
    /// نگ for another shop (PRD FR-12.41); the rest are for our shop. At most `qty`.
    pub other_qty: i64,
    /// The other shop, as typed.
    pub shop_name: Option<&'a str>,
}

/// The next arrival number: the biggest ever given plus one (a cancelled arrival keeps its number).
///
/// # Errors
/// Any SQLite error.
pub fn next_number(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM garden_arrivals",
        [],
        |r| r.get(0),
    )
}

/// Adds an arrival and returns its id.
///
/// # Errors
/// Any SQLite error (the CHECKs of `garden_arrivals`).
pub fn insert(
    conn: &Connection,
    arrival: &NewArrival<'_>,
    number: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO garden_arrivals (created_at, created_by, updated_at, updated_by, number, venture_id,
             entry_date, sell_date, item_id, packing_id, qty, weight_grams, note, other_qty, shop_name)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            now,
            user_id,
            number,
            arrival.venture_id,
            arrival.entry_date,
            arrival.sell_date,
            arrival.item_id,
            arrival.packing_id,
            arrival.qty,
            arrival.weight_grams,
            arrival.note,
            arrival.other_qty,
            arrival.shop_name,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Moves what the garden lots took from an arrival: `qty` more (or fewer, if negative) نگ sold. The
/// table checks that sold plus written off is never more than arrived.
///
/// # Errors
/// Any SQLite error.
pub fn add_sold(
    conn: &Connection,
    id: i64,
    qty: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE garden_arrivals SET qty_sold = qty_sold + ?2, updated_at = ?3, updated_by = ?4,
             version = version + 1
         WHERE id = ?1",
        params![id, qty, now, user_id],
    )?;
    Ok(())
}

/// Moves a write-off: `qty` more (or fewer, if negative) نگ written off.
///
/// # Errors
/// Any SQLite error.
pub fn add_written_off(
    conn: &Connection,
    id: i64,
    qty: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE garden_arrivals SET qty_written_off = qty_written_off + ?2, updated_at = ?3,
             updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![id, qty, now, user_id],
    )?;
    Ok(())
}

/// Moves what other shops took from an arrival: `qty` more (or fewer, if negative) نگ sold in other
/// shops, in ONE update with `qty_sold`, so the trigger `garden_arrivals_split_fits_*` sees a whole
/// row. `moved` is how many were moved from our shop's part to the part for another shop (plan 35
/// D1; negative when a cancel moves them back).
///
/// # Errors
/// Any SQLite error.
pub fn add_sold_in_other_shop(
    conn: &Connection,
    id: i64,
    qty: i64,
    moved: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE garden_arrivals SET qty_sold = qty_sold + ?2, qty_sold_other = qty_sold_other + ?2,
             other_qty = other_qty + ?3, updated_at = ?4, updated_by = ?5, version = version + 1
         WHERE id = ?1",
        params![id, qty, moved, now, user_id],
    )?;
    Ok(())
}

/// Changes the split of an arrival (PRD FR-12.41): the نگ for another shop and its name.
///
/// # Errors
/// Any SQLite error (the trigger `garden_arrivals_split_fits_update`).
pub fn set_split(
    conn: &Connection,
    id: i64,
    other_qty: i64,
    shop_name: Option<&str>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE garden_arrivals SET other_qty = ?2, shop_name = ?3, updated_at = ?4, updated_by = ?5,
             version = version + 1
         WHERE id = ?1",
        params![id, other_qty, shop_name, now, user_id],
    )?;
    Ok(())
}

/// Changes the day to sell. The table refuses a day before the arrival day.
///
/// # Errors
/// Any SQLite error.
pub fn set_sell_date(
    conn: &Connection,
    id: i64,
    sell_date: &str,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE garden_arrivals SET sell_date = ?2, updated_at = ?3, updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![id, sell_date, now, user_id],
    )?;
    Ok(())
}

/// Marks an arrival cancelled, with the reason.
///
/// # Errors
/// Any SQLite error.
pub fn mark_cancelled(
    conn: &Connection,
    id: i64,
    reason: &str,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE garden_arrivals SET status = 'cancelled', cancel_reason = ?2, updated_at = ?3,
             updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![id, reason, now, user_id],
    )?;
    Ok(())
}

/// An arrival as stored, with the garden and the names of what it points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrivalRec {
    /// Row id.
    pub id: i64,
    /// Version.
    pub version: i64,
    /// Its number.
    pub number: i64,
    /// Its garden: id.
    pub garden_id: i64,
    /// Its garden: number.
    pub garden_number: i64,
    /// Its garden: Urdu name.
    pub garden_name_ur: String,
    /// Its garden: English name.
    pub garden_name_en: String,
    /// Its garden: `open` or `closed`.
    pub garden_status: String,
    /// The garden's account (the seller of a linked lot).
    pub account: RecordRef,
    /// The day it arrived, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The day to sell, `YYYY-MM-DD`.
    pub sell_date: String,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// نگ arrived.
    pub qty: i64,
    /// Weight arrived, grams.
    pub weight_grams: Option<i64>,
    /// A note.
    pub note: Option<String>,
    /// `saved` or `cancelled`.
    pub status: String,
    /// The reason of a cancel.
    pub cancel_reason: Option<String>,
    /// نگ sold.
    pub qty_sold: i64,
    /// نگ written off.
    pub qty_written_off: i64,
    /// نگ for another shop (PRD FR-12.41).
    pub other_qty: i64,
    /// نگ sold in other shops (part of `qty_sold`).
    pub qty_sold_other: i64,
    /// The other shop, as typed.
    pub shop_name: Option<String>,
}

const REC_COLUMNS: &str = "a.id, a.version, a.number, a.venture_id, v.number, v.name_ur, v.name_en,
    v.status, ac.id, ac.code, ac.name_ur, ac.name_en, ac.tag,
    a.entry_date, a.sell_date, it.id, it.code, it.name_ur, it.name_en, pk.id, pk.code, pk.name_ur,
    pk.name_en, a.qty, a.weight_grams, a.note, a.status, a.cancel_reason, a.qty_sold,
    a.qty_written_off, a.other_qty, a.qty_sold_other, a.shop_name";

const REC_FROM: &str = "FROM garden_arrivals a
    JOIN ventures v ON v.id = a.venture_id
    JOIN accounts ac ON ac.id = v.account_id
    JOIN items it ON it.id = a.item_id
    JOIN packings pk ON pk.id = a.packing_id";

fn read_rec(r: &Row<'_>) -> rusqlite::Result<ArrivalRec> {
    Ok(ArrivalRec {
        id: r.get(0)?,
        version: r.get(1)?,
        number: r.get(2)?,
        garden_id: r.get(3)?,
        garden_number: r.get(4)?,
        garden_name_ur: r.get(5)?,
        garden_name_en: r.get(6)?,
        garden_status: r.get(7)?,
        account: RecordRef {
            id: r.get(8)?,
            code: r.get(9)?,
            name_ur: r.get(10)?,
            name_en: r.get(11)?,
            tag: r.get(12)?,
        },
        entry_date: r.get(13)?,
        sell_date: r.get(14)?,
        item: RecordRef {
            id: r.get(15)?,
            code: r.get(16)?,
            name_ur: r.get(17)?,
            name_en: r.get(18)?,
            tag: None,
        },
        packing: RecordRef {
            id: r.get(19)?,
            code: r.get(20)?,
            name_ur: r.get(21)?,
            name_en: r.get(22)?,
            tag: None,
        },
        qty: r.get(23)?,
        weight_grams: r.get(24)?,
        note: r.get(25)?,
        status: r.get(26)?,
        cancel_reason: r.get(27)?,
        qty_sold: r.get(28)?,
        qty_written_off: r.get(29)?,
        other_qty: r.get(30)?,
        qty_sold_other: r.get(31)?,
        shop_name: r.get(32)?,
    })
}

/// One arrival by id.
///
/// # Errors
/// Any SQLite error.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<ArrivalRec>> {
    conn.query_row(
        &format!("SELECT {REC_COLUMNS} {REC_FROM} WHERE a.id = ?1"),
        [id],
        read_rec,
    )
    .optional()
}

/// Several arrivals by id, in one query (Code Standards B22).
///
/// # Errors
/// Any SQLite error.
pub fn get_many(conn: &Connection, ids: &[i64]) -> rusqlite::Result<Vec<ArrivalRec>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let marks = vec!["?"; ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT {REC_COLUMNS} {REC_FROM} WHERE a.id IN ({marks}) ORDER BY a.id"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(ids), read_rec)?;
    rows.collect()
}

/// The state filter of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateFilter {
    /// Saved, with goods left.
    InStock,
    /// Saved, nothing left.
    SoldOut,
    /// Cancelled.
    Cancelled,
}

/// What the list asks for. A field that is `None` is not a filter.
#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    /// Only this garden.
    pub garden_id: Option<i64>,
    /// First day, `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`.
    pub to: Option<String>,
    /// Only this item.
    pub item_id: Option<i64>,
    /// Only this state.
    pub state: Option<StateFilter>,
    /// Only arrivals whose number, note, other shop, item (code or name) or garden name holds this
    /// text.
    pub search: Option<String>,
    /// The `ORDER BY` expressions chosen by a sort, from [`SORT_COLUMNS`]. `None` is oldest day
    /// first.
    pub order_by: Option<String>,
    /// True to leave the cancelled arrivals out when no state is chosen (plan 38 R5).
    pub hide_cancelled: bool,
}

/// The columns a list can be sorted by: the name of the field of the row and its SQL expression
/// (the list query joins the garden, the item and the packing as `v`, `it` and `pk`).
pub const SORT_COLUMNS: [(&str, &str); 11] = [
    ("number", "a.number"),
    ("entry_date", "a.entry_date"),
    ("sell_date", "a.sell_date"),
    ("status", "a.status"),
    ("garden", "v.number"),
    ("item", "it.code"),
    ("qty", "a.qty"),
    ("qty_sold", "a.qty_sold"),
    ("qty_written_off", "a.qty_written_off"),
    ("qty_left", "(a.qty - a.qty_sold - a.qty_written_off)"),
    ("shop_name", "a.shop_name"),
];

/// The `WHERE` of a filter and the values it binds, in order. Written with `?` only, and with
/// subqueries instead of joins for the search, so the same text serves the page query, the totals
/// and the month sums. An empty filter gives an empty text.
fn filter_where(filter: &ListFilter) -> (String, Vec<Value>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<Value> = Vec::new();
    if let Some(garden) = filter.garden_id {
        conditions.push("a.venture_id = ?".to_owned());
        binds.push(Value::Integer(garden));
    }
    if let Some(from) = &filter.from {
        conditions.push("a.entry_date >= ?".to_owned());
        binds.push(Value::Text(from.clone()));
    }
    if let Some(to) = &filter.to {
        conditions.push("a.entry_date <= ?".to_owned());
        binds.push(Value::Text(to.clone()));
    }
    if let Some(item) = filter.item_id {
        conditions.push("a.item_id = ?".to_owned());
        binds.push(Value::Integer(item));
    }
    match filter.state {
        None if filter.hide_cancelled => conditions.push("a.status = 'saved'".to_owned()),
        None => {}
        Some(StateFilter::InStock) => conditions
            .push("a.status = 'saved' AND a.qty - a.qty_sold - a.qty_written_off > 0".to_owned()),
        Some(StateFilter::SoldOut) => conditions
            .push("a.status = 'saved' AND a.qty - a.qty_sold - a.qty_written_off = 0".to_owned()),
        Some(StateFilter::Cancelled) => conditions.push("a.status = 'cancelled'".to_owned()),
    }
    if let Some(text) = filter
        .search
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        let mut parts = String::new();
        if let Ok(number) = text.parse::<i64>() {
            parts.push_str("a.number = ? OR ");
            binds.push(Value::Integer(number));
        }
        parts.push_str(
            "a.note LIKE ? ESCAPE '\\' OR a.shop_name LIKE ? ESCAPE '\\'
             OR a.item_id IN (SELECT id FROM items WHERE code LIKE ? ESCAPE '\\'
                 OR name_ur LIKE ? ESCAPE '\\' OR name_en LIKE ? ESCAPE '\\')
             OR a.venture_id IN (SELECT id FROM ventures
                 WHERE name_ur LIKE ? ESCAPE '\\' OR name_en LIKE ? ESCAPE '\\')",
        );
        conditions.push(format!("({parts})"));
        let contains = like_contains(text);
        for _ in 0..2 {
            binds.push(Value::Text(contains.clone()));
        }
        binds.push(Value::Text(like_prefix(text)));
        for _ in 0..4 {
            binds.push(Value::Text(contains.clone()));
        }
    }
    if conditions.is_empty() {
        (String::new(), binds)
    } else {
        (format!(" WHERE {}", conditions.join(" AND ")), binds)
    }
}

/// One page of the arrivals that match, oldest arrival day first, then by number (Code Standards
/// B31).
///
/// # Errors
/// Any SQLite error.
pub fn list(
    conn: &Connection,
    filter: &ListFilter,
    offset: i64,
    limit: i64,
) -> rusqlite::Result<Vec<ArrivalRec>> {
    let (condition, mut binds) = filter_where(filter);
    binds.push(Value::Integer(limit));
    binds.push(Value::Integer(offset));
    // Oldest day first, then by number. A sort puts its column first and keeps that order after
    // it, so ties come out the same on every page. A sort by the day keeps the numbers in the
    // same direction.
    let order = match filter.order_by.as_deref() {
        None => "a.entry_date, a.number".to_owned(),
        Some(o) if o.starts_with("a.entry_date") => {
            let direction = if o.ends_with("DESC") { "DESC" } else { "ASC" };
            format!("{o}, a.number {direction}, a.id {direction}")
        }
        Some(o) => format!("{o}, a.entry_date, a.number, a.id"),
    };
    let mut statement = conn.prepare(&format!(
        "SELECT {REC_COLUMNS} {REC_FROM}{condition} ORDER BY {order} LIMIT ? OFFSET ?"
    ))?;
    let rows = statement.query_map(params_from_iter(binds.iter()), read_rec)?;
    rows.collect()
}

/// The count of each month of the filter (the arrival day) with the نگ that arrived, were sold,
/// written off and are left, of the saved ones, oldest month first, in one grouped query (plan 38
/// R4).
///
/// # Errors
/// Any SQLite error.
pub fn list_months(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<Vec<MonthSums>> {
    let (condition, binds) = filter_where(filter);
    month_sums(
        conn,
        "garden_arrivals a",
        &condition,
        "a.entry_date",
        &[
            "CASE WHEN a.status = 'saved' THEN a.qty ELSE 0 END",
            "CASE WHEN a.status = 'saved' THEN a.qty_sold ELSE 0 END",
            "CASE WHEN a.status = 'saved' THEN a.qty_written_off ELSE 0 END",
            "CASE WHEN a.status = 'saved' THEN a.qty - a.qty_sold - a.qty_written_off ELSE 0 END",
        ],
        &binds,
    )
}

/// What a filter adds up to, over every arrival that matches and not only one page (B31).
/// Cancelled arrivals count in `count` and in nothing else.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Totals {
    /// How many arrivals match.
    pub count: i64,
    /// Σ نگ arrived, of the saved ones.
    pub qty: i64,
    /// Σ نگ sold, of the saved ones.
    pub qty_sold: i64,
    /// Σ نگ written off, of the saved ones.
    pub qty_written_off: i64,
    /// Σ commission on the goods sold from the saved ones in our shop (plan 36).
    pub commission: crate::commission::Commission,
}

/// The totals of a filter, in two indexed queries.
///
/// # Errors
/// Any SQLite error.
pub fn totals(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<Totals> {
    let (condition, binds) = filter_where(filter);
    let args = || params_from_iter(binds.iter());
    let mut out = conn.query_row(
        &format!(
            "SELECT COUNT(*),
                    COALESCE(SUM(CASE WHEN a.status = 'saved' THEN a.qty END), 0),
                    COALESCE(SUM(CASE WHEN a.status = 'saved' THEN a.qty_sold END), 0),
                    COALESCE(SUM(CASE WHEN a.status = 'saved' THEN a.qty_written_off END), 0)
             FROM garden_arrivals a{condition}"
        ),
        args(),
        |r| {
            Ok(Totals {
                count: r.get(0)?,
                qty: r.get(1)?,
                qty_sold: r.get(2)?,
                qty_written_off: r.get(3)?,
                commission: crate::commission::Commission::default(),
            })
        },
    )?;
    out.commission = crate::commission::of_arrivals_where(conn, &condition, args())?;
    Ok(out)
}

/// The saved arrivals of open gardens with goods left and a day to sell on or before `date`, oldest
/// day to sell first, then by number (PRD FR-12.39, FR-12.40). At most `limit` rows.
///
/// # Errors
/// Any SQLite error.
pub fn ready_by(conn: &Connection, date: &str, limit: i64) -> rusqlite::Result<Vec<ArrivalRec>> {
    let mut statement = conn.prepare(&format!(
        "SELECT {REC_COLUMNS} {REC_FROM}
         WHERE a.status = 'saved' AND v.status = 'open'
           AND a.qty - a.other_qty - (a.qty_sold - a.qty_sold_other) - a.qty_written_off > 0
           AND a.sell_date <= ?1
         ORDER BY a.sell_date, a.number LIMIT ?2"
    ))?;
    let rows = statement.query_map(params![date, limit], read_rec)?;
    rows.collect()
}

/// The saved arrivals with goods left, for the باقی مال list: oldest arrival day first, then number.
/// The filters are the day arrived, the item and the garden's account (the party filter).
///
/// # Errors
/// Any SQLite error.
pub fn with_stock(
    conn: &Connection,
    from: Option<&str>,
    to: Option<&str>,
    item_id: Option<i64>,
    account_id: Option<i64>,
) -> rusqlite::Result<Vec<ArrivalRec>> {
    let mut statement = conn.prepare(&format!(
        "SELECT {REC_COLUMNS} {REC_FROM}
         WHERE a.status = 'saved' AND a.qty - a.qty_sold - a.qty_written_off > 0
           AND (?1 IS NULL OR a.entry_date >= ?1) AND (?2 IS NULL OR a.entry_date <= ?2)
           AND (?3 IS NULL OR a.item_id = ?3) AND (?4 IS NULL OR v.account_id = ?4)
         ORDER BY a.entry_date, a.number"
    ))?;
    let rows = statement.query_map(params![from, to, item_id, account_id], read_rec)?;
    rows.collect()
}

/// How many saved arrivals a garden has (a garden with some names the arrival of every sale in
/// another shop, PRD FR-12.41).
///
/// # Errors
/// Any SQLite error.
pub fn count_saved_of_garden(conn: &Connection, garden_id: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM garden_arrivals WHERE venture_id = ?1 AND status = 'saved'",
        [garden_id],
        |r| r.get(0),
    )
}

/// Σ نگ left in the saved arrivals of one garden (a garden with some left cannot be closed, PRD
/// FR-12.40).
///
/// # Errors
/// Any SQLite error.
pub fn left_of_garden(conn: &Connection, garden_id: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(SUM(qty - qty_sold - qty_written_off), 0) FROM garden_arrivals
         WHERE venture_id = ?1 AND status = 'saved'",
        [garden_id],
        |r| r.get(0),
    )
}

/// One garden lot that sold from an arrival.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotOfArrival {
    /// The بکری.
    pub bikri_id: i64,
    /// Its number.
    pub bikri_number: i64,
    /// The lot.
    pub lot_id: i64,
    /// The lot number inside its بکری.
    pub lot_no: i64,
    /// The day of the بکری, `YYYY-MM-DD`.
    pub entry_date: String,
    /// نگ sold on the lines in force.
    pub qty_sold: i64,
}

/// The lots of saved بکری that take from an arrival, oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn lots_of(conn: &Connection, arrival_id: i64) -> rusqlite::Result<Vec<LotOfArrival>> {
    let mut statement = conn.prepare(
        "SELECT e.id, e.number, l.id, l.lot_no, e.entry_date,
                COALESCE((SELECT SUM(s.qty) FROM bikri_sales s WHERE s.lot_id = l.id AND s.is_active = 1), 0)
         FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id
         WHERE l.arrival_id = ?1 AND e.status = 'saved' ORDER BY e.entry_date, e.number, l.lot_no",
    )?;
    let rows = statement.query_map([arrival_id], |r| {
        Ok(LotOfArrival {
            bikri_id: r.get(0)?,
            bikri_number: r.get(1)?,
            lot_id: r.get(2)?,
            lot_no: r.get(3)?,
            entry_date: r.get(4)?,
            qty_sold: r.get(5)?,
        })
    })?;
    rows.collect()
}

/// A write-off of an arrival that has not been undone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrivalWriteOff {
    /// The stock move.
    pub move_id: i64,
    /// The arrival.
    pub arrival_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub move_date: String,
    /// How many نگ (more than 0).
    pub qty: i64,
    /// The reason.
    pub reason: Option<String>,
}

fn read_write_off(r: &Row<'_>) -> rusqlite::Result<ArrivalWriteOff> {
    let qty: i64 = r.get(3)?;
    Ok(ArrivalWriteOff {
        move_id: r.get(0)?,
        arrival_id: r.get(1)?,
        move_date: r.get(2)?,
        qty: qty.saturating_neg(),
        reason: r.get(4)?,
    })
}

/// A write-off of an arrival by its move id, if it is one and has not been undone.
///
/// # Errors
/// Any SQLite error.
pub fn write_off(conn: &Connection, move_id: i64) -> rusqlite::Result<Option<ArrivalWriteOff>> {
    conn.query_row(
        "SELECT m.id, m.arrival_id, m.move_date, m.qty, m.reason FROM stock_moves m
         WHERE m.id = ?1 AND m.kind = 'written_off' AND m.lot_id IS NULL AND m.arrival_id IS NOT NULL
           AND NOT EXISTS (SELECT 1 FROM stock_moves r WHERE r.reverses_move_id = m.id)",
        [move_id],
        read_write_off,
    )
    .optional()
}

/// The write-offs of these arrivals that have not been undone, by arrival, oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn write_offs_of(
    conn: &Connection,
    arrival_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, Vec<ArrivalWriteOff>>> {
    let mut out: HashMap<i64, Vec<ArrivalWriteOff>> = HashMap::new();
    if arrival_ids.is_empty() {
        return Ok(out);
    }
    let marks = vec!["?"; arrival_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT m.id, m.arrival_id, m.move_date, m.qty, m.reason FROM stock_moves m
         WHERE m.arrival_id IN ({marks}) AND m.kind = 'written_off' AND m.lot_id IS NULL
           AND NOT EXISTS (SELECT 1 FROM stock_moves r WHERE r.reverses_move_id = m.id)
         ORDER BY m.move_date, m.id"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(arrival_ids), read_write_off)?;
    for row in rows {
        let row = row?;
        out.entry(row.arrival_id).or_default().push(row);
    }
    Ok(out)
}

/// The `received` stock move an arrival was saved with (the one a cancel reverses).
///
/// # Errors
/// Any SQLite error.
pub fn received_move(conn: &Connection, arrival_id: i64) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT id FROM stock_moves
         WHERE arrival_id = ?1 AND kind = 'received' AND lot_id IS NULL
           AND NOT EXISTS (SELECT 1 FROM stock_moves r WHERE r.reverses_move_id = stock_moves.id)
         ORDER BY id LIMIT 1",
        [arrival_id],
        |r| r.get(0),
    )
    .optional()
}
