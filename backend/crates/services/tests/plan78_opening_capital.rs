//! Where a partner's opening capital is (plan 78 E, PRD 17.1 v1.43, FR-10.9, FR-6.7, FR-16.14).
//!
//! The world is the worked day's opening position: cash 50,000 on 10-10-26. A partner D is made
//! with an opening capital of 50,000:
//! * in cash (the default): روکڑ بنام 50,000, his capital account جمع 50,000, cash 100,000;
//! * already counted elsewhere: ابتدائی فرق بنام 50,000, his capital account جمع 50,000, cash 50,000.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::core::accounts::{BalanceSide, OpeningLineInput, OpeningsInput};
use mandi_services::core::cashbook::{CashKind, CashbookInput};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::partners::{OpeningCapitalWhere, PartnerCreateInput, PartnerRow};
use mandi_services::{ServiceError, accounts, cashbook, ledger, partners};
use support::World;

fn make(w: &World, opening: &str, wh: Option<OpeningCapitalWhere>) -> PartnerRow {
    let mut words: Vec<String> = Vec::new();
    loop {
        let input = PartnerCreateInput {
            name_ur: "داؤد".into(),
            name_en: "Dawood".into(),
            opening_capital: Some(opening.into()),
            opening_capital_where: wh,
            confirmed_rules_words: words.clone(),
        };
        match partners::create(&w.env.ctx, &w.owner, &input) {
            Ok(row) => return row,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(found)) = item.values.get("words") {
                        words.extend(found.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

fn cash(w: &World) -> i64 {
    cashbook::get(&w.env.ctx, &CashbookInput::default())
        .unwrap()
        .closing_paisa
        .rupees()
}

fn balance(w: &World, account: i64) -> (BalanceSide, i64) {
    let b = ledger::account_balance(&w.env.ctx, account).unwrap();
    (b.side, b.amount_paisa.rupees())
}

/// An advance to رحیم that takes cash below zero (confirmed).
fn advance(w: &World, amount: &str) {
    mandi_services::vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::AdvanceInput {
            confirm_cash_below_zero: true,
            date: "10-10-2026".into(),
            seller_account_id: w.rahim,
            amount: amount.into(),
            removed: Default::default(),
            expected_goods: String::new(),
            narration: String::new(),
            reason: None,
        },
    )
    .unwrap();
}

fn change_opening(
    w: &World,
    capital: i64,
    amount: &str,
) -> Result<mandi_services::core::accounts::OpeningsSaved, ServiceError> {
    let version = accounts::get(&w.env.ctx, capital).unwrap().version;
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: capital,
                version,
                opening_date: "10-10-2026".into(),
                opening_dr: String::new(),
                opening_cr: amount.into(),
                opening_is_advance: false,
            }],
        },
    )
}

#[test]
fn an_opening_capital_in_cash_is_the_default_and_raises_cash() {
    let w = support::world_with(50_000, false);
    assert_eq!(cash(&w), 50_000);
    let d = make(&w, "50,000", None);
    assert_eq!(cash(&w), 100_000);
    assert_eq!(balance(&w, d.capital_account_id), (BalanceSide::Cr, 50_000));
    let difference = support::id_of(&w.env, "06019999");
    assert_eq!(
        balance(&w, difference).1,
        0,
        "nothing goes against ابتدائی فرق"
    );
    // The cash book names the row and the partner.
    let book = cashbook::get(&w.env.ctx, &CashbookInput::default()).unwrap();
    let row = book
        .rows
        .iter()
        .find(|r| r.kind == CashKind::OpeningCapital)
        .expect("an opening capital row");
    assert_eq!(row.in_paisa.rupees(), 50_000);
    assert_eq!(row.refs.partner.as_ref().map(|p| p.id), Some(d.id));
    // The trial balance still balances.
    let tb = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(tb.total_dr, tb.total_cr);
}

#[test]
fn an_opening_capital_already_counted_elsewhere_posts_against_the_difference() {
    let w = support::world_with(50_000, false);
    let d = make(&w, "50,000", Some(OpeningCapitalWhere::CountedElsewhere));
    assert_eq!(cash(&w), 50_000, "cash does not move");
    assert_eq!(balance(&w, d.capital_account_id), (BalanceSide::Cr, 50_000));
    let difference = support::id_of(&w.env, "06019999");
    assert_eq!(balance(&w, difference), (BalanceSide::Dr, 50_000));
}

#[test]
fn a_cash_opening_that_is_changed_stays_in_cash() {
    let w = support::world_with(50_000, false);
    let d = make(&w, "50,000", Some(OpeningCapitalWhere::Cash));
    assert_eq!(cash(&w), 100_000);
    // 50,000 down to 30,000: 20,000 leaves cash, the new entry is in cash too.
    change_opening(&w, d.capital_account_id, "30,000").unwrap();
    assert_eq!(cash(&w), 80_000);
    assert_eq!(balance(&w, d.capital_account_id), (BalanceSide::Cr, 30_000));
    assert_eq!(balance(&w, support::id_of(&w.env, "06019999")).1, 0);
    // Up to 70,000: cash goes up by 40,000.
    change_opening(&w, d.capital_account_id, "70,000").unwrap();
    assert_eq!(cash(&w), 120_000);
}

#[test]
fn a_cash_opening_cannot_be_changed_when_cash_would_go_below_zero() {
    // Cash is 50,000 and D's 50,000 is in it. After 80,000 is paid out, cash is 20,000 and the
    // opening cannot be lowered by 30,000 without the answer of the cash box: cash would be -10,000
    // (plan 80 gate R4-4: the form asks, and `gate_c_money.rs` saves it with the answer "yes"). The
    // world has shares because an advance is refused while partners exist and no share was set
    // (plan 80 gate Q-R1-3).
    let w = support::world();
    let d = make(&w, "50,000", Some(OpeningCapitalWhere::Cash));
    let [a, _, _] = w.partners;
    let expense = mandi_services::core::vouchers::AdvanceInput {
        confirm_cash_below_zero: true,
        date: "10-10-2026".into(),
        seller_account_id: w.rahim,
        amount: "80,000".into(),
        removed: Default::default(),
        expected_goods: String::new(),
        narration: String::new(),
        reason: None,
    };
    let _ = a;
    mandi_services::vouchers::save_advance(&w.env.ctx, &w.owner, &expense).unwrap();
    assert_eq!(cash(&w), 20_000);
    let error = change_opening(&w, d.capital_account_id, "20,000").unwrap_err();
    match error {
        ServiceError::Rejected(items) => {
            assert_eq!(items[0].code, ErrorCode::ErrCashBelowZeroNotConfirmed);
            assert_eq!(
                items[0].values.get("cash_after_paisa"),
                Some(&ErrorValue::Number(-10_000 * 100))
            );
        }
        other => panic!("{other:?}"),
    }
    // Nothing changed.
    assert_eq!(cash(&w), 20_000);
    assert_eq!(balance(&w, d.capital_account_id), (BalanceSide::Cr, 50_000));
    // A change that keeps cash at or above zero works: 50,000 to 30,000 leaves 0.
    change_opening(&w, d.capital_account_id, "30,000").unwrap();
    assert_eq!(cash(&w), 0);
}

#[test]
fn removing_a_cash_opening_takes_the_cash_out_and_is_checked() {
    let w = support::world_with(50_000, false);
    let d = make(&w, "50,000", Some(OpeningCapitalWhere::Cash));
    assert_eq!(cash(&w), 100_000);
    // Taking the opening away: 50,000 leaves cash, which is allowed while cash stays at or above 0.
    let version = accounts::get(&w.env.ctx, d.capital_account_id)
        .unwrap()
        .version;
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: d.capital_account_id,
                version,
                opening_date: String::new(),
                opening_dr: String::new(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    assert_eq!(cash(&w), 50_000);
    assert_eq!(balance(&w, d.capital_account_id).1, 0);
}

#[test]
fn the_preview_says_what_the_save_will_say_for_a_cash_opening() {
    let w = support::world();
    let d = make(&w, "50,000", Some(OpeningCapitalWhere::Cash));
    advance(&w, "80,000");
    assert_eq!(cash(&w), 20_000);
    let version = accounts::get(&w.env.ctx, d.capital_account_id)
        .unwrap()
        .version;
    let input = |amount: &str| OpeningsInput {
        lines: vec![OpeningLineInput {
            account_id: d.capital_account_id,
            version,
            opening_date: "10-10-2026".into(),
            opening_dr: String::new(),
            opening_cr: amount.into(),
            opening_is_advance: false,
        }],
    };
    // 50,000 down to 20,000 takes 30,000 out of 20,000: the preview refuses it, as the save does.
    let refused = accounts::openings_preview(&w.env.ctx, &input("20,000")).unwrap();
    assert_eq!(
        refused.rows[0].errors[0].code,
        ErrorCode::ErrCashBelowZeroNotConfirmed
    );
    assert_eq!(refused.rows[0].errors[0].field, "lines[0].opening_dr");
    // 50,000 down to 30,000 leaves cash at 0: no error, and ابتدائی فرق does not move.
    let ok = accounts::openings_preview(&w.env.ctx, &input("30,000")).unwrap();
    assert!(ok.rows[0].errors.is_empty());
    assert_eq!(ok.opening_difference_after.amount_paisa.rupees(), 0);
}

#[test]
fn an_opening_against_the_difference_stays_against_it_when_changed() {
    let w = support::world_with(50_000, false);
    let d = make(&w, "50,000", Some(OpeningCapitalWhere::CountedElsewhere));
    change_opening(&w, d.capital_account_id, "30,000").unwrap();
    assert_eq!(cash(&w), 50_000, "cash does not move");
    let difference = support::id_of(&w.env, "06019999");
    assert_eq!(balance(&w, difference), (BalanceSide::Dr, 30_000));
}
