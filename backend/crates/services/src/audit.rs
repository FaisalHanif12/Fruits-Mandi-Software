//! A small builder for audit rows, so a service says what happened in one line (PRD FR-21.3,
//! Code Standards S7).

use mandi_db::audit::AuditEntry;
use serde_json::Value;

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

    /// Why.
    pub fn reason(mut self, text: &str) -> Self {
        self.0.reason = Some(text.to_owned());
        self
    }

    pub fn into_entry(self) -> AuditEntry {
        self.0
    }
}
