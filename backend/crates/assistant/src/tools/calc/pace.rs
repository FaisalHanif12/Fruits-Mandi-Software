//! The estimate of a balance at the same pace (plan 55 F4 and F5, PRD FR-7.86): a party's balance,
//! the cash in hand, what the buyers owe.
//!
//! A balance is a level, so the answer is the balance NOW (a book figure, read by the very function
//! of its screen) plus the average net change of a month times the months ahead (an estimate).
//! The arithmetic is `domain::forecast::balance_at_pace`; the months of change come from
//! `mandi_reports::analytics::balance_series`. Nothing is posted or saved.
//!
//! What the model reads is the same shape as any estimate, so the checks and the answer card need
//! no change: `same_pace.per_month`, `.total`, and a row `ahead.ahN.same_pace` for the balance after
//! N months. The balance now is `now`. A change is a difference with its direction word; the
//! balances carry their side only for a party (the cash in hand and the dues are plain amounts).
//!
//! Example (fixture B, today 10-10-26): Akram's balance is 137,400 بنام; his monthly change
//! of the last 3 months averaged 35,800 زیادہ, so `ahead.ah6.same_pace` is 352,200 بنام.

use mandi_reports::analytics::{BalanceSubject, balance_series};
use mandi_reports::reports::ReportId;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::money::Paisa;
use mandi_services::domain::figures::Dir;
use mandi_services::domain::forecast::{BalanceEstimate, BalanceKind, MonthValue, YearMonth};
use mandi_services::forecast::{BalanceForecast, balance_at_pace};
use mandi_services::{accounts, assist};
use serde_json::Value as Json;

use super::{
    AnswerChart, AnswerEstimate, AnswerMonth, AnswerSource, ChartSeries, ChartSeriesKind, Draft,
    Env, EstimateInput, EstimateMethod, EstimateMethodKind, Money, Node, Shown,
    SourceReportFilters, State, Stop, ToolError, Val, answer_month, bad, figure_dir, key_of_month,
    magnitude, money_cell, month_node, months_ahead, named, node_side, report_label, service_stop,
    side_of, working_of, year_end_asked,
};
use crate::source::AnswerRecord;

/// The Fluent id of the title of a balance's working. It says what the pace leaves out: a party's
/// and the cash's openings (the cash's also the capital put in or taken out), the dues' openings
/// and moves to the dead amount ("one-off moves left out", PRD FR-7.86 v1.27).
fn working_title(kind: BalanceKind) -> &'static str {
    match kind {
        BalanceKind::Party => "assistant-working-balance-pace",
        BalanceKind::Cash => "assistant-working-balance-pace-cash",
        BalanceKind::Dues => "assistant-working-balance-pace-dues",
    }
}
/// The Fluent id of the monthly change, the chart's measure.
const CHANGE_MEASURE: &str = "assistant-measure-balance-change";

/// A balance as the model reads it: a party's with its side, the others as plain signed amounts.
fn balance_node(kind: BalanceKind, paisa: Paisa) -> Node {
    match kind {
        BalanceKind::Party => Node::Val(Val::Money(Money::balance(
            magnitude(paisa),
            node_side(paisa),
        ))),
        BalanceKind::Cash | BalanceKind::Dues => Node::Val(Val::Money(Money::plain(paisa))),
    }
}

/// A balance as the card draws it.
fn balance_cell(kind: BalanceKind, paisa: Paisa) -> mandi_reports::reports::Cell {
    match kind {
        BalanceKind::Party => mandi_reports::reports::Cell::Balance {
            paisa: magnitude(paisa),
            side: side_of(paisa),
        },
        BalanceKind::Cash | BalanceKind::Dues => mandi_reports::reports::Cell::Money { paisa },
    }
}

/// A net change as the model reads it: its size and the word for its direction.
fn change_node(paisa: Paisa) -> Node {
    Node::Val(Val::Diff {
        size: magnitude(paisa),
        dir: Dir::of(paisa),
    })
}

/// The party of a balance estimate: his key must be a party the books can read, not the whole
/// پختہ کھاتہ account (it holds many people, each with his own balance, PRD FR-13.5).
fn party_of(env: &Env<'_>, state: &State, text: &str) -> Result<(i64, AnswerRecord), ToolError> {
    let (id, record) = named(env, state, text, "party", &["a"])?;
    let account = accounts::get(env.ctx, id)
        .map_err(|_| ToolError::new("ERR_NOT_FOUND", "party", "the account could not be read"))?;
    if account.is_pukhta_khata {
        return Err(bad(
            "ERR_OUT_OF_RANGE",
            "party",
            "a پختہ کھاتہ account holds many people, each with his own balance; pick the person",
        ));
    }
    // The card is read as the party card tool reads it, so the base is the figure the Owner sees.
    assist::party_card(env.ctx, id)
        .map_err(|_| ToolError::new("ERR_NOT_FOUND", "party", "the account could not be read"))?;
    Ok((id, record))
}

/// The tool `estimate` for a balance.
pub(super) fn run(
    env: &Env<'_>,
    state: &State,
    args: &Json,
    parsed: &EstimateInput,
    kind: BalanceKind,
    current: YearMonth,
) -> Result<Draft, Stop> {
    // A balance is estimated for a number of months from today, and for nothing else.
    if parsed.until.is_some() || year_end_asked(env, parsed) {
        return Err(bad(
            "ERR_OUT_OF_RANGE",
            "until",
            "a balance is estimated for a number of months from today, not to the end of the \
             year; ask the Owner how many months",
        )
        .into());
    }
    for (field, set) in [
        ("what_if", parsed.what_if.is_some()),
        ("percent", parsed.percent.is_some()),
        ("target", parsed.target.is_some()),
    ] {
        if set {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                field,
                "a balance has no what-if and no target; leave it out",
            )
            .into());
        }
    }
    let ahead = months_ahead(env, parsed, current)?;

    let (subject, party) = match kind {
        BalanceKind::Party => {
            let text = parsed
                .party
                .as_deref()
                .ok_or_else(|| bad("ERR_REQUIRED", "party", "the party"))?;
            let (id, record) = party_of(env, state, text)?;
            (BalanceSubject::Party(id), Some(record))
        }
        BalanceKind::Cash => (BalanceSubject::Cash, None),
        BalanceKind::Dues => (BalanceSubject::BuyersDues, None),
    };
    let series = balance_series(env.ctx.db(), subject, env.today).map_err(super::reports_stop)?;
    let mut changes = Vec::new();
    for m in &series.changes {
        let month = YearMonth::new(m.year, m.month)
            .ok_or_else(|| bad("ERR_INTERNAL", "history", "a month is wrong"))?;
        changes.push(MonthValue {
            month,
            value: m.amount,
        });
    }
    let found =
        balance_at_pace(kind, series.now, &changes, current, ahead).map_err(service_stop)?;

    let history_nodes: Vec<Node> = changes
        .iter()
        .map(|h| {
            Node::obj(vec![
                ("key", Node::Str(key_of_month(h.month))),
                ("month", month_node(h.month)),
                ("value", change_node(h.value)),
            ])
        })
        .collect();
    let mut fields = vec![
        ("measure", Node::Str(parsed.measure.word().to_owned())),
        ("history", Node::List(history_nodes)),
        // The balance now is a book figure; it is the only one in this result.
        ("now", balance_node(kind, series.now)),
    ];
    let mut draft;
    // The months the working used: the cash book button opens on them, so the monthly changes of the
    // working can be seen there (hidden round 4).
    let span: Option<(YearMonth, YearMonth)>;
    match found {
        // The dues are never below 0: say what happened, so the model asks the Owner for fewer
        // months instead of losing the whole estimate.
        BalanceForecast::FallsBelowZero => {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "ahead",
                "at this pace the buyers' dues would fall below 0 within these months, which dues \
                 cannot; ask the Owner for fewer months",
            )
            .into());
        }
        BalanceForecast::TooShort { full_months } => {
            fields.insert(1, ("too_short", Node::Bool(true)));
            fields.insert(
                2,
                ("full_months", Node::Val(Val::Count(i64::from(full_months)))),
            );
            fields.push(("same_pace", Node::Null));
            fields.push(("same_growth", Node::Null));
            fields.push(("same_season", Node::Null));
            fields.push(("ahead", Node::List(Vec::new())));
            draft = Draft::new("estimate", args.clone(), Node::obj(fields));
            span = changes
                .first()
                .zip(changes.last())
                .map(|(first, last)| (first.month, last.month));
        }
        BalanceForecast::Estimate(e) => {
            fields.insert(1, ("too_short", Node::Bool(false)));
            fields.insert(
                2,
                (
                    "full_months",
                    Node::Val(Val::Count(
                        i64::try_from(changes.len())
                            .map_err(|_| bad("ERR_INTERNAL", "history", "too many months"))?,
                    )),
                ),
            );
            fields.push(("first", month_node(e.first)));
            fields.push(("last", month_node(e.last)));
            fields.push(("ahead_count", Node::Val(Val::Count(i64::from(e.ahead)))));
            fields.push(("same_pace", method_node(&e)));
            fields.push(("same_growth", Node::Null));
            fields.push(("same_season", Node::Null));
            // The balance after each month: "after N months", labelled with the month that day is in.
            let mut rows = Vec::new();
            for (at, balance) in e.after.iter().enumerate() {
                let wrong = || {
                    bad(
                        "ERR_INTERNAL",
                        "ahead",
                        "a month ahead is outside the calendar",
                    )
                };
                let n = i64::try_from(at).map_err(|_| wrong())?.saturating_add(1);
                let month = e
                    .first
                    .plus_months(u32::try_from(at).map_err(|_| wrong())?)
                    .ok_or_else(wrong)?;
                rows.push((
                    format!("ah{n}"),
                    Node::obj(vec![
                        ("after", Node::Val(Val::Count(n))),
                        ("month", month_node(month)),
                        ("same_pace", balance_node(kind, *balance)),
                    ]),
                ));
            }
            fields.push(("ahead", Node::Keyed(rows)));
            draft = Draft::new("estimate", args.clone(), Node::obj(fields));
            span = Some((e.used_from, e.used_to));
            draft.shown = Some(Shown::Chart(
                Box::new(chart(&changes, &e)?),
                Some(Box::new(estimate_card(parsed, &changes, &e)?)),
            ));
        }
    }
    draft.source = Some(source_of(kind, party, subject, span, &env.today.to_iso()));
    Ok(draft)
}

/// The same pace of a balance as the model reads it.
fn method_node(e: &BalanceEstimate) -> Node {
    Node::obj(vec![
        (
            "months_used",
            Node::Val(Val::Count(i64::from(e.months_used))),
        ),
        ("per_month", change_node(e.per_month)),
        ("change", change_node(e.change)),
        ("total", balance_node(e.kind, e.total)),
        ("target_month", Node::Null),
    ])
}

/// The chart: the net change of each month of the books, then the pace for each month ahead.
fn chart(changes: &[MonthValue], e: &BalanceEstimate) -> Result<AnswerChart, ToolError> {
    let mut months: Vec<AnswerMonth> = changes.iter().map(|h| answer_month(h.month)).collect();
    let past = months.len();
    for k in 0..e.ahead {
        // A month ahead outside the calendar is a bug, never a month left out of the chart.
        let m = e.first.plus_months(k).ok_or_else(|| {
            bad(
                "ERR_INTERNAL",
                "ahead",
                "a month ahead is outside the calendar",
            )
        })?;
        months.push(answer_month(m));
    }
    let actual = ChartSeries {
        kind: ChartSeriesKind::Actual,
        values: changes
            .iter()
            .map(|h| money_cell(h.value))
            .chain((0..e.ahead).map(|_| mandi_reports::reports::Cell::Empty))
            .collect(),
    };
    let pace = ChartSeries {
        kind: ChartSeriesKind::SamePace,
        values: (0..past)
            .map(|_| mandi_reports::reports::Cell::Empty)
            .chain((0..e.ahead).map(|_| money_cell(e.per_month)))
            .collect(),
    };
    Ok(AnswerChart {
        result_id: String::new(),
        measure_id: CHANGE_MEASURE.to_owned(),
        months,
        series: vec![actual, pace],
        is_estimate: true,
    })
}

/// The card under the answer: the amber band's content.
fn estimate_card(
    parsed: &EstimateInput,
    changes: &[MonthValue],
    e: &BalanceEstimate,
) -> Result<AnswerEstimate, ToolError> {
    let method = EstimateMethod {
        kind: EstimateMethodKind::SamePace,
        months_used: i64::from(e.months_used),
        used_from: answer_month(e.used_from),
        used_to: answer_month(e.used_to),
        // The change is shown as a change with its direction word, not as a balance.
        per_month: None,
        step: Some(money_cell(magnitude(e.per_month))),
        step_dir: Some(figure_dir(Dir::of(e.per_month))),
        total: balance_cell(e.kind, e.total),
        target_month: None,
        factor: None,
        year_total: None,
        working: working_of(working_title(e.kind), &e.steps),
    };
    Ok(AnswerEstimate {
        result_id: String::new(),
        measure_id: parsed.measure.measure_id(),
        full_months: i64::try_from(changes.len())
            .map_err(|_| bad("ERR_INTERNAL", "history", "too many months"))?,
        first: answer_month(e.first),
        last: answer_month(e.last),
        target: None,
        methods: vec![method],
        year: None,
    })
}

/// The first and the last day of a run of months, `YYYY-MM-DD`: the 1st of the first month and the
/// last day of the last month. `None` for a month outside the calendar.
fn days_of(first: YearMonth, last: YearMonth) -> Option<(String, String)> {
    let from = BusinessDate::from_ymd(first.year(), first.month(), 1)?;
    let after = last.plus_months(1)?;
    let to = BusinessDate::from_ymd(after.year(), after.month(), 1)?.previous_day()?;
    Some((from.to_iso(), to.to_iso()))
}

/// The screen the buttons open: the party's ledger, the cash book from the first month of the
/// working to today (so the cash now, the starting point of the estimate, is its closing cash), or
/// the buyers' dues. `today` is `YYYY-MM-DD`.
fn source_of(
    kind: BalanceKind,
    party: Option<AnswerRecord>,
    subject: BalanceSubject,
    span: Option<(YearMonth, YearMonth)>,
    today: &str,
) -> AnswerSource {
    let report = |id: ReportId, filters: SourceReportFilters| AnswerSource::Report {
        result_id: String::new(),
        label_id: report_label(id),
        id,
        filters,
    };
    match (kind, subject, party) {
        (BalanceKind::Party, BalanceSubject::Party(_), Some(party)) => AnswerSource::PartyCard {
            result_id: String::new(),
            party,
        },
        (BalanceKind::Cash, ..) => {
            // The working's months and the cash now are both in it: the book runs to today, so its
            // closing cash is the figure the estimate starts from (hidden round 4, H381).
            let (from, to) = span
                .and_then(|(first, last)| days_of(first, last))
                .map(|(from, last_day)| (from, today.to_owned().max(last_day)))
                .unwrap_or_default();
            report(
                ReportId::R12,
                SourceReportFilters {
                    from,
                    to,
                    ..SourceReportFilters::default()
                },
            )
        }
        _ => report(ReportId::R20, SourceReportFilters::default()),
    }
}
