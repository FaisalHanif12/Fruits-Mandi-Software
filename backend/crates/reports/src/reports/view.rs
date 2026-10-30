//! Sorting a report and month headings in it (plan 38 R4, PRD FR-19.11, Code Standards B31).
//!
//! A report is built whole (every row of the filter, no page) when the screen asks for a sort or
//! for month headings. Then:
//!
//! - [`sort_page`] sorts the rows of the whole filter by one column (the `key` of a column of the
//!   answer). Heading rows (`group`) stay where they are: the rows between two headings are
//!   sorted among themselves, so a group never loses its rows to another group. The sort is
//!   stable, so equal rows keep the order of the report.
//! - [`add_month_headings`] puts a heading row (kind `month`) above the first row of each month,
//!   for a report whose rows are in date order. Its cells are the month, the sums of the amount
//!   columns over the WHOLE month, and, for a running balance, the balance at the end of the month.
//!   The screen draws them and never adds rows up.
//! - [`cut`] takes the page asked for out of the whole list and, for a page that starts inside a
//!   month, gives the heading of that month in `group_above`.
//!
//! Example: R13 with `group_by_month` for a buyer who has rows in October and November gives a
//! heading "2026-10" with the October debits and credits and the balance on 31-10, then the
//! October rows, then a heading "2026-11" and so on.

use std::cmp::Ordering;

use mandi_core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_core::money::Paisa;
use mandi_core::paging::{SortDirection, SortInput};

use super::PageSpec;
use super::model::{Cell, Column, ColumnKind, ReportId, ReportPage, Row, RowKind};
use crate::error::ReportsError;

/// The `last_day` of a filter with no end: after every real day.
pub(crate) const OPEN_END_DAY: &str = "9999-12-31";

/// The reports whose rows are in date order, so month headings make sense.
pub(crate) fn has_months(id: ReportId) -> bool {
    matches!(
        id,
        ReportId::R2
            | ReportId::R3
            | ReportId::R4
            | ReportId::R11
            | ReportId::R12
            | ReportId::R13
            | ReportId::R14
            | ReportId::R15
            | ReportId::R23
            | ReportId::R25
    )
}

/// The index of the first column of kind date: the column whose month the headings use.
fn date_column(columns: &[Column]) -> Option<usize> {
    columns.iter().position(|c| c.kind == ColumnKind::Date)
}

/// The reports whose last column is a running balance (the cash book, the party ledger and the
/// party statement).
fn has_running_balance(id: ReportId) -> bool {
    matches!(id, ReportId::R12 | ReportId::R13 | ReportId::R14)
}

/// Whether the sort keeps the report in date order: no sort, or a sort by its date column.
pub(crate) fn in_date_order(columns: &[Column], sort: Option<&SortInput>) -> bool {
    match (sort, date_column(columns)) {
        (None, _) => true,
        (Some(sort), Some(index)) => columns
            .get(index)
            .is_some_and(|column| column.key == sort.column),
        (Some(_), None) => false,
    }
}

/// The column a sort names, or `ERR_OUT_OF_RANGE` on `sort.column` with the sortable keys.
fn sort_index(columns: &[Column], sort: &SortInput) -> Result<usize, ReportsError> {
    let sortable = |kind: ColumnKind| !matches!(kind, ColumnKind::Detail | ColumnKind::Message);
    match columns
        .iter()
        .position(|c| c.key == sort.column && sortable(c.kind))
    {
        Some(index) => Ok(index),
        None => {
            let allowed = columns
                .iter()
                .filter(|c| sortable(c.kind))
                .map(|c| c.key.as_str())
                .collect::<Vec<_>>()
                .join(",");
            Err(ReportsError::Rejected(vec![
                FieldError::new(ErrorCode::ErrOutOfRange, "sort.column")
                    .with("allowed", ErrorValue::Text(allowed)),
            ]))
        }
    }
}

/// A rank for the kind of a cell, so cells of different kinds in one column still have an order.
/// An empty cell is the smallest.
fn rank(cell: &Cell) -> u8 {
    match cell {
        Cell::Empty => 0,
        _ => 1,
    }
}

/// The balance as a signed number: a debit is plus, a credit is minus, zero is zero.
fn signed(paisa: Paisa, side: mandi_core::accounts::BalanceSide) -> i64 {
    use mandi_core::accounts::BalanceSide;
    match side {
        BalanceSide::Dr => paisa.as_paisa(),
        BalanceSide::Cr => paisa.as_paisa().saturating_neg(),
        BalanceSide::Zero => 0,
    }
}

/// Urdu alphabetical order of two texts: the ICU4X collator, made once.
pub(crate) fn urdu_compare(a: &str, b: &str) -> Ordering {
    use std::sync::OnceLock;
    static COLLATOR: OnceLock<Option<mandi_core::text::UrduCollator>> = OnceLock::new();
    match COLLATOR
        .get_or_init(|| mandi_core::text::UrduCollator::new().ok())
        .as_ref()
    {
        Some(collator) => collator.compare(a, b),
        None => a.cmp(b),
    }
}

/// Orders two cells of one column. Cells of the same kind compare by their value; cells of
/// different kinds (which a column does not hold, except empty) compare as equal.
pub(crate) fn compare(a: &Cell, b: &Cell, urdu: bool) -> Ordering {
    match rank(a).cmp(&rank(b)) {
        Ordering::Equal => {}
        other => return other,
    }
    match (a, b) {
        (Cell::Text { text: x }, Cell::Text { text: y }) => x.to_lowercase().cmp(&y.to_lowercase()),
        (Cell::Label { id: x }, Cell::Label { id: y }) => x.cmp(y),
        (Cell::Number { value: x }, Cell::Number { value: y }) => x.cmp(y),
        (Cell::Money { paisa: x }, Cell::Money { paisa: y }) => x.cmp(y),
        (
            Cell::Balance {
                paisa: x,
                side: side_x,
            },
            Cell::Balance {
                paisa: y,
                side: side_y,
            },
        ) => signed(*x, *side_x).cmp(&signed(*y, *side_y)),
        (Cell::Date { iso: x }, Cell::Date { iso: y }) => x.cmp(y),
        (
            Cell::Month {
                year: year_x,
                month: month_x,
            },
            Cell::Month {
                year: year_y,
                month: month_y,
            },
        ) => (year_x, month_x).cmp(&(year_y, month_y)),
        (Cell::Percent { hundredths: x }, Cell::Percent { hundredths: y }) => x.cmp(y),
        (Cell::Party { party: x }, Cell::Party { party: y }) => x.code.cmp(&y.code),
        // A name sorts in the script of the app: Urdu alphabetical order (NFR-URD-19) or English.
        (Cell::Names { names: x }, Cell::Names { names: y }) => {
            if urdu {
                urdu_compare(&x.ur, &y.ur)
            } else {
                x.en.to_lowercase().cmp(&y.en.to_lowercase())
            }
        }
        (
            Cell::Doc {
                type_id: type_x,
                number: number_x,
            },
            Cell::Doc {
                type_id: type_y,
                number: number_y,
            },
        ) => (type_x, number_x).cmp(&(type_y, number_y)),
        _ => Ordering::Equal,
    }
}

/// Sorts the rows of a whole report by the column the sort names. See the module doc.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `sort.column` if no sortable column has that key.
pub(crate) fn sort_page(page: &mut ReportPage, sort: &SortInput) -> Result<(), ReportsError> {
    let index = sort_index(&page.columns, sort)?;
    // A running balance means something only in the order of the days: sorted by another column,
    // the ledger leaves it empty and the screen says "balance shows in date order only" (plan 38
    // B2). The other balances (a buyer's dues in R20) are data of the row and stay.
    let date_sort = page
        .columns
        .get(index)
        .is_some_and(|c| c.kind == ColumnKind::Date);
    if !date_sort && has_running_balance(page.id) {
        for column in page
            .columns
            .iter()
            .enumerate()
            .filter(|(_, c)| c.kind == ColumnKind::Balance)
            .map(|(i, _)| i)
            .collect::<Vec<_>>()
        {
            for row in &mut page.rows {
                if let Some(cell) = row.cells.get_mut(column) {
                    *cell = Cell::Empty;
                }
            }
        }
    }
    let urdu = sort.language == Some(mandi_core::auth::Language::Ur);
    // The ledgers, the day book and the cash report are in the order of the days, so newest first
    // is the exact reverse of it, as their pages on the screen are (one SQL page, plan 38 B2): the
    // rows of one day run newest line first, in the print and the Excel file too.
    let reverse = date_sort
        && sort.direction == mandi_core::paging::SortDirection::Desc
        && matches!(
            page.id,
            ReportId::R11 | ReportId::R12 | ReportId::R13 | ReportId::R14
        );
    let mut sorted: Vec<Row> = Vec::with_capacity(page.rows.len());
    let mut segment: Vec<Row> = Vec::new();
    let flush = |segment: &mut Vec<Row>, sorted: &mut Vec<Row>| {
        if reverse {
            segment.reverse();
            sorted.append(segment);
            return;
        }
        // A stable sort over the natural order of the report.
        segment.sort_by(|a, b| {
            let empty = Cell::Empty;
            let (x, y) = (
                a.cells.get(index).unwrap_or(&empty),
                b.cells.get(index).unwrap_or(&empty),
            );
            sort.direction.apply(compare(x, y, urdu))
        });
        sorted.append(segment);
    };
    for row in std::mem::take(&mut page.rows) {
        if row.kind == RowKind::Normal {
            segment.push(row);
        } else {
            flush(&mut segment, &mut sorted);
            sorted.push(row);
        }
    }
    flush(&mut segment, &mut sorted);
    page.rows = sorted;
    Ok(())
}

/// The month of a date cell, `YYYY-MM`, with its year and month number.
fn month_of(cell: &Cell) -> Option<(String, i64, i64)> {
    let Cell::Date { iso } = cell else {
        return None;
    };
    let year = iso.get(..4)?.parse::<i64>().ok()?;
    let month = iso.get(5..7)?.parse::<i64>().ok()?;
    Some((iso.get(..7)?.to_owned(), year, month))
}

/// Puts a heading row above the first row of each month. A row with no date (an opening balance
/// row) is left where it is. See the module doc for the cells.
///
/// # Errors
/// `ReportsError::Money` if a sum does not fit.
pub(crate) fn add_month_headings(
    page: &mut ReportPage,
    newest_first: bool,
) -> Result<(), ReportsError> {
    let Some(date_index) = date_column(&page.columns) else {
        return Ok(());
    };
    // Which columns add up: those the totals row adds up (money or number), and for a running
    // balance column the balance of the last row of the month.
    let kinds: Vec<ColumnKind> = page.columns.iter().map(|c| c.kind).collect();
    let adds: Vec<bool> = page
        .totals
        .iter()
        .map(|cell| matches!(cell, Cell::Money { .. } | Cell::Number { .. }))
        .collect();
    let width = page.columns.len();
    let rows = std::mem::take(&mut page.rows);
    // First pass: the sums of each month, in the order the months appear.
    struct Month {
        key: String,
        year: i64,
        month: i64,
        sums: Vec<Option<i64>>,
        /// The balance at the end of the month: the balance of the last row of the month in the
        /// order of the days.
        last_balance: Vec<Option<Cell>>,
        /// Newest first only: a row of this month was already met.
        seen: bool,
    }
    let mut months: Vec<Month> = Vec::new();
    let mut month_of_row: Vec<Option<usize>> = Vec::with_capacity(rows.len());
    for row in &rows {
        let found = if row.kind == RowKind::Normal {
            row.cells.get(date_index).and_then(month_of)
        } else {
            None
        };
        // A cancelled line (a voucher in R15, R23, R25) stays in the rows but moves no money: it
        // adds nothing to its month, as the totals row leaves it out.
        let cancelled = row
            .cells
            .iter()
            .any(|cell| matches!(cell, Cell::Label { id } if id == super::ids::STATUS_CANCELLED));
        let Some((key, year, month)) = found else {
            month_of_row.push(None);
            continue;
        };
        let at = match months.last() {
            Some(last) if last.key == key => months.len().saturating_sub(1),
            _ => {
                months.push(Month {
                    key,
                    year,
                    month,
                    sums: vec![None; width],
                    last_balance: vec![None; width],
                    seen: false,
                });
                months.len().saturating_sub(1)
            }
        };
        month_of_row.push(Some(at));
        let Some(current) = months.get_mut(at) else {
            continue;
        };
        // Oldest first, every row moves the end of the month on. Newest first, the rows of a
        // report with a running balance are the exact reverse of the order of the days (see
        // `sort_page`), so the first row met in the month is its last line: the end of the month.
        let takes_balance = !newest_first || !current.seen;
        current.seen = true;
        for (column, cell) in row.cells.iter().enumerate() {
            if cancelled {
                break;
            }
            match cell {
                Cell::Money { paisa } if adds.get(column).copied().unwrap_or(false) => {
                    let slot = current.sums.get_mut(column);
                    if let Some(slot) = slot {
                        let sum = slot.unwrap_or(0);
                        *slot = Some(
                            sum.checked_add(paisa.as_paisa())
                                .ok_or(mandi_core::money::MoneyError::Overflow)?,
                        );
                    }
                }
                Cell::Number { value } if adds.get(column).copied().unwrap_or(false) => {
                    if let Some(slot) = current.sums.get_mut(column) {
                        let sum = slot.unwrap_or(0);
                        *slot = Some(
                            sum.checked_add(*value)
                                .ok_or(mandi_core::money::MoneyError::Overflow)?,
                        );
                    }
                }
                Cell::Balance { .. }
                    if takes_balance && kinds.get(column) == Some(&ColumnKind::Balance) =>
                {
                    if let Some(slot) = current.last_balance.get_mut(column) {
                        *slot = Some(cell.clone());
                    }
                }
                _ => {}
            }
        }
    }
    // Second pass: the heading goes above the first row of each month.
    let mut out: Vec<Row> = Vec::with_capacity(rows.len().saturating_add(months.len()));
    let mut last_put: Option<usize> = None;
    for (row, month_index) in rows.into_iter().zip(month_of_row) {
        if let Some(index) = month_index
            && last_put != Some(index)
            && let Some(month) = months.get(index)
        {
            let mut cells = vec![Cell::Empty; width];
            for (column, kind) in kinds.iter().enumerate() {
                let value = match kind {
                    _ if column == date_index => Some(Cell::Month {
                        year: month.year,
                        month: month.month,
                    }),
                    // A column the totals row adds up shows 0 for a month with no amount in it,
                    // as the heading of the screen does (made from the sub-run totals).
                    ColumnKind::Money => match month.sums.get(column).copied().flatten() {
                        Some(v) => Some(Cell::Money {
                            paisa: Paisa::try_from_paisa(v)?,
                        }),
                        None if adds.get(column).copied().unwrap_or(false) => {
                            Some(Cell::Money { paisa: Paisa::ZERO })
                        }
                        None => None,
                    },
                    ColumnKind::Number => month
                        .sums
                        .get(column)
                        .copied()
                        .flatten()
                        .or(adds.get(column).copied().unwrap_or(false).then_some(0))
                        .map(|value| Cell::Number { value }),
                    ColumnKind::Balance => month.last_balance.get(column).cloned().flatten(),
                    _ => None,
                };
                if let (Some(value), Some(slot)) = (value, cells.get_mut(column)) {
                    *slot = value;
                }
            }
            out.push(Row {
                cells,
                source: None,
                kind: RowKind::Month,
            });
            last_put = Some(index);
        }
        out.push(row);
    }
    page.rows = out;
    Ok(())
}

/// Sorts a whole report and gives it month headings, as the screen asked: the one place the run,
/// the print and the Excel export use, so what is printed is what is on the screen (plan 38 R4).
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `sort.column`; `ReportsError::Money` if a month sum does not fit.
pub(crate) fn shape(
    page: &mut ReportPage,
    id: ReportId,
    sort: Option<&SortInput>,
    group_by_month: bool,
) -> Result<(), ReportsError> {
    if let Some(sort) = sort {
        sort_page(page, sort)?;
    }
    if group_by_month && has_months(id) && in_date_order(&page.columns, sort) {
        let newest_first = sort.is_some_and(|s| s.direction == SortDirection::Desc);
        add_month_headings(page, newest_first)?;
    }
    Ok(())
}

/// Cuts the page asked for out of a report that holds every row. Month headings are not counted:
/// `offset`, `limit` and `total_count` count the other rows, so they mean the same with and without
/// months. A month heading leads its first row on the page; when the page starts inside a month
/// (`offset` above 0) that month's heading is given in `group_above` instead, and so is the
/// heading of the group a page of a grouped report starts in.
pub(crate) fn cut(page: &mut ReportPage, spec: PageSpec) {
    let all = std::mem::take(&mut page.rows);
    let counted = all.iter().filter(|r| r.kind != RowKind::Month).count();
    page.total_count = i64::try_from(counted).unwrap_or(i64::MAX);
    let skip = usize::try_from(spec.offset).unwrap_or(usize::MAX);
    let take = spec
        .limit
        .map_or(usize::MAX, |l| usize::try_from(l).unwrap_or(usize::MAX));
    let mut rows: Vec<Row> = Vec::new();
    let mut group_above: Option<Row> = None;
    let mut pending: Option<Row> = None;
    // The last heading of a group or a month before the page, for `group_above`.
    let mut last_heading: Option<Row> = None;
    let mut seen = 0_usize;
    for row in all {
        if row.kind == RowKind::Month {
            pending = Some(row);
            continue;
        }
        let in_page = seen >= skip && seen - skip < take;
        if in_page {
            if let Some(heading) = pending.take() {
                if rows.is_empty() && skip > 0 {
                    group_above = Some(heading);
                } else {
                    rows.push(heading);
                }
            } else if rows.is_empty() && skip > 0 && row.kind == RowKind::Normal {
                group_above = last_heading.clone();
            }
            rows.push(row);
        } else {
            if row.kind != RowKind::Normal {
                last_heading = Some(row.clone());
            }
            if let Some(heading) = pending.take() {
                last_heading = Some(heading);
            }
        }
        seen += 1;
    }
    page.rows = rows;
    page.group_above = group_above;
    page.offset = spec.offset;
    page.limit = spec.reported_limit();
}

/// The days of a month `(year, month)` clipped to the filter: `YYYY-MM-DD` of its first and last
/// day, the first not before `from` and the last not after `to`. `None` if nothing is left.
fn clipped_month(
    (year, month): (i64, i64),
    from: Option<&str>,
    to: &str,
) -> Option<(String, String)> {
    use mandi_core::dates::BusinessDate;
    let (year, month) = (i32::try_from(year).ok()?, u32::try_from(month).ok()?);
    let first = BusinessDate::from_ymd(year, month, 1)?;
    let next = if month == 12 {
        BusinessDate::from_ymd(year.checked_add(1)?, 1, 1)?
    } else {
        BusinessDate::from_ymd(year, month.checked_add(1)?, 1)?
    };
    let lo = first.to_iso();
    let hi = next.previous_day()?.to_iso();
    let lo = from.map_or(
        lo.clone(),
        |f| if f > lo.as_str() { f.to_owned() } else { lo },
    );
    let hi = if to < hi.as_str() { to.to_owned() } else { hi };
    (lo <= hi).then_some((lo, hi))
}

/// The heading row of one month of a report: the month in the date column, the sums of the whole
/// month in the amount columns, and the balance at the end of the month. The sums are the report's
/// own totals for the days of the month, so they are the same numbers the report would show for
/// that month, from the same queries (plan 38 B2).
pub(crate) fn month_heading(
    columns: &[Column],
    sub: &ReportPage,
    (year, month): (i64, i64),
) -> Row {
    let date_index = date_column(columns);
    let closing = sub
        .summary
        .iter()
        .find(|line| line.label_id == super::ids::CLOSING_BALANCE)
        .map(|line| line.cell.clone());
    let cells = columns
        .iter()
        .enumerate()
        .map(|(i, column)| {
            if Some(i) == date_index {
                return Cell::Month { year, month };
            }
            match (column.kind, sub.totals.get(i)) {
                (ColumnKind::Money, Some(total @ Cell::Money { .. }))
                | (ColumnKind::Number, Some(total @ Cell::Number { .. })) => total.clone(),
                (ColumnKind::Balance, _) => closing.clone().unwrap_or(Cell::Empty),
                _ => Cell::Empty,
            }
        })
        .collect();
    Row {
        cells,
        source: None,
        kind: RowKind::Month,
    }
}

/// The year and month of the date cell of a row.
fn row_month(row: &Row, date_index: usize) -> Option<(i64, i64)> {
    month_of(row.cells.get(date_index)?).map(|(_, year, month)| (year, month))
}

/// What a page needs to put month headings on itself.
pub(crate) struct MonthPlan<'a> {
    /// The filters of the run, to clip for each month.
    pub(crate) filters: &'a super::model::ReportFilters,
    /// The first day of the filter, if one is set.
    pub(crate) first_day: Option<String>,
    /// The last day of the filter; [`OPEN_END_DAY`] when the filter has no end, so a month is
    /// never cut at today (entries can be dated later if the clock was moved back, NFR-REL-11).
    pub(crate) last_day: String,
}

/// Puts a heading above the first row of each month on THIS page, and, when the page starts inside
/// a month, gives that month's heading in `group_above`. Each heading is one totals read for the
/// days of its month (`sub_run` runs the report for them with no rows), never a pass over the
/// whole filter.
///
/// # Errors
/// What `sub_run` gives.
pub(crate) fn add_page_months(
    page: &mut ReportPage,
    plan: &MonthPlan<'_>,
    mut sub_run: impl FnMut(&super::model::ReportFilters) -> Result<ReportPage, ReportsError>,
) -> Result<(), ReportsError> {
    let Some(date_index) = date_column(&page.columns) else {
        return Ok(());
    };
    let mut headings: std::collections::BTreeMap<(i64, i64), Row> =
        std::collections::BTreeMap::new();
    let mut heading_for = |month: (i64, i64),
                           page_columns: &[Column]|
     -> Result<Option<Row>, ReportsError> {
        if let Some(row) = headings.get(&month) {
            return Ok(Some(row.clone()));
        }
        let Some((lo, hi)) = clipped_month(month, plan.first_day.as_deref(), &plan.last_day) else {
            return Ok(None);
        };
        let mut filters = plan.filters.clone();
        filters.shortcut = None;
        filters.as_of = String::new();
        filters.from = day_text(&lo);
        filters.to = day_text(&hi);
        let sub = sub_run(&filters)?;
        let row = month_heading(page_columns, &sub, month);
        headings.insert(month, row.clone());
        Ok(Some(row))
    };
    let rows = std::mem::take(&mut page.rows);
    let mut out = Vec::with_capacity(rows.len().saturating_add(2));
    let mut last_month: Option<(i64, i64)> = None;
    for (position, row) in rows.into_iter().enumerate() {
        let month = if row.kind == RowKind::Normal {
            row_month(&row, date_index)
        } else {
            None
        };
        if let Some(month) = month
            && last_month != Some(month)
        {
            if let Some(heading) = heading_for(month, &page.columns)? {
                if position == 0 && page.offset > 0 {
                    // The page starts inside this month, or on its first row: the heading is given
                    // above the page, with the sums of the whole month.
                    page.group_above = Some(heading);
                } else {
                    out.push(heading);
                }
            }
            last_month = Some(month);
        }
        out.push(row);
    }
    page.rows = out;
    Ok(())
}

/// `DD-MM-YYYY` of a `YYYY-MM-DD` day, as the filters take it.
fn day_text(iso: &str) -> String {
    mandi_core::dates::BusinessDate::parse_iso(iso)
        .map(mandi_core::dates::format_ddmmyyyy)
        .unwrap_or_default()
}
