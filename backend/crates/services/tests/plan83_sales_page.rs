//! Plan 83 (PRD v1.47, FR-9.9, FR-9.1): the sales page that the tile آج کی بکری opens. One row for
//! each day of a period, the totals over the whole period, and the بکری behind a day. Every expected
//! number is the PRD's worked day (4.4, 4.5, 4.9, E2E-13, E2E-15, E2E-16b, E2E-34, E2E-65) or worked
//! out in the comments, and is never changed to suit the code.
//!
//! The figures of the page are the tile's (`sales_by_day`) and R2's (the sales register) for the same
//! days: the tests below read all three and compare them.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::{raw, set_today};
use mandi_reports::analytics::AnalyticsGoods;
use mandi_reports::dashboard::{
    self, BackupFacts, CommissionParty, CommissionPeriod, CommissionPeriodKind, CommissionSide,
    SalesDayInput, SalesPeriodInput, SalesPrintInput,
};
use mandi_reports::reports::{Cell, ReportFilters, ReportId};
use mandi_services::bikri;
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{
    CancelInput, EditOpenInput, EditPreviewInput, EditSaveInput, LaterSaleSaveInput, StockListInput,
};
use mandi_services::core::error::ErrorCode;
use reports_world::*;
use support::World;

fn off() -> BackupFacts {
    BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    }
}

fn period(kind: CommissionPeriodKind) -> CommissionPeriod {
    CommissionPeriod {
        kind,
        ..CommissionPeriod::default()
    }
}

fn input(period: Option<CommissionPeriod>) -> SalesPeriodInput {
    SalesPeriodInput {
        period,
        ..SalesPeriodInput::default()
    }
}

fn page(w: &World, input: &SalesPeriodInput) -> dashboard::SalesPeriod {
    dashboard::sales_period(w.env.ctx.db(), input).unwrap()
}

fn day_of(w: &World, date: &str) -> dashboard::SalesDay {
    dashboard::sales_day(
        w.env.ctx.db(),
        &SalesDayInput {
            date: date.into(),
            offset: 0,
            limit: 0,
            party: None,
            item_id: None,
            goods: None,
        },
    )
    .unwrap()
}

/// A row as whole rupees: (بکری, نگ, خام, buyer total, cash, credit).
fn figures(row: &dashboard::SalesPeriodRow) -> (i64, i64, i64, i64, i64, i64) {
    (
        row.bikri_count,
        row.qty,
        row.gross_paisa.rupees(),
        row.buyer_total_paisa.rupees(),
        row.cash_paisa.rupees(),
        row.credit_paisa.rupees(),
    )
}

fn refused(w: &World, input: &SalesPeriodInput) -> Vec<(ErrorCode, String)> {
    match dashboard::sales_period(w.env.ctx.db(), input) {
        Err(mandi_reports::ReportsError::Rejected(found)) => {
            found.into_iter().map(|e| (e.code, e.field)).collect()
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

fn tile(w: &World) -> (i64, i64) {
    let t = dashboard::get(w.env.ctx.db(), off()).unwrap().tiles;
    (t.sales_gross_paisa.rupees(), t.sales_count)
}

/// The totals of the sales registers over a range of days: R2 gives the نگ and the buyer total, R3 the
/// money paid on the spot and R4 what is left on account: (نگ, buyer total, paid, left).
fn r2_totals(w: &World, from: &str, to: &str) -> (i64, i64, i64, i64) {
    let total = |id: ReportId, key: &str| {
        let r = report(
            w,
            id,
            ReportFilters {
                from: from.into(),
                to: to.into(),
                ..ReportFilters::default()
            },
            0,
            100,
        );
        let at = r.columns.iter().position(|c| c.key == key).unwrap();
        match &r.totals[at] {
            Cell::Number { value } => *value,
            Cell::Money { paisa } => paisa.rupees(),
            other => panic!("{key}: {other:?}"),
        }
    };
    (
        total(ReportId::R2, "qty"),
        total(ReportId::R2, "buyer_total"),
        total(ReportId::R3, "paid"),
        total(ReportId::R4, "left"),
    )
}

// ---- The worked day (PRD 4.4, 4.5, 4.9) --------------------------------------------------------

#[test]
fn the_month_page_has_the_worked_numbers_and_todays_row_is_the_tile() {
    let w = full_day();
    let month = page(&w, &SalesPeriodInput::default());
    assert_eq!(month.month, "2026-10");
    assert_eq!(month.current_month, "2026-10");
    assert_eq!(month.first_month.as_deref(), Some("2026-10"));
    assert_eq!(month.today, "2026-10-10");
    assert_eq!(month.period.kind, CommissionPeriodKind::Month);
    assert_eq!(
        (month.period.from.as_str(), month.period.to.as_str()),
        ("2026-10-01", "2026-10-31")
    );
    assert_eq!(month.rows.len(), 1);
    assert_eq!(month.rows[0].date, "2026-10-10");
    // 2 بکری, 60 نگ, 41,400, buyer total 12,880 + 18,684 + 12,800 = 44,364, cash 12,800 (شاہد, چکتا),
    // credit 44,364 − 12,800 = 31,564.
    assert_eq!(
        figures(&month.rows[0]),
        (2, 60, 41_400, 44_364, 12_800, 31_564)
    );
    assert_eq!(
        figures(&month.totals),
        (2, 60, 41_400, 44_364, 12_800, 31_564)
    );
    assert_eq!(month.totals.date, "");
    assert_eq!(month.total_count, 1);
    // The row of today is the tile: 41,400 in 2 بکری.
    assert_eq!(tile(&w), (41_400, 2));
    assert_eq!(
        (
            month.rows[0].gross_paisa.rupees(),
            month.rows[0].bikri_count
        ),
        tile(&w)
    );
}

#[test]
fn the_month_total_is_the_total_of_r2_for_the_same_days() {
    let w = full_day();
    let month = page(&w, &SalesPeriodInput::default());
    let (qty, buyer_total, paid, left) = r2_totals(&w, "01-10-2026", "31-10-2026");
    assert_eq!(
        (
            month.totals.qty,
            month.totals.buyer_total_paisa.rupees(),
            month.totals.cash_paisa.rupees(),
            month.totals.credit_paisa.rupees()
        ),
        (qty, buyer_total, paid, left)
    );
    assert_eq!((qty, buyer_total, paid, left), (60, 44_364, 12_800, 31_564));
}

#[test]
fn a_day_opens_into_its_bikri_with_their_lot_names_and_adds_up_to_its_row() {
    let w = full_day();
    let day = day_of(&w, "2026-10-10");
    assert_eq!(day.date, "2026-10-10");
    assert_eq!(day.total_count, 2);
    assert_eq!(day.rows.len(), 2);
    let (first, second) = (&day.rows[0], &day.rows[1]);
    // بکری 1: رحیم، کیلا-1: 50 نگ, 29,400, 31,564, nothing paid.
    assert_eq!(first.bikri_number, 1);
    assert_eq!(
        (
            first.qty,
            first.gross_paisa.rupees(),
            first.buyer_total_paisa.rupees(),
            first.cash_paisa.rupees(),
            first.credit_paisa.rupees()
        ),
        (50, 29_400, 31_564, 0, 31_564)
    );
    assert_eq!(first.sellers.len(), 1);
    assert_eq!(first.sellers[0].name_en, "Rahim");
    assert_eq!(first.lots.len(), 1);
    assert_eq!(first.lots[0].item.name_en, "Banana");
    assert_eq!(first.lots[0].counter, 1);
    assert_eq!(first.lots[0].seller.id, first.sellers[0].id);
    // بکری 2: کریم، سیب-1: 10 نگ, 12,000, 12,800, all paid on the spot.
    assert_eq!(second.bikri_number, 2);
    assert_eq!(
        (
            second.qty,
            second.gross_paisa.rupees(),
            second.buyer_total_paisa.rupees(),
            second.cash_paisa.rupees(),
            second.credit_paisa.rupees()
        ),
        (10, 12_000, 12_800, 12_800, 0)
    );
    assert_eq!(second.sellers[0].name_en, "Karim");
    assert_eq!(second.lots[0].item.name_en, "Apple");
    assert_ne!(first.bikri_id, second.bikri_id);
    // The day's figures are the row of the period page, and the rows add up to them.
    let month = page(&w, &SalesPeriodInput::default());
    let mut same = month.rows[0].clone();
    same.date = "2026-10-10".into();
    assert_eq!(day.totals, same);
    let sum: i64 = day.rows.iter().map(|r| r.gross_paisa.rupees()).sum();
    assert_eq!(sum, 41_400);
    // The day also opens with DD-MM-YYYY and with a shortcut (PRD FR-7.19).
    assert_eq!(day_of(&w, "10-10-2026").totals, day.totals);
}

#[test]
fn money_paid_on_the_spot_splits_the_buyer_total_into_cash_and_credit_e2e_65() {
    // D8 image 1: 200 آم, three lines. مانی pays 21,360 of 21,360, انس 10,000 of 48,100, ہارون 0 of
    // 125,060. Buyer total 194,520, cash 31,360, credit 163,160, خام بکری 182,000.
    let (w, _bikri) = d8_paid_world();
    let month = page(&w, &SalesPeriodInput::default());
    assert_eq!(month.rows.len(), 1);
    assert_eq!(
        figures(&month.rows[0]),
        (1, 200, 182_000, 194_520, 31_360, 163_160)
    );
    assert_eq!(tile(&w), (182_000, 1));
    let day = day_of(&w, "2026-10-10");
    assert_eq!(day.rows.len(), 1);
    assert_eq!(
        (
            day.rows[0].cash_paisa.rupees(),
            day.rows[0].credit_paisa.rupees()
        ),
        (31_360, 163_160)
    );
    let (qty, buyer_total, paid, left) = r2_totals(&w, "01-10-2026", "31-10-2026");
    assert_eq!(
        (qty, buyer_total, paid, left),
        (200, 194_520, 31_360, 163_160)
    );
}

// ---- Later sales, edits, cancels (PRD FR-18.7, E2E-13, E2E-15, E2E-16b) ------------------------

#[test]
fn a_later_sale_counts_on_the_day_it_was_sold_and_its_row_is_the_tile_e2e_16b() {
    let w = worked_day();
    let mut part = support::event1(&w);
    part.lots[0].sales[1].qty = "25".into();
    save(&w, &part);
    // 10-10-26: 45 of 50 sold. خام 26,500; buyer total (12,000 + 720 + 160) + (14,500 + 870 + 200)
    // = 12,880 + 15,570 = 28,450.
    let ten = page(&w, &SalesPeriodInput::default());
    assert_eq!(figures(&ten.totals), (1, 45, 26_500, 28_450, 0, 28_450));
    assert_eq!(tile(&w), (26_500, 1));

    // 11-10-26: the last 5 are sold to ارشد at 560: 2,800, buyer total 2,800 + 168 + 40 = 3,008.
    set_today(&w.env, 2026, 10, 11);
    let lot_id = bikri::stock_list(&w.env.ctx, &StockListInput::default())
        .unwrap()
        .rows[0]
        .lot_id;
    let mut line = support::sale("s-3", w.arshad, "5", "560");
    line.sale_id = None;
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // The later sale of an old بکری is a row of its own day: 1 بکری, 5 نگ, 2,800, 3,008.
    let month = page(&w, &SalesPeriodInput::default());
    assert_eq!(month.rows.len(), 2);
    assert_eq!(month.rows[0].date, "2026-10-10");
    assert_eq!(figures(&month.rows[0]), (1, 45, 26_500, 28_450, 0, 28_450));
    assert_eq!(month.rows[1].date, "2026-10-11");
    assert_eq!(figures(&month.rows[1]), (1, 5, 2_800, 3_008, 0, 3_008));
    // The tile of today (11-10-26) is the row of 11-10-26.
    assert_eq!(tile(&w), (2_800, 1));
    assert_eq!(
        (
            month.rows[1].gross_paisa.rupees(),
            month.rows[1].bikri_count
        ),
        tile(&w)
    );
    // The month totals: 2 rows added, as R2.
    assert_eq!(figures(&month.totals), (2, 50, 29_300, 31_458, 0, 31_458));
    let (qty, buyer_total, paid, left) = r2_totals(&w, "01-10-2026", "31-10-2026");
    assert_eq!(
        (
            month.totals.qty,
            month.totals.buyer_total_paisa.rupees(),
            month.totals.cash_paisa.rupees(),
            month.totals.credit_paisa.rupees()
        ),
        (qty, buyer_total, paid, left)
    );
    // The day of the later sale opens into the same بکری 1, with this day's figures only.
    let day = day_of(&w, "2026-10-11");
    assert_eq!(day.total_count, 1);
    assert_eq!(day.rows[0].bikri_number, 1);
    assert_eq!(
        (day.rows[0].qty, day.rows[0].gross_paisa.rupees()),
        (5, 2_800)
    );
    assert_eq!(day.rows[0].lots[0].counter, 1);
}

#[test]
fn an_edit_replaces_the_figures_of_its_day_and_a_cancel_takes_a_bikri_out_e2e_13_e2e_15() {
    let w = worked_day();
    let saved = save(&w, &support::event1(&w));
    save(&w, &event2(&w));
    // E2E-13: اکرم's rate 600 → 610: his line 12,200 + 732 + 160 = 13,092; خام 29,600.
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: opened.entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
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
    // The reversal and the new posting are on the same day: the old values are gone, the count is
    // still 2, and the figures are those of the edited sale: 29,600 + 12,000, 13,092 + 18,684 + 12,800.
    let month = page(&w, &SalesPeriodInput::default());
    assert_eq!(
        figures(&month.totals),
        (2, 60, 41_600, 44_576, 12_800, 31_776)
    );
    assert_eq!(tile(&w), (41_600, 2));
    assert_eq!(
        r2_totals(&w, "01-10-2026", "31-10-2026"),
        (60, 44_576, 12_800, 31_776)
    );

    // E2E-15: بکری 2 is cancelled. Only بکری 1 is left, and its day opens one بکری.
    let detail = bikri::get(&w.env.ctx, 2).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: 2,
            version: detail.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let month = page(&w, &SalesPeriodInput::default());
    assert_eq!(figures(&month.totals), (1, 50, 29_600, 31_776, 0, 31_776));
    assert_eq!(tile(&w), (29_600, 1));
    let day = day_of(&w, "2026-10-10");
    assert_eq!(day.rows.len(), 1);
    assert_eq!(day.rows[0].bikri_number, 1);
    assert_eq!(
        r2_totals(&w, "01-10-2026", "31-10-2026"),
        (50, 31_776, 0, 31_776)
    );
}

#[test]
fn a_month_with_every_bikri_cancelled_has_no_rows_and_zero_totals() {
    let w = worked_day();
    let saved = save(&w, &support::event1(&w));
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: saved.bikri_id,
            version: detail.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let month = page(&w, &SalesPeriodInput::default());
    assert!(month.rows.is_empty());
    assert_eq!(month.total_count, 0);
    assert_eq!(figures(&month.totals), (0, 0, 0, 0, 0, 0));
    assert_eq!(day_of(&w, "2026-10-10").total_count, 0);
}

#[test]
fn an_empty_book_gives_an_empty_page() {
    let w = worked_day();
    let month = page(&w, &SalesPeriodInput::default());
    assert!(month.rows.is_empty());
    assert_eq!(month.first_month, None);
    assert_eq!(figures(&month.totals), (0, 0, 0, 0, 0, 0));
}

// ---- Edits that move or change a line (the money review of plan 83) ----------------------------

/// Opens the first posting of a بکری, changes its draft and saves the edit. Returns the journal
/// entry of the new first posting, to edit again.
fn edit_first(
    w: &World,
    bikri_id: i64,
    entry_id: i64,
    change: impl FnOnce(&mut mandi_services::core::bikri::BikriDraft),
) -> i64 {
    let opened = bikri::open_for_edit(&w.env.ctx, &EditOpenInput { bikri_id, entry_id }).unwrap();
    let mut draft = opened.draft.clone();
    change(&mut draft);
    bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("درستی".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .entry_id
}

#[test]
fn an_edit_that_changes_only_the_paid_amount_moves_cash_and_credit_and_nothing_else() {
    // اکرم pays 5,000 of his 12,880 on the spot after the save. The buyer total stays 31,564; the
    // cash is 5,000 and the credit 31,564 − 5,000 = 26,564, as R3 (paid) and R4 (left).
    let w = worked_day();
    let saved = save(&w, &support::event1(&w));
    edit_first(&w, saved.bikri_id, saved.entry_id, |d| {
        d.lots[0].sales[0].paid = Some("5,000".into());
    });
    let month = page(&w, &SalesPeriodInput::default());
    assert_eq!(
        figures(&month.totals),
        (1, 50, 29_400, 31_564, 5_000, 26_564)
    );
    assert_eq!(
        r2_totals(&w, "01-10-2026", "31-10-2026"),
        (50, 31_564, 5_000, 26_564)
    );
    let akram = page(
        &w,
        &SalesPeriodInput {
            party: buyer(w.akram),
            ..SalesPeriodInput::default()
        },
    );
    assert_eq!(
        figures(&akram.totals),
        (1, 20, 12_000, 12_880, 5_000, 7_880)
    );
}

#[test]
fn a_line_that_moves_to_another_buyer_or_seller_moves_with_the_filters_e2e_68() {
    use mandi_services::core::accounts::AccountType;
    let w = worked_day();
    let saved = save(&w, &support::event1(&w));
    let tariq = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "طارق",
        "Tariq",
        true,
        "",
    );
    // The first line goes to طارق: the shop's figures do not change, the buyers' do.
    let entry = edit_first(&w, saved.bikri_id, saved.entry_id, |d| {
        d.lots[0].sales[0].buyer_account_id = Some(tariq);
    });
    let with = |party| {
        page(
            &w,
            &SalesPeriodInput {
                party,
                ..SalesPeriodInput::default()
            },
        )
    };
    assert_eq!(
        figures(&with(None).totals),
        (1, 50, 29_400, 31_564, 0, 31_564)
    );
    assert!(with(buyer(w.akram)).rows.is_empty());
    assert_eq!(
        figures(&with(buyer(tariq)).totals),
        (1, 20, 12_000, 12_880, 0, 12_880)
    );
    // The lot goes to کریم: the figures of the seller filters follow the lot.
    edit_first(&w, saved.bikri_id, entry, |d| {
        d.lots[0].seller_account_id = Some(w.karim);
    });
    assert!(with(seller(w.rahim)).rows.is_empty());
    let karim = with(seller(w.karim));
    assert_eq!(
        (karim.totals.qty, karim.totals.gross_paisa.rupees()),
        (50, 29_400)
    );
    assert_eq!(figures(&with(None).totals).0, 1);
}

// ---- Periods ------------------------------------------------------------------------------------

/// The worked day, and the last 5 نگ sold on 11-10-26 (two days with sales).
fn two_days() -> World {
    let w = worked_day();
    let mut part = support::event1(&w);
    part.lots[0].sales[1].qty = "25".into();
    save(&w, &part);
    set_today(&w.env, 2026, 10, 11);
    let lot_id = bikri::stock_list(&w.env.ctx, &StockListInput::default())
        .unwrap()
        .rows[0]
        .lot_id;
    let mut line = support::sale("s-3", w.arshad, "5", "560");
    line.sale_id = None;
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    w
}

#[test]
fn cancelling_a_bikri_that_had_a_later_sale_takes_both_days_out() {
    let w = two_days();
    let detail = bikri::get(&w.env.ctx, 1).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: 1,
            version: detail.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let month = page(&w, &SalesPeriodInput::default());
    assert!(month.rows.is_empty());
    assert_eq!(figures(&month.totals), (0, 0, 0, 0, 0, 0));
    assert_eq!(day_of(&w, "2026-10-10").total_count, 0);
    assert_eq!(day_of(&w, "2026-10-11").total_count, 0);
    assert_eq!(r2_totals(&w, "01-10-2026", "31-10-2026"), (0, 0, 0, 0));
}

#[test]
fn a_bikri_sold_in_two_months_counts_once_in_each_month_row_of_a_year() {
    // 45 نگ on 10-10-26 and the last 5 on 11-11-26: the same بکری 1 in two month rows.
    let w = worked_day();
    let mut part = support::event1(&w);
    part.lots[0].sales[1].qty = "25".into();
    save(&w, &part);
    set_today(&w.env, 2026, 11, 11);
    let lot_id = bikri::stock_list(&w.env.ctx, &StockListInput::default())
        .unwrap()
        .rows[0]
        .lot_id;
    let mut line = support::sale("s-3", w.arshad, "5", "560");
    line.sale_id = None;
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: "11-11-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let year = page(
        &w,
        &input(Some(CommissionPeriod {
            year: Some(2026),
            ..period(CommissionPeriodKind::Year)
        })),
    );
    assert_eq!(year.rows.len(), 2);
    assert_eq!(year.rows[0].date, "2026-10");
    assert_eq!(figures(&year.rows[0]), (1, 45, 26_500, 28_450, 0, 28_450));
    assert_eq!(year.rows[1].date, "2026-11");
    assert_eq!(figures(&year.rows[1]), (1, 5, 2_800, 3_008, 0, 3_008));
    // The totals row is the sum of the rows, as every other column.
    assert_eq!(figures(&year.totals), (2, 50, 29_300, 31_458, 0, 31_458));
    // Each month alone is the same figures as its row.
    let november = page(&w, &SalesPeriodInput::default());
    assert_eq!(november.month, "2026-11");
    assert_eq!(figures(&november.totals), (1, 5, 2_800, 3_008, 0, 3_008));
}

#[test]
fn a_year_has_a_row_for_each_month_and_a_day_period_has_one_row() {
    let w = two_days();
    let year = page(
        &w,
        &input(Some(CommissionPeriod {
            year: Some(2026),
            ..period(CommissionPeriodKind::Year)
        })),
    );
    assert_eq!(year.period.kind, CommissionPeriodKind::Year);
    assert_eq!(
        (year.period.from.as_str(), year.period.to.as_str()),
        ("2026-01-01", "2026-12-31")
    );
    // One row for the month, with the sums of both days; the بکری 1 counts once in October.
    assert_eq!(year.rows.len(), 1);
    assert_eq!(year.rows[0].date, "2026-10");
    assert_eq!(figures(&year.rows[0]), (1, 50, 29_300, 31_458, 0, 31_458));
    assert_eq!(figures(&year.totals), (1, 50, 29_300, 31_458, 0, 31_458));

    let day = page(
        &w,
        &input(Some(CommissionPeriod {
            date: Some("11-10-2026".into()),
            ..period(CommissionPeriodKind::Day)
        })),
    );
    assert_eq!(day.rows.len(), 1);
    assert_eq!(day.rows[0].date, "2026-10-11");
    assert_eq!(figures(&day.rows[0]), (1, 5, 2_800, 3_008, 0, 3_008));
}

#[test]
fn a_range_is_paged_and_its_totals_cover_the_whole_range() {
    let w = two_days();
    let range = |offset: u32, limit: u32| {
        page(
            &w,
            &SalesPeriodInput {
                period: Some(CommissionPeriod {
                    from: Some("01-10-2026".into()),
                    to: Some("11-10-2026".into()),
                    ..period(CommissionPeriodKind::Range)
                }),
                offset: Some(offset),
                limit: Some(limit),
                ..SalesPeriodInput::default()
            },
        )
    };
    let first = range(0, 1);
    assert_eq!(first.total_count, 2);
    assert_eq!(first.rows.len(), 1);
    assert_eq!(first.rows[0].date, "2026-10-10");
    assert_eq!((first.offset, first.limit), (0, 1));
    // The totals are the whole range, never the page.
    assert_eq!(figures(&first.totals), (2, 50, 29_300, 31_458, 0, 31_458));
    let next = range(1, 1);
    assert_eq!(next.rows.len(), 1);
    assert_eq!(next.rows[0].date, "2026-10-11");
    assert_eq!((next.offset, next.limit), (1, 1));
    assert_eq!(next.totals, first.totals);
    // A page past the end has no rows and the same totals.
    let past = range(5, 1);
    assert!(past.rows.is_empty());
    assert_eq!(past.total_count, 2);
}

#[test]
fn a_future_date_a_bad_range_and_unknown_records_are_refused_at_their_field() {
    let w = full_day();
    // Today is 10-10-26: 11-10-26, November and 2027 are in the future.
    let future_day = input(Some(CommissionPeriod {
        date: Some("11-10-2026".into()),
        ..period(CommissionPeriodKind::Day)
    }));
    assert_eq!(
        refused(&w, &future_day),
        vec![(ErrorCode::ErrDateInFuture, "period.date".to_owned())]
    );
    let future_month = input(Some(CommissionPeriod {
        month: Some("2026-11".into()),
        ..period(CommissionPeriodKind::Month)
    }));
    assert_eq!(
        refused(&w, &future_month),
        vec![(ErrorCode::ErrDateInFuture, "period.month".to_owned())]
    );
    let future_year = input(Some(CommissionPeriod {
        year: Some(2027),
        ..period(CommissionPeriodKind::Year)
    }));
    assert_eq!(
        refused(&w, &future_year),
        vec![(ErrorCode::ErrDateInFuture, "period.year".to_owned())]
    );
    let bad_month = input(Some(CommissionPeriod {
        month: Some("10-2026".into()),
        ..period(CommissionPeriodKind::Month)
    }));
    assert_eq!(
        refused(&w, &bad_month),
        vec![(ErrorCode::ErrFormatInvalid, "period.month".to_owned())]
    );
    let backwards = input(Some(CommissionPeriod {
        from: Some("10-10-2026".into()),
        to: Some("01-10-2026".into()),
        ..period(CommissionPeriodKind::Range)
    }));
    assert_eq!(
        refused(&w, &backwards),
        vec![(ErrorCode::ErrDateRange, "period.to".to_owned())]
    );
    let missing = input(Some(period(CommissionPeriodKind::Day)));
    assert_eq!(
        refused(&w, &missing),
        vec![(ErrorCode::ErrRequired, "period.date".to_owned())]
    );
    // A party that does not exist, one of the wrong side, and an item that does not exist.
    let unknown_party = SalesPeriodInput {
        party: Some(CommissionParty {
            side: CommissionSide::Buyer,
            account_id: 99_999,
            chukta_person_key: None,
        }),
        ..SalesPeriodInput::default()
    };
    assert_eq!(
        refused(&w, &unknown_party),
        vec![(ErrorCode::ErrNotFound, "party.account_id".to_owned())]
    );
    let wrong_side = SalesPeriodInput {
        party: Some(CommissionParty {
            side: CommissionSide::Seller,
            account_id: w.akram,
            chukta_person_key: None,
        }),
        ..SalesPeriodInput::default()
    };
    assert_eq!(
        refused(&w, &wrong_side),
        vec![(ErrorCode::ErrWrongPartyType, "party.account_id".to_owned())]
    );
    let unknown_item = SalesPeriodInput {
        item_id: Some(99_999),
        ..SalesPeriodInput::default()
    };
    assert_eq!(
        refused(&w, &unknown_item),
        vec![(ErrorCode::ErrNotFound, "item_id".to_owned())]
    );
    // Today itself is read; the paging limits are checked.
    page(
        &w,
        &input(Some(CommissionPeriod {
            date: Some("10-10-2026".into()),
            ..period(CommissionPeriodKind::Day)
        })),
    );
    let too_many = SalesPeriodInput {
        limit: Some(5_000),
        ..SalesPeriodInput::default()
    };
    assert_eq!(
        refused(&w, &too_many),
        vec![(ErrorCode::ErrOutOfRange, "limit".to_owned())]
    );
}

#[test]
fn a_future_date_is_refused_on_the_day_page_at_its_field() {
    let w = full_day();
    for (date, code) in [
        ("11-10-2026", ErrorCode::ErrDateInFuture),
        ("2026-10-11", ErrorCode::ErrDateInFuture),
        ("", ErrorCode::ErrRequired),
    ] {
        let result = dashboard::sales_day(
            w.env.ctx.db(),
            &SalesDayInput {
                date: date.into(),
                offset: 0,
                limit: 0,
                party: None,
                item_id: None,
                goods: None,
            },
        );
        match result {
            Err(mandi_reports::ReportsError::Rejected(found)) => {
                assert_eq!(
                    found
                        .into_iter()
                        .map(|e| (e.code, e.field))
                        .collect::<Vec<_>>(),
                    vec![(code, "date".to_owned())],
                    "{date}"
                );
            }
            other => panic!("{date}: expected a refusal, got {other:?}"),
        }
    }
    let bad = dashboard::sales_day(
        w.env.ctx.db(),
        &SalesDayInput {
            date: "31-02-2026".into(),
            offset: 0,
            limit: 0,
            party: None,
            item_id: None,
            goods: None,
        },
    );
    assert!(matches!(bad, Err(mandi_reports::ReportsError::Rejected(_))));
}

// ---- Filters: party, item, goods ---------------------------------------------------------------

fn buyer(id: i64) -> Option<CommissionParty> {
    Some(CommissionParty {
        side: CommissionSide::Buyer,
        account_id: id,
        chukta_person_key: None,
    })
}

fn seller(id: i64) -> Option<CommissionParty> {
    Some(CommissionParty {
        side: CommissionSide::Seller,
        account_id: id,
        chukta_person_key: None,
    })
}

#[test]
fn a_buyer_or_a_seller_narrows_the_rows_and_the_day_to_his_lines() {
    let w = full_day();
    let with = |party| {
        page(
            &w,
            &SalesPeriodInput {
                party,
                ..SalesPeriodInput::default()
            },
        )
    };
    // اکرم: 20 نگ of کیلا at 600: خام 12,000, buyer total 12,880, nothing paid.
    let akram = with(buyer(w.akram));
    assert_eq!(akram.party.as_ref().unwrap().party.account_id, w.akram);
    assert_eq!(akram.rows.len(), 1);
    assert_eq!(figures(&akram.rows[0]), (1, 20, 12_000, 12_880, 0, 12_880));
    // ارشد: 30 نگ at 580: 17,400, 18,684.
    assert_eq!(
        figures(&with(buyer(w.arshad)).totals),
        (1, 30, 17_400, 18_684, 0, 18_684)
    );
    // رحیم, a seller: the whole بکری 1; کریم: بکری 2 with his 12,800 paid on the spot.
    assert_eq!(
        figures(&with(seller(w.rahim)).totals),
        (1, 50, 29_400, 31_564, 0, 31_564)
    );
    assert_eq!(
        figures(&with(seller(w.karim)).totals),
        (1, 10, 12_000, 12_800, 12_800, 0)
    );
    // The two sellers add up to the whole shop.
    let both = with(None);
    let (r, k) = (with(seller(w.rahim)).totals, with(seller(w.karim)).totals);
    assert_eq!(r.qty + k.qty, both.totals.qty);
    assert_eq!(
        r.buyer_total_paisa.rupees() + k.buyer_total_paisa.rupees(),
        both.totals.buyer_total_paisa.rupees()
    );
    // The day of a buyer holds only his بکری and his line.
    let day = dashboard::sales_day(
        w.env.ctx.db(),
        &SalesDayInput {
            date: "2026-10-10".into(),
            offset: 0,
            limit: 0,
            party: buyer(w.akram),
            item_id: None,
            goods: None,
        },
    )
    .unwrap();
    assert_eq!(day.total_count, 1);
    assert_eq!(day.rows[0].bikri_number, 1);
    assert_eq!(
        (day.rows[0].qty, day.rows[0].buyer_total_paisa.rupees()),
        (20, 12_880)
    );
    assert_eq!(day.totals, {
        let mut row = akram.rows[0].clone();
        row.date = "2026-10-10".into();
        row
    });
    assert_eq!(day.party.as_ref().unwrap().party.account_id, w.akram);
}

#[test]
fn a_chukta_buyer_is_read_by_his_person_key_on_the_account_0030() {
    let w = full_day();
    let facts = raw(&w.dir);
    let key: String = facts
        .query_row(
            "SELECT chukta_person_key FROM analytics_facts WHERE chukta_person_key IS NOT NULL LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let shown = page(
        &w,
        &SalesPeriodInput {
            party: Some(CommissionParty {
                side: CommissionSide::Buyer,
                account_id: w.chukta_khata,
                chukta_person_key: Some(key),
            }),
            ..SalesPeriodInput::default()
        },
    );
    // شاہد: 10 نگ, خام 12,000, 12,800 paid in full.
    assert_eq!(figures(&shown.totals), (1, 10, 12_000, 12_800, 12_800, 0));
    assert_eq!(
        shown.party.as_ref().unwrap().party.name_en.as_str(),
        "Shahid"
    );
}

#[test]
fn an_item_narrows_the_rows_and_the_two_items_add_up_to_the_shop() {
    let w = full_day();
    let item = |id| {
        page(
            &w,
            &SalesPeriodInput {
                item_id: Some(id),
                ..SalesPeriodInput::default()
            },
        )
    };
    let banana = item(w.banana);
    assert_eq!(banana.item.as_ref().unwrap().name_en, "Banana");
    assert_eq!(figures(&banana.totals), (1, 50, 29_400, 31_564, 0, 31_564));
    let apple = item(w.apple);
    assert_eq!(figures(&apple.totals), (1, 10, 12_000, 12_800, 12_800, 0));
    // An item nobody sold has no rows and zero totals.
    let mango = support::add_item(&w.env, &w.owner, "آم", "Mango", "fruit", "unit");
    let none = item(mango);
    assert!(none.rows.is_empty());
    assert_eq!(figures(&none.totals), (0, 0, 0, 0, 0, 0));
}

#[test]
fn the_goods_kinds_add_up_to_all_goods_with_the_worked_numbers() {
    // Consigned: بکری 1 of the worked day on 10-10-26 (50 نگ, 29,400, 31,564). Trip goods, E2E-34:
    // 15 پیٹی مالٹا at 600 on 10-10-26: 9,000, 9,000 + 540 + 120 = 9,660. Garden goods on 11-10-26:
    // 10 پیٹی کیلا at 600: 6,000, 6,000 + 360 + 80 = 6,440.
    let (w, _trip, _purchase, _sale) = trip_34_world();
    save(&w, &support::event1(&w));
    let garden = garden_made(&w);
    set_today(&w.env, 2026, 10, 11);
    garden_lot_sale(&w, garden.account.id, "11-10-2026", "10", w.arshad, "600");
    let kind = |goods| {
        page(
            &w,
            &SalesPeriodInput {
                goods,
                ..SalesPeriodInput::default()
            },
        )
    };
    let all = kind(None);
    assert_eq!(kind(Some(AnalyticsGoods::All)).totals, all.totals);
    assert_eq!(all.rows.len(), 2);
    assert_eq!(figures(&all.rows[0]), (2, 65, 38_400, 41_224, 0, 41_224));
    assert_eq!(figures(&all.rows[1]), (1, 10, 6_000, 6_440, 0, 6_440));
    let consigned = kind(Some(AnalyticsGoods::Consigned));
    assert_eq!(
        figures(&consigned.totals),
        (1, 50, 29_400, 31_564, 0, 31_564)
    );
    let trip = kind(Some(AnalyticsGoods::Trip));
    assert_eq!(trip.goods, AnalyticsGoods::Trip);
    assert_eq!(figures(&trip.totals), (1, 15, 9_000, 9_660, 0, 9_660));
    let garden = kind(Some(AnalyticsGoods::Garden));
    assert_eq!(garden.rows.len(), 1);
    assert_eq!(garden.rows[0].date, "2026-10-11");
    assert_eq!(figures(&garden.totals), (1, 10, 6_000, 6_440, 0, 6_440));
    // The three kinds are a partition: they add up to all goods in every column.
    let sum = |f: fn(&dashboard::SalesPeriodRow) -> i64| {
        f(&consigned.totals) + f(&trip.totals) + f(&garden.totals)
    };
    assert_eq!(sum(|r| r.qty), all.totals.qty);
    assert_eq!(
        sum(|r| r.gross_paisa.rupees()),
        all.totals.gross_paisa.rupees()
    );
    assert_eq!(
        sum(|r| r.buyer_total_paisa.rupees()),
        all.totals.buyer_total_paisa.rupees()
    );
    assert_eq!(sum(|r| r.bikri_count), 3);
    // The day of the trip goods holds that بکری only.
    let day = dashboard::sales_day(
        w.env.ctx.db(),
        &SalesDayInput {
            date: "2026-10-10".into(),
            offset: 0,
            limit: 0,
            party: None,
            item_id: None,
            goods: Some(AnalyticsGoods::Trip),
        },
    )
    .unwrap();
    assert_eq!(day.total_count, 1);
    assert_eq!(day.rows[0].qty, 15);
    assert_eq!(day.goods, AnalyticsGoods::Trip);
}

// ---- Paging of a day ---------------------------------------------------------------------------

#[test]
fn the_bikri_of_a_day_are_paged_and_their_totals_cover_the_whole_day() {
    let w = full_day();
    let one = |offset: u32| {
        dashboard::sales_day(
            w.env.ctx.db(),
            &SalesDayInput {
                date: "2026-10-10".into(),
                offset,
                limit: 1,
                party: None,
                item_id: None,
                goods: None,
            },
        )
        .unwrap()
    };
    let (first, second) = (one(0), one(1));
    assert_eq!((first.rows.len(), first.total_count), (1, 2));
    assert_eq!(first.rows[0].bikri_number, 1);
    assert_eq!(second.rows[0].bikri_number, 2);
    // The totals are the day, never the page.
    assert_eq!(first.totals, second.totals);
    assert_eq!(
        figures(&first.totals),
        (2, 60, 41_400, 44_364, 12_800, 31_564)
    );
    assert!(one(2).rows.is_empty());
}

// ---- Query plans (Code Standards B18, NFR-SPD-24) -----------------------------------------------

#[test]
fn the_sales_reads_use_the_date_index_and_the_key_of_the_sale_line_with_no_scan() {
    // A range of days is read from the facts through an index on the date, and each fact finds its
    // sale line by the key: ten years of the sale lines are never scanned.
    let w = full_day();
    let conn = raw(&w.dir);
    for (sql, args) in [
        dashboard::sales_plans::facts_by_day("2016-10-11", "2026-10-10"),
        dashboard::sales_plans::facts_narrowed_by_month(
            "2016-10-11",
            "2026-10-10",
            w.rahim,
            w.banana,
        ),
    ] {
        let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let plan = statement
            .query_map(
                mandi_services::db::rusqlite::params_from_iter(args.iter()),
                |r| r.get::<_, String>(3),
            )
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(plan.contains("SEARCH f USING"), "{plan}");
        assert!(plan.contains("entry_date>"), "{plan}");
        assert!(
            plan.contains("SEARCH s USING INTEGER PRIMARY KEY"),
            "{plan}"
        );
        assert!(!plan.contains("SCAN"), "{plan}");
    }
}

// ---- Print data and the sheet (FR-19.3) --------------------------------------------------------

fn text(cell: &mandi_reports::reports::PrintCell) -> String {
    use mandi_reports::reports::PrintCell;
    match cell {
        PrintCell::Money { text } | PrintCell::Number { text } => text.clone(),
        PrintCell::Date { text } => text.clone(),
        other => panic!("not an amount: {other:?}"),
    }
}

#[test]
fn the_print_holds_every_row_and_the_totals_of_the_period() {
    let w = two_days();
    let print = dashboard::sales_period_print_data(
        w.env.ctx.db(),
        &SalesPrintInput {
            period: Some(CommissionPeriod {
                from: Some("01-10-2026".into()),
                to: Some("11-10-2026".into()),
                ..period(CommissionPeriodKind::Range)
            }),
            party: None,
            item_id: None,
            goods: None,
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    let keys: Vec<&str> = print.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "date",
            "bikri_count",
            "qty",
            "gross",
            "buyer_total",
            "cash",
            "credit"
        ]
    );
    assert_eq!(print.rows.len(), 2);
    let amounts =
        |row: usize| -> Vec<String> { print.rows[row].cells[1..].iter().map(text).collect() };
    assert_eq!(amounts(0), ["1", "45", "26,500", "28,450", "0", "28,450"]);
    assert_eq!(amounts(1), ["1", "5", "2,800", "3,008", "0", "3,008"]);
    let totals: Vec<String> = print.totals[1..].iter().map(text).collect();
    assert_eq!(totals, ["2", "50", "29,300", "31,458", "0", "31,458"]);
    // A narrowed print names its filter and holds only the narrowed rows.
    let narrowed = dashboard::sales_period_print_data(
        w.env.ctx.db(),
        &SalesPrintInput {
            period: None,
            party: buyer(w.akram),
            item_id: Some(w.banana),
            goods: Some(AnalyticsGoods::Consigned),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(narrowed.rows.len(), 1);
    assert_eq!(
        narrowed.totals[1..].iter().map(text).collect::<Vec<_>>(),
        ["1", "20", "12,000", "12,880", "0", "12,880"]
    );
    assert!(narrowed.frame.header.facts.len() >= 4);
}
