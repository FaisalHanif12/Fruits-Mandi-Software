//! R29 خرچہ رپورٹ (the expense report) (PRD 19.2, 16.2, 16.3, FR-16.2, FR-17.3, FR-19.10).
//!
//! Two parts, one under the other:
//!
//! * the shop expenses (دکان خرچہ), one row for each expense head with its total in the filter,
//!   read from the journal lines of the head 05 accounts (debit less credit); and
//! * the money each partner took for himself (ذاتی خرچہ) and gave back (واپسی), read from the
//!   lines of his drawings account in live entries (PRD FR-16.2). A cancelled entry and its
//!   reversal are both left out.
//!
//! The shop expense total is the same figure as the expenses line of R30 (a test holds the two
//! together), and a partner's taken and returned are the same figures as the columns of R30.
//! The entries of a month close are left out: they only move amounts between accounts
//! (PRD FR-17.6).
//!
//! With a partner picked, the report shows only that partner's money taken and returned: the
//! shop expenses belong to the shop, not to him.
//!
//! Example (PRD 4.9, E2E-9): for 10-10-26 the shop expense بجلی is 3,000 and الف took 2,000 and
//! returned 0. The totals row reads 3,000, 2,000, 0.

use mandi_core::money::Paisa;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{Conds, LIVE, add, col, count_of, names_cell, paisa};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{Cell, ColumnKind, ReportId, ReportPage, Row, RowKind};

/// The shop expense heads with a movement in the filter. A head is an account of head 05.
fn heads_query(typed: &Typed) -> (String, Vec<Value>) {
    let mut c = Conds::new();
    c.always("e.status = 'posted'")
        .always("e.source_type <> 'month_close'")
        .always("h.code = '05'")
        .text("e.entry_date >= ?", typed.from.as_deref())
        .text("e.entry_date <= ?", typed.to.as_deref());
    let sql = format!(
        "SELECT a.code, a.name_ur, a.name_en,
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls c ON c.id = a.control_id
         JOIN account_heads h ON h.id = c.head_id
         WHERE {} GROUP BY a.id ORDER BY a.code",
        c.sql()
    );
    (sql, c.args().to_vec())
}

/// What each partner took and gave back: the lines of his drawings account in live entries.
fn partners_query(typed: &Typed) -> (String, Vec<Value>) {
    let mut c = Conds::new();
    c.always(LIVE)
        .always("e.source_type <> 'month_close'")
        .id("p.id = ?", typed.partner_id)
        .text("e.entry_date >= ?", typed.from.as_deref())
        .text("e.entry_date <= ?", typed.to.as_deref());
    let sql = format!(
        "SELECT p.name_ur, p.name_en, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM partners p
         JOIN journal_lines l ON l.account_id = p.drawings_account_id
         JOIN journal_entries e ON e.id = l.entry_id
         WHERE {} GROUP BY p.id ORDER BY p.id",
        c.sql()
    );
    (sql, c.args().to_vec())
}

/// A heading row: the label and the totals of the rows under it, in the columns they use.
fn heading(
    label: &str,
    expense: Option<Paisa>,
    taken: Option<Paisa>,
    returned: Option<Paisa>,
) -> Row {
    let cell = |amount: Option<Paisa>| amount.map_or(Cell::Empty, |paisa| Cell::Money { paisa });
    Row {
        cells: vec![
            Cell::Label {
                id: label.to_owned(),
            },
            Cell::Empty,
            cell(expense),
            cell(taken),
            cell(returned),
        ],
        source: None,
        kind: RowKind::Group,
    }
}

/// Builds R29.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a total
/// does not fit.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let mut all_rows = Vec::new();
    let (mut expense_total, mut taken_total, mut returned_total) =
        (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);

    // The shop expenses, unless one partner is picked.
    if typed.partner_id.is_none() {
        let (sql, args) = heads_query(typed);
        let mut statement = conn.prepare(&sql)?;
        let heads = statement
            .query_map(params_from_iter(args.iter()), |r| {
                Ok((
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut rows = Vec::with_capacity(heads.len());
        for (name_ur, name_en, dr, cr) in heads {
            // A shop expense is a debit; a credit on its head takes some back.
            let amount = paisa(dr)?.checked_sub(paisa(cr)?)?;
            add(&mut expense_total, amount)?;
            rows.push(Row {
                cells: vec![
                    Cell::Empty,
                    names_cell(Some(name_ur), Some(name_en)),
                    Cell::Money { paisa: amount },
                    Cell::Empty,
                    Cell::Empty,
                ],
                source: None,
                kind: RowKind::Normal,
            });
        }
        if !rows.is_empty() {
            all_rows.push(heading(
                ids::SECTION_SHOP_EXPENSES,
                Some(expense_total),
                None,
                None,
            ));
            all_rows.extend(rows);
        }
    }

    // The money partners took and gave back.
    {
        let (sql, args) = partners_query(typed);
        let mut statement = conn.prepare(&sql)?;
        let partners = statement
            .query_map(params_from_iter(args.iter()), |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut rows = Vec::with_capacity(partners.len());
        for (name_ur, name_en, dr, cr) in partners {
            let (taken, returned) = (paisa(dr)?, paisa(cr)?);
            if taken == Paisa::ZERO && returned == Paisa::ZERO {
                continue;
            }
            add(&mut taken_total, taken)?;
            add(&mut returned_total, returned)?;
            rows.push(Row {
                cells: vec![
                    Cell::Empty,
                    names_cell(Some(name_ur), Some(name_en)),
                    Cell::Empty,
                    Cell::Money { paisa: taken },
                    Cell::Money { paisa: returned },
                ],
                source: None,
                kind: RowKind::Normal,
            });
        }
        if !rows.is_empty() {
            all_rows.push(heading(
                ids::SECTION_PARTNER_MONEY,
                None,
                Some(taken_total),
                Some(returned_total),
            ));
            all_rows.extend(rows);
        }
    }

    Ok(ReportPage {
        id: ReportId::R29,
        title_id: ids::title_id("R29"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("section", ids::COL_SECTION, ColumnKind::Label),
            col("name", ids::COL_NAME, ColumnKind::Names),
            col("expense", ids::COL_EXPENSE, ColumnKind::Money),
            col("taken", ids::COL_TAKEN, ColumnKind::Money),
            col("returned", ids::COL_RETURNED, ColumnKind::Money),
        ],
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count_of(all_rows.len()),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: expense_total,
            },
            Cell::Money { paisa: taken_total },
            Cell::Money {
                paisa: returned_total,
            },
        ],
        summary: Vec::new(),
    })
}
