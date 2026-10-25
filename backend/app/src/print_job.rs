//! The print action `reports_print` (plan 14, PRD FR-12.12, FR-7.57, FR-19.3, NFR-SEC-13).
//!
//! Steps:
//!
//! 1. Check the request and read the settings (`services::printing::plan`), then ask the print
//!    data of each document whether it has any page. A document with nothing to print (no
//!    one-time buyer for the پختہ slip) is left out.
//! 2. On any system but Windows, stop with `ERR_PRINT_WINDOWS_ONLY`. The screen then offers the
//!    PDF preview inside the app.
//! 3. On Windows, print or save each document through a hidden window (`print_windows`).
//! 4. If anything was printed, raise the print count of the بکری and write the audit row in one
//!    transaction (`services::printing::record`). A print that failed before printing anything
//!    changes nothing. If a later document fails after an earlier one was printed, the earlier
//!    one is still counted and the error is returned.

use std::collections::BTreeMap;
use std::path::PathBuf;

use mandi_reports::ReportsError;
use mandi_reports::print::{
    BikriPrintDataInput, VoucherPrintDataInput, bikri_print_data, voucher_print_data,
};
use mandi_services::ServiceError;
use mandi_services::auth::{self, Actor};
use mandi_services::core::auth::Language;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::printing::{
    PrintDoc, PrintInput, PrintOutcome, PrintTo, PrintVoucherInput, pdf_file_name,
};
use mandi_services::printing::{self, PrintPlan, VoucherPrintPlan};
use tauri::Manager;

use crate::error::{UiError, UiErrorItem};
use crate::print_types::{Job, JobError, JobTarget};
use crate::runner::run_blocking;
use crate::state::AppState;

/// The folder inside the app data folder where PDFs are written.
const PDF_FOLDER: &str = "prints";

/// A request that passed its checks, with the jobs it will run.
struct Prepared {
    actor: Actor,
    plan: PrintPlan,
    jobs: Vec<Job>,
}

/// What the print system did.
#[derive(Default)]
struct Executed {
    printed: Vec<PrintDoc>,
    pdf_paths: Vec<PathBuf>,
    error: Option<UiError>,
}

/// Runs the print action. See the module docs.
pub async fn run<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintInput,
) -> Result<PrintOutcome, UiError> {
    let state = AppState::of(&app)?;
    // Only looked up here; an error is raised after the sign-in check (Code Standards S1).
    let data_dir = app.path().app_data_dir();
    let prepared = {
        let state = state.clone();
        run_blocking("reports_print", move || {
            prepare(&state, &input, data_dir.map(|dir| dir.join(PDF_FOLDER)))
        })
        .await?
    };
    if !cfg!(windows) {
        return Err(UiError::single(ErrorCode::ErrPrintWindowsOnly, None));
    }
    let executed = execute(&app, &prepared.jobs).await;

    let to = prepared.plan.to;
    if !executed.printed.is_empty() {
        let state = state.clone();
        let printed = executed.printed.clone();
        run_blocking("reports_print", move || {
            let ctx = state.context()?;
            printing::record(&ctx, &prepared.actor, &prepared.plan, &printed)?;
            Ok(())
        })
        .await?;
    }
    if let Some(error) = executed.error {
        return Err(error);
    }
    Ok(PrintOutcome {
        printed: executed.printed,
        pdf_paths: (to == PrintTo::Pdf).then(|| {
            executed
                .pdf_paths
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect()
        }),
    })
}

/// Runs the slip print of one voucher (plan 15, PRD FR-15.2): the same hidden window and the same
/// settings as the بکری documents, one page. A voucher has no print count, so a print that
/// happened is written to the audit log.
pub async fn run_voucher<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintVoucherInput,
) -> Result<PrintOutcome, UiError> {
    let state = AppState::of(&app)?;
    let data_dir = app.path().app_data_dir();
    let (actor, plan, job) = {
        let state = state.clone();
        run_blocking("reports_print_voucher", move || {
            prepare_voucher(&state, &input, data_dir.map(|dir| dir.join(PDF_FOLDER)))
        })
        .await?
    };
    if !cfg!(windows) {
        return Err(UiError::single(ErrorCode::ErrPrintWindowsOnly, None));
    }
    let executed = execute(&app, std::slice::from_ref(&job)).await;
    if !executed.printed.is_empty() {
        let state = state.clone();
        let plan = plan.clone();
        run_blocking("reports_print_voucher", move || {
            let ctx = state.context()?;
            printing::record_voucher(&ctx, &actor, &plan)?;
            Ok(())
        })
        .await?;
    }
    if let Some(error) = executed.error {
        return Err(error);
    }
    Ok(PrintOutcome {
        printed: executed.printed,
        pdf_paths: (plan.to == PrintTo::Pdf).then(|| {
            executed
                .pdf_paths
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect()
        }),
    })
}

/// Checks a voucher slip request and makes its one job.
fn prepare_voucher(
    state: &AppState,
    input: &PrintVoucherInput,
    pdf_dir: tauri::Result<PathBuf>,
) -> Result<(Actor, VoucherPrintPlan, Job), UiError> {
    let ctx = state.context()?;
    let actor = auth::require(&ctx)?;
    let plan = printing::plan_voucher(&ctx, input)?;
    let pdf_dir = pdf_dir.map_err(UiError::internal)?;
    let query = VoucherPrintDataInput {
        voucher_id: plan.voucher_id,
        language: plan.language,
    };
    let data = voucher_print_data(ctx.db(), &query).map_err(reports_voucher_error)?;
    let language = match plan.language {
        Language::Ur => "ur",
        Language::En => "en",
    };
    let job = Job {
        doc: PrintDoc::ReceiptSlip,
        paper: data.paper,
        query: format!(
            "voucher={}&doc={}&language={language}",
            plan.voucher_id,
            PrintDoc::ReceiptSlip.as_str()
        ),
        target: match plan.to {
            PrintTo::Printer => JobTarget::Printer {
                name: plan.setup.printer.clone(),
                copies: plan.copies,
            },
            // The file is named by the voucher's row id: the numbers of the three series repeat.
            PrintTo::Pdf => JobTarget::Pdf(pdf_dir.join(pdf_file_name(
                PrintDoc::ReceiptSlip,
                plan.voucher_id,
                plan.language,
            ))),
        },
    };
    Ok((actor, plan, job))
}

/// Checks the request and lists the jobs: one for each document that has a page.
fn prepare(
    state: &AppState,
    input: &PrintInput,
    pdf_dir: tauri::Result<PathBuf>,
) -> Result<Prepared, UiError> {
    let ctx = state.context()?;
    let actor = auth::require(&ctx)?;
    let plan = printing::plan(&ctx, input)?;
    let pdf_dir = pdf_dir.map_err(UiError::internal)?;
    let mut jobs = Vec::with_capacity(plan.docs.len());
    for doc in &plan.docs {
        let query = BikriPrintDataInput {
            bikri_id: plan.bikri_id,
            doc: *doc,
            language: plan.language,
            only_buyer_id: plan.only_buyer_id,
        };
        let data = bikri_print_data(ctx.db(), &query).map_err(reports_error)?;
        if data.pages.is_empty() {
            continue;
        }
        jobs.push(Job {
            doc: *doc,
            paper: data.paper,
            query: print_query(&plan, *doc),
            target: match plan.to {
                PrintTo::Printer => JobTarget::Printer {
                    name: plan.setup.printer.clone(),
                    copies: plan.copies,
                },
                PrintTo::Pdf => {
                    JobTarget::Pdf(pdf_dir.join(pdf_file_name(*doc, plan.number, plan.language)))
                }
            },
        });
    }
    Ok(Prepared { actor, plan, jobs })
}

/// The query of the print route (without the job number, which the driver adds).
fn print_query(plan: &PrintPlan, doc: PrintDoc) -> String {
    let language = match plan.language {
        Language::Ur => "ur",
        Language::En => "en",
    };
    let mut query = format!(
        "bikri={}&doc={}&language={language}",
        plan.bikri_id,
        doc.as_str()
    );
    if let (PrintDoc::BuyerSlips, Some(buyer)) = (doc, plan.only_buyer_id) {
        query.push_str(&format!("&buyer={buyer}"));
    }
    query
}

/// A reports error as the error the screen sees. A missing record is `ERR_NOT_FOUND` on
/// `bikri_id`; the rest keep their cause for the log.
pub(crate) fn reports_error(error: ReportsError) -> UiError {
    match error {
        ReportsError::NotFound => UiError::single(ErrorCode::ErrNotFound, Some("bikri_id")),
        ReportsError::Db(error) => UiError::from(error),
        error @ (ReportsError::Money(_) | ReportsError::Words(_)) => {
            UiError::coded(error.code(), &error)
        }
        other => UiError::internal(other),
    }
}

/// A reports error of a voucher slip as the error the screen sees: a missing voucher is
/// `ERR_NOT_FOUND` on `voucher_id`.
pub(crate) fn reports_voucher_error(error: ReportsError) -> UiError {
    match error {
        ReportsError::NotFound => UiError::single(ErrorCode::ErrNotFound, Some("voucher_id")),
        // A voucher that has no slip (an expense, a loan, a جنرل) is refused with a code at the
        // field, not an internal error.
        ReportsError::Rejected(items) => UiError::from(ServiceError::from(items)),
        other => reports_error(other),
    }
}

/// The same, for a voucher slip inside `commands::run`.
pub(crate) fn reports_voucher_service_error(error: ReportsError) -> ServiceError {
    match error {
        ReportsError::NotFound => ServiceError::reject_field(ErrorCode::ErrNotFound, "voucher_id"),
        ReportsError::Rejected(items) => ServiceError::from(items),
        other => reports_service_error(other),
    }
}

/// The same, for a command that runs inside `commands::run`.
pub(crate) fn reports_service_error(error: ReportsError) -> ServiceError {
    match error {
        ReportsError::NotFound => ServiceError::reject_field(ErrorCode::ErrNotFound, "bikri_id"),
        ReportsError::Db(error) => ServiceError::Db(error),
        error @ (ReportsError::Money(_) | ReportsError::Words(_)) => {
            tracing::warn!(cause = %error, "a print total could not be made");
            ServiceError::reject(error.code())
        }
        other => ServiceError::Data(other.to_string()),
    }
}

/// Prints the jobs one after the other. Stops at the first failure.
#[cfg(windows)]
async fn execute<R: tauri::Runtime>(app: &tauri::AppHandle<R>, jobs: &[Job]) -> Executed {
    let mut executed = Executed::default();
    for job in jobs {
        match crate::print_windows::run_job(app, job).await {
            Ok(()) => {
                executed.printed.push(job.doc);
                if let JobTarget::Pdf(path) = &job.target {
                    executed.pdf_paths.push(path.clone());
                }
            }
            Err(error) => {
                executed.error = Some(job_error(&error, job));
                break;
            }
        }
    }
    executed
}

/// Does nothing on a system without WebView2 printing. The caller has already answered
/// `ERR_PRINT_WINDOWS_ONLY` before it gets here.
#[cfg(not(windows))]
async fn execute<R: tauri::Runtime>(_app: &tauri::AppHandle<R>, _jobs: &[Job]) -> Executed {
    Executed::default()
}

/// A job failure as the error the screen sees: a missing printer is `ERR_PRINTER_NOT_FOUND`
/// with the name looked for; anything else is `ERR_PRINT_FAILED` with the cause in the log.
pub fn job_error(error: &JobError, job: &Job) -> UiError {
    match error {
        JobError::PrinterNotFound => {
            let name = match &job.target {
                JobTarget::Printer { name, .. } => name.clone(),
                JobTarget::Pdf(_) => String::new(),
            };
            UiError::from_items(vec![UiErrorItem {
                code: ErrorCode::ErrPrinterNotFound,
                field: None,
                values: BTreeMap::from([("printer".to_owned(), ErrorValue::Text(name))]),
            }])
        }
        JobError::Failed(cause) => UiError::coded(ErrorCode::ErrPrintFailed, cause),
    }
}

/// The printers installed on this computer. Empty on any system but Windows.
///
/// # Errors
/// The cause, as text, if Windows refuses the list.
pub fn list_printers() -> Result<Vec<String>, String> {
    #[cfg(windows)]
    {
        crate::print_windows::printers()
    }
    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn printer_job(name: &str) -> Job {
        Job {
            doc: PrintDoc::SellerBill,
            paper: mandi_services::core::printing::Paper::A4,
            query: "bikri=1&doc=seller_bill&language=ur".to_owned(),
            target: JobTarget::Printer {
                name: name.to_owned(),
                copies: 1,
            },
        }
    }

    #[test]
    fn a_missing_printer_names_the_printer() {
        let error = job_error(&JobError::PrinterNotFound, &printer_job("Office Laser"));
        let item = error.errors.first().unwrap();
        assert_eq!(item.code, ErrorCode::ErrPrinterNotFound);
        assert_eq!(
            item.values.get("printer"),
            Some(&ErrorValue::Text("Office Laser".to_owned()))
        );
    }

    #[test]
    fn any_other_failure_is_print_failed_without_the_cause_on_screen() {
        let error = job_error(&JobError::Failed("secret detail".into()), &printer_job(""));
        assert_eq!(
            error.errors.first().unwrap().code,
            ErrorCode::ErrPrintFailed
        );
        assert!(error.errors.first().unwrap().values.is_empty());
    }

    #[test]
    fn the_print_route_query_names_the_document_and_the_buyer() {
        let plan = PrintPlan {
            bikri_id: 7,
            number: 3,
            language: Language::En,
            docs: vec![PrintDoc::BuyerSlips],
            to: PrintTo::Pdf,
            copies: 1,
            setup: Default::default(),
            only_buyer_id: Some(19),
        };
        assert_eq!(
            print_query(&plan, PrintDoc::BuyerSlips),
            "bikri=7&doc=buyer_slips&language=en&buyer=19"
        );
        // The buyer filter belongs to the buyer slips only.
        assert_eq!(
            print_query(&plan, PrintDoc::SellerBill),
            "bikri=7&doc=seller_bill&language=en"
        );
    }
}
