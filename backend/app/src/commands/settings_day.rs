//! The Day group of the settings and the number grouping (plan 22 early, PRD Section 21,
//! FR-19.7, FR-20.3). The Checks group (credit limit, rate check) is `settings_get_overrides` and
//! `settings_set_overrides`; the learned words list is `translit_pairs_list` and
//! `translit_pairs_remove` (FR-7.26).
//!
//! The Day group shares its closing time with the backup settings, so its commands answer with
//! the backup error shape (`BackupWireError`, the same fields as `UiError`).

use mandi_backup::day_settings::{self, DaySettings};
use mandi_services::auth;
use mandi_services::core::money::Grouping;
use serde::Deserialize;
use ts_rs::TS;

use super::backup::{BackupWireError, run_backup};
use super::run;
use crate::error::UiError;

/// The Day group: the closing time (`HH:MM`, Pakistan time, shared with the backup settings) and
/// whether the day closes by itself at that time (plan 19, `day_auto`). Read only.
#[tauri::command]
pub async fn settings_day_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<DaySettings, BackupWireError> {
    run_backup(app, "settings_day_get", |ctx| {
        auth::require(ctx)?;
        Ok(ctx.db().read(day_settings::load)?)
    })
    .await
}

/// Saves the Day group with an audit row of the old and the new value. Errors:
/// `ERR_OUT_OF_RANGE` on `closing_time` (values `min` `00:00` and `max` `23:59`).
#[tauri::command]
pub async fn settings_day_set<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: DaySettings,
) -> Result<DaySettings, BackupWireError> {
    run_backup(app, "settings_day_set", move |ctx| {
        let actor = auth::require(ctx)?;
        Ok(day_settings::save(ctx.db(), Some(actor.user_id), &input)?)
    })
    .await
}

/// The input of `settings_set_number_grouping`.
#[derive(Debug, Clone, Copy, Deserialize, TS)]
pub struct SetNumberGroupingInput {
    /// `international` (1,316,040) or `lakh` (13,16,040).
    pub grouping: Grouping,
}

/// Saves how big numbers are grouped on prints and report print data (PRD FR-19.7), with an
/// audit row of the old and the new value, and answers with the saved value.
#[tauri::command]
pub async fn settings_set_number_grouping<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SetNumberGroupingInput,
) -> Result<Grouping, UiError> {
    run(app, "settings_set_number_grouping", move |ctx| {
        let actor = auth::require(ctx)?;
        mandi_services::settings_ext::set_number_grouping(ctx, &actor, input.grouping)
    })
    .await
}
