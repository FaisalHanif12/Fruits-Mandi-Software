//! Property tests for money, each against a second, independent calculation (money agent 11.2
//! items 1, 2, 5, 6 and 9; Code Standards 9.3, M4, M5, M7, M9). 10,000 random cases each.
//!
//! The oracle in each test is written from the PRD formula in plain `i128` and never calls the
//! function under test, so a mistake in the real code cannot hide behind a mistake in the check.
//! Items 3, 4 (journal) are in `services/tests/robustness_journal.rs`. Items 7 and 8 (matching,
//! partner share rows over a sequence of postings) need D3 and D4, which are not built yet.

use mandi_core::money::Grams;
use mandi_core::money::{
    Bps, Grouping, MAX_RUPEES, Paisa, Qty, apply_bps, format_paisa, line_value, parse_rupees,
    split_by_bps, split_by_qty, weight_value,
};
use proptest::prelude::*;

/// Most rupees a `Paisa` holds.
const MAX_R: i64 = i64::MAX / 100;
/// The journal cap, 10^12 rupees: the bases real charges are worked on.
const BASE_CAP: i64 = 1_000_000_000_000;

fn config() -> ProptestConfig {
    ProptestConfig::with_cases(10_000)
}

fn rupees(r: i64) -> Paisa {
    Paisa::from_rupees(r).unwrap()
}

/// An amount in rupees: mostly ordinary, sometimes at a limit.
fn amount_rupees() -> impl Strategy<Value = i64> {
    prop_oneof![
        8 => -MAX_RUPEES..=MAX_RUPEES,
        1 => -MAX_R..=MAX_R,
        1 => prop_oneof![Just(0), Just(1), Just(-1), Just(MAX_R), Just(-MAX_R)],
    ]
}

/// Ids in a random order, so the "lowest id wins a tie" rule is not helped by position.
fn shuffled_ids(n: usize) -> impl Strategy<Value = Vec<u32>> {
    Just(
        (1..=u32::try_from(n).unwrap())
            .map(|i| i * 7 + 3)
            .collect::<Vec<u32>>(),
    )
    .prop_shuffle()
}

/// The reference split: floors, leftover to the largest weight, tie to the lowest id, sign last.
fn oracle_split(amount_r: i64, weights: &[(u32, i128)]) -> Vec<(u32, i128)> {
    let total: i128 = weights.iter().map(|w| w.1).sum();
    let abs = i128::from(amount_r).abs();
    let floors: Vec<i128> = weights.iter().map(|w| abs * w.1 / total).collect();
    let left = abs - floors.iter().sum::<i128>();
    // The winner: the largest weight, and among equals the lowest id.
    let winner = weights
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.1.cmp(&b.1.1).then(b.1.0.cmp(&a.1.0)))
        .map(|(i, _)| i)
        .unwrap();
    weights
        .iter()
        .enumerate()
        .map(|(i, &(id, _))| {
            let part = floors[i] + if i == winner { left } else { 0 };
            (id, if amount_r < 0 { -part } else { part } * 100)
        })
        .collect()
}

proptest! {
    #![proptest_config(config())]

    /// Item 1: split_by_bps equals the reference for any amount and any shares that total 100%.
    #[test]
    fn split_by_bps_equals_the_reference_and_adds_back(
        amount_r in amount_rupees(),
        n in 1_usize..=10,
        cuts in prop::collection::vec(0_i32..=10_000, 9),
        ids in (1_usize..=10).prop_flat_map(shuffled_ids),
    ) {
        // n shares from n-1 cut points, total exactly 10,000 (a share may be 0).
        let mut points: Vec<i32> = cuts.iter().copied().take(n - 1).collect();
        points.sort_unstable();
        let mut shares: Vec<(u32, Bps)> = Vec::new();
        let mut last = 0;
        for (i, p) in points.iter().copied().chain(std::iter::once(10_000)).enumerate() {
            shares.push((ids[i % ids.len()] + u32::try_from(i / ids.len()).unwrap() * 1000, Bps::new(p - last).unwrap()));
            last = p;
        }
        let amount = rupees(amount_r);
        let got = split_by_bps(amount, &shares).unwrap();
        let weights: Vec<(u32, i128)> = shares.iter().map(|&(id, b)| (id, i128::from(b.get()))).collect();
        let want = oracle_split(amount_r, &weights);
        let got_i128: Vec<(u32, i128)> = got.iter().map(|&(id, p)| (id, i128::from(p.as_paisa()))).collect();
        prop_assert_eq!(&got_i128, &want);
        // The rules in words: adds back exactly, every part whole rupees, signs follow the amount.
        let sum: i128 = got_i128.iter().map(|p| p.1).sum();
        prop_assert_eq!(sum, i128::from(amount.as_paisa()));
        for (_, part) in &got {
            prop_assert_eq!(part.as_paisa() % 100, 0);
            if amount_r >= 0 { prop_assert!(part.as_paisa() >= 0); } else { prop_assert!(part.as_paisa() <= 0); }
        }
    }

    /// Item 1 (PRD C11): a negative amount splits like the positive one, every part negated.
    #[test]
    fn a_negative_amount_splits_as_the_positive_one_negated(
        amount_r in 1_i64..=MAX_RUPEES,
        a in 0_i32..=10_000,
        b in 0_i32..=10_000,
    ) {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let shares = [
            (1_u32, Bps::new(lo).unwrap()),
            (2, Bps::new(hi - lo).unwrap()),
            (3, Bps::new(10_000 - hi).unwrap()),
        ];
        let plus = split_by_bps(rupees(amount_r), &shares).unwrap();
        let minus = split_by_bps(rupees(-amount_r), &shares).unwrap();
        for (p, m) in plus.iter().zip(&minus) {
            prop_assert_eq!(p.0, m.0);
            prop_assert_eq!(p.1.as_paisa(), -m.1.as_paisa());
        }
    }

    /// Item 2: split_by_qty equals the reference and adds back exactly.
    #[test]
    fn split_by_qty_equals_the_reference_and_adds_back(
        total_r in 0_i64..=MAX_RUPEES,
        qtys in prop::collection::vec(1_i64..=1_000_000, 1..=40),
    ) {
        let parts: Vec<(u32, Qty)> = qtys.iter().enumerate()
            .map(|(i, &q)| (u32::try_from(i).unwrap() + 1, Qty::new(q).unwrap()))
            .collect();
        let got = split_by_qty(rupees(total_r), &parts).unwrap();
        let weights: Vec<(u32, i128)> = qtys.iter().enumerate()
            .map(|(i, &q)| (u32::try_from(i).unwrap() + 1, i128::from(q))).collect();
        let want = oracle_split(total_r, &weights);
        let got_i128: Vec<(u32, i128)> = got.iter().map(|&(id, p)| (id, i128::from(p.as_paisa()))).collect();
        prop_assert_eq!(&got_i128, &want);
        prop_assert_eq!(got_i128.iter().map(|p| p.1).sum::<i128>(), i128::from(rupees(total_r).as_paisa()));
    }

    /// Item 5: text made from a value parses back to the same value, in both groupings.
    #[test]
    fn parse_of_format_is_the_same_money(r in 0_i64..=MAX_RUPEES) {
        for grouping in [Grouping::International, Grouping::Lakh] {
            let value = rupees(r);
            let text = format_paisa(value, grouping).unwrap();
            prop_assert_eq!(parse_rupees(&text), Ok(value), "{}", text);
        }
    }

    /// Item 6 and C2: a percent charge is the reference (half up), is never more than the base for
    /// 100% or less, and never goes down when the base goes up.
    #[test]
    fn a_percent_charge_is_the_reference_and_never_more_than_the_base(
        base_r in 0_i64..=BASE_CAP,
        grow in 0_i64..=1_000_000,
        bps in 0_i32..=10_000,
    ) {
        let charge = apply_bps(rupees(base_r), Bps::new(bps).unwrap()).unwrap();
        let want = (2 * i128::from(base_r) * i128::from(bps) + 10_000) / 20_000;
        prop_assert_eq!(i128::from(charge.rupees()), want);
        prop_assert!(charge.as_paisa() <= rupees(base_r).as_paisa());
        let bigger = apply_bps(rupees(base_r + grow), Bps::new(bps).unwrap()).unwrap();
        prop_assert!(bigger.as_paisa() >= charge.as_paisa());
    }

    /// C1: qty times rate is exact, or an overflow error. Never a wrapped number.
    #[test]
    fn line_value_is_exact_or_an_error(q in 1_i64..=1_000_000, r in 0_i64..=MAX_RUPEES) {
        let got = line_value(Qty::new(q).unwrap(), rupees(r));
        let want = i128::from(q) * i128::from(r);
        match got {
            Ok(v) => prop_assert_eq!(i128::from(v.rupees()), want),
            Err(_) => prop_assert!(want * 100 > i128::from(i64::MAX), "refused a value that fits"),
        }
    }

    /// C1 by weight: round(kg x rate), half up, in whole rupees.
    #[test]
    fn weight_value_is_the_reference(g in 1_i64..=100_000_000, rate_r in 0_i64..=MAX_RUPEES) {
        let got = weight_value(Grams::new(g).unwrap(), rupees(rate_r));
        let want = (2 * i128::from(g) * i128::from(rate_r) + 1_000) / 2_000;
        match got {
            Ok(v) => prop_assert_eq!(i128::from(v.rupees()), want),
            Err(_) => prop_assert!(want * 100 > i128::from(i64::MAX), "refused a value that fits"),
        }
    }

    /// Item 9: a Paisa exists only for a multiple of 100, and sums and differences of Paisa are
    /// multiples of 100 or an overflow error.
    #[test]
    fn every_money_value_is_a_multiple_of_100(p in any::<i64>(), q in any::<i64>()) {
        let made = Paisa::try_from_paisa(p);
        prop_assert_eq!(made.is_ok(), p % 100 == 0);
        if let (Ok(a), Ok(b)) = (made, Paisa::try_from_paisa(q)) {
            match a.checked_add(b) {
                Ok(s) => {
                    prop_assert_eq!(s.as_paisa() % 100, 0);
                    prop_assert_eq!(i128::from(s.as_paisa()), i128::from(p) + i128::from(q));
                }
                Err(_) => prop_assert!(i128::from(p) + i128::from(q) > i128::from(i64::MAX)
                    || i128::from(p) + i128::from(q) < i128::from(i64::MIN)),
            }
            match a.checked_sub(b) {
                Ok(s) => {
                    prop_assert_eq!(s.as_paisa() % 100, 0);
                    prop_assert_eq!(i128::from(s.as_paisa()), i128::from(p) - i128::from(q));
                }
                Err(_) => prop_assert!(i128::from(p) - i128::from(q) > i128::from(i64::MAX)
                    || i128::from(p) - i128::from(q) < i128::from(i64::MIN)),
            }
        }
    }
}
