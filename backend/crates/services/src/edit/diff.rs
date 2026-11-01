//! The change list of an edit: the stored row before and after, compared field by field (PRD
//! FR-7.70). The rows are the same JSON the screens read, so a worked-out amount that moved
//! (a commission, a total cost) is in the list next to the field that made it move.
//!
//! Money fields end in `_paisa`, weights in `_grams`, rates in `_bps`; a record with both names is a
//! party, an item or a partner. A list of objects (the parts of a payment) is compared by position
//! and its objects are named by the partner or account in them.

use mandi_domain::core::edit::{ChangeValue, EditChange, EditPlace, EditPlaceKind};
use mandi_domain::core::money::Paisa;
use serde_json::{Map, Value};

/// Fields that are never shown: they identify the row or say what the buttons may do.
const IGNORED: [&str; 17] = [
    "id",
    "version",
    "number",
    "entry_id",
    "journal_entry_id",
    "current_entry_id",
    "can_edit",
    "can_cancel",
    "locked_reason",
    "needs_reopen",
    "edited",
    "status",
    "cancel_reason",
    "created_at",
    "updated_at",
    "entry_date",
    "cancelled",
];

fn is_names(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|o| o.contains_key("name_ur") && o.contains_key("name_en"))
}

fn looks_like_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
}

/// The typed value of one side of a change.
pub(crate) fn value_of(key: &str, value: &Value) -> ChangeValue {
    match value {
        Value::Null => ChangeValue::Empty,
        Value::Bool(on) => ChangeValue::Flag { on: *on },
        Value::Number(n) => match n.as_i64() {
            Some(v) if key.ends_with("_paisa") => Paisa::try_from_paisa(v)
                .map(|paisa| ChangeValue::Money { paisa })
                .unwrap_or(ChangeValue::Number { value: v }),
            Some(v) if key.ends_with("_grams") => ChangeValue::Weight { grams: v },
            Some(v) if key.ends_with("_bps") => ChangeValue::Bps { bps: v },
            Some(v) => ChangeValue::Number { value: v },
            None => ChangeValue::Text {
                text: n.to_string(),
            },
        },
        Value::String(text) if looks_like_date(text) => ChangeValue::Date { iso: text.clone() },
        Value::String(text) => ChangeValue::Text { text: text.clone() },
        other if is_names(other) => ChangeValue::Names {
            ur: other["name_ur"].as_str().unwrap_or_default().to_owned(),
            en: other["name_en"].as_str().unwrap_or_default().to_owned(),
        },
        other => ChangeValue::Text {
            text: other.to_string(),
        },
    }
}

/// The key a screen turns into a label: `amount_paisa` is `amount`.
fn field_key(key: &str) -> String {
    key.strip_suffix("_paisa")
        .or_else(|| key.strip_suffix("_grams"))
        .or_else(|| key.strip_suffix("_bps"))
        .unwrap_or(key)
        .to_owned()
}

fn entry_place() -> EditPlace {
    EditPlace {
        kind: EditPlaceKind::Entry,
        lot_no: None,
        line_no: None,
        label_ur: None,
        label_en: None,
    }
}

/// The name of an object in a list: the first record with both names inside it.
fn label_of(object: &Map<String, Value>) -> (Option<String>, Option<String>) {
    for value in object.values() {
        if is_names(value) {
            return (
                value["name_ur"].as_str().map(str::to_owned),
                value["name_en"].as_str().map(str::to_owned),
            );
        }
    }
    (None, None)
}

fn diff_into(
    before: &Map<String, Value>,
    after: &Map<String, Value>,
    place: &EditPlace,
    out: &mut Vec<EditChange>,
) {
    let mut keys: Vec<&String> = before.keys().collect();
    for key in after.keys() {
        if !before.contains_key(key) {
            keys.push(key);
        }
    }
    for key in keys {
        if IGNORED.contains(&key.as_str()) || key.ends_with("_at") {
            continue;
        }
        let (b, a) = (before.get(key), after.get(key));
        if b == a {
            continue;
        }
        let null = Value::Null;
        let (b, a) = (b.unwrap_or(&null), a.unwrap_or(&null));
        match (b, a) {
            (Value::Array(bs), Value::Array(as_)) => {
                for i in 0..bs.len().max(as_.len()) {
                    let (bo, ao) = (bs.get(i), as_.get(i));
                    let empty = Map::new();
                    let bm = bo.and_then(Value::as_object).unwrap_or(&empty);
                    let am = ao.and_then(Value::as_object).unwrap_or(&empty);
                    let (ur, en) = label_of(if am.is_empty() { bm } else { am });
                    let part = EditPlace {
                        kind: EditPlaceKind::Part,
                        lot_no: None,
                        line_no: i64::try_from(i + 1).ok(),
                        label_ur: ur,
                        label_en: en,
                    };
                    diff_into(bm, am, &part, out);
                }
            }
            (Value::Object(bm), Value::Object(am)) if !is_names(b) || !is_names(a) => {
                diff_into(bm, am, place, out);
            }
            _ => out.push(EditChange {
                place: place.clone(),
                field: field_key(key),
                before: value_of(key, b),
                after: value_of(key, a),
            }),
        }
    }
}

/// The changes between two stored rows as JSON (the same function, for `preview_of`).
pub(crate) fn changes_of_rows(before: &Value, after: &Value) -> Vec<EditChange> {
    changes(before, after)
}

/// The changes between the row before and the row after, in the order of the row.
pub(crate) fn changes(before: &Value, after: &Value) -> Vec<EditChange> {
    let mut out = Vec::new();
    if let (Some(b), Some(a)) = (before.as_object(), after.as_object()) {
        diff_into(b, a, &entry_place(), &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn money_dates_names_and_lists_are_typed_and_the_row_identity_is_left_out() {
        let before = json!({
            "id": 1, "version": 2, "amount_paisa": 1_000_000, "narration": "x",
            "account": { "id": 4, "name_ur": "اکرم", "name_en": "Akram" },
            "parts": [{ "partner": { "name_ur": "الف", "name_en": "A" }, "amount_paisa": 416_000 }],
            "return_by": null, "qty": 20
        });
        let after = json!({
            "id": 1, "version": 3, "amount_paisa": 400_000, "narration": "x",
            "account": { "id": 5, "name_ur": "ارشد", "name_en": "Arshad" },
            "parts": [{ "partner": { "name_ur": "الف", "name_en": "A" }, "amount_paisa": 432_000 }],
            "return_by": "2026-10-15", "qty": 20
        });
        let list = changes(&before, &after);
        // The fields come in the order of their names (a JSON object is sorted by name).
        let fields: Vec<&str> = list.iter().map(|c| c.field.as_str()).collect();
        assert_eq!(fields, ["account", "amount", "amount", "return_by"]);
        assert_eq!(
            list[0].after,
            ChangeValue::Names {
                ur: "ارشد".into(),
                en: "Arshad".into()
            }
        );
        assert_eq!(
            list[1].before,
            ChangeValue::Money {
                paisa: Paisa::from_rupees(10_000).unwrap()
            }
        );
        assert_eq!(list[2].place.kind, EditPlaceKind::Part);
        assert_eq!(list[2].place.label_en.as_deref(), Some("A"));
        assert_eq!(list[3].before, ChangeValue::Empty);
        assert_eq!(
            list[3].after,
            ChangeValue::Date {
                iso: "2026-10-15".into()
            }
        );
        assert!(changes(&before, &before).is_empty());
    }
}
