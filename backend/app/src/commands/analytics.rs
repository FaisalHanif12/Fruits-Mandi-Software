//! The analytics commands (plan 21, step B4, PRD 18, FR-18.2, FR-18.4, FR-18.5). Every command
//! needs the signed-in Owner (NFR-SEC-4). The views only read (PRD FR-18.8, Code Standards
//! B29); the saved views are the one thing written, as a preference of the signed-in user.
//!
//! The work is in `mandi_reports::analytics`; a command checks sign-in, then calls one function.
//! The filters arrive as the text the user typed and are read by the backend; every problem is
//! returned together with its field (Code Standards 4.3).

use mandi_reports::ReportsError;
use mandi_reports::analytics::{
    AnalyticsCatalogEntry, AnalyticsLinesInput, AnalyticsLinesPage, AnalyticsPage,
    AnalyticsPrintData, AnalyticsPrintInput, AnalyticsRunInput, AnalyticsSavedView,
    AnalyticsViewsSaveInput, catalog, lines, print_data, run as run_view, views_get, views_save,
};
use mandi_services::ServiceError;
use mandi_services::auth;
use mandi_services::core::error::ErrorCode;

use super::run;
use crate::error::UiError;

/// A reports error as the error the screen sees. A problem with the filters keeps its field; the
/// rest keep their cause for the log.
fn analytics_error(error: ReportsError) -> ServiceError {
    match error {
        ReportsError::Rejected(items) => ServiceError::from(items),
        ReportsError::NotFound => ServiceError::reject_field(ErrorCode::ErrNotFound, "view"),
        ReportsError::Db(error) => ServiceError::Db(error),
        error @ (ReportsError::Money(_) | ReportsError::Words(_)) => {
            tracing::warn!(cause = %error, "an analytics total could not be made");
            ServiceError::reject(error.code())
        }
        other => ServiceError::Data(other.to_string()),
    }
}

/// The views of analytics: for each its letter, title ID and the filters it takes (`period`,
/// `item`, `seller`, `buyer`, `city`, `adda`, `side`, `compare`, and `group_by` for view D).
/// Read only.
#[tauri::command]
pub async fn analytics_catalog<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<AnalyticsCatalogEntry>, UiError> {
    run(app, "analytics_catalog", |ctx| {
        auth::require(ctx)?;
        Ok(catalog())
    })
    .await
}

/// One page of a view: typed columns and rows, the number of rows of the whole filter, the
/// totals over the whole filter (never a sum of the page), the filters as read, and for each row
/// the key that opens the lines behind it. `limit` 0 means 100; at most 1,000. Read only.
/// Errors: `ERR_OUT_OF_RANGE` on `offset`, `limit`, on `filters.group_by` (a view that does not
/// take it, or more than 36 columns) and on `filters.buyer.person_key`; `ERR_DATE_INVALID` on
/// `filters.from`, `filters.to`; `ERR_DATE_RANGE` on `filters.to`; `ERR_REQUIRED` on
/// `filters.from` and `filters.to` for a comparison; `ERR_NOT_FOUND` on a picked record.
#[tauri::command]
pub async fn analytics_run<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AnalyticsRunInput,
) -> Result<AnalyticsPage, UiError> {
    run(app, "analytics_run", move |ctx| {
        auth::require(ctx)?;
        run_view(ctx.db(), &input).map_err(analytics_error)
    })
    .await
}

/// The lines behind a row or a cell (FR-18.2): one line for each sale line, a reversal shown
/// negative so the lines add up to the number, each with the بکری it opens in `source`. Send the
/// `key` of the row as `row_key` and, for a cell of view D or E, the `drill` of the column as
/// `column_key`. Read only. Errors as `analytics_run`, and `ERR_OUT_OF_RANGE` on `column_key.*`
/// when the two keys name different records.
#[tauri::command]
pub async fn analytics_lines<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AnalyticsLinesInput,
) -> Result<AnalyticsLinesPage, UiError> {
    run(app, "analytics_lines", move |ctx| {
        auth::require(ctx)?;
        lines(ctx.db(), &input).map_err(analytics_error)
    })
    .await
}

/// The print data of a view with every row: the firm header and the filters in the print
/// language, the columns, the rows and the totals row formatted for print, the paper class,
/// `dir` and the message ID of the page footer. Read only. Errors: as `analytics_run`.
#[tauri::command]
pub async fn analytics_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AnalyticsPrintInput,
) -> Result<AnalyticsPrintData, UiError> {
    run(app, "analytics_print_data", move |ctx| {
        auth::require(ctx)?;
        print_data(ctx.db(), &input).map_err(analytics_error)
    })
    .await
}

/// The saved views of the signed-in user (FR-18.5), in the order he saved them. Read only.
#[tauri::command]
pub async fn analytics_views_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<AnalyticsSavedView>, UiError> {
    run(app, "analytics_views_get", |ctx| {
        let actor = auth::require(ctx)?;
        views_get(ctx.db(), actor.user_id).map_err(analytics_error)
    })
    .await
}

/// Replaces the saved views of the signed-in user with this list (at most 50) and returns it as
/// stored. Every name (1 to 60 letters) and every filter set is checked. Errors:
/// `ERR_OUT_OF_RANGE` on `views`, `ERR_REQUIRED`, `ERR_NAME_TOO_LONG` and `ERR_CONTROL_CHARS` on
/// `views[i].name`, and the errors of `analytics_run` on `views[i].filters.*`.
#[tauri::command]
pub async fn analytics_views_save<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: AnalyticsViewsSaveInput,
) -> Result<Vec<AnalyticsSavedView>, UiError> {
    run(app, "analytics_views_save", move |ctx| {
        let actor = auth::require(ctx)?;
        views_save(ctx.db(), actor.user_id, &input).map_err(analytics_error)
    })
    .await
}
