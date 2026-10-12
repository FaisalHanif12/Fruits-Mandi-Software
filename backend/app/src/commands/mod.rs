//! Tauri commands, one file per module (Tech Stack 5.4).
//! Each command: check sign-in, parse, call one service, return a typed result (Code Standards B2).

pub mod accounts;
pub mod app;
pub mod assist;
pub mod auth;
pub mod day;
pub mod masters;
pub mod rules;
pub mod search;
pub mod settings;
pub mod translit;

use std::fmt;

use mandi_services::{Context, ServiceError};

use crate::error::UiError;
use crate::runner::run_blocking;
use crate::state::AppState;

/// Stands in for a secret in a `Debug` print, so a password can never reach a log by accident
/// (Code Standards S2, NFR-OBS-2).
pub(crate) struct Hidden;

impl fmt::Debug for Hidden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[hidden]")
    }
}

/// Runs one command body on a worker thread with the services context (Code Standards 5.6).
/// The body does the sign-in check first (`auth::require`), then one service call. A panic or a
/// service failure becomes a `UiError`; nothing here can close the app.
pub(crate) async fn run<R, T, F>(
    app: tauri::AppHandle<R>,
    name: &'static str,
    body: F,
) -> Result<T, UiError>
where
    R: tauri::Runtime,
    T: Send + 'static,
    F: FnOnce(&Context) -> Result<T, ServiceError> + Send + 'static,
{
    let state = AppState::of(&app)?;
    run_blocking(name, move || {
        let ctx = state.context()?;
        body(&ctx).map_err(UiError::from)
    })
    .await
}
