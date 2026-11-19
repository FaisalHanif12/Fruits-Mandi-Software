//! Plan 72 T1: the real answers of `bikri_list` for پچھلی بکری, as JSON files for the frontend (PRD
//! 12.7, FR-12.46, FR-19.11). Each fixture is a pair: `bikri_past_<name>_request.json` is exactly the
//! `input` the command takes (read back into the real input type before it is used) and
//! `bikri_past_<name>.json` is the answer the real service gives for it. A refusal has the shape of
//! the error a command returns (`ok`, `errors`, `ref`), where `ref` is a stand-in because the real
//! reference number is a counter. The frontend copies the files to `frontend/tests/fixtures/`.
//!
//! The data is ten بکری saved on today = 12-11-2026 (each has one lot of 10 نگ and one line of 4 sold
//! to اکرم at 600 + n; the clock stands at 09:00 UTC, so every row was saved at 14:00 Pakistan time):
//!
//! | n | day | seller | item | later |
//! |---|---|---|---|---|
//! | 1 | 29-10 | رحیم | کیلا | |
//! | 2 | 02-11 | کریم | سیب | |
//! | 3, 4 | 05-11 | رحیم سیب, کریم کیلا | | |
//! | 5 | 06-11 | رحیم | کیلا | |
//! | 6 | 08-11 | کریم | کیلا | cancelled |
//! | 7 | 09-11 | رحیم | سیب | edited in the last fixture (rate 609 to 650) |
//! | 8 | 11-11 | کریم | سیب | |
//! | 9, 10 | 12-11 | today | | |
//!
//! The window is 05-11 to 11-11: it has 2 empty days with "show everything" (07-11 and 10-11) and 3
//! without it (08-11 holds only the cancelled one). The test compares the answers with the files on
//! disk and FAILS if any differ. After a deliberate change, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test past_bikri_fixtures`, read the diff and
//! tell `frontend`.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;

use std::path::{Path, PathBuf};

use common::set_today;
use mandi_services::ServiceError;
use mandi_services::bikri;
use mandi_services::core::bikri::{BikriListInput, BikriSaved, CancelInput};
use mandi_services::core::error::ErrorCode;
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

/// One pair of files: the request and the answer.
fn pair<A: Serialize>(name: &str, request: &Value, answer: &A) {
    write_or_check(
        &format!("bikri_past_{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    write_or_check(
        &format!("bikri_past_{name}.json"),
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
        "ref": "E-20261112-0001"
    })
}

/// The request read back into the real input type, so the file is exactly what the command takes.
fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

fn bikri_on(w: &World, n: i64, date: &str, seller: i64, item: i64) -> BikriSaved {
    let lot = support::with_sales(
        support::bare_lot(w, "lot-1", seller, item, "10"),
        vec![support::sale("s-1", w.akram, "4", &format!("{}", 600 + n))],
    );
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = date.into();
    support::save_old(w, &draft, false)
}

/// A request and the answer of the real service for it.
fn list_pair(w: &World, name: &str, request: Value) {
    let answer = bikri::list(&w.env.ctx, &typed::<BikriListInput>(&request)).unwrap();
    pair(name, &request, &answer);
}

#[test]
fn the_real_answers_of_the_past_sales_list_are_the_fixtures() {
    let w = support::world();
    set_today(&w.env, 2026, 11, 12);
    let (rahim, karim, banana, apple) = (w.rahim, w.karim, w.banana, w.apple);
    bikri_on(&w, 1, "29-10-2026", rahim, banana);
    bikri_on(&w, 2, "02-11-2026", karim, apple);
    bikri_on(&w, 3, "05-11-2026", rahim, apple);
    bikri_on(&w, 4, "05-11-2026", karim, banana);
    bikri_on(&w, 5, "06-11-2026", rahim, banana);
    let six = bikri_on(&w, 6, "08-11-2026", karim, banana);
    let seven = bikri_on(&w, 7, "09-11-2026", rahim, apple);
    bikri_on(&w, 8, "11-11-2026", karim, apple);
    bikri_on(&w, 9, "12-11-2026", rahim, banana);
    bikri_on(&w, 10, "12-11-2026", karim, apple);
    let detail = bikri::get(&w.env.ctx, six.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: six.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();

    // Today's section: one day, two بکری.
    list_pair(
        &w,
        "today",
        json!({ "from": "12-11-2026", "to": "12-11-2026", "group_by_day": true, "offset": 0, "limit": 25 }),
    );
    // The first window ends yesterday. 08-11 holds only the cancelled بکری, so it shows as empty.
    let window = json!({ "from": "05-11-2026", "to": "11-11-2026", "group_by_day": true, "offset": 0, "limit": 25 });
    list_pair(&w, "window", window.clone());
    // "Show everything": the filters are set aside and the cancelled بکری shows on its day.
    list_pair(
        &w,
        "window_all",
        json!({ "from": "05-11-2026", "to": "11-11-2026", "group_by_day": true, "all": true,
                "seller_id": w.karim, "offset": 0, "limit": 25 }),
    );
    // A filter: only رحیم.
    list_pair(
        &w,
        "window_seller",
        json!({ "from": "05-11-2026", "to": "11-11-2026", "group_by_day": true,
                "seller_id": w.rahim, "offset": 0, "limit": 25 }),
    );
    // One step back, and a second page of a window.
    list_pair(
        &w,
        "window_back",
        json!({ "from": "29-10-2026", "to": "04-11-2026", "group_by_day": true, "offset": 0, "limit": 25 }),
    );
    list_pair(
        &w,
        "window_second_page",
        json!({ "from": "05-11-2026", "to": "11-11-2026", "group_by_day": true, "offset": 2, "limit": 2 }),
    );
    // A window with no بکری: every day is there with 0.
    list_pair(
        &w,
        "window_empty",
        json!({ "from": "13-10-2026", "to": "19-10-2026", "group_by_day": true, "offset": 0, "limit": 25 }),
    );
    // A date after today is refused, with today in the answer.
    let future = json!({ "from": "05-11-2026", "to": "13-11-2026", "group_by_day": true, "offset": 0, "limit": 25 });
    let error = bikri::list(&w.env.ctx, &typed::<BikriListInput>(&future)).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrDateInFuture);
    pair("future_refused", &future, &wire(&error));
    // The window of 09-11 after an edit: the rate of number 7 goes from 609 to 650. The row, its day
    // heading and the window totals show the new values and carry the edit mark (PRD FR-7.71).
    let opened = tools::open(&w, seven.bikri_id, seven.entry_id);
    let mut draft = opened.draft.clone();
    draft
        .lots
        .first_mut()
        .unwrap()
        .sales
        .first_mut()
        .unwrap()
        .seller_rate = "650".into();
    tools::edit(&w, &opened, &draft);
    list_pair(&w, "window_after_edit", window);
}
