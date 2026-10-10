//! Money maths at the edges: 0, 1 rupee, the input maximum and the `i64` limits.
//! Rule (Code Standards M7, money agent 11.4): the exact answer or an error, never a wrapped
//! number, never a panic. Every `Ok` result is checked against an `i128` reference.

use mandi_core::money::{
    Bps, Grams, MAX_QTY, MAX_RUPEES, MAX_WEIGHT_GRAMS, MoneyError, Paisa, Qty, apply_bps, avg_rate,
    line_value, per_unit, weight_value,
};

use super::{MAX_PAISA, MIN_PAISA, bps, max_input, max_paisa, min_paisa, no_panic, paisa, qty, rs};

/// Amounts that sit on an edge.
fn edge_amounts() -> Vec<Paisa> {
    vec![
        Paisa::ZERO,
        rs(1),
        rs(-1),
        max_input(),
        rs(MAX_RUPEES + 1),
        paisa(MAX_PAISA),
        paisa(MAX_PAISA - 100),
        paisa(MIN_PAISA),
        paisa(MIN_PAISA + 100),
    ]
}

fn edge_qtys() -> Vec<Qty> {
    vec![
        qty(0),
        qty(1),
        qty(MAX_QTY),
        qty(i64::MAX / 2),
        qty(i64::MAX),
    ]
}

fn edge_bps() -> Vec<Bps> {
    vec![
        bps(0),
        bps(1),
        bps(4_999),
        bps(5_000),
        bps(9_999),
        bps(10_000),
    ]
}

/// Reference: nearest rupee, half up, of `num / den` paisa, in `i128`. `None` if negative
/// or if the rupees do not fit an `i64` paisa value.
fn reference_round(num: i128, den: i128) -> Option<i64> {
    if num < 0 {
        return None;
    }
    let rupees = (num + 50 * den) / (100 * den);
    i64::try_from(rupees.checked_mul(100)?).ok()
}

#[test]
fn paisa_constructors_at_i64_limits() {
    assert_eq!(Paisa::from_rupees(i64::MAX), Err(MoneyError::Overflow));
    assert_eq!(Paisa::from_rupees(i64::MIN), Err(MoneyError::Overflow));
    assert_eq!(
        Paisa::from_rupees(i64::MAX / 100 + 1),
        Err(MoneyError::Overflow)
    );
    assert_eq!(
        Paisa::from_rupees(i64::MIN / 100 - 1),
        Err(MoneyError::Overflow)
    );
    assert_eq!(
        Paisa::from_rupees(i64::MAX / 100).unwrap().as_paisa(),
        MAX_PAISA
    );
    assert_eq!(
        Paisa::from_rupees(i64::MIN / 100).unwrap().as_paisa(),
        MIN_PAISA
    );
    for bad in [
        i64::MAX,
        i64::MIN,
        1,
        -1,
        50,
        -50,
        99,
        MAX_PAISA + 1,
        MIN_PAISA - 1,
    ] {
        assert_eq!(
            Paisa::try_from_paisa(bad),
            Err(MoneyError::NotWholeRupee),
            "{bad}"
        );
    }
    assert_eq!(max_paisa().rupees(), i64::MAX / 100);
    assert_eq!(min_paisa().rupees(), i64::MIN / 100);
}

#[test]
fn add_sub_neg_never_wrap() {
    let one = rs(1);
    assert_eq!(max_paisa().checked_add(one), Err(MoneyError::Overflow));
    assert_eq!(min_paisa().checked_sub(one), Err(MoneyError::Overflow));
    assert_eq!(
        min_paisa().checked_add(min_paisa()),
        Err(MoneyError::Overflow)
    );
    assert_eq!(
        max_paisa().checked_sub(min_paisa()),
        Err(MoneyError::Overflow)
    );
    assert_eq!(max_paisa().checked_add(min_paisa()), Ok(Paisa::ZERO));
    assert_eq!(max_paisa().checked_neg(), Ok(min_paisa()));
    assert_eq!(min_paisa().checked_neg(), Ok(max_paisa()));
    assert_eq!(Paisa::ZERO.checked_neg(), Ok(Paisa::ZERO));
}

#[test]
fn sums_of_many_max_inputs_stop_with_overflow() {
    // A running total of quarter-of-max amounts: 4 fit, the 5th is Overflow, not a wrap.
    let mut total = Paisa::ZERO;
    let mut steps: i64 = 0;
    let big = paisa(MAX_PAISA / 4 - (MAX_PAISA / 4) % 100);
    let result = loop {
        match total.checked_add(big) {
            Ok(t) => {
                total = t;
                steps += 1;
            }
            Err(e) => break e,
        }
        assert!(steps < 10, "never overflowed");
    };
    assert_eq!(result, MoneyError::Overflow);
    assert_eq!(steps, 4);
    assert_eq!(total.as_paisa() % 100, 0);
}

#[test]
fn json_never_makes_a_bad_paisa() {
    // Paisa arrives from JSON in stored data and tests. Only whole-rupee integers pass.
    let ok: Paisa = serde_json::from_str(&MAX_PAISA.to_string()).unwrap();
    assert_eq!(ok, max_paisa());
    for bad in [
        "150",
        "-1",
        "9223372036854775807",
        "-9223372036854775808",
        "9223372036854775900",
        "100.0",
        "1e2",
        "\"100\"",
        "null",
        "true",
        "[]",
        "NaN",
    ] {
        assert!(serde_json::from_str::<Paisa>(bad).is_err(), "{bad}");
    }
    assert_eq!(
        serde_json::to_string(&max_input()).unwrap(),
        "1000000000000"
    );
}

#[test]
fn bps_qty_grams_reject_out_of_range() {
    for bad in [-1, 10_001, i32::MIN, i32::MAX] {
        assert_eq!(Bps::new(bad), Err(MoneyError::BpsOutOfRange), "{bad}");
    }
    assert_eq!(Qty::new(-1), Err(MoneyError::NegativeValue));
    assert_eq!(Qty::new(i64::MIN), Err(MoneyError::NegativeValue));
    assert_eq!(Grams::new(-1), Err(MoneyError::NegativeValue));
    assert_eq!(Grams::new(i64::MIN), Err(MoneyError::NegativeValue));
}

#[test]
fn line_value_and_per_unit_exact_or_overflow() {
    for amount in edge_amounts() {
        for q in edge_qtys() {
            for f in [line_value, per_unit] {
                let got = no_panic("qty x rate", || f(q, amount));
                let exact = i128::from(q.get()) * i128::from(amount.as_paisa());
                match got {
                    Ok(p) => assert_eq!(i128::from(p.as_paisa()), exact, "{q:?} x {amount:?}"),
                    Err(MoneyError::NegativeBase) => assert!(amount.is_negative()),
                    Err(MoneyError::Overflow) => {
                        assert!(i64::try_from(exact).is_err(), "{q:?} x {amount:?}");
                    }
                    Err(e) => panic!("unexpected {e:?} for {q:?} x {amount:?}"),
                }
            }
        }
    }
}

#[test]
fn largest_allowed_line_fits() {
    // 1,000,000 نگ at 10,00,00,00,000 per نگ: 10^16 rupees. Fits i64 paisa (10^18).
    let v = line_value(qty(MAX_QTY), max_input()).unwrap();
    assert_eq!(v.rupees(), MAX_QTY * MAX_RUPEES);
    let w = weight_value(Grams::new(MAX_WEIGHT_GRAMS).unwrap(), max_input()).unwrap();
    assert_eq!(w.rupees(), 100_000 * MAX_RUPEES);
}

#[test]
fn apply_bps_exact_or_error_on_every_edge() {
    for amount in edge_amounts() {
        for b in edge_bps() {
            let got = no_panic("apply_bps", || apply_bps(amount, b));
            let num = i128::from(amount.as_paisa()) * i128::from(b.get());
            match got {
                Ok(p) => {
                    assert_eq!(
                        Some(p.as_paisa()),
                        reference_round(num, 10_000),
                        "{amount:?} {b:?}"
                    );
                    assert!(p <= amount, "charge above base: {amount:?} {b:?}");
                }
                Err(MoneyError::NegativeBase) => assert!(amount.is_negative()),
                Err(e) => panic!("unexpected {e:?} for {amount:?} {b:?}"),
            }
        }
    }
    // Full 100% of the largest paisa value is the value itself.
    assert_eq!(apply_bps(max_paisa(), Bps::FULL), Ok(max_paisa()));
}

#[test]
fn apply_bps_tiny_amounts_round_half_up() {
    // 1 rupee at 50% = 0.5 → 1; at 49.99% → 0; 0.01% of 1 rupee → 0.
    assert_eq!(apply_bps(rs(1), bps(5_000)), Ok(rs(1)));
    assert_eq!(apply_bps(rs(1), bps(4_999)), Ok(Paisa::ZERO));
    assert_eq!(apply_bps(rs(1), bps(1)), Ok(Paisa::ZERO));
    // 5,000 rupees at 0.01% = 0.5 → 1.
    assert_eq!(apply_bps(rs(5_000), bps(1)), Ok(rs(1)));
    assert_eq!(apply_bps(rs(4_999), bps(1)), Ok(Paisa::ZERO));
}

#[test]
fn weight_value_exact_or_error() {
    let grams = [0, 1, 499, 500, 999, 1_000, MAX_WEIGHT_GRAMS, i64::MAX];
    for g in grams {
        for amount in edge_amounts() {
            let w = Grams::new(g).unwrap();
            let got = no_panic("weight_value", || weight_value(w, amount));
            let num = i128::from(g) * i128::from(amount.as_paisa());
            match got {
                Ok(p) => assert_eq!(Some(p.as_paisa()), reference_round(num, 1_000), "{g} g"),
                Err(MoneyError::NegativeBase) => assert!(amount.is_negative()),
                Err(MoneyError::Overflow) => assert!(reference_round(num, 1_000).is_none()),
                Err(e) => panic!("unexpected {e:?} for {g} g at {amount:?}"),
            }
        }
    }
    // 500 g at 1 rupee per kg = 0.5 rupee → 1 (half up); 499 g → 0.
    assert_eq!(weight_value(Grams::new(500).unwrap(), rs(1)), Ok(rs(1)));
    assert_eq!(
        weight_value(Grams::new(499).unwrap(), rs(1)),
        Ok(Paisa::ZERO)
    );
}

#[test]
fn avg_rate_edges() {
    assert_eq!(
        avg_rate(rs(29_400), qty(0)),
        Err(MoneyError::DivisionByZero)
    );
    assert_eq!(avg_rate(rs(-1), qty(1)), Err(MoneyError::NegativeBase));
    assert_eq!(avg_rate(max_paisa(), qty(1)), Ok(max_paisa()));
    assert_eq!(avg_rate(max_paisa(), qty(i64::MAX)), Ok(Paisa::ZERO));
    assert_eq!(avg_rate(Paisa::ZERO, qty(i64::MAX)), Ok(Paisa::ZERO));
    // 3 rupees over 2 نگ = 1.5 → 2 (half up).
    assert_eq!(avg_rate(rs(3), qty(2)), Ok(rs(2)));
}
