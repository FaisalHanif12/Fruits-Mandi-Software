//! The memories of the بکری screen, read from saved data (PRD FR-7.49 to FR-7.53, plan 12):
//! truck, seller and the last rate. The saved rows are put in with SQL, because the save comes
//! in plan 13.

use mandi_services::assist;
use mandi_services::core::bikri::{LastRateInput, RateHintSource, TruckMemoryInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::masters::{MarkaInput, MasterInput};
use mandi_services::masters;

use crate::common::raw;
use crate::support::*;

fn truck(w: &World, text: &str, date: &str) -> mandi_services::core::bikri::TruckMemory {
    assist::truck_memory(
        &w.env.ctx,
        &TruckMemoryInput {
            truck_no: text.into(),
            date: date.into(),
        },
    )
    .unwrap()
}

#[test]
fn a_truck_used_before_brings_back_its_stand_zamindar_and_driver() {
    let w = world();
    let mut s = SavedSale::new(1, "2026-10-10", w.rahim, w.akram, w.banana, w.box_);
    s.truck = Some("LES-1234");
    s.zamindar = Some(w.rahim);
    s.driver = Some("Aslam");
    insert_saved(&w, &s);
    // E2E-25: the same truck today names بکری 1; letter case does not matter.
    let m = truck(&w, "les-1234", "10-10-2026");
    let last = m.last.expect("the last بکری of the truck");
    assert_eq!(
        (
            last.bikri_number,
            last.date.as_str(),
            last.zamindar_account_id
        ),
        (1, "2026-10-10", Some(w.rahim))
    );
    assert_eq!(last.driver_name.as_deref(), Some("Aslam"));
    assert_eq!(m.same_day.map(|d| d.bikri_number), Some(1));
    // On another day the notice is gone, the memory stays.
    let other = truck(&w, "LES-1234", "09-10-2026");
    assert!(other.same_day.is_none() && other.last.is_some());
}

#[test]
fn truck_numbers_start_with_what_was_typed_newest_first() {
    let w = world();
    for (n, date, number) in [
        (1, "2026-10-08", "LES-5678"),
        (2, "2026-10-09", "LHR-0077"),
        (3, "2026-10-10", "LES-1234"),
    ] {
        let mut s = SavedSale::new(n, date, w.rahim, w.akram, w.banana, w.box_);
        s.truck = Some(number);
        insert_saved(&w, &s);
    }
    let all: Vec<String> = truck(&w, "L", "10-10-2026")
        .suggestions
        .into_iter()
        .map(|s| s.truck_no)
        .collect();
    assert_eq!(all, ["LES-1234", "LHR-0077", "LES-5678"]);
    let les: Vec<String> = truck(&w, "les", "10-10-2026")
        .suggestions
        .into_iter()
        .map(|s| s.truck_no)
        .collect();
    assert_eq!(les, ["LES-1234", "LES-5678"]);
    // `%` and `_` are plain characters, not wild cards.
    assert!(truck(&w, "%", "10-10-2026").suggestions.is_empty());
    assert!(truck(&w, "L_S", "10-10-2026").suggestions.is_empty());
    // Nothing typed, nothing offered.
    let empty = truck(&w, "  ", "10-10-2026");
    assert!(empty.last.is_none() && empty.suggestions.is_empty());
}

#[test]
fn a_cancelled_bikri_is_not_remembered() {
    let w = world();
    let mut s = SavedSale::new(1, "2026-10-10", w.rahim, w.akram, w.banana, w.box_);
    s.truck = Some("LES-1234");
    insert_saved(&w, &s);
    raw(&w.dir)
        .execute(
            "UPDATE bikri_entries SET status = 'cancelled', cancel_reason = 'x' WHERE number = 1",
            [],
        )
        .unwrap();
    let m = truck(&w, "LES-1234", "10-10-2026");
    assert!(m.last.is_none() && m.same_day.is_none() && m.suggestions.is_empty());
}

#[test]
fn the_truck_date_is_checked() {
    let w = world();
    let error = assist::truck_memory(
        &w.env.ctx,
        &TruckMemoryInput {
            truck_no: "LES".into(),
            date: "31-02-2026".into(),
        },
    )
    .unwrap_err();
    assert!(error.has_code(ErrorCode::ErrDateInvalid));
}

fn marka(w: &World, text: &str) -> i64 {
    masters::save(
        &w.env.ctx,
        &w.owner,
        &MasterInput::Marka(MarkaInput {
            id: None,
            version: None,
            code: String::new(),
            mark_text: text.into(),
            owner_account_id: None,
        }),
    )
    .unwrap()
    .id()
}

#[test]
fn a_seller_brings_back_the_item_packing_and_mark_of_his_last_lot() {
    let w = world();
    let ar = marka(&w, "AR");
    let mut older = SavedSale::new(1, "2026-10-08", w.rahim, w.akram, w.banana, w.box_);
    older.marka = Some(ar);
    insert_saved(&w, &older);
    let newer = SavedSale::new(2, "2026-10-09", w.rahim, w.akram, w.apple, w.box_);
    insert_saved(&w, &newer);
    let m = assist::seller_memory(&w.env.ctx, w.rahim).unwrap();
    // The newest lot gives the item; it has no mark, so the mark comes from nowhere.
    assert_eq!(
        (m.item_id, m.packing_id, m.from_last_lot),
        (Some(w.apple), Some(w.box_), true)
    );
    assert_eq!(m.marka_id, None);
    // His default mark comes first (PRD FR-7.51).
    raw(&w.dir)
        .execute(
            "UPDATE accounts SET default_marka_id = ?1 WHERE id = ?2",
            [ar, w.rahim],
        )
        .unwrap();
    let m = assist::seller_memory(&w.env.ctx, w.rahim).unwrap();
    assert_eq!(m.marka_id, Some(ar));
}

#[test]
fn a_mark_of_the_last_lot_is_used_when_he_has_no_default() {
    let w = world();
    let ar = marka(&w, "AR");
    let mut s = SavedSale::new(1, "2026-10-08", w.rahim, w.akram, w.banana, w.box_);
    s.marka = Some(ar);
    insert_saved(&w, &s);
    assert_eq!(
        assist::seller_memory(&w.env.ctx, w.rahim).unwrap().marka_id,
        Some(ar)
    );
}

#[test]
fn a_seller_with_no_lot_and_the_wrong_kinds_of_account() {
    let w = world();
    let none = assist::seller_memory(&w.env.ctx, w.karim).unwrap();
    assert_eq!(
        (
            none.item_id,
            none.packing_id,
            none.marka_id,
            none.from_last_lot
        ),
        (None, None, None, false)
    );
    // The one-time seller account holds many people, so it has no memory.
    let pukhta = assist::seller_memory(&w.env.ctx, w.pukhta_seller).unwrap();
    assert!(!pukhta.from_last_lot && pukhta.item_id.is_none());
    assert!(
        assist::seller_memory(&w.env.ctx, w.akram)
            .unwrap_err()
            .has_code(ErrorCode::ErrNotASeller)
    );
    assert!(
        assist::seller_memory(&w.env.ctx, 99_999)
            .unwrap_err()
            .has_code(ErrorCode::ErrNotFound)
    );
}

fn last_rate(w: &World, date: &str) -> Option<mandi_services::core::bikri::RateHint> {
    assist::last_rate(
        &w.env.ctx,
        &LastRateInput {
            item_id: w.banana,
            packing_id: w.box_,
            date: date.into(),
        },
    )
    .unwrap()
}

#[test]
fn the_last_rate_is_todays_else_the_last_day_it_sold() {
    let w = world();
    assert!(last_rate(&w, "10-10-2026").is_none(), "never sold");
    let mut old = SavedSale::new(1, "2026-10-07", w.rahim, w.akram, w.banana, w.box_);
    old.rate = 590;
    insert_saved(&w, &old);
    let hint = last_rate(&w, "10-10-2026").unwrap();
    assert_eq!(
        (r(hint.rate_paisa), hint.source, hint.date.as_str()),
        (590, RateHintSource::LastDay, "2026-10-07")
    );
    // Two sales today: the last one saved wins.
    for (n, rate) in [(2, 610), (3, 620)] {
        let mut s = SavedSale::new(n, "2026-10-10", w.rahim, w.akram, w.banana, w.box_);
        s.rate = rate;
        insert_saved(&w, &s);
    }
    let hint = last_rate(&w, "10-10-2026").unwrap();
    assert_eq!(
        (r(hint.rate_paisa), hint.source),
        (620, RateHintSource::Today)
    );
    // Looking at an earlier day does not see later sales.
    let hint = last_rate(&w, "08-10-2026").unwrap();
    assert_eq!(
        (r(hint.rate_paisa), hint.source),
        (590, RateHintSource::LastDay)
    );
    // Another packing or item has its own rate.
    let other = assist::last_rate(
        &w.env.ctx,
        &LastRateInput {
            item_id: w.apple,
            packing_id: w.box_,
            date: "10-10-2026".into(),
        },
    )
    .unwrap();
    assert!(other.is_none());
}

#[test]
fn a_replaced_or_cancelled_sale_does_not_give_a_rate() {
    let w = world();
    let mut s = SavedSale::new(1, "2026-10-10", w.rahim, w.akram, w.banana, w.box_);
    s.rate = 700;
    let (_, _, sale_id) = insert_saved(&w, &s);
    assert_eq!(r(last_rate(&w, "10-10-2026").unwrap().rate_paisa), 700);
    raw(&w.dir)
        .execute(
            "UPDATE bikri_sales SET is_active = 0, ended = 'cancelled' WHERE id = ?1",
            [sale_id],
        )
        .unwrap();
    assert!(last_rate(&w, "10-10-2026").is_none());
}

#[test]
fn a_saved_rate_is_the_hint_in_the_preview_until_a_line_above_has_one() {
    let w = world();
    let mut s = SavedSale::new(1, "2026-10-10", w.karim, w.arshad, w.banana, w.box_);
    s.rate = 590;
    insert_saved(&w, &s);
    let d = draft(vec![with_sales(
        lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![blank_sale("s-1", "50")],
    )]);
    let p = preview(&w, &d);
    let hint = p.lots[0].lines[0].rate_hint.as_ref().unwrap();
    assert_eq!(
        (r(hint.rate_paisa), hint.source),
        (590, RateHintSource::Today)
    );
}
