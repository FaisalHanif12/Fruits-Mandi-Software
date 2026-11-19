//! The OpenRouter key through the real commands (D6 Section 6, attack 9; D6-A7; NFR-SEC-15,
//! NFR-OBS-2). Owner: `robustness`.
//!
//! `robustness_assistant.rs` in the assistant crate proves the key is in no row, no file of the
//! data folder and no reply of the assistant's own service. This file adds the three places only
//! the app crate can reach: the replies of the real Tauri commands, the log, and the files the
//! app itself makes from its data, the support information file and a backup copy.
//!
//! The Owner saves a key, replaces it, fails to save one, reads the status and the log page,
//! changes the settings, sets up the backup key, backs up, makes the support file, and removes the
//! key. After each step nothing the app showed, logged or wrote holds the key. The key is a
//! placeholder that only looks like one; no test here reaches the network.
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
use mandi_services::db::rusqlite::Connection;
use mandi_services::testing::TestEnv;
use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tempfile::TempDir;

const OWNER_PASSWORD: &str = "secret1";
/// Looks like a key, is not one: after `sk-or-v1-` come letters that are not hex digits.
const KEY: &str = "sk-or-v1-robustness-placeholder-0123456789abcdef0123456789abcdef";
const NEW_KEY: &str = "sk-or-v1-robustness-replacement-fedcba9876543210fedcba9876543210";

/// The real log subscriber of the app, writing into memory (set once for this test file).
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
    /// The app data folder: the database, its log files and the `support` folder.
    data: TempDir,
    /// Where the backup copies go (the local folder is `backups/local`).
    backups: TempDir,
    _env: TestEnv,
    /// Every reply and every error the commands gave, as text.
    seen: std::cell::RefCell<Vec<String>>,
}

impl Harness {
    fn new() -> Self {
        log();
        let data = tempfile::tempdir().unwrap();
        let backups = tempfile::tempdir().unwrap();
        let env = TestEnv::new(data.path()).unwrap();
        let service = Arc::new(BackupService::new(
            Arc::clone(env.ctx.db()),
            BackupConfig {
                backups_dir: backups.path().join("backups"),
                app_version: "0.0.0-test".to_owned(),
            },
            BackupParts {
                secrets: Arc::new(MemorySecretStore::default()),
                cloud: Arc::new(FixedCloud(Arc::new(LocalFolderCloud::new(
                    backups.path().join("cloud"),
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
            data,
            backups,
            _env: env,
            seen: std::cell::RefCell::default(),
        }
    }

    /// Calls a command like the main screen and keeps what came back, good or bad.
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
        let answer = get_ipc_response(&self.window, request)
            .map(|body| body.deserialize::<Value>().unwrap());
        let text = match &answer {
            Ok(value) | Err(value) => value.to_string(),
        };
        self.seen.borrow_mut().push(format!("{command}: {text}"));
        answer
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

    fn conn(&self) -> Connection {
        Connection::open(self.data.path().join("data").join("mandi.db")).unwrap()
    }

    /// The text of the whole audit log.
    fn audit_text(&self) -> String {
        self.conn()
            .query_row(
                "SELECT COALESCE(group_concat(COALESCE(action, '') || COALESCE(entity, '') || \
                 COALESCE(old_json, '') || COALESCE(new_json, '') || COALESCE(reason, ''), ' '), '') \
                 FROM audit_log",
                [],
                |r| r.get(0),
            )
            .unwrap()
    }

    /// Fails if any of the keys is in a reply, the log, the audit trail or a file of either folder.
    fn assert_no_key(&self, keys: &[&str], when: &str) {
        let mut texts: Vec<(String, String)> = self
            .seen
            .borrow()
            .iter()
            .map(|t| ("a command reply".to_owned(), t.clone()))
            .collect();
        texts.push(("the log".to_owned(), log().text()));
        texts.push(("the audit log".to_owned(), self.audit_text()));
        for (place, text) in texts {
            for key in keys {
                assert!(!text.contains(key), "{when}: the key is in {place}");
                assert!(
                    !text.contains(&key[..key.len() - 4]),
                    "{when}: most of the key is in {place}"
                );
            }
        }
        let mut files = Vec::new();
        files_under(self.data.path(), &mut files);
        files_under(self.backups.path(), &mut files);
        assert!(!files.is_empty());
        for file in files {
            let bytes = std::fs::read(&file).unwrap_or_default();
            for key in keys {
                assert!(
                    !bytes.windows(key.len()).any(|w| w == key.as_bytes()),
                    "{when}: the key is in {}",
                    file.display()
                );
            }
        }
    }
}

fn files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files_under(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn key_input(key: &str, password: &str) -> Value {
    json!({ "input": { "key": key, "password": password } })
}

#[test]
fn the_key_is_in_no_reply_log_audit_row_support_file_or_backup() {
    let h = Harness::new();
    h.sign_in();

    // Saved, then replaced. The status shows its last 4 characters at most.
    let saved = h.ok("assistant_key_save", key_input(KEY, OWNER_PASSWORD));
    assert_eq!(saved["key_saved"], json!(true), "{saved}");
    assert_eq!(saved["key_last4"], json!(&KEY[KEY.len() - 4..]), "{saved}");
    h.assert_no_key(&[KEY], "after the key is saved");

    // Refusals repeat nothing of what was pasted: a wrong password, a key with a space, a key
    // that is far too long.
    assert_eq!(
        h.code("assistant_key_save", key_input(NEW_KEY, "not-the-password")),
        "ERR_WRONG_PASSWORD"
    );
    assert_eq!(
        h.code(
            "assistant_key_save",
            key_input(&format!("{NEW_KEY} spaced"), OWNER_PASSWORD)
        ),
        "ERR_FORMAT_INVALID"
    );
    assert_eq!(
        h.code(
            "assistant_key_save",
            key_input(&format!("{NEW_KEY}{}", "x".repeat(300)), OWNER_PASSWORD)
        ),
        "ERR_FORMAT_INVALID"
    );
    h.assert_no_key(&[KEY, NEW_KEY], "after the refusals");

    // The settings, the status, the log page, the connection test needs the network and is left
    // to the assistant crate's own attack 9.
    h.ok(
        "assistant_settings_set",
        json!({ "input": { "enabled": true, "model": "openai/gpt-6-luna" } }),
    );
    h.ok("assistant_status", json!({}));
    h.ok(
        "assistant_log_list",
        json!({ "input": { "offset": 0, "limit": 100 } }),
    );
    let replaced = h.ok("assistant_key_save", key_input(NEW_KEY, OWNER_PASSWORD));
    assert_eq!(
        replaced["key_last4"],
        json!(&NEW_KEY[NEW_KEY.len() - 4..]),
        "{replaced}"
    );
    h.assert_no_key(&[KEY, NEW_KEY], "after the key is replaced");

    // The backup key, then a backup copy to the local folder. The local copy is a plain copy of
    // the database, so it can only hold the key if the database does.
    let created = h.ok("backup_key_setup", json!({}));
    let printed = created["key"].as_str().unwrap().to_owned();
    h.ok(
        "backup_key_confirm",
        json!({ "input": { "key": printed.to_lowercase() } }),
    );
    let report = h.ok(
        "backup_now",
        json!({ "input": { "local": true, "usb": false, "cloud": false } }),
    );
    assert_eq!(report["results"][0]["ok"], json!(true), "{report}");
    let mut copies = Vec::new();
    files_under(&h.backups.path().join("backups"), &mut copies);
    assert!(!copies.is_empty(), "the backup made no file");
    h.assert_no_key(&[KEY, NEW_KEY], "after a backup");

    // The support information file, in the app data folder, and the replies that describe the app.
    let support = h.ok("support_info_save", json!({ "input": { "path": null } }));
    let support_path = PathBuf::from(support["path"].as_str().unwrap());
    let support_text = std::fs::read_to_string(&support_path).unwrap();
    assert!(!support_text.is_empty());
    for key in [KEY, NEW_KEY] {
        assert!(
            !support_text.contains(key),
            "the key is in the support file"
        );
    }
    h.ok("app_info", json!({}));
    h.ok("app_data_status", json!({}));
    h.ok("auth_status", json!({}));
    h.assert_no_key(&[KEY, NEW_KEY], "after the support file");

    // Removed: the status says so, the audit trail says what happened, and still no key anywhere.
    let removed = h.ok(
        "assistant_key_remove",
        json!({ "input": { "password": OWNER_PASSWORD } }),
    );
    assert_eq!(removed["key_saved"], json!(false), "{removed}");
    assert_eq!(removed["key_last4"], Value::Null, "{removed}");
    let audit = h.audit_text();
    for action in ["assistant_key_replace", "assistant_key_remove"] {
        assert!(audit.contains(action), "no audit row for {action}");
    }
    h.assert_no_key(&[KEY, NEW_KEY], "after the key is removed");
}

#[test]
fn a_key_command_is_refused_signed_out_and_writes_nothing() {
    let h = Harness::new();
    h.ok(
        "setup_first_run",
        json!({ "input": {
            "firm_name_ur": "ماہر عدنان اینڈ برادرز", "firm_name_en": "Maher Adnan and Brothers",
            "mandi_ur": "نئی سبزی منڈی", "mandi_en": "New Sabzi Mandi", "address": "", "phone": "",
            "owner_username": "owner", "owner_display_name": "Owner",
            "password": OWNER_PASSWORD, "password_repeat": OWNER_PASSWORD,
            "confirmed_rules_words": ["maher", "adnan", "sabzi", "mandi", "brothers", "new"] } }),
    );
    // Set up but not signed in: the key cannot be saved, removed or read about.
    for (command, payload) in [
        ("assistant_key_save", key_input(KEY, OWNER_PASSWORD)),
        (
            "assistant_key_remove",
            json!({ "input": { "password": OWNER_PASSWORD } }),
        ),
        ("assistant_status", json!({})),
    ] {
        assert_eq!(h.code(command, payload), "ERR_NOT_LOGGED_IN", "{command}");
    }
    assert!(!h.audit_text().contains("assistant_key"));
    h.assert_no_key(&[KEY], "signed out");
}
