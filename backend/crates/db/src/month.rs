//! Month close in SQL (plan 22 round 7, PRD FR-17.6, FR-21.2): the sums a month close moves and
//! the `month_closes` rows. No rules here: the plan is made and checked in `services::month`.
//!
//! A "month" is the entries with `entry_date` from the first to the last day of it, posted, and
//! not themselves month-close entries (`source_type <> 'month_close'`): the entry of a close and
//! the entry that reverses it never count. Money is summed only with `SUM()` on INTEGER columns;
//! every difference is worked out in Rust with checked maths.

use rusqlite::{Connection, OptionalExtension, params};

/// The debits and credits of one income or shop expense account in a month.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountNet {
    /// The account.
    pub account_id: i64,
    /// Its code.
    pub code: String,
    /// The code of its head: `04` (income) or `05` (shop expenses).
    pub head_code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// `SUM(dr_paisa)` of its lines in the month.
    pub dr_paisa: i64,
    /// `SUM(cr_paisa)` of its lines in the month.
    pub cr_paisa: i64,
}

/// Every account of head 04 (income) or 05 (shop expenses) with lines in the range, with its
/// sums.
///
/// # Errors
/// Any SQLite error.
pub fn head_nets(conn: &Connection, from: &str, to: &str) -> rusqlite::Result<Vec<AccountNet>> {
    // The head decides what an account is, as in R30 and in the split hook.
    let mut statement = conn.prepare(
        "SELECT a.id, a.code, h.code, a.name_ur, a.name_en,
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls c ON c.id = a.control_id
         JOIN account_heads h ON h.id = c.head_id
         WHERE h.code IN ('04', '05')
           AND e.status = 'posted' AND e.source_type <> 'month_close'
           AND e.entry_date >= ?1 AND e.entry_date <= ?2
         GROUP BY a.id, a.code, h.code, a.name_ur, a.name_en
         ORDER BY a.code",
    )?;
    let rows = statement.query_map(params![from, to], |r| {
        Ok(AccountNet {
            account_id: r.get(0)?,
            code: r.get(1)?,
            head_code: r.get(2)?,
            name_ur: r.get(3)?,
            name_en: r.get(4)?,
            dr_paisa: r.get(5)?,
            cr_paisa: r.get(6)?,
        })
    })?;
    rows.collect()
}

/// One partner's share rows of a month, added up, with his capital account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartnerMonth {
    /// The partner.
    pub partner_id: i64,
    /// His capital account (control 0602).
    pub capital_account_id: i64,
    /// Its code.
    pub capital_code: String,
    /// Its Urdu name.
    pub name_ur: String,
    /// Its English name.
    pub name_en: String,
    /// `SUM(amount_paisa)` of his share rows dated in the month: income shares are positive,
    /// expense shares negative (PRD FR-17.4).
    pub total_paisa: i64,
}

/// Each partner with share rows in the range, with the sum of all his rows (income and expense).
///
/// # Errors
/// Any SQLite error.
pub fn partner_months(
    conn: &Connection,
    from: &str,
    to: &str,
) -> rusqlite::Result<Vec<PartnerMonth>> {
    // The rows are read through the same entries as the income and expense sums (posted, not a
    // month close, dated in the range), so both sides of the close come from one set of entries.
    let mut statement = conn.prepare(
        "SELECT p.id, p.capital_account_id, a.code, a.name_ur, a.name_en,
                COALESCE(SUM(s.amount_paisa), 0)
         FROM partner_share_rows s
         JOIN journal_entries e ON e.id = s.journal_entry_id
         JOIN partners p ON p.id = s.partner_id
         JOIN accounts a ON a.id = p.capital_account_id
         WHERE e.status = 'posted' AND e.source_type <> 'month_close'
           AND e.entry_date >= ?1 AND e.entry_date <= ?2
         GROUP BY p.id, p.capital_account_id, a.code, a.name_ur, a.name_en
         ORDER BY p.id",
    )?;
    let rows = statement.query_map(params![from, to], |r| {
        Ok(PartnerMonth {
            partner_id: r.get(0)?,
            capital_account_id: r.get(1)?,
            capital_code: r.get(2)?,
            name_ur: r.get(3)?,
            name_en: r.get(4)?,
            total_paisa: r.get(5)?,
        })
    })?;
    rows.collect()
}

/// A stored month close with the date of its entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthCloseRow {
    /// Row id.
    pub id: i64,
    /// `YYYY-MM`.
    pub month: String,
    /// The close entry.
    pub journal_entry_id: i64,
    /// The date of the close entry (the day the Owner pressed the button).
    pub entry_date: String,
    /// True while the close is in force.
    pub is_active: bool,
    /// The entry that reversed it.
    pub reversed_entry_id: Option<i64>,
}

const CLOSE_SELECT: &str =
    "SELECT m.id, m.month, m.journal_entry_id, e.entry_date, m.is_active, m.reversed_entry_id
     FROM month_closes m JOIN journal_entries e ON e.id = m.journal_entry_id";

fn close_of(r: &rusqlite::Row<'_>) -> rusqlite::Result<MonthCloseRow> {
    Ok(MonthCloseRow {
        id: r.get(0)?,
        month: r.get(1)?,
        journal_entry_id: r.get(2)?,
        entry_date: r.get(3)?,
        is_active: r.get::<_, i64>(4)? == 1,
        reversed_entry_id: r.get(5)?,
    })
}

/// Every month close ever made, newest first, in force or reversed.
///
/// # Errors
/// Any SQLite error.
pub fn closes(conn: &Connection) -> rusqlite::Result<Vec<MonthCloseRow>> {
    let mut statement = conn.prepare(&format!("{CLOSE_SELECT} ORDER BY m.id DESC"))?;
    let rows = statement.query_map([], close_of)?;
    rows.collect()
}

/// The close in force for a month, if any.
///
/// # Errors
/// Any SQLite error.
pub fn active(conn: &Connection, month: &str) -> rusqlite::Result<Option<MonthCloseRow>> {
    conn.query_row(
        &format!("{CLOSE_SELECT} WHERE m.month = ?1 AND m.is_active = 1"),
        [month],
        close_of,
    )
    .optional()
}

/// Notes a month close in force.
///
/// # Errors
/// Any SQLite error.
pub fn insert(
    conn: &Connection,
    month: &str,
    journal_entry_id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO month_closes (created_at, created_by, updated_at, updated_by, month, journal_entry_id, is_active)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, 1)",
        params![now, user_id, month, journal_entry_id],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Marks a close as reversed by an entry.
///
/// # Errors
/// Any SQLite error.
pub fn deactivate(
    conn: &Connection,
    id: i64,
    reversed_entry_id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE month_closes SET is_active = 0, reversed_entry_id = ?2, updated_at = ?3, updated_by = ?4,
                version = version + 1
         WHERE id = ?1",
        params![id, reversed_entry_id, now, user_id],
    )?;
    Ok(())
}

/// The month (`YYYY-MM`) of the oldest posted entry, if there is one.
///
/// # Errors
/// Any SQLite error.
pub fn first_month(conn: &Connection) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT substr(MIN(entry_date), 1, 7) FROM journal_entries WHERE status = 'posted'",
        [],
        |r| r.get::<_, Option<String>>(0),
    )
}

/// The `closed up to` date in the transaction of the caller.
///
/// # Errors
/// Any SQLite error.
pub fn closed_up_to(conn: &Connection) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT closed_up_to_date FROM day_state WHERE id = 1",
        [],
        |r| r.get::<_, Option<String>>(0),
    )
    .optional()
    .map(Option::flatten)
}
