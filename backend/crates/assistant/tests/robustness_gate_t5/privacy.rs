//! Privacy attacks (PRD FR-7.83, Code Standards Q7, NFR-AI-4, E2E-54, E97; finding R6-1).
//!
//! 1. The two cases of R6-1 through the app's own saves: a trip purchase split to another shop and
//!    a garden arrival, each with a typed shop name.
//! 2. A sweep: typed free text is put in EVERY free-text column of the rich year of the books
//!    (see [`FREE_TEXT`]), then every report, every view, the tiles, the party card, every `find`
//!    role and every `app_state` read is run with the tools' own functions, and no request text
//!    may hold a marker. The sweep names each column that leaks.
//! 3. The questions the sweep leaves: a `find` by phone, father or firm name, address or mark.
#![allow(dead_code)]

use std::collections::BTreeMap;

use mandi_assistant::ask::AskOutcome;
use mandi_services::core::auth::Language;
use mandi_services::core::partner_code::{PartnerCodeSetInput, PartnerCodeUnlockInput};
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::partner_code;
use serde_json::{Value, json};

use crate::harness::Harness;
use crate::kit::*;
use crate::{common, seed, world};

/// Every free-text column of the schema: what a person types by hand, one by one, read from
/// `sqlite_master` and the migrations (0001 to 0025). Names, codes, tags, dates, kinds, keys and
/// JSON work columns are not in the list: names are sent on purpose and the others are not text
/// a person types. FR-7.83 lists what never goes: phones, addresses, father or firm names, notes,
/// reasons, narrations typed by hand, driver names, supplier and shop names, the market and the
/// person sent of a trip, مارکہ text.
pub const FREE_TEXT: &[(&str, &str)] = &[
    ("accounts", "address"),
    ("accounts", "father_or_firm_en"),
    ("accounts", "father_or_firm_ur"),
    ("accounts", "note"),
    ("accounts", "phone1"),
    ("accounts", "phone2"),
    ("bank_accounts", "bank_name"),
    ("bank_accounts", "title"),
    ("bank_accounts", "number"),
    ("bikri_entries", "cancel_reason"),
    ("bikri_entries", "driver_name"),
    ("bikri_entries", "driver_phone"),
    ("bikri_entries", "note"),
    ("bikri_lots", "chukta_phone"),
    ("bikri_sales", "chukta_phone"),
    ("bikri_sales", "note"),
    ("charge_rules", "note"),
    ("chukta_buyers", "phone"),
    ("dead_marks", "note"),
    ("dead_marks", "undo_reason"),
    ("fruit_takes", "cancel_reason"),
    ("fruit_takes", "note"),
    ("garden_arrivals", "cancel_reason"),
    ("garden_arrivals", "note"),
    ("garden_arrivals", "shop_name"),
    ("journal_entries", "narration"),
    ("journal_lines", "narration"),
    ("markas", "mark_text"),
    ("narrations", "text_ur"),
    ("narrations", "text_en"),
    ("purchases", "cancel_reason"),
    ("purchases", "note"),
    ("purchases", "shop_name"),
    ("purchases", "supplier_text"),
    ("staff", "phone"),
    ("stock_moves", "reason"),
    ("venture_entries", "cancel_reason"),
    ("venture_entries", "reason"),
    ("venture_entries", "shop_name"),
    ("ventures", "market"),
    ("ventures", "note"),
    ("ventures", "sent_person"),
    ("vouchers", "cancel_reason"),
    ("vouchers", "cheque_no"),
    ("vouchers", "expected_goods"),
    ("vouchers", "narration"),
    ("vouchers", "reason"),
    ("vouchers", "taken_by"),
    ("audit_log", "reason"),
    ("day_closes", "reason"),
];

/// Columns FR-7.83 does not name: a truck number typed on an old بکری. Marked too, and reported
/// as a question, not as a finding.
pub const NOT_LISTED: &[(&str, &str)] = &[("bikri_entries", "truck_no")];

/// The marker of a column: letters and underscores only, so no escaping or normalising hides it.
pub fn marker(table: &str, column: &str) -> String {
    format!("SECRETX_{table}_{column}")
}

/// Puts the marker in every row of every free-text column, as if a person had typed it in each
/// field. The append-only triggers are dropped first (the copy is a test database, the books are
/// only read afterwards). Gives, for each column, the number of rows that now hold it.
pub fn mark_all(dir: &tempfile::TempDir, split: bool) -> BTreeMap<String, i64> {
    let conn = common::raw(dir);
    let triggers: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    for t in triggers {
        conn.execute_batch(&format!("DROP TRIGGER IF EXISTS \"{t}\""))
            .unwrap();
    }
    // A shop name is shown only for goods that are split to another shop: split one نگ of every
    // purchase and arrival, so the stock and trip reports have a line that carries the name.
    for table in ["purchases", "garden_arrivals"]
        .into_iter()
        .filter(|_| split)
    {
        conn.execute(
            &format!("UPDATE OR IGNORE \"{table}\" SET other_qty = 1 WHERE qty >= 2"),
            [],
        )
        .unwrap();
    }
    // The audit log keeps the old and new values of a change as JSON. Put typed text under the
    // keys a form writes, beside a number, a flag and an amount.
    let json = r#"{"note":"SECRETX_audit_note","phone1":"SECRETX_audit_phone","reason":"SECRETX_audit_reason","narration":"SECRETX_audit_narration","name_en":"SECRETX_audit_name","amount_paisa":12300,"qty":5,"flag":true,"nested":{"address":"SECRETX_audit_nested"},"list":["SECRETX_audit_list"]}"#;
    // The old values differ from the new ones, so each key is a change the read can report.
    let old = json
        .replace("SECRETX_audit_", "SECRETX_audit_old_")
        .replace("12300", "100");
    conn.execute(
        "UPDATE OR IGNORE audit_log SET old_json = ?1, new_json = ?2, action = 'edit'",
        [old.as_str(), json],
    )
    .unwrap();
    let mut marked = BTreeMap::new();
    for (table, column) in FREE_TEXT.iter().chain(NOT_LISTED.iter()) {
        let m = marker(table, column);
        // OR IGNORE: a column with a CHECK that a marker cannot meet is skipped, not an error.
        let sql = format!("UPDATE OR IGNORE \"{table}\" SET \"{column}\" = '{m}'");
        let n = conn.execute(&sql, []).unwrap_or(0);
        marked.insert(format!("{table}.{column}"), i64::try_from(n).unwrap_or(0));
    }
    marked
}

/// Markers put inside the JSON of the audit log (old and new values), under the keys a form
/// writes: the audit read keeps only money, numbers and flags.
pub const AUDIT_JSON_MARKERS: &[&str] = &[
    "SECRETX_audit_note",
    "SECRETX_audit_phone",
    "SECRETX_audit_reason",
    "SECRETX_audit_narration",
    "SECRETX_audit_nested",
    "SECRETX_audit_list",
    "SECRETX_audit_name",
];

/// The markers that a text holds, by column (case does not matter).
pub fn leaks(text: &str) -> Vec<String> {
    let low = text.to_lowercase();
    let mut out: Vec<String> = FREE_TEXT
        .iter()
        .chain(NOT_LISTED.iter())
        .filter(|(t, c)| low.contains(&marker(t, c).to_lowercase()))
        .map(|(t, c)| format!("{t}.{c}"))
        .collect();
    out.extend(
        AUDIT_JSON_MARKERS
            .iter()
            .filter(|m| low.contains(&m.to_lowercase()))
            .map(|m| format!("audit_log.json:{m}")),
    );
    if low.contains("secretx_audit_old_") {
        out.push("audit_log.json:old values".to_owned());
    }
    out
}

// ---- 1. R6-1 through the app's own saves ---------------------------------------------------

const SHOP: &str = "SECRETXSHOPNAME";

fn partners_of(w: &world::World) -> Vec<VenturePartnerInput> {
    let [a, b, c] = w.partners;
    [(a, "40"), (b, "30"), (c, "30")]
        .into_iter()
        .map(|(partner_id, share)| VenturePartnerInput {
            partner_id,
            share: share.into(),
            source: MoneySource::Shop,
        })
        .collect()
}

/// The worked day, a trip with a purchase of 20 پیٹی of which 8 go to another shop (named by hand),
/// and a garden with a goods-arrived entry of 50 with 20 for another shop (named by hand), plus a
/// sale in another shop of each, typed with a shop name.
fn other_shop_day() -> Harness {
    use mandi_services::core::arrivals::{ArrivalEntryInput, ArrivalLineInput};
    use mandi_services::core::gardens::{GardenEntryInput, GardenInput, GardenOtherSaleInput};
    use mandi_services::core::purchases::PurchaseInput;
    use mandi_services::core::trips::OtherSaleInput;
    let h = Harness::day();
    let w = &h.world;
    let (ctx, owner) = (&w.env.ctx, &w.owner);
    let trip = mandi_services::trips::create(
        ctx,
        owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: partners_of(w),
        },
    )
    .unwrap();
    let purchase = mandi_services::purchases::save(
        ctx,
        owner,
        &PurchaseInput {
            other_qty: Some("8".into()),
            shop_name: Some(SHOP.into()),
            sell_date: None,
            trip_id: Some(trip.id),
            date: "10-10-2026".into(),
            supplier: None,
            item_id: Some(w.banana),
            packing_id: Some(w.box_),
            qty: "20".into(),
            weight: None,
            rate: "500".into(),
            freight: None,
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    mandi_services::trips::other_sale_save(
        ctx,
        owner,
        &OtherSaleInput {
            trip_id: Some(trip.id),
            date: "10-10-2026".into(),
            shop: Some("SECRETXTRIPSALESHOP".into()),
            purchase_id: Some(purchase.id),
            qty: "4".into(),
            amount: "3000".into(),
            note: None,
        },
    )
    .unwrap();
    let garden = mandi_services::gardens::create(
        ctx,
        owner,
        &GardenInput {
            name_ur: "شاہ پور باغ".into(),
            name_en: "Shahpur Garden".into(),
            confirmed_rules_words: vec![
                "شاہ".into(),
                "پور".into(),
                "باغ".into(),
                "Shahpur".into(),
                "Garden".into(),
            ],
            date: "10-10-2026".into(),
            item_id: Some(w.banana),
            item_ids: Vec::new(),
            rent: "50,000".into(),
            season_end: None,
            note: None,
            partners: partners_of(w),
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .row;
    mandi_services::gardens::arrivals::entry_save(
        ctx,
        owner,
        &ArrivalEntryInput {
            garden_id: Some(garden.id),
            date: "10-10-2026".into(),
            sell_date: Some("12-10-2026".into()),
            lines: vec![ArrivalLineInput {
                key: "l1".into(),
                item_id: Some(w.banana),
                packing_id: Some(w.box_),
                qty: "50".into(),
                weight: None,
                note: None,
                other_qty: Some("20".into()),
                shop_name: Some("SECRETXGARDENSHOP".into()),
            }],
        },
    )
    .unwrap();
    mandi_services::gardens::entry_save(
        ctx,
        owner,
        &GardenEntryInput::OtherSale(GardenOtherSaleInput {
            garden_id: Some(garden.id),
            date: "10-10-2026".into(),
            shop: Some("SECRETXGARDENSALESHOP".into()),
            arrival_id: None,
            qty: "5".into(),
            amount: "2500".into(),
            note: None,
        }),
    )
    .ok();
    h
}

/// One question that reads the stock report, the trip and garden reports and the views of trips
/// and gardens, then gives up. Gives the requests.
fn ask_the_other_shop_reads(h: &Harness) -> Vec<Value> {
    let reads = vec![
        tool_reply(&[
            ("report", report_args("R10", json!({}))),
            ("report", report_args("R33", json!({}))),
            ("report", report_args("R36", json!({}))),
            ("analytics", analytics_args("I", json!({}))),
            ("analytics", analytics_args("J", json!({}))),
            ("tiles", json!({})),
        ]),
        final_reply(&answer("not_understood", "")),
    ];
    let (reply, requests) = ask(h, "baqi mal kahan hai", None, reads);
    assert_eq!(reply.outcome, AskOutcome::NotUnderstood, "{reply:#?}");
    let errors = tool_errors(&requests[1]);
    assert!(
        errors.iter().filter(|e| e.is_none()).count() >= 5,
        "the reads must have run: {errors:?}"
    );
    requests
}

/// Which of the four typed shop names reach the model: the shop typed on a purchase split to
/// another shop, on a goods-arrived line, on a trip's sale in another shop and on a garden's.
fn other_shop_leaks() -> Vec<&'static str> {
    let h = other_shop_day();
    let requests = ask_the_other_shop_reads(&h);
    let sent = all_text(&requests);
    // The control: the books do hold the names, so the reports can reach them.
    let conn = common::raw(&h.world.dir);
    let one = |sql: &str| -> String { conn.query_row(sql, [], |r| r.get(0)).unwrap() };
    assert_eq!(
        one("SELECT shop_name FROM purchases WHERE shop_name IS NOT NULL"),
        SHOP
    );
    assert_eq!(
        one("SELECT shop_name FROM garden_arrivals WHERE shop_name IS NOT NULL"),
        "SECRETXGARDENSHOP"
    );
    assert_eq!(
        one("SELECT shop_name FROM venture_entries WHERE shop_name = 'SECRETXTRIPSALESHOP'"),
        "SECRETXTRIPSALESHOP"
    );
    h.scan_requests();
    [
        "SECRETXSHOPNAME",
        "SECRETXGARDENSHOP",
        "SECRETXTRIPSALESHOP",
        "SECRETXGARDENSALESHOP",
    ]
    .into_iter()
    .filter(|s| sent.contains(s))
    .collect()
}

/// The shop the Owner typed for the part of a trip purchase that goes to another shop, and for a
/// goods-arrived line, must not reach the model (FR-7.83: "the supplier and shop names typed on
/// trip, garden and پھل entries"). Finding R6-1: `is_left_out` knows the column `shop`, not
/// `shop_name`, so R10 sends it.
#[test]
fn t5_r6_1_the_shop_typed_on_a_purchase_or_an_arrival_line_is_never_sent() {
    let leaked = other_shop_leaks();
    let purchase_or_arrival: Vec<_> = leaked
        .iter()
        .filter(|s| ["SECRETXSHOPNAME", "SECRETXGARDENSHOP"].contains(s))
        .collect();
    assert!(purchase_or_arrival.is_empty(), "{leaked:?}");
}

/// The shop typed on a sale in another shop (a trip's or a garden's) is the same kind of text.
#[test]
fn t5_the_shop_typed_on_a_sale_in_another_shop_is_never_sent() {
    let leaked = other_shop_leaks();
    let sales: Vec<_> = leaked
        .iter()
        .filter(|s| ["SECRETXTRIPSALESHOP", "SECRETXGARDENSALESHOP"].contains(s))
        .collect();
    assert!(sales.is_empty(), "{leaked:?}");
}

/// The same reads, through the stock report with the shop typed on both a purchase and an arrival,
/// must at least keep the other private text out: the control case that passes today, so a fix of
/// R6-1 cannot break the rest.
#[test]
fn t5_the_other_shop_reads_keep_the_rest_of_the_books_out_of_the_request() {
    let h = other_shop_day();
    let requests = ask_the_other_shop_reads(&h);
    let sent = all_text(&requests);
    for secret in ["Lahore", "SECRETXTRIPNOTE"] {
        // "Lahore" is the market typed on the trip (FR-7.83: the market of a trip never goes).
        assert!(!sent.contains(secret), "a request holds {secret:?}");
    }
    h.scan_requests();
}

// ---- 2. the sweep -------------------------------------------------------------------------

/// The rich year with the marker in every free-text column.
fn marked_year() -> (seed::Year, BTreeMap<String, i64>) {
    let y = seed::build();
    let marked = mark_all(&y.w.dir, true);
    (y, marked)
}

/// Every read the model can make, as calls of the tools. The parties come from `find`, so the
/// calls name them the way the model does (`r3.matches.0`).
fn read_everything(r: &mut Reader<'_>) {
    r.call("tiles", json!({}));
    // The first match of every role of `find`; the lists come back with "all".
    let roles = [
        "buyer",
        "seller",
        "any_party",
        "chukta",
        "money_lent",
        "separate_khata",
        "partner",
        "garden",
        "trip",
        "item",
        "city",
        "adda",
        "collector",
        "staff",
        "expense_head",
        "truck",
        "packing",
        "marka",
        "store",
        "narration",
    ];
    let mut paths: BTreeMap<&str, String> = BTreeMap::new();
    for role in roles {
        for text in ["all", "a", "e", "i"] {
            if let Some(p) = r.first_match(text, role) {
                paths.entry(role).or_insert(p);
            }
        }
    }
    let party_paths: Vec<String> = [
        "buyer",
        "seller",
        "chukta",
        "money_lent",
        "separate_khata",
        "garden",
        "partner",
        "staff",
        "any_party",
    ]
    .iter()
    .filter_map(|k| paths.get(k).cloned())
    .collect();
    for p in &party_paths {
        r.call("party_card", json!({"party": p}));
    }
    let ids: Vec<String> = mandi_reports::reports::catalog()
        .iter()
        .map(|e| {
            serde_json::to_value(e.id)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    let today = "2026-11-05";
    let range = json!({"from": "2016-01-01", "to": today});
    let one_day = json!({"from": today, "to": today});
    let as_of = json!({"as_of": today});
    let get = |k: &str| paths.get(k).cloned();
    for id in &ids {
        r.call("report", report_args(id, json!({})));
        r.call("report", report_args(id, range.clone()));
        r.call("report", report_args(id, one_day.clone()));
        r.call("report", report_args(id, as_of.clone()));
        r.call(
            "report",
            report_args(
                id,
                json!({"from": "2016-01-01", "to": today, "group_by": "month"}),
            ),
        );
        r.call("report", report_args(id, json!({"bikri": 1})));
        r.call(
            "report",
            report_args(id, json!({"bikri": 2, "from": "2016-01-01", "to": today})),
        );
        r.call(
            "report",
            report_args(
                id,
                json!({"status": "due", "from": "2016-01-01", "to": today}),
            ),
        );
        r.call(
            "report",
            report_args(
                id,
                json!({"status": "paid", "from": "2016-01-01", "to": today}),
            ),
        );
        for p in &party_paths {
            r.call(
                "report",
                report_args(id, json!({"party": p, "from": "2016-01-01", "to": today})),
            );
            r.call(
                "report",
                report_args(id, json!({"party": p, "as_of": today})),
            );
        }
        for (key, role) in [
            ("partner", "partner"),
            ("trip", "trip"),
            ("garden", "garden"),
            ("collector", "collector"),
            ("item", "item"),
            ("city", "city"),
            ("adda", "adda"),
            ("seller", "seller"),
        ] {
            if let Some(p) = get(role) {
                r.call(
                    "report",
                    report_args(id, json!({key: p, "from": "2016-01-01", "to": today})),
                );
            }
        }
        for layout in [
            "detail", "trip", "item", "day", "month", "partner", "garden",
        ] {
            r.call(
                "report",
                report_args(
                    id,
                    json!({"layout": layout, "from": "2016-01-01", "to": today}),
                ),
            );
        }
    }
    // The vouchers list by type (R15).
    for t in [
        "receipt",
        "payment",
        "advance",
        "shop_expense",
        "personal_expense",
        "personal_return",
        "money_lent",
        "money_lent_return",
        "capital_in",
        "capital_out",
        "khata_out",
        "khata_in",
        "general",
    ] {
        r.call(
            "report",
            report_args(
                "R15",
                json!({"voucher_type": t, "from": "2016-01-01", "to": today}),
            ),
        );
    }
    for view in ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K"] {
        r.call("analytics", analytics_args(view, json!({})));
        r.call(
            "analytics",
            analytics_args(view, json!({"from": "2016-01-01", "to": today})),
        );
        for group in ["day", "month", "year"] {
            r.call(
                "analytics",
                analytics_args(
                    view,
                    json!({"from": "2016-01-01", "to": today, "group_by": group}),
                ),
            );
        }
        for compare in ["previous", "last_year"] {
            r.call(
                "analytics",
                analytics_args(
                    view,
                    json!({"from": "2026-10-01", "to": today, "compare": compare}),
                ),
            );
        }
        for goods in ["all", "consigned", "trip", "garden"] {
            r.call("analytics", analytics_args(view, json!({"goods": goods})));
        }
        for side in ["both", "seller", "buyer"] {
            r.call("analytics", analytics_args(view, json!({"side": side})));
        }
        for (key, role) in [
            ("partner", "partner"),
            ("garden", "garden"),
            ("trip", "trip"),
            ("item", "item"),
            ("seller", "seller"),
            ("buyer", "buyer"),
            ("city", "city"),
            ("adda", "adda"),
        ] {
            if let Some(p) = get(role) {
                r.call("analytics", analytics_args(view, json!({key: p})));
            }
        }
    }
    for what in ["rules", "attention", "days", "audit"] {
        r.call(
            "app_state",
            json!({"what": what, "party": null, "item": null, "bikri": null}),
        );
        for p in &party_paths {
            r.call(
                "app_state",
                json!({"what": what, "party": p, "item": null, "bikri": null}),
            );
        }
        if let Some(p) = get("item") {
            r.call(
                "app_state",
                json!({"what": what, "party": null, "item": p, "bikri": null}),
            );
        }
        r.call(
            "app_state",
            json!({"what": what, "party": null, "item": null, "bikri": 1}),
        );
    }
}

/// Runs the whole sweep for one language and one state of the partner lock, and gives the leaks:
/// each column that was found in a text for the model, with how many texts held it.
fn sweep(w: &world::World, language: Language, locked: bool) -> Sweep {
    with_reader(w, language, locked, |r| {
        read_everything(r);
        let mut found: BTreeMap<String, usize> = BTreeMap::new();
        for text in &r.seen {
            for column in leaks(text) {
                *found.entry(column).or_default() += 1;
            }
        }
        Sweep {
            found,
            ok: r.ok,
            refused: r.refused,
            paused: r.paused,
            by: r.by.clone(),
            codes: r.codes.clone(),
        }
    })
}

/// What one sweep found and how much it read.
struct Sweep {
    found: BTreeMap<String, usize>,
    ok: usize,
    refused: usize,
    paused: usize,
    by: BTreeMap<String, usize>,
    codes: BTreeMap<String, usize>,
}

impl Sweep {
    /// The reports and views that never gave a result: the sweep did not look at them.
    fn never_read(&self) -> Vec<String> {
        let mut missing: Vec<String> = Vec::new();
        for entry in mandi_reports::reports::catalog() {
            let id = serde_json::to_value(entry.id).unwrap();
            let key = format!("report:{}", id.as_str().unwrap());
            if !self.by.contains_key(&key) {
                missing.push(key);
            }
        }
        for v in ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K"] {
            let key = format!("analytics:{v}");
            if !self.by.contains_key(&key) {
                missing.push(key);
            }
        }
        missing
    }
}

/// The partner section opened (the code set and given), as it is while the Owner is on a screen
/// behind the code: the partner reports run and show their lines.
fn open_partners(w: &world::World) {
    let ctx = &w.env.ctx;
    partner_code::set(
        ctx,
        &w.owner,
        &PartnerCodeSetInput {
            password: common::OWNER_PASSWORD.to_owned(),
            code: "4826".to_owned(),
            code_again: "4826".to_owned(),
        },
    )
    .unwrap();
    partner_code::unlock(
        ctx,
        &w.owner,
        &PartnerCodeUnlockInput {
            code: "4826".to_owned(),
        },
    )
    .unwrap();
}

/// The columns R6-1 already names: the shop typed on a purchase, an arrival or an entry.
const KNOWN_R6_1: &[&str] = &[
    "purchases.shop_name",
    "garden_arrivals.shop_name",
    "venture_entries.shop_name",
];

fn describe(found: &BTreeMap<String, usize>) -> String {
    found
        .iter()
        .map(|(c, n)| format!("{c} x{n}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// One run of the sweep with the partner section open and one with it locked, in Urdu and in
/// English. Every free-text column that leaks is a finding; R6-1 names three of them.
#[test]
fn t5_sweep_every_free_text_column_is_kept_out_of_every_tool_result_except_the_known_r6_1() {
    let (y, marked) = marked_year();
    // The sweep is empty if the books have no such text: say how much of the schema it covers.
    let covered: Vec<&String> = marked
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(c, _)| c)
        .collect();
    let empty: Vec<&String> = marked
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(c, _)| c)
        .collect();
    eprintln!(
        "T5 sweep: {} free-text columns hold the marker",
        covered.len()
    );
    eprintln!("T5 sweep: columns with no row in the year: {empty:?}");
    assert!(
        covered.len() >= 25,
        "only {} columns hold rows; the sweep would be thin: {covered:?}",
        covered.len()
    );
    let mut all_leaks: BTreeMap<String, usize> = BTreeMap::new();
    let mut reads = 0usize;
    // Locked first: the tools run as they do while the section is locked.
    for (language, locked) in [(Language::Ur, true), (Language::En, true)] {
        let s = sweep(&y.w, language, locked);
        eprintln!(
            "T5 sweep year locked={locked} {language:?}: {} results, {} refusals, {} pauses, leaks: {}",
            s.ok,
            s.refused,
            s.paused,
            describe(&s.found)
        );
        eprintln!(
            "T5   never read: {:?}; refusals by code: {:?}",
            s.never_read(),
            s.codes
        );
        reads += s.ok;
        for (c, n) in s.found {
            *all_leaks.entry(c).or_default() += n;
        }
    }
    open_partners(&y.w);
    for language in [Language::Ur, Language::En] {
        let s = sweep(&y.w, language, false);
        eprintln!(
            "T5 sweep year open {language:?}: {} results, {} refusals, {} pauses, leaks: {}",
            s.ok,
            s.refused,
            s.paused,
            describe(&s.found)
        );
        eprintln!("T5   never read: {:?}", s.never_read());
        eprintln!("T5   results by source: {:?}", s.by);
        reads += s.ok;
        for (c, n) in s.found {
            *all_leaks.entry(c).or_default() += n;
        }
    }
    // The small world: goods split to another shop are in stock, and a garden has an arrival, so
    // the stock and trip reports carry the typed shop names the year's sold-out goods do not.
    let small = other_shop_day();
    let marked = mark_all(&small.world.dir, false);
    assert!(
        marked["purchases.shop_name"] > 0
            && marked["garden_arrivals.shop_name"] > 0
            && marked["venture_entries.shop_name"] > 0,
        "{marked:?}"
    );
    open_partners(&small.world);
    for locked in [false, true] {
        if locked {
            partner_code::lock(&small.world.env.ctx).unwrap();
        }
        for language in [Language::Ur, Language::En] {
            let s = sweep(&small.world, language, locked);
            eprintln!(
                "T5 sweep small locked={locked} {language:?}: {} results, {} refusals, leaks: {}",
                s.ok,
                s.refused,
                describe(&s.found)
            );
            reads += s.ok;
            for (c, n) in s.found {
                *all_leaks.entry(c).or_default() += n;
            }
        }
    }
    assert!(
        reads >= 1_000,
        "only {reads} reads gave a result: the sweep is thin"
    );
    let new_leaks: Vec<String> = all_leaks
        .keys()
        .filter(|c| !KNOWN_R6_1.contains(&c.as_str()))
        .cloned()
        .collect();
    assert!(
        new_leaks.is_empty(),
        "free text reached a tool result (all leaks: {}), new ones: {new_leaks:?}",
        describe(&all_leaks)
    );
}

/// R6-1 through the sweep: the three shop-name columns, with the typed text in every row.
#[test]
fn t5_sweep_r6_1_no_shop_name_column_leaks() {
    let (y, _) = marked_year();
    let found = sweep(&y.w, Language::En, true).found;
    let leaked: Vec<&String> = found
        .keys()
        .filter(|c| KNOWN_R6_1.contains(&c.as_str()))
        .collect();
    assert!(leaked.is_empty(), "{leaked:?}");
}

// ---- 3. the question the sweep leaves: search as an oracle ---------------------------------

fn rich_buyer(w: &world::World) -> i64 {
    use mandi_services::core::accounts::{AccountInput, AccountType};
    let input = AccountInput {
        id: None,
        version: None,
        account_type: AccountType::Buyer,
        control_id: None,
        code: String::new(),
        name_ur: "صابر".into(),
        name_en: "Sabir".into(),
        confirmed_rules_words: vec!["رازدار".into(), "Raazdar".into()],
        father_or_firm_ur: "حاجی رازدار".into(),
        father_or_firm_en: "Haji Raazdar".into(),
        city_id: None,
        adda_id: None,
        phone1: "0345-5551234".into(),
        phone2: "0333-9998887".into(),
        address: "SECRETADDRESS gali 9".into(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        is_active: true,
        note: "SECRETACCOUNTNOTE".into(),
        tag: String::new(),
    };
    mandi_services::accounts::save(&w.env.ctx, &w.owner, &input)
        .unwrap()
        .id
}

/// The model passes the text of `find` as it likes, and the pickers' search looks at the phone,
/// the father or firm name and the city of an account (PRD FR-7.40). A model that is told by a
/// name in the books to probe ("find 0345", "find 03455", ...) can learn a phone number digit by
/// digit from which account comes back, although no tool result ever holds the phone (FR-7.83).
/// Gives the probes that found Sabir.
fn probes_that_find_sabir() -> Vec<&'static str> {
    let h = Harness::day();
    let sabir = rich_buyer(&h.world);
    let probes = [
        "0345-5551234",
        "03455551234",
        "0345",
        "034555",
        "0333-9998887",
        "Haji Raazdar",
        "Raazdar",
        "حاجی رازدار",
    ];
    with_reader(&h.world, Language::En, false, |r| {
        let mut found = Vec::new();
        for probe in probes {
            for role in ["buyer", "any_party"] {
                r.call("find", json!({"text": probe, "role": role}));
                let last = r.seen.last().cloned().unwrap_or_default();
                if last.contains(&format!("a{sabir}")) || last.contains("Sabir") {
                    found.push(probe);
                    break;
                }
            }
        }
        found
    })
}

/// A QUESTION for Umar (gate T5-Q1), not a decision of mine: `find` matches on text the app keeps
/// back (phone, father or firm name). The test states the strict reading of FR-7.83 (the model
/// learns none of it). It is ignored until Umar says whether the search by phone and by father or
/// firm name is for the Owner's own typing only.
#[test]
fn t5_10_find_does_not_confirm_a_phone_or_a_father_or_firm_name() {
    let found = probes_that_find_sabir();
    assert!(found.is_empty(), "find confirmed private text: {found:?}");
}

/// The control that passes: the same probes find Sabir by his name, so the test above is not
/// empty because of a broken probe.
#[test]
fn t5_find_by_name_still_finds_the_account() {
    let h = Harness::day();
    rich_buyer(&h.world);
    with_reader(&h.world, Language::En, false, |r| {
        r.call("find", json!({"text": "sabir", "role": "buyer"}));
        assert!(r.text().contains("Sabir"), "{}", r.text());
    });
}
