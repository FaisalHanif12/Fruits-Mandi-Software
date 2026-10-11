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
    // Give the worker threads a moment to flush their last lines.
    std::thread::sleep(std::time::Duration::from_millis(100));
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
            &"search_touch"
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
