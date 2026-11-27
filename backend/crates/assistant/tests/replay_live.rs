//! Replays the questions of a copy of a real question log (plan 99 A3).
//!
//! IGNORED: `cargo test` never runs it. It reads a COPY of a Mandi database, takes the typed questions
//! of its `assistant_questions` table (source `model`), and asks them again through the app's own
//! question path (`run::ask`, the function behind `assistant_ask`), each conversation on its own fresh
//! copy of that database, with "today" set to the day the question was first asked. Nothing of the
//! original file is written. The daily count of the copy is cleared first, so no replay ever stops at
//! the limit of 20.
//!
//! A reply that waits for a click ("which one", or a name found in the other role) is clicked once, as
//! the Owner would: the first option, or the record found in the other role.
//!
//! - `MANDI_REPLAY_DB`: the database copy to read (never changed).
//! - `MANDI_REPLAY_OUT`: where to write the JSON results (rewritten after every conversation).
//! - `MANDI_REPLAY_ONLY`: optional, a comma list of log ids (for example `31,32`). A conversation with
//!   one of them is replayed whole, so a follow-up keeps its earlier questions.
//! - `MANDI_REPLAY_OFFLINE=1`: no key and no network: every model call ends as offline.
//! - `MANDI_REPLAY_THREADS`: conversations at once (default 3).
//!
//! The key is read by the app's own key store just before each call. This file never sees it.
#![cfg(test)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use chrono::{TimeZone, Utc};
use mandi_assistant::ask::{
    AskOutcome, AskReply, AssistantAskInput, AssistantPickInput, FailureKind, PickChoice,
};
use mandi_assistant::client::{CallError, ModelClient, OpenRouterClient, Reply};
use mandi_assistant::hedge::Hedge;
use mandi_assistant::keys::MemoryKeyStore;
use mandi_assistant::run::{self, Deps};
use mandi_assistant::runtime::Runtime;
use mandi_reports::dashboard::BackupFacts;
use mandi_services::db::rusqlite::Connection;
use mandi_services::ports::PasswordHasher;
use mandi_services::testing::{TestEnv, fast_hasher};
use mandi_services::{Secret, assistant, auth};
use serde_json::{Value, json};

/// The password the copy's Owner gets, so the replay can sign in. Only the throwaway copy.
const COPY_PASSWORD: &str = "replay-only-1";

/// The real client, with each reply kept as the API gave it, so a checked answer can become a
/// recorded golden transcript.
struct Recording<'a> {
    inner: &'a dyn ModelClient,
    replies: Mutex<Vec<Value>>,
}

impl Recording<'_> {
    fn keep(&self, reply: &Result<Reply, CallError>) {
        if let Ok(reply) = reply {
            self.replies.lock().expect("the replies").push(json!({
                "choices": [{"finish_reason": reply.finish_reason, "message": reply.message}],
            }));
        }
    }
}

/// Every method goes to the real client, its hedged call too, so a replay is timed as the app is.
impl ModelClient for Recording<'_> {
    fn call(&self, key: &str, body: &Value, timeout: Duration) -> Result<Reply, CallError> {
        let reply = self.inner.call(key, body, timeout);
        self.keep(&reply);
        reply
    }

    fn call_noted(
        &self,
        key: &str,
        body: &Value,
        timeout: Duration,
    ) -> (Result<Reply, CallError>, Option<Hedge>) {
        let (reply, hedge) = self.inner.call_noted(key, body, timeout);
        self.keep(&reply);
        (reply, hedge)
    }

    fn retry_wait(&self) -> Duration {
        self.inner.retry_wait()
    }

    fn clock_skew(&self) -> Duration {
        self.inner.clock_skew()
    }
}

#[derive(Clone)]
struct Asked {
    id: i64,
    text: String,
    day: String,
}

struct OfflineClient;

impl ModelClient for OfflineClient {
    fn call(&self, _key: &str, _body: &Value, _timeout: Duration) -> Result<Reply, CallError> {
        Err(CallError::new(FailureKind::Offline))
    }

    fn retry_wait(&self) -> Duration {
        Duration::ZERO
    }
}

/// The typed questions of the log, by conversation, in the order they were asked.
fn conversations(db: &Path, only: &[i64]) -> Vec<Vec<Asked>> {
    let conn = Connection::open(db).expect("the copy opens");
    let mut statement = conn
        .prepare(
            "SELECT id, conversation_id, question, asked_day FROM assistant_questions \
             WHERE source = 'model' ORDER BY id",
        )
        .expect("the log reads");
    let rows = statement
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .expect("the log rows");
    let mut by_conversation: BTreeMap<String, Vec<Asked>> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for row in rows {
        let (id, conversation, text, day) = row.expect("a log row");
        if !by_conversation.contains_key(&conversation) {
            order.push(conversation.clone());
        }
        by_conversation
            .entry(conversation)
            .or_default()
            .push(Asked { id, text, day });
    }
    order
        .into_iter()
        .filter_map(|c| by_conversation.remove(&c))
        .filter(|list| only.is_empty() || list.iter().any(|a| only.contains(&a.id)))
        .collect()
}

/// A fresh copy of the books in a temp folder, with the log emptied, the Owner's password replaced and
/// no wait left from a wrong try. The assistant stays as the log's Owner set it.
fn fresh_copy(source: &Path) -> (tempfile::TempDir, TestEnv) {
    let dir = tempfile::tempdir().expect("a temp folder");
    let data = dir.path().join("data");
    std::fs::create_dir_all(&data).expect("the data folder");
    let target = data.join("mandi.db");
    std::fs::copy(source, &target).expect("the copy");
    {
        let conn = Connection::open(&target).expect("the copy opens");
        let hasher = fast_hasher().expect("a hasher");
        let hash = hasher
            .hash(&Secret::new(COPY_PASSWORD))
            .expect("the password hash");
        conn.execute(
            "UPDATE users SET password_hash = ?1, failed_attempts = 0, locked_until_at = NULL",
            [hash],
        )
        .expect("the password");
        let trigger: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'trigger' \
                 AND name = 'assistant_questions_no_delete'",
                [],
                |r| r.get(0),
            )
            .expect("the delete trigger");
        conn.execute_batch(
            "DROP TRIGGER assistant_questions_no_delete; DELETE FROM assistant_questions;",
        )
        .expect("the log is emptied");
        conn.execute_batch(&trigger).expect("the trigger is back");
    }
    let env = TestEnv::new(dir.path()).expect("the books open");
    (dir, env)
}

fn set_day(env: &TestEnv, day: &str) {
    let mut parts = day
        .split('-')
        .map(|p| p.parse::<u32>().expect("a day part"));
    let (y, m, d) = (
        parts.next().expect("a year"),
        parts.next().expect("a month"),
        parts.next().expect("a day"),
    );
    let at = Utc
        .with_ymd_and_hms(i32::try_from(y).expect("a year"), m, d, 9, 0, 0)
        .single()
        .expect("the time");
    env.clock.set(at);
}

fn ask_one(
    env: &TestEnv,
    actor: &mandi_services::auth::Actor,
    runtime: &Runtime,
    client: &dyn ModelClient,
    asked: &Asked,
    conversation: &str,
) -> Value {
    let steps: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let emit = |step: mandi_assistant::step::AssistantStep| {
        steps.lock().expect("the steps").push(step.id);
    };
    let deps = Deps {
        ctx: &env.ctx,
        actor,
        runtime,
        client,
        emit: &emit,
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };
    let started = Instant::now();
    let reply = run::ask(
        &deps,
        &AssistantAskInput {
            text: asked.text.clone(),
            conversation_id: Some(conversation.to_owned()),
        },
    );
    let millis = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);
    let reply: AskReply = match reply {
        Ok(reply) => reply,
        Err(error) => {
            return json!({"id": asked.id, "question": asked.text, "day": asked.day,
                          "error": format!("{error:?}"), "millis": millis});
        }
    };
    let mut row = log_row(env, &reply, asked, millis);
    // One click, as the Owner would: the record found in the other role, or the first option.
    let choice = match reply.outcome {
        AskOutcome::WhichOne => reply.options.first().map(|o| o.key.clone()),
        AskOutcome::NotFound => reply
            .found_as
            .as_ref()
            .and(reply.options.first())
            .map(|o| o.key.clone()),
        _ => None,
    };
    if let Some(key) = choice {
        let started = Instant::now();
        let clicked = run::pick(
            &deps,
            &AssistantPickInput {
                question_id: reply.question_id,
                choice: PickChoice::Match { key },
            },
        );
        let more = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);
        match clicked {
            Ok(clicked) => row["clicked"] = log_row(env, &clicked, asked, more),
            Err(error) => row["clicked"] = json!({"error": format!("{error:?}")}),
        }
    }
    row
}

/// The reply and what the app wrote in its own log about it.
fn log_row(env: &TestEnv, reply: &AskReply, asked: &Asked, millis: i64) -> Value {
    let row = assistant::get_question(&env.ctx, reply.question_id)
        .expect("the log reads")
        .expect("the log row");
    json!({
        "id": asked.id,
        "question": asked.text,
        "day": asked.day,
        "outcome": serde_json::to_value(reply.outcome).expect("the outcome"),
        "reply": serde_json::to_value(reply).expect("the reply"),
        "millis": millis,
        "model_calls": row.model_calls,
        "finish_reason": row.finish_reason,
        "tokens": {"in": row.tokens_in, "cached": row.tokens_cached,
                   "out": row.tokens_out, "reasoning": row.tokens_reasoning},
        "total_millis": row.total_millis,
        "model_millis": row.model_millis,
        "tool_millis": row.tool_millis,
        "calls_json": row.calls_json,
        "tool_calls_json": row.tool_calls_json,
        "answer_json": row.answer_json,
    })
}

#[test]
#[ignore = "live: replays a copy of a real question log; calls the model unless MANDI_REPLAY_OFFLINE=1"]
fn replay_the_log() {
    let source = PathBuf::from(std::env::var("MANDI_REPLAY_DB").expect("MANDI_REPLAY_DB"));
    let out = std::env::var("MANDI_REPLAY_OUT").expect("MANDI_REPLAY_OUT");
    let only: Vec<i64> = std::env::var("MANDI_REPLAY_ONLY")
        .map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
        .unwrap_or_default();
    let offline = std::env::var("MANDI_REPLAY_OFFLINE").as_deref() == Ok("1");
    let threads: usize = std::env::var("MANDI_REPLAY_THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
        .clamp(1, 8);
    if !offline {
        // Only whether a key is there, or why it cannot be read, is ever printed: never the key.
        let saved = Runtime::real().keys().get().map(|k| k.is_some());
        assert!(
            matches!(saved, Ok(true)),
            "the app's key store gave no key: {saved:?}"
        );
    }
    // The list is read from a copy, so the source file is never opened.
    let reading = tempfile::tempdir().expect("a temp folder");
    let list_copy = reading.path().join("list.db");
    std::fs::copy(&source, &list_copy).expect("the copy");
    let list = conversations(&list_copy, &only);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let rows: Mutex<Vec<Value>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let at = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let Some(conversation) = list.get(at) else {
                        return;
                    };
                    let (_dir, env) = fresh_copy(&source);
                    let runtime = if offline {
                        Runtime::new(std::sync::Arc::new(MemoryKeyStore::with_key(
                            "offline-run-this-is-not-a-key",
                        )))
                    } else {
                        Runtime::real()
                    };
                    let real = OpenRouterClient;
                    let none = OfflineClient;
                    let client: &dyn ModelClient = if offline { &none } else { &real };
                    let first_day = &conversation[0].day;
                    set_day(&env, first_day);
                    auth::login(&env.ctx, "owner", &Secret::new(COPY_PASSWORD))
                        .expect("the Owner signs in");
                    let actor = auth::require(&env.ctx).expect("signed in");
                    let id = runtime.new_conversation_id(&env.ctx);
                    for asked in conversation {
                        set_day(&env, &asked.day);
                        let recording = Recording {
                            inner: client,
                            replies: Mutex::new(Vec::new()),
                        };
                        let mut row = ask_one(&env, &actor, &runtime, &recording, asked, &id);
                        row["replies"] =
                            Value::Array(recording.replies.into_inner().expect("the replies"));
                        eprintln!(
                            "{} {} ({} ms)",
                            asked.id,
                            row["outcome"].as_str().unwrap_or("error"),
                            row["millis"]
                        );
                        let mut rows = rows.lock().expect("the rows");
                        rows.push(row);
                        let mut sorted = rows.clone();
                        sorted.sort_by_key(|r| r["id"].as_i64().unwrap_or(0));
                        std::fs::write(&out, json!({"results": sorted}).to_string())
                            .expect("the results file");
                    }
                }
            });
        }
    });
}
