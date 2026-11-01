//! Real answers of the plan 47 commands (PRD FR-13.11, v1.15), as JSON files for the frontend: a
//! پختہ party's own account, the next code of the پختہ series, the list filter, the short code in
//! a buyer or a seller field, the party card and the refusals.
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used) and `<name>.json` is the answer the real
//! service gives for it. The frontend copies the files to `frontend/tests/fixtures/`.
//!
//! The test compares the answers with the files on disk and FAILS if any differ, so a fixture can
//! never go stale. After a deliberate change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test pukhta_party_fixtures`, look at the
//! diff and tell `frontend`.
//!
//! The story (plan 47 Section 3): the worked day's world, one more ordinary buyer (0022, so that the
//! short code `2` has two answers), then نذیر a پختہ کھاتہ گاہک (0032, phone 0300-1111111, credit
//! off) and کمال a پختہ کھاتہ بیوپاری (0033).
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_save/helpers.rs"]
mod helpers;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use mandi_services::core::accounts::{
    AccountInput, AccountRow, AccountType, AccountsListInput, AccountsNextCodeInput,
    ResolveCodeInput, ResolveStatus, SearchAccountsInput,
};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, assist, auth, bikri, search};
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

/// The refusal as a command returns it. `ref` is a stand-in: the real one is a counter.
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

/// The request read back into the real input type, so the file is exactly what the command takes.
fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

/// The account form of the screen: a پختہ party (`pukhta` true) or an ordinary buyer.
fn form(kind: &str, pukhta: bool, ur: &str, en: &str, phone: &str, credit: bool) -> Value {
    json!({
        "id": null, "version": null, "account_type": kind, "is_pukhta_party": pukhta,
        "control_id": null, "code": "", "name_ur": ur, "name_en": en,
        "confirmed_rules_words": [], "father_or_firm_ur": "", "father_or_firm_en": "",
        "city_id": null, "adda_id": null, "phone1": phone, "phone2": "", "address": "",
        "linked_account_id": null, "opening_date": "", "opening_dr": "", "opening_cr": "",
        "opening_is_advance": false, "credit_allowed": credit, "credit_limit": "",
        "default_marka_id": null, "collector_staff_id": null, "collection_weekday": null,
        "is_active": true, "note": "", "tag": ""
    })
}

/// Saves a request, confirming the words of the spelling check like the screen does.
fn save(env: &TestEnv, owner: &auth::Actor, request: &Value) -> Result<AccountRow, ServiceError> {
    let mut input: AccountInput = typed(request);
    loop {
        match accounts::save(&env.ctx, owner, &input) {
            Err(ServiceError::Rejected(items))
                if !items.is_empty()
                    && items
                        .iter()
                        .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(words)) = item.values.get("words") {
                        input
                            .confirmed_rules_words
                            .extend(words.split(' ').map(str::to_owned));
                    }
                }
            }
            other => return other,
        }
    }
}

#[test]
fn the_real_answers_of_the_plan_47_commands_are_the_fixtures() {
    let w = support::world();
    let ctx = &w.env.ctx;

    // One more ordinary buyer: 0022, so the short code `2` fits him and the پختہ گاہک 0032.
    let tariq = save(
        &w.env,
        &w.owner,
        &form("buyer", false, "طارق محمود", "Tariq Mehmood", "", true),
    )
    .unwrap();
    assert_eq!(tariq.code, "0022");
    assert!(!tariq.is_pukhta_party);

    // The next code of each side: the پختہ series, from 0032 (0030 and 0031 are the shared ones).
    let ask = |kind: &str, pukhta: bool| json!({ "account_type": kind, "is_pukhta_party": pukhta, "control_id": null });
    let request = ask("buyer", true);
    let next = accounts::next_code_for(ctx, AccountType::Buyer, true, None).unwrap();
    assert_eq!(next, "0032");
    pair("accounts_next_code_pukhta_buyer", &request, &next);
    // The same for a seller: one series for both sides.
    let next = accounts::next_code_for(ctx, AccountType::Seller, true, None).unwrap();
    assert_eq!(next, "0032");
    pair(
        "accounts_next_code_pukhta_seller",
        &ask("seller", true),
        &next,
    );
    // Not a پختہ party: the ordinary series, as before. A request without the field is the same.
    let next = accounts::next_code_for(ctx, AccountType::Buyer, false, None).unwrap();
    assert_eq!(next, "0023");
    let old: AccountsNextCodeInput = typed(&json!({ "account_type": "buyer", "control_id": null }));
    assert!(!old.is_pukhta_party, "a missing flag is false");

    // نذیر: a پختہ کھاتہ گاہک, credit off (cash first, PRD A79).
    let request = form("buyer", true, "نذیر", "Nazir", "0300-1111111", false);
    let nazir = save(&w.env, &w.owner, &request).unwrap();
    assert_eq!(nazir.code, "0032");
    assert_eq!(nazir.serial, 2);
    assert_eq!(nazir.account_type, AccountType::Buyer);
    assert!(nazir.is_pukhta_party && !nazir.is_pukhta_khata && !nazir.is_system);
    assert!(!nazir.credit_allowed);
    pair("accounts_save_pukhta_buyer", &request, &nazir);

    // کمال: a پختہ کھاتہ بیوپاری. The series is shared, so he is 0033.
    let request = form("seller", true, "کمال", "Kamal", "", true);
    let kamal = save(&w.env, &w.owner, &request).unwrap();
    assert_eq!(kamal.code, "0033");
    assert_eq!(kamal.account_type, AccountType::Seller);
    assert!(kamal.is_pukhta_party && !kamal.is_pukhta_khata);
    pair("accounts_save_pukhta_seller", &request, &kamal);

    // The next code after both.
    let next = accounts::next_code_for(ctx, AccountType::Buyer, true, None).unwrap();
    assert_eq!(next, "0034");
    pair(
        "accounts_next_code_pukhta_after_two",
        &ask("buyer", true),
        &next,
    );

    // The list filter پختہ کھاتہ: the two shared accounts and the two own accounts, by code.
    let request = json!({ "filter": { "kind": "pukhta", "text": "", "include_inactive": false,
        "has_opening": null }, "offset": 0, "limit": 0, "sort": null });
    let list = accounts::list(ctx, &typed::<AccountsListInput>(&request)).unwrap();
    let codes: Vec<&str> = list.rows.iter().map(|r| r.code.as_str()).collect();
    assert_eq!(codes, ["0030", "0031", "0032", "0033"]);
    assert_eq!(list.count, 4);
    pair("accounts_list_pukhta", &request, &list);
    // The buyer and seller filters leave him out; "all" has everybody.
    for (kind, leaves_out) in [("buyer", "0032"), ("seller", "0033")] {
        let request = json!({ "filter": { "kind": kind, "text": "", "include_inactive": false,
            "has_opening": null }, "offset": 0, "limit": 0, "sort": null });
        let list = accounts::list(ctx, &typed::<AccountsListInput>(&request)).unwrap();
        assert!(list.rows.iter().all(|r| r.code != leaves_out), "{kind}");
        assert!(list.rows.iter().all(|r| !r.is_pukhta_party), "{kind}");
    }
    let all = json!({ "filter": { "kind": "all", "text": "", "include_inactive": false,
        "has_opening": null }, "offset": 0, "limit": 0, "sort": null });
    let list = accounts::list(ctx, &typed::<AccountsListInput>(&all)).unwrap();
    assert!(list.rows.iter().any(|r| r.code == "0032"));
    assert!(list.rows.iter().any(|r| r.code == "0033"));

    // The short code `2` in a buyer field: the buyer 0022 and the پختہ گاہک 0032, ordinary first.
    let request = json!({ "scope": "buyer", "control_id": null, "text": "2" });
    let resolved = search::resolve_code(ctx, &typed::<ResolveCodeInput>(&request)).unwrap();
    assert_eq!(resolved.status, ResolveStatus::Several);
    let codes: Vec<&str> = resolved
        .candidates
        .iter()
        .map(|c| c.code.as_str())
        .collect();
    assert_eq!(codes, ["0022", "0032"]);
    assert!(resolved.candidates[1].is_pukhta_party && !resolved.candidates[0].is_pukhta_party);
    pair(
        "search_resolve_code_buyer_short_2_several",
        &request,
        &resolved,
    );
    // In a seller field `2` finds no seller (0033 has the running number 3): it is unknown.
    let request = json!({ "scope": "seller", "control_id": null, "text": "2" });
    let resolved = search::resolve_code(ctx, &typed::<ResolveCodeInput>(&request)).unwrap();
    assert_eq!(resolved.status, ResolveStatus::Unknown);
    assert_eq!(resolved.suggested_code.as_deref(), Some("0012"));
    pair(
        "search_resolve_code_seller_short_2_unknown",
        &request,
        &resolved,
    );
    // `3` in a seller field finds کمال alone; in a buyer field nobody.
    let request = json!({ "scope": "seller", "control_id": null, "text": "3" });
    let resolved = search::resolve_code(ctx, &typed::<ResolveCodeInput>(&request)).unwrap();
    assert_eq!(resolved.status, ResolveStatus::Found);
    assert_eq!(resolved.account.as_ref().map(|a| a.id), Some(kamal.id));
    pair(
        "search_resolve_code_seller_short_3_found",
        &request,
        &resolved,
    );
    let request = json!({ "scope": "buyer", "control_id": null, "text": "3" });
    let resolved = search::resolve_code(ctx, &typed::<ResolveCodeInput>(&request)).unwrap();
    assert_eq!(resolved.status, ResolveStatus::Unknown);
    // The full code 0032 works in a buyer field and not in a seller field.
    let request = json!({ "scope": "buyer", "control_id": null, "text": "0032" });
    let resolved = search::resolve_code(ctx, &typed::<ResolveCodeInput>(&request)).unwrap();
    assert_eq!(resolved.status, ResolveStatus::Found);
    assert_eq!(resolved.account.as_ref().map(|a| a.id), Some(nazir.id));
    pair("search_resolve_code_buyer_full_0032", &request, &resolved);
    let request = json!({ "scope": "seller", "control_id": null, "text": "0032" });
    let resolved = search::resolve_code(ctx, &typed::<ResolveCodeInput>(&request)).unwrap();
    assert_eq!(resolved.status, ResolveStatus::Unknown);
    // In a field that takes any account `3` lists the seller کمال only... and `2` lists the buyer
    // 0022 and the پختہ گاہک 0032 (a seller 0012 does not exist).
    let request = json!({ "scope": "any", "control_id": null, "text": "2" });
    let resolved = search::resolve_code(ctx, &typed::<ResolveCodeInput>(&request)).unwrap();
    assert_eq!(resolved.status, ResolveStatus::Several);
    pair(
        "search_resolve_code_any_short_2_several",
        &request,
        &resolved,
    );

    // The picker: typing a name finds him, and the hit says he is a پختہ party and which side.
    let request = json!({ "scope": "buyer", "control_id": null, "text": "nazir", "limit": 10 });
    let hits =
        search::search_accounts(ctx, &w.owner, &typed::<SearchAccountsInput>(&request)).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].is_pukhta_party);
    assert_eq!(hits[0].account_type, AccountType::Buyer);
    pair("search_accounts_pukhta_buyer", &request, &hits);
    // He is in no seller field, and کمال is in no buyer field.
    let request = json!({ "scope": "seller", "control_id": null, "text": "nazir", "limit": 10 });
    let hits =
        search::search_accounts(ctx, &w.owner, &typed::<SearchAccountsInput>(&request)).unwrap();
    assert!(hits.is_empty());
    let request = json!({ "scope": "buyer", "control_id": null, "text": "kamal", "limit": 10 });
    let hits =
        search::search_accounts(ctx, &w.owner, &typed::<SearchAccountsInput>(&request)).unwrap();
    assert!(hits.is_empty());

    // The party card carries the mark and the side: a full buyer's card (credit limit and used).
    let card = assist::party_card(ctx, nazir.id).unwrap();
    assert!(card.is_pukhta_party);
    assert_eq!(card.account_type, AccountType::Buyer);
    assert!(
        card.credit_used.is_some(),
        "a buyer's card shows credit used"
    );
    pair(
        "assist_party_card_pukhta_buyer",
        &json!({ "account_id": nazir.id }),
        &card,
    );
    let card = assist::party_card(ctx, kamal.id).unwrap();
    assert!(card.is_pukhta_party && card.advance_open.is_some());

    // Refusals. Only a seller or a buyer can be a پختہ party.
    let mut request = form("money_lent", true, "اسلم", "Aslam", "", true);
    request["is_pukhta_party"] = json!(true);
    let error = save(&w.env, &w.owner, &request).unwrap_err();
    let ServiceError::Rejected(items) = &error else {
        panic!("{error:?}")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrFormatInvalid, Some("is_pukhta_party"))
    );
    write_or_check(
        "accounts_save_pukhta_party_wrong_type_request.json",
        &(serde_json::to_string_pretty(&request).unwrap() + "\n"),
    );
    write_or_check(
        "accounts_save_pukhta_party_wrong_type.json",
        &(serde_json::to_string_pretty(&wire(&error)).unwrap() + "\n"),
    );
    // And the flag cannot change on an edit.
    let mut edit = form("buyer", false, "نذیر", "Nazir", "0300-1111111", false);
    edit["id"] = json!(nazir.id);
    edit["version"] = json!(nazir.version);
    edit["code"] = json!(nazir.code);
    let error = save(&w.env, &w.owner, &edit).unwrap_err();
    let ServiceError::Rejected(items) = &error else {
        panic!("{error:?}")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrFormatInvalid, Some("is_pukhta_party"))
    );
    write_or_check(
        "accounts_save_pukhta_party_change_request.json",
        &(serde_json::to_string_pretty(&edit).unwrap() + "\n"),
    );
    write_or_check(
        "accounts_save_pukhta_party_change.json",
        &(serde_json::to_string_pretty(&wire(&error)).unwrap() + "\n"),
    );

    // The sale screen's preview (PRD 4.5 numbers, 10 Apple at 1,200 from Karim's lot shape): the
    // party buyer starts as cash; the party seller with an open advance of 5,000 gets the default
    // نقد 10,160 - 5,000 = 5,160 (Umar, 01-11-2026) and the working that says so.
    let lot_json = |seller: i64, buyer: i64| -> Value {
        let mut line = support::sale("s-1", buyer, "10", "1200");
        line.payment_type = None;
        let mut l = support::lot(&w, "lot-1", seller, w.apple, "10");
        l.freight = Some("1,000".into());
        l.cash_paid = None;
        l.sales = vec![line];
        json!({ "draft": support::draft(vec![l]) })
    };
    let request = lot_json(w.karim, nazir.id);
    let input: mandi_services::core::bikri::BikriPreviewInput = typed(&request);
    let shown = bikri::preview(ctx, &input.draft).unwrap();
    assert!(shown.errors.is_empty(), "{:?}", shown.errors);
    assert_eq!(
        support::calc(&shown.lots[0], 0).payment_type,
        mandi_services::core::bikri::PaymentType::Cash
    );
    pair("bikri_preview_pukhta_party_buyer_cash", &request, &shown);

    helpers::give_advance(&w, kamal.id, "5,000");
    let request = lot_json(kamal.id, w.akram);
    let input: mandi_services::core::bikri::BikriPreviewInput = typed(&request);
    let shown = bikri::preview(ctx, &input.draft).unwrap();
    assert!(shown.errors.is_empty(), "{:?}", shown.errors);
    assert_eq!(
        (shown.summary.cash_paid.rupees(), shown.summary.net.rupees()),
        (5_160, 5_000)
    );
    pair(
        "bikri_preview_pukhta_party_seller_advance",
        &request,
        &shown,
    );

    // The warning (Umar, option 2): a بکری of 12-10 is saved (cash 5,160, balance 0); a back-dated
    // بکری of 11-10 of the same seller comes before it and takes its 5,000, so the preview warns
    // with `taken_paisa` 500000 on the نقد of the first lot. The default is still 5,160.
    common::set_today(&w.env, 2026, 10, 12);
    let dated = |day: &str, seller: i64, buyer: i64| -> Value {
        let mut value = lot_json(seller, buyer);
        value["draft"]["header"]["entry_date"] = json!(day);
        value
    };
    let later: mandi_services::core::bikri::BikriPreviewInput =
        typed(&dated("12-10-2026", kamal.id, w.akram));
    helpers::save_ok(&w, &later.draft);
    let request = dated("11-10-2026", kamal.id, w.arshad);
    let input: mandi_services::core::bikri::BikriPreviewInput = typed(&request);
    let shown = bikri::preview(ctx, &input.draft).unwrap();
    assert!(shown.errors.is_empty(), "{:?}", shown.errors);
    assert_eq!(shown.summary.cash_paid.rupees(), 5_160);
    assert_eq!(shown.warnings.len(), 1, "{:?}", shown.warnings);
    assert_eq!(
        (
            shown.warnings[0].code,
            shown.warnings[0].field.as_str(),
            shown.warnings[0].values.get("taken_paisa")
        ),
        (
            ErrorCode::ErrAdvanceTakenFromLater,
            "lots[0].cash_paid",
            Some(&ErrorValue::Number(500_000))
        )
    );
    pair(
        "bikri_preview_pukhta_party_seller_advance_taken",
        &request,
        &shown,
    );
}
