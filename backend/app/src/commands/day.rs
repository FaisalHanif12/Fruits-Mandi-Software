//! The open-day model (PRD FR-6.6, plan 04 Section 4). Day close comes in D2.

use serde::Serialize;
use ts_rs::TS;

use mandi_services::core::vouchers::{
    DayCloseInput, DayClosePreview, DayClosePreviewInput, DayReopenInput,
};
use mandi_services::{auth, day};

use super::run;
use crate::error::UiError;

/// Today and the closed-up-to date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DayStatus {
    /// Today's date on this computer, `YYYY-MM-DD`.
    pub today: String,
    /// The last closed day, `YYYY-MM-DD`. Null until a day is closed.
    pub closed_up_to: Option<String>,
    /// True when today is after the closed-up-to date, so entries can be posted.
    pub today_open: bool,
}

/// Today and the day state.
#[tauri::command]
pub async fn day_status<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<DayStatus, UiError> {
    run(app, "day_status", |ctx| {
        auth::require(ctx)?;
        let info = day::status(ctx)?;
        Ok(DayStatus {
            today: info.today,
            closed_up_to: info.closed_up_to,
            today_open: info.today_open,
        })
    })
    .await
}

fn status_of(info: day::DayStatusInfo) -> DayStatus {
    DayStatus {
        today: info.today,
        closed_up_to: info.closed_up_to,
        today_open: info.today_open,
    }
}

/// What closing the days up to a day would do (PRD FR-21.1): the days, the trial balance, the
/// cash in hand. Read only. Errors: `ERR_DATE_CLOSED` (the day is already closed),
/// `ERR_DATE_IN_FUTURE` (`up_to`).
#[tauri::command]
pub async fn day_close_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DayClosePreviewInput,
) -> Result<DayClosePreview, UiError> {
    run(app, "day_close_preview", move |ctx| {
        auth::require(ctx)?;
        day::close_preview(ctx, &input)
    })
    .await
}

/// Closes the days up to a day, oldest first (PRD FR-21.1). Errors as the preview, and
/// `ERR_TRIAL_NOT_BALANCED` when the trial balance does not balance. Returns the new status.
#[tauri::command]
pub async fn day_close<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DayCloseInput,
) -> Result<DayStatus, UiError> {
    run(app, "day_close", move |ctx| {
        let actor = auth::require(ctx)?;
        Ok(status_of(day::close(ctx, &actor, &input)?))
    })
    .await
}

/// Reopens a day and every later day (PRD FR-21.2). Errors: `ERR_REASON_REQUIRED` (`reason`),
/// `ERR_DAY_NOT_CLOSED` (`date`), `ERR_MONTH_CLOSED` (`date`). Returns the new status.
#[tauri::command]
pub async fn day_reopen<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DayReopenInput,
) -> Result<DayStatus, UiError> {
    run(app, "day_reopen", move |ctx| {
        let actor = auth::require(ctx)?;
        Ok(status_of(day::reopen(ctx, &actor, &input)?))
    })
    .await
}
