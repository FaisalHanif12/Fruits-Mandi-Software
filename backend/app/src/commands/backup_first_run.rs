//! Restore on a new computer, from the first screen (plan 56, PRD FR-20.11, NFR-INS-6).
//!
//! These three commands run BEFORE sign-in, because no Owner exists on the new computer. They
//! are allowed only in that state: the service refuses each of them with `ERR_ALREADY_SET_UP`
//! once any user exists, like `setup_first_run` (NFR-SEC-4). They keep nothing between calls, so
//! every call carries the secrets it needs (the backup key, the cloud keys). A secret is never
//! logged: the input types print `[hidden]`.
//!
//! Errors use the field names of the input: `source.cloud.endpoint`, `source.cloud.bucket`,
//! `source.cloud.region`, `source.cloud.prefix`, `source.cloud.key_id`, `source.cloud.secret`,
//! `source.folder`, `source.file`, `copy`, `key`.
//!
//! Progress goes to the screen as the event `backup-restore-progress`.

use mandi_backup::api::{
    FirstRunCopies, FirstRunListInput, FirstRunPreview, FirstRunPreviewInput, FirstRunRestoreDone,
    FirstRunRestoreInput, RestoreProgress, RestoreStage,
};
use tauri::Emitter;

use super::backup::{BackupWireError, run_backup, service_of};

/// The event the first screen listens to while a copy is checked or restored.
pub const PROGRESS_EVENT: &str = "backup-restore-progress";

/// Lists the copies in a cloud account, a folder (for example a USB drive) or one file, newest
/// first. An empty place is an empty list, not an error.
///
/// Errors: `ERR_ALREADY_SET_UP`; `ERR_REQUIRED` / `ERR_OUT_OF_RANGE` on the fields of the source,
/// all together; `ERR_BACKUP_CLOUD_FAILED` (value `reason`: `refused` marks `source.cloud.key_id`
/// and `source.cloud.secret`, `not_found` marks `source.cloud.bucket`, `offline` marks
/// `source.cloud.endpoint`); `ERR_BACKUP_USB_NOT_PRESENT` on `source.folder` or `source.file`;
/// `ERR_BACKUP_BUSY`.
#[tauri::command]
pub async fn backup_first_run_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FirstRunListInput,
) -> Result<FirstRunCopies, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_first_run_list", move |_ctx| {
        Ok(service_of(&handle)?.first_run_list(&input)?)
    })
    .await
}

/// Fetches one copy, unlocks it, checks it (it opens, its checksum, its integrity, its counts,
/// the books balance, both hash chains verify) and returns its date and its books. Changes
/// nothing on this computer. With no copy named it takes the newest that is not marked bad.
///
/// Errors: those of the list, and `ERR_BACKUP_KEY_FORMAT` / `ERR_BACKUP_KEY_WRONG` (field `key`),
/// `ERR_BACKUP_NOT_FOUND`, `ERR_BACKUP_CORRUPT` (value `check`), `ERR_BACKUP_NEWER_THAN_APP`,
/// `ERR_BACKUP_CHAIN_KEY_MISSING` (all on `copy`), `ERR_DISK_FULL`, `ERR_MIGRATION_FAILED`.
#[tauri::command]
pub async fn backup_first_run_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FirstRunPreviewInput,
) -> Result<FirstRunPreview, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_first_run_preview", move |_ctx| {
        let emit = |progress: RestoreProgress| {
            handle.emit(PROGRESS_EVENT, progress).ok();
        };
        Ok(service_of(&handle)?.first_run_preview(&input, &emit)?)
    })
    .await
}

/// Restores a copy on this new computer. The Owner is not signed in afterwards: the screen goes
/// to the sign-in page. If anything fails, this computer is still a fresh install.
///
/// Errors: those of the preview, and `ERR_BACKUP_RESTORE_FAILED`.
#[tauri::command]
pub async fn backup_first_run_restore<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FirstRunRestoreInput,
) -> Result<FirstRunRestoreDone, BackupWireError> {
    let handle = app.clone();
    run_backup(app, "backup_first_run_restore", move |ctx| {
        let emit = |progress: RestoreProgress| {
            handle.emit(PROGRESS_EVENT, progress).ok();
        };
        let result = service_of(&handle)?.first_run_restore(&input, &emit);
        // The data file was replaced behind the services' back, or put back: either way, forget
        // what they remember.
        ctx.invalidate_caches();
        let done = result?;
        emit(RestoreProgress {
            stage: RestoreStage::SearchIndex,
            percent: None,
        });
        mandi_services::search::warm_up(ctx);
        emit(RestoreProgress {
            stage: RestoreStage::Done,
            percent: None,
        });
        Ok(done)
    })
    .await
}
