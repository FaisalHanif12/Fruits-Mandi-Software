//! Splits under stress (PRD C10, C11, C17; money agent 5.1 to 5.3 and 11.2 items 1 and 2).
//! Rules: parts add back exactly; positive amounts give parts >= 0, negative amounts parts
//! <= 0; each part is within the leftover of its exact share; the leftover goes to the
//! largest weight, tie to the lowest id; no panic at the `i64` limits.

use mandi_core::money::{Bps, MoneyError, Paisa, Qty, split_by_bps, split_by_qty};

use super::{MAX_PAISA, bps, max_input, max_paisa, min_paisa, no_panic, paisa, qty, rs};

/// Checks the split rules for any result. `weights` are in the same order as `parts`.
fn check_split(amount: Paisa, weights: &[(u32, i128)], parts: &[(u32, Paisa)]) {
    assert_eq!(parts.len(), weights.len());
    let total_w: i128 = weights.iter().map(|w| w.1).sum();
    let mut sum: i128 = 0;
    let abs_rupees = i128::from(amount.rupees()).abs();
    for ((id_w, w), (id_p, p)) in weights.iter().zip(parts) {
        assert_eq!(id_w, id_p, "order changed");
        assert_eq!(p.as_paisa() % 100, 0, "part not a whole rupee");
        sum += i128::from(p.as_paisa());
        if amount.is_negative() {
            assert!(p.as_paisa() <= 0, "negative amount gave a positive part");
        } else {
            assert!(p.as_paisa() >= 0, "positive amount gave a negative part");
        }
        // Floor of the exact share; the winner may hold up to n-1 extra rupees.
        let floor = abs_rupees * w / total_w;
        let got = i128::from(p.rupees()).abs();
        assert!(got >= floor, "part below its floor share");
        assert!(
            got - floor < i128::try_from(weights.len()).unwrap().max(1),
            "leftover too big"
        );
    }
    assert_eq!(sum, i128::from(amount.as_paisa()), "parts do not add back");
}

fn ten_equal() -> Vec<(u32, Bps)> {
    (1..=10).map(|id| (id, bps(1_000))).collect()
}

fn as_weights(shares: &[(u32, Bps)]) -> Vec<(u32, i128)> {
    shares
        .iter()
        .map(|&(id, b)| (id, i128::from(b.get())))
        .collect()
}

fn edge_amounts() -> Vec<Paisa> {
    vec![
        Paisa::ZERO,
        rs(1),
        rs(-1),
        rs(9),
        rs(11),
        rs(-9),
        max_input(),
        max_paisa(),
        min_paisa(),
        paisa(MAX_PAISA - 900),
    ]
}

#[test]
fn ten_partners_at_extreme_amounts() {
    let uneven: Vec<(u32, Bps)> = vec![
        (7, bps(1)),
        (3, bps(1)),
        (9, bps(9_991)),
        (1, bps(1)),
        (2, bps(1)),
        (4, bps(1)),
        (5, bps(1)),
        (6, bps(1)),
        (8, bps(1)),
        (10, bps(1)),
    ];
    let with_zero: Vec<(u32, Bps)> = (1..=10)
        .map(|id| {
            (
                id,
                bps(if id == 10 {
                    1_000 * 2
                } else if id == 9 {
                    0
                } else {
                    1_000
                }),
            )
        })
        .collect();
    for shares in [ten_equal(), uneven, with_zero] {
        for amount in edge_amounts() {
            let parts = no_panic("split_by_bps", || split_by_bps(amount, &shares)).unwrap();
            check_split(amount, &as_weights(&shares), &parts);
        }
    }
}

#[test]
fn ten_equal_partners_leftover_goes_to_lowest_id() {
    // 9 rupees over 10 equal partners: every floor is 0; all 9 go to partner 1.
    let parts = split_by_bps(rs(9), &ten_equal()).unwrap();
    assert_eq!(parts.first(), Some(&(1, rs(9))));
    assert!(parts.iter().skip(1).all(|(_, p)| *p == Paisa::ZERO));
    // Negative: the same, negated.
    let parts = split_by_bps(rs(-9), &ten_equal()).unwrap();
    assert_eq!(parts.first(), Some(&(1, rs(-9))));
    // Ids given out of order: the lowest id still wins.
    let shuffled: Vec<(u32, Bps)> = [5, 3, 9, 1, 7, 2, 4, 6, 8, 10]
        .iter()
        .map(|&id| (id, bps(1_000)))
        .collect();
    let parts = split_by_bps(rs(9), &shuffled).unwrap();
    assert_eq!(
        parts.iter().find(|(id, _)| *id == 1).map(|p| p.1),
        Some(rs(9))
    );
}

#[test]
fn shares_must_total_exactly_100() {
    let mut nine = ten_equal();
    nine.pop();
    assert_eq!(split_by_bps(rs(100), &nine), Err(MoneyError::SharesNot100));
    let mut over = ten_equal();
    over.push((11, bps(1)));
    assert_eq!(split_by_bps(rs(100), &over), Err(MoneyError::SharesNot100));
    let none: [(u32, Bps); 0] = [];
    assert_eq!(split_by_bps(rs(100), &none), Err(MoneyError::SharesNot100));
    let mut dup = ten_equal();
    if let Some(last) = dup.last_mut() {
        last.0 = 1;
    }
    assert_eq!(split_by_bps(rs(100), &dup), Err(MoneyError::DuplicateId));
}

#[test]
fn ten_thousand_partners_of_one_bps() {
    // Far past NFR-SCL-5 (10 partners): still exact, no panic.
    let shares: Vec<(u32, Bps)> = (1..=10_000).map(|id| (id, bps(1))).collect();
    for amount in [rs(9_999), rs(10_000), max_input(), min_paisa()] {
        let parts = no_panic("10,000 partners", || split_by_bps(amount, &shares)).unwrap();
        check_split(amount, &as_weights(&shares), &parts);
    }
}

#[test]
fn split_by_qty_many_lots() {
    // 50 lots (NFR-SCL-3) and 10,000 lots, each up to 1,000,000 نگ.
    for n in [2_u32, 50, 10_000] {
        let lots: Vec<(u32, Qty)> = (1..=n)
            .map(|id| (id, qty(i64::from(id % 7) * 142_857 + 1)))
            .collect();
        let weights: Vec<(u32, i128)> = lots
            .iter()
            .map(|&(id, q)| (id, i128::from(q.get())))
            .collect();
        for amount in edge_amounts() {
            let parts = no_panic("split_by_qty", || split_by_qty(amount, &lots)).unwrap();
            check_split(amount, &weights, &parts);
        }
    }
}

#[test]
fn split_by_qty_extremes() {
    // Two lots of i64::MAX نگ: the weight total needs i128 and still works.
    let lots = [(1_u32, qty(i64::MAX)), (2, qty(i64::MAX))];
    let parts = split_by_qty(max_paisa(), &lots).unwrap();
    check_split(
        max_paisa(),
        &[(1, i128::from(i64::MAX)), (2, i128::from(i64::MAX))],
        &parts,
    );
    // A lot of 0 نگ next to a real lot takes nothing.
    let parts = split_by_qty(rs(1_000), &[(1, qty(0)), (2, qty(20))]).unwrap();
    assert_eq!(parts, vec![(1, Paisa::ZERO), (2, rs(1_000))]);
    // Only empty lots: no split.
    assert_eq!(
        split_by_qty(rs(1_000), &[(1, qty(0)), (2, qty(0))]),
        Err(MoneyError::DivisionByZero)
    );
    // C17 tie: equal نگ, the lowest lot number takes the leftover.
    let parts = split_by_qty(rs(1_001), &[(2, qty(10)), (1, qty(10))]).unwrap();
    assert_eq!(parts, vec![(2, rs(500)), (1, rs(501))]);
}
