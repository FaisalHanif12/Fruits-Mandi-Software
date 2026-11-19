//! P9 of D6-system-prompt.md Section 7 (plan 51 P51-B3): the guide that `app_help` returns.
//!
//! The guide is the app's own text, put into the answer word for word, so it must be right and it
//! must follow the same rules as the model's words: the topics are the topics of the tool, both
//! languages are there, no word of the never-use list is in it, and every key it names is a key
//! that the app has. A key or a step that is wrong here is wrong in front of the Owner.
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use mandi_assistant::prompt::{Guide, GuideText, TOOLS_JSON, WordKind, guide, words};
use serde_json::Value;

/// The most words of one topic in one language (summary, steps, keys and example), a sanity cap:
/// a help answer cites the summary and the steps, and the check counts a reference as one word.
const MAX_WORDS_EN: usize = 165;
const MAX_WORDS_UR: usize = 190;

fn texts(g: &GuideText) -> Vec<&str> {
    let mut all = vec![g.title.as_str(), g.summary.as_str()];
    all.extend(g.steps.iter().map(String::as_str));
    all.push(g.keys.as_str());
    all.push(g.example.as_str());
    all
}

fn word_count(g: &GuideText) -> usize {
    [g.summary.as_str(), g.keys.as_str(), g.example.as_str()]
        .into_iter()
        .chain(g.steps.iter().map(String::as_str))
        .map(|t| t.split_whitespace().count())
        .sum()
}

/// The keys the app has: every `label: '...'` of `shared/keyboard/shortcuts.ts`, and Ctrl+S, which
/// the registry gives to save as a second key (`also`).
fn app_keys() -> BTreeSet<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/src/shared/keyboard/shortcuts.ts");
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    let mut keys: BTreeSet<String> = source
        .split("label: '")
        .skip(1)
        .filter_map(|rest| rest.split('\'').next())
        .map(str::to_owned)
        .collect();
    keys.insert("Ctrl+S".to_owned());
    keys
}

/// The keys a text names: `F10`, `Alt+B`, `Ctrl+Enter`, `Ctrl+Up`, `Alt+Down`, `Esc`.
fn named_keys(text: &str) -> BTreeSet<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = BTreeSet::new();
    let mut i = 0;
    while i < chars.len() {
        let rest: String = chars[i..].iter().collect();
        let before_ok = i == 0 || !chars[i - 1].is_alphanumeric();
        let modifier = ["Ctrl+", "Alt+"].iter().find(|m| rest.starts_with(**m));
        if before_ok && let Some(modifier) = modifier {
            let tail: String = rest[modifier.len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '↑' | '↓'))
                .collect();
            let key = match tail.as_str() {
                "Up" => "↑",
                "Down" => "↓",
                other => other,
            };
            found.insert(format!("{modifier}{key}"));
            i += modifier.len();
            continue;
        }
        // F1 to F12 as a whole word.
        if before_ok && chars[i] == 'F' {
            let digits: String = chars[i + 1..]
                .iter()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            let after = chars.get(i + 1 + digits.len());
            if !digits.is_empty() && !after.is_some_and(|c| c.is_alphanumeric()) {
                found.insert(format!("F{digits}"));
            }
        }
        i += 1;
    }
    found
}

/// True when `needle` is in `text` as a whole word or phrase.
fn has_word(text: &str, needle: &str) -> bool {
    let norm = |s: &str| -> String {
        let mut out = String::from(" ");
        for c in s.chars() {
            out.push(if c.is_alphanumeric() { c } else { ' ' });
        }
        out.push(' ');
        out.split_whitespace().collect::<Vec<_>>().join(" ")
    };
    format!(" {} ", norm(text).to_lowercase())
        .contains(&format!(" {} ", norm(needle).to_lowercase()))
}

fn topics_of_tool(tools_text: &str) -> BTreeSet<String> {
    let tools: Value = serde_json::from_str(tools_text).unwrap();
    tools
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["function"]["name"] == "app_help")
        .unwrap_or_else(|| panic!("no app_help in tools.json"))["function"]["parameters"]["properties"]["topic"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap().to_owned())
        .collect()
}

pub fn check(guide: &Guide, tools_text: &str) {
    // The topics of the guide are the topics of the tool, both ways.
    let in_guide: BTreeSet<String> = guide.topics.keys().cloned().collect();
    assert_eq!(
        in_guide,
        topics_of_tool(tools_text),
        "the guide and the enum of app_help"
    );
    assert!(in_guide.len() >= 30, "a topic for every screen and module");

    let keys = app_keys();
    for (topic, entry) in &guide.topics {
        for (language, g) in [("en", &entry.en), ("ur", &entry.ur)] {
            let at = format!("{topic} ({language})");
            for text in texts(g) {
                assert!(!text.trim().is_empty(), "{at}: an empty text");
                assert_eq!(text, text.trim(), "{at}: spaces around a text");
            }
            assert!(
                (1..=4).contains(&g.steps.len()),
                "{at}: 1 to 4 steps, not {}",
                g.steps.len()
            );
            let cap = if language == "en" {
                MAX_WORDS_EN
            } else {
                MAX_WORDS_UR
            };
            assert!(
                word_count(g) <= cap,
                "{at}: {} words, at most {cap}",
                word_count(g)
            );
            // The words the app never uses (Code Standards Q11), in the language of the text.
            let never: Vec<&String> = words().get(WordKind::Never, language).iter().collect();
            for text in texts(g) {
                for word in &never {
                    assert!(!has_word(text, word), "{at}: the word {word:?} in {text:?}");
                }
            }
            // Every key it names is a key the app has.
            for text in texts(g) {
                for key in named_keys(text) {
                    assert!(
                        keys.contains(&key),
                        "{at}: the key {key} is not in the registry ({text:?})"
                    );
                }
            }
        }
        // The same keys in both languages.
        let en: BTreeSet<String> = texts(&entry.en).into_iter().flat_map(named_keys).collect();
        let ur: BTreeSet<String> = texts(&entry.ur).into_iter().flat_map(named_keys).collect();
        assert_eq!(en, ur, "{topic}: the keys named in English and in Urdu");
        // The steps are as many in both languages.
        assert_eq!(entry.en.steps.len(), entry.ur.steps.len(), "{topic}: steps");
    }
}

#[test]
fn p9_the_guide_is_the_topics_of_the_tool_in_both_languages_with_the_keys_of_the_app() {
    check(guide(), TOOLS_JSON);
}
