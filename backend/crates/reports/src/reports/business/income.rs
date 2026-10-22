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
use crate::reports::model::{Cell, ColumnKind, GroupBy, ReportId, ReportPage, Row, RowKind};

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

/// The number of periods and accounts over the whole filter.
fn count_query(typed: &Typed) -> (String, Vec<Value>) {
    let c = conds(typed);
    let len = period_length(typed.group_by);
    let sql = format!(
        "SELECT COUNT(*) FROM (SELECT 1 {JOINS}
         WHERE {}
         GROUP BY substr(e.entry_date, 1, {len}), l.account_id)",
        c.sql()
    );
    (sql, c.args().to_vec())
}

/// The totals over the whole filter.
fn totals_query(typed: &Typed) -> (String, Vec<Value>) {
    let c = conds(typed);
    let sql = format!(
        "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0) {JOINS} WHERE {}",
        c.sql()
    );
    (sql, c.args().to_vec())
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
    let period_kind = match typed.group_by {
        GroupBy::Day => ColumnKind::Date,
        GroupBy::Month => ColumnKind::Month,
    };
    let (count_sql, count_args) = count_query(typed);
    let count: i64 = conn.query_row(&count_sql, params_from_iter(count_args.iter()), |r| {
        r.get(0)
    })?;
    let (totals_sql, totals_args) = totals_query(typed);
    let (dr, cr): (i64, i64) =
        conn.query_row(&totals_sql, params_from_iter(totals_args.iter()), |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?;
    let (page_sql, page_args) = page_query(typed, spec.sql_limit(), spec.offset);
    let mut statement = conn.prepare(&page_sql)?;
    let lines = statement
        .query_map(params_from_iter(page_args.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
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
        id: ReportId::R28,
        title_id: ids::title_id("R28"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("period", ids::COL_PERIOD, period_kind),
            col("account", ids::COL_ACCOUNT, ColumnKind::Party),
            col("amount", ids::COL_AMOUNT, ColumnKind::Balance),
        ],
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![Cell::Empty, Cell::Empty, balance_cell(net)],
        summary: Vec::new(),
    })
}
