//! The running backup (plan 18): the service the commands use, the scheduler thread that makes
//! the 15-minute and daily copies, and the copy at exit (PRD FR-20.2, FR-20.3, FR-20.6).
//!
//! The service is made once at start-up, when the data file opened, and managed by Tauri. If the
//! data file did not open there is no service and every backup command answers `ERR_INTERNAL`
//! (the screen already shows the data error from `app_data_status`).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use mandi_backup::run::Scope;
use mandi_backup::service::{BackupConfig, BackupParts, BackupService};
use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::error::UiError;
use crate::state::AppState;

/// How often the scheduler looks at what is due.
const TICK: Duration = Duration::from_secs(30);
/// How long the app waits for the copy at exit before it closes anyway (a copy that is still
/// uploading is not lost: it is in the outbox and goes on at the next start).
const EXIT_WAIT: Duration = Duration::from_secs(120);
/// The event the screen listens to while the copy at exit runs.
pub const EXIT_EVENT: &str = "backup-exit";

/// The service, managed by Tauri.
#[derive(Clone)]
pub struct BackupRuntime {
    service: Arc<BackupService>,
    stop: Arc<AtomicBool>,
}

impl BackupRuntime {
    /// A runtime for a service. Tests give a service with a folder for the cloud and secrets in
    /// memory.
    pub fn new(service: Arc<BackupService>) -> Self {
        Self {
            service,
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    /// The service.
    pub fn service(&self) -> &Arc<BackupService> {
        &self.service
    }

    /// The real runtime for the open data file: the Credential Manager, S3, the real disk. The
    /// backups folder is `backups` next to the `data` folder.
    pub fn real(state: &AppState, app_version: &str) -> Result<Self, UiError> {
        let db = state.db()?;
        let backups_dir = db
            .path()
            .parent()
            .and_then(std::path::Path::parent)
            .map_or_else(
                || std::path::PathBuf::from("backups"),
                |root| root.join("backups"),
            );
        let config = BackupConfig {
            backups_dir,
            app_version: app_version.to_owned(),
        };
        Ok(Self::new(Arc::new(BackupService::new(
            db,
            config,
            BackupParts::real(),
        ))))
    }

    /// The runtime the app manages.
    ///
    /// # Errors
    /// `ERR_INTERNAL` when the app has none (the data file did not open).
    pub fn of<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<Self, UiError> {
        app.try_state::<BackupRuntime>()
            .map(|runtime| runtime.inner().clone())
            .ok_or_else(|| UiError::internal("the backup is not started"))
    }

    /// Starts the scheduler on its own thread. It looks every 30 seconds and runs what is due.
    /// A failure or a panic in one round is logged and the next round goes on.
    pub fn start_scheduler(&self) {
        let runtime = self.clone();
        let spawned = std::thread::Builder::new()
            .name("mandi-backup-scheduler".to_owned())
            .spawn(move || {
                loop {
                    // Sleep in short steps, so a stop is heard at once.
                    let mut waited = Duration::ZERO;
                    while waited < TICK {
                        if runtime.stop.load(Ordering::SeqCst) {
                            return;
                        }
                        std::thread::sleep(Duration::from_secs(1));
                        waited += Duration::from_secs(1);
                    }
                    let round = catch_unwind(AssertUnwindSafe(|| runtime.service.tick()));
                    match round {
                        Ok(Ok(reports)) => {
                            for report in reports {
                                tracing::info!(
                                    results = report.results.len(),
                                    "scheduled backup done"
                                );
                            }
                        }
                        Ok(Err(error)) => tracing::warn!("scheduled backup: {error}"),
                        Err(_) => tracing::error!("scheduled backup panicked"),
                    }
                }
            });
        if let Err(error) = spawned {
            tracing::error!("cannot start the backup scheduler: {error}");
        }
    }

    /// Stops the scheduler thread.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    /// Starts a copy to every place in the background, for example when a day is closed
    /// (PRD FR-21.1: closing a day starts the cloud copy). Day close itself lives in
    /// `services::day`; the late part of plan 22 calls this from the `day_close` command.
    pub fn run_in_background(&self) {
        let service = Arc::clone(&self.service);
        let spawned = std::thread::Builder::new()
            .name("mandi-backup-now".to_owned())
            .spawn(move || {
                if let Err(error) = service.run(Scope::ALL) {
                    tracing::warn!("backup after day close: {error}");
                }
            });
        if let Err(error) = spawned {
            tracing::error!("cannot start the backup: {error}");
        }
    }
}

/// What the screen hears while the copy at exit runs.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ExitProgress {
    /// `running` while the copy is made, then `done`.
    pub phase: &'static str,
}

/// Makes the copy at exit (FR-20.3) when the main window is closed: the first close request is
/// held, the copy runs on a thread (the screen shows progress from the `backup-exit` event), and
/// the window closes when the copy is done or after two minutes, whichever comes first.
pub fn hold_close_for_backup<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    window: &tauri::WebviewWindow<R>,
) {
    let done = Arc::new(AtomicBool::new(false));
    let handle = app.clone();
    let closing = window.clone();
    window.on_window_event(move |event| {
        let tauri::WindowEvent::CloseRequested { api, .. } = event else {
            return;
        };
        if done.swap(true, Ordering::SeqCst) {
            return;
        }
        let Ok(runtime) = BackupRuntime::of(&handle) else {
            return;
        };
        api.prevent_close();
        runtime.stop();
        let app = handle.clone();
        let window = closing.clone();
        let worker = std::thread::Builder::new()
            .name("mandi-backup-exit".to_owned())
            .spawn(move || {
                app.emit(EXIT_EVENT, ExitProgress { phase: "running" }).ok();
                let (tx, rx) = mpsc::channel();
                let service = Arc::clone(runtime.service());
                let copy = std::thread::spawn(move || {
                    let result = service.run_on_exit();
                    tx.send(()).ok();
                    result
                });
                if rx.recv_timeout(EXIT_WAIT).is_err() {
                    tracing::warn!("the copy at exit took too long, closing anyway");
                } else if let Ok(Err(error)) = copy.join() {
                    tracing::warn!("copy at exit: {error}");
                }
                app.emit(EXIT_EVENT, ExitProgress { phase: "done" }).ok();
                window.close().ok();
            });
        if let Err(error) = worker {
            tracing::error!("cannot start the copy at exit: {error}");
            closing.close().ok();
        }
    });
}
