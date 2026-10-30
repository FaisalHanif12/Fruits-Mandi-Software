//! The dead amount (ڈوبی رقم, PRD FR-10.10, plan 40, migration 0011): the moves, and the reads
//! the services and the reports share.
//!
//! A move holds no balance. What a person still owes is his balance, read from `journal_lines`
//! (PRD FR-6.1); the table only sorts people into a group and keeps what he owed on the day of
//! the move for the history. Nothing here writes a journal line.
//!
//! What came after a move is counted by journal entry id, not by date: the move stores the highest
//! POSTED entry id of that moment (`after_entry_id`), and everything with a higher id is "since"
//! (two entries of one day are told apart exactly, and a back-dated recovery still counts). Of the
//! lines since:
//!
//! * **recovered** = what a recovery put on his credit: an entry of type `receipt` or
//!   `money_lent_return`, less its reversal if it was cancelled or edited (PRD FR-10.10), so a
//!   cancelled recovery counts 0 and an edited one counts its new amount;
//! * **corrections** = every other movement: an edit, cancel or undo of an earlier posting (PRD
//!   A67, FR-10.10 Corrections), and the reversal of a recovery that was saved before the move.
//!
//! Debits and credits are summed apart and subtracted by the caller with checked maths, so the
//! identity `still = owed then − recovered + corrections` holds with no rounding (whole paisa,
//! only sums).
//!
//! Example (PRD E2E-44): ارشد owes 18,684 on 11-10-26 (`owed_paisa` 1,868,400, `after_entry_id`
//! the highest entry then). A receipt of 4,000 on 12-10-26 is a credit line of 400,000 in a
//! `receipt` entry: recovery credit 400,000, other 0, balance now 1,468,400.

use std::collections::{HashMap, HashSet};

use rusqlite::{Connection, OptionalExtension, params};

/// The posting types of a recovery: a وصولی (also a seller's, PRD FR-15.12) and an ادھار رقم
/// واپسی in cash or from salary (PRD FR-16.4).
pub const RECOVERY_POSTING_TYPES: [&str; 2] = ["receipt", "money_lent_return"];

/// The SQL test "this entry is a recovery, or the reversal of one". `e` is the entry alias and `m`
/// the move.
///
/// A recovery is a `receipt` or a `money_lent_return` entry. Its reversal (a cancel, or the first
/// half of an edit) takes it off "recovered" again, so a recovery of 4,000 that is cancelled shows
/// recovered 0, and one edited from 4,000 to 3,000 shows 3,000 (the lead, 30-10-2026, PRD
/// FR-10.10). A reversal of anything else is a correction. A reversal of a recovery saved BEFORE the
/// move (its id is not above the move's boundary) is a correction too: that receipt is in what he
/// owed then and was never counted as recovered, so taking it off would make "recovered" negative.
const IS_RECOVERY: &str = "(e.posting_type IN ('receipt', 'money_lent_return')
      OR (e.posting_type = 'reversal' AND EXISTS (
            SELECT 1 FROM journal_entries o
            WHERE o.id = e.reverses_entry_id
              AND o.posting_type IN ('receipt', 'money_lent_return')
              AND o.id > m.after_entry_id)))";

/// One move to the dead amount, as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkRow {
    /// Row id.
    pub id: i64,
    /// The account. For a پختہ کھاتہ person it is the پختہ account.
    pub account_id: i64,
    /// The person inside a پختہ کھاتہ account, else `None`.
    pub person_key: Option<String>,
    /// The day of the move, `YYYY-MM-DD`.
    pub entry_date: String,
    /// `died`, `left` or `other`.
    pub reason: String,
    /// The note typed with the move. Empty if none.
    pub note: String,
    /// What he owed at the moment of the move, paisa.
    pub owed_paisa: i64,
    /// The highest journal entry id at the moment of the move.
    pub after_entry_id: i64,
    /// False once the move was undone.
    pub is_active: bool,
    /// The highest journal entry id when it was undone, if it was.
    pub undone_after_entry_id: Option<i64>,
    /// The business day it was undone, `YYYY-MM-DD`, if it was.
    pub undone_date: Option<String>,
}

impl MarkRow {
    /// The move as the party card and the ledger heading show it (PRD FR-7.38, FR-10.10).
    pub fn info(&self) -> mandi_core::dead::DeadInfo {
        mandi_core::dead::DeadInfo {
            mark_id: self.id,
            entry_date: self.entry_date.clone(),
            reason: match self.reason.as_str() {
                "died" => mandi_core::dead::DeadReason::Died,
                "left" => mandi_core::dead::DeadReason::Left,
                _ => mandi_core::dead::DeadReason::Other,
            },
            note: self.note.clone(),
        }
    }
}

const MARK_COLUMNS: &str =
    "m.id, m.account_id, m.pukhta_person_key, m.entry_date, m.reason, m.note,
    m.owed_paisa, m.after_entry_id, m.is_active, m.undone_after_entry_id, m.undone_date";

fn mark_from(r: &rusqlite::Row<'_>) -> rusqlite::Result<MarkRow> {
    Ok(MarkRow {
        id: r.get(0)?,
        account_id: r.get(1)?,
        person_key: r.get(2)?,
        entry_date: r.get(3)?,
        reason: r.get(4)?,
        note: r.get(5)?,
        owed_paisa: r.get(6)?,
        after_entry_id: r.get(7)?,
        is_active: r.get::<_, i64>(8)? == 1,
        undone_after_entry_id: r.get(9)?,
        undone_date: r.get(10)?,
    })
}

/// A move to write.
#[derive(Debug, Clone, Copy)]
pub struct NewMark<'a> {
    /// The account.
    pub account_id: i64,
    /// The person key, for a پختہ کھاتہ person.
    pub person_key: Option<&'a str>,
    /// The day of the move, `YYYY-MM-DD`.
    pub entry_date: &'a str,
    /// `died`, `left` or `other`.
    pub reason: &'a str,
    /// The note. Empty if none.
    pub note: &'a str,
    /// What he owes now, paisa (above 0, whole rupees).
    pub owed_paisa: i64,
}

/// The highest POSTED journal entry id now (0 for an empty journal). A draft exists only inside the
/// transaction that is posting it, so inside the transaction of a move there is none; the test on
/// the status makes that a rule and not a hope.
///
/// # Errors
/// Any SQLite error.
pub fn max_entry_id(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(id), 0) FROM journal_entries WHERE status = 'posted'",
        [],
        |r| r.get(0),
    )
}

/// Writes a move and returns its id. The caller makes the account inactive in the same transaction
/// (not for a پختہ person) and writes the audit row.
///
/// # Errors
/// Any SQLite error, including the unique index: a second active move for one account or person.
pub fn insert(
    conn: &Connection,
    new: &NewMark<'_>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    let after = max_entry_id(conn)?;
    conn.execute(
        "INSERT INTO dead_marks (created_at, created_by, updated_at, updated_by, account_id,
                pukhta_person_key, entry_date, reason, note, owed_paisa, after_entry_id)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            now,
            user_id,
            new.account_id,
            new.person_key,
            new.entry_date,
            new.reason,
            new.note,
            new.owed_paisa,
            after
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// One move by id (active or undone).
///
/// # Errors
/// Any SQLite error.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<MarkRow>> {
    conn.query_row(
        &format!("SELECT {MARK_COLUMNS} FROM dead_marks m WHERE m.id = ?1"),
        [id],
        mark_from,
    )
    .optional()
}

/// The active move of an account (or of one person of a پختہ کھاتہ account), if any.
///
/// # Errors
/// Any SQLite error.
pub fn active_for(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<Option<MarkRow>> {
    conn.query_row(
        &format!(
            "SELECT {MARK_COLUMNS} FROM dead_marks m
             WHERE m.account_id = ?1 AND COALESCE(m.pukhta_person_key, '') = COALESCE(?2, '')
               AND m.is_active = 1"
        ),
        params![account_id, person_key],
        mark_from,
    )
    .optional()
}

/// The active move of any person of an account (used for the party card: an account that is moved
/// has a move without a key; a پختہ account has one move for each person).
///
/// # Errors
/// Any SQLite error.
pub fn active_for_account(conn: &Connection, account_id: i64) -> rusqlite::Result<Option<MarkRow>> {
    conn.query_row(
        &format!(
            "SELECT {MARK_COLUMNS} FROM dead_marks m
             WHERE m.account_id = ?1 AND m.pukhta_person_key IS NULL AND m.is_active = 1"
        ),
        [account_id],
        mark_from,
    )
    .optional()
}

/// Ends a move: who, when, why, the business day and the entry id it stopped at. Returns false if
/// there is no active move with this id.
///
/// # Errors
/// Any SQLite error.
pub fn undo(
    conn: &Connection,
    id: i64,
    reason: &str,
    day: &str,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<bool> {
    let stop = max_entry_id(conn)?;
    let changed = conn.execute(
        "UPDATE dead_marks SET is_active = 0, undone_at = ?1, undone_by = ?2, undo_reason = ?3,
                undone_after_entry_id = ?4, undone_date = ?5, updated_at = ?1, updated_by = ?2,
                version = version + 1
         WHERE id = ?6 AND is_active = 1",
        params![now, user_id, reason, stop, day, id],
    )?;
    Ok(changed == 1)
}

/// Every active move, oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn active_marks(conn: &Connection) -> rusqlite::Result<Vec<MarkRow>> {
    let mut statement = conn.prepare(&format!(
        "SELECT {MARK_COLUMNS} FROM dead_marks m WHERE m.is_active = 1
         ORDER BY m.entry_date, m.id"
    ))?;
    let rows = statement.query_map([], mark_from)?;
    rows.collect()
}

/// Every move, active or undone, oldest first (R38 with the filter "all").
///
/// # Errors
/// Any SQLite error.
pub fn all_marks(conn: &Connection) -> rusqlite::Result<Vec<MarkRow>> {
    let mut statement = conn.prepare(&format!(
        "SELECT {MARK_COLUMNS} FROM dead_marks m ORDER BY m.entry_date, m.id"
    ))?;
    let rows = statement.query_map([], mark_from)?;
    rows.collect()
}

/// The people in the dead amount, to leave them out of a list (PRD FR-10.10): the plain accounts
/// that are moved, and the one-time people (account and person key) that are moved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeadSet {
    accounts: HashSet<i64>,
    persons: HashSet<(i64, String)>,
}

impl DeadSet {
    /// True when there is nobody in the dead amount. The lists skip their filter then.
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty() && self.persons.is_empty()
    }

    /// True if this account is moved (a plain account).
    pub fn has_account(&self, account_id: i64) -> bool {
        self.accounts.contains(&account_id)
    }

    /// True if this key is moved: the account when `person_key` is `None`, the person when it is
    /// given. A line of a پختہ کھاتہ account with no key is never dead.
    pub fn has(&self, account_id: i64, person_key: Option<&str>) -> bool {
        match person_key {
            None => self.accounts.contains(&account_id),
            Some(key) => self.persons.contains(&(account_id, key.to_owned())),
        }
    }
}

/// The moves in force on a day: dated on or before `as_of` and not undone by then (`None` is now:
/// the active moves). A report for a day before a move still shows the person as he was then, and
/// a report for a day inside a move that was undone later still shows him dead.
///
/// # Errors
/// Any SQLite error.
pub fn marks_on(conn: &Connection, as_of: Option<&str>) -> rusqlite::Result<Vec<MarkRow>> {
    let mut statement = conn.prepare(&format!(
        "SELECT {MARK_COLUMNS} FROM dead_marks m
         WHERE (?1 IS NULL AND m.is_active = 1)
            OR (?1 IS NOT NULL AND m.entry_date <= ?1
                AND (m.is_active = 1 OR m.undone_date > ?1))
         ORDER BY m.entry_date, m.id"
    ))?;
    let rows = statement.query_map([as_of], mark_from)?;
    rows.collect()
}

/// The people in the dead amount on a day (see [`marks_on`]), to leave them out of a list.
///
/// # Errors
/// Any SQLite error.
pub fn dead_set(conn: &Connection, as_of: Option<&str>) -> rusqlite::Result<DeadSet> {
    let mut set = DeadSet::default();
    for mark in marks_on(conn, as_of)? {
        match mark.person_key {
            None => {
                set.accounts.insert(mark.account_id);
            }
            Some(key) => {
                set.persons.insert((mark.account_id, key));
            }
        }
    }
    Ok(set)
}

/// The day of the last posted line of an account (or one person of it), `YYYY-MM-DD`. The move to
/// the dead amount may not be dated before it (plan 40 D6).
///
/// # Errors
/// Any SQLite error.
pub fn last_posting_date(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT MAX(e.entry_date) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND (?2 IS NULL OR l.pukhta_person_key = ?2)
           AND e.status = 'posted'",
        params![account_id, person_key],
        |r| r.get(0),
    )
}

/// The sums behind the figures of one move, in paisa. Debits and credits are summed apart and
/// subtracted by the caller with checked maths (the money agent: no `SUM(dr - cr)` in SQL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Figures {
    /// Σ debit of the posted lines of the account (or person), up to the undo for an undone move.
    pub balance_dr: i64,
    /// Σ credit of the posted lines of the account (or person), up to the undo for an undone move.
    pub balance_cr: i64,
    /// Σ debit of the recovery entries since the move, with the reversals of recoveries.
    pub recovery_dr: i64,
    /// Σ credit of the recovery entries since the move, with the reversals of recoveries.
    pub recovery_cr: i64,
    /// Σ debit of every other entry since the move (edits, cancels and undos of earlier postings).
    pub other_dr: i64,
    /// Σ credit of every other entry since the move.
    pub other_cr: i64,
}

/// The figures of every move (active and undone) in two grouped queries, never one query for each
/// move (Code Standards B22). A move that is undone stops at `undone_after_entry_id`.
///
/// # Errors
/// Any SQLite error.
pub fn figures(conn: &Connection) -> rusqlite::Result<HashMap<i64, Figures>> {
    let mut out: HashMap<i64, Figures> = HashMap::new();
    // The balance of each moved account or person now.
    let mut statement = conn.prepare(
        "SELECT m.id, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM dead_marks m
         JOIN journal_lines l ON l.account_id = m.account_id
          AND l.pukhta_person_key IS m.pukhta_person_key
         JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
         -- A move that was undone stops at the undo: what is posted after he is active again is
         -- ordinary business and must not move the figures of the old move.
         WHERE m.is_active = 1 OR e.id <= m.undone_after_entry_id
         GROUP BY m.id",
    )?;
    let rows = statement.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
        ))
    })?;
    for row in rows {
        let (id, dr, cr) = row?;
        let f = out.entry(id).or_default();
        f.balance_dr = dr;
        f.balance_cr = cr;
    }
    // What came after the move.
    let mut statement = conn.prepare(&format!(
        "SELECT m.id,
                COALESCE(SUM(CASE WHEN {IS_RECOVERY} THEN l.dr_paisa ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN {IS_RECOVERY} THEN l.cr_paisa ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN {IS_RECOVERY} THEN 0 ELSE l.dr_paisa END), 0),
                COALESCE(SUM(CASE WHEN {IS_RECOVERY} THEN 0 ELSE l.cr_paisa END), 0)
         FROM dead_marks m
         JOIN journal_lines l ON l.account_id = m.account_id
          AND l.pukhta_person_key IS m.pukhta_person_key
         JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
         WHERE e.id > m.after_entry_id
           AND (m.is_active = 1 OR e.id <= m.undone_after_entry_id)
         GROUP BY m.id"
    ))?;
    let rows = statement.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, i64>(4)?,
        ))
    })?;
    for row in rows {
        let (id, rec_dr, rec_cr, other_dr, other_cr) = row?;
        let f = out.entry(id).or_default();
        f.recovery_dr = rec_dr;
        f.recovery_cr = rec_cr;
        f.other_dr = other_dr;
        f.other_cr = other_cr;
    }
    Ok(out)
}

/// The day of the last recovery of each move, `YYYY-MM-DD`: the latest posted `receipt` or
/// `money_lent_return` entry after the move that put a credit on him and was not reversed.
///
/// # Errors
/// Any SQLite error.
pub fn last_recovery_dates(conn: &Connection) -> rusqlite::Result<HashMap<i64, String>> {
    let mut statement = conn.prepare(
        "SELECT m.id, MAX(e.entry_date)
         FROM dead_marks m
         JOIN journal_lines l ON l.account_id = m.account_id
          AND l.pukhta_person_key IS m.pukhta_person_key AND l.cr_paisa > 0
         JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
         WHERE e.id > m.after_entry_id
           AND (m.is_active = 1 OR e.id <= m.undone_after_entry_id)
           AND e.posting_type IN ('receipt', 'money_lent_return')
           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                           WHERE r.reverses_entry_id = e.id AND r.status = 'posted')
         GROUP BY m.id",
    )?;
    let rows = statement.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
    rows.collect()
}
