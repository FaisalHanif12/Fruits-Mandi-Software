//! Plan 64 (PRD Section 13, FR-7.29, FR-7.38, FR-7.69, FR-13.1 to FR-13.7): the چکتا buyers on a
//! real database. A چکتا buyer is a walk-in buyer who pays the whole bill on the spot. He is kept by
//! name in a list, is not an account, and has no ledger and no balance.
//!
//! The numbers are PRD 4.5 (Event 2): شاہد buys 10 سیب at 1,200, value 12,000, commission 720,
//! لاگا 80, paid 12,800 on the spot. The entry is 25,600 = 25,600 and the shared income is 1,540.
//!
//! Run it with `cargo test -p mandi-services --test chukta`.

#![cfg(test)]

#[path = "bikri_save/../common/mod.rs"]
mod common;
#[allow(dead_code)]
#[path = "bikri_save/helpers.rs"]
mod helpers;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use helpers::*;
use mandi_services::core::accounts::{
    AccountScope, AccountsRecentInput, BalanceSide, HitKind, PartyCardInput, SearchAccountsInput,
};
use mandi_services::core::bikri::{BikriDraft, ChuktaInput};
use mandi_services::core::chukta::{ChuktaAddInput, ChuktaSearchInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::{ReceiptInput, RemovedPaymentFields};
use mandi_services::{ServiceError, assist, chukta, ledger, search, vouchers};
use support::*;

fn add(w: &World, ur: &str, en: &str, phone: Option<&str>) -> Result<i64, ServiceError> {
    chukta::add(
        &w.env.ctx,
        &w.owner,
        &ChuktaAddInput {
            name_ur: ur.into(),
            name_en: en.into(),
            phone: phone.map(str::to_owned),
        },
    )
    .map(|b| b.id)
}

fn find(w: &World, text: &str) -> Vec<String> {
    chukta::search(&w.env.ctx, &ChuktaSearchInput { text: text.into() })
        .unwrap()
        .into_iter()
        .map(|b| b.name_en)
        .collect()
}

/// Event 2 of PRD 4.5, with شاہد as a چکتا buyer.
fn event2(w: &World) -> BikriDraft {
    let mut line = sale("s-1", w.chukta_khata, "10", "1200");
    line.chukta = Some(ChuktaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let mut lot = lot(w, "lot-2", w.karim, w.apple, "10");
    lot.freight = Some("1,000".into());
    lot.cash_paid = None;
    lot.sales = vec![line];
    BikriDraft {
        header: header(),
        lots: vec![lot],
    }
}

#[test]
fn a_buyer_is_added_with_both_names_and_found_by_either_script_and_by_phone() {
    let w = world();
    let shahid = add(&w, "شاہد", "Shahid", Some("0300-0000000")).unwrap();
    let nasir = add(&w, "ناصر", "Nasir", None).unwrap();
    assert_ne!(shahid, nasir);
    // Either name, a spelling mistake in the English one, and the phone (PRD FR-7.29, FR-7.40).
    assert_eq!(find(&w, "shahid"), ["Shahid"]);
    assert_eq!(find(&w, "شاہ"), ["Shahid"]);
    assert_eq!(find(&w, "shahd"), ["Shahid"]);
    assert_eq!(find(&w, "0300"), ["Shahid"]);
    assert!(find(&w, "tariq").is_empty());
    // Empty text lists both, newest first; at most 10.
    assert_eq!(find(&w, ""), ["Nasir", "Shahid"]);
    // His phone is kept as 11 digits.
    let found = chukta::search(
        &w.env.ctx,
        &ChuktaSearchInput {
            text: "shahid".into(),
        },
    )
    .unwrap();
    assert_eq!(found[0].phone.as_deref(), Some("03000000000"));
    assert_eq!(found[0].last_date, None, "he has no sale line yet");
    // Every buyer of the list carries the id of the account 0030, which a sale line to him takes as
    // its buyer account, so the screen needs no second lookup (plan 64).
    assert!(found.iter().all(|b| b.account_id == w.chukta_khata));
}

#[test]
fn the_same_person_is_one_row_and_another_phone_is_another_person() {
    let w = world();
    let first = add(&w, "شاہد", "Shahid", Some("0300-0000000")).unwrap();
    let again = add(&w, "شاہد", "Shahid", Some("03000000000")).unwrap();
    assert_eq!(first, again, "the same name and phone are one person");
    let other = add(&w, "شاہد", "Shahid", Some("0300-9999999")).unwrap();
    assert_ne!(first, other);
    // Two adds, one audit row for each new person (Code Standards S7).
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'chukta_buyer' AND action = 'create'"
        ),
        2
    );
}

#[test]
fn a_new_buyer_is_checked_on_both_names_and_the_phone_together() {
    let w = world();
    let refused = add(&w, "", "Shahid", Some("123")).unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("not a refusal");
    };
    let fields: Vec<(&str, ErrorCode)> = items
        .iter()
        .map(|i| (i.field.as_deref().unwrap_or(""), i.code))
        .collect();
    assert_eq!(
        fields,
        [
            ("name_ur", ErrorCode::ErrChuktaNameRequired),
            ("phone", ErrorCode::ErrPhoneInvalid)
        ]
    );
    // A name in the wrong script is refused (PRD FR-7.28).
    let wrong = add(&w, "Shahid", "شاہد", None).unwrap_err();
    assert_eq!(wrong.code(), ErrorCode::ErrNameUrScript);
    assert_eq!(find(&w, "").len(), 0, "nothing was saved");
}

#[test]
fn a_sale_line_to_a_new_person_puts_him_in_the_list_and_leaves_no_balance_on_0030() {
    let w = world();
    let saved = save_ok(&w, &event2(&w));
    // Event 2: the whole entry is 25,600 = 25,600; shared income 1,540 (PRD 4.5).
    let lines = entry_lines(&w, saved.entry_id);
    let key = Some("شاہد|03000000000".to_owned());
    assert_eq!(
        lines.iter().map(|l| l.1).sum::<i64>(),
        lines.iter().map(|l| l.2).sum::<i64>()
    );
    assert_eq!(lines.iter().map(|l| l.1).sum::<i64>(), 25_600);
    assert!(
        lines.iter().filter(|l| l.0 == "0030").all(|l| l.3 == key),
        "every line on 0030 carries his person key"
    );
    // He paid in full, so 0030 holds nothing (PRD FR-13.1).
    assert_eq!(balance(&w, w.chukta_khata).0, BalanceSide::Zero);
    // The save made him a row of the list, with the day of his last line.
    let found = chukta::search(
        &w.env.ctx,
        &ChuktaSearchInput {
            text: "shahid".into(),
        },
    )
    .unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].last_date.as_deref(), Some("2026-10-10"));
    assert_eq!(found[0].phone.as_deref(), Some("03000000000"));
    // A second sale to the same name and phone does not make him twice (PRD FR-13.2).
    let mut again = event2(&w);
    again.lots[0].key = "lot-3".into();
    again.header.truck_no = Some("LHR-0078".into());
    save_ok(&w, &again);
    assert_eq!(find(&w, "").len(), 1);
}

#[test]
fn a_chukta_line_always_pays_the_whole_bill_whatever_is_typed() {
    // PRD FR-13.5, FR-12.44: the backend sets paid = buyer total for a چکتا buyer and ignores what
    // is typed in the paid field. 12,800 is on the cash credit line of the entry.
    let w = world();
    for typed in [None, Some("1"), Some("0"), Some("5,000")] {
        let mut draft = event2(&w);
        draft.lots[0].key = format!("lot-{}", typed.unwrap_or("none"));
        draft.header.truck_no = Some(format!("T-{}", typed.unwrap_or("none")));
        draft.lots[0].sales[0].paid = typed.map(str::to_owned);
        let saved = save_ok(&w, &draft);
        let cash_credit = entry_lines(&w, saved.entry_id)
            .iter()
            .filter(|l| l.2 == 12_800 && l.3.is_some())
            .count();
        assert_eq!(cash_credit, 1, "typed {typed:?}: the whole 12,800 is paid");
    }
}

#[test]
fn a_line_on_0030_needs_a_name_and_a_stray_name_on_an_ordinary_buyer_is_ignored() {
    let w = world();
    let mut draft = event2(&w);
    draft.lots[0].sales[0].chukta = None;
    let refused = save(&w, &draft).unwrap_err();
    let ServiceError::Rejected(items) = refused else {
        panic!("not a refusal");
    };
    assert!(
        items
            .iter()
            .any(|i| i.code == ErrorCode::ErrChuktaNameRequired
                && i.field.as_deref() == Some("lots[0].sales[0].chukta.name_ur"))
    );
    // A چکتا name left on the line of an ordinary buyer (the screen switched buyers) is ignored: he is
    // saved as the ordinary buyer he is, and no person key is written.
    let mut draft = event2(&w);
    draft.lots[0].sales[0].buyer_account_id = Some(w.akram);
    let saved = save_ok(&w, &draft);
    assert!(
        entry_lines(&w, saved.entry_id)
            .iter()
            .all(|l| l.3.is_none()),
        "no person key on an ordinary buyer"
    );
}

#[test]
fn only_the_buyer_field_of_a_sale_line_lists_chukta_beside_buyers_and_never_the_account_0030() {
    let w = world();
    save_ok(&w, &event2(&w));
    let ask = |scope: AccountScope, text: &str, with_chukta: Option<bool>| {
        search::search_accounts(
            &w.env.ctx,
            &w.owner,
            &SearchAccountsInput {
                scope,
                control_id: None,
                text: text.into(),
                limit: 50,
                offset: 0,
                include_dead: None,
                with_chukta,
            },
        )
        .unwrap()
    };
    // The buyer field of a sale line asks for them (decision C4 of plan 60).
    let hits = ask(AccountScope::Buyer, "shahid", Some(true));
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].kind, HitKind::Chukta);
    assert_eq!(
        hits[0].id, w.chukta_khata,
        "a sale line takes this as its buyer"
    );
    assert!(hits[0].chukta_id.is_some());
    assert_eq!(hits[0].phone.as_deref(), Some("03000000000"));
    // Every other buyer picker, filter and list leaves them out, so its pages stay right.
    for flag in [None, Some(false)] {
        assert!(
            ask(AccountScope::Buyer, "shahid", flag).is_empty(),
            "{flag:?}"
        );
        assert!(
            ask(AccountScope::Buyer, "", flag)
                .iter()
                .all(|h| h.kind == HitKind::Account),
            "{flag:?}"
        );
    }
    let recent = |with_chukta: Option<bool>| {
        search::recent_accounts(
            &w.env.ctx,
            &w.owner,
            &AccountsRecentInput {
                scope: AccountScope::Buyer,
                control_id: None,
                include_dead: None,
                with_chukta,
            },
        )
        .unwrap()
    };
    assert!(recent(None).iter().all(|h| h.kind == HitKind::Account));
    assert!(recent(Some(true)).iter().any(|h| h.kind == HitKind::Chukta));
    // An ordinary buyer is a plain account hit either way.
    let akram = ask(AccountScope::Buyer, "akram", Some(true));
    assert!(akram.iter().all(|h| h.kind == HitKind::Account));
    // The account چکتا کھاتہ itself is never offered, by name or by code (PRD FR-13.4).
    for flag in [None, Some(true)] {
        for text in ["chukta", "0030", ""] {
            assert!(
                ask(AccountScope::Buyer, text, flag)
                    .iter()
                    .all(|h| h.kind == HitKind::Chukta || h.id != w.chukta_khata),
                "{text} {flag:?}"
            );
        }
    }
    // The other pickers never list him, even when asked (PRD FR-13.4).
    assert!(ask(AccountScope::Seller, "shahid", Some(true)).is_empty());
    assert!(ask(AccountScope::Any, "shahid", Some(true)).is_empty());
}

#[test]
fn the_party_card_of_a_chukta_buyer_has_his_name_and_phone_and_no_balance() {
    let w = world();
    save_ok(&w, &event2(&w));
    let found = chukta::search(
        &w.env.ctx,
        &ChuktaSearchInput {
            text: "shahid".into(),
        },
    )
    .unwrap();
    let card = assist::chukta_card(&w.env.ctx, found[0].id).unwrap();
    assert!(card.is_chukta);
    assert_eq!(
        (card.name_en.as_str(), card.phone.as_str()),
        ("Shahid", "03000000000")
    );
    assert_eq!(card.balance.side, BalanceSide::Zero);
    assert!(
        card.credit_limit.is_none() && card.credit_used.is_none() && card.advance_open.is_none()
    );
    assert!(card.warnings.is_empty());
    // The card of the account itself says the same: no balance (PRD FR-7.38, FR-13.6).
    let account = assist::party_card(&w.env.ctx, w.chukta_khata).unwrap();
    assert!(account.is_chukta);
    assert_eq!(account.balance.side, BalanceSide::Zero);
    // The command's input has the optional `chukta_id`.
    let input: PartyCardInput = serde_json::from_value(
        serde_json::json!({ "account_id": w.chukta_khata, "chukta_id": found[0].id }),
    )
    .unwrap();
    assert_eq!(input.chukta_id, Some(found[0].id));
    // A row that does not exist is not found.
    assert_eq!(
        assist::chukta_card(&w.env.ctx, 9_999).unwrap_err().code(),
        ErrorCode::ErrNotFound
    );
}

#[test]
fn no_voucher_takes_the_account_0030_he_has_no_ledger() {
    let w = world();
    save_ok(&w, &event2(&w));
    let receipt = ReceiptInput {
        date: "10-10-2026".into(),
        account_id: w.chukta_khata,
        amount: "1,000".into(),
        removed: RemovedPaymentFields::default(),
        collector_staff_id: None,
        narration: String::new(),
    };
    assert!(vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt).is_err());
    // And his bill figures are nothing: 0030 holds no balance by any reading.
    let b = ledger::account_balance(&w.env.ctx, w.chukta_khata).unwrap();
    assert_eq!(b.side, BalanceSide::Zero);
}
