//! R28 آمدن رپورٹ (the income report): each income account by day or by month (PRD 19.2,
//! FR-19.10, FR-17.3).
//!
//! The income accounts are the accounts of head 04: the commission on each side, لاگا, the phone
//! charge and ریٹ فرق. For each period and account the amount is the posted `Σ جمع − Σ بنام` of
//! its lines, shown on its side (a loss on ریٹ فرق shows as بنام). The entries of a month close
//! move the month's income into the partners' capital; they are not income earned, so they are
//! left out here, and the report shows the same figures before and after a month close.
//!
//! The rows are found by period then account code; the page and the totals are two queries over
//! the whole filter (Code Standards B31). Every sum is a `SUM()` of stored `INTEGER` columns.
//!
//! Example (the worked day, by day): 10-10-2026 کمیشن آمدن (بیوپاری) 2,484 جمع, کمیشن آمدن (گاہک)
//! 2,484, لاگا آمدن 480, فون آمدن 40; the total row 5,488 جمع (PRD 4.9).

use mandi_core::accounts::Balance;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{Conds, balance_cell, col, load_parties, paisa, party_of_line};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, Column, ColumnKind, GroupBy, ReportId, ReportPage, Row, RowKind,
};

/// The conditions of R28, with only the filters that are set (Code Standards B18). The entries of
/// a month close are left out: they move income into capital, they are not income earned.
fn conds(typed: &Typed) -> Conds {
    let mut c = Conds::new();
    c.always("e.status = 'posted'")
        .always("e.source_type <> 'month_close'")
        .always("h.code = '04'")
        .text("e.entry_date >= ?", typed.from.as_deref())
        .text("e.entry_date <= ?", typed.to.as_deref())
        .id("l.account_id = ?", typed.party_id);
    c
}

/// The length of the period text: 10 for a day (`2026-10-10`), 7 for a month (`2026-10`). A
/// constant of this crate, written into the query as a number.
const fn period_length(group_by: GroupBy) -> u8 {
    match group_by {
        GroupBy::Day => 10,
        GroupBy::Month => 7,
    }
}

const JOINS: &str = "FROM journal_lines l
     JOIN journal_entries e ON e.id = l.entry_id
     JOIN accounts a ON a.id = l.account_id
     JOIN account_controls c ON c.id = a.control_id
     JOIN account_heads h ON h.id = c.head_id";

/// The tables with the entries of the range first, by the date index of the entries.
const JOINS_ENTRIES_FIRST: &str = "FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
     CROSS JOIN journal_lines l ON l.entry_id = e.id
     CROSS JOIN accounts a ON a.id = l.account_id
     CROSS JOIN account_controls c ON c.id = a.control_id
     CROSS JOIN account_heads h ON h.id = c.head_id";

/// The same tables with the income accounts first: the lines of the few income accounts are read
/// from the covering index of the amounts, and each line's entry is looked up for its day and kind.
/// For a long range (ten years) this reads 780,000 lines instead of every line of 1.08 million
/// entries (plan 80 gate: 4 seconds down to 0.7). `CROSS JOIN` fixes the order, as the app's file
/// has no table statistics.
const JOINS_INCOME_FIRST: &str = "FROM account_heads h
     CROSS JOIN account_controls c ON c.head_id = h.id
     CROSS JOIN accounts a ON a.control_id = c.id
     CROSS JOIN journal_lines l INDEXED BY idx_journal_lines_account_amounts ON l.account_id = a.id
     CROSS JOIN journal_entries e ON e.id = l.entry_id";

/// A date range with this many entries or more reads the income accounts first (see
/// [`JOINS_INCOME_FIRST`]): the lines of the entries of a shorter range are fewer than the lines of
/// the income accounts.
const WIDE_ENTRIES: i64 = 60_000;

/// The periods and accounts of the whole filter, in the order of the report, with no page cut. The
/// SQL of the page and of the totals together: the count, the totals and every page come from this
/// one read (plan 80 gate: three passes over the lines became one).
pub(crate) fn grouped_query(typed: &Typed, income_first: bool) -> (String, Vec<Value>) {
    let c = conds(typed);
    let len = period_length(typed.group_by);
    // The entries first, found through the date index, when the range has a first or a last day; the
    // app's file has no table statistics, so without this SQLite starts from the income accounts.
    let dated = typed.from.is_some() || typed.to.is_some();
    let joins = if income_first {
        JOINS_INCOME_FIRST
    } else if dated {
        JOINS_ENTRIES_FIRST
    } else {
        JOINS
    };
    let sql = format!(
        "SELECT substr(e.entry_date, 1, {len}), l.account_id, COALESCE(SUM(l.dr_paisa), 0),
            COALESCE(SUM(l.cr_paisa), 0)
         {joins}
         WHERE {}
         GROUP BY substr(e.entry_date, 1, {len}), l.account_id
         ORDER BY substr(e.entry_date, 1, {len}), a.code",
        c.sql()
    );
    (sql, c.args().to_vec())
}

/// True when the date range of the filter holds at least [`WIDE_ENTRIES`] journal entries. Reads at
/// most that many entries of the date index.
fn is_wide(conn: &Connection, typed: &Typed) -> Result<bool, ReportsError> {
    let mut sql = String::from("SELECT COUNT(*) FROM (SELECT 1 FROM journal_entries WHERE 1 = 1");
    let mut args: Vec<Value> = Vec::new();
    if let Some(from) = &typed.from {
        sql.push_str(" AND entry_date >= ?");
        args.push(Value::Text(from.clone()));
    }
    if let Some(to) = &typed.to {
        sql.push_str(" AND entry_date <= ?");
        args.push(Value::Text(to.clone()));
    }
    sql.push_str(" LIMIT ?)");
    args.push(Value::Integer(WIDE_ENTRIES));
    let counted: i64 = conn.query_row(&sql, params_from_iter(args.iter()), |r| r.get(0))?;
    Ok(counted >= WIDE_ENTRIES)
}

/// One period and account of the report with its sums.
type Group = (String, i64, i64, i64);

/// Every period and account of the filter, in the order of the report.
fn groups_of(conn: &Connection, typed: &Typed) -> Result<std::sync::Arc<Vec<Group>>, ReportsError> {
    let (sql, args) = grouped_query(typed, is_wide(conn, typed)?);
    let what = format!("r28_groups|{sql}|{args:?}");
    crate::result_cache::remembered(conn, &what, || {
        let mut statement = conn.prepare(&sql)?;
        Ok(statement
            .query_map(params_from_iter(args.iter()), |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?)
    })
}

/// One page of periods and accounts.
pub(crate) fn page_query(typed: &Typed, limit: i64, offset: i64) -> (String, Vec<Value>) {
    let c = conds(typed);
    let len = period_length(typed.group_by);
    let sql = format!(
        "SELECT substr(e.entry_date, 1, {len}), l.account_id, COALESCE(SUM(l.dr_paisa), 0),
            COALESCE(SUM(l.cr_paisa), 0)
         {JOINS}
         WHERE {}
         GROUP BY substr(e.entry_date, 1, {len}), l.account_id
         ORDER BY substr(e.entry_date, 1, {len}), a.code
         LIMIT ? OFFSET ?",
        c.sql()
    );
    (sql, c.page_args(limit, offset))
}

/// The cell of a period text: `2026-10-10` is a date, `2026-10` a month.
fn period_cell(text: &str) -> Result<Cell, ReportsError> {
    let bad = || ReportsError::Data(format!("a period is not a date: {text}"));
    if text.len() == 10 {
        return Ok(Cell::Date {
            iso: text.to_owned(),
        });
    }
    let (year, month) = text.split_once('-').ok_or_else(bad)?;
    Ok(Cell::Month {
        year: year.parse().map_err(|_| bad())?,
        month: month.parse().map_err(|_| bad())?,
    })
}

/// The columns of R28 for a grouping: the period is a day or a month. The one definition: the run
/// and the catalogue both use it (plan 75 T1).
pub(crate) fn columns(group_by: GroupBy) -> Vec<Column> {
    let period_kind = match group_by {
        GroupBy::Day => ColumnKind::Date,
        GroupBy::Month => ColumnKind::Month,
    };
    vec![
        col("period", ids::COL_PERIOD, period_kind),
        col("account", ids::COL_ACCOUNT, ColumnKind::Party),
        col("amount", ids::COL_AMOUNT, ColumnKind::Balance),
    ]
}

/// Builds R28.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a total
/// does not fit.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    // The count, the totals and the rows are one read of the periods and accounts of the whole
    // filter. It is the same for every page, so the first page reads it and the next pages take it
    // from memory while the books are as they were (`result_cache`).
    let groups = groups_of(conn, typed)?;
    let count = i64::try_from(groups.len()).unwrap_or(i64::MAX);
    let overflow = || ReportsError::Data("the income total does not fit".to_owned());
    let (mut dr, mut cr) = (0_i64, 0_i64);
    for group in groups.iter() {
        dr = dr.checked_add(group.2).ok_or_else(overflow)?;
        cr = cr.checked_add(group.3).ok_or_else(overflow)?;
    }
    let lines: Vec<Group> = spec.slice(&groups);
    let account_ids: Vec<i64> = lines.iter().map(|l| l.1).collect();
    let parties = load_parties(conn, &account_ids)?;
    let mut rows = Vec::with_capacity(lines.len());
    for (period, account_id, line_dr, line_cr) in lines {
        rows.push(Row {
            cells: vec![
                period_cell(&period)?,
                Cell::Party {
                    party: party_of_line(&parties, account_id, None)?,
                },
                balance_cell(Balance::from_totals(paisa(line_dr)?, paisa(line_cr)?)?),
            ],
            source: None,
            kind: RowKind::Normal,
        });
    }

    // The total is the net income of the whole filter, on its side.
    let net = Balance::from_totals(paisa(dr)?, paisa(cr)?)?;
    Ok(ReportPage {
        group_above: None,
        tables: Vec::new(),
        id: ReportId::R28,
        title_id: ids::title_id("R28"),
        filters: typed.resolved.clone(),
        columns: columns(typed.group_by),
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![Cell::Empty, Cell::Empty, balance_cell(net)],
        summary: Vec::new(),
    })
}
