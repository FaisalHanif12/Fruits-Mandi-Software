//! The open-day model (PRD FR-6.6, plan 04 Section 4). Day close comes in D2.

use serde::Serialize;
use ts_rs::TS;

use super::not_implemented;
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

/// Today and the day state. Any signed-in user.
#[tauri::command]
pub async fn day_status<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
) -> Result<DayStatus, UiError> {
    not_implemented()
}
