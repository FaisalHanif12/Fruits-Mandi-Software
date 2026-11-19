//! The one place the app talks to the model (Tech Stack 4.8, Code Standards Q8, NFR-SEC-9,
//! NFR-SEC-15).
//!
//! [`OpenRouterClient`] sends the request as plain JSON to the one URL [`API_URL`] over HTTPS, with
//! the key in the `Authorization` header, read from the key store just before the call by the
//! caller and dropped after it. A redirect is never followed. Each call is over after the time it
//! is given at the latest, TRUE to the clock (see [`crate::hedge`]: the HTTP client alone restarts
//! its timer when the headers arrive), and a second copy of the request goes out when the first is
//! slow. A call that runs out of time at any stage, waiting for the first byte or reading the body,
//! is a timeout and never an unreadable reply.
//!
//! [`RecordedClient`] replays recorded replies for the tests and keeps the requests it was given, so
//! a test can scan every request body (E2E-54). No commit test calls the network.
//!
//! A reply is used only when `finish_reason` is `stop` or `tool_calls`; the caller treats the rest
//! as "not understood". The usage block gives the tokens for the log and the limits.
//!
//! Example: a reply `{"choices":[{"message":{"role":"assistant","content":"{...}"},
//! "finish_reason":"stop"}],"usage":{"prompt_tokens":5200,"completion_tokens":300}}` is a [`Reply`]
//! with `content` set, no tool calls and 5,200 tokens in.

use std::collections::VecDeque;
use std::io::Read as _;
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use serde_json::Value;

use crate::ask::{AssistantFailure, FailureKind};
use crate::hedge::{Hedge, hedged_with};
use crate::limits::{HEDGE_AFTER_SECS, HEDGE_MIN_COPY_SECS, RETRY_WAIT_MILLIS};
use crate::provider_reason::{MAX_ERROR_BODY_BYTES, reason_of};

/// The only URL the app calls for a question (NFR-SEC-9).
pub const API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
/// The biggest reply read, in bytes. A bigger one is "not understood".
pub const MAX_REPLY_BYTES: usize = 1_000_000;

/// A tool the model asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    /// The id the reply must carry back.
    pub id: String,
    /// The tool.
    pub name: String,
    /// The input, as the model wrote it (JSON text).
    pub arguments: String,
}

/// The tokens of one reply.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    /// Input tokens, cached ones included.
    pub prompt_tokens: i64,
    /// Of those, the cached ones.
    pub cached_tokens: i64,
    /// Output tokens, reasoning included.
    pub completion_tokens: i64,
    /// Of those, the reasoning ones.
    pub reasoning_tokens: i64,
}

/// One usable reply.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    /// `stop`, `tool_calls`, `length` ...
    pub finish_reason: String,
    /// The text of the reply (the final answer as JSON text), if any.
    pub content: Option<String>,
    /// The tools asked for, in order.
    pub tool_calls: Vec<ToolCall>,
    /// The tokens.
    pub usage: Usage,
    /// The model that answered, as the service names it.
    pub model: Option<String>,
    /// The assistant message as sent back to us, to be put into the next request.
    pub message: Value,
}

/// Why a call failed (the reply has no body to use).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallError {
    /// What kind of failure.
    pub kind: FailureKind,
    /// The HTTP status when there was a reply.
    pub http_status: Option<i64>,
    /// What the provider said, scrubbed of keys ([`crate::provider_reason`]). Only for a reply that
    /// was not a success and had a body.
    pub reason: Option<String>,
}

impl CallError {
    /// A failure with no status.
    pub fn new(kind: FailureKind) -> Self {
        Self {
            kind,
            http_status: None,
            reason: None,
        }
    }

    /// True for a failure that one more try can cure (plan 52 item 6): the network (no internet, or
    /// a connection that dropped), HTTP 429, HTTP 5xx and, since the live run of 03-11-2026 (Q237
    /// and Q313: a second call that timed out after 30 seconds ended the question with nothing), a
    /// timeout. A wrong key, no credit, no Azure endpoint, a reply that cannot be read or any other
    /// status is not tried again. The retry is only made when `RETRY_MIN_LEFT_SECS` of the question
    /// are left, so a timeout late in a question is final.
    pub fn retryable(&self) -> bool {
        matches!(
            self.kind,
            FailureKind::Offline
                | FailureKind::Dropped
                | FailureKind::Busy
                | FailureKind::Server
                | FailureKind::Timeout
        )
    }

    /// The failure the panel is told.
    pub fn failure(&self) -> AssistantFailure {
        AssistantFailure {
            kind: self.kind,
            http_status: self.http_status,
            reason: self.reason.clone(),
        }
    }
}

/// Where model calls go.
pub trait ModelClient: Send + Sync {
    /// One call. `key` is used for this call only.
    ///
    /// # Errors
    /// [`CallError`].
    fn call(&self, key: &str, body: &Value, timeout: Duration) -> Result<Reply, CallError>;

    /// [`ModelClient::call`] with a note of what the hedge did, when the client sent a second copy
    /// of the request (plan 55 Section 3). The real client does; the others send one request and
    /// say nothing.
    fn call_noted(
        &self,
        key: &str,
        body: &Value,
        timeout: Duration,
    ) -> (Result<Reply, CallError>, Option<Hedge>) {
        (self.call(key, body, timeout), None)
    }

    /// How long to wait before the one retry of a failed call (plan 52 item 6). The tests' client
    /// says none.
    fn retry_wait(&self) -> Duration {
        Duration::from_millis(RETRY_WAIT_MILLIS)
    }
}

/// A token count: never below 0, whatever the reply says (the log refuses a negative count).
fn int(v: Option<&Value>) -> i64 {
    v.and_then(Value::as_i64).unwrap_or(0).max(0)
}

impl Reply {
    /// Reads a reply in the API's shape.
    ///
    /// # Errors
    /// `bad_reply` when it has no first choice.
    pub fn from_json(value: &Value) -> Result<Reply, CallError> {
        let bad = || CallError::new(FailureKind::BadReply);
        let choice = value
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|c| c.first())
            .ok_or_else(bad)?;
        let message = choice.get("message").cloned().ok_or_else(bad)?;
        let finish_reason = choice
            .get("finish_reason")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let content = message
            .get("content")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let tool_calls = message
            .get("tool_calls")
            .and_then(Value::as_array)
            .map(|calls| {
                calls
                    .iter()
                    .filter_map(|c| {
                        let f = c.get("function")?;
                        Some(ToolCall {
                            id: c.get("id")?.as_str()?.to_owned(),
                            name: f.get("name")?.as_str()?.to_owned(),
                            arguments: f.get("arguments")?.as_str()?.to_owned(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let usage = value.get("usage");
        Ok(Reply {
            finish_reason,
            content,
            tool_calls,
            usage: Usage {
                prompt_tokens: int(usage.and_then(|u| u.get("prompt_tokens"))),
                cached_tokens: int(usage
                    .and_then(|u| u.get("prompt_tokens_details"))
                    .and_then(|d| d.get("cached_tokens"))),
                completion_tokens: int(usage.and_then(|u| u.get("completion_tokens"))),
                reasoning_tokens: int(usage
                    .and_then(|u| u.get("completion_tokens_details"))
                    .and_then(|d| d.get("reasoning_tokens"))),
            },
            model: value
                .get("model")
                .and_then(Value::as_str)
                .map(str::to_owned),
            message,
        })
    }
}

/// The real client.
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenRouterClient;

fn status_kind(status: u16) -> FailureKind {
    match status {
        401 => FailureKind::BadKey,
        402 => FailureKind::NoCredit,
        404 => FailureKind::NoEndpoint,
        429 => FailureKind::Busy,
        500..=599 => FailureKind::Server,
        _ => FailureKind::Other,
    }
}

/// The failure of a request that could not be sent or got no reply. No time left is a timeout. A
/// connection that could not be made (no network, no route, DNS) is `offline`, the one case where
/// "no internet" is true. A connection that was made and then dropped, or any other fault in
/// sending, is `dropped`: the internet may be fine, so the panel does not say it is not (plan 52
/// item 6).
fn send_failure(is_timeout: bool, is_connect: bool) -> CallError {
    CallError::new(if is_timeout {
        FailureKind::Timeout
    } else if is_connect {
        FailureKind::Offline
    } else {
        FailureKind::Dropped
    })
}

/// The failure of a read of the reply body: a call that ran out of time is a timeout, anything else
/// is a reply that could not be read.
fn body_failure(is_timeout: bool) -> CallError {
    CallError::new(if is_timeout {
        FailureKind::Timeout
    } else {
        FailureKind::BadReply
    })
}

/// What the provider said in the body of an error reply (live run 2: a 403 told us nothing). At
/// most [`MAX_ERROR_BODY_BYTES`] are read, and a body that fails to read leaves what came before the
/// failure. The key is cut out of it and anything that looks like a key is hidden before it is kept
/// (NFR-OBS-2, NFR-SEC-7); neither the key nor a header is ever logged.
fn read_reason(response: reqwest::blocking::Response, key: &str) -> Option<String> {
    let mut body = Vec::new();
    // A read that fails half way is not a failure of its own: the status already says what happened.
    let _ = response.take(MAX_ERROR_BODY_BYTES).read_to_end(&mut body);
    reason_of(&body, key)
}

impl OpenRouterClient {
    /// One call to `url`. The app only ever calls [`API_URL`] over HTTPS (`call`); the tests call a
    /// local address over plain HTTP, so the timeouts of every stage can be tried for real.
    fn call_to(
        url: &str,
        https_only: bool,
        key: &str,
        body: &Value,
        timeout: Duration,
    ) -> Result<Reply, CallError> {
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .https_only(https_only)
            .timeout(timeout)
            .build()
            .map_err(|_| CallError::new(FailureKind::Other))?;
        let response = client
            .post(url)
            .bearer_auth(key)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .map_err(|e| send_failure(e.is_timeout(), e.is_connect()))?;
        let status = response.status().as_u16();
        if !response.status().is_success() {
            return Err(CallError {
                kind: status_kind(status),
                http_status: Some(i64::from(status)),
                reason: read_reason(response, key),
            });
        }
        // The timeout covers the body too: a reply that stalls half way is a timeout (plan 51 B3).
        let bytes = response.bytes().map_err(|e| body_failure(e.is_timeout()))?;
        if bytes.len() > MAX_REPLY_BYTES {
            return Err(CallError::new(FailureKind::BadReply));
        }
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|_| CallError::new(FailureKind::BadReply))?;
        Reply::from_json(&value)
    }
}

impl OpenRouterClient {
    /// One call with a true deadline and a second copy when the first is slow (see [`hedged_with`]).
    fn call_hedged(
        url: &str,
        https_only: bool,
        key: &str,
        body: &Value,
        timeout: Duration,
        hedge_after: Duration,
        min_copy: Duration,
    ) -> (Result<Reply, CallError>, Option<Hedge>) {
        let (url, key, body) = (url.to_owned(), key.to_owned(), body.clone());
        hedged_with(
            move |budget| Self::call_to(&url, https_only, &key, &body, budget),
            timeout,
            hedge_after,
            min_copy,
        )
    }
}

impl ModelClient for OpenRouterClient {
    fn call(&self, key: &str, body: &Value, timeout: Duration) -> Result<Reply, CallError> {
        self.call_noted(key, body, timeout).0
    }

    fn call_noted(
        &self,
        key: &str,
        body: &Value,
        timeout: Duration,
    ) -> (Result<Reply, CallError>, Option<Hedge>) {
        Self::call_hedged(
            API_URL,
            true,
            key,
            body,
            timeout,
            Duration::from_secs(HEDGE_AFTER_SECS),
            Duration::from_secs(HEDGE_MIN_COPY_SECS),
        )
    }
}

/// A client that replays recorded replies (tests). Every request it was given is kept.
#[derive(Debug, Default)]
pub struct RecordedClient {
    replies: Mutex<VecDeque<Result<Value, CallError>>>,
    seen: Mutex<Vec<Value>>,
    waits: Mutex<Vec<Duration>>,
}

impl RecordedClient {
    /// Replays these API replies, in order.
    pub fn new(replies: Vec<Value>) -> Self {
        Self::with_results(replies.into_iter().map(Ok).collect())
    }

    /// Replays replies and failures, in order.
    pub fn with_results(results: Vec<Result<Value, CallError>>) -> Self {
        Self {
            replies: Mutex::new(results.into()),
            seen: Mutex::new(Vec::new()),
            waits: Mutex::new(Vec::new()),
        }
    }

    /// The time each call was allowed to wait, in order (the `timeout` the app gave it).
    pub fn waits(&self) -> Vec<Duration> {
        self.waits
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The request bodies it was given, in order.
    pub fn requests(&self) -> Vec<Value> {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl ModelClient for RecordedClient {
    fn retry_wait(&self) -> Duration {
        Duration::ZERO
    }

    fn call(&self, _key: &str, body: &Value, timeout: Duration) -> Result<Reply, CallError> {
        self.waits
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(timeout);
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(body.clone());
        let next = self
            .replies
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front();
        match next {
            Some(Ok(value)) => Reply::from_json(&value),
            Some(Err(error)) => Err(error),
            // A transcript that ran out: the question asked for more calls than were recorded.
            None => Err(CallError::new(FailureKind::Other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_reply_with_tool_calls_and_usage_is_read() {
        let value = json!({
            "model": "openai/gpt-6-luna",
            "choices": [{"finish_reason": "tool_calls", "message": {
                "role": "assistant", "content": null,
                "tool_calls": [{"id": "call_1", "type": "function",
                    "function": {"name": "find", "arguments": "{\"text\":\"ikram\",\"role\":\"buyer\"}"}}]}}],
            "usage": {"prompt_tokens": 5200, "completion_tokens": 300,
                "prompt_tokens_details": {"cached_tokens": 4000},
                "completion_tokens_details": {"reasoning_tokens": 120}}
        });
        let reply = Reply::from_json(&value).unwrap();
        assert_eq!(reply.finish_reason, "tool_calls");
        assert_eq!(reply.tool_calls.len(), 1);
        assert_eq!(reply.tool_calls[0].name, "find");
        assert_eq!(reply.usage.prompt_tokens, 5200);
        assert_eq!(reply.usage.cached_tokens, 4000);
        assert_eq!(reply.usage.reasoning_tokens, 120);
        assert!(reply.content.is_none());
    }

    #[test]
    fn a_negative_token_count_in_a_reply_is_taken_as_zero() {
        let value = json!({
            "choices": [{"finish_reason": "stop", "message": {"content": "{}"}}],
            "usage": {"prompt_tokens": -5, "completion_tokens": -1,
                "prompt_tokens_details": {"cached_tokens": -3},
                "completion_tokens_details": {"reasoning_tokens": -2}}
        });
        let usage = Reply::from_json(&value).unwrap().usage;
        assert_eq!(usage, Usage::default());
    }

    #[test]
    fn a_reply_with_no_choice_is_bad_and_statuses_map_to_kinds() {
        let error = Reply::from_json(&json!({"choices": []})).unwrap_err();
        assert_eq!(error.kind, FailureKind::BadReply);
        assert_eq!(status_kind(401), FailureKind::BadKey);
        assert_eq!(status_kind(402), FailureKind::NoCredit);
        assert_eq!(status_kind(404), FailureKind::NoEndpoint);
        assert_eq!(status_kind(429), FailureKind::Busy);
        assert_eq!(status_kind(503), FailureKind::Server);
        assert_eq!(status_kind(418), FailureKind::Other);
    }

    #[test]
    fn the_recorded_client_replays_in_order_and_keeps_the_requests() {
        let reply = json!({"choices": [{"finish_reason": "stop", "message": {"content": "{}"}}]});
        let client = RecordedClient::new(vec![reply]);
        let ok = client
            .call("k", &json!({"n": 1}), Duration::from_secs(1))
            .unwrap();
        assert_eq!(ok.content.as_deref(), Some("{}"));
        // Out of replies: a failure, not a panic.
        assert!(
            client
                .call("k", &json!({"n": 2}), Duration::from_secs(1))
                .is_err()
        );
        assert_eq!(client.requests(), vec![json!({"n": 1}), json!({"n": 2})]);
    }

    // ---- the timeout of every stage, against a real local server (plan 51 B3) ----------------

    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::Instant;

    /// What a local server does with one request.
    #[derive(Clone, Copy)]
    enum Server {
        /// Takes the request and never answers.
        Silent,
        /// Sends the headers and a part of the body, then stalls.
        StallsInTheBody,
        /// Waits this long, sends the headers and a part of the body, then stalls.
        HeadersLateThenStall(Duration),
        /// Sends a whole reply.
        Replies(&'static str),
        /// Sends a whole reply with this HTTP status.
        Fails(u16, &'static str),
        /// Takes the request and closes the connection with no reply.
        HangsUp,
    }

    fn serve(how: Server, hold: Duration) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            // Read the request head, then the body the client sent with it.
            let mut seen = Vec::new();
            let mut chunk = [0_u8; 4096];
            while !seen.windows(4).any(|w| w == b"\r\n\r\n") {
                match stream.read(&mut chunk) {
                    Ok(0) | Err(_) => return,
                    Ok(n) => seen.extend_from_slice(&chunk[..n]),
                }
            }
            match how {
                Server::HangsUp => {}
                Server::Silent => std::thread::sleep(hold),
                Server::StallsInTheBody => {
                    let _ = stream.write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 5000\r\n\r\n{\"choices\":[",
                    );
                    let _ = stream.flush();
                    std::thread::sleep(hold);
                }
                Server::HeadersLateThenStall(late) => {
                    std::thread::sleep(late);
                    let _ = stream.write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 5000\r\n\r\n{\"choices\":[",
                    );
                    let _ = stream.flush();
                    std::thread::sleep(hold);
                }
                Server::Replies(body) => {
                    let head = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(head.as_bytes());
                    let _ = stream.write_all(body.as_bytes());
                    let _ = stream.flush();
                }
                Server::Fails(status, body) => {
                    let head = format!(
                        "HTTP/1.1 {status} Refused\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(head.as_bytes());
                    let _ = stream.write_all(body.as_bytes());
                    let _ = stream.flush();
                }
            }
        });
        url
    }

    fn call_local(url: &str, timeout: Duration) -> (Result<Reply, CallError>, Duration) {
        let started = Instant::now();
        let result = OpenRouterClient::call_to(url, false, "sk-test", &json!({}), timeout);
        (result, started.elapsed())
    }

    #[test]
    fn a_server_that_never_answers_is_a_timeout() {
        let url = serve(Server::Silent, Duration::from_secs(5));
        let (result, took) = call_local(&url, Duration::from_millis(400));
        assert_eq!(result.unwrap_err().kind, FailureKind::Timeout);
        assert!(took < Duration::from_secs(3), "{took:?}");
    }

    #[test]
    fn a_reply_that_stalls_in_the_body_is_a_timeout_and_not_an_unreadable_reply() {
        // Umar's row 12: the headers came, the body did not, and the call was reported as
        // "the model's answer could not be read".
        let url = serve(Server::StallsInTheBody, Duration::from_secs(5));
        let (result, took) = call_local(&url, Duration::from_millis(400));
        assert_eq!(result.unwrap_err().kind, FailureKind::Timeout);
        assert!(took < Duration::from_secs(3), "{took:?}");
    }

    /// A server that handles one connection for each entry of the script, in the order they come.
    fn serve_script(script: Vec<Server>, hold: Duration) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for how in script {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                std::thread::spawn(move || {
                    let mut seen = Vec::new();
                    let mut chunk = [0_u8; 4096];
                    while !seen.windows(4).any(|w| w == b"\r\n\r\n") {
                        match stream.read(&mut chunk) {
                            Ok(0) | Err(_) => return,
                            Ok(n) => seen.extend_from_slice(&chunk[..n]),
                        }
                    }
                    match how {
                        Server::Silent => std::thread::sleep(hold),
                        Server::Replies(body) => {
                            let head = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                body.len()
                            );
                            let _ = stream.write_all(head.as_bytes());
                            let _ = stream.write_all(body.as_bytes());
                            let _ = stream.flush();
                        }
                        _ => std::thread::sleep(hold),
                    }
                });
            }
        });
        url
    }

    const GOOD: &str = r#"{"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"{}"}}],"usage":{"prompt_tokens":10,"completion_tokens":2}}"#;

    #[test]
    fn headers_that_come_late_and_a_body_that_stalls_end_at_the_cut_and_not_after_it() {
        // The plain HTTP client restarts its timer when the headers arrive: with a cut of 1.0 s and
        // headers at 0.7 s it took 1.7 s (measured 05-11-2026; live run 1: a call of 23.8 s against
        // a cut of 12 s, and six questions past 25 s). The call must end at the cut.
        let url = serve(
            Server::HeadersLateThenStall(Duration::from_millis(700)),
            Duration::from_secs(5),
        );
        let started = Instant::now();
        let (result, hedge) = OpenRouterClient::call_hedged(
            &url,
            false,
            "sk-test",
            &json!({}),
            Duration::from_secs(1),
            Duration::from_secs(60),
            Duration::from_secs(3),
        );
        let took = started.elapsed();
        assert_eq!(result.unwrap_err().kind, FailureKind::Timeout);
        assert!(hedge.is_none(), "the copy was not due");
        assert!(
            took >= Duration::from_millis(950) && took < Duration::from_millis(1_300),
            "{took:?}"
        );
    }

    #[test]
    fn a_silent_first_request_is_answered_by_the_copy_sent_after_the_hedge_time() {
        // The first connection takes the request and never answers; the copy gets a whole reply.
        let url = serve_script(
            vec![Server::Silent, Server::Replies(GOOD)],
            Duration::from_secs(5),
        );
        let started = Instant::now();
        let (result, hedge) = OpenRouterClient::call_hedged(
            &url,
            false,
            "sk-test",
            &json!({}),
            Duration::from_secs(3),
            Duration::from_millis(300),
            Duration::ZERO,
        );
        let took = started.elapsed();
        assert_eq!(result.unwrap().usage.prompt_tokens, 10);
        assert!(
            took >= Duration::from_millis(300) && took < Duration::from_millis(1_200),
            "{took:?}"
        );
        let hedge = hedge.expect("the copy was sent");
        assert_eq!(hedge.attempts.len(), 2);
        assert_eq!(hedge.attempts[1].end, crate::hedge::End::Won);
    }

    #[test]
    fn a_whole_reply_inside_the_time_is_read_and_a_broken_one_is_not_a_timeout() {
        let good = r#"{"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"{}"}}],"usage":{"prompt_tokens":10,"completion_tokens":2}}"#;
        let url = serve(Server::Replies(good), Duration::ZERO);
        let (result, _) = call_local(&url, Duration::from_secs(5));
        assert_eq!(result.unwrap().usage.prompt_tokens, 10);
        let url = serve(Server::Replies("not json at all"), Duration::ZERO);
        let (result, _) = call_local(&url, Duration::from_secs(5));
        assert_eq!(result.unwrap_err().kind, FailureKind::BadReply);
    }

    #[test]
    fn a_connection_that_cannot_be_made_is_offline_and_one_that_drops_is_not() {
        // Nothing listens on this port any more: the connection is refused, which is "no internet".
        let closed = {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            format!("http://{}/", listener.local_addr().unwrap())
        };
        let (result, _) = call_local(&closed, Duration::from_secs(5));
        let error = result.unwrap_err();
        assert_eq!(error.kind, FailureKind::Offline);
        assert!(error.retryable());
        // A server that takes the request and hangs up with no reply: the internet is fine, so
        // the panel must not say it is down. The kind is `dropped`, with no status, and is tried
        // again.
        let url = serve(Server::HangsUp, Duration::ZERO);
        let (result, _) = call_local(&url, Duration::from_secs(5));
        let error = result.unwrap_err();
        assert_eq!(error.kind, FailureKind::Dropped);
        assert_eq!(error.http_status, None);
        assert!(error.retryable());
    }

    /// A key that looks like a real one, used only in these tests. The `zz` keeps it from being hex,
    /// so the scan for real keys in the sources (attack 09) does not take it for one.
    const TEST_KEY: &str = "sk-or-v1-zz0123456789abcdef0123456789abcdef0123456789abcdef";

    fn call_failing(status: u16, body: &'static str) -> CallError {
        let url = serve(Server::Fails(status, body), Duration::ZERO);
        let result =
            OpenRouterClient::call_to(&url, false, TEST_KEY, &json!({}), Duration::from_secs(5));
        result.unwrap_err()
    }

    #[test]
    fn a_403_keeps_what_the_provider_said_next_to_its_status() {
        // Live run 2: a 403 came back as "other, 403" and nothing said why.
        let error = call_failing(
            403,
            r#"{"error":{"code":403,"message":"Your key is not allowed to use this model"}}"#,
        );
        assert_eq!(error.kind, FailureKind::Other);
        assert_eq!(error.http_status, Some(403));
        assert_eq!(
            error.reason.as_deref(),
            Some("403: Your key is not allowed to use this model")
        );
        assert!(!error.retryable());
        // The failure the panel is told carries it, and its JSON names the field.
        let failure = error.failure();
        assert_eq!(failure.http_status, Some(403));
        let json = serde_json::to_value(&failure).unwrap();
        assert_eq!(
            json["reason"],
            "403: Your key is not allowed to use this model"
        );
        assert_eq!(json["http_status"], 403);
    }

    #[test]
    fn a_403_body_that_echoes_the_key_or_a_header_is_scrubbed_before_it_is_kept() {
        // The provider repeats the key, the header and a second key in its message.
        let error = call_failing(
            403,
            r#"{"error":{"code":403,"message":"Rejected sk-or-v1-zz0123456789abcdef0123456789abcdef0123456789abcdef. Request had Authorization: Bearer sk-or-v1-zz0123456789abcdef0123456789abcdef0123456789abcdef and x-api-key: other-secret-value-1"}}"#,
        );
        let reason = error.reason.unwrap();
        for secret in [TEST_KEY, "0123456789abcdef", "other-secret-value-1"] {
            assert!(!reason.contains(secret), "{secret} stayed in {reason}");
        }
        assert!(reason.starts_with("403: Rejected [hidden]."), "{reason}");
        assert!(
            reason.contains("Authorization: Bearer [hidden]"),
            "{reason}"
        );
    }

    #[test]
    fn a_web_address_in_a_403_is_cut_to_its_host() {
        // Live probe: the message names the key's own page, with its id. The id is not the secret
        // but it names the key, so only the host stays.
        let error = call_failing(
            403,
            r#"{"error":{"code":403,"message":"Key limit exceeded (total limit). Manage it at https://openrouter.ai/workspaces/default/keys/k_ab12CD34. $1.01 of $1.00 used."}}"#,
        );
        assert_eq!(
            error.reason.as_deref(),
            Some(
                "403: Key limit exceeded (total limit). Manage it at openrouter.ai/[hidden]. $1.01 of $1.00 used."
            )
        );
    }

    #[test]
    fn a_403_with_a_body_that_is_not_json_gives_its_first_300_characters() {
        let error = call_failing(
            403,
            "<html><body><h1>403 Forbidden</h1><p>This account is blocked in your region.</p></body></html>",
        );
        let reason = error.reason.unwrap();
        assert!(
            reason.starts_with("<html><body><h1>403 Forbidden</h1>"),
            "{reason}"
        );
        assert!(reason.chars().count() <= 300);
        // Other kinds of refusal keep theirs too, and a reply with no body has no reason.
        let busy = call_failing(
            429,
            r#"{"error":{"message":"Rate limit exceeded","code":429}}"#,
        );
        assert_eq!(busy.kind, FailureKind::Busy);
        assert_eq!(busy.reason.as_deref(), Some("429: Rate limit exceeded"));
        assert!(busy.retryable());
        let empty = call_failing(403, "");
        assert_eq!(empty.http_status, Some(403));
        assert_eq!(empty.reason, None);
        assert!(
            serde_json::to_value(empty.failure())
                .unwrap()
                .get("reason")
                .is_none()
        );
    }

    #[test]
    fn a_body_far_past_the_cap_is_read_only_as_far_as_the_cap() {
        // 100 KB of text: the call returns at once with a reason of at most 300 characters.
        let body: &'static str = Box::leak("x ".repeat(50_000).into_boxed_str());
        let error = call_failing(500, body);
        assert_eq!(error.kind, FailureKind::Server);
        assert!(error.reason.unwrap().chars().count() <= 300);
    }

    #[test]
    fn which_failures_are_tried_again() {
        let status = |kind, code| CallError {
            kind,
            http_status: Some(code),
            reason: None,
        };
        for kind in [
            FailureKind::Offline,
            FailureKind::Dropped,
            FailureKind::Busy,
            FailureKind::Server,
        ] {
            assert!(CallError::new(kind).retryable(), "{kind:?}");
        }
        assert!(status(FailureKind::Busy, 429).retryable());
        assert!(status(FailureKind::Server, 503).retryable());
        // A timeout is tried once more too (Q237 and Q313 of the live run of 03-11-2026).
        assert!(CallError::new(FailureKind::Timeout).retryable());
        // Never: a wrong key, no credit, no Azure endpoint, a reply that cannot be read, and an
        // unknown status such as 400.
        for kind in [
            FailureKind::BadKey,
            FailureKind::NoCredit,
            FailureKind::NoEndpoint,
            FailureKind::BadReply,
        ] {
            assert!(!CallError::new(kind).retryable(), "{kind:?}");
        }
        assert!(!status(FailureKind::Other, 400).retryable());
        assert!(!CallError::new(FailureKind::Other).retryable());
        assert_eq!(
            send_failure(true, true).kind,
            FailureKind::Timeout,
            "a timeout wins"
        );
    }

    #[test]
    fn only_a_timeout_of_the_body_read_is_a_timeout() {
        assert_eq!(body_failure(true).kind, FailureKind::Timeout);
        assert_eq!(body_failure(false).kind, FailureKind::BadReply);
    }
}
