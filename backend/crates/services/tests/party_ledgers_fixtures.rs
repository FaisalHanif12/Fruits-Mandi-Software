//! Plan 66: the real answers of the party ledgers commands (`party_ledgers_list` and
//! `party_ledgers_statement`) on the PRD worked day after Events 1 to 5 (10-10-26), as JSON files
//! for the frontend. The third level is the R35 slip, whose fixtures are in `day_slips_fixtures`.
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used), and `<name>.json` is the answer the real
//! service gives for it. The frontend copies the files to `frontend/tests/fixtures/`.
//!
//! The test compares the answers with the files on disk and FAILS if any differ. After a
//! deliberate change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test party_ledgers_fixtures`, look at the
//! diff, and tell `frontend`.
//!
//! The numbers are the PRD's (E2E-67): اکرم 2,880 بنام and ارشد 18,684 بنام; اکرم on 10-10-26
//! previous 0, charged 12,880, paid 10,000, balance 2,880; رحیم on 10-10-26 previous 15,000 بنام,
//! charged 24,316, paid 9,316, balance 0.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use mandi_services::core::party_ledgers::{PartyLedgerStatementInput, PartyLedgersListInput};
use mandi_services::party_ledgers;
use reports_world::full_day;
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

#[test]
fn the_party_ledgers_fixtures_are_the_real_answers() {
    let w = full_day();
    for (name, side) in [
        ("party_ledgers_list_buyers", "buyer"),
        ("party_ledgers_list_sellers", "seller"),
    ] {
        let request = json!({ "side": side, "text": null, "offset": 0, "limit": 0 });
        let input: PartyLedgersListInput = serde_json::from_value(request.clone()).unwrap();
        pair(
            name,
            &request,
            &party_ledgers::list(&w.env.ctx, &input).unwrap(),
        );
    }
    for (name, account_id) in [
        ("party_ledgers_statement_akram", w.akram),
        ("party_ledgers_statement_rahim", w.rahim),
    ] {
        let request = json!({
            "account_id": account_id, "from": null, "to": null, "offset": 0, "limit": 0
        });
        let input: PartyLedgerStatementInput = serde_json::from_value(request.clone()).unwrap();
        pair(
            name,
            &request,
            &party_ledgers::statement(&w.env.ctx, &input).unwrap(),
        );
    }
}
