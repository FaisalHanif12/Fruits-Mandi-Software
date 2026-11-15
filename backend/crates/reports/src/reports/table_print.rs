//! A print of a table that is not one of the engine's reports (plan 76: the commission month, the
//! simple buyer dues, the partner statement of a month). The page is built from the same typed
//! cells the screen shows ([`Cell`]) and written in the print language by the shared formatter
//! ([`Fmt`]), so money, dates and digits are the backend's, never the template's (CLAUDE.md
//! rule 1, PRD FR-19.6, FR-19.13). The header and the footer are the shared print frame.
//!
//! A column whose heading is a name (a partner's column) has a label next to its title ID: the
//! name in the print language. The words of a closing amount (PRD FR-19.7) come with it when asked.

use mandi_core::auth::Language;
use mandi_core::money::Paisa;
use mandi_core::printing::Paper;
use mandi_db::Db;
use serde::Serialize;
use ts_rs::TS;

use super::model::{Cell, Column, Names, Row};
use super::print_data::{Fmt, PrintCell, PrintOptions, PrintReportColumn, PrintReportRow};
use crate::error::ReportsError;
use crate::print::{PrintFact, PrintFrame, PrintFrameInput, PrintWords, print_frame};

/// What a table print is made from.
pub struct TableSpec {
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The facts under the title: the month, the filters.
    pub facts: Vec<PrintFact>,
    /// The columns.
    pub columns: Vec<Column>,
    /// The name each column is headed with, when its heading is a name (same length as
    /// `columns`).
    pub column_names: Vec<Option<Names>>,
    /// Every row of the filter.
    pub rows: Vec<Row>,
    /// Fluent message ID of the totals row label.
    pub totals_label_id: String,
    /// The totals row, one cell for each column.
    pub totals: Vec<Cell>,
    /// How many rows the filter has.
    pub total_count: i64,
    /// An amount to write in words under the table (FR-19.7), or none.
    pub words_of: Option<Paisa>,
    /// True to write the quantity and rate of a sale line plain, with no grouping, as the client's
    /// own print does: "(سیب 10 . 1200)" (PRD 19.2 R5).
    pub plain_sale: bool,
    /// More totals rows under the main one (label ID and one cell for each column): the people in
    /// the dead amount, apart from the total (PRD FR-10.10).
    pub extra_totals: Vec<(String, Vec<Cell>)>,
}

/// Everything a template needs to print one table with every row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TablePrintData {
    /// The header and the footer (PRD FR-19.13) with the language and direction.
    pub frame: PrintFrame,
    /// The paper class.
    pub paper: Paper,
    /// The columns.
    pub columns: Vec<PrintReportColumn>,
    /// The name of a column whose heading is a name, in the print language; null for the others.
    pub column_labels: Vec<Option<String>>,
    /// Every row, already text.
    pub rows: Vec<PrintReportRow>,
    /// How many rows there are, as printed.
    pub row_count: String,
    /// Fluent message ID of the totals row label.
    pub totals_label_id: String,
    /// The totals row, one cell for each column.
    pub totals: Vec<PrintCell>,
    /// The amount in words, or null.
    pub words: Option<PrintWords>,
    /// More totals rows, each with its label ID and one cell for each column.
    pub extra_totals: Vec<PrintExtraTotals>,
}

/// A totals row under the main one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PrintExtraTotals {
    /// Fluent message ID of its label.
    pub label_id: String,
    /// One cell for each column.
    pub cells: Vec<PrintCell>,
}

/// Builds the print of a table in the print language.
///
/// # Errors
/// A database error; a stored amount that cannot be written in words.
pub fn table_print_data(
    db: &Db,
    language: Language,
    paper: Option<Paper>,
    spec: TableSpec,
) -> Result<TablePrintData, ReportsError> {
    let grouping = db.read(crate::print::load::load_grouping)?;
    let fmt = Fmt::new(&PrintOptions {
        language,
        grouping,
        paper: paper.unwrap_or(Paper::A4),
    });
    let fmt = if spec.plain_sale {
        fmt.plain_sale()
    } else {
        fmt
    };
    let frame = print_frame(
        db,
        &PrintFrameInput {
            language,
            title_id: spec.title_id,
            facts: spec.facts,
        },
    )?;
    let mut rows = Vec::with_capacity(spec.rows.len());
    for row in &spec.rows {
        rows.push(PrintReportRow {
            cells: fmt.cells(&row.cells)?,
            kind: row.kind,
        });
    }
    Ok(TablePrintData {
        frame,
        paper: paper.unwrap_or(Paper::A4),
        columns: spec
            .columns
            .iter()
            .map(|c| PrintReportColumn {
                key: c.key.clone(),
                title_id: c.title_id.clone(),
                kind: c.kind,
            })
            .collect(),
        column_labels: spec
            .column_names
            .iter()
            .map(|n| n.as_ref().map(|n| fmt.names(n)))
            .collect(),
        rows,
        row_count: fmt.style.count(spec.total_count),
        totals_label_id: spec.totals_label_id,
        totals: fmt.cells(&spec.totals)?,
        words: spec.words_of.map(|a| fmt.words(a)).transpose()?,
        extra_totals: spec
            .extra_totals
            .iter()
            .map(|(label, cells)| {
                Ok(PrintExtraTotals {
                    label_id: label.clone(),
                    cells: fmt.cells(cells)?,
                })
            })
            .collect::<Result<Vec<_>, ReportsError>>()?,
    })
}
