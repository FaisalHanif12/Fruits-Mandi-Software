//! The partner code (شراکت دار کوڈ, PRD FR-17.11, NFR-SEC-14; plan 44): a 4-digit code that opens the
//! partner section. It is kept only as a one-way hash; the unlock lives in the app's memory like
//! the sign-in, and goes when the user leaves the section, on the idle lock, on logout and on exit.
//!
//! Pure types. The commands are `partners_code_status`, `partners_code_set`,
//! `partners_code_change`, `partners_code_unlock`, `partners_code_lock` and `partners_code_reset`.
//! A wrong code is `ERR_PARTNER_CODE_WRONG` (value `tries_left`); after 5 wrong tries sign-in style
//! waits apply and `ERR_LOCKED_WAIT` (value `seconds`) is returned (as FR-8.2); every wrong try is
//! in the audit log. A command that needs the unlock returns `ERR_PARTNER_LOCKED` without it.
//!
//! Example (PRD E2E-46): the code is 4826. `partners_code_unlock("1111")` gives
//! `ERR_PARTNER_CODE_WRONG` with `tries_left` 4; `partners_code_unlock("4826")` gives a status with
//! `unlocked` true; the overview then shows الف 996, ب 746, ج 746.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Whether a code is set, whether the section is open, and how long to wait.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PartnerCodeStatus {
    /// A code has been set. When false the first visit asks to set one (the Owner's password is
    /// needed).
    pub is_set: bool,
    /// The partner section is open now.
    pub unlocked: bool,
    /// Seconds to wait before another try, after too many wrong ones. 0 when none.
    #[ts(type = "number")]
    pub wait_seconds: i64,
}

/// Input of `partners_code_set`: the first code. The Owner's password proves it is him.
#[derive(Clone, Deserialize, TS)]
pub struct PartnerCodeSetInput {
    /// The Owner's password. Never logged, never returned.
    pub password: String,
    /// The code: exactly 4 digits.
    pub code: String,
    /// The same code again.
    pub code_again: String,
}

impl std::fmt::Debug for PartnerCodeSetInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartnerCodeSetInput")
            .finish_non_exhaustive()
    }
}

/// Input of `partners_code_change`: a new code, proved by the old code or the Owner's password.
#[derive(Clone, Deserialize, TS)]
pub struct PartnerCodeChangeInput {
    /// The old code, or the Owner's password.
    pub old_code_or_password: String,
    /// The new code: exactly 4 digits.
    pub code: String,
    /// The new code again.
    pub code_again: String,
}

impl std::fmt::Debug for PartnerCodeChangeInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartnerCodeChangeInput")
            .finish_non_exhaustive()
    }
}

/// Input of `partners_code_unlock`.
#[derive(Clone, Deserialize, TS)]
pub struct PartnerCodeUnlockInput {
    /// The code.
    pub code: String,
}

impl std::fmt::Debug for PartnerCodeUnlockInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartnerCodeUnlockInput")
            .finish_non_exhaustive()
    }
}

/// Input of `partners_code_reset` ("کوڈ بھول گئے"): a new code, proved by the Owner's password or
/// the recovery code (PRD FR-8.5).
#[derive(Clone, Deserialize, TS)]
pub struct PartnerCodeResetInput {
    /// The Owner's password, or the recovery code.
    pub password_or_recovery: String,
    /// The new code: exactly 4 digits.
    pub code: String,
    /// The new code again.
    pub code_again: String,
}

impl std::fmt::Debug for PartnerCodeResetInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartnerCodeResetInput")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_inputs_hide_their_secrets_in_debug_text() {
        let input = PartnerCodeUnlockInput {
            code: "4826".to_owned(),
        };
        assert!(!format!("{input:?}").contains("4826"));
        let set = PartnerCodeSetInput {
            password: "secret1".to_owned(),
            code: "4826".to_owned(),
            code_again: "4826".to_owned(),
        };
        let text = format!("{set:?}");
        assert!(!text.contains("secret1") && !text.contains("4826"));
    }

    #[test]
    fn a_status_is_plain_data() {
        let status = PartnerCodeStatus {
            is_set: true,
            unlocked: false,
            wait_seconds: 60,
        };
        assert_eq!(
            serde_json::to_value(status).unwrap(),
            serde_json::json!({ "is_set": true, "unlocked": false, "wait_seconds": 60 })
        );
    }
}
