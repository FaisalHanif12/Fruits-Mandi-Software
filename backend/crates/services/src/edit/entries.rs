//! The pieces every new edit shares (plan 44 T2a part 2, PRD FR-7.70): what an edit tells its caller,
//! the refusal "this edit is not allowed" with its rule, the review (a rehearsal of the real edit)
//! and the save (the real edit, one transaction).
//!
//! The edit of a trip or garden entry is a reversal of the old posting, a rewrite of the entry's own
//! rows inside its edit window (`mandi_db::edit_windows`, migration 12), and a new posting on the same
//! day. The window is opened and closed inside the one transaction, so a failed save leaves nothing.

use mandi_db::edit_stamps;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::edit::{EditEntity, EditPreview, EditStamp, EntryEdited};
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use serde_json::Value;

use super::{Scope, diff, preview_of, rehearse};
use crate::context::Context;
use crate::error::ServiceError;

/// What the body of an edit tells its caller.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Edited {
    /// The entry.
    pub id: i64,
    /// Its number, if it has one.
    pub number: Option<i64>,
    /// Its new version.
    pub version: i64,
    /// The posting that replaced the old one. `None` when nothing posts.
    pub entry_id: Option<i64>,
    /// The reversal of the old posting. `None` when the old entry had no posting.
    pub reversal_entry_id: Option<i64>,
}

/// The cash question of an edit (PRD FR-6.7) is about its NET change: `cash_before` is cash in hand
/// before the edit, `old_change` what the old posting did to cash, `new_change` what the new entry
/// does. An edit that makes cash go up never asks, whatever cash is.
///
/// Example: cash is -15,000, a shop part of 30,000 is cut to 20,000: net +10,000, no question.
///
/// # Errors
/// `ERR_OVERFLOW`.
pub(crate) fn net_cash_check(
    cash_before: mandi_domain::core::money::Paisa,
    old_change: mandi_domain::core::money::Paisa,
    new_change: mandi_domain::core::money::Paisa,
) -> Result<mandi_domain::core::cash::CashCheck, ServiceError> {
    let net = new_change
        .checked_sub(old_change)
        .map_err(crate::error::money_error)?;
    mandi_domain::cash::check(cash_before, net).map_err(crate::error::money_error)
}

/// `ERR_EDIT_NOT_ALLOWED` at `field`, with the `rule` that forbids it (`goods_moved`,
/// `carries_cost`, ...).
pub(crate) fn not_allowed(field: &str, rule: &str) -> ServiceError {
    ServiceError::from(
        FieldError::new(ErrorCode::ErrEditNotAllowed, field)
            .with("rule", ErrorValue::Text(rule.to_owned())),
    )
}

/// The review of a new edit: `body` is the real edit; it runs in a transaction that is rolled back,
/// and the change list is the difference of the entry's row before and after (`row` reads it).
///
/// # Errors
/// `ServiceError::Db` and `ServiceError::Data`. The mistakes of the edit come back inside the answer.
pub(crate) fn review(
    ctx: &Context,
    scope: Scope<'_>,
    entry_date: BusinessDate,
    row: impl Fn(&Connection) -> Result<Value, ServiceError>,
    body: impl FnOnce(&Connection) -> Result<Edited, ServiceError>,
) -> Result<EditPreview, ServiceError> {
    let outcome = rehearse(ctx, scope, row, body);
    preview_of(ctx, entry_date, outcome, diff::changes_of_rows)
}

/// The save of a new edit: the mistakes the review found are returned as they are (nothing is
/// written), else `body` runs in one transaction and the answer carries the review and the stamp.
///
/// # Errors
/// What `body` returns; the mistakes of `preview`.
pub(crate) fn save(
    ctx: &Context,
    entity: EditEntity,
    preview: EditPreview,
    body: impl FnOnce(&Connection) -> Result<Edited, ServiceError>,
) -> Result<EntryEdited, ServiceError> {
    if !preview.errors.is_empty() {
        return Err(ServiceError::from(preview.errors));
    }
    let edited = ctx.db.write(|tx| body(tx))?;
    finish(ctx, entity, preview, edited)
}

/// The answer of a save that has committed: the review of the preview, and the stamp the entry now
/// carries.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` when the edit left no audit entry.
pub(crate) fn finish(
    ctx: &Context,
    entity: EditEntity,
    preview: EditPreview,
    edited: Edited,
) -> Result<EntryEdited, ServiceError> {
    let stamp = stamp_of(ctx, entity, edited.id)?;
    Ok(EntryEdited {
        id: edited.id,
        number: edited.number,
        version: edited.version,
        entry_id: edited.entry_id,
        reversal_entry_id: edited.reversal_entry_id,
        review: preview.review,
        edited: stamp,
    })
}

/// The stamp the entry carries after its edit.
fn stamp_of(ctx: &Context, entity: EditEntity, id: i64) -> Result<EditStamp, ServiceError> {
    let last = ctx
        .db
        .read(|conn| {
            edit_stamps::last_edits(conn, entity.audit_entity(), &[id]).map_err(ServiceError::from)
        })?
        .remove(&id)
        .ok_or_else(|| ServiceError::Data("an edit left no audit entry".to_owned()))?;
    EditStamp::from_utc(
        &last.at,
        last.by.unwrap_or_default(),
        last.count,
        last.reason,
    )
    .ok_or_else(|| ServiceError::Data(format!("an audit time is not a time: {}", last.at)))
}
