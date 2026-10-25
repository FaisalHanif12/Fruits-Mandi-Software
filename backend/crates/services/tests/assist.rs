//! The assistance tests of PRD Section 27 (Code Standards 9.2), backend parts. Plan 07 adds
//! E2E-23 (a new buyer typed in English letters). Plan 08 adds the short codes and the party
//! card of E2E-24 (the open-bill part of E2E-24 comes with the matching plans). Plans 12 and 13
//! add E2E-25 to E2E-27.
#![cfg(test)]

/// The worked day's parties, items and partners and the draft builders of plan 12.
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod bikri_support;
mod common;

use std::time::Instant;

use common::*;
use mandi_services::core::accounts::{
    AccountInput, AccountRow, AccountScope, AccountType, AccountsBalancesInput, BalanceSide,
    CardWarningKind, GlobalHit, OpeningLineInput, OpeningsInput, ResolveCodeInput, ResolveStatus,
    SearchAccountsInput, SearchGlobalInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::money::Paisa;
use mandi_services::core::search::SearchScope;
use mandi_services::core::text;
use mandi_services::core::translit::{Direction, Source, WordList};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, assist, auth, search, translit};

fn ready() -> (
    tempfile::TempDir,
    mandi_services::testing::TestEnv,
    auth::Actor,
) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

fn buyer_form(code: &str, ur: &str, en: &str) -> AccountInput {
    AccountInput {
        id: None,
        version: None,
        account_type: AccountType::Buyer,
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

#[test]
fn e2e_23_a_new_buyer_typed_in_english_letters_becomes_urdu_and_is_found_in_both_scripts() {
    // PRD Section 27, E2E-23, backend part. After the worked day the buyers are 03040001 اکرم
    // ٹول ٹیکس and 03040002 ارشد ملتانی.
    let (_dir, env, owner) = ready();
    for (ur, en) in [
        ("اکرم ٹول ٹیکس", "Akram Toll Tax"),
        ("ارشد ملتانی", "Arshad Multani"),
    ] {
        accounts::save(&env.ctx, &owner, &buyer_form("", ur, en)).unwrap();
    }

    // The user types `tariq mehmood` in the Urdu name field: the first suggestion is the one
    // that is taken on Enter, and it is not a rules-only guess.
    let up = translit::suggest(&env.ctx, "tariq mehmood", Direction::ToUrdu).unwrap();
    let first = up.first().expect("a suggestion");
    assert_eq!(first.text, "طارق محمود");
    assert_ne!(
        first.source,
        Source::Rules,
        "both words are in the word list"
    );

    // The English name is filled by itself from the Urdu name, with capital first letters.
    let down = translit::suggest(&env.ctx, &first.text, Direction::ToEnglish).unwrap();
    assert_eq!(down.first().map(|s| s.text.as_str()), Some("Tariq Mehmood"));

    // Save with the code left empty: the next free code, with no spelling confirmation needed.
    let saved = accounts::save(
        &env.ctx,
        &owner,
        &buyer_form("", &first.text, "Tariq Mehmood"),
    )
    .unwrap();
    assert_eq!(saved.code, "03040003");
    assert_eq!(saved.name_ur, "طارق محمود");
    assert_eq!(saved.name_en, "Tariq Mehmood");

    // `tarq` in a buyer picker finds him; so does the Urdu name, and a field that takes any
    // account lists him with the others.
    let by_roman = search::search(&env.ctx, &owner, SearchScope::Buyer, "tarq", 10).unwrap();
    assert_eq!(
        by_roman.iter().map(|h| h.id).collect::<Vec<_>>(),
        [saved.id]
    );
    assert_eq!(by_roman[0].code, "03040003");
    assert_eq!(by_roman[0].name_en, "Tariq Mehmood");
    assert_eq!(by_roman[0].name_ur, "طارق محمود");
    let by_urdu = search::search(&env.ctx, &owner, SearchScope::Buyer, "طارق", 10).unwrap();
    assert_eq!(by_urdu.iter().map(|h| h.id).collect::<Vec<_>>(), [saved.id]);
    let any = search::search(&env.ctx, &owner, SearchScope::AnyAccount, "tarq", 10).unwrap();
    assert!(any.iter().any(|h| h.id == saved.id));
    // He is a buyer only: a seller picker does not offer him.
    assert!(
        search::search(&env.ctx, &owner, SearchScope::Seller, "tarq", 10)
            .unwrap()
            .is_empty()
    );
}

// ---- Plan 08: a code brings the whole party ----

fn form(kind: AccountType, code: &str, ur: &str, en: &str) -> AccountInput {
    AccountInput {
        account_type: kind,
        ..buyer_form(code, ur, en)
    }
}

fn add(env: &TestEnv, owner: &auth::Actor, kind: AccountType, ur: &str, en: &str) -> AccountRow {
    accounts::save(&env.ctx, owner, &form(kind, "", ur, en)).unwrap()
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn resolve(
    env: &TestEnv,
    scope: AccountScope,
    text: &str,
) -> mandi_services::core::accounts::ResolvedCode {
    search::resolve_code(
        &env.ctx,
        &ResolveCodeInput {
            scope,
            control_id: None,
            text: text.into(),
        },
    )
    .unwrap()
}

fn find(
    env: &TestEnv,
    owner: &auth::Actor,
    scope: AccountScope,
    text: &str,
) -> Vec<mandi_services::core::accounts::AccountHit> {
    search::search_accounts(
        &env.ctx,
        owner,
        &SearchAccountsInput {
            scope,
            control_id: None,
            text: text.into(),
            limit: 20,
        },
    )
    .unwrap()
}

/// The worked day's parties: sellers Rahim and Karim, buyers Akram and Arshad.
fn the_parties(env: &TestEnv, owner: &auth::Actor) -> [AccountRow; 4] {
    [
        add(env, owner, AccountType::Seller, "رحیم", "Rahim"),
        add(env, owner, AccountType::Seller, "کریم", "Karim"),
        add(
            env,
            owner,
            AccountType::Buyer,
            "اکرم ٹول ٹیکس",
            "Akram Toll Tax",
        ),
        add(
            env,
            owner,
            AccountType::Buyer,
            "ارشد ملتانی",
            "Arshad Multani",
        ),
    ]
}

#[test]
fn a_short_code_in_a_buyer_field_brings_the_buyer_after_enter() {
    // Plan 08 Section 3, rows 1, 2 and 5.
    let (_dir, env, owner) = ready();
    let [_, _, akram, _] = the_parties(&env, &owner);
    for typed in ["1", "0001", " 1 ", "01"] {
        let found = resolve(&env, AccountScope::Buyer, typed);
        assert_eq!(found.status, ResolveStatus::Found, "{typed}");
        let hit = found.account.expect("the account");
        assert_eq!((hit.id, hit.code.as_str()), (akram.id, "03040001"));
        assert_eq!(hit.name_ur, "اکرم ٹول ٹیکس");
        assert_eq!(hit.account_type, AccountType::Buyer);
        assert!(found.candidates.is_empty() && found.suggested_code.is_none());
    }
    // While typing, the list previews the top match; `1` is not yet a taken code. The exact
    // serial is first, and `12` is a different serial, so it does not bring Akram.
    let preview = find(&env, &owner, AccountScope::Buyer, "1");
    assert_eq!(preview.first().map(|h| h.id), Some(akram.id));
    assert!(
        find(&env, &owner, AccountScope::Buyer, "12")
            .iter()
            .all(|h| h.id != akram.id)
    );
    assert_eq!(
        resolve(&env, AccountScope::Buyer, "12").status,
        ResolveStatus::Unknown
    );
    // The full code works too, and a seller's code in a buyer field is not a buyer.
    assert_eq!(
        resolve(&env, AccountScope::Buyer, "03040001")
            .account
            .map(|a| a.id),
        Some(akram.id)
    );
    let wrong_series = resolve(&env, AccountScope::Buyer, "030110001");
    assert_eq!(
        (wrong_series.status, wrong_series.suggested_code),
        (ResolveStatus::Unknown, None)
    );
}

#[test]
fn a_short_code_in_a_seller_field_and_an_unknown_one() {
    // Plan 08 Section 3, row 3: `7` in a seller field is 030110007 if it exists, else the
    // unknown-code path for that code.
    let (_dir, env, owner) = ready();
    the_parties(&env, &owner);
    let unknown = resolve(&env, AccountScope::Seller, "7");
    assert_eq!(unknown.status, ResolveStatus::Unknown);
    assert_eq!(unknown.suggested_code.as_deref(), Some("030110007"));
    assert!(unknown.account.is_none());
    accounts::save(
        &env.ctx,
        &owner,
        &form(AccountType::Seller, "030110007", "ناصر", "Nasir"),
    )
    .unwrap();
    let found = resolve(&env, AccountScope::Seller, "7");
    assert_eq!(found.status, ResolveStatus::Found);
    assert_eq!(found.account.map(|a| a.code), Some("030110007".to_owned()));
    // `1` in a seller field is Rahim, not the buyer with serial 1.
    assert_eq!(
        resolve(&env, AccountScope::Seller, "1")
            .account
            .map(|a| a.code),
        Some("030110001".to_owned())
    );
    let hits = find(&env, &owner, AccountScope::Seller, "1");
    assert!(hits.iter().all(|h| h.account_type == AccountType::Seller));
}

#[test]
fn a_field_for_any_account_lists_the_seller_and_the_buyer_with_that_serial() {
    // Plan 08 Section 3, row 4 (E2E-24, second part).
    let (_dir, env, owner) = ready();
    let [rahim, _, akram, _] = the_parties(&env, &owner);
    let several = resolve(&env, AccountScope::Any, "1");
    assert_eq!(several.status, ResolveStatus::Several);
    let listed: Vec<(i64, AccountType, &str)> = several
        .candidates
        .iter()
        .map(|c| (c.id, c.account_type, c.code.as_str()))
        .collect();
    assert_eq!(listed.len(), 2);
    assert!(listed.contains(&(akram.id, AccountType::Buyer, "03040001")));
    assert!(listed.contains(&(rahim.id, AccountType::Seller, "030110001")));
    // The search lists both too, in the same rows.
    let ids: Vec<i64> = find(&env, &owner, AccountScope::Any, "1")
        .iter()
        .take(2)
        .map(|h| h.id)
        .collect();
    assert!(ids.contains(&akram.id) && ids.contains(&rahim.id));
    // A serial that only a buyer has is one account, not several.
    let only_buyer = resolve(&env, AccountScope::Any, "2");
    assert_eq!(
        only_buyer.status,
        ResolveStatus::Several,
        "Karim and Arshad share serial 2"
    );
    add(&env, &owner, AccountType::Buyer, "ناصر", "Nasir");
    add(
        &env,
        &owner,
        AccountType::Buyer,
        "طارق محمود",
        "Tariq Mehmood",
    );
    assert_eq!(
        resolve(&env, AccountScope::Any, "4").status,
        ResolveStatus::Found
    );
}

#[test]
fn reserved_serials_are_never_a_short_code() {
    // Plan 08 Section 3, row 6: 9999 is the system's, never a normal account's short code.
    let (_dir, env, owner) = ready();
    the_parties(&env, &owner);
    for typed in ["9999", "0000", "9000", "0", "8999x"] {
        let status = resolve(&env, AccountScope::Buyer, typed).status;
        let want = if typed == "8999x" {
            ResolveStatus::Unknown
        } else {
            ResolveStatus::Reserved
        };
        assert_eq!(status, want, "{typed}");
    }
    // 8999 is the last normal serial.
    assert_eq!(
        resolve(&env, AccountScope::Buyer, "8999").status,
        ResolveStatus::Unknown
    );
    // The system buyer پختہ کھاتہ (serial 9999) is not found by the short code, and is found by
    // its full code, as the buyer field must offer it.
    assert!(
        find(&env, &owner, AccountScope::Buyer, "9999")
            .iter()
            .all(|h| h.code != "03049999")
    );
    let pukhta = resolve(&env, AccountScope::Buyer, "03049999");
    assert_eq!(pukhta.status, ResolveStatus::Found);
    assert_eq!(
        resolve(&env, AccountScope::Buyer, "03040000").status,
        ResolveStatus::Found
    );
}

#[test]
fn a_full_code_that_does_not_exist_offers_to_add_it() {
    // Plan 08 Section 3, row 7: "اس کوڈ کا کوئی گاہک نہیں" with an F4 add button.
    let (_dir, env, owner) = ready();
    the_parties(&env, &owner);
    let unknown = resolve(&env, AccountScope::Buyer, "03045555");
    assert_eq!(unknown.status, ResolveStatus::Unknown);
    assert_eq!(unknown.suggested_code.as_deref(), Some("03045555"));
    // A reserved serial that no account has cannot be added by F4.
    assert_eq!(
        resolve(&env, AccountScope::Buyer, "03049500").status,
        ResolveStatus::Reserved
    );
    // Text that is not digits is a name, not a code.
    let name = resolve(&env, AccountScope::Buyer, "akram");
    assert_eq!(
        (name.status, name.suggested_code),
        (ResolveStatus::Unknown, None)
    );
    // 5 to 7 digits are not a code of any series.
    let odd = resolve(&env, AccountScope::Buyer, "30400");
    assert_eq!(
        (odd.status, odd.suggested_code),
        (ResolveStatus::Unknown, None)
    );
    // The suggested code really is free: the add works with it.
    let added = accounts::save(
        &env.ctx,
        &owner,
        &form(AccountType::Buyer, "03045555", "ناصر", "Nasir"),
    )
    .unwrap();
    assert_eq!(added.code, "03045555");
    assert_eq!(
        resolve(&env, AccountScope::Buyer, "03045555").status,
        ResolveStatus::Found
    );
}

#[test]
fn an_inactive_account_is_named_but_not_taken() {
    let (_dir, env, owner) = ready();
    let [_, _, akram, _] = the_parties(&env, &owner);
    accounts::set_active(&env.ctx, &owner, akram.id, false).unwrap();
    let found = resolve(&env, AccountScope::Buyer, "1");
    assert_eq!(found.status, ResolveStatus::Inactive);
    assert_eq!(
        found.account.map(|a| (a.id, a.is_active)),
        Some((akram.id, false))
    );
    assert_eq!(
        resolve(&env, AccountScope::Buyer, "03040001").status,
        ResolveStatus::Inactive
    );
    assert!(
        find(&env, &owner, AccountScope::Buyer, "1")
            .iter()
            .all(|h| h.id != akram.id)
    );
}

#[test]
fn a_field_for_one_control_has_no_short_codes_but_finds_by_full_code_and_name() {
    // PRD FR-7.39: other accounts are found by full code or name.
    let (_dir, env, owner) = ready();
    let person = add(&env, &owner, AccountType::MoneyLent, "ناصر", "Nasir");
    assert_eq!(person.code, "01030001");
    let control = person.control.id;
    let ask = |text: &str| {
        search::resolve_code(
            &env.ctx,
            &ResolveCodeInput {
                scope: AccountScope::Control,
                control_id: Some(control),
                text: text.into(),
            },
        )
        .unwrap()
    };
    assert_eq!(ask("01030001").status, ResolveStatus::Found);
    assert_eq!(
        ask("1").status,
        ResolveStatus::Unknown,
        "no short code for this control"
    );
    let next = ask("01030002");
    assert_eq!(
        (next.status, next.suggested_code.as_deref()),
        (ResolveStatus::Unknown, Some("01030002"))
    );
    // The picker of that control sees only its accounts.
    let hits = search::search_accounts(
        &env.ctx,
        &owner,
        &SearchAccountsInput {
            scope: AccountScope::Control,
            control_id: Some(control),
            text: "nasir".into(),
            limit: 10,
        },
    )
    .unwrap();
    assert_eq!(hits.iter().map(|h| h.id).collect::<Vec<_>>(), [person.id]);
    add(&env, &owner, AccountType::Buyer, "ناصر خان", "Nasir Khan");
    assert_eq!(find(&env, &owner, AccountScope::Any, "nasir").len(), 2);
    // A control scope needs its control.
    let error = search::search_accounts(
        &env.ctx,
        &owner,
        &SearchAccountsInput {
            scope: AccountScope::Control,
            control_id: None,
            text: "x".into(),
            limit: 5,
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrRequired);
}

#[test]
fn text_limits_and_empty_codes_are_refused() {
    let (_dir, env, owner) = ready();
    let long = "a".repeat(201);
    let error = search::search_accounts(
        &env.ctx,
        &owner,
        &SearchAccountsInput {
            scope: AccountScope::Any,
            control_id: None,
            text: long.clone(),
            limit: 5,
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrTextTooLong);
    let error = search::resolve_code(
        &env.ctx,
        &ResolveCodeInput {
            scope: AccountScope::Buyer,
            control_id: None,
            text: "  ".into(),
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrRequired);
    assert!(find(&env, &owner, AccountScope::Any, "  ").is_empty());
}

#[test]
fn row_balances_and_global_search() {
    let (_dir, env, owner) = ready();
    let [rahim, _, akram, arshad] = the_parties(&env, &owner);
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![
                OpeningLineInput {
                    account_id: akram.id,
                    version: akram.version,
                    opening_date: "09-10-2026".into(),
                    opening_dr: "12,880".into(),
                    opening_cr: String::new(),
                    opening_is_advance: false,
                },
                OpeningLineInput {
                    account_id: arshad.id,
                    version: arshad.version,
                    opening_date: "09-10-2026".into(),
                    opening_dr: String::new(),
                    opening_cr: "3000".into(),
                    opening_is_advance: false,
                },
            ],
        },
    )
    .unwrap();
    let rows = search::balances(&env.ctx, &[akram.id, arshad.id, rahim.id, 999_999]).unwrap();
    let shown: Vec<(i64, BalanceSide, i64)> = rows
        .iter()
        .map(|r| {
            (
                r.id,
                r.balance.side,
                r.balance.amount_paisa.as_paisa() / 100,
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            (akram.id, BalanceSide::Dr, 12_880),
            (arshad.id, BalanceSide::Cr, 3_000),
            (rahim.id, BalanceSide::Zero, 0),
            (999_999, BalanceSide::Zero, 0)
        ]
    );
    assert!(search::balances(&env.ctx, &[]).unwrap().is_empty());
    let too_many: Vec<i64> = (1..=501).collect();
    let error = search::balances(&env.ctx, &too_many).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrOutOfRange);
    let _ = AccountsBalancesInput { ids: vec![] };

    // Global search finds accounts by name in either script and by short code.
    let by_name = search::global(
        &env.ctx,
        &owner,
        &SearchGlobalInput {
            text: "akrm".into(),
            limit: 10,
        },
    )
    .unwrap();
    assert!(matches!(by_name.first(), Some(GlobalHit::Account { hit }) if hit.id == akram.id));
    let by_code = search::global(
        &env.ctx,
        &owner,
        &SearchGlobalInput {
            text: "1".into(),
            limit: 10,
        },
    )
    .unwrap();
    assert!(by_code.len() >= 2);
    let by_urdu = search::global(
        &env.ctx,
        &owner,
        &SearchGlobalInput {
            text: "ارشد".into(),
            limit: 10,
        },
    )
    .unwrap();
    assert!(matches!(by_urdu.first(), Some(GlobalHit::Account { hit }) if hit.id == arshad.id));
}

#[test]
fn the_party_card_of_a_buyer_shows_balance_credit_and_last_entry() {
    // PRD FR-7.38: a buyer's card, with his credit limit and how much of it is used.
    let (_dir, env, owner) = ready();
    let mut input = form(AccountType::Buyer, "", "اکرم ٹول ٹیکس", "Akram Toll Tax");
    input.phone1 = "0300-1234567".into();
    input.father_or_firm_en = "Ghulam Nabi".into();
    input.father_or_firm_ur = "غلام نبی".into();
    input.credit_limit = "20,000".into();
    input.opening_date = "09-10-2026".into();
    input.opening_dr = "12,880".into();
    let akram = accounts::save(&env.ctx, &owner, &input).unwrap();

    let card = assist::party_card(&env.ctx, akram.id).unwrap();
    assert_eq!(
        (card.code.as_str(), card.account_type),
        ("03040001", AccountType::Buyer)
    );
    assert_eq!(
        (card.name_en.as_str(), card.father_or_firm_en.as_str()),
        ("Akram Toll Tax", "Ghulam Nabi")
    );
    assert_eq!(card.phone, "03001234567");
    assert_eq!(
        (card.balance.side, card.balance.amount_paisa),
        (BalanceSide::Dr, rupees(12_880))
    );
    assert_eq!(card.last_entry_date.as_deref(), Some("2026-10-09"));
    assert_eq!(
        (card.credit_limit, card.credit_used),
        (Some(rupees(20_000)), Some(rupees(12_880)))
    );
    assert_eq!(card.advance_open, None);
    assert!(card.warnings.is_empty() && card.is_active);

    // Owing 25,000 against a 20,000 limit: over by 5,000.
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: akram.id,
                version: accounts::get(&env.ctx, akram.id).unwrap().version,
                opening_date: "09-10-2026".into(),
                opening_dr: "25000".into(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    let card = assist::party_card(&env.ctx, akram.id).unwrap();
    assert_eq!(card.credit_used, Some(rupees(25_000)));
    assert_eq!(card.warnings.len(), 1);
    assert_eq!(card.warnings[0].kind, CardWarningKind::OverCreditLimit);
    assert_eq!(card.warnings[0].amount_paisa, Some(rupees(5_000)));
    // A credit balance means he owes nothing: used is 0.
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: akram.id,
                version: accounts::get(&env.ctx, akram.id).unwrap().version,
                opening_date: "09-10-2026".into(),
                opening_dr: String::new(),
                opening_cr: "700".into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    let card = assist::party_card(&env.ctx, akram.id).unwrap();
    assert_eq!(
        (card.balance.side, card.balance.amount_paisa),
        (BalanceSide::Cr, rupees(700))
    );
    assert_eq!(card.credit_used, Some(rupees(0)));
    assert!(card.warnings.is_empty());
    // Inactive shows its own warning.
    accounts::set_active(&env.ctx, &owner, akram.id, false).unwrap();
    let card = assist::party_card(&env.ctx, akram.id).unwrap();
    assert_eq!(
        card.warnings.iter().map(|w| w.kind).collect::<Vec<_>>(),
        [CardWarningKind::Inactive]
    );
    assert!(!card.is_active);
}

#[test]
fn the_party_card_of_a_seller_shows_the_open_advance_and_what_he_owes_beyond_it() {
    // PRD 4.3: Rahim has an open advance of 15,000. FR-15.9: only what he owes beyond his open
    // advances is "owes the firm".
    let (_dir, env, owner) = ready();
    let mut input = form(AccountType::Seller, "", "رحیم", "Rahim");
    input.opening_date = "01-09-2026".into();
    input.opening_dr = "15000".into();
    input.opening_is_advance = true;
    let rahim = accounts::save(&env.ctx, &owner, &input).unwrap();
    let card = assist::party_card(&env.ctx, rahim.id).unwrap();
    assert_eq!(
        (card.balance.side, card.balance.amount_paisa),
        (BalanceSide::Dr, rupees(15_000))
    );
    assert_eq!(card.advance_open, Some(rupees(15_000)));
    assert_eq!((card.credit_limit, card.credit_used), (None, None));
    assert!(card.warnings.is_empty(), "his debit is exactly his advance");

    // 5,000 paid to him on top (a general debit, not an advance): he now owes 5,000 beyond it.
    use mandi_services::core::ids::AccountId;
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::{self, PostingEvent};
    let cash = env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, "01010001"))
        .unwrap()
        .unwrap();
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(rahim.id), Side::Dr, rupees(5_000)),
        LedgerLine::new(AccountId::new(cash), Side::Cr, rupees(5_000)),
    ])
    .unwrap();
    env.ctx
        .db()
        .write(|tx| {
            ledger::post(
                &env.ctx,
                tx,
                owner.user_id,
                &PostingEvent {
                    date: mandi_services::core::dates::BusinessDate::parse_iso("2026-10-10")
                        .unwrap(),
                    posting_type: PostingType::Payment,
                    source_type: SourceType::Voucher,
                    source_id: 1,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap();
    let card = assist::party_card(&env.ctx, rahim.id).unwrap();
    assert_eq!(
        (card.balance.side, card.balance.amount_paisa),
        (BalanceSide::Dr, rupees(20_000))
    );
    assert_eq!(card.advance_open, Some(rupees(15_000)));
    assert_eq!(card.last_entry_date.as_deref(), Some("2026-10-10"));
    assert_eq!(card.warnings.len(), 1);
    assert_eq!(card.warnings[0].kind, CardWarningKind::OwesFirm);
    assert_eq!(card.warnings[0].amount_paisa, Some(rupees(5_000)));

    // Clearing the opening reverses the advance line: the advance is gone with it.
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: rahim.id,
                version: accounts::get(&env.ctx, rahim.id).unwrap().version,
                opening_date: String::new(),
                opening_dr: String::new(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    let card = assist::party_card(&env.ctx, rahim.id).unwrap();
    assert_eq!(card.advance_open, Some(rupees(0)));
    assert_eq!(
        (card.balance.side, card.balance.amount_paisa),
        (BalanceSide::Dr, rupees(5_000))
    );
    assert_eq!(card.warnings[0].amount_paisa, Some(rupees(5_000)));
}

#[test]
fn a_card_for_a_missing_account_and_for_another_kind_of_account() {
    let (_dir, env, owner) = ready();
    let error = assist::party_card(&env.ctx, 999_999).unwrap_err();
    let ServiceError::Rejected(items) = error else {
        panic!("not a rejection")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrNotFound, Some("account_id"))
    );
    let cash = env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, "01010001"))
        .unwrap()
        .unwrap();
    let card = assist::party_card(&env.ctx, cash).unwrap();
    assert_eq!(card.account_type, AccountType::Other);
    assert_eq!(
        (card.credit_limit, card.credit_used, card.advance_open),
        (None, None, None)
    );
    assert_eq!(card.last_entry_date, None);
    assert_eq!(card.balance.side, BalanceSide::Zero);
    let _ = (&owner, ErrorCode::ErrInternal);
}

#[test]
fn twenty_thousand_accounts_are_searched_and_resolved_fast() {
    // NFR-SPD-6 for the account scopes: a generous bound for a debug build on any computer;
    // `pnpm perf` holds the real number.
    const BOUND_MS: u128 = 600;
    let (dir, env, owner) = ready();
    let list = WordList::embedded();
    let words: Vec<(&str, &str)> = list
        .entries()
        .iter()
        .filter(|e| !e.roman.contains(' ') && e.kind == mandi_services::core::translit::Kind::Name)
        .take(3_000)
        .map(|e| (e.roman.as_str(), e.urdu.as_str()))
        .collect();
    let capital = |w: &str| {
        let mut c = w.chars();
        c.next().map_or(String::new(), |f| {
            f.to_uppercase().collect::<String>() + c.as_str()
        })
    };
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
        let (sellers, buyers, others) = (control("03011"), control("0304"), control("0101"));
        conn.execute_batch("BEGIN").unwrap();
        let mut statement = conn
            .prepare(
                "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur, search_en, search_phonetic)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .unwrap();
        let mut n = 0_usize;
        let plan = [
            (sellers, "03011", "seller", 8_999_usize),
            (buyers, "0304", "buyer", 8_999),
            (others, "0101", "other", 2_100),
        ];
        for (control_id, code, party, count) in plan {
            for serial in (if code == "0101" { 2 } else { 1 })..=count {
                let (a, b) = (words[n % words.len()], words[(n * 7 + 13) % words.len()]);
                let en = format!("{} {}", capital(a.0), capital(b.0));
                let ur = format!("{} {}", a.1, b.1);
                statement
                    .execute((
                        control_id,
                        format!("{code}{serial:04}"),
                        i64::try_from(serial).unwrap(),
                        party,
                        &ur,
                        &en,
                        text::search_ur(&ur),
                        text::search_en(&en),
                        text::search_phonetic(&en),
                    ))
                    .unwrap();
                n += 1;
            }
        }
        drop(statement);
        conn.execute_batch("COMMIT").unwrap();
        assert!(n > 20_000);
    }
    let started = Instant::now();
    search::rebuild(&env.ctx).unwrap();
    let build = started.elapsed();
    let (en, ur) = env
        .ctx
        .db()
        .read(|c| {
            c.query_row(
                "SELECT name_en, name_ur FROM accounts WHERE code = '03040500'",
                [],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
        })
        .unwrap();
    let word = en.split(' ').next().unwrap().to_lowercase();
    let mistaken: String = {
        let mut chars: Vec<char> = word.chars().collect();
        if chars.len() > 3 {
            chars.remove(2);
        }
        chars.into_iter().collect()
    };
    let urdu_word = ur.split(' ').next().unwrap().to_owned();
    let mut slowest = 0;
    for (scope, typed) in [
        (AccountScope::Buyer, "500"),
        (AccountScope::Buyer, word.as_str()),
        (AccountScope::Buyer, mistaken.as_str()),
        (AccountScope::Buyer, urdu_word.as_str()),
        (AccountScope::Any, "7"),
        (AccountScope::Seller, "8999"),
        (AccountScope::Any, "zzzqqq"),
        (AccountScope::Any, "03040500"),
    ] {
        let t = Instant::now();
        let hits = find(&env, &owner, scope, typed);
        let took = t.elapsed().as_millis();
        slowest = slowest.max(took);
        assert!(took < BOUND_MS, "{typed}: {took} ms");
        if typed == "500" {
            assert_eq!(hits.first().map(|h| h.code.as_str()), Some("03040500"));
        }
    }
    for (scope, typed) in [
        (AccountScope::Buyer, "500"),
        (AccountScope::Any, "7"),
        (AccountScope::Seller, "03011042"),
    ] {
        let t = Instant::now();
        resolve(&env, scope, typed);
        let took = t.elapsed().as_millis();
        slowest = slowest.max(took);
        assert!(took < BOUND_MS, "resolve {typed}: {took} ms");
    }
    // Balances for a page of rows.
    let ids: Vec<i64> = (1..=50).collect();
    let t = Instant::now();
    search::balances(&env.ctx, &ids).unwrap();
    assert!(t.elapsed().as_millis() < BOUND_MS);
    eprintln!("index built in {build:?}; slowest account search {slowest} ms (debug build)");
    // 9,000 is reserved and 8,999 is the last serial: the generated series is full.
    let (code, _) = common::code_and_value(
        &accounts::next_code(&env.ctx, AccountType::Buyer, None).unwrap_err(),
        "x",
    );
    assert_eq!(code, ErrorCode::ErrSeriesFull);
    let _ = Source::Rules;
}

#[test]
fn the_card_of_an_account_with_ten_years_of_entries_is_fast() {
    // NFR-SPD-21: under 100 ms with 10 years of data. 10,000 entries on one buyer is about
    // 3 a day for 10 years. A generous bound for a debug build; `pnpm perf` holds the number.
    const BOUND_MS: u128 = 300;
    let (dir, env, _owner) = ready();
    let akram = {
        let owner = auth::require(&env.ctx).unwrap();
        add(
            &env,
            &owner,
            AccountType::Buyer,
            "اکرم ٹول ٹیکس",
            "Akram Toll Tax",
        )
    };
    let cash = env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, "01010001"))
        .unwrap()
        .unwrap();
    {
        let conn = raw(&dir);
        conn.execute_batch("BEGIN").unwrap();
        for i in 0..10_000_i64 {
            let date = format!(
                "{}-{:02}-{:02}",
                2016 + i / 1_000,
                1 + (i / 28) % 12,
                1 + i % 28
            );
            conn.execute(
                "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id) VALUES (?1, 'general', 'voucher', ?2)",
                (&date, i),
            )
            .unwrap();
            let entry = conn.last_insert_rowid();
            let (buyer_dr, buyer_cr) = if i % 3 == 0 {
                (0, 100_000)
            } else {
                (1_000_000, 0)
            };
            conn.execute(
                "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, 1, ?2, ?3, ?4)",
                (entry, akram.id, buyer_dr, buyer_cr),
            )
            .unwrap();
            conn.execute(
                "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, 2, ?2, ?3, ?4)",
                (entry, cash, buyer_cr, buyer_dr),
            )
            .unwrap();
            conn.execute(
                "UPDATE journal_entries SET status = 'posted', total_paisa = ?2 WHERE id = ?1",
                (entry, buyer_dr + buyer_cr),
            )
            .unwrap();
        }
        conn.execute_batch("COMMIT").unwrap();
    }
    let t = Instant::now();
    let card = assist::party_card(&env.ctx, akram.id).unwrap();
    let took = t.elapsed().as_millis();
    eprintln!("party card over 10,000 entries: {took} ms (debug build)");
    assert!(took < BOUND_MS, "{took} ms");
    // 6,666 debits of 10,000 rupees and 3,334 credits of 1,000 rupees.
    assert_eq!(
        (card.balance.side, card.balance.amount_paisa),
        (BalanceSide::Dr, rupees(63_326_000))
    );
    assert!(card.last_entry_date.is_some());
}

#[test]
fn the_card_of_the_pukhta_khata_account_does_not_net_one_person_against_another() {
    // PRD FR-13.5: balances there are per person, so the account's card shows its balance and
    // no credit or advance figure.
    let (_dir, env, _owner) = ready();
    let pukhta = env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, "03049999"))
        .unwrap()
        .unwrap();
    let card = assist::party_card(&env.ctx, pukhta).unwrap();
    assert_eq!(card.account_type, AccountType::Buyer);
    assert_eq!(
        (card.credit_limit, card.credit_used, card.advance_open),
        (None, None, None)
    );
    assert!(card.warnings.is_empty());
    let seller = env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, "030119999"))
        .unwrap()
        .unwrap();
    let card = assist::party_card(&env.ctx, seller).unwrap();
    assert_eq!((card.credit_used, card.advance_open), (None, None));
    assert!(card.warnings.is_empty());
}

#[test]
fn an_adjustment_that_points_at_a_reversed_entry_does_not_lower_the_open_advance() {
    // The open advance counts the same lines on both sides: given and adjusted. If a matching
    // row was left active when its credit line was reversed, the card is still right.
    use mandi_services::core::ids::AccountId;
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::{self, PostingEvent};
    let (dir, env, owner) = ready();
    let mut input = form(AccountType::Seller, "", "رحیم", "Rahim");
    input.opening_date = "01-09-2026".into();
    input.opening_dr = "15000".into();
    input.opening_is_advance = true;
    let rahim = accounts::save(&env.ctx, &owner, &input).unwrap();
    let cash = env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, "01010001"))
        .unwrap()
        .unwrap();
    // 10,000 comes back from him (a credit on his account).
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(cash), Side::Dr, rupees(10_000)),
        LedgerLine::new(AccountId::new(rahim.id), Side::Cr, rupees(10_000)),
    ])
    .unwrap();
    let posted = env
        .ctx
        .db()
        .write(|tx| {
            ledger::post(
                &env.ctx,
                tx,
                owner.user_id,
                &PostingEvent {
                    date: mandi_services::core::dates::BusinessDate::parse_iso("2026-10-10")
                        .unwrap(),
                    posting_type: PostingType::Receipt,
                    source_type: SourceType::Voucher,
                    source_id: 1,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap();
    let conn = raw(&dir);
    let line_of = |entry_id: i64| -> i64 {
        conn.query_row(
            "SELECT id FROM journal_lines WHERE entry_id = ?1 AND account_id = ?2",
            (entry_id, rahim.id),
            |r| r.get(0),
        )
        .unwrap()
    };
    let opening_entry: i64 = conn
        .query_row(
            "SELECT opening_entry_id FROM accounts WHERE id = ?1",
            [rahim.id],
            |r| r.get(0),
        )
        .unwrap();
    conn.execute(
        "INSERT INTO advance_allocations (advance_line_id, credit_line_id, account_id, amount_paisa) VALUES (?1, ?2, ?3, 1000000)",
        (line_of(opening_entry), line_of(posted.entry_id), rahim.id),
    )
    .unwrap();
    let card = assist::party_card(&env.ctx, rahim.id).unwrap();
    assert_eq!(
        card.advance_open,
        Some(rupees(5_000)),
        "15,000 given, 10,000 adjusted"
    );

    // The credit entry is reversed but its matching row was not switched off.
    env.ctx
        .db()
        .write(|tx| ledger::reverse(&env.ctx, tx, owner.user_id, posted.entry_id, "wrong"))
        .unwrap();
    let card = assist::party_card(&env.ctx, rahim.id).unwrap();
    assert_eq!(
        card.advance_open,
        Some(rupees(15_000)),
        "the reversed credit adjusts nothing"
    );
}

#[test]
fn recent_accounts_come_first_in_every_field_and_each_control_has_its_own_list() {
    // PRD FR-7.41: `Alt+Down` shows the last 10 chosen, then the rest. Account fields get an
    // `AccountHit` (with the type); a control has its own list (plan 08, lead's addition).
    use mandi_services::core::accounts::{AccountsRecentInput, AccountsTouchInput};
    let (_dir, env, owner) = ready();
    let [rahim, karim, akram, arshad] = the_parties(&env, &owner);
    let recent = |scope: AccountScope, control_id: Option<i64>| {
        search::recent_accounts(&env.ctx, &owner, &AccountsRecentInput { scope, control_id })
            .unwrap()
            .into_iter()
            .map(|h| (h.id, h.account_type))
            .collect::<Vec<_>>()
    };
    let touch = |scope: AccountScope, control_id: Option<i64>, id: i64| {
        search::touch_account(
            &env.ctx,
            &owner,
            &AccountsTouchInput {
                scope,
                control_id,
                id,
            },
        )
    };
    // Nothing chosen yet: Urdu alphabetical order of the buyers, with their type.
    let before = recent(AccountScope::Buyer, None);
    assert_eq!(before.len(), 4, "Akram, Arshad, نقدی منڈی and پختہ کھاتہ");
    assert!(before.iter().all(|(_, t)| *t == AccountType::Buyer));
    // Choose Arshad then Akram: the latest first.
    touch(AccountScope::Buyer, None, arshad.id).unwrap();
    touch(AccountScope::Buyer, None, akram.id).unwrap();
    let after = recent(AccountScope::Buyer, None);
    assert_eq!(
        after[..2],
        [
            (akram.id, AccountType::Buyer),
            (arshad.id, AccountType::Buyer)
        ]
    );
    // The seller list is separate; the any-account list is its own too.
    assert!(
        recent(AccountScope::Seller, None)
            .iter()
            .all(|(id, _)| *id != akram.id)
    );
    touch(AccountScope::Any, None, rahim.id).unwrap();
    assert_eq!(
        recent(AccountScope::Any, None)[0],
        (rahim.id, AccountType::Seller)
    );
    // A control has its own list, and the search puts a chosen account first among equals.
    let person = add(&env, &owner, AccountType::MoneyLent, "ناصر", "Nasir");
    let other = add(
        &env,
        &owner,
        AccountType::MoneyLent,
        "ناصر علی",
        "Nasir Ali",
    );
    let control = Some(person.control.id);
    touch(AccountScope::Control, control, other.id).unwrap();
    let list = recent(AccountScope::Control, control);
    assert_eq!(list[0], (other.id, AccountType::MoneyLent));
    assert_eq!(list.len(), 2);
    let hits = search::search_accounts(
        &env.ctx,
        &owner,
        &SearchAccountsInput {
            scope: AccountScope::Control,
            control_id: control,
            text: "nas".into(),
            limit: 5,
        },
    )
    .unwrap();
    assert_eq!(
        hits[0].id, other.id,
        "used before comes first among equal matches"
    );
    // Wrong things are refused: an account that is not in the field's list, and no control.
    let error = touch(AccountScope::Buyer, None, rahim.id).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrNotFound);
    let error = touch(AccountScope::Control, control, rahim.id).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrNotFound);
    let error = touch(AccountScope::Control, None, rahim.id).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrRequired);
    let error = search::recent_accounts(
        &env.ctx,
        &owner,
        &AccountsRecentInput {
            scope: AccountScope::Control,
            control_id: None,
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrRequired);
    let _ = karim;
}

// ---- Plan 12: E2E-26, the backend part ----

#[test]
fn e2e_26_a_new_row_starts_at_the_remaining_qty_with_the_last_rate_as_a_hint() {
    // PRD E2E-26: in بکری 1, after اکرم's line (20 of 50), ارشد's line starts at 30, the rate
    // field is empty with the hint 600, and the commission of line 1 works out as
    // "12,000 x 6 / 100 = 720". `Ctrl+D` and the keys are the frontend's part.
    use bikri_support::*;
    use mandi_services::core::bikri::RateHintSource;
    use mandi_services::core::rules::WorkingValue;
    let w = world();
    let d = draft(vec![with_sales(
        lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![sale("s-1", w.akram, "20", "600"), blank_sale("s-2", "30")],
    )]);
    let p = preview(&w, &d);
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let next = &p.lots[0].lines[1];
    assert_eq!(next.remaining_qty, 30, "ارشد's line starts at 30");
    let hint = next.rate_hint.as_ref().expect("the hint");
    assert_eq!(
        (r(hint.rate_paisa), hint.source),
        (600, RateHintSource::ThisBikri)
    );
    assert!(
        next.calc.is_none(),
        "the rate is empty, so nothing is worked out for it"
    );
    // The working of line 1's seller commission, as the screen shows it.
    let working = &calc(&p.lots[0], 0).seller_comm.working;
    let money = |name: &str| {
        working
            .args
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.value)
    };
    assert_eq!(working.message_id, "working-percent");
    assert_eq!(
        money("base"),
        Some(WorkingValue::Money {
            paisa: rupees(12_000)
        })
    );
    assert_eq!(money("bps"), Some(WorkingValue::Percent { bps: 600 }));
    assert_eq!(
        money("result"),
        Some(WorkingValue::Money { paisa: rupees(720) })
    );
}

// ---- Plan 13: E2E-25 and E2E-26 on saved data ----

/// A saved بکری 1 with the truck LES-1234 from سوات اڈا (Event 1), saved through the real save.
fn saved_event_1_with_adda() -> (bikri_support::World, i64) {
    use bikri_support::*;
    use mandi_services::core::masters::{AddaInput, MasterInput};
    let w = world();
    let adda = mandi_services::masters::save(
        &w.env.ctx,
        &w.owner,
        &MasterInput::Adda(AddaInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: "سوات اڈا".into(),
            name_en: "Swat Adda".into(),
            city_id: None,
            confirmed_rules_words: vec!["سوات".into()],
        }),
    )
    .map(|row| row.id());
    // The spelling check may ask for the words of the name; confirm them like the screen does.
    let adda = match adda {
        Ok(id) => id,
        Err(ServiceError::Rejected(items)) => {
            let words: Vec<String> = items
                .iter()
                .filter_map(|i| match i.values.get("words") {
                    Some(mandi_services::core::error::ErrorValue::Text(t)) => Some(t.clone()),
                    _ => None,
                })
                .flat_map(|t| t.split(' ').map(str::to_owned).collect::<Vec<_>>())
                .collect();
            mandi_services::masters::save(
                &w.env.ctx,
                &w.owner,
                &MasterInput::Adda(AddaInput {
                    id: None,
                    version: None,
                    code: String::new(),
                    name_ur: "سوات اڈا".into(),
                    name_en: "Swat Adda".into(),
                    city_id: None,
                    confirmed_rules_words: words,
                }),
            )
            .unwrap()
            .id()
        }
        Err(other) => panic!("{other:?}"),
    };
    let mut d = event1(&w);
    d.header.adda_id = Some(adda);
    d.header.zamindar_account_id = Some(w.rahim);
    d.header.driver_name = Some("Aslam".into());
    mandi_services::bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: d,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    (w, adda)
}

#[test]
fn e2e_25_after_a_saved_bikri_the_same_truck_is_noticed_and_the_seller_is_remembered() {
    // PRD E2E-25 (backend part): بکری 1 is saved. A new بکری with the truck LES-1234 gets the
    // same-truck notice naming بکری 1; continuing fills the اڈا, the زمیندار and the driver; the
    // lot of رحیم fills کیلا and پیٹی.
    use mandi_services::core::bikri::TruckMemoryInput;
    let (w, adda) = saved_event_1_with_adda();
    let truck = assist::truck_memory(
        &w.env.ctx,
        &TruckMemoryInput {
            truck_no: "LES-1234".into(),
            date: "10-10-2026".into(),
        },
    )
    .unwrap();
    assert_eq!(truck.same_day.map(|d| d.bikri_number), Some(1));
    let last = truck.last.expect("the last بکری of the truck");
    assert_eq!(
        (
            last.bikri_number,
            last.adda_id,
            last.zamindar_account_id,
            last.driver_name.as_deref()
        ),
        (1, Some(adda), Some(w.rahim), Some("Aslam"))
    );
    // A short code `1` in a seller field is رحیم; his lot fills the item and the packing.
    let memory = assist::seller_memory(&w.env.ctx, w.rahim).unwrap();
    assert_eq!(
        (memory.item_id, memory.packing_id, memory.from_last_lot),
        (Some(w.banana), Some(w.box_), true)
    );
    // Another day: no notice, the memory stays.
    let tomorrow = assist::truck_memory(
        &w.env.ctx,
        &TruckMemoryInput {
            truck_no: "LES-1234".into(),
            date: "09-10-2026".into(),
        },
    )
    .unwrap();
    assert!(tomorrow.same_day.is_none() && tomorrow.last.is_some());
    // The notice's "کھولیں" finds بکری 1 by its number.
    let found = mandi_services::bikri::find(
        &w.env.ctx,
        &mandi_services::core::bikri::BikriFindInput {
            number: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(found.len(), 1);
}

#[test]
fn e2e_26_on_saved_data_the_hint_is_the_last_rate_saved_today() {
    // PRD E2E-26 on saved data: after بکری 1 is saved (اکرم at 600, ارشد at 580), a new لاٹ of the
    // same item and packing shows today's last saved rate (580) as a grey hint, never as a
    // value, and the remaining qty of a fresh lot is its whole quantity.
    use bikri_support::*;
    use mandi_services::core::bikri::{LastRateInput, RateHintSource};
    let w = world();
    mandi_services::bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: event1(&w),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let hint = assist::last_rate(
        &w.env.ctx,
        &LastRateInput {
            item_id: w.banana,
            packing_id: w.box_,
            date: "10-10-2026".into(),
        },
    )
    .unwrap()
    .expect("a saved rate");
    assert_eq!(
        (r(hint.rate_paisa), hint.source),
        (580, RateHintSource::Today)
    );
    // The preview of a new بکری shows the same hint on its first row and starts it at 40.
    let d = draft(vec![with_sales(
        lot(&w, "lot-1", w.rahim, w.banana, "40"),
        vec![blank_sale("s-1", "40")],
    )]);
    let p = preview(&w, &d);
    let row = &p.lots[0].lines[0];
    assert_eq!(row.remaining_qty, 40);
    let shown = row.rate_hint.as_ref().expect("the hint");
    assert_eq!(
        (r(shown.rate_paisa), shown.source),
        (580, RateHintSource::Today)
    );
    assert!(
        row.calc.is_none(),
        "the hint is never a rate: nothing is worked out"
    );
    // The rate check now has data: 580 and 600 of 50 نگ are near an average of 588.
    assert!(
        p.warnings
            .iter()
            .all(|w| w.code != ErrorCode::ErrRateFarFromAverage)
    );
}

#[test]
fn e2e_27_after_event_2_the_payables_of_karim_are_his_net_of_bikri_2() {
    // PRD E2E-27: after Event 2 the payable postings of کریم show بکری 2, صافی بکری 10,160 and
    // open 10,160, and "پورا بقایا" gives 10,160. رحیم has his posting of بکری 1 beside it
    // (PRD FR-7.59, money agent 8.2a).
    use bikri_support::*;
    use mandi_services::core::bikri::{PaymentType, PukhtaInput};
    use mandi_services::core::vouchers::{LineSource, OpenPayablesInput, PaymentInput};
    let w = world();
    mandi_services::bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: event1(&w),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let mut line = sale("s-1", w.pukhta_buyer, "10", "1200");
    line.pukhta = Some(PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    line.payment_type = Some(PaymentType::Cash);
    let mut lot2 = lot(&w, "lot-2", w.karim, w.apple, "10");
    lot2.freight = Some("1,000".into());
    lot2.cash_paid = None;
    lot2.sales = vec![line];
    mandi_services::bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: draft(vec![lot2]),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();

    let p = mandi_services::vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: w.karim,
            person_key: None,
            include_settled: false,
        },
    )
    .unwrap();
    assert_eq!(p.postings.len(), 1);
    let posting = &p.postings[0];
    assert_eq!(
        (
            posting.source,
            posting.number,
            posting.credit_paisa.rupees(),
            posting.advance_adjusted_paisa.rupees(),
            posting.paid_paisa.rupees(),
            posting.still_open_paisa.rupees(),
            posting.entry_date.as_str()
        ),
        (
            LineSource::Bikri,
            Some(2),
            10_160,
            0,
            0,
            10_160,
            "2026-10-10"
        )
    );
    assert_eq!(p.open_paisa.rupees(), 10_160, "پورا بقایا");
    assert_eq!(
        (p.balance.side, p.balance.amount_paisa.rupees()),
        (BalanceSide::Cr, 10_160)
    );

    // Pay what the button gave; the payable is gone and the posting is paid.
    let pay = PaymentInput {
        date: "10-10-2026".into(),
        account_id: w.karim,
        person_key: None,
        amount: "10,160".into(),
        removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
        reason: String::new(),
        narration: String::new(),
        confirm_cash_below_zero: false,
    };
    mandi_services::vouchers::save_payment(&w.env.ctx, &w.owner, &pay).unwrap();
    let p = mandi_services::vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: w.karim,
            person_key: None,
            include_settled: true,
        },
    )
    .unwrap();
    assert_eq!(
        (p.open_paisa.rupees(), p.postings[0].paid_paisa.rupees()),
        (0, 10_160)
    );
    // The reads refuse a buyer for payables and a seller for bills.
    assert_eq!(
        mandi_services::vouchers::open_payables(
            &w.env.ctx,
            &OpenPayablesInput {
                seller_id: w.akram,
                person_key: None,
                include_settled: false
            }
        )
        .unwrap_err()
        .code(),
        ErrorCode::ErrNotASeller
    );
}

#[test]
fn e2e_24_after_the_worked_day_the_code_1_brings_akram_his_card_and_his_open_bill() {
    // PRD E2E-24, backend part (plan 17): after Events 1 to 5, `1` in a buyer field is 03040001
    // اکرم ٹول ٹیکس; the card shows 2,880 بنام; his oldest open bill is بکری 1 of 10-10-26, 0
    // days old, and "پورا بقایا" is 2,880. In a field for any account `1` lists both the buyer
    // 03040001 and the seller 030110001. Events 2, 4 and 5 do not touch Akram's account, so
    // Events 1 and 3 are enough to build his side of the day.
    use bikri_support::*;
    use mandi_services::core::vouchers::{LineSource, OpenBillsInput, ReceiptInput};
    let w = world();
    mandi_services::bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: event1(&w),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    mandi_services::vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: "10-10-2026".into(),
            account_id: w.akram,
            person_key: None,
            amount: "10,000".into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .unwrap();

    // `1` in a buyer field, taken on Enter.
    let found = resolve(&w.env, AccountScope::Buyer, "1");
    assert_eq!(found.status, ResolveStatus::Found);
    let account = found.account.unwrap();
    assert_eq!(
        (account.id, account.code.as_str(), account.name_ur.as_str()),
        (w.akram, "03040001", "اکرم ٹول ٹیکس")
    );

    // The card: 2,880 بنام.
    let card = assist::party_card(&w.env.ctx, account.id).unwrap();
    assert_eq!(
        (card.balance.side, card.balance.amount_paisa),
        (BalanceSide::Dr, rupees(2_880))
    );
    assert_eq!(card.last_entry_date.as_deref(), Some("2026-10-10"));

    // The oldest open bill: بکری 1 of 10-10-26, 0 days; "پورا بقایا" is 2,880.
    let bills = mandi_services::vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: account.id,
            person_key: None,
        },
    )
    .unwrap();
    let bill = &bills.bills[0];
    assert_eq!(
        (
            bill.source,
            bill.number,
            bill.entry_date.as_str(),
            bill.days,
            bill.open_paisa.rupees()
        ),
        (LineSource::Bikri, Some(1), "2026-10-10", 0, 2_880)
    );
    assert_eq!(bills.open_paisa.rupees(), 2_880);
    assert_eq!(bills.oldest_days, Some(0));

    // In a جنرل line (any account) `1` lists the buyer and the seller with their types.
    let several = resolve(&w.env, AccountScope::Any, "1");
    assert_eq!(several.status, ResolveStatus::Several);
    let listed: Vec<(i64, AccountType, &str)> = several
        .candidates
        .iter()
        .map(|c| (c.id, c.account_type, c.code.as_str()))
        .collect();
    assert_eq!(listed.len(), 2);
    assert!(listed.contains(&(w.akram, AccountType::Buyer, "03040001")));
    assert!(listed.contains(&(w.rahim, AccountType::Seller, "030110001")));
}
