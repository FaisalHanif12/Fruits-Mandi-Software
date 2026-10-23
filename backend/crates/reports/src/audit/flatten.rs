//! Reading the old and new value of an audit entry: hiding secrets and listing the fields that
//! changed (PRD FR-21.3, NFR-SEC-5; Code Standards S2: a secret is never returned to the screen).
//!
//! A value is stored as JSON. Any field whose name holds `password`, `secret`, `token`,
//! `recovery`, `hash` or `audit_key` is a secret: its value is replaced by `"[hidden]"` in the
//! text and shown as `hidden` in the list of changes. A change to a secret still shows (that it
//! changed, not what it was).
//!
//! Example: old `{"firm":{"name_en":"Maher"}}` and new `{"firm":{"name_en":"Maher Adnan"}}` give
//! one change: `firm.name_en` from `Maher` to `Maher Adnan`.

use std::collections::BTreeMap;

use mandi_core::money::Paisa;
use serde_json::Value;

use super::model::{AuditChange, AuditValue};

/// The most changed fields listed for one entry.
pub(crate) const MAX_CHANGES: usize = 200;

/// The text that stands for a secret in the stored text.
const HIDDEN_TEXT: &str = "[hidden]";

/// True for a field name that holds a secret.
fn is_secret(key: &str) -> bool {
    let key = key.to_lowercase();
    [
        "password",
        "secret",
        "token",
        "recovery",
        "hash",
        "audit_key",
    ]
    .iter()
    .any(|word| key.contains(word))
}

/// The value with every secret replaced by `"[hidden]"`.
pub(crate) fn redact(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, inner)| {
                    let shown = if is_secret(key) {
                        Value::String(HIDDEN_TEXT.to_owned())
                    } else {
                        redact(inner)
                    };
                    (key.clone(), shown)
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(redact).collect()),
        other => other.clone(),
    }
}

/// One field of a flattened value. A hidden field keeps its raw text only to tell whether it
/// changed; the text never leaves this module.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Leaf {
    Null,
    Bool(bool),
    Int(i64),
    Text(String),
    Hidden(String),
}

fn flatten_into(value: &Value, path: &str, out: &mut BTreeMap<String, Leaf>) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, inner) in map {
                let inner_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                if is_secret(key) {
                    out.insert(inner_path, Leaf::Hidden(inner.to_string()));
                } else {
                    flatten_into(inner, &inner_path, out);
                }
            }
        }
        Value::Array(items) if !items.is_empty() => {
            for (index, inner) in items.iter().enumerate() {
                flatten_into(inner, &format!("{path}[{index}]"), out);
            }
        }
        Value::Object(_) => {
            out.insert(path.to_owned(), Leaf::Text("{}".to_owned()));
        }
        Value::Array(_) => {
            out.insert(path.to_owned(), Leaf::Text("[]".to_owned()));
        }
        Value::Null => {
            out.insert(path.to_owned(), Leaf::Null);
        }
        Value::Bool(flag) => {
            out.insert(path.to_owned(), Leaf::Bool(*flag));
        }
        Value::Number(number) => {
            let leaf = number
                .as_i64()
                .map_or_else(|| Leaf::Text(number.to_string()), Leaf::Int);
            out.insert(path.to_owned(), leaf);
        }
        Value::String(text) => {
            out.insert(path.to_owned(), Leaf::Text(text.clone()));
        }
    }
}

fn flatten(value: Option<&Value>) -> BTreeMap<String, Leaf> {
    let mut out = BTreeMap::new();
    if let Some(value) = value {
        flatten_into(value, "", &mut out);
    }
    out
}

/// The leaf as a value for the screen. A whole number in a field named `..._paisa` is money.
fn shown(path: &str, leaf: Option<&Leaf>) -> AuditValue {
    match leaf {
        None | Some(Leaf::Null) => AuditValue::Null,
        Some(Leaf::Bool(value)) => AuditValue::Bool { value: *value },
        Some(Leaf::Int(value)) => {
            let last = path.rsplit(['.', '[']).next().unwrap_or(path);
            match Paisa::try_from_paisa(*value) {
                Ok(paisa) if last.ends_with("_paisa") => AuditValue::Money { paisa },
                _ => AuditValue::Number { value: *value },
            }
        }
        Some(Leaf::Text(text)) => AuditValue::Text { text: text.clone() },
        Some(Leaf::Hidden(_)) => AuditValue::Hidden,
    }
}

/// The fields that differ between the old and the new value, in path order, and whether there
/// were more than [`MAX_CHANGES`]. A field that is only on one side shows `null` on the other.
pub(crate) fn changes(old: Option<&Value>, new: Option<&Value>) -> (Vec<AuditChange>, bool) {
    let before = flatten(old);
    let after = flatten(new);
    let mut paths: Vec<&String> = before.keys().chain(after.keys()).collect();
    paths.sort();
    paths.dedup();
    let mut listed = Vec::new();
    let mut truncated = false;
    for path in paths {
        let (a, b) = (before.get(path), after.get(path));
        if a == b {
            continue;
        }
        // An absent field and a null field are the same thing to the Owner.
        if matches!(a, None | Some(Leaf::Null)) && matches!(b, None | Some(Leaf::Null)) {
            continue;
        }
        if listed.len() >= MAX_CHANGES {
            truncated = true;
            break;
        }
        listed.push(AuditChange {
            path: path.clone(),
            old: shown(path, a),
            new: shown(path, b),
        });
    }
    (listed, truncated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    #[test]
    fn a_changed_field_shows_old_and_new() {
        let old = json!({ "firm": { "name_en": "Maher", "phone": "" } });
        let new = json!({ "firm": { "name_en": "Maher Adnan", "phone": "" } });
        let (list, truncated) = changes(Some(&old), Some(&new));
        assert!(!truncated);
        assert_eq!(
            list,
            [AuditChange {
                path: "firm.name_en".into(),
                old: AuditValue::Text {
                    text: "Maher".into()
                },
                new: AuditValue::Text {
                    text: "Maher Adnan".into()
                },
            }]
        );
    }

    #[test]
    fn money_fields_are_money_and_lists_use_indexes() {
        let old = json!({ "lines": [{ "dr_paisa": 120_000 }] });
        let new = json!({ "lines": [{ "dr_paisa": 130_000 }, { "dr_paisa": 5_000_000 }] });
        let (list, _) = changes(Some(&old), Some(&new));
        let paths: Vec<&str> = list.iter().map(|c| c.path.as_str()).collect();
        assert_eq!(paths, ["lines[0].dr_paisa", "lines[1].dr_paisa"]);
        assert_eq!(list[0].old, AuditValue::Money { paisa: p(1_200) });
        assert_eq!(list[0].new, AuditValue::Money { paisa: p(1_300) });
        assert_eq!(list[1].old, AuditValue::Null);
        assert_eq!(list[1].new, AuditValue::Money { paisa: p(50_000) });
    }

    #[test]
    fn a_new_entry_lists_every_field_against_null() {
        let new = json!({ "name": "x", "active": true, "n": 3, "rate": 2.5, "empty": [] });
        let (list, _) = changes(None, Some(&new));
        assert_eq!(list.len(), 5);
        assert!(list.iter().all(|c| c.old == AuditValue::Null));
        let rate = list.iter().find(|c| c.path == "rate").unwrap();
        assert_eq!(rate.new, AuditValue::Text { text: "2.5".into() });
    }

    #[test]
    fn secrets_are_hidden_but_a_change_to_one_still_shows() {
        let old = json!({ "user": "owner", "password_hash": "$argon2id$one", "audit_key": "k1" });
        let new = json!({ "user": "owner", "password_hash": "$argon2id$two", "audit_key": "k1" });
        let (list, _) = changes(Some(&old), Some(&new));
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].path, "password_hash");
        assert_eq!(
            (&list[0].old, &list[0].new),
            (&AuditValue::Hidden, &AuditValue::Hidden)
        );
        let text = redact(&new).to_string();
        assert!(!text.contains("argon2id") && !text.contains("k1"), "{text}");
        assert!(text.contains("[hidden]"));
    }

    #[test]
    fn more_than_the_limit_is_cut_and_said() {
        let big: serde_json::Map<String, Value> =
            (0..250).map(|i| (format!("f{i:03}"), json!(i))).collect();
        let (list, truncated) = changes(None, Some(&Value::Object(big)));
        assert_eq!(list.len(), MAX_CHANGES);
        assert!(truncated);
    }
}
