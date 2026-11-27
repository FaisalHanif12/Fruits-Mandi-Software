//! The print of the advances page (plan 79 item 3, PRD FR-19.10): the tile پیشگی باقی opens one row
//! for each seller (`advances_report`), and its print is that very list with the totals of the
//! whole filter. The rows and totals come from the service; this builds the table of R26's columns.
//!
//! For today the sheet also has R26's columns of what his payable covers and what it does not
//! (v1.61, plan 100 A); the total of the second is the tile.
//!
//! Example (E2E-28 after the payment): رحیم given 15,000, returned 0, adjusted 0, open 15,000, not
//! covered 15,000; totals 15,000 and 15,000.

use mandi_core::auth::Language;
use mandi_core::money::Paisa;
use mandi_core::printing::Paper;
use mandi_core::vouchers::AdvancesReport;
use mandi_db::Db;

use crate::error::ReportsError;
use crate::print::PrintFact;
use crate::reports::common::load_parties;
use crate::reports::ids;
use crate::reports::{
    Cell, Column, ColumnKind, Row, RowKind, TablePrintData, TableSpec, table_print_data,
};

/// The covered and uncovered cells of a row or the totals. A covered part of 0 is `None` in the
/// data and prints as 0.
fn split_cells(covered: Option<Paisa>, uncovered: Option<Paisa>) -> [Cell; 2] {
    match uncovered {
        Some(uncovered) => [
            Cell::Money {
                paisa: covered.unwrap_or(Paisa::ZERO),
            },
            Cell::Money { paisa: uncovered },
        ],
        None => [Cell::Empty, Cell::Empty],
    }
}

/// The table of an advances report: every row, with the totals of the whole filter.
///
/// # Errors
/// A database error; a seller that is missing; a stored amount that cannot be written in words.
pub fn advances_spec(
    db: &Db,
    report: &AdvancesReport,
    search: Option<&str>,
) -> Result<TableSpec, ReportsError> {
    let ids_of: Vec<i64> = report.rows.iter().map(|r| r.account.id).collect();
    let parties = db.read(|conn| load_parties(conn, &ids_of))?;
    let column = |key: &str, title: &str, kind: ColumnKind| Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    };
    let mut columns = vec![
        column("seller", ids::COL_SELLER, ColumnKind::Party),
        column("given", ids::COL_GIVEN, ColumnKind::Money),
        column("returned", ids::COL_RETURNED, ColumnKind::Money),
        column("adjusted", ids::COL_ADJUSTED, ColumnKind::Money),
        column("open", ids::COL_OPEN, ColumnKind::Money),
        column("last_date", ids::COL_LAST_DATE, ColumnKind::Date),
    ];
    // R26's covered and uncovered columns go before the last date, when the report has them.
    let split_at = columns.len().saturating_sub(1);
    if report.uncovered_paisa.is_some() {
        columns.splice(
            split_at..split_at,
            [
                column("covered", ids::COL_ADVANCE_COVERED, ColumnKind::Money),
                column("uncovered", ids::COL_ADVANCE_UNCOVERED, ColumnKind::Money),
            ],
        );
    }
    let mut rows = Vec::with_capacity(report.rows.len());
    for r in &report.rows {
        let Some(party) = parties.get(&r.account.id).cloned() else {
            return Err(ReportsError::Data(format!(
                "seller account {} is missing",
                r.account.id
            )));
        };
        let mut row = Row {
            cells: vec![
                Cell::Party { party },
                Cell::Money {
                    paisa: r.given_paisa,
                },
                Cell::Money {
                    paisa: r.returned_paisa,
                },
                Cell::Money {
                    paisa: r.adjusted_paisa,
                },
                Cell::Money {
                    paisa: r.open_paisa,
                },
                r.last_date
                    .clone()
                    .map_or(Cell::Empty, |iso| Cell::Date { iso }),
            ],
            source: None,
            kind: RowKind::Normal,
        };
        if report.uncovered_paisa.is_some() {
            row.cells.splice(
                split_at..split_at,
                split_cells(r.advance_covered, r.uncovered_paisa),
            );
        }
        rows.push(row);
    }
    let facts = match search.map(str::trim).filter(|s| !s.is_empty()) {
        Some(text) => vec![PrintFact {
            label_id: "print-fact-search".to_owned(),
            value: text.to_owned(),
        }],
        None => Vec::new(),
    };
    let mut totals = vec![
        Cell::Empty,
        Cell::Money {
            paisa: report.given_paisa,
        },
        Cell::Money {
            paisa: report.returned_paisa,
        },
        Cell::Money {
            paisa: report.adjusted_paisa,
        },
        Cell::Money {
            paisa: report.open_paisa,
        },
        Cell::Empty,
    ];
    if report.uncovered_paisa.is_some() {
        totals.splice(
            split_at..split_at,
            split_cells(report.advance_covered, report.uncovered_paisa),
        );
    }
    let spec = TableSpec {
        title_id: ids::title_id("R26"),
        facts,
        column_names: vec![None; columns.len()],
        columns,
        total_count: i64::from(report.total_count),
        rows,
        totals_label_id: "print-buyer-dues-total".to_owned(),
        totals,
        words_of: None,
        plain_sale: false,
        extra_totals: Vec::new(),
    };
    Ok(spec)
}

/// The print of the same table, in the print language.
///
/// # Errors
/// As [`advances_spec`]; a stored amount that cannot be written in words.
pub fn advances_print_data(
    db: &Db,
    report: &AdvancesReport,
    search: Option<&str>,
    language: Language,
    paper: Option<Paper>,
) -> Result<TablePrintData, ReportsError> {
    let spec = advances_spec(db, report, search)?;
    table_print_data(db, language, paper, spec)
}
