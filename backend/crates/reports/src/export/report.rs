//! A report as a sheet (PRD FR-19.3, FR-19.5, FR-19.10, FR-19.13): the same page the screen
//! shows with every row, its filters above the table, its totals row from the backend.
//!
//! Nothing is worked out here: every amount is a stored or engine value, and the totals row is
//! the engine's `totals`, never a sum of the rows (Code Standards B20, B31).

use crate::error::ReportsError;
use crate::reports::print_data::filter_facts;
use crate::reports::{ReportPage, RowKind};

use super::cells::{CellContext, shape_of};
use super::model::{ExportColumn, ExportRow, ExportSheet};

/// What a sheet header needs besides the page.
pub(crate) struct Header {
    /// The firm line.
    pub(crate) firm: String,
    /// The time of the export, in Pakistan time, as text.
    pub(crate) printed_at: String,
    /// True for Urdu.
    pub(crate) right_to_left: bool,
}

/// The sheet of a report page that holds every row.
pub(crate) fn sheet(
    page: &ReportPage,
    ctx: &CellContext<'_>,
    header: &Header,
) -> Result<ExportSheet, ReportsError> {
    let l10n = ctx.l10n;
    let mut facts: Vec<(String, String)> = filter_facts(ctx.fmt, page.id, &page.filters)?
        .into_iter()
        .map(|fact| (l10n.text(&fact.label_id), fact.value))
        .collect();
    facts.push((
        l10n.text("report-print-row-count"),
        ctx.fmt.style.count(page.total_count),
    ));
    facts.push((l10n.text("print-printed-at"), header.printed_at.clone()));

    let mut summary = Vec::with_capacity(page.summary.len());
    for line in &page.summary {
        summary.push((l10n.text(&line.label_id), ctx.cell(&line.cell)?));
    }
    let mut rows = Vec::with_capacity(page.rows.len());
    for row in &page.rows {
        rows.push(ExportRow {
            cells: ctx.cells(&row.cells)?,
            bold: row.kind != RowKind::Normal,
        });
    }
    let title = l10n.text(&page.title_id);
    Ok(ExportSheet {
        name: title.clone(),
        right_to_left: header.right_to_left,
        firm: header.firm.clone(),
        title,
        facts,
        summary,
        columns: page
            .columns
            .iter()
            .map(|c| ExportColumn {
                title: l10n.text(&c.title_id),
                shape: shape_of(c.kind),
            })
            .collect(),
        rows,
        totals_label: l10n.text(&page.totals_label_id),
        totals: ctx.cells(&page.totals)?,
        debit_word: l10n.text("report-side-dr"),
        credit_word: l10n.text("report-side-cr"),
    })
}
