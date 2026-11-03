//! The tools `calculate` and `estimate`: the only places a figure is worked out (PRD FR-7.75,
//! FR-7.86, Code Standards Q5, M14).
//!
//! `calculate` takes paths of figures already read, or numbers of the question, and works them out
//! with `services::figures` (this crate does no arithmetic of its own). `estimate` reads the
//! monthly series of the books from analytics view G, the screen's own function, and works the
//! months ahead out with `services::forecast`. Both are display only.
//!
//! The kind of each input (D6-system-prompt "Rules for the inputs", lead decision):
//!
//! | Input | Kind |
//! |---|---|
//! | A balance (it has a side) | balance: بنام above zero, جمع below |
//! | A question number with an amount unit ("3 lakh") | amount, in rupees |
//! | A question number with months, days or years | count |
//! | A plain question number | takes the kind of the other inputs (an amount beside money, a count beside counts) |
//! | A percent | never an input |
//!
//! Example (worked day, E2E-61): `calculate{op:"sum", inputs:["r1.matches.0.balance",
//! "r2.matches.0.balance"]}` is 21,564 بنام, checked because both balances were.

use chrono::{Datelike, NaiveDate};
use mandi_reports::analytics::{
    AnalyticsFilters, AnalyticsGroupBy, AnalyticsRunInput, AnalyticsSide, AnalyticsView,
    run as engine_run,
};
use mandi_reports::reports::{Cell, PartyFilter};
use mandi_services::core::accounts::{AccountType, BalanceSide};
use mandi_services::core::money::Paisa;
use mandi_services::domain::figures::{
    Calc, CalcOp, Dir, NumberUnit, Op, QuestionNumber, Step, Value,
};
use mandi_services::domain::forecast::{Method, MethodEstimate, MonthValue, PaceFactor, YearMonth};
use mandi_services::{figures, forecast};
use serde::Deserialize;
use serde_json::Value as Json;

use super::cells::{is_total_line, magnitude};
use super::cues;
use super::records::{named, require_type};
use super::report::reports_stop;
use super::{Draft, Env, State, Stop, ToolError, input, lookup, service_stop};
use crate::answer::{
    AnswerChart, AnswerEstimate, AnswerMonth, AnswerSource, AnswerWorking, ChartSeries,
    ChartSeriesKind, EstimateMethod, EstimateMethodKind, EstimateYear, FigureDirection, WorkingOp,
    WorkingStep,
};
use crate::results::{Money, Node, Shown, Side, Val};
use crate::source::SourceAnalyticsFilters;

fn bad(code: &str, field: &str, detail: &str) -> ToolError {
    ToolError::new(code, field, detail)
}

// ---- values and cells -----------------------------------------------------------------------

fn side_of(paisa: Paisa) -> BalanceSide {
    match paisa.as_paisa().cmp(&0) {
        std::cmp::Ordering::Greater => BalanceSide::Dr,
        std::cmp::Ordering::Less => BalanceSide::Cr,
        std::cmp::Ordering::Equal => BalanceSide::Zero,
    }
}

fn node_side(paisa: Paisa) -> Side {
    match side_of(paisa) {
        BalanceSide::Dr => Side::Dr,
        BalanceSide::Cr => Side::Cr,
        BalanceSide::Zero => Side::Zero,
    }
}

/// A calculator value as a report cell (a balance has its side in the cell).
fn value_cell(value: Value) -> Cell {
    match value {
        Value::Money(paisa) => Cell::Money { paisa },
        Value::Balance(paisa) => Cell::Balance {
            paisa: magnitude(paisa),
            side: side_of(paisa),
        },
        Value::Count(value) => Cell::Number { value },
        Value::Percent(hundredths) => Cell::Percent { hundredths },
    }
}

fn working_op(op: Op) -> WorkingOp {
    match op {
        Op::Sum => WorkingOp::Sum,
        Op::Difference => WorkingOp::Difference,
        Op::Average => WorkingOp::Average,
        Op::PercentOf => WorkingOp::PercentOf,
        Op::ChangePercent => WorkingOp::ChangePercent,
        Op::Times => WorkingOp::Times,
        Op::DividedBy => WorkingOp::DividedBy,
    }
}

/// The working of a calculation. An average sends how many figures it was over as its LAST input,
/// a count cell, so the panel writes (a + b + c) ÷ n (Tech Stack 4.8, lead decision).
fn working_of(title_id: &str, steps: &[Step]) -> AnswerWorking {
    AnswerWorking {
        title_id: title_id.to_owned(),
        steps: steps
            .iter()
            .map(|s| {
                let mut inputs: Vec<Cell> = s.inputs.iter().map(|v| value_cell(*v)).collect();
                if s.op == Op::Average {
                    inputs.push(Cell::Number {
                        value: i64::try_from(s.inputs.len()).unwrap_or(i64::MAX),
                    });
                }
                WorkingStep {
                    op: working_op(s.op),
                    inputs,
                    result: value_cell(s.result),
                }
            })
            .collect(),
    }
}

fn figure_dir(dir: Dir) -> FigureDirection {
    match dir {
        Dir::Up => FigureDirection::Up,
        Dir::Down => FigureDirection::Down,
        Dir::Same => FigureDirection::Same,
    }
}

// ---- calculate -------------------------------------------------------------------------------

/// The operations of `calculate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpWord {
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
    /// An amount and a percent more (`first × (100% + percent)`).
    PercentMore,
    /// An amount and a percent less (`first × (100% − percent)`).
    PercentLess,
}

impl OpWord {
    fn calc(self) -> CalcOp {
        match self {
            OpWord::Sum => CalcOp::Sum,
            OpWord::Difference => CalcOp::Difference,
            OpWord::Average => CalcOp::Average,
            OpWord::PercentOf => CalcOp::PercentOf,
            OpWord::ChangePercent => CalcOp::ChangePercent,
            OpWord::PercentMore => CalcOp::PercentMore,
            OpWord::PercentLess => CalcOp::PercentLess,
        }
    }

    fn word(self) -> &'static str {
        match self {
            OpWord::Sum => "sum",
            OpWord::Difference => "difference",
            OpWord::Average => "average",
            OpWord::PercentOf => "percent_of",
            OpWord::ChangePercent => "change_percent",
            OpWord::PercentMore => "percent_more",
            OpWord::PercentLess => "percent_less",
        }
    }

    fn title_id(self) -> &'static str {
        match self {
            OpWord::Sum => "assistant-working-sum",
            OpWord::Difference => "assistant-working-difference",
            OpWord::Average => "assistant-working-average",
            OpWord::PercentOf => "assistant-working-percent-of",
            OpWord::ChangePercent => "assistant-working-change-percent",
            OpWord::PercentMore => "assistant-working-percent-more",
            OpWord::PercentLess => "assistant-working-percent-less",
        }
    }
}

/// The input of `calculate`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalcInput {
    /// The operation.
    pub op: OpWord,
    /// Paths of figures (`r3.balance`, `r4.rows.*.balance`) or numbers of the question (`q.n1`).
    pub inputs: Vec<String>,
}

/// An input before its kind is settled.
enum Pending {
    /// A figure read from a result, with its `checked` flag when it is a balance.
    Known(Value, Option<bool>),
    /// A plain question number.
    Plain(i64),
    /// A question number with an amount unit.
    Amount(i64),
    /// A question number with months, days or years.
    Counted(i64),
    /// A question number with a % sign, in hundredths of a percent (10% is 1,000). Only a percent
    /// more or less takes it.
    Percent(i64),
}

fn money_value(paisa: Paisa) -> Value {
    Value::Money(paisa)
}

fn known_of(val: &Val, field: &str) -> Result<Pending, ToolError> {
    let signed = |size: Paisa, dir: Dir| -> Result<Paisa, ToolError> {
        match dir {
            Dir::Down => size
                .checked_neg()
                .map_err(|_| bad("ERR_OVERFLOW", field, "the figure does not fit")),
            _ => Ok(size),
        }
    };
    Ok(match val {
        Val::Money(m) => match m.side {
            Some(Side::Dr) | Some(Side::Zero) => Pending::Known(Value::Balance(m.paisa), m.checked),
            Some(Side::Cr) => {
                let neg = m
                    .paisa
                    .checked_neg()
                    .map_err(|_| bad("ERR_OVERFLOW", field, "the figure does not fit"))?;
                Pending::Known(Value::Balance(neg), m.checked)
            }
            None => Pending::Known(money_value(m.paisa), None),
        },
        Val::Diff { size, dir } => Pending::Known(Value::Money(signed(*size, *dir)?), None),
        Val::Count(n) => Pending::Known(Value::Count(*n), None),
        Val::CountDiff { size, dir } => {
            let n = if *dir == Dir::Down {
                size.checked_neg()
                    .ok_or_else(|| bad("ERR_OVERFLOW", field, "the figure does not fit"))?
            } else {
                *size
            };
            Pending::Known(Value::Count(n), None)
        }
        Val::Pct(_) | Val::Change { .. } => {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                field,
                "a percent cannot be an input",
            ));
        }
        _ => {
            return Err(bad(
                "ERR_FORMAT_INVALID",
                field,
                "this path is not a number",
            ));
        }
    })
}

/// The question number a path `q.n3` names, with its unit.
pub(super) fn question_number<'e>(env: &'e Env<'_>, path: &str) -> Option<&'e QuestionNumber> {
    let n: usize = path.strip_prefix("q.n")?.parse().ok()?;
    env.numbers.get(n.checked_sub(1)?)
}

fn pending_of_question(number: &QuestionNumber, field: &str) -> Result<Pending, ToolError> {
    match number.unit {
        NumberUnit::Plain => Ok(Pending::Plain(number.number)),
        NumberUnit::Amount => Ok(Pending::Amount(number.number)),
        NumberUnit::Months | NumberUnit::Days | NumberUnit::Years => {
            Ok(Pending::Counted(number.number))
        }
        // The calculator refuses a percent beside any operation but a percent more or less.
        NumberUnit::Percent => number
            .bps
            .map(|bps| Pending::Percent(i64::from(bps)))
            .ok_or_else(|| bad("ERR_OUT_OF_RANGE", field, "a percent cannot be an input")),
    }
}

/// Every figure a path names: one for a plain path, one for each row for `rows.*`.
fn expand(
    env: &Env<'_>,
    state: &State,
    path: &str,
    field: &str,
) -> Result<Vec<Pending>, ToolError> {
    if let Some(number) = question_number(env, path) {
        return Ok(vec![pending_of_question(number, field)?]);
    }
    let parts: Vec<&str> = path.split('.').collect();
    let Some(star) = parts.iter().position(|p| *p == "*") else {
        let found = lookup(env, state, path, field)?;
        let Node::Val(val) = found.node() else {
            return Err(bad(
                "ERR_FORMAT_INVALID",
                field,
                "this path is not a number",
            ));
        };
        return Ok(vec![known_of(val, field)?]);
    };
    let (head, tail) = (
        parts.get(..star).unwrap_or(&[]),
        parts.get(star.saturating_add(1)..).unwrap_or(&[]),
    );
    let head_path = head.join(".");
    let root = head.first().copied().unwrap_or("");
    let result = state
        .find(root)
        .ok_or_else(|| bad("ERR_UNKNOWN_REFERENCE", field, path))?;
    let container = lookup(env, state, &head_path, field)?;
    // A heading or total line holds sums of the lines under it: it is not a row to add up.
    let (rows, lines_sent): (Vec<&Node>, usize) = match container.node() {
        Node::Keyed(rows) => (
            rows.iter()
                .filter(|(key, _)| !is_total_line(key))
                .map(|(_, n)| n)
                .collect(),
            rows.len(),
        ),
        Node::List(items) => (items.iter().collect(), items.len()),
        _ => return Err(bad("ERR_FORMAT_INVALID", field, "this path has no rows")),
    };
    // A sum over a cut list would be wrong: refuse it (D6-system-prompt "Rules for the inputs").
    if let Node::Obj(fields) = &result.node {
        let cut = fields
            .iter()
            .any(|(k, v)| k == "rows_cut" && matches!(v, Node::Bool(true)));
        let count = fields.iter().find_map(|(k, v)| match (k.as_str(), v) {
            ("count", Node::Val(Val::Count(n))) => Some(*n),
            _ => None,
        });
        // The heading and total lines of a sorted report are given apart (`headings`): they are
        // part of its count and were sent.
        let apart = match result.node.get(&["headings"]) {
            Some(Node::Keyed(rows)) => rows.len(),
            _ => 0,
        };
        let sent = i64::try_from(lines_sent.saturating_add(apart)).unwrap_or(i64::MAX);
        if cut || count.is_some_and(|n| n > sent) {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                field,
                "not every row was read; use totals or matched.totals",
            ));
        }
    }
    let mut out = Vec::new();
    for row in rows {
        let Some(Node::Val(val)) = row.get(tail) else {
            continue;
        };
        out.push(known_of(val, field)?);
    }
    if out.is_empty() {
        return Err(bad("ERR_NO_INPUTS", field, "no figure at this path"));
    }
    Ok(out)
}

/// Settles the kind of each input and gives the values with whether the result was checked.
///
/// A plain question number takes the kind of the figures beside it: a balance beside balances (on
/// the same side as they are: a number beside credit balances is a credit), money beside money,
/// else a count. Balances on both sides beside a plain number are refused: the side would be a
/// guess (money review C2).
///
/// The result is checked only when every input is a balance read in two places that agreed. A
/// number the Owner typed, or a figure with no second place, never makes a check mark (money
/// review C1; the tool says "checked when all inputs are"). One balance whose places differ makes
/// the result `Some(false)`.
fn settle(pending: Vec<Pending>) -> Result<(Vec<Value>, Option<bool>), ToolError> {
    let money_near = pending.iter().any(|p| {
        matches!(
            p,
            Pending::Amount(_) | Pending::Known(Value::Money(_) | Value::Balance(_), _)
        )
    });
    let (mut debit_near, mut credit_near) = (false, false);
    for item in &pending {
        if let Pending::Known(Value::Balance(p), _) = item {
            match p.as_paisa().cmp(&0) {
                std::cmp::Ordering::Greater => debit_near = true,
                std::cmp::Ordering::Less => credit_near = true,
                std::cmp::Ordering::Equal => {}
            }
        }
    }
    let balance_near = pending
        .iter()
        .any(|p| matches!(p, Pending::Known(Value::Balance(_), _)));
    let has_plain = pending.iter().any(|p| matches!(p, Pending::Plain(_)));
    // The side of a typed number is that of the balances beside it. With balances on both sides,
    // or only balances of exactly 0, it would be a guess.
    if has_plain && balance_near && debit_near == credit_near {
        return Err(bad(
            "ERR_OUT_OF_RANGE",
            "inputs",
            "no side to give the number beside these balances; read the figure instead",
        ));
    }
    let rupees = |n: i64| -> Result<Paisa, ToolError> {
        Paisa::from_rupees(n).map_err(|_| bad("ERR_OVERFLOW", "inputs", "the number does not fit"))
    };
    let mut values = Vec::new();
    let mut every_input_checked = true;
    let mut any_unchecked = false;
    for item in pending {
        let value = match item {
            Pending::Known(v, c) => {
                match (v, c) {
                    (Value::Balance(_), Some(true)) => {}
                    (Value::Balance(_), Some(false)) => any_unchecked = true,
                    _ => every_input_checked = false,
                }
                v
            }
            Pending::Amount(n) => {
                every_input_checked = false;
                Value::Money(rupees(n)?)
            }
            Pending::Counted(n) => {
                every_input_checked = false;
                Value::Count(n)
            }
            Pending::Percent(h) => {
                every_input_checked = false;
                Value::Percent(h)
            }
            Pending::Plain(n) => {
                every_input_checked = false;
                if balance_near {
                    let size = rupees(n)?;
                    Value::Balance(if credit_near {
                        size.checked_neg()
                            .map_err(|_| bad("ERR_OVERFLOW", "inputs", "the number does not fit"))?
                    } else {
                        size
                    })
                } else if money_near {
                    Value::Money(rupees(n)?)
                } else {
                    Value::Count(n)
                }
            }
        };
        values.push(value);
    }
    let checked = if any_unchecked {
        Some(false)
    } else if every_input_checked && !values.is_empty() {
        Some(true)
    } else {
        None
    };
    Ok((values, checked))
}

/// "What percent of the total dues does Arshad owe" (Q071 of the live run of 02-11-2026): a buyer's
/// balance is a debit balance and the total dues is a plain amount, and the calculator refuses to
/// mix the two kinds. For `percent_of` only, and only for two figures READ from results (never a
/// number the Owner typed), a buyer's debit balance (or 0) beside a plain amount is read as its
/// size, so the percent is what he owes over the amount. It is refused, never guessed, when the
/// balance is a credit (its sign would have to be guessed), when the account is not a buyer's (a
/// seller's debit is an advance, a different thing from dues), or when the balance's two places
/// differ (a percent has no check mark to warn with). A sum or a difference of a balance and an
/// amount is still refused: there the side matters.
fn buyer_balance_over_an_amount(
    env: &Env<'_>,
    state: &State,
    paths: &[String],
    pending: Vec<Pending>,
) -> Result<Vec<Pending>, ToolError> {
    let [first, second] = pending.as_slice() else {
        return Ok(pending);
    };
    let read_balance = |p: &Pending| matches!(p, Pending::Known(Value::Balance(_), _));
    let read_amount = |p: &Pending| matches!(p, Pending::Known(Value::Money(_), _));
    let at = if read_balance(first) && read_amount(second) {
        0
    } else if read_amount(first) && read_balance(second) {
        1
    } else {
        return Ok(pending);
    };
    let Some(Pending::Known(Value::Balance(size), checked)) = pending.get(at) else {
        return Ok(pending);
    };
    let (size, checked) = (*size, *checked);
    let refuse = |detail: &str| bad("ERR_OUT_OF_RANGE", "inputs", detail);
    if size.as_paisa() < 0 {
        return Err(refuse(
            "a credit balance beside a plain amount has no sign to use; compare balances with \
             balances, or read the amount of the credit",
        ));
    }
    if checked == Some(false) {
        return Err(refuse(
            "the two places of this balance differ, so no percent is given from it; open the \
             report",
        ));
    }
    let path = paths.get(at).map_or("", String::as_str);
    if owner_type(env, state, path).as_deref() != Some("buyer") {
        return Err(refuse(
            "only a buyer's debit balance may be taken as a size beside a plain amount; this \
             balance is not known to be a buyer's",
        ));
    }
    Ok(pending
        .into_iter()
        .enumerate()
        .map(|(i, p)| match (i == at, p) {
            (true, Pending::Known(Value::Balance(size), _)) => {
                Pending::Known(Value::Money(size), None)
            }
            (_, other) => other,
        })
        .collect())
}

/// The kind of account a balance path belongs to (`buyer`, `seller` ...): the `type` of the find
/// match or the party card that holds it (`r1.matches.0.balance`, `r2.balance`).
fn owner_type(env: &Env<'_>, state: &State, path: &str) -> Option<String> {
    let head = path.rsplit_once('.')?.0;
    let found = lookup(env, state, head, "inputs").ok()?;
    match found.node().get(&["type"])? {
        Node::Str(word) => Some(word.clone()),
        _ => None,
    }
}

fn result_node(calc: &Calc, checked: Option<bool>) -> Node {
    let dir = calc.dir;
    Node::Val(match (calc.result, dir) {
        (Value::Money(p), Some(dir)) => Val::Diff {
            size: magnitude(p),
            dir,
        },
        (Value::Money(p), None) => Val::Money(Money::plain(p)),
        (Value::Balance(p), _) => Val::Money(Money {
            paisa: magnitude(p),
            side: Some(node_side(p)),
            checked,
            places: Vec::new(),
        }),
        (Value::Count(n), Some(dir)) => Val::CountDiff {
            size: n.checked_abs().unwrap_or(i64::MAX),
            dir,
        },
        (Value::Count(n), None) => Val::Count(n),
        (Value::Percent(h), Some(dir)) => Val::Change { hundredths: h, dir },
        (Value::Percent(h), None) => Val::Pct(h),
    })
}

/// Whose figure each input path is: the single item, seller, buyer or party its result was read
/// for. Inputs of a result with no such filter (or a question number) have none and are left out.
fn scopes_of(state: &State, paths: &[String]) -> Vec<String> {
    use crate::answer::AnswerSource;
    paths
        .iter()
        .filter_map(|path| {
            let root = path.split('.').next()?;
            let result = state.find(root)?;
            let key = |kind: &str, id: Option<i64>| id.map(|id| format!("{kind}{id}"));
            match result.source.as_ref()? {
                AnswerSource::Analytics { filters, .. } => {
                    let parts: Vec<String> = [
                        key("i", filters.item.as_ref().map(|r| r.id)),
                        key("s", filters.seller.as_ref().map(|r| r.id)),
                        key("b", filters.buyer.as_ref().map(|p| p.record.id)),
                    ]
                    .into_iter()
                    .flatten()
                    .collect();
                    Some(if parts.is_empty() {
                        "*".to_owned()
                    } else {
                        parts.join("|")
                    })
                }
                AnswerSource::Report { filters, .. } => {
                    let parts: Vec<String> = [
                        key("p", filters.party.as_ref().map(|p| p.record.id)),
                        key("s", filters.seller.as_ref().map(|r| r.id)),
                        key("i", filters.item.as_ref().map(|r| r.id)),
                    ]
                    .into_iter()
                    .flatten()
                    .collect();
                    Some(if parts.is_empty() {
                        "*".to_owned()
                    } else {
                        parts.join("|")
                    })
                }
                AnswerSource::PartyCard { party, .. } => Some(format!("p{}", party.id)),
                AnswerSource::Tiles { .. } => None,
            }
        })
        .collect()
}

/// The tool `calculate`.
pub fn run_calculate(env: &Env<'_>, state: &State, args: &Json) -> Result<Draft, Stop> {
    let parsed: CalcInput = input(args)?;
    if parsed.inputs.is_empty() {
        return Err(bad("ERR_NO_INPUTS", "inputs", "no inputs").into());
    }
    let mut pending = Vec::new();
    for path in &parsed.inputs {
        pending.extend(expand(env, state, path, "inputs")?);
    }
    if parsed.op == OpWord::PercentOf {
        pending = buyer_balance_over_an_amount(env, state, &parsed.inputs, pending)?;
    }
    let (values, checked) = settle(pending)?;
    // Figures that belong to different items or parties (or one item and the unfiltered total) are
    // combined only when the question asks for that kind of figure: a sum or an average when it
    // says together (mila kar, total), a difference when it compares, a percent when it asks for a
    // share or a change. Otherwise each is asked for on its own, and a result of the combination
    // would show as if it were one item's figure (Q112; robustness_cues: 9,376 and 2,724 for
    // "kela ka commission").
    let wanted: Option<(bool, &str)> = match parsed.op {
        OpWord::Sum | OpWord::Average => Some((
            cues::together(env.question),
            "the question does not ask for them together (mila kar, total)",
        )),
        OpWord::Difference => Some((
            cues::comparing(env.question),
            "the question does not compare them (farq, difference, zyada, kam)",
        )),
        OpWord::PercentOf | OpWord::ChangePercent => Some((
            cues::share_or_change(env.question),
            "the question does not ask for a share or a change in percent",
        )),
        OpWord::PercentMore | OpWord::PercentLess => None,
    };
    if let Some((asked, why)) = wanted
        && !asked
    {
        let scopes = scopes_of(state, &parsed.inputs);
        let mut distinct: Vec<&String> = Vec::new();
        for scope in &scopes {
            if !distinct.contains(&scope) {
                distinct.push(scope);
            }
        }
        if distinct.len() >= 2 {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "inputs",
                &format!(
                    "these figures belong to different items or parties, and {why}; give each \
                     one's own figure, read for that item or party"
                ),
            )
            .into());
        }
    }
    // "10 percent more" is the Owner's own what-if: the question must be a what-if (agar, if),
    // hold one percent number and say more (or less) right after it. A rate ("7% hota to") or a
    // past fact ("pichle saal se 10% zyada hai") is not a figure to scale.
    let percents = env
        .numbers
        .iter()
        .filter(|n| n.unit == NumberUnit::Percent)
        .count();
    let said = cues::percent_change(env.question, percents);
    match (parsed.op, said) {
        (OpWord::PercentMore, Some(cues::PercentChange::More))
        | (OpWord::PercentLess, Some(cues::PercentChange::Less))
        | (
            OpWord::Sum
            | OpWord::Difference
            | OpWord::Average
            | OpWord::PercentOf
            | OpWord::ChangePercent,
            _,
        ) => {}
        (OpWord::PercentMore | OpWord::PercentLess, _) => {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "op",
                "a percent more or less needs a what-if question (agar, if) with one percent \
                 number and the word more or less right after it; ask the Owner what he means",
            )
            .into());
        }
    }
    let calc = figures::calculate(parsed.op.calc(), &values).map_err(service_stop)?;
    let node = Node::obj(vec![
        ("op", Node::Str(parsed.op.word().to_owned())),
        (
            "inputs",
            Node::List(parsed.inputs.iter().cloned().map(Node::Str).collect()),
        ),
        ("result", result_node(&calc, checked)),
    ]);
    let mut draft = Draft::new("calculate", args.clone(), node);
    draft.working = Some(working_of(parsed.op.title_id(), &calc.steps));
    Ok(draft)
}

// ---- estimate --------------------------------------------------------------------------------

/// What is estimated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Measure {
    /// Sales (خام بکری).
    Sales,
    /// All commission and لاگا.
    CommissionAll,
    /// Seller commission.
    CommissionSeller,
    /// Buyer commission.
    CommissionBuyer,
    /// لاگا.
    Laga,
    /// Shop expenses.
    ShopExpenses,
    /// Profit.
    Profit,
    /// Receipts.
    Receipts,
    /// Garden sales.
    GardenSales,
    /// Trip sales.
    TripSales,
}

impl Measure {
    fn word(self) -> &'static str {
        match self {
            Measure::Sales => "sales",
            Measure::CommissionAll => "commission_all",
            Measure::CommissionSeller => "commission_seller",
            Measure::CommissionBuyer => "commission_buyer",
            Measure::Laga => "laga",
            Measure::ShopExpenses => "shop_expenses",
            Measure::Profit => "profit",
            Measure::Receipts => "receipts",
            Measure::GardenSales => "garden_sales",
            Measure::TripSales => "trip_sales",
        }
    }

    /// The Fluent message of what is estimated (the frontend's `assistant-measure-*`).
    fn measure_id(self) -> String {
        match self {
            Measure::Sales => "assistant-measure-sales",
            Measure::CommissionAll => "assistant-measure-commission-all",
            Measure::CommissionSeller => "assistant-measure-commission-seller",
            Measure::CommissionBuyer => "assistant-measure-commission-buyer",
            Measure::Laga => "assistant-measure-commission-laga",
            Measure::ShopExpenses => "assistant-measure-expenses",
            Measure::Profit => "assistant-measure-profit",
            Measure::Receipts => "assistant-measure-receipts",
            Measure::GardenSales => "assistant-measure-garden-sales",
            Measure::TripSales => "assistant-measure-trip-sales",
        }
        .to_owned()
    }

    /// The column of view G and the side it is read with. `None` for a measure the books give no
    /// monthly series of yet.
    fn column(self) -> Option<(&'static str, AnalyticsSide)> {
        match self {
            Measure::Sales => Some(("gross", AnalyticsSide::Both)),
            Measure::CommissionAll => Some(("commission", AnalyticsSide::Both)),
            Measure::CommissionSeller => Some(("commission", AnalyticsSide::Seller)),
            Measure::ShopExpenses => Some(("shop_expenses", AnalyticsSide::Both)),
            Measure::Profit => Some(("profit", AnalyticsSide::Both)),
            _ => None,
        }
    }
}

/// The input of `estimate`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EstimateInput {
    /// What is estimated.
    pub measure: Measure,
    /// An item.
    pub item: Option<String>,
    /// A seller.
    pub seller: Option<String>,
    /// A buyer.
    pub buyer: Option<String>,
    /// A garden.
    pub garden: Option<String>,
    /// A trip.
    pub trip: Option<String>,
    /// Months ahead, 1 to 12. Null with `until`.
    pub ahead: Option<i64>,
    /// `year_end`: from the current month to December. Only when the question says so.
    pub until: Option<String>,
    /// A what-if on the pace: `double`, `half`, `triple`, `up` or `down`. Only when the question
    /// says so (plan 52 item 5).
    pub what_if: Option<String>,
    /// For `up` and `down`: the percent, a number of the question (`q.n1`).
    pub percent: Option<String>,
    /// A number from the question (`q.n1`) or a figure already read.
    pub target: Option<String>,
}

/// True when the estimate runs to the end of the year: the model said `until: "year_end"`, or it
/// sent neither `ahead` nor `until` and the question says "to the end of the year" and names no
/// number of months (plan 52 P52-S1, the time analysis: 14 questions lost a call to this).
fn year_end_asked(env: &Env<'_>, parsed: &EstimateInput) -> bool {
    if parsed.until.is_some() {
        return true;
    }
    let months = env.numbers.iter().any(|n| n.unit == NumberUnit::Months);
    parsed.ahead.is_none() && !months && cues::year_end(env.question)
}

/// Months ahead: a number of the question with months, or 6 (12 with a target) when the question
/// has none; or, with `until: "year_end"`, the months left in the year when the question says "to
/// the end of the year" (plan 52 item 5). A year end together with a months number is refused:
/// which one is meant is the Owner's to say. When the model sends no `ahead` the app takes what the
/// tool's own rule says: the one months number of the question, else 6, or 12 with a target.
fn months_ahead(
    env: &Env<'_>,
    parsed: &EstimateInput,
    current: YearMonth,
) -> Result<u32, ToolError> {
    let months: Vec<&QuestionNumber> = env
        .numbers
        .iter()
        .filter(|n| n.unit == NumberUnit::Months)
        .collect();
    if year_end_asked(env, parsed) {
        if let Some(until) = &parsed.until {
            if until != "year_end" {
                return Err(bad("ERR_OUT_OF_RANGE", "until", "only year_end"));
            }
            if !cues::year_end(env.question) {
                return Err(bad(
                    "ERR_OUT_OF_RANGE",
                    "until",
                    "the question does not say to the end of the year; ask the Owner how far ahead",
                ));
            }
        }
        if parsed.ahead.is_some() {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "ahead",
                "with until year_end, leave ahead out",
            ));
        }
        if !months.is_empty() {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "until",
                "the question has both a number of months and the end of the year; ask the Owner \
                 which one he means",
            ));
        }
        // A year target could count the books' months so far or only the months ahead: the two
        // readings give opposite answers, so the Owner is asked (money review).
        if parsed.target.is_some() {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "target",
                "a target with the end of the year can mean two things (counting the months already \
                 in the books, or only the months ahead); ask the Owner which one he means",
            ));
        }
        return mandi_services::domain::forecast::months_to_year_end(current)
            .map_err(|e| ToolError::from_code(e.code(), Some("until")));
    }
    let has_target = parsed.target.is_some();
    // "ek saal" (one year ahead) is 12 months: the question holds no digit, so no months number
    // (Q285 and Q286 of the live run of 03-11-2026).
    let default_ahead: i64 = if has_target || cues::year_ahead(env.question) {
        12
    } else {
        6
    };
    let ahead = match parsed.ahead {
        Some(ahead) => ahead,
        // Nothing sent: the question's own months number, else the default of the rule.
        None => match months.as_slice() {
            [] => default_ahead,
            [only] => only.number,
            _ => {
                return Err(bad(
                    "ERR_REQUIRED",
                    "ahead",
                    "the question has more than one number of months; ask the Owner which one",
                ));
            }
        },
    };
    if months.is_empty() && cues::year_end(env.question) {
        return Err(bad(
            "ERR_OUT_OF_RANGE",
            "ahead",
            "the question says to the end of the year: use until year_end, not a number of months",
        ));
    }
    let allowed = if months.is_empty() {
        ahead == default_ahead
    } else {
        months.iter().any(|n| n.number == ahead)
    };
    if !allowed {
        return Err(bad(
            "ERR_OUT_OF_RANGE",
            "ahead",
            "months ahead must be a number of the question with months; with none, 6 (12 for a target or one year)",
        ));
    }
    u32::try_from(ahead)
        .ok()
        .filter(|a| (1..=crate::limits::MAX_MONTHS_AHEAD).contains(a))
        .ok_or_else(|| bad("ERR_OUT_OF_RANGE", "ahead", "1 to 12"))
}

/// The factor of a what-if, from the word the question holds (double, half, triple) or from its
/// percent with the word for more or less. The model never sends a factor: it names the kind, and
/// the app checks that the Owner said it (plan 52 item 5, money review).
fn pace_factor(env: &Env<'_>, parsed: &EstimateInput) -> Result<Option<PaceFactor>, ToolError> {
    let Some(kind) = parsed.what_if.as_deref() else {
        if parsed.percent.is_some() {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "percent",
                "percent goes with what_if up or down",
            ));
        }
        return Ok(None);
    };
    // A scenario is a "what if": the question must say so, or a past fact ("commission doubled
    // last year") could pass for a scenario the Owner did not ask for.
    if !cues::conditional(env.question) {
        return Err(bad(
            "ERR_OUT_OF_RANGE",
            "what_if",
            "the question does not ask a what if (agar, if); leave what_if out",
        ));
    }
    let said = |word: &str, wanted: cues::Scenario| -> Result<(), ToolError> {
        if cues::scenarios_clash(env.question) {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "what_if",
                "the question has more than one of double, half and triple; ask the Owner which",
            ));
        }
        if cues::scenario(env.question) != Some(wanted) {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "what_if",
                &format!("the question does not say {word}; leave what_if out or ask the Owner"),
            ));
        }
        Ok(())
    };
    let no_percent = || -> Result<(), ToolError> {
        if parsed.percent.is_some() {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "percent",
                "percent goes with up or down only",
            ));
        }
        Ok(())
    };
    let factor = match kind {
        "double" => {
            said("double", cues::Scenario::Double)?;
            no_percent()?;
            PaceFactor::DOUBLE
        }
        "half" => {
            said("half", cues::Scenario::Half)?;
            no_percent()?;
            PaceFactor::HALF
        }
        "triple" => {
            said("triple", cues::Scenario::Triple)?;
            no_percent()?;
            PaceFactor::TRIPLE
        }
        "up" | "down" => {
            let path = parsed.percent.as_deref().ok_or_else(|| {
                bad(
                    "ERR_REQUIRED",
                    "percent",
                    "up and down need the percent of the question",
                )
            })?;
            let number = question_number(env, path)
                .filter(|n| n.unit == NumberUnit::Percent)
                .ok_or_else(|| {
                    bad(
                        "ERR_OUT_OF_RANGE",
                        "percent",
                        "a number of the question with a % sign",
                    )
                })?;
            let bps = number.bps.ok_or_else(|| {
                bad(
                    "ERR_OUT_OF_RANGE",
                    "percent",
                    "a number of the question with a % sign",
                )
            })?;
            let (more, less) = (cues::says_more(env.question), cues::says_less(env.question));
            if kind == "up" {
                if !more || less {
                    return Err(bad(
                        "ERR_OUT_OF_RANGE",
                        "what_if",
                        "the question must say more (and not less); ask the Owner up or down",
                    ));
                }
                PaceFactor::up(bps)
            } else {
                if !less || more {
                    return Err(bad(
                        "ERR_OUT_OF_RANGE",
                        "what_if",
                        "the question must say less (and not more); ask the Owner up or down",
                    ));
                }
                PaceFactor::down(bps)
            }
            .map_err(|e| ToolError::from_code(e.code(), Some("percent")))?
        }
        _ => {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "what_if",
                "double, half, triple, up or down",
            ));
        }
    };
    Ok(Some(factor))
}

/// The target: a number of the question (rupees) or a figure already read.
fn read_target(env: &Env<'_>, state: &State, path: &str) -> Result<Paisa, ToolError> {
    if let Some(number) = question_number(env, path) {
        return match number.unit {
            NumberUnit::Plain | NumberUnit::Amount => Paisa::from_rupees(number.number)
                .map_err(|_| bad("ERR_OVERFLOW", "target", "the number does not fit")),
            _ => Err(bad("ERR_OUT_OF_RANGE", "target", "the target is an amount")),
        };
    }
    let found = lookup(env, state, path, "target")?;
    match found.node() {
        Node::Val(Val::Money(m)) => Ok(magnitude(m.paisa)),
        _ => Err(bad(
            "ERR_FORMAT_INVALID",
            "target",
            "the target is an amount",
        )),
    }
}

fn year_month(year: i64, month: i64) -> Option<YearMonth> {
    YearMonth::new(i32::try_from(year).ok()?, u32::try_from(month).ok()?)
}

fn answer_month(m: YearMonth) -> AnswerMonth {
    AnswerMonth {
        year: i64::from(m.year()),
        month: i64::from(m.month()),
    }
}

fn month_node(m: YearMonth) -> Node {
    Node::Val(Val::Month {
        year: m.year(),
        month: m.month(),
    })
}

fn key_of_month(m: YearMonth) -> String {
    format!("m{}{:02}", m.year(), m.month())
}

/// The full months before `today`, oldest first, from view G by month; a month with nothing is 0.
fn read_history(
    env: &Env<'_>,
    state: &State,
    parsed: &EstimateInput,
    current: YearMonth,
    column: &str,
    side: AnalyticsSide,
) -> Result<(Vec<MonthValue>, AnalyticsFilters, SourceAnalyticsFilters), Stop> {
    let mut filters = AnalyticsFilters {
        group_by: Some(AnalyticsGroupBy::Month),
        side,
        ..AnalyticsFilters::default()
    };
    let mut source = SourceAnalyticsFilters {
        group_by: Some(AnalyticsGroupBy::Month),
        side,
        ..SourceAnalyticsFilters::default()
    };
    if let Some(text) = &parsed.item {
        let (id, record) = named(env, state, text, "item", &["i"])?;
        filters.item_id = Some(id);
        source.item = Some(record);
    }
    if let Some(text) = &parsed.seller {
        let (id, record) = named(env, state, text, "seller", &["a"])?;
        require_type(env, id, "seller", AccountType::Seller)?;
        filters.seller_id = Some(id);
        source.seller = Some(record);
    }
    if let Some(text) = &parsed.buyer {
        let (id, record) = named(env, state, text, "buyer", &["a"])?;
        require_type(env, id, "buyer", AccountType::Buyer)?;
        filters.buyer = Some(PartyFilter {
            account_id: id,
            person_key: None,
        });
        source.buyer = Some(crate::source::SourceParty {
            record,
            person_key: None,
        });
    }
    if let Some(text) = &parsed.garden {
        let (id, record) = named(env, state, text, "garden", &["g"])?;
        filters.garden_id = Some(id);
        source.garden = Some(record);
    }
    if let Some(text) = &parsed.trip {
        let (id, record) = named(env, state, text, "trip", &["t"])?;
        filters.trip_id = Some(id);
        source.trip = Some(record);
    }
    // The books up to the last day of the month before the current one.
    let first_of_current = NaiveDate::from_ymd_opt(current.year(), current.month(), 1);
    let last_full = first_of_current.and_then(|d| d.pred_opt());
    if let Some(last) = last_full {
        let iso = last.format("%Y-%m-%d").to_string();
        filters.to = iso.clone();
        source.to = iso;
    }
    let page = engine_run(
        env.ctx.db(),
        &AnalyticsRunInput {
            view: AnalyticsView::G,
            filters: filters.clone(),
            offset: 0,
            limit: 1_000,
            sort: None,
        },
    )
    .map_err(reports_stop)?;
    let at = page
        .columns
        .iter()
        .position(|c| c.key == column)
        .ok_or_else(|| bad("ERR_OUT_OF_RANGE", "measure", "no monthly series"))?;
    let mut series: Vec<(YearMonth, Paisa)> = Vec::new();
    for row in &page.rows {
        let (Some(Cell::Month { year, month }), Some(cell)) =
            (row.cells.first(), row.cells.get(at))
        else {
            continue;
        };
        let Some(m) = year_month(*year, *month) else {
            continue;
        };
        let paisa = match cell {
            Cell::Money { paisa } => *paisa,
            // The column is empty when a sale filter narrows it (the share rows are kept for
            // each posting, not for each lot).
            _ => {
                return Err(bad(
                    "ERR_OUT_OF_RANGE",
                    "measure",
                    "this measure is not available with these filters",
                )
                .into());
            }
        };
        series.push((m, paisa));
    }
    series.sort_by_key(|(m, _)| *m);
    let mut history = Vec::new();
    if let (Some((first, _)), Some(prev)) = (series.first().copied(), previous_month(current)) {
        let mut month = first;
        while month <= prev {
            let value = series
                .iter()
                .find(|(m, _)| *m == month)
                .map_or(Paisa::ZERO, |(_, v)| *v);
            history.push(MonthValue { month, value });
            match month.plus_months(1) {
                Some(next) => month = next,
                None => break,
            }
        }
    }
    Ok((history, filters, source))
}

fn previous_month(m: YearMonth) -> Option<YearMonth> {
    // The month before: 11 months ahead of the month a year earlier.
    YearMonth::new(m.year().checked_sub(1)?, m.month())?.plus_months(11)
}

/// The methods in the order the result gives them.
const METHODS: [Method; 4] = [
    Method::SamePace,
    Method::WhatIfPace,
    Method::SameGrowth,
    Method::SameSeason,
];

fn method_key(method: Method) -> &'static str {
    match method {
        Method::SamePace => "same_pace",
        Method::SameGrowth => "same_growth",
        Method::SameSeason => "same_season",
        Method::WhatIfPace => "what_if_pace",
    }
}

fn method_kind(method: Method) -> EstimateMethodKind {
    match method {
        Method::SamePace => EstimateMethodKind::SamePace,
        Method::SameGrowth => EstimateMethodKind::SameGrowth,
        Method::SameSeason => EstimateMethodKind::SameSeason,
        Method::WhatIfPace => EstimateMethodKind::WhatIfPace,
    }
}

fn chart_kind(method: Method) -> ChartSeriesKind {
    match method {
        Method::SamePace => ChartSeriesKind::SamePace,
        Method::SameGrowth => ChartSeriesKind::SameGrowth,
        Method::SameSeason => ChartSeriesKind::SameSeason,
        Method::WhatIfPace => ChartSeriesKind::WhatIfPace,
    }
}

fn money_cell(p: Paisa) -> Cell {
    Cell::Money { paisa: p }
}

fn plain_money(p: Paisa) -> Node {
    Node::Val(Val::Money(Money::plain(p)))
}

fn method_node(m: &MethodEstimate) -> Node {
    let mut fields = vec![
        (
            "months_used",
            Node::Val(Val::Count(i64::from(m.months_used))),
        ),
        ("total", plain_money(m.total)),
        (
            "target_month",
            m.target_month.map_or(Node::Null, month_node),
        ),
    ];
    // The whole year (to the end of the year): the books so far plus the total.
    if let Some(whole) = m.year_total {
        fields.insert(2, ("year_total", plain_money(whole)));
    }
    if let Some(p) = m.per_month {
        fields.insert(1, ("per_month", plain_money(p)));
    }
    if let Some((size, dir)) = m.step {
        fields.insert(1, ("step", Node::Val(Val::Diff { size, dir })));
    }
    // A what-if: the factor as a percent the model may cite, since it may not write "double".
    if let Some(factor) = m.factor {
        fields.insert(1, ("factor", Node::Val(Val::Pct(i64::from(factor.bps())))));
    }
    Node::obj(fields)
}

fn method_estimate(m: &MethodEstimate, title_id: &str) -> EstimateMethod {
    EstimateMethod {
        kind: method_kind(m.method),
        months_used: i64::from(m.months_used),
        used_from: answer_month(m.used_from),
        used_to: answer_month(m.used_to),
        per_month: m.per_month.map(money_cell),
        step: m.step.map(|(size, _)| money_cell(size)),
        step_dir: m.step.map(|(_, dir)| figure_dir(dir)),
        total: money_cell(m.total),
        target_month: m.target_month.map(answer_month),
        factor: m.factor.map(|f| Cell::Percent {
            hundredths: i64::from(f.bps()),
        }),
        year_total: m.year_total.map(money_cell),
        working: working_of(title_id, &m.steps),
    }
}

fn working_title(method: Method) -> &'static str {
    match method {
        Method::SamePace => "assistant-working-same-pace",
        Method::SameGrowth => "assistant-working-same-growth",
        Method::SameSeason => "assistant-working-same-season",
        Method::WhatIfPace => "assistant-working-what-if-pace",
    }
}

/// The tool `estimate`.
pub fn run_estimate(env: &Env<'_>, state: &State, args: &Json) -> Result<Draft, Stop> {
    let parsed: EstimateInput = input(args)?;
    let (column, side) = parsed.measure.column().ok_or_else(|| {
        bad(
            "ERR_OUT_OF_RANGE",
            "measure",
            "the books give no monthly series of this measure yet",
        )
    })?;
    let today = NaiveDate::parse_from_str(&env.today.to_iso(), "%Y-%m-%d")
        .map_err(|_| bad("ERR_INTERNAL", "today", "the date is wrong"))?;
    let current = YearMonth::new(today.year(), today.month())
        .ok_or_else(|| bad("ERR_INTERNAL", "today", "the date is wrong"))?;
    let ahead = months_ahead(env, &parsed, current)?;
    let what_if = pace_factor(env, &parsed)?;
    let target = parsed
        .target
        .as_deref()
        .map(|t| read_target(env, state, t))
        .transpose()?;
    let (history, filters, source) = read_history(env, state, &parsed, current, column, side)?;
    let history_nodes: Vec<Node> = history
        .iter()
        .map(|h| {
            Node::obj(vec![
                ("key", Node::Str(key_of_month(h.month))),
                ("month", month_node(h.month)),
                ("value", plain_money(h.value)),
            ])
        })
        .collect();
    let to_year_end = year_end_asked(env, &parsed);
    let worked = if to_year_end {
        forecast::estimate_to_year_end(&history, current, what_if)
    } else {
        forecast::estimate_with(&history, current, ahead, target, what_if)
    };
    let found = worked.map_err(|e| {
        match service_stop(e) {
            // A what-if on a pace below 0 (a loss) or a factor out of range: say so, so the model
            // asks again without it instead of losing the whole estimate.
            Stop::Error(mut error) if what_if.is_some() && error.code == "ERR_OUT_OF_RANGE" => {
                error.detail = "the what-if cannot be worked out: the pace is below 0 (a loss), or \
                                the factor is out of range; leave what_if out for the plain estimate"
                    .to_owned();
                Stop::Error(error)
            }
            other => other,
        }
    })?;
    let _ = filters;
    let mut fields = vec![
        ("measure", Node::Str(parsed.measure.word().to_owned())),
        ("history", Node::List(history_nodes)),
    ];
    let mut draft;
    match found {
        forecast::Forecast::TooShort { full_months } => {
            fields.insert(1, ("too_short", Node::Bool(true)));
            fields.insert(
                2,
                ("full_months", Node::Val(Val::Count(i64::from(full_months)))),
            );
            fields.push(("same_pace", Node::Null));
            fields.push(("same_growth", Node::Null));
            fields.push(("same_season", Node::Null));
            if to_year_end {
                fields.push(("year", Node::Null));
            }
            fields.push(("ahead", Node::List(Vec::new())));
            draft = Draft::new("estimate", args.clone(), Node::obj(fields));
        }
        forecast::Forecast::Estimate(e) => {
            fields.insert(1, ("too_short", Node::Bool(false)));
            fields.insert(
                2,
                (
                    "full_months",
                    Node::Val(Val::Count(i64::from(e.full_months))),
                ),
            );
            fields.push(("first", month_node(e.first)));
            fields.push(("last", month_node(e.last)));
            fields.push(("ahead_count", Node::Val(Val::Count(i64::from(e.ahead)))));
            if let Some(t) = target {
                fields.push(("target", plain_money(t)));
            }
            // To the end of the year: the books' part of the whole year, read, not estimated.
            if let Some(year) = &e.year {
                fields.push((
                    "year",
                    Node::obj(vec![
                        ("so_far", plain_money(year.so_far)),
                        ("months", Node::Val(Val::Count(i64::from(year.months)))),
                        (
                            "from",
                            year.span.map_or(Node::Null, |(from, _)| month_node(from)),
                        ),
                        ("to", year.span.map_or(Node::Null, |(_, to)| month_node(to))),
                    ]),
                ));
            }
            for method in METHODS {
                // The what-if is only there when it was asked for.
                if method == Method::WhatIfPace && what_if.is_none() {
                    continue;
                }
                let node = e
                    .methods
                    .iter()
                    .find(|m| m.method == method)
                    .map_or(Node::Null, method_node);
                fields.push((method_key(method), node));
            }
            // The months ahead, each with the figure of every method that was worked out.
            let mut rows = Vec::new();
            for k in 0..e.ahead {
                let Some(month) = e.first.plus_months(k) else {
                    continue;
                };
                let at = usize::try_from(k).unwrap_or(0);
                let mut row = vec![("month", month_node(month))];
                for method in METHODS {
                    if method == Method::WhatIfPace && what_if.is_none() {
                        continue;
                    }
                    let node = e
                        .methods
                        .iter()
                        .find(|m| m.method == method)
                        .and_then(|m| m.months.get(at))
                        .map_or(Node::Null, |p| plain_money(*p));
                    row.push((method_key(method), node));
                }
                rows.push((format!("ah{}", at.saturating_add(1)), Node::obj(row)));
            }
            fields.push(("ahead", Node::Keyed(rows)));
            draft = Draft::new("estimate", args.clone(), Node::obj(fields));
            // The chart: the months of the books, then the months ahead, one line for each method.
            let mut months: Vec<AnswerMonth> =
                history.iter().map(|h| answer_month(h.month)).collect();
            let past = months.len();
            for k in 0..e.ahead {
                if let Some(m) = e.first.plus_months(k) {
                    months.push(answer_month(m));
                }
            }
            let mut series = vec![ChartSeries {
                kind: ChartSeriesKind::Actual,
                values: history
                    .iter()
                    .map(|h| money_cell(h.value))
                    .chain((0..e.ahead).map(|_| Cell::Empty))
                    .collect(),
            }];
            for m in &e.methods {
                series.push(ChartSeries {
                    kind: chart_kind(m.method),
                    values: (0..past)
                        .map(|_| Cell::Empty)
                        .chain(m.months.iter().map(|p| money_cell(*p)))
                        .collect(),
                });
            }
            let measure_id = parsed.measure.measure_id();
            let chart = AnswerChart {
                result_id: String::new(),
                measure_id: measure_id.clone(),
                months,
                series,
                is_estimate: true,
            };
            let estimate = AnswerEstimate {
                result_id: String::new(),
                measure_id,
                full_months: i64::from(e.full_months),
                first: answer_month(e.first),
                last: answer_month(e.last),
                target: target.map(money_cell),
                methods: e
                    .methods
                    .iter()
                    .map(|m| method_estimate(m, working_title(m.method)))
                    .collect(),
                year: e.year.as_ref().map(|year| EstimateYear {
                    so_far: money_cell(year.so_far),
                    months: i64::from(year.months),
                    from: year.span.map(|(from, _)| answer_month(from)),
                    to: year.span.map(|(_, to)| answer_month(to)),
                }),
            };
            draft.shown = Some(Shown::Chart(Box::new(chart), Some(Box::new(estimate))));
        }
    }
    draft.source = Some(AnswerSource::Analytics {
        result_id: String::new(),
        label_id: "analytics-view-g-title".to_owned(),
        view: AnalyticsView::G,
        filters: source,
    });
    Ok(draft)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn balance(rupees: i64) -> Value {
        Value::Balance(Paisa::from_rupees(rupees).unwrap())
    }

    #[test]
    fn the_result_is_checked_only_when_every_input_is_a_balance_checked_in_two_places() {
        let checked = Pending::Known(balance(2_880), Some(true));
        let unchecked = Pending::Known(balance(10_000), None);
        let different = Pending::Known(balance(500), Some(false));
        // Both checked: checked.
        let (_, c) = settle(vec![
            Pending::Known(balance(2_880), Some(true)),
            Pending::Known(balance(500), Some(true)),
        ])
        .unwrap();
        assert_eq!(c, Some(true));
        // A balance with no second place beside a checked one: no check mark.
        let (_, c) = settle(vec![checked, unchecked]).unwrap();
        assert_eq!(c, None);
        // A balance whose two places differ: the result says so, whatever stands beside it.
        let (_, c) = settle(vec![Pending::Known(balance(2_880), Some(true)), different]).unwrap();
        assert_eq!(c, Some(false));
        // A typed number beside a checked balance: no check mark.
        let (_, c) = settle(vec![
            Pending::Known(balance(2_880), Some(true)),
            Pending::Plain(48_000),
        ])
        .unwrap();
        assert_eq!(c, None);
    }

    #[test]
    fn a_plain_number_beside_balances_that_are_all_zero_has_no_side_and_is_refused() {
        let refused = settle(vec![
            Pending::Known(balance(0), Some(true)),
            Pending::Plain(100),
        ]);
        assert_eq!(refused.unwrap_err().code, "ERR_OUT_OF_RANGE");
        // Beside one real side it takes that side; a credit makes it a credit.
        let (values, _) = settle(vec![
            Pending::Known(balance(-10_160), Some(true)),
            Pending::Plain(100),
        ])
        .unwrap();
        assert_eq!(values.get(1), Some(&balance(-100)));
    }
}
