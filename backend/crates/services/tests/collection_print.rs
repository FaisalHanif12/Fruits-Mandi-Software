//! The collection list as a print (اگراہی فہرست, PRD FR-15.6, FR-19.13; plan 17e, D4R-A12). The
//! list is read from a real database, exactly as the screen reads it, and printed from that same
//! value. The worked numbers are the PRD worked day after Events 1 to 5: اکرم 2,880 and
//! ارشد 18,684, together 21,564.
#![cfg(test)]

mod common;

use common::*;
use mandi_reports::print::collection_print_data;
use mandi_services::core::accounts::{AccountInput, AccountRow, AccountType};
use mandi_services::core::auth::Language;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::money::Paisa;
use mandi_services::core::printing::{PrintCollectionInput, PrintDoc, PrintInput, PrintTo};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, auth, printing};
use tempfile::TempDir;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    set_today(&env, 2026, 10, 10);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

fn form(ur: &str, en: &str, owes: &str, tag: &str) -> AccountInput {
    AccountInput {
        id: None,
        version: None,
        account_type: AccountType::Buyer,
        is_pukhta_party: false,
        control_id: None,
        code: String::new(),
        name_ur: ur.into(),
        name_en: en.into(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: "0300-1234567".into(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: "01-09-2026".into(),
        opening_dr: owes.into(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        is_active: true,
        note: String::new(),
        tag: tag.into(),
    }
}

/// Saves an account, confirming whatever spelling the check asks about.
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

/// The three buyers: اکرم, ارشد and ناصر. A weekday is not kept on an account any more (plan 53).
fn buyers(env: &TestEnv, owner: &auth::Actor) -> (AccountRow, AccountRow, AccountRow) {
    let akram = save(
        env,
        owner,
        form("اکرم ٹول ٹیکس", "Akram Toll Tax", "2,880", ""),
    );
    let arshad = save(
        env,
        owner,
        form("ارشد ملتانی", "Arshad Multani", "18,684", "k"),
    );
    let nasir = save(env, owner, form("ناصر", "Nasir", "5,000", ""));
    (akram, arshad, nasir)
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

#[test]
fn the_print_has_every_buyer_who_owes_and_a_total_equal_to_their_dues() {
    let (_dir, env, owner) = ready();
    let (akram, arshad, nasir) = buyers(&env, &owner);
    let (list, filters) = printing::collection_for_print(&env.ctx, None, None).unwrap();
    // The same list the screen reads.
    let screen = mandi_services::vouchers::collection_list(
        &env.ctx,
        &mandi_services::core::vouchers::CollectionListInput {
            search: None,
            collector_staff_id: None,
            city_id: None,
            // A print holds every buyer, so the screen is read the same way.
            offset: 0,
            limit: u32::MAX,
        },
    )
    .unwrap();
    assert_eq!(list, screen);
    for language in [Language::En, Language::Ur] {
        let data = collection_print_data(env.ctx.db(), &list, &filters, language).unwrap();
        let rows = &data.groups[0].rows;
        assert_eq!(
            rows.iter().map(|r| r.dues.as_str()).collect::<Vec<_>>(),
            vec!["2,880", "18,684", "5,000"]
        );
        assert_eq!(data.total.amount, "26,564");
        assert_eq!(data.buyer_count, "3");
        // Short codes: the running numbers of the buyers, 0020, 0021 and 0022 (plan 32).
        assert_eq!(rows[0].code, "0");
        assert_eq!(rows[1].code, "1");
        assert_eq!(rows[2].code, "2");
        // The tag of ارشد is its own field.
        assert_eq!(rows[0].tag, None);
        assert_eq!(rows[1].tag.as_deref(), Some("K"));
    }
    // Plan 53 T2: every active buyer who owes is on the list, so its total is what they owe
    // (here 26,564). It is the shop's اگراہی when no filter is set (PRD FR-19.9).
    let all: Paisa = [&akram, &arshad, &nasir]
        .iter()
        .map(|row| row.balance.amount_paisa)
        .try_fold(Paisa::ZERO, Paisa::checked_add)
        .unwrap();
    assert_eq!(all, rupees(26_564));
    assert_eq!(list.total_paisa, all);
}

#[test]
fn unknown_filters_are_refused_at_their_fields() {
    let (_dir, env, _owner) = ready();
    assert_eq!(
        first_error(printing::collection_for_print(&env.ctx, Some(999), None).unwrap_err()),
        (
            ErrorCode::ErrNotFound,
            Some("collector_staff_id".to_owned())
        )
    );
    assert_eq!(
        first_error(printing::collection_for_print(&env.ctx, None, Some(999)).unwrap_err()),
        (ErrorCode::ErrNotFound, Some("city_id".to_owned()))
    );
}

#[test]
fn the_plan_checks_copies_names_the_pdf_by_the_day_and_the_audit_row_records_the_print() {
    let (dir, env, owner) = ready();
    buyers(&env, &owner);
    let input = |copies: Option<&str>| PrintCollectionInput {
        search: None,
        collector_staff_id: None,
        city_id: None,
        language: Language::Ur,
        copies: copies.map(str::to_owned),
        to: PrintTo::Pdf,
    };
    let error = printing::plan_collection(&env.ctx, &input(Some("0"))).unwrap_err();
    assert_eq!(
        first_error(error),
        (ErrorCode::ErrOutOfRange, Some("copies".to_owned()))
    );
    let plan = printing::plan_collection(&env.ctx, &input(None)).unwrap();
    assert_eq!(plan.file_number, 20_261_010);
    assert_eq!(
        mandi_services::core::printing::pdf_file_name(
            PrintDoc::CollectionList,
            plan.file_number,
            plan.language
        ),
        "collection-list-20261010-ur.pdf"
    );
    assert_eq!(plan.copies, 1);
    let before = audit_count(&dir, "print");
    printing::record_collection(&env.ctx, &owner, &plan).unwrap();
    assert_eq!(audit_count(&dir, "print"), before + 1);
    let new_json: String = raw(&dir)
        .query_row(
            "SELECT new_json FROM audit_log WHERE action = 'print' AND entity = 'collection_list'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(new_json.contains("\"buyers\":3"), "{new_json}");
    assert!(new_json.contains("\"total_paisa\":2656400"), "{new_json}");
    assert_chain_ok(&dir);
}

#[test]
fn a_bikri_print_refuses_the_voucher_slip_and_the_collection_list() {
    let (_dir, env, _owner) = ready();
    for doc in [PrintDoc::ReceiptSlip, PrintDoc::CollectionList] {
        let error = printing::plan(
            &env.ctx,
            &PrintInput {
                bikri_id: 1,
                docs: vec![PrintDoc::SellerBill, doc],
                language: Language::Ur,
                copies: None,
                to: PrintTo::Pdf,
                only_buyer_id: None,
            },
        )
        .unwrap_err();
        assert_eq!(
            first_error(error),
            (ErrorCode::ErrFormatInvalid, Some("docs".to_owned())),
            "{doc:?}"
        );
    }
}

#[test]
fn a_search_keeps_the_buyers_it_names_and_the_print_and_totals_follow() {
    let (_dir, env, owner) = ready();
    buyers(&env, &owner);
    let all = printing::collection_for_print(&env.ctx, None, None)
        .unwrap()
        .0;
    assert_eq!(all.total_count, 3);
    // The screen's search and the print's search give the same list.
    let searched = |text: &str| {
        printing::collection_for_print_searching(&env.ctx, None, None, Some(text))
            .unwrap()
            .0
    };
    let arshad = searched("arshad");
    assert_eq!(arshad.total_count, 1);
    assert_eq!(arshad.groups[0].buyers[0].dues_paisa, rupees(18_684));
    // The group total and the grand total are over the buyers the search keeps.
    assert_eq!(arshad.group_totals.len(), 1);
    assert_eq!(arshad.group_totals[0].total_paisa, rupees(18_684));
    assert_eq!(arshad.total_paisa, rupees(18_684));
    // Urdu name, phone and code are searched too.
    assert_eq!(searched("اکرم").total_count, 1);
    assert_eq!(searched("1234567").total_count, 3);
    assert_eq!(searched("zzz").total_count, 0);
    assert_eq!(searched("zzz").total_paisa, rupees(0));
}

#[test]
fn the_print_job_and_the_print_data_have_the_list_the_screen_shows_for_the_same_search() {
    let (_dir, env, owner) = ready();
    buyers(&env, &owner);
    for text in ["arshad", "اکرم", "1234567", "zzz", ""] {
        let screen = mandi_services::vouchers::collection_list(
            &env.ctx,
            &mandi_services::core::vouchers::CollectionListInput {
                search: Some(text.to_owned()),
                collector_staff_id: None,
                city_id: None,
                offset: 0,
                // A print holds every buyer, so the screen is read with a page that holds them all.
                limit: u32::MAX,
            },
        )
        .unwrap();
        // The print job (what the printer or the PDF gets) is planned from the same text.
        let plan = printing::plan_collection(
            &env.ctx,
            &PrintCollectionInput {
                search: Some(text.to_owned()),
                collector_staff_id: None,
                city_id: None,
                language: Language::En,
                copies: None,
                to: PrintTo::Pdf,
            },
        )
        .unwrap();
        assert_eq!(plan.list, screen, "print job, search {text:?}");
        // The print data of the preview window is the same list.
        let (data_list, _) =
            printing::collection_for_print_searching(&env.ctx, None, None, Some(text)).unwrap();
        assert_eq!(data_list, screen, "print data, search {text:?}");
    }
}

#[test]
fn every_active_buyer_who_owes_is_listed_on_any_day_and_nobody_else() {
    // Plan 53 T2: no collection day. A buyer who owes and is active is listed; one who owes
    // nothing, one who is owed, and an inactive one are not. The cash buyer and the پختہ کھاتہ
    // account are never on it.
    let (_dir, env, owner) = ready();
    let (akram, arshad, nasir) = buyers(&env, &owner);
    save(&env, &owner, form("صفر", "Zero", "", ""));
    let mut owed = form("طلب", "Talab", "", "");
    owed.opening_cr = "700".into();
    save(&env, &owner, owed);
    accounts::set_active(&env.ctx, &owner, nasir.id, false).unwrap();
    let list_on = |year: i32, month: u32, day: u32| {
        set_today(&env, year, month, day);
        mandi_services::vouchers::collection_list(
            &env.ctx,
            &mandi_services::core::vouchers::CollectionListInput {
                search: None,
                collector_staff_id: None,
                city_id: None,
                offset: 0,
                limit: 0,
            },
        )
        .unwrap()
    };
    let ids = |list: &mandi_services::core::vouchers::CollectionList| {
        list.groups
            .iter()
            .flat_map(|g| g.buyers.iter().map(|b| b.account.id))
            .collect::<Vec<_>>()
    };
    // Saturday 10-10-26.
    let saturday = list_on(2026, 10, 10);
    assert_eq!(ids(&saturday), [akram.id, arshad.id]);
    assert_eq!(saturday.total_paisa, rupees(21_564));
    assert_eq!(saturday.total_count, 2);
    // Any other weekday: the same buyers, and the age of the bills counted to that day.
    for (day, extra) in [(12, 2), (14, 4), (17, 7)] {
        let other = list_on(2026, 10, day);
        assert_eq!(ids(&other), [akram.id, arshad.id], "10-{day}");
        assert_eq!(other.total_paisa, rupees(21_564));
        let days = |list: &mandi_services::core::vouchers::CollectionList| {
            list.groups[0].buyers[0].oldest_days
        };
        assert_eq!(days(&other), days(&saturday).map(|d| d + extra), "10-{day}");
    }
}
