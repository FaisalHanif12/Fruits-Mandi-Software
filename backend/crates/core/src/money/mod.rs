//! Money: the one place where amounts are made, multiplied, rounded and split
//! (PRD Section 22, Code Standards 3, money-calculations agent Sections 1 to 5).
//!
//! - Money is `Paisa(i64)`, always a whole number of rupees (a multiple of 100).
//!   1,050 rupees is `Paisa(105000)`. The field is private, so no other value can exist.
//! - Percent is `Bps(i32)`: 1% = 100 bps, 6% = `Bps(600)`.
//! - Multiply first, divide last, in `i128`, then round once with `round_to_rupee`
//!   (nearest rupee, half up). Only `apply_bps`, `weight_value` and `avg_rate` call it.
//! - Every operation is checked. Overflow is an error, never a wrap (Code Standards M7).
//! - No floats anywhere.

mod format;
mod parse;
mod split;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::ErrorCode;

pub use format::{Grouping, format_paisa};
pub use parse::{
    MAX_QTY, MAX_RUPEES, MAX_WEIGHT_GRAMS, parse_bps, parse_qty, parse_rupees, parse_sum_qty,
    parse_sum_rupees, parse_weight,
};
pub use split::{split_by_bps, split_by_qty};

/// Paisa in one rupee.
const PAISA_PER_RUPEE: i64 = 100;
/// Basis points in 100% (1% = 100 bps).
pub const BPS_FULL: i32 = 10_000;
/// Grams in one kg.
const GRAMS_PER_KG: i128 = 1_000;

/// A problem inside a money calculation. Most are bugs in the caller; `Overflow` and
/// `SharesNot100` can come from data the user entered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MoneyError {
    /// A result does not fit in the integer type (Code Standards M7).
    #[error("money overflow")]
    Overflow,
    /// A paisa value that is not a whole rupee (PRD Section 22).
    #[error("amount is not a whole rupee")]
    NotWholeRupee,
    /// A charge or rounding was asked for on a negative base. Charges are worked out on
    /// positive bases only (money agent 4.1).
    #[error("negative base")]
    NegativeBase,
    /// A negative quantity, weight or rate where only 0 or more makes sense.
    #[error("negative value")]
    NegativeValue,
    /// A division by zero, for example an average over 0 نگ or a split over no lots.
    #[error("division by zero")]
    DivisionByZero,
    /// A percent outside 0 to 100% (0 to 10,000 bps).
    #[error("bps out of range")]
    BpsOutOfRange,
    /// Partner shares do not total exactly 10,000 bps (PRD FR-17.1).
    #[error("shares do not total 100%")]
    SharesNot100,
    /// The same partner or lot appears twice in one split.
    #[error("duplicate id in split")]
    DuplicateId,
}

impl MoneyError {
    /// The error code for the screen. Caller bugs become `ERR_INTERNAL`.
    pub fn code(self) -> ErrorCode {
        match self {
            MoneyError::Overflow => ErrorCode::ErrOverflow,
            MoneyError::SharesNot100 => ErrorCode::ErrSharesNot100,
            MoneyError::NotWholeRupee
            | MoneyError::NegativeBase
            | MoneyError::NegativeValue
            | MoneyError::DivisionByZero
            | MoneyError::BpsOutOfRange
            | MoneyError::DuplicateId => ErrorCode::ErrInternal,
        }
    }
}

/// Money in paisa, always a whole number of rupees (a multiple of 100). May be negative
/// for sums and balances. Travels to the frontend as a JSON integer and is typed `Paisa`
/// there (plan 02 Section 4.1).
///
/// ```
/// use mandi_core::money::Paisa;
/// let p = Paisa::from_rupees(1_050).unwrap();
/// assert_eq!(p.as_paisa(), 105_000);
/// assert!(Paisa::try_from_paisa(150).is_err());
/// ```
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(try_from = "i64", into = "i64")]
pub struct Paisa(i64);

impl Paisa {
    /// Zero rupees.
    pub const ZERO: Paisa = Paisa(0);

    /// Whole rupees to paisa. `from_rupees(720)` is `Paisa(72000)`.
    pub fn from_rupees(rupees: i64) -> Result<Paisa, MoneyError> {
        rupees
            .checked_mul(PAISA_PER_RUPEE)
            .map(Paisa)
            .ok_or(MoneyError::Overflow)
    }

    /// A stored paisa value. Fails if it is not a whole rupee.
    pub fn try_from_paisa(paisa: i64) -> Result<Paisa, MoneyError> {
        match paisa.checked_rem(PAISA_PER_RUPEE) {
            Some(0) => Ok(Paisa(paisa)),
            _ => Err(MoneyError::NotWholeRupee),
        }
    }

    /// The paisa inside, for storage (`*_paisa` columns) and JSON.
    pub fn as_paisa(self) -> i64 {
        self.0
    }

    /// Whole rupees. Exact, because the value is always a multiple of 100.
    pub fn rupees(self) -> i64 {
        // Division by a non-zero constant other than -1 cannot overflow; the value is
        // a multiple of 100, so nothing is lost.
        self.0.checked_div(PAISA_PER_RUPEE).unwrap_or_default()
    }

    /// True for amounts below zero.
    pub fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// `self + other`, checked.
    pub fn checked_add(self, other: Paisa) -> Result<Paisa, MoneyError> {
        self.0
            .checked_add(other.0)
            .map(Paisa)
            .ok_or(MoneyError::Overflow)
    }

    /// `self - other`, checked.
    pub fn checked_sub(self, other: Paisa) -> Result<Paisa, MoneyError> {
        self.0
            .checked_sub(other.0)
            .map(Paisa)
            .ok_or(MoneyError::Overflow)
    }

    /// `-self`, checked. Used to split negative amounts (PRD C11).
    pub fn checked_neg(self) -> Result<Paisa, MoneyError> {
        self.0.checked_neg().map(Paisa).ok_or(MoneyError::Overflow)
    }
}

impl TryFrom<i64> for Paisa {
    type Error = MoneyError;

    fn try_from(paisa: i64) -> Result<Self, Self::Error> {
        Paisa::try_from_paisa(paisa)
    }
}

impl From<Paisa> for i64 {
    fn from(p: Paisa) -> i64 {
        p.0
    }
}

/// In TypeScript, money is the opaque type `Paisa` from `frontend/src/shared/format/money.ts`
/// (plan 02 Section 4.1). No `Paisa.ts` file is written; `pnpm gen:types` adds the import.
impl ts_rs::TS for Paisa {
    type WithoutGenerics = Self;
    type OptionInnerType = Self;

    fn name(_: &ts_rs::Config) -> String {
        "Paisa".to_owned()
    }

    fn inline(cfg: &ts_rs::Config) -> String {
        <Self as ts_rs::TS>::name(cfg)
    }
}

/// A percent in basis points, 0 to 10,000 (1% = 100, 2.5% = 250).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bps(i32);

impl Bps {
    /// 100%.
    pub const FULL: Bps = Bps(BPS_FULL);

    /// A percent in bps. Only 0 to 10,000 is allowed.
    pub fn new(bps: i32) -> Result<Bps, MoneyError> {
        if (0..=BPS_FULL).contains(&bps) {
            Ok(Bps(bps))
        } else {
            Err(MoneyError::BpsOutOfRange)
        }
    }

    /// The bps inside, for storage (`*_bps` columns).
    pub fn get(self) -> i32 {
        self.0
    }
}

/// A count of نگ, 0 or more. One line takes 1 to 1,000,000 (checked by `parse_qty`);
/// sums of lines may be larger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Qty(i64);

impl Qty {
    /// A quantity of 0 or more.
    pub fn new(qty: i64) -> Result<Qty, MoneyError> {
        if qty < 0 {
            Err(MoneyError::NegativeValue)
        } else {
            Ok(Qty(qty))
        }
    }

    /// The count inside.
    pub fn get(self) -> i64 {
        self.0
    }
}

/// A weight in grams, 0 or more. One input takes 1 g to 100,000 kg (checked by `parse_weight`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Grams(i64);

impl Grams {
    /// A weight of 0 or more grams.
    pub fn new(grams: i64) -> Result<Grams, MoneyError> {
        if grams < 0 {
            Err(MoneyError::NegativeValue)
        } else {
            Ok(Grams(grams))
        }
    }

    /// The grams inside.
    pub fn get(self) -> i64 {
        self.0
    }
}

/// The only rounding (PRD C15, money agent 4.1): the exact value in paisa is
/// `numerator ÷ denominator`; the result is the nearest whole rupee, half up.
///
/// `rupees = (numerator + 50 × denominator) ÷ (100 × denominator)`.
/// Example: 337,500,000 ÷ 10,000 = 33,750 paisa = 337.5 rupees → 338.
fn round_to_rupee(numerator: i128, denominator: i128) -> Result<Paisa, MoneyError> {
    if denominator <= 0 {
        return Err(MoneyError::DivisionByZero);
    }
    if numerator < 0 {
        return Err(MoneyError::NegativeBase);
    }
    let half = denominator.checked_mul(50).ok_or(MoneyError::Overflow)?;
    let whole = denominator.checked_mul(100).ok_or(MoneyError::Overflow)?;
    let rupees = numerator
        .checked_add(half)
        .and_then(|n| n.checked_div(whole))
        .ok_or(MoneyError::Overflow)?;
    let rupees = i64::try_from(rupees).map_err(|_| MoneyError::Overflow)?;
    Paisa::from_rupees(rupees)
}

/// Line value by نگ (PRD C1): `qty × rate`, exact.
///
/// Example: 20 × 600 = 12,000 rupees.
pub fn line_value(qty: Qty, rate: Paisa) -> Result<Paisa, MoneyError> {
    exact_product(qty, rate)
}

/// A per-نگ charge (PRD C3): `qty × value`, exact. Line charges use the line's qty;
/// lot charges use the lot's qty received.
///
/// Example: 10 نگ × 20 = 200 rupees.
pub fn per_unit(qty: Qty, value: Paisa) -> Result<Paisa, MoneyError> {
    exact_product(qty, value)
}

/// `qty × amount` for a non-negative amount. The product of a whole-rupee amount and a
/// whole number is a whole rupee, so no rounding.
fn exact_product(qty: Qty, amount: Paisa) -> Result<Paisa, MoneyError> {
    if amount.is_negative() {
        return Err(MoneyError::NegativeBase);
    }
    qty.0
        .checked_mul(amount.0)
        .map(Paisa)
        .ok_or(MoneyError::Overflow)
}

/// Line value by weight (PRD C1, E23): `round(kg × rate per kg)`.
///
/// Example: 12.345 kg at 100 per kg = 1,234.5 → 1,235 rupees.
pub fn weight_value(weight: Grams, rate_per_kg: Paisa) -> Result<Paisa, MoneyError> {
    let numerator = i128::from(weight.0)
        .checked_mul(i128::from(rate_per_kg.0))
        .ok_or(MoneyError::Overflow)?;
    round_to_rupee(numerator, GRAMS_PER_KG)
}

/// A percent charge (PRD C2): `round(base × bps ÷ 10,000)`, half up.
///
/// Example: 12,000 at 6% = 720; 5,630 at 6% = 337.8 → 338; 5,625 at 6% = 337.5 → 338.
pub fn apply_bps(base: Paisa, bps: Bps) -> Result<Paisa, MoneyError> {
    if base.is_negative() {
        return Err(MoneyError::NegativeBase);
    }
    let numerator = i128::from(base.0)
        .checked_mul(i128::from(bps.0))
        .ok_or(MoneyError::Overflow)?;
    round_to_rupee(numerator, i128::from(BPS_FULL))
}

/// Average rate for display only (PRD C18): `round(total ÷ qty)`. Never posted.
///
/// Example: 29,400 over 50 نگ = 588 rupees.
pub fn avg_rate(total: Paisa, qty: Qty) -> Result<Paisa, MoneyError> {
    round_to_rupee(i128::from(total.0), i128::from(qty.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_to_rupee_rejects_bad_inputs() {
        assert_eq!(round_to_rupee(100, 0), Err(MoneyError::DivisionByZero));
        assert_eq!(round_to_rupee(100, -5), Err(MoneyError::DivisionByZero));
        assert_eq!(round_to_rupee(-1, 1), Err(MoneyError::NegativeBase));
        assert_eq!(round_to_rupee(i128::MAX, 1), Err(MoneyError::Overflow));
        assert_eq!(round_to_rupee(1, i128::MAX), Err(MoneyError::Overflow));
        assert_eq!(round_to_rupee(1, i128::MAX / 60), Err(MoneyError::Overflow));
        // Fits i128 but not i64 rupees.
        assert_eq!(
            round_to_rupee(i128::from(i64::MAX) * 1_000, 1),
            Err(MoneyError::Overflow)
        );
    }

    #[test]
    fn round_to_rupee_half_up() {
        // 337.5 rupees = 33,750 paisa.
        assert_eq!(round_to_rupee(33_750, 1), Paisa::from_rupees(338));
        assert_eq!(round_to_rupee(33_749, 1), Paisa::from_rupees(337));
        assert_eq!(round_to_rupee(0, 7), Ok(Paisa::ZERO));
    }

    #[test]
    fn typescript_name_is_paisa() {
        use ts_rs::TS;
        let cfg = ts_rs::Config::new();
        assert_eq!(Paisa::name(&cfg), "Paisa");
        assert_eq!(Paisa::inline(&cfg), "Paisa");
        assert!(Paisa::output_path().is_none());
    }

    #[test]
    fn error_codes() {
        assert_eq!(MoneyError::Overflow.code(), ErrorCode::ErrOverflow);
        assert_eq!(MoneyError::SharesNot100.code(), ErrorCode::ErrSharesNot100);
        for bug in [
            MoneyError::NotWholeRupee,
            MoneyError::NegativeBase,
            MoneyError::NegativeValue,
            MoneyError::DivisionByZero,
            MoneyError::BpsOutOfRange,
            MoneyError::DuplicateId,
        ] {
            assert_eq!(bug.code(), ErrorCode::ErrInternal, "{bug}");
        }
    }
}
