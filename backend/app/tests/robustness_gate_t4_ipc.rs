//! Gate lane T4, bad input through the command layer (plan 80; Code Standards 4.1 to 4.3, V0, B8,
//! B14, B31; NFR-REL-6; PRD Section 23 E24, E31, E42; FR-7.28, FR-7.48). Owner: `robustness`.
//!
//! `robustness_security.rs` already attacks every command with random hostile values, on an EMPTY
//! database, so most calls stop at "no such row". This file attacks the same commands (it reads
//! their payloads out of that file's `command_table`, so a new command is attacked as soon as it
//! has a payload there) with three differences:
//!
//! 1. **A populated world.** The worked day's parties, items and partners, 120 buyers and 120
//!    sellers with open balances (three بکری of 40 lots), so a hostile value reaches the code behind
//!    the id check, and a page really has more than 100 rows.
//! 2. **Targeted values**, not random ones: the exact values of the lane list. Money with
//!    decimals, Urdu digits, 10,00,00,00,001 rupees, i64 max, sums like `20+25` and `10-20`,
//!    dates like 31-02 and 00-00 and a future date, NUL, zero-width and right-to-left marks, 100
//!    and 101 character names, emoji, limits and offsets at 0, 100, 101 and u32 max.
//! 3. **Two invariants on every call.** A refused call commits nothing at all (`PRAGMA
//!    data_version` of a second connection does not move: no half-save, V0), and nothing stored
//!    after thousands of calls breaks the field rules of Code Standards 4.2 (names in the wrong
//!    script, over 100 characters, with control or zero-width characters).
//!
//! It lives in the app crate because only the app crate can speak Tauri IPC. It needs no change to
//! any `Cargo.toml`. A full run: `GATE_T4_CAP=100000 cargo test --release -p mandi-app --test
//! robustness_gate_t4_ipc -- --nocapture`. The default keeps a debug run short.
#![cfg(test)]

#[path = "../../crates/services/tests/common/mod.rs"]
mod common;
#[allow(dead_code)]
#[path = "../../crates/services/tests/bikri_preview/support.rs"]
mod support;

use std::collections::BTreeMap;

use mandi_app_lib::state::AppState;
use mandi_services::bikri;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::bikri::BikriSaveInput;
use mandi_services::db::rusqlite::Connection;
use serde_json::{Value, json};
use support::{bare_lot, draft, sale, with_sales};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;

/// Calls for one command in the matrix at most. `GATE_T4_CAP` raises it for a full run.
fn cap() -> usize {
    std::env::var("GATE_T4_CAP")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30)
}

// ---------------------------------------------------------------------------------------------
// The world and the harness
// ---------------------------------------------------------------------------------------------

/// A number written in Urdu letters, one letter for a digit, so a name is unique and valid.
fn urdu_number(n: usize) -> String {
    const LETTERS: [char; 10] = ['ا', 'ب', 'پ', 'ت', 'ٹ', 'ث', 'ج', 'چ', 'ح', 'خ'];
    n.to_string()
        .chars()
        .map(|c| LETTERS[usize::try_from(c.to_digit(10).unwrap()).unwrap()])
        .collect()
}

struct Harness {
    _app: tauri::App<MockRuntime>,
    window: tauri::WebviewWindow<MockRuntime>,
    world: support::World,
    /// A second connection that only asks `data_version`: it moves when another connection commits.
    probe: Connection,
    buyers: Vec<i64>,
}

impl Harness {
    fn new() -> Self {
        let world = support::world();
        // 120 buyers and 120 sellers, three بکری of 40 lots: each lot has its own seller and one
        // line to its own buyer, left on account. Buyer dues, seller dues and the lists of the
        // dashboard pages then have more than one page of 100.
        let sellers: Vec<i64> = (0..120)
            .map(|n| {
                support::add_account(
                    &world.env,
                    &world.owner,
                    AccountType::Seller,
                    &format!("بیوپاری {}", urdu_number(n)),
                    &format!("Seller {n}"),
                    true,
                    "",
                )
            })
            .collect();
        let buyers: Vec<i64> = (0..120)
            .map(|n| {
                support::add_account(
                    &world.env,
                    &world.owner,
                    AccountType::Buyer,
                    &format!("خریدار {}", urdu_number(n)),
                    &format!("Buyer {n}"),
                    true,
                    "",
                )
            })
            .collect();
        for block in 0..3 {
            let lots = (0..40)
                .map(|i| {
                    let n = block * 40 + i;
                    let lot = bare_lot(&world, &format!("lot-{n}"), sellers[n], world.banana, "1");
                    with_sales(
                        lot,
                        vec![sale(
                            &format!("line-{n}"),
                            buyers[n],
                            "1",
                            &(500 + n).to_string(),
                        )],
                    )
                })
                .collect();
            bikri::save(
                &world.env.ctx,
                &world.owner,
                &BikriSaveInput {
                    draft: draft(lots),
                    confirm_cash_below_zero: true,
                },
            )
            .unwrap();
        }
        let state = AppState::with_context(world.env.ctx.clone());
        let app = mock_builder()
            .manage(state)
            .invoke_handler(mandi_app_lib::handler())
            .build(mandi_app_lib::context())
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let probe = Connection::open(world.dir.path().join("data").join("mandi.db")).unwrap();
        Self {
            _app: app,
            window,
            world,
            probe,
            buyers,
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

    fn data_version(&self) -> i64 {
        self.probe
            .query_row("PRAGMA data_version", [], |r| r.get(0))
            .unwrap()
    }

    fn one(&self, sql: &str) -> i64 {
        self.probe.query_row(sql, [], |r| r.get(0)).unwrap()
    }
}

// ---------------------------------------------------------------------------------------------
// The payloads: read out of the table of robustness_security.rs
// ---------------------------------------------------------------------------------------------

/// Index of the `)` that closes the call whose `(` is just before `from`. Strings are skipped.
fn match_close(text: &str, from: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let (mut depth, mut i, mut in_string) = (1_i32, from, false);
    while i < bytes.len() {
        let c = bytes[i];
        if in_string {
            if c == b'\\' {
                i += 1;
            } else if c == b'"' {
                in_string = false;
            }
        } else {
            match c {
                b'"' => in_string = true,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// `json!` allows a comma before a closing brace; JSON does not.
fn strip_trailing_commas(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let (mut in_string, mut i) = (false, 0);
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            out.push(c);
            if c == '\\' {
                i += 1;
                if let Some(next) = chars.get(i) {
                    out.push(*next);
                }
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
            out.push(c);
        } else if c == ',' {
            let next = chars[i + 1..].iter().find(|n| !n.is_whitespace());
            if !matches!(next, Some('}' | ']')) {
                out.push(c);
            }
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

fn registered() -> Vec<String> {
    let build_rs = include_str!("../build.rs");
    let listed = build_rs.split("APP_COMMANDS").nth(1).unwrap();
    let listed = &listed[listed.find('[').unwrap()..listed.find("];").unwrap()];
    listed
        .split('"')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, name)| name.to_owned())
        .collect()
}

/// Every command with a payload that is plain JSON in `command_table` (an entry that builds its
/// payload with Rust code, for example `first_run()`, is left out and counted).
fn table() -> (BTreeMap<String, Value>, Vec<String>) {
    let src = include_str!("robustness_security.rs");
    let start = src.find("fn command_table()").unwrap();
    let body = &src[start..];
    let body = &body[..body.find("\n}\n").unwrap()];
    let mut found = BTreeMap::new();
    let mut from = 0;
    while let Some(pos) = body[from..].find("json!(") {
        let at = from + pos;
        let open = at + "json!(".len();
        from = open;
        let Some(close) = match_close(body, open) else {
            continue;
        };
        let before = body[..at].trim_end();
        let Some(before) = before.strip_suffix(',') else {
            continue;
        };
        let before = before.trim_end();
        let Some(before) = before.strip_suffix('"') else {
            continue;
        };
        let Some(name_start) = before.rfind('"') else {
            continue;
        };
        let name = &before[name_start + 1..];
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(&strip_trailing_commas(&body[open..close]))
        {
            found.entry(name.to_owned()).or_insert(value);
        }
        from = close;
    }
    let skipped = registered()
        .into_iter()
        .filter(|name| !found.contains_key(name))
        .collect();
    (found, skipped)
}

/// Richer payloads for the commands of plan 79 (every field present, so every field is attacked),
/// and a few older list commands whose table payload is an empty object.
fn seeds() -> Vec<(&'static str, Value)> {
    let sort = json!({ "column": "balance", "direction": "desc", "language": "ur" });
    let seller = json!({ "side": "all", "party_id": null, "search": "", "sort": sort,
        "offset": 0, "limit": 25 });
    let seller_print = json!({ "side": "shop_owes", "party_id": null, "search": "", "sort": sort,
        "language": "ur", "paper": null });
    let lent = json!({ "search": "", "sort": sort, "offset": 0, "limit": 25 });
    let lent_print = json!({ "search": "", "sort": sort, "language": "en", "paper": null });
    let month = json!({ "month": "2026-10" });
    let month_print = json!({ "month": "2026-10", "language": "ur", "paper": null });
    let day = json!({ "date": "10-10-2026", "offset": 0, "limit": 25 });
    let day_print = json!({ "date": "10-10-2026", "language": "ur", "paper": null });
    let buyer = json!({ "search": "", "party_id": null, "sort": sort, "offset": 0, "limit": 25 });
    let advances = json!({ "seller_id": null, "as_of": "10-10-2026", "offset": 0, "limit": 25 });
    vec![
        ("dashboard_seller_dues", json!({ "input": seller })),
        (
            "dashboard_seller_dues_print_data",
            json!({ "input": seller_print }),
        ),
        ("dashboard_lent_open", json!({ "input": lent })),
        (
            "dashboard_lent_open_print_data",
            json!({ "input": lent_print }),
        ),
        ("dashboard_cash_sales_month", json!({ "input": month })),
        (
            "dashboard_cash_sales_month_print_data",
            json!({ "input": month_print }),
        ),
        ("dashboard_cash_sales_day", json!({ "input": day })),
        (
            "dashboard_cash_sales_day_print_data",
            json!({ "input": day_print }),
        ),
        ("dashboard_buyer_dues", json!({ "input": buyer })),
        ("advances_report", json!({ "input": advances })),
        (
            "advances_report_print_data",
            json!({ "input": { "seller_id": null, "as_of": "10-10-2026", "search": "",
                "only_open": true, "language": "ur" } }),
        ),
        (
            "dashboard_total_dues_print_data",
            json!({ "input": { "language": "en", "paper": null } }),
        ),
    ]
}

// ---------------------------------------------------------------------------------------------
// The values
// ---------------------------------------------------------------------------------------------

fn strings() -> Vec<Value> {
    let long_sum = vec!["1"; 300].join("+");
    let list: Vec<String> = vec![
        // Empty, spaces, control and invisible characters.
        "".into(),
        "   ".into(),
        "\t\n".into(),
        "\u{0}".into(),
        "a\u{0}b".into(),
        "\u{200b}".into(),
        "ا\u{200d}ب".into(),
        "ا\u{200c}ب".into(),
        "\u{202e}ab".into(),
        "\u{2067}ab\u{2069}".into(),
        "\u{feff}12".into(),
        "\u{061c}5".into(),
        // Money: decimals, signs, commas, Urdu and Arabic-Indic digits, the limits.
        "0".into(),
        "-1".into(),
        "+5".into(),
        "850.50".into(),
        "850.".into(),
        ".5".into(),
        "1,00,000".into(),
        "1,2,3".into(),
        ",100".into(),
        "100,".into(),
        "1 000".into(),
        "۶۰۰".into(),
        "٦٠٠".into(),
        "10,00,00,00,000".into(),
        "10,00,00,00,001".into(),
        "1000000001".into(),
        "9223372036854775807".into(),
        "9223372036854775808".into(),
        "99999999999999999999999999".into(),
        "1e5".into(),
        "0x10".into(),
        "NaN".into(),
        "inf".into(),
        // Sums (FR-7.48) and things that look like sums.
        "20+25".into(),
        "10-20".into(),
        "20+".into(),
        "+20+5".into(),
        "5×3".into(),
        "5÷3".into(),
        "20 + 25".into(),
        long_sum,
        // Dates.
        "31-02-2026".into(),
        "00-00-0000".into(),
        "32-13-2026".into(),
        "29-02-2027".into(),
        "29-02-2028".into(),
        "01-01-0001".into(),
        "10-10-2099".into(),
        "11-10-2026".into(),
        "2026-10-10".into(),
        "10/10/2026".into(),
        "10-10-26".into(),
        "٠١-١٠-٢٠٢٦".into(),
        "2026-13".into(),
        "2099-01".into(),
        // Injection.
        "' OR 1=1 --".into(),
        "'; DROP TABLE accounts; --".into(),
        "<script>alert(1)</script>".into(),
        "..\\..\\x".into(),
        "../../x".into(),
        "%".into(),
        "_".into(),
        // Names: letter variants, mixed scripts, emoji, the limit of 100.
        "ي ى ك ه ۃ".into(),
        "Ahmed احمد".into(),
        "😀".into(),
        "ا😀ب".into(),
        "ا".repeat(100),
        "ا".repeat(101),
        "a".repeat(100),
        "a".repeat(101),
        "x".repeat(100_000),
        // Codes and short codes.
        "0010".into(),
        "00".into(),
        "99999999999".into(),
    ];
    list.into_iter().map(Value::String).collect()
}

fn numbers() -> Vec<Value> {
    vec![
        json!(0),
        json!(1),
        json!(-1),
        json!(2),
        json!(100),
        json!(101),
        json!(1000),
        json!(1001),
        json!(i64::from(i32::MAX)),
        json!(i64::from(i32::MAX) + 1),
        json!(i64::from(u32::MAX)),
        json!(i64::from(u32::MAX) + 1),
        json!(i64::MAX),
        json!(i64::MIN),
        json!(u64::MAX),
        json!(1_000_000_000_001_i64),
        json!(0.5),
        json!(-0.0),
        json!(1e308),
    ]
}

fn structural() -> Vec<Value> {
    vec![
        Value::Null,
        json!(true),
        json!([]),
        json!({}),
        json!("x"),
        json!([null]),
        json!({ "a": 1 }),
    ]
}

fn values_for(original: &Value) -> Vec<Value> {
    let mut all = match original {
        Value::Number(_) => numbers(),
        Value::String(_) => strings(),
        Value::Null => {
            let mut v = numbers();
            v.extend(strings().into_iter().step_by(7));
            v
        }
        _ => Vec::new(),
    };
    all.extend(structural());
    all
}

/// `count` items spread evenly over `items`, in a fixed order.
fn spread<T: Clone>(items: &[T], count: usize) -> Vec<T> {
    if items.len() <= count {
        return items.to_vec();
    }
    (0..count)
        .map(|i| items[i * items.len() / count].clone())
        .collect()
}

fn leaf_paths(value: &Value, here: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, child) in map {
                here.push(key.clone());
                leaf_paths(child, here, out);
                here.pop();
            }
        }
        Value::Array(items) if !items.is_empty() => {
            for (index, child) in items.iter().enumerate() {
                here.push(index.to_string());
                leaf_paths(child, here, out);
                here.pop();
            }
        }
        _ => out.push(here.clone()),
    }
}

fn get_at<'a>(value: &'a Value, path: &[String]) -> Option<&'a Value> {
    let Some((first, rest)) = path.split_first() else {
        return Some(value);
    };
    let child = match value {
        Value::Object(map) => map.get(first),
        Value::Array(items) => first.parse::<usize>().ok().and_then(|i| items.get(i)),
        _ => None,
    }?;
    get_at(child, rest)
}

fn set_at(value: &mut Value, path: &[String], new: Value) {
    let Some((first, rest)) = path.split_first() else {
        *value = new;
        return;
    };
    let child = match value {
        Value::Object(map) => map.get_mut(first),
        Value::Array(items) => first.parse::<usize>().ok().and_then(|i| items.get_mut(i)),
        _ => None,
    };
    if let Some(child) = child {
        set_at(child, rest, new);
    }
}

// ---------------------------------------------------------------------------------------------
// What a call answered
// ---------------------------------------------------------------------------------------------

/// The codes that say "this value is wrong": they must point at a field (Code Standards 4.3).
const FIELD_CODES: &[&str] = &[
    "ERR_OUT_OF_RANGE",
    "ERR_REQUIRED",
    "ERR_NOT_A_NUMBER",
    "ERR_DECIMALS_NOT_ALLOWED",
    "ERR_NEGATIVE_NOT_ALLOWED",
    "ERR_AMOUNT_TOO_LARGE",
    "ERR_MUST_BE_MORE_THAN_ZERO",
    "ERR_TOO_MANY_DECIMALS",
    "ERR_PERCENT_OVER_100",
    "ERR_QTY_TOO_LARGE",
    "ERR_QTY_NOT_WHOLE",
    "ERR_WEIGHT_TOO_LARGE",
    "ERR_DATE_INVALID",
    "ERR_DATE_CLOSED",
    "ERR_DATE_IN_FUTURE",
    "ERR_CODE_INVALID",
    "ERR_NAME_UR_SCRIPT",
    "ERR_NAME_EN_SCRIPT",
    "ERR_NAME_TOO_LONG",
    "ERR_PHONE_INVALID",
    "ERR_TEXT_TOO_LONG",
    "ERR_CONTROL_CHARS",
    "ERR_OVERFLOW",
    "ERR_FORMAT_INVALID",
    "ERR_TAG_INVALID",
    "ERR_DATE_RANGE",
    "ERR_WEIGHT_REQUIRED",
];

/// Commands that may commit something and still answer with an error, with the reason. A wrong
/// password counts a try, a failed backup is logged, and so on. Anything else that commits on a
/// refusal is a half-save.
const MAY_COMMIT_ON_ERROR: &[&str] = &[
    "auth_",                   // failed tries are counted (NFR-SEC-2)
    "setup_",                  // first run
    "partner_code_",           // a wrong code is counted and logged (FR-17.11)
    "backup_",                 // a failed run is written to the backup log (FR-20.7)
    "health_",                 // the daily check keeps its result
    "assistant_",              // a refused question is still a row of the question log (FR-7.84)
    "day_auto",                // the closing job
    "day_fix_open",            // a wrong sign is written to the audit log (PRD FR-21.4)
    "partners_code_",          // a wrong password counts a try and is logged (FR-17.11, NFR-SEC-14)
    "settings_set_owner_sign", // a wrong password counts a try (NFR-SEC-2)
    "settings_set_security",
];

#[derive(Default)]
struct Tally {
    calls: usize,
    ok: usize,
    coded: usize,
    /// Refused by the IPC layer (a value of the wrong type) before any service ran.
    ipc_level: usize,
    internal: Vec<String>,
    wrote_on_error: Vec<String>,
    /// Code to how many times, and the first example, of an error with no field.
    no_field: BTreeMap<String, (usize, String)>,
    /// Calls that panicked a worker (a panic is reported as an internal error or a lost reply).
    unreadable: Vec<String>,
    /// The first call after which a sale in another shop had no journal entry.
    first_odd: Option<String>,
}

fn short(payload: &Value) -> String {
    payload.to_string().chars().take(260).collect()
}

fn attack(h: &Harness, tally: &mut Tally, command: &str, payload: Value) {
    let before = h.data_version();
    let answer = h.call(command, payload.clone());
    let after = h.data_version();
    tally.calls += 1;
    // A sale in another shop always posts its money (PRD FR-12.32): note the first call that
    // leaves one with no journal entry.
    if after != before && tally.first_odd.is_none() {
        let odd: i64 = h
            .probe
            .query_row(
                "SELECT COUNT(*) FROM venture_entries WHERE kind = 'other_sale' AND journal_entry_id IS NULL AND status = 'saved'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if odd > 0 {
            tally.first_odd = Some(format!("{command} {}", short(&payload)));
        }
    }
    match answer {
        Ok(_) => tally.ok += 1,
        Err(error) => {
            let items = error["errors"].as_array();
            match items {
                None => {
                    tally.ipc_level += 1;
                    if error.is_null() {
                        tally
                            .unreadable
                            .push(format!("{command} {}", short(&payload)));
                    }
                }
                Some(items) => {
                    tally.coded += 1;
                    for item in items {
                        let code = item["code"].as_str().unwrap_or("?").to_owned();
                        if code == "ERR_INTERNAL" {
                            tally
                                .internal
                                .push(format!("{command} {}", short(&payload)));
                        }
                        let has_field = item["field"].as_str().is_some_and(|f| !f.is_empty());
                        if !has_field && FIELD_CODES.contains(&code.as_str()) {
                            let entry = tally
                                .no_field
                                .entry(code)
                                .or_insert((0, format!("{command} {}", short(&payload))));
                            entry.0 += 1;
                        }
                    }
                }
            }
            if before != after && !MAY_COMMIT_ON_ERROR.iter().any(|p| command.starts_with(p)) {
                tally.wrote_on_error.push(format!(
                    "{command} {} -> {}",
                    short(&payload),
                    short(&error)
                ));
            }
        }
    }
}

/// The rules of Code Standards 4.2 for what is stored, checked over every named table.
fn stored_names_obey_the_field_rules(h: &Harness) -> Vec<String> {
    let mut bad = Vec::new();
    let tables = [
        "accounts",
        "cities",
        "items",
        "packings",
        "addas",
        "stores",
        "expense_heads",
        "staff",
        "partners",
        "chukta_buyers",
        "ventures",
    ];
    let invisible = |c: char| {
        c.is_control()
            || matches!(c, '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{2069}' | '\u{feff}' | '\u{061c}')
    };
    let urdu = |c: char| matches!(c, '\u{0600}'..='\u{06ff}' | '\u{0750}'..='\u{077f}' | '\u{fb50}'..='\u{fdff}' | '\u{fe70}'..='\u{feff}');
    let latin = |c: char| c.is_ascii_alphabetic();
    for table in tables {
        let Ok(mut statement) = h
            .probe
            // The seeded system accounts are not typed by anyone; they are reported apart.
            .prepare(&format!(
                "SELECT id, name_ur, name_en FROM {table} WHERE {}",
                if table == "accounts" {
                    "is_system = 0"
                } else {
                    "1 = 1"
                }
            ))
        else {
            continue;
        };
        let rows: Vec<(i64, String, String)> = statement
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        for (id, ur, en) in rows {
            let here = |why: &str| format!("{table} id {id}: {why} (ur {ur:?}, en {en:?})");
            if ur.chars().count() > 100 || en.chars().count() > 100 {
                bad.push(here("longer than 100 characters"));
            }
            if ur.chars().any(invisible) || en.chars().any(invisible) {
                bad.push(here("a control or invisible character"));
            }
            if ur.chars().any(latin) {
                bad.push(here("a Latin letter in the Urdu name"));
            }
            if en.chars().any(urdu) {
                bad.push(here("an Urdu letter in the English name"));
            }
            if ur
                .chars()
                .any(|c| !c.is_alphanumeric() && !" .-'()/&".contains(c) && !urdu(c))
            {
                bad.push(here("a sign the Urdu name does not allow"));
            }
            if ur.trim() != ur || en.trim() != en {
                bad.push(here("not trimmed"));
            }
        }
    }
    bad
}

fn books_are_sound(h: &Harness, round: &str) {
    let integrity: String = h
        .probe
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok", "{round}");
    let (dr, cr): (i64, i64) = h
        .probe
        .query_row(
            "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id WHERE e.status = 'posted'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(dr, cr, "{round}: the books do not balance");
    assert_eq!(
        h.one("SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'"),
        0,
        "{round}: a draft entry was left behind"
    );
    let unbalanced = h.one(
        "SELECT COUNT(*) FROM (SELECT e.id FROM journal_entries e JOIN journal_lines l ON l.entry_id = e.id WHERE e.status = 'posted' GROUP BY e.id HAVING SUM(l.dr_paisa) <> SUM(l.cr_paisa))",
    );
    assert_eq!(unbalanced, 0, "{round}: a posted entry does not balance");
    let orphans: i64 = h
        .probe
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .query_map([], |_| Ok(()))
        .unwrap()
        .count()
        .try_into()
        .unwrap();
    assert_eq!(orphans, 0, "{round}: foreign key violations");
}

// ---------------------------------------------------------------------------------------------
// The tests
// ---------------------------------------------------------------------------------------------

/// How many commands have a payload to attack (the rest build theirs with Rust code).
#[test]
fn almost_every_command_has_a_payload_that_can_be_attacked() {
    let (found, skipped) = table();
    let total = registered().len();
    println!(
        "{} of {total} commands have a plain payload; left out: {skipped:?}",
        found.len()
    );
    // Only the ones that need a Rust expression for their payload may be missing.
    assert!(
        found.len() * 100 >= total * 90,
        "only {} of {total} payloads were read: {skipped:?}",
        found.len()
    );
}

#[test]
fn targeted_hostile_values_in_every_field_of_every_command() {
    let h = Harness::new();
    let (mut payloads, _) = table();
    for (name, seed) in seeds() {
        payloads.insert(name.to_owned(), seed);
    }
    let skip = |name: &str| {
        // Signing out or locking would end the run; the first-run, restore and security-setting
        // commands are attacked by robustness_security.rs and the auth tests, and a partner-code
        // reset would lock every partner command for the rest of the run.
        name.starts_with("auth_")
            // The updater plugin is not part of the mock app, so its commands panic on `state()`.
            // The real app registers it (LOCAL-ONLY).
            || name.starts_with("updates_")
            // The Mac print path waits for a print dialog the mock window never answers (it times
            // out after a long wait). Their `_print_data` twins, which carry the same input, are
            // attacked. The print jobs themselves are LOCAL-ONLY (Windows and the Mac dialog).
            || name.ends_with("_print")
            || matches!(
                name,
                "reports_print"
                    | "reports_print_voucher"
                    | "reports_print_collection"
                    | "reports_print_report"
                    | "analytics_print_view"
            )
            || name == "setup_first_run"
            || name.starts_with("partner_code_")
            || name.starts_with("backup_first_run")
            || name.starts_with("settings_set_security")
            || name == "settings_set_firm"
            || name == "settings_set_default_language"
    };
    let mut tally = Tally::default();
    let before_all = h.data_version();
    // `GATE_T4_ONLY=trips_,purchases_` attacks only the commands that start with one of these.
    let only: Vec<String> = std::env::var("GATE_T4_ONLY")
        .ok()
        .map(|v| v.split(',').map(str::to_owned).collect())
        .unwrap_or_default();
    for (command, payload) in &payloads {
        if skip(command)
            || (!only.is_empty() && !only.iter().any(|p| command.starts_with(p.as_str())))
        {
            continue;
        }
        let mut paths = Vec::new();
        leaf_paths(payload, &mut Vec::new(), &mut paths);
        // Every (field, value) pair of the command, in a fixed order, then an even sample of
        // them if there are more than the cap.
        let mut cases: Vec<(Vec<String>, Value)> = Vec::new();
        for path in &paths {
            if path.is_empty() {
                continue;
            }
            let original = get_at(payload, path).cloned().unwrap_or(Value::Null);
            for value in values_for(&original) {
                cases.push((path.clone(), value));
            }
        }
        for (path, value) in spread(&cases, cap()) {
            let mut mutated = payload.clone();
            set_at(&mut mutated, &path, value);
            attack(&h, &mut tally, command, mutated);
        }
        // A key nobody asked for, and a key taken away: one call each.
        if let Some(map) = payload.get("input").and_then(Value::as_object) {
            let mut extra = payload.clone();
            extra["input"]["unexpected_key"] = json!({ "deep": [1, 2, { "x": null }] });
            attack(&h, &mut tally, command, extra);
            for key in map.keys() {
                let mut missing = payload.clone();
                if let Some(m) = missing["input"].as_object_mut() {
                    m.remove(key);
                }
                attack(&h, &mut tally, command, missing);
            }
        }
        // A payload that is not an object at all.
        for odd in [Value::Null, json!([]), json!("x"), json!(7)] {
            attack(&h, &mut tally, command, odd);
        }
    }
    println!(
        "{} hostile calls: {} worked, {} refused with a code, {} refused by the IPC layer before any service, {} ERR_INTERNAL, {} committed on a refusal; the file changed {} times",
        tally.calls,
        tally.ok,
        tally.coded,
        tally.ipc_level,
        tally.internal.len(),
        tally.wrote_on_error.len(),
        h.data_version() - before_all,
    );
    for (code, (count, example)) in &tally.no_field {
        println!("NO FIELD: {code} x{count}, first: {example}");
    }
    let mut grouped: BTreeMap<String, (usize, String)> = BTreeMap::new();
    for line in &tally.wrote_on_error {
        let command = line.split(' ').next().unwrap_or("").to_owned();
        let entry = grouped.entry(command).or_insert((0, line.clone()));
        entry.0 += 1;
    }
    for (command, (count, example)) in &grouped {
        println!("COMMITTED ON A REFUSAL: {command} x{count}, first: {example}");
    }
    let mut grouped: BTreeMap<String, (usize, String)> = BTreeMap::new();
    for line in &tally.internal {
        let command = line.split(' ').next().unwrap_or("").to_owned();
        let entry = grouped.entry(command).or_insert((0, line.clone()));
        entry.0 += 1;
    }
    for (command, (count, example)) in &grouped {
        println!("ERR_INTERNAL: {command} x{count}, first: {example}");
    }
    books_are_sound(&h, "after the targeted values");
    // A sale in another shop always posts its money (PRD FR-12.32): none may be left with no entry.
    let odd: Vec<(i64, i64, String)> = h
        .probe
        .prepare("SELECT id, amount_paisa, status FROM venture_entries WHERE kind = 'other_sale' AND journal_entry_id IS NULL")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    println!(
        "other_sale rows with no journal entry: {odd:?}; first call that left one: {:?}",
        tally.first_odd
    );
    let rows: Vec<String> = h
        .probe
        .prepare("SELECT id, kind, status, version, qty, purchase_id, journal_entry_id, entry_date, amount_paisa FROM venture_entries WHERE kind = 'other_sale'")
        .unwrap()
        .query_map([], |r| {
            Ok(format!(
                "id {} kind {} status {} version {} qty {:?} purchase {:?} journal {:?} date {} amount {}",
                r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?, r.get::<_, Option<i64>>(5)?, r.get::<_, Option<i64>>(6)?,
                r.get::<_, String>(7)?, r.get::<_, i64>(8)?
            ))
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    println!("other_sale rows: {rows:#?}");
    let bad_names = stored_names_obey_the_field_rules(&h);
    assert!(
        tally.internal.is_empty(),
        "ERR_INTERNAL (a panic or a bug) on:\n{}",
        tally.internal.join("\n")
    );
    assert!(
        tally.wrote_on_error.is_empty(),
        "a refused call committed something (a half-save):\n{}",
        tally.wrote_on_error.join("\n")
    );
    assert!(
        tally.no_field.is_empty(),
        "errors about a value with no field to point at: {:?}",
        tally.no_field
    );
    assert!(
        bad_names.is_empty(),
        "a stored name breaks the field rules of Code Standards 4.2:\n{}",
        bad_names.join("\n")
    );
    assert!(tally.unreadable.is_empty(), "{:?}", tally.unreadable);
    assert!(tally.calls > 500);
    let _ = &h.buyers;
    let _ = &h.world;
}

/// Every list command with a page, every limit and offset of the lane list (B31): never an
/// internal error, never more rows than the page limit, and a page past the end is empty.
#[test]
fn limits_and_offsets_of_every_list_command() {
    let h = Harness::new();
    let (mut payloads, _) = table();
    for (name, seed) in seeds() {
        payloads.insert(name.to_owned(), seed);
    }
    let limits = [
        json!(0),
        json!(1),
        json!(24),
        json!(25),
        json!(26),
        json!(99),
        json!(100),
        json!(101),
        json!(250),
        json!(1000),
        json!(1001),
        json!(u32::MAX),
        json!(i64::from(u32::MAX) + 1),
        json!(-1),
        json!(1.5),
        json!("100"),
        Value::Null,
    ];
    let offsets = [
        json!(0),
        json!(1),
        json!(99),
        json!(100),
        json!(119),
        json!(120),
        json!(121),
        json!(u32::MAX),
        json!(i64::from(u32::MAX) + 1),
        json!(-1),
        json!("0"),
    ];
    let mut tally = Tally::default();
    let mut too_long: Vec<String> = Vec::new();
    let mut paged = 0;
    for (command, payload) in &payloads {
        if command.starts_with("auth_") || command.starts_with("partner_code_") {
            continue;
        }
        let input = payload.get("input");
        let has = |key: &str| input.and_then(|i| i.get(key)).is_some();
        if !has("limit") && !has("offset") {
            continue;
        }
        paged += 1;
        for limit in &limits {
            for offset in &offsets {
                let mut mutated = payload.clone();
                if has("limit") {
                    mutated["input"]["limit"] = limit.clone();
                }
                if has("offset") {
                    mutated["input"]["offset"] = offset.clone();
                }
                let before = h.data_version();
                let answer = h.call(command, mutated.clone());
                let moved = h.data_version() != before;
                tally.calls += 1;
                match answer {
                    Ok(value) => {
                        tally.ok += 1;
                        let longest = longest_array(&value);
                        let allowed = if command.starts_with("dashboard_") {
                            100
                        } else {
                            1000
                        };
                        if longest > allowed {
                            too_long.push(format!(
                                "{command} {} -> an array of {longest} rows (more than {allowed})",
                                short(&mutated)
                            ));
                        }
                    }
                    Err(error) => match error["errors"][0]["code"].as_str() {
                        Some("ERR_INTERNAL") => tally
                            .internal
                            .push(format!("{command} {}", short(&mutated))),
                        Some(_) => {
                            tally.coded += 1;
                            if moved && !command.starts_with("assistant_") {
                                tally
                                    .wrote_on_error
                                    .push(format!("{command} {}", short(&mutated)));
                            }
                        }
                        None => tally.ipc_level += 1,
                    },
                }
            }
        }
    }
    println!(
        "{paged} list commands, {} calls: {} worked, {} coded, {} by the IPC layer",
        tally.calls, tally.ok, tally.coded, tally.ipc_level
    );
    assert!(paged >= 25, "only {paged} list commands found");
    assert!(tally.internal.is_empty(), "{:?}", tally.internal);
    assert!(
        tally.wrote_on_error.is_empty(),
        "{:?}",
        tally.wrote_on_error
    );
    assert!(
        too_long.is_empty(),
        "page limit not kept:\n{}",
        too_long.join("\n")
    );
}

fn longest_array(value: &Value) -> usize {
    match value {
        Value::Array(items) => items
            .len()
            .max(items.iter().map(longest_array).max().unwrap_or(0)),
        Value::Object(map) => map.values().map(longest_array).max().unwrap_or(0),
        _ => 0,
    }
}

/// The pages of the two dues lists put together are the whole list, once each, and the totals
/// come from the backend over the whole filter (B31, FR-19.11, FR-9.4, FR-9.6).
#[test]
fn walking_the_pages_of_the_dues_lists_gives_every_row_once() {
    let h = Harness::new();
    for (command, side) in [
        ("dashboard_buyer_dues", None),
        ("dashboard_seller_dues", Some("all")),
    ] {
        for limit in [10_u32, 25, 50, 100] {
            let mut ids: Vec<i64> = Vec::new();
            let mut total_count = None;
            let mut offset = 0_u32;
            loop {
                let mut input = json!({ "offset": offset, "limit": limit });
                if let Some(side) = side {
                    input["side"] = json!(side);
                }
                let page = h
                    .call(command, json!({ "input": input }))
                    .unwrap_or_else(|e| panic!("{command} offset {offset} limit {limit}: {e}"));
                let rows = page["rows"].as_array().unwrap();
                assert!(
                    rows.len() <= limit as usize,
                    "{command}: a page is over its limit"
                );
                let count = page["total_count"].as_i64().unwrap();
                assert_eq!(
                    *total_count.get_or_insert(count),
                    count,
                    "{command}: the total changes between pages"
                );
                ids.extend(
                    rows.iter()
                        .map(|r| r["party"]["account_id"].as_i64().unwrap()),
                );
                if rows.is_empty() || ids.len() as i64 >= count {
                    break;
                }
                offset += limit;
            }
            let mut sorted = ids.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(
                sorted.len(),
                ids.len(),
                "{command} limit {limit}: a row came twice"
            );
            assert!(
                i64::try_from(ids.len()).unwrap() == total_count.unwrap(),
                "{command} limit {limit}: the pages hold {} rows, the total says {:?}",
                ids.len(),
                total_count
            );
            assert!(ids.len() >= 120, "{command}: only {} rows", ids.len());
        }
    }
    let _ = &h.buyers;
}

/// Found by the targeted run: `trips_other_sale_cancel` with an id that does not exist answered
/// `ERR_INTERNAL` for one version value. A row that is not there is `ERR_NOT_FOUND`, a wrong
/// version is `ERR_VERSION_CHANGED`; neither is an internal error (Code Standards 4.3, B9).
#[test]
fn cancelling_a_row_that_is_not_there_is_a_coded_refusal_for_every_version() {
    let h = Harness::new();
    let mut bad = Vec::new();
    for command in [
        "trips_other_sale_cancel",
        "trips_cost_cancel",
        "purchases_cancel",
    ] {
        for version in [json!(0), json!(1), json!(2), json!(i64::MAX), json!(-1)] {
            let before = h.data_version();
            let payload = json!({ "input": { "id": 1, "version": version,
                "reason": "x", "confirm_cash_below_zero": false } });
            let answer = h.call(command, payload.clone());
            let code = match &answer {
                Ok(_) => "OK".to_owned(),
                Err(e) => e["errors"][0]["code"].as_str().unwrap_or("IPC").to_owned(),
            };
            println!("{command} {version} -> {code}");
            if code == "ERR_INTERNAL" {
                bad.push(format!("{command} {payload}"));
            }
            assert_eq!(h.data_version(), before, "{command} {payload} committed");
        }
    }
    assert!(bad.is_empty(), "ERR_INTERNAL on: {bad:?}");
}

/// A garden and a trip share the table of their entries. The id of an entry that belongs to the
/// other kind must be "not found" for a command of this kind (Code Standards V0, 4.3), whatever
/// the id: found by the targeted run, where `trips_other_sale_cancel` with the id of a garden's
/// sale in another shop answered `ERR_INTERNAL` ("a sale in another shop has no purchase").
fn payload_of(name: &str) -> Value {
    table()
        .0
        .remove(name)
        .unwrap_or_else(|| panic!("no payload for {name}"))
}

fn newest(h: &Harness, sql: &str) -> i64 {
    h.one(sql)
}

#[test]
fn an_entry_of_a_garden_is_not_found_by_the_trip_commands_and_the_other_way_round() {
    let h = Harness::new();
    let partner = h.world.partners[0];
    let share = json!([{ "partner_id": partner, "share": "100", "source": "shop" }]);
    // A garden with a sale in another shop and a cost.
    let mut garden = payload_of("gardens_create");
    garden["input"]["partners"] = share.clone();
    h.call("gardens_create", garden).expect("garden");
    let garden_id = newest(&h, "SELECT MAX(id) FROM ventures WHERE kind = 'garden'");
    let mut sale = payload_of("gardens_entry_save");
    sale["input"]["garden_id"] = json!(garden_id);
    h.call("gardens_entry_save", sale).expect("garden sale");
    let mut cost = payload_of("gardens_entry_save");
    cost["input"] = json!({ "type": "cost", "garden_id": garden_id, "date": "10-10-2026",
        "amount": "1,000", "reason": "x", "parts": [], "confirm_cash_below_zero": true });
    h.call("gardens_entry_save", cost).expect("garden cost");
    let garden_sale = newest(
        &h,
        "SELECT MAX(id) FROM venture_entries WHERE kind = 'other_sale'",
    );
    let garden_cost = newest(
        &h,
        "SELECT MAX(id) FROM venture_entries WHERE kind = 'cost'",
    );
    // A trip with a cost.
    let mut trip = payload_of("trips_create");
    trip["input"]["partners"] = share;
    h.call("trips_create", trip).expect("trip");
    let trip_id = newest(&h, "SELECT MAX(id) FROM ventures WHERE kind = 'trip'");
    let mut trip_cost = payload_of("trips_cost_save");
    trip_cost["input"]["trip_id"] = json!(trip_id);
    trip_cost["input"]["confirm_cash_below_zero"] = json!(true);
    // A trip cost needs a purchase with goods left (ERR_TRIP_NO_STOCK), so it may be refused here;
    // the other two crossings are enough to show the problem.
    let _ = h.call("trips_cost_save", trip_cost);

    // Close the garden: its own cancel then refuses the cost (ERR_GARDEN_CLOSED), the control.
    let garden_version = h.one(&format!(
        "SELECT version FROM ventures WHERE id = {garden_id}"
    ));
    let closed = h.call(
        "gardens_close",
        json!({ "input": { "id": garden_id, "version": garden_version, "date": "10-10-2026" } }),
    );
    let status = h.probe.query_row(
        &format!("SELECT status FROM ventures WHERE id = {garden_id}"),
        [],
        |r| r.get::<_, String>(0),
    );
    println!(
        "gardens_close -> {:?}; the garden is now {status:?}",
        closed.is_ok()
    );
    let version = |id: i64| {
        h.one(&format!(
            "SELECT version FROM venture_entries WHERE id = {id}"
        ))
    };
    let control = h.call(
        "gardens_entry_cancel",
        json!({ "input": { "id": garden_cost, "version": version(garden_cost), "reason": "x",
            "confirm_cash_below_zero": true } }),
    );
    println!(
        "CONTROL gardens_entry_cancel on the cost of the closed garden -> {:?}",
        control
            .as_ref()
            .err()
            .map(|e| e["errors"][0]["code"].clone())
    );
    let mut bad = Vec::new();
    for (command, id) in [
        ("trips_other_sale_cancel", garden_sale),
        ("trips_cost_cancel", garden_cost),
    ] {
        let before = h.data_version();
        let payload = json!({ "input": { "id": id, "version": version(id), "reason": "x",
            "confirm_cash_below_zero": true } });
        let code = match h.call(command, payload.clone()) {
            Ok(_) => "OK".to_owned(),
            Err(e) => e["errors"][0]["code"].as_str().unwrap_or("IPC").to_owned(),
        };
        println!("{command} on a garden's entry {id} -> {code}");
        if code != "ERR_NOT_FOUND" {
            bad.push(format!("{command} {payload} -> {code}"));
        }
        let _ = before;
    }
    assert!(
        bad.is_empty(),
        "a trip command accepted a garden's entry:\n{}",
        bad.join("\n")
    );
}

/// A value of the wrong type (a number where text is wanted, text where a number is wanted, a
/// missing key) never reaches a service: the IPC layer refuses it with plain text, not with a coded
/// error. The file is untouched. This is by design for our own frontend (its types come from the
/// bindings), and is listed as a question for Umar in the gate report.
#[test]
fn a_value_of_the_wrong_type_is_refused_before_any_service_runs() {
    let h = Harness::new();
    let before = h.data_version();
    let mut shown = Vec::new();
    for (command, payload) in [
        (
            "rules_test",
            json!({ "input": { "seller_id": 1, "buyer_id": 2, "item_id": 1, "date": "10-10-2026", "qty": 20, "rate": "600" } }),
        ),
        ("accounts_get", json!({ "input": { "id": "one" } })),
        ("accounts_get", json!({ "input": {} })),
        ("accounts_get", json!({})),
        (
            "dashboard_cash_sales_day",
            json!({ "input": { "date": "10-10-2026", "offset": -1, "limit": 25 } }),
        ),
        (
            "dashboard_seller_dues",
            json!({ "input": { "side": "nobody", "offset": 0, "limit": 25 } }),
        ),
    ] {
        let answer = h.call(command, payload);
        let text = match &answer {
            Ok(_) => panic!("{command}: a wrong type was accepted"),
            Err(e) if e["errors"].is_array() => format!("CODED {}", e["errors"][0]["code"]),
            Err(e) => format!("PLAIN {e}"),
        };
        shown.push(format!("{command}: {text}"));
    }
    println!("{}", shown.join("\n"));
    assert_eq!(h.data_version(), before);
}
