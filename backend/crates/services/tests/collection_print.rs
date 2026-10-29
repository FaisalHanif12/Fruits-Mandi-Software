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

/// The weekday of 10-10-2026, a Saturday (0 is Monday).
const SATURDAY: i64 = 5;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    set_today(&env, 2026, 10, 10);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

fn form(ur: &str, en: &str, owes: &str, weekday: Option<i64>, tag: &str) -> AccountInput {
    AccountInput {
        id: None,
        version: None,
        account_type: AccountType::Buyer,
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
        collection_weekday: weekday,
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

/// The three buyers: اکرم and ارشد on Saturday, ناصر on Wednesday.
fn buyers(env: &TestEnv, owner: &auth::Actor) -> (AccountRow, AccountRow, AccountRow) {
    let akram = save(
        env,
        owner,
        form(
            "اکرم ٹول ٹیکس",
            "Akram Toll Tax",
            "2,880",
            Some(SATURDAY),
            "",
        ),
    );
    let arshad = save(
        env,
        owner,
        form(
            "ارشد ملتانی",
            "Arshad Multani",
            "18,684",
            Some(SATURDAY),
            "k",
        ),
    );
    let nasir = save(env, owner, form("ناصر", "Nasir", "5,000", Some(2), ""));
    (akram, arshad, nasir)
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

#[test]
fn the_print_has_the_buyers_of_the_day_and_a_total_below_the_shops_dues() {
    let (_dir, env, owner) = ready();
    let (akram, arshad, nasir) = buyers(&env, &owner);
    let (list, filters) =
        printing::collection_for_print(&env.ctx, "10-10-2026", None, None).unwrap();
    // The same list the screen reads.
    let screen = mandi_services::vouchers::collection_list(
        &env.ctx,
        &mandi_services::core::vouchers::CollectionListInput {
            date: "10-10-2026".into(),
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
            vec!["2,880", "18,684"]
        );
        assert_eq!(data.total.amount, "21,564");
        assert_eq!(data.buyer_count, "2");
        // Short codes: the running numbers of the first two buyers, 0020 and 0021 (plan 32).
        assert_eq!(rows[0].code, "0");
        assert_eq!(rows[1].code, "1");
        // The tag of ارشد is its own field.
        assert_eq!(rows[0].tag, None);
        assert_eq!(rows[1].tag.as_deref(), Some("K"));
    }
    // ناصر is on another weekday, so the total of this list is below what all buyers owe
    // (PRD FR-19.9): 21,564 here against 26,564 for every buyer. It is not the shop's اگراہی.
    let all: Paisa = [&akram, &arshad, &nasir]
        .iter()
        .map(|row| row.balance.amount_paisa)
        .try_fold(Paisa::ZERO, Paisa::checked_add)
        .unwrap();
    assert_eq!(all, rupees(26_564));
    assert_eq!(list.total_paisa, rupees(21_564));
    assert!(list.total_paisa < all);
    // Another day of the week is another list.
    let (wednesday, _) =
        printing::collection_for_print(&env.ctx, "07-10-2026", None, None).unwrap();
    assert_eq!(wednesday.total_paisa, rupees(5_000));
}

#[test]
fn unknown_filters_and_bad_dates_are_refused_at_their_fields() {
    let (_dir, env, _owner) = ready();
    assert_eq!(
        first_error(
            printing::collection_for_print(&env.ctx, "10-10-2026", Some(999), None).unwrap_err()
        ),
        (
            ErrorCode::ErrNotFound,
            Some("collector_staff_id".to_owned())
        )
    );
    assert_eq!(
        first_error(
            printing::collection_for_print(&env.ctx, "10-10-2026", None, Some(999)).unwrap_err()
        ),
        (ErrorCode::ErrNotFound, Some("city_id".to_owned()))
    );
    let (code, field) = first_error(
        printing::collection_for_print(&env.ctx, "31-02-2026", None, None).unwrap_err(),
    );
    assert_eq!(field.as_deref(), Some("date"));
    assert_ne!(code, ErrorCode::ErrInternal);
}

#[test]
fn the_plan_checks_copies_names_the_pdf_by_the_day_and_the_audit_row_records_the_print() {
    let (dir, env, owner) = ready();
    buyers(&env, &owner);
    let input = |copies: Option<&str>| PrintCollectionInput {
        date: "10-10-2026".into(),
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
    assert!(new_json.contains("\"buyers\":2"), "{new_json}");
    assert!(new_json.contains("\"total_paisa\":2156400"), "{new_json}");
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
