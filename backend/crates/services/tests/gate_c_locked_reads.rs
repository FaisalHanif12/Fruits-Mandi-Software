//! Plan 80 gate fixes R4-1 and R4-2 (backend-c): what a partner's own money shows while the partner
//! section is locked (PRD FR-17.11, NFR-SEC-14, Code Standards S8).
//!
//! - R4-1: `fruit::list` gives the shop's guests' fruit only while locked, with `partners_locked`
//!   true; a request for one partner's fruit is refused with `ERR_PARTNER_LOCKED` (as report R37).
//! - R4-2: `vouchers::list` refuses the four types of a partner's own money while locked and leaves
//!   them out of the list of every type; `lent::attention_overdue` leaves the partner takes out;
//!   `lent::list` and `khata::statement` give no partner id of a partner link.
//!
//! Each test fails on the code before the fix: the lists showed everything, whatever the lock.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::OWNER_PASSWORD;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::fruit::{FruitInput, FruitListInput};
use mandi_services::core::khata::{KhataCreateRequest, KhataStatementInput};
use mandi_services::core::lent::{
    LentListInput, LentPersonCreateRequest, MoneyLentInput, OverdueKind,
};
use mandi_services::core::partner_code::PartnerCodeSetInput;
use mandi_services::core::vouchers::{VoucherType, VouchersListInput};
use mandi_services::{fruit, khata, lent, partner_code, vouchers};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use support::World;

fn typed<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

fn set_code_and_lock(w: &World) {
    let input = PartnerCodeSetInput {
        password: OWNER_PASSWORD.to_owned(),
        code: "4826".to_owned(),
        code_again: "4826".to_owned(),
    };
    partner_code::set(&w.env.ctx, &w.owner, &input).unwrap();
    partner_code::lock(&w.env.ctx).unwrap();
    assert!(partner_code::is_locked(&w.env.ctx).unwrap());
}

fn unlock(w: &World) {
    partner_code::unlock(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::partner_code::PartnerCodeUnlockInput {
            code: "4826".to_owned(),
        },
    )
    .unwrap();
}

#[test]
fn r4_1_fruit_list_gives_the_guests_fruit_only_while_locked() {
    let w = support::world();
    let ctx = &w.env.ctx;
    let agent = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "حاجی نذیر",
        "Haji Nazeer",
        true,
        "",
    );
    // Home fruit for الف: 2 x 1,400 = 2,800. Guest fruit: 3 x 1,000 = 3,000.
    let entry = |for_guests: bool, qty: &str, rate: &str| -> FruitInput {
        typed(json!({
            "for_guests": for_guests,
            "partner_id": if for_guests { Value::Null } else { json!(w.partners[0]) },
            "date": "10-10-2026", "source": "other_shop", "lot_id": null,
            "agent_account_id": agent, "item_id": w.apple, "packing_id": w.box_,
            "qty": qty, "weight": null, "rate": rate, "note": null, "editing_id": null
        }))
    };
    fruit::save(ctx, &w.owner, &entry(false, "2", "1,400")).unwrap();
    fruit::save(ctx, &w.owner, &entry(true, "3", "1,000")).unwrap();

    // Open: both entries, both totals, nothing hidden.
    let open = fruit::list(ctx, &FruitListInput::default()).unwrap();
    assert_eq!(open.total_count, 2);
    assert_eq!(open.totals.value_paisa.as_paisa(), 580_000);
    assert!(!open.partners_locked);

    set_code_and_lock(&w);

    // Locked, for everyone: the guests' fruit only. No partner's entry, no partner's amount.
    let locked = fruit::list(ctx, &FruitListInput::default()).unwrap();
    assert!(locked.partners_locked);
    assert_eq!(locked.total_count, 1);
    assert_eq!(locked.totals.value_paisa.as_paisa(), 300_000);
    assert_eq!(locked.totals.qty, 3);
    assert_eq!(locked.rows.len(), 1);
    assert!(locked.rows[0].partner.is_none());

    // A search cannot reach a partner's entry either (the partner's name is in the search text).
    let searched = fruit::list(
        ctx,
        &typed::<FruitListInput>(json!({ "search": "Partner A" })),
    )
    .unwrap();
    assert_eq!(searched.total_count, 0);
    assert_eq!(searched.totals.value_paisa.as_paisa(), 0);

    // Locked, for one partner: refused.
    let refused = fruit::list(
        ctx,
        &typed::<FruitListInput>(json!({ "partner_id": w.partners[0] })),
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrPartnerLocked);

    // The guests filter still works while locked.
    let guests = fruit::list(
        ctx,
        &typed::<FruitListInput>(json!({ "guests_only": true })),
    )
    .unwrap();
    assert_eq!(guests.total_count, 1);

    // With the code everything is back.
    unlock(&w);
    let back = fruit::list(ctx, &FruitListInput::default()).unwrap();
    assert_eq!(back.total_count, 2);
    assert!(!back.partners_locked);
    assert_eq!(
        fruit::list(
            ctx,
            &typed::<FruitListInput>(json!({ "partner_id": w.partners[0] })),
        )
        .unwrap()
        .total_count,
        1
    );
}

fn save_partner_vouchers(w: &World) {
    let ctx = &w.env.ctx;
    let a = w.partners[0];
    vouchers::save_personal_expense(
        ctx,
        &w.owner,
        &typed(
            json!({ "date": "10-10-2026", "partner_id": a, "amount": "2,000",
            "reason": "گھر کا خرچ", "return_by": null, "taken_by": null, "narration": null,
            "confirm_cash_below_zero": false }),
        ),
    )
    .unwrap();
    vouchers::save_personal_return(
        ctx,
        &w.owner,
        &typed(
            json!({ "date": "10-10-2026", "partner_id": a, "amount": "500",
            "narration": null, "confirm_cash_below_zero": false }),
        ),
    )
    .unwrap();
    vouchers::save_capital_in(
        ctx,
        &w.owner,
        &typed(
            json!({ "date": "10-10-2026", "partner_id": a, "amount": "50,000",
            "narration": null, "confirm_cash_below_zero": false }),
        ),
    )
    .unwrap();
    vouchers::save_capital_out(
        ctx,
        &w.owner,
        &typed(
            json!({ "date": "10-10-2026", "partner_id": a, "amount": "1,000",
            "narration": null, "confirm_cash_below_zero": false }),
        ),
    )
    .unwrap();
}

fn list_of(w: &World, kind: Option<VoucherType>) -> Result<(i64, Vec<VoucherType>), ErrorCode> {
    let input: VouchersListInput = typed(json!({
        "voucher_type": kind.map(|k| k.as_str()), "from": "", "to": ""
    }));
    vouchers::list(&w.env.ctx, &input)
        .map(|list| {
            (
                list.total_count,
                list.rows.iter().map(|r| r.voucher_type).collect(),
            )
        })
        .map_err(|e| e.code())
}

#[test]
fn r4_2_vouchers_list_refuses_a_partners_own_money_while_locked() {
    let w = support::world();
    let ctx = &w.env.ctx;
    save_partner_vouchers(&w);
    // Money lent to a person is not a partner's money: it stays listed.
    let person = lent::person_create_linked(
        ctx,
        &w.owner,
        &typed::<LentPersonCreateRequest>(json!({
            "name_ur": "اسلم", "name_en": "Aslam", "phone": null, "staff_id": null,
            "confirmed_rules_words": [], "person_type": "other", "link": null })),
    )
    .unwrap();
    vouchers::save_money_lent(
        ctx,
        &w.owner,
        &typed::<MoneyLentInput>(json!({ "date": "10-10-2026",
            "person_account_id": person.account.id, "amount": "100", "reason": "ضرورت",
            "return_by": null, "narration": null, "confirm_cash_below_zero": false })),
    )
    .unwrap();

    let partner_types = [
        VoucherType::PersonalExpense,
        VoucherType::PersonalReturn,
        VoucherType::CapitalIn,
        VoucherType::CapitalOut,
    ];
    // Open: each type lists its voucher; the list of every type holds all four.
    for kind in partner_types {
        assert_eq!(list_of(&w, Some(kind)).unwrap().0, 1, "{kind:?}");
    }
    let all_open = list_of(&w, None).unwrap();
    for kind in partner_types {
        assert!(all_open.1.contains(&kind), "{kind:?}");
    }

    set_code_and_lock(&w);

    // Locked: each of the four is refused.
    for kind in partner_types {
        assert_eq!(
            list_of(&w, Some(kind)).unwrap_err(),
            ErrorCode::ErrPartnerLocked,
            "{kind:?}"
        );
    }
    // The list of every type leaves them out of its rows and its count.
    let all_locked = list_of(&w, None).unwrap();
    for kind in partner_types {
        assert!(!all_locked.1.contains(&kind), "{kind:?} is still listed");
    }
    assert_eq!(
        all_locked.0,
        i64::try_from(all_locked.1.len()).unwrap(),
        "the count is of the rows that are not partner money"
    );
    assert!(all_locked.0 < all_open.0);
    // The other types are not affected.
    assert_eq!(list_of(&w, Some(VoucherType::MoneyLent)).unwrap().0, 1);

    unlock(&w);
    assert_eq!(list_of(&w, None).unwrap().0, all_open.0);
    assert_eq!(list_of(&w, Some(VoucherType::CapitalIn)).unwrap().0, 1);
}

#[test]
fn r4_2_the_overdue_list_leaves_the_partner_takes_out_while_locked() {
    let w = support::world();
    let ctx = &w.env.ctx;
    let a = w.partners[0];
    // A take of الف and a loan to a person, both with a return-by day that has passed.
    vouchers::save_personal_expense(
        ctx,
        &w.owner,
        &typed(
            json!({ "date": "01-10-2026", "partner_id": a, "amount": "2,000",
            "reason": "گھر کا خرچ", "return_by": "05-10-2026", "taken_by": null,
            "narration": null, "confirm_cash_below_zero": false }),
        ),
    )
    .unwrap();
    let person = lent::person_create_linked(
        ctx,
        &w.owner,
        &typed::<LentPersonCreateRequest>(json!({
            "name_ur": "اسلم", "name_en": "Aslam", "phone": null, "staff_id": null,
            "confirmed_rules_words": [], "person_type": "other", "link": null })),
    )
    .unwrap();
    vouchers::save_money_lent(
        ctx,
        &w.owner,
        &typed::<MoneyLentInput>(json!({ "date": "01-10-2026",
            "person_account_id": person.account.id, "amount": "5,000", "reason": "ضرورت",
            "return_by": "05-10-2026", "narration": null, "confirm_cash_below_zero": false })),
    )
    .unwrap();

    let open = lent::attention_overdue(ctx).unwrap();
    assert!(!open.partners_locked);
    let kinds: Vec<OverdueKind> = open.items.iter().map(|i| i.kind).collect();
    assert!(kinds.contains(&OverdueKind::Take) && kinds.contains(&OverdueKind::Loan));
    assert_eq!(open.total_count, 2);

    set_code_and_lock(&w);
    let locked = lent::attention_overdue(ctx).unwrap();
    assert!(locked.partners_locked);
    assert!(
        locked.items.iter().all(|i| i.kind == OverdueKind::Loan),
        "no partner take while locked"
    );
    assert_eq!(locked.total_count, 1);

    unlock(&w);
    assert_eq!(lent::attention_overdue(ctx).unwrap().total_count, 2);
}

#[test]
fn r4_2_a_partner_link_carries_no_partner_while_locked() {
    let w = support::world();
    let ctx = &w.env.ctx;
    let a = w.partners[0];
    let link = json!({ "kind": "partner", "partner_id": a });
    // A person lent money and a khata, both linked to الف.
    let person = lent::person_create_linked(
        ctx,
        &w.owner,
        &typed::<LentPersonCreateRequest>(json!({
            "name_ur": "نواز", "name_en": "Nawaz", "phone": null, "staff_id": null,
            "confirmed_rules_words": [], "person_type": "garden_owner", "link": link })),
    )
    .unwrap();
    let family = khata::create_linked(
        ctx,
        &w.owner,
        &typed::<KhataCreateRequest>(json!({
            "name_ur": "خاندانی کھاتہ", "name_en": "Family Khata", "note": null,
            "confirmed_rules_words": ["خاندانی", "Family"], "link": link })),
    )
    .unwrap();
    let loans = |w: &World| {
        lent::list(
            &w.env.ctx,
            &typed::<LentListInput>(json!({ "person_id": person.account.id })),
        )
        .unwrap()
    };
    let statement = |w: &World| {
        khata::statement(
            &w.env.ctx,
            &typed::<KhataStatementInput>(json!({ "khata_account_id": family.account.id })),
        )
        .unwrap()
    };
    assert_eq!(loans(&w).person.link.partner_id, Some(a));
    assert_eq!(statement(&w).khata.link.partner_id, Some(a));

    set_code_and_lock(&w);
    let hidden = loans(&w).person.link;
    assert_eq!(hidden.partner_id, None, "lent_list shows the partner");
    assert_eq!(
        statement(&w).khata.link.partner_id,
        None,
        "khata_statement shows the partner"
    );
    // The kind stays: the screen still knows the loan is a partner's.
    assert_eq!(hidden.kind, statement(&w).khata.link.kind);

    unlock(&w);
    assert_eq!(loans(&w).person.link.partner_id, Some(a));
    assert_eq!(statement(&w).khata.link.partner_id, Some(a));
}

/// Plan 80 gate Q-R7-2 (NFR-SEC-14, PRD FR-8.2): after the 20th wrong partner code every further wrong
/// code waits 15 minutes. Before the fix the 21st to the 29th wrong codes got no wait at all.
#[test]
fn q_r7_2_every_wrong_partner_code_after_the_20th_waits_fifteen_minutes() {
    let w = support::world();
    let ctx = &w.env.ctx;
    set_code_and_lock(&w);
    let wrong = || {
        partner_code::unlock(
            ctx,
            &w.owner,
            &mandi_services::core::partner_code::PartnerCodeUnlockInput {
                code: "1111".to_owned(),
            },
        )
        .unwrap_err()
    };
    let seconds = |error: &mandi_services::ServiceError| match error {
        mandi_services::ServiceError::Rejected(items) => (
            items[0].code,
            match items[0].values.get("seconds") {
                Some(mandi_services::core::error::ErrorValue::Number(n)) => Some(*n),
                _ => None,
            },
        ),
        other => panic!("{other:?}"),
    };
    // Tries 1 to 9 count down, the 10th waits a minute, 11 to 19 count down, the 20th waits 5 minutes.
    for _ in 0..9 {
        assert_eq!(wrong().code(), ErrorCode::ErrPartnerCodeWrong);
    }
    assert_eq!(seconds(&wrong()), (ErrorCode::ErrLockedWait, Some(60)));
    w.env.mono.advance_seconds(61);
    for _ in 0..9 {
        assert_eq!(wrong().code(), ErrorCode::ErrPartnerCodeWrong);
    }
    assert_eq!(seconds(&wrong()), (ErrorCode::ErrLockedWait, Some(300)));
    w.env.mono.advance_seconds(301);
    // The 21st, 22nd and 23rd wrong codes each wait 15 minutes.
    for try_number in 21..=23 {
        assert_eq!(
            seconds(&wrong()),
            (ErrorCode::ErrLockedWait, Some(900)),
            "wrong code {try_number}"
        );
        w.env.mono.advance_seconds(901);
    }
    // The right code after the wait opens the section.
    unlock(&w);
    assert!(!partner_code::is_locked(ctx).unwrap());
}
