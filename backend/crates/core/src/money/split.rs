//! Splits that always add back exactly to the total (PRD C10, C11, C17; Code Standards M9;
//! money agent Section 5).
//!
//! Each part is the floor of its exact share in whole rupees. The leftover rupees go to the
//! part with the largest weight (largest share or most نگ); on a tie, to the smallest id
//! (the partner created first, or the lowest lot number). A negative amount is split as
//! positive, then every part is made negative.

use super::{BPS_FULL, Bps, MoneyError, Paisa, Qty};

/// Splits income or an expense between partners by their shares (PRD C10, C11).
///
/// The shares must total exactly 10,000 bps (100%), else `SharesNot100`.
/// `Id` is the partner id; a lower id means created first.
///
/// Example (PRD 4.4): 3,948 at 40/30/30 → 1,579 / 1,184 / 1,184, leftover 1 to the 40%
/// partner → 1,580 / 1,184 / 1,184.
pub fn split_by_bps<Id: Copy + Ord>(
    amount: Paisa,
    shares: &[(Id, Bps)],
) -> Result<Vec<(Id, Paisa)>, MoneyError> {
    let weights: Vec<(Id, i128)> = shares
        .iter()
        .map(|&(id, bps)| (id, i128::from(bps.get())))
        .collect();
    let total = weights
        .iter()
        .try_fold(0i128, |acc, &(_, w)| acc.checked_add(w))
        .ok_or(MoneyError::Overflow)?;
    if total != i128::from(BPS_FULL) {
        return Err(MoneyError::SharesNot100);
    }
    split_weighted(amount, &weights, total)
}

/// Splits a header freight total across lots by نگ received (PRD C17).
///
/// `Id` is the lot number; on a tie in نگ the lowest lot number gets the leftover.
///
/// Example: 1,000 over lots of 30 and 20 نگ → 600 / 400.
pub fn split_by_qty<Id: Copy + Ord>(
    amount: Paisa,
    parts: &[(Id, Qty)],
) -> Result<Vec<(Id, Paisa)>, MoneyError> {
    let weights: Vec<(Id, i128)> = parts
        .iter()
        .map(|&(id, qty)| (id, i128::from(qty.get())))
        .collect();
    let total = weights
        .iter()
        .try_fold(0i128, |acc, &(_, w)| acc.checked_add(w))
        .ok_or(MoneyError::Overflow)?;
    if total == 0 {
        return Err(MoneyError::DivisionByZero);
    }
    split_weighted(amount, &weights, total)
}

/// The one split algorithm. Weights are 0 or more and total more than 0.
fn split_weighted<Id: Copy + Ord>(
    amount: Paisa,
    weights: &[(Id, i128)],
    total: i128,
) -> Result<Vec<(Id, Paisa)>, MoneyError> {
    let winner = winner_index(weights)?;
    let negative = amount.is_negative();
    let abs_rupees = i128::from(amount.rupees())
        .checked_abs()
        .ok_or(MoneyError::Overflow)?;

    // Floor part of every partner but the winner, in whole rupees.
    let mut parts: Vec<i128> = Vec::with_capacity(weights.len());
    for &(_, weight) in weights {
        let part = abs_rupees
            .checked_mul(weight)
            .and_then(|p| p.checked_div(total))
            .ok_or(MoneyError::Overflow)?;
        parts.push(part);
    }
    // The winner takes the rest, which is its floor part plus every leftover rupee,
    // so the parts always add back exactly to the amount.
    let others = parts
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != winner)
        .try_fold(0i128, |acc, (_, &p)| acc.checked_add(p))
        .ok_or(MoneyError::Overflow)?;
    let winner_part = abs_rupees.checked_sub(others).ok_or(MoneyError::Overflow)?;
    if let Some(slot) = parts.get_mut(winner) {
        *slot = winner_part;
    }

    weights
        .iter()
        .zip(parts)
        .map(|(&(id, _), rupees)| {
            let rupees = i64::try_from(rupees).map_err(|_| MoneyError::Overflow)?;
            let part = Paisa::from_rupees(rupees)?;
            let part = if negative { part.checked_neg()? } else { part };
            Ok((id, part))
        })
        .collect()
}

/// Index of the part that takes the leftover: largest weight, tie → smallest id.
/// Fails on an empty list or a repeated id.
fn winner_index<Id: Copy + Ord>(weights: &[(Id, i128)]) -> Result<usize, MoneyError> {
    let mut ids: Vec<Id> = weights.iter().map(|&(id, _)| id).collect();
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair.first() == pair.last()) {
        return Err(MoneyError::DuplicateId);
    }
    weights
        .iter()
        .enumerate()
        .max_by(|(_, (id_a, w_a)), (_, (id_b, w_b))| w_a.cmp(w_b).then(id_b.cmp(id_a)))
        .map(|(i, _)| i)
        .ok_or(MoneyError::DivisionByZero)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rupees(r: i64) -> Paisa {
        Paisa::from_rupees(r).unwrap()
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let b = |v| Bps::new(v).unwrap();
        let shares = [(1, b(5_000)), (1, b(5_000))];
        assert_eq!(
            split_by_bps(rupees(100), &shares),
            Err(MoneyError::DuplicateId)
        );
        let q = |v| Qty::new(v).unwrap();
        assert_eq!(
            split_by_qty(rupees(100), &[(2, q(1)), (2, q(1))]),
            Err(MoneyError::DuplicateId)
        );
    }

    #[test]
    fn empty_lists_are_rejected() {
        let none: [(u32, Bps); 0] = [];
        assert_eq!(
            split_by_bps(rupees(1), &none),
            Err(MoneyError::SharesNot100)
        );
        let none: [(u32, Qty); 0] = [];
        assert_eq!(
            split_by_qty(rupees(1), &none),
            Err(MoneyError::DivisionByZero)
        );
        // Weighted helper on an empty list directly.
        let none: [(u32, i128); 0] = [];
        assert_eq!(
            split_weighted(rupees(1), &none, 1),
            Err(MoneyError::DivisionByZero)
        );
    }

    #[test]
    fn huge_quantities_overflow_safely() {
        let q = |v| Qty::new(v).unwrap();
        let max_amount = Paisa::try_from_paisa(i64::MAX - i64::MAX % 100).unwrap();
        // i64 rupees × i64 qty fits i128, so this works and adds back.
        let parts = split_by_qty(max_amount, &[(1, q(i64::MAX)), (2, q(1))]).unwrap();
        let sum = parts
            .iter()
            .try_fold(Paisa::ZERO, |acc, (_, p)| acc.checked_add(*p))
            .unwrap();
        assert_eq!(sum, max_amount);
        // Totals of weights that overflow i128 cannot be built from i64 quantities, so the
        // helper is tested with a weight list directly.
        let weights = [(1u8, i128::MAX), (2u8, i128::MAX)];
        assert_eq!(
            split_weighted(rupees(10), &weights, 1),
            Err(MoneyError::Overflow)
        );
    }
}
