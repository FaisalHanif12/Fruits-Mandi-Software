//! Plan 80 gate fixes of the assistant (findings R6-2 and R6-x, PRD FR-7.76, FR-7.83, FR-7.87,
//! NFR-AI-1, NFR-AI-4, Code Standards Q7), through the real loop with a recorded model.
//!
//! - R6-x: "kul ugrahi" is the tile `total_dues` with all four parts (buyers, sellers, the dead
//!   amount, the open money lent). The tiles tool has the tiles of the dashboard (no backup tile;
//!   آج کی نقدی and the others' dues are in). An answer that cites only `buyers_dues` is made to cite
//!   the total.
//! - R6-2: no audit row ever reaches the model: `app_state` has no audit read, an old call for one is
//!   refused, and no request of the conversation holds a row of the audit log.
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

use mandi_assistant::ask::AskOutcome;
use mandi_services::core::lent::{LentPersonCreateInput, MoneyLentInput};
use mandi_services::{lent, vouchers};
use serde_json::{Value, json};

use harness::{Harness, answer_of, figures, money};

// ---- recorded replies ---------------------------------------------------------------------------

fn usage() -> Value {
    json!({"prompt_tokens": 5200, "completion_tokens": 60,
           "prompt_tokens_details": {"cached_tokens": 0},
           "completion_tokens_details": {"reasoning_tokens": 20}})
}

fn tool_reply(calls: &[(&str, Value)]) -> Value {
    let calls: Vec<Value> = calls
        .iter()
        .enumerate()
        .map(|(i, (name, args))| {
            json!({"id": format!("call_{i}"), "type": "function",
                   "function": {"name": name, "arguments": args.to_string()}})
        })
        .collect();
    json!({"model": "openai/gpt-6-luna",
           "choices": [{"finish_reason": "tool_calls",
                        "message": {"role": "assistant", "content": null, "tool_calls": calls}}],
           "usage": usage()})
}

fn final_reply(kind: &str, text: &str) -> Value {
    let content = json!({"kind": kind, "text": text, "show": [], "next": [],
                         "choose": null, "entry": null, "nearest": null})
    .to_string();
    json!({"model": "openai/gpt-6-luna",
           "choices": [{"finish_reason": "stop",
                        "message": {"role": "assistant", "content": content}}],
           "usage": usage()})
}

/// The results of the tools in the requests, as the model read them (the `tool` messages that hold
/// JSON), newest last.
fn tool_results(requests: &[Value]) -> Vec<Value> {
    let mut out = Vec::new();
    for body in requests {
        for m in body["messages"].as_array().into_iter().flatten() {
            if m["role"] == "tool"
                && let Some(text) = m["content"].as_str()
                && let Ok(v) = serde_json::from_str::<Value>(text)
            {
                out.push(v);
            }
        }
    }
    out
}

fn rs(v: &Value) -> i64 {
    v["rs"].as_i64().unwrap()
}

// ---- R6-x ---------------------------------------------------------------------------------------

/// The worked day and 5,000 lent to Aslam: buyers owe 21,564, the shop is owed 26,564.
fn lent_day() -> Harness {
    let h = Harness::day();
    let w = &h.world;
    let person = lent::person_create(
        &w.env.ctx,
        &w.owner,
        &LentPersonCreateInput {
            name_ur: "اسلم".into(),
            name_en: "Aslam".into(),
            phone: None,
            staff_id: None,
            confirmed_rules_words: vec!["اسلم".into(), "Aslam".into()],
        },
    )
    .unwrap();
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &MoneyLentInput {
            date: "10-10-2026".into(),
            person_account_id: Some(person.account.id),
            amount: "5,000".into(),
            reason: "ghar".into(),
            return_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    h
}

/// With money lent open, "kul ugrahi" is the tile `total_dues`: 26,564 with all four of its parts
/// (21,564 of the buyers, 0 of the sellers, 0 dead, 5,000 lent), checked against its parts. The
/// tiles result is the dashboard's: no backup tile, and آج کی نقدی and the others' dues are in.
#[test]
fn kul_ugrahi_is_the_total_dues_tile_with_all_four_parts() {
    let h = lent_day();
    let (reply, requests) = h.ask(
        "kul ugrahi kitni hai",
        None,
        vec![
            tool_reply(&[("tiles", json!({}))]),
            final_reply("answer", "کل اگراہی {{r1.total_dues.amount}} روپے ہے۔"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    assert_eq!(money(figs[0].0), 26_564);
    assert!(figs[0].1, "checked against its four parts");
    // The tiles result the model read.
    let tiles = tool_results(&requests)
        .into_iter()
        .find(|v| v.get("total_dues").is_some())
        .expect("a tiles result");
    assert_eq!(rs(&tiles["total_dues"]["amount"]), 26_564);
    assert_eq!(tiles["total_dues"]["amount"]["checked"], true);
    let holds: Vec<&str> = tiles["total_dues"]["holds"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert_eq!(
        holds,
        [
            "buyers_dues",
            "sellers_dues",
            "dead_amount",
            "money_lent_open"
        ]
    );
    assert_eq!(rs(&tiles["buyers_dues"]["amount"]), 21_564);
    assert_eq!(rs(&tiles["sellers_dues"]["amount"]), 0);
    assert_eq!(rs(&tiles["dead_amount"]["amount"]), 0);
    assert_eq!(rs(&tiles["money_lent_open"]["amount"]), 5_000);
    // The tiles of the dashboard (PRD 9.1 v1.41 and v1.44).
    assert!(tiles.get("last_backup").is_none(), "no backup tile");
    assert_eq!(rs(&tiles["today_cash_sales"]["amount"]), 12_800);
    assert_eq!(tiles["today_cash_sales"]["count"]["n"], 1);
    let tile_now = mandi_reports::dashboard::tiles_now(h.world.env.ctx.db())
        .unwrap()
        .tiles;
    assert_eq!(
        rs(&tiles["others_dues"]["amount"]),
        tile_now.others_dues_paisa.rupees()
    );
    assert_eq!(
        tiles["others_dues"]["count"]["n"].as_i64().unwrap(),
        tile_now.others_people_count
    );
}

/// An answer to "kul ugrahi" that cites only the buyers' dues is made to cite the total (the
/// guard rewrites the reference to the same tiles result's `total_dues`), so the figure shown is
/// 26,564, not 21,564 (R6-x).
#[test]
fn an_answer_to_kul_ugrahi_that_cites_the_buyers_dues_alone_shows_the_total() {
    let h = lent_day();
    let (reply, _) = h.ask(
        "kul ugrahi kitni hai",
        None,
        vec![
            tool_reply(&[("tiles", json!({}))]),
            final_reply("answer", "کل اگراہی {{r1.buyers_dues.amount}} روپے ہے۔"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    assert_eq!(money(figs[0].0), 26_564);
}

/// A question about the buyers' dues alone, and an answer that cites the total as well as a part,
/// are left as they are.
#[test]
fn the_buyers_dues_alone_and_a_breakdown_are_not_changed() {
    // "buyers ke dues": the part is the right figure.
    let h = lent_day();
    let (reply, _) = h.ask(
        "buyers ke dues kitne hain",
        None,
        vec![
            tool_reply(&[("tiles", json!({}))]),
            final_reply(
                "answer",
                "گاہکوں کے ذمے {{r1.buyers_dues.amount}} روپے ہیں۔",
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(money(figures(answer_of(&reply))[0].0), 21_564);
    // "kul ugrahi" answered with the total and its part: both stay.
    let h = lent_day();
    let (reply, _) = h.ask(
        "kul ugrahi kitni hai",
        None,
        vec![
            tool_reply(&[("tiles", json!({}))]),
            final_reply(
                "answer",
                "کل اگراہی {{r1.total_dues.amount}} روپے ہے، اس میں گاہکوں کے {{r1.buyers_dues.amount}}۔",
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let shown: Vec<i64> = figures(answer_of(&reply))
        .iter()
        .map(|f| money(f.0))
        .collect();
    assert_eq!(shown, [26_564, 21_564]);
}

/// The question words that ask the shop's total dues, and those that do not.
#[test]
fn the_words_that_ask_the_total_dues() {
    use mandi_assistant::understand::asks_total_dues;
    for yes in [
        "kul ugrahi kitni hai",
        "total ugrahi kya hai",
        "what are the total dues",
        "کل اگراہی کتنی ہے",
    ] {
        assert!(asks_total_dues(yes), "{yes}");
    }
    for no in [
        "akram ka bakaya",
        "buyers ke dues kitne hain",
        "collector wise ugrahi",
        "city wise total dues",
        "aaj ki bikri",
    ] {
        assert!(!asks_total_dues(no), "{no}");
    }
}

// ---- R6-2 ---------------------------------------------------------------------------------------

/// No audit row reaches the model. The model tries `app_state` with `what: "audit"` in every shape
/// (a بکری, the whole log, a party, a huge number); every call is refused as an input error; no
/// request of the conversation lists `audit` among the reads of `app_state`; and no request holds a
/// row of the audit log: not its chain hash, not its action words.
#[test]
fn no_request_holds_a_row_of_the_audit_log() {
    let h = Harness::day();
    // The audit log of the worked day: every row has a chain hash that no answer could produce.
    let conn = common::raw(&h.world.dir);
    let hashes: Vec<String> = conn
        .prepare("SELECT chain_hash FROM audit_log WHERE chain_hash IS NOT NULL")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert!(
        hashes.len() >= 5,
        "the worked day has audit rows: {}",
        hashes.len()
    );
    let (reply, requests) = h.ask(
        "bikri 1 mein kisne kya badla",
        None,
        vec![
            tool_reply(&[
                (
                    "app_state",
                    json!({"what": "audit", "party": null, "item": null, "bikri": 1}),
                ),
                (
                    "app_state",
                    json!({"what": "audit", "party": null, "item": null, "bikri": null}),
                ),
                (
                    "app_state",
                    json!({"what": "audit", "party": "a1", "item": null, "bikri": null}),
                ),
                (
                    "app_state",
                    json!({"what": "audit", "party": null, "item": null, "bikri": 99999}),
                ),
            ]),
            final_reply("not_understood", ""),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::NotUnderstood, "{reply:#?}");
    // Every call came back as an error result.
    let errors: Vec<Value> = tool_results(&requests)
        .into_iter()
        .filter(|v| v["error"].is_object())
        .collect();
    assert_eq!(errors.len(), 4, "{errors:#?}");
    let all: String = requests
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    for hash in &hashes {
        assert!(
            !all.contains(hash.as_str()),
            "an audit chain hash was sent: {hash}"
        );
    }
    for marker in [
        "\\\"action\\\":\\\"create\\\"",
        "\\\"entity\\\":\\\"bikri\\\"",
        "\"changes\"",
    ] {
        assert!(
            !all.contains(marker),
            "an audit row field was sent: {marker}"
        );
    }
    // The tool definitions the model got: `app_state` offers rules, attention and days only.
    for body in &requests {
        let state = body["tools"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|t| t["function"]["name"] == "app_state")
            .expect("app_state is offered");
        let whats: Vec<&str> = state["function"]["parameters"]["properties"]["what"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert_eq!(whats, ["rules", "attention", "days"]);
    }
    h.scan_requests();
}
