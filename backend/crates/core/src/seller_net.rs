//! A seller's payable and open advance netted against each other (PRD 9.1, FR-9.6 and FR-7.59
//! v1.61, plan 100 A). The tiles, the seller dues page, the پیشگی page, the "پورا بقایا" button and
//! the reminders all use this one function, so they always agree. R24 keeps the three columns, and
//! matching does not change (PRD FR-15.8).
//!
//! A seller's ledger balance is `payable - open advance - owed beyond advances` (R24's identity),
//! and the payable and "owed beyond" are never both above 0. So the netted payable is exactly his
//! جمع balance when he has one.
//!
//! Money is `Paisa`, whole rupees, all maths checked.

use crate::money::{MoneyError, Paisa};

/// A seller's payable and advance netted against each other (see [`net_of`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetPayable {
    /// What the shop really owes him: `max(open - advances_open, 0)`.
    pub payable: Paisa,
    /// The part of his open advances his payable does not cover: `max(advances_open - open, 0)`.
    pub uncovered_advance: Paisa,
}

/// What the shop really owes a seller and what of his advances it does not cover. At most one of
/// the two is above 0, and `payable - uncovered_advance` is `open - advances_open`.
///
/// ```
/// use mandi_core::money::Paisa;
/// use mandi_core::seller_net::net_of;
/// let p = |rupees| Paisa::from_rupees(rupees).unwrap();
/// // Umar's فیصل: payable 117,040, advance 100,000 -> the shop owes him 17,040.
/// let n = net_of(p(117_040), p(100_000)).unwrap();
/// assert_eq!((n.payable, n.uncovered_advance), (p(17_040), Paisa::ZERO));
/// // E2E-28 (Umar 28-11-2026): 19,316 and 15,000 -> 4,316; after paying 19,316 the advance of
/// // 15,000 is uncovered.
/// let n = net_of(p(19_316), p(15_000)).unwrap();
/// assert_eq!((n.payable, n.uncovered_advance), (p(4_316), Paisa::ZERO));
/// let n = net_of(Paisa::ZERO, p(15_000)).unwrap();
/// assert_eq!((n.payable, n.uncovered_advance), (Paisa::ZERO, p(15_000)));
/// ```
///
/// # Errors
/// `MoneyError::NegativeValue` for a negative input (a bug in the working);
/// `MoneyError::Overflow`.
pub fn net_of(open: Paisa, advances_open: Paisa) -> Result<NetPayable, MoneyError> {
    if open.is_negative() || advances_open.is_negative() {
        return Err(MoneyError::NegativeValue);
    }
    let difference = open.checked_sub(advances_open)?;
    let (payable, uncovered_advance) = if difference.is_negative() {
        (Paisa::ZERO, difference.checked_neg()?)
    } else {
        (difference, Paisa::ZERO)
    };
    Ok(NetPayable {
        payable,
        uncovered_advance,
    })
}

/// A seller's open advance split by what his own credit already covers (PRD FR-9.6 v1.61,
/// "covered advance", Umar 28-11-2026). See [`advance_cover`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvanceCover {
    /// The part his payable already covers: it is taken out of his balance.
    pub covered: Paisa,
    /// The part not covered: the only part shown as پیشگی باقی.
    pub uncovered: Paisa,
}

/// Splits a seller's open advance by his ledger balance: `uncovered = min(advance, owed)`, where
/// `owed` is what his balance says he owes the shop (his بنام balance, 0 when it is جمع). With
/// R24's identity this is `max(advance - payable, 0)`, the same as [`net_of`]`.uncovered_advance`,
/// so the party card and the tiles always agree, and the card needs no payables working.
///
/// ```
/// use mandi_core::money::Paisa;
/// use mandi_core::seller_net::advance_cover;
/// let p = |rupees| Paisa::from_rupees(rupees).unwrap();
/// // فیصل: advance 100,000, balance 17,040 جمع -> all covered.
/// let c = advance_cover(p(100_000), Paisa::ZERO).unwrap();
/// assert_eq!((c.covered, c.uncovered), (p(100_000), Paisa::ZERO));
/// // رحیم after E2E-28's payment: advance 15,000, balance 15,000 بنام -> nothing covered.
/// let c = advance_cover(p(15_000), p(15_000)).unwrap();
/// assert_eq!((c.covered, c.uncovered), (Paisa::ZERO, p(15_000)));
/// ```
///
/// # Errors
/// `MoneyError::NegativeValue` for a negative input; `MoneyError::Overflow`.
pub fn advance_cover(advances_open: Paisa, owed: Paisa) -> Result<AdvanceCover, MoneyError> {
    if advances_open.is_negative() || owed.is_negative() {
        return Err(MoneyError::NegativeValue);
    }
    let uncovered = advances_open.min(owed);
    Ok(AdvanceCover {
        covered: advances_open.checked_sub(uncovered)?,
        uncovered,
    })
}

/// The open amounts of a seller's payable postings, given OLDEST FIRST in payable order (PRD
/// FR-7.59, his opening credit first), after the open advance is netted against them oldest first
/// (as advance matching takes the oldest, FR-15.8): each posting's part that is still really to
/// pay. Their sum is `net_of(the sum of still_open, advances_open).payable`.
///
/// ```
/// use mandi_core::money::Paisa;
/// use mandi_core::seller_net::net_oldest_first;
/// let p = |rupees| Paisa::from_rupees(rupees).unwrap();
/// let left = net_oldest_first(&[p(100), p(50), p(30)], p(120)).unwrap();
/// assert_eq!(left, vec![Paisa::ZERO, p(30), p(30)]);
/// ```
///
/// # Errors
/// `MoneyError::NegativeValue` for a negative input; `MoneyError::Overflow`.
pub fn net_oldest_first(
    still_open: &[Paisa],
    advances_open: Paisa,
) -> Result<Vec<Paisa>, MoneyError> {
    if advances_open.is_negative() {
        return Err(MoneyError::NegativeValue);
    }
    let mut advance_left = advances_open;
    let mut out = Vec::with_capacity(still_open.len());
    for open in still_open {
        if open.is_negative() {
            return Err(MoneyError::NegativeValue);
        }
        let taken = advance_left.min(*open);
        advance_left = advance_left.checked_sub(taken)?;
        out.push(open.checked_sub(taken)?);
    }
    Ok(out)
}

/// Which credit is the oldest one still needed to hold `target`, counting from the NEWEST credit
/// (payments, and the advance netted here, settle the oldest first, PRD FR-7.59): the index, in the
/// newest-first list given, of the credit where the running sum reaches `target`. `None` for a
/// target of 0, or when the credits together hold less than it.
///
/// Used for the days of a seller's net payable (PRD FR-7.44 v1.61).
///
/// ```
/// use mandi_core::money::Paisa;
/// use mandi_core::seller_net::oldest_holding;
/// let p = |rupees| Paisa::from_rupees(rupees).unwrap();
/// // Newest first 30, 50, 100: 70 is held by the first two.
/// assert_eq!(oldest_holding([p(30), p(50), p(100)], p(70)).unwrap(), Some(1));
/// assert_eq!(oldest_holding([p(30)], Paisa::ZERO).unwrap(), None);
/// ```
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn oldest_holding(
    newest_first: impl IntoIterator<Item = Paisa>,
    target: Paisa,
) -> Result<Option<usize>, MoneyError> {
    if target <= Paisa::ZERO {
        return Ok(None);
    }
    let mut held = Paisa::ZERO;
    for (index, amount) in newest_first.into_iter().enumerate() {
        held = held.checked_add(amount.max(Paisa::ZERO))?;
        if held >= target {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn rupees(n: i64) -> Paisa {
        Paisa::from_rupees(n).unwrap()
    }

    #[test]
    fn a_negative_figure_is_refused() {
        let minus = rupees(1).checked_neg().unwrap();
        assert!(net_of(minus, Paisa::ZERO).is_err());
        assert!(net_of(Paisa::ZERO, minus).is_err());
        assert!(net_oldest_first(&[minus], Paisa::ZERO).is_err());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        /// The split by the balance equals the netting of the payable (R24's identity): the
        /// balance is `payable - advance - owed beyond`, with payable and owed beyond never both
        /// above 0.
        #[test]
        fn the_cover_by_the_balance_is_the_netted_advance(
            payable in 0_i64..20_000,
            advance in 0_i64..20_000,
            beyond in 0_i64..5_000,
        ) {
            let beyond = if payable == 0 { beyond } else { 0 };
            let balance = rupees(payable)
                .checked_sub(rupees(advance))
                .and_then(|b| b.checked_sub(rupees(beyond)))
                .unwrap();
            let owed = if balance.is_negative() {
                balance.checked_neg().unwrap()
            } else {
                Paisa::ZERO
            };
            let cover = advance_cover(rupees(advance), owed).unwrap();
            let net = net_of(rupees(payable), rupees(advance)).unwrap();
            prop_assert_eq!(cover.uncovered, net.uncovered_advance);
            prop_assert_eq!(cover.covered.checked_add(cover.uncovered).unwrap(), rupees(advance));
        }

        /// The netted figures always add back, at most one is above 0, and netting the postings
        /// oldest first leaves exactly the net payable.
        #[test]
        fn netting_keeps_the_books_and_matches_the_postings(
            amounts in proptest::collection::vec(0_i64..5_000, 0..12),
            advance in 0_i64..20_000,
        ) {
            let postings: Vec<Paisa> = amounts.iter().map(|a| rupees(*a)).collect();
            let open = postings
                .iter()
                .try_fold(Paisa::ZERO, |sum, a| sum.checked_add(*a))
                .unwrap();
            let net = net_of(open, rupees(advance)).unwrap();
            prop_assert!(net.payable == Paisa::ZERO || net.uncovered_advance == Paisa::ZERO);
            prop_assert_eq!(
                net.payable.checked_sub(net.uncovered_advance).unwrap(),
                open.checked_sub(rupees(advance)).unwrap()
            );
            let left = net_oldest_first(&postings, rupees(advance)).unwrap();
            let total = left
                .iter()
                .try_fold(Paisa::ZERO, |sum, a| sum.checked_add(*a))
                .unwrap();
            prop_assert_eq!(total, net.payable);
        }
    }
}
