//! Excel export of every report and analytics view (PRD FR-18.4, FR-19.3, FR-19.10, NFR-SEC-13,
//! NFR-SPD-17), and the writer the readable backup pack uses (FR-20.5).
//!
//! - [`export_report`] and [`export_analytics`] read every row of the filter (never a page), build
//!   a sheet with the filters above the table and the engine's totals row, and write an `.xlsx`
//!   file to the path the Owner chose;
//! - headings, labels and month names are the Fluent messages of the sheet language
//!   ([`l10n`]); names are in that language; Urdu sheets read right to left;
//! - dates are real Excel dates, money and counts are numbers, a balance is one signed number
//!   (debit positive) with a number format that prints the side word;
//! - money becomes a double in exactly one place, [`numbers`] (Code Standards M13), and a sheet
//!   holds values only, no formulas. A sheet is never read back into the app.
//!
//! Reads only; nothing is written to the books. A read failure or a bad path leaves no file.
//!
//! Example: R18 for the worked day in English gives a sheet "Trial Balance" whose totals row
//! holds the numbers 85048 and 85048.

pub(crate) mod analytics;
pub(crate) mod cells;
pub mod l10n;
pub mod model;
pub mod numbers;
pub(crate) mod report;
pub mod writer;

use mandi_core::auth::Language;
use mandi_core::money::Grouping;
use mandi_core::printing::Paper;
use mandi_db::Db;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use self::cells::CellContext;
use self::l10n::L10n;
use self::report::Header;
use crate::analytics::{AnalyticsFilters, AnalyticsView};
use crate::error::ReportsError;
use crate::print::load::FirmNames;
use crate::reports::print_data::{Fmt, PrintOptions};
use crate::reports::{ReportFilters, ReportId};

pub use model::{
    ColumnShape, ExportColumn, ExportProgress, ExportRow, ExportSheet, ExportStage, XCell,
};
pub use numbers::{count_as_f64, rupees_as_f64};
pub use writer::{check_path, signed_balance, tab_name, write_workbook};

/// The input of `reports_export_excel`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ReportExportInput {
    /// The report.
    pub id: ReportId,
    /// The filters of the screen. Every row of the filter is written, not a page.
    #[serde(default)]
    pub filters: ReportFilters,
    /// The language of the sheet: headings, names, month names.
    pub language: Language,
    /// Where to save: an absolute path ending in `.xlsx`, in a folder that exists.
    pub path: String,
}

/// The input of `analytics_export_excel`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AnalyticsExportInput {
    /// The view, A to E or H.
    pub view: AnalyticsView,
    /// The filters of the screen.
    pub filters: AnalyticsFilters,
    /// The language of the sheet.
    pub language: Language,
    /// Where to save: an absolute path ending in `.xlsx`, in a folder that exists.
    pub path: String,
}

/// What an export made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ExcelExported {
    /// The file.
    pub path: String,
    /// Its size in bytes.
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// The rows of the table (not counting the heading and the totals).
    #[ts(type = "number")]
    pub rows: u64,
}

/// The firm line, the time and the direction every sheet of one export shares.
pub(crate) fn header_of(
    fmt: &Fmt,
    firm: &FirmNames,
    language: Language,
    now_utc: chrono::DateTime<chrono::Utc>,
) -> Header {
    let name = fmt.name(&firm.name_ur, &firm.name_en);
    let mandi = fmt.name(&firm.mandi_ur, &firm.mandi_en);
    let firm_line = [name, mandi]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    Header {
        firm: firm_line,
        printed_at: fmt.style.printed_at(now_utc),
        right_to_left: language == Language::Ur,
    }
}

/// The formatter of a sheet: names in the sheet language, English digits in text.
pub(crate) fn fmt_of(
    language: Language,
    grouping: Grouping,
    firm: &FirmNames,
    now_utc: chrono::DateTime<chrono::Utc>,
) -> Fmt {
    Fmt::new(&PrintOptions {
        language,
        grouping,
        urdu_digits_setting: false,
        paper: Paper::A4,
        firm: firm.clone(),
        now_utc,
    })
}

/// Exports a report to an `.xlsx` file.
///
/// # Errors
/// `ReportsError::Rejected` with the problems of the filters or `ERR_OUT_OF_RANGE` on `path`;
/// a database or money error; `ReportsError::File` when the file cannot be written.
pub fn export_report(
    db: &Db,
    input: &ReportExportInput,
    progress: &mut dyn FnMut(ExportProgress),
) -> Result<ExcelExported, ReportsError> {
    let path = check_path(&input.path)?;
    progress(ExportProgress {
        stage: ExportStage::Reading,
        done: 0,
        total: 0,
    });
    let (page, firm, grouping) = crate::reports::export_page(db, input.id, &input.filters)?;
    let now = db.clock().now_utc();
    let l10n = L10n::new(input.language)?;
    let fmt = fmt_of(input.language, grouping, &firm, now);
    let header = header_of(&fmt, &firm, input.language, now);
    let sheet = report::sheet(
        &page,
        &CellContext {
            fmt: &fmt,
            l10n: &l10n,
        },
        &header,
    )?;
    let rows = u64::try_from(sheet.rows.len()).unwrap_or(0);
    let size_bytes = write_workbook(&[sheet], &path, progress)?;
    Ok(ExcelExported {
        path: path.display().to_string(),
        size_bytes,
        rows,
    })
}

/// Exports an analytics view to an `.xlsx` file.
///
/// # Errors
/// As [`export_report`].
pub fn export_analytics(
    db: &Db,
    input: &AnalyticsExportInput,
    progress: &mut dyn FnMut(ExportProgress),
) -> Result<ExcelExported, ReportsError> {
    let path = check_path(&input.path)?;
    progress(ExportProgress {
        stage: ExportStage::Reading,
        done: 0,
        total: 0,
    });
    let (page, firm, grouping) = crate::analytics::export_page(db, input.view, &input.filters)?;
    let now = db.clock().now_utc();
    let l10n = L10n::new(input.language)?;
    let fmt = fmt_of(input.language, grouping, &firm, now);
    let header = header_of(&fmt, &firm, input.language, now);
    let sheet = analytics::sheet(
        &page,
        &CellContext {
            fmt: &fmt,
            l10n: &l10n,
        },
        &header,
    )?;
    let rows = u64::try_from(sheet.rows.len()).unwrap_or(0);
    let size_bytes = write_workbook(&[sheet], &path, progress)?;
    Ok(ExcelExported {
        path: path.display().to_string(),
        size_bytes,
        rows,
    })
}
