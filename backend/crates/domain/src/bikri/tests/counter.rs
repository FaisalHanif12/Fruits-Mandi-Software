//! The counter of each lot's name (PRD FR-12.45, plan 71 L1 to L5, E2E-70).
//!
//! The numbers worked out here are the ones the preview shows and the save writes: one function,
//! `calc_posting`, serves both (Code Standards B5).

use std::collections::HashMap;

use mandi_core::bikri::prepare_draft;

use super::*;
use crate::bikri::{PostingTarget, StoredIdentity, StoredLot, StoredPosting};

/// In these tests BANANA stands for آم (the mango) and APPLE for کیلا.
const MANGO: i64 = BANANA;
const BANANA_ITEM: i64 = APPLE;

fn counters(calc: &PostingCalc) -> Vec<Option<i64>> {
    calc.lots.iter().map(|l| l.lot_counter).collect()
}

fn with_counters(pairs: &[((i64, i64), i64)]) -> Lookups {
    Lookups {
        lot_counters: pairs.iter().copied().collect(),
        ..lookups()
    }
}

fn sold(seller: i64, item: i64) -> LotInput {
    with(lot(seller, item, "10"), vec![sale(AKRAM, "10", "600")])
}

#[test]
fn lots_typed_mango_banana_mango_show_1_1_2_and_the_next_draft_continues() {
    // E2E-70: Ehsan (here رحیم) has no lot yet. آم, کیلا, آم show آم-1, کیلا-1, آم-2.
    let d = draft(vec![
        sold(RAHIM, MANGO),
        sold(RAHIM, BANANA_ITEM),
        sold(RAHIM, MANGO),
    ]);
    let calc = run_with(&d, &lookups(), default_rules(), switches());
    assert_eq!(counters(&calc), vec![Some(1), Some(1), Some(2)]);

    // After that بکری is saved, lot_counters holds 2 for mango and 1 for banana: his next بکری's
    // mango lot is آم-3 and his next banana lot is کیلا-2.
    let after = with_counters(&[((RAHIM, MANGO), 2), ((RAHIM, BANANA_ITEM), 1)]);
    let next = draft(vec![sold(RAHIM, MANGO), sold(RAHIM, BANANA_ITEM)]);
    let calc = run_with(&next, &after, default_rules(), switches());
    assert_eq!(counters(&calc), vec![Some(3), Some(2)]);
}

#[test]
fn the_counter_belongs_to_one_seller_and_one_item() {
    // Another seller starts at 1 for the same item (L2).
    let lookups = with_counters(&[((RAHIM, MANGO), 7)]);
    let d = draft(vec![
        sold(KARIM, MANGO),
        sold(RAHIM, MANGO),
        sold(KARIM, MANGO),
    ]);
    let calc = run_with(&d, &lookups, default_rules(), switches());
    assert_eq!(counters(&calc), vec![Some(1), Some(8), Some(2)]);
}

#[test]
fn a_lot_without_a_seller_or_an_item_has_no_number_yet_and_takes_none_from_the_next() {
    let mut no_seller = sold(RAHIM, MANGO);
    no_seller.seller_account_id = None;
    let mut no_item = sold(RAHIM, MANGO);
    no_item.item_id = None;
    let d = draft(vec![no_seller, no_item, sold(RAHIM, MANGO)]);
    let calc = run(&d);
    assert_eq!(counters(&calc), vec![None, None, Some(1)]);
}

fn stored_lot(counter: i64, seller: i64, item: i64) -> StoredLot {
    StoredLot {
        lot_counter: Some(counter),
        identity: Some(StoredIdentity {
            seller_account_id: seller,
            item_id: item,
            purchase_id: None,
            arrival_id: None,
        }),
        ..StoredLot::default()
    }
}

fn stored(lots: Vec<(i64, StoredLot)>) -> StoredPosting {
    StoredPosting {
        kind: crate::bikri::PostingKind::First,
        lots: lots.into_iter().collect::<HashMap<_, _>>(),
        sales: HashMap::new(),
    }
}

fn edit(
    d: &mandi_core::bikri::BikriDraft,
    lookups: &Lookups,
    stored: &StoredPosting,
) -> PostingCalc {
    let prepared = prepare_draft(d, today(), None);
    calc_posting(
        PostingTarget::Edit { entry_id: 1 },
        &prepared,
        lookups,
        &crate::rules::RuleSet::build(default_rules()),
        switches(),
        stored,
    )
    .unwrap()
}

#[test]
fn an_edit_keeps_the_number_of_a_lot_that_keeps_its_seller_and_item() {
    // Lot 1 is آم-2 and his biggest ever is 5. The edit changes nothing of its owner: it stays آم-2.
    // A new lot of the same seller and item below it takes 6, not 3 (L4).
    let lookups = with_counters(&[((RAHIM, MANGO), 5)]);
    let st = stored(vec![(1, stored_lot(2, RAHIM, MANGO))]);
    let mut saved = sold(RAHIM, MANGO);
    saved.lot_id = Some(1);
    let d = draft(vec![saved, sold(RAHIM, MANGO)]);
    let calc = edit(&d, &lookups, &st);
    assert_eq!(counters(&calc), vec![Some(2), Some(6)]);
}

#[test]
fn an_edit_that_changes_the_seller_or_the_item_takes_the_next_number_of_the_new_pair() {
    // L5. Lot 1 is رحیم آم-2. The edit moves it to کریم, who has had 3 mango lots: it becomes کریم
    // آم-4. The old number stays unused (lot_counters still says 5 for رحیم's mango).
    let lookups = with_counters(&[((RAHIM, MANGO), 5), ((KARIM, MANGO), 3)]);
    let st = stored(vec![(1, stored_lot(2, RAHIM, MANGO))]);
    let mut moved = sold(KARIM, MANGO);
    moved.lot_id = Some(1);
    let calc = edit(&draft(vec![moved]), &lookups, &st);
    assert_eq!(counters(&calc), vec![Some(4)]);

    // Another item of the same seller starts that pair's own count.
    let mut changed_item = sold(RAHIM, BANANA_ITEM);
    changed_item.lot_id = Some(1);
    let calc = edit(&draft(vec![changed_item]), &lookups, &st);
    assert_eq!(counters(&calc), vec![Some(1)]);

    // Two lots that land on the same new pair take consecutive numbers, in the order of the draft.
    let mut first = sold(KARIM, MANGO);
    first.lot_id = Some(1);
    let calc = edit(&draft(vec![sold(KARIM, MANGO), first]), &lookups, &st);
    assert_eq!(counters(&calc), vec![Some(4), Some(5)]);
}

#[test]
fn a_later_sale_keeps_the_number_of_its_lot() {
    // A later sale does not say who the lot belongs to; it counts as unchanged.
    let lookups = with_counters(&[((RAHIM, MANGO), 5)]);
    let mut later = sold(RAHIM, MANGO);
    later.lot_id = Some(1);
    let st = stored(vec![(
        1,
        StoredLot {
            lot_counter: Some(3),
            identity: None,
            ..StoredLot::default()
        },
    )]);
    let prepared = prepare_draft(&draft(vec![later]), today(), None);
    let calc = calc_posting(
        PostingTarget::LaterSale { lot_id: 1 },
        &prepared,
        &lookups,
        &crate::rules::RuleSet::build(default_rules()),
        switches(),
        &st,
    )
    .unwrap();
    assert_eq!(counters(&calc), vec![Some(3)]);
}
