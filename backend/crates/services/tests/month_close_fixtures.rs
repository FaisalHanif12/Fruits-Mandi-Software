//! Real answers of the month close commands, as JSON files for the frontend (plan 22 round 7,
//! PRD FR-17.6, FR-21.2): `month_list`, `month_close_preview`, `month_close_save`,
//! `month_close_reverse` and their refusals, on the PRD worked day (October 2026).
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (read
//! back into the real input type before it is used; a command with no input has `null`) and
//! `<name>.json` is the answer the real service gives. A refusal has the error shape of a command
//! (`ok`, `errors`, `ref`), where `ref` is a stand-in. The test compares with the files on disk
//! and FAILS if any differ. After a deliberate change write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test month_close_fixtures`, read the
//! diff and tell `frontend`.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::*;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::month::{MonthInput, MonthReverseInput, close, list, preview, reverse};
use mandi_services::{ServiceError, day};
use reports_world::*;
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
        "ref": "E-20261102-0001"
    })
}

fn refusal(name: &str, request: &Value, error: &ServiceError, code: ErrorCode) {
    assert_eq!(error.code(), code, "{name}");
    pair(name, request, &wire(error));
}

fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

#[test]
fn the_month_close_answers_match_their_fixture_files() {
    let w = full_day();
    // Today is 10-10-26: October is not over.
    let request = json!({ "month": "2026-10" });
    let early = preview(&w.env.ctx, &typed::<MonthInput>(&request)).unwrap_err();
    refusal(
        "month_close_preview_not_finished",
        &request,
        &early,
        ErrorCode::ErrDateInFuture,
    );
    set_today(&w.env, 2026, 11, 2);
    let open_days = close(&w.env.ctx, &w.owner, &typed::<MonthInput>(&request)).unwrap_err();
    refusal(
        "month_close_save_days_open",
        &request,
        &open_days,
        ErrorCode::ErrDayNotClosed,
    );
    let bad = json!({ "month": "2026-13" });
    let bad_month = preview(&w.env.ctx, &typed::<MonthInput>(&bad)).unwrap_err();
    refusal(
        "month_close_preview_bad_month",
        &bad,
        &bad_month,
        ErrorCode::ErrFormatInvalid,
    );
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "31-10-2026".into(),
        },
    )
    .unwrap();
    pair("month_list_ready", &Value::Null, &list(&w.env.ctx).unwrap());
    pair(
        "month_close_preview",
        &request,
        &preview(&w.env.ctx, &typed::<MonthInput>(&request)).unwrap(),
    );
    pair(
        "month_close_save",
        &request,
        &close(&w.env.ctx, &w.owner, &typed::<MonthInput>(&request)).unwrap(),
    );
    let twice = close(&w.env.ctx, &w.owner, &typed::<MonthInput>(&request)).unwrap_err();
    refusal(
        "month_close_save_twice",
        &request,
        &twice,
        ErrorCode::ErrMonthClosed,
    );
    pair(
        "month_list_closed",
        &Value::Null,
        &list(&w.env.ctx).unwrap(),
    );
    let nothing = json!({ "month": "2026-09" });
    let none = close(&w.env.ctx, &w.owner, &typed::<MonthInput>(&nothing)).unwrap_err();
    refusal(
        "month_close_save_nothing_to_close",
        &nothing,
        &none,
        ErrorCode::ErrNothingToClose,
    );
    let no_reason = json!({ "month": "2026-10", "reason": "  " });
    let missing = reverse(
        &w.env.ctx,
        &w.owner,
        &typed::<MonthReverseInput>(&no_reason),
    )
    .unwrap_err();
    refusal(
        "month_close_reverse_no_reason",
        &no_reason,
        &missing,
        ErrorCode::ErrReasonRequired,
    );
    let undo = json!({ "month": "2026-10", "reason": "wrong month" });
    pair(
        "month_close_reverse",
        &undo,
        &reverse(&w.env.ctx, &w.owner, &typed::<MonthReverseInput>(&undo)).unwrap(),
    );
    let again = reverse(&w.env.ctx, &w.owner, &typed::<MonthReverseInput>(&undo)).unwrap_err();
    refusal(
        "month_close_reverse_not_in_force",
        &undo,
        &again,
        ErrorCode::ErrNotFound,
    );
}
