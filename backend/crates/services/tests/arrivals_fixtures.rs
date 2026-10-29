//! Real answers of the date to sell, the garden arrivals (مال آیا), the garden lot linked to an
//! arrival and the ready goods of a new sale, as JSON files for the frontend (plan 33 v1.1, T2).
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used), and `<name>.json` is the answer the real
//! service gives for it. A refusal is a pair too: its answer has the shape of the error a command
//! returns (`ok`, `errors`, `ref`). The frontend copies the files to `frontend/tests/fixtures/`.
//!
//! The test compares the answers with the files on disk and FAILS if any differ. After a deliberate
//! change, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test arrivals_fixtures`, look at the diff,
//! and tell `frontend`.
//!
//! The story: the garden شاہ پور باغ (0080) and a trip (0070) on 10-10-2026. On 11-10 100 پیٹی کیلا
//! arrive to sell on 12-10, and 20 پیٹی are bought on the trip to sell on 12-10. On 12-10 a new sale
//! opens with both as ready goods; 60 of the arrival are sold in a linked garden lot, 40 stay.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::set_today;
use mandi_services::core::arrivals::{
    ArrivalCancelInput, ArrivalGetInput, ArrivalInput, ArrivalWriteOffInput,
    ArrivalWriteOffUndoInput, ArrivalsListInput, SellDateInput,
};
use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, HeaderInput, LotInput, ReadyGoodsInput, StockListInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{GardenCloseInput, GardenInput};
use mandi_services::core::purchases::PurchaseInput;
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{ServiceError, bikri, gardens, purchases, trips};
use serde::Serialize;
use serde_json::{Value, json};
use support::{World, bare_lot, sale, with_sales};

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
        "ref": "E-20261011-0001"
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

fn partners_json(w: &World) -> Vec<VenturePartnerInput> {
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

fn draft_json(draft: &BikriDraft) -> Value {
    serde_json::to_value(draft).unwrap()
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

#[test]
fn arrivals_fixtures_of_the_date_to_sell_the_arrival_and_its_linked_lot() {
    let w = support::world();
    let ctx = &w.env.ctx;
    let owner = &w.owner;
    set_today(&w.env, 2026, 10, 11);
    let garden = gardens::create(
        ctx,
        owner,
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
            partners: partners_json(&w),
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .row;
    let trip = trips::create(
        ctx,
        owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: partners_json(&w),
        },
    )
    .unwrap();

    // The purchase gets its day to sell with the save; it can be moved later.
    let buy = json!({
        "trip_id": trip.id, "date": "11-10-2026", "supplier": null, "item_id": w.banana,
        "packing_id": w.box_, "qty": "20", "weight": null, "sell_date": "12-10-2026",
        "rate": "50", "freight": null, "labour": null, "other_cost": null, "note": null,
        "parts": [], "confirm_cash_below_zero": false
    });
    let bought = purchases::save(ctx, owner, &typed::<PurchaseInput>(&buy)).unwrap();
    pair("purchases_save_with_sell_date", &buy, &bought);
    let early = json!({
        "trip_id": trip.id, "date": "11-10-2026", "supplier": null, "item_id": w.banana,
        "packing_id": w.box_, "qty": "20", "weight": null, "sell_date": "10-10-2026",
        "rate": "50", "freight": null, "labour": null, "other_cost": null, "note": null,
        "parts": [], "confirm_cash_below_zero": false
    });
    let error = purchases::save(ctx, owner, &typed::<PurchaseInput>(&early)).unwrap_err();
    refusal(
        "purchases_save_sell_date_before_entry",
        &early,
        &error,
        ErrorCode::ErrSellDateBeforeEntry,
    );
    let moved = json!({ "id": bought.id, "version": bought.version, "sell_date": "13-10-2026" });
    let set = purchases::set_sell_date(ctx, owner, &typed::<SellDateInput>(&moved)).unwrap();
    pair("purchases_set_sell_date", &moved, &set);
    let back = json!({ "id": bought.id, "version": set.version, "sell_date": "12-10-2026" });
    purchases::set_sell_date(ctx, owner, &typed::<SellDateInput>(&back)).unwrap();

    // The arrival.
    let arrive = json!({
        "garden_id": garden.id, "date": "11-10-2026", "sell_date": "12-10-2026",
        "item_id": w.banana, "packing_id": w.box_, "qty": "100", "weight": null, "note": null
    });
    let saved = gardens::arrivals::save(ctx, owner, &typed::<ArrivalInput>(&arrive)).unwrap();
    pair("gardens_arrival_save", &arrive, &saved);
    let id = saved.row.id;
    let wrong = json!({
        "garden_id": garden.id, "date": "11-10-2026", "sell_date": "10-10-2026",
        "item_id": w.banana, "packing_id": w.box_, "qty": "100", "weight": null, "note": null
    });
    let error = gardens::arrivals::save(ctx, owner, &typed::<ArrivalInput>(&wrong)).unwrap_err();
    refusal(
        "gardens_arrival_save_sell_date_before_entry",
        &wrong,
        &error,
        ErrorCode::ErrSellDateBeforeEntry,
    );
    let sell = json!({ "id": id, "version": saved.row.version, "sell_date": "14-10-2026" });
    let set = gardens::arrivals::set_sell_date(ctx, owner, &typed::<SellDateInput>(&sell)).unwrap();
    pair("gardens_arrival_set_sell_date", &sell, &set);
    let sell = json!({ "id": id, "version": set.version, "sell_date": "12-10-2026" });
    gardens::arrivals::set_sell_date(ctx, owner, &typed::<SellDateInput>(&sell)).unwrap();

    let list = json!({
        "garden_id": garden.id, "from": null, "to": null, "item_id": null,
        "state": "in_stock", "offset": 0, "limit": 0
    });
    pair(
        "gardens_arrivals_list_in_stock",
        &list,
        &gardens::arrivals::list(ctx, &typed::<ArrivalsListInput>(&list)).unwrap(),
    );

    // The garden cannot be closed while the arrival has goods left.
    let fresh = gardens::get(
        ctx,
        &mandi_services::core::gardens::GardenGetInput {
            id: garden.id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap();
    let close = json!({ "id": garden.id, "version": fresh.row.version, "date": "11-10-2026" });
    let error = gardens::close(ctx, owner, &typed::<GardenCloseInput>(&close)).unwrap_err();
    refusal(
        "gardens_close_has_stock",
        &close,
        &error,
        ErrorCode::ErrGardenHasStock,
    );

    // A new sale of 12-10 opens with both kinds of goods, oldest day to sell first.
    set_today(&w.env, 2026, 10, 12);
    let ready = json!({ "date": "12-10-2026" });
    pair(
        "bikri_ready_goods",
        &ready,
        &bikri::ready_goods(ctx, &typed::<ReadyGoodsInput>(&ready)).unwrap(),
    );
    let none = json!({ "date": "11-10-2026" });
    pair(
        "bikri_ready_goods_none_yet",
        &none,
        &bikri::ready_goods(ctx, &typed::<ReadyGoodsInput>(&none)).unwrap(),
    );

    // The garden lot linked to the arrival: 60 of the 100 are sold.
    let lot = |arrival: Option<i64>, received: &str, sold: &str| {
        with_sales(
            LotInput {
                arrival_id: arrival,
                ..bare_lot(&w, "lot-1", garden.account.id, w.banana, received)
            },
            vec![sale("sale-1", w.akram, sold, "600")],
        )
    };
    let linked = draft_on("12-10-2026", vec![lot(Some(id), "60", "60")]);
    let preview_request = json!({ "draft": draft_json(&linked) });
    pair(
        "bikri_preview_garden_lot_with_arrival",
        &preview_request,
        &support::preview(&w, &linked),
    );
    let not_full = draft_on("12-10-2026", vec![lot(Some(id), "60", "50")]);
    let request = json!({ "draft": draft_json(&not_full) });
    pair(
        "bikri_preview_garden_lot_with_arrival_not_fully_sold",
        &request,
        &support::preview(&w, &not_full),
    );
    let too_many = draft_on("12-10-2026", vec![lot(Some(id), "101", "101")]);
    let request = json!({ "draft": draft_json(&too_many) });
    pair(
        "bikri_preview_garden_lot_with_arrival_too_many",
        &request,
        &support::preview(&w, &too_many),
    );
    let save_request = json!({ "draft": draft_json(&linked), "confirm_cash_below_zero": false });
    let done = bikri::save(ctx, owner, &typed::<BikriSaveInput>(&save_request)).unwrap();
    pair("bikri_save_garden_lot_with_arrival", &save_request, &done);
    // A second sale opened on the same 100 is refused on qty.
    let again = bikri::save(ctx, owner, &typed::<BikriSaveInput>(&save_request)).unwrap_err();
    refusal(
        "bikri_save_garden_lot_with_arrival_oversold",
        &save_request,
        &again,
        ErrorCode::ErrQtyExceedsLot,
    );
    let detail = bikri::get(ctx, done.bikri_id).unwrap();
    pair(
        "bikri_get_garden_lot_with_arrival",
        &json!({ "id": done.bikri_id }),
        &detail,
    );

    // The 40 that are left come again, and show in باقی مال.
    pair(
        "bikri_ready_goods_after_a_part_sale",
        &ready,
        &bikri::ready_goods(ctx, &typed::<ReadyGoodsInput>(&ready)).unwrap(),
    );
    let stock = json!({
        "date": null, "from": null, "to": null, "seller_id": null, "item_id": null,
        "lot_id": null, "include_written_off": false, "offset": 0, "limit": 0
    });
    pair(
        "bikri_stock_list_with_garden_rows",
        &stock,
        &bikri::stock_list(ctx, &typed::<StockListInput>(&stock)).unwrap(),
    );
    let get = json!({ "id": id });
    pair(
        "gardens_arrival_get_after_a_sale",
        &get,
        &gardens::arrivals::get(ctx, &typed::<ArrivalGetInput>(&get)).unwrap(),
    );

    // Cancel is refused once something is sold; a write-off of the rest can be undone.
    let after = gardens::arrivals::get(ctx, &typed::<ArrivalGetInput>(&get)).unwrap();
    let cancel = json!({ "id": id, "version": after.version, "reason": "غلط" });
    let error =
        gardens::arrivals::cancel(ctx, owner, &typed::<ArrivalCancelInput>(&cancel)).unwrap_err();
    refusal(
        "gardens_arrival_cancel_has_sales",
        &cancel,
        &error,
        ErrorCode::ErrArrivalHasSales,
    );
    let off = json!({ "arrival_id": id, "qty": "40", "reason": "خراب" });
    let written =
        gardens::arrivals::write_off(ctx, owner, &typed::<ArrivalWriteOffInput>(&off)).unwrap();
    pair("gardens_arrival_write_off", &off, &written);
    let too_much = json!({ "arrival_id": id, "qty": "1", "reason": "خراب" });
    let error = gardens::arrivals::write_off(ctx, owner, &typed::<ArrivalWriteOffInput>(&too_much))
        .unwrap_err();
    refusal(
        "gardens_arrival_write_off_exceeds",
        &too_much,
        &error,
        ErrorCode::ErrWriteOffExceedsBalance,
    );
    let undo = json!({ "move_id": written.move_id, "reason": "غلطی" });
    pair(
        "gardens_arrival_write_off_undo",
        &undo,
        &gardens::arrivals::write_off_undo(ctx, owner, &typed::<ArrivalWriteOffUndoInput>(&undo))
            .unwrap(),
    );

    // An untouched arrival is cancelled with a reason.
    let spare = json!({
        "garden_id": garden.id, "date": "12-10-2026", "sell_date": null,
        "item_id": w.banana, "packing_id": w.box_, "qty": "5", "weight": null, "note": "ٹوٹا"
    });
    let spare = gardens::arrivals::save(ctx, owner, &typed::<ArrivalInput>(&spare)).unwrap();
    let cancel =
        json!({ "id": spare.row.id, "version": spare.row.version, "reason": "غلط اندراج" });
    pair(
        "gardens_arrival_cancel",
        &cancel,
        &gardens::arrivals::cancel(ctx, owner, &typed::<ArrivalCancelInput>(&cancel)).unwrap(),
    );
    let all = json!({
        "garden_id": null, "from": null, "to": null, "item_id": null, "state": null,
        "offset": 0, "limit": 0, "show_finished": true
    });
    pair(
        "gardens_arrivals_list_all",
        &all,
        &gardens::arrivals::list(ctx, &typed::<ArrivalsListInput>(&all)).unwrap(),
    );
}
