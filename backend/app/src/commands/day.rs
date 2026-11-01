//! The open-day model (PRD FR-6.6, plan 04 Section 4). Day close comes in D2.

use serde::Serialize;
use ts_rs::TS;

use mandi_services::core::edit::{DayFixCloseInput, DayFixClosed, DayFixOpenInput, DayFixOpened};
use mandi_services::core::vouchers::{
    DayCloseInput, DayClosePreview, DayClosePreviewInput, DayReopenInput,
};
use mandi_services::partner_code;
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
        let mut shown = day::close_preview(ctx, &input)?;
        if partner_code::is_locked(ctx)? {
            shown.summary.hide_partners();
        }
        Ok(shown)
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

/// Reopens the date of an entry to fix, and every later closed date, and gives a token to close
/// the same days again (PRD FR-21.4, plan 44). The reason goes to the audit log. Errors:
/// `ERR_REASON_REQUIRED` on `reason`; the date codes on `date`; `ERR_DAY_NOT_CLOSED` on `date`;
/// `ERR_MONTH_CLOSED` on `date`. Until plan 44 T2c: `ERR_NOT_IMPLEMENTED`.
#[tauri::command]
pub async fn day_fix_open<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DayFixOpenInput,
) -> Result<DayFixOpened, UiError> {
    run(app, "day_fix_open", move |ctx| {
        let actor = auth::require(ctx)?;
        day::fix_open(ctx, &actor, &input)
    })
    .await
}

/// Closes the days `day_fix_open` reopened, oldest first, each with the checks of PRD FR-21.1. A
/// day whose checks fail stays open and is listed with the reason. Errors:
/// `ERR_FIX_TOKEN_INVALID` on `token`. Until plan 44 T2c: `ERR_NOT_IMPLEMENTED`.
#[tauri::command]
pub async fn day_fix_close<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DayFixCloseInput,
) -> Result<DayFixClosed, UiError> {
    run(app, "day_fix_close", move |ctx| {
        let actor = auth::require(ctx)?;
        day::fix_close(ctx, &actor, &input)
    })
    .await
}
