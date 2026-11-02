//! The 8 tools on the PRD worked day (PRD FR-7.87, Code Standards Q1 to Q3, Q5): their inputs are
//! read strictly, each filter belongs to the report it is used on, a refusal is an error result and
//! not a wrong figure, and the calculator and estimator follow the rules for the kind of each
//! input. The tools are called directly, as the loop calls them.
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
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::domain::figures::{NumberUnit, QuestionNumber};
use serde_json::{Value, json};

/// A session of tool calls on one world: results get their ids and stay for the next call.
struct Session<'a> {
    world: &'a world::World,
    words: Words,
    numbers: Vec<QuestionNumber>,
    state: State,
    locked: bool,
    today: &'static str,
}

fn number(n: i64, text: &str, unit: NumberUnit) -> QuestionNumber {
    QuestionNumber {
        number: n,
        text: text.to_owned(),
        unit,
        bps: None,
    }
}

impl<'a> Session<'a> {
    fn new(world: &'a world::World) -> Self {
        Self {
            world,
            words: Words::new().unwrap(),
            numbers: Vec::new(),
            state: State::default(),
            locked: false,
            today: "2026-10-10",
        }
    }

    fn with_today(mut self, today: &'static str) -> Self {
        self.today = today;
        self
    }

    fn with_numbers(mut self, numbers: Vec<QuestionNumber>) -> Self {
        self.numbers = numbers;
        self
    }

    /// Runs a tool; on success the result is kept and its id given back.
    fn run(&mut self, tool: &str, args: Value) -> Result<String, Stop> {
        let env = Env {
            ctx: &self.world.env.ctx,
            actor: &self.world.owner,
            words: &self.words,
            language: Language::Ur,
            today: BusinessDate::parse_iso(self.today).unwrap(),
            partner_locked: self.locked,
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

    fn error(&mut self, tool: &str, args: Value) -> ToolError {
        match self.run(tool, args) {
            Err(Stop::Error(e)) => e,
            Err(Stop::Locked) => panic!("locked, not an error"),
            Ok(id) => panic!("{tool} should have been refused, gave {id}"),
        }
    }

    fn node(&self, id: &str) -> &Node {
        &self.state.find(id).unwrap().node
    }

    fn json(&self, id: &str) -> Value {
        tool_result_json(self.state.find(id).unwrap(), &self.words)
    }
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

fn day() -> world::World {
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    w
}

fn rs(v: &Value) -> i64 {
    v["rs"].as_i64().unwrap()
}

// ---- find -------------------------------------------------------------------------------------

#[test]
fn find_reads_every_role_and_gives_a_checked_balance_for_accounts() {
    let w = day();
    let mut s = Session::new(&w);
    let buyer = s
        .run("find", json!({"text": "akram", "role": "buyer"}))
        .unwrap();
    let j = s.json(&buyer);
    assert_eq!(j["matches"].as_array().unwrap().len(), 1);
    let m = &j["matches"][0];
    assert_eq!(m["key"], "a20");
    assert_eq!(m["type"], "buyer");
    assert_eq!(
        (rs(&m["balance"]), m["balance"]["side"].as_str()),
        (2_880, Some("بنام"))
    );
    assert_eq!(m["balance"]["checked"], true);
    assert_eq!(m["balance"]["places"]["party_card"], 2_880);
    assert_eq!(m["balance"]["places"]["R13"], 2_880);
    // A seller is not found as a buyer, and any_party finds both.
    let none = s
        .run("find", json!({"text": "karim", "role": "buyer"}))
        .unwrap();
    assert!(s.json(&none)["matches"].as_array().unwrap().is_empty());
    let any = s
        .run("find", json!({"text": "karim", "role": "any_party"}))
        .unwrap();
    assert_eq!(s.json(&any)["matches"][0]["key"], "a19");
    // Items, partners and the lists with no match are lists, not errors.
    let item = s
        .run("find", json!({"text": "کیلا", "role": "item"}))
        .unwrap();
    assert_eq!(s.json(&item)["matches"][0]["key"], "i1");
    let partner = s
        .run("find", json!({"text": "الف", "role": "partner"}))
        .unwrap();
    assert_eq!(s.json(&partner)["matches"][0]["key"], "s1");
    for role in [
        "city",
        "adda",
        "collector",
        "trip",
        "garden",
        "money_lent",
        "separate_khata",
        "pukhta",
    ] {
        let id = s.run("find", json!({"text": "zzz", "role": role})).unwrap();
        assert!(
            s.json(&id)["matches"].as_array().unwrap().is_empty(),
            "{role}"
        );
    }
    // A misspelled name is forgiven; no phone, address or father's name is in the result.
    let close = s
        .run("find", json!({"text": "rahim", "role": "seller"}))
        .unwrap();
    let text = s.json(&close).to_string();
    for private in [
        worked::PRIVATE_PHONE,
        worked::PRIVATE_ADDRESS,
        "Haji Sahib",
        "حاجی صاحب",
        worked::PRIVATE_NOTE,
    ] {
        assert!(!text.contains(private), "{private}");
    }
}

#[test]
fn find_and_the_other_tools_refuse_what_they_cannot_read() {
    let w = day();
    let mut s = Session::new(&w);
    assert_eq!(
        s.error("find", json!({"text": "  ", "role": "buyer"})).code,
        "ERR_REQUIRED"
    );
    assert_eq!(
        s.error("find", json!({"text": "a", "role": "robber"})).code,
        "ERR_FORMAT_INVALID"
    );
    assert_eq!(
        s.error("find", json!({"text": "a", "role": "buyer", "x": 1}))
            .code,
        "ERR_FORMAT_INVALID"
    );
    assert_eq!(s.error("nonsense", json!({})).code, "ERR_UNKNOWN_TOOL");
    // A key the conversation never showed is not a record.
    assert_eq!(
        s.error("party_card", json!({"party": "a20"})).code,
        "ERR_UNKNOWN_REFERENCE"
    );
    assert_eq!(
        s.error("party_card", json!({"party": "r7.matches.0"})).code,
        "ERR_UNKNOWN_REFERENCE"
    );
    let found = s
        .run("find", json!({"text": "کیلا", "role": "item"}))
        .unwrap();
    assert_eq!(found, "r1");
    // An item is not a party.
    assert_eq!(
        s.error("party_card", json!({"party": "i1"})).code,
        "ERR_FORMAT_INVALID"
    );
}

// ---- party_card -------------------------------------------------------------------------------

#[test]
fn party_card_gives_the_card_and_the_oldest_bill() {
    let w = day();
    let mut s = Session::new(&w);
    s.run("find", json!({"text": "akram", "role": "buyer"}))
        .unwrap();
    let card = s
        .run("party_card", json!({"party": "r1.matches.0"}))
        .unwrap();
    let j = s.json(&card);
    assert_eq!(rs(&j["balance"]), 2_880);
    assert_eq!(j["balance"]["checked"], true);
    assert_eq!(rs(&j["oldest_bill"]["open"]), 2_880);
    assert_eq!(j["oldest_bill"]["days"]["n"], 0);
    // A seller's card has his payable.
    s.run("find", json!({"text": "karim", "role": "seller"}))
        .unwrap();
    let karim = s
        .run("party_card", json!({"party": "r3.matches.0"}))
        .unwrap();
    assert_eq!(rs(&s.json(&karim)["payable"]), 10_160);
}

// ---- report -----------------------------------------------------------------------------------

#[test]
fn report_filters_belong_to_the_report_and_a_wrong_one_is_an_error() {
    let w = day();
    let mut s = Session::new(&w);
    s.run("find", json!({"text": "akram", "role": "buyer"}))
        .unwrap();
    s.run("find", json!({"text": "الف", "role": "partner"}))
        .unwrap();
    // A filter the report does not have.
    for (id, extra) in [
        ("R2", json!({"partner": "r2.matches.0"})),
        ("R12", json!({"party": "r1.matches.0"})),
        ("R20", json!({"from": "2026-10-01"})),
        (
            "R13",
            json!({"as_of": "2026-10-10", "party": "r1.matches.0"}),
        ),
        ("R2", json!({"collector": "r1.matches.0"})),
        ("R2", json!({"trip": "r1.matches.0"})),
    ] {
        assert_eq!(
            s.error("report", report(id, extra)).code,
            "ERR_OUT_OF_RANGE",
            "{id}"
        );
    }
    // The report's own required filters.
    assert_eq!(
        s.error("report", report("R13", json!({}))).code,
        "ERR_REQUIRED"
    );
    assert_eq!(
        s.error("report", report("R1", json!({}))).code,
        "ERR_REQUIRED"
    );
    assert_eq!(
        s.error("report", report("R99", json!({}))).code,
        "ERR_OUT_OF_RANGE"
    );
    // A date that does not exist, a page that is too big, too many sort keys.
    assert_eq!(
        s.error("report", report("R2", json!({"from": "2026-02-31"})))
            .code,
        "ERR_DATE_INVALID"
    );
    assert_eq!(
        s.error("report", report("R2", json!({"rows": 101}))).code,
        "ERR_OUT_OF_RANGE"
    );
    assert_eq!(
        s.error("report", report("R2", json!({"rows": 0}))).code,
        "ERR_OUT_OF_RANGE"
    );
    let keys = json!([{"column": "a", "order": "asc"}, {"column": "b", "order": "asc"}, {"column": "c", "order": "asc"}]);
    assert_eq!(
        s.error("report", report("R2", json!({"sort": keys}))).code,
        "ERR_OUT_OF_RANGE"
    );
    // A column the report does not have.
    let bad = json!({"column": "nothing", "op": "gt", "value": "r1.matches.0.balance"});
    assert_eq!(
        s.error("report", report("R20", json!({"where": bad}))).code,
        "ERR_OUT_OF_RANGE"
    );
    // An item is not a party.
    s.run("find", json!({"text": "کیلا", "role": "item"}))
        .unwrap();
    assert_eq!(
        s.error("report", report("R13", json!({"party": "r3.matches.0"})))
            .code,
        "ERR_FORMAT_INVALID"
    );
}

#[test]
fn report_where_and_sort_apply_to_the_reports_own_rows_with_matched_totals_from_the_calculator() {
    let w = day();
    let mut s = Session::new(&w).with_numbers(vec![number(10_000, "10,000", NumberUnit::Plain)]);
    // Example 5 of the prompt: buyers owing more than 10,000, biggest first.
    let id = s
        .run(
            "report",
            report(
                "R20",
                json!({"where": {"column": "balance", "op": "gt", "value": "q.n1"},
                       "sort": [{"column": "balance", "order": "desc"}]}),
            ),
        )
        .unwrap();
    let j = s.json(&id);
    assert_eq!(
        j["count"]["n"], 2,
        "count is over the whole filter, not the matches"
    );
    assert_eq!(j["matched"]["count"]["n"], 1);
    assert_eq!(rs(&j["matched"]["totals"]["balance"]), 18_684);
    let rows = j["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["key"], "a21");
    assert_eq!(rows[0]["name"]["en"], "Arshad Multani");
    // Each row's balance has its side and both places.
    assert_eq!(rows[0]["balance"]["side"], "بنام");
    assert_eq!(rows[0]["balance"]["checked"], true);
    assert_eq!(rs(&j["totals"]["balance"]), 21_564);
    // Two sort keys: days (all 0), then the biggest balance first.
    let two = s
        .run(
            "report",
            report("R20", json!({"sort": [{"column": "days", "order": "desc"}, {"column": "balance", "order": "desc"}]})),
        )
        .unwrap();
    let listed = s.json(&two);
    let keys: Vec<&str> = listed["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["key"].as_str().unwrap())
        .collect();
    assert_eq!(keys, ["a21", "a20"]);
    // A value of the wrong kind for the column is refused.
    let mut s = Session::new(&w).with_numbers(vec![number(6, "6", NumberUnit::Months)]);
    let bad = json!({"column": "balance", "op": "gt", "value": "q.n1"});
    // A months number is a count; the balance is money: the kinds differ, so it is refused (money
    // review R3). The same number against the days column is a count and reads.
    assert_eq!(
        s.error("report", report("R20", json!({"where": bad}))).code,
        "ERR_FORMAT_INVALID"
    );
    assert!(
        s.run(
            "report",
            report(
                "R20",
                json!({"where": {"column": "days", "op": "gt", "value": "q.n1"}})
            )
        )
        .is_ok()
    );
    let unknown = json!({"column": "balance", "op": "gt", "value": "q.n9"});
    assert_eq!(
        s.error("report", report("R20", json!({"where": unknown})))
            .code,
        "ERR_UNKNOWN_REFERENCE"
    );
}

#[test]
fn ledgers_come_newest_first_with_stable_row_keys_and_the_checked_closing() {
    let w = day();
    let mut s = Session::new(&w);
    s.run("find", json!({"text": "rahim", "role": "seller"}))
        .unwrap();
    let id = s
        .run("report", report("R13", json!({"party": "r1.matches.0"})))
        .unwrap();
    let j = s.json(&id);
    let rows = j["rows"].as_array().unwrap();
    // The payment of 10-10-26 first; the advance of 01-09-26 last.
    assert_eq!(rs(&rows[0]["debit"]), 4_316);
    assert_eq!(rs(&rows.last().unwrap()["debit"]), 15_000);
    // Row keys are the records', never the place; a record with two lines gets a second key.
    let keys: Vec<&str> = rows.iter().map(|r| r["key"].as_str().unwrap()).collect();
    let unique: std::collections::BTreeSet<&&str> = keys.iter().collect();
    assert_eq!(unique.len(), keys.len(), "{keys:?}");
    // The closing balance carries both places; no narration or other typed text is in the rows.
    assert_eq!(j["summary"]["closing"]["checked"], true);
    assert!(rows.iter().all(|r| r.get("narration").is_none()));
    // A private note typed on the account never appears.
    assert!(!j.to_string().contains(worked::PRIVATE_NOTE));
}

#[test]
fn a_big_result_is_cut_from_the_end_and_says_so() {
    let w = day();
    let mut s = Session::new(&w);
    // R18 (the trial balance) has the most rows of the worked day; ask for the most.
    let id = s
        .run("report", report("R18", json!({"rows": 100})))
        .unwrap();
    let j = s.json(&id);
    assert!(j.to_string().len() <= mandi_assistant::limits::MAX_RESULT_BYTES);
    assert!(j["rows"].as_array().unwrap().len() > 5);
    assert_eq!(j["rows_cut"].as_bool(), Some(false));
}

#[test]
fn r31_with_no_partner_gives_one_row_for_each_partner_and_the_lock_pauses_it() {
    let w = day();
    let mut s = Session::new(&w);
    let id = s.run("report", report("R31", json!({}))).unwrap();
    let j = s.json(&id);
    let rows = j["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(
        (rows[0]["key"].as_str(), rs(&rows[0]["total"])),
        (Some("s1"), -1_004)
    );
    assert_eq!(rs(&rows[1]["total"]), 746);
    assert!(rows.iter().all(|r| r["total"]["checked"] == true));
    // Behind the code: the call waits.
    let mut locked = Session::new(&w);
    locked.locked = true;
    assert!(matches!(
        locked.run("report", report("R31", json!({}))),
        Err(Stop::Locked)
    ));
    assert!(matches!(
        locked.run("analytics", analytics("F", json!({}))),
        Err(Stop::Locked)
    ));
    // R29 runs without the partner parts while locked.
    let r29 = locked.run("report", report("R29", json!({}))).unwrap();
    assert_eq!(locked.json(&r29)["partner_parts"], "locked");
}

// ---- analytics --------------------------------------------------------------------------------

#[test]
fn analytics_compare_gives_earlier_and_difference_and_a_sort_on_the_signed_difference() {
    let w = worked::months_world(&[
        (8, true, 100, 1_000),
        (8, false, 50, 2_000),
        (9, true, 60, 1_000),
        (9, false, 50, 2_000),
    ]);
    let mut s = Session::new(&w);
    let period = s.run("period", json!({"kind": "last_month", "month": null, "day": null, "from": null, "to": null, "n": null})).unwrap();
    assert_eq!(s.json(&period)["label"]["month"], "2026-09");
    let asc = s
        .run(
            "analytics",
            analytics(
                "A",
                json!({"from": "r1.from", "to": "r1.to", "compare": "previous",
                       "sort": [{"column": "difference.total_comm", "order": "asc"}]}),
            ),
        )
        .unwrap();
    let j = s.json(&asc);
    let rows = j["rows"].as_array().unwrap();
    // The banana fell by 5,120 (12,800 to 7,680): the biggest fall comes first with "asc".
    assert_eq!(rows[0]["key"], "i1");
    assert_eq!(rs(&rows[0]["total_comm"]), 7_680);
    assert_eq!(rs(&rows[0]["earlier"]["total_comm"]), 12_800);
    assert_eq!(rs(&rows[0]["difference"]["total_comm"]), 5_120);
    assert_eq!(rows[0]["difference"]["total_comm"]["dir"], "down");
    assert_eq!(rows[1]["difference"]["total_comm"]["dir"], "same");
    assert_eq!(rs(&j["compare"]["totals"]["total_comm"]), 25_200);
    assert_eq!(j["compare"]["difference"]["total_comm"]["dir"], "down");
    // "desc" puts the unchanged one before the fall.
    let desc = s
        .run(
            "analytics",
            analytics(
                "A",
                json!({"from": "r1.from", "to": "r1.to", "compare": "previous",
                       "sort": [{"column": "difference.total_comm", "order": "desc"}]}),
            ),
        )
        .unwrap();
    assert_eq!(s.json(&desc)["rows"][0]["key"], "i2");
    // The quantity is called `nag` and the first column `name`.
    assert_eq!(rows[0]["nag"]["n"], 60);
    assert_eq!(rows[0]["name"]["en"], "Banana");
    // A filter the view does not have; a trend by month is also a series.
    s.run("find", json!({"text": "الف", "role": "partner"}))
        .unwrap();
    assert_eq!(
        s.error(
            "analytics",
            analytics("C", json!({"partner": "r4.matches.0"}))
        )
        .code,
        "ERR_OUT_OF_RANGE"
    );
    let trend = s
        .run("analytics", analytics("G", json!({"group_by": "month"})))
        .unwrap();
    assert!(
        s.json(&trend)["series"]
            .as_array()
            .is_some_and(|a| a.len() >= 2)
    );
}

// ---- tiles ------------------------------------------------------------------------------------

#[test]
fn tiles_carry_both_places_where_there_are_two() {
    let w = day();
    let mut s = Session::new(&w);
    let id = s.run("tiles", json!({})).unwrap();
    let j = s.json(&id);
    for (tile, value, places) in [
        ("today_sales", 41_400, vec!["tile", "view_A"]),
        ("today_commission", 5_448, vec!["tile", "view_A"]),
        ("cash_in_hand", 58_484, vec!["tile", "R12"]),
        ("buyers_dues", 21_564, vec!["tile", "R20"]),
        ("sellers_dues", 0, vec!["tile", "R24"]),
        ("dead_amount", 0, vec!["tile", "R38"]),
        ("total_dues", 21_564, vec!["tile", "parts"]),
    ] {
        let amount = &j[tile]["amount"];
        assert_eq!(rs(amount), value, "{tile}");
        assert_eq!(amount["checked"], true, "{tile}");
        for place in places {
            assert!(
                amount["places"].get(place).is_some(),
                "{tile} has no place {place}"
            );
        }
    }
    assert_eq!(j["stock"]["qty"]["n"], 0);
    assert_eq!(j["last_backup"]["cloud_on"], false);
}

// ---- calculate --------------------------------------------------------------------------------

fn result(s: &Session<'_>, id: &str) -> Value {
    s.json(id)["result"].clone()
}

#[test]
fn calculate_keeps_the_kind_of_its_inputs_and_gives_a_difference_its_size_and_direction() {
    let w = day();
    let mut s = Session::new(&w).with_numbers(vec![
        number(48_000, "48,000", NumberUnit::Plain),
        number(51_000, "51,000", NumberUnit::Plain),
        number(3, "3", NumberUnit::Months),
        number(300_000, "3 lakh", NumberUnit::Amount),
        number(10, "10%", NumberUnit::Percent),
    ]);
    // "48,000 to 51,000": 3,000 more (size and direction up).
    let up = s
        .run(
            "calculate",
            json!({"op": "difference", "inputs": ["q.n2", "q.n1"]}),
        )
        .unwrap();
    assert_eq!(result(&s, &up), json!({"n": 3_000, "dir": "up"}));
    // The other way round is 3,000 less.
    let down = s
        .run(
            "calculate",
            json!({"op": "difference", "inputs": ["q.n1", "q.n2"]}),
        )
        .unwrap();
    assert_eq!(result(&s, &down), json!({"n": 3_000, "dir": "down"}));
    // Two balances: their sum is a balance on the debit side, checked because both were.
    s.run("find", json!({"text": "akram", "role": "buyer"}))
        .unwrap();
    s.run("find", json!({"text": "arshad", "role": "buyer"}))
        .unwrap();
    let sum = s
        .run(
            "calculate",
            json!({"op": "sum", "inputs": ["r1.matches.0.balance", "r2.matches.0.balance"]}),
        )
        .unwrap();
    let j = result(&s, &sum);
    assert_eq!(
        (rs(&j), j["side"].as_str(), j["checked"].as_bool()),
        (21_564, Some("بنام"), Some(true))
    );
    // A plain number beside money is rupees; "3 lakh" is an amount.
    let plus = s
        .run(
            "calculate",
            json!({"op": "sum", "inputs": ["r1.matches.0.balance", "q.n1"]}),
        )
        .unwrap();
    assert_eq!(rs(&result(&s, &plus)), 50_880);
    // A plain number beside a balance takes its kind: 2,880 is 6.00% of 48,000.
    let share = s
        .run(
            "calculate",
            json!({"op": "percent_of", "inputs": ["r1.matches.0.balance", "q.n1"]}),
        )
        .unwrap();
    assert_eq!(result(&s, &share), json!({"pct": "6.00"}));
    // "3 lakh" is an amount, and an amount does not mix with a balance.
    assert_eq!(
        s.error(
            "calculate",
            json!({"op": "percent_of", "inputs": ["r1.matches.0.balance", "q.n4"]})
        )
        .code,
        "ERR_OUT_OF_RANGE"
    );
    let amounts = s
        .run(
            "calculate",
            json!({"op": "percent_of", "inputs": ["q.n1", "q.n4"]}),
        )
        .unwrap();
    assert_eq!(result(&s, &amounts), json!({"pct": "16.00"}));
    // A percent is never an input; money and counts do not mix; a base of 0 is refused.
    assert_eq!(
        s.error(
            "calculate",
            json!({"op": "sum", "inputs": ["q.n5", "q.n1"]})
        )
        .code,
        "ERR_OUT_OF_RANGE"
    );
    assert_eq!(
        s.error(
            "calculate",
            json!({"op": "sum", "inputs": ["r1.matches.0.balance", "q.n3"]})
        )
        .code,
        "ERR_OUT_OF_RANGE"
    );
    // A path that does not exist, and no inputs at all.
    assert_eq!(
        s.error("calculate", json!({"op": "sum", "inputs": ["r9.x"]}))
            .code,
        "ERR_UNKNOWN_REFERENCE"
    );
    assert_eq!(
        s.error("calculate", json!({"op": "sum", "inputs": []}))
            .code,
        "ERR_NO_INPUTS"
    );
}

#[test]
fn a_sum_over_every_row_is_refused_when_the_rows_were_cut() {
    let w = day();
    let mut s = Session::new(&w);
    // All rows are there: summed.
    let r20 = s.run("report", report("R20", json!({}))).unwrap();
    assert_eq!(r20, "r1");
    let sum = s
        .run(
            "calculate",
            json!({"op": "sum", "inputs": ["r1.rows.*.balance"]}),
        )
        .unwrap();
    assert_eq!(rs(&result(&s, &sum)), 21_564);
    // Only one row was asked for out of two: the sum would be wrong, so it is refused.
    let short = s.run("report", report("R20", json!({"rows": 1}))).unwrap();
    assert_eq!(short, "r2");
    let refused = s.error(
        "calculate",
        json!({"op": "sum", "inputs": ["r2.rows.*.balance"]}),
    );
    assert_eq!(refused.code, "ERR_OUT_OF_RANGE");
}

// ---- estimate ---------------------------------------------------------------------------------

#[test]
fn estimate_months_ahead_must_be_a_number_of_the_question() {
    let w = worked::history_world(&[
        (4, 180, 1_600),
        (5, 150, 2_100),
        (6, 120, 2_850),
        (7, 225, 1_600),
        (8, 150, 2_600),
        (9, 173, 2_390),
    ]);
    let args = |ahead: i64| json!({"measure": "commission_all", "item": null, "seller": null, "buyer": null, "garden": null, "trip": null, "ahead": ahead, "target": null});
    // No months in the question: 6 is the default, anything else is refused.
    let mut s = Session::new(&w);
    assert!(s.run("estimate", args(6)).is_ok());
    assert_eq!(s.error("estimate", args(9)).code, "ERR_OUT_OF_RANGE");
    assert_eq!(s.error("estimate", args(13)).code, "ERR_OUT_OF_RANGE");
    // With "agle 3 mahine" in the question, 3 is allowed and 6 is not.
    let mut s = Session::new(&w).with_numbers(vec![number(3, "3", NumberUnit::Months)]);
    let id = s.run("estimate", args(3)).unwrap();
    assert_eq!(s.json(&id)["ahead_count"]["n"], 3);
    assert_eq!(s.error("estimate", args(6)).code, "ERR_OUT_OF_RANGE");
    // A measure the books give no monthly series of is refused, not guessed.
    let mut s = Session::new(&w);
    let laga = json!({"measure": "laga", "item": null, "seller": null, "buyer": null, "garden": null, "trip": null, "ahead": 6, "target": null});
    assert_eq!(s.error("estimate", laga).code, "ERR_OUT_OF_RANGE");
    // Seller commission only is a different series from all commission.
    let seller = json!({"measure": "commission_seller", "item": null, "seller": null, "buyer": null, "garden": null, "trip": null, "ahead": 6, "target": null});
    let id = s.run("estimate", seller).unwrap();
    // Half of the six months' 12% is the seller's 6%: the last 3 months of seller commission.
    let pace = s.json(&id)["same_pace"]["per_month"].clone();
    assert!(rs(&pace) > 0 && rs(&pace) < 48_000);
    // The history is oldest first with a key for each month.
    let h = s.json(&id)["history"].clone();
    assert_eq!(h[0]["key"], "m202604");
    assert_eq!(h.as_array().unwrap().len(), 6);
}

// ---- period -----------------------------------------------------------------------------------

#[test]
fn period_words_need_the_right_numbers_of_the_question() {
    let w = day();
    let p = |kind: &str, n: Option<i64>| json!({"kind": kind, "month": null, "day": null, "from": null, "to": null, "n": n});
    let mut s = Session::new(&w).with_numbers(vec![
        number(10, "10", NumberUnit::Days),
        number(10_000, "10,000", NumberUnit::Amount),
    ]);
    let ok = s.run("period", p("last_n_days", Some(10))).unwrap();
    assert_eq!(s.json(&ok)["from"]["date"], "2026-09-30");
    // 10,000 is an amount, not days: "pichle 10 din mein 10,000 se zyada" must not use it.
    assert_eq!(
        s.error("period", p("last_n_days", Some(10_000))).code,
        "ERR_OUT_OF_RANGE"
    );
    // No number of the question has months after it.
    assert_eq!(
        s.error("period", p("last_n_months", Some(10))).code,
        "ERR_OUT_OF_RANGE"
    );
    // A day that does not exist.
    let day =
        json!({"kind": "day", "month": null, "day": "31-02", "from": null, "to": null, "n": null});
    assert_eq!(s.error("period", day).code, "ERR_DATE_INVALID");
    // Yesterday, last week and "up to now".
    for kind in [
        "today",
        "yesterday",
        "this_week",
        "last_week",
        "this_month",
        "last_month",
        "this_year",
        "last_year",
        "up_to_now",
    ] {
        assert!(s.run("period", p(kind, None)).is_ok(), "{kind}");
    }
}

// ---- values the model reads ---------------------------------------------------------------

#[test]
fn every_value_in_a_result_is_a_small_typed_object() {
    let w = day();
    let mut s = Session::new(&w);
    let id = s.run("tiles", json!({})).unwrap();
    // No bare number or string stands for a figure: a figure is an object with `rs`, `n`, `pct`
    // or `date` (D6-system-prompt 5.1).
    fn walk(v: &Value, path: &str, bad: &mut Vec<String>) {
        match v {
            Value::Object(map) => map
                .iter()
                .for_each(|(k, x)| walk(x, &format!("{path}.{k}"), bad)),
            Value::Array(items) => items
                .iter()
                .enumerate()
                .for_each(|(i, x)| walk(x, &format!("{path}[{i}]"), bad)),
            Value::Number(_)
                if !path.ends_with(".rs")
                    && !path.ends_with(".n")
                    && !path.contains("places")
                    && !path.contains("cloud") =>
            {
                bad.push(path.to_owned());
            }
            _ => {}
        }
    }
    let mut bad = Vec::new();
    walk(&s.json(&id), "", &mut bad);
    assert!(bad.is_empty(), "bare numbers: {bad:?}");
    // The node tree also tells a figure from text the model may not cite.
    let Node::Obj(fields) = s.node(&id) else {
        panic!()
    };
    assert!(fields.iter().any(|(_, n)| matches!(n, Node::Obj(_))));
    let _ = Val::Count(0);
}

// ---- the money review of plan 48 T3 --------------------------------------------------------

fn working_of<'s>(s: &'s Session<'_>, id: &str) -> &'s mandi_assistant::answer::AnswerWorking {
    s.state
        .find(id)
        .and_then(|r| r.working.as_ref())
        .expect("a working")
}

#[test]
fn a_check_mark_needs_every_input_checked_and_a_number_takes_the_side_beside_it() {
    // C1 and C2. Akram owes 2,880 (checked); Karim is owed 10,160 (checked); 48,000 is a number
    // the Owner typed, never read from the books.
    let w = day();
    let mut s = Session::new(&w).with_numbers(vec![number(48_000, "48,000", NumberUnit::Plain)]);
    s.run("find", json!({"text": "akram", "role": "buyer"}))
        .unwrap();
    s.run("find", json!({"text": "karim", "role": "seller"}))
        .unwrap();
    let mut calc = |op: &str, inputs: Value| {
        let id = s.run("calculate", json!({"op": op, "inputs": inputs}));
        id.map(|id| result(&s, &id))
    };
    // A typed number beside a checked balance: 50,880 بنام and no check mark.
    let j = calc("sum", json!(["r1.matches.0.balance", "q.n1"])).unwrap();
    assert_eq!(
        (rs(&j), j["side"].as_str(), j["checked"].as_bool()),
        (50_880, Some("بنام"), None)
    );
    // Beside a credit balance the number is a credit: 10,160 + 48,000 owed to him, no check mark.
    let j = calc("sum", json!(["r2.matches.0.balance", "q.n1"])).unwrap();
    assert_eq!(
        (rs(&j), j["side"].as_str(), j["checked"].as_bool()),
        (58_160, Some("جمع"), None)
    );
    // 10,160 as a percent of 48,000 (both credit): 21.17%.
    let j = calc("percent_of", json!(["r2.matches.0.balance", "q.n1"])).unwrap();
    assert_eq!(j, json!({"pct": "21.17"}));
    // A debit and a credit beside a number: the side of the number would be a guess.
    let error = calc(
        "sum",
        json!(["r1.matches.0.balance", "r2.matches.0.balance", "q.n1"]),
    )
    .map(|_| ())
    .unwrap_err();
    let Stop::Error(error) = error else {
        panic!("locked")
    };
    assert_eq!(error.code, "ERR_OUT_OF_RANGE");
    // Two balances that were both checked still give a checked sum: 2,880 − 10,160.
    let j = calc(
        "sum",
        json!(["r1.matches.0.balance", "r2.matches.0.balance"]),
    )
    .unwrap();
    assert_eq!(
        (rs(&j), j["side"].as_str(), j["checked"].as_bool()),
        (7_280, Some("جمع"), Some(true))
    );
}

#[test]
fn an_average_sends_how_many_figures_it_was_over_as_its_last_input() {
    use mandi_reports::reports::Cell;
    let w = day();
    let mut s = Session::new(&w);
    s.run("find", json!({"text": "akram", "role": "buyer"}))
        .unwrap();
    s.run("find", json!({"text": "arshad", "role": "buyer"}))
        .unwrap();
    let id = s
        .run(
            "calculate",
            json!({"op": "average", "inputs": ["r1.matches.0.balance", "r2.matches.0.balance"]}),
        )
        .unwrap();
    let j = result(&s, &id);
    assert_eq!((rs(&j), j["side"].as_str()), (10_782, Some("بنام")));
    // (2,880 + 18,684) ÷ 2: the two balances, then the count 2.
    let working = working_of(&s, &id);
    let step = &working.steps[0];
    assert_eq!(step.inputs.len(), 3);
    assert_eq!(step.inputs.last(), Some(&Cell::Number { value: 2 }));
}

#[test]
fn a_heading_or_total_line_is_never_counted_added_up_or_filtered() {
    // C3 and R1. R21 and R24 have a heading line with the sum of the lines under it.
    let w = day();
    let mut s = Session::new(&w).with_numbers(vec![number(0, "0", NumberUnit::Plain)]);
    let more_than_nothing = |column: &str| json!({"column": column, "op": "gt", "value": "q.n1"});
    let r21 = s
        .run(
            "report",
            report("R21", json!({"where": more_than_nothing("balance")})),
        )
        .unwrap();
    let j = s.json(&r21);
    assert_eq!(j["matched"]["count"]["n"], 2, "two buyers, no heading");
    assert_eq!(rs(&j["matched"]["totals"]["balance"]), 21_564);
    let r24 = s
        .run(
            "report",
            report("R24", json!({"where": more_than_nothing("payable")})),
        )
        .unwrap();
    let j = s.json(&r24);
    assert_eq!(j["matched"]["count"]["n"], 1);
    assert_eq!(rs(&j["matched"]["totals"]["payable"]), 10_160);
    // rows.* leaves the heading out: the sum of the buyers' balances is the shop's dues.
    let plain = s.run("report", report("R21", json!({}))).unwrap();
    let sum = s
        .run(
            "calculate",
            json!({"op": "sum", "inputs": [format!("{plain}.rows.*.balance")]}),
        )
        .unwrap();
    let j = result(&s, &sum);
    assert_eq!(
        (rs(&j), j["side"].as_str(), j["checked"].as_bool()),
        (21_564, Some("بنام"), Some(true))
    );
    // A single payable column counted over the rows is the seller's own 10,160, not 20,320.
    let plain24 = s.run("report", report("R24", json!({}))).unwrap();
    let sum = s
        .run(
            "calculate",
            json!({"op": "sum", "inputs": [format!("{plain24}.rows.*.payable")]}),
        )
        .unwrap();
    assert_eq!(rs(&result(&s, &sum)), 10_160);
}

#[test]
fn a_list_as_of_an_earlier_day_is_not_checked_against_todays_party_card() {
    // R2. Akram pays 1,000 on 11-10: his balance on 10-10 was 2,880 and is 1,880 now. That is not
    // a difference in the books, so the row carries no second place.
    let w = day();
    common::set_today(&w.env, 2026, 10, 11);
    let mut receipt = worked::receipt(w.akram, "1,000");
    receipt.date = "11-10-2026".into();
    mandi_services::vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt).unwrap();
    let mut s = Session::new(&w).with_today("2026-10-11");
    let past = s
        .run("report", report("R20", json!({"as_of": "2026-10-10"})))
        .unwrap();
    let j = s.json(&past);
    let akram = j["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["key"] == "a20")
        .unwrap();
    assert_eq!(rs(&akram["balance"]), 2_880);
    assert!(
        akram["balance"]["checked"].is_null(),
        "no check mark and no false difference: {akram}"
    );
    // Now: 1,880, checked in two places.
    let now = s.run("report", report("R20", json!({}))).unwrap();
    let j = s.json(&now);
    let akram = j["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["key"] == "a20")
        .unwrap();
    assert_eq!(rs(&akram["balance"]), 1_880);
    assert_eq!(akram["balance"]["checked"], true);
}

#[test]
fn a_question_number_must_have_the_unit_of_the_column_it_is_compared_with() {
    // R3. A percent is not rupees and "3 lakh" is not a count of days.
    let w = day();
    let mut percent = number(10, "10%", NumberUnit::Percent);
    percent.bps = Some(1_000);
    let mut s = Session::new(&w).with_numbers(vec![
        percent,
        number(300_000, "3 lakh", NumberUnit::Amount),
        number(5, "5", NumberUnit::Plain),
    ]);
    let clause = |column: &str, value: &str| json!({"column": column, "op": "gt", "value": value});
    for (column, value) in [("balance", "q.n1"), ("days", "q.n2"), ("days", "q.n1")] {
        assert_eq!(
            s.error(
                "report",
                report("R20", json!({"where": clause(column, value)}))
            )
            .code,
            "ERR_FORMAT_INVALID",
            "{column} {value}"
        );
    }
    // "3 lakh" is rupees for a money column and a plain 5 is a count of days.
    for (column, value) in [("balance", "q.n2"), ("days", "q.n3"), ("balance", "q.n3")] {
        assert!(
            s.run(
                "report",
                report("R20", json!({"where": clause(column, value)}))
            )
            .is_ok(),
            "{column} {value}"
        );
    }
}

#[test]
fn where_and_sort_refuse_a_balance_column_that_holds_both_sides() {
    // R5. After a payment of 100,000 the cash book runs from بنام to جمع: "more than 50,000" would
    // take a cash of −60,000 as 60,000.
    let w = day();
    let mut payment = worked::payment(w.karim, "100,000");
    payment.confirm_cash_below_zero = true;
    mandi_services::vouchers::save_payment(&w.env.ctx, &w.owner, &payment).unwrap();
    let mut s = Session::new(&w).with_numbers(vec![number(50_000, "50,000", NumberUnit::Plain)]);
    let both = report(
        "R12",
        json!({"from": "2026-10-10", "to": "2026-10-10",
               "where": {"column": "balance", "op": "gt", "value": "q.n1"}}),
    );
    assert_eq!(s.error("report", both).code, "ERR_OUT_OF_RANGE");
    let sorted = report(
        "R12",
        json!({"from": "2026-10-10", "to": "2026-10-10",
               "sort": [{"column": "date", "order": "desc"}, {"column": "balance", "order": "desc"}]}),
    );
    assert_eq!(s.error("report", sorted).code, "ERR_OUT_OF_RANGE");
    // A column of one side only still filters and sorts (R20 is all بنام).
    assert!(
        s.run(
            "report",
            report(
                "R20",
                json!({"where": {"column": "balance", "op": "gt", "value": "q.n1"}})
            )
        )
        .is_ok()
    );
}

#[test]
fn a_difference_or_earlier_sort_key_names_the_figure_the_view_compares() {
    // Analytics note (b). View A compares the total commission, view C the purchase value.
    let w = day();
    let mut s = Session::new(&w);
    let sorted = |view: &str, column: &str| {
        analytics(
            view,
            json!({"from": "2026-10-01", "to": "2026-10-31", "compare": "previous",
                   "sort": [{"column": column, "order": "desc"}]}),
        )
    };
    assert_eq!(
        s.error("analytics", sorted("A", "difference.purchase_value"))
            .code,
        "ERR_OUT_OF_RANGE"
    );
    assert_eq!(
        s.error("analytics", sorted("C", "difference.total_comm"))
            .code,
        "ERR_OUT_OF_RANGE"
    );
    assert_eq!(
        s.error("analytics", sorted("A", "difference.gross")).code,
        "ERR_OUT_OF_RANGE"
    );
    assert!(
        s.run("analytics", sorted("A", "difference.total_comm"))
            .is_ok()
    );
    assert!(
        s.run("analytics", sorted("C", "difference.purchase_value"))
            .is_ok()
    );
}

#[test]
fn an_analytics_sort_on_a_balance_column_that_holds_both_sides_is_refused() {
    // Akram pays 20,000 against his 2,880: he is owed 17,120. "Biggest dues" by size would rank him
    // after Arshad's 18,684 although he owes nothing.
    let w = day();
    let mut receipt = worked::receipt(w.akram, "20,000");
    receipt.date = "10-10-2026".into();
    mandi_services::vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt).unwrap();
    let mut s = Session::new(&w);
    let by_dues = analytics(
        "C",
        json!({"from": "2026-10-01", "to": "2026-10-31",
               "sort": [{"column": "dues_now", "order": "desc"}, {"column": "nag", "order": "desc"}]}),
    );
    assert_eq!(s.error("analytics", by_dues).code, "ERR_OUT_OF_RANGE");
    // Another column of the same view still sorts.
    let by_nag = analytics(
        "C",
        json!({"from": "2026-10-01", "to": "2026-10-31",
               "sort": [{"column": "nag", "order": "desc"}, {"column": "buyer_comm", "order": "desc"}]}),
    );
    assert!(s.run("analytics", by_nag).is_ok());
}

#[test]
fn a_balance_with_no_second_place_beside_a_checked_one_gives_no_check_mark() {
    // R20 as of an earlier day has no second place (money review R2); beside a checked balance
    // the sum has none either.
    let w = day();
    common::set_today(&w.env, 2026, 10, 11);
    let mut s = Session::new(&w).with_today("2026-10-11");
    let past = s
        .run("report", report("R20", json!({"as_of": "2026-10-10"})))
        .unwrap();
    s.run("find", json!({"text": "arshad", "role": "buyer"}))
        .unwrap();
    let sum = s
        .run(
            "calculate",
            json!({"op": "sum", "inputs": [format!("{past}.rows.a20.balance"), "r2.matches.0.balance"]}),
        )
        .unwrap();
    let j = result(&s, &sum);
    assert_eq!(
        (rs(&j), j["side"].as_str(), j["checked"].as_bool()),
        (21_564, Some("بنام"), None)
    );
}
