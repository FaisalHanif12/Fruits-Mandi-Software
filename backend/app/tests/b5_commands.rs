//! The commands of plan 22 early (step B5) through Tauri's real IPC and the real capability file:
//! the Day settings and the number grouping, the audit viewer, the health check and the support
//! information file, with the request and answer pairs the frontend copies
//! (`backend/app/tests/fixtures/b5_*.json`).
//!
//! A pair is `<name>_request.json` (exactly the `input` the command takes) and `<name>.json` (the
//! answer, or the error shape `ok`, `errors`, `ref`). What changes from run to run is replaced
//! before comparing: the reference number, times, the file path and the size. The test FAILS when
//! an answer differs from its file. After a deliberate change write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-app --test b5_commands`, read the diff and tell
//! `frontend`. The updates commands have their own test, `updates.rs`.
#![cfg(test)]

mod common;

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use mandi_app_lib::backup_runtime::BackupRuntime;
use mandi_app_lib::state::AppState;
use mandi_backup::cloud::{FixedCloud, LocalFolderCloud};
use mandi_backup::daily_pack::PendingPack;
use mandi_backup::io::NoHook;
use mandi_backup::secrets::MemorySecretStore;
use mandi_backup::service::{BackupConfig, BackupParts, BackupService};
use mandi_services::db::disk::FixedDiskProbe;
use mandi_services::testing::TestEnv;
use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tempfile::TempDir;

const OWNER_PASSWORD: &str = "secret1";
const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn log() -> &'static common::Capture {
    static LOG: OnceLock<common::Capture> = OnceLock::new();
    LOG.get_or_init(|| {
        let capture = common::Capture::default();
        // Another test file of this binary may have set it; one is enough.
        let _ = tracing::subscriber::set_global_default(mandi_app_lib::logging::subscriber(
            capture.clone(),
        ));
        capture
    })
}

struct Harness {
    _app: tauri::App<MockRuntime>,
    window: tauri::WebviewWindow<MockRuntime>,
    _env: TestEnv,
    root: TempDir,
    _dir: TempDir,
}

impl Harness {
    fn new() -> Self {
        log();
        let dir = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let env = TestEnv::new(dir.path()).unwrap();
        let service = Arc::new(BackupService::new(
            Arc::clone(env.ctx.db()),
            BackupConfig {
                backups_dir: root.path().join("backups"),
                app_version: "0.0.0-test".to_owned(),
            },
            BackupParts {
                secrets: Arc::new(MemorySecretStore::default()),
                cloud: Arc::new(FixedCloud(Arc::new(LocalFolderCloud::new(
                    root.path().join("cloud"),
                )))),
                disk: Arc::new(FixedDiskProbe(10 * 1024 * 1024 * 1024)),
                hook: Arc::new(NoHook),
                pack: Arc::new(PendingPack),
            },
        ));
        let app = mock_builder()
            .manage(AppState::with_context(env.ctx.clone()))
            .manage(BackupRuntime::new(service))
            .invoke_handler(mandi_app_lib::handler())
            .build(mandi_app_lib::context())
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        Self {
            _app: app,
            window,
            _env: env,
            root,
            _dir: dir,
        }
    }

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

    fn sign_in(&self) {
        self.ok(
            "setup_first_run",
            json!({ "input": {
                "firm_name_ur": "ماہر عدنان اینڈ برادرز", "firm_name_en": "Maher Adnan and Brothers",
                "mandi_ur": "نئی سبزی منڈی", "mandi_en": "New Sabzi Mandi", "address": "", "phone": "",
                "owner_username": "owner", "owner_display_name": "Owner",
                "password": OWNER_PASSWORD, "password_repeat": OWNER_PASSWORD,
                "confirmed_rules_words": ["maher", "adnan", "sabzi", "mandi", "brothers", "new"] } }),
        );
        self.ok(
            "auth_login",
            json!({ "input": { "username": "owner", "password": OWNER_PASSWORD } }),
        );
    }
}

fn fixtures_folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Replaces what changes from run to run, so the fixture can be compared.
fn steady(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map.iter_mut() {
                match key.as_str() {
                    "ref" => *inner = json!("E-00000000-0000"),
                    "at" | "checked_at" | "time" | "date" if inner.is_string() => {
                        *inner = json!("<time>");
                    }
                    "size_bytes" if inner.is_number() => *inner = json!(0),
                    // The place of the support file; a field `path` of an audit change is a
                    // field name and stays.
                    "path"
                        if inner
                            .as_str()
                            .is_some_and(|p| p.starts_with('/') || p.contains(":\\")) =>
                    {
                        *inner = json!("<chosen path>");
                    }
                    "id" if inner.is_number() => *inner = json!(0),
                    "chain_hash" => *inner = json!("<hash>"),
                    _ => steady(inner),
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(steady),
        _ => {}
    }
}

fn pretty(value: &Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap();
    text.push('\n');
    text
}

fn write_or_check(file: &str, value: &Value) {
    let path = fixtures_folder().join(file);
    let text = pretty(value);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::create_dir_all(fixtures_folder()).unwrap();
        std::fs::write(&path, text).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing fixture {file}; write it with {WRITE_VARIABLE}=1"));
    assert_eq!(
        on_disk, text,
        "fixture {file} is out of date; write it with {WRITE_VARIABLE}=1"
    );
}

/// Calls the command, then checks both files. Returns the real answer.
fn pair(h: &Harness, name: &str, command: &str, input: Option<Value>) -> Result<Value, Value> {
    let payload = input
        .as_ref()
        .map_or_else(|| json!({}), |i| json!({ "input": i }));
    if let Some(input) = &input {
        let mut shown = input.clone();
        steady(&mut shown);
        write_or_check(&format!("b5_{name}_request.json"), &shown);
    }
    let answer = h.call(command, payload);
    let mut shown = match &answer {
        Ok(value) => value.clone(),
        Err(error) => error.clone(),
    };
    steady(&mut shown);
    write_or_check(&format!("b5_{name}.json"), &shown);
    answer
}

#[test]
fn day_settings_and_number_grouping_through_the_real_commands() {
    let h = Harness::new();
    // Before sign-in every one of them is refused.
    for command in [
        "settings_day_get",
        "settings_day_set",
        "settings_set_number_grouping",
    ] {
        let input = match command {
            "settings_day_get" => json!({}),
            "settings_day_set" => {
                json!({ "input": { "closing_time": "21:00", "auto_day_close": false } })
            }
            _ => json!({ "input": { "grouping": "lakh" } }),
        };
        let error = h.call(command, input).unwrap_err();
        assert_eq!(error["errors"][0]["code"], "ERR_NOT_LOGGED_IN", "{command}");
    }
    h.sign_in();
    let start = pair(&h, "settings_day_get", "settings_day_get", None).unwrap();
    assert_eq!(
        start,
        json!({ "closing_time": "21:00", "auto_day_close": false })
    );
    let saved = pair(
        &h,
        "settings_day_set",
        "settings_day_set",
        Some(json!({ "closing_time": "22:30", "auto_day_close": true })),
    )
    .unwrap();
    assert_eq!(
        saved,
        json!({ "closing_time": "22:30", "auto_day_close": true })
    );
    assert_eq!(
        h.ok("settings_day_get", json!({})),
        json!({ "closing_time": "22:30", "auto_day_close": true })
    );
    // The backup screen reads the same closing time.
    assert_eq!(
        h.ok("backup_settings_get", json!({}))["settings"]["closing_time"],
        "22:30"
    );
    let refused = pair(
        &h,
        "settings_day_set_refused",
        "settings_day_set",
        Some(json!({ "closing_time": "25:00", "auto_day_close": false })),
    )
    .unwrap_err();
    assert_eq!(refused["errors"][0]["code"], "ERR_OUT_OF_RANGE");
    assert_eq!(refused["errors"][0]["field"], "closing_time");
    assert_eq!(refused["errors"][0]["values"]["min"], "00:00");
    assert_eq!(refused["errors"][0]["values"]["max"], "23:59");

    let grouping = pair(
        &h,
        "settings_set_number_grouping",
        "settings_set_number_grouping",
        Some(json!({ "grouping": "lakh" })),
    )
    .unwrap();
    assert_eq!(grouping, json!("lakh"));
    assert_eq!(h.ok("settings_get", json!({}))["number_grouping"], "lakh");
}

#[test]
fn the_audit_viewer_through_the_real_commands() {
    let h = Harness::new();
    h.sign_in();
    h.ok(
        "settings_set_number_grouping",
        json!({ "input": { "grouping": "lakh" } }),
    );
    h.ok(
        "settings_day_set",
        json!({ "input": { "closing_time": "20:15", "auto_day_close": true } }),
    );
    let filters = |extra: Value| {
        let mut base = json!({ "user_id": null, "action": "", "entity": "", "entity_id": null,
            "from": "", "to": "", "text": "" });
        if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
            for (k, v) in extra {
                base.insert(k.clone(), v.clone());
            }
        }
        base
    };
    let page = pair(
        &h,
        "audit_list_settings",
        "audit_list",
        Some(
            json!({ "filters": filters(json!({ "entity": "settings" })), "offset": 0, "limit": 0 }),
        ),
    )
    .unwrap();
    assert_eq!(page["rows"][0]["changes"][0]["path"], "number_grouping");
    let first_page = pair(
        &h,
        "audit_list_first_page",
        "audit_list",
        Some(json!({ "filters": filters(json!({ "action": "edit" })), "offset": 0, "limit": 2 })),
    )
    .unwrap();
    assert_eq!(first_page["rows"].as_array().unwrap().len(), 2);
    // The grouping, the closing time and the auto close switch are three edits.
    assert!(first_page["total_count"].as_i64().unwrap() >= 3);
    let refused = pair(
        &h,
        "audit_list_refused",
        "audit_list",
        Some(json!({ "filters": filters(json!({ "user_id": 9999, "from": "abc" })), "offset": 0, "limit": 0 })),
    )
    .unwrap_err();
    let fields: Vec<&str> = refused["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, ["filters.user_id", "filters.from"]);
    let options = pair(&h, "audit_options", "audit_options", None).unwrap();
    assert_eq!(options["users"][0]["username"], "owner");
}

#[test]
fn the_health_check_through_the_real_commands() {
    let h = Harness::new();
    h.sign_in();
    // Nothing ran yet.
    assert_eq!(
        pair(&h, "health_last_none", "health_last", None).unwrap(),
        Value::Null
    );
    let report = pair(&h, "health_run", "health_run", None).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["books"]["balanced"], true);
    assert_eq!(report["books"]["difference_paisa"], 0);
    assert_eq!(report["books"]["journal_chain"]["ok"], true);
    assert_eq!(report["books"]["audit_chain"]["ok"], true);
    assert_eq!(report["data"]["state"], "ok");
    assert_eq!(report["backup"]["cloud_level"], "off");
    // The latest result is kept: the same report comes back without a new run.
    let last = pair(&h, "health_last", "health_last", None).unwrap();
    assert_eq!(last, report);
    // No balance of any account is in it.
    let text = report.to_string();
    assert!(!text.contains("balance_paisa"), "{text}");
    // Before sign-in the commands are refused.
    let other = Harness::new();
    for command in ["health_run", "health_last"] {
        let error = other.call(command, json!({})).unwrap_err();
        assert_eq!(error["errors"][0]["code"], "ERR_NOT_LOGGED_IN", "{command}");
    }
}

#[test]
fn the_support_file_has_the_facts_and_never_a_secret() {
    let h = Harness::new();
    h.sign_in();
    let target = h.root.path().join("support-for-umar.txt");
    let request = json!({ "path": target.display().to_string() });
    let saved = h.ok("support_info_save", json!({ "input": request }));
    let mut shown = saved.clone();
    steady(&mut shown);
    write_or_check("b5_support_info_save.json", &shown);
    write_or_check(
        "b5_support_info_save_request.json",
        &json!({ "path": "<chosen path>" }),
    );
    assert_eq!(saved["path"], target.display().to_string());
    let text = std::fs::read_to_string(&target).unwrap();
    assert_eq!(saved["size_bytes"], u64::try_from(text.len()).unwrap());
    for part in [
        "[app]",
        "[system]",
        "[data]",
        "[health]",
        "[backup]",
        "[settings]",
        "[log]",
    ] {
        assert!(text.contains(part), "{part} in the file");
    }
    assert!(text.contains("version: "));
    assert!(text.contains("accounts: "));
    // No password, key, recovery code or balance.
    let lower = text.to_lowercase();
    assert!(
        !lower.contains(OWNER_PASSWORD),
        "the password is in the file"
    );
    assert!(!lower.contains("argon2"), "a password hash is in the file");
    assert!(!lower.contains("balance_paisa"));
    // A path that is not a full path to a text file in a folder that exists is refused.
    for bad in [
        "relative.txt",
        &h.root.path().join("a.exe").display().to_string(),
        &h.root
            .path()
            .join("missing")
            .join("a.txt")
            .display()
            .to_string(),
        "",
    ] {
        let error = h
            .call("support_info_save", json!({ "input": { "path": bad } }))
            .unwrap_err();
        assert_eq!(error["errors"][0]["code"], "ERR_OUT_OF_RANGE", "{bad}");
        assert_eq!(error["errors"][0]["field"], "path", "{bad}");
    }
    let refused = h
        .call(
            "support_info_save",
            json!({ "input": { "path": "relative.txt" } }),
        )
        .unwrap_err();
    let mut shown = refused;
    steady(&mut shown);
    write_or_check("b5_support_info_save_refused.json", &shown);
    write_or_check(
        "b5_support_info_save_refused_request.json",
        &json!({ "path": "relative.txt" }),
    );
}
