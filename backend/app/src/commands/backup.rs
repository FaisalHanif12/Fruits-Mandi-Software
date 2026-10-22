//! Backup and restore (plan 18, PRD 20, NFR 4). Every command needs the signed-in Owner
//! (NFR-SEC-4). The work is in `mandi-backup`; a command checks sign-in, then calls one function.
//!
//! The backup codes are `mandi-backup`'s own list (`BackupCode`), because `mandi-core`'s list
//! belongs to the other run for now. A failure sends the same JSON shape as every other command
//! (`ok`, `errors`, `ref`), and each `code` is a text the screen turns into words through
//! `errors.ftl`. The code is one of `ErrorCode` (sign-in problems, for example) or `BackupCode`.

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

use mandi_backup::BackupCode;
use mandi_backup::BackupError;
use mandi_backup::api::{
    BackupList, BackupListInput, BackupNowInput, BackupSettingsInput, BackupSettingsView,
    BackupStatus, KeyConfirmInput, KeyCreated, RestoreDone, RestoreInput, RestorePreview,
    RestorePreviewInput, RunReport, TestRestoreInput, TestRestoreResult,
};
use mandi_backup::run::Scope;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::{Context, ServiceError, auth};
use serde::Serialize;
use ts_rs::TS;

use crate::backup_runtime::BackupRuntime;
use crate::error::{NotOk, UiError};
use crate::refs::next_reference;
use crate::state::AppState;

/// A code on the wire: one of the core codes or one of the backup codes. Both are the text
/// `ERR_...`, so the screen reads them the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(untagged)]
pub enum WireCode {
    /// A code of `mandi-core` (sign-in, internal failures).
    Core(ErrorCode),
    /// A code of `mandi-backup`.
    Backup(BackupCode),
}

/// One problem of a backup command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BackupErrorItem {
    /// The code.
    pub code: WireCode,
    /// The field it belongs to, for example `key` or `settings.closing_time`.
    pub field: Option<String>,
    /// Values for the message.
    #[ts(type = "Record<string, string | number>")]
    pub values: BTreeMap<String, ErrorValue>,
}

/// The error of a backup command. The same shape as `UiError`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BackupWireError {
    /// Always `false`.
    #[ts(type = "false")]
    pub ok: NotOk,
    /// One or more problems.
    pub errors: Vec<BackupErrorItem>,
    /// Reference number, also in the log (NFR-OBS-3).
    #[serde(rename = "ref")]
    pub reference: String,
}

impl From<UiError> for BackupWireError {
    fn from(error: UiError) -> Self {
        Self {
            ok: NotOk,
            errors: error
                .errors
                .into_iter()
                .map(|item| BackupErrorItem {
                    code: WireCode::Core(item.code),
                    field: item.field,
                    values: item.values,
                })
                .collect(),
            reference: error.reference,
        }
    }
}

impl From<BackupError> for BackupWireError {
    fn from(error: BackupError) -> Self {
        let reference = next_reference();
        let items = error.all();
        let codes: Vec<String> = items.iter().map(|e| format!("{:?}", e.code)).collect();
        for item in &items {
            // The cause goes to the log with the reference, never to the screen (B12).
            tracing::warn!(reference = %reference, code = ?item.code, detail = %item.detail, "backup command failed");
        }
        tracing::warn!(reference = %reference, codes = %codes.join(","), "backup command returned an error");
        Self {
            ok: NotOk,
            errors: items
                .into_iter()
                .map(|item| BackupErrorItem {
                    code: WireCode::Backup(item.code),
                    field: item.field,
                    values: item.values,
                })
                .collect(),
            reference,
        }
    }
}

/// What a backup command body can fail with.
#[derive(Debug)]
pub enum CmdError {
    /// A service failure, for example no sign-in.
    Service(ServiceError),
    /// A backup failure.
    Backup(BackupError),
    /// An app failure, for example no backup service.
    Ui(UiError),
}

impl From<ServiceError> for CmdError {
    fn from(error: ServiceError) -> Self {
        Self::Service(error)
    }
}

impl From<BackupError> for CmdError {
    fn from(error: BackupError) -> Self {
        Self::Backup(error)
    }
}

impl From<UiError> for CmdError {
    fn from(error: UiError) -> Self {
        Self::Ui(error)
    }
}

impl From<CmdError> for BackupWireError {
    fn from(error: CmdError) -> Self {
        match error {
            CmdError::Service(e) => UiError::from(e).into(),
            CmdError::Backup(e) => e.into(),
            CmdError::Ui(e) => e.into(),
        }
    }
}

/// Runs one command body on a worker thread with the services context, like `run`, but with the
/// backup error shape. The body does the sign-in check first (`auth::require`).
async fn run_backup<R, T, F>(
    app: tauri::AppHandle<R>,
    name: &'static str,
    body: F,
) -> Result<T, BackupWireError>
where
    R: tauri::Runtime,
    T: Send + 'static,
    F: FnOnce(&Context) -> Result<T, CmdError> + Send + 'static,
{
    let state = AppState::of(&app).map_err(BackupWireError::from)?;
    let joined = tauri::async_runtime::spawn_blocking(move || {
        catch_unwind(AssertUnwindSafe(|| {
            let ctx = state.context().map_err(BackupWireError::from)?;
            body(&ctx).map_err(BackupWireError::from)
        }))
        .unwrap_or_else(|_| Err(UiError::internal(format!("command {name} panicked")).into()))
    })
    .await;
    match joined {
        Ok(result) => result,
        Err(error) => {
            Err(UiError::internal(format!("command {name} did not finish: {error}")).into())
        }
    }
}

fn service_of<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<std::sync::Arc<mandi_backup::service::BackupService>, CmdError> {
    BackupRuntime::of(app)
        .map(|runtime| std::sync::Arc::clone(runtime.service()))
        .map_err(|_| BackupError::new(BackupCode::ErrBackupNotStarted, "no backup service").into())
}

/// The state of the backups: the key, the last good copies, the cloud colour, the queue, the
/// problems (PRD FR-20.7, NFR-BAK-10, NFR-BAK-11). Reads only; no network.
#[tauri::command]
pub async fn backup_status<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<BackupStatus, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_status", move |ctx| {
        auth::require(ctx)?;
        Ok(service_of(&handle)?.status()?)
    })
    .await
}

/// The copies in the local folder, on the USB drive and in the cloud, newest first, one page
/// (FR-20.8, Code Standards B31). Errors: none the user fixes; an unreachable cloud shows as
/// `cloud_reachable = false`.
#[tauri::command]
pub async fn backup_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BackupListInput,
) -> Result<BackupList, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_list", move |ctx| {
        auth::require(ctx)?;
        Ok(service_of(&handle)?.list(&input)?)
    })
    .await
}

/// "Back up now" (FR-20.8): copies to the places asked, then tries the cloud copies that wait.
/// Returns one result for each place tried; a failure of one place is a result, not an error.
/// Errors: `ERR_BACKUP_BUSY`.
#[tauri::command]
pub async fn backup_now<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BackupNowInput,
) -> Result<RunReport, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_now", move |ctx| {
        auth::require(ctx)?;
        let service = service_of(&handle)?;
        if input.cloud {
            // Copies that wait for the internet go first. Busy or offline is not an error here.
            if let Err(error) = service.retry_queue() {
                tracing::debug!("queue retry before backup now: {error}");
            }
        }
        Ok(service.run(Scope {
            local: input.local,
            usb: input.usb,
            cloud: input.cloud,
        })?)
    })
    .await
}

/// What a restore of one copy would do: its date, its books, the books now, and what would be
/// lost (FR-20.9). Changes nothing. Errors: `ERR_BACKUP_NOT_FOUND`, `ERR_BACKUP_KEY_REQUIRED`,
/// `ERR_BACKUP_KEY_FORMAT`, `ERR_BACKUP_KEY_WRONG` (field `key`), `ERR_BACKUP_CORRUPT`,
/// `ERR_BACKUP_NEWER_THAN_APP`, `ERR_BACKUP_CLOUD_FAILED`, `ERR_BACKUP_USB_NOT_PRESENT`,
/// `ERR_BACKUP_BUSY`, `ERR_OUT_OF_RANGE` (field `name`).
#[tauri::command]
pub async fn backup_restore_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: RestorePreviewInput,
) -> Result<RestorePreview, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_restore_preview", move |ctx| {
        auth::require(ctx)?;
        Ok(service_of(&handle)?.restore_preview(&input)?)
    })
    .await
}

/// Restores a copy over the live data (FR-20.9): a safety copy of the data first, then the
/// restore. Afterwards the search index is rebuilt and the Owner is signed out, because the users
/// come from the restored data: the screen goes to the sign-in. Errors as the preview, and
/// `ERR_BACKUP_NOT_CONFIRMED` (field `confirmed`), `ERR_DISK_FULL`, `ERR_BACKUP_CHECK_FAILED`,
/// `ERR_BACKUP_RESTORE_FAILED`. If anything fails the data is as it was.
#[tauri::command]
pub async fn backup_restore<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: RestoreInput,
) -> Result<RestoreDone, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_restore", move |ctx| {
        let actor = auth::require(ctx)?;
        let service = service_of(&handle)?;
        let done = service.restore(Some(actor.user_id), &input)?;
        // The data was replaced behind the services' back: throw away what they remember, sign
        // out (the session belongs to the old data), and build the search index again.
        ctx.invalidate_caches();
        if let Err(error) = auth::logout(ctx) {
            tracing::debug!("sign-out after restore: {error:?}");
        }
        if let Ok(state) = AppState::of(&handle)
            && let Ok(shared) = state.context()
        {
            let spawned = std::thread::Builder::new()
                .name("mandi-search-rebuild".to_owned())
                .spawn(move || mandi_services::search::warm_up(&shared));
            if let Err(error) = spawned {
                tracing::warn!("cannot rebuild the search index in the background: {error}");
            }
        }
        Ok(done)
    })
    .await
}

/// Restores a copy into a separate scratch place and checks it (NFR-BAK-11). The live data is
/// not touched. With no copy named it takes the newest cloud copy, else USB, else local. A
/// damaged copy is a result with `ok = false`, not an error. Errors: `ERR_BACKUP_NOT_FOUND`,
/// the key and cloud codes of the preview, `ERR_BACKUP_BUSY`.
#[tauri::command]
pub async fn backup_test_restore<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: TestRestoreInput,
) -> Result<TestRestoreResult, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_test_restore", move |ctx| {
        auth::require(ctx)?;
        Ok(service_of(&handle)?.test_restore(&input)?)
    })
    .await
}

/// The backup settings and whether the cloud keys are stored. The keys are never sent back.
#[tauri::command]
pub async fn backup_settings_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<BackupSettingsView, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_settings_get", move |ctx| {
        auth::require(ctx)?;
        Ok(service_of(&handle)?.settings_view()?)
    })
    .await
}

/// Saves the backup settings (audited) and the cloud keys (Credential Manager only). A key left
/// null stays; an empty text removes it. Errors: `ERR_REQUIRED` and `ERR_OUT_OF_RANGE`, each on
/// its field (`closing_time`, `local_dir`, `usb_folder`, `cloud_endpoint`, `cloud_bucket`,
/// `cloud_region`, `cloud_prefix`), all together; `ERR_BACKUP_BUSY`.
#[tauri::command]
pub async fn backup_settings_set<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: BackupSettingsInput,
) -> Result<BackupSettingsView, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_settings_set", move |ctx| {
        let actor = auth::require(ctx)?;
        Ok(service_of(&handle)?.settings_set(Some(actor.user_id), &input)?)
    })
    .await
}

/// Makes the backup key and returns it to be shown once and printed (FR-20.10, NFR-BAK-7). It is
/// kept in the Credential Manager only. Errors: `ERR_BACKUP_KEY_ALREADY_SET` once the key is
/// confirmed or locks a USB or cloud copy.
#[tauri::command]
pub async fn backup_key_setup<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<KeyCreated, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_key_setup", move |ctx| {
        let actor = auth::require(ctx)?;
        Ok(service_of(&handle)?.key_setup(Some(actor.user_id))?)
    })
    .await
}

/// Checks the key the Owner typed from the printed sheet and remembers that he has it
/// (NFR-BAK-7). Errors: `ERR_BACKUP_KEY_FORMAT`, `ERR_BACKUP_KEY_WRONG` (field `key`),
/// `ERR_BACKUP_KEY_NOT_SET`.
#[tauri::command]
pub async fn backup_key_confirm<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: KeyConfirmInput,
) -> Result<(), BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_key_confirm", move |ctx| {
        let actor = auth::require(ctx)?;
        Ok(service_of(&handle)?.key_confirm(Some(actor.user_id), &input.key)?)
    })
    .await
}
