//! The reads of the party ledgers screen (PRD FR-12.43, plan 66): one party's journal lines added
//! up by date. Read only (Code Standards B29). No rules and no arithmetic on money: every figure is
//! a `SUM()` of an INTEGER paisa column, and the differences are the caller's, with checked maths
//! (money agent Section 9).
//!
//! A date is listed when the party has a LIVE line on it. A line is live unless it is one of the
//! two halves of a same-day edit or cancel: an entry that reverses an entry of its own date, or an
//! entry that an entry of its own date reverses. This is the test the R35 sheet makes
//! (`reports::business::sheet::activity::SAME_DAY_PAIR`), so a day's `charged` and `paid` here are
//! the debit and credit columns of the R35 all-parties table (not the "paid today" line of its day
//! slip, which keeps cash lent and journal entries apart). The two halves cancel on the party's own account, so his balance after
//! the day is the same with or without them.
//!
//! The running balance is a window `SUM()` over the daily sums of ALL posted lines, oldest first,
//! so the balance of the newest date is the balance of his party card and the closing balance of
//! R13. The party's lines are found through `idx_journal_lines_account_amounts` (Code Standards
//! B18, NFR-SPD-24).

use rusqlite::{Connection, OptionalExtension, params};

/// The two halves of a same-day edit or cancel, as a test on the entry `e` (the same test as
/// `SAME_DAY_PAIR` of the R35 sheet in the reports crate; the parity tests of the services hold the
/// two statements equal on a same-day edit and a same-day cancel).
const SAME_DAY_PAIR: &str = "(e.reverses_entry_id IS NOT NULL AND EXISTS (
        SELECT 1 FROM journal_entries o
        WHERE o.id = e.reverses_entry_id AND o.entry_date = e.entry_date))
    OR EXISTS (SELECT 1 FROM journal_entries r
        WHERE r.reverses_entry_id = e.id AND r.status = 'posted' AND r.entry_date = e.entry_date)";

/// The daily sums of one party: `?1` is the account. Every posted line, by the date of its entry.
fn days_cte() -> String {
    format!(
        "WITH days AS (
           SELECT e.entry_date AS d,
                  SUM(l.dr_paisa) AS all_dr, SUM(l.cr_paisa) AS all_cr,
                  SUM(CASE WHEN {SAME_DAY_PAIR} THEN 0 ELSE l.dr_paisa END) AS live_dr,
                  SUM(CASE WHEN {SAME_DAY_PAIR} THEN 0 ELSE l.cr_paisa END) AS live_cr,
                  SUM(CASE WHEN {SAME_DAY_PAIR} THEN 0 ELSE 1 END) AS live_n
           FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
           WHERE l.account_id = ?1 AND e.status = 'posted'
           GROUP BY e.entry_date)"
    )
}

/// The page query of [`statement_days`]: `?1` the account, `?2` the first date or NULL, `?3` the
/// last date or NULL, `?4` the limit, `?5` the offset. Public so a test can read its plan (B18).
pub fn statement_days_sql() -> String {
    format!(
        "{cte}
         SELECT d, all_dr, all_cr, live_dr, live_cr, prev_dr, prev_cr FROM (
           SELECT d, all_dr, all_cr, live_dr, live_cr, live_n,
                  COALESCE(SUM(all_dr) OVER w, 0) AS prev_dr,
                  COALESCE(SUM(all_cr) OVER w, 0) AS prev_cr
           FROM days
           WINDOW w AS (ORDER BY d ROWS BETWEEN UNBOUNDED PRECEDING AND 1 PRECEDING))
         WHERE live_n > 0 AND (?2 IS NULL OR d >= ?2) AND (?3 IS NULL OR d <= ?3)
         ORDER BY d DESC LIMIT ?4 OFFSET ?5",
        cte = days_cte()
    )
}

/// The totals query of [`statement_totals`]: `?1` the account, `?2` the first date or NULL, `?3` the
/// last date or NULL. Public so a test can read its plan (B18).
pub fn statement_totals_sql() -> String {
    format!(
        "{cte}
         SELECT COUNT(CASE WHEN live_n > 0 AND (?2 IS NULL OR d >= ?2) AND (?3 IS NULL OR d <= ?3)
                           THEN 1 END),
                COALESCE(SUM(CASE WHEN live_n > 0 AND (?2 IS NULL OR d >= ?2) AND (?3 IS NULL OR d <= ?3)
                                  THEN live_dr END), 0),
                COALESCE(SUM(CASE WHEN live_n > 0 AND (?2 IS NULL OR d >= ?2) AND (?3 IS NULL OR d <= ?3)
                                  THEN live_cr END), 0),
                COALESCE(SUM(CASE WHEN ?2 IS NOT NULL AND d < ?2 THEN all_dr END), 0),
                COALESCE(SUM(CASE WHEN ?2 IS NOT NULL AND d < ?2 THEN all_cr END), 0),
                COALESCE(SUM(CASE WHEN ?3 IS NULL OR d <= ?3 THEN all_dr END), 0),
                COALESCE(SUM(CASE WHEN ?3 IS NULL OR d <= ?3 THEN all_cr END), 0)
         FROM days",
        cte = days_cte()
    )
}

/// One date of a party, in paisa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayRow {
    /// The date, `YYYY-MM-DD`.
    pub date: String,
    /// Σ debits of every posted line of the day.
    pub all_dr: i64,
    /// Σ credits of every posted line of the day.
    pub all_cr: i64,
    /// Σ debits of the live lines of the day.
    pub live_dr: i64,
    /// Σ credits of the live lines of the day.
    pub live_cr: i64,
    /// Σ debits of every posted line before the day.
    pub prev_dr: i64,
    /// Σ credits of every posted line before the day.
    pub prev_cr: i64,
}

/// A page of the dates of one party, newest first. `from` and `to` are `YYYY-MM-DD` or `None`.
///
/// # Errors
/// Any SQLite error (a `SUM` that overflows is an error, never a float).
pub fn statement_days(
    conn: &Connection,
    account_id: i64,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    offset: i64,
) -> rusqlite::Result<Vec<DayRow>> {
    let mut statement = conn.prepare_cached(&statement_days_sql())?;
    let rows = statement.query_map(params![account_id, from, to, limit, offset], |r| {
        Ok(DayRow {
            date: r.get(0)?,
            all_dr: r.get(1)?,
            all_cr: r.get(2)?,
            live_dr: r.get(3)?,
            live_cr: r.get(4)?,
            prev_dr: r.get(5)?,
            prev_cr: r.get(6)?,
        })
    })?;
    rows.collect()
}

/// The totals of a statement over every date the filter matches, not only a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatementTotals {
    /// How many dates match.
    pub count: i64,
    /// Σ debits of the live lines of the matching dates.
    pub live_dr: i64,
    /// Σ credits of the live lines of the matching dates.
    pub live_cr: i64,
    /// Σ debits of every posted line before the first date (0 with no `from`).
    pub opening_dr: i64,
    /// Σ credits of every posted line before the first date (0 with no `from`).
    pub opening_cr: i64,
    /// Σ debits of every posted line up to the last date (all of them with no `to`).
    pub closing_dr: i64,
    /// Σ credits of every posted line up to the last date (all of them with no `to`).
    pub closing_cr: i64,
}

/// The count, the live sums, and the opening and closing sums of a statement, in one query.
///
/// # Errors
/// Any SQLite error.
pub fn statement_totals(
    conn: &Connection,
    account_id: i64,
    from: Option<&str>,
    to: Option<&str>,
) -> rusqlite::Result<StatementTotals> {
    conn.prepare_cached(&statement_totals_sql())?
        .query_row(params![account_id, from, to], |r| {
            Ok(StatementTotals {
                count: r.get(0)?,
                live_dr: r.get(1)?,
                live_cr: r.get(2)?,
                opening_dr: r.get(3)?,
                opening_cr: r.get(4)?,
                closing_dr: r.get(5)?,
                closing_cr: r.get(6)?,
            })
        })
}

/// The condition that picks the parties of one control (`?1`) by the typed text (`?2` the Urdu key,
/// `?3` the English key; both empty is no text). The same test as the account list (`filter_sql` in
/// `accounts.rs`); a test of the services holds the two lists equal. The shared چکتا account,
/// نقدی منڈی and اپنا مال are system accounts and are left out: they have no ledger (PRD FR-13.6).
const PARTY_WHERE: &str = "c.code = ?1 AND a.is_system = 0
       AND ((?2 = '' AND ?3 = '')
            OR instr(a.code, ?3) > 0 OR instr(a.search_ur, ?2) > 0 OR instr(a.search_en, ?3) > 0)";

/// The page query of [`list_page`]: `?1` the control, `?2` and `?3` the search keys, `?4` the limit,
/// `?5` the offset. Public so a test can read its plan (B18).
pub fn list_page_sql() -> String {
    format!(
        "SELECT a.id, a.code, a.name_ur, a.name_en, a.tag, a.is_active
         FROM accounts a JOIN account_controls c ON c.id = a.control_id
         WHERE {PARTY_WHERE}
         ORDER BY {}, a.id LIMIT ?4 OFFSET ?5",
        crate::codes::order_by_code("a.code")
    )
}

/// One party of the list, with no figures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyRow {
    /// The account.
    pub id: i64,
    /// The full code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The tag, if any.
    pub tag: Option<String>,
    /// False for an account made inactive.
    pub is_active: bool,
}

fn party_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PartyRow> {
    Ok(PartyRow {
        id: r.get(0)?,
        code: r.get(1)?,
        name_ur: r.get(2)?,
        name_en: r.get(3)?,
        tag: r.get(4)?,
        is_active: r.get::<_, i64>(5)? != 0,
    })
}

/// A page of the parties of a control by code, active or not (an inactive account may still hold a
/// balance: the dead amount, PRD FR-10.10).
///
/// # Errors
/// Any SQLite error.
pub fn list_page(
    conn: &Connection,
    control: &str,
    text_ur: &str,
    text_en: &str,
    limit: i64,
    offset: i64,
) -> rusqlite::Result<Vec<PartyRow>> {
    let mut statement = conn.prepare_cached(&list_page_sql())?;
    let rows = statement.query_map(params![control, text_ur, text_en, limit, offset], party_row)?;
    rows.collect()
}

/// The ids of every party the filter matches, for the footer (never from a page, B31).
///
/// # Errors
/// Any SQLite error.
pub fn matching_ids(
    conn: &Connection,
    control: &str,
    text_ur: &str,
    text_en: &str,
) -> rusqlite::Result<Vec<i64>> {
    let mut statement = conn.prepare_cached(&format!(
        "SELECT a.id FROM accounts a JOIN account_controls c ON c.id = a.control_id
         WHERE {PARTY_WHERE}"
    ))?;
    let ids = statement.query_map(params![control, text_ur, text_en], |r| r.get(0))?;
    ids.collect()
}

/// One account for the head of a statement: the row and the code of its control and whether it is
/// a system account, so the service can refuse an account that has no ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadRow {
    /// The party.
    pub party: PartyRow,
    /// The code of the control it is under.
    pub control_code: String,
    /// True for a system account.
    pub is_system: bool,
}

/// The head of a statement, or `None` when there is no such account.
///
/// # Errors
/// Any SQLite error.
pub fn head(conn: &Connection, account_id: i64) -> rusqlite::Result<Option<HeadRow>> {
    conn.prepare_cached(
        "SELECT a.id, a.code, a.name_ur, a.name_en, a.tag, a.is_active, c.code, a.is_system
         FROM accounts a JOIN account_controls c ON c.id = a.control_id WHERE a.id = ?1",
    )?
    .query_row([account_id], |r| {
        Ok(HeadRow {
            party: party_row(r)?,
            control_code: r.get(6)?,
            is_system: r.get::<_, i64>(7)? != 0,
        })
    })
    .optional()
}
