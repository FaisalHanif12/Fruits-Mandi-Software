//! Accounts and the chart (plan 07, PRD 10.1, 10.2, FR-10.3, FR-10.6, FR-10.9, acceptance D2-A1 to
//! D2-A4): codes, edits, inactive accounts, the list with its footer, the duplicate warning and
//! the opening balances as balanced journal entries against ابتدائی فرق.
#![cfg(test)]

mod common;

use common::*;
use mandi_services::core::accounts::{
    AccountFilter, AccountHit, AccountInput, AccountListKind, AccountRow, AccountScope,
    AccountType, AccountsListInput, AccountsSimilarInput, BalanceSide, OpeningLineInput,
    OpeningsInput, SearchAccountsInput, SimilarAccount,
};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::money::Paisa;
use mandi_services::core::search::SearchScope;
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, auth, coa, ledger, search};
use std::time::Instant;
use tempfile::TempDir;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn input(kind: AccountType, code: &str, ur: &str, en: &str) -> AccountInput {
    AccountInput {
        id: None,
        version: None,
        account_type: kind,
        control_id: None,
        code: code.into(),
        name_ur: ur.into(),
        name_en: en.into(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    }
}

/// Saves an account, confirming whatever spelling the check asks about (most tests here are not
/// about the spelling check).
fn save(env: &TestEnv, owner: &auth::Actor, mut input: AccountInput) -> AccountRow {
    loop {
        match accounts::save(&env.ctx, owner, &input) {
            Ok(row) => return row,
            Err(ServiceError::Rejected(items))
                if items
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
            Err(other) => panic!("{other:?}"),
        }
    }
}

fn first_error(error: ServiceError) -> (ErrorCode, Option<String>) {
    match error {
        ServiceError::Rejected(items) => (items[0].code, items[0].field.clone()),
        other => panic!("not a rejection: {other:?}"),
    }
}

fn seller(env: &TestEnv, owner: &auth::Actor, ur: &str, en: &str) -> AccountRow {
    save(env, owner, input(AccountType::Seller, "", ur, en))
}

fn buyer(env: &TestEnv, owner: &auth::Actor, ur: &str, en: &str) -> AccountRow {
    save(env, owner, input(AccountType::Buyer, "", ur, en))
}

/// Every account of the chart with the code of its control, read the way the chart screen reads
/// it: the tree for the controls, then each control a page at a time (plan 38 B1).
fn chart_accounts(env: &TestEnv) -> Vec<(String, mandi_services::core::accounts::AccountNode)> {
    let tree = coa::tree(&env.ctx).unwrap();
    let mut out = Vec::new();
    for control in tree.heads.iter().flat_map(|h| &h.controls) {
        let mut offset = 0;
        loop {
            let page = coa::control_accounts(
                &env.ctx,
                &mandi_services::core::accounts::CoaControlAccountsInput {
                    control_id: control.id,
                    offset,
                    limit: 25,
                },
            )
            .unwrap();
            let got = u32::try_from(page.rows.len()).unwrap();
            out.extend(page.rows.into_iter().map(|a| (control.code.clone(), a)));
            offset += got;
            if got == 0 || offset >= page.total_count {
                break;
            }
        }
    }
    out
}

fn account_id(env: &TestEnv, code: &str) -> i64 {
    env.ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, code))
        .unwrap()
        .unwrap_or_else(|| panic!("no account {code}"))
}

/// One grid line for the account, with the version it has now.
fn line(env: &TestEnv, id: i64, date: &str, dr: &str, cr: &str, advance: bool) -> OpeningLineInput {
    OpeningLineInput {
        account_id: id,
        version: accounts::get(&env.ctx, id).unwrap().version,
        opening_date: date.into(),
        opening_dr: dr.into(),
        opening_cr: cr.into(),
        opening_is_advance: advance,
    }
}

fn count(dir: &TempDir, sql: &str) -> i64 {
    raw(dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

fn difference(env: &TestEnv) -> (BalanceSide, Paisa) {
    let id = account_id(env, "06019999");
    let balance = ledger::account_balance(&env.ctx, id).unwrap();
    (balance.side, balance.amount_paisa)
}

/// The three search columns and the tag of one account, as stored.
fn stored_keys(dir: &TempDir, id: i64) -> (String, String, String, Option<String>) {
    raw(dir)
        .query_row(
            "SELECT search_ur, search_en, search_phonetic, tag FROM accounts WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap()
}

#[test]
fn the_tag_is_checked_stored_once_and_never_inside_a_name() {
    // PRD 10.2, Code Standards 4.2: "k" is saved as "K"; "1A", "ABCDE", "ب" and "K-" are refused.
    let (dir, env, owner) = ready();
    for (typed, saved) in [("K", "K"), ("BA", "BA"), ("D.P", "D.P"), ("k", "K")] {
        let mut form = input(AccountType::Buyer, "", "طارق", "Tariq");
        form.tag = typed.into();
        let row = save(&env, &owner, form);
        assert_eq!(row.tag.as_deref(), Some(saved), "{typed}");
        // Stored once, in its own column: never inside either name.
        assert_eq!(
            (row.name_ur.as_str(), row.name_en.as_str()),
            ("طارق", "Tariq")
        );
        assert_eq!(stored_keys(&dir, row.id).3.as_deref(), Some(saved));
    }
    let before = count(&dir, "SELECT COUNT(*) FROM accounts");
    for bad in ["1A", "ABCDE", "ب", "K-"] {
        let mut form = input(AccountType::Buyer, "", "طارق", "Tariq");
        form.tag = bad.into();
        let error = accounts::save(&env.ctx, &owner, &form).unwrap_err();
        assert_eq!(
            first_error(error),
            (ErrorCode::ErrTagInvalid, Some("tag".to_owned())),
            "{bad}"
        );
    }
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM accounts"),
        before,
        "nothing written"
    );
    // No tag: the row has none, and the JSON has no `tag` key at all.
    let plain = save(&env, &owner, input(AccountType::Buyer, "", "طارق", "Tariq"));
    assert_eq!(plain.tag, None);
    assert!(serde_json::to_value(&plain).unwrap().get("tag").is_none());
    assert_eq!(stored_keys(&dir, plain.id).3, None);
}

#[test]
fn the_tag_is_one_more_word_in_the_three_keys_and_an_untagged_account_keeps_its_keys() {
    let (dir, env, owner) = ready();
    let mut form = input(AccountType::Buyer, "", "عاشق ظفر", "Ashiq Zafar");
    form.tag = "k".into();
    let tagged = save(&env, &owner, form);
    assert_eq!(
        stored_keys(&dir, tagged.id),
        (
            "عاشق ظفر k".to_owned(),
            "ashiq zafar k".to_owned(),
            "ashqzfrk".to_owned(),
            Some("K".to_owned())
        )
    );
    // An account with no tag has exactly the keys `core::text` makes from its names.
    let plain = buyer(&env, &owner, "عاشق ظفر", "Ashiq Zafar");
    let (ur, en, ph, tag) = stored_keys(&dir, plain.id);
    assert_eq!(
        (ur, en, ph, tag),
        (
            mandi_services::core::text::search_ur("عاشق ظفر"),
            mandi_services::core::text::search_en("Ashiq Zafar"),
            mandi_services::core::text::search_phonetic("Ashiq Zafar"),
            None
        )
    );
}

#[test]
fn ashiq_k_finds_the_tagged_account_in_both_scripts_and_the_hit_carries_the_tag() {
    // PRD FR-7.40: "ashiq k" finds عاشق ظفر K.
    let (_dir, env, owner) = ready();
    let mut form = input(AccountType::Buyer, "", "عاشق ظفر", "Ashiq Zafar");
    form.tag = "K".into();
    let tagged = save(&env, &owner, form);
    let plain = buyer(&env, &owner, "عاشق ظفر", "Ashiq Zafar");
    let kamran = buyer(&env, &owner, "عاشق کامران", "Ashiq Kamran");
    let find = |text: &str| -> Vec<AccountHit> {
        search::search_accounts(
            &env.ctx,
            &owner,
            &SearchAccountsInput {
                include_dead: None,
                scope: AccountScope::Any,
                control_id: None,
                text: text.into(),
                limit: 20,
                offset: 0,
            },
        )
        .unwrap()
    };
    for typed in [
        "ashiq k",
        "ASHIQ K",
        "ashiq zafar k",
        "عاشق k",
        "عاشق ظفر k",
    ] {
        let hits = find(typed);
        let hit = hits
            .iter()
            .find(|h| h.id == tagged.id)
            .unwrap_or_else(|| panic!("{typed} did not find the tagged account: {hits:?}"));
        assert_eq!(hit.tag.as_deref(), Some("K"), "{typed}");
    }
    // The tag narrows the search: the same name with no tag is not an "ashiq zafar k".
    let ids: Vec<i64> = find("ashiq zafar k").iter().map(|h| h.id).collect();
    assert_eq!(ids, vec![tagged.id]);
    // "ashiq" alone still finds all three, and only the tagged one carries a tag.
    let all = find("ashiq");
    for id in [tagged.id, plain.id, kamran.id] {
        assert!(all.iter().any(|h| h.id == id));
    }
    assert!(
        all.iter()
            .filter(|h| h.id != tagged.id)
            .all(|h| h.tag.is_none())
    );
}

#[test]
fn the_duplicate_check_compares_name_tag_and_phone() {
    // PRD FR-10.7, E38: اسلم BA and اسلم are two different people.
    let (_dir, env, owner) = ready();
    let mut with_tag = input(AccountType::Buyer, "", "اسلم", "Aslam");
    with_tag.phone1 = "0300-1111111".into();
    with_tag.tag = "BA".into();
    let ba = save(&env, &owner, with_tag);
    let mut without = input(AccountType::Buyer, "", "اسلم", "Aslam");
    without.phone1 = "0300-1111111".into();
    let none = save(&env, &owner, without);
    let ask = |tag: &str, phone: &str| {
        accounts::similar(
            &env.ctx,
            &AccountsSimilarInput {
                name_ur: "اسلم".into(),
                name_en: "".into(),
                phone: phone.into(),
                tag: tag.into(),
                exclude_id: None,
            },
        )
        .unwrap()
    };
    let flags = |found: &[SimilarAccount]| -> Vec<(i64, bool)> {
        let mut v: Vec<_> = found.iter().map(|a| (a.id, a.is_duplicate)).collect();
        v.sort();
        v
    };
    // Same name, tag and phone: that one is the duplicate; the other is only listed as similar.
    let found = ask("BA", "03001111111");
    assert_eq!(found.len(), 2, "both are listed");
    assert_eq!(flags(&found).iter().filter(|(_, d)| *d).count(), 1);
    assert!(
        found[0].is_duplicate && found[0].id == ba.id,
        "a duplicate comes first"
    );
    assert_eq!(found[0].tag.as_deref(), Some("BA"));
    // A lower-case tag is the same tag.
    assert!(ask("ba", "0300-1111111")[0].is_duplicate);
    // No tag typed: the account with no tag is the duplicate.
    let found = ask("", "0300-1111111");
    assert!(found[0].is_duplicate && found[0].id == none.id);
    assert_eq!(found.iter().filter(|a| a.is_duplicate).count(), 1);
    // Another tag, or another phone: nothing is a duplicate (same name and phone, different tag).
    assert!(ask("K", "0300-1111111").iter().all(|a| !a.is_duplicate));
    assert!(ask("BA", "0300-2222222").iter().all(|a| !a.is_duplicate));
    // A tag that is not valid yet marks nothing, and the list is still there.
    let found = ask("1A", "0300-1111111");
    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|a| !a.is_duplicate));
}

#[test]
fn an_edit_changes_the_tag_and_the_keys_and_the_search_follows() {
    let (dir, env, owner) = ready();
    let row = buyer(&env, &owner, "اسلم", "Aslam");
    let edit = |tag: &str| {
        let current = accounts::get(&env.ctx, row.id).unwrap();
        let mut form = input(AccountType::Buyer, &current.code, "اسلم", "Aslam");
        form.id = Some(current.id);
        form.version = Some(current.version);
        form.tag = tag.into();
        save(&env, &owner, form)
    };
    let find = |text: &str| {
        search::search_accounts(
            &env.ctx,
            &owner,
            &SearchAccountsInput {
                include_dead: None,
                scope: AccountScope::Buyer,
                control_id: None,
                text: text.into(),
                limit: 5,
                offset: 0,
            },
        )
        .unwrap()
    };
    assert!(find("aslam ba").is_empty() || find("aslam ba").iter().all(|h| h.tag.is_none()));
    let tagged = edit("ba");
    assert_eq!(tagged.tag.as_deref(), Some("BA"));
    assert_eq!(stored_keys(&dir, row.id).1, "aslam ba");
    assert_eq!(find("aslam ba")[0].tag.as_deref(), Some("BA"));
    // The audit row of the edit holds the old and the new tag (Code Standards S7).
    let (old, new): (String, String) = raw(&dir)
        .query_row(
            "SELECT old_json, new_json FROM audit_log WHERE entity = 'account' AND action = 'edit' AND entity_id = ?1 ORDER BY id DESC LIMIT 1",
            [row.id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(!old.contains("\"tag\""), "no tag before: {old}");
    assert!(new.contains("\"tag\":\"BA\""), "{new}");
    // Clearing the tag gives the old keys back.
    let cleared = edit("");
    assert_eq!(cleared.tag, None);
    assert_eq!(
        stored_keys(&dir, row.id),
        (
            mandi_services::core::text::search_ur("اسلم"),
            mandi_services::core::text::search_en("Aslam"),
            mandi_services::core::text::search_phonetic("Aslam"),
            None
        )
    );
    assert!(find("aslam ba").iter().all(|h| h.tag.is_none()));
}

#[test]
fn the_tag_shows_in_the_row_the_party_card_and_the_chart() {
    let (_dir, env, owner) = ready();
    let mut form = input(AccountType::Buyer, "", "اکرم ٹول ٹیکس", "Akram Toll Tax");
    form.tag = "gm".into();
    let row = save(&env, &owner, form);
    assert_eq!(
        accounts::get(&env.ctx, row.id).unwrap().tag.as_deref(),
        Some("GM")
    );
    assert_eq!(
        mandi_services::assist::party_card(&env.ctx, row.id)
            .unwrap()
            .tag
            .as_deref(),
        Some("GM")
    );
    let node = chart_accounts(&env)
        .into_iter()
        .map(|(_, a)| a)
        .find(|a| a.id == row.id)
        .unwrap();
    assert_eq!(node.tag.as_deref(), Some("GM"));
    // A linked account carries its own tag in the row that points at it.
    let mut other = input(AccountType::Seller, "", "اکرم", "Akram");
    other.linked_account_id = Some(row.id);
    let linked = save(&env, &owner, other);
    assert_eq!(
        linked
            .linked_account
            .as_ref()
            .and_then(|l| l.tag.as_deref()),
        Some("GM")
    );
}

#[test]
fn codes_are_the_type_number_and_a_running_number_from_zero() {
    // Plan 32 R1, R2 (PRD 1.8): seller 001, buyer 002, then the running number from 0 with no
    // padding. The two پختہ کھاتہ accounts are 0030 and 0031 and نقدی منڈی keeps 03040000, so
    // none of them is in the way: the first buyer is 0020.
    let (_dir, env, owner) = ready();
    let rahim = seller(&env, &owner, "رحیم", "Rahim");
    let karim = seller(&env, &owner, "کریم", "Karim");
    let akram = buyer(&env, &owner, "اکرم ٹول ٹیکس", "Akram Toll Tax");
    let arshad = buyer(&env, &owner, "ارشد ملتانی", "Arshad Multani");
    assert_eq!(
        (rahim.code.as_str(), rahim.serial),
        ("0010", 0),
        "seller codes are 001 plus the running number; the serial is the running number"
    );
    assert_eq!((karim.code.as_str(), karim.serial), ("0011", 1));
    assert_eq!(akram.code, "0020");
    assert_eq!(arshad.code, "0021");
    assert_eq!(rahim.account_type, AccountType::Seller);
    assert_eq!(akram.account_type, AccountType::Buyer);
    assert!(!akram.is_system && akram.is_active && akram.credit_allowed);
    // The next code the form suggests.
    assert_eq!(
        accounts::next_code(&env.ctx, AccountType::Buyer, None).unwrap(),
        "0022"
    );
    assert_eq!(
        accounts::next_code(&env.ctx, AccountType::Seller, None).unwrap(),
        "0012"
    );
    assert_eq!(
        accounts::next_code(&env.ctx, AccountType::MoneyLent, None).unwrap(),
        "01030001"
    );
    // The type "other" needs a control, and only an allowed one.
    let (code, field) =
        first_error(accounts::next_code(&env.ctx, AccountType::Other, None).unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrRequired, Some("control_id"))
    );
    let controls = coa::controls(&env.ctx).unwrap();
    let cash = controls.iter().find(|c| c.code == "0101").unwrap();
    assert!(cash.allowed_for_other);
    let money_lent = controls.iter().find(|c| c.code == "0103").unwrap();
    assert!(!money_lent.allowed_for_other, "0103 has its own type");
    assert_eq!(
        accounts::next_code(&env.ctx, AccountType::Other, Some(cash.id)).unwrap(),
        "01010002"
    );
    let (code, field) = first_error(
        accounts::next_code(&env.ctx, AccountType::Other, Some(money_lent.id)).unwrap_err(),
    );
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrFormatInvalid, Some("control_id"))
    );
}

#[test]
fn a_typed_code_must_be_free_and_a_full_code_of_the_type() {
    // Plan 32 R2: the code box is filled by itself and stays editable to a free code of the same
    // type. A buyer's code is 002 plus a running number with no padding.
    let (dir, env, owner) = ready();
    let first = save(
        &env,
        &owner,
        input(AccountType::Buyer, "0027", "ناصر", "Nasir"),
    );
    assert_eq!((first.code.as_str(), first.serial), ("0027", 7));
    // The suggestion goes on from the highest running number, so no code is used twice.
    assert_eq!(
        accounts::next_code(&env.ctx, AccountType::Buyer, None).unwrap(),
        "0028"
    );
    assert_eq!(
        accounts::next_code(&env.ctx, AccountType::Seller, None).unwrap(),
        "0010",
        "sellers have their own numbers"
    );
    let written = count(&dir, "SELECT COUNT(*) FROM accounts");
    for (typed, wanted) in [
        ("0027", ErrorCode::ErrCodeTaken),
        // A code of another type, of the system, padded, without a running number, or the old form.
        ("0010", ErrorCode::ErrCodeInvalid),
        ("0030", ErrorCode::ErrCodeInvalid),
        ("03040000", ErrorCode::ErrCodeInvalid),
        ("03040007", ErrorCode::ErrCodeInvalid),
        ("00201", ErrorCode::ErrCodeInvalid),
        ("002", ErrorCode::ErrCodeInvalid),
        ("0020x", ErrorCode::ErrCodeInvalid),
    ] {
        let error = accounts::save(
            &env.ctx,
            &owner,
            &input(AccountType::Buyer, typed, "ناصر", "Nasir"),
        )
        .unwrap_err();
        let (code, field) = first_error(error);
        assert_eq!(
            (code, field.as_deref()),
            (wanted, Some("code")),
            "typed code {typed}"
        );
    }
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM accounts"),
        written,
        "a refused save writes nothing"
    );
    // A free code of the type is taken as typed, the lowest one included.
    let low = save(
        &env,
        &owner,
        input(AccountType::Buyer, "0020", "فرید", "Farid"),
    );
    assert_eq!((low.code.as_str(), low.serial), ("0020", 0));
}

#[test]
fn the_eleventh_seller_is_00110_and_a_code_is_never_reused() {
    // Plan 32 Section 3: first seller 0010, second 0011, eleventh 00110. The next code is the
    // highest running number plus one, so it also moves on after a hand-typed code.
    let (_dir, env, owner) = ready();
    let mut codes = Vec::new();
    for n in 0..11 {
        let row = seller(
            &env,
            &owner,
            &format!("بیوپاری {n}"),
            &format!("Seller {n}"),
        );
        codes.push(row.code);
    }
    assert_eq!(codes[0], "0010");
    assert_eq!(codes[1], "0011");
    assert_eq!(codes[9], "0019");
    assert_eq!(codes[10], "00110");
    assert_eq!(
        accounts::next_code(&env.ctx, AccountType::Seller, None).unwrap(),
        "00111"
    );
    // A hand-typed code far ahead moves the suggestion past it.
    save(
        &env,
        &owner,
        input(AccountType::Seller, "00150", "دور", "Dur"),
    );
    assert_eq!(
        accounts::next_code(&env.ctx, AccountType::Seller, None).unwrap(),
        "00151"
    );
    // The accounts list shows the sellers in running-number order: 0019 before 00110.
    let list = accounts::list(
        &env.ctx,
        &AccountsListInput {
            sort: None,
            filter: AccountFilter {
                kind: AccountListKind::Seller,
                text: String::new(),
                include_inactive: false,
                has_opening: None,
            },
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let listed: Vec<&str> = list.rows.iter().map(|r| r.code.as_str()).collect();
    assert_eq!(
        &listed[8..],
        ["0018", "0019", "00110", "00150"],
        "numbers, not text, decide the order"
    );
}

#[test]
fn the_numbers_of_a_type_can_run_out_with_a_clear_code() {
    // The running number has at most 7 digits (a code has at most 10). A buyer takes the last one by
    // typing it; the next suggestion then has nowhere to go (ERR_SERIES_FULL).
    let (_dir, env, owner) = ready();
    let last = save(
        &env,
        &owner,
        input(AccountType::Buyer, "0029999999", "آخری", "Akhri"),
    );
    assert_eq!((last.code.as_str(), last.serial), ("0029999999", 9_999_999));
    let (code, _) =
        first_error(accounts::next_code(&env.ctx, AccountType::Buyer, None).unwrap_err());
    assert_eq!(code, ErrorCode::ErrSeriesFull);
    let (code, field) = first_error(
        accounts::save(
            &env.ctx,
            &owner,
            &input(AccountType::Buyer, "", "اور", "Aur"),
        )
        .unwrap_err(),
    );
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrSeriesFull, Some("code"))
    );
    // Sellers are not affected.
    assert_eq!(
        accounts::next_code(&env.ctx, AccountType::Seller, None).unwrap(),
        "0010"
    );
}

#[test]
fn an_edit_keeps_the_code_checks_the_version_and_refreshes_search() {
    let (dir, env, owner) = ready();
    let row = buyer(&env, &owner, "اکرم ٹول ٹیکس", "Akram Toll Tax");
    let mut edit = input(
        AccountType::Buyer,
        &row.code,
        "اکرم ٹول ٹیکس",
        "Akram Toll Tax",
    );
    edit.id = Some(row.id);
    edit.version = Some(row.version);
    edit.phone1 = "0300-1234567".into();
    edit.note = "نیا نوٹ".into();
    let saved = save(&env, &owner, edit.clone());
    assert_eq!(saved.phone1, "03001234567");
    assert_eq!(saved.version, row.version + 1);
    assert_eq!(saved.code, row.code);
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'account' AND action = 'edit'"
        ),
        1
    );

    // The old version is refused.
    let (code, _) = first_error(accounts::save(&env.ctx, &owner, &edit).unwrap_err());
    assert_eq!(code, ErrorCode::ErrVersionChanged);

    // The code and the type cannot change.
    let mut again = edit.clone();
    again.version = Some(saved.version);
    again.code = "0029".into();
    let (code, field) = first_error(accounts::save(&env.ctx, &owner, &again).unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrFormatInvalid, Some("code"))
    );
    let mut again = edit.clone();
    again.version = Some(saved.version);
    again.account_type = AccountType::Seller;
    let (code, field) = first_error(accounts::save(&env.ctx, &owner, &again).unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrFormatInvalid, Some("account_type"))
    );

    // A new name is found at once (the index is refreshed after the commit).
    let mut rename = edit.clone();
    rename.version = Some(saved.version);
    rename.name_ur = "اکرم خان".into();
    rename.name_en = "Akram Khan".into();
    let renamed = save(&env, &owner, rename);
    let hits = search::search(&env.ctx, &owner, SearchScope::Buyer, "khan", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, renamed.id);
    assert!(
        search::search(&env.ctx, &owner, SearchScope::Buyer, "toll", 10)
            .unwrap()
            .is_empty()
    );
    assert_chain_ok(&dir);
}

#[test]
fn an_account_is_made_inactive_never_deleted() {
    // PRD FR-10.6: no delete command. Inactive accounts leave the pickers; reports keep them.
    let (dir, env, owner) = ready();
    let row = buyer(&env, &owner, "ناصر", "Nasir");
    let off = accounts::set_active(&env.ctx, &owner, row.id, false).unwrap();
    assert!(!off.is_active);
    assert!(
        search::search(&env.ctx, &owner, SearchScope::Buyer, "nasir", 10)
            .unwrap()
            .is_empty(),
        "an inactive account is not offered"
    );
    // Asking for the state it already has writes nothing.
    let audit_before = audit_total(&dir);
    accounts::set_active(&env.ctx, &owner, row.id, false).unwrap();
    assert_eq!(audit_total(&dir), audit_before);
    let on = accounts::set_active(&env.ctx, &owner, row.id, true).unwrap();
    assert!(on.is_active);
    assert_eq!(
        search::search(&env.ctx, &owner, SearchScope::Buyer, "nasir", 10)
            .unwrap()
            .len(),
        1
    );
    let (code, field) =
        first_error(accounts::set_active(&env.ctx, &owner, 999_999, false).unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrNotFound, Some("id"))
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM accounts WHERE name_en = 'Nasir'"
        ),
        1
    );
}

#[test]
fn a_system_account_cannot_be_made_inactive() {
    // PRD FR-10.4: the app posts to these by itself (cash, opening difference, the income
    // accounts, نقدی منڈی, the two پختہ کھاتہ accounts ...), so a picker must always offer them.
    // The lead decided on 13-10-2026: refused with a code on `id`; the screen shows the Active
    // box ticked and cannot be changed.
    let (dir, env, owner) = ready();
    let system: Vec<(i64, String)> = {
        let conn = raw(&dir);
        let mut statement = conn
            .prepare("SELECT id, code FROM accounts WHERE is_system = 1 ORDER BY code")
            .unwrap();
        statement
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    for code in [
        "01010001", "06019999", "04010001", "03040000", "0030", "0031",
    ] {
        assert!(
            system.iter().any(|(_, c)| c == code),
            "{code} should be a system account"
        );
    }
    let audit_before = audit_total(&dir);
    for (id, code) in &system {
        let (error_code, field) =
            first_error(accounts::set_active(&env.ctx, &owner, *id, false).unwrap_err());
        assert_eq!(
            (error_code, field.as_deref()),
            (ErrorCode::ErrSystemAccount, Some("id")),
            "account {code}"
        );
        assert!(accounts::get(&env.ctx, *id).unwrap().is_active, "{code}");
        // Asking for active again is a no-op, not an error.
        assert!(
            accounts::set_active(&env.ctx, &owner, *id, true)
                .unwrap()
                .is_active
        );
    }
    assert_eq!(audit_total(&dir), audit_before, "a refusal writes nothing");
    // The Active box of the account form is held to the same rule.
    let cash = accounts::get(&env.ctx, account_id(&env, "01010001")).unwrap();
    let mut edit = form_of(&cash);
    edit.is_active = false;
    let (error_code, field) = first_error(accounts::save(&env.ctx, &owner, &edit).unwrap_err());
    assert_eq!(
        (error_code, field.as_deref()),
        (ErrorCode::ErrSystemAccount, Some("is_active"))
    );
    assert!(accounts::get(&env.ctx, cash.id).unwrap().is_active);
    assert_eq!(audit_total(&dir), audit_before);
    // An ordinary account is still switched off and on.
    let row = buyer(&env, &owner, "ناصر", "Nasir");
    let audit_before = audit_total(&dir);
    assert!(
        !accounts::set_active(&env.ctx, &owner, row.id, false)
            .unwrap()
            .is_active
    );
    assert_eq!(audit_total(&dir), audit_before + 1);
}

#[test]
fn the_list_has_the_columns_and_the_footer_of_fr_10_5() {
    let (_dir, env, owner) = ready();
    let rahim = seller(&env, &owner, "رحیم", "Rahim");
    let akram = buyer(&env, &owner, "اکرم ٹول ٹیکس", "Akram Toll Tax");
    buyer(&env, &owner, "ارشد ملتانی", "Arshad Multani");
    accounts::set_active(&env.ctx, &owner, akram.id, false).unwrap();
    // Openings: Rahim 15,000 debit, Arshad 3,000 credit (the date is open: nothing is closed).
    let arshad = account_id(&env, "0021");
    let grid = OpeningsInput {
        lines: vec![
            line(&env, rahim.id, "31-08-2026", "15,000", "", true),
            line(&env, arshad, "31-08-2026", "", "3000", false),
        ],
    };
    accounts::openings_save(&env.ctx, &owner, &grid).unwrap();

    let list = |text: &str, kind: AccountListKind, inactive: bool| {
        accounts::list(
            &env.ctx,
            &AccountsListInput {
                sort: None,
                filter: AccountFilter {
                    kind,
                    text: text.into(),
                    include_inactive: inactive,
                    has_opening: None,
                },
                offset: 0,
                limit: 0,
            },
        )
        .unwrap()
    };
    let buyers = list("", AccountListKind::Buyer, false);
    let codes: Vec<&str> = buyers.rows.iter().map(|r| r.code.as_str()).collect();
    assert!(codes.contains(&"0021") && !codes.contains(&"0020"));
    let with_inactive = list("", AccountListKind::Buyer, true);
    assert!(with_inactive.rows.iter().any(|r| r.code == "0020"));
    assert_eq!(
        with_inactive.count,
        u32::try_from(with_inactive.rows.len()).unwrap()
    );

    // Footer: the opening debit and credit of all matching accounts.
    let sellers = list("", AccountListKind::Seller, false);
    assert_eq!(sellers.total_dr_paisa, rupees(15_000));
    assert_eq!(sellers.total_cr_paisa, rupees(0));
    assert_eq!(buyers.total_cr_paisa, rupees(3_000));
    // Balances come from the journal.
    let row = sellers.rows.iter().find(|r| r.id == rahim.id).unwrap();
    assert_eq!(
        (row.balance.side, row.balance.amount_paisa),
        (BalanceSide::Dr, rupees(15_000))
    );
    let row = buyers.rows.iter().find(|r| r.id == arshad).unwrap();
    assert_eq!(
        (row.balance.side, row.balance.amount_paisa),
        (BalanceSide::Cr, rupees(3_000))
    );
    // A text filter finds by code or by a name in either script.
    assert_eq!(list("arshad", AccountListKind::All, false).rows.len(), 1);
    assert_eq!(list("ارشد", AccountListKind::All, false).rows.len(), 1);
    assert_eq!(list("0021", AccountListKind::All, false).rows.len(), 1);
}

#[test]
fn an_opening_is_a_balanced_entry_against_the_opening_difference() {
    // PRD Section 4.3 and E2E-19, backend part: cash 65,000 debit and capital 65,000 credit on
    // 31-08-26 leave ابتدائی فرق at 0 and the trial balance 65,000 = 65,000.
    let (dir, env, owner) = ready();
    let cash = account_id(&env, "01010001");
    let capital = account_id(&env, "06010001");
    let grid = OpeningsInput {
        lines: vec![
            line(&env, cash, "31-08-2026", "65,000", "", false),
            line(&env, capital, "31-08-2026", "", "65000", false),
        ],
    };

    // The preview and the save agree (one plan function).
    let preview = accounts::openings_preview(&env.ctx, &grid).unwrap();
    assert!(preview.rows.iter().all(|r| r.errors.is_empty()));
    assert_eq!(preview.total_dr_paisa, rupees(65_000));
    assert_eq!(preview.total_cr_paisa, rupees(65_000));
    assert_eq!(preview.opening_difference_after.side, BalanceSide::Zero);
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM journal_entries"),
        0,
        "the preview writes nothing"
    );

    let saved = accounts::openings_save(&env.ctx, &owner, &grid).unwrap();
    assert_eq!(saved.rows.len(), 2);
    assert_eq!(saved.opening_difference.side, BalanceSide::Zero);
    assert_eq!(difference(&env), (BalanceSide::Zero, rupees(0)));

    let trial = ledger::trial_balance(&env.ctx, None).unwrap();
    assert_eq!(
        (trial.total_dr, trial.total_cr),
        (rupees(65_000), rupees(65_000))
    );
    let row = |code: &str| trial.rows.iter().find(|r| r.code == code).unwrap().balance;
    assert_eq!(
        (row("01010001").side, row("01010001").amount_paisa),
        (BalanceSide::Dr, rupees(65_000))
    );
    assert_eq!(
        (row("06010001").side, row("06010001").amount_paisa),
        (BalanceSide::Cr, rupees(65_000))
    );
    assert_eq!(row("06019999").side, BalanceSide::Zero);

    // Two posted opening entries, each balanced by its own line against ابتدائی فرق.
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM journal_entries WHERE posting_type = 'opening' AND status = 'posted' AND source_type = 'account'"
        ),
        2
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM journal_entries WHERE entry_date <> '2026-08-31'"
        ),
        0
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_lines"), 4);
    // The account rows keep the opening, the entry and the audit trail.
    let cash_row = accounts::get(&env.ctx, cash).unwrap();
    assert_eq!(cash_row.opening_date.as_deref(), Some("2026-08-31"));
    assert_eq!(cash_row.opening_dr_paisa, rupees(65_000));
    assert_eq!(cash_row.balance.amount_paisa, rupees(65_000));
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'account' AND action = 'edit'"
        ),
        2
    );
    ledger::verify_journal_chain(&env.ctx).unwrap();
    assert_chain_ok(&dir);

    // Saving the same grid again changes nothing: no new entries, no new audit rows.
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let audits = audit_total(&dir);
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![
                line(&env, cash, "31-08-2026", "65000", "", false),
                line(&env, capital, "31-08-2026", "", "65,000", false),
            ],
        },
    )
    .unwrap();
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
    assert_eq!(audit_total(&dir), audits);
}

#[test]
fn changing_an_opening_reverses_the_old_entry_and_posts_the_new_one() {
    // PRD FR-10.9: "Changing an opening reverses the old opening posting and posts the new one".
    let (dir, env, owner) = ready();
    let cash = account_id(&env, "01010001");
    let capital = account_id(&env, "06010001");
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![
                line(&env, cash, "31-08-2026", "65000", "", false),
                line(&env, capital, "31-08-2026", "", "65000", false),
            ],
        },
    )
    .unwrap();
    // The cash was typed wrong: it is 70,000. ابتدائی فرق now shows the 5,000 that is missing.
    let preview = accounts::openings_preview(
        &env.ctx,
        &OpeningsInput {
            lines: vec![line(&env, cash, "31-08-2026", "70000", "", false)],
        },
    )
    .unwrap();
    assert_eq!(
        (
            preview.opening_difference_after.side,
            preview.opening_difference_after.amount_paisa
        ),
        (BalanceSide::Cr, rupees(5_000))
    );
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(&env, cash, "31-08-2026", "70000", "", false)],
        },
    )
    .unwrap();
    assert_eq!(difference(&env), (BalanceSide::Cr, rupees(5_000)));
    assert_eq!(
        accounts::get(&env.ctx, cash).unwrap().balance.amount_paisa,
        rupees(70_000)
    );
    // One reversal entry, same date as the opening it reverses; old rows are untouched.
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM journal_entries WHERE posting_type = 'reversal' AND reverses_entry_id IS NOT NULL AND entry_date = '2026-08-31'"
        ),
        1
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM journal_entries WHERE posting_type = 'opening'"
        ),
        3
    );
    let trial = ledger::trial_balance(&env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);

    // The capital is corrected to 70,000 and the difference goes back to nothing.
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(&env, capital, "31-08-2026", "", "70000", false)],
        },
    )
    .unwrap();
    assert_eq!(difference(&env), (BalanceSide::Zero, rupees(0)));

    // Clearing an opening removes its effect and its date.
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![
                line(&env, cash, "", "", "", false),
                line(&env, capital, "", "", "", false),
            ],
        },
    )
    .unwrap();
    let row = accounts::get(&env.ctx, cash).unwrap();
    assert_eq!((row.opening_date, row.opening_dr_paisa), (None, rupees(0)));
    assert_eq!(row.balance.side, BalanceSide::Zero);
    assert_eq!(difference(&env), (BalanceSide::Zero, rupees(0)));
    ledger::verify_journal_chain(&env.ctx).unwrap();
}

#[test]
fn the_account_form_posts_its_opening_and_marks_a_seller_advance() {
    // Rahim's open advance of 15,000 (PRD Section 4.3), typed on his form.
    let (dir, env, owner) = ready();
    let mut form = input(AccountType::Seller, "", "رحیم", "Rahim");
    form.opening_date = "01-09-2026".into();
    form.opening_dr = "15,000".into();
    form.opening_is_advance = true;
    let rahim = save(&env, &owner, form);
    assert_eq!(rahim.opening_date.as_deref(), Some("2026-09-01"));
    assert_eq!(
        (rahim.balance.side, rahim.balance.amount_paisa),
        (BalanceSide::Dr, rupees(15_000))
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM journal_lines WHERE account_id = (SELECT id FROM accounts WHERE code = '0010') AND dr_paisa = 1500000 AND is_advance = 1"
        ),
        1
    );
    assert_eq!(difference(&env), (BalanceSide::Cr, rupees(15_000)));
    // Editing the form without touching the opening writes no new entry.
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let mut edit = input(AccountType::Seller, &rahim.code, "رحیم", "Rahim");
    edit.id = Some(rahim.id);
    edit.version = Some(rahim.version);
    edit.opening_date = "01-09-2026".into();
    edit.opening_dr = "15000".into();
    edit.opening_is_advance = true;
    edit.note = "اب نوٹ ہے".into();
    save(&env, &owner, edit);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
}

#[test]
fn opening_mistakes_name_the_field_and_write_nothing() {
    let (dir, env, owner) = ready();
    let written = count(&dir, "SELECT COUNT(*) FROM accounts");
    let try_save = |edit: &dyn Fn(&mut AccountInput)| {
        let mut form = input(AccountType::Buyer, "", "ناصر", "Nasir");
        edit(&mut form);
        first_error(accounts::save(&env.ctx, &owner, &form).unwrap_err())
    };
    // Both sides.
    let (code, field) = try_save(&|f| {
        f.opening_date = "31-08-2026".into();
        f.opening_dr = "100".into();
        f.opening_cr = "100".into();
    });
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrOpeningBothSides, Some("opening_cr"))
    );
    // A decimal amount (PRD E31).
    let (code, field) = try_save(&|f| {
        f.opening_date = "31-08-2026".into();
        f.opening_dr = "850.50".into();
    });
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrDecimalsNotAllowed, Some("opening_dr"))
    );
    // An advance only on a seller.
    let (code, field) = try_save(&|f| {
        f.opening_date = "31-08-2026".into();
        f.opening_dr = "100".into();
        f.opening_is_advance = true;
    });
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrAdvanceNotSeller, Some("opening_is_advance"))
    );
    // A date in the future (today is 10-10-2026 in the test clock).
    let (code, field) = try_save(&|f| {
        f.opening_date = "11-10-2026".into();
        f.opening_dr = "100".into();
    });
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrDateInFuture, Some("opening_date"))
    );
    // An amount without a date.
    let (code, field) = try_save(&|f| f.opening_dr = "100".into());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrRequired, Some("opening_date"))
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM accounts"), written);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), 0);
}

#[test]
fn a_closed_day_cannot_take_or_lose_an_opening() {
    // PRD FR-6.6: the opening date must be after the closed-up-to date. Reversing an opening
    // whose own day is closed is also refused (the Owner reopens the day first, FR-21.2).
    let (dir, env, owner) = ready();
    let cash = account_id(&env, "01010001");
    let capital = account_id(&env, "06010001");
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![
                line(&env, cash, "31-08-2026", "65000", "", false),
                line(&env, capital, "31-08-2026", "", "65000", false),
            ],
        },
    )
    .unwrap();
    raw(&dir)
        .execute(
            "UPDATE day_state SET closed_up_to_date = '2026-09-09' WHERE id = 1",
            [],
        )
        .unwrap();
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");

    // A new opening on a closed day.
    let akram = buyer(&env, &owner, "اکرم ٹول ٹیکس", "Akram Toll Tax");
    let error = accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(&env, akram.id, "01-09-2026", "500", "", false)],
        },
    )
    .unwrap_err();
    let ServiceError::Rejected(items) = error else {
        panic!("not a rejection")
    };
    assert_eq!(items[0].code, ErrorCode::ErrDateClosed);
    assert_eq!(items[0].field.as_deref(), Some("lines[0].opening_date"));
    // On the open day it works.
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(&env, akram.id, "10-09-2026", "500", "", false)],
        },
    )
    .unwrap();
    // Changing the cash opening, whose day (31-08) is closed, is refused and nothing moves.
    let before = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let error = accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(&env, cash, "10-09-2026", "70000", "", false)],
        },
    )
    .unwrap_err();
    let (code, _) = first_error(error);
    assert_eq!(code, ErrorCode::ErrDateClosed);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), before);
    assert!(before > entries);
    assert_eq!(
        accounts::get(&env.ctx, cash).unwrap().opening_dr_paisa,
        rupees(65_000)
    );
    ledger::verify_journal_chain(&env.ctx).unwrap();
}

#[test]
fn the_grid_saves_all_lines_or_none_and_points_at_each_bad_line() {
    let (dir, env, owner) = ready();
    let cash = account_id(&env, "01010001");
    let capital = account_id(&env, "06010001");
    let income = account_id(&env, "04010001");
    let akram = buyer(&env, &owner, "اکرم ٹول ٹیکس", "Akram Toll Tax");
    let bad = OpeningsInput {
        lines: vec![
            line(&env, cash, "31-08-2026", "65000", "", false),
            // An income account takes no opening (partner share rows would be needed, FR-6.5).
            line(&env, income, "31-08-2026", "", "100", false),
            // Both sides.
            line(&env, capital, "31-08-2026", "10", "10", false),
            // The same account twice, and a stale version.
            line(&env, akram.id, "31-08-2026", "1", "", false),
            OpeningLineInput {
                version: 99,
                ..line(&env, akram.id, "31-08-2026", "2", "", false)
            },
        ],
    };
    let preview = accounts::openings_preview(&env.ctx, &bad).unwrap();
    let errors: Vec<(usize, ErrorCode, String)> = preview
        .rows
        .iter()
        .enumerate()
        .flat_map(|(i, r)| r.errors.iter().map(move |e| (i, e.code, e.field.clone())))
        .collect();
    assert!(errors.contains(&(
        1,
        ErrorCode::ErrOpeningNotAllowed,
        "lines[1].opening_dr".into()
    )));
    assert!(errors.contains(&(
        2,
        ErrorCode::ErrOpeningBothSides,
        "lines[2].opening_cr".into()
    )));
    assert!(errors.contains(&(4, ErrorCode::ErrVersionChanged, "lines[4].version".into())));
    assert!(
        errors
            .iter()
            .any(|(i, c, _)| *i == 4 && *c == ErrorCode::ErrFormatInvalid),
        "duplicate account"
    );
    assert!(preview.rows[0].errors.is_empty());

    let error = accounts::openings_save(&env.ctx, &owner, &bad).unwrap_err();
    let ServiceError::Rejected(items) = error else {
        panic!("not a rejection")
    };
    assert!(items.len() >= 4);
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM journal_entries"),
        0,
        "all or nothing"
    );
    assert_eq!(
        accounts::get(&env.ctx, cash).unwrap().opening_dr_paisa,
        rupees(0)
    );

    // The same refusals on the single form: an opening on an income account.
    let error = accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(&env, income, "31-08-2026", "100", "", false)],
        },
    )
    .unwrap_err();
    assert_eq!(first_error(error).0, ErrorCode::ErrOpeningNotAllowed);
    // ابتدائی فرق itself and a head 05 account are refused too.
    let difference_id = account_id(&env, "06019999");
    let error = accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(&env, difference_id, "31-08-2026", "100", "", false)],
        },
    )
    .unwrap_err();
    assert_eq!(first_error(error).0, ErrorCode::ErrOpeningNotAllowed);
    // نقدی منڈی pays at once, so a balance there is refused too.
    let naqdi = account_id(&env, "03040000");
    let error = accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(&env, naqdi, "31-08-2026", "100", "", false)],
        },
    )
    .unwrap_err();
    assert_eq!(first_error(error).0, ErrorCode::ErrOpeningNotAllowed);
    // A پختہ کھاتہ account has no person to hold a balance (FR-13.5).
    let pukhta = account_id(&env, "0030");
    let error = accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(&env, pukhta, "31-08-2026", "100", "", false)],
        },
    )
    .unwrap_err();
    assert_eq!(first_error(error).0, ErrorCode::ErrOpeningNotAllowed);
}

#[test]
fn the_chart_tree_adds_balances_up_through_the_levels() {
    let (_dir, env, owner) = ready();
    let rahim = seller(&env, &owner, "رحیم", "Rahim");
    let akram = buyer(&env, &owner, "اکرم ٹول ٹیکس", "Akram Toll Tax");
    let cash = account_id(&env, "01010001");
    let capital = account_id(&env, "06010001");
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![
                line(&env, cash, "31-08-2026", "50000", "", false),
                line(&env, rahim.id, "31-08-2026", "15000", "", true),
                line(&env, akram.id, "31-08-2026", "2,880", "", false),
                line(&env, capital, "31-08-2026", "", "67,880", false),
            ],
        },
    )
    .unwrap();
    let tree = coa::tree(&env.ctx).unwrap();
    assert_eq!(
        tree.heads
            .iter()
            .map(|h| h.code.as_str())
            .collect::<Vec<_>>(),
        ["01", "02", "03", "04", "05", "06"]
    );
    let head = |code: &str| tree.heads.iter().find(|h| h.code == code).unwrap();
    // Assets: cash 50,000 debit. Parties: 15,000 + 2,880 debit. Capital: 67,880 credit.
    assert_eq!(
        (head("01").balance.side, head("01").balance.amount_paisa),
        (BalanceSide::Dr, rupees(50_000))
    );
    assert_eq!(
        (head("03").balance.side, head("03").balance.amount_paisa),
        (BalanceSide::Dr, rupees(17_880))
    );
    assert_eq!(
        (head("06").balance.side, head("06").balance.amount_paisa),
        (BalanceSide::Cr, rupees(67_880))
    );
    let buyers = head("03")
        .controls
        .iter()
        .find(|c| c.code == "0304")
        .unwrap();
    assert_eq!(buyers.balance.amount_paisa, rupees(2_880));
    // The accounts of the control come a page at a time (plan 38 B1).
    let page = coa::control_accounts(
        &env.ctx,
        &mandi_services::core::accounts::CoaControlAccountsInput {
            control_id: buyers.id,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(i64::from(page.total_count), buyers.account_count);
    assert_eq!(page.balance, buyers.balance, "the control's own balance");
    let node = page.rows.iter().find(|a| a.id == akram.id).unwrap();
    assert_eq!(
        (node.balance.side, node.balance.amount_paisa),
        (BalanceSide::Dr, rupees(2_880))
    );
    // The seeded accounts are in the tree and marked as system accounts, by code.
    let codes: Vec<&str> = page.rows.iter().map(|a| a.code.as_str()).collect();
    let mut sorted = codes.clone();
    sorted.sort_unstable();
    assert_eq!(codes, sorted);
    assert!(page.rows.iter().any(|a| a.code == "0030" && a.is_system));
    // Heads add up to the books: the debit heads and the credit heads are equal.
    let (mut dr, mut cr) = (0_i64, 0_i64);
    for h in &tree.heads {
        match h.balance.side {
            BalanceSide::Dr => dr += h.balance.amount_paisa.as_paisa(),
            BalanceSide::Cr => cr += h.balance.amount_paisa.as_paisa(),
            BalanceSide::Zero => {}
        }
    }
    assert_eq!(dr, cr);
    assert_eq!(coa::heads(&env.ctx).unwrap().len(), 6);
}

#[test]
fn a_control_gives_its_accounts_a_page_at_a_time_with_the_count_and_balance_of_all_of_them() {
    // Plan 38 B1: the chart sends heads and controls with a balance and an account count; the
    // accounts of a control come in pages of 25, 50 or 100, and the control's own balance is over
    // every account, not the page.
    let (_dir, env, owner) = ready();
    for i in 0..7 {
        save(
            &env,
            &owner,
            input(
                AccountType::Buyer,
                "",
                &format!("گاہک {i}"),
                &format!("Buyer {i}"),
            ),
        );
    }
    let tree = coa::tree(&env.ctx).unwrap();
    let buyers = tree
        .heads
        .iter()
        .flat_map(|h| &h.controls)
        .find(|c| c.code == "0304")
        .unwrap();
    // The two seeded buyers (نقدی منڈی, پختہ کھاتہ) and the seven.
    assert_eq!(buyers.account_count, 9);
    let page = |offset: u32, limit: u32| {
        coa::control_accounts(
            &env.ctx,
            &mandi_services::core::accounts::CoaControlAccountsInput {
                control_id: buyers.id,
                offset,
                limit,
            },
        )
        .unwrap()
    };
    for limit in [25, 50, 100] {
        let all = page(0, limit);
        assert_eq!((all.rows.len(), all.total_count, all.limit), (9, 9, limit));
    }
    // Pages of 4 read one after the other hold every account once, in the order of one big page.
    let whole: Vec<i64> = page(0, 100).rows.iter().map(|a| a.id).collect();
    let mut paged = Vec::new();
    for offset in [0, 4, 8] {
        let p = page(offset, 4);
        assert_eq!((p.total_count, p.balance), (9, buyers.balance));
        paged.extend(p.rows.iter().map(|a| a.id));
    }
    assert_eq!(paged, whole);
    // Past the end: no rows, the same count and balance. A limit of 0 is a page of 100.
    let past = page(40, 25);
    assert_eq!((past.rows.len(), past.total_count), (0, 9));
    assert_eq!(page(0, 0).limit, 100);
    // An unknown control and a page above 500 are refused.
    let missing = coa::control_accounts(
        &env.ctx,
        &mandi_services::core::accounts::CoaControlAccountsInput {
            control_id: 9_999,
            offset: 0,
            limit: 25,
        },
    )
    .unwrap_err();
    assert!(
        matches!(missing, ServiceError::Rejected(ref r) if r[0].code == ErrorCode::ErrNotFound)
    );
    let big = coa::control_accounts(
        &env.ctx,
        &mandi_services::core::accounts::CoaControlAccountsInput {
            control_id: buyers.id,
            offset: 0,
            limit: 501,
        },
    )
    .unwrap_err();
    assert!(matches!(big, ServiceError::Rejected(ref r) if r[0].code == ErrorCode::ErrOutOfRange));
}

#[test]
fn similar_accounts_are_found_by_name_in_either_script_and_by_phone() {
    // PRD FR-10.7 and FR-7.63: the warning shows before save.
    let (_dir, env, owner) = ready();
    let mut form = input(AccountType::Buyer, "", "اکرم ٹول ٹیکس", "Akram Toll Tax");
    form.phone1 = "0300-1234567".into();
    form.father_or_firm_en = "Ghulam Nabi".into();
    form.father_or_firm_ur = "غلام نبی".into();
    let akram = save(&env, &owner, form);
    buyer(&env, &owner, "ارشد ملتانی", "Arshad Multani");
    let ask = |ur: &str, en: &str, phone: &str, exclude: Option<i64>| {
        accounts::similar(
            &env.ctx,
            &AccountsSimilarInput {
                name_ur: ur.into(),
                name_en: en.into(),
                phone: phone.into(),
                tag: String::new(),
                exclude_id: exclude,
            },
        )
        .unwrap()
    };
    let found = ask("", "akrm", "", None);
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].id, found[0].father_or_firm_en.as_str()),
        (akram.id, "Ghulam Nabi")
    );
    assert_eq!(ask("اکرم", "", "", None).len(), 1);
    // The same phone shows even if the name is different.
    assert_eq!(ask("", "Someone Else", "03001234567", None)[0].id, akram.id);
    // The account being edited is left out.
    assert!(ask("", "akram", "", Some(akram.id)).is_empty());
    assert!(ask("", "", "", None).is_empty());
}

#[test]
fn a_rules_only_spelling_asks_to_be_confirmed_and_nothing_is_written() {
    // PRD FR-7.66 on the account form: names and the father or firm name are checked together.
    let (dir, env, owner) = ready();
    let mut form = input(AccountType::Buyer, "", "زیبق", "Zaibq");
    form.father_or_firm_ur = "قوبرط".into();
    form.father_or_firm_en = "Qobrt".into();
    let error = accounts::save(&env.ctx, &owner, &form).unwrap_err();
    let ServiceError::Rejected(items) = error else {
        panic!("not a rejection")
    };
    assert!(!items.is_empty());
    assert!(
        items
            .iter()
            .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed)
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM accounts WHERE name_en = 'Zaibq'"
        ),
        0
    );
    // After confirming, it saves, and the pair is learned.
    let row = save(&env, &owner, form);
    assert_eq!(row.name_en, "Zaibq");
}

#[test]
fn random_opening_changes_always_leave_the_books_balanced() {
    // Property: after every change in a long run of random openings, the trial balance is equal
    // and ابتدائی فرق is exactly Σ debits − Σ credits of the openings in force, on its credit side (PRD FR-10.9).
    let (dir, env, owner) = ready();
    let rows: Vec<AccountRow> = ["Akram", "Arshad", "Nasir", "Tariq"]
        .iter()
        .map(|name| buyer(&env, &owner, "ناصر", name))
        .collect();
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    // The opening now on each account, in rupees: (debit, credit).
    let mut now = [(0_i64, 0_i64); 4];
    for _ in 0..60 {
        let pick = usize::try_from(next() % 4).unwrap();
        let amount = i64::try_from(next() % 90_000).unwrap();
        let kind = next() % 3;
        let (dr, cr) = match kind {
            0 => (amount, 0),
            1 => (0, amount),
            _ => (0, 0),
        };
        let (date, dr_text, cr_text) = if dr == 0 && cr == 0 {
            (String::new(), String::new(), String::new())
        } else {
            (
                "31-08-2026".to_owned(),
                if dr > 0 {
                    dr.to_string()
                } else {
                    String::new()
                },
                if cr > 0 {
                    cr.to_string()
                } else {
                    String::new()
                },
            )
        };
        let id = rows[pick].id;
        let grid = OpeningsInput {
            lines: vec![line(&env, id, &date, &dr_text, &cr_text, false)],
        };
        // The preview says what ابتدائی فرق will be; the save must give exactly that (B5).
        let preview = accounts::openings_preview(&env.ctx, &grid).unwrap();
        let saved = accounts::openings_save(&env.ctx, &owner, &grid).unwrap();
        assert_eq!(preview.opening_difference_after, saved.opening_difference);
        now[pick] = (dr, cr);
        let trial = ledger::trial_balance(&env.ctx, None).unwrap();
        assert_eq!(trial.total_dr, trial.total_cr);
        let want: i64 = now.iter().map(|(d, c)| d - c).sum();
        let (side, amount) = difference(&env);
        let got = match side {
            BalanceSide::Cr => amount.as_paisa(),
            BalanceSide::Dr => -amount.as_paisa(),
            BalanceSide::Zero => 0,
        };
        assert_eq!(got, want * 100, "ابتدائی فرق after {now:?}");
        for (i, row) in rows.iter().enumerate() {
            let balance = accounts::get(&env.ctx, row.id).unwrap().balance;
            let expected = (now[i].0 - now[i].1) * 100;
            let shown = match balance.side {
                BalanceSide::Dr => balance.amount_paisa.as_paisa(),
                BalanceSide::Cr => -balance.amount_paisa.as_paisa(),
                BalanceSide::Zero => 0,
            };
            assert_eq!(shown, expected);
        }
    }
    ledger::verify_journal_chain(&env.ctx).unwrap();
    assert_chain_ok(&dir);
}

#[test]
fn twenty_thousand_accounts_are_listed_whole_with_their_balances() {
    // NFR-SCL-1 and NFR-SPD-3: the list returns every matching account up to 20,000 (no page
    // cap), with balances from the journal. The target is 300 ms in a release build; this checks
    // a generous bound in a debug build.
    const BOUND_MS: u128 = 5_000;
    let (dir, env, owner) = ready();
    {
        let conn = raw(&dir);
        let control = |code: &str| -> i64 {
            conn.query_row(
                "SELECT id FROM account_controls WHERE code = ?1",
                [code],
                |r| r.get(0),
            )
            .unwrap()
        };
        let (buyers, sellers) = (control("0304"), control("03011"));
        conn.execute_batch("BEGIN").unwrap();
        let mut statement = conn
            .prepare(
                "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur, search_en, search_phonetic)
                 VALUES (?1, ?2, ?3, ?4, 'ناصر', 'Nasir', 'ناصر', 'nasir', 'nsr')",
            )
            .unwrap();
        // Plan 32: sellers 001n and buyers 002n, the running number n from 0, no padding. The list
        // shows only sellers, buyers and the two پختہ کھاتہ accounts, so 10,000 of each make more
        // than 20,000 listed accounts.
        for n in 0..10_000_i64 {
            statement
                .execute((buyers, format!("002{n}"), n, "buyer"))
                .unwrap();
            statement
                .execute((sellers, format!("001{n}"), n, "seller"))
                .unwrap();
        }
        drop(statement);
        // 2,000 postings on the first buyers, so some balances are not zero. Each is a posted
        // entry: buyer debit 1,000 rupees against ابتدائی فرق.
        let difference: i64 = conn
            .query_row("SELECT id FROM accounts WHERE code = '06019999'", [], |r| {
                r.get(0)
            })
            .unwrap();
        for n in 0..2_000_i64 {
            let buyer: i64 = conn
                .query_row(
                    "SELECT id FROM accounts WHERE code = ?1",
                    [format!("002{n}")],
                    |r| r.get(0),
                )
                .unwrap();
            conn.execute(
                "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id) VALUES ('2026-09-01', 'general', 'voucher', ?1)",
                [n + 1],
            )
            .unwrap();
            let entry = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, 1, ?2, 100000, 0)",
                (entry, buyer),
            )
            .unwrap();
            conn.execute(
                "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, 2, ?2, 0, 100000)",
                (entry, difference),
            )
            .unwrap();
            conn.execute(
                "UPDATE journal_entries SET status = 'posted', total_paisa = 100000 WHERE id = ?1",
                [entry],
            )
            .unwrap();
        }
        conn.execute_batch("COMMIT").unwrap();
    }
    let started = Instant::now();
    let all = accounts::list(
        &env.ctx,
        &AccountsListInput {
            sort: None,
            filter: AccountFilter {
                kind: AccountListKind::All,
                text: String::new(),
                include_inactive: true,
                has_opening: None,
            },
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let took = started.elapsed();
    eprintln!(
        "{} accounts listed in {took:?} (debug build)",
        all.rows.len()
    );
    assert!(took.as_millis() < BOUND_MS, "{took:?}");
    // Every listed account is there: no cap. 2 x 10,000 made here and the 2 پختہ کھاتہ accounts.
    assert_eq!(all.rows.len(), 2 * 10_000 + 2);
    assert!(all.rows.len() > 20_000);
    assert_eq!(all.count, u32::try_from(all.rows.len()).unwrap());
    let codes: Vec<&str> = all.rows.iter().map(|r| r.code.as_str()).collect();
    let mut sorted = codes.clone();
    sorted.sort_by_key(|code| mandi_services::core::codes::code_sort_key(code));
    assert_eq!(codes, sorted, "by code, as numbers: 0019 before 00110");
    let row = all.rows.iter().find(|r| r.code == "0026").unwrap();
    assert_eq!(
        (row.balance.side, row.balance.amount_paisa),
        (BalanceSide::Dr, rupees(1_000))
    );
    let row = all.rows.iter().find(|r| r.code == "0022000").unwrap();
    assert_eq!(row.balance.side, BalanceSide::Zero);
    // A page of it is the same rows with the same balances, with a short named query.
    let page = accounts::list(
        &env.ctx,
        &AccountsListInput {
            sort: None,
            filter: AccountFilter {
                kind: AccountListKind::Buyer,
                text: String::new(),
                include_inactive: true,
                has_opening: None,
            },
            offset: 5,
            limit: 3,
        },
    )
    .unwrap();
    assert_eq!(page.rows.len(), 3);
    assert_eq!(
        page.count, 10_000,
        "the count is of all that match, not of the page"
    );
    assert_eq!(
        page.rows
            .iter()
            .map(|r| r.code.as_str())
            .collect::<Vec<_>>(),
        ["0025", "0026", "0027"]
    );
    assert!(
        page.rows
            .iter()
            .all(|r| r.balance.amount_paisa == rupees(1_000))
    );
    let _ = owner;
}

#[test]
fn the_account_rows_say_whether_they_can_take_an_opening() {
    // The grid leaves out the accounts that cannot take one, by the same named list the save
    // uses (ERR_OPENING_NOT_ALLOWED).
    let (_dir, env, owner) = ready();
    let rahim = seller(&env, &owner, "رحیم", "Rahim");
    // The list shows only sellers, buyers and the two پختہ کھاتہ accounts (plan 32 R6); the other
    // accounts are read one by one.
    let by_code = |code: &str| accounts::get(&env.ctx, account_id(&env, code)).unwrap();
    let flag = |code: &str| by_code(code).can_take_opening;
    assert!(flag("01010001"), "روکڑ");
    assert!(flag("06010001"), "ابتدائی سرمایہ");
    assert!(flag(&rahim.code), "a seller");
    assert!(!flag("04010001"), "income");
    assert!(!flag("06019999"), "ابتدائی فرق");
    assert!(!flag("03040000"), "نقدی منڈی");
    assert!(!flag("0030"), "پختہ کھاتہ (buyer)");
    assert!(!flag("0031"), "پختہ کھاتہ (seller)");
    // The flag and the save agree for these accounts.
    let mut codes = vec![
        "01010001",
        "01020001",
        "02010001",
        "04010001",
        "06010001",
        "06019999",
        "03040000",
        "030119998",
        "0030",
        "0031",
    ];
    codes.push(rahim.code.as_str());
    for row in codes.iter().map(|code| by_code(code)) {
        let tried = accounts::openings_save(
            &env.ctx,
            &owner,
            &OpeningsInput {
                lines: vec![line(&env, row.id, "01-09-2026", "100", "", false)],
            },
        );
        let refused = matches!(&tried, Err(ServiceError::Rejected(items)) if items.iter().any(|i| i.code == ErrorCode::ErrOpeningNotAllowed));
        assert_eq!(refused, !row.can_take_opening, "{}", row.code);
        if tried.is_ok() {
            // Put it back, so the next account's difference is not counted twice.
            accounts::openings_save(
                &env.ctx,
                &owner,
                &OpeningsInput {
                    lines: vec![line(&env, row.id, "", "", "", false)],
                },
            )
            .unwrap();
        }
    }
}

#[test]
fn a_line_error_of_the_openings_preview_carries_its_values() {
    // Code Standards 4.3: every error has a code and its values, also on a line of the grid, so
    // an ERR_DATE_CLOSED row can show the closed-up-to date.
    let (dir, env, owner) = ready();
    raw(&dir)
        .execute(
            "UPDATE day_state SET closed_up_to_date = '2026-09-09' WHERE id = 1",
            [],
        )
        .unwrap();
    let akram = buyer(&env, &owner, "اکرم ٹول ٹیکس", "Akram Toll Tax");
    let preview = accounts::openings_preview(
        &env.ctx,
        &OpeningsInput {
            lines: vec![
                line(&env, akram.id, "01-09-2026", "100", "", false),
                line(&env, akram.id, "01-09-2026", "x", "", false),
            ],
        },
    )
    .unwrap();
    let closed = preview.rows[0]
        .errors
        .iter()
        .find(|e| e.code == ErrorCode::ErrDateClosed)
        .expect("a closed day error");
    assert_eq!(closed.field, "lines[0].opening_date");
    assert_eq!(
        closed.values.get("closed_up_to"),
        Some(&ErrorValue::Text("09-09-2026".into()))
    );
    // The same values arrive when the grid is saved and refused.
    let error = accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(&env, akram.id, "01-09-2026", "100", "", false)],
        },
    )
    .unwrap_err();
    let ServiceError::Rejected(items) = error else {
        panic!("not a rejection")
    };
    assert_eq!(
        items[0].values.get("closed_up_to"),
        Some(&ErrorValue::Text("09-09-2026".into()))
    );
}

/// Rahim with the 15,000 opening advance of PRD 4.3 on 31-08-2026, and every day up to and
/// including 09-10-2026 closed. Returns his row.
fn rahim_on_a_closed_day(dir: &TempDir, env: &TestEnv, owner: &auth::Actor) -> AccountRow {
    let mut form = input(AccountType::Seller, "", "رحیم", "Rahim");
    form.opening_date = "31-08-2026".into();
    form.opening_dr = "15,000".into();
    form.opening_is_advance = true;
    let rahim = save(env, owner, form);
    raw(dir)
        .execute(
            "UPDATE day_state SET closed_up_to_date = '2026-10-09' WHERE id = 1",
            [],
        )
        .unwrap();
    rahim
}

/// The form as it comes back from the screen: the stored opening, untouched.
fn form_of(row: &AccountRow) -> AccountInput {
    let mut form = input(AccountType::Seller, &row.code, "رحیم", "Rahim");
    form.id = Some(row.id);
    form.version = Some(row.version);
    form.opening_date = "31-08-2026".into();
    form.opening_dr = "15000".into();
    form.opening_is_advance = true;
    form
}

#[test]
fn an_account_whose_opening_day_is_closed_can_still_be_edited() {
    // End-gate review 1: phone, name and note do not touch the opening, so a closed opening day
    // must not stop them, and no new entry is posted.
    let (dir, env, owner) = ready();
    let rahim = rahim_on_a_closed_day(&dir, &env, &owner);
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let lines = count(&dir, "SELECT COUNT(*) FROM journal_lines");

    let mut edit = form_of(&rahim);
    edit.phone1 = "0300-1234567".into();
    edit.note = "نیا نوٹ".into();
    let saved = save(&env, &owner, edit);
    assert_eq!(saved.phone1, "03001234567");
    assert_eq!(saved.version, rahim.version + 1);
    assert_eq!(saved.opening_date.as_deref(), Some("2026-08-31"));
    assert_eq!(saved.opening_dr_paisa, rupees(15_000));
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_lines"), lines);

    // A real change of the opening is still refused on the closed day, with nothing written.
    let mut change = form_of(&saved);
    change.opening_dr = "16000".into();
    let (code, field) = first_error(accounts::save(&env.ctx, &owner, &change).unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrDateClosed, Some("opening_date"))
    );

    // Empty opening fields mean "clear the opening", which reverses the entry on its closed
    // day: refused.
    let mut clear = form_of(&saved);
    clear.opening_date = String::new();
    clear.opening_dr = String::new();
    clear.opening_is_advance = false;
    let (code, field) = first_error(accounts::save(&env.ctx, &owner, &clear).unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrDateClosed, Some("opening_date"))
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
    assert_eq!(
        accounts::get(&env.ctx, saved.id).unwrap().opening_dr_paisa,
        rupees(15_000)
    );
    ledger::verify_journal_chain(&env.ctx).unwrap();
}

#[test]
fn the_golden_cash_account_can_be_edited_after_its_opening_day_is_closed() {
    // The PRD 4.3 setup: روکڑ has an opening on 31-08-2026 and the days are closed to 09-10-2026.
    let (dir, env, owner) = ready();
    let cash = account_id(&env, "01010001");
    let capital = account_id(&env, "06010001");
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![
                line(&env, cash, "31-08-2026", "65000", "", false),
                line(&env, capital, "31-08-2026", "", "65000", false),
            ],
        },
    )
    .unwrap();
    raw(&dir)
        .execute(
            "UPDATE day_state SET closed_up_to_date = '2026-10-09' WHERE id = 1",
            [],
        )
        .unwrap();
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let row = accounts::get(&env.ctx, cash).unwrap();
    let mut edit = input(AccountType::Other, &row.code, &row.name_ur, &row.name_en);
    edit.id = Some(row.id);
    edit.version = Some(row.version);
    edit.control_id = Some(row.control.id);
    edit.opening_date = "31-08-2026".into();
    edit.opening_dr = "65000".into();
    edit.note = "روکڑ کا نوٹ".into();
    let saved = save(&env, &owner, edit);
    assert_eq!(saved.version, row.version + 1);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
}

#[test]
fn the_openings_preview_and_save_give_the_same_errors_for_a_closed_old_day() {
    // End-gate review 2: changing an opening reverses the old entry on its own day, so the
    // PREVIEW must refuse a closed old day too, and every error must name its line.
    let (dir, env, owner) = ready();
    let rahim = rahim_on_a_closed_day(&dir, &env, &owner);
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");

    // The new date is open, the old one (31-08) is closed.
    let moved = OpeningsInput {
        lines: vec![line(&env, rahim.id, "10-10-2026", "15000", "", true)],
    };
    let preview = accounts::openings_preview(&env.ctx, &moved).unwrap();
    let preview_errors: Vec<(ErrorCode, String)> = preview.rows[0]
        .errors
        .iter()
        .map(|e| (e.code, e.field.clone()))
        .collect();
    assert_eq!(
        preview_errors,
        [(ErrorCode::ErrDateClosed, "lines[0].opening_date".to_owned())]
    );
    let ServiceError::Rejected(items) =
        accounts::openings_save(&env.ctx, &owner, &moved).unwrap_err()
    else {
        panic!("not a rejection")
    };
    let save_errors: Vec<(ErrorCode, String)> = items
        .iter()
        .map(|i| (i.code, i.field.clone().unwrap_or_default()))
        .collect();
    assert_eq!(save_errors, preview_errors);

    // Clearing the opening is the same.
    let clear = OpeningsInput {
        lines: vec![line(&env, rahim.id, "", "", "", false)],
    };
    let preview = accounts::openings_preview(&env.ctx, &clear).unwrap();
    assert_eq!(preview.rows[0].errors.len(), 1);
    assert_eq!(preview.rows[0].errors[0].code, ErrorCode::ErrDateClosed);
    assert_eq!(preview.rows[0].errors[0].field, "lines[0].opening_date");
    assert!(accounts::openings_save(&env.ctx, &owner, &clear).is_err());

    // A line that leaves the opening as it is has no error, whatever day it is on.
    let same = OpeningsInput {
        lines: vec![line(&env, rahim.id, "31-08-2026", "15000", "", true)],
    };
    let preview = accounts::openings_preview(&env.ctx, &same).unwrap();
    assert!(preview.rows[0].errors.is_empty());
    accounts::openings_save(&env.ctx, &owner, &same).unwrap();
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);
}

#[test]
fn an_inactive_account_takes_no_new_opening_but_can_be_edited_and_cleared() {
    // End-gate review 20, PRD FR-10.2 and FR-10.9: a new opening is a new posting, so it needs an
    // active account; editing the other fields and taking the opening away do not.
    let (dir, env, owner) = ready();
    let mut form = input(AccountType::Seller, "", "رحیم", "Rahim");
    form.opening_date = "31-08-2026".into();
    form.opening_dr = "15,000".into();
    form.opening_is_advance = true;
    let rahim = save(&env, &owner, form);
    accounts::set_active(&env.ctx, &owner, rahim.id, false).unwrap();
    let rahim = accounts::get(&env.ctx, rahim.id).unwrap();
    let entries = count(&dir, "SELECT COUNT(*) FROM journal_entries");

    // The form with the stored opening back unchanged: the phone can be edited.
    let mut edit = form_of(&rahim);
    edit.is_active = false;
    edit.phone1 = "0300-1234567".into();
    let saved = save(&env, &owner, edit);
    assert_eq!(saved.phone1, "03001234567");
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);

    // A new amount is refused on the amount field, in the form and in the grid, the same way.
    let mut change = form_of(&saved);
    change.is_active = false;
    change.opening_dr = "16000".into();
    let (code, field) = first_error(accounts::save(&env.ctx, &owner, &change).unwrap_err());
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrAccountInactive, Some("opening_dr"))
    );
    let grid = OpeningsInput {
        lines: vec![line(&env, saved.id, "31-08-2026", "16000", "", true)],
    };
    let preview = accounts::openings_preview(&env.ctx, &grid).unwrap();
    let preview_errors: Vec<(ErrorCode, String)> = preview.rows[0]
        .errors
        .iter()
        .map(|e| (e.code, e.field.clone()))
        .collect();
    assert_eq!(
        preview_errors,
        [(
            ErrorCode::ErrAccountInactive,
            "lines[0].opening_dr".to_owned()
        )]
    );
    let ServiceError::Rejected(items) =
        accounts::openings_save(&env.ctx, &owner, &grid).unwrap_err()
    else {
        panic!("not a rejection")
    };
    let save_errors: Vec<(ErrorCode, String)> = items
        .iter()
        .map(|i| (i.code, i.field.clone().unwrap_or_default()))
        .collect();
    assert_eq!(save_errors, preview_errors);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), entries);

    // Taking the opening away only reverses the old entry, which is always allowed.
    let clear = OpeningsInput {
        lines: vec![line(&env, saved.id, "", "", "", false)],
    };
    assert!(
        accounts::openings_preview(&env.ctx, &clear).unwrap().rows[0]
            .errors
            .is_empty()
    );
    accounts::openings_save(&env.ctx, &owner, &clear).unwrap();
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM journal_entries"),
        entries + 1,
        "one reversal"
    );
    assert_eq!(
        accounts::get(&env.ctx, saved.id).unwrap().opening_dr_paisa,
        Paisa::ZERO
    );
    ledger::verify_journal_chain(&env.ctx).unwrap();
}

#[test]
fn the_list_with_every_is_the_full_list_the_opening_balances_screen_needs() {
    // Plan 32 decision D4: `kind = every` returns every active account, as the list did before
    // plan 32, in chart order, with the footer over those rows. `all` leaves cash and the other
    // system accounts out. The Openings screen needs cash and the capital accounts.
    let (_dir, env, owner) = ready();
    let rahim = seller(&env, &owner, "رحیم", "Rahim");
    let akram = buyer(&env, &owner, "اکرم ٹول ٹیکس", "Akram Toll Tax");
    save(
        &env,
        &owner,
        input(AccountType::MoneyLent, "", "اسلم", "Aslam"),
    );
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![
                line(
                    &env,
                    account_id(&env, "01010001"),
                    "31-08-2026",
                    "65,000",
                    "",
                    false,
                ),
                line(
                    &env,
                    account_id(&env, "06010001"),
                    "31-08-2026",
                    "",
                    "65000",
                    false,
                ),
                line(&env, rahim.id, "31-08-2026", "", "4,000", false),
                line(&env, akram.id, "31-08-2026", "4,000", "", false),
            ],
        },
    )
    .unwrap();
    let list = |kind: AccountListKind| {
        accounts::list(
            &env.ctx,
            &AccountsListInput {
                sort: None,
                filter: AccountFilter {
                    kind,
                    text: String::new(),
                    include_inactive: false,
                    has_opening: None,
                },
                offset: 0,
                limit: 0,
            },
        )
        .unwrap()
    };
    let every = list(AccountListKind::Every);
    let codes: Vec<&str> = every.rows.iter().map(|r| r.code.as_str()).collect();
    for want in [
        "01010001",
        "01020001",
        "02010001",
        "04010001",
        "06010001",
        "06019999",
        "06020001",
        "03040000",
        "030119998",
        "0030",
        "0031",
        "0010",
        "0020",
        "01030001",
    ] {
        if want == "06020001" {
            continue; // a partner's capital account: there is no partner in this test
        }
        assert!(
            codes.contains(&want),
            "{want} is in the full list: {codes:?}"
        );
    }
    // Chart order: by control, then by running number inside it (cash first, sellers before buyers).
    let at = |code: &str| codes.iter().position(|c| *c == code).unwrap();
    assert_eq!(at("01010001"), 0);
    assert!(at("01010001") < at("01030001"));
    assert!(at("01030001") < at("0010"));
    assert!(at("0010") < at("0020"));
    assert!(at("0020") < at("04010001"));
    assert!(at("04010001") < at("06010001"));
    // The footer is over these rows: the openings that were saved.
    assert_eq!(every.count, u32::try_from(every.rows.len()).unwrap());
    assert_eq!(every.total_dr_paisa, rupees(69_000));
    assert_eq!(every.total_cr_paisa, rupees(69_000));
    // Cash and the opening capital take an opening; ابتدائی فرق and income do not.
    let flag = |code: &str| {
        every
            .rows
            .iter()
            .find(|r| r.code == code)
            .unwrap()
            .can_take_opening
    };
    assert!(flag("01010001") && flag("06010001") && flag("01030001") && flag("0010"));
    assert!(!flag("06019999") && !flag("04010001") && !flag("03040000"));
    // `all` is the Accounts list: no cash, no capital, no money-lent person.
    let all = list(AccountListKind::All);
    let all_codes: Vec<&str> = all.rows.iter().map(|r| r.code.as_str()).collect();
    assert_eq!(all_codes, ["0010", "0020", "0030", "0031"]);
    assert_eq!(all.total_dr_paisa, rupees(4_000));
    assert_eq!(all.total_cr_paisa, rupees(4_000));
    // A text filter works inside `every` too.
    let found = accounts::list(
        &env.ctx,
        &AccountsListInput {
            sort: None,
            filter: AccountFilter {
                kind: AccountListKind::Every,
                text: "cash".into(),
                include_inactive: false,
                has_opening: None,
            },
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert!(found.rows.iter().any(|r| r.code == "01010001"));
}

#[test]
fn an_account_that_is_not_in_the_list_still_opens_and_saves_from_the_chart() {
    // Plan 32 R6 (lead's addition): cash, a payable, a money-lent person and a khata are not in the
    // Accounts list, but the chart opens them by id, and the form saves them with the type they have.
    let (_dir, env, owner) = ready();
    let lent = save(
        &env,
        &owner,
        input(AccountType::MoneyLent, "", "اسلم", "Aslam"),
    );
    let khata = mandi_services::khata::create(
        &env.ctx,
        &owner,
        &mandi_services::core::khata::KhataCreateInput {
            name_ur: "خاندانی کھاتہ".into(),
            name_en: "Family Khata".into(),
            note: None,
            confirmed_rules_words: vec![
                "Family".into(),
                "family".into(),
                "خاندانی".into(),
                "khata".into(),
            ],
        },
    )
    .unwrap();
    let chart = chart_accounts(&env);
    let in_chart: Vec<&str> = chart.iter().map(|(_, a)| a.code.as_str()).collect();
    for id in [
        account_id(&env, "01010001"),
        account_id(&env, "02010001"),
        lent.id,
        khata.account.id,
    ] {
        let row = accounts::get(&env.ctx, id).unwrap();
        assert!(
            in_chart.contains(&row.code.as_str()),
            "{} is in the chart",
            row.code
        );
        let mut form = input(row.account_type, &row.code, &row.name_ur, &row.name_en);
        form.id = Some(row.id);
        form.version = Some(row.version);
        // The form of an account under another control sends its control, as the chart does.
        form.control_id = Some(row.control.id);
        form.phone1 = "0300-1234567".into();
        form.note = "نوٹ".into();
        form.is_active = row.is_active;
        let saved = save(&env, &owner, form);
        assert_eq!(saved.code, row.code, "the code stays");
        assert_eq!(saved.account_type, row.account_type, "the type stays");
        assert_eq!(saved.control.id, row.control.id);
        assert_eq!(saved.phone1, "03001234567");
        assert_eq!(saved.version, row.version + 1);
    }
}

#[test]
fn the_opening_filter_lists_the_accounts_that_can_take_an_opening_and_have_none() {
    // Plan 38 (PRD FR-10.9): the Opening balances screen opens on "no opening yet".
    let (_dir, env, owner) = ready();
    let rahim = seller(&env, &owner, "رحیم", "Rahim");
    buyer(&env, &owner, "اکرم ٹول ٹیکس", "Akram Toll Tax");
    buyer(&env, &owner, "ارشد ملتانی", "Arshad Multani");
    let arshad = account_id(&env, "0021");
    let list = |has_opening: Option<bool>| {
        accounts::list(
            &env.ctx,
            &AccountsListInput {
                sort: None,
                filter: AccountFilter {
                    kind: AccountListKind::Every,
                    text: String::new(),
                    include_inactive: false,
                    has_opening,
                },
                offset: 0,
                limit: 0,
            },
        )
        .unwrap()
    };
    let codes = |l: &mandi_services::core::accounts::AccountsList| -> Vec<String> {
        l.rows.iter().map(|r| r.code.clone()).collect()
    };
    let none_yet = list(Some(false));
    let names = codes(&none_yet);
    // Cash, the capital account and the people can take an opening.
    for code in ["01010001", "06010001", "0010", "0020", "0021"] {
        assert!(names.contains(&code.to_owned()), "{code} is missing");
    }
    // Income, ابتدائی فرق, نقدی منڈی and the two پختہ کھاتہ accounts never can.
    for code in ["04010001", "06019999", "03040000", "0030", "0031"] {
        assert!(!names.contains(&code.to_owned()), "{code} cannot take one");
    }
    // The SQL and `can_take_opening` say the same: this is the whole list, less the rows that
    // cannot take an opening or already have one.
    let every = list(None);
    let expected: Vec<String> = every
        .rows
        .iter()
        .filter(|r| {
            r.can_take_opening
                && r.opening_dr_paisa.as_paisa() == 0
                && r.opening_cr_paisa.as_paisa() == 0
        })
        .map(|r| r.code.clone())
        .collect();
    assert_eq!(names, expected);
    assert_eq!(none_yet.total_dr_paisa.as_paisa(), 0);
    assert_eq!(none_yet.total_cr_paisa.as_paisa(), 0);

    // Two openings are entered: those two accounts move from "none yet" to "has one".
    let grid = OpeningsInput {
        lines: vec![
            line(&env, rahim.id, "31-08-2026", "15,000", "", true),
            line(&env, arshad, "31-08-2026", "", "3000", false),
        ],
    };
    accounts::openings_save(&env.ctx, &owner, &grid).unwrap();
    let after = list(Some(false));
    let names_after = codes(&after);
    assert!(!names_after.contains(&"0010".to_owned()) && !names_after.contains(&"0021".to_owned()));
    assert_eq!(after.count, none_yet.count - 2);
    let with = list(Some(true));
    assert_eq!(codes(&with), ["0010", "0021"]);
    assert_eq!(with.total_dr_paisa.as_paisa(), 1_500_000);
    assert_eq!(with.total_cr_paisa.as_paisa(), 300_000);
    // The three lists add up to the whole list: with an opening, without, and the accounts that
    // cannot take one.
    let cannot = every.rows.iter().filter(|r| !r.can_take_opening).count();
    let all = list(None);
    assert_eq!(
        usize::try_from(with.count + after.count).unwrap() + cannot,
        usize::try_from(all.count).unwrap()
    );
    // The totals of the whole list are the totals of the accounts with an opening.
    assert_eq!(all.total_dr_paisa, with.total_dr_paisa);
    assert_eq!(all.total_cr_paisa, with.total_cr_paisa);
}
