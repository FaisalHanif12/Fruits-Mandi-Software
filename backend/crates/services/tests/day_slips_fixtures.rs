//! Plan 42: the real answers of the day slips (R35 for one buyer and one seller) and the day total
//! sheet (R34), on the PRD worked day after Events 1 to 5 (10-10-26), as JSON files for the
//! frontend: the screen page (`reports_run`) and the print data in both languages
//! (`reports_print_data`). They replace the hand-built contract files of T1 under the same names.
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used), and `<name>.json` is the answer the real
//! engine gives for it. The frontend copies the files to `frontend/tests/fixtures/`.
//!
//! The test compares the answers with the files on disk and FAILS if any differ. After a
//! deliberate change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test day_slips_fixtures`, look at the
//! diff, and tell `frontend`.
//!
//! The numbers are the PRD's (E2E-36 and E2E-43): اکرم 12,000 + 720 + 160 = 12,880, paid 10,000,
//! left 2,880; رحیم 29,400 less 1,764, 3,000, 250, 50 and 20 = 24,316, paid 9,316, left 15,000
//! جمع, previous 15,000 بنام, new 0; the buyers' table 44,364 / 22,800 / 21,564; the sellers'
//! table 34,476 / 9,316.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use mandi_reports::reports::{ReportPrintInput, ReportRunInput, print_data, run};
use reports_world::{full_day, save, worked_day};
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

/// The report filters with every key, as the frontend sends them.
fn filters(extra: Value) -> Value {
    let mut base = json!({
        "shortcut": null, "from": "", "to": "", "as_of": "", "party": null,
        "seller_id": null, "zamindar_id": null, "item_id": null, "city_id": null,
        "adda_id": null, "detail": false, "partner_id": null, "garden_id": null,
        "bikri_id": null
    });
    if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
        for (key, value) in extra {
            base.insert(key.clone(), value.clone());
        }
    }
    base
}

/// One report: the page for the screen, and the print data in both languages.
fn report_files(w: &support::World, name: &str, id: &str, filters: &Value) {
    let run_request = json!({ "id": id, "filters": filters, "offset": 0, "limit": 0 });
    let input: ReportRunInput = serde_json::from_value(run_request.clone()).unwrap();
    let page = run(w.env.ctx.db(), &input).unwrap();
    pair(name, &run_request, &page);
    for (suffix, language) in [("ur", "ur"), ("en", "en")] {
        let print_request = json!({
            "id": id, "filters": filters, "language": language, "paper": "a4",
            "sort": null, "group_by_month": false
        });
        let input: ReportPrintInput = serde_json::from_value(print_request.clone()).unwrap();
        let data = print_data(w.env.ctx.db(), &input).unwrap();
        pair(&format!("{name}_print_{suffix}"), &print_request, &data);
    }
}

#[test]
fn the_day_slips_and_the_day_total_sheet_fixtures_are_the_real_answers() {
    let w = full_day();
    report_files(
        &w,
        "report_r35_slip_buyer_akram",
        "R35",
        &filters(json!({
            "as_of": "10-10-2026",
            "party": { "account_id": w.akram, "person_key": null }
        })),
    );
    report_files(
        &w,
        "report_r35_slip_seller_rahim",
        "R35",
        &filters(json!({
            "as_of": "10-10-2026",
            "party": { "account_id": w.rahim, "person_key": null }
        })),
    );
    report_files(
        &w,
        "report_r34_day_sheet",
        "R34",
        &filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })),
    );
}

#[test]
fn the_slip_of_a_weighed_line_fixture_is_the_real_answer() {
    // Plan 42, PRD A73: 2 نگ of grapes, 12.345 kg at 100 a kilo, sold to اکرم: the line is worth
    // 1,235 (C1), commission 74, لاگا 16, اکرم pays 1,325. The goods row has the weight cell last
    // (12.345 kg); the rate is for one kilo.
    let w = worked_day();
    let mut lot = support::bare_lot(&w, "lot-g", w.rahim, w.weight_fruit, "2");
    lot.weight_received = Some("12.345".into());
    let mut sale = support::sale("s-1", w.akram, "2", "100");
    sale.weight = Some("12.345".into());
    sale.payment_type = Some(mandi_services::core::bikri::PaymentType::Credit);
    lot.sales = vec![sale];
    save(&w, &support::draft(vec![lot]));
    report_files(
        &w,
        "report_r35_slip_buyer_weighed",
        "R35",
        &filters(json!({
            "as_of": "10-10-2026",
            "party": { "account_id": w.akram, "person_key": null }
        })),
    );
}
