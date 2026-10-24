//! The dashboard and the attention list (plan 19, PRD 9, FR-9.1, FR-7.44, NFR-SPD-2, NFR-SPD-22).
//! Both commands need the signed-in Owner (NFR-SEC-4) and only read (Code Standards B29).
//!
//! The work is in `mandi_reports::dashboard`; a command checks sign-in, tells it what the backup
//! service knows (the backup state is kept by that service, not in the database), and calls one
//! function.

use mandi_reports::dashboard::{self, AttentionList, BackupFacts, Dashboard};
use mandi_services::auth;

use super::reports_engine::engine_error;
use super::run;
use crate::backup_runtime::BackupRuntime;
use crate::error::UiError;

/// What the backup service says about the cloud copy. If the service is not running, the cloud
/// copy counts as off: no old-backup item is made up for a copy that cannot run.
fn backup_facts<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> BackupFacts {
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
        dashboard::get(ctx.db(), backup).map_err(engine_error)
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
        dashboard::attention(ctx.db(), backup).map_err(engine_error)
    })
    .await
}
