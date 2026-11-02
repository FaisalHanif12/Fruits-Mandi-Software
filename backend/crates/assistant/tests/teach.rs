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
    question: String,
    state: State,
}

impl<'a> Session<'a> {
    fn new(world: &'a world::World) -> Self {
        Self {
            world,
            words: Words::new().unwrap(),
            numbers: Vec::new(),
            question: String::new(),
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
            question: &self.question,
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
fn a_sort_on_a_word_that_is_not_a_column_is_left_out_with_a_note_and_each_listed_word_works() {
    // Plan 52 item 2: the tool fixes what is plainly meant and says what it did. A sort word the
    // view has no column for is dropped; every row is given in the view's own order, so a cut can
    // never pass for a top list.
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
        let id = s.run("analytics", bad).expect("the sort word is left out");
        let v = s.json(&id);
        let note = v["note"].as_str().expect("a note");
        assert!(note.contains(invented), "{view}: {note}");
        assert!(note.contains("left out"), "{view}: {note}");
        assert!(note.contains("every row is given"), "{view}: {note}");
        // Every row of the view is there.
        assert_eq!(
            i64::try_from(v["rows"].as_array().unwrap().len()).unwrap(),
            v["count"]["n"].as_i64().unwrap(),
            "{view}"
        );
        // The note lists the sort words of the view: the columns the view gives back.
        let ok = s
            .run(
                "analytics",
                analytics(view, json!({"from": "r1.from", "to": "r1.to"})),
            )
            .unwrap();
        assert!(
            s.json(&ok).get("note").is_none(),
            "no note when nothing was changed"
        );
        let columns: Vec<String> = s.json(&ok)["columns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().to_owned())
            .collect();
        let listed = note.split("sort words of this view are ").nth(1).unwrap();
        for column in &columns {
            assert!(
                listed.contains(column.as_str()),
                "{view}: {column} in {listed}"
            );
        }
        // And every word offered is a sort that works.
        for word in &columns {
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
    s.period("this_month");
    let id = s
        .run(
            "analytics",
            analytics(
                "A",
                json!({"from": "r1.from", "to": "r1.to", "compare": "previous",
                       "sort": [{"column": "sales", "order": "desc"}]}),
            ),
        )
        .expect("the sort word is left out");
    let note = s.json(&id)["note"].as_str().unwrap().to_owned();
    for word in ["difference.total_comm", "earlier.total_comm"] {
        assert!(note.contains(word), "{note}");
        let retry = analytics(
            "A",
            json!({"from": "r1.from", "to": "r1.to", "compare": "previous",
                   "sort": [{"column": word, "order": "desc"}]}),
        );
        assert!(s.run("analytics", retry).is_ok(), "sort {word}");
    }
}

#[test]
fn a_report_sort_on_a_word_that_is_not_a_column_is_left_out_and_each_listed_word_works() {
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
        let done = s
            .run(
                "report",
                report(
                    id,
                    extra(json!([{"column": "amount_due_xyz", "order": "desc"}])),
                ),
            )
            .unwrap_or_else(|_| panic!("{id}: the sort word is left out"));
        let note = s.json(&done)["note"].as_str().expect("a note").to_owned();
        assert!(
            note.contains("amount_due_xyz") && note.contains("left out"),
            "{id}: {note}"
        );
        let ok = s.run("report", report(id, extra(Value::Null))).unwrap();
        let columns: Vec<String> = s.json(&ok)["columns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().to_owned())
            .collect();
        let listed = note.split("sort words of this report are ").nth(1).unwrap();
        for word in &columns {
            assert!(listed.contains(word.as_str()), "{id}: {word} in {listed}");
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

#[test]
fn a_sort_word_is_left_out_only_when_every_row_can_be_given() {
    // A report of more than 100 rows cannot be given whole, and its first 100 rows in the report's
    // own order would pass for a top list: so a word that is no column is still refused there,
    // with the words that work (plan 52 item 2).
    use mandi_services::core::accounts::AccountType;
    let w = day();
    for n in 0..110 {
        world::add_account(
            &w.env,
            &w.owner,
            AccountType::Buyer,
            &format!("گاہک {n}"),
            &format!("Buyer {n}"),
            true,
            "",
        );
    }
    let mut s = Session::new(&w);
    let e = s.refused(
        "report",
        report(
            "R16",
            json!({"sort": [{"column": "amount_due_xyz", "order": "desc"}]}),
        ),
    );
    let v = read(&e);
    assert_eq!(v["error"]["field"], "sort.column");
    assert!(!words_of(&e).is_empty());
    // A real column still sorts it.
    let word = words_of(&e).first().unwrap().clone();
    let ok = s
        .run(
            "report",
            report("R16", json!({"sort": [{"column": word, "order": "desc"}]})),
        )
        .unwrap();
    assert!(s.json(&ok).get("note").is_none());
}

// ---- up_to_now and the comparison -------------------------------------------------------------

#[test]
fn a_comparison_needs_both_dates_and_one_from_the_first_day_of_the_books_is_left_out() {
    let w = day();
    let mut s = Session::new(&w);
    s.period("up_to_now"); // r1: from is the first day of the books
    // A `from` that is plainly empty (null) with a `to` is still refused, with the reason.
    let e = s.refused(
        "analytics",
        analytics(
            "A",
            json!({"from": null, "to": "r1.to", "compare": "previous", "rows": 10}),
        ),
    );
    let v = read(&e);
    assert_eq!(v["error"]["code"], "ERR_REQUIRED");
    assert_eq!(v["error"]["field"], "from");
    let detail = v["error"]["detail"].as_str().unwrap();
    assert!(detail.contains("both dates"), "{detail}");
    assert!(detail.contains("this_year"), "{detail}");
    // The period `up_to_now` starts on the first day of the books: nothing is before it, so the
    // comparison is left out and the note says why. No figure is shown as new.
    let id = s
        .run(
            "analytics",
            analytics(
                "A",
                json!({"from": "r1.from", "to": "r1.to", "compare": "previous", "rows": 10}),
            ),
        )
        .expect("no refusal");
    let v = s.json(&id);
    let note = v["note"].as_str().expect("a note");
    assert!(note.contains("no earlier period"), "{note}");
    assert!(v["compare"].is_null(), "{v}");
}

#[test]
fn a_comparison_with_no_dates_means_this_month_so_far_or_this_year_with_years() {
    // Plan 52 item 2. Today is 10-10-26 in these tests.
    let w = day();
    let mut s = Session::new(&w);
    let ok = |s: &mut Session<'_>, extra: Value| {
        let id = s
            .run("analytics", analytics("A", extra))
            .expect("the comparison is given");
        s.json(&id)
    };
    // No dates: this month so far against the same number of days before it.
    let v = ok(&mut s, json!({"compare": "previous", "rows": 10}));
    assert_eq!(v["filters"]["from"], "2026-10-01");
    assert_eq!(v["filters"]["to"], "2026-10-10");
    assert_eq!(v["compare"]["from"]["date"], "2026-09-21");
    assert_eq!(v["compare"]["to"]["date"], "2026-09-30");
    let note = v["note"].as_str().unwrap();
    assert!(note.contains("this month so far"), "{note}");
    assert!(
        note.contains("2026-10-01") && note.contains("2026-10-10"),
        "{note}"
    );
    // A comparison by year: this year so far against the same dates a year earlier. The books
    // start on 31-08-26, so last year is wholly before them: the comparison is left out, and the
    // note says both what was meant and why it is not given.
    let v = ok(&mut s, json!({"compare": "last_year", "rows": 10}));
    assert_eq!(v["filters"]["from"], "2026-01-01");
    assert!(v["compare"].is_null(), "{v}");
    let note = v["note"].as_str().unwrap();
    assert!(note.contains("this year so far"), "{note}");
    assert!(note.contains("no earlier period"), "{note}");
    // A year-by-year comparison of a period inside the books: the same dates a year earlier.
    let id = s
        .run(
            "analytics",
            analytics(
                "A",
                json!({"from": "2026-10-01", "to": "2026-10-10", "compare": "last_year",
                       "rows": 10}),
            ),
        )
        .unwrap();
    let v = s.json(&id);
    assert!(v["compare"].is_null(), "{v}");
    // The comparison figure is the engine's own: the same as a period typed by hand.
    let p = s.period("this_month");
    let by_hand = s
        .run(
            "analytics",
            analytics(
                "A",
                json!({"from": format!("{p}.from"), "to": format!("{p}.to"),
                       "compare": "previous", "rows": 10}),
            ),
        )
        .unwrap();
    let by_hand = s.json(&by_hand);
    let again = ok(&mut s, json!({"compare": "previous", "rows": 10}));
    assert_eq!(again["rows"], by_hand["rows"]);
    assert_eq!(again["totals"], by_hand["totals"]);
    assert_eq!(again["compare"], by_hand["compare"]);
}

#[test]
fn up_to_now_starts_at_the_first_day_of_the_books_so_its_from_is_a_real_date() {
    let w = day();
    let mut s = Session::new(&w);
    s.period("up_to_now"); // r1
    // The worked day starts on 31-08-26, its opening entry (plan 52 item 2).
    let period = s.json("r1");
    assert_eq!(period["from"]["date"], "2026-08-31", "{period}");
    assert!(
        period["note"]
            .as_str()
            .unwrap()
            .contains("first day of the books")
    );
    let a = s
        .run(
            "analytics",
            analytics("A", json!({"from": "r1.from", "to": "r1.to"})),
        )
        .unwrap();
    let a = s.json(&a);
    assert_eq!(a["count"]["n"], 2, "{a}");
    assert_eq!(a["filters"]["from"], "2026-08-31", "{a}");
    assert_eq!(a["filters"]["to"], "2026-10-10");
    // The same figures as asking with no from at all.
    let b = s
        .run("analytics", analytics("A", json!({"to": "r1.to"})))
        .unwrap();
    assert_eq!(s.json(&b)["rows"], a["rows"]);
    assert_eq!(s.json(&b)["totals"], a["totals"]);
    // A report takes it too.
    let r = s
        .run(
            "report",
            report("R12", json!({"from": "r1.from", "to": "r1.to"})),
        )
        .unwrap();
    assert!(s.json(&r)["count"]["n"].as_i64().unwrap() > 0);
    // With nothing posted yet there is no first day: the `from` stays empty, and it is a "no date"
    // only for a `from`, never for a `to` or an `as_of` (money review).
    let empty = world::world_with(0, false);
    let mut e = Session::new(&empty);
    e.period("up_to_now");
    assert!(e.json("r1")["from"].is_null(), "{}", e.json("r1"));
    let err = e.refused("report", report("R20", json!({"as_of": "r1.from"})));
    assert_eq!(err.code, "ERR_UNKNOWN_REFERENCE");
    assert_eq!(err.field.as_deref(), Some("as_of"));
    assert!(
        read(&err)["error"]["detail"]
            .as_str()
            .unwrap()
            .contains("no start date")
    );
    let err = e.refused(
        "analytics",
        analytics("A", json!({"from": "r1.from", "to": "r1.from"})),
    );
    assert_eq!(err.field.as_deref(), Some("to"));
    // A money field that is empty is no date either, even as a from.
    let find = s
        .run("find", json!({"text": "akram", "role": "buyer"}))
        .unwrap();
    let card = s
        .run("party_card", json!({"party": format!("{find}.matches.0")}))
        .unwrap();
    let err = s.refused(
        "analytics",
        analytics(
            "A",
            json!({"from": format!("{card}.open_advance"), "to": "r1.to"}),
        ),
    );
    assert_eq!(err.code, "ERR_UNKNOWN_REFERENCE");
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
fn a_closed_period_still_compares_as_before_when_the_earlier_period_is_inside_the_books() {
    let w = day();
    let mut s = Session::new(&w);
    // The books start on 31-08-26, so the 10 days before this month so far (21-09 to 30-09) are
    // inside them: a real 0 against 5,448.
    s.period("this_month");
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
    assert!(j.get("note").is_none(), "{j}");
}

#[test]
fn an_earlier_period_before_the_books_is_left_out_or_flagged_as_partly_covered() {
    // Money review of plan 52 item 2: "5,448 up from 0" would be wrong when there was no book at
    // all before. The worked day's books start on 31-08-26.
    let w = day();
    let mut s = Session::new(&w);
    // This year so far (01-01 to 10-10) against the 283 days before it: wholly before the books.
    s.period("this_year");
    let id = s
        .run(
            "analytics",
            analytics(
                "A",
                json!({"from": "r1.from", "to": "r1.to", "compare": "previous", "rows": 10}),
            ),
        )
        .unwrap();
    let v = s.json(&id);
    assert!(v["compare"].is_null(), "{v}");
    assert!(
        v["note"].as_str().unwrap().contains("no earlier period"),
        "{v}"
    );
    // The first 30 days of the books against the 30 days before them: also wholly before.
    // A period that starts a week into the books has an earlier period that starts before them.
    let id = s
        .run(
            "analytics",
            analytics(
                "A",
                json!({"from": "2026-09-07", "to": "2026-09-20", "compare": "previous", "rows": 10}),
            ),
        )
        .unwrap();
    let v = s.json(&id);
    let note = v["note"].as_str().expect("a note");
    assert!(note.contains("only partly covered"), "{note}");
    assert!(
        v["compare"].is_object(),
        "the comparison is still given: {v}"
    );
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
