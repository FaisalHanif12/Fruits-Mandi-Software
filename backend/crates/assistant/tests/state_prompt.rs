//! The prompt and the code agree on the read gaps of plan 55 §11.1 (R1 to R6, P1 to P4, the
//! collector): every word the real `tools.json` offers is a word the code reads.
//!
//! - each role of `find`, including `staff`, `expense_head` and `truck`;
//! - every property of `report`, with the inputs `seller`, `zamindar`, `status`, `voucher_type`,
//!   `layout` and `collector`;
//! - every property and every `what` of `app_state`;
//! - the enum words of the new report inputs, and the instructions naming where to look.
//!
//! The size of the fixed part is test P1 of `prompt.rs`.
#![cfg(test)]

use mandi_assistant::prompt::{SYSTEM, TOOLS_JSON};
use mandi_assistant::tools::find::{Input as FindInput, Role};
use mandi_assistant::tools::report::Input as ReportInput;
use mandi_assistant::tools::state::{Input as StateInput, What};
use mandi_services::core::vouchers::VoucherType;
use serde_json::{Value, json};

fn tools() -> Value {
    serde_json::from_str(TOOLS_JSON).unwrap()
}

fn tool(name: &str) -> Value {
    tools()
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["function"]["name"] == name)
        .unwrap_or_else(|| panic!("no tool {name}"))
        .clone()
}

#[test]
fn every_role_of_find_is_a_role_the_code_reads() {
    let found = tool("find");
    let roles = found["function"]["parameters"]["properties"]["role"]["enum"]
        .as_array()
        .unwrap()
        .clone();
    assert!(roles.len() >= 16);
    for role in roles {
        serde_json::from_value::<FindInput>(json!({"text": "x", "role": role}))
            .unwrap_or_else(|e| panic!("{role}: {e}"));
    }
    for (word, role) in [
        ("staff", Role::Staff),
        ("expense_head", Role::ExpenseHead),
        ("truck", Role::Truck),
    ] {
        let parsed: FindInput = serde_json::from_value(json!({"text": "x", "role": word})).unwrap();
        assert_eq!(parsed.role, role);
    }
}

#[test]
fn every_property_of_report_is_one_the_code_reads() {
    let report = tool("report");
    let props = report["function"]["parameters"]["properties"]
        .as_object()
        .unwrap();
    // Sent null, as a strict call does.
    let all_null: serde_json::Map<String, Value> = props
        .keys()
        .map(|k| {
            let value = if k == "id" { json!("R2") } else { Value::Null };
            (k.clone(), value)
        })
        .collect();
    let parsed: ReportInput = serde_json::from_value(Value::Object(all_null)).unwrap();
    assert!(parsed.seller.is_none() && parsed.layout.is_none() && parsed.collector.is_none());
    for name in [
        "seller",
        "zamindar",
        "status",
        "voucher_type",
        "layout",
        "collector",
    ] {
        assert!(props.contains_key(name), "{name}");
    }
}

#[test]
fn every_property_of_app_state_is_one_the_code_reads() {
    let state = tool("app_state");
    let props = state["function"]["parameters"]["properties"]
        .as_object()
        .unwrap();
    let whats = props["what"]["enum"].as_array().unwrap().clone();
    assert_eq!(whats.len(), 4);
    for what in whats {
        let input: StateInput = serde_json::from_value(
            json!({"what": what, "party": null, "item": null, "bikri": null}),
        )
        .unwrap_or_else(|e| panic!("{what}: {e}"));
        assert!(matches!(
            input.what,
            What::Rules | What::Attention | What::Days | What::Audit
        ));
    }
    // Nothing the code does not read.
    let all: serde_json::Map<String, Value> = props
        .keys()
        .map(|k| {
            let value = if k == "what" {
                json!("days")
            } else {
                Value::Null
            };
            (k.clone(), value)
        })
        .collect();
    serde_json::from_value::<StateInput>(Value::Object(all)).unwrap();
}

#[test]
fn the_new_enum_words_of_report_are_the_words_the_code_takes() {
    let report = tool("report");
    let props = &report["function"]["parameters"]["properties"];
    let words = |name: &str| -> Vec<String> {
        props[name]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect()
    };
    assert_eq!(words("status"), ["paid", "due"]);
    assert_eq!(
        words("layout"),
        [
            "detail", "trip", "item", "day", "month", "partner", "garden"
        ]
    );
    // `voucher_type` is a plain string (a 13-word list would cost about 60 tokens): its
    // description names the main kinds, each a real kind, and a wrong word is refused with the
    // whole list.
    let description = props["voucher_type"]["description"].as_str().unwrap();
    let named: Vec<&str> = description
        .trim_start_matches("R15:")
        .split_whitespace()
        .filter(|w| *w != "...")
        .collect();
    assert!(named.len() >= 5);
    for word in named {
        assert!(VoucherType::parse(word).is_some(), "{word}");
    }
}

#[test]
fn the_instructions_name_where_to_look() {
    // Only what the instructions must say: the properties of `report` explain themselves in
    // `tools.json` (checked above), so the instructions may be short about them.
    for word in ["app_state", "guests", "role staff", "role truck"] {
        assert!(SYSTEM.contains(word), "{word}");
    }
}
