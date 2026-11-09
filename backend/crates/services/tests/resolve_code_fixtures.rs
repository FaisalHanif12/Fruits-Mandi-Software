//! Real answers of `search_resolve_code` (PRD FR-7.39, plan 64: no one-time party accounts any more),
//! as JSON files for the frontend. Each fixture is a pair: `<name>_request.json` is exactly the
//! `input` the command takes and `<name>.json` is the answer the real service gives for it.
//!
//! The test compares the answers with the files on disk and FAILS if any differ. After a deliberate
//! change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test resolve_code_fixtures`, look at the diff
//! and tell `frontend`.
//!
//! The story: the worked day's world (sellers 0010 and 0011, buyers 0020 and 0021), then a seller 0012,
//! a seller 0013 and a buyer 0022, so that the short code `2` fits a seller and a buyer.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use mandi_services::core::accounts::{AccountType, ResolveCodeInput, ResolveStatus};
use mandi_services::search;
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

#[test]
fn the_real_answers_of_resolve_code_are_the_fixtures() {
    let w = support::world();
    let ctx = &w.env.ctx;
    let resolve = |request: &Value| {
        let input: ResolveCodeInput = serde_json::from_value(request.clone()).unwrap();
        search::resolve_code(ctx, &input).unwrap()
    };

    // The full code of an ordinary buyer is found in a buyer field.
    let request = json!({ "scope": "buyer", "control_id": null, "text": "0020" });
    let resolved = resolve(&request);
    assert_eq!(resolved.status, ResolveStatus::Found);
    assert_eq!(resolved.account.as_ref().map(|a| a.id), Some(w.akram));
    pair("search_resolve_code_buyer_full_0020", &request, &resolved);

    // چکتا کھاتہ is no account a field offers: his code finds nobody (PRD FR-13.4).
    let request = json!({ "scope": "buyer", "control_id": null, "text": "0030" });
    let resolved = resolve(&request);
    assert_eq!(resolved.status, ResolveStatus::Unknown);
    pair(
        "search_resolve_code_buyer_chukta_0030_unknown",
        &request,
        &resolved,
    );

    // In a seller field `2` finds no seller yet (0010 and 0011 only): unknown, and the code an add
    // would use is suggested.
    let request = json!({ "scope": "seller", "control_id": null, "text": "2" });
    let resolved = resolve(&request);
    assert_eq!(resolved.status, ResolveStatus::Unknown);
    assert_eq!(resolved.suggested_code.as_deref(), Some("0012"));
    pair(
        "search_resolve_code_seller_short_2_unknown",
        &request,
        &resolved,
    );

    // Two more sellers (0012, 0013) and a buyer 0022.
    let add = |kind: AccountType, ur: &str, en: &str| {
        support::add_account(&w.env, &w.owner, kind, ur, en, true, "")
    };
    let seller_2 = add(AccountType::Seller, "نذیر", "Nazir");
    let seller_3 = add(AccountType::Seller, "کمال", "Kamal");
    let buyer_3 = add(AccountType::Buyer, "طارق", "Tariq");
    let _ = (seller_2, buyer_3);

    // `3` in a seller field finds کمال alone.
    let request = json!({ "scope": "seller", "control_id": null, "text": "3" });
    let resolved = resolve(&request);
    assert_eq!(resolved.status, ResolveStatus::Found);
    assert_eq!(resolved.account.as_ref().map(|a| a.id), Some(seller_3));
    pair(
        "search_resolve_code_seller_short_3_found",
        &request,
        &resolved,
    );

    // In a field that takes any account `2` lists the seller 0012 and the buyer 0022, each with its type.
    let request = json!({ "scope": "any", "control_id": null, "text": "2" });
    let resolved = resolve(&request);
    assert_eq!(resolved.status, ResolveStatus::Several);
    let codes: Vec<&str> = resolved
        .candidates
        .iter()
        .map(|c| c.code.as_str())
        .collect();
    assert_eq!(codes.len(), 2, "{codes:?}");
    pair(
        "search_resolve_code_any_short_2_several",
        &request,
        &resolved,
    );
}
