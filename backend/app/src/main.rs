//! Entry point of the Mandi Software desktop app.

// No console window behind the app in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if mandi_app_lib::run().is_err() {
        // Nothing can be shown once Tauri fails to start. Logging comes in plan 03.
        std::process::exit(1);
    }
}
