//! Build script. Lists our commands so Tauri makes a permission for each one,
//! and only the commands named in `capabilities/default.json` can be called
//! from the screens (Code Standards S4, NFR-SEC-10).

/// Every command in `generate_handler!` in `src/lib.rs`. Keep both lists the same.
const APP_COMMANDS: &[&str] = &["app_info"];

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
