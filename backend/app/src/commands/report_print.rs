//! The print commands of a report and of an analytics view (plan d5-late, after 17e; PRD FR-19.3,
//! FR-18.4). Each is thin: the work is in `crate::print_report`, which checks the request, builds
//! the print data once and runs the shared print job.

use mandi_services::core::printing::PrintOutcome;

use crate::error::UiError;
use crate::print_report::{
    PrintAnalyticsViewInput, PrintBikriListInput, PrintReportInput, PrintStatementInput,
    run_analytics, run_bikri_list, run_report, run_statement,
};

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

/// Prints پچھلی بکری (today's section and a window of days, with the filters or the switch "show
/// every بکری") on the printer of Settings, or saves it as a PDF named
/// `bikri-list-<YYYYMMDD>-<language>.pdf` (PRD FR-12.46, plan 72 P8). Like `analytics_print_view`.
/// The data is `bikri_list_print_data`, the header and footer `reports_print_header`. Errors: as
/// `bikri_list_print_data`, the copies codes on `copies`, `ERR_PRINT_WINDOWS_ONLY`,
/// `ERR_PRINTER_NOT_FOUND`, `ERR_PRINT_FAILED`.
#[tauri::command]
pub async fn bikri_list_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintBikriListInput,
) -> Result<PrintOutcome, UiError> {
    run_bikri_list(app, input).await
}

/// Prints one party's statement for a period on the printer of Settings, or saves it as a PDF named
/// `party-statement-<code>-<YYYYMMDD>-<language>.pdf` (PRD FR-12.47, plan 74 T1). Like
/// `bikri_list_print`. The data is `party_ledgers_statement_print_data`, the header and footer
/// `reports_print_header`. Errors: as `party_ledgers_statement_print_data`, the copies codes on
/// `copies`, `ERR_PRINT_WINDOWS_ONLY`, `ERR_PRINTER_NOT_FOUND`, `ERR_PRINT_FAILED`.
#[tauri::command]
pub async fn party_ledgers_statement_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintStatementInput,
) -> Result<PrintOutcome, UiError> {
    run_statement(app, input).await
}
