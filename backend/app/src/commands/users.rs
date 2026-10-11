//! Users, Owner only (PRD FR-2.1, FR-2.2, plan 04 Section 4).

use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Hidden, not_implemented};
use crate::error::UiError;
use mandi_services::core::auth::Role;

/// One row of the users list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct UserRow {
    /// User id.
    #[ts(type = "number")]
    pub id: i64,
    /// Sign-in name.
    pub username: String,
    /// Name shown in the top bar.
    pub display_name: String,
    /// What he may do.
    pub role: Role,
    /// False once the Owner made him inactive. He cannot sign in.
    pub is_active: bool,
    /// UTC time of his last sign-in, `2026-10-10T07:30:12.345Z`. Null if he never signed in.
    pub last_login_at: Option<String>,
}

/// A new user.
#[derive(Clone, Deserialize, TS)]
pub struct UserCreateInput {
    /// Sign-in name: 3 to 32 of `a-z 0-9 . _ -`.
    pub username: String,
    /// Name shown in the top bar.
    pub display_name: String,
    /// His role.
    pub role: Role,
    /// Password, at least 6 characters.
    pub password: String,
    /// The password again.
    pub password_repeat: String,
}

impl fmt::Debug for UserCreateInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserCreateInput")
            .field("username", &self.username)
            .field("role", &self.role)
            .field("password", &Hidden)
            .field("password_repeat", &Hidden)
            .finish_non_exhaustive()
    }
}

/// Changes a user's name, role or active flag.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct UserUpdateInput {
    /// User id.
    #[ts(type = "number")]
    pub id: i64,
    /// Name shown in the top bar.
    pub display_name: String,
    /// His role.
    pub role: Role,
    /// False makes him inactive.
    pub is_active: bool,
}

/// A new password for a user, set by the Owner (PRD FR-8.5).
#[derive(Clone, Deserialize, TS)]
pub struct ResetPasswordInput {
    /// User id.
    #[ts(type = "number")]
    pub id: i64,
    /// New password, at least 6 characters.
    pub password: String,
    /// The password again.
    pub password_repeat: String,
}

impl fmt::Debug for ResetPasswordInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResetPasswordInput")
            .field("id", &self.id)
            .field("password", &Hidden)
            .field("password_repeat", &Hidden)
            .finish()
    }
}

/// All users, Owner only. Errors: `ERR_NOT_ALLOWED`.
#[tauri::command]
pub async fn users_list<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
) -> Result<Vec<UserRow>, UiError> {
    not_implemented()
}

/// Adds a user, Owner only. Errors: `ERR_USERNAME_TAKEN` and `ERR_USERNAME_INVALID`
/// (`username`), `ERR_REQUIRED` (`display_name`), the password errors.
#[tauri::command]
pub async fn users_create<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
    _input: UserCreateInput,
) -> Result<UserRow, UiError> {
    not_implemented()
}

/// Changes a user, Owner only. Errors: `ERR_NOT_FOUND`, `ERR_LAST_OWNER`.
#[tauri::command]
pub async fn users_update<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
    _input: UserUpdateInput,
) -> Result<UserRow, UiError> {
    not_implemented()
}

/// Sets a user's password, Owner only. Errors: `ERR_NOT_FOUND`, the password errors.
#[tauri::command]
pub async fn users_reset_password<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
    _input: ResetPasswordInput,
) -> Result<(), UiError> {
    not_implemented()
}
