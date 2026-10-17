//! Prints of a saved بکری: the print data, the print itself and the printer list (plan 14,
//! PRD FR-12.12, FR-7.36, FR-19.3 to FR-19.7). Every command needs the signed-in Owner
//! (NFR-SEC-13). The print data only reads (Code Standards B29).

use mandi_reports::print::{
    BikriPrintDataInput, PrintData, VoucherPrintDataInput, bikri_print_data, voucher_print_data,
};
use mandi_services::ServiceError;
use mandi_services::auth;
use mandi_services::core::printing::{PrintInput, PrintOutcome, PrintVoucherInput};

use super::run;
use crate::error::UiError;
use crate::print_job;

/// The print data of one document of a saved بکری: the firm header in the print language, the
/// title and every label as a Fluent message ID, the lines and totals as stored and already
/// formatted for print, the amount in words, the paper class, `dir` and `lang`. One page per
/// seller (seller bill), per buyer (buyer slips) or per one-time buyer (پختہ slip). Read only.
/// Errors: `ERR_NOT_FOUND` on `bikri_id`.
#[tauri::command]
pub async fn reports_bikri_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BikriPrintDataInput,
) -> Result<PrintData, UiError> {
    run(app, "reports_bikri_print_data", move |ctx| {
        auth::require(ctx)?;
        bikri_print_data(ctx.db(), &input).map_err(print_job::reports_service_error)
    })
    .await
}

/// Prints the documents of a saved بکری on the printer of Settings, or saves each as a PDF named
/// `<document>-<number>-<language>.pdf`. Raises the print count of the بکری in the same
/// transaction as its audit row. Printing needs Windows. Errors: `ERR_NOT_FOUND` on `bikri_id`,
/// `ERR_REQUIRED` on `docs`, the copies codes on `copies`, `ERR_PRINT_WINDOWS_ONLY` on any other
/// system, `ERR_PRINTER_NOT_FOUND` (value `printer`), `ERR_PRINT_FAILED`.
#[tauri::command]
pub async fn reports_print<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintInput,
) -> Result<PrintOutcome, UiError> {
    print_job::run(app, input).await
}

/// The names of the printers installed on this computer, for the Printing settings. Empty on
/// any system but Windows. Read only.
#[tauri::command]
pub async fn reports_list_printers<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<String>, UiError> {
    run(app, "reports_list_printers", |ctx| {
        auth::require(ctx)?;
        print_job::list_printers().map_err(ServiceError::Data)
    })
    .await
}

/// The print data of the slip of one voucher (PRD FR-15.2): the firm header, the title and
/// every label as a Fluent message ID, the party, the amount and its words in the print
/// language, the mode with the bank and cheque, the paper class, `dir` and `lang`. One page.
/// Read only. Errors: `ERR_NOT_FOUND` on `voucher_id`.
#[tauri::command]
pub async fn reports_voucher_print_data<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: VoucherPrintDataInput,
) -> Result<PrintData, UiError> {
    run(app, "reports_voucher_print_data", move |ctx| {
        auth::require(ctx)?;
        voucher_print_data(ctx.db(), &input).map_err(print_job::reports_voucher_service_error)
    })
    .await
}

/// Prints the slip of a saved voucher on the printer of Settings, or saves it as a PDF named
/// `receipt-slip-<voucher id>-<language>.pdf` (the numbers of the three series repeat, the row id
/// does not). The print is written to the audit log (vouchers have no print count). Printing
/// needs Windows. Errors: as `reports_print`, with `ERR_NOT_FOUND` on `voucher_id`.
#[tauri::command]
pub async fn reports_print_voucher<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintVoucherInput,
) -> Result<PrintOutcome, UiError> {
    print_job::run_voucher(app, input).await
}
