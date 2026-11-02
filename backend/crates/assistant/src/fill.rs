//! Filling a checked answer (PRD FR-7.74, FR-7.76, Code Standards Q4).
//!
//! The model wrote words and references. The app turns each reference into the value it points at
//! (a figure with its ✓, a name, a date, a month, an entry's label), so every digit on the screen
//! comes from a tool result. This file also builds the other things the panel shows: the options of
//! "کون سا؟", the tool results as plain tables when the second answer failed its checks, and the
//! differences of a figure whose two places disagree.
//!
//! Example: `{{r2.balance}}` on a بنام balance of 288,000 paisa that was checked becomes
//! `figure(balance cell 288000 dr, checked)`.

use mandi_reports::reports::{Cell, Names};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::auth::Language;
use mandi_services::domain::figures::Dir;

use crate::answer::{
    Answer, AnswerChart, AnswerEstimate, AnswerName, AnswerOption, AnswerPart, AnswerSource,
    AnswerTable, FigureDifference, FigureDirection, PlaceValue,
};
use crate::limits::MAX_TABLE_ROWS;
use crate::model_answer::{ModelAnswer, Piece, ShowAs, pieces};
use crate::results::{Money, NameVal, Node, Shown, Side, ToolResult, Val};
use crate::tools::{Env, State, lookup};

fn direction(dir: Dir) -> FigureDirection {
    match dir {
        Dir::Up => FigureDirection::Up,
        Dir::Down => FigureDirection::Down,
        Dir::Same => FigureDirection::Same,
    }
}

fn balance_side(side: Side) -> BalanceSide {
    match side {
        Side::Dr => BalanceSide::Dr,
        Side::Cr => BalanceSide::Cr,
        Side::Zero => BalanceSide::Zero,
    }
}

/// A money value as a report cell: a balance has its side in the cell.
fn money_cell(m: &Money) -> Cell {
    match m.side {
        Some(side) => Cell::Balance {
            paisa: m.paisa,
            side: balance_side(side),
        },
        None => Cell::Money { paisa: m.paisa },
    }
}

/// The text of a name in the app language, with its tag after it ("عاشق ظفر K").
pub fn name_text(language: Language, name: &NameVal) -> String {
    let base = match language {
        Language::Ur => &name.name_ur,
        Language::En => &name.name_en,
    };
    match &name.tag {
        Some(tag) => format!("{base} {tag}"),
        None => base.clone(),
    }
}

fn figure(
    cell: Cell,
    dir: Option<FigureDirection>,
    checked: bool,
    working: Option<crate::answer::AnswerWorking>,
) -> AnswerPart {
    AnswerPart::Figure {
        cell,
        dir,
        checked,
        working,
    }
}

/// The working of a figure: the calculator's, or the method of an estimate the path points into.
fn working_of(result: &ToolResult, path: &str) -> Option<crate::answer::AnswerWorking> {
    if result.tool == "calculate" {
        return result.working.clone();
    }
    if result.tool == "estimate"
        && let Some(Shown::Chart(_, Some(estimate))) = &result.shown
    {
        let key = path.split('.').nth(1)?;
        let kind = match key {
            "same_pace" => crate::answer::EstimateMethodKind::SamePace,
            "same_growth" => crate::answer::EstimateMethodKind::SameGrowth,
            "same_season" => crate::answer::EstimateMethodKind::SameSeason,
            _ => return None,
        };
        return estimate
            .methods
            .iter()
            .find(|m| m.kind == kind)
            .map(|m| m.working.clone());
    }
    None
}

/// One reference as a part of the answer.
fn part_of(env: &Env<'_>, state: &State, path: &str) -> Option<AnswerPart> {
    let found = lookup(env, state, path, "reference").ok()?;
    let Node::Val(val) = found.node() else {
        return None;
    };
    let first = path.split('.').next().unwrap_or("");
    let result = state.find(first);
    let working = result.and_then(|r| working_of(r, path));
    Some(part_of_val(val, working))
}

/// A value as a part of the answer (a figure with its ✓, a name, a date ...).
pub fn part_of_val(val: &Val, working: Option<crate::answer::AnswerWorking>) -> AnswerPart {
    match val {
        Val::Money(m) => figure(money_cell(m), None, m.checked == Some(true), working),
        Val::Diff { size, dir } => figure(
            Cell::Money { paisa: *size },
            Some(direction(*dir)),
            false,
            working,
        ),
        Val::Change { hundredths, dir } => figure(
            Cell::Percent {
                hundredths: *hundredths,
            },
            Some(direction(*dir)),
            false,
            working,
        ),
        Val::Pct(h) => figure(Cell::Percent { hundredths: *h }, None, false, working),
        Val::Count(n) => figure(Cell::Number { value: *n }, None, false, working),
        Val::CountDiff { size, dir } => figure(
            Cell::Number { value: *size },
            Some(direction(*dir)),
            false,
            working,
        ),
        Val::Question { number } => figure(Cell::Number { value: *number }, None, false, None),
        Val::Date(iso) => AnswerPart::Date { iso: iso.clone() },
        Val::Month { year, month } => AnswerPart::Month {
            year: i64::from(*year),
            month: i64::from(*month),
        },
        Val::Name(name) => AnswerPart::Name {
            name: name.answer_name(),
        },
        Val::Doc { type_id, number } => AnswerPart::Doc {
            type_id: type_id.clone(),
            number: *number,
        },
        // The guide's own words, in the answer as text (plan 51 B3).
        Val::Guide(text) => AnswerPart::Text { text: text.clone() },
    }
}

/// The next questions with their references as plain words (a name only).
fn next_text(env: &Env<'_>, state: &State, question: &str) -> String {
    let mut out = String::new();
    for piece in pieces(question) {
        match piece {
            Piece::Words(w) => out.push_str(&w),
            Piece::Ref(r) => {
                if let Ok(found) = lookup(env, state, &r.path, "reference")
                    && let Node::Val(Val::Name(name)) = found.node()
                {
                    out.push_str(&name_text(env.language, name));
                }
            }
        }
    }
    out
}

fn set_table_id(mut table: AnswerTable, id: &str, rows: Option<i64>) -> AnswerTable {
    table.result_id = id.to_owned();
    let cap = rows
        .and_then(|r| usize::try_from(r).ok())
        .unwrap_or(MAX_TABLE_ROWS)
        .min(MAX_TABLE_ROWS);
    table.rows.truncate(cap);
    table
}

fn set_chart_id(mut chart: AnswerChart, id: &str) -> AnswerChart {
    chart.result_id = id.to_owned();
    chart
}

fn set_source_id(mut source: AnswerSource, id: &str) -> AnswerSource {
    match &mut source {
        AnswerSource::PartyCard { result_id, .. }
        | AnswerSource::Report { result_id, .. }
        | AnswerSource::Analytics { result_id, .. }
        | AnswerSource::Tiles { result_id } => *result_id = id.to_owned(),
    }
    source
}

/// Every source read for this question, in the order of the results.
fn sources_of(state: &State) -> Vec<AnswerSource> {
    state
        .results
        .iter()
        .filter(|r| r.turn == state.turn)
        .filter_map(|r| r.source.clone().map(|s| set_source_id(s, &r.id)))
        .collect()
}

fn estimate_of(result: &ToolResult) -> Option<AnswerEstimate> {
    match &result.shown {
        Some(Shown::Chart(_, Some(estimate))) => {
            let mut e = (**estimate).clone();
            e.result_id = result.id.clone();
            Some(e)
        }
        _ => None,
    }
}

/// Fills a checked answer.
pub fn fill(env: &Env<'_>, state: &State, answer: &ModelAnswer, read_at: &str) -> Answer {
    let mut parts = Vec::new();
    let mut estimate: Option<AnswerEstimate> = None;
    if answer.kind.has_text() {
        for piece in pieces(&answer.text) {
            match piece {
                Piece::Words(text) => parts.push(AnswerPart::Text { text }),
                Piece::Ref(r) => {
                    if let Some(part) = part_of(env, state, &r.path) {
                        parts.push(part);
                    }
                    // A reference into an estimate shows its band.
                    let first = r.path.split('.').next().unwrap_or("");
                    if let Some(result) = state.find(first)
                        && result.tool == "estimate"
                        && estimate.is_none()
                    {
                        estimate = estimate_of(result);
                    }
                }
            }
        }
    }
    let mut tables = Vec::new();
    let mut charts = Vec::new();
    for show in &answer.show {
        let Some(result) = state.find(&show.result) else {
            continue;
        };
        match (&result.shown, show.as_) {
            (Some(Shown::Table(t) | Shown::TableChart(t, _)), ShowAs::Table) => {
                tables.push(set_table_id(t.clone(), &result.id, show.rows));
            }
            (Some(Shown::Chart(c, _) | Shown::TableChart(_, c)), ShowAs::Chart) => {
                charts.push(set_chart_id((**c).clone(), &result.id));
                if estimate.is_none() {
                    estimate = estimate_of(result);
                }
            }
            _ => {}
        }
    }
    Answer {
        parts,
        tables,
        charts,
        estimate,
        advice: answer.kind == crate::model_answer::AnswerKind::Advice,
        sources: sources_of(state),
        differences: Vec::new(),
        next: answer
            .next
            .iter()
            .map(|q| next_text(env, state, q))
            .collect(),
        read_at: read_at.to_owned(),
    }
}

/// The tool results of this question as plain tables with their sources: what the panel shows
/// when the second answer also failed its checks (PRD FR-7.76).
pub fn plain_tables(state: &State, read_at: &str) -> Answer {
    let tables = state
        .results
        .iter()
        .filter(|r| r.turn == state.turn)
        .filter_map(|r| match &r.shown {
            Some(Shown::Table(t) | Shown::TableChart(t, _)) => {
                Some(set_table_id(t.clone(), &r.id, None))
            }
            _ => None,
        })
        .collect();
    Answer {
        parts: Vec::new(),
        tables,
        charts: Vec::new(),
        estimate: None,
        advice: false,
        sources: sources_of(state),
        differences: Vec::new(),
        next: Vec::new(),
        read_at: read_at.to_owned(),
    }
}

/// The Fluent message of a place of a figure (the frontend's `assistant-place-*`).
fn place_id(place: &str, path: &str) -> String {
    match place {
        "party_card" => "assistant-place-party-card",
        "R13" => "assistant-place-r13-closing",
        "R12" => "assistant-place-r12-closing",
        "R20" => "assistant-place-r20-total",
        "R24" => "assistant-place-r24-total",
        "R38" => "assistant-place-r38-total",
        "view_A" => "assistant-place-view-a-total",
        "partner_overview" => "assistant-place-partner-overview",
        "R31" => "assistant-place-r31-closing",
        _ if path.contains("cash_in_hand") => "assistant-place-tile-cash",
        _ if path.contains("buyers_dues") => "assistant-place-tile-buyers-dues",
        _ if path.contains("sellers_dues") => "assistant-place-tile-sellers-dues",
        _ if path.contains("dead_amount") => "assistant-place-tile-dead-amount",
        _ if path.contains("total_dues") => "assistant-place-tile-total-dues",
        _ => "assistant-place-tile-day",
    }
    .to_owned()
}

fn figure_id(path: &str, m: &Money) -> String {
    if m.places.iter().any(|(k, _)| *k == "party_card") {
        "assistant-figure-party-balance"
    } else if m.places.iter().any(|(k, _)| *k == "partner_overview") {
        "assistant-figure-partner-total"
    } else if path.contains("cash_in_hand") {
        "assistant-figure-cash"
    } else if path.contains("buyers_dues") {
        "assistant-figure-buyers-dues"
    } else if path.contains("sellers_dues") {
        "assistant-figure-sellers-dues"
    } else if path.contains("dead_amount") {
        "assistant-figure-dead-amount"
    } else if path.contains("total_dues") {
        "assistant-figure-total-dues"
    } else if path.contains("today_commission") {
        "assistant-figure-day-commission"
    } else {
        "assistant-figure-day-sales"
    }
    .to_owned()
}

/// The name beside a figure: the `name` of the same object, or the `party` of its result.
fn name_beside(state: &State, path: &str) -> Option<AnswerName> {
    let parts: Vec<&str> = path.split('.').collect();
    let (_, parent) = parts.split_last()?;
    let root = state.find(parts.first()?)?;
    let sibling = root
        .node
        .get(parent.get(1..)?)
        .and_then(|n| n.get(&["name"]));
    let name = sibling.or_else(|| root.node.get(&["party", "name"]))?;
    match name {
        Node::Val(Val::Name(n)) => Some(n.answer_name()),
        _ => None,
    }
}

/// The figures cited whose two places differ, for the answer with outcome `records_differ`.
pub fn differences(env: &Env<'_>, state: &State, paths: &[String]) -> Vec<FigureDifference> {
    let mut out = Vec::new();
    for path in paths {
        let Ok(found) = lookup(env, state, path, "reference") else {
            continue;
        };
        let Node::Val(Val::Money(m)) = found.node() else {
            continue;
        };
        out.push(FigureDifference {
            figure_id: figure_id(path, m),
            name: name_beside(state, path),
            places: m
                .places
                .iter()
                .map(|(place, paisa)| PlaceValue {
                    place_id: place_id(place, path),
                    cell: Cell::Money { paisa: *paisa },
                })
                .collect(),
        });
    }
    out
}

/// The answer that carries only the differences and the sources read.
pub fn differ_answer(env: &Env<'_>, state: &State, paths: &[String], read_at: &str) -> Answer {
    Answer {
        parts: Vec::new(),
        tables: Vec::new(),
        charts: Vec::new(),
        estimate: None,
        advice: false,
        sources: sources_of(state),
        differences: differences(env, state, paths),
        next: Vec::new(),
        read_at: read_at.to_owned(),
    }
}

/// The matches of a `find` result as the buttons of "کون سا؟".
pub fn options_of(state: &State, find_id: &str) -> Vec<AnswerOption> {
    let Some(result) = state.find(find_id) else {
        return Vec::new();
    };
    let Some(Node::List(matches)) = result.node.get(&["matches"]) else {
        return Vec::new();
    };
    matches
        .iter()
        .filter_map(|m| {
            let key = match m.get(&["key"])? {
                Node::Str(s) => s.clone(),
                _ => return None,
            };
            let name = match m.get(&["name"])? {
                Node::Val(Val::Name(n)) => n.answer_name(),
                _ => return None,
            };
            let city = match m.get(&["city"]) {
                Some(Node::Val(Val::Name(n)))
                    if !(n.name_ur.is_empty() && n.name_en.is_empty()) =>
                {
                    Some(Names {
                        ur: n.name_ur.clone(),
                        en: n.name_en.clone(),
                    })
                }
                _ => None,
            };
            let (balance, checked) = match m.get(&["balance"]) {
                Some(Node::Val(Val::Money(b))) => (Some(money_cell(b)), b.checked == Some(true)),
                _ => (None, false),
            };
            let dead = matches!(m.get(&["dead"]), Some(Node::Bool(true)));
            Some(AnswerOption {
                key,
                name,
                city,
                balance,
                checked,
                dead,
            })
        })
        .collect()
}
