//! Gardens in SQL (PRD 12.11, plan 17c): the garden columns of `ventures` (rent, season end, account,
//! closing), the figures of a garden read from its stored rows, and the lines on its account. The
//! shared parts (partners, entries, entry parts) are in `ventures.rs`. No rules here: the closing
//! split is in `mandi_domain::ventures`, checked and written by `services::gardens`.
//!
//! Sums of money are `SUM()` on `INTEGER` columns only, one for the debits and one for the credits;
//! the caller subtracts them with checked maths (money agent Section 9).
//!
//! The money in of a garden is every posted line on its account except the closing and the undo of a
//! closing (a reversal of a `garden_close` entry): the closing moves the money out to the partners, it
//! is not money in. The account's own balance counts every line, so it is 0 after the closing.

use std::collections::HashMap;

use mandi_core::accounts::RecordRef;
use rusqlite::{Connection, OptionalExtension, Row, params};

/// The part of a `WHERE` that leaves out the closing entries and their reversals (see the module
/// doc). `e` is `journal_entries`.
const NOT_A_CLOSING: &str = "e.posting_type <> 'garden_close'
       AND (e.reverses_entry_id IS NULL OR NOT EXISTS
            (SELECT 1 FROM journal_entries c WHERE c.id = e.reverses_entry_id AND c.posting_type = 'garden_close'))";

/// A new garden.
#[derive(Debug, Clone)]
pub struct NewGarden<'a> {
    /// Its number in the series of gardens.
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// Urdu name as typed.
    pub name_ur: &'a str,
    /// English name as typed.
    pub name_en: &'a str,
    /// Search keys, built by `core::text`.
    pub search_ur: &'a str,
    /// Search key.
    pub search_en: &'a str,
    /// Search key.
    pub search_phonetic: &'a str,
    /// The main fruit.
    pub item_id: Option<i64>,
    /// The rent, paisa.
    pub rent_paisa: i64,
    /// The last day of the season.
    pub season_end_date: Option<&'a str>,
    /// Its account under control 0306.
    pub account_id: i64,
    /// A note.
    pub note: Option<&'a str>,
}

/// Inserts a garden (open) and returns its id.
///
/// # Errors
/// Any SQLite error (the CHECKs of `ventures`).
pub fn insert(
    conn: &Connection,
    garden: &NewGarden<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO ventures (created_at, created_by, updated_at, updated_by, kind, number, entry_date,
             name_ur, name_en, search_ur, search_en, search_phonetic, item_id, rent_paisa,
             season_end_date, account_id, note)
         VALUES (?1, ?2, ?1, ?2, 'garden', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            now,
            user_id,
            garden.number,
            garden.entry_date,
            garden.name_ur,
            garden.name_en,
            garden.search_ur,
            garden.search_en,
            garden.search_phonetic,
            garden.item_id,
            garden.rent_paisa,
            garden.season_end_date,
            garden.account_id,
            garden.note,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// A garden as the services read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GardenRec {
    /// Row id (`ventures.id`).
    pub id: i64,
    /// Version.
    pub version: i64,
    /// Number.
    pub number: i64,
    /// The day made, `YYYY-MM-DD`.
    pub entry_date: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The main fruit.
    pub item: Option<RecordRef>,
    /// The rent, paisa.
    pub rent_paisa: i64,
    /// The last day of the season.
    pub season_end_date: Option<String>,
    /// A note.
    pub note: Option<String>,
    /// `open` or `closed`.
    pub status: String,
    /// The day closed, `YYYY-MM-DD`.
    pub closed_date: Option<String>,
    /// The journal entry of the closing, if it posted one.
    pub close_entry_id: Option<i64>,
    /// Its account (under control 0306).
    pub account: RecordRef,
}

const COLUMNS: &str = "v.id, v.version, v.number, v.entry_date, v.name_ur, v.name_en,
    it.id, it.code, it.name_ur, it.name_en, v.rent_paisa, v.season_end_date, v.note, v.status,
    v.closed_date, v.close_entry_id, a.id, a.code, a.name_ur, a.name_en";

const FROM: &str = "FROM ventures v JOIN accounts a ON a.id = v.account_id
    LEFT JOIN items it ON it.id = v.item_id";

fn read(r: &Row<'_>) -> rusqlite::Result<GardenRec> {
    let item_id: Option<i64> = r.get(6)?;
    let item = match item_id {
        Some(id) => Some(RecordRef {
            id,
            code: r.get(7)?,
            name_ur: r.get(8)?,
            name_en: r.get(9)?,
        }),
        None => None,
    };
    Ok(GardenRec {
        id: r.get(0)?,
        version: r.get(1)?,
        number: r.get(2)?,
        entry_date: r.get(3)?,
        name_ur: r.get(4)?,
        name_en: r.get(5)?,
        item,
        rent_paisa: r.get(10)?,
        season_end_date: r.get(11)?,
        note: r.get(12)?,
        status: r.get(13)?,
        closed_date: r.get(14)?,
        close_entry_id: r.get(15)?,
        account: RecordRef {
            id: r.get(16)?,
            code: r.get(17)?,
            name_ur: r.get(18)?,
            name_en: r.get(19)?,
        },
    })
}

/// One garden by id (a trip is not found here).
///
/// # Errors
/// Any SQLite error.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<GardenRec>> {
    conn.query_row(
        &format!("SELECT {COLUMNS} {FROM} WHERE v.id = ?1 AND v.kind = 'garden'"),
        [id],
        read,
    )
    .optional()
}

/// The gardens with these ids, in the order of `ids`.
///
/// # Errors
/// Any SQLite error.
pub fn get_many(conn: &Connection, ids: &[i64]) -> rusqlite::Result<Vec<GardenRec>> {
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(rec) = get(conn, *id)? {
            out.push(rec);
        }
    }
    Ok(out)
}

/// The garden whose account this is: its id and status. `None` if the account belongs to no garden.
///
/// # Errors
/// Any SQLite error.
pub fn of_account(conn: &Connection, account_id: i64) -> rusqlite::Result<Option<(i64, String)>> {
    conn.query_row(
        "SELECT id, status FROM ventures WHERE kind = 'garden' AND account_id = ?1",
        [account_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
}

/// Whether this account is the account of a closed garden.
///
/// # Errors
/// Any SQLite error.
pub fn account_is_closed(conn: &Connection, account_id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM ventures WHERE kind = 'garden' AND account_id = ?1
                        AND status = 'closed')",
        [account_id],
        |r| r.get(0),
    )
}

/// True once a garden's shares are fixed: it has an entry of its own (an installment, a cost, a sale
/// in another shop), or its account has a posted line, for example the seller credit of a بکری lot
/// sold into it (PRD FR-12.29, E66, Umar 25-10-2026).
///
/// # Errors
/// Any SQLite error.
pub fn shares_locked(conn: &Connection, garden_id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM venture_entries WHERE venture_id = ?1)
             OR EXISTS (SELECT 1 FROM journal_lines l JOIN ventures v ON v.account_id = l.account_id
                        WHERE v.id = ?1)",
        [garden_id],
        |r| r.get(0),
    )
}

/// Whether a بکری has a lot whose seller is the account of a closed garden (PRD FR-12.38): such a
/// بکری cannot be edited or cancelled until the closing is undone.
///
/// # Errors
/// Any SQLite error.
pub fn bikri_has_closed_garden_lot(conn: &Connection, bikri_id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM bikri_lots l
                        JOIN ventures v ON v.account_id = l.seller_account_id
                        WHERE l.bikri_id = ?1 AND v.kind = 'garden' AND v.status = 'closed')",
        [bikri_id],
        |r| r.get(0),
    )
}

/// Closes a garden: its status, the day and the journal entry of the closing (null when there was
/// nothing to post).
///
/// # Errors
/// Any SQLite error.
pub fn set_closed(
    conn: &Connection,
    id: i64,
    closed_date: &str,
    close_entry_id: Option<i64>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ventures SET status = 'closed', closed_date = ?2, close_entry_id = ?3,
             updated_at = ?4, updated_by = ?5, version = version + 1 WHERE id = ?1",
        params![id, closed_date, close_entry_id, now, user_id],
    )?;
    Ok(())
}

/// Opens a closed garden again (the undo of its closing).
///
/// # Errors
/// Any SQLite error.
pub fn set_open(conn: &Connection, id: i64, user_id: i64, now: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ventures SET status = 'open', closed_date = NULL, close_entry_id = NULL,
             updated_at = ?2, updated_by = ?3, version = version + 1 WHERE id = ?1",
        params![id, now, user_id],
    )?;
    Ok(())
}

/// The sums a garden's figures are made of, all read from stored rows. Money is paisa.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Figures {
    /// Σ live installments.
    pub installments_paisa: i64,
    /// Σ live running costs.
    pub costs_paisa: i64,
    /// Σ parts of live payments paid from the shop's cash.
    pub paid_shop_paisa: i64,
    /// Σ parts of live payments paid from pockets.
    pub paid_pocket_paisa: i64,
    /// Σ live sales in other shops.
    pub other_shop_paisa: i64,
    /// Σ credits of the lines on its account, the closing and its undo left out.
    pub credits_paisa: i64,
    /// Σ debits of the same lines.
    pub debits_paisa: i64,
    /// Σ credits of every line on its account.
    pub account_credits_paisa: i64,
    /// Σ debits of every line on its account.
    pub account_debits_paisa: i64,
    /// The latest day of a live entry or of a line on its account (the closing and its undo left
    /// out), and never before the day the garden was made. `YYYY-MM-DD`.
    pub last_date: Option<String>,
}

/// The figures of one garden.
///
/// # Errors
/// Any SQLite error.
pub fn figures(conn: &Connection, garden: &GardenRec) -> rusqlite::Result<Figures> {
    let (installments, costs, other_shop): (i64, i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(CASE WHEN kind = 'installment' THEN amount_paisa END), 0),
                COALESCE(SUM(CASE WHEN kind = 'cost' THEN amount_paisa END), 0),
                COALESCE(SUM(CASE WHEN kind = 'other_sale' THEN amount_paisa END), 0)
         FROM venture_entries WHERE venture_id = ?1 AND status = 'saved'",
        [garden.id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    // What he paid is read by the kind of the entry, never by the source alone: the closing parts
    // carry a source too.
    let (paid_shop, paid_pocket): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(CASE WHEN p.source = 'shop' THEN p.amount_paisa END), 0),
                COALESCE(SUM(CASE WHEN p.source = 'pocket' THEN p.amount_paisa END), 0)
         FROM venture_entry_parts p JOIN venture_entries x ON x.id = p.entry_id
         WHERE x.venture_id = ?1 AND x.status = 'saved' AND x.kind IN ('installment', 'cost')",
        [garden.id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let (credits, debits): (i64, i64) = conn.query_row(
        &format!(
            "SELECT COALESCE(SUM(l.cr_paisa), 0), COALESCE(SUM(l.dr_paisa), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE l.account_id = ?1 AND e.status = 'posted' AND {NOT_A_CLOSING}"
        ),
        [garden.account.id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let (account_credits, account_debits): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(l.cr_paisa), 0), COALESCE(SUM(l.dr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND e.status = 'posted'",
        [garden.account.id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let last_date: Option<String> = conn.query_row(
        &format!(
            "SELECT MAX(d) FROM (
                SELECT e.entry_date AS d FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                 WHERE l.account_id = ?1 AND e.status = 'posted' AND {NOT_A_CLOSING}
                UNION ALL
                SELECT x.entry_date FROM venture_entries x
                 WHERE x.venture_id = ?2 AND x.status = 'saved' AND x.kind IN ('installment', 'cost', 'other_sale')
                UNION ALL SELECT ?3)"
        ),
        params![garden.account.id, garden.id, garden.entry_date],
        |r| r.get(0),
    )?;
    Ok(Figures {
        installments_paisa: installments,
        costs_paisa: costs,
        paid_shop_paisa: paid_shop,
        paid_pocket_paisa: paid_pocket,
        other_shop_paisa: other_shop,
        credits_paisa: credits,
        debits_paisa: debits,
        account_credits_paisa: account_credits,
        account_debits_paisa: account_debits,
        last_date,
    })
}

/// What one journal entry did to a garden's account, the closing and its undo left out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountEvent {
    /// The journal entry.
    pub entry_id: i64,
    /// Its day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// What kind of record it belongs to (`bikri`, `venture`, `fruit`, ...).
    pub source_type: String,
    /// The id of that record.
    pub source_id: i64,
    /// Σ credits of its lines on the account.
    pub cr_paisa: i64,
    /// Σ debits of its lines on the account.
    pub dr_paisa: i64,
}

/// The journal entries that touched a garden's account, oldest entry first, the closing and its undo
/// left out. One row for each entry: a بکری with two lots of the garden is one row.
///
/// # Errors
/// Any SQLite error.
pub fn account_events(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<AccountEvent>> {
    let mut statement = conn.prepare(&format!(
        "SELECT e.id, e.entry_date, e.source_type, e.source_id,
                COALESCE(SUM(l.cr_paisa), 0), COALESCE(SUM(l.dr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND e.status = 'posted' AND {NOT_A_CLOSING}
         GROUP BY e.id ORDER BY e.id"
    ))?;
    let rows = statement.query_map([account_id], |r| {
        Ok(AccountEvent {
            entry_id: r.get(0)?,
            entry_date: r.get(1)?,
            source_type: r.get(2)?,
            source_id: r.get(3)?,
            cr_paisa: r.get(4)?,
            dr_paisa: r.get(5)?,
        })
    })?;
    rows.collect()
}

/// The saved closing entry of a garden, if it is closed: its `venture_entries` row id.
///
/// # Errors
/// Any SQLite error.
pub fn saved_close_entry(conn: &Connection, garden_id: i64) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT id FROM venture_entries WHERE venture_id = ?1 AND kind = 'close' AND status = 'saved'
         ORDER BY id DESC LIMIT 1",
        [garden_id],
        |r| r.get(0),
    )
    .optional()
}

/// The open gardens a partner is in, with his share: the data under his card.
/// Each row is `(garden id, share_bps)`.
///
/// # Errors
/// Any SQLite error.
pub fn open_gardens_of_partner(
    conn: &Connection,
    partner_id: i64,
) -> rusqlite::Result<Vec<(i64, i64)>> {
    let mut statement = conn.prepare(
        "SELECT v.id, vp.share_bps FROM venture_partners vp JOIN ventures v ON v.id = vp.venture_id
         WHERE vp.partner_id = ?1 AND v.kind = 'garden' AND v.status = 'open' ORDER BY v.number",
    )?;
    let rows = statement.query_map([partner_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

/// What each partner paid into one garden, from the parts of its live installments and costs: for
/// each partner id, `(shop, pocket)` in paisa.
///
/// # Errors
/// Any SQLite error.
pub fn paid_by_partner(
    conn: &Connection,
    garden_id: i64,
) -> rusqlite::Result<HashMap<i64, (i64, i64)>> {
    let mut statement = conn.prepare(
        "SELECT p.partner_id,
                COALESCE(SUM(CASE WHEN p.source = 'shop' THEN p.amount_paisa END), 0),
                COALESCE(SUM(CASE WHEN p.source = 'pocket' THEN p.amount_paisa END), 0)
         FROM venture_entry_parts p JOIN venture_entries x ON x.id = p.entry_id
         WHERE x.venture_id = ?1 AND x.status = 'saved' AND x.kind IN ('installment', 'cost')
         GROUP BY p.partner_id",
    )?;
    let rows = statement.query_map([garden_id], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))?;
    rows.collect()
}
