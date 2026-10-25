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
//! 2. On any system but Windows, stop with `ERR_PRINT_WINDOWS_ONLY`. The screen then offers the
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
    ReportId, ReportPrintInput, buyer_bills_print_data, print_data, print_data_for_page, today_iso,
};
use mandi_services::auth::{self, Actor};
use mandi_services::core::auth::Language;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::printing::{Paper, PrintOutcome, PrintTo, parse_copies};
use mandi_services::db::audit::AuditEntry;
use mandi_services::{Context, ServiceError, printing};
use serde::Deserialize;
use serde_json::{Value, json};
use tauri::Manager;
use ts_rs::TS;

use crate::commands::reports_engine::{engine_error, statement_all};
use crate::error::UiError;
#[cfg(windows)]
use crate::print_job::job_error;
use crate::print_types::{Job, JobTarget};
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

/// The route query of a report print: `report=R20&language=ur&paper=a4[&slips=1]&filters=<json>`.
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
    Ok(query)
}

/// The route query of an analytics print: `analytics=A&language=ur&paper=a4&filters=<json>`.
fn analytics_query(input: &PrintAnalyticsViewInput, paper: Paper) -> Result<String, UiError> {
    let filters = serde_json::to_string(&input.view.filters).map_err(UiError::internal)?;
    Ok(format!(
        "analytics={}&language={}&paper={}&filters={}",
        input.view.view.code(),
        input.view.language.as_str(),
        paper_text(paper),
        encode_query(&filters)
    ))
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

/// Prints or saves the one job. On a system without WebView2 printing it never gets here: the
/// caller has already answered `ERR_PRINT_WINDOWS_ONLY`.
#[cfg(windows)]
async fn execute<R: tauri::Runtime>(app: &tauri::AppHandle<R>, job: &Job) -> Result<(), UiError> {
    crate::print_windows::run_job(app, job)
        .await
        .map_err(|error| job_error(&error, job))
}

#[cfg(not(windows))]
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
    if !cfg!(windows) {
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
    let data_dir = app.path().app_data_dir();
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
    let data_dir = app.path().app_data_dir();
    let prepared = {
        let state = state.clone();
        run_blocking("analytics_print_view", move || {
            prepare_analytics(&state, &input, data_dir.map(|dir| dir.join(PDF_FOLDER)))
        })
        .await?
    };
    run_prepared(&app, state, "analytics_print_view", prepared).await
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
