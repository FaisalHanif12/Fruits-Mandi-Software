//! R11 روزنامچہ, the daily journal (PRD 19.2, FR-19.10, FR-19.11).
//!
//! Every posted journal line in the date range, in the order `(entry_date, entry id, line no)`,
//! with the document it belongs to, the account, the narration and the debit and credit. The
//! totals are `SUM()` of the stored debit and credit columns over the whole filter, from one
//! query; the two are equal for a whole day because every entry balances (PRD FR-6.2).
//!
//! Example (the worked day, 10-10-2026): the first line is the cash part of an earlier posting
//! or the sale part of بکری 1, "اکرم ٹول ٹیکس بنام 12,880", and the totals row ends at
//! `Σ بنام = Σ جمع`.

use mandi_db::rusqlite::Connection;

use super::PageSpec;
use super::lines::{self as lines_sql, LineFilter};
use crate::error::ReportsError;
use crate::reports::common::{amount_or_empty, load_parties, paisa, party_of_line, source_of};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{Cell, Column, ColumnKind, ReportId, ReportPage, Row, RowKind};

/// The columns of R11.
fn columns() -> Vec<Column> {
    let col = |key: &str, title: &str, kind| Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    };
    vec![
        col("date", ids::COL_DATE, ColumnKind::Date),
        col("doc", ids::COL_DOC, ColumnKind::Doc),
        col("account", ids::COL_ACCOUNT, ColumnKind::Party),
        col("narration", ids::COL_NARRATION, ColumnKind::Text),
        col("debit", ids::COL_DEBIT, ColumnKind::Money),
        col("credit", ids::COL_CREDIT, ColumnKind::Money),
    ]
}

/// Builds R11.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a total
/// does not fit.
pub(crate) fn run_r11(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let filter = LineFilter {
        account: typed.party_id,
        person: typed.person_key.clone(),
        from: typed.from.clone(),
        to: typed.to.clone(),
    };
    let sums = lines_sql::totals(conn, &filter)?;
    let (count, total_dr, total_cr) = (sums.count, sums.dr_paisa, sums.cr_paisa);
    let lines = lines_sql::page_by(
        conn,
        &filter,
        typed.line_sort,
        (spec.offset, spec.sql_limit()),
    )?;

    let account_ids: Vec<i64> = lines.iter().map(|l| l.account_id).collect();
    let parties = load_parties(conn, &account_ids)?;
    let mut rows = Vec::with_capacity(lines.len());
    for line in lines {
        let narration = line
            .narration
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(|t| Cell::Text { text: t.to_owned() })
            .unwrap_or(Cell::Empty);
        rows.push(Row {
            cells: vec![
                Cell::Date {
                    iso: line.entry_date.clone(),
                },
                Cell::Doc {
                    type_id: ids::doc_id(&line.posting_type),
                    number: line.doc_number,
                },
                Cell::Party {
                    party: party_of_line(&parties, line.account_id, line.person_key.clone())?,
                },
                narration,
                amount_or_empty(paisa(line.dr)?),
                amount_or_empty(paisa(line.cr)?),
            ],
            source: Some(source_of(&line.source_type, line.source_id, line.entry_id)),
            kind: RowKind::Normal,
        });
    }

    Ok(ReportPage {
        group_above: None,
        id: ReportId::R11,
        title_id: ids::title_id("R11"),
        filters: typed.resolved.clone(),
        columns: columns(),
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: paisa(total_dr)?,
            },
            Cell::Money {
                paisa: paisa(total_cr)?,
            },
        ],
        summary: Vec::new(),
    })
}
