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

use mandi_db::rusqlite::{Connection, params};

use super::PageSpec;
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

/// Count and totals over the whole filter, from one query (public for the query plan test,
/// Code Standards B18).
#[doc(hidden)]
pub const TOTALS_SQL: &str =
    "SELECT COUNT(*), COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE e.status = 'posted'
           AND (?1 IS NULL OR e.entry_date >= ?1) AND (?2 IS NULL OR e.entry_date <= ?2)
           AND (?3 IS NULL OR l.account_id = ?3) AND (?4 IS NULL OR l.pukhta_person_key = ?4)";

/// One page of lines in the order of the journal (public for the query plan test).
#[doc(hidden)]
pub const PAGE_SQL: &str =
    "SELECT e.id, e.entry_date, e.posting_type, e.source_type, e.source_id, l.account_id,
                l.pukhta_person_key, COALESCE(l.narration, e.narration), l.dr_paisa, l.cr_paisa,
                COALESCE(b.number, v.number)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         LEFT JOIN bikri_entries b ON e.source_type = 'bikri' AND b.id = e.source_id
         LEFT JOIN vouchers v ON e.source_type = 'voucher' AND v.id = e.source_id
         WHERE e.status = 'posted'
           AND (?1 IS NULL OR e.entry_date >= ?1) AND (?2 IS NULL OR e.entry_date <= ?2)
           AND (?3 IS NULL OR l.account_id = ?3) AND (?4 IS NULL OR l.pukhta_person_key = ?4)
         ORDER BY e.entry_date, e.id, l.line_no
         LIMIT ?5 OFFSET ?6";

/// One stored line as read.
struct Line {
    entry_id: i64,
    entry_date: String,
    posting_type: String,
    source_type: String,
    source_id: i64,
    account_id: i64,
    person_key: Option<String>,
    narration: Option<String>,
    dr: i64,
    cr: i64,
    doc_number: Option<i64>,
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
    let from = typed.from.as_deref();
    let to = typed.to.as_deref();
    let account = typed.party_id;
    let person = typed.person_key.as_deref();

    let (count, total_dr, total_cr): (i64, i64, i64) =
        conn.query_row(TOTALS_SQL, params![from, to, account, person], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;

    let mut statement = conn.prepare(PAGE_SQL)?;
    let lines = statement
        .query_map(
            params![from, to, account, person, spec.sql_limit(), spec.offset],
            |r| {
                Ok(Line {
                    entry_id: r.get(0)?,
                    entry_date: r.get(1)?,
                    posting_type: r.get(2)?,
                    source_type: r.get(3)?,
                    source_id: r.get(4)?,
                    account_id: r.get(5)?,
                    person_key: r.get(6)?,
                    narration: r.get(7)?,
                    dr: r.get(8)?,
                    cr: r.get(9)?,
                    doc_number: r.get(10)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

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
