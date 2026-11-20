//! Plan 82, lane C: the live check of the complex-question list (`scripts/ai-eval/complex/`).
//!
//! IGNORED: `cargo test` never runs it. `scripts/ai-eval/complex/run_live.py` runs it by hand, with
//! the key the Owner saved in the app (`Runtime::real` reads the key store just before each call;
//! this file never sees it and no argument or variable carries it).
//!
//! It asks each question of `questions.json` (one world at a time) on a FRESH COPY of the database,
//! so the daily limit of 20 never stops it and no question sees another's answer. A follow-up goes
//! on the same copy and the same conversation as the question it follows. The app is run as it is
//! (its own instant answers and its reads before the first call are ON), because this check measures
//! what the Owner gets. It writes one JSON line for each question: the whole reply (answer parts,
//! tables, options), the outcome, the wall time, and what the app wrote in its own question log
//! (`finish_reason`, model calls, tokens, calls, tools). `score.py` scores the lines against the
//! expected figures of `questions.json`, which never come from the assistant.
//!
//! Variables (paths, numbers and words only, nothing secret):
//! - `MANDI_CX_DB`: a COPY of the database (never the file the app has open).
//! - `MANDI_CX_TODAY`: `YYYY-MM-DD`, the day the books are read as.
//! - `MANDI_CX_QUESTIONS`: `questions.json`; `MANDI_CX_WORLD`: `dev` or `year`.
//! - `MANDI_CX_ONLY`: optional, a comma list of ids (a follow-up needs the id it follows too).
//! - `MANDI_CX_OUT`: where to write the lines. `MANDI_CX_BUDGET`: dollars this run may spend.
//! - `MANDI_CX_LOCAL`: `1` (default) keeps the app's own instant answers; `0` sends every question
//!   to the model.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;

use std::path::{Path, PathBuf};
use std::time::Instant;

use std::time::Duration;

use mandi_assistant::ask::{AskOutcome, AssistantAskInput, FailureKind};
use mandi_assistant::client::{CallError, ModelClient, OpenRouterClient, Reply};
use mandi_assistant::run::{self, Deps};
use mandi_assistant::runtime::Runtime;
use mandi_assistant::settings::DEFAULT_MODEL;
use mandi_reports::dashboard::BackupFacts;
use mandi_services::assistant;
use mandi_services::auth::Actor;
use serde_json::{Value, json};

/// Dollars a million tokens (scripts/ai-eval/budget.mjs, `openai/gpt-6-luna`) and OpenRouter's fee.
const PRICE_IN: f64 = 0.10;
const PRICE_CACHED: f64 = 0.01;
const PRICE_OUT: f64 = 0.50;
const FEE: f64 = 1.055;
/// What one question may still cost in the worst case, kept free before it starts.
const WORST_QUESTION: f64 = 0.02;

fn cost(tokens_in: i64, cached: i64, out: i64) -> f64 {
    let fresh = (tokens_in - cached).max(0) as f64;
    (fresh * PRICE_IN + cached as f64 * PRICE_CACHED + out as f64 * PRICE_OUT) / 1_000_000.0 * FEE
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn parse_today(text: &str) -> (i32, u32, u32) {
    let mut parts = text.split('-').map(|p| p.parse::<u32>().expect("a number"));
    let y = parts.next().expect("a year");
    let m = parts.next().expect("a month");
    let d = parts.next().expect("a day");
    (i32::try_from(y).expect("a year"), m, d)
}

/// A fresh copy of the database in a temp folder, opened as the app opens it.
fn fresh_env(source: &Path, today: (i32, u32, u32)) -> (tempfile::TempDir, mandi_services::testing::TestEnv) {
    let dir = tempfile::tempdir().expect("a folder for the copy");
    std::fs::create_dir_all(dir.path().join("data")).expect("the data folder");
    std::fs::copy(source, dir.path().join("data").join("mandi.db")).expect("the copy");
    for ext in ["-wal", "-shm"] {
        let mut from = source.to_path_buf().into_os_string();
        from.push(ext);
        let from = PathBuf::from(from);
        if from.exists() {
            std::fs::copy(&from, dir.path().join("data").join(format!("mandi.db{ext}")))
                .expect("the side file");
        }
    }
    let env = mandi_services::testing::TestEnv::new(dir.path()).expect("open the copy");
    common::set_today(&env, today.0, today.1, today.2);
    (dir, env)
}

/// How many calls of a question left no usage: a timeout or a copy that lost.
fn failed_calls(calls_json: Option<&str>) -> usize {
    calls_json
        .and_then(|c| serde_json::from_str::<Value>(c).ok())
        .and_then(|v| v.as_array().cloned())
        .map(|calls| {
            calls
                .iter()
                .filter(|c| c["failed"].as_bool().unwrap_or(false))
                .count()
        })
        .unwrap_or(0)
}

/// A model that never answers: every call ends the way a call ends with no internet, with no wait.
struct OfflineClient;

impl ModelClient for OfflineClient {
    fn call(&self, _key: &str, _body: &Value, _timeout: Duration) -> Result<Reply, CallError> {
        Err(CallError::new(FailureKind::Offline))
    }

    fn retry_wait(&self) -> Duration {
        Duration::ZERO
    }
}

#[test]
#[ignore = "calls the real model with the key of the app; run by hand with scripts/ai-eval/complex/run_live.py (plan 82 lane C)"]
fn live_complex_questions() {
    run_list(false);
}

#[test]
fn offline_complex_questions() {
    // Not live: no key, no network, no model. Does nothing unless `MANDI_CX_OFFLINE=1`. The app's own
    // instant answers and its reads run for real; what needs the model ends as `offline`.
    if std::env::var("MANDI_CX_OFFLINE").as_deref() != Ok("1") {
        return;
    }
    run_list(true);
}

fn run_list(offline: bool) {
    let source = PathBuf::from(std::env::var("MANDI_CX_DB").expect("MANDI_CX_DB: a copy of the database"));
    let questions_path = std::env::var("MANDI_CX_QUESTIONS").expect("MANDI_CX_QUESTIONS");
    let world = std::env::var("MANDI_CX_WORLD").expect("MANDI_CX_WORLD");
    let only: Vec<String> = env_or("MANDI_CX_ONLY", "")
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    let budget: f64 = env_or("MANDI_CX_BUDGET", "0.40").parse().expect("a number");
    let today = parse_today(&std::env::var("MANDI_CX_TODAY").expect("MANDI_CX_TODAY"));
    let local = env_or("MANDI_CX_LOCAL", "1") == "1";
    let out_path = PathBuf::from(std::env::var("MANDI_CX_OUT").expect("MANDI_CX_OUT"));

    let all: Vec<Value> = serde_json::from_str(
        &std::fs::read_to_string(&questions_path).expect("the questions file"),
    )
    .expect("questions.json");
    let list: Vec<&Value> = all
        .iter()
        .filter(|q| q["world"].as_str() == Some(world.as_str()))
        .filter(|q| only.is_empty() || only.iter().any(|id| Some(id.as_str()) == q["id"].as_str()))
        .collect();

    let owner = Actor {
        user_id: 1,
        username: "owner".to_owned(),
        display_name: "Maher Adnan".to_owned(),
    };
    let runtime = if offline {
        // A key store in memory with a placeholder that is not a key: the real store is never opened.
        Runtime::new(std::sync::Arc::new(
            mandi_assistant::keys::MemoryKeyStore::with_key("offline-run-this-is-not-a-key"),
        ))
    } else {
        Runtime::real()
    };
    runtime.set_low_effort(true);
    runtime.set_local(local);
    runtime.set_instant(local);
    if !offline {
        // The first read of the key store in a process is slow on a Mac: paid here, before any clock.
        assert!(
            runtime.keys().get().expect("the key store").is_some(),
            "WAITING FOR KEY: save the OpenRouter key in Settings"
        );
    }

    let live_client = OpenRouterClient;
    let offline_client = OfflineClient;
    let client: &dyn ModelClient = if offline { &offline_client } else { &live_client };
    let emit = |_s| {};
    let mut out = String::new();
    let mut spent = 0.0_f64;
    let mut i = 0;
    while i < list.len() {
        // One group: a question and the follow-ups after it, on one copy of the books.
        let mut j = i + 1;
        while j < list.len() && !list[j]["follow_of"].is_null() {
            j += 1;
        }
        let (_dir, env) = fresh_env(&source, today);
        assistant::set_config(&env.ctx, &owner, true, DEFAULT_MODEL, true).expect("switch on");
        let deps = Deps {
            ctx: &env.ctx,
            actor: &owner,
            runtime: &runtime,
            client,
            emit: &emit,
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let mut conversation: Option<String> = None;
        for q in &list[i..j] {
            let id = q["id"].as_str().unwrap_or("?");
            let question = q["question"].as_str().unwrap_or("");
            if spent + WORST_QUESTION > budget {
                eprintln!("BUDGET STOP at ${spent:.4} of ${budget:.2}: not asked: {id}");
                std::fs::write(&out_path, &out).expect("write the results");
                return;
            }
            let started = Instant::now();
            let reply = run::ask(
                &deps,
                &AssistantAskInput {
                    text: question.to_owned(),
                    conversation_id: conversation.clone(),
                },
            );
            let millis = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);
            let reply = match reply {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{id} {question:?} FAILED: {e:?}");
                    out.push_str(
                        &json!({"id": id, "question": question, "ask_error": format!("{e:?}"), "millis": millis})
                            .to_string(),
                    );
                    out.push('\n');
                    continue;
                }
            };
            conversation = Some(reply.conversation_id.clone());
            let row = assistant::get_question(&env.ctx, reply.question_id)
                .expect("the log")
                .expect("the row of the question");
            let (t_in, t_cached, t_out, t_reason, t_calls) = (
                row.tokens_in.unwrap_or(0),
                row.tokens_cached.unwrap_or(0),
                row.tokens_out.unwrap_or(0),
                row.tokens_reasoning.unwrap_or(0),
                row.model_calls.unwrap_or(0),
            );
            // An offline run sends nothing, so it costs nothing.
            let spend = if offline {
                0.0
            } else {
                cost(t_in, t_cached, t_out)
                    + failed_calls(row.calls_json.as_deref()) as f64 * cost(13_500, 0, 300)
            };
            spent += spend;
            let reply_json = serde_json::to_value(&reply).expect("the reply as JSON");
            eprintln!(
                "{id} {:?} | {millis} ms | calls {t_calls} | ${spend:.4} (total ${spent:.4}) | {} | {question}",
                reply.outcome,
                row.finish_reason.clone().unwrap_or_default()
            );
            let line = json!({
                "id": id,
                "question": question,
                "world": world,
                "offline": offline,
                "outcome": reply.outcome,
                "millis": millis,
                "question_id": reply.question_id,
                "model_calls": t_calls,
                "tokens": {"in": t_in, "cached": t_cached, "out": t_out, "reasoning": t_reason},
                "cost": spend,
                "finish_reason": row.finish_reason,
                "total_millis": row.total_millis,
                "model_millis": row.model_millis,
                "tool_millis": row.tool_millis,
                "tool_calls": serde_json::from_str::<Value>(&row.tool_calls_json).unwrap_or(Value::Null),
                "calls": row.calls_json.as_deref().and_then(|c| serde_json::from_str::<Value>(c).ok()),
                "reply": reply_json,
            });
            out.push_str(&line.to_string());
            out.push('\n');
            std::fs::write(&out_path, &out).expect("write the results");
            assert!(
                reply.outcome != AskOutcome::LimitReached,
                "the daily limit stopped the run"
            );
        }
        i = j;
    }
    eprintln!("DONE ${spent:.4} of ${budget:.2} | {}", out_path.display());
}
