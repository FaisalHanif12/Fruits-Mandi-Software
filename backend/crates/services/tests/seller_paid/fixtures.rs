//! The real answers of the seller-paid commands and prints as JSON files for the frontend (plan 91),
//! and the print tests on the worked cases, in Urdu and English.
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (read back
//! into the real input type before it is used), and `<name>.json` is the real answer. The tests
//! compare them with the files on disk and FAIL if any differ. After a deliberate change write them
//! again with `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test seller_paid`, read the
//! diff, and tell frontend.
//!
//! Files (all under `backend/crates/services/tests/fixtures/`):
//! * `seller_paid_preview_case_a|b|c|d` (+ `_request`): `bikri_preview` for the four worked numbers;
//! * `seller_paid_spread_two_lots` (+ `_request`): `bikri_seller_paid_spread`, 15,000 over 12,000 + 8,000;
//! * `seller_paid_slip_r35_case_a_ur|en`, `seller_paid_slip_r35_two_lots_ur|en`: the R35 seller slip
//!   print data (`reports_print_data`);
//! * `seller_paid_seller_bill_case_a_ur|en`, `seller_paid_seller_bill_two_lots_ur|en`: the seller
//!   bill of the print menu (`reports_seller_bills_print_data`).

use std::path::{Path, PathBuf};

use mandi_reports::reports::{
    DaySlipsInput, ReportPrintInput, SlipSections, print_data, seller_bills,
};
use mandi_services::bikri;
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{BikriDraft, SellerPaidSpreadInput};
use serde::Serialize;
use serde_json::{Value, json};

use super::*;

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

fn answer_only<A: Serialize>(name: &str, answer: &A) {
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

// ---- the worked cases ----

/// Case A: previous 30,000 بنام (since 09-10-2026), one lot with credit 20,000, paid 5,000.
fn case_a(w: &World) -> BikriDraft {
    rahim_draft(w, 20, Some("5000"))
}

/// Two lots, credits 12,000 and 8,000, no نقد typed (the spread command fills it).
fn two_lots(w: &World) -> BikriDraft {
    support::draft(vec![
        credit_lot(w, "lot-1", w.rahim, "12", "1000", None),
        credit_lot(w, "lot-2", w.rahim, "8", "1000", None),
    ])
}

fn preview_fixture(w: &World, name: &str, draft: &BikriDraft) {
    let request = json!({ "draft": serde_json::to_value(draft).unwrap() });
    let preview = support::preview(w, draft);
    pair(name, &request, &preview);
}

#[test]
fn the_preview_fixtures_of_the_four_worked_numbers_are_the_real_answers() {
    // A: previous 30,000 بنام, credit 20,000, paid 5,000.
    let w = rich_world();
    owes_on(&w, "30,000", "09-10-2026");
    preview_fixture(&w, "seller_paid_preview_case_a", &case_a(&w));
    // D: the same ledger, paid 25,000: above today's credit, needs the question.
    preview_fixture(
        &w,
        "seller_paid_preview_case_d",
        &rahim_draft(&w, 20, Some("25000")),
    );
    // B: two lots, 15,000 paid, as the screen writes it after the spread (12,000 and 3,000).
    let mut b = two_lots(&w);
    b.lots[0].cash_paid = Some("12000".into());
    b.lots[1].cash_paid = Some("3000".into());
    preview_fixture(&w, "seller_paid_preview_case_b", &b);
    // C: the shop owes him 10,000 (an earlier بکری), credit 20,000, paid 28,000: no question.
    let w = rich_world();
    save_confirmed(&w, &rahim_draft(&w, 10, None), false).unwrap();
    preview_fixture(
        &w,
        "seller_paid_preview_case_c",
        &rahim_draft(&w, 20, Some("28000")),
    );
}

#[test]
fn the_spread_fixture_of_two_lots_is_the_real_answer() {
    let w = rich_world();
    owes_on(&w, "30,000", "09-10-2026");
    let draft = two_lots(&w);
    let request = json!({
        "draft": serde_json::to_value(&draft).unwrap(),
        "entry_id": null,
        "seller_paid": [{ "seller_account_id": w.rahim, "amount": "15000" }],
    });
    let input: SellerPaidSpreadInput = serde_json::from_value(request.clone()).unwrap();
    let answer = bikri::seller_paid_spread(&w.env.ctx, &input).unwrap();
    // The numbers of the plan: 12,000 and 3,000, and the same in the preview's lot totals.
    let parts: Vec<i64> = answer
        .lots
        .iter()
        .map(|l| l.cash_paid_paisa.rupees())
        .collect();
    assert_eq!(parts, vec![12_000, 3_000]);
    pair("seller_paid_spread_two_lots", &request, &answer);
}

// ---- the prints ----

/// The summary lines of a print as (label id, text, side id).
fn summary(data: &Value) -> Vec<(String, String, Option<String>)> {
    data["summary"]
        .as_array()
        .unwrap_or_else(|| panic!("no summary: {data}"))
        .iter()
        .map(|line| {
            (
                line["label_id"].as_str().unwrap().to_owned(),
                line["cell"]["text"].as_str().unwrap_or("").to_owned(),
                line["cell"]["side_id"].as_str().map(str::to_owned),
            )
        })
        .collect()
}

fn line_of(lines: &[(String, String, Option<String>)], label: &str) -> (String, Option<String>) {
    let found = lines
        .iter()
        .find(|(l, _, _)| l == label)
        .unwrap_or_else(|| panic!("no line {label}: {lines:?}"));
    (found.1.clone(), found.2.clone())
}

fn r35_print(w: &World, language: &str) -> (Value, Value) {
    let request = json!({
        "id": "R35",
        "filters": {
            "shortcut": null, "from": "", "to": "", "as_of": "10-10-2026",
            "party": { "account_id": w.rahim, "person_key": null },
            "seller_id": null, "zamindar_id": null, "item_id": null, "city_id": null,
            "adda_id": null, "detail": false, "partner_id": null, "garden_id": null,
            "bikri_id": null
        },
        "language": language, "paper": "a4", "sort": null, "group_by_month": false
    });
    let input: ReportPrintInput = serde_json::from_value(request.clone()).unwrap();
    let data = print_data(w.env.ctx.db(), &input).unwrap();
    (request, serde_json::to_value(&data).unwrap())
}

fn bill_print(w: &World, language: Language) -> (Value, Value) {
    let request = json!({
        "date": "10-10-2026", "bikri_id": null, "bikri_ids": null, "buyer_ids": null,
        "sections": "sellers",
        "language": if language == Language::Ur { "ur" } else { "en" },
        "paper": "a4",
    });
    let input: DaySlipsInput = serde_json::from_value(request.clone()).unwrap();
    assert_eq!(input.sections, SlipSections::Sellers);
    let data = seller_bills(w.env.ctx.db(), &input).unwrap();
    assert_eq!(data.sellers.len(), 1, "one seller, one bill");
    (request, serde_json::to_value(&data.sellers[0]).unwrap())
}

/// The five lines of Umar on a seller's slip, as (previous, net sale, paid, rest, new) with sides.
fn five_lines(data: &Value) -> [(String, Option<String>); 5] {
    let lines = summary(data);
    [
        line_of(&lines, "report-r35-previous"),
        line_of(&lines, "report-r35-net-sale"),
        line_of(&lines, "report-r35-paid-today"),
        line_of(&lines, "report-r35-sale-rest"),
        line_of(&lines, "report-r35-new-balance"),
    ]
}

fn side(id: &str) -> Option<String> {
    Some(format!("report-side-{id}"))
}

fn check_slip_case_a(data: &Value) {
    let [previous, net, paid, rest, new] = five_lines(data);
    // پچھلا بقایا 30,000 بنام; آج کی صافی بکری 20,000 جمع; نقد ادا کر دیا 5,000;
    // بکری کی باقی رقم 15,000; نیا بقایا 15,000 بنام.
    assert_eq!(previous, ("30,000".to_owned(), side("dr")));
    assert_eq!(net, ("20,000".to_owned(), side("cr")));
    assert_eq!(paid.0, "5,000");
    assert_eq!(rest.0, "15,000");
    assert_eq!(new, ("15,000".to_owned(), side("dr")));
    // The new line sits between paid today and today's left.
    let labels: Vec<String> = summary(data).into_iter().map(|(l, _, _)| l).collect();
    let at = |label: &str| labels.iter().position(|l| l == label).unwrap();
    assert_eq!(at("report-r35-sale-rest"), at("report-r35-paid-today") + 1);
    assert_eq!(at("report-r35-today-left"), at("report-r35-sale-rest") + 1);
}

fn check_slip_two_lots(data: &Value) {
    let [previous, net, paid, rest, new] = five_lines(data);
    // 15,000 spread over 12,000 + 8,000 is one paid amount of 15,000 on the slip: the rest of the
    // sale is 5,000 and 30,000 + 15,000 - 20,000 = 25,000 بنام.
    assert_eq!(previous, ("30,000".to_owned(), side("dr")));
    assert_eq!(net, ("20,000".to_owned(), side("cr")));
    assert_eq!(paid.0, "15,000");
    assert_eq!(rest.0, "5,000");
    assert_eq!(new, ("25,000".to_owned(), side("dr")));
}

fn save_case_a(w: &World) {
    owes_on(w, "30,000", "09-10-2026");
    save_confirmed(w, &case_a(w), false).unwrap();
}

fn save_two_lots_spread(w: &World) {
    owes_on(w, "30,000", "09-10-2026");
    let draft = two_lots(w);
    let out = spread(w, &draft, None, &[(w.rahim, "15000")]);
    let mut typed = draft;
    for (lot, part) in typed.lots.iter_mut().zip(&out.lots) {
        lot.cash_paid = Some(part.cash_paid_paisa.rupees().to_string());
    }
    save_confirmed(w, &typed, false).unwrap();
}

#[test]
fn the_r35_slip_and_the_seller_bill_show_the_five_lines_on_the_worked_case_in_both_languages() {
    let w = rich_world();
    save_case_a(&w);
    for language in ["ur", "en"] {
        let (request, data) = r35_print(&w, language);
        check_slip_case_a(&data);
        pair(
            &format!("seller_paid_slip_r35_case_a_{language}"),
            &request,
            &data,
        );
        let lang = if language == "ur" {
            Language::Ur
        } else {
            Language::En
        };
        let (request, data) = bill_print(&w, lang);
        check_slip_case_a(&data);
        pair(
            &format!("seller_paid_seller_bill_case_a_{language}"),
            &request,
            &data,
        );
    }
}

#[test]
fn the_r35_slip_and_the_seller_bill_of_two_lots_show_one_paid_amount_spread_over_them() {
    let w = rich_world();
    save_two_lots_spread(&w);
    for language in ["ur", "en"] {
        let (request, data) = r35_print(&w, language);
        check_slip_two_lots(&data);
        pair(
            &format!("seller_paid_slip_r35_two_lots_{language}"),
            &request,
            &data,
        );
        let lang = if language == "ur" {
            Language::Ur
        } else {
            Language::En
        };
        let (request, data) = bill_print(&w, lang);
        check_slip_two_lots(&data);
        pair(
            &format!("seller_paid_seller_bill_two_lots_{language}"),
            &request,
            &data,
        );
    }
    // Each lot kept its own نقد in the books: 12,000 and 3,000.
    let conn = raw(&w.dir);
    let stored: Vec<i64> = conn
        .prepare("SELECT cash_paid_paisa FROM bikri_lots ORDER BY lot_no")
        .unwrap()
        .query_map([], |r| r.get::<_, i64>(0))
        .unwrap()
        .map(|r| r.unwrap() / 100)
        .collect();
    assert_eq!(stored, vec![12_000, 3_000]);
}

#[test]
fn the_rest_of_the_sale_is_never_below_zero_and_the_line_is_always_there() {
    // He owed 30,000, today's credit is 10,000 and he takes 15,000: nothing of the sale is left for
    // his account (rest 0) and he owes 35,000. The line is still printed, with 0.
    let w = rich_world();
    owes_on(&w, "30,000", "09-10-2026");
    save_confirmed(&w, &rahim_draft(&w, 10, Some("15000")), true).unwrap();
    let (_, data) = r35_print(&w, "en");
    let [previous, _net, paid, rest, new] = five_lines(&data);
    assert_eq!(previous, ("30,000".to_owned(), side("dr")));
    assert_eq!(paid.0, "15,000");
    assert_eq!(rest.0, "0");
    assert_eq!(new, ("35,000".to_owned(), side("dr")));
    // A seller who was not paid anything: the whole credit is the rest.
    let w = rich_world();
    save_confirmed(&w, &rahim_draft(&w, 20, None), false).unwrap();
    let (_, data) = r35_print(&w, "en");
    let [previous, net, _paid, rest, new] = five_lines(&data);
    assert_eq!(previous.1, side("zero"));
    assert_eq!(net, ("20,000".to_owned(), side("cr")));
    assert_eq!(rest.0, "20,000");
    assert_eq!(new, ("20,000".to_owned(), side("cr")));
}

#[test]
fn the_buyers_slip_has_no_such_line() {
    let w = rich_world();
    save_case_a(&w);
    let request = json!({
        "id": "R35",
        "filters": {
            "shortcut": null, "from": "", "to": "", "as_of": "10-10-2026",
            "party": { "account_id": w.akram, "person_key": null },
            "seller_id": null, "zamindar_id": null, "item_id": null, "city_id": null,
            "adda_id": null, "detail": false, "partner_id": null, "garden_id": null,
            "bikri_id": null
        },
        "language": "en", "paper": "a4", "sort": null, "group_by_month": false
    });
    let input: ReportPrintInput = serde_json::from_value(request).unwrap();
    let data = serde_json::to_value(print_data(w.env.ctx.db(), &input).unwrap()).unwrap();
    assert!(
        !summary(&data)
            .iter()
            .any(|(l, _, _)| l == "report-r35-sale-rest"),
        "only a seller's slip has it"
    );
}

#[test]
fn the_refusal_names_every_reason_cash_first_then_each_seller() {
    // Cash in hand 10,000; رحیم is paid 25,000 on a 20,000 credit while he owes 30,000: both the cash
    // question and the seller question apply, and one refusal lists both.
    let w = support::world_with(10_000, true);
    owes_on(&w, "30,000", "09-10-2026");
    let draft = rahim_draft(&w, 20, Some("25000"));
    let preview = support::preview(&w, &draft);
    assert!(preview.cash.needs_confirm);
    assert!(preview.sellers[0].needs_confirm);
    let refused = save_confirmed(&w, &draft, false).unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("expected a rejection");
    };
    let codes: Vec<(ErrorCode, Option<&str>)> =
        items.iter().map(|i| (i.code, i.field.as_deref())).collect();
    assert_eq!(
        codes,
        vec![
            (ErrorCode::ErrCashBelowZeroNotConfirmed, None),
            (ErrorCode::ErrSellerPaidAboveToday, Some("sellers[0].paid")),
        ]
    );
    // One yes answers both.
    save_confirmed(&w, &draft, true).unwrap();
}

#[test]
fn a_refusal_fixture_for_case_d() {
    let w = rich_world();
    owes_on(&w, "30,000", "09-10-2026");
    let refused = save_confirmed(&w, &rahim_draft(&w, 20, Some("25000")), false).unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("expected a rejection");
    };
    let shown: Vec<Value> = items
        .iter()
        .map(|i| {
            json!({
                "code": i.code,
                "field": i.field,
                "values": i.values,
            })
        })
        .collect();
    answer_only("seller_paid_save_refused_case_d", &shown);
}
