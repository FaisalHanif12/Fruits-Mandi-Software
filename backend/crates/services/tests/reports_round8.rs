//! The reports of plan d5-late, round 8 (PRD 19.2, FR-12.27, FR-19.10, FR-19.11): R33 trip profit
//! or loss, R34 day summary, R15 vouchers, R23 receipts, R25 payments and R10 stock.
//!
//! The expected numbers are the PRD's: the trip of E2E-34, E2E-34b and E2E-41 (result 660, then
//! −1,940, parts −776 / −582 / −582; 5,280 and +2,640 each), the day of E2E-36 (2 بکری, خام
//! 41,400, commission 5,448, cash 50,000 → 58,484, buyers' dues 21,564) and the lists of the
//! worked day. Each report is also held against the screen or the report that shows the same
//! figure (the trip screen, R12, R20, R23, R29, the stock screen), so they cannot disagree.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
mod xlsx_support;

use common::*;
use mandi_reports::export::{ReportExportInput, export_report};
use mandi_reports::reports::{
    Cell, FilterKind, ReportFilters, ReportGroup, ReportId, ReportPage, Row, SourceKind, TripView,
    catalog,
};
use mandi_reports::reports::{ReportPrintInput, print_data};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{BikriSaveInput, StockListInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
use mandi_services::core::purchases::{PurchaseInput, PurchaseWriteOffInput};
use mandi_services::core::trips::{TripGetInput, TripInput};
use mandi_services::core::ventures::{EntryPartInput, MoneySource, VenturePartnerInput};
use mandi_services::core::vouchers::{VoucherCancelInput, VoucherType};
use mandi_services::{bikri, purchases, trips, vouchers};
use reports_world::*;
use support::World;

fn dates(from: &str, to: &str) -> ReportFilters {
    ReportFilters {
        from: from.into(),
        to: to.into(),
        ..ReportFilters::default()
    }
}

fn view(trip_view: TripView) -> ReportFilters {
    ReportFilters {
        trip_view: Some(trip_view),
        ..ReportFilters::default()
    }
}

fn qty(cell: &Cell) -> i64 {
    match cell {
        Cell::Number { value } => *value,
        Cell::Empty => 0,
        other => panic!("not a number: {other:?}"),
    }
}

fn label_id(cell: &Cell) -> &str {
    match cell {
        Cell::Label { id } => id,
        other => panic!("not a label: {other:?}"),
    }
}

// ---- The trip of E2E-34, E2E-34b and E2E-41 -----------------------------------------------------

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
    buy_on(w, "10-10-2026", trip, item, qty, rate, freight)
}

fn buy_on(
    w: &World,
    date: &str,
    trip: i64,
    item: i64,
    qty: &str,
    rate: &str,
    freight: &str,
) -> i64 {
    purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(trip),
            date: date.into(),
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

fn sell_own(w: &World, purchase: i64, qty: &str, buyer: i64, rate: &str) {
    let lot = mandi_services::core::bikri::LotInput {
        seller_account_id: Some(support::id_of(&w.env, "030119998")),
        purchase_id: Some(purchase),
        item_id: None,
        packing_id: None,
        adjust_advance: false,
        ..support::bare_lot(w, "own-1", w.rahim, w.banana, qty)
    };
    let draft = support::draft(vec![support::with_sales(
        lot,
        vec![support::sale("s-1", buyer, qty, rate)],
    )]);
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

/// E2E-34: a trip of the three partners at 40 / 30 / 30 paying from the shop's cash; 20 پیٹی
/// مالٹا at 500 with freight 400; an own lot sells 15 of them to اکرم at 600 on credit.
fn trip_34() -> (World, i64, i64) {
    let w = worked_day();
    let item = orange(&w);
    let [a, b, c] = w.partners;
    let trip = new_trip(&w, vec![shop(a, "40"), shop(b, "30"), shop(c, "30")]);
    let purchase = buy(&w, trip, item, "20", "500", "400");
    sell_own(&w, purchase, "15", w.akram, "600");
    (w, trip, purchase)
}

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

/// The eleven figures of a purchase row, in rupees and نگ.
fn figures(row: &Row, from: usize) -> Vec<i64> {
    row.cells[from..]
        .iter()
        .map(|c| match c {
            Cell::Money { paisa } => paisa.rupees(),
            Cell::Number { value } => *value,
            other => panic!("{other:?}"),
        })
        .collect()
}

#[test]
fn the_catalogue_has_the_new_reports_with_their_filters() {
    let entries = catalog();
    let find = |id| entries.iter().find(|e| e.id == id).unwrap();
    assert_eq!(find(ReportId::R10).group, ReportGroup::Sales);
    assert_eq!(
        find(ReportId::R10).filters,
        vec![FilterKind::DateRange, FilterKind::Seller, FilterKind::Item]
    );
    assert_eq!(
        find(ReportId::R15).filters,
        vec![
            FilterKind::DateRange,
            FilterKind::Party,
            FilterKind::VoucherType
        ]
    );
    for id in [ReportId::R23, ReportId::R25] {
        assert_eq!(find(id).group, ReportGroup::Collections);
        assert_eq!(
            find(id).filters,
            vec![FilterKind::DateRange, FilterKind::Party]
        );
    }
    assert_eq!(
        find(ReportId::R33).filters,
        vec![
            FilterKind::DateRange,
            FilterKind::Item,
            FilterKind::TripView
        ]
    );
    assert_eq!(find(ReportId::R34).filters, vec![FilterKind::DateRange]);
    assert_eq!(find(ReportId::R34).title_id, "report-r34-title");
}

#[test]
fn r33_e2e_34_the_trip_after_the_sale_of_15() {
    let (w, trip, _) = trip_34();
    let page = report(&w, ReportId::R33, ReportFilters::default(), 0, 0);
    // A heading for the trip, then its one purchase.
    assert_eq!(page.rows.len(), 2);
    let heading = &page.rows[0];
    let purchase = &page.rows[1];
    // bought 20 for 10,400; sold 15 for 8,460 after 540 commission; cost sold 7,800; nothing written
    // off; left 5 for 2,600; result 8,460 − 7,800 = 660.
    let want = vec![20, 10_400, 15, 8_460, 540, 7_800, 0, 0, 5, 2_600, 660];
    assert_eq!(figures(purchase, 4), want);
    assert_eq!(
        figures(heading, 4),
        want,
        "the trip heading carries the trip's totals"
    );
    assert_eq!(figures_of_cells(&page.totals, 4), want);
    assert_eq!(page.total_count, 2);
    assert!(matches!(
        &purchase.cells[1],
        Cell::Doc {
            number: Some(1),
            ..
        }
    ));
    // The trip screen reads the same result.
    let screen = trips::get(&w.env.ctx, &trip_input(trip)).unwrap();
    assert_eq!(money(&purchase.cells[14]), screen.row.result_paisa.rupees());
    assert_eq!(money(&purchase.cells[14]), 660);
}

fn trip_input(id: i64) -> TripGetInput {
    TripGetInput {
        id,
        entries_offset: 0,
        entries_limit: 0,
    }
}

/// The figures of R10's stock rows: the cells from `from`, without the text cell of the other
/// shop's name (index 12).
fn stock_figures(cells: &[Cell]) -> Vec<i64> {
    let mut figures = figures_of_cells(&cells[5..12], 0);
    figures.push(match &cells[13] {
        Cell::Number { value } => *value,
        other => panic!("{other:?}"),
    });
    figures
}

fn figures_of_cells(cells: &[Cell], from: usize) -> Vec<i64> {
    cells[from..]
        .iter()
        .map(|c| match c {
            Cell::Money { paisa } => paisa.rupees(),
            Cell::Number { value } => *value,
            other => panic!("{other:?}"),
        })
        .collect()
}

#[test]
fn r33_e2e_34b_the_last_5_spoil_and_the_result_is_a_loss_of_1940() {
    let (w, trip, purchase) = trip_34();
    write_off_5(&w, purchase);
    let page = report(&w, ReportId::R33, ReportFilters::default(), 0, 0);
    let row = &page.rows[1];
    // written off 5 for 2,600; left 0; result 8,460 − 7,800 − 2,600 = −1,940.
    assert_eq!(
        figures(row, 4),
        vec![20, 10_400, 15, 8_460, 540, 7_800, 5, 2_600, 0, 0, -1_940]
    );
    let screen = trips::get(&w.env.ctx, &trip_input(trip)).unwrap();
    assert_eq!(screen.row.result_paisa.rupees(), -1_940);
    assert_eq!(money(&row.cells[14]), screen.row.result_paisa.rupees());
    assert_eq!(money(&page.totals[14]), -1_940);

    // The partners: 40 / 30 / 30, money in 4,160 / 3,120 / 3,120, back 3,384 / 2,538 / 2,538,
    // parts −776 / −582 / −582 (E2E-34b).
    let partners = report(&w, ReportId::R33, view(TripView::Partner), 0, 0);
    let rows: Vec<&Row> = partners
        .rows
        .iter()
        .filter(|r| r.kind == mandi_reports::reports::RowKind::Normal)
        .collect();
    assert_eq!(rows.len(), 3);
    let line = |r: &Row| {
        (
            matches!(&r.cells[2], Cell::Percent { hundredths } if *hundredths > 0),
            money(&r.cells[3]),
            money(&r.cells[4]),
            money(&r.cells[5]),
            money(&r.cells[6]),
        )
    };
    assert_eq!(line(rows[0]), (true, 4_160, 0, 3_384, -776));
    assert_eq!(line(rows[1]), (true, 3_120, 0, 2_538, -582));
    assert_eq!(line(rows[2]), (true, 3_120, 0, 2_538, -582));
    assert!(matches!(
        &rows[0].cells[2],
        Cell::Percent { hundredths: 4_000 }
    ));
    // The trip heading adds the partners up, and the cost left is 0, so the parts add up to the
    // result: −776 − 582 − 582 = −1,940.
    let heading = &partners.rows[0];
    assert_eq!(
        (
            money(&heading.cells[3]),
            money(&heading.cells[5]),
            money(&heading.cells[6]),
            money(&heading.cells[7])
        ),
        (10_400, 8_460, -1_940, 0)
    );
    assert_eq!(money(&partners.totals[6]), -1_940);
    // Each partner's part is his stored money back less his money in, as the trip screen reads.
    let nets: Vec<i64> = screen
        .partner_results
        .iter()
        .map(|p| p.net_paisa.rupees())
        .collect();
    assert_eq!(nets, vec![-776, -582, -582]);
}

#[test]
fn r33_the_parts_of_a_trip_plus_its_cost_left_are_its_result() {
    // E2E-34 before the write-off: the parts are −776 / −582 / −582 = −1,940, the cost left is
    // 2,600 and the result is 660 = −1,940 + 2,600.
    let (w, _, _) = trip_34();
    let partners = report(&w, ReportId::R33, view(TripView::Partner), 0, 0);
    assert_eq!(money(&partners.totals[6]), -1_940);
    assert_eq!(money(&partners.totals[7]), 2_600);
    let result = report(&w, ReportId::R33, ReportFilters::default(), 0, 0);
    assert_eq!(
        money(&partners.totals[6]) + money(&partners.totals[7]),
        money(&result.totals[14])
    );
}

#[test]
fn r33_partner_layout_takes_the_cost_left_of_the_whole_trip_whatever_the_filter() {
    // E2E-34 plus a second purchase on 12-10 of 10 نگ at 560 (5,600), nothing sold from it. The
    // partners' money is for the whole trip, so the cost left must be too: the filter picks the
    // trip by the purchase date, it does not cut the trip's cost left (money agent, round 8).
    let (w, trip, _) = trip_34();
    let item = orange(&w);
    set_today(&w.env, 2026, 10, 12);
    buy_on(&w, "12-10-2026", trip, item, "10", "560", "");

    let partner_view = |from: &str, to: &str| ReportFilters {
        from: from.into(),
        to: to.into(),
        trip_view: Some(TripView::Partner),
        ..ReportFilters::default()
    };
    let narrow = report(
        &w,
        ReportId::R33,
        partner_view("10-10-2026", "10-10-2026"),
        0,
        0,
    );
    let wide = report(&w, ReportId::R33, view(TripView::Partner), 0, 0);
    // Money in 10,400 + 5,600, money back 8,460: parts −7,540. Cost left 2,600 + 5,600 = 8,200.
    for page in [&narrow, &wide] {
        assert_eq!(money(&page.totals[6]), -7_540);
        assert_eq!(money(&page.totals[7]), 8_200);
        assert_eq!(money(&page.totals[6]) + money(&page.totals[7]), 660);
    }
    // The result of the trip on the default layout is the same 660.
    let result = report(&w, ReportId::R33, ReportFilters::default(), 0, 0);
    assert_eq!(money(&result.totals[14]), 660);
}

#[test]
fn r33_e2e_41_a_pocket_part_a_truck_cost_and_a_trip_sold_out() {
    let w = worked_day();
    let item = orange(&w);
    let [a, b, c] = w.partners;
    let trip = new_trip(&w, vec![shop(a, "50"), pocket(b, "50"), shop(c, "0")]);
    let purchase = buy(&w, trip, item, "80", "1,200", "");
    trips::cost_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::trips::TripCostInput {
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
    let page = report(&w, ReportId::R33, ReportFilters::default(), 0, 0);
    // bought 80 for 100,000 (96,000 + the truck 4,000); sold 80 for 105,280 after 6,720; cost sold
    // 100,000; left 0; result 5,280 (E2E-41).
    assert_eq!(
        figures(&page.rows[1], 4),
        vec![80, 100_000, 80, 105_280, 6_720, 100_000, 0, 0, 0, 0, 5_280]
    );
    let partners = report(&w, ReportId::R33, view(TripView::Partner), 0, 0);
    let lines: Vec<(i64, i64, i64, i64)> = partners
        .rows
        .iter()
        .filter(|r| r.kind == mandi_reports::reports::RowKind::Normal)
        .map(|r| {
            (
                money(&r.cells[3]),
                money(&r.cells[4]),
                money(&r.cells[5]),
                money(&r.cells[6]),
            )
        })
        .collect();
    // الف paid 50,000 from the shop, ب paid 50,000 from his pocket; each got back 52,640; +2,640.
    assert_eq!(
        lines,
        vec![(50_000, 0, 52_640, 2_640), (0, 50_000, 52_640, 2_640)]
    );
    assert_eq!(money(&partners.totals[6]), 5_280);
}

#[test]
fn r33_by_item_by_day_and_by_month_group_the_same_purchases() {
    let (w, _, _) = trip_34();
    let base = report(&w, ReportId::R33, ReportFilters::default(), 0, 0);
    for (trip_view, kind) in [
        (TripView::Item, "names"),
        (TripView::Day, "date"),
        (TripView::Month, "month"),
    ] {
        let page = report(&w, ReportId::R33, view(trip_view), 0, 0);
        assert_eq!(page.rows.len(), 1, "{trip_view:?}");
        assert_eq!(
            figures(&page.rows[0], 1),
            figures_of_cells(&base.totals, 4),
            "{trip_view:?}"
        );
        assert_eq!(
            figures_of_cells(&page.totals, 1),
            figures_of_cells(&base.totals, 4)
        );
        match (&page.rows[0].cells[0], kind) {
            (Cell::Names { names }, "names") => assert_eq!(names.en, "Orange"),
            (Cell::Date { iso }, "date") => assert_eq!(iso, "2026-10-10"),
            (
                Cell::Month {
                    year: 2026,
                    month: 10,
                },
                "month",
            ) => {}
            (other, _) => panic!("{trip_view:?}: {other:?}"),
        }
        assert_eq!(page.filters.trip_view, Some(trip_view));
    }
}

#[test]
fn r33_filters_by_day_and_item_and_leaves_out_a_cancelled_purchase() {
    let (w, trip, _) = trip_34();
    let item2 = support::add_item(&w.env, &w.owner, "کینو", "Kinnow", "fruit", "unit");
    let second = buy(&w, trip, item2, "10", "300", "");
    // Another day: nothing; this day: the trip.
    assert!(
        report(&w, ReportId::R33, dates("11-10-2026", "12-10-2026"), 0, 0)
            .rows
            .is_empty()
    );
    let both = report(&w, ReportId::R33, dates("10-10-2026", "10-10-2026"), 0, 0);
    assert_eq!(both.rows.len(), 3, "a heading and two purchases");
    // The item filter keeps one purchase.
    let only = report(
        &w,
        ReportId::R33,
        ReportFilters {
            item_id: Some(item2),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(only.rows.len(), 2);
    assert_eq!(figures(&only.rows[1], 4)[0], 10);
    assert_eq!(
        only.filters.item.as_ref().map(|n| n.en.as_str()),
        Some("Kinnow")
    );
    // A cancelled purchase has no goods and leaves the report.
    let row = purchases::get(
        &w.env.ctx,
        &mandi_services::core::purchases::PurchaseGetInput { id: second },
    )
    .unwrap();
    purchases::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::purchases::PurchaseCancelInput {
            id: second,
            version: row.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(
        report(&w, ReportId::R33, ReportFilters::default(), 0, 0)
            .rows
            .len(),
        2
    );
}

#[test]
fn r33_a_page_keeps_the_count_and_the_totals_of_the_whole_filter() {
    let (w, _, _) = trip_34();
    let page = report(&w, ReportId::R33, ReportFilters::default(), 1, 1);
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.total_count, 2);
    assert_eq!(money(&page.totals[14]), 660);
}

#[test]
fn the_new_filters_are_refused_by_the_reports_that_do_not_take_them() {
    let w = full_day();
    assert_eq!(
        rejected(&w, ReportId::R30, view(TripView::Item), 0, 0),
        vec![(ErrorCode::ErrOutOfRange, "filters.trip_view".to_owned())]
    );
    assert_eq!(
        rejected(
            &w,
            ReportId::R23,
            ReportFilters {
                voucher_type: Some(VoucherType::Payment),
                ..ReportFilters::default()
            },
            0,
            0
        ),
        vec![(ErrorCode::ErrOutOfRange, "filters.voucher_type".to_owned())]
    );
}

// ---- R34 ---------------------------------------------------------------------------------------

/// The row of a label on the page.
fn find<'a>(page: &'a ReportPage, id: &str) -> &'a Row {
    page.rows
        .iter()
        .find(|r| matches!(&r.cells[1], Cell::Label { id: x } if x == id))
        .unwrap_or_else(|| panic!("no row {id}"))
}

fn amount(page: &ReportPage, id: &str) -> i64 {
    money(&find(page, id).cells[3])
}

fn count(page: &ReportPage, id: &str) -> i64 {
    qty(&find(page, id).cells[2])
}

#[test]
fn r34_e2e_36_the_worked_day() {
    let w = full_day();
    let page = report(&w, ReportId::R34, dates("10-10-2026", "10-10-2026"), 0, 0);
    assert_eq!(page.title_id, "report-r34-title");
    assert_eq!(
        (
            count(&page, "report-r34-bikri"),
            amount(&page, "report-r34-bikri")
        ),
        (2, 41_400)
    );
    assert_eq!(amount(&page, "report-r34-commission"), 5_448);
    // Cash: opening 50,000, in 12,800 + 10,000, out 5,000 + 4,316 + 3,000 + 2,000, closing 58,484.
    assert_eq!(amount(&page, "report-r34-cash-opening"), 50_000);
    assert_eq!(amount(&page, "report-r34-cash-in"), 22_800);
    assert_eq!(amount(&page, "report-r34-cash-out"), 14_316);
    assert_eq!(amount(&page, "report-r34-cash-closing"), 58_484);
    assert_eq!(
        (
            count(&page, "report-r34-receipts"),
            amount(&page, "report-r34-receipts")
        ),
        (1, 10_000)
    );
    assert_eq!(
        (
            count(&page, "report-r34-payments"),
            amount(&page, "report-r34-payments")
        ),
        (1, 4_316)
    );
    assert_eq!(count(&page, "report-r34-advances"), 0);
    assert_eq!(amount(&page, "report-r34-expenses"), 3_000);
    assert_eq!(amount(&page, "report-r34-partner-taken"), 2_000);
    assert_eq!(amount(&page, "report-r34-partner-returned"), 0);
    assert_eq!(amount(&page, "report-r34-capital"), 0);
    assert_eq!(amount(&page, "report-r34-money-lent"), 0);
    assert_eq!(amount(&page, "report-r34-buyers-dues"), 21_564);
    assert_eq!(
        (
            count(&page, "report-r34-open-lots"),
            count(&page, "report-r34-open-qty")
        ),
        (0, 0)
    );
    // Each partner's profit share of the day: 996 / 746 / 746 (income less expense, C12).
    let shares: Vec<i64> = page
        .rows
        .iter()
        .filter(|r| matches!(&r.cells[1], Cell::Names { .. }))
        .map(|r| money(&r.cells[3]))
        .collect();
    assert_eq!(shares, vec![996, 746, 746]);
    assert_eq!(page.filters.from.as_deref(), Some("2026-10-10"));
}

#[test]
fn r34_ties_to_r12_r20_r23_r29_and_the_sales_of_the_day() {
    let w = full_day();
    let day = || dates("10-10-2026", "10-10-2026");
    let page = report(&w, ReportId::R34, day(), 0, 0);
    let cash = report(&w, ReportId::R12, day(), 0, 0);
    let (opening, closing) = (
        balance(&cash.summary[0].cell).1,
        balance(&cash.summary[1].cell).1,
    );
    assert_eq!(amount(&page, "report-r34-cash-opening"), opening);
    assert_eq!(amount(&page, "report-r34-cash-closing"), closing);
    assert_eq!(amount(&page, "report-r34-cash-in"), money(&cash.totals[4]));
    assert_eq!(amount(&page, "report-r34-cash-out"), money(&cash.totals[5]));
    let receipts = report(&w, ReportId::R23, day(), 0, 0);
    assert_eq!(
        amount(&page, "report-r34-receipts"),
        money(&receipts.totals[3])
    );
    let payments = report(&w, ReportId::R25, day(), 0, 0);
    assert_eq!(
        amount(&page, "report-r34-payments"),
        money(&payments.totals[3])
    );
    let dues = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    let dues_total = dues.totals.iter().rev().find_map(|c| match c {
        Cell::Balance { paisa, .. } | Cell::Money { paisa } => Some(paisa.rupees()),
        _ => None,
    });
    assert_eq!(Some(amount(&page, "report-r34-buyers-dues")), dues_total);
    let expenses = report(&w, ReportId::R29, day(), 0, 0);
    assert_eq!(
        amount(&page, "report-r34-expenses"),
        money(&expenses.totals[2])
    );
    // The sales and the commission are those of the sales facts, which the day close reads too.
    let (gross, commission): (i64, i64) = raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(sign * seller_amount_paisa), 0) / 100,
                    COALESCE(SUM(sign * (seller_comm_paisa + buyer_comm_paisa + laga_paisa)), 0) / 100
             FROM analytics_facts WHERE entry_date = '2026-10-10'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(amount(&page, "report-r34-bikri"), gross);
    assert_eq!(amount(&page, "report-r34-commission"), commission);
}

#[test]
fn r34_lists_each_day_with_its_own_opening_and_a_quiet_day_when_asked_for_alone() {
    let w = full_day();
    set_today(&w.env, 2026, 11, 11);
    let mut later = receipt(w.akram, "1,000");
    later.date = "11-11-2026".into();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &later).unwrap();
    let page = report(&w, ReportId::R34, dates("10-10-2026", "11-11-2026"), 0, 0);
    // Only the days with something posted: 10-10 and 11-11.
    let days: std::collections::BTreeSet<String> = page
        .rows
        .iter()
        .map(|r| match &r.cells[0] {
            Cell::Date { iso } => iso.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        days.into_iter().collect::<Vec<_>>(),
        vec!["2026-10-10".to_owned(), "2026-11-11".to_owned()]
    );
    let on = |day: &str, id: &str| -> i64 {
        let row = page
            .rows
            .iter()
            .find(|r| {
                matches!(&r.cells[0], Cell::Date { iso } if iso == day)
                    && matches!(&r.cells[1], Cell::Label { id: x } if x == id)
            })
            .unwrap();
        money(&row.cells[3])
    };
    assert_eq!(on("2026-11-11", "report-r34-cash-opening"), 58_484);
    assert_eq!(on("2026-11-11", "report-r34-cash-in"), 1_000);
    assert_eq!(on("2026-11-11", "report-r34-cash-closing"), 59_484);
    assert_eq!(on("2026-11-11", "report-r34-buyers-dues"), 20_564);
    assert_eq!(on("2026-11-11", "report-r34-expenses"), 0);
    // A day on which nothing happened is shown when it is asked for alone.
    set_today(&w.env, 2026, 11, 12);
    let quiet = report(&w, ReportId::R34, dates("12-11-2026", "12-11-2026"), 0, 0);
    assert_eq!(amount(&quiet, "report-r34-cash-opening"), 59_484);
    assert_eq!(amount(&quiet, "report-r34-cash-closing"), 59_484);
    assert_eq!(amount(&quiet, "report-r34-buyers-dues"), 20_564);
    assert_eq!(count(&quiet, "report-r34-bikri"), 0);
    // With no dates the report is for today.
    let today = report(&w, ReportId::R34, ReportFilters::default(), 0, 0);
    assert_eq!(today.filters.from.as_deref(), Some("2026-11-12"));
    assert_eq!(today.filters.to.as_deref(), Some("2026-11-12"));
}

#[test]
fn r34_leaves_out_the_entry_of_a_month_close_and_counts_a_later_sale_on_its_own_day() {
    let w = full_day();
    set_today(&w.env, 2026, 11, 2);
    mandi_services::day::close(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::DayCloseInput {
            up_to: "31-10-2026".into(),
        },
    )
    .unwrap();
    mandi_services::month::close(
        &w.env.ctx,
        &w.owner,
        &mandi_services::month::MonthInput {
            month: "2026-10".into(),
        },
    )
    .unwrap();
    // The close entry is dated today. The day summary of today shows nothing of it.
    let today = report(&w, ReportId::R34, dates("02-11-2026", "02-11-2026"), 0, 0);
    assert_eq!(amount(&today, "report-r34-expenses"), 0);
    assert_eq!(amount(&today, "report-r34-cash-opening"), 58_484);
    assert_eq!(amount(&today, "report-r34-cash-closing"), 58_484);
    assert!(
        today
            .rows
            .iter()
            .all(|r| !matches!(&r.cells[1], Cell::Names { .. })),
        "a month close writes no share rows"
    );
    // October still reads as it was.
    let october = report(&w, ReportId::R34, dates("10-10-2026", "10-10-2026"), 0, 0);
    assert_eq!(amount(&october, "report-r34-expenses"), 3_000);
    assert_eq!(amount(&october, "report-r34-commission"), 5_448);
}

#[test]
fn r34_lots_in_stock_are_the_stock_of_that_day_even_after_a_later_sale() {
    // E2E-16: 45 of 50 sold on 10-10-26, the last 5 sold on 11-10-26 (E2E-16b). R34 made after the
    // later sale still shows 5 نگ in 1 lot on 10-10 (worked from the stock moves dated up to the
    // day, as the day close does), and 0 on 11-10.
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    save(&w, &draft);
    set_today(&w.env, 2026, 10, 11);
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    let mut line = support::sale("s-3", w.arshad, "5", "560");
    line.sale_id = None;
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::LaterSaleSaveInput {
            lot_id: stock.rows[0].lot_id,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let both = report(&w, ReportId::R34, dates("10-10-2026", "11-10-2026"), 0, 0);
    let open: Vec<(String, i64)> = both
        .rows
        .iter()
        .filter_map(|r| match (&r.cells[0], &r.cells[1], &r.cells[2]) {
            (Cell::Date { iso }, Cell::Label { id }, Cell::Number { value })
                if id == "report-r34-open-lots" || id == "report-r34-open-qty" =>
            {
                Some((format!("{iso} {id}"), *value))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        open,
        vec![
            ("2026-10-10 report-r34-open-lots".to_owned(), 1),
            ("2026-10-10 report-r34-open-qty".to_owned(), 5),
            ("2026-10-11 report-r34-open-lots".to_owned(), 0),
            ("2026-10-11 report-r34-open-qty".to_owned(), 0),
        ]
    );
}

#[test]
fn r34_shows_a_trip_bought_and_sold() {
    let (w, _, _) = trip_34();
    let page = report(&w, ReportId::R34, dates("10-10-2026", "10-10-2026"), 0, 0);
    // Bought 20 نگ for 10,400; sold for 8,460 after commission.
    assert_eq!(
        (
            count(&page, "report-r34-trips-bought"),
            amount(&page, "report-r34-trips-bought")
        ),
        (20, 10_400)
    );
    assert_eq!(amount(&page, "report-r34-trips-sold"), 8_460);
}

// ---- R15, R23, R25 -----------------------------------------------------------------------------

fn general_of_1000(w: &World) {
    let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
        account_id: Some(account),
        person_key: None,
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    };
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &GeneralInput {
            date: "10-10-2026".into(),
            narration: Some("check".into()),
            lines: vec![
                line(support::id_of(&w.env, "04010005"), "1,000", ""),
                line(support::id_of(&w.env, "01010001"), "", "1,000"),
            ],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

#[test]
fn r23_and_r25_are_the_receipts_and_the_payments_of_the_worked_day() {
    let w = full_day();
    let day = || dates("10-10-2026", "10-10-2026");
    let receipts = report(&w, ReportId::R23, day(), 0, 0);
    assert_eq!(receipts.rows.len(), 1);
    let row = &receipts.rows[0];
    assert_eq!(code_of(&row.cells[2]), "0020");
    assert_eq!(money(&row.cells[3]), 10_000);
    assert!(
        matches!(&row.cells[1], Cell::Doc { number: Some(1), type_id } if type_id == "report-doc-receipt")
    );
    assert_eq!(money(&receipts.totals[3]), 10_000);
    assert_eq!(
        receipts
            .columns
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>(),
        vec![
            "date",
            "doc",
            "account",
            "amount",
            "collector",
            "narration",
            "status"
        ]
    );
    assert_eq!(receipts.title_id, "report-r23-title");

    let payments = report(&w, ReportId::R25, day(), 0, 0);
    assert_eq!(payments.rows.len(), 1);
    assert_eq!(code_of(&payments.rows[0].cells[2]), "0010");
    assert_eq!(money(&payments.totals[3]), 4_316);
    // There is no mode, bank or cheque column.
    assert_eq!(
        payments
            .columns
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>(),
        vec!["date", "doc", "account", "amount", "narration", "status"]
    );
}

#[test]
fn r15_lists_every_kind_by_day_then_kind_and_leaves_a_general_out_of_the_money() {
    let w = full_day();
    general_of_1000(&w);
    let all = report(&w, ReportId::R15, ReportFilters::default(), 0, 0);
    // The advance of 01-09-26, then on 10-10-26 the جنرل, the payment and the receipt.
    let docs: Vec<String> = all
        .rows
        .iter()
        .map(|r| match &r.cells[1] {
            Cell::Doc { type_id, number } => format!("{type_id}:{}", number.unwrap_or(0)),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        docs,
        vec![
            "report-doc-advance:1",
            "report-doc-general:1",
            "report-doc-payment:1",
            "report-doc-receipt:1"
        ]
    );
    assert_eq!(all.total_count, 4);
    // The kinds are mixed, so there is no total amount. Each kind has its own line above the
    // table: the جنرل of 1,000 is its debit total, not money that moved, so it is never added to
    // the others.
    assert!(matches!(&all.totals[3], Cell::Empty));
    let lines: Vec<(String, i64)> = all
        .summary
        .iter()
        .map(|l| (l.label_id.clone(), money(&l.cell)))
        .collect();
    assert_eq!(
        lines,
        vec![
            ("report-doc-advance".to_owned(), 15_000),
            ("report-doc-general".to_owned(), 1_000),
            ("report-doc-payment".to_owned(), 4_316),
            ("report-doc-receipt".to_owned(), 10_000),
        ]
    );
    // Picking جنرل adds its own amount.
    let general = report(
        &w,
        ReportId::R15,
        ReportFilters {
            voucher_type: Some(VoucherType::General),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(general.rows.len(), 1);
    assert_eq!(money(&general.totals[3]), 1_000);
    assert!(general.summary.is_empty());
    assert_eq!(general.filters.voucher_type, Some(VoucherType::General));
    // Picking payments keeps one.
    let payment = report(
        &w,
        ReportId::R15,
        ReportFilters {
            voucher_type: Some(VoucherType::Payment),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!((payment.rows.len(), money(&payment.totals[3])), (1, 4_316));
    // A page keeps the whole filter's count and totals.
    let second = report(&w, ReportId::R15, ReportFilters::default(), 1, 1);
    assert_eq!(
        (
            second.rows.len(),
            second.total_count,
            second.summary.len(),
            matches!(&second.totals[3], Cell::Empty)
        ),
        (1, 4, 4, true)
    );
}

#[test]
fn a_cancelled_voucher_stays_in_the_list_and_leaves_the_total() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 10);
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.arshad, "500")).unwrap();
    let before = report(&w, ReportId::R23, ReportFilters::default(), 0, 0);
    assert_eq!((before.rows.len(), money(&before.totals[3])), (2, 10_500));
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: saved.voucher_id,
            version: vouchers::get(&w.env.ctx, saved.voucher_id).unwrap().version,
            reason: "غلط".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let after = report(&w, ReportId::R23, ReportFilters::default(), 0, 0);
    assert_eq!(after.rows.len(), 2, "the cancelled receipt is listed");
    assert_eq!(
        money(&after.totals[3]),
        10_000,
        "but it is not in the money"
    );
    let cancelled: Vec<&Row> = after
        .rows
        .iter()
        .filter(|r| matches!(&r.cells[6], Cell::Label { .. }))
        .collect();
    assert_eq!(cancelled.len(), 1);
    assert_eq!(label_id(&cancelled[0].cells[6]), "report-status-cancelled");
    // R34 counts only the saved receipt.
    let day = report(&w, ReportId::R34, dates("10-10-2026", "10-10-2026"), 0, 0);
    assert_eq!(
        (
            count(&day, "report-r34-receipts"),
            amount(&day, "report-r34-receipts")
        ),
        (1, 10_000)
    );
}

#[test]
fn the_voucher_lists_filter_by_party_and_day() {
    let w = full_day();
    let by_akram = report(&w, ReportId::R23, party(w.akram), 0, 0);
    assert_eq!(by_akram.rows.len(), 1);
    assert_eq!(
        by_akram.filters.party.as_ref().map(|p| p.code.clone()),
        Some("0020".to_owned())
    );
    assert!(
        report(&w, ReportId::R23, party(w.arshad), 0, 0)
            .rows
            .is_empty()
    );
    assert!(
        report(&w, ReportId::R25, dates("11-10-2026", "12-10-2026"), 0, 0)
            .rows
            .is_empty()
    );
}

// ---- R10 ---------------------------------------------------------------------------------------

#[test]
fn r10_lists_the_lots_with_goods_left_like_the_stock_screen() {
    let w = worked_day();
    // 50 received, 20 sold to اکرم: 30 left.
    let draft = support::draft(vec![support::with_sales(
        support::bare_lot(&w, "k-1", w.rahim, w.banana, "50"),
        vec![support::sale("s-1", w.akram, "20", "600")],
    )]);
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let page = report(&w, ReportId::R10, ReportFilters::default(), 0, 0);
    assert_eq!(page.rows.len(), 1);
    let row = &page.rows[0];
    assert_eq!(code_of(&row.cells[2]), "0010");
    // received, sold, written off, taken as fruit, balance, left for our shop, left for another
    // shop (a lot has only our shop's part, FR-12.41), days (C16).
    assert_eq!(stock_figures(&row.cells), vec![50, 20, 0, 0, 30, 30, 0, 0]);
    assert_eq!(
        figures_of_cells(&page.totals[..12], 5),
        [50, 20, 0, 0, 30, 30, 0]
    );
    // The screen shows the same lot with the same balance and days.
    let screen = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert_eq!(screen.rows.len(), 1);
    assert_eq!(screen.rows[0].balance, 30);
    assert_eq!(screen.rows[0].days_waiting, 0);
    // The seller filter and the item filter.
    let other = report(
        &w,
        ReportId::R10,
        ReportFilters {
            seller_id: Some(w.karim),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert!(other.rows.is_empty());
    // Two days later the goods have waited 2 days.
    set_today(&w.env, 2026, 10, 12);
    let later = report(&w, ReportId::R10, ReportFilters::default(), 0, 0);
    // The days waiting are the last column (after the balance and its two parts).
    assert_eq!(qty(&later.rows[0].cells[13]), 2);
}

#[test]
fn r10_also_lists_the_garden_arrivals_with_goods_left_like_the_stock_screen() {
    let w = worked_day();
    // 100 arrive on 10-10-26 to sell on 10-10-26; 60 are sold in a linked lot.
    let (garden_account, arrival) = garden_with_arrival(&w, "100", "10-10-2026");
    let lot = support::with_sales(
        mandi_services::core::bikri::LotInput {
            arrival_id: Some(arrival.id),
            ..support::bare_lot(&w, "g-1", garden_account, w.banana, "60")
        },
        vec![support::sale("s-1", w.akram, "60", "600")],
    );
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::draft(vec![lot]),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let page = report(&w, ReportId::R10, ReportFilters::default(), 0, 0);
    // The garden lot is sold in full, so the only row is the arrival with its 40 left.
    assert_eq!(page.rows.len(), 1);
    let row = &page.rows[0];
    assert!(
        matches!(&row.cells[1], Cell::Doc { number: Some(1), type_id } if type_id == "report-doc-garden-arrival")
    );
    assert_eq!(
        code_of(&row.cells[2]),
        "0080",
        "the seller is the garden's account"
    );
    // received, sold, written off, taken as fruit, balance, left for our shop, left for another
    // shop, days (from the day to sell).
    assert_eq!(stock_figures(&row.cells), vec![100, 60, 0, 0, 40, 40, 0, 0]);
    assert_eq!(
        figures_of_cells(&page.totals[..12], 5),
        [100, 60, 0, 0, 40, 40, 0]
    );
    let source = row.source.expect("a row opens its record");
    assert_eq!((source.kind, source.id), (SourceKind::Arrival, arrival.id));
    // The screen shows the same arrival with the same balance.
    let screen = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert_eq!(screen.garden_rows.len(), 1);
    assert_eq!(screen.garden_rows[0].balance, 40);
    assert_eq!(screen.garden_rows[0].arrival_id, arrival.id);
    // The seller filter keeps it for the garden's account and drops it for another seller; the
    // item filter keeps it for کیلا and drops it for another item.
    let by = |seller: Option<i64>, item: Option<i64>| {
        report(
            &w,
            ReportId::R10,
            ReportFilters {
                seller_id: seller,
                item_id: item,
                ..ReportFilters::default()
            },
            0,
            0,
        )
        .rows
        .len()
    };
    assert_eq!(by(Some(garden_account), None), 1);
    assert_eq!(by(Some(w.karim), None), 0);
    assert_eq!(by(None, Some(w.banana)), 1);
    assert_eq!(by(None, Some(w.apple)), 0);
    // Two days after the day to sell, the goods have waited 2 days; before it, 0.
    set_today(&w.env, 2026, 10, 12);
    let later = report(&w, ReportId::R10, ReportFilters::default(), 0, 0);
    assert_eq!(qty(&later.rows[0].cells[13]), 2);
    // Written off for good, the arrival leaves the report.
    mandi_services::gardens::arrivals::write_off(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::arrivals::ArrivalWriteOffInput {
            arrival_id: arrival.id,
            qty: "40".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
    assert!(
        report(&w, ReportId::R10, ReportFilters::default(), 0, 0)
            .rows
            .is_empty()
    );
}

#[test]
fn r10_counts_the_days_of_an_arrival_from_its_day_to_sell_and_prints_its_row() {
    let w = worked_day();
    // Arrived on 10-10-26 to sell on 12-10-26: on 10-10-26 it has waited 0 days, not negative.
    let (_, arrival) = garden_with_arrival(&w, "30", "12-10-2026");
    let page = report(&w, ReportId::R10, ReportFilters::default(), 0, 0);
    assert_eq!(
        stock_figures(&page.rows[0].cells),
        vec![30, 0, 0, 0, 30, 30, 0, 0]
    );
    set_today(&w.env, 2026, 10, 15);
    let page = report(&w, ReportId::R10, ReportFilters::default(), 0, 0);
    assert_eq!(
        qty(&page.rows[0].cells[13]),
        3,
        "from 12-10-26, not from 10-10-26"
    );

    // The print data has the same row, with the document label ID the frontend writes in words.
    let printed = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id: ReportId::R10,
            filters: ReportFilters::default(),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(printed.rows.len(), 1);
    let json = serde_json::to_value(&printed.rows[0]).unwrap().to_string();
    assert!(json.contains("report-doc-garden-arrival"), "{json}");
    assert_eq!(printed.rows[0].cells.len(), page.rows[0].cells.len());
    let _ = arrival;
}

#[test]
fn r10_splits_the_goods_left_into_our_shops_part_and_the_part_for_another_shop() {
    // PRD FR-12.41, plan 35 R6: 20 bought, 15 sold, 5 left. 3 of the 20 are for another shop, so
    // the 5 left are 2 for our shop and 3 for another shop. The two add up to the balance.
    let (w, _, purchase) = trip_34();
    let row = purchases::get(
        &w.env.ctx,
        &mandi_services::core::purchases::PurchaseGetInput { id: purchase },
    )
    .unwrap();
    purchases::change_split(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::split::SplitInput {
            id: purchase,
            version: row.version,
            other_qty: "3".into(),
            shop_name: Some("Haji Shop".into()),
        },
    )
    .unwrap();
    let page = report(&w, ReportId::R10, ReportFilters::default(), 0, 0);
    assert_eq!(page.rows.len(), 1);
    assert_eq!(
        stock_figures(&page.rows[0].cells),
        vec![20, 15, 0, 0, 5, 2, 3, 0]
    );
    assert_eq!(
        figures_of_cells(&page.totals[..12], 5),
        [20, 15, 0, 0, 5, 2, 3]
    );
    let titles: Vec<&str> = page.columns.iter().map(|c| c.title_id.as_str()).collect();
    assert_eq!(
        &titles[10..13],
        [
            "report-col-our-left",
            "report-col-other-left",
            "report-col-shop-name"
        ]
    );
    // The other shop's name, as typed (FR-12.41); a lot has none.
    assert_eq!(
        page.rows[0].cells[12],
        Cell::Text {
            text: "Haji Shop".into()
        }
    );
}

#[test]
fn r10_also_lists_the_stock_of_a_trip_marked_with_the_own_goods_account() {
    let (w, _, _) = trip_34();
    let page = report(&w, ReportId::R10, ReportFilters::default(), 0, 0);
    // 20 bought, 15 sold, 5 left, seller اپنا مال, the document is the purchase خ-1.
    assert_eq!(page.rows.len(), 1);
    let row = &page.rows[0];
    assert_eq!(code_of(&row.cells[2]), "030119998");
    assert!(
        matches!(&row.cells[1], Cell::Doc { number: Some(1), type_id } if type_id == "report-doc-purchase")
    );
    assert_eq!(stock_figures(&row.cells), vec![20, 15, 0, 0, 5, 5, 0, 0]);
    let screen = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert_eq!(screen.own_rows.len(), 1);
    assert_eq!(screen.own_rows[0].balance, 5);
    // After the write-off nothing is left, and the report is empty.
    let purchase_id = screen.own_rows[0].purchase_id;
    write_off_5(&w, purchase_id);
    assert!(
        report(&w, ReportId::R10, ReportFilters::default(), 0, 0)
            .rows
            .is_empty()
    );
}

// ---- Excel and print ---------------------------------------------------------------------------

#[test]
fn the_trip_report_in_every_layout_and_the_day_summary_export_with_the_numbers_of_the_page() {
    let (w, _, purchase) = trip_34();
    write_off_5(&w, purchase);
    let dir = tempfile::tempdir().unwrap();
    let export = |id: ReportId, filters: ReportFilters, language: Language, name: &str| {
        let path = dir.path().join(name);
        let done = export_report(
            w.env.ctx.db(),
            &ReportExportInput {
                id,
                filters,
                language,
                path: path.display().to_string(),
            },
            &mut |_| {},
        )
        .unwrap();
        (done, xlsx_support::read(&path))
    };
    for (i, trip_view) in [
        TripView::Trip,
        TripView::Item,
        TripView::Day,
        TripView::Month,
        TripView::Partner,
    ]
    .into_iter()
    .enumerate()
    {
        let page = report(&w, ReportId::R33, view(trip_view), 0, 0);
        let (done, sheet) = export(
            ReportId::R33,
            view(trip_view),
            Language::En,
            &format!("r33-{i}.xlsx"),
        );
        assert_eq!(
            done.rows,
            u64::try_from(page.rows.len()).unwrap(),
            "{trip_view:?}"
        );
        // The loss of the trip, or each partner's part, is in the sheet as a signed number.
        let numbers: Vec<String> = (0..=sheet.last_row())
            .flat_map(|r| (0..16).map(move |c| (r, c)))
            .filter_map(|(r, c)| match sheet.get(r, c) {
                xlsx_support::Value::Num(n) => Some(n.clone()),
                _ => None,
            })
            .collect();
        let want = if trip_view == TripView::Partner {
            "-776"
        } else {
            "-1940"
        };
        assert!(
            numbers.contains(&want.to_owned()),
            "{trip_view:?}: {want} in {numbers:?}"
        );
    }
    // R34 in Urdu: the cash closes at 58,484 on the worked day.
    let w2 = full_day();
    let path = dir.path().join("r34.xlsx");
    let done = export_report(
        w2.env.ctx.db(),
        &ReportExportInput {
            id: ReportId::R34,
            filters: dates("10-10-2026", "10-10-2026"),
            language: Language::Ur,
            path: path.display().to_string(),
        },
        &mut |_| {},
    )
    .unwrap();
    let page = report(&w2, ReportId::R34, dates("10-10-2026", "10-10-2026"), 0, 0);
    assert_eq!(done.rows, u64::try_from(page.rows.len()).unwrap());
    let sheet = xlsx_support::read(&path);
    assert!(sheet.right_to_left);
    let numbers: Vec<String> = (0..=sheet.last_row())
        .flat_map(|r| (0..6).map(move |c| (r, c)))
        .filter_map(|(r, c)| match sheet.get(r, c) {
            xlsx_support::Value::Num(n) => Some(n.clone()),
            _ => None,
        })
        .collect();
    for want in ["41400", "5448", "50000", "22800", "14316", "58484", "21564"] {
        assert!(numbers.contains(&want.to_owned()), "{want} in {numbers:?}");
    }
}

#[test]
fn the_print_data_of_the_day_summary_and_the_trip_layouts_names_their_filters() {
    let (w, _, _) = trip_34();
    let print = |id: ReportId, filters: ReportFilters| {
        print_data(
            w.env.ctx.db(),
            &ReportPrintInput {
                id,
                filters,
                language: Language::En,
                paper: None,
            },
        )
        .unwrap()
    };
    let r33 = print(ReportId::R33, view(TripView::Partner));
    assert!(
        r33.filters
            .iter()
            .any(|f| f.label_id == "report-filter-trip-by-partner")
    );
    let r34 = print(ReportId::R34, dates("10-10-2026", "10-10-2026"));
    assert!(
        r34.filters
            .iter()
            .any(|f| f.label_id == "report-filter-from")
    );
    assert!(!r34.rows.is_empty());
    let r15 = print(
        ReportId::R15,
        ReportFilters {
            voucher_type: Some(VoucherType::Payment),
            ..ReportFilters::default()
        },
    );
    assert!(
        r15.filters
            .iter()
            .any(|f| f.label_id == "report-doc-payment")
    );
}

// ---- Fixtures for the frontend ----------------------------------------------------------------

/// Real answers of the round 8 reports as JSON files for the frontend: each pair is the request
/// (read back into the real input type, so it is exactly what the command takes) and the answer
/// the real code gives. The test compares the answers with the files on disk and FAILS if any
/// differ, so a fixture can never go stale. After a deliberate change, write the files again with
/// `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test reports_round8 fixtures`, read the
/// diff and tell `frontend`.
mod fixtures {
    use std::path::{Path, PathBuf};

    use serde::Serialize;
    use serde_json::{Value, json};

    use super::*;

    const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

    fn folder() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }

    fn write_or_check(file: &str, text: &str) {
        let path = folder().join(file);
        if std::env::var_os(WRITE_VARIABLE).is_some() {
            std::fs::create_dir_all(folder()).unwrap();
            std::fs::write(&path, text).unwrap();
            return;
        }
        let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!("{file} is missing: run with {WRITE_VARIABLE}=1 to write the fixtures")
        });
        assert!(
            on_disk == text,
            "{file} is not the real answer any more. If the change is on purpose, run with \
             {WRITE_VARIABLE}=1, read the diff and tell frontend."
        );
    }

    fn pair<A: Serialize>(name: &str, request: &Value, answer: &A) {
        write_or_check(
            &format!("{name}_request.json"),
            &(serde_json::to_string_pretty(request).unwrap() + "\n"),
        );
        write_or_check(
            &format!("{name}.json"),
            &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
        );
    }

    fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
        serde_json::from_value(request.clone()).unwrap()
    }

    /// The filters with every key, as the frontend sends them.
    fn filters(extra: Value) -> Value {
        let mut base = json!({
            "shortcut": null, "from": "", "to": "", "as_of": "", "party": null,
            "seller_id": null, "zamindar_id": null, "item_id": null, "city_id": null,
            "adda_id": null, "partner_id": null, "voucher_type": null, "trip_view": null,
            "detail": false
        });
        // Every key of the generated type, so the request is the exact input of the command.
        if let Some(map) = base.as_object_mut() {
            for key in ["partner_id", "garden_id", "bikri_id"] {
                map.entry(key).or_insert(Value::Null);
            }
        }
        if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
            for (k, v) in extra {
                base.insert(k.clone(), v.clone());
            }
        }
        base
    }

    fn run_pair(w: &World, name: &str, request: Value) {
        let input: mandi_reports::reports::ReportRunInput = typed(&request);
        let page = mandi_reports::reports::run(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &page);
    }

    fn print_pair(w: &World, name: &str, request: Value) {
        let input: ReportPrintInput = typed(&request);
        pair(name, &request, &print_data(w.env.ctx.db(), &input).unwrap());
    }

    #[test]
    fn the_trip_fixtures_are_the_real_answers() {
        let (w, _, purchase) = trip_34();
        write_off_5(&w, purchase);
        for (name, view) in [
            ("reports_run_r33_trip_e2e_34b", "trip"),
            ("reports_run_r33_partner_e2e_34b", "partner"),
            ("reports_run_r33_item_e2e_34b", "item"),
            ("reports_run_r33_month_e2e_34b", "month"),
        ] {
            run_pair(
                &w,
                name,
                json!({ "id": "R33", "offset": 0, "limit": 100,
                    "filters": filters(json!({ "trip_view": view })) }),
            );
        }
        print_pair(
            &w,
            "reports_print_data_r33_partner_en",
            json!({ "id": "R33", "language": "en", "paper": null,
                "filters": filters(json!({ "trip_view": "partner" })) }),
        );
        let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
        assert!(stock.own_rows.is_empty());
    }

    #[test]
    fn the_stock_fixture_with_a_garden_arrival_is_the_real_answer() {
        // Plan 33 v1.1: R10 holds a garden arrival (60 of 100 sold in a linked lot, 40 left).
        let w = worked_day();
        let (garden_account, arrival) = garden_with_arrival(&w, "100", "10-10-2026");
        let lot = support::with_sales(
            mandi_services::core::bikri::LotInput {
                arrival_id: Some(arrival.id),
                ..support::bare_lot(&w, "g-1", garden_account, w.banana, "60")
            },
            vec![support::sale("s-1", w.akram, "60", "600")],
        );
        bikri::save(
            &w.env.ctx,
            &w.owner,
            &BikriSaveInput {
                draft: support::draft(vec![lot]),
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
        run_pair(
            &w,
            "reports_run_r10_with_garden_arrival",
            json!({ "id": "R10", "offset": 0, "limit": 100, "filters": filters(json!({})) }),
        );
        print_pair(
            &w,
            "reports_print_data_r10_with_garden_arrival_en",
            json!({ "id": "R10", "language": "en", "paper": null, "filters": filters(json!({})) }),
        );
    }

    #[test]
    fn the_day_and_voucher_list_fixtures_are_the_real_answers() {
        let w = full_day();
        run_pair(
            &w,
            "reports_run_r34_e2e_36",
            json!({ "id": "R34", "offset": 0, "limit": 100,
                "filters": filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })) }),
        );
        print_pair(
            &w,
            "reports_print_data_r34_en",
            json!({ "id": "R34", "language": "en", "paper": null,
                "filters": filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })) }),
        );
        run_pair(
            &w,
            "reports_run_r15_all",
            json!({ "id": "R15", "offset": 0, "limit": 100, "filters": filters(json!({})) }),
        );
        run_pair(
            &w,
            "reports_run_r15_payments",
            json!({ "id": "R15", "offset": 0, "limit": 100,
                "filters": filters(json!({ "voucher_type": "payment" })) }),
        );
        run_pair(
            &w,
            "reports_run_r23_day",
            json!({ "id": "R23", "offset": 0, "limit": 100,
                "filters": filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })) }),
        );
        run_pair(
            &w,
            "reports_run_r25_day",
            json!({ "id": "R25", "offset": 0, "limit": 100,
                "filters": filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })) }),
        );
    }

    #[test]
    fn the_stock_fixture_is_the_real_answer() {
        let w = worked_day();
        let draft = support::draft(vec![support::with_sales(
            support::bare_lot(&w, "k-1", w.rahim, w.banana, "50"),
            vec![support::sale("s-1", w.akram, "20", "600")],
        )]);
        bikri::save(
            &w.env.ctx,
            &w.owner,
            &BikriSaveInput {
                draft,
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
        run_pair(
            &w,
            "reports_run_r10_stock",
            json!({ "id": "R10", "offset": 0, "limit": 100, "filters": filters(json!({})) }),
        );
    }
}
