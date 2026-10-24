//! The Excel export commands (plan 22 round 10, PRD FR-18.4, FR-19.3, NFR-SEC-13, NFR-SPD-17).
//! Every command needs the signed-in Owner (NFR-SEC-4). They only read the books; the one thing
//! written is the file the Owner chose.
//!
//! The work is in `mandi_reports::export`. It reads every row of the filter (never a page) and
//! writes a sheet in the language asked for. It runs on a worker thread, so the window never
//! waits, and it tells the screen how far it has come with the event `export-progress` (one
//! event for the reading, one for every 500 rows written, one at the end). The screen picks its
//! own `job_id`, which comes back in every event, so two exports never mix.
//!
//! Example: `reports_export_excel` with `{ job_id: "j1", id: "R18", language: "en", path:
//! "C:\\Users\\Owner\\trial.xlsx" }` answers `{ path, size_bytes, rows }` after the events
//! `reading`, `writing` (0 of 17), `writing` (17 of 17) and `done`.

use mandi_reports::export::{
    AnalyticsExportInput, ExcelExported, ExportProgress, ExportStage, ReportExportInput,
    export_analytics, export_report, export_report_page,
};
use mandi_reports::reports::ReportId;
use mandi_services::ServiceError;
use mandi_services::auth;
use mandi_services::core::error::ErrorCode;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use ts_rs::TS;

use super::reports_engine::{engine_error, statement_all};
use super::run;
use crate::error::UiError;

/// The event the screen listens to.
pub const PROGRESS_EVENT: &str = "export-progress";

/// The longest job id the screen may send.
const MAX_JOB_ID: usize = 64;

/// The input of `reports_export_excel`: the export plus the screen's job id.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ReportsExportRequest {
    /// An id the screen picks (letters, digits, `-` and `_`, at most 64) to match the events.
    pub job_id: String,
    /// The report, filters, language and path.
    #[serde(flatten)]
    #[ts(flatten)]
    pub export: ReportExportInput,
}

/// The input of `analytics_export_excel`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AnalyticsExportRequest {
    /// An id the screen picks to match the events.
    pub job_id: String,
    /// The view, filters, language and path.
    #[serde(flatten)]
    #[ts(flatten)]
    pub export: AnalyticsExportInput,
}

/// The payload of the event `export-progress`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ExportProgressEvent {
    /// The job id the screen sent.
    pub job_id: String,
    /// The step: `reading`, `writing` or `done`.
    pub stage: ExportStage,
    /// Rows written so far.
    #[ts(type = "number")]
    pub done: u64,
    /// Rows to write. 0 while the rows are still being read.
    #[ts(type = "number")]
    pub total: u64,
}

fn check_job_id(job_id: &str) -> Result<(), ServiceError> {
    let good = !job_id.is_empty()
        && job_id.len() <= MAX_JOB_ID
        && job_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if good {
        Ok(())
    } else {
        Err(ServiceError::reject_field(
            ErrorCode::ErrOutOfRange,
            "job_id",
        ))
    }
}

/// Sends one progress event. A screen that is not listening is not a problem.
fn emit<R: tauri::Runtime>(app: &tauri::AppHandle<R>, job_id: &str, progress: ExportProgress) {
    let event = ExportProgressEvent {
        job_id: job_id.to_owned(),
        stage: progress.stage,
        done: progress.done,
        total: progress.total,
    };
    if let Err(error) = app.emit(PROGRESS_EVENT, event) {
        tracing::debug!("export progress event: {error}");
    }
}

/// Saves a report as an `.xlsx` file with every row of the filter (PRD FR-19.3).
#[tauri::command]
pub async fn reports_export_excel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ReportsExportRequest,
) -> Result<ExcelExported, UiError> {
    let handle = app.clone();
    run(app, "reports_export_excel", move |ctx| {
        auth::require(ctx)?;
        check_job_id(&input.job_id)?;
        let job = input.job_id.clone();
        // R31, the partner statement, is put together by the services (see `reports_engine`).
        if input.export.id == ReportId::R31 {
            let page = statement_all(ctx, &input.export.filters)?;
            return export_report_page(ctx.db(), &page, &input.export, &mut |p| {
                emit(&handle, &job, p);
            })
            .map_err(engine_error);
        }
        export_report(ctx.db(), &input.export, &mut |p| emit(&handle, &job, p))
            .map_err(engine_error)
    })
    .await
}

/// Saves an analytics view as an `.xlsx` file with every row (PRD FR-18.4).
#[tauri::command]
pub async fn analytics_export_excel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AnalyticsExportRequest,
) -> Result<ExcelExported, UiError> {
    let handle = app.clone();
    run(app, "analytics_export_excel", move |ctx| {
        auth::require(ctx)?;
        check_job_id(&input.job_id)?;
        let job = input.job_id.clone();
        export_analytics(ctx.db(), &input.export, &mut |p| emit(&handle, &job, p))
            .map_err(engine_error)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_job_id_is_short_and_plain() {
        assert!(check_job_id("export-1_a").is_ok());
        for bad in ["", "a b", "x/y", &"a".repeat(65), "اردو"] {
            assert!(check_job_id(bad).is_err(), "{bad}");
        }
    }
}
