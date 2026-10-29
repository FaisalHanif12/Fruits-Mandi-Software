//! Analytics views F (partners), G (trend) and J (trips) of plan d5-late, round 11 (PRD 18.2,
//! FR-17.5, FR-17.9, FR-12.27, FR-18.1 to FR-18.8, C12, C21, C22, C25; E2E-34, E2E-34b, E2E-41).
//!
//! The expected numbers are the PRD's and the money agent's worked numbers:
//!
//! * the worked day (Section 4): income shares 2,196 / 1,646 / 1,646, expense shares 1,200 / 900 /
//!   900, profit 996 / 746 / 746; day figures 41,400, 5,448, 5,488, 3,000, 2,488;
//! * E2E-34: bought 20 for 10,400, sold 15 for 8,460 after commission 1,200 (540 seller side),
//!   cost sold 7,800, left 5 for 2,600, result 660, parts -776 / -582 / -582;
//! * E2E-34b: the last 5 written off, result -1,940;
//! * E2E-41: sales money 105,280, commission 14,080 (6,720 seller side), result 5,280, parts 2,640
//!   and 2,640, partner totals changing by 8,272 / 56,864 / 4,224.
//!
//! Each view is also held against the report or the screen that shows the same figure (R33, the
//! partner overview), so they cannot disagree.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::analytics::{
    AnalyticsCompare, AnalyticsFilters, AnalyticsGoods, AnalyticsGroupBy, AnalyticsLinesInput,
    AnalyticsPage, AnalyticsPeriod, AnalyticsPrintInput, AnalyticsRunInput, AnalyticsSide,
    AnalyticsTripView, AnalyticsView, catalog, lines, print_data, run,
};
use mandi_reports::reports::{Cell, PartyFilter, ReportFilters, ReportId, RowKind};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::BikriSaveInput;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::purchases::{PurchaseInput, PurchaseWriteOffInput};
use mandi_services::core::trips::{TripCostInput, TripInput};
use mandi_services::core::ventures::{EntryPartInput, MoneySource, VenturePartnerInput};
use mandi_services::{bikri, partners, purchases, trips};
use reports_world::*;
use support::World;

// ---- Reading a page ------------------------------------------------------------------------

fn filters() -> AnalyticsFilters {
    AnalyticsFilters::default()
}

fn view(w: &World, v: AnalyticsView, f: AnalyticsFilters) -> AnalyticsPage {
    run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            sort: None,
            view: v,
            filters: f,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
}

fn refused(w: &World, v: AnalyticsView, f: AnalyticsFilters) -> Vec<(ErrorCode, String)> {
    match run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            sort: None,
            view: v,
            filters: f,
            offset: 0,
            limit: 0,
        },
    ) {
        Err(mandi_reports::ReportsError::Rejected(items)) => {
            items.into_iter().map(|i| (i.code, i.field)).collect()
        }
        other => panic!("expected a rejection, got {other:?}"),
    }
}

/// Whole rupees, a count or hundredths of a percent of a cell; 0 for an empty one.
fn n(cell: &Cell) -> i64 {
    match cell {
        Cell::Number { value } => *value,
        Cell::Money { paisa } => paisa.rupees(),
        Cell::Percent { hundredths } => *hundredths,
        Cell::Empty => 0,
        other => panic!("not a number: {other:?}"),
    }
}

fn name_of(cell: &Cell) -> String {
    match cell {
        Cell::Names { names } => names.en.clone(),
        other => panic!("not names: {other:?}"),
    }
}

fn data_rows(page: &AnalyticsPage) -> Vec<&mandi_reports::analytics::AnalyticsRow> {
    page.rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .collect()
}

fn column(page: &AnalyticsPage, key: &str) -> usize {
    page.columns
        .iter()
        .position(|c| c.key == key)
        .unwrap_or_else(|| panic!("no column {key}: {:?}", page.columns))
}

/// The numbers of a row in the given columns.
fn pick(row: &[Cell], keys: &[usize]) -> Vec<i64> {
    keys.iter().map(|i| n(&row[*i])).collect()
}

fn only_fields(found: Vec<(ErrorCode, String)>) -> Vec<String> {
    found.into_iter().map(|e| e.1).collect()
}

// ---- The trip worlds -------------------------------------------------------------------------

fn shop(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    }
}

fn pocket(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Pocket,
    }
}

fn new_trip(w: &World, partners: Vec<VenturePartnerInput>) -> i64 {
    trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners,
        },
    )
    .unwrap()
    .id
}

fn orange(w: &World) -> i64 {
    support::add_item(&w.env, &w.owner, "مالٹا", "Orange", "fruit", "unit")
}

fn buy(w: &World, trip: i64, item: i64, qty: &str, rate: &str, freight: &str) -> i64 {
    purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(trip),
            date: "10-10-2026".into(),
            supplier: Some("Orange Traders".into()),
            item_id: Some(item),
            packing_id: Some(w.box_),
            qty: qty.into(),
            weight: None,
            rate: rate.into(),
            freight: (!freight.is_empty()).then(|| freight.to_owned()),
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::<EntryPartInput>::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .id
}

fn sell_own(w: &World, purchase: i64, qty: &str, buyer: i64, rate: &str) -> i64 {
    sell_own_on(w, "10-10-2026", purchase, qty, buyer, rate)
}

/// The same sale on a given day (the header of the support draft is dated 10-10-26).
fn sell_own_on(w: &World, date: &str, purchase: i64, qty: &str, buyer: i64, rate: &str) -> i64 {
    let lot = mandi_services::core::bikri::LotInput {
        seller_account_id: Some(support::id_of(&w.env, "030119998")),
        purchase_id: Some(purchase),
        item_id: None,
        packing_id: None,
        adjust_advance: false,
        ..support::bare_lot(w, "own-1", w.rahim, w.banana, qty)
    };
    let mut draft = support::draft(vec![support::with_sales(
        lot,
        vec![support::sale("s-1", buyer, qty, rate)],
    )]);
    draft.header.entry_date = date.into();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .bikri_id
}

/// E2E-34: a trip of the three partners at 40 / 30 / 30 paying from the shop's cash; 20 پیٹی
/// مالٹا at 500 with freight 400; an own lot sells 15 of them to اکرم at 600 on credit.
/// Returns the world, the trip, the purchase and the بکری of the sale.
fn trip_34() -> (World, i64, i64, i64) {
    let w = worked_day();
    let item = orange(&w);
    let [a, b, c] = w.partners;
    let trip = new_trip(&w, vec![shop(a, "40"), shop(b, "30"), shop(c, "30")]);
    let purchase = buy(&w, trip, item, "20", "500", "400");
    let sale = sell_own(&w, purchase, "15", w.akram, "600");
    (w, trip, purchase, sale)
}

/// E2E-34b: the last 5 spoil on 11-10-26.
fn write_off_5(w: &World, purchase: i64) {
    set_today(&w.env, 2026, 10, 11);
    purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: purchase,
            qty: "5".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
}

/// E2E-41: الف 50% from the shop, ب 50% from his pocket, ج not in it; 80 مالٹا at 1,200 and a
/// truck of 4,000; one own lot sells all 80 to ارشد at 1,400. Returns the world, trip, purchase.
fn trip_41() -> (World, i64, i64) {
    let w = worked_day();
    let item = orange(&w);
    let [a, b, c] = w.partners;
    let trip = new_trip(&w, vec![shop(a, "50"), pocket(b, "50"), shop(c, "0")]);
    let purchase = buy(&w, trip, item, "80", "1,200", "");
    trips::cost_save(
        &w.env.ctx,
        &w.owner,
        &TripCostInput {
            trip_id: Some(trip),
            date: "10-10-2026".into(),
            amount: "4,000".into(),
            reason: "truck".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    sell_own(&w, purchase, "80", w.arshad, "1,400");
    (w, trip, purchase)
}

fn by(id: i64) -> AnalyticsFilters {
    AnalyticsFilters {
        partner_id: Some(id),
        ..filters()
    }
}

fn daily() -> AnalyticsFilters {
    AnalyticsFilters {
        group_by: Some(AnalyticsGroupBy::Day),
        ..filters()
    }
}

fn save(w: &World, draft: &mandi_services::core::bikri::BikriDraft) {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

// ---- View F: partners --------------------------------------------------------------------------

/// The figure columns of view F, by key.
const F_FIGURES: [&str; 8] = [
    "income_share",
    "expense_share",
    "profit_share",
    "gardens",
    "trips",
    "fruit",
    "other",
    "net_change",
];

fn f_rows(page: &AnalyticsPage) -> Vec<Vec<i64>> {
    let keys: Vec<usize> = F_FIGURES.iter().map(|k| column(page, k)).collect();
    data_rows(page)
        .iter()
        .map(|r| pick(&r.cells, &keys))
        .collect()
}

#[test]
fn the_catalogue_lists_f_g_and_j_with_their_filters() {
    let entries = catalog();
    let find = |v: AnalyticsView| entries.iter().find(|e| e.id == v).unwrap();
    let has =
        |v: AnalyticsView, kind: &str| find(v).filters.iter().any(|f| format!("{f:?}") == kind);
    for v in [AnalyticsView::F, AnalyticsView::G, AnalyticsView::J] {
        assert!(has(v, "Partner"), "{v:?}");
        assert!(!has(v, "Compare"), "{v:?}");
    }
    assert!(has(AnalyticsView::J, "TripView"));
    assert!(has(AnalyticsView::G, "GroupBy"));
    assert!(!has(AnalyticsView::A, "Partner"));
    assert!(has(AnalyticsView::A, "Goods"));
    assert!(has(AnalyticsView::A, "Trip"));
}

#[test]
fn view_f_on_the_worked_day_is_the_partner_table_of_the_prd() {
    let w = full_day();
    let page = view(&w, AnalyticsView::F, filters());
    // Income 2,196 / 1,646 / 1,646; expense 1,200 / 900 / 900; profit 996 / 746 / 746; no
    // trips; net change = profit share (the 2,000 الف took is not in it).
    assert_eq!(
        f_rows(&page),
        vec![
            vec![2_196, 1_200, 996, 0, 0, 0, 0, 996],
            vec![1_646, 900, 746, 0, 0, 0, 0, 746],
            vec![1_646, 900, 746, 0, 0, 0, 0, 746],
        ]
    );
    let names: Vec<String> = data_rows(&page)
        .iter()
        .map(|r| name_of(&r.cells[0]))
        .collect();
    assert_eq!(names, vec!["Partner A", "Partner B", "Partner C"]);
    // The totals row adds the rows: 5,488 / 3,000 / 2,488.
    let keys: Vec<usize> = F_FIGURES.iter().map(|k| column(&page, k)).collect();
    assert_eq!(
        pick(&page.totals, &keys),
        vec![5_488, 3_000, 2_488, 0, 0, 0, 0, 2_488]
    );
    // The gardens and fruit columns are money (0 here: no garden and no fruit on the worked day).
    for r in data_rows(&page) {
        for key in ["gardens", "fruit"] {
            assert!(matches!(
                r.cells[column(&page, key)],
                Cell::Money { paisa } if paisa.rupees() == 0
            ));
        }
    }
}

#[test]
fn view_f_agrees_with_the_partner_overview() {
    // The month figures of the overview are the same sums for the month of today.
    let w = full_day();
    let page = view(&w, AnalyticsView::F, filters());
    let overview = partners::overview(&w.env.ctx).unwrap();
    let profits: Vec<i64> = overview
        .iter()
        .map(|p| p.month_profit_paisa.rupees())
        .collect();
    let f_profit: Vec<i64> = f_rows(&page).iter().map(|r| r[2]).collect();
    assert_eq!(profits, f_profit);
}

#[test]
fn view_f_e2e_34_the_trip_money_is_in_the_trips_column_and_the_net_is_the_partner_total() {
    let (w, _, _, _) = trip_34();
    let page = view(&w, AnalyticsView::F, filters());
    // Commission 1,200 shared 480 / 360 / 360; trips -4,160 + 3,384 = -776 and so on; net change
    // -296 / -222 / -222, the partner totals of the PRD.
    assert_eq!(
        f_rows(&page),
        vec![
            vec![480, 0, 480, 0, -776, 0, 0, -296],
            vec![360, 0, 360, 0, -582, 0, 0, -222],
            vec![360, 0, 360, 0, -582, 0, 0, -222],
        ]
    );
}

#[test]
fn view_f_e2e_41_a_pocket_part_is_not_in_the_books_and_the_net_is_what_the_total_changed_by() {
    let (w, _, _) = trip_41();
    let page = view(&w, AnalyticsView::F, filters());
    // الف: shop part -50,000 + back 52,640 = 2,640. ب: his part came from his pocket, so only
    // the 52,640 is in the books. ج is not in the trip.
    assert_eq!(
        f_rows(&page),
        vec![
            vec![5_632, 0, 5_632, 0, 2_640, 0, 0, 8_272],
            vec![4_224, 0, 4_224, 0, 52_640, 0, 0, 56_864],
            vec![4_224, 0, 4_224, 0, 0, 0, 0, 4_224],
        ]
    );
}

#[test]
fn view_f_picks_a_partner_and_a_period() {
    let (w, _, _) = trip_41();
    let [_, b, c] = w.partners;
    let one = view(&w, AnalyticsView::F, by(c));
    assert_eq!(f_rows(&one), vec![vec![4_224, 0, 4_224, 0, 0, 0, 0, 4_224]]);
    let one = view(&w, AnalyticsView::F, by(b));
    assert_eq!(f_rows(&one).len(), 1);
    // A period with nothing in it: every active partner is still listed, with zeros.
    set_today(&w.env, 2026, 10, 12);
    let later = view(
        &w,
        AnalyticsView::F,
        AnalyticsFilters {
            from: "11-10-2026".into(),
            to: "12-10-2026".into(),
            ..filters()
        },
    );
    assert!(f_rows(&later).iter().all(|r| r.iter().all(|v| *v == 0)));
}

#[test]
fn view_f_after_a_cancelled_sale_has_the_reversal_in_the_same_figures() {
    let (w, _, _, sale) = trip_34();
    let detail = bikri::get(&w.env.ctx, sale).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::CancelInput {
            id: sale,
            version: detail.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // The sale and its reversal are both in the period: nothing is left of the commission or
    // of the trip credit; only the shop's part of the purchase (-4,160 / -3,120 / -3,120).
    let page = view(&w, AnalyticsView::F, filters());
    assert_eq!(
        f_rows(&page),
        vec![
            vec![0, 0, 0, 0, -4_160, 0, 0, -4_160],
            vec![0, 0, 0, 0, -3_120, 0, 0, -3_120],
            vec![0, 0, 0, 0, -3_120, 0, 0, -3_120],
        ]
    );
}

#[test]
fn view_f_opens_the_share_rows_behind_a_partner() {
    let w = full_day();
    let page = view(&w, AnalyticsView::F, filters());
    let first = data_rows(&page)[0];
    let key = first.key.clone().unwrap();
    assert!(key.partner_id.is_some());
    let behind = lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::F,
            filters: filters(),
            row_key: key,
            column_key: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    // His share rows: 1,580 (بکری 1) + 616 (بکری 2) - 1,200 (the electricity) = 996.
    assert_eq!(behind.total_count, 3);
    assert_eq!(n(behind.totals.last().unwrap()), 996);
}

// ---- View G: trend ---------------------------------------------------------------------------------

const G_FIGURES: [&str; 6] = [
    "gross",
    "commission",
    "shared_income",
    "shop_expenses",
    "profit",
    "trips_result",
];

fn g_rows(page: &AnalyticsPage) -> Vec<Vec<i64>> {
    let keys: Vec<usize> = G_FIGURES.iter().map(|k| column(page, k)).collect();
    data_rows(page)
        .iter()
        .map(|r| pick(&r.cells, &keys))
        .collect()
}

#[test]
fn view_g_on_the_worked_day_is_the_day_of_the_prd() {
    let w = full_day();
    let page = view(&w, AnalyticsView::G, filters());
    // October 2026: gross 41,400; commission 5,448; shared income 5,488; shop expenses 3,000;
    // profit 2,488. No trip.
    assert_eq!(
        g_rows(&page),
        vec![vec![41_400, 5_448, 5_488, 3_000, 2_488, 0]]
    );
    let by_day = view(&w, AnalyticsView::G, daily());
    assert_eq!(g_rows(&by_day), g_rows(&page));
}

#[test]
fn view_g_e2e_34_shows_the_trip_result_on_the_day_of_the_sale() {
    let (w, _, _, _) = trip_34();
    let page = view(&w, AnalyticsView::G, daily());
    // Gross 9,000; commission 540 + 540 + 120 = 1,200 all shared; no shop expense; the trips'
    // result for the day 660.
    assert_eq!(
        g_rows(&page),
        vec![vec![9_000, 1_200, 1_200, 0, 1_200, 660]]
    );
}

#[test]
fn view_g_e2e_34b_the_write_off_is_a_loss_on_its_own_day_and_the_two_days_add_to_r33() {
    let (w, _, purchase, _) = trip_34();
    write_off_5(&w, purchase);
    let page = view(&w, AnalyticsView::G, daily());
    let rows = g_rows(&page);
    assert_eq!(rows.len(), 2);
    // 10-10: the sale (660); 11-10: the 5 written off (-2,600). Together -1,940.
    assert_eq!(rows[0][5], 660);
    assert_eq!(rows[1], vec![0, 0, 0, 0, 0, -2_600]);
    assert_eq!(n(&page.totals[column(&page, "trips_result")]), -1_940);
    // R33 says the same for the whole trip.
    let r33 = report(&w, ReportId::R33, ReportFilters::default(), 0, 0);
    assert_eq!(n(r33.totals.last().unwrap()), -1_940);
}

#[test]
fn view_g_with_a_partner_has_his_trip_money_and_it_is_the_trips_column_of_view_f() {
    let (w, _, _) = trip_41();
    let [_, b, _] = w.partners;
    let g = view(&w, AnalyticsView::G, by(b));
    let f = view(&w, AnalyticsView::F, by(b));
    let g_trips = n(&g.totals[column(&g, "trips_result")]);
    let f_trips = n(&f.totals[column(&f, "trips")]);
    assert_eq!(g_trips, f_trips);
    assert_eq!(g_trips, 52_640);
}

#[test]
fn view_g_leaves_the_trip_column_empty_when_a_buyer_filter_cannot_narrow_it() {
    let w = full_day();
    let page = view(
        &w,
        AnalyticsView::G,
        AnalyticsFilters {
            buyer: Some(PartyFilter {
                account_id: w.akram,
                person_key: None,
            }),
            ..filters()
        },
    );
    let trips_idx = column(&page, "trips_result");
    assert!(
        data_rows(&page)
            .iter()
            .all(|r| matches!(r.cells[trips_idx], Cell::Empty))
    );
}

// ---- View J: trips ---------------------------------------------------------------------------------

/// The eleven figures after the first column: bought qty and cost, sold qty, sales money,
/// commission, cost sold, written off qty and cost, left qty and cost, result.
fn j_figures(page: &AnalyticsPage, row: usize) -> Vec<i64> {
    data_rows(page)[row].cells[1..=11].iter().map(n).collect()
}

fn part(page: &AnalyticsPage, partner: i64) -> i64 {
    n(&data_rows(page)[0].cells[column(page, &format!("part_{partner}"))])
}

fn share(page: &AnalyticsPage, partner: i64) -> i64 {
    n(&data_rows(page)[0].cells[column(page, &format!("share_{partner}"))])
}

#[test]
fn view_j_e2e_34_the_trip_after_the_sale_of_15() {
    let (w, _, _, _) = trip_34();
    let page = view(&w, AnalyticsView::J, filters());
    assert_eq!(data_rows(&page).len(), 1);
    // bought 20 / 10,400; sold 15; sales money 8,460; commission on its goods 1,200 (seller side
    // 540 + buyer side 540 + لاگا 120); cost sold 7,800; written off 0 / 0; left 5 / 2,600;
    // result 660.
    assert_eq!(
        j_figures(&page, 0),
        vec![20, 10_400, 15, 8_460, 1_200, 7_800, 0, 0, 5, 2_600, 660]
    );
    // The seller side only: 540 (E2E-34).
    let seller = view(
        &w,
        AnalyticsView::J,
        AnalyticsFilters {
            side: AnalyticsSide::Seller,
            ..filters()
        },
    );
    assert_eq!(j_figures(&seller, 0)[4], 540);
    // The parts of the partners: -776 / -582 / -582, and the shares 40 / 30 / 30 (hundredths).
    let [a, b, c] = w.partners;
    assert_eq!(
        (part(&page, a), part(&page, b), part(&page, c)),
        (-776, -582, -582)
    );
    assert_eq!(
        (share(&page, a), share(&page, b), share(&page, c)),
        (4_000, 3_000, 3_000)
    );
    // The parts plus the cost still left are the result: -1,940 + 2,600 = 660.
    assert_eq!(
        part(&page, a) + part(&page, b) + part(&page, c) + 2_600,
        660
    );
    // The same result as R33.
    let r33 = report(&w, ReportId::R33, ReportFilters::default(), 0, 0);
    assert_eq!(n(r33.totals.last().unwrap()), 660);
}

#[test]
fn view_j_e2e_34b_after_the_write_off_the_trip_has_nothing_left_and_a_loss() {
    let (w, _, purchase, _) = trip_34();
    write_off_5(&w, purchase);
    let page = view(&w, AnalyticsView::J, filters());
    assert_eq!(
        j_figures(&page, 0),
        vec![20, 10_400, 15, 8_460, 1_200, 7_800, 5, 2_600, 0, 0, -1_940]
    );
    let [a, b, c] = w.partners;
    // -776 / -582 / -582 are already in the partner totals; they add up to the result.
    assert_eq!(part(&page, a) + part(&page, b) + part(&page, c), -1_940);
}

#[test]
fn view_j_e2e_41_a_pocket_part_a_truck_and_a_trip_sold_out() {
    let (w, _, _) = trip_41();
    let page = view(&w, AnalyticsView::J, filters());
    assert_eq!(
        j_figures(&page, 0),
        vec![80, 100_000, 80, 105_280, 14_080, 100_000, 0, 0, 0, 0, 5_280]
    );
    let seller = view(
        &w,
        AnalyticsView::J,
        AnalyticsFilters {
            side: AnalyticsSide::Seller,
            ..filters()
        },
    );
    assert_eq!(j_figures(&seller, 0)[4], 6_720);
    // الف and ب each +2,640 (ب's 50,000 came from his pocket); ج has 0% of it, so he has no
    // column.
    let [a, b, c] = w.partners;
    assert_eq!((part(&page, a), part(&page, b)), (2_640, 2_640));
    assert!(
        page.columns
            .iter()
            .all(|col| col.key != format!("share_{c}") && col.key != format!("part_{c}"))
    );
}

/// The commission figures of the trip screens, in rupees: the trip row, its first purchase and the
/// total row of the trips list.
fn screen_commissions(w: &World, trip: i64) -> (i64, i64, i64) {
    let detail = trips::get(
        &w.env.ctx,
        &mandi_services::core::trips::TripGetInput {
            id: trip,
            entries_offset: 0,
            entries_limit: 0,
        },
    )
    .unwrap();
    let list = trips::list(
        &w.env.ctx,
        &mandi_services::core::trips::TripsListInput::default(),
    )
    .unwrap();
    (
        detail.row.commission_paisa.rupees(),
        detail.purchases[0].commission_paisa.rupees(),
        list.totals.commission_paisa.rupees(),
    )
}

#[test]
fn the_commission_of_the_trip_screens_is_the_commission_of_view_j_e2e_34_and_e2e_41() {
    // Plan 36 (PRD FR-12.27): seller commission + buyer commission + لاگا of the sales in our shop.
    // E2E-34: 540 + 540 + 120 = 1,200 (not the seller-only 540 of R33).
    let (w, trip, purchase, _) = trip_34();
    let j = j_figures(&view(&w, AnalyticsView::J, filters()), 0)[4];
    assert_eq!(j, 1_200);
    assert_eq!(screen_commissions(&w, trip), (1_200, 1_200, 1_200));
    // The dashboard tile of the open trips says the same.
    assert_eq!(
        trips::summary(&w.env.ctx)
            .unwrap()
            .commission_paisa
            .rupees(),
        1_200
    );
    // A write-off adds and takes nothing (E2E-34b).
    write_off_5(&w, purchase);
    assert_eq!(screen_commissions(&w, trip), (1_200, 1_200, 1_200));
    // A sale in another shop adds nothing: it is money, not a sale line of ours.
    let (w, trip, purchase, _) = trip_34();
    trips::other_sale_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::trips::OtherSaleInput {
            trip_id: Some(trip),
            purchase_id: Some(purchase),
            date: "10-10-2026".into(),
            shop: Some("Haji Traders".into()),
            qty: "5".into(),
            amount: "3,000".into(),
            note: None,
        },
    )
    .unwrap();
    assert_eq!(screen_commissions(&w, trip), (1_200, 1_200, 1_200));
    // E2E-41: 6,720 + 6,720 + 640 = 14,080.
    let (w, trip, _) = trip_41();
    let j = j_figures(&view(&w, AnalyticsView::J, filters()), 0)[4];
    assert_eq!(j, 14_080);
    assert_eq!(screen_commissions(&w, trip), (14_080, 14_080, 14_080));
}

#[test]
fn view_j_in_every_layout_adds_up_to_the_same_totals() {
    let (w, trip, _, _) = trip_34();
    // A second purchase of the same trip that is not sold at all.
    let item = orange(&w);
    buy(&w, trip, item, "10", "300", "");
    let totals = |v: AnalyticsTripView| {
        let page = view(
            &w,
            AnalyticsView::J,
            AnalyticsFilters {
                trip_view: Some(v),
                ..filters()
            },
        );
        page.totals[1..=11].iter().map(n).collect::<Vec<i64>>()
    };
    let by_trip = totals(AnalyticsTripView::Trip);
    // bought 30; sold 15; left 15.
    assert_eq!((by_trip[0], by_trip[2], by_trip[8]), (30, 15, 15));
    for v in [
        AnalyticsTripView::Purchase,
        AnalyticsTripView::Item,
        AnalyticsTripView::Day,
    ] {
        assert_eq!(totals(v), by_trip, "{v:?}");
    }
    let purchases_page = view(
        &w,
        AnalyticsView::J,
        AnalyticsFilters {
            trip_view: Some(AnalyticsTripView::Purchase),
            ..filters()
        },
    );
    assert_eq!(data_rows(&purchases_page).len(), 2);
}

#[test]
fn view_j_picks_by_partner_goods_and_trip() {
    let (w, trip, _) = trip_41();
    let [a, _, c] = w.partners;
    // ج has 0% of the trip: the partner filter leaves it out.
    assert_eq!(data_rows(&view(&w, AnalyticsView::J, by(c))).len(), 0);
    assert_eq!(data_rows(&view(&w, AnalyticsView::J, by(a))).len(), 1);
    let picked = view(
        &w,
        AnalyticsView::J,
        AnalyticsFilters {
            trip_id: Some(trip),
            goods: AnalyticsGoods::Trip,
            ..filters()
        },
    );
    assert_eq!(data_rows(&picked).len(), 1);
    // Consigned goods have no trips.
    let consigned = view(
        &w,
        AnalyticsView::J,
        AnalyticsFilters {
            goods: AnalyticsGoods::Consigned,
            ..filters()
        },
    );
    assert_eq!(data_rows(&consigned).len(), 0);
}

#[test]
fn view_j_opens_the_sale_lines_of_its_goods() {
    let (w, _, _, _) = trip_34();
    let page = view(&w, AnalyticsView::J, filters());
    let key = data_rows(&page)[0].key.clone().unwrap();
    assert!(!key.purchase_ids.is_empty());
    let behind = lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::J,
            filters: filters(),
            row_key: key,
            column_key: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    // One sale line of 15 نگ.
    assert_eq!(behind.total_count, 1);
}

// ---- The filters of the other views -----------------------------------------------------------------

#[test]
fn the_goods_and_trip_filters_narrow_views_a_to_e_and_h() {
    let (w, trip, _, _) = trip_34();
    // A consigned sale as well: بکری 1 of the worked day.
    save(&w, &support::event1(&w));
    let gross = |p: &AnalyticsPage| n(&p.totals[column(p, "gross")]);
    let with = |goods: AnalyticsGoods, trip_id: Option<i64>| {
        view(
            &w,
            AnalyticsView::A,
            AnalyticsFilters {
                goods,
                trip_id,
                ..filters()
            },
        )
    };
    assert_eq!(gross(&with(AnalyticsGoods::All, None)), 38_400);
    assert_eq!(gross(&with(AnalyticsGoods::Trip, None)), 9_000);
    assert_eq!(gross(&with(AnalyticsGoods::Consigned, None)), 29_400);
    assert_eq!(gross(&with(AnalyticsGoods::All, Some(trip))), 9_000);
    // Gardens are not built yet: no goods.
    let garden = view(
        &w,
        AnalyticsView::H,
        AnalyticsFilters {
            goods: AnalyticsGoods::Garden,
            ..filters()
        },
    );
    assert!(data_rows(&garden).is_empty());
}

#[test]
fn filters_a_view_does_not_take_are_refused_with_the_field() {
    let w = full_day();
    let [a, _, _] = w.partners;
    // Compare on the three new views.
    for v in [AnalyticsView::F, AnalyticsView::G, AnalyticsView::J] {
        let found = refused(
            &w,
            v,
            AnalyticsFilters {
                from: "01-10-2026".into(),
                to: "31-10-2026".into(),
                compare: AnalyticsCompare::Previous,
                ..filters()
            },
        );
        assert_eq!(
            found,
            vec![(ErrorCode::ErrOutOfRange, "filters.compare".to_owned())],
            "{v:?}"
        );
    }
    // A sale filter on view F; a partner on views A to E; trip_view and group_by on the wrong views.
    assert_eq!(
        only_fields(refused(
            &w,
            AnalyticsView::F,
            AnalyticsFilters {
                item_id: Some(w.banana),
                ..filters()
            }
        )),
        vec!["filters.item_id".to_owned()]
    );
    assert_eq!(
        only_fields(refused(&w, AnalyticsView::A, by(a))),
        vec!["filters.partner_id".to_owned()]
    );
    assert_eq!(
        only_fields(refused(
            &w,
            AnalyticsView::A,
            AnalyticsFilters {
                trip_view: Some(AnalyticsTripView::Day),
                ..filters()
            }
        )),
        vec!["filters.trip_view".to_owned()]
    );
    assert_eq!(
        only_fields(refused(
            &w,
            AnalyticsView::J,
            AnalyticsFilters {
                group_by: Some(AnalyticsGroupBy::Day),
                ..filters()
            }
        )),
        vec!["filters.group_by".to_owned()]
    );
}

#[test]
fn a_period_shortcut_and_a_page_work_on_the_new_views() {
    let w = full_day();
    for v in [AnalyticsView::F, AnalyticsView::G, AnalyticsView::J] {
        let page = run(
            w.env.ctx.db(),
            &AnalyticsRunInput {
                sort: None,
                view: v,
                filters: AnalyticsFilters {
                    period: Some(AnalyticsPeriod::ThisYear),
                    ..filters()
                },
                offset: 1,
                limit: 1,
            },
        )
        .unwrap();
        assert!(page.rows.len() <= 1, "{v:?}");
        assert_eq!(page.view, v);
    }
}

// ---- Print ------------------------------------------------------------------------------------------

#[test]
fn the_new_views_print_in_both_languages_with_the_rows_of_the_page() {
    let (w, _, _) = trip_41();
    for v in [AnalyticsView::F, AnalyticsView::G, AnalyticsView::J] {
        let page = view(&w, v, filters());
        for language in [Language::Ur, Language::En] {
            let data = print_data(
                w.env.ctx.db(),
                &AnalyticsPrintInput {
                    view: v,
                    filters: filters(),
                    language,
                    paper: None,
                },
            )
            .unwrap();
            assert_eq!(data.rows.len(), page.rows.len(), "{v:?} {language:?}");
            assert_eq!(data.columns.len(), page.columns.len(), "{v:?} {language:?}");
        }
    }
}

// ---- Money agent review: M1, M2, S5, S6 -------------------------------------------------------------

/// A second purchase of the trip of E2E-34, of a vegetable, sold for 4,000 in another بکری: 20
/// at 200 (seller comm 240, buyer comm 240, no لاگا on a vegetable, so 480).
fn second_item_sold(w: &World, trip: i64) -> i64 {
    let yam = support::add_item(&w.env, &w.owner, "آلو", "Potato", "vegetable", "unit");
    let purchase = buy(w, trip, yam, "20", "200", "");
    sell_own(w, purchase, "20", w.arshad, "200");
    purchase
}

#[test]
fn view_j_by_trip_commission_is_of_the_whole_trip_whatever_the_item_filter() {
    let (w, trip, _, _) = trip_34();
    let second = second_item_sold(&w, trip);
    let orange = orange_item(&w);
    let all = view(&w, AnalyticsView::J, filters());
    let narrowed = view(
        &w,
        AnalyticsView::J,
        AnalyticsFilters {
            item_id: Some(orange),
            ..filters()
        },
    );
    // The row is of the whole trip in both: the same eleven figures, the commission too.
    assert_eq!(j_figures(&all, 0), j_figures(&narrowed, 0));
    assert_eq!(j_figures(&all, 0)[4], 1_200 + 480);
    // The purchase layout picks the purchases of the item and shows their own commission.
    let by_purchase = view(
        &w,
        AnalyticsView::J,
        AnalyticsFilters {
            item_id: Some(orange),
            trip_view: Some(AnalyticsTripView::Purchase),
            ..filters()
        },
    );
    assert_eq!(data_rows(&by_purchase).len(), 1);
    assert_eq!(j_figures(&by_purchase, 0)[4], 1_200);
    let _ = second;
}

fn orange_item(w: &World) -> i64 {
    w.env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row("SELECT id FROM items WHERE name_en = 'Orange'", [], |r| {
                r.get(0)
            })
        })
        .unwrap()
}

#[test]
fn the_lines_behind_a_trip_row_add_up_to_its_commission_even_when_the_sale_is_after_the_period() {
    // Bought on 10-10, sold on 11-10; the period is 10-10 only. The row says 1,200; the lines
    // behind it are the sale of 11-10 and add up to 1,200.
    let w = worked_day();
    let item = orange(&w);
    let [a, b, c] = w.partners;
    let trip = new_trip(&w, vec![shop(a, "40"), shop(b, "30"), shop(c, "30")]);
    let purchase = buy(&w, trip, item, "20", "500", "400");
    set_today(&w.env, 2026, 10, 11);
    sell_own_on(&w, "11-10-2026", purchase, "15", w.akram, "600");
    let only_10 = AnalyticsFilters {
        from: "10-10-2026".into(),
        to: "10-10-2026".into(),
        trip_view: Some(AnalyticsTripView::Purchase),
        ..filters()
    };
    let page = view(&w, AnalyticsView::J, only_10.clone());
    assert_eq!(j_figures(&page, 0)[4], 1_200);
    let key = data_rows(&page)[0].key.clone().unwrap();
    let behind = lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::J,
            filters: only_10,
            row_key: key,
            column_key: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(behind.total_count, 1);
    assert_eq!(n(behind.totals.last().unwrap()), 1_200);
}

#[test]
fn the_lines_behind_a_trip_row_ignore_the_item_filter_like_the_row_does() {
    let (w, trip, _, _) = trip_34();
    second_item_sold(&w, trip);
    let orange = orange_item(&w);
    let f = AnalyticsFilters {
        item_id: Some(orange),
        ..filters()
    };
    let page = view(&w, AnalyticsView::J, f.clone());
    let key = data_rows(&page)[0].key.clone().unwrap();
    let behind = lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::J,
            filters: f,
            row_key: key,
            column_key: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(behind.total_count, 2);
    assert_eq!(n(behind.totals.last().unwrap()), 1_680);
}

#[test]
fn view_g_names_the_trip_column_as_his_money_when_a_partner_is_picked() {
    let (w, _, _) = trip_41();
    let [_, b, _] = w.partners;
    let own = view(&w, AnalyticsView::G, filters());
    let his = view(&w, AnalyticsView::G, by(b));
    let f = view(&w, AnalyticsView::F, by(b));
    let title = |p: &AnalyticsPage, key: &str| p.columns[column(p, key)].title_id.clone();
    assert_ne!(title(&own, "trips_result"), title(&his, "trips_result"));
    assert_eq!(title(&his, "trips_result"), title(&f, "trips"));
}

#[test]
fn two_purchase_sets_in_a_row_and_a_column_key_are_intersected() {
    let (w, trip, purchase, _) = trip_34();
    let second = second_item_sold(&w, trip);
    let page = view(&w, AnalyticsView::J, filters());
    let key = data_rows(&page)[0].key.clone().unwrap();
    assert!(key.purchase_ids.contains(&purchase) && key.purchase_ids.contains(&second));
    let only_second = mandi_reports::analytics::AnalyticsKey {
        purchase_ids: vec![second],
        ..Default::default()
    };
    let behind = lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::J,
            filters: filters(),
            row_key: key,
            column_key: Some(only_second),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(behind.total_count, 1);
    assert_eq!(n(behind.totals.last().unwrap()), 480);
}

#[test]
fn view_g_with_a_partner_by_day_has_his_money_on_the_day_it_moved() {
    // Bought on 10-10 (the shop pays: -4,160 for الف), sold on 11-10 (+3,384): one query for all
    // the days, and the days add up to view F's trips column.
    let w = worked_day();
    let item = orange(&w);
    let [a, b, c] = w.partners;
    let trip = new_trip(&w, vec![shop(a, "40"), shop(b, "30"), shop(c, "30")]);
    let purchase = buy(&w, trip, item, "20", "500", "400");
    set_today(&w.env, 2026, 10, 11);
    sell_own_on(&w, "11-10-2026", purchase, "15", w.akram, "600");
    let page = view(
        &w,
        AnalyticsView::G,
        AnalyticsFilters {
            partner_id: Some(a),
            group_by: Some(AnalyticsGroupBy::Day),
            ..filters()
        },
    );
    let trips = column(&page, "trips_result");
    let by_day: Vec<i64> = data_rows(&page)
        .iter()
        .map(|r| n(&r.cells[trips]))
        .collect();
    assert_eq!(by_day, vec![-4_160, 3_384]);
    let f = view(&w, AnalyticsView::F, by(a));
    assert_eq!(n(&page.totals[trips]), n(&f.totals[column(&f, "trips")]));
    assert_eq!(n(&page.totals[trips]), -776);
}
