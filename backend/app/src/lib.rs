//! The Mandi Software Tauri app: window, plugins and thin commands (Tech Stack 5.4).

pub mod commands;
pub mod error;
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
            #[cfg(not(windows))]
            let _ = app;
            Ok(())
        })
        .invoke_handler(handler())
        .run(context())
}

/// The app configuration and assets, built in once. Shared by `run` and the
/// tests, because the macro may only be used once per crate on macOS.
fn context<R: tauri::Runtime>() -> tauri::Context<R> {
    tauri::generate_context!()
}

/// Every command the screens may call. Keep in step with `APP_COMMANDS` in
/// `build.rs` and with `capabilities/default.json`.
fn handler<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![commands::app::app_info]
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
