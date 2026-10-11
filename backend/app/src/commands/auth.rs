//! Sign-in, first run, lock and recovery (plan 04 Section 4, PRD Section 8, FR-2.1 to FR-2.4).
//!
//! All text the user types arrives as text and is checked again here (Code Standards V0).
//! Field names in errors are the property names below, for example `password_repeat`.

use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Hidden, not_implemented};
use crate::error::UiError;
use mandi_services::core::auth::{Language, Role};

/// The signed-in user, as the screens need him (plan 04 Section 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SessionUser {
    /// User id.
    #[ts(type = "number")]
    pub id: i64,
    /// Sign-in name, lower case.
    pub username: String,
    /// Name shown in the top bar.
    pub display_name: String,
    /// What he may do.
    pub role: Role,
    /// His language (his last choice, else the default language of the firm).
    pub language: Language,
    /// His text size in percent: 80 to 150 in steps of 10 (NFR-USE-18). 100 is normal.
    pub text_size: u32,
}

/// The firm as the sign-in screen shows it. All empty before the first run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FirmDisplay {
    /// Firm name in Urdu.
    pub name_ur: String,
    /// Firm name in English.
    pub name_en: String,
    /// Mandi name in Urdu.
    pub mandi_ur: String,
    /// Mandi name in English.
    pub mandi_en: String,
}

/// What the start screen needs to decide between the wizard, the sign-in and the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AuthStatus {
    /// True while no user exists: show the first-run wizard.
    pub first_run: bool,
    /// The signed-in user. Present also while the session is locked.
    pub session: Option<SessionUser>,
    /// True when the session is locked (idle lock): show the unlock overlay (PRD FR-8.4).
    pub locked: bool,
    /// The language last used on this computer, for the sign-in screen (PRD FR-7.30).
    pub login_language: Language,
    /// Firm and mandi names for the sign-in screen.
    pub firm: FirmDisplay,
    /// The app version, for example `0.1.12`.
    pub app_version: String,
}

/// The first-run wizard (PRD NFR-INS-2). Allowed only while no user exists.
#[derive(Clone, Deserialize, TS)]
pub struct FirstRunInput {
    /// Firm name, Urdu letters. Required.
    pub firm_name_ur: String,
    /// Firm name, English letters. Required.
    pub firm_name_en: String,
    /// Mandi name, Urdu letters. Required.
    pub mandi_ur: String,
    /// Mandi name, English letters. Required.
    pub mandi_en: String,
    /// Address. May be empty. Up to 500 characters.
    pub address: String,
    /// Phone. May be empty. `03XXXXXXXXX` or `03XX-XXXXXXX`.
    pub phone: String,
    /// Owner sign-in name: 3 to 32 of `a-z 0-9 . _ -`.
    pub owner_username: String,
    /// Owner name shown in the top bar. Required, up to 100 characters.
    pub owner_display_name: String,
    /// Owner password, at least 6 characters.
    pub password: String,
    /// The password again.
    pub password_repeat: String,
}

impl fmt::Debug for FirstRunInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FirstRunInput")
            .field("owner_username", &self.owner_username)
            .field("password", &Hidden)
            .field("password_repeat", &Hidden)
            .finish_non_exhaustive()
    }
}

/// The Owner's recovery code, shown once to print (NFR-SEC-12).
#[derive(Clone, PartialEq, Eq, Serialize, TS)]
pub struct FirstRunDone {
    /// `XXXX-XXXX-XXXX-XXXX`. Never shown again.
    pub recovery_code: String,
}

impl fmt::Debug for FirstRunDone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FirstRunDone")
            .field("recovery_code", &Hidden)
            .finish()
    }
}

/// Sign-in.
#[derive(Clone, Deserialize, TS)]
pub struct LoginInput {
    /// Sign-in name (not case sensitive).
    pub username: String,
    /// Password.
    pub password: String,
}

impl fmt::Debug for LoginInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoginInput")
            .field("username", &self.username)
            .field("password", &Hidden)
            .finish()
    }
}

/// Unlock after an idle lock. Same lockout counter as sign-in (plan 04 Section 4).
#[derive(Clone, Deserialize, TS)]
pub struct UnlockInput {
    /// The locked user's password.
    pub password: String,
}

impl fmt::Debug for UnlockInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnlockInput")
            .field("password", &Hidden)
            .finish()
    }
}

/// Reset the Owner password with the printed recovery code (PRD FR-8.5).
#[derive(Clone, Deserialize, TS)]
pub struct RecoverInput {
    /// The Owner's sign-in name.
    pub username: String,
    /// The printed code, with or without dashes, in any case.
    pub recovery_code: String,
    /// The new password, at least 6 characters.
    pub new_password: String,
    /// The new password again.
    pub new_password_repeat: String,
}

impl fmt::Debug for RecoverInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecoverInput")
            .field("username", &self.username)
            .field("recovery_code", &Hidden)
            .field("new_password", &Hidden)
            .field("new_password_repeat", &Hidden)
            .finish()
    }
}

/// A used recovery code is spent. This is the new one, to print.
#[derive(Clone, PartialEq, Eq, Serialize, TS)]
pub struct RecoverDone {
    /// `XXXX-XXXX-XXXX-XXXX`. Never shown again.
    pub new_recovery_code: String,
}

impl fmt::Debug for RecoverDone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecoverDone")
            .field("new_recovery_code", &Hidden)
            .finish()
    }
}

/// Who is signed in, whether a first run is needed, and the sign-in screen's data.
/// No role needed: it is asked before sign-in.
#[tauri::command]
pub async fn auth_status<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
) -> Result<AuthStatus, UiError> {
    not_implemented()
}

/// Creates the firm settings and the Owner. Only while no user exists.
///
/// Errors: `ERR_ALREADY_SET_UP`; `ERR_REQUIRED` and `ERR_NAME_UR_SCRIPT` / `ERR_NAME_EN_SCRIPT`
/// on `firm_name_ur`, `firm_name_en`, `mandi_ur`, `mandi_en`; `ERR_TEXT_TOO_LONG` (`address`);
/// `ERR_PHONE_INVALID` (`phone`); `ERR_USERNAME_INVALID` (`owner_username`);
/// `ERR_REQUIRED` (`owner_display_name`); `ERR_PASSWORD_TOO_SHORT` (`password`);
/// `ERR_PASSWORDS_DIFFER` (`password_repeat`).
#[tauri::command]
pub async fn setup_first_run<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
    _input: FirstRunInput,
) -> Result<FirstRunDone, UiError> {
    not_implemented()
}

/// Signs in. Errors: `ERR_WRONG_PASSWORD` (`password`, value `tries_left`), `ERR_LOCKED_WAIT`
/// (value `seconds`), `ERR_USER_INACTIVE`, `ERR_REQUIRED`.
#[tauri::command]
pub async fn auth_login<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
    _input: LoginInput,
) -> Result<SessionUser, UiError> {
    not_implemented()
}

/// Signs out. Needs a signed-in user (a locked one too).
#[tauri::command]
pub async fn auth_logout<R: tauri::Runtime>(_app: tauri::AppHandle<R>) -> Result<(), UiError> {
    not_implemented()
}

/// Locks the session (idle lock or the lock button). Unsaved work stays in the screens.
#[tauri::command]
pub async fn auth_lock<R: tauri::Runtime>(_app: tauri::AppHandle<R>) -> Result<(), UiError> {
    not_implemented()
}

/// Unlocks the session with the user's password. Errors as sign-in.
#[tauri::command]
pub async fn auth_unlock<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
    _input: UnlockInput,
) -> Result<SessionUser, UiError> {
    not_implemented()
}

/// Resets the Owner password with the recovery code and spends the code.
/// Errors: `ERR_RECOVERY_CODE_WRONG` (counts as a wrong try), `ERR_LOCKED_WAIT`, the
/// password errors on `new_password` and `new_password_repeat`.
#[tauri::command]
pub async fn auth_recover<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
    _input: RecoverInput,
) -> Result<RecoverDone, UiError> {
    not_implemented()
}
