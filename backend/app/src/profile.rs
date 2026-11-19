//! The developer profile in the app (plan 56, R5): `MANDI_PROFILE=<name>` in a debug build moves
//! the data folder to `<app data>/profiles/<name>/` and every Keychain or Credential Manager
//! entry to the service `pk.mandisoftware.app.<name>`, so a test run never touches the real data
//! of the client or of a developer's own use of the app. A release build ignores the variable.
//! The rules and the names are in `mandi_core::profile`; this file applies them to Tauri.
//!
//! Under a profile two more things change, so a profile run can sit beside the real app:
//! the window position is kept in its own file, and the single-instance plugin is not used (a
//! second launch of the real app would otherwise only bring the other window to the front).

use std::path::{Path, PathBuf};

use mandi_services::core::profile;
use tauri::{AppHandle, Manager, Runtime};

/// The default file name of the saved window position (`tauri-plugin-window-state`).
const WINDOW_STATE_FILE: &str = ".window-state.json";

fn refused(error: profile::ProfileError) -> tauri::Error {
    tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, error))
}

/// The profile of this run: none for the real app.
///
/// # Errors
/// An error for a `MANDI_PROFILE` name that is not allowed (debug builds only). The app stops
/// then: it never falls back to the real data.
pub fn current() -> tauri::Result<Option<String>> {
    profile::current().map_err(refused)
}

/// The data folder below `app_data` for this run: `app_data` itself, or `<app_data>/profiles/<name>`.
///
/// # Errors
/// As [`current`].
pub fn data_dir_in(app_data: &Path) -> tauri::Result<PathBuf> {
    profile::data_dir(app_data).map_err(refused)
}

/// The data folder of this run. Every place that used `app.path().app_data_dir()` for data,
/// logs or files the app writes asks here instead.
///
/// # Errors
/// The error of Tauri's `app_data_dir`, or [`current`].
pub fn app_data_dir<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<PathBuf> {
    data_dir_in(&app.path().app_data_dir()?)
}

/// The file name of the saved window position: the default one, or one named after the profile.
pub fn window_state_file(profile: Option<&str>) -> String {
    match profile {
        Some(name) => format!(".window-state-{name}.json"),
        None => WINDOW_STATE_FILE.to_owned(),
    }
}

/// True when the single-instance plugin is used: for the real app (NFR-REL-9), not for a
/// profile run.
pub fn uses_single_instance(profile: Option<&str>) -> bool {
    profile.is_none()
}
