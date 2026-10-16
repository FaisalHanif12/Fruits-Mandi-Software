//! The state every command can reach: the open data file, or why it could not be opened
//! (Tech Stack 5.4, plan 03 Sections 4.3 and 4.6).
//!
//! No failure at start-up closes the app. If the data file is damaged or an upgrade failed,
//! the state keeps the error with its reference number. Every command that needs the data
//! returns that error, and `app_data_status` tells the screen so it can show the message
//! (NFR-REL-6, NFR-REL-7).

use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use mandi_services::Context;
use mandi_services::db::{DataState, Db, DbError, DbOptions};
use serde::Serialize;
use tauri::Manager;
use ts_rs::TS;

use crate::error::UiError;

/// Where the start-up check of the data file stands (plan 03 Section 4.2b).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DataStatus {
    /// `checking` while the check runs, `ok` when it passed, `failed` when the file is
    /// damaged or could not be opened or upgraded.
    #[ts(type = "'checking' | 'ok' | 'failed'")]
    pub state: StatusName,
    /// The reference number of the failure, also written in the log. Null unless `failed`.
    pub reference: Option<String>,
}

/// The three words of [`DataStatus::state`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusName {
    /// The check is running.
    Checking,
    /// The check passed.
    Ok,
    /// The file failed.
    Failed,
}

/// The shared state of the running app. Cheap to clone: the parts are shared.
#[derive(Clone)]
pub struct AppState {
    ctx: Result<Arc<Context>, UiError>,
    check_failure: Arc<Mutex<Option<String>>>,
}

impl AppState {
    /// Opens `<data_dir>/data/mandi.db` (copies go to `<data_dir>/backups`), upgrades it if
    /// needed, and starts the start-up check on a background thread. Never fails: a failure
    /// is kept in the state.
    pub fn start(data_dir: &Path) -> Self {
        let path = data_dir.join("data").join("mandi.db");
        let options = DbOptions::new(data_dir.join("backups"));
        match Db::open(&path, options) {
            Ok(db) => Self::with_db(Arc::new(db)),
            Err(error) => Self {
                ctx: Err(UiError::from(error)),
                check_failure: Arc::default(),
            },
        }
    }

    /// State for an already open database, with the real parts, and starts its check.
    pub fn with_db(db: Arc<Db>) -> Self {
        Self::with_context(Arc::new(Context::new(db)))
    }

    /// State for a ready context (tests give one with fake parts), and starts its check.
    pub fn with_context(ctx: Arc<Context>) -> Self {
        let check_failure: Arc<Mutex<Option<String>>> = Arc::default();
        let worker = {
            let db = Arc::clone(ctx.db());
            let slot = Arc::clone(&check_failure);
            move || check_and_note(&db, &slot)
        };
        if let Err(error) = std::thread::Builder::new()
            .name("mandi-startup-check".to_owned())
            .spawn(worker.clone())
        {
            tracing::warn!("cannot start the check thread, running it now: {error}");
            worker();
        }
        // Train the name suggestions now, so the first key press in an Urdu field is not the
        // slow one (NFR-SPD-20), and build the search index. A failure here is not an error: the
        // first use tries again.
        let warm = Arc::clone(&ctx);
        if let Err(error) = std::thread::Builder::new()
            .name("mandi-translit-warm-up".to_owned())
            .spawn(move || {
                mandi_services::translit::warm_up(&warm);
                // Build the search index too (NFR-SPD-6, Code Standards B27).
                mandi_services::search::warm_up(&warm);
            })
        {
            tracing::warn!("cannot start the warm-up thread: {error}");
        }
        Self {
            ctx: Ok(ctx),
            check_failure,
        }
    }

    /// The state the running app manages, as a copy a worker thread can own.
    ///
    /// # Errors
    /// `ERR_INTERNAL` if the app forgot to manage a state.
    pub fn of<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<Self, UiError> {
        app.try_state::<AppState>()
            .map(|state| state.inner().clone())
            .ok_or_else(|| UiError::internal("the app has no state"))
    }

    /// The services context, or the error that stopped the data file opening (the same
    /// reference every time).
    ///
    /// # Errors
    /// The start-up `UiError` when the file could not be opened or upgraded.
    pub fn context(&self) -> Result<Arc<Context>, UiError> {
        self.ctx.clone()
    }

    /// The data file, or the start-up error.
    ///
    /// # Errors
    /// As [`AppState::context`].
    pub fn db(&self) -> Result<Arc<Db>, UiError> {
        self.ctx.clone().map(|ctx| Arc::clone(ctx.db()))
    }

    /// What `app_data_status` returns.
    pub fn data_status(&self) -> DataStatus {
        match &self.ctx {
            Err(error) => DataStatus {
                state: StatusName::Failed,
                reference: Some(error.reference.clone()),
            },
            Ok(ctx) => match ctx.db().data_state() {
                DataState::Checking => DataStatus {
                    state: StatusName::Checking,
                    reference: None,
                },
                DataState::Ok => DataStatus {
                    state: StatusName::Ok,
                    reference: None,
                },
                // The check thread sets `Failed` a moment before it notes the reference. Asking
                // here makes the reference at once if it is not there yet, so `failed` never
                // goes out without a number that is also in the log (NFR-REL-6, NFR-OBS-3).
                DataState::Failed => DataStatus {
                    state: StatusName::Failed,
                    reference: Some(failure_reference(ctx.db(), &self.check_failure)),
                },
            },
        }
    }
}

/// The reference number of the failed check. The first caller makes it (and writes the log
/// line); every later caller gets the same one.
fn failure_reference(db: &Db, slot: &Mutex<Option<String>>) -> String {
    let mut held = slot.lock().unwrap_or_else(PoisonError::into_inner);
    held.get_or_insert_with(|| {
        let detail = db.failure_detail().unwrap_or_default();
        UiError::from(DbError::DataCheckFailed(detail)).reference
    })
    .clone()
}

/// Runs the check. On failure it makes the reference number (and the log line) right away.
fn check_and_note(db: &Db, slot: &Mutex<Option<String>>) {
    if db.run_quick_check() == DataState::Failed {
        failure_reference(db, slot);
    }
}
