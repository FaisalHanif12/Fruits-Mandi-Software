//! A table that is not one of the engine's reports as a sheet (plan 76: the commission month, the
//! simple buyer dues, the اگراہی sheet and a month of a partner's statement). It is the same
//! [`TableSpec`] the print is made from, so the sheet and the print hold the same cells in the same
//! order, and nothing is worked out here (CLAUDE.md rule 1, Code Standards B31).
//!
//! A column headed by a name (a partner's column) is headed `title name`. A second totals row (the
//! people in the dead amount, PRD FR-10.10) is a bold row under the table with its label in the
//! first empty cell.
//!
//! Example: the commission page of October 2026 gives a sheet with one row for 10-10-26 holding
//! 2484, 2484, 480, 5448, 5488 and the three partner shares, and a totals row.

use mandi_core::auth::Language;
use mandi_db::Db;

use super::cells::{CellContext, shape_of};
use super::l10n::L10n;
use super::model::{ExportColumn, ExportProgress, ExportRow, ExportSheet, ExportStage, XCell};
use super::report::Header;
use super::writer::{check_path, write_workbook};
use super::{ExcelExported, fmt_of, header_of};
use crate::error::ReportsError;
use crate::reports::{RowKind, TableSpec};

/// The sheet of a table: its facts above, every row, the totals row from the backend.
pub(crate) fn sheet(
    spec: &TableSpec,
    ctx: &CellContext<'_>,
    header: &Header,
) -> Result<ExportSheet, ReportsError> {
    let l10n = ctx.l10n;
    let mut facts: Vec<(String, String)> = spec
        .facts
        .iter()
        .map(|fact| (l10n.text(&fact.label_id), fact.value.clone()))
        .collect();
    facts.push((
        l10n.text("report-print-row-count"),
        ctx.fmt.style.count(spec.total_count),
    ));
    facts.push((l10n.text("print-printed-at"), header.printed_at.clone()));

    let mut rows = Vec::with_capacity(spec.rows.len().saturating_add(spec.extra_totals.len()));
    for row in &spec.rows {
        rows.push(ExportRow {
            cells: ctx.cells(&row.cells)?,
            bold: row.kind != RowKind::Normal,
        });
    }
    // The people apart from the total come after the table, each with its label.
    let mut extra: Vec<ExportRow> = Vec::with_capacity(spec.extra_totals.len());
    for (label_id, cells) in &spec.extra_totals {
        let mut cells = ctx.cells(cells)?;
        let label = l10n.text(label_id);
        if let Some(slot) = cells.iter_mut().find(|c| matches!(c, XCell::Empty)) {
            *slot = XCell::Text(label);
        }
        extra.push(ExportRow { cells, bold: true });
    }
    let title = l10n.text(&spec.title_id);
    let columns = spec
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let base = l10n.text(&c.title_id);
            let name = spec
                .column_names
                .get(i)
                .and_then(Option::as_ref)
                .map(|n| ctx.fmt.names(n))
                .filter(|n| !n.trim().is_empty());
            ExportColumn {
                title: match name {
                    Some(name) => format!("{base} {name}"),
                    None => base,
                },
                shape: shape_of(c.kind),
            }
        })
        .collect();
    // The extra totals rows are rows of the table under the main ones; the totals row of the
    // sheet is the main total, as in the print.
    rows.extend(extra);
    Ok(ExportSheet {
        name: title.clone(),
        right_to_left: header.right_to_left,
        firm: header.firm.clone(),
        title,
        facts,
        summary: Vec::new(),
        columns,
        rows,
        totals_label: l10n.text(&spec.totals_label_id),
        totals: ctx.cells(&spec.totals)?,
        tables: Vec::new(),
        debit_word: l10n.text("report-side-dr"),
        credit_word: l10n.text("report-side-cr"),
    })
}

/// Writes a table to an `.xlsx` file.
///
/// # Errors
/// `ReportsError::Rejected` with `ERR_OUT_OF_RANGE` on `path` or for too many rows; a stored
/// amount that cannot be written; `ReportsError::File` when the file cannot be written.
pub fn export_table(
    db: &Db,
    spec: &TableSpec,
    language: Language,
    path: &str,
    progress: &mut dyn FnMut(ExportProgress),
) -> Result<ExcelExported, ReportsError> {
    let path = check_path(path)?;
    progress(ExportProgress {
        stage: ExportStage::Reading,
        done: 0,
        total: 0,
    });
    let (firm, grouping) = db.read(|conn| {
        Ok::<_, ReportsError>((
            crate::print::load::load_firm(conn)?,
            crate::print::load::load_grouping(conn)?,
        ))
    })?;
    let l10n = L10n::new(language)?;
    let fmt = if spec.plain_sale {
        fmt_of(language, grouping).plain_sale()
    } else {
        fmt_of(language, grouping)
    };
    let header = header_of(&fmt, &firm, language, db.clock().now_utc());
    let sheet = sheet(
        spec,
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
