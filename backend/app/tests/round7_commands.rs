//! The commands of plan d5-late round 7 through Tauri's real IPC and the real capability file:
//! the month close commands, and the partner statement R31 on the reports commands (PRD FR-17.6,
//! FR-21.2, FR-17.9, FR-19.10).
//!
//! R31 is the one report the report engine cannot build alone: its lines come from the services.
//! The commands `reports_run`, `reports_print_data` and `reports_export_excel` join the two, so
//! these tests call them the way the screen does and check that the statement answers, prints
//! and exports like every other report, and that a missing partner is refused at its field.
//! The money and the sentences of the statement are checked in
//! `crates/services/tests/reports_round7.rs`.
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

fn log() {
    static LOG: OnceLock<common::Capture> = OnceLock::new();
    LOG.get_or_init(|| {
        let capture = common::Capture::default();
        // Another test file of this binary may have set it; one is enough.
        let _ = tracing::subscriber::set_global_default(mandi_app_lib::logging::subscriber(
            capture.clone(),
        ));
        capture
    });
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
        let app = mock_builder()
            .manage(AppState::with_context(env.ctx.clone()))
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

    /// Makes one partner (confirming the spelling of rule-made words when asked) and gives his id.
    fn partner(&self) -> i64 {
        let mut words: Vec<String> = Vec::new();
        for _ in 0..5 {
            let made = self.call(
                "partners_create",
                json!({ "input": {
                    "name_ur": "الف", "name_en": "Partner A", "opening_capital": null,
                    "confirmed_rules_words": words } }),
            );
            match made {
                Ok(row) => return row["id"].as_i64().unwrap(),
                Err(error) if error["errors"][0]["code"] == "ERR_SPELLING_UNCONFIRMED" => {
                    let text = error["errors"][0]["values"]["words"].as_str().unwrap();
                    words.extend(text.split(' ').map(str::to_owned));
                }
                Err(other) => panic!("{other}"),
            }
        }
        panic!("the spelling question did not end");
    }
}

fn filters(partner: Option<i64>) -> Value {
    json!({ "partner_id": partner })
}

#[test]
fn the_partner_statement_answers_prints_and_exports_like_any_report() {
    let h = Harness::new();
    h.sign_in();
    let partner = h.partner();

    let page = h.ok(
        "reports_run",
        json!({ "input": { "id": "R31", "filters": filters(Some(partner)), "offset": 0, "limit": 0 } }),
    );
    assert_eq!(page["id"], "R31");
    assert_eq!(page["title_id"], "report-r31-title");
    assert_eq!(page["filters"]["partner"]["en"], "Partner A");
    assert_eq!(page["columns"][2]["kind"], "message");
    // A partner with nothing posted: no rows, opening, added, taken away and closing at 0.
    assert_eq!(page["total_count"], 0);
    assert_eq!(page["limit"], 100, "a page of 100 when the screen sends 0");
    assert_eq!(page["summary"].as_array().unwrap().len(), 4);
    assert_eq!(page["summary"][3]["cell"]["paisa"], 0);

    let print = h.ok(
        "reports_print_data",
        json!({ "input": { "id": "R31", "filters": filters(Some(partner)), "language": "en" } }),
    );
    assert_eq!(print["report"], "R31");
    assert_eq!(print["language"], "en");
    assert!(
        print["filters"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["label_id"] == "report-filter-partner" && f["value"] == "Partner A"),
        "{}",
        print["filters"]
    );

    let target = h.root.path().join("statement.xlsx").display().to_string();
    let exported = h.ok(
        "reports_export_excel",
        json!({ "input": { "job_id": "job-r31", "id": "R31", "filters": filters(Some(partner)),
            "language": "ur", "path": target } }),
    );
    assert_eq!(exported["path"], json!(target));
    assert_eq!(&std::fs::read(&target).unwrap()[..2], b"PK");
}

#[test]
fn the_partner_statement_needs_a_partner_and_the_others_refuse_one() {
    let h = Harness::new();
    h.sign_in();
    let field_of = |error: &Value| {
        (
            error["errors"][0]["code"].as_str().unwrap().to_owned(),
            error["errors"][0]["field"].as_str().unwrap().to_owned(),
        )
    };
    for (command, extra) in [
        ("reports_run", json!({ "offset": 0, "limit": 0 })),
        ("reports_print_data", json!({ "language": "en" })),
    ] {
        let mut input = json!({ "id": "R31", "filters": filters(None) });
        input
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let error = h.call(command, json!({ "input": input })).unwrap_err();
        assert_eq!(
            field_of(&error),
            ("ERR_REQUIRED".to_owned(), "filters.partner_id".to_owned()),
            "{command}"
        );
    }
    let unknown = h
        .call(
            "reports_run",
            json!({ "input": { "id": "R31", "filters": filters(Some(9_999)), "offset": 0, "limit": 0 } }),
        )
        .unwrap_err();
    assert_eq!(
        field_of(&unknown),
        ("ERR_NOT_FOUND".to_owned(), "filters.partner_id".to_owned())
    );
    // A report that takes no partner refuses one at its field.
    let other = h
        .call(
            "reports_run",
            json!({ "input": { "id": "R30", "filters": filters(Some(1)), "offset": 0, "limit": 0 } }),
        )
        .unwrap_err();
    assert_eq!(
        field_of(&other),
        (
            "ERR_OUT_OF_RANGE".to_owned(),
            "filters.partner_id".to_owned()
        )
    );
    // The export is refused the same way and writes no file.
    let target = h.root.path().join("none.xlsx").display().to_string();
    let export = h
        .call(
            "reports_export_excel",
            json!({ "input": { "job_id": "j1", "id": "R31", "filters": filters(None),
                "language": "en", "path": target } }),
        )
        .unwrap_err();
    assert_eq!(field_of(&export).1, "filters.partner_id");
    assert!(!std::path::Path::new(&target).exists());
}

#[test]
fn the_month_commands_answer_and_refuse_a_bad_month_at_its_field() {
    let h = Harness::new();
    h.sign_in();
    let list = h.ok("month_list", json!({}));
    assert!(list.is_object(), "{list}");
    for command in ["month_close_preview", "month_close_save"] {
        let error = h
            .call(command, json!({ "input": { "month": "2026-13" } }))
            .unwrap_err();
        assert_eq!(
            error["errors"][0]["code"], "ERR_FORMAT_INVALID",
            "{command}"
        );
        assert_eq!(error["errors"][0]["field"], "month", "{command}");
    }
    let reverse = h
        .call(
            "month_close_reverse",
            json!({ "input": { "month": "October", "reason": "wrong" } }),
        )
        .unwrap_err();
    assert_eq!(reverse["errors"][0]["field"], "month");
}
