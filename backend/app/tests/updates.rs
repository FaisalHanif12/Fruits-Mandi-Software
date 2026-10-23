//! Updates (plan 22 early, NFR-INS-3, NFR-INS-4, NFR-SEC-11, NFR-REL-13) against a fake update
//! server on this computer, with the real updater plugin and its signature check.
//!
//! The keys in `tests/fixtures/updates` are throw-away test keys made with the Tauri signer:
//! `test.pub` is the public key the plugin is given, `bundle.bin` stands for the installer,
//! `good.sig` is its signature by the matching private key and `other.sig` its signature by a
//! different key. Nothing is ever installed: the install step is a stand-in that records that it
//! was called.
//!
//! The story: a package not signed by our key is refused and no copy of the books is taken and
//! nothing is installed; a changed package is refused the same way; a good package is checked,
//! the books are copied, and only then the installer is called; and an update never starts while
//! an entry waits unsaved, without the Owner's yes, or for a version he did not see.
#![cfg(test)]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use mandi_app_lib::commands::backup::BackupWireError;
use mandi_app_lib::state::AppState;
use mandi_app_lib::updates::{UpdateInstallInput, check, install_flow};
use mandi_backup::error::{BackupCode, BackupError};
use mandi_services::testing::TestEnv;
use serde_json::{Value, json};
use tauri::test::{MockRuntime, mock_builder};
use tempfile::TempDir;

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn updates_folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("updates")
}

fn read(name: &str) -> Vec<u8> {
    std::fs::read(updates_folder().join(name))
        .unwrap_or_else(|e| panic!("test key file {name}: {e}"))
}

fn read_text(name: &str) -> String {
    String::from_utf8(read(name)).unwrap().trim().to_owned()
}

const NOTES: &str = "## نیا کیا ہے\n\n- ایک\n- دو\n\n## What's new\n\n- One\n- Two\n";

/// One answer of the fake server.
struct Route {
    path: &'static str,
    status: u16,
    body: Vec<u8>,
}

/// A small HTTP server on this computer. It keeps the paths it was asked for.
struct Server {
    base: String,
    hits: Arc<Mutex<Vec<String>>>,
}

impl Server {
    /// Starts a server; `routes` gets the base address, so the manifest can point at the package.
    fn start(routes: impl FnOnce(&str) -> Vec<Route>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        let routes = routes(&base);
        let hits = Arc::new(Mutex::new(Vec::<String>::new()));
        let seen = Arc::clone(&hits);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut buffer = [0_u8; 8192];
                let read = stream.read(&mut buffer).unwrap_or(0);
                let request = String::from_utf8_lossy(buffer.get(..read).unwrap_or_default());
                let path = request
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("/")
                    .split('?')
                    .next()
                    .unwrap_or("/")
                    .to_owned();
                seen.lock().unwrap().push(path.clone());
                let (status, body) = routes
                    .iter()
                    .find(|r| r.path == path)
                    .map_or((404, Vec::new()), |r| (r.status, r.body.clone()));
                let reason = if status == 200 { "OK" } else { "No" };
                let head = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        Self { base, hits }
    }

    /// A server that offers version 99.0.0 with this signature and serves this package.
    fn offering(signature: String, bundle: Vec<u8>) -> Self {
        Self::start(|base| {
            let manifest = json!({
                "version": "99.0.0",
                "notes": NOTES,
                "pub_date": "2026-10-23T09:00:00Z",
                "url": format!("{base}/package.bin"),
                "signature": signature,
            });
            vec![
                Route {
                    path: "/latest.json",
                    status: 200,
                    body: manifest.to_string().into_bytes(),
                },
                Route {
                    path: "/package.bin",
                    status: 200,
                    body: bundle,
                },
            ]
        })
    }

    fn endpoint(&self) -> String {
        format!("{}/latest.json", self.base)
    }

    fn paths(&self) -> Vec<String> {
        self.hits.lock().unwrap().clone()
    }
}

/// A mock app with the real updater plugin, given our test key and this server.
struct Rig {
    app: tauri::App<MockRuntime>,
    env: TestEnv,
    _dir: TempDir,
}

fn rig(endpoint: &str) -> Rig {
    let dir = tempfile::tempdir().unwrap();
    let env = TestEnv::new(dir.path()).unwrap();
    let mut context = mandi_app_lib::context();
    context.config_mut().plugins.0.insert(
        "updater".to_owned(),
        json!({
            "pubkey": read_text("test.pub"),
            "endpoints": [endpoint],
            "dangerousInsecureTransportProtocol": true
        }),
    );
    let app = mock_builder()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::with_context(env.ctx.clone()))
        .build(context)
        .unwrap();
    Rig {
        app,
        env,
        _dir: dir,
    }
}

/// What the stand-ins were asked to do, in order.
type Calls = Arc<Mutex<Vec<String>>>;

fn confirmed(version: &str) -> UpdateInstallInput {
    UpdateInstallInput {
        expected_version: version.to_owned(),
        confirmed: true,
    }
}

/// Runs the install flow with stand-ins that only note that they were called.
fn flow(
    rig: &Rig,
    input: &UpdateInstallInput,
    backup_ok: bool,
) -> (Result<String, BackupError>, Vec<String>, Vec<u8>) {
    let calls: Calls = Arc::default();
    let installed: Arc<Mutex<Vec<u8>>> = Arc::default();
    let (c1, c2, bytes_seen) = (
        Arc::clone(&calls),
        Arc::clone(&calls),
        Arc::clone(&installed),
    );
    let result = tauri::async_runtime::block_on(install_flow(
        rig.app.handle(),
        &rig.env.ctx,
        input,
        move || {
            c1.lock().unwrap().push("backup".to_owned());
            if backup_ok {
                Ok(Some("mandi-test-copy.db".to_owned()))
            } else {
                Err(BackupError::new(
                    BackupCode::ErrUpdateBackupFailed,
                    "no good local copy",
                ))
            }
        },
        move |update, bytes| {
            c2.lock()
                .unwrap()
                .push(format!("install {}", update.version));
            *bytes_seen.lock().unwrap() = bytes;
            Ok(())
        },
    ))
    .map(|done| format!("{} {}", done.version, done.backup_file.unwrap_or_default()));
    let order = calls.lock().unwrap().clone();
    let bytes = installed.lock().unwrap().clone();
    (result, order, bytes)
}

fn code_of(result: &Result<String, BackupError>) -> BackupCode {
    result.as_ref().unwrap_err().code
}

#[test]
fn a_package_signed_by_another_key_is_refused_and_nothing_else_happens() {
    let server = Server::offering(read_text("other.sig"), read("bundle.bin"));
    let rig = rig(&server.endpoint());
    let (result, calls, _) = flow(&rig, &confirmed("99.0.0"), true);
    assert_eq!(
        code_of(&result),
        BackupCode::ErrUpdateNotSigned,
        "{result:?}"
    );
    // No copy of the books was taken and the installer was never called.
    assert!(calls.is_empty(), "{calls:?}");
    // The package was fetched to check it, and that is all.
    assert_eq!(server.paths(), ["/latest.json", "/package.bin"]);
}

#[test]
fn a_package_changed_on_the_way_is_refused_too() {
    let mut tampered = read("bundle.bin");
    tampered.push(0);
    let server = Server::offering(read_text("good.sig"), tampered);
    let rig = rig(&server.endpoint());
    let (result, calls, _) = flow(&rig, &confirmed("99.0.0"), true);
    assert_eq!(code_of(&result), BackupCode::ErrUpdateNotSigned);
    assert!(calls.is_empty(), "{calls:?}");
}

#[test]
fn a_garbage_signature_is_refused_as_not_signed() {
    let server = Server::offering("not a signature".to_owned(), read("bundle.bin"));
    let rig = rig(&server.endpoint());
    let (result, calls, _) = flow(&rig, &confirmed("99.0.0"), true);
    assert_eq!(code_of(&result), BackupCode::ErrUpdateNotSigned);
    assert!(calls.is_empty());
}

#[test]
fn a_good_package_is_checked_then_the_books_are_copied_then_it_is_installed() {
    let bundle = read("bundle.bin");
    let server = Server::offering(read_text("good.sig"), bundle.clone());
    let rig = rig(&server.endpoint());
    let (result, calls, installed) = flow(&rig, &confirmed("99.0.0"), true);
    assert_eq!(result.unwrap(), "99.0.0 mandi-test-copy.db");
    // The copy comes first, the installer second, and it gets exactly the signed package.
    assert_eq!(calls, ["backup", "install 99.0.0"]);
    assert_eq!(installed, bundle);
}

#[test]
fn if_the_copy_of_the_books_fails_nothing_is_installed() {
    let server = Server::offering(read_text("good.sig"), read("bundle.bin"));
    let rig = rig(&server.endpoint());
    let (result, calls, _) = flow(&rig, &confirmed("99.0.0"), false);
    assert_eq!(code_of(&result), BackupCode::ErrUpdateBackupFailed);
    assert_eq!(calls, ["backup"], "the installer was not called");
}

#[test]
fn an_update_never_starts_without_a_yes_or_during_work() {
    let server = Server::offering(read_text("good.sig"), read("bundle.bin"));
    let rig = rig(&server.endpoint());
    // No yes: nothing is asked of the server.
    let no_yes = UpdateInstallInput {
        confirmed: false,
        ..confirmed("99.0.0")
    };
    let (result, calls, _) = flow(&rig, &no_yes, true);
    assert_eq!(code_of(&result), BackupCode::ErrUpdateNotConfirmed);
    assert_eq!(
        result.as_ref().unwrap_err().field.as_deref(),
        Some("confirmed")
    );
    assert!(calls.is_empty() && server.paths().is_empty());
    // An entry waits unsaved: refused, with the number of drafts, and the server is not asked.
    rig.env
        .ctx
        .db()
        .write(|tx| {
            tx.execute(
                "INSERT INTO users (username, password_hash, display_name, role)
                 VALUES ('owner', 'x', 'Owner', 'owner')",
                [],
            )?;
            let owner = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO drafts (screen, user_id, payload_json, saved_at)
                 VALUES ('bikri', ?1, '{}', '2026-10-10T09:00:00.000Z')",
                [owner],
            )?;
            Ok::<(), mandi_services::db::DbError>(())
        })
        .unwrap();
    let (result, calls, _) = flow(&rig, &confirmed("99.0.0"), true);
    assert_eq!(code_of(&result), BackupCode::ErrUpdateDuringWork);
    assert!(result.as_ref().unwrap_err().values.contains_key("drafts"));
    assert!(calls.is_empty() && server.paths().is_empty());
    // The check tells the screen the same, so it can say why the button is off.
    let seen = tauri::async_runtime::block_on(check(rig.app.handle(), &rig.env.ctx)).unwrap();
    assert_eq!(seen.drafts, 1);
}

#[test]
fn an_offer_the_owner_did_not_see_or_no_offer_is_not_installed() {
    let server = Server::offering(read_text("good.sig"), read("bundle.bin"));
    let rig = rig(&server.endpoint());
    let (result, calls, _) = flow(&rig, &confirmed("98.0.0"), true);
    assert_eq!(code_of(&result), BackupCode::ErrUpdateChanged);
    assert_eq!(
        result.as_ref().unwrap_err().field.as_deref(),
        Some("expected_version")
    );
    assert!(calls.is_empty());
    // The package was not even downloaded.
    assert_eq!(server.paths(), ["/latest.json"]);

    // The server has nothing newer (204): nothing to install.
    let none = Server::start(|_| {
        vec![Route {
            path: "/latest.json",
            status: 204,
            body: Vec::new(),
        }]
    });
    let rig = self::rig(&none.endpoint());
    let (result, calls, _) = flow(&rig, &confirmed("99.0.0"), true);
    assert_eq!(code_of(&result), BackupCode::ErrUpdateNone);
    assert!(calls.is_empty());
}

#[test]
fn the_check_says_what_is_new_in_both_languages_and_fails_clearly_without_a_server() {
    let server = Server::offering(read_text("good.sig"), read("bundle.bin"));
    let rig = rig(&server.endpoint());
    let found = tauri::async_runtime::block_on(check(rig.app.handle(), &rig.env.ctx)).unwrap();
    assert!(found.available);
    assert_eq!(found.version.as_deref(), Some("99.0.0"));
    assert_eq!(found.current_version, "0.1.0");
    assert_eq!(found.date.as_deref(), Some("2026-10-23T09:00:00Z"));
    assert_eq!(found.notes, NOTES);
    assert_eq!(found.notes_ur, "- ایک\n- دو");
    assert_eq!(found.notes_en, "- One\n- Two");
    // The check never downloads the package.
    assert_eq!(server.paths(), ["/latest.json"]);
    write_or_check(
        "b5_updates_check_available.json",
        &serde_json::to_value(&found).unwrap(),
    );

    // No newer version.
    let none = Server::start(|_| {
        vec![Route {
            path: "/latest.json",
            status: 204,
            body: Vec::new(),
        }]
    });
    let rig = self::rig(&none.endpoint());
    let found = tauri::async_runtime::block_on(check(rig.app.handle(), &rig.env.ctx)).unwrap();
    assert!(!found.available && found.version.is_none() && found.notes.is_empty());
    write_or_check(
        "b5_updates_check_none.json",
        &serde_json::to_value(&found).unwrap(),
    );

    // The server is not there: a clear code, nothing changed.
    let down = rig_for_down_server();
    let error =
        tauri::async_runtime::block_on(check(down.app.handle(), &down.env.ctx)).unwrap_err();
    assert_eq!(error.code, BackupCode::ErrUpdateCheckFailed);
    write_or_check("b5_updates_check_failed.json", &wire(error));
}

/// A rig whose endpoint is a port nobody listens on.
fn rig_for_down_server() -> Rig {
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    rig(&format!("http://127.0.0.1:{port}/latest.json"))
}

/// An error as the frontend gets it, with the volatile reference replaced.
fn wire(error: BackupError) -> Value {
    let mut value = serde_json::to_value(BackupWireError::from(error)).unwrap();
    value["ref"] = json!("E-00000000-0000");
    value
}

fn fixtures_folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
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

#[test]
fn the_refusals_have_the_fixtures_the_frontend_needs() {
    // The wire shape of each refusal of the install, as a fixture each.
    let server = Server::offering(read_text("other.sig"), read("bundle.bin"));
    let rig = rig(&server.endpoint());
    let (not_signed, _, _) = flow(&rig, &confirmed("99.0.0"), true);
    write_or_check(
        "b5_updates_install_not_signed.json",
        &wire(not_signed.unwrap_err()),
    );
    let (changed, _, _) = flow(&rig, &confirmed("98.0.0"), true);
    // (a refusal for the version comes before the package is fetched)
    write_or_check(
        "b5_updates_install_changed.json",
        &wire(changed.unwrap_err()),
    );
    let no_yes = UpdateInstallInput {
        confirmed: false,
        ..confirmed("99.0.0")
    };
    let (not_confirmed, _, _) = flow(&rig, &no_yes, true);
    write_or_check(
        "b5_updates_install_not_confirmed.json",
        &wire(not_confirmed.unwrap_err()),
    );
    let ok = mandi_app_lib::updates::UpdateInstalled {
        version: "99.0.0".to_owned(),
        backup_file: Some("mandi-local-20261023-140000.db".to_owned()),
    };
    write_or_check(
        "b5_updates_install_ok.json",
        &serde_json::to_value(&ok).unwrap(),
    );
    write_or_check(
        "b5_updates_install_request.json",
        &json!({ "expected_version": "99.0.0", "confirmed": true }),
    );
}
