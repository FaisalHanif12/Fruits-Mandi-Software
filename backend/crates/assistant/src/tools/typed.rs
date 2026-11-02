//! A typed name in a filter is looked up for the model (plan 52 item 6).
//!
//! The tools take a record as a reference to a `find` match (`r1.matches.0`). A model often types
//! the name instead (`item: "apple"`), and that cost a refusal and a whole round (Q054, Q090 of the
//! live run of 02-11-2026). Now the app does the `find` itself, in the role the field is for:
//!
//! - exactly one match that the typed word names (a name or a word of it, or the code): the filter
//!   becomes a reference to it, and the result's `note` says so;
//! - none, several, or one that only a close spelling found ("Faizal" for "Faisal"): the call is
//!   refused, and the find result is kept as `f1`, `f2` ... so the model can answer `which_one`
//!   or `not_found` with `choose: "f1"` at once, as it would after its own `find`. A close
//!   spelling is never taken without the Owner's word (money review, plan 52 item 2).
//!
//! The find results have their own letter `f`, so the ids `r1`, `r2` ... that the model counts for
//! its own calls never move. A name is never guessed: one match or none.
//!
//! Example: `analytics{view:"A", item:"apple", ...}` finds the item Apple, runs with
//! `item:"f1.matches.0"` and says "item apple was taken as the only match, f1.matches.0".

use mandi_services::core::text;
use serde_json::{Value, json};

use super::{Env, State, Stop, ToolError, find, stored};
use crate::results::{Node, Val};

/// The find role of each filter field a tool has, by tool.
fn role_of(tool: &str, field: &str) -> Option<&'static str> {
    match (tool, field) {
        ("analytics", "item") | ("report", "item") => Some("item"),
        ("analytics", "seller") => Some("seller"),
        ("analytics", "buyer") => Some("buyer"),
        ("report", "party") | ("party_card", "party") => Some("any_party"),
        ("analytics", "city") | ("report", "city") => Some("city"),
        ("analytics", "adda") | ("report", "adda") => Some("adda"),
        ("analytics", "garden") | ("report", "garden") => Some("garden"),
        ("analytics", "trip") | ("report", "trip") => Some("trip"),
        ("analytics", "partner") | ("report", "partner") => Some("partner"),
        ("report", "collector") => Some("collector"),
        _ => None,
    }
}

/// The filter fields to look at, in a fixed order.
const FIELDS: [&str; 10] = [
    "item",
    "seller",
    "buyer",
    "party",
    "city",
    "adda",
    "garden",
    "trip",
    "partner",
    "collector",
];

/// True when the text is a reference or a record key the conversation knows, not a typed name.
fn is_reference(state: &State, text: &str) -> bool {
    match text.split_once('.') {
        // A path such as `r1.matches.0` is a reference even when no such result exists: the tool
        // refuses it. A name with a dot ("Dr. Ali") has no key before the dot.
        Some((root, _)) => {
            state.find(root).is_some() || root == "q" || super::parse_key(root).is_some()
        }
        // A record key such as `a20` or `i41` is a key, known or not: an unknown one is the
        // tool's own refusal, never a name to look up.
        None => state.knows_key(text) || super::parse_key(text).is_some(),
    }
}

/// What was done: the new input, what to tell the model, and the tool-log entries of the finds the
/// app made (so a question that is read again finds them with the same ids).
pub struct Resolved {
    /// The input with the typed names replaced by references.
    pub arguments: String,
    /// What the tool did that the model did not ask for.
    pub notes: Vec<String>,
}

/// Looks up the typed names of a call. `logged` gets one entry for each find made; `asked` gets the
/// id of the find the Owner must choose from, when a name matches none, several, or only close.
///
/// # Errors
/// `ERR_UNKNOWN_REFERENCE` when a name matches none or several records (the find is kept for
/// `which_one`); `Stop::Locked` for a partner while the section is locked.
pub fn resolve(
    env: &Env<'_>,
    state: &mut State,
    turn: u32,
    tool: &str,
    arguments: &str,
    logged: &mut Vec<Value>,
    asked: &mut Option<String>,
) -> Result<Resolved, Stop> {
    let untouched = || Resolved {
        arguments: arguments.to_owned(),
        notes: Vec::new(),
    };
    let Ok(mut args) = serde_json::from_str::<Value>(arguments) else {
        return Ok(untouched());
    };
    let mut notes = Vec::new();
    let mut changed = false;
    for field in FIELDS {
        let Some(role) = role_of(tool, field) else {
            continue;
        };
        let Some(text) = args.get(field).and_then(Value::as_str).map(str::to_owned) else {
            continue;
        };
        let text = text.trim().to_owned();
        // A code is never looked up from a typed filter: a made-up code that happens to exist
        // would pick a party the Owner never named. The model finds a code with `find`.
        let only_digits = text
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c == ' ');
        if text.is_empty() || only_digits || is_reference(state, &text) {
            continue;
        }
        let input = json!({"text": text, "role": role});
        let draft = find::run(env, state, &input)?;
        let (count, more) = match &draft.node {
            Node::Obj(fields) => (
                fields.iter().find_map(|(k, v)| match (k.as_str(), v) {
                    ("matches", Node::List(items)) => Some(items.len()),
                    _ => None,
                }),
                fields
                    .iter()
                    .any(|(k, v)| k == "more" && matches!(v, Node::Bool(true))),
            ),
            _ => (None, false),
        };
        let count = count.unwrap_or(0);
        let named = count == 1 && !more && names_it(&draft.node, &text);
        let id = state.next_id('f');
        let result = stored(draft, id.clone(), turn);
        state.results.push(result);
        logged.push(json!({"id": id, "tool": "find", "input": input}));
        if named {
            if let Some(map) = args.as_object_mut() {
                map.insert(field.to_owned(), Value::String(format!("{id}.matches.0")));
            }
            notes.push(format!(
                "{field} \"{text}\" was typed, not a find result: it was taken as the only \
                 match, {id}.matches.0"
            ));
            changed = true;
        } else {
            *asked = Some(id.clone());
            let detail = if count == 0 {
                format!(
                    "{field} \"{text}\" matches no record; the search is kept as {id}, so \
                     answer not_found with choose \"{id}\""
                )
            } else if count == 1 && !more {
                format!(
                    "{field} \"{text}\" is only close to one record, not that name; it is kept \
                     as {id}, so answer which_one with choose \"{id}\" and let the Owner confirm"
                )
            } else {
                format!(
                    "{field} \"{text}\" matches several records; they are kept as {id}, so \
                     answer which_one with choose \"{id}\""
                )
            };
            return Err(ToolError::new("ERR_UNKNOWN_REFERENCE", field, &detail).into());
        }
    }
    if !changed {
        return Ok(untouched());
    }
    Ok(Resolved {
        arguments: args.to_string(),
        notes,
    })
}

/// True when the typed word names the one match: it is the code, or a word of the name (or the
/// start of the name) in either script, or the tag, or its Urdu spelling from the word list. A
/// match found only by a near spelling, or by the spelling rules alone, is not.
fn names_it(node: &Node, typed: &str) -> bool {
    let Some(first) = node.get(&["matches", "0"]) else {
        return false;
    };
    // Found as its Urdu spelling ("kela" for کیلا): named, when the word list or a learned pair
    // made the spelling; a spelling made by the rules alone is only a guess.
    if let Some(Node::Str(via)) = node.get(&["via"]) {
        return via == "word_list" || via == "learned";
    }
    let (en, ur) = (text::search_en(typed), text::search_ur(typed));
    if en.is_empty() {
        return false;
    }
    if let Some(Node::Str(code)) = first.get(&["code"])
        && code.eq_ignore_ascii_case(typed.trim())
    {
        return true;
    }
    let Some(Node::Val(Val::Name(name))) = first.get(&["name"]) else {
        return false;
    };
    let starts = |name: &str, typed: &str| {
        !typed.is_empty()
            && (name.starts_with(typed) || name.split(' ').any(|word| word.starts_with(typed)))
    };
    let tag = name.tag.as_deref().map(text::search_en).unwrap_or_default();
    starts(&text::search_en(&name.name_en), &en)
        || starts(&text::search_ur(&name.name_ur), &ur)
        || (!tag.is_empty() && tag == en)
}

/// Adds a sentence to the `note` of a result, or gives it one.
pub fn add_note(node: &mut Node, text: &str) {
    let Node::Obj(fields) = node else {
        return;
    };
    for (key, value) in fields.iter_mut() {
        if key == "note"
            && let Node::Str(old) = value
        {
            *old = format!("{text}; {old}");
            return;
        }
    }
    fields.push(("note".to_owned(), Node::Str(text.to_owned())));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(via: Option<&str>) -> Node {
        let mut fields = vec![
            ("text", Node::Str("kela".to_owned())),
            (
                "matches",
                Node::List(vec![Node::obj(vec![
                    ("key", Node::Str("i1".to_owned())),
                    (
                        "name",
                        Node::Val(Val::Name(crate::results::NameVal {
                            name_ur: "کیلا".to_owned(),
                            name_en: "Banana".to_owned(),
                            tag: None,
                            code: None,
                        })),
                    ),
                ])]),
            ),
        ];
        if let Some(via) = via {
            fields.push(("via", Node::Str(via.to_owned())));
        }
        Node::obj(fields)
    }

    #[test]
    fn a_match_found_by_its_urdu_spelling_is_named_only_when_the_word_list_made_it() {
        assert!(names_it(&found(Some("word_list")), "kela"));
        assert!(names_it(&found(Some("learned")), "kela"));
        assert!(!names_it(&found(Some("rules")), "kela"));
        // Without a spelling step: a word of the English name, the Urdu name, or a near word.
        assert!(names_it(&found(None), "banan"));
        assert!(names_it(&found(None), "کیلا"));
        assert!(!names_it(&found(None), "bananna"));
        assert!(!names_it(&found(None), ""));
    }
}
