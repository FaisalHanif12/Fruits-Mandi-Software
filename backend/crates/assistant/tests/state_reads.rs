//! Plan 55 §11.1, the READ gaps R1 to R6 and P1 to P4, through the real tools on the PRD worked
//! day (and, read only, on the rich test year).
//!
//! - R1: `app_state{what:"rules"}` gives the rule that wins for a party, an item, a group or the
//!   defaults, and the rules behind it. Its winners are the very ones the "test a rule" box gives.
//! - R2: `app_state{what:"attention"}` is the attention list of the bell.
//! - R3: `app_state{what:"days"}` is the day state and the last month close.
//! - R4: `app_state{what:"audit"}` gives the audit rows of one بکری or one account, and no text.
//! - R5: `find` with the roles `staff` and `expense_head` lists and counts them.
//! - R6: `find` with the role `truck` finds the بکری of a truck number.
//! - P1 to P4: `report` passes `seller`, `zamindar`, `status`, `voucher_type`, `layout` and
//!   `partner: "guests"` to the report, as that report's spec allows, and into the button's filters.
//!
//! Every figure is also compared with the screen's own function, never only with a number typed
//! here (Code Standards Q3).
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "robustness_year/seed.rs"]
mod seed;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use mandi_assistant::answer::{AnswerPart, AnswerSource, ScreenRoute};
use mandi_assistant::check::{Check, NameSet, Verdict};
use mandi_assistant::fill::fill;
use mandi_assistant::model_answer::ModelAnswer;
use mandi_assistant::results::{ToolResult, Words, fit, tool_result_json};
use mandi_assistant::tools::{Env, State, Stop, ToolError, call};
use mandi_reports::analytics::{
    AnalyticsFilters, AnalyticsGroupBy, AnalyticsPage, AnalyticsRunInput, AnalyticsView,
    run as analytics_run,
};
use mandi_reports::dashboard::BackupFacts;
use mandi_reports::reports::{
    GardenView, PaidFilter, ReportFilters, ReportId, ReportRunInput, TripView, run as engine_run,
};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{BikriDraft, BikriSaveInput, HeaderInput};
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::masters::{MasterInput, MasterKind, MasterRow, StaffInput};
use mandi_services::core::rules::{ChargeMethod, ChargeSide, ChargeType, ItemGroup, RuleInput};
use mandi_services::core::vouchers::VoucherType;
use mandi_services::{ServiceError, bikri, masters, rules};
use serde_json::{Value, json};

use common::set_today;

/// A session of tool calls on one world; results get their ids and stay for the next call.
struct Session<'a> {
    world: &'a world::World,
    words: Words,
    state: State,
    locked: bool,
    today: String,
}

impl<'a> Session<'a> {
    fn new(world: &'a world::World) -> Self {
        Self {
            world,
            words: Words::new().unwrap(),
            state: State::default(),
            locked: false,
            today: "2026-10-10".to_owned(),
        }
    }

    fn on(mut self, today: &str) -> Self {
        self.today = today.to_owned();
        self
    }

    fn run(&mut self, tool: &str, args: Value) -> Result<String, Stop> {
        let env = Env {
            ctx: &self.world.env.ctx,
            actor: &self.world.owner,
            words: &self.words,
            language: Language::Ur,
            today: BusinessDate::parse_iso(&self.today).unwrap(),
            partner_locked: self.locked,
            numbers: &[],
            question: "",
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let draft = call(&env, &self.state, tool, &args.to_string())?;
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
        Ok(id)
    }

    fn ok(&mut self, tool: &str, args: Value) -> String {
        match self.run(tool, args) {
            Ok(id) => id,
            Err(Stop::Error(e)) => panic!("{tool} refused: {} {:?} {}", e.code, e.field, e.detail),
            Err(Stop::Locked) => panic!("{tool} locked"),
        }
    }

    fn error(&mut self, tool: &str, args: Value) -> ToolError {
        match self.run(tool, args) {
            Err(Stop::Error(e)) => e,
            Err(Stop::Locked) => panic!("locked, not an error"),
            Ok(id) => panic!("{tool} should have been refused, gave {id}"),
        }
    }

    fn json(&self, id: &str) -> Value {
        tool_result_json(self.state.find(id).unwrap(), &self.words)
    }

    fn result(&self, id: &str) -> &ToolResult {
        self.state.find(id).unwrap()
    }

    /// `find` one word in one role and give back the id of the result.
    fn find(&mut self, text: &str, role: &str) -> String {
        self.ok("find", json!({"text": text, "role": role}))
    }

    /// An answer the model might write, run through the checks of PRD FR-7.76 and, when they pass,
    /// filled. All the results so far belong to the question being answered.
    fn answer(&self, text: &str) -> (Verdict, Vec<AnswerPart>) {
        let env = Env {
            ctx: &self.world.env.ctx,
            actor: &self.world.owner,
            words: &self.words,
            language: Language::Ur,
            today: BusinessDate::parse_iso(&self.today).unwrap(),
            partner_locked: self.locked,
            numbers: &[],
            question: "",
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let full = json!({"kind": "answer", "text": text, "show": [], "next": [], "choose": null,
                          "entry": null, "nearest": null});
        let parsed = ModelAnswer::parse(&full.to_string()).unwrap();
        let names = NameSet::new(&self.state, &[]);
        let verdict = Check {
            env: &env,
            state: &self.state,
            names: &names,
        }
        .run(&parsed);
        let parts = fill(&env, &self.state, &parsed, "2026-10-10T09:00:00Z").parts;
        (verdict, parts)
    }
}

/// A `report` input as the model sends it: every field, the ones not used null.
fn report(id: &str, extra: Value) -> Value {
    let mut base = json!({
        "id": id, "from": null, "to": null, "as_of": null, "party": null, "item": null, "city": null,
        "adda": null, "collector": null, "partner": null, "trip": null, "garden": null,
        "bikri": null, "seller": null, "zamindar": null, "status": null, "voucher_type": null,
        "layout": null, "group_by": null, "where": null, "sort": null, "rows": null
    });
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

fn state_input(what: &str, extra: Value) -> Value {
    let mut base = json!({"what": what, "party": null, "item": null, "bikri": null});
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

/// Calls `f` with the spellings confirmed so far and confirms the ones the engine is unsure of, as
/// the screen asks the user.
fn confirmed<T>(mut f: impl FnMut(&[String]) -> Result<T, ServiceError>) -> T {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..8 {
        match f(&words) {
            Ok(done) => return done,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(text)) = item.values.get("words") {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    f(&words).unwrap()
}

const STAFF_PHONE: &str = "0311-7654321";

fn add_staff(w: &world::World, ur: &str, en: &str, job: &str) {
    confirmed(|words| {
        masters::save(
            &w.env.ctx,
            &w.owner,
            &MasterInput::Staff(StaffInput {
                id: None,
                version: None,
                name_ur: ur.into(),
                name_en: en.into(),
                code: String::new(),
                job: job.into(),
                phone: STAFF_PHONE.into(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    });
}

/// A rule to save: whose side, which charge, the party and the item it is for (none: any), how it
/// is worked out, its value and the day it starts.
struct RuleSpec<'a> {
    side: ChargeSide,
    charge: ChargeType,
    account: Option<i64>,
    item: Option<i64>,
    method: ChargeMethod,
    value: &'a str,
    from: &'a str,
}

fn add_rule(w: &world::World, spec: &RuleSpec<'_>) -> i64 {
    rules::save(
        &w.env.ctx,
        &w.owner,
        &RuleInput {
            id: None,
            version: None,
            side: spec.side,
            charge_type: spec.charge,
            account_id: spec.account,
            item_id: spec.item,
            item_group: None,
            method: spec.method,
            value: spec.value.into(),
            from_date: spec.from.into(),
            to_date: String::new(),
            is_active: true,
            note: "SECRET NOTE OF A RULE".into(),
        },
    )
    .unwrap()
    .row
    .id
}

fn rs(v: &Value) -> i64 {
    v["rs"].as_i64().unwrap()
}

// ---- P1 to P4: the report filters ---------------------------------------------------------------

/// The rows the engine itself gives for the same filters: the tool must agree with the screen.
fn engine_count(w: &world::World, id: ReportId, filters: ReportFilters) -> i64 {
    engine_run(
        w.env.ctx.db(),
        &ReportRunInput {
            id,
            filters,
            offset: 0,
            limit: 100,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap()
    .total_count
}

fn button_filters(s: &Session<'_>, id: &str) -> mandi_assistant::source::SourceReportFilters {
    match &s.result(id).source {
        Some(AnswerSource::Report { filters, .. }) => filters.clone(),
        other => panic!("no report button: {other:?}"),
    }
}

#[test]
fn status_keeps_the_paid_or_the_due_lines_of_r9_and_r38_and_the_button_says_so() {
    let w = day();
    let mut s = Session::new(&w);
    // Event 2: شاہد paid cash on the spot, so his line is paid and nothing is due.
    let paid = s.ok("report", report("R9", json!({"status": "paid"})));
    let due = s.ok("report", report("R9", json!({"status": "due"})));
    let all = s.ok("report", report("R9", json!({})));
    let count = |id: &str| s.json(id)["count"]["n"].as_i64().unwrap();
    assert_eq!(
        count(&paid),
        engine_count(
            &w,
            ReportId::R9,
            ReportFilters {
                status: Some(PaidFilter::Paid),
                ..ReportFilters::default()
            }
        )
    );
    assert_eq!(count(&due), 0);
    assert_eq!(count(&paid), count(&all));
    assert!(count(&paid) >= 1);
    assert_eq!(button_filters(&s, &due).status, Some(PaidFilter::Due));
    assert_eq!(button_filters(&s, &all).status, None);
    // R38 takes it too; R2 does not.
    let dead = s.ok("report", report("R38", json!({"status": "due"})));
    assert_eq!(s.json(&dead)["source"], "R38");
    let refused = s.error("report", report("R2", json!({"status": "due"})));
    assert_eq!(refused.code, "ERR_OUT_OF_RANGE");
    assert_eq!(refused.field.as_deref(), Some("status"));
    // A word the input does not take lists the words.
    let wrong = s.error("report", report("R9", json!({"status": "open"})));
    assert_eq!(wrong.allowed, ["paid", "due"]);
}

#[test]
fn voucher_type_keeps_one_kind_of_voucher_in_r15() {
    let w = day();
    let mut s = Session::new(&w);
    let receipt = s.ok("report", report("R15", json!({"voucher_type": "receipt"})));
    let payment = s.ok("report", report("R15", json!({"voucher_type": "payment"})));
    let all = s.ok("report", report("R15", json!({})));
    let count = |id: &str| s.json(id)["count"]["n"].as_i64().unwrap();
    for (id, kind) in [
        (&receipt, VoucherType::Receipt),
        (&payment, VoucherType::Payment),
    ] {
        assert_eq!(
            count(id),
            engine_count(
                &w,
                ReportId::R15,
                ReportFilters {
                    voucher_type: Some(kind),
                    ..ReportFilters::default()
                }
            )
        );
        assert_eq!(button_filters(&s, id).voucher_type, Some(kind));
    }
    // Events 3 and 4: one receipt and one payment, among all the vouchers.
    assert_eq!((count(&receipt), count(&payment)), (1, 1));
    assert!(count(&all) > count(&receipt));
    let wrong = s.error("report", report("R15", json!({"voucher_type": "cheque"})));
    assert_eq!(wrong.code, "ERR_OUT_OF_RANGE");
    assert!(wrong.allowed.contains(&"receipt".to_owned()) && wrong.allowed.len() == 13);
    assert_eq!(
        s.error("report", report("R2", json!({"voucher_type": "receipt"})))
            .code,
        "ERR_OUT_OF_RANGE"
    );
}

#[test]
fn seller_and_zamindar_narrow_the_sales_reports() {
    let w = day();
    let mut s = Session::new(&w);
    let rahim = s.find("rahim", "seller");
    let karim = s.find("karim", "seller");
    let akram = s.find("akram", "buyer");
    for id in ["R2", "R3", "R4", "R5", "R7"] {
        let all = s.ok("report", report(id, json!({})));
        let one = s.ok(
            "report",
            report(id, json!({"seller": format!("{rahim}.matches.0")})),
        );
        let filters = button_filters(&s, &one);
        assert_eq!(filters.seller.as_ref().map(|r| r.id), Some(w.rahim), "{id}");
        assert_eq!(
            s.json(&one)["count"]["n"].as_i64().unwrap(),
            engine_count(
                &w,
                report_id(id),
                ReportFilters {
                    seller_id: Some(w.rahim),
                    ..ReportFilters::default()
                }
            ),
            "{id}"
        );
        assert!(
            s.json(&one)["count"]["n"].as_i64().unwrap()
                <= s.json(&all)["count"]["n"].as_i64().unwrap(),
            "{id}"
        );
    }
    // R2 of رحیم's goods holds his two lines; کریم's one line is not among them.
    let r2 = s.ok(
        "report",
        report("R2", json!({"seller": format!("{rahim}.matches.0")})),
    );
    assert_eq!(s.json(&r2)["count"]["n"], 2);
    let r2k = s.ok(
        "report",
        report("R2", json!({"seller": format!("{karim}.matches.0")})),
    );
    assert_eq!(s.json(&r2k)["count"]["n"], 1);
    // A buyer's key is not a seller.
    let not_seller = s.error(
        "report",
        report("R2", json!({"seller": format!("{akram}.matches.0")})),
    );
    assert_eq!(not_seller.code, "ERR_FORMAT_INVALID");
    // The zamindar of the truck, on the reports that have him; the buyer reports of the dues do not.
    let zam = s.ok(
        "report",
        report("R2", json!({"zamindar": format!("{rahim}.matches.0")})),
    );
    assert_eq!(
        s.json(&zam)["count"]["n"].as_i64().unwrap(),
        engine_count(
            &w,
            ReportId::R2,
            ReportFilters {
                zamindar_id: Some(w.rahim),
                ..ReportFilters::default()
            }
        )
    );
    assert_eq!(
        button_filters(&s, &zam).zamindar.map(|r| r.id),
        Some(w.rahim)
    );
    let r27 = s.ok(
        "report",
        report("R27", json!({"zamindar": format!("{rahim}.matches.0")})),
    );
    assert_eq!(s.json(&r27)["source"], "R27");
    for id in ["R7", "R20", "R13"] {
        let mut args = json!({"zamindar": format!("{rahim}.matches.0")});
        if id == "R13" {
            args["party"] = json!(format!("{akram}.matches.0"));
        }
        assert_eq!(
            s.error("report", report(id, args)).code,
            "ERR_OUT_OF_RANGE",
            "{id}"
        );
    }
}

fn report_id(code: &str) -> ReportId {
    serde_json::from_value(json!(code)).unwrap()
}

#[test]
fn a_seller_named_twice_on_a_seller_only_report_is_refused() {
    let w = day();
    let mut s = Session::new(&w);
    let rahim = s.find("rahim", "seller");
    // R6 has no buyer filter, so `party` is already the seller: `seller` as well is a second one.
    let once = s.ok(
        "report",
        report("R6", json!({"party": format!("{rahim}.matches.0")})),
    );
    assert_eq!(
        button_filters(&s, &once).seller.map(|r| r.id),
        Some(w.rahim)
    );
    let once_seller = s.ok(
        "report",
        report("R6", json!({"seller": format!("{rahim}.matches.0")})),
    );
    assert_eq!(
        s.json(&once)["count"],
        s.json(&once_seller)["count"],
        "party and seller are the same filter on R6"
    );
    let twice = s.error(
        "report",
        report(
            "R6",
            json!({"party": format!("{rahim}.matches.0"), "seller": format!("{rahim}.matches.0")}),
        ),
    );
    assert_eq!(twice.code, "ERR_FORMAT_INVALID");
    assert_eq!(twice.field.as_deref(), Some("seller"));
}

#[test]
fn layout_is_the_statement_detail_the_trip_view_and_the_garden_view() {
    let w = day();
    let mut s = Session::new(&w);
    let akram = s.find("akram", "buyer");
    let party = format!("{akram}.matches.0");
    // R14: balances only, or with its lines.
    let short = s.ok("report", report("R14", json!({"party": party})));
    let long = s.ok(
        "report",
        report("R14", json!({"party": party, "layout": "detail"})),
    );
    assert!(!button_filters(&s, &short).detail);
    assert!(button_filters(&s, &long).detail);
    let rows = |id: &str| s.json(id)["rows"].as_array().unwrap().len();
    assert!(rows(&long) >= rows(&short));
    // R33 by month and R36 by partner; neither has the other's words.
    let trips = s.ok("report", report("R33", json!({"layout": "month"})));
    assert_eq!(button_filters(&s, &trips).trip_view, Some(TripView::Month));
    let gardens = s.ok("report", report("R36", json!({"layout": "partner"})));
    assert_eq!(
        button_filters(&s, &gardens).garden_view,
        Some(GardenView::Partner)
    );
    let wrong = s.error("report", report("R36", json!({"layout": "month"})));
    assert_eq!(wrong.allowed, ["garden", "partner"]);
    let wrong = s.error("report", report("R33", json!({"layout": "garden"})));
    assert_eq!(wrong.allowed, ["trip", "item", "day", "month", "partner"]);
    // A report with no layout refuses and lists the fields it does take.
    let none = s.error("report", report("R2", json!({"layout": "detail"})));
    assert_eq!(none.code, "ERR_OUT_OF_RANGE");
    assert!(none.allowed.contains(&"seller".to_owned()));
    assert!(none.allowed.contains(&"zamindar".to_owned()));
}

#[test]
fn partner_guests_reads_only_the_fruit_for_the_shops_guests() {
    let w = day();
    let mut s = Session::new(&w);
    let guests = s.ok("report", report("R37", json!({"partner": "guests"})));
    assert_eq!(s.json(&guests)["source"], "R37");
    assert!(button_filters(&s, &guests).guests_only);
    assert!(button_filters(&s, &guests).partner.is_none());
    assert_eq!(
        s.json(&guests)["count"]["n"].as_i64().unwrap(),
        engine_count(
            &w,
            ReportId::R37,
            ReportFilters {
                guests_only: Some(true),
                ..ReportFilters::default()
            }
        )
    );
    // Not a fruit register: refused, and the allowed fields are listed.
    let refused = s.error("report", report("R29", json!({"partner": "guests"})));
    assert_eq!(refused.code, "ERR_OUT_OF_RANGE");
}

// ---- the collector filter (R22, R23, R35) ---------------------------------------------------------

/// A buyer with an opening debit and a collector (the collector on the account form, PRD 10.2).
fn buyer_of_collector(
    w: &world::World,
    ur: &str,
    en: &str,
    collector: Option<i64>,
    opening_dr: &str,
) -> i64 {
    let account = confirmed(|words| {
        mandi_services::accounts::save(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::accounts::AccountInput {
                id: None,
                version: None,
                account_type: mandi_services::core::accounts::AccountType::Buyer,
                is_pukhta_party: false,
                control_id: None,
                code: String::new(),
                name_ur: ur.into(),
                name_en: en.into(),
                confirmed_rules_words: words.to_vec(),
                father_or_firm_ur: String::new(),
                father_or_firm_en: String::new(),
                city_id: None,
                adda_id: None,
                phone1: String::new(),
                phone2: String::new(),
                address: String::new(),
                linked_account_id: None,
                opening_date: "10-10-2026".into(),
                opening_dr: opening_dr.into(),
                opening_cr: String::new(),
                opening_is_advance: false,
                credit_allowed: true,
                credit_limit: String::new(),
                default_marka_id: None,
                collector_staff_id: collector,
                is_active: true,
                note: String::new(),
                tag: String::new(),
            },
        )
    });
    account.id
}

fn staff_id(w: &world::World, ur: &str, en: &str, job: &str) -> i64 {
    add_staff(w, ur, en, job);
    masters::list(
        &w.env.ctx,
        mandi_services::core::masters::MasterKind::Staff,
        false,
    )
    .unwrap()
    .into_iter()
    .find_map(|row| match row {
        mandi_services::core::masters::MasterRow::Staff(s) if s.name_en == en => Some(s.id),
        _ => None,
    })
    .expect("the staff member just made")
}

/// The worked day (Akram 2,880 and Arshad 18,684 owe, none has a collector), plus two collectors,
/// Bilal and Aslam. Fazal owes 5,000 and is Bilal's; Gul owes 3,000 and is Aslam's. Fazal paid
/// 1,000 to Bilal on 10-10-26.
fn collectors_world() -> (world::World, i64, i64) {
    let w = day();
    let bilal = staff_id(&w, "بلال", "Bilal", "collector");
    let aslam = staff_id(&w, "اسلم", "Aslam", "collector");
    let fazal = buyer_of_collector(&w, "فضل", "Fazal", Some(bilal), "5,000");
    buyer_of_collector(&w, "گل", "Gul", Some(aslam), "3,000");
    mandi_services::vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::ReceiptInput {
            collector_staff_id: Some(bilal),
            ..worked::receipt(fazal, "1,000")
        },
    )
    .unwrap();
    (w, bilal, aslam)
}

#[test]
fn the_collector_narrows_the_dues_by_collector_r22() {
    let (w, bilal, aslam) = collectors_world();
    let mut s = Session::new(&w);
    let all = s.ok("report", report("R22", json!({})));
    let key = s.find("bilal", "collector");
    let mine = s.ok(
        "report",
        report("R22", json!({"collector": format!("{key}.matches.0")})),
    );
    let key2 = s.find("aslam", "staff");
    let his = s.ok(
        "report",
        report("R22", json!({"collector": format!("{key2}.matches.0")})),
    );
    let total = |id: &str| rs(&s.json(id)["totals"]["balance"]);
    // Fazal owes 5,000 less the 1,000 he paid; Gul owes 3,000; the worked day's two buyers have none.
    assert_eq!(total(&mine), 4_000);
    assert_eq!(total(&his), 3_000);
    assert_eq!(total(&all), 21_564 + 4_000 + 3_000);
    // The tool's counts are the engine's own, and the button carries the collector.
    for (id, collector) in [(&mine, bilal), (&his, aslam)] {
        assert_eq!(
            s.json(id)["count"]["n"].as_i64().unwrap(),
            engine_count(
                &w,
                ReportId::R22,
                ReportFilters {
                    collector_id: Some(collector),
                    ..ReportFilters::default()
                }
            )
        );
        assert_eq!(
            button_filters(&s, id).collector.map(|r| r.id),
            Some(collector)
        );
    }
    assert!(s.json(&mine)["filters"].is_object());
    // R20 and R21 do not take it, and a buyer's key is not a collector.
    assert_eq!(
        s.error(
            "report",
            report("R20", json!({"collector": format!("{key}.matches.0")}))
        )
        .code,
        "ERR_OUT_OF_RANGE"
    );
    let buyer = s.find("akram", "buyer");
    let wrong = s.error(
        "report",
        report("R22", json!({"collector": format!("{buyer}.matches.0")})),
    );
    assert_eq!(wrong.code, "ERR_FORMAT_INVALID");
}

#[test]
fn the_collector_narrows_the_receipts_register_r23() {
    let (w, bilal, aslam) = collectors_world();
    let mut s = Session::new(&w);
    let key = s.find("bilal", "collector");
    let all = s.ok("report", report("R23", json!({})));
    let mine = s.ok(
        "report",
        report("R23", json!({"collector": format!("{key}.matches.0")})),
    );
    // Event 3's receipt has no collector; Fazal's 1,000 went to Bilal.
    assert_eq!(s.json(&all)["count"]["n"], 2);
    assert_eq!(s.json(&mine)["count"]["n"], 1);
    assert_eq!(rs(&s.json(&mine)["totals"]["amount"]), 1_000);
    assert_eq!(
        s.json(&mine)["count"]["n"].as_i64().unwrap(),
        engine_count(
            &w,
            ReportId::R23,
            ReportFilters {
                collector_id: Some(bilal),
                ..ReportFilters::default()
            }
        )
    );
    // Aslam took none.
    let key2 = s.find("aslam", "staff");
    let none = s.ok(
        "report",
        report("R23", json!({"collector": format!("{key2}.matches.0")})),
    );
    assert_eq!(s.json(&none)["count"]["n"], 0);
    assert_eq!(
        engine_count(
            &w,
            ReportId::R23,
            ReportFilters {
                collector_id: Some(aslam),
                ..ReportFilters::default()
            }
        ),
        0
    );
    // The payments register (R25) and the vouchers list (R15) do not take a collector.
    for id in ["R25", "R15"] {
        assert_eq!(
            s.error(
                "report",
                report(id, json!({"collector": format!("{key}.matches.0")}))
            )
            .code,
            "ERR_OUT_OF_RANGE",
            "{id}"
        );
    }
}

#[test]
fn the_collector_narrows_the_party_day_sheet_r35() {
    let (w, bilal, _) = collectors_world();
    let mut s = Session::new(&w);
    let key = s.find("bilal", "collector");
    let fazal = s.find("fazal", "buyer");
    let akram = s.find("akram", "buyer");
    let party_rows = |s: &Session<'_>, id: &str| -> usize {
        s.json(id)["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r.get("party").is_some_and(|p| !p.is_null()))
            .count()
    };
    let all = s.ok("report", report("R35", json!({"as_of": "2026-10-10"})));
    let mine = s.ok(
        "report",
        report(
            "R35",
            json!({"as_of": "2026-10-10", "collector": format!("{key}.matches.0")}),
        ),
    );
    // Only Bilal's buyer is left, and the page is the engine's.
    assert_eq!(party_rows(&s, &mine), 1);
    assert!(party_rows(&s, &all) > 1);
    assert_eq!(
        s.json(&mine)["count"]["n"].as_i64().unwrap(),
        engine_count(
            &w,
            ReportId::R35,
            ReportFilters {
                as_of: "2026-10-10".into(),
                collector_id: Some(bilal),
                ..ReportFilters::default()
            }
        )
    );
    // His buyer's slip works with him; another buyer's slip does not.
    s.ok(
        "report",
        report(
            "R35",
            json!({"as_of": "2026-10-10", "party": format!("{fazal}.matches.0"),
                   "collector": format!("{key}.matches.0")}),
        ),
    );
    let other = s.error(
        "report",
        report(
            "R35",
            json!({"as_of": "2026-10-10", "party": format!("{akram}.matches.0"),
                   "collector": format!("{key}.matches.0")}),
        ),
    );
    assert_eq!(other.code, "ERR_OUT_OF_RANGE");
    assert_eq!(other.field.as_deref(), Some("collector"));
    // R34 (the day summary) has no collector.
    assert_eq!(
        s.error(
            "report",
            report("R34", json!({"collector": format!("{key}.matches.0")}))
        )
        .code,
        "ERR_OUT_OF_RANGE"
    );
}

// ---- the screen buttons ----------------------------------------------------------------------------

#[test]
fn each_state_read_has_the_button_of_its_screen() {
    let w = day();
    let mut s = Session::new(&w);
    let screen = |s: &Session<'_>, id: &str| match &s.result(id).source {
        Some(AnswerSource::Screen { route, .. }) => *route,
        other => panic!("no screen button: {other:?}"),
    };
    let rules_id = s.ok("app_state", state_input("rules", json!({})));
    let days_id = s.ok("app_state", state_input("days", json!({})));
    let attention_id = s.ok("app_state", state_input("attention", json!({})));
    let audit_id = s.ok("app_state", state_input("audit", json!({"bikri": 1})));
    assert_eq!(screen(&s, &rules_id), ScreenRoute::CommissionRules);
    assert_eq!(screen(&s, &days_id), ScreenRoute::DayClose);
    assert_eq!(screen(&s, &attention_id), ScreenRoute::Dashboard);
    assert_eq!(screen(&s, &audit_id), ScreenRoute::Audit);
    // On the wire: the kind, the words of the screen, the id of the result filled in later.
    let wire = serde_json::to_value(s.result(&days_id).source.as_ref().unwrap()).unwrap();
    assert_eq!(
        wire,
        json!({"kind": "screen", "result_id": "", "route": "day_close"})
    );
}

// ---- R5 and R6: find ------------------------------------------------------------------------------

#[test]
fn find_counts_the_staff_by_job_and_by_name_and_never_gives_a_phone() {
    let w = day();
    add_staff(&w, "اسلم", "Aslam", "munshi");
    add_staff(&w, "ناصر", "Nasir", "munshi");
    add_staff(&w, "بلال", "Bilal", "collector");
    let mut s = Session::new(&w);
    let munshis = s.find("munshi", "staff");
    let j = s.json(&munshis);
    assert_eq!(j["count"]["n"], 2);
    assert_eq!(j["matches"].as_array().unwrap().len(), 2);
    assert!(
        j["matches"]
            .as_array()
            .unwrap()
            .iter()
            .all(|m| m["job"] == "munshi")
    );
    assert_eq!(j["by_job"]["munshi"]["n"], 2);
    assert_eq!(j["by_job"]["collector"]["n"], 0);
    // The Urdu word and the all-staff word.
    let urdu = s.find("منشی", "staff");
    assert_eq!(s.json(&urdu)["count"]["n"], 2);
    let all = s.find("staff", "staff");
    let j = s.json(&all);
    assert_eq!(j["count"]["n"], 3);
    assert_eq!(
        (
            j["by_job"]["munshi"]["n"].as_i64(),
            j["by_job"]["collector"]["n"].as_i64(),
            j["by_job"]["labour"]["n"].as_i64()
        ),
        (Some(2), Some(1), Some(0))
    );
    let nobody = s.find("mazdoor", "staff");
    assert_eq!(s.json(&nobody)["count"]["n"], 0);
    // A name, with the job of the one found, and a key the other tools may be given.
    let by_name = s.find("bilal", "staff");
    let j = s.json(&by_name);
    assert_eq!(j["count"]["n"], 1);
    assert_eq!(j["matches"][0]["job"], "collector");
    assert!(j["matches"][0]["key"].as_str().unwrap().starts_with("st"));
    assert!(j.get("by_job").is_none() || j["by_job"].is_null());
    // The phone and the code of the account are not in any result.
    for id in [&munshis, &all, &by_name] {
        assert!(!s.json(id).to_string().contains(STAFF_PHONE));
    }
}

#[test]
fn find_lists_and_counts_the_expense_heads() {
    let w = day();
    worked::expense_head(&w, "چائے", "Tea");
    worked::expense_head(&w, "کرایہ دکان", "Shop Rent");
    let mut s = Session::new(&w);
    let all = s.find("all", "expense_head");
    let j = s.json(&all);
    let heads = masters::list(
        &w.env.ctx,
        mandi_services::core::masters::MasterKind::ExpenseHead,
        false,
    )
    .unwrap()
    .len();
    assert_eq!(
        j["count"]["n"].as_i64().unwrap(),
        i64::try_from(heads).unwrap()
    );
    assert!(heads >= 3, "Electricity, Tea and Shop Rent");
    assert_eq!(j["matches"].as_array().unwrap().len(), heads.min(12));
    assert!(j["matches"][0]["key"].as_str().unwrap().starts_with("eh"));
    let one = s.find("electricity", "expense_head");
    let j = s.json(&one);
    assert_eq!(j["count"]["n"], 1);
    assert_eq!(j["matches"][0]["name"]["en"], "Electricity");
    let kharcha = s.find("kharche", "expense_head");
    assert_eq!(
        s.json(&kharcha)["count"]["n"].as_i64().unwrap(),
        i64::try_from(heads).unwrap()
    );
}

#[test]
fn find_finds_the_bikri_of_a_truck_number() {
    let w = day();
    let mut s = Session::new(&w);
    let les = s.find("LES-1234", "truck");
    let j = s.json(&les);
    assert_eq!(j["count"]["n"], 1);
    let m = &j["matches"][0];
    assert_eq!(m["number"]["n"], 1);
    assert_eq!(m["bikri"]["label"]["en"], "Sale 1");
    assert_eq!(m["status"], "saved");
    assert_eq!(m["seller"]["en"], "Rahim");
    assert_eq!(m["date"]["date"], "2026-10-10");
    // Event 1: gross 29,400, net 19,316 (PRD 4.4).
    assert_eq!((rs(&m["gross"]), rs(&m["net"])), (29_400, 19_316));
    // The start of a number, in any case; a number typed with a space; none at all.
    let start = s.find("lhr", "truck");
    assert_eq!(s.json(&start)["matches"][0]["number"]["n"], 2);
    let spaced = s.find("les 1234", "truck");
    assert_eq!(s.json(&spaced)["matches"][0]["number"]["n"], 1);
    let none = s.find("ZZZ-0000", "truck");
    assert_eq!(s.json(&none)["count"]["n"], 0);
    // The number the model reads is what `report` R1 takes.
    let sale_book = s.ok("report", report("R1", json!({"bikri": 1})));
    assert_eq!(s.json(&sale_book)["source"], "R1");
}

// ---- the whole audit log and the partner lock -------------------------------------------------------

#[test]
fn the_whole_audit_log_hides_a_partners_shares_while_the_section_is_locked() {
    let w = day();
    // The newest event of the log is a change of the partners' shares (the same 40 / 30 / 30,
    // from today): an event of entity `partner`, with his shares in it.
    mandi_services::partners::set_shares(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::partners::PartnersSetSharesInput {
            effective_from: "10-10-2026".into(),
            shares: [
                (w.partners[0], "40"),
                (w.partners[1], "30"),
                (w.partners[2], "30"),
            ]
            .iter()
            .map(
                |&(partner_id, share)| mandi_services::core::partners::ShareInput {
                    partner_id,
                    share: share.to_owned(),
                },
            )
            .collect(),
        },
    )
    .unwrap();
    let read = |locked: bool| {
        let mut s = Session::new(&w);
        s.locked = locked;
        let id = s.ok("app_state", state_input("audit", json!({})));
        let flags = (
            s.result(&id).partner_data,
            s.result(&id).partner_parts_locked,
        );
        (s.json(&id), flags)
    };
    let (open, open_flags) = read(false);
    let (locked, locked_flags) = read(true);
    let partner_row = |j: &Value| {
        j["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["entity"] == "partner")
            .cloned()
            .expect("an event of the partners in the newest rows")
    };
    // Open: his shares are in the row, and the result is marked so it is not resent once locked.
    assert!(!partner_row(&open)["changes"].as_array().unwrap().is_empty());
    assert_eq!(open_flags, (true, false));
    // Locked: who, when and what stay; his old and new values do not, and the result says so.
    assert!(
        partner_row(&locked)["changes"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(locked_flags, (false, true));
    assert_eq!(locked["partner_parts"], "locked");
}

// ---- gap Q5: the smaller master lists ---------------------------------------------------------------

fn add_store(w: &world::World, ur: &str, en: &str) {
    confirmed(|words| {
        masters::save(
            &w.env.ctx,
            &w.owner,
            &MasterInput::Store(mandi_services::core::masters::StoreInput {
                id: None,
                version: None,
                code: String::new(),
                name_ur: ur.into(),
                name_en: en.into(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    });
}

fn add_narration(w: &world::World, ur: &str, en: &str, used_in: &[&str]) {
    masters::save(
        &w.env.ctx,
        &w.owner,
        &MasterInput::Narration(mandi_services::core::masters::NarrationInput {
            id: None,
            version: None,
            text_ur: ur.into(),
            text_en: en.into(),
            used_in: used_in.iter().map(|s| (*s).to_owned()).collect(),
        }),
    )
    .unwrap();
}

const MARK_TEXT: &str = "XQZ-77";

fn add_marka(w: &world::World, owner: i64) {
    masters::save(
        &w.env.ctx,
        &w.owner,
        &MasterInput::Marka(mandi_services::core::masters::MarkaInput {
            id: None,
            version: None,
            code: String::new(),
            mark_text: MARK_TEXT.into(),
            owner_account_id: Some(owner),
        }),
    )
    .unwrap();
}

#[test]
fn find_counts_and_lists_the_packings_stores_marks_and_narrations_without_their_typed_text() {
    let w = day();
    world::add_packing(&w.env, &w.owner, "کریٹ", "Crate");
    add_store(&w, "گودام ایک", "Main Store");
    add_store(&w, "ٹھنڈا گودام", "Cold Store");
    add_marka(&w, w.rahim);
    add_narration(&w, "نقد وصول", "Cash received", &["receipt"]);
    add_narration(&w, "بجلی کا بل", "Electricity bill", &["shop_expense"]);
    let mut s = Session::new(&w);
    let count_in = |kind: mandi_services::core::masters::MasterKind| {
        i64::try_from(masters::list(&w.env.ctx, kind, false).unwrap().len()).unwrap()
    };
    use mandi_services::core::masters::MasterKind;

    // Figure and list: the count is the master list's own, the matches are its records.
    for (role, kind, prefix) in [
        ("packing", MasterKind::Packing, "pk"),
        ("store", MasterKind::Store, "sr"),
        ("marka", MasterKind::Marka, "mk"),
        ("narration", MasterKind::Narration, "nr"),
    ] {
        let id = s.find("all", role);
        let j = s.json(&id);
        assert_eq!(j["role"], role);
        assert_eq!(j["count"]["n"].as_i64().unwrap(), count_in(kind), "{role}");
        assert!(j["count"]["n"].as_i64().unwrap() >= 1, "{role}");
        assert!(
            j["matches"]
                .as_array()
                .unwrap()
                .iter()
                .all(|m| m["key"].as_str().unwrap().starts_with(prefix)),
            "{role}"
        );
        // The role's own word lists everything too.
        let own = s.find(role, role);
        assert_eq!(s.json(&own)["count"], j["count"], "{role}");
    }
    // Two stores, two narrations, one mark, two packings (Box and Crate).
    let id = s.find("all", "store");
    assert_eq!(s.json(&id)["count"]["n"], 2);
    let id = s.find("all", "narration");
    assert_eq!(s.json(&id)["count"]["n"], 2);
    let id = s.find("all", "marka");
    assert_eq!(s.json(&id)["count"]["n"], 1);
    let id = s.find("all", "packing");
    assert_eq!(s.json(&id)["count"]["n"], 2);

    // A name finds its record, and a word nobody has finds none ("no such").
    let cold = s.find("cold", "store");
    let j = s.json(&cold);
    assert_eq!(j["count"]["n"], 1);
    assert_eq!(j["matches"][0]["name"]["en"], "Cold Store");
    let crate_ = s.find("crate", "packing");
    assert_eq!(s.json(&crate_)["matches"][0]["name"]["en"], "Crate");
    for role in ["packing", "store", "marka", "narration"] {
        let none = s.find("zzzzqq", role);
        let j = s.json(&none);
        assert_eq!(j["count"]["n"], 0, "{role}");
        assert!(j["matches"].as_array().unwrap().is_empty(), "{role}");
    }

    // A mark is told by its code and its owner; a narration by where it is used.
    let marka_id = s.find("all", "marka");
    let marks = s.json(&marka_id);
    assert_eq!(marks["matches"][0]["owner"]["en"], "Rahim");
    assert!(marks["matches"][0]["code"].is_string());
    let narration_id = s.find("all", "narration");
    let narrations = s.json(&narration_id);
    let used: Vec<&str> = narrations["matches"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|m| m["used_in"].as_array().unwrap().iter())
        .filter_map(Value::as_str)
        .collect();
    assert!(used.contains(&"receipt") && used.contains(&"shop_expense"));

    // What is typed by hand never goes: not the mark text, not a narration's words, whichever way
    // the record was asked for (FR-7.83).
    let mut everything = String::new();
    for (word, role) in [
        ("all", "marka"),
        ("all", "narration"),
        ("xqz", "marka"),
        ("bijli", "narration"),
        ("electricity", "narration"),
        ("cash", "narration"),
    ] {
        let id = s.find(word, role);
        everything.push_str(&s.json(&id).to_string());
    }
    for private in [
        MARK_TEXT,
        "نقد وصول",
        "Cash received",
        "بجلی کا بل",
        "Electricity bill",
    ] {
        assert!(!everything.contains(private), "{private}");
    }
}

#[test]
fn the_new_master_results_can_be_cited() {
    let w = day();
    add_store(&w, "گودام ایک", "Main Store");
    let mut s = Session::new(&w);
    let stores = s.find("all", "store");
    let packings = s.find("all", "packing");
    let (verdict, parts) = s.answer(&format!(
        "سٹور {{{{{stores}.count}}}} ہیں، {{{{{stores}.matches.0.name}}}}۔ پیکنگ {{{{{packings}.count}}}} ہیں۔"
    ));
    assert_eq!(verdict, Verdict::Ok);
    assert!(parts.iter().any(|p| matches!(p, AnswerPart::Name { .. })));
}

// ---- R3: days -------------------------------------------------------------------------------------

#[test]
fn days_says_what_is_closed_and_when_today_is_closed_too() {
    let w = day();
    let mut s = Session::new(&w);
    let before = s.ok("app_state", state_input("days", json!({})));
    let j = s.json(&before);
    // The worked day closes up to 09-10-26 and works on 10-10-26 (PRD 4.3).
    assert_eq!(j["today"]["date"], "2026-10-10");
    assert_eq!(j["closed_up_to"]["date"], "2026-10-09");
    assert_eq!(j["today_open"], true);
    assert!(j["last_month_close"].is_null());
    let list = mandi_services::month::list(&w.env.ctx).unwrap();
    let to_close = list
        .rows
        .iter()
        .filter(|r| {
            matches!(
                r.state,
                mandi_services::month::MonthState::Ready
                    | mandi_services::month::MonthState::DaysOpen
            )
        })
        .count();
    assert_eq!(
        j["months_to_close"]["n"].as_i64().unwrap(),
        i64::try_from(to_close).unwrap()
    );
    assert!(to_close >= 1);
    // Close today as well: today is then not open.
    mandi_services::day::close(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    let closed = s.ok("app_state", state_input("days", json!({})));
    let j = s.json(&closed);
    assert_eq!(j["closed_up_to"]["date"], "2026-10-10");
    assert_eq!(j["today_open"], false);
}

#[test]
fn days_gives_the_last_month_close_as_the_month_list_has_it() {
    // E2E-59's books: a sale on the 15th of April, May and June, today 10-10-26. Close the days
    // of April, then the month of April.
    let w = worked::months_world(&[(4, true, 10, 600), (5, true, 10, 600), (6, true, 10, 600)]);
    let mut s = Session::new(&w);
    let before = s.ok("app_state", state_input("days", json!({})));
    assert!(s.json(&before)["last_month_close"].is_null());
    mandi_services::day::close(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::DayCloseInput {
            up_to: "30-04-2026".into(),
        },
    )
    .unwrap();
    mandi_services::month::close(
        &w.env.ctx,
        &w.owner,
        &mandi_services::month::MonthInput {
            month: "2026-04".into(),
        },
    )
    .unwrap();
    let list = mandi_services::month::list(&w.env.ctx).unwrap();
    let closed = list
        .rows
        .iter()
        .find(|r| r.state == mandi_services::month::MonthState::Closed)
        .expect("April is closed");
    let to_close = list
        .rows
        .iter()
        .filter(|r| {
            matches!(
                r.state,
                mandi_services::month::MonthState::Ready
                    | mandi_services::month::MonthState::DaysOpen
            )
        })
        .count();
    let after = s.ok("app_state", state_input("days", json!({})));
    let j = s.json(&after);
    assert_eq!(j["last_month_close"]["month"]["month"], "2026-04");
    assert_eq!(
        j["last_month_close"]["closed_on"]["date"].as_str(),
        closed.closed_on.as_deref()
    );
    assert_eq!(j["closed_up_to"]["date"], "2026-04-30");
    assert_eq!(
        j["months_to_close"]["n"].as_i64().unwrap(),
        i64::try_from(to_close).unwrap()
    );
    // May and June are over but their days are open: both wait to be closed.
    assert!(to_close >= 2);
}

// ---- R2: attention --------------------------------------------------------------------------------

#[test]
fn attention_is_the_list_of_the_bell_with_its_records_to_open() {
    let w = day();
    // E2E-63's date: 15-11-26. اکرم's 2,880 and ارشد's 18,684 are over 30 days old, کریم's 10,160
    // payable is over 7 days old (PRD FR-7.44).
    set_today(&w.env, 2026, 11, 15);
    let mut s = Session::new(&w).on("2026-11-15");
    let id = s.ok("app_state", state_input("attention", json!({})));
    let j = s.json(&id);
    let list = mandi_reports::dashboard::attention(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap();
    // The very list of the bell: the same kinds, counts and total.
    assert_eq!(j["total"]["n"].as_i64().unwrap(), list.total_count);
    assert_eq!(j["items"].as_array().unwrap().len(), list.items.len());
    let dues = j["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["key"] == "dues_old")
        .expect("buyer dues older than 30 days");
    assert_eq!(dues["count"]["n"], 2);
    assert_eq!(rs(&dues["amount"]), 21_564);
    let payables = j["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["key"] == "payables_old")
        .expect("seller payables older than 7 days");
    assert_eq!(payables["count"]["n"], 1);
    assert_eq!(rs(&payables["amount"]), 10_160);
    // The first record is a key the other tools take.
    let key = dues["first"]["key"].as_str().unwrap().to_owned();
    assert!(
        key == format!("a{}", w.akram) || key == format!("a{}", w.arshad),
        "{key}"
    );
    let card = s.ok("party_card", json!({"party": key}));
    assert!(rs(&s.json(&card)["balance"]) > 0);
    // The button is the dashboard, and the filters of other reads are refused.
    assert!(matches!(
        s.result(&id).source,
        Some(AnswerSource::Screen {
            route: ScreenRoute::Dashboard,
            ..
        })
    ));
    assert_eq!(
        s.error("app_state", state_input("attention", json!({"bikri": 1})))
            .code,
        "ERR_OUT_OF_RANGE"
    );
}

#[test]
fn attention_leaves_out_the_partner_money_while_the_section_is_locked() {
    let w = day();
    // Partner A takes 500 more on 10-10-26 and promises it back by 12-10-26: it is past on 15-11-26.
    mandi_services::vouchers::save_personal_expense(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::expenses::PersonalExpenseInput {
            date: "10-10-2026".into(),
            partner_id: Some(w.partners[0]),
            amount: "500".into(),
            reason: "ذاتی".into(),
            return_by: Some("12-10-2026".into()),
            taken_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 11, 15);
    let takes = |locked: bool| {
        let mut s = Session::new(&w).on("2026-11-15");
        s.locked = locked;
        let id = s.ok("app_state", state_input("attention", json!({})));
        let j = s.json(&id);
        let item = j["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["key"] == "takes_overdue")
            .cloned()
            .expect("money a partner took, past its return-by day");
        let flags = (
            s.result(&id).partner_data,
            s.result(&id).partner_parts_locked,
        );
        (item, j, flags)
    };
    let (open, _, flags) = takes(false);
    assert_eq!(open["count"]["n"], 1);
    assert_eq!(rs(&open["amount"]), 500);
    assert_eq!(flags, (true, false));
    // Locked: the count stays, the money is left out, and the result says so.
    let (locked, j, flags) = takes(true);
    assert_eq!(locked["count"]["n"], 1);
    assert!(locked["amount"].is_null());
    assert_eq!(j["partner_parts"], "locked");
    assert_eq!(flags, (false, true));
}

// ---- R1: rules ------------------------------------------------------------------------------------

#[test]
fn rules_without_a_subject_give_the_defaults_and_the_group_rule() {
    let w = day();
    let mut s = Session::new(&w);
    let id = s.ok("app_state", state_input("rules", json!({})));
    let j = s.json(&id);
    let winners = &j["winners"];
    let by_key = |key: &str| {
        winners
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["key"] == key)
            .unwrap_or_else(|| panic!("no winner {key}"))
    };
    // Seller commission 6% and buyer commission 6% (PRD 4.2), as the default level.
    for key in ["seller_commission", "buyer_commission"] {
        let row = by_key(key);
        assert_eq!(row["found"], true, "{key}");
        assert_eq!(row["level"], "default", "{key}");
        assert_eq!(row["method"], "percent", "{key}");
        assert_eq!(row["value"]["pct"], "6.00", "{key}");
    }
    // لاگا is 8 for each نگ on fruit: a group rule, so with no group named no rule wins, and the
    // group rule is in the list behind it.
    assert_eq!(by_key("buyer_laga")["found"], false);
    let laga_group = j["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["charge"] == "laga" && r["level"] == "item_group")
        .expect("the fruit group rule");
    assert_eq!(
        (laga_group["method"].as_str(), rs(&laga_group["value"])),
        (Some("per_unit"), 8)
    );
    assert_eq!(laga_group["group"], "fruit");
    // Labour, munshiana and phone have no default (PRD 4.2).
    for key in ["seller_labour", "seller_munshiana", "seller_phone"] {
        assert_eq!(by_key(key)["found"], false, "{key}");
    }
    assert_eq!(j["as_of"]["date"], "2026-10-10");
}

#[test]
fn rules_for_a_group_or_an_item_find_the_group_rule() {
    let w = day();
    let mut s = Session::new(&w);
    let banana = s.find("kela", "item");
    for item in [json!("fruit"), json!(format!("{banana}.matches.0"))] {
        let id = s.ok("app_state", state_input("rules", json!({"item": item})));
        let j = s.json(&id);
        let laga = j["winners"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["key"] == "buyer_laga")
            .unwrap()
            .clone();
        assert_eq!(laga["found"], true);
        assert_eq!(laga["level"], "item_group");
        assert_eq!(laga["method"], "per_unit");
        assert_eq!(rs(&laga["value"]), 8);
        assert_eq!(j["subject"]["group"], "fruit");
    }
    // A vegetable has no لاگا (PRD 4.2).
    let id = s.ok(
        "app_state",
        state_input("rules", json!({"item": "vegetable"})),
    );
    let laga = s.json(&id)["winners"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["key"] == "buyer_laga")
        .unwrap()
        .clone();
    assert_eq!(laga["found"], false);
}

#[test]
fn rules_for_a_party_win_over_the_defaults_and_agree_with_test_a_rule() {
    let w = day();
    // رحیم sells کیلا at 5% (party and item), اکرم pays 4% commission (party only).
    let r1 = add_rule(
        &w,
        &RuleSpec {
            side: ChargeSide::Seller,
            charge: ChargeType::Commission,
            account: Some(w.rahim),
            item: Some(w.banana),
            method: ChargeMethod::Percent,
            value: "5",
            from: "01-10-2026",
        },
    );
    let r2 = add_rule(
        &w,
        &RuleSpec {
            side: ChargeSide::Buyer,
            charge: ChargeType::Commission,
            account: Some(w.akram),
            item: None,
            method: ChargeMethod::Percent,
            value: "4",
            from: "01-10-2026",
        },
    );
    // A rule that starts later does not win today.
    add_rule(
        &w,
        &RuleSpec {
            side: ChargeSide::Seller,
            charge: ChargeType::Labour,
            account: Some(w.karim),
            item: None,
            method: ChargeMethod::PerUnit,
            value: "5",
            from: "01-11-2026",
        },
    );
    let mut s = Session::new(&w);
    let rahim = s.find("rahim", "seller");
    let akram = s.find("akram", "buyer");
    let banana = s.find("kela", "item");
    let winner = |s: &Session<'_>, id: &str, key: &str| -> Value {
        s.json(id)["winners"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["key"] == key)
            .unwrap()
            .clone()
    };
    // رحیم and کیلا: his party-and-item rule decides his commission; the buyer side is the default.
    let id = s.ok(
        "app_state",
        state_input(
            "rules",
            json!({"party": format!("{rahim}.matches.0"), "item": format!("{banana}.matches.0")}),
        ),
    );
    let sc = winner(&s, &id, "seller_commission");
    assert_eq!(
        (sc["level"].as_str(), sc["value"]["pct"].as_str()),
        (Some("party_item"), Some("5.00"))
    );
    assert_eq!(sc["rule"], format!("ru{r1}"));
    assert_eq!(sc["from"]["date"], "2026-10-01");
    assert_eq!(winner(&s, &id, "buyer_commission")["level"], "default");
    // Without the item, only his party level could win: none, so the default.
    let id = s.ok(
        "app_state",
        state_input("rules", json!({"party": format!("{rahim}.matches.0")})),
    );
    assert_eq!(winner(&s, &id, "seller_commission")["level"], "default");
    // His rules are listed all the same, and nothing a rule's note says.
    let listed = s.json(&id)["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["key"] == format!("ru{r1}"));
    assert!(listed);
    assert!(!s.json(&id).to_string().contains("SECRET NOTE"));
    // اکرم: a buyer's party rule on the buyer side.
    let id = s.ok(
        "app_state",
        state_input("rules", json!({"party": format!("{akram}.matches.0")})),
    );
    let bc = winner(&s, &id, "buyer_commission");
    assert_eq!(
        (bc["level"].as_str(), bc["value"]["pct"].as_str()),
        (Some("party"), Some("4.00"))
    );
    assert_eq!(bc["rule"], format!("ru{r2}"));
    assert_eq!(winner(&s, &id, "seller_commission")["level"], "default");
    // کریم's labour rule starts on 01-11-26: not today's winner.
    let karim = s.find("karim", "seller");
    let id = s.ok(
        "app_state",
        state_input("rules", json!({"party": format!("{karim}.matches.0")})),
    );
    assert_eq!(winner(&s, &id, "seller_labour")["found"], false);
    // The winners are the very ones "test a rule" gives (PRD FR-11.8): رحیم, اکرم and کیلا.
    let test = rules::test(
        &w.env.ctx,
        &mandi_services::core::rules::RuleTestInput {
            seller_id: w.rahim,
            buyer_id: w.akram,
            item_id: w.banana,
            date: "10-10-2026".into(),
            qty: "10".into(),
            rate: "100".into(),
        },
    )
    .unwrap();
    let both = s.ok(
        "app_state",
        state_input(
            "rules",
            json!({"party": format!("{rahim}.matches.0"), "item": format!("{banana}.matches.0")}),
        ),
    );
    let both_buyer = s.ok(
        "app_state",
        state_input(
            "rules",
            json!({"party": format!("{akram}.matches.0"), "item": format!("{banana}.matches.0")}),
        ),
    );
    for charge in &test.charges {
        let key = format!("{}_{}", charge.side.as_str(), charge.charge_type.as_str());
        let from = match charge.side {
            ChargeSide::Seller => &both,
            ChargeSide::Buyer => &both_buyer,
        };
        let ours = winner(&s, from, &key);
        match &charge.winner {
            None => assert_eq!(ours["found"], false, "{key}"),
            Some(view) => {
                assert_eq!(ours["found"], true, "{key}");
                assert_eq!(ours["method"], view.method.as_str(), "{key}");
                match (view.value_bps, view.value_paisa) {
                    (Some(bps), _) => {
                        let text = format!("{}.{:02}", bps / 100, bps % 100);
                        assert_eq!(ours["value"]["pct"], text, "{key}");
                    }
                    (None, Some(paisa)) => assert_eq!(rs(&ours["value"]), paisa.rupees(), "{key}"),
                    (None, None) => panic!("a winner has a value"),
                }
            }
        }
    }
    // An item group in a rule list, a seller or a buyer only: a key of another kind is refused.
    let item_as_party = s.error(
        "app_state",
        state_input("rules", json!({"party": format!("{banana}.matches.0")})),
    );
    assert_eq!(item_as_party.code, "ERR_FORMAT_INVALID");
}

// ---- R4: audit ------------------------------------------------------------------------------------

#[test]
fn audit_gives_the_rows_of_a_bikri_or_an_account_and_never_a_text() {
    let w = day();
    let mut s = Session::new(&w);
    // بکری 1: the row that saved it.
    let id = s.ok("app_state", state_input("audit", json!({"bikri": 1})));
    let j = s.json(&id);
    assert_eq!(j["entity"], "bikri");
    assert!(j["count"]["n"].as_i64().unwrap() >= 1);
    let rows = j["rows"].as_array().unwrap();
    assert!(rows.iter().any(|r| r["action"] == "create"));
    assert!(rows.iter().all(|r| r["date"]["date"].is_string()));
    assert!(
        !j.to_string().contains("LES-1234"),
        "a truck number is a text"
    );
    // رحیم's account: it was made with a phone, an address, a father's name and a note.
    let rahim = s.find("rahim", "seller");
    let id = s.ok(
        "app_state",
        state_input("audit", json!({"party": format!("{rahim}.matches.0")})),
    );
    let text = s.json(&id).to_string();
    assert_eq!(s.json(&id)["entity"], "account");
    for private in [
        worked::PRIVATE_PHONE,
        worked::PRIVATE_ADDRESS,
        worked::PRIVATE_NOTE,
        "Haji Sahib",
        "حاجی صاحب",
    ] {
        assert!(!text.contains(private), "{private}");
    }
    // The count agrees with the audit viewer.
    let page = mandi_reports::audit::list(
        w.env.ctx.db(),
        &mandi_reports::audit::AuditListInput {
            filters: mandi_reports::audit::AuditFilters {
                entity: "account".into(),
                entity_id: Some(w.rahim),
                ..Default::default()
            },
            offset: 0,
            limit: 100,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap();
    assert_eq!(
        s.json(&id)["count"]["n"].as_i64().unwrap(),
        page.total_count
    );
    // Exactly one of a party and a number; a بکری that does not exist.
    // Neither: the newest rows of the whole log (the Owner asks what changed lately).
    let whole = s.ok("app_state", state_input("audit", json!({})));
    let j = s.json(&whole);
    assert_eq!(j["entity"], "all");
    let everything = mandi_reports::audit::list(
        w.env.ctx.db(),
        &mandi_reports::audit::AuditListInput {
            filters: mandi_reports::audit::AuditFilters::default(),
            offset: 0,
            limit: 100,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap();
    assert_eq!(j["count"]["n"].as_i64().unwrap(), everything.total_count);
    assert_eq!(j["rows"].as_array().unwrap().len(), 12);
    assert_eq!(j["more"], true);
    assert!(matches!(
        s.result(&whole).source,
        Some(AnswerSource::Screen {
            route: ScreenRoute::Audit,
            ..
        })
    ));
    assert_eq!(
        s.error(
            "app_state",
            state_input(
                "audit",
                json!({"bikri": 1, "party": format!("{rahim}.matches.0")})
            )
        )
        .code,
        "ERR_REQUIRED"
    );
    assert_eq!(
        s.error("app_state", state_input("audit", json!({"bikri": 9999})))
            .code,
        "ERR_NOT_FOUND"
    );
}

// ---- the tool itself ---------------------------------------------------------------------------------

#[test]
fn app_state_reads_its_input_strictly() {
    let w = day();
    let mut s = Session::new(&w);
    assert_eq!(
        s.error(
            "app_state",
            json!({"what": "weather", "party": null, "item": null, "bikri": null})
        )
        .code,
        "ERR_FORMAT_INVALID"
    );
    assert_eq!(
        s.error("app_state", json!({"what": "days", "color": "red"}))
            .code,
        "ERR_FORMAT_INVALID"
    );
    let days = s.error("app_state", state_input("days", json!({"party": "a1"})));
    assert_eq!(
        (days.code.as_str(), days.field.as_deref()),
        ("ERR_OUT_OF_RANGE", Some("party"))
    );
    // The fields a model may leave out are optional.
    s.ok("app_state", json!({"what": "days"}));
    // It writes nothing: the books are the same after every read.
    let before = s.json("r1");
    s.ok("app_state", json!({"what": "days"}));
    assert_eq!(before["today"], s.json("r2")["today"]);
}

// ---- the new results with the answer checks (PRD FR-7.76) ------------------------------------------

fn figures(parts: &[AnswerPart]) -> Vec<&mandi_reports::reports::Cell> {
    parts
        .iter()
        .filter_map(|p| match p {
            AnswerPart::Figure { cell, .. } => Some(cell),
            _ => None,
        })
        .collect()
}

#[test]
fn the_new_results_can_be_cited_and_a_typed_figure_or_name_is_still_refused() {
    use mandi_reports::reports::Cell;
    let w = day();
    add_staff(&w, "اسلم", "Aslam", "munshi");
    let mut s = Session::new(&w);
    let rules_id = s.ok("app_state", state_input("rules", json!({})));
    let staff = s.find("munshi", "staff");
    let days = s.ok("app_state", state_input("days", json!({})));
    let truck = s.find("LES-1234", "truck");
    let heads = s.find("all", "expense_head");
    // Rules: a percent from `winners`, an amount per نگ and its date from `rules`.
    let (verdict, parts) = s.answer(&format!(
        "فروخت کنندہ کا کمیشن {{{{{rules_id}.winners.seller_commission.value}}}} ہے۔"
    ));
    assert_eq!(verdict, Verdict::Ok);
    assert_eq!(
        figures(&parts),
        [&Cell::Percent { hundredths: 600 }],
        "6% is 600 hundredths"
    );
    let group_rule = s.json(&rules_id)["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["charge"] == "laga")
        .and_then(|r| r["key"].as_str().map(str::to_owned))
        .expect("the fruit group rule of لاگا");
    let (verdict, parts) = s.answer(&format!(
        "لاگا {{{{{rules_id}.rules.{group_rule}.value}}}} فی نگ، {{{{{rules_id}.rules.{group_rule}.from}}}} سے۔"
    ));
    assert_eq!(verdict, Verdict::Ok);
    assert!(parts.iter().any(|p| matches!(p, AnswerPart::Date { .. })));
    // Staff and heads: the count, and a name by reference.
    let (verdict, parts) = s.answer(&format!(
        "منشی {{{{{staff}.count}}}} ہیں، {{{{{staff}.matches.0.name}}}}۔ مد {{{{{heads}.count}}}} ہیں۔"
    ));
    assert_eq!(verdict, Verdict::Ok);
    assert!(figures(&parts).contains(&&Cell::Number { value: 1 }));
    assert!(parts.iter().any(|p| matches!(p, AnswerPart::Name { .. })));
    // The staff member's name typed instead of a reference is refused, as any name is.
    let (typed, _) = s.answer("منشی اسلم ہیں۔");
    assert!(matches!(typed, Verdict::Retry(f) if f.iter().any(|x| x.rule == "name")));
    // Days: dates; a truck: a label of the entry and its gross.
    let (verdict, parts) = s.answer(&format!(
        "{{{{{days}.closed_up_to}}}} تک دن بند ہیں۔ {{{{{truck}.matches.0.bikri}}}} کی خام بکری {{{{{truck}.matches.0.gross}}}}۔"
    ));
    assert_eq!(verdict, Verdict::Ok);
    assert!(parts.iter().any(|p| matches!(p, AnswerPart::Doc { .. })));
    assert!(parts.iter().any(|p| matches!(p, AnswerPart::Date { .. })));
    assert!(figures(&parts).contains(&&Cell::Money {
        paisa: mandi_services::core::money::Paisa::from_rupees(29_400).unwrap()
    }));
    // A figure typed by the model is refused whatever the result.
    let (typed, _) = s.answer("کمیشن 6 فیصد ہے۔");
    assert!(matches!(typed, Verdict::Retry(f) if f.iter().any(|x| x.rule == "digit")));
    // A path the result does not have is refused (a rule that is not listed).
    let (missing, _) = s.answer(&format!("{{{{{rules_id}.rules.ru9999.value}}}}"));
    assert!(matches!(missing, Verdict::Retry(f) if f.iter().any(|x| x.rule == "reference")));
}

// ---- the rich year, read only ----------------------------------------------------------------------

/// A year of books (about half a minute to build): every read is compared with the screen's own
/// function over real volume, with the gardens, the trips, the fruit and the dead amount in it.
#[test]
fn on_the_rich_year_every_read_agrees_with_the_screens() {
    let y = seed::build();
    let w = &y.w;
    let today = mandi_services::day::status(&w.env.ctx).unwrap().today;
    let mut s = Session::new(w).on(&today);

    // R3: the day state is the status bar's and the month list's.
    let status = mandi_services::day::status(&w.env.ctx).unwrap();
    let months = mandi_services::month::list(&w.env.ctx).unwrap();
    let id = s.ok("app_state", state_input("days", json!({})));
    let j = s.json(&id);
    assert_eq!(j["today"]["date"], status.today);
    assert_eq!(j["today_open"], status.today_open);
    assert_eq!(
        j["closed_up_to"]["date"].as_str().map(str::to_owned),
        status.closed_up_to
    );
    let to_close = months
        .rows
        .iter()
        .filter(|r| {
            matches!(
                r.state,
                mandi_services::month::MonthState::Ready
                    | mandi_services::month::MonthState::DaysOpen
            )
        })
        .count();
    assert_eq!(
        j["months_to_close"]["n"].as_i64().unwrap(),
        i64::try_from(to_close).unwrap()
    );

    // R2: the attention list is the bell's.
    let list = mandi_reports::dashboard::attention(
        w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap();
    let id = s.ok("app_state", state_input("attention", json!({})));
    let j = s.json(&id);
    assert_eq!(j["total"]["n"].as_i64().unwrap(), list.total_count);
    assert_eq!(j["items"].as_array().unwrap().len(), list.items.len());

    // R5: the staff and the expense heads are counted as the masters count them.
    let heads = masters::list(
        &w.env.ctx,
        mandi_services::core::masters::MasterKind::ExpenseHead,
        false,
    )
    .unwrap()
    .len();
    let id = s.find("all", "expense_head");
    assert_eq!(
        s.json(&id)["count"]["n"].as_i64().unwrap(),
        i64::try_from(heads).unwrap()
    );
    assert!(heads >= 5, "rent, electricity, salaries, tea, guests");
    let staff = masters::list(
        &w.env.ctx,
        mandi_services::core::masters::MasterKind::Staff,
        false,
    )
    .unwrap()
    .len();
    let id = s.find("staff", "staff");
    assert_eq!(
        s.json(&id)["count"]["n"].as_i64().unwrap(),
        i64::try_from(staff).unwrap()
    );

    // R6: the newest trucks are found, and the count is the service's own.
    let newest = mandi_services::bikri::find(
        &w.env.ctx,
        &mandi_services::core::bikri::BikriFindInput::default(),
    )
    .unwrap();
    for row in newest.iter().take(3) {
        let truck = row
            .truck_no
            .clone()
            .expect("the year gives every بکری a truck");
        let id = s.find(&truck, "truck");
        let j = s.json(&id);
        let same = mandi_services::bikri::find(
            &w.env.ctx,
            &mandi_services::core::bikri::BikriFindInput {
                truck_no: Some(truck.clone()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            j["count"]["n"].as_i64().unwrap(),
            i64::try_from(same.len()).unwrap()
        );
        assert!(
            j["matches"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m["number"]["n"] == row.number),
            "{truck}"
        );
        assert!(j.to_string().contains("\"gross\""));
    }

    // R4: the audit rows of the last بکری and of a buyer are the viewer's.
    let last = newest.first().unwrap();
    let id = s.ok(
        "app_state",
        state_input("audit", json!({"bikri": last.number})),
    );
    let viewer = |entity: &str, entity_id: i64| {
        mandi_reports::audit::list(
            w.env.ctx.db(),
            &mandi_reports::audit::AuditListInput {
                filters: mandi_reports::audit::AuditFilters {
                    entity: entity.into(),
                    entity_id: Some(entity_id),
                    ..Default::default()
                },
                offset: 0,
                limit: 100,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap()
        .total_count
    };
    assert_eq!(
        s.json(&id)["count"]["n"].as_i64().unwrap(),
        viewer("bikri", last.id)
    );

    // R1: the winners are "test a rule" for sellers, buyers and items of the year.
    let sellers: Vec<_> = y.sellers.iter().take(3).collect();
    let buyers: Vec<_> = y.buyers.iter().take(3).collect();
    let items: Vec<_> = y.items.iter().take(3).collect();
    for ((seller, buyer), item) in sellers.iter().zip(&buyers).zip(&items) {
        let name = |key: &str| key.replace('_', " ");
        let seller_key = s.find(&name(seller.key), "seller");
        let buyer_key = s.find(&name(buyer.key), "buyer");
        let item_key = s.find(&name(item.key), "item");
        let seller_state = s.ok(
            "app_state",
            state_input(
                "rules",
                json!({"party": format!("{seller_key}.matches.0"), "item": format!("{item_key}.matches.0")}),
            ),
        );
        let buyer_state = s.ok(
            "app_state",
            state_input(
                "rules",
                json!({"party": format!("{buyer_key}.matches.0"), "item": format!("{item_key}.matches.0")}),
            ),
        );
        let test = rules::test(
            &w.env.ctx,
            &mandi_services::core::rules::RuleTestInput {
                seller_id: seller.id,
                buyer_id: buyer.id,
                item_id: item.id,
                date: mandi_services::core::dates::format_ddmmyyyy(
                    BusinessDate::parse_iso(&today).unwrap(),
                ),
                qty: "10".into(),
                rate: "100".into(),
            },
        )
        .unwrap();
        for charge in &test.charges {
            let key = format!("{}_{}", charge.side.as_str(), charge.charge_type.as_str());
            let from = match charge.side {
                ChargeSide::Seller => &seller_state,
                ChargeSide::Buyer => &buyer_state,
            };
            let ours = s.json(from)["winners"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["key"] == key)
                .unwrap()
                .clone();
            assert_eq!(ours["found"], charge.winner.is_some(), "{key}");
            if let Some(view) = &charge.winner {
                assert_eq!(ours["method"], view.method.as_str(), "{key}");
            }
        }
    }

    // P1 to P4: every voucher kind, both statuses, every layout and the guests are the engine's.
    for kind in [
        "receipt",
        "payment",
        "advance",
        "shop_expense",
        "personal_expense",
        "personal_return",
        "money_lent",
        "money_lent_return",
        "general",
        "capital_in",
        "capital_out",
        "khata_out",
        "khata_in",
    ] {
        let id = s.ok("report", report("R15", json!({"voucher_type": kind})));
        assert_eq!(
            s.json(&id)["count"]["n"].as_i64().unwrap(),
            engine_count(
                w,
                ReportId::R15,
                ReportFilters {
                    voucher_type: VoucherType::parse(kind),
                    ..ReportFilters::default()
                }
            ),
            "{kind}"
        );
    }
    for (report_name, word, filter) in [
        ("R9", "paid", PaidFilter::Paid),
        ("R9", "due", PaidFilter::Due),
        ("R38", "due", PaidFilter::Due),
    ] {
        let id = s.ok("report", report(report_name, json!({"status": word})));
        assert_eq!(
            s.json(&id)["count"]["n"].as_i64().unwrap(),
            engine_count(
                w,
                report_id(report_name),
                ReportFilters {
                    status: Some(filter),
                    ..ReportFilters::default()
                }
            ),
            "{report_name} {word}"
        );
    }
    for (word, view) in [
        ("trip", TripView::Trip),
        ("item", TripView::Item),
        ("day", TripView::Day),
        ("month", TripView::Month),
        ("partner", TripView::Partner),
    ] {
        let id = s.ok("report", report("R33", json!({"layout": word})));
        assert_eq!(
            s.json(&id)["count"]["n"].as_i64().unwrap(),
            engine_count(
                w,
                ReportId::R33,
                ReportFilters {
                    trip_view: Some(view),
                    ..ReportFilters::default()
                }
            ),
            "R33 {word}"
        );
    }
    for (word, view) in [
        ("garden", GardenView::Garden),
        ("partner", GardenView::Partner),
    ] {
        let id = s.ok("report", report("R36", json!({"layout": word})));
        assert_eq!(
            s.json(&id)["count"]["n"].as_i64().unwrap(),
            engine_count(
                w,
                ReportId::R36,
                ReportFilters {
                    garden_view: Some(view),
                    ..ReportFilters::default()
                }
            ),
            "R36 {word}"
        );
    }
    let guests = s.ok("report", report("R37", json!({"partner": "guests"})));
    let guests_count = s.json(&guests)["count"]["n"].as_i64().unwrap();
    assert_eq!(
        guests_count,
        engine_count(
            w,
            ReportId::R37,
            ReportFilters {
                guests_only: Some(true),
                ..ReportFilters::default()
            }
        )
    );
    assert!(
        guests_count >= 1,
        "the year has fruit for the shop's guests"
    );
    for seller in y.sellers.iter().take(4) {
        let key = s.find(&seller.key.replace('_', " "), "seller");
        for id in ["R2", "R7"] {
            let one = s.ok(
                "report",
                report(id, json!({"seller": format!("{key}.matches.0")})),
            );
            assert_eq!(
                s.json(&one)["count"]["n"].as_i64().unwrap(),
                engine_count(
                    w,
                    report_id(id),
                    ReportFilters {
                        seller_id: Some(seller.id),
                        ..ReportFilters::default()
                    }
                ),
                "{id} {}",
                seller.key
            );
        }
    }
}

// ---- PRD v1.29: the item group filter (پھل / سبزی) ---------------------------------------------

/// The worked day and a third بکری of a vegetable, so both groups have sales. Worked first: کریم's
/// 20 مولی (vegetable) at 100 to اکرم, on credit, no lot charges.
/// Value 20 × 100 = 2,000; seller comm 2,000 × 6 ÷ 100 = 120; buyer comm 120; لاگا 0 (the لاگا
/// rule is for the fruit group, PRD 4.2); so the مولی row is qty 20, gross 2,000, total comm 240.
fn day_with_a_vegetable() -> world::World {
    let w = day();
    let mut lot = world::bare_lot(&w, "lot-3", w.karim, w.mooli, "20");
    lot.sales = vec![world::sale("s-1", w.akram, "20", "100")];
    let draft = BikriDraft {
        header: HeaderInput {
            truck_no: Some("KHI-0099".into()),
            ..world::header()
        },
        lots: vec![lot],
    };
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    w
}

/// The group of each item of the books, as the masters list it.
fn groups_of_items(w: &world::World) -> std::collections::HashMap<i64, ItemGroup> {
    masters::list(&w.env.ctx, MasterKind::Item, false)
        .unwrap()
        .into_iter()
        .filter_map(|row| match row {
            MasterRow::Item(item) => ItemGroup::parse(&item.item_group).map(|g| (item.id, g)),
            _ => None,
        })
        .collect()
}

fn view_filters(group: Option<ItemGroup>, by: Option<AnalyticsGroupBy>) -> AnalyticsFilters {
    AnalyticsFilters {
        item_group: group,
        group_by: by,
        ..AnalyticsFilters::default()
    }
}

fn view_page(w: &world::World, view: AnalyticsView, filters: AnalyticsFilters) -> AnalyticsPage {
    analytics_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view,
            filters,
            offset: 0,
            limit: 1_000,
            sort: None,
        },
    )
    .unwrap()
}

/// A whole number or whole rupees of a cell; nothing for any other cell.
fn figure_of(cell: &mandi_reports::reports::Cell) -> Option<i64> {
    match cell {
        mandi_reports::reports::Cell::Number { value } => Some(*value),
        mandi_reports::reports::Cell::Money { paisa } => Some(paisa.rupees()),
        _ => None,
    }
}

fn column_of(page: &AnalyticsPage, key: &str) -> usize {
    page.columns
        .iter()
        .position(|c| c.key == key)
        .unwrap_or_else(|| panic!("no column {key}"))
}

fn total_of(page: &AnalyticsPage, key: &str) -> i64 {
    page.totals
        .get(column_of(page, key))
        .and_then(figure_of)
        .unwrap_or_else(|| panic!("no total for {key}"))
}

/// The sum of a column over the rows whose item passes `keep` (a row with no item is kept out).
fn rows_of_items(page: &AnalyticsPage, key: &str, keep: impl Fn(i64) -> bool) -> i64 {
    let at = column_of(page, key);
    page.rows
        .iter()
        .filter(|row| row.key.as_ref().and_then(|k| k.item_id).is_some_and(&keep))
        .filter_map(|row| row.cells.get(at).and_then(figure_of))
        .sum()
}

/// The totals row by column key: only whole numbers and whole rupees, and not an average (a share
/// is a percent column, so it is left out by its kind). An empty cell is a column with nothing in
/// it, so it is left out too and counts as nothing when two are added.
fn additive_sums(
    columns: &[(String, mandi_reports::reports::ColumnKind)],
    totals: &[mandi_reports::reports::Cell],
) -> std::collections::BTreeMap<String, i64> {
    use mandi_reports::reports::ColumnKind;
    columns
        .iter()
        .zip(totals)
        .filter(|((key, kind), _)| {
            matches!(kind, ColumnKind::Number | ColumnKind::Money) && key != "avg_rate"
        })
        .filter_map(|((key, _), cell)| figure_of(cell).map(|v| (key.clone(), v)))
        .collect()
}

fn page_sums(page: &AnalyticsPage) -> std::collections::BTreeMap<String, i64> {
    let columns: Vec<_> = page
        .columns
        .iter()
        .map(|c| (c.key.clone(), c.kind))
        .collect();
    additive_sums(&columns, &page.totals)
}

/// Fruit and vegetable add up to the whole shop in every column, by the key of the column: a column
/// that one of them does not have, or has empty, counts as nothing. Gives how many columns were
/// compared.
fn assert_groups_add_up(
    what: &str,
    all: &std::collections::BTreeMap<String, i64>,
    fruit: &std::collections::BTreeMap<String, i64>,
    veg: &std::collections::BTreeMap<String, i64>,
    skip: &[&str],
) -> usize {
    let keys: std::collections::BTreeSet<&String> =
        all.keys().chain(fruit.keys()).chain(veg.keys()).collect();
    let mut compared = 0;
    for key in keys {
        if skip.contains(&key.as_str()) {
            continue;
        }
        let at =
            |sums: &std::collections::BTreeMap<String, i64>| sums.get(key).copied().unwrap_or(0);
        assert_eq!(at(fruit) + at(veg), at(all), "{what} column {key}");
        compared += 1;
    }
    compared
}

/// The additive columns of view A.
const VIEW_A_SUMS: [&str; 6] = [
    "qty",
    "gross",
    "seller_comm",
    "buyer_comm",
    "laga",
    "total_comm",
];

#[test]
fn view_a_for_a_group_is_the_sum_of_its_items_and_the_two_groups_make_the_whole_shop() {
    let w = day_with_a_vegetable();
    let groups = groups_of_items(&w);
    let all = view_page(&w, AnalyticsView::A, view_filters(None, None));
    let fruit = view_page(
        &w,
        AnalyticsView::A,
        view_filters(Some(ItemGroup::Fruit), None),
    );
    let veg = view_page(
        &w,
        AnalyticsView::A,
        view_filters(Some(ItemGroup::Vegetable), None),
    );
    let in_group = |group: ItemGroup| {
        let groups = &groups;
        move |id: i64| groups.get(&id) == Some(&group)
    };
    for key in VIEW_A_SUMS {
        // The group's total is the sum of the rows of its items in the whole-shop view.
        assert_eq!(
            total_of(&fruit, key),
            rows_of_items(&all, key, in_group(ItemGroup::Fruit)),
            "fruit {key}"
        );
        assert_eq!(
            total_of(&veg, key),
            rows_of_items(&all, key, in_group(ItemGroup::Vegetable)),
            "vegetable {key}"
        );
        // Its own rows add up to its own total, and fruit + vegetable is the whole shop.
        assert_eq!(
            total_of(&fruit, key),
            rows_of_items(&fruit, key, |_| true),
            "{key}"
        );
        assert_eq!(
            total_of(&fruit, key) + total_of(&veg, key),
            total_of(&all, key),
            "whole shop {key}"
        );
    }
    // The worked numbers: fruit is PRD 4.9 (qty 60, gross 41,400, comm 2,484 + 2,484, لاگا 480,
    // 5,448), vegetable is the مولی sale above, the whole shop is both.
    let want = |key: &str, fruit_n: i64, veg_n: i64| {
        assert_eq!(total_of(&fruit, key), fruit_n, "fruit {key}");
        assert_eq!(total_of(&veg, key), veg_n, "vegetable {key}");
        assert_eq!(total_of(&all, key), fruit_n + veg_n, "shop {key}");
    };
    want("qty", 60, 20);
    want("gross", 41_400, 2_000);
    want("seller_comm", 2_484, 120);
    want("buyer_comm", 2_484, 120);
    want("laga", 480, 0);
    want("total_comm", 5_448, 240);
    // Each group has only its own items as rows.
    assert_eq!(fruit.total_count, 2);
    assert_eq!(veg.total_count, 1);
    assert_eq!(all.total_count, 3);
    assert_eq!(fruit.filters.item_group, Some(ItemGroup::Fruit));
}

#[test]
fn views_d_and_g_for_a_group_add_up_to_the_whole_shop() {
    let w = day_with_a_vegetable();
    let groups = groups_of_items(&w);
    // View D: one column for each month; the totals row of each group adds up to the whole.
    let by_month = Some(AnalyticsGroupBy::Month);
    let all = view_page(&w, AnalyticsView::D, view_filters(None, by_month));
    let fruit = view_page(
        &w,
        AnalyticsView::D,
        view_filters(Some(ItemGroup::Fruit), by_month),
    );
    let veg = view_page(
        &w,
        AnalyticsView::D,
        view_filters(Some(ItemGroup::Vegetable), by_month),
    );
    let compared = assert_groups_add_up(
        "view D",
        &page_sums(&all),
        &page_sums(&fruit),
        &page_sums(&veg),
        &[],
    );
    assert!(compared >= 2, "the month and the total columns");
    assert_eq!(total_of(&fruit, "total_comm"), 5_448);
    assert_eq!(total_of(&veg, "total_comm"), 240);
    // Every row of a group view is one of its own items.
    for (page, group) in [(&fruit, ItemGroup::Fruit), (&veg, ItemGroup::Vegetable)] {
        for row in &page.rows {
            if let Some(id) = row.key.as_ref().and_then(|k| k.item_id) {
                assert_eq!(groups.get(&id), Some(&group));
            }
        }
    }
    // View G, by day: the sales and commission columns of the two groups add up to the whole.
    let by_day = Some(AnalyticsGroupBy::Day);
    let all = view_page(&w, AnalyticsView::G, view_filters(None, by_day));
    let fruit = view_page(
        &w,
        AnalyticsView::G,
        view_filters(Some(ItemGroup::Fruit), by_day),
    );
    let veg = view_page(
        &w,
        AnalyticsView::G,
        view_filters(Some(ItemGroup::Vegetable), by_day),
    );
    for key in ["gross", "commission"] {
        assert_eq!(
            total_of(&fruit, key) + total_of(&veg, key),
            total_of(&all, key),
            "view G {key}"
        );
        let sum = |page: &AnalyticsPage| {
            let at = column_of(page, key);
            page.rows
                .iter()
                .filter_map(|r| r.cells.get(at).and_then(figure_of))
                .sum::<i64>()
        };
        assert_eq!(sum(&fruit), total_of(&fruit, key), "{key}");
    }
    assert_eq!(total_of(&fruit, "gross"), 41_400);
    assert_eq!(total_of(&veg, "gross"), 2_000);
    assert_eq!(total_of(&fruit, "commission"), 5_448);
    assert_eq!(total_of(&veg, "commission"), 240);
}

#[test]
fn the_group_belongs_to_views_a_b_c_d_g_and_h_and_never_comes_with_an_item() {
    let w = day_with_a_vegetable();
    // The catalogue says which views take it.
    let with: Vec<String> = mandi_reports::analytics::catalog()
        .iter()
        .filter(|e| {
            e.filters
                .contains(&mandi_reports::analytics::AnalyticsFilterKind::ItemGroup)
        })
        .map(|e| e.id.code().to_owned())
        .collect();
    assert_eq!(with, ["A", "B", "C", "D", "G", "H"]);
    // Any other view refuses it, at `filters.item_group`.
    let refused = analytics_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: AnalyticsView::E,
            filters: view_filters(Some(ItemGroup::Fruit), None),
            offset: 0,
            limit: 100,
            sort: None,
        },
    )
    .unwrap_err();
    assert!(
        matches!(
            &refused,
            mandi_reports::ReportsError::Rejected(items)
                if items.iter().any(|i| i.field == "filters.item_group")
        ),
        "{refused:?}"
    );
    // A group and an item together are refused too.
    let both = analytics_run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: AnalyticsView::A,
            filters: AnalyticsFilters {
                item_id: Some(w.banana),
                ..view_filters(Some(ItemGroup::Fruit), None)
            },
            offset: 0,
            limit: 100,
            sort: None,
        },
    )
    .unwrap_err();
    assert!(
        matches!(
            &both,
            mandi_reports::ReportsError::Rejected(items)
                if items.iter().any(|i| i.field == "filters.item_group")
        ),
        "{both:?}"
    );
}

/// An `analytics` input as the model sends it: every field, the ones not used null.
fn analytics_input(view: &str, extra: Value) -> Value {
    let mut base = json!({
        "view": view, "from": null, "to": null, "group_by": null, "item": null, "item_group": null,
        "seller": null, "buyer": null, "city": null, "adda": null, "garden": null, "trip": null,
        "goods": null, "partner": null, "side": null, "compare": null, "sort": null, "rows": null
    });
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

#[test]
fn the_analytics_tool_takes_the_group_and_the_button_carries_it() {
    let w = day_with_a_vegetable();
    let mut s = Session::new(&w);
    let banana = s.find("banana", "item");
    let fruit = s.ok(
        "analytics",
        analytics_input("A", json!({"item_group": "fruit"})),
    );
    let veg = s.ok(
        "analytics",
        analytics_input("A", json!({"item_group": "vegetable"})),
    );
    // The tool gives the engine's own figure.
    for (id, group, want) in [
        (&fruit, ItemGroup::Fruit, 5_448),
        (&veg, ItemGroup::Vegetable, 240),
    ] {
        let page = view_page(&w, AnalyticsView::A, view_filters(Some(group), None));
        assert_eq!(total_of(&page, "total_comm"), want);
        match &s.result(id).source {
            Some(AnswerSource::Analytics { filters, view, .. }) => {
                assert_eq!(*view, AnalyticsView::A);
                assert_eq!(filters.item_group, Some(group));
                assert_eq!(filters.to_filters().item_group, Some(group));
            }
            other => panic!("no analytics button: {other:?}"),
        }
    }
    // A group and an item together, a wrong word, and a view that has no group are refused.
    let both = s.error(
        "analytics",
        analytics_input(
            "A",
            json!({"item_group": "fruit", "item": format!("{banana}.matches.0")}),
        ),
    );
    assert_eq!(both.code, "ERR_OUT_OF_RANGE");
    assert_eq!(both.field.as_deref(), Some("item_group"));
    let word = s.error(
        "analytics",
        analytics_input("A", json!({"item_group": "grain"})),
    );
    assert_eq!(word.allowed, ["fruit", "vegetable"]);
    let wrong_view = s.error(
        "analytics",
        analytics_input("E", json!({"item_group": "fruit"})),
    );
    assert_eq!(wrong_view.code, "ERR_OUT_OF_RANGE");
    assert_eq!(wrong_view.allowed, ["A", "B", "C", "D", "G", "H"]);
}

/// The lines of a report as the engine counts them for one group.
fn group_count(w: &world::World, id: ReportId, group: Option<ItemGroup>) -> i64 {
    engine_count(
        w,
        id,
        ReportFilters {
            item_group: group,
            ..ReportFilters::default()
        },
    )
}

/// The totals of a report's whole filter by column key, for one group or the whole shop.
fn report_sums(
    w: &world::World,
    id: ReportId,
    group: Option<ItemGroup>,
) -> std::collections::BTreeMap<String, i64> {
    let page = engine_run(
        w.env.ctx.db(),
        &ReportRunInput {
            id,
            filters: ReportFilters {
                item_group: group,
                ..ReportFilters::default()
            },
            offset: 0,
            limit: 100,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap();
    let columns: Vec<_> = page
        .columns
        .iter()
        .map(|c| (c.key.clone(), c.kind))
        .collect();
    additive_sums(&columns, &page.totals)
}

#[test]
fn the_sales_reports_keep_the_lines_of_one_group() {
    let w = day_with_a_vegetable();
    let mut s = Session::new(&w);
    // R2 the register: banana 2 lines, apple 1 line (fruit 3), مولی 1 line. R3 holds the cash lines
    // (the apple sale to شاہد), R4 the credit lines (banana 2, مولی 1). R27 is one row for each lot:
    // 2 fruit lots, 1 vegetable lot.
    let want = [
        (ReportId::R2, "R2", 3, 1),
        (ReportId::R3, "R3", 1, 0),
        (ReportId::R4, "R4", 2, 1),
        (ReportId::R27, "R27", 2, 1),
    ];
    for (id, code, fruit, veg) in want {
        let all = group_count(&w, id, None);
        assert_eq!(group_count(&w, id, Some(ItemGroup::Fruit)), fruit, "{code}");
        assert_eq!(
            group_count(&w, id, Some(ItemGroup::Vegetable)),
            veg,
            "{code}"
        );
        assert_eq!(
            fruit + veg,
            all,
            "{code}: fruit + vegetable is the whole shop"
        );
        // The tool gives the same count and the button carries the group.
        let read = s.ok("report", report(code, json!({"item_group": "fruit"})));
        assert_eq!(
            s.json(&read)["count"]["n"].as_i64().unwrap(),
            fruit,
            "{code}"
        );
        assert_eq!(
            button_filters(&s, &read).item_group,
            Some(ItemGroup::Fruit),
            "{code}"
        );
    }
    // R7, the item-wise sales: the totals of the two groups add up to the whole shop.
    let run = |group: Option<ItemGroup>| {
        engine_run(
            w.env.ctx.db(),
            &ReportRunInput {
                id: ReportId::R7,
                filters: ReportFilters {
                    item_group: group,
                    ..ReportFilters::default()
                },
                offset: 0,
                limit: 100,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap()
    };
    let (all, fruit, veg) = (
        run(None),
        run(Some(ItemGroup::Fruit)),
        run(Some(ItemGroup::Vegetable)),
    );
    assert_eq!(fruit.total_count, 2);
    assert_eq!(veg.total_count, 1);
    let sums = |page: &mandi_reports::reports::ReportPage| {
        let columns: Vec<_> = page
            .columns
            .iter()
            .map(|c| (c.key.clone(), c.kind))
            .collect();
        additive_sums(&columns, &page.totals)
    };
    let compared = assert_groups_add_up("R7", &sums(&all), &sums(&fruit), &sums(&veg), &[]);
    assert!(compared >= 5, "quantity, gross, commissions and لاگا");
    assert_eq!(fruit.filters.item_group, Some(ItemGroup::Fruit));
}

#[test]
fn a_report_that_has_no_group_refuses_it_and_the_group_never_comes_with_an_item() {
    let w = day_with_a_vegetable();
    let mut s = Session::new(&w);
    let banana = s.find("banana", "item");
    // R5 and R6 are not in the list of PRD v1.29.
    for id in ["R5", "R6", "R13", "R20"] {
        let error = s.error("report", report(id, json!({"item_group": "fruit"})));
        assert_eq!(error.code, "ERR_OUT_OF_RANGE", "{id}");
    }
    let both = s.error(
        "report",
        report(
            "R7",
            json!({"item_group": "fruit", "item": format!("{banana}.matches.0")}),
        ),
    );
    assert_eq!(both.code, "ERR_OUT_OF_RANGE");
    assert_eq!(both.field.as_deref(), Some("item_group"));
    let word = s.error("report", report("R7", json!({"item_group": "grain"})));
    assert_eq!(word.allowed, ["fruit", "vegetable"]);
    // The engine refuses it too, with the field.
    let engine = engine_run(
        w.env.ctx.db(),
        &ReportRunInput {
            id: ReportId::R7,
            filters: ReportFilters {
                item_id: Some(w.banana),
                item_group: Some(ItemGroup::Fruit),
                ..ReportFilters::default()
            },
            offset: 0,
            limit: 100,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap_err();
    assert!(
        matches!(
            &engine,
            mandi_reports::ReportsError::Rejected(items)
                if items.iter().any(|i| i.field == "filters.item_group")
        ),
        "{engine:?}"
    );
}

#[test]
fn the_stock_report_r10_keeps_the_lots_of_one_group() {
    // E2E-16: 5 کیلا (fruit) are left. A مولی lot of 20 with 15 sold leaves 5 more (vegetable).
    let w = worked::part_sold_world();
    let mut lot = world::bare_lot(&w, "lot-3", w.karim, w.mooli, "20");
    lot.sales = vec![world::sale("s-1", w.akram, "15", "100")];
    let draft = BikriDraft {
        header: HeaderInput {
            entry_date: "13-10-2026".into(),
            truck_no: Some("KHI-0099".into()),
            ..world::header()
        },
        lots: vec![lot],
    };
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(group_count(&w, ReportId::R10, None), 2);
    assert_eq!(group_count(&w, ReportId::R10, Some(ItemGroup::Fruit)), 1);
    assert_eq!(
        group_count(&w, ReportId::R10, Some(ItemGroup::Vegetable)),
        1
    );
    // The quantities of the two groups are the whole stock's, by column.
    let compared = assert_groups_add_up(
        "R10",
        &report_sums(&w, ReportId::R10, None),
        &report_sums(&w, ReportId::R10, Some(ItemGroup::Fruit)),
        &report_sums(&w, ReportId::R10, Some(ItemGroup::Vegetable)),
        &[],
    );
    assert!(compared >= 3, "received, sold and balance");
    let mut s = Session::new(&w);
    let read = s.ok("report", report("R10", json!({"item_group": "vegetable"})));
    assert_eq!(s.json(&read)["count"]["n"], 1);
}

#[test]
fn on_the_rich_year_the_group_is_the_sum_of_its_items_and_fruit_with_vegetable_is_the_whole() {
    let y = seed::build();
    let w = &y.w;
    let groups = groups_of_items(w);
    let fruit_g = Some(ItemGroup::Fruit);
    let veg_g = Some(ItemGroup::Vegetable);
    let in_group = |group: ItemGroup| {
        let groups = &groups;
        move |id: i64| groups.get(&id) == Some(&group)
    };
    // View A over the whole year: each group is the sum of the rows of its items.
    let all = view_page(w, AnalyticsView::A, view_filters(None, None));
    let fruit = view_page(w, AnalyticsView::A, view_filters(fruit_g, None));
    let veg = view_page(w, AnalyticsView::A, view_filters(veg_g, None));
    assert!(
        fruit.total_count >= 5 && veg.total_count >= 5,
        "five fruits, five vegetables"
    );
    assert_eq!(fruit.total_count + veg.total_count, all.total_count);
    for key in VIEW_A_SUMS {
        assert_eq!(
            total_of(&fruit, key),
            rows_of_items(&all, key, in_group(ItemGroup::Fruit)),
            "fruit {key}"
        );
        assert_eq!(
            total_of(&veg, key),
            rows_of_items(&all, key, in_group(ItemGroup::Vegetable)),
            "vegetable {key}"
        );
        assert_eq!(
            total_of(&fruit, key),
            rows_of_items(&fruit, key, |_| true),
            "{key}"
        );
        assert_eq!(
            total_of(&fruit, key) + total_of(&veg, key),
            total_of(&all, key),
            "whole shop {key}"
        );
    }
    // Views D and G, by month: the totals row of the two groups is the whole shop's. A column the
    // group view leaves out (no value) counts as nothing.
    for (view, by) in [
        (AnalyticsView::D, AnalyticsGroupBy::Month),
        (AnalyticsView::G, AnalyticsGroupBy::Month),
    ] {
        let by = Some(by);
        let all = view_page(w, view, view_filters(None, by));
        let fruit = view_page(w, view, view_filters(fruit_g, by));
        let veg = view_page(w, view, view_filters(veg_g, by));
        // The shared income, expenses and profit are left out of a sale-filtered view G.
        // The gardens column is empty under a group (PRD v1.31): no garden belongs to a group.
        let skip = ["shared_income", "shop_expenses", "profit", "gardens_result"];
        let compared = assert_groups_add_up(
            &format!("{view:?}"),
            &page_sums(&all),
            &page_sums(&fruit),
            &page_sums(&veg),
            &skip,
        );
        assert!(compared >= 3, "{view:?}");
    }
    // The same with a comparison: a window of the year and the month before it. The compared
    // figure and the change of the two groups add up to the whole shop's, for views A and D.
    let window = |group: Option<ItemGroup>| AnalyticsFilters {
        from: "2026-05-01".into(),
        to: "2026-05-31".into(),
        compare: mandi_reports::analytics::AnalyticsCompare::Previous,
        item_group: group,
        ..AnalyticsFilters::default()
    };
    for view in [AnalyticsView::A, AnalyticsView::D] {
        let all = page_sums(&view_page(w, view, window(None)));
        let fruit = page_sums(&view_page(w, view, window(fruit_g)));
        let veg = page_sums(&view_page(w, view, window(veg_g)));
        assert!(
            all.contains_key("compare") && all.contains_key("change"),
            "{view:?}"
        );
        let compared = assert_groups_add_up(&format!("{view:?} compare"), &all, &fruit, &veg, &[]);
        assert!(compared >= 4, "{view:?}");
    }
    // The sales reports: the lines of the two groups are the whole shop's.
    for id in [
        ReportId::R2,
        ReportId::R3,
        ReportId::R4,
        ReportId::R7,
        ReportId::R10,
    ] {
        assert_eq!(
            group_count(w, id, fruit_g) + group_count(w, id, veg_g),
            group_count(w, id, None),
            "{id:?}"
        );
    }
    // R27 is one row for each seller bill. A bill of a seller who sent fruit and vegetables in
    // one بکری shows in both groups, each with its own lots only, so the rows may repeat but the
    // money of the two groups is the whole shop's.
    let sums_of = |group: Option<ItemGroup>, id: ReportId| {
        let page = engine_run(
            w.env.ctx.db(),
            &ReportRunInput {
                id,
                filters: ReportFilters {
                    item_group: group,
                    ..ReportFilters::default()
                },
                offset: 0,
                limit: 100,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap();
        let columns: Vec<_> = page
            .columns
            .iter()
            .map(|c| (c.key.clone(), c.kind))
            .collect();
        additive_sums(&columns, &page.totals)
    };
    for id in [ReportId::R27, ReportId::R7] {
        let compared = assert_groups_add_up(
            &format!("{id:?}"),
            &sums_of(None, id),
            &sums_of(fruit_g, id),
            &sums_of(veg_g, id),
            &[],
        );
        assert!(compared >= 3, "{id:?}");
    }
}

#[test]
fn the_print_header_names_the_group_in_the_words_of_the_masters() {
    let w = day_with_a_vegetable();
    let facts = |list: &[mandi_reports::print::PrintFact]| {
        list.iter()
            .map(|f| (f.label_id.clone(), f.value.clone()))
            .collect::<Vec<_>>()
    };
    // A report: the group is a whole fact, with no value, in the words of the masters screen.
    for (group, label) in [
        (ItemGroup::Fruit, "masters-item-group-fruit"),
        (ItemGroup::Vegetable, "masters-item-group-vegetable"),
    ] {
        let report = mandi_reports::reports::print_data(
            w.env.ctx.db(),
            &mandi_reports::reports::ReportPrintInput {
                id: ReportId::R7,
                filters: ReportFilters {
                    item_group: Some(group),
                    ..ReportFilters::default()
                },
                language: Language::En,
                paper: None,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap();
        assert_eq!(facts(&report.filters), [(label.to_owned(), String::new())]);
        // A view: the same words.
        let view = mandi_reports::analytics::print_data(
            w.env.ctx.db(),
            &mandi_reports::analytics::AnalyticsPrintInput {
                view: AnalyticsView::A,
                filters: view_filters(Some(group), None),
                language: Language::En,
                paper: None,
                sort: None,
            },
        )
        .unwrap();
        assert_eq!(facts(&view.filters), [(label.to_owned(), String::new())]);
    }
}

/// The rows of a view with their keys and the figure of one column, for the rows that have a key.
fn keyed_figures(
    page: &AnalyticsPage,
    key: &str,
) -> Vec<(mandi_reports::analytics::AnalyticsKey, i64)> {
    let at = column_of(page, key);
    page.rows
        .iter()
        .filter_map(|row| {
            let figure = row.cells.get(at).and_then(figure_of)?;
            row.key.clone().map(|k| (k, figure))
        })
        .collect()
}

#[test]
fn views_b_and_c_for_a_group_add_up_and_the_top_buyer_of_h_is_the_top_of_c() {
    for w in [day_with_a_vegetable(), seed::build().w] {
        let by = |view: AnalyticsView, group: Option<ItemGroup>| {
            view_page(&w, view, view_filters(group, None))
        };
        for view in [AnalyticsView::B, AnalyticsView::C] {
            let all = page_sums(&by(view, None));
            let fruit = page_sums(&by(view, Some(ItemGroup::Fruit)));
            let veg = page_sums(&by(view, Some(ItemGroup::Vegetable)));
            // The count of بکری of a seller is not a sum: a بکری of fruit and vegetables is in both.
            let compared =
                assert_groups_add_up(&format!("{view:?}"), &all, &fruit, &veg, &["bikri_count"]);
            assert!(compared >= 3, "{view:?}");
        }
        // View H, by buyer side: the buyer at the top of a group is the top buyer of view C over the
        // goods of that group (view C's buyer commission and لاگا are the buyer side).
        let top_of_h = |group: ItemGroup| {
            let page = view_page(
                &w,
                AnalyticsView::H,
                AnalyticsFilters {
                    side: mandi_reports::analytics::AnalyticsSide::Buyer,
                    ..view_filters(Some(group), None)
                },
            );
            // The third heading starts the buyers; its first row is the top one.
            let mut headings = 0;
            let mut first_after = None;
            for row in &page.rows {
                if row.kind == mandi_reports::reports::RowKind::Group {
                    headings += 1;
                } else if headings == 3 && first_after.is_none() {
                    first_after = Some(row);
                }
            }
            let row = first_after.expect("a top buyer");
            let total = row
                .cells
                .get(column_of(&page, "total_comm"))
                .and_then(figure_of)
                .unwrap();
            (row.key.clone().expect("a key"), total)
        };
        for group in [ItemGroup::Fruit, ItemGroup::Vegetable] {
            let (key, total) = top_of_h(group);
            let c = by(AnalyticsView::C, Some(group));
            let buyer_side: Vec<_> = keyed_figures(&c, "buyer_comm")
                .into_iter()
                .zip(keyed_figures(&c, "laga"))
                .map(|((k, comm), (_, laga))| (k, comm + laga))
                .collect();
            let best = buyer_side.iter().map(|(_, t)| *t).max().unwrap();
            assert_eq!(total, best, "{group:?}: the top of H is the top of C");
            let same = buyer_side
                .iter()
                .find(|(k, _)| k.buyer_id == key.buyer_id && k.person_key == key.person_key)
                .map(|(_, t)| *t);
            assert_eq!(same, Some(total), "{group:?}: his own row in C");
        }
    }
}

#[test]
fn view_b_leaves_the_net_empty_when_a_garden_a_trip_or_the_goods_narrow_the_sales() {
    // HIGH finding of the money review: the lots' charges are read for every lot, so under these
    // filters the net took other sellers' charges off and showed negative nets for sellers who had
    // no part in the goods.
    let y = seed::build();
    let w = &y.w;
    let narrowed = [
        AnalyticsFilters {
            goods: mandi_reports::analytics::AnalyticsGoods::Consigned,
            ..AnalyticsFilters::default()
        },
        AnalyticsFilters {
            goods: mandi_reports::analytics::AnalyticsGoods::Garden,
            ..AnalyticsFilters::default()
        },
        AnalyticsFilters {
            goods: mandi_reports::analytics::AnalyticsGoods::Trip,
            ..AnalyticsFilters::default()
        },
        AnalyticsFilters {
            garden_id: Some(y.garden_id),
            ..AnalyticsFilters::default()
        },
        AnalyticsFilters {
            trip_id: Some(y.trip_one),
            ..AnalyticsFilters::default()
        },
        AnalyticsFilters {
            item_group: Some(ItemGroup::Fruit),
            goods: mandi_reports::analytics::AnalyticsGoods::Garden,
            ..AnalyticsFilters::default()
        },
    ];
    for filters in narrowed {
        let page = view_page(w, AnalyticsView::B, filters.clone());
        let at = column_of(&page, "net");
        assert!(
            page.rows
                .iter()
                .all(|r| r.cells.get(at) == Some(&mandi_reports::reports::Cell::Empty)),
            "{filters:?}: no row has a net"
        );
        assert_eq!(
            page.totals.get(at),
            Some(&mandi_reports::reports::Cell::Empty),
            "{filters:?}: no net in the totals"
        );
    }
    // With no such filter the net is given, as before.
    let plain = view_page(w, AnalyticsView::B, AnalyticsFilters::default());
    let at = column_of(&plain, "net");
    assert!(plain.rows.iter().any(|r| matches!(
        r.cells.get(at),
        Some(mandi_reports::reports::Cell::Money { .. })
    )));
}

#[test]
fn view_g_leaves_the_gardens_column_empty_under_a_group_and_r27_says_it_shows_part_of_a_bill() {
    // PRD v1.31: no garden belongs to an item group, so view G has no gardens figure under one.
    let y = seed::build();
    let w = &y.w;
    let by = Some(AnalyticsGroupBy::Month);
    let plain = view_page(w, AnalyticsView::G, view_filters(None, by));
    let at = column_of(&plain, "gardens_result");
    assert!(
        plain.rows.iter().any(|r| matches!(
            r.cells.get(at),
            Some(mandi_reports::reports::Cell::Money { .. })
        )),
        "without a group the gardens column has figures"
    );
    for group in [ItemGroup::Fruit, ItemGroup::Vegetable] {
        let page = view_page(w, AnalyticsView::G, view_filters(Some(group), by));
        let at = column_of(&page, "gardens_result");
        assert!(
            page.rows
                .iter()
                .all(|r| r.cells.get(at) == Some(&mandi_reports::reports::Cell::Empty)),
            "{group:?}: no gardens figure in any row"
        );
        assert_eq!(
            page.totals.get(at),
            Some(&mandi_reports::reports::Cell::Empty),
            "{group:?}"
        );
        // The garden's sales money by month, used by the estimates, follows the same rule.
        let months = mandi_reports::analytics::garden_sales_months(
            w.env.ctx.db(),
            &view_filters(Some(group), None),
        )
        .unwrap();
        assert!(months.is_empty(), "{group:?}");
    }
    assert!(
        !mandi_reports::analytics::garden_sales_months(w.env.ctx.db(), &view_filters(None, None))
            .unwrap()
            .is_empty()
    );
    // R27 under a group carries the note in its print header; the other group reports do not.
    let facts = |id: ReportId, group: Option<ItemGroup>| {
        mandi_reports::reports::print_data(
            w.env.ctx.db(),
            &mandi_reports::reports::ReportPrintInput {
                id,
                filters: ReportFilters {
                    item_group: group,
                    ..ReportFilters::default()
                },
                language: Language::En,
                paper: None,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap()
        .filters
        .into_iter()
        .map(|f| (f.label_id, f.value))
        .collect::<Vec<_>>()
    };
    assert_eq!(
        facts(ReportId::R27, Some(ItemGroup::Fruit)),
        [
            ("masters-item-group-fruit".to_owned(), String::new()),
            ("report-r27-group-note".to_owned(), String::new())
        ]
    );
    assert_eq!(
        facts(ReportId::R7, Some(ItemGroup::Fruit)),
        [("masters-item-group-fruit".to_owned(), String::new())]
    );
    assert!(facts(ReportId::R27, None).is_empty());
}
