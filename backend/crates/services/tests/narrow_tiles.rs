//! The narrow reads of the dashboard tiles (plan 48 P48-S1, NFR-AI-3, NFR-SPD-2, PRD FR-6.1,
//! FR-19.9). The assistant's quick chips and its `tiles` tool read one tile, or all the tiles
//! without the ages of the buyers' bills, instead of the whole `dashboard::get`. Each narrow
//! function must give exactly the figure of the tile of `get`, and R20 without ages must be R20
//! row for row with the age cells empty.
//!
//! The expected numbers are the PRD's (4.9, E2E-8, E2E-15, E2E-21, E2E-28, E2E-32, E2E-34, E2E-40)
//! and the cases of the money review of P48-S1 (a buyer who paid in advance, a dead mark and its
//! undo, a چکتا buyer, cash below zero, a seller with an advance and a zero balance).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_reports::reports::{
    Cell, ReportFilters, ReportId, ReportPage, ReportRunInput, run_dues_without_ages,
};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{ChuktaInput, LineChargeMethod};
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::dead::{DeadMarkInput, DeadUndoInput};
use mandi_services::core::money::Paisa;
use mandi_services::{bikri, dead, vouchers};
use reports_world::*;
use serde_json::json;
use support::World;

fn off() -> BackupFacts {
    BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    }
}

fn db(w: &World) -> &mandi_services::db::Db {
    w.env.ctx.db()
}

/// Today as the dashboard reads it.
fn today_of(w: &World) -> BusinessDate {
    BusinessDate::parse_iso(&dashboard::get(db(w), off()).unwrap().today).unwrap()
}

fn rs(p: Paisa) -> i64 {
    p.rupees()
}

fn column(page: &ReportPage, key: &str) -> usize {
    page.columns
        .iter()
        .position(|c| c.key == key)
        .unwrap_or_else(|| panic!("no column {key}"))
}

fn total_of(page: &ReportPage, key: &str) -> Cell {
    page.totals[column(page, key)].clone()
}

fn dues_without_ages(w: &World, filters: ReportFilters) -> ReportPage {
    run_dues_without_ages(
        db(w),
        &ReportRunInput {
            id: ReportId::R20,
            filters,
            offset: 0,
            limit: 1_000,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap()
}

/// Every narrow function is the tile of `get`, and R20 without ages is R20 without its age cells.
/// Gives the tiles of `get` so a test can also hold them to the PRD's numbers.
fn narrow_is_the_dashboard(w: &World) -> mandi_reports::dashboard::DashboardTiles {
    let d = dashboard::get(db(w), off()).unwrap();
    let today = BusinessDate::parse_iso(&d.today).unwrap();
    let t = d.tiles;
    let now = dashboard::tiles_now(db(w)).unwrap();
    assert_eq!(now.today, d.today);
    assert_eq!(now.tiles, t, "tiles_now is the tiles of get");

    assert_eq!(
        dashboard::cash_in_hand(db(w)).unwrap(),
        t.cash_in_hand_paisa
    );
    let sales = dashboard::sales_today(db(w), today).unwrap();
    assert_eq!(
        (sales.gross_paisa, sales.count, sales.commission_paisa),
        (t.sales_gross_paisa, t.sales_count, t.commission_paisa)
    );
    assert_eq!(
        dashboard::stock(db(w), today).unwrap(),
        (t.stock_lots, t.stock_qty)
    );
    assert_eq!(
        dashboard::buyers_dues(db(w), today).unwrap(),
        t.buyers_dues_paisa
    );
    let sellers = dashboard::sellers(db(w), today).unwrap();
    assert_eq!(
        (
            sellers.payable_paisa,
            sellers.advances_open_paisa,
            sellers.dues_paisa
        ),
        (
            t.sellers_payable_paisa,
            t.advances_open_paisa,
            t.sellers_dues_paisa
        )
    );
    assert_eq!(
        dashboard::dead(db(w)).unwrap(),
        (t.dead_amount_paisa, t.dead_count)
    );
    let parts = dashboard::total_dues(db(w), today).unwrap();
    assert_eq!(
        (
            parts.buyers_paisa,
            parts.sellers_paisa,
            parts.dead_paisa,
            parts.total_paisa
        ),
        (
            t.buyers_dues_paisa,
            t.sellers_dues_paisa,
            t.dead_amount_paisa,
            t.total_dues_paisa
        )
    );
    // The total is the three parts added.
    assert_eq!(
        parts.total_paisa,
        parts
            .buyers_paisa
            .checked_add(parts.sellers_paisa)
            .unwrap()
            .checked_add(parts.dead_paisa)
            .unwrap()
    );

    // R20, row for row: the same buyers in the same order with the same balances; the narrow
    // rows carry no ages; the total row has the same balance.
    let full = report(w, ReportId::R20, ReportFilters::default(), 0, 1_000);
    let narrow = dues_without_ages(w, ReportFilters::default());
    assert_eq!(full.total_count, narrow.total_count);
    assert_eq!(full.rows.len(), narrow.rows.len());
    let (buyer, balance) = (column(&full, "buyer"), column(&full, "balance"));
    for (a, b) in full.rows.iter().zip(&narrow.rows) {
        assert_eq!(a.cells[buyer], b.cells[buyer]);
        assert_eq!(a.cells[balance], b.cells[balance]);
        assert_eq!(a.source, b.source);
        for key in [
            "oldest_date",
            "days",
            "age_0_7",
            "age_8_30",
            "age_31_90",
            "age_91",
        ] {
            assert_eq!(b.cells[column(&narrow, key)], Cell::Empty, "{key}");
        }
    }
    assert_eq!(total_of(&full, "balance"), total_of(&narrow, "balance"));
    // The second place of the tool reads R20 as of today: the same total as the tile.
    let dated = dues_without_ages(
        w,
        ReportFilters {
            as_of: d.today.clone(),
            ..ReportFilters::default()
        },
    );
    assert_eq!(
        total_of(&dated, "balance"),
        Cell::Money {
            paisa: t.buyers_dues_paisa
        }
    );
    // R21 and R22 without ages: the same balance total as the full reports.
    for id in [ReportId::R21, ReportId::R22] {
        let full = report(w, id, ReportFilters::default(), 0, 1_000);
        let narrow = run_dues_without_ages(
            db(w),
            &ReportRunInput {
                id,
                filters: ReportFilters::default(),
                offset: 0,
                limit: 1_000,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap();
        assert_eq!(full.total_count, narrow.total_count, "{id:?}");
        assert_eq!(
            total_of(&full, "balance"),
            total_of(&narrow, "balance"),
            "{id:?}"
        );
    }
    assert_eq!(
        total_of(&narrow, "balance"),
        Cell::Money {
            paisa: t.buyers_dues_paisa
        }
    );
    // R24: the tile's three sums are the totals of the report.
    let r24 = report(w, ReportId::R24, ReportFilters::default(), 0, 1_000);
    assert_eq!(
        total_of(&r24, "payable"),
        Cell::Money {
            paisa: t.sellers_payable_paisa
        }
    );
    assert_eq!(
        total_of(&r24, "advance_open"),
        Cell::Money {
            paisa: t.advances_open_paisa
        }
    );
    assert_eq!(
        total_of(&r24, "owes_firm"),
        Cell::Money {
            paisa: t.sellers_dues_paisa
        }
    );
    t
}

#[test]
fn the_worked_day_is_the_prd_4_9_in_every_narrow_read() {
    let w = full_day();
    let t = narrow_is_the_dashboard(&w);
    assert_eq!(rs(dashboard::cash_in_hand(db(&w)).unwrap()), 58_484);
    let sales = dashboard::sales_today(db(&w), today_of(&w)).unwrap();
    assert_eq!(
        (
            rs(sales.gross_paisa),
            sales.count,
            rs(sales.commission_paisa)
        ),
        (41_400, 2, 5_448),
        "the commission is 5,448, not the shared income 5,488"
    );
    assert_eq!(
        rs(dashboard::buyers_dues(db(&w), today_of(&w)).unwrap()),
        21_564
    );
    let sellers = dashboard::sellers(db(&w), today_of(&w)).unwrap();
    assert_eq!(
        (
            rs(sellers.payable_paisa),
            rs(sellers.advances_open_paisa),
            rs(sellers.dues_paisa)
        ),
        (10_160, 0, 0)
    );
    assert_eq!(dashboard::stock(db(&w), today_of(&w)).unwrap(), (0, 0));
    let parts = dashboard::total_dues(db(&w), today_of(&w)).unwrap();
    assert_eq!(
        (
            rs(parts.buyers_paisa),
            rs(parts.dead_paisa),
            rs(parts.total_paisa)
        ),
        (21_564, 0, 21_564)
    );
    assert_eq!(rs(t.expenses_today_paisa), 3_000);
}

/// E2E-36 and E2E-43 (PRD 27.2): the narrow reads give the figures of the day summary (R34) and
/// of the party day sheet (R35) for 10-10-26, which the PRD fixes: 41,400 sold in 2 بکری, commission
/// 5,448, cash 58,484, buyers' dues 21,564 (0 + 44,364 − 22,800).
#[test]
fn e2e_36_and_e2e_43_the_day_summary_and_the_party_sheet_equal_the_narrow_reads() {
    let w = full_day();
    narrow_is_the_dashboard(&w);
    let today = today_of(&w);

    // E2E-36: the rows of R34 for today.
    let r34 = report(&w, ReportId::R34, ReportFilters::default(), 0, 0);
    let amount_of = |label: &str| -> i64 {
        r34.rows
            .iter()
            .find(|r| matches!(&r.cells[1], Cell::Label { id } if id == label))
            .map(|r| money(&r.cells[3]))
            .unwrap_or_else(|| panic!("no row {label}"))
    };
    let sales = dashboard::sales_today(db(&w), today).unwrap();
    assert_eq!(
        (amount_of("report-r34-bikri"), rs(sales.gross_paisa)),
        (41_400, 41_400)
    );
    assert_eq!(
        (
            amount_of("report-r34-commission"),
            rs(sales.commission_paisa)
        ),
        (5_448, 5_448)
    );
    let cash = dashboard::cash_in_hand(db(&w)).unwrap();
    assert_eq!(
        (amount_of("report-r34-cash-closing"), rs(cash)),
        (58_484, 58_484)
    );
    let dues = dashboard::buyers_dues(db(&w), today).unwrap();
    assert_eq!(
        (amount_of("report-r34-buyers-dues"), rs(dues)),
        (21_564, 21_564)
    );

    // E2E-43: the all-buyers heading of R35 remains 21,564, the total of R20 and the narrow dues.
    let r35 = report(
        &w,
        ReportId::R35,
        ReportFilters {
            as_of: "10-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(
        balance(&r35.rows[0].cells[5]),
        (BalanceSide::Dr, rs(dues)),
        "the buyers' remaining is the narrow dues"
    );
    // The whole line of E2E-43: previous 0 + bought 44,364 - paid 22,800 = remaining 21,564.
    assert_eq!(balance(&r35.rows[0].cells[2]).1, 0, "previous balance");
    assert_eq!(
        (money(&r35.rows[0].cells[3]), money(&r35.rows[0].cells[4])),
        (44_364, 22_800)
    );
    // The same figure from the narrow R20 total and the total of the three parts.
    let r20 = dues_without_ages(&w, ReportFilters::default());
    assert_eq!(money(&total_of(&r20, "balance")), rs(dues));
    assert_eq!(
        rs(dashboard::total_dues(db(&w), today).unwrap().total_paisa),
        21_564
    );
}

#[test]
fn e2e_32_cash_lent_to_a_buyer_is_a_due_and_leaves_the_cash() {
    let w = full_day();
    let mut lend = payment(w.akram, "3,000");
    lend.reason = "نقد ادھار".into();
    vouchers::save_payment(&w.env.ctx, &w.owner, &lend).unwrap();
    narrow_is_the_dashboard(&w);
    assert_eq!(
        rs(dashboard::buyers_dues(db(&w), today_of(&w)).unwrap()),
        24_564
    );
    assert_eq!(rs(dashboard::cash_in_hand(db(&w)).unwrap()), 55_484);
}

#[test]
fn a_buyer_who_paid_in_advance_owes_nothing_and_takes_nothing_off_the_dues() {
    // اکرم owes 2,880 and pays 20,000: he is owed 17,120. He is not a buyer who owes, so the dues
    // are ارشد's 18,684 alone; his credit does not reduce them.
    let w = full_day();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "20,000")).unwrap();
    narrow_is_the_dashboard(&w);
    assert_eq!(
        rs(dashboard::buyers_dues(db(&w), today_of(&w)).unwrap()),
        18_684
    );
    assert_eq!(rs(dashboard::cash_in_hand(db(&w)).unwrap()), 78_484);
}

#[test]
fn a_regular_buyer_in_the_dead_amount_leaves_the_dues_and_comes_back_with_the_undo() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    let mark: DeadMarkInput = serde_json::from_value(json!({
        "account_id": w.arshad,
        "entry_date": "11-10-2026", "reason": "died", "note": "فوت ہو گئے"
    }))
    .unwrap();
    let marked = dead::mark(&w.env.ctx, &w.owner, &mark).unwrap();
    narrow_is_the_dashboard(&w);
    let parts = dashboard::total_dues(db(&w), today_of(&w)).unwrap();
    assert_eq!(
        (
            rs(parts.buyers_paisa),
            rs(parts.dead_paisa),
            rs(parts.total_paisa)
        ),
        (2_880, 18_684, 21_564),
        "the total is the three parts, nobody counted twice"
    );
    let undo: DeadUndoInput =
        serde_json::from_value(json!({ "mark_id": marked.mark_id, "reason": "غلط اندراج" }))
            .unwrap();
    dead::undo(&w.env.ctx, &w.owner, &undo).unwrap();
    narrow_is_the_dashboard(&w);
    let parts = dashboard::total_dues(db(&w), today_of(&w)).unwrap();
    assert_eq!((rs(parts.buyers_paisa), rs(parts.dead_paisa)), (21_564, 0));
}

/// E2E-21 (v1.32): شاہد 1 × 1,000 and ناصر 2 × 1,000, both چکتا buyers, each paying in full on the
/// spot. No commission and no لاگا.
fn two_chukta_buyers() -> World {
    let w = worked_day();
    let line = |key: &str, qty: &str, ur: &str, en: &str, phone: &str| {
        let mut l = support::sale(key, w.chukta_khata, qty, "1000");
        l.chukta = Some(ChuktaInput {
            name_ur: ur.into(),
            name_en: en.into(),
            phone: phone.into(),
        });
        l.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
        l.seller_comm = support::typed(LineChargeMethod::Percent, "0");
        l.laga = support::typed(LineChargeMethod::PerUnit, "0");
        l
    };
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "3");
    lot.sales = vec![
        line("s-1", "1", "شاہد", "Shahid", "0300-0000000"),
        line("s-2", "2", "ناصر", "Nasir", "0300-1111111"),
    ];
    save(&w, &support::draft(vec![lot]));
    w
}

#[test]
fn chukta_buyers_are_in_no_dues_and_the_narrow_tiles_still_equal_the_dashboard() {
    let w = two_chukta_buyers();
    narrow_is_the_dashboard(&w);
    // They paid in full, so nobody owes (PRD FR-13.6).
    assert_eq!(rs(dashboard::buyers_dues(db(&w), today_of(&w)).unwrap()), 0);
    let parts = dashboard::total_dues(db(&w), today_of(&w)).unwrap();
    assert_eq!(
        (
            rs(parts.buyers_paisa),
            rs(parts.dead_paisa),
            rs(parts.total_paisa)
        ),
        (0, 0, 0)
    );
}

#[test]
fn e2e_15_a_cancelled_bikri_is_out_of_the_sales_and_the_dues_by_its_reversal() {
    let w = worked_day();
    save(&w, &support::event1(&w));
    let two = save(&w, &event2(&w));
    let version = bikri::get(&w.env.ctx, two.bikri_id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::CancelInput {
            id: two.bikri_id,
            version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    narrow_is_the_dashboard(&w);
    let sales = dashboard::sales_today(db(&w), today_of(&w)).unwrap();
    assert_eq!(
        (
            rs(sales.gross_paisa),
            sales.count,
            rs(sales.commission_paisa)
        ),
        (29_400, 1, 3_928),
        "the cancelled بکری adds nothing"
    );
    assert_eq!(
        rs(dashboard::buyers_dues(db(&w), today_of(&w)).unwrap()),
        31_564
    );
}

#[test]
fn cash_below_zero_is_negative_and_is_the_signed_closing_of_the_cash_book() {
    // FR-6.7: a payment of 100,000 to کریم that the Owner confirms takes the cash below 0.
    let w = full_day();
    let mut big = payment(w.karim, "100,000");
    big.confirm_cash_below_zero = true;
    vouchers::save_payment(&w.env.ctx, &w.owner, &big).unwrap();
    narrow_is_the_dashboard(&w);
    let cash = dashboard::cash_in_hand(db(&w)).unwrap();
    assert_eq!(rs(cash), 58_484 - 100_000);
    let r12 = report(
        &w,
        ReportId::R12,
        ReportFilters {
            from: "10-10-2026".into(),
            to: "10-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        1,
    );
    let closing = r12
        .summary
        .iter()
        .find(|l| l.label_id == "report-closing-balance")
        .expect("the closing balance");
    assert_eq!(
        closing.cell,
        Cell::Balance {
            paisa: Paisa::from_rupees(41_516).unwrap(),
            side: BalanceSide::Cr
        }
    );
}

#[test]
fn e2e_28_an_advance_that_stays_open_and_a_seller_with_an_advance_and_no_balance() {
    // The tick off: رحیم's whole 19,316 is payable and his 15,000 advance stays open.
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].adjust_advance = false;
    save(&w, &draft);
    narrow_is_the_dashboard(&w);
    let sellers = dashboard::sellers(db(&w), today_of(&w)).unwrap();
    assert_eq!(
        (rs(sellers.payable_paisa), rs(sellers.advances_open_paisa)),
        (19_316, 15_000)
    );
    // After he is paid the 19,316 his balance is the advance alone: 15,000 بنام, advance open,
    // and he is not in what sellers owe beyond their advances.
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "19,316")).unwrap();
    narrow_is_the_dashboard(&w);
    let sellers = dashboard::sellers(db(&w), today_of(&w)).unwrap();
    assert_eq!(
        (
            rs(sellers.payable_paisa),
            rs(sellers.advances_open_paisa),
            rs(sellers.dues_paisa)
        ),
        (0, 15_000, 0)
    );
}

#[test]
fn a_seller_overpaid_owes_the_firm_and_is_in_the_sellers_dues() {
    // کریم is owed 10,160 and is paid 12,000: he owes the firm 1,840 beyond any advance.
    let w = full_day();
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.karim, "12,000")).unwrap();
    narrow_is_the_dashboard(&w);
    let sellers = dashboard::sellers(db(&w), today_of(&w)).unwrap();
    assert_eq!(rs(sellers.dues_paisa), 1_840);
    let parts = dashboard::total_dues(db(&w), today_of(&w)).unwrap();
    assert_eq!(
        (
            rs(parts.buyers_paisa),
            rs(parts.sellers_paisa),
            rs(parts.total_paisa)
        ),
        (21_564, 1_840, 23_404)
    );
}

#[test]
fn a_part_sold_lot_is_stock_in_the_narrow_read() {
    // E2E-16: 45 of 50 sold, 5 نگ left.
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    save(&w, &draft);
    narrow_is_the_dashboard(&w);
    assert_eq!(dashboard::stock(db(&w), today_of(&w)).unwrap(), (1, 5));
}

#[test]
fn the_narrow_dues_are_the_dues_of_r20_when_the_bills_are_old() {
    // 120 days later every bill is over 90 days old: R20 puts them in the last band, the narrow
    // read gives the same buyers and balances and no ages.
    let w = full_day();
    set_today(&w.env, 2027, 2, 7);
    narrow_is_the_dashboard(&w);
    let full = report(&w, ReportId::R20, ReportFilters::default(), 0, 1_000);
    assert_eq!(
        total_of(&full, "age_91"),
        Cell::Money {
            paisa: Paisa::from_rupees(21_564).unwrap()
        },
        "the full report has the ages the narrow one leaves out"
    );
    assert_eq!(
        total_of(&dues_without_ages(&w, ReportFilters::default()), "age_91"),
        Cell::Empty
    );
}

#[test]
fn trips_gardens_and_the_other_tiles_are_the_tiles_of_get() {
    // E2E-34: the trip's purchase cost 10,400 comes out of the cash (50,000 - 10,400 = 39,600);
    // 15 of 20 are sold to اکرم, who pays 9,660; 5 نگ are left with cost 2,600.
    let (w, _, _, _) = trip_34_world();
    let t = narrow_is_the_dashboard(&w);
    assert_eq!(rs(t.cash_in_hand_paisa), 39_600);
    assert_eq!(rs(t.buyers_dues_paisa), 9_660);
    assert_eq!((t.trips_qty_left, rs(t.trips_cost_left_paisa)), (5, 2_600));
    // E2E-40 before the closing: الف pays 33,000 from the shop's cash (50,000 - 33,000 = 17,000),
    // the other shop brings 10,000, so the cash is 27,000; ارشد owes 64,400 for the 100 پیٹی.
    let (w, _garden) = garden_40();
    let t = narrow_is_the_dashboard(&w);
    assert_eq!(rs(t.cash_in_hand_paisa), 27_000);
    assert_eq!(rs(t.buyers_dues_paisa), 64_400);
}

#[test]
fn only_r20_to_r22_without_a_sort_can_be_read_without_ages() {
    let w = full_day();
    let ask = |id: ReportId, sort: bool| {
        run_dues_without_ages(
            db(&w),
            &ReportRunInput {
                id,
                filters: ReportFilters::default(),
                offset: 0,
                limit: 10,
                sort: sort.then(|| mandi_services::core::paging::SortInput {
                    column: "balance".into(),
                    direction: mandi_services::core::paging::SortDirection::Desc,
                    language: None,
                }),
                group_by_month: false,
            },
        )
    };
    assert!(ask(ReportId::R20, false).is_ok());
    assert!(ask(ReportId::R21, false).is_ok());
    assert!(ask(ReportId::R22, false).is_ok());
    assert!(ask(ReportId::R24, false).is_err());
    assert!(ask(ReportId::R20, true).is_err());
}
