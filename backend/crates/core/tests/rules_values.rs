//! Rule values and the allowed combinations of PRD 11.2 (plan 09): the percent text round trip,
//! the table of charges and methods, the value checks and the whole-form checks.
#![cfg(test)]

use mandi_core::dates::BusinessDate;
use mandi_core::error::ErrorCode;
use mandi_core::money::{Bps, Paisa, parse_bps};
use mandi_core::rules::{
    ChargeMethod, ChargeSide, ChargeType, ChargeValue, ItemGroup, RuleInput, charge_allowed,
    method_allowed, percent_text, prepare_rule, rule_value,
};
use proptest::prelude::*;

fn today() -> BusinessDate {
    BusinessDate::parse_iso("2026-10-10").unwrap()
}

#[test]
fn the_percent_text_round_trips_for_every_bps() {
    // What the form is filled with is read back to the same value: 0 to 10,000 bps.
    for bps in 0..=10_000 {
        let text = percent_text(Bps::new(bps).unwrap());
        assert_eq!(parse_bps(&text).unwrap().get(), bps, "{bps} -> {text}");
    }
}

#[test]
fn the_allowed_combinations_are_exactly_the_table_of_prd_11_2() {
    use ChargeMethod::{PerLot, PerUnit, Percent};
    use ChargeSide::{Buyer, Seller};
    use ChargeType::{Commission, Labour, Laga, Munshiana, Phone};
    // (side, charge, methods allowed)
    let table: [(ChargeSide, ChargeType, &[ChargeMethod]); 6] = [
        (Seller, Commission, &[Percent, PerUnit, PerLot]),
        (Seller, Labour, &[PerUnit, PerLot]),
        (Seller, Munshiana, &[PerUnit, PerLot]),
        (Seller, Phone, &[PerUnit, PerLot]),
        (Buyer, Commission, &[Percent, PerUnit]),
        (Buyer, Laga, &[Percent, PerUnit]),
    ];
    for side in [Seller, Buyer] {
        for charge in ChargeType::ALL {
            let row = table.iter().find(|r| r.0 == side && r.1 == charge);
            assert_eq!(
                charge_allowed(side, charge),
                row.is_some(),
                "{side:?} {charge:?}"
            );
            for method in [Percent, PerUnit, PerLot] {
                let want = row.is_some_and(|r| r.2.contains(&method));
                assert_eq!(
                    method_allowed(side, charge, method),
                    want,
                    "{side:?} {charge:?} {method:?}"
                );
            }
        }
    }
}

#[test]
fn rule_values_follow_the_method() {
    let value = |m, t| rule_value("value", m, t);
    // Percent: 0 to 100 with up to 2 decimals.
    assert_eq!(
        value(ChargeMethod::Percent, "6").unwrap(),
        ChargeValue::Percent(Bps::new(600).unwrap())
    );
    assert_eq!(
        value(ChargeMethod::Percent, "0").unwrap(),
        ChargeValue::Percent(Bps::new(0).unwrap())
    );
    assert_eq!(
        value(ChargeMethod::Percent, "100").unwrap(),
        ChargeValue::Percent(Bps::FULL)
    );
    assert_eq!(
        value(ChargeMethod::Percent, "100.01").unwrap_err().code,
        ErrorCode::ErrPercentOver100
    );
    assert_eq!(
        value(ChargeMethod::Percent, "2.255").unwrap_err().code,
        ErrorCode::ErrTooManyDecimals
    );
    assert_eq!(
        value(ChargeMethod::Percent, "-1").unwrap_err().code,
        ErrorCode::ErrNegativeNotAllowed
    );
    // Per نگ: whole rupees, more than 0. Per lot: whole rupees, 0 allowed.
    assert_eq!(
        value(ChargeMethod::PerUnit, "8").unwrap(),
        ChargeValue::PerUnit(Paisa::from_rupees(8).unwrap())
    );
    // Zero per نگ is allowed: it is how the Owner says "no لاگا for this buyer".
    assert_eq!(
        value(ChargeMethod::PerUnit, "0").unwrap(),
        ChargeValue::PerUnit(Paisa::ZERO)
    );
    assert_eq!(
        value(ChargeMethod::PerUnit, "-1").unwrap_err().code,
        ErrorCode::ErrNegativeNotAllowed
    );
    assert_eq!(
        value(ChargeMethod::PerUnit, "8.5").unwrap_err().code,
        ErrorCode::ErrDecimalsNotAllowed
    );
    assert_eq!(
        value(ChargeMethod::PerLot, "0").unwrap(),
        ChargeValue::PerLot(Paisa::ZERO)
    );
    assert_eq!(
        value(ChargeMethod::PerLot, "1,000").unwrap(),
        ChargeValue::PerLot(Paisa::from_rupees(1_000).unwrap())
    );
    // The value and its columns agree.
    let v = value(ChargeMethod::Percent, "2.5").unwrap();
    assert_eq!(v.columns(), (Some(250), None));
    assert_eq!(
        ChargeValue::from_columns(ChargeMethod::Percent, Some(250), None),
        Some(v)
    );
    assert_eq!(
        ChargeValue::from_columns(ChargeMethod::Percent, Some(250), Some(100)),
        None
    );
    assert_eq!(
        ChargeValue::from_columns(ChargeMethod::PerUnit, Some(250), None),
        None
    );
}

fn form() -> RuleInput {
    RuleInput {
        id: None,
        version: None,
        side: ChargeSide::Seller,
        charge_type: ChargeType::Commission,
        account_id: None,
        item_id: None,
        item_group: None,
        method: ChargeMethod::Percent,
        value: "6".into(),
        from_date: "01-10-2026".into(),
        to_date: String::new(),
        is_active: true,
        note: String::new(),
    }
}

#[test]
fn a_good_form_passes_and_the_dates_are_read() {
    let ok = prepare_rule(&form(), today()).unwrap();
    assert_eq!(ok.from_date.to_iso(), "2026-10-01");
    assert_eq!(ok.to_date, None);
    assert_eq!(ok.note, None);
    // The last day may equal the first day, and the note is trimmed.
    let one_day = RuleInput {
        to_date: "01-10-2026".into(),
        note: "  آج صرف  ".into(),
        ..form()
    };
    let ok = prepare_rule(&one_day, today()).unwrap();
    assert_eq!(ok.to_date, ok.from_date.into());
    assert_eq!(ok.note.as_deref(), Some("آج صرف"));
    // An edit needs the version it read.
    let edit = RuleInput {
        id: Some(3),
        version: None,
        ..form()
    };
    let errors = prepare_rule(&edit, today()).unwrap_err();
    assert_eq!(
        (errors[0].code, errors[0].field.as_str()),
        (ErrorCode::ErrRequired, "version")
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Whatever is typed, checking a form never panics, and a form that passes has a value that
    /// fits its method and a group only on a rule with no party and no item.
    #[test]
    fn prepare_rule_never_panics_and_passed_forms_are_sound(
        value in "[0-9., ,+-]{0,10}",
        method in 0_u8..3,
        charge in 0_usize..5,
        buyer in any::<bool>(),
        group in any::<bool>(),
        party in any::<bool>(),
        item in any::<bool>(),
        from in "[0-9-]{0,10}",
        to in "[0-9-]{0,10}",
    ) {
        let input = RuleInput {
            side: if buyer { ChargeSide::Buyer } else { ChargeSide::Seller },
            charge_type: ChargeType::ALL[charge],
            method: [ChargeMethod::Percent, ChargeMethod::PerUnit, ChargeMethod::PerLot][usize::from(method)],
            value,
            item_group: group.then_some(ItemGroup::Fruit),
            account_id: party.then_some(1),
            item_id: item.then_some(2),
            from_date: from,
            to_date: to,
            ..form()
        };
        if let Ok(ok) = prepare_rule(&input, today()) {
            prop_assert!(method_allowed(ok.side, ok.charge, ok.value.method()));
            prop_assert_eq!(ok.value.method(), input.method);
            if ok.item_group.is_some() {
                prop_assert!(ok.account_id.is_none() && ok.item_id.is_none());
            }
            if let Some(to) = ok.to_date {
                prop_assert!(to >= ok.from_date);
            }
            match ok.value {
                ChargeValue::Percent(bps) => prop_assert!(bps.get() <= 10_000),
                ChargeValue::PerUnit(p) => prop_assert!(!p.is_negative() && p.as_paisa() % 100 == 0),
                ChargeValue::PerLot(p) => prop_assert!(!p.is_negative() && p.as_paisa() % 100 == 0),
            }
        }
    }
}
