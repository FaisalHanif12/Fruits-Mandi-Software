//! Real answers of the plan 13 commands, as JSON files for the frontend (plan 13, T4).
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used), and `<name>.json` is the answer the
//! real service gives for it on the PRD worked day. The frontend copies the files to
//! `frontend/tests/fixtures/`.
//!
//! The test compares the answer with the file on disk and FAILS if they differ, so a fixture can
//! never go stale. After a deliberate change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test bikri_save fixtures`, look at the
//! diff, and tell `frontend`.

use std::path::{Path, PathBuf};

use mandi_services::bikri;
use mandi_services::core::accounts::{PartyCardInput, SearchAccountsInput, SearchGlobalInput};
use mandi_services::core::bikri::{BikriListInput, BikriSaveInput};
use mandi_services::core::chukta::{ChuktaAddInput, ChuktaSearchInput};
use mandi_services::core::drafts::{DraftGetInput, DraftSaveInput};
use mandi_services::{assist, drafts, search};
use serde::Serialize;
use serde_json::{Value, json};

use crate::support::*;

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

/// The JSON of one sale row as the screen sends it.
fn row(key: &str, buyer: i64, qty: &str, rate: &str) -> Value {
    json!({ "key": key, "buyer_account_id": buyer, "chukta": null, "qty": qty, "weight": null,
        "seller_rate": rate, "rate_differs": false, "buyer_rate": null, "paid": null,
        "seller_comm": null, "buyer_comm": null, "laga": null, "note": null })
}

/// What `bikri_save` takes inside `input`: Event 1, Rahim's 50 banana (PRD 4.4).
fn event1_request(w: &World) -> Value {
    json!({
        "draft": {
            "header": {
                "entry_date": "10-10-2026", "bill_date": "", "book": null, "truck_no": "LES-1234",
                "zamindar_account_id": w.rahim, "adda_id": null, "driver_name": null,
                "driver_phone": null, "total_freight": null, "store_id": null, "note": null
            },
            "lots": [{
                "key": "lot-1", "seller_account_id": w.rahim, "item_id": w.banana, "packing_id": w.box_, "marka_id": null, "store_id": null,
                "qty_received": "50", "weight_received": null, "freight": "3,000",
                "freight_paid_by": "firm",
                "labour": { "method": "per_unit", "value": "5" },
                "munshiana": { "method": "manual", "value": "50" },
                "phone": { "method": "manual", "value": "20" },
                "cash_paid": "5,000", "store_rent": null, "adjust_advance": true,
                "sales": [
                    row("line-1", w.akram, "20", "600"),
                    row("line-2", w.arshad, "30", "580")
                ]
            }]
        },
        "confirm_cash_below_zero": false
    })
}

/// Event 2: Karim's 10 apple, sold to the چکتا buyer Shahid in cash (PRD 4.5).
fn event2_request(w: &World) -> Value {
    let mut line = row("line-1", w.chukta_khata, "10", "1200");
    line["chukta"] = json!({ "name_ur": "شاہد", "name_en": "Shahid", "phone": "0300-0000000" });
    json!({
        "draft": {
            "header": {
                "entry_date": "10-10-2026", "bill_date": "", "book": null, "truck_no": "LHR-0077",
                "zamindar_account_id": w.karim, "adda_id": null, "driver_name": null,
                "driver_phone": null, "total_freight": null, "store_id": null, "note": null
            },
            "lots": [{
                "key": "lot-1", "seller_account_id": w.karim, "item_id": w.apple, "packing_id": w.box_, "marka_id": null, "store_id": null,
                "qty_received": "10", "weight_received": null, "freight": "1,000",
                "freight_paid_by": "firm",
                "labour": { "method": "per_unit", "value": "5" },
                "munshiana": { "method": "manual", "value": "50" },
                "phone": { "method": "manual", "value": "20" },
                "cash_paid": null, "store_rent": null, "adjust_advance": true,
                "sales": [line]
            }]
        },
        "confirm_cash_below_zero": false
    })
}

/// The request read back into the real input type, so the file is exactly what the command takes.
fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

#[test]
fn the_real_answers_of_the_plan_13_commands_are_the_fixtures() {
    let w = world();

    // bikri_save: Event 1 and Event 2.
    let request1 = event1_request(&w);
    let saved1 = bikri::save(&w.env.ctx, &w.owner, &typed::<BikriSaveInput>(&request1)).unwrap();
    pair("bikri_save_event1", &request1, &saved1);
    let request2 = event2_request(&w);
    let saved2 = bikri::save(&w.env.ctx, &w.owner, &typed::<BikriSaveInput>(&request2)).unwrap();
    pair("bikri_save_event2", &request2, &saved2);
    // The golden numbers of the worked day, so a fixture of wrong numbers cannot be written.
    assert_eq!((saved1.number, saved2.number), (1, 2));
    assert_eq!(saved1.preview.summary.net.rupees(), 19_316);
    assert_eq!(saved2.preview.summary.net.rupees(), 10_160);

    // bikri_get: both, with their saved totals.
    for (name, saved) in [("bikri_get_event1", &saved1), ("bikri_get_event2", &saved2)] {
        let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
        pair(name, &json!({ "id": saved.bikri_id }), &detail);
    }

    // bikri_list: the day, newest first.
    let list_request = json!({ "date": "10-10-2026", "offset": 0, "limit": 0 });
    let rows = bikri::list(&w.env.ctx, &typed::<BikriListInput>(&list_request)).unwrap();
    assert_eq!(
        rows.rows.iter().map(|r| r.number).collect::<Vec<_>>(),
        [2, 1],
        "the list is newest first"
    );
    pair("bikri_list_day", &list_request, &rows);

    // chukta_search and chukta_add (plan 64): Shahid is in the list after his line.
    let search_request = json!({ "text": "shahid" });
    let found =
        mandi_services::chukta::search(&w.env.ctx, &typed::<ChuktaSearchInput>(&search_request))
            .unwrap();
    assert_eq!(found.len(), 1);
    pair("chukta_search_shahid", &search_request, &found);
    // The buyer picker lists him beside ordinary buyers (`kind` chukta, id of 0030, his own id), and
    // his card has his name and phone and no balance.
    let hits_request = json!({ "scope": "buyer", "control_id": null, "text": "shahid", "limit": 10, "offset": 0, "include_dead": null, "with_chukta": true });
    let hits = search::search_accounts(
        &w.env.ctx,
        &w.owner,
        &typed::<SearchAccountsInput>(&hits_request),
    )
    .unwrap();
    assert_eq!(hits.len(), 1);
    pair("search_accounts_chukta_buyer", &hits_request, &hits);
    let card_request = json!({ "account_id": w.chukta_khata, "chukta_id": found[0].id });
    let card_input: PartyCardInput = typed(&card_request);
    let card = assist::chukta_card(&w.env.ctx, card_input.chukta_id.unwrap()).unwrap();
    pair("assist_party_card_chukta_buyer", &card_request, &card);
    let add_request = json!({ "name_ur": "ناصر", "name_en": "Nasir", "phone": "0300-1111111" });
    let added =
        mandi_services::chukta::add(&w.env.ctx, &w.owner, &typed::<ChuktaAddInput>(&add_request))
            .unwrap();
    pair("chukta_add_nasir", &add_request, &added);
    let again =
        mandi_services::chukta::add(&w.env.ctx, &w.owner, &typed::<ChuktaAddInput>(&add_request))
            .unwrap();
    assert_eq!(added, again, "the same person is returned, not made twice");
    let bad_request = json!({ "name_ur": "", "name_en": "Nasir", "phone": "12" });
    let error =
        mandi_services::chukta::add(&w.env.ctx, &w.owner, &typed::<ChuktaAddInput>(&bad_request))
            .unwrap_err();
    let mandi_services::ServiceError::Rejected(items) = &error else {
        panic!("not a refusal of the user's entry: {error:?}");
    };
    let wire = json!({
        "ok": false,
        "errors": items
            .iter()
            .map(|i| json!({ "code": i.code, "field": i.field, "values": i.values }))
            .collect::<Vec<_>>(),
        "ref": "E-20261010-0001"
    });
    pair("chukta_add_refused", &bad_request, &wire);

    // drafts_save and drafts_get: the unsaved work of the screen (a save removes the draft, so
    // this comes after the saves).
    let payload = serde_json::to_string(&request1["draft"]).unwrap();
    let save_request = json!({ "screen": "bikri", "payload_json": payload });
    let draft_saved = drafts::save(
        &w.env.ctx,
        &w.owner,
        &typed::<DraftSaveInput>(&save_request),
    )
    .unwrap();
    pair("drafts_save_bikri", &save_request, &draft_saved);
    let get_request = json!({ "screen": "bikri" });
    let draft = drafts::get(&w.env.ctx, &w.owner, &typed::<DraftGetInput>(&get_request))
        .unwrap()
        .unwrap();
    pair("drafts_get_bikri", &get_request, &draft);

    // search_global: a بکری found by its number, and by the start of its truck number.
    for (name, text) in [
        ("search_global_bikri_number", "1"),
        ("search_global_bikri_truck", "LES-1234"),
    ] {
        let request = json!({ "text": text, "limit": 10 });
        let hits =
            search::global(&w.env.ctx, &w.owner, &typed::<SearchGlobalInput>(&request)).unwrap();
        assert!(
            serde_json::to_value(&hits)
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .any(|h| h["kind"] == "bikri"),
            "{text} must find a بکری"
        );
        pair(name, &request, &hits);
    }
}
