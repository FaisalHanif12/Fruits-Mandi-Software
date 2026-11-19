//! The split between our shop and another shop (PRD FR-12.41, plan 35): every line of goods (a trip
//! purchase, a garden arrival) says how many of its نگ are for our shop and how many are for
//! another shop, with an optional name of that shop. Our shop + another shop = the line's qty. It
//! starts as all for our shop and can be changed while goods are left (`purchases_change_split`,
//! `gardens_arrival_change_split`), never below what each part has already sold. No money effect.
//!
//! Our shop sells its part in a بکری (an own lot or a linked garden lot, `bikri`); another shop sells
//! its part through an entry on the trip or garden (`trips_other_sale_*`, `gardens_entry_*`).
//!
//! A write-off (spoiled goods) comes off our shop's part first, then off the other shop's part: the
//! goods that spoil are the ones that sit in our shop. The write-off is one number on the line; the
//! parts are worked out here, the same way everywhere ([`StockParts::of`]).
//!
//! Example: 80 نگ, other shop 30. 20 sold in our shop, 10 sold in the other shop, 5 written off.
//! Our part is 50: sold 20, written off 5, left 25. Other part is 30: sold 10, left 20. Total left
//! 45 = 80 − 30 − 5.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What is left of each part of a line, worked out from the stored counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StockParts {
    /// نگ left for our shop: the part for our shop, less what it sold, less the write-off.
    pub our_left: i64,
    /// نگ left for another shop.
    pub other_left: i64,
    /// نگ sold in our shop (lots of بکری).
    pub our_sold: i64,
    /// نگ sold in other shops.
    pub other_sold: i64,
    /// Everything left: arrived less sold less written off.
    pub total_left: i64,
}

impl StockParts {
    /// Works out the parts. `other_qty` is the part for another shop, `sold` all نگ sold (both
    /// shops), `sold_other` the part of those sold in other shops, `written_off` all نگ written off.
    ///
    /// Returns `None` if the counters do not fit together (a part below what it sold, or sold plus
    /// written off above the qty): a bug signal, never a user mistake.
    pub fn of(
        qty: i64,
        other_qty: i64,
        sold: i64,
        sold_other: i64,
        written_off: i64,
    ) -> Option<Self> {
        if qty < 0
            || other_qty < 0
            || other_qty > qty
            || sold_other < 0
            || sold_other > sold
            || written_off < 0
        {
            return None;
        }
        let our_sold = sold.checked_sub(sold_other)?;
        let our_left_raw = qty.checked_sub(other_qty)?.checked_sub(our_sold)?;
        let other_left_raw = other_qty.checked_sub(sold_other)?;
        if our_left_raw < 0 || other_left_raw < 0 {
            return None;
        }
        let total_left = qty.checked_sub(sold)?.checked_sub(written_off)?;
        if total_left < 0 {
            return None;
        }
        // The write-off comes off our shop's part first.
        let off_ours = written_off.min(our_left_raw);
        let off_other = written_off.checked_sub(off_ours)?;
        Some(StockParts {
            our_left: our_left_raw.checked_sub(off_ours)?,
            other_left: other_left_raw.checked_sub(off_other)?,
            our_sold,
            other_sold: sold_other,
            total_left,
        })
    }

    /// The range the part for another shop may take: not below what other shops have sold, and
    /// not above the qty less what our shop has sold. `None` as in [`StockParts::of`].
    pub fn other_range(qty: i64, sold: i64, sold_other: i64) -> Option<(i64, i64)> {
        let our_sold = sold.checked_sub(sold_other)?;
        let max = qty.checked_sub(our_sold)?;
        (sold_other <= max).then_some((sold_other, max))
    }
}

/// Input of `purchases_change_split` and `gardens_arrival_change_split` (PRD FR-12.41): the part
/// for another shop, and its name. Errors: `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on
/// `version`; the purchase or arrival is cancelled (`ERR_PURCHASE_CANCELLED`,
/// `ERR_ARRIVAL_CANCELLED`) on `id`; `ERR_NOTHING_TO_SELL` on `id` (nothing is left, so there is
/// nothing to split); the quantity codes on `other_qty`; `ERR_SPLIT_ABOVE_QTY` on `other_qty`
/// (value `qty`); `ERR_SPLIT_BELOW_SOLD` on `other_qty` (values `min_other`, the most other shops
/// have sold, and `max_other`, the qty less what our shop has sold).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SplitInput {
    /// The purchase or the arrival.
    #[ts(type = "number")]
    pub id: i64,
    /// Its version.
    #[ts(type = "number")]
    pub version: i64,
    /// The نگ for another shop. Empty means 0. A simple sum is taken.
    #[serde(default)]
    pub other_qty: String,
    /// The name of the other shop, as typed. Empty means none.
    #[serde(default)]
    pub shop_name: Option<String>,
}

/// What the two change-split commands give back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SplitSet {
    /// The purchase or the arrival.
    #[ts(type = "number")]
    pub id: i64,
    /// Its new version.
    #[ts(type = "number")]
    pub version: i64,
    /// The نگ for another shop now.
    #[ts(type = "number")]
    pub qty_other: i64,
    /// The name of the other shop now.
    pub shop_name: Option<String>,
    /// نگ left for our shop now.
    #[ts(type = "number")]
    pub qty_our_left: i64,
    /// نگ left for another shop now.
    #[ts(type = "number")]
    pub qty_other_left: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn the_worked_example_of_the_header() {
        let parts = StockParts::of(80, 30, 30, 10, 5).unwrap();
        assert_eq!(
            parts,
            StockParts {
                our_left: 25,
                other_left: 20,
                our_sold: 20,
                other_sold: 10,
                total_left: 45
            }
        );
    }

    #[test]
    fn an_untouched_line_is_all_for_our_shop() {
        let parts = StockParts::of(20, 0, 5, 0, 3).unwrap();
        assert_eq!(
            (parts.our_left, parts.other_left, parts.total_left),
            (12, 0, 12)
        );
    }

    #[test]
    fn a_write_off_beyond_our_part_comes_off_the_other_part() {
        // 100, other 30, our sold 60: our part has 10 left; 25 written off takes 10 then 15.
        let parts = StockParts::of(100, 30, 60, 0, 25).unwrap();
        assert_eq!(
            (parts.our_left, parts.other_left, parts.total_left),
            (0, 15, 15)
        );
    }

    #[test]
    fn counters_that_do_not_fit_are_refused() {
        assert!(StockParts::of(10, 11, 0, 0, 0).is_none());
        assert!(StockParts::of(10, 5, 4, 5, 0).is_none());
        assert!(StockParts::of(10, 2, 9, 0, 2).is_none());
        // Our part is 8 and our shop sold 9.
        assert!(StockParts::of(10, 2, 9, 0, 0).is_none());
    }

    #[test]
    fn the_range_of_the_other_part() {
        assert_eq!(StockParts::other_range(80, 30, 10), Some((10, 60)));
        assert_eq!(StockParts::other_range(80, 0, 0), Some((0, 80)));
        assert_eq!(StockParts::other_range(10, 12, 0), None);
    }

    proptest! {
        #[test]
        fn the_parts_always_add_up_to_what_is_left(
            qty in 0_i64..1_000,
            other in 0_i64..1_000,
            sold in 0_i64..1_000,
            sold_other in 0_i64..1_000,
            off in 0_i64..1_000,
        ) {
            if let Some(p) = StockParts::of(qty, other, sold, sold_other, off) {
                prop_assert!(p.our_left >= 0 && p.other_left >= 0);
                prop_assert_eq!(p.our_left + p.other_left, p.total_left);
                prop_assert_eq!(p.our_sold + p.other_sold, sold);
                prop_assert_eq!(p.total_left, qty - sold - off);
            }
        }

        #[test]
        fn any_split_inside_the_range_is_valid(
            qty in 1_i64..1_000,
            sold_pct in 0_i64..=100,
            other_pct in 0_i64..=100,
            off_pct in 0_i64..=100,
            pick in 0_i64..1_000,
        ) {
            // Build counters that fit: sold <= qty, sold_other <= sold, sold + off <= qty.
            let sold = qty * sold_pct / 100;
            let sold_other = sold * other_pct / 100;
            let off = (qty - sold) * off_pct / 100;
            let (min, max) = StockParts::other_range(qty, sold, sold_other).unwrap();
            let other = min + pick % (max - min + 1);
            prop_assert!(StockParts::of(qty, other, sold, sold_other, off).is_some());
        }
    }
}
