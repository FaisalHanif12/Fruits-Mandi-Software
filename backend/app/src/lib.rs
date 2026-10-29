//! The Mandi Software Tauri app: window, plugins and thin commands (Tech Stack 5.4).

pub mod backup_runtime;
pub mod commands;
pub mod day_auto;
pub mod error;
pub mod logging;
pub mod print_job;
pub mod print_report;
pub mod print_types;
#[cfg(windows)]
mod print_windows;
pub mod refs;
pub mod runner;
pub mod sheet_pack;
pub mod state;
pub mod support_info;
pub mod updates;
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
            // Run B, plan 18 (append-only block): the backup service, its scheduler and the copy
            // at exit.
            start_backup(app);
            // Run B, plan 22 (append-only block): the daily health check.
            commands::health::start_daily(app.handle());
            // Run B, plan 19 (append-only block): the automatic day close at the closing time.
            day_auto::start(app.handle());
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
        commands::auth::auth_change_password,
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
        commands::search::search_accounts,
        commands::search::search_global,
        commands::search::search_accounts_recent,
        commands::search::search_accounts_touch,
        commands::accounts::accounts_resolve_code,
        commands::accounts::accounts_balances,
        commands::assist::assist_party_card,
        commands::rules::rules_list,
        commands::rules::rules_get,
        commands::rules::rules_save,
        commands::rules::rules_set_active,
        commands::rules::rules_test,
        commands::settings::settings_get_charges,
        commands::settings::settings_set_charges,
        commands::settings::settings_get_overrides,
        commands::settings::settings_set_overrides,
        commands::vouchers::vouchers_save_advance,
        commands::purchases::purchases_preview,
        commands::purchases::purchases_save,
        commands::purchases::purchases_cancel,
        commands::purchases::purchases_list,
        commands::purchases::purchases_get,
        commands::purchases::purchases_write_off,
        commands::purchases::purchases_write_off_undo,
        commands::purchases::purchases_set_sell_date,
        commands::purchases::purchases_entry_preview,
        commands::purchases::purchases_entry_save,
        commands::purchases::purchases_change_split,
        commands::trips::trips_defaults,
        commands::trips::trips_create,
        commands::trips::trips_set_partners,
        commands::trips::trips_list,
        commands::trips::trips_get,
        commands::trips::trips_summary,
        commands::trips::trips_cost_preview,
        commands::trips::trips_cost_save,
        commands::trips::trips_cost_cancel,
        commands::trips::trips_other_sale_preview,
        commands::trips::trips_other_sale_save,
        commands::trips::trips_other_sale_cancel,
        commands::fruit::fruit_preview,
        commands::fruit::fruit_save,
        commands::fruit::fruit_edit,
        commands::fruit::fruit_cancel,
        commands::fruit::fruit_get,
        commands::fruit::fruit_list,
        commands::fruit::fruit_lots,
        commands::gardens::gardens_defaults,
        commands::gardens::gardens_create_preview,
        commands::gardens::gardens_create,
        commands::gardens::gardens_set_partners,
        commands::gardens::gardens_list,
        commands::gardens::gardens_get,
        commands::gardens::gardens_summary,
        commands::gardens::gardens_entry_preview,
        commands::gardens::gardens_entry_save,
        commands::gardens::gardens_entry_cancel,
        commands::gardens::gardens_close_preview,
        commands::gardens::gardens_close,
        commands::gardens::gardens_reopen,
        commands::gardens::gardens_arrival_save,
        commands::gardens::gardens_arrival_cancel,
        commands::gardens::gardens_arrival_set_sell_date,
        commands::gardens::gardens_arrivals_list,
        commands::gardens::gardens_arrival_get,
        commands::gardens::gardens_arrival_write_off,
        commands::gardens::gardens_arrival_write_off_undo,
        commands::gardens::gardens_arrival_entry_preview,
        commands::gardens::gardens_arrival_entry_save,
        commands::gardens::gardens_arrival_change_split,
        commands::vouchers::vouchers_preview_advances,
        commands::vouchers::vouchers_save_advances,
        commands::vouchers::vouchers_list,
        commands::vouchers::vouchers_save_receipt,
        commands::vouchers::vouchers_save_payment,
        commands::vouchers::vouchers_preview,
        commands::vouchers::vouchers_edit,
        commands::vouchers::vouchers_cancel,
        commands::vouchers::vouchers_get,
        commands::vouchers::vouchers_save_receipts_batch,
        commands::vouchers::advances_report,
        commands::assist::assist_advances,
        commands::assist::assist_open_bills,
        commands::assist::assist_open_payables,
        commands::assist::assist_collection_list,
        commands::pukhta::pukhta_make_regular,
        commands::day::day_close_preview,
        commands::day::day_close,
        commands::day::day_reopen,
        commands::partners::partners_list,
        commands::partners::partners_get,
        commands::partners::partners_create,
        commands::partners::partners_update,
        commands::partners::partners_set_shares,
        commands::partners::partners_check_shares,
        commands::partners::partners_overview,
        commands::cashbook::cashbook_get,
        commands::expenses::vouchers_save_shop_expense,
        commands::expenses::vouchers_save_personal_expense,
        commands::expenses::vouchers_save_personal_return,
        commands::expenses::vouchers_preview_expense,
        commands::expenses::partner_takes,
        commands::expenses::vouchers_repeat_last_month_preview,
        commands::expenses::masters_expense_heads,
        commands::lent::vouchers_save_money_lent,
        commands::lent::vouchers_save_money_lent_return,
        commands::lent::lent_person_create,
        commands::lent::lent_list,
        commands::lent::lent_report,
        commands::lent::assist_attention_overdue,
        commands::capital::vouchers_save_capital_in,
        commands::capital::vouchers_save_capital_out,
        commands::capital::capital_history,
        commands::partners::partners_statement,
        commands::khata::khata_create,
        commands::khata::vouchers_save_khata_out,
        commands::khata::vouchers_save_khata_in,
        commands::khata::khata_statement,
        commands::general::general_preview,
        commands::general::general_save,
        commands::bikri::bikri_preview,
        commands::bikri::bikri_split_freight,
        commands::assist::assist_truck_memory,
        commands::assist::assist_seller_memory,
        commands::assist::assist_last_rate,
        commands::bikri::bikri_save,
        commands::bikri::bikri_get,
        commands::bikri::bikri_list,
        commands::bikri::bikri_find,
        commands::bikri::bikri_navigate,
        commands::bikri::bikri_stock_list,
        commands::bikri::bikri_ready_goods,
        commands::bikri::bikri_later_sale_preview,
        commands::bikri::bikri_later_sale_save,
        commands::bikri::bikri_open_for_edit,
        commands::bikri::bikri_edit_preview,
        commands::bikri::bikri_edit_save,
        commands::bikri::bikri_cancel_preview,
        commands::bikri::bikri_cancel,
        commands::bikri::bikri_undo,
        commands::bikri::bikri_write_off,
        commands::bikri::bikri_write_off_undo,
        commands::pukhta::search_pukhta_people,
        commands::pukhta::pukhta_register,
        commands::drafts::drafts_save,
        commands::drafts::drafts_get,
        commands::drafts::drafts_discard,
        commands::reports::reports_bikri_print_data,
        commands::reports::reports_print,
        commands::reports::reports_voucher_print_data,
        commands::reports::reports_print_voucher,
        commands::reports::reports_print_header,
        commands::reports::reports_collection_print_data,
        commands::reports::reports_print_collection,
        commands::reports::reports_list_printers,
        commands::settings::settings_get_printing,
        commands::settings::settings_set_printing,
        // Run B, plan 18 (append-only block).
        commands::backup::backup_status,
        commands::backup::backup_list,
        commands::backup::backup_now,
        commands::backup::backup_restore_preview,
        commands::backup::backup_restore,
        commands::backup::backup_test_restore,
        commands::backup::backup_settings_get,
        commands::backup::backup_settings_set,
        commands::backup::backup_key_setup,
        commands::backup::backup_key_confirm,
        // Run B, plan 20 (append-only block).
        commands::reports_engine::reports_catalog,
        commands::reports_engine::reports_run,
        commands::reports_engine::reports_print_data,
        // Run B, plan 20, step B3 (append-only block).
        commands::reports_people::reports_find_people,
        // Run B, plan 21 (append-only block).
        commands::analytics::analytics_catalog,
        commands::analytics::analytics_run,
        commands::analytics::analytics_lines,
        commands::analytics::analytics_print_data,
        commands::analytics::analytics_views_get,
        commands::analytics::analytics_views_save,
        // Run B, plan 22 (append-only block).
        commands::settings_day::settings_day_get,
        commands::settings_day::settings_day_set,
        commands::settings_day::settings_set_number_grouping,
        commands::audit::audit_list,
        commands::audit::audit_options,
        commands::health::health_run,
        commands::health::health_last,
        commands::support::support_info_save,
        commands::updates::updates_check,
        commands::updates::updates_install,
        commands::export::reports_export_excel,
        commands::export::analytics_export_excel,
        commands::month::month_list,
        commands::month::month_close_preview,
        commands::month::month_close_save,
        commands::month::month_close_reverse,
        // Run B, plan 19 (append-only block).
        commands::dashboard::dashboard_get,
        commands::dashboard::assist_attention,
        commands::screen_prefs::prefs_screen_get,
        commands::screen_prefs::prefs_screen_set,
        // Run B, D5 round 12 (append-only block).
        commands::reports_engine::reports_buyer_bills_print_data,
        // Run B, D5 round 14 (append-only block).
        commands::report_print::reports_print_report,
        commands::report_print::analytics_print_view,
    ]
}

/// Starts the backup service when the data file opened: the scheduler thread (15-minute and daily
/// copies, retry of the cloud queue) and the copy at exit (plan 18, PRD FR-20.2 to FR-20.6). If the
/// data file did not open there is nothing to back up and the backup commands answer
/// `ERR_INTERNAL`.
fn start_backup<R: tauri::Runtime>(app: &tauri::App<R>) {
    let Ok(state) = state::AppState::of(app.handle()) else {
        return;
    };
    let version = app.package_info().version.to_string();
    match backup_runtime::BackupRuntime::real(&state, &version) {
        Ok(runtime) => {
            runtime.start_scheduler();
            app.manage(runtime);
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                backup_runtime::hold_close_for_backup(app.handle(), &window);
            }
        }
        Err(error) => tracing::warn!("the backup is not started: {error:?}"),
    }
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
