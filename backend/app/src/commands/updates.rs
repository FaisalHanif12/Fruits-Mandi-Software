//! Updates (plan 22 early, PRD NFR-INS-3, NFR-INS-4, NFR-SEC-11, NFR-REL-13). The work is in
//! `crate::updates`; a command checks sign-in, then calls it. An update is only looked for when
//! the Owner asks, and only installed when he agrees, never during work, after a good copy of
//! the books, and only if it is signed by us. Both commands answer with the backup error shape
//! (`BackupWireError`).

use mandi_backup::error::{BackupCode, BackupError};
use mandi_backup::model::BackupKind;
use mandi_backup::run::Scope;
use mandi_services::auth;

use super::backup::{BackupWireError, service_of};
use crate::error::UiError;
use crate::state::AppState;
use crate::updates::{self, UpdateCheck, UpdateInstallInput, UpdateInstalled};

/// Looks for a newer version now (needs the internet) and says what is new in it, in the release
/// notes (`docs/release/whats-new.md`, Urdu then English). Changes nothing. Errors:
/// `ERR_UPDATE_CHECK_FAILED` (no internet, or the server answered with something else).
#[tauri::command]
pub async fn updates_check<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<UpdateCheck, BackupWireError> {
    let state = AppState::of(&app).map_err(BackupWireError::from)?;
    let ctx = state.context().map_err(BackupWireError::from)?;
    auth::require(&ctx).map_err(|e| BackupWireError::from(UiError::from(e)))?;
    updates::check(&app, &ctx)
        .await
        .map_err(BackupWireError::from)
}

/// Installs the update the Owner agreed to. In this order, each step able to stop it: the Owner
/// agreed, no entry is waiting unsaved, the offer is the version he saw, the package is
/// downloaded and its signature checked, a full copy of the books is taken and checked, and only
/// then the installer runs (on Windows it closes the app). Errors: `ERR_UPDATE_NOT_CONFIRMED`
/// (field `confirmed`), `ERR_UPDATE_DURING_WORK` (value `drafts`), `ERR_UPDATE_NONE`,
/// `ERR_UPDATE_CHANGED` (field `expected_version`), `ERR_UPDATE_NOT_SIGNED` (nothing is
/// installed, no copy is taken), `ERR_UPDATE_CHECK_FAILED`, `ERR_UPDATE_BACKUP_FAILED`.
#[tauri::command]
pub async fn updates_install<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: UpdateInstallInput,
) -> Result<UpdateInstalled, BackupWireError> {
    let state = AppState::of(&app).map_err(BackupWireError::from)?;
    let ctx = state.context().map_err(BackupWireError::from)?;
    auth::require(&ctx).map_err(|e| BackupWireError::from(UiError::from(e)))?;
    let service = service_of(&app).map_err(BackupWireError::from)?;
    let take_backup = move || -> Result<Option<String>, BackupError> {
        let report = service
            .run(Scope {
                local: true,
                usb: true,
                cloud: true,
            })
            .map_err(|e| {
                BackupError::new(BackupCode::ErrUpdateBackupFailed, format!("the copy: {e}"))
            })?;
        // The local copy must be good. A USB or cloud copy that cannot be made (no drive, no
        // internet) is queued or skipped and does not stop the update.
        match report
            .results
            .iter()
            .find(|r| r.kind == BackupKind::Local && r.ok)
        {
            Some(local) => Ok(local.file.clone()),
            None => Err(BackupError::new(
                BackupCode::ErrUpdateBackupFailed,
                "no good local copy",
            )),
        }
    };
    updates::install_flow(&app, &ctx, &input, take_backup, |update, bytes| {
        update.install(bytes)
    })
    .await
    .map_err(BackupWireError::from)
}
