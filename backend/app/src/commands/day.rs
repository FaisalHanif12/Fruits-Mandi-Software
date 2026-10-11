//! The open-day model (PRD FR-6.6, plan 04 Section 4). Day close comes in D2.

use serde::Serialize;
use ts_rs::TS;

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
