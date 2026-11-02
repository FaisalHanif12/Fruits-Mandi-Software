//! The loop with refusals that teach (plan 51 P51-B2): the model's first try is refused with the
//! words that are allowed, and its second try, with one of them, is answered. This is Umar's
//! question "apni sale ko ziyada kaise karain" with the model replaced by a recorded transcript.
//! Before the fix, the same first try gave an empty reason and no list, the model guessed again, and
//! the question ended as "not understood" after 4 calls and 8 tools.
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

use harness::Harness;
use mandi_assistant::ask::AskOutcome;
use serde_json::{Value, json};

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_200, "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 7_000},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

fn tool_reply(calls: &[(&str, Value)]) -> Value {
    let tool_calls: Vec<Value> = calls
        .iter()
        .enumerate()
        .map(|(i, (name, args))| {
            json!({"id": format!("call_{i}"), "type": "function",
                   "function": {"name": name, "arguments": args.to_string()}})
        })
        .collect();
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls",
                     "message": {"role": "assistant", "content": null, "tool_calls": tool_calls}}],
        "usage": usage(),
    })
}

fn final_reply(content: &Value) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop",
                     "message": {"role": "assistant", "content": content.to_string()}}],
        "usage": usage(),
    })
}

fn period(kind: &str) -> (&'static str, Value) {
    (
        "period",
        json!({"kind": kind, "month": null, "day": null, "from": null, "to": null, "n": null}),
    )
}

fn analytics(view: &str, extra: Value) -> (&'static str, Value) {
    let mut base = json!({
        "view": view, "from": null, "to": null, "group_by": null, "item": null, "seller": null,
        "buyer": null, "city": null, "adda": null, "garden": null, "trip": null, "goods": null,
        "partner": null, "side": null, "compare": null, "sort": null, "rows": null
    });
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    ("analytics", base)
}

fn compared(view: &str, column: &str) -> (&'static str, Value) {
    analytics(
        view,
        json!({"from": "r1.from", "to": "r1.to", "compare": "previous",
               "sort": [{"column": column, "order": "desc"}], "rows": 10, "goods": "all", "side": "both"}),
    )
}

/// The tool messages of a request, in order, as text.
fn tool_messages(request: &Value) -> Vec<String> {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .map(|m| m["content"].as_str().unwrap().to_owned())
        .collect()
}

fn advice() -> Value {
    json!({
        "kind": "advice",
        "text": "سب سے زیادہ کمیشن {{r2.rows.i1.name}} سے آیا: {{r2.rows.i1.total_comm}}۔\n• سب سے بڑا بیوپاری {{r3.rows.a18.name}} ہے: {{r3.rows.a18.gross}}۔",
        "show": [{"result": "r2", "as": "table", "rows": null}],
        "next": [], "choose": null, "entry": null, "nearest": null
    })
}

#[test]
fn a_wrong_sort_word_is_corrected_in_one_try_and_the_question_is_answered() {
    let h = Harness::day();
    let replies = vec![
        // Call 1: the period.
        tool_reply(&[period("this_year")]),
        // Call 2: three views sorted on `sales`, which is no column.
        tool_reply(&[
            compared("A", "sales"),
            compared("B", "sales"),
            compared("H", "sales"),
        ]),
        // Call 3: the same views with words from the refusals.
        tool_reply(&[
            compared("A", "total_comm"),
            compared("B", "gross"),
            compared("H", "total_comm"),
        ]),
        // Call 4: the answer.
        final_reply(&advice()),
    ];
    let (reply, requests) = h.ask("apni sale ko ziyada kaise karain ?", None, replies);
    assert_eq!(reply.outcome, AskOutcome::Advice, "{reply:#?}");
    assert_eq!(requests.len(), 4);
    // What the model read after its first try: three refusals, each with the field, a reason and
    // the columns of its own view.
    let refusals: Vec<Value> = tool_messages(&requests[2])
        .iter()
        .filter(|m| m.contains("ERR_OUT_OF_RANGE"))
        .map(|m| serde_json::from_str(m).unwrap())
        .collect();
    assert_eq!(refusals.len(), 3);
    for (refusal, view_word) in refusals.iter().zip(["gross", "net", "rank"]) {
        let e = &refusal["error"];
        assert_eq!(e["field"], "sort.column");
        assert!(!e["detail"].as_str().unwrap().is_empty());
        let allowed: Vec<&str> = e["allowed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| w.as_str().unwrap())
            .collect();
        assert!(allowed.contains(&"total_comm"), "{allowed:?}");
        assert!(allowed.contains(&"difference.total_comm"), "{allowed:?}");
        assert!(allowed.contains(&view_word), "{view_word} in {allowed:?}");
        assert!(!allowed.contains(&"sales"));
    }
    h.scan_requests();
}

#[test]
fn an_up_to_now_comparison_is_refused_with_the_reason_and_corrected_with_a_closed_period() {
    let h = Harness::day();
    let replies = vec![
        // Call 1: up to now, whose from is empty.
        tool_reply(&[period("up_to_now")]),
        // Call 2: a comparison over it, as the playbook line "commission kaise barhe" suggests.
        tool_reply(&[analytics(
            "A",
            json!({"from": "r1.from", "to": "r1.to", "compare": "previous", "rows": 10}),
        )]),
        // Call 3: the reason named both dates and a closed period, so it asks for one and uses it
        // in the same batch (the second tool names the id the first will get, r2).
        tool_reply(&[
            period("this_year"),
            analytics(
                "A",
                json!({"from": "r2.from", "to": "r2.to", "compare": "previous", "rows": 10,
                       "sort": [{"column": "total_comm", "order": "desc"}]}),
            ),
        ]),
        final_reply(&json!({
            "kind": "advice",
            "text": "سب سے زیادہ کمیشن {{r3.rows.i1.name}} سے آیا: {{r3.rows.i1.total_comm}}۔",
            "show": [], "next": [], "choose": null, "entry": null, "nearest": null
        })),
    ];
    let (reply, requests) = h.ask("abhi tak ki sale kaise barhaun", None, replies);
    assert_eq!(reply.outcome, AskOutcome::Advice, "{reply:#?}");
    assert_eq!(requests.len(), 4);
    let refusals: Vec<String> = tool_messages(&requests[2])
        .into_iter()
        .filter(|m| m.contains("ERR_REQUIRED"))
        .collect();
    assert_eq!(refusals.len(), 1, "{:?}", tool_messages(&requests[2]));
    let first: Value = serde_json::from_str(&refusals[0]).unwrap();
    assert_eq!(first["error"]["field"], "from");
    let detail = first["error"]["detail"].as_str().unwrap();
    assert!(
        detail.contains("both dates") && detail.contains("this_year"),
        "{detail}"
    );
    assert!(!detail.contains("filters."));
    h.scan_requests();
}

#[test]
fn a_period_and_the_view_that_uses_it_in_one_call_work_for_up_to_now() {
    // Two calls and the answer, not three: the second tool names the id the first will get, and
    // the empty `from` of up_to_now means "from the first day of the books".
    let h = Harness::day();
    let replies = vec![
        tool_reply(&[
            period("up_to_now"),
            analytics(
                "A",
                json!({"from": "r1.from", "to": "r1.to", "rows": 10,
                       "sort": [{"column": "total_comm", "order": "desc"}]}),
            ),
        ]),
        final_reply(&json!({
            "kind": "answer",
            "text": "اب تک سب سے زیادہ کمیشن {{r2.rows.i1.name}} سے آیا: {{r2.rows.i1.total_comm}}۔",
            "show": [], "next": [], "choose": null, "entry": null, "nearest": null
        })),
    ];
    let (reply, requests) = h.ask("ab tak ki commission kis item se zyada", None, replies);
    assert_eq!(requests.len(), 2);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let results: Vec<String> = tool_messages(&requests[1]);
    assert_eq!(results.len(), 2);
    assert!(results[1].contains("\"total_comm\""), "{}", results[1]);
    assert!(!results[1].contains("ERR_"), "{}", results[1]);
    h.scan_requests();
}
