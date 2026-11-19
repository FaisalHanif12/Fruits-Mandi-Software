//! The only place where a number becomes a floating-point value (PRD FR-18.4, FR-19.3; Code
//! Standards M13, approved by Umar for the Excel export, D5-Q1).
//!
//! Excel stores every number as a double, so a sheet cannot hold a whole-rupee integer. The money
//! rules (M1 to M12) stay whole: the amount arrives as `Paisa`, is divided by 100 with
//! `Paisa::rupees` (integer division), is refused if it is 10^15 rupees or more, and only then
//! converted. Nothing is added, subtracted, multiplied, divided or rounded in floating point:
//! totals come from the backend as `Paisa`, and a value in a sheet is never read back into the
//! app. Write no formulas (`=SUM`) either: a sheet holds values only.
//!
//! The limit is stricter than M13's 2^53 on purpose: Excel shows 15 significant digits, so a
//! larger number would show a wrong last digit. The largest amount the app allows is
//! 10,00,00,00,000 rupees (10^10), far below it.
//!
//! Counts and quantities (whole numbers, such as نگ) use the same check and live here too, so a
//! search for the floating-point type in this crate finds only this file (Umar approved this
//! for the Excel export, Code Standards 1.8, rule M13). A share of a total is written as text,
//! never as a double.
//!
//! Example: `Paisa(105_000)` (1,050 rupees) gives `1050.0`; `Paisa(-1_200_000)` gives `-12000.0`.

use mandi_core::money::{MoneyError, Paisa};

use crate::error::ReportsError;

/// Excel shows 15 significant digits. A whole number of 10^15 or more would show a wrong last
/// digit, so it is refused. (This is below 2^53, where a double stops being exact.)
const SHEET_LIMIT: u64 = 1_000_000_000_000_000;

/// One whole number as a double, refused if Excel could not show it exactly.
fn exact(value: i64) -> Result<f64, ReportsError> {
    if value.unsigned_abs() >= SHEET_LIMIT {
        return Err(ReportsError::Money(MoneyError::Overflow));
    }
    // Below 10^15 in size, so every such integer is held exactly and shown in full.
    Ok(value as f64)
}

/// Whole rupees of a money amount, as a double for a sheet cell (the one money exception, M13).
///
/// # Errors
/// `ReportsError::Money(Overflow)` for 10^15 rupees or more (the largest allowed amount is
/// 10,00,00,00,000 rupees).
///
/// ```
/// use mandi_core::money::Paisa;
/// let rupees = mandi_reports::export::rupees_as_f64(Paisa::from_rupees(1_050).unwrap());
/// assert_eq!(rupees.unwrap().to_string(), "1050");
/// ```
pub fn rupees_as_f64(amount: Paisa) -> Result<f64, ReportsError> {
    // `Paisa` is always a multiple of 100, so this division loses nothing.
    exact(amount.rupees())
}

/// A count or a quantity (a whole number such as نگ) as a double for a sheet cell (the second
/// exception of M13: exact, never added up, never read back).
///
/// # Errors
/// `ReportsError::Money(Overflow)` for 10^15 or more.
///
/// ```
/// let packages = mandi_reports::export::count_as_f64(60);
/// assert_eq!(packages.unwrap().to_string(), "60");
/// ```
pub fn count_as_f64(value: i64) -> Result<f64, ReportsError> {
    exact(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paisa(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    #[test]
    fn worked_amounts_become_the_same_whole_number() {
        assert_eq!(
            rupees_as_f64(Paisa::ZERO).unwrap().to_bits(),
            0.0_f64.to_bits()
        );
        assert_eq!(rupees_as_f64(paisa(1_050)).unwrap().to_string(), "1050");
        assert_eq!(rupees_as_f64(paisa(720)).unwrap().to_string(), "720");
        // E2E-10 trial balance, cash in hand, a debit and a credit total.
        assert_eq!(rupees_as_f64(paisa(85_048)).unwrap().to_string(), "85048");
        assert_eq!(rupees_as_f64(paisa(58_484)).unwrap().to_string(), "58484");
        // A negative amount keeps its sign.
        assert_eq!(rupees_as_f64(paisa(-12_000)).unwrap().to_string(), "-12000");
        assert_eq!(rupees_as_f64(paisa(-1_004)).unwrap().to_string(), "-1004");
    }

    #[test]
    fn the_largest_allowed_amount_is_exact() {
        // 10,00,00,00,000 rupees (one thousand crore) is the top of the money range (Code
        // Standards 4.2).
        let top = paisa(10_000_000_000);
        assert_eq!(top.as_paisa(), 1_000_000_000_000);
        let value = rupees_as_f64(top).unwrap();
        assert_eq!(value.to_string(), "10000000000");
        let back = format!("{value:.0}").parse::<i64>().unwrap();
        assert_eq!(back, 10_000_000_000);
        let low = rupees_as_f64(paisa(-10_000_000_000)).unwrap();
        assert_eq!(format!("{low:.0}"), "-10000000000");
    }

    #[test]
    fn the_edge_is_ten_to_the_fifteenth_rupees() {
        // 999,999,999,999,999 rupees still shows in full in Excel.
        let edge = 999_999_999_999_999_i64;
        let value = rupees_as_f64(paisa(edge)).unwrap();
        assert_eq!(format!("{value:.0}").parse::<i64>().unwrap(), edge);
        let low = rupees_as_f64(paisa(-edge)).unwrap();
        assert_eq!(format!("{low:.0}").parse::<i64>().unwrap(), -edge);
        // 10^15 rupees and more are refused, on either sign. 2^53 is refused too.
        for refused in [
            1_000_000_000_000_000_i64,
            1_000_000_000_000_001,
            -1_000_000_000_000_000,
            1_i64 << 53,
            -(1_i64 << 53),
            9_000_000_000_000_000,
        ] {
            assert!(
                matches!(
                    rupees_as_f64(paisa(refused)),
                    Err(ReportsError::Money(MoneyError::Overflow))
                ),
                "{refused}"
            );
        }
    }

    #[test]
    fn only_whole_rupees_can_exist_as_paisa() {
        // The type refuses a stored value that is not a whole rupee, so nothing reaches the
        // conversion with a remainder.
        assert!(Paisa::try_from_paisa(150).is_err());
        assert!(Paisa::try_from_paisa(100).is_ok());
    }

    #[test]
    fn counts_use_the_same_check() {
        assert_eq!(count_as_f64(60).unwrap().to_string(), "60");
        assert_eq!(count_as_f64(-5).unwrap().to_string(), "-5");
        assert_eq!(count_as_f64(0).unwrap().to_bits(), 0.0_f64.to_bits());
        assert!(count_as_f64(999_999_999_999_999).is_ok());
        assert!(count_as_f64(1_000_000_000_000_000).is_err());
        assert!(count_as_f64(-1_000_000_000_000_000).is_err());
        assert!(count_as_f64(i64::MIN).is_err());
    }
}
