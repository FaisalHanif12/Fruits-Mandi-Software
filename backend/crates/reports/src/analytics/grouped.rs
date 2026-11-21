//! Day, month or year grouping for the views whose rows are items, sellers, buyers, partners, gardens,
//! trips or fruit (views A, B, C, E, L, F, I, J and K, and the partner part of view M; PRD 18.2,
//! FR-18.9, plan 77 coverage audit).
//!
//! Each of these views reads its rows for one period. With a grouping, the view is read once for each
//! day, month or year of the period (the first and last bucket are cut to the period) and every row
//! gets the bucket as its first cell, so the Owner reads, for example, each partner's net change month
//! by month. The totals row is the view read once over the whole period, never a sum of the buckets,
//! so it is the figure of the ungrouped view. A row with no figure in its bucket (a partner or a garden
//! the view lists with all zeros) and a bucket with no row are left out.
//!
//! Without a first and a last day the period is the days that have a record of any kind (the journal,
//! the purchases, the gardens and trips, the fruit entries). At most 366 buckets are read.
//!
//! Example (the worked day, view F by month, October 2026): one row for each partner for 2026-10, the
//! same figures as the ungrouped view; the totals row is the same too.

use mandi_core::dates::BusinessDate;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_core::months::Month;
use mandi_db::rusqlite::Connection;

use super::filters::Typed;
use super::ids;
use super::measure::Figure;
use super::model::{
    AnalyticsColumn, AnalyticsExpensePart, AnalyticsGroupBy, AnalyticsRow, AnalyticsView,
};
use super::table::{Table, col, count_of};
use super::{
    buyer, buyer_matrix, expense_partners, fruit, gardens, item, matrix, partner, seller, trips,
};
use crate::error::ReportsError;
use crate::reports::{Cell, ColumnKind, PageSpec};
use crate::result_cache;

/// Most buckets one grouped view reads.
const MAX_BUCKETS: usize = 366;

/// One bucket: its label cell and its first and last day.
struct Bucket {
    label: Cell,
    from: BusinessDate,
    to: BusinessDate,
}

/// True when the view, with these filters, groups its rows by period: F, I, J and K when a grouping
/// is asked for, and the partner part of view M.
pub(crate) fn applies(view: AnalyticsView, typed: &Typed) -> bool {
    typed.rows_by.is_some() && takes_row_grouping(view, typed.expense_part)
}

/// True for the views that can group their rows by period; for view M only its partner part (the
/// shop part groups its columns).
pub(crate) fn takes_row_grouping(view: AnalyticsView, part: AnalyticsExpensePart) -> bool {
    match view {
        AnalyticsView::A
        | AnalyticsView::B
        | AnalyticsView::C
        | AnalyticsView::E
        | AnalyticsView::F
        | AnalyticsView::I
        | AnalyticsView::J
        | AnalyticsView::K
        | AnalyticsView::L => true,
        AnalyticsView::M => part == AnalyticsExpensePart::Partner,
        _ => false,
    }
}

/// The first and last day of any record the views read, or `None` when the books are empty.
fn data_bounds(conn: &Connection) -> Result<Option<(BusinessDate, BusinessDate)>, ReportsError> {
    // Each end is its own subquery: a MIN and a MAX in one SELECT cannot use the two ends of the
    // index and scan all of it (0.2 seconds for a million journal entries).
    let (lo, hi): (Option<String>, Option<String>) = conn.query_row(
        "SELECT MIN(lo), MAX(hi) FROM (
             SELECT (SELECT MIN(entry_date) FROM journal_entries) AS lo,
                    (SELECT MAX(entry_date) FROM journal_entries) AS hi
             UNION ALL SELECT (SELECT MIN(entry_date) FROM purchases),
                              (SELECT MAX(entry_date) FROM purchases)
             UNION ALL SELECT (SELECT MIN(entry_date) FROM ventures),
                              (SELECT MAX(entry_date) FROM ventures)
             UNION ALL SELECT (SELECT MIN(entry_date) FROM venture_entries),
                              (SELECT MAX(entry_date) FROM venture_entries)
             UNION ALL SELECT (SELECT MIN(entry_date) FROM fruit_takes),
                              (SELECT MAX(entry_date) FROM fruit_takes))",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    Ok(lo
        .as_deref()
        .and_then(BusinessDate::parse_iso)
        .zip(hi.as_deref().and_then(BusinessDate::parse_iso)))
}

/// The buckets of a period, oldest first, the first and last cut to the period.
fn buckets_of(
    group: AnalyticsGroupBy,
    first: BusinessDate,
    last: BusinessDate,
) -> Result<Vec<Bucket>, ReportsError> {
    let too_many = || {
        ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.group_by",
        )])
    };
    let bad = || ReportsError::Data("a period bucket does not exist".to_owned());
    let mut out: Vec<Bucket> = Vec::new();
    let mut cursor = first;
    while cursor <= last {
        let (from, to, next, label) = match group {
            AnalyticsGroupBy::Day => (
                cursor,
                cursor,
                cursor.next_day().ok_or_else(bad)?,
                Cell::Date {
                    iso: cursor.to_iso(),
                },
            ),
            AnalyticsGroupBy::Month => {
                let month = Month::of(cursor);
                (
                    cursor,
                    month.last_day().ok_or_else(bad)?.min(last),
                    month.next().first_day().ok_or_else(bad)?,
                    Cell::Month {
                        year: i64::from(cursor.year()),
                        month: i64::from(cursor.month()),
                    },
                )
            }
            AnalyticsGroupBy::Year => (
                cursor,
                BusinessDate::from_ymd(cursor.year(), 12, 31)
                    .ok_or_else(bad)?
                    .min(last),
                BusinessDate::from_ymd(cursor.year().checked_add(1).ok_or_else(bad)?, 1, 1)
                    .ok_or_else(bad)?,
                Cell::Number {
                    value: i64::from(cursor.year()),
                },
            ),
        };
        if out.len() >= MAX_BUCKETS {
            return Err(too_many());
        }
        out.push(Bucket { label, from, to });
        cursor = next;
    }
    Ok(out)
}

/// Every row, no page.
const EVERY_ROW: PageSpec = PageSpec {
    offset: 0,
    limit: None,
};

/// No row at all: the totals row, the columns and the count of the rows.
const COUNT_ONLY: PageSpec = PageSpec {
    offset: 0,
    limit: Some(0),
};

/// The view read for the days of `typed`, the rows of `spec`.
fn run_view(
    conn: &Connection,
    typed: &Typed,
    view: AnalyticsView,
    spec: PageSpec,
) -> Result<Table, ReportsError> {
    match view {
        AnalyticsView::A => item::run(conn, typed, spec),
        AnalyticsView::B => seller::run(conn, typed, spec),
        AnalyticsView::C => buyer::run(conn, typed, spec),
        AnalyticsView::E => matrix::run_e(conn, typed, spec),
        AnalyticsView::L => buyer_matrix::run_l(conn, typed, spec),
        AnalyticsView::F => partner::run(conn, typed, spec),
        AnalyticsView::I => gardens::run(conn, typed, spec),
        AnalyticsView::J => trips::run(conn, typed, spec),
        AnalyticsView::K => fruit::run(conn, typed, spec),
        AnalyticsView::M => expense_partners::run(conn, typed, spec),
        _ => Err(ReportsError::Data(
            "this view has no row grouping".to_owned(),
        )),
    }
}

/// True for the sale views A, B, C, E and L: their rows come from the facts, none of them is blank,
/// and a page of them is read by counting the rows of each bucket first.
pub(crate) fn is_sale_view(view: AnalyticsView) -> bool {
    matches!(
        view,
        AnalyticsView::A
            | AnalyticsView::B
            | AnalyticsView::C
            | AnalyticsView::E
            | AnalyticsView::L
    )
}

/// True when a request for `view` with this grouping reads a page, not every row, so the table
/// holds only the page (`run` cuts it itself).
pub(crate) fn pages_itself(view: AnalyticsView, group_by: Option<AnalyticsGroupBy>) -> bool {
    is_sale_view(view) && group_by.is_some()
}

/// True when no cell of the row holds an amount or a count other than 0: only names and labels.
fn is_blank(cells: &[Cell]) -> bool {
    !cells.iter().any(|cell| match cell {
        Cell::Money { paisa } | Cell::Balance { paisa, .. } => *paisa != Paisa::ZERO,
        Cell::Number { value } | Cell::Percent { hundredths: value } => *value != 0,
        Cell::Weight { grams } => *grams != 0,
        _ => false,
    })
}

/// The period column and its kind for a grouping.
fn period_column(group: AnalyticsGroupBy) -> AnalyticsColumn {
    let kind = match group {
        AnalyticsGroupBy::Day => ColumnKind::Date,
        AnalyticsGroupBy::Month => ColumnKind::Month,
        AnalyticsGroupBy::Year => ColumnKind::Number,
    };
    col("period", ids::COL_PERIOD, kind)
}

/// Lays one row of a bucket under the columns of the whole period: the period first, then the cells by
/// column key. `None` for a row with no figure at all (a partner or a garden the view lists with all
/// zeros).
fn lay_row(
    bucket: &Bucket,
    view: AnalyticsView,
    whole: &Table,
    other_at: Option<usize>,
    table_columns: &[AnalyticsColumn],
    source: AnalyticsRow,
) -> Result<Option<AnalyticsRow>, ReportsError> {
    // A view lists a partner or a garden even when he has no figure in the bucket: such a row says
    // nothing, and a long grouping would be full of them. The sale views never list such a row, and
    // keeping all of theirs keeps the count of a bucket equal to its rows.
    if !is_sale_view(view) && is_blank(&source.cells) {
        return Ok(None);
    }
    let place: Vec<Option<usize>> = whole
        .columns
        .iter()
        .map(|c| table_columns.iter().position(|d| d.key == c.key))
        .collect();
    let mut cells = vec![bucket.label.clone()];
    cells.extend(place.iter().map(|at| {
        at.and_then(|i| source.cells.get(i))
            .cloned()
            .unwrap_or(Cell::Empty)
    }));
    // Views E and L keep at most 24 item columns over the whole period. Each bucket is read with a
    // column for every item, so an item the whole period put in "other items" is added to that cell
    // here, and the cells of a row always add up to the row.
    // The cells are money, or whole numbers when the measure is the quantity (PRD FR-18.10).
    if let Some(other) = other_at {
        let mut rest: Option<Figure> = None;
        for (j, column) in table_columns.iter().enumerate() {
            let is_placed = whole.columns.iter().any(|c| c.key == column.key);
            if column.key.starts_with("i_")
                && !is_placed
                && let Some(figure) = source.cells.get(j).and_then(Figure::from_cell)
            {
                rest = Some(match rest {
                    Some(held) => held.plus(figure)?,
                    None => figure,
                });
            }
        }
        if let Some(held) = cells.get_mut(other.saturating_add(1)) {
            *held = rest.map_or(Cell::Empty, Figure::cell_or_empty);
        }
    }
    // A row of the sale views, F, K and the partner part of M opens its lines inside the bucket. A
    // row of view I or J is picked by its garden or purchases, whose sales fall on any later day, so
    // its key stays.
    let key = source.key.map(|mut key| {
        if !matches!(view, AnalyticsView::I | AnalyticsView::J) {
            key.from = Some(bucket.from.to_iso());
            key.to = Some(bucket.to.to_iso());
        }
        key
    });
    Ok(Some(AnalyticsRow {
        cells,
        key,
        kind: source.kind,
    }))
}

/// The view for the days of one bucket: the filters of the run cut to the bucket, with no
/// comparison (it is of the whole period, on the totals row) and a column for every item.
fn bucket_typed(typed: &Typed, bucket: &Bucket) -> Typed {
    let mut part = typed.clone();
    part.from = Some(bucket.from.to_iso());
    part.to = Some(bucket.to.to_iso());
    part.compare = None;
    part.every_item_column = true;
    part
}

/// The first row and the end of the page, as row numbers over all the buckets (`None`: no end).
fn window_of(spec: PageSpec) -> (usize, Option<usize>) {
    let start = usize::try_from(spec.offset).unwrap_or(usize::MAX);
    let end = spec
        .limit
        .map(|l| start.saturating_add(usize::try_from(l).unwrap_or(usize::MAX)));
    (start, end)
}

/// The buckets of a grouped view: the first and last day of the filter (or of the books), cut to the
/// grouping.
fn buckets_for(
    conn: &Connection,
    typed: &Typed,
    group: AnalyticsGroupBy,
) -> Result<Vec<Bucket>, ReportsError> {
    let first = typed.from.as_deref().and_then(BusinessDate::parse_iso);
    let last = typed.to.as_deref().and_then(BusinessDate::parse_iso);
    // The books are asked for their first and last day only when the filter lacks one of them.
    let bounds = if first.is_some() && last.is_some() {
        None
    } else {
        data_bounds(conn)?
    };
    let from = first.or_else(|| bounds.map(|(lo, _)| lo));
    let to = last.or_else(|| bounds.map(|(_, hi)| hi));
    match (from, to) {
        (Some(from), Some(to)) => buckets_of(group, from, to),
        _ => Ok(Vec::new()),
    }
}

/// What a page of a grouped sale view needs besides its own rows: the columns and the totals of the
/// whole period, the buckets and how many rows each has. The next page of the same filter takes it
/// from memory (`result_cache`), so it reads only the buckets of its own rows.
struct Counted {
    whole: Table,
    buckets: Vec<Bucket>,
    counts: Vec<usize>,
}

fn counted(
    conn: &Connection,
    typed: &Typed,
    view: AnalyticsView,
    group: AnalyticsGroupBy,
) -> Result<Counted, ReportsError> {
    let whole = run_view(conn, typed, view, COUNT_ONLY)?;
    let buckets = buckets_for(conn, typed, group)?;
    // The rows of each bucket, counted with no row built.
    let mut counts: Vec<usize> = Vec::with_capacity(buckets.len());
    for bucket in &buckets {
        let part = bucket_typed(typed, bucket);
        let table = run_view(conn, &part, view, COUNT_ONLY)?;
        counts.push(usize::try_from(table.total_count).unwrap_or(0));
    }
    Ok(Counted {
        whole,
        buckets,
        counts,
    })
}

/// Builds a grouped view: the whole period for the columns and the totals, then each bucket for the
/// rows.
///
/// A page of a sale view (A, B, C, E, L) is read by counting the rows of every bucket first, with no
/// row built, then reading only the buckets that hold the rows of the page. Page 2 of a ten-year
/// view by month is then two buckets of rows, not a hundred and twenty, and the counts are kept for
/// the next page of the same filter (robustness finding 2).
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `filters.group_by` for more than 366 buckets; the errors of the view; a
/// database or money error.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    view: AnalyticsView,
    group: AnalyticsGroupBy,
    spec: PageSpec,
) -> Result<Table, ReportsError> {
    // A request for every row (a print, an export) is read bucket by bucket in one pass.
    if is_sale_view(view) && (spec.offset > 0 || spec.limit.is_some()) {
        return run_page(conn, typed, view, group, spec);
    }
    let whole = run_view(conn, typed, view, EVERY_ROW)?;
    let buckets = buckets_for(conn, typed, group)?;
    // The rows of the buckets are laid under the columns of the whole period, by key, so a column that
    // only some buckets have (a partner of one garden) stays under its own heading.
    let other_at = whole.columns.iter().position(|c| c.key == "i_other");
    let mut rows: Vec<AnalyticsRow> = Vec::new();
    for bucket in &buckets {
        let part = bucket_typed(typed, bucket);
        let table = run_view(conn, &part, view, EVERY_ROW)?;
        for source in table.rows {
            if let Some(row) = lay_row(bucket, view, &whole, other_at, &table.columns, source)? {
                rows.push(row);
            }
        }
    }
    let total_count = rows.len();
    let mut columns = vec![period_column(group)];
    columns.extend(whole.columns);
    let mut totals = vec![Cell::Empty];
    totals.extend(whole.totals);
    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(total_count),
        rows: spec.slice(&rows),
        totals,
    })
}

/// One page of a grouped sale view: the counts from memory (or counted now), then only the buckets
/// that hold the rows of the page.
fn run_page(
    conn: &Connection,
    typed: &Typed,
    view: AnalyticsView,
    group: AnalyticsGroupBy,
    spec: PageSpec,
) -> Result<Table, ReportsError> {
    // The question without its page: every page of one filter takes the same counts.
    let what = format!("analytics grouped|{view:?}|{group:?}|{typed:?}");
    let counted = result_cache::remembered(conn, &what, || counted(conn, typed, view, group))?;
    let whole = &counted.whole;
    let other_at = whole.columns.iter().position(|c| c.key == "i_other");
    let total_count: usize = counted
        .counts
        .iter()
        .copied()
        .fold(0, usize::saturating_add);
    let (start, end) = window_of(spec);
    let end = end.unwrap_or(usize::MAX);
    let mut rows: Vec<AnalyticsRow> = Vec::new();
    let mut base: usize = 0;
    for (bucket, count) in counted.buckets.iter().zip(&counted.counts) {
        let bucket_end = base.saturating_add(*count);
        let low = start.max(base);
        let high = end.min(bucket_end);
        let first = base;
        base = bucket_end;
        if low >= high {
            continue;
        }
        let sub = PageSpec {
            offset: i64::try_from(low.saturating_sub(first)).unwrap_or(i64::MAX),
            limit: Some(i64::try_from(high.saturating_sub(low)).unwrap_or(i64::MAX)),
        };
        let part = bucket_typed(typed, bucket);
        let table = run_view(conn, &part, view, sub)?;
        for source in table.rows {
            if let Some(row) = lay_row(bucket, view, whole, other_at, &table.columns, source)? {
                rows.push(row);
            }
        }
    }
    let mut columns = vec![period_column(group)];
    columns.extend(whole.columns.iter().cloned());
    let mut totals = vec![Cell::Empty];
    totals.extend(whole.totals.iter().cloned());
    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(total_count),
        rows,
        totals,
    })
}

#[cfg(test)]
mod tests {
    //! The "other items" cell of a grouped matrix adds the columns the whole period left out, in money
    //! or, for the quantity measure, in whole numbers (PRD FR-18.10, plan 84).

    use super::*;

    fn bucket() -> Bucket {
        let day = BusinessDate::from_ymd(2026, 10, 1).unwrap();
        Bucket {
            label: Cell::Month {
                year: 2026,
                month: 10,
            },
            from: day,
            to: BusinessDate::from_ymd(2026, 10, 31).unwrap(),
        }
    }

    fn laid(kind: ColumnKind, cells: Vec<Cell>) -> Vec<Cell> {
        let key = |k: &str| col(k, "title", kind);
        // The whole period: seller, item 1, "other items", total. This bucket was read with a column
        // for every item: seller, items 1, 2 and 3, total.
        let whole = Table {
            columns: vec![key("seller"), key("i_1"), key("i_other"), key("qty")],
            rows: Vec::new(),
            total_count: 0,
            totals: Vec::new(),
            charts: Vec::new(),
        };
        let table_columns = vec![
            key("seller"),
            key("i_1"),
            key("i_2"),
            key("i_3"),
            key("qty"),
        ];
        let source = AnalyticsRow {
            cells,
            key: None,
            kind: crate::reports::RowKind::Normal,
        };
        lay_row(
            &bucket(),
            AnalyticsView::E,
            &whole,
            Some(2),
            &table_columns,
            source,
        )
        .unwrap()
        .unwrap()
        .cells
    }

    #[test]
    fn the_other_items_of_a_count_are_added_as_a_count() {
        let n = |value| Cell::Number { value };
        let cells = laid(
            ColumnKind::Number,
            vec![Cell::Empty, n(5), n(3), n(4), n(12)],
        );
        // The period, the seller, item 1, "other items" = 3 + 4, the total.
        assert_eq!(cells[3], n(7));
        assert_eq!(cells[2], n(5));
        // Nothing left out is an empty cell.
        let cells = laid(
            ColumnKind::Number,
            vec![Cell::Empty, n(5), Cell::Empty, Cell::Empty, n(5)],
        );
        assert_eq!(cells[3], Cell::Empty);
    }

    #[test]
    fn the_other_items_of_money_are_added_as_money() {
        let m = |rupees| Cell::Money {
            paisa: Paisa::from_rupees(rupees).unwrap(),
        };
        let cells = laid(
            ColumnKind::Money,
            vec![Cell::Empty, m(500), m(300), m(400), m(1_200)],
        );
        assert_eq!(cells[3], m(700));
    }
}
