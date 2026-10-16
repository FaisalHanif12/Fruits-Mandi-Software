//! The plan 04 commands through Tauri's real IPC and the real capability file: JSON shapes,
//! errors, and the sign-in check on every command (D1-A12, D1-A13, NFR-SEC-4). There are no
//! roles: the Owner is the only user (PRD Section 2).
#![cfg(test)]

mod common;

use std::sync::OnceLock;

use mandi_app_lib::state::AppState;
use mandi_services::testing::TestEnv;
use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tempfile::TempDir;

const OWNER_PASSWORD: &str = "secret1";

/// One global capturing subscriber for this test file: commands run on worker threads.
fn log() -> &'static common::Capture {
    static LOG: OnceLock<common::Capture> = OnceLock::new();
    LOG.get_or_init(|| {
        let capture = common::Capture::default();
        tracing::subscriber::set_global_default(mandi_app_lib::logging::subscriber(
            capture.clone(),
        ))
        .unwrap();
        capture
    })
}

struct Harness {
    _app: tauri::App<MockRuntime>,
    window: tauri::WebviewWindow<MockRuntime>,
    dir: TempDir,
}

impl Harness {
    fn new() -> Self {
        log();
        let dir = tempfile::tempdir().unwrap();
        let env = TestEnv::new(dir.path()).unwrap();
        let state = AppState::with_context(env.ctx.clone());
        let app = mock_builder()
            .manage(state)
            .invoke_handler(mandi_app_lib::handler())
            .build(mandi_app_lib::context())
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        Self {
            _app: app,
            window,
            dir,
        }
    }

    /// Calls a command like the screen does: `Ok(result json)` or `Err(UiError json)`.
    fn call(&self, command: &str, payload: Value) -> Result<Value, Value> {
        let request = InvokeRequest {
            cmd: command.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: self.window.url().unwrap(),
            body: InvokeBody::Json(payload),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        };
        get_ipc_response(&self.window, request).map(|body| body.deserialize::<Value>().unwrap())
    }

    fn ok(&self, command: &str, payload: Value) -> Value {
        self.call(command, payload)
            .unwrap_or_else(|e| panic!("{command} failed: {e}"))
    }

    fn code(&self, command: &str, payload: Value) -> String {
        let error = self.call(command, payload).unwrap_err();
        error["errors"][0]["code"].as_str().unwrap().to_owned()
    }

    fn login(&self, username: &str, password: &str) -> Result<Value, Value> {
        self.call(
            "auth_login",
            json!({ "input": { "username": username, "password": password } }),
        )
    }

    fn sql<T: rusqlite_free::FromSqlText>(&self, query: &str) -> T {
        let conn = mandi_services::db::rusqlite::Connection::open(
            self.dir.path().join("data").join("mandi.db"),
        )
        .unwrap();
        let text: String = conn
            .query_row(query, [], |r| r.get::<_, rusqlite_free::Text>(0))
            .unwrap()
            .0;
        T::from_text(&text)
    }

    /// Runs the wizard and signs the Owner in. Returns the recovery code.
    fn set_up_and_login(&self) -> String {
        let done = self.ok("setup_first_run", json!({ "input": first_run() }));
        self.ok_login("owner", OWNER_PASSWORD);
        done["recovery_code"].as_str().unwrap().to_owned()
    }

    fn ok_login(&self, username: &str, password: &str) -> Value {
        self.login(username, password)
            .unwrap_or_else(|e| panic!("login failed: {e}"))
    }
}

/// Reads one value of a query as text.
mod rusqlite_free {
    use mandi_services::db::rusqlite::types::{FromSql, FromSqlResult, ValueRef};

    pub struct Text(pub String);

    impl FromSql for Text {
        fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
            Ok(Text(match value {
                ValueRef::Integer(i) => i.to_string(),
                ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
                ValueRef::Null => String::new(),
                other => format!("{other:?}"),
            }))
        }
    }

    pub trait FromSqlText {
        fn from_text(text: &str) -> Self;
    }

    impl FromSqlText for String {
        fn from_text(text: &str) -> Self {
            text.to_owned()
        }
    }

    impl FromSqlText for i64 {
        fn from_text(text: &str) -> Self {
            text.parse().unwrap_or(-1)
        }
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

fn is_reference(text: &str) -> bool {
    text.len() == 15
        && text.starts_with("E-")
        && text.as_bytes().get(10) == Some(&b'-')
        && text[2..10].bytes().all(|c| c.is_ascii_digit())
        && text[11..].bytes().all(|c| c.is_ascii_digit())
}

#[test]
fn the_json_shapes_are_the_contract() {
    let h = Harness::new();
    let conf: Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    // Before the wizard.
    let status = h.ok("auth_status", json!({}));
    assert_eq!(
        status,
        json!({
            "first_run": true, "session": null, "locked": false, "login_language": "ur",
            "firm": { "name_ur": "", "name_en": "", "mandi_ur": "", "mandi_en": "" },
            "app_version": conf["version"],
        })
    );
    let done = h.ok("setup_first_run", json!({ "input": first_run() }));
    assert_eq!(
        done.as_object().unwrap().keys().collect::<Vec<_>>(),
        ["recovery_code"]
    );
    // Sign in: the SessionUser.
    let user = h.ok_login("Owner", OWNER_PASSWORD);
    assert_eq!(
        user,
        json!({ "id": 1, "username": "owner", "display_name": "Maher Adnan", "language": "ur", "text_size": 100 })
    );
    let status = h.ok("auth_status", json!({}));
    assert_eq!(status["first_run"], false);
    assert_eq!(status["session"], user);
    assert_eq!(status["firm"]["name_en"], "Maher Adnan and Brothers");
    // The rest of the read commands.
    let settings = h.ok("settings_get", json!({}));
    assert_eq!(settings["idle_lock_minutes"], 15);
    assert_eq!(settings["default_language"], "ur");
    assert_eq!(settings["number_grouping"], "international");
    assert_eq!(settings["firm"]["phone"], "03001234567");
    assert_eq!(h.ok("day_status", json!({}))["today_open"], true);
    assert_eq!(h.ok("day_status", json!({}))["closed_up_to"], Value::Null);
    // Writes return null or the saved value.
    assert_eq!(
        h.ok(
            "settings_set_security",
            json!({ "input": { "idle_lock_minutes": "20" } })
        ),
        Value::Null
    );
    assert_eq!(
        h.ok(
            "settings_set_default_language",
            json!({ "input": { "language": "en" } })
        ),
        Value::Null
    );
    assert_eq!(
        h.ok(
            "prefs_set",
            json!({ "input": { "key": "text_size", "value": "120" } })
        ),
        Value::Null
    );
    assert_eq!(
        h.ok(
            "prefs_set",
            json!({ "input": { "key": "language", "value": "en" } })
        ),
        Value::Null
    );
    assert_eq!(
        h.ok(
            "prefs_set",
            json!({ "input": { "key": "last_screen", "value": "/bikri" } })
        ),
        Value::Null
    );
    let firm = settings["firm"].clone();
    assert_eq!(
        h.ok("settings_set_firm", json!({ "input": firm })),
        settings["firm"]
    );
    assert_eq!(h.ok("auth_lock", json!({})), Value::Null);
    assert_eq!(h.ok("auth_status", json!({}))["locked"], true);
    let again = h.ok(
        "auth_unlock",
        json!({ "input": { "password": OWNER_PASSWORD } }),
    );
    assert_eq!(
        (again["language"].as_str(), again["text_size"].as_i64()),
        (Some("en"), Some(120))
    );
    assert_eq!(h.ok("auth_logout", json!({})), Value::Null);
}

#[test]
fn errors_arrive_as_ui_errors_with_codes_fields_values_and_a_reference() {
    let h = Harness::new();
    // Field errors of the wizard, all together.
    let mut bad = first_run();
    bad["firm_name_ur"] = json!("English");
    bad["password"] = json!("abc");
    bad["password_repeat"] = json!("abd");
    let error = h
        .call("setup_first_run", json!({ "input": bad }))
        .unwrap_err();
    assert_eq!(error["ok"], false);
    assert!(is_reference(error["ref"].as_str().unwrap()), "{error}");
    let items = error["errors"].as_array().unwrap();
    let find = |field: &str| {
        items
            .iter()
            .find(|i| i["field"] == field)
            .unwrap_or_else(|| panic!("{field} in {error}"))
    };
    assert_eq!(find("firm_name_ur")["code"], "ERR_NAME_UR_SCRIPT");
    assert_eq!(find("password")["code"], "ERR_PASSWORD_TOO_SHORT");
    assert_eq!(find("password")["values"], json!({ "min": 6 }));
    h.ok("setup_first_run", json!({ "input": first_run() }));
    // A wrong password: the field, and the tries left as a value.
    let error = h.login("owner", "nope-nope").unwrap_err();
    assert_eq!(
        error["errors"],
        json!([{ "code": "ERR_WRONG_PASSWORD", "field": "password", "values": { "tries_left": 4 } }])
    );
    // The fifth wrong try is the lock.
    for _ in 0..3 {
        h.login("owner", "nope-nope").unwrap_err();
    }
    let error = h.login("owner", "nope-nope").unwrap_err();
    assert_eq!(error["errors"][0]["code"], "ERR_LOCKED_WAIT");
    assert_eq!(error["errors"][0]["values"], json!({ "seconds": 60 }));
    assert_eq!(error["errors"][0]["field"], Value::Null);
    // The reference is written to the log with the codes.
    let reference = error["ref"].as_str().unwrap();
    assert!(
        log()
            .text()
            .lines()
            .any(|l| l.contains(reference) && l.contains("ErrLockedWait")),
        "{reference} not in the log"
    );
    // A second wizard.
    assert_eq!(
        h.code("setup_first_run", json!({ "input": first_run() })),
        "ERR_ALREADY_SET_UP"
    );
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
        (
            "auth_change_password",
            json!({ "input": { "current": "y", "new": "abcdef", "repeat": "abcdef" } }),
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
        ("translit_pairs_list", json!({}), true),
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
            "vouchers_save_advance",
            json!({ "input": { "date": "10-10-2026", "seller_account_id": 1, "amount": "15,000", "mode": "cash",
                "bank_account_id": null, "cheque_no": "", "cheque_date": "", "expected_goods": "", "narration": "" } }),
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
            "bikri_preview",
            json!({ "input": { "draft": { "header": { "entry_date": "10-10-2026" }, "lots": [] } } }),
            true,
        ),
        (
            "bikri_split_freight",
            json!({ "input": { "total_freight": "1,000",
                "lots": [{ "key": "a", "qty_received": "30" }, { "key": "b", "qty_received": "20" }] } }),
            true,
        ),
        (
            "assist_truck_memory",
            json!({ "input": { "truck_no": "LES", "date": "10-10-2026" } }),
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
            "search_pukhta_people",
            json!({ "input": { "text": "shahid", "side": "buyer" } }),
            true,
        ),
        (
            "pukhta_register",
            json!({ "input": { "from": "", "to": "", "side": null, "name": null,
                "phone": null, "state": null } }),
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
                "collector_staff_id": null, "collection_weekday": null, "is_active": true, "note": "" } }),
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
    ]
}

#[test]
fn every_registered_command_is_in_the_table() {
    let build_rs = include_str!("../build.rs");
    let listed = build_rs.split("APP_COMMANDS").nth(1).unwrap();
    let listed = &listed[listed.find('[').unwrap()..listed.find("];").unwrap()];
    let registered: Vec<String> = listed
        .split('"')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, name)| name.to_owned())
        .collect();
    let table: Vec<&str> = command_table().iter().map(|(name, _, _)| *name).collect();
    for name in &registered {
        assert!(
            table.contains(&name.as_str()),
            "{name} is registered but is not in the table"
        );
    }
    for name in &table {
        assert!(
            registered.iter().any(|r| r == name),
            "{name} is in the table but not registered"
        );
    }
    // And the capability file allows exactly these commands (plus the plugin permissions).
    let capability: Value =
        serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
    for name in &registered {
        let permission = format!("allow-{}", name.replace('_', "-"));
        assert!(
            capability["permissions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p == &permission),
            "{permission}"
        );
    }
}

/// A snapshot of what the protected commands could change.
fn snapshot(h: &Harness) -> (String, String, i64) {
    (
        h.sql::<String>("SELECT COALESCE(group_concat(id || username || role || is_active || version || password_hash, '|'), '') FROM users"),
        h.sql::<String>("SELECT COALESCE(group_concat(key || value_json || version, '|'), '') FROM settings"),
        h.sql::<i64>("SELECT COUNT(*) FROM user_prefs"),
    )
}

#[test]
fn without_a_session_every_protected_command_is_refused_without_a_row() {
    // D1-A13: every command except first run, sign-in, recovery, app info, data status and the
    // sign-in status (the screen needs it to know which page to show) refuses before sign-in
    // with ERR_NOT_LOGGED_IN, and writes nothing.
    let h = Harness::new();
    h.set_up_and_login();
    h.ok("auth_logout", json!({}));
    let rows = || h.sql::<i64>("SELECT COUNT(*) FROM audit_log");
    let (before, data) = (rows(), snapshot(&h));
    for (command, payload, protected) in command_table() {
        if !protected {
            assert!(
                matches!(
                    command,
                    "app_info"
                        | "app_data_status"
                        | "auth_status"
                        | "setup_first_run"
                        | "auth_login"
                        | "auth_recover"
                ),
                "{command} is open to everyone but is not on the list"
            );
            continue;
        }
        assert_eq!(h.code(command, payload), "ERR_NOT_LOGGED_IN", "{command}");
    }
    assert_eq!(rows(), before, "nothing was written");
    assert_eq!(snapshot(&h), data, "no table changed");
}

#[test]
fn while_locked_every_protected_command_is_refused_without_a_row() {
    // The idle lock (PRD FR-8.4): only status, unlock and logout work until the password is typed.
    let h = Harness::new();
    h.set_up_and_login();
    h.ok("auth_lock", json!({}));
    let rows = || h.sql::<i64>("SELECT COUNT(*) FROM audit_log");
    let (before, data) = (rows(), snapshot(&h));
    for (command, payload, protected) in command_table() {
        if !protected || matches!(command, "auth_unlock" | "auth_logout" | "auth_lock") {
            continue;
        }
        assert_eq!(h.code(command, payload), "ERR_SESSION_LOCKED", "{command}");
    }
    assert_eq!(rows(), before, "nothing was written while locked");
    assert_eq!(snapshot(&h), data, "no table changed while locked");
    assert_eq!(h.ok("auth_status", json!({}))["locked"], true);
    // The wrong password does not open it; the right one does, and the commands work again.
    assert_eq!(
        h.code(
            "auth_unlock",
            json!({ "input": { "password": "wrong-Zq77" } })
        ),
        "ERR_WRONG_PASSWORD"
    );
    h.ok(
        "auth_unlock",
        json!({ "input": { "password": OWNER_PASSWORD } }),
    );
    h.ok("settings_get", json!({}));
}

#[test]
fn recovery_through_the_ipc_spends_the_code() {
    let h = Harness::new();
    let code = h.set_up_and_login();
    h.ok("auth_logout", json!({}));
    let input = |code: &str, password: &str| json!({ "input": { "username": "owner", "recovery_code": code, "new_password": password, "new_password_repeat": password } });
    let done = h.ok("auth_recover", input(&code, "brand-new-1"));
    let new_code = done["new_recovery_code"].as_str().unwrap().to_owned();
    assert_ne!(new_code, code);
    h.ok_login("owner", "brand-new-1");
    h.ok("auth_logout", json!({}));
    assert_eq!(
        h.code("auth_recover", input(&code, "again-pass1")),
        "ERR_RECOVERY_CODE_WRONG"
    );
    assert!(
        h.call("auth_recover", input(&new_code, "third-pass1"))
            .is_ok()
    );
}

#[test]
fn passwords_and_codes_never_reach_the_log_or_the_debug_text() {
    let h = Harness::new();
    let code = h.set_up_and_login();
    h.login("owner", "wrong-Zq77").unwrap_err();
    h.call(
        "setup_first_run",
        json!({ "input": first_run_with("leaky-Pw88") }),
    )
    .unwrap_err();
    h.ok("auth_logout", json!({}));
    h.call("auth_recover", json!({ "input": { "username": "owner", "recovery_code": "ABCD-EFGH-JKMN-PQRS", "new_password": "recover-Pw99", "new_password_repeat": "recover-Pw99" } })).unwrap_err();
    // The worker threads write their last lines a moment after the answer: wait for the line
    // we need, up to five seconds (a busy computer running every test at once can be slow).
    for _ in 0..50 {
        if log().text().contains("ErrWrongPassword") {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let text = log().text();
    for secret in [
        OWNER_PASSWORD,
        "wrong-Zq77",
        "leaky-Pw88",
        "recover-Pw99",
        "ABCD-EFGH-JKMN-PQRS",
        code.as_str(),
        "$argon2",
    ] {
        assert!(!text.contains(secret), "{secret} leaked into the log");
    }
    assert!(text.contains("ErrWrongPassword"), "the codes are logged");
}

#[test]
fn inputs_hide_their_secrets_in_debug_text() {
    use mandi_app_lib::commands::auth::{FirstRunInput, LoginInput, RecoverInput, UnlockInput};
    let login: LoginInput =
        serde_json::from_value(json!({ "username": "owner", "password": "secret-A1" })).unwrap();
    let unlock: UnlockInput = serde_json::from_value(json!({ "password": "secret-B2" })).unwrap();
    let recover: RecoverInput = serde_json::from_value(json!({ "username": "o", "recovery_code": "CODE-C3", "new_password": "secret-D4", "new_password_repeat": "secret-D4" })).unwrap();
    let first: FirstRunInput = serde_json::from_value(first_run_with("secret-E5")).unwrap();
    let text = format!("{login:?} {unlock:?} {recover:?} {first:?}");
    for secret in [
        "secret-A1",
        "secret-B2",
        "CODE-C3",
        "secret-D4",
        "secret-E5",
    ] {
        assert!(!text.contains(secret), "{secret} in {text}");
    }
    assert!(text.contains("[hidden]") && text.contains("owner"));
}

fn first_run_with(password: &str) -> Value {
    let mut input = first_run();
    input["password"] = json!(password);
    input["password_repeat"] = json!(password);
    input
}

#[test]
fn bad_json_never_reaches_the_service() {
    let h = Harness::new();
    h.set_up_and_login();
    // A language that does not exist is refused by the IPC layer, and nothing is written.
    let before = h.sql::<i64>("SELECT COUNT(*) FROM audit_log");
    let result = h.call(
        "settings_set_default_language",
        json!({ "input": { "language": "klingon" } }),
    );
    assert!(result.is_err());
    assert_eq!(h.sql::<i64>("SELECT COUNT(*) FROM audit_log"), before);
    let result = h.call(
        "prefs_set",
        json!({ "input": { "key": "colour", "value": "red" } }),
    );
    assert!(result.is_err());
}

#[test]
fn the_translit_commands_work_through_the_ipc() {
    let h = Harness::new();
    h.set_up_and_login();
    // Suggestions: the plan examples, in the JSON shape the screen reads.
    let up = |text: &str| {
        h.ok(
            "translit_suggest",
            json!({ "input": { "text": text, "direction": "to_urdu" } }),
        )
    };
    assert_eq!(up("tariq mehmood")["items"][0]["text"], "طارق محمود");
    assert_eq!(up("tariq mehmood")["items"][0]["source"], "word_list");
    assert_eq!(up("")["items"], json!([]));
    let down = h.ok(
        "translit_suggest",
        json!({ "input": { "text": "ارشد ملتانی", "direction": "to_english" } }),
    );
    assert_eq!(down["items"].as_array().unwrap().len(), 1);
    assert_eq!(down["items"][0]["text"], "Arshad Multani");
    assert_eq!(down["items"][0]["source"], "word_list");
    // Too long: the field and the limit.
    let error = h
        .call(
            "translit_suggest",
            json!({ "input": { "text": "a".repeat(501), "direction": "to_urdu" } }),
        )
        .unwrap_err();
    assert_eq!(
        error["errors"],
        json!([{ "code": "ERR_TEXT_TOO_LONG", "field": "text", "values": { "max": 500 } }])
    );
    // A direction that does not exist is refused by the IPC layer.
    assert!(
        h.call(
            "translit_suggest",
            json!({ "input": { "text": "ali", "direction": "sideways" } })
        )
        .is_err()
    );
    // Learning: nothing comes back, the next suggestion has it first, the list shows it.
    assert_eq!(
        h.ok(
            "translit_learn",
            json!({ "input": { "roman": "xyz", "urdu": "زیڈ", "kind": "word" } })
        ),
        Value::Null
    );
    assert_eq!(up("xyz")["items"][0]["text"], "زیڈ");
    assert_eq!(up("xyz")["items"][0]["source"], "learned");
    let list = h.ok("translit_pairs_list", json!({}));
    assert_eq!(list.as_array().unwrap().len(), 1);
    let row = &list[0];
    assert_eq!(
        (
            row["roman"].as_str(),
            row["urdu"].as_str(),
            row["kind"].as_str()
        ),
        (Some("xyz"), Some("زیڈ"), Some("word"))
    );
    assert_eq!(row["use_count"], 1);
    assert!(row["last_used_at"].is_string() && row["id"].is_number());
    // Errors name their field.
    let error = h
        .call(
            "translit_learn",
            json!({ "input": { "roman": "زیڈ", "urdu": "xyz", "kind": "name" } }),
        )
        .unwrap_err();
    let items = error["errors"].as_array().unwrap();
    let find = |field: &str| items.iter().find(|i| i["field"] == field).unwrap()["code"].clone();
    assert_eq!(find("roman"), "ERR_NAME_EN_SCRIPT");
    assert_eq!(find("urdu"), "ERR_NAME_UR_SCRIPT");
    // A kind that does not exist is refused by the IPC layer.
    assert!(
        h.call(
            "translit_learn",
            json!({ "input": { "roman": "xyz", "urdu": "زیڈ", "kind": "colour" } })
        )
        .is_err()
    );
    // Removing.
    let id = row["id"].clone();
    assert_eq!(
        h.ok("translit_pairs_remove", json!({ "input": { "id": id } })),
        Value::Null
    );
    assert_eq!(
        h.code("translit_pairs_remove", json!({ "input": { "id": id } })),
        "ERR_NOT_FOUND"
    );
    assert_ne!(up("xyz")["items"][0]["text"], "زیڈ");
    assert_eq!(h.ok("translit_pairs_list", json!({})), json!([]));
}

#[test]
fn the_masters_and_search_commands_work_through_the_ipc() {
    let h = Harness::new();
    h.set_up_and_login();
    let save = |input: Value| h.call("masters_save", json!({ "input": input }));
    let city = |code: &str, ur: &str, en: &str, confirmed: &[&str]| {
        json!({ "kind": "city", "id": null, "version": null, "code": code,
                "name_ur": ur, "name_en": en, "confirmed_rules_words": confirmed })
    };
    // Add: the row comes back tagged by kind, with its version.
    let lahore = save(city("", "لاہور", "Lahore", &[])).unwrap();
    assert_eq!(
        lahore,
        json!({ "kind": "city", "id": 1, "version": 1, "code": "1", "name_ur": "لاہور",
                "name_en": "Lahore", "is_active": true })
    );
    assert_eq!(
        h.ok("masters_next_code", json!({ "input": { "kind": "city" } })),
        json!("2")
    );
    assert_eq!(
        h.ok("masters_next_code", json!({ "input": { "kind": "staff" } })),
        json!("")
    );
    // The error shape: code and field.
    let taken = save(city("1", "ملتان", "Multan", &[])).unwrap_err();
    assert_eq!(taken["errors"][0]["code"], "ERR_CODE_TAKEN");
    assert_eq!(taken["errors"][0]["field"], "code");
    let script = save(city("", "Multan", "ملتان", &[])).unwrap_err();
    let fields: Vec<(&str, &str)> = script["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| (e["field"].as_str().unwrap(), e["code"].as_str().unwrap()))
        .collect();
    assert_eq!(
        fields,
        [
            ("name_ur", "ERR_NAME_UR_SCRIPT"),
            ("name_en", "ERR_NAME_EN_SCRIPT")
        ]
    );
    // A spelling only the rules can make is refused until it is confirmed.
    let rules = save(city("", "علی قزکس", "Ali Qzxwv", &[])).unwrap_err();
    assert_eq!(rules["errors"][0]["code"], "ERR_SPELLING_UNCONFIRMED");
    assert_eq!(rules["errors"][0]["field"], "name_ur");
    assert_eq!(rules["errors"][0]["values"]["words"], "قزکس");
    save(city("", "علی قزکس", "Ali Qzxwv", &["قزکس", "Qzxwv"])).unwrap();
    // An edit with a stale version.
    let stale = save(
        json!({ "kind": "city", "id": 1, "version": 99, "code": "", "name_ur": "لاہور",
        "name_en": "Lahore", "confirmed_rules_words": [] }),
    )
    .unwrap_err();
    assert_eq!(stale["errors"][0]["code"], "ERR_VERSION_CHANGED");
    // List and make inactive.
    let list = h.ok(
        "masters_list",
        json!({ "input": { "kind": "city", "include_inactive": false } }),
    );
    assert_eq!(list.as_array().unwrap().len(), 2);
    let off = h.ok(
        "masters_set_active",
        json!({ "input": { "kind": "city", "id": 1, "is_active": false } }),
    );
    assert_eq!(off["is_active"], false);
    assert_eq!(
        h.ok(
            "masters_list",
            json!({ "input": { "kind": "city", "include_inactive": false } })
        )
        .as_array()
        .unwrap()
        .len(),
        1
    );
    // Search: the shape the Picker reads, in both scripts; an inactive record is not offered.
    let hits = h.ok(
        "search_masters",
        json!({ "input": { "scope": "city", "text": "qzxw", "limit": 10 } }),
    );
    assert_eq!(
        hits,
        json!([{ "id": 2, "kind": "city", "code": "2", "name_ur": "علی قزکس", "name_en": "Ali Qzxwv",
                 "extra_ur": "", "extra_en": "", "is_active": true }])
    );
    assert_eq!(
        h.ok(
            "search_masters",
            json!({ "input": { "scope": "city", "text": "lahore", "limit": 10 } })
        ),
        json!([])
    );
    h.ok(
        "search_touch",
        json!({ "input": { "scope": "city", "id": 2 } }),
    );
    let recent = h.ok("search_recent", json!({ "input": { "scope": "city" } }));
    assert_eq!(recent[0]["id"], 2);
    assert_eq!(
        h.code(
            "search_touch",
            json!({ "input": { "scope": "city", "id": 77 } })
        ),
        "ERR_NOT_FOUND"
    );
    // A kind that does not exist is refused by the IPC layer.
    assert!(
        h.call(
            "masters_list",
            json!({ "input": { "kind": "colour", "include_inactive": true } })
        )
        .is_err()
    );
}

#[test]
fn there_is_no_command_that_deletes_a_master() {
    // PRD E21, Code Standards D3: a master can only be made inactive. The app exposes exactly
    // these commands for the masters and none of them deletes.
    let build_rs = include_str!("../build.rs");
    let listed = build_rs.split("APP_COMMANDS").nth(1).unwrap();
    let listed = &listed[listed.find('[').unwrap()..listed.find("];").unwrap()];
    let names: Vec<&str> = listed
        .split('"')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, name)| name)
        .collect();
    let masters: Vec<&&str> = names
        .iter()
        .filter(|n| n.starts_with("masters_") || n.starts_with("search_"))
        .collect();
    assert_eq!(
        masters,
        [
            &"masters_list",
            &"masters_save",
            &"masters_set_active",
            &"masters_next_code",
            &"search_masters",
            &"search_recent",
            &"search_touch",
            &"search_accounts",
            &"search_global",
            &"search_accounts_recent",
            &"search_accounts_touch",
            &"search_pukhta_people"
        ]
    );
    for name in &names {
        assert!(
            !(name.contains("delete")
                || name.ends_with("_remove") && !name.starts_with("translit_")),
            "{name} looks like a delete command"
        );
    }
}

#[test]
fn change_password_works_through_the_ipc_and_counts_wrong_tries() {
    // End-gate review 12: the command calls the sign-in check first, then the service.
    let h = Harness::new();
    h.set_up_and_login();
    let change = |current: &str, new: &str, repeat: &str| json!({ "input": { "current": current, "new": new, "repeat": repeat } });
    // Field errors name the fields of the input.
    let error = h
        .call("auth_change_password", change("", "short", "short"))
        .unwrap_err();
    let items = error["errors"].as_array().unwrap();
    let find = |field: &str| {
        items
            .iter()
            .find(|i| i["field"] == field)
            .map(|i| i["code"].clone())
    };
    assert_eq!(find("current"), Some(json!("ERR_REQUIRED")));
    assert_eq!(find("new"), Some(json!("ERR_PASSWORD_TOO_SHORT")));
    let error = h
        .call(
            "auth_change_password",
            change("nope-nope", "brand-new-9", "brand-new-8"),
        )
        .unwrap_err();
    assert_eq!(error["errors"][0]["code"], "ERR_PASSWORDS_DIFFER");
    assert_eq!(error["errors"][0]["field"], "repeat");
    // A wrong current password: the field and the tries left.
    let error = h
        .call(
            "auth_change_password",
            change("nope-nope", "brand-new-9", "brand-new-9"),
        )
        .unwrap_err();
    assert_eq!(
        error["errors"],
        json!([{ "code": "ERR_WRONG_PASSWORD", "field": "current", "values": { "tries_left": 4 } }])
    );
    // A right one: null, and the new password signs in.
    assert_eq!(
        h.ok(
            "auth_change_password",
            change(OWNER_PASSWORD, "brand-new-9", "brand-new-9")
        ),
        Value::Null
    );
    h.ok("auth_logout", json!({}));
    assert!(h.login("owner", OWNER_PASSWORD).is_err());
    h.ok_login("owner", "brand-new-9");
}

#[test]
fn the_firm_spelling_check_works_through_the_ipc() {
    // End-gate review 3: the wizard and Settings both take `confirmed_rules_words`.
    let h = Harness::new();
    let mut input = first_run();
    input["firm_name_ur"] = json!("علی قزکس");
    input["firm_name_en"] = json!("Ali Qzxwv");
    let error = h
        .call("setup_first_run", json!({ "input": input.clone() }))
        .unwrap_err();
    let items = error["errors"].as_array().unwrap();
    let got: Vec<(&str, &str, &str)> = items
        .iter()
        .map(|i| {
            (
                i["code"].as_str().unwrap(),
                i["field"].as_str().unwrap(),
                i["values"]["words"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            ("ERR_SPELLING_UNCONFIRMED", "firm_name_ur", "قزکس"),
            ("ERR_SPELLING_UNCONFIRMED", "firm_name_en", "Qzxwv"),
        ]
    );
    input["confirmed_rules_words"] = json!(["قزکس", "Qzxwv"]);
    h.ok("setup_first_run", json!({ "input": input }));
    h.ok_login("owner", OWNER_PASSWORD);

    // Settings: a new rules-only word is refused until it is sent back as confirmed.
    let mut firm = h.ok("settings_get", json!({}))["firm"].clone();
    firm["mandi_ur"] = json!("وزبنم منڈی");
    firm["mandi_en"] = json!("Wvbnm Mandi");
    let error = h
        .call("settings_set_firm", json!({ "input": firm.clone() }))
        .unwrap_err();
    assert_eq!(error["errors"][0]["code"], "ERR_SPELLING_UNCONFIRMED");
    assert_eq!(error["errors"][0]["field"], "mandi_ur");
    assert_eq!(error["errors"][1]["field"], "mandi_en");
    assert_eq!(
        h.ok("settings_get", json!({}))["firm"]["mandi_en"],
        "New Sabzi Mandi Kot Radha Kishan",
        "nothing saved on error"
    );
    firm["confirmed_rules_words"] = json!(["وزبنم", "Wvbnm"]);
    let saved = h.ok("settings_set_firm", json!({ "input": firm }));
    assert_eq!(saved["mandi_en"], "Wvbnm Mandi");
    assert!(
        saved.get("confirmed_rules_words").is_none(),
        "an output has no input field"
    );
}

#[test]
fn the_partners_commands_work_through_the_ipc() {
    // Plan 11: partners, the shares and the overview, as the screens call them (PRD 17.1).
    let h = Harness::new();
    h.set_up_and_login();
    let create = |ur: &str, en: &str, opening: Value| {
        h.call(
            "partners_create",
            json!({ "input": { "name_ur": ur, "name_en": en, "opening_capital": opening,
                "confirmed_rules_words": [] } }),
        )
    };
    let a = create("الف", "Partner A", Value::Null).unwrap();
    assert_eq!(
        a,
        json!({ "id": 1, "name_ur": "الف", "name_en": "Partner A", "capital_account_id": a["capital_account_id"],
                "capital_account_code": "06020001", "drawings_account_id": a["drawings_account_id"],
                "drawings_account_code": "06030001", "share_bps": 0, "is_active": true, "version": 1 })
    );
    let b = create("ب", "Partner B", json!("50,000")).unwrap();
    let c = create("ج", "Partner C", Value::Null).unwrap();
    // A bad opening capital names its field.
    let bad = create("د", "Partner D", json!("1.5")).unwrap_err();
    assert_eq!(bad["errors"][0]["code"], "ERR_DECIMALS_NOT_ALLOWED");
    assert_eq!(bad["errors"][0]["field"], "opening_capital");

    let shares = |rows: &[(&Value, &str)]| -> Value {
        json!(
            rows.iter()
                .map(|(p, s)| json!({ "partner_id": p["id"], "share": s }))
                .collect::<Vec<_>>()
        )
    };
    // The running total comes from the backend (rule 1: the screen never adds percents).
    let check = h.ok(
        "partners_check_shares",
        json!({ "input": { "shares": shares(&[(&a, "40"), (&b, "30"), (&c, "20")]) } }),
    );
    assert_eq!(check["total_bps"], 9_000);
    assert_eq!(check["is_100"], false);
    assert_eq!(check["errors"][0]["code"], "ERR_SHARES_NOT_100");
    assert_eq!(check["errors"][0]["field"], "shares");
    assert_eq!(check["errors"][0]["values"]["total_bps"], 9_000);
    let good = h.ok(
        "partners_check_shares",
        json!({ "input": { "shares": shares(&[(&a, "40"), (&b, "30"), (&c, "30")]) } }),
    );
    assert_eq!(
        (good["total_bps"].as_i64(), good["is_100"].as_bool()),
        (Some(10_000), Some(true))
    );

    // Saving the same 90% is refused with the same code; back-dating is refused too.
    let refused = h
        .call(
            "partners_set_shares",
            json!({ "input": { "effective_from": "10-10-2026",
                "shares": shares(&[(&a, "40"), (&b, "30"), (&c, "20")]) } }),
        )
        .unwrap_err();
    assert_eq!(refused["errors"][0]["code"], "ERR_SHARES_NOT_100");
    let early = h
        .call(
            "partners_set_shares",
            json!({ "input": { "effective_from": "09-10-2026",
                "shares": shares(&[(&a, "40"), (&b, "30"), (&c, "30")]) } }),
        )
        .unwrap_err();
    assert_eq!(early["errors"][0]["code"], "ERR_SHARE_BACKDATED");
    assert_eq!(early["errors"][0]["field"], "effective_from");
    assert_eq!(early["errors"][0]["values"]["today"], "10-10-2026");

    let saved = h.ok(
        "partners_set_shares",
        json!({ "input": { "effective_from": "10-10-2026",
            "shares": shares(&[(&a, "40"), (&b, "30"), (&c, "30")]) } }),
    );
    let bps: Vec<i64> = saved
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["share_bps"].as_i64().unwrap())
        .collect();
    assert_eq!(bps, [4_000, 3_000, 3_000]);
    assert_eq!(h.ok("partners_list", json!({})), saved);

    let detail = h.ok("partners_get", json!({ "input": { "id": 1 } }));
    assert_eq!(detail["row"]["share_bps"], 4_000);
    assert_eq!(
        detail["shares"],
        json!([{ "share_bps": 4_000, "effective_from": "2026-10-10" }])
    );
    let missing = h
        .call("partners_get", json!({ "input": { "id": 99 } }))
        .unwrap_err();
    assert_eq!(missing["errors"][0]["code"], "ERR_NOT_FOUND");

    // An edit with a stale version, and the overview with money as plain numbers.
    let stale = h
        .call(
            "partners_update",
            json!({ "input": { "id": 1, "version": 9, "name_ur": "الف", "name_en": "Partner A",
                "is_active": true, "confirmed_rules_words": [] } }),
        )
        .unwrap_err();
    assert_eq!(stale["errors"][0]["code"], "ERR_VERSION_CHANGED");
    let overview = h.ok("partners_overview", json!({}));
    let rows = overview.as_array().unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[1]["partner_id"], 2);
    assert_eq!(rows[1]["share_bps"], 3_000);
    assert_eq!(
        rows[1]["capital_balance_paisa"], 5_000_000,
        "50,000 rupees in paisa"
    );
    assert_eq!(rows[0]["today_income_paisa"], 0);
}

#[test]
fn the_bikri_commands_work_through_the_ipc() {
    // Plan 12: Event 1 of PRD 4.4 typed as text, answered in paisa, and the memories.
    let h = Harness::new();
    h.set_up_and_login();
    let master = |kind: &str, ur: &str, en: &str, extra: Value| -> i64 {
        let mut input = json!({ "kind": kind, "id": null, "version": null, "code": "",
            "name_ur": ur, "name_en": en, "confirmed_rules_words": [] });
        for (key, value) in extra.as_object().unwrap() {
            input[key] = value.clone();
        }
        h.ok("masters_save", json!({ "input": input }))["id"]
            .as_i64()
            .unwrap()
    };
    let account = |kind: &str, ur: &str, en: &str| -> i64 {
        h.ok(
            "accounts_save",
            json!({ "input": { "id": null, "version": null, "account_type": kind, "control_id": null,
                "code": "", "name_ur": ur, "name_en": en, "confirmed_rules_words": [],
                "father_or_firm_ur": "", "father_or_firm_en": "", "city_id": null, "adda_id": null,
                "phone1": "", "phone2": "", "address": "", "linked_account_id": null,
                "opening_date": "", "opening_dr": "", "opening_cr": "", "opening_is_advance": false,
                "credit_allowed": true, "credit_limit": "", "default_marka_id": null,
                "collector_staff_id": null, "collection_weekday": null, "is_active": true,
                "note": "" } }),
        )["id"]
            .as_i64()
            .unwrap()
    };
    let rahim = account("seller", "رحیم", "Rahim");
    let akram = account("buyer", "اکرم ٹول ٹیکس", "Akram Toll Tax");
    let arshad = account("buyer", "ارشد ملتانی", "Arshad Multani");
    let banana = master(
        "item",
        "کیلا",
        "Banana",
        json!({ "item_group": "fruit", "default_packing_id": null, "sold_by": "unit" }),
    );
    let packing = master("packing", "پیٹی", "Box", json!({ "approx_kg": "" }));
    let mut partner_ids = Vec::new();
    for (ur, en) in [("الف", "Partner A"), ("ب", "Partner B"), ("ج", "Partner C")] {
        let row = h.ok(
            "partners_create",
            json!({ "input": { "name_ur": ur, "name_en": en, "opening_capital": null,
                "confirmed_rules_words": [] } }),
        );
        partner_ids.push(row["id"].as_i64().unwrap());
    }
    h.ok(
        "partners_set_shares",
        json!({ "input": { "effective_from": "10-10-2026", "shares": [
            { "partner_id": partner_ids[0], "share": "40" },
            { "partner_id": partner_ids[1], "share": "30" },
            { "partner_id": partner_ids[2], "share": "30" } ] } }),
    );

    let sale = |key: &str, buyer: i64, qty: &str, rate: &str| {
        json!({ "key": key, "buyer_account_id": buyer, "pukhta": null, "qty": qty, "weight": null,
            "seller_rate": rate, "rate_differs": false, "buyer_rate": null, "payment_type": null,
            "seller_comm": null, "buyer_comm": null, "laga": null, "note": null })
    };
    let event1 = json!({ "input": { "draft": {
        "header": { "entry_date": "10-10-2026", "bill_date": "", "book": null, "truck_no": "LES-1234",
            "zamindar_account_id": rahim, "adda_id": null, "driver_name": null, "driver_phone": null,
            "total_freight": null, "store_id": null, "note": null },
        "lots": [{ "key": "lot-1", "seller_account_id": rahim, "pukhta": null, "item_id": banana,
            "packing_id": packing, "marka_id": null, "store_id": null, "qty_received": "50",
            "weight_received": null, "freight": "3,000", "freight_paid_by": "firm",
            "labour": { "method": "per_unit", "value": "5" },
            "munshiana": { "method": "manual", "value": "50" },
            "phone": { "method": "manual", "value": "20" },
            "cash_paid": "5,000", "store_rent": null, "adjust_advance": true,
            "sales": [sale("s-1", akram, "20", "600"), sale("s-2", arshad, "30", "580")] }]
    } } });
    let p = h.ok("bikri_preview", event1.clone());
    assert_eq!(p["errors"], json!([]), "{p}");
    assert_eq!(p["warnings"], json!([]));
    assert_eq!(p["allow_line_override"], true);
    // Money is paisa in plain JSON numbers: 19,316 rupees net, 31,564 of buyer bills.
    assert_eq!(p["summary"]["net"], 1_931_600);
    assert_eq!(p["summary"]["buyer_credit"], 3_156_400);
    assert_eq!(p["summary"]["shared_income"], 394_800);
    let line = &p["lots"][0]["lines"][0];
    assert_eq!(line["calc"]["buyer_total"], 1_288_000);
    assert_eq!(
        line["calc"]["seller_comm"]["working"]["message_id"],
        "working-percent"
    );
    assert_eq!(line["calc"]["seller_comm"]["source"], "default");
    assert_eq!(
        p["lots"][0]["lines"][0]["rate_hint"],
        Value::Null,
        "the first line has no hint"
    );
    assert_eq!(
        p["lots"][0]["lines"][1]["rate_hint"],
        json!({ "rate_paisa": 60_000, "source": "this_bikri", "date": "2026-10-10" }),
        "the second line is shown the rate of the first"
    );
    let split: Vec<i64> = p["split"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["income_paisa"].as_i64().unwrap())
        .collect();
    assert_eq!(split, [158_000, 118_400, 118_400]);
    // No cash was put in this database, so Event 1 takes cash below zero and asks.
    assert_eq!(p["cash"]["cash_after_paisa"], -500_000);
    assert_eq!(p["cash"]["needs_confirm"], true);

    // The freight split (C17) and its error shape.
    let parts = h.ok(
        "bikri_split_freight",
        json!({ "input": { "total_freight": "1,000",
            "lots": [{ "key": "a", "qty_received": "30" }, { "key": "b", "qty_received": "20" }] } }),
    );
    assert_eq!(
        parts,
        json!([{ "key": "a", "freight_paisa": 60_000 }, { "key": "b", "freight_paisa": 40_000 }])
    );
    let bad = h
        .call(
            "bikri_split_freight",
            json!({ "input": { "total_freight": "10.5", "lots": [{ "key": "a", "qty_received": "30" }] } }),
        )
        .unwrap_err();
    assert_eq!(bad["errors"][0]["code"], "ERR_DECIMALS_NOT_ALLOWED");
    assert_eq!(bad["errors"][0]["field"], "total_freight");

    // The memories read nothing yet.
    let truck = h.ok(
        "assist_truck_memory",
        json!({ "input": { "truck_no": "LES", "date": "10-10-2026" } }),
    );
    assert_eq!(
        truck,
        json!({ "last": null, "same_day": null, "suggestions": [] })
    );
    let seller = h.ok(
        "assist_seller_memory",
        json!({ "input": { "seller_account_id": rahim } }),
    );
    assert_eq!(
        seller,
        json!({ "item_id": null, "packing_id": null, "marka_id": null, "from_last_lot": false })
    );
    assert_eq!(
        h.code(
            "assist_seller_memory",
            json!({ "input": { "seller_account_id": akram } })
        ),
        "ERR_NOT_A_SELLER"
    );
    assert_eq!(
        h.ok(
            "assist_last_rate",
            json!({ "input": { "item_id": banana, "packing_id": packing, "date": "10-10-2026" } })
        ),
        Value::Null
    );
    // The new settings fields start on.
    let overrides = h.ok("settings_get_overrides", json!({}));
    assert_eq!(overrides["rate_check_enabled"], true);
    assert_eq!(overrides["rate_check_percent"], 30);

    // Plan 13: the save, the reads, the drafts and the global search through the IPC.
    let draft = event1["input"]["draft"].clone();
    let save_input = |confirm: bool| json!({ "input": { "draft": draft.clone(), "confirm_cash_below_zero": confirm } });
    // No cash was put in, so Event 1 asks for the confirmation and writes nothing.
    let asked = h.call("bikri_save", save_input(false)).unwrap_err();
    assert_eq!(
        asked["errors"][0]["code"],
        "ERR_CASH_BELOW_ZERO_NOT_CONFIRMED"
    );
    assert_eq!(asked["errors"][0]["values"]["cash_after_paisa"], -500_000);
    assert_eq!(h.sql::<i64>("SELECT COUNT(*) FROM bikri_entries"), 0);
    // A draft is kept, then removed by the save.
    h.ok(
        "drafts_save",
        json!({ "input": { "screen": "bikri", "payload_json": "{\"typing\":true}" } }),
    );
    let kept = h.ok("drafts_get", json!({ "input": { "screen": "bikri" } }));
    assert_eq!(kept["payload_json"], "{\"typing\":true}");
    assert_eq!(
        h.code(
            "drafts_save",
            json!({ "input": { "screen": "bikri", "payload_json": "not json" } })
        ),
        "ERR_DRAFT_INVALID"
    );
    let saved = h.ok("bikri_save", save_input(true));
    assert_eq!(saved["number"], 1);
    assert_eq!(saved["bikri_id"], 1);
    // The answer carries the numbers that were saved, in paisa.
    assert_eq!(saved["preview"]["summary"]["net"], 1_931_600);
    assert_eq!(
        h.ok("drafts_get", json!({ "input": { "screen": "bikri" } })),
        Value::Null
    );
    // Reads.
    let detail = h.ok("bikri_get", json!({ "input": { "id": 1 } }));
    assert_eq!(detail["lots"][0]["net"], 1_931_600);
    assert_eq!(detail["lots"][0]["sales"][0]["buyer_total"], 1_288_000);
    assert_eq!(detail["status"], "saved");
    let listed = h.ok("bikri_list", json!({ "input": { "date": "10-10-2026" } }));
    assert_eq!(listed[0]["number"], 1);
    assert_eq!(listed[0]["buyer_total"], 3_156_400);
    let found = h.ok("bikri_find", json!({ "input": { "truck_no": "les" } }));
    assert_eq!(found[0]["id"], 1);
    assert_eq!(
        h.ok(
            "bikri_navigate",
            json!({ "input": { "current_id": null, "to": "last" } })
        ),
        json!(1)
    );
    assert_eq!(
        h.ok(
            "bikri_navigate",
            json!({ "input": { "current_id": 1, "to": "next" } })
        ),
        Value::Null
    );
    assert_eq!(
        h.code("bikri_get", json!({ "input": { "id": 99 } })),
        "ERR_NOT_FOUND"
    );
    let register = h.ok("pukhta_register", json!({ "input": {} }));
    assert_eq!(register, json!([]));
    assert_eq!(
        h.ok(
            "search_pukhta_people",
            json!({ "input": { "text": "", "side": "buyer" } })
        ),
        json!([])
    );
    let hits = h.ok(
        "search_global",
        json!({ "input": { "text": "LES-12", "limit": 10 } }),
    );
    assert!(
        hits.as_array()
            .unwrap()
            .iter()
            .any(|hit| hit["kind"] == "bikri" && hit["hit"]["number"] == 1),
        "{hits}"
    );
    // The books balance after the save through the IPC.
    let (dr, cr): (i64, i64) = {
        let conn = mandi_services::db::rusqlite::Connection::open(
            h.dir.path().join("data").join("mandi.db"),
        )
        .unwrap();
        conn.query_row(
            "SELECT COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0) FROM journal_lines",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
    };
    assert_eq!((dr, cr), (3_656_400, 3_656_400));
}
