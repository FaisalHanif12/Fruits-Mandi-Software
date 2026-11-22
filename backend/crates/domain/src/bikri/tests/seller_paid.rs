//! A seller's money paid on the spot (PRD FR-12.23 v1.53, plan 91): the spread over his lots, the
//! lines of his summary, and the summary inside `calc_posting`.
//!
//! The worked numbers are the ones of the plan: previous 30,000 بنام, credit 20,000, paid 5,000 gives
//! cut 15,000 and new 15,000 بنام; credits 12,000 and 8,000 with 15,000 paid give 12,000 and 3,000.

use mandi_core::accounts::BalanceSide;
use mandi_core::bikri::{LineChargeInput, LineChargeMethod};
use proptest::prelude::*;

use super::*;
use crate::bikri::{SpreadLot, merge_sellers, seller_lines, spread_seller_paid};

fn lot_at(index: usize, seller_id: i64, credit: i64) -> SpreadLot {
    SpreadLot {
        index,
        seller_id,
        credit: p(credit),
        takes_cash: true,
    }
}

fn amounts(parts: Option<Vec<(usize, Paisa)>>) -> Vec<(usize, i64)> {
    parts
        .unwrap()
        .into_iter()
        .map(|(i, cash)| (i, cash.rupees()))
        .collect()
}

// ---- the spread ----

#[test]
fn two_lots_with_15000_give_12000_and_3000() {
    let lots = [lot_at(0, RAHIM, 12_000), lot_at(1, RAHIM, 8_000)];
    let parts = spread_seller_paid(&lots, RAHIM, p(15_000)).unwrap();
    assert_eq!(amounts(parts), vec![(0, 12_000), (1, 3_000)]);
}

#[test]
fn the_rest_goes_on_the_last_lot_even_above_its_credit() {
    let lots = [lot_at(0, RAHIM, 12_000), lot_at(1, RAHIM, 8_000)];
    let parts = spread_seller_paid(&lots, RAHIM, p(30_000)).unwrap();
    assert_eq!(amounts(parts), vec![(0, 12_000), (1, 18_000)]);
}

#[test]
fn a_lot_that_credits_nothing_takes_nothing_but_the_last_one_takes_the_rest() {
    // Lot 1 has a negative credit (its charges are more than its sales), lot 2 credits 0.
    let lots = [
        lot_at(0, RAHIM, -1_000),
        lot_at(1, RAHIM, 5_000),
        lot_at(2, RAHIM, 0),
    ];
    let parts = spread_seller_paid(&lots, RAHIM, p(7_000)).unwrap();
    assert_eq!(amounts(parts), vec![(0, 0), (1, 5_000), (2, 2_000)]);
}

#[test]
fn only_the_lots_of_that_seller_that_take_cash_are_used() {
    let mut own = lot_at(1, RAHIM, 9_000);
    own.takes_cash = false;
    let lots = [
        lot_at(0, KARIM, 4_000),
        own,
        lot_at(2, RAHIM, 3_000),
        lot_at(3, KARIM, 4_000),
    ];
    let parts = spread_seller_paid(&lots, RAHIM, p(5_000)).unwrap();
    assert_eq!(amounts(parts), vec![(2, 5_000)]);
}

#[test]
fn no_lot_that_takes_cash_gives_nothing_to_place() {
    let mut only = lot_at(0, RAHIM, 3_000);
    only.takes_cash = false;
    assert_eq!(spread_seller_paid(&[only], RAHIM, p(1_000)).unwrap(), None);
    assert_eq!(spread_seller_paid(&[], RAHIM, p(1_000)).unwrap(), None);
}

#[test]
fn zero_puts_zero_on_every_lot() {
    let lots = [lot_at(0, RAHIM, 12_000), lot_at(1, RAHIM, 8_000)];
    let parts = spread_seller_paid(&lots, RAHIM, Paisa::ZERO).unwrap();
    assert_eq!(amounts(parts), vec![(0, 0), (1, 0)]);
}

#[test]
fn a_negative_amount_is_refused() {
    let lots = [lot_at(0, RAHIM, 12_000)];
    let negative = Paisa::try_from_paisa(-100).unwrap();
    assert!(spread_seller_paid(&lots, RAHIM, negative).is_err());
}

// ---- the lines of the summary ----

/// previous in rupees, debit positive; credit in rupees; paid in rupees.
fn lines(previous: i64, credit: i64, paid: i64) -> mandi_core::bikri::SellerPaidLines {
    seller_lines(
        RAHIM,
        p(previous),
        p(credit),
        p(paid),
        Paisa::ZERO,
        Paisa::ZERO,
    )
    .unwrap()
}

/// The same with a پیشگی typed on his lots (PRD FR-12.52 v1.57).
fn lines_with_advance(
    previous: i64,
    credit: i64,
    paid: i64,
    advance: i64,
) -> mandi_core::bikri::SellerPaidLines {
    seller_lines(
        RAHIM,
        p(previous),
        p(credit),
        p(paid),
        p(advance),
        Paisa::ZERO,
    )
    .unwrap()
}

/// PRD FR-12.52 v1.57, money agent worked numbers: previous 9,260 جمع (-9,260), credit 52,900, نقد
/// 30,000, advance 20,000: new = -9,260 + 30,000 + 20,000 - 52,900 = -12,160 (12,160 جمع). The
/// advance is not part of the sale: carry, cut, added and the limit are those without it.
#[test]
fn an_advance_moves_the_new_balance_and_nothing_else_of_the_summary() {
    let with = lines_with_advance(-9_260, 52_900, 30_000, 20_000);
    let without = lines(-9_260, 52_900, 30_000);
    assert_eq!(with.advance_paisa, p(20_000));
    assert_eq!(
        (with.new_paisa, with.new_side),
        (p(12_160), BalanceSide::Cr)
    );
    // The sale's own lines are the same with or without it.
    assert_eq!(with.carry_paisa, p(22_900));
    assert_eq!(with.carry_paisa, without.carry_paisa);
    assert_eq!(with.cut_paisa, without.cut_paisa);
    assert_eq!(with.added_to_credit_paisa, p(22_900));
    assert_eq!(with.limit_paisa, without.limit_paisa);
    assert_eq!(with.needs_confirm, without.needs_confirm);
    assert_eq!(
        with.working.message_id,
        "working-seller-new-balance-advance"
    );
    assert_eq!(without.working.message_id, "working-seller-new-balance");
    let result = with
        .working
        .args
        .iter()
        .find(|a| a.name == "result")
        .unwrap();
    assert_eq!(result.value, WorkingValue::Money { paisa: p(-12_160) });
    // It owes more after the advance than without it, by exactly the advance.
    // (both are shown as جمع amounts: 32,160 and 12,160).
    assert_eq!(without.new_side, BalanceSide::Cr);
    assert_eq!(
        without.new_paisa.checked_sub(with.new_paisa).unwrap(),
        p(20_000)
    );
}

#[test]
fn case_a_previous_30000_credit_20000_paid_5000() {
    let l = lines(30_000, 20_000, 5_000);
    assert_eq!(
        (l.previous_paisa, l.previous_side),
        (p(30_000), BalanceSide::Dr)
    );
    assert_eq!(l.credit_paisa, p(20_000));
    assert_eq!(l.paid_paisa, p(5_000));
    assert_eq!(l.carry_paisa, p(15_000));
    assert_eq!(l.cut_paisa, p(15_000));
    assert_eq!(l.added_to_credit_paisa, Paisa::ZERO);
    assert_eq!(l.over_paid_paisa, Paisa::ZERO);
    assert_eq!(l.increase_owed_paisa, Paisa::ZERO);
    assert_eq!((l.new_paisa, l.new_side), (p(15_000), BalanceSide::Dr));
    assert_eq!(l.limit_paisa, p(20_000));
    assert!(!l.needs_confirm);
}

#[test]
fn case_c_the_shop_owed_him_10000_credit_20000_paid_28000() {
    let l = lines(-10_000, 20_000, 28_000);
    assert_eq!(
        (l.previous_paisa, l.previous_side),
        (p(10_000), BalanceSide::Cr)
    );
    assert_eq!(
        l.carry_paisa,
        Paisa::ZERO,
        "he took more than today's credit"
    );
    assert_eq!(l.cut_paisa, Paisa::ZERO);
    assert_eq!(l.added_to_credit_paisa, Paisa::ZERO);
    assert_eq!(l.over_paid_paisa, p(8_000));
    assert_eq!((l.new_paisa, l.new_side), (p(2_000), BalanceSide::Cr));
    assert_eq!(l.limit_paisa, p(30_000));
    assert!(
        !l.needs_confirm,
        "within today's credit plus what he was owed"
    );
}

#[test]
fn case_d_previous_30000_credit_20000_paid_25000_needs_the_question() {
    let l = lines(30_000, 20_000, 25_000);
    assert_eq!(l.over_paid_paisa, p(5_000));
    assert_eq!(l.cut_paisa, Paisa::ZERO);
    assert_eq!((l.new_paisa, l.new_side), (p(35_000), BalanceSide::Dr));
    assert_eq!(l.limit_paisa, p(20_000));
    assert!(l.needs_confirm);
}

#[test]
fn case_e_event_1_rahim_with_his_open_advance() {
    // He owes 15,000 (the advance), today's credit is 24,316 and he takes 5,000 (PRD 4.4, 4.7).
    let l = lines(15_000, 24_316, 5_000);
    assert_eq!(l.carry_paisa, p(19_316));
    assert_eq!(l.cut_paisa, p(15_000));
    assert_eq!(l.added_to_credit_paisa, p(4_316));
    assert_eq!((l.new_paisa, l.new_side), (p(4_316), BalanceSide::Cr));
    assert!(!l.needs_confirm);
}

#[test]
fn credit_above_what_he_owed_turns_the_rest_into_money_the_shop_owes() {
    // Owes 4,000, credit 50,000, takes 5,000: 45,000 is not paid; 4,000 of it cuts what he owed and
    // 41,000 is added to what the shop owes him.
    let l = lines(4_000, 50_000, 5_000);
    assert_eq!(l.carry_paisa, p(45_000));
    assert_eq!(l.cut_paisa, p(4_000));
    assert_eq!(l.added_to_credit_paisa, p(41_000));
    assert_eq!((l.new_paisa, l.new_side), (p(41_000), BalanceSide::Cr));
}

#[test]
fn a_negative_credit_is_not_called_over_paid() {
    // The charges of his lots are 1,000 more than their sales and nothing was paid: he owes 1,000
    // more. Nothing is over paid and no question is asked.
    let l = lines(0, -1_000, 0);
    assert_eq!(l.increase_owed_paisa, p(1_000));
    assert_eq!(l.over_paid_paisa, Paisa::ZERO);
    assert_eq!(l.cut_paisa, Paisa::ZERO);
    assert_eq!(l.added_to_credit_paisa, Paisa::ZERO);
    assert_eq!((l.new_paisa, l.new_side), (p(1_000), BalanceSide::Dr));
    assert_eq!(l.limit_paisa, Paisa::ZERO);
    assert!(!l.needs_confirm);
    // With 500 paid on top, all of it is over paid and the question is asked.
    let l = lines(0, -1_000, 500);
    assert_eq!(l.increase_owed_paisa, p(1_000));
    assert_eq!(l.over_paid_paisa, p(500));
    assert_eq!((l.new_paisa, l.new_side), (p(1_500), BalanceSide::Dr));
    assert!(l.needs_confirm);
}

#[test]
fn a_negative_credit_takes_off_what_the_shop_owed_him_before_the_limit() {
    // The shop owed him 10,000, the charges of his lots are 2,000 above their sales and he takes
    // 10,000: he would go from being owed 10,000 to owing 2,000 (-10,000 + 10,000 + 2,000), so the
    // question is asked. The limit is 8,000.
    let l = lines(-10_000, -2_000, 10_000);
    assert_eq!(l.limit_paisa, p(8_000));
    assert!(l.needs_confirm);
    assert_eq!((l.new_paisa, l.new_side), (p(2_000), BalanceSide::Dr));
    // Taking 8,000 stays within it: he is still owed 0 after the charges.
    let l = lines(-10_000, -2_000, 8_000);
    assert!(!l.needs_confirm);
    assert_eq!(l.new_side, BalanceSide::Zero);
}

#[test]
fn cash_on_arrival_with_no_sale_needs_the_question() {
    // A lot received, nothing sold, 5,000 given on arrival: today's credit is 0.
    let l = lines(0, 0, 5_000);
    assert_eq!(l.over_paid_paisa, p(5_000));
    assert_eq!((l.new_paisa, l.new_side), (p(5_000), BalanceSide::Dr));
    assert!(l.needs_confirm);
}

#[test]
fn nothing_at_all_is_a_zero_balance() {
    let l = lines(0, 0, 0);
    assert_eq!(l.previous_side, BalanceSide::Zero);
    assert_eq!(l.new_side, BalanceSide::Zero);
    assert!(!l.needs_confirm);
}

#[test]
fn the_working_of_the_new_balance_is_the_new_balance() {
    use mandi_core::rules::WorkingValue;
    let l = lines(30_000, 20_000, 5_000);
    assert_eq!(l.working.message_id, "working-seller-new-balance");
    let result = l
        .working
        .args
        .iter()
        .find(|a| a.name == "result")
        .map(|a| a.value);
    // Debit positive: 30,000 + 5,000 - 20,000 = 15,000.
    assert_eq!(result, Some(WorkingValue::Money { paisa: p(15_000) }));
    let l = lines(-10_000, 20_000, 28_000);
    let result = l.working.args.iter().find(|a| a.name == "result").unwrap();
    assert_eq!(result.value, WorkingValue::Money { paisa: p(-2_000) });
}

#[test]
fn a_negative_paid_is_refused() {
    let negative = Paisa::try_from_paisa(-100).unwrap();
    assert!(
        seller_lines(
            RAHIM,
            Paisa::ZERO,
            p(10),
            negative,
            Paisa::ZERO,
            Paisa::ZERO
        )
        .is_err()
    );
    assert!(
        seller_lines(
            RAHIM,
            Paisa::ZERO,
            p(10),
            Paisa::ZERO,
            negative,
            Paisa::ZERO
        )
        .is_err()
    );
}

// ---- the summary inside calc_posting ----

/// One lot of RAHIM's, 100 نگ at 200 = 20,000 sold to AKRAM, commission typed 0% so the credit is
/// exactly 20,000, no lot charge, and the نقد given.
fn rahim_lot(cash: Option<&str>) -> mandi_core::bikri::LotInput {
    let mut sale = sale(AKRAM, "100", "200");
    sale.seller_comm = Some(LineChargeInput {
        method: LineChargeMethod::Percent,
        value: "0".into(),
    });
    let mut lot = with(bare_lot(RAHIM, BANANA, "100"), vec![sale]);
    lot.cash_paid = cash.map(str::to_owned);
    lot
}

fn with_previous(seller: i64, rupees: i64) -> Lookups {
    let mut lookups = lookups();
    lookups.previous.insert(seller, p(rupees));
    lookups
}

#[test]
fn calc_posting_gives_the_summary_of_case_a() {
    let calc = run_with(
        &draft(vec![rahim_lot(Some("5,000"))]),
        &with_previous(RAHIM, 30_000),
        default_rules(),
        switches(),
    );
    assert!(calc.errors.is_empty(), "{:?}", calc.errors);
    assert_eq!(calc.sellers.len(), 1);
    let s = &calc.sellers[0];
    assert_eq!(s.seller_account_id, RAHIM);
    assert_eq!(
        (s.previous_paisa, s.previous_side),
        (p(30_000), BalanceSide::Dr)
    );
    assert_eq!(s.credit_paisa, p(20_000));
    assert_eq!(s.paid_paisa, p(5_000));
    assert_eq!(s.cut_paisa, p(15_000));
    assert_eq!((s.new_paisa, s.new_side), (p(15_000), BalanceSide::Dr));
    // The cash of the summary is the cash of the journal.
    assert_eq!(calc.journal.lots[0].cash_paid, p(5_000));
    assert_eq!(calc.lots[0].totals.as_ref().unwrap().cash_paid, p(5_000));
}

#[test]
fn event_1_summary_is_rahim_24316_credit_and_5000_paid() {
    let calc = run_with(
        &event1(),
        &with_previous(RAHIM, 15_000),
        default_rules(),
        switches(),
    );
    let s = &calc.sellers[0];
    assert_eq!(s.credit_paisa, p(24_316));
    assert_eq!(s.paid_paisa, p(5_000));
    assert_eq!(s.cut_paisa, p(15_000));
    assert_eq!(s.added_to_credit_paisa, p(4_316));
    assert_eq!((s.new_paisa, s.new_side), (p(4_316), BalanceSide::Cr));
    assert!(!s.needs_confirm);
}

#[test]
fn two_lots_of_one_seller_are_one_summary_and_each_seller_has_his_own() {
    let mut second = with(
        bare_lot(RAHIM, APPLE, "10"),
        vec![sale(ARSHAD, "10", "100")],
    );
    second.cash_paid = Some("300".into());
    let karim = with(
        bare_lot(KARIM, BANANA, "10"),
        vec![sale(ARSHAD, "10", "100")],
    );
    let calc = run_with(
        &draft(vec![rahim_lot(Some("5,000")), karim, second]),
        &lookups(),
        default_rules(),
        switches(),
    );
    let ids: Vec<i64> = calc.sellers.iter().map(|s| s.seller_account_id).collect();
    assert_eq!(
        ids,
        vec![RAHIM, KARIM],
        "in the order of each seller's first lot"
    );
    // Rahim: 20,000 + (1,000 - 6% commission 60) = 20,940 credit; 5,000 + 300 paid.
    assert_eq!(calc.sellers[0].credit_paisa, p(20_940));
    assert_eq!(calc.sellers[0].paid_paisa, p(5_300));
    assert_eq!(calc.sellers[1].credit_paisa, p(940));
    assert_eq!(calc.sellers[1].paid_paisa, Paisa::ZERO);
}

#[test]
fn trip_goods_have_no_summary_and_a_khata_has_one() {
    let own = with(
        bare_lot(OWN_GOODS, BANANA, "10"),
        vec![sale(ARSHAD, "10", "100")],
    );
    let khata = with(
        bare_lot(KHATA, BANANA, "10"),
        vec![sale(ARSHAD, "10", "100")],
    );
    let calc = run_with(
        &draft(vec![own, khata]),
        &lookups(),
        default_rules(),
        switches(),
    );
    // The اپنا مال lot is refused here for want of a purchase, so it has no totals either way.
    let ids: Vec<i64> = calc.sellers.iter().map(|s| s.seller_account_id).collect();
    assert_eq!(ids, vec![KHATA]);
}

#[test]
fn a_lot_that_cannot_be_worked_out_has_no_summary() {
    let mut lot = rahim_lot(Some("5,000"));
    lot.item_id = None;
    let calc = run(&draft(vec![lot]));
    assert!(calc.sellers.is_empty());
}

#[test]
fn a_mixed_screen_puts_a_seller_of_two_entries_in_one_summary() {
    // Carried lot: credit 2,632, no cash. New lot: credit 20,000, cash 5,000. Both worked out
    // against the same previous balance, 30,000 بنام.
    let carried = lines(30_000, 2_632, 0);
    let new = lines(30_000, 20_000, 5_000);
    let merged = merge_sellers(&[&[carried], &[new]]).unwrap();
    assert_eq!(merged.len(), 1);
    let m = &merged[0];
    assert_eq!(
        (m.previous_paisa, m.previous_side),
        (p(30_000), BalanceSide::Dr)
    );
    assert_eq!(m.credit_paisa, p(22_632));
    assert_eq!(m.paid_paisa, p(5_000));
    // 30,000 + 5,000 - 22,632 = 12,368 بنام: what the ledger holds after both entries.
    assert_eq!((m.new_paisa, m.new_side), (p(12_368), BalanceSide::Dr));
    assert_eq!(m.cut_paisa, p(17_632));
}

#[test]
fn merging_a_credit_balance_keeps_its_side() {
    let a = lines(-1_000, 500, 0);
    let b = lines(-1_000, 700, 100);
    let merged = merge_sellers(&[&[a], &[b]]).unwrap();
    assert_eq!(
        (merged[0].previous_paisa, merged[0].previous_side),
        (p(1_000), BalanceSide::Cr)
    );
    // -1,000 + 100 - 1,200 = -2,100.
    assert_eq!(
        (merged[0].new_paisa, merged[0].new_side),
        (p(2_100), BalanceSide::Cr)
    );
}

// ---- properties ----

fn signed(l: &mandi_core::bikri::SellerPaidLines, new: bool) -> i64 {
    let (amount, side) = if new {
        (l.new_paisa, l.new_side)
    } else {
        (l.previous_paisa, l.previous_side)
    };
    match side {
        BalanceSide::Cr => -amount.as_paisa(),
        _ => amount.as_paisa(),
    }
}

proptest! {
    #[test]
    fn the_spread_always_adds_up_and_is_never_negative(
        credits in prop::collection::vec(-50_000i64..=500_000, 1..8),
        amount in 0i64..=2_000_000,
    ) {
        let lots: Vec<SpreadLot> = credits
            .iter()
            .enumerate()
            .map(|(i, c)| lot_at(i, RAHIM, *c))
            .collect();
        let parts = spread_seller_paid(&lots, RAHIM, p(amount)).unwrap().unwrap();
        prop_assert_eq!(parts.len(), lots.len());
        let total: i64 = parts.iter().map(|(_, cash)| cash.rupees()).sum();
        prop_assert_eq!(total, amount);
        prop_assert!(parts.iter().all(|(_, cash)| !cash.is_negative()));
        // Every lot but the last stays within its own credit (never below 0).
        for ((index, cash), lot) in parts.iter().zip(&lots).take(lots.len() - 1) {
            prop_assert_eq!(*index, lot.index);
            prop_assert!(cash.rupees() <= lot.credit.rupees().max(0));
        }
    }

    #[test]
    fn the_new_balance_is_previous_plus_paid_less_credit(
        previous in -1_000_000i64..=1_000_000,
        credit in -500_000i64..=1_000_000,
        paid in 0i64..=2_000_000,
    ) {
        let l = lines(previous, credit, paid);
        let (previous_p, credit_p, paid_p) = (p(previous).as_paisa(), p(credit).as_paisa(), p(paid).as_paisa());
        prop_assert_eq!(signed(&l, true), previous_p + paid_p - credit_p);
        prop_assert_eq!(signed(&l, false), previous_p);
        // The parts say the same thing as the new balance.
        let parts = l.cut_paisa.as_paisa() + l.added_to_credit_paisa.as_paisa();
        prop_assert_eq!(parts, p(credit.max(0)).as_paisa().saturating_sub(paid_p).max(0));
        // The rest of today's sale that goes into his account is those parts, never negative.
        prop_assert_eq!(l.carry_paisa.as_paisa(), parts);
        prop_assert!(l.cut_paisa.as_paisa() <= p(previous.max(0)).as_paisa());
        prop_assert_eq!(l.over_paid_paisa.as_paisa(), (paid_p - p(credit.max(0)).as_paisa()).max(0));
        prop_assert_eq!(l.increase_owed_paisa.as_paisa(), p((-credit).max(0)).as_paisa());
        // The question is asked exactly when paid is above today's credit plus what he was owed.
        let limit = (p(credit).as_paisa() + p((-previous).max(0)).as_paisa()).max(0);
        prop_assert_eq!(l.limit_paisa.as_paisa(), limit);
        prop_assert_eq!(l.needs_confirm, paid_p > limit);
        // A balance is never shown negative.
        prop_assert!(!l.previous_paisa.is_negative() && !l.new_paisa.is_negative());
    }
}

/// Lead decision, plan 95 (PRD FR-12.52 v1.57): an advance on lots with the tick ON is taken off today's
/// credit, so it counts with نقد for the question "above today's credit". Previous 0, credit 52,900,
/// نقد 50,000 and advance 20,000: 70,000 is above 52,900, so he is asked; with the tick off, or an
/// advance of 2,000 (52,000 is within it), he is not.
#[test]
fn an_advance_with_the_tick_on_counts_for_the_question_and_with_the_tick_off_it_does_not() {
    let ask = |advance: i64, adjusted: i64| {
        seller_lines(
            RAHIM,
            Paisa::ZERO,
            p(52_900),
            p(50_000),
            p(advance),
            p(adjusted),
        )
        .unwrap()
        .needs_confirm
    };
    assert!(ask(20_000, 20_000), "tick on: 70,000 > 52,900");
    assert!(!ask(20_000, 0), "tick off: only the نقد 50,000 counts");
    assert!(!ask(2_000, 2_000), "tick on, 52,000 is within 52,900");
    assert!(ask(2_900, 2_900) == (52_900 < 50_000 + 2_900));
    // The limit and the other lines are those without it.
    let on = seller_lines(
        RAHIM,
        Paisa::ZERO,
        p(52_900),
        p(50_000),
        p(20_000),
        p(20_000),
    )
    .unwrap();
    let off = seller_lines(
        RAHIM,
        Paisa::ZERO,
        p(52_900),
        p(50_000),
        p(20_000),
        Paisa::ZERO,
    )
    .unwrap();
    assert_eq!(on.limit_paisa, off.limit_paisa);
    assert_eq!(on.paid_paisa, off.paid_paisa);
    assert_eq!(on.advance_adjusted_paisa, p(20_000));
}

/// A negative credit (the charges are more than the sales) leaves a limit of 0, so an advance with
/// the tick on is asked; with the tick off it is not (money agent, plan 95 review).
#[test]
fn a_ticked_advance_is_asked_against_a_limit_of_zero_when_the_credit_is_negative() {
    let ask = |adjusted: i64| {
        seller_lines(
            RAHIM,
            Paisa::ZERO,
            p(-500),
            Paisa::ZERO,
            p(1_000),
            p(adjusted),
        )
        .unwrap()
    };
    let on = ask(1_000);
    assert_eq!(on.limit_paisa, Paisa::ZERO);
    assert!(on.needs_confirm, "1,000 is above a limit of 0");
    assert!(!ask(0).needs_confirm, "tick off: nothing is counted");
    // The ledger does not depend on the tick: previous 0 + advance 1,000 - credit (-500) = 1,500.
    assert_eq!(on.new_paisa, ask(0).new_paisa);
}

/// Two parts of one save for one seller (a carried lot and a new lot): the advance that counts is the
/// sum over the parts of the lots with the tick on, and the new balance is the sum of all of it.
#[test]
fn merging_the_parts_of_a_save_sums_the_ticked_advance() {
    let first = seller_lines(
        RAHIM,
        Paisa::ZERO,
        p(40_000),
        p(10_000),
        p(20_000),
        p(20_000),
    )
    .unwrap();
    let second = seller_lines(
        RAHIM,
        Paisa::ZERO,
        p(12_900),
        p(20_000),
        p(5_000),
        Paisa::ZERO,
    )
    .unwrap();
    let merged = merge_sellers(&[&[first], &[second]]).unwrap();
    assert_eq!(merged.len(), 1);
    let m = &merged[0];
    assert_eq!(m.advance_paisa, p(25_000));
    assert_eq!(m.advance_adjusted_paisa, p(20_000));
    assert_eq!(m.paid_paisa, p(30_000));
    assert_eq!(m.credit_paisa, p(52_900));
    // 30,000 + 20,000 = 50,000 is within 52,900: no question; the ticked 20,000 only is counted.
    assert!(!m.needs_confirm);
    // new = 0 + paid 30,000 + advance 25,000 - credit 52,900 = 2,100 (he owes).
    assert_eq!(m.new_paisa, p(2_100));
    assert_eq!(m.new_side, BalanceSide::Dr);
}

proptest! {
    #[test]
    fn the_tick_changes_the_question_and_never_the_ledger(
        previous in -1_000_000i64..=1_000_000,
        credit in -500_000i64..=1_000_000,
        paid in 0i64..=2_000_000,
        advance in 0i64..=2_000_000,
        ticked in proptest::bool::ANY,
    ) {
        let adjusted = if ticked { advance } else { 0 };
        let on = seller_lines(RAHIM, p(previous), p(credit), p(paid), p(advance), p(advance)).unwrap();
        let off = seller_lines(RAHIM, p(previous), p(credit), p(paid), p(advance), Paisa::ZERO).unwrap();
        let this = seller_lines(RAHIM, p(previous), p(credit), p(paid), p(advance), p(adjusted)).unwrap();
        // The tick never moves the ledger or any other line.
        prop_assert_eq!(on.new_paisa, off.new_paisa);
        prop_assert_eq!(on.new_side, off.new_side);
        prop_assert_eq!(on.limit_paisa, off.limit_paisa);
        prop_assert_eq!(on.carry_paisa, off.carry_paisa);
        // The question counts paid + the ticked advance.
        let limit = this.limit_paisa.as_paisa();
        prop_assert_eq!(this.needs_confirm, p(paid).as_paisa() + p(adjusted).as_paisa() > limit);
        // A ticked advance can only add to the question, never remove it.
        prop_assert!(!off.needs_confirm || on.needs_confirm);
    }
}
