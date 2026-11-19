//! `calc_posting` for an edit and a later sale (plan 16, PRD FR-12.17, FR-12.20, FR-11.6,
//! FR-11.9): stored lines, the per-lot commission that moves, the quantity left, and no نقد in a
//! later sale. The stored rows are built by hand here, as the services read them.

use std::collections::HashMap;

use mandi_core::bikri::{LineChargeInput, LineChargeMethod, StoredMethod, prepare_draft};
use mandi_core::error::ErrorCode;
use mandi_core::rules::RuleLevel;

use super::*;
use crate::bikri::{
    PostingKind, StoredCarrier, StoredCharge, StoredIdentity, StoredLot, StoredSale,
};

pub(super) fn percent(amount: i64, bps: i32, rule_id: i64) -> StoredCharge {
    StoredCharge {
        method: StoredMethod::Percent,
        value_bps: Some(bps),
        value_paisa: None,
        amount: p(amount),
        rule_id: Some(rule_id),
        level: Some(RuleLevel::Default),
    }
}

pub(super) fn per_unit_charge(amount: i64, value: i64) -> StoredCharge {
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

pub(super) fn stored_sale(
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
        paid: Paisa::ZERO,
        seller_comm: seller,
        buyer_comm,
        laga,
        cost: Paisa::ZERO,
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
    edit_with_lookups(draft, rules, stored, &lookups())
}

fn edit_with_lookups(
    draft: &mandi_core::bikri::BikriDraft,
    rules: Vec<Rule>,
    stored: &StoredPosting,
    lookups: &Lookups,
) -> PostingCalc {
    let prepared = prepare_draft(draft, today(), None);
    calc_posting(
        PostingTarget::Edit { entry_id: 1 },
        &prepared,
        lookups,
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
fn a_quality_only_change_leaves_the_line_unchanged_and_keeps_its_stored_rules() {
    // PRD FR-12.42, FR-12.20: the quality is not in the comparison of `kept_stored`. With the
    // default buyer commission now 7%, a line whose only change is its quality keeps its stored
    // 6% and every stored amount, and the totals are those of event 1.
    let mut d = event1_with_ids();
    d.lots[0].sales[0].grade = Some("first".into());
    d.lots[0].sales[1].grade = Some("second".into());
    let calc = edit_with(&d, rules_with_buyer_7(), &stored_event1());
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).buyer_comm.amount), 720);
    assert_eq!(r(line(&calc, 0, 0).buyer_total), 12_880);
    assert_eq!(r(line(&calc, 0, 1).buyer_comm.amount), 1_044);
    assert_eq!(r(line(&calc, 0, 1).buyer_total), 18_684);
    assert_eq!(r(single_lot_totals(&calc).shared_income), 3_948);
    // A word that is not a quality is refused at its field and the row is not worked out.
    let mut bad = event1_with_ids();
    bad.lots[0].sales[0].grade = Some("fourth".into());
    let calc = edit_with(&bad, rules_with_buyer_7(), &stored_event1());
    assert!(has(
        &calc.errors,
        ErrorCode::ErrOutOfRange,
        "lots[0].sales[0].grade"
    ));
    assert!(calc.lots[0].lines[0].calc.is_none());
}

#[test]
fn a_paid_only_change_keeps_the_stored_rules_and_amounts() {
    // PRD FR-12.44, FR-12.20, plan 60 P11 (E2E-13b style): the default buyer commission is now 7%.
    // Only ارشد's paid on the spot changes, so his line keeps its stored 6% and every stored amount;
    // only the cash part moves.
    let mut d = event1_with_ids();
    d.lots[0].sales[1].paid = Some("10,000".into());
    let calc = edit_with(&d, rules_with_buyer_7(), &stored_event1());
    assert_no_errors(&calc);
    let l = line(&calc, 0, 1);
    assert_eq!((r(l.buyer_comm.amount), r(l.buyer_total)), (1_044, 18_684));
    assert_eq!((r(l.paid_paisa), r(l.left_paisa)), (10_000, 8_684));
    assert_eq!(r(line(&calc, 0, 0).buyer_total), 12_880);
    // The shared income and the lot totals are those of event 1.
    let t = single_lot_totals(&calc);
    assert_eq!(r(t.shared_income), 3_948);
    assert_eq!((r(t.buyer_cash), r(t.buyer_credit)), (10_000, 21_564));
    // A paid above the stored total is refused, and the line keeps its figures.
    let mut over = event1_with_ids();
    over.lots[0].sales[1].paid = Some("18,685".into());
    let calc = edit_with(&over, rules_with_buyer_7(), &stored_event1());
    assert!(has(
        &calc.errors,
        ErrorCode::ErrPaidExceedsTotal,
        "lots[0].sales[1].paid"
    ));
    assert_eq!(r(line(&calc, 0, 1).buyer_total), 18_684);
}

/// A posting whose one line is for a buyer who must pay in full (credit off), saved with `paid`.
fn stored_must_pay_line(paid: i64) -> StoredPosting {
    let mut stored = stored_event1();
    let mut s = stored_sale(
        CASH_ONLY_BUYER,
        20,
        600,
        percent(720, 600, 1),
        percent(720, 600, 2),
        per_unit_charge(160, 8),
    );
    s.paid = p(paid);
    stored.sales = HashMap::from([(1, s)]);
    stored
}

#[test]
fn an_unchanged_line_of_a_must_pay_buyer_keeps_the_paid_it_was_saved_with() {
    // Plan 65 point 10a: the line was saved as credit (paid 0) when the buyer could take credit,
    // and he has since lost it. An edit that leaves the line alone must not turn it into cash.
    let draft_of = |rate: &str, id: Option<i64>| {
        let mut row = sale(CASH_ONLY_BUYER, "20", rate);
        row.sale_id = id;
        let mut d = draft(vec![with(lot(RAHIM, BANANA, "50"), vec![row])]);
        d.lots[0].lot_id = Some(1);
        d
    };
    let calc = edit_with(
        &draft_of("600", Some(1)),
        default_rules(),
        &stored_must_pay_line(0),
    );
    assert_no_errors(&calc);
    let l = line(&calc, 0, 0);
    assert_eq!(
        (r(l.paid_paisa), r(l.left_paisa), l.paid_fixed),
        (0, 12_880, true)
    );
    assert_every_working_matches(&calc);
    // Saved paid in full: it stays paid in full.
    let calc = edit_with(
        &draft_of("600", Some(1)),
        default_rules(),
        &stored_must_pay_line(12_880),
    );
    assert_eq!(
        (
            r(line(&calc, 0, 0).paid_paisa),
            r(line(&calc, 0, 0).left_paisa)
        ),
        (12_880, 0)
    );
    // A changed line (the rate) follows the rule of a buyer who must pay in full: paid = total.
    let calc = edit_with(
        &draft_of("610", Some(1)),
        default_rules(),
        &stored_must_pay_line(0),
    );
    assert_eq!(
        r(line(&calc, 0, 0).paid_paisa),
        r(line(&calc, 0, 0).buyer_total)
    );
    assert_eq!(r(line(&calc, 0, 0).left_paisa), 0);
    // A new line (no stored id) also pays in full.
    let calc = edit_with(
        &draft_of("600", None),
        default_rules(),
        &stored_must_pay_line(0),
    );
    assert_eq!(r(line(&calc, 0, 0).paid_paisa), 12_880);
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
                arrival_id: None,
                seller_account_id: RAHIM,
                item_id: BANANA,
                purchase_id: None,
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

// ---- Plan 67: an edit that changes only the buyer, or only the seller (PRD FR-12.20) ------------

/// Event 1 as saved, with the lot's seller and item stored, so a change of the seller is seen.
fn stored_event1_with_identity() -> StoredPosting {
    let mut stored = stored_event1();
    stored.lots.insert(
        1,
        StoredLot {
            identity: Some(StoredIdentity {
                arrival_id: None,
                seller_account_id: RAHIM,
                item_id: BANANA,
                purchase_id: None,
            }),
            ..StoredLot::default()
        },
    );
    stored
}

/// The defaults after the Owner raised both commissions to 7% (the lines were saved at 6%).
fn rules_with_both_7() -> Vec<Rule> {
    let mut rules = rules_with_buyer_7();
    rules[0] = rule(
        1,
        ChargeSide::Seller,
        ChargeType::Commission,
        ChargeValue::Percent(Bps::new(700).unwrap()),
    );
    rules
}

/// The seller and buyer commission and the buyer total of one line, in rupees.
fn money_of(calc: &PostingCalc, lot: usize, index: usize) -> (i64, i64, i64) {
    let l = line(calc, lot, index);
    (
        r(l.seller_comm.amount),
        r(l.buyer_comm.amount),
        r(l.buyer_total),
    )
}

#[test]
fn a_seller_only_change_rereads_the_seller_side_and_keeps_the_buyer_side_as_stored() {
    // Umar, 10-11-2026 (FR-12.20): the buyer default is 7% now and only the seller of the lot is
    // corrected. اکرم stays 12,880 (his stored 6%); the seller commission follows the rules, 7%.
    let mut d = event1_with_ids();
    d.lots[0].seller_account_id = Some(KARIM);
    let calc = edit_with(&d, rules_with_both_7(), &stored_event1_with_identity());
    assert_no_errors(&calc);
    // 12,000 × 7% = 840 and 17,400 × 7% = 1,218 for the seller; the buyer side is as stored.
    assert_eq!(money_of(&calc, 0, 0), (840, 720, 12_880));
    assert_eq!(money_of(&calc, 0, 1), (1_218, 1_044, 18_684));
    assert_eq!(r(line(&calc, 0, 0).laga.amount), 160);
    assert_every_working_matches(&calc);
}

#[test]
fn a_buyer_only_change_rereads_the_buyer_side_and_keeps_the_seller_side_as_stored() {
    // Umar, 10-11-2026 (FR-12.20): the seller default is 7% now and only the buyer of line 1 is
    // corrected (اکرم to ارشد). The seller commission of that line stays 720 (stored 6%); the buyer
    // side follows the rules, 7%. The other line is not touched.
    let mut d = event1_with_ids();
    d.lots[0].sales[0].buyer_account_id = Some(ARSHAD);
    let calc = edit_with(&d, rules_with_both_7(), &stored_event1_with_identity());
    assert_no_errors(&calc);
    // Buyer commission 12,000 × 7% = 840; the buyer pays 12,000 + 840 + 160 = 13,000.
    assert_eq!(money_of(&calc, 0, 0), (720, 840, 13_000));
    assert_eq!(money_of(&calc, 0, 1), (1_044, 1_044, 18_684));
    assert_every_working_matches(&calc);
}

#[test]
fn a_change_of_the_item_or_of_both_buyer_and_seller_rereads_every_rule() {
    // The item changed: nothing stored is kept, on either side.
    let mut d = event1_with_ids();
    d.lots[0].item_id = Some(APPLE);
    let calc = edit_with(&d, rules_with_both_7(), &stored_event1_with_identity());
    assert_no_errors(&calc);
    assert_eq!(money_of(&calc, 0, 0), (840, 840, 13_000));
    // 30 × 580 = 17,400; buyer 7% = 1,218; لاگا 240.
    assert_eq!(money_of(&calc, 0, 1), (1_218, 1_218, 18_858));
    // The buyer of a line and the seller of its lot both changed: the line is a changed line.
    let mut d = event1_with_ids();
    d.lots[0].seller_account_id = Some(KARIM);
    d.lots[0].sales[0].buyer_account_id = Some(ARSHAD);
    let calc = edit_with(&d, rules_with_both_7(), &stored_event1_with_identity());
    assert_no_errors(&calc);
    assert_eq!(money_of(&calc, 0, 0), (840, 840, 13_000));
    // The other line has only the seller changed.
    assert_eq!(money_of(&calc, 0, 1), (1_218, 1_044, 18_684));
}

#[test]
fn a_seller_only_change_keeps_the_paid_a_must_pay_buyer_was_saved_with() {
    // Plan 65 point 10a with plan 67: the line was saved as credit (paid 0) and its buyer has since
    // lost credit. Correcting only the seller of the lot must not turn it into cash.
    let mut stored = stored_must_pay_line(0);
    stored.lots = stored_event1_with_identity().lots;
    let mut row = sale(CASH_ONLY_BUYER, "20", "600");
    row.sale_id = Some(1);
    let mut d = draft(vec![with(lot(KARIM, BANANA, "50"), vec![row])]);
    d.lots[0].lot_id = Some(1);
    let calc = edit_with(&d, default_rules(), &stored);
    assert_no_errors(&calc);
    let l = line(&calc, 0, 0);
    assert_eq!(
        (r(l.paid_paisa), r(l.left_paisa), l.paid_fixed),
        (0, 12_880, true)
    );
    // A buyer-only change onto a must-pay buyer does follow the new buyer's rule: paid in full,
    // whatever the old line had paid.
    let mut stored = stored_event1_with_identity();
    if let Some(s) = stored.sales.get_mut(&1) {
        s.paid = p(5_000);
    }
    let mut d = event1_with_ids();
    d.lots[0].sales[0].buyer_account_id = Some(CASH_ONLY_BUYER);
    d.lots[0].sales[0].paid = Some("5,000".into());
    let calc = edit_with(&d, default_rules(), &stored);
    assert_no_errors(&calc);
    let l = line(&calc, 0, 0);
    assert_eq!((r(l.paid_paisa), r(l.left_paisa)), (12_880, 0));
    // Onto a buyer who may take credit, what was typed stands.
    let mut d = event1_with_ids();
    d.lots[0].sales[0].buyer_account_id = Some(ARSHAD);
    d.lots[0].sales[0].paid = Some("5,000".into());
    let calc = edit_with(&d, default_rules(), &stored);
    assert_no_errors(&calc);
    let l = line(&calc, 0, 0);
    assert_eq!((r(l.paid_paisa), r(l.left_paisa)), (5_000, 7_880));
}

#[test]
fn the_per_lot_commission_is_charged_once_after_a_seller_only_or_a_buyer_only_change() {
    // The carrier line (line 1, stored 500) has only its buyer corrected: the seller side is as
    // stored, so it keeps 500 and the other line stays 0 (the rule now says 600).
    let mut stored = per_lot_stored();
    stored.lots = HashMap::from([(
        1,
        StoredLot {
            identity: Some(StoredIdentity {
                arrival_id: None,
                seller_account_id: RAHIM,
                item_id: BANANA,
                purchase_id: None,
            }),
            carrier: Some(StoredCarrier {
                sale_id: 1,
                charge: per_lot_charge(500, 500),
            }),
            ..StoredLot::default()
        },
    )]);
    let mut d = event1_with_ids();
    d.lots[0].sales[0].buyer_account_id = Some(ARSHAD);
    let calc = edit_with(&d, rules_per_lot_600(), &stored);
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).seller_comm.amount), 500);
    assert_eq!(r(line(&calc, 0, 1).seller_comm.amount), 0);
    assert_eq!(r(single_lot_totals(&calc).seller_comm), 500);
    // The other line has only its buyer corrected: still once, still on line 1.
    let mut d = event1_with_ids();
    d.lots[0].sales[1].buyer_account_id = Some(AKRAM);
    let calc = edit_with(&d, rules_per_lot_600(), &stored);
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).seller_comm.amount), 500);
    assert_eq!(r(line(&calc, 0, 1).seller_comm.amount), 0);
    // Only the seller changed, to one with a per-lot rule of 600: the seller side is read again, the
    // first line carries it, once. The old 500 is gone with the old seller.
    let mut rules = default_rules();
    rules.push(Rule {
        account_id: Some(KARIM),
        ..rule(
            11,
            ChargeSide::Seller,
            ChargeType::Commission,
            ChargeValue::PerLot(p(600)),
        )
    });
    let mut d = event1_with_ids();
    d.lots[0].seller_account_id = Some(KARIM);
    let calc = edit_with(&d, rules, &stored);
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).seller_comm.amount), 600);
    assert_eq!(r(line(&calc, 0, 1).seller_comm.amount), 0);
    assert_eq!(r(single_lot_totals(&calc).seller_comm), 600);
    // Only the seller changed, to one with no per-lot rule: both lines are plain percent lines.
    let mut d = event1_with_ids();
    d.lots[0].seller_account_id = Some(KARIM);
    let calc = edit_with(&d, default_rules(), &stored);
    assert_no_errors(&calc);
    assert_eq!(r(single_lot_totals(&calc).seller_comm), 1_764);
}

#[test]
fn a_person_in_the_dead_amount_keeps_the_line_he_has_and_nothing_else_goes_onto_him() {
    // PRD FR-10.10, A67: ارشد is inactive (dead) but has a line in the posting being edited. His own
    // stored line (line 2) may stay, and may be corrected. A stored line that was اکرم's moved onto
    // him, and a new line for him, are refused. Money is as the rules say for every other line.
    let mut dead = lookups();
    if let Some(a) = dead.accounts.get_mut(&ARSHAD) {
        a.is_active = false;
        a.in_edited_posting = true;
    }
    let stored = stored_event1_with_identity();
    // His own line, with its rate corrected, and the lot's seller corrected.
    let mut d = event1_with_ids();
    d.lots[0].sales[1].seller_rate = "590".into();
    d.lots[0].seller_account_id = Some(KARIM);
    let calc = edit_with_lookups(&d, default_rules(), &stored, &dead);
    assert_no_errors(&calc);
    // اکرم's line moved onto him: refused at that line, and not at his own.
    let mut d = event1_with_ids();
    d.lots[0].sales[0].buyer_account_id = Some(ARSHAD);
    let calc = edit_with_lookups(&d, default_rules(), &stored, &dead);
    assert!(has(
        &calc.errors,
        ErrorCode::ErrAccountInactive,
        "lots[0].sales[0].buyer_account_id"
    ));
    assert!(!has(
        &calc.errors,
        ErrorCode::ErrAccountInactive,
        "lots[0].sales[1].buyer_account_id"
    ));
    // A new line (no stored id) for him is refused too. Line 2 is cut to 25 to leave room.
    let mut d = event1_with_ids();
    d.lots[0].sales[1].qty = "25".into();
    d.lots[0].sales.push(sale(ARSHAD, "5", "580"));
    let calc = edit_with_lookups(&d, default_rules(), &stored, &dead);
    assert!(has(
        &calc.errors,
        ErrorCode::ErrAccountInactive,
        "lots[0].sales[2].buyer_account_id"
    ));
    // A dead person who is not in the posting is refused everywhere, his own line or not.
    if let Some(a) = dead.accounts.get_mut(&ARSHAD) {
        a.in_edited_posting = false;
    }
    let calc = edit_with_lookups(&event1_with_ids(), default_rules(), &stored, &dead);
    assert!(has(
        &calc.errors,
        ErrorCode::ErrAccountInactive,
        "lots[0].sales[1].buyer_account_id"
    ));
}

#[test]
fn a_dead_seller_stays_the_seller_of_his_own_lot_but_no_lot_moves_onto_him() {
    // PRD FR-10.10, A67 (money review of plan 67, finding 7): کریم is inactive (dead) and is the
    // stored seller of lot 1, so he stays its seller while a line is corrected. A lot whose stored
    // seller is another person cannot be given to him.
    let mut dead = lookups();
    if let Some(a) = dead.accounts.get_mut(&KARIM) {
        a.is_active = false;
        a.in_edited_posting = true;
    }
    // Lot 1 was saved with کریم as its seller.
    let mut stored = stored_event1_with_identity();
    if let Some(identity) = stored.lots.get_mut(&1).and_then(|l| l.identity.as_mut()) {
        identity.seller_account_id = KARIM;
    }
    let mut d = event1_with_ids();
    d.lots[0].seller_account_id = Some(KARIM);
    d.lots[0].sales[0].seller_rate = "610".into();
    let calc = edit_with_lookups(&d, default_rules(), &stored, &dead);
    assert_no_errors(&calc);
    // Lot 1 was رحیم's: giving it to the dead کریم is refused at the seller of the lot.
    let stored = stored_event1_with_identity();
    let mut d = event1_with_ids();
    d.lots[0].seller_account_id = Some(KARIM);
    let calc = edit_with_lookups(&d, default_rules(), &stored, &dead);
    assert!(has(
        &calc.errors,
        ErrorCode::ErrAccountInactive,
        "lots[0].seller_account_id"
    ));
    // A new lot (no stored id) with him as the seller is refused too.
    let mut d = event1();
    d.lots[0].seller_account_id = Some(KARIM);
    let calc = edit_with_lookups(&d, default_rules(), &stored, &dead);
    assert!(has(
        &calc.errors,
        ErrorCode::ErrAccountInactive,
        "lots[0].seller_account_id"
    ));
}
