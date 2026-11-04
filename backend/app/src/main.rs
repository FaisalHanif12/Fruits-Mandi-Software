//! Entry point of the Mandi Software desktop app.

// No console window behind the app in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;

fn main() {
    if let Err(error) = mandi_app_lib::run() {
        // Nothing can be shown once Tauri fails to start, and logging has not started. A
        // developer who started the app from a terminal sees why, for example a bad
        // MANDI_PROFILE name (plan 56 R5).
        let _ = writeln!(std::io::stderr(), "mandi-app could not start: {error}");
        std::process::exit(1);
    }
}
