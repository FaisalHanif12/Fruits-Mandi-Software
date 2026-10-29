//! Plan 33 v1.1 (T2): the date to sell on a trip purchase, garden goods arrived (مال آیا), a garden
//! lot linked to an arrival, and the ready goods of a new sale (PRD FR-12.39, FR-12.40, R7 to R12).
//! No amount changes: the money of an own lot and of a garden lot is exactly as before (E2E-34,
//! 34b, 40, 40b, 41 are in `trips.rs` and `golden_day`); here a linked lot is compared with the same
//! lot typed by hand.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_services::core::accounts::{AccountScope, ResolveCodeInput, ResolveStatus};
use mandi_services::core::arrivals::{
    ArrivalCancelInput, ArrivalGetInput, ArrivalInput, ArrivalRow, ArrivalStateFilter,
    ArrivalWriteOffInput, ArrivalWriteOffUndoInput, ArrivalsListInput, SellDateInput,
};
use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, BikriSaved, CancelInput, EditOpenInput, EditSaveInput, HeaderInput,
    LotInput, ReadyGoodsInput, StockListInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{GardenCloseInput, GardenInput, GardenRow};
use mandi_services::core::purchases::{PurchaseInput, PurchaseRow, PurchaseWriteOffInput};
use mandi_services::core::search::SearchScope;
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{MoneySource, VentureKind, VenturePartnerInput};
use mandi_services::{ServiceError, bikri, gardens, purchases, search, trips};
use support::{World, bare_lot, sale, with_sales, world};

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

fn garden(w: &World, name_en: &str, name_ur: &str) -> GardenRow {
    gardens::create(
        &w.env.ctx,
        &w.owner,
        &GardenInput {
            name_ur: name_ur.into(),
            name_en: name_en.into(),
            confirmed_rules_words: vec![
                "شاہ".into(),
                "پور".into(),
                "باغ".into(),
                "ملتان".into(),
                "Shahpur".into(),
                "Garden".into(),
                "Multan".into(),
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

fn shahpur(w: &World) -> GardenRow {
    garden(w, "Shahpur Garden", "شاہ پور باغ")
}

fn arrival_input(garden_id: i64, date: &str, sell: Option<&str>, qty: &str) -> ArrivalInput {
    ArrivalInput {
        garden_id: Some(garden_id),
        date: date.into(),
        sell_date: sell.map(str::to_owned),
        item_id: None,
        packing_id: None,
        qty: qty.into(),
        weight: None,
        note: None,
    }
}

fn arrive(w: &World, garden_id: i64, date: &str, sell: Option<&str>, qty: &str) -> ArrivalRow {
    let mut input = arrival_input(garden_id, date, sell, qty);
    input.item_id = Some(w.banana);
    input.packing_id = Some(w.box_);
    gardens::arrivals::save(&w.env.ctx, &w.owner, &input)
        .unwrap()
        .row
}

fn arrival(w: &World, id: i64) -> ArrivalRow {
    gardens::arrivals::get(&w.env.ctx, &ArrivalGetInput { id }).unwrap()
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

fn buy(w: &World, trip_id: i64, date: &str, sell: Option<&str>, qty: &str) -> PurchaseRow {
    let saved = purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: sell.map(str::to_owned),
            trip_id: Some(trip_id),
            date: date.into(),
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
    purchases::get(
        &w.env.ctx,
        &mandi_services::core::purchases::PurchaseGetInput { id: saved.id },
    )
    .unwrap()
}

fn rejected_codes(error: &ServiceError) -> Vec<(ErrorCode, Option<String>)> {
    match error {
        ServiceError::Rejected(list) => list.iter().map(|r| (r.code, r.field.clone())).collect(),
        other => panic!("not a rejection: {other:?}"),
    }
}

fn is_rejected(error: &ServiceError, code: ErrorCode, field: &str) -> bool {
    rejected_codes(error).contains(&(code, Some(field.to_owned())))
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

/// A garden lot with no charges at all, linked to an arrival (or not) and selling `qty` at 600.
fn garden_lot(
    w: &World,
    key: &str,
    garden: &GardenRow,
    arrival_id: Option<i64>,
    qty: &str,
) -> LotInput {
    with_sales(
        LotInput {
            arrival_id,
            ..bare_lot(w, key, garden.account.id, w.banana, qty)
        },
        vec![sale(&format!("{key}-s1"), w.akram, qty, "600")],
    )
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

fn ready(w: &World, date: &str) -> Vec<mandi_services::core::bikri::ReadyGoodsRow> {
    bikri::ready_goods(&w.env.ctx, &ReadyGoodsInput { date: date.into() })
        .unwrap()
        .rows
}

// ---- The lead's check ----

#[test]
fn the_own_goods_account_is_found_by_its_full_code_in_a_lot_seller_box() {
    let w = world();
    let found = search::resolve_code(
        &w.env.ctx,
        &ResolveCodeInput {
            scope: AccountScope::LotSeller,
            control_id: None,
            text: "030119998".into(),
        },
    )
    .unwrap();
    assert_eq!(found.status, ResolveStatus::Found);
    assert_eq!(found.account.expect("the account").code, "030119998");
}

// ---- R7: the date to sell on a trip purchase ----

#[test]
fn a_purchase_starts_with_its_own_day_as_the_day_to_sell_and_it_can_move_later_but_not_earlier() {
    let w = world();
    day(&w, 11);
    let t = trip(&w);
    let first = buy(&w, t, "11-10-2026", None, "20");
    assert_eq!(first.sell_date, "2026-10-11");
    assert!(first.can_change_sell_date);
    let later = buy(&w, t, "11-10-2026", Some("13-10-2026"), "10");
    assert_eq!(later.sell_date, "2026-10-13");

    // Earlier than the purchase day is refused at its field, on the save and on the change.
    let early = purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: Some("10-10-2026".into()),
            trip_id: Some(t),
            date: "11-10-2026".into(),
            supplier: None,
            item_id: Some(w.banana),
            packing_id: Some(w.box_),
            qty: "5".into(),
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
    .unwrap_err();
    assert!(is_rejected(
        &early,
        ErrorCode::ErrSellDateBeforeEntry,
        "sell_date"
    ));
    let refused = purchases::set_sell_date(
        &w.env.ctx,
        &w.owner,
        &SellDateInput {
            id: first.id,
            version: first.version,
            sell_date: "10-10-2026".into(),
        },
    )
    .unwrap_err();
    assert!(is_rejected(
        &refused,
        ErrorCode::ErrSellDateBeforeEntry,
        "sell_date"
    ));

    // A later day is allowed while goods are left; the version moves on.
    let set = purchases::set_sell_date(
        &w.env.ctx,
        &w.owner,
        &SellDateInput {
            id: first.id,
            version: first.version,
            sell_date: "12-10-2026".into(),
        },
    )
    .unwrap();
    assert_eq!(set.sell_date, "2026-10-12");
    assert!(set.version > first.version);
    // An old version is refused.
    let stale = purchases::set_sell_date(
        &w.env.ctx,
        &w.owner,
        &SellDateInput {
            id: first.id,
            version: first.version,
            sell_date: "14-10-2026".into(),
        },
    )
    .unwrap_err();
    assert!(is_rejected(&stale, ErrorCode::ErrVersionChanged, "version"));

    // With nothing left the day has no use.
    purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: first.id,
            qty: "20".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
    let after = purchases::get(
        &w.env.ctx,
        &mandi_services::core::purchases::PurchaseGetInput { id: first.id },
    )
    .unwrap();
    assert!(!after.can_change_sell_date);
    let none_left = purchases::set_sell_date(
        &w.env.ctx,
        &w.owner,
        &SellDateInput {
            id: first.id,
            version: after.version,
            sell_date: "14-10-2026".into(),
        },
    )
    .unwrap_err();
    assert!(is_rejected(&none_left, ErrorCode::ErrNothingToSell, "id"));
}

// ---- R9, R11: garden goods arrived ----

#[test]
fn an_arrival_is_a_stock_record_with_its_own_number_and_posts_no_money() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let cash_before = w
        .env
        .ctx
        .db()
        .read(|c| mandi_db::ledger::account_totals(c, None))
        .unwrap();
    let first = arrive(&w, g.id, "11-10-2026", None, "100");
    let second = arrive(&w, g.id, "11-10-2026", Some("12-10-2026"), "40");
    assert_eq!((first.number, second.number), (1, 2));
    assert_eq!(first.garden.code, "0080");
    assert_eq!(first.garden.account.id, g.account.id);
    assert_eq!(first.sell_date, "2026-10-11");
    assert_eq!(second.sell_date, "2026-10-12");
    assert_eq!((first.qty, first.qty_left, first.qty_sold), (100, 100, 0));
    assert!(first.can_cancel && first.can_change_sell_date);
    // No journal entry: the books are exactly as they were.
    let cash_after = w
        .env
        .ctx
        .db()
        .read(|c| mandi_db::ledger::account_totals(c, None))
        .unwrap();
    assert_eq!(cash_before, cash_after);

    let list = gardens::arrivals::list(
        &w.env.ctx,
        &ArrivalsListInput {
            garden_id: Some(g.id),
            state: Some(ArrivalStateFilter::InStock),
            ..ArrivalsListInput::default()
        },
    )
    .unwrap();
    assert_eq!(list.rows.len(), 2);
    assert_eq!(list.total_count, 2);
    assert_eq!(
        (list.totals.qty, list.totals.qty_left, list.totals.qty_sold),
        (140, 140, 0)
    );
}

#[test]
fn an_arrival_checks_its_garden_its_dates_and_its_fields() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let mut input = arrival_input(g.id, "11-10-2026", Some("10-10-2026"), "100");
    input.item_id = Some(w.banana);
    input.packing_id = Some(w.box_);
    let e = gardens::arrivals::save(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert!(is_rejected(
        &e,
        ErrorCode::ErrSellDateBeforeEntry,
        "sell_date"
    ));
    // Before the garden's own day.
    let mut input = arrival_input(g.id, "09-10-2026", None, "100");
    input.item_id = Some(w.banana);
    input.packing_id = Some(w.box_);
    let e = gardens::arrivals::save(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert!(is_rejected(&e, ErrorCode::ErrDateBeforeGarden, "date"));
    // Missing item and packing, a bad qty.
    let e = gardens::arrivals::save(
        &w.env.ctx,
        &w.owner,
        &arrival_input(g.id, "11-10-2026", None, "0"),
    )
    .unwrap_err();
    let codes = rejected_codes(&e);
    assert!(codes.contains(&(ErrorCode::ErrRequired, Some("item_id".into()))));
    assert!(codes.contains(&(ErrorCode::ErrRequired, Some("packing_id".into()))));
    assert!(codes.iter().any(|(_, f)| f.as_deref() == Some("qty")));
    // No garden.
    let mut input = arrival_input(g.id, "11-10-2026", None, "5");
    input.garden_id = None;
    let e = gardens::arrivals::save(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert!(is_rejected(&e, ErrorCode::ErrRequired, "garden_id"));
}

#[test]
fn an_arrival_is_cancelled_with_a_reason_only_while_untouched_and_a_write_off_can_be_undone() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let a = arrive(&w, g.id, "11-10-2026", None, "100");
    let no_reason = gardens::arrivals::cancel(
        &w.env.ctx,
        &w.owner,
        &ArrivalCancelInput {
            id: a.id,
            version: a.version,
            reason: " ".into(),
        },
    )
    .unwrap_err();
    assert!(is_rejected(
        &no_reason,
        ErrorCode::ErrReasonRequired,
        "reason"
    ));

    // A write-off takes goods out; asking for more than is left is refused with what is available.
    let off = |qty: &str| {
        gardens::arrivals::write_off(
            &w.env.ctx,
            &w.owner,
            &ArrivalWriteOffInput {
                arrival_id: a.id,
                qty: qty.into(),
                reason: "خراب".into(),
            },
        )
    };
    let too_many = off("101").unwrap_err();
    assert!(is_rejected(
        &too_many,
        ErrorCode::ErrWriteOffExceedsBalance,
        "qty"
    ));
    let done = off("30").unwrap();
    assert_eq!((done.qty_written_off, done.qty_left), (30, 70));
    let touched = arrival(&w, a.id);
    assert_eq!(touched.write_offs.len(), 1);
    assert!(!touched.can_cancel, "a write-off stands on it");
    let e = gardens::arrivals::cancel(
        &w.env.ctx,
        &w.owner,
        &ArrivalCancelInput {
            id: a.id,
            version: touched.version,
            reason: "غلط".into(),
        },
    )
    .unwrap_err();
    assert!(is_rejected(&e, ErrorCode::ErrArrivalHasSales, "id"));

    // The undo gives the goods back; then the arrival is untouched again and can be cancelled.
    let undone = gardens::arrivals::write_off_undo(
        &w.env.ctx,
        &w.owner,
        &ArrivalWriteOffUndoInput {
            move_id: done.move_id,
            reason: "غلطی".into(),
        },
    )
    .unwrap();
    assert_eq!((undone.qty_written_off, undone.qty_left), (0, 100));
    let again = arrival(&w, a.id);
    assert!(again.can_cancel);
    assert!(again.write_offs.is_empty());
    let cancelled = gardens::arrivals::cancel(
        &w.env.ctx,
        &w.owner,
        &ArrivalCancelInput {
            id: a.id,
            version: again.version,
            reason: "غلط اندراج".into(),
        },
    )
    .unwrap();
    assert_eq!(cancelled.number, 1);
    let gone = arrival(&w, a.id);
    assert_eq!(gone.qty_left, 0);
    assert_eq!(gone.cancel_reason.as_deref(), Some("غلط اندراج"));
    // It is not offered for a new sale, and cancelling it twice is refused.
    assert!(ready(&w, "11-10-2026").is_empty());
    let twice = gardens::arrivals::cancel(
        &w.env.ctx,
        &w.owner,
        &ArrivalCancelInput {
            id: a.id,
            version: gone.version,
            reason: "x".into(),
        },
    )
    .unwrap_err();
    assert!(is_rejected(&twice, ErrorCode::ErrArrivalCancelled, "id"));
}

#[test]
fn the_day_to_sell_of_an_arrival_moves_while_goods_are_left() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let a = arrive(&w, g.id, "11-10-2026", None, "100");
    let set = gardens::arrivals::set_sell_date(
        &w.env.ctx,
        &w.owner,
        &SellDateInput {
            id: a.id,
            version: a.version,
            sell_date: "15-10-2026".into(),
        },
    )
    .unwrap();
    assert_eq!(set.sell_date, "2026-10-15");
    // The goods wait: a new sale of 12-10 does not get them, one of 15-10 does.
    assert!(ready(&w, "12-10-2026").is_empty());
    assert_eq!(ready(&w, "15-10-2026").len(), 1);
    let early = gardens::arrivals::set_sell_date(
        &w.env.ctx,
        &w.owner,
        &SellDateInput {
            id: a.id,
            version: set.version,
            sell_date: "10-10-2026".into(),
        },
    )
    .unwrap_err();
    assert!(is_rejected(
        &early,
        ErrorCode::ErrSellDateBeforeEntry,
        "sell_date"
    ));
}

#[test]
fn a_garden_with_goods_left_is_not_closed_and_a_closed_garden_takes_no_arrival() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let a = arrive(&w, g.id, "11-10-2026", None, "100");
    let close = |version: i64| {
        gardens::close(
            &w.env.ctx,
            &w.owner,
            &GardenCloseInput {
                id: g.id,
                version,
                date: "11-10-2026".into(),
            },
        )
    };
    let fresh = gardens::get(
        &w.env.ctx,
        &mandi_services::core::gardens::GardenGetInput {
            id: g.id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap();
    let refused = close(fresh.row.version).unwrap_err();
    assert!(is_rejected(&refused, ErrorCode::ErrGardenHasStock, "id"));
    // Written off for good, the garden closes.
    let off = gardens::arrivals::write_off(
        &w.env.ctx,
        &w.owner,
        &ArrivalWriteOffInput {
            arrival_id: a.id,
            qty: "100".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
    close(fresh.row.version).unwrap();
    // A closed garden takes no new arrival, and a write-off in it is not undone.
    let mut input = arrival_input(g.id, "11-10-2026", None, "5");
    input.item_id = Some(w.banana);
    input.packing_id = Some(w.box_);
    let e = gardens::arrivals::save(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert!(is_rejected(&e, ErrorCode::ErrGardenClosed, "garden_id"));
    let e = gardens::arrivals::write_off_undo(
        &w.env.ctx,
        &w.owner,
        &ArrivalWriteOffUndoInput {
            move_id: off.move_id,
            reason: "x".into(),
        },
    )
    .unwrap_err();
    assert!(is_rejected(&e, ErrorCode::ErrGardenClosed, "move_id"));
}

// ---- R8: the goods a new sale opens with ----

#[test]
fn ready_goods_are_both_kinds_oldest_day_to_sell_first_and_only_on_or_before_the_date() {
    let w = world();
    day(&w, 12);
    let t = trip(&w);
    let g = shahpur(&w);
    let p1 = buy(&w, t, "11-10-2026", Some("12-10-2026"), "20");
    let a1 = arrive(&w, g.id, "11-10-2026", Some("11-10-2026"), "100");
    let p2 = buy(&w, t, "11-10-2026", Some("14-10-2026"), "10");
    let a2 = arrive(&w, g.id, "12-10-2026", None, "7");

    let rows = ready(&w, "12-10-2026");
    let shown: Vec<(VentureKind, i64, &str)> = rows
        .iter()
        .map(|r| (r.kind, r.id, r.sell_date.as_str()))
        .collect();
    assert_eq!(
        shown,
        [
            (VentureKind::Garden, a1.id, "2026-10-11"),
            (VentureKind::Trip, p1.id, "2026-10-12"),
            (VentureKind::Garden, a2.id, "2026-10-12"),
        ],
        "oldest day to sell first, then number; the purchase of 14-10 waits"
    );
    let first = &rows[0];
    assert_eq!((first.qty, first.qty_left), (100, 100));
    assert_eq!(
        first.garden.as_ref().map(|g| g.account.id),
        Some(g.account.id)
    );
    assert!(first.trip.is_none());
    assert_eq!(rows[1].trip.as_ref().map(|t| t.code.as_str()), Some("0070"));
    assert_eq!(rows[1].item.id, w.banana);
    // The purchase of 14-10 comes with that day.
    let later = ready(&w, "14-10-2026");
    assert_eq!(later.len(), 4);
    assert!(
        later
            .iter()
            .any(|r| r.kind == VentureKind::Trip && r.id == p2.id)
    );
    // A date before everything lists nothing.
    assert!(ready(&w, "10-10-2026").is_empty());
}

#[test]
fn goods_not_sold_stay_in_the_arrival_and_come_again_into_the_next_new_sale() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let a = arrive(&w, g.id, "11-10-2026", None, "100");

    // 60 of the 100 are sold on 11-10 in a lot linked to the arrival.
    let saved = try_save(
        &w,
        &draft_on(
            "11-10-2026",
            vec![garden_lot(&w, "lot-1", &g, Some(a.id), "60")],
        ),
    )
    .unwrap();
    let after = arrival(&w, a.id);
    assert_eq!((after.qty_sold, after.qty_left), (60, 40));
    assert_eq!(after.lots.len(), 1);
    assert_eq!(after.lots[0].qty_sold, 60);
    assert_eq!(after.lots[0].bikri_number, saved.number);
    assert!(!after.can_cancel, "a sale stands on it");

    // The next day's new sale gets the 40 that are left.
    day(&w, 12);
    let rows = ready(&w, "12-10-2026");
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].id, rows[0].qty_left), (a.id, 40));
    // Written off for good, nothing comes again.
    day(&w, 12);
    gardens::arrivals::write_off(
        &w.env.ctx,
        &w.owner,
        &ArrivalWriteOffInput {
            arrival_id: a.id,
            qty: "40".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
    assert!(ready(&w, "12-10-2026").is_empty());
}

// ---- R10: a garden lot linked to an arrival ----

#[test]
fn a_linked_garden_lot_has_exactly_the_money_of_the_same_lot_typed_by_hand() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let a = arrive(&w, g.id, "11-10-2026", None, "100");
    let linked = draft_on(
        "11-10-2026",
        vec![garden_lot(&w, "lot-1", &g, Some(a.id), "60")],
    );
    let by_hand = draft_on("11-10-2026", vec![garden_lot(&w, "lot-1", &g, None, "60")]);
    let p_linked = support::preview(&w, &linked);
    let p_hand = support::preview(&w, &by_hand);
    assert!(p_linked.errors.is_empty(), "{:?}", p_linked.errors);
    assert!(p_hand.errors.is_empty(), "{:?}", p_hand.errors);
    assert_eq!(p_linked.summary, p_hand.summary);
    assert_eq!(p_linked.split, p_hand.split);
    assert_eq!(p_linked.cash, p_hand.cash);
    // The preview names the arrival and what it has left for the lot.
    let shown = p_linked.lots[0].arrival.as_ref().expect("the arrival");
    assert_eq!(
        (shown.arrival_id, shown.arrival_number, shown.qty_left),
        (a.id, 1, 100)
    );
    assert_eq!(shown.garden.code, "0080");
    assert!(p_hand.lots[0].arrival.is_none());
    // Saving both ways gives the same journal totals; only the stock differs.
    let saved = try_save(&w, &linked).unwrap();
    assert_eq!(saved.preview.summary, p_hand.summary);
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let lot = &detail.lots[0];
    assert_eq!(
        lot.arrival.as_ref().map(|r| (r.id, r.number)),
        Some((a.id, 1))
    );
    assert_eq!(
        lot.arrival.as_ref().map(|r| r.garden_code.as_str()),
        Some("0080")
    );
    assert_eq!(lot.seller.id, g.account.id);
}

#[test]
fn a_linked_lot_must_be_sold_in_full_and_may_not_take_more_than_the_arrival_has() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let a = arrive(&w, g.id, "11-10-2026", None, "100");

    // Received 60 but 50 sold: the rest stays in the arrival, so the lot is not accepted.
    let mut lot = garden_lot(&w, "lot-1", &g, Some(a.id), "60");
    lot.sales = vec![sale("s-1", w.akram, "50", "600")];
    let p = support::preview(&w, &draft_on("11-10-2026", vec![lot]));
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrArrivalLotNotFullySold,
        "lots[0].qty_received"
    ));
    // A hand-typed garden lot with the same lines is only a warning, as before.
    let mut hand = garden_lot(&w, "lot-1", &g, None, "60");
    hand.sales = vec![sale("s-1", w.akram, "50", "600")];
    let p = support::preview(&w, &draft_on("11-10-2026", vec![hand]));
    assert!(p.errors.is_empty(), "{:?}", p.errors);

    // More than the arrival has.
    let p = support::preview(
        &w,
        &draft_on(
            "11-10-2026",
            vec![garden_lot(&w, "lot-1", &g, Some(a.id), "101")],
        ),
    );
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrQtyExceedsLot,
        "lots[0].qty_received"
    ));

    // Two lots of one draft share the arrival: 60 and 60 are more than 100; the second is told 40.
    let two = draft_on(
        "11-10-2026",
        vec![
            garden_lot(&w, "lot-1", &g, Some(a.id), "60"),
            garden_lot(&w, "lot-2", &g, Some(a.id), "60"),
        ],
    );
    let p = support::preview(&w, &two);
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrQtyExceedsLot,
        "lots[1].qty_received"
    ));
    let shown: Vec<i64> = p
        .lots
        .iter()
        .map(|l| l.arrival.as_ref().map_or(-1, |a| a.qty_left))
        .collect();
    assert_eq!(shown, [100, 40]);
}

#[test]
fn a_linked_lot_checks_its_seller_its_goods_and_its_day() {
    let w = world();
    day(&w, 12);
    let g = shahpur(&w);
    let other = garden(&w, "Multan Garden", "ملتان باغ");
    let a = arrive(&w, g.id, "11-10-2026", None, "100");

    // The seller must be the arrival's garden.
    let p = support::preview(
        &w,
        &draft_on(
            "12-10-2026",
            vec![garden_lot(&w, "lot-1", &other, Some(a.id), "10")],
        ),
    );
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrFormatInvalid,
        "lots[0].arrival_id"
    ));
    // Only a garden's lot may name an arrival.
    let p = support::preview(
        &w,
        &draft_on(
            "12-10-2026",
            vec![with_sales(
                LotInput {
                    arrival_id: Some(a.id),
                    ..bare_lot(&w, "lot-1", w.rahim, w.banana, "10")
                },
                vec![sale("s-1", w.akram, "10", "600")],
            )],
        ),
    );
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrFormatInvalid,
        "lots[0].arrival_id"
    ));
    // A different item is refused; an empty one is filled from the arrival.
    let mut lot = garden_lot(&w, "lot-1", &g, Some(a.id), "10");
    lot.item_id = Some(w.apple);
    let p = support::preview(&w, &draft_on("12-10-2026", vec![lot]));
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrFormatInvalid,
        "lots[0].item_id"
    ));
    let mut lot = garden_lot(&w, "lot-1", &g, Some(a.id), "10");
    lot.item_id = None;
    lot.packing_id = None;
    let p = support::preview(&w, &draft_on("12-10-2026", vec![lot]));
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    // Not before the day it arrived.
    day(&w, 11);
    let p = support::preview(
        &w,
        &draft_on(
            "10-10-2026",
            vec![garden_lot(&w, "lot-1", &g, Some(a.id), "10")],
        ),
    );
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrDateBeforeReceived,
        "lots[0].arrival_id"
    ));
    // A cancelled arrival gives nothing.
    day(&w, 12);
    let cancelled = gardens::arrivals::cancel(
        &w.env.ctx,
        &w.owner,
        &ArrivalCancelInput {
            id: a.id,
            version: a.version,
            reason: "غلط".into(),
        },
    )
    .unwrap();
    assert_eq!(cancelled.id, a.id);
    let p = support::preview(
        &w,
        &draft_on(
            "12-10-2026",
            vec![garden_lot(&w, "lot-1", &g, Some(a.id), "10")],
        ),
    );
    assert!(support::has(
        &p.errors,
        ErrorCode::ErrArrivalCancelled,
        "lots[0].arrival_id"
    ));
}

// ---- Two open sales on the same goods can never oversell ----

#[test]
fn two_open_sales_on_the_same_arrival_cannot_oversell_and_the_second_is_refused_on_qty() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let a = arrive(&w, g.id, "11-10-2026", None, "100");
    // Both screens opened with the same 100 and both previews are clean.
    let first = draft_on(
        "11-10-2026",
        vec![garden_lot(&w, "lot-1", &g, Some(a.id), "60")],
    );
    let second = draft_on(
        "11-10-2026",
        vec![garden_lot(&w, "lot-1", &g, Some(a.id), "60")],
    );
    assert!(support::preview(&w, &first).errors.is_empty());
    assert!(support::preview(&w, &second).errors.is_empty());
    try_save(&w, &first).unwrap();
    // The second save reads what is left inside its own transaction: refused, with the number left.
    let error = try_save(&w, &second).unwrap_err();
    assert!(
        is_rejected(&error, ErrorCode::ErrQtyExceedsLot, "lots[0].qty_received"),
        "{error:?}"
    );
    let after = arrival(&w, a.id);
    assert_eq!(
        (after.qty_sold, after.qty_left),
        (60, 40),
        "nothing of the second was written"
    );
    assert_eq!(after.lots.len(), 1);
}

#[test]
fn two_open_sales_on_the_same_trip_purchase_cannot_oversell_either() {
    let w = world();
    day(&w, 11);
    let t = trip(&w);
    let p = buy(&w, t, "11-10-2026", None, "20");
    let own = w
        .env
        .ctx
        .db()
        .read(|c| mandi_db::accounts::id_by_code(c, "030119998"))
        .unwrap()
        .unwrap();
    let make = || {
        draft_on(
            "11-10-2026",
            vec![with_sales(
                LotInput {
                    purchase_id: Some(p.id),
                    ..bare_lot(&w, "lot-1", own, w.banana, "15")
                },
                vec![sale("s-1", w.akram, "15", "600")],
            )],
        )
    };
    try_save(&w, &make()).unwrap();
    let error = try_save(&w, &make()).unwrap_err();
    assert!(
        is_rejected(&error, ErrorCode::ErrQtyExceedsLot, "lots[0].qty_received"),
        "{error:?}"
    );
    let after = purchases::get(
        &w.env.ctx,
        &mandi_services::core::purchases::PurchaseGetInput { id: p.id },
    )
    .unwrap();
    assert_eq!((after.qty_sold, after.qty_left), (15, 5));
}

// ---- Edit and cancel give the goods back ----

#[test]
fn cancelling_or_editing_a_sale_gives_the_arrival_its_goods_back() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let a = arrive(&w, g.id, "11-10-2026", None, "100");
    let saved = try_save(
        &w,
        &draft_on(
            "11-10-2026",
            vec![garden_lot(&w, "lot-1", &g, Some(a.id), "60")],
        ),
    )
    .unwrap();
    assert_eq!(arrival(&w, a.id).qty_left, 40);

    // An edit that sells 50 instead of 60: the 60 go back first, then the 50 are taken.
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    assert_eq!(opened.draft.lots[0].arrival_id, Some(a.id));
    let mut edited = opened.draft.clone();
    edited.lots[0].qty_received = "50".into();
    edited.lots[0].sales[0].qty = "50".into();
    let preview = bikri::edit_preview(
        &w.env.ctx,
        &mandi_services::core::bikri::EditPreviewInput {
            entry_id: opened.entry_id,
            draft: edited.clone(),
        },
    )
    .unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert_eq!(
        preview.lots[0].arrival.as_ref().map(|a| a.qty_left),
        Some(100)
    );
    let done = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft: edited,
            reason: Some("تعداد کی درستی".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert!(done.preview.errors.is_empty());
    let after_edit = arrival(&w, a.id);
    assert_eq!((after_edit.qty_sold, after_edit.qty_left), (50, 50));

    // The link of a saved lot cannot change in an edit.
    let reopened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: done.entry_id,
        },
    )
    .unwrap();
    let mut unlinked = reopened.draft.clone();
    unlinked.lots[0].arrival_id = None;
    let refused = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: reopened.entry_id,
            version: reopened.version,
            draft: unlinked,
            reason: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert!(
        rejected_codes(&refused)
            .iter()
            .any(|(_, f)| f.as_deref() == Some("lots[0].arrival_id")),
        "{refused:?}"
    );

    // Cancelling the sale gives all the goods back, and the arrival can be cancelled again.
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: saved.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let after_cancel = arrival(&w, a.id);
    assert_eq!((after_cancel.qty_sold, after_cancel.qty_left), (0, 100));
    assert!(after_cancel.lots.is_empty());
    assert!(after_cancel.can_cancel);
}

#[test]
fn the_commission_on_the_goods_of_an_arrival_and_its_garden_takes_a_cancelled_sale_out() {
    // Plan 36 (PRD FR-12.37): the shop's commission on goods sold in our shop = seller commission +
    // buyer commission + لاگا of the stored sale lines, reversals included. 60 at 600 = 36,000:
    // seller 6% 2,160, buyer 6% 2,160, لاگا 60 × 8 = 480, so 4,800.
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let a = arrive(&w, g.id, "11-10-2026", None, "100");
    let garden_commission = |w: &World| {
        gardens::get(
            &w.env.ctx,
            &mandi_services::core::gardens::GardenGetInput {
                id: g.id,
                history_offset: 0,
                history_limit: 0,
            },
        )
        .unwrap()
        .row
    };
    // Nothing sold: 0, with a working.
    assert_eq!(arrival(&w, a.id).commission_paisa.rupees(), 0);
    let saved = try_save(
        &w,
        &draft_on(
            "11-10-2026",
            vec![garden_lot(&w, "lot-1", &g, Some(a.id), "60")],
        ),
    )
    .unwrap();
    let row = arrival(&w, a.id);
    assert_eq!(row.commission_paisa.rupees(), 4_800);
    let args: Vec<(String, i64)> = row
        .commission_working
        .args
        .iter()
        .map(|arg| match &arg.value {
            mandi_services::core::rules::WorkingValue::Money { paisa } => {
                (arg.name.clone(), paisa.rupees())
            }
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        args,
        vec![
            ("seller".to_owned(), 2_160),
            ("buyer".to_owned(), 2_160),
            ("laga".to_owned(), 480),
            ("total".to_owned(), 4_800)
        ]
    );
    // The garden shows the same, and so do the totals of both lists.
    assert_eq!(garden_commission(&w).commission_paisa.rupees(), 4_800);
    let garden_list = gardens::list(
        &w.env.ctx,
        &mandi_services::core::gardens::GardensListInput::default(),
    )
    .unwrap();
    assert_eq!(garden_list.totals.commission_paisa.rupees(), 4_800);
    let arrival_list = gardens::arrivals::list(
        &w.env.ctx,
        &mandi_services::core::arrivals::ArrivalsListInput::default(),
    )
    .unwrap();
    assert_eq!(arrival_list.totals.commission_paisa.rupees(), 4_800);

    // A lot of the garden sold with no link to an arrival is in the garden's commission and in no
    // arrival's: 10 at 600 = 6,000 → 360 + 360 + 80 = 800.
    try_save(
        &w,
        &draft_on("11-10-2026", vec![garden_lot(&w, "lot-2", &g, None, "10")]),
    )
    .unwrap();
    assert_eq!(arrival(&w, a.id).commission_paisa.rupees(), 4_800);
    assert_eq!(garden_commission(&w).commission_paisa.rupees(), 5_600);

    // Cancelling the first sale takes its commission out again (the reversal facts, sign −1).
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: saved.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(arrival(&w, a.id).commission_paisa.rupees(), 0);
    assert_eq!(garden_commission(&w).commission_paisa.rupees(), 800);
}

// ---- باقی مال and the filters ----

#[test]
fn the_stock_list_shows_the_arrivals_with_goods_left() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let t = trip(&w);
    let a = arrive(&w, g.id, "11-10-2026", Some("12-10-2026"), "100");
    buy(&w, t, "11-10-2026", Some("13-10-2026"), "20");
    let list = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    assert_eq!(list.garden_rows.len(), 1);
    let row = &list.garden_rows[0];
    assert_eq!((row.arrival_id, row.number, row.balance), (a.id, 1, 100));
    assert_eq!(row.sell_date, "2026-10-12");
    assert_eq!(row.garden.code, "0080");
    assert_eq!(list.garden_balance_total, 100);
    assert_eq!(list.own_rows[0].sell_date, "2026-10-13");
    assert_eq!(list.total_count, 2);
    // The seller filter keeps the garden's own arrivals and drops the rest.
    let mine = bikri::stock_list(
        &w.env.ctx,
        &StockListInput {
            seller_id: Some(g.account.id),
            ..StockListInput::default()
        },
    )
    .unwrap();
    assert_eq!(mine.garden_rows.len(), 1);
    assert!(mine.own_rows.is_empty());
    let other = bikri::stock_list(
        &w.env.ctx,
        &StockListInput {
            seller_id: Some(w.rahim),
            ..StockListInput::default()
        },
    )
    .unwrap();
    assert!(other.garden_rows.is_empty());
}

#[test]
fn entry_pickers_offer_open_trips_and_gardens_and_filter_pickers_offer_the_closed_ones_too() {
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let fresh = gardens::get(
        &w.env.ctx,
        &mandi_services::core::gardens::GardenGetInput {
            id: g.id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap();
    let hits = |scope: SearchScope| -> Vec<String> {
        search::search_page(&w.env.ctx, &w.owner, scope, "", 0, 50)
            .unwrap()
            .into_iter()
            .map(|h| h.code)
            .collect()
    };
    assert_eq!(hits(SearchScope::Garden), ["0080"]);
    assert_eq!(hits(SearchScope::GardenAll), ["0080"]);
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &GardenCloseInput {
            id: g.id,
            version: fresh.row.version,
            date: "11-10-2026".into(),
        },
    )
    .unwrap();
    assert!(
        hits(SearchScope::Garden).is_empty(),
        "an entry field offers open gardens only"
    );
    assert_eq!(
        hits(SearchScope::GardenAll),
        ["0080"],
        "a filter offers the closed one too"
    );
    // A short code works in the filter scope too (`0` finds 0080).
    let short =
        search::search_page(&w.env.ctx, &w.owner, SearchScope::GardenAll, "0", 0, 50).unwrap();
    assert_eq!(short.first().map(|h| h.code.as_str()), Some("0080"));
    let _ = trip(&w);
    assert_eq!(hits(SearchScope::Trip), ["0070"]);
    assert_eq!(hits(SearchScope::TripAll), ["0070"]);
}

// ---- Decision D2: the arrivals count in the stock tile and on the bell ----

fn no_backup() -> mandi_reports::dashboard::BackupFacts {
    mandi_reports::dashboard::BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    }
}

fn waiting(w: &World) -> Option<mandi_reports::dashboard::AttentionItem> {
    mandi_reports::dashboard::attention(w.env.ctx.db(), no_backup())
        .unwrap()
        .items
        .into_iter()
        .find(|i| i.kind == mandi_reports::dashboard::AttentionKind::StockWaiting)
}

#[test]
fn arrivals_count_in_the_stock_tile_and_in_the_day_close_as_of_the_day() {
    use mandi_services::core::vouchers::DayClosePreviewInput;
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let a = arrive(&w, g.id, "11-10-2026", None, "100");
    let tiles = |w: &World| {
        mandi_reports::dashboard::get(w.env.ctx.db(), no_backup())
            .unwrap()
            .tiles
    };
    let t = tiles(&w);
    assert_eq!((t.stock_lots, t.stock_qty), (1, 100));

    // 60 are sold on 12-10: the tile shows the 40 left.
    day(&w, 12);
    try_save(
        &w,
        &draft_on(
            "12-10-2026",
            vec![garden_lot(&w, "lot-1", &g, Some(a.id), "60")],
        ),
    )
    .unwrap();
    let t = tiles(&w);
    assert_eq!((t.stock_lots, t.stock_qty), (1, 40));
    // The day close is worked AS OF the day: 11-10 still had all 100, 12-10 had 40.
    let as_of = |up_to: &str| {
        let p = mandi_services::day::close_preview(
            &w.env.ctx,
            &DayClosePreviewInput {
                up_to: up_to.into(),
            },
        )
        .unwrap();
        (p.summary.open_lots, p.summary.open_qty)
    };
    assert_eq!(as_of("11-10-2026"), (1, 100));
    assert_eq!(as_of("12-10-2026"), (1, 40));
    // Written off for good, the arrival leaves the tile.
    gardens::arrivals::write_off(
        &w.env.ctx,
        &w.owner,
        &ArrivalWriteOffInput {
            arrival_id: a.id,
            qty: "40".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
    let t = tiles(&w);
    assert_eq!((t.stock_lots, t.stock_qty), (0, 0));
}

#[test]
fn goods_wait_on_the_bell_from_their_day_to_sell_and_not_from_the_day_they_came() {
    use mandi_reports::dashboard::AttentionTarget;
    let w = world();
    day(&w, 11);
    let g = shahpur(&w);
    let t = trip(&w);
    // An arrival and a purchase of 11-10 to sell on 11-10, and one of each to sell on 20-10.
    let due = arrive(&w, g.id, "11-10-2026", None, "100");
    let late = arrive(&w, g.id, "11-10-2026", Some("20-10-2026"), "50");
    let due_purchase = buy(&w, t, "11-10-2026", None, "20");
    let late_purchase = buy(&w, t, "11-10-2026", Some("20-10-2026"), "10");

    // 13-10: two days after the day to sell is not more than two days.
    day(&w, 13);
    assert!(waiting(&w).is_none());
    // 14-10: the two goods of 11-10 have waited three days; the ones for 20-10 have not waited.
    day(&w, 14);
    let item = waiting(&w).expect("goods waiting");
    assert_eq!(item.count, 2);
    assert_eq!(item.qty, Some(100 + 20));
    assert_eq!(item.days, Some(3));
    // On a tie of age the bigger balance is first: the arrival of 100.
    let first = item.first.expect("the first record");
    assert_eq!((first.target, first.id), (AttentionTarget::Arrival, due.id));

    // The rows of the screens count the days the same way.
    let row = arrival(&w, late.id);
    assert_eq!(row.days_waiting, 0, "its day to sell is in the future");
    assert_eq!(arrival(&w, due.id).days_waiting, 3);
    let list = bikri::stock_list(&w.env.ctx, &StockListInput::default()).unwrap();
    let by_arrival = |id: i64| {
        list.garden_rows
            .iter()
            .find(|r| r.arrival_id == id)
            .unwrap()
    };
    assert_eq!(by_arrival(due.id).days_waiting, 3);
    assert_eq!(by_arrival(late.id).days_waiting, 0);
    let by_purchase = |id: i64| list.own_rows.iter().find(|r| r.purchase_id == id).unwrap();
    assert_eq!(by_purchase(due_purchase.id).days_waiting, 3);
    assert_eq!(by_purchase(late_purchase.id).days_waiting, 0);

    // The day to sell moves forward: the arrival is no longer late.
    let moved = gardens::arrivals::set_sell_date(
        &w.env.ctx,
        &w.owner,
        &SellDateInput {
            id: due.id,
            version: due.version,
            sell_date: "14-10-2026".into(),
        },
    )
    .unwrap();
    assert_eq!(moved.sell_date, "2026-10-14");
    let item = waiting(&w).expect("the purchase still waits");
    assert_eq!((item.count, item.qty), (1, Some(20)));
    assert_eq!(
        item.first.map(|f| (f.target, f.id)),
        Some((AttentionTarget::Purchase, due_purchase.id))
    );
}
