//! Real answers of the many-lines entries and the shop split, as JSON files for the frontend (plan
//! 35, T2). Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes
//! and `<name>.json` is the answer the real service gives. A refusal is a pair too (`ok`, `errors`,
//! `ref`). The test compares the answers with the files on disk and FAILS if any differ; write them
//! again with `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test shop_split_fixtures`.
//!
//! The story is plan 35 Section 3: trip 0070, one purchase entry with two lines (مالٹا 80 at 1,200
//! with 30 for another shop, کیلا 40 at 900), then a garden with a goods-arrived entry of two lines.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::set_today;
use mandi_services::core::arrivals::ArrivalEntryInput;
use mandi_services::core::bikri::{ReadyGoodsInput, StockListInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{GardenEntryInput, GardenInput};
use mandi_services::core::purchases::PurchaseEntryInput;
use mandi_services::core::split::SplitInput;
use mandi_services::core::trips::{OtherSaleInput, TripInput};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{ServiceError, bikri, gardens, purchases, trips};
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

#[test]
fn shop_split_fixtures_of_the_entries_the_split_and_the_sales_in_another_shop() {
    let w = support::world();
    let ctx = &w.env.ctx;
    let owner = &w.owner;
    set_today(&w.env, 2026, 10, 10);
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
    let line = |key: &str, item: i64, qty: &str, rate: &str, other: Value| {
        json!({
            "key": key, "item_id": item, "packing_id": w.box_, "qty": qty, "weight": null,
            "rate": rate, "freight": null, "labour": null, "other_cost": null, "note": null,
            "other_qty": other, "shop_name": if other.is_null() { Value::Null } else { json!("حاجی شاپ") }
        })
    };
    let entry = |lines: Vec<Value>, confirm: bool| {
        json!({
            "trip_id": trip.id, "date": "10-10-2026", "supplier": null, "sell_date": null,
            "parts": [], "confirm_cash_below_zero": confirm, "lines": lines
        })
    };

    // A one-line entry: exactly one purchase of today.
    let one = entry(vec![line("a", w.banana, "20", "500", Value::Null)], false);
    pair(
        "purchases_entry_preview_one_line",
        &one,
        &purchases::entry_preview(ctx, &typed::<PurchaseEntryInput>(&one)).unwrap(),
    );
    // A two-line entry with the split on line 1; the cash question is for both lines together.
    let two = entry(
        vec![
            line("a", w.banana, "80", "1,200", json!("30")),
            line("b", w.apple, "40", "900", Value::Null),
        ],
        false,
    );
    pair(
        "purchases_entry_preview_two_lines",
        &two,
        &purchases::entry_preview(ctx, &typed::<PurchaseEntryInput>(&two)).unwrap(),
    );
    let refused =
        purchases::entry_save(ctx, owner, &typed::<PurchaseEntryInput>(&two)).unwrap_err();
    refusal(
        "purchases_entry_save_cash_below_zero",
        &two,
        &refused,
        ErrorCode::ErrCashBelowZeroNotConfirmed,
    );
    // A mistake in a line: the preview and the save point at the line and the field.
    let bad = entry(
        vec![
            line("a", w.banana, "20", "500", Value::Null),
            line("b", w.apple, "10", "500", json!("11")),
        ],
        false,
    );
    pair(
        "purchases_entry_preview_line_error",
        &bad,
        &purchases::entry_preview(ctx, &typed::<PurchaseEntryInput>(&bad)).unwrap(),
    );
    let error = purchases::entry_save(ctx, owner, &typed::<PurchaseEntryInput>(&bad)).unwrap_err();
    refusal(
        "purchases_entry_save_line_error",
        &bad,
        &error,
        ErrorCode::ErrSplitAboveQty,
    );
    let small = entry(
        vec![
            line("a", w.banana, "80", "50", json!("30")),
            line("b", w.apple, "40", "30", Value::Null),
        ],
        false,
    );
    let saved = purchases::entry_save(ctx, owner, &typed::<PurchaseEntryInput>(&small)).unwrap();
    pair("purchases_entry_save_two_lines", &small, &saved);
    let first = saved.saved[0].id;

    // The split: change it, and refuse it below what a part sold.
    let get = purchases::get(
        ctx,
        &mandi_services::core::purchases::PurchaseGetInput { id: first },
    )
    .unwrap();
    pair("purchases_get_with_a_split", &json!({ "id": first }), &get);
    let change =
        json!({ "id": first, "version": get.version, "other_qty": "20", "shop_name": "حاجی شاپ" });
    let set = purchases::change_split(ctx, owner, &typed::<SplitInput>(&change)).unwrap();
    pair("purchases_change_split", &change, &set);
    let above =
        json!({ "id": first, "version": set.version, "other_qty": "81", "shop_name": null });
    let error = purchases::change_split(ctx, owner, &typed::<SplitInput>(&above)).unwrap_err();
    refusal(
        "purchases_change_split_above_qty",
        &above,
        &error,
        ErrorCode::ErrSplitAboveQty,
    );

    // A sale in another shop takes the other part first (20) and moves the rest (5) from our shop.
    let sale = json!({
        "trip_id": trip.id, "date": "10-10-2026", "shop": "حاجی شاپ", "purchase_id": first,
        "qty": "25", "amount": "2,000", "note": null
    });
    pair(
        "trips_other_sale_preview_moves_from_our_shop",
        &sale,
        &trips::other_sale_preview(ctx, &typed::<OtherSaleInput>(&sale)).unwrap(),
    );
    trips::other_sale_save(ctx, owner, &typed::<OtherSaleInput>(&sale)).unwrap();
    let too_many = json!({
        "trip_id": trip.id, "date": "10-10-2026", "shop": null, "purchase_id": first,
        "qty": "56", "amount": "100", "note": null
    });
    let error =
        trips::other_sale_save(ctx, owner, &typed::<OtherSaleInput>(&too_many)).unwrap_err();
    refusal(
        "trips_other_sale_above_all_left",
        &too_many,
        &error,
        ErrorCode::ErrQtyExceedsLot,
    );
    let ready = json!({ "date": "10-10-2026" });
    pair(
        "bikri_ready_goods_our_shop_only",
        &ready,
        &bikri::ready_goods(ctx, &typed::<ReadyGoodsInput>(&ready)).unwrap(),
    );
    let stock = json!({
        "date": null, "from": null, "to": null, "seller_id": null, "item_id": null,
        "lot_id": null, "include_written_off": false, "offset": 0, "limit": 0
    });
    pair(
        "bikri_stock_list_with_the_split",
        &stock,
        &bikri::stock_list(ctx, &typed::<StockListInput>(&stock)).unwrap(),
    );

    // The garden: a goods-arrived entry of two lines, and a sale in another shop that names one.
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
    let aline = |key: &str, item: i64, qty: &str, other: Value| {
        json!({
            "key": key, "item_id": item, "packing_id": w.box_, "qty": qty, "weight": null,
            "note": null, "other_qty": other,
            "shop_name": if other.is_null() { Value::Null } else { json!("حاجی شاپ") }
        })
    };
    let arrivals = json!({
        "garden_id": garden.id, "date": "10-10-2026", "sell_date": null,
        "lines": [aline("a", w.banana, "100", json!("40")), aline("b", w.apple, "30", Value::Null)]
    });
    pair(
        "gardens_arrival_entry_preview_two_lines",
        &arrivals,
        &gardens::arrivals::entry_preview(ctx, &typed::<ArrivalEntryInput>(&arrivals)).unwrap(),
    );
    let bad = json!({
        "garden_id": garden.id, "date": "10-10-2026", "sell_date": null,
        "lines": [aline("a", w.banana, "10", Value::Null), aline("b", w.apple, "5", json!("9"))]
    });
    pair(
        "gardens_arrival_entry_preview_line_error",
        &bad,
        &gardens::arrivals::entry_preview(ctx, &typed::<ArrivalEntryInput>(&bad)).unwrap(),
    );
    let error =
        gardens::arrivals::entry_save(ctx, owner, &typed::<ArrivalEntryInput>(&bad)).unwrap_err();
    refusal(
        "gardens_arrival_entry_save_line_error",
        &bad,
        &error,
        ErrorCode::ErrSplitAboveQty,
    );
    let done =
        gardens::arrivals::entry_save(ctx, owner, &typed::<ArrivalEntryInput>(&arrivals)).unwrap();
    pair("gardens_arrival_entry_save_two_lines", &arrivals, &done);
    let arrival = done.rows[0].id;
    let split = json!({ "id": arrival, "version": done.rows[0].version, "other_qty": "50", "shop_name": null });
    pair(
        "gardens_arrival_change_split",
        &split,
        &gardens::arrivals::change_split(ctx, owner, &typed::<SplitInput>(&split)).unwrap(),
    );
    let sale_json = json!({
        "type": "other_sale", "garden_id": garden.id, "date": "10-10-2026", "shop": "حاجی شاپ",
        "arrival_id": arrival, "qty": "60", "amount": "20,000", "note": null
    });
    pair(
        "gardens_entry_preview_other_sale_with_arrival",
        &sale_json,
        &gardens::entry_preview(ctx, &typed::<GardenEntryInput>(&sale_json)).unwrap(),
    );
}
