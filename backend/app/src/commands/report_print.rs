//! The print commands of a report and of an analytics view (plan d5-late, after 17e; PRD FR-19.3,
//! FR-18.4). Each is thin: the work is in `crate::print_report`, which checks the request, builds
//! the print data once and runs the shared print job.

use mandi_services::core::printing::PrintOutcome;

use crate::error::UiError;
use crate::print_report::{
    PrintAdhariInput, PrintAnalyticsViewInput, PrintBikriListInput, PrintBuyerDuesInput,
    PrintCollectionSheetInput, PrintCommissionInput, PrintPartnerMonthInput, PrintReportInput,
    PrintSlipsInput, PrintStatementInput, PrintStatementRowsInput, run_adhari, run_analytics,
    run_bikri_list, run_buyer_dues, run_collection_sheet, run_commission, run_parchi,
    run_partner_month, run_report, run_sale, run_seller_bills, run_statement, run_statement_rows,
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

/// Prints the buyer slips (the parchi) of a بکری, a save or a day on the printer of Settings, or
/// saves them as a PDF named `parchi-<YYYYMMDD>-<language>.pdf` (plan 78 A1). The data is
/// `reports_buyer_parchi_print_data`. Errors: as that command, the copies codes on `copies`,
/// `ERR_PRINT_WINDOWS_ONLY`, `ERR_PRINTER_NOT_FOUND`, `ERR_PRINT_FAILED`.
#[tauri::command]
pub async fn reports_buyer_parchi_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintSlipsInput,
) -> Result<PrintOutcome, UiError> {
    run_parchi(app, input).await
}

/// Prints the seller bills of a بکری, a save or a day, or saves them as `seller-bills-...pdf` (plan 78
/// A1, A3). The data is `reports_seller_bills_print_data`. Errors as `reports_buyer_parchi_print`.
#[tauri::command]
pub async fn reports_seller_bills_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintSlipsInput,
) -> Result<PrintOutcome, UiError> {
    run_seller_bills(app, input).await
}

/// Prints the whole sale print (پوری بکری) of a بکری, a save or a day, or saves it as `sale-...pdf`
/// (plan 78 A4). The data is `reports_sale_print_data`. Errors as `reports_buyer_parchi_print`.
#[tauri::command]
pub async fn reports_sale_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintSlipsInput,
) -> Result<PrintOutcome, UiError> {
    run_sale(app, input).await
}

/// Prints the ادھاری تفصیل of a day, or saves it as `adhari-...pdf` (plan 78 A5). The data is
/// `reports_adhari_sheet_print_data`. Errors as `reports_buyer_parchi_print`.
#[tauri::command]
pub async fn reports_adhari_sheet_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintAdhariInput,
) -> Result<PrintOutcome, UiError> {
    run_adhari(app, input).await
}

/// Prints the اگراہی sheet of a day (the R5 screen), or saves it as `collection-sheet-...pdf` (plan 76
/// point 2). The data is `reports_collection_sheet_print_data`. Errors as `reports_buyer_parchi_print`.
#[tauri::command]
pub async fn reports_collection_sheet_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintCollectionSheetInput,
) -> Result<PrintOutcome, UiError> {
    run_collection_sheet(app, input).await
}

/// Prints one party's statement with a row for each entry for a period on the printer of Settings, or
/// saves it as `statement-rows-<YYYYMMDD>-<language>.pdf` (plan 78 A7). The data is
/// `party_ledgers_statement_rows_print_data`. Errors as `reports_buyer_parchi_print`.
#[tauri::command]
pub async fn party_ledgers_statement_rows_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintStatementRowsInput,
) -> Result<PrintOutcome, UiError> {
    run_statement_rows(app, input).await
}

/// Prints the buyer dues page (every buyer of the filter, with the total), or saves it as
/// `buyer-dues-<YYYYMMDD>-<language>.pdf` (plan 76 point 9). The data is
/// `dashboard_buyer_dues_print_data`. Errors as `reports_buyer_parchi_print`.
#[tauri::command]
pub async fn dashboard_buyer_dues_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintBuyerDuesInput,
) -> Result<PrintOutcome, UiError> {
    run_buyer_dues(app, input).await
}

/// Prints the commission page of a period, or saves it as `commission-<YYYYMMDD>-<language>.pdf`
/// (plan 76 point 7, plan 77). The data is `dashboard_commission_print_data`; the partner columns
/// are left out while the partner section is locked. Errors as `reports_buyer_parchi_print`.
#[tauri::command]
pub async fn dashboard_commission_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintCommissionInput,
) -> Result<PrintOutcome, UiError> {
    run_commission(app, input).await
}

/// Prints a partner's month statement, or saves it as `partner-month-<YYYYMMDD>-<language>.pdf`
/// (plan 76 point 11). The data is `partners_statement_month_print_data`. Behind the partner code
/// (`ERR_PARTNER_LOCKED`). Errors as `reports_buyer_parchi_print`.
#[tauri::command]
pub async fn partners_statement_month_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintPartnerMonthInput,
) -> Result<PrintOutcome, UiError> {
    run_partner_month(app, input).await
}
