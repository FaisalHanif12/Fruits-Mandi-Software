//! The garden and trip cards of the summary (PRD FR-12.52, plan 97 item 5): the worked numbers
//! below, the card of a garden lot and of a trip lot through `calc_posting`, and the merge of the
//! parts of a mixed screen. Every credit is read against the lot totals the journal is built from.

use mandi_core::accounts::{BalanceSide, RecordRef};
use mandi_core::bikri::{LotInput, LotPreview};
use mandi_core::ids::{AccountId, PartnerId};
use mandi_core::money::{Bps, Paisa};
use mandi_core::rules::WorkingValue;
use mandi_core::ventures::{GardenRef, MoneySource, TripRef};

use super::*;
use crate::bikri::{
    PostingCalc, PurchaseFacts, TripPartnerFacts, garden_lines, merge_gardens, merge_trips,
    trip_lines,
};
use crate::ventures::VenturePartner;

const GARDEN_ACCOUNT: i64 = 40;
const TRIP_A: i64 = 201;
const TRIP_B: i64 = 202;

fn garden() -> GardenRef {
    GardenRef {
        id: 7,
        number: 1,
        code: "0080".into(),
        name_ur: "شاہ پور باغ".into(),
        name_en: "Shahpur Garden".into(),
        account: RecordRef {
            id: GARDEN_ACCOUNT,
            code: "0080".into(),
            name_ur: "شاہ پور باغ".into(),
            name_en: "Shahpur Garden".into(),
            tag: None,
        },
    }
}

fn trip(id: i64) -> TripRef {
    TripRef {
        id,
        number: id,
        code: format!("007{}", id - 1),
        name_ur: format!("007{}", id - 1),
        name_en: format!("007{}", id - 1),
    }
}

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

/// A purchase of 40 نگ for 20,800 in the trip `trip_id`, with two partners: enough for two lots of 15.
fn purchase(trip_id: i64) -> PurchaseFacts {
    PurchaseFacts {
        number: 1,
        entry_date: "2026-10-10".into(),
        trip: trip(trip_id),
        partners: vec![partner(1, TRIP_A, 5_000), partner(2, TRIP_B, 5_000)],
        item_id: BANANA,
        packing_id: BOX,
        our_left: 40,
        qty_left: 40,
        cost_left: p(20_800),
        cancelled: false,
    }
}

fn with_garden(previous: Option<i64>) -> Lookups {
    let mut lookups = lookups();
    lookups.accounts.insert(
        GARDEN_ACCOUNT,
        AccountFacts {
            kind: PartyKind::Garden,
            is_chukta: false,
            is_naqdi_mandi: false,
            is_own_goods: false,
            garden_closed: false,
            is_active: true,
            in_edited_posting: false,
            credit_allowed: true,
            credit_limit: None,
            balance: Paisa::ZERO,
        },
    );
    lookups.gardens.insert(GARDEN_ACCOUNT, garden());
    if let Some(rupees) = previous {
        lookups.previous.insert(GARDEN_ACCOUNT, p(rupees));
    }
    lookups
}

/// A garden lot of 100 نگ, all sold at 200: gross 20,000, seller commission 6% = 1,200, no lot
/// charge, so the credit is 18,800.
fn garden_lot(key: &str) -> LotInput {
    LotInput {
        key: key.into(),
        ..with(
            bare_lot(GARDEN_ACCOUNT, BANANA, "100"),
            vec![sale(AKRAM, "100", "200")],
        )
    }
}

fn totals(lot: &LotPreview) -> &mandi_core::bikri::SummaryTotals {
    lot.totals.as_ref().expect("the lot has totals")
}

fn money_arg(working: &mandi_core::rules::Working, name: &str) -> Paisa {
    working
        .args
        .iter()
        .find(|a| a.name == name)
        .and_then(|a| match &a.value {
            WorkingValue::Money { paisa } => Some(*paisa),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no money argument {name}"))
}

fn run_garden(lots: Vec<LotInput>, previous: Option<i64>) -> PostingCalc {
    run_with(
        &draft(lots),
        &with_garden(previous),
        default_rules(),
        switches(),
    )
}

/// A garden lot: previous 12,000 جمع, credit 18,800, new 30,800 جمع. The card has no seller summary
/// beside it, and the credit is the lot's seller credit (C8), the line the journal posts.
#[test]
fn a_garden_lot_has_a_card_with_its_previous_credit_and_new_balance() {
    let calc = run_garden(vec![garden_lot("g1")], Some(-12_000));
    assert!(calc.errors.is_empty(), "{:?}", calc.errors);
    assert!(
        calc.sellers.is_empty(),
        "a garden lot has no seller summary"
    );
    assert!(calc.trips.is_empty());
    assert_eq!(calc.gardens.len(), 1);
    let card = &calc.gardens[0];
    assert_eq!(card.garden.id, 7);
    assert_eq!(
        (card.previous_paisa, card.previous_side),
        (p(12_000), BalanceSide::Cr)
    );
    assert_eq!(card.credit_paisa, p(18_800));
    assert_eq!(
        (card.new_paisa, card.new_side),
        (p(30_800), BalanceSide::Cr)
    );
    // The card's credit is the lot's seller credit, the same number the journal reads.
    assert_eq!(totals(&calc.lots[0]).seller_credit, card.credit_paisa);
    assert_eq!(totals(&calc.lots[0]).gross, p(20_000));
    // The working says how the new balance was made, with the signed figures.
    assert_eq!(card.working.message_id, "working-garden-new-balance");
    assert_eq!(money_arg(&card.working, "previous"), p(-12_000));
    assert_eq!(money_arg(&card.working, "credit"), p(18_800));
    assert_eq!(money_arg(&card.working, "result"), p(-30_800));
}

/// A garden with nothing before: previous 0 (side zero), new 18,800 جمع.
#[test]
fn a_garden_with_no_balance_starts_at_zero() {
    let calc = run_garden(vec![garden_lot("g1")], None);
    let card = &calc.gardens[0];
    assert_eq!(
        (card.previous_paisa, card.previous_side),
        (Paisa::ZERO, BalanceSide::Zero)
    );
    assert_eq!(
        (card.new_paisa, card.new_side),
        (p(18_800), BalanceSide::Cr)
    );
}

/// Two lots of one garden are one card: the credits add up and the previous balance is the one
/// balance of its account.
#[test]
fn two_lots_of_one_garden_are_one_card() {
    let calc = run_garden(vec![garden_lot("g1"), garden_lot("g2")], Some(-12_000));
    assert_eq!(calc.gardens.len(), 1);
    let card = &calc.gardens[0];
    assert_eq!(card.credit_paisa, p(37_600));
    assert_eq!(
        (card.new_paisa, card.new_side),
        (p(49_600), BalanceSide::Cr)
    );
    let lots_credit: i64 = calc
        .lots
        .iter()
        .map(|l| totals(l).seller_credit.rupees())
        .sum();
    assert_eq!(lots_credit, 37_600);
}

/// A lot that could not be worked out (no item) has no card.
#[test]
fn a_garden_lot_that_cannot_be_worked_out_has_no_card() {
    let mut lot = garden_lot("g1");
    lot.item_id = None;
    let calc = run_garden(vec![lot], Some(-12_000));
    assert!(calc.gardens.is_empty());
}

/// E2E-34: 15 of 20 نگ مالٹا at 600 sold from a trip purchase. Gross 9,000, seller commission 540 and
/// no lot charge: deductions 540, credit 8,460 (the money that goes to the trip's partners).
#[test]
fn e2e_34_a_trip_lot_has_a_card_with_gross_deductions_and_credit() {
    let own = LotInput {
        purchase_id: Some(1),
        adjust_advance: false,
        ..with(
            bare_lot(OWN_GOODS, BANANA, "15"),
            vec![sale(AKRAM, "15", "600")],
        )
    };
    let mut lookups = lookups();
    lookups.purchases.insert(1, purchase(1));
    let calc = run_with(&draft(vec![own]), &lookups, default_rules(), switches());
    assert!(calc.errors.is_empty(), "{:?}", calc.errors);
    assert!(calc.sellers.is_empty(), "a trip lot has no seller summary");
    assert!(calc.gardens.is_empty());
    assert_eq!(calc.trips.len(), 1);
    let card = &calc.trips[0];
    assert_eq!(card.trip.id, 1);
    assert_eq!(card.trip.code, "0070");
    assert_eq!(
        (card.gross_paisa, card.deductions_paisa, card.credit_paisa),
        (p(9_000), p(540), p(8_460))
    );
    // No نقد on a trip lot, so the credit is the gross less the deductions, and it is the credit
    // the trip's partners are paid from.
    assert_eq!(
        card.gross_paisa.checked_sub(card.deductions_paisa).unwrap(),
        card.credit_paisa
    );
    assert_eq!(
        calc.lots[0].own.as_ref().unwrap().credit_paisa,
        card.credit_paisa
    );
    assert_eq!(card.working.message_id, "working-trip-card-credit");
    assert_eq!(money_arg(&card.working, "gross"), p(9_000));
    assert_eq!(money_arg(&card.working, "deductions"), p(540));
    assert_eq!(money_arg(&card.working, "result"), p(8_460));
}

/// Two trips and a garden in one بکری: one card each, in the order of the first lot of each.
#[test]
fn a_garden_and_two_trips_in_one_bikri_have_one_card_each() {
    let own = |purchase_id: i64, key: &str| LotInput {
        key: key.into(),
        purchase_id: Some(purchase_id),
        adjust_advance: false,
        ..with(
            bare_lot(OWN_GOODS, BANANA, "15"),
            vec![sale(AKRAM, "15", "600")],
        )
    };
    let mut lookups = with_garden(Some(-12_000));
    lookups.purchases.insert(1, purchase(1));
    lookups.purchases.insert(2, purchase(2));
    let calc = run_with(
        &draft(vec![
            own(2, "t2"),
            garden_lot("g1"),
            own(1, "t1"),
            own(2, "t2b"),
        ]),
        &lookups,
        default_rules(),
        switches(),
    );
    assert!(calc.errors.is_empty(), "{:?}", calc.errors);
    let ids: Vec<i64> = calc.trips.iter().map(|t| t.trip.id).collect();
    assert_eq!(ids, vec![2, 1], "in the order of each trip's first lot");
    // Trip 2 has two lots: 2 × 8,460.
    assert_eq!(calc.trips[0].credit_paisa, p(16_920));
    assert_eq!(calc.trips[1].credit_paisa, p(8_460));
    assert_eq!(calc.gardens.len(), 1);
    assert_eq!(calc.gardens[0].credit_paisa, p(18_800));
}

/// `new = previous - credit` on both sides of zero. A debit of 5,000 and a credit of 8,000 turn into
/// 3,000 جمع; a credit below 0 (the charges are more than the sales) adds to the debit.
#[test]
fn the_new_balance_is_previous_less_credit_on_either_side() {
    let owes = garden_lines(garden(), p(5_000), p(8_000)).unwrap();
    assert_eq!((owes.new_paisa, owes.new_side), (p(3_000), BalanceSide::Cr));
    assert_eq!(
        (owes.previous_paisa, owes.previous_side),
        (p(5_000), BalanceSide::Dr)
    );
    let loss = garden_lines(garden(), p(5_000), p(-2_000)).unwrap();
    assert_eq!((loss.new_paisa, loss.new_side), (p(7_000), BalanceSide::Dr));
    let even = garden_lines(garden(), p(-4_000), p(-4_000)).unwrap();
    assert_eq!(
        (even.new_paisa, even.new_side),
        (Paisa::ZERO, BalanceSide::Zero)
    );
}

/// The parts of a mixed screen: a garden in two parts is one card with the credits added and the
/// previous balance of the first part; trips are added by trip.
#[test]
fn the_parts_of_a_mixed_screen_merge_into_one_card_each() {
    let first = garden_lines(garden(), p(-12_000), p(18_800)).unwrap();
    let second = garden_lines(garden(), p(-12_000), p(1_200)).unwrap();
    let merged = merge_gardens(&[&[first], &[second]]).unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(
        (merged[0].previous_paisa, merged[0].previous_side),
        (p(12_000), BalanceSide::Cr)
    );
    assert_eq!(merged[0].credit_paisa, p(20_000));
    assert_eq!(
        (merged[0].new_paisa, merged[0].new_side),
        (p(32_000), BalanceSide::Cr)
    );

    let a = trip_lines(trip(1), p(9_000), p(540), p(8_460)).unwrap();
    let b = trip_lines(trip(1), p(1_000), p(60), p(940)).unwrap();
    let c = trip_lines(trip(2), p(500), p(30), p(470)).unwrap();
    let merged = merge_trips(&[&[a], &[b, c]]).unwrap();
    assert_eq!(merged.len(), 2);
    assert_eq!(
        (
            merged[0].gross_paisa,
            merged[0].deductions_paisa,
            merged[0].credit_paisa
        ),
        (p(10_000), p(600), p(9_400))
    );
    assert_eq!(merged[1].trip.id, 2);
}
