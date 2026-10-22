//! Real answers of the market trip commands and of the own lot in بکری, as JSON files for the
//! frontend (plan 16b, T2b).
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used), and `<name>.json` is the answer the real
//! service gives for it. A refusal is a pair too: its answer has the shape of the error a command
//! returns (`ok`, `errors`, `ref`), where `ref` is a stand-in because the real reference number is a
//! counter. The frontend copies the files to `frontend/tests/fixtures/`.
//!
//! The test compares the answers with the files on disk and FAILS if any differ, so a fixture can
//! never go stale. After a deliberate change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test trips_fixtures`, look at the diff, and
//! tell `frontend`.
//!
//! Story A is E2E-34 and E2E-34b (a trip of the three partners, 20 پیٹی مالٹا, an own lot that sells
//! 15, a sale in another shop, a write-off, a cancel). Story B is E2E-41 (الف from the shop, ب from his
//! pocket, a truck, an own lot that sells all 80).
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::set_today;
use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, CancelInput, EditOpenInput, LotInput, StockListInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::purchases::{
    PurchaseCancelInput, PurchaseGetInput, PurchaseInput, PurchaseWriteOffInput,
    PurchaseWriteOffUndoInput, PurchasesListInput,
};
use mandi_services::core::trips::{
    OtherSaleInput, TripCostInput, TripEntryCancelInput, TripGetInput, TripInput, TripsListInput,
};
use mandi_services::{ServiceError, bikri, purchases, trips};
use serde::Serialize;
use serde_json::{Value, json};
use support::World;

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Writes the file (when asked to) or checks that it holds exactly this text.
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

/// One pair of files: the request and the real answer.
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

/// A refusal as a command returns it. `ref` is a stand-in: the real one is a counter.
fn wire(error: &ServiceError) -> Value {
    let ServiceError::Rejected(items) = error else {
        panic!("not a refusal of the user's entry: {error:?}");
    };
    json!({
        "ok": false,
        "errors": items
            .iter()
            .map(|i| json!({ "code": i.code, "field": i.field, "values": i.values }))
            .collect::<Vec<_>>(),
        "ref": "E-20261010-0001"
    })
}

/// A pair for a refusal; checks the code too.
fn refusal(name: &str, request: &Value, error: &ServiceError, code: ErrorCode) {
    assert_eq!(error.code(), code, "{name}");
    pair(name, request, &wire(error));
}

/// The request read back into the real input type, so the file is exactly what the command takes.
fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

// ---- The JSON of the entries, as the screen sends them ----

fn partner_json(id: i64, share: &str, source: &str) -> Value {
    json!({ "partner_id": id, "share": share, "source": source })
}

fn trip_json(partners: &[Value]) -> Value {
    json!({
        "date": "10-10-2026", "market": "Lahore", "sent_person": "Aslam", "note": null,
        "partners": partners
    })
}

fn purchase_json(trip: i64, item: i64, w: &World, qty: &str, rate: &str, freight: &str) -> Value {
    json!({
        "trip_id": trip, "date": "10-10-2026", "supplier": "Orange Traders", "item_id": item,
        "packing_id": w.box_, "qty": qty, "weight": null, "rate": rate,
        "freight": if freight.is_empty() { Value::Null } else { json!(freight) },
        "labour": null, "other_cost": null, "note": null, "parts": [],
        "confirm_cash_below_zero": false
    })
}

/// An own lot taking `qty` from a purchase, selling all of it to `buyer` at `rate` on credit.
fn own_draft(w: &World, purchase: i64, qty: &str, buyer: i64, rate: &str) -> BikriDraft {
    let lot = LotInput {
        seller_account_id: Some(support::id_of(&w.env, "030119998")),
        purchase_id: Some(purchase),
        item_id: None,
        packing_id: None,
        adjust_advance: false,
        ..support::bare_lot(w, "own-1", w.rahim, w.banana, qty)
    };
    support::draft(vec![support::with_sales(
        lot,
        vec![support::sale("s-1", buyer, qty, rate)],
    )])
}

fn draft_json(draft: &BikriDraft) -> Value {
    serde_json::to_value(draft).unwrap()
}

fn orange(w: &World) -> i64 {
    support::add_item(&w.env, &w.owner, "مالٹا", "Orange", "fruit", "unit")
}

#[test]
fn trip_fixtures_of_e2e_34_and_34b() {
    let w = support::world();
    let item = orange(&w);
    let [a, b, c] = w.partners;
    let owner = &w.owner;
    let ctx = &w.env.ctx;

    // The trip: the first one starts from the firm's shares (PRD FR-12.29).
    pair(
        "trips_defaults_first",
        &json!({}),
        &trips::defaults(ctx).unwrap(),
    );
    let shares_bad = trip_json(&[
        partner_json(a, "40", "shop"),
        partner_json(b, "30", "shop"),
        partner_json(c, "20", "shop"),
    ]);
    let error = trips::create(ctx, owner, &typed::<TripInput>(&shares_bad)).unwrap_err();
    refusal(
        "trips_create_shares_not_100",
        &shares_bad,
        &error,
        ErrorCode::ErrSharesNot100,
    );
    let three = trip_json(&[
        partner_json(a, "40", "shop"),
        partner_json(b, "30", "shop"),
        partner_json(c, "30", "shop"),
    ]);
    let trip = trips::create(ctx, owner, &typed::<TripInput>(&three)).unwrap();
    pair("trips_create_three", &three, &trip);
    // The next trip starts from the last trip's shares and sources.
    pair(
        "trips_defaults_from_last_trip",
        &json!({}),
        &trips::defaults(ctx).unwrap(),
    );

    // The purchase of E2E-34: 20 x 500 with freight 400, paid at once from the shop's cash.
    let buy = purchase_json(trip.id, item, &w, "20", "500", "400");
    pair(
        "purchases_preview_e2e34",
        &buy,
        &purchases::preview(ctx, &typed::<PurchaseInput>(&buy)).unwrap(),
    );
    // A purchase that is not complete shows its mistakes at their fields.
    let mut no_qty = buy.clone();
    no_qty["qty"] = json!("");
    pair(
        "purchases_preview_no_qty",
        &no_qty,
        &purchases::preview(ctx, &typed::<PurchaseInput>(&no_qty)).unwrap(),
    );
    let bought = purchases::save(ctx, owner, &typed::<PurchaseInput>(&buy)).unwrap();
    pair("purchases_save_e2e34", &buy, &bought);

    // The own lot: 15 of the 20 to اکرم at 600 (E2E-34).
    let draft = own_draft(&w, bought.id, "15", w.akram, "600");
    let preview_request = json!({ "draft": draft_json(&draft) });
    pair(
        "bikri_preview_own_lot_e2e34",
        &preview_request,
        &bikri::preview(ctx, &draft).unwrap(),
    );
    // A lot that is not sold in full is refused with the reason on the lot (FR-12.26).
    let short = own_draft(&w, bought.id, "15", w.akram, "600");
    let mut short_json = draft_json(&short);
    short_json["lots"][0]["sales"][0]["qty"] = json!("10");
    let short_draft: BikriDraft = typed(&short_json);
    pair(
        "bikri_preview_own_lot_not_fully_sold",
        &json!({ "draft": short_json }),
        &bikri::preview(ctx, &short_draft).unwrap(),
    );
    let save_request = json!({ "draft": draft_json(&draft), "confirm_cash_below_zero": false });
    let saved = bikri::save(ctx, owner, &typed::<BikriSaveInput>(&save_request)).unwrap();
    pair("bikri_save_own_lot_e2e34", &save_request, &saved);
    let get_request = json!({ "id": saved.bikri_id });
    pair(
        "bikri_get_own_lot_e2e34",
        &get_request,
        &bikri::get(ctx, saved.bikri_id).unwrap(),
    );
    pair(
        "bikri_stock_list_own_rows",
        &json!({}),
        &bikri::stock_list(ctx, &StockListInput::default()).unwrap(),
    );
    let open_request = json!({ "bikri_id": saved.bikri_id, "entry_id": saved.entry_id });
    pair(
        "bikri_open_for_edit_own_lot",
        &open_request,
        &bikri::open_for_edit(ctx, &typed::<EditOpenInput>(&open_request)).unwrap(),
    );

    // A sale in another shop: 3 of the 5 left for 2,000.
    let other = json!({
        "trip_id": trip.id, "date": "10-10-2026", "shop": "Haji Shop", "purchase_id": bought.id,
        "qty": "3", "amount": "2,000", "note": null
    });
    pair(
        "trips_other_sale_preview_3",
        &other,
        &trips::other_sale_preview(ctx, &typed::<OtherSaleInput>(&other)).unwrap(),
    );
    let sold = trips::other_sale_save(ctx, owner, &typed::<OtherSaleInput>(&other)).unwrap();
    pair("trips_other_sale_save_3", &other, &sold);

    // The screens that read them.
    pair(
        "purchases_get_after_sales",
        &json!({ "id": bought.id }),
        &purchases::get(ctx, &typed::<PurchaseGetInput>(&json!({ "id": bought.id }))).unwrap(),
    );
    pair(
        "purchases_list_all",
        &json!({}),
        &purchases::list(ctx, &PurchasesListInput::default()).unwrap(),
    );
    let get_trip = json!({ "id": trip.id, "entries_offset": 0, "entries_limit": 0 });
    pair(
        "trips_get_open",
        &get_trip,
        &trips::get(ctx, &typed::<TripGetInput>(&get_trip)).unwrap(),
    );
    pair(
        "trips_list_all",
        &json!({}),
        &trips::list(ctx, &TripsListInput::default()).unwrap(),
    );
    pair(
        "trips_summary_open",
        &json!({}),
        &trips::summary(ctx).unwrap(),
    );

    // E2E-34b: on 11-10-26 the last 2 spoil. Nothing posts, and the trip closes by itself.
    set_today(&w.env, 2026, 10, 11);
    let too_many = json!({ "purchase_id": bought.id, "qty": "3", "reason": "spoiled" });
    let error =
        purchases::write_off(ctx, owner, &typed::<PurchaseWriteOffInput>(&too_many)).unwrap_err();
    refusal(
        "purchases_write_off_too_many",
        &too_many,
        &error,
        ErrorCode::ErrWriteOffExceedsBalance,
    );
    let spoil = json!({ "purchase_id": bought.id, "qty": "2", "reason": "spoiled" });
    let off = purchases::write_off(ctx, owner, &typed::<PurchaseWriteOffInput>(&spoil)).unwrap();
    assert!(off.trip_closed);
    pair("purchases_write_off_last_2", &spoil, &off);
    pair(
        "purchases_get_after_write_off",
        &json!({ "id": bought.id }),
        &purchases::get(ctx, &typed::<PurchaseGetInput>(&json!({ "id": bought.id }))).unwrap(),
    );
    pair(
        "trips_get_closed",
        &get_trip,
        &trips::get(ctx, &typed::<TripGetInput>(&get_trip)).unwrap(),
    );
    let error = purchases::save(ctx, owner, &typed::<PurchaseInput>(&buy)).unwrap_err();
    refusal(
        "purchases_save_trip_closed",
        &buy,
        &error,
        ErrorCode::ErrTripClosed,
    );

    // Cancelling the sale in another shop and the بکری: the trip opens again.
    let cancel_other = json!({
        "id": sold.id, "version": sold.version, "reason": "wrong shop",
        "confirm_cash_below_zero": false
    });
    let cancelled =
        trips::other_sale_cancel(ctx, owner, &typed::<TripEntryCancelInput>(&cancel_other))
            .unwrap();
    pair("trips_other_sale_cancel", &cancel_other, &cancelled);
    let detail = bikri::get(ctx, saved.bikri_id).unwrap();
    let cancel = json!({
        "id": saved.bikri_id, "version": detail.version, "reason": "غلط اندراج",
        "confirm_cash_below_zero": false
    });
    let done = bikri::cancel(ctx, owner, &typed::<CancelInput>(&cancel)).unwrap();
    pair("bikri_cancel_own_lot", &cancel, &done);
    pair(
        "trips_get_reopened",
        &get_trip,
        &trips::get(ctx, &typed::<TripGetInput>(&get_trip)).unwrap(),
    );
    // Undoing the write-off gives back the 2 nag and their cost.
    let undo = json!({ "move_id": off.move_id, "reason": "not spoiled" });
    let undone =
        purchases::write_off_undo(ctx, owner, &typed::<PurchaseWriteOffUndoInput>(&undo)).unwrap();
    pair("purchases_write_off_undo", &undo, &undone);
}

#[test]
fn trip_fixtures_of_e2e_41() {
    let w = support::world();
    let item = orange(&w);
    let [a, b, c] = w.partners;
    let owner = &w.owner;
    let ctx = &w.env.ctx;

    // الف 50% from the shop, ب 50% from his pocket, ج not in the trip.
    let request = trip_json(&[
        partner_json(a, "50", "shop"),
        partner_json(b, "50", "pocket"),
        partner_json(c, "0", "shop"),
    ]);
    let trip = trips::create(ctx, owner, &typed::<TripInput>(&request)).unwrap();
    pair("trips_create_pocket", &request, &trip);
    let buy = purchase_json(trip.id, item, &w, "80", "1,200", "");
    pair(
        "purchases_preview_e2e41_pocket_part",
        &buy,
        &purchases::preview(ctx, &typed::<PurchaseInput>(&buy)).unwrap(),
    );
    let bought = purchases::save(ctx, owner, &typed::<PurchaseInput>(&buy)).unwrap();
    pair("purchases_save_e2e41", &buy, &bought);

    // The truck: 4,000, paid in by the trip's shares and spread over the purchase.
    let cost = json!({
        "trip_id": trip.id, "date": "10-10-2026", "amount": "4,000", "reason": "truck",
        "parts": [], "confirm_cash_below_zero": false
    });
    pair(
        "trips_cost_preview_truck",
        &cost,
        &trips::cost_preview(ctx, &typed::<TripCostInput>(&cost)).unwrap(),
    );
    let saved_cost = trips::cost_save(ctx, owner, &typed::<TripCostInput>(&cost)).unwrap();
    pair("trips_cost_save_truck", &cost, &saved_cost);

    // The own lot sells all 80 to ارشد at 1,400: the trip closes by itself (E2E-41).
    let draft = own_draft(&w, bought.id, "80", w.arshad, "1,400");
    let save_request = json!({ "draft": draft_json(&draft), "confirm_cash_below_zero": false });
    let saved = bikri::save(ctx, owner, &typed::<BikriSaveInput>(&save_request)).unwrap();
    pair("bikri_save_own_lot_e2e41", &save_request, &saved);
    pair(
        "bikri_get_own_lot_e2e41",
        &json!({ "id": saved.bikri_id }),
        &bikri::get(ctx, saved.bikri_id).unwrap(),
    );
    let get_trip = json!({ "id": trip.id, "entries_offset": 0, "entries_limit": 0 });
    pair(
        "trips_get_e2e41_closed",
        &get_trip,
        &trips::get(ctx, &typed::<TripGetInput>(&get_trip)).unwrap(),
    );
    // The truck cannot be cancelled once its purchase sold something.
    let cancel_cost = json!({
        "id": saved_cost.id, "version": saved_cost.version, "reason": "wrong",
        "confirm_cash_below_zero": false
    });
    let error =
        trips::cost_cancel(ctx, owner, &typed::<TripEntryCancelInput>(&cancel_cost)).unwrap_err();
    refusal(
        "trips_cost_cancel_has_sales",
        &cancel_cost,
        &error,
        ErrorCode::ErrTripCostHasSales,
    );
}

/// Story C: a purchase and a truck that are cancelled again, and a purchase that takes the cash
/// below zero (PRD FR-6.7).
#[test]
fn trip_fixtures_of_cancels_and_the_cash_question() {
    let w = support::world();
    let item = orange(&w);
    let [a, b, c] = w.partners;
    let owner = &w.owner;
    let ctx = &w.env.ctx;
    let three = trip_json(&[
        partner_json(a, "40", "shop"),
        partner_json(b, "30", "shop"),
        partner_json(c, "30", "shop"),
    ]);
    let trip = trips::create(ctx, owner, &typed::<TripInput>(&three)).unwrap();

    // 100 x 1,000 is 100,000 and the cash in hand is 50,000: the preview asks, the save refuses
    // until the user confirms.
    let big = purchase_json(trip.id, item, &w, "100", "1,000", "");
    let preview = purchases::preview(ctx, &typed::<PurchaseInput>(&big)).unwrap();
    assert!(preview.cash.needs_confirm);
    pair("purchases_preview_cash_below_zero", &big, &preview);
    let error = purchases::save(ctx, owner, &typed::<PurchaseInput>(&big)).unwrap_err();
    refusal(
        "purchases_save_cash_below_zero_not_confirmed",
        &big,
        &error,
        ErrorCode::ErrCashBelowZeroNotConfirmed,
    );

    // A small purchase, a truck, and both cancelled (the truck first).
    let small = purchase_json(trip.id, item, &w, "10", "100", "");
    let bought = purchases::save(ctx, owner, &typed::<PurchaseInput>(&small)).unwrap();
    let cost = json!({
        "trip_id": trip.id, "date": "10-10-2026", "amount": "600", "reason": "truck",
        "parts": [], "confirm_cash_below_zero": false
    });
    let truck = trips::cost_save(ctx, owner, &typed::<TripCostInput>(&cost)).unwrap();
    let cancel_truck = json!({
        "id": truck.id, "version": truck.version, "reason": "wrong amount",
        "confirm_cash_below_zero": false
    });
    let cancelled =
        trips::cost_cancel(ctx, owner, &typed::<TripEntryCancelInput>(&cancel_truck)).unwrap();
    pair("trips_cost_cancel", &cancel_truck, &cancelled);
    let row = purchases::get(ctx, &PurchaseGetInput { id: bought.id }).unwrap();
    let cancel_purchase = json!({
        "id": row.id, "version": row.version, "reason": "wrong goods",
        "confirm_cash_below_zero": false
    });
    let done =
        purchases::cancel(ctx, owner, &typed::<PurchaseCancelInput>(&cancel_purchase)).unwrap();
    pair("purchases_cancel", &cancel_purchase, &done);
    pair(
        "purchases_list_cancelled",
        &json!({ "state": "cancelled" }),
        &purchases::list(
            ctx,
            &typed::<PurchasesListInput>(&json!({ "state": "cancelled" })),
        )
        .unwrap(),
    );
}
