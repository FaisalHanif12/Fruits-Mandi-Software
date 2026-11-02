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
//! A what-if (plan 52, Umar 02-11-2026) is a fourth line, worked only when same pace is: the same
//! pace figure scaled by a factor the Owner named in his question ("if the pace doubles"): the
//! scaled pace is `round_to_rupee(pace × factor)`, once, through `apply_bps`, and its total is
//! that times the months, so the shown figures always agree. "Until the end of this year" is the
//! months from the current month to December (`months_to_year_end`), and its answer is the whole
//! year: the months of this year so far, read from the books and never estimated, plus the months
//! left (`estimate_to_year_end`, plan 52, Umar 02-11-2026). The current month is counted as a
//! whole estimated month; its days so far are not added.
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
use mandi_core::money::{Bps, MAX_RUPEES, MoneyError, Paisa, apply_bps};
use thiserror::Error;

use crate::figures::{CalcOp, Dir, FigureError, Op, Step, Value, calculate, divided_by, times};

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
/// 100% in bps.
const ONE: i32 = 10_000;
/// The biggest what-if factor, in bps of 100%: 10 times the pace.
pub const MAX_FACTOR_BPS: i32 = 100_000;

/// The factor of a what-if on the pace, in bps of 100% (20,000 is 200%, "the pace doubles").
/// From 1 to [`MAX_FACTOR_BPS`], never 10,000 (the same pace again).
///
/// ```
/// use mandi_domain::forecast::PaceFactor;
/// assert_eq!(PaceFactor::DOUBLE.bps(), 20_000);
/// assert_eq!(PaceFactor::up(1_000).unwrap().bps(), 11_000); // up 10%
/// assert_eq!(PaceFactor::down(2_500).unwrap().bps(), 7_500); // down 25%
/// assert!(PaceFactor::down(10_000).is_err()); // down 100% is nothing
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaceFactor(i32);

impl PaceFactor {
    /// The pace doubles.
    pub const DOUBLE: PaceFactor = PaceFactor(20_000);
    /// The pace halves.
    pub const HALF: PaceFactor = PaceFactor(5_000);
    /// The pace triples.
    pub const TRIPLE: PaceFactor = PaceFactor(30_000);

    /// The pace goes up by `bps` of itself (1,000 is up 10%): 1 to 90,000.
    ///
    /// # Errors
    /// `FactorOutOfRange`.
    pub fn up(bps: i32) -> Result<PaceFactor, ForecastError> {
        if !(1..=MAX_FACTOR_BPS.saturating_sub(ONE)).contains(&bps) {
            return Err(ForecastError::FactorOutOfRange);
        }
        ONE.checked_add(bps)
            .map(PaceFactor)
            .ok_or(ForecastError::FactorOutOfRange)
    }

    /// The pace goes down by `bps` of itself (2,500 is down 25%): 1 to 9,999.
    ///
    /// # Errors
    /// `FactorOutOfRange`.
    pub fn down(bps: i32) -> Result<PaceFactor, ForecastError> {
        if !(1..ONE).contains(&bps) {
            return Err(ForecastError::FactorOutOfRange);
        }
        ONE.checked_sub(bps)
            .map(PaceFactor)
            .ok_or(ForecastError::FactorOutOfRange)
    }

    /// The factor in bps of 100%, for example 20,000 for double.
    pub fn bps(self) -> i32 {
        self.0
    }
}

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
    /// A what-if: the same pace scaled by the factor of the question ("if the pace doubles").
    WhatIfPace,
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
    /// A what-if: the factor the pace was scaled by.
    pub factor: Option<PaceFactor>,
    /// To the end of the year: the months of the year so far from the books plus `total`. `None`
    /// for any other estimate.
    pub year_total: Option<Paisa>,
}

/// The months of the year before the current month, read from the books (never estimated): the
/// "so far" of a whole-year total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YearSoFar {
    /// Their sum.
    pub so_far: Paisa,
    /// How many months of the books are in it: January to the month before the current one, or
    /// from the first month of the books. In an estimate to the end of the year it is 0 only in
    /// January, because the history always ends the month before the current one.
    pub months: u32,
    /// The first and the last of those months; `None` when there are none.
    pub span: Option<(YearMonth, YearMonth)>,
    /// How the sum was made: one `Sum` step over the months, so the Owner can check it (PRD
    /// FR-7.43). `None` with fewer than 2 months, where the sum is the month itself.
    pub step: Option<Step>,
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
    /// To the end of the year: the months of the year so far from the books. `None` for any other
    /// estimate.
    pub year: Option<YearSoFar>,
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
    /// A what-if factor outside 1 to 100,000 bps, or 10,000 (the same pace again).
    #[error("the factor must be from 0.01% to 1,000%, and not 100%")]
    FactorOutOfRange,
    /// A what-if on a pace below 0 (a loss): up or down of a loss would be a guess.
    #[error("a what-if needs a pace of 0 or more")]
    NegativePace,
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
            | ForecastError::HistoryNotConsecutive
            | ForecastError::FactorOutOfRange
            | ForecastError::NegativePace => ErrorCode::ErrOutOfRange,
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
    estimate_with(history, current, ahead, target, None)
}

/// [`estimate`], and with `what_if` a fourth line right after same pace: the same pace scaled by
/// the factor, worked from the same-pace figure and never from the history a second time. A
/// what-if on a pace below 0 is refused.
///
/// # Errors
/// See [`ForecastError`].
pub fn estimate_with(
    history: &[MonthValue],
    current: YearMonth,
    ahead: u32,
    target: Option<Paisa>,
    what_if: Option<PaceFactor>,
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
    let pace = same_pace(&ctx)?;
    let mut methods = vec![];
    if let Some(factor) = what_if {
        methods.push(pace.clone());
        methods.push(what_if_pace(&ctx, &pace, factor)?);
    } else {
        methods.push(pace);
    }
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
        year: None,
    })
}

/// The months of the year of `current` before it, summed from `history` (the full months before
/// the current month): the books' part of a whole-year total. A month is counted by its place, not
/// by its value: a month with 0 is still a month of the books.
///
/// # Errors
/// `Overflow` for a sum that does not fit.
pub fn year_so_far(history: &[MonthValue], current: YearMonth) -> Result<YearSoFar, ForecastError> {
    let mut values: Vec<Value> = Vec::new();
    let mut span: Option<(YearMonth, YearMonth)> = None;
    for item in history
        .iter()
        .filter(|h| h.month.year == current.year && h.month < current)
    {
        values.push(Value::Money(item.value));
        span = Some(match span {
            Some((first, _)) => (first, item.month),
            None => (item.month, item.month),
        });
    }
    let months = u32::try_from(values.len()).map_err(|_| FigureError::Overflow)?;
    // One month is itself; two or more are one `Sum` step, kept as the working of the sum.
    let (so_far, step) = match values.as_slice() {
        [] => (Paisa::ZERO, None),
        [only] => (result_money(*only)?, None),
        many => {
            let sum = calculate(CalcOp::Sum, many)?;
            (result_money(sum.result)?, sum.steps.into_iter().next())
        }
    };
    Ok(YearSoFar {
        so_far,
        months,
        span,
        step,
    })
}

/// [`estimate_with`] to the end of the year of `current` (October gives 3 months ahead), and the
/// whole year on every method: the months of this year so far from the books plus its months ahead,
/// as the last step of its working (`so_far + total`, after the `Sum` step of the books' months). With no month so far (January) the whole
/// year is the total and no step is added. The books' months end the month before `current` and the
/// months ahead start at `current`, so none is counted twice. There is no target: a year target
/// could count the books' months or not, and that is the Owner's to say.
///
/// Example (E2E-59, today 10-10-26): 261,000 so far; same pace 48,000 × 3 = 144,000; the whole
/// year 405,000.
///
/// # Errors
/// See [`ForecastError`].
pub fn estimate_to_year_end(
    history: &[MonthValue],
    current: YearMonth,
    what_if: Option<PaceFactor>,
) -> Result<Estimate, ForecastError> {
    let ahead = months_to_year_end(current)?;
    let mut found = estimate_with(history, current, ahead, None, what_if)?;
    let year = year_so_far(history, current)?;
    // The year steps are added only now: a what-if copies the working of same pace, and must not
    // take its year step with it.
    for method in &mut found.methods {
        if year.months == 0 {
            method.year_total = Some(method.total);
            continue;
        }
        // The books' sum first (when it has two or more months), then the whole year.
        if let Some(step) = &year.step {
            method.steps.push(step.clone());
        }
        let whole = calculate(
            CalcOp::Sum,
            &[Value::Money(year.so_far), Value::Money(method.total)],
        )?;
        method.year_total = Some(result_money(whole.result)?);
        method.steps.extend(whole.steps);
    }
    found.year = Some(year);
    Ok(found)
}

/// The months from `current` to `last`, both counted: October to December is 3 (the current month
/// is the first month ahead, as in E2E-59).
///
/// # Errors
/// `AheadOutOfRange` when `last` is before `current` or more than 12 months are counted.
pub fn months_through(current: YearMonth, last: YearMonth) -> Result<u32, ForecastError> {
    let index = |m: YearMonth| {
        i64::from(m.year)
            .checked_mul(12)
            .and_then(|y| y.checked_add(i64::from(m.month)))
    };
    let count = index(last)
        .zip(index(current))
        .and_then(|(l, c)| l.checked_sub(c))
        .and_then(|d| d.checked_add(1))
        .ok_or(ForecastError::AheadOutOfRange)?;
    u32::try_from(count)
        .ok()
        .filter(|c| (1..=MAX_MONTHS_AHEAD).contains(c))
        .ok_or(ForecastError::AheadOutOfRange)
}

/// The months left in the year of `current`, the current month counted: October gives 3, December
/// gives 1, January gives 12.
///
/// # Errors
/// `AheadOutOfRange` for a year that has no December.
pub fn months_to_year_end(current: YearMonth) -> Result<u32, ForecastError> {
    let december = YearMonth::new(current.year, 12).ok_or(ForecastError::AheadOutOfRange)?;
    months_through(current, december)
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
        factor: None,
        year_total: None,
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

/// The pace scaled by a factor: `round_half_up(pace × factor ÷ 10,000)`, rounded once.
///
/// `Bps` holds only 0 to 10,000, so the factor is split: `factor = whole × 10,000 + rest`, and
/// the result is `pace × whole` (exact) plus `apply_bps(pace, rest)` (the one rounding, half up,
/// by the shared function). Whole rupees added to a half-up rounded figure give the same result as
/// rounding the whole product, so double and triple are exact and 48,333 down 50% is 24,167.
/// The step is `pace × factor%` with the rounded result, so the Owner can check it.
fn scale_pace(per_month: Paisa, factor: PaceFactor) -> Result<Step, ForecastError> {
    if per_month < Paisa::ZERO {
        return Err(ForecastError::NegativePace);
    }
    let f = i64::from(factor.0);
    let whole = f.checked_div(i64::from(ONE)).ok_or(FigureError::Overflow)?;
    let rest = f.checked_rem(i64::from(ONE)).ok_or(FigureError::Overflow)?;
    let rest = i32::try_from(rest).map_err(|_| FigureError::Overflow)?;
    let exact = per_month
        .as_paisa()
        .checked_mul(whole)
        .ok_or(FigureError::Overflow)?;
    let fraction = apply_bps(per_month, Bps::new(rest)?)?;
    let scaled = exact
        .checked_add(fraction.as_paisa())
        .ok_or(FigureError::Overflow)?;
    let scaled = Paisa::try_from_paisa(scaled)?;
    // A scaled month above the largest allowed amount is refused. A total (the month times at most
    // 12 months) still fits a 64-bit figure and a safe JavaScript integer, as same pace always did.
    let largest = Paisa::from_rupees(MAX_RUPEES)?;
    if scaled > largest {
        return Err(FigureError::Overflow.into());
    }
    Ok(Step {
        op: Op::Times,
        inputs: vec![Value::Money(per_month), Value::Percent(f)],
        result: Value::Money(scaled),
    })
}

/// The what-if line: the same-pace figure scaled by `factor`, for each month ahead.
fn what_if_pace(
    ctx: &Context<'_>,
    pace: &MethodEstimate,
    factor: PaceFactor,
) -> Result<MethodEstimate, ForecastError> {
    let per_month = pace.per_month.ok_or(ForecastError::HistoryNotConsecutive)?;
    let used = tail(ctx, PACE_MONTHS)?;
    let scale = scale_pace(per_month, factor)?;
    let scaled = result_money(scale.result)?;
    let multiplied = times(scaled, i64::from(ctx.ahead))?;
    let total = result_money(multiplied.result)?;
    // The average step of same pace, then the scaling, then the months.
    let mut steps: Vec<Step> = pace
        .steps
        .iter()
        .take(pace.steps.len().saturating_sub(1))
        .cloned()
        .collect();
    steps.push(scale);
    steps.push(multiplied);
    let mut found = finish(
        ctx,
        Parts {
            method: Method::WhatIfPace,
            used,
            months: vec![scaled; ahead_count(ctx)],
            steps,
            total: Some(total),
            per_month: Some(scaled),
            step: None,
        },
    )?;
    found.factor = Some(factor);
    Ok(found)
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
