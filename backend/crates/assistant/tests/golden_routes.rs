//! Golden questions of the direct routes of plan 99 (PRD v1.60, FR-7.87), with recorded model
//! replies and no network (Code Standards 9.2). Each transcript takes one round of tools: the period
//! word on the read, `bikri: "last"`, R13 with `layout: "bikri"`, the gardens (R36 and view I) and
//! the market trips (R33 and view J). The tools run, every reference resolves, `check.rs` passes the
//! answer and the filled text equals the expected text of the fixture
//! (`fixtures/model/route-*.json`). A run with `MANDI_WRITE_FIXTURES=1` writes the filled texts,
//! which are then read against the books before they are committed.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/plain_text.rs"]
mod plain;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use std::sync::Arc;

use mandi_assistant::ask::{AskReply, AssistantAskInput};
use mandi_assistant::client::RecordedClient;
use mandi_assistant::keys::MemoryKeyStore;
use mandi_assistant::results::Words;
use mandi_assistant::run::{Deps, ask};
use mandi_assistant::runtime::Runtime;
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::auth::Language;
use mandi_services::core::gardens::GardenInput;
use mandi_services::core::purchases::PurchaseInput;
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{assistant, gardens, purchases, trips};
use plain::plain_text;
use serde_json::Value;

const TEST_KEY: &str = "sk-or-v1-0000000000000000000000000000000000000000000000000000";

const ROUTES: [&str; 7] = [
    "last-bikri",
    "khata-bikri",
    "garden",
    "trip",
    "month-expense",
    "no-dates",
    "estimate-period",
];

fn share(partner_id: i64, share: &str, source: MoneySource) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source,
    }
}

/// The worked day, then the first installment of E2E-40's garden (rent 50,000, paid 30,000: الف 60%
/// from the shop, ب 40% from his pocket).
fn garden_world() -> world::World {
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let [a, b, _] = w.partners;
    gardens::create(
        &w.env.ctx,
        &w.owner,
        &GardenInput {
            name_ur: "شاہ پور باغ".into(),
            name_en: "Shahpur Garden".into(),
            confirmed_rules_words: Vec::new(),
            date: "10-10-2026".into(),
            item_id: Some(w.banana),
            item_ids: Vec::new(),
            rent: "50,000".into(),
            season_end: None,
            note: None,
            partners: vec![
                share(a, "60", MoneySource::Shop),
                share(b, "40", MoneySource::Pocket),
            ],
            paid_now: Some("30,000".into()),
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .expect("the garden");
    w
}

/// E2E-34's trip purchase: 20 پیٹی at 500 with 400 freight (10,400), shares 40 / 30 / 30.
fn trip_world() -> world::World {
    let w = worked::worked_day();
    let [a, b, c] = w.partners;
    let trip = trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: vec![
                share(a, "40", MoneySource::Shop),
                share(b, "30", MoneySource::Shop),
                share(c, "30", MoneySource::Shop),
            ],
        },
    )
    .expect("the trip");
    purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(trip.id),
            date: "10-10-2026".into(),
            supplier: None,
            item_id: Some(w.banana),
            packing_id: Some(w.box_),
            qty: "20".into(),
            weight: None,
            rate: "500".into(),
            freight: Some("400".into()),
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .expect("the purchase");
    w
}

fn build(name: &str) -> world::World {
    match name {
        "day" => {
            let w = worked::worked_day();
            worked::events_1_to_5(&w);
            w
        }
        "garden" => garden_world(),
        "trip" => trip_world(),
        "history" => worked::history_world(&[
            (4, 180, 1_600),
            (5, 150, 2_100),
            (6, 120, 2_850),
            (7, 225, 1_600),
            (8, 150, 2_600),
            (9, 173, 2_390),
        ]),
        other => panic!("no world {other}"),
    }
}

#[test]
fn the_new_routes_answer_in_one_round() {
    let words = Words::new().unwrap();
    let write = std::env::var("MANDI_WRITE_FIXTURES").is_ok_and(|v| v == "1");
    let mut failures = Vec::new();
    for name in ROUTES {
        let path = format!(
            "{}/tests/fixtures/model/route-{name}.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut fixture: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let w = build(fixture["world"].as_str().unwrap());
        let runtime = Runtime::new(Arc::new(MemoryKeyStore::default()));
        runtime.set_local(false);
        runtime.set_read_first(false);
        runtime.set_low_effort(false);
        runtime.keys().set(TEST_KEY).unwrap();
        assistant::set_config(&w.env.ctx, &w.owner, true, "openai/gpt-6-luna", true).unwrap();
        let step = fixture["steps"][0].clone();
        let replies: Vec<Value> = step["replies"].as_array().unwrap().clone();
        let client = RecordedClient::new(replies);
        let emit = |_s| {};
        let deps = Deps {
            ctx: &w.env.ctx,
            actor: &w.owner,
            runtime: &runtime,
            client: &client,
            emit: &emit,
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let reply: AskReply = ask(
            &deps,
            &AssistantAskInput {
                text: step["question"].as_str().unwrap().to_owned(),
                conversation_id: None,
            },
        )
        .unwrap();
        let got = serde_json::to_value(reply.outcome).unwrap();
        let logged = assistant::get_question(&w.env.ctx, reply.question_id)
            .unwrap()
            .unwrap();
        if got != step["outcome"] {
            failures.push(format!(
                "{name}: outcome {got}: {:?}\n{reply:#?}",
                logged.calls_json
            ));
            continue;
        }
        let calls = logged.model_calls;
        if calls != Some(2) {
            failures.push(format!(
                "{name}: {calls:?} model calls, not 2 (one round): {:?}",
                logged.calls_json
            ));
        }
        let answer = reply.answer.as_ref().expect("an answer");
        let text = plain_text(answer, Language::Ur, &words);
        if write {
            eprintln!("route {name}:\n{text}\n");
            fixture["steps"][0]["expected"] = Value::String(text);
            let mut out = serde_json::to_string_pretty(&fixture).unwrap();
            out.push('\n');
            std::fs::write(&path, out).unwrap();
            continue;
        }
        match step["expected"].as_str() {
            Some(expected) if expected == text => {}
            Some(expected) => {
                failures.push(format!("{name}: expected\n{expected}\nbut got\n{text}"));
            }
            None => failures.push(format!("{name}: no expected text; it is\n{text}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}
