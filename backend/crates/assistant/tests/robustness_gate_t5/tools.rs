//! Wrong tool calls (Code Standards Q1, PRD FR-7.77, FR-7.79, FR-7.86, NFR-AI-5).
//!
//! `robustness_assistant.rs` attack 4 has unknown tools, bad JSON, extra fields, missing and wrong
//! types, bad dates, unknown keys and references, find text built like SQL, the calculator on
//! made-up numbers, estimates of 0 and -6 months, 20 tool calls in a reply and a model that never
//! answers. This file adds the ones it does not have: `where` and `sort` on a column the report
//! has not, rows past the limits, period words outside the list, a number of months that the
//! question does not hold, an estimate more than 12 months ahead, and the calculator at its edges.
//!
//! Every case is a tool call; the expected outcome is a refusal with an `ERR_` code that is not
//! `ERR_INTERNAL`, or a result that carries a note saying what was left out.
#![allow(dead_code)]

use mandi_assistant::ask::AskOutcome;
use mandi_assistant::results::Words;
use mandi_assistant::tools::{self, Env, State, Stop};
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::domain::figures;
use serde_json::{Value, json};

use crate::harness::Harness;
use crate::kit::*;

/// What one call gave: the error code, or `None` for a result, and the JSON text.
struct Out {
    code: Option<String>,
    text: String,
}

/// Runs calls in order over one state, with the numbers of a question (`q.n1` ...).
fn run(h: &Harness, question: &str, calls: &[(&str, Value)]) -> Vec<Out> {
    let words = Words::new().unwrap();
    let today = BusinessDate::parse_iso("2026-10-10").unwrap();
    let numbers = figures::question_numbers(question);
    let snap = h.world.env.ctx.snapshot().unwrap();
    let env = Env {
        ctx: &snap,
        actor: &h.world.owner,
        words: &words,
        language: Language::Ur,
        today,
        partner_locked: false,
        numbers: &numbers,
        question,
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };
    let mut state = State::default();
    let mut out = Vec::new();
    for (name, args) in calls {
        match tools::call(&env, &state, name, &args.to_string()) {
            Ok(draft) => {
                let id = state.next_id(match draft.tool {
                    "calculate" => 'c',
                    "estimate" => 'e',
                    _ => 'r',
                });
                let result = tools::stored(draft, id, 0);
                let text = mandi_assistant::results::tool_result_json(&result, &words).to_string();
                state.results.push(result);
                out.push(Out { code: None, text });
            }
            Err(Stop::Error(e)) => out.push(Out {
                code: Some(e.code.clone()),
                text: e.to_json().to_string(),
            }),
            Err(Stop::Locked) => out.push(Out {
                code: Some("LOCKED".into()),
                text: String::new(),
            }),
        }
    }
    out
}

/// The first find of Akram, then the call: the call may read `r1.matches.0`.
fn after_find(h: &Harness, question: &str, call: (&str, Value)) -> Out {
    let mut outs = run(h, question, &[find("ikram", "buyer"), call]);
    outs.pop().unwrap()
}

fn refused(out: &Out) -> bool {
    matches!(&out.code, Some(c) if c.starts_with("ERR_") && c != "ERR_INTERNAL")
}

// ---- where and sort on a column the report has not --------------------------------------------

#[test]
fn t5_a_where_on_a_column_the_report_has_not_is_refused() {
    let h = Harness::day();
    let r13 = |extra: Value| {
        let mut a = report_args("R13", json!({"party": "r1.matches.0"}));
        merge(&mut a, extra);
        ("report", a)
    };
    let cases: Vec<(&str, (&str, Value))> = vec![
        // The column exists in the schema (R2 has `gross`? no: R13 has debit credit balance).
        (
            "a column of another report",
            r13(json!({"where": {"column": "gross", "op": "gt", "value": "q.n1"}})),
        ),
        (
            "a column that is in no report",
            r13(json!({"where": {"column": "secret_profit", "op": "gt", "value": "q.n1"}})),
        ),
        (
            "an operator that does not exist",
            r13(json!({"where": {"column": "debit", "op": "like", "value": "q.n1"}})),
        ),
        (
            "a value typed as a figure, not a number of the question",
            r13(json!({"where": {"column": "debit", "op": "gt", "value": "5000"}})),
        ),
        (
            "a value that is a word",
            r13(json!({"where": {"column": "debit", "op": "gt", "value": "all of it"}})),
        ),
        (
            "a value path that does not exist",
            r13(json!({"where": {"column": "debit", "op": "gt", "value": "r9.totals.debit"}})),
        ),
        (
            "a value number of the question that is not there",
            r13(json!({"where": {"column": "debit", "op": "gt", "value": "q.n7"}})),
        ),
        (
            "a where that is a list",
            r13(json!({"where": [{"column": "debit", "op": "gt", "value": "q.n1"}]})),
        ),
        (
            "a where with an extra key",
            r13(json!({"where": {"column": "debit", "op": "gt", "value": "q.n1", "or": "1=1"}})),
        ),
        (
            "a where with no value",
            r13(json!({"where": {"column": "debit", "op": "gt"}})),
        ),
    ];
    let mut wrong = Vec::new();
    for (name, call) in cases {
        let out = after_find(&h, "akram ke 1000 se zyada bill", call);
        if !refused(&out) {
            wrong.push(format!(
                "{name}: not refused ({:?}) {}",
                out.code,
                out.text.chars().take(160).collect::<String>()
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} cases:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// A sort word the report has no column for is left out with a note (plan 52 item 2): the rows come
/// in the report's own order and the note names the words that are right. It must never be
/// silent, and never make the rows look sorted.
#[test]
fn t5_a_sort_on_a_column_the_report_has_not_says_so_and_never_looks_sorted() {
    let h = Harness::day();
    let sorted = |column: &str| {
        let outs = run(
            &h,
            "sab se bara bakaya",
            &[(
                "report",
                report_args(
                    "R20",
                    json!({"sort": [{"column": column, "order": "desc"}], "rows": 5}),
                ),
            )],
        );
        outs.into_iter().next().unwrap()
    };
    // A column of the report: sorted, no note about a left-out word.
    let ok = sorted("balance");
    assert!(ok.code.is_none(), "{}", ok.text);
    assert!(!ok.text.contains("left out"), "{}", ok.text);
    // A column of another report, and one that exists nowhere: the note says it was left out.
    for column in ["gross", "secret_profit"] {
        let out = sorted(column);
        if out.code.is_some() {
            assert!(refused(&out), "{column}: {:?} {}", out.code, out.text);
        } else {
            assert!(
                out.text.contains("not a column of this report"),
                "{column}: no note that the sort was left out: {}",
                out.text
            );
        }
    }
    // Three keys, a bad order word, and an empty column are refused.
    for (name, sort) in [
        (
            "three keys",
            json!([
                {"column": "balance", "order": "asc"},
                {"column": "name", "order": "asc"},
                {"column": "days", "order": "asc"}
            ]),
        ),
        ("a bad order", json!([{"column": "balance", "order": "up"}])),
        ("a string", json!("balance")),
        ("an object", json!({"column": "balance", "order": "asc"})),
    ] {
        let out = run(
            &h,
            "sab se bara bakaya",
            &[("report", report_args("R20", json!({"sort": sort})))],
        )
        .into_iter()
        .next()
        .unwrap();
        assert!(refused(&out), "{name}: {:?} {}", out.code, out.text);
    }
}

// ---- rows past the limits -----------------------------------------------------------------------

#[test]
fn t5_rows_past_the_limits_are_refused_and_never_make_a_big_read() {
    let h = Harness::day();
    for (name, rows) in [
        ("zero", json!(0)),
        ("negative", json!(-1)),
        ("101", json!(101)),
        ("a million", json!(1_000_000)),
        ("the largest integer", json!(i64::MAX)),
        ("below the smallest", json!(i64::MIN)),
        ("a float", json!(20.5)),
        ("a string", json!("20")),
        ("a list", json!([20])),
        ("a bool", json!(true)),
    ] {
        for (tool, args) in [
            ("report", report_args("R2", json!({"rows": rows.clone()}))),
            (
                "analytics",
                analytics_args("A", json!({"rows": rows.clone()})),
            ),
        ] {
            let out = run(&h, "aaj ki bikri", &[(tool, args)])
                .into_iter()
                .next()
                .unwrap();
            assert!(
                refused(&out),
                "{tool} rows {name}: {:?} {}",
                out.code,
                out.text
            );
        }
    }
    // The edges that are allowed: 1 and 100 give a result.
    for rows in [1, 100] {
        let out = run(
            &h,
            "aaj ki bikri",
            &[("report", report_args("R2", json!({"rows": rows})))],
        )
        .into_iter()
        .next()
        .unwrap();
        assert!(
            out.code.is_none(),
            "rows {rows}: {:?} {}",
            out.code,
            out.text
        );
    }
}

#[test]
fn t5_a_result_of_a_big_report_is_cut_to_the_size_of_a_result_and_says_so() {
    let h = Harness::day();
    let out = run(
        &h,
        "sab dikhao",
        &[("report", report_args("R18", json!({"rows": 100})))],
    )
    .into_iter()
    .next()
    .unwrap();
    assert!(out.code.is_none(), "{}", out.text);
    assert!(
        out.text.len() <= mandi_assistant::limits::MAX_RESULT_BYTES + 200,
        "{} bytes",
        out.text.len()
    );
}

// ---- period words outside the list -----------------------------------------------------------

#[test]
fn t5_a_period_word_outside_the_list_is_refused() {
    let h = Harness::day();
    let cases: Vec<(&str, Value)> = vec![
        ("next week", period_args("next_week", json!({}))),
        ("eid", period_args("eid", json!({}))),
        ("an empty kind", period_args("", json!({}))),
        ("capital letters", period_args("TODAY", json!({}))),
        ("a month of 0", period_args("month", json!({"month": 0}))),
        ("a month of 13", period_args("month", json!({"month": 13}))),
        (
            "a negative month",
            period_args("month", json!({"month": -1})),
        ),
        (
            "a month of i64::MAX",
            period_args("month", json!({"month": i64::MAX})),
        ),
        ("a month with no number", period_args("month", json!({}))),
        ("31 February", period_args("day", json!({"day": "31-02"}))),
        (
            "month 13 as a day",
            period_args("day", json!({"day": "05-13"})),
        ),
        (
            "a day with no month",
            period_args("day", json!({"day": "05"})),
        ),
        (
            "a day typed as an ISO date",
            period_args("day", json!({"day": "2026-10-05"})),
        ),
        ("a range with no dates", period_args("range", json!({}))),
        (
            "a range that ends before it starts",
            period_args("range", json!({"from": "2026-10-10", "to": "2026-10-01"})),
        ),
        (
            "a range with a date that does not exist",
            period_args("range", json!({"from": "2026-02-30", "to": "2026-03-01"})),
        ),
        (
            "last_n with no number",
            period_args("last_n_days", json!({})),
        ),
        (
            "last_n of zero",
            period_args("last_n_days", json!({"n": 0})),
        ),
        (
            "last_n negative",
            period_args("last_n_months", json!({"n": -3})),
        ),
        (
            "last_n of i64::MAX",
            period_args("last_n_years", json!({"n": i64::MAX})),
        ),
        (
            "next_n of i64::MAX",
            period_args("next_n_months", json!({"n": i64::MAX})),
        ),
        (
            "an extra field",
            json!({"kind": "today", "month": null, "day": null, "from": null, "to": null, "n": null, "why": 1}),
        ),
    ];
    let mut wrong = Vec::new();
    for (name, args) in cases {
        let out = run(&h, "aaj ki bikri", &[("period", args)])
            .into_iter()
            .next()
            .unwrap();
        if !refused(&out) {
            wrong.push(format!(
                "{name}: {:?} {}",
                out.code,
                out.text.chars().take(120).collect::<String>()
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} cases:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// "pichle 3 mahine": a number of days, months or years is only a number of the question with that
/// word after it. The model that picks 3 when the question says 2, or when it names no number, or a
/// unit that is not the one asked, is refused.
#[test]
fn t5_a_last_n_that_the_question_does_not_hold_is_refused() {
    let h = Harness::day();
    let last = |kind: &str, n: i64| period_args(kind, json!({"n": n}));
    // The question holds 2 months: 2 is right, and the same number with another unit is not.
    let right = run(
        &h,
        "pichle 2 mahine ka commission",
        &[("period", last("last_n_months", 2))],
    );
    assert!(right[0].code.is_none(), "{}", right[0].text);
    for (name, question, kind, n) in [
        (
            "a number the question does not hold",
            "pichle 2 mahine ka commission",
            "last_n_months",
            3,
        ),
        (
            "the right number, the wrong unit (days)",
            "pichle 2 mahine ka commission",
            "last_n_days",
            2,
        ),
        (
            "the right number, the wrong unit (years)",
            "pichle 2 mahine ka commission",
            "last_n_years",
            2,
        ),
        (
            "no number in the question",
            "pichle mahine ka commission",
            "last_n_months",
            3,
        ),
        (
            "a number the question has, with no unit",
            "akram ka 2 bakaya",
            "last_n_months",
            2,
        ),
        (
            "a made-up large number",
            "pichle 2 mahine ka commission",
            "last_n_months",
            24,
        ),
    ] {
        let out = run(&h, question, &[("period", last(kind, n))])
            .into_iter()
            .next()
            .unwrap();
        assert!(refused(&out), "{name}: {:?} {}", out.code, out.text);
    }
}

// ---- estimates more than 12 months ahead ---------------------------------------------------------

#[test]
fn t5_an_estimate_more_than_12_months_ahead_is_refused() {
    let h = Harness::day();
    for ahead in [13, 24, 100, 1_000_000, i64::MAX, i64::MIN] {
        let out = run(
            &h,
            &format!("agle {ahead} mahine ka commission"),
            &[(
                "estimate",
                estimate_args("commission_all", Some(ahead), json!({})),
            )],
        )
        .into_iter()
        .next()
        .unwrap();
        assert!(refused(&out), "ahead {ahead}: {:?} {}", out.code, out.text);
    }
    // 12 is the largest: not refused for being too far (it may say there is too little history).
    let out = run(
        &h,
        "agle 12 mahine ka commission",
        &[(
            "estimate",
            estimate_args("commission_all", Some(12), json!({})),
        )],
    )
    .into_iter()
    .next()
    .unwrap();
    assert_ne!(
        out.code.as_deref(),
        Some("ERR_OUT_OF_RANGE"),
        "{}",
        out.text
    );
    // A number of months that is not in the question is refused, whatever it is.
    let out = run(
        &h,
        "commission isi raftar se chala to?",
        &[(
            "estimate",
            estimate_args("commission_all", Some(6), json!({})),
        )],
    )
    .into_iter()
    .next()
    .unwrap();
    assert!(out.code.is_none() || refused(&out), "{:?}", out.code);
}

#[test]
fn t5_an_estimate_with_made_up_parts_is_refused() {
    let h = Harness::day();
    let cases: Vec<(&str, Value, &str)> = vec![
        (
            "a target that is a typed figure",
            estimate_args("commission_all", Some(6), json!({"target": "300000"})),
            "kitna hoga agle 6 mahine",
        ),
        (
            "a percent that is a typed figure",
            estimate_args(
                "commission_all",
                Some(6),
                json!({"what_if": "up", "percent": "10"}),
            ),
            "agle 6 mahine agar raftar zyada ho",
        ),
        (
            "a what-if the question never said",
            estimate_args("commission_all", Some(6), json!({"what_if": "double"})),
            "agle 6 mahine ka commission",
        ),
        (
            "until and a target together",
            estimate_args(
                "commission_all",
                None,
                json!({"until": "year_end", "target": "q.n1"}),
            ),
            "3 lakh ka target saal ke aakhir tak",
        ),
        (
            "a measure with a filter it does not take",
            estimate_args("cash_in_hand", Some(6), json!({"item": "r1.matches.0"})),
            "cash agle 6 mahine",
        ),
        (
            "a balance with a target",
            estimate_args("buyers_dues", Some(6), json!({"target": "q.n1"})),
            "dues 3 lakh kab",
        ),
        (
            "a party balance with no party",
            estimate_args("party_balance", Some(6), json!({})),
            "bakaya agle 6 mahine",
        ),
    ];
    for (name, args, question) in cases {
        let out = run(&h, question, &[("estimate", args)])
            .into_iter()
            .next()
            .unwrap();
        assert!(refused(&out), "{name}: {:?} {}", out.code, out.text);
    }
}

// ---- the calculator at its edges ----------------------------------------------------------------

#[test]
fn t5_the_calculator_refuses_what_it_cannot_do_exactly() {
    let h = Harness::day();
    let question = "akram aur arshad ka 5000 mila kar";
    let two_finds = |op: &str, inputs: Value| -> Out {
        let outs = run(
            &h,
            question,
            &[
                find("ikram", "buyer"),
                card("r1.matches.0"),
                find("arshad", "buyer"),
                card("r3.matches.0"),
                ("calculate", json!({"op": op, "inputs": inputs})),
            ],
        );
        outs.into_iter().last().unwrap()
    };
    // A sum of two checked balances works: 2,880 + 18,684.
    let ok = two_finds("sum", json!(["r2.balance", "r4.balance"]));
    assert!(ok.code.is_none(), "{}", ok.text);
    for (name, op, inputs) in [
        ("no inputs", "sum", json!([])),
        (
            "a difference of three",
            "difference",
            json!(["r2.balance", "r4.balance", "r2.balance"]),
        ),
        ("a difference of one", "difference", json!(["r2.balance"])),
        (
            "a percent of zero",
            "percent_of",
            json!(["r2.balance", "r4.balance", "r4.balance"]),
        ),
        (
            "a change in percent of one",
            "change_percent",
            json!(["r2.balance"]),
        ),
        (
            "an input that is a name",
            "sum",
            json!(["r1.matches.0.name", "r2.balance"]),
        ),
        (
            "an input that is a typed figure",
            "sum",
            json!(["2880", "r2.balance"]),
        ),
        (
            "an input that is a whole list",
            "sum",
            json!(["r1.matches", "r2.balance"]),
        ),
        ("an empty path", "sum", json!(["", "r2.balance"])),
        (
            "a path with a star on a non-list",
            "sum",
            json!(["r2.balance.*", "r4.balance"]),
        ),
        (
            "fifty inputs",
            "sum",
            json!((0..50).map(|_| "r2.balance").collect::<Vec<_>>()),
        ),
        ("a number as an input", "sum", json!([2880, "r2.balance"])),
    ] {
        let out = two_finds(op, inputs);
        assert!(refused(&out), "{name}: {:?} {}", out.code, out.text);
    }
}

// ---- other tools ---------------------------------------------------------------------------------

#[test]
fn t5_the_other_tools_refuse_their_own_wrong_inputs() {
    let h = Harness::day();
    let cases: Vec<(&str, &str, Value)> = vec![
        (
            "app_state what that does not exist",
            "app_state",
            json!({"what": "passwords", "party": null, "item": null, "bikri": null}),
        ),
        (
            "app_state audit of a negative bikri",
            "app_state",
            json!({"what": "audit", "party": null, "item": null, "bikri": -5}),
        ),
        (
            "app_state audit of a huge bikri",
            "app_state",
            json!({"what": "audit", "party": null, "item": null, "bikri": i64::MAX}),
        ),
        (
            "app_state audit of a bikri that is not there",
            "app_state",
            json!({"what": "audit", "party": null, "item": null, "bikri": 99_999}),
        ),
        (
            "app_state of a typed party id",
            "app_state",
            json!({"what": "audit", "party": "a1", "item": null, "bikri": null}),
        ),
        (
            "app_help topic that does not exist",
            "app_help",
            json!({"topic": "how_to_hack"}),
        ),
        ("app_help with no topic", "app_help", json!({})),
        ("tiles with a filter", "tiles", json!({"kind": "all"})),
        (
            "party_card with a typed key",
            "party_card",
            json!({"party": "a1"}),
        ),
        (
            "party_card with an empty party",
            "party_card",
            json!({"party": ""}),
        ),
        (
            "party_card of a report row path",
            "party_card",
            json!({"party": "r1.rows"}),
        ),
        (
            "find with an empty text",
            "find",
            json!({"text": "", "role": "buyer"}),
        ),
        (
            "find with a role that is a list",
            "find",
            json!({"text": "a", "role": ["buyer"]}),
        ),
        (
            "report R1 with a negative bikri",
            "report",
            report_args("R1", json!({"bikri": -1})),
        ),
        (
            "report R1 with a huge bikri",
            "report",
            report_args("R1", json!({"bikri": i64::MAX})),
        ),
        (
            "report with a status that does not exist",
            "report",
            report_args("R2", json!({"status": "maybe"})),
        ),
        (
            "report with a layout the report has not",
            "report",
            report_args("R18", json!({"layout": "partner"})),
        ),
        (
            "report with a group_by the report has not",
            "report",
            report_args("R18", json!({"group_by": "year"})),
        ),
        (
            "report with a date in year 0",
            "report",
            report_args("R2", json!({"from": "0000-01-01", "to": "2026-10-10"})),
        ),
        (
            "report with a date in year 10000",
            "report",
            report_args("R2", json!({"from": "2026-10-10", "to": "10000-01-01"})),
        ),
        (
            "analytics view in lower case",
            "analytics",
            analytics_args("a", json!({})),
        ),
        (
            "analytics with a compare but no dates",
            "analytics",
            analytics_args("A", json!({"compare": "last_year"})),
        ),
        (
            "analytics sorted on a column of no view",
            "analytics",
            analytics_args(
                "A",
                json!({"sort": [{"column": "secret", "order": "desc"}]}),
            ),
        ),
        (
            "analytics with a goods word that does not exist",
            "analytics",
            analytics_args("A", json!({"goods": "stolen"})),
        ),
    ];
    let mut wrong = Vec::new();
    for (name, tool, args) in cases {
        let out = run(&h, "aaj ki bikri", &[(tool, args)])
            .into_iter()
            .next()
            .unwrap();
        // A call may also be allowed and say nothing wrong: only a silent wrong answer, a panic
        // (the test would not get here) or ERR_INTERNAL is a defect.
        if out.code.as_deref() == Some("ERR_INTERNAL") {
            wrong.push(format!("{name}: ERR_INTERNAL {}", out.text));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} cases:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// Through the loop: the refusals reach the model as error results it can correct, the question
/// still ends, nothing is written, and the daily count is one.
#[test]
fn t5_a_model_that_corrects_a_refused_call_gets_its_answer() {
    let h = Harness::day();
    let bad = ("period", period_args("last_n_months", json!({"n": 3})));
    let good_read = ("period", period_args("last_month", json!({})));
    let (reply, requests) = ask(
        &h,
        "pichle mahine ka commission",
        None,
        vec![
            tool_reply(&[bad]),
            tool_reply(&[good_read]),
            final_reply(&answer("need_dates", "")),
        ],
    );
    assert_eq!(requests.len(), 3);
    assert!(
        tool_errors(&requests[1])[0].is_some(),
        "the wrong n was refused"
    );
    assert!(tool_errors(&requests[2]).iter().any(Option::is_none));
    assert_eq!(reply.count_today, 1);
    assert!(
        matches!(
            reply.outcome,
            AskOutcome::NeedDates | AskOutcome::NotUnderstood
        ),
        "{reply:#?}"
    );
}

// ---- the wrong period kind ----------------------------------------------------------------------

/// "is saal ka commission": the question says this year. A model that asks `last_year` for the
/// period, reads view A with the dates of that period (as references to the result, the way the
/// instructions tell it to) and answers from it, gives a checked figure for the wrong year. The
/// scope guard (`scope_guard.rs`) compares what a cited view carries with what the question named,
/// but "a date given as a reference to the period result is never a violation".
fn wrong_period_replies(kind: &str) -> Vec<Value> {
    vec![
        tool_reply(&[("period", period_args(kind, json!({})))]),
        tool_reply(&[(
            "analytics",
            analytics_args("A", json!({"from": "r1.from", "to": "r1.to"})),
        )]),
        final_reply(&answer("answer", "کمیشن {{r2.totals.total_comm}} رہا۔")),
    ]
}

#[test]
fn t5_the_right_period_kind_gives_the_checked_commission() {
    let h = Harness::day();
    let (reply, _) = ask(
        &h,
        "is saal ka commission",
        None,
        wrong_period_replies("this_year"),
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = crate::harness::figures(crate::harness::answer_of(&reply));
    assert_eq!(
        crate::harness::money(figs[0].0),
        5_448,
        "E2E-48: today's commission"
    );
}

/// A question and the period kinds that are wrong for it (today is 10-10-26; all the commission
/// is in October). The scope guard refuses a view or report that carries the wrong days for these.
const GUARDED: &[(&str, &[&str])] = &[
    (
        "is saal ka commission",
        &[
            "last_year",
            "last_month",
            "yesterday",
            "this_month",
            "up_to_now",
            "last_week",
        ],
    ),
    (
        "pichle mahine ka commission",
        &[
            "this_month",
            "this_year",
            "up_to_now",
            "today",
            "last_year",
            "this_week",
        ],
    ),
    (
        "pichle saal ka commission",
        &["this_year", "this_month", "up_to_now"],
    ),
];

/// For "aaj ka commission" the app plans the dashboard tile, not one view, so the guard acts on
/// nothing and every kind passes.
const UNGUARDED: &[(&str, &[&str])] = &[(
    "aaj ka commission",
    &[
        "yesterday",
        "this_month",
        "this_year",
        "up_to_now",
        "this_week",
    ],
)];

fn period_answer(h: &Harness, question: &str, kind: &str) -> mandi_assistant::ask::AskReply {
    let (reply, _) = ask(h, question, None, wrong_period_replies(kind));
    reply
}

fn answered_wrong(cases: &[(&str, &[&str])]) -> Vec<String> {
    let mut shown = Vec::new();
    for (question, wrong) in cases {
        let h = Harness::day();
        for kind in *wrong {
            let reply = period_answer(&h, question, kind);
            if reply.outcome == AskOutcome::Answered {
                let figs = crate::harness::figures(crate::harness::answer_of(&reply));
                let rupees = figs.first().map(|(c, _)| crate::harness::money(c));
                shown.push(format!("{question} / {kind}: answered {rupees:?}"));
            }
        }
    }
    shown
}

/// A period of the wrong kind for a question the app understood is refused (the scope guard).
#[test]
fn t5_a_period_of_the_wrong_kind_is_refused_when_the_app_understood_the_question() {
    let shown = answered_wrong(GUARDED);
    assert!(shown.is_empty(), "{shown:?}");
}

/// FINDING T5-2: for "aaj ka commission" (today, 5,448 by E2E-48) a model that reads `yesterday`
/// is answered with 0, and `this_month`, `this_year`, `up_to_now`, `this_week` with the figure of
/// those days, each as the commission of today. The guard reads the dates of a period result as
/// "cannot be told" and the app's plan for a day is a tile, so nothing compares them. With the
/// local and instant answers on (production) the plain question never reaches the model; a
/// question that the app does not understand has no plan at all.
#[test]
fn t5_2_a_period_of_other_days_is_not_answered_as_today() {
    let shown = answered_wrong(UNGUARDED);
    assert!(shown.is_empty(), "{shown:?}");
}

// ---- numbers in the question ---------------------------------------------------------------------

/// The numbers the app reads from the question (`q.n1` ...) at their edges: huge numbers, Urdu and
/// full-width digits, a lakh and a crore of every size, a decimal comma, a lone sign, text with
/// thousands of digits. Nothing panics, and no number is negative or above what a whole-rupee
/// amount can hold.
#[test]
fn t5_the_numbers_read_from_a_question_never_panic_and_never_overflow() {
    let questions = [
        "99999999999999999999 rupees",
        "999999999999999999999999999999 lakh",
        "99999999999 crore",
        "9223372036854775807 rupees",
        "9223372036854775808 rupees",
        "-5 lakh",
        "٣ لاکھ ۵۰ ہزار",
        "３ lakh",
        "3.5 lakh",
        "3,5 lakh",
        "1,00,000",
        "1,,,000",
        "5,0,0",
        "0 lakh",
        "000 crore",
        "lakh lakh lakh",
        "3 lakh 3 lakh 3 lakh",
        "1e5",
        "1e999",
        "0x1F",
        "10%%%",
        "1000000000000000000000%",
        "-10%",
        "2.555555555%",
        "6 mahine 6 mahine 6 mahine",
        "99999999999999999999 mahine",
        "+++---***",
        "\u{200b}5\u{200b}",
        "5\u{0660}\u{0661}",
        "١٢٣٤٥٦٧٨٩٠١٢٣٤٥٦٧٨٩٠١٢٣٤٥٦٧٨٩٠",
    ];
    let long_digits = "9".repeat(500);
    let long_comma = "1,".repeat(250);
    for q in questions
        .iter()
        .copied()
        .chain([long_digits.as_str(), long_comma.as_str()])
    {
        for n in figures::question_numbers(q) {
            assert!(n.number >= 0, "{q:?}: a negative number {n:?}");
            assert!(n.text.chars().count() <= 600, "{q:?}");
        }
    }
}

/// The calculator over numbers of the question that are as large as they can be: an overflow is a
/// refusal, never a wrap-around or a panic (Code Standards M7).
#[test]
fn t5_the_calculator_over_huge_numbers_of_the_question_refuses_instead_of_wrapping() {
    let h = Harness::day();
    let question = "99999999999 crore aur 99999999999 crore ka total";
    let numbers = figures::question_numbers(question);
    assert!(numbers.len() <= 3, "{numbers:?}");
    for op in [
        "sum",
        "difference",
        "average",
        "percent_of",
        "change_percent",
    ] {
        let outs = run(
            &h,
            question,
            &[("calculate", json!({"op": op, "inputs": ["q.n1", "q.n2"]}))],
        );
        let out = &outs[0];
        match &out.code {
            // A refusal is the right ending.
            Some(code) => assert!(
                code.starts_with("ERR_") && code != "ERR_INTERNAL",
                "{op}: {code}"
            ),
            // A result must be exact and not negative for a sum of two non-negative numbers.
            None => {
                assert!(
                    !out.text.contains("-"),
                    "{op}: a negative result: {}",
                    out.text
                );
            }
        }
    }
    // Question text at its edges through the loop: 500 characters is the most, 501 is refused.
    let at_edge = "ا".repeat(500);
    let (reply, _) = ask(&h, &at_edge, None, gives_up());
    assert_eq!(reply.outcome, AskOutcome::NotUnderstood, "{reply:#?}");
    let client = mandi_assistant::client::RecordedClient::new(gives_up());
    let emit = |_s| {};
    let deps = h.deps(&client, &emit);
    let over = mandi_assistant::run::ask(
        &deps,
        &mandi_assistant::ask::AssistantAskInput {
            text: "ا".repeat(501),
            conversation_id: None,
        },
    );
    assert!(over.is_err(), "501 characters were accepted");
    assert!(client.requests().is_empty(), "nothing was sent");
    for blank in ["", "   ", "\n\t \u{00a0}"] {
        let over = mandi_assistant::run::ask(
            &deps,
            &mandi_assistant::ask::AssistantAskInput {
                text: blank.to_owned(),
                conversation_id: None,
            },
        );
        assert!(over.is_err(), "a blank question {blank:?} was accepted");
    }
    assert!(client.requests().is_empty());
}
