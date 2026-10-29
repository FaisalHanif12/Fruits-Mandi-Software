//! The chart of an analytics view and the sort of its rows (plan 38 R4, decision D1, PRD FR-18.3).
//!
//! The chart is made from the rows of the WHOLE filter, never from the page, so it does not change
//! when the table is sorted or paged. Its numbers are the cells of the table (the same stored
//! sums), added with checked maths:
//!
//! - **Ranked** views (A by item, B by seller, C by buyer, E seller by item, H top 10, I gardens,
//!   J by trip, purchase or item): the 10 biggest rows by the main measure of the view, and, when
//!   there are more rows, one point "others" that adds up the rest.
//! - **Trend** views (D item by period, G trend, J by day): one point for every period of the
//!   filter. A day grouping with more than 120 points is turned into months (`by_month`).
//!
//! Example (the worked day, view A): the points کیلا 3,928 and سیب 1,520, no "others".

use std::cmp::Ordering;

use mandi_core::money::Paisa;
use mandi_core::paging::SortInput;

use super::model::{
    AnalyticsChart, AnalyticsChartKind, AnalyticsChartPoint, AnalyticsChartSeries, AnalyticsColumn,
    AnalyticsHeading, AnalyticsRow, AnalyticsTripView, AnalyticsView,
};
use crate::error::ReportsError;
use crate::reports::view::compare;
use crate::reports::{Cell, ColumnKind, RowKind};
use mandi_core::error::{ErrorCode, ErrorValue, FieldError};

/// How many points a ranked chart has before "others".
pub(crate) const TOP: usize = 10;
/// The most points a trend chart has before days are turned into months.
pub(crate) const MAX_TREND_POINTS: usize = 120;

/// What a view charts.
enum Plan {
    /// No chart (the table is short, or a chart means nothing).
    None,
    /// The biggest rows by one column, in sections when the rows have heading rows.
    Ranked(&'static str),
    /// One point for each row, with these columns as series.
    TrendRows(&'static [&'static str]),
    /// One point for each period column of the table, with the totals row as the value.
    TrendColumns,
}

fn plan_of(view: AnalyticsView, trip_view: Option<AnalyticsTripView>) -> Plan {
    match view {
        AnalyticsView::A | AnalyticsView::B | AnalyticsView::E | AnalyticsView::H => {
            Plan::Ranked("total_comm")
        }
        AnalyticsView::C => Plan::Ranked("purchase_value"),
        AnalyticsView::I => Plan::Ranked("result"),
        AnalyticsView::J => match trip_view {
            Some(AnalyticsTripView::Day) => Plan::TrendRows(&["sales_money", "result"]),
            _ => Plan::Ranked("result"),
        },
        AnalyticsView::D => Plan::TrendColumns,
        AnalyticsView::G => Plan::TrendRows(&[
            "gross",
            "commission",
            "shared_income",
            "shop_expenses",
            "profit",
        ]),
        AnalyticsView::F | AnalyticsView::K => Plan::None,
    }
}

/// The index of the column with this key.
fn index_of(columns: &[AnalyticsColumn], key: &str) -> Option<usize> {
    columns.iter().position(|c| c.key == key)
}

/// A cell that adds up (money or a whole number), as a number, with its kind.
fn amount(cell: &Cell) -> Option<(i64, ColumnKind)> {
    match cell {
        Cell::Money { paisa } => Some((paisa.as_paisa(), ColumnKind::Money)),
        Cell::Number { value } => Some((*value, ColumnKind::Number)),
        _ => None,
    }
}

/// A cell of the kind, from a sum.
fn cell_of(kind: ColumnKind, value: i64) -> Result<Cell, ReportsError> {
    Ok(match kind {
        ColumnKind::Number => Cell::Number { value },
        _ => Cell::Money {
            paisa: Paisa::try_from_paisa(value)?,
        },
    })
}

fn overflow() -> ReportsError {
    mandi_core::money::MoneyError::Overflow.into()
}

/// Ranked points: the biggest `TOP` rows by `measure`, then "others". The heading rows split the
/// rows into sections, one chart for each.
fn ranked(
    columns: &[AnalyticsColumn],
    rows: &[AnalyticsRow],
    measure: &str,
) -> Result<Vec<AnalyticsChart>, ReportsError> {
    let Some(at) = index_of(columns, measure) else {
        return Ok(Vec::new());
    };
    let kind = columns.get(at).map_or(ColumnKind::Money, |c| c.kind);
    let series = vec![AnalyticsChartSeries {
        key: measure.to_owned(),
        kind,
    }];
    // The sections: the rows between two heading rows. Without heading rows, one section.
    let mut sections: Vec<(Option<&AnalyticsRow>, Vec<&AnalyticsRow>)> = Vec::new();
    for row in rows {
        if row.kind == RowKind::Normal {
            match sections.last_mut() {
                Some((_, members)) => members.push(row),
                None => sections.push((None, vec![row])),
            }
        } else {
            sections.push((Some(row), Vec::new()));
        }
    }
    let mut charts = Vec::new();
    for (heading, mut members) in sections {
        if members.is_empty() {
            continue;
        }
        // Biggest first; equal rows keep the order of the table.
        members.sort_by(|a, b| {
            let value = |r: &AnalyticsRow| r.cells.get(at).and_then(amount).map_or(0, |v| v.0);
            value(b).cmp(&value(a))
        });
        let mut points = Vec::new();
        let mut rest = 0_i64;
        let mut rest_count = 0_i64;
        for (rank, row) in members.iter().enumerate() {
            let value = row.cells.get(at).and_then(amount).map_or(0, |v| v.0);
            if rank < TOP {
                points.push(AnalyticsChartPoint {
                    label: row.cells.first().cloned().unwrap_or(Cell::Empty),
                    key: row.key.clone(),
                    values: vec![cell_of(kind, value)?],
                    is_others: false,
                });
            } else {
                rest = rest.checked_add(value).ok_or_else(overflow)?;
                rest_count = rest_count.checked_add(1).ok_or_else(overflow)?;
            }
        }
        if rest_count > 0 {
            points.push(AnalyticsChartPoint {
                label: Cell::Empty,
                key: None,
                values: vec![cell_of(kind, rest)?],
                is_others: true,
            });
        }
        charts.push(AnalyticsChart {
            kind: AnalyticsChartKind::Ranked,
            title: heading.and_then(|h| h.cells.first().cloned()),
            series: series.clone(),
            points,
            others_count: rest_count,
            by_month: false,
        });
    }
    Ok(charts)
}

/// Trend points from rows: one for each row, in the order of the table, and the same for every
/// series. Days become months when there are too many.
fn trend_rows(
    columns: &[AnalyticsColumn],
    rows: &[AnalyticsRow],
    keys: &[&str],
) -> Result<Vec<AnalyticsChart>, ReportsError> {
    let at: Vec<(usize, ColumnKind)> = keys
        .iter()
        .filter_map(|key| {
            index_of(columns, key)
                .map(|i| (i, columns.get(i).map_or(ColumnKind::Money, |c| c.kind)))
        })
        .collect();
    if at.is_empty() {
        return Ok(Vec::new());
    }
    let series: Vec<AnalyticsChartSeries> = at
        .iter()
        .filter_map(|(i, kind)| {
            columns.get(*i).map(|c| AnalyticsChartSeries {
                key: c.key.clone(),
                kind: *kind,
            })
        })
        .collect();
    let normal: Vec<&AnalyticsRow> = rows.iter().filter(|r| r.kind == RowKind::Normal).collect();
    let mut points: Vec<AnalyticsChartPoint> = Vec::with_capacity(normal.len());
    for row in &normal {
        let mut values = Vec::with_capacity(at.len());
        for (i, kind) in &at {
            let v = row.cells.get(*i).and_then(amount).map_or(0, |v| v.0);
            values.push(cell_of(*kind, v)?);
        }
        points.push(AnalyticsChartPoint {
            label: row.cells.first().cloned().unwrap_or(Cell::Empty),
            key: row.key.clone(),
            values,
            is_others: false,
        });
    }
    let mut by_month = false;
    let days = points.iter().all(|p| matches!(p.label, Cell::Date { .. }));
    if days && points.len() > MAX_TREND_POINTS {
        by_month = true;
        points = months_of(&points, &series)?;
    }
    Ok(vec![AnalyticsChart {
        kind: AnalyticsChartKind::Trend,
        title: None,
        series,
        points,
        others_count: 0,
        by_month,
    }])
}

/// Adds the points of a day chart month by month (checked sums), in date order.
fn months_of(
    points: &[AnalyticsChartPoint],
    series: &[AnalyticsChartSeries],
) -> Result<Vec<AnalyticsChartPoint>, ReportsError> {
    let mut months: Vec<(String, i64, i64, Vec<i64>)> = Vec::new();
    for point in points {
        let Cell::Date { iso } = &point.label else {
            continue;
        };
        let key = iso.get(..7).unwrap_or(iso.as_str()).to_owned();
        let year = iso.get(..4).and_then(|y| y.parse().ok()).unwrap_or(0);
        let month = iso.get(5..7).and_then(|m| m.parse().ok()).unwrap_or(0);
        let at = match months.iter().position(|m| m.0 == key) {
            Some(at) => at,
            None => {
                months.push((key, year, month, vec![0; series.len()]));
                months.len().saturating_sub(1)
            }
        };
        let Some(entry) = months.get_mut(at) else {
            continue;
        };
        for (sum, value) in entry.3.iter_mut().zip(&point.values) {
            let add = amount(value).map_or(0, |v| v.0);
            *sum = sum.checked_add(add).ok_or_else(overflow)?;
        }
    }
    months.sort_by(|a, b| a.0.cmp(&b.0));
    months
        .into_iter()
        .map(|(_, year, month, sums)| {
            let mut values = Vec::with_capacity(sums.len());
            for (sum, s) in sums.into_iter().zip(series) {
                values.push(cell_of(s.kind, sum)?);
            }
            Ok(AnalyticsChartPoint {
                label: Cell::Month { year, month },
                key: None,
                values,
                is_others: false,
            })
        })
        .collect()
}

/// Trend points from the period columns of a view D: one for each column that has a heading, with
/// the value of the totals row.
fn trend_columns(
    columns: &[AnalyticsColumn],
    totals: &[Cell],
) -> Result<Vec<AnalyticsChart>, ReportsError> {
    let mut points = Vec::new();
    let mut kind = ColumnKind::Money;
    for (i, column) in columns.iter().enumerate() {
        let label = match &column.heading {
            Some(AnalyticsHeading::Month { year, month }) => Cell::Month {
                year: *year,
                month: *month,
            },
            Some(AnalyticsHeading::Day { iso }) => Cell::Date { iso: iso.clone() },
            Some(AnalyticsHeading::Year { year }) => Cell::Number { value: *year },
            _ => continue,
        };
        let (value, k) = totals
            .get(i)
            .and_then(amount)
            .unwrap_or((0, ColumnKind::Money));
        kind = k;
        points.push(AnalyticsChartPoint {
            label,
            key: column.drill.clone(),
            values: vec![cell_of(k, value)?],
            is_others: false,
        });
    }
    if points.is_empty() {
        return Ok(Vec::new());
    }
    let mut by_month = false;
    let series = vec![AnalyticsChartSeries {
        key: "total_comm".to_owned(),
        kind,
    }];
    if points.len() > MAX_TREND_POINTS
        && points.iter().all(|p| matches!(p.label, Cell::Date { .. }))
    {
        by_month = true;
        points = months_of(&points, &series)?;
    }
    Ok(vec![AnalyticsChart {
        kind: AnalyticsChartKind::Trend,
        title: None,
        series,
        points,
        others_count: 0,
        by_month,
    }])
}

/// The charts of a view built from its whole table. An empty list when the view has no chart or
/// the table has no row.
///
/// # Errors
/// `ReportsError::Money` if a sum does not fit.
pub(crate) fn build(
    view: AnalyticsView,
    trip_view: Option<AnalyticsTripView>,
    columns: &[AnalyticsColumn],
    rows: &[AnalyticsRow],
    totals: &[Cell],
) -> Result<Vec<AnalyticsChart>, ReportsError> {
    match plan_of(view, trip_view) {
        Plan::None => Ok(Vec::new()),
        Plan::Ranked(measure) => ranked(columns, rows, measure),
        Plan::TrendRows(keys) => trend_rows(columns, rows, keys),
        Plan::TrendColumns => trend_columns(columns, totals),
    }
}

/// Sorts the rows of a whole table by the column the sort names. The rows between two heading
/// rows are sorted among themselves; the heading rows stay. A stable sort.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `sort.column` if no column has that key.
pub(crate) fn sort_rows(
    columns: &[AnalyticsColumn],
    rows: Vec<AnalyticsRow>,
    sort: &SortInput,
) -> Result<Vec<AnalyticsRow>, ReportsError> {
    let Some(at) = index_of(columns, &sort.column) else {
        let allowed = columns
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>()
            .join(",");
        return Err(ReportsError::Rejected(vec![
            FieldError::new(ErrorCode::ErrOutOfRange, "sort.column")
                .with("allowed", ErrorValue::Text(allowed)),
        ]));
    };
    let by = |a: &AnalyticsRow, b: &AnalyticsRow| -> Ordering {
        let empty = Cell::Empty;
        sort.direction.apply(compare(
            a.cells.get(at).unwrap_or(&empty),
            b.cells.get(at).unwrap_or(&empty),
        ))
    };
    let mut out = Vec::with_capacity(rows.len());
    let mut segment: Vec<AnalyticsRow> = Vec::new();
    for row in rows {
        if row.kind == RowKind::Normal {
            segment.push(row);
        } else {
            segment.sort_by(&by);
            out.append(&mut segment);
            out.push(row);
        }
    }
    segment.sort_by(&by);
    out.append(&mut segment);
    Ok(out)
}
