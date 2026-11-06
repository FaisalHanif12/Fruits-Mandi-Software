//! Plan 55 Section 3 after live run 1 (the lead's decisions of 05-11-2026): a model call has a TRUE
//! deadline, a slow call is sent twice, and no question ever passes 25 seconds.
//!
//! The model here is a small web server on this computer that does what Azure did in the live run:
//! it sends the headers late and then stalls the body, or takes the request and says nothing. The
//! client is the real combination of the app (a request with a timeout, behind `hedged_with`),
//! pointed at that server. The tests really wait: about 25 s and about 8 s.
//!
//! The plain HTTP client restarts its timer when the headers arrive (measured: cut 1.0 s, headers
//! at 0.7 s, call ended at 1.7 s), so before the fix the first test ended at about 26.5 s.
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

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use harness::Harness;
use mandi_assistant::ask::{AskOutcome, AssistantAskInput, FailureKind};
use mandi_assistant::client::{CallError, ModelClient, Reply};
use mandi_assistant::hedge::{Hedge, hedged_with};
use mandi_assistant::limits::{HEDGE_AFTER_SECS, HEDGE_MIN_COPY_SECS};
use mandi_assistant::run::ask;
use mandi_assistant::step::AssistantStep;
use serde_json::{Value, json};

/// What the local server does with one request (the nth connection gets the nth entry; any later
/// one gets the last).
#[derive(Clone)]
enum Plan {
    /// Waits, then sends this whole reply.
    ReplyAfter(Duration, String),
    /// Waits, then sends the headers and a part of the body and stalls.
    HeadersThenStall(Duration),
    /// Takes the request and says nothing.
    Silent,
}

fn serve(plans: Vec<Plan>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port");
    let url = format!("http://{}/", listener.local_addr().expect("an address"));
    let seen = Arc::new(AtomicUsize::new(0));
    std::thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            let n = seen.fetch_add(1, Ordering::SeqCst);
            let plan = plans
                .get(n)
                .or_else(|| plans.last())
                .cloned()
                .expect("a plan");
            std::thread::spawn(move || {
                let mut got = Vec::new();
                let mut chunk = [0_u8; 8192];
                // Read the head and then the body the client sent with it.
                let mut head_end = None;
                let mut want = usize::MAX;
                loop {
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => return,
                        Ok(read) => got.extend_from_slice(&chunk[..read]),
                    }
                    if head_end.is_none() {
                        head_end = got.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4);
                        if let Some(end) = head_end {
                            let head = String::from_utf8_lossy(&got[..end]).to_lowercase();
                            let length = head
                                .lines()
                                .find_map(|l| l.strip_prefix("content-length:"))
                                .and_then(|v| v.trim().parse::<usize>().ok())
                                .unwrap_or(0);
                            want = end + length;
                        }
                    }
                    if got.len() >= want {
                        break;
                    }
                }
                match plan {
                    Plan::ReplyAfter(wait, body) => {
                        std::thread::sleep(wait);
                        let head = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = stream.write_all(head.as_bytes());
                        let _ = stream.write_all(body.as_bytes());
                        let _ = stream.flush();
                    }
                    Plan::HeadersThenStall(wait) => {
                        std::thread::sleep(wait);
                        let _ = stream.write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 50000\r\n\r\n{\"choices\":[",
                        );
                        let _ = stream.flush();
                        std::thread::sleep(Duration::from_secs(120));
                    }
                    Plan::Silent => std::thread::sleep(Duration::from_secs(120)),
                }
            });
        }
    });
    url
}

/// The one request of the real client: a timeout of its own, and the reply read the way the app
/// reads it.
fn send(url: &str, body: &Value, budget: Duration) -> Result<Reply, CallError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(budget)
        .build()
        .map_err(|_| CallError::new(FailureKind::Other))?;
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .map_err(|e| {
            CallError::new(if e.is_timeout() {
                FailureKind::Timeout
            } else {
                FailureKind::Dropped
            })
        })?;
    let bytes = response.bytes().map_err(|e| {
        CallError::new(if e.is_timeout() {
            FailureKind::Timeout
        } else {
            FailureKind::BadReply
        })
    })?;
    let value: Value =
        serde_json::from_slice(&bytes).map_err(|_| CallError::new(FailureKind::BadReply))?;
    Reply::from_json(&value)
}

/// The app's client, pointed at the local server: the same hedge and the same limits.
struct LocalClient {
    url: String,
}

impl ModelClient for LocalClient {
    fn call(&self, key: &str, body: &Value, timeout: Duration) -> Result<Reply, CallError> {
        self.call_noted(key, body, timeout).0
    }

    fn call_noted(
        &self,
        _key: &str,
        body: &Value,
        timeout: Duration,
    ) -> (Result<Reply, CallError>, Option<Hedge>) {
        let (url, body) = (self.url.clone(), body.clone());
        hedged_with(
            move |budget| send(&url, &body, budget),
            timeout,
            Duration::from_secs(HEDGE_AFTER_SECS),
            Duration::from_secs(HEDGE_MIN_COPY_SECS),
        )
    }

    fn retry_wait(&self) -> Duration {
        Duration::ZERO
    }
}

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_200, "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 7_000},
        "completion_tokens_details": {"reasoning_tokens": 0}
    })
}

/// A reply that asks for the `find` of a buyer.
fn find_reply() -> String {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null,
            "tool_calls": [{"id": "call_0", "type": "function", "function": {
                "name": "find",
                "arguments": json!({"text": "ikram", "role": "buyer"}).to_string()}}]}}],
        "usage": usage(),
    })
    .to_string()
}

/// A reply that finishes the question with a few plain words.
fn words_reply() -> String {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": "جی", "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
    .to_string()
}

fn ask_with(
    h: &Harness,
    client: &LocalClient,
    text: &str,
) -> (mandi_assistant::ask::AskReply, Duration) {
    let emit = |_s: AssistantStep| {};
    let deps = h.deps(client, &emit);
    let started = Instant::now();
    let reply = ask(
        &deps,
        &AssistantAskInput {
            text: text.to_owned(),
            conversation_id: None,
        },
    )
    .expect("ask");
    (reply, started.elapsed())
}

fn calls_of_last_question(h: &Harness) -> Vec<Value> {
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    serde_json::from_str(rows[0].calls_json.as_deref().unwrap_or("[]")).unwrap()
}

#[test]
fn a_call_that_sends_headers_and_then_stalls_the_body_never_takes_a_question_past_25_seconds() {
    // Call 1 comes back after 3 s (the model asks for a find). Call 2, every copy and the retry
    // get the headers after 11.5 s and then nothing. The cut of call 2 is 12 s, so before the fix
    // it ended at 3 + 11.5 + 12 = 26.5 s. Now it ends at its cut, the one retry has what is left,
    // and the question is over at 25 s.
    let url = serve(vec![
        Plan::ReplyAfter(Duration::from_secs(3), find_reply()),
        Plan::HeadersThenStall(Duration::from_millis(11_500)),
    ]);
    let h = Harness::day();
    let client = LocalClient { url };
    let (reply, took) = ask_with(&h, &client, "ikram ke total bakaya kitne hai");
    assert!(
        took < Duration::from_millis(25_600),
        "the question took {took:?}: {:?}",
        reply.outcome
    );
    assert!(
        took > Duration::from_secs(24),
        "it used all its time: {took:?}"
    );
    // The read-ahead after the find had read his card, so the slow line shows his checked balance
    // as `time_limit` (2,880 بنام), and not an error and not a guess.
    assert_eq!(reply.outcome, AskOutcome::TimeLimit, "{reply:#?}");
    let shown = harness::figures(harness::answer_of(&reply));
    assert_eq!(shown.len(), 1, "{reply:#?}");
    assert_eq!(
        harness::balance(shown[0].0),
        (2_880, mandi_services::core::accounts::BalanceSide::Dr)
    );
    // The copy of call 2 is in the log, marked hedged.
    let calls = calls_of_last_question(&h);
    assert!(
        calls.iter().any(|c| c["hedged"] == "copy"),
        "the copy is logged: {calls:?}"
    );
}

#[test]
fn when_the_first_request_says_nothing_the_copy_answers_and_the_call_counts_once() {
    // The first request is taken and never answered; at 7 s the copy goes out and is answered at
    // once. The question is answered at about 7 s instead of waiting for the cut.
    let url = serve(vec![
        Plan::Silent,
        Plan::ReplyAfter(Duration::ZERO, words_reply()),
    ]);
    let h = Harness::day();
    let client = LocalClient { url };
    let (reply, took) = ask_with(&h, &client, "kya haal hai");
    assert!(
        took >= Duration::from_secs(7) && took < Duration::from_secs(9),
        "{took:?}: {:?}",
        reply.outcome
    );
    assert_ne!(reply.outcome, AskOutcome::Error, "{reply:#?}");
    // Counted once: one model call, one question today.
    assert_eq!(reply.count_today, 1);
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    assert_eq!(rows[0].model_calls, Some(1));
    // Both requests are in calls_json: the first lost, the copy won. The lost one is not in the
    // model time of the question.
    let calls = calls_of_last_question(&h);
    let lost = calls
        .iter()
        .find(|c| c["hedged"] == "first")
        .expect("the first request is logged");
    assert_eq!(lost["error"], "hedge_lost");
    assert_eq!(lost["hedge_extra"], true);
    let won = calls
        .iter()
        .find(|c| c["hedged"] == "copy")
        .expect("the copy is logged");
    assert_eq!(won["failed"], false);
    assert!(won.get("hedge_extra").is_none());
    let sum: i64 = calls
        .iter()
        .filter(|c| c.get("hedge_extra").is_none())
        .map(|c| c["model_ms"].as_i64().unwrap_or(0))
        .sum();
    assert_eq!(rows[0].model_millis, Some(sum));
}
