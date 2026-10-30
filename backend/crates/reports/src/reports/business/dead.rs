//! R38 ڈوبی رقم رپورٹ in the report engine (PRD 19.2 R38, FR-10.10, plan 40): the same rows as the
//! typed page `reports_dead_amount` (`crate::dead`), as a report page, so the screen's print, PDF
//! and Excel work like every other report.
//!
//! The status filter `due` lists the people who still owe something (the tile ڈوبی رقم is its
//! total); with no status every move is listed, also the fully recovered ones and the ones made
//! active again (their still owed is 0 and the total is the same).
//!
//! Example (E2E-44, 12-10-26): ارشد, died, moved 11-10-26, owed then 18,684, recovered 4,000,
//! corrections 0, still owed 14,684, last recovery 12-10-26; the total row 14,684.

use mandi_core::dead::{DeadFilter, DeadPartyType, DeadReason, DeadRow, DeadStatus};
use mandi_core::money::Paisa;
use mandi_db::rusqlite::Connection;

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::collections::pukhta_names;
use crate::reports::common::{count_of, load_parties, party_of_line, pukhta_party};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, Column, ColumnKind, PaidFilter, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind,
};

fn col(key: &str, title: &str, kind: ColumnKind) -> Column {
    crate::reports::common::col(key, title, kind)
}

fn label(id: &str) -> Cell {
    Cell::Label { id: id.to_owned() }
}

fn type_label(kind: DeadPartyType) -> &'static str {
    match kind {
        DeadPartyType::Buyer => ids::R38_TYPE_BUYER,
        DeadPartyType::Seller => ids::R38_TYPE_SELLER,
        DeadPartyType::Pukhta => ids::R38_TYPE_PUKHTA,
        DeadPartyType::MoneyLent => ids::R38_TYPE_MONEY_LENT,
    }
}

fn reason_label(reason: DeadReason) -> &'static str {
    match reason {
        DeadReason::Died => ids::R38_REASON_DIED,
        DeadReason::Left => ids::R38_REASON_LEFT,
        DeadReason::Other => ids::R38_REASON_OTHER,
    }
}

fn status_label(status: DeadStatus) -> &'static str {
    match status {
        DeadStatus::Owing => ids::R38_STATUS_OWING,
        DeadStatus::Recovered => ids::R38_STATUS_RECOVERED,
        DeadStatus::Undone => ids::R38_STATUS_UNDONE,
        DeadStatus::Credit => ids::R38_STATUS_CREDIT,
    }
}

fn columns() -> Vec<Column> {
    vec![
        col("party", ids::COL_DEAD_PARTY, ColumnKind::Party),
        col("type", ids::COL_DEAD_TYPE, ColumnKind::Label),
        col("reason", ids::COL_DEAD_REASON, ColumnKind::Label),
        col("note", ids::COL_DEAD_NOTE, ColumnKind::Text),
        col("date", ids::COL_DEAD_DATE, ColumnKind::Date),
        col("owed_then", ids::COL_OWED_THEN, ColumnKind::Money),
        col("recovered", ids::COL_RECOVERED, ColumnKind::Money),
        col("corrections", ids::COL_CORRECTIONS, ColumnKind::Money),
        col("still_owed", ids::COL_STILL_OWED, ColumnKind::Money),
        col("last_recovery", ids::COL_LAST_RECOVERY, ColumnKind::Date),
        col("status", ids::COL_DEAD_STATUS, ColumnKind::Label),
    ]
}

/// Builds R38.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a total does
/// not fit.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let filter = match typed.status {
        Some(PaidFilter::Due) => DeadFilter::Owed,
        _ => DeadFilter::All,
    };
    let (dead_rows, totals) = crate::dead::filtered(conn, filter, "")?;
    let account_ids: Vec<i64> = dead_rows.iter().map(|r| r.account_id).collect();
    let parties = load_parties(conn, &account_ids)?;
    let names = pukhta_names(conn, &account_ids)?;

    let mut all_rows = Vec::with_capacity(dead_rows.len());
    for row in &dead_rows {
        let mut party = party_of_line(&parties, row.account_id, None)?;
        if let Some(key) = &row.pukhta_person_key {
            let known = names.get(key);
            party = pukhta_party(
                party,
                Some(key.clone()),
                known.map(|n| n.name_ur.as_str()),
                known.map(|n| n.name_en.as_str()),
            );
        }
        all_rows.push(row_of(row, party)?);
    }

    Ok(ReportPage {
        group_above: None,
        id: ReportId::R38,
        title_id: ids::title_id("R38"),
        filters: typed.resolved.clone(),
        columns: columns(),
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count_of(all_rows.len()),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: totals.owed_then_paisa,
            },
            Cell::Money {
                paisa: totals.recovered_paisa,
            },
            Cell::Money {
                paisa: Paisa::try_from_paisa(totals.corrections_paisa)?,
            },
            Cell::Money {
                paisa: totals.still_owed_paisa,
            },
            Cell::Empty,
            Cell::Empty,
        ],
        summary: Vec::new(),
    })
}

fn row_of(row: &DeadRow, party: crate::reports::model::PartyRef) -> Result<Row, ReportsError> {
    Ok(Row {
        cells: vec![
            Cell::Party { party },
            label(type_label(row.party_type)),
            label(reason_label(row.reason)),
            if row.note.is_empty() {
                Cell::Empty
            } else {
                Cell::Text {
                    text: row.note.clone(),
                }
            },
            Cell::Date {
                iso: row.entry_date.clone(),
            },
            Cell::Money {
                paisa: row.owed_then_paisa,
            },
            Cell::Money {
                paisa: row.recovered_paisa,
            },
            Cell::Money {
                paisa: Paisa::try_from_paisa(row.corrections_paisa)?,
            },
            Cell::Money {
                paisa: row.still_owed_paisa,
            },
            row.last_recovery_date
                .clone()
                .map_or(Cell::Empty, |iso| Cell::Date { iso }),
            label(status_label(row.status)),
        ],
        source: Some(RowSource {
            kind: SourceKind::Account,
            id: row.account_id,
        }),
        kind: RowKind::Normal,
    })
}
