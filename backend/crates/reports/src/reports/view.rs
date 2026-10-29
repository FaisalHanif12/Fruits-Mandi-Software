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
use mandi_core::paging::SortInput;

use super::PageSpec;
use super::model::{Cell, Column, ColumnKind, ReportId, ReportPage, Row, RowKind};
use crate::error::ReportsError;

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

/// Orders two cells of one column. Cells of the same kind compare by their value; cells of
/// different kinds (which a column does not hold, except empty) compare as equal.
pub(crate) fn compare(a: &Cell, b: &Cell) -> Ordering {
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
        (Cell::Names { names: x }, Cell::Names { names: y }) => {
            x.en.to_lowercase().cmp(&y.en.to_lowercase())
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
    let mut sorted: Vec<Row> = Vec::with_capacity(page.rows.len());
    let mut segment: Vec<Row> = Vec::new();
    let flush = |segment: &mut Vec<Row>, sorted: &mut Vec<Row>| {
        // A stable sort over the natural order of the report.
        segment.sort_by(|a, b| {
            let empty = Cell::Empty;
            let (x, y) = (
                a.cells.get(index).unwrap_or(&empty),
                b.cells.get(index).unwrap_or(&empty),
            );
            sort.direction.apply(compare(x, y))
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
pub(crate) fn add_month_headings(page: &mut ReportPage) -> Result<(), ReportsError> {
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
        last_balance: Vec<Option<Cell>>,
    }
    let mut months: Vec<Month> = Vec::new();
    let mut month_of_row: Vec<Option<usize>> = Vec::with_capacity(rows.len());
    for row in &rows {
        let found = if row.kind == RowKind::Normal {
            row.cells.get(date_index).and_then(month_of)
        } else {
            None
        };
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
                });
                months.len().saturating_sub(1)
            }
        };
        month_of_row.push(Some(at));
        let Some(current) = months.get_mut(at) else {
            continue;
        };
        for (column, cell) in row.cells.iter().enumerate() {
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
                Cell::Balance { .. } if kinds.get(column) == Some(&ColumnKind::Balance) => {
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
                    ColumnKind::Money => month
                        .sums
                        .get(column)
                        .copied()
                        .flatten()
                        .and_then(|v| Paisa::try_from_paisa(v).ok())
                        .map(|paisa| Cell::Money { paisa }),
                    ColumnKind::Number => month
                        .sums
                        .get(column)
                        .copied()
                        .flatten()
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

/// Cuts the page asked for out of a report that holds every row. `total_count` becomes the number
/// of rows of the whole list (headings included), and a page that starts inside a group or a month
/// gets the heading of it, with the totals of the WHOLE group or month, in `group_above`.
pub(crate) fn cut(page: &mut ReportPage, spec: PageSpec) {
    let all = std::mem::take(&mut page.rows);
    page.total_count = i64::try_from(all.len()).unwrap_or(i64::MAX);
    let skip = usize::try_from(spec.offset).unwrap_or(usize::MAX);
    let take = spec
        .limit
        .map_or(usize::MAX, |l| usize::try_from(l).unwrap_or(usize::MAX));
    let starts_with_heading = all.get(skip).is_some_and(|row| row.kind != RowKind::Normal);
    page.group_above = if skip > 0 && !starts_with_heading {
        all.get(..skip.min(all.len()))
            .and_then(|before| before.iter().rev().find(|r| r.kind != RowKind::Normal))
            .cloned()
    } else {
        None
    };
    page.rows = all.into_iter().skip(skip).take(take).collect();
    page.offset = spec.offset;
    page.limit = spec.reported_limit();
}
