//! The request and answer pairs of the Sales, Collections and Business reports and of the search
//! for one-time people, for the frontend (plan 20 step B3).
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used) and `<name>.json` is the answer the real
//! engine gives for it on the PRD worked day (Events 1 to 5), with اکرم given a city and a
//! collector and بکری 1 an اڈا, so the grouped reports have groups. A refusal has the shape of the
//! error a command returns (`ok`, `errors`, `ref`), where `ref` is a stand-in. The frontend copies
//! the files. The test fails if a file differs from the real answer, so a fixture can never go
//! stale; after a deliberate change, write them again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test reports_fixtures_b3`.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::people::{FindPeopleInput, find_people};
use mandi_reports::reports::{ReportPrintInput, ReportRunInput, print_data, run};
use mandi_services::core::error::ErrorCode;
use reports_world::*;

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

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

fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

/// A page of a report for a request, as a pair.
fn run_pair(w: &support::World, name: &str, request: Value) {
    let input: ReportRunInput = typed(&request);
    let page = run(w.env.ctx.db(), &input).unwrap();
    pair(name, &request, &page);
}

fn print_pair(w: &support::World, name: &str, request: Value) {
    let input: ReportPrintInput = typed(&request);
    let data = print_data(w.env.ctx.db(), &input).unwrap();
    pair(name, &request, &data);
}

/// A refusal as a command returns it.
fn refusal_pair(w: &support::World, name: &str, request: Value, code: ErrorCode) {
    let input: ReportRunInput = typed(&request);
    let error = run(w.env.ctx.db(), &input).unwrap_err();
    let mandi_reports::ReportsError::Rejected(items) = error else {
        panic!("not a refusal of the user's entry: {error:?}");
    };
    assert_eq!(items.first().map(|i| i.code), Some(code), "{name}");
    let wire = json!({
        "ok": false,
        "errors": items
            .iter()
            .map(|i| json!({ "code": i.code, "field": i.field, "values": i.values }))
            .collect::<Vec<_>>(),
        "ref": "E-20261010-0001"
    });
    pair(name, &request, &wire);
}

/// The filters with every key, as the frontend sends them. The two keys of step B3 are optional
/// in the type; the fixtures send them so both forms of the request are covered.
fn filters(extra: Value) -> Value {
    let mut base = json!({
        "shortcut": null, "from": "", "to": "", "as_of": "", "party": null,
        "seller_id": null, "zamindar_id": null, "item_id": null, "city_id": null,
        "adda_id": null, "detail": false, "group_by": null, "status": null
    });
    if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
        for (k, v) in extra {
            base.insert(k.clone(), v.clone());
        }
    }
    base
}

/// The worked day with a city and a collector on اکرم and an اڈا on بکری 1.
fn world() -> support::World {
    let w = full_day();
    let conn = raw(&w.dir);
    conn.execute(
        "INSERT INTO cities (code, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES ('1', 'کوٹ رادھا کشن', 'Kot Radha Kishan', 'kot', 'kot radha kishan', 'kt')",
        [],
    )
    .unwrap();
    let city = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO staff (name_ur, name_en, search_ur, search_en, search_phonetic, role)
         VALUES ('اسلم', 'Aslam', 'aslam', 'aslam', 'aslm', 'collector')",
        [],
    )
    .unwrap();
    let staff = conn.last_insert_rowid();
    conn.execute(
        "UPDATE accounts SET city_id = ?1, collector_staff_id = ?2 WHERE id = ?3",
        [city, staff, w.akram],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO addas (code, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES ('1', 'سوات اڈا', 'Swat Adda', 'swat', 'swat adda', 'swt')",
        [],
    )
    .unwrap();
    let adda = conn.last_insert_rowid();
    conn.execute(
        "UPDATE bikri_entries SET adda_id = ?1 WHERE number = 1",
        [adda],
    )
    .unwrap();
    w
}

/// A refusal of the search for people, as a command returns it.
fn people_refusal_pair(w: &support::World, name: &str, request: Value, code: ErrorCode) {
    let input: FindPeopleInput = typed(&request);
    let error = find_people(w.env.ctx.db(), &input).unwrap_err();
    let mandi_reports::ReportsError::Rejected(items) = error else {
        panic!("not a refusal of the user's entry: {error:?}");
    };
    assert_eq!(items.first().map(|i| i.code), Some(code), "{name}");
    let wire = json!({
        "ok": false,
        "errors": items
            .iter()
            .map(|i| json!({ "code": i.code, "field": i.field, "values": i.values }))
            .collect::<Vec<_>>(),
        "ref": "E-20261010-0001"
    });
    pair(name, &request, &wire);
}

#[test]
fn fixtures_are_the_real_answers() {
    let w = world();
    let party = |id: i64| json!({ "account_id": id, "person_key": null });
    let all = |id: &str, extra: Value| json!({ "id": id, "filters": filters(extra), "offset": 0, "limit": 0 });

    // Sales.
    run_pair(
        &w,
        "reports_run_r2_all",
        json!({ "id": "R2", "filters": filters(json!({})), "offset": 0, "limit": 2 }),
    );
    run_pair(
        &w,
        "reports_run_r2_page2",
        json!({ "id": "R2", "filters": filters(json!({})), "offset": 2, "limit": 2 }),
    );
    run_pair(
        &w,
        "reports_run_r2_akram",
        all("R2", json!({ "party": party(w.akram) })),
    );
    run_pair(
        &w,
        "reports_run_r3_cash",
        all("R3", json!({ "shortcut": "today" })),
    );
    run_pair(
        &w,
        "reports_run_r4_credit",
        all("R4", json!({ "item_id": w.banana })),
    );
    run_pair(&w, "reports_run_r6_lots", all("R6", json!({})));
    run_pair(
        &w,
        "reports_run_r7_items",
        all("R7", json!({ "from": "10-10-2026", "to": "10-10-2026" })),
    );
    run_pair(&w, "reports_run_r8_adda", all("R8", json!({})));
    run_pair(&w, "reports_run_r9_all", all("R9", json!({})));
    run_pair(
        &w,
        "reports_run_r9_due",
        all("R9", json!({ "status": "due" })),
    );
    run_pair(&w, "reports_run_r27_series", all("R27", json!({})));
    refusal_pair(
        &w,
        "reports_run_r9_not_a_one_time_account",
        all("R9", json!({ "party": party(w.akram) })),
        ErrorCode::ErrOutOfRange,
    );
    refusal_pair(
        &w,
        "reports_run_r2_unknown_seller",
        all("R2", json!({ "seller_id": 999_999 })),
        ErrorCode::ErrNotFound,
    );

    // Collections.
    run_pair(&w, "reports_run_r20_dues", all("R20", json!({})));
    run_pair(
        &w,
        "reports_run_r20_as_of_10_10",
        all("R20", json!({ "as_of": "10-10-2026" })),
    );
    run_pair(&w, "reports_run_r21_by_city", all("R21", json!({})));
    run_pair(&w, "reports_run_r22_by_collector", all("R22", json!({})));
    run_pair(&w, "reports_run_r24_payables", all("R24", json!({})));
    run_pair(&w, "reports_run_r26_advances", all("R26", json!({})));

    // Business.
    run_pair(
        &w,
        "reports_run_r28_by_day",
        all("R28", json!({ "group_by": "day" })),
    );
    run_pair(&w, "reports_run_r28_by_month", all("R28", json!({})));
    run_pair(&w, "reports_run_r30_profit", all("R30", json!({})));

    // Print data.
    print_pair(
        &w,
        "reports_print_data_r7_en",
        json!({ "id": "R7", "filters": filters(json!({})), "language": "en", "paper": null }),
    );
    print_pair(
        &w,
        "reports_print_data_r20_ur",
        json!({ "id": "R20", "filters": filters(json!({})), "language": "ur", "paper": "a4" }),
    );
    print_pair(
        &w,
        "reports_print_data_r28_month_en",
        json!({ "id": "R28", "filters": filters(json!({})), "language": "en", "paper": null }),
    );
    print_pair(
        &w,
        "reports_print_data_r9_due_en",
        json!({ "id": "R9", "filters": filters(json!({ "status": "due" })), "language": "en", "paper": null }),
    );

    // The search for one-time people.
    let request = json!({ "text": "shahid", "limit": 10 });
    let hits = find_people(w.env.ctx.db(), &typed::<FindPeopleInput>(&request)).unwrap();
    pair("reports_find_people_shahid", &request, &hits);
    let request = json!({ "text": "nasir", "limit": 0 });
    let hits = find_people(w.env.ctx.db(), &typed::<FindPeopleInput>(&request)).unwrap();
    pair("reports_find_people_none", &request, &hits);
    people_refusal_pair(
        &w,
        "reports_find_people_limit_too_big",
        json!({ "text": "shahid", "limit": 51 }),
        ErrorCode::ErrOutOfRange,
    );
}
