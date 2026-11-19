//! A small builder for audit rows, so a service says what happened in one line (PRD FR-21.3,
//! Code Standards S7).

use mandi_db::audit::AuditEntry;
use serde_json::{Map, Value};

/// One thing that happened. Never put a password, a hash or a recovery code in it.
pub(crate) struct Event(AuditEntry);

impl Event {
    /// Who did what to what kind of record.
    pub fn new(user_id: Option<i64>, action: &str, entity: &str) -> Self {
        Self(AuditEntry {
            user_id,
            action: action.to_owned(),
            entity: entity.to_owned(),
            entity_id: None,
            old_json: None,
            new_json: None,
            reason: None,
        })
    }

    /// The record that was changed.
    pub fn entity_id(mut self, id: i64) -> Self {
        self.0.entity_id = Some(id);
        self
    }

    /// The value before.
    pub fn old(mut self, value: Value) -> Self {
        self.0.old_json = Some(value);
        self
    }

    /// The value after.
    pub fn new_value(mut self, value: Value) -> Self {
        self.0.new_json = Some(value);
        self
    }

    /// Why it was done (a reopened day, a cancel).
    pub fn reason(mut self, reason: &str) -> Self {
        self.0.reason = Some(reason.to_owned());
        self
    }

    /// The Owner's sign given when the day was reopened to fix an entry (PRD FR-21.4, plan 73): saved
    /// beside the reason, as the key `sign` of the new value. Nothing is added when there is none.
    pub fn sign(mut self, sign: Option<&str>) -> Self {
        if let Some(sign) = sign {
            match self.0.new_json.as_mut() {
                Some(Value::Object(object)) => {
                    object.insert("sign".to_owned(), Value::String(sign.to_owned()));
                }
                Some(_) => {}
                None => {
                    let mut object = Map::new();
                    object.insert("sign".to_owned(), Value::String(sign.to_owned()));
                    self.0.new_json = Some(Value::Object(object));
                }
            }
        }
        self
    }

    pub fn into_entry(self) -> AuditEntry {
        self.0
    }
}
