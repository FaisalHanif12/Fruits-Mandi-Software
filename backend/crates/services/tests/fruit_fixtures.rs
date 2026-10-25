//! Real answers of the fruit commands, as JSON files for the frontend (plan 17d, T2).
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used), and `<name>.json` is the answer the real
//! service gives for it. A refusal is a pair too: its answer has the shape of the error a command
//! returns (`ok`, `errors`, `ref`), where `ref` is a stand-in because the real reference number is a
//! counter. The frontend copies the files to `frontend/tests/fixtures/`.
//!
//! The test compares the answers with the files on disk and FAILS if any differ, so a fixture can
//! never go stale. After a deliberate change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test fruit_fixtures`, look at the diff,
//! and tell `frontend`.
//!
//! Story A is E2E-42 on the world after E2E-16 (5 کیلا left in رحیم's lot): home fruit for الف from the
//! lot, guest fruit from حاجی نذیر's shop, the lists, an edit, a cancel and the refusals. Story B is a
//! refusal that needs a lot received after the day typed.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::set_today;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::bikri::{BikriSaveInput, CancelInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::fruit::{
    FruitCancelInput, FruitEditInput, FruitGetInput, FruitInput, FruitListInput, FruitLotsInput,
};
use mandi_services::core::statement::PartnersStatementInput;
use mandi_services::{ServiceError, bikri, fruit, partners};
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

fn home_json(w: &World, qty: &str, rate: &str) -> Value {
    json!({
        "for_guests": false, "partner_id": w.partners[0], "date": "10-10-2026", "source": "shop",
        "lot_id": 1, "agent_account_id": null, "item_id": null, "packing_id": null,
        "qty": qty, "weight": null, "rate": rate, "note": null, "editing_id": null
    })
}

fn guests_json(w: &World, agent: i64, qty: &str, rate: &str) -> Value {
    json!({
        "for_guests": true, "partner_id": null, "date": "10-10-2026", "source": "other_shop",
        "lot_id": null, "agent_account_id": agent, "item_id": w.apple, "packing_id": w.box_,
        "qty": qty, "weight": null, "rate": rate, "note": null, "editing_id": null
    })
}

/// The world after E2E-16: رحیم's lot of 50 کیلا with 45 sold and 5 left.
fn after_e2e_16() -> World {
    let w = support::world();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    w
}

fn nazeer(w: &World) -> i64 {
    support::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "حاجی نذیر",
        "Haji Nazeer",
        true,
        "",
    )
}

#[test]
fn fruit_fixtures_of_e2e_42() {
    let w = after_e2e_16();
    let ctx = &w.env.ctx;
    let owner = &w.owner;
    let agent = nazeer(&w);

    // The lot picker: 5 left in the lot. A search by the seller's name; a search with no match.
    pair(
        "fruit_lots_all",
        &json!({}),
        &fruit::lots(ctx, &FruitLotsInput::default()).unwrap(),
    );
    let search = json!({ "search": "rahim", "limit": 0 });
    pair(
        "fruit_lots_search",
        &search,
        &fruit::lots(ctx, &typed::<FruitLotsInput>(&search)).unwrap(),
    );

    // Home fruit: 2 x 580 = 1,160 for الف from the lot.
    let home = home_json(&w, "2", "580");
    pair(
        "fruit_preview_home",
        &home,
        &fruit::preview(ctx, &typed::<FruitInput>(&home)).unwrap(),
    );
    // More than the lot has left, a rate of 0 and a closed day come back inside the preview.
    let too_many = home_json(&w, "6", "580");
    pair(
        "fruit_preview_qty_exceeds_lot",
        &too_many,
        &fruit::preview(ctx, &typed::<FruitInput>(&too_many)).unwrap(),
    );
    let error = fruit::save(ctx, owner, &typed::<FruitInput>(&too_many)).unwrap_err();
    refusal(
        "fruit_save_qty_exceeds_lot",
        &too_many,
        &error,
        ErrorCode::ErrQtyExceedsLot,
    );
    let no_rate = home_json(&w, "2", "0");
    pair(
        "fruit_preview_rate_zero",
        &no_rate,
        &fruit::preview(ctx, &typed::<FruitInput>(&no_rate)).unwrap(),
    );
    let home_saved = fruit::save(ctx, owner, &typed::<FruitInput>(&home)).unwrap();
    pair("fruit_save_home", &home, &home_saved);

    // Guest fruit: 3 x 1,000 = 3,000 from حاجی نذیر's shop, split 1,000 / 1,000 / 1,000. One rupee
    // over three partners is 1 / 0 / 0: the parts of 0 are left out.
    let guests = guests_json(&w, agent, "3", "1,000");
    pair(
        "fruit_preview_guests",
        &guests,
        &fruit::preview(ctx, &typed::<FruitInput>(&guests)).unwrap(),
    );
    let guests_saved = fruit::save(ctx, owner, &typed::<FruitInput>(&guests)).unwrap();
    pair("fruit_save_guests", &guests, &guests_saved);
    let one_rupee = guests_json(&w, agent, "1", "1");
    pair(
        "fruit_preview_guests_one_rupee",
        &one_rupee,
        &fruit::preview(ctx, &typed::<FruitInput>(&one_rupee)).unwrap(),
    );

    // An item sold by weight: 12.345 kg at 100 per kg = 1,235. The weight is required.
    let grapes = support::add_item(&w.env, owner, "انگور", "Grapes", "fruit", "weight");
    let mut weighed = guests_json(&w, agent, "1", "100");
    weighed["item_id"] = json!(grapes);
    pair(
        "fruit_preview_weight_required",
        &weighed,
        &fruit::preview(ctx, &typed::<FruitInput>(&weighed)).unwrap(),
    );
    weighed["weight"] = json!("12.345");
    pair(
        "fruit_preview_weight_item",
        &weighed,
        &fruit::preview(ctx, &typed::<FruitInput>(&weighed)).unwrap(),
    );

    // The screens that read them.
    pair(
        "fruit_get_home",
        &json!({ "id": home_saved.id }),
        &fruit::get(
            ctx,
            &typed::<FruitGetInput>(&json!({ "id": home_saved.id })),
        )
        .unwrap(),
    );
    pair(
        "fruit_list_all",
        &json!({}),
        &fruit::list(ctx, &FruitListInput::default()).unwrap(),
    );
    let guests_only = json!({ "guests_only": true });
    pair(
        "fruit_list_guests_only",
        &guests_only,
        &fruit::list(ctx, &typed::<FruitListInput>(&guests_only)).unwrap(),
    );
    let for_alif = json!({ "partner_id": w.partners[0], "offset": 0, "limit": 1 });
    pair(
        "fruit_list_partner_paged",
        &for_alif,
        &fruit::list(ctx, &typed::<FruitListInput>(&for_alif)).unwrap(),
    );
    let statement =
        json!({ "partner_id": w.partners[0], "from": null, "to": null, "offset": 0, "limit": 0 });
    pair(
        "partners_statement_fruit_alif",
        &statement,
        &partners::statement(ctx, &typed::<PartnersStatementInput>(&statement)).unwrap(),
    );

    // Edit 2 -> 3 نگ: the preview with editing_id gives the old 2 back to the lot.
    let mut edited = home_json(&w, "3", "580");
    edited["editing_id"] = json!(home_saved.id);
    pair(
        "fruit_preview_edit_home",
        &edited,
        &fruit::preview(ctx, &typed::<FruitInput>(&edited)).unwrap(),
    );
    let edit = json!({
        "id": home_saved.id, "version": home_saved.version, "reason": "wrong quantity",
        "entry": edited
    });
    let done = fruit::edit(ctx, owner, &typed::<FruitEditInput>(&edit)).unwrap();
    pair("fruit_edit_home", &edit, &done);
    pair(
        "fruit_get_home_edited",
        &json!({ "id": home_saved.id }),
        &fruit::get(
            ctx,
            &typed::<FruitGetInput>(&json!({ "id": home_saved.id })),
        )
        .unwrap(),
    );

    // While fruit is taken from its lot, the بکری cannot be cancelled.
    let cancel_bikri = json!({
        "id": 1, "version": bikri::get(ctx, 1).unwrap().version, "reason": "wrong",
        "confirm_cash_below_zero": false
    });
    let error = bikri::cancel(ctx, owner, &typed::<CancelInput>(&cancel_bikri)).unwrap_err();
    refusal(
        "bikri_cancel_has_fruit",
        &cancel_bikri,
        &error,
        ErrorCode::ErrBikriHasFruit,
    );

    // Cancel the edited entry by reversal; a second cancel is refused.
    let cancel = json!({ "id": home_saved.id, "version": done.version, "reason": "not taken" });
    let cancelled = fruit::cancel(ctx, owner, &typed::<FruitCancelInput>(&cancel)).unwrap();
    pair("fruit_cancel_home", &cancel, &cancelled);
    let error = fruit::cancel(ctx, owner, &typed::<FruitCancelInput>(&cancel)).unwrap_err();
    refusal(
        "fruit_cancel_again",
        &cancel,
        &error,
        ErrorCode::ErrFruitCancelled,
    );
    pair(
        "fruit_get_home_cancelled",
        &json!({ "id": home_saved.id }),
        &fruit::get(
            ctx,
            &typed::<FruitGetInput>(&json!({ "id": home_saved.id })),
        )
        .unwrap(),
    );
    let cancelled_only = json!({ "status": "cancelled" });
    pair(
        "fruit_list_cancelled",
        &cancelled_only,
        &fruit::list(ctx, &typed::<FruitListInput>(&cancelled_only)).unwrap(),
    );
}

#[test]
fn fruit_fixtures_of_a_day_before_the_lot_was_received() {
    let w = after_e2e_16();
    let ctx = &w.env.ctx;
    // Today is 11-10 and a lot is received on that day; fruit dated 10-10 is before it.
    set_today(&w.env, 2026, 10, 11);
    let mut draft = support::draft(vec![support::with_sales(
        support::bare_lot(&w, "later-1", w.karim, w.apple, "10"),
        vec![support::sale("s-1", w.akram, "8", "1,200")],
    )]);
    draft.header.entry_date = "11-10-2026".into();
    let saved = bikri::save(
        ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let lot_id = {
        let detail = bikri::get(ctx, saved.bikri_id).unwrap();
        detail.lots[0].id
    };
    let mut early = home_json(&w, "1", "1,200");
    early["lot_id"] = json!(lot_id);
    let error = fruit::save(ctx, &w.owner, &typed::<FruitInput>(&early)).unwrap_err();
    refusal(
        "fruit_save_date_before_received",
        &early,
        &error,
        ErrorCode::ErrDateBeforeReceived,
    );
}
