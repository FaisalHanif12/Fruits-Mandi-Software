//! Trips and gardens in SQL (PRD 12.10, 12.11, plan 16b): the rows of `ventures`, `venture_partners`,
//! `venture_entries` and `venture_entry_parts`. No rules here: shares, parts and the cost spread are
//! worked out in `mandi_domain::ventures`, checked and written by `services::trips`. Sums of money
//! are `SUM()` on `INTEGER` columns only (money agent Section 9); the caller adds them with checked
//! maths.
//!
//! A venture is never deleted and a saved entry is never changed (Code Standards D3, B17): a cancel
//! sets the status. The shares are locked by a trigger once the venture has an entry (PRD FR-12.29).

use std::collections::{HashMap, HashSet};

use mandi_core::accounts::RecordRef;
use rusqlite::{Connection, OptionalExtension, Row, params};

// ---- The venture ----

/// A venture to insert. A trip's names and search keys are made from its number and date by the
/// caller.
#[derive(Debug, Clone)]
pub struct NewVenture<'a> {
    /// `trip` or `garden`.
    pub kind: &'a str,
    /// Its number in the series of its kind.
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// Urdu name.
    pub name_ur: &'a str,
    /// English name.
    pub name_en: &'a str,
    /// `search_ur`.
    pub search_ur: &'a str,
    /// `search_en`.
    pub search_en: &'a str,
    /// `search_phonetic`.
    pub search_phonetic: &'a str,
    /// The market, as typed.
    pub market: Option<&'a str>,
    /// Who was sent, as typed.
    pub sent_person: Option<&'a str>,
    /// A note.
    pub note: Option<&'a str>,
}

/// The next number of a kind: the biggest ever given plus one.
///
/// # Errors
/// Any SQLite error.
pub fn next_number(conn: &Connection, kind: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM ventures WHERE kind = ?1",
        [kind],
        |r| r.get(0),
    )
}

/// Adds a venture and returns its id.
///
/// # Errors
/// Any SQLite error (the CHECKs of `ventures`).
pub fn insert(
    conn: &Connection,
    venture: &NewVenture<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO ventures (created_at, created_by, updated_at, updated_by, kind, number, entry_date,
             name_ur, name_en, search_ur, search_en, search_phonetic, market, sent_person, note)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            now,
            user_id,
            venture.kind,
            venture.number,
            venture.entry_date,
            venture.name_ur,
            venture.name_en,
            venture.search_ur,
            venture.search_en,
            venture.search_phonetic,
            venture.market,
            venture.sent_person,
            venture.note,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// A venture as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VentureRec {
    /// Row id.
    pub id: i64,
    /// Version.
    pub version: i64,
    /// `trip` or `garden`.
    pub kind: String,
    /// Its number.
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The market.
    pub market: Option<String>,
    /// Who was sent.
    pub sent_person: Option<String>,
    /// A note.
    pub note: Option<String>,
    /// `open` or `closed`.
    pub status: String,
    /// The day it closed.
    pub closed_date: Option<String>,
}

const VENTURE_COLUMNS: &str =
    "v.id, v.version, v.kind, v.number, v.entry_date, v.name_ur, v.name_en,
    v.market, v.sent_person, v.note, v.status, v.closed_date";

fn read_venture(r: &Row<'_>) -> rusqlite::Result<VentureRec> {
    Ok(VentureRec {
        id: r.get(0)?,
        version: r.get(1)?,
        kind: r.get(2)?,
        number: r.get(3)?,
        entry_date: r.get(4)?,
        name_ur: r.get(5)?,
        name_en: r.get(6)?,
        market: r.get(7)?,
        sent_person: r.get(8)?,
        note: r.get(9)?,
        status: r.get(10)?,
        closed_date: r.get(11)?,
    })
}

/// One venture by id.
///
/// # Errors
/// Any SQLite error.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<VentureRec>> {
    conn.query_row(
        &format!("SELECT {VENTURE_COLUMNS} FROM ventures v WHERE v.id = ?1"),
        [id],
        read_venture,
    )
    .optional()
}

/// The state filter of the list.
#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    /// `trip` or `garden`.
    pub kind: String,
    /// First day, `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`.
    pub to: Option<String>,
    /// `open` or `closed`.
    pub status: Option<String>,
    /// Only ventures this partner is in.
    pub partner_id: Option<i64>,
    /// Lower-case text to find in the English name, the Urdu name, the market or the sent person.
    pub search: Option<String>,
    /// The `ORDER BY` expressions chosen by a sort, from [`SORT_COLUMNS`]. `None` is newest day
    /// first.
    pub order_by: Option<String>,
}

/// The columns a list of trips or gardens can be sorted by: the name of the field of the row and
/// its SQL expression. The figures worked out from the entries (cost, sales, result) are not
/// sortable here.
pub const SORT_COLUMNS: [(&str, &str); 11] = [
    ("number", "v.number"),
    ("code", "v.number"),
    ("entry_date", "v.entry_date"),
    ("name_ur", "v.name_ur COLLATE URDU"),
    ("name_en", "v.name_en COLLATE NOCASE"),
    ("market", "v.market"),
    ("sent_person", "v.sent_person"),
    ("rent_paisa", "v.rent_paisa"),
    ("season_end_date", "v.season_end_date"),
    ("status", "v.status"),
    ("closed_date", "v.closed_date"),
];

const FILTER_WHERE: &str = "WHERE v.kind = ?1
       AND (?2 IS NULL OR v.entry_date >= ?2) AND (?3 IS NULL OR v.entry_date <= ?3)
       AND (?4 IS NULL OR v.status = ?4)
       AND (?5 IS NULL OR EXISTS (SELECT 1 FROM venture_partners vp WHERE vp.venture_id = v.id AND vp.partner_id = ?5))
       AND (?6 IS NULL OR instr(v.search_en, ?6) > 0 OR instr(v.search_ur, ?6) > 0
            OR instr(lower(COALESCE(v.market, '')), ?6) > 0 OR instr(lower(COALESCE(v.sent_person, '')), ?6) > 0)";

/// One page of the ventures that match, newest day first, then by number (Code Standards B31).
///
/// # Errors
/// Any SQLite error.
pub fn list(
    conn: &Connection,
    filter: &ListFilter,
    offset: i64,
    limit: i64,
) -> rusqlite::Result<Vec<VentureRec>> {
    // Newest day first, then by number. A sort puts its column first and keeps that order after
    // it, so ties come out the same on every page. A sort by the day keeps the numbers in the
    // same direction.
    let order = match filter.order_by.as_deref() {
        None => "v.entry_date DESC, v.number DESC".to_owned(),
        Some(o) if o.starts_with("v.entry_date") => {
            let direction = if o.ends_with("DESC") { "DESC" } else { "ASC" };
            format!("{o}, v.number {direction}, v.id {direction}")
        }
        Some(o) => format!("{o}, v.entry_date DESC, v.number DESC, v.id DESC"),
    };
    let mut statement = conn.prepare(&format!(
        "SELECT {VENTURE_COLUMNS} FROM ventures v {FILTER_WHERE}
         ORDER BY {order} LIMIT ?7 OFFSET ?8"
    ))?;
    let rows = statement.query_map(
        params![
            filter.kind,
            filter.from,
            filter.to,
            filter.status,
            filter.partner_id,
            filter.search,
            limit,
            offset
        ],
        read_venture,
    )?;
    rows.collect()
}

/// The count of each month of the filter (the day of the venture) with the garden rent, oldest
/// month first, in one grouped query (plan 38 R4).
///
/// # Errors
/// Any SQLite error.
pub fn list_months(
    conn: &Connection,
    filter: &ListFilter,
) -> rusqlite::Result<Vec<crate::listing::MonthSums>> {
    let binds = [
        rusqlite::types::Value::Text(filter.kind.clone()),
        filter.from.clone().into(),
        filter.to.clone().into(),
        filter.status.clone().into(),
        filter.partner_id.into(),
        filter.search.clone().into(),
    ];
    crate::listing::month_sums(
        conn,
        "ventures v",
        &format!(" {FILTER_WHERE}"),
        "v.entry_date",
        &["v.rent_paisa"],
        &binds,
    )
}

/// The ids of every venture that matches, to add up their figures over the whole filter (B31).
///
/// # Errors
/// Any SQLite error.
pub fn ids_of(conn: &Connection, filter: &ListFilter) -> rusqlite::Result<Vec<i64>> {
    let mut statement = conn.prepare(&format!(
        "SELECT v.id FROM ventures v {FILTER_WHERE} ORDER BY v.id"
    ))?;
    let rows = statement.query_map(
        params![
            filter.kind,
            filter.from,
            filter.to,
            filter.status,
            filter.partner_id,
            filter.search
        ],
        |r| r.get(0),
    )?;
    rows.collect()
}

/// Opens or closes a venture. `closed_date` is `Some` exactly when `status` is `closed`.
///
/// # Errors
/// Any SQLite error.
pub fn set_status(
    conn: &Connection,
    id: i64,
    status: &str,
    closed_date: Option<&str>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ventures SET status = ?2, closed_date = ?3, updated_at = ?4, updated_by = ?5,
             version = version + 1 WHERE id = ?1",
        params![id, status, closed_date, now, user_id],
    )?;
    Ok(())
}

/// Moves the version of a venture on (its partners changed).
///
/// # Errors
/// Any SQLite error.
pub fn touch(conn: &Connection, id: i64, user_id: i64, now: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ventures SET updated_at = ?2, updated_by = ?3, version = version + 1 WHERE id = ?1",
        params![id, now, user_id],
    )?;
    Ok(())
}

/// True once a venture has an entry of any kind, cancelled ones too (PRD FR-12.29).
///
/// # Errors
/// Any SQLite error.
pub fn has_entries(conn: &Connection, venture_id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM venture_entries WHERE venture_id = ?1)",
        [venture_id],
        |r| r.get(0),
    )
}

// ---- Its partners ----

/// One partner of a venture, with his names and his 0604 account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartnerShare {
    /// The partner. A lower id means created first (PRD C10 tie rule).
    pub partner_id: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// His share in bps.
    pub share_bps: i64,
    /// His default source, `shop` or `pocket`.
    pub source: String,
    /// His gardens, trips and fruit account (0604). 0 if he has none.
    pub venture_account_id: i64,
}

fn read_share(r: &Row<'_>) -> rusqlite::Result<PartnerShare> {
    Ok(PartnerShare {
        partner_id: r.get(0)?,
        name_ur: r.get(1)?,
        name_en: r.get(2)?,
        share_bps: r.get(3)?,
        source: r.get(4)?,
        venture_account_id: r.get(5)?,
    })
}

/// The partners of a venture, in the order the partners were created.
///
/// # Errors
/// Any SQLite error.
pub fn partners_of(conn: &Connection, venture_id: i64) -> rusqlite::Result<Vec<PartnerShare>> {
    let mut statement = conn.prepare(
        "SELECT p.id, p.name_ur, p.name_en, vp.share_bps, vp.source, COALESCE(p.venture_account_id, 0)
         FROM venture_partners vp JOIN partners p ON p.id = vp.partner_id
         WHERE vp.venture_id = ?1 ORDER BY p.id",
    )?;
    let rows = statement.query_map([venture_id], read_share)?;
    rows.collect()
}

/// The partners of several ventures, by venture id, each list in creation order.
///
/// # Errors
/// Any SQLite error.
pub fn partners_of_many(
    conn: &Connection,
    venture_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, Vec<PartnerShare>>> {
    let mut out: HashMap<i64, Vec<PartnerShare>> = HashMap::new();
    if venture_ids.is_empty() {
        return Ok(out);
    }
    let marks = vec!["?"; venture_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT vp.venture_id, p.id, p.name_ur, p.name_en, vp.share_bps, vp.source, COALESCE(p.venture_account_id, 0)
         FROM venture_partners vp JOIN partners p ON p.id = vp.partner_id
         WHERE vp.venture_id IN ({marks}) ORDER BY vp.venture_id, p.id"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(venture_ids), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            PartnerShare {
                partner_id: r.get(1)?,
                name_ur: r.get(2)?,
                name_en: r.get(3)?,
                share_bps: r.get(4)?,
                source: r.get(5)?,
                venture_account_id: r.get(6)?,
            },
        ))
    })?;
    for row in rows {
        let (venture_id, share) = row?;
        out.entry(venture_id).or_default().push(share);
    }
    Ok(out)
}

/// The partners of the latest trip (the highest number), for the form of a new trip (PRD A56). Empty
/// if there is no trip.
///
/// # Errors
/// Any SQLite error.
pub fn partners_of_last_trip(conn: &Connection) -> rusqlite::Result<Vec<PartnerShare>> {
    let last: Option<i64> = conn
        .query_row(
            "SELECT id FROM ventures WHERE kind = 'trip' ORDER BY number DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    match last {
        Some(id) => partners_of(conn, id),
        None => Ok(Vec::new()),
    }
}

/// Adds a partner to a venture.
///
/// # Errors
/// Any SQLite error (the CHECKs and the trigger that locks the shares after the first entry).
pub fn insert_partner(
    conn: &Connection,
    venture_id: i64,
    partner_id: i64,
    share_bps: i64,
    source: &str,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO venture_partners (created_at, created_by, updated_at, updated_by, venture_id,
             partner_id, share_bps, source)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6)",
        params![now, user_id, venture_id, partner_id, share_bps, source],
    )?;
    Ok(())
}

/// Takes all the partners off a venture that has no entry yet, before new ones are set.
///
/// # Errors
/// Any SQLite error (the trigger refuses it once the venture has an entry).
pub fn delete_partners(conn: &Connection, venture_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM venture_partners WHERE venture_id = ?1",
        [venture_id],
    )?;
    Ok(())
}

/// The ids of the active partners, in the order they were created.
///
/// # Errors
/// Any SQLite error.
pub fn active_partner_ids(conn: &Connection) -> rusqlite::Result<Vec<i64>> {
    let mut statement = conn.prepare("SELECT id FROM partners WHERE is_active = 1 ORDER BY id")?;
    let rows = statement.query_map([], |r| r.get(0))?;
    rows.collect()
}

// ---- Its entries ----

/// An entry to insert.
#[derive(Debug, Clone)]
pub struct NewEntry<'a> {
    /// The venture.
    pub venture_id: i64,
    /// `installment`, `cost`, `purchase`, `other_sale` or `close`.
    pub kind: &'a str,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// The payment, or the money received, paisa.
    pub amount_paisa: i64,
    /// Why (a payment).
    pub reason: Option<&'a str>,
    /// The purchase (a purchase, or a sale in another shop).
    pub purchase_id: Option<i64>,
    /// نگ sold in another shop.
    pub qty: Option<i64>,
    /// The shop, as typed.
    pub shop_name: Option<&'a str>,
    /// C21: the cost of the goods sold in another shop, paisa.
    pub cost_paisa: i64,
    /// The garden arrival a sale in another shop takes its goods from (PRD FR-12.41).
    pub arrival_id: Option<i64>,
    /// نگ a sale in another shop moved from our shop's part to the part for another shop (plan 35
    /// D1). 0 for every other entry.
    pub moved_qty: i64,
}

/// The next entry number of a kind: the biggest ever given plus one (all ventures share the series
/// of a kind, so a number is used once).
///
/// # Errors
/// Any SQLite error.
pub fn next_entry_number(conn: &Connection, kind: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM venture_entries WHERE kind = ?1",
        [kind],
        |r| r.get(0),
    )
}

/// Adds an entry and returns its id. Its journal entry is set right after it is posted.
///
/// # Errors
/// Any SQLite error (the CHECKs of `venture_entries`).
pub fn insert_entry(
    conn: &Connection,
    entry: &NewEntry<'_>,
    number: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO venture_entries (created_at, created_by, updated_at, updated_by, venture_id, kind,
             number, entry_date, amount_paisa, reason, purchase_id, qty, shop_name, cost_paisa, arrival_id,
             moved_qty)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            now,
            user_id,
            entry.venture_id,
            entry.kind,
            number,
            entry.entry_date,
            entry.amount_paisa,
            entry.reason,
            entry.purchase_id,
            entry.qty,
            entry.shop_name,
            entry.cost_paisa,
            entry.arrival_id,
            entry.moved_qty,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Names the journal entry of a venture entry (once, right after it is posted).
///
/// # Errors
/// Any SQLite error.
pub fn set_entry_journal(
    conn: &Connection,
    id: i64,
    journal_entry_id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE venture_entries SET journal_entry_id = ?2, current_entry_id = ?2, updated_at = ?3,
             updated_by = ?4, version = version + 1 WHERE id = ?1",
        params![id, journal_entry_id, now, user_id],
    )?;
    Ok(())
}

/// Marks an entry cancelled, with the reason.
///
/// # Errors
/// Any SQLite error.
pub fn mark_entry_cancelled(
    conn: &Connection,
    id: i64,
    reason: &str,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE venture_entries SET status = 'cancelled', cancel_reason = ?2, updated_at = ?3,
             updated_by = ?4, version = version + 1 WHERE id = ?1",
        params![id, reason, now, user_id],
    )?;
    Ok(())
}

/// An entry as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryRec {
    /// Row id.
    pub id: i64,
    /// Version.
    pub version: i64,
    /// The venture.
    pub venture_id: i64,
    /// `installment`, `cost`, `purchase`, `other_sale` or `close`.
    pub kind: String,
    /// Its number in the series of its kind.
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The payment, or the money received, paisa.
    pub amount_paisa: i64,
    /// Why.
    pub reason: Option<String>,
    /// The purchase.
    pub purchase_id: Option<i64>,
    /// The purchase's number.
    pub purchase_number: Option<i64>,
    /// نگ sold in another shop.
    pub qty: Option<i64>,
    /// The shop, as typed.
    pub shop_name: Option<String>,
    /// C21: the cost of the goods sold in another shop, paisa.
    pub cost_paisa: i64,
    /// The journal entry.
    pub journal_entry_id: Option<i64>,
    /// `saved` or `cancelled`.
    pub status: String,
    /// Why it was cancelled.
    pub cancel_reason: Option<String>,
    /// The garden arrival a sale in another shop took its goods from.
    pub arrival_id: Option<i64>,
    /// نگ the sale moved from our shop's part to the part for another shop.
    pub moved_qty: i64,
}

const ENTRY_COLUMNS: &str = "x.id, x.version, x.venture_id, x.kind, x.number, x.entry_date, x.amount_paisa,
    x.reason, x.purchase_id, pu.number, x.qty, x.shop_name, x.cost_paisa, x.journal_entry_id, x.status,
    x.cancel_reason, x.arrival_id, x.moved_qty";

const ENTRY_FROM: &str = "FROM venture_entries x LEFT JOIN purchases pu ON pu.id = x.purchase_id";

fn read_entry(r: &Row<'_>) -> rusqlite::Result<EntryRec> {
    Ok(EntryRec {
        id: r.get(0)?,
        version: r.get(1)?,
        venture_id: r.get(2)?,
        kind: r.get(3)?,
        number: r.get(4)?,
        entry_date: r.get(5)?,
        amount_paisa: r.get(6)?,
        reason: r.get(7)?,
        purchase_id: r.get(8)?,
        purchase_number: r.get(9)?,
        qty: r.get(10)?,
        shop_name: r.get(11)?,
        cost_paisa: r.get(12)?,
        journal_entry_id: r.get(13)?,
        status: r.get(14)?,
        cancel_reason: r.get(15)?,
        arrival_id: r.get(16)?,
        moved_qty: r.get(17)?,
    })
}

/// One entry by id.
///
/// # Errors
/// Any SQLite error.
pub fn get_entry(conn: &Connection, id: i64) -> rusqlite::Result<Option<EntryRec>> {
    conn.query_row(
        &format!("SELECT {ENTRY_COLUMNS} {ENTRY_FROM} WHERE x.id = ?1"),
        [id],
        read_entry,
    )
    .optional()
}

/// The venture entry of a purchase (kind `purchase`).
///
/// # Errors
/// Any SQLite error.
pub fn entry_of_purchase(
    conn: &Connection,
    purchase_id: i64,
) -> rusqlite::Result<Option<EntryRec>> {
    conn.query_row(
        &format!(
            "SELECT {ENTRY_COLUMNS} {ENTRY_FROM} WHERE x.purchase_id = ?1 AND x.kind = 'purchase'"
        ),
        [purchase_id],
        read_entry,
    )
    .optional()
}

/// One page of the history of a venture, newest first, and how many entries it has in all.
///
/// # Errors
/// Any SQLite error.
pub fn entries_page(
    conn: &Connection,
    venture_id: i64,
    offset: i64,
    limit: i64,
) -> rusqlite::Result<(Vec<EntryRec>, i64)> {
    let mut statement = conn.prepare(&format!(
        "SELECT {ENTRY_COLUMNS} {ENTRY_FROM} WHERE x.venture_id = ?1
         ORDER BY x.entry_date DESC, x.id DESC LIMIT ?2 OFFSET ?3"
    ))?;
    let rows = statement
        .query_map(params![venture_id, limit, offset], read_entry)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM venture_entries WHERE venture_id = ?1",
        [venture_id],
        |r| r.get(0),
    )?;
    Ok((rows, count))
}

// ---- The parts of an entry ----

/// One partner's part of an entry, to write.
///
/// # Errors
/// Any SQLite error.
#[allow(clippy::too_many_arguments)]
// Eight plain values of one INSERT; a struct would only repeat the same fields.
pub fn insert_part(
    conn: &Connection,
    entry_id: i64,
    partner_id: i64,
    share_bps: i64,
    source: &str,
    amount_paisa: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO venture_entry_parts (created_at, created_by, updated_at, updated_by, entry_id,
             partner_id, share_bps, source, amount_paisa)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            now,
            user_id,
            entry_id,
            partner_id,
            share_bps,
            source,
            amount_paisa
        ],
    )?;
    Ok(())
}

/// One partner's part of an entry as stored, with his names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartRec {
    /// The entry.
    pub entry_id: i64,
    /// The partner.
    pub partner: RecordRef,
    /// His share in bps.
    pub share_bps: i64,
    /// `shop` or `pocket`.
    pub source: String,
    /// His part, paisa.
    pub amount_paisa: i64,
}

/// The parts of several entries, each entry's parts in the order the partners were created.
///
/// # Errors
/// Any SQLite error.
pub fn parts_of(conn: &Connection, entry_ids: &[i64]) -> rusqlite::Result<Vec<PartRec>> {
    if entry_ids.is_empty() {
        return Ok(Vec::new());
    }
    let marks = vec!["?"; entry_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT e.entry_id, p.id, p.name_ur, p.name_en, e.share_bps, e.source, e.amount_paisa
         FROM venture_entry_parts e JOIN partners p ON p.id = e.partner_id
         WHERE e.entry_id IN ({marks}) ORDER BY e.entry_id, p.id"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(entry_ids), |r| {
        Ok(PartRec {
            entry_id: r.get(0)?,
            partner: RecordRef {
                id: r.get(1)?,
                code: String::new(),
                name_ur: r.get(2)?,
                name_en: r.get(3)?,
                tag: None,
            },
            share_bps: r.get(4)?,
            source: r.get(5)?,
            amount_paisa: r.get(6)?,
        })
    })?;
    rows.collect()
}

// ---- Figures of a venture, read from stored rows ----

/// What a partner paid into a trip and got back from its other-shop sales, from the parts of the
/// saved entries.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PartnerMoney {
    /// Σ his parts paid from the shop's cash (purchases and costs), paisa.
    pub in_shop_paisa: i64,
    /// Σ his parts paid from his pocket, paisa.
    pub in_pocket_paisa: i64,
    /// Σ his parts of the money of sales in other shops (C25), paisa.
    pub back_other_paisa: i64,
}

/// Each partner's money in and money back from other-shop sales on a venture, by partner id.
///
/// # Errors
/// Any SQLite error.
pub fn partner_money(
    conn: &Connection,
    venture_id: i64,
) -> rusqlite::Result<HashMap<i64, PartnerMoney>> {
    let mut statement = conn.prepare(
        "SELECT e.partner_id,
                COALESCE(SUM(CASE WHEN x.kind IN ('purchase', 'cost', 'installment') AND e.source = 'shop'
                                  THEN e.amount_paisa END), 0),
                COALESCE(SUM(CASE WHEN x.kind IN ('purchase', 'cost', 'installment') AND e.source = 'pocket'
                                  THEN e.amount_paisa END), 0),
                COALESCE(SUM(CASE WHEN x.kind = 'other_sale' THEN e.amount_paisa END), 0)
         FROM venture_entry_parts e JOIN venture_entries x ON x.id = e.entry_id
         WHERE x.venture_id = ?1 AND x.status = 'saved'
         GROUP BY e.partner_id",
    )?;
    let rows = statement.query_map([venture_id], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            PartnerMoney {
                in_shop_paisa: r.get(1)?,
                in_pocket_paisa: r.get(2)?,
                back_other_paisa: r.get(3)?,
            },
        ))
    })?;
    rows.collect()
}

/// What each partner got back from the seller credits of the trip's own lots (C25), from the stored
/// parts (`lot_credit_parts`, reversals counted), by partner id.
///
/// # Errors
/// Any SQLite error.
pub fn lot_credits_by_partner(
    conn: &Connection,
    venture_id: i64,
) -> rusqlite::Result<HashMap<i64, i64>> {
    let mut statement = conn.prepare(
        "SELECT c.partner_id, COALESCE(SUM(c.amount_paisa), 0)
         FROM lot_credit_parts c
         JOIN bikri_lots l ON l.id = c.lot_id
         JOIN purchases pu ON pu.id = l.purchase_id
         WHERE pu.venture_id = ?1 GROUP BY c.partner_id",
    )?;
    let rows = statement.query_map([venture_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

/// The purchase figures of ventures: how many saved purchases, the نگ, the cost and what was
/// released, by venture id. Saved purchases only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PurchaseFigures {
    /// How many saved purchases.
    pub purchase_count: i64,
    /// Σ نگ bought.
    pub qty_bought: i64,
    /// Σ نگ sold.
    pub qty_sold: i64,
    /// Σ نگ written off.
    pub qty_written_off: i64,
    /// Σ total cost, paisa.
    pub cost_paisa: i64,
    /// Σ cost released, paisa.
    pub cost_released_paisa: i64,
    /// Σ cost written off (write-offs not undone), paisa.
    pub cost_written_off_paisa: i64,
    /// Σ sales money of the own lots, paisa.
    pub sales_paisa: i64,
    /// Σ money of sales in other shops, paisa.
    pub other_shop_paisa: i64,
}

/// The figures of these ventures, in four grouped queries (Code Standards B22).
///
/// # Errors
/// Any SQLite error.
pub fn purchase_figures(
    conn: &Connection,
    venture_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, PurchaseFigures>> {
    let mut out: HashMap<i64, PurchaseFigures> = HashMap::new();
    if venture_ids.is_empty() {
        return Ok(out);
    }
    let marks = vec!["?"; venture_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT p.venture_id, COUNT(*), COALESCE(SUM(p.qty), 0), COALESCE(SUM(p.qty_sold), 0),
                COALESCE(SUM(p.qty_written_off), 0), COALESCE(SUM(p.total_cost_paisa), 0),
                COALESCE(SUM(p.cost_released_paisa), 0)
         FROM purchases p WHERE p.venture_id IN ({marks}) AND p.status = 'saved' GROUP BY p.venture_id"
    ))?;
    for row in statement.query_map(rusqlite::params_from_iter(venture_ids), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            PurchaseFigures {
                purchase_count: r.get(1)?,
                qty_bought: r.get(2)?,
                qty_sold: r.get(3)?,
                qty_written_off: r.get(4)?,
                cost_paisa: r.get(5)?,
                cost_released_paisa: r.get(6)?,
                ..PurchaseFigures::default()
            },
        ))
    })? {
        let (id, figures) = row?;
        out.insert(id, figures);
    }
    let mut statement = conn.prepare(&format!(
        "SELECT pu.venture_id, COALESCE(SUM(c.amount_paisa), 0)
         FROM lot_credit_parts c JOIN bikri_lots l ON l.id = c.lot_id
         JOIN purchases pu ON pu.id = l.purchase_id
         WHERE pu.venture_id IN ({marks}) AND pu.status = 'saved'
         GROUP BY pu.venture_id"
    ))?;
    for row in statement.query_map(rusqlite::params_from_iter(venture_ids), |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
    })? {
        let (id, sales) = row?;
        out.entry(id).or_default().sales_paisa = sales;
    }
    let mut statement = conn.prepare(&format!(
        "SELECT x.venture_id, COALESCE(SUM(x.amount_paisa), 0)
         FROM venture_entries x
         WHERE x.venture_id IN ({marks}) AND x.kind = 'other_sale' AND x.status = 'saved'
         GROUP BY x.venture_id"
    ))?;
    for row in statement.query_map(rusqlite::params_from_iter(venture_ids), |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
    })? {
        let (id, money) = row?;
        out.entry(id).or_default().other_shop_paisa = money;
    }
    let mut statement = conn.prepare(&format!(
        "SELECT pu.venture_id, COALESCE(SUM(m.cost_paisa), 0)
         FROM stock_moves m JOIN purchases pu ON pu.id = m.purchase_id
         WHERE pu.venture_id IN ({marks}) AND pu.status = 'saved'
           AND (m.kind = 'written_off'
                OR (m.kind = 'reversal'
                    AND EXISTS (SELECT 1 FROM stock_moves o WHERE o.id = m.reverses_move_id AND o.kind = 'written_off')))
         GROUP BY pu.venture_id"
    ))?;
    for row in statement.query_map(rusqlite::params_from_iter(venture_ids), |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
    })? {
        let (id, cost) = row?;
        out.entry(id).or_default().cost_written_off_paisa = cost;
    }
    Ok(out)
}

/// The tile data: how many trips are open, and over the saved purchases with goods left their
/// number, their نگ left and their cost left.
///
/// # Errors
/// Any SQLite error.
pub fn tile_totals(conn: &Connection) -> rusqlite::Result<(i64, i64, i64, i64)> {
    let open: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ventures WHERE kind = 'trip' AND status = 'open'",
        [],
        |r| r.get(0),
    )?;
    // The two money sums are read apart and subtracted here with a checked subtraction (the money
    // agent's Section 9: money arithmetic is not done in SQL).
    let (with_stock, qty_left, total_cost, released): (i64, i64, i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(qty - qty_sold - qty_written_off), 0),
                COALESCE(SUM(total_cost_paisa), 0), COALESCE(SUM(cost_released_paisa), 0)
         FROM purchases p JOIN ventures v ON v.id = p.venture_id
         WHERE v.kind = 'trip' AND v.status = 'open' AND p.status = 'saved'
           AND p.qty - p.qty_sold - p.qty_written_off > 0",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )?;
    let cost_left = total_cost
        .checked_sub(released)
        .ok_or(rusqlite::Error::IntegralValueOutOfRange(3, released))?;
    Ok((open, with_stock, qty_left, cost_left))
}

/// Which of these ventures already have an entry of any kind (their shares are locked).
///
/// # Errors
/// Any SQLite error.
pub fn with_entries(conn: &Connection, venture_ids: &[i64]) -> rusqlite::Result<HashSet<i64>> {
    if venture_ids.is_empty() {
        return Ok(HashSet::new());
    }
    let marks = vec!["?"; venture_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT DISTINCT venture_id FROM venture_entries WHERE venture_id IN ({marks})"
    ))?;
    let rows = statement.query_map(rusqlite::params_from_iter(venture_ids), |r| r.get(0))?;
    rows.collect()
}

/// The open trips or the open gardens as the pickers search them (plan 34): the code of each (the
/// number in its own series as a code), both names, the market of a trip as the second line, the three
/// search columns, and the flag that says it is open. A closed trip or garden is not offered in a
/// picker. `kind` is `trip` or `garden`.
///
/// # Errors
/// Any SQLite error.
pub fn search_sources(
    conn: &Connection,
    kind: &str,
    include_closed: bool,
) -> rusqlite::Result<Vec<crate::masters::SearchSource>> {
    let code_type = if kind == "garden" {
        mandi_core::codes::CodeType::Garden
    } else {
        mandi_core::codes::CodeType::Trip
    };
    let mut statement = conn.prepare(
        "SELECT v.id, v.number, v.name_ur, v.name_en, COALESCE(v.market, ''),
                v.search_ur, v.search_en, v.search_phonetic, v.status = 'open'
         FROM ventures v WHERE v.kind = ?1 ORDER BY v.number",
    )?;
    let rows = statement.query_map([kind], |r| {
        let market: String = r.get(4)?;
        Ok(crate::masters::SearchSource {
            id: r.get(0)?,
            code: mandi_core::codes::number_code(code_type, r.get(1)?),
            name_ur: r.get(2)?,
            name_en: r.get(3)?,
            extra_ur: market.clone(),
            extra_en: market,
            more_keys: String::new(),
            search_ur: r.get(5)?,
            search_en: r.get(6)?,
            search_phonetic: r.get(7)?,
            // An entry field offers the open ones only; a filter field offers them all.
            is_active: include_closed || r.get::<_, bool>(8)?,
            account_type: None,
            control_id: None,
            control_code: None,
            tag: None,
            sold_by_weight: false,
        })
    })?;
    rows.collect()
}
