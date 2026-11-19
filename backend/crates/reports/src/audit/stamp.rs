//! The edit stamp and the edit history of an entry, read from the audit log (PRD FR-7.71, plan
//! 44). Read only (Code Standards B29): nothing here writes.
//!
//! - `edit_stamps` gives, for a list of ids of one kind of entry, when each was last edited, by
//!   whom, how many times and the last reason. An entry never edited has no stamp.
//! - `edit_history` gives one entry's versions, oldest first, each with its changed fields (the
//!   audit flatten code, so the paths and the values are the same as in the audit viewer).
//!
//! Example (PRD E2E-45): بکری 1 was edited at 4:35 pm on 10-10-26 (rate 600 → 610). The stamp of
//! its id has `local_date` 2026-10-10, `local_time` 16:35, `count` 1; its history has one version
//! with the change `lots[0].sales[0].seller_rate` 600 → 610.

use mandi_core::edit::{EditEntity, EditStamp};
use mandi_core::error::{ErrorCode, FieldError};
use mandi_db::Db;
use mandi_db::rusqlite::params;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::flatten;
use super::model::AuditChange;
use super::{local_parts, parse_json};
use crate::error::ReportsError;

/// The most ids one `audit_edit_stamp` call takes.
pub const MAX_STAMP_IDS: usize = 500;

/// Top-level keys of an edit's audit entry that are for the audit viewer and not edits the user
/// made: the numbers of the postings an edit wrote (every edit writes new ones), the totals of the
/// new posting and the cash question. The history shows the fields the user changed.
const AUDIT_ONLY: [&str; 13] = [
    "entry_id",
    "journal_entry_id",
    "current_entry_id",
    "reversal_entry_id",
    "version",
    "cash_after_paisa",
    "cash_below_zero_confirmed",
    "gross_paisa",
    "deductions_paisa",
    "net_paisa",
    "buyer_total_paisa",
    "shared_income_paisa",
    // The Owner's sign of the fix the edit was made in (plan 73): shown as the version's `sign`.
    "sign",
];

/// The input of `audit_edit_stamp`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct EditStampInput {
    /// The kind of entry.
    pub entity: EditEntity,
    /// The ids, at most [`MAX_STAMP_IDS`] (`ERR_OUT_OF_RANGE` on `ids` above it).
    #[ts(type = "number[]")]
    pub ids: Vec<i64>,
}

/// The stamp of one entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EditStampRow {
    /// The entry.
    #[ts(type = "number")]
    pub id: i64,
    /// Its last edit. Null when it was never edited.
    pub stamp: Option<EditStamp>,
}

/// The input of `audit_edit_history`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct EditHistoryInput {
    /// The kind of entry.
    pub entity: EditEntity,
    /// The entry. An entry never edited has an empty list.
    #[ts(type = "number")]
    pub id: i64,
}

/// One edit of an entry: when, by whom, why, and which fields changed from what to what.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EditVersion {
    /// The audit entry number.
    #[ts(type = "number")]
    pub audit_id: i64,
    /// When, UTC, as stored.
    pub at: String,
    /// The Pakistan date, `YYYY-MM-DD`.
    pub local_date: String,
    /// The Pakistan clock time, `HH:MM`.
    pub local_time: String,
    /// Who made it (display name).
    pub by: String,
    /// The audit action: `edit`.
    pub action: String,
    /// The reason typed. Null when none was given.
    pub reason: Option<String>,
    /// The Owner's sign given when the day was reopened to fix the entry (PRD FR-21.4, FR-7.71).
    /// Null for an edit of an open day.
    pub sign: Option<String>,
    /// The fields that changed, at most 200 (the audit flatten code).
    pub changes: Vec<AuditChange>,
    /// True when there were more changed fields than `changes` holds.
    pub changes_truncated: bool,
}

/// The history of one entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EditHistory {
    /// The kind of entry.
    pub entity: EditEntity,
    /// The entry.
    #[ts(type = "number")]
    pub id: i64,
    /// Its edits, oldest first. Empty when it was never edited.
    pub versions: Vec<EditVersion>,
}

/// The stamps of the ids asked for, in the order asked, one row for each. An entry that was never
/// edited has no stamp.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `ids` when there are more than [`MAX_STAMP_IDS`]; a database error.
pub fn edit_stamps(db: &Db, input: &EditStampInput) -> Result<Vec<EditStampRow>, ReportsError> {
    if input.ids.len() > MAX_STAMP_IDS {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "ids",
        )]));
    }
    db.read(|conn| {
        let found =
            mandi_db::edit_stamps::last_edits(conn, input.entity.audit_entity(), &input.ids)?;
        input
            .ids
            .iter()
            .map(|id| {
                let stamp = found
                    .get(id)
                    .map(|last| {
                        EditStamp::from_utc(
                            &last.at,
                            last.by.clone().unwrap_or_default(),
                            last.count,
                            last.reason.clone(),
                        )
                        .map(|stamp| stamp.with_sign(last.sign.clone()))
                        .ok_or_else(|| {
                            ReportsError::Data(format!("an audit time is not a time: {}", last.at))
                        })
                    })
                    .transpose()?;
                Ok(EditStampRow { id: *id, stamp })
            })
            .collect()
    })
}

/// The history of one entry: its edits, oldest first, each with the fields that changed.
///
/// # Errors
/// A database error.
pub fn edit_history(db: &Db, input: &EditHistoryInput) -> Result<EditHistory, ReportsError> {
    db.read(|conn| {
        let mut statement = conn.prepare(
            "SELECT a.id, a.at, a.action, a.reason, a.old_json, a.new_json,
                    COALESCE(NULLIF(u.display_name, ''), u.username)
             FROM audit_log a LEFT JOIN users u ON u.id = a.user_id
             WHERE a.entity = ?1 AND a.entity_id = ?2 AND a.action = 'edit' ORDER BY a.id",
        )?;
        let rows = statement
            .query_map(params![input.entity.audit_entity(), input.id], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut versions = Vec::with_capacity(rows.len());
        for (audit_id, at, action, reason, old_json, new_json, by) in rows {
            let (local_date, local_time) = local_parts(&at)?;
            let old = parse_json(old_json)?;
            let new = parse_json(new_json)?;
            let (mut changes, changes_truncated) = flatten::changes(old.as_ref(), new.as_ref());
            changes.retain(|c| !AUDIT_ONLY.contains(&c.path.as_str()));
            let sign = new
                .as_ref()
                .and_then(|value| value.get("sign"))
                .and_then(serde_json::Value::as_str)
                .filter(|sign| !sign.is_empty())
                .map(str::to_owned);
            versions.push(EditVersion {
                audit_id,
                at,
                local_date,
                local_time: local_time.chars().take(5).collect(),
                by: by.unwrap_or_default(),
                action,
                reason: reason.filter(|r| !r.is_empty()),
                sign,
                changes,
                changes_truncated,
            });
        }
        Ok(EditHistory {
            entity: input.entity,
            id: input.id,
            versions,
        })
    })
}
