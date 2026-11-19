//! F1 (live run 2, 36 questions): a period with nothing in it is a real 0 the model can cite, never
//! `not_found`. The tools give totals of 0 for a valid party, a valid item and a valid partner when
//! the period holds nothing of theirs; the answer cites them and is shown as 0. And a partner the
//! model asked for as a buyer is found as a partner (the Owner is asked to confirm him) instead of
//! "no such buyer".
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
    let calls: Vec<Value> = calls
        .iter()
        .enumerate()
        .map(|(i, (name, args))| {
            json!({"id": format!("call_{i}"), "type": "function",
                   "function": {"name": name, "arguments": args.to_string()}})
        })
        .collect();
    json!({"model": "openai/gpt-6-luna", "choices": [{"finish_reason": "tool_calls", "message": {
        "role": "assistant", "content": null, "tool_calls": calls}}], "usage": usage()})
}

fn answer(text: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

fn quiet_period() -> Value {
    // After the opening of the worked day and before its first sale: nothing was posted.
    json!({"kind": "range", "from": "2026-09-05", "to": "2026-09-06", "day": null,
           "month": null, "n": null})
}

fn view(view: &str, extra: Value) -> Value {
    let mut v = json!({"view": view, "from": "r1.from", "to": "r1.to", "item": null, "side": null,
        "adda": null, "buyer": null, "city": null, "compare": null, "garden": null, "goods": null,
        "grade": null, "group_by": null, "item_group": null, "partner": null, "rows": null,
        "seller": null, "sort": null, "trip": null});
    if let (Some(m), Some(e)) = (v.as_object_mut(), extra.as_object()) {
        for (k, x) in e {
            m.insert(k.clone(), x.clone());
        }
    }
    v
}

fn report(id: &str, extra: Value) -> Value {
    let mut v = json!({"id": id, "from": "r1.from", "to": "r1.to", "item": null, "party": null,
        "as_of": null, "city": null, "adda": null, "collector": null, "partner": null,
        "trip": null, "garden": null, "bikri": null, "group_by": null, "where": null,
        "sort": null, "rows": null, "layout": null, "seller": null, "status": null,
        "voucher_type": null, "zamindar": null, "item_group": null, "grade": null});
    if let (Some(m), Some(e)) = (v.as_object_mut(), extra.as_object()) {
        for (k, x) in e {
            m.insert(k.clone(), x.clone());
        }
    }
    v
}

#[test]
fn a_valid_item_a_valid_party_and_a_valid_partner_with_nothing_in_the_period_are_a_cited_zero() {
    for (what, calls, text) in [
        // The item: nothing of it was sold in the period.
        (
            "item",
            vec![
                ("period", quiet_period()),
                ("find", json!({"role": "item", "text": "banana"})),
                ("analytics", view("A", json!({"item": "r2.matches.0"}))),
            ],
            "{{r3.totals.gross}} کی بکری رہی۔",
        ),
        // The party: nothing was bought by him (his ledger of the period, and his view C).
        (
            "party",
            vec![
                ("period", quiet_period()),
                ("find", json!({"role": "buyer", "text": "akram"})),
                ("analytics", view("C", json!({"buyer": "r2.matches.0"}))),
            ],
            "{{r3.totals.purchase_value}} کی خریداری رہی۔",
        ),
        // The partner: he took nothing for himself in the period.
        (
            "partner",
            vec![
                ("period", quiet_period()),
                ("find", json!({"role": "partner", "text": "alif"})),
                ("report", report("R31", json!({"partner": "r2.matches.0"}))),
            ],
            "{{r3.totals.amount}} کا لین دین رہا۔",
        ),
    ] {
        let h = Harness::day();
        h.runtime.set_local(false);
        h.runtime.set_read_first(false);
        h.runtime.set_instant(false);
        let (reply, requests) = h.ask("kitna hua", None, vec![tool_reply(&calls), answer(text)]);
        assert_eq!(reply.outcome, AskOutcome::Answered, "{what}: {reply:#?}");
        assert_eq!(requests.len(), 2, "{what}: no rewrite");
        let shown: Vec<i64> = figures(answer_of(&reply))
            .iter()
            .map(|(cell, _)| money(cell))
            .collect();
        assert_eq!(shown, [0], "{what}: a real zero");
    }
}

#[test]
fn a_partner_asked_for_as_a_buyer_is_found_as_a_partner_and_not_as_no_such_buyer() {
    // Q3827: "Alif ne kal zaati kharche ke liye kitna liya": the model found him as a buyer. No buyer
    // has that name, so the search is widened to the partners: the find says `widened: partner` and
    // gives him, and the question goes on (before, the empty find ended it as "no such buyer" while
    // the partner had taken 0).
    let h = Harness::day();
    h.runtime.set_local(false);
    h.runtime.set_read_first(false);
    h.runtime.set_instant(false);
    let (reply, requests) = h.ask(
        "Alif ne kal zaati kharche ke liye kitna liya",
        None,
        vec![
            tool_reply(&[("find", json!({"role": "buyer", "text": "alif"}))]),
            answer("{{r1.matches.0.name}} نے کچھ نہیں لیا۔"),
        ],
    );
    assert_eq!(
        requests.len(),
        2,
        "the model went on after the find: {reply:#?}"
    );
    assert_ne!(reply.outcome, AskOutcome::NotFound, "{reply:#?}");
    let find = requests[1]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .find(|v| v["tool"] == "find")
        .expect("the find result");
    assert_eq!(find["widened"], "partner", "{find}");
    assert_eq!(find["matches"].as_array().map(Vec::len), Some(1), "{find}");
}

// ---- C7, C5, F5 of live run 2 -------------------------------------------------------------------

/// The result a tool gave in the last request, by its result id.
fn result_in(request: &Value, id: &str) -> Value {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .find(|v| v["id"] == id)
        .unwrap_or_else(|| panic!("no result {id}"))
}

/// The `find` result the model was given in a request (the period may be read before it).
fn find_in(request: &Value) -> Value {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .find(|v| v["tool"] == "find")
        .expect("the find result")
}

fn plain_world() -> Harness {
    let h = Harness::day();
    h.runtime.set_local(false);
    h.runtime.set_read_first(false);
    h.runtime.set_instant(false);
    h
}

#[test]
fn the_debit_and_the_credit_total_of_a_day_book_are_not_added_together() {
    // C7, Q1024 "total of yesterday's day book": the model added totals.debit and totals.credit of
    // R11 (722,194 for a book of 361,097). The two are one amount seen from two sides.
    let h = plain_world();
    let period = json!({"kind": "range", "from": "2026-10-10", "to": "2026-10-10", "day": null,
                        "month": null, "n": null});
    let sum = |a: &str, b: &str| json!({"op": "sum", "inputs": [a, b]});
    let (reply, requests) = h.ask(
        "roznamche ka total kitna hai",
        None,
        vec![
            tool_reply(&[("period", period), ("report", report("R11", json!({})))]),
            tool_reply(&[("calculate", sum("r2.totals.debit", "r2.totals.credit"))]),
            answer("{{r2.totals.debit}} کا روزنامچہ ہے۔"),
        ],
    );
    // The call was refused, with the reason, and no figure was made from it.
    let refusal = requests[2]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
        .find(|v| v["error"]["code"] == "ERR_OUT_OF_RANGE")
        .unwrap_or_else(|| {
            panic!(
                "the calculate call was refused: {}",
                requests[2]["messages"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|m| m["role"] == "tool")
                    .map(|m| m["content"]
                        .as_str()
                        .unwrap_or("")
                        .chars()
                        .take(300)
                        .collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        });
    assert!(
        refusal["error"]["detail"]
            .as_str()
            .unwrap()
            .contains("same amount seen from two sides"),
        "{refusal}"
    );
    // The one total, cited on its own, is the book's figure and is shown.
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let book = result_in(&requests[2], "r2");
    let debit = book["totals"]["debit"]["rs"].as_i64().unwrap();
    let shown: Vec<i64> = figures(answer_of(&reply))
        .iter()
        .map(|(cell, _)| money(cell))
        .collect();
    assert_eq!(shown, [debit]);
}

#[test]
fn a_partner_name_typed_as_a_buyer_is_the_partner_and_not_a_close_spelling_of_a_buyer() {
    // C5, Q3849: the model's find{role:buyer, text:"Alif"} fuzzily matched the buyer Kashif Ali. The
    // text IS the partner's name, so the close spelling is left out and the partner is found, and
    // the result says so.
    let h = plain_world();
    world::add_account(
        &h.world.env,
        &h.world.owner,
        mandi_services::core::accounts::AccountType::Buyer,
        "پارٹنر ای",
        "Partner E",
        true,
        "",
    );
    let (_, requests) = h.ask(
        "Partner A ka liya hua pichle saal",
        None,
        vec![
            tool_reply(&[("find", json!({"role": "buyer", "text": "Partner A"}))]),
            answer("{{r1.matches.0.name}} کا ذکر ہے۔"),
        ],
    );
    let find = find_in(&requests[1]);
    assert_eq!(find["widened"], "partner", "{find}");
    let matches = find["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1, "{find}");
    assert!(
        matches[0]["name"]["en"]
            .as_str()
            .unwrap()
            .starts_with("Partner A"),
        "{find}"
    );
}

#[test]
fn one_clear_account_is_the_answer_and_a_close_spelling_beside_it_is_not_asked_about() {
    // F5: "kareem" has the phonetic key of Karim ("krm"); Kareen is only a close spelling. The one
    // clear account is returned, so the Owner is not asked which one. Two equally clear accounts
    // (Karim and Karima, both "krm") still ask.
    let h = plain_world();
    let add = |ur: &str, en: &str| {
        world::add_account(
            &h.world.env,
            &h.world.owner,
            mandi_services::core::accounts::AccountType::Buyer,
            ur,
            en,
            true,
            "",
        );
    };
    add("کرین", "Kareen");
    let find = |text: &str| {
        let (_, requests) = h.ask(
            "kareem ka khata",
            None,
            vec![
                tool_reply(&[("find", json!({"role": "any_party", "text": text}))]),
                answer(""),
            ],
        );
        find_in(&requests[1])
    };
    let names = |v: &Value| -> Vec<String> {
        v["matches"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["name"]["en"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(names(&find("kareem")), ["Karim"], "one clear account");
    // Equally clear: Karima has the same key, so both are listed.
    add("کریمہ", "Karima");
    let both = names(&find("kareem"));
    assert!(
        both.contains(&"Karim".to_owned()) && both.contains(&"Karima".to_owned()),
        "{both:?}"
    );
}
