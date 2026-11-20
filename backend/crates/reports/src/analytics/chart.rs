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
//! With a measure (PRD FR-18.10, plan 84) the chart follows it: the ranking and the trend are of
//! the measure (`gross`, `buyer_total`, `qty`, `total_comm`, `seller_comm`, `buyer_comm`, `laga`),
//! the series is named by its key, and a quantity is a count, never money. The matrices D, E and L
//! and the top ten H carry the measure in their own cells, so their chart reads it from the table
//! as before. The tables of A, B, C and G keep every column, so those views make their own chart
//! from the stored sums of their rows ([`ranked_chart`], [`trend_chart`]).
//!
//! Example (the worked day, view A): the points کیلا 3,928 and سیب 1,520, no "others". By خام بکری
//! the points are کیلا 29,400 and سیب 12,000.

use std::cmp::Ordering;

use mandi_core::money::Paisa;
use mandi_core::paging::SortInput;

use super::measure::Figure;
use super::model::{
    AnalyticsChart, AnalyticsChartKind, AnalyticsChartPoint, AnalyticsChartSeries, AnalyticsColumn,
    AnalyticsHeading, AnalyticsKey, AnalyticsMeasure, AnalyticsRow, AnalyticsTripView,
    AnalyticsView,
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
    /// The biggest rows by one column, in sections when the rows have heading rows.
    Ranked(&'static str),
    /// One point for each row, with these columns as series.
    TrendRows(&'static [&'static str]),
    /// One point for each period column of the table, with the totals row as the value. The key
    /// is the series' name.
    TrendColumns(&'static str),
}

/// True when the view makes its own chart for the measure asked for: A, B, C and G, whose tables
/// keep every column whatever the measure is (PRD FR-18.10). Without a measure they chart as before.
pub(crate) fn makes_own_chart(view: AnalyticsView, measure: Option<AnalyticsMeasure>) -> bool {
    measure.is_some()
        && matches!(
            view,
            AnalyticsView::A | AnalyticsView::B | AnalyticsView::C | AnalyticsView::G
        )
}

fn plan_of(
    view: AnalyticsView,
    trip_view: Option<AnalyticsTripView>,
    measure: Option<AnalyticsMeasure>,
) -> Plan {
    // The key of the column that holds the measure in D, E, H and L: the total commission when no
    // measure is asked for.
    let key = measure.unwrap_or(AnalyticsMeasure::TotalComm).key();
    match view {
        AnalyticsView::A | AnalyticsView::B => Plan::Ranked("total_comm"),
        AnalyticsView::E | AnalyticsView::H | AnalyticsView::L => Plan::Ranked(key),
        AnalyticsView::C => Plan::Ranked("purchase_value"),
        AnalyticsView::I => Plan::Ranked("result"),
        AnalyticsView::J => match trip_view {
            Some(AnalyticsTripView::Day) => Plan::TrendRows(&["sales_money", "result"]),
            _ => Plan::Ranked("result"),
        },
        AnalyticsView::D => Plan::TrendColumns(key),
        // Expenses by head: the net expense of each period (plan 77).
        AnalyticsView::M => Plan::TrendColumns("expense"),
        AnalyticsView::G => Plan::TrendRows(&[
            "gross",
            "commission",
            "shared_income",
            "shop_expenses",
            "profit",
        ]),
        // The partners by their net change, behind the partner code like the view itself; the fruit
        // taken by value (plan 76 point 4).
        AnalyticsView::F => Plan::Ranked("net_change"),
        AnalyticsView::K => Plan::Ranked("value"),
    }
}

/// The index of the column that names a row: the first one that is not the rank (view H puts the
/// rank first and the name second; the point is called by the name, in both scripts like the
/// table, never by "1", "2").
fn label_index(columns: &[AnalyticsColumn]) -> usize {
    columns.iter().position(|c| c.key != "rank").unwrap_or(0)
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
    let named_by = label_index(columns);
    // With a compare on, the table has a `compare` column: the compared period's figure of the
    // main measure. It becomes the second series of the chart (plan 76 point 4).
    let compare_at = index_of(columns, "compare");
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
        let mut rest_compare = 0_i64;
        let mut rest_count = 0_i64;
        for (rank, row) in members.iter().enumerate() {
            let value = row.cells.get(at).and_then(amount).map_or(0, |v| v.0);
            let compared = compare_at.and_then(|i| row.cells.get(i)).and_then(amount);
            if rank < TOP {
                points.push(AnalyticsChartPoint {
                    label: row.cells.get(named_by).cloned().unwrap_or(Cell::Empty),
                    key: row.key.clone(),
                    values: vec![cell_of(kind, value)?],
                    is_others: false,
                    compare: compared.map(|(v, _)| cell_of(kind, v)).transpose()?,
                });
            } else {
                rest = rest.checked_add(value).ok_or_else(overflow)?;
                rest_compare = rest_compare
                    .checked_add(compared.map_or(0, |c| c.0))
                    .ok_or_else(overflow)?;
                rest_count = rest_count.checked_add(1).ok_or_else(overflow)?;
            }
        }
        if rest_count > 0 {
            points.push(AnalyticsChartPoint {
                label: Cell::Empty,
                key: None,
                values: vec![cell_of(kind, rest)?],
                is_others: true,
                compare: if compare_at.is_some() {
                    Some(cell_of(kind, rest_compare)?)
                } else {
                    None
                },
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
            compare: None,
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
                compare: None,
            })
        })
        .collect()
}

/// Trend points from the period columns of a view D: one for each column that has a heading, with
/// the value of the totals row.
fn trend_columns(
    columns: &[AnalyticsColumn],
    totals: &[Cell],
    series_key: &str,
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
            compare: None,
        });
    }
    if points.is_empty() {
        return Ok(Vec::new());
    }
    let mut by_month = false;
    let series = vec![AnalyticsChartSeries {
        key: series_key.to_owned(),
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

/// The ranked chart of one measure of a table: the 10 biggest rows and "others". For the partner
/// part of view M (the kharch each partner spent himself, measure `own`).
///
/// # Errors
/// `ReportsError::Money` if a sum does not fit.
pub(crate) fn ranked_by(
    columns: &[AnalyticsColumn],
    rows: &[AnalyticsRow],
    measure: &str,
) -> Result<Vec<AnalyticsChart>, ReportsError> {
    ranked(columns, rows, measure)
}

/// The charts of a view built from its whole table. An empty list when the view has no chart or
/// the table has no row.
///
/// # Errors
/// `ReportsError::Money` if a sum does not fit.
pub(crate) fn build(
    view: AnalyticsView,
    trip_view: Option<AnalyticsTripView>,
    measure: Option<AnalyticsMeasure>,
    columns: &[AnalyticsColumn],
    rows: &[AnalyticsRow],
    totals: &[Cell],
) -> Result<Vec<AnalyticsChart>, ReportsError> {
    match plan_of(view, trip_view, measure) {
        Plan::Ranked(measure) => ranked(columns, rows, measure),
        Plan::TrendRows(keys) => trend_rows(columns, rows, keys),
        Plan::TrendColumns(series_key) => trend_columns(columns, totals, series_key),
    }
}

/// One row of a ranked chart made from stored sums (views A, B and C with a measure): what the point
/// is called, what opens its lines, its figure, and the same figure in the compared period.
pub(crate) struct RankedEntry {
    /// A cell like the table's: an item, a seller, a buyer.
    pub(crate) label: Cell,
    /// Opens the lines behind the point.
    pub(crate) key: AnalyticsKey,
    /// The figure of the measure.
    pub(crate) value: Figure,
    /// The figure of the compared period; `None` when there is no comparison.
    pub(crate) compared: Option<Figure>,
}

/// The ranked chart of a measure: the 10 biggest entries, then "others" for the rest, from every
/// entry of the filter (checked sums). The entries are in the order of the table, so equal figures
/// keep it. No chart when there is no entry.
///
/// # Errors
/// `ReportsError::Money` or `ReportsError::Data` if a sum does not fit.
pub(crate) fn ranked_chart(
    measure: AnalyticsMeasure,
    mut entries: Vec<RankedEntry>,
) -> Result<Vec<AnalyticsChart>, ReportsError> {
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.value));
    let with_compare = entries.iter().any(|e| e.compared.is_some());
    let zero = Figure::zero(measure);
    let mut points = Vec::new();
    let (mut rest, mut rest_compared, mut rest_count) = (zero, zero, 0_i64);
    for (rank, entry) in entries.into_iter().enumerate() {
        if rank < TOP {
            points.push(AnalyticsChartPoint {
                label: entry.label,
                key: Some(entry.key),
                values: vec![entry.value.cell()],
                is_others: false,
                compare: entry.compared.map(Figure::cell),
            });
        } else {
            rest = rest.plus(entry.value)?;
            rest_compared = rest_compared.plus(entry.compared.unwrap_or(zero))?;
            rest_count = rest_count.checked_add(1).ok_or_else(overflow)?;
        }
    }
    if rest_count > 0 {
        points.push(AnalyticsChartPoint {
            label: Cell::Empty,
            key: None,
            values: vec![rest.cell()],
            is_others: true,
            compare: with_compare.then(|| rest_compared.cell()),
        });
    }
    Ok(vec![AnalyticsChart {
        kind: AnalyticsChartKind::Ranked,
        title: None,
        series: vec![AnalyticsChartSeries {
            key: measure.key().to_owned(),
            kind: measure.kind(),
        }],
        points,
        others_count: rest_count,
        by_month: false,
    }])
}

/// The trend chart of a measure (view G with a measure): one point for every period, in order, of
/// the one series. Days become months when there are more than 120 (as every trend chart).
///
/// # Errors
/// `ReportsError::Money` or `ReportsError::Data` if a sum does not fit.
pub(crate) fn trend_chart(
    measure: AnalyticsMeasure,
    periods: Vec<(Cell, AnalyticsKey, Figure)>,
) -> Result<Vec<AnalyticsChart>, ReportsError> {
    if periods.is_empty() {
        return Ok(Vec::new());
    }
    let series = vec![AnalyticsChartSeries {
        key: measure.key().to_owned(),
        kind: measure.kind(),
    }];
    let mut points: Vec<AnalyticsChartPoint> = periods
        .into_iter()
        .map(|(label, key, value)| AnalyticsChartPoint {
            label,
            key: Some(key),
            values: vec![value.cell()],
            is_others: false,
            compare: None,
        })
        .collect();
    let mut by_month = false;
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
    let urdu = sort.language == Some(mandi_core::auth::Language::Ur);
    let by = |a: &AnalyticsRow, b: &AnalyticsRow| -> Ordering {
        let empty = Cell::Empty;
        sort.direction.apply(compare(
            a.cells.get(at).unwrap_or(&empty),
            b.cells.get(at).unwrap_or(&empty),
            urdu,
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

#[cfg(test)]
mod tests {
    //! The chart of a measure (PRD FR-18.10, plan 84): the ten biggest and "others", and the days of a
    //! long trend turned into months, with a count as a count and money as money.

    use super::*;
    use mandi_core::dates::BusinessDate;

    fn entry(n: i64, value: Figure, compared: Option<Figure>) -> RankedEntry {
        RankedEntry {
            label: Cell::Number { value: n },
            key: AnalyticsKey {
                item_id: Some(n),
                ..AnalyticsKey::default()
            },
            value,
            compared,
        }
    }

    fn p(rupees: i64) -> Figure {
        Figure::Money(Paisa::from_rupees(rupees).unwrap())
    }

    #[test]
    fn a_ranked_chart_of_a_count_keeps_ten_points_and_adds_the_rest_as_others() {
        // 12, 11 ... 1 in any order: the ten biggest are 12 down to 3, the rest are 2 and 1.
        let entries: Vec<RankedEntry> =
            (1..=12).map(|n| entry(n, Figure::Count(n), None)).collect();
        let charts = ranked_chart(AnalyticsMeasure::Qty, entries).unwrap();
        assert_eq!(charts.len(), 1);
        let chart = &charts[0];
        assert_eq!(chart.kind, AnalyticsChartKind::Ranked);
        assert_eq!(chart.series[0].key, "qty");
        assert_eq!(chart.series[0].kind, ColumnKind::Number);
        assert_eq!(chart.points.len(), 11);
        let values: Vec<Cell> = chart.points.iter().map(|p| p.values[0].clone()).collect();
        assert_eq!(values[0], Cell::Number { value: 12 });
        assert_eq!(values[9], Cell::Number { value: 3 });
        assert_eq!(values[10], Cell::Number { value: 3 });
        assert!(chart.points[10].is_others && chart.points[10].key.is_none());
        assert_eq!(chart.others_count, 2);
        assert!(chart.points.iter().all(|p| p.compare.is_none()));
    }

    #[test]
    fn a_ranked_chart_of_money_carries_the_compared_figure_and_adds_it_up_for_others() {
        let entries: Vec<RankedEntry> =
            (1..=11).map(|n| entry(n, p(n * 100), Some(p(n)))).collect();
        let chart = &ranked_chart(AnalyticsMeasure::Gross, entries).unwrap()[0];
        assert_eq!(chart.series[0].kind, ColumnKind::Money);
        assert_eq!(chart.points.len(), 11);
        // The biggest is 1,100 against 11; the one left over is 100 against 1.
        assert_eq!(chart.points[0].values[0], p(1_100).cell());
        assert_eq!(chart.points[0].compare, Some(p(11).cell()));
        assert_eq!(chart.points[10].values[0], p(100).cell());
        assert_eq!(chart.points[10].compare, Some(p(1).cell()));
    }

    #[test]
    fn equal_figures_keep_the_order_of_the_table_and_no_entry_means_no_chart() {
        let entries = vec![
            entry(1, p(5), None),
            entry(2, p(9), None),
            entry(3, p(5), None),
        ];
        let chart = &ranked_chart(AnalyticsMeasure::Gross, entries).unwrap()[0];
        let order: Vec<Cell> = chart.points.iter().map(|p| p.label.clone()).collect();
        assert_eq!(
            order,
            [
                Cell::Number { value: 2 },
                Cell::Number { value: 1 },
                Cell::Number { value: 3 }
            ]
        );
        assert!(
            ranked_chart(AnalyticsMeasure::Qty, Vec::new())
                .unwrap()
                .is_empty()
        );
        assert!(
            trend_chart(AnalyticsMeasure::Qty, Vec::new())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_trend_of_more_than_120_days_is_turned_into_months_of_the_same_figures() {
        // 121 days from 01-01-26 end on 01-05-26: 31, 28, 31, 30 and 1 days.
        let mut day = BusinessDate::from_ymd(2026, 1, 1).unwrap();
        let mut periods = Vec::new();
        for _ in 0..121 {
            periods.push((
                Cell::Date { iso: day.to_iso() },
                AnalyticsKey::default(),
                Figure::Count(1),
            ));
            day = day.next_day().unwrap();
        }
        let chart = &trend_chart(AnalyticsMeasure::Qty, periods).unwrap()[0];
        assert!(chart.by_month);
        assert_eq!(chart.series[0].kind, ColumnKind::Number);
        let counts: Vec<Cell> = chart.points.iter().map(|p| p.values[0].clone()).collect();
        assert_eq!(
            counts,
            [31, 28, 31, 30, 1].map(|value| Cell::Number { value })
        );
        assert_eq!(
            chart.points[0].label,
            Cell::Month {
                year: 2026,
                month: 1
            }
        );
        // Up to 120 days stay days.
        let few: Vec<_> = (1..=3)
            .map(|d| {
                (
                    Cell::Date {
                        iso: format!("2026-01-0{d}"),
                    },
                    AnalyticsKey::default(),
                    p(d * 100),
                )
            })
            .collect();
        let chart = &trend_chart(AnalyticsMeasure::Gross, few).unwrap()[0];
        assert!(!chart.by_month);
        assert_eq!(chart.points.len(), 3);
        assert_eq!(chart.points[2].values[0], p(300).cell());
    }
}
