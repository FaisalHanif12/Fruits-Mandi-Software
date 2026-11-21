//! Plan 84 (PRD FR-18.10): the "ناپ" measure of the analytics views. Sales, the buyers' bills, the
//! quantity or one commission decide the series of a chart, the figure in each cell of the matrices
//! D, E and L with their totals, and the order of view H. The tables of A, B, C and G keep every
//! column whatever the measure is.
//!
//! The numbers are the PRD worked day (Section 4, October 2026, Events 1 to 5):
//!
//! | | خام بکری | خریدار کی رقم | نگ | کل کمیشن | بیوپاری | گاہک | لاگا |
//! |---|---|---|---|---|---|---|---|
//! | کیلا | 29,400 | 31,564 | 50 | 3,928 | 1,764 | 1,764 | 400 |
//! | سیب | 12,000 | 12,800 | 10 | 1,520 | 720 | 720 | 80 |
//! | total | 41,400 | 44,364 | 60 | 5,448 | 2,484 | 2,484 | 480 |
//!
//! The buyer total is the value plus the buyer commission plus لاگا (C5): 29,400 + 1,764 + 400 =
//! 31,564, and 44,364 is the day's "billed" of R34.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::analytics::{
    AnalyticsChart, AnalyticsChartKind, AnalyticsCompare, AnalyticsFilterKind, AnalyticsFilters,
    AnalyticsGoods, AnalyticsGroupBy, AnalyticsMeasure, AnalyticsPage, AnalyticsPrintInput,
    AnalyticsRunInput, AnalyticsSide, AnalyticsView, catalog, print_data, run, screen_catalog,
};
use mandi_reports::reports::{Cell, ColumnKind, PartyFilter, RowKind};
use mandi_services::core::auth::Language;
use mandi_services::core::error::ErrorCode;
use reports_world::*;

use AnalyticsMeasure::{BuyerComm, BuyerTotal, Gross, Laga, Qty, SellerComm, TotalComm};

const ALL_MEASURES: [AnalyticsMeasure; 7] = [
    Gross, BuyerTotal, Qty, TotalComm, SellerComm, BuyerComm, Laga,
];

fn with(measure: Option<AnalyticsMeasure>) -> AnalyticsFilters {
    AnalyticsFilters {
        measure,
        ..AnalyticsFilters::default()
    }
}

fn page(w: &support::World, view: AnalyticsView, filters: AnalyticsFilters) -> AnalyticsPage {
    run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view,
            filters,
            offset: 0,
            limit: 0,
            sort: None,
        },
    )
    .unwrap()
}

fn refused(
    w: &support::World,
    view: AnalyticsView,
    filters: AnalyticsFilters,
) -> Vec<(ErrorCode, String)> {
    match run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view,
            filters,
            offset: 0,
            limit: 0,
            sort: None,
        },
    ) {
        Err(mandi_reports::ReportsError::Rejected(items)) => {
            items.into_iter().map(|i| (i.code, i.field)).collect()
        }
        other => panic!("expected a rejection, got {other:?}"),
    }
}

/// A money cell in rupees, a count as it is, an empty cell as 0.
fn figure(cell: &Cell) -> i64 {
    match cell {
        Cell::Money { paisa } => paisa.as_paisa() / 100,
        Cell::Number { value } => *value,
        Cell::Empty => 0,
        other => panic!("not a figure: {other:?}"),
    }
}

fn is_number(cell: &Cell) -> bool {
    matches!(cell, Cell::Number { .. } | Cell::Empty)
}

fn is_money(cell: &Cell) -> bool {
    matches!(cell, Cell::Money { .. } | Cell::Empty)
}

fn name(cell: &Cell) -> String {
    match cell {
        Cell::Names { names } => names.en.clone(),
        Cell::Party { party } => party.name_en.clone(),
        Cell::Empty => String::new(),
        other => panic!("not a name: {other:?}"),
    }
}

fn col(page: &AnalyticsPage, key: &str) -> usize {
    page.columns
        .iter()
        .position(|c| c.key == key)
        .unwrap_or_else(|| {
            let keys: Vec<&str> = page.columns.iter().map(|c| c.key.as_str()).collect();
            panic!("no column {key} in {keys:?}")
        })
}

fn data_rows(page: &AnalyticsPage) -> Vec<&mandi_reports::analytics::AnalyticsRow> {
    page.rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .collect()
}

/// The figures of the chart points, in order.
fn points(chart: &AnalyticsChart) -> Vec<i64> {
    chart.points.iter().map(|p| figure(&p.values[0])).collect()
}

fn labels(chart: &AnalyticsChart) -> Vec<String> {
    chart.points.iter().map(|p| name(&p.label)).collect()
}

/// What each measure gives for کیلا, سیب and the total (the table above).
fn expected(measure: AnalyticsMeasure) -> [i64; 3] {
    match measure {
        Gross => [29_400, 12_000, 41_400],
        BuyerTotal => [31_564, 12_800, 44_364],
        Qty => [50, 10, 60],
        TotalComm => [3_928, 1_520, 5_448],
        SellerComm => [1_764, 720, 2_484],
        BuyerComm => [1_764, 720, 2_484],
        Laga => [400, 80, 480],
    }
}

// ---- Matrices D, E and L -------------------------------------------------------------------

#[test]
fn view_d_counts_the_measure_in_every_cell_the_row_total_and_the_totals_row() {
    let w = full_day();
    for measure in ALL_MEASURES {
        let p = page(&w, AnalyticsView::D, with(Some(measure)));
        let [kela, seb, total] = expected(measure);
        // The row total column is named by the measure, and the period column is of its kind.
        let at = col(&p, measure.key());
        assert_eq!(
            p.columns.len(),
            3,
            "item, one month, the total: {measure:?}"
        );
        assert_eq!(p.columns[at].kind, measure.kind(), "{measure:?}");
        assert_eq!(p.columns[1].kind, measure.kind(), "{measure:?}");
        // The heading of the row total: the old words for the total commission, its own for the rest.
        let heading = if measure == TotalComm {
            "analytics-col-row-total".to_owned()
        } else {
            format!(
                "analytics-col-row-total-{}",
                measure.key().replace('_', "-")
            )
        };
        assert_eq!(p.columns[at].title_id, heading, "{measure:?}");
        let rows = data_rows(&p);
        assert_eq!(rows.len(), 2, "{measure:?}");
        assert_eq!(name(&rows[0].cells[0]), "Banana", "{measure:?}");
        assert_eq!(figure(&rows[0].cells[1]), kela, "{measure:?} کیلا, October");
        assert_eq!(figure(&rows[0].cells[at]), kela, "{measure:?} کیلا, total");
        assert_eq!(name(&rows[1].cells[0]), "Apple", "{measure:?}");
        assert_eq!(figure(&rows[1].cells[1]), seb, "{measure:?} سیب, October");
        assert_eq!(figure(&p.totals[1]), total, "{measure:?} column total");
        assert_eq!(figure(&p.totals[at]), total, "{measure:?} grand total");
        assert_eq!(p.filters.measure, Some(measure));
        // A count is a number cell, money is a money cell.
        for cell in [&rows[0].cells[1], &rows[0].cells[at], &p.totals[at]] {
            assert!(
                if measure == Qty {
                    is_number(cell)
                } else {
                    is_money(cell)
                },
                "{measure:?} {cell:?}"
            );
        }
    }
}

#[test]
fn view_d_without_a_measure_is_the_total_commission_as_before() {
    let w = full_day();
    let classic = page(&w, AnalyticsView::D, with(None));
    let explicit = page(&w, AnalyticsView::D, with(Some(TotalComm)));
    assert_eq!(classic.columns, explicit.columns);
    assert_eq!(classic.rows, explicit.rows);
    assert_eq!(classic.totals, explicit.totals);
    assert_eq!(classic.charts, explicit.charts);
    assert_eq!(classic.columns[2].key, "total_comm");
    assert_eq!(classic.filters.measure, None);
    assert_eq!(figure(&classic.totals[2]), 5_448);
}

#[test]
fn view_e_counts_the_measure_for_each_seller_and_item() {
    let w = full_day();
    for measure in ALL_MEASURES {
        let p = page(&w, AnalyticsView::E, with(Some(measure)));
        let [kela, seb, total] = expected(measure);
        let at = col(&p, measure.key());
        let rows = data_rows(&p);
        assert_eq!(rows.len(), 2, "{measure:?}");
        // رحیم sold کیلا, کریم sold سیب; the item columns are in the order of the item codes.
        assert_eq!(name(&rows[0].cells[0]), "Rahim", "{measure:?}");
        assert_eq!(name(&rows[1].cells[0]), "Karim", "{measure:?}");
        let banana = p
            .columns
            .iter()
            .position(|c| matches!(&c.heading, Some(mandi_reports::analytics::AnalyticsHeading::Item { names }) if names.en == "Banana"))
            .unwrap();
        let apple = p
            .columns
            .iter()
            .position(|c| matches!(&c.heading, Some(mandi_reports::analytics::AnalyticsHeading::Item { names }) if names.en == "Apple"))
            .unwrap();
        assert_eq!(figure(&rows[0].cells[banana]), kela, "{measure:?}");
        assert_eq!(figure(&rows[0].cells[apple]), 0, "{measure:?}");
        assert_eq!(figure(&rows[1].cells[apple]), seb, "{measure:?}");
        assert_eq!(figure(&rows[0].cells[at]), kela, "{measure:?} row total");
        assert_eq!(figure(&p.totals[banana]), kela, "{measure:?} column total");
        assert_eq!(figure(&p.totals[apple]), seb, "{measure:?} column total");
        assert_eq!(figure(&p.totals[at]), total, "{measure:?} grand total");
        assert_eq!(p.columns[at].kind, measure.kind(), "{measure:?}");
        assert_eq!(p.columns[banana].kind, measure.kind(), "{measure:?}");
    }
}

#[test]
fn view_l_counts_the_measure_for_each_buyer_and_item() {
    let w = full_day();
    // By خام بکری: ارشد × کیلا 17,400, اکرم × کیلا 12,000, شاہد × سیب 12,000; totals 29,400, 12,000,
    // 41,400. (Gross is the seller amounts of the lines each buyer bought.)
    let p = page(&w, AnalyticsView::L, with(Some(Gross)));
    let rows = data_rows(&p);
    let at = col(&p, "gross");
    let by_buyer: Vec<(String, i64)> = rows
        .iter()
        .map(|r| (name(&r.cells[0]), figure(&r.cells[at])))
        .collect();
    assert_eq!(
        by_buyer,
        [
            ("Arshad Multani".to_owned(), 17_400),
            ("Akram Toll Tax".to_owned(), 12_000),
            ("Shahid".to_owned(), 12_000),
        ]
    );
    assert_eq!(figure(&p.totals[at]), 41_400);
    // By the buyer's bill: 17,400 + 1,044 + 240 = 18,684 for ارشد (R5 of the day).
    let p = page(&w, AnalyticsView::L, with(Some(BuyerTotal)));
    let at = col(&p, "buyer_total");
    let rows = data_rows(&p);
    assert_eq!(
        rows.iter()
            .map(|r| figure(&r.cells[at]))
            .collect::<Vec<_>>(),
        [18_684, 12_880, 12_800]
    );
    assert_eq!(figure(&p.totals[at]), 44_364);
    // By نگ: a count, never rupees.
    let p = page(&w, AnalyticsView::L, with(Some(Qty)));
    let at = col(&p, "qty");
    assert_eq!(p.columns[at].kind, ColumnKind::Number);
    assert!(p.columns[1..].iter().all(|c| c.kind == ColumnKind::Number));
    let rows = data_rows(&p);
    assert_eq!(
        rows.iter()
            .map(|r| figure(&r.cells[at]))
            .collect::<Vec<_>>(),
        [30, 20, 10]
    );
    assert_eq!(figure(&p.totals[at]), 60);
    // The classic view is each buyer's own commission: his commission plus لاگا (2,964 in all).
    let classic = page(&w, AnalyticsView::L, with(None));
    let at = col(&classic, "total_comm");
    assert_eq!(figure(&classic.totals[at]), 2_964);
    // And the explicit total commission is the same page.
    assert_eq!(
        page(&w, AnalyticsView::L, with(Some(TotalComm))).rows,
        classic.rows
    );
    // The seller commission on the goods a buyer bought: 1,764 + 720 = 2,484 over all buyers.
    let p = page(&w, AnalyticsView::L, with(Some(SellerComm)));
    assert_eq!(figure(&p.totals[col(&p, "seller_comm")]), 2_484);
}

// ---- View H, the rankings ------------------------------------------------------------------

fn section_rows(p: &AnalyticsPage, at: usize) -> Vec<Vec<(String, i64)>> {
    let mut sections: Vec<Vec<(String, i64)>> = Vec::new();
    for row in &p.rows {
        match row.kind {
            RowKind::Normal => sections
                .last_mut()
                .expect("a heading row comes first")
                .push((name(&row.cells[1]), figure(&row.cells[at]))),
            _ => sections.push(Vec::new()),
        }
    }
    sections
}

#[test]
fn view_h_ranks_by_the_measure() {
    let w = full_day();
    let p = page(&w, AnalyticsView::H, with(Some(Gross)));
    let keys: Vec<&str> = p.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(keys, ["rank", "name", "qty", "gross", "share"]);
    let sections = section_rows(&p, col(&p, "gross"));
    assert_eq!(
        sections,
        [
            vec![("Banana".to_owned(), 29_400), ("Apple".to_owned(), 12_000)],
            vec![("Rahim".to_owned(), 29_400), ("Karim".to_owned(), 12_000)],
            vec![
                ("Arshad Multani".to_owned(), 17_400),
                ("Akram Toll Tax".to_owned(), 12_000),
                ("Shahid".to_owned(), 12_000),
            ],
        ]
    );
    assert_eq!(figure(&p.totals[col(&p, "gross")]), 41_400);
    // The share is of ALL the measure of the filter: 29,400 of 41,400 is 71.01%.
    let rows = data_rows(&p);
    assert_eq!(
        rows[0].cells[col(&p, "share")],
        Cell::Percent { hundredths: 7_101 }
    );
}

#[test]
fn view_h_by_quantity_shows_the_quantity_once_and_by_total_commission_is_the_classic_view() {
    let w = full_day();
    let p = page(&w, AnalyticsView::H, with(Some(Qty)));
    let keys: Vec<&str> = p.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(keys, ["rank", "name", "qty", "share"]);
    let sections = section_rows(&p, col(&p, "qty"));
    assert_eq!(
        sections[0],
        [("Banana".to_owned(), 50), ("Apple".to_owned(), 10)]
    );
    assert_eq!(figure(&p.totals[col(&p, "qty")]), 60);
    // 50 of 60 is 83.33%.
    assert_eq!(
        data_rows(&p)[0].cells[col(&p, "share")],
        Cell::Percent { hundredths: 8_333 }
    );
    // No measure and the total commission are the same page, the PRD 18.4 ranking.
    let classic = page(&w, AnalyticsView::H, with(None));
    let explicit = page(&w, AnalyticsView::H, with(Some(TotalComm)));
    assert_eq!(classic.columns, explicit.columns);
    assert_eq!(classic.rows, explicit.rows);
    assert_eq!(classic.totals, explicit.totals);
    let sections = section_rows(&classic, col(&classic, "total_comm"));
    assert_eq!(
        sections[2],
        [
            ("Arshad Multani".to_owned(), 2_328),
            ("Akram Toll Tax".to_owned(), 1_600),
            ("Shahid".to_owned(), 1_520),
        ]
    );
    // By the buyers' bills the order changes with the figure: ارشد, اکرم, شاہد.
    let p = page(&w, AnalyticsView::H, with(Some(BuyerTotal)));
    let sections = section_rows(&p, col(&p, "buyer_total"));
    assert_eq!(
        sections[2],
        [
            ("Arshad Multani".to_owned(), 18_684),
            ("Akram Toll Tax".to_owned(), 12_880),
            ("Shahid".to_owned(), 12_800),
        ]
    );
}

// ---- The charts ----------------------------------------------------------------------------

fn ranked_of(p: &AnalyticsPage) -> &AnalyticsChart {
    assert_eq!(p.charts.len(), 1, "one chart");
    &p.charts[0]
}

#[test]
fn the_chart_of_view_a_follows_the_measure_and_its_table_does_not_change() {
    let w = full_day();
    let classic = page(&w, AnalyticsView::A, with(None));
    assert_eq!(ranked_of(&classic).series[0].key, "total_comm");
    for measure in ALL_MEASURES {
        let p = page(&w, AnalyticsView::A, with(Some(measure)));
        let chart = ranked_of(&p);
        let [kela, seb, _] = expected(measure);
        assert_eq!(chart.kind, AnalyticsChartKind::Ranked);
        assert_eq!(chart.series.len(), 1);
        assert_eq!(chart.series[0].key, measure.key(), "{measure:?}");
        assert_eq!(chart.series[0].kind, measure.kind(), "{measure:?}");
        assert_eq!(points(chart), [kela, seb], "{measure:?}");
        assert_eq!(labels(chart), ["Banana", "Apple"], "{measure:?}");
        // The table is the classic one whatever the measure is.
        assert_eq!(p.columns, classic.columns, "{measure:?}");
        assert_eq!(p.rows, classic.rows, "{measure:?}");
        assert_eq!(p.totals, classic.totals, "{measure:?}");
        // A point opens the lines of its item, as the row does.
        assert_eq!(chart.points[0].key, classic.rows[0].key);
    }
}

#[test]
fn the_chart_of_view_b_ranks_the_sellers_by_the_measure_and_its_table_does_not_change() {
    let w = full_day();
    let classic = page(&w, AnalyticsView::B, with(None));
    for measure in ALL_MEASURES {
        let p = page(&w, AnalyticsView::B, with(Some(measure)));
        let chart = ranked_of(&p);
        let [kela, seb, _] = expected(measure);
        assert_eq!(chart.series[0].key, measure.key(), "{measure:?}");
        // رحیم sold the کیلا, کریم the سیب.
        assert_eq!(points(chart), [kela, seb], "{measure:?}");
        assert_eq!(labels(chart), ["Rahim", "Karim"], "{measure:?}");
        assert_eq!(p.columns, classic.columns, "{measure:?}");
        assert_eq!(p.rows, classic.rows, "{measure:?}");
        assert_eq!(p.totals, classic.totals, "{measure:?}");
    }
}

#[test]
fn the_chart_of_view_c_ranks_the_buyers_by_the_measure_and_its_table_does_not_change() {
    let w = full_day();
    let classic = page(&w, AnalyticsView::C, with(None));
    // Without a measure the chart is of the purchase value, as it always was.
    assert_eq!(ranked_of(&classic).series[0].key, "purchase_value");
    let p = page(&w, AnalyticsView::C, with(Some(Gross)));
    let chart = ranked_of(&p);
    assert_eq!(chart.series[0].key, "gross");
    assert_eq!(points(chart), [17_400, 12_000, 12_000]);
    assert_eq!(
        labels(chart),
        ["Arshad Multani", "Akram Toll Tax", "Shahid"]
    );
    // The buyers' bills: ارشد 18,684, اکرم 12,880, شاہد 12,800.
    let p2 = page(&w, AnalyticsView::C, with(Some(BuyerTotal)));
    assert_eq!(points(ranked_of(&p2)), [18_684, 12_880, 12_800]);
    // The quantity is a count.
    let p3 = page(&w, AnalyticsView::C, with(Some(Qty)));
    assert_eq!(ranked_of(&p3).series[0].kind, ColumnKind::Number);
    assert_eq!(points(ranked_of(&p3)), [30, 20, 10]);
    for measure in ALL_MEASURES {
        let p = page(&w, AnalyticsView::C, with(Some(measure)));
        assert_eq!(p.columns, classic.columns, "{measure:?}");
        assert_eq!(p.rows, classic.rows, "{measure:?}");
        assert_eq!(p.totals, classic.totals, "{measure:?}");
    }
}

#[test]
fn the_chart_of_view_d_has_a_point_for_each_period_of_the_measure() {
    let w = full_day();
    for measure in ALL_MEASURES {
        let p = page(&w, AnalyticsView::D, with(Some(measure)));
        let chart = ranked_of(&p);
        assert_eq!(chart.kind, AnalyticsChartKind::Trend);
        assert_eq!(chart.series[0].key, measure.key(), "{measure:?}");
        assert_eq!(chart.series[0].kind, measure.kind(), "{measure:?}");
        // One month, October, with the whole month's total.
        assert_eq!(points(chart), [expected(measure)[2]], "{measure:?}");
    }
}

#[test]
fn the_charts_of_e_h_and_l_follow_the_measure() {
    let w = full_day();
    let e = page(&w, AnalyticsView::E, with(Some(Gross)));
    assert_eq!(ranked_of(&e).series[0].key, "gross");
    assert_eq!(points(ranked_of(&e)), [29_400, 12_000]);
    let l = page(&w, AnalyticsView::L, with(Some(Qty)));
    assert_eq!(ranked_of(&l).series[0].key, "qty");
    assert_eq!(ranked_of(&l).series[0].kind, ColumnKind::Number);
    assert_eq!(points(ranked_of(&l)), [30, 20, 10]);
    // View H has one chart for each of its three lists.
    let h = page(&w, AnalyticsView::H, with(Some(Gross)));
    assert_eq!(h.charts.len(), 3);
    assert!(h.charts.iter().all(|c| c.series[0].key == "gross"));
    assert_eq!(points(&h.charts[0]), [29_400, 12_000]);
    assert_eq!(points(&h.charts[1]), [29_400, 12_000]);
    assert_eq!(points(&h.charts[2]), [17_400, 12_000, 12_000]);
}

#[test]
fn the_chart_of_view_g_with_a_measure_is_that_measure_alone_and_the_table_is_unchanged() {
    let w = full_day();
    let classic = page(&w, AnalyticsView::G, with(None));
    // Without a measure: the five series it always had.
    assert_eq!(ranked_of(&classic).series.len(), 5);
    for measure in ALL_MEASURES {
        let p = page(&w, AnalyticsView::G, with(Some(measure)));
        let chart = ranked_of(&p);
        assert_eq!(chart.kind, AnalyticsChartKind::Trend);
        assert_eq!(chart.series.len(), 1, "{measure:?}");
        assert_eq!(chart.series[0].key, measure.key(), "{measure:?}");
        assert_eq!(chart.series[0].kind, measure.kind(), "{measure:?}");
        assert_eq!(points(chart), [expected(measure)[2]], "{measure:?}");
        // The point is of the period of the row, and opens its lines.
        assert_eq!(chart.points[0].key, classic.rows[0].key);
        assert_eq!(p.columns, classic.columns, "{measure:?}");
        assert_eq!(p.rows, classic.rows, "{measure:?}");
        assert_eq!(p.totals, classic.totals, "{measure:?}");
    }
}

// ---- Filters: backend totals over the whole filter ----------------------------------------

#[test]
fn a_buyer_filter_with_a_measure_counts_over_the_whole_filter() {
    let w = full_day();
    let filters = |measure| AnalyticsFilters {
        buyer: Some(PartyFilter {
            account_id: w.akram,
            person_key: None,
        }),
        measure: Some(measure),
        ..AnalyticsFilters::default()
    };
    // اکرم bought 20 کیلا at 600: gross 12,000; his bill 12,000 + 720 + 160 = 12,880. The total
    // commission of the lines he bought is both sides, as view A and H count it (PRD 18.2): the
    // seller commission 720 + the buyer commission 720 + لاگا 160 = 1,600 (H of the worked day).
    for (measure, kela) in [
        (Gross, 12_000),
        (BuyerTotal, 12_880),
        (Qty, 20),
        (TotalComm, 1_600),
        (BuyerComm, 720),
        (Laga, 160),
        (SellerComm, 720),
    ] {
        let d = page(&w, AnalyticsView::D, filters(measure));
        let rows = data_rows(&d);
        assert_eq!(rows.len(), 1, "{measure:?}");
        assert_eq!(name(&rows[0].cells[0]), "Banana", "{measure:?}");
        assert_eq!(
            figure(&d.totals[col(&d, measure.key())]),
            kela,
            "{measure:?}"
        );
        let a = page(&w, AnalyticsView::A, filters(measure));
        assert_eq!(points(ranked_of(&a)), [kela], "{measure:?}");
    }
}

#[test]
fn a_seller_an_item_and_a_goods_filter_with_a_measure() {
    let w = full_day();
    let seller = AnalyticsFilters {
        seller_id: Some(w.karim),
        measure: Some(Gross),
        ..AnalyticsFilters::default()
    };
    let d = page(&w, AnalyticsView::D, seller.clone());
    assert_eq!(figure(&d.totals[col(&d, "gross")]), 12_000);
    let h = page(&w, AnalyticsView::H, seller);
    assert_eq!(
        section_rows(&h, col(&h, "gross")),
        [
            vec![("Apple".to_owned(), 12_000)],
            vec![("Karim".to_owned(), 12_000)],
            vec![("Shahid".to_owned(), 12_000)],
        ]
    );
    let item = AnalyticsFilters {
        item_id: Some(w.banana),
        measure: Some(Qty),
        ..AnalyticsFilters::default()
    };
    let e = page(&w, AnalyticsView::E, item);
    assert_eq!(figure(&e.totals[col(&e, "qty")]), 50);
    // Every lot of the worked day is consigned goods; there are no trip or garden goods.
    for (goods, total) in [
        (AnalyticsGoods::Consigned, 41_400),
        (AnalyticsGoods::Trip, 0),
        (AnalyticsGoods::Garden, 0),
    ] {
        let f = AnalyticsFilters {
            goods,
            measure: Some(Gross),
            ..AnalyticsFilters::default()
        };
        let d = page(&w, AnalyticsView::D, f.clone());
        assert_eq!(figure(&d.totals[col(&d, "gross")]), total, "{goods:?}");
        let g = page(&w, AnalyticsView::G, f);
        let at = col(&g, "gross");
        assert_eq!(figure(&g.totals[at]), total, "{goods:?}");
    }
    // A group of items: the fruit (کیلا and سیب) is all of the day.
    let group = AnalyticsFilters {
        item_group: Some(mandi_services::core::rules::ItemGroup::Fruit),
        measure: Some(BuyerTotal),
        ..AnalyticsFilters::default()
    };
    let d = page(&w, AnalyticsView::D, group);
    assert_eq!(figure(&d.totals[col(&d, "buyer_total")]), 44_364);
}

#[test]
fn the_side_narrows_the_commissions_only() {
    let w = full_day();
    let by = |measure, side| {
        let d = page(
            &w,
            AnalyticsView::D,
            AnalyticsFilters {
                measure: Some(measure),
                side,
                ..AnalyticsFilters::default()
            },
        );
        figure(&d.totals[col(&d, measure.key())])
    };
    assert_eq!(by(TotalComm, AnalyticsSide::Seller), 2_484);
    assert_eq!(by(TotalComm, AnalyticsSide::Buyer), 2_964);
    assert_eq!(by(SellerComm, AnalyticsSide::Seller), 2_484);
    assert_eq!(by(SellerComm, AnalyticsSide::Buyer), 0);
    assert_eq!(by(BuyerComm, AnalyticsSide::Seller), 0);
    assert_eq!(by(Laga, AnalyticsSide::Buyer), 480);
    // Sales, the buyers' bills and the quantity are not commissions.
    for side in [AnalyticsSide::Seller, AnalyticsSide::Buyer] {
        assert_eq!(by(Gross, side), 41_400);
        assert_eq!(by(BuyerTotal, side), 44_364);
        assert_eq!(by(Qty, side), 60);
    }
}

// ---- Comparison, grouping ------------------------------------------------------------------

#[test]
fn a_comparison_follows_the_measure_and_its_kind() {
    let w = full_day();
    let filters = |measure| AnalyticsFilters {
        from: "10-10-2026".into(),
        to: "10-10-2026".into(),
        compare: AnalyticsCompare::Previous,
        measure: Some(measure),
        ..AnalyticsFilters::default()
    };
    for measure in [Gross, Qty] {
        for view in [
            AnalyticsView::D,
            AnalyticsView::E,
            AnalyticsView::H,
            AnalyticsView::L,
        ] {
            let p = page(&w, view, filters(measure));
            let compare = col(&p, "compare");
            let change = col(&p, "change");
            assert_eq!(
                p.columns[compare].kind,
                measure.kind(),
                "{view:?} {measure:?}"
            );
            assert_eq!(
                p.columns[change].kind,
                measure.kind(),
                "{view:?} {measure:?}"
            );
            // Nothing the day before: the compared figure is 0 and the change is the whole figure.
            let total = figure(&p.totals[col(&p, measure.key())]);
            assert_eq!(figure(&p.totals[compare]), 0, "{view:?} {measure:?}");
            assert_eq!(figure(&p.totals[change]), total, "{view:?} {measure:?}");
            assert_eq!(total, expected(measure)[2], "{view:?} {measure:?}");
        }
    }
    // The ranked chart carries the compared figure as its second series.
    let a = page(&w, AnalyticsView::A, filters(Gross));
    let chart = ranked_of(&a);
    assert_eq!(points(chart), [29_400, 12_000]);
    assert!(chart.points.iter().all(|p| p.compare
        == Some(Cell::Money {
            paisa: mandi_services::core::money::Paisa::ZERO
        })));
}

#[test]
fn a_matrix_grouped_by_month_takes_the_measure_too() {
    let w = full_day();
    for view in [AnalyticsView::E, AnalyticsView::L] {
        let p = page(
            &w,
            view,
            AnalyticsFilters {
                group_by: Some(AnalyticsGroupBy::Month),
                measure: Some(Qty),
                ..AnalyticsFilters::default()
            },
        );
        let at = col(&p, "qty");
        assert_eq!(p.columns[at].kind, ColumnKind::Number, "{view:?}");
        // The period first, then one row for each seller (or buyer) of the month.
        assert_eq!(p.columns[0].key, "period");
        assert_eq!(figure(&p.totals[at]), 60, "{view:?}");
        let rows = data_rows(&p);
        let sum: i64 = rows.iter().map(|r| figure(&r.cells[at])).sum();
        assert_eq!(sum, 60, "{view:?}");
        assert!(rows.iter().all(|r| is_number(&r.cells[at])), "{view:?}");
    }
}

// ---- The inputs ----------------------------------------------------------------------------

#[test]
fn a_view_that_counts_no_sale_refuses_a_measure() {
    let w = full_day();
    for view in [
        AnalyticsView::F,
        AnalyticsView::I,
        AnalyticsView::J,
        AnalyticsView::K,
        AnalyticsView::M,
    ] {
        let items = refused(&w, view, with(Some(Gross)));
        assert!(
            items.iter().any(
                |(code, field)| *code == ErrorCode::ErrOutOfRange && field == "filters.measure"
            ),
            "{view:?}: {items:?}"
        );
        // No measure is fine, as before.
        let _ = page(&w, view, with(None));
    }
    // The views that count sales take every measure.
    for view in [
        AnalyticsView::A,
        AnalyticsView::B,
        AnalyticsView::C,
        AnalyticsView::D,
        AnalyticsView::E,
        AnalyticsView::G,
        AnalyticsView::H,
        AnalyticsView::L,
    ] {
        for measure in ALL_MEASURES {
            let p = page(&w, view, with(Some(measure)));
            assert_eq!(p.filters.measure, Some(measure));
        }
    }
}

#[test]
fn the_screens_catalogue_lists_the_measure_and_the_assistants_does_not() {
    let takes = [
        AnalyticsView::A,
        AnalyticsView::B,
        AnalyticsView::C,
        AnalyticsView::D,
        AnalyticsView::E,
        AnalyticsView::G,
        AnalyticsView::H,
        AnalyticsView::L,
    ];
    for entry in screen_catalog() {
        assert_eq!(
            entry.filters.contains(&AnalyticsFilterKind::Measure),
            takes.contains(&entry.id),
            "{:?}",
            entry.id
        );
    }
    for entry in catalog() {
        assert!(
            !entry.filters.contains(&AnalyticsFilterKind::Measure),
            "{:?}",
            entry.id
        );
    }
}

#[test]
fn the_measure_travels_in_the_filters_and_is_left_out_when_none() {
    let none = serde_json::to_value(with(None)).unwrap();
    assert!(none.get("measure").is_none(), "{none}");
    let some = serde_json::to_value(with(Some(BuyerTotal))).unwrap();
    assert_eq!(some["measure"], "buyer_total");
    let back: AnalyticsFilters = serde_json::from_value(some).unwrap();
    assert_eq!(back.measure, Some(BuyerTotal));
    // A saved view of before plan 84 has no measure.
    let old: AnalyticsFilters = serde_json::from_str("{\"from\":\"\",\"to\":\"\"}").unwrap();
    assert_eq!(old.measure, None);
    for measure in ALL_MEASURES {
        let text = serde_json::to_string(&measure).unwrap();
        assert_eq!(text, format!("\"{}\"", measure.key()));
    }
}

// ---- Print and Excel name the measure -------------------------------------------------------

#[test]
fn the_print_header_names_the_measure() {
    let w = full_day();
    for (measure, id) in [
        (Gross, "analytics-filter-measure-gross"),
        (BuyerTotal, "analytics-filter-measure-buyer-total"),
        (Qty, "analytics-filter-measure-qty"),
        (TotalComm, "analytics-filter-measure-total-comm"),
        (SellerComm, "analytics-filter-measure-seller-comm"),
        (BuyerComm, "analytics-filter-measure-buyer-comm"),
        (Laga, "analytics-filter-measure-laga"),
    ] {
        let printed = print_data(
            w.env.ctx.db(),
            &AnalyticsPrintInput {
                view: AnalyticsView::D,
                filters: with(Some(measure)),
                language: Language::En,
                paper: None,
                sort: None,
            },
        )
        .unwrap();
        assert!(
            printed.filters.iter().any(|f| f.label_id == id),
            "{measure:?}: {:?}",
            printed.filters
        );
        // The row total column is of the measure, with its own key and the printed totals.
        assert!(printed.columns.iter().any(|c| c.key == measure.key()));
        assert_eq!(printed.charts.len(), 1);
        assert_eq!(printed.charts[0].series[0].key, measure.key());
    }
    // No measure: no such line.
    let printed = print_data(
        w.env.ctx.db(),
        &AnalyticsPrintInput {
            view: AnalyticsView::D,
            filters: with(None),
            language: Language::En,
            paper: None,
            sort: None,
        },
    )
    .unwrap();
    assert!(
        !printed
            .filters
            .iter()
            .any(|f| f.label_id.starts_with("analytics-filter-measure-"))
    );
}
