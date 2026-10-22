//! The report engine commands (plan 20, step B2, PRD 19.1 to 19.5, 19.10, 19.11, NFR-SEC-13).
//! Every command needs the signed-in Owner. They only read (PRD FR-18.8, Code Standards B29).
//!
//! The work is in `mandi_reports::reports`; a command checks sign-in, then calls one function.
//! The filters arrive as the text the user typed and are read by the backend; every problem is
//! returned together with its field (Code Standards 4.3).

use mandi_reports::ReportsError;
use mandi_reports::reports::{
    CatalogEntry, ReportPage, ReportPrintData, ReportPrintInput, ReportRunInput, catalog,
    print_data, run as run_report,
};
use mandi_services::ServiceError;
use mandi_services::auth;
use mandi_services::core::error::ErrorCode;

use super::run;
use crate::error::UiError;

/// A reports error as the error the screen sees. A problem with the filters keeps its field; an
/// unknown report is `ERR_NOT_FOUND` on `id`; the rest keep their cause for the log.
fn engine_error(error: ReportsError) -> ServiceError {
    match error {
        ReportsError::Rejected(items) => ServiceError::from(items),
        ReportsError::NotFound => ServiceError::reject_field(ErrorCode::ErrNotFound, "id"),
        ReportsError::Db(error) => ServiceError::Db(error),
        error @ (ReportsError::Money(_) | ReportsError::Words(_)) => {
            tracing::warn!(cause = %error, "a report total could not be made");
            ServiceError::reject(error.code())
        }
        other => ServiceError::Data(other.to_string()),
    }
}

/// The catalogue of reports: for each its PRD ID, group, title ID and the filters it takes
/// (`date_range`, `as_of`, `party`, `seller`, `zamindar`, `item`, `city`, `adda`, `detail`), and
/// whether the party must be filled. Read only.
#[tauri::command]
pub async fn reports_catalog<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<CatalogEntry>, UiError> {
    run(app, "reports_catalog", |ctx| {
        auth::require(ctx)?;
        Ok(catalog())
    })
    .await
}

/// One page of a report: typed columns and rows, the number of rows of the whole filter, the
/// totals over the whole filter (never a sum of the page), the filters as read, and for each row
/// the record it opens. `limit` 0 means 100; at most 1,000. Read only. Errors: `ERR_NOT_FOUND`
/// on `id`; `ERR_REQUIRED` on `filters.party` (R13, R14); `ERR_OUT_OF_RANGE` on a filter the
/// report does not take, on `offset` and on `limit`; `ERR_DATE_INVALID` on `filters.from`,
/// `filters.to` or `filters.as_of`; `ERR_DATE_RANGE` on `filters.to`; `ERR_NOT_FOUND` on a
/// picked record (`filters.party.account_id`, `filters.item_id`, ...).
#[tauri::command]
pub async fn reports_run<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ReportRunInput,
) -> Result<ReportPage, UiError> {
    run(app, "reports_run", move |ctx| {
        auth::require(ctx)?;
        run_report(ctx.db(), &input).map_err(engine_error)
    })
    .await
}

/// The print data of a report with every row: the firm header and the filters in the print
/// language, the columns, the rows and the totals row formatted for print, the closing balance
/// in words for R13 and R14, the paper class, `dir` and the message ID of the page footer. Read
/// only. Errors: as `reports_run`.
#[tauri::command]
pub async fn reports_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: ReportPrintInput,
) -> Result<ReportPrintData, UiError> {
    run(app, "reports_print_data", move |ctx| {
        auth::require(ctx)?;
        print_data(ctx.db(), &input).map_err(engine_error)
    })
    .await
}
