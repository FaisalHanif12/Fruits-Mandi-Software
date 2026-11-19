//! Plan 79 item 3 (PRD v1.44, FR-9.1): every tile of the dashboard has its own page, and the total of
//! that page is the figure of the tile. One test for each row of the tile table of the plan
//! (Section 8), on the PRD worked day. The expected numbers are the PRD's (4.4, 4.5, 4.9, E2E-8,
//! E2E-16, E2E-28, E2E-30, E2E-34, E2E-40, E2E-44, FR-9.7, FR-9.8) and are never changed to suit the
//! code. A page that is read here is read from the same posted records as the tile, so a figure that
//! drifts apart shows as a failing test (PRD FR-6.1, FR-19.9).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::analytics::{AnalyticsFilters, AnalyticsRunInput, AnalyticsView};
use mandi_reports::dashboard::{
    self, BackupFacts, BuyerDuesInput, CashSalesDayInput, CashSalesMonthInput,
    CommissionMonthInput, DashboardTiles, LentOpenInput, SellerDuesInput, SellerDuesSide,
    TotalDuesSection,
};
use mandi_reports::reports::Cell;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{
    BikriDraft, BikriListInput, EditOpenInput, EditPreviewInput, EditSaveInput, StockListInput,
};
use mandi_services::core::cashbook::CashbookInput;
use mandi_services::core::dead::{DeadAmountInput, DeadFilter, DeadMarkInput, DeadReason};
use mandi_services::core::lent::{LentDuesInput, MoneyLentInput};
use mandi_services::core::vouchers::{AdvancesReportInput, ReceiptInput};
use mandi_services::{bikri, cashbook, dead, gardens, lent, trips, vouchers};
use reports_world::*;

fn tiles(w: &support::World) -> DashboardTiles {
    dashboard::get(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
    .tiles
}

fn seller_page(w: &support::World, side: SellerDuesSide) -> dashboard::SellerDuesPage {
    dashboard::seller_dues(
        w.env.ctx.db(),
        &SellerDuesInput {
            side,
            ..SellerDuesInput::default()
        },
    )
    .unwrap()
}

fn advances(w: &support::World, only_open: bool) -> mandi_services::core::vouchers::AdvancesReport {
    vouchers::advances_report(
        &w.env.ctx,
        &AdvancesReportInput {
            seller_id: None,
            as_of: String::new(),
            offset: 0,
            limit: 0,
            search: None,
            only_open,
        },
    )
    .unwrap()
}

/// Makes the person اسلم and lends him `amount` on 10-10-26 (PRD E2E-30). He is a person of the type
/// `other`, so he is on the page ادھار رقم باقی and on the page of the others.
fn lend(w: &support::World, amount: &str) -> i64 {
    let mut words: Vec<String> = Vec::new();
    let person = loop {
        let input = mandi_services::core::lent::LentPersonCreateInput {
            name_ur: "اسلم".into(),
            name_en: "Aslam".into(),
            phone: None,
            staff_id: None,
            confirmed_rules_words: words.clone(),
        };
        match lent::person_create(&w.env.ctx, &w.owner, &input) {
            Ok(person) => break person.account.id,
            Err(mandi_services::ServiceError::Rejected(items))
                if items.iter().all(|i| {
                    i.code == mandi_services::core::error::ErrorCode::ErrSpellingUnconfirmed
                }) =>
            {
                for item in items {
                    if let Some(mandi_services::core::error::ErrorValue::Text(text)) =
                        item.values.get("words")
                    {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    };
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &MoneyLentInput {
            date: "10-10-2026".into(),
            person_account_id: Some(person),
            amount: amount.into(),
            reason: "گھریلو ضرورت".into(),
            return_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    person
}

/// The last money cell of a totals row, in rupees.
fn last_total(cells: &[Cell]) -> i64 {
    match cells.last() {
        Some(Cell::Money { paisa }) => paisa.rupees(),
        other => panic!("not an amount: {other:?}"),
    }
}

// ---- آج کی بکری: the list of today's بکری (PRD 12.7, FR-12.46) -------------------------------------

#[test]
fn tile_sales_today_is_the_total_of_the_list_of_todays_bikri() {
    // 4.9: خام بکری 29,400 + 12,000 = 41,400 in 2 بکری.
    let w = full_day();
    let t = tiles(&w);
    let page = bikri::list(
        &w.env.ctx,
        &BikriListInput {
            date: Some("10-10-2026".into()),
            ..BikriListInput::default()
        },
    )
    .unwrap();
    assert_eq!((page.total_count, page.totals.gross.rupees()), (2, 41_400));
    assert_eq!(page.totals.gross, t.sales_gross_paisa);
    assert_eq!(page.total_count, t.sales_count);
}

// ---- آج کا کمیشن: the commission page (FR-9.3) ----------------------------------------------------

#[test]
fn tile_commission_today_is_the_commission_page_total() {
    // 4.9: commission 3,928 + 1,520 = 5,448; the shared income 5,488 adds the فون of both بکری.
    let w = full_day();
    let t = tiles(&w);
    let page = dashboard::commission_month(
        w.env.ctx.db(),
        &CommissionMonthInput {
            month: None,
            period: None,
            party: None,
            goods: None,
            offset: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(page.month, "2026-10");
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.rows[0].date, "2026-10-10");
    assert_eq!(page.totals.total_comm_paisa.rupees(), 5_448);
    assert_eq!(page.totals.shared_income_paisa.rupees(), 5_488);
    assert_eq!(page.totals.total_comm_paisa, t.commission_paisa);
}

// ---- موجود روکڑ: the cash book (FR-16.14) --------------------------------------------------------

#[test]
fn tile_cash_in_hand_is_the_closing_of_the_cash_book() {
    // E2E-8: cash in hand 58,484. The book opens on the day at 50,000 (the opening position).
    let w = full_day();
    let t = tiles(&w);
    let book = cashbook::get(&w.env.ctx, &CashbookInput::default()).unwrap();
    assert_eq!(book.closing_paisa.rupees(), 58_484);
    assert_eq!(book.closing_paisa, t.cash_in_hand_paisa);
    // The same book for the one day: opening 50,000, in 22,800, out 14,316, closing 58,484.
    let day = cashbook::get(
        &w.env.ctx,
        &CashbookInput {
            from: Some("10-10-2026".into()),
            to: Some("10-10-2026".into()),
            ..CashbookInput::default()
        },
    )
    .unwrap();
    assert_eq!(day.opening_paisa.rupees(), 50_000);
    assert_eq!(day.closing_paisa, t.cash_in_hand_paisa);
}

// ---- گاہکوں کے ذمے: the buyer dues page (FR-9.4) ---------------------------------------------------

#[test]
fn tile_buyers_dues_is_the_buyer_dues_page_total() {
    // 4.9: اکرم 2,880 + ارشد 18,684 = 21,564; شاہد (چکتا) paid all and is not on it.
    let w = full_day();
    let t = tiles(&w);
    let page = dashboard::buyer_dues(w.env.ctx.db(), &BuyerDuesInput::default()).unwrap();
    assert_eq!((page.total_count, page.total_paisa.rupees()), (2, 21_564));
    assert_eq!(page.total_paisa, t.buyers_dues_paisa);
}

// ---- بیوپاریوں کو ادائیگی باقی and بیوپاریوں کے ذمے: the seller page (FR-9.6) ------------------------

#[test]
fn tile_sellers_payable_is_the_seller_page_on_the_side_the_shop_owes() {
    // 4.9: کریم is owed 10,160; رحیم is paid and has no advance left.
    let w = full_day();
    let t = tiles(&w);
    let shop = seller_page(&w, SellerDuesSide::ShopOwes);
    assert_eq!(shop.total_count, 1);
    assert_eq!(shop.rows[0].party.name_en, "Karim");
    assert_eq!(shop.rows[0].balance.side, BalanceSide::Cr);
    assert_eq!(shop.shop_owes_paisa.rupees(), 10_160);
    assert_eq!(shop.shop_owes_paisa, t.sellers_payable_paisa);
    // سب shows the same figure at the top.
    assert_eq!(
        seller_page(&w, SellerDuesSide::All).shop_owes_paisa,
        t.sellers_payable_paisa
    );
}

#[test]
fn tile_sellers_dues_is_the_seller_page_on_the_side_the_seller_owes() {
    // FR-15.7: the shop pays کریم 12,000 against 10,160 payable. The 1,840 over is what he owes the
    // shop beyond his advances (not an advance).
    let w = full_day();
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.karim, "12,000")).unwrap();
    let t = tiles(&w);
    let owes = seller_page(&w, SellerDuesSide::SellerOwes);
    assert_eq!(owes.total_count, 1);
    assert_eq!(owes.rows[0].party.name_en, "Karim");
    assert_eq!(owes.rows[0].owes_paisa.rupees(), 1_840);
    assert_eq!(owes.rows[0].balance.side, BalanceSide::Dr);
    assert_eq!(owes.seller_owes_paisa.rupees(), 1_840);
    assert_eq!(owes.seller_owes_paisa, t.sellers_dues_paisa);
    // Nothing is payable any more, on any side.
    assert_eq!(owes.shop_owes_paisa.rupees(), 0);
    assert_eq!(owes.shop_owes_paisa, t.sellers_payable_paisa);
    assert_eq!(seller_page(&w, SellerDuesSide::ShopOwes).total_count, 0);
}

// ---- کل اگراہی: the total dues page (FR-9.7) ------------------------------------------------------

#[test]
fn tile_total_dues_is_the_grand_total_and_each_part_is_the_page_of_its_section() {
    // FR-9.7 example, after Events 1 to 5: buyers 21,564 (2), sellers 0, dead 0, others 0.
    let w = full_day();
    let t = tiles(&w);
    let total = dashboard::total_dues_parts(w.env.ctx.db()).unwrap();
    let part = |section: TotalDuesSection| {
        let p = total.parts.iter().find(|p| p.section == section).unwrap();
        (p.count, p.total_paisa)
    };
    assert_eq!(total.parts.len(), 4);
    assert_eq!(total.grand_total_paisa.rupees(), 21_564);
    assert_eq!(total.grand_total_paisa, t.total_dues_paisa);
    // Each section is the page that opens from it.
    let buyers = dashboard::buyer_dues(w.env.ctx.db(), &BuyerDuesInput::default()).unwrap();
    assert_eq!(part(TotalDuesSection::Buyers), (2, buyers.total_paisa));
    let sellers = seller_page(&w, SellerDuesSide::SellerOwes);
    assert_eq!(
        part(TotalDuesSection::Sellers),
        (sellers.total_count, sellers.seller_owes_paisa)
    );
    let dead_page = mandi_reports::dead::page(
        w.env.ctx.db(),
        &DeadAmountInput {
            filter: DeadFilter::Owed,
            search: String::new(),
            offset: 0,
            limit: 100,
        },
    )
    .unwrap();
    assert_eq!(
        part(TotalDuesSection::Dead),
        (dead_page.row_count, dead_page.totals.still_owed_paisa)
    );
    let others = dashboard::lent_open(w.env.ctx.db(), &LentOpenInput::default()).unwrap();
    assert_eq!(
        part(TotalDuesSection::Others),
        (others.total_count, others.total_paisa)
    );
    // The four parts add up to the grand total.
    let sum: i64 = total.parts.iter().map(|p| p.total_paisa.rupees()).sum();
    assert_eq!(sum, 21_564);
}

#[test]
fn tile_total_dues_follows_the_dead_amount_and_the_money_lent() {
    // E2E-44: ارشد dies on 11-10-26 owing 18,684; his son pays 4,000 on 12-10-26. E2E-30: اسلم is
    // lent 5,000. کل اگراہی = buyers + sellers + dead + money lent, on every page.
    let w = full_day();
    lend(&w, "5,000");
    assert_eq!(tiles(&w).total_dues_paisa.rupees(), 26_564);
    set_today(&w.env, 2026, 10, 11);
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.arshad,
            entry_date: "11-10-2026".into(),
            reason: DeadReason::Died,
            note: String::new(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 12);
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: "12-10-2026".into(),
            ..receipt(w.arshad, "4,000")
        },
    )
    .unwrap();
    let t = tiles(&w);
    // Buyers 2,880 + sellers 0 + dead 14,684 + lent 5,000.
    assert_eq!(t.buyers_dues_paisa.rupees(), 2_880);
    assert_eq!(t.dead_amount_paisa.rupees(), 14_684);
    assert_eq!(t.lent_open_paisa.rupees(), 5_000);
    assert_eq!(t.total_dues_paisa.rupees(), 22_564);
    let total = dashboard::total_dues_parts(w.env.ctx.db()).unwrap();
    assert_eq!(total.grand_total_paisa, t.total_dues_paisa);
}

// ---- ڈوبی رقم: R38 (FR-10.10) ----------------------------------------------------------------------

#[test]
fn tile_dead_amount_is_the_still_owed_total_of_r38() {
    // E2E-44: ارشد moved on 11-10-26 with 18,684; 4,000 recovered on 12-10-26, 14,684 still owed.
    let w = full_day();
    let owed = |w: &support::World| {
        mandi_reports::dead::page(
            w.env.ctx.db(),
            &DeadAmountInput {
                filter: DeadFilter::Owed,
                search: String::new(),
                offset: 0,
                limit: 100,
            },
        )
        .unwrap()
    };
    let t = tiles(&w);
    assert_eq!(
        (
            owed(&w).row_count,
            t.dead_amount_paisa.rupees(),
            t.dead_count
        ),
        (0, 0, 0)
    );
    set_today(&w.env, 2026, 10, 11);
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.arshad,
            entry_date: "11-10-2026".into(),
            reason: DeadReason::Died,
            note: String::new(),
        },
    )
    .unwrap();
    let (page, t) = (owed(&w), tiles(&w));
    assert_eq!(page.totals.still_owed_paisa.rupees(), 18_684);
    assert_eq!(page.totals.still_owed_paisa, t.dead_amount_paisa);
    assert_eq!((page.row_count, t.dead_count), (1, 1));
    set_today(&w.env, 2026, 10, 12);
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: "12-10-2026".into(),
            ..receipt(w.arshad, "4,000")
        },
    )
    .unwrap();
    let (page, t) = (owed(&w), tiles(&w));
    assert_eq!(page.totals.still_owed_paisa.rupees(), 14_684);
    assert_eq!(page.totals.still_owed_paisa, t.dead_amount_paisa);
}

// ---- پیشگی باقی: the advances page (FR-15.8) -------------------------------------------------------

#[test]
fn tile_advances_open_is_the_open_total_of_the_advances_page() {
    // The opening position: رحیم was given 15,000 on 01-09-26, nothing adjusted, 15,000 open.
    let w = worked_day();
    let t = tiles(&w);
    let page = advances(&w, true);
    assert_eq!(page.total_count, 1);
    let row = &page.rows[0];
    assert_eq!(
        (
            row.given_paisa.rupees(),
            row.adjusted_paisa.rupees(),
            row.returned_paisa.rupees(),
            row.open_paisa.rupees()
        ),
        (15_000, 0, 0, 15_000)
    );
    assert_eq!(page.open_paisa.rupees(), 15_000);
    assert_eq!(page.open_paisa, t.advances_open_paisa);
    // The page of the tile and the full report agree while an advance is open.
    assert_eq!(advances(&w, false).open_paisa, page.open_paisa);
}

#[test]
fn tile_advances_open_is_zero_after_event_one_adjusts_the_advance() {
    // E2E-7: given 15,000, adjusted 15,000 by بکری 1, open 0. The page says so.
    let w = full_day();
    let t = tiles(&w);
    let page = advances(&w, false);
    assert_eq!(page.given_paisa.rupees(), 15_000);
    assert_eq!(page.adjusted_paisa.rupees(), 15_000);
    assert_eq!(page.open_paisa.rupees(), 0);
    assert_eq!(page.open_paisa, t.advances_open_paisa);
    // The page of the tile lists nobody and says 0.
    let open = advances(&w, true);
    assert_eq!((open.total_count, open.open_paisa.rupees()), (0, 0));
    assert_eq!(open.open_paisa, t.advances_open_paisa);
    // The seller page shows the same advance total at its top.
    assert_eq!(
        seller_page(&w, SellerDuesSide::All).advances_open_paisa,
        t.advances_open_paisa
    );
}

// ---- ادھار رقم باقی: money lent (FR-16.5) ---------------------------------------------------------

#[test]
fn tile_money_lent_is_the_total_of_the_page_of_people_with_something_open() {
    // E2E-30: اسلم is lent 5,000 on 10-10-26, nothing returned.
    let w = full_day();
    let t = tiles(&w);
    let none = dashboard::lent_open(w.env.ctx.db(), &LentOpenInput::default()).unwrap();
    assert_eq!((none.total_count, none.total_paisa.rupees()), (0, 0));
    assert_eq!(none.total_paisa, t.lent_open_paisa);
    let person = lend(&w, "5,000");
    let t = tiles(&w);
    let page = dashboard::lent_open(w.env.ctx.db(), &LentOpenInput::default()).unwrap();
    assert_eq!((page.total_count, page.total_paisa.rupees()), (1, 5_000));
    assert_eq!(page.total_paisa, t.lent_open_paisa);
    assert_eq!(page.rows[0].party.account_id, person);
    assert_eq!(page.overdue_paisa, t.lent_overdue_paisa);
}

// ---- باغوں کے مالکوں / دوسروں کے ذمے: the page of FR-16.13 ----------------------------------------

#[test]
fn tile_others_dues_is_the_open_total_of_the_lent_dues_page() {
    // اسلم is a person of the type other, so his 5,000 is on the tile of the others and on its page.
    let w = full_day();
    lend(&w, "5,000");
    let t = tiles(&w);
    let page = lent::dues(&w.env.ctx, &LentDuesInput::default()).unwrap();
    assert_eq!(page.totals.open_paisa.rupees(), 5_000);
    assert_eq!(page.totals.open_paisa, t.others_dues_paisa);
    assert_eq!(page.totals.open_people_count, t.others_people_count);
    assert_eq!(page.totals.open_people_count, 1);
}

// ---- آج کا خرچہ: the kharch page, view M (FR-9.5) -------------------------------------------------

#[test]
fn tile_expenses_today_is_the_total_of_the_shop_part_of_view_m() {
    // E2E-39: بجلی 3,000 as دکان خرچہ; the 2,000 of الف is personal and is not on it.
    let w = full_day();
    let t = tiles(&w);
    let page = mandi_reports::analytics::run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: AnalyticsView::M,
            filters: AnalyticsFilters {
                from: "10-10-2026".into(),
                to: "10-10-2026".into(),
                ..AnalyticsFilters::default()
            },
            offset: 0,
            limit: 0,
            sort: None,
        },
    )
    .unwrap();
    assert_eq!(page.total_count, 1);
    assert_eq!(last_total(&page.totals), 3_000);
    assert_eq!(last_total(&page.totals), t.expenses_today_paisa.rupees());
}

// ---- باقی مال: unsold stock, lot by lot (FR-12.14) ------------------------------------------------

#[test]
fn tile_stock_is_the_lot_count_and_the_balance_of_the_stock_list() {
    // E2E-16: 45 of رحیم's 50 کیلا sold, so 1 lot with 5 نگ is left.
    let w = worked_day();
    let none = tiles(&w);
    assert_eq!((none.stock_lots, none.stock_qty), (0, 0));
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    save(&w, &draft);
    let t = tiles(&w);
    let list = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    let lots = i64::try_from(list.rows.len() + list.garden_rows.len()).unwrap();
    assert_eq!(
        (lots, list.balance_total + list.garden_balance_total),
        (1, 5)
    );
    assert_eq!((t.stock_lots, t.stock_qty), (lots, 5));
    assert_eq!(list.balance_total + list.garden_balance_total, t.stock_qty);
}

// ---- مارکیٹ سفر: open trips (FR-12.25) ---------------------------------------------------------------

#[test]
fn tile_trips_is_the_trips_summary() {
    // E2E-34: 20 پیٹی مالٹا bought, 15 sold: 5 نگ left, cost left 2,600 (520 each).
    let (w, _trip, _purchase, _bikri) = trip_34_world();
    let t = tiles(&w);
    let page = trips::summary(&w.env.ctx).unwrap();
    assert_eq!(
        (
            page.open_trips,
            page.purchases_with_stock,
            page.qty_left,
            page.cost_left_paisa.rupees()
        ),
        (1, 1, 5, 2_600)
    );
    assert_eq!(page.open_trips, t.trips_open);
    assert_eq!(page.purchases_with_stock, t.trips_purchases_with_stock);
    assert_eq!(page.qty_left, t.trips_qty_left);
    assert_eq!(page.cost_left_paisa, t.trips_cost_left_paisa);
}

// ---- باغات: open gardens (FR-12.37) ------------------------------------------------------------------

#[test]
fn tile_gardens_is_the_gardens_summary() {
    // PRD 12.11: rent 50,000 with 30,000 paid: the running balance is -30,000.
    let w = worked_day();
    let none = tiles(&w);
    assert_eq!(none.gardens_open, 0);
    garden_made(&w);
    let t = tiles(&w);
    let page = gardens::summary(&w.env.ctx).unwrap();
    assert_eq!(
        (page.open_gardens, page.balance_total_paisa.rupees()),
        (1, -30_000)
    );
    assert_eq!(page.open_gardens, t.gardens_open);
    assert_eq!(page.balance_total_paisa, t.gardens_balance_paisa);
    assert_eq!(i64::try_from(page.rows.len()).unwrap(), t.gardens_open);
}

// ---- آج کی نقدی: the cash sales page (FR-9.8) ------------------------------------------------------

#[test]
fn tile_cash_sales_is_the_day_the_month_and_the_buyers_of_the_page() {
    // FR-9.8 example: the tile 12,800 in 1 buyer; the month row for 10-10-26 1 buyer, 12,800; its day:
    // شاہد (چکتا), بکری 2, bill 12,800, نقدی 12,800.
    let w = full_day();
    let t = tiles(&w);
    assert_eq!(
        (t.cash_sales_today_paisa.rupees(), t.cash_sales_today_buyers),
        (12_800, 1)
    );
    let month =
        dashboard::cash_sales_month(w.env.ctx.db(), &CashSalesMonthInput::default()).unwrap();
    assert_eq!((month.rows.len(), month.total_paisa.rupees()), (1, 12_800));
    assert_eq!(month.rows[0].cash_paisa, t.cash_sales_today_paisa);
    assert_eq!(month.rows[0].buyer_count, t.cash_sales_today_buyers);
    let day = dashboard::cash_sales_day(
        w.env.ctx.db(),
        &CashSalesDayInput {
            date: "10-10-2026".into(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(day.total_count, t.cash_sales_today_buyers);
    assert_eq!(day.cash_total_paisa, t.cash_sales_today_paisa);
    assert_eq!(day.rows[0].bikri_number, 2);
    assert!(day.rows[0].is_chukta);
    assert_eq!(
        (
            day.rows[0].bill_paisa.rupees(),
            day.rows[0].cash_paisa.rupees()
        ),
        (12_800, 12_800)
    );
}

/// Edits one بکری (previewed first, as the screen does) and gives the money paid on the spot the
/// tile, the month row and the day page show, which must be one figure.
fn edit_and_read_cash(
    w: &support::World,
    bikri_id: i64,
    entry_id: i64,
    change: impl Fn(&mut BikriDraft),
) -> (i64, i64, i64) {
    let opened = bikri::open_for_edit(&w.env.ctx, &EditOpenInput { bikri_id, entry_id }).unwrap();
    let mut draft = opened.draft.clone();
    change(&mut draft);
    let preview = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id,
            draft: draft.clone(),
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let edited = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id,
            version: opened.version,
            draft,
            reason: Some("موقع پر جمع درست کیا".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let t = tiles(w);
    let month =
        dashboard::cash_sales_month(w.env.ctx.db(), &CashSalesMonthInput::default()).unwrap();
    let day = dashboard::cash_sales_day(
        w.env.ctx.db(),
        &CashSalesDayInput {
            date: "10-10-2026".into(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(month.total_paisa, t.cash_sales_today_paisa);
    assert_eq!(day.cash_total_paisa, t.cash_sales_today_paisa);
    assert_eq!(day.total_count, t.cash_sales_today_buyers);
    (
        t.cash_sales_today_paisa.rupees(),
        t.cash_sales_today_buyers,
        edited.entry_id,
    )
}

#[test]
fn tile_cash_sales_follows_an_edit_of_the_paid_amount_on_every_page() {
    // PRD FR-12.44 and FR-7.70: an edit reverses the old lines and posts new ones, and only the
    // lines in force count. اکرم pays 5,000 of his 12,880 on the spot (an edit), then takes it back
    // to 0. شاہد (چکتا) pays his whole 12,800 throughout.
    let w = worked_day();
    let one = save(&w, &support::event1(&w));
    save(&w, &event2(&w));
    assert_eq!(tiles(&w).cash_sales_today_paisa.rupees(), 12_800);
    let set_paid = |paid: &'static str| {
        move |draft: &mut BikriDraft| draft.lots[0].sales[0].paid = Some(paid.into())
    };
    let (cash, buyers, entry_id) =
        edit_and_read_cash(&w, one.bikri_id, one.entry_id, set_paid("5,000"));
    assert_eq!((cash, buyers), (17_800, 2));
    // The day page has his row with his bill and what he paid.
    let day = dashboard::cash_sales_day(
        w.env.ctx.db(),
        &CashSalesDayInput {
            date: "10-10-2026".into(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(day.bill_total_paisa.rupees(), 12_880 + 12_800);
    let akram = day.rows.iter().find(|r| !r.is_chukta).unwrap();
    assert_eq!(
        (
            akram.bikri_number,
            akram.bill_paisa.rupees(),
            akram.cash_paisa.rupees()
        ),
        (1, 12_880, 5_000)
    );
    let (cash, buyers, _) = edit_and_read_cash(&w, one.bikri_id, entry_id, set_paid("0"));
    assert_eq!((cash, buyers), (12_800, 1));
}
