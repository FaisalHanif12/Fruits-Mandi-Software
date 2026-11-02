//! Tests of the question panel's calculator and estimator (plan 48 T2, PRD FR-7.75, FR-7.86,
//! E2E-59, E2E-61, Code Standards M14, money agent Section 9.1): the PRD worked numbers as unit
//! tests, `question_numbers`, and property tests.
//!
//! All amounts below are whole rupees; the code works in paisa.
#![cfg(test)]

use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Paisa;
use mandi_domain::figures::{
    CalcOp, Dir, FigureError, MAX_INPUTS, MAX_QUESTION_NUMBERS, NumberUnit, Op, Value, calculate,
    question_numbers,
};
use mandi_domain::forecast::{
    Estimate, ForecastError, MAX_FACTOR_BPS, Method, MethodEstimate, MonthValue, PaceFactor,
    YearMonth, estimate, estimate_to_year_end, estimate_with, months_through, months_to_year_end,
    year_so_far,
};
use proptest::prelude::*;

fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

fn m(rupees: i64) -> Value {
    Value::Money(rs(rupees))
}

/// A party's balance: بنام positive, جمع negative.
fn b(rupees: i64) -> Value {
    Value::Balance(rs(rupees))
}

fn ym(year: i32, month: u32) -> YearMonth {
    YearMonth::new(year, month).unwrap()
}

/// Full months from `(year, month)` on, oldest first, with the given totals in rupees.
fn history(year: i32, month: u32, rupees: &[i64]) -> Vec<MonthValue> {
    let mut at = ym(year, month);
    let mut out = Vec::new();
    for value in rupees {
        out.push(MonthValue {
            month: at,
            value: rs(*value),
        });
        at = at.plus_months(1).unwrap();
    }
    out
}

/// The E2E-59 data: April to September 2026.
fn e2e_59() -> Vec<MonthValue> {
    history(2026, 4, &[36_000, 39_000, 42_000, 45_000, 48_000, 51_000])
}

fn method(estimate: &Estimate, which: Method) -> &MethodEstimate {
    estimate
        .methods
        .iter()
        .find(|e| e.method == which)
        .unwrap_or_else(|| panic!("{which:?} was not worked out"))
}

fn months_in_rupees(e: &MethodEstimate) -> Vec<i64> {
    e.months.iter().map(|p| p.rupees()).collect()
}

// ---- E2E-59: estimates --------------------------------------------------------------------------

#[test]
fn e2e_59_same_pace_and_same_growth_over_six_months() {
    let e = estimate(&e2e_59(), ym(2026, 10), 6, None).unwrap();
    assert_eq!((e.first, e.last), (ym(2026, 10), ym(2027, 3)));
    assert_eq!((e.ahead, e.full_months), (6, 6));
    // Same season needs 12 full months; there are 6, so it is not worked out.
    let which: Vec<Method> = e.methods.iter().map(|x| x.method).collect();
    assert_eq!(which, vec![Method::SamePace, Method::SameGrowth]);

    let pace = method(&e, Method::SamePace);
    assert_eq!(pace.per_month, Some(rs(48_000)));
    assert_eq!(pace.total, rs(288_000));
    assert_eq!(months_in_rupees(pace), vec![48_000; 6]);
    assert_eq!(
        (pace.months_used, pace.used_from, pace.used_to),
        (3, ym(2026, 7), ym(2026, 9))
    );

    let growth = method(&e, Method::SameGrowth);
    assert_eq!(growth.step, Some((rs(3_000), Dir::Up)));
    assert_eq!(
        months_in_rupees(growth),
        vec![54_000, 57_000, 60_000, 63_000, 66_000, 69_000]
    );
    assert_eq!(growth.total, rs(369_000));
    assert_eq!(
        (growth.months_used, growth.used_from, growth.used_to),
        (6, ym(2026, 4), ym(2026, 9))
    );
    assert_eq!((pace.target_month, growth.target_month), (None, None));
}

#[test]
fn e2e_59_target_of_three_lakh_is_april_2027_by_pace_and_february_2027_by_growth() {
    let target = Some(rs(300_000));
    let e = estimate(&e2e_59(), ym(2026, 10), 12, target).unwrap();
    let pace = method(&e, Method::SamePace);
    // 6 x 48,000 = 288,000 is short; 7 x 48,000 = 336,000 reaches it: the 7th month, April 2027.
    assert_eq!(pace.target_month, Some(ym(2027, 4)));
    assert_eq!(pace.total, rs(576_000));
    let growth = method(&e, Method::SameGrowth);
    // 54,000 + 57,000 + 60,000 + 63,000 + 66,000 = 300,000 exactly: the 5th month, February 2027.
    assert_eq!(growth.target_month, Some(ym(2027, 2)));
    // 12 x 54,000 + 3,000 x (0 + 1 + ... + 11) = 648,000 + 198,000.
    assert_eq!(growth.total, rs(846_000));
    assert_eq!(e.last, ym(2027, 9));
}

#[test]
fn a_target_not_reached_in_the_months_ahead_has_no_target_month() {
    // 6 months ahead only: 288,000 and 369,000; a target of 400,000 is not reached either way.
    let e = estimate(&e2e_59(), ym(2026, 10), 6, Some(rs(400_000))).unwrap();
    assert!(e.methods.iter().all(|x| x.target_month.is_none()));
    // 369,000 reaches 369,000 exactly (at least the target) in the 6th month, March 2027.
    let e = estimate(&e2e_59(), ym(2026, 10), 6, Some(rs(369_000))).unwrap();
    assert_eq!(
        method(&e, Method::SameGrowth).target_month,
        Some(ym(2027, 3))
    );
    assert_eq!(method(&e, Method::SamePace).target_month, None);
}

#[test]
fn the_per_month_figure_is_rounded_once_and_the_total_is_built_from_it() {
    // 43,000 + 47,000 + 49,000 = 139,000 over 3 = 46,333.33, so 46,333 a month, and 6 months is
    // 277,998 (not 278,000): the shown figures always agree.
    let h = history(2026, 7, &[10_000, 43_000, 47_000, 49_000]);
    let e = estimate(&h, ym(2026, 11), 6, None).unwrap();
    let pace = method(&e, Method::SamePace);
    assert_eq!(pace.per_month, Some(rs(46_333)));
    assert_eq!(pace.total, rs(277_998));
}

#[test]
fn same_season_is_the_same_months_of_last_year() {
    // October 2025 to September 2026: 10,000 to 21,000.
    let values: Vec<i64> = (10..=21).map(|v| v * 1_000).collect();
    let h = history(2025, 10, &values);
    let e = estimate(&h, ym(2026, 10), 6, None).unwrap();
    let season = method(&e, Method::SameSeason);
    assert_eq!(
        months_in_rupees(season),
        vec![10_000, 11_000, 12_000, 13_000, 14_000, 15_000]
    );
    assert_eq!(season.total, rs(75_000));
    // Month 1 ahead (October 2026) is October 2025; the months it used are the first six.
    assert_eq!(
        (season.used_from, season.used_to),
        (ym(2025, 10), ym(2026, 3))
    );
    // All three methods are there, in order.
    let which: Vec<Method> = e.methods.iter().map(|x| x.method).collect();
    assert_eq!(
        which,
        vec![Method::SamePace, Method::SameGrowth, Method::SameSeason]
    );
    // 12 months ahead uses the whole year.
    let e = estimate(&h, ym(2026, 10), 12, None).unwrap();
    assert_eq!(method(&e, Method::SameSeason).total, rs(186_000));
}

#[test]
fn a_falling_trend_never_goes_below_zero() {
    let h = history(2026, 4, &[100_000, 90_000, 80_000, 70_000, 60_000, 50_000]);
    let e = estimate(&h, ym(2026, 10), 6, None).unwrap();
    let growth = method(&e, Method::SameGrowth);
    // change = 50,000 / 5 = 10,000 down; 40,000, 30,000, 20,000, 10,000, then 0 and 0.
    assert_eq!(growth.step, Some((rs(10_000), Dir::Down)));
    assert_eq!(
        months_in_rupees(growth),
        vec![40_000, 30_000, 20_000, 10_000, 0, 0]
    );
    assert_eq!(growth.total, rs(100_000));
}

#[test]
fn too_little_history_gives_fewer_methods_or_none() {
    for months in 0..3usize {
        let h = history(2026, 8, &vec![1_000; months]);
        let current = ym(2026, 8)
            .plus_months(u32::try_from(months).unwrap())
            .unwrap();
        let error = estimate(&h, current, 6, None).unwrap_err();
        assert_eq!(
            error,
            ForecastError::TooLittleHistory {
                months: u32::try_from(months).unwrap()
            }
        );
        assert_eq!(error.code(), ErrorCode::ErrTooLittleHistory);
    }
    for (months, methods) in [(3, 1), (5, 1), (6, 2), (11, 2), (12, 3), (20, 3)] {
        let h = history(2025, 1, &vec![1_000; months]);
        let current = ym(2025, 1)
            .plus_months(u32::try_from(months).unwrap())
            .unwrap();
        let e = estimate(&h, current, 6, None).unwrap();
        assert_eq!(e.methods.len(), methods, "{months} months");
        assert_eq!(e.full_months, u32::try_from(months).unwrap());
    }
}

#[test]
fn bad_inputs_are_refused() {
    let h = e2e_59();
    let now = ym(2026, 10);
    assert_eq!(
        estimate(&h, now, 0, None).unwrap_err(),
        ForecastError::AheadOutOfRange
    );
    assert_eq!(
        estimate(&h, now, 13, None).unwrap_err(),
        ForecastError::AheadOutOfRange
    );
    assert_eq!(
        estimate(&h, now, 6, Some(Paisa::ZERO)).unwrap_err(),
        ForecastError::TargetNotPositive
    );
    assert_eq!(
        estimate(&h, now, 6, Some(rs(-5))).unwrap_err(),
        ForecastError::TargetNotPositive
    );
    // The history must end the month before the current one, with no gap.
    assert_eq!(
        estimate(&h, ym(2026, 11), 6, None).unwrap_err(),
        ForecastError::HistoryNotConsecutive
    );
    assert_eq!(
        estimate(&h, ym(2026, 9), 6, None).unwrap_err(),
        ForecastError::HistoryNotConsecutive
    );
    let mut gap = h.clone();
    gap.remove(2);
    assert_eq!(
        estimate(&gap, now, 6, None).unwrap_err(),
        ForecastError::HistoryNotConsecutive
    );
}

#[test]
fn year_months_roll_over_the_year_end() {
    assert_eq!(ym(2026, 10).plus_months(6), Some(ym(2027, 4)));
    assert_eq!(ym(2026, 12).plus_months(1), Some(ym(2027, 1)));
    assert_eq!(ym(2026, 1).plus_months(0), Some(ym(2026, 1)));
    assert_eq!(ym(2026, 5).plus_months(24), Some(ym(2028, 5)));
    assert_eq!(YearMonth::new(2026, 13), None);
    assert_eq!(YearMonth::new(2026, 0), None);
    assert_eq!(ym(9999, 12).plus_months(1), None);
}

#[test]
fn the_working_is_built_by_the_same_code() {
    let e = estimate(&e2e_59(), ym(2026, 10), 6, None).unwrap();
    // Same pace: the average of the last 3, then per month times the months.
    let pace = method(&e, Method::SamePace);
    assert_eq!(pace.steps.len(), 2);
    assert_eq!(pace.steps[0].op, Op::Average);
    assert_eq!(pace.steps[0].inputs, vec![m(45_000), m(48_000), m(51_000)]);
    assert_eq!(pace.steps[0].result, m(48_000));
    assert_eq!(pace.steps[1].op, Op::Times);
    assert_eq!(pace.steps[1].inputs, vec![m(48_000), Value::Count(6)]);
    assert_eq!(pace.steps[1].result, m(288_000));
    // Same growth: last minus first, divided by 5, then the sum of the months.
    let growth = method(&e, Method::SameGrowth);
    let ops: Vec<Op> = growth.steps.iter().map(|s| s.op).collect();
    assert_eq!(ops, vec![Op::Difference, Op::DividedBy, Op::Sum]);
    assert_eq!(growth.steps[0].inputs, vec![m(51_000), m(36_000)]);
    assert_eq!(growth.steps[0].result, m(15_000));
    assert_eq!(growth.steps[1].inputs, vec![m(15_000), Value::Count(5)]);
    assert_eq!(growth.steps[1].result, m(3_000));
    assert_eq!(growth.steps[2].result, m(369_000));
}

// ---- E2E-61: the calculator ---------------------------------------------------------------------

#[test]
fn e2e_61_the_sum_of_two_balances() {
    let c = calculate(CalcOp::Sum, &[b(2_880), b(18_684)]).unwrap();
    assert_eq!(c.result, b(21_564));
    assert_eq!(
        c.result,
        Value::Balance(Paisa::try_from_paisa(2_156_400).unwrap())
    );
    assert_eq!(c.dir, None);
    assert_eq!(c.steps.len(), 1);
    assert_eq!(c.steps[0].op, Op::Sum);
    assert_eq!(c.steps[0].inputs, vec![b(2_880), b(18_684)]);
    // The same numbers as plain amounts add up the same way.
    assert_eq!(
        calculate(CalcOp::Sum, &[m(2_880), m(18_684)])
            .unwrap()
            .result,
        m(21_564)
    );
}

#[test]
fn signed_balances_net_to_the_side_of_the_sign() {
    // 2,880 Dr and 10,160 Cr: 288,000 + (-1,016,000) = -728,000 paisa = 7,280 Cr.
    let c = calculate(CalcOp::Sum, &[b(2_880), b(-10_160)]).unwrap();
    assert_eq!(
        c.result,
        Value::Balance(Paisa::try_from_paisa(-728_000).unwrap())
    );
    // Two balances that cancel are a zero balance: no side to show.
    assert_eq!(
        calculate(CalcOp::Sum, &[b(500), b(-500)]).unwrap().result,
        b(0)
    );
}

#[test]
fn a_difference_of_two_credit_balances_is_more_not_less() {
    // Money agent F1: 10,160 Cr less 7,280 Cr is 2,880 MORE (the first is owed more), even though
    // both signs are below 0 and the signed difference (-288,000) would say "less".
    let c = calculate(CalcOp::Difference, &[b(-10_160), b(-7_280)]).unwrap();
    assert_eq!((c.result, c.dir), (m(2_880), Some(Dir::Up)));
    let c = calculate(CalcOp::Difference, &[b(-7_280), b(-10_160)]).unwrap();
    assert_eq!((c.result, c.dir), (m(-2_880), Some(Dir::Down)));
    // Debit balances: 18,684 Dr less 2,880 Dr is 15,804 more.
    let c = calculate(CalcOp::Difference, &[b(18_684), b(2_880)]).unwrap();
    assert_eq!((c.result, c.dir), (m(15_804), Some(Dir::Up)));
    // A zero balance has no side; it can be compared with either.
    let c = calculate(CalcOp::Difference, &[b(0), b(-100)]).unwrap();
    assert_eq!((c.result, c.dir), (m(-100), Some(Dir::Down)));
    // Opposite sides are refused (use a sum to net them), and so are a balance and an amount.
    assert_eq!(
        calculate(CalcOp::Difference, &[b(2_880), b(-1_000)]).unwrap_err(),
        FigureError::MixedSigns
    );
    assert_eq!(
        calculate(CalcOp::Difference, &[b(2_880), m(1_000)]).unwrap_err(),
        FigureError::MixedKinds
    );
}

#[test]
fn percent_and_change_of_balances_work_on_their_sizes() {
    // 2,880 Dr of 21,564 Dr is 13.36%; 2,880 Cr of 21,564 Cr is the same.
    let c = calculate(CalcOp::PercentOf, &[b(2_880), b(21_564)]).unwrap();
    assert_eq!(c.result, Value::Percent(1_336));
    let c = calculate(CalcOp::PercentOf, &[b(-2_880), b(-21_564)]).unwrap();
    assert_eq!(c.result, Value::Percent(1_336));
    // A Cr balance that grew from 10,000 to 12,000 rose by 20%: up, not down.
    let c = calculate(CalcOp::ChangePercent, &[b(-10_000), b(-12_000)]).unwrap();
    assert_eq!((c.result, c.dir), (Value::Percent(2_000), Some(Dir::Up)));
    let c = calculate(CalcOp::ChangePercent, &[b(10_000), b(8_000)]).unwrap();
    assert_eq!((c.result, c.dir), (Value::Percent(2_000), Some(Dir::Down)));
    assert_eq!(
        calculate(CalcOp::ChangePercent, &[b(10_000), b(-8_000)]).unwrap_err(),
        FigureError::MixedSigns
    );
    assert_eq!(
        calculate(CalcOp::PercentOf, &[b(10_000), b(-8_000)]).unwrap_err(),
        FigureError::MixedSigns
    );
    // Amounts are signed: a profit that went from -5,000 to 3,000 rose by 8,000, 160% of 5,000.
    let c = calculate(CalcOp::ChangePercent, &[m(-5_000), m(3_000)]).unwrap();
    assert_eq!((c.result, c.dir), (Value::Percent(16_000), Some(Dir::Up)));
}

#[test]
fn averages_round_half_up_on_the_magnitude() {
    assert_eq!(
        calculate(CalcOp::Average, &[m(29_400)]).unwrap().result,
        m(29_400)
    );
    assert_eq!(
        calculate(CalcOp::Average, &[m(2_880), m(18_684)])
            .unwrap()
            .result,
        m(10_782)
    );
    // 1.5 rupees rounds to 2, and -1.5 to -2.
    assert_eq!(
        calculate(CalcOp::Average, &[m(1), m(2)]).unwrap().result,
        m(2)
    );
    assert_eq!(
        calculate(CalcOp::Average, &[m(-1), m(-2)]).unwrap().result,
        m(-2)
    );
    // 139,000 / 3 = 46,333.33 -> 46,333.
    assert_eq!(
        calculate(CalcOp::Average, &[m(43_000), m(47_000), m(49_000)])
            .unwrap()
            .result,
        m(46_333)
    );
    // Balances average like amounts, with the sign of their total.
    assert_eq!(
        calculate(CalcOp::Average, &[b(2_880), b(18_684)])
            .unwrap()
            .result,
        b(10_782)
    );
    assert_eq!(
        calculate(CalcOp::Average, &[b(-1), b(-2)]).unwrap().result,
        b(-2)
    );
    // Counts: 1.5 -> 2, 1.33 -> 1.
    let counts = |a, b| calculate(CalcOp::Average, &[Value::Count(a), Value::Count(b)]).unwrap();
    assert_eq!(counts(1, 2).result, Value::Count(2));
    assert_eq!(
        calculate(
            CalcOp::Average,
            &[Value::Count(1), Value::Count(1), Value::Count(2)]
        )
        .unwrap()
        .result,
        Value::Count(1)
    );
}

#[test]
fn differences_say_which_way() {
    let c = calculate(CalcOp::Difference, &[m(51_000), m(48_000)]).unwrap();
    assert_eq!((c.result, c.dir), (m(3_000), Some(Dir::Up)));
    let c = calculate(CalcOp::Difference, &[m(48_000), m(51_000)]).unwrap();
    assert_eq!((c.result, c.dir), (m(-3_000), Some(Dir::Down)));
    let c = calculate(CalcOp::Difference, &[m(5), m(5)]).unwrap();
    assert_eq!((c.result, c.dir), (m(0), Some(Dir::Same)));
    let c = calculate(CalcOp::Difference, &[Value::Count(2), Value::Count(5)]).unwrap();
    assert_eq!((c.result, c.dir), (Value::Count(-3), Some(Dir::Down)));
}

#[test]
fn percent_of_uses_the_percent_of_all_formula() {
    // 3,928 of 5,448 = (39,280,000 + 2,724) / 5,448 = 7,210: 72.10% (PRD 18.4).
    let c = calculate(CalcOp::PercentOf, &[m(3_928), m(5_448)]).unwrap();
    assert_eq!(c.result, Value::Percent(7_210));
    // 2,880 of 21,564: (28,800,000 + 10,782) / 21,564 = 1,336.03 -> 1,336: 13.36%.
    let c = calculate(CalcOp::PercentOf, &[m(2_880), m(21_564)]).unwrap();
    assert_eq!(c.result, Value::Percent(1_336));
    // Over 100% is fine; zero part is 0%; both Cr use the magnitudes.
    assert_eq!(
        calculate(CalcOp::PercentOf, &[m(300), m(200)])
            .unwrap()
            .result,
        Value::Percent(15_000)
    );
    assert_eq!(
        calculate(CalcOp::PercentOf, &[m(0), m(200)])
            .unwrap()
            .result,
        Value::Percent(0)
    );
    assert_eq!(
        calculate(CalcOp::PercentOf, &[m(-50), m(-200)])
            .unwrap()
            .result,
        Value::Percent(2_500)
    );
    // Counts: 1 of 3 = 33.33%.
    let c = calculate(CalcOp::PercentOf, &[Value::Count(1), Value::Count(3)]).unwrap();
    assert_eq!(c.result, Value::Percent(3_333));
    // Half up: 1 of 8 = 12.5% = 1,250 exactly; 1 of 16 = 6.25% = 625.
    assert_eq!(
        calculate(CalcOp::PercentOf, &[Value::Count(1), Value::Count(16)])
            .unwrap()
            .result,
        Value::Percent(625)
    );
}

#[test]
fn percent_of_refuses_a_zero_base_and_opposite_sides() {
    assert_eq!(
        calculate(CalcOp::PercentOf, &[m(10), m(0)]).unwrap_err(),
        FigureError::NoBase
    );
    assert_eq!(FigureError::NoBase.code(), ErrorCode::ErrNoBase);
    assert_eq!(
        calculate(CalcOp::PercentOf, &[m(10), m(-20)]).unwrap_err(),
        FigureError::MixedSigns
    );
}

#[test]
fn change_in_percent_is_up_or_down_from_the_first() {
    // 48,000 to 51,000: (3,000 x 10,000 + 24,000) / 48,000 = 625: 6.25% up.
    let c = calculate(CalcOp::ChangePercent, &[m(48_000), m(51_000)]).unwrap();
    assert_eq!((c.result, c.dir), (Value::Percent(625), Some(Dir::Up)));
    // 51,000 to 48,000: (30,000,000 + 25,500) / 51,000 = 588.7 -> 588: 5.88% down.
    let c = calculate(CalcOp::ChangePercent, &[m(51_000), m(48_000)]).unwrap();
    assert_eq!((c.result, c.dir), (Value::Percent(588), Some(Dir::Down)));
    let c = calculate(CalcOp::ChangePercent, &[m(100), m(100)]).unwrap();
    assert_eq!((c.result, c.dir), (Value::Percent(0), Some(Dir::Same)));
    // A fall to zero is 100% down; a rise from 100 to 350 is 250% up.
    let c = calculate(CalcOp::ChangePercent, &[m(100), m(0)]).unwrap();
    assert_eq!((c.result, c.dir), (Value::Percent(10_000), Some(Dir::Down)));
    let c = calculate(CalcOp::ChangePercent, &[m(100), m(350)]).unwrap();
    assert_eq!((c.result, c.dir), (Value::Percent(25_000), Some(Dir::Up)));
    assert_eq!(
        calculate(CalcOp::ChangePercent, &[m(0), m(5)]).unwrap_err(),
        FigureError::NoBase
    );
}

#[test]
fn bad_calculator_inputs_are_refused() {
    assert_eq!(
        calculate(CalcOp::Sum, &[]).unwrap_err(),
        FigureError::NoInputs
    );
    assert_eq!(
        calculate(CalcOp::Sum, &[m(1), Value::Count(1)]).unwrap_err(),
        FigureError::MixedKinds
    );
    assert_eq!(
        calculate(CalcOp::Difference, &[m(1)]).unwrap_err(),
        FigureError::NeedsTwoInputs
    );
    assert_eq!(
        calculate(CalcOp::PercentOf, &[m(1), m(2), m(3)]).unwrap_err(),
        FigureError::NeedsTwoInputs
    );
    assert_eq!(
        calculate(CalcOp::Sum, &[Value::Percent(5), Value::Percent(5)]).unwrap_err(),
        FigureError::PercentInput
    );
    let many = vec![m(1); MAX_INPUTS + 1];
    assert_eq!(
        calculate(CalcOp::Sum, &many).unwrap_err(),
        FigureError::TooManyInputs
    );
    assert_eq!(
        calculate(CalcOp::Sum, many.get(..MAX_INPUTS).unwrap())
            .unwrap()
            .result,
        m(1_000)
    );
}

#[test]
fn nothing_overflows_silently() {
    // The largest allowed amount is 10,00,00,00,000 rupees; 1,000 of them still fit.
    let largest = m(1_000_000_000_000);
    let c = calculate(CalcOp::Sum, &vec![largest; 1_000]).unwrap();
    assert_eq!(c.result, m(1_000_000_000_000_000));
    // Beyond i64 it is an error, never a wrap-around.
    let top = Paisa::try_from_paisa(i64::MAX - i64::MAX % 100).unwrap();
    let big = Value::Money(top);
    assert_eq!(
        calculate(CalcOp::Sum, &[big, big]).unwrap_err(),
        FigureError::Overflow
    );
    assert_eq!(
        calculate(
            CalcOp::Difference,
            &[
                Value::Money(Paisa::try_from_paisa(-top.as_paisa()).unwrap()),
                big
            ]
        )
        .unwrap_err(),
        FigureError::Overflow
    );
    assert_eq!(FigureError::Overflow.code(), ErrorCode::ErrOverflow);
}

// ---- question_numbers ---------------------------------------------------------------------------

fn numbers(text: &str) -> Vec<(i64, NumberUnit)> {
    question_numbers(text)
        .iter()
        .map(|n| (n.number, n.unit))
        .collect()
}

#[test]
fn the_numbers_of_a_question_with_their_units() {
    assert_eq!(numbers("agle 6 mahine"), vec![(6, NumberUnit::Months)]);
    assert_eq!(
        numbers("agar commission isi raftar se chalta raha to agle 6 mahine mein kitna hoga"),
        vec![(6, NumberUnit::Months)]
    );
    assert_eq!(
        numbers("commission ka 3 lakh ka target kab pura hoga"),
        vec![(300_000, NumberUnit::Amount)]
    );
    assert_eq!(
        numbers("pichle 10 din mein 10,000 se zyada"),
        vec![(10, NumberUnit::Days), (10_000, NumberUnit::Plain)]
    );
    assert_eq!(
        numbers("kis kis ke zimme 10,000 se zyada hai"),
        vec![(10_000, NumberUnit::Plain)]
    );
    assert_eq!(
        numbers("pichle 2 saal ka hisab"),
        vec![(2, NumberUnit::Years)]
    );
    assert_eq!(numbers("pichle 3 mahine"), vec![(3, NumberUnit::Months)]);
    assert_eq!(numbers("2.5 lakh"), vec![(250_000, NumberUnit::Amount)]);
    assert_eq!(numbers("5 hazar"), vec![(5_000, NumberUnit::Amount)]);
    assert_eq!(numbers("2 crore"), vec![(20_000_000, NumberUnit::Amount)]);
    assert_eq!(
        numbers("3,00,000 se zyada"),
        vec![(300_000, NumberUnit::Plain)]
    );
    assert_eq!(numbers("بکری 12"), vec![(12, NumberUnit::Plain)]);
}

#[test]
fn a_percent_is_read_with_its_basis_points() {
    let n = question_numbers("10% zyada");
    assert_eq!(
        (n[0].number, n[0].unit, n[0].bps),
        (10, NumberUnit::Percent, Some(1_000))
    );
    assert_eq!(n[0].text, "10%");
    let n = question_numbers("2.5 % kam");
    assert_eq!((n[0].number, n[0].bps), (2, Some(250)));
    let n = question_numbers("12.75%");
    assert_eq!(n[0].bps, Some(1_275));
    // Three decimals do not fit basis points: not read.
    assert!(question_numbers("2.555%").is_empty());
}

#[test]
fn digits_of_every_script_are_read() {
    assert_eq!(numbers("٣ لاکھ"), vec![(300_000, NumberUnit::Amount)]);
    assert_eq!(numbers("۶ مہینے"), vec![(6, NumberUnit::Months)]);
    assert_eq!(numbers("٢٨٨٠"), vec![(2_880, NumberUnit::Plain)]);
    assert_eq!(numbers("２８８０"), vec![(2_880, NumberUnit::Plain)]);
    assert_eq!(numbers("پچھلے ۱۰ دن"), vec![(10, NumberUnit::Days)]);
    assert_eq!(
        numbers("10 فیصد").first().map(|n| n.1),
        Some(NumberUnit::Percent)
    );
}

#[test]
fn what_is_not_a_number_is_not_read() {
    // A plain decimal, a number word, a number too long, and a lakh that does not come out whole.
    assert!(question_numbers("2.5").is_empty());
    assert!(question_numbers("teen mahine").is_empty());
    assert!(question_numbers("تین لاکھ").is_empty());
    assert!(question_numbers("1234567890123456789012").is_empty());
    assert!(question_numbers("0.0000001 lakh").is_empty());
    assert!(question_numbers("").is_empty());
    // Two numbers apart by a comma and a single digit are two numbers, not one.
    assert_eq!(
        numbers("5,6 mahine"),
        vec![(5, NumberUnit::Plain), (6, NumberUnit::Months)]
    );
}

#[test]
fn a_date_gives_its_parts_and_at_most_twenty_numbers_are_read() {
    assert_eq!(
        numbers("10-10-2026"),
        vec![
            (10, NumberUnit::Plain),
            (10, NumberUnit::Plain),
            (2_026, NumberUnit::Plain)
        ]
    );
    let many = (1..=50)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(question_numbers(&many).len(), MAX_QUESTION_NUMBERS);
}

#[test]
fn amounts_next_to_each_other_are_one_amount() {
    let n = question_numbers("3 lakh 50 hazar ka target");
    assert_eq!(n.len(), 1);
    assert_eq!((n[0].number, n[0].unit), (350_000, NumberUnit::Amount));
    assert_eq!(n[0].text, "3 lakh 50 hazar");
    assert_eq!(
        numbers("2 crore 50 lakh"),
        vec![(25_000_000, NumberUnit::Amount)]
    );
    assert_eq!(
        numbers("1 crore 2 lakh 5 hazar"),
        vec![(10_205_000, NumberUnit::Amount)]
    );
    // A bigger word after a smaller one is not a joined amount: two amounts.
    assert_eq!(
        numbers("3 hazar 5 lakh"),
        vec![(3_000, NumberUnit::Amount), (500_000, NumberUnit::Amount)]
    );
    assert_eq!(
        numbers("3 lakh 5 lakh"),
        vec![(300_000, NumberUnit::Amount), (500_000, NumberUnit::Amount)]
    );
    // A plain number after an amount stays its own number.
    assert_eq!(
        numbers("3 lakh 6 mahine"),
        vec![(300_000, NumberUnit::Amount), (6, NumberUnit::Months)]
    );
}

#[test]
fn multiplier_words_the_app_does_not_read_skip_the_number() {
    // "3 thousand" must never become a plain 3 (a target 1,000 times too small).
    for text in [
        "3 thousand",
        "50k",
        "50 K",
        "2 million",
        "5 sau",
        "1 billion",
        "3 hundred",
        "٣ ارب",
    ] {
        assert!(question_numbers(text).is_empty(), "{text}");
    }
    // The numbers around it are still read.
    assert_eq!(
        numbers("3 thousand aur 6 mahine"),
        vec![(6, NumberUnit::Months)]
    );
}

#[test]
fn percent_signs_and_separators_of_other_scripts() {
    for text in ["10٪", "10 ٪", "１０％", "10％"] {
        let n = question_numbers(text);
        assert_eq!(
            (n[0].number, n[0].unit),
            (10, NumberUnit::Percent),
            "{text}"
        );
    }
    assert_eq!(numbers("1٬00٬000"), vec![(100_000, NumberUnit::Plain)]);
    assert_eq!(numbers("2٫5 lakh"), vec![(250_000, NumberUnit::Amount)]);
    // The Urdu comma is never a thousands separator; a minus is never read.
    assert_eq!(
        numbers("5،6"),
        vec![(5, NumberUnit::Plain), (6, NumberUnit::Plain)]
    );
    assert_eq!(
        numbers("10-20"),
        vec![(10, NumberUnit::Plain), (20, NumberUnit::Plain)]
    );
}

#[test]
fn an_amount_followed_by_a_bare_number_is_not_read_at_all() {
    // Money agent final review #1: "3 lakh 50" could be 3,50,000 or 3 lakh and 50. The app cannot
    // tell, so it reads neither, and the model's own number is refused.
    for text in [
        "3 lakh 50 ka target",
        "3 lakh 50",
        "5 hazar 3 sau",
        "3 lakh50k",
        "3 lakh 2.5",
        "3 lakh 5,000 se zyada",
    ] {
        assert!(
            question_numbers(text).is_empty(),
            "{text}: {:?}",
            question_numbers(text)
        );
    }
    // A number with its own unit after an amount is its own number.
    assert_eq!(
        numbers("3 lakh 6 mahine"),
        vec![(300_000, NumberUnit::Amount), (6, NumberUnit::Months)]
    );
    assert_eq!(
        numbers("3 lakh 10 din"),
        vec![(300_000, NumberUnit::Amount), (10, NumberUnit::Days)]
    );
    // An amount, then a plain number far away (words between) is a separate number.
    assert_eq!(
        numbers("3 lakh ka target aur bikri 12"),
        vec![(300_000, NumberUnit::Amount), (12, NumberUnit::Plain)]
    );
}

#[test]
fn thousands_commas_must_make_a_proper_number() {
    // Money agent final review #2: a list of بکری numbers must not become one invented number.
    assert_eq!(
        numbers("bikri 12,15 aur 18"),
        vec![
            (12, NumberUnit::Plain),
            (15, NumberUnit::Plain),
            (18, NumberUnit::Plain)
        ]
    );
    assert_eq!(
        numbers("1,00"),
        vec![(1, NumberUnit::Plain), (0, NumberUnit::Plain)]
    );
    assert_eq!(
        numbers("12,34"),
        vec![(12, NumberUnit::Plain), (34, NumberUnit::Plain)]
    );
    assert_eq!(
        numbers("1234,567"),
        vec![(1_234, NumberUnit::Plain), (567, NumberUnit::Plain)]
    );
    // Proper groupings, the international and the lakh way, are one number.
    assert_eq!(numbers("1,234,567"), vec![(1_234_567, NumberUnit::Plain)]);
    assert_eq!(numbers("12,34,567"), vec![(1_234_567, NumberUnit::Plain)]);
    assert_eq!(numbers("3,00,000"), vec![(300_000, NumberUnit::Plain)]);
    assert_eq!(numbers("10,000"), vec![(10_000, NumberUnit::Plain)]);
    assert_eq!(numbers("500,600"), vec![(500_600, NumberUnit::Plain)]);
    assert_eq!(
        numbers("10,00,00,000"),
        vec![(100_000_000, NumberUnit::Plain)]
    );
    // A mixed grouping is not one number: it is never read as 10,00,00,000.
    let mixed = numbers("1,00,000,000");
    assert!(mixed.iter().all(|(n, _)| *n != 100_000_000), "{mixed:?}");
    assert_eq!(mixed, vec![(1, NumberUnit::Plain), (0, NumberUnit::Plain)]);
}

#[test]
fn a_comma_used_as_a_decimal_point_reads_neither_part() {
    // Money agent final review #4: "1,5 lakh" is not 1 and 5 lakh.
    assert!(question_numbers("1,5 lakh").is_empty());
    assert!(question_numbers("10,5%").is_empty());
    assert!(question_numbers("2,5 hazar").is_empty());
    // A list with units is still a list.
    assert_eq!(
        numbers("5,6 mahine"),
        vec![(5, NumberUnit::Plain), (6, NumberUnit::Months)]
    );
}

#[test]
fn a_misspelled_multiplier_is_not_read_as_a_plain_number() {
    // Money agent final review #3: "3 laakh" must never become a plain 3.
    for text in [
        "3 laakh",
        "3 lak",
        "3 laks",
        "3 lac",
        "5 hazr",
        "5 hajaar",
        "2 cror",
        "٣ لاکھھ",
    ] {
        let found = question_numbers(text);
        // "lac" and "hajaar"-like known spellings may be read as amounts; none may be plain.
        assert!(
            found.iter().all(|n| n.unit != NumberUnit::Plain),
            "{text}: {found:?}"
        );
    }
    assert!(question_numbers("3 laakh").is_empty());
    assert!(question_numbers("3 lak").is_empty());
    // The known spellings still work, and ordinary words after a number do not hide it.
    assert_eq!(numbers("3 lac"), vec![(300_000, NumberUnit::Amount)]);
    assert_eq!(numbers("12 karo"), vec![(12, NumberUnit::Plain)]);
    assert_eq!(numbers("12 bikri"), vec![(12, NumberUnit::Plain)]);
    // Ordinary words that only begin like a multiplier word are not caught.
    assert_eq!(numbers("12 bill"), vec![(12, NumberUnit::Plain)]);
    assert_eq!(
        numbers("20,000 mil gaye"),
        vec![(20_000, NumberUnit::Plain)]
    );
    // Million and billion are listed whole and stay unread.
    assert!(question_numbers("2 million").is_empty());
    assert!(question_numbers("2 billion").is_empty());
}

#[test]
fn january_follows_december() {
    // History October to December 2025, current month January 2026.
    let h = history(2025, 10, &[1_000, 2_000, 3_000]);
    let e = estimate(&h, ym(2026, 1), 3, None).unwrap();
    assert_eq!((e.first, e.last), (ym(2026, 1), ym(2026, 3)));
    assert_eq!(method(&e, Method::SamePace).per_month, Some(rs(2_000)));
    // A history that ends in November is not the month before January.
    let h = history(2025, 9, &[1_000, 2_000, 3_000]);
    assert_eq!(
        estimate(&h, ym(2026, 1), 3, None).unwrap_err(),
        ForecastError::HistoryNotConsecutive
    );
}

#[test]
fn every_step_of_the_working_is_the_figure_shown() {
    // Money agent B30, E2E-59: the steps carry the shown numbers.
    let e = estimate(&e2e_59(), ym(2026, 10), 6, None).unwrap();
    let growth = method(&e, Method::SameGrowth);
    // The last step is the sum of the months shown (after the floor of 0).
    let last = growth.steps.last().unwrap();
    assert_eq!(last.op, Op::Sum);
    let shown: Vec<Value> = growth.months.iter().map(|p| Value::Money(*p)).collect();
    assert_eq!(last.inputs, shown);
    assert_eq!(last.result, Value::Money(growth.total));
    // And on a falling trend the floored months (0, not -10,000) are what the working sums.
    let h = history(2026, 4, &[100_000, 90_000, 80_000, 70_000, 60_000, 50_000]);
    let e = estimate(&h, ym(2026, 10), 6, None).unwrap();
    let growth = method(&e, Method::SameGrowth);
    assert_eq!(growth.steps[1].result, m(-10_000));
    assert_eq!(growth.steps.last().unwrap().inputs[5], m(0));
}

#[test]
fn a_negative_month_is_followed_for_pace_and_season_but_not_below_zero_for_growth() {
    // PRD FR-7.86 floors only the same growth; a loss month stays a loss for the others
    // (money agent Q1: until Umar answers, follow the PRD as written).
    let h = history(2025, 10, &[-3_000; 12]);
    let e = estimate(&h, ym(2026, 10), 3, None).unwrap();
    assert_eq!(method(&e, Method::SamePace).per_month, Some(rs(-3_000)));
    assert_eq!(method(&e, Method::SamePace).total, rs(-9_000));
    assert_eq!(
        months_in_rupees(method(&e, Method::SameSeason)),
        vec![-3_000; 3]
    );
    // Growth: change 0 from -3,000, floored at 0.
    assert_eq!(months_in_rupees(method(&e, Method::SameGrowth)), vec![0; 3]);
}

// ---- property tests -----------------------------------------------------------------------------

/// Writes digits with commas: `[3]` is 1,234,567; `[3, 2]` is 12,34,567 (the lakh grouping).
fn group(digits: &str, sizes: &[usize]) -> String {
    let chars: Vec<char> = digits.chars().collect();
    let mut parts: Vec<String> = Vec::new();
    let mut end = chars.len();
    let mut i = 0;
    while end > 0 {
        let size = sizes[i.min(sizes.len() - 1)];
        let start = end.saturating_sub(size);
        parts.push(chars[start..end].iter().collect());
        end = start;
        i += 1;
    }
    parts.reverse();
    parts.join(",")
}

fn amounts() -> impl Strategy<Value = i64> {
    -10_000_000_000i64..=10_000_000_000
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// A sum equals its parts, and every result is a whole number of rupees.
    #[test]
    fn a_sum_equals_its_parts(values in prop::collection::vec(amounts(), 1..40)) {
        let inputs: Vec<Value> = values.iter().map(|v| m(*v)).collect();
        let c = calculate(CalcOp::Sum, &inputs).unwrap();
        let expected: i64 = values.iter().sum();
        prop_assert_eq!(c.result, m(expected));
        let Value::Money(p) = c.result else { panic!() };
        prop_assert_eq!(p.as_paisa() % 100, 0);
    }

    /// `first - second + second == first`, and the direction follows the sign.
    #[test]
    fn a_difference_gives_back_the_first(a in amounts(), b in amounts()) {
        let c = calculate(CalcOp::Difference, &[m(a), m(b)]).unwrap();
        let Value::Money(diff) = c.result else { panic!() };
        prop_assert_eq!(diff.rupees() + b, a);
        let want = if a > b { Dir::Up } else if a < b { Dir::Down } else { Dir::Same };
        prop_assert_eq!(c.dir, Some(want));
    }

    /// The average of equal figures is that figure; any average sits between the smallest and the
    /// biggest (to the nearest rupee).
    #[test]
    fn an_average_sits_between_its_inputs(values in prop::collection::vec(amounts(), 1..30), same in amounts(), n in 1usize..30) {
        let inputs = vec![m(same); n];
        prop_assert_eq!(calculate(CalcOp::Average, &inputs).unwrap().result, m(same));
        let c = calculate(CalcOp::Average, &values.iter().map(|v| m(*v)).collect::<Vec<_>>()).unwrap();
        let Value::Money(avg) = c.result else { panic!() };
        let (lo, hi) = (values.iter().min().unwrap(), values.iter().max().unwrap());
        prop_assert!(avg.rupees() >= *lo && avg.rupees() <= *hi, "{} not in {lo}..{hi}", avg.rupees());
    }

    /// A part of a total is never more than 100%, and a bigger part is never a smaller percent.
    #[test]
    fn a_percent_of_a_total_is_between_zero_and_a_hundred(total in 1i64..10_000_000_000, p1 in 0i64..=1_000, p2 in 0i64..=1_000) {
        let (a, b) = (total * p1.min(p2) / 1_000, total * p1.max(p2) / 1_000);
        let pct = |x: i64| match calculate(CalcOp::PercentOf, &[m(x), m(total)]).unwrap().result {
            Value::Percent(v) => v,
            _ => panic!(),
        };
        prop_assert!(pct(a) <= 10_000 && pct(b) <= 10_000);
        prop_assert!(pct(a) <= pct(b));
    }

    /// The direction of a change is the sign of new - old.
    #[test]
    fn a_change_in_percent_points_the_right_way(old in 1i64..10_000_000_000, new in 0i64..10_000_000_000) {
        let c = calculate(CalcOp::ChangePercent, &[m(old), m(new)]).unwrap();
        let want = if new > old { Dir::Up } else if new < old { Dir::Down } else { Dir::Same };
        prop_assert_eq!(c.dir, Some(want));
    }

    /// Equal months give that same figure by every method, and the totals agree with the months.
    #[test]
    fn equal_months_give_the_same_figure_by_every_method(v in 0i64..1_000_000_000, n in 3usize..30, ahead in 1u32..=12) {
        let h = history(2020, 1, &vec![v; n]);
        let current = ym(2020, 1).plus_months(u32::try_from(n).unwrap()).unwrap();
        let e = estimate(&h, current, ahead, None).unwrap();
        for x in &e.methods {
            prop_assert!(x.months.iter().all(|p| *p == rs(v)), "{:?}", x.method);
            prop_assert_eq!(x.total, rs(v * i64::from(ahead)));
            prop_assert_eq!(x.months.len(), ahead as usize);
        }
    }

    /// With history of 0 or more, no projected month is below 0, every figure is a whole rupee,
    /// and the total is built from the months (same pace: per month times the months).
    #[test]
    fn no_projected_month_is_below_zero(values in prop::collection::vec(0i64..1_000_000_000, 3..30), ahead in 1u32..=12) {
        let n = values.len();
        let h = history(2020, 1, &values);
        let current = ym(2020, 1).plus_months(u32::try_from(n).unwrap()).unwrap();
        let e = estimate(&h, current, ahead, None).unwrap();
        for x in &e.methods {
            prop_assert!(x.months.iter().all(|p| !p.is_negative() && p.as_paisa() % 100 == 0));
            prop_assert!(!x.total.is_negative());
            let sum: i64 = x.months.iter().map(|p| p.rupees()).sum();
            prop_assert_eq!(x.total.rupees(), sum);
        }
        let pace = method(&e, Method::SamePace);
        prop_assert_eq!(pace.total.rupees(), pace.per_month.unwrap().rupees() * i64::from(ahead));
    }

    /// The target month is the smallest month whose running total reaches the target.
    #[test]
    fn the_target_month_is_the_first_to_reach_it(values in prop::collection::vec(0i64..1_000_000, 6..20), target in 1i64..20_000_000) {
        let n = values.len();
        let h = history(2020, 1, &values);
        let current = ym(2020, 1).plus_months(u32::try_from(n).unwrap()).unwrap();
        let e = estimate(&h, current, 12, Some(rs(target))).unwrap();
        for x in &e.methods {
            let mut running = 0i64;
            let mut first = None;
            for (k, p) in x.months.iter().enumerate() {
                running += p.rupees();
                if running >= target {
                    first = Some(current.plus_months(u32::try_from(k).unwrap()).unwrap());
                    break;
                }
            }
            prop_assert_eq!(x.target_month, first, "{:?}", x.method);
        }
    }

    /// Reading numbers never panics, on any text, and reads at most twenty.
    #[test]
    fn question_numbers_never_panic(text in ".*") {
        prop_assert!(question_numbers(&text).len() <= MAX_QUESTION_NUMBERS);
    }

    /// The same, for text made of digits of every script, separators and unit words.
    #[test]
    fn question_numbers_never_panic_on_digits_and_units(parts in prop::collection::vec(
        prop::sample::select(vec!["1", "23", "٣", "۶", "２", ",", ".", "٫", "٬", " ", "%", "lakh", "hazar", "crore", "mahine", "din", "saal", "لاکھ", "مہینے", "999999999999999999999", "-", "/"]),
        0..40,
    )) {
        let text: String = parts.concat();
        let found = question_numbers(&text);
        prop_assert!(found.len() <= MAX_QUESTION_NUMBERS);
        for n in found {
            prop_assert!(n.number >= 0);
        }
    }

    /// `difference(a, b) = -difference(b, a)`, a sum is the same in any order, a figure is 100% of
    /// itself and 0 is 0% of anything.
    #[test]
    fn the_calculator_obeys_its_identities(a in amounts(), c in amounts(), values in prop::collection::vec(amounts(), 1..20), x in 1i64..10_000_000_000, base in 1i64..10_000_000_000) {
        let ab = calculate(CalcOp::Difference, &[m(a), m(c)]).unwrap().result;
        let ba = calculate(CalcOp::Difference, &[m(c), m(a)]).unwrap().result;
        let (Value::Money(ab), Value::Money(ba)) = (ab, ba) else { panic!() };
        prop_assert_eq!(ab.as_paisa(), -ba.as_paisa());
        let forward: Vec<Value> = values.iter().map(|v| m(*v)).collect();
        let mut backward = forward.clone();
        backward.reverse();
        prop_assert_eq!(
            calculate(CalcOp::Sum, &forward).unwrap().result,
            calculate(CalcOp::Sum, &backward).unwrap().result
        );
        prop_assert_eq!(calculate(CalcOp::PercentOf, &[m(x), m(x)]).unwrap().result, Value::Percent(10_000));
        prop_assert_eq!(calculate(CalcOp::PercentOf, &[m(0), m(base)]).unwrap().result, Value::Percent(0));
    }

    /// The direction of a change is the direction of the difference, for amounts and for balances
    /// on one side (by size).
    #[test]
    fn a_change_and_a_difference_point_the_same_way(old in 1i64..10_000_000_000, new in 0i64..10_000_000_000, credit in prop::bool::ANY) {
        let (a, z) = if credit { (b(-old), b(-new)) } else { (b(old), b(new)) };
        let change = calculate(CalcOp::ChangePercent, &[a, z]).unwrap();
        let diff = calculate(CalcOp::Difference, &[z, a]).unwrap();
        prop_assert_eq!(change.dir, diff.dir);
        let want = if new > old { Dir::Up } else if new < old { Dir::Down } else { Dir::Same };
        prop_assert_eq!(change.dir, Some(want));
    }

    /// A larger `ahead` keeps the earlier months and the same target month (when it is within the
    /// shorter span), and only the last 3, 6 or 12 months change the result.
    #[test]
    fn more_months_ahead_keep_the_earlier_ones(values in prop::collection::vec(0i64..1_000_000, 12..24), short in 1u32..=6, target in 1i64..5_000_000) {
        let n = values.len();
        let h = history(2020, 1, &values);
        let current = ym(2020, 1).plus_months(u32::try_from(n).unwrap()).unwrap();
        let near = estimate(&h, current, short, Some(rs(target))).unwrap();
        let far = estimate(&h, current, 12, Some(rs(target))).unwrap();
        for (x, y) in near.methods.iter().zip(far.methods.iter()) {
            prop_assert_eq!(x.method, y.method);
            prop_assert_eq!(&x.months[..], &y.months[..x.months.len()]);
            if let Some(t) = x.target_month {
                prop_assert_eq!(y.target_month, Some(t));
            }
        }
        // Only the last 12 months matter: change an older month and nothing changes.
        let mut older = values.clone();
        older[0] += 12_345;
        let h2 = history(2020, 1, &older);
        let again = estimate(&h2, current, 12, Some(rs(target))).unwrap();
        for (x, y) in far.methods.iter().zip(again.methods.iter()) {
            if n > 12 {
                prop_assert_eq!(&x.months, &y.months);
            }
        }
    }

    /// The same growth is never below 0 for any history (a loss included), and the same season is
    /// exactly the slice of last year.
    #[test]
    fn growth_is_never_below_zero_and_season_is_the_slice(values in prop::collection::vec(-1_000_000i64..1_000_000, 12..20), ahead in 1u32..=12) {
        let n = values.len();
        let h = history(2020, 1, &values);
        let current = ym(2020, 1).plus_months(u32::try_from(n).unwrap()).unwrap();
        let e = estimate(&h, current, ahead, None).unwrap();
        prop_assert!(method(&e, Method::SameGrowth).months.iter().all(|p| !p.is_negative()));
        let year = &values[n - 12..];
        let season = method(&e, Method::SameSeason);
        prop_assert_eq!(months_in_rupees(season), year[..ahead as usize].to_vec());
    }

    /// An arithmetic series continues exactly by the same growth: month k is last + k x step.
    #[test]
    fn an_arithmetic_series_continues_exactly(start in 0i64..1_000_000, step in -1_000i64..=1_000, ahead in 1u32..=12) {
        let values: Vec<i64> = (0..6).map(|i| start + step * i + 6_000).collect();
        let h = history(2026, 4, &values);
        let e = estimate(&h, ym(2026, 10), ahead, None).unwrap();
        let growth = method(&e, Method::SameGrowth);
        let last = values[5];
        for (k, p) in growth.months.iter().enumerate() {
            let want = (last + step * (k as i64 + 1)).max(0);
            prop_assert_eq!(p.rupees(), want);
        }
    }

    /// Numbers written with international or lakh grouping are read back the same.
    #[test]
    fn grouped_numbers_read_back(n in 100i64..1_000_000_000_000) {
        let digits = n.to_string();
        let international = group(&digits, &[3]);
        let lakh = group(&digits, &[3, 2]);
        for text in [international, lakh] {
            let found = question_numbers(&format!("ka {text} se zyada"));
            prop_assert_eq!(found.len(), 1, "{}", text);
            prop_assert_eq!(found[0].number, n, "{}", text);
        }
    }

    /// "n lakh" is n x 1,00,000 when it fits.
    #[test]
    fn n_lakh_is_n_times_a_lakh(n in 1i64..90_000_000_000_000) {
        let found = question_numbers(&format!("{n} lakh"));
        prop_assert_eq!(found[0].number, n * 100_000);
        prop_assert_eq!(found[0].unit, NumberUnit::Amount);
    }
}

// ---- what-ifs and the end of the year (plan 52 item 5, money agent design review) ---------------

fn what_if(history: &[MonthValue], now: YearMonth, ahead: u32, factor: PaceFactor) -> Estimate {
    estimate_with(history, now, ahead, None, Some(factor)).unwrap()
}

#[test]
fn the_months_to_the_end_of_the_year_count_the_current_month() {
    assert_eq!(months_to_year_end(ym(2026, 10)).unwrap(), 3);
    assert_eq!(months_to_year_end(ym(2026, 12)).unwrap(), 1);
    assert_eq!(months_to_year_end(ym(2027, 1)).unwrap(), 12);
    assert_eq!(months_through(ym(2026, 10), ym(2027, 3)).unwrap(), 6);
    // Before the current month, or more than 12 months: refused.
    assert_eq!(
        months_through(ym(2026, 10), ym(2026, 9)).unwrap_err(),
        ForecastError::AheadOutOfRange
    );
    assert_eq!(
        months_through(ym(2026, 10), ym(2027, 10)).unwrap_err(),
        ForecastError::AheadOutOfRange
    );
}

#[test]
fn the_end_of_the_year_on_the_e2e_59_data() {
    // Today 10-10-26: October, November and December are 3 months ahead.
    let h = e2e_59();
    let e = estimate(&h, ym(2026, 10), 3, None).unwrap();
    assert_eq!(method(&e, Method::SamePace).total, rs(144_000));
    assert_eq!(method(&e, Method::SameGrowth).total, rs(171_000));
}

#[test]
fn the_pace_doubles_halves_triples_and_moves_by_a_percent() {
    let h = e2e_59(); // the pace is 48,000
    let now = ym(2026, 10);
    for (factor, monthly, three, six) in [
        (PaceFactor::DOUBLE, 96_000, 288_000, 576_000),
        (PaceFactor::HALF, 24_000, 72_000, 144_000),
        (PaceFactor::TRIPLE, 144_000, 432_000, 864_000),
        (PaceFactor::up(1_000).unwrap(), 52_800, 158_400, 316_800),
        (PaceFactor::down(2_500).unwrap(), 36_000, 108_000, 216_000),
    ] {
        let e3 = what_if(&h, now, 3, factor);
        let w = method(&e3, Method::WhatIfPace);
        assert_eq!(w.per_month, Some(rs(monthly)), "{factor:?}");
        assert_eq!(w.total, rs(three), "{factor:?}");
        assert_eq!(w.factor, Some(factor));
        // Same pace is still there, as the baseline, and the what-if comes right after it.
        assert_eq!(e3.methods[0].method, Method::SamePace);
        assert_eq!(e3.methods[1].method, Method::WhatIfPace);
        assert_eq!(method(&e3, Method::SamePace).total, rs(144_000));
        let e6 = what_if(&h, now, 6, factor);
        assert_eq!(method(&e6, Method::WhatIfPace).total, rs(six), "{factor:?}");
    }
}

#[test]
fn a_what_if_is_rounded_once_on_the_monthly_pace_and_the_total_is_month_times_months() {
    // The last 3 months 45,000; 48,000; 52,000: the pace 48,333.33 rounds to 48,333.
    let h = history(2026, 4, &[36_000, 39_000, 42_000, 45_000, 48_000, 52_000]);
    let now = ym(2026, 10);
    assert_eq!(
        method(&estimate(&h, now, 3, None).unwrap(), Method::SamePace).per_month,
        Some(rs(48_333))
    );
    for (factor, monthly) in [
        (PaceFactor::up(1_000).unwrap(), 53_166), // 53,166.3
        (PaceFactor::DOUBLE, 96_666),
        (PaceFactor::HALF, 24_167),                 // 24,166.5 half up
        (PaceFactor::up(5_000).unwrap(), 72_500),   // 48,333 + 24,166.5 half up
        (PaceFactor::down(5_000).unwrap(), 24_167), // one rounding, not 24,166
        (PaceFactor::up(250).unwrap(), 49_541),     // 48,333 + round(1,208.325)
        (PaceFactor::down(7_500).unwrap(), 12_083), // round(12,083.25)
    ] {
        let e = what_if(&h, now, 3, factor);
        let w = method(&e, Method::WhatIfPace);
        assert_eq!(w.per_month, Some(rs(monthly)), "{factor:?}");
        assert_eq!(w.total, rs(monthly * 3), "{factor:?}");
    }
    // The totals the money agent worked out: up 10% 159,498; doubled 289,998; half 72,501.
    let total = |f| method(&what_if(&h, now, 3, f), Method::WhatIfPace).total;
    assert_eq!(total(PaceFactor::up(1_000).unwrap()), rs(159_498));
    assert_eq!(total(PaceFactor::DOUBLE), rs(289_998));
    assert_eq!(total(PaceFactor::HALF), rs(72_501));
    // The working shows the rounded step: "48,333 × 110.00% = 53,166".
    let e = what_if(&h, now, 3, PaceFactor::up(1_000).unwrap());
    let w = method(&e, Method::WhatIfPace);
    assert!(w.steps.iter().any(|s| s.op == Op::Times
        && s.inputs == vec![m(48_333), Value::Percent(11_000)]
        && s.result == m(53_166)));
}

#[test]
fn the_agent_example_doubled() {
    // Money agent 9.1: a pace of 46,333 doubled is 92,666, and 555,996 over 6 months.
    let h = history(2026, 4, &[40_000, 41_000, 42_000, 45_000, 46_000, 48_000]);
    let e = what_if(&h, ym(2026, 10), 6, PaceFactor::DOUBLE);
    let w = method(&e, Method::WhatIfPace);
    assert_eq!(w.per_month, Some(rs(92_666)));
    assert_eq!(w.total, rs(555_996));
}

#[test]
fn a_target_with_a_what_if_is_the_first_month_whose_running_total_reaches_it() {
    let h = e2e_59();
    let now = ym(2026, 10);
    // 5,00,000 with the doubled pace: 96,000 a month reaches it in the 6th month, March 2027.
    let e = estimate_with(&h, now, 12, Some(rs(500_000)), Some(PaceFactor::DOUBLE)).unwrap();
    assert_eq!(
        method(&e, Method::WhatIfPace).target_month,
        Some(ym(2027, 3))
    );
    // Other methods with the same target and 12 months (money agent): August, May.
    assert_eq!(method(&e, Method::SamePace).target_month, Some(ym(2027, 8)));
    assert_eq!(
        method(&e, Method::SameGrowth).target_month,
        Some(ym(2027, 5))
    );
    let up = estimate_with(
        &h,
        now,
        12,
        Some(rs(500_000)),
        Some(PaceFactor::up(1_000).unwrap()),
    )
    .unwrap();
    assert_eq!(
        method(&up, Method::WhatIfPace).target_month,
        Some(ym(2027, 7))
    );
    let half = estimate_with(&h, now, 12, Some(rs(500_000)), Some(PaceFactor::HALF)).unwrap();
    assert_eq!(method(&half, Method::WhatIfPace).target_month, None);
    // Only to the end of the year (3 months): not reached.
    let short = estimate_with(&h, now, 3, Some(rs(500_000)), Some(PaceFactor::DOUBLE)).unwrap();
    assert_eq!(method(&short, Method::WhatIfPace).target_month, None);
}

#[test]
fn a_what_if_edge_cases_are_refused_or_zero_never_guessed() {
    let now = ym(2026, 10);
    // Exactly 3 months: same pace and the what-if, not same growth.
    let three = history(2026, 7, &[45_000, 48_000, 51_000]);
    let e = what_if(&three, now, 3, PaceFactor::DOUBLE);
    assert_eq!(e.methods.len(), 2);
    // Fewer than 3 months: the usual too-little-history, no what-if.
    let two = history(2026, 8, &[48_000, 51_000]);
    assert_eq!(
        estimate_with(&two, now, 3, None, Some(PaceFactor::DOUBLE)).unwrap_err(),
        ForecastError::TooLittleHistory { months: 2 }
    );
    // A pace of 0 gives 0 and a target is never reached.
    let zero = history(2026, 7, &[0, 0, 0]);
    let e = estimate_with(&zero, now, 3, Some(rs(1)), Some(PaceFactor::DOUBLE)).unwrap();
    let w = method(&e, Method::WhatIfPace);
    assert_eq!(
        (w.per_month, w.total, w.target_month),
        (Some(rs(0)), rs(0), None)
    );
    // A loss (profit below 0): up or down of it is a guess, so it is refused.
    let loss = history(2026, 7, &[-5_000, -4_000, -3_000]);
    assert_eq!(
        estimate_with(&loss, now, 3, None, Some(PaceFactor::DOUBLE)).unwrap_err(),
        ForecastError::NegativePace
    );
    // Factors outside the range, or 100%.
    assert_eq!(
        PaceFactor::up(0).unwrap_err(),
        ForecastError::FactorOutOfRange
    );
    assert_eq!(
        PaceFactor::up(90_001).unwrap_err(),
        ForecastError::FactorOutOfRange
    );
    assert_eq!(
        PaceFactor::down(0).unwrap_err(),
        ForecastError::FactorOutOfRange
    );
    assert_eq!(
        PaceFactor::down(10_000).unwrap_err(),
        ForecastError::FactorOutOfRange
    );
    assert_eq!(PaceFactor::up(90_000).unwrap().bps(), MAX_FACTOR_BPS);
}

#[test]
fn a_scaled_month_above_the_largest_amount_is_refused_and_at_it_is_accepted() {
    let now = ym(2026, 10);
    // 1,000,000,000 rupees a month, times 10: exactly the largest amount a screen shows.
    let big = history(2026, 7, &[1_000_000_000; 3]);
    let ten = PaceFactor::up(90_000).unwrap();
    let e = what_if(&big, now, 1, ten);
    assert_eq!(
        method(&e, Method::WhatIfPace).per_month,
        Some(rs(10_000_000_000))
    );
    // 1,000,000,001 rupees times 10 is above it.
    let over = history(2026, 7, &[1_000_000_001; 3]);
    assert_eq!(
        estimate_with(&over, now, 1, None, Some(ten)).unwrap_err(),
        ForecastError::Figure(FigureError::Overflow)
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// The scaled pace is the i128 oracle `(P × F + 500,000) ÷ 1,000,000` rupees, and the total is
    /// the month times the months.
    #[test]
    fn the_scaled_pace_is_one_half_up_rounding_of_pace_times_factor(
        a in 0_i64..2_000_000, b in 0_i64..2_000_000, c in 0_i64..2_000_000,
        up in prop::bool::ANY, bps in 1_i32..=90_000, ahead in 1_u32..=12,
    ) {
        let h = history(2026, 7, &[a, b, c]);
        let now = ym(2026, 10);
        // Up is 1 to 90,000 bps (factors with a whole part of 2 or more and a rest, such as up
        // 150% = 25,000); down is 1 to 9,999.
        let factor = if up {
            PaceFactor::up(bps)
        } else {
            PaceFactor::down(bps.min(9_999))
        }
        .unwrap();
        let e = estimate_with(&h, now, ahead, None, Some(factor)).unwrap();
        let pace = method(&e, Method::SamePace).per_month.unwrap().as_paisa() / 100;
        let w = method(&e, Method::WhatIfPace);
        // P rupees × F bps ÷ 10,000, half up, in whole rupees (an i128 oracle).
        let oracle = (i128::from(pace) * i128::from(factor.bps()) + 5_000) / 10_000;
        prop_assert_eq!(i128::from(w.per_month.unwrap().as_paisa() / 100), oracle);
        prop_assert_eq!(w.total.as_paisa(), w.per_month.unwrap().as_paisa() * i64::from(ahead));
        // Down never goes above the pace, up never below it, and the pace itself is the baseline.
        if up {
            prop_assert!(w.per_month.unwrap() >= method(&e, Method::SamePace).per_month.unwrap());
        } else {
            prop_assert!(w.per_month.unwrap() <= method(&e, Method::SamePace).per_month.unwrap());
        }
    }

    /// Double is exactly twice the pace, and a bigger factor never gives a smaller figure.
    #[test]
    fn double_is_twice_the_pace_and_the_scaled_pace_never_falls_as_the_factor_grows(
        a in 0_i64..2_000_000, b in 0_i64..2_000_000, c in 0_i64..2_000_000,
        low in 1_i32..9_999, step in 1_i32..9_999,
    ) {
        let h = history(2026, 7, &[a, b, c]);
        let now = ym(2026, 10);
        let double = what_if(&h, now, 1, PaceFactor::DOUBLE);
        let pace = method(&double, Method::SamePace).per_month.unwrap().as_paisa();
        prop_assert_eq!(method(&double, Method::WhatIfPace).per_month.unwrap().as_paisa(), pace * 2);
        let first = PaceFactor::up(low).unwrap();
        let second = PaceFactor::up(low.saturating_add(step).min(90_000)).unwrap();
        let x = method(&what_if(&h, now, 1, first), Method::WhatIfPace).per_month.unwrap();
        let y = method(&what_if(&h, now, 1, second), Method::WhatIfPace).per_month.unwrap();
        prop_assert!(y >= x);
    }
}

// ---- the whole year (plan 52, Umar 02-11-2026, money agent 9.1 v1.9) ------------------------------
//
// "saal ke aakhir tak": the months of this year so far from the books (never estimated) plus the
// months left, the current month counted as a whole estimated month. Every number below was worked
// by hand by the money agent before the code.

fn year_total(e: &Estimate, which: Method) -> Option<i64> {
    method(e, which).year_total.map(|p| p.rupees())
}

fn to_year_end(history: &[MonthValue], now: YearMonth, factor: Option<PaceFactor>) -> Estimate {
    estimate_to_year_end(history, now, factor).unwrap()
}

#[test]
fn the_whole_year_on_the_e2e_59_data() {
    // April to September 2026 from the books: 36,000 + 39,000 + 42,000 + 45,000 + 48,000 + 51,000.
    let e = to_year_end(&e2e_59(), ym(2026, 10), None);
    assert_eq!((e.first, e.last, e.ahead), (ym(2026, 10), ym(2026, 12), 3));
    let year = e.year.clone().unwrap();
    assert_eq!(year.so_far, rs(261_000));
    assert_eq!(year.months, 6);
    assert_eq!(year.span, Some((ym(2026, 4), ym(2026, 9))));
    // Same season needs 12 months: only two methods, each with its whole year.
    assert_eq!(e.methods.len(), 2);
    let pace = method(&e, Method::SamePace);
    assert_eq!(
        (pace.total, pace.year_total),
        (rs(144_000), Some(rs(405_000)))
    );
    let growth = method(&e, Method::SameGrowth);
    assert_eq!(
        (growth.total, growth.year_total),
        (rs(171_000), Some(rs(432_000)))
    );
    // The working ends with two steps the Owner can check: the books' months added up, then the
    // whole year (books so far + the months ahead).
    let six: Vec<Value> = [36_000, 39_000, 42_000, 45_000, 48_000, 51_000]
        .iter()
        .map(|r| m(*r))
        .collect();
    for found in [pace, growth] {
        let [.., books, last] = found.steps.as_slice() else {
            panic!("too few steps");
        };
        assert_eq!((books.op, &books.inputs), (Op::Sum, &six));
        assert_eq!(books.result, m(261_000));
        assert_eq!(last.op, Op::Sum);
        assert_eq!(last.inputs, vec![m(261_000), Value::Money(found.total)]);
        assert_eq!(last.result, Value::Money(found.year_total.unwrap()));
    }
}

#[test]
fn the_what_if_whole_year_keeps_the_books_and_scales_only_the_months_left() {
    let h = e2e_59(); // the pace is 48,000
    let now = ym(2026, 10);
    for (factor, monthly, whole) in [
        (PaceFactor::DOUBLE, 96_000, 549_000),
        (PaceFactor::HALF, 24_000, 333_000),
        (PaceFactor::up(1_000).unwrap(), 52_800, 419_400),
    ] {
        let e = to_year_end(&h, now, Some(factor));
        let w = method(&e, Method::WhatIfPace);
        assert_eq!(w.per_month, Some(rs(monthly)), "{factor:?}");
        assert_eq!(w.year_total.map(|p| p.rupees()), Some(whole), "{factor:?}");
        // Same pace is the baseline, 405,000, unchanged by the scenario.
        assert_eq!(year_total(&e, Method::SamePace), Some(405_000));
        // The what-if's working is the average, the scaling, the months, then the books' sum and
        // the year: it did not take the year steps of same pace with it.
        let ops: Vec<Op> = w.steps.iter().map(|s| s.op).collect();
        assert_eq!(
            ops,
            vec![Op::Average, Op::Times, Op::Times, Op::Sum, Op::Sum],
            "{factor:?}"
        );
        let pace_ops: Vec<Op> = method(&e, Method::SamePace)
            .steps
            .iter()
            .map(|s| s.op)
            .collect();
        assert_eq!(pace_ops, vec![Op::Average, Op::Times, Op::Sum, Op::Sum]);
    }
}

/// July 2025 to September 2026, 15 months.
fn fifteen_months() -> Vec<MonthValue> {
    history(
        2025,
        7,
        &[
            30_000, 32_000, 34_000, 40_000, 44_000, 50_000, // Jul to Dec 2025
            20_000, 22_000, 25_000, 36_000, 39_000, 42_000, 45_000, 48_000, 51_000, // 2026
        ],
    )
}

#[test]
fn the_whole_year_with_the_same_season_worked_out() {
    let e = to_year_end(&fifteen_months(), ym(2026, 10), None);
    let year = e.year.clone().unwrap();
    // January to September 2026: 67,000 + 261,000.
    assert_eq!(year.so_far, rs(328_000));
    assert_eq!(year.months, 9);
    assert_eq!(year.span, Some((ym(2026, 1), ym(2026, 9))));
    assert_eq!(year_total(&e, Method::SamePace), Some(472_000));
    assert_eq!(year_total(&e, Method::SameGrowth), Some(499_000));
    // The same months of last year: October, November, December 2025 = 40,000 + 44,000 + 50,000.
    assert_eq!(method(&e, Method::SameSeason).total, rs(134_000));
    assert_eq!(year_total(&e, Method::SameSeason), Some(462_000));
    for (factor, whole) in [
        (PaceFactor::DOUBLE, 616_000),
        (PaceFactor::HALF, 400_000),
        (PaceFactor::up(1_000).unwrap(), 486_400),
    ] {
        let w = to_year_end(&fifteen_months(), ym(2026, 10), Some(factor));
        assert_eq!(
            year_total(&w, Method::WhatIfPace),
            Some(whole),
            "{factor:?}"
        );
    }
}

/// April to December 2026: the E2E-59 months and 54,000; 57,000; 60,000.
fn to_december() -> Vec<MonthValue> {
    history(
        2026,
        4,
        &[
            36_000, 39_000, 42_000, 45_000, 48_000, 51_000, 54_000, 57_000, 60_000,
        ],
    )
}

#[test]
fn in_january_nothing_is_so_far_and_the_whole_year_is_the_total() {
    let e = to_year_end(&to_december(), ym(2027, 1), None);
    assert_eq!((e.first, e.last, e.ahead), (ym(2027, 1), ym(2027, 12), 12));
    let year = e.year.clone().unwrap();
    assert_eq!(
        (year.so_far, year.months, year.span),
        (Paisa::ZERO, 0, None)
    );
    // 57,000 × 12; growth 63,000 up to 96,000 in 12 steps of 3,000.
    assert_eq!(method(&e, Method::SamePace).total, rs(684_000));
    assert_eq!(year_total(&e, Method::SamePace), Some(684_000));
    assert_eq!(year_total(&e, Method::SameGrowth), Some(954_000));
    assert_eq!(e.methods.len(), 2, "9 months: no same season");
    // No year step is added: the working is the plain estimate's.
    let plain = estimate(&to_december(), ym(2027, 1), 12, None).unwrap();
    for m in &e.methods {
        let steps = plain
            .methods
            .iter()
            .find(|p| p.method == m.method)
            .map(|p| p.steps.len());
        assert_eq!(Some(m.steps.len()), steps, "{:?}", m.method);
    }
    let double = to_year_end(&to_december(), ym(2027, 1), Some(PaceFactor::DOUBLE));
    assert_eq!(year_total(&double, Method::WhatIfPace), Some(1_368_000));
}

#[test]
fn in_december_one_month_is_left() {
    let h = history(
        2026,
        4,
        &[
            36_000, 39_000, 42_000, 45_000, 48_000, 51_000, 54_000, 57_000,
        ],
    );
    let now = ym(2026, 12);
    let e = to_year_end(&h, now, None);
    assert_eq!((e.first, e.last, e.ahead), (ym(2026, 12), ym(2026, 12), 1));
    let year = e.year.clone().unwrap();
    assert_eq!((year.so_far, year.months), (rs(372_000), 8));
    assert_eq!(year_total(&e, Method::SamePace), Some(426_000)); // 54,000
    assert_eq!(year_total(&e, Method::SameGrowth), Some(432_000)); // 60,000
    for (factor, whole) in [
        (PaceFactor::DOUBLE, 480_000),
        (PaceFactor::HALF, 399_000),
        (PaceFactor::up(1_000).unwrap(), 431_400),
    ] {
        let w = to_year_end(&h, now, Some(factor));
        assert_eq!(
            year_total(&w, Method::WhatIfPace),
            Some(whole),
            "{factor:?}"
        );
    }
    // A rising straight line has the same whole year by growth in October and in December.
    let october = to_year_end(&e2e_59(), ym(2026, 10), None);
    assert_eq!(year_total(&october, Method::SameGrowth), Some(432_000));
}

#[test]
fn a_month_with_nothing_is_still_a_month_of_the_books() {
    // January to March have 0 in the books, April to September as in E2E-59.
    let h = history(
        2026,
        1,
        &[0, 0, 0, 36_000, 39_000, 42_000, 45_000, 48_000, 51_000],
    );
    let year = year_so_far(&h, ym(2026, 10)).unwrap();
    assert_eq!((year.so_far, year.months), (rs(261_000), 9));
    assert_eq!(year.span, Some((ym(2026, 1), ym(2026, 9))));
    // Only the months of this year count: an earlier year is not in it.
    let h2 = history(
        2025,
        7,
        &[
            999_000, 999_000, 999_000, 999_000, 999_000, 999_000, 1_000, 2_000,
        ],
    );
    let year = year_so_far(&h2, ym(2026, 3)).unwrap();
    assert_eq!((year.so_far, year.months), (rs(3_000), 2));
}

#[test]
fn a_loss_keeps_its_sign_in_the_whole_year() {
    // Six months of a 30,000 loss: same pace -30,000 × 3 = -90,000, so -180,000 + -90,000.
    let h = history(2026, 4, &[-30_000; 6]);
    let e = to_year_end(&h, ym(2026, 10), None);
    assert_eq!(e.year.clone().unwrap().so_far, rs(-180_000));
    assert_eq!(method(&e, Method::SamePace).total, rs(-90_000));
    assert_eq!(year_total(&e, Method::SamePace), Some(-270_000));
    // Same growth is never below 0 a month (FR-7.86): the months ahead are 0.
    assert_eq!(method(&e, Method::SameGrowth).total, Paisa::ZERO);
    assert_eq!(year_total(&e, Method::SameGrowth), Some(-180_000));
    // A what-if on a loss is refused, as before.
    assert_eq!(
        estimate_to_year_end(&h, ym(2026, 10), Some(PaceFactor::DOUBLE)).unwrap_err(),
        ForecastError::NegativePace
    );
}

#[test]
fn the_whole_year_needs_three_months_and_a_plain_estimate_has_no_year() {
    let short = history(2026, 8, &[10_000, 20_000]);
    assert_eq!(
        estimate_to_year_end(&short, ym(2026, 10), None).unwrap_err(),
        ForecastError::TooLittleHistory { months: 2 }
    );
    let plain = estimate(&e2e_59(), ym(2026, 10), 3, None).unwrap();
    assert_eq!(plain.year, None);
    assert!(plain.methods.iter().all(|m| m.year_total.is_none()));
    // A history that is not the months before the current one is refused.
    assert_eq!(
        estimate_to_year_end(&e2e_59(), ym(2026, 12), None).unwrap_err(),
        ForecastError::HistoryNotConsecutive
    );
}

#[test]
fn one_month_so_far_is_itself_with_no_sum_step() {
    // November 2025 to January 2026; asked in February, so only January is of this year.
    let h = history(2025, 11, &[10_000, 20_000, 30_000]);
    let e = to_year_end(&h, ym(2026, 2), None);
    let year = e.year.clone().unwrap();
    assert_eq!(
        (year.so_far, year.months, year.step.is_none()),
        (rs(30_000), 1, true)
    );
    assert_eq!(year.span, Some((ym(2026, 1), ym(2026, 1))));
    // The pace is 20,000; 11 months left (February to December) = 220,000; the year 250,000.
    let pace = method(&e, Method::SamePace);
    assert_eq!(
        (pace.total, pace.year_total),
        (rs(220_000), Some(rs(250_000)))
    );
    let ops: Vec<Op> = pace.steps.iter().map(|s| s.op).collect();
    assert_eq!(ops, vec![Op::Average, Op::Times, Op::Sum]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// The year is the books plus the months ahead, for every method, whatever the history, and the
    /// rest is exactly the plain estimate to December (nothing changed, nothing counted twice).
    #[test]
    fn the_whole_year_is_the_books_so_far_plus_the_months_ahead(
        current_month in 1u32..=12,
        values in proptest::collection::vec(-1_000_000_000i64..=1_000_000_000, 3..=30),
        bps in proptest::option::of(1i32..=50_000),
    ) {
        let now = ym(2026, current_month);
        // The history ends the month before `now`.
        let n = u32::try_from(values.len()).unwrap();
        let start = (0..n).fold(now, |at, _| {
            if at.month() == 1 {
                ym(at.year() - 1, 12)
            } else {
                ym(at.year(), at.month() - 1)
            }
        });
        let h = history(start.year(), start.month(), &values);
        let ahead = months_to_year_end(now).unwrap();
        let factor = bps.map(|b| {
            if b >= 10_000 {
                PaceFactor::up(b - 10_000).unwrap_or(PaceFactor::DOUBLE)
            } else {
                PaceFactor::down(10_000 - b).unwrap_or(PaceFactor::HALF)
            }
        });
        // A what-if may be refused: a pace below 0, or a scaled month above the largest amount.
        // Then the plain whole year must still be right.
        let (e, factor) = match estimate_to_year_end(&h, now, factor) {
            Ok(e) => (e, factor),
            Err(error) => {
                prop_assert!(factor.is_some(), "{error:?}");
                let plain = estimate(&h, now, ahead, None).unwrap();
                let pace = method(&plain, Method::SamePace).per_month.unwrap();
                if pace < Paisa::ZERO {
                    prop_assert_eq!(error, ForecastError::NegativePace);
                } else {
                    prop_assert_eq!(error, ForecastError::Figure(FigureError::Overflow));
                }
                (to_year_end(&h, now, None), None)
            }
        };
        let year = e.year.clone().unwrap();
        // So far is the sum of the history months of this year, counted by place.
        let expected: i64 = h.iter().filter(|x| x.month.year() == 2026).map(|x| x.value.rupees()).sum();
        let in_year = h.iter().filter(|x| x.month.year() == 2026).count();
        prop_assert_eq!(year.so_far.rupees(), expected);
        prop_assert_eq!(usize::try_from(year.months).unwrap(), in_year);
        prop_assert_eq!(in_year, std::cmp::min(values.len(), usize::try_from(current_month - 1).unwrap()));
        // The books' months and the months ahead never overlap and fill at most the year.
        prop_assert!(year.months + e.ahead <= 12);
        if values.len() >= usize::try_from(current_month - 1).unwrap() {
            prop_assert_eq!(year.months + e.ahead, 12);
        }
        prop_assert_eq!((e.first, e.last, e.ahead), (now, ym(2026, 12), ahead));
        let plain = estimate_with(&h, now, ahead, None, factor).unwrap();
        for m in &e.methods {
            let whole = m.year_total.unwrap();
            // Nothing but the year steps is different from the plain estimate to December.
            let base = plain.methods.iter().find(|p| p.method == m.method).unwrap();
            prop_assert_eq!(&m.months, &base.months);
            prop_assert_eq!(m.total, base.total);
            prop_assert_eq!(&m.steps[..base.steps.len()], &base.steps[..]);
            if year.months == 0 {
                prop_assert_eq!(whole, m.total);
                prop_assert_eq!(m.steps.len(), base.steps.len());
            } else {
                prop_assert_eq!(whole.rupees() - m.total.rupees(), year.so_far.rupees());
                let books_steps = usize::from(year.months >= 2);
                prop_assert_eq!(m.steps.len(), base.steps.len() + books_steps + 1);
                let last = m.steps.last().unwrap();
                prop_assert_eq!(last.op, Op::Sum);
                prop_assert_eq!(&last.inputs, &vec![Value::Money(year.so_far), Value::Money(m.total)]);
                prop_assert_eq!(last.result, Value::Money(whole));
            }
        }
        // A what-if: the books so far, then the scaled month times the months ahead.
        if let Some(w) = e.methods.iter().find(|m| m.method == Method::WhatIfPace) {
            let scaled = w.per_month.unwrap().rupees();
            prop_assert_eq!(
                w.year_total.unwrap().rupees(),
                year.so_far.rupees() + scaled * i64::from(e.ahead)
            );
        }
    }

    /// A flat series gives 12 times the month, whatever the month of the year.
    #[test]
    fn a_flat_series_gives_twelve_months_of_it(
        current_month in 4u32..=12,
        v in 0i64..=1_000_000_000,
    ) {
        let n = usize::try_from(current_month - 1).unwrap();
        let h = history(2026, 1, &vec![v; n]);
        let e = to_year_end(&h, ym(2026, current_month), None);
        prop_assert_eq!(year_total(&e, Method::SamePace), Some(12 * v));
        // Same growth needs 6 months of the books.
        if n >= 6 {
            prop_assert_eq!(year_total(&e, Method::SameGrowth), Some(12 * v));
        }
    }

    /// A rising straight line (a + b per month from January) has one same-growth whole year,
    /// 12a + 66b, whatever the month it is asked in.
    #[test]
    fn a_rising_line_has_one_whole_year_by_growth_in_every_month(
        current_month in 7u32..=12,
        a in 0i64..=1_000_000,
        b in 0i64..=100_000,
    ) {
        let n = usize::try_from(current_month - 1).unwrap();
        let values: Vec<i64> = (0..n).map(|i| a + b * i64::try_from(i).unwrap()).collect();
        let h = history(2026, 1, &values);
        let e = to_year_end(&h, ym(2026, current_month), None);
        prop_assert_eq!(year_total(&e, Method::SameGrowth), Some(12 * a + 66 * b));
    }
}
