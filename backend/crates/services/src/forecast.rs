//! Estimates of the months ahead, as a service (PRD FR-7.86, Code Standards M14, Q5).
//!
//! A thin wrapper over [`mandi_domain::forecast`]. The monthly series is read by the caller
//! through the analytics view functions (the same functions as the analytics screen, Code
//! Standards Q3) and passed in: this crate cannot reach `mandi-reports` (Code Standards B1), and
//! the series must not come from a second query. A history that is too short is a result, not an
//! error: the answer says how many full months there are.
//!
//! Example (E2E-59): April to September 2026 of 36,000 to 51,000, today in October: same pace
//! 288,000 and same growth 369,000 over 6 months.

use mandi_domain::core::money::Paisa;
use mandi_domain::forecast::{
    self, BalanceEstimate, BalanceKind, Estimate, ForecastError, MonthValue, PaceFactor, YearMonth,
};

use crate::error::ServiceError;

/// What an estimate gives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Forecast {
    /// The methods that could be worked out.
    Estimate(Estimate),
    /// Fewer than 3 full months: no method can be worked out. The answer says how many there are.
    TooShort {
        /// How many full months the books gave.
        full_months: u32,
    },
}

/// The months ahead from the full months in `history` (see [`mandi_domain::forecast::estimate`]).
///
/// # Errors
/// A rejection with the code of the refusal: `ERR_OUT_OF_RANGE` for months ahead outside 1 to 12,
/// a target of 0 or less, or a history that is not the full months before `current`;
/// `ERR_OVERFLOW` for a figure that does not fit.
pub fn estimate(
    history: &[MonthValue],
    current: YearMonth,
    ahead: u32,
    target: Option<Paisa>,
) -> Result<Forecast, ServiceError> {
    estimate_with(history, current, ahead, target, None)
}

/// [`estimate`] and, with `what_if`, the same pace scaled by the factor as a fourth line (plan 52
/// item 5). A pace below 0 or a factor out of range is `ERR_OUT_OF_RANGE`.
///
/// # Errors
/// As [`estimate`].
pub fn estimate_with(
    history: &[MonthValue],
    current: YearMonth,
    ahead: u32,
    target: Option<Paisa>,
    what_if: Option<PaceFactor>,
) -> Result<Forecast, ServiceError> {
    match forecast::estimate_with(history, current, ahead, target, what_if) {
        Ok(found) => Ok(Forecast::Estimate(found)),
        Err(ForecastError::TooLittleHistory { months }) => Ok(Forecast::TooShort {
            full_months: months,
        }),
        Err(error) => Err(ServiceError::reject(error.code())),
    }
}

/// [`estimate_with`] to the end of the year, with the whole year on every method: the books' months
/// of this year so far plus the months left (see [`mandi_domain::forecast::estimate_to_year_end`]).
/// There is no target.
///
/// Example (E2E-59, today 10-10-26): 261,000 so far, same pace 144,000 ahead, the whole year
/// 405,000.
///
/// # Errors
/// As [`estimate`].
pub fn estimate_to_year_end(
    history: &[MonthValue],
    current: YearMonth,
    what_if: Option<PaceFactor>,
) -> Result<Forecast, ServiceError> {
    match forecast::estimate_to_year_end(history, current, what_if) {
        Ok(found) => Ok(Forecast::Estimate(found)),
        Err(ForecastError::TooLittleHistory { months }) => Ok(Forecast::TooShort {
            full_months: months,
        }),
        Err(error) => Err(ServiceError::reject(error.code())),
    }
}

/// What a balance estimate gives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BalanceForecast {
    /// The balance some months from today at the pace of the past months.
    Estimate(BalanceEstimate),
    /// Fewer than 3 months of change: no estimate. The answer says how many there are.
    TooShort {
        /// How many full months the books gave.
        full_months: u32,
    },
    /// The buyers' dues would fall below 0 within the months asked, which dues cannot. It is a
    /// result of its own, so no other refusal is ever read as this one.
    FallsBelowZero,
}

/// A balance `1 .. ahead` months from today at the pace of its past months (see
/// [`mandi_domain::forecast::balance_at_pace`]). Too little history is a result, not an error.
///
/// Example (PRD): Akram owes 2,880 and his net change in the last 3 full months was +3,000, -1,000
/// and +4,000, an average of +2,000, so after 2 months he owes 6,880.
///
/// # Errors
/// A rejection with the code of the refusal: `ERR_OUT_OF_RANGE` for months ahead outside 1 to 12,
/// or a history that is not the full months before `current`; `ERR_OVERFLOW` for a figure above
/// the largest allowed amount. Dues that would fall below 0 are the result `FallsBelowZero`.
pub fn balance_at_pace(
    kind: BalanceKind,
    now: Paisa,
    changes: &[MonthValue],
    current: YearMonth,
    ahead: u32,
) -> Result<BalanceForecast, ServiceError> {
    match forecast::balance_at_pace(kind, now, changes, current, ahead) {
        Ok(found) => Ok(BalanceForecast::Estimate(found)),
        Err(ForecastError::TooLittleHistory { months }) => Ok(BalanceForecast::TooShort {
            full_months: months,
        }),
        Err(ForecastError::BelowZero) => Ok(BalanceForecast::FallsBelowZero),
        Err(error) => Err(ServiceError::reject(error.code())),
    }
}
