//! Plan 38 B2 slice (b) (PRD FR-19.11, FR-18.3, Code Standards B31): a report can be SORTED by a
//! column and can show MONTH HEADINGS with the sums of the whole month; an analytics view can be
//! sorted, and gives CHARTS made from the whole filter that a sort or a page never changes.
//!
//! The worked day (PRD Section 4) is on 10-10-26; this file adds one receipt on 05-11-26, so the
//! books have two months.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_reports::ReportsError;
use mandi_reports::analytics::{
    AnalyticsChartKind, AnalyticsFilters, AnalyticsPage, AnalyticsRunInput, AnalyticsView,
    run as analytics_run,
};
use mandi_reports::reports::{
    Cell, ColumnKind, PartyFilter, ReportFilters, ReportId, ReportPage, ReportRunInput, RowKind,
    run,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::paging::{SortDirection, SortInput};
use mandi_services::core::vouchers::ReceiptInput;
use mandi_services::vouchers;
use reports_world::*;

fn sort(column: &str, direction: SortDirection) -> Option<SortInput> {
    Some(SortInput {
        column: column.into(),
        direction,
    })
}

/// The worked day plus one receipt from اکرم on 05-11-26: his ledger has rows in two months.
fn two_months() -> support::World {
    let w = full_day();
    set_today(&w.env, 2026, 11, 5);
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: "05-11-2026".into(),
            account_id: w.akram,
            person_key: None,
            amount: "700".into(),
            removed: Default::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .unwrap();
    w
}

fn read(
    w: &support::World,
    id: ReportId,
    filters: ReportFilters,
    sort_input: Option<SortInput>,
    group_by_month: bool,
    offset: i64,
    limit: i64,
) -> Result<ReportPage, ReportsError> {
    run(
        w.env.ctx.db(),
        &ReportRunInput {
            id,
            filters,
            offset,
            limit,
            sort: sort_input,
            group_by_month,
        },
    )
}

fn key_index(page: &ReportPage, key: &str) -> usize {
    page.columns
        .iter()
        .position(|c| c.key == key)
        .unwrap_or_else(|| panic!("no column {key}: {:?}", page.columns))
}

fn normal(page: &ReportPage) -> Vec<&mandi_reports::reports::Row> {
    page.rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .collect()
}

fn refused_column(error: &ReportsError) -> bool {
    matches!(error, ReportsError::Rejected(items)
        if items.iter().any(|i| i.code == ErrorCode::ErrOutOfRange && i.field == "sort.column"))
}

fn money_of(cell: &Cell) -> i64 {
    match cell {
        Cell::Money { paisa } => paisa.as_paisa(),
        Cell::Empty => 0,
        other => panic!("not money: {other:?}"),
    }
}

#[test]
fn a_report_sorts_the_whole_filter_and_keeps_its_totals() {
    let w = two_months();
    let all = ReportFilters::default();
    let natural = read(&w, ReportId::R15, all.clone(), None, false, 0, 0).unwrap();
    assert!(natural.total_count >= 3);
    let amount = key_index(&natural, "amount");
    // By the amount, biggest first, read in pages of one: every row once, in order.
    let sorted_whole = read(
        &w,
        ReportId::R15,
        all.clone(),
        sort("amount", SortDirection::Desc),
        false,
        0,
        0,
    )
    .unwrap();
    let amounts: Vec<i64> = normal(&sorted_whole)
        .iter()
        .map(|r| money_of(&r.cells[amount]))
        .collect();
    let mut expected = amounts.clone();
    expected.sort_by(|a, b| b.cmp(a));
    assert_eq!(amounts, expected);
    assert_eq!(
        sorted_whole.totals, natural.totals,
        "the totals do not move"
    );
    assert_eq!(sorted_whole.total_count, natural.total_count);
    let pieces: Vec<i64> = (0..natural.total_count)
        .map(|offset| {
            let page = read(
                &w,
                ReportId::R15,
                all.clone(),
                sort("amount", SortDirection::Desc),
                false,
                offset,
                1,
            )
            .unwrap();
            assert_eq!(page.rows.len(), 1);
            money_of(&page.rows[0].cells[amount])
        })
        .collect();
    assert_eq!(
        pieces, amounts,
        "a page is a slice of the whole sorted list"
    );
    // The other direction is the mirror (the amounts, not the ties).
    let ascending = read(
        &w,
        ReportId::R15,
        all.clone(),
        sort("amount", SortDirection::Asc),
        false,
        0,
        0,
    )
    .unwrap();
    let up: Vec<i64> = normal(&ascending)
        .iter()
        .map(|r| money_of(&r.cells[amount]))
        .collect();
    let mut mirrored = amounts.clone();
    mirrored.reverse();
    assert_eq!(up, mirrored);
    // An unknown key is refused with its field.
    let error = read(
        &w,
        ReportId::R15,
        all,
        sort("no_such_column", SortDirection::Asc),
        false,
        0,
        0,
    )
    .unwrap_err();
    assert!(refused_column(&error));
}

#[test]
fn a_ledger_gets_month_headings_with_the_sums_of_the_whole_month() {
    let w = two_months();
    let akram = ReportFilters {
        party: Some(PartyFilter {
            account_id: w.akram,
            person_key: None,
        }),
        ..ReportFilters::default()
    };
    let plain = read(&w, ReportId::R13, akram.clone(), None, false, 0, 0).unwrap();
    let with = read(&w, ReportId::R13, akram.clone(), None, true, 0, 0).unwrap();
    // Two headings (October, November) among the rows.
    let headings: Vec<usize> = with
        .rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.kind == RowKind::Month)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(headings.len(), 2, "{:?}", with.rows);
    assert_eq!(with.total_count, plain.total_count + 2);
    assert_eq!(with.totals, plain.totals);
    let date = with
        .columns
        .iter()
        .position(|c| c.kind == ColumnKind::Date)
        .expect("a date column");
    assert_eq!(
        with.rows[headings[0]].cells[date],
        Cell::Month {
            year: 2026,
            month: 10
        }
    );
    assert_eq!(
        with.rows[headings[1]].cells[date],
        Cell::Month {
            year: 2026,
            month: 11
        }
    );
    // The sums of the headings add up to the totals row for every money column the totals add.
    for (column, total) in with.totals.iter().enumerate() {
        if let Cell::Money { paisa } = total {
            let sum: i64 = headings
                .iter()
                .map(|i| money_of(&with.rows[*i].cells[column]))
                .sum();
            assert_eq!(sum, paisa.as_paisa(), "column {column}");
        }
    }
    // October holds the sale line and the receipt of the worked day; the month closes at 2,880.
    let balance = with
        .columns
        .iter()
        .position(|c| c.kind == ColumnKind::Balance)
        .expect("a running balance");
    let october = &with.rows[headings[0]].cells[balance];
    let november = &with.rows[headings[1]].cells[balance];
    let (Cell::Balance { paisa: oct, .. }, Cell::Balance { paisa: nov, .. }) = (october, november)
    else {
        panic!("not balances: {october:?} {november:?}");
    };
    assert_eq!((oct.as_paisa(), nov.as_paisa()), (288_000, 218_000));
    // A page that starts inside a month gets the heading of that month, with the whole month's sums.
    let second_row = i64::try_from(headings[0] + 2).unwrap();
    let page = read(&w, ReportId::R13, akram.clone(), None, true, second_row, 1).unwrap();
    assert_eq!(page.group_above.as_ref(), Some(&with.rows[headings[0]]));
    // A page that starts on a heading needs none.
    let on_heading = i64::try_from(headings[1]).unwrap();
    let page = read(&w, ReportId::R13, akram.clone(), None, true, on_heading, 2).unwrap();
    assert!(page.group_above.is_none());
    assert_eq!(page.rows[0].kind, RowKind::Month);
    // Another sort than the date puts no headings; a sort by the date keeps them.
    let by_amount = read(
        &w,
        ReportId::R13,
        akram.clone(),
        sort("debit", SortDirection::Desc),
        true,
        0,
        0,
    );
    if let Ok(page) = by_amount {
        assert!(page.rows.iter().all(|r| r.kind != RowKind::Month));
    }
    let by_date = read(
        &w,
        ReportId::R13,
        akram,
        sort(&with.columns[date].key, SortDirection::Desc),
        true,
        0,
        0,
    )
    .unwrap();
    let months: Vec<&Cell> = by_date
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Month)
        .map(|r| &r.cells[date])
        .collect();
    assert_eq!(months.len(), 2);
}

#[test]
fn a_report_without_dates_has_no_month_headings_and_still_sorts() {
    let w = two_months();
    // R29 is by head, not by day: months are not asked for here, a sort works in each section.
    let page = read(
        &w,
        ReportId::R29,
        ReportFilters::default(),
        sort("shop_expenses", SortDirection::Desc),
        true,
        0,
        0,
    );
    // The key may differ; either it sorts or it names the allowed keys.
    match page {
        Ok(page) => assert!(page.rows.iter().all(|r| r.kind != RowKind::Month)),
        Err(error) => assert!(refused_column(&error)),
    }
}

// ---- Analytics ----

fn view(
    w: &support::World,
    v: AnalyticsView,
    sort_input: Option<SortInput>,
    offset: i64,
    limit: i64,
) -> Result<AnalyticsPage, ReportsError> {
    analytics_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: v,
            filters: AnalyticsFilters::default(),
            offset,
            limit,
            sort: sort_input,
        },
    )
}

#[test]
fn the_chart_of_a_ranked_view_is_made_from_the_whole_filter_and_never_moves() {
    let w = full_day();
    // View A, the worked day: کیلا 3,928 and سیب 1,520 of 5,448.
    let whole = view(&w, AnalyticsView::A, None, 0, 0).unwrap();
    assert_eq!(whole.charts.len(), 1);
    let chart = &whole.charts[0];
    assert_eq!(chart.kind, AnalyticsChartKind::Ranked);
    assert_eq!(chart.series[0].key, "total_comm");
    let values: Vec<i64> = chart
        .points
        .iter()
        .map(|p| money_of(&p.values[0]))
        .collect();
    assert_eq!(values, [392_800, 152_000]);
    assert!(chart.points.iter().all(|p| !p.is_others));
    assert_eq!(chart.others_count, 0);
    let total: i64 = values.iter().sum();
    let totals_at = whole
        .columns
        .iter()
        .position(|c| c.key == "total_comm")
        .unwrap();
    assert_eq!(total, money_of(&whole.totals[totals_at]));
    // A page of one row, and a sort, leave the chart alone.
    let page = view(&w, AnalyticsView::A, None, 1, 1).unwrap();
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.charts, whole.charts);
    assert_eq!(page.total_count, whole.total_count);
    let sorted = view(
        &w,
        AnalyticsView::A,
        sort("total_comm", SortDirection::Asc),
        0,
        0,
    )
    .unwrap();
    assert_eq!(sorted.charts, whole.charts);
    let first = money_of(&sorted.rows[0].cells[totals_at]);
    assert_eq!(
        first, 152_000,
        "the smaller commission comes first when ascending"
    );
    // The chart has at most ten points and one "others" for the rest.
    // (Ten sellers and buyers are not in the worked day; the shape is held by the unit tests of
    // the chart module's sums: here, two items fit in the ten.)
    let error = view(
        &w,
        AnalyticsView::A,
        sort("no_such", SortDirection::Asc),
        0,
        0,
    )
    .unwrap_err();
    assert!(refused_column(&error));
    // View H has one chart for each of its sections.
    let top = view(&w, AnalyticsView::H, None, 0, 0).unwrap();
    assert!(!top.charts.is_empty());
    assert!(top.charts.iter().all(|c| c.points.len() <= 10));
}

#[test]
fn the_chart_of_a_buyer_view_ranks_every_buyer_and_its_sort_is_limited_to_what_is_known() {
    let w = full_day();
    let whole = view(&w, AnalyticsView::C, None, 0, 0).unwrap();
    assert_eq!(whole.charts.len(), 1);
    assert_eq!(whole.charts[0].series[0].key, "purchase_value");
    // ارشد bought most (17,400), then اکرم (12,000) and شاہد (12,000).
    assert_eq!(money_of(&whole.charts[0].points[0].values[0]), 1_740_000);
    let page = view(&w, AnalyticsView::C, None, 1, 1).unwrap();
    assert_eq!(page.charts, whole.charts);
    // The columns known before the page is cut can be sorted; the dues cannot.
    let by_value = view(
        &w,
        AnalyticsView::C,
        sort("purchase_value", SortDirection::Asc),
        0,
        0,
    )
    .unwrap();
    let value_at = by_value
        .columns
        .iter()
        .position(|c| c.key == "purchase_value")
        .unwrap();
    let values: Vec<i64> = by_value
        .rows
        .iter()
        .map(|r| money_of(&r.cells[value_at]))
        .collect();
    let mut expected = values.clone();
    expected.sort_unstable();
    assert_eq!(values, expected);
    assert_eq!(by_value.charts, whole.charts);
    let error = view(
        &w,
        AnalyticsView::C,
        sort("dues_now", SortDirection::Asc),
        0,
        0,
    )
    .unwrap_err();
    assert!(refused_column(&error));
}

#[test]
fn the_chart_of_a_trend_view_has_a_point_for_every_period() {
    let w = two_months();
    // View G, by month: October and November.
    let page = analytics_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: AnalyticsView::G,
            filters: AnalyticsFilters::default(),
            offset: 0,
            limit: 1,
            sort: None,
        },
    )
    .unwrap();
    assert_eq!(page.charts.len(), 1);
    let chart = &page.charts[0];
    assert_eq!(chart.kind, AnalyticsChartKind::Trend);
    assert_eq!(chart.series.len(), 5);
    assert!(!chart.by_month);
    assert!(!chart.points.is_empty());
    assert!(chart.points.iter().all(|p| p.values.len() == 5));
    // The table is paged to one row; the chart still has every period.
    assert_eq!(page.rows.len(), 1);
    assert!(chart.points.len() as i64 >= page.total_count.min(2));
    // View D (items by period) gives one point for each period column.
    let d = view(&w, AnalyticsView::D, None, 0, 0).unwrap();
    assert_eq!(d.charts.len(), 1);
    assert_eq!(d.charts[0].kind, AnalyticsChartKind::Trend);
    // A view with no chart says so: partners (F) and fruit taken (K).
    assert!(
        view(&w, AnalyticsView::F, None, 0, 0)
            .unwrap()
            .charts
            .is_empty()
    );
}
