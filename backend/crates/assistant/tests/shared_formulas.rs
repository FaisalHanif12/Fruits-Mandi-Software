//! One formula, proven equal in its two homes (plan 48 T2, Code Standards G6, money agent 9.1).
//!
//! The "% of all" of the analytics views (`mandi_reports::analytics::share_hundredths`) and the
//! percent of the question panel's calculator (`mandi_services::core::money::hundredths`, used by
//! `domain::figures`) are the same formula: `(part × 10,000 + total ÷ 2) ÷ total`, in hundredths
//! of a percent. Domain cannot reach the reports crate, so the formula lives in `core` and this
//! test keeps the reports copy equal to it for every part from 0 to the total.
#![cfg(test)]

use mandi_reports::analytics::share_hundredths;
use mandi_services::core::money::{Paisa, hundredths};
use mandi_services::domain::figures::{CalcOp, Value, calculate};
use proptest::prelude::*;

fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

#[test]
fn the_worked_numbers_are_the_same_in_both() {
    // PRD 18.4: 3,928 of 5,448 is 72.10%.
    let (part, total) = (rs(3_928), rs(5_448));
    assert_eq!(share_hundredths(part, total).unwrap(), Some(7_210));
    assert_eq!(
        hundredths(i128::from(part.as_paisa()), i128::from(total.as_paisa())),
        Ok(7_210)
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn the_calculator_percent_equals_the_analytics_share(total in 1i64..10_000_000_000, per_mille in 0i64..=1_000) {
        let part = total / 1_000 * per_mille + total % 1_000 % (per_mille + 1);
        prop_assume!(part <= total);
        let (p, t) = (rs(part), rs(total));
        let analytics = share_hundredths(p, t).unwrap().unwrap();
        let core = hundredths(i128::from(p.as_paisa()), i128::from(t.as_paisa())).unwrap();
        prop_assert_eq!(analytics, core);
        let calc = calculate(CalcOp::PercentOf, &[Value::Money(p), Value::Money(t)]).unwrap();
        prop_assert_eq!(calc.result, Value::Percent(analytics));
    }
}
