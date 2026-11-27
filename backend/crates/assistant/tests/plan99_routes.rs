//! Plan 99 A1 (PRD v1.60, FR-7.87): the direct reads of the assistant, on the PRD worked day
//! (Events 1 to 5, 10-10-26) and the E2E-59 history. The tools are called as the loop calls them; no
//! model and no network.
//!
//! - `period` on `report`, `analytics` and `estimate` gives the same figures as the dates it stands
//!   for, and the result carries the period to cite.
//! - R1 and R5 with `bikri: "last"` read the latest بکری (بکری 2 of 10-10-26).
//! - R13 with `layout: "bikri"` gives one row per بکری or entry, as پارٹی کھاتے.
//! - A view with no dates reads all the books; a comparison with one date is filled, never refused.
//! - The pre-reads of "pichli bikri", a garden and a market trip with no name.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use mandi_assistant::results::{Node, ToolResult, Val, Words, fit, tool_result_json};
use mandi_assistant::tools::{Env, State, Stop, ToolError, call};
use mandi_assistant::understand::{Reader, plan_pre_reads, plan_scope_reads, understand};
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::domain::figures::{NumberUnit, QuestionNumber, question_numbers};
use serde_json::{Value, json};

struct Session<'a> {
    world: &'a world::World,
    words: Words,
    numbers: Vec<QuestionNumber>,
    question: &'static str,
    state: State,
}

impl<'a> Session<'a> {
    fn new(world: &'a world::World) -> Self {
        Self {
            world,
            words: Words::new().unwrap(),
            numbers: Vec::new(),
            question: "",
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
            question: self.question,
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

    fn ok(&mut self, tool: &str, args: Value) -> String {
        self.run(tool, args.clone())
            .unwrap_or_else(|e| panic!("{tool} {args}: {e:?}"))
    }

    fn error(&mut self, tool: &str, args: Value) -> ToolError {
        match self.run(tool, args.clone()) {
            Err(Stop::Error(e)) => e,
            other => panic!("{tool} {args} should be refused: {other:?}"),
        }
    }

    fn json(&self, id: &str) -> Value {
        tool_result_json(self.state.find(id).unwrap(), &self.words)
    }

    fn input(&self, id: &str) -> &Value {
        &self.state.find(id).unwrap().input
    }

    /// The rupees of a money value at `path`.
    fn rs(&self, id: &str, path: &str) -> i64 {
        let parts: Vec<&str> = path.split('.').collect();
        match self.state.find(id).unwrap().node.get(&parts) {
            Some(Node::Val(Val::Money(m))) => m.paisa.rupees(),
            other => panic!("{id}.{path} is not money: {other:?}\n{}", self.json(id)),
        }
    }

    fn find_party(&mut self, text: &str, role: &str) -> String {
        let id = self.ok("find", json!({"text": text, "role": role}));
        format!("{id}.matches.0")
    }
}

fn rows_of(s: &Session<'_>, id: &str) -> Vec<(String, Node)> {
    match s.state.find(id).unwrap().node.get(&["rows"]) {
        Some(Node::Keyed(rows)) => rows.clone(),
        other => panic!("no rows: {other:?}"),
    }
}

fn day_world() -> world::World {
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    w
}

// ---- the period word ----------------------------------------------------------------------------

#[test]
fn a_period_word_reads_the_same_figures_as_its_dates_and_is_citable() {
    let w = day_world();
    let mut s = Session::new(&w);
    let by_word = s.ok(
        "report",
        json!({"id": "R2", "from": null, "to": null, "period": "this_month"}),
    );
    let by_dates = s.ok(
        "report",
        json!({"id": "R2", "from": "2026-10-01", "to": "2026-10-10"}),
    );
    // خام بکری 41,400 on both (PRD 4.9).
    assert_eq!(
        s.rs(&by_word, "totals.value"),
        s.rs(&by_dates, "totals.value")
    );
    let json = s.json(&by_word);
    assert_eq!(json["period"]["label"]["month"], "2026-10", "{json}");
    assert_eq!(json["period"]["from"]["date"], "2026-10-01");
    assert_eq!(json["period"]["to"]["date"], "2026-10-10");
    // The guards and the button read the days.
    assert_eq!(s.input(&by_word)["from"], "2026-10-01");
    assert_eq!(s.input(&by_word)["to"], "2026-10-10");
    // The expense report of last month (September) has nothing; this month has بجلی 3,000.
    let last = s.ok("report", json!({"id": "R29", "period": "last_month"}));
    let this = s.ok("report", json!({"id": "R29", "period": "this_month"}));
    assert_eq!(s.input(&last)["to"], "2026-09-30");
    assert!(s.json(&this).to_string().contains("3000"));
}

#[test]
fn a_balance_list_reads_a_period_as_of_its_last_day_and_r35_takes_one_day_only() {
    let w = day_world();
    let mut s = Session::new(&w);
    let id = s.ok("report", json!({"id": "R20", "period": "last_month"}));
    assert_eq!(s.input(&id)["as_of"], "2026-09-30");
    assert!(s.input(&id)["from"].is_null());
    let party = s.find_party("akram", "buyer");
    let today = s.ok(
        "report",
        json!({"id": "R35", "party": party, "period": "today"}),
    );
    assert_eq!(s.input(&today)["as_of"], "2026-10-10");
    let week = s.error(
        "report",
        json!({"id": "R35", "party": party, "period": "this_week"}),
    );
    assert_eq!(week.field.as_deref(), Some("period"));
}

#[test]
fn a_wrong_period_word_or_a_word_with_dates_is_refused_and_taught() {
    let w = day_world();
    let mut s = Session::new(&w);
    let wrong = s.error("report", json!({"id": "R2", "period": "eid"}));
    assert_eq!(wrong.field.as_deref(), Some("period"));
    assert!(wrong.allowed.iter().any(|w| w == "this_month"), "{wrong:?}");
    let both = s.error(
        "report",
        json!({"id": "R2", "period": "today", "from": "2026-10-01"}),
    );
    assert_eq!(both.field.as_deref(), Some("period"));
    // last_days needs the question to hold the number with days.
    let none = s.error("analytics", json!({"view": "A", "period": "last_days:10"}));
    assert_eq!(none.field.as_deref(), Some("period"));
    s.numbers = question_numbers("pichle 10 din ki bikri");
    s.ok("analytics", json!({"view": "A", "period": "last_days:10"}));
}

#[test]
fn month_and_day_words_are_the_latest_that_have_started() {
    let w = day_world();
    let mut s = Session::new(&w);
    let september = s.ok("report", json!({"id": "R2", "period": "month:9"}));
    assert_eq!(s.input(&september)["from"], "2026-09-01");
    assert_eq!(s.input(&september)["to"], "2026-09-30");
    let day = s.ok("report", json!({"id": "R2", "period": "day:10-10"}));
    assert_eq!(s.input(&day)["from"], "2026-10-10");
    let all = s.ok("report", json!({"id": "R2", "period": "up_to_now"}));
    // Up to now starts at the first day of the books (the opening of 31-08-26).
    assert_eq!(s.input(&all)["from"], "2026-08-31");
}

// ---- the latest sale ----------------------------------------------------------------------------

#[test]
fn bikri_last_reads_the_latest_sale_and_names_it() {
    let w = day_world();
    let mut s = Session::new(&w);
    let r1 = s.ok("report", json!({"id": "R1", "bikri": "last"}));
    // بکری 2 (کریم، سیب): gross 12,000, deductions 1,840, net 10,160 (PRD 4.5).
    assert_eq!(s.rs(&r1, "totals.gross"), 12_000);
    assert_eq!(s.rs(&r1, "totals.deductions"), 1_840);
    assert_eq!(s.rs(&r1, "totals.net"), 10_160);
    let json = s.json(&r1);
    assert_eq!(json["latest_bikri"]["date"]["date"], "2026-10-10", "{json}");
    assert_eq!(s.input(&r1)["bikri"], 2);
    // Its buyers: the walk-in buyer شاہد with his 12,800, paid on the spot; the whole بکری is
    // read, with no date filter.
    let r5 = s.ok("report", json!({"id": "R5", "bikri": "last"}));
    assert_eq!(s.input(&r5)["bikri"], 2);
    assert!(s.input(&r5)["from"].is_null(), "{}", s.input(&r5));
    assert_eq!(s.rs(&r5, "totals.buyer_total"), 12_800);
    assert_eq!(s.rs(&r5, "totals.paid"), 12_800);
    // A day with no sale says so.
    let none = s.error(
        "report",
        json!({"id": "R1", "bikri": "last", "period": "yesterday"}),
    );
    assert_eq!(none.code, "ERR_NOT_FOUND");
    // R1 given a day and no number is that day's latest.
    let day = s.ok("report", json!({"id": "R1", "period": "today"}));
    assert_eq!(s.rs(&day, "totals.net"), 10_160);
    // A number still works, as a number or as text.
    let one = s.ok("report", json!({"id": "R1", "bikri": "1"}));
    assert_eq!(s.rs(&one, "totals.net"), 19_316);
}

// ---- one row for each sale ----------------------------------------------------------------------

#[test]
fn r13_layout_bikri_is_one_row_per_sale_with_the_checked_closing() {
    let w = day_world();
    let mut s = Session::new(&w);
    let party = s.find_party("akram", "buyer");
    let id = s.ok(
        "report",
        json!({"id": "R13", "party": party, "layout": "bikri"}),
    );
    let rows = rows_of(&s, &id);
    assert_eq!(rows.len(), 2, "{}", s.json(&id));
    // Newest first: the receipt, then بکری 1 (E2E-33).
    let first = &rows[0].1;
    assert!(
        matches!(first.get(&["credit"]), Some(Node::Val(Val::Money(m))) if m.paisa.rupees() == 10_000)
    );
    assert!(
        matches!(first.get(&["balance"]), Some(Node::Val(Val::Money(m))) if m.paisa.rupees() == 2_880)
    );
    let second = &rows[1].1;
    assert!(
        matches!(second.get(&["debit"]), Some(Node::Val(Val::Money(m))) if m.paisa.rupees() == 12_880)
    );
    assert_eq!(s.rs(&id, "totals.debit"), 12_880);
    assert_eq!(s.rs(&id, "totals.credit"), 10_000);
    let json = s.json(&id);
    assert_eq!(json["summary"]["closing"]["rs"], 2_880, "{json}");
    assert_eq!(json["summary"]["closing"]["checked"], true, "{json}");
    assert_eq!(json["summary"]["closing"]["side"], "بنام");

    // رحیم, a seller: پیشگی, بکری 1 and the payment; settled at 0.
    let seller = s.find_party("rahim", "seller");
    let id = s.ok(
        "report",
        json!({"id": "R13", "party": seller, "layout": "bikri"}),
    );
    assert_eq!(rows_of(&s, &id).len(), 3, "{}", s.json(&id));
    assert_eq!(s.rs(&id, "summary.closing"), 0);

    // It is read as it is: a sort or an item is refused.
    let sorted = s.error(
        "report",
        json!({"id": "R13", "party": party, "layout": "bikri",
               "sort": [{"column": "date", "order": "asc"}]}),
    );
    assert_eq!(sorted.field.as_deref(), Some("sort"));
}

// ---- analytics with no dates --------------------------------------------------------------------

/// A view with no dates reads all the books, exactly as a report with no dates does, so one answer
/// never mixes two periods (lead, 27-11-2026; the gross and the net of the whole books stay side by
/// side, `plan82_measure.rs`). Only a comparison is given its missing dates.
#[test]
fn a_view_with_no_dates_reads_all_the_books() {
    let w = day_world();
    let mut s = Session::new(&w);
    let id = s.ok("analytics", json!({"view": "A"}));
    assert!(s.input(&id)["from"].is_null(), "{}", s.input(&id));
    assert!(s.json(&id).get("period").is_none());
    assert_eq!(s.rs(&id, "totals.total_comm"), 5_448);
}

#[test]
fn a_comparison_with_one_date_only_is_read_and_not_refused() {
    let w = day_world();
    let mut s = Session::new(&w);
    let id = s.ok(
        "analytics",
        json!({"view": "A", "to": "2026-10-10", "compare": "previous"}),
    );
    assert_eq!(s.input(&id)["from"], "2026-01-01", "{}", s.json(&id));
    let id = s.ok(
        "analytics",
        json!({"view": "A", "from": "2026-10-01", "compare": "previous"}),
    );
    assert_eq!(s.input(&id)["to"], "2026-10-10");
}

/// A group word typed as one item ("fruit", logged question of 27-11-2026) reads the group.
#[test]
fn a_group_word_typed_as_an_item_reads_the_item_group() {
    let w = day_world();
    let mut s = Session::new(&w);
    let env = Env {
        ctx: &w.env.ctx,
        actor: &w.owner,
        words: &s.words,
        language: Language::Ur,
        today: BusinessDate::parse_iso("2026-10-10").unwrap(),
        partner_locked: false,
        numbers: &s.numbers,
        question: "",
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };
    let (mut logged, mut asked) = (Vec::new(), None);
    let resolved = mandi_assistant::tools::typed::resolve(
        &env,
        &mut s.state,
        0,
        "analytics",
        &json!({"view": "A", "item": "fruit", "item_group": null}).to_string(),
        &mut logged,
        &mut asked,
    )
    .unwrap();
    assert!(asked.is_none() && logged.is_empty());
    let args: Value = serde_json::from_str(&resolved.arguments).unwrap();
    assert!(args["item"].is_null());
    assert_eq!(args["item_group"], "fruit");
    let id = s.ok("analytics", args);
    assert_eq!(s.input(&id)["item_group"], "fruit");
}

/// A question about a partner's خالص never gets view F's net change, which does not take off the
/// money he took (live check NFR-AI-2 Q055: 996 shown for −1,004).
#[test]
fn a_khaalis_question_gets_no_net_change_from_view_f() {
    let w = day_world();
    for (question, kept) in [
        ("alif ka khaalis kitna hai", false),
        ("الف کا خالص کتنا ہے", false),
        ("partner ka hissa batao", true),
    ] {
        let mut s = Session::new(&w);
        s.question = question;
        let id = s.ok("analytics", json!({"view": "F"}));
        let text = s.json(&id).to_string();
        assert_eq!(text.contains("net_change\":"), kept, "{question}: {text}");
        assert!(text.contains("profit_share"), "{text}");
    }
}

/// "What percent of the dues" is never answered with the plain balance (live check Q071).
#[test]
fn a_percent_of_the_dues_gets_no_instant_balance() {
    let w = day_world();
    let reader = Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language: Language::Ur,
        today: BusinessDate::parse_iso("2026-10-10").unwrap(),
        partner_locked: false,
    };
    for question in [
        "what percent of the total dues does arshad owe",
        "arshad ka baqaya kul ugrahi ka kitne fisad hai",
    ] {
        assert!(understand(&reader, question).instant_stop, "{question}");
    }
    assert!(!understand(&reader, "arshad ka bakaya kitna hai").instant_stop);
}

// ---- the estimate period ------------------------------------------------------------------------

#[test]
fn an_estimate_takes_a_period_word_for_how_far() {
    let w = worked::history_world(&[
        (4, 180, 1_600),
        (5, 150, 2_100),
        (6, 120, 2_850),
        (7, 225, 1_600),
        (8, 150, 2_600),
        (9, 173, 2_390),
    ]);
    let mut s = Session::new(&w);
    s.numbers = question_numbers("agle 6 mahine mein commission kitna hoga");
    let base = json!({"measure": "commission_all", "item": null, "seller": null, "buyer": null,
                      "garden": null, "trip": null, "ahead": null, "until": null,
                      "what_if": null, "percent": null, "target": null, "head": null,
                      "partner": null, "party": null});
    let mut by_word = base.clone();
    by_word["period"] = json!("next_months:6");
    let mut by_ahead = base.clone();
    by_ahead["ahead"] = json!(6);
    let a = s.ok("estimate", by_word.clone());
    let b = s.ok("estimate", by_ahead);
    assert_eq!(
        s.json(&a)["same_pace"]["total"],
        s.json(&b)["same_pace"]["total"]
    );
    by_word["ahead"] = json!(6);
    assert_eq!(
        s.error("estimate", by_word).field.as_deref(),
        Some("period")
    );
}

// ---- the pre-reads ------------------------------------------------------------------------------

fn pre_reads(w: &world::World, question: &str) -> Vec<(String, Value)> {
    let reader = Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language: Language::Ur,
        today: BusinessDate::parse_iso("2026-10-10").unwrap(),
        partner_locked: false,
    };
    let u = understand(&reader, question);
    // As the app does: the whole plan when it is sure, else only the pre-reads safe all the same.
    let plan = if u.sure {
        plan_pre_reads(&u)
    } else {
        plan_scope_reads(&u)
    };
    plan.unwrap_or_default()
        .into_iter()
        .map(|r| (r.tool.to_owned(), r.input))
        .collect()
}

fn named(reads: &[(String, Value)]) -> Vec<String> {
    reads
        .iter()
        .filter_map(|(tool, input)| match tool.as_str() {
            "report" => input["id"].as_str().map(str::to_owned),
            "analytics" => input["view"].as_str().map(|v| format!("view {v}")),
            _ => None,
        })
        .collect()
}

#[test]
fn the_last_sale_is_read_ahead() {
    let w = day_world();
    for question in [
        "pichli bikri ki details batao",
        "pichli bikri ki details batao ?",
        "aakhri bikri ki tafseel",
        "last sale details",
        "پچھلی بکری کی تفصیل بتائیں",
        "akhri bikri dikhao",
    ] {
        let reads = pre_reads(&w, question);
        assert_eq!(named(&reads), ["R1", "R5"], "{question}: {reads:?}");
        assert!(
            reads
                .iter()
                .filter(|(t, _)| t == "report")
                .all(|(_, i)| i["bikri"] == "last"),
            "{question}: {reads:?}"
        );
    }
    // A rate, or a period, is not the last sale.
    for question in [
        "kele ka aakhri rate",
        "pichle mahine ki bikri",
        "akram ki aakhri bikri",
    ] {
        let reads = pre_reads(&w, question);
        assert!(
            !reads.iter().any(|(_, i)| i["bikri"] == "last"),
            "{question}: {reads:?}"
        );
    }
}

#[test]
fn a_garden_or_a_trip_with_no_name_is_read_ahead() {
    let w = day_world();
    for question in [
        "bagh ka hisaab batao",
        "bagh ka munafa kitna hai",
        "باغ کا حساب بتائیں",
    ] {
        let names = named(&pre_reads(&w, question));
        assert!(
            names.contains(&"R36".to_owned()) && names.contains(&"view I".to_owned()),
            "{question}: {names:?}"
        );
    }
    for question in [
        "market safar ka hisaab batao",
        "trip ka munafa kitna hai",
        "apna maal ka hisaab batao",
    ] {
        let names = named(&pre_reads(&w, question));
        assert!(
            names.contains(&"R33".to_owned()) && names.contains(&"view J".to_owned()),
            "{question}: {names:?}"
        );
    }
    // A thing to be explained reads the guide, never the books (logged question of 27-11-2026).
    for question in [
        "is main market safar kiya hai iski mujhay samaj nahi aa rahi ?",
        "bagh ka matlab samjhao",
        "market safar kaise banate hain",
        "باغ کیا ہوتا ہے سمجھ نہیں آئی",
    ] {
        let reads = pre_reads(&w, question);
        assert!(reads.is_empty(), "{question}: {reads:?}");
    }
    // The reads run on the books.
    let mut s = Session::new(&w);
    for (tool, input) in pre_reads(&w, "bagh ka munafa kitna hai") {
        s.ok(&tool, input);
    }
}

#[test]
fn the_numbers_of_a_question_are_unchanged() {
    // A guard for the helper used above: "10 din" is days.
    let numbers = question_numbers("pichle 10 din");
    assert_eq!(numbers.first().map(|n| n.unit), Some(NumberUnit::Days));
}
