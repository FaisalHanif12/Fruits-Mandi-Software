//! `find` covers what the Owner names in any script (plan 52 item 3): items and partners in Roman
//! Urdu, Urdu and English, and a Roman Urdu word that the raw search cannot match is tried as its
//! Urdu spelling, from the word list and the learned pairs. Raw hits always win. The worked day has
//! the items Banana/کیلا and Apple/سیب and the partners A/B/C = الف/ب/ج.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use mandi_assistant::results::{ToolResult, Words, fit, tool_result_json};
use mandi_assistant::tools::{Env, State, Stop, call};
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use serde_json::{Value, json};

struct Session<'a> {
    world: &'a world::World,
    words: Words,
    state: State,
}

impl<'a> Session<'a> {
    fn new(world: &'a world::World) -> Self {
        Self {
            world,
            words: Words::new().unwrap(),
            state: State::default(),
        }
    }

    fn find(&mut self, role: &str, text: &str) -> Value {
        let env = Env {
            ctx: &self.world.env.ctx,
            actor: &self.world.owner,
            words: &self.words,
            language: Language::Ur,
            today: BusinessDate::parse_iso("2026-10-10").unwrap(),
            partner_locked: false,
            numbers: &[],
            question: "",
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let draft = match call(
            &env,
            &self.state,
            "find",
            &json!({"text": text, "role": role}).to_string(),
        ) {
            Ok(draft) => draft,
            Err(Stop::Error(e)) => panic!("{}", e.to_json()),
            Err(Stop::Locked) => panic!("locked"),
        };
        let id = self.state.next_id('r');
        let mut result = ToolResult {
            id: id.clone(),
            tool: draft.tool,
            input: draft.input,
            node: draft.node,
            shown: draft.shown,
            source: draft.source,
            partner_parts_locked: draft.partner_parts_locked,
            partner_data: draft.partner_data,
            turn: 0,
            working: draft.working,
        };
        fit(&mut result, &self.words);
        self.state.results.push(result);
        tool_result_json(self.state.find(&id).unwrap(), &self.words)
    }
}

fn day() -> world::World {
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    w
}

fn names(found: &Value) -> Vec<String> {
    found["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"]["en"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn an_item_is_found_in_roman_urdu_urdu_and_english() {
    let w = day();
    let mut s = Session::new(&w);
    for (typed, want) in [
        ("kela", "Banana"),
        ("saib", "Apple"),
        ("seb", "Apple"),
        ("banana", "Banana"),
        ("Apple", "Apple"),
        ("کیلا", "Banana"),
        ("سیب", "Apple"),
    ] {
        let found = s.find("item", typed);
        assert_eq!(names(&found), [want], "{typed}: {found}");
        // Only a word that the raw search could not match is tried as its Urdu spelling.
        let spelled = found.get("via").is_some();
        assert_eq!(spelled, matches!(typed, "kela" | "saib" | "seb"), "{typed}");
        if spelled {
            assert_eq!(found["via"], "word_list");
            assert!(found["note"].as_str().unwrap().contains("Urdu spelling"));
        }
    }
    // Nothing named: empty, and no spelling to show.
    let none = s.find("item", "zzzfruit");
    assert!(names(&none).is_empty());
    assert!(none.get("via").is_none());
}

#[test]
fn a_partner_is_found_by_his_letter_his_urdu_name_or_a_roman_spelling() {
    let w = day();
    let mut s = Session::new(&w);
    for (typed, want) in [
        ("A", "Partner A"),
        ("a", "Partner A"),
        ("B", "Partner B"),
        ("alif", "Partner A"),
        ("الف", "Partner A"),
        ("be", "Partner B"),
        ("jeem", "Partner C"),
        ("Partner C", "Partner C"),
    ] {
        let found = s.find("partner", typed);
        assert_eq!(names(&found), [want], "{typed}: {found}");
    }
    // The word "partner" is in every name: all three, as before.
    assert_eq!(names(&s.find("partner", "partner")).len(), 3);
}

#[test]
fn a_roman_word_is_tried_as_urdu_for_an_account_too_and_raw_hits_still_win() {
    let w = day();
    let mut s = Session::new(&w);
    // "akram" is in the English name: found raw, no spelling step.
    let raw = s.find("buyer", "akram");
    assert_eq!(names(&raw), ["Akram Toll Tax"]);
    assert!(raw.get("via").is_none());
    // A seller by his Urdu name typed as is.
    assert_eq!(names(&s.find("seller", "رحیم")), ["Rahim"]);
}

#[test]
fn an_item_alias_finds_mosambi_without_the_typing_word_list() {
    use mandi_services::core::masters::{ItemInput, MasterInput};
    let w = day();
    // The shop's item is "Sweet Lime" in English, موسمی in Urdu: the raw search has nothing close to
    // "mausami" or "mosambi" in its English name.
    mandi_services::masters::save(
        &w.env.ctx,
        &w.owner,
        &MasterInput::Item(ItemInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: "موسمی".into(),
            name_en: "Sweet Lime".into(),
            item_group: "fruit".into(),
            default_packing_id: None,
            sold_by: "unit".into(),
            confirmed_rules_words: vec!["Sweet".into()],
        }),
    )
    .unwrap();
    let mut s = Session::new(&w);
    // All four spellings reach the item. The typing word list already covers most; the assistant's
    // own alias list (usual spelling "mosambi") is the safety net for the rest.
    for typed in ["mausami", "Mosami", "musambi", "mosambi", "موسمی"] {
        let found = s.find("item", typed);
        assert_eq!(names(&found), ["Sweet Lime"], "{typed}: {found}");
        let via = found.get("via").and_then(Value::as_str);
        assert!(
            matches!(via, None | Some("word_list" | "alias")),
            "{typed}: {via:?}"
        );
    }
    // The alias is for items only: a seller named so finds nothing.
    assert!(names(&s.find("seller", "mausami")).is_empty());
}
