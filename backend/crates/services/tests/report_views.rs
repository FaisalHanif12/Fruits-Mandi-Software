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
    Cell, ColumnKind, PartyFilter, ReportFilters, ReportId, ReportPage, ReportPrintInput,
    ReportRunInput, RowKind, print_data, run,
};
use mandi_services::core::auth::Language;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::paging::{SortDirection, SortInput};
use mandi_services::core::vouchers::{ReceiptInput, VoucherCancelInput};
use mandi_services::vouchers;
use reports_world::*;

fn sort(column: &str, direction: SortDirection) -> Option<SortInput> {
    Some(SortInput {
        language: None,
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
    // Month headings are not counted: the offsets and the count mean the same with and without.
    assert_eq!(with.total_count, plain.total_count);
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
    // A page that starts inside a month gets the heading of that month, with the whole month's
    // sums, above its rows (a page after the first: the first month's heading is `group_above`).
    let data_rows_of_october = i64::try_from(headings[1] - headings[0] - 1).unwrap();
    let page = read(&w, ReportId::R13, akram.clone(), None, true, 1, 1).unwrap();
    assert_eq!(page.group_above.as_ref(), Some(&with.rows[headings[0]]));
    assert!(page.rows.iter().all(|r| r.kind != RowKind::Month));
    // The page that starts on the first row of November has the November heading above it, and a
    // page that holds the end of October and the start of November has the heading in its rows.
    let page = read(
        &w,
        ReportId::R13,
        akram.clone(),
        None,
        true,
        data_rows_of_october,
        2,
    )
    .unwrap();
    assert_eq!(page.group_above.as_ref(), Some(&with.rows[headings[1]]));
    let page = read(
        &w,
        ReportId::R13,
        akram.clone(),
        None,
        true,
        data_rows_of_october - 1,
        2,
    )
    .unwrap();
    assert_eq!(page.group_above.as_ref(), Some(&with.rows[headings[0]]));
    assert_eq!(
        page.rows
            .iter()
            .filter(|r| r.kind == RowKind::Month)
            .count(),
        1
    );
    assert_eq!(
        page.rows.iter().find(|r| r.kind == RowKind::Month),
        Some(&with.rows[headings[1]])
    );
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
    // Newest first, a month's heading still shows the balance at the END of the month, as in the
    // natural order: October closes at 2,880 (after the sale and the receipt of the same day),
    // November at 2,180. A review found it once showing the balance of the oldest row.
    let balances: Vec<i64> = by_date
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Month)
        .map(|r| match &r.cells[balance] {
            Cell::Balance { paisa, .. } => paisa.as_paisa(),
            other => panic!("not a balance: {other:?}"),
        })
        .collect();
    assert_eq!(balances, vec![218_000, 288_000]);
}

#[test]
fn a_ledger_or_day_book_sorted_by_debit_or_credit_is_a_sql_page_in_the_order_of_a_full_sort() {
    let w = two_months();
    let akram = ReportFilters {
        party: Some(PartyFilter {
            account_id: w.akram,
            person_key: None,
        }),
        ..ReportFilters::default()
    };
    let day = ReportFilters {
        from: "10-10-2026".into(),
        to: "10-10-2026".into(),
        ..ReportFilters::default()
    };
    for (id, filters) in [(ReportId::R13, akram), (ReportId::R11, day)] {
        let natural = read(&w, id, filters.clone(), None, false, 0, 0).unwrap();
        let date = key_index(&natural, "date");
        let debit = key_index(&natural, "debit");
        let credit = key_index(&natural, "credit");
        let balance = natural
            .columns
            .iter()
            .position(|c| c.kind == ColumnKind::Balance);
        let shown = |row: &mandi_reports::reports::Row| {
            (
                row.cells[date].clone(),
                row.cells[debit].clone(),
                row.cells[credit].clone(),
            )
        };
        for (column, at) in [("debit", debit), ("credit", credit)] {
            for direction in [SortDirection::Asc, SortDirection::Desc] {
                // The expected order: all rows, sorted by the amount; the same amounts keep the
                // order of the days (a stable sort of the natural order).
                let mut expected: Vec<&mandi_reports::reports::Row> = natural.rows.iter().collect();
                expected.sort_by(|a, b| {
                    direction.apply(money_of(&a.cells[at]).cmp(&money_of(&b.cells[at])))
                });
                let expected: Vec<_> = expected.into_iter().map(shown).collect();
                let page = read(
                    &w,
                    id,
                    filters.clone(),
                    sort(column, direction),
                    false,
                    0,
                    0,
                )
                .unwrap();
                let got: Vec<_> = page.rows.iter().map(shown).collect();
                assert_eq!(got, expected, "{id:?} {column} {direction:?}");
                assert_eq!(page.total_count, natural.total_count);
                assert_eq!(page.totals, natural.totals, "the totals do not move");
                // A running balance means something only in the order of the days.
                if let Some(balance) = balance {
                    assert!(page.rows.iter().all(|r| r.cells[balance] == Cell::Empty));
                }
                // The pages cut the same order, one row at a time.
                for (offset, want) in expected.iter().enumerate() {
                    let one = read(
                        &w,
                        id,
                        filters.clone(),
                        sort(column, direction),
                        false,
                        i64::try_from(offset).unwrap(),
                        1,
                    )
                    .unwrap();
                    assert_eq!(one.rows.len(), 1);
                    assert_eq!(&shown(&one.rows[0]), want, "{id:?} {column} row {offset}");
                }
            }
        }
    }
}

#[test]
fn a_date_sort_of_a_ledger_day_book_or_cash_report_is_the_natural_order_or_its_reverse() {
    let w = two_months();
    let akram = ReportFilters {
        party: Some(PartyFilter {
            account_id: w.akram,
            person_key: None,
        }),
        ..ReportFilters::default()
    };
    for (id, filters) in [
        (ReportId::R13, akram),
        (ReportId::R11, ReportFilters::default()),
        (ReportId::R12, ReportFilters::default()),
    ] {
        let natural = read(&w, id, filters.clone(), None, false, 0, 0).unwrap();
        assert!(natural.rows.len() >= 3, "{id:?}");
        for direction in [SortDirection::Asc, SortDirection::Desc] {
            // Oldest first is the natural order; newest first is exactly its reverse, every cell
            // the same, the running balance included (it is the balance in the order of the days).
            let mut expected: Vec<_> = natural.rows.clone();
            if direction == SortDirection::Desc {
                expected.reverse();
            }
            let page = read(
                &w,
                id,
                filters.clone(),
                sort("date", direction),
                false,
                0,
                0,
            )
            .unwrap();
            assert_eq!(page.rows, expected, "{id:?} {direction:?}");
            assert_eq!(page.total_count, natural.total_count);
            assert_eq!(page.totals, natural.totals);
            assert_eq!(page.summary, natural.summary);
            // The pages cut that order, one row at a time, two at a time, and past the end.
            for (offset, want) in expected.iter().enumerate() {
                let one = read(
                    &w,
                    id,
                    filters.clone(),
                    sort("date", direction),
                    false,
                    i64::try_from(offset).unwrap(),
                    1,
                )
                .unwrap();
                assert_eq!(
                    one.rows.as_slice(),
                    std::slice::from_ref(want),
                    "{id:?} {direction:?} row {offset}"
                );
            }
            let two = read(
                &w,
                id,
                filters.clone(),
                sort("date", direction),
                false,
                1,
                2,
            )
            .unwrap();
            assert_eq!(two.rows, expected[1..3], "{id:?} {direction:?}");
            let last = i64::try_from(expected.len()).unwrap();
            let past = read(
                &w,
                id,
                filters.clone(),
                sort("date", direction),
                false,
                last,
                5,
            )
            .unwrap();
            assert!(past.rows.is_empty());
            let tail = read(
                &w,
                id,
                filters.clone(),
                sort("date", direction),
                false,
                last - 1,
                5,
            )
            .unwrap();
            assert_eq!(
                tail.rows,
                expected[expected.len() - 1..],
                "{id:?} {direction:?}"
            );
        }
        // Month headings with the newest first: the same headings (the balance at the end of
        // each month), met in the other order.
        let months = read(&w, id, filters.clone(), None, true, 0, 0).unwrap();
        let newest = read(
            &w,
            id,
            filters.clone(),
            sort("date", SortDirection::Desc),
            true,
            0,
            0,
        )
        .unwrap();
        let headings = |page: &ReportPage| -> Vec<_> {
            page.rows
                .iter()
                .filter(|r| r.kind == RowKind::Month)
                .cloned()
                .collect()
        };
        let mut reversed = headings(&newest);
        reversed.reverse();
        assert_eq!(reversed, headings(&months), "{id:?}");
        assert!(headings(&months).len() >= 2, "{id:?}");
    }
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
    // A point is called by the item, seller or buyer, with both scripts like the table, never by
    // its rank ("1", "2").
    let name_at = top
        .columns
        .iter()
        .position(|c| c.key == "name")
        .expect("a name column");
    let mut named = 0;
    for chart in &top.charts {
        for point in chart.points.iter().filter(|p| !p.is_others) {
            assert!(
                matches!(point.label, Cell::Names { .. } | Cell::Party { .. }),
                "{:?}",
                point.label
            );
            let row = top
                .rows
                .iter()
                .find(|r| r.kind == RowKind::Normal && r.key == point.key)
                .expect("the row of the point");
            assert_eq!(point.label, row.cells[name_at]);
            named += 1;
        }
    }
    // The worked day: two items, two sellers and three buyers.
    assert_eq!(named, 7);
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

#[test]
fn a_cancelled_voucher_adds_nothing_to_its_month_heading() {
    let w = two_months();
    // A second receipt of 5,000 on 05-11-26, cancelled: the list shows it, the money leaves it out.
    let saved = vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: "05-11-2026".into(),
            account_id: w.akram,
            person_key: None,
            amount: "5,000".into(),
            removed: Default::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .unwrap();
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: saved.voucher_id,
            version: row.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let page = read(
        &w,
        ReportId::R23,
        ReportFilters::default(),
        None,
        true,
        0,
        0,
    )
    .unwrap();
    let amount = key_index(&page, "amount");
    let sum_of_headings: i64 = page
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Month)
        .map(|r| money_of(&r.cells[amount]))
        .sum();
    assert_eq!(sum_of_headings, money_of(&page.totals[amount]));
    // The cancelled row is in the list (it is a row of the report), only its money is out.
    let november = page
        .rows
        .iter()
        .find(|r| r.kind == RowKind::Month && matches!(r.cells[0], Cell::Month { month: 11, .. }))
        .expect("a November heading");
    assert_eq!(money_of(&november.cells[amount]), 70_000);
}

#[test]
fn the_print_has_the_rows_of_the_screen_in_the_same_order() {
    let w = two_months();
    let akram = ReportFilters {
        party: Some(PartyFilter {
            account_id: w.akram,
            person_key: None,
        }),
        ..ReportFilters::default()
    };
    let print =
        |filters: ReportFilters, id: ReportId, sort_input: Option<SortInput>, months: bool| {
            print_data(
                w.env.ctx.db(),
                &ReportPrintInput {
                    id,
                    filters,
                    language: Language::En,
                    paper: None,
                    sort: sort_input,
                    group_by_month: months,
                },
            )
            .unwrap()
        };
    // Month headings: the print has the same rows (headings included) as the screen with no limit.
    let screen = read(&w, ReportId::R13, akram.clone(), None, true, 0, 0).unwrap();
    let printed = print(akram.clone(), ReportId::R13, None, true);
    let kinds = |rows: &[mandi_reports::reports::PrintReportRow]| -> Vec<RowKind> {
        rows.iter().map(|r| r.kind).collect()
    };
    assert_eq!(
        kinds(&printed.rows),
        screen.rows.iter().map(|r| r.kind).collect::<Vec<_>>()
    );
    // Newest first: the print and the Excel rows are the exact reverse of the natural order, as the
    // screen is (rows of one day newest line first), and a month heading keeps the balance at the
    // end of its month.
    let by_date = sort("date", SortDirection::Desc);
    for (id, filters) in [
        (ReportId::R13, akram.clone()),
        (ReportId::R11, ReportFilters::default()),
        (ReportId::R12, ReportFilters::default()),
    ] {
        for months in [false, true] {
            let natural = print(filters.clone(), id, None, months);
            let newest = print(filters.clone(), id, by_date.clone(), months);
            let lines = |p: &mandi_reports::reports::ReportPrintData| -> Vec<_> {
                p.rows
                    .iter()
                    .filter(|r| r.kind != RowKind::Month)
                    .cloned()
                    .collect()
            };
            let headings = |p: &mandi_reports::reports::ReportPrintData| -> Vec<_> {
                p.rows
                    .iter()
                    .filter(|r| r.kind == RowKind::Month)
                    .cloned()
                    .collect()
            };
            let mut reversed = lines(&natural);
            reversed.reverse();
            assert_eq!(lines(&newest), reversed, "{id:?} months {months}");
            let mut reversed = headings(&newest);
            reversed.reverse();
            assert_eq!(reversed, headings(&natural), "{id:?} months {months}");
            assert_eq!(headings(&natural).is_empty(), !months, "{id:?}");
        }
    }
    // A sort: the print rows come in the screen's order (the amounts of R15, biggest first).
    let by_amount = sort("amount", SortDirection::Desc);
    let screen = read(
        &w,
        ReportId::R15,
        ReportFilters::default(),
        by_amount.clone(),
        false,
        0,
        0,
    )
    .unwrap();
    let printed = print(ReportFilters::default(), ReportId::R15, by_amount, false);
    assert_eq!(printed.rows.len(), screen.rows.len());
    let natural = print(ReportFilters::default(), ReportId::R15, None, false);
    assert_ne!(
        format!("{:?}", printed.rows),
        format!("{:?}", natural.rows),
        "the sorted print is not the natural order"
    );
    // The same for an analytics view.
    let a_screen = view(
        &w,
        AnalyticsView::A,
        sort("total_comm", SortDirection::Asc),
        0,
        0,
    )
    .unwrap();
    let a_print = mandi_reports::analytics::print_data(
        w.env.ctx.db(),
        &mandi_reports::analytics::AnalyticsPrintInput {
            view: AnalyticsView::A,
            filters: AnalyticsFilters::default(),
            language: Language::En,
            paper: None,
            sort: sort("total_comm", SortDirection::Asc),
        },
    )
    .unwrap();
    assert_eq!(a_print.rows.len(), a_screen.rows.len());
    let a_natural = mandi_reports::analytics::print_data(
        w.env.ctx.db(),
        &mandi_reports::analytics::AnalyticsPrintInput {
            view: AnalyticsView::A,
            filters: AnalyticsFilters::default(),
            language: Language::En,
            paper: None,
            sort: None,
        },
    )
    .unwrap();
    assert_ne!(
        format!("{:?}", a_print.rows),
        format!("{:?}", a_natural.rows),
        "the sorted analytics print is not the natural order"
    );
}
