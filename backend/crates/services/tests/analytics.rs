//! Analytics views A, B, C, D, E and H on the PRD worked day (plan 21 step B4, PRD 18, FR-18.1 to
//! FR-18.8, E2E-11, E2E-13, E2E-15, E2E-16, E2E-16b).
//!
//! The expected numbers are the PRD's: view A as in Section 18.4 (کیلا 50 نگ, 29,400, average 588,
//! total commission 3,928, 72.10%; سیب 10 نگ, 1,520, 27.90%; the total row 60, 41,400, 5,448), views
//! B and C as in 18.4, and the numbers after an edit (E2E-13), a cancel (E2E-15) and a part-sold
//! بکری with its later sale (E2E-16, E2E-16b). Every total is also checked against the journal
//! (money agent Section 9): the seller commission, buyer commission and لاگا of the facts equal the
//! net جمع of 04010001, 04010002 and 04010003 for the sale postings, and the lines behind a
//! number add up to the number.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::analytics::{
    AnalyticsCompare, AnalyticsFilters, AnalyticsGroupBy, AnalyticsKey, AnalyticsLinesInput,
    AnalyticsLinesPage, AnalyticsPage, AnalyticsPeriod, AnalyticsPrintInput, AnalyticsRunInput,
    AnalyticsSavedView, AnalyticsSide, AnalyticsView, AnalyticsViewsSaveInput, catalog, lines,
    print_data, run, views_get, views_save,
};
use mandi_reports::reports::{Cell, PartyFilter, PrintCell, RowKind};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{
    CancelInput, EditOpenInput, EditSaveInput, LaterSaleSaveInput, StockListInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::{bikri, vouchers};
use reports_world::*;

// ---- Reading a page ------------------------------------------------------------------------

fn filters() -> AnalyticsFilters {
    AnalyticsFilters::default()
}

fn on(from: &str, to: &str) -> AnalyticsFilters {
    AnalyticsFilters {
        from: from.into(),
        to: to.into(),
        ..filters()
    }
}

fn view(w: &support::World, v: AnalyticsView, f: AnalyticsFilters) -> AnalyticsPage {
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

fn refused(w: &support::World, v: AnalyticsView, f: AnalyticsFilters) -> Vec<(ErrorCode, String)> {
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

fn num(cell: &Cell) -> i64 {
    match cell {
        Cell::Number { value } => *value,
        Cell::Empty => 0,
        other => panic!("not a number: {other:?}"),
    }
}

fn pct(cell: &Cell) -> Option<i64> {
    match cell {
        Cell::Percent { hundredths } => Some(*hundredths),
        Cell::Empty => None,
        other => panic!("not a percent: {other:?}"),
    }
}

fn name(cell: &Cell) -> String {
    match cell {
        Cell::Names { names } => names.en.clone(),
        Cell::Party { party } => party.name_en.clone(),
        Cell::Empty => String::new(),
        other => panic!("not a name: {other:?}"),
    }
}

/// The cells of a row as `[n, money, money, ...]` rupees, `0` for an empty cell.
fn rupees_of(cells: &[Cell], range: std::ops::Range<usize>) -> Vec<i64> {
    cells[range]
        .iter()
        .map(|c| match c {
            Cell::Number { value } => *value,
            other => money(other),
        })
        .collect()
}

fn data_rows(page: &AnalyticsPage) -> Vec<&mandi_reports::analytics::AnalyticsRow> {
    page.rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .collect()
}

// ---- Worlds ------------------------------------------------------------------------------

/// Events 1 and 2 and the receipt, the way `full_day` makes them, with the ids to cancel.
struct Day {
    w: support::World,
    one: mandi_services::core::bikri::BikriSaved,
    two: mandi_services::core::bikri::BikriSaved,
}

fn day() -> Day {
    let w = worked_day();
    let one = save(&w, &support::event1(&w));
    let two = save(&w, &event2(&w));
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    Day { w, one, two }
}

/// E2E-13: اکرم's rate 600 to 610.
fn edit_akram(day: &Day) {
    let w = &day.w;
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: day.one.bikri_id,
            entry_id: day.one.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("ریٹ کی درستی".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

/// E2E-15: بکری 2 cancelled.
fn cancel_two(day: &Day) {
    let w = &day.w;
    let detail = bikri::get(&w.env.ctx, day.two.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: day.two.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
}

/// E2E-16 and E2E-16b: 45 of 50 sold on 10-10-26, the last 5 sold to ارشد at 560 on 11-10-26.
fn part_sold_world() -> support::World {
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
        &LaterSaleSaveInput {
            lot_id: stock.rows[0].lot_id,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    w
}

/// The net جمع of an income account over the posting entries of بکری, in rupees (the journal
/// side of the reconciliation, money agent Section 9).
fn journal_income(w: &support::World, code: &str) -> i64 {
    let (cr, dr): (i64, i64) = raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(l.cr_paisa), 0), COALESCE(SUM(l.dr_paisa), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             JOIN accounts a ON a.id = l.account_id
             WHERE a.code = ?1 AND e.source_type = 'bikri' AND e.status = 'posted'",
            [code],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    (cr - dr) / 100
}

/// The totals row of view A against the journal and against every other view.
fn assert_consistent(w: &support::World) {
    let a = view(w, AnalyticsView::A, filters());
    let t = &a.totals;
    assert_eq!(
        money(&t[4]),
        journal_income(w, "04010001"),
        "seller commission"
    );
    assert_eq!(
        money(&t[5]),
        journal_income(w, "04010002"),
        "buyer commission"
    );
    assert_eq!(money(&t[6]), journal_income(w, "04010003"), "laga");
    let total = money(&t[7]);
    assert_eq!(total, money(&t[4]) + money(&t[5]) + money(&t[6]));
    // Every view adds up to the same total commission.
    let b = view(w, AnalyticsView::B, filters());
    assert_eq!(money(&b.totals[6]), total, "view B");
    let d = view(w, AnalyticsView::D, filters());
    assert_eq!(money(d.totals.last().unwrap()), total, "view D");
    let e = view(w, AnalyticsView::E, filters());
    assert_eq!(money(e.totals.last().unwrap()), total, "view E");
    let h = view(w, AnalyticsView::H, filters());
    assert_eq!(money(&h.totals[3]), total, "view H");
    // The row totals of D and E are the A and B figures of the same item or seller.
    for row in data_rows(&d) {
        let item = row.key.as_ref().unwrap().item_id.unwrap();
        let in_a = data_rows(&a)
            .into_iter()
            .find(|r| r.key.as_ref().unwrap().item_id == Some(item))
            .unwrap();
        assert_eq!(money(row.cells.last().unwrap()), money(&in_a.cells[7]));
    }
    for row in data_rows(&e) {
        let seller = row.key.as_ref().unwrap().seller_id.unwrap();
        let in_b = data_rows(&b)
            .into_iter()
            .find(|r| r.key.as_ref().unwrap().seller_id == Some(seller))
            .unwrap();
        assert_eq!(money(row.cells.last().unwrap()), money(&in_b.cells[6]));
    }
    // R7 has the same sums by item (view A without the share column).
    let r7 = report(
        w,
        mandi_reports::reports::ReportId::R7,
        Default::default(),
        0,
        0,
    );
    assert_eq!(money(&r7.totals[7]), total, "R7");
}

// ---- Catalogue ---------------------------------------------------------------------------

#[test]
fn the_catalogue_lists_the_eleven_views_and_only_d_and_g_take_the_grouping() {
    use mandi_reports::analytics::AnalyticsFilterKind::GroupBy;
    let entries = catalog();
    let ids: Vec<AnalyticsView> = entries.iter().map(|e| e.id).collect();
    assert_eq!(
        ids,
        vec![
            AnalyticsView::A,
            AnalyticsView::B,
            AnalyticsView::C,
            AnalyticsView::D,
            AnalyticsView::E,
            AnalyticsView::F,
            AnalyticsView::G,
            AnalyticsView::H,
            AnalyticsView::I,
            AnalyticsView::J,
            AnalyticsView::K
        ]
    );
    for entry in &entries {
        assert_eq!(
            entry.filters.contains(&GroupBy),
            matches!(entry.id, AnalyticsView::D | AnalyticsView::G),
            "{:?}",
            entry.id
        );
        assert_eq!(
            entry.title_id,
            format!("analytics-view-{}-title", entry.id.code().to_lowercase())
        );
    }
}

// ---- The worked day: E2E-11 --------------------------------------------------------------

#[test]
fn view_a_is_the_prd_table_of_section_18_4() {
    let w = full_day();
    let page = view(&w, AnalyticsView::A, filters());
    let rows = data_rows(&page);
    assert_eq!(rows.len(), 2);
    // کیلا: 50 نگ, 29,400, average 588, 1,764, 1,764, 400, total 3,928, 72.10%.
    assert_eq!(name(&rows[0].cells[0]), "Banana");
    assert_eq!(rupees_of(&rows[0].cells, 1..4), [50, 29_400, 588]);
    assert_eq!(rupees_of(&rows[0].cells, 4..8), [1_764, 1_764, 400, 3_928]);
    assert_eq!(pct(&rows[0].cells[8]), Some(7_210));
    // سیب: 10 نگ, 12,000, average 1,200, 720, 720, 80, total 1,520, 27.90%.
    assert_eq!(name(&rows[1].cells[0]), "Apple");
    assert_eq!(rupees_of(&rows[1].cells, 1..4), [10, 12_000, 1_200]);
    assert_eq!(rupees_of(&rows[1].cells, 4..8), [720, 720, 80, 1_520]);
    assert_eq!(pct(&rows[1].cells[8]), Some(2_790));
    // The total row: 60, 41,400, no average, 2,484, 2,484, 480, 5,448, 100.00%.
    assert_eq!(page.totals[3], Cell::Empty);
    assert_eq!(rupees_of(&page.totals, 1..3), [60, 41_400]);
    assert_eq!(rupees_of(&page.totals, 4..8), [2_484, 2_484, 480, 5_448]);
    assert_eq!(pct(&page.totals[8]), Some(10_000));
    assert_eq!(page.total_count, 2);
    // The commission is the dashboard's: 5,448 (PRD 4.9).
    assert_eq!(money(&page.totals[7]), 5_448);
    // Each row opens its lines by item.
    assert_eq!(rows[0].key.as_ref().unwrap().item_id, Some(w.banana));
    assert_consistent(&w);
}

#[test]
fn view_b_is_the_prd_table_of_section_18_4() {
    let w = full_day();
    let page = view(&w, AnalyticsView::B, filters());
    let rows = data_rows(&page);
    assert_eq!(rows.len(), 2);
    // رحیم: 1 بکری, 50 نگ, 29,400, 1,764, 1,764, total 3,928, net 19,316.
    assert_eq!(name(&rows[0].cells[0]), "Rahim");
    assert_eq!(
        rupees_of(&rows[0].cells, 1..8),
        [1, 50, 29_400, 1_764, 1_764, 3_928, 19_316]
    );
    // کریم: 1, 10, 12,000, 720, 720, 1,520, net 10,160.
    assert_eq!(name(&rows[1].cells[0]), "Karim");
    assert_eq!(
        rupees_of(&rows[1].cells, 1..8),
        [1, 10, 12_000, 720, 720, 1_520, 10_160]
    );
    // The total row: 2 بکری, 60 نگ, 41,400, 2,484, 2,484, 5,448 and the nets add up.
    assert_eq!(
        rupees_of(&page.totals, 1..8),
        [2, 60, 41_400, 2_484, 2_484, 5_448, 29_476]
    );
}

#[test]
fn view_c_is_the_prd_table_of_section_18_4_with_the_one_time_buyer() {
    let w = full_day();
    let page = view(&w, AnalyticsView::C, filters());
    let rows = data_rows(&page);
    assert_eq!(rows.len(), 3);
    // ارشد: 30 نگ, 17,400, 1,044, 240, dues 18,684, no average.
    assert_eq!(name(&rows[0].cells[0]), "Arshad Multani");
    assert_eq!(rupees_of(&rows[0].cells, 1..5), [30, 17_400, 1_044, 240]);
    assert_eq!(
        balance(&rows[0].cells[5]),
        (mandi_services::core::accounts::BalanceSide::Dr, 18_684)
    );
    assert_eq!(rows[0].cells[6], Cell::Empty);
    // اکرم: 20, 12,000, 720, 160, dues 2,880, no average (nothing is fully paid).
    assert_eq!(name(&rows[1].cells[0]), "Akram Toll Tax");
    assert_eq!(rupees_of(&rows[1].cells, 1..5), [20, 12_000, 720, 160]);
    assert_eq!(
        balance(&rows[1].cells[5]),
        (mandi_services::core::accounts::BalanceSide::Dr, 2_880)
    );
    assert_eq!(rows[1].cells[6], Cell::Empty);
    // شاہد, the one-time buyer: 10, 12,000, 720, 80, dues 0, paid on the day: 0 days.
    assert_eq!(name(&rows[2].cells[0]), "Shahid");
    assert_eq!(rupees_of(&rows[2].cells, 1..5), [10, 12_000, 720, 80]);
    assert_eq!(
        balance(&rows[2].cells[5]),
        (mandi_services::core::accounts::BalanceSide::Zero, 0)
    );
    assert_eq!(rows[2].cells[6], Cell::Number { value: 0 });
    let key = rows[2].key.as_ref().unwrap();
    assert_eq!(key.buyer_id, Some(w.pukhta_buyer));
    assert!(key.person_key.is_some());
    // The total row.
    assert_eq!(rupees_of(&page.totals, 1..5), [60, 41_400, 2_484, 480]);
}

#[test]
fn views_d_e_and_h_on_the_worked_day() {
    let w = full_day();
    // D: one column for October 2026; کیلا 3,928, سیب 1,520, total 5,448.
    let d = view(&w, AnalyticsView::D, filters());
    assert_eq!(d.columns.len(), 3, "item, 2026-10, total");
    assert!(matches!(
        d.columns[1].heading,
        Some(mandi_reports::analytics::AnalyticsHeading::Month {
            year: 2026,
            month: 10
        })
    ));
    let drill = d.columns[1].drill.as_ref().unwrap();
    assert_eq!(
        (drill.from.as_deref(), drill.to.as_deref()),
        (Some("2026-10-01"), Some("2026-10-31"))
    );
    let rows = data_rows(&d);
    assert_eq!(rupees_of(&rows[0].cells, 1..3), [3_928, 3_928]);
    assert_eq!(rupees_of(&rows[1].cells, 1..3), [1_520, 1_520]);
    assert_eq!(rupees_of(&d.totals, 1..3), [5_448, 5_448]);
    // E: رحیم × کیلا 3,928; کریم × سیب 1,520. The item columns are in item code order.
    let e = view(&w, AnalyticsView::E, filters());
    assert_eq!(e.columns.len(), 4, "seller, two items, total");
    let rows = data_rows(&e);
    assert_eq!(name(&rows[0].cells[0]), "Rahim");
    let by_item = |row: &mandi_reports::analytics::AnalyticsRow, item: i64| -> i64 {
        let index = e
            .columns
            .iter()
            .position(|c| c.drill.as_ref().and_then(|d| d.item_id) == Some(item))
            .unwrap();
        money(&row.cells[index])
    };
    assert_eq!(by_item(rows[0], w.banana), 3_928);
    assert_eq!(by_item(rows[0], w.apple), 0);
    assert_eq!(by_item(rows[1], w.apple), 1_520);
    assert_eq!(money(rows[0].cells.last().unwrap()), 3_928);
    assert_eq!(money(e.totals.last().unwrap()), 5_448);
    // H: items کیلا 3,928, سیب 1,520; sellers رحیم, کریم; buyers ارشد 2,328, اکرم 1,600, شاہد 1,520.
    let h = view(&w, AnalyticsView::H, filters());
    let kinds: Vec<RowKind> = h.rows.iter().map(|r| r.kind).collect();
    assert_eq!(
        kinds,
        [
            RowKind::Group,
            RowKind::Normal,
            RowKind::Normal,
            RowKind::Group,
            RowKind::Normal,
            RowKind::Normal,
            RowKind::Group,
            RowKind::Normal,
            RowKind::Normal,
            RowKind::Normal,
        ]
    );
    let listed: Vec<(String, i64)> = data_rows(&h)
        .iter()
        .map(|r| (name(&r.cells[1]), money(&r.cells[3])))
        .collect();
    assert_eq!(
        listed,
        [
            ("Banana".to_owned(), 3_928),
            ("Apple".to_owned(), 1_520),
            ("Rahim".to_owned(), 3_928),
            ("Karim".to_owned(), 1_520),
            ("Arshad Multani".to_owned(), 2_328),
            ("Akram Toll Tax".to_owned(), 1_600),
            ("Shahid".to_owned(), 1_520),
        ]
    );
    assert_eq!(money(&h.totals[3]), 5_448);
    assert_eq!(pct(&data_rows(&h)[0].cells[4]), Some(7_210));
}

// ---- Edit and cancel: E2E-13 and E2E-15 ----------------------------------------------------

#[test]
fn an_edit_leaves_the_stored_values_e2e_13() {
    let d = day();
    edit_akram(&d);
    let w = &d.w;
    // A: کیلا 50 نگ, 29,600, average 592, 1,776, 1,776, 400, total 3,952, 72.22%; سیب 27.78%.
    let a = view(w, AnalyticsView::A, filters());
    let rows = data_rows(&a);
    assert_eq!(rupees_of(&rows[0].cells, 1..4), [50, 29_600, 592]);
    assert_eq!(rupees_of(&rows[0].cells, 4..8), [1_776, 1_776, 400, 3_952]);
    assert_eq!(pct(&rows[0].cells[8]), Some(7_222));
    assert_eq!(pct(&rows[1].cells[8]), Some(2_778));
    assert_eq!(money(&a.totals[7]), 5_472);
    // B: رحیم 1 بکری, 29,600, total 3,952, net 19,504.
    let b = view(w, AnalyticsView::B, filters());
    assert_eq!(
        rupees_of(&data_rows(&b)[0].cells, 1..8),
        [1, 50, 29_600, 1_776, 1_776, 3_952, 19_504]
    );
    // C: اکرم 20 نگ, 12,200, 732, 160, dues 13,092 − 10,000 = 3,092.
    let c = view(w, AnalyticsView::C, filters());
    let akram = data_rows(&c)
        .into_iter()
        .find(|r| name(&r.cells[0]) == "Akram Toll Tax")
        .unwrap();
    assert_eq!(rupees_of(&akram.cells, 1..5), [20, 12_200, 732, 160]);
    assert_eq!(
        balance(&akram.cells[5]),
        (mandi_services::core::accounts::BalanceSide::Dr, 3_092)
    );
    assert_consistent(w);
}

#[test]
fn a_cancel_leaves_nothing_of_the_bikri_e2e_15() {
    let d = day();
    cancel_two(&d);
    let w = &d.w;
    // A has only کیلا: 100.00% of 3,928. سیب has no row.
    let a = view(w, AnalyticsView::A, filters());
    let rows = data_rows(&a);
    assert_eq!(rows.len(), 1);
    assert_eq!(name(&rows[0].cells[0]), "Banana");
    assert_eq!(pct(&rows[0].cells[8]), Some(10_000));
    assert_eq!(money(&a.totals[7]), 3_928);
    // B has only رحیم, C has no one-time buyer, H lists nobody from بکری 2.
    let b = view(w, AnalyticsView::B, filters());
    assert_eq!(data_rows(&b).len(), 1);
    assert_eq!(rupees_of(&b.totals, 1..2), [1]);
    let c = view(w, AnalyticsView::C, filters());
    let names: Vec<String> = data_rows(&c).iter().map(|r| name(&r.cells[0])).collect();
    assert_eq!(names, ["Arshad Multani", "Akram Toll Tax"]);
    assert_consistent(w);
    // The lines of سیب keep the cancelled bill and its reversal: +1,520 and −1,520 add to 0.
    let apple = lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::A,
            filters: filters(),
            row_key: AnalyticsKey {
                item_id: Some(w.apple),
                ..AnalyticsKey::default()
            },
            column_key: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(apple.total_count, 2);
    assert_eq!(money(&apple.rows[0].cells[11]), 1_520);
    assert_eq!(money(&apple.rows[1].cells[11]), -1_520);
    assert_eq!(money(&apple.totals[11]), 0);
}

// ---- The lines behind a number: FR-18.2 -----------------------------------------------------

fn lines_for(
    w: &support::World,
    v: AnalyticsView,
    f: AnalyticsFilters,
    row_key: AnalyticsKey,
    column_key: Option<AnalyticsKey>,
) -> AnalyticsLinesPage {
    lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: v,
            filters: f,
            row_key,
            column_key,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
}

#[test]
fn the_lines_behind_every_number_add_up_to_the_number_after_an_edit() {
    let d = day();
    edit_akram(&d);
    let w = &d.w;
    // View A and view D/E by row: the lines' totals are the row's figures.
    let a = view(w, AnalyticsView::A, filters());
    for row in data_rows(&a) {
        let page = lines_for(
            w,
            AnalyticsView::A,
            filters(),
            row.key.clone().unwrap(),
            None,
        );
        // qty, gross, seller comm, buyer comm, laga, total comm.
        assert_eq!(
            [
                num(&page.totals[5]),
                money(&page.totals[6]),
                money(&page.totals[8]),
                money(&page.totals[9]),
                money(&page.totals[10]),
                money(&page.totals[11])
            ],
            [
                num(&row.cells[1]),
                money(&row.cells[2]),
                money(&row.cells[4]),
                money(&row.cells[5]),
                money(&row.cells[6]),
                money(&row.cells[7])
            ]
        );
        // The rows themselves add up to the totals (signed lines).
        let sum: i64 = page.rows.iter().map(|r| money(&r.cells[11])).sum();
        assert_eq!(sum, money(&page.totals[11]));
        // Each line opens its بکری.
        for line in &page.rows {
            let source = line.source.as_ref().unwrap();
            assert_eq!(source.kind, mandi_reports::reports::SourceKind::Bikri);
        }
    }
    // کیلا has 6 lines: the first posting (2), its reversal (2) and the new posting (2).
    let banana = lines_for(
        w,
        AnalyticsView::A,
        filters(),
        AnalyticsKey {
            item_id: Some(w.banana),
            ..AnalyticsKey::default()
        },
        None,
    );
    assert_eq!(banana.total_count, 6);
    let signs: Vec<bool> = banana.rows.iter().map(|r| money(&r.cells[6]) < 0).collect();
    assert_eq!(signs.iter().filter(|negative| **negative).count(), 2);
    // View B by seller, view C by buyer (including the one-time person).
    let b = view(w, AnalyticsView::B, filters());
    for row in data_rows(&b) {
        let page = lines_for(
            w,
            AnalyticsView::B,
            filters(),
            row.key.clone().unwrap(),
            None,
        );
        assert_eq!(money(&page.totals[11]), money(&row.cells[6]));
        assert_eq!(money(&page.totals[6]), money(&row.cells[3]));
    }
    let c = view(w, AnalyticsView::C, filters());
    for row in data_rows(&c) {
        let page = lines_for(
            w,
            AnalyticsView::C,
            filters(),
            row.key.clone().unwrap(),
            None,
        );
        assert_eq!(
            money(&page.totals[7]),
            money(&row.cells[2]),
            "purchase value"
        );
        assert_eq!(
            money(&page.totals[9]),
            money(&row.cells[3]),
            "buyer commission"
        );
        assert_eq!(money(&page.totals[10]), money(&row.cells[4]), "laga");
    }
    // A cell of D: the row key and the column's drill together; a cell of E the same.
    let dpage = view(w, AnalyticsView::D, filters());
    let column = dpage.columns[1].drill.clone();
    for row in data_rows(&dpage) {
        let page = lines_for(
            w,
            AnalyticsView::D,
            filters(),
            row.key.clone().unwrap(),
            column.clone(),
        );
        assert_eq!(money(&page.totals[11]), money(&row.cells[1]));
    }
    let epage = view(w, AnalyticsView::E, filters());
    for row in data_rows(&epage) {
        for (index, column) in epage.columns.iter().enumerate() {
            let Some(drill) = &column.drill else { continue };
            let page = lines_for(
                w,
                AnalyticsView::E,
                filters(),
                row.key.clone().unwrap(),
                Some(drill.clone()),
            );
            assert_eq!(
                money(&page.totals[11]),
                money(&row.cells[index]),
                "{}",
                column.key
            );
        }
    }
    // The whole filter: the empty key gives the totals row.
    let all = lines_for(
        w,
        AnalyticsView::A,
        filters(),
        AnalyticsKey::default(),
        None,
    );
    assert_eq!(money(&all.totals[11]), money(&a.totals[7]));
    // Two keys that name different records are refused.
    let clash = lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::E,
            filters: filters(),
            row_key: AnalyticsKey {
                item_id: Some(w.banana),
                ..AnalyticsKey::default()
            },
            column_key: Some(AnalyticsKey {
                item_id: Some(w.apple),
                ..AnalyticsKey::default()
            }),
            offset: 0,
            limit: 0,
        },
    );
    assert!(matches!(
        clash,
        Err(mandi_reports::ReportsError::Rejected(items)) if items[0].field == "column_key.item_id"
    ));
}

#[test]
fn lines_page_with_totals_over_the_whole_filter() {
    let w = full_day();
    let whole = lines_for(
        &w,
        AnalyticsView::A,
        filters(),
        AnalyticsKey::default(),
        None,
    );
    assert_eq!(whole.total_count, 3);
    let mut joined = Vec::new();
    for offset in 0..3 {
        let part = lines(
            w.env.ctx.db(),
            &AnalyticsLinesInput {
                view: AnalyticsView::A,
                filters: filters(),
                row_key: AnalyticsKey::default(),
                column_key: None,
                offset,
                limit: 1,
            },
        )
        .unwrap();
        assert_eq!(part.total_count, 3);
        assert_eq!(part.totals, whole.totals);
        joined.extend(part.rows);
    }
    assert_eq!(joined, whole.rows);
    // A one-time buyer's line carries his own name.
    let shahid = whole
        .rows
        .iter()
        .find_map(|r| match &r.cells[3] {
            Cell::Party { party } if party.person_key.is_some() => Some(party.name_en.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(shahid, "Shahid");
}

// ---- Periods and the lot net: E2E-16, E2E-16b -----------------------------------------------

#[test]
fn a_period_split_adds_up_and_the_seller_net_follows_the_lots_e2e_16b() {
    let w = part_sold_world();
    let first = view(&w, AnalyticsView::B, on("10-10-2026", "10-10-2026"));
    let second = view(&w, AnalyticsView::B, on("11-10-2026", "11-10-2026"));
    let both = view(&w, AnalyticsView::B, on("10-10-2026", "11-10-2026"));
    // E2E-16 on 10-10: خام 26,500; seller commission 1,590; net 16,590.
    assert_eq!(
        rupees_of(&data_rows(&first)[0].cells, 1..8),
        [1, 45, 26_500, 1_590, 1_590, 3_540, 16_590]
    );
    // E2E-16b on 11-10: 5 نگ, 2,800, commission 168, 168, total 376, net 2,632, no lot charge.
    assert_eq!(
        rupees_of(&data_rows(&second)[0].cells, 1..8),
        [1, 5, 2_800, 168, 168, 376, 2_632]
    );
    // Both days: خام 29,300, صافی 19,222 (the lot's stored net).
    assert_eq!(
        rupees_of(&data_rows(&both)[0].cells, 1..8),
        [1, 50, 29_300, 1_758, 1_758, 3_916, 19_222]
    );
    // Every column adds up across the split.
    let value = |cell: &Cell| match cell {
        Cell::Number { value } => *value,
        other => money(other),
    };
    for index in 2..8 {
        assert_eq!(
            value(&first.totals[index]) + value(&second.totals[index]),
            value(&both.totals[index]),
            "column {index}"
        );
    }
    // The stored lot net is the same 19,222 (E2E-16b).
    let stored: i64 = raw(&w.dir)
        .query_row("SELECT SUM(net_paisa) FROM bikri_lots", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stored / 100, 19_222);
    // The same split for view A.
    let a1 = view(&w, AnalyticsView::A, on("10-10-2026", "10-10-2026"));
    let a2 = view(&w, AnalyticsView::A, on("11-10-2026", "11-10-2026"));
    let a3 = view(&w, AnalyticsView::A, on("10-10-2026", "11-10-2026"));
    for index in [1, 2, 4, 5, 6, 7] {
        let value = |p: &AnalyticsPage| match &p.totals[index] {
            Cell::Number { value } => *value,
            other => money(other),
        };
        assert_eq!(value(&a1) + value(&a2), value(&a3), "column {index}");
    }
    assert_consistent(&w);
    // The buyer-only side leaves the net empty (a lot's charges are the seller's).
    let buyer_side = view(
        &w,
        AnalyticsView::B,
        AnalyticsFilters {
            side: AnalyticsSide::Buyer,
            ..filters()
        },
    );
    assert_eq!(data_rows(&buyer_side)[0].cells[7], Cell::Empty);
}

#[test]
fn a_lot_with_charges_and_no_sale_in_the_period_keeps_the_nets_adding_up() {
    // Money review of the code, finding 1: کریم's second lot waits in باقی مال on 10-10 with its
    // labour 50 and munshiana 50 typed, and is sold on 11-10. Its charges belong to 10-10.
    let w = worked_day();
    let mut draft = support::event1(&w);
    let mut waiting = support::bare_lot(&w, "lot-2", w.karim, w.apple, "10");
    waiting.labour = support::manual("50");
    waiting.munshiana = support::manual("50");
    draft.lots.push(waiting);
    save(&w, &draft);
    set_today(&w.env, 2026, 10, 11);
    let stock = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert_eq!(stock.rows.len(), 1);
    let mut line = support::sale("s-3", w.arshad, "10", "1200");
    line.sale_id = None;
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id: stock.rows[0].lot_id,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let day1 = view(&w, AnalyticsView::B, on("10-10-2026", "10-10-2026"));
    let day2 = view(&w, AnalyticsView::B, on("11-10-2026", "11-10-2026"));
    let both = view(&w, AnalyticsView::B, on("10-10-2026", "11-10-2026"));
    // 10-10: رحیم 19,316 and کریم with no sales and the charges taken off: −100.
    let karim1 = data_rows(&day1)
        .into_iter()
        .find(|r| name(&r.cells[0]) == "Karim")
        .unwrap();
    assert_eq!(rupees_of(&karim1.cells, 1..8), [0, 0, 0, 0, 0, 0, -100]);
    assert_eq!(money(&day1.totals[7]), 19_316 - 100);
    // 11-10: 12,000 less 720 commission, and no charge again.
    let karim2 = data_rows(&day2)[0];
    assert_eq!(
        rupees_of(&karim2.cells, 1..8),
        [1, 10, 12_000, 720, 720, 1_520, 11_280]
    );
    assert_eq!(money(&day2.totals[7]), 11_280);
    // Both days: 19,216 + 11,280 = 30,496, which is also the sum of the stored lot nets
    // (19,316 + 11,180).
    assert_eq!(money(&both.totals[7]), 30_496);
    assert_eq!(
        money(&day1.totals[7]) + money(&day2.totals[7]),
        money(&both.totals[7])
    );
    let stored: i64 = raw(&w.dir)
        .query_row(
            "SELECT SUM(net_paisa) FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id
             WHERE b.status = 'saved'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(stored / 100, 30_496);
}

#[test]
fn the_average_days_to_pay_counts_live_fully_paid_bills_only() {
    // Money review of the code, finding 2. اکرم has two bills dated 10-10: 12,880 (بکری 1) and a
    // small one. The first is paid in full on 11-10 (1 day), the second on 12-10 (2 days).
    let w = worked_day();
    save(&w, &support::event1(&w));
    let small = support::with_sales(
        support::bare_lot(&w, "lot-1", w.karim, w.apple, "5"),
        vec![support::sale("s-1", w.akram, "5", "100")],
    );
    let mut second = support::draft(vec![small]);
    second.header.truck_no = Some("T-2".into());
    save(&w, &second);
    let bill2: i64 = raw(&w.dir)
        .query_row(
            "SELECT l.dr_paisa FROM journal_lines l WHERE l.account_id = ?1 AND l.dr_paisa > 0
             ORDER BY l.id DESC LIMIT 1",
            [w.akram],
            |r| r.get(0),
        )
        .unwrap();
    let bill2 = (bill2 / 100).to_string();
    let akram_days = |f: AnalyticsFilters| -> Cell {
        let page = view(&w, AnalyticsView::C, f);
        data_rows(&page)
            .into_iter()
            .find(|r| name(&r.cells[0]) == "Akram Toll Tax")
            .unwrap()
            .cells[6]
            .clone()
    };
    // Nothing is paid yet.
    assert_eq!(akram_days(filters()), Cell::Empty);

    set_today(&w.env, 2026, 10, 11);
    let mut first = receipt(w.akram, "12,880");
    first.date = "11-10-2026".into();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &first).unwrap();
    // One bill paid in 1 day.
    assert_eq!(akram_days(filters()), Cell::Number { value: 1 });

    set_today(&w.env, 2026, 10, 12);
    let mut last = receipt(w.akram, &bill2);
    last.date = "12-10-2026".into();
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &last).unwrap();
    // 1 and 2 days: 1.5 rounds half up to 2.
    assert_eq!(akram_days(filters()), Cell::Number { value: 2 });
    // Only the bills paid in full inside the period count. The buyer has a row in a period only
    // if he bought in it, so each period starts on the day of the purchases.
    assert_eq!(
        akram_days(on("10-10-2026", "11-10-2026")),
        Cell::Number { value: 1 }
    );
    assert_eq!(
        akram_days(on("10-10-2026", "12-10-2026")),
        Cell::Number { value: 2 }
    );
    assert_eq!(akram_days(on("10-10-2026", "10-10-2026")), Cell::Empty);
    // An item filter leaves the cell empty: a bill cannot be split by item.
    assert_eq!(
        akram_days(AnalyticsFilters {
            item_id: Some(w.apple),
            ..filters()
        }),
        Cell::Empty
    );
    // A cancelled receipt does not count: the second bill is open again.
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::VoucherCancelInput {
            id: saved.voucher_id,
            version: row.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    assert_eq!(akram_days(filters()), Cell::Number { value: 1 });
}

// ---- Compare -----------------------------------------------------------------------------

#[test]
fn compare_previous_period_and_same_period_last_year() {
    let w = part_sold_world();
    // 11-10 against the day before: total commission 376 against 3,540, change −3,164.
    let page = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            compare: AnalyticsCompare::Previous,
            ..on("11-10-2026", "11-10-2026")
        },
    );
    assert_eq!(
        (
            page.filters.compare_from.as_deref(),
            page.filters.compare_to.as_deref()
        ),
        (Some("2026-10-10"), Some("2026-10-10"))
    );
    assert_eq!(page.columns.len(), 11, "two compare columns");
    let banana = data_rows(&page)[0];
    assert_eq!(money(&banana.cells[7]), 376);
    assert_eq!(money(&banana.cells[9]), 3_540);
    assert_eq!(money(&banana.cells[10]), -3_164);
    assert_eq!(money(&page.totals[9]), 3_540);
    assert_eq!(money(&page.totals[10]), -3_164);
    // Same dates last year: nothing then, so the change is the whole figure.
    let year = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            compare: AnalyticsCompare::LastYear,
            ..on("11-10-2026", "11-10-2026")
        },
    );
    assert_eq!(year.filters.compare_from.as_deref(), Some("2025-10-11"));
    assert_eq!(money(&data_rows(&year)[0].cells[9]), 0);
    assert_eq!(money(&data_rows(&year)[0].cells[10]), 376);
    // A row that exists only in the earlier period is listed with 0 now: 10-10 against 11-10.
    let back = view(
        &w,
        AnalyticsView::B,
        AnalyticsFilters {
            compare: AnalyticsCompare::Previous,
            ..on("10-10-2026", "10-10-2026")
        },
    );
    assert_eq!(data_rows(&back).len(), 1);
    // C compares the purchase value; H the commission of the same names.
    let c = view(
        &w,
        AnalyticsView::C,
        AnalyticsFilters {
            compare: AnalyticsCompare::Previous,
            ..on("11-10-2026", "11-10-2026")
        },
    );
    let arshad = data_rows(&c)
        .into_iter()
        .find(|r| name(&r.cells[0]) == "Arshad Multani")
        .unwrap();
    assert_eq!(money(&arshad.cells[2]), 2_800);
    assert_eq!(money(&arshad.cells[7]), 14_500);
    // The compare also works for D, E and H.
    for v in [AnalyticsView::D, AnalyticsView::E, AnalyticsView::H] {
        let page = view(
            &w,
            v,
            AnalyticsFilters {
                compare: AnalyticsCompare::Previous,
                ..on("11-10-2026", "11-10-2026")
            },
        );
        assert_eq!(money(&page.totals[page.totals.len() - 2]), 3_540, "{v:?}");
    }
}

// ---- Filters -----------------------------------------------------------------------------

#[test]
fn filters_combine_freely_and_the_side_leaves_one_commission_out() {
    let w = full_day();
    // An item, a seller, a buyer.
    let by_item = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            item_id: Some(w.apple),
            ..filters()
        },
    );
    assert_eq!(data_rows(&by_item).len(), 1);
    assert_eq!(money(&by_item.totals[7]), 1_520);
    let by_seller = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            seller_id: Some(w.rahim),
            ..filters()
        },
    );
    assert_eq!(money(&by_seller.totals[7]), 3_928);
    let by_buyer = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            buyer: Some(PartyFilter {
                account_id: w.akram,
                person_key: None,
            }),
            ..filters()
        },
    );
    assert_eq!(rupees_of(&by_buyer.totals, 1..3), [20, 12_000]);
    assert_eq!(money(&by_buyer.totals[7]), 1_600);
    // The one-time buyer by his person key.
    let person: String = raw(&w.dir)
        .query_row(
            "SELECT pukhta_person_key FROM analytics_facts WHERE buyer_account_id = ?1",
            [w.pukhta_buyer],
            |r| r.get(0),
        )
        .unwrap();
    let by_person = view(
        &w,
        AnalyticsView::C,
        AnalyticsFilters {
            buyer: Some(PartyFilter {
                account_id: w.pukhta_buyer,
                person_key: Some(person),
            }),
            ..filters()
        },
    );
    assert_eq!(data_rows(&by_person).len(), 1);
    // The side: seller commission only. کیلا 1,764; the buyer columns are empty; the share is
    // taken of the seller commission alone (1,764 of 2,484 is 71.01%).
    let seller_side = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            side: AnalyticsSide::Seller,
            ..filters()
        },
    );
    let banana = data_rows(&seller_side)[0];
    assert_eq!(money(&banana.cells[4]), 1_764);
    assert_eq!(banana.cells[5], Cell::Empty);
    assert_eq!(banana.cells[6], Cell::Empty);
    assert_eq!(money(&banana.cells[7]), 1_764);
    assert_eq!(pct(&banana.cells[8]), Some(7_101));
    assert_eq!(money(&seller_side.totals[7]), 2_484);
    // Buyer commission and لاگا only: 2,484 + 480 = 2,964.
    let buyer_side = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            side: AnalyticsSide::Buyer,
            ..filters()
        },
    );
    assert_eq!(money(&buyer_side.totals[7]), 2_964);
    // Period shortcuts: today in the test clock is 10-10-26.
    let today = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            period: Some(AnalyticsPeriod::Today),
            ..filters()
        },
    );
    assert_eq!(money(&today.totals[7]), 5_448);
    assert_eq!(today.filters.from.as_deref(), Some("2026-10-10"));
    let last_month = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            period: Some(AnalyticsPeriod::LastMonth),
            ..filters()
        },
    );
    assert!(data_rows(&last_month).is_empty());
    assert_eq!(money(&last_month.totals[7]), 0);
    assert_eq!(pct(&last_month.totals[8]), None, "no commission, no share");
    // View D by day and by year.
    let by_day = view(
        &w,
        AnalyticsView::D,
        AnalyticsFilters {
            group_by: Some(AnalyticsGroupBy::Day),
            ..filters()
        },
    );
    assert!(matches!(
        by_day.columns[1].heading,
        Some(mandi_reports::analytics::AnalyticsHeading::Day { .. })
    ));
    let by_year = view(
        &w,
        AnalyticsView::D,
        AnalyticsFilters {
            group_by: Some(AnalyticsGroupBy::Year),
            ..filters()
        },
    );
    let drill = by_year.columns[1].drill.as_ref().unwrap();
    assert_eq!(
        (drill.from.as_deref(), drill.to.as_deref()),
        (Some("2026-01-01"), Some("2026-12-31"))
    );
}

#[test]
fn the_buyers_city_and_the_adda_narrow_the_facts() {
    let w = worked_day();
    let conn = raw(&w.dir);
    conn.execute(
        "INSERT INTO cities (code, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES ('1', 'کوٹ رادھا کشن', 'Kot Radha Kishan', 'kot', 'kot radha kishan', 'kt')",
        [],
    )
    .unwrap();
    let city = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO addas (code, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES ('1', 'سوات اڈا', 'Swat Adda', 'swat', 'swat adda', 'swt')",
        [],
    )
    .unwrap();
    let adda = conn.last_insert_rowid();
    conn.execute(
        "UPDATE accounts SET city_id = ?1 WHERE id = ?2",
        [city, w.akram],
    )
    .unwrap();
    let mut one = support::event1(&w);
    one.header.adda_id = Some(adda);
    save(&w, &one);
    save(&w, &event2(&w));
    // Only اکرم lives in the city: 20 نگ of کیلا.
    let by_city = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            city_id: Some(city),
            ..filters()
        },
    );
    assert_eq!(rupees_of(&by_city.totals, 1..3), [20, 12_000]);
    // The adda of بکری 1 holds all of it and not بکری 2.
    let by_adda = view(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            adda_id: Some(adda),
            ..filters()
        },
    );
    assert_eq!(rupees_of(&by_adda.totals, 1..3), [50, 29_400]);
    // The seller view with a buyer or city filter gives no net; with an adda it does.
    let b_city = view(
        &w,
        AnalyticsView::B,
        AnalyticsFilters {
            city_id: Some(city),
            ..filters()
        },
    );
    assert_eq!(data_rows(&b_city)[0].cells[7], Cell::Empty);
    let b_adda = view(
        &w,
        AnalyticsView::B,
        AnalyticsFilters {
            adda_id: Some(adda),
            ..filters()
        },
    );
    assert_eq!(money(&data_rows(&b_adda)[0].cells[7]), 19_316);
    // The echo names the picked records.
    assert_eq!(
        by_city.filters.city.as_ref().unwrap().en,
        "Kot Radha Kishan"
    );
    assert_eq!(by_adda.filters.adda.as_ref().unwrap().en, "Swat Adda");
}

#[test]
fn bad_filters_are_refused_together_with_their_fields() {
    let w = full_day();
    // The grouping belongs to view D only.
    assert_eq!(
        refused(
            &w,
            AnalyticsView::A,
            AnalyticsFilters {
                group_by: Some(AnalyticsGroupBy::Day),
                ..filters()
            }
        ),
        [(ErrorCode::ErrOutOfRange, "filters.group_by".to_owned())]
    );
    // A comparison needs both dates.
    assert_eq!(
        refused(
            &w,
            AnalyticsView::A,
            AnalyticsFilters {
                compare: AnalyticsCompare::Previous,
                ..filters()
            }
        ),
        [
            (ErrorCode::ErrRequired, "filters.from".to_owned()),
            (ErrorCode::ErrRequired, "filters.to".to_owned())
        ]
    );
    // Bad dates, a wrong order, unknown records, a person key on an ordinary buyer.
    let problems = refused(
        &w,
        AnalyticsView::A,
        AnalyticsFilters {
            from: "31-02-2026".into(),
            to: "abc".into(),
            item_id: Some(9_999),
            seller_id: Some(9_999),
            city_id: Some(9_999),
            adda_id: Some(9_999),
            buyer: Some(PartyFilter {
                account_id: 9_999,
                person_key: None,
            }),
            ..filters()
        },
    );
    let fields: Vec<&str> = problems.iter().map(|(_, f)| f.as_str()).collect();
    for expected in [
        "filters.from",
        "filters.to",
        "filters.item_id",
        "filters.seller_id",
        "filters.city_id",
        "filters.adda_id",
        "filters.buyer.account_id",
    ] {
        assert!(fields.contains(&expected), "{expected} in {fields:?}");
    }
    assert_eq!(
        refused(&w, AnalyticsView::A, on("11-10-2026", "10-10-2026")),
        [(ErrorCode::ErrDateRange, "filters.to".to_owned())]
    );
    assert_eq!(
        refused(
            &w,
            AnalyticsView::A,
            AnalyticsFilters {
                buyer: Some(PartyFilter {
                    account_id: w.akram,
                    person_key: Some("x".into())
                }),
                ..filters()
            }
        ),
        [(
            ErrorCode::ErrOutOfRange,
            "filters.buyer.person_key".to_owned()
        )]
    );
    // The page.
    let page = run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            sort: None,
            view: AnalyticsView::A,
            filters: filters(),
            offset: -1,
            limit: 5_000,
        },
    );
    assert!(matches!(
        page,
        Err(mandi_reports::ReportsError::Rejected(items)) if items.len() == 2
    ));
}

#[test]
fn paging_keeps_the_totals_of_the_whole_filter() {
    let w = full_day();
    let full = view(&w, AnalyticsView::C, filters());
    let mut joined = Vec::new();
    for offset in 0..3 {
        let part = run(
            w.env.ctx.db(),
            &AnalyticsRunInput {
                sort: None,
                view: AnalyticsView::C,
                filters: filters(),
                offset,
                limit: 1,
            },
        )
        .unwrap();
        assert_eq!(part.total_count, 3);
        assert_eq!(part.rows.len(), 1);
        assert_eq!(part.totals, full.totals);
        joined.extend(part.rows);
    }
    assert_eq!(joined, full.rows);
}

// ---- Print, saved views, writes, query plans ------------------------------------------------

#[test]
fn print_data_has_every_row_in_both_languages() {
    let w = full_day();
    let data = print_data(
        w.env.ctx.db(),
        &AnalyticsPrintInput {
            sort: None,
            view: AnalyticsView::A,
            filters: AnalyticsFilters {
                compare: AnalyticsCompare::Previous,
                ..on("10-10-2026", "10-10-2026")
            },
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(data.title_id, "analytics-view-a-title");
    assert_eq!(data.row_count, "2");
    let labels: Vec<(&str, &str)> = data
        .filters
        .iter()
        .map(|f| (f.label_id.as_str(), f.value.as_str()))
        .collect();
    assert_eq!(
        labels,
        [
            ("report-filter-from", "10-10-2026"),
            ("report-filter-to", "10-10-2026"),
            ("analytics-filter-compare-from", "09-10-2026"),
            ("analytics-filter-compare-to", "09-10-2026"),
        ]
    );
    let text = |cell: &PrintCell| match cell {
        PrintCell::Money { text } | PrintCell::Number { text } | PrintCell::Percent { text } => {
            text.clone()
        }
        PrintCell::Names { text } => text.clone(),
        PrintCell::Empty => String::new(),
        other => panic!("{other:?}"),
    };
    let banana: Vec<String> = data.rows[0].cells.iter().map(text).collect();
    assert_eq!(
        banana,
        [
            "Banana", "50", "29,400", "588", "1,764", "1,764", "400", "3,928", "72.10", "0",
            "3,928"
        ]
    );
    let totals: Vec<String> = data.totals.iter().map(text).collect();
    assert_eq!(totals[7], "5,448");
    assert_eq!(totals[8], "100.00");
    // Urdu names and a month column in D.
    let ur = print_data(
        w.env.ctx.db(),
        &AnalyticsPrintInput {
            sort: None,
            view: AnalyticsView::D,
            filters: filters(),
            language: Language::Ur,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(
        ur.columns[1].heading,
        Some(mandi_reports::analytics::AnalyticsPrintHeading::Month {
            year: "2026".into(),
            month: 10
        })
    );
    assert_eq!(text(&ur.rows[0].cells[0]), "کیلا");
    // View D says how it groups; the label is the whole fact.
    assert_eq!(
        ur.filters
            .iter()
            .map(|f| (f.label_id.as_str(), f.value.as_str()))
            .collect::<Vec<_>>(),
        [("report-filter-by-month", "")]
    );
}

#[test]
fn saved_views_round_trip_per_user_and_are_checked() {
    let w = full_day();
    let db = w.env.ctx.db();
    let user = w.owner.user_id;
    assert!(views_get(db, user).unwrap().is_empty());
    let saved = views_save(
        db,
        user,
        &AnalyticsViewsSaveInput {
            views: vec![
                AnalyticsSavedView {
                    name: "  کیلا کمیشن ماہانہ ".into(),
                    view: AnalyticsView::D,
                    filters: AnalyticsFilters {
                        item_id: Some(w.banana),
                        ..on("01-01-2026", "31-12-2026")
                    },
                },
                AnalyticsSavedView {
                    name: "Top".into(),
                    view: AnalyticsView::H,
                    filters: filters(),
                },
            ],
        },
    )
    .unwrap();
    assert_eq!(saved[0].name, "کیلا کمیشن ماہانہ", "the name is trimmed");
    assert_eq!(views_get(db, user).unwrap(), saved);
    // Another user sees none of them.
    assert!(views_get(db, user + 1).unwrap().is_empty());
    // Replacing the list replaces the row; an empty list clears it.
    views_save(db, user, &AnalyticsViewsSaveInput { views: Vec::new() }).unwrap();
    assert!(views_get(db, user).unwrap().is_empty());
    // The checks, with their fields.
    let bad = |views: Vec<AnalyticsSavedView>| match views_save(
        db,
        user,
        &AnalyticsViewsSaveInput { views },
    ) {
        Err(mandi_reports::ReportsError::Rejected(items)) => items
            .into_iter()
            .map(|i| (i.code, i.field))
            .collect::<Vec<_>>(),
        other => panic!("{other:?}"),
    };
    let one = |name: &str, f: AnalyticsFilters| AnalyticsSavedView {
        name: name.into(),
        view: AnalyticsView::A,
        filters: f,
    };
    assert_eq!(
        bad(vec![one("   ", filters())]),
        [(ErrorCode::ErrRequired, "views[0].name".to_owned())]
    );
    assert_eq!(
        bad(vec![one(&"ا".repeat(61), filters())]),
        [(ErrorCode::ErrNameTooLong, "views[0].name".to_owned())]
    );
    assert_eq!(
        bad(vec![one("a\u{7}b", filters())]),
        [(ErrorCode::ErrControlChars, "views[0].name".to_owned())]
    );
    assert_eq!(
        bad(vec![
            one("ok", filters()),
            one(
                "gone",
                AnalyticsFilters {
                    item_id: Some(9_999),
                    ..filters()
                }
            )
        ]),
        [(
            ErrorCode::ErrNotFound,
            "views[1].filters.item_id".to_owned()
        )]
    );
    let many: Vec<AnalyticsSavedView> = (0..51).map(|i| one(&format!("v{i}"), filters())).collect();
    assert_eq!(bad(many), [(ErrorCode::ErrOutOfRange, "views".to_owned())]);
    assert!(
        views_get(db, user).unwrap().is_empty(),
        "nothing was stored by a refusal"
    );
}

#[test]
fn analytics_writes_nothing_of_the_books() {
    let w = full_day();
    let count = |table: &str| -> i64 {
        raw(&w.dir)
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    };
    let tables = [
        "analytics_facts",
        "journal_entries",
        "journal_lines",
        "audit_log",
        "bikri_sales",
        "partner_share_rows",
        "stock_moves",
        "bill_allocations",
        "advance_allocations",
        "user_prefs",
    ];
    let before: Vec<i64> = tables.iter().map(|t| count(t)).collect();
    for v in [
        AnalyticsView::A,
        AnalyticsView::B,
        AnalyticsView::C,
        AnalyticsView::D,
        AnalyticsView::E,
        AnalyticsView::H,
    ] {
        view(&w, v, filters());
        lines_for(&w, v, filters(), AnalyticsKey::default(), None);
        print_data(
            w.env.ctx.db(),
            &AnalyticsPrintInput {
                sort: None,
                view: v,
                filters: filters(),
                language: Language::Ur,
                paper: None,
            },
        )
        .unwrap();
    }
    let after: Vec<i64> = tables.iter().map(|t| count(t)).collect();
    assert_eq!(before, after);
    assert_chain_ok(&w.dir);
}

#[test]
fn the_fact_queries_use_the_date_index() {
    // Code Standards B18: with a date range the facts are found through the date index. Without
    // a range a scan is the only way, so only the dated queries are checked.
    use mandi_reports::analytics::plans::{view_a, view_b};
    let w = full_day();
    let conn = raw(&w.dir);
    let plan = |query: (String, Vec<mandi_services::db::rusqlite::types::Value>)| -> String {
        let mut statement = conn
            .prepare(&format!("EXPLAIN QUERY PLAN {}", query.0))
            .unwrap();
        let rows = statement
            .query_map(
                mandi_services::db::rusqlite::params_from_iter(query.1.iter()),
                |r| r.get::<_, String>(3),
            )
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        rows.join(" | ")
    };
    let day = Some("2026-10-10");
    for query in [view_a(day, day), view_b(day, day)] {
        let text = plan(query);
        assert!(text.contains("idx_analytics_facts_date"), "{text}");
    }
}

// ---- Random sequences of postings, edits and cancels ----------------------------------------

/// A small deterministic random source, so a failing sequence can be run again.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, below: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % below
    }
}

/// The columns of a view-A total row that add up across periods: نگ, خام بکری, the three
/// commissions and the total.
fn a_sums(w: &support::World, f: AnalyticsFilters) -> [i64; 6] {
    let page = view(w, AnalyticsView::A, f);
    let get = |index: usize| match &page.totals[index] {
        Cell::Number { value } => *value,
        other => money(other),
    };
    [get(1), get(2), get(4), get(5), get(6), get(7)]
}

#[test]
fn every_view_adds_up_after_random_postings_edits_and_cancels() {
    let (mut edits, mut cancels) = (0_u32, 0_u32);
    for seed in 0..6_u64 {
        let mut rng = Lcg(seed.wrapping_mul(7_919).wrapping_add(11));
        let w = worked_day();
        // (bikri id, first entry id, edited, cancelled)
        let mut made: Vec<(i64, i64, bool, bool)> = Vec::new();
        for step in 0..6_u32 {
            let day = 10 + step / 2;
            set_today(&w.env, 2026, 10, day);
            let live: Vec<usize> = (0..made.len()).filter(|i| !made[*i].3).collect();
            let action = rng.next(100);
            if action < 60 || live.is_empty() {
                let (qa, qb) = (1 + rng.next(15), 1 + rng.next(15));
                let sellers = [w.rahim, w.karim];
                let items = [w.banana, w.apple, w.mooli];
                let seller = sellers[rng.next(2) as usize];
                let item = items[rng.next(3) as usize];
                let lot = support::with_sales(
                    support::bare_lot(&w, "lot-1", seller, item, &(qa + qb).to_string()),
                    vec![
                        support::sale(
                            "s-1",
                            w.akram,
                            &qa.to_string(),
                            &(100 + rng.next(800)).to_string(),
                        ),
                        support::sale(
                            "s-2",
                            w.arshad,
                            &qb.to_string(),
                            &(100 + rng.next(800)).to_string(),
                        ),
                    ],
                );
                let mut draft = support::draft(vec![lot]);
                draft.header.entry_date = format!("{day:02}-10-2026");
                draft.header.truck_no = Some(format!("T-{seed}-{step}"));
                let saved = save(&w, &draft);
                made.push((saved.bikri_id, saved.entry_id, false, false));
            } else if action < 85 {
                let pick = live[rng.next(live.len() as u64) as usize];
                if made[pick].2 {
                    continue;
                }
                let opened = bikri::open_for_edit(
                    &w.env.ctx,
                    &EditOpenInput {
                        bikri_id: made[pick].0,
                        entry_id: made[pick].1,
                    },
                )
                .unwrap();
                let mut draft = opened.draft.clone();
                draft.lots[0].sales[0].seller_rate = (100 + rng.next(800)).to_string();
                bikri::edit_save(
                    &w.env.ctx,
                    &w.owner,
                    &EditSaveInput {
                        entry_id: opened.entry_id,
                        version: opened.version,
                        draft,
                        reason: Some("ریٹ کی درستی".into()),
                        confirm_cash_below_zero: false,
                    },
                )
                .unwrap();
                made[pick].2 = true;
                edits += 1;
            } else {
                let pick = live[rng.next(live.len() as u64) as usize];
                let detail = bikri::get(&w.env.ctx, made[pick].0).unwrap();
                bikri::cancel(
                    &w.env.ctx,
                    &w.owner,
                    &CancelInput {
                        id: made[pick].0,
                        version: detail.version,
                        reason: "غلط اندراج".into(),
                        confirm_cash_below_zero: true,
                    },
                )
                .unwrap();
                made[pick].3 = true;
                cancels += 1;
            }
        }
        // Journal, R7 and the other views agree (reversal rows copy date and dimensions, so no
        // group goes below 0 and the run itself would fail otherwise).
        assert_consistent(&w);
        // The lines behind each row of A, B and C add up to the row.
        let a = view(&w, AnalyticsView::A, filters());
        for row in data_rows(&a) {
            let page = lines_for(
                &w,
                AnalyticsView::A,
                filters(),
                row.key.clone().unwrap(),
                None,
            );
            assert_eq!(money(&page.totals[11]), money(&row.cells[7]), "seed {seed}");
            assert_eq!(num(&page.totals[5]), num(&row.cells[1]), "seed {seed}");
        }
        // Splitting the period at any day adds up, on every column and on the seller net.
        let whole = a_sums(&w, on("10-10-2026", "12-10-2026"));
        for split in ["10-10-2026", "11-10-2026"] {
            let next = if split == "10-10-2026" {
                "11-10-2026"
            } else {
                "12-10-2026"
            };
            let (first, second) = (
                a_sums(&w, on("10-10-2026", split)),
                a_sums(&w, on(next, "12-10-2026")),
            );
            for i in 0..6 {
                assert_eq!(
                    first[i] + second[i],
                    whole[i],
                    "seed {seed} split {split} column {i}"
                );
            }
            let nets = |f: AnalyticsFilters| match &view(&w, AnalyticsView::B, f).totals[7] {
                Cell::Money { paisa } => paisa.rupees(),
                _ => 0,
            };
            assert_eq!(
                nets(on("10-10-2026", split)) + nets(on(next, "12-10-2026")),
                nets(on("10-10-2026", "12-10-2026")),
                "seed {seed} split {split}: seller net"
            );
        }
        // Over the whole range the seller net is the sum of the stored net of the saved lots
        // that have sales.
        let nets_all = match &view(&w, AnalyticsView::B, filters()).totals[7] {
            Cell::Money { paisa } => paisa.rupees(),
            _ => 0,
        };
        let stored: i64 = raw(&w.dir)
            .query_row(
                "SELECT COALESCE(SUM(l.net_paisa), 0) FROM bikri_lots l
                 JOIN bikri_entries b ON b.id = l.bikri_id WHERE b.status = 'saved'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            nets_all,
            stored / 100,
            "seed {seed}: net against the stored lots"
        );
    }
    // The sequences really had edits and cancels in them.
    assert!(edits > 0 && cancels > 0, "edits {edits}, cancels {cancels}");
}

// ---- The widest matrices --------------------------------------------------------------------

/// Copies the first fact to another item, so a matrix has many columns.
fn copy_fact(w: &support::World, item: i64, date: &str) {
    raw(&w.dir)
        .execute(
            "INSERT INTO analytics_facts (created_at, created_by, updated_at, updated_by,
                 journal_entry_id, entry_date, year, month, bikri_id, lot_id, sale_id, item_id,
                 seller_account_id, buyer_account_id, pukhta_person_key, city_id, adda_id, qty,
                 seller_amount_paisa, seller_comm_paisa, buyer_value_paisa, buyer_comm_paisa,
                 laga_paisa, sign, purchase_id, venture_id, cost_paisa)
             SELECT created_at, created_by, updated_at, updated_by, journal_entry_id, ?2,
                 CAST(substr(?2, 1, 4) AS INTEGER), CAST(substr(?2, 6, 2) AS INTEGER), bikri_id,
                 lot_id, sale_id, ?1, seller_account_id, buyer_account_id, pukhta_person_key,
                 city_id, adda_id, qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa,
                 buyer_comm_paisa, laga_paisa, sign, purchase_id, venture_id, cost_paisa
             FROM analytics_facts ORDER BY id LIMIT 1",
            mandi_services::db::rusqlite::params![item, date],
        )
        .unwrap();
}

#[test]
fn view_e_collapses_the_items_beyond_the_column_limit_into_one_column() {
    let w = full_day();
    for n in 0..25 {
        let item = support::add_item(
            &w.env,
            &w.owner,
            &format!("قسم {n}"),
            &format!("Kind {n}"),
            "fruit",
            "unit",
        );
        copy_fact(&w, item, "2026-10-10");
    }
    let e = view(&w, AnalyticsView::E, filters());
    // seller + 23 items + "other items" + total.
    assert_eq!(e.columns.len(), 26);
    let other = &e.columns[24];
    assert!(matches!(
        other.heading,
        Some(mandi_reports::analytics::AnalyticsHeading::OtherItems)
    ));
    assert_eq!(other.drill.as_ref().unwrap().not_item_ids.len(), 23);
    // Every row adds up: its cells are its total, and the grand total is the sum of the facts.
    for row in data_rows(&e) {
        let cells: i64 = row.cells[1..25].iter().map(money).sum();
        assert_eq!(cells, money(row.cells.last().unwrap()));
    }
    let column_sum: i64 = e.totals[1..25].iter().map(money).sum();
    assert_eq!(column_sum, money(e.totals.last().unwrap()));
    // The lines of the "other items" cell add up to its cell.
    let first = data_rows(&e)[0];
    let page = lines_for(
        &w,
        AnalyticsView::E,
        filters(),
        first.key.clone().unwrap(),
        other.drill.clone(),
    );
    assert_eq!(money(&page.totals[11]), money(&first.cells[24]));
    assert!(money(&first.cells[24]) > 0);
}

#[test]
fn view_d_refuses_more_than_36_columns_and_a_coarser_grouping_works() {
    let w = full_day();
    for n in 0..37_u32 {
        let (year, month) = (2020 + n / 12, 1 + n % 12);
        copy_fact(&w, w.banana, &format!("{year}-{month:02}-15"));
    }
    assert_eq!(
        refused(&w, AnalyticsView::D, filters()),
        [(ErrorCode::ErrOutOfRange, "filters.group_by".to_owned())]
    );
    let by_year = view(
        &w,
        AnalyticsView::D,
        AnalyticsFilters {
            group_by: Some(AnalyticsGroupBy::Year),
            ..filters()
        },
    );
    // 2020 to 2023 and 2026: five years.
    assert_eq!(by_year.columns.len(), 7);
    let bounded = view(&w, AnalyticsView::D, on("01-10-2026", "31-10-2026"));
    assert_eq!(bounded.columns.len(), 3);
}
