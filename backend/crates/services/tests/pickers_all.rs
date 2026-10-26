//! Plan 32 R1 to R8 (PRD 1.8), backend part: the codes of items, packings and staff, the short code
//! in every box, and the whole list a click on a picker shows (empty text): the records chosen last
//! first, then the rest by code, one page at a time, for the masters and for the account fields.
#![cfg(test)]

mod common;

use common::*;
use mandi_services::core::accounts::{
    AccountInput, AccountScope, AccountType, AccountsTouchInput, ResolveCodeInput, ResolveStatus,
    SearchAccountsInput,
};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::masters::{
    ItemInput, MasterInput, MasterKind, MasterRow, PackingInput, StaffInput,
};
use mandi_services::core::search::SearchScope;
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, auth, masters, search};
use tempfile::TempDir;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

/// Saves a master, confirming whatever spelling the check asks about.
fn save_master(env: &TestEnv, owner: &auth::Actor, mut input: MasterInput) -> MasterRow {
    loop {
        match masters::save(&env.ctx, owner, &input) {
            Ok(row) => return row,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                let mut words: Vec<String> = Vec::new();
                for item in items {
                    if let Some(ErrorValue::Text(found)) = item.values.get("words") {
                        words.extend(found.split(' ').map(str::to_owned));
                    }
                }
                match &mut input {
                    MasterInput::Item(i) => i.confirmed_rules_words.extend(words),
                    MasterInput::Packing(i) => i.confirmed_rules_words.extend(words),
                    MasterInput::Staff(i) => i.confirmed_rules_words.extend(words),
                    _ => panic!("not a kind of this test"),
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

fn item(env: &TestEnv, owner: &auth::Actor, code: &str, ur: &str, en: &str) -> MasterRow {
    save_master(
        env,
        owner,
        MasterInput::Item(ItemInput {
            id: None,
            version: None,
            code: code.into(),
            name_ur: ur.into(),
            name_en: en.into(),
            item_group: "fruit".into(),
            default_packing_id: None,
            sold_by: "unit".into(),
            confirmed_rules_words: Vec::new(),
        }),
    )
}

fn packing(env: &TestEnv, owner: &auth::Actor, code: &str, ur: &str, en: &str) -> MasterRow {
    save_master(
        env,
        owner,
        MasterInput::Packing(PackingInput {
            id: None,
            version: None,
            code: code.into(),
            name_ur: ur.into(),
            name_en: en.into(),
            approx_kg: String::new(),
            confirmed_rules_words: Vec::new(),
        }),
    )
}

fn staff(env: &TestEnv, owner: &auth::Actor, code: &str, ur: &str, en: &str) -> MasterRow {
    save_master(
        env,
        owner,
        MasterInput::Staff(StaffInput {
            id: None,
            version: None,
            name_ur: ur.into(),
            name_en: en.into(),
            code: code.into(),
            job: "munshi".into(),
            phone: String::new(),
            confirmed_rules_words: Vec::new(),
        }),
    )
}

fn code_of(row: &MasterRow) -> String {
    match row {
        MasterRow::Item(r) => r.code.clone(),
        MasterRow::Packing(r) => r.code.clone(),
        MasterRow::Staff(r) => r.code.clone(),
        _ => panic!("not a kind of this test"),
    }
}

fn account(
    env: &TestEnv,
    owner: &auth::Actor,
    kind: AccountType,
    ur: &str,
    en: &str,
) -> mandi_services::core::accounts::AccountRow {
    let mut input = AccountInput {
        id: None,
        version: None,
        account_type: kind,
        control_id: None,
        code: String::new(),
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
    };
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

fn account_page(
    env: &TestEnv,
    owner: &auth::Actor,
    scope: AccountScope,
    text: &str,
    offset: u32,
    limit: u32,
) -> Vec<String> {
    search::search_accounts(
        &env.ctx,
        owner,
        &SearchAccountsInput {
            scope,
            control_id: None,
            text: text.into(),
            limit,
            offset,
        },
    )
    .unwrap()
    .into_iter()
    .map(|h| h.code)
    .collect()
}

fn master_page(
    env: &TestEnv,
    owner: &auth::Actor,
    scope: SearchScope,
    text: &str,
    offset: u32,
    limit: u32,
) -> Vec<String> {
    search::search_page(&env.ctx, owner, scope, text, offset, limit)
        .unwrap()
        .into_iter()
        .map(|h| h.code)
        .collect()
}

#[test]
fn items_packings_and_staff_get_their_own_type_number_and_running_numbers() {
    // Plan 32 R1, R2: item 004, packing 005, staff 006; first codes 0040, 0050, 0060.
    let (_dir, env, owner) = ready();
    assert_eq!(
        masters::next_code(&env.ctx, MasterKind::Item).unwrap(),
        "0040"
    );
    assert_eq!(
        masters::next_code(&env.ctx, MasterKind::Packing).unwrap(),
        "0050"
    );
    assert_eq!(
        masters::next_code(&env.ctx, MasterKind::Staff).unwrap(),
        "0060"
    );
    assert_eq!(code_of(&item(&env, &owner, "", "کیلا", "Banana")), "0040");
    assert_eq!(code_of(&item(&env, &owner, "", "سیب", "Apple")), "0041");
    assert_eq!(code_of(&packing(&env, &owner, "", "پیٹی", "Box")), "0050");
    assert_eq!(code_of(&staff(&env, &owner, "", "اسلم", "Aslam")), "0060");
    assert_eq!(code_of(&staff(&env, &owner, "", "بلال", "Bilal")), "0061");
    assert_eq!(
        masters::next_code(&env.ctx, MasterKind::Item).unwrap(),
        "0042"
    );
    // Cities and the others keep a plain number (R4).
    assert_eq!(masters::next_code(&env.ctx, MasterKind::City).unwrap(), "1");
    // A typed code must be a full code of the type, and free.
    for (typed, wanted) in [
        ("0050", ErrorCode::ErrCodeInvalid),
        ("0043x", ErrorCode::ErrFormatInvalid),
        ("00403", ErrorCode::ErrCodeInvalid),
        ("42", ErrorCode::ErrCodeInvalid),
        ("0041", ErrorCode::ErrCodeTaken),
    ] {
        let error = masters::save(
            &env.ctx,
            &owner,
            &MasterInput::Item(ItemInput {
                id: None,
                version: None,
                code: typed.into(),
                name_ur: "آم".into(),
                name_en: "Mango".into(),
                item_group: "fruit".into(),
                default_packing_id: None,
                sold_by: "unit".into(),
                confirmed_rules_words: Vec::new(),
            }),
        )
        .unwrap_err();
        match error {
            ServiceError::Rejected(items) => {
                assert_eq!(
                    (items[0].code, items[0].field.as_deref()),
                    (wanted, Some("code")),
                    "{typed}"
                );
            }
            other => panic!("{typed}: {other:?}"),
        }
    }
    // A free code of the type is taken as typed, and the next goes on from the highest.
    assert_eq!(
        code_of(&item(&env, &owner, "0049", "انار", "Pomegranate")),
        "0049"
    );
    assert_eq!(
        masters::next_code(&env.ctx, MasterKind::Item).unwrap(),
        "00410"
    );
}

#[test]
fn the_short_code_of_an_item_is_its_running_number() {
    // Plan 32 R5: the running number alone finds the record in its box; the full code works too.
    let (_dir, env, owner) = ready();
    item(&env, &owner, "", "کیلا", "Banana");
    item(&env, &owner, "", "سیب", "Apple");
    packing(&env, &owner, "", "پیٹی", "Box");
    let hits = |scope, text| master_page(&env, &owner, scope, text, 0, 10);
    assert_eq!(
        hits(SearchScope::Item, "0").first().map(String::as_str),
        Some("0040")
    );
    assert_eq!(
        hits(SearchScope::Item, "1").first().map(String::as_str),
        Some("0041")
    );
    assert_eq!(
        hits(SearchScope::Item, "0041").first().map(String::as_str),
        Some("0041")
    );
    // `0` in the packing box is the first packing, not the first item.
    assert_eq!(hits(SearchScope::Packing, "0"), ["0050"]);
    // A number nobody has finds nothing by code.
    assert!(hits(SearchScope::Item, "7").is_empty());
}

#[test]
fn empty_text_gives_every_active_record_recent_first_then_by_code() {
    // Plan 32 R7: a click on a picker shows all records of its type. Example: a buyer box with
    // the buyers and the two system buyers.
    let (_dir, env, owner) = ready();
    let mut buyers = Vec::new();
    for n in 0..12 {
        buyers.push(account(
            &env,
            &owner,
            AccountType::Buyer,
            &format!("گاہک {n}"),
            &format!("Buyer {n}"),
        ));
    }
    // By code in running-number order: 0020 ... 0029, 00210, 00211, then the other codes of the
    // box (the پختہ کھاتہ buyer is also a code of the scheme; نقدی منڈی keeps its old code).
    let all = account_page(&env, &owner, AccountScope::Buyer, "", 0, 50);
    let want: Vec<String> = (0..12)
        .map(|n| format!("002{n}"))
        .chain(["0030".to_owned(), "03040000".to_owned()])
        .collect();
    assert_eq!(all, want);
    // Choosing one puts it first next time, then the one before it, then the rest by code.
    for id in [buyers[5].id, buyers[2].id] {
        search::touch_account(
            &env.ctx,
            &owner,
            &AccountsTouchInput {
                scope: AccountScope::Buyer,
                control_id: None,
                id,
            },
        )
        .unwrap();
    }
    let all = account_page(&env, &owner, AccountScope::Buyer, "", 0, 50);
    assert_eq!(&all[..4], ["0022", "0025", "0020", "0021"]);
    assert_eq!(all.len(), 14);
    // Whitespace is empty text.
    assert_eq!(
        account_page(&env, &owner, AccountScope::Buyer, "   ", 0, 50),
        all
    );
    // Pages: 5 at a time, no record twice and none missing; a short page is the last.
    let mut joined: Vec<String> = Vec::new();
    for page in 0..3 {
        let part = account_page(&env, &owner, AccountScope::Buyer, "", page * 5, 5);
        if page < 2 {
            assert_eq!(part.len(), 5);
        } else {
            assert_eq!(part.len(), 4);
        }
        joined.extend(part);
    }
    assert_eq!(joined, all);
    assert!(account_page(&env, &owner, AccountScope::Buyer, "", 14, 5).is_empty());
    // An inactive account is not offered.
    accounts::set_active(&env.ctx, &owner, buyers[0].id, false).unwrap();
    assert!(
        !account_page(&env, &owner, AccountScope::Buyer, "", 0, 50).contains(&"0020".to_owned())
    );
    // The seller box has its own list, with the پختہ کھاتہ seller (0031) after the sellers.
    let rahim = account(&env, &owner, AccountType::Seller, "رحیم", "Rahim");
    assert_eq!(
        account_page(&env, &owner, AccountScope::Seller, "", 0, 50),
        [rahim.code.as_str(), "0031"]
    );
    // اپنا مال is only in the seller field of a lot.
    let lot = account_page(&env, &owner, AccountScope::LotSeller, "", 0, 50);
    assert!(lot.contains(&"030119998".to_owned()));
    assert!(
        !account_page(&env, &owner, AccountScope::Seller, "", 0, 50)
            .contains(&"030119998".to_owned())
    );
}

#[test]
fn a_list_that_is_long_loads_page_by_page_with_the_masters_search_too() {
    let (_dir, env, owner) = ready();
    for n in 0..23 {
        item(&env, &owner, "", &format!("مال {n}"), &format!("Goods {n}"));
    }
    let first = master_page(&env, &owner, SearchScope::Item, "", 0, 10);
    let second = master_page(&env, &owner, SearchScope::Item, "", 10, 10);
    let third = master_page(&env, &owner, SearchScope::Item, "", 20, 10);
    assert_eq!((first.len(), second.len(), third.len()), (10, 10, 3));
    let joined: Vec<String> = first.into_iter().chain(second).chain(third).collect();
    let want: Vec<String> = (0..23).map(|n| format!("004{n}")).collect();
    assert_eq!(joined, want, "by code as numbers: 0049 before 00410");
    // A typed text pages too: the running numbers that start with `1` are 1, 10 ... 19.
    let typed = master_page(&env, &owner, SearchScope::Item, "1", 0, 50);
    assert_eq!(typed.first().map(String::as_str), Some("0041"));
    assert!(typed.contains(&"00410".to_owned()) && typed.contains(&"00419".to_owned()));
    let rest = master_page(&env, &owner, SearchScope::Item, "1", 1, 50);
    assert_eq!(rest, typed[1..]);
    // The recent list (Alt+Down) is the same order: chosen first, then by code.
    let recent = search::recent(&env.ctx, &owner, SearchScope::Item).unwrap();
    assert_eq!(recent.first().map(|h| h.code.as_str()), Some("0040"));
    assert_eq!(recent.len(), 23);
}

#[test]
fn a_short_code_names_a_seller_a_buyer_or_both_and_a_full_code_is_never_a_short_code() {
    // Plan 32 Section 3.
    let (_dir, env, owner) = ready();
    let mut sellers = Vec::new();
    for n in 0..12 {
        sellers.push(account(
            &env,
            &owner,
            AccountType::Seller,
            &format!("بیوپاری {n}"),
            &format!("Seller {n}"),
        ));
    }
    let buyer = account(&env, &owner, AccountType::Buyer, "گاہک", "Buyer");
    let ask = |scope, text: &str| {
        search::resolve_code(
            &env.ctx,
            &ResolveCodeInput {
                scope,
                control_id: None,
                text: text.into(),
            },
        )
        .unwrap()
    };
    // `0` finds 0010 in a seller box and 0020 in a buyer box.
    assert_eq!(
        ask(AccountScope::Seller, "0").account.map(|a| a.code),
        Some("0010".to_owned())
    );
    assert_eq!(
        ask(AccountScope::Buyer, "0").account.map(|a| a.code),
        Some(buyer.code.clone())
    );
    // `11` is the seller 00111 (the twelfth).
    assert_eq!(
        ask(AccountScope::Seller, "11").account.map(|a| a.code),
        Some("00111".to_owned())
    );
    // `10` is 00110, not the code 0010 read as a number.
    assert_eq!(
        ask(AccountScope::Seller, "10").account.map(|a| a.code),
        Some("00110".to_owned())
    );
    // `1` in a box for any account lists the seller 0011 only (no buyer has 1 yet).
    let one = ask(AccountScope::Any, "1");
    assert_eq!(one.status, ResolveStatus::Found);
    assert_eq!(one.account.map(|a| a.code), Some("0011".to_owned()));
    // `0` in a box for any account lists the seller and the buyer.
    let zero = ask(AccountScope::Any, "0");
    assert_eq!(zero.status, ResolveStatus::Several);
    assert_eq!(zero.candidates.len(), 2);
    // Typing `10` in the search puts the seller 00110 first (the running number), not 0010.
    let hits = account_page(&env, &owner, AccountScope::Seller, "10", 0, 5);
    assert_eq!(hits.first().map(String::as_str), Some("00110"));
    // Typing the full code `0010` puts that seller first.
    let hits = account_page(&env, &owner, AccountScope::Seller, "0010", 0, 5);
    assert_eq!(hits.first().map(String::as_str), Some("0010"));
}
