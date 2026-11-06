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

use mandi_reports::reports::{Cell, Names, ReportId};
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
use crate::source::AnswerRecord;
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

/// The working of a figure: the calculator's, or the method of an estimate the path points into
/// (a total, a per-month step, or the figure of one month ahead: every estimated figure carries the
/// working of its method, so it never reads like a figure of the books).
fn working_of(result: &ToolResult, path: &str) -> Option<crate::answer::AnswerWorking> {
    if result.tool == "calculate" {
        return result.working.clone();
    }
    if result.tool == "estimate"
        && let Some(Shown::Chart(_, Some(estimate))) = &result.shown
    {
        // The method is named by the first segment that is one: `e1.same_pace.total`, and also the
        // figure of one month ahead, `e1.ahead.ah1.same_pace`, which belongs to the same method
        // (live run 2, Q708: that figure was shown with no working, like a figure of the books).
        let kind = path.split('.').skip(1).find_map(|segment| match segment {
            "same_pace" => Some(crate::answer::EstimateMethodKind::SamePace),
            "same_growth" => Some(crate::answer::EstimateMethodKind::SameGrowth),
            "same_season" => Some(crate::answer::EstimateMethodKind::SameSeason),
            "what_if_pace" => Some(crate::answer::EstimateMethodKind::WhatIfPace),
            _ => None,
        })?;
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
        | AnswerSource::Screen { result_id, .. }
        | AnswerSource::Tiles { result_id } => *result_id = id.to_owned(),
    }
    source
}

/// Every source read for this question, in the order of the results. R31 read for everyone (the
/// partner overview) is left out: the report screen needs a partner, so a button without one is
/// refused (live run 1, Q052 and Q400). [`sources_cited`] gives the buttons of the partners an
/// answer names.
fn sources_of(state: &State) -> Vec<AnswerSource> {
    state
        .results
        .iter()
        .filter(|r| r.turn == state.turn)
        .filter_map(|r| r.source.clone().map(|s| set_source_id(s, &r.id)))
        .filter(|s| !is_partner_overview(s))
        .collect()
}

/// R31 with no partner: the overview of every partner, which the report screen cannot open.
fn is_partner_overview(source: &AnswerSource) -> bool {
    matches!(
        source,
        AnswerSource::Report { id: ReportId::R31, filters, .. } if filters.partner.is_none()
    )
}

/// The rows an answer cites inside a result: `r1.rows.s2.total` is (`r1`, `rows`, `s2`) and
/// `r3.by_for.guests.value` is (`r3`, `by_for`, `guests`). Each one once, in the order cited.
fn cited_rows(text: &str) -> Vec<(String, String, String)> {
    let mut out: Vec<(String, String, String)> = Vec::new();
    for piece in pieces(text) {
        if let Piece::Ref(r) = piece {
            let parts: Vec<&str> = r.path.split('.').collect();
            if let [result, area @ ("rows" | "by_for"), key, ..] = parts.as_slice() {
                let row = ((*result).to_owned(), (*area).to_owned(), (*key).to_owned());
                if !out.contains(&row) {
                    out.push(row);
                }
            }
        }
    }
    out
}

/// The partner of a row named `s<id>` in the keyed rows `area` of a result, as the record a picker
/// holds. `None` when the row has no such key or no name.
fn partner_of_row(result: &ToolResult, area: &str, key: &str) -> Option<AnswerRecord> {
    let id = key.strip_prefix('s')?.parse::<i64>().ok()?;
    let Some(Node::Val(Val::Name(name))) = result.node.get(&[area, key, "name"]) else {
        return None;
    };
    Some(AnswerRecord {
        id,
        code: name.code.clone(),
        name_ur: name.name_ur.clone(),
        name_en: name.name_en.clone(),
        tag: name.tag.clone(),
    })
}

/// The buttons of an answer: every source read, with the report made to open what the answer
/// CITES (live run 1: Q052 and Q400 opened R31 with no partner and were refused; Q216, Q217,
/// Q395 and Q218 opened the whole fruit register for a figure that is one partner's, or the
/// guests'):
/// - R31 read for everyone has one button for each partner whose row is cited, with that partner;
/// - R37 read for everyone, when the answer cites its `by_for` rows, has one button for each
///   partner cited and one with `guests_only` for the guests.
///
/// Anything else is as read.
///
/// Example: "ب کا سرمایہ {{r1.rows.s2.capital.invested}}" gives R31 with ب.
fn sources_cited(state: &State, text: &str) -> Vec<AnswerSource> {
    let cited = cited_rows(text);
    let mut out = Vec::new();
    for result in state.results.iter().filter(|r| r.turn == state.turn) {
        let Some(source) = result.source.clone().map(|s| set_source_id(s, &result.id)) else {
            continue;
        };
        let AnswerSource::Report {
            result_id,
            label_id,
            id,
            filters,
        } = &source
        else {
            out.push(source);
            continue;
        };
        let (area, whole_register) = match (id, filters.partner.is_none(), filters.guests_only) {
            (ReportId::R31, true, _) => ("rows", false),
            (ReportId::R37, true, false) => ("by_for", true),
            _ => {
                out.push(source);
                continue;
            }
        };
        let mut buttons = Vec::new();
        for (cited_in, cited_area, key) in &cited {
            if cited_in != &result.id || cited_area != area {
                continue;
            }
            let mut opened = filters.clone();
            if key == "guests" && area == "by_for" {
                opened.guests_only = true;
            } else if let Some(partner) = partner_of_row(result, area, key) {
                opened.partner = Some(partner);
            } else {
                continue;
            }
            buttons.push(AnswerSource::Report {
                result_id: result_id.clone(),
                label_id: label_id.clone(),
                id: *id,
                filters: opened,
            });
        }
        if buttons.is_empty() && whole_register {
            // Nothing of the fruit register was cited by partner: the whole register, as read.
            out.push(source);
        } else {
            out.extend(buttons);
        }
    }
    out
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

/// The word of a direction, as the panel prints it after a difference or a change.
fn direction_word(dir: FigureDirection, language: Language) -> &'static str {
    match (dir, language) {
        (FigureDirection::Up, Language::Ur) => "زیادہ",
        (FigureDirection::Down, Language::Ur) => "کم",
        (FigureDirection::Same, Language::Ur) => "برابر",
        (FigureDirection::Up, Language::En) => "more",
        (FigureDirection::Down, Language::En) => "less",
        (FigureDirection::Same, Language::En) => "the same",
    }
}

/// The panel prints a direction word after a difference ("3,000 زیادہ"). If the model wrote the
/// same word right after it ("3,000 زیادہ زیادہ"), that one is dropped (Q081 of the live run of
/// 02-11-2026). Only the exact word of the direction the app just printed, at the start of the
/// words that follow, is dropped; anything else stays.
///
/// Example: `without_repeated_direction(" زیادہ ہے۔", Up, Ur)` is `" ہے۔"`.
pub fn without_repeated_direction(text: &str, dir: FigureDirection, language: Language) -> String {
    let word = direction_word(dir, language);
    let Some(rest) = text.trim_start().strip_prefix(word) else {
        return text.to_owned();
    };
    // The word must end there: a longer word that starts with it is another word.
    if rest.chars().next().is_some_and(char::is_alphanumeric) {
        return text.to_owned();
    }
    rest.to_owned()
}

/// Fills a checked answer.
pub fn fill(env: &Env<'_>, state: &State, answer: &ModelAnswer, read_at: &str) -> Answer {
    let mut parts = Vec::new();
    let mut estimate: Option<AnswerEstimate> = None;
    if answer.kind.has_text() {
        // The direction a difference or a change just printed, for the word after it.
        let mut printed: Option<FigureDirection> = None;
        for piece in pieces(&answer.text) {
            match piece {
                Piece::Words(text) => {
                    let text = match printed.take() {
                        Some(dir) => without_repeated_direction(&text, dir, env.language),
                        None => text,
                    };
                    parts.push(AnswerPart::Text { text });
                }
                Piece::Ref(r) => {
                    printed = None;
                    if let Some(part) = part_of(env, state, &r.path) {
                        if let AnswerPart::Figure { dir, .. } = &part {
                            printed = *dir;
                        }
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
        advice: matches!(
            answer.kind,
            crate::model_answer::AnswerKind::Advice
                | crate::model_answer::AnswerKind::GeneralAdvice
        ),
        general_advice: answer.kind == crate::model_answer::AnswerKind::GeneralAdvice,
        sources: sources_cited(state, &answer.text),
        how: Vec::new(),
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
        general_advice: false,
        sources: sources_of(state),
        how: Vec::new(),
        differences: Vec::new(),
        next: Vec::new(),
        read_at: read_at.to_owned(),
    }
}

/// The field of a party card that holds the figure a question asked for (money review R-b): the
/// balance for a balance or dues question, the open advance for a peshgi question, the payable
/// for an adaigi question.
pub const CARD_BALANCE: &str = "balance";
/// The card field of an advance question.
pub const CARD_ADVANCE: &str = "open_advance";
/// The card field of a payable question.
pub const CARD_PAYABLE: &str = "payable";

/// What was read for a question that ran out of time (plan 55 Section 3): the plain tables as in
/// [`plain_tables`], and for each party card read, his name and the figure of the card that the
/// Owner asked for (`card_field`); a read of the tiles shows as its source. `None` when nothing
/// that shows was read: a `find` alone has no figure and is not an answer.
///
/// When the app does not know which figure of the card was asked (`card_field` is `None`), the
/// card shows no figure: a balance given for an advance question would be a real number for the
/// wrong thing.
///
/// Example: "ikram ke total bakaya kitne hai", the card read and then the line drops: the answer
/// holds `name اکرم ٹول ٹیکس`, `figure 2,880 بنام ✓`, and the card and R13 as sources.
/// "rahim ki peshgi kitni hai" shows his open advance, 15,000, and not his balance.
pub fn read_so_far(state: &State, read_at: &str, card_field: Option<&str>) -> Option<Answer> {
    let mut answer = plain_tables(state, read_at);
    for result in state
        .results
        .iter()
        .filter(|r| r.turn == state.turn && r.tool == "party_card")
    {
        let Some(field) = card_field else { break };
        let (Some(Node::Val(name)), Some(Node::Val(figure))) = (
            result.node.get(&["party", "name"]),
            result.node.get(&[field]),
        ) else {
            continue;
        };
        if !answer.parts.is_empty() {
            answer.parts.push(AnswerPart::Text {
                text: "\n".to_owned(),
            });
        }
        answer.parts.push(part_of_val(name, None));
        answer.parts.push(part_of_val(figure, None));
    }
    let shows = !answer.tables.is_empty()
        || !answer.parts.is_empty()
        || answer
            .sources
            .iter()
            .any(|s| matches!(s, AnswerSource::Tiles { .. }));
    shows.then_some(answer)
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
        general_advice: false,
        sources: sources_of(state),
        how: Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceReportFilters;

    #[test]
    fn a_direction_word_the_panel_already_prints_is_dropped_once() {
        use FigureDirection::{Down, Same, Up};
        assert_eq!(
            without_repeated_direction(" زیادہ ہے۔", Up, Language::Ur),
            " ہے۔"
        );
        assert_eq!(without_repeated_direction("زیادہ۔", Up, Language::Ur), "۔");
        assert_eq!(
            without_repeated_direction(" کم رہا", Down, Language::Ur),
            " رہا"
        );
        assert_eq!(
            without_repeated_direction(" برابر ہے", Same, Language::Ur),
            " ہے"
        );
        assert_eq!(
            without_repeated_direction(" more than last month", Up, Language::En),
            " than last month"
        );
        assert_eq!(
            without_repeated_direction(" the same", Same, Language::En),
            ""
        );
        // Another direction, another word, or a longer word: left as it is.
        assert_eq!(
            without_repeated_direction(" کم ہے", Up, Language::Ur),
            " کم ہے"
        );
        assert_eq!(
            without_repeated_direction(" moreover", Up, Language::En),
            " moreover"
        );
        assert_eq!(without_repeated_direction(" ہے۔", Up, Language::Ur), " ہے۔");
        // The English word does not count in an Urdu answer.
        assert_eq!(
            without_repeated_direction(" more", Up, Language::Ur),
            " more"
        );
    }

    // ---- the buttons follow what the answer cites (live run 1: Q052, Q400, Q216, Q217, Q395, Q218)

    fn name(en: &str, ur: &str) -> Node {
        Node::Val(Val::Name(NameVal {
            name_ur: ur.to_owned(),
            name_en: en.to_owned(),
            tag: None,
            code: None,
        }))
    }

    /// A report result with the given node, read for everyone (no partner).
    fn everyone(id: ReportId, node: Node) -> State {
        let result = ToolResult {
            id: "r2".to_owned(),
            tool: "report",
            input: serde_json::json!({}),
            node,
            shown: None,
            source: Some(AnswerSource::Report {
                result_id: String::new(),
                label_id: "report-title".to_owned(),
                id,
                filters: SourceReportFilters::default(),
            }),
            partner_parts_locked: false,
            partner_data: true,
            turn: 0,
            working: None,
        };
        State {
            results: vec![result],
            turn: 0,
        }
    }

    fn report_filters(source: &AnswerSource) -> &SourceReportFilters {
        match source {
            AnswerSource::Report { filters, .. } => filters,
            other => panic!("not a report: {other:?}"),
        }
    }

    fn overview_node() -> Node {
        Node::obj(vec![(
            "rows",
            Node::Keyed(vec![
                (
                    "s1".to_owned(),
                    Node::obj(vec![("name", name("Partner A", "الف"))]),
                ),
                (
                    "s2".to_owned(),
                    Node::obj(vec![("name", name("Partner B", "ب"))]),
                ),
                (
                    "s3".to_owned(),
                    Node::obj(vec![("name", name("Partner C", "ج"))]),
                ),
            ]),
        )])
    }

    fn fruit_node() -> Node {
        Node::obj(vec![(
            "by_for",
            Node::Keyed(vec![
                (
                    "s1".to_owned(),
                    Node::obj(vec![("name", name("Partner A", "الف"))]),
                ),
                ("guests".to_owned(), Node::obj(vec![])),
            ]),
        )])
    }

    #[test]
    fn the_partner_overview_has_one_button_for_each_partner_cited_and_none_when_none_is() {
        let state = everyone(ReportId::R31, overview_node());
        // Everyone cited: three buttons, each with its partner (Q052).
        let all = sources_cited(
            &state,
            "{{r2.rows.s1.total}} {{r2.rows.s2.total}} {{r2.rows.s3.total}}",
        );
        let names: Vec<&str> = all
            .iter()
            .map(|s| {
                report_filters(s)
                    .partner
                    .as_ref()
                    .map_or("none", |p| p.name_en.as_str())
            })
            .collect();
        assert_eq!(names, ["Partner A", "Partner B", "Partner C"]);
        // One cited: one button with him (Q400).
        let one = sources_cited(&state, "{{r2.rows.s2.capital.invested}}");
        assert_eq!(one.len(), 1);
        assert_eq!(
            report_filters(&one[0]).partner.as_ref().map(|p| p.id),
            Some(2)
        );
        // The same row twice is one button.
        assert_eq!(
            sources_cited(
                &state,
                "{{r2.rows.s2.total}} {{r2.rows.s2.capital.balance}}"
            )
            .len(),
            1
        );
        // Nothing cited: no button, because R31 cannot open without a partner. The plain tables
        // and the slow-line answer drop it too.
        assert!(sources_cited(&state, "no figures").is_empty());
        assert!(sources_of(&state).is_empty());
    }

    #[test]
    fn the_fruit_register_opens_for_the_partner_or_the_guests_whose_row_is_cited() {
        let state = everyone(ReportId::R37, fruit_node());
        // Alif's home fruit (Q216, Q217, Q395): R37 with Partner A.
        let home = sources_cited(&state, "{{r2.by_for.s1.home_value}}");
        assert_eq!(home.len(), 1);
        let filters = report_filters(&home[0]);
        assert_eq!(
            filters.partner.as_ref().map(|p| p.name_en.as_str()),
            Some("Partner A")
        );
        assert!(!filters.guests_only);
        // The guests (Q218): R37 with guests_only and no partner.
        let guests = sources_cited(&state, "{{r2.by_for.guests.value}}");
        assert_eq!(guests.len(), 1);
        let filters = report_filters(&guests[0]);
        assert!(filters.guests_only && filters.partner.is_none());
        // The register as a whole (the total row cited, no by_for row): as read, for everyone.
        let whole = sources_cited(&state, "{{r2.totals.value}}");
        assert_eq!(whole.len(), 1);
        let filters = report_filters(&whole[0]);
        assert!(filters.partner.is_none() && !filters.guests_only);
    }
}
