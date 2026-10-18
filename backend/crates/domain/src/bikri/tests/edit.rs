//! `calc_posting` for an edit and a later sale (plan 16, PRD FR-12.17, FR-12.20, FR-11.6,
//! FR-11.9): stored lines, the per-lot commission that moves, the quantity left, and no نقد in a
//! later sale. The stored rows are built by hand here, as the services read them.

use std::collections::HashMap;

use mandi_core::bikri::{
    LineChargeInput, LineChargeMethod, PaymentType, StoredMethod, prepare_draft,
};
use mandi_core::error::ErrorCode;
use mandi_core::rules::RuleLevel;

use super::*;
use crate::bikri::{
    PostingKind, StoredCarrier, StoredCharge, StoredIdentity, StoredLot, StoredSale,
};

fn percent(amount: i64, bps: i32, rule_id: i64) -> StoredCharge {
    StoredCharge {
        method: StoredMethod::Percent,
        value_bps: Some(bps),
        value_paisa: None,
        amount: p(amount),
        rule_id: Some(rule_id),
        level: Some(RuleLevel::Default),
    }
}

fn per_unit_charge(amount: i64, value: i64) -> StoredCharge {
    StoredCharge {
        method: StoredMethod::PerUnit,
        value_bps: None,
        value_paisa: Some(p(value)),
        amount: p(amount),
        rule_id: Some(3),
        level: Some(RuleLevel::Default),
    }
}

fn per_lot_charge(amount: i64, value: i64) -> StoredCharge {
    StoredCharge {
        method: StoredMethod::PerLot,
        value_bps: None,
        value_paisa: Some(p(value)),
        amount: p(amount),
        rule_id: Some(10),
        level: Some(RuleLevel::Party),
    }
}

fn stored_sale(
    buyer: i64,
    qty: i64,
    rate: i64,
    seller: StoredCharge,
    buyer_comm: StoredCharge,
    laga: StoredCharge,
) -> StoredSale {
    StoredSale {
        lot_id: 1,
        buyer_account_id: buyer,
        person_key: None,
        qty,
        weight_grams: None,
        seller_rate: p(rate),
        buyer_rate: p(rate),
        payment_type: PaymentType::Credit,
        seller_comm: seller,
        buyer_comm,
        laga,
    }
}

/// Event 1 as saved with the defaults of PRD 4.2.
fn stored_event1() -> StoredPosting {
    StoredPosting {
        kind: PostingKind::First,
        lots: HashMap::from([(1, StoredLot::default())]),
        sales: HashMap::from([
            (
                1,
                stored_sale(
                    AKRAM,
                    20,
                    600,
                    percent(720, 600, 1),
                    percent(720, 600, 2),
                    per_unit_charge(160, 8),
                ),
            ),
            (
                2,
                stored_sale(
                    ARSHAD,
                    30,
                    580,
                    percent(1_044, 600, 1),
                    percent(1_044, 600, 2),
                    per_unit_charge(240, 8),
                ),
            ),
        ]),
    }
}

/// Event 1 as the screen gives it back for an edit: the lot and the lines carry their ids.
fn event1_with_ids() -> mandi_core::bikri::BikriDraft {
    let mut d = event1();
    d.lots[0].lot_id = Some(1);
    d.lots[0].sales[0].sale_id = Some(1);
    d.lots[0].sales[1].sale_id = Some(2);
    d
}

fn edit_with(
    draft: &mandi_core::bikri::BikriDraft,
    rules: Vec<Rule>,
    stored: &StoredPosting,
) -> PostingCalc {
    let prepared = prepare_draft(draft, today(), None);
    calc_posting(
        PostingTarget::Edit { entry_id: 1 },
        &prepared,
        &lookups(),
        &RuleSet::build(rules),
        switches(),
        stored,
    )
    .unwrap()
}

/// The defaults with the buyer commission changed to 7%.
fn rules_with_buyer_7() -> Vec<Rule> {
    let mut rules = default_rules();
    rules[1] = rule(
        2,
        ChargeSide::Buyer,
        ChargeType::Commission,
        ChargeValue::Percent(Bps::new(700).unwrap()),
    );
    rules
}

/// The defaults and a per-lot seller commission of 600 for رحیم.
fn rules_per_lot_600() -> Vec<Rule> {
    let mut rules = default_rules();
    rules.push(Rule {
        account_id: Some(RAHIM),
        ..rule(
            10,
            ChargeSide::Seller,
            ChargeType::Commission,
            ChargeValue::PerLot(p(600)),
        )
    });
    rules
}

#[test]
fn an_unchanged_edit_gives_back_event_1_exactly() {
    let calc = edit_with(&event1_with_ids(), default_rules(), &stored_event1());
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).buyer_total), 12_880);
    assert_eq!(r(line(&calc, 0, 1).buyer_total), 18_684);
    let t = single_lot_totals(&calc);
    assert_eq!(
        (r(t.gross), r(t.deductions), r(t.net), r(t.shared_income)),
        (29_400, 10_084, 19_316, 3_948)
    );
    assert_every_working_matches(&calc);
}

#[test]
fn an_unchanged_line_keeps_its_stored_rate_and_a_changed_line_looks_the_rules_up_again() {
    // PRD E2E-13b: the default buyer commission is now 7%. اکرم's rate changes to 610, so his line
    // is worked out again with 7%; ارشد's line is not changed and keeps its stored 6%.
    let mut d = event1_with_ids();
    d.lots[0].sales[0].seller_rate = "610".into();
    let calc = edit_with(&d, rules_with_buyer_7(), &stored_event1());
    assert_no_errors(&calc);
    let changed = line(&calc, 0, 0);
    assert_eq!(r(changed.buyer_comm.amount), 854);
    assert_eq!(r(changed.seller_comm.amount), 732);
    assert_eq!(r(changed.buyer_total), 13_214);
    let kept = line(&calc, 0, 1);
    assert_eq!(r(kept.buyer_comm.amount), 1_044);
    assert_eq!(r(kept.buyer_total), 18_684);
    // Shared income 1,776 + 1,898 + 400 + 20 = 4,094 (PRD E2E-13b).
    assert_eq!(r(single_lot_totals(&calc).shared_income), 4_094);
    assert_every_working_matches(&calc);
}

#[test]
fn a_line_without_its_stored_id_is_a_new_line() {
    // The same draft without line ids: both lines look the rules up again, so both take 7%.
    let mut d = event1();
    d.lots[0].lot_id = Some(1);
    let calc = edit_with(&d, rules_with_buyer_7(), &stored_event1());
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).buyer_comm.amount), 840);
    assert_eq!(r(line(&calc, 0, 1).buyer_comm.amount), 1_218);
}

fn per_lot_stored() -> StoredPosting {
    let mut stored = stored_event1();
    stored.lots.insert(
        1,
        StoredLot {
            carrier: Some(StoredCarrier {
                sale_id: 1,
                charge: per_lot_charge(500, 500),
            }),
            ..StoredLot::default()
        },
    );
    if let Some(s) = stored.sales.get_mut(&1) {
        s.seller_comm = per_lot_charge(500, 500);
    }
    if let Some(s) = stored.sales.get_mut(&2) {
        s.seller_comm = per_lot_charge(0, 500);
    }
    stored
}

#[test]
fn the_per_lot_commission_stays_on_its_line_while_that_line_is_in_the_draft() {
    let calc = edit_with(&event1_with_ids(), rules_per_lot_600(), &per_lot_stored());
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).seller_comm.amount), 500);
    assert_eq!(r(line(&calc, 0, 1).seller_comm.amount), 0);
    assert_eq!(r(single_lot_totals(&calc).seller_comm), 500);
}

#[test]
fn the_per_lot_commission_moves_with_its_stored_value_when_its_line_is_removed() {
    // PRD FR-11.9: the rule now says 600, but the charge that moves is the stored 500.
    let mut d = event1_with_ids();
    d.lots[0].sales.remove(0);
    let calc = edit_with(&d, rules_per_lot_600(), &per_lot_stored());
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).seller_comm.amount), 500);
    assert_eq!(r(single_lot_totals(&calc).seller_comm), 500);
}

#[test]
fn a_typed_commission_on_the_line_left_gets_no_moved_charge() {
    let mut d = event1_with_ids();
    d.lots[0].sales.remove(0);
    d.lots[0].sales[0].seller_comm = Some(LineChargeInput {
        method: LineChargeMethod::Percent,
        value: "4".into(),
    });
    let calc = edit_with(&d, rules_per_lot_600(), &per_lot_stored());
    assert_no_errors(&calc);
    // 30 × 580 = 17,400 × 4% = 696 and nothing else.
    assert_eq!(r(line(&calc, 0, 0).seller_comm.amount), 696);
    assert_eq!(r(single_lot_totals(&calc).seller_comm), 696);
}

#[test]
fn a_changed_first_line_charges_the_per_lot_rule_again_and_the_other_line_stays_at_zero() {
    let mut d = event1_with_ids();
    d.lots[0].sales[0].seller_rate = "610".into();
    let calc = edit_with(&d, rules_per_lot_600(), &per_lot_stored());
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).seller_comm.amount), 600);
    assert_eq!(r(line(&calc, 0, 1).seller_comm.amount), 0);
    assert_eq!(r(single_lot_totals(&calc).seller_comm), 600);
}

#[test]
fn the_quantity_other_postings_took_is_not_available_to_an_edit() {
    // 50 received; a later sale took 30 elsewhere. The edited posting may sell 20 at most.
    let mut stored = stored_event1();
    stored.lots.insert(
        1,
        StoredLot {
            elsewhere_qty: 30,
            ..StoredLot::default()
        },
    );
    let calc = edit_with(&event1_with_ids(), default_rules(), &stored);
    assert!(
        has(
            &calc.errors,
            ErrorCode::ErrQtyExceedsLot,
            "lots[0].sales[1].qty"
        ),
        "{:?}",
        calc.errors
    );
    // Receiving less than what others took is its own mistake.
    let mut d = event1_with_ids();
    d.lots[0].qty_received = "20".into();
    let calc = edit_with(&d, default_rules(), &stored);
    assert!(
        has(
            &calc.errors,
            ErrorCode::ErrQtyBelowSold,
            "lots[0].qty_received"
        ),
        "{:?}",
        calc.errors
    );
}

#[test]
fn a_later_posting_refuses_a_typed_cash_and_charges_nothing_else_of_the_lot() {
    let mut l = lot(RAHIM, BANANA, "50");
    l.lot_id = Some(1);
    l.sales = vec![sale(AKRAM, "5", "600")];
    let d = draft(vec![l]);
    let prepared = prepare_draft(&d, today(), None);
    let stored = StoredPosting {
        kind: PostingKind::Later,
        lots: HashMap::from([(
            1,
            StoredLot {
                elsewhere_qty: 45,
                ..StoredLot::default()
            },
        )]),
        sales: HashMap::new(),
    };
    let calc = calc_posting(
        PostingTarget::LaterSale { lot_id: 1 },
        &prepared,
        &lookups(),
        &RuleSet::build(default_rules()),
        switches(),
        &stored,
    )
    .unwrap();
    assert!(
        has(
            &calc.errors,
            ErrorCode::ErrFormatInvalid,
            "lots[0].cash_paid"
        ),
        "{:?}",
        calc.errors
    );
    // With no cash typed, the lot charges are 0 and only 5 are left to sell.
    let mut l = lot(RAHIM, BANANA, "50");
    l.lot_id = Some(1);
    l.cash_paid = None;
    l.sales = vec![sale(AKRAM, "6", "600")];
    let prepared = prepare_draft(&draft(vec![l]), today(), None);
    let calc = calc_posting(
        PostingTarget::LaterSale { lot_id: 1 },
        &prepared,
        &lookups(),
        &RuleSet::build(default_rules()),
        switches(),
        &stored,
    )
    .unwrap();
    assert!(
        has(
            &calc.errors,
            ErrorCode::ErrQtyExceedsLot,
            "lots[0].sales[0].qty"
        ),
        "{:?}",
        calc.errors
    );
    let mut l = lot(RAHIM, BANANA, "50");
    l.lot_id = Some(1);
    l.cash_paid = None;
    l.sales = vec![sale(AKRAM, "5", "560")];
    let prepared = prepare_draft(&draft(vec![l]), today(), None);
    let calc = calc_posting(
        PostingTarget::LaterSale { lot_id: 1 },
        &prepared,
        &lookups(),
        &RuleSet::build(default_rules()),
        switches(),
        &stored,
    )
    .unwrap();
    assert_no_errors(&calc);
    let t = single_lot_totals(&calc);
    // PRD E2E-16b on akram's 5 at 560: 2,800 − 168 = 2,632 to the seller, nothing else charged.
    assert_eq!(
        (
            r(t.gross),
            r(t.seller_comm),
            r(t.seller_credit),
            r(t.cash_paid),
            r(t.freight)
        ),
        (2_800, 168, 2_632, 0, 0)
    );
}

#[test]
fn one_time_seller_cash_is_not_defaulted_in_a_later_posting() {
    // PRD FR-13.10 pays a one-time seller his whole net on the spot at the FIRST posting only.
    let mut l = bare_lot(PUKHTA_SELLER, BANANA, "10");
    l.lot_id = Some(1);
    l.pukhta = Some(mandi_core::bikri::PukhtaInput {
        name_ur: "ناصر".into(),
        name_en: "Nasir".into(),
        phone: "0301-1111111".into(),
    });
    l.sales = vec![sale(AKRAM, "5", "600")];
    let prepared = prepare_draft(&draft(vec![l]), today(), None);
    let stored = StoredPosting {
        kind: PostingKind::Later,
        lots: HashMap::from([(
            1,
            StoredLot {
                elsewhere_qty: 5,
                ..StoredLot::default()
            },
        )]),
        sales: HashMap::new(),
    };
    let calc = calc_posting(
        PostingTarget::LaterSale { lot_id: 1 },
        &prepared,
        &lookups(),
        &RuleSet::build(default_rules()),
        switches(),
        &stored,
    )
    .unwrap();
    assert_no_errors(&calc);
    assert_eq!(r(single_lot_totals(&calc).cash_paid), 0);
}

#[test]
fn a_lot_whose_seller_or_item_changed_keeps_nothing_stored_for_the_old_one() {
    // Money agent review of plan 16. The lines were saved with a seller commission of 5% (rule 7);
    // the default now says 6%. With the same seller the lines keep 5%; with another seller or
    // another item they are worked out again.
    let mut stored = stored_event1();
    for (id, amount) in [(1, 600), (2, 870)] {
        if let Some(s) = stored.sales.get_mut(&id) {
            s.seller_comm = percent(amount, 500, 7);
        }
    }
    stored.lots.insert(
        1,
        StoredLot {
            identity: Some(StoredIdentity {
                seller_account_id: RAHIM,
                item_id: BANANA,
                person_key: None,
            }),
            ..StoredLot::default()
        },
    );
    let same = edit_with(&event1_with_ids(), default_rules(), &stored);
    assert_no_errors(&same);
    assert_eq!(r(single_lot_totals(&same).seller_comm), 1_470);
    // Another seller.
    let mut d = event1_with_ids();
    d.lots[0].seller_account_id = Some(KARIM);
    let calc = edit_with(&d, default_rules(), &stored);
    assert_no_errors(&calc);
    assert_eq!(r(single_lot_totals(&calc).seller_comm), 1_764);
    // Another item: a vegetable has no لاگا, so the stored 8 per نگ goes too.
    let mut d = event1_with_ids();
    d.lots[0].item_id = Some(MOOLI);
    let calc = edit_with(&d, default_rules(), &stored);
    assert_no_errors(&calc);
    assert_eq!(r(single_lot_totals(&calc).laga), 0);
}

#[test]
fn leftover_one_time_name_text_on_an_ordinary_sellers_lot_is_not_a_change() {
    // Money agent second pass, MINOR-2: only a one-time seller has a person key to compare.
    let mut stored = stored_event1();
    stored.lots.insert(
        1,
        StoredLot {
            identity: Some(StoredIdentity {
                seller_account_id: RAHIM,
                item_id: BANANA,
                person_key: None,
            }),
            ..StoredLot::default()
        },
    );
    let mut d = event1_with_ids();
    d.lots[0].pukhta = Some(mandi_core::bikri::PukhtaInput {
        name_ur: "ناصر".into(),
        name_en: "Nasir".into(),
        phone: "0301-1111111".into(),
    });
    // The default buyer commission is 7% now, but the unchanged lines keep their stored 6%.
    let calc = edit_with(&d, rules_with_buyer_7(), &stored);
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).buyer_comm.amount), 720);
    assert_eq!(r(line(&calc, 0, 1).buyer_comm.amount), 1_044);
}
