//! The Excel export commands (plan 22 round 10) through Tauri's real IPC and the real capability
//! file, with the request and answer pairs the frontend copies
//! (`backend/app/tests/fixtures/export_*.json`).
//!
//! A pair is `<name>_request.json` (exactly the `input` the command takes) and `<name>.json` (the
//! answer, or the error shape `ok`, `errors`, `ref`). What changes from run to run is replaced
//! before comparing: the reference number, the file path and the size. The test FAILS when an
//! answer differs from its file. After a deliberate change write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-app --test export_commands`, read the diff and
//! tell `frontend`.
#![cfg(test)]

mod common;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

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
use tauri::Listener;
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
    app: tauri::App<MockRuntime>,
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
            app,
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
        write_or_check(&format!("export_{name}_request.json"), &shown);
    }
    let answer = h.call(command, payload);
    let mut shown = match &answer {
        Ok(value) => value.clone(),
        Err(error) => error.clone(),
    };
    steady(&mut shown);
    write_or_check(&format!("export_{name}.json"), &shown);
    answer
}

fn listen(h: &Harness) -> Arc<Mutex<Vec<Value>>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    h.app.listen("export-progress", move |event| {
        let value: Value = serde_json::from_str(event.payload()).unwrap();
        sink.lock().unwrap().push(value);
    });
    seen
}

fn path_in(h: &Harness, name: &str) -> String {
    h.root.path().join(name).display().to_string()
}

#[test]
fn the_export_commands_are_refused_before_sign_in() {
    let h = Harness::new();
    let report = json!({ "input": { "job_id": "j1", "id": "R18", "language": "en", "path": path_in(&h, "a.xlsx") } });
    let view = json!({ "input": { "job_id": "j1", "view": "A", "filters": {}, "language": "en", "path": path_in(&h, "b.xlsx") } });
    for (command, input) in [
        ("reports_export_excel", report),
        ("analytics_export_excel", view),
    ] {
        let error = h.call(command, input).unwrap_err();
        assert_eq!(error["errors"][0]["code"], "ERR_NOT_LOGGED_IN", "{command}");
    }
    assert!(
        std::fs::read_dir(h.root.path()).unwrap().next().is_none(),
        "nothing written"
    );
}

#[test]
fn a_report_is_saved_as_a_file_with_progress_events() {
    let h = Harness::new();
    h.sign_in();
    let seen = listen(&h);
    let target = path_in(&h, "trial-balance.xlsx");
    let request = json!({
        "job_id": "job-r18", "id": "R18", "filters": {}, "language": "en", "path": target
    });
    let answer = pair(
        &h,
        "reports_export_excel",
        "reports_export_excel",
        Some(request),
    )
    .unwrap();
    assert_eq!(answer["path"], json!(target));
    assert!(answer["size_bytes"].as_u64().unwrap() > 0);
    // A real .xlsx is a zip file.
    let bytes = std::fs::read(&target).unwrap();
    assert_eq!(&bytes[..2], b"PK");
    // The events: the reading, the writing and the end, all with the job id.
    let events = seen.lock().unwrap().clone();
    assert!(events.len() >= 3, "{events:?}");
    assert!(events.iter().all(|e| e["job_id"] == "job-r18"));
    assert_eq!(events.first().unwrap()["stage"], "reading");
    assert_eq!(events.last().unwrap()["stage"], "done");
    let done = events.last().unwrap();
    assert_eq!(done["done"], done["total"]);
    // The same export in Urdu.
    let urdu = pair(
        &h,
        "reports_export_excel_urdu",
        "reports_export_excel",
        Some(json!({
            "job_id": "job-r18-ur", "id": "R18", "filters": { "as_of": "2026-10-10" },
            "language": "ur", "path": path_in(&h, "trial-balance-ur.xlsx")
        })),
    )
    .unwrap();
    assert!(urdu["size_bytes"].as_u64().unwrap() > 0);
}

#[test]
fn an_analytics_view_is_saved_as_a_file() {
    let h = Harness::new();
    h.sign_in();
    let seen = listen(&h);
    let target = path_in(&h, "view-a.xlsx");
    let answer = pair(
        &h,
        "analytics_export_excel",
        "analytics_export_excel",
        Some(json!({
            "job_id": "job-a", "view": "A", "filters": {}, "language": "ur", "path": target
        })),
    )
    .unwrap();
    assert_eq!(answer["path"], json!(target));
    assert!(std::path::Path::new(&target).is_file());
    let events = seen.lock().unwrap().clone();
    assert_eq!(events.last().unwrap()["stage"], "done");
    assert!(events.iter().all(|e| e["job_id"] == "job-a"));
}

#[test]
fn a_bad_path_job_id_or_filter_is_refused_with_its_field_and_nothing_is_written() {
    let h = Harness::new();
    h.sign_in();
    let field_of = |error: &Value| error["errors"][0]["field"].as_str().unwrap().to_owned();
    let relative = pair(
        &h,
        "reports_export_excel_refused_path",
        "reports_export_excel",
        Some(json!({ "job_id": "j1", "id": "R18", "filters": {}, "language": "en", "path": "trial.xlsx" })),
    )
    .unwrap_err();
    assert_eq!(relative["errors"][0]["code"], "ERR_OUT_OF_RANGE");
    assert_eq!(field_of(&relative), "path");
    let wrong_ext = h
        .call(
            "reports_export_excel",
            json!({ "input": { "job_id": "j1", "id": "R18", "filters": {}, "language": "en", "path": path_in(&h, "trial.csv") } }),
        )
        .unwrap_err();
    assert_eq!(field_of(&wrong_ext), "path");
    let job = pair(
        &h,
        "reports_export_excel_refused_job",
        "reports_export_excel",
        Some(json!({ "job_id": "bad id!", "id": "R18", "filters": {}, "language": "en", "path": path_in(&h, "a.xlsx") })),
    )
    .unwrap_err();
    assert_eq!(field_of(&job), "job_id");
    let filter = pair(
        &h,
        "reports_export_excel_refused_filter",
        "reports_export_excel",
        Some(json!({ "job_id": "j1", "id": "R11", "filters": { "from": "not a date" }, "language": "en", "path": path_in(&h, "a.xlsx") })),
    )
    .unwrap_err();
    assert_eq!(field_of(&filter), "filters.from");
    let view = h
        .call(
            "analytics_export_excel",
            json!({ "input": { "job_id": "j1", "view": "A", "filters": { "from": "99-99-9999" }, "language": "en", "path": path_in(&h, "v.xlsx") } }),
        )
        .unwrap_err();
    assert_eq!(view["errors"][0]["field"], "filters.from");
    assert!(
        std::fs::read_dir(h.root.path()).unwrap().all(|e| {
            let name = e.unwrap().file_name().to_string_lossy().into_owned();
            name == "backups" || name == "cloud"
        }),
        "a refused export leaves no file"
    );
}
