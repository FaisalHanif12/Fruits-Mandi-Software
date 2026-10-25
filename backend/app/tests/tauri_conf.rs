//! Guards `tauri.conf.json` and the capability file against
//! Release_Pipeline 3.3, Tech Stack 5.5 and 7, NFR-USE-16, NFR-SEC-9 and NFR-SEC-10.
#![cfg(test)]
// The permission list below is one `json!` of more than 200 names.
#![recursion_limit = "512"]

use serde_json::{Value, json};

fn conf() -> Value {
    serde_json::from_str(include_str!("../tauri.conf.json")).unwrap()
}

#[test]
fn identity_matches_release_pipeline() {
    let conf = conf();
    assert_eq!(conf["productName"], "Mandi Software");
    assert_eq!(conf["identifier"], "pk.mandisoftware.app");
    // Release keeps major.minor and replaces the last part with the run number.
    let version = conf["version"].as_str().unwrap();
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 3, "version {version}");
    assert!(
        parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    );
}

#[test]
fn updater_uses_the_real_public_key_and_endpoint() {
    let conf = conf();
    let key_file = include_str!("../../../docs/release/updater-pubkey.txt").trim();
    assert!(!key_file.is_empty());
    assert!(!key_file.contains('\n'), "the key file must be one line");
    let updater = &conf["plugins"]["updater"];
    assert_eq!(updater["pubkey"], key_file);
    assert_eq!(
        updater["endpoints"],
        json!([
            "https://github.com/FaisalHanif12/Fruits-Mandi-Software/releases/latest/download/latest.json"
        ])
    );
    assert_eq!(updater["windows"]["installMode"], "passive");
}

#[test]
fn bundle_makes_signed_windows_installers_with_offline_webview2() {
    let bundle = &conf()["bundle"];
    assert_eq!(bundle["targets"], json!(["nsis", "msi"]));
    assert_eq!(bundle["createUpdaterArtifacts"], true);
    assert_eq!(
        bundle["windows"]["webviewInstallMode"]["type"],
        "offlineInstaller"
    );
}

#[test]
fn build_hooks_match_tech_stack() {
    let build = &conf()["build"];
    assert_eq!(build["beforeDevCommand"], "pnpm --filter frontend dev");
    assert_eq!(build["devUrl"], "http://localhost:5173");
    assert_eq!(build["beforeBuildCommand"], "pnpm --filter frontend build");
    assert_eq!(build["frontendDist"], "../../frontend/dist");
}

#[test]
fn one_main_window_with_minimum_size() {
    let windows = conf()["app"]["windows"].as_array().unwrap().clone();
    assert_eq!(windows.len(), 1);
    let main = &windows[0];
    assert_eq!(main["label"], "main");
    assert_eq!(main["title"], "Mandi Software");
    assert_eq!(main["minWidth"], 1024);
    assert_eq!(main["minHeight"], 600);
    assert_eq!(main["width"], 1366);
    assert_eq!(main["height"], 768);
    assert_eq!(main["center"], true);
    // Browser zoom off (Tech Stack 7). Our own Ctrl + / Ctrl - text size replaces it.
    assert_eq!(main["zoomHotkeysEnabled"], false);
}

/// Every source in a CSP that is not a keyword is local: no remote scripts,
/// styles, fonts or connections (Code Standards S5, NFR-SEC-9).
fn assert_only_local_sources(csp: &str, allowed_urls: &[&str]) {
    for directive in csp.split(';') {
        for source in directive.split_whitespace().skip(1) {
            let is_keyword = source.starts_with('\'');
            let is_scheme = matches!(source, "data:" | "ipc:");
            let is_allowed_url = allowed_urls.contains(&source);
            assert!(
                is_keyword || is_scheme || is_allowed_url,
                "CSP source {source} is not local"
            );
        }
    }
}

#[test]
fn csp_has_no_remote_source() {
    let security = &conf()["app"]["security"];
    let csp = security["csp"].as_str().unwrap();
    assert_eq!(
        csp,
        "default-src 'self'; script-src 'self'; style-src 'self'; font-src 'self'; img-src 'self' data:; connect-src ipc: http://ipc.localhost"
    );
    assert_only_local_sources(csp, &["http://ipc.localhost"]);
    assert!(!csp.contains("unsafe"));

    let dev_csp = security["devCsp"].as_str().unwrap();
    assert_only_local_sources(
        dev_csp,
        &[
            "http://ipc.localhost",
            "http://localhost:5173",
            "ws://localhost:5173",
        ],
    );
}

#[test]
fn capability_allows_only_our_command_and_shell_plugins() {
    let cap: Value = serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
    assert_eq!(cap["identifier"], "default");
    assert_eq!(cap["windows"], json!(["main"]));
    assert_eq!(
        cap["permissions"],
        json!([
            "allow-app-info",
            "allow-app-data-status",
            "allow-auth-status",
            "allow-setup-first-run",
            "allow-auth-login",
            "allow-auth-logout",
            "allow-auth-lock",
            "allow-auth-unlock",
            "allow-auth-recover",
            "allow-auth-change-password",
            "allow-settings-get",
            "allow-settings-set-firm",
            "allow-settings-set-security",
            "allow-settings-set-default-language",
            "allow-prefs-set",
            "allow-day-status",
            "allow-translit-suggest",
            "allow-translit-learn",
            "allow-translit-pairs-list",
            "allow-translit-pairs-remove",
            "allow-masters-list",
            "allow-masters-save",
            "allow-masters-set-active",
            "allow-masters-next-code",
            "allow-search-masters",
            "allow-search-recent",
            "allow-search-touch",
            "allow-accounts-save",
            "allow-accounts-get",
            "allow-accounts-list",
            "allow-accounts-set-active",
            "allow-accounts-next-code",
            "allow-accounts-similar",
            "allow-accounts-openings-preview",
            "allow-accounts-openings-save",
            "allow-coa-tree",
            "allow-coa-heads-list",
            "allow-coa-controls-list",
            "allow-search-accounts",
            "allow-search-global",
            "allow-search-accounts-recent",
            "allow-search-accounts-touch",
            "allow-accounts-resolve-code",
            "allow-accounts-balances",
            "allow-assist-party-card",
            "allow-rules-list",
            "allow-rules-get",
            "allow-rules-save",
            "allow-rules-set-active",
            "allow-rules-test",
            "allow-settings-get-charges",
            "allow-settings-set-charges",
            "allow-settings-get-overrides",
            "allow-settings-set-overrides",
            "allow-vouchers-save-advance",
            "allow-purchases-preview",
            "allow-purchases-save",
            "allow-purchases-cancel",
            "allow-purchases-list",
            "allow-purchases-get",
            "allow-purchases-write-off",
            "allow-purchases-write-off-undo",
            "allow-trips-defaults",
            "allow-trips-create",
            "allow-trips-set-partners",
            "allow-trips-list",
            "allow-trips-get",
            "allow-trips-summary",
            "allow-trips-cost-preview",
            "allow-trips-cost-save",
            "allow-trips-cost-cancel",
            "allow-trips-other-sale-preview",
            "allow-trips-other-sale-save",
            "allow-trips-other-sale-cancel",
            "allow-fruit-preview",
            "allow-fruit-save",
            "allow-fruit-edit",
            "allow-fruit-cancel",
            "allow-fruit-get",
            "allow-fruit-list",
            "allow-fruit-lots",
            "allow-gardens-defaults",
            "allow-gardens-create-preview",
            "allow-gardens-create",
            "allow-gardens-set-partners",
            "allow-gardens-list",
            "allow-gardens-get",
            "allow-gardens-summary",
            "allow-gardens-entry-preview",
            "allow-gardens-entry-save",
            "allow-gardens-entry-cancel",
            "allow-gardens-close-preview",
            "allow-gardens-close",
            "allow-gardens-reopen",
            "allow-vouchers-preview-advances",
            "allow-vouchers-save-advances",
            "allow-vouchers-list",
            "allow-vouchers-save-receipt",
            "allow-vouchers-save-payment",
            "allow-vouchers-preview",
            "allow-vouchers-edit",
            "allow-vouchers-cancel",
            "allow-vouchers-get",
            "allow-vouchers-save-receipts-batch",
            "allow-advances-report",
            "allow-assist-advances",
            "allow-assist-open-bills",
            "allow-assist-open-payables",
            "allow-assist-collection-list",
            "allow-pukhta-make-regular",
            "allow-day-close-preview",
            "allow-day-close",
            "allow-day-reopen",
            "allow-partners-list",
            "allow-partners-get",
            "allow-partners-create",
            "allow-partners-update",
            "allow-partners-set-shares",
            "allow-partners-check-shares",
            "allow-partners-overview",
            "allow-cashbook-get",
            "allow-vouchers-save-shop-expense",
            "allow-vouchers-save-personal-expense",
            "allow-vouchers-save-personal-return",
            "allow-vouchers-preview-expense",
            "allow-partner-takes",
            "allow-vouchers-repeat-last-month-preview",
            "allow-masters-expense-heads",
            "allow-vouchers-save-money-lent",
            "allow-vouchers-save-money-lent-return",
            "allow-lent-person-create",
            "allow-lent-list",
            "allow-lent-report",
            "allow-assist-attention-overdue",
            "allow-vouchers-save-capital-in",
            "allow-vouchers-save-capital-out",
            "allow-capital-history",
            "allow-partners-statement",
            "allow-khata-create",
            "allow-vouchers-save-khata-out",
            "allow-vouchers-save-khata-in",
            "allow-khata-statement",
            "allow-general-preview",
            "allow-general-save",
            "allow-bikri-preview",
            "allow-bikri-split-freight",
            "allow-assist-truck-memory",
            "allow-assist-seller-memory",
            "allow-assist-last-rate",
            "allow-bikri-save",
            "allow-bikri-get",
            "allow-bikri-list",
            "allow-bikri-find",
            "allow-bikri-navigate",
            "allow-bikri-stock-list",
            "allow-bikri-later-sale-preview",
            "allow-bikri-later-sale-save",
            "allow-bikri-open-for-edit",
            "allow-bikri-edit-preview",
            "allow-bikri-edit-save",
            "allow-bikri-cancel-preview",
            "allow-bikri-cancel",
            "allow-bikri-undo",
            "allow-bikri-write-off",
            "allow-bikri-write-off-undo",
            "allow-search-pukhta-people",
            "allow-pukhta-register",
            "allow-drafts-save",
            "allow-drafts-get",
            "allow-drafts-discard",
            "allow-reports-bikri-print-data",
            "allow-reports-print",
            "allow-reports-voucher-print-data",
            "allow-reports-print-voucher",
            "allow-reports-print-header",
            "allow-reports-collection-print-data",
            "allow-reports-print-collection",
            "allow-reports-list-printers",
            "allow-settings-get-printing",
            "allow-settings-set-printing",
            "core:event:default",
            "window-state:default",
            "updater:allow-check",
            "process:allow-restart",
            "process:allow-exit",
            "dialog:allow-message",
            "dialog:allow-ask",
            "dialog:allow-confirm"
        ])
    );
    assert_eq!(
        conf()["app"]["security"]["capabilities"],
        json!(["default", "print"])
    );
}

/// The hidden print windows (plan 14) may only read the print data of a document, check the
/// sign-in state, and use events to say the page is ready (NFR-SEC-10).
#[test]
fn print_capability_is_small_and_only_for_print_windows() {
    let cap: Value = serde_json::from_str(include_str!("../capabilities/print.json")).unwrap();
    assert_eq!(cap["identifier"], "print");
    assert_eq!(cap["windows"], json!(["print-*"]));
    assert_eq!(
        cap["permissions"],
        json!([
            "allow-app-data-status",
            "allow-auth-status",
            "allow-reports-bikri-print-data",
            "allow-reports-collection-print-data",
            "allow-reports-voucher-print-data",
            "core:event:default"
        ])
    );
    // The hidden window reads the print data of exactly three documents (the بکری documents, the
    // voucher slip and the collection list, plan 17e) and calls no command that writes or prints.
    let print_data: Vec<&str> = cap["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .filter(|p| p.ends_with("-print-data"))
        .collect();
    assert_eq!(
        print_data,
        [
            "allow-reports-bikri-print-data",
            "allow-reports-collection-print-data",
            "allow-reports-voucher-print-data"
        ]
    );
    let others: Vec<&str> = cap["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .filter(|p| p.starts_with("allow-") && !p.ends_with("-print-data"))
        .collect();
    assert_eq!(others, ["allow-app-data-status", "allow-auth-status"]);
}
