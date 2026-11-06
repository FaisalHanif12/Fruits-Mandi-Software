//! A balance at the same pace (plan 55 F4 and F5, PRD FR-7.86, Code Standards M14, money agent
//! Section 9.1): the worked numbers of the forecast design note as unit tests, every row of its
//! rounding table, the refusals, the bounds, the working, and property tests. At the end: the
//! working of the same-growth estimate has a step for each projected month (robustness, FR-7.43,
//! B30).
//!
//! All amounts below are whole rupees; the code works in paisa.
#![cfg(test)]

use mandi_domain::core::money::{MAX_RUPEES, Paisa};
use mandi_domain::figures::{Op, Value};
use mandi_domain::forecast::{
    BALANCE_MONTHS_MAX, BalanceKind, ForecastError, Method, MonthValue, YearMonth, balance_at_pace,
    estimate,
};
use proptest::prelude::*;

fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

fn ym(year: i32, month: u32) -> YearMonth {
    YearMonth::new(year, month).unwrap()
}

/// Net changes from `(year, month)` on, oldest first, in rupees.
fn changes(year: i32, month: u32, rupees: &[i64]) -> Vec<MonthValue> {
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

/// The changes of the last `rupees.len()` months ending September 2026 (today is in October).
fn ending_september(rupees: &[i64]) -> Vec<MonthValue> {
    let n = i64::try_from(rupees.len()).unwrap();
    let september = 2026_i64 * 12 + 8;
    let start = september - (n - 1);
    changes(
        i32::try_from(start.div_euclid(12)).unwrap(),
        u32::try_from(start.rem_euclid(12)).unwrap() + 1,
        rupees,
    )
}

/// April to September 2026, so today is in October.
fn april_to_september(rupees: &[i64]) -> Vec<MonthValue> {
    changes(2026, 4, rupees)
}

fn october() -> YearMonth {
    ym(2026, 10)
}

fn rupees_of(list: &[Paisa]) -> Vec<i64> {
    list.iter().map(|p| p.rupees()).collect()
}

/// Fixture B, Akram: 137,400 owed, six changes of the bills less the receipts.
const AKRAM: [i64; 6] = [1_400, 10_000, 18_600, 27_200, 35_800, 44_400];

#[test]
fn akrams_balance_at_the_same_pace_is_35_800_a_month() {
    // Six months of change are given; only the last 3 (July to September) make the pace:
    // (27,200 + 35,800 + 44,400) / 3 = 35,800.
    let e = balance_at_pace(
        BalanceKind::Party,
        rs(137_400),
        &april_to_september(&AKRAM),
        october(),
        6,
    )
    .unwrap();
    assert_eq!(e.now, rs(137_400));
    assert_eq!(e.months_used, 3);
    assert_eq!((e.used_from, e.used_to), (ym(2026, 7), ym(2026, 9)));
    assert_eq!(e.per_month, rs(35_800));
    assert_eq!(e.change, rs(214_800));
    assert_eq!(
        rupees_of(&e.after),
        vec![173_200, 209_000, 244_800, 280_600, 316_400, 352_200]
    );
    assert_eq!(e.total, rs(352_200));
    // After 3 months is the third row.
    assert_eq!(e.after.get(2), Some(&rs(244_800)));
}

#[test]
fn the_rows_are_months_from_today_and_the_first_is_next_month() {
    let e = balance_at_pace(
        BalanceKind::Party,
        rs(137_400),
        &april_to_september(&AKRAM),
        october(),
        3,
    )
    .unwrap();
    assert_eq!((e.first, e.last), (ym(2026, 11), ym(2027, 1)));
    // From November the year turns after one more month.
    let nov = changes(2026, 5, &AKRAM);
    let e = balance_at_pace(BalanceKind::Party, rs(1), &nov, ym(2026, 11), 3).unwrap();
    assert_eq!((e.first, e.last), (ym(2026, 12), ym(2027, 2)));
}

#[test]
fn the_cash_worked_numbers() {
    let cash = [5_000, 7_000, 8_000, 10_000, 11_000, 13_000];
    let e = balance_at_pace(
        BalanceKind::Cash,
        rs(54_000),
        &april_to_september(&cash),
        october(),
        6,
    )
    .unwrap();
    // (10,000 + 11,000 + 13,000) / 3 = 11,333.33, which is 11,333 a month.
    assert_eq!(e.per_month, rs(11_333));
    assert_eq!(
        rupees_of(&e.after),
        vec![65_333, 76_666, 87_999, 99_332, 110_665, 121_998]
    );
}

#[test]
fn the_rounding_table_of_the_design_note() {
    // (now, changes, per month, after 1, after 3)
    let rows: [(i64, [i64; 3], i64, i64, i64); 6] = [
        (1_000, [100, 100, 102], 101, 1_101, 1_303), // 100.67 rounds up
        (1_000, [100, 100, 101], 100, 1_100, 1_300), // 100.33 rounds down
        (1_000, [-100, -100, -102], -101, 899, 697), // half up on the size, then the sign
        (100, [1, 2, 0], 1, 101, 103),               // exact
        (
            50_000,
            [-30_000, -30_000, -30_000],
            -30_000,
            20_000,
            -40_000,
        ),
        (0, [0, 0, 0], 0, 0, 0),
    ];
    for (now, list, per, one, three) in rows {
        let e = balance_at_pace(
            BalanceKind::Party,
            rs(now),
            &ending_september(&list),
            october(),
            3,
        )
        .unwrap();
        assert_eq!(e.per_month, rs(per), "per month of {list:?}");
        assert_eq!(e.after.first(), Some(&rs(one)), "after 1 of {list:?}");
        assert_eq!(e.after.get(2), Some(&rs(three)), "after 3 of {list:?}");
    }
}

#[test]
fn with_three_months_the_average_is_the_nearer_rupee_on_both_sides() {
    // A division by 3 leaves a third or two thirds, never a half: 2/3 rounds up, 1/3 rounds down,
    // and the opposite changes give the opposite pace (half away from zero is never needed).
    let pace = |list: [i64; 3]| {
        balance_at_pace(
            BalanceKind::Cash,
            rs(0),
            &ending_september(&list),
            october(),
            1,
        )
        .unwrap()
        .per_month
        .rupees()
    };
    assert_eq!(pace([1, 1, 0]), 1);
    assert_eq!(pace([1, 0, 0]), 0);
    assert_eq!(pace([-1, -1, 0]), -1);
    assert_eq!(pace([-1, 0, 0]), 0);
    assert_eq!(pace([5, 5, 6]), 5); // 5.33
    assert_eq!(pace([5, 6, 6]), 6); // 5.67
}

#[test]
fn a_party_can_change_side_and_cash_can_go_below_zero() {
    // July to September: the balance falls by 30,000 a month.
    let three = ending_september(&[-30_000, -30_000, -30_000]);
    let party = balance_at_pace(BalanceKind::Party, rs(50_000), &three, october(), 3).unwrap();
    assert_eq!(rupees_of(&party.after), vec![20_000, -10_000, -40_000]);
    // The working says it with balances: the last step is a sum of two balances.
    let last = party.steps.last().unwrap();
    assert_eq!(last.op, Op::Sum);
    assert_eq!(last.result, Value::Balance(rs(-40_000)));
    // Cash goes below zero the same way, as plain amounts.
    let cash = balance_at_pace(BalanceKind::Cash, rs(50_000), &three, october(), 3).unwrap();
    assert_eq!(rupees_of(&cash.after), vec![20_000, -10_000, -40_000]);
    assert_eq!(cash.steps.last().unwrap().result, Value::Money(rs(-40_000)));
}

#[test]
fn the_dues_never_go_below_zero() {
    let three = ending_september(&[-30_000, -30_000, -30_000]);
    // 50,000 less 30,000 a month: 20,000 after 1 month, then below 0.
    let ok = balance_at_pace(BalanceKind::Dues, rs(50_000), &three, october(), 1).unwrap();
    assert_eq!(ok.total, rs(20_000));
    assert_eq!(
        balance_at_pace(BalanceKind::Dues, rs(50_000), &three, october(), 2).unwrap_err(),
        ForecastError::BelowZero
    );
    // Exactly 0 is allowed.
    let zero = balance_at_pace(BalanceKind::Dues, rs(60_000), &three, october(), 2).unwrap();
    assert_eq!(zero.total, Paisa::ZERO);
}

#[test]
fn the_last_three_months_are_averaged_and_no_more() {
    // 24 months: the first 21 are 1,000 each, the last 3 are 3,000, 6,000 and 9,000.
    let mut all = vec![1_000; 21];
    all.extend([3_000, 6_000, 9_000]);
    let e = balance_at_pace(
        BalanceKind::Cash,
        rs(0),
        &changes(2024, 10, &all),
        october(),
        2,
    )
    .unwrap();
    assert_eq!(usize::try_from(e.months_used).unwrap(), BALANCE_MONTHS_MAX);
    assert_eq!(e.months_used, 3);
    assert_eq!((e.used_from, e.used_to), (ym(2026, 7), ym(2026, 9)));
    assert_eq!(e.per_month, rs(6_000));
}

#[test]
fn fewer_than_three_months_is_no_estimate() {
    for n in 0..3_usize {
        let history = ending_september(&vec![1_000; n]);
        let err = balance_at_pace(BalanceKind::Party, rs(5), &history, october(), 3).unwrap_err();
        assert_eq!(
            err,
            ForecastError::TooLittleHistory {
                months: u32::try_from(n).unwrap()
            },
            "{n} months"
        );
    }
}

#[test]
fn bad_inputs_are_refused() {
    let good = april_to_september(&AKRAM);
    let party = |h: &[MonthValue], ahead| {
        balance_at_pace(BalanceKind::Party, rs(1_000), h, october(), ahead)
    };
    assert_eq!(party(&good, 0).unwrap_err(), ForecastError::AheadOutOfRange);
    assert_eq!(
        party(&good, 13).unwrap_err(),
        ForecastError::AheadOutOfRange
    );
    assert!(party(&good, 12).is_ok());
    // A gap in the months, and a history that does not end the month before today's.
    let mut gap = good.clone();
    gap.remove(2);
    assert_eq!(
        party(&gap, 3).unwrap_err(),
        ForecastError::HistoryNotConsecutive
    );
    let late = changes(2026, 3, &AKRAM);
    assert_eq!(
        party(&late, 3).unwrap_err(),
        ForecastError::HistoryNotConsecutive
    );
}

#[test]
fn nothing_beyond_the_largest_amount_is_shown_in_size() {
    let big = MAX_RUPEES;
    let h = april_to_september(&[0, 0, 0, 0, 0, 0]);
    // The largest allowed debit or credit now is fine; one rupee more is refused.
    assert!(balance_at_pace(BalanceKind::Party, rs(big), &h, october(), 1).is_ok());
    assert!(balance_at_pace(BalanceKind::Party, rs(-big), &h, october(), 1).is_ok());
    for bad in [big + 1, -(big + 1)] {
        assert!(
            balance_at_pace(BalanceKind::Party, rs(bad), &h, october(), 1).is_err(),
            "{bad}"
        );
    }
    // A change that takes the balance beyond the limit, in either direction, is refused.
    for step in [big / 2, -(big / 2)] {
        let moving = april_to_september(&[step; 6]);
        let now = rs(step.signum().checked_mul(big - 1).unwrap());
        assert!(
            balance_at_pace(BalanceKind::Party, now, &moving, october(), 1).is_err(),
            "{step}"
        );
    }
}

#[test]
fn the_working_is_the_figures_shown() {
    let e = balance_at_pace(
        BalanceKind::Party,
        rs(137_400),
        &april_to_september(&AKRAM),
        october(),
        6,
    )
    .unwrap();
    assert_eq!(e.steps.len(), 3);
    // 1. The average of the last three changes is the per-month figure.
    let average = e.steps.first().unwrap();
    assert_eq!(average.op, Op::Average);
    assert_eq!(average.inputs.len(), 3);
    // A party's changes are balances, so the working says a side and never a minus sign.
    assert!(
        average
            .inputs
            .iter()
            .all(|v| matches!(v, Value::Balance(_)))
    );
    assert_eq!(average.result, Value::Balance(e.per_month));
    // 2. Times the months: the change.
    let times = e.steps.get(1).unwrap();
    assert_eq!(times.op, Op::Times);
    assert_eq!(
        times.inputs,
        vec![Value::Balance(e.per_month), Value::Count(6)]
    );
    assert_eq!(times.result, Value::Balance(e.change));
    // 3. The balance now plus the change is the total.
    let sum = e.steps.get(2).unwrap();
    assert_eq!(sum.op, Op::Sum);
    assert_eq!(
        sum.inputs,
        vec![Value::Balance(e.now), Value::Balance(e.change)]
    );
    assert_eq!(sum.result, Value::Balance(e.total));
}

/// The sizes of the properties: up to a million rupees of change and balance.
fn amounts() -> impl Strategy<Value = i64> {
    -1_000_000_i64..=1_000_000
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Every row is the balance now plus the pace times the months; the last row is the total, and
    /// the total is the balance now plus the change.
    #[test]
    fn the_rows_agree_with_the_total(
        now in amounts(),
        list in proptest::collection::vec(amounts(), 3..=24),
        ahead in 1_u32..=12,
        party in any::<bool>(),
    ) {
        let kind = if party { BalanceKind::Party } else { BalanceKind::Cash };
        let history = ending_september(&list);
        let e = balance_at_pace(kind, rs(now), &history, october(), ahead).unwrap();
        prop_assert_eq!(e.after.len(), usize::try_from(ahead).unwrap());
        for (j, row) in e.after.iter().enumerate() {
            let j = i64::try_from(j).unwrap() + 1;
            prop_assert_eq!(row.as_paisa(), rs(now).as_paisa() + e.per_month.as_paisa() * j);
        }
        prop_assert_eq!(e.after.last().copied(), Some(e.total));
        prop_assert_eq!(e.total.as_paisa(), rs(now).as_paisa() + e.change.as_paisa());
        prop_assert_eq!(e.change.as_paisa(), e.per_month.as_paisa() * i64::from(ahead));
    }

    /// The pace is the average of the last 3 months to the nearest rupee: it is never more than half
    /// a rupee away from the exact average, and it is the same size for the opposite changes.
    #[test]
    fn the_pace_is_the_average_to_the_nearest_rupee_and_symmetric(
        list in proptest::collection::vec(amounts(), 3..=12),
    ) {
        let up = ending_september(&list);
        let negated: Vec<i64> = list.iter().map(|v| -v).collect();
        let down = ending_september(&negated);
        let a = balance_at_pace(BalanceKind::Cash, rs(0), &up, october(), 1).unwrap();
        let b = balance_at_pace(BalanceKind::Cash, rs(0), &down, october(), 1).unwrap();
        // Only the last 3 months count.
        let sum: i64 = list.iter().rev().take(3).sum();
        // |sum - 3 * pace| <= 3 / 2 rupees, written without a fraction.
        prop_assert!(2 * (sum - 3 * a.per_month.rupees()).abs() <= 3);
        prop_assert_eq!(a.per_month.as_paisa(), -b.per_month.as_paisa());
    }
}

// ---- the working of same growth: a step for each projected month ---------------------------

fn growth_steps(
    history: &[MonthValue],
    ahead: u32,
) -> (Vec<mandi_domain::figures::Step>, Vec<i64>) {
    let e = estimate(history, october(), ahead, None).unwrap();
    let growth = e
        .methods
        .iter()
        .find(|m| m.method == Method::SameGrowth)
        .unwrap();
    (growth.steps.clone(), rupees_of(&growth.months))
}

fn m(rupees: i64) -> Value {
    Value::Money(rs(rupees))
}

#[test]
fn the_months_of_same_growth_have_the_steps_that_produce_them() {
    // The case robustness found: a change of 2,766,200 a month on 90,502,400. The sum of two
    // projected months (93,268,600 and 96,034,800) needs the step that makes each of them.
    let h = april_to_september(&[
        76_671_400, 79_437_600, 82_203_800, 84_970_000, 87_736_200, 90_502_400,
    ]);
    let (steps, months) = growth_steps(&h, 2);
    assert_eq!(months, vec![93_268_600, 96_034_800]);
    let ops: Vec<Op> = steps.iter().map(|s| s.op).collect();
    assert_eq!(
        ops,
        vec![Op::Difference, Op::DividedBy, Op::Sum, Op::Sum, Op::Sum]
    );
    // "last month + 2,766,200 a month", then "that + 2,766,200 a month", then the sum.
    assert_eq!(steps[2].inputs, vec![m(90_502_400), m(2_766_200)]);
    assert_eq!(steps[2].result, m(93_268_600));
    assert_eq!(steps[3].inputs, vec![m(93_268_600), m(2_766_200)]);
    assert_eq!(steps[3].result, m(96_034_800));
    assert_eq!(steps[4].inputs, vec![m(93_268_600), m(96_034_800)]);
    assert_eq!(steps[4].result, m(189_303_400));
}

#[test]
fn a_month_that_would_be_below_zero_shows_its_raw_figure_and_is_zero_in_the_sum() {
    // Falling by 3,000 a month from 5,000: 2,000, then -1,000 and -4,000 raw (so 0 and 0 in the sum).
    let h = april_to_september(&[20_000, 17_000, 14_000, 11_000, 8_000, 5_000]);
    let (steps, months) = growth_steps(&h, 3);
    assert_eq!(months, vec![2_000, 0, 0]);
    let ops: Vec<Op> = steps.iter().map(|s| s.op).collect();
    assert_eq!(
        ops,
        vec![
            Op::Difference,
            Op::DividedBy,
            Op::Sum,
            Op::Sum,
            Op::Sum,
            Op::Sum
        ]
    );
    assert_eq!(steps[2].inputs, vec![m(5_000), m(-3_000)]);
    assert_eq!(steps[2].result, m(2_000));
    assert_eq!(steps[3].inputs, vec![m(2_000), m(-3_000)]);
    assert_eq!(steps[3].result, m(-1_000));
    assert_eq!(steps[4].inputs, vec![m(-1_000), m(-3_000)]);
    assert_eq!(steps[4].result, m(-4_000));
    // The sum adds the months as shown: the floored ones are 0.
    assert_eq!(steps[5].inputs, vec![m(2_000), m(0), m(0)]);
    assert_eq!(steps[5].result, m(2_000));
    // Exactly 0 is a month, with its step.
    let h = april_to_september(&[18_000, 15_000, 12_000, 9_000, 6_000, 3_000]);
    let (steps, months) = growth_steps(&h, 2);
    assert_eq!(months, vec![0, 0]);
    assert_eq!(steps[2].inputs, vec![m(3_000), m(-3_000)]);
    assert_eq!(steps[2].result, m(0));
}

#[test]
fn a_loss_that_comes_back_above_zero_is_explained_step_by_step() {
    // Robustness and the money review: the last month is -1,000 and the change is +300 a month, so
    // the months are 0, 0, 0, 200, 500 (raw -700, -400, -100, 200, 500). The 200 and the 500 need
    // the steps that make them, and those start from the raw -100.
    let h = april_to_september(&[-2_500, -2_200, -1_900, -1_600, -1_300, -1_000]);
    let (steps, months) = growth_steps(&h, 5);
    assert_eq!(months, vec![0, 0, 0, 200, 500]);
    let results: Vec<Value> = steps.iter().skip(2).take(5).map(|s| s.result).collect();
    assert_eq!(results, vec![m(-700), m(-400), m(-100), m(200), m(500)]);
    assert_eq!(steps[5].inputs, vec![m(-100), m(300)]);
    assert_eq!(steps[6].inputs, vec![m(200), m(300)]);
    assert_eq!(steps.last().unwrap().result, m(700));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// Every projected month of same growth has a step that produces it from the month before (the
    /// first from the last month of the books), on the raw chain; the month shown is that figure
    /// or 0 when it is below 0; the last step sums the months shown. Losses included.
    #[test]
    fn the_steps_of_same_growth_chain_to_the_months_shown(
        list in proptest::collection::vec(-1_000_000_i64..=1_000_000, 6..=12),
        ahead in 1_u32..=12,
    ) {
        let h = ending_september(&list);
        let (steps, months) = growth_steps(&h, ahead);
        let (head, tail) = steps.split_at(2);
        prop_assert_eq!(head.len(), 2);
        let (month_steps, total) = tail.split_at(tail.len() - 1);
        prop_assert_eq!(month_steps.len(), months.len());
        let mut previous = *list.last().unwrap();
        for (k, step) in month_steps.iter().enumerate() {
            prop_assert_eq!(step.op, Op::Sum);
            prop_assert_eq!(step.inputs.first().copied(), Some(m(previous)));
            // The month shown is the result of its step, or 0 when that is below 0.
            let raw = match step.result {
                Value::Money(p) => p.rupees(),
                other => panic!("{other:?}"),
            };
            prop_assert_eq!(months[k], raw.max(0));
            previous = raw;
        }
        let shown: Vec<Value> = months.iter().map(|v| m(*v)).collect();
        prop_assert_eq!(total.first().map(|t| t.inputs.clone()), Some(shown));
    }
}
