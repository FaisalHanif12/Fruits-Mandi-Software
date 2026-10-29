//! Fruit taken (PRD 16.7, FR-16.10 to FR-16.12, plan 17d): the table `fruit_takes` and what reads it.
//!
//! No money is worked out here. The value is stored by the service that saved the entry, from the
//! domain function that worked it out. A saved entry's parts are not stored in a table of their own;
//! they are read from the journal lines of the posting in force (the 0604 debits). Sums use `SUM()`
//! on `INTEGER` `*_paisa` columns only.

use mandi_core::accounts::RecordRef;
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Row, params, params_from_iter};

use crate::listing::{MonthSums, like_contains, like_prefix, month_sums};

/// One fruit entry with the names its screens show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FruitRec {
    /// Row id.
    pub id: i64,
    /// Row version.
    pub version: i64,
    /// Number in its own series.
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The partner who took it, or `None` for the shop's guests.
    pub partner: Option<RecordRef>,
    /// `shop` or `other_shop`.
    pub source: String,
    /// The lot: id, number inside its بکری, بکری id, بکری number.
    pub lot: Option<(i64, i64, i64, i64)>,
    /// The account credited.
    pub credit: RecordRef,
    /// The person key of a پختہ کھاتہ credit.
    pub pukhta_person_key: Option<String>,
    /// The one-time seller of the lot (Urdu name, English name, phone, key), if any.
    pub pukhta: Option<(String, String, String, String)>,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// نگ taken.
    pub qty: i64,
    /// Weight in grams.
    pub weight_grams: Option<i64>,
    /// The rate, paisa.
    pub rate_paisa: i64,
    /// The value, paisa.
    pub value_paisa: i64,
    /// The note.
    pub note: Option<String>,
    /// The first posting.
    pub journal_entry_id: Option<i64>,
    /// `saved` or `cancelled`.
    pub status: String,
    /// Why it was cancelled.
    pub cancel_reason: Option<String>,
    /// The posting in force (the last one posted; after a cancel, the one that was reversed).
    pub current_entry_id: Option<i64>,
}

const SELECT: &str = "SELECT f.id, f.version, f.number, f.entry_date,
        f.partner_id, pa.name_ur, pa.name_en,
        f.source, f.lot_id, l.lot_no, l.bikri_id, b.number,
        ca.id, ca.code, ca.name_ur, ca.name_en, f.pukhta_person_key,
        l.pukhta_name_ur, l.pukhta_name_en, l.pukhta_phone,
        it.id, it.code, it.name_ur, it.name_en, pk.id, pk.code, pk.name_ur, pk.name_en,
        f.qty, f.weight_grams, f.rate_paisa, f.value_paisa, f.note, f.journal_entry_id,
        f.status, f.cancel_reason, f.current_entry_id, ca.tag
    FROM fruit_takes f
    LEFT JOIN partners pa ON pa.id = f.partner_id
    LEFT JOIN bikri_lots l ON l.id = f.lot_id
    LEFT JOIN bikri_entries b ON b.id = l.bikri_id
    JOIN accounts ca ON ca.id = f.credit_account_id
    JOIN items it ON it.id = f.item_id
    JOIN packings pk ON pk.id = f.packing_id";

fn read(r: &Row<'_>) -> rusqlite::Result<FruitRec> {
    let partner_id: Option<i64> = r.get(4)?;
    let partner = match partner_id {
        Some(id) => Some(RecordRef {
            id,
            code: String::new(),
            name_ur: r.get(5)?,
            name_en: r.get(6)?,
            tag: None,
        }),
        None => None,
    };
    let lot_id: Option<i64> = r.get(8)?;
    let lot = match lot_id {
        Some(id) => Some((id, r.get(9)?, r.get(10)?, r.get(11)?)),
        None => None,
    };
    let key: Option<String> = r.get(16)?;
    let pukhta = match &key {
        Some(key) => {
            let ur: Option<String> = r.get(17)?;
            let en: Option<String> = r.get(18)?;
            let phone: Option<String> = r.get(19)?;
            Some((
                ur.unwrap_or_default(),
                en.unwrap_or_default(),
                phone.unwrap_or_default(),
                key.clone(),
            ))
        }
        None => None,
    };
    Ok(FruitRec {
        id: r.get(0)?,
        version: r.get(1)?,
        number: r.get(2)?,
        entry_date: r.get(3)?,
        partner,
        source: r.get(7)?,
        lot,
        credit: RecordRef {
            id: r.get(12)?,
            code: r.get(13)?,
            name_ur: r.get(14)?,
            name_en: r.get(15)?,
            tag: r.get(37)?,
        },
        pukhta_person_key: key,
        pukhta,
        item: RecordRef {
            id: r.get(20)?,
            code: r.get(21)?,
            name_ur: r.get(22)?,
            name_en: r.get(23)?,
            tag: None,
        },
        packing: RecordRef {
            id: r.get(24)?,
            code: r.get(25)?,
            name_ur: r.get(26)?,
            name_en: r.get(27)?,
            tag: None,
        },
        qty: r.get(28)?,
        weight_grams: r.get(29)?,
        rate_paisa: r.get(30)?,
        value_paisa: r.get(31)?,
        note: r.get(32)?,
        journal_entry_id: r.get(33)?,
        status: r.get(34)?,
        cancel_reason: r.get(35)?,
        current_entry_id: r.get(36)?,
    })
}

/// One entry by id.
///
/// # Errors
/// Any SQLite error.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<FruitRec>> {
    conn.query_row(&format!("{SELECT} WHERE f.id = ?1"), [id], read)
        .optional()
}

/// The entry whose posting in force (or last posting) is this journal entry, if any.
///
/// # Errors
/// Any SQLite error.
pub fn of_current_entry(conn: &Connection, entry_id: i64) -> rusqlite::Result<Option<FruitRec>> {
    conn.query_row(
        &format!("{SELECT} WHERE f.current_entry_id = ?1"),
        [entry_id],
        read,
    )
    .optional()
}

/// The next number of the series: one more than the highest, never reused.
///
/// # Errors
/// Any SQLite error.
pub fn next_number(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM fruit_takes",
        [],
        |r| r.get(0),
    )
}

/// The columns of an entry as the service worked them out.
#[derive(Debug, Clone, Copy)]
pub struct NewFruit<'a> {
    /// The day, `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// The partner, or `None` for the shop's guests.
    pub partner_id: Option<i64>,
    /// `shop` or `other_shop`.
    pub source: &'a str,
    /// The lot (for `shop`).
    pub lot_id: Option<i64>,
    /// The account credited.
    pub credit_account_id: i64,
    /// The person key of a پختہ کھاتہ credit.
    pub pukhta_person_key: Option<&'a str>,
    /// The item.
    pub item_id: i64,
    /// The packing.
    pub packing_id: i64,
    /// نگ taken.
    pub qty: i64,
    /// Weight in grams.
    pub weight_grams: Option<i64>,
    /// The rate, paisa.
    pub rate_paisa: i64,
    /// The value, paisa.
    pub value_paisa: i64,
    /// The note.
    pub note: Option<&'a str>,
}

/// Inserts an entry and returns its id.
///
/// # Errors
/// Any SQLite error (the CHECKs of `fruit_takes`).
pub fn insert(
    conn: &Connection,
    fruit: &NewFruit<'_>,
    number: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO fruit_takes (created_at, created_by, updated_at, updated_by, number, entry_date,
             partner_id, source, lot_id, credit_account_id, pukhta_person_key, item_id, packing_id,
             qty, weight_grams, rate_paisa, value_paisa, note)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            now,
            user_id,
            number,
            fruit.entry_date,
            fruit.partner_id,
            fruit.source,
            fruit.lot_id,
            fruit.credit_account_id,
            fruit.pukhta_person_key,
            fruit.item_id,
            fruit.packing_id,
            fruit.qty,
            fruit.weight_grams,
            fruit.rate_paisa,
            fruit.value_paisa,
            fruit.note
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Changes the columns of an entry (an edit by reversal): the number stays, the version goes up.
///
/// # Errors
/// Any SQLite error.
pub fn update(
    conn: &Connection,
    id: i64,
    fruit: &NewFruit<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE fruit_takes SET updated_at = ?2, updated_by = ?3, version = version + 1,
             entry_date = ?4, partner_id = ?5, source = ?6, lot_id = ?7, credit_account_id = ?8,
             pukhta_person_key = ?9, item_id = ?10, packing_id = ?11, qty = ?12, weight_grams = ?13,
             rate_paisa = ?14, value_paisa = ?15, note = ?16
         WHERE id = ?1",
        params![
            id,
            now,
            user_id,
            fruit.entry_date,
            fruit.partner_id,
            fruit.source,
            fruit.lot_id,
            fruit.credit_account_id,
            fruit.pukhta_person_key,
            fruit.item_id,
            fruit.packing_id,
            fruit.qty,
            fruit.weight_grams,
            fruit.rate_paisa,
            fruit.value_paisa,
            fruit.note
        ],
    )?;
    Ok(())
}

/// Names the posting in force. `first` is set once, by the first save.
///
/// # Errors
/// Any SQLite error.
pub fn set_entries(
    conn: &Connection,
    id: i64,
    first: Option<i64>,
    current: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE fruit_takes SET journal_entry_id = COALESCE(journal_entry_id, ?2),
             current_entry_id = ?3, updated_at = ?4, updated_by = ?5, version = version + 1
         WHERE id = ?1",
        params![id, first, current, now, user_id],
    )?;
    Ok(())
}

/// Marks an entry cancelled with its reason.
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
        "UPDATE fruit_takes SET status = 'cancelled', cancel_reason = ?2, updated_at = ?3,
             updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![id, reason, now, user_id],
    )?;
    Ok(())
}

// ---- The list ----

/// What the list asks for. A field that is `None` (or false) is not a filter.
#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    /// Only this partner's entries.
    pub partner_id: Option<i64>,
    /// Only the shop's guests' entries.
    pub guests_only: bool,
    /// Only this item.
    pub item_id: Option<i64>,
    /// `shop` or `other_shop`.
    pub source: Option<String>,
    /// First day, `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`.
    pub to: Option<String>,
    /// `saved` or `cancelled`.
    pub status: Option<String>,
    /// Only entries whose number, note, item, partner or credited account holds this text.
    pub search: Option<String>,
    /// The `ORDER BY` expressions chosen by a sort, from [`SORT_COLUMNS`]. `None` is newest day
    /// first.
    pub order_by: Option<String>,
    /// True to leave the cancelled entries out when no status is chosen (plan 38 R5).
    pub hide_cancelled: bool,
}

/// The columns a list can be sorted by: the name of the field of the row and its SQL expression
/// (the list query joins the partner, the item and the credited account as `pa`, `it` and `ca`).
pub const SORT_COLUMNS: [(&str, &str); 10] = [
    ("number", "f.number"),
    ("entry_date", "f.entry_date"),
    ("partner", "pa.name_en"),
    ("source", "f.source"),
    ("credit_account", "ca.code"),
    ("item", "it.code"),
    ("qty", "f.qty"),
    ("rate_paisa", "f.rate_paisa"),
    ("value_paisa", "f.value_paisa"),
    ("status", "f.status"),
];

/// The `WHERE` of a filter and the values it binds, in order. Written with `?` only, and with
/// subqueries instead of joins for the search, so the same text serves the page query, the totals
/// and the month sums. An empty filter gives an empty text.
fn filter_where(filter: &ListFilter) -> (String, Vec<Value>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<Value> = Vec::new();
    if let Some(partner) = filter.partner_id {
        conditions.push("f.partner_id = ?".to_owned());
        binds.push(Value::Integer(partner));
    }
    if filter.guests_only {
        conditions.push("f.partner_id IS NULL".to_owned());
    }
    if let Some(item) = filter.item_id {
        conditions.push("f.item_id = ?".to_owned());
        binds.push(Value::Integer(item));
    }
    if let Some(source) = &filter.source {
        conditions.push("f.source = ?".to_owned());
        binds.push(Value::Text(source.clone()));
    }
    if let Some(from) = &filter.from {
        conditions.push("f.entry_date >= ?".to_owned());
        binds.push(Value::Text(from.clone()));
    }
    if let Some(to) = &filter.to {
        conditions.push("f.entry_date <= ?".to_owned());
        binds.push(Value::Text(to.clone()));
    }
    match &filter.status {
        Some(status) => {
            conditions.push("f.status = ?".to_owned());
            binds.push(Value::Text(status.clone()));
        }
        None if filter.hide_cancelled => conditions.push("f.status = 'saved'".to_owned()),
        None => {}
    }
    if let Some(text) = filter
        .search
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        let mut parts = String::new();
        if let Ok(number) = text.parse::<i64>() {
            parts.push_str("f.number = ? OR ");
            binds.push(Value::Integer(number));
        }
        parts.push_str(
            "f.note LIKE ? ESCAPE '\\'
             OR f.item_id IN (SELECT id FROM items WHERE code LIKE ? ESCAPE '\\'
                 OR name_ur LIKE ? ESCAPE '\\' OR name_en LIKE ? ESCAPE '\\')
             OR f.partner_id IN (SELECT id FROM partners
                 WHERE name_ur LIKE ? ESCAPE '\\' OR name_en LIKE ? ESCAPE '\\')
             OR f.credit_account_id IN (SELECT id FROM accounts WHERE code LIKE ? ESCAPE '\\'
                 OR name_ur LIKE ? ESCAPE '\\' OR name_en LIKE ? ESCAPE '\\')",
        );
        conditions.push(format!("({parts})"));
        let contains = like_contains(text);
        binds.push(Value::Text(contains.clone()));
        binds.push(Value::Text(like_prefix(text)));
        for _ in 0..4 {
            binds.push(Value::Text(contains.clone()));
        }
        binds.push(Value::Text(like_prefix(text)));
        binds.push(Value::Text(contains.clone()));
        binds.push(Value::Text(contains));
    }
    if conditions.is_empty() {
        (String::new(), binds)
    } else {
        (format!(" WHERE {}", conditions.join(" AND ")), binds)
    }
}

/// One page of entries that match, newest day first, then by number (Code Standards B31).
///
/// # Errors
/// Any SQLite error.
pub fn list(
    conn: &Connection,
    filter: &ListFilter,
    offset: i64,
    limit: i64,
) -> rusqlite::Result<Vec<FruitRec>> {
    let (condition, mut binds) = filter_where(filter);
    binds.push(Value::Integer(limit));
    binds.push(Value::Integer(offset));
    // Newest day first, then by number. A sort puts its column first and keeps that order after
    // it, so ties come out the same on every page. A sort by the day keeps the numbers in the
    // same direction.
    let order = match filter.order_by.as_deref() {
        None => "f.entry_date DESC, f.number DESC".to_owned(),
        Some(o) if o.starts_with("f.entry_date") => {
            let direction = if o.ends_with("DESC") { "DESC" } else { "ASC" };
            format!("{o}, f.number {direction}, f.id {direction}")
        }
        Some(o) => format!("{o}, f.entry_date DESC, f.number DESC, f.id DESC"),
    };
    let mut statement = conn.prepare(&format!(
        "{SELECT}{condition} ORDER BY {order} LIMIT ? OFFSET ?"
    ))?;
    let rows = statement.query_map(params_from_iter(binds.iter()), read)?;
    rows.collect()
}

/// The row count of the whole filter, and Σ نگ and Σ value of its saved entries, paisa (B31).
///
/// # Errors
/// Any SQLite error.
pub fn totals(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<(i64, i64, i64)> {
    let (condition, binds) = filter_where(filter);
    conn.query_row(
        &format!(
            "SELECT COUNT(*),
                    COALESCE(SUM(CASE WHEN f.status = 'saved' THEN f.qty END), 0),
                    COALESCE(SUM(CASE WHEN f.status = 'saved' THEN f.value_paisa END), 0)
             FROM fruit_takes f{condition}"
        ),
        params_from_iter(binds.iter()),
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
}

/// The count of each month of the filter with the نگ and the value of the saved entries, oldest
/// month first, in one grouped query (plan 38 R4).
///
/// # Errors
/// Any SQLite error.
pub fn list_months(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<Vec<MonthSums>> {
    let (condition, binds) = filter_where(filter);
    month_sums(
        conn,
        "fruit_takes f",
        &condition,
        "f.entry_date",
        &[
            "CASE WHEN f.status = 'saved' THEN f.qty ELSE 0 END",
            "CASE WHEN f.status = 'saved' THEN f.value_paisa ELSE 0 END",
        ],
        &binds,
    )
}

/// One partner's part of a posting: the debit on his 0604 account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartRec {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His part, paisa.
    pub amount_paisa: i64,
}

/// The parts of a posting: each partner's 0604 debit, in the order the partners were created.
///
/// # Errors
/// Any SQLite error.
pub fn parts_of_entry(conn: &Connection, entry_id: i64) -> rusqlite::Result<Vec<PartRec>> {
    let mut statement = conn.prepare(
        "SELECT pa.id, pa.name_ur, pa.name_en, SUM(l.dr_paisa)
         FROM journal_lines l JOIN partners pa ON pa.venture_account_id = l.account_id
         WHERE l.entry_id = ?1 AND l.dr_paisa > 0
         GROUP BY pa.id ORDER BY pa.id",
    )?;
    let rows = statement.query_map([entry_id], |r| {
        Ok(PartRec {
            partner: RecordRef {
                id: r.get(0)?,
                code: String::new(),
                name_ur: r.get(1)?,
                name_en: r.get(2)?,
                tag: None,
            },
            amount_paisa: r.get(3)?,
        })
    })?;
    rows.collect()
}

// ---- The lot picker ----

fn read_lot(r: &Row<'_>) -> rusqlite::Result<LotRec> {
    let ur: Option<String> = r.get(9)?;
    let en: Option<String> = r.get(10)?;
    let phone: Option<String> = r.get(11)?;
    let key: Option<String> = r.get(12)?;
    Ok(LotRec {
        lot_id: r.get(0)?,
        lot_no: r.get(1)?,
        bikri_id: r.get(2)?,
        bikri_number: r.get(3)?,
        received_date: r.get(4)?,
        seller: RecordRef {
            id: r.get(5)?,
            code: r.get(6)?,
            name_ur: r.get(7)?,
            name_en: r.get(8)?,
            tag: r.get(23)?,
        },
        pukhta: key.map(|key| {
            (
                ur.unwrap_or_default(),
                en.unwrap_or_default(),
                phone.unwrap_or_default(),
                key,
            )
        }),
        item: RecordRef {
            id: r.get(13)?,
            code: r.get(14)?,
            name_ur: r.get(15)?,
            name_en: r.get(16)?,
            tag: None,
        },
        packing: RecordRef {
            id: r.get(17)?,
            code: r.get(18)?,
            name_ur: r.get(19)?,
            name_en: r.get(20)?,
            tag: None,
        },
        sold_by_weight: r.get(21)?,
        balance: r.get(22)?,
    })
}

/// A lot fruit can be taken from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotRec {
    /// The lot.
    pub lot_id: i64,
    /// Its number inside the بکری.
    pub lot_no: i64,
    /// The بکری.
    pub bikri_id: i64,
    /// Its number.
    pub bikri_number: i64,
    /// The day received.
    pub received_date: String,
    /// The seller account.
    pub seller: RecordRef,
    /// The one-time seller (Urdu name, English name, phone, key).
    pub pukhta: Option<(String, String, String, String)>,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// True if the item is sold by weight.
    pub sold_by_weight: bool,
    /// نگ left: received less sold, written off and taken (C16).
    pub balance: i64,
}

/// A lot is offered when its بکری is saved, it has a balance, it is not trip goods and its seller is
/// not the account of a closed garden (PRD FR-16.11).
const LOT_WHERE: &str = "WHERE e.status = 'saved' AND l.purchase_id IS NULL
       AND l.qty_received - l.qty_sold - l.qty_written_off - l.qty_taken > 0
       AND NOT EXISTS (SELECT 1 FROM ventures v WHERE v.kind = 'garden' AND v.status = 'closed'
                       AND v.account_id = l.seller_account_id)
       AND (?1 IS NULL OR instr(sa.search_en, ?1) > 0 OR instr(sa.search_ur, ?1) > 0
            OR instr(it.search_en, ?1) > 0 OR instr(it.search_ur, ?1) > 0
            OR CAST(e.number AS TEXT) = ?1)";

const LOT_FROM: &str = "FROM bikri_lots l
       JOIN bikri_entries e ON e.id = l.bikri_id
       JOIN accounts sa ON sa.id = l.seller_account_id
       JOIN items it ON it.id = l.item_id
       JOIN packings pk ON pk.id = l.packing_id";

/// The lots fruit can be taken from, oldest received first, and how many there are in all.
///
/// # Errors
/// Any SQLite error.
pub fn lots(
    conn: &Connection,
    search: Option<&str>,
    offset: i64,
    limit: i64,
) -> rusqlite::Result<(Vec<LotRec>, i64)> {
    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) {LOT_FROM} {LOT_WHERE}"),
        [search],
        |r| r.get(0),
    )?;
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, l.lot_no, e.id, e.number, e.entry_date,
                sa.id, sa.code, sa.name_ur, sa.name_en,
                l.pukhta_name_ur, l.pukhta_name_en, l.pukhta_phone, l.pukhta_person_key,
                it.id, it.code, it.name_ur, it.name_en, pk.id, pk.code, pk.name_ur, pk.name_en,
                it.sold_by = 'weight',
                l.qty_received - l.qty_sold - l.qty_written_off - l.qty_taken, sa.tag
         {LOT_FROM} {LOT_WHERE}
         ORDER BY e.entry_date, e.number, l.lot_no LIMIT ?2 OFFSET ?3"
    ))?;
    let rows = statement.query_map(params![search, limit, offset], read_lot)?;
    Ok((rows.collect::<rusqlite::Result<Vec<_>>>()?, total))
}

/// One lot with its names, whatever its balance (for a check at save and an edit).
///
/// # Errors
/// Any SQLite error.
pub fn lot_one(conn: &Connection, lot_id: i64) -> rusqlite::Result<Option<LotRec>> {
    conn.query_row(
        &format!(
            "SELECT l.id, l.lot_no, e.id, e.number, e.entry_date,
                    sa.id, sa.code, sa.name_ur, sa.name_en,
                    l.pukhta_name_ur, l.pukhta_name_en, l.pukhta_phone, l.pukhta_person_key,
                    it.id, it.code, it.name_ur, it.name_en, pk.id, pk.code, pk.name_ur, pk.name_en,
                    it.sold_by = 'weight',
                    l.qty_received - l.qty_sold - l.qty_written_off - l.qty_taken, sa.tag
             {LOT_FROM} WHERE l.id = ?1"
        ),
        [lot_id],
        read_lot,
    )
    .optional()
}

/// An account as a reference, with its control code and whether it is active.
///
/// # Errors
/// Any SQLite error.
pub fn account_ref(
    conn: &Connection,
    id: i64,
) -> rusqlite::Result<Option<(RecordRef, String, bool)>> {
    conn.query_row(
        "SELECT a.id, a.code, a.name_ur, a.name_en, c.code, a.is_active, a.tag
         FROM accounts a JOIN account_controls c ON c.id = a.control_id WHERE a.id = ?1",
        [id],
        |r| {
            Ok((
                RecordRef {
                    id: r.get(0)?,
                    code: r.get(1)?,
                    name_ur: r.get(2)?,
                    name_en: r.get(3)?,
                    tag: r.get(6)?,
                },
                r.get(4)?,
                r.get(5)?,
            ))
        },
    )
    .optional()
}

/// An item as a reference, with whether it is sold by weight and whether it is active.
///
/// # Errors
/// Any SQLite error.
pub fn item_ref(conn: &Connection, id: i64) -> rusqlite::Result<Option<(RecordRef, bool, bool)>> {
    conn.query_row(
        "SELECT id, code, name_ur, name_en, sold_by = 'weight', is_active FROM items WHERE id = ?1",
        [id],
        |r| {
            Ok((
                RecordRef {
                    id: r.get(0)?,
                    code: r.get(1)?,
                    name_ur: r.get(2)?,
                    name_en: r.get(3)?,
                    tag: None,
                },
                r.get(4)?,
                r.get(5)?,
            ))
        },
    )
    .optional()
}

/// A packing as a reference, with whether it is active.
///
/// # Errors
/// Any SQLite error.
pub fn packing_ref(conn: &Connection, id: i64) -> rusqlite::Result<Option<(RecordRef, bool)>> {
    conn.query_row(
        "SELECT id, code, name_ur, name_en, is_active FROM packings WHERE id = ?1",
        [id],
        |r| {
            Ok((
                RecordRef {
                    id: r.get(0)?,
                    code: r.get(1)?,
                    name_ur: r.get(2)?,
                    name_en: r.get(3)?,
                    tag: None,
                },
                r.get(4)?,
            ))
        },
    )
    .optional()
}

// ---- Stock ----

/// Changes the fruit taken from a lot by `delta` (plus when fruit is taken, minus when it is given
/// back). The table checks that sold, written off and taken together are never more than received.
///
/// # Errors
/// Any SQLite error.
pub fn add_taken(
    conn: &Connection,
    lot_id: i64,
    delta: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE bikri_lots SET qty_taken = qty_taken + ?2, updated_at = ?3, updated_by = ?4,
             version = version + 1
         WHERE id = ?1",
        params![lot_id, delta, now, user_id],
    )?;
    Ok(())
}

/// The نگ taken as fruit from the lots of this بکری that are still in force: such a بکری cannot be
/// cancelled until those entries are cancelled (plan 17d).
///
/// # Errors
/// Any SQLite error.
pub fn bikri_qty_taken(conn: &Connection, bikri_id: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(SUM(qty_taken), 0) FROM bikri_lots WHERE bikri_id = ?1",
        [bikri_id],
        |r| r.get(0),
    )
}
