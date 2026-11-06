//! The whole path of a question, through `ask()` on the rich year (today 05-11-2026) with a model that
//! only answers "x": what the app read by itself before the first model call (live run 2, H348 and H371).
//!
//! The run starts only from a sure understanding, so a question the planner can read but the
//! understanding calls unsure reads nothing and the model has to find its own way (25 s). The planner's
//! own table is in `understand_reads.rs`; this file holds the path.
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
#[allow(dead_code)]
#[path = "robustness_year/mod.rs"]
mod year;

use harness::Harness;
use serde_json::{Value, json};

fn answer(text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": {"prompt_tokens": 7200, "completion_tokens": 60,
            "prompt_tokens_details": {"cached_tokens": 7000},
            "completion_tokens_details": {"reasoning_tokens": 0}},
    })
}

/// The tools whose results were in the first request to the model, in order.
fn pre_read(request: &Value) -> Vec<String> {
    request["messages"]
        .as_array()
        .map(|messages| {
            messages
                .iter()
                .filter(|m| m["role"] == "tool")
                .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
                .map(|v| v["tool"].as_str().unwrap_or("?").to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn ask_on_the_rich_year(question: &str) -> Vec<String> {
    let y = year::seed::build();
    let h = Harness::over(y.w);
    h.runtime.set_local(true);
    h.runtime.set_read_first(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    let (_, requests) = h.ask(question, None, vec![answer("x"), answer("x")]);
    requests.first().map(pre_read).unwrap_or_default()
}

#[test]
fn the_months_list_and_the_two_months_compare_are_pre_read_through_ask() {
    let months = [
        "period",
        "period",
        "period",
        "find",
        "analytics",
        "analytics",
        "analytics",
    ];
    let compare = ["period", "period", "find", "analytics", "analytics"];
    for (question, want) in [
        (
            "how did onion sales go in August, September and October",
            &months[..],
        ),
        (
            "is potato selling more this month than last month",
            &compare[..],
        ),
    ] {
        assert_eq!(ask_on_the_rich_year(question), want, "{question}");
    }
}

/// A comparison the app cannot read (a balance is no flow) stays unsure: no figure is pre-read for it
/// (at most the day of its period), and the model finds its own way.
#[test]
fn a_comparison_the_app_cannot_read_is_not_made_sure() {
    let tools = ask_on_the_rich_year("akram ka bakaya pichle mahine se kitna zyada hai");
    assert!(
        tools.iter().all(|t| t == "period"),
        "a figure was pre-read for an unsure comparison: {tools:?}"
    );
}
