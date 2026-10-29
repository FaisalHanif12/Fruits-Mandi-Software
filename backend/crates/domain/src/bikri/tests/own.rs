//! The cost of own goods (PRD C21, FR-12.26, plan 16b): the worked numbers of E2E-34 and E2E-34b
//! and the cases the money agent asked for, then the property of Code Standards 9.3: for any
//! sequence of sales and write-offs of one purchase, the cost released never exceeds the total
//! cost, is never negative, and equals it once nothing is left.

use mandi_core::money::Paisa;
use proptest::prelude::*;

use crate::bikri::own_cost;

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn cost(left: i64, qty_left: i64, taken: i64) -> i64 {
    own_cost(rupees(left), qty_left, taken).unwrap().0.rupees()
}

/// E2E-34: 10,400 over 20 نگ. Selling 15 takes 7,800; the last 5 take the 2,600 that is left.
#[test]
fn e2e_34_and_34b_10400_over_20() {
    assert_eq!(cost(10_400, 20, 15), 7_800);
    assert_eq!(cost(2_600, 5, 5), 2_600);
}

/// The working says what was done: the share of the cost, or "the last نگ take all".
#[test]
fn the_working_names_the_rule_used() {
    let (taken, w) = own_cost(rupees(10_400), 20, 15).unwrap();
    assert_eq!(w.message_id, "working-cost-of-goods");
    assert_eq!(taken.rupees(), 7_800);
    let (all, w) = own_cost(rupees(2_600), 5, 5).unwrap();
    assert_eq!(w.message_id, "working-cost-of-goods-last");
    assert_eq!(all.rupees(), 2_600);
}

/// Money agent test 1: cost 1,000 over 3 نگ. Two lines of 1 take 333 and 334, and the last نگ takes
/// the 333 that is left, 1,000 in all.
#[test]
fn a_cost_that_does_not_divide_gives_the_leftover_rupee_to_the_larger_part() {
    let first = cost(1_000, 3, 1);
    assert_eq!(first, 333);
    // 667 over 2: a tie, so the part taken gets the leftover rupee.
    let second = cost(667, 2, 1);
    assert_eq!(second, 334);
    let last = cost(333, 1, 1);
    assert_eq!(last, 333);
    assert_eq!(first + second + last, 1_000);
}

/// Money agent point 2: cost 10 over 4 نگ. Two lines of 1 take 2 and 2; one line of 2 takes 5. The
/// take is per line (a team decision), and neither takes more than the cost left.
#[test]
fn the_take_is_per_line() {
    assert_eq!(cost(10, 4, 1), 2);
    assert_eq!(cost(8, 3, 1), 2);
    assert_eq!(cost(10, 4, 2), 5);
}

/// Money agent test 6: a weight purchase of 3 نگ whose value is 1,235 (12.345 kg at 100). One نگ
/// takes 411, and the 824 left is shared by the other 2.
#[test]
fn a_weight_purchase_is_costed_by_nag() {
    assert_eq!(cost(1_235, 3, 1), 411);
    assert_eq!(cost(824, 2, 2), 824);
}

#[test]
fn a_quantity_outside_the_stock_is_refused() {
    assert!(own_cost(rupees(100), 5, 0).is_err());
    assert!(own_cost(rupees(100), 5, 6).is_err());
    assert!(own_cost(Paisa::ZERO, 5, 5).is_ok());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Code Standards 9.3 and PRD C21: whatever sizes are sold or written off, in any order, the
    /// cost released adds up to at most the total, every take is 0 or more, and the whole cost is
    /// released exactly when the last نگ goes.
    #[test]
    fn cost_released_never_exceeds_the_total_and_equals_it_when_nothing_is_left(
        qty in 1_i64..200,
        total in 1_i64..1_000_000,
        takes in proptest::collection::vec(1_i64..40, 1..30),
    ) {
        let mut qty_left = qty;
        let mut cost_left = total;
        let mut released = 0_i64;
        for want in takes {
            if qty_left == 0 {
                break;
            }
            let taken_qty = want.min(qty_left);
            let (part, _) = own_cost(rupees(cost_left), qty_left, taken_qty).unwrap();
            let part = part.rupees();
            prop_assert!(part >= 0);
            prop_assert!(part <= cost_left);
            released += part;
            cost_left -= part;
            qty_left -= taken_qty;
            prop_assert!(released <= total);
            prop_assert_eq!(released + cost_left, total);
            if qty_left == 0 {
                prop_assert_eq!(released, total);
                prop_assert_eq!(cost_left, 0);
            }
        }
    }
}

// ---- The own lot in calc_posting (plan 16b, T2b, money review second pass) ----

mod lot {
    use std::collections::HashMap;

    use mandi_core::bikri::{LotInput, prepare_draft};
    use mandi_core::error::{ErrorCode, ErrorValue};
    use mandi_core::ids::{AccountId, PartnerId};
    use mandi_core::money::Bps;
    use mandi_core::ventures::{MoneySource, TripRef};

    use super::super::edit::{per_unit_charge, percent, stored_sale};
    use super::super::*;
    use crate::bikri::{
        PostingKind, PurchaseFacts, StoredIdentity, StoredLot, StoredPosting, TripPartnerFacts,
    };
    use crate::ventures::VenturePartner;

    const TRIP_A: i64 = 201;
    const TRIP_B: i64 = 202;
    const TRIP_C: i64 = 203;

    fn partner(id: i64, account: i64, bps: i32) -> TripPartnerFacts {
        TripPartnerFacts {
            partner: VenturePartner {
                partner_id: PartnerId::new(id),
                account_id: AccountId::new(account),
                share: Bps::new(bps).unwrap(),
                source: MoneySource::Shop,
            },
            name_ur: format!("شریک {id}"),
            name_en: format!("Partner {id}"),
        }
    }

    /// A trip of three partners at 40 / 30 / 30, and a purchase in it.
    fn purchase(item_id: i64, qty_left: i64, cost_left: i64) -> PurchaseFacts {
        PurchaseFacts {
            number: 1,
            entry_date: "2026-10-10".into(),
            trip: TripRef {
                id: 1,
                number: 1,
                code: "0070".into(),
                name_ur: "0070".into(),
                name_en: "0070".into(),
            },
            partners: vec![
                partner(1, TRIP_A, 4_000),
                partner(2, TRIP_B, 3_000),
                partner(3, TRIP_C, 3_000),
            ],
            item_id,
            packing_id: BOX,
            qty_left,
            cost_left: p(cost_left),
            cancelled: false,
        }
    }

    fn with_purchase(facts: PurchaseFacts) -> Lookups {
        let mut lookups = lookups();
        lookups.purchases.insert(1, facts);
        lookups
    }

    /// A lot of own goods taking `qty` from purchase 1: no charges, no نقد, no advance.
    fn own_lot(item: i64, qty: &str) -> LotInput {
        LotInput {
            purchase_id: Some(1),
            adjust_advance: false,
            ..bare_lot(OWN_GOODS, item, qty)
        }
    }

    fn parts_of(calc: &PostingCalc, lot: usize) -> Vec<i64> {
        calc.lots[lot]
            .own
            .as_ref()
            .unwrap()
            .credits
            .iter()
            .map(|c| r(c.amount_paisa))
            .collect()
    }

    /// E2E-34 through `calc_posting`: 15 of 20 نگ at 600. The credit 8,460 goes 3,384 / 2,538 / 2,538
    /// to the partners, the cost 7,800 is shown, the result is 660, and the entry is 9,660.
    #[test]
    fn e2e_34_the_credit_goes_to_the_trip_partners_and_the_cost_is_shown_only() {
        let d = draft(vec![with(
            own_lot(BANANA, "15"),
            vec![sale(AKRAM, "15", "600")],
        )]);
        let calc = run_with(
            &d,
            &with_purchase(purchase(BANANA, 20, 10_400)),
            default_rules(),
            switches(),
        );
        assert!(calc.errors.is_empty(), "{:?}", calc.errors);
        let t = single_lot_totals(&calc);
        assert_eq!(
            (
                r(t.gross),
                r(t.seller_comm),
                r(t.buyer_comm),
                r(t.laga),
                r(t.buyer_total)
            ),
            (9_000, 540, 540, 120, 9_660)
        );
        assert_eq!((r(t.seller_credit), r(t.shared_income)), (8_460, 1_200));
        let own = calc.lots[0].own.as_ref().unwrap();
        assert_eq!(
            (r(own.cost_paisa), r(own.credit_paisa), r(own.result_paisa)),
            (7_800, 8_460, 660)
        );
        assert_eq!(parts_of(&calc, 0), vec![3_384, 2_538, 2_538]);
        assert_eq!(r(line(&calc, 0, 0).cost_of_goods), 7_800);
        // The same parts, to the same accounts, are what the journal gets.
        let credits = &calc.journal.lots[0].partner_credits;
        assert_eq!(
            credits
                .iter()
                .map(|(a, m)| (a.get(), r(*m)))
                .collect::<Vec<_>>(),
            vec![(TRIP_A, 3_384), (TRIP_B, 2_538), (TRIP_C, 2_538)]
        );
        let entry = entry_of(&calc);
        assert_eq!(r(entry.total()), 9_660, "9,660 = 540 + 540 + 120 + 8,460");
        assert_every_working_matches(&calc);
        assert_result("own cost", own.cost_paisa, &own.cost_working);
        assert_result("own result", own.result_paisa, &own.result_working);
    }

    /// A loss: 1 x 10 with labour 50 typed. The credit is -41, split 17 / 12 / 12 and posted on the
    /// debit side of the partners. The lot net is not "the seller's debt".
    #[test]
    fn a_negative_credit_is_split_as_positive_and_negated() {
        let mut lot = own_lot(BANANA, "1");
        lot.labour = manual("50");
        let d = draft(vec![with(lot, vec![sale(AKRAM, "1", "10")])]);
        let calc = run_with(
            &d,
            &with_purchase(purchase(BANANA, 2, 10)),
            default_rules(),
            switches(),
        );
        assert!(calc.errors.is_empty(), "{:?}", calc.errors);
        assert!(
            !has(
                &calc.warnings,
                ErrorCode::ErrLotNetNegative,
                "lots[0].seller_account_id"
            ),
            "the loss of trip goods is not the seller's debt"
        );
        assert_eq!(parts_of(&calc, 0), vec![-17, -12, -12]);
        let own = calc.lots[0].own.as_ref().unwrap();
        // The cost of 1 of 2 نگ is 5, so the result is -41 - 5 = -46.
        assert_eq!((r(own.cost_paisa), r(own.result_paisa)), (5, -46));
        let entry = entry_of(&calc);
        let debit_of = |account: i64| -> i64 {
            entry
                .lines()
                .iter()
                .filter(|l| {
                    l.account_id == AccountId::new(account) && l.side == crate::ledger::Side::Dr
                })
                .map(|l| r(l.amount))
                .sum()
        };
        assert_eq!(
            (debit_of(TRIP_A), debit_of(TRIP_B), debit_of(TRIP_C)),
            (17, 12, 12)
        );
    }

    fn stored_identity() -> StoredIdentity {
        StoredIdentity {
            arrival_id: None,
            seller_account_id: OWN_GOODS,
            item_id: BANANA,
            person_key: None,
            purchase_id: Some(1),
        }
    }

    /// The first posting of E2E-34 extended: line a sold 15 (cost 7,800), line b sold 5 (cost 2,600),
    /// both stored in lot 1 of a posting that took the whole purchase.
    fn stored_two_lines() -> StoredPosting {
        let mut a = stored_sale(
            AKRAM,
            15,
            600,
            percent(540, 600, 1),
            percent(540, 600, 2),
            per_unit_charge(120, 8),
        );
        a.cost = p(7_800);
        let mut b = stored_sale(
            ARSHAD,
            5,
            600,
            percent(180, 600, 1),
            percent(180, 600, 2),
            per_unit_charge(40, 8),
        );
        b.cost = p(2_600);
        StoredPosting {
            kind: PostingKind::First,
            lots: HashMap::from([(
                1,
                StoredLot {
                    identity: Some(stored_identity()),
                    ..StoredLot::default()
                },
            )]),
            sales: HashMap::from([(1, a), (2, b)]),
        }
    }

    fn edit_calc(d: &mandi_core::bikri::BikriDraft, facts: PurchaseFacts) -> PostingCalc {
        let prepared = prepare_draft(d, today(), None);
        calc_posting(
            PostingTarget::Edit { entry_id: 1 },
            &prepared,
            &with_purchase(facts),
            &RuleSet::build(default_rules()),
            switches(),
            &stored_two_lines(),
        )
        .unwrap()
    }

    fn edit_draft(qty_a: &str, qty_b: &str, lot_qty: &str) -> mandi_core::bikri::BikriDraft {
        let mut lot = own_lot(BANANA, lot_qty);
        lot.lot_id = Some(1);
        let mut a = sale(AKRAM, qty_a, "600");
        a.sale_id = Some(1);
        let mut b = sale(ARSHAD, qty_b, "600");
        b.sale_id = Some(2);
        draft(vec![with(lot, vec![a, b])])
    }

    /// An edit keeps the stored cost of a line whose quantity did not change, takes it out of the
    /// stock first, and works the other lines out from what is left (FR-12.20, C21).
    /// Stored: a 15 / 7,800 and b 5 / 2,600 (the whole purchase: 20 / 10,400 once they are added
    /// back). Line b is cut from 5 to 4: it takes split(2,600, [4, 1]) = 2,080, and 1 نگ stays.
    #[test]
    fn an_edit_keeps_an_unchanged_line_and_works_the_changed_one_from_what_is_left() {
        let calc = edit_calc(&edit_draft("15", "4", "19"), purchase(BANANA, 20, 10_400));
        assert!(calc.errors.is_empty(), "{:?}", calc.errors);
        assert_eq!(r(line(&calc, 0, 0).cost_of_goods), 7_800, "kept");
        assert_eq!(
            line(&calc, 0, 0).workings.cost_of_goods.message_id,
            "working-cost-of-goods-kept"
        );
        assert_eq!(r(line(&calc, 0, 1).cost_of_goods), 2_080);
        assert_eq!(
            line(&calc, 0, 1).workings.cost_of_goods.message_id,
            "working-cost-of-goods"
        );
        let own = calc.lots[0].own.as_ref().unwrap();
        assert_eq!(r(own.cost_paisa), 9_880);
        // Gross 9,000 + 2,400, seller commission 540 + 144: credit 10,716; result 10,716 - 9,880.
        assert_eq!((r(own.credit_paisa), r(own.result_paisa)), (10_716, 836));
        let parts = parts_of(&calc, 0);
        assert_eq!(
            parts.iter().sum::<i64>(),
            10_716,
            "the parts add up to the credit"
        );
        assert_eq!(
            parts,
            vec![4_288, 3_214, 3_214],
            "the leftover rupees go to the largest share"
        );
        entry_of(&calc);
    }

    /// Both lines unchanged: nothing is worked out again and the whole stored cost comes back.
    #[test]
    fn an_edit_with_both_lines_unchanged_gives_back_the_stored_costs() {
        let calc = edit_calc(&edit_draft("15", "5", "20"), purchase(BANANA, 20, 10_400));
        assert!(calc.errors.is_empty(), "{:?}", calc.errors);
        assert_eq!(
            (
                r(line(&calc, 0, 0).cost_of_goods),
                r(line(&calc, 0, 1).cost_of_goods)
            ),
            (7_800, 2_600)
        );
        assert_eq!(r(calc.lots[0].own.as_ref().unwrap().cost_paisa), 10_400);
    }

    /// A line sent twice with the same id is a mistake the edit reports; the domain must not call it
    /// an internal error (money review, finding 4). The second copy takes no stock again.
    #[test]
    fn a_line_id_sent_twice_is_not_an_internal_error() {
        let mut lot = own_lot(BANANA, "20");
        lot.lot_id = Some(1);
        let mut a = sale(AKRAM, "15", "600");
        a.sale_id = Some(1);
        let again = a.clone();
        let d = draft(vec![with(lot, vec![a, again])]);
        let prepared = prepare_draft(&d, today(), None);
        let result = calc_posting(
            PostingTarget::Edit { entry_id: 1 },
            &prepared,
            &with_purchase(purchase(BANANA, 20, 10_400)),
            &RuleSet::build(default_rules()),
            switches(),
            &stored_two_lines(),
        );
        let calc = result.unwrap();
        // The first copy keeps the stored cost; the second is worked out from what is left (5 نگ),
        // so it is refused for 15 نگ and shows no stored cost (money review, finding 4).
        assert_eq!(r(line(&calc, 0, 0).cost_of_goods), 7_800);
        assert!(has(
            &calc.errors,
            ErrorCode::ErrQtyExceedsLot,
            "lots[0].sales[1].qty"
        ));
    }

    /// An own lot dated before its purchase is refused, as a sale in another shop is (money review,
    /// finding 1; PRD FR-12.24, FR-12.26, FR-12.32). The purchase was bought on 11-10-26; the
    /// posting is on 10-10-26.
    #[test]
    fn an_own_lot_cannot_be_dated_before_its_purchase() {
        let d = draft(vec![with(
            own_lot(BANANA, "15"),
            vec![sale(AKRAM, "15", "600")],
        )]);
        let mut facts = purchase(BANANA, 20, 10_400);
        facts.entry_date = "2026-10-11".into();
        let calc = run_with(&d, &with_purchase(facts), default_rules(), switches());
        assert!(has(
            &calc.errors,
            ErrorCode::ErrDateBeforeReceived,
            "lots[0].purchase_id"
        ));
        let error = calc
            .errors
            .iter()
            .find(|e| e.code == ErrorCode::ErrDateBeforeReceived)
            .unwrap();
        assert_eq!(
            error.values.get("received_date"),
            Some(&ErrorValue::Text("2026-10-11".into()))
        );
        assert!(
            calc.lots[0].own.is_none(),
            "no numbers for a lot that cannot be saved"
        );
    }

    /// A line that cannot be worked out takes no stock, so the next line is shown with the stock it
    /// would really have (money review, finding 5). Purchase of 3 نگ of a weight item, cost 1,001.
    /// Line 1 has no weight (a mistake). Line 2 takes split(1,001, [1, 2]) = 333, not 334.
    #[test]
    fn a_line_with_a_mistake_gives_its_stock_back_to_the_next_line() {
        let mut lot = own_lot(GRAPES, "2");
        lot.weight_received = Some("3".into());
        let mut missing = sale(AKRAM, "1", "100");
        missing.weight = None;
        let mut fine = sale(ARSHAD, "1", "100");
        fine.weight = Some("1.5".into());
        let d = draft(vec![with(lot, vec![missing, fine])]);
        let calc = run_with(
            &d,
            &with_purchase(purchase(GRAPES, 3, 1_001)),
            default_rules(),
            switches(),
        );
        assert!(has(
            &calc.errors,
            ErrorCode::ErrWeightRequired,
            "lots[0].sales[0].weight"
        ));
        assert_eq!(r(line(&calc, 0, 1).cost_of_goods), 333);
    }
}
