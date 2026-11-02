//! What-ifs and the end of the year (plan 52 item 5, PRD FR-7.86, money agent design review). The
//! books are the E2E-59 history: April to September 2026 with commission 36,000 ... 51,000, today
//! 10-10-26, so the same pace is 48,000 a month. The model is a recorded transcript where a loop is
//! needed; the tool is called directly otherwise.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/harness.rs"]
mod harness;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use harness::{Harness, answer_of, figures, money};
use mandi_assistant::answer::EstimateMethodKind;
use mandi_assistant::ask::AskOutcome;
use mandi_assistant::results::{ToolResult, Words, fit, tool_result_json};
use mandi_assistant::tools::{Env, State, Stop, ToolError, call};
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::domain::figures::question_numbers;
use serde_json::{Value, json};

/// April to September 2026: the commission of each month is 36,000 ... 51,000.
const HISTORY: [(u32, i64, i64); 6] = [
    (4, 180, 1_600),
    (5, 150, 2_100),
    (6, 120, 2_850),
    (7, 225, 1_600),
    (8, 150, 2_600),
    (9, 173, 2_390),
];

struct Session<'a> {
    world: &'a world::World,
    words: Words,
    question: String,
    state: State,
}

impl<'a> Session<'a> {
    fn new(world: &'a world::World, question: &str) -> Self {
        Self {
            world,
            words: Words::new().unwrap(),
            question: question.to_owned(),
            state: State::default(),
        }
    }

    fn estimate(&mut self, extra: Value) -> Result<Value, ToolError> {
        let mut input = json!({
            "measure": "commission_all", "item": null, "seller": null, "buyer": null,
            "garden": null, "trip": null, "ahead": null, "until": null, "what_if": null,
            "percent": null, "target": null
        });
        for (k, v) in extra.as_object().unwrap() {
            input[k] = v.clone();
        }
        let numbers = question_numbers(&self.question);
        let env = Env {
            ctx: &self.world.env.ctx,
            actor: &self.world.owner,
            words: &self.words,
            language: Language::Ur,
            today: BusinessDate::parse_iso("2026-10-10").unwrap(),
            partner_locked: false,
            numbers: &numbers,
            question: &self.question,
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let draft = match call(&env, &self.state, "estimate", &input.to_string()) {
            Ok(draft) => draft,
            Err(Stop::Error(e)) => return Err(e),
            Err(Stop::Locked) => panic!("locked"),
        };
        let id = self.state.next_id('e');
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
        Ok(tool_result_json(self.state.find(&id).unwrap(), &self.words))
    }
}

fn rs(v: &Value) -> i64 {
    v["rs"].as_i64().unwrap_or_else(|| panic!("not money: {v}"))
}

#[test]
fn until_the_end_of_the_year_is_the_months_left_and_needs_the_owners_word() {
    let w = worked::history_world(&HISTORY);
    let mut s = Session::new(&w, "is raftar se saal ke aakhir tak commission kitna hoga");
    let v = s.estimate(json!({"until": "year_end"})).unwrap();
    assert_eq!(v["ahead_count"]["n"], 3, "October, November, December: {v}");
    assert_eq!(rs(&v["same_pace"]["total"]), 144_000);
    assert_eq!(rs(&v["same_growth"]["total"]), 171_000);
    assert!(v.get("what_if_pace").is_none(), "only when asked for");
    // The model's own number is refused with until, and the word must be in the question.
    let e = s
        .estimate(json!({"until": "year_end", "ahead": 3}))
        .unwrap_err();
    assert_eq!(e.code, "ERR_OUT_OF_RANGE");
    let w2 = worked::history_world(&HISTORY);
    let mut plain = Session::new(&w2, "agle mahine ka commission kitna hoga");
    let e = plain.estimate(json!({"until": "year_end"})).unwrap_err();
    assert!(e.detail.contains("does not say"), "{}", e.detail);
    // A months number and the end of the year together: ask the Owner.
    let mut both = Session::new(&w2, "agle 6 mahine ya saal ke aakhir tak commission");
    let e = both.estimate(json!({"until": "year_end"})).unwrap_err();
    assert!(e.detail.contains("which one"), "{}", e.detail);
}

#[test]
fn the_pace_doubles_and_the_factor_is_a_percent_the_model_can_cite() {
    let w = worked::history_world(&HISTORY);
    let mut s = Session::new(
        &w,
        "is raftar se saal ke aakhir tak commission kitna, aur agar raftar double ho to",
    );
    let v = s
        .estimate(json!({"until": "year_end", "what_if": "double"}))
        .unwrap();
    assert_eq!(rs(&v["same_pace"]["total"]), 144_000);
    assert_eq!(rs(&v["what_if_pace"]["per_month"]), 96_000);
    assert_eq!(rs(&v["what_if_pace"]["total"]), 288_000);
    assert_eq!(v["what_if_pace"]["factor"]["pct"], "200.00");
    // The months ahead carry the what-if beside the other methods.
    assert_eq!(rs(&v["ahead"][0]["what_if_pace"]), 96_000);
    // The word must be there, and only one kind of scenario word.
    let w2 = worked::history_world(&HISTORY);
    let mut none = Session::new(&w2, "agar ho to saal ke aakhir tak commission kitna");
    let e = none
        .estimate(json!({"until": "year_end", "what_if": "double"}))
        .unwrap_err();
    assert!(e.detail.contains("does not say double"), "{}", e.detail);
    let mut clash = Session::new(
        &w2,
        "agar raftar double ya half ho jaye to saal ke aakhir tak",
    );
    let e = clash
        .estimate(json!({"until": "year_end", "what_if": "double"}))
        .unwrap_err();
    assert!(e.detail.contains("more than one"), "{}", e.detail);
    // A past fact is not a scenario: no "agar", no what-if.
    let mut past = Session::new(
        &w2,
        "is saal commission double hua, saal ke aakhir tak kitna",
    );
    let e = past
        .estimate(json!({"until": "year_end", "what_if": "double"}))
        .unwrap_err();
    assert!(e.detail.contains("what if"), "{}", e.detail);
    // The end of the year in the question: the model may not send 6 months instead.
    let mut year = Session::new(&w2, "saal ke aakhir tak commission kitna hoga");
    let e = year.estimate(json!({"ahead": 6})).unwrap_err();
    assert!(e.detail.contains("until year_end"), "{}", e.detail);
    // A half, and a triple.
    let mut half = Session::new(&w2, "agar raftar aadhi reh jaye to agle 3 mahine");
    let v = half
        .estimate(json!({"ahead": 3, "what_if": "half"}))
        .unwrap();
    assert_eq!(rs(&v["what_if_pace"]["total"]), 72_000);
    let mut triple = Session::new(&w2, "agar raftar teen guna ho jaye to agle 3 mahine");
    let v = triple
        .estimate(json!({"ahead": 3, "what_if": "triple"}))
        .unwrap();
    assert_eq!(rs(&v["what_if_pace"]["total"]), 432_000);
}

#[test]
fn a_percent_up_or_down_needs_the_percent_of_the_question_and_its_word() {
    let w = worked::history_world(&HISTORY);
    // Up 10%: 52,800 a month, 158,400 in 3 months.
    let mut up = Session::new(
        &w,
        "agar raftar 10% zyada ho jaye to agle 3 mahine mein kitna",
    );
    let v = up
        .estimate(json!({"ahead": 3, "what_if": "up", "percent": "q.n1"}))
        .unwrap();
    assert_eq!(rs(&v["what_if_pace"]["per_month"]), 52_800);
    assert_eq!(rs(&v["what_if_pace"]["total"]), 158_400);
    assert_eq!(v["what_if_pace"]["factor"]["pct"], "110.00");
    // Down 25%: 36,000 a month, 108,000.
    let mut down = Session::new(
        &w,
        "agar raftar 25% kam ho jaye to agle 3 mahine mein kitna",
    );
    let v = down
        .estimate(json!({"ahead": 3, "what_if": "down", "percent": "q.n1"}))
        .unwrap();
    assert_eq!(rs(&v["what_if_pace"]["total"]), 108_000);
    // The wrong direction word, no word, both words: refused, the Owner is asked.
    for (question, what_if) in [
        ("agar raftar 10% kam ho jaye to agle 3 mahine", "up"),
        ("agar raftar 10% ho jaye to agle 3 mahine", "up"),
        ("agar raftar 10% zyada ya kam ho to agle 3 mahine", "up"),
        ("agar raftar 10% zyada ho jaye to agle 3 mahine", "down"),
    ] {
        let mut s = Session::new(&w, question);
        let e = s
            .estimate(json!({"ahead": 3, "what_if": what_if, "percent": "q.n1"}))
            .unwrap_err();
        assert_eq!(e.code, "ERR_OUT_OF_RANGE", "{question}: {}", e.detail);
    }
    // The percent must be a number of the question with a % sign; 100% down and a typed percent
    // the model made up are refused.
    let mut s = Session::new(&w, "agar raftar 100% kam ho jaye to agle 3 mahine");
    assert!(
        s.estimate(json!({"ahead": 3, "what_if": "down", "percent": "q.n1"}))
            .is_err()
    );
    let mut s = Session::new(&w, "agar raftar zyada ho jaye to agle 3 mahine");
    assert!(
        s.estimate(json!({"ahead": 3, "what_if": "up", "percent": "q.n1"}))
            .is_err()
    );
    // A percent with a scenario word that takes none.
    let mut s = Session::new(&w, "agar raftar double ho, 10% zyada, agle 3 mahine");
    assert!(
        s.estimate(json!({"ahead": 3, "what_if": "double", "percent": "q.n1"}))
            .is_err()
    );
}

#[test]
fn a_target_works_on_the_what_if_line() {
    let w = worked::history_world(&HISTORY);
    // 5,00,000 with the doubled pace is reached in the 6th month: March 2027. Same pace: August.
    let mut s = Session::new(
        &w,
        "agar raftar double ho to 5 lakh ka target kab tak pura hoga",
    );
    let v = s
        .estimate(json!({"what_if": "double", "target": "q.n1", "ahead": 12}))
        .unwrap();
    assert_eq!(v["what_if_pace"]["target_month"]["month"], "2027-03", "{v}");
    assert_eq!(v["same_pace"]["target_month"]["month"], "2027-08");
    assert_eq!(
        v["what_if_pace"]["total"]["rs"], 1_152_000,
        "12 months of 96,000"
    );
}

#[test]
fn the_answer_cites_the_factor_and_shows_the_what_if_under_the_estimate_band() {
    let w = worked::history_world(&HISTORY);
    let h = Harness::over(w);
    let usage = json!({
        "prompt_tokens": 7_800, "completion_tokens": 90,
        "prompt_tokens_details": {"cached_tokens": 7_600},
        "completion_tokens_details": {"reasoning_tokens": 20}
    });
    let tool = json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null,
            "tool_calls": [{"id": "call_0", "type": "function", "function": {
                "name": "estimate",
                "arguments": json!({
                    "measure": "commission_all", "item": null, "seller": null, "buyer": null,
                    "garden": null, "trip": null, "ahead": null, "until": "year_end",
                    "what_if": "double", "percent": null, "target": null
                }).to_string()}}]}}],
        "usage": usage,
    });
    // The model cannot write "double": the factor is a reference, shown as 200.00%.
    let text = "{{e1.first}} سے {{e1.last}} تک اسی رفتار سے {{e1.same_pace.total}} بنے گا۔ اگر رفتار {{e1.what_if_pace.factor}} ہو تو {{e1.what_if_pace.total}}۔";
    let answer = json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "estimate", "text": text, "show": [{"result": "e1", "as": "chart", "rows": null}],
            "next": [], "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage,
    });
    let (reply, _) = h.ask(
        "is raftar se saal ke aakhir tak commission kitna, aur agar raftar double ho to",
        None,
        vec![tool, answer],
    );
    assert_eq!(reply.outcome, AskOutcome::Estimate, "{reply:#?}");
    let a = answer_of(&reply);
    let estimate = a.estimate.as_ref().expect("the estimate band");
    let kinds: Vec<EstimateMethodKind> = estimate.methods.iter().map(|m| m.kind).collect();
    assert!(kinds.contains(&EstimateMethodKind::WhatIfPace), "{kinds:?}");
    let figs = figures(a);
    assert_eq!(money(figs[0].0), 144_000);
    assert_eq!(money(figs[2].0), 288_000);
    // A typed "double" in the model's text is refused (FR-7.76).
    h.scan_requests();
}
