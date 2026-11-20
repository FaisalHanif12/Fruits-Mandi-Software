//! Plan 82: the live check of the MODEL path on a COPY of a real database (the shop's own books),
//! with the real model and the real key of the app.
//!
//! IGNORED: `cargo test` never runs it. It is run by hand, by the lead or by the backend teammate of
//! plan 82, after the Owner has saved his OpenRouter key in Settings. The key is read through the
//! app's own key store (`Runtime::real`) by the loop, just before each call; this file never sees
//! it, and no argument or environment variable carries it. The variables below hold paths, a list
//! of questions and numbers, nothing secret.
//!
//! - `MANDI_LIVE_DB`: a COPY of `mandi.db` (never the file the app has open). Its folder is copied
//!   again into a temp folder for the run, so the copy itself stays as it was.
//! - `MANDI_LIVE_QUESTIONS`: a text file, one question a line. A line that starts with `>` is a
//!   follow-up in the same conversation as the line above it. Empty lines and `#` lines are skipped.
//! - `MANDI_LIVE_OUT`: where to write one JSON line a question (default the temp folder).
//! - `MANDI_LIVE_TODAY`: `YYYY-MM-DD`, the day the books are read as (default 2026-11-20).
//! - `MANDI_LIVE_EFFORT`: `low` (default, the app's setting) or `medium`.
//! - `MANDI_LIVE_BUDGET`: the most to spend in dollars, default 0.30. Before each question the money
//!   spent plus the worst one question may still cost must stay inside it; the run stops when not.
//! - `MANDI_LIVE_WARM`: `1` sends the warm-up call first, as the panel does when it opens, and waits
//!   `MANDI_LIVE_WARM_WAIT` seconds (default 4) before the first question.
//! - `MANDI_LIVE_LOCAL`: `1` leaves the app's own instant answers on (the app as it is); the default
//!   `0` sends every question to the model, which is what this check measures.
//!
//! What it prints for each question: the outcome, the time, the model calls, the tokens, the cost,
//! the figures of the answer, and each call's time. The prices are those of
//! `scripts/ai-eval/budget.mjs` for `openai/gpt-6-luna` (a million tokens: $0.10 input, $0.01
//! cached, $0.50 output, plus OpenRouter's 5.5% fee).
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;

use std::path::PathBuf;
use std::time::Instant;

use mandi_assistant::ask::{AskOutcome, AssistantAskInput};
use mandi_assistant::client::OpenRouterClient;
use mandi_assistant::run::{self, Deps};
use mandi_assistant::runtime::Runtime;
use mandi_assistant::settings::DEFAULT_MODEL;
use mandi_reports::dashboard::BackupFacts;
use mandi_services::assistant;
use mandi_services::auth::Actor;
use serde_json::{Value, json};

/// Dollars a million tokens, with the fee of OpenRouter (5.5%) added by `cost`.
const PRICE_IN: f64 = 0.10;
const PRICE_CACHED: f64 = 0.01;
const PRICE_OUT: f64 = 0.50;
const FEE: f64 = 1.055;
/// What one question may still cost in the worst case, kept free before it starts: six calls with
/// the whole prefix uncached and a full answer each.
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

/// The text and the figures an answer shows, for the printout.
fn shown(reply: &Value) -> Value {
    json!({
        "parts": reply["answer"]["parts"],
        "tables": reply["answer"]["tables"].as_array().map(|t| t.len()),
    })
}

#[test]
#[ignore = "calls the real model with the key of the app; run by hand (plan 82)"]
fn live_questions_on_a_copy_of_the_dev_database() {
    let source =
        PathBuf::from(std::env::var("MANDI_LIVE_DB").expect("MANDI_LIVE_DB: a copy of mandi.db"));
    let questions_path = std::env::var("MANDI_LIVE_QUESTIONS").expect("MANDI_LIVE_QUESTIONS");
    let budget: f64 = env_or("MANDI_LIVE_BUDGET", "0.30")
        .parse()
        .expect("a number");
    let (y, m, d) = parse_today(&env_or("MANDI_LIVE_TODAY", "2026-11-20"));
    let low = env_or("MANDI_LIVE_EFFORT", "low") != "medium";
    let local = env_or("MANDI_LIVE_LOCAL", "0") == "1";

    let dir = tempfile::tempdir().expect("a folder for the copy");
    std::fs::create_dir_all(dir.path().join("data")).expect("the data folder");
    std::fs::copy(&source, dir.path().join("data").join("mandi.db")).expect("the copy");
    for ext in ["-wal", "-shm"] {
        let mut from = source.clone().into_os_string();
        from.push(ext);
        let from = PathBuf::from(from);
        if from.exists() {
            std::fs::copy(
                &from,
                dir.path().join("data").join(format!("mandi.db{ext}")),
            )
            .expect("the side file");
        }
    }
    let env = mandi_services::testing::TestEnv::new(dir.path()).expect("open the copy");
    common::set_today(&env, y, m, d);
    // The Owner of the copy: no sign-in is needed to ask a question through `run::ask`.
    let owner = Actor {
        user_id: 1,
        username: "owner".to_owned(),
        display_name: "Maher Adnan".to_owned(),
    };
    assistant::set_config(&env.ctx, &owner, true, DEFAULT_MODEL, true).expect("switch on");
    let runtime = Runtime::real();
    runtime.set_low_effort(low);
    runtime.set_local(local);
    runtime.set_instant(local);
    // The first read of the key store in a process is slow on a Mac (the keychain checks the
    // unsigned test binary). It is the computer's, so it is paid here, before any clock.
    let began = Instant::now();
    assert!(
        runtime.keys().get().expect("the key store").is_some(),
        "WAITING FOR KEY: save the OpenRouter key in Settings"
    );
    eprintln!("key store read: {} ms", began.elapsed().as_millis());

    let client = OpenRouterClient;
    let emit = |_s| {};
    let deps = Deps {
        ctx: &env.ctx,
        actor: &owner,
        runtime: &runtime,
        client: &client,
        emit: &emit,
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };

    // The warm-up call (plan 82): sent first, as the panel does when it opens; then the Owner
    // "types" for `MANDI_LIVE_WARM_WAIT` seconds (default 4) before the first question.
    if env_or("MANDI_LIVE_WARM", "0") == "1" {
        let began = Instant::now();
        let outcome = mandi_assistant::warm::warm(&deps);
        eprintln!("WARM-UP {outcome:?} in {} ms", began.elapsed().as_millis());
        let wait: u64 = env_or("MANDI_LIVE_WARM_WAIT", "4")
            .parse()
            .expect("seconds");
        std::thread::sleep(std::time::Duration::from_secs(wait));
    }

    let text = std::fs::read_to_string(&questions_path).expect("the questions file");
    let out_path = PathBuf::from(env_or(
        "MANDI_LIVE_OUT",
        &dir.path().join("live.jsonl").to_string_lossy(),
    ));
    let mut out = String::new();
    let mut spent = 0.0_f64;
    let mut conversation: Option<String> = None;
    let mut times: Vec<i64> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (follow, question) = match line.strip_prefix('>') {
            Some(rest) => (true, rest.trim()),
            None => (false, line),
        };
        if spent + WORST_QUESTION > budget {
            eprintln!("BUDGET STOP at ${spent:.4} of ${budget:.2}: not asked: {question}");
            break;
        }
        if !follow {
            conversation = None;
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
                eprintln!("{question:?} FAILED: {e:?}");
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
        let spend = cost(t_in, t_cached, t_out)
            // A call that got no reply may still have been billed: count each at the ceiling of a
            // call with the prefix uncached.
            + failed_calls(row.calls_json.as_deref()) as f64 * cost(13_500, 0, 300);
        spent += spend;
        times.push(millis);
        let json_reply = serde_json::to_value(&reply).expect("the reply as JSON");
        eprintln!(
            "{:?} | {millis} ms | calls {} | tok in {} cached {} out {} reasoning {} | ${spend:.4} (total ${spent:.4}) | {}",
            reply.outcome, t_calls, t_in, t_cached, t_out, t_reason, question
        );
        let line = json!({
            "question": question,
            "follow_up": follow,
            "outcome": reply.outcome,
            "millis": millis,
            "total_millis": row.total_millis,
            "model_millis": row.model_millis,
            "tool_millis": row.tool_millis,
            "model_calls": t_calls,
            "tokens": {"in": t_in, "cached": t_cached, "out": t_out, "reasoning": t_reason},
            "cost": spend,
            "finish_reason": row.finish_reason,
            "tool_calls": serde_json::from_str::<Value>(&row.tool_calls_json).unwrap_or(Value::Null),
            "calls": row.calls_json.as_deref().and_then(|c| serde_json::from_str::<Value>(c).ok()),
            "answer": shown(&json_reply),
            "failure": json_reply["failure"],
            "answer_json": row.answer_json,
        });
        out.push_str(&line.to_string());
        out.push('\n');
        std::fs::write(&out_path, &out).expect("write the results");
        assert!(
            reply.outcome != AskOutcome::LimitReached,
            "the daily limit stopped the run"
        );
    }
    times.sort_unstable();
    let p = |q: f64| {
        times
            .get(((times.len() as f64 - 1.0) * q).round() as usize)
            .copied()
            .unwrap_or(0)
    };
    eprintln!(
        "DONE {} questions | spent ${spent:.4} of ${budget:.2} | ms: median {} p90 {} p95 {} max {} | results: {}",
        times.len(),
        p(0.5),
        p(0.9),
        p(0.95),
        times.last().copied().unwrap_or(0),
        out_path.display()
    );
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
