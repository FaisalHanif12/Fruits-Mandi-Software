//! The روکڑ بک in SQL (PRD 16.1, FR-19.11, plan 17, plan 38 B2, NFR-SPD-24): one row for each
//! journal entry that moves cash, one page at a time, with the sums over the whole range.
//!
//! No rules here. Sums are `SUM()` on INTEGER columns only (money agent Section 9); the caller
//! subtracts in checked `Paisa` maths. Reads posted entries only (PRD FR-6.1).
//!
//! The cash account has hundreds of thousands of lines, so a page must not read them all:
//!
//! - **Natural order** (by day, then entry): the page walks `journal_entries` by the day index and
//!   stops after `offset + limit` cash entries, so the first pages cost a few hundred lookups. The
//!   running cash of the first row comes from the sums of the entries before the page, or, past
//!   the middle of the book, from the total less the entries after it.
//! - **By amount**: the cash lines are grouped by entry on the covering index (no join with the
//!   entries) and the biggest are taken. The running cash is not given for this order.
//! - **By another column** (posting type, number, narration) the entries are read and sorted; this
//!   is the slow path and gives no running cash either.
//! - **Totals** of the whole book come from the covering index when no day is asked for, and from
//!   the day index when a range is; the last answers are kept (`memo`).

use std::collections::HashMap;

use rusqlite::types::Value;
use rusqlite::{Connection, params_from_iter};

use crate::listing::MonthSums;
use crate::memo::{self, Sums};

/// One entry that moves cash.
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
    /// Σ of `in_paisa` over this row and every row before it in the range, in day order. `None`
    /// when the page is not in day order: the running cash means nothing there.
    pub run_in_paisa: Option<i64>,
    /// Σ of `out_paisa` over this row and every row before it in the range. `None` as above.
    pub run_out_paisa: Option<i64>,
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

/// How a page is ordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    /// By day, then entry, oldest first (`desc` false) or newest first. The only order with a
    /// running cash.
    Date {
        /// Newest first.
        desc: bool,
    },
    /// By what cash received (`out` false) or paid out (`out` true).
    Amount {
        /// Sort by the amount paid out.
        out: bool,
        /// Biggest first.
        desc: bool,
    },
    /// By another column: the SQL expression over `r` (see [`SORT_COLUMNS`]) with its direction.
    Other(String),
}

/// The columns the cash book can be sorted by: the name of the field of the row and its SQL
/// expression over the slow path's `r`. `entry_date`, `in_paisa` and `out_paisa` have their own
/// fast paths (see [`Order`]); `running_paisa` is the day order, so it is not a sort of its own.
pub const SORT_COLUMNS: [(&str, &str); 6] = [
    ("entry_date", "r.entry_date"),
    ("posting_type", "r.posting_type"),
    ("number", NUMBER_OF_ENTRY),
    ("narration", "r.narration"),
    ("in_paisa", "r.in_p"),
    ("out_paisa", "r.out_p"),
];

/// The number of the voucher or بکری of an entry, as the slow path reads it.
const NUMBER_OF_ENTRY: &str = "CASE r.source_type
    WHEN 'voucher' THEN (SELECT v.number FROM vouchers v WHERE v.id = r.source_id)
    WHEN 'bikri' THEN (SELECT b.number FROM bikri_entries b WHERE b.id = r.source_id)
END";

/// `AND` conditions on the day of an entry `e`, only for the ends that are set, and their values.
fn day_conditions(from: Option<&str>, to: Option<&str>) -> (String, Vec<Value>) {
    let mut sql = String::new();
    let mut binds = Vec::new();
    if let Some(from) = from {
        sql.push_str(" AND e.entry_date >= ?");
        binds.push(Value::Text(from.to_owned()));
    }
    if let Some(to) = to {
        sql.push_str(" AND e.entry_date <= ?");
        binds.push(Value::Text(to.to_owned()));
    }
    (sql, binds)
}

/// True if every entry in the file is posted (a draft only exists inside a save). Uses the index
/// of unposted entries, so it is instant.
fn no_drafts(conn: &Connection) -> rusqlite::Result<bool> {
    let any: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM journal_entries WHERE status <> 'posted')",
        [],
        |r| r.get(0),
    )?;
    Ok(!any)
}

/// The count of entries, Σ in and Σ out of the entries in a range. `from` and `to` are
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
    let what = format!("cash {cash_id} {from:?} {to:?}");
    let sums = memo::sums(conn, &what, || {
        if from.is_none() && to.is_none() && no_drafts(conn)? {
            // The whole book: the covering index of the account alone, no join with the entries.
            return conn.query_row(
                "SELECT COUNT(DISTINCT entry_id), COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0)
                 FROM journal_lines WHERE account_id = ?1",
                [cash_id],
                |r| {
                    Ok(Sums {
                        count: r.get(0)?,
                        dr_paisa: r.get(1)?,
                        cr_paisa: r.get(2)?,
                    })
                },
            );
        }
        let (days, mut binds) = day_conditions(from, to);
        binds.insert(0, Value::Integer(cash_id));
        conn.query_row(
            &format!(
                "SELECT COUNT(DISTINCT e.id), COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
                 FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
                 JOIN journal_lines l ON l.entry_id = e.id AND l.account_id = ?
                 WHERE e.status = 'posted'{days}"
            ),
            params_from_iter(binds.iter()),
            |r| {
                Ok(Sums {
                    count: r.get(0)?,
                    dr_paisa: r.get(1)?,
                    cr_paisa: r.get(2)?,
                })
            },
        )
    })?;
    Ok(CashRange {
        count: sums.count,
        in_paisa: sums.dr_paisa,
        out_paisa: sums.cr_paisa,
    })
}

/// The part of an entry's row the page query reads: its sums on cash and its own columns.
struct Part {
    entry_id: i64,
    entry_date: String,
    posting_type: String,
    in_paisa: i64,
    out_paisa: i64,
    number: Option<i64>,
    narration: String,
    reason: String,
}

/// The sums on cash and the own columns of these entries, one query.
fn parts_of(conn: &Connection, cash_id: i64, ids: &[i64]) -> rusqlite::Result<HashMap<i64, Part>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let marks = vec!["?"; ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT e.id, e.entry_date, e.posting_type, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0),
                CASE e.source_type
                    WHEN 'voucher' THEN (SELECT v.number FROM vouchers v WHERE v.id = e.source_id)
                    WHEN 'bikri' THEN (SELECT b.number FROM bikri_entries b WHERE b.id = e.source_id)
                END,
                COALESCE(e.narration, ''),
                CASE e.source_type
                    WHEN 'voucher' THEN COALESCE((SELECT v.reason FROM vouchers v WHERE v.id = e.source_id), '')
                    ELSE ''
                END
         FROM journal_entries e JOIN journal_lines l ON l.entry_id = e.id AND l.account_id = ?
         WHERE e.id IN ({marks}) GROUP BY e.id"
    ))?;
    let mut values: Vec<i64> = Vec::with_capacity(ids.len().saturating_add(1));
    values.push(cash_id);
    values.extend_from_slice(ids);
    let rows = statement.query_map(params_from_iter(values.iter()), |r| {
        Ok(Part {
            entry_id: r.get(0)?,
            entry_date: r.get(1)?,
            posting_type: r.get(2)?,
            in_paisa: r.get(3)?,
            out_paisa: r.get(4)?,
            number: r.get(5)?,
            narration: r.get(6)?,
            reason: r.get(7)?,
        })
    })?;
    for part in rows {
        let part = part?;
        out.insert(part.entry_id, part);
    }
    Ok(out)
}

/// Rows in the order of `ids`, with the running sums given by `running` (or none).
fn rows_in_order(
    parts: &mut HashMap<i64, Part>,
    ids: &[i64],
    mut running: impl FnMut(&Part) -> rusqlite::Result<(Option<i64>, Option<i64>)>,
) -> rusqlite::Result<Vec<CashRow>> {
    let mut rows = Vec::with_capacity(ids.len());
    for id in ids {
        let Some(part) = parts.remove(id) else {
            continue;
        };
        let (run_in_paisa, run_out_paisa) = running(&part)?;
        rows.push(CashRow {
            entry_id: part.entry_id,
            entry_date: part.entry_date,
            posting_type: part.posting_type,
            in_paisa: part.in_paisa,
            out_paisa: part.out_paisa,
            run_in_paisa,
            run_out_paisa,
            number: part.number,
            narration: part.narration,
            reason: part.reason,
        });
    }
    Ok(rows)
}

fn overflow() -> rusqlite::Error {
    rusqlite::Error::IntegralValueOutOfRange(0, i64::MAX)
}

/// Σ in and Σ out of the cash entries in a range whose (day, id) is on one side of `key`: `later`
/// true is after it, false before it.
fn sums_beside(
    conn: &Connection,
    cash_id: i64,
    from: Option<&str>,
    to: Option<&str>,
    key: (&str, i64),
    later: bool,
) -> rusqlite::Result<(i64, i64)> {
    let (days, mut binds) = day_conditions(from, to);
    let op = if later { ">" } else { "<" };
    let mut all = vec![Value::Integer(cash_id)];
    all.append(&mut binds);
    all.push(Value::Text(key.0.to_owned()));
    all.push(Value::Text(key.0.to_owned()));
    all.push(Value::Integer(key.1));
    conn.query_row(
        &format!(
            "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
             FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
             JOIN journal_lines l ON l.entry_id = e.id AND l.account_id = ?
             WHERE e.status = 'posted'{days}
               AND (e.entry_date {op} ? OR (e.entry_date = ? AND e.id {op} ?))"
        ),
        params_from_iter(all.iter()),
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
}

/// One page of the entries in day order, with the running sums. See the module doc.
fn page_by_date(
    conn: &Connection,
    cash_id: i64,
    (from, to): (Option<&str>, Option<&str>),
    desc: bool,
    (offset, limit): (i64, i64),
) -> rusqlite::Result<Vec<CashRow>> {
    let (days, mut binds) = day_conditions(from, to);
    let direction = if desc { "DESC" } else { "ASC" };
    // Values in the order of the `?` marks: the day conditions, the cash account, limit, offset.
    binds.push(Value::Integer(cash_id));
    binds.push(Value::Integer(limit));
    binds.push(Value::Integer(offset));
    let mut statement = conn.prepare(&format!(
        "SELECT e.id, e.entry_date FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
         WHERE e.status = 'posted'{days}
           AND EXISTS (SELECT 1 FROM journal_lines l WHERE l.account_id = ? AND l.entry_id = e.id)
         ORDER BY e.entry_date {direction}, e.id {direction} LIMIT ? OFFSET ?"
    ))?;
    let keys: Vec<(i64, String)> = statement
        .query_map(params_from_iter(binds.iter()), |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<Result<_, _>>()?;
    let (Some(first), Some(last)) = (keys.first(), keys.last()) else {
        return Ok(Vec::new());
    };
    let ids: Vec<i64> = keys.iter().map(|k| k.0).collect();
    let mut parts = parts_of(conn, cash_id, &ids)?;
    // The sums of the entries before the first row of the page in the order walked, from the
    // nearer end of the book.
    let total = range(conn, cash_id, from, to)?;
    let page_in: i64 = ids
        .iter()
        .filter_map(|id| parts.get(id))
        .try_fold(0_i64, |sum, p| sum.checked_add(p.in_paisa))
        .ok_or_else(overflow)?;
    let page_out: i64 = ids
        .iter()
        .filter_map(|id| parts.get(id))
        .try_fold(0_i64, |sum, p| sum.checked_add(p.out_paisa))
        .ok_or_else(overflow)?;
    // `walked_before` is the sums of the entries the walk passed before the page.
    let walked_before: (i64, i64) = if offset <= total.count / 2 {
        // `desc` walks from the newest, so what is before the page is what is later in day order.
        sums_beside(conn, cash_id, from, to, (&first.1, first.0), desc)?
    } else {
        let after = sums_beside(conn, cash_id, from, to, (&last.1, last.0), !desc)?;
        (
            total
                .in_paisa
                .checked_sub(page_in)
                .and_then(|v| v.checked_sub(after.0))
                .ok_or_else(overflow)?,
            total
                .out_paisa
                .checked_sub(page_out)
                .and_then(|v| v.checked_sub(after.1))
                .ok_or_else(overflow)?,
        )
    };
    // The running sums in day order. Oldest first: the walked entries are the earlier ones. Newest
    // first: the first row has everything except what was walked before it.
    let (mut run_in, mut run_out) = if desc {
        (
            total
                .in_paisa
                .checked_sub(walked_before.0)
                .ok_or_else(overflow)?,
            total
                .out_paisa
                .checked_sub(walked_before.1)
                .ok_or_else(overflow)?,
        )
    } else {
        walked_before
    };
    rows_in_order(&mut parts, &ids, |part| {
        if desc {
            // This row's running sums include itself; the next (older) row takes it off.
            let out = (Some(run_in), Some(run_out));
            run_in = run_in.checked_sub(part.in_paisa).ok_or_else(overflow)?;
            run_out = run_out.checked_sub(part.out_paisa).ok_or_else(overflow)?;
            Ok(out)
        } else {
            run_in = run_in.checked_add(part.in_paisa).ok_or_else(overflow)?;
            run_out = run_out.checked_add(part.out_paisa).ok_or_else(overflow)?;
            Ok((Some(run_in), Some(run_out)))
        }
    })
}

/// One page of the entries by the cash received or paid out, biggest first or smallest first.
fn page_by_amount(
    conn: &Connection,
    cash_id: i64,
    (from, to): (Option<&str>, Option<&str>),
    (out, desc): (bool, bool),
    (offset, limit): (i64, i64),
) -> rusqlite::Result<Vec<CashRow>> {
    let column = if out { "o" } else { "i" };
    let direction = if desc { "DESC" } else { "ASC" };
    let mut binds = vec![Value::Integer(cash_id)];
    // The covering index alone gives the sums of an entry; the entries are joined only when a day
    // is asked for or a draft exists.
    let join = if from.is_none() && to.is_none() && no_drafts(conn)? {
        String::new()
    } else {
        let (days, day_binds) = day_conditions(from, to);
        binds.extend(day_binds);
        format!(" JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'{days}")
    };
    // The cash account id is the first `?` only when there is no join: with the join the day values
    // come first in the text, so the id goes first there too (see the order of the clauses below).
    let sql = format!(
        "SELECT l.entry_id, SUM(l.dr_paisa) AS i, SUM(l.cr_paisa) AS o
         FROM journal_lines l{join}
         WHERE l.account_id = ?
         GROUP BY l.entry_id ORDER BY {column} {direction}, l.entry_id {direction} LIMIT ? OFFSET ?"
    );
    // Values in the order of the `?` marks: the join's day values, then the account, limit, offset.
    let mut ordered: Vec<Value> = binds.iter().skip(1).cloned().collect();
    ordered.push(Value::Integer(cash_id));
    ordered.push(Value::Integer(limit));
    ordered.push(Value::Integer(offset));
    let mut statement = conn.prepare(&sql)?;
    let ids: Vec<i64> = statement
        .query_map(params_from_iter(ordered.iter()), |r| r.get::<_, i64>(0))?
        .collect::<Result<_, _>>()?;
    let mut parts = parts_of(conn, cash_id, &ids)?;
    rows_in_order(&mut parts, &ids, |_| Ok((None, None)))
}

/// One page of the entries sorted by another column. This reads every entry of the range: the slow
/// path (posting type, number, narration).
fn page_by_other(
    conn: &Connection,
    cash_id: i64,
    (from, to): (Option<&str>, Option<&str>),
    order: &str,
    (offset, limit): (i64, i64),
) -> rusqlite::Result<Vec<CashRow>> {
    let (days, day_binds) = day_conditions(from, to);
    let mut binds = vec![Value::Integer(cash_id)];
    binds.extend(day_binds);
    binds.push(Value::Integer(limit));
    binds.push(Value::Integer(offset));
    let mut statement = conn.prepare(&format!(
        "WITH r AS (
             SELECT e.id AS entry_id, e.entry_date AS entry_date, e.posting_type AS posting_type,
                    e.source_type AS source_type, e.source_id AS source_id,
                    COALESCE(e.narration, '') AS narration,
                    SUM(l.dr_paisa) AS in_p, SUM(l.cr_paisa) AS out_p
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE l.account_id = ? AND e.status = 'posted'{days}
             GROUP BY e.id)
         SELECT r.entry_id FROM r ORDER BY {order}, r.entry_date, r.entry_id LIMIT ? OFFSET ?"
    ))?;
    let ids: Vec<i64> = statement
        .query_map(params_from_iter(binds.iter()), |r| r.get::<_, i64>(0))?
        .collect::<Result<_, _>>()?;
    let mut parts = parts_of(conn, cash_id, &ids)?;
    rows_in_order(&mut parts, &ids, |_| Ok((None, None)))
}

/// One page of the entries of the range in the order asked for. Only [`Order::Date`] has the
/// running sums.
///
/// # Errors
/// Any SQLite error.
pub fn page(
    conn: &Connection,
    cash_id: i64,
    from: Option<&str>,
    to: Option<&str>,
    order: &Order,
    offset: i64,
    limit: i64,
) -> rusqlite::Result<Vec<CashRow>> {
    match order {
        Order::Date { desc } => page_by_date(conn, cash_id, (from, to), *desc, (offset, limit)),
        Order::Amount { out, desc } => {
            page_by_amount(conn, cash_id, (from, to), (*out, *desc), (offset, limit))
        }
        Order::Other(sql) => page_by_other(conn, cash_id, (from, to), sql, (offset, limit)),
    }
}

/// The count of entries and the cash in and out of each month from the month of `first_day` to the
/// month of `last_day` (both `YYYY-MM-DD`), clipped to the range: one query for each month, each
/// read through the day index. These are the months a page touches (plan 38 B2).
///
/// # Errors
/// Any SQLite error.
pub fn months_between(
    conn: &Connection,
    cash_id: i64,
    (from, to): (Option<&str>, Option<&str>),
    (first_day, last_day): (&str, &str),
) -> rusqlite::Result<Vec<MonthSums>> {
    let (low, high) = if first_day <= last_day {
        (first_day, last_day)
    } else {
        (last_day, first_day)
    };
    let (Some(start), Some(end)) = (low.get(..7), high.get(..7)) else {
        return Ok(Vec::new());
    };
    let parse = |m: &str| -> Option<(i32, u32)> {
        Some((m.get(..4)?.parse().ok()?, m.get(5..7)?.parse().ok()?))
    };
    let (Some((mut year, mut month)), Some(last)) = (parse(start), parse(end)) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    // A page cannot touch more months than this (a 100-row page of a sparse book).
    for _ in 0..240 {
        let key = format!("{year:04}-{month:02}");
        let lo = format!("{key}-01");
        let hi = format!("{key}-31");
        let lo = from.map_or(
            lo.clone(),
            |f| if f > lo.as_str() { f.to_owned() } else { lo },
        );
        let hi = to.map_or(
            hi.clone(),
            |t| if t < hi.as_str() { t.to_owned() } else { hi },
        );
        let sums = range(conn, cash_id, Some(&lo), Some(&hi))?;
        // A month with no entry gets no heading, as when the months were grouped from the rows.
        if sums.count > 0 {
            out.push(MonthSums {
                month: key,
                count: sums.count,
                sums: vec![sums.in_paisa, sums.out_paisa],
            });
        }
        if (year, month) >= last {
            break;
        }
        if month == 12 {
            year += 1;
            month = 1;
        } else {
            month += 1;
        }
    }
    Ok(out)
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
    /// Its tag (نشان), if any (PRD 10.2).
    pub tag: Option<String>,
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
        "SELECT l.entry_id, l.account_id, a.code, a.name_ur, a.name_en, l.dr_paisa > 0, a.tag
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
            tag: r.get(6)?,
        })
    })?;
    for row in rows {
        let row = row?;
        out.entry(row.entry_id).or_default().push(row);
    }
    Ok(out)
}
