//! The services wrappers of the calculator and the estimator (plan 48 T2): they give the domain's
//! numbers, and turn a refusal into a rejection with the right code. The numbers themselves are
//! tested in `crates/domain/tests/figures_forecast.rs`.
#![cfg(test)]

use mandi_services::core::error::ErrorCode;
use mandi_services::core::money::Paisa;
use mandi_services::domain::figures::{CalcOp, NumberUnit, Value};
use mandi_services::domain::forecast::{MonthValue, YearMonth};
use mandi_services::forecast::{Forecast, estimate};
use mandi_services::{ServiceError, figures};

fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

fn history(values: &[i64]) -> Vec<MonthValue> {
    let mut month = YearMonth::new(2026, 4).unwrap();
    let mut out = Vec::new();
    for v in values {
        out.push(MonthValue {
            month,
            value: rs(*v),
        });
        month = month.plus_months(1).unwrap();
    }
    out
}

#[test]
fn the_calculator_gives_e2e_61_and_codes_its_refusals() {
    let dr = |r| Value::Balance(rs(r));
    let sum = figures::calculate(CalcOp::Sum, &[dr(2_880), dr(18_684)]).unwrap();
    assert_eq!(sum.result, dr(21_564));
    let no_base = figures::calculate(CalcOp::PercentOf, &[dr(1), dr(0)]).unwrap_err();
    assert!(no_base.has_code(ErrorCode::ErrNoBase));
    let mixed = figures::calculate(CalcOp::Sum, &[dr(1), Value::Count(1)]).unwrap_err();
    assert!(mixed.has_code(ErrorCode::ErrOutOfRange));
    let numbers = figures::question_numbers("agle 6 mahine");
    assert_eq!(
        (numbers[0].number, numbers[0].unit),
        (6, NumberUnit::Months)
    );
}

#[test]
fn the_estimator_gives_e2e_59_and_says_when_the_history_is_too_short() {
    let now = YearMonth::new(2026, 10).unwrap();
    let ok = estimate(
        &history(&[36_000, 39_000, 42_000, 45_000, 48_000, 51_000]),
        now,
        6,
        None,
    );
    let Forecast::Estimate(found) = ok.unwrap() else {
        panic!("expected an estimate");
    };
    assert_eq!(found.methods[0].total, rs(288_000));
    assert_eq!(found.methods[1].total, rs(369_000));
    // Two full months (August and September): a result, not an error.
    let short = YearMonth::new(2026, 8).unwrap();
    let h = vec![
        MonthValue {
            month: short,
            value: rs(5_000),
        },
        MonthValue {
            month: short.plus_months(1).unwrap(),
            value: rs(6_000),
        },
    ];
    assert_eq!(
        estimate(&h, now, 6, None).unwrap(),
        Forecast::TooShort { full_months: 2 }
    );
    // A bad request is a rejection.
    let error: ServiceError = estimate(&h, now, 13, None).unwrap_err();
    assert!(error.has_code(ErrorCode::ErrOutOfRange));
}
