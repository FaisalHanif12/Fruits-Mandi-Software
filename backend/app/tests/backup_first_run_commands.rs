//! Restore on a new computer (plan 56, PRD FR-20.11) through Tauri's real IPC and the real
//! capability file: the three commands run before sign-in, their JSON shapes, their errors, the
//! progress event, the refusal once an Owner exists, and the request/answer fixture pairs the
//! frontend copies (`backend/crates/backup/tests/fixtures`).
//!
//! The old shop makes a copy on a USB folder. The new computer is a fresh data file with no Owner.
//! After the restore the Owner signs in with the old password.
//!
//! Volatile values are replaced before a fixture is compared (the reference number, sizes). After
//! a deliberate change write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-app --test backup_first_run_commands`, look at
//! the diff, and tell `frontend3`.
#![cfg(test)]

mod common;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use mandi_app_lib::backup_runtime::BackupRuntime;
use mandi_app_lib::state::AppState;
use mandi_backup::api::BackupSettingsInput;
use mandi_backup::cloud::{FixedCloud, LocalFolderCloud};
use mandi_backup::daily_pack::PendingPack;
use mandi_backup::io::NoHook;
use mandi_backup::run::Scope;
use mandi_backup::secrets::{MemorySecretStore, SecretName, SecretStore};
use mandi_backup::service::{BackupConfig, BackupParts, BackupService};
use mandi_backup::settings::BackupSettings;
use mandi_services::auth::{self, FirstRunRaw};
use mandi_services::db::disk::FixedDiskProbe;
use mandi_services::settings::Firm;
use mandi_services::testing::TestEnv;
use mandi_services::{Context, Secret};
use serde_json::{Value, json};
use tauri::Listener;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tempfile::TempDir;

const OWNER_PASSWORD: &str = "secret1";
const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";
const KEY_STAND_IN: &str = "XXXX-XXXX-XXXX-XXXX-XXXX-XXXX";

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

/// The old shop: an Owner, and a copy on a USB folder locked with a printed key.
struct OldShop {
    _env: TestEnv,
    _dir: TempDir,
    usb: PathBuf,
    printed_key: String,
}

fn old_shop(root: &Path) -> OldShop {
    let dir = tempfile::tempdir().unwrap();
    let env = TestEnv::new(dir.path()).unwrap();
    auth::first_run(
        &env.ctx,
        &FirstRunRaw {
            firm: Firm {
                name_ur: "ماہر عدنان اینڈ برادرز".to_owned(),
                name_en: "Maher Adnan and Brothers".to_owned(),
                mandi_ur: "نئی سبزی منڈی کوٹ رادھا کشن".to_owned(),
                mandi_en: "New Sabzi Mandi Kot Radha Kishan".to_owned(),
                address: String::new(),
                phone: String::new(),
            },
            owner_username: "Owner".to_owned(),
            owner_display_name: "Maher Adnan".to_owned(),
            password: Secret::new(OWNER_PASSWORD),
            password_repeat: Secret::new(OWNER_PASSWORD),
        },
    )
    .unwrap();
    auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap();
    let owner = auth::require(&env.ctx).unwrap();
    let usb = root.join("usb").join("MandiBackup");
    std::fs::create_dir_all(root.join("usb")).unwrap();
    let service = BackupService::new(
        Arc::clone(env.ctx.db()),
        BackupConfig {
            backups_dir: root.join("old-backups"),
            app_version: "0.0.0-test".to_owned(),
        },
        BackupParts {
            secrets: Arc::new(MemorySecretStore::default()),
            cloud: Arc::new(FixedCloud(Arc::new(LocalFolderCloud::new(
                root.join("old-cloud"),
            )))),
            disk: Arc::new(FixedDiskProbe(10 * 1024 * 1024 * 1024)),
            hook: Arc::new(NoHook),
            pack: Arc::new(PendingPack),
        },
    );
    service
        .settings_set(
            Some(owner.user_id),
            &BackupSettingsInput {
                settings: BackupSettings {
                    usb_enabled: true,
                    usb_folder: usb.display().to_string(),
                    ..BackupSettings::default()
                },
                cloud_access_key_id: None,
                cloud_secret_access_key: None,
            },
        )
        .unwrap();
    let printed_key = service.key_setup(Some(owner.user_id)).unwrap().key;
    let report = service
        .run(Scope {
            local: true,
            usb: true,
            cloud: false,
        })
        .unwrap();
    assert!(report.results.iter().all(|r| r.ok), "{report:?}");
    OldShop {
        _env: env,
        _dir: dir,
        usb,
        printed_key,
    }
}

struct Harness {
    app: tauri::App<MockRuntime>,
    window: tauri::WebviewWindow<MockRuntime>,
    ctx: Arc<Context>,
    secrets: Arc<MemorySecretStore>,
    cloud: LocalFolderCloud,
    root: TempDir,
    _dir: TempDir,
}

impl Harness {
    /// A new computer: a fresh data file, no Owner, no keys.
    fn new() -> Self {
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
            app,
            window,
            ctx: env.ctx.clone(),
            secrets,
            cloud,
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

    fn code(&self, command: &str, payload: Value) -> String {
        let error = self.call(command, payload).unwrap_err();
        error["errors"][0]["code"].as_str().unwrap().to_owned()
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

/// Replaces what changes from run to run.
fn steady(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map.iter_mut() {
                match key.as_str() {
                    "ref" => *inner = json!("E-00000000-0000"),
                    "size_bytes" => *inner = json!(0),
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

/// Calls the command, then checks both fixture files. The request file shows the printed key as
/// `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX` and a folder as `E:\MandiBackup`, because the real ones change
/// from run to run.
fn pair(
    h: &Harness,
    name: &str,
    command: &str,
    input: &Value,
    shown_input: &Value,
) -> Result<Value, Value> {
    write_or_check(&format!("{name}_request.json"), shown_input);
    let answer = h.call(command, json!({ "input": input }));
    let mut shown = match &answer {
        Ok(value) => value.clone(),
        Err(error) => error.clone(),
    };
    steady(&mut shown);
    write_or_check(&format!("{name}.json"), &shown);
    answer
}

#[test]
fn the_restore_story_before_sign_in_matches_the_fixtures() {
    let h = Harness::new();
    let old = old_shop(h.root.path());
    let events: Arc<Mutex<Vec<Value>>> = Arc::default();
    {
        let sink = Arc::clone(&events);
        h.app.listen_any("backup-restore-progress", move |event| {
            let payload: Value = serde_json::from_str(event.payload()).unwrap();
            sink.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(payload);
        });
    }

    // Before anything: a fresh computer, nobody signed in.
    let status = h.ok("auth_status", json!({}));
    assert_eq!(status["first_run"], true);
    assert_eq!(h.code("backup_status", json!({})), "ERR_NOT_LOGGED_IN");

    let folder = old.usb.display().to_string();
    let source = json!({ "folder": folder });
    let shown_source = json!({ "folder": "E:\\MandiBackup" });

    // The list.
    let listed = pair(
        &h,
        "backup_first_run_list",
        "backup_first_run_list",
        &json!({ "source": source }),
        &json!({ "source": shown_source }),
    )
    .unwrap();
    assert_eq!(listed["total_count"], 1);
    let name = listed["rows"][0]["name"].as_str().unwrap().to_owned();
    assert_eq!(listed["rows"][0]["kind"], "usb");
    assert_eq!(listed["rows"][0]["encrypted"], true);

    // A wrong key: an error on the field `key`, nothing changes.
    let wrong = "ABCD-EFGH-JKLM-NPQR-STUV-WXYZ";
    let refused = pair(
        &h,
        "backup_first_run_preview_wrong_key",
        "backup_first_run_preview",
        &json!({ "source": source, "copy": name, "key": wrong }),
        &json!({ "source": shown_source, "copy": name, "key": wrong }),
    )
    .unwrap_err();
    assert_eq!(refused["errors"][0]["code"], "ERR_BACKUP_KEY_WRONG");
    assert_eq!(refused["errors"][0]["field"], "key");
    assert_eq!(h.ok("auth_status", json!({}))["first_run"], true);

    // The preview: the newest copy (no name sent), the printed key in small letters.
    let typed = old.printed_key.to_lowercase();
    let preview = pair(
        &h,
        "backup_first_run_preview",
        "backup_first_run_preview",
        &json!({ "source": source, "copy": null, "key": typed }),
        &json!({ "source": shown_source, "copy": null, "key": KEY_STAND_IN }),
    )
    .unwrap();
    assert_eq!(preview["copy"], json!({ "kind": "usb", "name": name }));
    assert_eq!(preview["needs_upgrade"], false);
    let checks: Vec<&str> = preview["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        checks,
        [
            "opens",
            "checksum",
            "integrity",
            "counts",
            "balanced",
            "chains"
        ]
    );
    assert!(
        preview["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["ok"] == true)
    );
    assert!(preview["totals"]["accounts"].as_i64().unwrap() > 0);

    // The restore.
    events
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clear();
    let done = pair(
        &h,
        "backup_first_run_restore",
        "backup_first_run_restore",
        &json!({ "source": source, "copy": name, "key": typed }),
        &json!({ "source": shown_source, "copy": name, "key": KEY_STAND_IN }),
    )
    .unwrap();
    assert_eq!(done["upgraded_from"], Value::Null);
    assert_eq!(done["assistant_was_on"], false);
    assert_eq!(
        done["cloud_needs_keys"], false,
        "the cloud is off in the copy"
    );
    // The progress event came as {stage, percent}, ending with search_index and done.
    let seen = events
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    let stages: Vec<&str> = seen.iter().map(|e| e["stage"].as_str().unwrap()).collect();
    for stage in [
        "downloading",
        "decrypting",
        "checking",
        "keys",
        "putting_in_place",
    ] {
        assert!(stages.contains(&stage), "{stage} in {stages:?}");
    }
    assert_eq!(stages.last(), Some(&"done"));
    assert!(stages.contains(&"search_index"));
    assert!(
        seen.iter()
            .all(|e| e["percent"].is_null() || e["percent"].is_u64())
    );

    // The Owner is not signed in; the old password works; the first-run commands are closed.
    let status = h.ok("auth_status", json!({}));
    assert_eq!(status["first_run"], false);
    assert!(status["session"].is_null());
    h.ok(
        "auth_login",
        json!({ "input": { "username": "owner", "password": OWNER_PASSWORD } }),
    );
    assert_eq!(h.ok("backup_status", json!({}))["key_set"], true);
    for (command, input) in [
        ("backup_first_run_list", json!({ "source": source })),
        (
            "backup_first_run_preview",
            json!({ "source": source, "copy": null, "key": typed }),
        ),
        (
            "backup_first_run_restore",
            json!({ "source": source, "copy": name, "key": typed }),
        ),
    ] {
        assert_eq!(
            h.code(command, json!({ "input": input })),
            "ERR_ALREADY_SET_UP",
            "{command}"
        );
    }
    let closed = h
        .call(
            "backup_first_run_list",
            json!({ "input": { "source": source } }),
        )
        .unwrap_err();
    let mut shown = closed;
    steady(&mut shown);
    write_or_check("backup_first_run_already_set_up.json", &shown);
    write_or_check(
        "backup_first_run_already_set_up_request.json",
        &json!({ "source": shown_source }),
    );

    // The backup key is kept, and it never reached the log.
    assert!(h.secrets.get(SecretName::BackupKey).unwrap().is_some());
    let text = log().text();
    assert!(
        !text.contains(&old.printed_key) && !text.contains(&old.printed_key.replace('-', "")),
        "the key is in the log"
    );
    assert!(!text.contains(&typed.replace('-', "")));
    let _ = &h.ctx;
}

#[test]
fn an_unreachable_cloud_and_bad_fields_name_their_fields_through_the_commands() {
    let h = Harness::new();
    let account = json!({ "endpoint": "https://new.example.invalid", "bucket": "mandi-backup",
        "region": "auto", "prefix": "mandi", "key_id": "AKIA-EXAMPLE", "secret": "example-secret" });
    let shown_account = json!({ "endpoint": "https://<account>.r2.cloudflarestorage.com", "bucket": "mandi-backup",
        "region": "auto", "prefix": "mandi", "key_id": "<access key id>", "secret": "<secret access key>" });

    // No internet: the field is the address; the reason travels as a value.
    h.cloud.set_offline(true);
    let offline = pair(
        &h,
        "backup_first_run_list_offline",
        "backup_first_run_list",
        &json!({ "source": { "cloud": account } }),
        &json!({ "source": { "cloud": shown_account } }),
    )
    .unwrap_err();
    assert_eq!(offline["errors"][0]["code"], "ERR_BACKUP_CLOUD_FAILED");
    assert_eq!(offline["errors"][0]["field"], "source.cloud.endpoint");
    assert_eq!(offline["errors"][0]["values"]["reason"], "offline");

    // An empty place is an empty list.
    h.cloud.set_offline(false);
    let empty = h.ok(
        "backup_first_run_list",
        json!({ "input": { "source": { "cloud": account } } }),
    );
    assert_eq!(empty, json!({ "rows": [], "total_count": 0 }));

    // Bad fields: all together, each on its own field.
    let bad = json!({ "endpoint": "http://nope", "bucket": "", "region": "auto", "prefix": "mandi",
        "key_id": "", "secret": "x" });
    let refused = pair(
        &h,
        "backup_first_run_list_refused",
        "backup_first_run_list",
        &json!({ "source": { "cloud": bad } }),
        &json!({ "source": { "cloud": bad } }),
    )
    .unwrap_err();
    let fields: Vec<&str> = refused["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["field"].as_str().unwrap())
        .collect();
    for field in [
        "source.cloud.endpoint",
        "source.cloud.bucket",
        "source.cloud.key_id",
    ] {
        assert!(fields.contains(&field), "{field} in {fields:?}");
    }
    // The address says why: not https here.
    let endpoint = refused["errors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["field"] == "source.cloud.endpoint")
        .unwrap();
    assert_eq!(endpoint["values"], json!({ "reason": "https" }));
    assert_eq!(
        h.code(
            "backup_first_run_list",
            json!({ "input": { "source": { "file": "relative.mbk" } } })
        ),
        "ERR_OUT_OF_RANGE"
    );

    // The secrets typed never reached the log.
    let text = log().text();
    assert!(
        !text.contains("example-secret"),
        "the cloud secret is in the log"
    );
    assert!(
        !text.contains("AKIA-EXAMPLE"),
        "the cloud key id is in the log"
    );
}
