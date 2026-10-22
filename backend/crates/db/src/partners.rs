//! Partners, their share history and their share rows in SQL (plan 11, PRD 17.1 to 17.3, FR-6.5).
//!
//! No rules here: the split is worked out in `mandi_domain::partners`, and the one place that
//! decides when rows are written is `services::ledger::post`. Money is summed only with `SUM()`
//! on INTEGER columns (money agent Section 9); every other calculation is done in Rust.
//!
//! The tables are `partners`, `partner_shares` (history) and `partner_share_rows` (append-only,
//! Code Standards B17: triggers refuse UPDATE and DELETE).

use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension, Row, params, params_from_iter};

/// A partner as stored, with the codes of his two accounts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartnerRecord {
    /// Row id. A lower id means created first (PRD C10 tie rule).
    pub id: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// His capital account.
    pub capital_account_id: i64,
    /// Its code.
    pub capital_account_code: String,
    /// His drawings account.
    pub drawings_account_id: i64,
    /// Its code.
    pub drawings_account_code: String,
    /// His gardens, trips and fruit account (control 0604, PRD FR-10.4 v1.6). 0 only for a partner
    /// made without one, which the partner save never does.
    pub venture_account_id: i64,
    /// Its code (empty if he has none).
    pub venture_account_code: String,
    /// Active.
    pub is_active: bool,
    /// Version.
    pub version: i64,
}

/// A partner to insert.
#[derive(Debug, Clone, Copy)]
pub struct NewPartner<'a> {
    /// Urdu name.
    pub name_ur: &'a str,
    /// English name.
    pub name_en: &'a str,
    /// His capital account (already made).
    pub capital_account_id: i64,
    /// His drawings account (already made).
    pub drawings_account_id: i64,
    /// His gardens, trips and fruit account (already made, control 0604).
    pub venture_account_id: i64,
}

const PARTNER_SELECT: &str = "SELECT p.id, p.name_ur, p.name_en, p.capital_account_id, ca.code,
        p.drawings_account_id, da.code, COALESCE(p.venture_account_id, 0), COALESCE(va.code, ''),
        p.is_active, p.version
     FROM partners p
     JOIN accounts ca ON ca.id = p.capital_account_id
     JOIN accounts da ON da.id = p.drawings_account_id
     LEFT JOIN accounts va ON va.id = p.venture_account_id";

fn partner_from(r: &Row<'_>) -> rusqlite::Result<PartnerRecord> {
    Ok(PartnerRecord {
        id: r.get(0)?,
        name_ur: r.get(1)?,
        name_en: r.get(2)?,
        capital_account_id: r.get(3)?,
        capital_account_code: r.get(4)?,
        drawings_account_id: r.get(5)?,
        drawings_account_code: r.get(6)?,
        venture_account_id: r.get(7)?,
        venture_account_code: r.get(8)?,
        is_active: r.get(9)?,
        version: r.get(10)?,
    })
}

/// Every partner: active ones first, then by id (the partner created first comes first).
///
/// # Errors
/// Any SQLite error.
pub fn list(conn: &Connection) -> rusqlite::Result<Vec<PartnerRecord>> {
    let mut statement =
        conn.prepare(&format!("{PARTNER_SELECT} ORDER BY p.is_active DESC, p.id"))?;
    let rows = statement.query_map([], partner_from)?;
    rows.collect()
}

/// One partner, or `None`.
///
/// # Errors
/// Any SQLite error.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<PartnerRecord>> {
    conn.query_row(
        &format!("{PARTNER_SELECT} WHERE p.id = ?1"),
        [id],
        partner_from,
    )
    .optional()
}

/// How many partners are active.
///
/// # Errors
/// Any SQLite error.
pub fn active_count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM partners WHERE is_active = 1",
        [],
        |r| r.get(0),
    )
}

/// True if the partner is in a garden or a trip that is still open (PRD A63: he cannot be made
/// inactive then).
///
/// # Errors
/// Any SQLite error.
pub fn in_open_venture(conn: &Connection, partner_id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM venture_partners vp JOIN ventures v ON v.id = vp.venture_id
                        WHERE vp.partner_id = ?1 AND v.status = 'open')",
        [partner_id],
        |r| r.get(0),
    )
}

/// Inserts a partner and returns his id. The partner has no share until `insert_share`.
///
/// # Errors
/// Any SQLite error.
pub fn insert(
    conn: &Connection,
    partner: &NewPartner<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO partners (created_at, created_by, updated_at, updated_by, name_ur, name_en,
             capital_account_id, drawings_account_id, venture_account_id, is_active)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
        params![
            now,
            user_id,
            partner.name_ur,
            partner.name_en,
            partner.capital_account_id,
            partner.drawings_account_id,
            partner.venture_account_id
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Changes a partner's names and active flag if `version` is still the stored one. Returns
/// false when the version has moved on.
///
/// # Errors
/// Any SQLite error.
#[allow(clippy::too_many_arguments)]
// Seven plain values of one UPDATE; a struct would only repeat the same fields.
pub fn update(
    conn: &Connection,
    id: i64,
    version: i64,
    name_ur: &str,
    name_en: &str,
    is_active: bool,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE partners SET name_ur = ?1, name_en = ?2, is_active = ?3, updated_at = ?4,
             updated_by = ?5, version = version + 1
         WHERE id = ?6 AND version = ?7",
        params![name_ur, name_en, is_active, now, user_id, id, version],
    )?;
    Ok(changed == 1)
}

/// One entry of a partner's share history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareRecord {
    /// The share in bps.
    pub share_bps: i64,
    /// The first day it applies, `YYYY-MM-DD`.
    pub effective_from: String,
}

/// A partner's share history, oldest first (same day: the row saved first).
///
/// # Errors
/// Any SQLite error.
pub fn share_history(conn: &Connection, partner_id: i64) -> rusqlite::Result<Vec<ShareRecord>> {
    let mut statement = conn.prepare(
        "SELECT share_bps, effective_from FROM partner_shares WHERE partner_id = ?1
         ORDER BY effective_from, id",
    )?;
    let rows = statement.query_map([partner_id], |r| {
        Ok(ShareRecord {
            share_bps: r.get(0)?,
            effective_from: r.get(1)?,
        })
    })?;
    rows.collect()
}

/// Adds one row to a partner's share history (PRD FR-17.2: history is kept, never changed).
///
/// # Errors
/// Any SQLite error.
pub fn insert_share(
    conn: &Connection,
    partner_id: i64,
    share_bps: i64,
    effective_from: &str,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO partner_shares (created_at, created_by, updated_at, updated_by, partner_id,
             share_bps, effective_from)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5)",
        params![now, user_id, partner_id, share_bps, effective_from],
    )?;
    Ok(())
}

/// The share of every partner on a day: for each partner his latest history row with
/// `effective_from` on or before the day; on the same day, the row saved last (PRD FR-17.2).
/// Partners without such a row are not listed. Inactive partners are listed too, because an
/// open day before his share was set to 0 still splits with him. By partner id.
///
/// # Errors
/// Any SQLite error.
pub fn shares_on(conn: &Connection, date: &str) -> rusqlite::Result<Vec<(i64, i64)>> {
    let mut statement = conn.prepare(
        "SELECT s.partner_id, s.share_bps FROM partner_shares s
         WHERE s.id = (SELECT s2.id FROM partner_shares s2
                       WHERE s2.partner_id = s.partner_id AND s2.effective_from <= ?1
                       ORDER BY s2.effective_from DESC, s2.id DESC LIMIT 1)
         ORDER BY s.partner_id",
    )?;
    let rows = statement.query_map([date], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

/// A share row to insert (append-only, Code Standards B17).
#[derive(Debug, Clone, Copy)]
pub struct NewShareRow<'a> {
    /// The posted entry it belongs to.
    pub journal_entry_id: i64,
    /// The entry's date, `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// The partner.
    pub partner_id: i64,
    /// `income` or `expense`.
    pub kind: &'a str,
    /// The signed base that was split, paisa.
    pub base_paisa: i64,
    /// The partner's share on that date, bps.
    pub share_bps: i64,
    /// The partner's signed part, paisa.
    pub amount_paisa: i64,
}

/// Inserts one share row.
///
/// # Errors
/// Any SQLite error (the CHECKs of `partner_share_rows`).
pub fn insert_share_row(
    conn: &Connection,
    row: &NewShareRow<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO partner_share_rows (created_at, created_by, updated_at, updated_by,
             journal_entry_id, entry_date, partner_id, kind, base_paisa, share_bps, amount_paisa)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            now,
            user_id,
            row.journal_entry_id,
            row.entry_date,
            row.partner_id,
            row.kind,
            row.base_paisa,
            row.share_bps,
            row.amount_paisa
        ],
    )?;
    Ok(())
}

/// A share row as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredShareRow {
    /// The partner.
    pub partner_id: i64,
    /// `income` or `expense`.
    pub kind: String,
    /// The signed base, paisa.
    pub base_paisa: i64,
    /// The share in bps.
    pub share_bps: i64,
    /// The signed part, paisa.
    pub amount_paisa: i64,
}

/// The share rows of one entry, in the order they were written.
///
/// # Errors
/// Any SQLite error.
pub fn share_rows_of_entry(
    conn: &Connection,
    entry_id: i64,
) -> rusqlite::Result<Vec<StoredShareRow>> {
    let mut statement = conn.prepare(
        "SELECT partner_id, kind, base_paisa, share_bps, amount_paisa FROM partner_share_rows
         WHERE journal_entry_id = ?1 ORDER BY id",
    )?;
    let rows = statement.query_map([entry_id], |r| {
        Ok(StoredShareRow {
            partner_id: r.get(0)?,
            kind: r.get(1)?,
            base_paisa: r.get(2)?,
            share_bps: r.get(3)?,
            amount_paisa: r.get(4)?,
        })
    })?;
    rows.collect()
}

/// The head code (`01` to `06`) of each account in `ids`. One query for the whole entry.
///
/// # Errors
/// Any SQLite error.
pub fn head_codes_of(conn: &Connection, ids: &[i64]) -> rusqlite::Result<HashMap<i64, String>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let marks = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT a.id, h.code FROM accounts a
         JOIN account_controls c ON c.id = a.control_id
         JOIN account_heads h ON h.id = c.head_id
         WHERE a.id IN ({marks})"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(ids.iter()), |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
    })?;
    rows.collect()
}

/// One partner's total of one kind of share rows in a date range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareSum {
    /// The partner.
    pub partner_id: i64,
    /// `income` or `expense`.
    pub kind: String,
    /// `SUM(amount_paisa)` of the rows in the range.
    pub total_paisa: i64,
}

/// The sum of share rows per partner and kind with `entry_date` from `from` to `to`, both
/// included (`YYYY-MM-DD`).
///
/// # Errors
/// Any SQLite error.
pub fn share_sums(conn: &Connection, from: &str, to: &str) -> rusqlite::Result<Vec<ShareSum>> {
    let mut statement = conn.prepare(
        "SELECT partner_id, kind, COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows
         WHERE entry_date >= ?1 AND entry_date <= ?2
         GROUP BY partner_id, kind ORDER BY partner_id, kind",
    )?;
    let rows = statement.query_map(params![from, to], |r| {
        Ok(ShareSum {
            partner_id: r.get(0)?,
            kind: r.get(1)?,
            total_paisa: r.get(2)?,
        })
    })?;
    rows.collect()
}

/// The two sums behind "money taken" for one account (PRD FR-16.2, FR-15.3, E56).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TakenSums {
    /// `SUM(dr_paisa)` over the lines of ordinary (not reversal) entries.
    pub debits: i64,
    /// `SUM(cr_paisa)` over the lines of reversal entries: the takes that were cancelled or
    /// edited. Money taken is `debits - reversed_credits`, subtracted in Rust with checked maths.
    pub reversed_credits: i64,
}

/// The sums behind the money taken, per account, in a date range. It is asked for the drawings
/// account (ذاتی خرچہ) of each partner. A reversal has the date of the entry it reverses, so a
/// take cancelled or edited later leaves the month it was made in. Credits of ordinary entries
/// (money returned) are not counted here; the returned column comes in plan 17. One query for all
/// accounts; an account with nothing in the range is not in the answer. Only `SUM()` on INTEGER
/// columns is used.
///
/// # Errors
/// Any SQLite error.
pub fn taken_between(
    conn: &Connection,
    account_ids: &[i64],
    from: &str,
    to: &str,
) -> rusqlite::Result<HashMap<i64, TakenSums>> {
    if account_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let marks = vec!["?"; account_ids.len()].join(",");
    let sql = format!(
        "SELECT l.account_id,
                COALESCE(SUM(CASE WHEN e.reverses_entry_id IS NULL THEN l.dr_paisa ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN e.reverses_entry_id IS NOT NULL THEN l.cr_paisa ELSE 0 END), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE e.status = 'posted' AND e.entry_date >= ? AND e.entry_date <= ?
           AND l.account_id IN ({marks})
         GROUP BY l.account_id"
    );
    let mut statement = conn.prepare(&sql)?;
    let dates = [from.to_owned(), to.to_owned()];
    let mut values: Vec<rusqlite::types::Value> = dates
        .iter()
        .map(|d| rusqlite::types::Value::Text(d.clone()))
        .collect();
    values.extend(
        account_ids
            .iter()
            .map(|id| rusqlite::types::Value::Integer(*id)),
    );
    let rows = statement.query_map(params_from_iter(values.iter()), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            TakenSums {
                debits: r.get(1)?,
                reversed_credits: r.get(2)?,
            },
        ))
    })?;
    rows.collect()
}

/// The date (`YYYY-MM-DD`) and the posting type of an entry, or `None` if there is no such entry.
///
/// # Errors
/// Any SQLite error.
pub fn entry_date_and_type(
    conn: &Connection,
    entry_id: i64,
) -> rusqlite::Result<Option<(String, String)>> {
    conn.query_row(
        "SELECT entry_date, posting_type FROM journal_entries WHERE id = ?1",
        [entry_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
}
