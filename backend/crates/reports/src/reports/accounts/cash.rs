//! R12 روکڑ (the cash book) (PRD 19.2, 16.1, FR-16.1, FR-19.10, FR-19.11).
//!
//! One row is one posted journal entry that moves cash: what came in (a debit on the cash
//! account), what went out (a credit), and the cash after it. The opening cash is the balance at
//! the end of the day before the first day of the range, and the closing cash is the opening
//! plus the cash in less the cash out over the whole range, so a page far down still shows the
//! right figure. There is no cash count (PRD FR-16.1): the book shows only what was posted.
//!
//! The money is read the same way as the cash book screen (`mandi_services::cashbook`): the
//! sums are `SUM()` of `INTEGER` columns, the differences are worked out in Rust with checked
//! maths. A test holds the two side by side.
//!
//! Example (PRD 4.9, E2E-8): for 10-10-26 the opening is 50,000, the cash in is 12,800 + 10,000
//! = 22,800, the cash out is 5,000 + 4,316 + 3,000 + 2,000 = 14,316, and the closing is 58,484.

use std::collections::HashMap;

use mandi_core::accounts::Balance;
use mandi_core::dates::BusinessDate;
use mandi_core::money::Paisa;
use mandi_db::accounts as account_repo;
use mandi_db::cashbook as cash_repo;
use mandi_db::ledger::totals_of;
use mandi_db::rusqlite::Connection;

use super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{
    IN_CHUNK, amount_or_empty, balance_cell, col, load_parties, paisa, party_of_line, placeholders,
    source_of, text_cell,
};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{Cell, ColumnKind, ReportId, ReportPage, Row, RowKind, SummaryLine};

/// The shop's one cash account (PRD FR-10.4: all money is روکڑ).
pub(crate) const CASH_ACCOUNT_CODE: &str = "01010001";

/// The one account on the other side of an entry that only takes cash in, or only gives it out.
/// An entry that does both has no single other side, and the entry of a بکری has many.
fn counterpart_of(
    others: &HashMap<i64, Vec<cash_repo::Counterpart>>,
    row: &cash_repo::CashRow,
) -> Option<i64> {
    if row.in_paisa > 0 && row.out_paisa > 0 {
        return None;
    }
    // Cash received (a debit on cash) is matched by credits; cash paid out by debits.
    let cash_received = row.in_paisa > 0;
    let mut found: Option<i64> = None;
    for line in others.get(&row.entry_id)? {
        if line.is_debit == cash_received {
            continue;
        }
        match found {
            Some(first) if first != line.account_id => return None,
            Some(_) => {}
            None => found = Some(line.account_id),
        }
    }
    found
}

/// Builds R12.
///
/// # Errors
/// A database error; `ReportsError::Data` if the cash account is missing;
/// `ReportsError::Money` if a stored amount is not whole rupees or a total does not fit.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let cash = account_repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ReportsError::Data("the cash account is missing".to_owned()))?;
    let from = typed.from.as_deref();
    let to = typed.to.as_deref();

    // The cash at the end of the day before the first day. Without a first day the book starts
    // at the first entry and the opening is 0.
    let (open_dr, open_cr) = match from
        .and_then(BusinessDate::parse_iso)
        .and_then(BusinessDate::previous_day)
    {
        Some(day) => {
            let t = totals_of(conn, cash, None, Some(&day.to_iso()))?;
            (paisa(t.dr_paisa)?, paisa(t.cr_paisa)?)
        }
        None => (Paisa::ZERO, Paisa::ZERO),
    };
    let opening = Balance::from_totals(open_dr, open_cr)?;

    let range = cash_repo::range(conn, cash, from, to)?;
    let (total_in, total_out) = (paisa(range.in_paisa)?, paisa(range.out_paisa)?);
    let closing = Balance::from_totals(
        open_dr.checked_add(total_in)?,
        open_cr.checked_add(total_out)?,
    )?;

    let stored = cash_repo::page(conn, cash, from, to, spec.offset, spec.sql_limit())?;
    let entry_ids: Vec<i64> = stored.iter().map(|r| r.entry_id).collect();
    let others = cash_repo::counterparts(conn, cash, &entry_ids)?;
    let account_ids: Vec<i64> = stored
        .iter()
        .filter_map(|r| counterpart_of(&others, r))
        .collect();
    let parties = load_parties(conn, &account_ids)?;
    let sources = sources_of(conn, &entry_ids)?;

    let mut rows = Vec::with_capacity(stored.len());
    for row in &stored {
        let balance = Balance::from_totals(
            open_dr.checked_add(paisa(row.run_in_paisa)?)?,
            open_cr.checked_add(paisa(row.run_out_paisa)?)?,
        )?;
        let party = match counterpart_of(&others, row) {
            Some(id) => Cell::Party {
                party: party_of_line(&parties, id, None)?,
            },
            None => Cell::Empty,
        };
        // The note of the entry, else the reason or "paid to" of its voucher (as the screen).
        let narration = if row.narration.trim().is_empty() {
            &row.reason
        } else {
            &row.narration
        };
        let (source_type, source_id) = sources
            .get(&row.entry_id)
            .cloned()
            .unwrap_or_else(|| (String::new(), row.entry_id));
        rows.push(Row {
            cells: vec![
                Cell::Date {
                    iso: row.entry_date.clone(),
                },
                Cell::Doc {
                    type_id: ids::doc_id(&row.posting_type),
                    number: row.number,
                },
                party,
                text_cell(Some(narration.clone())),
                amount_or_empty(paisa(row.in_paisa)?),
                amount_or_empty(paisa(row.out_paisa)?),
                balance_cell(balance),
            ],
            source: Some(source_of(&source_type, source_id, row.entry_id)),
            kind: RowKind::Normal,
        });
    }

    Ok(ReportPage {
        id: ReportId::R12,
        title_id: ids::title_id("R12"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("date", ids::COL_DATE, ColumnKind::Date),
            col("doc", ids::COL_DOC, ColumnKind::Doc),
            col("account", ids::COL_ACCOUNT, ColumnKind::Party),
            col("narration", ids::COL_NARRATION, ColumnKind::Text),
            col("cash_in", ids::COL_CASH_IN, ColumnKind::Money),
            col("cash_out", ids::COL_CASH_OUT, ColumnKind::Money),
            col("balance", ids::COL_BALANCE, ColumnKind::Balance),
        ],
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: range.count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money { paisa: total_in },
            Cell::Money { paisa: total_out },
            balance_cell(closing),
        ],
        summary: vec![
            SummaryLine {
                label_id: ids::OPENING_BALANCE.to_owned(),
                cell: balance_cell(opening),
            },
            SummaryLine {
                label_id: ids::CLOSING_BALANCE.to_owned(),
                cell: balance_cell(closing),
            },
        ],
    })
}

/// The source type and id of each entry of the page, so a row can open its record.
fn sources_of(
    conn: &Connection,
    entry_ids: &[i64],
) -> Result<HashMap<i64, (String, i64)>, ReportsError> {
    use mandi_db::rusqlite::params_from_iter;
    let mut found = HashMap::new();
    for chunk in entry_ids.chunks(IN_CHUNK) {
        let sql = format!(
            "SELECT id, source_type, source_id FROM journal_entries WHERE id IN ({})",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                (r.get::<_, String>(1)?, r.get::<_, i64>(2)?),
            ))
        })?;
        for row in rows {
            let (id, source) = row?;
            found.insert(id, source);
        }
    }
    Ok(found)
}
