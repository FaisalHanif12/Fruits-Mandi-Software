//! The dashboard and the attention list of plan d5-late, round 9 (PRD 9.1 to 9.4, FR-9.1,
//! FR-7.44, FR-7.45, FR-6.1, FR-19.9, NFR-OBS-5), on the PRD worked day.
//!
//! The expected numbers are the PRD's (4.9, E2E-8, E2E-16, E2E-28, E2E-30b, E2E-32, E2E-35,
//! E2E-37) and the worked cases of the money review of round 9 (the age of a seller's payable).
//! Every tile is also held against the report or the screen that shows the same figure (R20, R24,
//! R26, R34, the trips tile, the partner overview), so the dashboard can never disagree with the
//! books (PRD FR-6.1, FR-19.9).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::dashboard::{
    self, AttentionKind, AttentionList, AttentionTarget, BackupFacts, Dashboard,
};
use mandi_reports::reports::{Cell, ReportFilters, ReportId};
use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
use mandi_services::core::capital::CapitalInput;
use mandi_services::core::drafts::DraftSaveInput;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::expenses::PersonalExpenseInput;
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
use mandi_services::core::lent::{
    LentPersonCreateInput, LentReturnMode, MoneyLentInput, MoneyLentReturnInput,
};
use mandi_services::core::purchases::PurchaseInput;
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{EntryPartInput, MoneySource, VenturePartnerInput};
use mandi_services::{
    ServiceError, accounts, drafts, lent, partners, purchases, screen_prefs, trips, vouchers,
};
use reports_world::*;
use support::World;

fn off() -> BackupFacts {
    BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    }
}

fn dash(w: &World) -> Dashboard {
    dashboard::get(w.env.ctx.db(), off()).unwrap()
}

fn bell(w: &World, backup: BackupFacts) -> AttentionList {
    dashboard::attention(w.env.ctx.db(), backup).unwrap()
}

/// The kinds on the bell with their counts.
fn kinds(list: &AttentionList) -> Vec<(AttentionKind, i64)> {
    list.items.iter().map(|i| (i.kind, i.count)).collect()
}

fn item(list: &AttentionList, kind: AttentionKind) -> &mandi_reports::dashboard::AttentionItem {
    list.items
        .iter()
        .find(|i| i.kind == kind)
        .unwrap_or_else(|| panic!("no {kind:?} on the bell: {list:?}"))
}

fn rupees(p: mandi_services::core::money::Paisa) -> i64 {
    p.rupees()
}

/// Repeats a save that asks to confirm rule-made spellings, confirming what it asks.
fn confirming<T>(mut attempt: impl FnMut(&[String]) -> Result<T, ServiceError>) -> T {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        match attempt(&words) {
            Ok(done) => return done,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

/// A جنرل entry that debits the cash account and credits a seller: a payable posting of his on
/// that day (PRD FR-7.59). It never adjusts an advance (PRD 16.4).
fn credit_seller(w: &World, seller: i64, amount: &str, date: &str) {
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
            date: date.into(),
            narration: None,
            lines: vec![
                line(support::id_of(&w.env, "01010001"), amount, ""),
                line(seller, "", amount),
            ],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

fn pay_seller(w: &World, seller: i64, amount: &str, date: &str) {
    let mut p = payment(seller, amount);
    p.date = date.into();
    vouchers::save_payment(&w.env.ctx, &w.owner, &p).unwrap();
}

/// Runs one SQL statement on the test book (a setting the screens do not offer).
fn sql(w: &World, statement: &str, id: i64) {
    w.env
        .ctx
        .db()
        .write(|tx| {
            tx.execute(statement, [id])
                .map_err(mandi_services::db::DbError::from)
        })
        .unwrap();
}

fn totals_money(page: &mandi_reports::reports::ReportPage, at: usize) -> i64 {
    page.totals.get(at).map_or(0, money)
}

// ---- The worked day ----------------------------------------------------------------------------

#[test]
fn the_worked_day_shows_the_prd_4_9_numbers() {
    let w = full_day();
    let d = dash(&w);
    let t = d.tiles;
    assert_eq!(d.today, "2026-10-10");
    assert_eq!((rupees(t.sales_gross_paisa), t.sales_count), (41_400, 2));
    assert_eq!(rupees(t.commission_paisa), 5_448);
    assert_eq!(rupees(t.cash_in_hand_paisa), 58_484);
    assert_eq!(rupees(t.buyers_dues_paisa), 21_564);
    assert_eq!(rupees(t.sellers_payable_paisa), 10_160);
    assert_eq!(rupees(t.advances_open_paisa), 0);
    assert_eq!((t.stock_lots, t.stock_qty), (0, 0));
    assert_eq!(
        (
            t.trips_open,
            t.trips_purchases_with_stock,
            t.trips_qty_left,
            rupees(t.trips_cost_left_paisa)
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(
        (rupees(t.lent_open_paisa), rupees(t.lent_overdue_paisa)),
        (0, 0)
    );
    assert_eq!(rupees(t.expenses_today_paisa), 3_000);

    // The partner strip: today, month and year are the profit shares of the one day; کل رقم
    // is C22 (E2E-37 without the capital of ب).
    let strip: Vec<(i64, i64, i64, i64)> = d
        .partners
        .iter()
        .map(|p| {
            (
                rupees(p.today_paisa),
                rupees(p.month_paisa),
                rupees(p.year_paisa),
                rupees(p.total_paisa),
            )
        })
        .collect();
    assert_eq!(
        strip,
        vec![
            (996, 996, 996, -1_004),
            (746, 746, 746, 746),
            (746, 746, 746, 746)
        ]
    );
    // The strip adds up to the shared income less the expenses: 5,488 − 3,000 = 2,488.
    assert_eq!(strip.iter().map(|s| s.0).sum::<i64>(), 2_488);
    assert_eq!(d.partners[0].name_en, "Partner A");

    // Today's بکری.
    assert_eq!(d.bikri_today_count, 2);
    let rows: Vec<(i64, i64, i64, i64, bool)> = d
        .bikri_today
        .iter()
        .map(|b| {
            (
                b.number,
                rupees(b.gross_paisa),
                rupees(b.net_paisa),
                b.seller_count,
                b.cancelled,
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![(1, 29_400, 19_316, 1, false), (2, 12_000, 10_160, 1, false)]
    );
    assert_eq!(d.bikri_today[1].truck_no.as_deref(), Some("LHR-0077"));
    assert_eq!(
        d.bikri_today
            .iter()
            .map(|b| b.seller.clone().unwrap().en)
            .collect::<Vec<_>>(),
        vec!["Rahim".to_owned(), "Karim".to_owned()]
    );

    // The oldest dues: both bills are of 10-10-26, so the bigger balance comes first.
    let dues: Vec<(String, i64, Option<String>, Option<i64>)> = d
        .oldest_dues
        .iter()
        .map(|due| {
            (
                due.party.code.clone(),
                rupees(due.balance_paisa),
                due.oldest_date.clone(),
                due.days,
            )
        })
        .collect();
    assert_eq!(
        dues,
        vec![
            (
                "03040002".to_owned(),
                18_684,
                Some("2026-10-10".to_owned()),
                Some(0)
            ),
            (
                "03040001".to_owned(),
                2_880,
                Some("2026-10-10".to_owned()),
                Some(0)
            ),
        ]
    );

    // The charts: 30 days, today last; October's commission by item.
    assert_eq!(d.chart_days.len(), 30);
    assert_eq!(d.chart_days[0].date, "2026-09-11");
    let last = d.chart_days.last().unwrap();
    assert_eq!(
        (
            last.date.as_str(),
            rupees(last.gross_paisa),
            rupees(last.commission_paisa)
        ),
        ("2026-10-10", 41_400, 5_448)
    );
    assert!(
        d.chart_days[..29]
            .iter()
            .all(|c| rupees(c.gross_paisa) == 0 && rupees(c.commission_paisa) == 0)
    );
    let items: Vec<(String, i64)> = d
        .chart_items
        .iter()
        .map(|i| (i.name_en.clone(), rupees(i.commission_paisa)))
        .collect();
    assert_eq!(
        items,
        vec![("Banana".to_owned(), 3_928), ("Apple".to_owned(), 1_520)]
    );

    // Nothing needs attention on the worked day, and the dashboard and the bell agree.
    assert!(d.attention.items.is_empty());
    assert_eq!(d.attention.total_count, 0);
    assert_eq!(d.attention, bell(&w, off()));
}

#[test]
fn every_tile_ties_to_the_report_or_the_screen_that_shows_the_same_figure() {
    let w = full_day();
    let d = dash(&w);
    let t = d.tiles;
    // R20's total row is the shop's total اگراہی (FR-19.9): the balance column.
    let r20 = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(totals_money(&r20, 3), rupees(t.buyers_dues_paisa));
    // R24's payable total and R26's open total.
    let r24 = report(&w, ReportId::R24, ReportFilters::default(), 0, 0);
    assert_eq!(totals_money(&r24, 2), rupees(t.sellers_payable_paisa));
    let r26 = report(&w, ReportId::R26, ReportFilters::default(), 0, 0);
    assert_eq!(totals_money(&r26, 4), rupees(t.advances_open_paisa));
    // R34 for today: sales, commission and the cash at the end of the day.
    let r34 = report(&w, ReportId::R34, ReportFilters::default(), 0, 0);
    let amount_of = |label: &str| -> i64 {
        r34.rows
            .iter()
            .find(|r| matches!(&r.cells[1], Cell::Label { id } if id == label))
            .map(|r| money(&r.cells[3]))
            .unwrap_or_else(|| panic!("no row {label}"))
    };
    assert_eq!(amount_of("report-r34-bikri"), rupees(t.sales_gross_paisa));
    assert_eq!(
        amount_of("report-r34-commission"),
        rupees(t.commission_paisa)
    );
    assert_eq!(
        amount_of("report-r34-cash-closing"),
        rupees(t.cash_in_hand_paisa)
    );
    assert_eq!(
        amount_of("report-r34-expenses"),
        rupees(t.expenses_today_paisa)
    );
    assert_eq!(
        amount_of("report-r34-buyers-dues"),
        rupees(t.buyers_dues_paisa)
    );
    // The trips tile is the trips screen's summary.
    let summary = trips::summary(&w.env.ctx).unwrap();
    assert_eq!(
        (
            summary.open_trips,
            summary.purchases_with_stock,
            summary.qty_left,
            summary.cost_left_paisa
        ),
        (
            t.trips_open,
            t.trips_purchases_with_stock,
            t.trips_qty_left,
            t.trips_cost_left_paisa
        )
    );
    // The partner strip's کل رقم is the partner screen's.
    let overview = partners::overview(&w.env.ctx).unwrap();
    let from_screen: Vec<i64> = overview.iter().map(|p| rupees(p.total_paisa)).collect();
    let from_dashboard: Vec<i64> = d.partners.iter().map(|p| rupees(p.total_paisa)).collect();
    assert_eq!(from_screen, from_dashboard);
}

// ---- Other books -------------------------------------------------------------------------------

#[test]
fn e2e_28_the_advance_stays_open_and_the_payable_is_whole() {
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].adjust_advance = false;
    save(&w, &draft);
    let t = dash(&w).tiles;
    assert_eq!(rupees(t.sellers_payable_paisa), 19_316);
    assert_eq!(rupees(t.advances_open_paisa), 15_000);
    let r24 = report(&w, ReportId::R24, ReportFilters::default(), 0, 0);
    assert_eq!(totals_money(&r24, 2), 19_316);
    assert_eq!(totals_money(&r24, 3), 15_000);
}

#[test]
fn e2e_32_cash_lent_to_a_buyer_is_a_due_and_leaves_the_cash() {
    let w = full_day();
    let mut lend = payment(w.akram, "3,000");
    lend.reason = "نقد ادھار".into();
    vouchers::save_payment(&w.env.ctx, &w.owner, &lend).unwrap();
    let t = dash(&w).tiles;
    assert_eq!(rupees(t.buyers_dues_paisa), 24_564);
    assert_eq!(rupees(t.cash_in_hand_paisa), 55_484);
}

#[test]
fn e2e_35_capital_in_raises_the_cash_and_the_partner_total_but_not_the_shares() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_capital_in(
        &w.env.ctx,
        &w.owner,
        &CapitalInput {
            date: "11-10-2026".into(),
            partner_id: Some(w.partners[1]),
            amount: "50,000".into(),
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let d = dash(&w);
    assert_eq!(rupees(d.tiles.cash_in_hand_paisa), 108_484);
    let strip: Vec<(i64, i64)> = d
        .partners
        .iter()
        .map(|p| (rupees(p.today_paisa), rupees(p.total_paisa)))
        .collect();
    // 11-10: no sales, so nobody has a share today; the totals are E2E-37.
    assert_eq!(strip, vec![(0, -1_004), (0, 50_746), (0, 746)]);
    // The month still holds the shares of 10-10-26.
    assert_eq!(rupees(d.partners[0].month_paisa), 996);
    // Nothing was sold today.
    assert_eq!(
        (rupees(d.tiles.sales_gross_paisa), d.tiles.sales_count),
        (0, 0)
    );
    // The day close of 10-10 is still open: one earlier day (10-10) is not closed.
    assert_eq!(kinds(&d.attention), vec![(AttentionKind::OpenDays, 1)]);
}

// ---- Goods not sold, trips ---------------------------------------------------------------------

fn trip_with_one_purchase(w: &World) -> (i64, i64) {
    let [a, b, c] = w.partners;
    let shop = |partner_id: i64, share: &str| VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    };
    let trip = trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: vec![shop(a, "40"), shop(b, "30"), shop(c, "30")],
        },
    )
    .unwrap()
    .id;
    let item = support::add_item(&w.env, &w.owner, "مالٹا", "Orange", "fruit", "unit");
    let purchase = purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            trip_id: Some(trip),
            date: "10-10-2026".into(),
            supplier: None,
            item_id: Some(item),
            packing_id: Some(w.box_),
            qty: "20".into(),
            weight: None,
            rate: "500".into(),
            freight: Some("400".into()),
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::<EntryPartInput>::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .id;
    (trip, purchase)
}

#[test]
fn the_trips_tile_is_the_trips_summary_and_trip_goods_are_not_consigned_stock() {
    // E2E-34 before the sale: 20 نگ bought for 10,400 from the shop's cash.
    let w = worked_day();
    trip_with_one_purchase(&w);
    let d = dash(&w);
    let t = d.tiles;
    assert_eq!(
        (
            t.trips_open,
            t.trips_purchases_with_stock,
            t.trips_qty_left,
            rupees(t.trips_cost_left_paisa)
        ),
        (1, 1, 20, 10_400)
    );
    // The goods of the trip are not lots of a بکری, so the stock tile stays 0.
    assert_eq!((t.stock_lots, t.stock_qty), (0, 0));
    // 50,000 − 10,400.
    assert_eq!(rupees(t.cash_in_hand_paisa), 39_600);
}

#[test]
fn e2e_16_a_part_sold_lot_is_stock_and_is_on_the_bell_after_two_days() {
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    save(&w, &draft);
    let t = dash(&w).tiles;
    assert_eq!((t.stock_lots, t.stock_qty), (1, 5));
    // 2 days: not yet. 3 days: waiting more than 2 days.
    set_today(&w.env, 2026, 10, 12);
    assert!(
        !kinds(&bell(&w, off()))
            .iter()
            .any(|k| k.0 == AttentionKind::StockWaiting)
    );
    set_today(&w.env, 2026, 10, 13);
    let list = bell(&w, off());
    let waiting = item(&list, AttentionKind::StockWaiting);
    assert_eq!(waiting.count, 1);
    assert_eq!(waiting.qty, Some(5));
    assert_eq!(waiting.days, Some(3));
    assert_eq!(
        waiting.first.as_ref().unwrap().target,
        AttentionTarget::Bikri
    );
}

// ---- The attention list ------------------------------------------------------------------------

#[test]
fn the_bell_counts_the_earlier_days_that_are_not_closed() {
    // Closed up to 09-10-26. On 18-10-26 the days 10-10 to 17-10 are eight earlier open days.
    let w = full_day();
    set_today(&w.env, 2026, 10, 18);
    let list = bell(&w, off());
    let open = item(&list, AttentionKind::OpenDays);
    assert_eq!((open.count, open.days), (8, Some(8)));
}

#[test]
fn w1_a_payable_is_old_after_seven_days_and_not_on_the_seventh() {
    // کریم's 10,160 was posted on 10-10-26 (PRD 4.9). 7 days old is not older than 7 days.
    let w = full_day();
    set_today(&w.env, 2026, 10, 17);
    assert!(
        !bell(&w, off())
            .items
            .iter()
            .any(|i| i.kind == AttentionKind::PayablesOld)
    );
    set_today(&w.env, 2026, 10, 18);
    let list = bell(&w, off());
    let old = item(&list, AttentionKind::PayablesOld);
    assert_eq!((old.count, old.amount_paisa.map(rupees)), (1, Some(10_160)));
    let first = old.first.as_ref().unwrap();
    assert_eq!(
        (first.target, first.name_en.as_deref()),
        (AttentionTarget::Account, Some("Karim"))
    );
}

#[test]
fn w2_the_advance_adjusted_on_a_credit_is_not_part_of_the_old_payable() {
    // Event 1 alone: رحیم's credit 24,316 less the advance of 15,000 adjusted on it leaves 9,316;
    // 5,000 was paid in cash on the day: payable 4,316.
    let w = worked_day();
    save(&w, &support::event1(&w));
    assert_eq!(rupees(dash(&w).tiles.sellers_payable_paisa), 4_316);
    set_today(&w.env, 2026, 10, 17);
    assert!(
        !bell(&w, off())
            .items
            .iter()
            .any(|i| i.kind == AttentionKind::PayablesOld)
    );
    set_today(&w.env, 2026, 10, 18);
    let list = bell(&w, off());
    assert_eq!(
        item(&list, AttentionKind::PayablesOld)
            .amount_paisa
            .map(rupees),
        Some(4_316)
    );
}

#[test]
fn w4_a_part_paid_recent_credit_hides_the_old_one_until_the_payment_is_small() {
    // Credit A 10,000 on 10-10, credit B 6,000 on 14-10, today 19-10: the cutoff is 12-10.
    let seller = |w: &World| w.karim;
    let make = |paid: &str| {
        let w = worked_day();
        credit_seller(&w, seller(&w), "10,000", "10-10-2026");
        set_today(&w.env, 2026, 10, 14);
        credit_seller(&w, seller(&w), "6,000", "14-10-2026");
        set_today(&w.env, 2026, 10, 19);
        pay_seller(&w, seller(&w), paid, "19-10-2026");
        w
    };
    // Paid 12,000: 4,000 is open and it is all of B. Nothing is older than 7 days.
    let w = make("12,000");
    assert_eq!(rupees(dash(&w).tiles.sellers_payable_paisa), 4_000);
    assert!(
        !bell(&w, off())
            .items
            .iter()
            .any(|i| i.kind == AttentionKind::PayablesOld)
    );
    // Paid 8,000: 8,000 is open, B (6,000) and 2,000 of A. 2,000 is older than 7 days.
    let w = make("8,000");
    assert_eq!(rupees(dash(&w).tiles.sellers_payable_paisa), 8_000);
    let list = bell(&w, off());
    let old = item(&list, AttentionKind::PayablesOld);
    assert_eq!((old.count, old.amount_paisa.map(rupees)), (1, Some(2_000)));
}

#[test]
fn w6_an_edited_credit_counts_once_through_its_live_entry() {
    // A cancelled voucher and its reversal never count: a payment made and cancelled changes
    // nothing. (The edit of a جنرل credit is covered by the live predicate of the same query.)
    let w = worked_day();
    credit_seller(&w, w.karim, "5,000", "10-10-2026");
    set_today(&w.env, 2026, 10, 19);
    let paid = {
        let mut p = payment(w.karim, "5,000");
        p.date = "19-10-2026".into();
        vouchers::save_payment(&w.env.ctx, &w.owner, &p).unwrap()
    };
    assert_eq!(rupees(dash(&w).tiles.sellers_payable_paisa), 0);
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::VoucherCancelInput {
            id: paid.voucher_id,
            version: vouchers::get(&w.env.ctx, paid.voucher_id).unwrap().version,
            reason: "غلط".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(rupees(dash(&w).tiles.sellers_payable_paisa), 5_000);
    let list = bell(&w, off());
    let old = item(&list, AttentionKind::PayablesOld);
    assert_eq!(old.amount_paisa.map(rupees), Some(5_000));
}

#[test]
fn w7_an_opening_credit_is_aged_by_its_own_date() {
    // کریم's opening credit of 10,160 is dated 20-10-26, and 4,000 is paid on 22-10-26.
    let w = worked_day();
    set_today(&w.env, 2026, 10, 20);
    let version = accounts::get(&w.env.ctx, w.karim).unwrap().version;
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: w.karim,
                version,
                opening_date: "20-10-2026".into(),
                opening_dr: String::new(),
                opening_cr: "10,160".into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 22);
    pay_seller(&w, w.karim, "4,000", "22-10-2026");
    assert_eq!(rupees(dash(&w).tiles.sellers_payable_paisa), 6_160);
    // Two days old: not older than 7 days, though it is the oldest posting.
    assert!(
        !bell(&w, off())
            .items
            .iter()
            .any(|i| i.kind == AttentionKind::PayablesOld)
    );
    // 8 days old on 28-10-26.
    set_today(&w.env, 2026, 10, 28);
    let list = bell(&w, off());
    assert_eq!(
        item(&list, AttentionKind::PayablesOld)
            .amount_paisa
            .map(rupees),
        Some(6_160)
    );
}

/// Opens a seller's opening credit on a day (the opening date must be an open day).
fn opening_credit(w: &World, seller: i64, amount: &str, date: &str) {
    let version = accounts::get(&w.env.ctx, seller).unwrap().version;
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: seller,
                version,
                opening_date: date.into(),
                opening_dr: String::new(),
                opening_cr: amount.into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
}

fn payables_old_amount(w: &World) -> Option<i64> {
    bell(w, off())
        .items
        .iter()
        .find(|i| i.kind == AttentionKind::PayablesOld)
        .and_then(|i| i.amount_paisa.map(rupees))
}

#[test]
fn w8_an_older_credit_is_old_while_the_part_left_of_a_recent_opening_is_not() {
    // کریم: a credit of 3,000 on 10-10, an opening credit of 10,160 dated 20-10, 4,000 paid on
    // 22-10. The payment settles the opening first (FR-7.59): 6,160 of it is left, 2 days old,
    // and the credit of 3,000 is untouched and 12 days old.
    let w = worked_day();
    credit_seller(&w, w.karim, "3,000", "10-10-2026");
    set_today(&w.env, 2026, 10, 20);
    opening_credit(&w, w.karim, "10,160", "20-10-2026");
    set_today(&w.env, 2026, 10, 22);
    pay_seller(&w, w.karim, "4,000", "22-10-2026");
    assert_eq!(rupees(dash(&w).tiles.sellers_payable_paisa), 9_160);
    assert_eq!(payables_old_amount(&w), Some(3_000));
}

#[test]
fn w9_an_old_opening_left_behind_a_recent_credit_is_old() {
    // The opening credit of 10,160 is dated 10-10, a credit of 5,000 is dated 18-10 and 4,000 is
    // paid on 19-10. Payable 11,160: the recent 5,000, and 6,160 of the opening, which is old.
    let w = worked_day();
    opening_credit(&w, w.karim, "10,160", "10-10-2026");
    set_today(&w.env, 2026, 10, 18);
    credit_seller(&w, w.karim, "5,000", "18-10-2026");
    set_today(&w.env, 2026, 10, 19);
    pay_seller(&w, w.karim, "4,000", "19-10-2026");
    set_today(&w.env, 2026, 10, 20);
    assert_eq!(rupees(dash(&w).tiles.sellers_payable_paisa), 11_160);
    assert_eq!(payables_old_amount(&w), Some(6_160));
}

#[test]
fn w10_a_one_time_seller_is_a_key_of_his_own() {
    use mandi_services::core::bikri::PukhtaInput;
    // Two one-time sellers on the پختہ کھاتہ account, each with a lot of 10 نگ sold at 1,000
    // (net 10,000 − 600 commission = 9,400). Nazir is paid 4,000 on the spot and is owed 5,400;
    // Bashir is paid the whole 9,400 and is owed nothing. Only Nazir is old on 18-10, and not on
    // 17-10.
    let w = worked_day();
    let lot = |key: &str, name_ur: &str, name_en: &str, phone: &str, cash: &str, sale: &str| {
        let mut lot = support::bare_lot(&w, key, w.pukhta_seller, w.banana, "10");
        lot.pukhta = Some(PukhtaInput {
            name_ur: name_ur.into(),
            name_en: name_en.into(),
            phone: phone.into(),
        });
        lot.cash_paid = Some(cash.into());
        lot.sales = vec![support::sale(sale, w.akram, "10", "1000")];
        lot
    };
    save(
        &w,
        &support::draft(vec![
            lot("lot-1", "نذیر", "Nazir", "0300-2222222", "4,000", "s-1"),
            lot("lot-2", "بشیر", "Bashir", "0300-3333333", "9,400", "s-2"),
        ]),
    );
    assert_eq!(rupees(dash(&w).tiles.sellers_payable_paisa), 5_400);
    set_today(&w.env, 2026, 10, 17);
    assert_eq!(payables_old_amount(&w), None);
    set_today(&w.env, 2026, 10, 18);
    let list = bell(&w, off());
    let old = item(&list, AttentionKind::PayablesOld);
    assert_eq!((old.count, old.amount_paisa.map(rupees)), (1, Some(5_400)));
    let first = old.first.as_ref().unwrap();
    assert_eq!(first.id, w.pukhta_seller);
    let key: String = w
        .env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row(
                "SELECT pukhta_person_key FROM bikri_lots WHERE pukhta_name_en = 'Nazir'",
                [],
                |r| r.get(0),
            )
        })
        .unwrap();
    assert_eq!(first.person_key.as_deref(), Some(key.as_str()));
}

fn w11_world() -> World {
    // Opening position; on 10-10 رحیم returns 10,000 of his advance in cash (a وصولی on his
    // account), then Event 1 with the tick off.
    let w = worked_day();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.rahim, "10,000")).unwrap();
    let mut draft = support::event1(&w);
    draft.lots[0].adjust_advance = false;
    save(&w, &draft);
    w
}

#[test]
fn w11_a_seller_receipt_adjusted_on_his_advance_holds_nothing_of_the_payable() {
    let w = w11_world();
    let t = dash(&w).tiles;
    assert_eq!(rupees(t.advances_open_paisa), 5_000);
    assert_eq!(rupees(t.sellers_payable_paisa), 19_316);
    set_today(&w.env, 2026, 10, 18);
    assert_eq!(payables_old_amount(&w), Some(19_316));
}

#[test]
fn broken_matching_makes_the_bell_fail_and_not_guess() {
    // More adjusted on a credit than the credit holds cannot happen through the app. If it is
    // there, the numbers cannot be trusted: the bell returns an error, not a figure.
    let w = w11_world();
    set_today(&w.env, 2026, 10, 18);
    // The table is append-only (a trigger); the trigger is dropped in this test database only.
    let broken = w.env.ctx.db().write(|tx| {
        let triggers: Vec<String> = {
            let mut statement = tx.prepare(
                "SELECT name FROM sqlite_master
                 WHERE type = 'trigger' AND tbl_name = 'advance_allocations'",
            )?;
            statement
                .query_map([], |r| r.get(0))?
                .collect::<Result<_, _>>()?
        };
        for name in triggers {
            tx.execute_batch(&format!("DROP TRIGGER \"{name}\""))?;
        }
        tx.execute(
            "UPDATE advance_allocations SET amount_paisa = 99900000 WHERE is_active = 1",
            [],
        )
        .map_err(mandi_services::db::DbError::from)
    });
    assert!(
        broken.is_ok(),
        "the test database must allow the change: {broken:?}"
    );
    assert!(dashboard::attention(w.env.ctx.db(), off()).is_err());
}

#[test]
fn dues_older_than_30_days_and_buyers_over_their_credit_limit() {
    let w = full_day();
    // Both bills are of 10-10-26: 30 days old on 09-11, 31 on 10-11.
    set_today(&w.env, 2026, 11, 9);
    assert!(
        !bell(&w, off())
            .items
            .iter()
            .any(|i| i.kind == AttentionKind::DuesOld)
    );
    set_today(&w.env, 2026, 11, 10);
    let list = bell(&w, off());
    let old = item(&list, AttentionKind::DuesOld);
    assert_eq!(
        (old.count, old.amount_paisa.map(rupees), old.days),
        (2, Some(21_564), Some(31))
    );
    // اکرم's limit is 2,000 and he owes 2,880: over by 880. A limit of 0 is a limit too.
    sql(
        &w,
        "UPDATE accounts SET credit_limit_paisa = 200000 WHERE id = ?1",
        w.akram,
    );
    let list = bell(&w, off());
    let over = item(&list, AttentionKind::OverCreditLimit);
    assert_eq!((over.count, over.amount_paisa.map(rupees)), (1, Some(880)));
    assert_eq!(over.first.as_ref().unwrap().id, w.akram);
    // The party card says the same.
    let card = mandi_services::assist::party_card(&w.env.ctx, w.akram).unwrap();
    assert!(card.warnings.iter().any(|x| matches!(
        x.kind,
        mandi_services::core::accounts::CardWarningKind::OverCreditLimit
    )));
}

#[test]
fn drafts_names_and_the_cloud_backup_are_on_the_bell() {
    let w = full_day();
    drafts::save(
        &w.env.ctx,
        &w.owner,
        &DraftSaveInput {
            screen: "bikri".into(),
            payload_json: "{}".into(),
        },
    )
    .unwrap();
    // The table refuses an empty name, so the Phase 2 import case is made with the check off.
    w.env
        .ctx
        .db()
        .write(|tx| {
            tx.execute_batch(&format!(
                "PRAGMA ignore_check_constraints = ON;
                 UPDATE accounts SET name_en = '' WHERE id = {};
                 PRAGMA ignore_check_constraints = OFF;",
                w.karim
            ))
            .map_err(mandi_services::db::DbError::from)
        })
        .unwrap();
    let mut backup = BackupFacts {
        cloud_on: true,
        cloud_age_hours: Some(30),
    };
    let list = bell(&w, backup);
    assert_eq!(
        kinds(&list),
        vec![
            (AttentionKind::Drafts, 1),
            (AttentionKind::BackupOld, 1),
            (AttentionKind::NamesMissing, 1)
        ]
    );
    assert_eq!(list.total_count, 3);
    assert_eq!(item(&list, AttentionKind::BackupOld).days, Some(1));
    // 24 hours is not older than 24 hours; a copy that is off is no problem; none at all is one.
    backup.cloud_age_hours = Some(24);
    assert!(
        !kinds(&bell(&w, backup))
            .iter()
            .any(|k| k.0 == AttentionKind::BackupOld)
    );
    assert!(
        !kinds(&bell(&w, off()))
            .iter()
            .any(|k| k.0 == AttentionKind::BackupOld)
    );
    backup.cloud_age_hours = None;
    assert!(
        kinds(&bell(&w, backup))
            .iter()
            .any(|k| k.0 == AttentionKind::BackupOld)
    );
}

fn aslam(w: &World) -> i64 {
    confirming(|words| {
        lent::person_create(
            &w.env.ctx,
            &w.owner,
            &LentPersonCreateInput {
                name_ur: "اسلم".into(),
                name_en: "Aslam".into(),
                phone: Some("0300-1234567".into()),
                staff_id: None,
                confirmed_rules_words: words.to_vec(),
            },
        )
    })
    .account
    .id
}

#[test]
fn e2e_30b_the_loan_past_its_return_by_day_is_on_the_tile_and_the_bell() {
    let w = full_day();
    let person = aslam(&w);
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &MoneyLentInput {
            date: "10-10-2026".into(),
            person_account_id: Some(person),
            amount: "5,000".into(),
            reason: "گھریلو ضرورت".into(),
            return_by: Some("15-10-2026".into()),
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &MoneyLentReturnInput {
            date: "11-10-2026".into(),
            person_account_id: Some(person),
            amount: "3,000".into(),
            mode: LentReturnMode::Cash,
            salary_head_id: None,
            narration: None,
        },
    )
    .unwrap();
    // 15-10: the return-by day itself is not past.
    set_today(&w.env, 2026, 10, 15);
    assert!(
        !bell(&w, off())
            .items
            .iter()
            .any(|i| i.kind == AttentionKind::LoansOverdue)
    );
    set_today(&w.env, 2026, 10, 16);
    let d = dash(&w);
    assert_eq!(
        (
            rupees(d.tiles.lent_open_paisa),
            rupees(d.tiles.lent_overdue_paisa)
        ),
        (2_000, 2_000)
    );
    let overdue = item(&d.attention, AttentionKind::LoansOverdue);
    assert_eq!(
        (
            overdue.count,
            overdue.amount_paisa.map(rupees),
            overdue.days
        ),
        (1, Some(2_000), Some(1))
    );
    assert_eq!(overdue.first.as_ref().unwrap().id, person);
    // R32 says the same open amount.
    let r32 = report(&w, ReportId::R32, ReportFilters::default(), 0, 0);
    assert_eq!(totals_money(&r32, 5), 2_000);
}

#[test]
fn a_month_close_changes_no_tile_and_no_partner_total() {
    // The month close of October is dated 02-11-26 and moves income and expenses into the
    // partners' capital accounts. Today's expenses, the strip's shares, the partner totals and the
    // cash do not move because of it (PRD FR-17.6, FR-17.9).
    let w = full_day();
    let before = dash(&w);
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
    let after = dash(&w);
    assert_eq!(rupees(after.tiles.expenses_today_paisa), 0);
    assert_eq!(
        (
            rupees(after.tiles.sales_gross_paisa),
            after.tiles.sales_count
        ),
        (0, 0)
    );
    assert_eq!(
        after.tiles.cash_in_hand_paisa,
        before.tiles.cash_in_hand_paisa
    );
    assert_eq!(
        after.tiles.buyers_dues_paisa,
        before.tiles.buyers_dues_paisa
    );
    let totals =
        |d: &Dashboard| -> Vec<i64> { d.partners.iter().map(|p| rupees(p.total_paisa)).collect() };
    assert_eq!(totals(&after), totals(&before));
    // October's shares are still October's: the month column on 02-11 is November's.
    assert!(after.partners.iter().all(|p| rupees(p.today_paisa) == 0));
    assert!(after.partners.iter().all(|p| rupees(p.month_paisa) == 0));
    assert!(after.partners.iter().all(|p| rupees(p.year_paisa) > 0));
}

#[test]
fn every_kind_of_the_bell_appears_once_the_count_is_the_list_and_the_overdue_agree() {
    // A book where all ten kinds of PRD FR-7.44 are true at once, on 15-11-26.
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    save(&w, &draft);
    save(&w, &event2(&w));
    let person = aslam(&w);
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &MoneyLentInput {
            date: "10-10-2026".into(),
            person_account_id: Some(person),
            amount: "5,000".into(),
            reason: "گھریلو ضرورت".into(),
            return_by: Some("15-10-2026".into()),
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    vouchers::save_personal_expense(
        &w.env.ctx,
        &w.owner,
        &PersonalExpenseInput {
            date: "10-10-2026".into(),
            partner_id: Some(w.partners[1]),
            amount: "1,000".into(),
            reason: "گھر کا خرچ".into(),
            return_by: Some("12-10-2026".into()),
            taken_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    drafts::save(
        &w.env.ctx,
        &w.owner,
        &DraftSaveInput {
            screen: "bikri".into(),
            payload_json: "{}".into(),
        },
    )
    .unwrap();
    sql(
        &w,
        "UPDATE accounts SET credit_limit_paisa = 200000 WHERE id = ?1",
        w.akram,
    );
    w.env
        .ctx
        .db()
        .write(|tx| {
            tx.execute_batch(&format!(
                "PRAGMA ignore_check_constraints = ON;
                 UPDATE accounts SET name_en = '' WHERE id = {};
                 PRAGMA ignore_check_constraints = OFF;",
                w.rahim
            ))
            .map_err(mandi_services::db::DbError::from)
        })
        .unwrap();
    set_today(&w.env, 2026, 11, 15);
    let backup = BackupFacts {
        cloud_on: true,
        cloud_age_hours: Some(80),
    };
    let list = bell(&w, backup);
    // Every kind once, in the order of the list, and the bell number is the sum of the counts.
    let found: Vec<AttentionKind> = list.items.iter().map(|i| i.kind).collect();
    assert_eq!(found, AttentionKind::ALL.to_vec());
    assert!(list.items.iter().all(|i| i.count > 0));
    assert_eq!(
        list.total_count,
        list.items.iter().map(|i| i.count).sum::<i64>()
    );
    // The dashboard carries the same list.
    assert_eq!(
        dashboard::get(w.env.ctx.db(), backup).unwrap().attention,
        list
    );

    // The overdue loans and takes are the ones of `assist_attention_overdue`: the same count, the
    // same open money and the same days past.
    let overdue = lent::attention_overdue(&w.env.ctx).unwrap();
    for (kind, overdue_kind) in [
        (
            AttentionKind::LoansOverdue,
            mandi_services::core::lent::OverdueKind::Loan,
        ),
        (
            AttentionKind::TakesOverdue,
            mandi_services::core::lent::OverdueKind::Take,
        ),
    ] {
        let mine = item(&list, kind);
        let theirs: Vec<_> = overdue
            .items
            .iter()
            .filter(|i| i.kind == overdue_kind)
            .collect();
        assert_eq!(mine.count, i64::try_from(theirs.len()).unwrap(), "{kind:?}");
        assert_eq!(
            mine.amount_paisa.map(rupees),
            Some(theirs.iter().map(|i| rupees(i.open_paisa)).sum::<i64>()),
            "{kind:?}"
        );
        assert_eq!(
            mine.days,
            theirs.iter().map(|i| i.days_past).max(),
            "{kind:?}"
        );
    }
    assert_eq!(
        overdue.total_count,
        item(&list, AttentionKind::LoansOverdue).count
            + item(&list, AttentionKind::TakesOverdue).count
    );
}

#[test]
fn a_partner_take_with_a_return_by_day_is_overdue_and_one_without_is_not() {
    let w = full_day();
    // الف's 2,000 of Event 5 has no return-by day. A second take has one.
    vouchers::save_personal_expense(
        &w.env.ctx,
        &w.owner,
        &PersonalExpenseInput {
            date: "10-10-2026".into(),
            partner_id: Some(w.partners[1]),
            amount: "1,000".into(),
            reason: "گھر کا خرچ".into(),
            return_by: Some("12-10-2026".into()),
            taken_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 14);
    let list = bell(&w, off());
    let takes = item(&list, AttentionKind::TakesOverdue);
    assert_eq!(
        (takes.count, takes.amount_paisa.map(rupees), takes.days),
        (1, Some(1_000), Some(2))
    );
    let first = takes.first.as_ref().unwrap();
    assert_eq!(
        (first.target, first.id),
        (AttentionTarget::Partner, w.partners[1])
    );
    // A take is not money lent: the tile ادھار رقم باقی does not hold it.
    assert_eq!(rupees(dash(&w).tiles.lent_open_paisa), 0);
}

// ---- The tiles with signs ----------------------------------------------------------------------

#[test]
fn cash_below_zero_is_shown_as_a_negative_number() {
    // PRD FR-6.7: the cash can go below zero after the Owner confirms. 50,000 − 60,000.
    let w = worked_day();
    let mut lend = payment(w.akram, "60,000");
    lend.reason = "نقد ادھار".into();
    lend.confirm_cash_below_zero = true;
    vouchers::save_payment(&w.env.ctx, &w.owner, &lend).unwrap();
    assert_eq!(rupees(dash(&w).tiles.cash_in_hand_paisa), -10_000);
}

// ---- What a screen remembers -------------------------------------------------------------------

#[test]
fn a_screen_keeps_its_filters_per_user_and_forgets_them_on_an_empty_value() {
    let w = worked_day();
    let user = w.owner.user_id;
    let ctx = &w.env.ctx;
    assert_eq!(screen_prefs::get(ctx, user, "report.r20").unwrap(), None);
    screen_prefs::set(ctx, user, "report.r20", "{\"city_id\":3}").unwrap();
    assert_eq!(
        screen_prefs::get(ctx, user, "report.r20")
            .unwrap()
            .as_deref(),
        Some("{\"city_id\":3}")
    );
    // Another screen has its own value.
    assert_eq!(screen_prefs::get(ctx, user, "analytics.a").unwrap(), None);
    // Not JSON, too long, and a bad name are refused with their field.
    let field_of = |result: Result<(), ServiceError>| match result {
        Err(ServiceError::Rejected(items)) => items[0].field.clone().unwrap_or_default(),
        other => panic!("{other:?}"),
    };
    assert_eq!(
        field_of(screen_prefs::set(ctx, user, "report.r20", "{oops")),
        "value"
    );
    assert_eq!(
        field_of(screen_prefs::set(
            ctx,
            user,
            "report.r20",
            &"1".repeat(4_001)
        )),
        "value"
    );
    assert_eq!(
        field_of(screen_prefs::set(ctx, user, "Report R20", "{}")),
        "screen"
    );
    assert_eq!(field_of(screen_prefs::set(ctx, user, "", "{}")), "screen");
    // An empty value forgets it, and forgetting a screen that was never saved does nothing.
    screen_prefs::set(ctx, user, "report.r20", "").unwrap();
    assert_eq!(screen_prefs::get(ctx, user, "report.r20").unwrap(), None);
    screen_prefs::set(ctx, user, "never.saved", "").unwrap();
    let rows: i64 = ctx
        .db()
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM user_prefs WHERE key = 'filters.never.saved'",
                [],
                |r| r.get(0),
            )
        })
        .unwrap();
    assert_eq!(rows, 0);
}

#[test]
fn a_user_keeps_at_most_200_screens() {
    let w = worked_day();
    let user = w.owner.user_id;
    for n in 0..screen_prefs::MAX_SCREENS {
        screen_prefs::set(&w.env.ctx, user, &format!("screen.{n}"), "{}").unwrap();
    }
    assert!(matches!(
        screen_prefs::set(&w.env.ctx, user, "one.more", "{}"),
        Err(ServiceError::Rejected(_))
    ));
    // A screen already kept can still be changed, and forgetting one frees its slot.
    screen_prefs::set(&w.env.ctx, user, "screen.0", "{\"a\":1}").unwrap();
    screen_prefs::set(&w.env.ctx, user, "screen.1", "").unwrap();
    screen_prefs::set(&w.env.ctx, user, "one.more", "{}").unwrap();
}

#[test]
fn every_dashboard_read_is_one_snapshot_and_changes_nothing() {
    // The dashboard writes nothing: the audit log and the journal are the same after it.
    let w = full_day();
    let count = |sql: &str| -> i64 {
        w.env
            .ctx
            .db()
            .read(|conn| conn.query_row(sql, [], |r| r.get(0)))
            .unwrap()
    };
    let before = (
        count("SELECT COUNT(*) FROM audit_log"),
        count("SELECT COUNT(*) FROM journal_lines"),
        count("SELECT COUNT(*) FROM user_prefs"),
    );
    let first = dash(&w);
    let second = dash(&w);
    assert_eq!(first, second);
    let _ = bell(&w, off());
    let after = (
        count("SELECT COUNT(*) FROM audit_log"),
        count("SELECT COUNT(*) FROM journal_lines"),
        count("SELECT COUNT(*) FROM user_prefs"),
    );
    assert_eq!(before, after);
}

// ---- Fixtures for the frontend -----------------------------------------------------------------

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

    #[test]
    fn the_dashboard_and_the_bell_fixtures_are_the_real_answers() {
        // The worked day: every tile of PRD 4.9, the strip, the lists and the charts.
        let w = full_day();
        pair("dashboard_get_worked_day", &json!({}), &dash(&w));
        // The same book on 18-10-26: eight earlier days open and کریم's payable older than 7
        // days, with the cloud copy 30 hours old.
        set_today(&w.env, 2026, 10, 18);
        let backup = BackupFacts {
            cloud_on: true,
            cloud_age_hours: Some(30),
        };
        pair("assist_attention_18_10", &json!({}), &bell(&w, backup));
        // An open garden on the tile: E2E-40 up to the day before the closing (balance 11,400).
        let (garden_world, _) = garden_40();
        pair("dashboard_get_garden", &json!({}), &dash(&garden_world));
    }

    #[test]
    fn the_screen_state_fixtures_are_the_real_answers() {
        let w = worked_day();
        let user = w.owner.user_id;
        let request = json!({ "screen": "report.r20", "value": "{\"city_id\":3}" });
        screen_prefs::set(&w.env.ctx, user, "report.r20", "{\"city_id\":3}").unwrap();
        let state = serde_json::to_value(json!({
            "screen": "report.r20",
            "value": screen_prefs::get(&w.env.ctx, user, "report.r20").unwrap(),
        }))
        .unwrap();
        pair("prefs_screen_set", &request, &state);
    }
}
