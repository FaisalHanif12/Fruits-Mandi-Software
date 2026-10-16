//! The Windows print driver (plan 14, Tech Stack 1, reference.md of the `print-and-export`
//! skill). Compiled on Windows only.
//!
//! One document at a time:
//!
//! 1. A hidden window `print-<job>` opens the print route of the frontend
//!    (`index.html#/print?bikri=..&doc=..&language=..&job=<job>`). The route fetches the print
//!    data with `reports_bikri_print_data`, lays it out, waits for the fonts, and emits the event
//!    `print-ready-<job>` with `{ "height_px": <height of the printed document> }` (or `print-error-<job>`).
//! 2. The web view of that window prints (`Print`) or saves a PDF (`PrintToPdf`) through
//!    `webview2::print::start`, on the thread that owns the web view.
//! 3. The window is closed, whatever happened.
//!
//! Nothing blocks the window or the command thread: the waits run on worker threads.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use tauri::{AppHandle, Listener, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::print_types::{Job, JobError, JobTarget};
use crate::webview2::print::{self, Failure, Request, Target};

/// How long the print route may take to get ready (data, fonts, layout).
const READY_TIMEOUT: Duration = Duration::from_secs(20);
/// How long the print system may take to finish one document.
const PRINT_TIMEOUT: Duration = Duration::from_secs(120);
/// CSS pixels per inch.
const CSS_PIXELS_PER_INCH: f64 = 96.0;
/// A millimetre in inches is 1 / 25.4.
const MM_PER_INCH: f64 = 25.4;
/// The roll is never shorter than this (inches), so a tiny slip still feeds a little paper.
const MIN_ROLL_HEIGHT_IN: f64 = 1.5;
/// The roll is never longer than this (inches), so a broken page cannot waste a roll.
const MAX_ROLL_HEIGHT_IN: f64 = 60.0;

static NEXT_JOB: AtomicU64 = AtomicU64::new(1);

/// Prints or saves one document. The hidden window is always closed afterwards.
///
/// # Errors
/// `JobError::PrinterNotFound` if the printer is missing or offline; `JobError::Failed`
/// (with the cause) for anything else, including a page that never got ready.
pub async fn run_job<R: Runtime>(app: &AppHandle<R>, job: &Job) -> Result<(), JobError> {
    let job_no = NEXT_JOB.fetch_add(1, Ordering::Relaxed);
    let (ready_tx, ready_rx) = mpsc::channel::<Result<f64, String>>();
    let error_tx = ready_tx.clone();
    let ready_id = app.once(format!("print-ready-{job_no}"), move |event| {
        let height = serde_json::from_str::<serde_json::Value>(event.payload())
            .ok()
            .and_then(|payload| payload.get("height_px").and_then(serde_json::Value::as_f64))
            .unwrap_or(0.0);
        ready_tx.send(Ok(height)).ok();
    });
    let error_id = app.once(format!("print-error-{job_no}"), move |event| {
        // The route sends `{ "codes": ["ERR_NOT_FOUND"] }`; the codes go to the log with the job.
        error_tx.send(Err(event.payload().to_owned())).ok();
    });

    let (width_in, _) = page_size_in(job, 0.0);
    let url = format!("index.html#/print?{}&job={job_no}", job.query);
    let built =
        WebviewWindowBuilder::new(app, format!("print-{job_no}"), WebviewUrl::App(url.into()))
            .visible(false)
            .focused(false)
            .decorations(false)
            .skip_taskbar(true)
            .inner_size(width_in * CSS_PIXELS_PER_INCH, 1_000.0)
            .build();

    let result = match built {
        Ok(window) => {
            let result = drive(&window, job, ready_rx).await;
            // Closing can fail only if the window is already gone.
            window.destroy().ok();
            result
        }
        Err(error) => Err(JobError::Failed(format!("print window: {error}"))),
    };
    app.unlisten(ready_id);
    app.unlisten(error_id);
    result
}

/// Waits for the page, then prints it.
async fn drive<R: Runtime>(
    window: &WebviewWindow<R>,
    job: &Job,
    ready: Receiver<Result<f64, String>>,
) -> Result<(), JobError> {
    let height_px = match wait(ready, READY_TIMEOUT).await? {
        Ok(height) => height,
        Err(payload) => {
            return Err(JobError::Failed(format!(
                "the print page reported an error: {payload}"
            )));
        }
    };
    let (width_in, height_in) = page_size_in(job, height_px);
    let [top, right, bottom, left] = job.paper.margins_mm().map(|mm| f64::from(mm) / MM_PER_INCH);
    let target = match &job.target {
        JobTarget::Printer { name, copies } => Target::Printer {
            name: name.clone(),
            copies: *copies,
        },
        JobTarget::Pdf(path) => {
            if let Some(folder) = path.parent() {
                std::fs::create_dir_all(folder)
                    .map_err(|error| JobError::Failed(format!("PDF folder: {error}")))?;
            }
            Target::Pdf(path.clone())
        }
    };
    let request = Request {
        target,
        width_in,
        height_in,
        margins_in: [top, right, bottom, left],
    };
    let (done_tx, done_rx) = mpsc::channel::<Result<(), Failure>>();
    let start_tx = done_tx.clone();
    window
        .with_webview(move |webview| {
            // Runs on the thread of the web view. A failure to start is reported here; a
            // failure later comes through the handler.
            if let Err(failure) = print::start(
                &webview.controller(),
                &webview.environment(),
                &request,
                start_tx,
            ) {
                done_tx.send(Err(failure)).ok();
            }
        })
        .map_err(|error| JobError::Failed(format!("web view: {error}")))?;
    match wait(done_rx, PRINT_TIMEOUT).await? {
        Ok(()) => Ok(()),
        Err(Failure::PrinterUnavailable) => Err(JobError::PrinterNotFound),
        Err(Failure::Other(cause)) => Err(JobError::Failed(cause)),
    }
}

/// The page size in inches. A4 and A5 have a fixed height; the 80 mm roll takes the height of
/// the printed document (`height_px` of the ready event, CSS pixels at 96 per inch, measured
/// by the frontend on the document itself and not on the window) plus its top and bottom margins.
fn page_size_in(job: &Job, content_height_px: f64) -> (f64, f64) {
    let width = f64::from(job.paper.width_centi_inches()) / 100.0;
    let height = match job.paper.height_centi_inches() {
        Some(centi) => f64::from(centi) / 100.0,
        None => {
            let [top, _, bottom, _] = job.paper.margins_mm();
            let margins = f64::from(top + bottom) / MM_PER_INCH;
            (content_height_px / CSS_PIXELS_PER_INCH + margins)
                .clamp(MIN_ROLL_HEIGHT_IN, MAX_ROLL_HEIGHT_IN)
        }
    };
    (width, height)
}

/// Waits for one message on a worker thread, so no async thread blocks.
async fn wait<T: Send + 'static>(receiver: Receiver<T>, limit: Duration) -> Result<T, JobError> {
    match tauri::async_runtime::spawn_blocking(move || receiver.recv_timeout(limit)).await {
        Ok(Ok(message)) => Ok(message),
        Ok(Err(_)) => Err(JobError::Failed(
            "timed out or the page closed before it answered".to_owned(),
        )),
        Err(error) => Err(JobError::Failed(format!("wait: {error}"))),
    }
}

/// The printers installed on this computer.
///
/// # Errors
/// The cause, as text, if Windows refuses the list.
pub fn printers() -> Result<Vec<String>, String> {
    print::list_printers().map_err(|failure| match failure {
        Failure::PrinterUnavailable => "no printer list".to_owned(),
        Failure::Other(cause) => cause,
    })
}
