//! Refusals that teach (plan 51 P51-B2). Umar asked "apni sale ko ziyada kaise karain": the model
//! sorted on `sales`, which is no column, and asked for a comparison over `up_to_now`, which has no
//! start date. Each refusal gave the model an engine field name, an empty detail and no list, so it
//! could not correct itself, and the question ended as "not understood".
//!
//! These tests send the first tries a model plausibly makes (from the playbook of `system.md`) and
//! check what it reads back: the tool's own field name, a reason in words, and the allowed words.
//! The key test: every word offered in `allowed` works when the model tries again.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use mandi_assistant::results::{ToolResult, Words, fit, tool_result_json};
use mandi_assistant::tools::{Env, State, Stop, ToolError, call};
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::domain::figures::{NumberUnit, QuestionNumber};
use serde_json::{Value, json};

struct Session<'a> {
    world: &'a world::World,
    words: Words,
    numbers: Vec<QuestionNumber>,
    state: State,
}

impl<'a> Session<'a> {
    fn new(world: &'a world::World) -> Self {
        Self {
            world,
            words: Words::new().unwrap(),
            numbers: Vec::new(),
            state: State::default(),
        }
    }

    fn run(&mut self, tool: &str, args: Value) -> Result<String, Stop> {
        let env = Env {
            ctx: &self.world.env.ctx,
            actor: &self.world.owner,
            words: &self.words,
            language: Language::Ur,
            today: BusinessDate::parse_iso("2026-10-10").unwrap(),
            partner_locked: false,
            numbers: &self.numbers,
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let draft = call(&env, &self.state, tool, &args.to_string())?;
        let prefix = match draft.tool {
            "calculate" => 'c',
            "estimate" => 'e',
            _ => 'r',
        };
        let id = self.state.next_id(prefix);
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
        Ok(id)
    }

    fn json(&self, id: &str) -> Value {
        tool_result_json(self.state.find(id).unwrap(), &self.words)
    }

    /// The refusal the model reads.
    fn refused(&mut self, tool: &str, args: Value) -> ToolError {
        match self.run(tool, args) {
            Err(Stop::Error(e)) => e,
            Err(Stop::Locked) => panic!("locked, not an error"),
            Ok(id) => panic!("{tool} should have been refused, gave {id}"),
        }
    }

    fn period(&mut self, kind: &str) -> String {
        self.run(
            "period",
            json!({"kind": kind, "month": null, "day": null, "from": null, "to": null, "n": null}),
        )
        .unwrap()
    }
}

fn analytics(view: &str, extra: Value) -> Value {
    let mut base = json!({
        "view": view, "from": null, "to": null, "group_by": null, "item": null, "seller": null,
        "buyer": null, "city": null, "adda": null, "garden": null, "trip": null, "goods": null,
        "partner": null, "side": null, "compare": null, "sort": null, "rows": null
    });
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

fn report(id: &str, extra: Value) -> Value {
    let mut base = json!({
        "id": id, "from": null, "to": null, "as_of": null, "party": null, "item": null, "city": null,
        "adda": null, "collector": null, "partner": null, "trip": null, "garden": null,
        "bikri": null, "group_by": null, "where": null, "sort": null, "rows": null
    });
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

fn day() -> world::World {
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    w
}

fn words_of(error: &ToolError) -> Vec<String> {
    error.allowed.clone()
}

/// What the model reads: code, field, a non-empty detail, and the words if there are any.
fn read(error: &ToolError) -> Value {
    let v = error.to_json();
    assert!(
        !v["error"]["detail"].as_str().unwrap().is_empty(),
        "an empty detail: {v}"
    );
    let field = v["error"]["field"].as_str().unwrap_or_default();
    assert!(
        !field.starts_with("filters."),
        "the engine's field name reached the model: {v}"
    );
    v
}

// ---- the sort column --------------------------------------------------------------------------

#[test]
fn a_sort_on_a_word_that_is_not_a_column_lists_the_columns_and_each_one_works() {
    let w = day();
    let mut s = Session::new(&w);
    s.period("this_year"); // r1
    for (view, invented) in [
        ("A", "sales"),
        ("B", "sales"),
        ("H", "sales"),
        ("A", "commission"),
        ("B", "qty"),
        ("C", "balance"),
    ] {
        let bad = analytics(
            view,
            json!({"from": "r1.from", "to": "r1.to", "sort": [{"column": invented, "order": "desc"}], "rows": 10}),
        );
        let e = s.refused("analytics", bad);
        let v = read(&e);
        assert_eq!(v["error"]["code"], "ERR_OUT_OF_RANGE", "{view} {invented}");
        assert_eq!(v["error"]["field"], "sort.column");
        let allowed = words_of(&e);
        assert!(allowed.len() >= 4, "{view}: {allowed:?}");
        // The words are the columns the same view gives back when it runs.
        let ok = s
            .run(
                "analytics",
                analytics(view, json!({"from": "r1.from", "to": "r1.to"})),
            )
            .unwrap();
        let columns: Vec<String> = s.json(&ok)["columns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().to_owned())
            .collect();
        assert_eq!(allowed, columns, "view {view}");
        // And every word offered is a sort that works: the model corrects itself in one try.
        for word in &allowed {
            let retry = analytics(
                view,
                json!({"from": "r1.from", "to": "r1.to", "sort": [{"column": word, "order": "desc"}], "rows": 10}),
            );
            assert!(s.run("analytics", retry).is_ok(), "{view} sort {word}");
        }
    }
}

#[test]
fn a_compared_view_also_offers_the_difference_and_earlier_words() {
    let w = day();
    let mut s = Session::new(&w);
    s.period("this_year");
    let e = s.refused(
        "analytics",
        analytics(
            "A",
            json!({"from": "r1.from", "to": "r1.to", "compare": "previous",
                   "sort": [{"column": "sales", "order": "desc"}]}),
        ),
    );
    let allowed = words_of(&e);
    assert!(
        allowed.contains(&"difference.total_comm".to_owned()),
        "{allowed:?}"
    );
    assert!(
        allowed.contains(&"earlier.total_comm".to_owned()),
        "{allowed:?}"
    );
    for word in &allowed {
        let retry = analytics(
            "A",
            json!({"from": "r1.from", "to": "r1.to", "compare": "previous",
                   "sort": [{"column": word, "order": "desc"}]}),
        );
        assert!(s.run("analytics", retry).is_ok(), "sort {word}");
    }
}

#[test]
fn a_report_sort_on_a_word_that_is_not_a_column_lists_the_columns_and_each_one_works() {
    let w = day();
    let mut s = Session::new(&w);
    s.period("this_year"); // r1
    for (id, dated) in [
        ("R2", true),
        ("R10", false),
        ("R20", false),
        ("R24", false),
        ("R26", false),
        ("R29", true),
        ("R12", true),
        ("R32", false),
    ] {
        let extra = |sort: Value| {
            if dated {
                json!({"from": "r1.from", "to": "r1.to", "sort": sort})
            } else {
                json!({"sort": sort})
            }
        };
        let e = s.refused(
            "report",
            report(
                id,
                extra(json!([{"column": "amount_due_xyz", "order": "desc"}])),
            ),
        );
        let v = read(&e);
        assert_eq!(v["error"]["field"], "sort.column", "{id}");
        let allowed = words_of(&e);
        assert!(!allowed.is_empty(), "{id} lists no column");
        let ok = s.run("report", report(id, extra(Value::Null))).unwrap();
        let columns: Vec<String> = s.json(&ok)["columns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().to_owned())
            .collect();
        assert_eq!(allowed, columns, "report {id}");
        for word in &allowed {
            let retry = report(id, extra(json!([{"column": word, "order": "desc"}])));
            // A column that holds both sides is refused with its own message, which is still a
            // reason and not a missing column.
            match s.run("report", retry) {
                Ok(_) => {}
                Err(Stop::Error(e)) => assert!(
                    e.detail.contains("both sides"),
                    "{id} sort {word}: {}",
                    e.to_json()
                ),
                Err(Stop::Locked) => panic!("locked"),
            }
        }
    }
}

// ---- up_to_now and the comparison -------------------------------------------------------------

#[test]
fn a_comparison_over_up_to_now_says_it_needs_both_dates() {
    let w = day();
    let mut s = Session::new(&w);
    s.period("up_to_now"); // r1: from is empty
    for from in [json!(null), json!("r1.from")] {
        let e = s.refused(
            "analytics",
            analytics(
                "A",
                json!({"from": from, "to": "r1.to", "compare": "previous", "rows": 10}),
            ),
        );
        let v = read(&e);
        assert_eq!(v["error"]["code"], "ERR_REQUIRED");
        assert_eq!(v["error"]["field"], "from");
        let detail = v["error"]["detail"].as_str().unwrap();
        assert!(detail.contains("both dates"), "{detail}");
        assert!(detail.contains("this_year"), "{detail}");
    }
}

#[test]
fn a_reference_to_the_empty_from_of_up_to_now_means_no_date() {
    let w = day();
    let mut s = Session::new(&w);
    s.period("up_to_now"); // r1
    // The prompt says: pass a period's dates on by reference. For up_to_now the from is empty.
    let a = s
        .run(
            "analytics",
            analytics("A", json!({"from": "r1.from", "to": "r1.to"})),
        )
        .unwrap();
    let a = s.json(&a);
    assert_eq!(a["count"]["n"], 2, "{a}");
    assert!(a["filters"].get("from").is_none(), "{a}");
    assert_eq!(a["filters"]["to"], "2026-10-10");
    // The same figures as asking with no from at all.
    let b = s
        .run("analytics", analytics("A", json!({"to": "r1.to"})))
        .unwrap();
    assert_eq!(s.json(&b)["rows"], a["rows"]);
    assert_eq!(s.json(&b)["totals"], a["totals"]);
    // A report takes it too: an empty from is the first day of the books.
    let r = s
        .run(
            "report",
            report("R12", json!({"from": "r1.from", "to": "r1.to"})),
        )
        .unwrap();
    assert!(s.json(&r)["count"]["n"].as_i64().unwrap() > 0);
    // Only the from of a period may be empty: an as_of or a to with it is refused, not made today
    // (money review).
    let e = s.refused("report", report("R20", json!({"as_of": "r1.from"})));
    assert_eq!(e.code, "ERR_UNKNOWN_REFERENCE");
    assert_eq!(e.field.as_deref(), Some("as_of"));
    assert!(
        read(&e)["error"]["detail"]
            .as_str()
            .unwrap()
            .contains("no start date")
    );
    let e = s.refused(
        "analytics",
        analytics("A", json!({"from": "r1.from", "to": "r1.from"})),
    );
    assert_eq!(e.field.as_deref(), Some("to"));
    // A money field that is empty is no date either, even as a from.
    let find = s
        .run("find", json!({"text": "akram", "role": "buyer"}))
        .unwrap();
    let card = s
        .run("party_card", json!({"party": format!("{find}.matches.0")}))
        .unwrap();
    let e = s.refused(
        "analytics",
        analytics(
            "A",
            json!({"from": format!("{card}.open_advance"), "to": "r1.to"}),
        ),
    );
    assert_eq!(e.code, "ERR_UNKNOWN_REFERENCE");
    // A path that is no date is still refused, with a reason.
    let e = s.refused(
        "analytics",
        analytics("A", json!({"from": "r1.kind", "to": "r1.to"})),
    );
    assert_eq!(e.code, "ERR_UNKNOWN_REFERENCE");
    assert!(
        read(&e)["error"]["detail"]
            .as_str()
            .unwrap()
            .contains("not a date")
    );
}

#[test]
fn a_closed_period_still_compares_as_before() {
    let w = day();
    let mut s = Session::new(&w);
    s.period("this_year");
    let r = s
        .run(
            "analytics",
            analytics(
                "A",
                json!({"from": "r1.from", "to": "r1.to", "compare": "previous", "rows": 10}),
            ),
        )
        .unwrap();
    let j = s.json(&r);
    assert_eq!(j["compare"]["difference"]["total_comm"]["rs"], 5_448);
    assert_eq!(j["totals"]["total_comm"]["rs"], 5_448);
}

// ---- filters a view or a report does not have -------------------------------------------------

#[test]
fn a_filter_a_view_does_not_have_lists_the_views_that_do() {
    let w = day();
    let mut s = Session::new(&w);
    s.period("this_year");
    let dated = |extra: Value| {
        let mut base = json!({"from": "r1.from", "to": "r1.to"});
        for (k, v) in extra.as_object().unwrap() {
            base[k] = v.clone();
        }
        base
    };
    for (view, extra, field, must_have, must_not) in [
        ("G", json!({"compare": "previous"}), "compare", "A", "G"),
        ("F", json!({"compare": "previous"}), "compare", "D", "F"),
        ("I", json!({"compare": "last_year"}), "compare", "H", "I"),
        ("A", json!({"group_by": "month"}), "group_by", "D", "A"),
        ("B", json!({"group_by": "month"}), "group_by", "G", "B"),
    ] {
        let e = s.refused("analytics", analytics(view, dated(extra)));
        let v = read(&e);
        assert_eq!(v["error"]["field"], field, "{view}");
        let allowed = words_of(&e);
        assert!(
            allowed.contains(&must_have.to_owned()),
            "{view}: {allowed:?}"
        );
        assert!(
            !allowed.contains(&must_not.to_owned()),
            "{view}: {allowed:?}"
        );
    }
}

#[test]
fn a_filter_a_report_does_not_have_lists_the_filters_it_takes() {
    let w = day();
    let mut s = Session::new(&w);
    s.period("up_to_now");
    // A balance list takes as_of, not from and to.
    for id in ["R20", "R24", "R26"] {
        let e = s.refused("report", report(id, json!({"to": "r1.to"})));
        let v = read(&e);
        assert_eq!(v["error"]["field"], "to", "{id}");
        assert!(
            words_of(&e).contains(&"as_of".to_owned()),
            "{id}: {:?}",
            e.allowed
        );
        assert!(!words_of(&e).contains(&"to".to_owned()), "{id}");
        assert!(
            s.run("report", report(id, json!({"as_of": "r1.to"})))
                .is_ok(),
            "{id} as_of"
        );
    }
}

// ---- references and numbers -------------------------------------------------------------------

#[test]
fn a_number_typed_in_where_says_where_numbers_come_from() {
    let w = day();
    let mut s = Session::new(&w);
    s.numbers = vec![QuestionNumber {
        number: 30,
        text: "30".to_owned(),
        unit: NumberUnit::Days,
        bps: None,
    }];
    let e = s.refused(
        "report",
        report(
            "R20",
            json!({"where": {"column": "days", "op": "gt", "value": "30"}}),
        ),
    );
    let v = read(&e);
    assert_eq!(v["error"]["code"], "ERR_UNKNOWN_REFERENCE");
    assert_eq!(v["error"]["field"], "where.value");
    assert!(words_of(&e).contains(&"q.n1".to_owned()), "{:?}", e.allowed);
    // Trying again with the word it was offered works.
    let ok = s.run(
        "report",
        report(
            "R20",
            json!({"where": {"column": "days", "op": "gt", "value": "q.n1"}}),
        ),
    );
    assert!(ok.is_ok());
}

#[test]
fn an_engine_refusal_reaches_the_model_with_the_tools_own_field_and_a_reason() {
    use mandi_services::core::error::ErrorCode;
    let e = ToolError::from_code(ErrorCode::ErrRequired, Some("filters.from"));
    let v = read(&e);
    assert_eq!(v["error"]["field"], "from");
    assert!(
        v["error"]["detail"]
            .as_str()
            .unwrap()
            .contains("both dates")
    );
    let e = ToolError::from_code(ErrorCode::ErrDateRange, Some("filters.to"));
    assert_eq!(read(&e)["error"]["detail"], "to is before from");
    let e = ToolError::from_code(ErrorCode::ErrNotFound, None);
    assert!(!read(&e)["error"]["detail"].as_str().unwrap().is_empty());
}

#[test]
fn a_refusal_with_no_detail_still_gets_one() {
    let e = ToolError {
        code: "ERR_REQUIRED".to_owned(),
        field: Some("from".to_owned()),
        detail: String::new(),
        allowed: Vec::new(),
    };
    assert!(!read(&e)["error"]["detail"].as_str().unwrap().is_empty());
    // And no `allowed` key when there is no list.
    assert!(e.to_json()["error"].get("allowed").is_none());
}

// ---- where on a column with nothing in it ------------------------------------------------------

#[test]
fn a_where_on_an_empty_report_counts_zero_and_a_where_on_a_text_column_lists_the_number_columns() {
    let w = day();
    let mut s = Session::new(&w);
    s.numbers = vec![QuestionNumber {
        number: 2,
        text: "2".to_owned(),
        unit: NumberUnit::Days,
        bps: None,
    }];
    // No goods are waiting on the worked day: "kaun sa maal 2 din se para hai" is 0, not a refusal.
    let r = s
        .run(
            "report",
            report(
                "R10",
                json!({"where": {"column": "days", "op": "gt", "value": "q.n1"}}),
            ),
        )
        .unwrap();
    let j = s.json(&r);
    assert_eq!(j["matched"]["count"]["n"], 0, "{j}");
    assert_eq!(j["count"]["n"], 0);
    // A column with rows but no number in it (the name) lists the columns that have numbers.
    let e = s.refused(
        "report",
        report(
            "R20",
            json!({"where": {"column": "name", "op": "gt", "value": "q.n1"}}),
        ),
    );
    let v = read(&e);
    assert_eq!(v["error"]["field"], "where.column");
    let allowed = words_of(&e);
    assert!(allowed.contains(&"balance".to_owned()), "{allowed:?}");
    assert!(allowed.contains(&"days".to_owned()), "{allowed:?}");
    assert!(!allowed.contains(&"name".to_owned()), "{allowed:?}");
    // A value that is no number of the question is still refused when the column is empty.
    let e = s.refused(
        "report",
        report(
            "R10",
            json!({"where": {"column": "days", "op": "gt", "value": "99"}}),
        ),
    );
    assert_eq!(e.code, "ERR_UNKNOWN_REFERENCE");
}
