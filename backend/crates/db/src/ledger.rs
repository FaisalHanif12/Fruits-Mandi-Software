//! The journal in SQL (plan 07, PRD FR-6.2, FR-6.3, NFR-SEC-6): drafts, lines, the one UPDATE
//! that posts an entry with its total and its keyed hash, balances and the chain check.
//!
//! No rules here: the entry was built and checked in `mandi_domain::ledger`, and the database
//! triggers are the second guard. `services::ledger` is the only caller that writes.
//!
//! The hash chain (NFR-SEC-6): `chain_hash = blake3::keyed_hash(key, canonical bytes)` where the
//! bytes hold the previous posted entry's hash, the entry (id, date, types, source, reversed
//! entry, note, total) and every line in `line_no` order. Every field is written with a type
//! byte and a length, so NULL and an empty text differ and no two entries give the same bytes.
//! [`post`] and [`verify_journal_chain`] use the one function [`canonical_bytes`] on rows read
//! from the database.

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::audit::{AuditKey, GENESIS_HASH};
use crate::error::DbError;

/// The head of an entry to insert as a draft.
#[derive(Debug, Clone, Copy)]
pub struct EntryHeader<'a> {
    /// `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// One of the `posting_type` values of Architecture 15.7.
    pub posting_type: &'a str,
    /// `bikri`, `voucher`, `account` or `month_close`.
    pub source_type: &'a str,
    /// The id of the source record.
    pub source_id: i64,
    /// The entry this one reverses, if it is a reversal.
    pub reverses_entry_id: Option<i64>,
    /// A note.
    pub narration: Option<&'a str>,
}

/// One line to insert. Exactly one of `dr_paisa` and `cr_paisa` is more than 0.
#[derive(Debug, Clone, Copy)]
pub struct NewLine<'a> {
    /// Line number from 1.
    pub line_no: i64,
    /// The account.
    pub account_id: i64,
    /// The person inside a پختہ کھاتہ account.
    pub person_key: Option<&'a str>,
    /// Debit in paisa.
    pub dr_paisa: i64,
    /// Credit in paisa.
    pub cr_paisa: i64,
    /// A seller debit that counts as an advance.
    pub is_advance: bool,
    /// A note.
    pub narration: Option<&'a str>,
}

/// A line read back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredLine {
    /// Row id.
    pub id: i64,
    /// Line number.
    pub line_no: i64,
    /// The account.
    pub account_id: i64,
    /// The person key.
    pub person_key: Option<String>,
    /// Debit in paisa.
    pub dr_paisa: i64,
    /// Credit in paisa.
    pub cr_paisa: i64,
    /// Advance flag.
    pub is_advance: bool,
    /// Note.
    pub narration: Option<String>,
}

/// An entry read back with its lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEntry {
    /// Row id.
    pub id: i64,
    /// `YYYY-MM-DD`.
    pub entry_date: String,
    /// Posting type.
    pub posting_type: String,
    /// Source type.
    pub source_type: String,
    /// Source id.
    pub source_id: i64,
    /// The reversed entry, if any.
    pub reverses_entry_id: Option<i64>,
    /// Note.
    pub narration: Option<String>,
    /// `draft` or `posted`.
    pub status: String,
    /// Total in paisa (0 for a draft).
    pub total_paisa: i64,
    /// The hash, once posted.
    pub chain_hash: Option<String>,
    /// The lines in `line_no` order.
    pub lines: Vec<StoredLine>,
}

/// Inserts an entry as a draft and returns its id.
///
/// # Errors
/// Any SQLite error.
pub fn insert_draft(
    conn: &Connection,
    header: &EntryHeader<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO journal_entries (created_at, created_by, updated_at, updated_by, entry_date, posting_type,
             source_type, source_id, reverses_entry_id, narration, status, total_paisa)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'draft', 0)",
        params![
            now,
            user_id,
            header.entry_date,
            header.posting_type,
            header.source_type,
            header.source_id,
            header.reverses_entry_id,
            header.narration
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Inserts one line into a draft entry.
///
/// # Errors
/// Any SQLite error (the CHECKs and triggers of `journal_lines`).
pub fn insert_line(
    conn: &Connection,
    entry_id: i64,
    line: &NewLine<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO journal_lines (created_at, created_by, updated_at, updated_by, entry_id, line_no, account_id,
             pukhta_person_key, dr_paisa, cr_paisa, is_advance, narration)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            now,
            user_id,
            entry_id,
            line.line_no,
            line.account_id,
            line.person_key,
            line.dr_paisa,
            line.cr_paisa,
            line.is_advance,
            line.narration
        ],
    )?;
    Ok(())
}

/// One entry with its lines, or `None`.
///
/// # Errors
/// Any SQLite error.
pub fn get_entry(conn: &Connection, entry_id: i64) -> rusqlite::Result<Option<StoredEntry>> {
    let head = conn
        .query_row(
            "SELECT id, entry_date, posting_type, source_type, source_id, reverses_entry_id, narration, status,
                    total_paisa, chain_hash
             FROM journal_entries WHERE id = ?1",
            [entry_id],
            |r| {
                Ok(StoredEntry {
                    id: r.get(0)?,
                    entry_date: r.get(1)?,
                    posting_type: r.get(2)?,
                    source_type: r.get(3)?,
                    source_id: r.get(4)?,
                    reverses_entry_id: r.get(5)?,
                    narration: r.get(6)?,
                    status: r.get(7)?,
                    total_paisa: r.get(8)?,
                    chain_hash: r.get(9)?,
                    lines: Vec::new(),
                })
            },
        )
        .optional()?;
    let Some(mut entry) = head else {
        return Ok(None);
    };
    let mut statement = conn.prepare(
        "SELECT id, line_no, account_id, pukhta_person_key, dr_paisa, cr_paisa, is_advance, narration
         FROM journal_lines WHERE entry_id = ?1 ORDER BY line_no",
    )?;
    let lines = statement.query_map([entry_id], |r| {
        Ok(StoredLine {
            id: r.get(0)?,
            line_no: r.get(1)?,
            account_id: r.get(2)?,
            person_key: r.get(3)?,
            dr_paisa: r.get(4)?,
            cr_paisa: r.get(5)?,
            is_advance: r.get(6)?,
            narration: r.get(7)?,
        })
    })?;
    entry.lines = lines.collect::<Result<Vec<_>, _>>()?;
    Ok(Some(entry))
}

fn put_text(out: &mut Vec<u8>, text: Option<&str>) {
    match text {
        None => out.push(0),
        Some(text) => {
            out.push(1);
            out.extend_from_slice(&(text.len() as u64).to_le_bytes());
            out.extend_from_slice(text.as_bytes());
        }
    }
}

fn put_number(out: &mut Vec<u8>, number: Option<i64>) {
    put_text(out, number.map(|n| n.to_string()).as_deref());
}

/// The bytes that are hashed: the previous hash, then the entry and its lines. `total_paisa` is
/// passed in because at post time the row still says 0.
fn canonical_bytes(previous: &str, entry: &StoredEntry, total_paisa: i64) -> Vec<u8> {
    let mut out = Vec::new();
    put_text(&mut out, Some(previous));
    put_number(&mut out, Some(entry.id));
    put_text(&mut out, Some(&entry.entry_date));
    put_text(&mut out, Some(&entry.posting_type));
    put_text(&mut out, Some(&entry.source_type));
    put_number(&mut out, Some(entry.source_id));
    put_number(&mut out, entry.reverses_entry_id);
    put_text(&mut out, entry.narration.as_deref());
    put_number(&mut out, Some(total_paisa));
    put_number(&mut out, i64::try_from(entry.lines.len()).ok());
    for line in &entry.lines {
        put_number(&mut out, Some(line.line_no));
        put_number(&mut out, Some(line.account_id));
        put_text(&mut out, line.person_key.as_deref());
        put_number(&mut out, Some(line.dr_paisa));
        put_number(&mut out, Some(line.cr_paisa));
        put_number(&mut out, Some(i64::from(line.is_advance)));
        put_text(&mut out, line.narration.as_deref());
    }
    out
}

fn hash(key: &AuditKey, bytes: &[u8]) -> String {
    blake3::keyed_hash(key.as_bytes(), bytes)
        .to_hex()
        .to_string()
}

/// The hash of the last posted entry (the highest id), or the genesis hash. Also its id.
fn last_posted(conn: &Connection) -> rusqlite::Result<(i64, String)> {
    let found: Option<(i64, Option<String>)> = conn
        .query_row(
            "SELECT id, chain_hash FROM journal_entries WHERE status = 'posted' ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    Ok(match found {
        Some((id, Some(hash))) => (id, hash),
        // A posted entry without a hash is read as a break by `verify_journal_chain`; the
        // next entry then starts from the genesis value so posting itself does not stop.
        Some((id, None)) => (id, GENESIS_HASH.to_owned()),
        None => (0, GENESIS_HASH.to_owned()),
    })
}

/// Posts a draft entry: ONE update that sets the status, the total and the chain hash, so the
/// balance trigger runs on it (Code Standards B16). The entry and its lines are read back from
/// the database for the hash. Returns the hash.
///
/// # Errors
/// `DbError::JournalUnbalanced` if the trigger refuses it; `DbError::Constraint` if a posted
/// entry with a higher id exists (the chain must follow the ids); any other database error.
pub fn post(
    conn: &Connection,
    key: &AuditKey,
    entry_id: i64,
    total_paisa: i64,
    user_id: i64,
    now: &str,
) -> Result<String, DbError> {
    let entry = get_entry(conn, entry_id)?
        .ok_or_else(|| DbError::Constraint(format!("journal entry {entry_id} is missing")))?;
    let (last_id, previous) = last_posted(conn)?;
    if last_id > entry_id {
        return Err(DbError::Constraint(format!(
            "entry {entry_id} would be posted after entry {last_id}"
        )));
    }
    let chain_hash = hash(key, &canonical_bytes(&previous, &entry, total_paisa));
    conn.execute(
        "UPDATE journal_entries SET status = 'posted', total_paisa = ?1, chain_hash = ?2,
             updated_at = ?3, updated_by = ?4, version = version + 1 WHERE id = ?5",
        params![total_paisa, chain_hash, now, user_id, entry_id],
    )?;
    set_head(conn, entry_id, &chain_hash, user_id, now)?;
    Ok(chain_hash)
}

/// The settings key that holds the head of the journal chain.
const HEAD_KEY: &str = "journal_head";

/// Records the last posted entry (its id and hash) in the same transaction as the post. The hash
/// chain alone cannot see the newest entry being removed or turned back into a draft, because
/// no later entry points at it; this record can. It is an internal integrity note, not a user
/// setting. Someone who also rewrites this record by hand can hide a removed last entry; that is
/// the limit of any check kept inside the same file (the daily backup copy is the outside one).
fn set_head(
    conn: &Connection,
    entry_id: i64,
    chain_hash: &str,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    let value = format!("{{\"id\":{entry_id},\"hash\":\"{chain_hash}\"}}");
    conn.execute(
        "INSERT INTO settings (key, value_json, created_by, updated_by) VALUES (?1, ?2, ?3, ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
             updated_at = ?4, updated_by = ?3, version = version + 1",
        params![HEAD_KEY, value, user_id, now],
    )?;
    Ok(())
}

/// The recorded head: the id and hash of the last posted entry, if one was recorded.
fn read_head(conn: &Connection) -> rusqlite::Result<Option<(i64, String)>> {
    let text: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [HEAD_KEY],
            |r| r.get(0),
        )
        .optional()?;
    Ok(text.and_then(|text| {
        let value: serde_json::Value = serde_json::from_str(&text).ok()?;
        Some((
            value.get("id")?.as_i64()?,
            value.get("hash")?.as_str()?.to_owned(),
        ))
    }))
}

/// The first entry whose hash does not match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("journal chain broken at entry {entry_id}")]
pub struct JournalChainBreak {
    /// The id of the first entry that does not match. If an entry was removed, this is the
    /// entry after the gap.
    pub entry_id: i64,
}

/// Why [`verify_journal_chain`] did not say "good".
#[derive(Debug, Error)]
pub enum JournalVerifyError {
    /// An entry or line was changed, an entry was removed, or a posted entry has no hash.
    #[error(transparent)]
    Broken(#[from] JournalChainBreak),
    /// The journal could not be read.
    #[error(transparent)]
    Db(#[from] DbError),
}

/// Walks every posted entry in id order and checks its hash (NFR-SEC-6). A posted entry with no
/// hash counts as changed. A draft entry found at rest counts as changed too: a draft never
/// outlives the save that made it, so one that is still there means a posted entry was turned
/// back. The last posted entry must also match the recorded head, so the newest entry cannot be
/// removed or turned back without a trace.
///
/// # Errors
/// `JournalVerifyError::Broken` with the first bad entry, or `JournalVerifyError::Db`.
pub fn verify_journal_chain(conn: &Connection, key: &AuditKey) -> Result<(), JournalVerifyError> {
    let draft: Option<i64> = conn
        .query_row(
            "SELECT id FROM journal_entries WHERE status <> 'posted' ORDER BY id LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(DbError::from)?;
    if let Some(entry_id) = draft {
        return Err(JournalChainBreak { entry_id }.into());
    }
    let ids: Vec<i64> = {
        let mut statement = conn
            .prepare("SELECT id FROM journal_entries WHERE status = 'posted' ORDER BY id")
            .map_err(DbError::from)?;
        let rows = statement
            .query_map([], |r| r.get(0))
            .map_err(DbError::from)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)?
    };
    let mut previous = GENESIS_HASH.to_owned();
    let mut last = 0_i64;
    for id in ids {
        let entry = get_entry(conn, id)
            .map_err(DbError::from)?
            .ok_or(JournalChainBreak { entry_id: id })?;
        let expected = hash(key, &canonical_bytes(&previous, &entry, entry.total_paisa));
        match &entry.chain_hash {
            Some(stored) if *stored == expected => previous = expected,
            _ => return Err(JournalChainBreak { entry_id: id }.into()),
        }
        last = id;
    }
    // The head: with no entry there must be no record; with entries it must name the last one.
    let head = read_head(conn).map_err(DbError::from)?;
    let good = match &head {
        None => last == 0,
        Some((head_id, head_hash)) => *head_id == last && *head_hash == previous && last > 0,
    };
    if good {
        Ok(())
    } else {
        let entry_id = head.map_or(last, |(head_id, _)| head_id.max(last));
        Err(JournalChainBreak { entry_id }.into())
    }
}

/// What one account has on each side, over posted entries only (PRD FR-6.1), in paisa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Totals {
    /// Σ debits.
    pub dr_paisa: i64,
    /// Σ credits.
    pub cr_paisa: i64,
}

/// The posted debit and credit totals of every account that has lines, up to `as_of`
/// (`YYYY-MM-DD`, included) or all time. Sums are `SUM()` on the INTEGER columns only
/// (money agent Section 9); the difference is worked out by the caller with checked maths.
///
/// # Errors
/// Any SQLite error (a `SUM` that overflows is an error, never a float).
pub fn account_totals(
    conn: &Connection,
    as_of: Option<&str>,
) -> rusqlite::Result<Vec<(i64, Totals)>> {
    let mut statement = conn.prepare(
        "SELECT l.account_id, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE e.status = 'posted' AND (?1 IS NULL OR e.entry_date <= ?1)
         GROUP BY l.account_id ORDER BY l.account_id",
    )?;
    let rows = statement.query_map([as_of], |r| {
        Ok((
            r.get(0)?,
            Totals {
                dr_paisa: r.get(1)?,
                cr_paisa: r.get(2)?,
            },
        ))
    })?;
    rows.collect()
}

/// The posted debit and credit totals of the given accounts only (at most 2,000; SQLite allows
/// 32,766 parameters). One query with a parameter for each id, so a picker row or a page of the account list
/// does not sum the whole journal. An account with no posted line is not in the answer.
///
/// # Errors
/// Any SQLite error.
pub fn totals_for_accounts(
    conn: &Connection,
    ids: &[i64],
) -> rusqlite::Result<std::collections::HashMap<i64, Totals>> {
    if ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let marks = vec!["?"; ids.len()].join(", ");
    let sql = format!(
        "SELECT l.account_id, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id IN ({marks}) AND e.status = 'posted' GROUP BY l.account_id"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(rusqlite::params_from_iter(ids.iter()), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            Totals {
                dr_paisa: r.get(1)?,
                cr_paisa: r.get(2)?,
            },
        ))
    })?;
    rows.collect()
}

/// The date of the last posted entry that has a line on this account (`YYYY-MM-DD`), if any.
///
/// # Errors
/// Any SQLite error.
pub fn last_entry_date(conn: &Connection, account_id: i64) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT MAX(e.entry_date) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND e.status = 'posted'",
        [account_id],
        |r| r.get(0),
    )
}

/// What a seller was given as advances, in paisa: the posted debit lines flagged as an advance
/// (PRD FR-15.8), without those of an entry that has been reversed.
///
/// # Errors
/// Any SQLite error.
pub fn advance_given(conn: &Connection, account_id: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(SUM(l.dr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND l.is_advance = 1 AND e.status = 'posted'
           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                           WHERE r.reverses_entry_id = e.id AND r.status = 'posted')",
        [account_id],
        |r| r.get(0),
    )
}

/// What has been adjusted against a seller's advances, in paisa: the active rows of
/// `advance_allocations` (PRD FR-15.8) whose advance line and credit line are both still in
/// force. A row that points at a line of a reversed entry is left out, the same way
/// [`advance_given`] leaves out that line, so the two numbers always cover the same lines even
/// if a matching row was not switched off. Nothing writes these rows until the matching plans.
///
/// # Errors
/// Any SQLite error.
pub fn advance_adjusted(conn: &Connection, account_id: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(SUM(a.amount_paisa), 0)
         FROM advance_allocations a
         JOIN journal_lines advance_line ON advance_line.id = a.advance_line_id
         JOIN journal_lines credit_line ON credit_line.id = a.credit_line_id
         WHERE a.account_id = ?1 AND a.is_active = 1
           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                           WHERE r.reverses_entry_id = advance_line.entry_id AND r.status = 'posted')
           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                           WHERE r.reverses_entry_id = credit_line.entry_id AND r.status = 'posted')",
        [account_id],
        |r| r.get(0),
    )
}

/// The posted totals of one account. `person` is `None` for the whole account (all persons of a
/// پختہ کھاتہ account together), `Some(key)` for one person.
///
/// # Errors
/// Any SQLite error.
pub fn totals_of(
    conn: &Connection,
    account_id: i64,
    person: Option<&str>,
    as_of: Option<&str>,
) -> rusqlite::Result<Totals> {
    conn.query_row(
        "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE e.status = 'posted' AND l.account_id = ?1 AND (?2 IS NULL OR l.pukhta_person_key = ?2)
           AND (?3 IS NULL OR e.entry_date <= ?3)",
        params![account_id, person, as_of],
        |r| {
            Ok(Totals {
                dr_paisa: r.get(0)?,
                cr_paisa: r.get(1)?,
            })
        },
    )
}

/// One posted line of an account's ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerRow {
    /// The entry.
    pub entry_id: i64,
    /// Its date.
    pub entry_date: String,
    /// Line number in the entry.
    pub line_no: i64,
    /// The posting type.
    pub posting_type: String,
    /// Line note, else the entry note.
    pub narration: Option<String>,
    /// The person key.
    pub person_key: Option<String>,
    /// Debit in paisa.
    pub dr_paisa: i64,
    /// Credit in paisa.
    pub cr_paisa: i64,
}

/// The posted lines of an account from `from` to `to` (both included, either may be open), in
/// the order `(entry_date, entry_id, line_no)`.
///
/// # Errors
/// Any SQLite error.
pub fn ledger_rows(
    conn: &Connection,
    account_id: i64,
    person: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> rusqlite::Result<Vec<LedgerRow>> {
    let mut statement = conn.prepare(
        "SELECT e.id, e.entry_date, l.line_no, e.posting_type, COALESCE(l.narration, e.narration),
                l.pukhta_person_key, l.dr_paisa, l.cr_paisa
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE e.status = 'posted' AND l.account_id = ?1 AND (?2 IS NULL OR l.pukhta_person_key = ?2)
           AND (?3 IS NULL OR e.entry_date >= ?3) AND (?4 IS NULL OR e.entry_date <= ?4)
         ORDER BY e.entry_date, e.id, l.line_no",
    )?;
    let rows = statement.query_map(params![account_id, person, from, to], |r| {
        Ok(LedgerRow {
            entry_id: r.get(0)?,
            entry_date: r.get(1)?,
            line_no: r.get(2)?,
            posting_type: r.get(3)?,
            narration: r.get(4)?,
            person_key: r.get(5)?,
            dr_paisa: r.get(6)?,
            cr_paisa: r.get(7)?,
        })
    })?;
    rows.collect()
}

/// The ids of the پختہ کھاتہ accounts, whose lines carry a person key (PRD FR-13.5). There are
/// only the two system accounts, so this is one small query.
///
/// # Errors
/// Any SQLite error.
pub fn pukhta_khata_ids(conn: &Connection) -> rusqlite::Result<std::collections::HashSet<i64>> {
    let mut statement = conn.prepare("SELECT id FROM accounts WHERE is_pukhta_khata = 1")?;
    let rows = statement.query_map([], |r| r.get(0))?;
    rows.collect()
}

/// True if a posted reversal of this entry exists.
///
/// # Errors
/// Any SQLite error.
pub fn is_reversed(conn: &Connection, entry_id: i64) -> rusqlite::Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM journal_entries WHERE reverses_entry_id = ?1 AND status = 'posted'",
            [entry_id],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

/// Switches off the matching rows that point at a line of this entry, who and when (Code
/// Standards B17: matching rows are never deleted). Called when the entry is reversed: a
/// payment or an advance adjustment that rested on a reversed line no longer holds (PRD 12.9).
/// Returns how many advance rows and bill rows were switched off.
///
/// # Errors
/// Any SQLite error.
pub fn deactivate_matching(
    conn: &Connection,
    entry_id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<(usize, usize)> {
    let advances = conn.execute(
        "UPDATE advance_allocations SET is_active = 0, undone_at = ?2, undone_by = ?3,
             updated_at = ?2, updated_by = ?3, version = version + 1
         WHERE is_active = 1
           AND (advance_line_id IN (SELECT id FROM journal_lines WHERE entry_id = ?1)
                OR credit_line_id IN (SELECT id FROM journal_lines WHERE entry_id = ?1))",
        params![entry_id, now, user_id],
    )?;
    let bills = conn.execute(
        "UPDATE bill_allocations SET is_active = 0, undone_at = ?2, undone_by = ?3,
             updated_at = ?2, updated_by = ?3, version = version + 1
         WHERE is_active = 1
           AND (bill_line_id IN (SELECT id FROM journal_lines WHERE entry_id = ?1)
                OR payment_line_id IN (SELECT id FROM journal_lines WHERE entry_id = ?1))",
        params![entry_id, now, user_id],
    )?;
    Ok((advances, bills))
}
