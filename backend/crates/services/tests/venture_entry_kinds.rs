//! The entries of a trip and the entries of a garden are told apart (gate finding T4-1, plan 80,
//! Code Standards V0, PRD FR-12.38).
//!
//! The entry kinds `cost` and `other_sale` exist for both. A trip command that checked only the
//! kind accepted the entry of a garden, even of a CLOSED garden (so a trip cost cancel bypassed
//! `ERR_GARDEN_CLOSED`), and a garden command accepted the entry of a trip. Each command must
//! answer `ERR_NOT_FOUND` on `id` for an entry of the other kind of venture, and write nothing.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::core::edit::{GardenEntryEditInput, TripCostEditInput, TripOtherSaleEditInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{
    GardenCloseInput, GardenEntryCancelInput, GardenEntryInput, GardenGetInput, GardenInput,
};
use mandi_services::core::purchases::PurchaseInput;
use mandi_services::core::trips::{OtherSaleInput, TripCostInput, TripEntryCancelInput, TripInput};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{ServiceError, edit, gardens, purchases, trips};
use serde_json::{Value, json};
use support::World;

fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

fn code_and_field(error: &ServiceError) -> (ErrorCode, Option<String>) {
    let ServiceError::Rejected(items) = error else {
        panic!("not a refusal of the user's entry: {error:?}");
    };
    let first = items.first().expect("a refusal has an error");
    (first.code, first.field.clone())
}

fn not_found_on_id(error: &ServiceError) {
    assert_eq!(
        code_and_field(error),
        (ErrorCode::ErrNotFound, Some("id".to_owned())),
        "{error:?}"
    );
}

/// Everything the books and the venture entries hold, so a refusal can be shown to write nothing.
fn state(w: &World) -> (i64, i64, String) {
    let conn = common::raw(&w.dir);
    let journal: i64 = conn
        .query_row("SELECT COUNT(*) FROM journal_entries", [], |r| r.get(0))
        .unwrap();
    let lines: i64 = conn
        .query_row("SELECT COUNT(*) FROM journal_lines", [], |r| r.get(0))
        .unwrap();
    let entries: String = conn
        .query_row(
            "SELECT group_concat(id || ':' || status || ':' || version, ',') FROM
             (SELECT id, status, version FROM venture_entries ORDER BY id)",
            [],
            |r| r.get(0),
        )
        .unwrap();
    (journal, lines, entries)
}

/// The ids of the entries of one garden and one trip, both of them with a cost and a sale in
/// another shop.
struct Ventures {
    garden: i64,
    trip: i64,
    purchase: i64,
    garden_cost: (i64, i64),
    garden_sale: (i64, i64),
    trip_cost: (i64, i64),
    trip_sale: (i64, i64),
}

fn build(w: &World) -> Ventures {
    let ctx = &w.env.ctx;
    let [a, ..] = w.partners;
    let one = |source: &str| json!([{ "partner_id": a, "share": "100", "source": source }]);
    let garden = gardens::create(
        ctx,
        &w.owner,
        &typed::<GardenInput>(&json!({
            "name_ur": "شاہ پور باغ",
            "name_en": "Shahpur Garden",
            "confirmed_rules_words": ["شاہ", "پور", "باغ", "Shahpur", "Garden"],
            "date": "10-10-2026",
            "item_id": w.banana,
            "rent": "50,000",
            "season_end": null,
            "note": null,
            "partners": one("shop"),
            "paid_now": null,
            "paid_now_reason": null,
            "parts": [],
            "confirm_cash_below_zero": false
        })),
    )
    .unwrap()
    .row
    .id;
    let garden_cost = gardens::entry_save(
        ctx,
        &w.owner,
        &typed::<GardenEntryInput>(&json!({
            "type": "cost", "garden_id": garden, "date": "10-10-2026", "amount": "1,000",
            "reason": "labour", "parts": [], "confirm_cash_below_zero": false
        })),
    )
    .unwrap();
    let garden_sale = gardens::entry_save(
        ctx,
        &w.owner,
        &typed::<GardenEntryInput>(&json!({
            "type": "other_sale", "garden_id": garden, "date": "10-10-2026",
            "shop": "Haji Traders", "qty": "5", "amount": "3,000", "note": null
        })),
    )
    .unwrap();

    let trip = trips::create(
        ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: vec![VenturePartnerInput {
                partner_id: a,
                share: "100".into(),
                source: MoneySource::Shop,
            }],
        },
    )
    .unwrap()
    .id;
    let purchase = purchases::save(
        ctx,
        &w.owner,
        &PurchaseInput {
            // The 5 sold in another shop below are set aside for it (PRD FR-12.41, v1.52).
            other_qty: Some("5".into()),
            shop_name: None,
            sell_date: None,
            trip_id: Some(trip),
            date: "10-10-2026".into(),
            supplier: Some("Orange Traders".into()),
            item_id: Some(w.banana),
            packing_id: Some(w.box_),
            qty: "20".into(),
            weight: None,
            rate: "500".into(),
            freight: None,
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let trip_cost = trips::cost_save(
        ctx,
        &w.owner,
        &TripCostInput {
            trip_id: Some(trip),
            date: "10-10-2026".into(),
            amount: "400".into(),
            reason: "truck".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let trip_sale = trips::other_sale_save(
        ctx,
        &w.owner,
        &OtherSaleInput {
            trip_id: Some(trip),
            date: "10-10-2026".into(),
            shop: Some("Haji Shop".into()),
            purchase_id: Some(purchase.id),
            qty: "5".into(),
            amount: "3,000".into(),
            note: None,
        },
    )
    .unwrap();
    Ventures {
        garden,
        trip,
        purchase: purchase.id,
        garden_cost: (garden_cost.id, garden_cost.version),
        garden_sale: (garden_sale.id, garden_sale.version),
        trip_cost: (trip_cost.id, trip_cost.version),
        trip_sale: (trip_sale.id, trip_sale.version),
    }
}

fn trip_cancel(id: i64, version: i64) -> TripEntryCancelInput {
    TripEntryCancelInput {
        id,
        version,
        reason: "wrong".into(),
        confirm_cash_below_zero: true,
    }
}

fn garden_cancel(id: i64, version: i64) -> GardenEntryCancelInput {
    GardenEntryCancelInput {
        id,
        version,
        reason: "wrong".into(),
        confirm_cash_below_zero: true,
    }
}

/// A trip command given the entry of a garden, an open one or a closed one, is `ERR_NOT_FOUND`,
/// and the garden's entry is not touched (the T4-1 case: it used to be cancelled).
#[test]
fn a_trip_command_does_not_find_the_entry_of_a_garden_even_a_closed_one() {
    let w = support::world();
    let v = build(&w);
    let ctx = &w.env.ctx;
    let before = state(&w);

    // While the garden is open.
    not_found_on_id(
        &trips::cost_cancel(
            ctx,
            &w.owner,
            &trip_cancel(v.garden_cost.0, v.garden_cost.1),
        )
        .unwrap_err(),
    );
    not_found_on_id(
        &trips::other_sale_cancel(
            ctx,
            &w.owner,
            &trip_cancel(v.garden_sale.0, v.garden_sale.1),
        )
        .unwrap_err(),
    );
    assert_eq!(state(&w), before, "a refusal writes nothing");

    // Close the garden: its own cancel refuses the cost with ERR_GARDEN_CLOSED, the control.
    let version = gardens::get(
        ctx,
        &typed::<GardenGetInput>(
            &json!({ "id": v.garden, "history_offset": 0, "history_limit": 0 }),
        ),
    )
    .unwrap()
    .row
    .version;
    gardens::close(
        ctx,
        &w.owner,
        &typed::<GardenCloseInput>(
            &json!({ "id": v.garden, "version": version, "date": "10-10-2026" }),
        ),
    )
    .unwrap();
    let closed = gardens::entry_cancel(
        ctx,
        &w.owner,
        &garden_cancel(v.garden_cost.0, v.garden_cost.1),
    )
    .unwrap_err();
    assert_eq!(code_and_field(&closed).0, ErrorCode::ErrGardenClosed);
    let before_closed = state(&w);

    // The trip command must not be a way round that refusal.
    not_found_on_id(
        &trips::cost_cancel(
            ctx,
            &w.owner,
            &trip_cancel(v.garden_cost.0, v.garden_cost.1),
        )
        .unwrap_err(),
    );
    not_found_on_id(
        &trips::other_sale_cancel(
            ctx,
            &w.owner,
            &trip_cancel(v.garden_sale.0, v.garden_sale.1),
        )
        .unwrap_err(),
    );
    assert_eq!(
        state(&w),
        before_closed,
        "the cost of a closed garden is still saved"
    );
}

/// A garden command given the entry of a trip is `ERR_NOT_FOUND` too, and nothing is written.
#[test]
fn a_garden_command_does_not_find_the_entry_of_a_trip() {
    let w = support::world();
    let v = build(&w);
    let ctx = &w.env.ctx;
    let before = state(&w);
    for (id, version) in [v.trip_cost, v.trip_sale] {
        not_found_on_id(
            &gardens::entry_cancel(ctx, &w.owner, &garden_cancel(id, version)).unwrap_err(),
        );
    }
    assert_eq!(state(&w), before);
}

/// The edits and their reviews follow the same rule, in both directions.
#[test]
fn the_edit_of_an_entry_and_its_review_are_for_its_own_kind_of_venture() {
    let w = support::world();
    let v = build(&w);
    let ctx = &w.env.ctx;
    let before = state(&w);

    let trip_cost_edit = |id: i64, version: i64| TripCostEditInput {
        id,
        version,
        cost: TripCostInput {
            trip_id: None,
            date: "10-10-2026".into(),
            amount: "500".into(),
            reason: "truck".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        },
        reason: "fix".into(),
    };
    let trip_sale_edit = |id: i64, version: i64| TripOtherSaleEditInput {
        id,
        version,
        sale: OtherSaleInput {
            trip_id: None,
            date: "10-10-2026".into(),
            shop: Some("Haji Shop".into()),
            purchase_id: None,
            qty: "5".into(),
            amount: "3,100".into(),
            note: None,
        },
        reason: "fix".into(),
        confirm_cash_below_zero: true,
    };
    let garden_edit = |id: i64, version: i64, kind: &str| GardenEntryEditInput {
        id,
        version,
        entry: typed::<GardenEntryInput>(&match kind {
            "cost" => json!({
                "type": "cost", "garden_id": v.garden, "date": "10-10-2026", "amount": "1,100",
                "reason": "labour", "parts": [], "confirm_cash_below_zero": true
            }),
            _ => json!({
                "type": "other_sale", "garden_id": v.garden, "date": "10-10-2026",
                "shop": "Haji Traders", "qty": "5", "amount": "3,100", "note": null
            }),
        }),
        reason: "fix".into(),
        confirm_cash_below_zero: true,
    };

    // A trip edit given the entry of the garden.
    not_found_on_id(
        &edit::trip_cost_preview(ctx, &trip_cost_edit(v.garden_cost.0, v.garden_cost.1))
            .unwrap_err(),
    );
    not_found_on_id(
        &edit::trip_cost_save(
            ctx,
            &w.owner,
            &trip_cost_edit(v.garden_cost.0, v.garden_cost.1),
        )
        .unwrap_err(),
    );
    not_found_on_id(
        &edit::trip_other_sale_preview(ctx, &trip_sale_edit(v.garden_sale.0, v.garden_sale.1))
            .unwrap_err(),
    );
    not_found_on_id(
        &edit::trip_other_sale_save(
            ctx,
            &w.owner,
            &trip_sale_edit(v.garden_sale.0, v.garden_sale.1),
        )
        .unwrap_err(),
    );
    // A garden edit given the entry of the trip.
    not_found_on_id(
        &edit::garden_entry_preview(ctx, &garden_edit(v.trip_cost.0, v.trip_cost.1, "cost"))
            .unwrap_err(),
    );
    not_found_on_id(
        &edit::garden_entry_save(
            ctx,
            &w.owner,
            &garden_edit(v.trip_sale.0, v.trip_sale.1, "other_sale"),
        )
        .unwrap_err(),
    );
    assert_eq!(state(&w), before, "a refused edit writes nothing");

    // The control: the entries of the trip and of the garden can still be edited through their own
    // commands.
    let mut own_sale = trip_sale_edit(v.trip_sale.0, v.trip_sale.1);
    own_sale.sale.trip_id = Some(v.trip);
    own_sale.sale.purchase_id = Some(v.purchase);
    edit::trip_other_sale_save(ctx, &w.owner, &own_sale).unwrap();
    edit::garden_entry_save(
        ctx,
        &w.owner,
        &garden_edit(v.garden_cost.0, v.garden_cost.1, "cost"),
    )
    .unwrap();
}
