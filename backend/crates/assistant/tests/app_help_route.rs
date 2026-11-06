//! A help answer may carry a button for every main screen (plan 51 P51-B3, PRD FR-7.89). The
//! screen opens as it is: it takes no party, and the assistant still never fills or saves anything
//! (FR-7.79). The model is a recorded transcript.
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
use mandi_assistant::answer::EntryScreen;
use mandi_assistant::ask::AskOutcome;
use mandi_assistant::model_answer::EntryScreenName;
use serde_json::{Value, json};

fn usage() -> Value {
    json!({
        "prompt_tokens": 7_800, "completion_tokens": 90,
        "prompt_tokens_details": {"cached_tokens": 7_600},
        "completion_tokens_details": {"reasoning_tokens": 20}
    })
}

fn tool_call(name: &str, args: &Value, id: &str) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null,
            "tool_calls": [{"id": id, "type": "function",
                "function": {"name": name, "arguments": args.to_string()}}]}}],
        "usage": usage(),
    })
}

fn answer_with(text: &str, entry: &Value) -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": entry, "nearest": null
        }).to_string()}}],
        "usage": usage(),
    })
}

#[test]
fn every_screen_name_of_the_schema_is_a_screen_the_answer_can_carry() {
    // 11 entry screens as before, 19 main screens, and the 10 entry screens of garden and trip
    // entries, a later sale, an edit and a cancel (D6 v1.5), in one list.
    assert_eq!(EntryScreenName::ALL.len(), 40);
    for name in EntryScreenName::ALL {
        let parsed: EntryScreenName = serde_json::from_value(json!(name))
            .unwrap_or_else(|_| panic!("{name} is not an EntryScreenName"));
        assert_eq!(
            parsed.is_entry_screen(),
            matches!(
                name,
                "bikri"
                    | "wasooli"
                    | "adaigi"
                    | "peshgi"
                    | "kharcha"
                    | "udhar_raqam"
                    | "alag_khata"
                    | "sarmaya"
                    | "phal"
                    | "general"
                    | "account"
                    | "garden_installment"
                    | "garden_cost"
                    | "garden_sale"
                    | "garden_close"
                    | "trip_purchase"
                    | "trip_cost"
                    | "trip_sale"
                    | "bikri_later_sale"
                    | "bikri_edit"
                    | "bikri_cancel"
            ),
            "{name}"
        );
    }
    // The frontend reads the same words back.
    assert_eq!(
        serde_json::to_value(EntryScreen::MarketTrips).unwrap(),
        "market_trips"
    );
}

#[test]
fn a_help_answer_carries_the_main_screen_it_is_about_with_no_party() {
    for (screen, want) in [
        ("market_trips", EntryScreen::MarketTrips),
        ("partners", EntryScreen::Partners),
        ("reports", EntryScreen::Reports),
        ("day_close", EntryScreen::DayClose),
        ("dead_amount", EntryScreen::DeadAmount),
        ("commission_rules", EntryScreen::CommissionRules),
        ("unsold_stock", EntryScreen::UnsoldStock),
    ] {
        let h = Harness::day();
        let (reply, _) = h.ask(
            "is main market safar kiya hai",
            None,
            vec![
                tool_call("app_help", &json!({"topic": "bikri_book"}), "call_0"),
                answer_with("{{r1.summary}}", &json!({"screen": screen, "party": null})),
            ],
        );
        assert_eq!(reply.outcome, AskOutcome::Answered, "{screen}: {reply:#?}");
        let route = reply.entry_route.expect("a route");
        assert_eq!(route.screen, want, "{screen}");
        assert!(
            route.party.is_none(),
            "{screen}: a main screen takes no party"
        );
    }
}

#[test]
fn a_party_given_with_a_main_screen_is_dropped_and_an_entry_screen_still_keeps_it() {
    // A buyer is found, then the model points to a main screen with him: no party is kept.
    let h = Harness::day();
    let find = json!({"text": "akram", "role": "buyer"});
    let (reply, _) = h.ask(
        "akram ka hisab kahan dekhun",
        None,
        vec![
            tool_call("find", &find, "call_0"),
            answer_with(
                "{{r1.matches.0.name}}",
                &json!({"screen": "reports", "party": "r1.matches.0"}),
            ),
        ],
    );
    let route = reply.entry_route.expect("a route");
    assert_eq!(route.screen, EntryScreen::Reports);
    assert!(route.party.is_none(), "{route:#?}");
    // The same party with an entry screen is kept, as before.
    let h = Harness::day();
    let (reply, _) = h.ask(
        "akram ka hisab kahan dekhun",
        None,
        vec![
            tool_call("find", &find, "call_0"),
            answer_with(
                "{{r1.matches.0.name}}",
                &json!({"screen": "wasooli", "party": "r1.matches.0"}),
            ),
        ],
    );
    let route = reply.entry_route.expect("a route");
    assert_eq!(route.screen, EntryScreen::Wasooli);
    assert!(route.party.is_some());
}
