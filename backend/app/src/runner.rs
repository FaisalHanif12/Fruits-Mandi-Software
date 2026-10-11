//! The wrapper every command runs in, so no failure closes the app (plan 03 Section 4.6,
//! NFR-REL-6).
//!
//! An unexpected failure, including a panic inside a dependency, becomes `ERR_INTERNAL` with a
//! reference number. The same number and the cause are written to the log.

use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::error::UiError;

/// Runs `command` and turns a panic into `ERR_INTERNAL`. `name` is the command name for the log.
///
/// Example: a bug divides by zero inside a dependency. The screen gets
/// `ERR_INTERNAL` and a reference like `E-20261010-0007`; the log line with the same number
/// says `command bikri_save panicked: attempt to divide by zero`.
pub fn guard<T>(name: &str, command: impl FnOnce() -> Result<T, UiError>) -> Result<T, UiError> {
    match catch_unwind(AssertUnwindSafe(command)) {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_owned())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "non-text panic".to_owned());
            Err(UiError::internal(format!(
                "command {name} panicked: {message}"
            )))
        }
    }
}

/// Runs blocking work (SQLite) on a worker thread inside [`guard`], so the window never
/// waits (Code Standards 5.6). A worker that cannot finish is also `ERR_INTERNAL`.
pub async fn run_blocking<T, F>(name: &'static str, command: F) -> Result<T, UiError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, UiError> + Send + 'static,
{
    match tauri::async_runtime::spawn_blocking(move || guard(name, command)).await {
        Ok(result) => result,
        Err(error) => Err(UiError::internal(format!(
            "command {name} did not finish: {error}"
        ))),
    }
}
