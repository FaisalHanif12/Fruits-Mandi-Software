//! A figure and a percent more or less (plan 52 P52-S1, Q290 and Q291 of the live run of
//! 03-11-2026: "if this year's commission had been 10 percent higher, what would it be"). The
//! calculator's `PercentMore` and `PercentLess`: `base × (100% ± percent)`, rounded once through the
//! shared function, the working a step `base × factor%`.
//!
//! All amounts are whole rupees; the code works in paisa.
#![cfg(test)]

use mandi_domain::core::money::{Bps, Paisa, apply_bps};
use mandi_domain::figures::{CalcOp, FigureError, Op, Value, calculate};
use proptest::prelude::*;

fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

fn m(rupees: i64) -> Value {
    Value::Money(rs(rupees))
}

/// A percent in hundredths of a percent: 10.00% is 1,000.
fn pct(hundredths: i64) -> Value {
    Value::Percent(hundredths)
}

fn more(base: i64, hundredths: i64) -> Result<Value, FigureError> {
    calculate(CalcOp::PercentMore, &[m(base), pct(hundredths)]).map(|c| c.result)
}

fn less(base: i64, hundredths: i64) -> Result<Value, FigureError> {
    calculate(CalcOp::PercentLess, &[m(base), pct(hundredths)]).map(|c| c.result)
}

#[test]
fn the_books_total_and_ten_percent_more() {
    // Q290: this year's commission so far 8,360,110; 10% more is 9,196,121 exactly.
    let calc = calculate(CalcOp::PercentMore, &[m(8_360_110), pct(1_000)]).unwrap();
    assert_eq!(calc.result, m(9_196_121));
    assert_eq!(calc.dir, None, "an amount, not a difference");
    // The working is one step: base × 110.00% = the result.
    assert_eq!(calc.steps.len(), 1);
    assert_eq!(calc.steps[0].op, Op::Times);
    assert_eq!(calc.steps[0].inputs, vec![m(8_360_110), pct(11_000)]);
    assert_eq!(calc.steps[0].result, m(9_196_121));
}

#[test]
fn the_rounding_is_half_up_to_a_whole_rupee_once() {
    // 5,630 + 6% = 5,967.8, which is 5,968.
    assert_eq!(more(5_630, 600).unwrap(), m(5_968));
    // 48,333 less 50% = 24,166.5, which is 24,167 (half up).
    assert_eq!(less(48_333, 5_000).unwrap(), m(24_167));
    // 12,000 + 2.5% = 12,300 exactly; 12,000 less 2.5% = 11,700.
    assert_eq!(more(12_000, 250).unwrap(), m(12_300));
    assert_eq!(less(12_000, 250).unwrap(), m(11_700));
    // 0.01% of 12,000 is 1.2: 12,001.
    assert_eq!(more(12_000, 1).unwrap(), m(12_001));
    // 0 stays 0.
    assert_eq!(more(0, 1_000).unwrap(), m(0));
    // Up to 900% more: the base times ten.
    assert_eq!(more(1_000, 90_000).unwrap(), m(10_000));
    // 99.99% less leaves 0.01%.
    assert_eq!(less(1_000, 9_999).unwrap(), m(0));
}

#[test]
fn only_an_amount_and_a_percent_are_taken() {
    // A count, a balance or a second amount in place of the percent.
    for bad in [
        vec![Value::Count(10), pct(1_000)],
        vec![Value::Balance(rs(100)), pct(1_000)],
        vec![m(100), m(10)],
        vec![pct(1_000), m(100)],
        vec![m(100), Value::Count(10)],
    ] {
        for op in [CalcOp::PercentMore, CalcOp::PercentLess] {
            assert_eq!(calculate(op, &bad).unwrap_err(), FigureError::MixedKinds);
        }
    }
    // Exactly two inputs.
    for op in [CalcOp::PercentMore, CalcOp::PercentLess] {
        assert_eq!(
            calculate(op, &[m(100)]).unwrap_err(),
            FigureError::NeedsTwoInputs
        );
        assert_eq!(
            calculate(op, &[m(100), pct(10), pct(10)]).unwrap_err(),
            FigureError::NeedsTwoInputs
        );
        assert_eq!(calculate(op, &[]).unwrap_err(), FigureError::NoInputs);
    }
}

#[test]
fn a_loss_and_a_percent_out_of_range_are_refused() {
    // A percent more or less of a loss would be a guess.
    assert_eq!(more(-5_000, 1_000).unwrap_err(), FigureError::NegativeBase);
    assert_eq!(less(-5_000, 1_000).unwrap_err(), FigureError::NegativeBase);
    // 0, and above 900% more; 0, and 100% or more less.
    assert_eq!(more(100, 0).unwrap_err(), FigureError::PercentOutOfRange);
    assert_eq!(
        more(100, 90_001).unwrap_err(),
        FigureError::PercentOutOfRange
    );
    assert_eq!(less(100, 0).unwrap_err(), FigureError::PercentOutOfRange);
    assert_eq!(
        less(100, 10_000).unwrap_err(),
        FigureError::PercentOutOfRange
    );
    assert_eq!(
        less(100, 20_000).unwrap_err(),
        FigureError::PercentOutOfRange
    );
    // A percent that does not even fit a 32-bit number.
    assert_eq!(
        more(100, i64::MAX).unwrap_err(),
        FigureError::PercentOutOfRange
    );
    // A result above the largest allowed amount (10,00,00,00,000 rupees is allowed).
    assert_eq!(more(1_000_000_000, 90_000).unwrap(), m(10_000_000_000));
    assert_eq!(
        more(1_100_000_000, 90_000).unwrap_err(),
        FigureError::Overflow
    );
    // The refusals carry an error code.
    assert_eq!(
        FigureError::NegativeBase.code(),
        mandi_domain::core::error::ErrorCode::ErrOutOfRange
    );
    assert_eq!(
        FigureError::PercentOutOfRange.code(),
        mandi_domain::core::error::ErrorCode::ErrOutOfRange
    );
}

#[test]
fn a_percent_is_still_no_input_for_the_other_operations() {
    for op in [
        CalcOp::Sum,
        CalcOp::Difference,
        CalcOp::Average,
        CalcOp::PercentOf,
        CalcOp::ChangePercent,
    ] {
        assert_eq!(
            calculate(op, &[m(100), pct(1_000)]).unwrap_err(),
            FigureError::PercentInput
        );
    }
}

proptest! {
    /// More is the base and one rounded part of it; less is the base at the rest of 100%, rounded
    /// once (the rounding is on the result, half up, as the what-if on the pace does); 100% more is
    /// double. Rounding the removed part instead would differ by a rupee at a half.
    #[test]
    fn more_and_less_round_the_result_once(
        base in 0i64..1_000_000_000,
        percent in 1i64..10_000,
    ) {
        let bps = |n: i64| Bps::new(i32::try_from(n).unwrap()).unwrap();
        let part = apply_bps(rs(base), bps(percent)).unwrap().rupees();
        prop_assert_eq!(more(base, percent).unwrap(), m(base + part));
        let rest = apply_bps(rs(base), bps(10_000 - percent)).unwrap().rupees();
        prop_assert_eq!(less(base, percent).unwrap(), m(rest));
        prop_assert_eq!(more(base, 10_000).unwrap(), m(base * 2));
    }
}
