//! Real answers of the plan 17 T2b commands, as JSON files for the frontend (plan 17, T4): partner
//! capital, the partner overview with the partner total, the partner statement, الگ کھاتہ and جنرل.
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used; a command with no input has `null`), and
//! `<name>.json` is the answer the real service gives for it. A refusal is a pair too: its answer
//! has the shape of the error a command returns (`ok`, `errors`, `ref`), where `ref` is a
//! stand-in because the real reference number is a counter. The frontend copies the files to
//! `frontend/tests/fixtures/`.
//!
//! The test compares the answers with the files on disk and FAILS if any differ, so a fixture can
//! never go stale. After a deliberate change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test day_money_fixtures`, look at the
//! diff, and tell `frontend`.
//!
//! The story is the PRD worked day: بکری 1 of Event 1, the electricity bill and الف's 2,000 of
//! Event 5, ب's 50,000 of E2E-35, the family khata of E2E-38 (paid out and received) and two
//! جنرل entries. The names of the files are those of the commands.
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test partner_money_fixtures`.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::*;
use mandi_services::core::bikri::BikriSaveInput;
use mandi_services::core::capital::CapitalHistoryInput;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::general::{GeneralInput, GeneralPreviewInput};
use mandi_services::core::khata::{KhataCreateInput, KhataStatementInput};
use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
use mandi_services::core::statement::PartnersStatementInput;
use mandi_services::core::vouchers::VoucherPreviewInput;
use mandi_services::{ServiceError, bikri, khata, masters, partners, vouchers};
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

fn refusal(name: &str, request: &Value, error: &ServiceError, code: ErrorCode) {
    assert_eq!(error.code(), code, "{name}");
    pair(name, request, &wire(error));
}

/// The request read back into the real input type, so the file is exactly what the command takes.
fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

/// Repeats a save that asks the user to confirm rule-made spellings (PRD FR-7.66), confirming
/// what it asks, and gives back the answer and the words that were confirmed.
fn confirming<T>(
    mut attempt: impl FnMut(&[String]) -> Result<T, ServiceError>,
) -> (T, Vec<String>) {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        match attempt(&words) {
            Ok(done) => return (done, words),
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

fn head(w: &World, ur: &str, en: &str) -> i64 {
    confirming(|words| {
        masters::save(
            &w.env.ctx,
            &w.owner,
            &MasterInput::ExpenseHead(ExpenseHeadInput {
                id: None,
                version: None,
                code: String::new(),
                name_ur: ur.into(),
                name_en: en.into(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    });
    masters::expense_heads(&w.env.ctx)
        .unwrap()
        .rows
        .iter()
        .find(|r| r.head.name_en == en)
        .unwrap()
        .id
}

/// The worked-day world with the shares of 40 / 30 / 30 started on 31-08-2026, cash 50,000, and
/// today 10-10-2026.
fn world() -> World {
    let w = support::world_with(50_000, false);
    set_today(&w.env, 2026, 8, 31);
    partners::set_shares(
        &w.env.ctx,
        &w.owner,
        &PartnersSetSharesInput {
            effective_from: "31-08-2026".into(),
            shares: [
                (w.partners[0], "40"),
                (w.partners[1], "30"),
                (w.partners[2], "30"),
            ]
            .iter()
            .map(|&(partner_id, share)| ShareInput {
                partner_id,
                share: share.to_owned(),
            })
            .collect(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 10);
    w
}

#[test]
fn the_partner_money_fixtures_are_the_real_answers() {
    let w = world();
    let (a, b) = (w.partners[0], w.partners[1]);
    let electricity = head(&w, "بجلی", "Electricity");

    // Event 1, the electricity bill and الف's 2,000 (PRD 4.4, 4.8).
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::event1(&w),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let shop = json!({
        "date": "10-10-2026", "expense_head_id": electricity, "amount": "3,000",
        "paid_to": null, "narration": null, "confirm_cash_below_zero": false
    });
    vouchers::save_shop_expense(&w.env.ctx, &w.owner, &typed(&shop)).unwrap();
    let personal = json!({
        "date": "10-10-2026", "partner_id": a, "amount": "2,000", "reason": "گھر کا خرچ",
        "return_by": null, "taken_by": null, "narration": null, "confirm_cash_below_zero": false
    });
    vouchers::save_personal_expense(&w.env.ctx, &w.owner, &typed(&personal)).unwrap();

    // E2E-35: ب invests 50,000 on 11-10-26.
    set_today(&w.env, 2026, 10, 11);
    let invest = json!({
        "date": "11-10-2026", "partner_id": b, "amount": "50,000", "narration": "start",
        "confirm_cash_below_zero": false
    });
    pair(
        "vouchers_save_capital_in",
        &invest,
        &vouchers::save_capital_in(&w.env.ctx, &w.owner, &typed(&invest)).unwrap(),
    );
    // Taking back more than his capital: the preview warns (FR-17.8), and the cash question.
    let back = json!({
        "entry": { "type": "capital_out", "date": "11-10-2026", "partner_id": b,
                   "amount": "60,000", "confirm_cash_below_zero": false },
        "replacing_voucher_id": null
    });
    pair(
        "vouchers_preview_capital_out",
        &back,
        &vouchers::preview(&w.env.ctx, &typed::<VoucherPreviewInput>(&back)).unwrap(),
    );
    let too_much = json!({
        "date": "11-10-2026", "partner_id": b, "amount": "120,000", "narration": null,
        "confirm_cash_below_zero": false
    });
    refusal(
        "vouchers_save_capital_out_cash_confirm",
        &too_much,
        &vouchers::save_capital_out(&w.env.ctx, &w.owner, &typed(&too_much)).unwrap_err(),
        ErrorCode::ErrCashBelowZeroNotConfirmed,
    );
    let history = json!({ "partner_id": b, "offset": 0, "limit": 0 });
    pair(
        "capital_history",
        &history,
        &partners::capital_history(&w.env.ctx, &typed::<CapitalHistoryInput>(&history)).unwrap(),
    );

    // The overview with returned, خالص, capital, ventures and کل رقم; B's statement; A's too.
    pair(
        "partners_overview",
        &Value::Null,
        &partners::overview(&w.env.ctx).unwrap(),
    );
    let statement = json!({ "partner_id": b, "from": null, "to": null, "offset": 0, "limit": 0 });
    pair(
        "partners_statement",
        &statement,
        &partners::statement(&w.env.ctx, &typed::<PartnersStatementInput>(&statement)).unwrap(),
    );
    let statement_a =
        json!({ "partner_id": a, "from": "10-10-2026", "to": null, "offset": 0, "limit": 2 });
    pair(
        "partners_statement_page",
        &statement_a,
        &partners::statement(&w.env.ctx, &typed::<PartnersStatementInput>(&statement_a)).unwrap(),
    );

    // E2E-38: the family khata.
    let (made, words) = confirming(|words| {
        khata::create(
            &w.env.ctx,
            &w.owner,
            &KhataCreateInput {
                name_ur: "خاندانی کھاتہ".into(),
                name_en: "Family Khata".into(),
                note: Some("family".into()),
                confirmed_rules_words: words.to_vec(),
            },
        )
    });
    let create = json!({
        "name_ur": "خاندانی کھاتہ", "name_en": "Family Khata", "note": "family",
        "confirmed_rules_words": words
    });
    let _ = typed::<KhataCreateInput>(&create);
    pair("khata_create", &create, &made);
    let family = made.account.id;
    let bad = json!({
        "date": "10-10-2026", "khata_account_id": null, "amount": "10.5", "reason": "",
        "narration": null, "confirm_cash_below_zero": false
    });
    refusal(
        "vouchers_save_khata_out_mistakes",
        &bad,
        &vouchers::save_khata_out(&w.env.ctx, &w.owner, &typed(&bad)).unwrap_err(),
        ErrorCode::ErrDecimalsNotAllowed,
    );
    let out = json!({
        "date": "10-10-2026", "khata_account_id": family, "amount": "10,000",
        "reason": "خاندان کا خرچ", "narration": null, "confirm_cash_below_zero": false
    });
    pair(
        "vouchers_save_khata_out",
        &out,
        &vouchers::save_khata_out(&w.env.ctx, &w.owner, &typed(&out)).unwrap(),
    );
    let received = json!({
        "date": "11-10-2026", "khata_account_id": family, "amount": "4,000",
        "reason": "واپس آئے", "narration": null, "confirm_cash_below_zero": false
    });
    pair(
        "vouchers_save_khata_in",
        &received,
        &vouchers::save_khata_in(&w.env.ctx, &w.owner, &typed(&received)).unwrap(),
    );
    let khata_page = json!({
        "khata_account_id": family, "from": null, "to": null, "offset": 0, "limit": 0
    });
    pair(
        "khata_statement",
        &khata_page,
        &khata::statement(&w.env.ctx, &typed::<KhataStatementInput>(&khata_page)).unwrap(),
    );

    // جنرل: a correction between a buyer and a seller; one that does not balance.
    let cash_id = support::id_of(&w.env, "01010001");
    let ok_lines = json!([
        { "account_id": w.akram, "person_key": null, "dr": "1,000", "cr": "", "narration": "fix" },
        { "account_id": w.rahim, "person_key": null, "dr": "", "cr": "1,000", "narration": null }
    ]);
    let preview_ok = json!({ "date": "11-10-2026", "lines": ok_lines });
    pair(
        "general_preview_balanced",
        &preview_ok,
        &vouchers::general_preview(&w.env.ctx, &typed::<GeneralPreviewInput>(&preview_ok)).unwrap(),
    );
    let preview_bad = json!({ "date": null, "lines": [
        { "account_id": w.akram, "person_key": null, "dr": "1,000", "cr": "", "narration": null },
        { "account_id": cash_id, "person_key": null, "dr": "", "cr": "900", "narration": null }
    ] });
    pair(
        "general_preview_unbalanced",
        &preview_bad,
        &vouchers::general_preview(&w.env.ctx, &typed::<GeneralPreviewInput>(&preview_bad))
            .unwrap(),
    );
    let save_bad = json!({
        "date": "11-10-2026", "narration": null, "confirm_cash_below_zero": false,
        "lines": preview_bad["lines"]
    });
    refusal(
        "general_save_not_balanced",
        &save_bad,
        &vouchers::general_save(&w.env.ctx, &w.owner, &typed::<GeneralInput>(&save_bad))
            .unwrap_err(),
        ErrorCode::ErrGeneralNotBalanced,
    );
    let save_ok = json!({
        "date": "11-10-2026", "narration": "correction", "confirm_cash_below_zero": false,
        "lines": ok_lines
    });
    pair(
        "general_save",
        &save_ok,
        &vouchers::general_save(&w.env.ctx, &w.owner, &typed::<GeneralInput>(&save_ok)).unwrap(),
    );
}
