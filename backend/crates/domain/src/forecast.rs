//! Estimates of the months ahead (PRD FR-7.86, Code Standards M14, money agent Section 9.1).
//!
//! The model asks the estimator; it never makes up a future figure. An estimate is worked out from
//! the **full calendar months** in the books (the current month is not finished, so it is left
//! out) by three fixed methods, in whole rupees, with checked maths. It is **display only**: never
//! posted, saved, or added to a report or a tile.
//!
//! | Method | Needs | Month k ahead (k = 1 is the current month) |
//! |---|---|---|
//! | Same pace | 3 full months | the average of the last 3, rounded once (`avg_rate`) |
//! | Same growth | 6 full months | `max(0, last + k × change)`, change = `|last − first| ÷ 5` over the last 6, with the sign of `last − first` |
//! | Same season | 12 full months | the same month of last year, as stored |
//!
//! The total of same pace is the per-month figure times the months, so the shown figures always
//! agree. The total of the other two is the sum of their months. With a target, the target month
//! is the first month ahead whose running total reaches the target (at least the target).
//!
//! Example (E2E-59, today 10-10-26, April to September 2026: 36,000; 39,000; 42,000; 45,000;
//! 48,000; 51,000): same pace is 48,000 a month, 288,000 in 6 months; same growth is
//! (51,000 − 36,000) ÷ 5 = 3,000 more each month: 54,000 + 57,000 + 60,000 + 63,000 + 66,000 +
//! 69,000 = 369,000; same season is not worked out (6 months, it needs 12). With a target of
//! 3,00,000 over 12 months, same pace reaches it in the 7th month (April 2027), same growth in the
//! 5th (February 2027).
//!
//! ```
//! use mandi_domain::core::money::Paisa;
//! use mandi_domain::forecast::{Method, MonthValue, YearMonth, estimate};
//! let rs = |r| Paisa::from_rupees(r).unwrap();
//! let history: Vec<MonthValue> = [36_000, 39_000, 42_000, 45_000, 48_000, 51_000]
//!     .iter()
//!     .enumerate()
//!     .map(|(i, v)| MonthValue {
//!         month: YearMonth::new(2026, 4 + i as u32).unwrap(),
//!         value: rs(*v),
//!     })
//!     .collect();
//! let current = YearMonth::new(2026, 10).unwrap();
//! let e = estimate(&history, current, 6, None).unwrap();
//! assert_eq!(e.methods.len(), 2);
//! assert_eq!(e.methods[0].method, Method::SamePace);
//! assert_eq!(e.methods[0].total, rs(288_000));
//! assert_eq!(e.methods[1].total, rs(369_000));
//! ```

use mandi_core::error::ErrorCode;
use mandi_core::money::{MoneyError, Paisa};
use thiserror::Error;

use crate::figures::{CalcOp, Dir, FigureError, Step, Value, calculate, divided_by, times};

/// Most months ahead an estimate looks (PRD FR-7.86).
pub const MAX_MONTHS_AHEAD: u32 = 12;
/// Full months the same pace needs.
pub const PACE_MONTHS: usize = 3;
/// Full months the same growth needs.
pub const GROWTH_MONTHS: usize = 6;
/// Full months the same season needs.
pub const SEASON_MONTHS: usize = 12;
/// Most months of history one estimate takes.
const MAX_HISTORY: usize = 1_200;
/// The steps between the first and the last of the growth months.
const GROWTH_STEPS: i64 = 5;

/// A calendar month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YearMonth {
    year: i32,
    month: u32,
}

impl YearMonth {
    /// A month of a year: the year 1000 to 9999, the month 1 to 12.
    pub fn new(year: i32, month: u32) -> Option<YearMonth> {
        if (1000..=9999).contains(&year) && (1..=12).contains(&month) {
            Some(YearMonth { year, month })
        } else {
            None
        }
    }

    /// The year, for example 2026.
    pub fn year(self) -> i32 {
        self.year
    }

    /// The month, 1 to 12.
    pub fn month(self) -> u32 {
        self.month
    }

    /// The month `n` months later (October 2026 plus 6 is April 2027).
    pub fn plus_months(self, n: u32) -> Option<YearMonth> {
        let index = i64::from(self.year)
            .checked_mul(12)?
            .checked_add(i64::from(self.month.checked_sub(1)?))?
            .checked_add(i64::from(n))?;
        let year = i32::try_from(index.checked_div_euclid(12)?).ok()?;
        let month = u32::try_from(index.checked_rem_euclid(12)?.checked_add(1)?).ok()?;
        YearMonth::new(year, month)
    }
}

/// The total of one full month of a measure (sales, commission, expenses ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthValue {
    /// The month.
    pub month: YearMonth,
    /// Its total. A month with nothing in it is 0.
    pub value: Paisa,
}

/// The three methods of PRD FR-7.86.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// اسی رفتار سے.
    SamePace,
    /// اسی اضافے سے.
    SameGrowth,
    /// پچھلے سال جیسا.
    SameSeason,
}

/// The months ahead by one method, with its working.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodEstimate {
    /// The method.
    pub method: Method,
    /// How many full months it used.
    pub months_used: u32,
    /// The first of the months it used.
    pub used_from: YearMonth,
    /// The last of the months it used.
    pub used_to: YearMonth,
    /// Same pace: the figure of each month.
    pub per_month: Option<Paisa>,
    /// Same growth: the change for each month, its size and its direction.
    pub step: Option<(Paisa, Dir)>,
    /// The figure of each month ahead, the current month first.
    pub months: Vec<Paisa>,
    /// The total of the months ahead.
    pub total: Paisa,
    /// With a target: the first month whose running total is at least the target; `None` if no
    /// month ahead reaches it (or no target was asked).
    pub target_month: Option<YearMonth>,
    /// The working, built by the same code that worked the figures out.
    pub steps: Vec<Step>,
}

/// An estimate: every method that could be worked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Estimate {
    /// The first month ahead: the current month.
    pub first: YearMonth,
    /// The last month ahead.
    pub last: YearMonth,
    /// How many months ahead.
    pub ahead: u32,
    /// How many full months the books gave.
    pub full_months: u32,
    /// Same pace, then same growth, then same season, the ones with enough history.
    pub methods: Vec<MethodEstimate>,
}

/// Why an estimate was refused. None is shown to the Owner as an error: a history that is too
/// short becomes a "too short" result, the rest go back to the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ForecastError {
    /// Fewer than 3 full months: no method can be worked out.
    #[error("too little history: {months} full months")]
    TooLittleHistory {
        /// How many full months there are.
        months: u32,
    },
    /// Months ahead is 0 or more than 12.
    #[error("months ahead must be 1 to 12")]
    AheadOutOfRange,
    /// The target is 0 or less.
    #[error("the target must be more than 0")]
    TargetNotPositive,
    /// The history is not consecutive months ending the month before the current one.
    #[error("the history is not the full months before the current month")]
    HistoryNotConsecutive,
    /// A calculation failed.
    #[error(transparent)]
    Figure(#[from] FigureError),
}

impl ForecastError {
    /// The error code of the tool result.
    pub fn code(self) -> ErrorCode {
        match self {
            ForecastError::TooLittleHistory { .. } => ErrorCode::ErrTooLittleHistory,
            ForecastError::AheadOutOfRange
            | ForecastError::TargetNotPositive
            | ForecastError::HistoryNotConsecutive => ErrorCode::ErrOutOfRange,
            ForecastError::Figure(error) => error.code(),
        }
    }
}

impl From<MoneyError> for ForecastError {
    fn from(error: MoneyError) -> Self {
        ForecastError::Figure(FigureError::from(error))
    }
}

/// Works out the months ahead from the full months before `current`.
///
/// `history` is oldest first: consecutive calendar months, the last one the month before
/// `current` (a month with no sales is 0; the caller fills it). `ahead` is 1 to 12; the current
/// month is the first month ahead. `target`, when given, must be more than 0.
///
/// # Errors
/// See [`ForecastError`].
pub fn estimate(
    history: &[MonthValue],
    current: YearMonth,
    ahead: u32,
    target: Option<Paisa>,
) -> Result<Estimate, ForecastError> {
    if ahead == 0 || ahead > MAX_MONTHS_AHEAD {
        return Err(ForecastError::AheadOutOfRange);
    }
    if target.is_some_and(|t| t <= Paisa::ZERO) {
        return Err(ForecastError::TargetNotPositive);
    }
    check_history(history, current)?;
    let n = history.len();
    let full_months = u32::try_from(n).map_err(|_| FigureError::Overflow)?;
    if n < PACE_MONTHS {
        return Err(ForecastError::TooLittleHistory {
            months: full_months,
        });
    }
    let last_month = current
        .plus_months(ahead.saturating_sub(1))
        .ok_or(FigureError::Overflow)?;
    let ctx = Context {
        history,
        current,
        ahead,
        target,
    };
    let mut methods = vec![same_pace(&ctx)?];
    if n >= GROWTH_MONTHS {
        methods.push(same_growth(&ctx)?);
    }
    if n >= SEASON_MONTHS {
        methods.push(same_season(&ctx)?);
    }
    Ok(Estimate {
        first: current,
        last: last_month,
        ahead,
        full_months,
        methods,
    })
}

struct Context<'a> {
    history: &'a [MonthValue],
    current: YearMonth,
    ahead: u32,
    target: Option<Paisa>,
}

fn check_history(history: &[MonthValue], current: YearMonth) -> Result<(), ForecastError> {
    if history.len() > MAX_HISTORY {
        return Err(ForecastError::HistoryNotConsecutive);
    }
    let mut expected = match history.first() {
        Some(first) => first.month,
        None => return Ok(()),
    };
    for item in history {
        if item.month != expected {
            return Err(ForecastError::HistoryNotConsecutive);
        }
        expected = expected
            .plus_months(1)
            .ok_or(ForecastError::HistoryNotConsecutive)?;
    }
    // After the last month comes the current month.
    if expected != current {
        return Err(ForecastError::HistoryNotConsecutive);
    }
    Ok(())
}

/// The last `count` months of the history.
fn tail<'a>(ctx: &Context<'a>, count: usize) -> Result<&'a [MonthValue], ForecastError> {
    let from = ctx
        .history
        .len()
        .checked_sub(count)
        .ok_or(ForecastError::HistoryNotConsecutive)?;
    ctx.history
        .get(from..)
        .ok_or(ForecastError::HistoryNotConsecutive)
}

fn money(values: &[MonthValue]) -> Vec<Value> {
    values.iter().map(|m| Value::Money(m.value)).collect()
}

fn result_money(value: Value) -> Result<Paisa, ForecastError> {
    match value {
        Value::Money(p) => Ok(p),
        _ => Err(FigureError::MixedKinds.into()),
    }
}

fn span(used: &[MonthValue]) -> Result<(YearMonth, YearMonth), ForecastError> {
    match (used.first(), used.last()) {
        (Some(a), Some(b)) => Ok((a.month, b.month)),
        _ => Err(ForecastError::HistoryNotConsecutive),
    }
}

fn used_count(used: &[MonthValue]) -> Result<u32, ForecastError> {
    u32::try_from(used.len()).map_err(|_| FigureError::Overflow.into())
}

/// What one method worked out, before its total and target month are settled.
struct Parts<'a> {
    method: Method,
    /// The months of the history it used.
    used: &'a [MonthValue],
    /// The figure of each month ahead.
    months: Vec<Paisa>,
    /// The working so far.
    steps: Vec<Step>,
    /// The total, when the method has worked it out (same pace); otherwise it is the sum of the
    /// months, added to the working as its last step.
    total: Option<Paisa>,
    per_month: Option<Paisa>,
    step: Option<(Paisa, Dir)>,
}

/// Settles the total (the sum of the months, as the last step of the working) and the target month.
fn finish(ctx: &Context<'_>, parts: Parts<'_>) -> Result<MethodEstimate, ForecastError> {
    let Parts {
        method,
        used,
        months,
        mut steps,
        total,
        per_month,
        step,
    } = parts;
    let total = match total {
        Some(total) => total,
        None => {
            let values: Vec<Value> = months.iter().map(|p| Value::Money(*p)).collect();
            let sum = calculate(CalcOp::Sum, &values)?;
            steps.extend(sum.steps);
            result_money(sum.result)?
        }
    };
    let (used_from, used_to) = span(used)?;
    Ok(MethodEstimate {
        method,
        months_used: used_count(used)?,
        used_from,
        used_to,
        per_month,
        step,
        target_month: target_month(ctx, &months)?,
        months,
        total,
        steps,
    })
}

/// The first month ahead whose running total is at least the target.
fn target_month(ctx: &Context<'_>, months: &[Paisa]) -> Result<Option<YearMonth>, ForecastError> {
    let Some(target) = ctx.target else {
        return Ok(None);
    };
    let mut running = Paisa::ZERO;
    for (k, value) in months.iter().enumerate() {
        running = running.checked_add(*value)?;
        if running >= target {
            let k = u32::try_from(k).map_err(|_| FigureError::Overflow)?;
            return Ok(Some(
                ctx.current.plus_months(k).ok_or(FigureError::Overflow)?,
            ));
        }
    }
    Ok(None)
}

fn ahead_count(ctx: &Context<'_>) -> usize {
    usize::try_from(ctx.ahead).unwrap_or(0)
}

fn same_pace(ctx: &Context<'_>) -> Result<MethodEstimate, ForecastError> {
    let used = tail(ctx, PACE_MONTHS)?;
    let average = calculate(CalcOp::Average, &money(used))?;
    let per_month = result_money(average.result)?;
    let multiplied = times(per_month, i64::from(ctx.ahead))?;
    let total = result_money(multiplied.result)?;
    let months = vec![per_month; ahead_count(ctx)];
    let mut steps = average.steps;
    steps.push(multiplied);
    finish(
        ctx,
        Parts {
            method: Method::SamePace,
            used,
            months,
            steps,
            total: Some(total),
            per_month: Some(per_month),
            step: None,
        },
    )
}

fn same_growth(ctx: &Context<'_>) -> Result<MethodEstimate, ForecastError> {
    let used = tail(ctx, GROWTH_MONTHS)?;
    let (first, last) = match (used.first(), used.last()) {
        (Some(a), Some(b)) => (a.value, b.value),
        _ => return Err(ForecastError::HistoryNotConsecutive),
    };
    let difference = calculate(
        CalcOp::Difference,
        &[Value::Money(last), Value::Money(first)],
    )?;
    let change_total = result_money(difference.result)?;
    let divided = divided_by(change_total, GROWTH_STEPS)?;
    let change = result_money(divided.result)?;
    let mut months = Vec::new();
    for k in 1..=i64::from(ctx.ahead) {
        let moved = change
            .as_paisa()
            .checked_mul(k)
            .ok_or(FigureError::Overflow)?;
        let month = last
            .as_paisa()
            .checked_add(moved)
            .ok_or(FigureError::Overflow)?;
        // A month is never below 0 (PRD FR-7.86).
        months.push(Paisa::try_from_paisa(month.max(0))?);
    }
    let size = Paisa::try_from_paisa(
        change
            .as_paisa()
            .checked_abs()
            .ok_or(FigureError::Overflow)?,
    )?;
    let mut steps = difference.steps;
    steps.push(divided);
    finish(
        ctx,
        Parts {
            method: Method::SameGrowth,
            used,
            months,
            steps,
            total: None,
            per_month: None,
            step: Some((size, Dir::of(change))),
        },
    )
}

fn same_season(ctx: &Context<'_>) -> Result<MethodEstimate, ForecastError> {
    // Month k ahead is the same month one year earlier: the first `ahead` months of the last 12.
    let year = tail(ctx, SEASON_MONTHS)?;
    let used = year
        .get(..ahead_count(ctx))
        .ok_or(ForecastError::AheadOutOfRange)?;
    let months: Vec<Paisa> = used.iter().map(|m| m.value).collect();
    finish(
        ctx,
        Parts {
            method: Method::SameSeason,
            used,
            months,
            steps: Vec::new(),
            total: None,
            per_month: None,
            step: None,
        },
    )
}
