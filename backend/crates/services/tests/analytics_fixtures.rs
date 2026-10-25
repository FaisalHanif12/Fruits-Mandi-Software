//! The request and answer pairs of the analytics commands, for the frontend (plan 21 step B4).
//!
//! Each fixture is a pair: `analytics_<name>_request.json` is exactly the `input` the command
//! takes (it is read back into the real input type before it is used) and
//! `analytics_<name>.json` is the answer the real engine gives for it on the PRD worked day
//! (Events 1 to 5). A refusal has the shape of the error a command returns (`ok`, `errors`,
//! `ref`), where `ref` is a stand-in. The frontend copies the files. The test fails if a file
//! differs from the real answer, so a fixture can never go stale; after a deliberate change,
//! write them again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test analytics_fixtures`.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::analytics::{
    AnalyticsLinesInput, AnalyticsPrintInput, AnalyticsRunInput, AnalyticsViewsSaveInput, catalog,
    lines, print_data, run, views_get, views_save,
};
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

fn answer_only<A: Serialize>(name: &str, answer: &A) {
    write_or_check(
        &format!("analytics_{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

fn pair<A: Serialize>(name: &str, request: &Value, answer: &A) {
    write_or_check(
        &format!("analytics_{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    answer_only(name, answer);
}

fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

/// The filters with every key, as the frontend sends them.
fn filters(extra: Value) -> Value {
    let mut base = json!({
        "period": null, "from": "", "to": "", "group_by": null, "item_id": null,
        "seller_id": null, "buyer": null, "city_id": null, "adda_id": null,
        "side": "both", "compare": "off", "partner_id": null, "trip_id": null,
        "goods": "all", "trip_view": null, "garden_id": null
    });
    if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
        for (k, v) in extra {
            base.insert(k.clone(), v.clone());
        }
    }
    base
}

fn run_pair(w: &support::World, name: &str, request: Value) {
    let input: AnalyticsRunInput = typed(&request);
    let page = run(w.env.ctx.db(), &input).unwrap();
    pair(name, &request, &page);
}

fn lines_pair(w: &support::World, name: &str, request: Value) {
    let input: AnalyticsLinesInput = typed(&request);
    let page = lines(w.env.ctx.db(), &input).unwrap();
    pair(name, &request, &page);
}

fn print_pair(w: &support::World, name: &str, request: Value) {
    let input: AnalyticsPrintInput = typed(&request);
    let data = print_data(w.env.ctx.db(), &input).unwrap();
    pair(name, &request, &data);
}

/// A refusal as a command returns it.
fn refusal_pair(
    name: &str,
    request: &Value,
    items: Vec<mandi_services::core::error::FieldError>,
    code: ErrorCode,
) {
    assert_eq!(items.first().map(|i| i.code), Some(code), "{name}");
    let wire = json!({
        "ok": false,
        "errors": items
            .iter()
            .map(|i| json!({ "code": i.code, "field": i.field, "values": i.values }))
            .collect::<Vec<_>>(),
        "ref": "E-20261010-0001"
    });
    pair(name, request, &wire);
}

fn refusal_run(w: &support::World, name: &str, request: Value, code: ErrorCode) {
    let input: AnalyticsRunInput = typed(&request);
    let mandi_reports::ReportsError::Rejected(items) = run(w.env.ctx.db(), &input).unwrap_err()
    else {
        panic!("not a refusal of the user's entry");
    };
    refusal_pair(name, &request, items, code);
}

#[test]
fn fixtures_are_the_real_answers() {
    let w = full_day();
    answer_only("catalog", &catalog());

    let page = |view: &str, extra: Value| json!({ "view": view, "filters": filters(extra), "offset": 0, "limit": 0 });
    run_pair(&w, "run_a", page("A", json!({})));
    run_pair(
        &w,
        "run_a_seller_side",
        page("A", json!({ "side": "seller" })),
    );
    run_pair(
        &w,
        "run_a_compare",
        page(
            "A",
            json!({ "from": "10-10-2026", "to": "10-10-2026", "compare": "previous" }),
        ),
    );
    run_pair(&w, "run_b", page("B", json!({})));
    run_pair(&w, "run_c", page("C", json!({})));
    run_pair(&w, "run_d_month", page("D", json!({})));
    run_pair(&w, "run_d_day", page("D", json!({ "group_by": "day" })));
    run_pair(&w, "run_e", page("E", json!({})));
    run_pair(&w, "run_h", page("H", json!({})));
    // Round 11: the partner table, the trend and the trips on the worked day (no trip yet).
    run_pair(&w, "run_f", page("F", json!({})));
    run_pair(&w, "run_g_month", page("G", json!({})));
    run_pair(&w, "run_g_day", page("G", json!({ "group_by": "day" })));
    run_pair(&w, "run_j_empty", page("J", json!({})));
    run_pair(
        &w,
        "run_a_goods_trip",
        page("A", json!({ "goods": "trip" })),
    );
    run_pair(
        &w,
        "run_a_page2",
        json!({ "view": "A", "filters": filters(json!({})), "offset": 1, "limit": 1 }),
    );

    // The lines behind a row of A, a cell of D and a cell of E.
    lines_pair(
        &w,
        "lines_a_banana",
        json!({ "view": "A", "filters": filters(json!({})), "row_key": { "item_id": w.banana },
                "column_key": null, "offset": 0, "limit": 0 }),
    );
    lines_pair(
        &w,
        "lines_d_banana_october",
        json!({ "view": "D", "filters": filters(json!({})), "row_key": { "item_id": w.banana },
                "column_key": { "from": "2026-10-01", "to": "2026-10-31" }, "offset": 0, "limit": 0 }),
    );
    lines_pair(
        &w,
        "lines_e_rahim_banana",
        json!({ "view": "E", "filters": filters(json!({})), "row_key": { "seller_id": w.rahim },
                "column_key": { "item_id": w.banana }, "offset": 0, "limit": 0 }),
    );

    lines_pair(
        &w,
        "lines_f_partner_a",
        json!({ "view": "F", "filters": filters(json!({})), "row_key": { "partner_id": w.partners[0] },
                "column_key": null, "offset": 0, "limit": 0 }),
    );

    print_pair(
        &w,
        "print_data_a_en",
        json!({ "view": "A", "filters": filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })),
                "language": "en", "paper": null }),
    );
    print_pair(
        &w,
        "print_data_d_ur",
        json!({ "view": "D", "filters": filters(json!({})), "language": "ur", "paper": "a4" }),
    );

    // Saved views: an empty list, then two saved, then the list as stored.
    answer_only(
        "views_get_empty",
        &views_get(w.env.ctx.db(), w.owner.user_id).unwrap(),
    );
    let save_request = json!({ "views": [
        { "name": "کیلا کمیشن ماہانہ", "view": "D",
          "filters": filters(json!({ "item_id": w.banana, "from": "01-01-2026", "to": "31-12-2026" })) },
        { "name": "Top 10", "view": "H", "filters": filters(json!({})) }
    ] });
    let input: AnalyticsViewsSaveInput = typed(&save_request);
    let saved = views_save(w.env.ctx.db(), w.owner.user_id, &input).unwrap();
    pair("views_save_two", &save_request, &saved);
    answer_only(
        "views_get_two",
        &views_get(w.env.ctx.db(), w.owner.user_id).unwrap(),
    );
    let bad_request = json!({ "views": [
        { "name": "  ", "view": "A", "filters": filters(json!({ "item_id": 9999 })) }
    ] });
    let input: AnalyticsViewsSaveInput = typed(&bad_request);
    let mandi_reports::ReportsError::Rejected(items) =
        views_save(w.env.ctx.db(), w.owner.user_id, &input).unwrap_err()
    else {
        panic!("not a refusal");
    };
    refusal_pair(
        "views_save_bad",
        &bad_request,
        items,
        ErrorCode::ErrRequired,
    );

    // Round 11: a trip on its own world (E2E-34 after the sale of 15, then the 5 written off).
    trip_fixtures();
    // Round 11: a garden on its own world (E2E-40, closed).
    garden_fixtures();

    // Refusals.
    refusal_run(
        &w,
        "run_f_compare_refused",
        page(
            "F",
            json!({ "from": "01-10-2026", "to": "31-10-2026", "compare": "previous" }),
        ),
        ErrorCode::ErrOutOfRange,
    );
    refusal_run(
        &w,
        "run_a_group_by_refused",
        page("A", json!({ "group_by": "day" })),
        ErrorCode::ErrOutOfRange,
    );
    refusal_run(
        &w,
        "run_a_compare_needs_dates",
        page("A", json!({ "compare": "last_year" })),
        ErrorCode::ErrRequired,
    );
    refusal_run(
        &w,
        "run_a_unknown_item",
        page("A", json!({ "item_id": 9999 })),
        ErrorCode::ErrNotFound,
    );
}

/// The trip views on the trip of E2E-34. The frontend copies these to draw the trip layouts.
fn trip_fixtures() {
    let (w, _trip, purchase, _sale) = trip_34_world();
    let page = |view: &str, extra: Value| json!({ "view": view, "filters": filters(extra), "offset": 0, "limit": 0 });
    run_pair(&w, "run_f_trip", page("F", json!({})));
    run_pair(
        &w,
        "run_g_trip_day",
        page("G", json!({ "group_by": "day" })),
    );
    run_pair(&w, "run_j_trip", page("J", json!({})));
    run_pair(
        &w,
        "run_j_trip_by_purchase",
        page("J", json!({ "trip_view": "purchase" })),
    );
    run_pair(
        &w,
        "run_j_trip_partner",
        page("J", json!({ "partner_id": w.partners[0] })),
    );
    run_pair(
        &w,
        "run_a_goods_consigned",
        page("A", json!({ "goods": "consigned" })),
    );
    let first = run(
        w.env.ctx.db(),
        &typed::<AnalyticsRunInput>(&page("J", json!({}))),
    )
    .unwrap();
    let key = first.rows[0].key.clone().unwrap();
    lines_pair(
        &w,
        "lines_j_trip",
        json!({ "view": "J", "filters": filters(json!({})), "row_key": key,
                "column_key": null, "offset": 0, "limit": 0 }),
    );
    print_pair(
        &w,
        "print_data_j_en",
        json!({ "view": "J", "filters": filters(json!({})), "language": "en", "paper": null }),
    );
    // The five that are left spoil on 11-10-26: the loss shows on its own day.
    set_today(&w.env, 2026, 10, 11);
    mandi_services::purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::purchases::PurchaseWriteOffInput {
            purchase_id: purchase,
            qty: "5".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
    run_pair(
        &w,
        "run_g_trip_written_off",
        page("G", json!({ "group_by": "day" })),
    );
    run_pair(&w, "run_j_trip_written_off", page("J", json!({})));
}

/// The garden views on the garden of E2E-40, closed on 13-10-26. The frontend copies these to draw
/// view I and the gardens columns.
fn garden_fixtures() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let page = |view: &str, extra: Value| json!({ "view": view, "filters": filters(extra), "offset": 0, "limit": 0 });
    run_pair(&w, "run_i_garden", page("I", json!({})));
    run_pair(
        &w,
        "run_i_garden_seller_side",
        page("I", json!({ "side": "seller" })),
    );
    run_pair(&w, "run_f_garden", page("F", json!({})));
    run_pair(
        &w,
        "run_g_garden_day",
        page("G", json!({ "group_by": "day" })),
    );
    run_pair(
        &w,
        "run_a_garden_goods",
        page("A", json!({ "garden_id": garden.id })),
    );
    let first = run(
        w.env.ctx.db(),
        &typed::<AnalyticsRunInput>(&page("I", json!({}))),
    )
    .unwrap();
    let key = first.rows[0].key.clone().unwrap();
    lines_pair(
        &w,
        "lines_i_garden",
        json!({ "view": "I", "filters": filters(json!({})), "row_key": key,
                "column_key": null, "offset": 0, "limit": 0 }),
    );
    print_pair(
        &w,
        "print_data_i_en",
        json!({ "view": "I", "filters": filters(json!({})), "language": "en", "paper": null }),
    );
}
