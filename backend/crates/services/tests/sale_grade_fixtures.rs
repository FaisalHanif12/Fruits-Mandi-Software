//! Real answers for the quality of a sale line (درجہ, PRD FR-12.42, plan 58), as JSON files for the
//! frontend: the saved بکری view, the print data of the seller bill and of the buyer slips, and the
//! rows of R2 (the sales register) and R35 (the party day slip) with the quality.
//!
//! The story is the worked example of plan 58 on the PRD worked day: رحیم's 20 پیٹی سیب, اکرم takes
//! 10 اول at 1,200 and 6 دوم at 800, ارشد takes 4 سوم at 500 (12,000 / 4,800 / 2,000, commission
//! 1,128 on each side). Each fixture is a pair: `<name>_request.json` is exactly the `input` the
//! command takes and `<name>.json` is the real answer.
//!
//! The test compares the answers with the files and FAILS if any differ, so a fixture can never go
//! stale. After a deliberate change write them again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test sale_grade_fixtures`, read the diff
//! and tell `frontend`.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use mandi_reports::print::{BikriPrintDataInput, bikri_print_data};
use mandi_reports::reports::{ReportRunInput, run};
use mandi_services::bikri;
use mandi_services::core::bikri::BikriGetInput;
use serde::Serialize;
use serde_json::{Value, json};

use reports_world::*;

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

/// رحیم's 20 apples with the qualities of the plan.
fn graded(w: &support::World) -> mandi_services::core::bikri::BikriDraft {
    let mut lines = vec![
        support::sale("s-1", w.akram, "10", "1200"),
        support::sale("s-2", w.akram, "6", "800"),
        support::sale("s-3", w.arshad, "4", "500"),
    ];
    for (line, grade) in lines.iter_mut().zip(["first", "second", "third"]) {
        line.grade = Some(grade.to_owned());
    }
    support::draft(vec![support::with_sales(
        support::bare_lot(w, "lot-1", w.rahim, w.apple, "20"),
        lines,
    )])
}

fn report_request(id: &str, filters: Value) -> Value {
    json!({ "id": id, "filters": filters, "offset": 0, "limit": 0 })
}

fn report(w: &support::World, request: &Value) -> mandi_reports::reports::ReportPage {
    let input: ReportRunInput = serde_json::from_value(request.clone()).unwrap();
    run(w.env.ctx.db(), &input).unwrap()
}

fn print_request(bikri_id: i64, doc: &str, language: &str) -> Value {
    json!({ "bikri_id": bikri_id, "doc": doc, "language": language, "only_buyer_id": null })
}

fn print_data(w: &support::World, request: &Value) -> mandi_reports::print::PrintData {
    let input: BikriPrintDataInput = serde_json::from_value(request.clone()).unwrap();
    bikri_print_data(w.env.ctx.db(), &input).unwrap()
}

#[test]
fn the_real_answers_for_the_quality_of_a_sale_line_are_the_fixtures() {
    let w = worked_day();
    let saved = save(&w, &graded(&w));

    // The saved view: each line carries `grade` (the key is absent on a line with none).
    let request = json!({ "id": saved.bikri_id });
    let input: BikriGetInput = serde_json::from_value(request.clone()).unwrap();
    let detail = bikri::get(&w.env.ctx, input.id).unwrap();
    pair("sale_grade_bikri_get", &request, &detail);

    // The seller bill in English and the buyer slips in Urdu: the line has `grade`, and its
    // `detail` text has the word after the item.
    let request = print_request(saved.bikri_id, "seller_bill", "en");
    pair(
        "sale_grade_print_seller_bill_en",
        &request,
        &print_data(&w, &request),
    );
    let request = print_request(saved.bikri_id, "buyer_slips", "ur");
    pair(
        "sale_grade_print_buyer_slips_ur",
        &request,
        &print_data(&w, &request),
    );

    // R2: the register has the quality column after the item.
    let request = report_request("R2", json!({}));
    pair("sale_grade_report_r2", &request, &report(&w, &request));

    // R35: اکرم's day slip has the quality column last.
    let request = report_request(
        "R35",
        json!({ "as_of": "10-10-2026", "party": { "account_id": w.akram, "person_key": null } }),
    );
    pair(
        "sale_grade_report_r35_akram",
        &request,
        &report(&w, &request),
    );

    // R5: the buyer bills, with the quality in the detail parts.
    let request = report_request("R5", json!({}));
    pair("sale_grade_report_r5", &request, &report(&w, &request));
}
