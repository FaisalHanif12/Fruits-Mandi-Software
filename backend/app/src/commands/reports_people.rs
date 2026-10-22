//! Global search for one-time people (plan 20, step B3, PRD FR-5.4). Needs the signed-in Owner
//! (NFR-SEC-4). It only reads (Code Standards B29).
//!
//! The accounts, the بکری (by number or truck) and the vouchers are found by `search_global`; this
//! command finds the people of the پختہ کھاتہ accounts, so `Ctrl+K` can list them in the same box.

use mandi_reports::ReportsError;
use mandi_reports::people::{FindPeopleInput, PersonHit, find_people as find};
use mandi_services::ServiceError;
use mandi_services::auth;

use super::run;
use crate::error::UiError;

/// One-time people (پختہ کھاتہ) that match the typed text in either name, a close English
/// spelling or the phone, newest first, with the accounts each has lines on and his person key
/// (send both as `party` to open his ledger R13 or his register R9). The text is never converted.
/// `limit` 0 means 10, at most 50. Read only. Errors: `ERR_OUT_OF_RANGE` on `text` and `limit`.
#[tauri::command]
pub async fn reports_find_people<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FindPeopleInput,
) -> Result<Vec<PersonHit>, UiError> {
    run(app, "reports_find_people", move |ctx| {
        auth::require(ctx)?;
        find(ctx.db(), &input).map_err(|error| match error {
            ReportsError::Rejected(items) => ServiceError::from(items),
            ReportsError::Db(error) => ServiceError::Db(error),
            other => ServiceError::Data(other.to_string()),
        })
    })
    .await
}
