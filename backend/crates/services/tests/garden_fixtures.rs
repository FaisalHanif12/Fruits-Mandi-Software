//! Real answers of the garden commands and of the garden lot in بکری, as JSON files for the
//! frontend (plan 17c, T2).
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used), and `<name>.json` is the answer the real
//! service gives for it. A refusal is a pair too: its answer has the shape of the error a command
//! returns (`ok`, `errors`, `ref`), where `ref` is a stand-in because the real reference number is a
//! counter. The frontend copies the files to `frontend/tests/fixtures/`.
//!
//! The test compares the answers with the files on disk and FAILS if any differ, so a fixture can
//! never go stale. After a deliberate change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test garden_fixtures`, look at the diff,
//! and tell `frontend`.
//!
//! Story A is E2E-40 (الف 60% from the shop, ب 40% from his pocket, ج not in it, rent 50,000, a sale
//! in our shop, a sale in another shop, the closing, its undo). Story B is E2E-40b (no sale in our
//! shop, a loss of 45,000). Story C is a garden with no entry yet, whose partners are changed.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::set_today;
use mandi_services::core::bikri::{BikriDraft, BikriSaveInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{
    GardenCloseInput, GardenEntryCancelInput, GardenEntryInput, GardenGetInput, GardenInput,
    GardenReopenInput, GardenSetPartnersInput, GardensListInput,
};
use mandi_services::core::statement::PartnersStatementInput;
use mandi_services::{ServiceError, bikri, gardens, partners};
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

fn partner_json(id: i64, share: &str, source: &str) -> Value {
    json!({ "partner_id": id, "share": share, "source": source })
}

/// The garden of E2E-40: الف 60% from the shop, ب 40% from his pocket.
fn garden_json(w: &World, paid_now: Option<&str>, partners: &[Value]) -> Value {
    json!({
        "name_ur": "شاہ پور باغ",
        "name_en": "Shahpur Garden",
        "confirmed_rules_words": ["شاہ", "پور", "باغ", "Shahpur", "Garden"],
        "date": "10-10-2026",
        "item_id": w.banana,
        "rent": "50,000",
        "season_end": null,
        "note": null,
        "partners": partners,
        "paid_now": paid_now,
        "paid_now_reason": paid_now.map(|_| "first installment"),
        "parts": [],
        "confirm_cash_below_zero": false
    })
}

fn payment_json(kind: &str, garden: i64, date: &str, amount: &str, reason: &str) -> Value {
    json!({
        "type": kind, "garden_id": garden, "date": date, "amount": amount, "reason": reason,
        "parts": [], "confirm_cash_below_zero": false
    })
}

fn other_sale_json(garden: i64, date: &str, qty: &str, amount: &str) -> Value {
    json!({
        "type": "other_sale", "garden_id": garden, "date": date, "shop": "Haji Traders",
        "qty": qty, "amount": amount, "note": null
    })
}

fn get_json(garden: i64) -> Value {
    json!({ "id": garden, "history_offset": 0, "history_limit": 0 })
}

fn close_json(w: &World, garden: i64, date: &str) -> Value {
    let version = gardens::get(&w.env.ctx, &typed::<GardenGetInput>(&get_json(garden)))
        .unwrap()
        .row
        .version;
    json!({ "id": garden, "version": version, "date": date })
}

/// A lot of بکری whose seller is the garden: 100 پیٹی کیلا sold to ارشد at 600 on credit.
fn garden_lot_draft(w: &World, garden_account: i64) -> BikriDraft {
    let mut lot = support::with_sales(
        support::bare_lot(w, "garden-1", garden_account, w.banana, "100"),
        vec![support::sale("s-1", w.arshad, "100", "600")],
    );
    lot.adjust_advance = false;
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = "12-10-2026".into();
    draft
}

fn draft_json(draft: &BikriDraft) -> Value {
    serde_json::to_value(draft).unwrap()
}

/// The garden of E2E-40 with its three payments of 10-10 and 11-10 (30,000, 20,000 and the labour
/// 5,000), the same as `pay_the_garden` of the golden test.
fn story_a_start(w: &World) -> (i64, i64) {
    let [a, b, _] = w.partners;
    let ctx = &w.env.ctx;
    let owner = &w.owner;

    // The first garden starts from the firm's shares; ج is left out by the screen.
    pair(
        "gardens_defaults_first",
        &json!({}),
        &gardens::defaults(ctx).unwrap(),
    );
    // A garden whose shares do not total 100% is refused at its partners (FR-12.29).
    let bad = garden_json(
        w,
        None,
        &[
            partner_json(a, "60", "shop"),
            partner_json(b, "30", "pocket"),
        ],
    );
    let error = gardens::create(ctx, owner, &typed::<GardenInput>(&bad)).unwrap_err();
    refusal(
        "gardens_create_shares_not_100",
        &bad,
        &error,
        ErrorCode::ErrSharesNot100,
    );
    let e2e40 = garden_json(
        w,
        Some("30,000"),
        &[
            partner_json(a, "60", "shop"),
            partner_json(b, "40", "pocket"),
        ],
    );
    pair(
        "gardens_create_preview_first_installment",
        &e2e40,
        &gardens::create_preview(ctx, &typed::<GardenInput>(&e2e40)).unwrap(),
    );
    let created = gardens::create(ctx, owner, &typed::<GardenInput>(&e2e40)).unwrap();
    pair("gardens_create_e2e40", &e2e40, &created);
    // The next garden starts from the last garden's shares and sources.
    pair(
        "gardens_defaults_from_last_garden",
        &json!({}),
        &gardens::defaults(ctx).unwrap(),
    );
    (created.row.id, created.row.account.id)
}

#[test]
fn garden_fixtures_of_e2e_40() {
    let w = support::world();
    let ctx = &w.env.ctx;
    let owner = &w.owner;
    let (garden, account) = story_a_start(&w);

    // 11-10: an installment of 20,000 and a labour cost of 5,000. A payment too big for the cash
    // asks first (FR-6.7).
    set_today(&w.env, 2026, 10, 11);
    let installment = payment_json(
        "installment",
        garden,
        "11-10-2026",
        "20,000",
        "second installment",
    );
    pair(
        "gardens_entry_preview_installment",
        &installment,
        &gardens::entry_preview(ctx, &typed::<GardenEntryInput>(&installment)).unwrap(),
    );
    pair(
        "gardens_entry_save_installment",
        &installment,
        &gardens::entry_save(ctx, owner, &typed::<GardenEntryInput>(&installment)).unwrap(),
    );
    let labour = payment_json("cost", garden, "11-10-2026", "5,000", "labour");
    let labour_saved =
        gardens::entry_save(ctx, owner, &typed::<GardenEntryInput>(&labour)).unwrap();
    pair("gardens_entry_save_cost", &labour, &labour_saved);
    let too_big = payment_json("installment", garden, "11-10-2026", "90,000", "too much");
    pair(
        "gardens_entry_preview_cash_below_zero",
        &too_big,
        &gardens::entry_preview(ctx, &typed::<GardenEntryInput>(&too_big)).unwrap(),
    );
    let error = gardens::entry_save(ctx, owner, &typed::<GardenEntryInput>(&too_big)).unwrap_err();
    refusal(
        "gardens_entry_save_cash_below_zero",
        &too_big,
        &error,
        ErrorCode::ErrCashBelowZeroNotConfirmed,
    );
    // An entry before the garden's day is refused at its date.
    let early = payment_json("cost", garden, "09-10-2026", "100", "too early");
    let error = gardens::entry_save(ctx, owner, &typed::<GardenEntryInput>(&early)).unwrap_err();
    refusal(
        "gardens_entry_save_date_before_garden",
        &early,
        &error,
        ErrorCode::ErrDateBeforeGarden,
    );
    // The shares are fixed after the first entry.
    let locked = json!({
        "id": garden,
        "version": gardens::get(ctx, &typed::<GardenGetInput>(&get_json(garden))).unwrap().row.version,
        "partners": [partner_json(w.partners[0], "50", "shop"), partner_json(w.partners[1], "50", "shop")]
    });
    let error =
        gardens::set_partners(ctx, owner, &typed::<GardenSetPartnersInput>(&locked)).unwrap_err();
    refusal(
        "gardens_set_partners_locked",
        &locked,
        &error,
        ErrorCode::ErrVentureSharesLocked,
    );

    // 12-10: the sale in our shop (the garden is the lot's seller) and the sale in another shop.
    set_today(&w.env, 2026, 10, 12);
    let draft = garden_lot_draft(&w, account);
    let preview_request = json!({ "draft": draft_json(&draft) });
    pair(
        "bikri_preview_garden_lot_e2e40",
        &preview_request,
        &bikri::preview(ctx, &draft).unwrap(),
    );
    let save_request = json!({ "draft": draft_json(&draft), "confirm_cash_below_zero": false });
    let saved = bikri::save(ctx, owner, &typed::<BikriSaveInput>(&save_request)).unwrap();
    pair("bikri_save_garden_lot_e2e40", &save_request, &saved);
    let other = other_sale_json(garden, "12-10-2026", "20", "10,000");
    pair(
        "gardens_entry_preview_other_sale",
        &other,
        &gardens::entry_preview(ctx, &typed::<GardenEntryInput>(&other)).unwrap(),
    );
    pair(
        "gardens_entry_save_other_sale",
        &other,
        &gardens::entry_save(ctx, owner, &typed::<GardenEntryInput>(&other)).unwrap(),
    );

    // The screens that read it: the history with its running balance, the list and the tile.
    pair(
        "gardens_get_open",
        &get_json(garden),
        &gardens::get(ctx, &typed::<GardenGetInput>(&get_json(garden))).unwrap(),
    );
    pair(
        "gardens_list_all",
        &json!({}),
        &gardens::list(ctx, &GardensListInput::default()).unwrap(),
    );
    pair(
        "gardens_list_open_paged",
        &json!({ "status": "open", "offset": 0, "limit": 1 }),
        &gardens::list(
            ctx,
            &typed::<GardensListInput>(&json!({ "status": "open", "offset": 0, "limit": 1 })),
        )
        .unwrap(),
    );
    pair(
        "gardens_summary_open",
        &json!({}),
        &gardens::summary(ctx).unwrap(),
    );
    pair(
        "partners_overview_with_garden_cards",
        &json!({}),
        &partners::overview(ctx).unwrap(),
    );

    // 13-10: the closing. Before the day of the last entry it is refused.
    set_today(&w.env, 2026, 10, 13);
    let before_last = close_json(&w, garden, "11-10-2026");
    pair(
        "gardens_close_preview_before_last_entry",
        &before_last,
        &gardens::close_preview(ctx, &typed::<GardenCloseInput>(&before_last)).unwrap(),
    );
    let error = gardens::close(ctx, owner, &typed::<GardenCloseInput>(&before_last)).unwrap_err();
    refusal(
        "gardens_close_before_last_entry",
        &before_last,
        &error,
        ErrorCode::ErrCloseBeforeLastEntry,
    );
    let closing = close_json(&w, garden, "13-10-2026");
    pair(
        "gardens_close_preview_e2e40",
        &closing,
        &gardens::close_preview(ctx, &typed::<GardenCloseInput>(&closing)).unwrap(),
    );
    pair(
        "gardens_close_e2e40",
        &closing,
        &gardens::close(ctx, owner, &typed::<GardenCloseInput>(&closing)).unwrap(),
    );
    pair(
        "gardens_get_closed",
        &get_json(garden),
        &gardens::get(ctx, &typed::<GardenGetInput>(&get_json(garden))).unwrap(),
    );
    pair(
        "gardens_summary_after_close",
        &json!({}),
        &gardens::summary(ctx).unwrap(),
    );
    // A closed garden takes no entry, no second closing and no cancel of its entries.
    let error = gardens::entry_save(ctx, owner, &typed::<GardenEntryInput>(&labour)).unwrap_err();
    refusal(
        "gardens_entry_save_garden_closed",
        &labour,
        &error,
        ErrorCode::ErrGardenClosed,
    );
    let cancel_closed = json!({
        "id": labour_saved.id, "version": labour_saved.version, "reason": "wrong",
        "confirm_cash_below_zero": false
    });
    let error = gardens::entry_cancel(ctx, owner, &typed::<GardenEntryCancelInput>(&cancel_closed))
        .unwrap_err();
    refusal(
        "gardens_entry_cancel_garden_closed",
        &cancel_closed,
        &error,
        ErrorCode::ErrGardenClosed,
    );
    // The partner statement and the cards after the closing: الف's lines are -18,000, -12,000,
    // -3,000, +3,200 and +39,840 (E2E-40).
    let statement =
        json!({ "partner_id": w.partners[0], "from": null, "to": null, "offset": 0, "limit": 0 });
    pair(
        "partners_statement_garden_alif",
        &statement,
        &partners::statement(ctx, &typed::<PartnersStatementInput>(&statement)).unwrap(),
    );
    pair(
        "partners_overview_after_garden_close",
        &json!({}),
        &partners::overview(ctx).unwrap(),
    );

    // The undo of the closing, the cancel of the labour, and the closing again.
    let reopen = json!({
        "id": garden,
        "version": gardens::get(ctx, &typed::<GardenGetInput>(&get_json(garden))).unwrap().row.version,
        "reason": "wrong day"
    });
    pair(
        "gardens_reopen_e2e40",
        &reopen,
        &gardens::reopen(ctx, owner, &typed::<GardenReopenInput>(&reopen)).unwrap(),
    );
    pair(
        "gardens_get_reopened",
        &get_json(garden),
        &gardens::get(ctx, &typed::<GardenGetInput>(&get_json(garden))).unwrap(),
    );
    let error = gardens::reopen(
        ctx,
        owner,
        &typed::<GardenReopenInput>(&json!({
            "id": garden,
            "version": gardens::get(ctx, &typed::<GardenGetInput>(&get_json(garden))).unwrap().row.version,
            "reason": "again"
        })),
    )
    .unwrap_err();
    refusal(
        "gardens_reopen_not_closed",
        &json!({
            "id": garden,
            "version": gardens::get(ctx, &typed::<GardenGetInput>(&get_json(garden))).unwrap().row.version,
            "reason": "again"
        }),
        &error,
        ErrorCode::ErrGardenNotClosed,
    );
    let cancel = json!({
        "id": labour_saved.id, "version": labour_saved.version, "reason": "wrong garden",
        "confirm_cash_below_zero": false
    });
    pair(
        "gardens_entry_cancel_cost",
        &cancel,
        &gardens::entry_cancel(ctx, owner, &typed::<GardenEntryCancelInput>(&cancel)).unwrap(),
    );
    // Cancelling it again is refused: it is cancelled already.
    let error =
        gardens::entry_cancel(ctx, owner, &typed::<GardenEntryCancelInput>(&cancel)).unwrap_err();
    assert!(matches!(error, ServiceError::Rejected(_)));
}

#[test]
fn garden_fixtures_of_e2e_40b_a_loss() {
    let w = support::world();
    let ctx = &w.env.ctx;
    let owner = &w.owner;
    let (garden, _) = story_a_start(&w);
    set_today(&w.env, 2026, 10, 11);
    for (kind, amount, reason) in [
        ("installment", "20,000", "second"),
        ("cost", "5,000", "labour"),
    ] {
        let entry = payment_json(kind, garden, "11-10-2026", amount, reason);
        gardens::entry_save(ctx, owner, &typed::<GardenEntryInput>(&entry)).unwrap();
    }
    // Only the other shop's 10,000 comes in: 10,000 - 55,000 is a loss of 45,000.
    set_today(&w.env, 2026, 10, 12);
    let other = other_sale_json(garden, "12-10-2026", "20", "10,000");
    gardens::entry_save(ctx, owner, &typed::<GardenEntryInput>(&other)).unwrap();
    set_today(&w.env, 2026, 10, 13);
    let closing = close_json(&w, garden, "13-10-2026");
    pair(
        "gardens_close_preview_loss",
        &closing,
        &gardens::close_preview(ctx, &typed::<GardenCloseInput>(&closing)).unwrap(),
    );
    pair(
        "gardens_close_loss",
        &closing,
        &gardens::close(ctx, owner, &typed::<GardenCloseInput>(&closing)).unwrap(),
    );
    pair(
        "gardens_get_closed_loss",
        &get_json(garden),
        &gardens::get(ctx, &typed::<GardenGetInput>(&get_json(garden))).unwrap(),
    );
    // A garden with no sale at all closes at 0 and posts nothing (E64).
    let [a, b, _] = w.partners;
    let empty = garden_json(
        &w,
        None,
        &[partner_json(a, "50", "shop"), partner_json(b, "50", "shop")],
    );
    let mut empty = empty;
    empty["name_ur"] = json!("باغ دو");
    empty["name_en"] = json!("Garden Two");
    empty["confirmed_rules_words"] = json!(["باغ", "دو", "Garden", "Two"]);
    let second = gardens::create(ctx, owner, &typed::<GardenInput>(&empty)).unwrap();
    let closing = close_json(&w, second.row.id, "13-10-2026");
    pair(
        "gardens_close_preview_nothing_to_move",
        &closing,
        &gardens::close_preview(ctx, &typed::<GardenCloseInput>(&closing)).unwrap(),
    );
}

#[test]
fn garden_fixtures_of_a_garden_with_no_entry_yet() {
    let w = support::world();
    let ctx = &w.env.ctx;
    let owner = &w.owner;
    let [a, b, c] = w.partners;
    let made = garden_json(
        &w,
        None,
        &[
            partner_json(a, "60", "shop"),
            partner_json(b, "40", "pocket"),
        ],
    );
    let created = gardens::create(ctx, owner, &typed::<GardenInput>(&made)).unwrap();
    pair("gardens_create_no_installment", &made, &created);
    // No entry yet, so the shares can still change.
    let change = json!({
        "id": created.row.id,
        "version": created.row.version,
        "partners": [
            partner_json(a, "50", "shop"),
            partner_json(b, "30", "pocket"),
            partner_json(c, "20", "shop")
        ]
    });
    pair(
        "gardens_set_partners_three",
        &change,
        &gardens::set_partners(ctx, owner, &typed::<GardenSetPartnersInput>(&change)).unwrap(),
    );
    // A share change that does not total 100% is refused.
    let wrong = json!({
        "id": created.row.id,
        "version": gardens::get(ctx, &typed::<GardenGetInput>(&get_json(created.row.id))).unwrap().row.version,
        "partners": [partner_json(a, "50", "shop"), partner_json(b, "30", "pocket")]
    });
    let error =
        gardens::set_partners(ctx, owner, &typed::<GardenSetPartnersInput>(&wrong)).unwrap_err();
    refusal(
        "gardens_set_partners_not_100",
        &wrong,
        &error,
        ErrorCode::ErrSharesNot100,
    );
    // The first screen of the list while the only garden has no entry.
    pair(
        "gardens_list_no_entry",
        &json!({}),
        &gardens::list(ctx, &GardensListInput::default()).unwrap(),
    );
}
