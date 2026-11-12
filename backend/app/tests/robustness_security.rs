//! The command layer under attack (D1 attack list item 4, D1-A13, D2 attack list item 7;
//! NFR-SEC-4, NFR-SEC-10, NFR-REL-6, Code Standards S1, S4, V0). Owner: `robustness`.
//!
//! This is separate from `ipc_auth.rs` (which checks the answer of each command) and stronger in
//! three ways:
//!
//! 1. A fingerprint of EVERY table is taken before and after the calls, so "writes nothing" is
//!    proved for the whole file, not only for the audit log, the users and the settings.
//! 2. It runs in each signed-out state: before the first run (no Owner exists), set up but never
//!    signed in, signed out again, and locked.
//! 3. Calls come from places they must not: another window, a remote page. And every command
//!    gets hostile input (null, huge, SQL, markup, deep nesting, wrong types) while signed in:
//!    none may end in `ERR_INTERNAL`, none may break the books.
//!
//! It lives in the app crate because only the app crate can speak Tauri IPC. It needs no change
//! to any `Cargo.toml`.
#![cfg(test)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use std::sync::Arc;

use mandi_app_lib::backup_runtime::BackupRuntime;
use mandi_app_lib::state::AppState;
use mandi_backup::cloud::{FixedCloud, LocalFolderCloud};
use mandi_backup::daily_pack::PendingPack;
use mandi_backup::io::NoHook;
use mandi_backup::secrets::MemorySecretStore;
use mandi_backup::service::{BackupConfig, BackupParts, BackupService};
use mandi_services::db::disk::FixedDiskProbe;
use mandi_services::db::rusqlite::{Connection, types::Value as SqlValue};
use mandi_services::testing::TestEnv;
use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tempfile::TempDir;

const OWNER_PASSWORD: &str = "secret1";

/// The commands anyone may call before signing in (NFR-SEC-4).
const OPEN: [&str; 9] = [
    "app_info",
    "app_data_status",
    "auth_status",
    "setup_first_run",
    "auth_login",
    "auth_recover",
    // Plan 56, restore on a new computer: open before sign-in, refused with ERR_ALREADY_SET_UP once
    // an Owner exists.
    "backup_first_run_list",
    "backup_first_run_preview",
    "backup_first_run_restore",
];

struct Harness {
    app: tauri::App<MockRuntime>,
    window: tauri::WebviewWindow<MockRuntime>,
    dir: TempDir,
}

impl Harness {
    fn new() -> Self {
        Self::build(false)
    }

    /// The same app with the backup service started (memory secrets, a folder for the cloud), as
    /// it is in the real app. Without it every backup command says ERR_BACKUP_NOT_STARTED.
    fn with_backup() -> Self {
        Self::build(true)
    }

    fn build(backup: bool) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let env = TestEnv::new(dir.path()).unwrap();
        let state = AppState::with_context(env.ctx.clone());
        let mut builder = mock_builder().manage(state);
        if backup {
            let service = Arc::new(BackupService::new(
                Arc::clone(env.ctx.db()),
                BackupConfig {
                    backups_dir: dir.path().join("backups"),
                    app_version: "0.0.0-test".to_owned(),
                },
                BackupParts {
                    secrets: Arc::new(MemorySecretStore::default()),
                    cloud: Arc::new(FixedCloud(Arc::new(LocalFolderCloud::new(
                        dir.path().join("cloud"),
                    )))),
                    disk: Arc::new(FixedDiskProbe(10 * 1024 * 1024 * 1024)),
                    hook: Arc::new(NoHook),
                    pack: Arc::new(PendingPack),
                },
            ));
            builder = builder.manage(BackupRuntime::new(service));
        }
        let app = builder
            .invoke_handler(mandi_app_lib::handler())
            .build(mandi_app_lib::context())
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        Self { app, window, dir }
    }

    fn request(
        &self,
        window: &tauri::WebviewWindow<MockRuntime>,
        command: &str,
        payload: Value,
        url: tauri::Url,
    ) -> InvokeRequest {
        let _ = window;
        InvokeRequest {
            cmd: command.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url,
            body: InvokeBody::Json(payload),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        }
    }

    /// Calls a command like the main screen: `Ok(result json)` or `Err(error json)`.
    fn call(&self, command: &str, payload: Value) -> Result<Value, Value> {
        let request = self.request(&self.window, command, payload, self.window.url().unwrap());
        get_ipc_response(&self.window, request).map(|body| body.deserialize::<Value>().unwrap())
    }

    fn ok(&self, command: &str, payload: Value) -> Value {
        self.call(command, payload)
            .unwrap_or_else(|e| panic!("{command} failed: {e}"))
    }

    /// The first error code of a refusal, or the whole error text if it is not a `UiError`.
    fn code(&self, command: &str, payload: Value) -> String {
        match self.call(command, payload) {
            Ok(value) => format!("OK {value}"),
            Err(error) => error["errors"][0]["code"]
                .as_str()
                .map_or_else(|| format!("RAW {error}"), str::to_owned),
        }
    }

    fn conn(&self) -> Connection {
        Connection::open(self.dir.path().join("data").join("mandi.db")).unwrap()
    }

    /// Every table, row by row, hashed. Equal before and after means nothing was written.
    fn fingerprint(&self) -> u64 {
        let conn = self.conn();
        let tables: Vec<String> = {
            let mut st = conn
                .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
                .unwrap();
            st.query_map([], |r| r.get(0))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap()
        };
        let mut hasher = DefaultHasher::new();
        for table in tables {
            table.hash(&mut hasher);
            let mut st = conn
                .prepare(&format!("SELECT * FROM \"{table}\" ORDER BY rowid"))
                .unwrap();
            let columns = st.column_count();
            let mut rows = st.query([]).unwrap();
            while let Some(row) = rows.next().unwrap() {
                for i in 0..columns {
                    let value: SqlValue = row.get(i).unwrap();
                    format!("{value:?}").hash(&mut hasher);
                }
            }
        }
        hasher.finish()
    }

    fn one(&self, sql: &str) -> i64 {
        self.conn().query_row(sql, [], |r| r.get(0)).unwrap()
    }

    fn set_up(&self) {
        self.ok("setup_first_run", json!({ "input": first_run() }));
    }

    fn login(&self) {
        self.ok(
            "auth_login",
            json!({ "input": { "username": "owner", "password": OWNER_PASSWORD } }),
        );
    }
}

fn first_run() -> Value {
    json!({
        "firm_name_ur": "ماہر عدنان اینڈ برادرز",
        "firm_name_en": "Maher Adnan and Brothers",
        "mandi_ur": "نئی سبزی منڈی کوٹ رادھا کشن",
        "mandi_en": "New Sabzi Mandi Kot Radha Kishan",
        "address": "Kot Radha Kishan",
        "phone": "0300-1234567",
        "owner_username": "owner",
        "owner_display_name": "Maher Adnan",
        "password": OWNER_PASSWORD,
        "password_repeat": OWNER_PASSWORD,
    })
}

/// Every command, its payload, and whether it needs the Owner signed in (false: open to everyone).
fn command_table() -> Vec<(&'static str, Value, bool)> {
    let firm = json!({ "name_ur": "فرم", "name_en": "Firm", "mandi_ur": "منڈی", "mandi_en": "Mandi", "address": "", "phone": "" });
    vec![
        ("app_info", json!({}), false),
        ("app_data_status", json!({}), false),
        ("auth_status", json!({}), false),
        ("setup_first_run", json!({ "input": first_run() }), false),
        (
            "auth_login",
            json!({ "input": { "username": "x", "password": "y" } }),
            false,
        ),
        (
            "auth_recover",
            json!({ "input": { "username": "x", "recovery_code": "y", "new_password": "abcdef", "new_password_repeat": "abcdef" } }),
            false,
        ),
        ("auth_logout", json!({}), true),
        ("auth_lock", json!({}), true),
        ("auth_unlock", json!({ "input": { "password": "y" } }), true),
        // The repeat differs, so a signed-in Owner gets a field error and nothing changes.
        (
            "auth_change_password",
            json!({ "input": { "current": OWNER_PASSWORD, "new": "newsecret1", "repeat": "other-secret" } }),
            true,
        ),
        ("settings_get", json!({}), true),
        (
            "prefs_set",
            json!({ "input": { "key": "text_size", "value": "110" } }),
            true,
        ),
        ("day_status", json!({}), true),
        (
            "translit_suggest",
            json!({ "input": { "text": "arshad", "direction": "to_urdu" } }),
            true,
        ),
        (
            "translit_learn",
            json!({ "input": { "roman": "xyz", "urdu": "زیڈ", "kind": "word" } }),
            true,
        ),
        ("translit_pairs_list", json!({ "input": {} }), true),
        (
            "masters_list",
            json!({ "input": { "kind": "city", "include_inactive": true } }),
            true,
        ),
        (
            "masters_save",
            json!({ "input": { "kind": "city", "id": null, "version": null, "code": "", "name_ur": "لاہور", "name_en": "Lahore", "confirmed_rules_words": [] } }),
            true,
        ),
        (
            "masters_set_active",
            json!({ "input": { "kind": "city", "id": 1, "is_active": false } }),
            true,
        ),
        (
            "masters_next_code",
            json!({ "input": { "kind": "city" } }),
            true,
        ),
        (
            "search_masters",
            json!({ "input": { "scope": "city", "text": "lah", "limit": 10 } }),
            true,
        ),
        (
            "search_recent",
            json!({ "input": { "scope": "city" } }),
            true,
        ),
        (
            "search_touch",
            json!({ "input": { "scope": "city", "id": 1 } }),
            true,
        ),
        ("accounts_get", json!({ "input": { "id": 1 } }), true),
        (
            "accounts_list",
            json!({ "input": { "filter": { "account_type": null, "text": "", "include_inactive": true }, "offset": 0, "limit": 10 } }),
            true,
        ),
        (
            "accounts_set_active",
            json!({ "input": { "id": 1, "is_active": false } }),
            true,
        ),
        (
            "accounts_next_code",
            json!({ "input": { "account_type": "buyer", "control_id": null } }),
            true,
        ),
        (
            "accounts_similar",
            json!({ "input": { "name_ur": "", "name_en": "akram", "phone": "", "exclude_id": null } }),
            true,
        ),
        (
            "accounts_openings_preview",
            json!({ "input": { "lines": [] } }),
            true,
        ),
        (
            "accounts_openings_save",
            json!({ "input": { "lines": [] } }),
            true,
        ),
        ("coa_tree", json!({}), true),
        (
            "coa_control_accounts",
            json!({ "input": { "control_id": 1, "offset": 0, "limit": 25 } }),
            true,
        ),
        ("coa_heads_list", json!({}), true),
        ("coa_controls_list", json!({}), true),
        (
            "search_accounts",
            json!({ "input": { "scope": "buyer", "control_id": null, "text": "1", "limit": 10 } }),
            true,
        ),
        (
            "search_global",
            json!({ "input": { "text": "akram", "limit": 10 } }),
            true,
        ),
        (
            "search_accounts_recent",
            json!({ "input": { "scope": "control", "control_id": 1 } }),
            true,
        ),
        (
            "search_accounts_touch",
            json!({ "input": { "scope": "buyer", "control_id": null, "id": 1 } }),
            true,
        ),
        (
            "accounts_resolve_code",
            json!({ "input": { "scope": "buyer", "control_id": null, "text": "1" } }),
            true,
        ),
        (
            "accounts_balances",
            json!({ "input": { "ids": [1] } }),
            true,
        ),
        (
            "assist_party_card",
            json!({ "input": { "account_id": 1 } }),
            true,
        ),
        (
            "rules_list",
            json!({ "input": { "filter": { "side": null, "charge_type": null, "account_id": null, "item_id": null, "include_inactive": true } } }),
            true,
        ),
        ("rules_get", json!({ "input": { "id": 1 } }), true),
        (
            "rules_save",
            json!({ "input": { "id": null, "version": null, "side": "seller", "charge_type": "commission",
                "account_id": null, "item_id": null, "item_group": null, "method": "percent", "value": "6",
                "from_date": "01-01-2026", "to_date": "", "is_active": true, "note": "" } }),
            true,
        ),
        (
            "rules_set_active",
            json!({ "input": { "id": 1, "is_active": false } }),
            true,
        ),
        (
            "rules_test",
            json!({ "input": { "seller_id": 1, "buyer_id": 2, "item_id": 1, "date": "10-10-2026", "qty": "20", "rate": "600" } }),
            true,
        ),
        ("settings_get_charges", json!({}), true),
        (
            "settings_set_charges",
            json!({ "input": {
                "seller_commission": { "version": 1, "method": "percent", "value": "6" },
                "buyer_commission": { "version": 1, "method": "percent", "value": "6" },
                "laga_fruit": { "version": 1, "method": "per_unit", "value": "8" } } }),
            true,
        ),
        ("settings_get_overrides", json!({}), true),
        (
            "purchases_preview",
            json!({ "input": { "trip_id": 1, "date": "10-10-2026", "supplier": null, "item_id": 1,
                "packing_id": 1, "qty": "20", "weight": null, "rate": "500", "freight": "400", "labour": null,
                "other_cost": null, "note": null, "parts": [], "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "purchases_save",
            json!({ "input": { "trip_id": 1, "date": "10-10-2026", "supplier": null, "item_id": 1,
                "packing_id": 1, "qty": "20", "weight": null, "rate": "500", "freight": "400", "labour": null,
                "other_cost": null, "note": null, "parts": [], "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "purchases_cancel",
            json!({ "input": { "id": 1, "version": 1, "reason": "x", "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "purchases_list",
            json!({ "input": { "trip_id": null, "from": null, "to": null, "item_id": null, "state": null,
                "offset": 0, "limit": 0 } }),
            true,
        ),
        ("purchases_get", json!({ "input": { "id": 1 } }), true),
        (
            "purchases_write_off",
            json!({ "input": { "purchase_id": 1, "qty": "1", "reason": "x" } }),
            true,
        ),
        (
            "purchases_write_off_undo",
            json!({ "input": { "move_id": 1, "reason": "x" } }),
            true,
        ),
        (
            "purchases_set_sell_date",
            json!({ "input": { "id": 1, "version": 1, "sell_date": "11-10-2026" } }),
            true,
        ),
        (
            "purchases_entry_preview",
            json!({ "input": { "trip_id": 1, "date": "10-10-2026", "supplier": null, "sell_date": null,
                "parts": [], "confirm_cash_below_zero": false, "lines": [] } }),
            true,
        ),
        (
            "purchases_entry_save",
            json!({ "input": { "trip_id": 1, "date": "10-10-2026", "supplier": null, "sell_date": null,
                "parts": [], "confirm_cash_below_zero": false, "lines": [] } }),
            true,
        ),
        (
            "purchases_change_split",
            json!({ "input": { "id": 1, "version": 1, "other_qty": "5", "shop_name": null } }),
            true,
        ),
        (
            "gardens_arrival_entry_preview",
            json!({ "input": { "garden_id": 1, "date": "10-10-2026", "sell_date": null, "lines": [] } }),
            true,
        ),
        (
            "gardens_arrival_entry_save",
            json!({ "input": { "garden_id": 1, "date": "10-10-2026", "sell_date": null, "lines": [] } }),
            true,
        ),
        (
            "gardens_arrival_change_split",
            json!({ "input": { "id": 1, "version": 1, "other_qty": "5", "shop_name": null } }),
            true,
        ),
        ("trips_defaults", json!({}), true),
        (
            "trips_create",
            json!({ "input": { "date": "10-10-2026", "market": null, "sent_person": null, "note": null,
                "partners": [{ "partner_id": 1, "share": "100", "source": "shop" }] } }),
            true,
        ),
        (
            "trips_set_partners",
            json!({ "input": { "id": 1, "version": 1,
                "partners": [{ "partner_id": 1, "share": "100", "source": "shop" }] } }),
            true,
        ),
        (
            "trips_list",
            json!({ "input": { "from": null, "to": null, "status": null, "partner_id": null, "search": null,
                "offset": 0, "limit": 0 } }),
            true,
        ),
        (
            "trips_get",
            json!({ "input": { "id": 1, "entries_offset": 0, "entries_limit": 0 } }),
            true,
        ),
        ("trips_summary", json!({}), true),
        (
            "trips_cost_preview",
            json!({ "input": { "trip_id": 1, "date": "10-10-2026", "amount": "4,000", "reason": "x",
                "parts": [], "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "trips_cost_save",
            json!({ "input": { "trip_id": 1, "date": "10-10-2026", "amount": "4,000", "reason": "x",
                "parts": [], "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "trips_cost_cancel",
            json!({ "input": { "id": 1, "version": 1, "reason": "x", "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "trips_other_sale_preview",
            json!({ "input": { "trip_id": 1, "date": "10-10-2026", "shop": null, "purchase_id": 1,
                "qty": "5", "amount": "3,000", "note": null } }),
            true,
        ),
        (
            "trips_other_sale_save",
            json!({ "input": { "trip_id": 1, "date": "10-10-2026", "shop": null, "purchase_id": 1,
                "qty": "5", "amount": "3,000", "note": null } }),
            true,
        ),
        (
            "trips_other_sale_cancel",
            json!({ "input": { "id": 1, "version": 1, "reason": "x", "confirm_cash_below_zero": false } }),
            true,
        ),
        ("gardens_defaults", json!({}), true),
        (
            "gardens_create_preview",
            json!({ "input": { "name_ur": "شاہ پور باغ", "name_en": "Shahpur Garden", "confirmed_rules_words": [], "date": "10-10-2026", "item_id": null, "rent": "50,000", "season_end": null, "note": null, "partners": [{ "partner_id": 1, "share": "100", "source": "shop" }], "paid_now": null, "parts": [], "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "gardens_create",
            json!({ "input": { "name_ur": "شاہ پور باغ", "name_en": "Shahpur Garden", "confirmed_rules_words": [], "date": "10-10-2026", "item_id": null, "rent": "50,000", "season_end": null, "note": null, "partners": [{ "partner_id": 1, "share": "100", "source": "shop" }], "paid_now": null, "parts": [], "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "gardens_set_partners",
            json!({ "input": { "id": 1, "version": 1, "partners": [{ "partner_id": 1, "share": "100", "source": "shop" }] } }),
            true,
        ),
        ("gardens_list", json!({ "input": {} }), true),
        ("gardens_get", json!({ "input": { "id": 1 } }), true),
        ("gardens_summary", json!({}), true),
        (
            "gardens_entry_preview",
            json!({ "input": { "type": "installment", "garden_id": 1, "date": "10-10-2026", "amount": "30,000", "reason": "x", "parts": [], "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "gardens_entry_save",
            json!({ "input": { "type": "other_sale", "garden_id": 1, "date": "10-10-2026", "shop": null, "qty": "20", "amount": "10,000", "note": null } }),
            true,
        ),
        (
            "gardens_entry_cancel",
            json!({ "input": { "id": 1, "version": 1, "reason": "x", "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "gardens_close_preview",
            json!({ "input": { "id": 1, "version": 1, "date": "13-10-2026" } }),
            true,
        ),
        (
            "gardens_close",
            json!({ "input": { "id": 1, "version": 1, "date": "13-10-2026" } }),
            true,
        ),
        (
            "gardens_reopen",
            json!({ "input": { "id": 1, "version": 1, "reason": "x" } }),
            true,
        ),
        (
            "gardens_arrival_save",
            json!({ "input": { "garden_id": 1, "date": "10-10-2026", "sell_date": null, "item_id": 1,
                "packing_id": 1, "qty": "100", "weight": null, "note": null } }),
            true,
        ),
        (
            "gardens_arrival_cancel",
            json!({ "input": { "id": 1, "version": 1, "reason": "x" } }),
            true,
        ),
        (
            "gardens_arrival_set_sell_date",
            json!({ "input": { "id": 1, "version": 1, "sell_date": "11-10-2026" } }),
            true,
        ),
        (
            "gardens_arrivals_list",
            json!({ "input": { "garden_id": null, "from": null, "to": null, "item_id": null,
                "state": null, "offset": 0, "limit": 0 } }),
            true,
        ),
        ("gardens_arrival_get", json!({ "input": { "id": 1 } }), true),
        (
            "gardens_arrival_write_off",
            json!({ "input": { "arrival_id": 1, "qty": "1", "reason": "x" } }),
            true,
        ),
        (
            "gardens_arrival_write_off_undo",
            json!({ "input": { "move_id": 1, "reason": "x" } }),
            true,
        ),
        (
            "fruit_preview",
            json!({ "input": { "for_guests": false, "partner_id": 1, "date": "10-10-2026", "source": "other_shop", "lot_id": null, "agent_account_id": 1, "item_id": 1, "packing_id": 1, "qty": "2", "weight": null, "rate": "580", "note": null } }),
            true,
        ),
        (
            "fruit_save",
            json!({ "input": { "for_guests": false, "partner_id": 1, "date": "10-10-2026", "source": "other_shop", "lot_id": null, "agent_account_id": 1, "item_id": 1, "packing_id": 1, "qty": "2", "weight": null, "rate": "580", "note": null } }),
            true,
        ),
        (
            "fruit_edit",
            json!({ "input": { "id": 1, "version": 1, "reason": "x", "entry": { "for_guests": false, "partner_id": 1, "date": "10-10-2026", "source": "other_shop", "lot_id": null, "agent_account_id": 1, "item_id": 1, "packing_id": 1, "qty": "2", "weight": null, "rate": "580", "note": null } } }),
            true,
        ),
        (
            "fruit_cancel",
            json!({ "input": { "id": 1, "version": 1, "reason": "x" } }),
            true,
        ),
        ("fruit_get", json!({ "input": { "id": 1 } }), true),
        ("fruit_list", json!({ "input": {} }), true),
        ("fruit_lots", json!({ "input": {} }), true),
        (
            "masters_dead_preview",
            json!({ "input": { "account_id": 1 } }),
            true,
        ),
        (
            "masters_dead_mark",
            json!({ "input": { "account_id": 1, "entry_date": "11-10-2026", "reason": "died", "note": "" } }),
            true,
        ),
        (
            "masters_dead_undo",
            json!({ "input": { "mark_id": 1, "reason": "x" } }),
            true,
        ),
        (
            "reports_dead_amount",
            json!({ "input": { "filter": "owed", "search": "", "offset": 0, "limit": 10 } }),
            true,
        ),
        (
            "vouchers_save_advance",
            json!({ "input": { "date": "10-10-2026", "seller_account_id": 1, "amount": "15,000", "expected_goods": "", "narration": "" } }),
            true,
        ),
        (
            "vouchers_preview_advances",
            json!({ "input": { "lines": [] } }),
            true,
        ),
        (
            "vouchers_save_advances",
            json!({ "input": { "lines": [] } }),
            true,
        ),
        (
            "vouchers_list",
            json!({ "input": { "voucher_type": "advance", "from": "", "to": "" } }),
            true,
        ),
        (
            "assist_advances",
            json!({ "input": { "seller_id": 1 } }),
            true,
        ),
        (
            "vouchers_save_receipt",
            json!({ "input": { "date": "10-10-2026", "account_id": 1, "amount": "10,000", "collector_staff_id": null, "narration": "" } }),
            true,
        ),
        (
            "vouchers_save_payment",
            json!({ "input": { "date": "10-10-2026", "account_id": 1, "amount": "4,316", "reason": "", "narration": "" } }),
            true,
        ),
        (
            "vouchers_preview",
            json!({ "input": { "entry": { "type": "receipt", "date": "10-10-2026", "account_id": 1, "amount": "10,000", "collector_staff_id": null, "narration": "" }, "replacing_voucher_id": null } }),
            true,
        ),
        (
            "vouchers_edit",
            json!({ "input": { "id": 1, "version": 1, "reason": "", "entry": { "type": "receipt", "date": "10-10-2026",
                "account_id": 1, "amount": "10,000", "collector_staff_id": null, "narration": "" } } }),
            true,
        ),
        (
            "vouchers_cancel",
            json!({ "input": { "id": 1, "version": 1, "reason": "x" } }),
            true,
        ),
        ("vouchers_get", json!({ "input": { "id": 1 } }), true),
        (
            "vouchers_open_for_edit",
            json!({ "input": { "id": 1 } }),
            true,
        ),
        (
            "vouchers_save_receipts_batch",
            json!({ "input": { "rows": [] } }),
            true,
        ),
        (
            "advances_report",
            json!({ "input": { "seller_id": null, "as_of": "" } }),
            true,
        ),
        (
            "assist_open_bills",
            json!({ "input": { "account_id": 1 } }),
            true,
        ),
        (
            "assist_open_payables",
            json!({ "input": { "seller_id": 1, "include_settled": false } }),
            true,
        ),
        (
            "assist_collection_list",
            json!({ "input": { "date": "10-10-2026", "collector_staff_id": null, "city_id": null } }),
            true,
        ),
        (
            "day_close_preview",
            json!({ "input": { "up_to": "" } }),
            true,
        ),
        ("day_close", json!({ "input": { "up_to": "" } }), true),
        (
            "day_reopen",
            json!({ "input": { "date": "10-10-2026", "reason": "x" } }),
            true,
        ),
        ("partners_list", json!({}), true),
        ("partners_get", json!({ "input": { "id": 1 } }), true),
        (
            "partners_create",
            json!({ "input": { "name_ur": "الف", "name_en": "Partner A", "opening_capital": null,
                "confirmed_rules_words": [] } }),
            true,
        ),
        (
            "partners_update",
            json!({ "input": { "id": 1, "version": 1, "name_ur": "الف", "name_en": "Partner A",
                "is_active": true, "confirmed_rules_words": [] } }),
            true,
        ),
        (
            "partners_set_shares",
            json!({ "input": { "effective_from": "10-10-2026",
                "shares": [{ "partner_id": 1, "share": "100" }] } }),
            true,
        ),
        (
            "partners_check_shares",
            json!({ "input": { "shares": [{ "partner_id": 1, "share": "100" }] } }),
            true,
        ),
        ("partners_overview", json!({}), true),
        (
            "cashbook_get",
            json!({ "input": { "from": null, "to": null, "offset": 0, "limit": 0 } }),
            true,
        ),
        (
            "vouchers_save_shop_expense",
            json!({ "input": { "date": "10-10-2026", "expense_head_id": 1, "amount": "3,000", "paid_to": null, "narration": null, "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "vouchers_save_personal_expense",
            json!({ "input": { "date": "10-10-2026", "partner_id": 1, "amount": "2,000", "reason": "x", "return_by": null, "taken_by": null, "narration": null, "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "vouchers_save_personal_return",
            json!({ "input": { "date": "11-10-2026", "partner_id": 1, "amount": "2,000", "narration": null } }),
            true,
        ),
        (
            "vouchers_preview_expense",
            json!({ "input": { "entry": { "type": "shop_expense", "date": "10-10-2026", "expense_head_id": 1, "amount": "3,000", "paid_to": null, "narration": null, "confirm_cash_below_zero": false }, "replacing_voucher_id": null } }),
            true,
        ),
        (
            "partner_takes",
            json!({ "input": { "partner_id": 1, "offset": 0, "limit": 0 } }),
            true,
        ),
        (
            "vouchers_repeat_last_month_preview",
            json!({ "input": { "month": "2026-10" } }),
            true,
        ),
        ("masters_expense_heads", json!({}), true),
        (
            "vouchers_save_money_lent",
            json!({ "input": { "date": "10-10-2026", "person_account_id": 1, "amount": "5,000", "reason": "x", "return_by": null, "narration": null, "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "vouchers_save_money_lent_return",
            json!({ "input": { "date": "11-10-2026", "person_account_id": 1, "amount": "3,000", "mode": "cash", "salary_head_id": null, "narration": null } }),
            true,
        ),
        (
            "lent_person_create",
            json!({ "input": { "name_ur": "اسلم", "name_en": "Aslam", "phone": null, "staff_id": null, "confirmed_rules_words": [] } }),
            true,
        ),
        (
            "lent_list",
            json!({ "input": { "person_id": 1, "offset": 0, "limit": 0 } }),
            true,
        ),
        (
            "lent_report",
            json!({ "input": { "as_of": null, "offset": 0, "limit": 0 } }),
            true,
        ),
        ("assist_attention_overdue", json!({}), true),
        (
            "vouchers_save_capital_in",
            json!({ "input": { "date": "11-10-2026", "partner_id": 1, "amount": "50,000", "narration": null, "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "vouchers_save_capital_out",
            json!({ "input": { "date": "20-10-2026", "partner_id": 1, "amount": "10,000", "narration": null, "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "capital_history",
            json!({ "input": { "partner_id": 1, "offset": 0, "limit": 0 } }),
            true,
        ),
        (
            "partners_statement",
            json!({ "input": { "partner_id": 1, "from": null, "to": null, "offset": 0, "limit": 0 } }),
            true,
        ),
        (
            "khata_create",
            json!({ "input": { "name_ur": "خاندانی کھاتہ", "name_en": "Family Khata", "note": null, "confirmed_rules_words": [] } }),
            true,
        ),
        (
            "vouchers_save_khata_out",
            json!({ "input": { "date": "10-10-2026", "khata_account_id": 1, "amount": "10,000", "reason": "x", "narration": null, "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "vouchers_save_khata_in",
            json!({ "input": { "date": "11-10-2026", "khata_account_id": 1, "amount": "4,000", "reason": "x", "narration": null, "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "khata_statement",
            json!({ "input": { "khata_account_id": 1, "from": null, "to": null, "offset": 0, "limit": 0 } }),
            true,
        ),
        (
            "general_preview",
            json!({ "input": { "date": "10-10-2026", "lines": [{ "account_id": 1, "dr": "100", "cr": "", "narration": null }, { "account_id": 2, "dr": "", "cr": "100", "narration": null }] } }),
            true,
        ),
        (
            "general_save",
            json!({ "input": { "date": "10-10-2026", "narration": null, "lines": [{ "account_id": 1, "dr": "100", "cr": "", "narration": null }, { "account_id": 2, "dr": "", "cr": "100", "narration": null }], "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "bikri_preview",
            json!({ "input": { "draft": { "header": { "entry_date": "10-10-2026" }, "lots": [] } } }),
            true,
        ),
        (
            "assist_seller_memory",
            json!({ "input": { "seller_account_id": 1 } }),
            true,
        ),
        (
            "assist_last_rate",
            json!({ "input": { "item_id": 1, "packing_id": 1, "date": "10-10-2026" } }),
            true,
        ),
        (
            "bikri_save",
            json!({ "input": { "draft": { "header": { "entry_date": "10-10-2026" }, "lots": [] },
                "confirm_cash_below_zero": false } }),
            true,
        ),
        ("bikri_get", json!({ "input": { "id": 1 } }), true),
        (
            "bikri_list",
            json!({ "input": { "date": "10-10-2026" } }),
            true,
        ),
        ("bikri_find", json!({ "input": { "number": 1 } }), true),
        (
            "bikri_navigate",
            json!({ "input": { "current_id": null, "to": "last" } }),
            true,
        ),
        (
            "bikri_stock_list",
            json!({ "input": { "date": null, "from": null, "to": null, "seller_id": null,
                "item_id": null, "include_written_off": false } }),
            true,
        ),
        (
            "bikri_ready_goods",
            json!({ "input": { "date": "10-10-2026" } }),
            true,
        ),
        (
            "bikri_later_sale_preview",
            json!({ "input": { "lot_id": 1, "date": "10-10-2026", "sales": [] } }),
            true,
        ),
        (
            "bikri_later_sale_save",
            json!({ "input": { "lot_id": 1, "date": "10-10-2026", "sales": [],
                "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "bikri_open_for_edit",
            json!({ "input": { "bikri_id": 1, "entry_id": 1 } }),
            true,
        ),
        (
            "bikri_edit_preview",
            json!({ "input": { "entry_id": 1,
                "draft": { "header": { "entry_date": "10-10-2026" }, "lots": [] } } }),
            true,
        ),
        (
            "bikri_edit_save",
            json!({ "input": { "entry_id": 1, "version": 1, "reason": null,
                "draft": { "header": { "entry_date": "10-10-2026" }, "lots": [] },
                "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "bikri_cancel_preview",
            json!({ "input": { "id": 1 } }),
            true,
        ),
        (
            "bikri_cancel",
            json!({ "input": { "id": 1, "version": 1, "reason": "غلط اندراج",
                "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "bikri_undo",
            json!({ "input": { "id": 1, "confirm_cash_below_zero": false } }),
            true,
        ),
        (
            "bikri_write_off",
            json!({ "input": { "lot_id": 1, "qty": "1", "reason": "خراب" } }),
            true,
        ),
        (
            "bikri_write_off_undo",
            json!({ "input": { "move_id": 1, "reason": "غلط" } }),
            true,
        ),
        (
            "chukta_search",
            json!({ "input": { "text": "shahid" } }),
            true,
        ),
        (
            "chukta_add",
            json!({ "input": { "name_ur": "شاہد", "name_en": "Shahid", "phone": null } }),
            true,
        ),
        (
            "party_ledgers_list",
            json!({ "input": { "side": "buyer", "text": null, "offset": 0, "limit": 0 } }),
            true,
        ),
        (
            "party_ledgers_statement",
            json!({ "input": { "account_id": 1, "from": null, "to": null, "offset": 0, "limit": 0 } }),
            true,
        ),
        (
            "drafts_save",
            json!({ "input": { "screen": "bikri", "payload_json": "{}" } }),
            true,
        ),
        (
            "drafts_get",
            json!({ "input": { "screen": "bikri" } }),
            true,
        ),
        (
            "drafts_discard",
            json!({ "input": { "screen": "bikri" } }),
            true,
        ),
        (
            "settings_set_overrides",
            json!({ "input": { "allow_line_override": false, "credit_limit_mode": "warn",
                "rate_check_enabled": true, "rate_check_percent": 30 } }),
            true,
        ),
        (
            "accounts_save",
            json!({ "input": { "id": null, "version": null, "account_type": "buyer", "control_id": null, "code": "",
                "name_ur": "اکرم", "name_en": "Akram", "confirmed_rules_words": [], "father_or_firm_ur": "",
                "father_or_firm_en": "", "city_id": null, "adda_id": null, "phone1": "", "phone2": "",
                "address": "", "linked_account_id": null, "opening_date": "", "opening_dr": "", "opening_cr": "",
                "opening_is_advance": false, "credit_allowed": true, "credit_limit": "", "default_marka_id": null,
                "collector_staff_id": null, "is_active": true, "note": "" } }),
            true,
        ),
        (
            "translit_pairs_remove",
            json!({ "input": { "id": 1 } }),
            true,
        ),
        ("settings_set_firm", json!({ "input": firm }), true),
        (
            "settings_set_security",
            json!({ "input": { "idle_lock_minutes": "5" } }),
            true,
        ),
        (
            "settings_set_default_language",
            json!({ "input": { "language": "en" } }),
            true,
        ),
        (
            "reports_bikri_print_data",
            json!({ "input": { "bikri_id": 1, "doc": "seller_bill", "language": "ur", "only_buyer_id": null } }),
            true,
        ),
        (
            "reports_print",
            json!({ "input": { "bikri_id": 1, "docs": ["seller_bill"], "language": "ur", "copies": null,
                "to": "pdf", "only_buyer_id": null } }),
            true,
        ),
        (
            "reports_voucher_print_data",
            json!({ "input": { "voucher_id": 1, "language": "ur" } }),
            true,
        ),
        (
            "reports_print_voucher",
            json!({ "input": { "voucher_id": 1, "language": "ur", "copies": null, "to": "pdf" } }),
            true,
        ),
        (
            "reports_print_header",
            json!({ "input": { "language": "ur", "title_id": "print-collection-list-title", "facts": [] } }),
            true,
        ),
        (
            "reports_collection_print_data",
            json!({ "input": { "date": "10-10-2026", "collector_staff_id": null, "city_id": null, "language": "ur" } }),
            true,
        ),
        (
            "reports_print_collection",
            json!({ "input": { "date": "10-10-2026", "collector_staff_id": null, "city_id": null, "language": "ur",
                "copies": null, "to": "pdf" } }),
            true,
        ),
        ("reports_list_printers", json!({}), true),
        ("settings_get_printing", json!({}), true),
        (
            "settings_set_printing",
            json!({ "input": { "paper_seller_bill": "a4", "paper_buyer_slips": "a5", "printer": "", "copies": "1", "print_after_save": ["seller_bill"] } }),
            true,
        ),
        // Run B, plan 18 (append-only block).
        ("backup_status", json!({}), true),
        (
            "backup_list",
            json!({ "input": { "kind": null, "offset": 0, "limit": 10 } }),
            true,
        ),
        (
            "backup_now",
            json!({ "input": { "local": true, "usb": false, "cloud": false } }),
            true,
        ),
        (
            "backup_restore_preview",
            json!({ "input": { "backup": { "kind": "local", "name": "mandi-local-20200101T000000000Z.db" }, "key": null } }),
            true,
        ),
        (
            "backup_restore",
            json!({ "input": { "backup": { "kind": "local", "name": "mandi-local-20200101T000000000Z.db" }, "key": null, "confirmed": false } }),
            true,
        ),
        (
            "backup_test_restore",
            json!({ "input": { "backup": null, "key": null } }),
            true,
        ),
        ("backup_settings_get", json!({}), true),
        (
            "backup_settings_set",
            json!({ "input": { "settings": { "local_dir": "", "usb_enabled": false, "usb_folder": "", "cloud_enabled": false,
                "cloud_endpoint": "", "cloud_bucket": "", "cloud_region": "auto", "cloud_prefix": "mandi", "closing_time": "21:00" },
                "cloud_access_key_id": null, "cloud_secret_access_key": null } }),
            true,
        ),
        ("backup_key_setup", json!({}), true),
        (
            "backup_key_confirm",
            json!({ "input": { "key": "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ" } }),
            true,
        ),
        // Plan 56, restore on a new computer: open before sign-in, refused with ERR_ALREADY_SET_UP
        // once an Owner exists.
        (
            "backup_first_run_list",
            json!({ "input": { "source": { "folder": "/mandi-test-no-such-folder" } } }),
            false,
        ),
        (
            "backup_first_run_preview",
            json!({ "input": { "source": { "folder": "/mandi-test-no-such-folder" }, "copy": null,
                "key": "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ" } }),
            false,
        ),
        (
            "backup_first_run_restore",
            json!({ "input": { "source": { "folder": "/mandi-test-no-such-folder" },
                "copy": "mandi-20200101T000000000Z.mbk", "key": "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ" } }),
            false,
        ),
        // Run B, plan 20 (append-only block).
        ("reports_catalog", json!({}), true),
        (
            "reports_run",
            json!({ "input": { "id": "R18", "filters": {}, "offset": 0, "limit": 10 } }),
            true,
        ),
        (
            "reports_print_data",
            json!({ "input": { "id": "R18", "filters": {}, "language": "en" } }),
            true,
        ),
        // Run B, plan 21 (append-only block).
        ("analytics_catalog", json!({}), true),
        (
            "analytics_run",
            json!({ "input": { "view": "A", "filters": {}, "offset": 0, "limit": 10 } }),
            true,
        ),
        (
            "analytics_lines",
            json!({ "input": { "view": "A", "filters": {}, "row_key": {}, "column_key": null, "offset": 0, "limit": 10 } }),
            true,
        ),
        (
            "analytics_print_data",
            json!({ "input": { "view": "A", "filters": {}, "language": "en" } }),
            true,
        ),
        ("analytics_views_get", json!({}), true),
        (
            "analytics_views_save",
            json!({ "input": { "views": [] } }),
            true,
        ), // Run B, plan 22 (append-only block).
        ("settings_day_get", json!({}), true),
        (
            "settings_day_set",
            json!({ "input": { "closing_time": "21:00", "auto_day_close": false } }),
            true,
        ),
        (
            "settings_set_number_grouping",
            json!({ "input": { "grouping": "lakh" } }),
            true,
        ),
        (
            "audit_list",
            json!({ "input": { "filters": {}, "offset": 0, "limit": 10 } }),
            true,
        ),
        ("audit_options", json!({}), true),
        ("health_run", json!({}), true),
        ("health_last", json!({}), true),
        (
            "support_info_save",
            json!({ "input": { "path": null } }),
            true,
        ),
        ("updates_check", json!({}), true),
        (
            "updates_install",
            json!({ "input": { "expected_version": "99.0.0", "confirmed": true } }),
            true,
        ),
        (
            "reports_export_excel",
            json!({ "input": { "job_id": "j1", "id": "R18", "language": "en", "path": "/no-such-folder/mandi-ipc-auth.xlsx" } }),
            true,
        ),
        (
            "analytics_export_excel",
            json!({ "input": { "job_id": "j1", "view": "A", "filters": {}, "language": "en", "path": "/no-such-folder/mandi-ipc-auth.xlsx" } }),
            true,
        ),
        ("month_list", json!({}), true),
        (
            "month_close_preview",
            json!({ "input": { "month": "2026-10" } }),
            true,
        ),
        (
            "month_close_save",
            json!({ "input": { "month": "2026-10" } }),
            true,
        ),
        (
            "month_close_reverse",
            json!({ "input": { "month": "2026-10", "reason": "check" } }),
            true,
        ),
        // Run B, plan 19 (append-only block).
        ("dashboard_get", json!({}), true),
        ("assist_attention", json!({}), true),
        (
            "prefs_screen_get",
            json!({ "input": { "screen": "report.r20" } }),
            true,
        ),
        (
            "prefs_screen_set",
            json!({ "input": { "screen": "report.r20", "value": "{\"city_id\":3}" } }),
            true,
        ),
        // Run B, D5 round 12 (append-only block).
        (
            "reports_buyer_bills_print_data",
            json!({ "input": { "id": "R5", "filters": {}, "language": "en" } }),
            true,
        ),
        // Run B, D5 round 14 (append-only block).
        (
            "reports_print_report",
            json!({ "input": { "report": { "id": "R18", "filters": {}, "language": "en" }, "to": "pdf" } }),
            true,
        ),
        (
            "analytics_print_view",
            json!({ "input": { "view": { "view": "A", "filters": {}, "language": "en" }, "to": "pdf" } }),
            true,
        ),
        // Plan 44 (append-only block).
        (
            "bikri_edit_review",
            json!({ "input": { "entry_id": 1,
                "draft": { "header": { "entry_date": "10-10-2026" }, "lots": [] } } }),
            true,
        ),
        (
            "vouchers_edit_preview",
            json!({ "input": { "id": 1, "version": 1, "reason": "", "entry": { "type": "receipt", "date": "10-10-2026",
                "account_id": 1, "amount": "10,000", "collector_staff_id": null, "narration": "" } } }),
            true,
        ),
        (
            "fruit_edit_preview",
            json!({ "input": { "id": 1, "version": 1, "reason": "x", "entry": { "for_guests": false, "partner_id": 1, "date": "10-10-2026", "source": "other_shop", "lot_id": null, "agent_account_id": 1, "item_id": 1, "packing_id": 1, "qty": "2", "weight": null, "rate": "580", "note": null } } }),
            true,
        ),
        (
            "purchases_edit_preview",
            json!({ "input": { "id": 1, "version": 1, "purchase": { "trip_id": 1, "date": "10-10-2026", "supplier": null, "item_id": 1, "packing_id": 1, "qty": "20", "weight": null, "rate": "500", "freight": "400", "labour": null, "other_cost": null, "note": null, "parts": [], "confirm_cash_below_zero": false }, "reason": "" } }),
            true,
        ),
        (
            "purchases_edit_save",
            json!({ "input": { "id": 1, "version": 1, "purchase": { "trip_id": 1, "date": "10-10-2026", "supplier": null, "item_id": 1, "packing_id": 1, "qty": "20", "weight": null, "rate": "500", "freight": "400", "labour": null, "other_cost": null, "note": null, "parts": [], "confirm_cash_below_zero": false }, "reason": "" } }),
            true,
        ),
        (
            "trips_cost_edit_preview",
            json!({ "input": { "id": 1, "version": 1, "cost": { "trip_id": 1, "date": "10-10-2026", "amount": "4,000", "reason": "x", "parts": [], "confirm_cash_below_zero": false }, "reason": "" } }),
            true,
        ),
        (
            "trips_cost_edit_save",
            json!({ "input": { "id": 1, "version": 1, "cost": { "trip_id": 1, "date": "10-10-2026", "amount": "4,000", "reason": "x", "parts": [], "confirm_cash_below_zero": false }, "reason": "" } }),
            true,
        ),
        (
            "trips_other_sale_edit_preview",
            json!({ "input": { "id": 1, "version": 1, "sale": { "trip_id": 1, "date": "10-10-2026", "shop": null, "purchase_id": 1, "qty": "5", "amount": "3,000", "note": null }, "reason": "" } }),
            true,
        ),
        (
            "trips_other_sale_edit_save",
            json!({ "input": { "id": 1, "version": 1, "sale": { "trip_id": 1, "date": "10-10-2026", "shop": null, "purchase_id": 1, "qty": "5", "amount": "3,000", "note": null }, "reason": "" } }),
            true,
        ),
        (
            "gardens_entry_edit_preview",
            json!({ "input": { "id": 1, "version": 1, "entry": { "type": "other_sale", "garden_id": 1, "date": "10-10-2026", "shop": null, "qty": "20", "amount": "10,000", "note": null }, "reason": "" } }),
            true,
        ),
        (
            "gardens_entry_edit_save",
            json!({ "input": { "id": 1, "version": 1, "entry": { "type": "other_sale", "garden_id": 1, "date": "10-10-2026", "shop": null, "qty": "20", "amount": "10,000", "note": null }, "reason": "" } }),
            true,
        ),
        (
            "gardens_arrival_edit_preview",
            json!({ "input": { "id": 1, "version": 1, "arrival": { "garden_id": 1, "date": "10-10-2026", "sell_date": null, "item_id": 1, "packing_id": 1, "qty": "100", "weight": null, "note": null }, "reason": "" } }),
            true,
        ),
        (
            "gardens_arrival_edit_save",
            json!({ "input": { "id": 1, "version": 1, "arrival": { "garden_id": 1, "date": "10-10-2026", "sell_date": null, "item_id": 1, "packing_id": 1, "qty": "100", "weight": null, "note": null }, "reason": "" } }),
            true,
        ),
        (
            "trips_header_edit_preview",
            json!({ "input": { "id": 1, "version": 1, "trip": { "date": "10-10-2026", "market": null, "sent_person": null, "note": null, "partners": [{ "partner_id": 1, "share": "100", "source": "shop" }] }, "reason": "" } }),
            true,
        ),
        (
            "trips_header_edit_save",
            json!({ "input": { "id": 1, "version": 1, "trip": { "date": "10-10-2026", "market": null, "sent_person": null, "note": null, "partners": [{ "partner_id": 1, "share": "100", "source": "shop" }] }, "reason": "" } }),
            true,
        ),
        (
            "gardens_header_edit_preview",
            json!({ "input": { "id": 1, "version": 1, "garden": { "name_ur": "شاہ پور باغ", "name_en": "Shahpur Garden", "confirmed_rules_words": [], "date": "10-10-2026", "item_id": null, "rent": "50,000", "season_end": null, "note": null, "partners": [{ "partner_id": 1, "share": "100", "source": "shop" }], "paid_now": null, "parts": [], "confirm_cash_below_zero": false }, "reason": "" } }),
            true,
        ),
        (
            "gardens_header_edit_save",
            json!({ "input": { "id": 1, "version": 1, "garden": { "name_ur": "شاہ پور باغ", "name_en": "Shahpur Garden", "confirmed_rules_words": [], "date": "10-10-2026", "item_id": null, "rent": "50,000", "season_end": null, "note": null, "partners": [{ "partner_id": 1, "share": "100", "source": "shop" }], "paid_now": null, "parts": [], "confirm_cash_below_zero": false }, "reason": "" } }),
            true,
        ),
        (
            "day_fix_open",
            json!({ "input": { "date": "10-10-2026", "reason": "x" } }),
            true,
        ),
        ("day_fix_close", json!({ "input": { "token": "x" } }), true),
        (
            "audit_edit_stamp",
            json!({ "input": { "entity": "bikri", "ids": [1] } }),
            true,
        ),
        (
            "audit_edit_history",
            json!({ "input": { "entity": "bikri", "id": 1 } }),
            true,
        ),
        ("partners_code_status", json!({}), true),
        (
            "partners_code_set",
            json!({ "input": { "password": "x", "code": "4826", "code_again": "4826" } }),
            true,
        ),
        (
            "partners_code_change",
            json!({ "input": { "old_code_or_password": "x", "code": "4826", "code_again": "4826" } }),
            true,
        ),
        (
            "partners_code_unlock",
            json!({ "input": { "code": "4826" } }),
            true,
        ),
        ("partners_code_lock", json!({}), true),
        (
            "partners_code_reset",
            json!({ "input": { "password_or_recovery": "x", "code": "4826", "code_again": "4826" } }),
            true,
        ),
        // Plan 48, ask the app (append-only block).
        ("assistant_status", json!({}), true),
        (
            "assistant_ask",
            json!({ "input": { "text": "ikram ke total bakaya kitne hai", "conversation_id": null } }),
            true,
        ),
        (
            "assistant_quick",
            json!({ "input": { "chip": "cash_in_hand", "party_id": null, "conversation_id": null } }),
            true,
        ),
        (
            "assistant_pick",
            json!({ "input": { "question_id": 1, "choice": { "type": "match", "key": "a20" } } }),
            true,
        ),
        (
            "assistant_rerun",
            json!({ "input": { "question_id": 1 } }),
            true,
        ),
        (
            "assistant_new_conversation",
            json!({ "input": { "conversation_id": null } }),
            true,
        ),
        (
            "assistant_settings_set",
            json!({ "input": { "enabled": false, "model": "openai/gpt-6-luna" } }),
            true,
        ),
        (
            "assistant_key_save",
            json!({ "input": { "key": "test-key", "password": "x" } }),
            true,
        ),
        (
            "assistant_key_remove",
            json!({ "input": { "password": "x" } }),
            true,
        ),
        ("assistant_test", json!({}), true),
        (
            "assistant_log_list",
            json!({ "input": { "offset": 0, "limit": 100 } }),
            true,
        ),
    ]
}

fn registered() -> Vec<String> {
    let build_rs = include_str!("../build.rs");
    let listed = build_rs.split("APP_COMMANDS").nth(1).unwrap();
    let listed = &listed[listed.find('[').unwrap()..listed.find("];").unwrap()];
    listed
        .split('"')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, name)| name.to_owned())
        .collect()
}

#[test]
fn this_table_has_every_registered_command() {
    // A new command added by a later plan must get an entry here, so it is attacked too.
    let table: Vec<&str> = command_table().iter().map(|(name, _, _)| *name).collect();
    for name in registered() {
        assert!(
            table.contains(&name.as_str()),
            "{name} is registered but has no payload in the robustness table"
        );
    }
    for name in &table {
        assert!(
            registered().iter().any(|r| r == name),
            "{name} is in the table but not registered"
        );
    }
    for (name, _, protected) in command_table() {
        assert_eq!(
            !protected,
            OPEN.contains(&name),
            "{name}: the table and the open list disagree"
        );
    }
}

#[test]
fn nothing_is_open_before_the_first_run_but_the_five_commands_and_nothing_is_written() {
    // No Owner exists yet. Every protected command is refused with ERR_NOT_LOGGED_IN, and the
    // file is the same afterwards, table by table.
    let h = Harness::new();
    let before = h.fingerprint();
    for (command, payload, protected) in command_table() {
        if !protected {
            continue;
        }
        assert_eq!(
            h.code(command, payload),
            "ERR_NOT_LOGGED_IN",
            "{command} before the first run"
        );
    }
    assert_eq!(
        h.fingerprint(),
        before,
        "a refused command wrote something before the first run"
    );
    assert_eq!(h.one("SELECT COUNT(*) FROM users"), 0);
}

#[test]
fn every_command_is_refused_in_every_signed_out_state_and_the_whole_file_stays_the_same() {
    let h = Harness::new();
    h.set_up();
    // State 1: set up, nobody signed in (a fresh start of the app).
    // State 2: signed in, then signed out.
    // State 3: signed in, then locked by the idle lock.
    type Enter = Box<dyn Fn(&Harness)>;
    let states: [(&str, Enter, &str); 3] = [
        (
            "never signed in",
            Box::new(|_h: &Harness| {}),
            "ERR_NOT_LOGGED_IN",
        ),
        (
            "signed out again",
            Box::new(|h: &Harness| {
                h.login();
                h.ok("auth_logout", json!({}));
            }),
            "ERR_NOT_LOGGED_IN",
        ),
        (
            "locked",
            Box::new(|h: &Harness| {
                h.login();
                h.ok("auth_lock", json!({}));
            }),
            "ERR_SESSION_LOCKED",
        ),
    ];
    for (label, enter, want) in states {
        enter(&h);
        let before = h.fingerprint();
        let mut refused = 0;
        for (command, payload, protected) in command_table() {
            if !protected {
                continue;
            }
            // While locked, the session may be unlocked, left or locked again (PRD FR-8.4).
            if label == "locked" && matches!(command, "auth_unlock" | "auth_logout" | "auth_lock") {
                continue;
            }
            assert_eq!(h.code(command, payload), want, "{command} while {label}");
            refused += 1;
        }
        assert!(refused >= 50, "{label}: only {refused} commands were tried");
        assert_eq!(
            h.fingerprint(),
            before,
            "a refused command wrote something while {label}"
        );
        if label == "locked" {
            h.ok(
                "auth_unlock",
                json!({ "input": { "password": OWNER_PASSWORD } }),
            );
            h.ok("auth_logout", json!({}));
        }
    }
    // The open commands: the setup wizard cannot run twice, and nothing leaks while signed out.
    let before = h.fingerprint();
    assert_eq!(
        h.code("setup_first_run", json!({ "input": first_run() })),
        "ERR_ALREADY_SET_UP"
    );
    assert_eq!(h.fingerprint(), before, "a second wizard wrote something");
    let status = h.ok("auth_status", json!({}));
    assert!(status["session"].is_null(), "{status}");
    let text = status.to_string();
    // The sign-in screen shows the firm (PRD 8.1). It must show nothing about the user.
    for secret in [
        "display_name",
        "username",
        "password",
        "argon2",
        "recovery",
        "owner",
    ] {
        assert!(
            !text.contains(secret),
            "auth_status leaks '{secret}' to a signed-out screen: {text}"
        );
    }
    let info = h.ok("app_info", json!({}));
    let keys: Vec<&String> = info.as_object().unwrap().keys().collect();
    assert!(
        keys.iter()
            .all(|k| ["version", "identifier"].contains(&k.as_str())),
        "app_info: {info}"
    );
    let data = h.ok("app_data_status", json!({}));
    assert!(
        !data.to_string().contains('/') && !data.to_string().contains('\\'),
        "app_data_status leaks a path: {data}"
    );
}

#[test]
fn a_signed_in_owner_can_use_every_command_once_and_signing_out_closes_them_again() {
    // The other side of the refusal: the same table works when signed in (so the refusal above
    // is the sign-in check and not a broken payload).
    let h = Harness::new();
    h.set_up();
    h.login();
    let mut worked = 0;
    for (command, payload, protected) in command_table() {
        if !protected || matches!(command, "auth_logout" | "auth_lock" | "auth_unlock") {
            continue;
        }
        let result = h.code(command, payload);
        // Not a sign-in refusal. (Some payloads name records that do not exist: a different
        // refusal is fine.)
        assert!(
            result != "ERR_NOT_LOGGED_IN" && result != "ERR_SESSION_LOCKED",
            "{command} was refused as signed out while signed in"
        );
        worked += 1;
    }
    assert!(worked >= 50);
    h.ok("auth_logout", json!({}));
    assert_eq!(h.code("settings_get", json!({})), "ERR_NOT_LOGGED_IN");
}

#[test]
fn a_command_from_another_window_or_a_remote_page_is_never_run() {
    let h = Harness::new();
    h.set_up();
    h.login();
    let before = h.fingerprint();
    // A second window the capability file does not name.
    let other = tauri::WebviewWindowBuilder::new(&h.app, "intruder", Default::default())
        .build()
        .unwrap();
    let mut refused_other = 0;
    let mut refused_remote = 0;
    for (command, payload, _) in command_table() {
        let request = h.request(&other, command, payload.clone(), other.url().unwrap());
        if get_ipc_response(&other, request).is_err() {
            refused_other += 1;
        }
        // A remote page in the main window.
        let request = h.request(
            &h.window,
            command,
            payload,
            "https://example.com/".parse().unwrap(),
        );
        if get_ipc_response(&h.window, request).is_err() {
            refused_remote += 1;
        }
    }
    let total = command_table().len();
    assert_eq!(
        refused_remote,
        total,
        "a remote page could call {} commands",
        total - refused_remote
    );
    assert_eq!(
        refused_other,
        total,
        "an unnamed window could call {} commands",
        total - refused_other
    );
    assert_eq!(h.fingerprint(), before, "a forbidden call wrote something");
}

#[test]
fn plugin_and_unknown_commands_the_capability_file_does_not_name_are_refused() {
    let h = Harness::new();
    h.set_up();
    h.login();
    for command in [
        "plugin:fs|read_file",
        "plugin:fs|write_file",
        "plugin:shell|execute",
        "plugin:shell|open",
        "plugin:http|fetch",
        "plugin:opener|open_url",
        "plugin:os|platform",
        "plugin:updater|download_and_install",
        "plugin:updater|install",
        "plugin:process|exec",
        // `plugin:dialog|open` and `plugin:dialog|save` ARE named in the capability file (plan 56 Section 8 and the
        // Excel export), so they are not listed here; the screens' own dialogs only return a path.
        "plugin:dialog|confirm_everything",
        "plugin:dialog|message_and_exec",
        "plugin:clipboard-manager|read_text",
        "plugin:event|emit",
        "plugin:event|listen_to_everything",
        "plugin:window|set_fullscreen",
        "plugin:webview|create_webview_window",
        "plugin:path|resolve_directory",
        "plugin:app|app_hide",
        "plugin:resources|close",
        "no_such_command",
        "",
        "../app_info",
        "app_info ",
        "APP_INFO",
        "accounts_delete",
        "masters_delete",
        "journal_delete",
        "journal_entries_update",
        "sql_execute",
        "db_query",
        "run_sql",
        "audit_clear",
        "settings_reset",
    ] {
        let request = h.request(&h.window, command, json!({}), h.window.url().unwrap());
        let result = get_ipc_response(&h.window, request);
        assert!(result.is_err(), "'{command}' was run: {result:?}");
    }
}

#[test]
fn every_command_wrapper_checks_the_sign_in_first() {
    // The source of every command: the closure must begin with `auth::require(ctx)?` (the first
    // statement, Code Standards S1) unless the command is on the open list.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands");
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "rs")
            || path.file_name().is_some_and(|n| n == "mod.rs")
        {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        // Each `run(app, "name", <closure>)` call and the lines that follow it.
        let mut rest = text.as_str();
        while let Some(at) = rest.find("run(app, \"") {
            let after = &rest[at + "run(app, \"".len()..];
            let name = &after[..after.find('"').unwrap()];
            let tail = after[name.len() + 1..].trim_start_matches([',', ' ']);
            if tail.starts_with("auth::") {
                // The two commands that hand a service function to `run` directly: the service
                // looks at the session itself (lock and logout).
                assert!(
                    matches!(name, "auth_lock" | "auth_logout"),
                    "{name} hands a function to run: {}",
                    &tail[..40.min(tail.len())]
                );
                rest = after;
                continue;
            }
            let body = &after[after.find('{').unwrap() + 1..];
            let first = body
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty() && !l.starts_with("//"))
                .unwrap_or("");
            if name == "auth_unlock" {
                // The unlock service looks at the session itself (no session is
                // ERR_NOT_LOGGED_IN); the runtime tests above prove the refusal.
                assert!(first.starts_with("auth::unlock("), "{first}");
                rest = after;
                continue;
            }
            if !OPEN.contains(&name) {
                assert!(
                    first.contains("auth::require(ctx)"),
                    "{}: command {name} does not begin with the sign-in check; it begins with `{first}`",
                    path.display()
                );
                checked += 1;
            }
            rest = after;
        }
    }
    assert!(checked >= 50, "only {checked} commands found");
}

// ---------------------------------------------------------------------------------------------
// Hostile input to every command while signed in.
// ---------------------------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(n.max(1)).unwrap()).unwrap()
    }
}

/// The paths (as lists of keys or indexes) of every leaf of a JSON value.
fn leaf_paths(value: &Value, here: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, child) in map {
                here.push(key.clone());
                leaf_paths(child, here, out);
                here.pop();
            }
        }
        Value::Array(items) if !items.is_empty() => {
            for (index, child) in items.iter().enumerate() {
                here.push(index.to_string());
                leaf_paths(child, here, out);
                here.pop();
            }
        }
        _ => out.push(here.clone()),
    }
}

fn set_at(value: &mut Value, path: &[String], new: Value) {
    let Some((first, rest)) = path.split_first() else {
        *value = new;
        return;
    };
    let child = match value {
        Value::Object(map) => map.get_mut(first),
        Value::Array(items) => first.parse::<usize>().ok().and_then(|i| items.get_mut(i)),
        _ => None,
    };
    if let Some(child) = child {
        set_at(child, rest, new);
    }
}

fn hostile(rng: &mut Rng) -> Value {
    let deep = {
        let mut v = json!(1);
        for _ in 0..300 {
            v = json!([v]);
        }
        v
    };
    let options: Vec<Value> = vec![
        Value::Null,
        json!(true),
        json!(-1),
        json!(0),
        json!(i64::MAX),
        json!(i64::MIN),
        json!(u64::MAX),
        json!(1e308),
        json!(-0.0),
        json!(0.5),
        json!(""),
        json!("   "),
        json!("x".repeat(100_000)),
        json!("'; DROP TABLE accounts; --"),
        json!("' OR 1=1 --"),
        json!("../../../etc/passwd"),
        json!("..\\..\\windows\\system32"),
        json!("<script>alert(1)</script>"),
        json!("\u{0}"),
        json!("\u{202E}abc\u{200B}"),
        json!("😀😀😀"),
        json!("۱۲۳۴۵"),
        json!("10,00,00,00,001"),
        json!("9".repeat(400)),
        json!("1e5"),
        json!("-5"),
        json!("1.5"),
        json!([]),
        json!({}),
        json!([null, null]),
        json!({ "a": { "b": null } }),
        deep,
    ];
    options[rng.below(options.len())].clone()
}

#[test]
fn hostile_input_to_every_command_never_ends_in_an_internal_error_or_breaks_the_books() {
    let h = Harness::new();
    h.set_up();
    h.login();
    let mut rng = Rng(0x0D1D_25EC);
    let mut internal: Vec<String> = Vec::new();
    let mut calls = 0_u32;
    let mut tauri_level = 0_u32;
    let mut service_level = 0_u32;
    let mut worked = 0_u32;
    let commands: Vec<(&'static str, Value)> = command_table()
        .into_iter()
        .filter(|(name, _, protected)| {
            *protected
                && !matches!(
                    *name,
                    "auth_logout"
                        | "auth_lock"
                        | "settings_set_security"
                        | "settings_set_firm"
                        | "settings_set_default_language"
                )
        })
        .map(|(name, payload, _)| (name, payload))
        .collect();
    for round in 0..12 {
        for (command, payload) in &commands {
            let mut paths = Vec::new();
            leaf_paths(payload, &mut Vec::new(), &mut paths);
            for _ in 0..4 {
                let mut mutated = payload.clone();
                if !paths.is_empty() {
                    let path = &paths[rng.below(paths.len())];
                    match rng.below(10) {
                        0 => {
                            // Add a key nobody asked for.
                            if let Some(map) =
                                mutated.get_mut("input").and_then(Value::as_object_mut)
                            {
                                map.insert("extra".into(), hostile(&mut rng));
                            }
                        }
                        1 => {
                            // Take a key away.
                            if let Some(map) =
                                mutated.get_mut("input").and_then(Value::as_object_mut)
                            {
                                let keys: Vec<String> = map.keys().cloned().collect();
                                if !keys.is_empty() {
                                    map.remove(&keys[rng.below(keys.len())]);
                                }
                            }
                        }
                        _ => set_at(&mut mutated, path, hostile(&mut rng)),
                    }
                }
                calls += 1;
                match h.call(command, mutated.clone()) {
                    Ok(_) => worked += 1,
                    Err(error) => match error["errors"][0]["code"].as_str() {
                        Some("ERR_INTERNAL") => {
                            internal.push(format!(
                                "{command} {}",
                                mutated.to_string().chars().take(300).collect::<String>()
                            ));
                        }
                        Some(_) => service_level += 1,
                        None => tauri_level += 1,
                    },
                }
                // The session survives any input; if it does not, sign in again and note it.
                if h.code("settings_get", json!({})).starts_with("ERR_") {
                    h.login();
                }
            }
        }
        // The books and the file are sound after every round.
        let conn = h.conn();
        let integrity: String = conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok", "round {round}");
        let (dr, cr): (i64, i64) = conn
            .query_row(
                "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id WHERE e.status = 'posted'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            dr, cr,
            "round {round}: the books do not balance after hostile input"
        );
        assert_eq!(
            h.one("SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'"),
            0
        );
        assert_eq!(
            h.one("SELECT COUNT(*) FROM accounts WHERE name_en LIKE '%DROP TABLE%'"),
            0
        );
    }
    println!(
        "{calls} hostile calls: {worked} worked, {service_level} refused by a service with a code, {tauri_level} refused by the IPC layer before any service, {} internal errors",
        internal.len()
    );
    assert!(calls >= 2_000);
    assert!(
        internal.is_empty(),
        "hostile input ended in ERR_INTERNAL:\n{}",
        internal.join("\n")
    );
}

// ---------------------------------------------------------------------------------------------
// auth_change_password through the command (PRD FR-5.6, NFR-SEC-1, NFR-SEC-2, NFR-SEC-4).
// ---------------------------------------------------------------------------------------------

/// Plan 56: once a shop exists, the three commands of "restore on a new computer" are refused with
/// ERR_ALREADY_SET_UP, whatever the input, whoever is signed in (nobody, the Owner, a locked
/// screen). The refusal comes BEFORE any look at the input, so a hostile path or key tells nothing
/// (not even whether a folder exists), reads nothing and writes nothing.
#[test]
fn the_restore_on_a_new_computer_is_refused_once_a_shop_exists_for_every_source_and_input() {
    let h = Harness::with_backup();
    h.set_up();
    let files = |h: &Harness| {
        let mut names: Vec<String> = files_under(h.dir.path())
            .into_iter()
            .map(|p| p.display().to_string())
            .filter(|p| !p.ends_with("-wal") && !p.ends_with("-shm"))
            .collect();
        names.sort();
        names
    };

    let key = "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ";
    let cloud = json!({ "cloud": {
        "endpoint": "https://example.invalid", "bucket": "b", "region": "auto", "prefix": "mandi",
        "key_id": "AKIAEXAMPLE", "secret": "secret-value" } });
    let sources: Vec<Value> = vec![
        cloud,
        json!({ "folder": "/mandi-test-no-such-folder" }),
        json!({ "folder": "/" }),
        json!({ "folder": "../../etc" }),
        json!({ "file": "/mandi-test-no-such-file.mbk" }),
        json!({ "file": "/etc/passwd" }),
        json!({ "folder": "x".repeat(5_000) }),
        json!({ "cloud": { "endpoint": "http://not-secure", "bucket": "", "region": "", "prefix": "..",
            "key_id": "", "secret": "" } }),
    ];
    let payloads = |source: &Value| -> Vec<(&'static str, Value)> {
        vec![
            (
                "backup_first_run_list",
                json!({ "input": { "source": source } }),
            ),
            (
                "backup_first_run_preview",
                json!({ "input": { "source": source, "copy": null, "key": key } }),
            ),
            (
                "backup_first_run_restore",
                json!({ "input": { "source": source, "copy": "mandi-20200101T000000000Z.mbk", "key": key } }),
            ),
        ]
    };
    let mut rng = Rng(0x5E71_0056);
    let mut problems: Vec<String> = Vec::new();
    let mut calls = 0_u32;
    for state in ["signed out", "signed in", "locked"] {
        match state {
            "signed in" => h.login(),
            "locked" => {
                h.ok("auth_lock", json!({}));
            }
            _ => {}
        }
        // Signing in and locking write their own audit rows; the calls below must write nothing.
        let before = h.fingerprint();
        let files_before = files(&h);
        for source in &sources {
            for (command, payload) in payloads(source) {
                calls += 1;
                let code = h.code(command, payload);
                if code != "ERR_ALREADY_SET_UP" {
                    problems.push(format!("{state}: {command} {source} gave {code}"));
                }
            }
        }
        // Hostile values in every leaf of every payload: refused by a service with
        // ERR_ALREADY_SET_UP, or by the IPC layer before any service. Never accepted, never
        // ERR_INTERNAL.
        for (command, payload) in payloads(&sources[1]) {
            let mut paths = Vec::new();
            leaf_paths(&payload, &mut Vec::new(), &mut paths);
            for _ in 0..40 {
                let mut mutated = payload.clone();
                let path = &paths[rng.below(paths.len())];
                set_at(&mut mutated, path, hostile(&mut rng));
                calls += 1;
                match h.call(command, mutated.clone()) {
                    Ok(_) => problems.push(format!(
                        "{state}: {command} ACCEPTED {}",
                        mutated.to_string().chars().take(200).collect::<String>()
                    )),
                    Err(error) => match error["errors"][0]["code"].as_str() {
                        Some("ERR_ALREADY_SET_UP") | None => {}
                        Some(other) => problems.push(format!(
                            "{state}: {command} gave {other} for {}",
                            mutated.to_string().chars().take(200).collect::<String>()
                        )),
                    },
                }
            }
        }
        assert_eq!(
            h.fingerprint(),
            before,
            "{state}: a refused call wrote something"
        );
        assert_eq!(
            files(&h),
            files_before,
            "{state}: a refused call made or removed a file"
        );
    }
    assert!(calls >= 400);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

fn change(current: &str, new: &str, repeat: &str) -> Value {
    json!({ "input": { "current": current, "new": new, "repeat": repeat } })
}

fn files_under(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(files_under(&path));
            } else {
                out.push(path);
            }
        }
    }
    out
}

fn bytes_contain(haystack: &[u8], needle: &str) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle.as_bytes())
}

#[test]
fn a_password_change_is_refused_signed_out_and_locked_and_writes_nothing() {
    let h = Harness::new();
    h.set_up();
    let payload = change(OWNER_PASSWORD, "brand-new-pass-9", "brand-new-pass-9");
    let hash_before: String = h
        .conn()
        .query_row("SELECT password_hash FROM users", [], |r| r.get(0))
        .unwrap();
    // Set up but not signed in.
    assert_eq!(
        h.code("auth_change_password", payload.clone()),
        "ERR_NOT_LOGGED_IN"
    );
    // Signed in and then locked by the idle lock: the password cannot be changed from behind it.
    h.login();
    h.ok("auth_lock", json!({}));
    let locked_before = h.fingerprint();
    assert_eq!(
        h.code("auth_change_password", payload.clone()),
        "ERR_SESSION_LOCKED"
    );
    assert_eq!(h.fingerprint(), locked_before);
    h.ok(
        "auth_unlock",
        json!({ "input": { "password": OWNER_PASSWORD } }),
    );
    h.ok("auth_logout", json!({}));
    // Signed out again.
    assert_eq!(h.code("auth_change_password", payload), "ERR_NOT_LOGGED_IN");
    let hash_after: String = h
        .conn()
        .query_row("SELECT password_hash FROM users", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        hash_before, hash_after,
        "the password changed without a sign-in"
    );
}

#[test]
fn wrong_current_passwords_through_the_command_count_towards_the_lockout() {
    let h = Harness::new();
    h.set_up();
    h.login();
    let hash_before: String = h
        .conn()
        .query_row("SELECT password_hash FROM users", [], |r| r.get(0))
        .unwrap();
    let typed = "not-the-right-one-77";
    let mut codes = Vec::new();
    for _ in 0..5 {
        let result = h.call(
            "auth_change_password",
            change(typed, "brand-new-pass-9", "brand-new-pass-9"),
        );
        let error = result.expect_err("a wrong current password was accepted");
        // The answer never repeats what was typed.
        let text = error.to_string();
        assert!(
            !text.contains(typed)
                && !text.contains("brand-new-pass-9")
                && !text.contains(OWNER_PASSWORD),
            "the error shows a password: {text}"
        );
        codes.push(
            error["errors"][0]["code"]
                .as_str()
                .unwrap_or("?")
                .to_owned(),
        );
    }
    assert_eq!(
        codes,
        [
            "ERR_WRONG_PASSWORD",
            "ERR_WRONG_PASSWORD",
            "ERR_WRONG_PASSWORD",
            "ERR_WRONG_PASSWORD",
            "ERR_LOCKED_WAIT"
        ]
    );
    // During the wait the right current password is refused too, and so is the sign-in.
    assert_eq!(
        h.code(
            "auth_change_password",
            change(OWNER_PASSWORD, "brand-new-pass-9", "brand-new-pass-9")
        ),
        "ERR_LOCKED_WAIT"
    );
    let hash_after: String = h
        .conn()
        .query_row("SELECT password_hash FROM users", [], |r| r.get(0))
        .unwrap();
    assert_eq!(hash_before, hash_after);
    h.ok("auth_logout", json!({}));
    assert_eq!(
        h.code(
            "auth_login",
            json!({ "input": { "username": "owner", "password": OWNER_PASSWORD } })
        ),
        "ERR_LOCKED_WAIT"
    );
}

#[test]
fn a_changed_password_is_in_no_audit_row_no_answer_and_no_file() {
    let h = Harness::new();
    h.set_up();
    h.login();
    let new = "brand-new-pass-9";
    let answer = h.ok("auth_change_password", change(OWNER_PASSWORD, new, new));
    assert!(
        !answer.to_string().contains(new) && !answer.to_string().contains(OWNER_PASSWORD),
        "the answer holds a password: {answer}"
    );
    assert_eq!(
        h.one("SELECT COUNT(*) FROM audit_log WHERE action = 'change_password'"),
        1
    );
    // The old password no longer signs in; the new one does.
    h.ok("auth_logout", json!({}));
    assert_ne!(
        h.code(
            "auth_login",
            json!({ "input": { "username": "owner", "password": OWNER_PASSWORD } })
        ),
        "OK null"
    );
    h.ok(
        "auth_login",
        json!({ "input": { "username": "owner", "password": new } }),
    );
    // Neither password is anywhere in the audit log or in any file of the data folder (the
    // database, its log files and the safety copies).
    let audit: String = h
        .conn()
        .query_row(
            "SELECT group_concat(COALESCE(old_json, '') || COALESCE(new_json, '') || COALESCE(reason, ''), ' ') FROM audit_log",
            [],
            |r| r.get(0),
        )
        .unwrap();
    for password in [new, OWNER_PASSWORD] {
        assert!(!audit.contains(password), "an audit row holds a password");
    }
    for path in files_under(h.dir.path()) {
        let bytes = std::fs::read(&path).unwrap_or_default();
        for password in [new, OWNER_PASSWORD] {
            assert!(
                !bytes_contain(&bytes, password),
                "{} holds a password in plain text",
                path.display()
            );
        }
    }
}
