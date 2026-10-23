//! The two small workings of analytics that are not sums (PRD 18.2, money agent Section 9):
//! the share of a total ("% of all commission") and the average number of days to pay. Both are
//! display only: nothing here is stored or posted.
//!
//! Example (PRD 18.4, view A): کیلا 3,928 of 5,448 is `(3,928 × 10,000 + 5,448 / 2) / 5,448 =
//! 7,210`, shown as 72.10%.

use mandi_core::money::Paisa;

use crate::error::ReportsError;

/// The whole in hundredths of a percent: 100.00%.
pub(crate) const WHOLE: i64 = 10_000;

fn data(what: &str) -> ReportsError {
    ReportsError::Data(what.to_owned())
}

/// `part` as hundredths of a percent of `total`: `(part × 10,000 + total / 2) / total` in `i128`
/// with checked maths, whole numbers, half up. Paisa totals are multiples of 100, so paisa and
/// rupees give the same answer.
///
/// Returns `None` when `total` is 0 or less (there is nothing to take a share of). The rows are
/// not adjusted to add up to exactly 100.00; each is within half a hundredth of the exact value.
///
/// ```
/// use mandi_core::money::Paisa;
/// use mandi_reports::analytics::share_hundredths;
/// let p = |rupees| Paisa::from_rupees(rupees).unwrap();
/// assert_eq!(share_hundredths(p(3_928), p(5_448)).unwrap(), Some(7_210));
/// assert_eq!(share_hundredths(p(1_520), p(5_448)).unwrap(), Some(2_790));
/// assert_eq!(share_hundredths(p(10), p(0)).unwrap(), None);
/// ```
///
/// # Errors
/// `ReportsError::Data` for a part below 0 or above the total (a fact net that cannot happen
/// when the facts are right); `ReportsError::Money` is not used.
pub fn share_hundredths(part: Paisa, total: Paisa) -> Result<Option<i64>, ReportsError> {
    if total <= Paisa::ZERO {
        return Ok(None);
    }
    if part.is_negative() || part > total {
        return Err(data("a share is outside 0 to 100 percent"));
    }
    let part = i128::from(part.as_paisa());
    let total = i128::from(total.as_paisa());
    let value = part
        .checked_mul(i128::from(WHOLE))
        .and_then(|scaled| scaled.checked_add(total / 2))
        .and_then(|scaled| scaled.checked_div(total))
        .ok_or_else(|| data("a share does not fit"))?;
    i64::try_from(value)
        .map(Some)
        .map_err(|_| data("a share does not fit"))
}

/// The average of `count` whole-day amounts that add up to `sum_days`, to the nearest whole day,
/// half up: `(2 × sum + count) / (2 × count)`. `None` when there is nothing to average.
///
/// ```
/// use mandi_reports::analytics::average_days;
/// assert_eq!(average_days(0, 0).unwrap(), None);
/// assert_eq!(average_days(3, 2).unwrap(), Some(2)); // 1.5 days
/// assert_eq!(average_days(1, 3).unwrap(), Some(0)); // 0.33 days
/// ```
///
/// # Errors
/// `ReportsError::Data` for a negative sum or count, or a value that does not fit.
pub fn average_days(sum_days: i64, count: i64) -> Result<Option<i64>, ReportsError> {
    if count == 0 {
        return Ok(None);
    }
    if sum_days < 0 || count < 0 {
        return Err(data("days to pay are negative"));
    }
    let sum = i128::from(sum_days);
    let count = i128::from(count);
    let value = sum
        .checked_mul(2)
        .and_then(|twice| twice.checked_add(count))
        .and_then(|scaled| scaled.checked_div(count.checked_mul(2)?))
        .ok_or_else(|| data("average days do not fit"))?;
    i64::try_from(value)
        .map(Some)
        .map_err(|_| data("average days do not fit"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    #[test]
    fn the_prd_shares_of_view_a() {
        // PRD 18.4: 3,928 and 1,520 of 5,448.
        assert_eq!(share_hundredths(p(3_928), p(5_448)).unwrap(), Some(7_210));
        assert_eq!(share_hundredths(p(1_520), p(5_448)).unwrap(), Some(2_790));
        // After E2E-13: 3,952 and 1,520 of 5,472.
        assert_eq!(share_hundredths(p(3_952), p(5_472)).unwrap(), Some(7_222));
        assert_eq!(share_hundredths(p(1_520), p(5_472)).unwrap(), Some(2_778));
        // After E2E-15: only کیلا is left.
        assert_eq!(share_hundredths(p(3_928), p(3_928)).unwrap(), Some(10_000));
    }

    #[test]
    fn half_rounds_up_and_zero_total_has_no_share() {
        // 1 of 8 is 12.5%: exactly 1,250 hundredths. 1 of 3 is 33.33 (3,333.3 rounds down).
        assert_eq!(share_hundredths(p(1), p(8)).unwrap(), Some(1_250));
        assert_eq!(share_hundredths(p(1), p(3)).unwrap(), Some(3_333));
        assert_eq!(share_hundredths(p(2), p(3)).unwrap(), Some(6_667));
        assert_eq!(share_hundredths(Paisa::ZERO, Paisa::ZERO).unwrap(), None);
    }

    #[test]
    fn a_negative_or_too_big_part_is_a_bug_signal() {
        assert!(share_hundredths(p(-1), p(5)).is_err());
        assert!(share_hundredths(p(6), p(5)).is_err());
    }

    #[test]
    fn average_days_rounds_half_up() {
        assert_eq!(average_days(0, 1).unwrap(), Some(0));
        assert_eq!(average_days(1, 2).unwrap(), Some(1)); // 0.5
        assert_eq!(average_days(2, 3).unwrap(), Some(1)); // 0.67
        assert_eq!(average_days(10, 4).unwrap(), Some(3)); // 2.5
        assert!(average_days(-1, 1).is_err());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        #[test]
        fn a_share_is_within_half_a_hundredth_of_the_exact_value(
            parts in proptest::collection::vec(0_i64..=50_000, 1..12),
        ) {
            let total_rupees: i64 = parts.iter().sum();
            prop_assume!(total_rupees > 0);
            let total = p(total_rupees);
            let mut sum_of_shares = 0_i64;
            for part in &parts {
                let share = share_hundredths(p(*part), total).unwrap().unwrap();
                prop_assert!((0..=WHOLE).contains(&share));
                // |share - exact| <= 0.5, i.e. 2 * |share * total - part * 10,000| <= total.
                let exact_twice = (i128::from(share) * i128::from(total_rupees)
                    - i128::from(*part) * i128::from(WHOLE)).abs() * 2;
                prop_assert!(exact_twice <= i128::from(total_rupees));
                sum_of_shares += share;
            }
            // The rows are not forced to add up; each is off by at most half a hundredth.
            let rows = i64::try_from(parts.len()).unwrap();
            prop_assert!((sum_of_shares - WHOLE).abs() <= rows);
        }

        #[test]
        fn average_days_is_between_the_smallest_and_the_largest(
            days in proptest::collection::vec(0_i64..=3_650, 1..20),
        ) {
            let sum: i64 = days.iter().sum();
            let count = i64::try_from(days.len()).unwrap();
            let average = average_days(sum, count).unwrap().unwrap();
            let low = *days.iter().min().unwrap();
            let high = *days.iter().max().unwrap();
            prop_assert!(average >= low && average <= high);
        }
    }
}
