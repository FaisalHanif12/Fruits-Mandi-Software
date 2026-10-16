//! Sign-in, sessions, lock, recovery and the first run (PRD Section 8, plan 04).
//!
//! - `session`: who is signed in, `require` (the sign-in check every command starts with).
//! - `login`: sign-in and unlock, with the lockout steps of PRD FR-8.2.
//! - `lockout`: what the app remembers about wrong tries while it runs (NFR-REL-11).
//! - `firstrun`: the first-run wizard and the recovery code (NFR-SEC-12).

mod firstrun;
pub(crate) mod lockout;
mod login;
mod password;
mod session;

use mandi_domain::core::auth::Language;

use crate::secret::Secret;
use crate::settings::Firm;

pub use firstrun::{first_run, first_run_confirmed, recover};
pub use login::{login, unlock};
pub use password::change_password;
pub use session::{lock, logout, require, status};

/// The signed-in user who passed `require`. Services use his id for audit rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    /// User id.
    pub user_id: i64,
    /// Sign-in name.
    pub username: String,
    /// Name shown in the top bar.
    pub display_name: String,
}

/// The signed-in user as the screens show him.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    /// User id.
    pub id: i64,
    /// Sign-in name.
    pub username: String,
    /// Name shown in the top bar.
    pub display_name: String,
    /// His language.
    pub language: Language,
    /// His text size in percent.
    pub text_size: u32,
}

/// What the start screen needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthStatusInfo {
    /// True while no user exists.
    pub first_run: bool,
    /// The signed-in user, also while locked.
    pub session: Option<SessionInfo>,
    /// True when the session is locked.
    pub locked: bool,
    /// The language last used on this computer.
    pub login_language: Language,
    /// The firm, for the sign-in screen.
    pub firm: Firm,
}

/// The first-run wizard, as typed.
#[derive(Debug, Clone)]
pub struct FirstRunRaw {
    /// Firm details. The names must be in their own script.
    pub firm: Firm,
    /// Owner sign-in name.
    pub owner_username: String,
    /// Owner display name.
    pub owner_display_name: String,
    /// Owner password.
    pub password: Secret,
    /// The password again.
    pub password_repeat: Secret,
}

/// A change of password from the user menu, as typed.
#[derive(Debug, Clone)]
pub struct ChangePasswordRaw {
    /// The password now.
    pub current: Secret,
    /// The new password, at least 6 characters.
    pub new: Secret,
    /// The new password again.
    pub repeat: Secret,
}

/// A recovery request, as typed.
#[derive(Debug, Clone)]
pub struct RecoverRaw {
    /// The Owner's sign-in name.
    pub username: String,
    /// The printed code.
    pub recovery_code: Secret,
    /// The new password.
    pub new_password: Secret,
    /// The new password again.
    pub new_password_repeat: Secret,
}
