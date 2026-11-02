//! The playbook of `system.md` and the failure list of plan 51 P51-B2, run on the tools.
//!
//! Each line of the playbook ("asked → read → advise") is sent as the model would send it, and every
//! call must give a result and not a refusal. Then the first tries a model makes by mistake (a word
//! that is no column, a comparison with no period, a filter the view does not have, a typed
//! number) must be refused with a reason and the allowed words, and the retry with one of those
//! words must work, so the question ends with an answer, or with a listed refusal and a clear ending
//! (`loop_stop.rs`), and never with "not understood" after two wasted rounds.
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
use mandi_services::core::partner_code::{PartnerCodeSetInput, PartnerCodeUnlockInput};
use mandi_services::domain::figures::{NumberUnit, QuestionNumber, question_numbers};
use mandi_services::partner_code;
use serde_json::{Value, json};

struct Session<'a> {
    world: &'a world::World,
    words: Words,
    numbers: Vec<QuestionNumber>,
    state: State,
    today: &'static str,
}

impl<'a> Session<'a> {
    fn new(world: &'a world::World, today: &'static str, question: &str) -> Self {
        Self {
            world,
            words: Words::new().unwrap(),
            numbers: question_numbers(question),
            state: State::default(),
            today,
        }
    }

    fn try_run(&mut self, tool: &str, args: Value) -> Result<String, ToolError> {
        let env = Env {
            ctx: &self.world.env.ctx,
            actor: &self.world.owner,
            words: &self.words,
            language: Language::Ur,
            today: BusinessDate::parse_iso(self.today).unwrap(),
            partner_locked: false,
            numbers: &self.numbers,
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let draft = match call(&env, &self.state, tool, &args.to_string()) {
            Ok(draft) => draft,
            Err(Stop::Error(e)) => return Err(e),
            Err(Stop::Locked) => panic!("{tool}: the partner section is locked"),
        };
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

    /// A call of the playbook: it must give a result.
    fn ok(&mut self, tool: &str, args: Value) -> String {
        match self.try_run(tool, args.clone()) {
            Ok(id) => id,
            Err(e) => panic!("{tool} {args} was refused: {}", e.to_json()),
        }
    }

    fn json(&self, id: &str) -> Value {
        tool_result_json(self.state.find(id).unwrap(), &self.words)
    }

    fn period(&mut self, kind: &str) -> String {
        self.ok(
            "period",
            json!({"kind": kind, "month": null, "day": null, "from": null, "to": null, "n": null}),
        )
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

fn unlocked_day() -> world::World {
    let w = day();
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
    w
}

fn sort(column: &str, order: &str) -> Value {
    json!([{"column": column, "order": order}])
}

// ---- the playbook -----------------------------------------------------------------------------

#[test]
fn playbook_1_kis_se_pehle_wasooli_karun() {
    let w = day();
    let mut s = Session::new(&w, "2026-10-10", "kis se pehle wasooli karun");
    let r20 = s.ok(
        "report",
        report(
            "R20",
            json!({"sort": [{"column": "days", "order": "desc"}, {"column": "balance", "order": "desc"}], "rows": 10}),
        ),
    );
    assert_eq!(s.json(&r20)["count"]["n"], 2);
    s.period("this_year");
    s.ok(
        "analytics",
        analytics(
            "C",
            json!({"from": "r2.from", "to": "r2.to", "sort": sort("dues_now", "desc"), "rows": 10}),
        ),
    );
}

#[test]
fn playbook_2_akram_ko_aur_udhar_dun() {
    let w = day();
    let mut s = Session::new(&w, "2026-10-10", "akram ko aur udhar dun");
    let find = s.ok("find", json!({"text": "akram", "role": "buyer"}));
    let party = format!("{find}.matches.0");
    s.ok("party_card", json!({"party": party}));
    s.period("this_year");
    s.ok(
        "analytics",
        analytics(
            "C",
            json!({"from": "r3.from", "to": "r3.to", "buyer": party}),
        ),
    );
    s.ok("report", report("R13", json!({"party": party})));
}

#[test]
fn playbook_3_kis_ko_pehle_paise_dun() {
    let w = day();
    let mut s = Session::new(
        &w,
        "2026-10-10",
        "kis ko pehle paise dun, aaj kitna de sakta hun",
    );
    let r24 = s.ok(
        "report",
        report("R24", json!({"sort": sort("payable", "desc")})),
    );
    let tiles = s.ok("tiles", json!({}));
    // The shortfall or what is left: cash minus what is payable, by the calculator.
    let payable = s.json(&r24)["totals"]["payable"].clone();
    assert!(payable["rs"].as_i64().unwrap() > 0, "{payable}");
    s.ok(
        "calculate",
        json!({"op": "difference", "inputs": [format!("{tiles}.cash_in_hand.amount"), format!("{r24}.totals.payable")]}),
    );
}

#[test]
fn playbook_4_cash_kam_kyun() {
    let w = day();
    let mut s = Session::new(&w, "2026-10-10", "cash kam kyun");
    s.period("this_month");
    s.ok(
        "report",
        report("R12", json!({"from": "r1.from", "to": "r1.to"})),
    );
    s.ok(
        "report",
        report("R29", json!({"from": "r1.from", "to": "r1.to"})),
    );
}

#[test]
fn playbook_5_commission_kaise_barhe() {
    let w = day();
    let mut s = Session::new(
        &w,
        "2026-10-10",
        "commission kaise barhe, kaun sa maal faida deta",
    );
    // This month so far: the 10 days before it are inside the books, which start on 31-08-26.
    s.period("this_month");
    for view in ["A", "B", "H", "D"] {
        s.ok(
            "analytics",
            analytics(
                view,
                json!({"from": "r1.from", "to": "r1.to", "compare": "previous", "rows": 10}),
            ),
        );
    }
    // Sorted by the change, as the example of the prompt does.
    let a = s.ok(
        "analytics",
        analytics(
            "A",
            json!({"from": "r1.from", "to": "r1.to", "compare": "previous",
                   "sort": sort("difference.total_comm", "asc")}),
        ),
    );
    assert_eq!(
        s.json(&a)["compare"]["difference"]["total_comm"]["rs"],
        5_448
    );
}

#[test]
fn playbook_6_kaun_sa_maal_para_hai() {
    let w = day();
    let mut s = Session::new(&w, "2026-10-10", "kaun sa maal para hai");
    s.ok(
        "report",
        report("R10", json!({"sort": sort("days", "desc")})),
    );
}

#[test]
fn playbook_7_kharcha_kahan_zyada() {
    let w = day();
    let mut s = Session::new(&w, "2026-10-10", "kharcha kahan zyada");
    s.period("this_month");
    s.period("last_month");
    let now = s.ok(
        "report",
        report("R29", json!({"from": "r1.from", "to": "r1.to"})),
    );
    let before = s.ok(
        "report",
        report("R29", json!({"from": "r2.from", "to": "r2.to"})),
    );
    // This month has the day's 3,000 of electricity; last month has none, so the percent change
    // from nothing is refused by the calculator, never guessed: the refusal is a result for the
    // model, with a reason.
    let change = s.try_run(
        "calculate",
        json!({"op": "change_percent", "inputs": [format!("{before}.totals.expense"), format!("{now}.totals.expense")]}),
    );
    match change {
        Ok(id) => assert!(s.json(&id).get("result").is_some()),
        Err(e) => assert!(!e.to_json()["error"]["detail"].as_str().unwrap().is_empty()),
    }
    // The difference works either way.
    s.ok(
        "calculate",
        json!({"op": "difference", "inputs": [format!("{now}.totals.expense"), format!("{before}.totals.expense")]}),
    );
}

#[test]
fn playbook_8_partners_trips_gardens() {
    let w = unlocked_day();
    let mut s = Session::new(&w, "2026-10-10", "partners trips gardens ka hisab");
    s.period("this_year");
    s.ok("report", report("R31", json!({})));
    s.ok(
        "report",
        report("R30", json!({"from": "r1.from", "to": "r1.to"})),
    );
    s.ok(
        "analytics",
        analytics("F", json!({"from": "r1.from", "to": "r1.to"})),
    );
    s.ok(
        "report",
        report("R33", json!({"from": "r1.from", "to": "r1.to"})),
    );
    s.ok(
        "analytics",
        analytics("J", json!({"from": "r1.from", "to": "r1.to"})),
    );
    s.ok(
        "report",
        report("R36", json!({"from": "r1.from", "to": "r1.to"})),
    );
    s.ok(
        "analytics",
        analytics("I", json!({"from": "r1.from", "to": "r1.to"})),
    );
}

#[test]
fn playbook_9_udhar_raqam_dobi_raqam() {
    let w = day();
    let mut s = Session::new(&w, "2026-10-10", "udhar raqam aur dobi raqam");
    s.ok("report", report("R32", json!({})));
    s.ok("report", report("R38", json!({})));
    s.ok("tiles", json!({}));
}

// ---- alerts: what the books can say today -----------------------------------------------------

#[test]
fn an_alert_about_old_bills_and_old_stock_has_a_path_with_the_number_of_the_question() {
    let w = day();
    // "kis ka bill 30 din se purana hai": the 30 is a number of the question, with days.
    let mut s = Session::new(&w, "2026-10-10", "kis ka bill 30 din se purana hai");
    assert_eq!(s.numbers.len(), 1);
    assert_eq!(s.numbers[0].unit, NumberUnit::Days);
    let r = s.ok(
        "report",
        report(
            "R20",
            json!({"where": {"column": "days", "op": "gt", "value": "q.n1"}, "sort": sort("days", "desc")}),
        ),
    );
    assert_eq!(
        s.json(&r)["matched"]["count"]["n"],
        0,
        "no bill is 30 days old on the worked day"
    );
    // "kaun sa maal 2 din se para hai"
    let mut s = Session::new(&w, "2026-10-10", "kaun sa maal 2 din se para hai");
    s.ok(
        "report",
        report(
            "R10",
            json!({"where": {"column": "days", "op": "gt", "value": "q.n1"}}),
        ),
    );
    // The backup alert is a tile.
    let tiles = s.ok("tiles", json!({}));
    assert!(s.json(&tiles).get("last_backup").is_some());
}

// ---- the failure list: the first try is refused with the way out -------------------------------

#[test]
fn every_view_and_report_leaves_out_an_invented_sort_word_and_says_so() {
    // Plan 52 item 2: the first try is not refused. The word is dropped, every row is given, and the
    // note names the word and lists the sort words that work.
    let w = unlocked_day();
    let mut s = Session::new(&w, "2026-10-10", "sab se zyada kya bikta hai");
    s.period("this_year");
    let dated = |extra: Value| {
        let mut base = json!({"from": "r1.from", "to": "r1.to"});
        for (k, v) in extra.as_object().unwrap() {
            base[k] = v.clone();
        }
        base
    };
    for view in ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K"] {
        for invented in ["sales", "amount", "qty", "commission", "balance", "total"] {
            let first = s
                .try_run(
                    "analytics",
                    analytics(view, dated(json!({"sort": sort(invented, "desc")}))),
                )
                .unwrap_or_else(|e| panic!("{view} {invented}: {}", e.to_json()));
            // The word may happen to be a column of this view: then there is no note.
            if let Some(note) = s.json(&first).get("note") {
                let note = note.as_str().unwrap();
                assert!(
                    note.contains(invented) && note.contains("left out"),
                    "{view}: {note}"
                );
                assert!(
                    note.contains("sort words of this view are"),
                    "{view}: {note}"
                );
            }
        }
    }
}

#[test]
fn a_report_leaves_out_an_invented_sort_word_and_says_so() {
    let w = unlocked_day();
    let mut s = Session::new(&w, "2026-10-10", "kharcha kahan zyada");
    s.period("this_year");
    for id in [
        "R2", "R6", "R10", "R12", "R20", "R24", "R26", "R28", "R29", "R30", "R32", "R38",
    ] {
        let dated = !matches!(id, "R10" | "R20" | "R24" | "R26" | "R32" | "R38");
        let extra = |sort_value: Value| {
            if dated {
                json!({"from": "r1.from", "to": "r1.to", "sort": sort_value})
            } else {
                json!({"sort": sort_value})
            }
        };
        let done = s
            .try_run("report", report(id, extra(sort("amount_xyz", "desc"))))
            .unwrap_or_else(|e| panic!("{id}: {}", e.to_json()));
        let note = s.json(&done)["note"].as_str().unwrap().to_owned();
        assert!(
            note.contains("amount_xyz") && note.contains("left out"),
            "{id}: {note}"
        );
        assert!(
            note.contains("sort words of this report are"),
            "{id}: {note}"
        );
    }
}

#[test]
fn up_to_now_works_for_every_view_and_report_that_takes_a_period_and_a_comparison_over_it_is_left_out()
 {
    let w = unlocked_day();
    let mut s = Session::new(&w, "2026-10-10", "ab tak ki sale aur commission");
    s.period("up_to_now"); // r1: from is the first day of the books
    for view in ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K"] {
        s.ok(
            "analytics",
            analytics(view, json!({"from": "r1.from", "to": "r1.to"})),
        );
        // A comparison over it: the books start on that day, so there is nothing earlier. For the
        // views that take a comparison it is left out with a note; the others say they have no
        // comparison and list the views that do.
        let result = s.try_run(
            "analytics",
            analytics(
                view,
                json!({"from": "r1.from", "to": "r1.to", "compare": "previous"}),
            ),
        );
        let compares = ["A", "B", "C", "D", "E", "H"].contains(&view);
        if compares {
            let id = result.unwrap_or_else(|e| panic!("{view}: {}", e.to_json()));
            let v = s.json(&id);
            assert!(
                v["note"].as_str().unwrap().contains("no earlier period"),
                "{view}: {v}"
            );
            assert!(v["compare"].is_null(), "{view}");
        } else {
            let e = result.unwrap_err();
            assert_eq!(e.code, "ERR_OUT_OF_RANGE", "{view}");
            assert_eq!(e.field.as_deref(), Some("compare"));
            assert!(
                e.allowed.contains(&"A".to_owned()),
                "{view}: {:?}",
                e.allowed
            );
        }
    }
    for id in ["R2", "R6", "R11", "R12", "R15", "R28", "R29"] {
        s.ok(
            "report",
            report(id, json!({"from": "r1.from", "to": "r1.to"})),
        );
    }
}

#[test]
fn group_by_belongs_to_two_views_and_the_refusal_says_which() {
    let w = unlocked_day();
    let mut s = Session::new(&w, "2026-10-10", "monthly commission by item");
    s.period("this_year");
    for view in ["A", "B", "C", "E", "H"] {
        let e = s
            .try_run(
                "analytics",
                analytics(
                    view,
                    json!({"from": "r1.from", "to": "r1.to", "group_by": "month"}),
                ),
            )
            .unwrap_err();
        assert_eq!(e.field.as_deref(), Some("group_by"), "{view}");
        assert_eq!(e.allowed, ["D", "G"], "{view}");
    }
    for view in ["D", "G"] {
        s.ok(
            "analytics",
            analytics(
                view,
                json!({"from": "r1.from", "to": "r1.to", "group_by": "month"}),
            ),
        );
    }
}

#[test]
fn a_balance_list_takes_as_of_and_says_so_when_it_is_given_a_period() {
    let w = day();
    let mut s = Session::new(&w, "2026-10-10", "kis ka bill purana hai");
    s.period("this_year");
    for id in ["R20", "R24", "R26", "R32"] {
        let e = s
            .try_run(
                "report",
                report(id, json!({"from": "r1.from", "to": "r1.to"})),
            )
            .unwrap_err();
        assert!(
            e.allowed.contains(&"as_of".to_owned()),
            "{id}: {:?}",
            e.allowed
        );
        s.ok("report", report(id, json!({"as_of": "r1.to"})));
    }
}

#[test]
fn a_months_ahead_number_that_is_not_in_the_question_says_what_to_do() {
    let w = day();
    let mut s = Session::new(&w, "2026-10-10", "agle mahine kitna hoga");
    let e = s
        .try_run(
            "estimate",
            json!({"measure": "commission_all", "item": null, "seller": null, "buyer": null,
                   "garden": null, "trip": null, "ahead": 3, "target": null}),
        )
        .unwrap_err();
    assert_eq!(e.field.as_deref(), Some("ahead"));
    assert!(e.detail.contains("6"), "{}", e.detail);
    // With none, 6 months works (the history is too short, which the result says).
    let id = s.ok(
        "estimate",
        json!({"measure": "commission_all", "item": null, "seller": null, "buyer": null,
               "garden": null, "trip": null, "ahead": 6, "target": null}),
    );
    assert_eq!(s.json(&id)["too_short"], true);
}
