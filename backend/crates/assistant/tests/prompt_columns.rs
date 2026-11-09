//! P8 of D6-system-prompt.md Section 7 (plan 51 P51-B2): the sort and where words that
//! `tools.json` lists are the columns the engine returns, both ways.
//!
//! Umar's question "apni sale ko ziyada kaise karain" failed because the model sorted on `sales`,
//! which is no column of any view. The tool descriptions now list the columns of every report and
//! every analytics view, and the schemas offer them as enums, so the model cannot invent one. A
//! word listed here that the engine does not return, or a column the engine returns that the list
//! leaves out, would send the model to a refusal again: this test fails on both.
//!
//! The columns are read from the engine itself, by running every report and every view on the
//! worked day (the partner section unlocked). Two views have columns that depend on the data: view
//! D has one column for each month (`p_2026_10`) and view E one for each item (`i_1`); the lists
//! write them as `p_<year>_<month>` and `i_<item_id>`, and they are read but not sorted, so they
//! are not in the enum.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use std::collections::{BTreeMap, BTreeSet};

use mandi_assistant::prompt::TOOLS_JSON;
use mandi_assistant::results::{ToolResult, Words, fit, tool_result_json};
use mandi_assistant::tools::{Env, State, Stop, call};
use mandi_reports::analytics::{AnalyticsFilterKind, catalog as view_catalog};
use mandi_reports::dashboard::BackupFacts;
use mandi_reports::reports::catalog as report_catalog;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::partner_code::{PartnerCodeSetInput, PartnerCodeUnlockInput};
use mandi_services::partner_code;
use serde_json::{Value, json};

type Words2 = BTreeSet<String>;

struct Session<'a> {
    world: &'a world::World,
    words: Words,
    state: State,
}

impl Session<'_> {
    fn run(&mut self, tool: &str, args: Value) -> Value {
        let env = Env {
            ctx: &self.world.env.ctx,
            actor: &self.world.owner,
            words: &self.words,
            language: Language::Ur,
            today: BusinessDate::parse_iso("2026-10-10").unwrap(),
            partner_locked: false,
            numbers: &[],
            question: "",
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let draft = match call(&env, &self.state, tool, &args.to_string()) {
            Ok(draft) => draft,
            Err(Stop::Error(e)) => panic!("{tool} {args} was refused: {}", e.to_json()),
            Err(Stop::Locked) => panic!("{tool} {args}: the partner section is locked"),
        };
        let id = self.state.next_id('r');
        let mut result = ToolResult {
            id: id.clone(),
            tool: draft.tool,
            input: draft.input,
            node: draft.node,
            shown: draft.shown,
            source: draft.source,
            partner_parts_locked: draft.partner_parts_locked,
            partner_data: draft.partner_data,
            turn: 0,
            working: draft.working,
        };
        fit(&mut result, &self.words);
        self.state.results.push(result);
        tool_result_json(self.state.find(&id).unwrap(), &self.words)
    }
}

fn report_args(id: &str, extra: Value) -> Value {
    let mut base = json!({
        "id": id, "from": null, "to": null, "as_of": null, "party": null, "item": null,
        "city": null, "adda": null, "collector": null, "partner": null, "trip": null,
        "garden": null, "bikri": null, "group_by": null, "where": null, "sort": null, "rows": 1
    });
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

fn view_args(view: &str, extra: Value) -> Value {
    let mut base = json!({
        "view": view, "from": "2026-01-01", "to": "2026-10-10", "group_by": null, "item": null,
        "seller": null, "buyer": null, "city": null, "adda": null, "garden": null, "trip": null,
        "goods": null, "partner": null, "side": null, "compare": null, "sort": null, "rows": 1
    });
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

fn columns_of(result: &Value) -> Vec<String> {
    result["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_owned())
        .collect()
}

/// The month columns of view D and the item columns of view E are written as a pattern.
fn pattern(view: &str, word: &str) -> String {
    let digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    let parts: Vec<&str> = word.split('_').collect();
    match (view, parts.as_slice()) {
        ("D", ["p", year, month]) if digits(year) && digits(month) => "p_<year>_<month>".to_owned(),
        ("D", ["p", year, month, day]) if digits(year) && digits(month) && digits(day) => {
            "p_<year>_<month>".to_owned()
        }
        ("E", ["i", id]) if digits(id) => "i_<item_id>".to_owned(),
        _ => word.to_owned(),
    }
}

/// What the engine returns: the columns of each report and each view on the worked day.
struct Engine {
    reports: BTreeMap<String, Words2>,
    views: BTreeMap<String, Words2>,
}

fn engine() -> Engine {
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    partner_code::set(
        &w.env.ctx,
        &w.owner,
        &PartnerCodeSetInput {
            password: common::OWNER_PASSWORD.to_owned(),
            code: "4826".to_owned(),
            code_again: "4826".to_owned(),
        },
    )
    .unwrap();
    partner_code::unlock(
        &w.env.ctx,
        &w.owner,
        &PartnerCodeUnlockInput {
            code: "4826".to_owned(),
        },
    )
    .unwrap();
    let mut s = Session {
        world: &w,
        words: Words::new().unwrap(),
        state: State::default(),
    };
    s.run("find", json!({"text": "akram", "role": "buyer"})); // r1
    s.run("find", json!({"text": "الف", "role": "partner"})); // r2
    let mut reports: BTreeMap<String, Words2> = BTreeMap::new();
    for entry in report_catalog() {
        let id = entry.id.code().to_owned();
        let variants: Vec<Value> = match id.as_str() {
            "R1" => vec![json!({"bikri": 1})],
            "R13" | "R14" => vec![json!({"party": "r1.matches.0"})],
            // The overview of every partner, and the statement of one.
            "R31" => vec![json!({}), json!({"partner": "r2.matches.0"})],
            "R28" => vec![
                json!({}),
                json!({"group_by": "day"}),
                json!({"group_by": "month"}),
            ],
            _ => vec![json!({})],
        };
        for extra in variants {
            let result = s.run("report", report_args(&id, extra));
            reports
                .entry(id.clone())
                .or_default()
                .extend(columns_of(&result));
        }
    }
    let mut views: BTreeMap<String, Words2> = BTreeMap::new();
    for entry in view_catalog() {
        let id = entry.id.code().to_owned();
        let mut variants = vec![json!({})];
        if entry.filters.contains(&AnalyticsFilterKind::Compare) {
            variants.push(json!({"compare": "previous"}));
        }
        if entry.filters.contains(&AnalyticsFilterKind::GroupBy) {
            variants.push(json!({"group_by": "day"}));
            variants.push(json!({"group_by": "month"}));
        }
        for extra in variants {
            let result = s.run("analytics", view_args(&id, extra));
            views
                .entry(id.clone())
                .or_default()
                .extend(columns_of(&result).iter().map(|c| pattern(&id, c)));
        }
    }
    Engine { reports, views }
}

/// The text of a tool's description and its parameters.
fn tool(tools: &Value, name: &str) -> Value {
    tools
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["function"]["name"] == name)
        .unwrap_or_else(|| panic!("no tool {name}"))["function"]
        .clone()
}

/// The part of a description between a marker and the end of the sentence.
fn list_after<'d>(description: &'d str, marker: &str, end: &str) -> &'d str {
    let from = description
        .find(marker)
        .unwrap_or_else(|| panic!("no {marker:?} in the description"))
        + marker.len();
    let rest = &description[from..];
    &rest[..rest
        .find(end)
        .unwrap_or_else(|| panic!("no {end:?} after {marker:?}"))]
}

/// `A name nag | B name ...` as (letters, words) in order, the words with no repeat.
fn segments(list: &str, is_id: impl Fn(&str) -> bool) -> Vec<(Vec<String>, Vec<String>)> {
    list.split(" | ")
        .map(|segment| {
            let tokens: Vec<&str> = segment.split_whitespace().collect();
            let ids: Vec<String> = tokens
                .iter()
                .take_while(|t| is_id(t))
                .map(|t| (*t).to_owned())
                .collect();
            let words: Vec<String> = tokens
                .iter()
                .skip(ids.len())
                .map(|t| (*t).to_owned())
                .collect();
            (ids, words)
        })
        .collect()
}

fn is_report_id(token: &str) -> bool {
    token
        .strip_prefix('R')
        .is_some_and(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit()))
}

fn is_view_id(token: &str) -> bool {
    token.len() == 1 && token.chars().all(|c| c.is_ascii_uppercase())
}

fn enum_of(node: &Value) -> Words2 {
    node["enum"]
        .as_array()
        .unwrap_or_else(|| panic!("no enum in {node}"))
        .iter()
        .map(|w| w.as_str().unwrap().to_owned())
        .collect()
}

/// The compared figure of a view: the buyer's purchase value in C, the total commission otherwise.
fn compared_figure(view: &str) -> &'static str {
    if view == "C" {
        "purchase_value"
    } else {
        "total_comm"
    }
}

pub fn check(tools_text: &str) {
    let tools: Value = serde_json::from_str(tools_text).unwrap();
    let engine = engine();

    // ---- reports: every id once, each list equal to the engine's columns -----------------------
    let report = tool(&tools, "report");
    let description = report["description"].as_str().unwrap();
    let listed = segments(
        list_after(
            description,
            "Sort and where words are the columns of the report: ",
            ".",
        ),
        is_report_id,
    );
    let mut seen: BTreeMap<String, Words2> = BTreeMap::new();
    for (ids, words) in &listed {
        let set: Words2 = words.iter().cloned().collect();
        assert_eq!(set.len(), words.len(), "a word listed twice for {ids:?}");
        for id in ids {
            assert!(
                seen.insert(id.clone(), set.clone()).is_none(),
                "{id} listed twice"
            );
        }
    }
    assert_eq!(
        seen.keys().collect::<BTreeSet<_>>(),
        engine.reports.keys().collect::<BTreeSet<_>>(),
        "the reports listed and the catalogue"
    );
    for (id, want) in &engine.reports {
        assert_eq!(
            &seen[id], want,
            "the columns of {id}: listed, then the engine's"
        );
    }
    // The enum of `report` is the union, and `sort` and `where` both use it.
    let union: Words2 = engine.reports.values().flatten().cloned().collect();
    let params = &report["parameters"];
    assert_eq!(
        enum_of(&params["$defs"]["column"]),
        union,
        "the column enum of report"
    );
    let uses = json!({"$ref": "#/$defs/column"});
    assert_eq!(params["$defs"]["where"]["properties"]["column"], uses);
    assert_eq!(
        params["$defs"]["sort"]["items"]["properties"]["column"],
        uses
    );

    // ---- views: every letter once, each list equal to the engine's columns ---------------------
    let analytics = tool(&tools, "analytics");
    let description = analytics["description"].as_str().unwrap();
    let listed = segments(
        list_after(
            description,
            "Sort words are the columns of the view: ",
            ". With compare",
        ),
        is_view_id,
    );
    let mut seen: BTreeMap<String, Words2> = BTreeMap::new();
    for (ids, words) in &listed {
        assert_eq!(ids.len(), 1, "{ids:?}");
        let set: Words2 = words.iter().cloned().collect();
        assert_eq!(
            set.len(),
            words.len(),
            "a word listed twice for view {ids:?}"
        );
        assert!(
            seen.insert(ids[0].clone(), set).is_none(),
            "{ids:?} listed twice"
        );
    }
    assert_eq!(
        seen.keys().collect::<BTreeSet<_>>(),
        engine.views.keys().collect::<BTreeSet<_>>(),
        "the views listed and the catalogue"
    );
    for (id, want) in &engine.views {
        assert_eq!(
            &seen[id], want,
            "the columns of view {id}: listed, then the engine's"
        );
    }
    // Compare and group_by: the letters of the description are the catalogue's.
    let letters = |text: &str| -> Words2 {
        text.split_whitespace()
            .take_while(|t| is_view_id(t) || *t == "and")
            .filter(|t| is_view_id(t))
            .map(str::to_owned)
            .collect()
    };
    let compare: Words2 = view_catalog()
        .iter()
        .filter(|e| e.filters.contains(&AnalyticsFilterKind::Compare))
        .map(|e| e.id.code().to_owned())
        .collect();
    let group: Words2 = view_catalog()
        .iter()
        .filter(|e| e.filters.contains(&AnalyticsFilterKind::GroupBy))
        .map(|e| e.id.code().to_owned())
        .collect();
    assert_eq!(
        letters(list_after(description, "Compare is for views ", ";")),
        compare
    );
    assert_eq!(
        letters(list_after(description, "group_by is for ", ".")),
        group
    );
    // The compared words: X is total_comm, and purchase_value in C.
    assert!(description.contains("X being total_comm (purchase_value in C)"));
    for view in &compare {
        assert!(
            engine.views[view].contains(compared_figure(view)),
            "view {view} has its compared figure as a column"
        );
    }
    // The enum of the analytics sort: the fixed words of every view, and the compared words.
    let mut want: Words2 = engine
        .views
        .values()
        .flatten()
        .filter(|w| !w.contains('<'))
        .cloned()
        .collect();
    for figure in ["total_comm", "purchase_value"] {
        want.insert(format!("difference.{figure}"));
        want.insert(format!("earlier.{figure}"));
    }
    let schema = &analytics["parameters"]["$defs"]["sort"]["items"]["properties"]["column"];
    assert_eq!(enum_of(schema), want, "the sort enum of analytics");
}

// Waiting for plan 65: D6 v1.8 lists the columns `paid` and `left` of R3, R4 and R5, which the engine
// gets with the money paid on the spot (PRD FR-12.44). Plan 64 only moves the prompt files to v1.8.
#[test]
#[ignore = "waiting for plan 65: columns paid and left of R3, R4 and R5"]
fn p8_the_columns_of_tools_json_are_the_columns_of_the_engine() {
    check(TOOLS_JSON);
}
