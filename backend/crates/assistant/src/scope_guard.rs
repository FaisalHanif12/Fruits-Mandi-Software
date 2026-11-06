//! The scope guard (live run 2, the lead's CRITICAL finding): an answer must not rest on a read that
//! dropped what the question said.
//!
//! Two wrong figures came from model reads that lost a part of the question: "how much was sold of
//! vegetables last week" read with no group gave the whole shop's 704,040 (the true answer is 0), and
//! "kela ka commission is saal" read with no period gave all the books' 3,071,368 under the words
//! "this year". The figure was real, checked in its two places, and for the wrong thing.
//!
//! When the app understood the question, the reads it plans for it (`understand::plan_reads`) are the
//! right ones for the period, the item, the group and the party the question names. The model may
//! read what it likes, but a view or a report it CITES must carry the same scope the planned read
//! carries. If it does not, the answer is not shown: the model is told which results hold the right
//! scope (read by the app, not by the model) and writes the answer once more.
//!
//! The guard only acts when the planned read is exactly one view or report (a comparison, a list or
//! a question the app is not sure of plans none or several, and is left to the other checks), and
//! only on the dimensions the cited view or report takes. A scope that cannot be told (a date given
//! as a reference to the period result) is never a violation.
//!
//! A quality (grade) of a sale line is a dimension like an item: "اول کیلا کی sale آج" asks for the
//! lines of the first quality only (75,980), and the whole banana of the day (183,640) is another
//! figure. When the planned read carries a `grade`, a cited report that takes the filter must carry
//! it too (live run 2, the lead's note). View A and the reports with the Grade filter take it; the
//! other views hold no quality.
//!
//! The same holds for the goods (consigned, trip, garden), the garden, the trip, the buyer and the
//! seller (live run 2, Q1257: "the gardens' sale" read with no garden filter gave the whole shop's
//! 95,006,520).
//!
//! Example: the question "phal ki is saal ki sale" plans view A with `item_group: fruit` and the
//! dates of this year; an answer that cites a view A with `item_group: null` is refused.

use serde_json::Value;

use crate::check::Failure;
use crate::model_answer::{ModelAnswer, Piece, pieces};
use crate::tools::State;

/// The views that take an item, a group, a buyer and a seller (PRD 18.1: A to E and H).
const VIEWS_WITH_FILTERS: [&str; 6] = ["A", "B", "C", "D", "E", "H"];

/// What a planned view or report carries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AskScope {
    /// First day, `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Last day.
    pub to: Option<String>,
    /// The item, as its key (`i1`).
    pub item: Option<String>,
    /// The group word (`fruit`, `vegetable`).
    pub group: Option<String>,
    /// The party, as his key (`a20`), in whichever of `party`, `buyer`, `seller` or `zamindar` the
    /// planned read carries it.
    pub party: Option<String>,
    /// The quality of a sale line (`first`, `second` ...), as the planned read carries it.
    pub grade: Option<String>,
    /// The buyer, as his key.
    pub buyer: Option<String>,
    /// The seller, as his key, in `seller` or `zamindar`.
    pub seller: Option<String>,
    /// The garden, as its key.
    pub garden: Option<String>,
    /// The market trip, as its key.
    pub trip: Option<String>,
    /// The kind of goods (`consigned`, `trip`, `garden`): `all` is no filter and is left out.
    pub goods: Option<String>,
}

fn text_of(input: &Value, field: &str) -> Option<String> {
    input
        .get(field)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

impl AskScope {
    /// The scope of the one planned view or report, or `None` when the reads hold no view or report,
    /// or more than one. A balance list (`as_of`) keeps no dates.
    pub fn of_planned(planned: &[(String, Value)]) -> Option<AskScope> {
        let mut views = planned
            .iter()
            .filter(|(tool, _)| tool == "analytics" || tool == "report");
        let (_, input) = views.next()?;
        if views.next().is_some() {
            return None;
        }
        let as_of = text_of(input, "as_of").is_some();
        let party = ["party", "buyer", "seller", "zamindar"]
            .iter()
            .find_map(|field| text_of(input, field));
        let scope = AskScope {
            from: text_of(input, "from").filter(|_| !as_of),
            to: text_of(input, "to").filter(|_| !as_of),
            item: text_of(input, "item"),
            group: text_of(input, "item_group"),
            party,
            grade: text_of(input, "grade"),
            buyer: text_of(input, "buyer"),
            seller: text_of(input, "seller").or_else(|| text_of(input, "zamindar")),
            garden: text_of(input, "garden"),
            trip: text_of(input, "trip"),
            goods: text_of(input, "goods").filter(|g| g != "all"),
        };
        (scope != AskScope::default()).then_some(scope)
    }
}

/// True for a text that is a day, `YYYY-MM-DD`.
fn is_day(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        })
}

/// True for a key such as `i1` or `a20`: a letter and digits.
fn is_key(text: &str) -> bool {
    let mut chars = text.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.clone().count() > 0
        && chars.all(|c| c.is_ascii_digit())
}

/// True for a group word (`fruit`, `vegetable`): letters only.
fn is_word(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_lowercase())
}

/// True for a quality as the tools write it: letters, digits and `_` (`first`, `a`, `grade_2`).
fn is_grade(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// True for a kind of goods as the tools write it: lower case letters and `_`.
fn is_goods(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_lowercase() || c == '_')
}

/// Whether a cited read dropped a party the question named: none of `fields` holds a key, or they
/// hold only other keys. A reference or any other shape cannot be told and is not a violation.
fn drops_party(wanted: &Option<String>, fields: &[&str], input: &Value) -> bool {
    let Some(wanted) = wanted else {
        return false;
    };
    for field in fields {
        match input.get(*field) {
            None | Some(Value::Null) => {}
            Some(Value::String(s)) if s.is_empty() => {}
            Some(Value::String(s)) if is_key(s) => {
                if s == wanted {
                    return false;
                }
            }
            Some(_) => return false,
        }
    }
    true
}

/// Whether a field of a cited read breaks the scope: it was dropped (null), or it is another day or
/// another key. A text that cannot be told (a reference) is not a violation.
fn breaks(wanted: &Option<String>, field: Option<&Value>, same_kind: fn(&str) -> bool) -> bool {
    let Some(wanted) = wanted else {
        return false;
    };
    match field {
        None | Some(Value::Null) => true,
        Some(Value::String(s)) if s.is_empty() => true,
        Some(Value::String(s)) if same_kind(s) => s != wanted,
        _ => false,
    }
}

/// Whether the cited result `input` breaks the scope. `view` is the analytics view, `None` for a
/// report; `takes` says which dimensions a report takes.
fn violation(scope: &AskScope, input: &Value, takes: &Takes) -> bool {
    let dates_ok = !(breaks(&scope.from, input.get("from"), is_day)
        || breaks(&scope.to, input.get("to"), is_day));
    let period_breaks = takes.period && !dates_ok;
    let item_breaks = takes.item && breaks(&scope.item, input.get("item"), is_key);
    let group_breaks = takes.group && breaks(&scope.group, input.get("item_group"), is_word);
    let grade_breaks = takes.grade && breaks(&scope.grade, input.get("grade"), is_grade);
    let party_breaks = takes.party
        && scope.party.is_some()
        && ["party", "buyer", "seller", "zamindar"]
            .iter()
            .filter_map(|f| input.get(*f))
            .all(|v| v.is_null() || v.as_str() == Some(""));
    // The goods, the garden, the trip and the buyer or seller of the question (live run 2, Q1257: a
    // read of the garden's sales that dropped the garden gave the whole shop's 95,006,520).
    let goods_breaks = takes.goods && breaks(&scope.goods, input.get("goods"), is_goods);
    let garden_breaks = takes.garden && breaks(&scope.garden, input.get("garden"), is_key);
    let trip_breaks = takes.trip && breaks(&scope.trip, input.get("trip"), is_key);
    let buyer_breaks = takes.buyer && drops_party(&scope.buyer, &["buyer", "party"], input);
    let seller_breaks =
        takes.seller && drops_party(&scope.seller, &["seller", "zamindar", "party"], input);
    period_breaks
        || item_breaks
        || group_breaks
        || party_breaks
        || grade_breaks
        || goods_breaks
        || garden_breaks
        || trip_breaks
        || buyer_breaks
        || seller_breaks
}

/// The dimensions a cited view or report takes.
#[derive(Debug, Clone, Copy)]
struct Takes {
    period: bool,
    item: bool,
    group: bool,
    party: bool,
    grade: bool,
    buyer: bool,
    seller: bool,
    garden: bool,
    trip: bool,
    goods: bool,
}

fn takes_of(tool: &str, input: &Value) -> Option<Takes> {
    match tool {
        "analytics" => {
            let view = input.get("view").and_then(Value::as_str).unwrap_or("");
            let filters = VIEWS_WITH_FILTERS.contains(&view);
            // What the view takes, from its catalogue entry: the quality of the sale lines (view A,
            // PRD FR-12.42), the buyer, the seller, the garden, the trip and the goods.
            use mandi_reports::analytics::AnalyticsFilterKind as K;
            let kinds = serde_json::from_value::<mandi_reports::analytics::AnalyticsView>(
                Value::String(view.to_owned()),
            )
            .ok()
            .and_then(|v| {
                mandi_reports::analytics::catalog()
                    .into_iter()
                    .find(|e| e.id == v)
            })
            .map(|e| e.filters)
            .unwrap_or_default();
            Some(Takes {
                period: true,
                item: filters,
                group: filters,
                party: filters,
                grade: kinds.contains(&K::Grade),
                buyer: kinds.contains(&K::Buyer),
                seller: kinds.contains(&K::Seller),
                garden: kinds.contains(&K::Garden),
                trip: kinds.contains(&K::Trip),
                goods: kinds.contains(&K::Goods),
            })
        }
        "report" => {
            let id = input.get("id").and_then(Value::as_str).unwrap_or("");
            let report: mandi_reports::reports::ReportId =
                serde_json::from_value(Value::String(id.to_owned())).ok()?;
            let entry = mandi_reports::reports::catalog()
                .into_iter()
                .find(|e| e.id == report)?;
            use mandi_reports::reports::FilterKind as F;
            let has = |k: F| entry.filters.contains(&k);
            // A balance list (as of a day) has no period.
            let as_of = has(F::AsOf);
            Some(Takes {
                period: has(F::DateRange) && !as_of,
                item: has(F::Item),
                group: has(F::ItemGroup),
                party: has(F::Party) || has(F::Seller) || has(F::Zamindar),
                grade: has(F::Grade),
                // A report's buyer is its `party`; no report takes a trip or the goods.
                buyer: false,
                seller: has(F::Seller) || has(F::Zamindar),
                garden: has(F::Garden),
                trip: false,
                goods: false,
            })
        }
        _ => None,
    }
}

/// The ids of the results an answer cites: in its words and in what it shows.
fn cited_ids(answer: &ModelAnswer) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    let mut put = |id: &str| {
        if !ids.iter().any(|x| x == id) {
            ids.push(id.to_owned());
        }
    };
    for piece in pieces(&answer.text) {
        if let Piece::Ref(r) = piece
            && let Some(first) = r.path.split('.').next()
        {
            put(first);
        }
    }
    for shown in &answer.show {
        put(&shown.result);
    }
    ids
}

/// The failures of an answer that cites a view or a report with another scope than the question's.
/// `right` are the ids of the results that hold the right scope, named to the model.
pub fn failures(
    scope: &AskScope,
    state: &State,
    answer: &ModelAnswer,
    right: &[String],
) -> Vec<Failure> {
    let mut out = Vec::new();
    for id in cited_ids(answer) {
        let Some(result) = state.find(&id).filter(|r| r.turn == state.turn) else {
            continue;
        };
        let Some(takes) = takes_of(result.tool, &result.input) else {
            continue;
        };
        if violation(scope, &result.input, &takes) {
            out.push(Failure::scope(&id, right));
        }
    }
    out
}

/// The failures of an answer made only of the app's guide (`app_help`) when the question asks for a
/// figure and a report, a view, a tile or an estimate was read for it (live run 2, G3: "total open
/// advances" read R26 and answered with the advance screen's guide). `how_to` is true for a question
/// that asks how to do something or where something is; for it the guide is the answer.
pub fn guide_only(state: &State, answer: &ModelAnswer, how_to: bool) -> Vec<Failure> {
    use crate::model_answer::AnswerKind;
    if how_to || !matches!(answer.kind, AnswerKind::Answer | AnswerKind::Advice) {
        return Vec::new();
    }
    let cited = cited_ids(answer);
    let Some(first) = cited.first() else {
        return Vec::new();
    };
    let guide_only = cited
        .iter()
        .all(|id| state.find(id).is_some_and(|r| r.tool == "app_help"));
    if !guide_only {
        return Vec::new();
    }
    let figures: Vec<String> = state
        .results
        .iter()
        .filter(|r| r.turn == state.turn)
        .filter(|r| {
            matches!(
                r.tool,
                "report" | "analytics" | "tiles" | "party_card" | "estimate"
            )
        })
        .map(|r| r.id.clone())
        .collect();
    if figures.is_empty() {
        return Vec::new();
    }
    vec![Failure {
        instead: figures,
        ..Failure::new("guide_only", first)
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn planned(input: Value) -> Vec<(String, Value)> {
        vec![("analytics".to_owned(), input)]
    }

    #[test]
    fn the_scope_is_that_of_the_one_planned_view_and_none_for_two() {
        let one = AskScope::of_planned(&planned(json!({
            "view": "A", "from": "2026-01-01", "to": "2026-11-05", "item": "i1",
            "item_group": null, "buyer": null, "seller": null
        })))
        .unwrap();
        assert_eq!(one.item.as_deref(), Some("i1"));
        assert_eq!(one.from.as_deref(), Some("2026-01-01"));
        let two = vec![
            (
                "analytics".to_owned(),
                json!({"view": "A", "from": "2026-01-01"}),
            ),
            (
                "analytics".to_owned(),
                json!({"view": "A", "from": "2025-01-01"}),
            ),
        ];
        assert_eq!(AskScope::of_planned(&two), None);
        assert_eq!(
            AskScope::of_planned(&[("tiles".to_owned(), json!({}))]),
            None
        );
    }

    #[test]
    fn a_dropped_dimension_breaks_the_scope_and_a_reference_never_does() {
        let scope = AskScope {
            from: Some("2026-01-01".into()),
            to: Some("2026-11-05".into()),
            item: Some("i1".into()),
            ..AskScope::default()
        };
        let takes = takes_of("analytics", &json!({"view": "A"})).unwrap();
        let right = json!({"view": "A", "from": "2026-01-01", "to": "2026-11-05", "item": "i1"});
        assert!(!violation(&scope, &right, &takes));
        let no_period = json!({"view": "A", "from": null, "to": null, "item": "i1"});
        assert!(violation(&scope, &no_period, &takes));
        let no_item = json!({"view": "A", "from": "2026-01-01", "to": "2026-11-05", "item": null});
        assert!(violation(&scope, &no_item, &takes));
        let other_day =
            json!({"view": "A", "from": "2025-01-01", "to": "2026-11-05", "item": "i1"});
        assert!(violation(&scope, &other_day, &takes));
        // The model gave the dates and the item as references to earlier results: cannot be told.
        let by_reference =
            json!({"view": "A", "from": "r1.from", "to": "r1.to", "item": "r2.matches.0"});
        assert!(!violation(&scope, &by_reference, &takes));
        // A view that takes no item is not asked for one.
        let trend = takes_of("analytics", &json!({"view": "G"})).unwrap();
        assert!(!violation(
            &scope,
            &json!({"view": "G", "from": "2026-01-01", "to": "2026-11-05"}),
            &trend
        ));
    }
    #[test]
    fn a_report_that_takes_the_quality_must_carry_the_quality_of_the_question() {
        // "اول کیلا کی sale آج": the register of the day for the item, the first quality only.
        let planned = vec![(
            "report".to_owned(),
            json!({"id": "R2", "from": "2026-11-05", "to": "2026-11-05", "item": "i1",
                   "grade": "first"}),
        )];
        let scope = AskScope::of_planned(&planned).unwrap();
        assert_eq!(scope.grade.as_deref(), Some("first"));
        let takes = takes_of("report", &json!({"id": "R2"})).unwrap();
        assert!(takes.grade, "R2 takes the quality filter");
        let right = json!({"id": "R2", "from": "2026-11-05", "to": "2026-11-05", "item": "i1",
                           "grade": "first"});
        assert!(!violation(&scope, &right, &takes));
        // The whole item of the day, with no quality: 183,640 and not 75,980.
        let none = json!({"id": "R2", "from": "2026-11-05", "to": "2026-11-05", "item": "i1",
                          "grade": null});
        assert!(violation(&scope, &none, &takes));
        let missing = json!({"id": "R2", "from": "2026-11-05", "to": "2026-11-05", "item": "i1"});
        assert!(violation(&scope, &missing, &takes));
        // Another quality.
        let other = json!({"id": "R2", "from": "2026-11-05", "to": "2026-11-05", "item": "i1",
                           "grade": "second"});
        assert!(violation(&scope, &other, &takes));
        // View A takes the quality (PRD FR-12.42): a planned view A with the grade refuses a cited
        // view A without it; the other views hold no quality and are not asked for one. A question
        // with no quality asks nothing of the cited read.
        let view_a = takes_of("analytics", &json!({"view": "A"})).unwrap();
        assert!(view_a.grade);
        let planned_view = vec![(
            "analytics".to_owned(),
            json!({"view": "A", "from": "2026-11-05", "to": "2026-11-05", "item": "i1",
                   "grade": "first"}),
        )];
        let view_scope = AskScope::of_planned(&planned_view).unwrap();
        assert_eq!(view_scope.grade.as_deref(), Some("first"));
        let whole_item = json!({"view": "A", "from": "2026-11-05", "to": "2026-11-05",
                                "item": "i1", "grade": null});
        assert!(violation(&view_scope, &whole_item, &view_a));
        let first_only = json!({"view": "A", "from": "2026-11-05", "to": "2026-11-05",
                                "item": "i1", "grade": "first"});
        assert!(!violation(&view_scope, &first_only, &view_a));
        assert!(!takes_of("analytics", &json!({"view": "B"})).unwrap().grade);
        let plain = AskScope {
            item: Some("i1".into()),
            ..AskScope::default()
        };
        assert!(!violation(&plain, &none, &takes));
    }
    #[test]
    fn a_read_that_dropped_the_goods_the_garden_the_trip_or_the_buyer_is_refused() {
        // Q1257, the sales of the gardens: view A for the goods of gardens, and one garden. A read
        // of the same view with no goods filter is the whole shop's sales.
        let planned = vec![(
            "analytics".to_owned(),
            json!({"view": "A", "from": "2026-01-01", "to": "2026-11-05", "goods": "garden",
                   "garden": "g1", "trip": null, "buyer": null, "seller": null}),
        )];
        let scope = AskScope::of_planned(&planned).unwrap();
        assert_eq!(scope.goods.as_deref(), Some("garden"));
        assert_eq!(scope.garden.as_deref(), Some("g1"));
        let takes = takes_of("analytics", &json!({"view": "A"})).unwrap();
        assert!(takes.goods && takes.garden && takes.trip && takes.buyer && takes.seller);
        let base = json!({"view": "A", "from": "2026-01-01", "to": "2026-11-05"});
        let with = |extra: Value| {
            let mut v = base.clone();
            if let (Some(m), Some(e)) = (v.as_object_mut(), extra.as_object()) {
                for (k, x) in e {
                    m.insert(k.clone(), x.clone());
                }
            }
            v
        };
        let right = with(json!({"goods": "garden", "garden": "g1"}));
        assert!(!violation(&scope, &right, &takes));
        // The goods dropped, "all", another kind, and the garden dropped or another garden.
        assert!(violation(&scope, &with(json!({"garden": "g1"})), &takes));
        assert!(violation(
            &scope,
            &with(json!({"goods": "all", "garden": "g1"})),
            &takes
        ));
        assert!(violation(
            &scope,
            &with(json!({"goods": "trip", "garden": "g1"})),
            &takes
        ));
        assert!(violation(&scope, &with(json!({"goods": "garden"})), &takes));
        assert!(violation(
            &scope,
            &with(json!({"goods": "garden", "garden": "g2"})),
            &takes
        ));
        // A reference cannot be told.
        assert!(!violation(
            &scope,
            &with(json!({"goods": "garden", "garden": "r2.matches.0"})),
            &takes
        ));
        // "all" in the planned read is no filter, so nothing is asked.
        let open = AskScope::of_planned(&[(
            "analytics".to_owned(),
            json!({"view": "A", "from": "2026-01-01", "to": "2026-11-05", "goods": "all"}),
        )])
        .unwrap();
        assert_eq!(open.goods, None);
        assert!(!violation(&open, &base, &takes));

        // A trip.
        let trip = AskScope::of_planned(&[(
            "analytics".to_owned(),
            json!({"view": "J", "from": "2026-01-01", "to": "2026-11-05", "trip": "t3"}),
        )])
        .unwrap();
        let trip_takes = takes_of("analytics", &json!({"view": "J"})).unwrap();
        assert!(trip_takes.trip);
        assert!(violation(&trip, &base, &trip_takes));
        assert!(violation(&trip, &with(json!({"trip": "t9"})), &trip_takes));
        assert!(!violation(&trip, &with(json!({"trip": "t3"})), &trip_takes));

        // A buyer: dropped, another buyer, or only a seller; the same buyer, or his party, pass.
        let buyer = AskScope::of_planned(&[(
            "analytics".to_owned(),
            json!({"view": "A", "from": "2026-01-01", "to": "2026-11-05", "buyer": "a20"}),
        )])
        .unwrap();
        assert!(violation(&buyer, &base, &takes));
        assert!(violation(&buyer, &with(json!({"buyer": "a21"})), &takes));
        assert!(violation(&buyer, &with(json!({"seller": "a5"})), &takes));
        assert!(!violation(&buyer, &with(json!({"buyer": "a20"})), &takes));
        assert!(!violation(
            &buyer,
            &with(json!({"buyer": "r2.matches.0"})),
            &takes
        ));
        // The reports take a garden and a seller too.
        let r36 = takes_of("report", &json!({"id": "R36"})).unwrap();
        assert!(r36.garden);
        let r6 = takes_of("report", &json!({"id": "R6"})).unwrap();
        assert!(r6.seller && !r6.goods && !r6.trip);
    }
}
