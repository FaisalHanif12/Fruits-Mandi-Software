//! App-level commands that are not part of a business module (plan 01 Section 4.1).

use serde::Serialize;
use ts_rs::TS;

use crate::error::UiError;

/// What the start screen shows about the running app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// App version, for example `0.1.12` (Release_Pipeline 2).
    pub version: String,
    /// Bundle identifier, `pk.mandisoftware.app` (Tech Stack 7).
    pub identifier: String,
}

impl AppInfo {
    /// Reads the version and identifier from the running app's configuration.
    pub fn from_config(package: &tauri::PackageInfo, config: &tauri::Config) -> Self {
        Self {
            version: package.version.to_string(),
            identifier: config.identifier.clone(),
        }
    }
}

/// Returns the app version and identifier. Needs no role: it is shown before login.
#[tauri::command]
pub fn app_info<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<AppInfo, UiError> {
    Ok(AppInfo::from_config(app.package_info(), app.config()))
}
