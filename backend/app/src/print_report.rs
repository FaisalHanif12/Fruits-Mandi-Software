//! The print and PDF of a report, of the slips of R5 "Print All", or of an analytics view, through
//! the shared print job of plan 17e (PRD FR-19.3, FR-18.4, FR-19.13; plan d5-late).
//!
//! A report is not one of the five documents of the Printing settings, so its job has no
//! `PrintDoc` (`Job.doc` is `None`) and a print answers an empty `printed` list, with the PDF paths
//! when it was saved as a PDF. It has no print count: the print is written to the audit log
//! instead. Paper and copies come from the request (A4 and the copies of the Printing settings when
//! they are left out); the printer is the one of the Printing settings.
//!
//! Steps, like the other prints:
//!
//! 1. Check the request, and build the print data once, so a request that cannot be built (a bad
//!    date, a missing party) stops before any paper is used.
//! 2. On any system but Windows and a Mac, stop with `ERR_PRINT_WINDOWS_ONLY`. The screen then offers the
//!    preview inside the app.
//! 3. On Windows, print or save through the same hidden window as the other documents. The print
//!    route reads `report=<id>` or `analytics=<view>` with the language, the paper and the filters
//!    as JSON, fetches the print data and the shared header and footer (`reports_print_header`),
//!    and lays them out.
//! 4. If it printed, write the audit row (action `print`, entity `report` or `analytics_view`).
//!
//! Example: R20 for Urdu to a PDF on 10-10-26 saves `report-r20-20261010-ur.pdf` in the folder of
//! the other PDFs; the route query is `report=R20&language=ur&paper=a4&filters=%7B...%7D`.

use std::path::PathBuf;

use mandi_reports::analytics::{AnalyticsPrintInput, print_data as analytics_print_data};
use mandi_reports::reports::{
    BikriListPrintInput, ReportId, ReportPrintInput, StatementPrintDataInput,
    bikri_list_print_data, buyer_bills_print_data, party_statement_print_data, print_data,
    print_data_for_page, today_iso,
};
use mandi_services::auth::{self, Actor};
use mandi_services::core::auth::Language;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::paging::SortInput;
use mandi_services::core::printing::{Paper, PrintOutcome, PrintTo, parse_copies};
use mandi_services::db::audit::AuditEntry;
use mandi_services::{Context, ServiceError, bikri, party_ledgers, printing};
use serde::Deserialize;
use serde_json::{Value, json};
use ts_rs::TS;

use crate::commands::reports_engine::{engine_error, statement_all};
use crate::error::UiError;
#[cfg(any(windows, target_os = "macos"))]
use crate::print_job::job_error;
use crate::print_types::{Job, JobTarget, opens_system_print_window, prints_here};
use crate::runner::run_blocking;
use crate::state::AppState;

/// The folder inside the app data folder where PDFs are written: the one of the other prints.
const PDF_FOLDER: &str = "prints";

/// Input of `reports_print_report`: a report, or the slips of R5, to the printer or to a PDF.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PrintReportInput {
    /// The report, its filters, the print language and the paper (A4 when left out).
    pub report: ReportPrintInput,
    /// True prints one slip for each credit buyer of R5 ("Print All", PRD 19.2 R5). Any other
    /// report is refused with `ERR_OUT_OF_RANGE` on `slips`.
    #[serde(default)]
    pub slips: bool,
    /// The printer of the Printing settings, or a PDF.
    pub to: PrintTo,
    /// Copies, 1 to 99, typed. Missing means the copies of the Printing settings.
    #[serde(default)]
    pub copies: Option<String>,
}

/// Input of `analytics_print_view`: an analytics view to the printer or to a PDF.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PrintAnalyticsViewInput {
    /// The view, its filters, the print language and the paper (A4 when left out).
    pub view: AnalyticsPrintInput,
    /// The printer of the Printing settings, or a PDF.
    pub to: PrintTo,
    /// Copies, 1 to 99, typed. Missing means the copies of the Printing settings.
    #[serde(default)]
    pub copies: Option<String>,
}

/// Input of `bikri_list_print`: پچھلی بکری (today's section and a window of days) to the printer or
/// to a PDF (PRD FR-12.46, plan 72 P8).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PrintBikriListInput {
    /// The window, the filters or the switch "show every بکری", the print language and the paper
    /// (A4 when left out): the input of `bikri_list_print_data`.
    pub list: BikriListPrintInput,
    /// The printer of the Printing settings, or a PDF.
    pub to: PrintTo,
    /// Copies, 1 to 99, typed. Missing means the copies of the Printing settings.
    #[serde(default)]
    pub copies: Option<String>,
}

/// Input of `party_ledgers_statement_print`: one party's statement for a period to the printer or to a
/// PDF (PRD FR-12.47, plan 74 T1).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PrintStatementInput {
    /// The party, the period as typed, the print language and the paper (A4 when left out): the input
    /// of `party_ledgers_statement_print_data`.
    pub statement: StatementPrintDataInput,
    /// The printer of the Printing settings, or a PDF.
    pub to: PrintTo,
    /// Copies, 1 to 99, typed. Missing means the copies of the Printing settings.
    #[serde(default)]
    pub copies: Option<String>,
}

/// A request that passed its checks.
struct Prepared {
    actor: Actor,
    job: Job,
    /// The entity of the audit row: `report` or `analytics_view`.
    entity: &'static str,
    /// What the audit row records.
    audit: Value,
}

/// Percent-encodes text for a URL query: every byte except `A-Z a-z 0-9 - _ . ~` becomes `%XX`.
fn encode_query(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// The paper as the print route reads it (`a4`, `a5`, `mm80`).
fn paper_text(paper: Paper) -> String {
    serde_json::to_value(paper)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "a4".to_owned())
}

/// The sort of a print as the route reads it: `&sort=<json of the screen's sort>`, or nothing.
fn sort_query(sort: Option<&SortInput>) -> Result<String, UiError> {
    match sort {
        Some(sort) => {
            let json = serde_json::to_string(sort).map_err(UiError::internal)?;
            Ok(format!("&sort={}", encode_query(&json)))
        }
        None => Ok(String::new()),
    }
}

/// The route query of a report print:
/// `report=R20&language=ur&paper=a4[&slips=1]&filters=<json>[&sort=<json>][&months=1]`.
/// `sort` and `months` are the screen's order and month rows (plan 38 R4), so the window has the
/// rows in the order of the screen.
fn report_query(input: &PrintReportInput, paper: Paper) -> Result<String, UiError> {
    let filters = serde_json::to_string(&input.report.filters).map_err(UiError::internal)?;
    let mut query = format!(
        "report={}&language={}&paper={}",
        input.report.id.code(),
        input.report.language.as_str(),
        paper_text(paper)
    );
    if input.slips {
        query.push_str("&slips=1");
    }
    query.push_str(&format!("&filters={}", encode_query(&filters)));
    query.push_str(&sort_query(input.report.sort.as_ref())?);
    if input.report.group_by_month {
        query.push_str("&months=1");
    }
    Ok(query)
}

/// The route query of an analytics print:
/// `analytics=A&language=ur&paper=a4&filters=<json>[&sort=<json>]`.
fn analytics_query(input: &PrintAnalyticsViewInput, paper: Paper) -> Result<String, UiError> {
    let filters = serde_json::to_string(&input.view.filters).map_err(UiError::internal)?;
    Ok(format!(
        "analytics={}&language={}&paper={}&filters={}{}",
        input.view.view.code(),
        input.view.language.as_str(),
        paper_text(paper),
        encode_query(&filters),
        sort_query(input.view.sort.as_ref())?
    ))
}

/// The route query of a print of پچھلی بکری:
/// `bikrilist=1&language=ur&paper=a4&list=<json of the window and the filters>`. The print window
/// asks `bikri_list_print_data` for the data with that JSON, the language and the paper.
fn bikri_list_query(input: &PrintBikriListInput, paper: Paper) -> Result<String, UiError> {
    let list = serde_json::to_string(&input.list.list).map_err(UiError::internal)?;
    Ok(format!(
        "bikrilist=1&language={}&paper={}&list={}",
        input.list.language.as_str(),
        paper_text(paper),
        encode_query(&list)
    ))
}

/// The route query of a print of one party's statement:
/// `statement=<account id>&from=<DD-MM-YYYY>&to=<DD-MM-YYYY>&language=ur&paper=a4`. The dates go as
/// typed: the print window asks `party_ledgers_statement_print_data` with them and the backend
/// checks them again.
fn statement_query(input: &PrintStatementInput, paper: Paper) -> String {
    let statement = &input.statement;
    format!(
        "statement={}&from={}&to={}&language={}&paper={}",
        statement.account_id,
        encode_query(&statement.from),
        encode_query(&statement.to),
        statement.language.as_str(),
        paper_text(paper)
    )
}

/// `YYYY-MM-DD` as `YYYYMMDD`.
fn compact_date(iso: &str) -> String {
    iso.replace('-', "")
}

/// The PDF name of a report: `report-<id>-<YYYYMMDD>-<lang>.pdf`, and `report-r5-slips-...` for the
/// slips of R5.
#[must_use]
pub fn report_pdf_name(id: ReportId, slips: bool, date_iso: &str, language: Language) -> String {
    format!(
        "report-{}{}-{}-{}.pdf",
        id.code().to_lowercase(),
        if slips { "-slips" } else { "" },
        compact_date(date_iso),
        language.as_str()
    )
}

/// The PDF name of an analytics view: `analytics-<view>-<YYYYMMDD>-<lang>.pdf`.
#[must_use]
pub fn analytics_pdf_name(view_code: &str, date_iso: &str, language: Language) -> String {
    format!(
        "analytics-{}-{}-{}.pdf",
        view_code.to_lowercase(),
        compact_date(date_iso),
        language.as_str()
    )
}

/// The PDF name of پچھلی بکری: `bikri-list-<YYYYMMDD>-<lang>.pdf`.
#[must_use]
pub fn bikri_list_pdf_name(date_iso: &str, language: Language) -> String {
    format!(
        "bikri-list-{}-{}.pdf",
        compact_date(date_iso),
        language.as_str()
    )
}

/// The PDF name of one party's statement: `party-statement-<code>-<YYYYMMDD>-<lang>.pdf`. The code is
/// the digits of the account code, so the name is safe for a file.
#[must_use]
pub fn statement_pdf_name(code: &str, date_iso: &str, language: Language) -> String {
    format!(
        "party-statement-{}-{}-{}.pdf",
        code,
        compact_date(date_iso),
        language.as_str()
    )
}

/// The printer and the copies of this print: the typed copies, else those of the Printing settings.
fn printer_and_copies(
    ctx: &Context,
    actor: &Actor,
    typed: Option<&str>,
) -> Result<(String, u32), ServiceError> {
    let settings = printing::get_settings(ctx, actor)?;
    let typed = typed.map(str::trim).filter(|text| !text.is_empty());
    let copies = match typed {
        Some(text) => parse_copies("copies", text).map_err(ServiceError::from)?,
        None => settings.copies,
    };
    Ok((settings.printer, copies))
}

/// Builds the print data once, so a request that cannot be built stops before any paper is used.
fn check_report(ctx: &Context, input: &PrintReportInput) -> Result<(), ServiceError> {
    let report = &input.report;
    let built = if input.slips {
        buyer_bills_print_data(ctx.db(), report).map(drop)
    } else if report.id == ReportId::R31 {
        // The partner statement comes from the services (PRD FR-17.9).
        let page = statement_all(ctx, &report.filters)?;
        print_data_for_page(ctx.db(), &page, report).map(drop)
    } else {
        print_data(ctx.db(), report).map(drop)
    };
    built.map_err(engine_error)
}

fn job_target(
    to: PrintTo,
    printer: String,
    copies: u32,
    pdf_dir: tauri::Result<PathBuf>,
    pdf_name: String,
) -> Result<JobTarget, UiError> {
    Ok(match to {
        PrintTo::Printer => JobTarget::Printer {
            name: printer,
            copies,
        },
        PrintTo::Pdf => JobTarget::Pdf(pdf_dir.map_err(UiError::internal)?.join(pdf_name)),
    })
}

/// Checks a report print request and makes its one job.
fn prepare_report(
    state: &AppState,
    input: &PrintReportInput,
    pdf_dir: tauri::Result<PathBuf>,
) -> Result<Prepared, UiError> {
    let ctx = state.context()?;
    let actor = auth::require(&ctx)?;
    if input.slips && input.report.id != ReportId::R5 {
        return Err(UiError::single(ErrorCode::ErrOutOfRange, Some("slips")));
    }
    let (printer, copies) = printer_and_copies(&ctx, &actor, input.copies.as_deref())?;
    // The partner code comes before anything is built (PRD FR-17.11). The print window asks the
    // data again through `reports_print_data`, which checks it too and leaves out the partner parts.
    let mut filters = input.report.filters.clone();
    crate::commands::partner_gate::report(&ctx, input.report.id, &mut filters)?;
    check_report(&ctx, input)?;
    let paper = input.report.paper.unwrap_or(Paper::A4);
    let today = today_iso(ctx.db()).map_err(engine_error)?;
    let name = report_pdf_name(input.report.id, input.slips, &today, input.report.language);
    let job = Job {
        doc: None,
        paper,
        query: report_query(input, paper)?,
        target: job_target(input.to, printer, copies, pdf_dir, name)?,
    };
    let audit = json!({
        "report": input.report.id.code(),
        "slips": input.slips,
        "filters": input.report.filters,
        "sort": input.report.sort,
        "group_by_month": input.report.group_by_month,
        "language": input.report.language.as_str(),
        "to": input.to,
        "copies": copies,
    });
    Ok(Prepared {
        actor,
        job,
        entity: "report",
        audit,
    })
}

/// Checks an analytics print request and makes its one job.
fn prepare_analytics(
    state: &AppState,
    input: &PrintAnalyticsViewInput,
    pdf_dir: tauri::Result<PathBuf>,
) -> Result<Prepared, UiError> {
    let ctx = state.context()?;
    let actor = auth::require(&ctx)?;
    let (printer, copies) = printer_and_copies(&ctx, &actor, input.copies.as_deref())?;
    let mut filters = input.view.filters.clone();
    crate::commands::partner_gate::analytics(&ctx, input.view.view, &mut filters)?;
    analytics_print_data(ctx.db(), &input.view)
        .map(drop)
        .map_err(engine_error)?;
    let paper = input.view.paper.unwrap_or(Paper::A4);
    let today = today_iso(ctx.db()).map_err(engine_error)?;
    let name = analytics_pdf_name(input.view.view.code(), &today, input.view.language);
    let job = Job {
        doc: None,
        paper,
        query: analytics_query(input, paper)?,
        target: job_target(input.to, printer, copies, pdf_dir, name)?,
    };
    let audit = json!({
        "view": input.view.view.code(),
        "filters": input.view.filters,
        "sort": input.view.sort,
        "language": input.view.language.as_str(),
        "to": input.to,
        "copies": copies,
    });
    Ok(Prepared {
        actor,
        job,
        entity: "analytics_view",
        audit,
    })
}

/// Checks a print request of پچھلی بکری and makes its one job. The whole list is read and laid out
/// once here, so a bad window stops before any paper is used.
fn prepare_bikri_list(
    state: &AppState,
    input: &PrintBikriListInput,
    pdf_dir: tauri::Result<PathBuf>,
) -> Result<Prepared, UiError> {
    let ctx = state.context()?;
    let actor = auth::require(&ctx)?;
    let (printer, copies) = printer_and_copies(&ctx, &actor, input.copies.as_deref())?;
    let whole = bikri::list_whole(&ctx, &input.list.list)?;
    bikri_list_print_data(ctx.db(), &whole, &input.list)
        .map(drop)
        .map_err(engine_error)?;
    let paper = input.list.paper.unwrap_or(Paper::A4);
    let today = today_iso(ctx.db()).map_err(engine_error)?;
    let name = bikri_list_pdf_name(&today, input.list.language);
    let job = Job {
        doc: None,
        paper,
        query: bikri_list_query(input, paper)?,
        target: job_target(input.to, printer, copies, pdf_dir, name)?,
    };
    let audit = json!({
        "list": input.list.list,
        "language": input.list.language.as_str(),
        "to": input.to,
        "copies": copies,
    });
    Ok(Prepared {
        actor,
        job,
        entity: "bikri_list",
        audit,
    })
}

/// Checks a print request of one party's statement and makes its one job. The whole statement is read
/// and laid out once here, so a bad period or a party with no ledger stops before any paper is used.
fn prepare_statement(
    state: &AppState,
    input: &PrintStatementInput,
    pdf_dir: tauri::Result<PathBuf>,
) -> Result<Prepared, UiError> {
    let ctx = state.context()?;
    let actor = auth::require(&ctx)?;
    let (printer, copies) = printer_and_copies(&ctx, &actor, input.copies.as_deref())?;
    let statement = &input.statement;
    let whole =
        party_ledgers::statement_whole(&ctx, statement.account_id, &statement.from, &statement.to)?;
    party_statement_print_data(ctx.db(), &whole, statement)
        .map(drop)
        .map_err(engine_error)?;
    let paper = statement.paper.unwrap_or(Paper::A4);
    let today = today_iso(ctx.db()).map_err(engine_error)?;
    let name = statement_pdf_name(&whole.statement.head.code, &today, statement.language);
    let job = Job {
        doc: None,
        paper,
        query: statement_query(input, paper),
        target: job_target(input.to, printer, copies, pdf_dir, name)?,
    };
    let audit = json!({
        "account_id": statement.account_id,
        "period_from": whole.from,
        "period_to": whole.to,
        "language": statement.language.as_str(),
        "to": input.to,
        "copies": copies,
    });
    Ok(Prepared {
        actor,
        job,
        entity: "party_statement",
        audit,
    })
}

/// Prints or saves the one job. On a system without WebView2 printing it never gets here: the
/// caller has already answered `ERR_PRINT_WINDOWS_ONLY`.
#[cfg(windows)]
async fn execute<R: tauri::Runtime>(app: &tauri::AppHandle<R>, job: &Job) -> Result<(), UiError> {
    crate::print_windows::run_job(app, job)
        .await
        .map_err(|error| job_error(&error, job))
}

/// Opens the system print window (Mac phase, plan 69). The windows of the last request are closed
/// first; no file is written.
#[cfg(target_os = "macos")]
async fn execute<R: tauri::Runtime>(app: &tauri::AppHandle<R>, job: &Job) -> Result<(), UiError> {
    crate::print_mac::close_old_windows(app);
    crate::print_mac::run_job(app, job)
        .await
        .map_err(|error| job_error(&error, job))
}

#[cfg(not(any(windows, target_os = "macos")))]
async fn execute<R: tauri::Runtime>(_app: &tauri::AppHandle<R>, _job: &Job) -> Result<(), UiError> {
    Ok(())
}

/// Runs a prepared job: prints it, then writes the audit row.
async fn run_prepared<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: AppState,
    name: &'static str,
    prepared: Prepared,
) -> Result<PrintOutcome, UiError> {
    if !prints_here() {
        return Err(UiError::single(ErrorCode::ErrPrintWindowsOnly, None));
    }
    execute(app, &prepared.job).await?;
    let (actor, entity, audit) = (prepared.actor, prepared.entity, prepared.audit);
    run_blocking(name, move || {
        let ctx = state.context()?;
        ctx.db().write(|tx| {
            ctx.db()
                .auditor()
                .write(
                    tx,
                    &AuditEntry {
                        user_id: Some(actor.user_id),
                        action: "print".to_owned(),
                        entity: entity.to_owned(),
                        entity_id: None,
                        old_json: None,
                        new_json: Some(audit),
                        reason: None,
                    },
                )
                .map_err(ServiceError::from)
        })?;
        Ok(())
    })
    .await?;
    Ok(PrintOutcome {
        printed: Vec::new(),
        pdf_paths: match prepared.job.target {
            // The system print window of a Mac saves the PDF itself: no path to tell.
            JobTarget::Pdf(_) if opens_system_print_window() => Some(Vec::new()),
            JobTarget::Pdf(path) => Some(vec![path.to_string_lossy().into_owned()]),
            JobTarget::Printer { .. } => None,
        },
    })
}

/// Runs the print of a report. See the module docs.
///
/// # Errors
/// As the report (`ERR_OUT_OF_RANGE`, `ERR_DATE_INVALID`, `ERR_REQUIRED`, `ERR_NOT_FOUND` on the
/// filters), `ERR_OUT_OF_RANGE` on `slips`, the copies codes on `copies`, `ERR_PRINT_WINDOWS_ONLY`,
/// `ERR_PRINTER_NOT_FOUND` and `ERR_PRINT_FAILED`.
pub async fn run_report<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintReportInput,
) -> Result<PrintOutcome, UiError> {
    let state = AppState::of(&app)?;
    let data_dir = crate::profile::app_data_dir(&app);
    let prepared = {
        let state = state.clone();
        run_blocking("reports_print_report", move || {
            prepare_report(&state, &input, data_dir.map(|dir| dir.join(PDF_FOLDER)))
        })
        .await?
    };
    run_prepared(&app, state, "reports_print_report", prepared).await
}

/// Runs the print of an analytics view. See the module docs.
///
/// # Errors
/// As the view, the copies codes on `copies`, `ERR_PRINT_WINDOWS_ONLY`, `ERR_PRINTER_NOT_FOUND`
/// and `ERR_PRINT_FAILED`.
pub async fn run_analytics<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintAnalyticsViewInput,
) -> Result<PrintOutcome, UiError> {
    let state = AppState::of(&app)?;
    let data_dir = crate::profile::app_data_dir(&app);
    let prepared = {
        let state = state.clone();
        run_blocking("analytics_print_view", move || {
            prepare_analytics(&state, &input, data_dir.map(|dir| dir.join(PDF_FOLDER)))
        })
        .await?
    };
    run_prepared(&app, state, "analytics_print_view", prepared).await
}

/// Runs the print of پچھلی بکری. See the module docs.
///
/// # Errors
/// As `bikri_list_print_data` on the window and the filters, the copies codes on `copies`,
/// `ERR_PRINT_WINDOWS_ONLY`, `ERR_PRINTER_NOT_FOUND` and `ERR_PRINT_FAILED`.
pub async fn run_bikri_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintBikriListInput,
) -> Result<PrintOutcome, UiError> {
    let state = AppState::of(&app)?;
    let data_dir = crate::profile::app_data_dir(&app);
    let prepared = {
        let state = state.clone();
        run_blocking("bikri_list_print", move || {
            prepare_bikri_list(&state, &input, data_dir.map(|dir| dir.join(PDF_FOLDER)))
        })
        .await?
    };
    run_prepared(&app, state, "bikri_list_print", prepared).await
}

/// Runs the print of one party's statement. See the module docs.
///
/// # Errors
/// As `party_ledgers_statement_print_data`, the copies codes on `copies`, `ERR_PRINT_WINDOWS_ONLY`,
/// `ERR_PRINTER_NOT_FOUND` and `ERR_PRINT_FAILED`.
pub async fn run_statement<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrintStatementInput,
) -> Result<PrintOutcome, UiError> {
    let state = AppState::of(&app)?;
    let data_dir = crate::profile::app_data_dir(&app);
    let prepared = {
        let state = state.clone();
        run_blocking("party_ledgers_statement_print", move || {
            prepare_statement(&state, &input, data_dir.map(|dir| dir.join(PDF_FOLDER)))
        })
        .await?
    };
    run_prepared(&app, state, "party_ledgers_statement_print", prepared).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_route_query_of_the_past_sales_print_carries_the_window_and_the_pdf_has_a_name() {
        let input: PrintBikriListInput = serde_json::from_value(json!({
            "list": { "from": "05-11-2026", "to": "11-11-2026", "all": true,
                      "language": "ur", "paper": "a5" },
            "to": "pdf"
        }))
        .unwrap();
        let query = bikri_list_query(&input, Paper::A5).unwrap();
        assert!(
            query.starts_with("bikrilist=1&language=ur&paper=a5&list=%7B"),
            "{query}"
        );
        assert!(!query.contains(' ') && !query.contains('{'), "{query}");
        // The window is in the JSON, and today's section is on unless it was turned off.
        assert!(query.contains("%2205-11-2026%22") && query.contains("%22with_today%22%3Atrue"));
        assert_eq!(
            bikri_list_pdf_name("2026-11-12", Language::En),
            "bikri-list-20261112-en.pdf"
        );
    }

    #[test]
    fn the_route_query_of_a_statement_print_names_the_party_the_period_and_the_pdf_has_a_name() {
        let input: PrintStatementInput = serde_json::from_value(json!({
            "statement": { "account_id": 20, "from": "01-10-2026", "to": "05-12-2026",
                           "language": "ur", "paper": "a5" },
            "to": "pdf"
        }))
        .unwrap();
        assert_eq!(
            statement_query(&input, Paper::A5),
            "statement=20&from=01-10-2026&to=05-12-2026&language=ur&paper=a5"
        );
        // A date typed as a shortcut still survives the URL.
        let typed: PrintStatementInput = serde_json::from_value(json!({
            "statement": { "account_id": 7, "from": "1 0", "to": "x&y", "language": "en" },
            "to": "printer"
        }))
        .unwrap();
        assert_eq!(
            statement_query(&typed, Paper::A4),
            "statement=7&from=1%200&to=x%26y&language=en&paper=a4"
        );
        assert_eq!(
            statement_pdf_name("0020", "2026-12-05", Language::Ur),
            "party-statement-0020-20261205-ur.pdf"
        );
    }

    #[test]
    fn a_query_value_is_percent_encoded_and_a_json_filter_survives_the_url() {
        assert_eq!(encode_query("a-b_c.d~e"), "a-b_c.d~e");
        assert_eq!(
            encode_query("{\"a\":\"x y\"}"),
            "%7B%22a%22%3A%22x%20y%22%7D"
        );
        // Urdu text is encoded byte by byte (UTF-8).
        assert_eq!(encode_query("ا"), "%D8%A7");
    }

    #[test]
    fn the_pdf_names_carry_the_report_the_day_and_the_language() {
        assert_eq!(
            report_pdf_name(ReportId::R20, false, "2026-10-10", Language::Ur),
            "report-r20-20261010-ur.pdf"
        );
        assert_eq!(
            report_pdf_name(ReportId::R5, true, "2026-10-10", Language::En),
            "report-r5-slips-20261010-en.pdf"
        );
        assert_eq!(
            analytics_pdf_name("K", "2026-10-10", Language::En),
            "analytics-k-20261010-en.pdf"
        );
    }

    #[test]
    fn the_paper_is_the_text_the_print_route_reads() {
        assert_eq!(paper_text(Paper::A4), "a4");
        assert_eq!(paper_text(Paper::A5), "a5");
        assert_eq!(paper_text(Paper::Mm80), "mm80");
    }

    fn report_input(slips: bool) -> PrintReportInput {
        serde_json::from_value(json!({
            "report": { "id": "R5", "filters": { "from": "10-10-2026", "to": "10-10-2026" },
                        "language": "ur" },
            "slips": slips,
            "to": "pdf"
        }))
        .unwrap()
    }

    #[test]
    fn the_route_query_names_the_report_the_language_the_paper_and_the_filters() {
        let query = report_query(&report_input(false), Paper::A4).unwrap();
        assert!(
            query.starts_with("report=R5&language=ur&paper=a4&filters=%7B"),
            "{query}"
        );
        assert!(!query.contains("slips"));
        assert!(!query.contains(' ') && !query.contains('{'));
        let slips = report_query(&report_input(true), Paper::A5).unwrap();
        assert!(
            slips.starts_with("report=R5&language=ur&paper=a5&slips=1&filters="),
            "{slips}"
        );
    }

    #[test]
    fn the_route_query_carries_the_sort_and_the_month_rows_of_the_screen() {
        let input: PrintReportInput = serde_json::from_value(json!({
            "report": { "id": "R13", "filters": {}, "language": "en",
                        "sort": { "column": "debit", "direction": "desc" },
                        "group_by_month": true },
            "to": "pdf"
        }))
        .unwrap();
        let query = report_query(&input, Paper::A4).unwrap();
        assert!(
            query.ends_with(
                "&sort=%7B%22column%22%3A%22debit%22%2C%22direction%22%3A%22desc%22%2C%22language%22%3Anull%7D&months=1"
            ),
            "{query}"
        );
        // A screen with no sort and no months adds nothing.
        let plain = report_query(&report_input(false), Paper::A4).unwrap();
        assert!(
            !plain.contains("&sort=") && !plain.contains("&months="),
            "{plain}"
        );
        let analytics: PrintAnalyticsViewInput = serde_json::from_value(json!({
            "view": { "view": "A", "filters": {}, "language": "ur",
                      "sort": { "column": "gross_paisa", "direction": "asc" } },
            "to": "pdf"
        }))
        .unwrap();
        let query = analytics_query(&analytics, Paper::A4).unwrap();
        assert!(
            query.contains("&sort=%7B%22column%22%3A%22gross_paisa%22"),
            "{query}"
        );
    }
}
