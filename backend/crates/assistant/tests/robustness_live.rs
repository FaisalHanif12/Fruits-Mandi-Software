//! The live run of the 100-question list (D6 Section 3 / plan 50 R2, NFR-AI-2). The engine under
//! `scripts/ai-eval/run.mjs`: it asks every question of `scripts/ai-eval/questions.tsv` through the
//! app's own question path (`run::ask`, the function behind the `assistant_ask` command) on a
//! seeded database, with the real client, and writes what happened as JSON.
//!
//! IGNORED: `cargo test` never runs it. Only `run.mjs` runs it, with `--ignored`, after the Owner
//! has saved his OpenRouter key in Settings. The key is read from the app's own place for it
//! (Windows Credential Manager, `KeyringStore`) just before each call by the loop; this file never
//! sees it, and no argument or environment variable carries it. What the variables below hold is
//! paths and a list of ids, nothing secret.
//!
//! - `MANDI_EVAL_QUESTIONS`: the path of `questions.tsv`.
//! - `MANDI_EVAL_OUT`: where to write the JSON results.
//! - `MANDI_EVAL_ONLY`: optional, a comma list of ids to run (for example `Q001,Q016`).
//! - `MANDI_EVAL_MODEL`: optional model name (the default is the app's default model).
//!
//! Each question gets a fresh seeded database, so the daily limit of 20 never stops the run and no
//! question sees another's answer. A follow-up first asks the question it follows, in the same
//! conversation. The worlds are the PRD's: `day` (Events 1 to 5, 10-10-26), `late` (the same day,
//! 15-11-26), `e59` (six months of commission, E2E-59), `e42` (E2E-16 with the fruit of E2E-42) and
//! `e34` (a trip purchase, E2E-34).
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use mandi_assistant::ask::{AskReply, AssistantAskInput};
use mandi_assistant::client::{CallError, ModelClient, OpenRouterClient, Reply};
use mandi_assistant::run::{self, Deps};
use mandi_assistant::runtime::Runtime;
use mandi_assistant::settings::DEFAULT_MODEL;
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::fruit::{FruitInput, FruitSource};
use mandi_services::core::purchases::PurchaseInput;
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{assistant, fruit, purchases, trips};
use serde_json::{Value, json};

/// The E2E-59 months: (month, qty, rate) of one banana sale a month, April to September.
const HISTORY: [(u32, i64, i64); 6] = [
    (4, 180, 1_600),
    (5, 150, 2_100),
    (6, 120, 2_850),
    (7, 225, 1_600),
    (8, 150, 2_600),
    (9, 173, 2_390),
];

struct Question {
    id: String,
    world: String,
    after: String,
    text: String,
}

fn questions(path: &str, only: &[String]) -> Vec<Question> {
    let text = std::fs::read_to_string(path).expect("the questions file");
    let mut rows = text.lines();
    let head: Vec<&str> = rows.next().expect("a header").split('\t').collect();
    let at = |name: &str| head.iter().position(|h| *h == name).expect(name);
    let (id, world, after, question) = (at("id"), at("world"), at("after"), at("question"));
    rows.filter(|l| !l.trim().is_empty())
        .map(|l| {
            let cells: Vec<&str> = l.split('\t').collect();
            Question {
                id: cells[id].to_owned(),
                world: cells[world].to_owned(),
                after: cells[after].to_owned(),
                text: cells[question].to_owned(),
            }
        })
        .filter(|q| only.is_empty() || only.contains(&q.id))
        .collect()
}

/// One seeded database for one question.
fn build(world_name: &str) -> world::World {
    match world_name {
        "late" | "day" => {
            let w = worked::worked_day();
            worked::events_1_to_5(&w);
            if world_name == "late" {
                common::set_today(&w.env, 2026, 11, 15);
            }
            w
        }
        "e59" => worked::history_world(&HISTORY),
        "e42" => {
            // E2E-16 (45 of 50 sold), then الف takes 2 کیلا home and the guests have 3 سیب.
            let w = worked::part_sold_world();
            let nazeer = world::add_account(
                &w.env,
                &w.owner,
                mandi_services::core::accounts::AccountType::Seller,
                "حاجی نذیر",
                "Haji Nazeer",
                true,
                "",
            );
            let input = |guests: bool| FruitInput {
                for_guests: guests,
                partner_id: (!guests).then_some(w.partners[0]),
                date: "10-10-2026".into(),
                source: if guests {
                    FruitSource::OtherShop
                } else {
                    FruitSource::Shop
                },
                lot_id: (!guests).then_some(1),
                agent_account_id: guests.then_some(nazeer),
                item_id: guests.then_some(w.apple),
                packing_id: guests.then_some(w.box_),
                qty: if guests { "3" } else { "2" }.into(),
                weight: None,
                rate: if guests { "1,000" } else { "580" }.into(),
                note: None,
                editing_id: None,
            };
            fruit::save(&w.env.ctx, &w.owner, &input(false)).expect("fruit for the home");
            fruit::save(&w.env.ctx, &w.owner, &input(true)).expect("fruit for the guests");
            w
        }
        "e34" => {
            // E2E-34: a trip of the three partners at 40 / 30 / 30 from the shop's cash buys 20
            // boxes at 500 with 400 freight (10,400).
            let w = worked::worked_day();
            let [a, b, c] = w.partners;
            let share = |partner_id, share: &str| VenturePartnerInput {
                partner_id,
                share: share.into(),
                source: MoneySource::Shop,
            };
            let trip = trips::create(
                &w.env.ctx,
                &w.owner,
                &TripInput {
                    date: "10-10-2026".into(),
                    market: Some("Lahore".into()),
                    sent_person: None,
                    note: None,
                    partners: vec![share(a, "40"), share(b, "30"), share(c, "30")],
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
        other => panic!("unknown world {other}"),
    }
}

/// One model call as the run saw it.
#[derive(Debug, Clone)]
struct CallRecord {
    millis: u128,
    prompt: i64,
    cached: i64,
    completion: i64,
    reasoning: i64,
    tool_calls: usize,
    finish: String,
    model: Option<String>,
}

/// The real client, with a note of every call: time, tokens, how many tools it asked for.
struct Recorder {
    inner: OpenRouterClient,
    calls: Mutex<Vec<CallRecord>>,
}

impl ModelClient for Recorder {
    fn call(&self, key: &str, body: &Value, timeout: Duration) -> Result<Reply, CallError> {
        let started = Instant::now();
        let result = self.inner.call(key, body, timeout);
        let millis = started.elapsed().as_millis();
        if let Ok(reply) = &result {
            self.calls.lock().expect("the notes").push(CallRecord {
                millis,
                prompt: reply.usage.prompt_tokens,
                cached: reply.usage.cached_tokens,
                completion: reply.usage.completion_tokens,
                reasoning: reply.usage.reasoning_tokens,
                tool_calls: reply.tool_calls.len(),
                finish: reply.finish_reason.clone(),
                model: reply.model.clone(),
            });
        }
        result
    }
}

fn ask(deps: &Deps<'_>, text: &str, conversation: Option<&str>) -> AskReply {
    run::ask(
        deps,
        &AssistantAskInput {
            text: text.to_owned(),
            conversation_id: conversation.map(str::to_owned),
        },
    )
    .expect("the question")
}

#[test]
#[ignore = "live: calls the real model with the key saved in the app; run it with scripts/ai-eval/run.mjs"]
fn live_100_questions() {
    let questions_path = std::env::var("MANDI_EVAL_QUESTIONS").expect("MANDI_EVAL_QUESTIONS");
    let out_path = std::env::var("MANDI_EVAL_OUT").expect("MANDI_EVAL_OUT");
    let only: Vec<String> = std::env::var("MANDI_EVAL_ONLY")
        .map(|v| v.split(',').map(|s| s.trim().to_owned()).collect())
        .unwrap_or_default();
    let model = std::env::var("MANDI_EVAL_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_owned());
    // The key is in the app's own store. Is it there? (Never printed, never kept.)
    let runtime = Runtime::real();
    let key_saved = runtime.keys().get().map(|k| k.is_some()).unwrap_or(false);
    if !key_saved {
        std::fs::write(&out_path, json!({"waiting_for_key": true}).to_string())
            .expect("the results file");
        return;
    }
    let list = questions(&questions_path, &only);
    let by_id: std::collections::HashMap<String, usize> = questions(&questions_path, &[])
        .iter()
        .enumerate()
        .map(|(i, q)| (q.id.clone(), i))
        .collect();
    let all = questions(&questions_path, &[]);
    let mut results: Vec<Value> = Vec::new();
    for q in &list {
        let started = Instant::now();
        let w = build(&q.world);
        assistant::set_config(&w.env.ctx, &w.owner, true, &model, true).expect("switch on");
        let client = Recorder {
            inner: OpenRouterClient,
            calls: Mutex::new(Vec::new()),
        };
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
        // A follow-up asks the question it follows first, in the same conversation.
        let mut conversation: Option<String> = None;
        if let Some(base) = by_id.get(&q.after).and_then(|i| all.get(*i)) {
            conversation = Some(ask(&deps, &base.text, None).conversation_id);
            client.calls.lock().expect("notes").clear();
        }
        let reply = ask(&deps, &q.text, conversation.as_deref());
        let row = assistant::get_question(&w.env.ctx, reply.question_id)
            .expect("the log")
            .expect("the row");
        let calls: Vec<Value> = client
            .calls
            .lock()
            .expect("notes")
            .iter()
            .map(|c| {
                json!({"millis": c.millis, "prompt": c.prompt, "cached": c.cached,
                       "completion": c.completion, "reasoning": c.reasoning,
                       "tool_calls": c.tool_calls, "finish": c.finish, "model": c.model})
            })
            .collect();
        let tool_calls: Value =
            serde_json::from_str(&row.tool_calls_json).unwrap_or(Value::Array(Vec::new()));
        results.push(json!({
            "id": q.id,
            "question": q.text,
            "world": q.world,
            "reply": serde_json::to_value(&reply).expect("the reply as JSON"),
            "tool_calls": tool_calls,
            "model_calls": row.model_calls,
            "tokens": {"in": row.tokens_in, "cached": row.tokens_cached,
                       "out": row.tokens_out, "reasoning": row.tokens_reasoning},
            "calls": calls,
            "millis": started.elapsed().as_millis(),
        }));
        eprintln!(
            "{} {} {:?} ({} ms)",
            q.id,
            q.world,
            reply.outcome,
            started.elapsed().as_millis()
        );
    }
    let meta = json!({
        "model": model,
        "endpoint": "azure (only: azure, allow_fallbacks: false, zdr: true, data_collection: deny)",
        "questions": list.len(),
    });
    std::fs::write(
        &out_path,
        json!({"meta": meta, "results": results}).to_string(),
    )
    .expect("the results file");
}

#[test]
fn the_five_worlds_of_the_list_are_the_prds() {
    // Not live: no model, no key. It proves that the databases the live run asks about hold the
    // PRD numbers its expected figures come from, so a miss is never the seed's fault.
    use mandi_services::{cash, ledger};
    let balance = |w: &world::World, id: i64| {
        let b = ledger::account_balance(&w.env.ctx, id).expect("balance");
        (b.amount_paisa.rupees(), format!("{:?}", b.side))
    };
    let cash_now = |w: &world::World| {
        w.env
            .ctx
            .db()
            .read(cash::cash_in_hand)
            .expect("cash")
            .rupees()
    };
    let day = build("day");
    assert_eq!(balance(&day, day.akram), (2_880, "Dr".to_owned()), "E2E-47");
    assert_eq!(balance(&day, day.arshad), (18_684, "Dr".to_owned()));
    assert_eq!(cash_now(&day), 58_484, "E2E-8");
    let late = build("late");
    assert_eq!(
        assistant::pakistan_day(&late.env.ctx),
        "2026-11-15",
        "E2E-63"
    );
    assert_eq!(balance(&late, late.akram), (2_880, "Dr".to_owned()));
    let e34 = build("e34");
    assert_eq!(cash_now(&e34), 39_600, "E2E-34: 50,000 less 10,400");
    let e42 = build("e42");
    assert_eq!(cash_now(&e42), 45_000, "E2E-42: cash does not move");
    let e59 = build("e59");
    assert_eq!(assistant::pakistan_day(&e59.env.ctx), "2026-10-10");
}
