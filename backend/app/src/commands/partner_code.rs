//! The partner code (PRD FR-17.11, NFR-SEC-14, plan 44): a 4-digit code that opens the partner
//! section. Every command needs the signed-in Owner. The unlock lives in the app's memory and goes
//! when the user leaves the section, on the idle lock, on logout and on exit. Until plan 44 T2e each
//! command answers `ERR_NOT_IMPLEMENTED`.

use mandi_services::core::partner_code::{
    PartnerCodeChangeInput, PartnerCodeResetInput, PartnerCodeSetInput, PartnerCodeStatus,
    PartnerCodeUnlockInput,
};
use mandi_services::{auth, partner_code};

use super::run;
use crate::error::UiError;

/// Whether a code is set, whether the partner section is open and how many seconds to wait.
/// Read only.
#[tauri::command]
pub async fn partners_code_status<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<PartnerCodeStatus, UiError> {
    run(app, "partners_code_status", |ctx| {
        auth::require(ctx)?;
        partner_code::status(ctx)
    })
    .await
}

/// Sets the first code, proved by the Owner's password. Errors: `ERR_PARTNER_CODE_ALREADY_SET`;
/// `ERR_WRONG_PASSWORD` on `password`; `ERR_PARTNER_CODE_INVALID` (not 4 digits) on `code`;
/// `ERR_PARTNER_CODES_DIFFER` on `code_again`.
#[tauri::command]
pub async fn partners_code_set<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnerCodeSetInput,
) -> Result<PartnerCodeStatus, UiError> {
    run(app, "partners_code_set", move |ctx| {
        let actor = auth::require(ctx)?;
        partner_code::set(ctx, &actor, &input)
    })
    .await
}

/// Changes the code, proved by the old code or the Owner's password. Errors:
/// `ERR_PARTNER_CODE_NOT_SET`; `ERR_PARTNER_CODE_WRONG` (value `tries_left`) or
/// `ERR_LOCKED_WAIT` (value `seconds`) on `old_code_or_password`; `ERR_PARTNER_CODE_INVALID` on
/// `code`; `ERR_PARTNER_CODES_DIFFER` on `code_again`.
#[tauri::command]
pub async fn partners_code_change<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnerCodeChangeInput,
) -> Result<PartnerCodeStatus, UiError> {
    run(app, "partners_code_change", move |ctx| {
        let actor = auth::require(ctx)?;
        partner_code::change(ctx, &actor, &input)
    })
    .await
}

/// Opens the partner section. Errors: `ERR_PARTNER_CODE_NOT_SET`; `ERR_PARTNER_CODE_WRONG` (value
/// `tries_left`) on `code`; `ERR_LOCKED_WAIT` (value `seconds`) after 5 wrong tries (as PRD FR-8.2).
/// Every wrong try is in the audit log.
#[tauri::command]
pub async fn partners_code_unlock<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnerCodeUnlockInput,
) -> Result<PartnerCodeStatus, UiError> {
    run(app, "partners_code_unlock", move |ctx| {
        let actor = auth::require(ctx)?;
        partner_code::unlock(ctx, &actor, &input)
    })
    .await
}

/// Closes the partner section again. Safe to call when it is closed already.
#[tauri::command]
pub async fn partners_code_lock<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<PartnerCodeStatus, UiError> {
    run(app, "partners_code_lock", |ctx| {
        auth::require(ctx)?;
        partner_code::lock(ctx)
    })
    .await
}

/// Sets a new code when the old one is forgotten, proved by the Owner's password or the recovery
/// code (PRD FR-8.5). Errors: `ERR_WRONG_PASSWORD` on `password_or_recovery`;
/// `ERR_PARTNER_CODE_INVALID` on `code`; `ERR_PARTNER_CODES_DIFFER` on `code_again`.
#[tauri::command]
pub async fn partners_code_reset<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PartnerCodeResetInput,
) -> Result<PartnerCodeStatus, UiError> {
    run(app, "partners_code_reset", move |ctx| {
        let actor = auth::require(ctx)?;
        partner_code::reset(ctx, &actor, &input)
    })
    .await
}
