//! The print commands of a report and of an analytics view (plan d5-late, after 17e; PRD FR-19.3,
//! FR-18.4). Each is thin: the work is in `crate::print_report`, which checks the request, builds
//! the print data once and runs the shared print job.

use mandi_services::core::printing::PrintOutcome;

use crate::error::UiError;
use crate::print_report::{PrintAnalyticsViewInput, PrintReportInput, run_analytics, run_report};

/// Prints a report on the printer of Settings, or saves it as a PDF named
/// `report-<id>-<YYYYMMDD>-<language>.pdf` (`report-r5-slips-...` for the slips of R5). With
/// `slips` true on R5 it prints one slip for each credit buyer. The paper and the copies come from
/// the request (A4 and the copies of Settings when left out). The header and footer are the shared
/// ones (`reports_print_header`). A report has no print count; the print is written to the audit
/// log. Printing needs Windows. Errors: as `reports_run` on the filters, `ERR_OUT_OF_RANGE` on
/// `slips`, the copies codes on `copies`, `ERR_PRINT_WINDOWS_ONLY`, `ERR_PRINTER_NOT_FOUND`,
/// `ERR_PRINT_FAILED`.
#[tauri::command]
pub async fn reports_print_report<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintReportInput,
) -> Result<PrintOutcome, UiError> {
    run_report(app, input).await
}

/// Prints an analytics view on the printer of Settings, or saves it as a PDF named
/// `analytics-<view>-<YYYYMMDD>-<language>.pdf`. Like `reports_print_report`. Errors: as
/// `analytics_run` on the filters, the copies codes on `copies`, `ERR_PRINT_WINDOWS_ONLY`,
/// `ERR_PRINTER_NOT_FOUND`, `ERR_PRINT_FAILED`.
#[tauri::command]
pub async fn analytics_print_view<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintAnalyticsViewInput,
) -> Result<PrintOutcome, UiError> {
    run_analytics(app, input).await
}
