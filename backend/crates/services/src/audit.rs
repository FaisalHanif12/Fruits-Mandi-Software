//! A small builder for audit rows, so a service says what happened in one line (PRD FR-21.3,
//! Code Standards S7), and the entry for a save refused on a closed day (PRD FR-21.3 v1.61).

use mandi_db::audit::AuditEntry;
use mandi_domain::core::error::{ErrorCode, ErrorValue};
use serde_json::{Map, Value};

use crate::context::Context;
use crate::error::ServiceError;

/// The audit action of a save refused because its day is closed.
pub const CLOSED_DAY_REFUSED: &str = "closed_day_refused";

/// Words in a command name that only reads or works out: its refusal is not a refused save.
const READING: [&str; 5] = ["preview", "review", "open_for_edit", "_check", "_get"];

/// The kind of record a command saves, by the start of its name.
const COMMAND_RECORDS: [(&str, &str); 9] = [
    ("bikri_", "bikri"),
    ("vouchers_", "voucher"),
    ("general_", "voucher"),
    ("fruit_", "fruit_take"),
    ("purchases_", "purchase"),
    ("trips_", "trip"),
    ("gardens_", "garden"),
    ("accounts_", "account"),
    ("partners_", "partner"),
];

/// Writes an audit entry when `command` was refused because a day is closed (PRD FR-21.3 v1.61,
/// plan 100 D): who, when, which screen (the command) and the day the books are closed up to.
/// Nothing for any other error, for a command that only reads, for the day commands (closing a
/// closed day saves no entry), or when nobody is signed in. The
/// write is its own save after the refused one; if it fails the refusal still stands and the
/// reason goes to the log.
pub fn note_refused_save(ctx: &Context, command: &str, error: &ServiceError) {
    if !error.has_code(ErrorCode::ErrDateClosed)
        || command.starts_with("day_")
        || READING.iter().any(|w| command.contains(w))
    {
        return;
    }
    let Some(session) = ctx.session_snapshot() else {
        return;
    };
    let closed_up_to = match error {
        ServiceError::Rejected(items) => items
            .iter()
            .filter(|item| item.code == ErrorCode::ErrDateClosed)
            .find_map(|item| match item.values.get("closed_up_to") {
                Some(ErrorValue::Text(day)) => Some(day.clone()),
                _ => None,
            }),
        _ => None,
    };
    let entity = COMMAND_RECORDS
        .iter()
        .find(|(start, _)| command.starts_with(start))
        .map_or("day", |(_, record)| record);
    let event = Event::new(Some(session.user_id), CLOSED_DAY_REFUSED, entity)
        .new_value(serde_json::json!({ "command": command, "closed_up_to": closed_up_to }));
    let written = ctx.db.write(|tx| {
        ctx.db.auditor().write(tx, &event.into_entry())?;
        Ok::<(), ServiceError>(())
    });
    if let Err(error) = written {
        tracing::warn!(
            command,
            "the closed day refusal was not written to the audit log: {error}"
        );
    }
}

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

    /// The Owner's sign was given when the day was reopened to fix an entry (PRD FR-21.4, FR-7.71
    /// v1.58, plan 96): saved beside the reason as the key `sign_given: true` of the new value. The sign
    /// is a secret: its text is never written to the audit log. Nothing is added when `given` is false.
    pub fn sign_given(mut self, given: bool) -> Self {
        if given {
            match self.0.new_json.as_mut() {
                Some(Value::Object(object)) => {
                    object.insert("sign_given".to_owned(), Value::Bool(true));
                }
                Some(_) => {}
                None => {
                    let mut object = Map::new();
                    object.insert("sign_given".to_owned(), Value::Bool(true));
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
