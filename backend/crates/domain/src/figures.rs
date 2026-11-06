//! The question panel's calculator and the numbers read from a question (PRD FR-7.75, FR-7.76,
//! FR-7.86, Code Standards M14, money agent Section 9.1).
//!
//! The model never does arithmetic. It asks `calculate`, and the answer is worked out here, in
//! whole rupees, with checked maths. Everything in this file is **display only**, like
//! [`avg_rate`]: a result is never posted, saved, or added to a report or a tile.
//!
//! - [`calculate`]: a sum, a difference (first minus second), an average, a percent of (first as a
//!   percent of second) or a change in percent (from first to second) of figures already read.
//!   Each result carries its working ([`Step`]), built here by the same code that works the
//!   figure out (Code Standards B30).
//! - [`question_numbers`]: the numbers the Owner typed ("3 lakh" is 3,00,000, "agle 6 mahine" is 6
//!   months, "10%" is 10 percent), each with its unit. A number the model passes for months ahead,
//!   last N days or a بکری number must be one of these, with the right unit.
//!
//! Three kinds of figure, never mixed in one calculation:
//!
//! | Kind | What it is | Signed? |
//! |---|---|---|
//! | [`Value::Money`] | an amount (a total, a commission, a profit) | yes: a loss is below 0 |
//! | [`Value::Balance`] | a party's balance: بنام positive, جمع negative | yes: the sign is the side |
//! | [`Value::Count`] | a whole number: بکری, نگ, days | yes |
//!
//! A sum or an average of balances is signed (2,880 بنام and 10,160 جمع is 7,280 جمع). A
//! difference, a percent of or a change of two balances works on their sizes and needs both on the
//! same side: 10,160 جمع less 7,280 جمع is 2,880 **more**, not "less" because the signs are below
//! 0. Balances on opposite sides are refused for these three (use a sum to net them).
//!
//! Rounding: an average is `avg_rate` (nearest rupee, half up, on the magnitude; a negative total
//! is averaged as positive and then negated, as splits are). A count is averaged the same way, as
//! rupees, so there is one rounding function. A percent is in hundredths of a percent through
//! [`hundredths`], the "% of all" formula of the analytics views. Nothing here uses a float.
//!
//! Example (E2E-61): `calculate(Sum, [2,880 Dr, 18,684 Dr])` is 288,000 + 1,868,400 = 2,156,400
//! paisa, which is 21,564 rupees, بنام. Example (E2E-59): `change_percent(48,000, 51,000)` is 625
//! hundredths, 6.25%, up.
//!
//! ```
//! use mandi_domain::core::money::Paisa;
//! use mandi_domain::figures::{CalcOp, Value, calculate};
//! let rs = |r| Value::Balance(Paisa::from_rupees(r).unwrap());
//! let sum = calculate(CalcOp::Sum, &[rs(2_880), rs(18_684)]).unwrap();
//! assert_eq!(sum.result, rs(21_564));
//! // A balance of 2,880 Dr and one of 10,160 Cr: 2,880 - 10,160 = 7,280 Cr (negative).
//! let net = calculate(CalcOp::Sum, &[rs(2_880), rs(-10_160)]).unwrap();
//! assert_eq!(net.result, rs(-7_280));
//! ```

use std::cmp::Ordering;

use mandi_core::error::ErrorCode;
use mandi_core::money::{Bps, MAX_RUPEES, MoneyError, Paisa, Qty, apply_bps, avg_rate, hundredths};
use thiserror::Error;

/// Most inputs one calculation takes (a table of 100 rows at most is summed in the tool).
pub const MAX_INPUTS: usize = 1_000;
/// Most numbers read from one question.
pub const MAX_QUESTION_NUMBERS: usize = 20;
/// Most digits of one number read from a question (so it always fits an `i64`).
const MAX_DIGITS: usize = 18;
/// Most decimals read after a number that has a lakh, hazar or crore after it.
const MAX_DECIMALS: usize = 6;

/// A figure the calculator takes and gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    /// An amount in whole rupees, signed paisa (a loss is below 0).
    Money(Paisa),
    /// A party's balance in whole rupees, signed paisa: بنام positive, جمع negative. The caller
    /// gives a result its side from its sign.
    Balance(Paisa),
    /// A whole number: a count of بکری, نگ or days.
    Count(i64),
    /// A percent in hundredths of a percent, 0 or more (12.50% is 1,250). Only a result, never an
    /// input of `calculate`; the working of an estimate's what-if has it as an input of its
    /// scaling step (`48,000 × 200.00%`).
    Percent(i64),
}

/// The direction of a difference or a change: the app puts the word زیادہ, کم or برابر next to the
/// figure, so the model never chooses it (PRD FR-7.75).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    /// More than before.
    Up,
    /// Less than before.
    Down,
    /// No change.
    Same,
}

impl Dir {
    /// The direction of a signed change: above 0 is up, below 0 is down.
    pub fn of(change: Paisa) -> Dir {
        Dir::of_i128(i128::from(change.as_paisa()))
    }

    fn of_i128(change: i128) -> Dir {
        match change.cmp(&0) {
            Ordering::Greater => Dir::Up,
            Ordering::Less => Dir::Down,
            Ordering::Equal => Dir::Same,
        }
    }
}

/// What the model may ask the calculator to do (the tool `calculate`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcOp {
    /// `a + b + c`, 1 to 1,000 inputs.
    Sum,
    /// `first − second`, exactly 2 inputs.
    Difference,
    /// `(a + b + c) ÷ n`, 1 to 1,000 inputs.
    Average,
    /// `first ÷ second × 100`, exactly 2 inputs.
    PercentOf,
    /// `(second − first) ÷ first × 100`, exactly 2 inputs.
    ChangePercent,
    /// `first × (100% + percent)`: an amount and a percent more, exactly 2 inputs (an amount, then
    /// a percent). The one rounding is the shared one, through [`scale_amount`].
    PercentMore,
    /// `first × (100% − percent)`: an amount and a percent less, exactly 2 inputs.
    PercentLess,
}

/// One kind of step in a working. The first five are the [`CalcOp`]s; `Times` and `DividedBy` are
/// used inside an estimate's working (per month × months, a difference ÷ months).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// A sum.
    Sum,
    /// A difference.
    Difference,
    /// An average.
    Average,
    /// A percent of.
    PercentOf,
    /// A change in percent.
    ChangePercent,
    /// `first × second`.
    Times,
    /// `first ÷ second`, rounded to the nearest rupee, half up.
    DividedBy,
}

/// One step of a working: its inputs in order and its result (PRD FR-7.43).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// What the step does.
    pub op: Op,
    /// The inputs.
    pub inputs: Vec<Value>,
    /// The result.
    pub result: Value,
}

/// The answer of [`calculate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calc {
    /// The result. A percent result is never negative: the direction is in `dir`. A difference of
    /// two balances is an amount (`Money`), not a balance. A difference is **signed** here
    /// (48,000 − 51,000 is −3,000, `Down`); the answer shows its size with the direction word
    /// ("3,000 کم"), never the minus sign and the word together.
    pub result: Value,
    /// For a difference or a change in percent: its direction. Null for the others.
    pub dir: Option<Dir>,
    /// The working: one step.
    pub steps: Vec<Step>,
}

/// Why a calculation was refused. None of these is shown to the Owner: the tool returns it to the
/// model as an error result, and the model may correct its input once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum FigureError {
    /// A sum or product does not fit.
    #[error("the result does not fit")]
    Overflow,
    /// No input at all.
    #[error("no inputs")]
    NoInputs,
    /// More than [`MAX_INPUTS`] inputs.
    #[error("too many inputs")]
    TooManyInputs,
    /// A difference, percent of or change needs exactly two inputs.
    #[error("this needs exactly two inputs")]
    NeedsTwoInputs,
    /// Money, balances and counts in one calculation.
    #[error("amounts, balances and counts cannot be mixed")]
    MixedKinds,
    /// A percent cannot be an input.
    #[error("a percent cannot be an input")]
    PercentInput,
    /// A percent of 0 (the base is 0), or a change from 0.
    #[error("the base is 0")]
    NoBase,
    /// Two figures on opposite sides (one بنام and one جمع, or a profit and a loss in a percent
    /// of) where that means nothing.
    #[error("the two figures are on opposite sides")]
    MixedSigns,
    /// A percent more or less of a figure below 0 (a loss): up or down of a loss would be a guess.
    #[error("the figure is below 0")]
    NegativeBase,
    /// A percent more or less outside 0.01% to 900% (more) or 0.01% to 99.99% (less).
    #[error("the percent is out of range")]
    PercentOutOfRange,
    /// A caller bug (a value that is not a whole rupee, a count of 0). Never the Owner's input.
    #[error("internal")]
    Internal,
}

impl FigureError {
    /// The error code the tool result carries.
    pub fn code(self) -> ErrorCode {
        match self {
            FigureError::Overflow => ErrorCode::ErrOverflow,
            FigureError::Internal => ErrorCode::ErrInternal,
            FigureError::NoBase => ErrorCode::ErrNoBase,
            FigureError::NoInputs
            | FigureError::TooManyInputs
            | FigureError::NeedsTwoInputs
            | FigureError::MixedKinds
            | FigureError::PercentInput
            | FigureError::NegativeBase
            | FigureError::PercentOutOfRange
            | FigureError::MixedSigns => ErrorCode::ErrOutOfRange,
        }
    }
}

impl From<MoneyError> for FigureError {
    fn from(error: MoneyError) -> Self {
        // A failure of a core function is an overflow, or a caller bug (a count of 0, a value that
        // is not a whole rupee) that the checks here make impossible; both end as a refusal, never
        // a wrong figure.
        match error {
            MoneyError::Overflow => FigureError::Overflow,
            _ => FigureError::Internal,
        }
    }
}

/// Works out one calculation, with its working.
///
/// - `Sum`, `Average`: 1 to 1,000 inputs, all of one kind.
/// - `Difference`, `PercentOf`, `ChangePercent`: exactly 2 inputs, both of one kind.
///
/// # Errors
/// See [`FigureError`].
pub fn calculate(op: CalcOp, inputs: &[Value]) -> Result<Calc, FigureError> {
    if inputs.is_empty() {
        return Err(FigureError::NoInputs);
    }
    if inputs.len() > MAX_INPUTS {
        return Err(FigureError::TooManyInputs);
    }
    // A percent more or less is the one operation whose second input is a percent, and whose two
    // inputs are of two kinds: its own function checks them.
    if !matches!(op, CalcOp::PercentMore | CalcOp::PercentLess) {
        if inputs.iter().any(|v| matches!(v, Value::Percent(_))) {
            return Err(FigureError::PercentInput);
        }
        let first = *inputs.first().ok_or(FigureError::NoInputs)?;
        if !inputs.iter().all(|v| same_kind(first, *v)) {
            return Err(FigureError::MixedKinds);
        }
    }
    let (result, dir, step_op) = match op {
        CalcOp::Sum => (sum(inputs)?, None, Op::Sum),
        CalcOp::Average => (average(inputs)?, None, Op::Average),
        CalcOp::Difference => {
            let (a, b) = two(inputs)?;
            let (result, dir) = difference(a, b)?;
            (result, Some(dir), Op::Difference)
        }
        CalcOp::PercentOf => {
            let (a, b) = two(inputs)?;
            (percent_of(a, b)?, None, Op::PercentOf)
        }
        CalcOp::ChangePercent => {
            let (a, b) = two(inputs)?;
            let (result, dir) = change_percent(a, b)?;
            (result, Some(dir), Op::ChangePercent)
        }
        // Its inputs are two kinds (an amount, a percent), so it ends here, with its own working.
        CalcOp::PercentMore | CalcOp::PercentLess => return percent_changed(op, inputs),
    };
    Ok(Calc {
        result,
        dir,
        steps: vec![Step {
            op: step_op,
            inputs: inputs.to_vec(),
            result,
        }],
    })
}

fn same_kind(a: Value, b: Value) -> bool {
    matches!(
        (a, b),
        (Value::Money(_), Value::Money(_))
            | (Value::Balance(_), Value::Balance(_))
            | (Value::Count(_), Value::Count(_))
    )
}

fn two(inputs: &[Value]) -> Result<(Value, Value), FigureError> {
    match inputs {
        [a, b] => Ok((*a, *b)),
        _ => Err(FigureError::NeedsTwoInputs),
    }
}

/// The amount in paisa or the count, as one signed number.
fn raw(value: Value) -> i128 {
    match value {
        Value::Money(p) | Value::Balance(p) => i128::from(p.as_paisa()),
        Value::Count(n) | Value::Percent(n) => i128::from(n),
    }
}

/// Puts a signed number back into the kind of `kind`.
fn like(kind: Value, number: i128) -> Result<Value, FigureError> {
    let number = i64::try_from(number).map_err(|_| FigureError::Overflow)?;
    match kind {
        Value::Money(_) => Ok(Value::Money(Paisa::try_from_paisa(number)?)),
        Value::Balance(_) => Ok(Value::Balance(Paisa::try_from_paisa(number)?)),
        Value::Count(_) => Ok(Value::Count(number)),
        Value::Percent(_) => Ok(Value::Percent(number)),
    }
}

fn is_balance(value: Value) -> bool {
    matches!(value, Value::Balance(_))
}

/// True when two numbers are on the same side: both 0 or above, or both 0 or below.
fn same_side(a: i128, b: i128) -> bool {
    a == 0 || b == 0 || (a < 0) == (b < 0)
}

fn sum(inputs: &[Value]) -> Result<Value, FigureError> {
    let first = *inputs.first().ok_or(FigureError::NoInputs)?;
    let mut total: i128 = 0;
    for value in inputs {
        total = total
            .checked_add(raw(*value))
            .ok_or(FigureError::Overflow)?;
    }
    like(first, total)
}

/// `first − second` with its direction. Two balances are compared by size (10,160 جمع is more than
/// 7,280 جمع), so the result is an amount; opposite sides are refused.
fn difference(a: Value, b: Value) -> Result<(Value, Dir), FigureError> {
    let (x, y) = (raw(a), raw(b));
    if is_balance(a) {
        if !same_side(x, y) {
            return Err(FigureError::MixedSigns);
        }
        let bigger = x
            .checked_abs()
            .ok_or(FigureError::Overflow)?
            .checked_sub(y.checked_abs().ok_or(FigureError::Overflow)?)
            .ok_or(FigureError::Overflow)?;
        return Ok((
            like(Value::Money(Paisa::ZERO), bigger)?,
            Dir::of_i128(bigger),
        ));
    }
    let result = x.checked_sub(y).ok_or(FigureError::Overflow)?;
    Ok((like(a, result)?, Dir::of_i128(result)))
}

/// The average of money, balances and counts is `avg_rate` on the magnitude of the total, then
/// given the total's sign (a count is averaged as rupees, so there is one rounding function).
fn average(inputs: &[Value]) -> Result<Value, FigureError> {
    let first = *inputs.first().ok_or(FigureError::NoInputs)?;
    let total = raw(sum(inputs)?);
    let count = i64::try_from(inputs.len()).map_err(|_| FigureError::Overflow)?;
    like(first, divide_half_up(first, total, count)?)
}

/// `total ÷ count` to the nearest whole unit of `kind` (a rupee for money and balances, one for a
/// count), half up on the magnitude, with the sign of `total`. The one rounding is `avg_rate`.
pub(crate) fn divide_half_up(kind: Value, total: i128, count: i64) -> Result<i128, FigureError> {
    let qty = Qty::new(count)?;
    let magnitude = total.checked_abs().ok_or(FigureError::Overflow)?;
    let magnitude = i64::try_from(magnitude).map_err(|_| FigureError::Overflow)?;
    let as_paisa = match kind {
        Value::Money(_) | Value::Balance(_) => Paisa::try_from_paisa(magnitude)?,
        // A count is a whole number: handled as that many rupees, so `avg_rate` rounds it too.
        Value::Count(_) | Value::Percent(_) => Paisa::from_rupees(magnitude)?,
    };
    let rounded = avg_rate(as_paisa, qty)?;
    let rounded = match kind {
        Value::Money(_) | Value::Balance(_) => i128::from(rounded.as_paisa()),
        Value::Count(_) | Value::Percent(_) => i128::from(rounded.rupees()),
    };
    if total < 0 {
        rounded.checked_neg().ok_or(FigureError::Overflow)
    } else {
        Ok(rounded)
    }
}

/// `part` as a percent of `base`, in hundredths of a percent (core `hundredths`). Both on the same
/// side, or the part 0. The result can be over 100%.
fn percent_of(part: Value, base: Value) -> Result<Value, FigureError> {
    let (p, b) = (raw(part), raw(base));
    if b == 0 {
        return Err(FigureError::NoBase);
    }
    if p != 0 && (p < 0) != (b < 0) {
        return Err(FigureError::MixedSigns);
    }
    Ok(Value::Percent(hundredths(p, b)?))
}

/// The change from `old` to `new` in percent of `old`, in hundredths, with the direction of the
/// change. Two balances are compared by size and must be on the same side; amounts and counts are
/// signed (a profit of −5,000 that becomes 3,000 rose by 8,000, which is 160% of 5,000).
fn change_percent(old: Value, new: Value) -> Result<(Value, Dir), FigureError> {
    let (o, n) = (raw(old), raw(new));
    if is_balance(old) {
        if !same_side(o, n) {
            return Err(FigureError::MixedSigns);
        }
        let (o, n) = (
            o.checked_abs().ok_or(FigureError::Overflow)?,
            n.checked_abs().ok_or(FigureError::Overflow)?,
        );
        return change_of(o, n);
    }
    change_of(o, n)
}

fn change_of(o: i128, n: i128) -> Result<(Value, Dir), FigureError> {
    if o == 0 {
        return Err(FigureError::NoBase);
    }
    let delta = n.checked_sub(o).ok_or(FigureError::Overflow)?;
    Ok((Value::Percent(hundredths(delta, o)?), Dir::of_i128(delta)))
}

/// The most a figure is scaled to, in basis points of itself (1,000%).
pub(crate) const MAX_FACTOR_BPS: i32 = 100_000;
/// 100% in basis points.
pub(crate) const ONE_BPS: i32 = 10_000;

/// `base × factor`, with `factor` in basis points of 100% (11,000 is 110%): `round_half_up(base ×
/// factor ÷ 10,000)`, rounded once, as the step `base × factor%` the Owner can check.
///
/// `Bps` holds only 0 to 10,000, so the factor is split: `factor = whole × 10,000 + rest`, and
/// the result is `base × whole` (exact) plus `apply_bps(base, rest)` (the one rounding, half up,
/// by the shared function). Whole rupees added to a half-up rounded figure give the same result as
/// rounding the whole product, so double and triple are exact and 48,333 down 50% is 24,167. A
/// result above the largest allowed amount is refused.
///
/// # Errors
/// `NegativeBase` for a figure below 0, `Overflow`.
pub(crate) fn scale_amount(base: Paisa, factor: i32) -> Result<Step, FigureError> {
    if base < Paisa::ZERO {
        return Err(FigureError::NegativeBase);
    }
    let f = i64::from(factor);
    let whole = f
        .checked_div(i64::from(ONE_BPS))
        .ok_or(FigureError::Overflow)?;
    let rest = f
        .checked_rem(i64::from(ONE_BPS))
        .ok_or(FigureError::Overflow)?;
    let rest = i32::try_from(rest).map_err(|_| FigureError::Overflow)?;
    let exact = base
        .as_paisa()
        .checked_mul(whole)
        .ok_or(FigureError::Overflow)?;
    let fraction = apply_bps(base, Bps::new(rest)?)?;
    let total = exact
        .checked_add(fraction.as_paisa())
        .ok_or(FigureError::Overflow)?;
    let total = Paisa::try_from_paisa(total)?;
    let largest = Paisa::from_rupees(MAX_RUPEES)?;
    if total > largest {
        return Err(FigureError::Overflow);
    }
    Ok(Step {
        op: Op::Times,
        inputs: vec![Value::Money(base), Value::Percent(f)],
        result: Value::Money(total),
    })
}

/// An amount and a percent more or less: `base × (100% ± percent)`, rounded once (see [`scale_amount`]).
/// The percent is the second input, in hundredths of a percent (1,000 is 10.00%): 0.01% to 900%
/// more, 0.01% to 99.99% less. Only an amount of 0 or more.
///
/// Example: 8,360,110 and 10.00% more is 9,196,121 (8,360,110 × 110.00%); 48,333 and 50.00% less
/// is 24,167.
fn percent_changed(op: CalcOp, inputs: &[Value]) -> Result<Calc, FigureError> {
    let (base, percent) = two(inputs)?;
    let (Value::Money(base), Value::Percent(percent)) = (base, percent) else {
        return Err(FigureError::MixedKinds);
    };
    let percent = i32::try_from(percent).map_err(|_| FigureError::PercentOutOfRange)?;
    let (factor, allowed) = match op {
        CalcOp::PercentLess => (
            ONE_BPS.checked_sub(percent),
            (1..ONE_BPS).contains(&percent),
        ),
        _ => (
            ONE_BPS.checked_add(percent),
            (1..=MAX_FACTOR_BPS.saturating_sub(ONE_BPS)).contains(&percent),
        ),
    };
    let factor = factor
        .filter(|_| allowed)
        .ok_or(FigureError::PercentOutOfRange)?;
    let step = scale_amount(base, factor)?;
    Ok(Calc {
        result: step.result,
        dir: None,
        steps: vec![step],
    })
}

/// A step `first × second` for an estimate's working (per month × months ahead). Money times a
/// count, exact.
pub(crate) fn times(per_month: Paisa, count: i64) -> Result<Step, FigureError> {
    let total = per_month
        .as_paisa()
        .checked_mul(count)
        .ok_or(FigureError::Overflow)?;
    Ok(Step {
        op: Op::Times,
        inputs: vec![Value::Money(per_month), Value::Count(count)],
        result: Value::Money(Paisa::try_from_paisa(total)?),
    })
}

/// A step `money ÷ count` for an estimate's working (a difference ÷ months), to the nearest rupee,
/// half up on the magnitude.
pub(crate) fn divided_by(money: Paisa, count: i64) -> Result<Step, FigureError> {
    let kind = Value::Money(money);
    let rounded = divide_half_up(kind, i128::from(money.as_paisa()), count)?;
    Ok(Step {
        op: Op::DividedBy,
        inputs: vec![kind, Value::Count(count)],
        result: like(kind, rounded)?,
    })
}

// ---- the numbers of a question -----------------------------------------------------------------

/// What follows a number in the question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberUnit {
    /// Months: "agle 6 mahine".
    Months,
    /// Days: "pichle 10 din".
    Days,
    /// Years: "pichle 2 saal".
    Years,
    /// A percent: "10%".
    Percent,
    /// An amount written with lakh, hazar or crore: "3 lakh".
    Amount,
    /// Anything else: "10,000", a بکری number.
    Plain,
}

/// A number the Owner typed in his question (D6-system-prompt 5.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionNumber {
    /// The value: for an amount the whole amount in rupees (300000 for "3 lakh"); for a percent the
    /// whole part (10 for "10%").
    pub number: i64,
    /// The number as typed, with its lakh, hazar or crore, or its `%`: "3 lakh", "10,000", "10%".
    pub text: String,
    /// Its unit.
    pub unit: NumberUnit,
    /// For a percent: basis points, 1000 for "10%" and 250 for "2.5%". Null otherwise.
    pub bps: Option<i32>,
}

const MONTH_WORDS: &[&str] = &[
    "mahine",
    "mahina",
    "mahiny",
    "mahinay",
    "mahiney",
    "mahinon",
    "mahino",
    "month",
    "months",
    // Misspellings people type (live run 2, Q696: "after 2 monhts"): a number must not lose its unit.
    "monhts",
    "mnths",
    "monts",
    "monthes",
    "mahene",
    "mahenay",
    "مہینے",
    "مہینہ",
    "مہینوں",
    "ماہ",
];
const DAY_WORDS: &[&str] = &["din", "dino", "dinon", "day", "days", "دن", "دنوں"];
const YEAR_WORDS: &[&str] = &[
    "saal",
    "sal",
    "saalon",
    "salon",
    "year",
    "years",
    "سال",
    "سالوں",
    "برس",
    "برسوں",
];
const PERCENT_WORDS: &[&str] = &["percent", "fisad", "فیصد"];
const LAKH_WORDS: &[&str] = &["lakh", "lakhs", "lac", "lacs", "لاکھ"];
const HAZAR_WORDS: &[&str] = &["hazar", "hazaar", "hazaro", "hajar", "ہزار"];
const CRORE_WORDS: &[&str] = &["crore", "crores", "karor", "karore", "کروڑ"];
/// Multiplier words the app does not read (PRD FR-7.76 lists lakh, hazar and crore only). A number
/// followed by one of them is **not** read at all: "3 thousand" must never become a plain 3, which
/// would make a target 1,000 times too small.
const UNREAD_MULTIPLIER_WORDS: &[&str] = &[
    "k",
    "thousand",
    "thousands",
    "million",
    "millions",
    "billion",
    "billions",
    "hundred",
    "hundreds",
    "sau",
    "arab",
    "arabs",
    "ارب",
    "سو",
];
/// A word that begins like a multiplier but is not one the app knows ("laakh", "lak", "hazr"): the
/// number before it is **not** read, because reading it as plain would make an amount 1,00,000 times
/// too small. Prefixes of at least three letters, so ordinary words ("kar", "karo", "bill", "mil gaye") are not caught; "million" and "billion" are listed whole.
const MULTIPLIER_PREFIXES: &[&str] = &[
    "lak", "lac", "laa", "haz", "haj", "cro", "kro", "karor", "karod", "thou", "لاک", "ہزا", "کرو",
];
const LAKH: i128 = 100_000;
const HAZAR: i128 = 1_000;
const CRORE: i128 = 10_000_000;

/// One digit of any script a keyboard gives: ASCII, Arabic-Indic, Urdu and full-width.
fn digit_value(c: char) -> Option<u32> {
    let code = u32::from(c);
    let base = match code {
        0x30..=0x39 => 0x30,
        0x660..=0x669 => 0x660,
        0x6F0..=0x6F9 => 0x6F0,
        0xFF10..=0xFF19 => 0xFF10,
        _ => return None,
    };
    code.checked_sub(base)
}

struct Scan {
    chars: Vec<char>,
    pos: usize,
}

impl Scan {
    fn peek_at(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.pos.saturating_add(ahead)).copied()
    }
    fn peek(&self) -> Option<char> {
        self.peek_at(0)
    }
    fn bump(&mut self) {
        self.pos = self.pos.saturating_add(1);
    }
    /// True when the character before the cursor is a comma that follows a digit: this number
    /// came right after "12,".
    fn after_list_comma(&self) -> bool {
        let before = |back: usize| {
            self.pos
                .checked_sub(back)
                .and_then(|i| self.chars.get(i))
                .copied()
        };
        matches!(before(1), Some(',' | '٬')) && before(2).and_then(digit_value).is_some()
    }
    fn digit_at(&self, ahead: usize) -> bool {
        self.peek_at(ahead).and_then(digit_value).is_some()
    }
    /// The lengths of the digit groups that follow from here, each after a thousands comma:
    /// "10,000" gives `[3]`, "3,00,000" gives `[2, 3]`. Nothing is consumed.
    fn group_lengths(&self) -> Vec<usize> {
        let mut lengths = Vec::new();
        let mut at = 0usize;
        while matches!(self.peek_at(at), Some(',' | '٬')) && self.digit_at(at.saturating_add(1)) {
            let mut n = 0usize;
            while self.digit_at(at.saturating_add(1).saturating_add(n)) {
                n = n.saturating_add(1);
            }
            lengths.push(n);
            at = at.saturating_add(1).saturating_add(n);
        }
        lengths
    }
}

/// True when digits written with thousands commas are a proper number: the first group of 1 to 3
/// digits and every later group of exactly 3 ("1,234,567"), or the lakh way, a first group of 1 or
/// 2 digits, middle groups of exactly 2 and a last group of 3 ("12,34,567"). "12,15", "1,00" and
/// "1234,567" are not: they are separate numbers, never one invented number (a list of بکری
/// numbers must not become one).
fn grouping_ok(first: usize, rest: &[usize]) -> bool {
    let Some((last, middle)) = rest.split_last() else {
        return true;
    };
    let international = (1..=3).contains(&first) && rest.iter().all(|n| *n == 3);
    let lakh = (1..=2).contains(&first) && *last == 3 && middle.iter().all(|n| *n == 2);
    international || lakh
}

/// A run of digits with the separators of a number, as typed.
struct Token {
    int: String,
    frac: String,
    text: String,
}

/// Takes the run of digits at the cursor into the whole part of the token.
fn take_digits(scan: &mut Scan, token: &mut Token) {
    while let Some(d) = scan.peek().and_then(digit_value) {
        token.int.push(char::from_digit(d, 10).unwrap_or('0'));
        token.text.extend(scan.peek());
        scan.bump();
    }
}

fn read_token(scan: &mut Scan) -> Token {
    let mut token = Token {
        int: String::new(),
        frac: String::new(),
        text: String::new(),
    };
    take_digits(scan, &mut token);
    let groups = scan.group_lengths();
    if !groups.is_empty() && grouping_ok(token.int.chars().count(), &groups) {
        for _ in &groups {
            // The comma, then the group's digits.
            token.text.extend(scan.peek());
            scan.bump();
            take_digits(scan, &mut token);
        }
    }
    if matches!(scan.peek(), Some('.' | '٫')) && scan.digit_at(1) {
        token.text.extend(scan.peek());
        scan.bump();
        while let Some(d) = scan.peek().and_then(digit_value) {
            token.frac.push(char::from_digit(d, 10).unwrap_or('0'));
            token.text.extend(scan.peek());
            scan.bump();
        }
    }
    token
}

/// The word after a number (spaces skipped), lower-cased, or `%`.
fn read_unit_word(scan: &mut Scan) -> Option<String> {
    let mut ahead = 0usize;
    while scan.peek_at(ahead).is_some_and(char::is_whitespace) {
        ahead = ahead.saturating_add(1);
    }
    let first = scan.peek_at(ahead)?;
    if matches!(first, '%' | '٪' | '％') {
        scan.pos = scan.pos.saturating_add(ahead).saturating_add(1);
        return Some("%".to_owned());
    }
    if !first.is_alphabetic() {
        return None;
    }
    let mut word = String::new();
    let mut end = ahead;
    while let Some(c) = scan.peek_at(end).filter(|c| c.is_alphabetic()) {
        word.extend(c.to_lowercase());
        end = end.saturating_add(1);
    }
    scan.pos = scan.pos.saturating_add(end);
    Some(word)
}

/// `digits` as an integer, or `None` for no digits or more than [`MAX_DIGITS`].
fn parse_digits(digits: &str) -> Option<i128> {
    if digits.is_empty() || digits.len() > MAX_DIGITS {
        return None;
    }
    digits.parse::<i128>().ok()
}

/// `int.frac × multiplier` as a whole number, only when it is exact ("2.5 lakh" is 250,000).
fn scaled(token: &Token, multiplier: i128) -> Option<i64> {
    if token.frac.len() > MAX_DECIMALS {
        return None;
    }
    let all = format!("{}{}", token.int, token.frac);
    let n = parse_digits(&all)?;
    let places = u32::try_from(token.frac.len()).ok()?;
    let scale = 10i128.checked_pow(places)?;
    let top = n.checked_mul(multiplier)?;
    if top.checked_rem(scale)? != 0 {
        return None;
    }
    i64::try_from(top.checked_div(scale)?).ok()
}

fn in_words(words: &[&str], word: &str) -> bool {
    words.contains(&word)
}

/// The numbers the Owner typed, in the order he typed them, at most [`MAX_QUESTION_NUMBERS`]
/// (D6-system-prompt 5.3, money agent Section 9.1). Digits of every script are read; a thousands
/// comma is read inside a number ("10,000", "3,00,000"). The unit comes from the word after the
/// number: months, days or years; `%` for a percent; lakh, hazar or crore for an amount (the
/// number is multiplied; amounts next to each other with smaller words after bigger ones are one
/// amount: "3 lakh 50 hazar" is 350,000); anything else is plain. A plain decimal ("2.5") is not
/// a number, and neither is a number of more than 18 digits or one that does not fit, or one with
/// a thousand, million or `k` after it (those words are not read). Number words ("teen", "ایک")
/// are not read. A minus sign is never read: "10-20" is two numbers. It never panics, on any text.
///
/// ```
/// use mandi_domain::figures::{NumberUnit, question_numbers};
/// let n = question_numbers("agle 6 mahine");
/// assert_eq!((n[0].number, n[0].unit), (6, NumberUnit::Months));
/// let n = question_numbers("3 lakh 50 hazar ka target");
/// assert_eq!((n[0].number, n[0].unit), (350_000, NumberUnit::Amount));
/// let n = question_numbers("10% zyada");
/// assert_eq!((n[0].number, n[0].bps), (10, Some(1000)));
/// ```
pub fn question_numbers(text: &str) -> Vec<QuestionNumber> {
    let mut scan = Scan {
        chars: text.chars().collect(),
        pos: 0,
    };
    let mut found: Vec<QuestionNumber> = Vec::new();
    // Where the last number read ended: a decimal comma ("1,5 lakh") takes it back.
    let mut last_end: Option<usize> = None;
    while scan.pos < scan.chars.len() && found.len() < MAX_QUESTION_NUMBERS {
        if scan.peek().and_then(digit_value).is_none() {
            scan.bump();
            continue;
        }
        let start = scan.pos;
        let after_comma = scan.after_list_comma();
        let touches_last = last_end.is_some() && last_end == start.checked_sub(1);
        let read = read_number(&mut scan);
        // "1,5 lakh" or "10,5%": a comma used as a decimal point. Neither part is read, because
        // "5 lakh" alone would be 5,00,000 and the 1 a separate number.
        let is_scaled =
            |n: &QuestionNumber| matches!(n.unit, NumberUnit::Amount | NumberUnit::Percent);
        if after_comma && touches_last && read.as_ref().is_none_or(|(n, _)| is_scaled(n)) {
            found.pop();
            last_end = None;
            continue;
        }
        let Some((mut number, mut multiplier)) = read else {
            continue;
        };
        // "3 lakh 50 hazar": an amount followed by a smaller amount is one amount. An amount
        // followed by a bare number ("3 lakh 50") or a number with a word the app does not read
        // ("50k") means something the app cannot tell: neither is read, so the model's own number
        // is refused rather than a wrong target accepted.
        let mut ambiguous = false;
        while let Some(bigger) = multiplier {
            let saved = scan.pos;
            if !skip_spaces(&mut scan) {
                scan.pos = saved;
                break;
            }
            match read_number(&mut scan) {
                Some((more, Some(smaller))) if smaller < bigger => {
                    let Some(joined) = number.number.checked_add(more.number) else {
                        ambiguous = true;
                        break;
                    };
                    number.number = joined;
                    number.text = format!("{} {}", number.text, more.text);
                    multiplier = Some(smaller);
                }
                // Another amount, as big or bigger: two amounts.
                Some((_, Some(_))) => {
                    scan.pos = saved;
                    break;
                }
                // A number with its own unit (months, days, years, a percent): its own number.
                Some((more, None)) if more.unit != NumberUnit::Plain => {
                    scan.pos = saved;
                    break;
                }
                Some((_, None)) | None => {
                    ambiguous = true;
                    break;
                }
            }
        }
        if ambiguous {
            last_end = None;
            continue;
        }
        last_end = Some(scan.pos);
        found.push(number);
    }
    found
}

/// Skips spaces; true if the next character is a digit.
fn skip_spaces(scan: &mut Scan) -> bool {
    while scan.peek().is_some_and(char::is_whitespace) {
        scan.bump();
    }
    scan.peek().and_then(digit_value).is_some()
}

/// One number with its unit word, and the multiplier when it is an amount.
fn read_number(scan: &mut Scan) -> Option<(QuestionNumber, Option<i128>)> {
    let token = read_token(scan);
    let word = read_unit_word(scan);
    number_with_unit(&token, word.as_deref())
}

fn number_with_unit(token: &Token, word: Option<&str>) -> Option<(QuestionNumber, Option<i128>)> {
    let word = word.unwrap_or("");
    let plain_unit = |unit: NumberUnit| -> Option<(QuestionNumber, Option<i128>)> {
        if !token.frac.is_empty() {
            return None;
        }
        Some((
            QuestionNumber {
                number: i64::try_from(parse_digits(&token.int)?).ok()?,
                text: token.text.clone(),
                unit,
                bps: None,
            },
            None,
        ))
    };
    let amount = |multiplier: i128| -> Option<(QuestionNumber, Option<i128>)> {
        Some((
            QuestionNumber {
                number: scaled(token, multiplier)?,
                text: format!("{} {}", token.text, word),
                unit: NumberUnit::Amount,
                bps: None,
            },
            Some(multiplier),
        ))
    };
    if in_words(UNREAD_MULTIPLIER_WORDS, word) {
        return None;
    }
    if word == "%" || in_words(PERCENT_WORDS, word) {
        if token.frac.len() > 2 {
            return None;
        }
        let bps = scaled(token, 100)?;
        return Some((
            QuestionNumber {
                number: i64::try_from(parse_digits(&token.int)?).ok()?,
                text: format!("{}%", token.text),
                unit: NumberUnit::Percent,
                bps: Some(i32::try_from(bps).ok()?),
            },
            None,
        ));
    }
    if in_words(LAKH_WORDS, word) {
        return amount(LAKH);
    }
    if in_words(HAZAR_WORDS, word) {
        return amount(HAZAR);
    }
    if in_words(CRORE_WORDS, word) {
        return amount(CRORE);
    }
    if in_words(MONTH_WORDS, word) {
        return plain_unit(NumberUnit::Months);
    }
    if in_words(DAY_WORDS, word) {
        return plain_unit(NumberUnit::Days);
    }
    if in_words(YEAR_WORDS, word) {
        return plain_unit(NumberUnit::Years);
    }
    if MULTIPLIER_PREFIXES.iter().any(|p| word.starts_with(p)) {
        return None;
    }
    plain_unit(NumberUnit::Plain)
}
