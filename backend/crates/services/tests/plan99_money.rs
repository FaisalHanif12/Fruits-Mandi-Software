//! Plan 99 (PRD v1.60) money: party openings through روکڑ and the one-time fix (FR-10.9), the
//! simple روکڑ بک (FR-16.14), where a garden's sale money is (FR-12.53) and the partner's open
//! gardens and trips by page (FR-17.3). The expected numbers are the money-calculations agent's
//! worked numbers for plan 99.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::core::accounts::{
    AccountType, BalanceSide, OpeningLineInput, OpeningRequest, OpeningsInput, OpeningsToCashInput,
};
use mandi_services::core::cashbook::{CashKind, CashbookInput};
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::garden_sale_money::{GardensSaleMoneyInput, SaleMoneyKind};
use mandi_services::core::gardens::GardenGetInput;
use mandi_services::core::money::Paisa;
use mandi_services::core::partners::PartnersOpenVenturesInput;
use mandi_services::db::rusqlite::params;
use mandi_services::testing::opening_before_v160;
use mandi_services::{ServiceError, accounts, cashbook, gardens, ledger, partners, vouchers};
use reports_world::*;
use support::World;

fn grid(w: &World, id: i64, dr: &str, cr: &str, advance: bool) -> OpeningsInput {
    OpeningsInput {
        lines: vec![OpeningLineInput {
            account_id: id,
            version: accounts::get(&w.env.ctx, id).unwrap().version,
            opening_date: "10-10-2026".into(),
            opening_dr: dr.into(),
            opening_cr: cr.into(),
            opening_is_advance: advance,
        }],
    }
}

/// An account's balance in rupees, debit positive.
fn net(w: &World, code: &str) -> i64 {
    let id = support::id_of(&w.env, code);
    let balance = accounts::get(&w.env.ctx, id).unwrap().balance;
    match balance.side {
        BalanceSide::Dr => balance.amount_paisa.rupees(),
        BalanceSide::Cr => -balance.amount_paisa.rupees(),
        BalanceSide::Zero => 0,
    }
}

fn first_code(error: ServiceError) -> (ErrorCode, Option<String>, Option<ErrorValue>) {
    match error {
        ServiceError::Rejected(items) => (
            items[0].code,
            items[0].field.clone(),
            items[0].values.get("cash_after_paisa").cloned(),
        ),
        other => panic!("not a rejection: {other:?}"),
    }
}

#[test]
fn a_partys_opening_goes_through_the_cash_book_and_a_change_posts_it_again() {
    // Cash 50,000. رحیم is given 15,000 before the books (بنام), اکرم left 2,000 (جمع).
    let w = support::world();
    let difference = net(&w, "06019999");
    accounts::openings_save(&w.env.ctx, &w.owner, &grid(&w, w.rahim, "15,000", "", true)).unwrap();
    accounts::openings_save(&w.env.ctx, &w.owner, &grid(&w, w.akram, "", "2,000", false)).unwrap();
    assert_eq!(net(&w, "01010001"), 50_000 - 15_000 + 2_000);
    assert_eq!(
        net(&w, "06019999"),
        difference,
        "ابتدائی فرق is not touched"
    );
    assert_eq!((net(&w, "0010"), net(&w, "0020")), (15_000, -2_000));

    // A change reverses the old opening and posts the new one, again through cash.
    accounts::openings_save(&w.env.ctx, &w.owner, &grid(&w, w.rahim, "20,000", "", true)).unwrap();
    assert_eq!(net(&w, "01010001"), 32_000);

    // More than the cash in hand asks first: 32,000 − 40,000 more = −8,000.
    let more = grid(&w, w.rahim, "60,000", "", true);
    let refused = accounts::openings_save(&w.env.ctx, &w.owner, &more).unwrap_err();
    let (code, field, after) = first_code(refused);
    assert_eq!(
        (code, field.as_deref(), after),
        (
            ErrorCode::ErrCashBelowZeroNotConfirmed,
            Some("lines[0].opening_dr"),
            Some(ErrorValue::Number(-800_000))
        )
    );
    assert_eq!(net(&w, "01010001"), 32_000, "nothing written");
    accounts::openings_save_with_confirm(&w.env.ctx, &w.owner, &more, true).unwrap();
    assert_eq!(net(&w, "01010001"), -8_000);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

/// An opening saved as before v1.60, against ابتدائی فرق.
fn old_opening(w: &World, id: i64, dr: i64, cr: i64, advance: bool) {
    opening_before_v160(
        &w.env.ctx,
        w.owner.user_id,
        id,
        &OpeningRequest {
            date: BusinessDate::parse_iso("2026-10-10"),
            dr: Paisa::from_rupees(dr).unwrap(),
            cr: Paisa::from_rupees(cr).unwrap(),
            is_advance: advance,
        },
    )
    .unwrap();
}

fn fix_input(sign: &str, confirmed: bool) -> OpeningsToCashInput {
    OpeningsToCashInput {
        reason: "openings through the cash book".into(),
        sign: sign.into(),
        confirm_cash_below_zero: confirmed,
    }
}

fn close_up_to(w: &World, iso: &str) {
    support_raw(w)
        .execute(
            "UPDATE day_state SET closed_up_to_date = ?1 WHERE id = 1",
            params![iso],
        )
        .unwrap();
}

fn support_raw(w: &World) -> mandi_services::db::rusqlite::Connection {
    common::raw(&w.dir)
}

#[test]
fn the_one_time_fix_moves_old_openings_into_the_cash_book_once() {
    // Old openings: رحیم 15,000 بنام (an advance) and اکرم 10,000 جمع, both against ابتدائی فرق.
    let w = support::world();
    old_opening(&w, w.rahim, 15_000, 0, true);
    old_opening(&w, w.akram, 0, 10_000, false);
    close_up_to(&w, "2026-10-10");
    let cash = net(&w, "01010001");
    assert_eq!((cash, net(&w, "06019999")), (50_000, -5_000));
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap().total_dr;
    let totals: Vec<i64> = partners::overview(&w.env.ctx)
        .unwrap()
        .iter()
        .map(|r| r.total_paisa.rupees())
        .collect();

    let preview = accounts::openings_to_cash_preview(&w.env.ctx).unwrap();
    assert_eq!(preview.rows.len(), 2);
    assert_eq!(
        (
            preview.total_dr_paisa.rupees(),
            preview.total_cr_paisa.rupees(),
            preview.cash_after_paisa.rupees(),
            preview.needs_cash_confirm,
            preview.opening_difference_before.amount_paisa.rupees(),
            preview.opening_difference_after.amount_paisa.rupees(),
        ),
        (15_000, 10_000, 45_000, false, 5_000, 0)
    );
    assert_eq!(preview.reopens_from.as_deref(), Some("2026-10-10"));

    let wrong = accounts::openings_to_cash(&w.env.ctx, &w.owner, &fix_input("munshi", false));
    assert_eq!(first_code(wrong.unwrap_err()).0, ErrorCode::ErrSignWrong);
    assert_eq!(net(&w, "01010001"), 50_000);

    let done =
        accounts::openings_to_cash(&w.env.ctx, &w.owner, &fix_input("adnan", false)).unwrap();
    assert_eq!(done.cash_after_paisa.rupees(), 45_000);
    assert_eq!(done.closed_up_to.as_deref(), Some("2026-10-10"));
    assert_eq!(done.reopened_from.as_deref(), Some("2026-10-10"));
    assert_eq!((net(&w, "01010001"), net(&w, "06019999")), (45_000, 0));
    assert_eq!((net(&w, "0010"), net(&w, "0020")), (15_000, -10_000));
    // رحیم's advance is still open, and no partner total moved.
    assert_eq!(
        mandi_services::assist::advances(&w.env.ctx, w.rahim)
            .unwrap()
            .open_paisa
            .rupees(),
        15_000
    );
    assert_eq!(
        partners::overview(&w.env.ctx)
            .unwrap()
            .iter()
            .map(|r| r.total_paisa.rupees())
            .collect::<Vec<_>>(),
        totals
    );
    let after = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(after.total_dr, after.total_cr);
    // ابتدائی فرق left the trial balance (5,000 on the credit side) and cash is 5,000 less.
    assert_eq!(after.total_dr.rupees(), trial.rupees() - 5_000);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();

    let again = accounts::openings_to_cash(&w.env.ctx, &w.owner, &fix_input("adnan", false));
    assert_eq!(
        first_code(again.unwrap_err()).0,
        ErrorCode::ErrOpeningsAlreadyInCash
    );
}

#[test]
fn the_one_time_fix_asks_once_when_the_cash_would_go_below_zero() {
    let w = support::world_with(0, true);
    old_opening(&w, w.rahim, 15_000, 0, true);
    let refused = accounts::openings_to_cash(&w.env.ctx, &w.owner, &fix_input("adnan", false));
    let (code, _, after) = first_code(refused.unwrap_err());
    assert_eq!(
        (code, after),
        (
            ErrorCode::ErrCashBelowZeroNotConfirmed,
            Some(ErrorValue::Number(-1_500_000))
        )
    );
    assert_eq!(net(&w, "01010001"), 0);
    accounts::openings_to_cash(&w.env.ctx, &w.owner, &fix_input("adnan", true)).unwrap();
    assert_eq!(net(&w, "01010001"), -15_000);
}

fn book(kinds: Option<Vec<CashKind>>) -> CashbookInput {
    CashbookInput {
        from: Some("10-10-2026".into()),
        to: Some("10-10-2026".into()),
        offset: 0,
        limit: 0,
        sort: None,
        group_by_month: false,
        show_reversed: false,
        kinds,
    }
}

#[test]
fn the_cash_book_of_the_worked_day_names_every_row_and_gives_the_three_cards() {
    // PRD FR-16.14 v1.60: 10-10-26 opens at 50,000 and closes at 58,484; capital 65,000.
    let w = full_day();
    let answer = cashbook::get(&w.env.ctx, &book(None)).unwrap();
    assert_eq!(
        (
            answer.total_capital_paisa.rupees(),
            answer.opening_paisa.rupees(),
            answer.closing_paisa.rupees(),
            answer.totals.in_paisa.rupees(),
            answer.totals.out_paisa.rupees(),
        ),
        (65_000, 50_000, 58_484, 22_800, 14_316)
    );
    let rows: Vec<(CashKind, i64, i64)> = answer
        .rows
        .iter()
        .map(|r| {
            (
                r.kind,
                r.in_paisa.rupees() - r.out_paisa.rupees(),
                r.running_paisa.unwrap().rupees(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            (CashKind::BikriSellerCash, -5_000, 45_000),
            (CashKind::BikriPaid, 12_800, 57_800),
            (CashKind::Receipt, 10_000, 67_800),
            (CashKind::Payment, -4_316, 63_484),
            (CashKind::ShopExpense, -3_000, 60_484),
            (CashKind::PersonalExpense, -2_000, 58_484),
        ]
    );
    let party = |i: usize| answer.rows[i].refs.parties[0].name_en.clone();
    assert_eq!((party(0), party(1)), ("Rahim".into(), "Shahid".into()));
    assert_eq!(
        (party(2), party(3)),
        ("Akram Toll Tax".into(), "Rahim".into())
    );

    // The type filter narrows the rows (no running balance), not the cards or the summary.
    let only = cashbook::get(&w.env.ctx, &book(Some(vec![CashKind::Receipt]))).unwrap();
    assert_eq!(only.rows.len(), 1);
    assert_eq!(only.rows[0].running_paisa, None);
    assert_eq!(only.closing_paisa.rupees(), 58_484);
    assert_eq!(only.summary, answer.summary);
    // The summary adds up to the totals.
    let (sum_in, sum_out) = answer.summary.iter().fold((0, 0), |(i, o), g| {
        (i + g.in_paisa.rupees(), o + g.out_paisa.rupees())
    });
    assert_eq!((sum_in, sum_out), (22_800, 14_316));
}

#[test]
fn a_gardens_sale_money_follows_each_bill_and_adds_up_to_its_money_in() {
    // 30 پیٹی of the garden: مانی 20 @ 1,000 on account, انس 10 @ 1,000 with 5,000 paid; مزدوری
    // 150, منشیانہ 50, فون 20. Next day 15,000 from مانی and 15,000 from another shop.
    let w = support::world();
    let garden = garden_made(&w);
    let account = gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id: garden.id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap()
    .row
    .account
    .id;
    let buyer = |ur: &str, en: &str| {
        support::add_account(&w.env, &w.owner, AccountType::Buyer, ur, en, true, "")
    };
    let (mani, anas) = (buyer("مانی", "Mani"), buyer("انس", "Anas"));
    let mut lot = support::bare_lot(&w, "g-1", account, w.banana, "30");
    lot.labour = support::manual("150");
    lot.munshiana = support::manual("50");
    lot.phone = support::manual("20");
    lot.adjust_advance = false;
    let mut lines = vec![
        support::sale("s-1", mani, "20", "1000"),
        support::sale("s-2", anas, "10", "1000"),
    ];
    lines[1].paid = Some("5,000".into());
    lot.sales = lines;
    save(&w, &support::draft(vec![lot]));
    common::set_today(&w.env, 2026, 10, 11);
    let mut paid = receipt(mani, "15,000");
    paid.date = "11-10-2026".into();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &paid).unwrap();
    garden_other_sale(&w, garden.id, "11-10-2026", "10", "15,000");

    let input = |search: Option<&str>, limit: u32| GardensSaleMoneyInput {
        garden_id: garden.id,
        from: None,
        to: None,
        arrival_id: None,
        search: search.map(str::to_owned),
        offset: 0,
        limit,
    };
    let money = gardens::sale_money(&w.env.ctx, &input(None, 0)).unwrap();
    assert_eq!(money.total_count, 4);
    let rows: Vec<(SaleMoneyKind, Option<i64>, Option<i64>, i64)> = money
        .rows
        .iter()
        .map(|r| {
            (
                r.kind,
                r.buyer_total_paisa.map(Paisa::rupees),
                r.still_due_paisa.map(Paisa::rupees),
                r.garden_credit_paisa.rupees(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            (SaleMoneyKind::Bikri, Some(21_360), Some(6_360), 18_800),
            (SaleMoneyKind::Bikri, Some(10_680), Some(5_680), 9_400),
            (SaleMoneyKind::LotCharges, None, None, -220),
            (SaleMoneyKind::OtherShop, None, None, 15_000),
        ]
    );
    let t = money.totals;
    assert_eq!(
        (
            t.buyer_total_paisa.rupees(),
            t.paid_paisa.rupees(),
            t.collected_paisa.rupees(),
            t.still_due_paisa.rupees(),
            t.lot_charges_paisa.rupees(),
            t.garden_credit_paisa.rupees(),
        ),
        (32_040, 5_000, 15_000, 12_040, 220, 42_980)
    );
    let row = gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id: garden.id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap()
    .row;
    assert_eq!(row.money_in_paisa, t.garden_credit_paisa);

    // A page keeps the totals of the whole filter; a search narrows both.
    let page = gardens::sale_money(&w.env.ctx, &input(None, 2)).unwrap();
    assert_eq!((page.rows.len(), page.total_count), (2, 4));
    assert_eq!(page.totals, t);
    let mani_only = gardens::sale_money(&w.env.ctx, &input(Some("مانی"), 0)).unwrap();
    assert_eq!(mani_only.total_count, 1);
    assert_eq!(mani_only.totals.buyer_total_paisa.rupees(), 21_360);

    // The partner's open gardens by page, with the totals over the whole filter.
    let ventures = |from: Option<&str>| {
        partners::open_ventures(
            &w.env.ctx,
            &PartnersOpenVenturesInput {
                partner_id: w.partners[0],
                from: from.map(str::to_owned),
                ..PartnersOpenVenturesInput::default()
            },
        )
        .unwrap()
    };
    let all = ventures(None);
    assert_eq!((all.rows.len(), all.total_count), (1, 1));
    assert_eq!(all.totals.paid_shop_paisa, all.rows[0].paid_shop_paisa);
    assert_eq!(all.totals.gardens_balance_paisa, all.rows[0].balance_paisa);
    assert_eq!(all.totals.trips_result_paisa, Paisa::ZERO);
    assert_eq!(ventures(Some("11-10-2026")).total_count, 0);
}
