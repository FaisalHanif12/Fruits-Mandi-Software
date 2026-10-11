//! Build script. Lists our commands so Tauri makes a permission for each one,
//! and only the commands named in `capabilities/default.json` can be called
//! from the screens (Code Standards S4, NFR-SEC-10).

/// Every command in `generate_handler!` in `src/lib.rs`. Keep both lists the same.
const APP_COMMANDS: &[&str] = &[
    "app_info",
    "app_data_status",
    "auth_status",
    "setup_first_run",
    "auth_login",
    "auth_logout",
    "auth_lock",
    "auth_unlock",
    "auth_recover",
    "users_list",
    "users_create",
    "users_update",
    "users_reset_password",
    "settings_get",
    "settings_set_firm",
    "settings_set_security",
    "settings_set_default_language",
    "prefs_set",
    "day_status",
];

fn main() {
    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(APP_COMMANDS));
    if let Err(error) = tauri_build::try_build(attributes) {
        // A build script reports failure through cargo's error line.
        let message = format!("cargo::error=tauri-build failed: {error:#}\n");
        std::io::Write::write_all(&mut std::io::stdout(), message.as_bytes()).ok();
        std::process::exit(1);
    }
}
