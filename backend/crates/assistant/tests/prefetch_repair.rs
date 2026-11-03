//! Fewer model calls for a named party (plan 52 P52-S1, the live run of 03-11-2026).
//!
//! 1. Prefetch: the model never sends `find` and `party_card` in one reply. When its reply holds
//!    only finds and a find matched exactly one buyer or seller, the app reads the card itself
//!    (and R13 when the question names a ledger) under the next normal numbers (find r1, card r2, ledger r3), so the next call can
//!    answer. The model's `r` numbers never move.
//! 2. Repair: a name typed in the words that the results hold exactly is put back as a reference
//!    by the app, instead of a rewrite call; every rule is checked again, and a name two records
//!    share, or any other fault, still ends in the rewrite.
//!
//! The worked day: اکرم ٹول ٹیکس (Akram Toll Tax) owes 2,880 بنام after Events 1 to 5.
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

use harness::{Harness, answer_of, balance, figures};
use mandi_assistant::answer::AnswerPart;
use mandi_assistant::ask::AskOutcome;
use mandi_services::core::accounts::BalanceSide;
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

fn answer_with(kind: &str, text: &str, next: &[&str], show: Value) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": kind, "text": text, "show": show, "next": next,
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

fn final_reply(text: &str) -> Value {
    answer_with("answer", text, &[], json!([]))
}

fn find(text: &str, role: &str) -> (&'static str, Value) {
    ("find", json!({"text": text, "role": role}))
}

/// The tool messages of a request, as (tool_call_id, parsed content).
fn tool_messages(body: &Value) -> Vec<(String, Value)> {
    body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .map(|m| {
            (
                m["tool_call_id"].as_str().unwrap_or("").to_owned(),
                serde_json::from_str(m["content"].as_str().unwrap_or("{}")).unwrap_or(Value::Null),
            )
        })
        .collect()
}

fn step_ids(h: &Harness) -> Vec<String> {
    h.steps
        .lock()
        .unwrap()
        .iter()
        .map(|s| s.id.clone())
        .collect()
}

fn log_row(h: &Harness) -> (Vec<Value>, Vec<Value>) {
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    let calls: Vec<Value> = serde_json::from_str(rows[0].calls_json.as_deref().unwrap()).unwrap();
    let tools: Vec<Value> = serde_json::from_str(&rows[0].tool_calls_json).unwrap();
    (calls, tools)
}

/// The rules the app named in its `check_failed` message to the model.
fn failed_rules(body: &Value) -> Vec<String> {
    let mut rules = Vec::new();
    for m in body["messages"].as_array().unwrap() {
        if m["role"] != "user" {
            continue;
        }
        let Some(v) = m["content"]
            .as_str()
            .and_then(|t| serde_json::from_str::<Value>(t).ok())
        else {
            continue;
        };
        for f in v["check_failed"].as_array().into_iter().flatten() {
            if let Some(r) = f["rule"].as_str() {
                rules.push(r.to_owned());
            }
        }
    }
    rules
}

// ---- prefetch ---------------------------------------------------------------------------------

#[test]
fn a_lone_find_leads_to_the_card_at_once_so_the_question_takes_two_model_calls() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "how much does akram owe",
        None,
        vec![
            tool_reply(&[find("akram", "buyer")]),
            final_reply("{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 2, "find, then the answer");
    let figs = figures(answer_of(&reply));
    assert_eq!(balance(figs[0].0), (2_880, BalanceSide::Dr));
    assert!(figs[0].1, "checked: card against his ledger");
    // The second call has the find (r1) and the card the app read (r2), as a call of its own with
    // a tool message: every tool call has its tool message.
    let tools = tool_messages(&requests[1]);
    let ids: Vec<&str> = tools.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, ["call_0", "pre_p1"]);
    assert_eq!(tools[0].1["id"], "r1");
    assert_eq!(tools[1].1["id"], "r2");
    let calls: usize = requests[1]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["tool_calls"].as_array())
        .map(Vec::len)
        .sum();
    assert_eq!(calls, tools.len());
    // The panel's step lines: finding, the card, then the check.
    assert_eq!(
        step_ids(&h),
        [
            "assistant-step-finding",
            "assistant-step-reading-card",
            "assistant-step-checking"
        ]
    );
    // The log: the card is marked as the app's own read, in the tool list and the call's tools.
    let (calls, tools) = log_row(&h);
    assert_eq!(tools[1]["id"], "r2");
    assert_eq!(tools[1]["prefetch"], true);
    assert_eq!(calls[0]["tools"][1]["name"], "party_card");
    assert_eq!(calls[0]["tools"][1]["prefetch"], true);
    assert!(calls[0]["tools"][0].get("prefetch").is_none());
    h.scan_requests();
}

#[test]
fn a_question_that_names_a_ledger_reads_it_too_and_the_models_own_read_takes_the_next_number() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "akram ka khata dikhao",
        None,
        vec![
            tool_reply(&[find("akram", "buyer")]),
            // The model asks for one more read of its own: it is r2, as it always was.
            tool_reply(&[("tiles", json!({}))]),
            answer_with(
                "answer",
                "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔\n• ان سے {{r3.rows.je2.credit}} وصول ہوئے۔",
                &[],
                json!([{"result": "r3", "as": "table", "rows": 7}]),
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 3);
    let tools = tool_messages(&requests[1]);
    let ids: Vec<String> = tools
        .iter()
        .map(|(_, v)| v["id"].as_str().unwrap_or("").to_owned())
        .collect();
    assert_eq!(ids, ["r1", "r2", "r3"], "the card, then his ledger R13");
    assert_eq!(tools[2].1["source"], "R13", "his ledger");
    let tools = tool_messages(&requests[2]);
    let last = tools.last().unwrap();
    assert_eq!(
        last.1["id"], "r4",
        "the model's own read is the next r number after the app's"
    );
    let answer = answer_of(&reply);
    assert_eq!(answer.tables.len(), 1);
    let figs = figures(answer);
    assert_eq!(balance(figs[0].0), (2_880, BalanceSide::Dr));
    assert_eq!(harness::money(figs[1].0), 10_000);
    h.scan_requests();
}

#[test]
fn several_matches_or_other_calls_or_other_roles_read_nothing_ahead() {
    // "a" is in Akram and Arshad: several buyers, nothing is read for the Owner's pick.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "a ka khata dikhao",
        None,
        vec![
            tool_reply(&[find("a", "buyer")]),
            json!({
                "model": "openai/gpt-6-luna",
                "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
                    "kind": "which_one", "text": "", "show": [], "next": [],
                    "choose": "r1", "entry": null, "nearest": null
                }).to_string()}}],
                "usage": usage(),
            }),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::WhichOne, "{reply:#?}");
    assert_eq!(tool_messages(&requests[1]).len(), 1, "only the find");
    let (_, tools) = log_row(&h);
    assert_eq!(tools.len(), 1);

    // A find with another call in the same reply: the model is already chaining; the app adds none.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "ikram ka hisab",
        None,
        vec![
            tool_reply(&[
                find("akram", "buyer"),
                ("party_card", json!({"party": "r1.matches.0"})),
            ]),
            final_reply("{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔"),
        ],
    );
    let ids: Vec<String> = tool_messages(&requests[1])
        .iter()
        .map(|(id, _)| id.clone())
        .collect();
    assert_eq!(ids, ["call_0", "call_1"]);

    // A find of an item is not a party: no card.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "kela kitna bika",
        None,
        vec![
            tool_reply(&[find("kela", "item")]),
            final_reply("{{r1.matches.0.name}}"),
        ],
    );
    assert_eq!(tool_messages(&requests[1]).len(), 1);
}

#[test]
fn two_parties_in_one_reply_get_a_card_each() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "akram aur arshad dono ka bakaya",
        None,
        vec![
            tool_reply(&[find("akram", "buyer"), find("arshad", "buyer")]),
            final_reply(
                "{{r1.matches.0.name}} {{r3.balance}} اور {{r2.matches.0.name}} {{r4.balance}}۔",
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 2);
    let figs = figures(answer_of(&reply));
    assert_eq!(balance(figs[0].0), (2_880, BalanceSide::Dr));
    assert_eq!(balance(figs[1].0), (18_684, BalanceSide::Dr));
    h.scan_requests();
}

#[test]
fn what_a_buyer_bought_and_commission_are_read_ahead_when_no_period_is_named() {
    // "akram ne kya khareeda": R2 with him as party (his lines: item, qty, rate, value).
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "akram ne kya khareeda",
        None,
        vec![
            tool_reply(&[find("akram", "buyer")]),
            answer_with(
                "answer",
                "{{r1.matches.0.name}} نے {{r3.rows.je1.item}} کے {{r3.rows.je1.qty}} نگ لیے، قیمت {{r3.totals.value}}۔",
                &[],
                json!([]),
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 2);
    let tools = tool_messages(&requests[1]);
    let ids: Vec<String> = tools
        .iter()
        .map(|(_, v)| v["id"].as_str().unwrap_or("").to_owned())
        .collect();
    assert_eq!(ids, ["r1", "r2", "r3"]);
    assert_eq!(tools[2].1["source"], "R2");
    assert!(tools[2].1.get("error").is_none());

    // Commission of a seller: view B with him as seller, over the whole books.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "rahim ke maal pe commission",
        None,
        vec![
            tool_reply(&[find("rahim", "seller")]),
            final_reply("{{r1.matches.0.name}} {{r2.balance}}"),
        ],
    );
    let _ = reply;
    let tools = tool_messages(&requests[1]);
    let ids: Vec<String> = tools
        .iter()
        .map(|(_, v)| v["id"].as_str().unwrap_or("").to_owned())
        .collect();
    assert_eq!(ids, ["r1", "r2", "r3"], "{:?}", tools);
    assert!(tools[2].1.get("error").is_none(), "{}", tools[2].1);
    assert!(
        tools[2].1.to_string().contains("total_comm"),
        "the commission column: {}",
        tools[2].1
    );

    // A period in the question: the card only; the model asks for the period itself.
    let h = Harness::day();
    let (_, requests) = h.ask(
        "akram ka khata is mahine",
        None,
        vec![
            tool_reply(&[find("akram", "buyer")]),
            final_reply("{{r1.matches.0.name}} {{r2.balance}}"),
        ],
    );
    let ids: Vec<String> = tool_messages(&requests[1])
        .iter()
        .map(|(_, v)| v["id"].as_str().unwrap_or("").to_owned())
        .collect();
    assert_eq!(ids, ["r1", "r2"]);
}

// ---- repair -----------------------------------------------------------------------------------

#[test]
fn a_typed_name_that_the_results_hold_is_put_back_and_no_rewrite_is_asked() {
    // The Akram trap of the live run: "اکرم ٹول ٹیکس" breaks two rules, the name and the never-use
    // word "ٹول". The app puts the name back as a reference, and checks everything again.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "how much does akram owe",
        None,
        vec![
            tool_reply(&[find("akram", "buyer")]),
            final_reply("اکرم ٹول ٹیکس کے ذمے {{r2.balance}} باقی ہیں۔"),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(
        requests.len(),
        2,
        "no third call: the rewrite was not needed"
    );
    let answer = answer_of(&reply);
    assert!(
        matches!(&answer.parts[0], AnswerPart::Name { name } if name.name_en == "Akram Toll Tax"),
        "{:?}",
        answer.parts
    );
    assert_eq!(balance(figures(answer)[0].0), (2_880, BalanceSide::Dr));
    // The log says so, and the shown answer holds the reference.
    let (calls, _) = log_row(&h);
    assert_eq!(calls.len(), 2);
    // The question log keeps the model's own words, with the name typed, as the model wrote them.
    let (rows, _, _) = mandi_services::assistant::list_questions(&h.world.env.ctx, 0, 10).unwrap();
    let written = rows[0].answer_json.as_deref().unwrap();
    assert!(written.contains("اکرم ٹول ٹیکس کے ذمے"), "{written}");
    assert_eq!(calls[1]["repaired"], json!(["name"]));
    assert_eq!(calls[1]["rewrite"], false);
    h.scan_requests();
}

#[test]
fn an_english_name_in_an_urdu_answer_and_a_name_in_a_next_question_are_put_back_too() {
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "how much does akram owe",
        None,
        vec![
            tool_reply(&[find("akram", "buyer")]),
            answer_with(
                "answer",
                "Akram Toll Tax کے ذمے {{r2.balance}} باقی ہیں۔",
                &["اکرم ٹول ٹیکس کی بکریاں؟"],
                json!([]),
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 2);
    let answer = answer_of(&reply);
    assert!(matches!(&answer.parts[0], AnswerPart::Name { .. }));
    assert_eq!(answer.next.len(), 1);
    h.scan_requests();
}

#[test]
fn the_repair_checks_everything_again_so_another_fault_still_costs_the_rewrite() {
    // The name is put back, but a typed figure is still there: the rewrite is asked, and it is told
    // about the figure only (the name is no longer a fault).
    let h = Harness::day();
    let bad = final_reply("اکرم ٹول ٹیکس کے ذمے 2,880 باقی ہیں۔");
    let (reply, requests) = h.ask(
        "how much does akram owe",
        None,
        vec![tool_reply(&[find("akram", "buyer")]), bad.clone(), bad],
    );
    assert_eq!(reply.outcome, AskOutcome::ChecksFailed, "{reply:#?}");
    assert_eq!(requests.len(), 3, "find, the answer, the one rewrite");
    let told = failed_rules(&requests[2]);
    assert!(told.contains(&"digit".to_owned()), "{told:?}");
    assert!(!told.contains(&"name".to_owned()), "{told:?}");
    h.scan_requests();
}

#[test]
fn a_refused_reference_is_logged_by_its_path_for_the_next_live_run() {
    let h = Harness::day();
    let bad = final_reply("{{r1.matches.0.name}} کے ذمے {{r2.rows.0.balance}} باقی ہیں۔");
    let (reply, _) = h.ask(
        "how much does akram owe",
        None,
        vec![tool_reply(&[find("akram", "buyer")]), bad.clone(), bad],
    );
    assert_eq!(reply.outcome, AskOutcome::ChecksFailed, "{reply:#?}");
    let (calls, _) = log_row(&h);
    let rewrite = calls.iter().find(|c| c["rewrite"] == true).unwrap();
    assert_eq!(rewrite["check_failed"], json!(["reference"]));
    assert_eq!(rewrite["check_paths"], json!(["r2.rows.0.balance"]));
}

// ---- the repair, name by name -----------------------------------------------------------------

mod put_back {
    use mandi_assistant::check::NameSet;
    use mandi_assistant::results::{NameVal, Node, ToolResult, Val};
    use mandi_assistant::tools::State;

    fn name(ur: &str, en: &str, code: &str) -> Node {
        Node::Val(Val::Name(NameVal {
            name_ur: ur.to_owned(),
            name_en: en.to_owned(),
            tag: None,
            code: Some(code.to_owned()),
        }))
    }

    fn result(id: &str, matches: Vec<Node>) -> ToolResult {
        ToolResult {
            id: id.to_owned(),
            tool: "find",
            input: serde_json::json!({}),
            node: Node::obj(vec![(
                "matches",
                Node::List(
                    matches
                        .into_iter()
                        .map(|n| Node::obj(vec![("name", n)]))
                        .collect(),
                ),
            )]),
            shown: None,
            source: None,
            partner_parts_locked: false,
            partner_data: false,
            turn: 0,
            working: None,
        }
    }

    fn state(results: Vec<ToolResult>) -> State {
        State { results, turn: 0 }
    }

    #[test]
    fn a_name_of_one_record_becomes_the_reference_to_it_in_either_script() {
        let s = state(vec![result(
            "r1",
            vec![name("اکرم ٹول ٹیکس", "Akram Toll Tax", "0020")],
        )]);
        let names = NameSet::new(&s, &[]);
        assert_eq!(
            names
                .put_back(&s, "اکرم ٹول ٹیکس کے ذمے {{r2.balance}} ہیں۔")
                .as_deref(),
            Some("{{r1.matches.0.name}} کے ذمے {{r2.balance}} ہیں۔")
        );
        // English, in any case, with other punctuation between the words.
        assert_eq!(
            names
                .put_back(&s, "Owed by akram, toll tax: {{r2.balance}}")
                .as_deref(),
            Some("Owed by {{r1.matches.0.name}}: {{r2.balance}}")
        );
        // Twice in a text, both put back; a reference already there is left as it is.
        assert_eq!(
            names
                .put_back(&s, "{{r1.matches.0.name}} اور اکرم ٹول ٹیکس، اکرم ٹول ٹیکس")
                .as_deref(),
            Some("{{r1.matches.0.name}} اور {{r1.matches.0.name}}، {{r1.matches.0.name}}")
        );
    }

    #[test]
    fn nothing_changes_for_a_text_without_a_name_or_with_a_name_the_results_do_not_hold() {
        let s = state(vec![result(
            "r1",
            vec![name("اکرم ٹول ٹیکس", "Akram Toll Tax", "0020")],
        )]);
        let names = NameSet::new(&s, &[]);
        assert_eq!(
            names.put_back(&s, "{{r1.matches.0.name}} کے ذمے ہیں۔"),
            None
        );
        assert_eq!(names.put_back(&s, "زاہد کے ذمے ہیں۔"), None);
        // Half a name is not the name (the check does not take it either).
        assert_eq!(names.put_back(&s, "اکرم کے ذمے ہیں۔"), None);
        assert_eq!(names.put_back(&s, ""), None);
    }

    #[test]
    fn a_name_that_two_records_share_is_left_for_the_rewrite() {
        // Two different accounts with the same name text: which one the sentence means is not
        // for the app to guess.
        let s = state(vec![
            result("r1", vec![name("اکرم", "Akram", "0020")]),
            result("r2", vec![name("اکرم", "Akram", "0021")]),
        ]);
        let names = NameSet::new(&s, &[]);
        assert_eq!(names.put_back(&s, "اکرم کے ذمے ہیں۔"), None);
        // The same record in two results is one record: put back, from the first result.
        let s = state(vec![
            result("r1", vec![name("اکرم", "Akram", "0020")]),
            result("r2", vec![name("اکرم", "Akram", "0020")]),
        ]);
        let names = NameSet::new(&s, &[]);
        assert_eq!(
            names.put_back(&s, "اکرم کے ذمے ہیں۔").as_deref(),
            Some("{{r1.matches.0.name}} کے ذمے ہیں۔")
        );
    }

    #[test]
    fn a_digit_or_a_brace_between_the_words_of_a_name_is_not_swallowed() {
        let s = state(vec![result(
            "r1",
            vec![name("اکرم ٹول ٹیکس", "Akram Toll Tax", "0020")],
        )]);
        let names = NameSet::new(&s, &[]);
        // The typed figure would vanish with the repair: the text is left to the rewrite.
        assert_eq!(names.put_back(&s, "اکرم 2,880 ٹول ٹیکس کے ذمے"), None);
        assert_eq!(names.put_back(&s, "اکرم {{ ٹول }} ٹیکس"), None);
        assert_eq!(names.put_back(&s, "اکرم ✓ ٹول ٹیکس"), None);
        // Spaces and the punctuation of a sentence between the words are fine.
        assert!(names.put_back(&s, "اکرم، ٹول - ٹیکس کے ذمے").is_some());
    }

    #[test]
    fn a_name_with_no_code_or_a_pukhta_account_code_is_not_put_back() {
        // An item has no code; a پختہ کھاتہ account stands for many people (name and phone).
        let item = Node::Val(Val::Name(NameVal {
            name_ur: "کیلا".to_owned(),
            name_en: "Banana".to_owned(),
            tag: None,
            code: None,
        }));
        let s = state(vec![result("r1", vec![item])]);
        let names = NameSet::new(&s, &[]);
        assert_eq!(names.put_back(&s, "کیلا بکا"), None);
        let s = state(vec![result("r1", vec![name("شاہد", "Shahid", "03049999")])]);
        let names = NameSet::new(&s, &[]);
        assert_eq!(names.put_back(&s, "شاہد کے ذمے"), None);
        let s = state(vec![result(
            "r1",
            vec![name("شاہد", "Shahid", "030119999")],
        )]);
        let names = NameSet::new(&s, &[]);
        assert_eq!(names.put_back(&s, "شاہد کے ذمے"), None);
    }

    #[test]
    fn an_item_or_a_partner_is_put_back_by_its_row_key_or_its_item_column_and_a_person_is_not() {
        let no_code = |ur: &str, en: &str| {
            Node::Val(Val::Name(NameVal {
                name_ur: ur.to_owned(),
                name_en: en.to_owned(),
                tag: None,
                code: None,
            }))
        };
        let result_of = |id: &str, node: Node| ToolResult {
            id: id.to_owned(),
            tool: "analytics",
            input: serde_json::json!({}),
            node,
            shown: None,
            source: None,
            partner_parts_locked: false,
            partner_data: false,
            turn: 0,
            working: None,
        };
        // View A: rows keyed i2 (an item), and an R2 row whose item column holds the same item.
        let view_a = result_of(
            "r1",
            Node::obj(vec![(
                "rows",
                Node::Keyed(vec![(
                    "i2".to_owned(),
                    Node::obj(vec![("name", no_code("سیب", "Apple"))]),
                )]),
            )]),
        );
        let s = state(vec![view_a]);
        let names = NameSet::new(&s, &[]);
        assert_eq!(
            names.put_back(&s, "سیب کی بکری ہوئی۔").as_deref(),
            Some("{{r1.rows.i2.name}} کی بکری ہوئی۔")
        );
        // A partner found by his key.
        let partner = result_of(
            "r1",
            Node::obj(vec![(
                "matches",
                Node::List(vec![Node::obj(vec![
                    ("key", Node::Str("s1".to_owned())),
                    ("name", no_code("الف", "Partner A")),
                ])]),
            )]),
        );
        let s = state(vec![partner]);
        let names = NameSet::new(&s, &[]);
        assert_eq!(
            names.put_back(&s, "Partner A کی کل رقم").as_deref(),
            Some("{{r1.matches.0.name}} کی کل رقم")
        );
        // A person in a report cell (the buyer column): no code, no row key of his own: left alone.
        let person = result_of(
            "r1",
            Node::obj(vec![(
                "rows",
                Node::Keyed(vec![(
                    "je1".to_owned(),
                    Node::obj(vec![("buyer", no_code("شاہد", "Shahid"))]),
                )]),
            )]),
        );
        let s = state(vec![person]);
        let names = NameSet::new(&s, &[]);
        assert_eq!(names.put_back(&s, "شاہد کے ذمے"), None);
    }

    #[test]
    fn the_longer_name_wins_over_the_shorter_one_inside_it() {
        let s = state(vec![result(
            "r1",
            vec![
                name("ارشد", "Arshad", "0030"),
                name("ارشد ملتانی", "Arshad Multani", "0031"),
            ],
        )]);
        let names = NameSet::new(&s, &[]);
        assert_eq!(
            names.put_back(&s, "ارشد ملتانی اور ارشد").as_deref(),
            Some("{{r1.matches.1.name}} اور {{r1.matches.0.name}}")
        );
    }
}

// ---- a name of the books that the results do not hold ------------------------------------------

#[test]
fn the_full_name_of_another_party_or_a_partner_is_refused_even_when_the_results_do_not_hold_it() {
    // An answer about Akram that writes Arshad Multani's name (an account of the books, not in
    // this question's results): refused, told as the rule `name`, and it cannot be repaired
    // because the results do not say which record it is.
    let h = Harness::day();
    let bad = final_reply("ارشد ملتانی کے ذمے {{r2.balance}} باقی ہیں۔");
    let (reply, requests) = h.ask(
        "how much does akram owe",
        None,
        vec![tool_reply(&[find("akram", "buyer")]), bad.clone(), bad],
    );
    assert_eq!(reply.outcome, AskOutcome::ChecksFailed, "{reply:#?}");
    assert_eq!(requests.len(), 3, "find, the answer, the one rewrite");
    assert!(failed_rules(&requests[2]).contains(&"name".to_owned()));
    let (calls, _) = log_row(&h);
    assert!(
        calls.iter().all(|c| c.get("repaired").is_none()),
        "nothing was put back: {calls:?}"
    );

    // The same in English, and a partner's name (Partner A, الف).
    for typed in [
        "Arshad Multani owes {{r2.balance}}.",
        "الف کے ذمے {{r2.balance}} باقی ہیں۔",
        "Partner A has {{r2.balance}} to pay.",
    ] {
        let h = Harness::day();
        let bad = final_reply(typed);
        let (reply, requests) = h.ask(
            "how much does akram owe",
            None,
            vec![tool_reply(&[find("akram", "buyer")]), bad.clone(), bad],
        );
        assert_eq!(
            reply.outcome,
            AskOutcome::ChecksFailed,
            "{typed}: {reply:#?}"
        );
        assert!(
            failed_rules(&requests[2]).contains(&"name".to_owned()),
            "{typed}"
        );
    }
}

#[test]
fn only_a_full_name_in_whole_words_is_refused_never_a_part_of_one() {
    // "ارشد" is the first word of "ارشد ملتانی", not his full name; "ملتانی" the last: allowed.
    // A system account's name (روکڑ) is an ordinary word and is allowed.
    let h = Harness::day();
    let (reply, requests) = h.ask(
        "how much does akram owe",
        None,
        vec![
            tool_reply(&[find("akram", "buyer")]),
            final_reply(
                "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں، ارشد سے الگ، ملتانی روکڑ۔",
            ),
        ],
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!(requests.len(), 2);
}

mod book {
    use mandi_assistant::check::NameBook;
    use std::time::Instant;

    /// A number written in letters, because digits are not part of a name (the check sees them as
    /// spaces): 77 is "ccc" in base 26 for the English name and the Urdu letters for the other.
    fn letters(mut i: usize, alphabet: &[char]) -> String {
        let mut out = String::new();
        loop {
            out.push(alphabet[i % alphabet.len()]);
            i /= alphabet.len();
            if i == 0 {
                return out;
            }
        }
    }

    fn names(n: usize) -> Vec<(String, String, Option<String>)> {
        // 20,000 made-up accounts: the old software had 1,572, the plan allows 20,000.
        let latin: Vec<char> = ('a'..='z').collect();
        let urdu: Vec<char> = "ابپتٹجچحخدڈرزسشصضطظعغفقکگلمنوہیے".chars().collect();
        (0..n)
            .map(|i| {
                (
                    format!("خریدار {} ٹول", letters(i, &urdu)),
                    format!("Buyer {} Toll", letters(i, &latin)),
                    (i % 7 == 0).then(|| "K".to_owned()),
                )
            })
            .collect()
    }

    #[test]
    fn a_book_of_twenty_thousand_accounts_is_built_and_searched_in_milliseconds() {
        let all = names(20_000);
        let began = Instant::now();
        let book = NameBook::new(&all, &["Cash".to_owned()]);
        let built = began.elapsed();
        assert_eq!(book.len(), 40_000);
        let target = format!(
            "خریدار {} ٹول",
            letters(
                77,
                &"ابپتٹجچحخدڈرزسشصضطظعغفقکگلمنوہیے"
                    .chars()
                    .collect::<Vec<_>>()
            )
        );
        let text = format!(
            "{} اور یہاں {target} آ گیا۔ {}",
            "اس جواب میں بہت سے الفاظ ہیں مگر کوئی پورا نام نہیں ہے۔ ".repeat(6),
            "اس کے بعد بھی کچھ الفاظ ہیں۔ ".repeat(6)
        );
        let began = Instant::now();
        for _ in 0..50 {
            assert_eq!(book.first_in(&text).as_deref(), Some(target.as_str()));
        }
        let searched = began.elapsed() / 50;
        eprintln!(
            "book of 20,000 accounts: built in {built:?}, one search of a long text {searched:?}"
        );
        // Generous limits: the real numbers are far below, a model call takes seconds.
        assert!(built.as_millis() < 2_000, "{built:?}");
        assert!(searched.as_millis() < 100, "{searched:?}");
        // Part of a name is not a name.
        assert_eq!(book.first_in("خریدار ٹول"), None);
        assert_eq!(
            book.first_in("Cash"),
            None,
            "a system name is an ordinary word"
        );
    }
}
