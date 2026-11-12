//! Plan 44 T1: the contract of "edit everywhere", the edit stamp, reopen and fix, and the partner
//! code, as JSON files for the frontend.
//!
//! These are examples of the answers the T2 work will give, built from the PRD worked numbers and
//! typed in Rust, so the files can never disagree with the real types. They are not read from a
//! database. Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes
//! (it is read back into the real input type before it is used), and `<name>.json` is the answer.
//! A refusal has the shape of the error a command returns (`ok`, `errors`, `ref`), where `ref` is a
//! stand-in because the real reference number is a counter. The frontend copies the files to
//! `frontend/tests/fixtures/`.
//!
//! The test compares the answers with the files on disk and FAILS if any differ. After a deliberate
//! change, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test edit_contract_fixtures`, read the
//! diff, and tell `frontend`.
//!
//! The numbers are the PRD's:
//! - E2E-13 and E2E-45: بکری 1, اکرم's rate 600 → 610. He pays 12,200 + 732 + 160 = 13,092 (was
//!   12,880). Seller credit 24,504 (was 24,316). Shared income 3,972 (was 3,948), split 1,590 /
//!   1,191 / 1,191 (was 1,580 / 1,184 / 1,184).
//! - E2E-34: the trip purchase of 20 پیٹی مالٹا at 500 with freight 400 costs 10,400, paid from the
//!   shop's cash 4,160 / 3,120 / 3,120. At 520 it costs 10,800: 4,320 / 3,240 / 3,240. Cash 39,600
//!   becomes 39,200.
//! - FR-15.3: a receipt of 10,000 edited to 4,000. اکرم 2,880 → 8,880 بنام, cash 58,484 → 52,484.
//! - E2E-42: الف's home fruit, 2 پیٹی at 580 = 1,160, edited to 3 پیٹی = 1,740. رحیم's credit
//!   2,750 → 3,330.
#![cfg(test)]

use std::path::{Path, PathBuf};

use mandi_services::ServiceError;
use mandi_services::core::accounts::{Balance, BalanceSide, RecordRef};
use mandi_services::core::edit::{
    ChangeValue, DayFixCloseInput, DayFixClosed, DayFixOpenInput, DayFixStayedOpen, EditChange,
    EditEffect, EditPlace, EditPlaceKind, EditPreview, EditReview, EditStamp, EffectPartner,
    EffectParty, EntryFlags, LockedReason,
};
use mandi_services::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_services::core::money::Paisa;
use mandi_services::core::partner_code::{
    PartnerCodeChangeInput, PartnerCodeResetInput, PartnerCodeSetInput, PartnerCodeStatus,
    PartnerCodeUnlockInput,
};
use mandi_services::core::partners::{PartnerRow, PartnersList};
use serde::Serialize;
use serde_json::{Value, json};

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
        "{file} is not the contract any more. If the change is on purpose, run with \
         {WRITE_VARIABLE}=1, read the diff and tell frontend."
    );
}

/// One pair of files: the request and the answer.
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

fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

fn record(id: i64, code: &str, ur: &str, en: &str) -> RecordRef {
    RecordRef {
        id,
        code: code.to_owned(),
        name_ur: ur.to_owned(),
        name_en: en.to_owned(),
        tag: None,
    }
}

fn rahim() -> RecordRef {
    record(2, "030110001", "رحیم", "Rahim")
}

fn partner(id: i64, ur: &str, en: &str) -> RecordRef {
    record(id, "", ur, en)
}

fn cr(rupees: i64) -> Balance {
    Balance {
        amount_paisa: rs(rupees),
        side: BalanceSide::Cr,
    }
}

fn money(rupees: i64) -> ChangeValue {
    ChangeValue::Money { paisa: rs(rupees) }
}

fn entry_place() -> EditPlace {
    EditPlace {
        kind: EditPlaceKind::Entry,
        lot_no: None,
        lot_counter: None,
        item_ur: None,
        item_en: None,
        line_no: None,
        label_ur: None,
        label_en: None,
    }
}

fn change(place: EditPlace, field: &str, before: ChangeValue, after: ChangeValue) -> EditChange {
    EditChange {
        place,
        field: field.to_owned(),
        before,
        after,
    }
}

fn effect_partner(id: i64, ur: &str, en: &str, before: i64, after: i64) -> EffectPartner {
    EffectPartner {
        partner: partner(id, ur, en),
        before_paisa: rs(before),
        after_paisa: rs(after),
    }
}

fn stamp(count: i64, reason: Option<&str>) -> EditStamp {
    // 11-10-26, 4:35 pm in Pakistan, is 11:35 UTC.
    EditStamp {
        at: "2026-10-11T11:35:00.000Z".to_owned(),
        local_date: "2026-10-11".to_owned(),
        local_time: "16:35".to_owned(),
        by: "Maher Adnan".to_owned(),
        count,
        reason: reason.map(str::to_owned),
    }
}

fn preview(review: EditReview) -> EditPreview {
    EditPreview {
        errors: Vec::new(),
        review,
    }
}

// ---- The review of an edit (change list and effect) ----

#[test]
fn the_edit_reviews_of_the_old_edits_are_the_contract() {
    // The reviews of a بکری and of a voucher are real answers now: `edit_review.rs` writes
    // `bikri_edit_review_akram_610` and `vouchers_edit_preview_receipt_4000`.

    // E2E-42: الف's home fruit, 2 پیٹی at 580 edited to 3 پیٹی.
    let request = json!({
        "id": 1, "version": 1, "reason": "گنتی غلط تھی",
        "entry": { "for_guests": false, "partner_id": 1, "date": "10-10-2026", "source": "shop",
                   "lot_id": 1, "agent_account_id": null, "item_id": null, "packing_id": null,
                   "qty": "3", "weight": null, "rate": "580", "note": null, "editing_id": null }
    });
    let review = EditReview {
        entry_date: "2026-10-10".to_owned(),
        changes: vec![
            change(
                entry_place(),
                "qty",
                ChangeValue::Number { value: 2 },
                ChangeValue::Number { value: 3 },
            ),
            change(entry_place(), "value", money(1_160), money(1_740)),
        ],
        effect: EditEffect {
            parties: vec![EffectParty {
                account: rahim(),
                before: cr(2_750),
                after: cr(3_330),
            }],
            cash: None,
            partners: vec![effect_partner(1, "الف", "Partner A", -1_160, -1_740)],
            partners_locked: false,
        },
        reason_required: true,
        needs_reopen: false,
    };
    let _ = typed::<mandi_services::core::fruit::FruitEditInput>(&request);
    pair("fruit_edit_preview_home_3", &request, &preview(review));
}

// ---- Row flags ----

#[test]
fn the_row_flags_are_the_contract() {
    // A row an Edit button works on, one of a closed day, one edited and one locked by moved goods.
    let open = EntryFlags::open();
    let closed_day = EntryFlags {
        needs_reopen: true,
        ..EntryFlags::open()
    };
    let edited = EntryFlags {
        edited: Some(stamp(2, Some("ریٹ غلط تھا"))),
        ..EntryFlags::open()
    };
    let locked = EntryFlags::locked(LockedReason::GoodsMoved);
    let all = json!({
        "open": open, "closed_day": closed_day, "edited": edited, "goods_moved": locked
    });
    write_or_check(
        "entry_flags_examples.json",
        &(serde_json::to_string_pretty(&all).unwrap() + "\n"),
    );
}

// ---- Reopen and fix ----

#[test]
fn reopen_and_fix_is_the_contract() {
    // The open and the close of E2E-45 are real answers: `golden_day::fix` writes
    // `day_fix_open_e2e_45` and `day_fix_close_e2e_45`. Here are the answers it cannot make.
    let request = json!({ "token": "fix-20261012-0001" });
    let _ = typed::<DayFixCloseInput>(&request);
    let _ = typed::<DayFixOpenInput>(&json!({ "date": "10-10-2026", "reason": "x" }));

    // A day whose checks fail stays open, and so does every day after it.
    let stayed = DayFixClosed {
        closed: vec!["2026-10-10".to_owned()],
        stayed_open: vec![
            DayFixStayedOpen {
                date: "2026-10-11".to_owned(),
                errors: vec![FieldError::new(ErrorCode::ErrTrialNotBalanced, "date")],
            },
            DayFixStayedOpen {
                date: "2026-10-12".to_owned(),
                errors: Vec::new(),
            },
        ],
        closed_up_to: Some("2026-10-10".to_owned()),
    };
    pair("day_fix_close_stayed_open", &request, &stayed);

    let refused = ServiceError::reject_field(ErrorCode::ErrFixTokenInvalid, "token");
    refusal(
        "day_fix_close_token_invalid",
        &json!({ "token": "wrong" }),
        &refused,
        ErrorCode::ErrFixTokenInvalid,
    );
}

// ---- Accounts on any day, and used rules ----

// ---- The partner code ----

fn partner_row(id: i64, ur: &str, en: &str, bps: i32) -> PartnerRow {
    PartnerRow {
        id,
        name_ur: ur.to_owned(),
        name_en: en.to_owned(),
        capital_account_id: 10 + id,
        capital_account_code: format!("0602000{id}"),
        drawings_account_id: 20 + id,
        drawings_account_code: format!("0603000{id}"),
        venture_account_id: 30 + id,
        venture_account_code: format!("0604000{id}"),
        share_bps: bps,
        is_active: true,
        version: 1,
    }
}

#[test]
fn the_partner_code_is_the_contract() {
    // The status in its four states.
    for (name, status) in [
        (
            "partners_code_status_not_set",
            PartnerCodeStatus {
                is_set: false,
                unlocked: false,
                wait_seconds: 0,
            },
        ),
        (
            "partners_code_status_locked",
            PartnerCodeStatus {
                is_set: true,
                unlocked: false,
                wait_seconds: 0,
            },
        ),
        (
            "partners_code_status_unlocked",
            PartnerCodeStatus {
                is_set: true,
                unlocked: true,
                wait_seconds: 0,
            },
        ),
        (
            "partners_code_status_waiting",
            PartnerCodeStatus {
                is_set: true,
                unlocked: false,
                wait_seconds: 60,
            },
        ),
    ] {
        pair(name, &json!({}), &status);
    }

    // set / change / reset take the code twice; unlock takes it once.
    let request = json!({ "password": "secret1", "code": "4826", "code_again": "4826" });
    let _ = typed::<PartnerCodeSetInput>(&request);
    pair(
        "partners_code_set",
        &request,
        &PartnerCodeStatus {
            is_set: true,
            unlocked: true,
            wait_seconds: 0,
        },
    );
    let _ = typed::<PartnerCodeChangeInput>(&json!({
        "old_code_or_password": "4826", "code": "7391", "code_again": "7391"
    }));
    let _ = typed::<PartnerCodeResetInput>(&json!({
        "password_or_recovery": "secret1", "code": "7391", "code_again": "7391"
    }));
    let request = json!({ "code": "4826" });
    let _ = typed::<PartnerCodeUnlockInput>(&request);
    pair(
        "partners_code_unlock",
        &request,
        &PartnerCodeStatus {
            is_set: true,
            unlocked: true,
            wait_seconds: 0,
        },
    );

    // E2E-46: a wrong try says how many are left (9 after the first); after ten the wait comes
    // instead (Umar, 12-11-2026).
    let wrong = ServiceError::Rejected(vec![mandi_services::Rejection {
        code: ErrorCode::ErrPartnerCodeWrong,
        field: Some("code".to_owned()),
        values: [("tries_left".to_owned(), ErrorValue::Number(9))]
            .into_iter()
            .collect(),
    }]);
    refusal(
        "partners_code_unlock_wrong",
        &json!({ "code": "1111" }),
        &wrong,
        ErrorCode::ErrPartnerCodeWrong,
    );
    let wait = ServiceError::Rejected(vec![mandi_services::Rejection {
        code: ErrorCode::ErrLockedWait,
        field: Some("code".to_owned()),
        values: [("seconds".to_owned(), ErrorValue::Number(60))]
            .into_iter()
            .collect(),
    }]);
    refusal(
        "partners_code_unlock_wait",
        &json!({ "code": "1111" }),
        &wait,
        ErrorCode::ErrLockedWait,
    );
    let differ = ServiceError::reject_field(ErrorCode::ErrPartnerCodesDiffer, "code_again");
    refusal(
        "partners_code_set_codes_differ",
        &json!({ "password": "secret1", "code": "4826", "code_again": "4827" }),
        &differ,
        ErrorCode::ErrPartnerCodesDiffer,
    );

    // A command behind the code, called while it is locked (for example partners_overview).
    let locked = ServiceError::reject(ErrorCode::ErrPartnerLocked);
    refusal(
        "partners_overview_locked",
        &json!({}),
        &locked,
        ErrorCode::ErrPartnerLocked,
    );

    // partners_list: open, with the shares; locked, names only and a flag.
    let open = PartnersList {
        partners: vec![
            partner_row(1, "الف", "Partner A", 4_000),
            partner_row(2, "ب", "Partner B", 3_000),
            partner_row(3, "ج", "Partner C", 3_000),
        ],
        partners_locked: false,
    };
    pair("partners_list_open", &json!({}), &open);
    let hidden = PartnersList {
        partners: vec![
            partner_row(1, "الف", "Partner A", 0),
            partner_row(2, "ب", "Partner B", 0),
            partner_row(3, "ج", "Partner C", 0),
        ],
        partners_locked: true,
    };
    pair("partners_list_locked", &json!({}), &hidden);
}
