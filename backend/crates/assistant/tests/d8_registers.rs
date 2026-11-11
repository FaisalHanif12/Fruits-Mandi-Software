//! Plan 70, H6 (live run of 11-11-2026): "2 november se 5 november tak naqd aur udhar bikri mein kitna farq
//! hai". The model read R3 and R4 and took the difference of their value totals. A line paid in part is in
//! both registers (PRD FR-12.44, A97), so that gave credit 172,000 and cash 80,000 for a true credit 170,840
//! and cash 39,740 (difference 131,100).
//!
//! The tool side of the fix, with no model and no key:
//! - a read of R3 or R4 carries a note that says the two overlap and where paid and left are;
//! - `calculate` refuses a figure of R3 with a figure of R4, except the pair that never shares a line's
//!   money, the paid of R3 and the left of R4, and its refusal names the right figures;
//! - the right figures work: R5's paid and left totals, and R3's paid with R4's left, give 39,740, 170,840
//!   and 131,100.
//!
//! Every figure is also read from the screen's own function, never only typed here.
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
use mandi_services::domain::figures::QuestionNumber;
use serde_json::{Value, json};

const QUESTION: &str = "2 november se 5 november tak naqd aur udhar bikri mein kitna farq hai";

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
            today: BusinessDate::parse_iso("2026-11-08").unwrap(),
            partner_locked: false,
            numbers: &self.numbers,
            question: QUESTION,
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let draft = call(&env, &self.state, tool, &args.to_string())?;
        let prefix = if draft.tool == "calculate" { 'c' } else { 'r' };
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
        match self.run(tool, args) {
            Ok(id) => id,
            Err(Stop::Error(e)) => panic!("{tool} refused: {} {:?} {}", e.code, e.field, e.detail),
            Err(Stop::Locked) => panic!("{tool} locked"),
        }
    }

    fn refused(&mut self, tool: &str, args: Value) -> ToolError {
        match self.run(tool, args) {
            Err(Stop::Error(e)) => e,
            Err(Stop::Locked) => panic!("locked, not an error"),
            Ok(id) => panic!("{tool} should have been refused, gave {id}"),
        }
    }

    fn json(&self, id: &str) -> Value {
        tool_result_json(self.state.find(id).unwrap(), &self.words)
    }
}

fn report(id: &str) -> Value {
    json!({
        "id": id, "from": "r1.from", "to": "r1.to", "as_of": null, "party": null, "item": null,
        "city": null, "adda": null, "collector": null, "partner": null, "trip": null,
        "garden": null, "bikri": null, "group_by": null, "where": null, "sort": null, "rows": null
    })
}

fn calculate(op: &str, inputs: &[&str]) -> Value {
    json!({"op": op, "inputs": inputs})
}

/// The period of the question, `r1`, and the three reads, `r2` R3, `r3` R4 and `r4` R5.
fn read_three(s: &mut Session<'_>) {
    let period = s.ok(
        "period",
        json!({"kind": "range", "month": null, "day": null, "from": "2026-11-02", "to": "2026-11-05",
               "n": null}),
    );
    assert_eq!(period, "r1");
    assert_eq!(s.ok("report", report("R3")), "r2");
    assert_eq!(s.ok("report", report("R4")), "r3");
    assert_eq!(s.ok("report", report("R5")), "r4");
}

fn total(s: &Session<'_>, id: &str, column: &str) -> i64 {
    let v = &s.json(id)["totals"][column];
    v.get("rs")
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("{id} totals.{column}: {v}"))
}

#[test]
fn r3_and_r4_say_a_part_paid_line_is_in_both_and_the_refusal_names_the_right_figures() {
    let d = worked::d8();
    let mut s = Session::new(&d.w);
    read_three(&mut s);

    // The two registers' figures over the range: R3 is the lines with money paid, R4 the lines with money
    // left; Anas's and Akram's lines are in both.
    assert_eq!(total(&s, "r2", "paid"), 39_740);
    assert_eq!(total(&s, "r3", "left"), 170_840);
    assert_eq!(total(&s, "r4", "paid"), 39_740);
    assert_eq!(total(&s, "r4", "left"), 170_840);
    // The registers' buyer totals overlap: Anas's 48,100 and Akram's 10,680 are in both, so adding R3 and R4
    // is 58,780 more than R5's own total (210,580).
    let both = total(&s, "r2", "buyer_total") + total(&s, "r3", "buyer_total");
    assert_eq!(total(&s, "r4", "buyer_total"), 210_580);
    assert_eq!(both - 210_580, 48_100 + 10_680);

    // The note the model reads with each register.
    for id in ["r2", "r3"] {
        let note = s.json(id)["note"].as_str().unwrap_or_default().to_owned();
        assert!(
            note.contains("R3") && note.contains("R4") && note.contains("R5"),
            "{id}: {note}"
        );
    }
    // R5 carries no such note: its paid and left are the right figures.
    assert!(
        !s.json("r4")["note"]
            .as_str()
            .unwrap_or_default()
            .contains("R3 and R4")
    );

    // The mistake of the live run, in each way the model can make it: refused, with the way out.
    for (op, inputs) in [
        ("difference", ["r3.totals.value", "r2.totals.value"]),
        (
            "difference",
            ["r3.totals.buyer_total", "r2.totals.buyer_total"],
        ),
        ("sum", ["r3.totals.value", "r2.totals.value"]),
        ("difference", ["r3.totals.paid", "r2.totals.paid"]),
        ("difference", ["r3.totals.left", "r2.totals.left"]),
    ] {
        let e = s.refused("calculate", calculate(op, &inputs));
        assert_eq!(e.code, "ERR_OUT_OF_RANGE", "{op} {inputs:?}");
        assert!(
            e.detail.contains("R5") && e.detail.contains("paid") && e.detail.contains("left"),
            "{op} {inputs:?}: {}",
            e.detail
        );
    }
}

#[test]
fn the_figures_the_refusal_points_to_give_cash_credit_and_the_difference() {
    let d = worked::d8();
    let mut s = Session::new(&d.w);
    read_three(&mut s);
    // R5's own totals: paid and left of one read.
    let c1 = s.ok(
        "calculate",
        calculate("difference", &["r4.totals.left", "r4.totals.paid"]),
    );
    assert_eq!(s.json(&c1)["result"]["rs"], 131_100);
    // The paid of R3 with the left of R4 is the same pair, from the registers.
    let c2 = s.ok(
        "calculate",
        calculate("difference", &["r3.totals.left", "r2.totals.paid"]),
    );
    assert_eq!(s.json(&c2)["result"]["rs"], 131_100);
    // R5 on its own is never refused.
    let c3 = s.ok(
        "calculate",
        calculate("sum", &["r4.totals.left", "r4.totals.paid"]),
    );
    assert_eq!(s.json(&c3)["result"]["rs"], 210_580);
}

#[test]
fn a_range_says_it_has_no_label_and_a_month_has_one() {
    // Plan 70, H1: the model wrote `{{r1.label}}` for "December se aaj tak", the range of the read-ahead. Only a
    // month has a label, so the check refused the first answer and a second call of 8 seconds followed. The result
    // now says so, where the model reads it.
    let d = worked::d8();
    let mut s = Session::new(&d.w);
    let range = s.ok(
        "period",
        json!({"kind": "range", "month": null, "day": null, "from": "2025-12-01", "to": "2026-11-08",
               "n": null}),
    );
    let month = s.ok(
        "period",
        json!({"kind": "month", "month": 11, "day": null, "from": null, "to": null, "n": null}),
    );
    let range_note = s.json(&range)["note"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(
        range_note.contains("no `label`") && range_note.contains("`from`"),
        "{range_note}"
    );
    assert!(s.json(&range).get("label").is_none());
    assert!(s.json(&month).get("label").is_some());
    assert!(s.json(&month).get("note").is_none(), "{}", s.json(&month));
}

#[test]
fn the_pair_that_is_apart_is_refused_when_the_two_reads_have_other_dates() {
    // Money review of plan 70: R3 paid for 2 to 5 November less R4 left for 1 to 5 November is not the
    // difference of one thing. The pair is allowed only when both were read with the same filters.
    let d = worked::d8();
    let mut s = Session::new(&d.w);
    read_three(&mut s); // r1 period, r2 R3, r3 R4, r4 R5
    let wider = s.ok(
        "period",
        json!({"kind": "range", "month": null, "day": null, "from": "2026-11-01", "to": "2026-11-05",
               "n": null}),
    );
    assert_eq!(wider, "r5");
    let mut args = report("R3");
    args["from"] = json!("r5.from");
    args["to"] = json!("r5.to");
    assert_eq!(s.ok("report", args), "r6");
    let e = s.refused(
        "calculate",
        calculate("difference", &["r3.totals.left", "r6.totals.paid"]),
    );
    assert_eq!(e.code, "ERR_OUT_OF_RANGE");
    assert!(e.detail.contains("same dates"), "{}", e.detail);
    // The same dates are fine (r2 is R3 for 2 to 5 November, as r3 is R4).
    let ok = s.ok(
        "calculate",
        calculate("difference", &["r3.totals.left", "r2.totals.paid"]),
    );
    assert_eq!(s.json(&ok)["result"]["rs"], 131_100);
}
