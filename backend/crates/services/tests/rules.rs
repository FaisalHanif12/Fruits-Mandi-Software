//! Commission rules and which rule wins (plan 09, PRD 11, acceptance D2-A12 and D2-A13): the
//! seeded defaults of PRD 4.2, every level of PRD 11.3 through the real commands, the checks of
//! the rule form, the overlap warning, the three defaults in Settings, the overrides and the
//! audit trail.
#![cfg(test)]

mod common;

use std::time::Instant;

use common::*;
use mandi_services::core::accounts::{AccountInput, AccountType};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::masters::{ItemInput, MasterInput};
use mandi_services::core::money::Paisa;
use mandi_services::core::rules::{
    ChargeMethod, ChargeSide, ChargeType, ChargesInput, CreditLimitMode, DefaultChargeInput,
    ItemGroup, OverridesSettings, RuleInput, RuleLevel, RuleSaved, RuleTest, RuleTestInput,
    RulesFilter, SkipReason, WorkingValue,
};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, auth, masters, rules};
use tempfile::TempDir;

struct Fixture {
    dir: TempDir,
    env: TestEnv,
    owner: auth::Actor,
    rahim: i64,
    karim: i64,
    akram: i64,
    banana: i64,
    apple: i64,
    mooli: i64,
    japani: i64,
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn add_account(env: &TestEnv, owner: &auth::Actor, kind: AccountType, ur: &str, en: &str) -> i64 {
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
    };
    loop {
        match accounts::save(&env.ctx, owner, &input) {
            Ok(row) => return row.id,
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

fn add_item(env: &TestEnv, owner: &auth::Actor, ur: &str, en: &str, group: &str) -> i64 {
    let mut input = ItemInput {
        id: None,
        version: None,
        code: String::new(),
        name_ur: ur.into(),
        name_en: en.into(),
        item_group: group.into(),
        default_packing_id: None,
        sold_by: "unit".into(),
        confirmed_rules_words: Vec::new(),
    };
    loop {
        match masters::save(&env.ctx, owner, &MasterInput::Item(input.clone())) {
            Ok(row) => return row.id(),
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

/// The worked day's parties and items (PRD 4.4, 11.3).
fn fixture() -> Fixture {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = add_account(&env, &owner, AccountType::Seller, "رحیم", "Rahim");
    let karim = add_account(&env, &owner, AccountType::Seller, "کریم", "Karim");
    let akram = add_account(
        &env,
        &owner,
        AccountType::Buyer,
        "اکرم ٹول ٹیکس",
        "Akram Toll Tax",
    );
    let banana = add_item(&env, &owner, "کیلا", "Banana", "fruit");
    let apple = add_item(&env, &owner, "سیب", "Apple", "fruit");
    let mooli = add_item(&env, &owner, "مولی", "Radish", "vegetable");
    let japani = add_item(&env, &owner, "جاپانی پھل", "Japanese Fruit", "fruit");
    Fixture {
        dir,
        env,
        owner,
        rahim,
        karim,
        akram,
        banana,
        apple,
        mooli,
        japani,
    }
}

fn form(side: ChargeSide, charge: ChargeType, method: ChargeMethod, value: &str) -> RuleInput {
    RuleInput {
        id: None,
        version: None,
        side,
        charge_type: charge,
        account_id: None,
        item_id: None,
        item_group: None,
        method,
        value: value.into(),
        from_date: "01-01-2026".into(),
        to_date: String::new(),
        is_active: true,
        note: String::new(),
    }
}

fn seller_commission(
    _f: &Fixture,
    account: Option<i64>,
    item: Option<i64>,
    value: &str,
) -> RuleInput {
    RuleInput {
        account_id: account,
        item_id: item,
        ..form(
            ChargeSide::Seller,
            ChargeType::Commission,
            ChargeMethod::Percent,
            value,
        )
    }
}

fn save(f: &Fixture, input: &RuleInput) -> RuleSaved {
    rules::save(&f.env.ctx, &f.owner, input).unwrap()
}

fn refused(f: &Fixture, input: &RuleInput) -> (ErrorCode, Option<String>) {
    match rules::save(&f.env.ctx, &f.owner, input).unwrap_err() {
        ServiceError::Rejected(items) => (items[0].code, items[0].field.clone()),
        other => panic!("not a rejection: {other:?}"),
    }
}

fn run(f: &Fixture, seller: i64, buyer: i64, item: i64) -> RuleTest {
    rules::test(
        &f.env.ctx,
        &RuleTestInput {
            seller_id: seller,
            buyer_id: buyer,
            item_id: item,
            date: "10-10-2026".into(),
            qty: "20".into(),
            rate: "600".into(),
        },
    )
    .unwrap()
}

/// (level, amount in rupees) for one charge of a test result.
fn of(test: &RuleTest, side: ChargeSide, charge: ChargeType) -> (Option<RuleLevel>, i64) {
    let c = test
        .charges
        .iter()
        .find(|c| c.side == side && c.charge_type == charge)
        .unwrap();
    (c.winner.map(|w| w.level), c.amount_paisa.rupees())
}

fn count(dir: &TempDir, sql: &str) -> i64 {
    raw(dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn the_seeded_defaults_are_those_of_prd_4_2() {
    // Plan 09 Section 3, rows 1 and 2.
    let f = fixture();
    let charges = rules::charges(&f.env.ctx).unwrap();
    assert_eq!(
        (
            charges.seller_commission.method,
            charges.seller_commission.value_text.as_str()
        ),
        (ChargeMethod::Percent, "6")
    );
    assert_eq!(
        (
            charges.buyer_commission.method,
            charges.buyer_commission.value_text.as_str()
        ),
        (ChargeMethod::Percent, "6")
    );
    assert_eq!(
        (
            charges.laga_fruit.method,
            charges.laga_fruit.value_text.as_str()
        ),
        (ChargeMethod::PerUnit, "8")
    );
    let all = rules::list(
        &f.env.ctx,
        &RulesFilter {
            side: None,
            charge_type: None,
            account_id: None,
            item_id: None,
            include_inactive: true,
        },
    )
    .unwrap();
    assert_eq!(all.len(), 3);
    assert!(all.iter().all(|r| r.is_default && r.is_active));
    // 20 نگ × 600 = 12,000.
    let banana = run(&f, f.rahim, f.akram, f.banana);
    assert_eq!(banana.line_value_paisa, rupees(12_000));
    assert_eq!(
        of(&banana, ChargeSide::Seller, ChargeType::Commission),
        (Some(RuleLevel::Default), 720)
    );
    assert_eq!(
        of(&banana, ChargeSide::Buyer, ChargeType::Commission),
        (Some(RuleLevel::Default), 720)
    );
    assert_eq!(
        of(&banana, ChargeSide::Buyer, ChargeType::Laga),
        (Some(RuleLevel::ItemGroup), 160)
    );
    // Lot charges have no default: typed on the lot, so no rule and 0.
    for charge in [ChargeType::Labour, ChargeType::Munshiana, ChargeType::Phone] {
        assert_eq!(of(&banana, ChargeSide::Seller, charge), (None, 0));
    }
    // A vegetable carries no لاگا, with the working that says so.
    let mooli = run(&f, f.rahim, f.akram, f.mooli);
    assert_eq!(
        of(&mooli, ChargeSide::Seller, ChargeType::Commission).1,
        720
    );
    assert_eq!(of(&mooli, ChargeSide::Buyer, ChargeType::Commission).1, 720);
    let laga = mooli
        .charges
        .iter()
        .find(|c| c.charge_type == ChargeType::Laga)
        .unwrap();
    assert_eq!((laga.winner, laga.amount_paisa), (None, Paisa::ZERO));
    assert_eq!(laga.working.message_id, "working-none");
    assert_eq!(laga.skipped.len(), 6);
    assert_eq!(laga.skipped[0].reason, SkipReason::NoOverride);
    // The working of the 6% example carries its inputs (PRD FR-7.43).
    let sc = banana
        .charges
        .iter()
        .find(|c| c.side == ChargeSide::Seller && c.charge_type == ChargeType::Commission)
        .unwrap();
    assert_eq!(sc.working.message_id, "working-percent");
    assert_eq!(
        sc.working.args[0].value,
        WorkingValue::Money {
            paisa: rupees(12_000)
        }
    );
    assert_eq!(sc.working.args[1].value, WorkingValue::Percent { bps: 600 });
    assert!(
        sc.working
            .args
            .iter()
            .any(|a| a.name == "result" && a.value == WorkingValue::Money { paisa: rupees(720) })
    );
    assert_eq!(sc.working.rule_id, sc.winner.and_then(|w| w.rule_id));
    // The order of the charges on the test screen.
    let order: Vec<(ChargeSide, ChargeType)> = banana
        .charges
        .iter()
        .map(|c| (c.side, c.charge_type))
        .collect();
    assert_eq!(order[0], (ChargeSide::Seller, ChargeType::Commission));
    assert_eq!(order[5], (ChargeSide::Buyer, ChargeType::Laga));
}

#[test]
fn the_levels_of_prd_11_3_through_the_real_commands() {
    // Plan 09 Section 3, rows 3 to 7, on 20 نگ × 600 = 12,000.
    let f = fixture();
    // "اکرم buys at 7%": a buyer party rule beats the default.
    save(
        &f,
        &RuleInput {
            account_id: Some(f.akram),
            ..form(
                ChargeSide::Buyer,
                ChargeType::Commission,
                ChargeMethod::Percent,
                "7",
            )
        },
    );
    let t = run(&f, f.rahim, f.akram, f.banana);
    assert_eq!(
        of(&t, ChargeSide::Buyer, ChargeType::Commission),
        (Some(RuleLevel::Party), 840)
    );
    assert_eq!(
        of(&t, ChargeSide::Seller, ChargeType::Commission).1,
        720,
        "the seller side is separate"
    );
    // PRD 11.3 examples for the seller commission.
    save(
        &f,
        &seller_commission(&f, Some(f.rahim), Some(f.banana), "5"),
    );
    save(&f, &seller_commission(&f, Some(f.rahim), None, "4"));
    save(
        &f,
        &RuleInput {
            item_id: Some(f.japani),
            ..form(
                ChargeSide::Seller,
                ChargeType::Commission,
                ChargeMethod::PerUnit,
                "30",
            )
        },
    );
    let by = |seller, item| {
        of(
            &run(&f, seller, f.akram, item),
            ChargeSide::Seller,
            ChargeType::Commission,
        )
    };
    assert_eq!(by(f.rahim, f.banana), (Some(RuleLevel::PartyItem), 600));
    assert_eq!(by(f.rahim, f.apple), (Some(RuleLevel::Party), 480));
    assert_eq!(by(f.karim, f.japani), (Some(RuleLevel::Item), 600));
    assert_eq!(by(f.karim, f.banana), (Some(RuleLevel::Default), 720));
    // The skipped levels of the item rule: the line override and the two party levels.
    let t = run(&f, f.karim, f.akram, f.japani);
    let c = t
        .charges
        .iter()
        .find(|c| c.side == ChargeSide::Seller && c.charge_type == ChargeType::Commission)
        .unwrap();
    assert_eq!(
        c.skipped.iter().map(|s| s.level).collect::<Vec<_>>(),
        [
            RuleLevel::LineOverride,
            RuleLevel::PartyItem,
            RuleLevel::Party
        ]
    );
    assert_eq!(c.working.message_id, "working-per-unit");
    // The group rule for another group: a vegetable rule for ساگ-like items does not touch fruit.
    save(
        &f,
        &RuleInput {
            item_group: Some(ItemGroup::Vegetable),
            ..form(
                ChargeSide::Buyer,
                ChargeType::Laga,
                ChargeMethod::PerUnit,
                "3",
            )
        },
    );
    let t = run(&f, f.rahim, f.akram, f.mooli);
    assert_eq!(
        of(&t, ChargeSide::Buyer, ChargeType::Laga),
        (Some(RuleLevel::ItemGroup), 60)
    );
    let t = run(&f, f.rahim, f.akram, f.banana);
    assert_eq!(
        of(&t, ChargeSide::Buyer, ChargeType::Laga),
        (Some(RuleLevel::ItemGroup), 160)
    );
    // A lot charge rule: labour 5 per نگ received, munshiana 50 and phone 20 per lot (PRD 4.4).
    save(
        &f,
        &form(
            ChargeSide::Seller,
            ChargeType::Labour,
            ChargeMethod::PerUnit,
            "5",
        ),
    );
    save(
        &f,
        &form(
            ChargeSide::Seller,
            ChargeType::Munshiana,
            ChargeMethod::PerLot,
            "50",
        ),
    );
    save(
        &f,
        &form(
            ChargeSide::Seller,
            ChargeType::Phone,
            ChargeMethod::PerLot,
            "20",
        ),
    );
    let t = run(&f, f.rahim, f.akram, f.banana);
    assert_eq!(
        of(&t, ChargeSide::Seller, ChargeType::Labour),
        (Some(RuleLevel::Default), 100),
        "20 × 5 on this test's 20 نگ"
    );
    assert_eq!(of(&t, ChargeSide::Seller, ChargeType::Munshiana).1, 50);
    assert_eq!(of(&t, ChargeSide::Seller, ChargeType::Phone).1, 20);
    let labour = t
        .charges
        .iter()
        .find(|c| c.charge_type == ChargeType::Labour)
        .unwrap();
    assert!(labour.working.args.iter().any(|a| a.name == "qty_received"));
}

#[test]
fn a_changed_rule_is_used_at_once_and_an_inactive_rule_is_not() {
    let f = fixture();
    let saved = save(&f, &seller_commission(&f, Some(f.rahim), None, "4"));
    assert_eq!(
        run(&f, f.rahim, f.akram, f.banana).charges[0].amount_paisa,
        rupees(480)
    );
    // Edit it to 5%: the rules in memory are rebuilt after the commit.
    let edited = save(
        &f,
        &RuleInput {
            id: Some(saved.row.id),
            version: Some(saved.row.version),
            ..seller_commission(&f, Some(f.rahim), None, "5")
        },
    );
    assert_eq!(edited.row.version, saved.row.version + 1);
    assert_eq!(
        run(&f, f.rahim, f.akram, f.banana).charges[0].amount_paisa,
        rupees(600)
    );
    // Switch it off: the default decides again.
    let off = rules::set_active(&f.env.ctx, &f.owner, saved.row.id, false).unwrap();
    assert!(!off.row.is_active);
    assert_eq!(
        run(&f, f.rahim, f.akram, f.banana).charges[0].amount_paisa,
        rupees(720)
    );
    // Asking for the state it has writes nothing.
    let audits = audit_total(&f.dir);
    rules::set_active(&f.env.ctx, &f.owner, saved.row.id, false).unwrap();
    assert_eq!(audit_total(&f.dir), audits);
    rules::set_active(&f.env.ctx, &f.owner, saved.row.id, true).unwrap();
    assert_eq!(
        run(&f, f.rahim, f.akram, f.banana).charges[0].amount_paisa,
        rupees(600)
    );
    // The row reads back what was typed.
    let row = rules::get(&f.env.ctx, saved.row.id).unwrap();
    assert_eq!(
        (row.value_text.as_str(), row.level, row.is_default),
        ("5", RuleLevel::Party, false)
    );
    assert_eq!(row.account.map(|a| a.id), Some(f.rahim));
    assert_eq!(row.from_date, "2026-01-01");
}

#[test]
fn the_rule_form_refuses_what_prd_11_2_and_11_3_do_not_allow() {
    let f = fixture();
    let cases: Vec<(RuleInput, ErrorCode, &str)> = vec![
        // The seller's labour has no percent; the buyer has no labour, munshiana or phone.
        (
            form(
                ChargeSide::Seller,
                ChargeType::Labour,
                ChargeMethod::Percent,
                "5",
            ),
            ErrorCode::ErrRuleNotAllowed,
            "method",
        ),
        (
            form(
                ChargeSide::Buyer,
                ChargeType::Labour,
                ChargeMethod::PerUnit,
                "5",
            ),
            ErrorCode::ErrRuleNotAllowed,
            "charge_type",
        ),
        (
            form(
                ChargeSide::Seller,
                ChargeType::Laga,
                ChargeMethod::PerUnit,
                "5",
            ),
            ErrorCode::ErrRuleNotAllowed,
            "charge_type",
        ),
        // The buyer's commission has no per-lot.
        (
            form(
                ChargeSide::Buyer,
                ChargeType::Commission,
                ChargeMethod::PerLot,
                "5",
            ),
            ErrorCode::ErrRuleNotAllowed,
            "method",
        ),
        // Values.
        (
            form(
                ChargeSide::Seller,
                ChargeType::Commission,
                ChargeMethod::Percent,
                "101",
            ),
            ErrorCode::ErrPercentOver100,
            "value",
        ),
        (
            form(
                ChargeSide::Seller,
                ChargeType::Commission,
                ChargeMethod::Percent,
                "2.555",
            ),
            ErrorCode::ErrTooManyDecimals,
            "value",
        ),
        (
            form(
                ChargeSide::Seller,
                ChargeType::Commission,
                ChargeMethod::PerUnit,
                "8.50",
            ),
            ErrorCode::ErrDecimalsNotAllowed,
            "value",
        ),
        (
            form(
                ChargeSide::Seller,
                ChargeType::Commission,
                ChargeMethod::PerUnit,
                "abc",
            ),
            ErrorCode::ErrNotANumber,
            "value",
        ),
        (
            form(
                ChargeSide::Seller,
                ChargeType::Commission,
                ChargeMethod::Percent,
                "",
            ),
            ErrorCode::ErrRequired,
            "value",
        ),
        // Dates.
        (
            RuleInput {
                to_date: "31-12-2025".into(),
                ..form(
                    ChargeSide::Seller,
                    ChargeType::Commission,
                    ChargeMethod::Percent,
                    "5",
                )
            },
            ErrorCode::ErrDateRange,
            "to_date",
        ),
        (
            RuleInput {
                from_date: "31-02-2026".into(),
                ..form(
                    ChargeSide::Seller,
                    ChargeType::Commission,
                    ChargeMethod::Percent,
                    "5",
                )
            },
            ErrorCode::ErrDateInvalid,
            "from_date",
        ),
        // A group together with an item or a party has no level in PRD 11.3.
        (
            RuleInput {
                item_group: Some(ItemGroup::Fruit),
                item_id: Some(f.banana),
                ..form(
                    ChargeSide::Seller,
                    ChargeType::Commission,
                    ChargeMethod::Percent,
                    "5",
                )
            },
            ErrorCode::ErrRuleNotAllowed,
            "item_group",
        ),
        (
            RuleInput {
                item_group: Some(ItemGroup::Fruit),
                account_id: Some(f.rahim),
                ..form(
                    ChargeSide::Seller,
                    ChargeType::Commission,
                    ChargeMethod::Percent,
                    "5",
                )
            },
            ErrorCode::ErrRuleNotAllowed,
            "item_group",
        ),
        // A party on the wrong side: a buyer on the seller side, and a seller on the buyer side.
        (
            RuleInput {
                account_id: Some(f.akram),
                ..form(
                    ChargeSide::Seller,
                    ChargeType::Commission,
                    ChargeMethod::Percent,
                    "5",
                )
            },
            ErrorCode::ErrRuleNotAllowed,
            "account_id",
        ),
        (
            RuleInput {
                account_id: Some(f.rahim),
                ..form(
                    ChargeSide::Buyer,
                    ChargeType::Commission,
                    ChargeMethod::Percent,
                    "5",
                )
            },
            ErrorCode::ErrRuleNotAllowed,
            "account_id",
        ),
        // Things that do not exist.
        (
            RuleInput {
                account_id: Some(999_999),
                ..form(
                    ChargeSide::Seller,
                    ChargeType::Commission,
                    ChargeMethod::Percent,
                    "5",
                )
            },
            ErrorCode::ErrNotFound,
            "account_id",
        ),
        (
            RuleInput {
                item_id: Some(999_999),
                ..form(
                    ChargeSide::Seller,
                    ChargeType::Commission,
                    ChargeMethod::Percent,
                    "5",
                )
            },
            ErrorCode::ErrNotFound,
            "item_id",
        ),
        // A second default: the Charges tab shows the one that is in force.
        (
            form(
                ChargeSide::Seller,
                ChargeType::Commission,
                ChargeMethod::Percent,
                "5",
            ),
            ErrorCode::ErrRuleNotAllowed,
            "charge_type",
        ),
        (
            form(
                ChargeSide::Buyer,
                ChargeType::Commission,
                ChargeMethod::PerUnit,
                "5",
            ),
            ErrorCode::ErrRuleNotAllowed,
            "charge_type",
        ),
        (
            RuleInput {
                item_group: Some(ItemGroup::Fruit),
                ..form(
                    ChargeSide::Buyer,
                    ChargeType::Laga,
                    ChargeMethod::PerUnit,
                    "9",
                )
            },
            ErrorCode::ErrRuleNotAllowed,
            "item_group",
        ),
    ];
    let before = count(&f.dir, "SELECT COUNT(*) FROM charge_rules");
    for (input, code, field) in cases {
        let (got, at) = refused(&f, &input);
        assert_eq!(
            (got, at.as_deref()),
            (code, Some(field)),
            "{:?} {:?} {}",
            input.side,
            input.charge_type,
            input.value
        );
    }
    assert_eq!(
        count(&f.dir, "SELECT COUNT(*) FROM charge_rules"),
        before,
        "a refused save writes nothing"
    );
    // Several mistakes come back together.
    let ServiceError::Rejected(items) = rules::save(
        &f.env.ctx,
        &f.owner,
        &RuleInput {
            to_date: "01-01-2020".into(),
            ..form(
                ChargeSide::Buyer,
                ChargeType::Labour,
                ChargeMethod::PerUnit,
                "x",
            )
        },
    )
    .unwrap_err() else {
        panic!("not a rejection")
    };
    assert!(items.len() >= 2);
    // Zero percent and a zero per-lot value are allowed.
    save(&f, &seller_commission(&f, Some(f.rahim), None, "0"));
    save(
        &f,
        &form(
            ChargeSide::Seller,
            ChargeType::Munshiana,
            ChargeMethod::PerLot,
            "0",
        ),
    );
    // The پختہ کھاتہ accounts take rules like any party (PRD FR-11.7).
    let pukhta_buyer = f
        .env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, "03049999"))
        .unwrap()
        .unwrap();
    save(
        &f,
        &RuleInput {
            account_id: Some(pukhta_buyer),
            ..form(
                ChargeSide::Buyer,
                ChargeType::Commission,
                ChargeMethod::Percent,
                "7",
            )
        },
    );
    // A stale version is refused.
    let saved = save(&f, &seller_commission(&f, Some(f.karim), None, "3"));
    let stale = RuleInput {
        id: Some(saved.row.id),
        version: Some(saved.row.version + 5),
        ..seller_commission(&f, Some(f.karim), None, "3.5")
    };
    assert_eq!(refused(&f, &stale).0, ErrorCode::ErrVersionChanged);
    let (code, field) = refused(
        &f,
        &RuleInput {
            id: Some(saved.row.id),
            version: None,
            ..seller_commission(&f, Some(f.karim), None, "3.5")
        },
    );
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrRequired, Some("version"))
    );
    let (code, field) = refused(
        &f,
        &RuleInput {
            id: Some(999_999),
            version: Some(1),
            ..seller_commission(&f, Some(f.karim), None, "3.5")
        },
    );
    assert_eq!(
        (code, field.as_deref()),
        (ErrorCode::ErrNotFound, Some("id"))
    );
}

#[test]
fn the_defaults_are_changed_only_in_settings() {
    let f = fixture();
    let charges = rules::charges(&f.env.ctx).unwrap();
    let id = charges.seller_commission.rule_id;
    let edit = RuleInput {
        id: Some(id),
        version: Some(charges.seller_commission.version),
        ..form(
            ChargeSide::Seller,
            ChargeType::Commission,
            ChargeMethod::Percent,
            "5",
        )
    };
    assert_eq!(
        refused(&f, &edit),
        (ErrorCode::ErrRuleNotAllowed, Some("id".to_owned()))
    );
    let error = rules::set_active(&f.env.ctx, &f.owner, id, false).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrRuleNotAllowed);
    assert!(rules::get(&f.env.ctx, id).unwrap().is_active);
    // Switching a default "on" when it is on is a no-op, not an error.
    rules::set_active(&f.env.ctx, &f.owner, id, true).unwrap();
}

#[test]
fn rules_with_dates_that_meet_are_warned_about_and_the_later_start_wins() {
    // PRD FR-11.5: overlapping rules at the same level; the newer one wins and the form warns.
    let f = fixture();
    let first = save(
        &f,
        &RuleInput {
            from_date: "01-10-2026".into(),
            to_date: "31-10-2026".into(),
            ..seller_commission(&f, Some(f.rahim), None, "4")
        },
    );
    assert!(first.overlaps.is_empty());
    // Starts the day the first ends: they meet on 31-10.
    let second = save(
        &f,
        &RuleInput {
            from_date: "31-10-2026".into(),
            ..seller_commission(&f, Some(f.rahim), None, "4.5")
        },
    );
    assert_eq!(
        second
            .overlaps
            .iter()
            .map(|o| (o.id, o.from_date.as_str(), o.to_date.as_deref()))
            .collect::<Vec<_>>(),
        [(first.row.id, "2026-10-01", Some("2026-10-31"))]
    );
    // The day after does not meet it.
    let third = save(
        &f,
        &RuleInput {
            from_date: "01-12-2026".into(),
            to_date: "31-12-2026".into(),
            ..seller_commission(&f, Some(f.rahim), None, "5")
        },
    );
    assert_eq!(
        third.overlaps.len(),
        1,
        "it meets the open-ended second rule"
    );
    assert_eq!(third.overlaps[0].id, second.row.id);
    // Another party's rule, an inactive rule and a rule of another level never overlap.
    assert!(
        save(&f, &seller_commission(&f, Some(f.karim), None, "4"))
            .overlaps
            .is_empty()
    );
    assert!(
        save(
            &f,
            &seller_commission(&f, Some(f.rahim), Some(f.banana), "4")
        )
        .overlaps
        .is_empty()
    );
    // Switching the inactive first rule on warns about the second.
    rules::set_active(&f.env.ctx, &f.owner, first.row.id, false).unwrap();
    let on = rules::set_active(&f.env.ctx, &f.owner, first.row.id, true).unwrap();
    assert_eq!(
        on.overlaps.iter().map(|o| o.id).collect::<Vec<_>>(),
        [second.row.id]
    );
    // On 31-10 the second (starts later) decides; on 15-10 the first.
    let on_day = |d: &str| {
        rules::test(
            &f.env.ctx,
            &RuleTestInput {
                seller_id: f.rahim,
                buyer_id: f.akram,
                item_id: f.apple,
                date: d.into(),
                qty: "20".into(),
                rate: "600".into(),
            },
        )
        .unwrap()
        .charges[0]
            .winner
            .and_then(|w| w.rule_id)
    };
    assert_eq!(on_day("15-10-2026"), Some(first.row.id));
    assert_eq!(on_day("31-10-2026"), Some(second.row.id));
    assert_eq!(on_day("01-12-2026"), Some(third.row.id));
}

#[test]
fn a_default_is_dated_like_any_rule_and_a_change_starts_today() {
    // Lead decision 12-10-2026 (PRD 11.3: the rule date is the posting date). "Today" is
    // 10-10-2026 in the test clock. E2E-13b: the Owner changes the buyer commission to 7%.
    let f = fixture();
    let before = rules::charges(&f.env.ctx).unwrap();
    assert_eq!(before.buyer_commission.from_date, "1000-01-01");
    let input = |c: &mandi_services::core::rules::ChargesSettings,
                 seller: &str,
                 buyer: &str,
                 laga: &str| ChargesInput {
        seller_commission: DefaultChargeInput {
            version: c.seller_commission.version,
            method: ChargeMethod::Percent,
            value: seller.into(),
        },
        buyer_commission: DefaultChargeInput {
            version: c.buyer_commission.version,
            method: ChargeMethod::Percent,
            value: buyer.into(),
        },
        laga_fruit: DefaultChargeInput {
            version: c.laga_fruit.version,
            method: ChargeMethod::PerUnit,
            value: laga.into(),
        },
    };
    let rule_count = || count(&f.dir, "SELECT COUNT(*) FROM charge_rules");
    assert_eq!(rule_count(), 3);
    let audits = audit_total(&f.dir);
    let after = rules::set_charges(&f.env.ctx, &f.owner, &input(&before, "6", "7", "8")).unwrap();
    // A new row from today; the old one ends on the day before.
    assert_eq!(rule_count(), 4);
    assert_eq!(
        (
            after.buyer_commission.value_text.as_str(),
            after.buyer_commission.from_date.as_str()
        ),
        ("7", "2026-10-10")
    );
    assert_ne!(
        after.buyer_commission.rule_id,
        before.buyer_commission.rule_id
    );
    assert_eq!(
        after.buyer_commission.version, 3,
        "a new row starts above the old row's version, so a stale form never matches it"
    );
    assert_eq!(
        after.seller_commission, before.seller_commission,
        "unchanged defaults are not touched"
    );
    let old_end: Option<String> = raw(&f.dir)
        .query_row(
            "SELECT to_date FROM charge_rules WHERE id = ?1",
            [before.buyer_commission.rule_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(old_end.as_deref(), Some("2026-10-09"));
    assert_eq!(
        audit_total(&f.dir),
        audits + 2,
        "one row ended, one row made"
    );
    let (old, new): (String, String) = raw(&f.dir)
        .query_row("SELECT old_json, new_json FROM audit_log WHERE entity = 'charge_rule' AND action = 'create' ORDER BY id DESC LIMIT 1", [], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap();
    assert!(
        old.contains("\"value_bps\":600") && new.contains("\"value_bps\":700"),
        "{old} {new}"
    );
    // A sale dated today uses 7%: 12,000 x 7% = 840 (E2E-13b); one dated before the change keeps
    // 6% (720), also a day on which the old default began, and far back.
    let on = |date: &str| {
        let t = rules::test(
            &f.env.ctx,
            &RuleTestInput {
                seller_id: f.rahim,
                buyer_id: f.akram,
                item_id: f.banana,
                date: date.into(),
                qty: "20".into(),
                rate: "600".into(),
            },
        )
        .unwrap();
        of(&t, ChargeSide::Buyer, ChargeType::Commission)
    };
    assert_eq!(on("10-10-2026"), (Some(RuleLevel::Default), 840));
    assert_eq!(on("09-10-2026"), (Some(RuleLevel::Default), 720));
    assert_eq!(on("01-01-2026"), (Some(RuleLevel::Default), 720));
    // The charges tab shows the default in force today.
    assert_eq!(rules::charges(&f.env.ctx).unwrap(), after);
    // A second change on the same day updates that day's row; no row is added.
    let again = rules::set_charges(&f.env.ctx, &f.owner, &input(&after, "6", "8", "8")).unwrap();
    assert_eq!(rule_count(), 4);
    assert_eq!(
        again.buyer_commission.rule_id,
        after.buyer_commission.rule_id
    );
    assert_eq!(
        (
            again.buyer_commission.value_text.as_str(),
            again.buyer_commission.version
        ),
        ("8", 4)
    );
    assert_eq!(on("10-10-2026").1, 960);
    assert_eq!(
        on("09-10-2026").1,
        720,
        "yesterday still has the first value"
    );
    // Saving the same values again changes nothing.
    let audits = audit_total(&f.dir);
    assert_eq!(
        rules::set_charges(&f.env.ctx, &f.owner, &input(&again, "6", "8", "8")).unwrap(),
        again
    );
    assert_eq!(audit_total(&f.dir), audits);
    // The next day another change: the row of the 10th ends on the 10th.
    set_today(&f.env, 2026, 10, 11);
    let next = rules::charges(&f.env.ctx).unwrap();
    assert_eq!(
        next.buyer_commission.from_date, "2026-10-10",
        "still in force"
    );
    let third = rules::set_charges(&f.env.ctx, &f.owner, &input(&next, "6", "5", "8")).unwrap();
    assert_eq!(rule_count(), 5);
    assert_eq!(
        (
            third.buyer_commission.value_text.as_str(),
            third.buyer_commission.from_date.as_str()
        ),
        ("5", "2026-10-11")
    );
    assert_eq!(on("11-10-2026").1, 600);
    assert_eq!(on("10-10-2026").1, 960);
    assert_eq!(on("09-10-2026").1, 720);
    // Every row of the chain is a default: it cannot be edited or switched off, old ones too.
    let rows = rules::list(
        &f.env.ctx,
        &RulesFilter {
            side: Some(ChargeSide::Buyer),
            charge_type: Some(ChargeType::Commission),
            account_id: None,
            item_id: None,
            include_inactive: true,
        },
    )
    .unwrap();
    assert_eq!(rows.len(), 3);
    assert!(
        rows.iter()
            .all(|r| r.is_default && r.level == RuleLevel::Default)
    );
    assert_eq!(rows[0].from_date, "2026-10-11", "newest first");
    let ended = rows.iter().find(|r| r.to_date.is_some()).unwrap();
    assert_eq!(
        rules::set_active(&f.env.ctx, &f.owner, ended.id, false)
            .unwrap_err()
            .code(),
        ErrorCode::ErrRuleNotAllowed
    );
    assert_eq!(
        refused(
            &f,
            &RuleInput {
                id: Some(ended.id),
                version: Some(ended.version),
                ..form(
                    ChargeSide::Buyer,
                    ChargeType::Commission,
                    ChargeMethod::Percent,
                    "9"
                )
            }
        )
        .0,
        ErrorCode::ErrRuleNotAllowed
    );
    // The seller commission and the fruit لاگا were not touched.
    assert_eq!(
        rules::charges(&f.env.ctx).unwrap().seller_commission,
        before.seller_commission
    );
    assert_chain_ok(&f.dir);
}

#[test]
fn the_three_defaults_can_be_other_methods_and_zero() {
    let f = fixture();
    let before = rules::charges(&f.env.ctx).unwrap();
    // The seller commission as a fixed amount per نگ, the buyer commission as 2.5%, and لاگا as
    // a percent; 0 is allowed ("no لاگا", lead decision 12-10-2026).
    let changed = rules::set_charges(
        &f.env.ctx,
        &f.owner,
        &ChargesInput {
            seller_commission: DefaultChargeInput {
                version: before.seller_commission.version,
                method: ChargeMethod::PerUnit,
                value: "30".into(),
            },
            buyer_commission: DefaultChargeInput {
                version: before.buyer_commission.version,
                method: ChargeMethod::Percent,
                value: "2.5".into(),
            },
            laga_fruit: DefaultChargeInput {
                version: before.laga_fruit.version,
                method: ChargeMethod::PerUnit,
                value: "0".into(),
            },
        },
    )
    .unwrap();
    assert_eq!(
        (
            changed.seller_commission.method,
            changed.seller_commission.value_text.as_str()
        ),
        (ChargeMethod::PerUnit, "30")
    );
    assert_eq!(changed.laga_fruit.value_text, "0");
    let t = run(&f, f.rahim, f.akram, f.banana);
    assert_eq!(
        of(&t, ChargeSide::Seller, ChargeType::Commission).1,
        600,
        "20 x 30"
    );
    assert_eq!(
        of(&t, ChargeSide::Buyer, ChargeType::Commission).1,
        300,
        "2.5% of 12,000"
    );
    assert_eq!(
        of(&t, ChargeSide::Buyer, ChargeType::Laga),
        (Some(RuleLevel::ItemGroup), 0)
    );
    // A rule form accepts 0 per نگ and 0 per lot too: "no لاگا for this buyer" under the fruit
    // group rule, and a free lot charge.
    save(
        &f,
        &RuleInput {
            account_id: Some(f.akram),
            ..form(
                ChargeSide::Buyer,
                ChargeType::Laga,
                ChargeMethod::PerUnit,
                "0",
            )
        },
    );
    save(
        &f,
        &form(
            ChargeSide::Seller,
            ChargeType::Labour,
            ChargeMethod::PerUnit,
            "0",
        ),
    );
    assert_eq!(
        refused(
            &f,
            &form(
                ChargeSide::Seller,
                ChargeType::Labour,
                ChargeMethod::PerUnit,
                "-1"
            )
        )
        .0,
        ErrorCode::ErrNegativeNotAllowed
    );
}

#[test]
fn settings_charges_check_every_value_and_name_the_field() {
    let f = fixture();
    let before = rules::charges(&f.env.ctx).unwrap();
    let ok = |c: &mandi_services::core::rules::DefaultCharge| DefaultChargeInput {
        version: c.version,
        method: c.method,
        value: c.value_text.clone(),
    };
    let audits = audit_total(&f.dir);
    let bad = ChargesInput {
        // 150 percent; the buyer commission has no per-lot method; a negative laga.
        seller_commission: DefaultChargeInput {
            value: "150".into(),
            ..ok(&before.seller_commission)
        },
        buyer_commission: DefaultChargeInput {
            method: ChargeMethod::PerLot,
            ..ok(&before.buyer_commission)
        },
        laga_fruit: DefaultChargeInput {
            value: "-1".into(),
            ..ok(&before.laga_fruit)
        },
    };
    let ServiceError::Rejected(items) = rules::set_charges(&f.env.ctx, &f.owner, &bad).unwrap_err()
    else {
        panic!("not a rejection")
    };
    let seen: Vec<(ErrorCode, String)> = items
        .into_iter()
        .map(|i| (i.code, i.field.unwrap_or_default()))
        .collect();
    assert!(
        seen.contains(&(
            ErrorCode::ErrPercentOver100,
            "seller_commission.value".into()
        )),
        "{seen:?}"
    );
    assert!(
        seen.contains(&(
            ErrorCode::ErrRuleNotAllowed,
            "buyer_commission.method".into()
        )),
        "{seen:?}"
    );
    assert!(
        seen.contains(&(ErrorCode::ErrNegativeNotAllowed, "laga_fruit.value".into())),
        "{seen:?}"
    );
    assert_eq!(audit_total(&f.dir), audits);
    // A stale version names its default.
    let stale = ChargesInput {
        seller_commission: ok(&before.seller_commission),
        buyer_commission: DefaultChargeInput {
            version: before.buyer_commission.version + 3,
            ..ok(&before.buyer_commission)
        },
        laga_fruit: ok(&before.laga_fruit),
    };
    let ServiceError::Rejected(items) =
        rules::set_charges(&f.env.ctx, &f.owner, &stale).unwrap_err()
    else {
        panic!("not a rejection")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (
            ErrorCode::ErrVersionChanged,
            Some("buyer_commission.version")
        )
    );
    assert_eq!(
        rules::charges(&f.env.ctx).unwrap(),
        before,
        "nothing changed"
    );
}

#[test]
fn the_overrides_start_on_and_are_kept_with_an_audit_row() {
    // PRD FR-11.10 v1.5 and Section 21 "Overrides": a typed commission or لاگا is allowed and a
    // credit limit warns (A18), until the Owner changes them.
    let f = fixture();
    let start = rules::overrides(&f.env.ctx).unwrap();
    assert_eq!(
        (start.allow_line_override, start.credit_limit_mode),
        (true, CreditLimitMode::Warn)
    );
    // Plan 12: the rate check starts on at 30% (PRD FR-7.55, E8).
    assert_eq!(
        (start.rate_check_enabled, start.rate_check_percent),
        (true, 30)
    );
    let audits = audit_total(&f.dir);
    let want = OverridesSettings {
        allow_line_override: false,
        credit_limit_mode: CreditLimitMode::Block,
        rate_check_enabled: false,
        rate_check_percent: 50,
    };
    assert_eq!(
        rules::set_overrides(&f.env.ctx, &f.owner, &want).unwrap(),
        want
    );
    assert_eq!(rules::overrides(&f.env.ctx).unwrap(), want);
    assert_eq!(audit_total(&f.dir), audits + 1);
    rules::set_overrides(&f.env.ctx, &f.owner, &want).unwrap();
    assert_eq!(
        audit_total(&f.dir),
        audits + 1,
        "the same values write nothing"
    );
    let (old, new): (String, String) = raw(&f.dir)
        .query_row("SELECT old_json, new_json FROM audit_log WHERE entity = 'settings' ORDER BY id DESC LIMIT 1", [], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap();
    assert!(
        old.contains("\"allow_line_override\":true")
            && new.contains("\"allow_line_override\":false")
            && new.contains("\"credit_limit_mode\":\"block\""),
        "{old} {new}"
    );
    // A value the Owner stored as false stays false, even when the other key is missing.
    let changed = raw(&f.dir)
        .execute(
            "UPDATE settings SET value_json = '{\"allow_line_override\":false}' WHERE key = 'overrides'",
            [],
        )
        .unwrap();
    assert_eq!(changed, 1, "the saved setting row exists");
    let kept = rules::overrides(&f.env.ctx).unwrap();
    assert_eq!(
        (kept.allow_line_override, kept.credit_limit_mode),
        (false, CreditLimitMode::Warn)
    );
    // A row saved before plan 12 has no rate check keys: they read as the starting position.
    assert_eq!(
        (kept.rate_check_enabled, kept.rate_check_percent),
        (true, 30)
    );
    // A stored `false` and a stored percent are kept as stored.
    raw(&f.dir)
        .execute(
            "UPDATE settings SET value_json = '{\"rate_check_enabled\":false,\"rate_check_percent\":45}' WHERE key = 'overrides'",
            [],
        )
        .unwrap();
    let kept = rules::overrides(&f.env.ctx).unwrap();
    assert_eq!(
        (kept.rate_check_enabled, kept.rate_check_percent),
        (false, 45)
    );
    // A stored percent outside 1 to 1,000 (written outside the app) is not used: the start is.
    raw(&f.dir)
        .execute(
            "UPDATE settings SET value_json = '{\"rate_check_enabled\":true,\"rate_check_percent\":0}' WHERE key = 'overrides'",
            [],
        )
        .unwrap();
    assert_eq!(rules::overrides(&f.env.ctx).unwrap().rate_check_percent, 30);
    raw(&f.dir)
        .execute(
            "UPDATE settings SET value_json = '{\"rate_check_enabled\":true,\"rate_check_percent\":1001}' WHERE key = 'overrides'",
            [],
        )
        .unwrap();
    assert_eq!(rules::overrides(&f.env.ctx).unwrap().rate_check_percent, 30);
    raw(&f.dir)
        .execute(
            "UPDATE settings SET value_json = '{\"rate_check_enabled\":false,\"rate_check_percent\":45}' WHERE key = 'overrides'",
            [],
        )
        .unwrap();
    // A percent outside 1 to 1,000 is refused, and nothing is written.
    for bad in [0, 1_001] {
        let refused = rules::set_overrides(
            &f.env.ctx,
            &f.owner,
            &OverridesSettings {
                rate_check_percent: bad,
                ..want
            },
        )
        .unwrap_err();
        assert!(refused.has_code(ErrorCode::ErrOutOfRange), "{bad}");
    }
    assert_eq!(rules::overrides(&f.env.ctx).unwrap().rate_check_percent, 45);
}

#[test]
fn rule_changes_are_audited_and_the_test_checks_its_input() {
    let f = fixture();
    let saved = save(
        &f,
        &seller_commission(&f, Some(f.rahim), Some(f.banana), "5"),
    );
    rules::save(
        &f.env.ctx,
        &f.owner,
        &RuleInput {
            id: Some(saved.row.id),
            version: Some(saved.row.version),
            ..seller_commission(&f, Some(f.rahim), Some(f.banana), "4.5")
        },
    )
    .unwrap();
    rules::set_active(&f.env.ctx, &f.owner, saved.row.id, false).unwrap();
    assert_eq!(
        count(
            &f.dir,
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'charge_rule' AND action = 'create'"
        ),
        1
    );
    assert_eq!(
        count(
            &f.dir,
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'charge_rule' AND action = 'edit'"
        ),
        2
    );
    assert_chain_ok(&f.dir);
    // The test screen's input is checked.
    let ask = |seller: i64, buyer: i64, item: i64, date: &str, qty: &str, rate: &str| {
        rules::test(
            &f.env.ctx,
            &RuleTestInput {
                seller_id: seller,
                buyer_id: buyer,
                item_id: item,
                date: date.into(),
                qty: qty.into(),
                rate: rate.into(),
            },
        )
        .map(|_| ())
        .unwrap_err()
    };
    let field_of = |e: ServiceError| match e {
        ServiceError::Rejected(items) => {
            (items[0].code, items[0].field.clone().unwrap_or_default())
        }
        other => panic!("not a rejection: {other:?}"),
    };
    assert_eq!(
        field_of(ask(f.rahim, f.akram, f.banana, "10-10-2026", "0", "600")),
        (ErrorCode::ErrMustBeMoreThanZero, "qty".into())
    );
    assert_eq!(
        field_of(ask(f.rahim, f.akram, f.banana, "10-10-2026", "20", "60.5")),
        (ErrorCode::ErrDecimalsNotAllowed, "rate".into())
    );
    assert_eq!(
        field_of(ask(f.rahim, f.akram, f.banana, "31-02-2026", "20", "600")),
        (ErrorCode::ErrDateInvalid, "date".into())
    );
    assert_eq!(
        field_of(ask(f.akram, f.akram, f.banana, "10-10-2026", "20", "600")),
        (ErrorCode::ErrFormatInvalid, "seller_id".into())
    );
    assert_eq!(
        field_of(ask(f.rahim, f.rahim, f.banana, "10-10-2026", "20", "600")),
        (ErrorCode::ErrFormatInvalid, "buyer_id".into())
    );
    assert_eq!(
        field_of(ask(f.rahim, f.akram, 999_999, "10-10-2026", "20", "600")),
        (ErrorCode::ErrNotFound, "item_id".into())
    );
    // The test accepts a simple sum like every quantity field (PRD FR-7.48), and any date.
    let sum = rules::test(
        &f.env.ctx,
        &RuleTestInput {
            seller_id: f.rahim,
            buyer_id: f.akram,
            item_id: f.banana,
            date: "01-01-2026".into(),
            qty: "10+10".into(),
            rate: "600".into(),
        },
    )
    .unwrap();
    assert_eq!(sum.line_value_paisa, rupees(12_000));
}

/// The first rejection of an error: (code, field, the `max` value).
fn first_with_max(error: ServiceError) -> (ErrorCode, String, Option<ErrorValue>) {
    match error {
        ServiceError::Rejected(items) => (
            items[0].code,
            items[0].field.clone().unwrap_or_default(),
            items[0].values.get("max").cloned(),
        ),
        other => panic!("not a rejection: {other:?}"),
    }
}

#[test]
fn the_decimal_and_quantity_errors_of_the_rule_screens_carry_their_maximum() {
    // End-gate review 13: the messages say "at most { $max } digits" and "the most is { $max }",
    // so every screen that takes a percent or a quantity must send `max` (the rule form, the
    // three defaults of Settings, and the "test a rule" box).
    let f = fixture();
    let two = ErrorValue::Number(2);
    let (code, field, max) = first_with_max(
        rules::save(
            &f.env.ctx,
            &f.owner,
            &form(
                ChargeSide::Seller,
                ChargeType::Commission,
                ChargeMethod::Percent,
                "2.555",
            ),
        )
        .unwrap_err(),
    );
    assert_eq!(
        (code, field.as_str(), max),
        (ErrorCode::ErrTooManyDecimals, "value", Some(two.clone()))
    );

    let before = rules::charges(&f.env.ctx).unwrap();
    let bad = ChargesInput {
        seller_commission: DefaultChargeInput {
            version: before.seller_commission.version,
            method: ChargeMethod::Percent,
            value: "6.125".into(),
        },
        buyer_commission: DefaultChargeInput {
            version: before.buyer_commission.version,
            method: before.buyer_commission.method,
            value: before.buyer_commission.value_text.clone(),
        },
        laga_fruit: DefaultChargeInput {
            version: before.laga_fruit.version,
            method: before.laga_fruit.method,
            value: before.laga_fruit.value_text.clone(),
        },
    };
    let (code, field, max) =
        first_with_max(rules::set_charges(&f.env.ctx, &f.owner, &bad).unwrap_err());
    assert_eq!(
        (code, field.as_str(), max),
        (
            ErrorCode::ErrTooManyDecimals,
            "seller_commission.value",
            Some(two)
        )
    );

    let ask = |qty: &str| {
        rules::test(
            &f.env.ctx,
            &RuleTestInput {
                seller_id: f.rahim,
                buyer_id: f.akram,
                item_id: f.banana,
                date: "10-10-2026".into(),
                qty: qty.into(),
                rate: "600".into(),
            },
        )
        .unwrap_err()
    };
    let (code, field, max) = first_with_max(ask("1000001"));
    assert_eq!(
        (code, field.as_str(), max),
        (
            ErrorCode::ErrQtyTooLarge,
            "qty",
            Some(ErrorValue::Number(1_000_000))
        )
    );
    // The same limit holds for a sum that goes over it.
    let (code, _, max) = first_with_max(ask("999999+2"));
    assert_eq!(
        (code, max),
        (
            ErrorCode::ErrQtyTooLarge,
            Some(ErrorValue::Number(1_000_000))
        )
    );
}

#[test]
fn ten_thousand_rules_are_filed_and_looked_up_fast() {
    // NFR-SCL-6: under 1 ms for a lookup with 10,000 rules. A generous bound for a debug build;
    // `pnpm perf` holds the real number.
    let f = fixture();
    {
        let conn = raw(&f.dir);
        conn.execute_batch("BEGIN").unwrap();
        let mut statement = conn
            .prepare(
                "INSERT INTO charge_rules (side, charge_type, account_id, item_id, method, value_bps, from_date)
                 VALUES ('seller', 'commission', ?1, ?2, 'percent', ?3, '2026-01-01')",
            )
            .unwrap();
        // Party + item rules for the one seller and the items that exist, with different dates.
        for n in 0..10_000_i64 {
            let item = [f.banana, f.apple, f.mooli, f.japani][usize::try_from(n % 4).unwrap()];
            statement.execute((f.karim, item, n % 1_000)).unwrap();
        }
        drop(statement);
        conn.execute_batch("COMMIT").unwrap();
    }
    // The cache was built before these rows existed; any rule change would rebuild it. Throw it
    // away the way a save does.
    rules::save(
        &f.env.ctx,
        &f.owner,
        &seller_commission(&f, Some(f.rahim), None, "4"),
    )
    .unwrap();
    let started = Instant::now();
    let set = rules::rule_set(&f.env.ctx).unwrap();
    let build = started.elapsed();
    assert!(set.len() >= 10_000);
    let started = Instant::now();
    let rounds = 200;
    for _ in 0..rounds {
        run(&f, f.karim, f.akram, f.banana);
    }
    let per_test = started.elapsed() / rounds;
    eprintln!(
        "rule set of {} built in {build:?}; one rules_test with all six charges {per_test:?} (debug build)",
        set.len()
    );
    assert!(per_test.as_millis() < 100, "{per_test:?}");
    assert!(build.as_millis() < 2_000, "{build:?}");
}

#[test]
fn a_posting_in_any_year_the_app_accepts_has_a_default() {
    // Money review: a year typo must never find no default and make a commission silently 0.
    let f = fixture();
    for date in ["01-01-1950", "31-12-1999", "01-01-2000", "01-01-1000"] {
        let t = rules::test(
            &f.env.ctx,
            &RuleTestInput {
                seller_id: f.rahim,
                buyer_id: f.akram,
                item_id: f.banana,
                date: date.into(),
                qty: "20".into(),
                rate: "600".into(),
            },
        )
        .unwrap();
        assert_eq!(
            of(&t, ChargeSide::Seller, ChargeType::Commission),
            (Some(RuleLevel::Default), 720),
            "{date}"
        );
        assert_eq!(
            of(&t, ChargeSide::Buyer, ChargeType::Laga),
            (Some(RuleLevel::ItemGroup), 160),
            "{date}"
        );
    }
}

#[test]
fn a_default_cannot_be_switched_off_or_deleted_even_outside_the_form() {
    let f = fixture();
    let conn = raw(&f.dir);
    assert!(
        conn.execute(
            "UPDATE charge_rules SET is_active = 0 WHERE charge_type = 'commission'",
            []
        )
        .is_err()
    );
    assert!(
        conn.execute(
            "DELETE FROM charge_rules WHERE charge_type = 'commission'",
            []
        )
        .is_err()
    );
    assert!(
        conn.execute(
            "UPDATE charge_rules SET is_active = 0 WHERE charge_type = 'laga'",
            []
        )
        .is_err()
    );
    assert_eq!(
        count(
            &f.dir,
            "SELECT COUNT(*) FROM charge_rules WHERE is_active = 1"
        ),
        3
    );
}

#[test]
fn a_change_at_the_turn_of_a_month_a_year_and_a_leap_day_ends_the_row_on_the_right_day() {
    // Money review: the old row ends the day before, whatever day that is.
    let f = fixture();
    let mut c = rules::charges(&f.env.ctx).unwrap();
    for (year, month, day, ends, value) in [
        (2026, 12, 1, "2026-11-30", "7"),
        (2027, 1, 1, "2026-12-31", "8"),
        (2028, 3, 1, "2028-02-29", "9"),
    ] {
        set_today(&f.env, year, month, day);
        let input = ChargesInput {
            seller_commission: DefaultChargeInput {
                version: c.seller_commission.version,
                method: ChargeMethod::Percent,
                value: "6".into(),
            },
            buyer_commission: DefaultChargeInput {
                version: c.buyer_commission.version,
                method: ChargeMethod::Percent,
                value: value.into(),
            },
            laga_fruit: DefaultChargeInput {
                version: c.laga_fruit.version,
                method: ChargeMethod::PerUnit,
                value: "8".into(),
            },
        };
        let old = c.buyer_commission.rule_id;
        c = rules::set_charges(&f.env.ctx, &f.owner, &input).unwrap();
        let end: Option<String> = raw(&f.dir)
            .query_row(
                "SELECT to_date FROM charge_rules WHERE id = ?1",
                [old],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(end.as_deref(), Some(ends));
        assert_eq!(
            c.buyer_commission.from_date,
            format!("{year:04}-{month:02}-{day:02}")
        );
    }
}

#[test]
fn a_stale_form_never_matches_the_new_row_of_a_default() {
    // Money review: after a change the new row starts above any version the old form holds.
    let f = fixture();
    let before = rules::charges(&f.env.ctx).unwrap();
    let change =
        |c: &mandi_services::core::rules::ChargesSettings, v: i64, value: &str| ChargesInput {
            seller_commission: DefaultChargeInput {
                version: c.seller_commission.version,
                method: ChargeMethod::Percent,
                value: "6".into(),
            },
            buyer_commission: DefaultChargeInput {
                version: v,
                method: ChargeMethod::Percent,
                value: value.into(),
            },
            laga_fruit: DefaultChargeInput {
                version: c.laga_fruit.version,
                method: ChargeMethod::PerUnit,
                value: "8".into(),
            },
        };
    set_today(&f.env, 2026, 10, 9);
    let first = rules::set_charges(
        &f.env.ctx,
        &f.owner,
        &change(&before, before.buyer_commission.version, "7"),
    )
    .unwrap();
    set_today(&f.env, 2026, 10, 10);
    let second = rules::set_charges(
        &f.env.ctx,
        &f.owner,
        &change(&first, first.buyer_commission.version, "8"),
    )
    .unwrap();
    // A form that still holds the first version is refused.
    let stale = change(&first, first.buyer_commission.version, "9");
    let error = rules::set_charges(&f.env.ctx, &f.owner, &stale).unwrap_err();
    assert_eq!(
        error_code_field(error),
        (
            ErrorCode::ErrVersionChanged,
            "buyer_commission.version".to_owned()
        )
    );
    // And one that holds the very first row's version too.
    let oldest = change(&first, before.buyer_commission.version, "9");
    assert_eq!(
        error_code_field(rules::set_charges(&f.env.ctx, &f.owner, &oldest).unwrap_err()).0,
        ErrorCode::ErrVersionChanged
    );
    assert_eq!(rules::charges(&f.env.ctx).unwrap(), second);
}

fn error_code_field(error: ServiceError) -> (ErrorCode, String) {
    match error {
        ServiceError::Rejected(items) => {
            (items[0].code, items[0].field.clone().unwrap_or_default())
        }
        other => panic!("not a rejection: {other:?}"),
    }
}

#[test]
fn a_clock_moved_back_past_a_change_is_refused_with_a_clear_code() {
    // NFR-REL-11 and the money review: the default starts after "today", so a change would
    // overlap it. The answer says so; nothing is written.
    let f = fixture();
    let before = rules::charges(&f.env.ctx).unwrap();
    let input = |c: &mandi_services::core::rules::ChargesSettings, value: &str| ChargesInput {
        seller_commission: DefaultChargeInput {
            version: c.seller_commission.version,
            method: ChargeMethod::Percent,
            value: "6".into(),
        },
        buyer_commission: DefaultChargeInput {
            version: c.buyer_commission.version,
            method: ChargeMethod::Percent,
            value: value.into(),
        },
        laga_fruit: DefaultChargeInput {
            version: c.laga_fruit.version,
            method: ChargeMethod::PerUnit,
            value: "8".into(),
        },
    };
    let changed = rules::set_charges(&f.env.ctx, &f.owner, &input(&before, "7")).unwrap();
    assert_eq!(changed.buyer_commission.from_date, "2026-10-10");
    set_today(&f.env, 2026, 10, 8);
    // Settings shows the row in force on the 8th: the first one.
    let shown = rules::charges(&f.env.ctx).unwrap();
    assert_eq!(shown.buyer_commission.value_text, "6");
    let rows = count(&f.dir, "SELECT COUNT(*) FROM charge_rules");
    let audits = audit_total(&f.dir);
    let error = rules::set_charges(&f.env.ctx, &f.owner, &input(&shown, "9")).unwrap_err();
    let ServiceError::Rejected(items) = error else {
        panic!("not a rejection")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrDateInFuture, Some("buyer_commission.value"))
    );
    assert_eq!(
        items[0].values.get("today"),
        Some(&ErrorValue::Text("10-10-2026".into()))
    );
    assert_eq!(count(&f.dir, "SELECT COUNT(*) FROM charge_rules"), rows);
    assert_eq!(audit_total(&f.dir), audits);
}

#[test]
fn a_zero_percent_rule_is_a_winner_that_charges_nothing() {
    // Money review: 0% is a rule, not "no rule": the working says 12,000 x 0%.
    let f = fixture();
    save(
        &f,
        &RuleInput {
            account_id: Some(f.akram),
            ..form(
                ChargeSide::Buyer,
                ChargeType::Commission,
                ChargeMethod::Percent,
                "0",
            )
        },
    );
    let t = run(&f, f.rahim, f.akram, f.banana);
    let c = t
        .charges
        .iter()
        .find(|c| c.side == ChargeSide::Buyer && c.charge_type == ChargeType::Commission)
        .unwrap();
    assert_eq!(
        (c.winner.map(|w| w.level), c.amount_paisa),
        (Some(RuleLevel::Party), Paisa::ZERO)
    );
    assert_eq!(c.working.message_id, "working-percent");
    assert_eq!(c.working.args[1].value, WorkingValue::Percent { bps: 0 });
    assert_eq!(
        rules::get(&f.env.ctx, c.winner.and_then(|w| w.rule_id).unwrap())
            .unwrap()
            .value_text,
        "0"
    );
}

#[test]
fn e2e_13b_numbers_through_the_default_in_force_on_the_day() {
    // E2E-13b: the Owner changes the default buyer commission to 7% on 10-10-26; a changed line
    // 20 x 610 = 12,200 is charged 12,200 x 7% = 854 (the buyer pays 12,200 + 854 + 160 =
    // 13,214); a posting of the day before keeps 6% (732). The unchanged line keeping its stored
    // 1,044 is the edit path of the sale plan.
    let f = fixture();
    let before = rules::charges(&f.env.ctx).unwrap();
    rules::set_charges(
        &f.env.ctx,
        &f.owner,
        &ChargesInput {
            seller_commission: DefaultChargeInput {
                version: before.seller_commission.version,
                method: ChargeMethod::Percent,
                value: "6".into(),
            },
            buyer_commission: DefaultChargeInput {
                version: before.buyer_commission.version,
                method: ChargeMethod::Percent,
                value: "7".into(),
            },
            laga_fruit: DefaultChargeInput {
                version: before.laga_fruit.version,
                method: ChargeMethod::PerUnit,
                value: "8".into(),
            },
        },
    )
    .unwrap();
    let on = |date: &str| {
        rules::test(
            &f.env.ctx,
            &RuleTestInput {
                seller_id: f.rahim,
                buyer_id: f.akram,
                item_id: f.banana,
                date: date.into(),
                qty: "20".into(),
                rate: "610".into(),
            },
        )
        .unwrap()
    };
    let t = on("10-10-2026");
    assert_eq!(t.line_value_paisa, rupees(12_200));
    let amounts = (
        of(&t, ChargeSide::Buyer, ChargeType::Commission).1,
        of(&t, ChargeSide::Buyer, ChargeType::Laga).1,
        of(&t, ChargeSide::Seller, ChargeType::Commission).1,
    );
    assert_eq!(amounts, (854, 160, 732));
    assert_eq!(12_200 + amounts.0 + amounts.1, 13_214);
    assert_eq!(
        of(&on("09-10-2026"), ChargeSide::Buyer, ChargeType::Commission).1,
        732,
        "the day before keeps 6%"
    );
}
