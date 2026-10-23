//! The audit viewer (plan 22 early, PRD FR-21.3, NFR-SEC-5, Code Standards B31). Both commands
//! need the signed-in Owner (NFR-SEC-4) and only read: the log cannot be changed or deleted from
//! the app.

use mandi_reports::audit::{AuditListInput, AuditOptions, AuditPage, list, options};
use mandi_services::auth;

use super::reports_engine::engine_error;
use super::run;
use crate::error::UiError;

/// One page of the audit log, newest first, with the count of the whole filter. Each entry has
/// who, when (UTC and Pakistan time), what was done, to what, the reason, the old and new value
/// as stored (secrets shown as `[hidden]`) and the list of the fields that changed. `limit` 0
/// means 100; at most 1,000. Read only. Errors: `ERR_NOT_FOUND` on `filters.user_id`,
/// `ERR_REQUIRED` on `filters.entity` (a record number without its kind), `ERR_DATE_INVALID` on
/// `filters.from` or `filters.to`, `ERR_DATE_RANGE` on `filters.to`, `ERR_CONTROL_CHARS` and
/// `ERR_TEXT_TOO_LONG` on `filters.action`, `filters.entity` and `filters.text`,
/// `ERR_OUT_OF_RANGE` on `offset` and `limit`.
#[tauri::command]
pub async fn audit_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AuditListInput,
) -> Result<AuditPage, UiError> {
    run(app, "audit_list", move |ctx| {
        auth::require(ctx)?;
        list(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The values the filter boxes of the viewer offer: the users, the actions and the kinds of
/// record found in the log. Read only.
#[tauri::command]
pub async fn audit_options<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<AuditOptions, UiError> {
    run(app, "audit_options", |ctx| {
        auth::require(ctx)?;
        options(ctx.db()).map_err(engine_error)
    })
    .await
}
