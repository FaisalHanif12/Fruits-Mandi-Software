//! Plan 97 item 5 (PRD FR-12.52, FR-12.26, FR-12.36, C8, C25): the garden card and the trip card of
//! the بکری summary. A garden lot and a trip lot in one بکری have one card each, and each figure is
//! the line the saved journal holds.
//!
//! The setup is E2E-34 and E2E-40 together. The trip: 20 پیٹی مالٹا for 10,400, 15 sold on 10-10-26,
//! 5 left. The garden شاہ پور باغ: on 12-10-26 a first sale of 100 کیلا at 600 credits its account
//! 56,400 (60,000 less seller commission 3,600). Then one بکری has:
//!
//! * a garden lot of 50 کیلا at 600: gross 30,000, seller commission 1,800, credit 28,200. The card:
//!   previous 56,400 جمع, credit 28,200, new 84,600 جمع;
//! * a trip lot of the last 5 مالٹا at 700: gross 3,500, seller commission 210, no other charge,
//!   credit 3,290, which goes to the trip's partners by their shares.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;

use std::path::{Path, PathBuf};

use common::{raw, set_today};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{BikriDraft, LotInput};
use reports_world::*;
use serde::Serialize;
use serde_json::{Value, json};

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// A fixture is a pair: `<name>_request.json` is the `input` of the command, `<name>.json` the real
/// answer. The test fails if a file differs; after a deliberate change write them again with
/// `MANDI_WRITE_FIXTURES=1`, read the diff and tell `frontend`.
fn write_or_check(file: &str, text: &str) {
    let path = folder().join(file);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::create_dir_all(folder()).unwrap();
        std::fs::write(&path, text).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{file} is missing: run with {WRITE_VARIABLE}=1 to write the fixtures")
    });
    assert!(
        on_disk == text,
        "{file} is not the real answer any more. If the change is on purpose, run with \
         {WRITE_VARIABLE}=1, read the diff and tell frontend."
    );
}

fn pair<A: Serialize>(name: &str, request: &Value, answer: &A) {
    write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

/// What a posted بکری credits an account: credit less debit, in paisa, over the lines of its
/// journal entries.
fn credited(w: &support::World, bikri_id: i64, account_id: i64) -> i64 {
    raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(l.cr_paisa), 0) - COALESCE(SUM(l.dr_paisa), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE e.source_type = 'bikri' AND e.source_id = ?1 AND l.account_id = ?2",
            [bikri_id, account_id],
            |r| r.get(0),
        )
        .unwrap()
}

/// What a posted بکری credits the 0604 accounts of the partners, together.
fn credited_to_partners(w: &support::World, bikri_id: i64) -> i64 {
    raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(l.cr_paisa), 0) - COALESCE(SUM(l.dr_paisa), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             JOIN accounts a ON a.id = l.account_id JOIN account_controls k ON k.id = a.control_id
             WHERE e.source_type = 'bikri' AND e.source_id = ?1 AND k.code = '0604'",
            [bikri_id],
            |r| r.get(0),
        )
        .unwrap()
}

/// The world: the trip of E2E-34 with 5 نگ left, the garden of E2E-40, and the garden's first sale
/// of 12-10-26 saved. Today is 12-10-26.
fn world() -> (
    support::World,
    mandi_services::core::gardens::GardenRow,
    i64,
    i64,
) {
    let (w, trip, purchase, _first) = trip_34_world();
    let garden = garden_made(&w);
    set_today(&w.env, 2026, 10, 12);
    let mut lot = support::with_sales(
        support::bare_lot(&w, "garden-1", garden.account.id, w.banana, "100"),
        vec![support::sale("s-1", w.arshad, "100", "600")],
    );
    lot.adjust_advance = false;
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = "12-10-2026".into();
    save(&w, &draft);
    (w, garden, trip, purchase)
}

/// The بکری of the test: the garden lot of 50 and the trip lot of the last 5 مالٹا.
fn mixed_draft(
    w: &support::World,
    garden: &mandi_services::core::gardens::GardenRow,
    purchase: i64,
) -> BikriDraft {
    let mut garden_lot = support::with_sales(
        support::bare_lot(w, "garden-2", garden.account.id, w.banana, "50"),
        vec![support::sale("g-1", w.arshad, "50", "600")],
    );
    garden_lot.adjust_advance = false;
    let own_lot = LotInput {
        seller_account_id: Some(support::id_of(&w.env, "030119998")),
        purchase_id: Some(purchase),
        item_id: None,
        packing_id: None,
        adjust_advance: false,
        ..support::with_sales(
            support::bare_lot(w, "own-2", w.rahim, w.banana, "5"),
            vec![support::sale("t-1", w.akram, "5", "700")],
        )
    };
    let mut draft = support::draft(vec![garden_lot, own_lot]);
    draft.header.entry_date = "12-10-2026".into();
    draft
}

#[test]
fn the_cards_of_a_garden_lot_and_a_trip_lot_are_the_lines_the_journal_posts() {
    let (w, garden, trip, purchase) = world();
    let draft = mixed_draft(&w, &garden, purchase);
    let preview = support::preview(&w, &draft);
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    // A garden lot and a trip lot take no نقد, so no seller has a summary.
    assert!(preview.sellers.is_empty());

    assert_eq!(preview.gardens.len(), 1);
    let card = &preview.gardens[0];
    assert_eq!(
        (card.garden.id, card.garden.account.id),
        (garden.id, garden.account.id)
    );
    assert_eq!(
        (support::r(card.previous_paisa), card.previous_side),
        (56_400, BalanceSide::Cr)
    );
    assert_eq!(support::r(card.credit_paisa), 28_200);
    assert_eq!(
        (support::r(card.new_paisa), card.new_side),
        (84_600, BalanceSide::Cr)
    );

    assert_eq!(preview.trips.len(), 1);
    let trip_card = &preview.trips[0];
    assert_eq!(trip_card.trip.id, trip);
    assert_eq!(
        (
            support::r(trip_card.gross_paisa),
            support::r(trip_card.deductions_paisa),
            support::r(trip_card.credit_paisa)
        ),
        (3_500, 210, 3_290)
    );

    // The same figures are what each lot of the preview says (the totals the journal is built from).
    let lot_credit = |i: usize| support::r(preview.lots[i].totals.as_ref().unwrap().seller_credit);
    assert_eq!(lot_credit(0), 28_200);
    assert_eq!(lot_credit(1), 3_290);

    // The save writes exactly these cards, and the journal holds exactly these credits.
    let saved = save(&w, &draft);
    assert_eq!(saved.preview.gardens, preview.gardens);
    assert_eq!(saved.preview.trips, preview.trips);
    assert_eq!(
        credited(&w, saved.bikri_id, garden.account.id),
        card.credit_paisa.as_paisa(),
        "the garden's account is credited the card's credit"
    );
    assert_eq!(
        credited_to_partners(&w, saved.bikri_id),
        trip_card.credit_paisa.as_paisa(),
        "the trip's partners are credited the card's credit, by their shares (C25)"
    );
    // After the save the garden's account holds what the card said it would.
    let after = mandi_services::ledger::account_balance(&w.env.ctx, garden.account.id).unwrap();
    assert_eq!(
        (after.side, after.amount_paisa.rupees()),
        (BalanceSide::Cr, support::r(card.new_paisa)),
        "the new balance of the card is the balance of the account"
    );
}

/// An edit of the saved بکری leaves its own posting out of the previous balance, so the card is the
/// same before and after the edit and the credit is not counted twice.
#[test]
fn the_card_of_an_edit_does_not_count_the_posting_being_edited() {
    let (w, garden, _trip, purchase) = world();
    let draft = mixed_draft(&w, &garden, purchase);
    let saved = save(&w, &draft);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let again = tools::preview(&w, &opened, &opened.draft);
    assert!(again.errors.is_empty(), "{:?}", again.errors);
    assert_eq!(again.gardens, saved.preview.gardens);
    assert_eq!(again.trips, saved.preview.trips);
    let card = &again.gardens[0];
    assert_eq!(
        (support::r(card.previous_paisa), card.previous_side),
        (56_400, BalanceSide::Cr)
    );
}

/// A بکری with no garden lot and no trip lot has no card.
#[test]
fn an_ordinary_bikri_has_no_garden_or_trip_card() {
    let (w, _garden, _trip, _purchase) = world();
    let preview = support::preview(&w, &support::event1(&w));
    assert!(preview.gardens.is_empty());
    assert!(preview.trips.is_empty());
}

/// The real answer for the screen: the preview of the بکری of this file with its garden card and its
/// trip card, as `bikri_preview` gives it.
#[test]
fn the_preview_with_a_garden_card_and_a_trip_card_is_a_fixture_for_the_screen() {
    let (w, garden, _trip, purchase) = world();
    let draft = mixed_draft(&w, &garden, purchase);
    let preview = support::preview(&w, &draft);
    assert_eq!((preview.gardens.len(), preview.trips.len()), (1, 1));
    pair(
        "bikri_preview_garden_and_trip_cards",
        &json!({ "draft": draft }),
        &preview,
    );
}
