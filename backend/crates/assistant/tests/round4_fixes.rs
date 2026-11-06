//! The fixes of hidden round 4 (live run 2, the lead's list):
//!
//! 1. "piyaz ki is saal ki sale aur agle 3 mahine ka andaza" ended in `checks_failed`. A mistyped
//!    reference that has one certain meaning is written right by the app (`n1` for `q.n1`, `rows.0`
//!    for the first row of a keyed list, as the model was shown it), the Owner's own number typed in
//!    digits is put back as `{{q.nK}}`, and a reference to a key that does not exist is told the
//!    keys of that list.
//! 2. (The Urdu-letter compound questions: the planner side, in `understand_reads.rs`.)
//! 3. "rahim ke marke pe kya likha hota he": `find` with the role marka finds the markas of an owner
//!    by his name, with the owner and never the mark text.
//! 4. The cash book button of the cash estimate opens on the months of the working.
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

use harness::{Harness, answer_of, figures, money};
use mandi_assistant::answer::AnswerSource;
use mandi_assistant::ask::AskOutcome;
use mandi_reports::reports::ReportId;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::masters::{MarkaInput, MasterInput};
use mandi_services::core::vouchers::{ReceiptInput, RemovedPaymentFields};
use mandi_services::{masters, vouchers};
use serde_json::{Value, json};

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_200, "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 7_000},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

fn tool_reply(calls: &[(&str, Value)]) -> Value {
    let calls: Vec<Value> = calls
        .iter()
        .enumerate()
        .map(|(i, (name, args))| {
            json!({"id": format!("call_{i}"), "type": "function",
                   "function": {"name": name, "arguments": args.to_string()}})
        })
        .collect();
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null, "tool_calls": calls}}],
        "usage": usage(),
    })
}

fn answer(kind: &str, text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": kind, "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

/// The tool results of a request as (id, parsed content).
fn results_in(request: &Value) -> Vec<(String, Value)> {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .filter_map(|v| Some((v["id"].as_str()?.to_owned(), v)))
        .collect()
}

/// The content of the last tool message of a request, error results included.
fn last_tool_message(request: &Value) -> Value {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|m| m["role"] == "tool")
        .and_then(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .unwrap_or(Value::Null)
}

/// The words of the `check_failed` messages the model was sent.
fn told(request: &Value) -> Vec<Value> {
    let mut out = Vec::new();
    for m in request["messages"].as_array().unwrap() {
        if m["role"] != "user" {
            continue;
        }
        if let Some(v) = m["content"]
            .as_str()
            .and_then(|t| serde_json::from_str::<Value>(t).ok())
        {
            out.extend(v["check_failed"].as_array().cloned().unwrap_or_default());
        }
    }
    out
}

/// Fixture B of the forecast tests with only banana sales: Rahim's banana to Akram on the 15th of
/// April to September, rates 100 to 600, today 10-10-26, with the app's pre-reads and the
/// local answers switched on as in the live app.
fn banana_year() -> Harness {
    let w = worked::months_world(&[
        (4, true, 100, 100),
        (5, true, 100, 200),
        (6, true, 100, 300),
        (7, true, 100, 400),
        (8, true, 100, 500),
        (9, true, 100, 600),
    ]);
    let h = Harness::over(w);
    h.runtime.set_local(true);
    h.runtime.set_read_first(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

// ---- 1: references the model mistyped ---------------------------------------------------------

#[test]
fn a_forecast_answer_with_a_place_for_the_one_row_the_number_in_digits_and_n1_is_repaired_by_the_app()
 {
    // H363: the planned reads are the period (r1), the item (r2), view A for it (r3, one row) and
    // the estimate (e1). The model cites the one row of view A by its place and writes the Owner's
    // "3" in digits. Both are certain: the app writes `{{r3.rows.i..}}` and `{{q.n1}}` and the answer
    // is shown from the model's first reply, with no rewrite call.
    let h = banana_year();
    let text = "{{r2.matches.0.name}} کی اس سال کی فروخت {{r3.rows.0.gross}} رہی۔ اگلے 3 مہینوں کا اندازہ \
                {{e1.same_pace.total}} ہے۔";
    let (reply, requests) = h.ask(
        "kela ki is saal ki sale aur agle 3 mahine ka andaza",
        None,
        vec![answer("estimate", text)],
    );
    assert_eq!(
        requests.len(),
        1,
        "one model call, no rewrite: told {:?}",
        requests.last().map(told)
    );
    assert_eq!(reply.outcome, AskOutcome::Estimate, "{reply:#?}");
    let first = &results_in(&requests[0]);
    let tools: Vec<&str> = first
        .iter()
        .map(|(_, v)| v["tool"].as_str().unwrap())
        .collect();
    assert_eq!(tools, ["period", "find", "analytics", "estimate"]);
    // The `n1` written for `q.n1` is repaired the same way.
    let h = banana_year();
    let text = "{{r2.matches.0.name}} کی اس سال کی فروخت {{r3.totals.gross}} رہی۔ اگلے {{n1}} مہینوں کا اندازہ \
                {{e1.same_pace.total}} ہے۔";
    let (reply, requests) = h.ask(
        "kela ki is saal ki sale aur agle 3 mahine ka andaza",
        None,
        vec![answer("estimate", text)],
    );
    assert_eq!(requests.len(), 1, "{reply:#?}");
    assert_eq!(reply.outcome, AskOutcome::Estimate, "{reply:#?}");
    h.scan_requests();
}

#[test]
fn a_place_in_a_keyed_list_is_the_row_the_model_was_shown_at_that_place_and_a_wrong_key_is_told_the_keys()
 {
    // The worked day: R20 has two rows (Akram a20 and Arshad a21). The model reads them as a list
    // of objects with a `key`, so `rows.0` is the first of them: the app writes its key and the
    // answer is shown with the one model call (Q924: `by_for.0.home_value`). A key that does not
    // exist is not guessed: the rewrite is told the keys, and a second wrong answer ends in
    // `checks_failed`.
    let h = Harness::day();
    let report = json!({
        "id": "R20", "from": null, "to": null, "as_of": null, "party": null, "item": null,
        "city": null, "adda": null, "collector": null, "partner": null, "trip": null,
        "garden": null, "bikri": null, "group_by": null, "where": null, "sort": null,
        "rows": null, "layout": null, "seller": null, "status": null, "voucher_type": null,
        "zamindar": null, "item_group": null
    });
    let by_place = answer("answer", "بقایا {{r1.rows.0.balance}} ہے۔");
    let (reply, requests) = h.ask(
        "kis kis ka udhar baqi hai",
        None,
        vec![tool_reply(&[("report", report.clone())]), by_place],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 2, "the report and the answer: no rewrite");
    let bad = answer("answer", "بقایا {{r1.rows.zzz.balance}} ہے۔");
    let (reply, requests) = h.ask(
        "kis kis ka udhar baqi hai",
        None,
        vec![tool_reply(&[("report", report)]), bad.clone(), bad],
    );
    assert_eq!(reply.outcome, AskOutcome::ChecksFailed, "{reply:#?}");
    assert_eq!(requests.len(), 3, "the report, the answer, the one rewrite");
    let said = told(&requests[2]);
    let miss = said
        .iter()
        .find(|f| f["rule"] == "reference")
        .unwrap_or_else(|| panic!("no reference failure: {said:?}"));
    assert_eq!(miss["at"], "r1.rows.zzz.balance");
    let keys: Vec<&str> = miss["keys"]
        .as_array()
        .unwrap_or_else(|| panic!("no keys: {miss}"))
        .iter()
        .map(|k| k.as_str().unwrap())
        .collect();
    assert!(
        keys.len() >= 2 && keys.iter().all(|k| k.starts_with('a')),
        "{keys:?}"
    );
}

// ---- 2: "next month" is one month ----------------------------------------------------------

#[test]
fn next_month_is_one_month_whether_ahead_is_sent_or_not_and_its_total_is_that_month() {
    // Lead ruling (live run 2, H369): "agle mahine" with no number is 1 month. `ahead: 1` is accepted,
    // and no `ahead` is 1 as well, so `same_pace.total` is the one month and a model that quotes it
    // for "next month" is right. The plural "agle mahino mein" stays 6, and a number the question
    // does not hold is still refused.
    let estimate = |ahead: Value| {
        json!({
            "measure": "sales", "item": null, "seller": null, "buyer": null, "garden": null,
            "trip": null, "head": null, "partner": null, "party": null, "ahead": ahead,
            "until": null, "what_if": null, "percent": null, "target": null
        })
    };
    let run = |question: &str, ahead: Value| {
        let h = banana_year();
        h.runtime.set_read_first(false);
        h.runtime.set_instant(false);
        h.runtime.set_local(false);
        let (_, requests) = h.ask(
            question,
            None,
            vec![
                tool_reply(&[("estimate", estimate(ahead))]),
                answer("estimate", "{{e1.same_pace.total}}"),
            ],
        );
        last_tool_message(requests.last().unwrap())
    };
    for ahead in [json!(1), Value::Null] {
        let one = run("kela agle mahine kitna bikega", ahead);
        assert_eq!(one["ahead_count"]["n"], 1, "{one}");
        // The total is the one month: the same figure as the month's own row.
        assert_eq!(
            one["same_pace"]["total"]["rs"], one["ahead"][0]["same_pace"]["rs"],
            "{one}"
        );
    }
    let six = run("kela agle mahino mein kitna bikega", Value::Null);
    assert_eq!(six["ahead_count"]["n"], 6, "{six}");
    let refused = run("kela agle mahino mein kitna bikega", json!(1));
    assert_eq!(refused["error"]["code"], "ERR_OUT_OF_RANGE", "{refused}");
}

// ---- 3: the markas of an owner ----------------------------------------------------------------

#[test]
fn the_markas_of_an_owner_are_found_by_his_name_with_the_owner_and_never_the_mark_text() {
    // H391: "rahim ke marke pe kya likha hota he". Rahim has the mark XQZ-77. The search keys of a
    // marka are its code and text, so the owner's name found nothing. Now the markas of the owner
    // come back, each with its code and its owner; the text typed by hand never leaves the computer.
    let h = Harness::day();
    let marka = masters::save(
        &h.world.env.ctx,
        &h.world.owner,
        &MasterInput::Marka(MarkaInput {
            id: None,
            version: None,
            code: String::new(),
            mark_text: "XQZ-77".into(),
            owner_account_id: Some(h.world.rahim),
        }),
    )
    .unwrap();
    let (reply, requests) = h.ask(
        "rahim ke marke pe kya likha hota he",
        None,
        vec![
            tool_reply(&[("find", json!({"text": "rahim", "role": "marka"}))]),
            answer(
                "answer",
                "{{r1.matches.0.owner}} کے مارکے {{r1.count}} ہیں۔",
            ),
        ],
    );
    assert_eq!(
        reply.outcome,
        AskOutcome::Answered,
        "told {:?}, last tool {}",
        requests.last().map(told),
        requests.last().map(last_tool_message).unwrap_or_default()
    );
    let found = results_in(&requests[1])
        .into_iter()
        .find(|(id, _)| id == "r1")
        .unwrap()
        .1;
    let matches = found["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1, "{found}");
    assert_eq!(matches[0]["key"], format!("mk{}", marka.id()));
    assert_eq!(matches[0]["owner"]["en"], "Rahim");
    assert!(
        !found.to_string().contains("XQZ-77"),
        "the mark text stays home: {found}"
    );
    // Another seller's name finds nothing, and the answer of "no such" is a list with count 0.
    let (_, requests) = h.ask(
        "karim ke marke pe kya likha hota he",
        None,
        vec![
            tool_reply(&[("find", json!({"text": "karim", "role": "marka"}))]),
            answer("answer", "x"),
        ],
    );
    let none = results_in(&requests[1])
        .into_iter()
        .find(|(id, _)| id == "r1")
        .unwrap()
        .1;
    assert_eq!(none["matches"].as_array().unwrap().len(), 0, "{none}");
    h.scan_requests();
}

// ---- 4: the cash book button ------------------------------------------------------------------

#[test]
fn the_cash_book_button_of_the_cash_estimate_opens_on_the_months_of_the_working() {
    // Akram pays 10,000 ... 15,000 on the 20th of April to September, so the cash changes by that
    // each month and the pace uses the last 3 months, July to September. The button opens R12 from
    // 01-07-2026 to 30-09-2026: the changes of the working are there.
    let w = worked::months_world(&[(4, true, 100, 100)]);
    for (month, amount) in [
        (4, "10,000"),
        (5, "11,000"),
        (6, "12,000"),
        (7, "13,000"),
        (8, "14,000"),
        (9, "15,000"),
    ] {
        common::set_today(&w.env, 2026, month, 20);
        vouchers::save_receipt(
            &w.env.ctx,
            &w.owner,
            &ReceiptInput {
                date: format!("20-{month:02}-2026"),
                account_id: w.akram,
                person_key: None,
                amount: amount.into(),
                removed: RemovedPaymentFields::default(),
                collector_staff_id: None,
                narration: String::new(),
            },
        )
        .unwrap();
    }
    common::set_today(&w.env, 2026, 10, 10);
    let h = Harness::over(w);
    let input = json!({
        "measure": "cash_in_hand", "item": null, "seller": null, "buyer": null, "garden": null,
        "trip": null, "head": null, "partner": null, "party": null, "ahead": 3, "until": null,
        "what_if": null, "percent": null, "target": null
    });
    let (reply, _) = h.ask(
        "cash in hand now and in 3 months",
        None,
        vec![
            tool_reply(&[("estimate", input)]),
            answer("estimate", "{{e1.same_pace.total}}"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Estimate, "{reply:#?}");
    let sources = &answer_of(&reply).sources;
    let Some(AnswerSource::Report { id, filters, .. }) = sources.first() else {
        panic!("no report source: {sources:#?}");
    };
    assert_eq!(*id, ReportId::R12);
    assert_eq!(
        (filters.from.as_str(), filters.to.as_str()),
        ("2026-07-01", "2026-09-30")
    );
}

// ---- 2 (end to end): Urdu-letter compound questions have their reads before the first call --------

#[test]
fn the_urdu_letter_compound_questions_are_read_before_the_first_call() {
    // H364 and H369 ended in `time_limit` at 25 s: with Urdu letters no read was planned (the Urdu
    // year-end phrase and the suffix "گا" were unknown words), so the model had to find, read and
    // estimate itself. The planner is backend2's (`understand_reads.rs` holds its table); this holds
    // the whole path: the first request already has the four reads.
    for question in [
        "کیلا اس سال کتنا بکا اور اگلے مہینے کتنا بکے گا",
        "کیلا کا اس سال کا کمیشن کتنا ہے اور سال کے آخر تک کتنا ہو گا",
    ] {
        let h = banana_year();
        let (_, requests) = h.ask(question, None, vec![answer("estimate", "x")]);
        let tools: Vec<String> = results_in(&requests[0])
            .into_iter()
            .map(|(_, v)| v["tool"].as_str().unwrap_or("").to_owned())
            .collect();
        assert_eq!(
            tools,
            ["period", "find", "analytics", "estimate"],
            "{question}"
        );
    }
}

// ---- 5: an exact name is the one match --------------------------------------------------------

/// The matches of one `find`, as the model reads them.
fn find_matches(h: &Harness, text: &str, role: &str) -> Vec<Value> {
    let (_, requests) = h.ask(
        &format!("{text} kya hai"),
        None,
        vec![
            tool_reply(&[("find", json!({"text": text, "role": role}))]),
            answer("answer", "x"),
        ],
    );
    let found = results_in(&requests[1])
        .into_iter()
        .find(|(id, _)| id == "r1")
        .unwrap_or_else(|| panic!("no find result for {text}"))
        .1;
    found["matches"].as_array().cloned().unwrap_or_default()
}

fn names_en(matches: &[Value]) -> Vec<String> {
    matches
        .iter()
        .map(|m| m["name"]["en"].as_str().unwrap_or("").to_owned())
        .collect()
}

#[test]
fn a_name_typed_exactly_is_the_one_match_and_close_spellings_stay_when_nothing_is_exact() {
    // "tomato" used to give Tomato and Potato (a close spelling after the exact hit), so the model
    // asked which one. An exact name, in either script and either case, is the only match; "tomatto"
    // (a typo) is exact for nothing, so its close spelling Tomato stays. (A Roman Urdu word such as
    // "tmatar" is not a spelling of the English name: the pickers never convert, so it finds nothing
    // here; the planner's own word table knows it.)
    let h = Harness::day();
    let (env, owner) = (&h.world.env, &h.world.owner);
    world::add_item(env, owner, "ٹماٹر", "Tomato", "vegetable", "unit");
    world::add_item(env, owner, "آلو", "Potato", "vegetable", "unit");
    assert_eq!(names_en(&find_matches(&h, "tomato", "item")), ["Tomato"]);
    assert_eq!(names_en(&find_matches(&h, "Tomato", "item")), ["Tomato"]);
    assert_eq!(names_en(&find_matches(&h, "potato", "item")), ["Potato"]);
    assert_eq!(names_en(&find_matches(&h, "ٹماٹر", "item")), ["Tomato"]);
    let close = find_matches(&h, "tomatto", "item");
    assert!(
        names_en(&close).contains(&"Tomato".to_owned()),
        "close spellings stay: {close:?}"
    );
}

#[test]
fn two_accounts_with_the_same_exact_name_both_show_with_or_without_a_tag() {
    // Two buyers named اسلم / Aslam, one with the tag BA: the Owner has to say which. The exact
    // rule keeps ALL the exact ones, and a third, differently named buyer (a close spelling) goes.
    let h = Harness::day();
    let (env, owner) = (&h.world.env, &h.world.owner);
    world::add_account(env, owner, AccountType::Buyer, "اسلم", "Aslam", true, "");
    world::add_account_tagged(
        env,
        owner,
        AccountType::Buyer,
        ("اسلم", "Aslam"),
        (true, ""),
        "BA",
    );
    world::add_account(env, owner, AccountType::Buyer, "اسلام", "Islam", true, "");
    let both = find_matches(&h, "aslam", "buyer");
    assert_eq!(names_en(&both), ["Aslam", "Aslam"], "{both:?}");
    let both = find_matches(&h, "اسلم", "buyer");
    assert_eq!(both.len(), 2, "{both:?}");
}

// ---- 6: the scope guard (live run 2, CRITICAL) -------------------------------------------------

/// A view A call, every field of its schema, with these fields set.
fn view_a(set: Value) -> Value {
    let mut base = json!({
        "view": "A", "from": null, "to": null, "group_by": null, "item": null, "seller": null,
        "buyer": null, "city": null, "adda": null, "garden": null, "trip": null, "goods": null,
        "partner": null, "side": null, "compare": null, "sort": null, "rows": null,
        "item_group": null
    });
    for (k, v) in set.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

/// The books of the guard tests: banana in April and June, apple in May (so the item's figure and
/// the whole books' differ), today 10-10-26, with the app's pre-reads on.
fn banana_and_apple() -> Harness {
    let w = worked::months_world(&[
        (4, true, 100, 100),
        (5, false, 100, 200),
        (6, true, 100, 300),
    ]);
    let h = Harness::over(w);
    h.runtime.set_read_first(true);
    h
}

#[test]
fn an_answer_that_cites_a_read_without_the_period_and_the_item_of_the_question_is_not_shown() {
    // Q345: "kela ka commission is saal" was answered from a view A read with no item and no period
    // (all the books, 9,600) under the words "this year". The app read view A for the banana and
    // this year before the first call (r3); the model's own read (r4) has no scope. The answer is
    // refused, the model is told to cite r3, and the figure shown is the banana's 6,400.
    let h = banana_and_apple();
    let wrong = answer("answer", "اس سال کا کمیشن {{r4.totals.total_comm}} رہا۔");
    let right = answer("answer", "اس سال کا کمیشن {{r3.totals.total_comm}} رہا۔");
    let (reply, requests) = h.ask(
        "kela ka commission is saal kitna bana",
        None,
        vec![
            tool_reply(&[("analytics", view_a(json!({})))]),
            wrong,
            right,
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 3, "the read, the answer, the one rewrite");
    let said = told(&requests[2]);
    let scope = said
        .iter()
        .find(|f| f["rule"] == "scope")
        .unwrap_or_else(|| panic!("no scope failure: {said:?}"));
    assert_eq!(scope["at"], "r4");
    assert_eq!(scope["cite_instead"], json!(["r3"]));
    let figs = figures(answer_of(&reply));
    assert_eq!(money(figs[0].0), 6_400);
    h.scan_requests();
}

#[test]
fn an_answer_that_cites_a_read_with_the_scope_of_the_question_passes_at_once() {
    // The same question, the model reads view A itself with the item and this year's dates: the app
    // gives back the read it made ahead (r3), the scope is right, one answer call.
    let h = banana_and_apple();
    let own = view_a(json!({"from": "2026-01-01", "to": "2026-10-10", "item": "i1"}));
    let (reply, requests) = h.ask(
        "kela ka commission is saal kitna bana",
        None,
        vec![
            tool_reply(&[("analytics", own)]),
            answer("answer", "اس سال کا کمیشن {{r3.totals.total_comm}} رہا۔"),
        ],
    );
    assert_eq!(
        reply.outcome,
        AskOutcome::Answered,
        "told {:?} / last tool {}",
        requests.last().map(told),
        requests.last().map(last_tool_message).unwrap_or_default()
    );
    assert_eq!(requests.len(), 2, "told {:?}", requests.last().map(told));
    assert_eq!(money(figures(answer_of(&reply))[0].0), 6_400);
}

#[test]
fn an_answer_that_cites_a_read_without_the_group_of_the_question_is_refused_and_a_second_wrong_one_ends_in_checks_failed()
 {
    // Q1663: "vegetables last week" read with no group gave the whole shop's sale. Here "phal ki is
    // saal ki sale": the planned view A has the group fruit; a read with the dates but no group is
    // refused, and when the rewrite cites it again nothing is shown.
    let h = banana_and_apple();
    let dates_only = view_a(json!({"from": "2026-01-01", "to": "2026-10-10"}));
    let wrong = answer("answer", "اس سال کی فروخت {{r3.totals.gross}} رہی۔");
    let (reply, requests) = h.ask(
        "phal ki is saal ki sale kitni hui",
        None,
        vec![
            tool_reply(&[("analytics", dates_only)]),
            wrong.clone(),
            wrong,
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::ChecksFailed, "{reply:#?}");
    assert_eq!(requests.len(), 3);
    let said = told(&requests[2]);
    assert!(
        said.iter().any(|f| f["rule"] == "scope" && f["at"] == "r3"),
        "{said:?}"
    );
}

// ---- 7: a guide is not the figure (G3), the two dues tiles (H1), a misspelt month word (G4) ----

/// A report call, every field of its schema, with these fields set.
fn report_call(set: Value) -> Value {
    let mut base = json!({
        "id": "R20", "from": null, "to": null, "as_of": null, "party": null, "item": null,
        "city": null, "adda": null, "collector": null, "partner": null, "trip": null,
        "garden": null, "bikri": null, "group_by": null, "where": null, "sort": null,
        "rows": null, "layout": null, "seller": null, "status": null, "voucher_type": null,
        "zamindar": null, "item_group": null
    });
    for (k, v) in set.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

#[test]
fn an_answer_of_the_guide_alone_is_refused_when_a_figure_was_asked_and_read() {
    // Q615: "total open advances" read R26 and answered with the advance screen's guide. The guide
    // is the answer to "how do I", not to "how much": the model is told to cite the report.
    let h = Harness::day();
    let calls = tool_reply(&[
        ("app_help", json!({"topic": "advance"})),
        ("report", report_call(json!({"id": "R26"}))),
    ]);
    let guide = answer("answer", "{{r1.summary}}");
    let figure = answer("answer", "کھلی پیشگی {{r2.totals.open}} ہے۔");
    let (reply, requests) = h.ask("total open advances", None, vec![calls, guide, figure]);
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 3, "the reads, the guide, the one rewrite");
    let said = told(&requests[2]);
    let refused = said
        .iter()
        .find(|f| f["rule"] == "guide_only")
        .unwrap_or_else(|| panic!("no guide_only failure: {said:?}"));
    assert_eq!(refused["at"], "r1");
    assert_eq!(refused["cite_instead"], json!(["r2"]));
    assert!(!figures(answer_of(&reply)).is_empty());
}

#[test]
fn the_guide_is_the_answer_to_a_how_to_question_even_when_a_report_was_read() {
    let h = Harness::day();
    let calls = tool_reply(&[
        ("app_help", json!({"topic": "advance"})),
        ("report", report_call(json!({"id": "R26"}))),
    ]);
    let (reply, requests) = h.ask(
        "peshgi kaise likhte hain",
        None,
        vec![calls, answer("answer", "{{r1.summary}}")],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 2, "told {:?}", requests.last().map(told));
}

#[test]
fn the_total_dues_tile_says_what_it_holds_and_that_the_collectors_total_is_buyers_dues() {
    // Q1052 and Q1054 ("total dues by collector") quoted 21,751,319, the tile total_dues, for the
    // collectors' total 20,324,367 (R20/R22). The tile is the buyers' dues plus the sellers who owe
    // the firm plus the dead amount (here 20,324,367 + 0 + 1,426,952): it says so in its result.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "total dues by collector",
        None,
        vec![tool_reply(&[("tiles", json!({}))]), answer("answer", "x")],
    );
    let tiles = results_in(&requests[1])
        .into_iter()
        .find(|(id, _)| id == "r1")
        .unwrap()
        .1;
    let total = &tiles["total_dues"];
    assert_eq!(
        total["holds"],
        json!(["buyers_dues", "sellers_dues", "dead_amount"])
    );
    assert!(total["note"].as_str().unwrap().contains("buyers_dues"));
    // The worked day has no seller owing the firm and no dead person: the parts are the buyers'.
    assert_eq!(total["amount"]["rs"], tiles["buyers_dues"]["amount"]["rs"]);
}

#[test]
fn a_misspelt_month_word_keeps_the_number_a_number_of_months() {
    // Q696: "how much at this pace after 2 monhts" lost the unit, so ahead 2 was refused.
    use mandi_services::domain::figures::{NumberUnit, question_numbers};
    for question in [
        "what does Faisal owe now and how much at this pace after 2 monhts",
        "next 3 mnths ka andaza",
        "agle 3 mahene ka andaza",
    ] {
        let numbers = question_numbers(question);
        assert_eq!(numbers.len(), 1, "{question}");
        assert_eq!(numbers[0].unit, NumberUnit::Months, "{question}");
    }
}

// ---- 8: every estimated figure carries the working of its method (Q708) -----------------------

#[test]
fn the_figure_of_one_month_ahead_carries_the_working_of_its_method_and_the_real_cash_none() {
    // Q708: "how much cash do we have now and how much will we have next month". The sentence has
    // the real cash (the tile, checked) and the estimate for next month (`e1.ahead.ah1.same_pace`).
    // The estimate figure had no working, so it read like a figure of the books. Now it carries the
    // working of its method; the real cash carries none and keeps its check mark.
    let w = worked::months_world(&[(4, true, 100, 100)]);
    for (month, amount) in [(4, "10,000"), (5, "11,000"), (6, "12,000"), (7, "13,000")] {
        common::set_today(&w.env, 2026, month, 20);
        vouchers::save_receipt(
            &w.env.ctx,
            &w.owner,
            &ReceiptInput {
                date: format!("20-{month:02}-2026"),
                account_id: w.akram,
                person_key: None,
                amount: amount.into(),
                removed: RemovedPaymentFields::default(),
                collector_staff_id: None,
                narration: String::new(),
            },
        )
        .unwrap();
    }
    common::set_today(&w.env, 2026, 10, 10);
    let h = Harness::over(w);
    let estimate = json!({
        "measure": "cash_in_hand", "item": null, "seller": null, "buyer": null, "garden": null,
        "trip": null, "head": null, "partner": null, "party": null, "ahead": 1, "until": null,
        "what_if": null, "percent": null, "target": null
    });
    let text = "ابھی روکڑ {{r1.cash_in_hand.amount}} ہے، اگلے مہینے اندازاً {{e1.ahead.ah1.same_pace}} ہو گا۔";
    let (reply, _) = h.ask(
        "how much cash do we have now and how much will we have next month",
        None,
        vec![
            tool_reply(&[("tiles", json!({})), ("estimate", estimate)]),
            answer("estimate", text),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Estimate, "{reply:#?}");
    let parts: Vec<_> = answer_of(&reply)
        .parts
        .iter()
        .filter_map(|p| match p {
            mandi_assistant::answer::AnswerPart::Figure {
                checked, working, ..
            } => Some((*checked, working.is_some())),
            _ => None,
        })
        .collect();
    // The real cash: checked, no working. The estimate: not checked, with its method's working.
    assert_eq!(parts, [(true, false), (false, true)], "{parts:?}");
    assert!(answer_of(&reply).estimate.is_some());
}
