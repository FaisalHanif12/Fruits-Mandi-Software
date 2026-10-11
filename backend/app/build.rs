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
    "settings_get",
    "settings_set_firm",
    "settings_set_security",
    "settings_set_default_language",
    "prefs_set",
    "day_status",
    "translit_suggest",
    "translit_learn",
    "translit_pairs_list",
    "translit_pairs_remove",
    "masters_list",
    "masters_save",
    "masters_set_active",
    "masters_next_code",
    "search_masters",
    "search_recent",
    "search_touch",
    "accounts_save",
    "accounts_get",
    "accounts_list",
    "accounts_set_active",
    "accounts_next_code",
    "accounts_similar",
    "accounts_openings_preview",
    "accounts_openings_save",
    "coa_tree",
    "coa_heads_list",
    "coa_controls_list",
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
