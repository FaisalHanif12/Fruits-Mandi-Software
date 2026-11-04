//! Plan 54 S3 and S4: the books are read BEFORE the first model call when the app understood the
//! question, so the first call has only to write the answer (one call), and a call whose data is
//! already read thinks "low".
//!
//! Each test asks with a recorded model that has one reply, then reads the first request the model
//! got: it must already hold the results of the tools the model would have chosen.
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

use harness::{Harness, answer_of, balance, figures, money};
use mandi_assistant::ask::AskOutcome;
use mandi_assistant::client::RecordedClient;
use mandi_services::core::accounts::BalanceSide;
use serde_json::{Value, json};

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_200, "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 7_000},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

fn answer_reply(text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

/// A harness with the reads before the first call switched on.
fn day() -> Harness {
    let h = Harness::day();
    h.runtime.set_read_first(true);
    h.runtime.set_low_effort(true);
    h
}

/// The tools whose results are in a request, as (result id, tool), in order.
fn results_in(request: &Value) -> Vec<(String, String)> {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .filter_map(|v| Some((v["id"].as_str()?.to_owned(), v["tool"].as_str()?.to_owned())))
        .collect()
}

fn tools_of(results: &[(String, String)]) -> Vec<&str> {
    results.iter().map(|(_, tool)| tool.as_str()).collect()
}

/// The effort of the first request.
fn effort(request: &Value) -> String {
    request["reasoning"]["effort"].as_str().unwrap().to_owned()
}

#[test]
fn a_partys_balance_is_read_before_the_first_call_and_one_call_answers() {
    // Pass one: what the first call already holds.
    let h = day();
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with("akram ka bakaya kitna hai", None, &client);
    let first = &client.requests()[0];
    let results = results_in(first);
    assert_eq!(tools_of(&results), ["find", "party_card"], "{results:?}");
    assert_eq!(
        effort(first),
        "low",
        "the data is read: the model only writes"
    );
    // Pass two: the model writes at once, citing what was read. One call, two ✓ places agree.
    let (find, card) = (&results[0].0, &results[1].0);
    let text = format!("{{{{{find}.matches.0.name}}}} کے ذمے {{{{{card}.balance}}}} باقی ہیں۔");
    let h = day();
    let client = RecordedClient::new(vec![answer_reply(&text)]);
    let reply = h.ask_with("akram ka bakaya kitna hai", None, &client);
    assert_eq!(client.requests().len(), 1, "one model call");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(reply.count_today, 1, "one question, counted once");
    let figs = figures(answer_of(&reply));
    assert_eq!(balance(figs[0].0), (2_880, BalanceSide::Dr));
    assert!(figs[0].1, "checked");
    h.scan_requests();
}

#[test]
fn commission_of_an_item_for_a_period_is_read_with_its_dates() {
    let h = day();
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with("kele ka commission is mahine", None, &client);
    let first = &client.requests()[0];
    let results = results_in(first);
    let analytics = results
        .iter()
        .find(|(_, tool)| tool == "analytics")
        .unwrap_or_else(|| panic!("no view read: {results:?}"));
    // The view is A, for banana, from the 1st to today.
    let message = first["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .find(|v| v["id"] == analytics.0.as_str())
        .unwrap();
    assert_eq!(message["source"], "A");
    assert_eq!(message["filters"]["from"], "2026-10-01");
    assert_eq!(message["filters"]["to"], "2026-10-10");
    assert_eq!(message["rows"].as_array().unwrap().len(), 1, "banana only");
    // And the model answers at once with the checked figure of the row (3,928, E2E-49).
    let text = format!("{{{{{id}.rows.i1.total_comm}}}}", id = analytics.0);
    let h = day();
    let client = RecordedClient::new(vec![answer_reply(&text)]);
    let reply = h.ask_with("kele ka commission is mahine", None, &client);
    assert_eq!(client.requests().len(), 1, "one model call");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(money(figures(answer_of(&reply))[0].0), 3_928);
}

#[test]
fn the_books_as_a_whole_are_read_from_the_tiles() {
    let h = day();
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with("cash kitna hai", None, &client);
    let results = results_in(&client.requests()[0]);
    assert_eq!(tools_of(&results), ["tiles"], "{results:?}");
}

#[test]
fn a_question_the_app_is_not_sure_of_reads_nothing_first_and_thinks_medium() {
    let h = day();
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with("akram ke kharche kam karne ke tareeke batao", None, &client);
    let first = &client.requests()[0];
    assert!(results_in(first).is_empty());
    assert_eq!(effort(first), "medium");
}

#[test]
fn the_switches_turn_the_reads_and_the_low_effort_off() {
    let h = Harness::day();
    h.runtime.set_read_first(false);
    h.runtime.set_low_effort(true);
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with("akram ka bakaya kitna hai", None, &client);
    let first = &client.requests()[0];
    assert!(results_in(first).is_empty(), "nothing read first");
    assert_eq!(effort(first), "medium");
    // The reads on, the low effort off: the data is read and the model still thinks medium.
    let h = Harness::day();
    h.runtime.set_read_first(true);
    h.runtime.set_low_effort(false);
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with("akram ka bakaya kitna hai", None, &client);
    let first = &client.requests()[0];
    assert!(!results_in(first).is_empty());
    assert_eq!(effort(first), "medium");
}

/// The tools whose results the first request holds, for a question asked with the reads on.
fn read_for(question: &str) -> Vec<String> {
    let h = day();
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with(question, None, &client);
    let results = results_in(&client.requests()[0]);
    results.into_iter().map(|(_, tool)| tool).collect()
}

/// The tool results of the first request: (tool, source, filters), in order.
fn reads_with_filters(question: &str) -> Vec<(String, String, Value)> {
    let h = day();
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with(question, None, &client);
    client.requests()[0]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .map(|v| {
            (
                v["tool"].as_str().unwrap_or_default().to_owned(),
                v["source"].as_str().unwrap_or_default().to_owned(),
                v["filters"].clone(),
            )
        })
        .collect()
}

#[test]
fn a_party_question_with_a_period_is_read_with_that_period_never_for_all_time() {
    // Review B1: a party figure over a period must not be read for the whole books, and a balance
    // on a past day must not be read as today's. Today in the worked day is 10-10-2026.
    for (question, from, to) in [
        (
            "akram ka is mahine ka commission kitna hai",
            "2026-10-01",
            "2026-10-10",
        ),
        (
            "akram ne aktubar mein kitna khareeda",
            "2026-10-01",
            "2026-10-10",
        ),
    ] {
        let reads = reads_with_filters(question);
        let dated: Vec<_> = reads
            .iter()
            .filter(|(tool, _, _)| tool == "analytics" || tool == "report")
            .collect();
        assert!(
            !dated.is_empty(),
            "{question}: nothing dated was read: {reads:?}"
        );
        for (tool, source, filters) in dated {
            assert_eq!(
                (filters["from"].as_str(), filters["to"].as_str()),
                (Some(from), Some(to)),
                "{question}: {tool} {source} was not read for the period: {filters}"
            );
        }
    }
    // A balance on a past day is not today's balance: the card is not read for it.
    let reads = reads_with_filters("akram ka 30 september ka bakaya kitna tha");
    assert!(
        !reads.iter().any(|(tool, _, _)| tool == "party_card"),
        "today's card was read for a past day: {reads:?}"
    );
}

#[test]
fn a_question_with_a_city_an_adda_or_a_scope_word_is_not_read_for_the_whole_shop() {
    // Review B2: a qualifier the plan does not carry must not be dropped from the read.
    for question in [
        "swat adda ki is mahine ki bikri kitni hai",
        "bagh ka is mahine ka commission kitna hai",
        "apna maal ki is mahine ki bikri kitni hai",
    ] {
        let tools = read_for(question);
        assert!(
            !tools.iter().any(|t| t == "analytics" || t == "tiles"),
            "{question}: the whole shop was read: {tools:?}"
        );
    }
}

#[test]
fn the_number_of_bikri_of_one_item_is_not_read_from_view_a() {
    // Review B3: view A has no count of بکری. The plan reads nothing for it; the older read for a
    // plural "bikriyan" then reads view B (the live run of 03-11-2026), which holds the count.
    let h = day();
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with("kele ki is mahine kitni bikriyan hui", None, &client);
    let first = &client.requests()[0];
    let views: Vec<String> = first["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .filter(|v| v["tool"] == "analytics")
        .filter_map(|v| v["source"].as_str().map(str::to_owned))
        .collect();
    assert!(
        !views.iter().any(|v| v == "A"),
        "view A was read for a count: {views:?}"
    );
}

#[test]
fn the_panel_is_told_the_app_understood_and_then_that_the_answer_is_being_written() {
    let h = day();
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with("akram ka bakaya kitna hai", None, &client);
    let ids: Vec<String> = h
        .steps
        .lock()
        .unwrap()
        .iter()
        .map(|s| s.id.clone())
        .collect();
    let understood = ids.iter().position(|i| i == "assistant-step-understood");
    let writing = ids.iter().position(|i| i == "assistant-step-writing");
    assert!(understood.is_some() && writing.is_some(), "{ids:?}");
    assert!(understood < writing, "{ids:?}");
    // A question the app is not sure of reads nothing first, so it never says it is writing.
    let h = day();
    let client = RecordedClient::new(vec![answer_reply("x")]);
    h.ask_with("akram ke kharche kam karne ke tareeke batao", None, &client);
    let ids: Vec<String> = h
        .steps
        .lock()
        .unwrap()
        .iter()
        .map(|s| s.id.clone())
        .collect();
    assert!(
        !ids.contains(&"assistant-step-writing".to_owned()),
        "{ids:?}"
    );
}

#[test]
fn a_follow_up_that_leans_on_the_earlier_question_is_left_to_the_model() {
    // Money review R-c. After "akram ka pichle mahine ka commission", "arshad ka commission?" means
    // last month too. The app reads only the new words, so alone it would read all time.
    let h = day();
    let first = RecordedClient::new(vec![answer_reply("کمیشن")]);
    let reply = h.ask_with(
        "akram ka pichle mahine ka commission kitna hai",
        None,
        &first,
    );
    let before = results_in(&first.requests()[0]);
    assert!(
        !before.is_empty(),
        "a question with its own period is read first"
    );

    // Control: asked alone, with no conversation before it, the same words ARE read first (so the
    // app understood them, and the next check is about the follow-up and not about the words).
    let alone = RecordedClient::new(vec![answer_reply("کمیشن")]);
    h.ask_with("arshad ka commission?", None, &alone);
    assert!(
        !results_in(&alone.requests()[0]).is_empty(),
        "asked alone, it is read first"
    );

    // In the conversation it is left to the model: nothing new is read before the call.
    let follow_up = RecordedClient::new(vec![answer_reply("کمیشن")]);
    h.ask_with(
        "arshad ka commission?",
        Some(&reply.conversation_id),
        &follow_up,
    );
    let request = &follow_up.requests()[0];
    assert_eq!(
        results_in(request).len(),
        before.len(),
        "only the results of the first question are in it: {:?}",
        results_in(request)
    );

    // A follow-up that names its own period is read first, as any question.
    let own_period = RecordedClient::new(vec![answer_reply("کمیشن")]);
    h.ask_with(
        "arshad ka pichle mahine ka commission?",
        Some(&reply.conversation_id),
        &own_period,
    );
    assert!(
        results_in(&own_period.requests()[0]).len() > before.len(),
        "a follow-up with its own period is read first"
    );
}
