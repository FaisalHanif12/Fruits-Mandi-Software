//! Plan 44 T2d (PRD FR-10.11, E79): an account or a list record can be added or changed on any day.
//! Only a new or changed opening balance posts, so only it needs an open date.
//!
//! Umar's data (31-10-2026): the days up to 30-10-26 are closed, today is 31-10-26. A new buyer with
//! no opening saves at once, even when the form sends the closed date 30-10-26 as the opening date
//! (the quick-add of the old screen did). An opening amount on 30-10-26 is refused, and the refusal
//! carries `closed_up_to` 30-10-2026, `today` 31-10-2026 and `today_open` 1, so the form can offer
//! "use today". When today is closed too (the days up to 31-10-26 are closed at night) the same
//! refusal carries `today_open` 0, and the form offers "save without the opening" or "reopen the
//! day".
//!
//! The refusals of the form are also written as fixtures for the frontend, with the real answers
//! (`MANDI_WRITE_FIXTURES=1` writes them; otherwise the test checks them).
#![cfg(test)]

mod common;

use std::path::{Path, PathBuf};

use common::*;
use mandi_services::core::accounts::{AccountInput, AccountRow, Balance, BalanceSide};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::masters::{CityInput, MasterInput};
use mandi_services::core::money::Paisa;
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, auth, ledger, masters};
use serde_json::{Value, json};
use tempfile::TempDir;

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn write_or_check(file: &str, text: &str) {
    let path = folder().join(file);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
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

fn pair(name: &str, request: &Value, error: &ServiceError) {
    write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(&wire(error)).unwrap() + "\n"),
    );
}

/// The owner is signed in, today is 31-10-2026 and the days up to `closed_up_to` are closed.
fn world(closed_up_to: &str) -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    set_today(&env, 2026, 10, 31);
    raw(&dir)
        .execute(
            "UPDATE day_state SET closed_up_to_date = ?1 WHERE id = 1",
            [closed_up_to],
        )
        .unwrap();
    (dir, env, owner)
}

fn count(dir: &TempDir, sql: &str) -> i64 {
    raw(dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

/// The buyer form of the quick-add: both names, and whatever opening the caller types.
fn buyer_request(date: &str, dr: &str) -> Value {
    json!({
        "id": null, "version": null, "account_type": "buyer", "control_id": null, "code": "",
        "name_ur": "طارق محمود", "name_en": "Tariq Mehmood", "confirmed_rules_words": [],
        "father_or_firm_ur": "", "father_or_firm_en": "", "city_id": null, "adda_id": null,
        "phone1": "", "phone2": "", "address": "", "linked_account_id": null,
        "opening_date": date, "opening_dr": dr, "opening_cr": "", "opening_is_advance": false,
        "credit_allowed": true, "credit_limit": "", "default_marka_id": null,
        "collector_staff_id": null, "is_active": true, "note": "",
        "tag": ""
    })
}

/// Saves a request, confirming the words of the spelling check like the screen does.
fn save(env: &TestEnv, owner: &auth::Actor, request: &Value) -> Result<AccountRow, ServiceError> {
    let mut input: AccountInput = serde_json::from_value(request.clone()).unwrap();
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

fn text_value(error: &ServiceError, key: &str) -> Option<String> {
    let ServiceError::Rejected(items) = error else {
        return None;
    };
    match items.first()?.values.get(key)? {
        ErrorValue::Text(text) => Some(text.clone()),
        ErrorValue::Number(_) => None,
    }
}

fn number_value(error: &ServiceError, key: &str) -> Option<i64> {
    let ServiceError::Rejected(items) = error else {
        return None;
    };
    match items.first()?.values.get(key)? {
        ErrorValue::Number(n) => Some(*n),
        ErrorValue::Text(_) => None,
    }
}

#[test]
fn a_new_buyer_with_no_opening_saves_on_a_closed_day() {
    let (dir, env, owner) = world("2026-10-30");
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");

    // The form sends the closed date 30-10-26 and no amount: there is nothing to post, so the date
    // is not checked.
    let tariq = save(&env, &owner, &buyer_request("30-10-2026", "")).unwrap();
    assert_eq!(tariq.name_en, "Tariq Mehmood");
    assert_eq!(tariq.opening_dr_paisa, Paisa::ZERO);
    assert_eq!(tariq.opening_date, None);
    // With no date at all it saves too.
    let mut second = buyer_request("", "");
    second["name_ur"] = json!("ناصر علی");
    second["name_en"] = json!("Nasir Ali");
    save(&env, &owner, &second).unwrap();
    // A typed 0 is the same as nothing.
    let mut third = buyer_request("30-10-2026", "0");
    third["name_ur"] = json!("شاہد خان");
    third["name_en"] = json!("Shahid Khan");
    save(&env, &owner, &third).unwrap();
    // Nothing was posted by any of the three.
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);

    // A list record saves on a closed day too.
    masters::save(
        &env.ctx,
        &owner,
        &MasterInput::City(CityInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: "ملتان".into(),
            name_en: "Multan".into(),
            confirmed_rules_words: vec!["ملتان".into()],
        }),
    )
    .unwrap();

    // Changing a phone or a name later works on any day, and the opening stays empty.
    let mut edit = buyer_request("", "");
    edit["id"] = json!(tariq.id);
    edit["version"] = json!(tariq.version);
    edit["phone1"] = json!("0300-1234567");
    let edited = save(&env, &owner, &edit).unwrap();
    assert_eq!(edited.phone1, "03001234567");
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
    ledger::verify_journal_chain(&env.ctx).unwrap();
}

/// The audit rows of an opening posted on a closed day.
fn closed_day_audits(dir: &TempDir) -> i64 {
    count(
        dir,
        "SELECT COUNT(*) FROM audit_log WHERE entity = 'account' \
         AND action = 'opening_on_closed_day' AND reason = 'opening on a closed day'",
    )
}

/// The debits and the credits of every line in the books.
fn books(dir: &TempDir) -> (i64, i64) {
    (
        count(dir, "SELECT COALESCE(SUM(dr_paisa), 0) FROM journal_lines"),
        count(dir, "SELECT COALESCE(SUM(cr_paisa), 0) FROM journal_lines"),
    )
}

#[test]
fn a_new_account_saves_its_opening_on_a_closed_day_and_the_books_balance() {
    // Plan 53 T1 (PRD FR-10.11): the days up to 30-10-26 are closed, today is 31-10-26. A new buyer
    // طارق with 5,000 بنام on 30-10-26 saves, and the opening is dated 30-10-26.
    let (dir, env, owner) = world("2026-10-30");
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let tariq = save(&env, &owner, &buyer_request("30-10-2026", "5,000")).unwrap();
    assert_eq!(tariq.opening_dr_paisa, Paisa::from_rupees(5_000).unwrap());
    assert_eq!(tariq.opening_date.as_deref(), Some("2026-10-30"));
    assert_eq!(
        tariq.balance,
        Balance {
            amount_paisa: Paisa::from_rupees(5_000).unwrap(),
            side: BalanceSide::Dr,
        }
    );
    // One opening entry, dated 30-10-26: طارق بنام 5,000, ابتدائی فرق جمع 5,000.
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM journal_entries"),
        entries + 1
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM journal_entries WHERE posting_type = 'opening' \
             AND entry_date = '2026-10-30' AND status = 'posted'"
        ),
        1
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM journal_lines WHERE dr_paisa = 500000 AND account_id = \
             (SELECT id FROM accounts WHERE name_en = 'Tariq Mehmood')"
        ),
        1
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM journal_lines WHERE cr_paisa = 500000 AND account_id = \
             (SELECT id FROM accounts WHERE code = '06019999')"
        ),
        1
    );
    // The books still balance, and an audit row says what was done and why.
    let (dr, cr) = books(&dir);
    assert_eq!(dr, cr);
    assert_eq!(closed_day_audits(&dir), 1);
    ledger::verify_journal_chain(&env.ctx).unwrap();

    // An opening on an open day writes no such row.
    let mut open = buyer_request("31-10-2026", "2,000");
    open["name_ur"] = json!("ناصر علی");
    open["name_en"] = json!("Nasir Ali");
    save(&env, &owner, &open).unwrap();
    assert_eq!(closed_day_audits(&dir), 1);
}

#[test]
fn a_new_account_saves_its_opening_even_when_today_is_closed_too() {
    // Closed at night: the days up to 31-10-26, today included, are closed.
    let (dir, env, owner) = world("2026-10-31");
    let tariq = save(&env, &owner, &buyer_request("31-10-2026", "5,000")).unwrap();
    assert_eq!(tariq.opening_date.as_deref(), Some("2026-10-31"));
    assert_eq!(closed_day_audits(&dir), 1);
    let (dr, cr) = books(&dir);
    assert_eq!(dr, cr);
    // An old closed day too.
    let mut old = buyer_request("01-01-2026", "300");
    old["name_ur"] = json!("ناصر علی");
    old["name_en"] = json!("Nasir Ali");
    save(&env, &owner, &old).unwrap();
    assert_eq!(closed_day_audits(&dir), 2);
    ledger::verify_journal_chain(&env.ctx).unwrap();
}

#[test]
fn a_new_account_with_an_opening_after_today_is_refused_and_nothing_is_written() {
    let (dir, env, owner) = world("2026-10-30");
    let accounts_before = count(&dir, "SELECT COUNT(*) FROM accounts");
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let error = save(&env, &owner, &buyer_request("01-11-2026", "5,000")).unwrap_err();
    let ServiceError::Rejected(items) = &error else {
        panic!("{error:?}");
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].code, ErrorCode::ErrDateInFuture);
    assert_eq!(items[0].field.as_deref(), Some("opening_date"));
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM accounts"),
        accounts_before
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
    assert_eq!(closed_day_audits(&dir), 0);
}

/// A saved buyer with 5,000 on 30-10-26, then the days up to `closed_up_to` closed, and the form
/// that changes his opening to 6,000 on `date`.
fn changed_opening_request(
    closed_up_to: &str,
    date: &str,
) -> (TempDir, TestEnv, auth::Actor, Value) {
    let (dir, env, owner) = world("2026-10-29");
    let tariq = save(&env, &owner, &buyer_request("30-10-2026", "5,000")).unwrap();
    raw(&dir)
        .execute(
            "UPDATE day_state SET closed_up_to_date = ?1 WHERE id = 1",
            [closed_up_to],
        )
        .unwrap();
    let mut change = buyer_request(date, "6,000");
    change["id"] = json!(tariq.id);
    change["version"] = json!(tariq.version);
    (dir, env, owner, change)
}

#[test]
fn a_changed_opening_on_a_closed_date_is_refused_with_what_the_form_needs() {
    // Only a NEW account is dated on a closed day. Changing the opening of an existing account
    // still needs an open date, and the refusal carries what the form needs (plan 53 T1).
    let (dir, env, owner, request) = changed_opening_request("2026-10-30", "30-10-2026");
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let error = save(&env, &owner, &request).unwrap_err();
    let ServiceError::Rejected(items) = &error else {
        panic!("{error:?}");
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].code, ErrorCode::ErrDateClosed);
    assert_eq!(items[0].field.as_deref(), Some("opening_date"));
    assert_eq!(
        text_value(&error, "closed_up_to").as_deref(),
        Some("30-10-2026")
    );
    assert_eq!(text_value(&error, "today").as_deref(), Some("31-10-2026"));
    assert_eq!(number_value(&error, "today_open"), Some(1));
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
    assert_eq!(closed_day_audits(&dir), 0);
    pair("accounts_save_opening_date_closed", &request, &error);

    // "Use today": the same change dated today saves; the old opening is reversed on its own day,
    // so that day must be open too, which it is not: the Owner reopens it first (FR-21.2).
    let mut today = request.clone();
    today["opening_date"] = json!("31-10-2026");
    let error = save(&env, &owner, &today).unwrap_err();
    assert!(matches!(&error, ServiceError::Rejected(items)
        if items[0].code == ErrorCode::ErrDateClosed));
}

#[test]
fn when_today_is_closed_too_the_refusal_of_a_changed_opening_says_so() {
    let (dir, env, owner, request) = changed_opening_request("2026-10-31", "31-10-2026");
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let error = save(&env, &owner, &request).unwrap_err();
    let ServiceError::Rejected(items) = &error else {
        panic!("{error:?}");
    };
    assert_eq!(items[0].code, ErrorCode::ErrDateClosed);
    assert_eq!(items[0].field.as_deref(), Some("opening_date"));
    assert_eq!(
        text_value(&error, "closed_up_to").as_deref(),
        Some("31-10-2026")
    );
    assert_eq!(text_value(&error, "today").as_deref(), Some("31-10-2026"));
    assert_eq!(number_value(&error, "today_open"), Some(0));
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
    pair("accounts_save_opening_date_today_closed", &request, &error);
}

#[test]
fn an_unchanged_opening_is_never_checked_again() {
    // The opening was posted on 30-10-26, then the day was closed. A new phone saves.
    let (dir, env, owner) = world("2026-10-29");
    let tariq = save(&env, &owner, &buyer_request("30-10-2026", "5,000")).unwrap();
    raw(&dir)
        .execute(
            "UPDATE day_state SET closed_up_to_date = '2026-10-30' WHERE id = 1",
            [],
        )
        .unwrap();
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");

    let mut edit = buyer_request("30-10-2026", "5000");
    edit["id"] = json!(tariq.id);
    edit["version"] = json!(tariq.version);
    edit["phone1"] = json!("0300-7654321");
    let edited = save(&env, &owner, &edit).unwrap();
    assert_eq!(edited.phone1, "03007654321");
    assert_eq!(edited.opening_dr_paisa, Paisa::from_rupees(5_000).unwrap());
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);

    // A changed amount on the closed date is refused, with the values for the form.
    let mut change = edit.clone();
    change["version"] = json!(edited.version);
    change["opening_dr"] = json!("6000");
    let error = save(&env, &owner, &change).unwrap_err();
    assert!(matches!(&error, ServiceError::Rejected(items)
        if items[0].code == ErrorCode::ErrDateClosed
            && items[0].field.as_deref() == Some("opening_date")));
    assert_eq!(number_value(&error, "today_open"), Some(1));
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
}
