//! An analytics view as a sheet (PRD FR-18.4): the same table the screen shows with every row,
//! the filters above it, the totals row from the backend. Matrix columns (a month, a year, a
//! day or an item) take the heading they stand for.

use crate::analytics::print::filter_facts;
use crate::analytics::{AnalyticsHeading, AnalyticsPage};
use crate::error::ReportsError;
use crate::reports::RowKind;

use super::cells::{CellContext, shape_of};
use super::model::{ExportColumn, ExportRow, ExportSheet};
use super::report::Header;

fn heading_text(ctx: &CellContext<'_>, heading: &AnalyticsHeading) -> Result<String, ReportsError> {
    Ok(match heading {
        AnalyticsHeading::Day { iso } => ctx.fmt.style.date(iso)?,
        AnalyticsHeading::Month { year, month } => ctx.month_text(*year, *month),
        AnalyticsHeading::Year { year } => ctx.fmt.style.plain(*year),
        AnalyticsHeading::Item { names } => ctx.fmt.names(names),
        AnalyticsHeading::OtherItems => ctx.l10n.text("analytics-col-other-items"),
        AnalyticsHeading::Partner { names } => ctx.fmt.names(names),
    })
}

/// The sheet of an analytics page that holds every row.
pub(crate) fn sheet(
    page: &AnalyticsPage,
    ctx: &CellContext<'_>,
    header: &Header,
) -> Result<ExportSheet, ReportsError> {
    let l10n = ctx.l10n;
    let mut facts: Vec<(String, String)> = filter_facts(ctx.fmt, &page.filters)?
        .into_iter()
        .map(|fact| (l10n.text(&fact.label_id), fact.value))
        .collect();
    facts.push((
        l10n.text("report-print-row-count"),
        ctx.fmt.style.count(page.total_count),
    ));
    facts.push((l10n.text("print-printed-at"), header.printed_at.clone()));

    let mut columns = Vec::with_capacity(page.columns.len());
    for column in &page.columns {
        let title = match &column.heading {
            Some(heading) => heading_text(ctx, heading)?,
            None => l10n.text(&column.title_id),
        };
        columns.push(ExportColumn {
            title,
            shape: shape_of(column.kind),
        });
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
        summary: Vec::new(),
        columns,
        rows,
        totals_label: l10n.text(&page.totals_label_id),
        totals: ctx.cells(&page.totals)?,
        debit_word: l10n.text("report-side-dr"),
        credit_word: l10n.text("report-side-cr"),
    })
}
