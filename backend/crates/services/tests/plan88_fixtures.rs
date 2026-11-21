//! Real answers of the plan 88 commands, as JSON files for the frontend (plan 88, PRD 15.4 and
//! FR-15.11 v1.51): a پیشگی given to a buyer, saved as cash lent. The seller-or-buyer picker, the
//! preview and the save of a buyer's advance, its refusal without a reason, the پیشگی list with its
//! buyer rows and the first day of its window.
//!
//! Each fixture is a pair, as in `voucher_fixtures.rs`: `<name>_request.json` is exactly the `input`
//! the command takes, `<name>.json` the answer of the real service on the PRD worked day. The test
//! compares the files with the real answers and FAILS if they differ. After a deliberate change write
//! them again with `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test plan88_fixtures`, read
//! the diff and tell `frontend`.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use mandi_services::core::accounts::SearchAccountsInput;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::{
    AdvanceInput, AdvancesInput, VoucherPreviewInput, VouchersListInput,
};
use mandi_services::core::windows::WindowsBoundsInput;
use mandi_services::{ServiceError, search, vouchers, windows};
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

fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

fn advance_json(account: i64, amount: &str, reason: Option<&str>) -> Value {
    let mut value = json!({
        "date": "10-10-2026", "seller_account_id": account, "amount": amount,
        "expected_goods": "", "narration": "", "confirm_cash_below_zero": false
    });
    if let Some(reason) = reason {
        value["reason"] = json!(reason);
    }
    value
}

#[test]
fn the_real_answers_of_the_plan_88_commands_are_the_fixtures() {
    let w = support::world();
    let (ctx, owner) = (&w.env.ctx, &w.owner);

    // 1. The picker of the پیشگی field: sellers and buyers, each with its type.
    let picker_request = json!({
        "scope": "seller_buyer", "control_id": null, "text": "", "limit": 20, "offset": 0
    });
    let hits = search::search_accounts(ctx, owner, &typed::<SearchAccountsInput>(&picker_request))
        .unwrap();
    assert_eq!(
        hits.iter().map(|h| h.code.as_str()).collect::<Vec<_>>(),
        ["0010", "0011", "0020", "0021"]
    );
    pair("search_accounts_seller_buyer", &picker_request, &hits);

    // 2. A seller's advance first (رحیم, 15,000): unchanged, a پیشگی voucher.
    let seller = advance_json(w.rahim, "15,000", None);
    let seller_saved = vouchers::save_advance(ctx, owner, &typed::<AdvanceInput>(&seller)).unwrap();
    assert_eq!(seller_saved.number, 1);

    // 3. A پیشگی of 3,000 to اکرم (E2E-32): the preview is the preview of a payment to him; the
    // save is a payment voucher (ا-1) with his new balance.
    let buyer = advance_json(w.akram, "3,000", Some("نقد ادھار"));
    let preview_request = json!({
        "entry": { "type": "advance", "date": "10-10-2026", "seller_account_id": w.akram,
                   "amount": "3,000", "expected_goods": "", "narration": "",
                   "confirm_cash_below_zero": false, "reason": "نقد ادھار" },
        "replacing_voucher_id": null
    });
    let preview = vouchers::preview(ctx, &typed::<VoucherPreviewInput>(&preview_request)).unwrap();
    assert!(preview.errors.is_empty());
    assert_eq!(preview.new_balance.amount_paisa.rupees(), 3_000);
    pair("vouchers_preview_advance_buyer", &preview_request, &preview);
    let no_reason = advance_json(w.akram, "3,000", None);
    let error = vouchers::save_advance(ctx, owner, &typed::<AdvanceInput>(&no_reason)).unwrap_err();
    refusal(
        "vouchers_save_advance_buyer_needs_reason",
        &no_reason,
        &error,
        ErrorCode::ErrPaymentToBuyerNeedsReason,
    );
    let buyer_saved = vouchers::save_advance(ctx, owner, &typed::<AdvanceInput>(&buyer)).unwrap();
    assert_eq!(
        serde_json::to_value(buyer_saved.voucher_type).unwrap(),
        json!("payment")
    );
    assert_eq!(buyer_saved.number, 1);
    pair("vouchers_save_advance_buyer", &buyer, &buyer_saved);

    // 4. The running list, a seller's line and a buyer's line.
    let list_input = json!({ "lines": [
        advance_json(w.karim, "2,000", None),
        advance_json(w.arshad, "1,000", Some("نقد ادھار"))
    ] });
    let list_preview =
        vouchers::preview_advances(ctx, &typed::<AdvancesInput>(&list_input)).unwrap();
    assert!(list_preview.rows.iter().all(|r| r.errors.is_empty()));
    pair(
        "vouchers_preview_advances_buyer_and_seller",
        &list_input,
        &list_preview,
    );

    // 5. The پیشگی list: the advance and the cash lent, each with its account type.
    let list_request = json!({
        "voucher_type": "advance", "from": "", "to": "", "account_id": null, "offset": 0,
        "limit": 0, "show_finished": false, "advance_view": true
    });
    let list = vouchers::list(ctx, &typed::<VouchersListInput>(&list_request)).unwrap();
    assert_eq!(list.total_count, 2);
    pair("vouchers_list_advances_view", &list_request, &list);

    // 6. The first day of that list's window.
    let bounds_request = json!({ "source": { "kind": "advances" } });
    let bounds = windows::bounds(ctx, &typed::<WindowsBoundsInput>(&bounds_request)).unwrap();
    assert_eq!(bounds.first_date.as_deref(), Some("2026-10-10"));
    pair("windows_bounds_advances", &bounds_request, &bounds);
}
