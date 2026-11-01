//! Plan 46 (B1): the seller list of a lot offers only what a lot can use (PRD 12.3 "Seller code",
//! FR-12.26, FR-12.36, FR-16.7). The sellers and the پختہ کھاتہ بیوپاری are always in it, a separate
//! khata is in it, اپنا مال only while an open trip has a purchase with goods left for our shop, and
//! a garden only while it is open. "Goods left" and "open" are read from the database at query time
//! (the in-memory index is a speed aid, Code Standards B27). Nothing about money changes here, and
//! the save still refuses an unusable اپنا مال or a closed garden.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::time::Instant;

use common::set_today;
use mandi_services::core::accounts::{
    AccountScope, AccountsRecentInput, ResolveCodeInput, ResolveStatus, SearchAccountsInput,
};
use mandi_services::core::bikri::{BikriDraft, BikriSaveInput, BikriSaved, HeaderInput, LotInput};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::gardens::{
    GardenCloseInput, GardenGetInput, GardenInput, GardenReopenInput, GardenRow,
};
use mandi_services::core::khata::KhataCreateInput;
use mandi_services::core::purchases::{
    PurchaseCancelInput, PurchaseGetInput, PurchaseInput, PurchaseRow, PurchaseWriteOffInput,
    PurchaseWriteOffUndoInput,
};
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{ServiceError, bikri, gardens, khata, purchases, search, trips};
use support::{World, bare_lot, id_of, sale, with_sales, world};

const OWN_GOODS: &str = "030119998";

fn day(w: &World, d: u32) {
    set_today(&w.env, 2026, 10, d);
}

fn partners_of(w: &World) -> Vec<VenturePartnerInput> {
    let [a, b, c] = w.partners;
    [(a, "40"), (b, "30"), (c, "30")]
        .into_iter()
        .map(|(partner_id, share)| VenturePartnerInput {
            partner_id,
            share: share.into(),
            source: MoneySource::Shop,
        })
        .collect()
}

fn trip(w: &World) -> i64 {
    trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: partners_of(w),
        },
    )
    .unwrap()
    .id
}

fn buy_split(w: &World, trip_id: i64, qty: &str, other_qty: Option<&str>) -> PurchaseRow {
    let saved = purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: other_qty.map(str::to_owned),
            shop_name: other_qty.map(|_| "Haji Shop".to_owned()),
            sell_date: None,
            trip_id: Some(trip_id),
            date: "11-10-2026".into(),
            supplier: None,
            item_id: Some(w.banana),
            packing_id: Some(w.box_),
            qty: qty.into(),
            weight: None,
            rate: "50".into(),
            freight: None,
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    purchase(w, saved.id)
}

fn buy(w: &World, trip_id: i64, qty: &str) -> PurchaseRow {
    buy_split(w, trip_id, qty, None)
}

fn purchase(w: &World, id: i64) -> PurchaseRow {
    purchases::get(&w.env.ctx, &PurchaseGetInput { id }).unwrap()
}

fn garden(w: &World) -> GardenRow {
    gardens::create(
        &w.env.ctx,
        &w.owner,
        &GardenInput {
            name_ur: "شاہ پور باغ".into(),
            name_en: "Shahpur Garden".into(),
            confirmed_rules_words: vec![
                "شاہ".into(),
                "پور".into(),
                "باغ".into(),
                "Shahpur".into(),
                "Garden".into(),
            ],
            date: "10-10-2026".into(),
            item_id: Some(w.banana),
            rent: "50,000".into(),
            season_end: None,
            note: None,
            partners: partners_of(w),
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .row
}

fn garden_version(w: &World, id: i64) -> i64 {
    gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap()
    .row
    .version
}

fn close_garden(w: &World, id: i64) {
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &GardenCloseInput {
            id,
            version: garden_version(w, id),
            date: "11-10-2026".into(),
        },
    )
    .unwrap();
}

/// The codes the seller field of a lot lists for this text, best first (the empty text is the
/// whole list that opens on focus).
fn list(w: &World, text: &str) -> Vec<String> {
    search::search_accounts(
        &w.env.ctx,
        &w.owner,
        &SearchAccountsInput {
            include_dead: None,
            scope: AccountScope::LotSeller,
            control_id: None,
            text: text.into(),
            limit: 50,
            offset: 0,
        },
    )
    .unwrap()
    .into_iter()
    .map(|hit| hit.code)
    .collect()
}

/// The codes of the "recent" list (`Alt+Down`) of the same field.
fn recent(w: &World) -> Vec<String> {
    search::recent_accounts(
        &w.env.ctx,
        &w.owner,
        &AccountsRecentInput {
            include_dead: None,
            scope: AccountScope::LotSeller,
            control_id: None,
        },
    )
    .unwrap()
    .into_iter()
    .map(|hit| hit.code)
    .collect()
}

/// True if the lot seller field offers the account in all three of its ways: the whole list, a
/// typed name (or code) and the recent list.
fn offered(w: &World, code: &str, typed: &str) -> bool {
    let a = list(w, "").iter().any(|c| c == code);
    let b = list(w, typed).iter().any(|c| c == code);
    let c = recent(w).iter().any(|c| c == code);
    assert_eq!(
        (a, b, c),
        (a, a, a),
        "{code}: whole list {a}, typed {b}, recent {c}"
    );
    a
}

fn draft_on(date: &str, lots: Vec<LotInput>) -> BikriDraft {
    BikriDraft {
        header: HeaderInput {
            entry_date: date.into(),
            ..support::header()
        },
        lots,
    }
}

fn try_save(w: &World, draft: &BikriDraft) -> Result<BikriSaved, ServiceError> {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: false,
        },
    )
}

/// A lot of own goods from one purchase, sold to one buyer in one line.
fn own_lot_draft(w: &World, purchase_id: Option<i64>, qty: &str) -> BikriDraft {
    let own = id_of(&w.env, OWN_GOODS);
    draft_on(
        "11-10-2026",
        vec![with_sales(
            LotInput {
                purchase_id,
                ..bare_lot(w, "lot-1", own, w.banana, qty)
            },
            vec![sale("s-1", w.akram, qty, "600")],
        )],
    )
}

fn is_rejected(error: &ServiceError, code: ErrorCode, field: &str) -> bool {
    match error {
        ServiceError::Rejected(list) => list
            .iter()
            .any(|r| r.code == code && r.field.as_deref() == Some(field)),
        other => panic!("not a rejection: {other:?}"),
    }
}

// ---- Sellers, پختہ کھاتہ and khatas are always in the list ----

#[test]
fn sellers_and_the_pukhta_seller_are_always_offered_and_buyers_never() {
    let w = world();
    day(&w, 11);
    let codes = list(&w, "");
    // رحیم (0010) and کریم (0011) are sellers; the پختہ کھاتہ (بیوپاری) is one of the sellers of a
    // lot too (0031).
    for seller in ["0010", "0011", "0031"] {
        assert!(codes.iter().any(|c| c == seller), "{seller} in {codes:?}");
    }
    // A buyer, the cash buyer and the پختہ کھاتہ buyer are not sellers.
    assert!(
        !codes.iter().any(|c| c == "0030" || c == "03040000"),
        "{codes:?}"
    );
    // Typed, the seller is found, and the recent list has the same sellers.
    assert_eq!(list(&w, "rahim"), ["0010"]);
    assert!(recent(&w).iter().any(|c| c == "0031"));
    // A trip, a purchase and a garden coming and going change none of this.
    let t = trip(&w);
    buy(&w, t, "20");
    let g = garden(&w);
    close_garden(&w, g.id);
    let after = list(&w, "");
    for seller in ["0010", "0011", "0031"] {
        assert!(after.iter().any(|c| c == seller), "{seller} in {after:?}");
    }
}

#[test]
fn a_separate_khata_that_exists_is_offered() {
    let w = world();
    day(&w, 11);
    let mut words: Vec<String> = Vec::new();
    let made = loop {
        match khata::create(
            &w.env.ctx,
            &w.owner,
            &KhataCreateInput {
                name_ur: "خاندانی کھاتہ".into(),
                name_en: "Family Khata".into(),
                note: None,
                confirmed_rules_words: words.clone(),
            },
        ) {
            Ok(done) => break done,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    };
    assert!(offered(&w, &made.account.code, "family"));
    assert!(list(&w, "خاندانی").contains(&made.account.code));
    // It is a lot-seller only: the seller field of anything else does not offer it.
    let sellers = search::search_accounts(
        &w.env.ctx,
        &w.owner,
        &SearchAccountsInput {
            include_dead: None,
            scope: AccountScope::Seller,
            control_id: None,
            text: String::new(),
            limit: 50,
            offset: 0,
        },
    )
    .unwrap();
    assert!(sellers.iter().all(|h| h.code != made.account.code));
}

// ---- اپنا مال follows the goods left of open trips ----

#[test]
fn own_goods_is_missing_with_no_trip_and_with_a_trip_that_has_no_purchase() {
    let w = world();
    day(&w, 11);
    assert!(!offered(&w, OWN_GOODS, "own"), "no trip at all");
    assert!(!list(&w, OWN_GOODS).iter().any(|c| c == OWN_GOODS));
    trip(&w);
    assert!(!offered(&w, OWN_GOODS, "own"), "a trip with no purchase");
}

#[test]
fn own_goods_is_offered_while_a_purchase_has_goods_left_and_goes_after_the_last_nag_is_sold() {
    let w = world();
    day(&w, 11);
    let t = trip(&w);
    let p = buy(&w, t, "20");
    assert!(offered(&w, OWN_GOODS, "own"), "a purchase with 20 left");
    // It is found by its name in both scripts and by its full code.
    assert!(list(&w, "own").iter().any(|c| c == OWN_GOODS));
    assert!(list(&w, OWN_GOODS).first().is_some_and(|c| c == OWN_GOODS));

    // 15 sold in an own lot: 5 are left, so it stays.
    try_save(&w, &own_lot_draft(&w, Some(p.id), "15")).unwrap();
    assert_eq!(purchase(&w, p.id).qty_left, 5);
    assert!(offered(&w, OWN_GOODS, "own"), "5 left");

    // The last 5 are sold: nothing is left, so the very next search leaves it out.
    try_save(&w, &own_lot_draft(&w, Some(p.id), "5")).unwrap();
    assert_eq!(purchase(&w, p.id).qty_left, 0);
    assert!(!offered(&w, OWN_GOODS, "own"), "the last nag is sold");

    // Another trip with a new purchase brings it back.
    let t2 = trip(&w);
    buy(&w, t2, "10");
    assert!(offered(&w, OWN_GOODS, "own"), "a new purchase");
}

#[test]
fn own_goods_goes_when_the_last_nag_is_written_off_and_comes_back_when_the_write_off_is_undone() {
    let w = world();
    day(&w, 11);
    let t = trip(&w);
    let p = buy(&w, t, "20");
    // A write-off of 12 leaves 8: still offered.
    let first = purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: p.id,
            qty: "12".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
    assert!(offered(&w, OWN_GOODS, "own"), "8 left");
    // The rest is written off: gone.
    let second = purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: p.id,
            qty: "8".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
    assert!(!offered(&w, OWN_GOODS, "own"), "all written off");
    // Undoing the second write-off gives the 8 back, and the account with them.
    purchases::write_off_undo(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffUndoInput {
            move_id: second.move_id,
            reason: "غلطی".into(),
        },
    )
    .unwrap();
    assert!(offered(&w, OWN_GOODS, "own"), "write-off undone");
    let _ = first;
}

#[test]
fn own_goods_goes_with_a_cancelled_purchase_and_with_goods_that_are_all_for_another_shop() {
    let w = world();
    day(&w, 11);
    let t = trip(&w);
    let p = buy(&w, t, "20");
    assert!(offered(&w, OWN_GOODS, "own"));
    purchases::cancel(
        &w.env.ctx,
        &w.owner,
        &PurchaseCancelInput {
            id: p.id,
            version: p.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert!(!offered(&w, OWN_GOODS, "own"), "a cancelled purchase");

    // A purchase whose goods are all for another shop has nothing our lot can take (FR-12.41).
    let all_other = buy_split(&w, t, "10", Some("10"));
    assert_eq!(all_other.qty_left, 10);
    assert!(
        !offered(&w, OWN_GOODS, "own"),
        "10 are left, but all for another shop"
    );
    // One نگ for our shop is enough.
    let mixed = buy_split(&w, t, "10", Some("9"));
    assert_eq!(mixed.qty_left, 10);
    assert!(offered(&w, OWN_GOODS, "own"), "1 for our shop");
}

// ---- A garden is offered only while it is open ----

#[test]
fn a_garden_is_offered_while_open_and_not_when_closed_and_again_when_reopened() {
    let w = world();
    day(&w, 11);
    let g = garden(&w);
    assert!(offered(&w, &g.code, "shahpur"), "an open garden");
    assert!(list(&w, "شاہ").iter().any(|c| c == &g.code));
    close_garden(&w, g.id);
    assert!(!offered(&w, &g.code, "shahpur"), "a closed garden");
    assert!(list(&w, "شاہ").iter().all(|c| c != &g.code));
    gardens::reopen(
        &w.env.ctx,
        &w.owner,
        &GardenReopenInput {
            id: g.id,
            version: garden_version(&w, g.id),
            reason: "غلط دن".into(),
        },
    )
    .unwrap();
    assert!(offered(&w, &g.code, "shahpur"), "reopened");
}

// ---- The list is the database's truth, not the index's ----

#[test]
fn a_stale_index_cannot_offer_a_garden_that_was_closed_or_own_goods_that_ran_out() {
    // The index is built once and a save refreshes it after commit. If it were ever behind (a
    // failed refresh, a restore), the answer must still follow the database. This writes the
    // database directly, so the index in memory is behind on purpose.
    let w = world();
    day(&w, 11);
    let t = trip(&w);
    let p = buy(&w, t, "20");
    let g = garden(&w);
    // Build the index now.
    assert!(offered(&w, OWN_GOODS, "own"));
    assert!(offered(&w, &g.code, "shahpur"));
    {
        let conn = common::raw(&w.dir);
        conn.execute_batch(&format!(
            "PRAGMA foreign_keys = OFF;
             UPDATE purchases SET qty_written_off = qty WHERE id = {};
             UPDATE ventures SET status = 'closed', closed_date = '2026-10-11' WHERE id = {};",
            p.id, g.id
        ))
        .unwrap();
    }
    assert!(
        !offered(&w, OWN_GOODS, "own"),
        "no goods left in the database"
    );
    assert!(!offered(&w, &g.code, "shahpur"), "closed in the database");
}

// ---- The full code typed in the field is still taken; the save refuses what cannot be used ----

#[test]
fn a_typed_full_code_is_still_taken_and_the_save_refuses_an_unusable_own_goods_or_closed_garden() {
    let w = world();
    day(&w, 11);
    // The code of اپنا مال resolves in the lot field as before (the ready goods of a new بکری
    // look it up by this code), even with no goods to sell.
    let resolved = search::resolve_code(
        &w.env.ctx,
        &ResolveCodeInput {
            scope: AccountScope::LotSeller,
            control_id: None,
            text: OWN_GOODS.into(),
        },
    )
    .unwrap();
    assert_eq!(resolved.status, ResolveStatus::Found);

    // No purchase named: the save asks for one at its field.
    let none = try_save(&w, &own_lot_draft(&w, None, "5")).unwrap_err();
    assert!(
        is_rejected(&none, ErrorCode::ErrRequired, "lots[0].purchase_id"),
        "{none:?}"
    );

    // A purchase with nothing left for our shop: the quantity is refused.
    let t = trip(&w);
    let p = buy(&w, t, "20");
    try_save(&w, &own_lot_draft(&w, Some(p.id), "20")).unwrap();
    let sold_out = try_save(&w, &own_lot_draft(&w, Some(p.id), "1")).unwrap_err();
    assert!(
        is_rejected(
            &sold_out,
            ErrorCode::ErrQtyExceedsLot,
            "lots[0].qty_received"
        ),
        "{sold_out:?}"
    );

    // A closed garden: the seller of the lot is refused.
    let g = garden(&w);
    close_garden(&w, g.id);
    let closed = try_save(
        &w,
        &draft_on(
            "11-10-2026",
            vec![with_sales(
                bare_lot(&w, "lot-1", g.account.id, w.banana, "5"),
                vec![sale("s-1", w.akram, "5", "600")],
            )],
        ),
    )
    .unwrap_err();
    assert!(
        is_rejected(
            &closed,
            ErrorCode::ErrGardenClosed,
            "lots[0].seller_account_id"
        ),
        "{closed:?}"
    );
}

// ---- Speed (NFR-SPD-6) ----

#[test]
fn the_lot_seller_list_with_twenty_thousand_sellers_stays_fast() {
    // NFR-SPD-6: under 50 ms on the reference computer in a release build. This runs in a debug
    // build on any computer, so it checks a generous bound; `pnpm perf` holds the real number.
    // The read of the unusable accounts is two indexed lookups and adds a fraction of it.
    const SELLERS: i64 = 20_000;
    const BOUND_MS: u128 = 600;
    let w = world();
    day(&w, 11);
    let t = trip(&w);
    buy(&w, t, "20");
    let g = garden(&w);
    close_garden(&w, g.id);
    {
        let conn = common::raw(&w.dir);
        conn.execute_batch("BEGIN").unwrap();
        let mut statement = conn
            .prepare(
                "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en,
                     search_ur, search_en, search_phonetic)
                 SELECT id, ?1, ?2, 'seller', 'ن', ?3, 'ن', ?4, ?5 FROM account_controls
                 WHERE code = '03011'",
            )
            .unwrap();
        for n in 0..SELLERS {
            let name = format!("seller {n}");
            statement
                .execute((
                    format!("001{}", n + 1_000),
                    n + 1_000,
                    &name,
                    &name,
                    format!("slr{n}"),
                ))
                .unwrap();
        }
        drop(statement);
        conn.execute_batch("COMMIT").unwrap();
    }
    search::rebuild(&w.env.ctx).unwrap();
    let mut slowest = 0;
    for typed in ["", "own", "seller 15000", "selr 77", "1"] {
        let started = Instant::now();
        let hits = list(&w, typed);
        let took = started.elapsed().as_millis();
        slowest = slowest.max(took);
        eprintln!("lot seller {typed:?}: {} hits in {took} ms", hits.len());
        if typed.is_empty() {
            assert_eq!(hits.len(), 50, "the first page of the whole list");
        }
    }
    let started = Instant::now();
    let whole = recent(&w);
    slowest = slowest.max(started.elapsed().as_millis());
    assert!(!whole.is_empty());
    assert!(
        slowest < BOUND_MS,
        "slowest lot seller search took {slowest} ms (bound {BOUND_MS} ms)"
    );
}
