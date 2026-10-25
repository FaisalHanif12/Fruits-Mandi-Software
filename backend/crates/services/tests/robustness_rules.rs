//! Commission rules under attack (D2 attack list item 6; PRD FR-11.1 to FR-11.10, PRD 11.2, Code
//! Standards 4.2 and V0, NFR-SEC-5). Owner: `robustness`.
//!
//! - Every hostile rule value (decimals where whole rupees are needed, negatives, Urdu digits,
//!   exponents, signs, spaces, huge numbers, markup) is refused on the `value` field with its
//!   error code, and the whole data file stays as it was: no rule, no audit row.
//! - A percent above 100 or with more than 2 decimals, and a per-unit value that is not a whole
//!   number of rupees or is over the limit, are refused. Zero is allowed where PRD 11 allows it.
//! - Dates (impossible, reversed), notes (too long, control characters) and references (an
//!   account that does not exist, one of the wrong type, an item that does not exist) are refused.
//! - "Test a rule" refuses hostile qty, rate and date and writes nothing.
#![cfg(test)]

mod common;

use common::*;
use mandi_services::core::accounts::{AccountInput, AccountType};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::masters::{ItemInput, MasterInput};
use mandi_services::core::rules::{ChargeMethod, ChargeSide, ChargeType, RuleInput, RuleTestInput};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, auth, masters, rules};
use tempfile::TempDir;

struct World {
    dir: TempDir,
    env: TestEnv,
    owner: auth::Actor,
    seller: i64,
    buyer: i64,
    item: i64,
}

fn confirmed_words(error: &ServiceError) -> Option<Vec<String>> {
    match error {
        ServiceError::Rejected(items)
            if items
                .iter()
                .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
        {
            let mut words = Vec::new();
            for item in items {
                if let Some(ErrorValue::Text(text)) = item.values.get("words") {
                    words.extend(text.split(' ').map(str::to_owned));
                }
            }
            Some(words)
        }
        _ => None,
    }
}

fn account(env: &TestEnv, owner: &auth::Actor, kind: AccountType, ur: &str, en: &str) -> i64 {
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
            Ok(row) => return row.id,
            Err(error) => match confirmed_words(&error) {
                Some(words) => input.confirmed_rules_words.extend(words),
                None => panic!("{error:?}"),
            },
        }
    }
}

fn world() -> World {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let seller = account(&env, &owner, AccountType::Seller, "رحیم", "Rahim");
    let buyer = account(&env, &owner, AccountType::Buyer, "اکرم", "Akram");
    let mut input = ItemInput {
        id: None,
        version: None,
        code: String::new(),
        name_ur: "کیلا".into(),
        name_en: "Banana".into(),
        item_group: "fruit".into(),
        default_packing_id: None,
        sold_by: "unit".into(),
        confirmed_rules_words: Vec::new(),
    };
    let item = loop {
        match masters::save(&env.ctx, &owner, &MasterInput::Item(input.clone())) {
            Ok(row) => break row.id(),
            Err(error) => match confirmed_words(&error) {
                Some(words) => input.confirmed_rules_words.extend(words),
                None => panic!("{error:?}"),
            },
        }
    };
    World {
        dir,
        env,
        owner,
        seller,
        buyer,
        item,
    }
}

fn rule(side: ChargeSide, charge: ChargeType, method: ChargeMethod, value: &str) -> RuleInput {
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

fn percent(value: &str) -> RuleInput {
    rule(
        ChargeSide::Seller,
        ChargeType::Commission,
        ChargeMethod::Percent,
        value,
    )
}

fn per_unit(value: &str) -> RuleInput {
    rule(
        ChargeSide::Buyer,
        ChargeType::Laga,
        ChargeMethod::PerUnit,
        value,
    )
}

fn snapshot(dir: &TempDir) -> (i64, i64, i64) {
    let conn = raw(dir);
    let one = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap() };
    (
        one("SELECT COUNT(*) FROM charge_rules"),
        one("SELECT COUNT(*) FROM audit_log"),
        one("SELECT COALESCE(SUM(version), 0) FROM charge_rules"),
    )
}

fn codes(error: &ServiceError) -> Vec<(ErrorCode, Option<String>)> {
    match error {
        ServiceError::Rejected(items) => items.iter().map(|i| (i.code, i.field.clone())).collect(),
        other => panic!("not a rejection: {other:?}"),
    }
}

#[test]
fn hostile_rule_values_are_refused_on_the_value_field_and_write_nothing() {
    let w = world();
    let before = snapshot(&w.dir);
    let long = "9".repeat(5_000);
    // A percent: whole or with up to 2 decimals, 0 to 100.
    let bad_percent: Vec<(&str, String)> = vec![
        ("over 100", "100.01".into()),
        ("101", "101".into()),
        ("1000", "1000".into()),
        ("three decimals", "6.125".into()),
        ("negative", "-5".into()),
        ("negative zero", "-0".into()),
        ("plus sign", "+6".into()),
        ("exponent", "1e1".into()),
        ("hex", "0x6".into()),
        ("percent sign", "6%".into()),
        ("comma decimal", "6,5".into()),
        ("two dots", "6.5.5".into()),
        ("only a dot", ".".into()),
        ("empty", String::new()),
        ("spaces", "   ".into()),
        ("letters", "six".into()),
        ("sql", "6; DROP TABLE charge_rules".into()),
        ("script", "<b>6</b>".into()),
        ("control", "6\u{7}".into()),
        ("emoji", "😀".into()),
        ("5,000 digits", long.clone()),
    ];
    for (label, value) in &bad_percent {
        // A rule for one seller, so a good value would be saved and only the value is wrong.
        let input = RuleInput {
            account_id: Some(w.seller),
            ..percent(value)
        };
        let error = rules::save(&w.env.ctx, &w.owner, &input).unwrap_err();
        let found = codes(&error);
        assert!(
            found.iter().any(|(_, f)| f.as_deref() == Some("value")),
            "percent {label} ('{value:.20}'): the error does not point to the value field: {found:?}"
        );
        assert!(
            found
                .iter()
                .all(|(c, _)| *c != ErrorCode::ErrInternal
                    && *c != ErrorCode::ErrSpellingUnconfirmed),
            "percent {label}: {found:?}"
        );
    }
    // A per-unit value: whole rupees, zero allowed (PRD 11, zero is a winner that charges nothing).
    let bad_unit: Vec<(&str, String)> = vec![
        ("decimals", "8.50".into()),
        ("one decimal", "8.5".into()),
        ("negative", "-8".into()),
        ("exponent", "1e1".into()),
        ("percent sign", "8%".into()),
        ("empty", String::new()),
        ("letters", "eight".into()),
        ("sql", "8' OR '1'='1".into()),
        ("over the limit", "10,00,00,00,001".into()),
        ("far over the limit", "99999999999999999999".into()),
        ("5,000 digits", long),
    ];
    for (label, value) in &bad_unit {
        let input = RuleInput {
            account_id: Some(w.buyer),
            ..per_unit(value)
        };
        let error = rules::save(&w.env.ctx, &w.owner, &input).unwrap_err();
        let found = codes(&error);
        assert!(
            found.iter().any(|(_, f)| f.as_deref() == Some("value")),
            "per unit {label} ('{value:.20}'): {found:?}"
        );
        assert!(
            found.iter().all(|(c, _)| *c != ErrorCode::ErrInternal),
            "per unit {label}: {found:?}"
        );
    }
    assert_eq!(
        snapshot(&w.dir),
        before,
        "a refused rule left a row or an audit entry behind"
    );
    // A zero-width mark inside a number is not seen by the person, so he meant the digits he
    // sees: "6<mark>5" is 65 percent, exactly, or it is refused. Never a different number.
    let marked = RuleInput {
        account_id: Some(w.seller),
        ..percent("6\u{200b}5")
    };
    match rules::save(&w.env.ctx, &w.owner, &marked) {
        Ok(_) => {
            let bps: i64 = raw(&w.dir)
                .query_row(
                    "SELECT value_bps FROM charge_rules WHERE account_id = ?1",
                    [w.seller],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(bps, 6_500, "a hidden mark changed the number");
        }
        Err(error) => assert!(
            error.has_code(ErrorCode::ErrControlChars)
                || codes(&error)
                    .iter()
                    .any(|(_, f)| f.as_deref() == Some("value"))
        ),
    }
    assert_chain_ok(&w.dir);
}

#[test]
fn the_edges_that_are_allowed_are_saved_exactly() {
    let w = world();
    let rahim = Some(w.seller);
    // 0 and 100 and 2 decimals are allowed percents; Urdu digits are accepted like money (4.2).
    for (value, party) in [
        ("0", rahim),
        ("100", rahim),
        ("2.5", rahim),
        ("0.01", rahim),
    ] {
        let mut input = percent(value);
        input.account_id = party;
        // The same party twice on the same start day is a warning for the later one, not a refusal.
        let saved = rules::save(&w.env.ctx, &w.owner, &input);
        assert!(saved.is_ok(), "percent '{value}': {saved:?}");
    }
    let stored: Vec<i64> = {
        let conn = raw(&w.dir);
        let mut st = conn
            .prepare("SELECT value_bps FROM charge_rules WHERE account_id IS NOT NULL ORDER BY id")
            .unwrap();
        st.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    // Basis points, as integers: 0%, 100%, 2.5% and 0.01%.
    assert_eq!(stored, [0, 10_000, 250, 1]);
    // A per-unit value of zero and of the largest allowed amount.
    for value in ["0", "10,00,00,00,000"] {
        let saved = rules::save(&w.env.ctx, &w.owner, &per_unit(value));
        assert!(saved.is_ok(), "per unit '{value}': {saved:?}");
    }
    assert_chain_ok(&w.dir);
}

#[test]
fn dates_notes_and_references_are_checked_and_a_refused_rule_writes_nothing() {
    let w = world();
    let before = snapshot(&w.dir);
    let mut cases: Vec<(&str, RuleInput)> = Vec::new();
    for (label, from, to) in [
        ("30 February", "30-02-2026", ""),
        ("month 13", "01-13-2026", ""),
        ("day 0", "00-01-2026", ""),
        ("two digit year", "01-01-26", ""),
        ("year 0", "01-01-0000", ""),
        ("text", "tomorrow", ""),
        ("iso order", "2026-01-01", ""),
        ("empty from", "", ""),
        ("end before start", "01-02-2026", "31-01-2026"),
        ("impossible end", "01-01-2026", "31-04-2026"),
        ("sql", "01-01-2026'; --", ""),
    ] {
        cases.push((
            label,
            RuleInput {
                from_date: from.into(),
                to_date: to.into(),
                ..percent("5")
            },
        ));
    }
    cases.push((
        "note of 501 characters",
        RuleInput {
            note: "ن".repeat(501),
            ..percent("5")
        },
    ));
    cases.push((
        "note with a control character",
        RuleInput {
            note: "a\u{1}b".into(),
            ..percent("5")
        },
    ));
    cases.push((
        "an account that does not exist",
        RuleInput {
            account_id: Some(9_999_999),
            ..percent("5")
        },
    ));
    cases.push((
        "a negative account id",
        RuleInput {
            account_id: Some(-1),
            ..percent("5")
        },
    ));
    cases.push((
        "a buyer on the seller side",
        RuleInput {
            account_id: Some(w.buyer),
            ..percent("5")
        },
    ));
    cases.push((
        "a seller on the buyer side",
        RuleInput {
            account_id: Some(w.seller),
            ..rule(
                ChargeSide::Buyer,
                ChargeType::Commission,
                ChargeMethod::Percent,
                "5",
            )
        },
    ));
    cases.push((
        "an item that does not exist",
        RuleInput {
            item_id: Some(9_999_999),
            ..percent("5")
        },
    ));
    cases.push((
        "a charge the side does not have (buyer pays labour)",
        rule(
            ChargeSide::Buyer,
            ChargeType::Labour,
            ChargeMethod::PerUnit,
            "5",
        ),
    ));
    cases.push((
        "per lot for the buyer commission",
        rule(
            ChargeSide::Buyer,
            ChargeType::Commission,
            ChargeMethod::PerLot,
            "5",
        ),
    ));
    cases.push((
        "percent labour for the seller",
        rule(
            ChargeSide::Seller,
            ChargeType::Labour,
            ChargeMethod::Percent,
            "5",
        ),
    ));
    cases.push((
        "an item and an item group together",
        RuleInput {
            item_id: Some(w.item),
            item_group: Some(mandi_services::core::rules::ItemGroup::Fruit),
            ..percent("5")
        },
    ));
    cases.push((
        "an edit of a row that does not exist",
        RuleInput {
            id: Some(9_999_999),
            version: Some(1),
            ..percent("5")
        },
    ));
    for (label, input) in &cases {
        match rules::save(&w.env.ctx, &w.owner, input) {
            Ok(saved) => panic!("{label}: a bad rule was saved: {saved:?}"),
            Err(error) => {
                let found = codes(&error);
                assert!(
                    found.iter().all(|(c, _)| *c != ErrorCode::ErrInternal),
                    "{label}: {found:?}"
                );
                assert!(
                    found.iter().all(|(_, f)| f.is_some()),
                    "{label}: an error without a field: {found:?}"
                );
            }
        }
    }
    assert_eq!(snapshot(&w.dir), before, "a refused rule wrote something");
    assert_chain_ok(&w.dir);
}

#[test]
fn test_a_rule_refuses_hostile_input_and_writes_nothing() {
    let w = world();
    let before = snapshot(&w.dir);
    let run = |seller: i64, buyer: i64, item: i64, date: &str, qty: &str, rate: &str| {
        rules::test(
            &w.env.ctx,
            &RuleTestInput {
                seller_id: seller,
                buyer_id: buyer,
                item_id: item,
                date: date.into(),
                qty: qty.into(),
                rate: rate.into(),
            },
        )
    };
    let (s, b, i) = (w.seller, w.buyer, w.item);
    // The worked check of PRD 11.3 works, and so does a sum in the quantity (FR-7.48).
    assert!(run(s, b, i, "10-10-2026", "20", "600").is_ok());
    assert!(run(s, b, i, "10-10-2026", "20+25", "600").is_ok());
    for (label, seller, buyer, item, date, qty, rate) in [
        ("qty 0", s, b, i, "10-10-2026", "0", "600"),
        ("qty negative", s, b, i, "10-10-2026", "-5", "600"),
        ("qty decimal", s, b, i, "10-10-2026", "2.5", "600"),
        (
            "qty over a million",
            s,
            b,
            i,
            "10-10-2026",
            "1,000,001",
            "600",
        ),
        (
            "qty a sum below zero",
            s,
            b,
            i,
            "10-10-2026",
            "10-20",
            "600",
        ),
        ("qty times", s, b, i, "10-10-2026", "2*3", "600"),
        ("qty exponent", s, b, i, "10-10-2026", "1e3", "600"),
        ("qty empty", s, b, i, "10-10-2026", "", "600"),
        ("rate 0", s, b, i, "10-10-2026", "20", "0"),
        ("rate decimal", s, b, i, "10-10-2026", "20", "850.50"),
        ("rate negative", s, b, i, "10-10-2026", "20", "-600"),
        (
            "rate over the limit",
            s,
            b,
            i,
            "10-10-2026",
            "20",
            "10,00,00,00,001",
        ),
        ("rate letters", s, b, i, "10-10-2026", "20", "abc"),
        ("rate sql", s, b, i, "10-10-2026", "20", "1' OR '1'='1"),
        ("date impossible", s, b, i, "31-02-2026", "20", "600"),
        ("date text", s, b, i, "today", "20", "600"),
        ("date empty", s, b, i, "", "20", "600"),
        (
            "a seller that does not exist",
            9_999_999,
            b,
            i,
            "10-10-2026",
            "20",
            "600",
        ),
        (
            "a buyer in the seller place",
            b,
            b,
            i,
            "10-10-2026",
            "20",
            "600",
        ),
        (
            "an item that does not exist",
            s,
            b,
            9_999_999,
            "10-10-2026",
            "20",
            "600",
        ),
    ] {
        match run(seller, buyer, item, date, qty, rate) {
            Ok(result) => panic!("{label}: accepted: {result:?}"),
            Err(error) => assert!(
                !error.has_code(ErrorCode::ErrInternal),
                "{label}: an internal error: {error:?}"
            ),
        }
    }
    assert_eq!(snapshot(&w.dir), before);
}
