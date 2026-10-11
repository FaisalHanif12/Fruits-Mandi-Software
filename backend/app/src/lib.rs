//! The Mandi Software Tauri app: window, plugins and thin commands (Tech Stack 5.4).

pub mod commands;
pub mod error;
pub mod logging;
pub mod refs;
pub mod runner;
pub mod state;
#[cfg(windows)]
mod webview2;

use tauri::Manager;

/// Label of the one main window, as in `tauri.conf.json`.
pub const MAIN_WINDOW: &str = "main";

/// Builds and runs the app.
///
/// Plugins are only the five of Tech Stack 2. Single instance comes first, so a
/// second launch only brings the first window to the front (NFR-REL-9).
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            focus_main_window(app);
        }))
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            #[cfg(windows)]
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                webview2::apply(&window)?;
            }
            let data_dir = app.path().app_data_dir()?;
            start_logging(app, &data_dir);
            // Never fails: a damaged or unupgradable data file is kept in the state, and the
            // screen learns of it through app_data_status (NFR-REL-6, NFR-REL-7).
            app.manage(state::AppState::start(&data_dir));
            Ok(())
        })
        .invoke_handler(handler())
        .run(context())
}

/// Starts the log files and the error reference counter. If the log folder cannot be made
/// the app still runs, without a log.
fn start_logging<R: tauri::Runtime>(app: &tauri::App<R>, data_dir: &std::path::Path) {
    let log_dir = data_dir.join("logs");
    if let Ok(guard) = logging::init(&log_dir) {
        // The guard flushes the last log lines when the app ends.
        app.manage(std::sync::Mutex::new(Some(guard)));
        refs::init(log_dir.join(refs::COUNTER_FILE));
    }
}

/// The app configuration and assets, built in once. Shared by `run` and the
/// tests, because the macro may only be used once per crate on macOS.
#[doc(hidden)]
pub fn context<R: tauri::Runtime>() -> tauri::Context<R> {
    tauri::generate_context!()
}

/// Every command the screens may call. Keep in step with `APP_COMMANDS` in
/// `build.rs` and with `capabilities/default.json`.
#[doc(hidden)]
pub fn handler<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
{
    tauri::generate_handler![
        commands::app::app_info,
        commands::app::app_data_status,
        commands::auth::auth_status,
        commands::auth::setup_first_run,
        commands::auth::auth_login,
        commands::auth::auth_logout,
        commands::auth::auth_lock,
        commands::auth::auth_unlock,
        commands::auth::auth_recover,
        commands::settings::settings_get,
        commands::settings::settings_set_firm,
        commands::settings::settings_set_security,
        commands::settings::settings_set_default_language,
        commands::settings::prefs_set,
        commands::day::day_status,
        commands::translit::translit_suggest,
        commands::translit::translit_learn,
        commands::translit::translit_pairs_list,
        commands::translit::translit_pairs_remove,
        commands::masters::masters_list,
        commands::masters::masters_save,
        commands::masters::masters_set_active,
        commands::masters::masters_next_code,
        commands::search::search_masters,
        commands::search::search_recent,
        commands::search::search_touch,
        commands::accounts::accounts_save,
        commands::accounts::accounts_get,
        commands::accounts::accounts_list,
        commands::accounts::accounts_set_active,
        commands::accounts::accounts_next_code,
        commands::accounts::accounts_similar,
        commands::accounts::accounts_openings_preview,
        commands::accounts::accounts_openings_save,
        commands::accounts::coa_tree,
        commands::accounts::coa_heads_list,
        commands::accounts::coa_controls_list,
    ]
}

/// Shows, restores and focuses the main window. Used when a second copy of the
/// app is started (NFR-REL-9).
fn focus_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        // Each step is best effort: if one fails, the window is still there.
        window.unminimize().ok();
        window.show().ok();
        window.set_focus().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
    use tauri::webview::InvokeRequest;

    fn app_and_window() -> (tauri::App<MockRuntime>, tauri::WebviewWindow<MockRuntime>) {
        let app = mock_builder()
            .invoke_handler(handler())
            .build(context())
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, MAIN_WINDOW, Default::default())
            .build()
            .unwrap();
        (app, window)
    }

    fn request(cmd: &str, url: tauri::Url) -> InvokeRequest {
        InvokeRequest {
            cmd: cmd.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url,
            body: tauri::ipc::InvokeBody::default(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        }
    }

    /// The real command through Tauri's IPC and the real capability file.
    #[test]
    fn app_info_returns_version_and_identifier() {
        let (_app, window) = app_and_window();
        let response =
            get_ipc_response(&window, request("app_info", window.url().unwrap())).unwrap();
        let info: serde_json::Value = response.deserialize().unwrap();
        // Release raises the version in tauri.conf.json, so compare with the file.
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(info["version"], conf["version"]);
        assert_eq!(info["identifier"], "pk.mandisoftware.app");
    }

    /// The status command through Tauri's IPC and the real capability file (plan 03 4.2b).
    #[test]
    fn app_data_status_reports_the_start_up_check() {
        let dir = tempfile::tempdir().unwrap();
        let state = state::AppState::start(dir.path());
        let app = mock_builder()
            .manage(state)
            .invoke_handler(handler())
            .build(context())
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, MAIN_WINDOW, Default::default())
            .build()
            .unwrap();
        let ask = || -> serde_json::Value {
            get_ipc_response(&window, request("app_data_status", window.url().unwrap()))
                .unwrap()
                .deserialize()
                .unwrap()
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            let status = ask();
            if status["state"] == "ok" {
                assert_eq!(
                    status,
                    serde_json::json!({ "state": "ok", "reference": null })
                );
                break;
            }
            assert_eq!(status["state"], "checking", "{status}");
            assert!(std::time::Instant::now() < deadline, "check did not finish");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// A remote page can never call our commands (NFR-SEC-10).
    #[test]
    fn remote_url_cannot_call_commands() {
        let (_app, window) = app_and_window();
        let remote = "https://example.com".parse().unwrap();
        assert!(get_ipc_response(&window, request("app_info", remote)).is_err());
    }

    /// A command that is not registered is refused.
    #[test]
    fn unknown_command_is_refused() {
        let (_app, window) = app_and_window();
        let url = window.url().unwrap();
        assert!(get_ipc_response(&window, request("plugin:fs|read_file", url)).is_err());
    }
}
