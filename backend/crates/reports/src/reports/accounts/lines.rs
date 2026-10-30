//! Pages of journal lines in day order, for R11 (the day book) and R13 / R14 (the party ledger)
//! (plan 38 B2, NFR-SPD-24).
//!
//! A page must not read the whole account. The window `SUM() OVER` that gave every row its
//! running balance read all the lines of the account up to the page (55,000 lines for the busiest
//! buyer of the 10-year data), for every page. Now:
//!
//! - **The page** is the lines of the filter in the order of the journal (day, entry, line), cut
//!   with `LIMIT` and `OFFSET`. For a big account, or for the whole journal, the entries are
//!   walked by the day index in that order, so only the lines the page needs are read. For a small
//!   account the lines of the account are read and sorted (a few thousand at most).
//! - **The running balance** of the first row is the sums of the lines before it, read from the
//!   nearer end of the account: the lines before the page, or the total less the page and the lines
//!   after it. Each row after the first adds its own line, in checked integer maths.
//! - **The totals** are kept for the next page (`mandi_db::memo`).
//!
//! Example: the busiest buyer has 55,658 lines. Page 1 reads about 100 lines and the sums of
//! nothing; page 2 the sums of 100 lines; the totals come from the memory.

use mandi_db::memo::{self, Sums};
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use crate::error::ReportsError;
use crate::reports::filters::LineSort;

/// An account with more lines than this is walked by the day index; a smaller one is sorted.
const LARGE_ACCOUNT: i64 = 20_000;

/// A page this close to an end of a big account is found by walking the entries by the day index:
/// each line costs about 20 entries read (the account is a twentieth of the journal), so up to this
/// many lines the walk is faster than reading all the lines of the account once. Further in, the
/// lines of the account are read and sorted (plan 38 B2, measured on the 10-year volume).
const WALK_LINES: i64 = 2_000;

/// A range of days this short is walked by the day index even for a big account: its entries are a
/// few thousand, fewer than the lines of the account.
const NARROW_DAYS: i64 = 62;

/// Which lines: an account (or all), one person of a پختہ کھاتہ account, and a range of days.
#[derive(Debug, Clone, Default)]
pub(crate) struct LineFilter {
    /// Only this account.
    pub(crate) account: Option<i64>,
    /// Only this one-time person.
    pub(crate) person: Option<String>,
    /// First day, `YYYY-MM-DD`, included.
    pub(crate) from: Option<String>,
    /// Last day, `YYYY-MM-DD`, included.
    pub(crate) to: Option<String>,
}

impl LineFilter {
    /// `AND` conditions on the entry `e` and the line `l`, only for what is set, and their values.
    fn conditions(&self) -> (String, Vec<Value>) {
        let mut sql = String::new();
        let mut binds = Vec::new();
        if let Some(account) = self.account {
            sql.push_str(" AND l.account_id = ?");
            binds.push(Value::Integer(account));
        }
        if let Some(person) = &self.person {
            sql.push_str(" AND l.pukhta_person_key = ?");
            binds.push(Value::Text(person.clone()));
        }
        if let Some(from) = &self.from {
            sql.push_str(" AND e.entry_date >= ?");
            binds.push(Value::Text(from.clone()));
        }
        if let Some(to) = &self.to {
            sql.push_str(" AND e.entry_date <= ?");
            binds.push(Value::Text(to.clone()));
        }
        (sql, binds)
    }

    /// What the memory is asked for.
    fn key(&self, what: &str) -> String {
        format!(
            "{what} {:?} {:?} {:?} {:?}",
            self.account, self.person, self.from, self.to
        )
    }

    /// The same lines with another range of days.
    pub(crate) fn between(&self, from: Option<&str>, to: Option<&str>) -> LineFilter {
        LineFilter {
            account: self.account,
            person: self.person.clone(),
            from: from.map(str::to_owned),
            to: to.map(str::to_owned),
        }
    }
}

/// True if the account is big enough to be walked by the day index. All the journal (no account)
/// is big.
fn is_large(conn: &Connection, account: Option<i64>) -> Result<bool, ReportsError> {
    let Some(account) = account else {
        return Ok(true);
    };
    let counted: i64 = conn.query_row(
        "SELECT COUNT(*) FROM (SELECT 1 FROM journal_lines WHERE account_id = ?1 LIMIT ?2)",
        [account, LARGE_ACCOUNT.saturating_add(1)],
        |r| r.get(0),
    )?;
    Ok(counted > LARGE_ACCOUNT)
}

/// True if the filter has both days and they are at most [`NARROW_DAYS`] apart.
fn is_narrow(f: &LineFilter) -> bool {
    use mandi_core::dates::BusinessDate;
    match (
        f.from.as_deref().and_then(BusinessDate::parse_iso),
        f.to.as_deref().and_then(BusinessDate::parse_iso),
    ) {
        (Some(from), Some(to)) => to.days_after(from) <= NARROW_DAYS,
        _ => false,
    }
}

/// The tables of a query that adds up a whole range (no `LIMIT`, so no early stop): it reads all
/// of the range. For a big account the lines of the account (at most its size) beat the entries of
/// a wide range (up to the whole journal); the entries by the day index win only for a narrow
/// range, and for the whole journal (no account).
fn sum_tables(large: bool, f: &LineFilter) -> &'static str {
    if large && f.account.is_some() && !is_narrow(f) {
        tables(false)
    } else {
        tables(large)
    }
}

/// The tables of a query: entries by the day index for a big account or the whole journal,
/// otherwise the lines of the account first.
fn tables(large: bool) -> &'static str {
    if large {
        "journal_entries e INDEXED BY idx_journal_entries_entry_date
         JOIN journal_lines l ON l.entry_id = e.id"
    } else {
        "journal_lines l JOIN journal_entries e ON e.id = l.entry_id"
    }
}

fn no_drafts(conn: &Connection) -> mandi_db::rusqlite::Result<bool> {
    let any: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM journal_entries WHERE status <> 'posted')",
        [],
        |r| r.get(0),
    )?;
    Ok(!any)
}

/// The count of lines and Σ debit and Σ credit of the filter.
///
/// # Errors
/// A database error.
pub(crate) fn totals(conn: &Connection, f: &LineFilter) -> Result<Sums, ReportsError> {
    let large = is_large(conn, f.account)?;
    let sums = memo::sums(conn, &f.key("lines"), || {
        if let (Some(account), None, None, None) = (f.account, &f.person, &f.from, &f.to)
            && no_drafts(conn)?
        {
            // The whole account: the covering index alone, no join with the entries.
            return conn.query_row(
                "SELECT COUNT(*), COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0)
                 FROM journal_lines WHERE account_id = ?1",
                [account],
                read_sums,
            );
        }
        let (conditions, binds) = f.conditions();
        conn.query_row(
            &format!(
                "SELECT COUNT(*), COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
                 FROM {} WHERE e.status = 'posted'{conditions}",
                sum_tables(large, f)
            ),
            params_from_iter(binds.iter()),
            read_sums,
        )
    })?;
    Ok(sums)
}

fn read_sums(r: &mandi_db::rusqlite::Row<'_>) -> mandi_db::rusqlite::Result<Sums> {
    Ok(Sums {
        count: r.get(0)?,
        dr_paisa: r.get(1)?,
        cr_paisa: r.get(2)?,
    })
}

/// The sums of the lines dated before `first_day`: the whole account less what is dated from the
/// first day on, both kept sums.
///
/// # Errors
/// A database error; `ReportsError::Data` if the sums do not fit.
pub(crate) fn before_day(
    conn: &Connection,
    f: &LineFilter,
    first_day: &str,
) -> Result<(i64, i64), ReportsError> {
    let whole = totals(conn, &f.between(None, None))?;
    let later = totals(conn, &f.between(Some(first_day), None))?;
    let dr = whole
        .dr_paisa
        .checked_sub(later.dr_paisa)
        .ok_or_else(|| ReportsError::Data("debits before the first day".to_owned()))?;
    let cr = whole
        .cr_paisa
        .checked_sub(later.cr_paisa)
        .ok_or_else(|| ReportsError::Data("credits before the first day".to_owned()))?;
    Ok((dr, cr))
}

/// One line as the pages read it, with the columns R11 and R13 both need.
pub(crate) struct RawLine {
    pub(crate) entry_id: i64,
    pub(crate) entry_date: String,
    pub(crate) line_no: i64,
    pub(crate) posting_type: String,
    pub(crate) source_type: String,
    pub(crate) source_id: i64,
    pub(crate) reverses: Option<i64>,
    pub(crate) account_id: i64,
    pub(crate) narration: Option<String>,
    pub(crate) person_key: Option<String>,
    pub(crate) dr: i64,
    pub(crate) cr: i64,
    pub(crate) doc_number: Option<i64>,
    pub(crate) voucher_type: Option<String>,
    pub(crate) voucher_reason: Option<String>,
}

/// The page query and the values it binds (public for the query plan test, Code Standards B18).
pub(crate) fn page_query(
    conn: &Connection,
    f: &LineFilter,
    sort: Option<LineSort>,
    (offset, limit): (i64, i64),
) -> Result<(String, Vec<Value>), ReportsError> {
    let large = is_large(conn, f.account)?;
    // By an amount there is no early stop on the day index: the lines of the account are read and
    // the best `offset + limit` are kept (SQLite keeps only that many while it sorts).
    let by_amount = matches!(sort, Some(LineSort::Amount { .. }));
    let by = if f.account.is_some() && (by_amount || offset > WALK_LINES) {
        tables(false)
    } else {
        tables(large)
    };
    let order = match sort {
        None => "e.entry_date, e.id, l.line_no".to_owned(),
        Some(LineSort::Newest) => "e.entry_date DESC, e.id DESC, l.line_no DESC".to_owned(),
        Some(LineSort::Amount { credit, desc }) => format!(
            "{} {}, e.entry_date, e.id, l.line_no",
            if credit { "l.cr_paisa" } else { "l.dr_paisa" },
            if desc { "DESC" } else { "ASC" }
        ),
    };
    let (conditions, mut binds) = f.conditions();
    binds.push(Value::Integer(limit));
    binds.push(Value::Integer(offset));
    let sql = format!(
        "SELECT e.id, e.entry_date, l.line_no, e.posting_type, e.source_type, e.source_id,
                e.reverses_entry_id, l.account_id, COALESCE(l.narration, e.narration),
                l.pukhta_person_key, l.dr_paisa, l.cr_paisa,
                COALESCE(b.number, v.number), v.type, v.reason
         FROM {by} LEFT JOIN bikri_entries b ON e.source_type = 'bikri' AND b.id = e.source_id
         LEFT JOIN vouchers v ON e.source_type = 'voucher' AND v.id = e.source_id
         WHERE e.status = 'posted'{conditions}
         ORDER BY {order} LIMIT ? OFFSET ?"
    );
    Ok((sql, binds))
}

/// One page of the lines in the order of the journal.
///
/// # Errors
/// A database error.
pub(crate) fn page(
    conn: &Connection,
    f: &LineFilter,
    window: (i64, i64),
) -> Result<Vec<RawLine>, ReportsError> {
    page_by(conn, f, None, window)
}

/// One page of the lines in the order of a sort by amount, or of the journal when `sort` is none.
/// The lines with the same amount keep the order of the journal.
///
/// # Errors
/// A database error.
pub(crate) fn page_by(
    conn: &Connection,
    f: &LineFilter,
    sort: Option<LineSort>,
    window: (i64, i64),
) -> Result<Vec<RawLine>, ReportsError> {
    let (sql, binds) = page_query(conn, f, sort, window)?;
    let mut statement = conn.prepare(&sql)?;
    let lines = statement
        .query_map(params_from_iter(binds.iter()), |r| {
            Ok(RawLine {
                entry_id: r.get(0)?,
                entry_date: r.get(1)?,
                line_no: r.get(2)?,
                posting_type: r.get(3)?,
                source_type: r.get(4)?,
                source_id: r.get(5)?,
                reverses: r.get(6)?,
                account_id: r.get(7)?,
                narration: r.get(8)?,
                person_key: r.get(9)?,
                dr: r.get(10)?,
                cr: r.get(11)?,
                doc_number: r.get(12)?,
                voucher_type: r.get(13)?,
                voucher_reason: r.get(14)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(lines)
}

/// Σ debit and Σ credit of the lines of the filter on one side of `key` (day, entry, line):
/// `later` true is after it, false before it.
fn sums_beside(
    conn: &Connection,
    f: &LineFilter,
    key: (&str, i64, i64),
    (later, walk): (bool, bool),
) -> Result<(i64, i64), ReportsError> {
    let large = is_large(conn, f.account)?;
    let (conditions, mut binds) = f.conditions();
    let op = if later { ">" } else { "<" };
    binds.push(Value::Text(key.0.to_owned()));
    binds.push(Value::Text(key.0.to_owned()));
    binds.push(Value::Integer(key.1));
    binds.push(Value::Integer(key.1));
    binds.push(Value::Integer(key.2));
    Ok(conn.query_row(
        &format!(
            "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
             FROM {} WHERE e.status = 'posted'{conditions}
               AND (e.entry_date {op} ? OR (e.entry_date = ?
                    AND (e.id {op} ? OR (e.id = ? AND l.line_no {op} ?))))",
            // Far from the end the lines of a big account (at most its size) beat the entries
            // before or after a day, which can be half of the journal.
            if large && f.account.is_some() && !walk {
                tables(false)
            } else {
                tables(large)
            }
        ),
        params_from_iter(binds.iter()),
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?)
}

/// One page of the lines with the running sums (Σ debit and Σ credit from the start of the filter
/// up to and including each line). See the module doc.
///
/// # Errors
/// A database error; `ReportsError::Data` if the sums do not fit.
pub(crate) fn page_with_running(
    conn: &Connection,
    f: &LineFilter,
    window: (i64, i64),
) -> Result<Vec<(RawLine, i64, i64)>, ReportsError> {
    let lines = page(conn, f, window)?;
    let (Some(first), Some(last)) = (lines.first(), lines.last()) else {
        return Ok(Vec::new());
    };
    let overflow = || ReportsError::Data("running sums do not fit".to_owned());
    let total = totals(conn, f)?;
    let mut page_dr = 0_i64;
    let mut page_cr = 0_i64;
    for line in &lines {
        page_dr = page_dr.checked_add(line.dr).ok_or_else(overflow)?;
        page_cr = page_cr.checked_add(line.cr).ok_or_else(overflow)?;
    }
    let in_page = i64::try_from(lines.len()).unwrap_or(i64::MAX);
    let (mut run_dr, mut run_cr) = if window.0 <= total.count / 2 {
        sums_beside(
            conn,
            f,
            (&first.entry_date, first.entry_id, first.line_no),
            (false, window.0 <= WALK_LINES),
        )?
    } else {
        let after = sums_beside(
            conn,
            f,
            (&last.entry_date, last.entry_id, last.line_no),
            (
                true,
                total.count.saturating_sub(window.0).saturating_sub(in_page) <= WALK_LINES,
            ),
        )?;
        (
            total
                .dr_paisa
                .checked_sub(page_dr)
                .and_then(|v| v.checked_sub(after.0))
                .ok_or_else(overflow)?,
            total
                .cr_paisa
                .checked_sub(page_cr)
                .and_then(|v| v.checked_sub(after.1))
                .ok_or_else(overflow)?,
        )
    };
    let mut out = Vec::with_capacity(lines.len());
    for line in lines {
        run_dr = run_dr.checked_add(line.dr).ok_or_else(overflow)?;
        run_cr = run_cr.checked_add(line.cr).ok_or_else(overflow)?;
        out.push((line, run_dr, run_cr));
    }
    Ok(out)
}

/// One page of the lines newest first, with the running sums the lines have in the order of the
/// days. The rows `offset..offset + limit` counted from the newest are the rows
/// `count - offset - n..count - offset` counted from the oldest, so the page is read in the natural
/// order, with the sums of that order, and turned round.
///
/// # Errors
/// A database error; `ReportsError::Data` if the sums do not fit.
pub(crate) fn page_with_running_newest(
    conn: &Connection,
    f: &LineFilter,
    (offset, limit): (i64, i64),
) -> Result<Vec<(RawLine, i64, i64)>, ReportsError> {
    let count = totals(conn, f)?.count;
    let end = count.saturating_sub(offset);
    if end <= 0 {
        return Ok(Vec::new());
    }
    // A negative limit is no limit (SQLite).
    let n = if limit < 0 { end } else { end.min(limit) };
    let mut rows = page_with_running(conn, f, (end - n, n))?;
    rows.reverse();
    Ok(rows)
}

/// The page queries, for the query plan test (Code Standards B18).
#[doc(hidden)]
pub mod plans {
    use super::{LineFilter, Value, page_query};
    use mandi_db::rusqlite::Connection;

    /// The page query of the party ledger R13 for one account and a range of days.
    pub fn ledger_page(
        conn: &Connection,
        account: i64,
        from: Option<&str>,
        to: Option<&str>,
    ) -> (String, Vec<Value>) {
        let f = LineFilter {
            account: Some(account),
            person: None,
            from: from.map(str::to_owned),
            to: to.map(str::to_owned),
        };
        page_query(conn, &f, None, (0, 100)).unwrap_or_default()
    }

    /// The page query of the day book R11 for a range of days.
    pub fn journal_page(
        conn: &Connection,
        from: Option<&str>,
        to: Option<&str>,
    ) -> (String, Vec<Value>) {
        let f = LineFilter {
            account: None,
            person: None,
            from: from.map(str::to_owned),
            to: to.map(str::to_owned),
        };
        page_query(conn, &f, None, (0, 100)).unwrap_or_default()
    }
}
