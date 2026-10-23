//! The types of the audit viewer (plan 22 early, PRD FR-21.3, B31).
//!
//! The viewer is read only and paged: one row for each audit entry, newest first, with the old
//! and the new value both as stored (secrets hidden) and as a list of the fields that changed.
//! Money in an old or new value travels as paisa and is written by the screen with the shared
//! formatter; the screen works out nothing.

use mandi_core::money::Paisa;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The filters of the audit viewer (PRD FR-21.3: the Owner can filter it). Every key may be left
/// out. Dates are text as typed (`10-10-2026`, `10`, `10-9`) or `YYYY-MM-DD`, in Pakistan time.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct AuditFilters {
    /// Who did it.
    #[ts(type = "number | null")]
    pub user_id: Option<i64>,
    /// What was done, for example `edit`. Empty means any.
    pub action: String,
    /// What it was done to, for example `settings`. Empty means any.
    pub entity: String,
    /// The record, with `entity`.
    #[ts(type = "number | null")]
    pub entity_id: Option<i64>,
    /// First day, included. Empty means open.
    pub from: String,
    /// Last day, included. Empty means open.
    pub to: String,
    /// Words to find in the reason or in the old and new values. Empty means any.
    pub text: String,
}

/// The input of `audit_list`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AuditListInput {
    /// The filters.
    pub filters: AuditFilters,
    /// The first row, 0 or more.
    #[ts(type = "number")]
    pub offset: i64,
    /// The rows to return. 0 means 100; at most 1,000.
    #[ts(type = "number")]
    pub limit: i64,
}

/// The user of an audit entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AuditUser {
    /// The user id.
    #[ts(type = "number")]
    pub id: i64,
    /// The sign-in name.
    pub username: String,
    /// The name shown.
    pub display_name: String,
}

/// One value of a changed field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuditValue {
    /// The field had no value (not there, or null).
    Null,
    /// Yes or no.
    Bool {
        /// The value.
        value: bool,
    },
    /// A whole number.
    Number {
        /// The value.
        #[ts(type = "number")]
        value: i64,
    },
    /// Money in whole rupees, from a field whose name ends in `_paisa`.
    Money {
        /// The amount.
        #[ts(type = "Paisa")]
        paisa: Paisa,
    },
    /// Text, or a number with decimals, as stored.
    Text {
        /// The text.
        text: String,
    },
    /// A secret: never shown (password hashes, keys, recovery codes).
    Hidden,
}

/// One field that changed: its path (for example `firm.name_en` or `lines[0].dr_paisa`), the old
/// and the new value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AuditChange {
    /// The path of the field.
    pub path: String,
    /// The value before. `null` when the field is new.
    pub old: AuditValue,
    /// The value after. `null` when the field is gone.
    pub new: AuditValue,
}

/// One audit entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AuditRow {
    /// The entry number.
    #[ts(type = "number")]
    pub id: i64,
    /// When, in UTC, as stored.
    pub at: String,
    /// The day in Pakistan time, `YYYY-MM-DD`.
    pub date: String,
    /// The time in Pakistan time, `HH:MM:SS`.
    pub time: String,
    /// Who did it. Null for an entry of the system (before sign-in).
    pub user: Option<AuditUser>,
    /// What was done.
    pub action: String,
    /// What it was done to.
    pub entity: String,
    /// The record.
    #[ts(type = "number | null")]
    pub entity_id: Option<i64>,
    /// The reason typed, if any.
    pub reason: Option<String>,
    /// The old value as stored, secrets replaced by `"[hidden]"`.
    pub old_json: Option<String>,
    /// The new value as stored, secrets replaced by `"[hidden]"`.
    pub new_json: Option<String>,
    /// The fields that changed, at most 200.
    pub changes: Vec<AuditChange>,
    /// True when there were more changed fields than `changes` holds.
    pub changes_truncated: bool,
}

/// The filters as read.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, TS)]
pub struct AuditResolvedFilters {
    /// Who did it.
    pub user: Option<AuditUser>,
    /// What was done.
    pub action: Option<String>,
    /// What it was done to.
    pub entity: Option<String>,
    /// The record.
    #[ts(type = "number | null")]
    pub entity_id: Option<i64>,
    /// First day, `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`.
    pub to: Option<String>,
    /// The words searched for.
    pub text: Option<String>,
}

/// One page of the audit log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AuditPage {
    /// The filters as read.
    pub filters: AuditResolvedFilters,
    /// The entries of this page, newest first.
    pub rows: Vec<AuditRow>,
    /// The first row of this page.
    #[ts(type = "number")]
    pub offset: i64,
    /// The page size asked for (100 when 0 was sent).
    #[ts(type = "number")]
    pub limit: i64,
    /// All the entries of the filter, not this page's.
    #[ts(type = "number")]
    pub total_count: i64,
}

/// The values the filter boxes offer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AuditOptions {
    /// Every user who did something.
    pub users: Vec<AuditUser>,
    /// Every action that was done.
    pub actions: Vec<String>,
    /// Everything something was done to.
    pub entities: Vec<String>,
}
