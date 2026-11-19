//! The Mac print driver (plan 69, PRD FR-12.12 and FR-19.3 Mac-phase note, decision M1 of plan 60).
//! Compiled on macOS only. The Windows driver (`print_windows.rs`) is not changed.
//!
//! One document at a time, like Windows:
//!
//! 1. A window `print-<job>` opens the print route of the frontend
//!    (`index.html#/print?bikri=..&doc=..&language=..&job=<job>`). The route fetches the print
//!    data, lays it out, waits for the fonts, and emits `print-ready-<job>` (or `print-error-<job>`).
//! 2. `WebviewWindow::print()` opens the native print window of macOS over that window. Its PDF
//!    button saves a PDF. No `unsafe` and no new dependency (CLAUDE.md rule 5).
//!
//! Differences from Windows, all because the native window is the user's, not ours:
//!
//! - The window is visible. The print window of macOS is a sheet of its window, and a sheet needs a
//!   window on the screen. It shows the document, so it is the print preview too.
//! - The window is NOT closed when `print()` returns. `print()` gives no answer when the user is
//!   done, and closing the window would take the print window with it. The windows of the last
//!   request are closed when the next request starts ([`close_old_windows`]) or by the user.
//! - The app cannot tell Print from Cancel, so a document counts as printed when its print window
//!   opened. The audit row says so by its action `print`, as on Windows.
//! - No file is written: a PDF is saved by the user from the print window.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use tauri::{
    AppHandle, Listener, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::print_types::{Job, JobError};
use mandi_services::core::printing::Paper;

/// How long the print route may take to get ready (data, fonts, layout).
const READY_TIMEOUT: Duration = Duration::from_secs(20);
/// CSS pixels per inch.
const CSS_PIXELS_PER_INCH: u32 = 96;
/// The height of the window that shows the document, in CSS pixels.
const WINDOW_HEIGHT_PX: f64 = 900.0;
/// The label of every print window starts with this (as `capabilities/print.json`).
const LABEL_PREFIX: &str = "print-";

static NEXT_JOB: AtomicU64 = AtomicU64::new(1);

/// The width of the window of a paper: the paper's width at 96 pixels an inch, rounded.
/// Example: A4 is 8.27 in, 794 pixels.
fn window_width_px(paper: Paper) -> u32 {
    (paper.width_centi_inches() * CSS_PIXELS_PER_INCH + 50) / 100
}

/// Closes the print windows that an earlier request left open (see the module docs). Called once at
/// the start of a request, never between the documents of one request: their print windows are
/// open at the same time.
pub fn close_old_windows<R: Runtime>(app: &AppHandle<R>) {
    for (label, window) in app.webview_windows() {
        if label.starts_with(LABEL_PREFIX) {
            // Only fails when the window is already gone.
            window.destroy().ok();
        }
    }
}

/// Opens the native print window for one document.
///
/// # Errors
/// `JobError::Failed` (with the cause) if the window cannot be made, the page never got ready or
/// reported an error, or the web view refuses to print.
pub async fn run_job<R: Runtime>(app: &AppHandle<R>, job: &Job) -> Result<(), JobError> {
    run_job_numbered(app, job, NEXT_JOB.fetch_add(1, Ordering::Relaxed)).await
}

/// [`run_job`] with the job number given, so a test can name the events.
async fn run_job_numbered<R: Runtime>(
    app: &AppHandle<R>,
    job: &Job,
    job_no: u64,
) -> Result<(), JobError> {
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

    let url = format!("index.html#/print?{}&job={job_no}", job.query);
    let built = WebviewWindowBuilder::new(
        app,
        format!("{LABEL_PREFIX}{job_no}"),
        WebviewUrl::App(url.into()),
    )
    .visible(true)
    .focused(true)
    .center()
    .inner_size(f64::from(window_width_px(job.paper)), WINDOW_HEIGHT_PX)
    .build();

    let result = match built {
        Ok(window) => {
            let result = drive(&window, ready_rx).await;
            if !keeps_window(&result) {
                window.destroy().ok();
            }
            result
        }
        Err(error) => Err(JobError::Failed(format!("print window: {error}"))),
    };
    app.unlisten(ready_id);
    app.unlisten(error_id);
    result
}

/// A window whose print window opened stays, because the print window is its sheet. A window that
/// did not print has nothing to keep.
fn keeps_window(result: &Result<(), JobError>) -> bool {
    result.is_ok()
}

/// Waits for the page, then opens the native print window.
async fn drive<R: Runtime>(
    window: &WebviewWindow<R>,
    ready: Receiver<Result<f64, String>>,
) -> Result<(), JobError> {
    match wait(ready, READY_TIMEOUT).await? {
        Ok(_height_px) => {}
        Err(payload) => {
            return Err(JobError::Failed(format!(
                "the print page reported an error: {payload}"
            )));
        }
    }
    // The sheet comes up on the window the user can see and answer.
    window.set_focus().ok();
    window
        .print()
        .map_err(|error| JobError::Failed(format!("print window: {error}")))
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tauri::{Emitter, Manager};

    use super::*;
    use crate::print_types::{Job, JobTarget};
    use mandi_services::core::printing::{Paper, PrintDoc};

    fn job() -> Job {
        Job {
            doc: Some(PrintDoc::SellerBill),
            paper: Paper::A4,
            query: "bikri=1&doc=seller_bill&language=ur".to_owned(),
            target: JobTarget::Printer {
                name: String::new(),
                copies: 1,
            },
        }
    }

    #[test]
    fn the_print_window_is_as_wide_as_the_paper() {
        // A4 is 8.27 in at 96 pixels an inch.
        assert_eq!(window_width_px(Paper::A4), 794);
        assert_eq!(window_width_px(Paper::A5), 560);
        assert_eq!(window_width_px(Paper::Mm80), 302);
    }

    #[test]
    fn a_ready_page_opens_the_native_print_window_and_stays_open() {
        let app = tauri::test::mock_app();
        let handle = app.handle().clone();
        let emitter = handle.clone();
        // The print route says it is ready, the way the real page does.
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            emitter
                .emit(
                    "print-ready-9001",
                    serde_json::json!({ "height_px": 1000.0 }),
                )
                .ok();
        });
        let result = tauri::async_runtime::block_on(run_job_numbered(&handle, &job(), 9001));
        assert_eq!(result, Ok(()));
        // The sheet of the print window belongs to its window: the window must still be there.
        assert!(handle.get_webview_window("print-9001").is_some());
        // The next request closes the windows of the last one. (The mock runtime never removes a
        // window, so only the call is checked; the real runtime closes it on its own thread.)
        close_old_windows(&handle);
    }

    #[test]
    fn a_page_that_reports_an_error_fails_the_job() {
        let app = tauri::test::mock_app();
        let handle = app.handle().clone();
        let emitter = handle.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            emitter
                .emit(
                    "print-error-9002",
                    serde_json::json!({ "codes": ["ERR_NOT_FOUND"] }),
                )
                .ok();
        });
        let result = tauri::async_runtime::block_on(run_job_numbered(&handle, &job(), 9002));
        assert!(matches!(result, Err(JobError::Failed(_))), "{result:?}");
        assert!(!keeps_window(&result));
    }

    #[test]
    fn only_a_job_that_opened_its_print_window_keeps_its_window() {
        assert!(keeps_window(&Ok(())));
        assert!(!keeps_window(&Err(JobError::PrinterNotFound)));
        assert!(!keeps_window(&Err(JobError::Failed("x".to_owned()))));
    }
}
