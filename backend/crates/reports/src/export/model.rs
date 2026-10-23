//! What a sheet holds before it is written (PRD FR-18.4, FR-19.3). The builders (`report`,
//! `analytics`, `pack`) turn a report page into an [`ExportSheet`]; the writer turns an
//! [`ExportSheet`] into a file. Money stays `Paisa` until the writer asks `numbers` for the
//! whole-rupee value of a cell (Code Standards M13).

use mandi_core::money::Paisa;
use serde::Serialize;
use ts_rs::TS;

/// What a column holds. The writer sets its width and alignment from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnShape {
    /// Words and names.
    Text,
    /// Whole numbers.
    Whole,
    /// Money in whole rupees.
    Money,
    /// Money with a side, written as one signed number.
    Balance,
    /// A date.
    Date,
}

/// One column of the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportColumn {
    /// The heading, in the sheet language.
    pub title: String,
    /// What it holds.
    pub shape: ColumnShape,
}

/// One cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XCell {
    /// Nothing.
    Empty,
    /// Text.
    Text(String),
    /// A whole number (a count, a quantity, a day count).
    Whole(i64),
    /// Money in whole rupees.
    Money(Paisa),
    /// Money with a side as one signed amount: a debit is positive, a credit negative (PRD C13).
    Signed(Paisa),
    /// A real date.
    Date {
        /// The year.
        year: i32,
        /// The month, 1 to 12.
        month: u32,
        /// The day of the month.
        day: u32,
    },
}

/// One row of the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportRow {
    /// One cell for each column.
    pub cells: Vec<XCell>,
    /// True for a heading row of a group (drawn in bold).
    pub bold: bool,
}

/// One sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSheet {
    /// The name of the tab. The writer cuts it to what Excel allows.
    pub name: String,
    /// True for Urdu: the sheet reads right to left.
    pub right_to_left: bool,
    /// The firm line at the top.
    pub firm: String,
    /// The title of the report.
    pub title: String,
    /// The filters of the run and the time of the export, as label and value (FR-19.10).
    pub facts: Vec<(String, String)>,
    /// Lines above the table (opening balance and so on).
    pub summary: Vec<(String, XCell)>,
    /// The table columns.
    pub columns: Vec<ExportColumn>,
    /// Every row of the filter.
    pub rows: Vec<ExportRow>,
    /// The label of the totals row.
    pub totals_label: String,
    /// The totals row, one cell for each column. Empty when the report has none.
    pub totals: Vec<XCell>,
    /// The word for a debit side, for the number format of a balance.
    pub debit_word: String,
    /// The word for a credit side.
    pub credit_word: String,
}

/// The step an export is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ExportStage {
    /// The rows are being read from the books.
    Reading,
    /// The rows are being written to the file.
    Writing,
    /// The file is whole.
    Done,
}

/// How far an export has come, for the progress the screen shows (NFR-SPD-17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct ExportProgress {
    /// The step.
    pub stage: ExportStage,
    /// Rows written so far (all sheets).
    #[ts(type = "number")]
    pub done: u64,
    /// Rows to write (all sheets). 0 while the rows are still being read.
    #[ts(type = "number")]
    pub total: u64,
}
