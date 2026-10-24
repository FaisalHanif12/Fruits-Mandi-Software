//! The روکڑ بک in SQL (PRD 16.1, FR-19.11, plan 17): one row for each journal entry that moves
//! cash, one page at a time, with the sums over the whole range.
//!
//! No rules here. Sums are `SUM()` on INTEGER columns only (money agent Section 9); the running
//! total is a window `SUM()` of the debits and a window `SUM()` of the credits, and the caller
//! subtracts them in checked `Paisa` maths. Reads posted entries only (PRD FR-6.1).

use std::collections::HashMap;

use rusqlite::{Connection, params, params_from_iter};

/// One entry that moves cash, with the running sums up to and including it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CashRow {
    /// The journal entry.
    pub entry_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The stored posting type.
    pub posting_type: String,
    /// What cash received in this entry (Σ debit on cash), paisa.
    pub in_paisa: i64,
    /// What cash paid out in this entry (Σ credit on cash), paisa.
    pub out_paisa: i64,
    /// Σ of `in_paisa` over this row and every row before it in the range.
    pub run_in_paisa: i64,
    /// Σ of `out_paisa` over this row and every row before it in the range.
    pub run_out_paisa: i64,
    /// The number of its voucher or بکری, if it has one.
    pub number: Option<i64>,
    /// The note of the entry. Empty if none.
    pub narration: String,
    /// The reason or the "paid to" of its voucher. Empty if none.
    pub reason: String,
}

/// The sums over a whole range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashRange {
    /// How many entries move cash in the range.
    pub count: i64,
    /// Σ debits on cash, paisa.
    pub in_paisa: i64,
    /// Σ credits on cash, paisa.
    pub out_paisa: i64,
}

/// The sums of the entries of the range: the count, Σ in and Σ out. `from` and `to` are
/// `YYYY-MM-DD` and included; `None` leaves that end open.
///
/// # Errors
/// Any SQLite error.
pub fn range(
    conn: &Connection,
    cash_id: i64,
    from: Option<&str>,
    to: Option<&str>,
) -> rusqlite::Result<CashRange> {
    conn.query_row(
        "SELECT COUNT(DISTINCT e.id), COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND e.status = 'posted'
           AND (?2 IS NULL OR e.entry_date >= ?2) AND (?3 IS NULL OR e.entry_date <= ?3)",
        params![cash_id, from, to],
        |r| {
            Ok(CashRange {
                count: r.get(0)?,
                in_paisa: r.get(1)?,
                out_paisa: r.get(2)?,
            })
        },
    )
}

/// One page of the entries of the range, by day and entry id, with the running sums. The sums
/// are over the whole range, not the page, so a page far down has the right running cash.
///
/// # Errors
/// Any SQLite error.
pub fn page(
    conn: &Connection,
    cash_id: i64,
    from: Option<&str>,
    to: Option<&str>,
    offset: i64,
    limit: i64,
) -> rusqlite::Result<Vec<CashRow>> {
    let mut statement = conn.prepare(
        "WITH per_entry AS (
             SELECT e.id AS entry_id, e.entry_date AS entry_date, e.posting_type AS posting_type,
                    e.source_type AS source_type, e.source_id AS source_id,
                    COALESCE(e.narration, '') AS narration,
                    SUM(l.dr_paisa) AS in_p, SUM(l.cr_paisa) AS out_p
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE l.account_id = ?1 AND e.status = 'posted'
               AND (?2 IS NULL OR e.entry_date >= ?2) AND (?3 IS NULL OR e.entry_date <= ?3)
             GROUP BY e.id),
         running AS (
             SELECT per_entry.*,
                    SUM(in_p) OVER (ORDER BY entry_date, entry_id) AS run_in,
                    SUM(out_p) OVER (ORDER BY entry_date, entry_id) AS run_out
             FROM per_entry)
         SELECT r.entry_id, r.entry_date, r.posting_type, r.in_p, r.out_p, r.run_in, r.run_out,
                CASE r.source_type
                    WHEN 'voucher' THEN (SELECT v.number FROM vouchers v WHERE v.id = r.source_id)
                    WHEN 'bikri' THEN (SELECT b.number FROM bikri_entries b WHERE b.id = r.source_id)
                END,
                r.narration,
                CASE r.source_type
                    WHEN 'voucher' THEN COALESCE((SELECT v.reason FROM vouchers v WHERE v.id = r.source_id), '')
                    ELSE ''
                END
         FROM running r
         ORDER BY r.entry_date, r.entry_id
         LIMIT ?4 OFFSET ?5",
    )?;
    let rows = statement.query_map(params![cash_id, from, to, limit, offset], |r| {
        Ok(CashRow {
            entry_id: r.get(0)?,
            entry_date: r.get(1)?,
            posting_type: r.get(2)?,
            in_paisa: r.get(3)?,
            out_paisa: r.get(4)?,
            run_in_paisa: r.get(5)?,
            run_out_paisa: r.get(6)?,
            number: r.get(7)?,
            narration: r.get(8)?,
            reason: r.get(9)?,
        })
    })?;
    rows.collect()
}

/// One account on the other side of an entry that moves cash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counterpart {
    /// The entry.
    pub entry_id: i64,
    /// The account.
    pub account_id: i64,
    /// Its code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// True if the line is a debit.
    pub is_debit: bool,
}

/// The lines of these entries that are not on the cash account, in line order. One query for the
/// whole page.
///
/// # Errors
/// Any SQLite error.
pub fn counterparts(
    conn: &Connection,
    cash_id: i64,
    entry_ids: &[i64],
) -> rusqlite::Result<HashMap<i64, Vec<Counterpart>>> {
    let mut out: HashMap<i64, Vec<Counterpart>> = HashMap::new();
    if entry_ids.is_empty() {
        return Ok(out);
    }
    let marks = vec!["?"; entry_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT l.entry_id, l.account_id, a.code, a.name_ur, a.name_en, l.dr_paisa > 0
         FROM journal_lines l JOIN accounts a ON a.id = l.account_id
         WHERE l.account_id <> ? AND l.entry_id IN ({marks})
         ORDER BY l.entry_id, l.line_no"
    ))?;
    let mut values: Vec<i64> = Vec::with_capacity(entry_ids.len().saturating_add(1));
    values.push(cash_id);
    values.extend_from_slice(entry_ids);
    let rows = statement.query_map(params_from_iter(values.iter()), |r| {
        Ok(Counterpart {
            entry_id: r.get(0)?,
            account_id: r.get(1)?,
            code: r.get(2)?,
            name_ur: r.get(3)?,
            name_en: r.get(4)?,
            is_debit: r.get(5)?,
        })
    })?;
    for row in rows {
        let row = row?;
        out.entry(row.entry_id).or_default().push(row);
    }
    Ok(out)
}
