//! R30 منافع و تقسیم (profit and partner split) (PRD 19.2, FR-6.5, FR-16.3, FR-17.3, C12, C20).
//!
//! Income less shop expenses is the profit; each partner's share of it is read from the partner
//! share rows the postings wrote (PRD FR-17.4): his income share, his expense share, his profit
//! share (C12), the money he took and returned (his drawings account, FR-16.2) and his خالص
//! (C20), profit share less what he took and did not return. The share rows are append-only and
//! a reversal writes the opposite rows, so a cancelled posting adds nothing.
//!
//! The lines above the table are the income, the shop expenses and the profit of the filter,
//! read from the journal (heads 04 and 05, without the entries of a month close, which only move
//! amounts between accounts). The income shares of all partners add up to the income, and their
//! profit shares to the profit, because every posting splits exactly (PRD FR-6.5); a test
//! checks both.
//!
//! A money amount here can be below 0 (a loss, a partner who took more than his profit share);
//! it is a plain amount with a minus sign, not a side.
//!
//! Example (PRD 4.9, E2E-9): الف income share 2,196, expense share 1,200, profit share 996,
//! taken 2,000, returned 0, خالص −1,004; ب and ج 1,646, 900, 746, 0, 0, 746. The lines above:
//! income 5,488, expenses 3,000, profit 2,488 = 996 + 746 + 746.

use std::collections::HashMap;

use mandi_core::money::Paisa;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{Conds, LIVE, add, col, count_of, names_cell, paisa};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{Cell, ColumnKind, ReportId, ReportPage, Row, RowKind, SummaryLine};

/// The income and expense shares of each partner, with only the dates that are set.
fn shares_query(typed: &Typed) -> (String, Vec<Value>) {
    let mut c = Conds::new();
    c.text("entry_date >= ?", typed.from.as_deref())
        .text("entry_date <= ?", typed.to.as_deref());
    let sql = format!(
        "SELECT partner_id,
            COALESCE(SUM(CASE WHEN kind = 'income' THEN amount_paisa END), 0),
            COALESCE(SUM(CASE WHEN kind = 'expense' THEN amount_paisa END), 0),
            COUNT(*)
         FROM partner_share_rows WHERE {} GROUP BY partner_id",
        c.sql()
    );
    (sql, c.args().to_vec())
}

/// What each partner took and returned: the lines of his drawings account in live entries.
fn drawings_query(typed: &Typed) -> (String, Vec<Value>) {
    let mut c = Conds::new();
    c.always(LIVE)
        .text("e.entry_date >= ?", typed.from.as_deref())
        .text("e.entry_date <= ?", typed.to.as_deref());
    let sql = format!(
        "SELECT p.id, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM partners p
         JOIN journal_lines l ON l.account_id = p.drawings_account_id
         JOIN journal_entries e ON e.id = l.entry_id
         WHERE {} GROUP BY p.id",
        c.sql()
    );
    (sql, c.args().to_vec())
}

/// The income and the shop expenses of the filter, from the journal.
fn heads_query(typed: &Typed) -> (String, Vec<Value>) {
    let mut c = Conds::new();
    c.always("e.status = 'posted'")
        .always("e.source_type <> 'month_close'")
        .always("h.code IN ('04', '05')")
        .text("e.entry_date >= ?", typed.from.as_deref())
        .text("e.entry_date <= ?", typed.to.as_deref());
    let sql = format!(
        "SELECT h.code, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls c ON c.id = a.control_id
         JOIN account_heads h ON h.id = c.head_id
         WHERE {} GROUP BY h.code",
        c.sql()
    );
    (sql, c.args().to_vec())
}

/// The partners. A partner who left but has rows still shows.
const PARTNERS_SQL: &str = "SELECT id, name_ur, name_en, is_active FROM partners ORDER BY id";

/// Builds R30.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a total
/// does not fit.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let shares: HashMap<i64, (i64, i64, i64)> = {
        let (sql, args) = shares_query(typed);
        let mut statement = conn.prepare(&sql)?;
        statement
            .query_map(params_from_iter(args.iter()), |r| {
                Ok((r.get::<_, i64>(0)?, (r.get(1)?, r.get(2)?, r.get(3)?)))
            })?
            .collect::<Result<_, _>>()?
    };
    let drawings: HashMap<i64, (i64, i64)> = {
        let (sql, args) = drawings_query(typed);
        let mut statement = conn.prepare(&sql)?;
        statement
            .query_map(params_from_iter(args.iter()), |r| {
                Ok((r.get::<_, i64>(0)?, (r.get(1)?, r.get(2)?)))
            })?
            .collect::<Result<_, _>>()?
    };
    let partners = {
        let mut statement = conn.prepare(PARTNERS_SQL)?;
        statement
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)? == 1,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?
    };

    let mut sums = [Paisa::ZERO; 6];
    let mut all_rows = Vec::new();
    for (id, name_ur, name_en, active) in partners {
        let (income_raw, expense_raw, share_rows) = shares.get(&id).copied().unwrap_or((0, 0, 0));
        let (taken_raw, returned_raw) = drawings.get(&id).copied().unwrap_or((0, 0));
        // A partner who left and has nothing in the filter has no row.
        if !active && share_rows == 0 && taken_raw == 0 && returned_raw == 0 {
            continue;
        }
        let income = paisa(income_raw)?;
        // Expense shares are stored below 0; the column shows what the partner bore.
        let expense = paisa(expense_raw)?.checked_neg()?;
        let profit = income.checked_sub(expense)?;
        let taken = paisa(taken_raw)?;
        let returned = paisa(returned_raw)?;
        let net = profit.checked_sub(taken.checked_sub(returned)?)?;
        for (slot, amount) in sums
            .iter_mut()
            .zip([income, expense, profit, taken, returned, net])
        {
            add(slot, amount)?;
        }
        let money = |paisa| Cell::Money { paisa };
        all_rows.push(Row {
            cells: vec![
                names_cell(Some(name_ur), Some(name_en)),
                money(income),
                money(expense),
                money(profit),
                money(taken),
                money(returned),
                money(net),
            ],
            source: None,
            kind: RowKind::Normal,
        });
    }

    // Income and shop expenses of the filter, from the journal.
    let (mut income_total, mut expense_total) = (Paisa::ZERO, Paisa::ZERO);
    {
        let (sql, args) = heads_query(typed);
        let mut statement = conn.prepare(&sql)?;
        let heads = statement
            .query_map(params_from_iter(args.iter()), |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (code, dr, cr) in heads {
            let (dr, cr) = (paisa(dr)?, paisa(cr)?);
            match code.as_str() {
                // Income: credit less debit.
                "04" => income_total = cr.checked_sub(dr)?,
                // Shop expenses: debit less credit.
                _ => expense_total = dr.checked_sub(cr)?,
            }
        }
    }
    let profit_total = income_total.checked_sub(expense_total)?;

    let [income, expense, profit, taken, returned, net] = sums;
    let money = |paisa| Cell::Money { paisa };
    Ok(ReportPage {
        group_above: None,
        id: ReportId::R30,
        title_id: ids::title_id("R30"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("partner", ids::COL_PARTNER, ColumnKind::Names),
            col("income_share", ids::COL_INCOME_SHARE, ColumnKind::Money),
            col("expense_share", ids::COL_EXPENSE_SHARE, ColumnKind::Money),
            col("profit_share", ids::COL_PROFIT_SHARE, ColumnKind::Money),
            col("taken", ids::COL_TAKEN, ColumnKind::Money),
            col("returned", ids::COL_RETURNED, ColumnKind::Money),
            col("net", ids::COL_PARTNER_NET, ColumnKind::Money),
        ],
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count_of(all_rows.len()),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            money(income),
            money(expense),
            money(profit),
            money(taken),
            money(returned),
            money(net),
        ],
        summary: vec![
            SummaryLine {
                label_id: ids::R30_INCOME.to_owned(),
                cell: money(income_total),
            },
            SummaryLine {
                label_id: ids::R30_EXPENSES.to_owned(),
                cell: money(expense_total),
            },
            SummaryLine {
                label_id: ids::R30_PROFIT.to_owned(),
                cell: money(profit_total),
            },
        ],
    })
}
