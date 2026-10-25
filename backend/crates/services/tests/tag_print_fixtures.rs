//! Real answers of the plan 17e commands that need a database with the worked day in it, as JSON
//! files for the frontend: the tag in the search, the party card, the duplicate check and the
//! prints, and the collection list print (plan 17e T2).
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type before it is used), and `<name>.json` is the answer the real
//! service gives for it. The frontend copies the files to `frontend/tests/fixtures/`. The answers of
//! `accounts_save`, `settings_*` and `reports_print_header` are written by the app test
//! `plan_17e_the_tag_the_firm_contacts_and_the_shared_header_through_the_ipc`, because their wire
//! types live in the app.
//!
//! The test compares the answers with the files on disk and FAILS if any differ, so a fixture can
//! never go stale. After a deliberate change of an answer, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test tag_print_fixtures`, look at the diff,
//! and tell `frontend`.
//!
//! The story is the PRD worked day after Events 1 to 5 as far as the buyers go: بکری 1, a receipt
//! of 10,000 from اکرم (he owes 2,880, ارشد owes 18,684), both buyers on the Saturday of the
//! collection round; اکرم has the tag K, and the firm has two contact persons and the trade mark RH.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::path::{Path, PathBuf};

use common::set_today;
use mandi_reports::print::{BikriPrintDataInput, bikri_print_data, collection_print_data};
use mandi_services::core::accounts::{
    AccountInput, AccountsSimilarInput, GlobalHit, SearchAccountsInput, SearchGlobalInput,
};
use mandi_services::core::auth::Language;
use mandi_services::core::firm::FirmContact;
use mandi_services::core::printing::PrintDoc;
use mandi_services::core::vouchers::ReceiptInput;
use mandi_services::settings::{self, FirmPrintInfo};
use mandi_services::{accounts, assist, bikri, printing, search, vouchers};
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

/// The request read back into the real input type, so the file is exactly what the command takes.
fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

#[test]
fn the_real_answers_of_the_plan_17e_commands_are_the_fixtures() {
    let w = support::world();
    set_today(&w.env, 2026, 10, 10);
    let ctx = &w.env.ctx;

    // Event 1, a receipt of 10,000 from اکرم, and the collection day of both buyers.
    bikri::save(
        ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: support::event1(&w),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let receipt = json!({ "date": "10-10-2026", "account_id": w.akram, "person_key": null,
        "amount": "10,000", "collector_staff_id": null, "narration": "" });
    vouchers::save_receipt(ctx, &w.owner, &typed::<ReceiptInput>(&receipt)).unwrap();
    // Both buyers go on the Saturday round (weekday 5, 0 is Monday), and اکرم gets the tag K: two
    // real account saves, so the search keys hold the tag.
    for (id, tag) in [(w.akram, "k"), (w.arshad, "")] {
        let row = accounts::get(ctx, id).unwrap();
        let edit = json!({ "id": row.id, "version": row.version, "account_type": "buyer",
            "control_id": null, "code": row.code, "name_ur": row.name_ur, "name_en": row.name_en,
            "confirmed_rules_words": [], "father_or_firm_ur": "", "father_or_firm_en": "",
            "city_id": null, "adda_id": null, "phone1": "", "phone2": "", "address": "",
            "linked_account_id": null, "opening_date": "", "opening_dr": "", "opening_cr": "",
            "opening_is_advance": false, "credit_allowed": true, "credit_limit": "",
            "default_marka_id": null, "collector_staff_id": null, "collection_weekday": 5,
            "is_active": true, "note": "", "tag": tag });
        accounts::save(ctx, &w.owner, &typed::<AccountInput>(&edit)).unwrap();
    }

    // The search finds "ashiq k"-style: here "akram k" finds اکرم ٹول ٹیکس K, and the hit has the tag.
    let request = json!({ "scope": "any", "control_id": null, "text": "akram k", "limit": 10 });
    let hits =
        search::search_accounts(ctx, &w.owner, &typed::<SearchAccountsInput>(&request)).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].tag.as_deref(), Some("K"));
    pair("search_accounts_tag", &request, &hits);

    // The global search (`Ctrl+K`, PRD FR-5.4, FR-7.40): the account hit carries the tag.
    let request = json!({ "text": "akram k", "limit": 10 });
    let global = search::global(ctx, &w.owner, &typed::<SearchGlobalInput>(&request)).unwrap();
    let [GlobalHit::Account { hit }] = global.as_slice() else {
        panic!("one account hit expected: {global:?}")
    };
    assert_eq!((hit.id, hit.tag.as_deref()), (w.akram, Some("K")));
    pair("search_global_tag", &request, &global);
    // The digit `1` finds the accounts with serial 1, بکری 1 and the receipt 1 of the day; the
    // voucher hit names its account with the tag.
    let request = json!({ "text": "1", "limit": 10 });
    let global = search::global(ctx, &w.owner, &typed::<SearchGlobalInput>(&request)).unwrap();
    let voucher_tag = global.iter().find_map(|hit| match hit {
        GlobalHit::Voucher { hit } => hit.account.as_ref().map(|a| a.tag.clone()),
        _ => None,
    });
    assert_eq!(voucher_tag, Some(Some("K".to_owned())));
    assert!(
        global
            .iter()
            .any(|hit| matches!(hit, GlobalHit::Bikri { .. }))
    );
    pair("search_global_tag_number", &request, &global);

    // The party card carries the tag.
    let card = assist::party_card(ctx, w.akram).unwrap();
    assert_eq!(card.tag.as_deref(), Some("K"));
    pair(
        "assist_party_card_tag",
        &json!({ "account_id": w.akram }),
        &card,
    );

    // The duplicate check: same name and phone are typed, the tag decides.
    let ask = |tag: &str| {
        json!({ "name_ur": "اکرم ٹول ٹیکس", "name_en": "", "phone": "", "tag": tag,
            "exclude_id": null })
    };
    let duplicate = accounts::similar(ctx, &typed::<AccountsSimilarInput>(&ask("k"))).unwrap();
    assert!(
        duplicate[0].is_duplicate,
        "same name and tag, no phone typed on either side"
    );
    pair("accounts_similar_tag_duplicate", &ask("k"), &duplicate);
    let other = accounts::similar(ctx, &typed::<AccountsSimilarInput>(&ask("BA"))).unwrap();
    assert!(
        other.iter().all(|a| !a.is_duplicate),
        "another tag is another person"
    );
    assert_eq!(other.len(), 1, "but he is listed as similar");
    pair("accounts_similar_tag_other", &ask("BA"), &other);

    // The firm: two contact persons and the trade mark.
    let firm = settings::get(ctx).unwrap().firm;
    let info = FirmPrintInfo {
        firm_contacts: vec![
            FirmContact {
                name_ur: "منشی آصف بھٹی".into(),
                name_en: "Munshi Asif Bhatti".into(),
                phone: "0300-4058860".into(),
            },
            FirmContact {
                name_ur: "اکرم".into(),
                name_en: "Akram".into(),
                phone: "03211234567".into(),
            },
        ],
        trade_mark: "RH".into(),
    };
    settings::set_firm_with_print_info(ctx, &w.owner, &firm, &info, &[]).unwrap();

    // The seller bill and the buyer slips (R5 columns) with the tag, short codes and contacts.
    let bikri_id = 1;
    for (doc, language, name) in [
        (PrintDoc::SellerBill, Language::Ur, "seller_bill_ur"),
        (PrintDoc::BuyerSlips, Language::Ur, "buyer_slips_ur"),
        (PrintDoc::BuyerSlips, Language::En, "buyer_slips_en"),
    ] {
        let request = json!({ "bikri_id": bikri_id, "doc": doc, "language": language,
            "only_buyer_id": null });
        let data =
            bikri_print_data(w.env.ctx.db(), &typed::<BikriPrintDataInput>(&request)).unwrap();
        assert_eq!(data.header.contacts.len(), 2);
        assert_eq!(data.header.trade_mark.as_deref(), Some("RH"));
        pair(
            &format!("reports_bikri_print_data_tag_{name}"),
            &request,
            &data,
        );
    }

    // The collection list: 2,880 and 18,684, 21,564 in all, in both languages.
    for (language, name) in [(Language::Ur, "ur"), (Language::En, "en")] {
        let request = json!({ "date": "10-10-2026", "collector_staff_id": null, "city_id": null,
            "language": language });
        let (list, filters) =
            printing::collection_for_print(ctx, "10-10-2026", None, None).unwrap();
        let data = collection_print_data(w.env.ctx.db(), &list, &filters, language).unwrap();
        assert_eq!(data.total.amount, "21,564");
        assert_eq!(data.groups[0].rows[0].tag.as_deref(), Some("K"));
        assert_eq!(data.header.contacts.len(), 2);
        pair(
            &format!("reports_collection_print_data_{name}"),
            &request,
            &data,
        );
    }
}
