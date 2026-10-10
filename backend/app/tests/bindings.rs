//! Exports the TypeScript types for the frontend with ts-rs (plan 01 Section 4).
//!
//! The root `pnpm gen:types` runs:
//!
//! ```bash
//! cd backend && TS_RS_EXPORT_DIR="$PWD/../frontend/src/bindings" cargo test -p mandi-app export_bindings
//! ```
//!
//! Without `TS_RS_EXPORT_DIR` the files go to a temporary folder, so a plain
//! `cargo test` checks the export works but never changes `frontend/`.
#![cfg(test)]

use std::path::PathBuf;

use mandi_app_lib::commands::app::AppInfo;
use mandi_app_lib::error::{ErrorCode, UiError, UiErrorItem};
use ts_rs::{Config, TS};

/// Every file the export must write. A new command type is added here too.
const EXPECTED_FILES: &[&str] = &["AppInfo.ts", "ErrorCode.ts", "UiError.ts", "UiErrorItem.ts"];

fn export_dir() -> PathBuf {
    match std::env::var_os("TS_RS_EXPORT_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => std::env::temp_dir().join(format!("mandi-bindings-{}", std::process::id())),
    }
}

#[test]
fn export_bindings() {
    let dir = export_dir();
    let cfg = Config::from_env().with_out_dir(&dir);

    // export_all also writes every type these types use.
    AppInfo::export_all(&cfg).unwrap();
    UiError::export_all(&cfg).unwrap();
    UiErrorItem::export_all(&cfg).unwrap();
    ErrorCode::export_all(&cfg).unwrap();

    for file in EXPECTED_FILES {
        assert!(
            dir.join(file).is_file(),
            "missing binding {file} in {}",
            dir.display()
        );
    }
}

#[test]
fn bindings_have_the_contract_shape() {
    let cfg = Config::new();
    let app_info = AppInfo::decl(&cfg);
    assert!(app_info.contains("version: string"), "{app_info}");
    assert!(app_info.contains("identifier: string"), "{app_info}");

    let ui_error = UiError::decl(&cfg);
    assert!(ui_error.contains("ok: false"), "{ui_error}");
    assert!(
        ui_error.contains("errors: Array<UiErrorItem>"),
        "{ui_error}"
    );
    assert!(ui_error.contains("ref: string"), "{ui_error}");

    let item = UiErrorItem::decl(&cfg);
    assert!(item.contains("code: ErrorCode"), "{item}");
    assert!(item.contains("field: string | null"), "{item}");
    assert!(
        item.contains("values: Record<string, string | number>"),
        "{item}"
    );

    let codes = ErrorCode::decl(&cfg);
    assert!(codes.contains("\"ERR_NOT_IMPLEMENTED\""), "{codes}");
    assert!(codes.contains("\"ERR_INTERNAL\""), "{codes}");
}
