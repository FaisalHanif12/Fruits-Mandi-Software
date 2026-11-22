//! `backup_key_reveal` (PRD FR-20.10, v1.54, NFR-SEC-7) through Tauri's real IPC: the Owner lost the
//! printed paper and sees the backup key again after his password and his sign.
//!
//! What is proved here:
//! - a right password and a right sign give the key, and one audit row says it was viewed;
//! - a wrong password, a wrong sign, and a locked name each give no key and write one audit row of
//!   their own;
//! - a missing password or sign is refused together and writes nothing;
//! - no sign-in, or no key on the computer, give no key;
//! - the key, the password and the typed sign are in no audit row, no log line and no refusal.
//!
//! Fixture pairs for the frontend are written to `backend/crates/backup/tests/fixtures` as in
//! `backup_commands.rs` (`MANDI_WRITE_FIXTURES=1` writes them again).
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
const WRONG_PASSWORD: &str = "not-the-password-77";
const WRONG_SIGN: &str = "munshi-sign-77";
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
    _root: TempDir,
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
            env,
            _root: root,
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

    fn set_up(&self) {
        self.ok(
            "setup_first_run",
            json!({ "input": {
                "firm_name_ur": "ماہر عدنان اینڈ برادرز", "firm_name_en": "Maher Adnan and Brothers",
                "mandi_ur": "نئی سبزی منڈی", "mandi_en": "New Sabzi Mandi", "address": "", "phone": "",
                "owner_username": "owner", "owner_display_name": "Owner",
                "password": OWNER_PASSWORD, "password_repeat": OWNER_PASSWORD,
                "confirmed_rules_words": ["maher", "adnan", "sabzi", "mandi", "brothers", "new"] } }),
        );
    }

    fn sign_in(&self) {
        self.ok(
            "auth_login",
            json!({ "input": { "username": "owner", "password": OWNER_PASSWORD } }),
        );
    }

    /// A signed-in Owner, with a backup key made and printed. Returns the printed key.
    fn with_key(&self) -> String {
        self.set_up();
        self.sign_in();
        let created = self.ok("backup_key_setup", json!({}));
        created["key"].as_str().unwrap().to_owned()
    }

    fn reveal(&self, password: &str, sign: &str) -> Result<Value, Value> {
        self.call(
            "backup_key_reveal",
            json!({ "input": { "password": password, "sign": sign } }),
        )
    }

    /// The audit rows of the backup key, oldest first: `action` and `new_json`.
    fn key_audit(&self) -> Vec<(String, String)> {
        self.env
            .ctx
            .db()
            .read(|conn| {
                let mut statement = conn.prepare(
                    "SELECT action, COALESCE(new_json, '') FROM audit_log
                     WHERE entity = 'backup_key' AND action LIKE 'reveal%' ORDER BY id",
                )?;
                let rows = statement
                    .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok::<_, mandi_services::ServiceError>(rows)
            })
            .unwrap()
    }

    /// The whole audit log as one text.
    fn audit_text(&self) -> String {
        self.env
            .ctx
            .db()
            .read(|conn| {
                conn.query_row(
                    "SELECT COALESCE(group_concat(COALESCE(action, '') || COALESCE(entity, '') ||
                     COALESCE(old_json, '') || COALESCE(new_json, '') || COALESCE(reason, ''), ' '), '')
                     FROM audit_log",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .map_err(mandi_services::ServiceError::from)
            })
            .unwrap()
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
                    "key" if inner.is_string() => *inner = json!("XXXX-XXXX-XXXX-XXXX-XXXX-XXXX"),
                    _ => steady(inner),
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(steady),
        _ => {}
    }
}

fn write_or_check(file: &str, value: &Value) {
    let path = fixtures_folder().join(file);
    let mut text = serde_json::to_string_pretty(value).unwrap();
    text.push('\n');
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

/// The request of a fixture pair holds stand-ins for the password and the sign.
fn request_fixture(name: &str) {
    write_or_check(
        &format!("{name}_request.json"),
        &json!({ "password": "<the Owner's password>", "sign": "<the Owner's sign>" }),
    );
}

fn first_error(error: &Value) -> (String, Option<String>) {
    (
        error["errors"][0]["code"].as_str().unwrap().to_owned(),
        error["errors"][0]["field"].as_str().map(str::to_owned),
    )
}

#[test]
fn a_right_password_and_sign_give_the_key_and_one_audit_row_without_it() {
    let h = Harness::new();
    let printed = h.with_key();
    let shown = h.ok(
        "backup_key_reveal",
        json!({ "input": { "password": OWNER_PASSWORD, "sign": "adnan" } }),
    );
    assert_eq!(shown["key"], json!(printed));
    let mut fixture = shown.clone();
    steady(&mut fixture);
    request_fixture("backup_key_reveal");
    write_or_check("backup_key_reveal.json", &fixture);
    // One row, and it says "viewed", not what.
    assert_eq!(
        h.key_audit(),
        [("reveal".to_owned(), r#"{"backup_key":"viewed"}"#.to_owned())]
    );
    // The sign is accepted in any case with spaces at the ends, and in Urdu letters (FR-21.4).
    for sign in ["  ADNAN ", "عدنان"] {
        let again = h.ok(
            "backup_key_reveal",
            json!({ "input": { "password": OWNER_PASSWORD, "sign": sign } }),
        );
        assert_eq!(again["key"], json!(printed), "{sign}");
    }
    assert_eq!(h.key_audit().len(), 3, "one row for each viewing");
    // The key is in no audit row and no log line (the reply above is the one place it goes).
    let key = printed.as_str();
    let raw = key.replace('-', "");
    for (place, text) in [("audit", h.audit_text()), ("log", log().text())] {
        assert!(!text.contains(key), "the key is in the {place}");
        assert!(!text.contains(&raw), "the raw key is in the {place}");
        assert!(
            !text.contains(&key[..key.len() - 4]),
            "most of the key is in the {place}"
        );
    }
}

#[test]
fn a_wrong_password_gives_no_key_and_one_audit_row_of_its_own() {
    let h = Harness::new();
    let printed = h.with_key();
    let refused = h.reveal(WRONG_PASSWORD, "adnan").unwrap_err();
    assert_eq!(
        first_error(&refused),
        ("ERR_WRONG_PASSWORD".to_owned(), Some("password".to_owned()))
    );
    request_fixture("backup_key_reveal_wrong_password");
    let mut fixture = refused.clone();
    steady(&mut fixture);
    write_or_check("backup_key_reveal_wrong_password.json", &fixture);
    assert!(
        !refused.to_string().contains(&printed),
        "no key in a refusal"
    );
    assert!(refused.get("key").is_none());
    assert_eq!(
        h.key_audit(),
        [(
            "reveal_refused".to_owned(),
            r#"{"refused":"password"}"#.to_owned()
        )]
    );
}

#[test]
fn a_wrong_sign_gives_no_key_and_one_audit_row_of_its_own() {
    let h = Harness::new();
    let printed = h.with_key();
    let refused = h.reveal(OWNER_PASSWORD, WRONG_SIGN).unwrap_err();
    assert_eq!(
        first_error(&refused),
        ("ERR_SIGN_WRONG".to_owned(), Some("sign".to_owned()))
    );
    request_fixture("backup_key_reveal_wrong_sign");
    let mut fixture = refused.clone();
    steady(&mut fixture);
    write_or_check("backup_key_reveal_wrong_sign.json", &fixture);
    assert!(!refused.to_string().contains(&printed));
    assert_eq!(
        h.key_audit(),
        [(
            "reveal_refused".to_owned(),
            r#"{"refused":"sign"}"#.to_owned()
        )]
    );
    // The typed sign and the typed password are in no audit row and no log line.
    for (place, text) in [("audit", h.audit_text()), ("log", log().text())] {
        assert!(
            !text.contains(WRONG_SIGN),
            "the typed sign is in the {place}"
        );
        assert!(
            !text.contains(OWNER_PASSWORD),
            "the password is in the {place}"
        );
    }
}

#[test]
fn a_missing_password_or_sign_is_refused_together_and_writes_nothing() {
    let h = Harness::new();
    h.with_key();
    let refused = h.reveal("", "  ").unwrap_err();
    let items = refused["errors"].as_array().unwrap();
    let got: Vec<(String, String)> = items
        .iter()
        .map(|i| {
            (
                i["code"].as_str().unwrap().to_owned(),
                i["field"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            ("ERR_REQUIRED".to_owned(), "password".to_owned()),
            ("ERR_SIGN_REQUIRED".to_owned(), "sign".to_owned())
        ]
    );
    assert_eq!(h.key_audit(), []);
}

#[test]
fn a_locked_name_gives_no_key_even_with_the_right_password_and_is_audited() {
    // PRD FR-8.2: the 10th wrong try starts a wait of 1 minute. The reveal counts as a try of his name.
    let h = Harness::new();
    let printed = h.with_key();
    for _ in 0..9 {
        let refused = h.reveal(WRONG_PASSWORD, "adnan").unwrap_err();
        assert_eq!(first_error(&refused).0, "ERR_WRONG_PASSWORD");
    }
    let tenth = h.reveal(WRONG_PASSWORD, "adnan").unwrap_err();
    assert_eq!(first_error(&tenth).0, "ERR_LOCKED_WAIT");
    // While the name waits, even the right password and sign give nothing.
    let locked = h.reveal(OWNER_PASSWORD, "adnan").unwrap_err();
    assert_eq!(first_error(&locked).0, "ERR_LOCKED_WAIT");
    assert!(!locked.to_string().contains(&printed));
    let rows = h.key_audit();
    let refused = |why: &str| {
        (
            "reveal_refused".to_owned(),
            format!(r#"{{"refused":"{why}"}}"#),
        )
    };
    assert_eq!(rows.len(), 11, "one row for each refused try");
    assert!(rows[..9].iter().all(|row| *row == refused("password")));
    assert!(rows[9..].iter().all(|row| *row == refused("locked")));
}

#[test]
fn no_sign_in_or_no_key_gives_no_key() {
    let h = Harness::new();
    h.set_up();
    // Not signed in: refused by the sign-in check, nothing written.
    let refused = h.reveal(OWNER_PASSWORD, "adnan").unwrap_err();
    assert_eq!(first_error(&refused).0, "ERR_NOT_LOGGED_IN");
    assert_eq!(h.key_audit(), []);
    // Signed in, but this computer has no key yet.
    h.sign_in();
    let none = h.reveal(OWNER_PASSWORD, "adnan").unwrap_err();
    assert_eq!(first_error(&none).0, "ERR_BACKUP_KEY_NOT_SET");
    assert!(
        h.key_audit().iter().all(|(action, _)| action != "reveal"),
        "nothing was viewed"
    );
}
