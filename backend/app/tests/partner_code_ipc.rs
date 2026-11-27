//! Plan 44 T2e: the partner code through the real IPC (PRD FR-17.11, NFR-SEC-14, Code Standards S8).
//!
//! Every command and report on the refuse list answers `ERR_PARTNER_LOCKED` while the section is
//! locked, whatever the screen shows; the commands on the hide list answer without the partner
//! figures and with `partners_locked` true; the commands that only name a partner stay open. With
//! the code given everything works as before. The commands of the code itself work while locked.
#![cfg(test)]

use mandi_app_lib::state::AppState;
use mandi_services::testing::TestEnv;
use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;

const OWNER_PASSWORD: &str = "secret1";
const CODE: &str = "4826";

struct Harness {
    _app: tauri::App<MockRuntime>,
    window: tauri::WebviewWindow<MockRuntime>,
    _dir: tempfile::TempDir,
}

impl Harness {
    fn new() -> Self {
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

    /// The code of the first error of a refusal.
    fn code(&self, command: &str, payload: Value) -> String {
        let error = self.call(command, payload).unwrap_err();
        error["errors"][0]["code"].as_str().unwrap().to_owned()
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

/// The commands that show a partner's totals or history, with an input that is well formed.
fn refused_commands() -> Vec<(&'static str, Value)> {
    vec![
        ("partners_get", json!({ "input": { "id": 1 } })),
        (
            "partners_create",
            json!({ "input": { "name_ur": "د", "name_en": "Partner D", "opening_capital": null,
                "confirmed_rules_words": [] } }),
        ),
        (
            "partners_update",
            json!({ "input": { "id": 1, "version": 1, "name_ur": "الف", "name_en": "Partner A",
                "is_active": true, "confirmed_rules_words": [] } }),
        ),
        (
            "partners_set_shares",
            json!({ "input": { "effective_from": "10-10-2026",
                "shares": [{ "partner_id": 1, "share": "100" }] } }),
        ),
        ("partners_overview", json!({})),
        (
            "partners_statement",
            json!({ "input": { "partner_id": 1, "from": null, "to": null, "offset": 0, "limit": 0 } }),
        ),
        (
            "partners_statement_month",
            json!({ "input": { "partner_id": 1, "month": null } }),
        ),
        (
            "partners_statement_day",
            json!({ "input": { "partner_id": 1, "date": "2026-10-10" } }),
        ),
        (
            "partners_statement_month_print_data",
            json!({ "input": { "partner_id": 1, "month": null, "language": "ur" } }),
        ),
        (
            "partners_open_ventures",
            json!({ "input": { "partner_id": 1 } }),
        ),
        (
            "vouchers_save_capital_in",
            json!({ "input": { "date": "10-10-2026", "partner_id": 1, "amount": "50,000",
                "narration": null, "confirm_cash_below_zero": false } }),
        ),
        (
            "vouchers_save_capital_out",
            json!({ "input": { "date": "10-10-2026", "partner_id": 1, "amount": "10,000",
                "narration": null, "confirm_cash_below_zero": false } }),
        ),
        (
            "capital_history",
            json!({ "input": { "partner_id": 1, "offset": 0, "limit": 0 } }),
        ),
        // Plan 80 gate R4-1 and R4-2: the fruit one partner took, and the lists of a partner's own
        // money (the lists of every type leave those vouchers out instead, see the second test).
        (
            "fruit_list",
            json!({ "input": { "partner_id": 1, "offset": 0, "limit": 0 } }),
        ),
        (
            "vouchers_list",
            json!({ "input": { "voucher_type": "capital_in", "from": "", "to": "" } }),
        ),
        (
            "vouchers_list",
            json!({ "input": { "voucher_type": "capital_out", "from": "", "to": "" } }),
        ),
        // The partner reports and views, in every way to ask for them.
        (
            "reports_run",
            json!({ "input": { "id": "R30", "filters": {}, "offset": 0, "limit": 10 } }),
        ),
        (
            "reports_run",
            json!({ "input": { "id": "R31", "filters": { "partner_id": 1 }, "offset": 0, "limit": 10 } }),
        ),
        (
            "reports_run",
            json!({ "input": { "id": "R29", "filters": { "partner_id": 1 }, "offset": 0, "limit": 10 } }),
        ),
        (
            "reports_run",
            json!({ "input": { "id": "R33", "filters": { "trip_view": "partner" }, "offset": 0, "limit": 10 } }),
        ),
        (
            "reports_run",
            json!({ "input": { "id": "R36", "filters": { "garden_view": "partner" }, "offset": 0, "limit": 10 } }),
        ),
        (
            "reports_run",
            json!({ "input": { "id": "R37", "filters": { "partner_id": 1 }, "offset": 0, "limit": 10 } }),
        ),
        (
            "reports_print_data",
            json!({ "input": { "id": "R30", "filters": {}, "language": "en" } }),
        ),
        (
            "reports_print_data",
            json!({ "input": { "id": "R31", "filters": { "partner_id": 1 }, "language": "en" } }),
        ),
        (
            "analytics_run",
            json!({ "input": { "view": "F", "filters": {}, "offset": 0, "limit": 10 } }),
        ),
        (
            "analytics_run",
            json!({ "input": { "view": "K", "filters": {}, "offset": 0, "limit": 10 } }),
        ),
        (
            "analytics_run",
            json!({ "input": { "view": "A", "filters": { "partner_id": 1 }, "offset": 0, "limit": 10 } }),
        ),
        (
            "analytics_print_data",
            json!({ "input": { "view": "F", "filters": {}, "language": "en" } }),
        ),
        (
            "reports_print_report",
            json!({ "input": { "report": { "id": "R30", "filters": {}, "language": "en" }, "to": "pdf" } }),
        ),
        (
            "analytics_print_view",
            json!({ "input": { "view": { "view": "F", "filters": {}, "language": "en" }, "to": "pdf" } }),
        ),
        // The first day of a partner's statement or of his money taken is a partner fact (plan 76
        // speed review): the other sources of `windows_bounds` stay open.
        (
            "windows_bounds",
            json!({ "input": { "source": { "kind": "partner_statement", "partner_id": 1 } } }),
        ),
    ]
}

fn partners_set_up(h: &Harness) {
    h.ok("setup_first_run", json!({ "input": first_run() }));
    h.ok(
        "auth_login",
        json!({ "input": { "username": "owner", "password": OWNER_PASSWORD } }),
    );
    for (ur, en) in [("الف", "Partner A"), ("ب", "Partner B"), ("ج", "Partner C")] {
        h.ok(
            "partners_create",
            json!({ "input": { "name_ur": ur, "name_en": en, "opening_capital": null,
                "confirmed_rules_words": [] } }),
        );
    }
    h.ok(
        "partners_set_shares",
        json!({ "input": { "effective_from": "10-10-2026", "shares": [
            { "partner_id": 1, "share": "40" }, { "partner_id": 2, "share": "30" },
            { "partner_id": 3, "share": "30" } ] } }),
    );
}

#[test]
fn every_partner_command_and_report_is_refused_while_locked() {
    let h = Harness::new();
    partners_set_up(&h);

    // No code yet: the first-run partner setup above worked, and the status says to set one.
    let status = h.ok("partners_code_status", json!({}));
    assert_eq!(
        status,
        json!({ "is_set": false, "unlocked": false, "wait_seconds": 0 })
    );
    assert_eq!(
        h.code("partners_code_unlock", json!({ "input": { "code": CODE } })),
        "ERR_PARTNER_CODE_NOT_SET"
    );

    // Setting the code needs the Owner's password.
    let wrong = h
        .call(
            "partners_code_set",
            json!({ "input": { "password": "not-it-1", "code": CODE, "code_again": CODE } }),
        )
        .unwrap_err();
    assert_eq!(wrong["errors"][0]["code"], "ERR_WRONG_PASSWORD");
    assert_eq!(wrong["errors"][0]["field"], "password");
    let set = h.ok(
        "partners_code_set",
        json!({ "input": { "password": OWNER_PASSWORD, "code": CODE, "code_again": CODE } }),
    );
    assert_eq!(
        set,
        json!({ "is_set": true, "unlocked": true, "wait_seconds": 0 })
    );

    // Open: every command on the list works, or fails for its own reasons, never because of the lock.
    for (command, input) in refused_commands() {
        if let Err(error) = h.call(command, input) {
            assert_ne!(
                error["errors"][0]["code"], "ERR_PARTNER_LOCKED",
                "{command} is refused while the section is open"
            );
        }
    }

    // Locked: every one is refused.
    h.ok("partners_code_lock", json!({}));
    for (command, input) in refused_commands() {
        assert_eq!(
            h.code(command, input.clone()),
            "ERR_PARTNER_LOCKED",
            "{command} {input}"
        );
    }

    // The commands that only name a partner, and the code itself, stay open while locked.
    // The running total of typed shares is one of them (plan 81): the new garden and trip forms
    // are outside the section and ask for it. It answers from the typed text only, still needs the
    // sign-in and names no stored figure. Four partners exist here (the open loop made one), and
    // every active partner is listed.
    let typed = h.ok(
        "partners_check_shares",
        json!({ "input": { "shares": [
            { "partner_id": 1, "share": "50" }, { "partner_id": 2, "share": "50" },
            { "partner_id": 3, "share": "0" }, { "partner_id": 4, "share": "0" } ] } }),
    );
    assert_eq!(typed["total_bps"], json!(10_000));
    assert_eq!(typed["is_100"], json!(true));
    let short = h.ok(
        "partners_check_shares",
        json!({ "input": { "shares": [
            { "partner_id": 1, "share": "50" }, { "partner_id": 2, "share": "40" },
            { "partner_id": 3, "share": "0" }, { "partner_id": 4, "share": "0" } ] } }),
    );
    assert_eq!(short["total_bps"], json!(9_000));
    assert_eq!(short["errors"][0]["code"], json!("ERR_SHARES_NOT_100"));
    let list = h.ok("partners_list", json!({}));
    assert_eq!(list["partners_locked"], json!(true));
    // Three partners were set up and the open loop above made a fourth with `partners_create`.
    let named = list["partners"].as_array().unwrap().len();
    assert_eq!(named, 4);
    for partner in list["partners"].as_array().unwrap() {
        assert_eq!(partner["share_bps"], json!(0));
    }
    assert_eq!(
        h.ok("partners_code_status", json!({})),
        json!({ "is_set": true, "unlocked": false, "wait_seconds": 0 })
    );
    // A report without a partner figure is not affected.
    h.ok(
        "reports_run",
        json!({ "input": { "id": "R18", "filters": {}, "offset": 0, "limit": 10 } }),
    );

    // A wrong code says how many tries are left; the right one opens the section.
    let wrong = h
        .call(
            "partners_code_unlock",
            json!({ "input": { "code": "1111" } }),
        )
        .unwrap_err();
    assert_eq!(wrong["errors"][0]["code"], "ERR_PARTNER_CODE_WRONG");
    assert_eq!(wrong["errors"][0]["values"]["tries_left"], json!(9));
    h.ok("partners_code_unlock", json!({ "input": { "code": CODE } }));
    let overview = h.ok("partners_overview", json!({}));
    assert_eq!(overview.as_array().unwrap().len(), named);
    assert_eq!(
        h.ok("partners_list", json!({}))["partners_locked"],
        json!(false)
    );
}

#[test]
fn what_is_hidden_while_locked_comes_back_with_the_code() {
    let h = Harness::new();
    partners_set_up(&h);
    h.ok(
        "partners_code_set",
        json!({ "input": { "password": OWNER_PASSWORD, "code": CODE, "code_again": CODE } }),
    );

    let hidden_flags = |h: &Harness| -> Vec<(&'static str, bool)> {
        vec![
            (
                "dashboard_get",
                h.ok("dashboard_get", json!({}))["partners_locked"] == json!(true),
            ),
            (
                "assist_attention",
                h.ok("assist_attention", json!({}))["partners_locked"] == json!(true),
            ),
            (
                "day_close_preview",
                h.ok("day_close_preview", json!({ "input": { "up_to": "" } }))["summary"]["partners_locked"]
                    == json!(true),
            ),
            (
                "audit_list",
                h.ok(
                    "audit_list",
                    json!({ "input": { "filters": {}, "offset": 0, "limit": 10 } }),
                )["partners_locked"]
                    == json!(true),
            ),
            (
                "lent_dues",
                h.ok("lent_dues", json!({ "input": {} }))["partner_locked"] == json!(true),
            ),
            // Plan 80 gate R4-1 and R4-2.
            (
                "fruit_list",
                h.ok("fruit_list", json!({ "input": {} }))["partners_locked"] == json!(true),
            ),
            (
                "assist_attention_overdue",
                h.ok("assist_attention_overdue", json!({}))["partners_locked"] == json!(true),
            ),
        ]
    };

    // Open: nothing is hidden, and the strip has the three partners.
    assert!(hidden_flags(&h).iter().all(|(_, hidden)| !hidden));
    assert_eq!(
        h.ok("dashboard_get", json!({}))["partners"]
            .as_array()
            .unwrap()
            .len(),
        3
    );

    // Locked: all are hidden, and the strip is empty.
    h.ok("partners_code_lock", json!({}));
    // Umar 28-11-2026, plan 100 E: a partner's personal expense, with his کل رقم before and after,
    // is open without the code.
    let own = h.ok(
        "vouchers_preview_expense",
        json!({ "input": { "entry": { "type": "personal_expense", "date": "10-10-2026",
            "partner_id": 1, "amount": "2,000", "reason": "گھر کا خرچ", "return_by": null,
            "taken_by": null, "narration": null, "confirm_cash_below_zero": false },
            "replacing_voucher_id": null } }),
    );
    assert_eq!(own["partners_locked"], json!(false));
    assert!(own["partner_effect"].is_object(), "{own}");
    // The first day of the sales list is not a partner fact: it still answers.
    h.ok(
        "windows_bounds",
        json!({ "input": { "source": { "kind": "sales" } } }),
    );
    for (command, hidden) in hidden_flags(&h) {
        assert!(hidden, "{command} still shows the partner figures");
    }
    assert_eq!(h.ok("dashboard_get", json!({}))["partners"], json!([]));

    // The unlock brings them back.
    h.ok("partners_code_unlock", json!({ "input": { "code": CODE } }));
    assert!(hidden_flags(&h).iter().all(|(_, hidden)| !hidden));
}

#[test]
fn the_code_commands_need_the_sign_in_and_a_logout_locks_the_section() {
    let h = Harness::new();
    partners_set_up(&h);
    h.ok(
        "partners_code_set",
        json!({ "input": { "password": OWNER_PASSWORD, "code": CODE, "code_again": CODE } }),
    );
    // The idle lock (auth_lock) closes the section; so does a logout and a new sign-in.
    assert_eq!(
        h.ok("partners_code_status", json!({}))["unlocked"],
        json!(true)
    );
    h.ok("auth_lock", json!({}));
    assert_eq!(
        h.code("partners_code_status", json!({})),
        "ERR_SESSION_LOCKED"
    );
    h.ok(
        "auth_unlock",
        json!({ "input": { "password": OWNER_PASSWORD } }),
    );
    assert_eq!(
        h.ok("partners_code_status", json!({}))["unlocked"],
        json!(false)
    );
    h.ok("partners_code_unlock", json!({ "input": { "code": CODE } }));
    h.ok("auth_logout", json!({}));
    assert_eq!(
        h.code("partners_code_status", json!({})),
        "ERR_NOT_LOGGED_IN"
    );
    h.ok(
        "auth_login",
        json!({ "input": { "username": "owner", "password": OWNER_PASSWORD } }),
    );
    assert_eq!(
        h.ok("partners_code_status", json!({}))["unlocked"],
        json!(false)
    );
}

/// The commands that cancel something (PRD FR-7.93 v1.61, plan 100 C), each with a request aimed at
/// a record that does not exist.
fn cancel_commands() -> Vec<(&'static str, Value)> {
    let by_id = |id: &str| json!({ id: 999_999, "version": 1 });
    let by_move = || json!({ "move_id": 999_999 });
    vec![
        ("bikri_cancel", by_id("id")),
        ("bikri_write_off_undo", by_move()),
        ("vouchers_cancel", by_id("id")),
        ("fruit_cancel", by_id("id")),
        ("purchases_cancel", by_id("id")),
        ("purchases_write_off_undo", by_move()),
        ("trips_cost_cancel", by_id("id")),
        ("trips_other_sale_cancel", by_id("id")),
        ("gardens_entry_cancel", by_id("id")),
        ("gardens_arrival_cancel", by_id("id")),
        ("gardens_reopen", by_id("id")),
        ("gardens_arrival_write_off_undo", by_move()),
    ]
}

fn with(mut base: Value, reason: &str, code: &str, sign: &str) -> Value {
    base["reason"] = json!(reason);
    base["partner_code"] = json!(code);
    base["sign"] = json!(sign);
    json!({ "input": base })
}

#[test]
fn every_cancel_command_asks_the_code_the_sign_and_a_reason_first() {
    let h = Harness::new();
    partners_set_up(&h);
    h.ok(
        "partners_code_set",
        json!({ "input": { "password": OWNER_PASSWORD, "code": CODE, "code_again": CODE } }),
    );
    h.ok("partners_code_lock", json!({}));
    for (command, base) in cancel_commands() {
        // An old client that sends no proof is refused before anything is looked for.
        assert_eq!(
            h.code(command, with(base.clone(), "غلط", "", "")),
            "ERR_SIGN_REQUIRED",
            "{command}"
        );
        assert_eq!(
            h.code(command, with(base.clone(), "غلط", "1111", "adnan")),
            "ERR_PARTNER_CODE_WRONG",
            "{command}"
        );
        assert_eq!(
            h.code(command, with(base.clone(), "غلط", CODE, "munshi")),
            "ERR_SIGN_WRONG",
            "{command}"
        );
        // The right proof gets through to the cancel, which finds nothing.
        assert_eq!(
            h.code(command, with(base, "غلط", CODE, "adnan")),
            "ERR_NOT_FOUND",
            "{command}"
        );
    }
    // The 10-second undo asks for nothing, but only straight after a save: an id nobody saved is
    // refused.
    assert_eq!(
        h.code(
            "bikri_undo",
            json!({ "input": { "id": 999_999, "confirm_cash_below_zero": false } })
        ),
        "ERR_UNDO_EXPIRED"
    );
}
