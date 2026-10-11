//! The state every command can reach: the open data file, or why it could not be opened
//! (Tech Stack 5.4, plan 03 Sections 4.3 and 4.6).
//!
//! No failure at start-up closes the app. If the data file is damaged or an upgrade failed,
//! the state keeps the error with its reference number. Every command that needs the data
//! returns that error, and `app_data_status` tells the screen so it can show the message
//! (NFR-REL-6, NFR-REL-7).

use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use mandi_services::db::{DataState, Db, DbError, DbOptions};
use serde::Serialize;
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

/// The shared state of the running app.
pub struct AppState {
    db: Result<Arc<Db>, UiError>,
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
                db: Err(UiError::from(error)),
                check_failure: Arc::default(),
            },
        }
    }

    /// State for an already open database, and starts its start-up check.
    pub fn with_db(db: Arc<Db>) -> Self {
        let check_failure: Arc<Mutex<Option<String>>> = Arc::default();
        let worker = {
            let db = Arc::clone(&db);
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
        Self {
            db: Ok(db),
            check_failure,
        }
    }

    /// The data file, or the error that stopped it opening (same reference every time).
    ///
    /// # Errors
    /// The start-up `UiError` when the file could not be opened or upgraded.
    pub fn db(&self) -> Result<Arc<Db>, UiError> {
        self.db.clone()
    }

    /// What `app_data_status` returns.
    pub fn data_status(&self) -> DataStatus {
        match &self.db {
            Err(error) => DataStatus {
                state: StatusName::Failed,
                reference: Some(error.reference.clone()),
            },
            Ok(db) => match db.data_state() {
                DataState::Checking => DataStatus {
                    state: StatusName::Checking,
                    reference: None,
                },
                DataState::Ok => DataStatus {
                    state: StatusName::Ok,
                    reference: None,
                },
                DataState::Failed => DataStatus {
                    state: StatusName::Failed,
                    reference: self
                        .check_failure
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .clone(),
                },
            },
        }
    }
}

/// Runs the check. On failure it makes the reference number (and the log line) first, and
/// only then is the failure visible, so the screen never shows `failed` without a number.
fn check_and_note(db: &Db, slot: &Mutex<Option<String>>) {
    if db.run_quick_check() == DataState::Failed {
        let detail = db.failure_detail().unwrap_or_default();
        let error = UiError::from(DbError::DataCheckFailed(detail));
        *slot.lock().unwrap_or_else(PoisonError::into_inner) = Some(error.reference);
    }
}
