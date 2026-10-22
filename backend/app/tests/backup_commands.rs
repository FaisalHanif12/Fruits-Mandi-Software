//! The backup commands (plan 18) through Tauri's real IPC and the real capability file: JSON
//! shapes, errors, sign-in, and the request/response fixture pairs the frontend copies
//! (`backend/crates/backup/tests/fixtures`).
//!
//! A fixture pair is `<name>_request.json` (exactly the `input` the command takes) and
//! `<name>.json` (the answer, or the error shape `ok`, `errors`, `ref`). Volatile values are
//! replaced before comparing: the reference number, the milliseconds a run took, file sizes and
//! the printed key. The test FAILS when an answer differs from its file. After a deliberate change
//! write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-app --test backup_commands`, look at the diff, and
//! tell `frontend`.
#![cfg(test)]

mod common;

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use mandi_app_lib::backup_runtime::BackupRuntime;
use mandi_app_lib::state::AppState;
use mandi_backup::cloud::{FixedCloud, LocalFolderCloud};
use mandi_backup::daily_pack::PendingPack;
use mandi_backup::io::NoHook;
use mandi_backup::secrets::{MemorySecretStore, SecretName, SecretStore};
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
    env: TestEnv,
    secrets: Arc<MemorySecretStore>,
    cloud: LocalFolderCloud,
    root: TempDir,
    _dir: TempDir,
    with_runtime: bool,
}

impl Harness {
    fn new(with_runtime: bool) -> Self {
        log();
        let dir = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let env = TestEnv::new(dir.path()).unwrap();
        let secrets = Arc::new(MemorySecretStore::default());
        let cloud = LocalFolderCloud::new(root.path().join("cloud"));
        let service = Arc::new(BackupService::new(
            Arc::clone(env.ctx.db()),
            BackupConfig {
                backups_dir: root.path().join("backups"),
                app_version: "0.0.0-test".to_owned(),
            },
            BackupParts {
                secrets: secrets.clone(),
                cloud: Arc::new(FixedCloud(Arc::new(cloud.clone()))),
                disk: Arc::new(FixedDiskProbe(10 * 1024 * 1024 * 1024)),
                hook: Arc::new(NoHook),
                pack: Arc::new(PendingPack),
            },
        ));
        let mut builder = mock_builder().manage(AppState::with_context(env.ctx.clone()));
        if with_runtime {
            builder = builder.manage(BackupRuntime::new(service));
        }
        let app = builder
            .invoke_handler(mandi_app_lib::handler())
            .build(mandi_app_lib::context())
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        Self {
            _app: app,
            window,
            env,
            secrets,
            cloud,
            root,
            _dir: dir,
            with_runtime,
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

    fn code(&self, command: &str, payload: Value) -> String {
        let error = self.call(command, payload).unwrap_err();
        error["errors"][0]["code"].as_str().unwrap().to_owned()
    }

    fn sign_in(&self) {
        let _ = self.with_runtime;
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
        .join("..")
        .join("crates")
        .join("backup")
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
                    "snapshot_ms" | "check_ms" | "size_bytes" => *inner = json!(0),
                    "local_dir_effective" => *inner = json!("<app data>/backups/local"),
                    "key" if inner.is_string() => *inner = json!("XXXX-XXXX-XXXX-XXXX-XXXX-XXXX"),
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

/// Calls the command with the request, then checks both files. Returns the real answer.
fn pair(h: &Harness, name: &str, command: &str, input: Option<Value>) -> Result<Value, Value> {
    let payload = input
        .as_ref()
        .map_or_else(|| json!({}), |i| json!({ "input": i }));
    if let Some(input) = &input {
        write_or_check(&format!("{name}_request.json"), input);
    }
    let answer = h.call(command, payload);
    let mut shown = match &answer {
        Ok(value) => value.clone(),
        Err(error) => error.clone(),
    };
    steady(&mut shown);
    write_or_check(&format!("{name}.json"), &shown);
    answer
}

fn settings(usb: &Path, extra: Value) -> Value {
    let mut base = json!({ "local_dir": "", "usb_enabled": true, "usb_folder": usb.display().to_string().replace(usb.parent().unwrap().to_str().unwrap(), "<root>"),
        "cloud_enabled": true, "cloud_endpoint": "https://example.r2.cloudflarestorage.com", "cloud_bucket": "mandi-backup",
        "cloud_region": "auto", "cloud_prefix": "mandi", "closing_time": "21:00" });
    if let (Some(map), Some(more)) = (base.as_object_mut(), extra.as_object()) {
        map.extend(more.clone());
    }
    base
}

#[test]
fn the_backup_story_through_the_real_commands_matches_the_fixtures() {
    let h = Harness::new(true);
    h.sign_in();
    let usb_dir = h.root.path().join("usb");
    std::fs::create_dir_all(&usb_dir).unwrap();
    let usb_real = usb_dir.join("MandiBackup");

    // Settings: the defaults, a refusal with all its problems, then good ones.
    let defaults = pair(&h, "backup_settings_get", "backup_settings_get", None).unwrap();
    assert_eq!(defaults["settings"]["closing_time"], "21:00");
    assert_eq!(defaults["cloud_secret_set"], false);
    let bad = json!({ "settings": { "local_dir": "", "usb_enabled": true, "usb_folder": "", "cloud_enabled": true,
        "cloud_endpoint": "http://nope", "cloud_bucket": "", "cloud_region": "auto", "cloud_prefix": "mandi",
        "closing_time": "9pm" }, "cloud_access_key_id": null, "cloud_secret_access_key": null });
    let refused = pair(
        &h,
        "backup_settings_set_refused",
        "backup_settings_set",
        Some(bad),
    )
    .unwrap_err();
    let fields: Vec<&str> = refused["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["field"].as_str().unwrap())
        .collect();
    for field in [
        "usb_folder",
        "cloud_endpoint",
        "cloud_bucket",
        "closing_time",
    ] {
        assert!(fields.contains(&field), "{field} in {fields:?}");
    }
    let mut good = settings(&usb_real, json!({}));
    good["usb_folder"] = json!(usb_real.display().to_string());
    let input = json!({ "settings": good, "cloud_access_key_id": "AKIA-EXAMPLE", "cloud_secret_access_key": "example-secret" });
    // The fixture request holds the shape; the folder is a real temp path, so it is stood in.
    let mut stood_in = input.clone();
    stood_in["settings"]["usb_folder"] = json!("E:\\MandiBackup");
    write_or_check("backup_settings_set_request.json", &stood_in);
    let saved = h.ok("backup_settings_set", json!({ "input": input }));
    let mut shown = saved.clone();
    shown["settings"]["usb_folder"] = json!("E:\\MandiBackup");
    shown["local_dir_effective"] = json!("<app data>/backups/local");
    write_or_check("backup_settings_set.json", &shown);
    assert_eq!(
        (
            saved["cloud_access_key_set"].clone(),
            saved["cloud_secret_set"].clone()
        ),
        (json!(true), json!(true))
    );

    // Status before any key.
    let status = pair(&h, "backup_status_new", "backup_status", None).unwrap();
    assert_eq!(
        (status["key_set"].clone(), status["cloud_level"].clone()),
        (json!(false), json!("red"))
    );

    // The key: shown once, confirmed by typing.
    let created = h.ok("backup_key_setup", json!({}));
    let printed = created["key"].as_str().unwrap().to_owned();
    let mut shown = created.clone();
    steady(&mut shown);
    write_or_check("backup_key_setup.json", &shown);
    let wrong = pair(
        &h,
        "backup_key_confirm_wrong",
        "backup_key_confirm",
        Some(json!({ "key": "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ" })),
    )
    .unwrap_err();
    assert_eq!(wrong["errors"][0]["code"], "ERR_BACKUP_KEY_WRONG");
    assert_eq!(wrong["errors"][0]["field"], "key");
    // The real key goes in the request, so its fixture holds a stand-in.
    write_or_check(
        "backup_key_confirm_request.json",
        &json!({ "key": "XXXX-XXXX-XXXX-XXXX-XXXX-XXXX" }),
    );
    let confirmed = h.ok(
        "backup_key_confirm",
        json!({ "input": { "key": printed.to_lowercase() } }),
    );
    assert_eq!(confirmed, Value::Null);
    write_or_check("backup_key_confirm.json", &confirmed);
    let again = pair(&h, "backup_key_setup_again", "backup_key_setup", None).unwrap_err();
    assert_eq!(again["errors"][0]["code"], "ERR_BACKUP_KEY_ALREADY_SET");

    // Back up now: local, USB and cloud.
    h.env.clock.advance_seconds(60);
    let report = pair(
        &h,
        "backup_now",
        "backup_now",
        Some(json!({ "local": true, "usb": true, "cloud": true })),
    )
    .unwrap();
    let results = report["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    assert!(
        results
            .iter()
            .all(|r| r["ok"] == json!(true) && r["queued"] == json!(false)),
        "{report}"
    );

    let list = pair(
        &h,
        "backup_list",
        "backup_list",
        Some(json!({ "kind": null, "offset": 0, "limit": 10 })),
    )
    .unwrap();
    assert_eq!(list["total_count"], 3);
    assert_eq!(list["cloud_reachable"], true);
    let cloud_name = list["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "cloud")
        .unwrap()["name"]
        .as_str()
        .unwrap()
        .to_owned();

    let status = pair(&h, "backup_status_after", "backup_status", None).unwrap();
    assert_eq!(
        (
            status["cloud_level"].clone(),
            status["queue_len"].clone(),
            status["key_confirmed"].clone()
        ),
        (json!("green"), json!(0), json!(true))
    );

    // Preview and test restore of the cloud copy.
    let preview_input = json!({ "backup": { "kind": "cloud", "name": cloud_name }, "key": null });
    let preview = h.ok("backup_restore_preview", json!({ "input": preview_input }));
    assert_eq!(preview["lost"]["journal_entries"], 0);
    assert_eq!(
        preview["totals"]["debit_paisa"],
        preview["totals"]["credit_paisa"]
    );
    let mut stood_in = preview_input.clone();
    stood_in["backup"]["name"] = json!("mandi-20261010T090100000Z.mbk");
    write_or_check("backup_restore_preview_request.json", &stood_in);
    let mut shown = preview.clone();
    steady(&mut shown);
    shown["backup"]["name"] = json!("mandi-20261010T090100000Z.mbk");
    write_or_check("backup_restore_preview.json", &shown);

    let test = h.ok(
        "backup_test_restore",
        json!({ "input": { "backup": null, "key": null } }),
    );
    assert_eq!(test["ok"], true, "{test}");
    assert_eq!(test["backup"]["kind"], "cloud");
    write_or_check(
        "backup_test_restore_request.json",
        &json!({ "backup": null, "key": null }),
    );
    let mut shown = test.clone();
    steady(&mut shown);
    shown["backup"]["name"] = json!("mandi-20261010T090100000Z.mbk");
    write_or_check("backup_test_restore.json", &shown);

    // A restore needs the confirmation; a wrong key changes nothing.
    let unconfirmed = json!({ "backup": { "kind": "cloud", "name": cloud_name }, "key": null, "confirmed": false });
    assert_eq!(
        h.code("backup_restore", json!({ "input": unconfirmed })),
        "ERR_BACKUP_NOT_CONFIRMED"
    );
    let mut stood_in = unconfirmed.clone();
    stood_in["backup"]["name"] = json!("mandi-20261010T090100000Z.mbk");
    write_or_check("backup_restore_unconfirmed_request.json", &stood_in);
    let mut shown = h
        .call("backup_restore", json!({ "input": unconfirmed }))
        .unwrap_err();
    steady(&mut shown);
    write_or_check("backup_restore_unconfirmed.json", &shown);

    h.secrets.delete(SecretName::BackupKey).unwrap();
    let wrong_key = json!({ "backup": { "kind": "cloud", "name": cloud_name }, "key": "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ", "confirmed": true });
    assert_eq!(
        h.code("backup_restore", json!({ "input": wrong_key })),
        "ERR_BACKUP_KEY_WRONG"
    );
    assert_eq!(
        h.code("backup_restore", json!({ "input": { "backup": { "kind": "cloud", "name": cloud_name }, "key": null, "confirmed": true } })),
        "ERR_BACKUP_KEY_REQUIRED"
    );

    // The restore with the printed key; the Owner is signed out and the sign-in works again.
    let typed = json!({ "backup": { "kind": "cloud", "name": cloud_name }, "key": printed.to_lowercase(), "confirmed": true });
    let done = h.ok("backup_restore", json!({ "input": typed }));
    let mut stood_in = typed.clone();
    stood_in["backup"]["name"] = json!("mandi-20261010T090100000Z.mbk");
    stood_in["key"] = json!("XXXX-XXXX-XXXX-XXXX-XXXX-XXXX");
    write_or_check("backup_restore_request.json", &stood_in);
    let mut shown = done.clone();
    steady(&mut shown);
    write_or_check("backup_restore.json", &shown);
    assert!(
        done["safety_copy"]
            .as_str()
            .unwrap()
            .starts_with("mandi-local-")
    );
    assert_eq!(
        h.code("backup_status", json!({})),
        "ERR_NOT_LOGGED_IN",
        "the restore signs the Owner out"
    );
    h.ok(
        "auth_login",
        json!({ "input": { "username": "owner", "password": OWNER_PASSWORD } }),
    );
    // The key was typed on a computer without one, so it is kept now.
    let status = h.ok("backup_status", json!({}));
    assert_eq!(
        (status["key_set"].clone(), status["key_confirmed"].clone()),
        (json!(true), json!(true))
    );

    // The key never reached the log.
    let text = log().text();
    assert!(
        !text.contains(&printed) && !text.contains(&printed.replace('-', "")),
        "the key is in the log"
    );
    assert!(!text.contains("example-secret"));
    let _ = &h.cloud;
}

#[test]
fn an_unreachable_cloud_is_a_queue_not_an_error_and_the_codes_are_the_backup_codes() {
    let h = Harness::new(true);
    h.sign_in();
    let usb = h.root.path().join("usb").join("MandiBackup");
    h.ok(
        "backup_settings_set",
        json!({ "input": { "settings": settings(&usb, json!({ "usb_enabled": false, "usb_folder": "" })),
            "cloud_access_key_id": "k", "cloud_secret_access_key": "s" } }),
    );
    h.ok("backup_key_setup", json!({}));
    h.cloud.set_offline(true);
    let report = h.ok(
        "backup_now",
        json!({ "input": { "local": true, "usb": false, "cloud": true } }),
    );
    let cloud = report["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "cloud")
        .unwrap()
        .clone();
    assert_eq!(
        (cloud["ok"].clone(), cloud["queued"].clone()),
        (json!(false), json!(true))
    );
    assert_eq!(cloud["code"], "ERR_BACKUP_CLOUD_FAILED");
    let status = h.ok("backup_status", json!({}));
    assert_eq!(status["queue_len"], 1);
    write_or_check(
        "backup_now_queued_request.json",
        &json!({ "local": true, "usb": false, "cloud": true }),
    );
    let mut shown = report.clone();
    steady(&mut shown);
    write_or_check("backup_now_queued.json", &shown);
    h.cloud.set_offline(false);
    let report = h.ok(
        "backup_now",
        json!({ "input": { "local": false, "usb": false, "cloud": true } }),
    );
    assert!(
        report["results"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["ok"] == json!(true))
    );
    assert_eq!(
        h.ok("backup_status", json!({}))["queue_len"],
        0,
        "the queue caught up"
    );
}

#[test]
fn without_the_backup_service_every_command_says_so_with_a_code_and_never_internal() {
    let h = Harness::new(false);
    h.sign_in();
    for (command, payload) in [
        ("backup_status", json!({})),
        ("backup_settings_get", json!({})),
        ("backup_key_setup", json!({})),
        (
            "backup_list",
            json!({ "input": { "kind": null, "offset": 0, "limit": 10 } }),
        ),
    ] {
        assert_eq!(
            h.code(command, payload),
            "ERR_BACKUP_NOT_STARTED",
            "{command}"
        );
    }
}

#[test]
fn every_backup_command_needs_the_owner_signed_in_and_not_locked() {
    let h = Harness::new(true);
    let commands = [
        ("backup_status", json!({})),
        (
            "backup_list",
            json!({ "input": { "kind": null, "offset": 0, "limit": 10 } }),
        ),
        (
            "backup_now",
            json!({ "input": { "local": true, "usb": false, "cloud": false } }),
        ),
        ("backup_settings_get", json!({})),
        ("backup_key_setup", json!({})),
        (
            "backup_key_confirm",
            json!({ "input": { "key": "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ" } }),
        ),
        (
            "backup_test_restore",
            json!({ "input": { "backup": null, "key": null } }),
        ),
    ];
    for (command, payload) in &commands {
        assert_eq!(
            h.code(command, payload.clone()),
            "ERR_NOT_LOGGED_IN",
            "{command}"
        );
    }
    h.sign_in();
    h.ok("auth_lock", json!({}));
    for (command, payload) in &commands {
        assert_eq!(
            h.code(command, payload.clone()),
            "ERR_SESSION_LOCKED",
            "{command}"
        );
    }
    // Nothing was written by a refused call.
    let conn = mandi_services::db::rusqlite::Connection::open(h.env.ctx.db().path()).unwrap();
    let rows: i64 = conn
        .query_row("SELECT COUNT(*) FROM backup_log", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 0);
}

#[test]
fn inputs_hide_their_keys_in_debug_text() {
    use mandi_backup::api::{
        BackupSettingsInput, KeyConfirmInput, RestoreInput, RestorePreviewInput, TestRestoreInput,
    };
    let key = "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ";
    let restore: RestoreInput = serde_json::from_value(json!({ "backup": { "kind": "cloud", "name": "mandi-x.mbk" }, "key": key, "confirmed": true })).unwrap();
    let preview: RestorePreviewInput = serde_json::from_value(
        json!({ "backup": { "kind": "cloud", "name": "mandi-x.mbk" }, "key": key }),
    )
    .unwrap();
    let test: TestRestoreInput =
        serde_json::from_value(json!({ "backup": null, "key": key })).unwrap();
    let confirm: KeyConfirmInput = serde_json::from_value(json!({ "key": key })).unwrap();
    let settings: BackupSettingsInput = serde_json::from_value(json!({ "settings": settings(Path::new("/x/y"), json!({})), "cloud_access_key_id": "AKIA-SECRET", "cloud_secret_access_key": "SECRET-VALUE" })).unwrap();
    let text = format!("{restore:?} {preview:?} {test:?} {confirm:?} {settings:?}");
    for secret in [key, "AKIA-SECRET", "SECRET-VALUE"] {
        assert!(!text.contains(secret), "{secret} in {text}");
    }
}
