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
use mandi_services::core::money::Paisa;
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

/// Every code of plan 02 Section 4.6 is in the exported union, spelled as the JSON the
/// commands send, and nothing else is.
#[test]
fn error_code_union_has_every_code() {
    let decl = ErrorCode::decl(&Config::new());
    for code in ErrorCode::ALL {
        let json = serde_json::to_string(&code).unwrap();
        assert!(decl.contains(&json), "{json} missing in {decl}");
    }
    assert_eq!(
        decl.matches("\"ERR_").count(),
        ErrorCode::ALL.len(),
        "{decl}"
    );
    for code in [
        "ERR_REQUIRED",
        "ERR_NOT_A_NUMBER",
        "ERR_DECIMALS_NOT_ALLOWED",
        "ERR_NEGATIVE_NOT_ALLOWED",
        "ERR_AMOUNT_TOO_LARGE",
        "ERR_MUST_BE_MORE_THAN_ZERO",
        "ERR_TOO_MANY_DECIMALS",
        "ERR_PERCENT_OVER_100",
        "ERR_QTY_TOO_LARGE",
        "ERR_QTY_NOT_WHOLE",
        "ERR_WEIGHT_TOO_LARGE",
        "ERR_DATE_INVALID",
        "ERR_DATE_CLOSED",
        "ERR_DATE_IN_FUTURE",
        "ERR_CODE_INVALID",
        "ERR_NAME_UR_SCRIPT",
        "ERR_NAME_EN_SCRIPT",
        "ERR_NAME_TOO_LONG",
        "ERR_PHONE_INVALID",
        "ERR_TEXT_TOO_LONG",
        "ERR_CONTROL_CHARS",
        "ERR_OVERFLOW",
        "ERR_SHARES_NOT_100",
    ] {
        assert!(
            decl.contains(&format!("\"{code}\"")),
            "{code} missing in {decl}"
        );
    }
}

/// Money fields are typed `Paisa` (the frontend's opaque type) and no `Paisa.ts` is
/// written or imported from `./Paisa` (plan 02 Section 4.1).
#[test]
fn money_fields_are_typed_paisa() {
    #[derive(TS)]
    #[allow(dead_code)]
    struct MoneyProbe {
        amount: Paisa,
        maybe: Option<Paisa>,
    }
    let cfg = Config::new();
    let decl = MoneyProbe::decl(&cfg);
    assert!(decl.contains("amount: Paisa"), "{decl}");
    assert!(decl.contains("maybe: Paisa | null"), "{decl}");
    let file = MoneyProbe::export_to_string(&cfg).unwrap();
    assert!(!file.contains("./Paisa"), "{file}");
    assert!(<Paisa as TS>::output_path().is_none());
}
