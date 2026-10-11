//! Tauri commands, one file per module (Tech Stack 5.4).
//! Each command: check role, parse, call one service, return a typed result (Code Standards B2).

pub mod app;
pub mod auth;
pub mod day;
pub mod settings;
pub mod users;

use std::fmt;

use crate::error::{ErrorCode, UiError};

/// The answer of a command that is in the contract but not built yet (task T1 stubs).
pub(crate) fn not_implemented<T>() -> Result<T, UiError> {
    Err(UiError::single(ErrorCode::ErrNotImplemented, None))
}

/// Stands in for a secret in a `Debug` print, so a password can never reach a log by accident
/// (Code Standards S2, NFR-OBS-2).
pub(crate) struct Hidden;

impl fmt::Debug for Hidden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[hidden]")
    }
}
