//! The goods bought on market trips (PRD 12.10, plan 16b): the rows of `purchases` and the figures
//! read about them. SQL only: every number is worked out and checked before it comes here, and the
//! CHECKs and triggers of `0006_trips_gardens_fruit_khata.sql` are the last guard (Code Standards 4.1
//! layer 4).
//!
//! A purchase is never deleted (Code Standards D3). A cancel sets its status; the running figures
//! `qty_sold`, `qty_written_off` and `cost_released_paisa` move together with the sale or write-off
//! that changes them, and `trip_cost_paisa` moves with a trip cost (PRD C24). Sums of money are
//! `SUM()` on `INTEGER` columns only (money agent Section 9); the caller adds them with checked maths.

use std::collections::HashMap;

use mandi_core::accounts::RecordRef;
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Row, params, params_from_iter};

use crate::listing::{MonthSums, like_contains, like_prefix, month_sums};

/// The columns of a new purchase. The money is final: the table checks that the paid-now amount is
/// the value plus the other costs and that the value is `qty x rate` for an item sold by نگ. The total
/// cost starts as the paid-now amount: no trip cost is spread over a purchase that is just bought
/// (PRD FR-12.31, A64).
#[derive(Debug, Clone)]
pub struct NewPurchase<'a> {
    /// The trip.
    pub venture_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// The day to sell, `YYYY-MM-DD` (PRD FR-12.39). Never before `entry_date`.
    pub sell_date: &'a str,
    /// Who it was bought from, as typed.
    pub supplier_text: Option<&'a str>,
    /// The item.
    pub item_id: i64,
    /// The packing.
    pub packing_id: i64,
    /// نگ bought.
    pub qty: i64,
    /// Weight bought, grams.
    pub weight_grams: Option<i64>,
    /// The rate, paisa.
    pub rate_paisa: i64,
    /// C1, paisa.
    pub value_paisa: i64,
    /// Freight, paisa.
    pub freight_paisa: i64,
    /// Labour, paisa.
    pub labour_paisa: i64,
    /// Other cost, paisa.
    pub other_cost_paisa: i64,
    /// Value plus the three costs, paisa.
    pub paid_now_paisa: i64,
    /// A note.
    pub note: Option<&'a str>,
    /// نگ for another shop (PRD FR-12.41); the rest are for our shop. At most `qty`.
    pub other_qty: i64,
    /// The other shop, as typed.
    pub shop_name: Option<&'a str>,
}

/// The next purchase number: the biggest ever given plus one (a cancelled purchase keeps its
/// number, so a number is never used twice).
///
/// # Errors
/// Any SQLite error.
pub fn next_number(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM purchases",
        [],
        |r| r.get(0),
    )
}

/// Adds a purchase and returns its id. Its journal entry is set right after it is posted.
///
/// # Errors
/// Any SQLite error (the CHECKs of `purchases`).
pub fn insert(
    conn: &Connection,
    purchase: &NewPurchase<'_>,
    number: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO purchases (created_at, created_by, updated_at, updated_by, number, venture_id,
             entry_date, supplier_text, item_id, packing_id, qty, weight_grams, rate_paisa, value_paisa,
             freight_paisa, labour_paisa, other_cost_paisa, paid_now_paisa, trip_cost_paisa,
             total_cost_paisa, note, sell_date, other_qty, shop_name)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 0, ?16, ?17, ?18, ?19, ?20)",
        params![
            now,
            user_id,
            number,
            purchase.venture_id,
            purchase.entry_date,
            purchase.supplier_text,
            purchase.item_id,
            purchase.packing_id,
            purchase.qty,
            purchase.weight_grams,
            purchase.rate_paisa,
            purchase.value_paisa,
            purchase.freight_paisa,
            purchase.labour_paisa,
            purchase.other_cost_paisa,
            purchase.paid_now_paisa,
            purchase.note,
            purchase.sell_date,
            purchase.other_qty,
            purchase.shop_name,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Changes the day to sell of a purchase (PRD FR-12.39). The table refuses a day before the
/// purchase day.
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
        "UPDATE purchases SET sell_date = ?2, updated_at = ?3, updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![id, sell_date, now, user_id],
    )?;
    Ok(())
}

/// Names the journal entry of a purchase (once, right after it is posted).
///
/// # Errors
/// Any SQLite error.
pub fn set_entry(
    conn: &Connection,
    id: i64,
    entry_id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE purchases SET journal_entry_id = ?2, updated_at = ?3, updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![id, entry_id, now, user_id],
    )?;
    Ok(())
}

/// A money or quantity column of a purchase plus `delta`, with a checked add. The money of a
/// purchase is added in Rust, not in SQL (the money agent's Section 9); the table's CHECKs are the
/// last line of defence. `column` is always a fixed name from this file.
fn plus(conn: &Connection, id: i64, column: &str, delta: i64) -> rusqlite::Result<i64> {
    let current: i64 = conn.query_row(
        &format!("SELECT {column} FROM purchases WHERE id = ?1"),
        [id],
        |r| r.get(0),
    )?;
    current
        .checked_add(delta)
        .ok_or(rusqlite::Error::IntegralValueOutOfRange(0, delta))
}

/// Moves what the own lots (and the sales in other shops) took from a purchase: `qty` more (or fewer,
/// if negative) نگ sold and `cost_paisa` more (or less) cost released (C21). The table checks that
/// sold plus written off is never more than bought and the cost released never more than the cost.
///
/// # Errors
/// Any SQLite error.
pub fn add_sold(
    conn: &Connection,
    id: i64,
    qty: i64,
    cost_paisa: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    let released = plus(conn, id, "cost_released_paisa", cost_paisa)?;
    conn.execute(
        "UPDATE purchases SET qty_sold = qty_sold + ?2, cost_released_paisa = ?3,
             updated_at = ?4, updated_by = ?5, version = version + 1
         WHERE id = ?1",
        params![id, qty, released, now, user_id],
    )?;
    Ok(())
}

/// Moves a write-off: `qty` more (or fewer, if negative) نگ written off and `cost_paisa` more (or
/// less) cost released.
///
/// # Errors
/// Any SQLite error.
pub fn add_written_off(
    conn: &Connection,
    id: i64,
    qty: i64,
    cost_paisa: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    let released = plus(conn, id, "cost_released_paisa", cost_paisa)?;
    conn.execute(
        "UPDATE purchases SET qty_written_off = qty_written_off + ?2,
             cost_released_paisa = ?3,
             updated_at = ?4, updated_by = ?5, version = version + 1
         WHERE id = ?1",
        params![id, qty, released, now, user_id],
    )?;
    Ok(())
}

/// Adds `cost_paisa` (or takes it off, if negative) of a trip cost to a purchase: its part of the
/// trip costs and its total cost move together (PRD C24). The table checks the sum.
///
/// # Errors
/// Any SQLite error.
pub fn add_trip_cost(
    conn: &Connection,
    id: i64,
    cost_paisa: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    let trip_cost = plus(conn, id, "trip_cost_paisa", cost_paisa)?;
    let total = plus(conn, id, "total_cost_paisa", cost_paisa)?;
    conn.execute(
        "UPDATE purchases SET trip_cost_paisa = ?2, total_cost_paisa = ?3,
             updated_at = ?4, updated_by = ?5, version = version + 1
         WHERE id = ?1",
        params![id, trip_cost, total, now, user_id],
    )?;
    Ok(())
}

/// Moves what other shops took from a purchase: `qty` more (or fewer, if negative) نگ sold in other
/// shops and `cost_paisa` more (or less) cost released (C21), in ONE update, so `qty_sold` and
/// `qty_sold_other` move together and the trigger `purchases_split_fits_*` sees a whole row.
/// `moved` is how many of those نگ were moved from our shop's part to the part for another shop
/// (plan 35 D1; negative when a cancel moves them back): `other_qty` moves with it.
///
/// # Errors
/// Any SQLite error.
pub fn add_sold_in_other_shop(
    conn: &Connection,
    id: i64,
    qty: i64,
    moved: i64,
    cost_paisa: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    let released = plus(conn, id, "cost_released_paisa", cost_paisa)?;
    conn.execute(
        "UPDATE purchases SET qty_sold = qty_sold + ?2, qty_sold_other = qty_sold_other + ?2,
             other_qty = other_qty + ?3, cost_released_paisa = ?4, updated_at = ?5, updated_by = ?6,
             version = version + 1
         WHERE id = ?1",
        params![id, qty, moved, released, now, user_id],
    )?;
    Ok(())
}

/// Changes the split of a purchase (PRD FR-12.41): the نگ for another shop and its name.
///
/// # Errors
/// Any SQLite error (the trigger `purchases_split_fits_update`).
pub fn set_split(
    conn: &Connection,
    id: i64,
    other_qty: i64,
    shop_name: Option<&str>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE purchases SET other_qty = ?2, shop_name = ?3, updated_at = ?4, updated_by = ?5,
             version = version + 1
         WHERE id = ?1",
        params![id, other_qty, shop_name, now, user_id],
    )?;
    Ok(())
}

/// Marks a purchase cancelled, with the reason.
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
        "UPDATE purchases SET status = 'cancelled', cancel_reason = ?2, updated_at = ?3, updated_by = ?4,
             version = version + 1
         WHERE id = ?1",
        params![id, reason, now, user_id],
    )?;
    Ok(())
}

/// A purchase as stored, with the names of what it points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurchaseRec {
    /// Row id.
    pub id: i64,
    /// Version.
    pub version: i64,
    /// Its number.
    pub number: i64,
    /// Its trip: id.
    pub trip_id: i64,
    /// Its trip: number.
    pub trip_number: i64,
    /// Its trip: Urdu name.
    pub trip_name_ur: String,
    /// Its trip: English name.
    pub trip_name_en: String,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The day to sell, `YYYY-MM-DD` (PRD FR-12.39). The entry date for a row from before it.
    pub sell_date: String,
    /// Who it was bought from, as typed.
    pub supplier_text: Option<String>,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// نگ bought.
    pub qty: i64,
    /// Weight bought, grams.
    pub weight_grams: Option<i64>,
    /// The rate, paisa.
    pub rate_paisa: i64,
    /// C1, paisa.
    pub value_paisa: i64,
    /// Freight, paisa.
    pub freight_paisa: i64,
    /// Labour, paisa.
    pub labour_paisa: i64,
    /// Other cost, paisa.
    pub other_cost_paisa: i64,
    /// Value plus the three costs, paisa.
    pub paid_now_paisa: i64,
    /// Its part of the trip costs, paisa.
    pub trip_cost_paisa: i64,
    /// Paid now plus the trip cost part, paisa.
    pub total_cost_paisa: i64,
    /// The journal entry of the purchase.
    pub entry_id: Option<i64>,
    /// `saved` or `cancelled`.
    pub status: String,
    /// The reason of a cancel.
    pub cancel_reason: Option<String>,
    /// A note.
    pub note: Option<String>,
    /// نگ sold.
    pub qty_sold: i64,
    /// نگ written off.
    pub qty_written_off: i64,
    /// The cost already released, paisa.
    pub cost_released_paisa: i64,
    /// نگ for another shop (PRD FR-12.41).
    pub other_qty: i64,
    /// نگ sold in other shops (part of `qty_sold`).
    pub qty_sold_other: i64,
    /// The other shop, as typed.
    pub shop_name: Option<String>,
}

const REC_COLUMNS: &str = "p.id, p.version, p.number, p.venture_id, v.number, v.name_ur, v.name_en,
    p.entry_date, p.supplier_text,
    it.id, it.code, it.name_ur, it.name_en, pk.id, pk.code, pk.name_ur, pk.name_en,
    p.qty, p.weight_grams, p.rate_paisa, p.value_paisa, p.freight_paisa, p.labour_paisa,
    p.other_cost_paisa, p.paid_now_paisa, p.trip_cost_paisa, p.total_cost_paisa, p.journal_entry_id,
    p.status, p.cancel_reason, p.note, p.qty_sold, p.qty_written_off, p.cost_released_paisa,
    COALESCE(p.sell_date, p.entry_date), p.other_qty, p.qty_sold_other, p.shop_name";

const REC_FROM: &str = "FROM purchases p
    JOIN ventures v ON v.id = p.venture_id
    JOIN items it ON it.id = p.item_id
    JOIN packings pk ON pk.id = p.packing_id";

fn read_rec(r: &Row<'_>) -> rusqlite::Result<PurchaseRec> {
    Ok(PurchaseRec {
        id: r.get(0)?,
        version: r.get(1)?,
        number: r.get(2)?,
        trip_id: r.get(3)?,
        trip_number: r.get(4)?,
        trip_name_ur: r.get(5)?,
        trip_name_en: r.get(6)?,
        entry_date: r.get(7)?,
        supplier_text: r.get(8)?,
        item: RecordRef {
            id: r.get(9)?,
            code: r.get(10)?,
            name_ur: r.get(11)?,
            name_en: r.get(12)?,
            tag: None,
        },
        packing: RecordRef {
            id: r.get(13)?,
            code: r.get(14)?,
            name_ur: r.get(15)?,
            name_en: r.get(16)?,
            tag: None,
        },
        qty: r.get(17)?,
        weight_grams: r.get(18)?,
        rate_paisa: r.get(19)?,
        value_paisa: r.get(20)?,
        freight_paisa: r.get(21)?,
        labour_paisa: r.get(22)?,
        other_cost_paisa: r.get(23)?,
        paid_now_paisa: r.get(24)?,
        trip_cost_paisa: r.get(25)?,
        total_cost_paisa: r.get(26)?,
        entry_id: r.get(27)?,
        status: r.get(28)?,
        cancel_reason: r.get(29)?,
        note: r.get(30)?,
        qty_sold: r.get(31)?,
        qty_written_off: r.get(32)?,
        cost_released_paisa: r.get(33)?,
        sell_date: r.get(34)?,
        other_qty: r.get(35)?,
        qty_sold_other: r.get(36)?,
        shop_name: r.get(37)?,
    })
}

/// One purchase by id.
///
/// # Errors
/// Any SQLite error.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<PurchaseRec>> {
    conn.query_row(
        &format!("SELECT {REC_COLUMNS} {REC_FROM} WHERE p.id = ?1"),
        [id],
        read_rec,
    )
    .optional()
}

/// Several purchases by id, in one query (Code Standards B22).
///
/// # Errors
/// Any SQLite error.
pub fn get_many(conn: &Connection, ids: &[i64]) -> rusqlite::Result<Vec<PurchaseRec>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let marks = vec!["?"; ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT {REC_COLUMNS} {REC_FROM} WHERE p.id IN ({marks}) ORDER BY p.id"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(ids), read_rec)?;
    rows.collect()
}

/// The saved purchases of a trip that still have goods left, oldest first (the order of the spread of
/// a trip cost, PRD C24: a tie goes to the earlier purchase).
///
/// # Errors
/// Any SQLite error.
pub fn with_stock_of_trip(conn: &Connection, trip_id: i64) -> rusqlite::Result<Vec<PurchaseRec>> {
    let mut statement = conn.prepare(&format!(
        "SELECT {REC_COLUMNS} {REC_FROM}
         WHERE p.venture_id = ?1 AND p.status = 'saved' AND p.qty - p.qty_sold - p.qty_written_off > 0
         ORDER BY p.id"
    ))?;
    let rows = statement.query_map([trip_id], read_rec)?;
    rows.collect()
}

/// All the saved purchases of a trip, oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn saved_of_trip(conn: &Connection, trip_id: i64) -> rusqlite::Result<Vec<PurchaseRec>> {
    let mut statement = conn.prepare(&format!(
        "SELECT {REC_COLUMNS} {REC_FROM} WHERE p.venture_id = ?1 AND p.status = 'saved' ORDER BY p.id"
    ))?;
    let rows = statement.query_map([trip_id], read_rec)?;
    rows.collect()
}

/// The state filter of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateFilter {
    /// Saved, with stock left.
    InStock,
    /// Saved, nothing left.
    SoldOut,
    /// Cancelled.
    Cancelled,
}

/// What the list asks for. A field that is `None` is not a filter.
#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    /// Only this trip.
    pub trip_id: Option<i64>,
    /// First day, `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`.
    pub to: Option<String>,
    /// Only this item.
    pub item_id: Option<i64>,
    /// Only this state.
    pub state: Option<StateFilter>,
    /// Only purchases whose number, supplier, other shop, item (code or name) or trip name holds
    /// this text.
    pub search: Option<String>,
    /// The `ORDER BY` expressions chosen by a sort, from [`SORT_COLUMNS`]. `None` is newest day
    /// first.
    pub order_by: Option<String>,
    /// True to leave the cancelled purchases out when no state is chosen (plan 38 R5).
    pub hide_cancelled: bool,
}

/// The columns a list can be sorted by: the name of the field of the row and its SQL expression
/// (the list query joins the item, the packing and the trip as `it`, `pk` and `v`).
pub const SORT_COLUMNS: [(&str, &str); 14] = [
    ("number", "p.number"),
    ("entry_date", "p.entry_date"),
    ("sell_date", "COALESCE(p.sell_date, p.entry_date)"),
    ("status", "p.status"),
    ("supplier", "p.supplier_text"),
    ("item", "it.code"),
    ("trip", "v.number"),
    ("qty", "p.qty"),
    ("qty_sold", "p.qty_sold"),
    ("qty_left", "(p.qty - p.qty_sold - p.qty_written_off)"),
    ("rate_paisa", "p.rate_paisa"),
    ("value_paisa", "p.value_paisa"),
    ("total_cost_paisa", "p.total_cost_paisa"),
    (
        "cost_left_paisa",
        "(p.total_cost_paisa - p.cost_released_paisa)",
    ),
];

/// The `WHERE` of a filter and the values it binds, in order. Written with `?` only, and with
/// subqueries instead of joins for the search, so the same text serves the page query, the totals
/// and the month sums. An empty filter gives an empty text.
fn filter_where(filter: &ListFilter) -> (String, Vec<Value>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<Value> = Vec::new();
    if let Some(trip) = filter.trip_id {
        conditions.push("p.venture_id = ?".to_owned());
        binds.push(Value::Integer(trip));
    }
    if let Some(from) = &filter.from {
        conditions.push("p.entry_date >= ?".to_owned());
        binds.push(Value::Text(from.clone()));
    }
    if let Some(to) = &filter.to {
        conditions.push("p.entry_date <= ?".to_owned());
        binds.push(Value::Text(to.clone()));
    }
    if let Some(item) = filter.item_id {
        conditions.push("p.item_id = ?".to_owned());
        binds.push(Value::Integer(item));
    }
    match filter.state {
        None if filter.hide_cancelled => conditions.push("p.status = 'saved'".to_owned()),
        None => {}
        Some(StateFilter::InStock) => conditions
            .push("p.status = 'saved' AND p.qty - p.qty_sold - p.qty_written_off > 0".to_owned()),
        Some(StateFilter::SoldOut) => conditions
            .push("p.status = 'saved' AND p.qty - p.qty_sold - p.qty_written_off = 0".to_owned()),
        Some(StateFilter::Cancelled) => conditions.push("p.status = 'cancelled'".to_owned()),
    }
    if let Some(text) = filter
        .search
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        let mut parts = String::new();
        if let Ok(number) = text.parse::<i64>() {
            parts.push_str("p.number = ? OR ");
            binds.push(Value::Integer(number));
        }
        parts.push_str(
            "p.supplier_text LIKE ? ESCAPE '\\' OR p.shop_name LIKE ? ESCAPE '\\'
             OR p.item_id IN (SELECT id FROM items WHERE code LIKE ? ESCAPE '\\'
                 OR name_ur LIKE ? ESCAPE '\\' OR name_en LIKE ? ESCAPE '\\')
             OR p.venture_id IN (SELECT id FROM ventures
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

/// One page of the purchases that match, newest day first, then by number (Code Standards B31).
///
/// # Errors
/// Any SQLite error.
pub fn list(
    conn: &Connection,
    filter: &ListFilter,
    offset: i64,
    limit: i64,
) -> rusqlite::Result<Vec<PurchaseRec>> {
    let (condition, mut binds) = filter_where(filter);
    binds.push(Value::Integer(limit));
    binds.push(Value::Integer(offset));
    // Newest day first, then by number. A sort puts its column first and keeps that order after
    // it, so ties come out the same on every page. A sort by the day keeps the numbers in the
    // same direction.
    let order = match filter.order_by.as_deref() {
        None => "p.entry_date DESC, p.number DESC".to_owned(),
        Some(o) if o.starts_with("p.entry_date") => {
            let direction = if o.ends_with("DESC") { "DESC" } else { "ASC" };
            format!("{o}, p.number {direction}, p.id {direction}")
        }
        Some(o) => format!("{o}, p.entry_date DESC, p.number DESC, p.id DESC"),
    };
    let mut statement = conn.prepare(&format!(
        "SELECT {REC_COLUMNS} {REC_FROM}{condition} ORDER BY {order} LIMIT ? OFFSET ?"
    ))?;
    let rows = statement.query_map(params_from_iter(binds.iter()), read_rec)?;
    rows.collect()
}

/// What a filter adds up to, over every purchase that matches and not only one page (B31).
/// Cancelled purchases count in `count` and in nothing else.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Totals {
    /// How many purchases match.
    pub count: i64,
    /// Σ total cost of the saved ones, paisa.
    pub total_cost_paisa: i64,
    /// Σ cost left of the saved ones, paisa.
    pub cost_left_paisa: i64,
    /// Σ نگ left of the saved ones.
    pub qty_left: i64,
    /// Σ cost released of the saved ones, paisa.
    pub cost_released_paisa: i64,
    /// Σ sales money in our shop of the saved ones, paisa.
    pub sales_paisa: i64,
    /// Σ money of sales in other shops of the saved ones, paisa.
    pub other_shop_paisa: i64,
    /// Σ commission on the goods sold in our shop, of the saved ones (plan 36).
    pub commission: crate::commission::Commission,
}

/// The totals of a filter, in four indexed queries.
///
/// # Errors
/// Any SQLite error.
pub fn totals(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<Totals> {
    let (condition, binds) = filter_where(filter);
    // The second and later queries read only the saved ones.
    let saved = if condition.is_empty() {
        " WHERE"
    } else {
        " AND"
    };
    let args = || params_from_iter(binds.iter());
    let mut out = conn.query_row(
        &format!(
            "SELECT COUNT(*),
                    COALESCE(SUM(CASE WHEN p.status = 'saved' THEN p.total_cost_paisa END), 0),
                    COALESCE(SUM(CASE WHEN p.status = 'saved' THEN p.qty - p.qty_sold - p.qty_written_off END), 0),
                    COALESCE(SUM(CASE WHEN p.status = 'saved' THEN p.cost_released_paisa END), 0)
             FROM purchases p{condition}"
        ),
        args(),
        |r| {
            Ok(Totals {
                count: r.get(0)?,
                total_cost_paisa: r.get(1)?,
                // Worked out below from the two sums, with a checked subtraction.
                cost_left_paisa: 0,
                qty_left: r.get(2)?,
                cost_released_paisa: r.get(3)?,
                sales_paisa: 0,
                other_shop_paisa: 0,
                commission: crate::commission::Commission::default(),
            })
        },
    )?;
    out.cost_left_paisa = out
        .total_cost_paisa
        .checked_sub(out.cost_released_paisa)
        .ok_or(rusqlite::Error::IntegralValueOutOfRange(
            1,
            out.cost_released_paisa,
        ))?;
    out.sales_paisa = conn.query_row(
        &format!(
            "SELECT COALESCE(SUM(c.amount_paisa), 0)
             FROM purchases p
             JOIN bikri_lots l ON l.purchase_id = p.id
             JOIN lot_credit_parts c ON c.lot_id = l.id
             {condition}{saved} p.status = 'saved'"
        ),
        args(),
        |r| r.get(0),
    )?;
    out.other_shop_paisa = conn.query_row(
        &format!(
            "SELECT COALESCE(SUM(x.amount_paisa), 0)
             FROM purchases p
             JOIN venture_entries x ON x.purchase_id = p.id AND x.kind = 'other_sale' AND x.status = 'saved'
             {condition}{saved} p.status = 'saved'"
        ),
        args(),
        |r| r.get(0),
    )?;
    out.commission = crate::commission::of_purchases_where(conn, &condition, args())?;
    Ok(out)
}

/// The count of each month of the filter (the day bought) with the total cost, the cost left and
/// the نگ left of the saved ones, oldest month first, in one grouped query (plan 38 R4). The
/// result and the commission need the sales, so they are in the page and the totals only.
///
/// # Errors
/// Any SQLite error.
pub fn list_months(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<Vec<MonthSums>> {
    let (condition, binds) = filter_where(filter);
    month_sums(
        conn,
        "purchases p",
        &condition,
        "p.entry_date",
        &[
            "CASE WHEN p.status = 'saved' THEN p.total_cost_paisa ELSE 0 END",
            "CASE WHEN p.status = 'saved' THEN p.total_cost_paisa - p.cost_released_paisa ELSE 0 END",
            "CASE WHEN p.status = 'saved' THEN p.qty - p.qty_sold - p.qty_written_off ELSE 0 END",
        ],
        &binds,
    )
}

/// What the purchases brought in and gave up, from stored rows: the money of the own lots (the sum of
/// the partners' parts of their seller credits, `lot_credit_parts`, reversals counted), the money of
/// sales in other shops, and the cost written off (a write-off moves its cost on the stock move).
/// The cost sold is what was released less what was written off. By purchase.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SoldFigures {
    /// Σ the seller credits of the own lots (C25 parts, reversals counted), paisa: sales money in
    /// our shop.
    pub sales_paisa: i64,
    /// Σ money of the sales in other shops in force, paisa.
    pub other_shop_paisa: i64,
    /// Σ cost of the write-offs not undone, paisa (C21).
    pub cost_written_off_paisa: i64,
}

/// The sold figures of these purchases, in three grouped queries.
///
/// # Errors
/// Any SQLite error.
pub fn sold_figures(conn: &Connection, ids: &[i64]) -> rusqlite::Result<HashMap<i64, SoldFigures>> {
    let mut out: HashMap<i64, SoldFigures> = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let marks = vec!["?"; ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT l.purchase_id, COALESCE(SUM(c.amount_paisa), 0)
         FROM bikri_lots l JOIN lot_credit_parts c ON c.lot_id = l.id
         WHERE l.purchase_id IN ({marks}) GROUP BY l.purchase_id"
    ))?;
    for row in statement.query_map(rusqlite::params_from_iter(ids), |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
    })? {
        let (id, sales) = row?;
        out.entry(id).or_default().sales_paisa = sales;
    }
    let mut statement = conn.prepare(&format!(
        "SELECT x.purchase_id, COALESCE(SUM(x.amount_paisa), 0)
         FROM venture_entries x
         WHERE x.purchase_id IN ({marks}) AND x.kind = 'other_sale' AND x.status = 'saved'
         GROUP BY x.purchase_id"
    ))?;
    for row in statement.query_map(rusqlite::params_from_iter(ids), |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
    })? {
        let (id, money) = row?;
        out.entry(id).or_default().other_shop_paisa = money;
    }
    // A write-off and the reversal that undoes it add up to 0; a sale in another shop and its
    // reversal are not written off, so only the moves of kind 'written_off' and their reversals count.
    let mut statement = conn.prepare(&format!(
        "SELECT m.purchase_id, COALESCE(SUM(m.cost_paisa), 0)
         FROM stock_moves m
         WHERE m.purchase_id IN ({marks})
           AND (m.kind = 'written_off'
                OR (m.kind = 'reversal'
                    AND EXISTS (SELECT 1 FROM stock_moves o WHERE o.id = m.reverses_move_id AND o.kind = 'written_off')))
         GROUP BY m.purchase_id"
    ))?;
    for row in statement.query_map(rusqlite::params_from_iter(ids), |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
    })? {
        let (id, cost) = row?;
        out.entry(id).or_default().cost_written_off_paisa = cost;
    }
    Ok(out)
}

/// An own lot of a purchase, with what its lines in force sold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotOfPurchase {
    /// The بکری.
    pub bikri_id: i64,
    /// Its number.
    pub bikri_number: i64,
    /// The lot.
    pub lot_id: i64,
    /// The lot number.
    pub lot_no: i64,
    /// The day of the sale (the first posting), `YYYY-MM-DD`.
    pub entry_date: String,
    /// نگ sold on the lines in force.
    pub qty_sold: i64,
    /// Σ line cost, paisa.
    pub cost_paisa: i64,
    /// The seller credit of the lot as stored in its partners' parts, paisa.
    pub sales_paisa: i64,
}

/// The own lots of a purchase in saved بکری, oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn lots_of(conn: &Connection, purchase_id: i64) -> rusqlite::Result<Vec<LotOfPurchase>> {
    let mut statement = conn.prepare(
        "SELECT e.id, e.number, l.id, l.lot_no, e.entry_date,
                COALESCE((SELECT SUM(s.qty) FROM bikri_sales s WHERE s.lot_id = l.id AND s.is_active = 1), 0),
                COALESCE((SELECT SUM(s.cost_paisa) FROM bikri_sales s WHERE s.lot_id = l.id AND s.is_active = 1), 0),
                COALESCE((SELECT SUM(c.amount_paisa) FROM lot_credit_parts c WHERE c.lot_id = l.id), 0)
         FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id
         WHERE l.purchase_id = ?1 AND e.status = 'saved' ORDER BY e.entry_date, e.number, l.lot_no",
    )?;
    let rows = statement.query_map([purchase_id], |r| {
        Ok(LotOfPurchase {
            bikri_id: r.get(0)?,
            bikri_number: r.get(1)?,
            lot_id: r.get(2)?,
            lot_no: r.get(3)?,
            entry_date: r.get(4)?,
            qty_sold: r.get(5)?,
            cost_paisa: r.get(6)?,
            sales_paisa: r.get(7)?,
        })
    })?;
    rows.collect()
}

/// A write-off of a purchase that has not been undone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurchaseWriteOff {
    /// The stock move.
    pub move_id: i64,
    /// The purchase.
    pub purchase_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub move_date: String,
    /// How many نگ (more than 0).
    pub qty: i64,
    /// The cost the write-off released (C21), paisa. Nothing is posted for it.
    pub cost_paisa: i64,
    /// The reason.
    pub reason: Option<String>,
}

fn read_write_off(r: &Row<'_>) -> rusqlite::Result<PurchaseWriteOff> {
    let qty: i64 = r.get(3)?;
    Ok(PurchaseWriteOff {
        move_id: r.get(0)?,
        purchase_id: r.get(1)?,
        move_date: r.get(2)?,
        qty: qty.saturating_neg(),
        cost_paisa: r.get(4)?,
        reason: r.get(5)?,
    })
}

/// A write-off of a purchase by its move id, if it is one and has not been undone.
///
/// # Errors
/// Any SQLite error.
pub fn write_off(conn: &Connection, move_id: i64) -> rusqlite::Result<Option<PurchaseWriteOff>> {
    conn.query_row(
        "SELECT m.id, m.purchase_id, m.move_date, m.qty, m.cost_paisa, m.reason FROM stock_moves m
         WHERE m.id = ?1 AND m.kind = 'written_off' AND m.lot_id IS NULL AND m.purchase_id IS NOT NULL
           AND NOT EXISTS (SELECT 1 FROM stock_moves r WHERE r.reverses_move_id = m.id)",
        [move_id],
        read_write_off,
    )
    .optional()
}

/// The write-offs of these purchases that have not been undone, oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn write_offs_of(
    conn: &Connection,
    purchase_ids: &[i64],
) -> rusqlite::Result<Vec<PurchaseWriteOff>> {
    if purchase_ids.is_empty() {
        return Ok(Vec::new());
    }
    let marks = vec!["?"; purchase_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT m.id, m.purchase_id, m.move_date, m.qty, m.cost_paisa, m.reason FROM stock_moves m
         WHERE m.purchase_id IN ({marks}) AND m.kind = 'written_off' AND m.lot_id IS NULL
           AND NOT EXISTS (SELECT 1 FROM stock_moves r WHERE r.reverses_move_id = m.id)
         ORDER BY m.move_date, m.id"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(purchase_ids), read_write_off)?;
    rows.collect()
}

/// How a trip cost was spread over a purchase (PRD C24), to write.
///
/// # Errors
/// Any SQLite error.
pub fn insert_cost_part(
    conn: &Connection,
    entry_id: i64,
    purchase_id: i64,
    amount_paisa: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO purchase_cost_parts (created_at, created_by, updated_at, updated_by, entry_id,
             purchase_id, amount_paisa)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5)",
        params![now, user_id, entry_id, purchase_id, amount_paisa],
    )?;
    Ok(())
}

/// One part of a trip cost on a purchase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostPart {
    /// The purchase.
    pub purchase_id: i64,
    /// Its part, paisa.
    pub amount_paisa: i64,
}

/// The parts a trip cost was spread into, by purchase.
///
/// # Errors
/// Any SQLite error.
pub fn cost_parts_of_entry(conn: &Connection, entry_id: i64) -> rusqlite::Result<Vec<CostPart>> {
    let mut statement = conn.prepare(
        "SELECT purchase_id, amount_paisa FROM purchase_cost_parts WHERE entry_id = ?1 ORDER BY purchase_id",
    )?;
    let rows = statement.query_map([entry_id], |r| {
        Ok(CostPart {
            purchase_id: r.get(0)?,
            amount_paisa: r.get(1)?,
        })
    })?;
    rows.collect()
}

/// True if a purchase carries a part of a trip cost that is not cancelled (it then cannot be
/// cancelled before that cost, PRD FR-12.31).
///
/// # Errors
/// Any SQLite error.
pub fn carries_trip_cost(conn: &Connection, purchase_id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM purchase_cost_parts c JOIN venture_entries x ON x.id = c.entry_id
                        WHERE c.purchase_id = ?1 AND x.status = 'saved' AND c.amount_paisa > 0)",
        [purchase_id],
        |r| r.get(0),
    )
}

/// The "received" move of a purchase, which a cancel reverses: its id and day.
///
/// # Errors
/// Any SQLite error.
pub fn received_move(
    conn: &Connection,
    purchase_id: i64,
) -> rusqlite::Result<Option<(i64, String)>> {
    conn.query_row(
        "SELECT id, move_date FROM stock_moves
         WHERE purchase_id = ?1 AND lot_id IS NULL AND kind = 'received'
           AND NOT EXISTS (SELECT 1 FROM stock_moves r WHERE r.reverses_move_id = stock_moves.id)",
        [purchase_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
}

/// One part of a trip cost with the purchase it went to, for the history of a trip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostPartDetail {
    /// The trip cost entry.
    pub entry_id: i64,
    /// The purchase.
    pub purchase_id: i64,
    /// Its number.
    pub purchase_number: i64,
    /// Its item.
    pub item: RecordRef,
    /// نگ bought.
    pub qty: i64,
    /// نگ sold.
    pub qty_sold: i64,
    /// نگ written off.
    pub qty_written_off: i64,
    /// Its part of the cost, paisa.
    pub amount_paisa: i64,
}

/// The parts of these trip costs with their purchases, by entry and purchase id.
///
/// # Errors
/// Any SQLite error.
pub fn cost_parts_detail(
    conn: &Connection,
    entry_ids: &[i64],
) -> rusqlite::Result<Vec<CostPartDetail>> {
    if entry_ids.is_empty() {
        return Ok(Vec::new());
    }
    let marks = vec!["?"; entry_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT c.entry_id, c.purchase_id, pu.number, it.id, it.code, it.name_ur, it.name_en, pu.qty,
                pu.qty_sold, pu.qty_written_off, c.amount_paisa
         FROM purchase_cost_parts c JOIN purchases pu ON pu.id = c.purchase_id
         JOIN items it ON it.id = pu.item_id
         WHERE c.entry_id IN ({marks}) ORDER BY c.entry_id, c.purchase_id"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(entry_ids), |r| {
        Ok(CostPartDetail {
            entry_id: r.get(0)?,
            purchase_id: r.get(1)?,
            purchase_number: r.get(2)?,
            item: RecordRef {
                id: r.get(3)?,
                code: r.get(4)?,
                name_ur: r.get(5)?,
                name_en: r.get(6)?,
                tag: None,
            },
            qty: r.get(7)?,
            qty_sold: r.get(8)?,
            qty_written_off: r.get(9)?,
            amount_paisa: r.get(10)?,
        })
    })?;
    rows.collect()
}

/// The trips whose purchases the lines of a posting sold from (in force or not), in id order. A
/// sale, an edit or a cancel of an own lot checks each of them again (PRD FR-12.33).
///
/// # Errors
/// Any SQLite error.
pub fn trips_of_entry(conn: &Connection, entry_id: i64) -> rusqlite::Result<Vec<i64>> {
    let mut statement = conn.prepare(
        "SELECT DISTINCT p.venture_id
         FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id
         JOIN purchases p ON p.id = l.purchase_id
         WHERE s.journal_entry_id = ?1
         ORDER BY p.venture_id",
    )?;
    let rows = statement.query_map([entry_id], |r| r.get(0))?;
    rows.collect()
}

/// The saved purchases of open trips with goods left and a day to sell on or before `date`, oldest
/// day to sell first, then by number (PRD FR-12.39, FR-12.40). At most `limit` rows.
///
/// # Errors
/// Any SQLite error.
pub fn ready_by(conn: &Connection, date: &str, limit: i64) -> rusqlite::Result<Vec<PurchaseRec>> {
    let mut statement = conn.prepare(&format!(
        "SELECT {REC_COLUMNS} {REC_FROM}
         WHERE p.status = 'saved' AND v.status = 'open'
           AND p.qty - p.other_qty - (p.qty_sold - p.qty_sold_other) - p.qty_written_off > 0
           AND COALESCE(p.sell_date, p.entry_date) <= ?1
         ORDER BY COALESCE(p.sell_date, p.entry_date), p.number LIMIT ?2"
    ))?;
    let rows = statement.query_map(params![date, limit], read_rec)?;
    rows.collect()
}
