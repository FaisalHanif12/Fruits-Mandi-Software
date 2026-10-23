//! The daily health check (plan 22 early, NFR-REL-8, NFR-OBS-5): the books balance, the hash
//! chains match, the data file passed its check, and when the last good backups were made. It is
//! run by itself once a day ([`start_daily`]) and when the Owner asks; the latest result is kept
//! and shown without waiting for a new run.
//!
//! It reads only. The result holds counts, dates, yes/no states and the trial balance
//! DIFFERENCE (not any balance); a problem also writes one audit row (`check_failed`).

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use mandi_backup::api::{BackupStatus, CloudLevel};
use mandi_services::core::money::Paisa;
use mandi_services::health::{self, BooksCheck, ChainState};
use mandi_services::{Context, ServiceError, auth};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::run;
use crate::backup_runtime::BackupRuntime;
use crate::error::UiError;
use crate::state::{AppState, StatusName};

/// How a hash chain stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ChainHealth {
    /// True when every hash matches (NFR-SEC-6).
    pub ok: bool,
    /// The first bad entry, when it does not match.
    pub at: Option<String>,
}

/// The books part of the check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct BooksHealth {
    /// False when the books check could not run at all (for example a line that is not a whole
    /// rupee). Then the other fields are empty and the result is not ok.
    pub ran: bool,
    /// True when the trial balance difference is 0 (NFR-REL-8).
    pub balanced: bool,
    /// Debit total less credit total. 0 when the books balance.
    #[ts(type = "Paisa")]
    pub difference_paisa: Paisa,
    /// The journal chain.
    pub journal_chain: ChainHealth,
    /// The audit chain.
    pub audit_chain: ChainHealth,
    /// Posted journal entries.
    #[ts(type = "number")]
    pub journal_entries: i64,
    /// Audit rows.
    #[ts(type = "number")]
    pub audit_rows: i64,
}

/// The start-up check of the data file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum DataHealthState {
    /// The check is still running.
    Checking,
    /// The file passed.
    Ok,
    /// The file is damaged or could not be opened.
    Failed,
}

/// The data check part.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct DataHealth {
    /// The state.
    pub state: DataHealthState,
    /// The reference of the failure, when it failed.
    pub reference: Option<String>,
}

/// The backup part: when the last good copies were made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct BackupHealth {
    /// The last good local copy (UTC).
    pub last_local_at: Option<String>,
    /// The last good USB copy (UTC).
    pub last_usb_at: Option<String>,
    /// The last good cloud copy (UTC).
    pub last_cloud_at: Option<String>,
    /// Whole hours since the last good cloud copy.
    #[ts(type = "number | null")]
    pub cloud_age_hours: Option<i64>,
    /// The colour state of the cloud copy (NFR-BAK-10).
    pub cloud_level: CloudLevel,
    /// Cloud copies waiting for the internet.
    #[ts(type = "number")]
    pub queue_len: i64,
    /// Problems the backups reported.
    #[ts(type = "number")]
    pub errors: i64,
}

/// The whole health report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct DailyHealth {
    /// When the check ran (UTC).
    pub checked_at: String,
    /// True when the books are right, the data file is good and the cloud copy is not in alert.
    pub ok: bool,
    /// The books.
    pub books: BooksHealth,
    /// The data file.
    pub data: DataHealth,
    /// The backups. Null when the backup service is not running.
    pub backup: Option<BackupHealth>,
}

fn chain(state: &ChainState) -> ChainHealth {
    match state {
        ChainState::Ok => ChainHealth { ok: true, at: None },
        ChainState::Broken(at) => ChainHealth {
            ok: false,
            at: Some(at.clone()),
        },
    }
}

fn backup_part(status: &BackupStatus) -> BackupHealth {
    BackupHealth {
        last_local_at: status.last_local_at.clone(),
        last_usb_at: status.last_usb_at.clone(),
        last_cloud_at: status.last_cloud_at.clone(),
        cloud_age_hours: status.cloud_age_hours,
        cloud_level: status.cloud_level,
        queue_len: status.queue_len,
        errors: i64::try_from(status.errors.len()).unwrap_or(i64::MAX),
    }
}

/// The books part of the report. A check that cannot run is a failed result too (`ran` false,
/// nothing ok), never an old "ok" left on screen.
fn books_part(books: &Result<BooksCheck, ServiceError>) -> BooksHealth {
    match books {
        Ok(found) => BooksHealth {
            ran: true,
            balanced: found.difference == Paisa::ZERO,
            difference_paisa: found.difference,
            journal_chain: chain(&found.journal_chain),
            audit_chain: chain(&found.audit_chain),
            journal_entries: found.journal_entries,
            audit_rows: found.audit_rows,
        },
        Err(_) => BooksHealth {
            ran: false,
            balanced: false,
            difference_paisa: Paisa::ZERO,
            journal_chain: ChainHealth {
                ok: false,
                at: None,
            },
            audit_chain: ChainHealth {
                ok: false,
                at: None,
            },
            journal_entries: 0,
            audit_rows: 0,
        },
    }
}

/// Runs the check and keeps the result. `user_id` is the Owner when he asked, none for the
/// daily run.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data`.
pub fn build_and_store(
    ctx: &Context,
    state: &AppState,
    backup: Option<&BackupStatus>,
    user_id: Option<i64>,
) -> Result<DailyHealth, ServiceError> {
    let books = health::check_books(ctx);
    let data = state.data_status();
    let data_state = match data.state {
        StatusName::Checking => DataHealthState::Checking,
        StatusName::Ok => DataHealthState::Ok,
        StatusName::Failed => DataHealthState::Failed,
    };
    let backup = backup.map(backup_part);
    let books_ok = books.as_ref().is_ok_and(BooksCheck::is_ok);
    let ok = books_ok
        && data_state != DataHealthState::Failed
        && backup
            .as_ref()
            .is_none_or(|b| b.cloud_level != CloudLevel::Alert);
    let report = DailyHealth {
        checked_at: ctx.now_text(),
        ok,
        books: books_part(&books),
        data: DataHealth {
            state: data_state,
            reference: data.reference,
        },
        backup,
    };
    let json = serde_json::to_value(&report).map_err(|e| ServiceError::Data(e.to_string()))?;
    match &books {
        Ok(found) => health::store_last(ctx, &json, found, user_id)?,
        Err(error) => {
            // The reason goes to the log (it holds no balance); the audit row holds none.
            tracing::warn!("the books check could not run: {error}");
            health::store_failed(ctx, &json, user_id)?;
        }
    }
    Ok(report)
}

/// The backup status, when the backup service is running.
fn backup_status<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Option<BackupStatus> {
    BackupRuntime::of(app)
        .ok()
        .and_then(|runtime| runtime.service().status().ok())
}

/// Runs the health check now: the trial balance difference, the journal and audit chains, the
/// data check, and when the last good copies were made. Keeps the result as the latest. Read
/// only for the books.
#[tauri::command]
pub async fn health_run<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<DailyHealth, UiError> {
    let state = AppState::of(&app)?;
    let backup = backup_status(&app);
    run(app, "health_run", move |ctx| {
        let actor = auth::require(ctx)?;
        build_and_store(ctx, &state, backup.as_ref(), Some(actor.user_id))
    })
    .await
}

/// The latest result, without running a new check. Null when no check has run yet. Read only.
#[tauri::command]
pub async fn health_last<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<DailyHealth>, UiError> {
    run(app, "health_last", |ctx| {
        auth::require(ctx)?;
        match health::last(ctx)? {
            None => Ok(None),
            Some(value) => serde_json::from_value(value)
                .map(Some)
                .map_err(|e| ServiceError::Data(format!("the stored health result: {e}"))),
        }
    })
    .await
}

/// How old the latest result may be before the daily run makes a new one.
const MAX_AGE_HOURS: i64 = 20;
/// How long after start-up the first check waits, so the data check can finish first.
const FIRST_DELAY: Duration = Duration::from_secs(120);
/// How often the daily run asks whether a check is due.
const TICK: Duration = Duration::from_secs(30 * 60);

/// True when a new check is due: none has run, or the latest is `MAX_AGE_HOURS` old or more.
pub fn is_due(last_checked_at: Option<&str>, now: DateTime<Utc>) -> bool {
    match last_checked_at.and_then(|t| DateTime::parse_from_rfc3339(t).ok()) {
        None => true,
        Some(at) => (now - at.with_timezone(&Utc)).num_hours() >= MAX_AGE_HOURS,
    }
}

/// One daily run: skipped if the data file did not open, or if a check is not due.
fn run_if_due<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let Ok(state) = AppState::of(app) else { return };
    let Ok(ctx) = state.context() else { return };
    let ctx: Arc<Context> = ctx;
    let last = health::last(&ctx).ok().flatten();
    let last_at = last
        .as_ref()
        .and_then(|v| v.get("checked_at"))
        .and_then(serde_json::Value::as_str);
    if !is_due(last_at, ctx.now_utc()) {
        return;
    }
    match build_and_store(&ctx, &state, backup_status(app).as_ref(), None) {
        Ok(report) if !report.ok => {
            tracing::error!(
                balanced = report.books.balanced,
                journal_chain_ok = report.books.journal_chain.ok,
                audit_chain_ok = report.books.audit_chain.ok,
                "the daily health check found a problem"
            );
        }
        Ok(_) => tracing::info!("the daily health check passed"),
        Err(error) => tracing::warn!("the daily health check could not run: {error}"),
    }
}

/// Starts the daily check in a background thread: the first run two minutes after start-up, then
/// whenever the latest result is 20 hours old or more (NFR-REL-8). It never stops the app.
pub fn start_daily<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("health-daily".to_owned())
        .spawn(move || {
            std::thread::sleep(FIRST_DELAY);
            loop {
                run_if_due(&app);
                std::thread::sleep(TICK);
            }
        });
    if let Err(error) = spawned {
        tracing::warn!("the daily health check thread did not start: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn a_check_that_could_not_run_is_a_failed_result() {
        let failed = books_part(&Err(ServiceError::Data("x".to_owned())));
        assert!(!failed.ran && !failed.balanced);
        assert!(!failed.journal_chain.ok && !failed.audit_chain.ok);
        let found = books_part(&Ok(BooksCheck {
            debit: Paisa::ZERO,
            credit: Paisa::ZERO,
            difference: Paisa::ZERO,
            journal_chain: ChainState::Ok,
            audit_chain: ChainState::Ok,
            journal_entries: 2,
            audit_rows: 5,
        }));
        assert!(found.ran && found.balanced && found.journal_chain.ok);
        assert_eq!((found.journal_entries, found.audit_rows), (2, 5));
    }

    #[test]
    fn a_check_is_due_when_none_ran_or_the_last_is_old() {
        let now = Utc.with_ymd_and_hms(2026, 10, 10, 12, 0, 0).unwrap();
        assert!(is_due(None, now));
        assert!(is_due(Some("not a time"), now));
        assert!(!is_due(Some("2026-10-10T00:00:00.000Z"), now));
        assert!(!is_due(Some("2026-10-09T17:00:00.000Z"), now));
        assert!(is_due(Some("2026-10-09T16:00:00.000Z"), now));
    }
}
