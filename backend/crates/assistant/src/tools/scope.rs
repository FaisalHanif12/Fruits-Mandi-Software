//! Whose figure an input of `calculate` is (plan 52 P52-S1, the robustness finding of 03-11-2026:
//! "kela ka commission" gave 9,376, the commission of banana added to the commission of every
//! item, and "kela aur saib ka commission" gave 2,724, the average of the two; the money review
//! of the guard added the wildcard, heading, `matched` and report-filter paths).
//!
//! Every figure an input names has a scope: the single thing it was read for. A scope is a set of
//! tokens, sorted and joined with `|`:
//!
//! | The token | Where it comes from |
//! |---|---|
//! | `v:I`, `r:R36` | gardens, trips and fruit taken: a part of everything, not its total |
//! | `i1`, `a18`, `ct2`, `ad3`, `s1`, `t4`, `g5` | a filter of the read: item, seller or party, city, adda, partner, trip, garden |
//! | `group:fruit`, `group:vegetable` | all the items of a group (PRD v1.29): a part of everything, and the whole of each of its items |
//! | `goods:trip`, `side:seller`, `bikri:7`, `voucher:receipt`, `status:paid`, `guests` | the other filters that cut the figures down |
//! | `i2` | the row of a result (`rows.i2`): an unfiltered view's row is the same as a read filtered to item 2 |
//! | `headings:h3`, `series:m202609`, `matched`, `peak`, `compare:earlier`, `by_for:s1` | a part of one result that is not its whole figure |
//! | `*` | everything: a dashboard tile, or a result read with no filter |
//!
//! A `find` match is its own key (`a18`), and so is a party card. A `calculate` result carries the
//! scopes of its inputs, joined with `+`. A number of the question has no scope. A figure whose
//! origin is not known is `?` and a number, and is never the same as any other scope.
//!
//! One account has one key whether it was read as a seller, a buyer or a party. Figures with two or
//! more different scopes are combined only when the question asks for that (see `run_calculate`).
//! A part and its whole (the total of all items and one item, a seller side and one item of it) are
//! never added up or averaged.
//!
//! Example: `r1.totals.total_comm` of view A read for banana is `i1`; the row `rows.i1` of the
//! unfiltered view A is the same; of the unfiltered view, `totals` is `*`; `rows.*` is one scope
//! for each row.

use mandi_reports::analytics::{AnalyticsGoods, AnalyticsSide, AnalyticsView};
use mandi_reports::reports::ReportId;

use crate::answer::AnswerSource;
use crate::results::{Node, ToolResult};
use crate::source::{SourceAnalyticsFilters, SourceParty, SourceReportFilters};
use crate::tools::State;
use crate::tools::cells::is_total_line;

/// The scope of everything: a dashboard tile, or a result read with no filter.
pub const TILE: &str = "*";
/// The start of the scope of a figure whose origin is not known.
pub const UNKNOWN: &str = "?";
/// The start of the token of an item group.
const GROUP_PREFIX: &str = "group:";
/// How deep a chain of calculations is followed.
const MAX_DEPTH: u8 = 4;
/// The parts of one result that are not its whole figure, and so are scopes of their own.
const PARTS: [&str; 7] = [
    "headings", "matched", "series", "peak", "lowest", "compare", "by_for",
];
/// Parts whose next name always makes a different scope (`compare.earlier`, `compare.difference`).
const PARTS_BY_NAME: [&str; 1] = ["compare"];
/// The token of the earlier period of the same figure. It is a scope of its own, but not a part of
/// the whole. (`compare:difference` and `compare:change` are figures worked out from both
/// periods, so they are parts.)
const PERIOD_TOKEN: &str = "compare:earlier";
/// The names of the figures a row or the comparison holds besides the figure itself.
const COMPARE_NAMES: [&str; 3] = ["earlier", "difference", "change"];

/// The time a figure was read for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Window {
    /// Not known (a tile, a find, a calculation, a shortcut): never taken as apart from another.
    Unknown,
    /// From and to, `YYYY-MM-DD`; an empty end is open.
    Span(String, String),
    /// A balance on one day (`as_of`): balances of two days are never added up.
    Point,
}

impl Window {
    /// True when the two are known to share no day.
    fn apart_from(&self, other: &Window) -> bool {
        match (self, other) {
            (Window::Span(_, a_to), Window::Span(b_from, _))
                if !a_to.is_empty() && !b_from.is_empty() && a_to < b_from =>
            {
                true
            }
            (Window::Span(a_from, _), Window::Span(_, b_to))
                if !a_from.is_empty() && !b_to.is_empty() && b_to < a_from =>
            {
                true
            }
            _ => false,
        }
    }
}

/// One figure an input of `calculate` reads, with whose it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    /// The scope (see the module doc).
    pub scope: String,
    /// The result the figure comes from (`r1`).
    pub root: String,
    /// What is measured: the last name of its path (`total_comm`), and for a tile its name too
    /// (`total_dues.amount`).
    pub measure: String,
    /// The time it was read for.
    pub window: Window,
    /// True when it is one of the figures a wildcard (`rows.*`, `matches.*`) names.
    pub wild: bool,
    /// True when it is the result of a calculation (its scope holds the scopes of its inputs).
    pub calc: bool,
}

/// How the figures of one calculation relate.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Mix {
    /// How many different scopes there are.
    pub distinct: usize,
    /// The same figure of the same scope is read from two different results (two periods).
    pub repeated: bool,
    /// Those two readings share a day, or their days are not known.
    pub overlapping: bool,
    /// One scope is a part of another (the total of all items, and one item), or one measure is a
    /// part of another of the same result (`laga` in `total_comm`).
    pub whole_and_part: bool,
    /// The same figure is named twice, or a calculation and an input both hold the same scope, so
    /// it would be counted twice.
    pub twice: bool,
    /// How many figures came from a wildcard.
    pub wild: usize,
}

/// Measures that are made of other measures of the same result: adding one to a part of itself
/// counts the part twice.
const MEASURE_PARTS: [(&str, &[&str]); 5] = [
    ("total_comm", &["seller_comm", "buyer_comm", "laga"]),
    ("buyer_total", &["value", "buyer_comm", "laga"]),
    (
        "deductions",
        &[
            "commission",
            "freight",
            "labour",
            "munshiana",
            "cash_paid",
            "store_rent",
        ],
    ),
    (
        "total_dues.amount",
        &[
            "buyers_dues.amount",
            "sellers_dues.amount",
            "dead_amount.amount",
        ],
    ),
    ("money_lent_open.amount", &["money_lent_open.overdue"]),
];

fn measure_inside(part: &str, whole: &str) -> bool {
    MEASURE_PARTS
        .iter()
        .any(|(w, parts)| *w == whole && parts.contains(&part))
}

/// The token of an item group filter.
fn group_token(group: mandi_services::core::rules::ItemGroup) -> String {
    format!("{GROUP_PREFIX}{}", group.as_str())
}

/// True for the token of an item group.
fn is_group(token: &str) -> bool {
    token.starts_with(GROUP_PREFIX)
}

/// True for the token of one item (`i1`, `i24`; not `ad3` or `ct2`).
fn is_item(token: &str) -> bool {
    token
        .strip_prefix('i')
        .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
}

/// True when one scope reads one item and the other reads a group, and the rest of their tokens
/// are the same. The scope does not know which group the item is in, so it takes the item as
/// possibly one of the group: adding them could count the item twice.
fn item_in_group(item_side: &[&str], group_side: &[&str]) -> bool {
    let mut items = item_side.iter().filter(|t| is_item(t));
    let mut groups = group_side.iter().filter(|t| is_group(t));
    let (Some(_), None) = (items.next(), items.next()) else {
        return false;
    };
    let (Some(_), None) = (groups.next(), groups.next()) else {
        return false;
    };
    fn rest<'a>(side: &[&'a str], skip: fn(&str) -> bool) -> Vec<&'a str> {
        let mut kept: Vec<&'a str> = side.iter().copied().filter(|t| !skip(t)).collect();
        kept.sort_unstable();
        kept
    }
    !group_side.iter().any(|t| is_item(t))
        && !item_side.iter().any(|t| is_group(t))
        && rest(item_side, is_item) == rest(group_side, is_group)
}

fn account(party: &SourceParty) -> String {
    match &party.person_key {
        Some(person) => format!("a{}~{person}", party.record.id),
        None => format!("a{}", party.record.id),
    }
}

/// The filters of an analytics view as tokens.
fn analytics_tokens(f: &SourceAnalyticsFilters) -> Vec<String> {
    let mut keys = Vec::new();
    let mut add = |prefix: &str, id: Option<i64>| {
        if let Some(id) = id {
            keys.push(format!("{prefix}{id}"));
        }
    };
    add("i", f.item.as_ref().map(|r| r.id));
    add("a", f.seller.as_ref().map(|r| r.id));
    add("ct", f.city.as_ref().map(|r| r.id));
    add("ad", f.adda.as_ref().map(|r| r.id));
    add("s", f.partner.as_ref().map(|r| r.id));
    add("t", f.trip.as_ref().map(|r| r.id));
    add("g", f.garden.as_ref().map(|r| r.id));
    if let Some(group) = f.item_group {
        keys.push(group_token(group));
    }
    if let Some(buyer) = &f.buyer {
        keys.push(account(buyer));
    }
    if f.goods != AnalyticsGoods::default() {
        keys.push(format!("goods:{:?}", f.goods).to_lowercase());
    }
    if f.side != AnalyticsSide::default() {
        keys.push(format!("side:{:?}", f.side).to_lowercase());
    }
    keys
}

/// The filters of a report as tokens.
fn report_tokens(f: &SourceReportFilters) -> Vec<String> {
    let mut keys = Vec::new();
    let mut add = |prefix: &str, id: Option<i64>| {
        if let Some(id) = id {
            keys.push(format!("{prefix}{id}"));
        }
    };
    add("a", f.seller.as_ref().map(|r| r.id));
    add("a", f.zamindar.as_ref().map(|r| r.id));
    add("i", f.item.as_ref().map(|r| r.id));
    add("ct", f.city.as_ref().map(|r| r.id));
    add("ad", f.adda.as_ref().map(|r| r.id));
    add("s", f.partner.as_ref().map(|r| r.id));
    add("g", f.garden.as_ref().map(|r| r.id));
    if let Some(group) = f.item_group {
        keys.push(group_token(group));
    }
    if let Some(party) = &f.party {
        keys.push(account(party));
    }
    if let Some(bikri) = &f.bikri {
        keys.push(format!("bikri:{}", bikri.id));
    }
    if let Some(kind) = &f.voucher_type {
        keys.push(format!("voucher:{kind:?}").to_lowercase());
    }
    if let Some(status) = &f.status {
        keys.push(format!("status:{status:?}").to_lowercase());
    }
    if f.guests_only {
        keys.push("guests".to_owned());
    }
    keys
}

/// A scope of a figure whose origin is not known, different from every other.
fn fresh_unknown(counter: &mut u32) -> String {
    *counter = counter.saturating_add(1);
    format!("{UNKNOWN}{counter}")
}

/// What a result was read for, before the part of a path is looked at. `None` when it is not known.
fn base_tokens(result: &ToolResult) -> Option<Vec<String>> {
    if result.tool == "tiles" {
        return Some(vec![TILE.to_owned()]);
    }
    match result.source.as_ref()? {
        AnswerSource::Analytics { view, filters, .. } => {
            let mut keys = analytics_tokens(filters);
            // Gardens, trips and fruit taken are a part of everything: their totals are not the
            // total of all.
            if matches!(view, AnalyticsView::I | AnalyticsView::J | AnalyticsView::K) {
                keys.push(format!("v:{}", view.code()));
            }
            Some(keys)
        }
        AnswerSource::Report { id, filters, .. } => {
            let mut keys = report_tokens(filters);
            if matches!(id, ReportId::R33 | ReportId::R36 | ReportId::R37) {
                keys.push(format!("r:{}", id.code()));
            }
            Some(keys)
        }
        AnswerSource::PartyCard { party, .. } => Some(vec![format!("a{}", party.id)]),
        AnswerSource::Tiles { .. } => Some(vec![TILE.to_owned()]),
        // A screen holds no figure of its own.
        AnswerSource::Screen { .. } => None,
    }
}

/// The token of one element of a container: a row is its key; any other part is `part:key`.
fn element_token(container: &str, key: &str) -> String {
    if container == "rows" {
        key.to_owned()
    } else {
        format!("{container}:{key}")
    }
}

/// The tokens of the other period a path names after a row: `earlier`, `difference`, `change`.
fn compare_tokens(after: &[&str]) -> Vec<String> {
    after
        .iter()
        .filter(|name| COMPARE_NAMES.contains(name))
        .map(|name| element_token("compare", name))
        .collect()
}

/// The tokens of the part a path without a wildcard names, if any: `rows.i2` is `i2`,
/// `rows.i2.earlier.x` is `i2` and `compare:earlier`, `headings.h3` is `headings:h3`,
/// `matched.totals` is `matched`, `compare.earlier` is `compare:earlier`.
fn part_tokens(result: &ToolResult, rest: &[&str]) -> Vec<String> {
    let mut node = &result.node;
    let mut at = 0usize;
    while let Some(segment) = rest.get(at) {
        if *segment == "rows" {
            // A heading or total line (`h3`) is the sum of the rows under it: it is the whole, so it
            // has no row of its own to be a part of anything.
            let mut out: Vec<String> = rest
                .get(at.saturating_add(1))
                .filter(|key| !is_total_line(key))
                .map(|key| (*key).to_owned())
                .into_iter()
                .collect();
            out.extend(compare_tokens(
                rest.get(at.saturating_add(2)..).unwrap_or(&[]),
            ));
            return out;
        }
        if PARTS.contains(segment) {
            let next = rest.get(at.saturating_add(1));
            let keyed = matches!(node.get(&[*segment]), Some(Node::Keyed(_)));
            return vec![match next {
                // The earlier period's total sits at `compare.totals`.
                Some(&"totals") if *segment == "compare" => PERIOD_TOKEN.to_owned(),
                Some(name) if keyed || PARTS_BY_NAME.contains(segment) => {
                    element_token(segment, name)
                }
                _ => (*segment).to_owned(),
            }];
        }
        node = match node.get(&[*segment]) {
            Some(next) => next,
            None => return Vec::new(),
        };
        at = at.saturating_add(1);
    }
    Vec::new()
}

fn join(mut tokens: Vec<String>) -> String {
    tokens.sort();
    tokens.dedup();
    if tokens.is_empty() {
        TILE.to_owned()
    } else {
        tokens.join("|")
    }
}

/// The scopes of the figures one path names: one for a path, one for each row for a wildcard.
fn scopes_of_path(
    state: &State,
    result: &ToolResult,
    segments: &[&str],
    depth: u8,
    counter: &mut u32,
) -> Vec<String> {
    let rest = segments.get(1..).unwrap_or(&[]);
    let star = rest.iter().position(|s| *s == "*");
    match result.tool {
        "find" => find_scopes(result, rest, counter),
        "calculate" => {
            if depth >= MAX_DEPTH {
                return vec![fresh_unknown(counter)];
            }
            // The scopes of its inputs, each part kept (`i1+i2`).
            let inputs = result
                .input
                .get("inputs")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();
            let mut parts: Vec<String> = Vec::new();
            for input in inputs_at(state, &inputs, depth.saturating_add(1), counter) {
                parts.extend(input.scope.split('+').map(str::to_owned));
            }
            parts.sort();
            parts.dedup();
            // A calculation of the Owner's own numbers has no scope.
            if parts.is_empty() {
                Vec::new()
            } else {
                vec![parts.join("+")]
            }
        }
        _ => {
            let Some(base) = base_tokens(result) else {
                return vec![fresh_unknown(counter)];
            };
            match star {
                None => {
                    let mut tokens = base;
                    tokens.extend(part_tokens(result, rest));
                    vec![join(tokens)]
                }
                Some(star) => {
                    let container_path = rest.get(..star).unwrap_or(&[]);
                    let name = container_path.last().copied().unwrap_or("");
                    let Some(container) = result.node.get(container_path) else {
                        return vec![fresh_unknown(counter)];
                    };
                    let keys: Vec<String> = match container {
                        Node::Keyed(rows) => rows
                            .iter()
                            .filter(|(key, _)| !is_total_line(key))
                            .map(|(key, _)| key.clone())
                            .collect(),
                        _ => Vec::new(),
                    };
                    if keys.is_empty() {
                        return vec![fresh_unknown(counter)];
                    }
                    let after = rest.get(star.saturating_add(1)..).unwrap_or(&[]);
                    keys.into_iter()
                        .map(|key| {
                            let mut tokens = base.clone();
                            tokens.push(element_token(name, &key));
                            tokens.extend(compare_tokens(after));
                            join(tokens)
                        })
                        .collect()
                }
            }
        }
    }
}

/// The days a result was read for.
fn window_of(result: &ToolResult) -> Window {
    match result.source.as_ref() {
        Some(AnswerSource::Analytics { filters, .. }) => {
            if filters.period.is_some() {
                Window::Unknown
            } else {
                Window::Span(filters.from.clone(), filters.to.clone())
            }
        }
        Some(AnswerSource::Report { filters, .. }) => {
            if !filters.as_of.is_empty() {
                Window::Point
            } else if filters.shortcut.is_some() {
                Window::Unknown
            } else {
                Window::Span(filters.from.clone(), filters.to.clone())
            }
        }
        _ => Window::Unknown,
    }
}

/// The scopes of the figures of a `find` result: each match is its own key.
fn find_scopes(result: &ToolResult, rest: &[&str], counter: &mut u32) -> Vec<String> {
    let key_of = |node: &Node| match node.get(&["key"]) {
        Some(Node::Str(key)) => key.clone(),
        _ => String::new(),
    };
    match rest {
        ["matches", "*", ..] => match result.node.get(&["matches"]) {
            Some(Node::List(items)) if !items.is_empty() => items
                .iter()
                .map(|item| {
                    let key = key_of(item);
                    if key.is_empty() {
                        fresh_unknown(counter)
                    } else {
                        key
                    }
                })
                .collect(),
            _ => vec![fresh_unknown(counter)],
        },
        ["matches", at, ..] => match result.node.get(&["matches", at]) {
            Some(item) if !key_of(item).is_empty() => vec![key_of(item)],
            _ => vec![fresh_unknown(counter)],
        },
        _ => vec![fresh_unknown(counter)],
    }
}

fn inputs_at(state: &State, paths: &[String], depth: u8, counter: &mut u32) -> Vec<Input> {
    let mut out = Vec::new();
    for path in paths {
        // A number of the question is the Owner's own: it has no scope.
        if path.starts_with("q.") {
            continue;
        }
        let segments: Vec<&str> = path.split('.').collect();
        let root = segments.first().copied().unwrap_or("").to_owned();
        let field = segments.last().copied().unwrap_or("").to_owned();
        let wild = segments.contains(&"*");
        let found = state.find(&root);
        let scopes = match found {
            Some(result) => scopes_of_path(state, result, &segments, depth, counter),
            None => vec![fresh_unknown(counter)],
        };
        // A tile holds several figures under its name: the name is part of what is measured.
        let measure = match found {
            Some(result) if result.tool == "tiles" && segments.len() > 2 => {
                format!("{}.{field}", segments.get(1).copied().unwrap_or(""))
            }
            _ => field,
        };
        // A balance or a closing figure is a position on one day, whatever dates the report was
        // read with: two of them are never added up.
        let is_balance = ["balance", "closing", "opening"]
            .iter()
            .any(|word| measure.contains(word));
        let window = if is_balance {
            Window::Point
        } else {
            found.map_or(Window::Unknown, window_of)
        };
        for scope in scopes {
            out.push(Input {
                scope,
                root: root.clone(),
                measure: measure.clone(),
                window: window.clone(),
                wild,
                calc: found.is_some_and(|result| result.tool == "calculate"),
            });
        }
    }
    out
}

/// The figures of the input paths of a calculation with their scopes: one for each figure that is
/// not a number of the question; a path with a wildcard gives one for each row it names.
pub fn inputs_of(state: &State, paths: &[String]) -> Vec<Input> {
    inputs_at(state, paths, 0, &mut 0)
}

/// The tokens of a scope that is one part (not a chain), without the periods.
fn tokens_of(scope: &str) -> Vec<&str> {
    scope
        .split('|')
        .filter(|t| *t != TILE && *t != PERIOD_TOKEN)
        .collect()
}

/// How the figures of a calculation relate.
pub fn mix_of(inputs: &[Input]) -> Mix {
    let mut distinct: Vec<&str> = Vec::new();
    for input in inputs {
        if !distinct.contains(&input.scope.as_str()) {
            distinct.push(input.scope.as_str());
        }
    }
    let mut mix = Mix {
        distinct: distinct.len(),
        wild: inputs.iter().filter(|i| i.wild).count(),
        ..Mix::default()
    };
    let inside = |small: &[&str], big: &[&str]| {
        small.len() < big.len() && small.iter().all(|t| big.contains(t))
    };
    for (at, a) in inputs.iter().enumerate() {
        for b in inputs.iter().skip(at.saturating_add(1)) {
            let same_scope = a.scope == b.scope && !a.scope.starts_with(UNKNOWN);
            if same_scope && a.root == b.root && a.measure == b.measure {
                // The same figure named twice (a row and a wildcard that holds it).
                mix.twice = true;
            }
            if same_scope && a.measure == b.measure && a.root != b.root {
                mix.repeated = true;
                if !a.window.apart_from(&b.window) {
                    mix.overlapping = true;
                }
            }
            if same_scope
                && !a.window.apart_from(&b.window)
                && (measure_inside(&a.measure, &b.measure)
                    || measure_inside(&b.measure, &a.measure))
            {
                mix.whole_and_part = true;
            }
            // Each part of a chain against each part of the other input.
            let (parts_a, parts_b): (Vec<&str>, Vec<&str>) = (
                a.scope
                    .split('+')
                    .filter(|p| !p.starts_with(UNKNOWN))
                    .collect(),
                b.scope
                    .split('+')
                    .filter(|p| !p.starts_with(UNKNOWN))
                    .collect(),
            );
            let chain = a.calc
                || b.calc
                || parts_a.len() > 1
                || parts_b.len() > 1
                || a.scope.contains('+')
                || b.scope.contains('+');
            for pa in &parts_a {
                for pb in &parts_b {
                    if pa == pb {
                        if chain {
                            mix.twice = true;
                        }
                    } else if inside(&tokens_of(pa), &tokens_of(pb))
                        || inside(&tokens_of(pb), &tokens_of(pa))
                        || item_in_group(&tokens_of(pa), &tokens_of(pb))
                        || item_in_group(&tokens_of(pb), &tokens_of(pa))
                    {
                        mix.whole_and_part = true;
                    }
                }
            }
        }
    }
    mix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_of_the_question_has_no_scope_and_an_unknown_result_is_unknown() {
        let state = State {
            results: Vec::new(),
            turn: 0,
        };
        assert!(inputs_of(&state, &["q.n1".to_owned()]).is_empty());
        let unknown = inputs_of(
            &state,
            &["r9.totals.x".to_owned(), "r8.totals.x".to_owned()],
        );
        assert_eq!(unknown.len(), 2);
        assert!(unknown.iter().all(|i| i.scope.starts_with(UNKNOWN)));
        // Two unknowns are never the same scope.
        assert_eq!(mix_of(&unknown).distinct, 2);
    }

    fn input(scope: &str, root: &str, measure: &str) -> Input {
        Input {
            scope: scope.to_owned(),
            root: root.to_owned(),
            measure: measure.to_owned(),
            window: Window::Unknown,
            wild: false,
            calc: false,
        }
    }

    fn span(mut input: Input, from: &str, to: &str) -> Input {
        input.window = Window::Span(from.to_owned(), to.to_owned());
        input
    }

    #[test]
    fn a_group_is_a_part_of_the_whole_and_of_its_items_but_two_groups_are_apart() {
        // The whole shop and a group: a part and its whole.
        assert!(mix_of(&[input("*", "r1", "x"), input("group:fruit", "r2", "x")]).whole_and_part);
        // A group and one item: the scope cannot know the item's group, so it is taken as nested.
        assert!(mix_of(&[input("group:fruit", "r1", "x"), input("i1", "r2", "x")]).whole_and_part);
        assert!(
            mix_of(&[input("i1", "r2", "x"), input("group:vegetable", "r1", "x")]).whole_and_part
        );
        // The same, with another filter alongside.
        assert!(
            mix_of(&[
                input("a3|group:fruit", "r1", "x"),
                input("a3|i1", "r2", "x")
            ])
            .whole_and_part
        );
        // Fruit and vegetable are two separate parts that make the whole: they may be added.
        let groups = mix_of(&[
            input("group:fruit", "r1", "x"),
            input("group:vegetable", "r2", "x"),
        ]);
        assert!(!groups.whole_and_part);
        assert_eq!(groups.distinct, 2);
        // A group of one seller and an item of another seller are not nested.
        assert!(
            !mix_of(&[
                input("a3|group:fruit", "r1", "x"),
                input("a4|i1", "r2", "x")
            ])
            .whole_and_part
        );
        // Tokens: `i1` is an item, `ad3` and `ct2` are not.
        assert!(is_item("i1") && is_item("i24"));
        assert!(!is_item("ad3") && !is_item("ct2") && !is_item("i") && !is_item("group:fruit"));
    }

    #[test]
    fn the_filters_of_a_read_give_the_group_token() {
        use mandi_services::core::rules::ItemGroup;
        let analytics = SourceAnalyticsFilters {
            item_group: Some(ItemGroup::Vegetable),
            ..SourceAnalyticsFilters::default()
        };
        assert_eq!(analytics_tokens(&analytics), ["group:vegetable"]);
        let report = SourceReportFilters {
            item_group: Some(ItemGroup::Fruit),
            ..SourceReportFilters::default()
        };
        assert_eq!(report_tokens(&report), ["group:fruit"]);
    }

    #[test]
    fn a_part_and_its_whole_are_found_by_their_tokens() {
        let whole = mix_of(&[input("*", "r1", "x"), input("i1", "r1", "x")]);
        assert!(whole.whole_and_part);
        let side = mix_of(&[
            input("side:seller", "r1", "x"),
            input("i1|side:seller", "r2", "x"),
        ]);
        assert!(side.whole_and_part);
        let two_items = mix_of(&[input("i1", "r1", "x"), input("i2", "r1", "x")]);
        assert!(!two_items.whole_and_part);
        assert_eq!(two_items.distinct, 2);
        // A different period of the same thing is a scope of its own but not a part.
        let period = mix_of(&[input("*", "r1", "x"), input("compare:earlier", "r1", "x")]);
        assert!(!period.whole_and_part);
        assert_eq!(period.distinct, 2);
    }

    #[test]
    fn the_same_figure_from_two_results_is_a_repeat_and_only_apart_periods_may_be_added() {
        let this_month = span(input("i1", "r1", "x"), "2026-10-01", "2026-10-31");
        let this_year = span(input("i1", "r2", "x"), "2026-01-01", "2026-12-31");
        let september = span(input("i1", "r3", "x"), "2026-09-01", "2026-09-30");
        let overlap = mix_of(&[this_month.clone(), this_year]);
        assert!(overlap.repeated && overlap.overlapping);
        let apart = mix_of(&[this_month.clone(), september]);
        assert!(apart.repeated && !apart.overlapping);
        // The same days twice, a balance of two days, and days that are not known all overlap.
        let again = mix_of(&[
            this_month.clone(),
            span(input("i1", "r4", "x"), "2026-10-01", "2026-10-31"),
        ]);
        assert!(again.overlapping);
        let mut point = input("i1", "r5", "x");
        point.window = Window::Point;
        assert!(mix_of(&[this_month, point]).overlapping);
        assert!(mix_of(&[input("i1", "r1", "x"), input("i1", "r2", "x")]).overlapping);
    }

    #[test]
    fn a_figure_named_twice_a_chain_holding_a_part_and_a_part_of_a_chain_are_counted_twice() {
        assert!(mix_of(&[input("i1", "r1", "x"), input("i1", "r1", "x")]).twice);
        // Different measures of one scope are not the same figure.
        assert!(!mix_of(&[input("i1", "r1", "x"), input("i1", "r1", "y")]).twice);
        let chain = mix_of(&[input("i1+i2", "c1", "value"), input("i1", "r3", "x")]);
        assert!(chain.twice);
        // A chain against the total of all: banana is a part of it.
        let chain = mix_of(&[input("i1+i2", "c1", "value"), input("*", "r3", "x")]);
        assert!(chain.whole_and_part);
        // A chain with an unknown part still has its known parts checked.
        let chain = mix_of(&[input("?1+i1", "c1", "value"), input("i1", "r3", "x")]);
        assert!(chain.twice);
    }

    #[test]
    fn a_measure_made_of_other_measures_of_its_own_result_is_a_whole() {
        let whole = mix_of(&[input("*", "r1", "total_comm"), input("*", "r1", "laga")]);
        assert!(whole.whole_and_part);
        let tiles = mix_of(&[
            input("*", "r1", "total_dues.amount"),
            input("*", "r1", "buyers_dues.amount"),
        ]);
        assert!(tiles.whole_and_part);
        // Two columns that are not made of each other are two figures.
        let apart = mix_of(&[
            input("*", "r1", "seller_comm"),
            input("*", "r1", "buyer_comm"),
        ]);
        assert!(!apart.whole_and_part);
    }
}
