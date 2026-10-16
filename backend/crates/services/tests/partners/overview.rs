//! The partner overview (PRD 17.3, C12): today, month and year income share, month expense
//! share, profit, money taken and the capital balance.

use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
use mandi_services::core::partners::PartnerUpdateInput;
use mandi_services::domain::ledger::PostingType;
use mandi_services::{accounts, partners};

use crate::common::set_today;
use crate::support::*;

#[test]
fn the_overview_has_the_numbers_of_the_worked_day_and_keeps_its_windows_apart() {
    let (_dir, env, owner) = ready();
    let income = id_of(&env, "04010001");
    let cash = id_of(&env, "01010001");

    // 05-01-26: the partners, ب with an opening capital of 50,000, 40 / 30 / 30, and 1,000 income.
    set_today(&env, 2026, 1, 5);
    let a = new_partner(&env, &owner, "الف", "Partner A", None).unwrap();
    let b = new_partner(&env, &owner, "ب", "Partner B", Some("50,000")).unwrap();
    let c = new_partner(&env, &owner, "ج", "Partner C", None).unwrap();
    set_shares(
        &env,
        &owner,
        "05-01-2026",
        &[(a.id, "40"), (b.id, "30"), (c.id, "30")],
    )
    .unwrap();
    against_cash(&env, &owner, "2026-01-05", income, Cr, 1_000).unwrap();

    // 15-09-26 (entered on 20-09): 2,000 income, 3,000 shop expense and 1,000 taken by الف.
    set_today(&env, 2026, 9, 20);
    let head = expense_head(&env, &owner);
    against_cash(&env, &owner, "2026-09-15", income, Cr, 2_000).unwrap();
    against_cash(&env, &owner, "2026-09-15", head, Dr, 3_000).unwrap();
    against_cash(&env, &owner, "2026-09-15", a.drawings_account_id, Dr, 1_000).unwrap();

    // October: the worked day of PRD 4.9, with the first income on 03-10 so that "today" and
    // "this month" differ.
    set_today(&env, 2026, 10, 10);
    post(
        &env,
        &owner,
        "2026-10-03",
        PostingType::General,
        vec![
            line(cash, Dr, 3_948),
            line(id_of(&env, "04010001"), Cr, 1_764),
            line(id_of(&env, "04010002"), Cr, 1_764),
            line(id_of(&env, "04010003"), Cr, 400),
            line(id_of(&env, "04010004"), Cr, 20),
        ],
    )
    .unwrap();
    post(
        &env,
        &owner,
        "2026-10-10",
        PostingType::General,
        vec![
            line(cash, Dr, 1_540),
            line(id_of(&env, "04010001"), Cr, 720),
            line(id_of(&env, "04010002"), Cr, 720),
            line(id_of(&env, "04010003"), Cr, 80),
            line(id_of(&env, "04010004"), Cr, 20),
        ],
    )
    .unwrap();
    against_cash(&env, &owner, "2026-10-10", head, Dr, 3_000).unwrap();
    against_cash(&env, &owner, "2026-10-10", a.drawings_account_id, Dr, 2_000).unwrap();

    let rows = partners::overview(&env.ctx).unwrap();
    let numbers = |i: usize| -> [i64; 8] {
        let r = &rows[i];
        [
            r.share_bps.into(),
            r.today_income_paisa.rupees(),
            r.month_income_paisa.rupees(),
            r.year_income_paisa.rupees(),
            r.month_expense_paisa.rupees(),
            r.month_profit_paisa.rupees(),
            r.month_taken_paisa.rupees(),
            r.capital_balance_paisa.rupees(),
        ]
    };
    assert_eq!(
        rows.iter().map(|r| r.partner_id).collect::<Vec<_>>(),
        vec![a.id, b.id, c.id]
    );
    // share bps, today, month, year, month expense, month profit, month taken, capital.
    // الف: today 616, month 1,580 + 616, year adds 400 and 800; expense 1,200; profit 996.
    assert_eq!(numbers(0), [4_000, 616, 2_196, 3_396, 1_200, 996, 2_000, 0]);
    assert_eq!(numbers(1), [3_000, 462, 1_646, 2_546, 900, 746, 0, 50_000]);
    assert_eq!(numbers(2), [3_000, 462, 1_646, 2_546, 900, 746, 0, 0]);
    // Income minus expense of the month for all three is the firm's profit: 5,488 - 3,000.
    let profit: i64 = rows.iter().map(|r| r.month_profit_paisa.rupees()).sum();
    assert_eq!(profit, 2_488);

    // An inactive partner with nothing to show is left out; one with numbers stays.
    let d = new_partner(&env, &owner, "د", "Partner D", None).unwrap();
    partners::update(
        &env.ctx,
        &owner,
        &PartnerUpdateInput {
            id: d.id,
            version: d.version,
            name_ur: d.name_ur.clone(),
            name_en: d.name_en.clone(),
            is_active: false,
            confirmed_rules_words: Vec::new(),
        },
    )
    .unwrap();
    assert_eq!(partners::overview(&env.ctx).unwrap().len(), 3);
    assert_eq!(partners::list(&env.ctx).unwrap().len(), 4);
}

#[test]
fn an_overview_before_any_share_is_set_lists_the_partners_with_zeros() {
    let (_dir, env, owner) = ready();
    new_partner(&env, &owner, "الف", "Partner A", None).unwrap();
    let rows = partners::overview(&env.ctx).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].share_bps, 0);
    assert_eq!(rows[0].month_profit_paisa.rupees(), 0);
}

#[test]
fn money_taken_is_net_of_a_take_that_was_cancelled() {
    // FR-15.3, E56: a ذاتی خرچہ cancelled or edited is reversed. The reversal puts a credit on
    // the drawings account; "taken" must not stay at the old figure. A return is not taken off
    // here (the returned column comes in plan 17), and cancelling a return is not a new take.
    let (_dir, env, owner) = ready();
    let [a, ..] = three_partners(&env, &owner);
    let drawings = partners::get(&env.ctx, a).unwrap().row.drawings_account_id;
    let taken = |env: &mandi_services::testing::TestEnv| {
        partners::overview(&env.ctx).unwrap()[0]
            .month_taken_paisa
            .rupees()
    };
    let first_take = against_cash(&env, &owner, "2026-10-10", drawings, Dr, 2_000).unwrap();
    against_cash(&env, &owner, "2026-10-10", drawings, Dr, 500).unwrap();
    assert_eq!(taken(&env), 2_500);
    reverse(&env, &owner, first_take);
    assert_eq!(taken(&env), 500, "the cancelled 2,000 is gone");
    let giving_back = against_cash(&env, &owner, "2026-10-10", drawings, Cr, 300).unwrap();
    assert_eq!(taken(&env), 500, "a return does not lower taken");
    reverse(&env, &owner, giving_back);
    assert_eq!(taken(&env), 500, "cancelling a return is not a new take");
}

#[test]
fn an_edit_of_a_take_is_the_new_amount_not_both() {
    // An edit is a reversal of the old posting plus a new posting (PRD FR-12.19, FR-15.3).
    let (_dir, env, owner) = ready();
    let [a, ..] = three_partners(&env, &owner);
    let drawings = partners::get(&env.ctx, a).unwrap().row.drawings_account_id;
    let old = against_cash(&env, &owner, "2026-10-10", drawings, Dr, 2_000).unwrap();
    reverse(&env, &owner, old);
    against_cash(&env, &owner, "2026-10-10", drawings, Dr, 1_500).unwrap();
    let rows = partners::overview(&env.ctx).unwrap();
    assert_eq!(rows[0].month_taken_paisa.rupees(), 1_500);
}

#[test]
fn an_opening_debit_on_a_drawings_account_is_money_taken_in_its_opening_month() {
    // PRD FR-10.9 (third bullet): the app starts mid-year and a partner has already taken money.
    // The opening debit counts as taken on the opening date. Changing the opening reverses the
    // old posting, so the old value stops counting (the reversal has the old opening's date).
    let (_dir, env, owner) = ready();
    let [a, ..] = three_partners(&env, &owner);
    let drawings = partners::get(&env.ctx, a).unwrap().row.drawings_account_id;
    let set_opening = |dr: &str, on: &str| {
        let version = accounts::get(&env.ctx, drawings).unwrap().version;
        accounts::openings_save(
            &env.ctx,
            &owner,
            &OpeningsInput {
                lines: vec![OpeningLineInput {
                    account_id: drawings,
                    version,
                    opening_date: on.to_owned(),
                    opening_dr: dr.to_owned(),
                    opening_cr: String::new(),
                    opening_is_advance: false,
                }],
            },
        )
        .unwrap();
    };
    let taken = || {
        partners::overview(&env.ctx).unwrap()[0]
            .month_taken_paisa
            .rupees()
    };
    assert_eq!(taken(), 0);
    // 2,000 taken before the app started, opening dated in October (this month).
    set_opening("2,000", "05-10-2026");
    assert_eq!(taken(), 2_000);
    // The opening is changed to 700 on the same day: the old 2,000 stops counting.
    set_opening("700", "05-10-2026");
    assert_eq!(taken(), 700);
    // Moved to September: it is no longer money taken this month, and the old posting is gone.
    set_opening("700", "15-09-2026");
    assert_eq!(taken(), 0);
    // And back: counted once, not twice.
    set_opening("700", "06-10-2026");
    assert_eq!(taken(), 700);
}
