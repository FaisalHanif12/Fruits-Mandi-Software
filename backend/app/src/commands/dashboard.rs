//! The dashboard and the attention list (plan 19, PRD 9, FR-9.1, FR-7.44, NFR-SPD-2, NFR-SPD-22).
//! Both commands need the signed-in Owner (NFR-SEC-4) and only read (Code Standards B29).
//!
//! The work is in `mandi_reports::dashboard`; a command checks sign-in, tells it what the backup
//! service knows (the backup state is kept by that service, not in the database), and calls one
//! function.

use mandi_reports::dashboard::{
    self, AttentionList, BackupFacts, BuyerDuesInput, BuyerDuesPage, BuyerDuesPrintInput,
    CommissionDay, CommissionDayInput, CommissionMonth, CommissionMonthInput, CommissionPrintInput,
    Dashboard,
};
use mandi_reports::reports::TablePrintData;
use mandi_services::auth;
use mandi_services::partner_code;

use super::reports_engine::engine_error;
use super::run;
use crate::backup_runtime::BackupRuntime;
use crate::error::UiError;

/// What the backup service says about the cloud copy. If the service is not running, the cloud
/// copy counts as off: no old-backup item is made up for a copy that cannot run.
pub(super) fn backup_facts<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> BackupFacts {
    let status = BackupRuntime::of(app)
        .ok()
        .and_then(|runtime| runtime.service().status().ok());
    match status {
        Some(status) => BackupFacts {
            cloud_on: status.cloud_enabled,
            cloud_age_hours: status.cloud_age_hours,
        },
        None => BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    }
}

/// Everything the dashboard shows: the tiles, the partner strip, today's بکری, the oldest buyer
/// dues, the two charts and the attention list (آج کے کام). Read only. The last backup and the
/// daily health result come from `backup_status` and `health_last`.
#[tauri::command]
pub async fn dashboard_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Dashboard, UiError> {
    let backup = backup_facts(&app);
    run(app, "dashboard_get", move |ctx| {
        auth::require(ctx)?;
        let mut shown = dashboard::get(ctx.db(), backup).map_err(engine_error)?;
        if partner_code::is_locked(ctx)? {
            shown.hide_partners();
        }
        Ok(shown)
    })
    .await
}

/// The attention list and the number on the bell (PRD FR-7.44). Read only. The bell asks again
/// after every save.
#[tauri::command]
pub async fn assist_attention<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<AttentionList, UiError> {
    let backup = backup_facts(&app);
    run(app, "assist_attention", move |ctx| {
        auth::require(ctx)?;
        let mut shown = dashboard::attention(ctx.db(), backup).map_err(engine_error)?;
        if partner_code::is_locked(ctx)? {
            shown.hide_partners();
        }
        Ok(shown)
    })
    .await
}

/// One month of the commission page: a row for each day with a figure and the month totals
/// (plan 76 point 7). The partner columns are hidden while the partner section is locked (PRD
/// FR-17.11). Read only.
#[tauri::command]
pub async fn dashboard_commission_month<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CommissionMonthInput,
) -> Result<CommissionMonth, UiError> {
    run(app, "dashboard_commission_month", move |ctx| {
        auth::require(ctx)?;
        let mut shown = dashboard::commission_month(ctx.db(), &input).map_err(engine_error)?;
        if partner_code::is_locked(ctx)? {
            shown.hide_partners();
        }
        Ok(shown)
    })
    .await
}

/// The postings behind one day's commission (plan 76 point 7). Read only.
#[tauri::command]
pub async fn dashboard_commission_day<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CommissionDayInput,
) -> Result<CommissionDay, UiError> {
    run(app, "dashboard_commission_day", move |ctx| {
        auth::require(ctx)?;
        let mut shown = dashboard::commission_day(ctx.db(), &input).map_err(engine_error)?;
        if partner_code::is_locked(ctx)? {
            shown.hide_partners();
        }
        Ok(shown)
    })
    .await
}

/// The print of one commission month, with every row, in the print language (plan 76 point 7).
#[tauri::command]
pub async fn dashboard_commission_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: CommissionPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "dashboard_commission_print_data", move |ctx| {
        auth::require(ctx)?;
        let show_partners = !partner_code::is_locked(ctx)?;
        dashboard::commission_month_print_data(ctx.db(), &input, show_partners)
            .map_err(engine_error)
    })
    .await
}

/// A page of the buyers who owe, with the total of all that match (plan 76 point 9). Read only.
#[tauri::command]
pub async fn dashboard_buyer_dues<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BuyerDuesInput,
) -> Result<BuyerDuesPage, UiError> {
    run(app, "dashboard_buyer_dues", move |ctx| {
        auth::require(ctx)?;
        dashboard::buyer_dues(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print of the buyer dues, every buyer of the filter (plan 76 point 9).
#[tauri::command]
pub async fn dashboard_buyer_dues_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BuyerDuesPrintInput,
) -> Result<TablePrintData, UiError> {
    run(app, "dashboard_buyer_dues_print_data", move |ctx| {
        auth::require(ctx)?;
        dashboard::buyer_dues_print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}
