//! The Excel file of پچھلی بکری (PRD 12.7, FR-12.46, FR-19.3, FR-19.10; Code Standards M13; plan 72 P8).
//!
//! The same rows as the print, with today's section and a bold heading for each day (its date, its
//! count and its خام بکری in the خام بکری column), then the totals row of the window. The filters and
//! the switch "show every بکری" are written above the table. Money and counts are numbers made by
//! `rupees_as_f64` and `count_as_f64` only (M13); a sheet holds values, no formulas, and is never read
//! back into the app.
//!
//! The services read the whole list with the screen's own query ([`BikriListWhole`]), so the sheet
//! can never differ from the screen. Nothing is worked out here.

use mandi_core::auth::Language;
use mandi_core::bikri::{BikriListRow, BikriListWhole, BikriListWholeInput};
use mandi_db::Db;
use serde::Deserialize;
use ts_rs::TS;

use super::cells::CellContext;
use super::l10n::L10n;
use super::model::{ExportColumn, ExportProgress, ExportRow, ExportSheet, ExportStage};
use super::{ExcelExported, check_path, fmt_of, header_of, write_workbook};
use crate::error::ReportsError;
use crate::print::load::{load_firm, load_grouping};
use crate::reports::bikri_list::{
    DAY_HEADING, DAY_HEADING_EMPTY, GROSS_COLUMN, TITLE_ID, TODAY_HEADING, columns, facts,
    printed_rows, row_cells, totals_cells,
};
use crate::reports::{ArgValue, Cell, MessageArg};

/// The input of `bikri_list_export_excel`: the window and the filters of the screen, the language
/// of the sheet and where to save it. Every row is written, not a page.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct BikriListExportInput {
    /// The window, the filters and "show everything".
    #[serde(flatten)]
    #[ts(flatten)]
    pub list: BikriListWholeInput,
    /// The language of the sheet: headings, names, digits of the text.
    pub language: Language,
    /// Where to save: an absolute path ending in `.xlsx`, in a folder that exists.
    pub path: String,
}

/// A heading row of a day: a message in the first column and the day's خام بکری under its column.
fn heading(
    ctx: &CellContext<'_>,
    id: &str,
    args: &[(&str, String)],
    gross: Option<mandi_core::money::Paisa>,
) -> Result<ExportRow, ReportsError> {
    let mut cells = vec![Cell::Empty; columns().len()];
    if let Some(first) = cells.first_mut() {
        *first = Cell::Message {
            id: id.to_owned(),
            args: args
                .iter()
                .map(|(name, text)| MessageArg {
                    name: (*name).to_owned(),
                    value: ArgValue::Text { text: text.clone() },
                })
                .collect(),
        };
    }
    if let (Some(gross), Some(slot)) = (gross, cells.get_mut(GROSS_COLUMN)) {
        *slot = Cell::Money { paisa: gross };
    }
    Ok(ExportRow {
        cells: ctx.cells(&cells)?,
        bold: true,
    })
}

/// The rows of a day, in the order the list sent them.
fn day_rows(
    ctx: &CellContext<'_>,
    rows: &[&BikriListRow],
    out: &mut Vec<ExportRow>,
) -> Result<(), ReportsError> {
    for row in rows {
        out.push(ExportRow {
            cells: ctx.cells(&row_cells(row)?)?,
            bold: false,
        });
    }
    Ok(())
}

/// Exports پچھلی بکری to an `.xlsx` file with every row of today's section and of the window.
///
/// # Errors
/// `ReportsError::Rejected` with `ERR_OUT_OF_RANGE` on `path`; a database error;
/// `ReportsError::File` when the file cannot be written.
pub fn export_bikri_list(
    db: &Db,
    whole: &BikriListWhole,
    input: &BikriListExportInput,
    progress: &mut dyn FnMut(ExportProgress),
) -> Result<ExcelExported, ReportsError> {
    let path = check_path(&input.path)?;
    progress(ExportProgress {
        stage: ExportStage::Reading,
        done: 0,
        total: 0,
    });
    let language = input.language;
    let (firm, grouping) =
        db.read(|conn| Ok::<_, ReportsError>((load_firm(conn)?, load_grouping(conn)?)))?;
    let fmt = fmt_of(language, grouping);
    let l10n = L10n::new(language)?;
    let ctx = CellContext {
        fmt: &fmt,
        l10n: &l10n,
    };
    let now = db.clock().now_utc();
    let header = header_of(&fmt, &firm, language, now);

    let mut sheet_facts: Vec<(String, String)> = db
        .read(|conn| facts(conn, &fmt, &input.list, &whole.window))?
        .into_iter()
        .map(|fact| (l10n.text(&fact.label_id), fact.value))
        .collect();
    sheet_facts.push((
        l10n.text("report-print-row-count"),
        fmt.style.count(printed_rows(whole)?),
    ));
    sheet_facts.push((l10n.text("print-printed-at"), header.printed_at.clone()));

    let mut rows: Vec<ExportRow> = Vec::new();
    let mut data_rows: u64 = 0;
    // Today's section.
    if let Some(list) = &whole.today {
        let day = list
            .days
            .first()
            .ok_or_else(|| ReportsError::Data("today's section has no day".to_owned()))?;
        rows.push(heading(
            &ctx,
            TODAY_HEADING,
            &[
                ("date", fmt.style.date(&day.date)?),
                ("count", fmt.style.count(day.count)),
            ],
            Some(day.gross_paisa),
        )?);
        let today_rows: Vec<&BikriListRow> = list.rows.iter().collect();
        day_rows(&ctx, &today_rows, &mut rows)?;
        data_rows = data_rows.saturating_add(u64::try_from(today_rows.len()).unwrap_or(0));
    }
    // The window, a day at a time, newest first, empty days with a heading and no rows.
    for day in &whole.window.days {
        let date = fmt.style.date(&day.date)?;
        if day.count == 0 {
            rows.push(heading(&ctx, DAY_HEADING_EMPTY, &[("date", date)], None)?);
            continue;
        }
        rows.push(heading(
            &ctx,
            DAY_HEADING,
            &[("date", date), ("count", fmt.style.count(day.count))],
            Some(day.gross_paisa),
        )?);
        let of_day: Vec<&BikriListRow> = whole
            .window
            .rows
            .iter()
            .filter(|row| row.entry_date == day.date)
            .collect();
        day_rows(&ctx, &of_day, &mut rows)?;
        data_rows = data_rows.saturating_add(u64::try_from(of_day.len()).unwrap_or(0));
    }

    let title = l10n.text(TITLE_ID);
    let sheet = ExportSheet {
        name: title.clone(),
        right_to_left: header.right_to_left,
        firm: header.firm.clone(),
        title,
        facts: sheet_facts,
        summary: Vec::new(),
        columns: columns()
            .iter()
            .map(|c| ExportColumn {
                title: l10n.text(&c.title_id),
                shape: super::cells::shape_of(c.kind),
            })
            .collect(),
        rows,
        totals_label: l10n.text(crate::reports::ids::TOTAL),
        totals: ctx.cells(&totals_cells(&whole.window))?,
        tables: Vec::new(),
        debit_word: l10n.text("report-side-dr"),
        credit_word: l10n.text("report-side-cr"),
    };
    let size_bytes = write_workbook(&[sheet], &path, progress)?;
    Ok(ExcelExported {
        path: path.display().to_string(),
        size_bytes,
        rows: data_rows,
    })
}
