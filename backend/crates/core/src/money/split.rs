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

/// Splits what an advance gives back over the parts of it that are still open (PRD C28, FR-17.13).
///
/// `settled` is what came back, more than 0 up to the total of the open parts. Each part is the
/// floor of its exact share in whole rupees; the leftover rupees are then given ONE rupee each,
/// starting with the largest open part, and on a tie the smallest id (the partner created first). A
/// part never goes above what is open of it, a part with nothing open gets nothing, and when
/// everything is settled each part is exactly its open part.
///
/// Example (PRD C28): open 6,000 / 4,500 / 4,500 and 14,999 settled: floors 5,999 / 4,499 / 4,499,
/// leftover 2 goes to the 6,000 and then to the first 4,500 → 6,000 / 4,500 / 4,499.
///
/// # Errors
/// `NegativeValue` if `settled` or an open part is below 0; `SettledMoreThanOpen` if `settled` is
/// more than the total open; `DuplicateId` for a repeated id; `Overflow`.
pub fn split_by_open_parts<Id: Copy + Ord>(
    settled: Paisa,
    open: &[(Id, Paisa)],
) -> Result<Vec<(Id, Paisa)>, MoneyError> {
    if settled.is_negative() || open.iter().any(|(_, part)| part.is_negative()) {
        return Err(MoneyError::NegativeValue);
    }
    let mut ids: Vec<Id> = open.iter().map(|&(id, _)| id).collect();
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair.first() == pair.last()) {
        return Err(MoneyError::DuplicateId);
    }
    let weights: Vec<i128> = open
        .iter()
        .map(|&(_, part)| i128::from(part.rupees()))
        .collect();
    let total = weights
        .iter()
        .try_fold(0i128, |acc, w| acc.checked_add(*w))
        .ok_or(MoneyError::Overflow)?;
    let settled_rupees = i128::from(settled.rupees());
    if settled_rupees > total {
        return Err(MoneyError::SettledMoreThanOpen);
    }
    if total == 0 {
        // Nothing is open and nothing is settled.
        return open.iter().map(|&(id, _)| Ok((id, Paisa::ZERO))).collect();
    }
    let mut parts: Vec<i128> = Vec::with_capacity(weights.len());
    for weight in &weights {
        let part = settled_rupees
            .checked_mul(*weight)
            .and_then(|p| p.checked_div(total))
            .ok_or(MoneyError::Overflow)?;
        parts.push(part);
    }
    let given = parts
        .iter()
        .try_fold(0i128, |acc, p| acc.checked_add(*p))
        .ok_or(MoneyError::Overflow)?;
    let mut leftover = settled_rupees
        .checked_sub(given)
        .ok_or(MoneyError::Overflow)?;
    // Largest open part first, then the smallest id; a part with nothing open is skipped.
    let mut order: Vec<usize> = (0..weights.len()).collect();
    order.sort_by(|&a, &b| {
        let (wa, wb) = (weights.get(a), weights.get(b));
        let (ia, ib) = (open.get(a).map(|o| o.0), open.get(b).map(|o| o.0));
        wb.cmp(&wa).then(ia.cmp(&ib))
    });
    for index in order {
        if leftover == 0 {
            break;
        }
        let open_here = weights.get(index).copied().unwrap_or(0);
        if let Some(slot) = parts.get_mut(index)
            && *slot < open_here
        {
            *slot = slot.checked_add(1).ok_or(MoneyError::Overflow)?;
            leftover = leftover.checked_sub(1).ok_or(MoneyError::Overflow)?;
        }
    }
    if leftover != 0 {
        // Cannot happen: the floors leave less than one rupee for each part with something open.
        return Err(MoneyError::Overflow);
    }
    open.iter()
        .zip(parts)
        .map(|(&(id, _), rupees)| {
            let rupees = i64::try_from(rupees).map_err(|_| MoneyError::Overflow)?;
            Ok((id, Paisa::from_rupees(rupees)?))
        })
        .collect()
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

    #[test]
    fn open_parts_follow_the_worked_numbers_of_c28() {
        let open = [(1, rupees(6_000)), (2, rupees(4_500)), (3, rupees(4_500))];
        let take = |settled: i64, open: &[(i32, Paisa)]| -> Vec<i64> {
            split_by_open_parts(rupees(settled), open)
                .unwrap()
                .into_iter()
                .map(|(_, p)| p.rupees())
                .collect()
        };
        // 14,999 of 15,000: floors 5,999 / 4,499 / 4,499, leftover 2 to the largest, then the first 4,500.
        assert_eq!(take(14_999, &open), vec![6_000, 4_500, 4_499]);
        // E2E-31: 10,000 of 15,000, then 5,000 of what is open.
        assert_eq!(take(10_000, &open), vec![4_000, 3_000, 3_000]);
        let rest = [(1, rupees(2_000)), (2, rupees(1_500)), (3, rupees(1_500))];
        assert_eq!(take(5_000, &rest), vec![2_000, 1_500, 1_500]);
        // Everything settled gives exactly the open parts; nothing settled gives zeros.
        assert_eq!(take(15_000, &open), vec![6_000, 4_500, 4_500]);
        assert_eq!(take(0, &open), vec![0, 0, 0]);
        // A part with nothing open gets nothing, and the leftover goes to the others.
        let two = [(1, rupees(0)), (2, rupees(100)), (3, rupees(100))];
        assert_eq!(take(101, &two[..]), vec![0, 51, 50]);
        // One partner takes all of it.
        assert_eq!(take(7, &[(9, rupees(10))]), vec![7]);
        // Open 0 / 5,000 / 5,000 and 1 settled: the part with nothing open gets nothing, even with
        // the lowest id.
        let skip = [(1, rupees(0)), (2, rupees(5_000)), (3, rupees(5_000))];
        assert_eq!(take(1, &skip), vec![0, 1, 0]);
        // Open 1 / 1 / 1 and 2 settled: one rupee each to the first two.
        let ones = [(1, rupees(1)), (2, rupees(1)), (3, rupees(1))];
        assert_eq!(take(2, &ones), vec![1, 1, 0]);
        // An advance of 1,001 split 401 / 300 / 300: 500 comes back as 201 / 150 / 149, and 501 as
        // 201 / 150 / 150.
        let uneven = [(1, rupees(401)), (2, rupees(300)), (3, rupees(300))];
        assert_eq!(take(500, &uneven), vec![201, 150, 149]);
        assert_eq!(take(501, &uneven), vec![201, 150, 150]);
    }

    #[test]
    fn open_parts_refuse_bad_input() {
        let open = [(1, rupees(100)), (2, rupees(100))];
        assert_eq!(
            split_by_open_parts(rupees(201), &open),
            Err(MoneyError::SettledMoreThanOpen)
        );
        assert_eq!(
            split_by_open_parts(rupees(-1), &open),
            Err(MoneyError::NegativeValue)
        );
        assert_eq!(
            split_by_open_parts(rupees(1), &[(1, rupees(100)), (1, rupees(100))]),
            Err(MoneyError::DuplicateId)
        );
        let none: [(u8, Paisa); 0] = [];
        assert_eq!(split_by_open_parts(rupees(0), &none), Ok(Vec::new()));
        assert_eq!(
            split_by_open_parts(rupees(1), &none),
            Err(MoneyError::SettledMoreThanOpen)
        );
    }
}
