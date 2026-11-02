//! Whole records from the keys the model was shown (lead decision, plan 48 T1).
//!
//! A button under an answer opens a report with its filters as picker values: id, code, both names
//! and the tag. The model only passes keys (`a20`, `i41`); the names are read back from the results
//! that showed those keys, so nothing is looked up twice and nothing is typed by the model.

use mandi_services::core::dates::BusinessDate;

use super::{Env, State, ToolError, parse_key, record};
use crate::results::{NameVal, Node, Val};
use crate::source::AnswerRecord;

/// The first name found inside a node: a field called `name`, else any name.
fn first_name(node: &Node) -> Option<&NameVal> {
    match node {
        Node::Val(Val::Name(name)) => Some(name),
        Node::Obj(fields) => fields
            .iter()
            .find(|(k, _)| k == "name")
            .and_then(|(_, v)| first_name(v))
            .or_else(|| fields.iter().find_map(|(_, v)| first_name(v))),
        Node::List(items) => items.iter().find_map(first_name),
        Node::Keyed(rows) => rows.iter().find_map(|(_, v)| first_name(v)),
        _ => None,
    }
}

/// The node of a record: an object with this `key`, or a keyed row.
fn node_of<'n>(node: &'n Node, key: &str) -> Option<&'n Node> {
    match node {
        Node::Obj(fields) => {
            let own = fields
                .iter()
                .any(|(k, v)| k == "key" && matches!(v, Node::Str(s) if s == key));
            if own {
                return Some(node);
            }
            fields.iter().find_map(|(_, v)| node_of(v, key))
        }
        Node::List(items) => items.iter().find_map(|n| node_of(n, key)),
        Node::Keyed(rows) => {
            let own = rows.iter().find(|(k, _)| k == key).map(|(_, v)| v);
            own.or_else(|| rows.iter().find_map(|(_, v)| node_of(v, key)))
        }
        _ => None,
    }
}

/// The whole record of a key the conversation showed.
pub fn answer_record(state: &State, key: &str) -> Option<AnswerRecord> {
    let id = parse_key(key)?.id;
    state.results.iter().find_map(|result| {
        let node = node_of(&result.node, key)?;
        let name = first_name(node)?;
        Some(AnswerRecord {
            id,
            code: name.code.clone(),
            name_ur: name.name_ur.clone(),
            name_en: name.name_en.clone(),
            tag: name.tag.clone(),
        })
    })
}

/// A record the model names (a find match or a key), with the key prefixes it may have, and its
/// whole record.
///
/// # Errors
/// `ERR_UNKNOWN_REFERENCE` for a record never shown; `ERR_FORMAT_INVALID` for another kind of
/// record than the field takes (an item where a party goes).
pub fn named(
    env: &Env<'_>,
    state: &State,
    text: &str,
    field: &str,
    prefixes: &[&str],
) -> Result<(i64, AnswerRecord), ToolError> {
    let rec = record(env, state, text, field)?;
    if !prefixes.contains(&rec.prefix.as_str()) {
        return Err(ToolError::new(
            "ERR_FORMAT_INVALID",
            field,
            "this key is another kind of record",
        ));
    }
    let key = format!("{}{}", rec.prefix, rec.id);
    let whole = answer_record(state, &key)
        .ok_or_else(|| ToolError::new("ERR_UNKNOWN_REFERENCE", field, text))?;
    Ok((rec.id, whole))
}

/// A date as `YYYY-MM-DD` text, for the filters of a source.
pub fn iso(date: BusinessDate) -> String {
    date.to_iso()
}
