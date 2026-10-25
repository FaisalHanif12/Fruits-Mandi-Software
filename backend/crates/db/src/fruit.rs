//! Fruit taken (PRD 16.7, FR-16.10 to FR-16.12, plan 17d): the table `fruit_takes` and what reads it.
//!
//! No money is worked out here. The value is stored by the service that saved the entry, from the
//! domain function that worked it out. A saved entry's parts are not stored in a table of their own;
//! they are read from the journal lines of the posting in force (the 0604 debits). Sums use `SUM()`
//! on `INTEGER` `*_paisa` columns only.

use mandi_core::accounts::RecordRef;
use rusqlite::{Connection, OptionalExtension, Row, params};

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
        f.status, f.cancel_reason, f.current_entry_id
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
        },
        pukhta_person_key: key,
        pukhta,
        item: RecordRef {
            id: r.get(20)?,
            code: r.get(21)?,
            name_ur: r.get(22)?,
            name_en: r.get(23)?,
        },
        packing: RecordRef {
            id: r.get(24)?,
            code: r.get(25)?,
            name_ur: r.get(26)?,
            name_en: r.get(27)?,
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
}

const FILTER_WHERE: &str = "WHERE (?1 IS NULL OR f.partner_id = ?1)
       AND (?2 = 0 OR f.partner_id IS NULL)
       AND (?3 IS NULL OR f.item_id = ?3) AND (?4 IS NULL OR f.source = ?4)
       AND (?5 IS NULL OR f.entry_date >= ?5) AND (?6 IS NULL OR f.entry_date <= ?6)
       AND (?7 IS NULL OR f.status = ?7)";

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
    let mut statement = conn.prepare(&format!(
        "{SELECT} {FILTER_WHERE} ORDER BY f.entry_date DESC, f.number DESC LIMIT ?8 OFFSET ?9"
    ))?;
    let rows = statement.query_map(
        params![
            filter.partner_id,
            i64::from(filter.guests_only),
            filter.item_id,
            filter.source,
            filter.from,
            filter.to,
            filter.status,
            limit,
            offset
        ],
        read,
    )?;
    rows.collect()
}

/// The row count of the whole filter, and Σ نگ and Σ value of its saved entries, paisa (B31).
///
/// # Errors
/// Any SQLite error.
pub fn totals(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<(i64, i64, i64)> {
    conn.query_row(
        &format!(
            "SELECT COUNT(*),
                    COALESCE(SUM(CASE WHEN f.status = 'saved' THEN f.qty END), 0),
                    COALESCE(SUM(CASE WHEN f.status = 'saved' THEN f.value_paisa END), 0)
             FROM fruit_takes f {FILTER_WHERE}"
        ),
        params![
            filter.partner_id,
            i64::from(filter.guests_only),
            filter.item_id,
            filter.source,
            filter.from,
            filter.to,
            filter.status
        ],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
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
        },
        packing: RecordRef {
            id: r.get(17)?,
            code: r.get(18)?,
            name_ur: r.get(19)?,
            name_en: r.get(20)?,
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
                l.qty_received - l.qty_sold - l.qty_written_off - l.qty_taken
         {LOT_FROM} {LOT_WHERE}
         ORDER BY e.entry_date, e.number, l.lot_no LIMIT ?2"
    ))?;
    let rows = statement.query_map(params![search, limit], read_lot)?;
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
                    l.qty_received - l.qty_sold - l.qty_written_off - l.qty_taken
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
        "SELECT a.id, a.code, a.name_ur, a.name_en, c.code, a.is_active
         FROM accounts a JOIN account_controls c ON c.id = a.control_id WHERE a.id = ?1",
        [id],
        |r| {
            Ok((
                RecordRef {
                    id: r.get(0)?,
                    code: r.get(1)?,
                    name_ur: r.get(2)?,
                    name_en: r.get(3)?,
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
