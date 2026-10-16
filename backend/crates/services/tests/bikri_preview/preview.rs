//! The worked numbers of PRD 4.4 and 4.5 through `bikri::preview` on a real database (plan 12
//! acceptance 1). Also writes the fixture the frontend copies for its tests.

use std::path::Path;

use mandi_services::bikri;
use mandi_services::core::bikri::{
    ChargeSource, LineChargeMethod, PaymentType, PukhtaInput, RateHintSource, StoredMethod,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::rules::{CreditLimitMode, OverridesSettings, WorkingValue};
use mandi_services::rules;
use serde_json::json;

use crate::common::{audit_total, raw};
use crate::support::*;

fn split_amounts(p: &mandi_services::core::bikri::BikriPreview) -> Vec<i64> {
    p.split.iter().map(|s| r(s.income_paisa)).collect()
}

#[test]
fn event_1_gives_every_number_of_prd_4_4() {
    let w = world();
    let p = preview(&w, &event1(&w));
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    assert!(p.warnings.is_empty(), "{:?}", p.warnings);
    assert!(p.allow_line_override);

    let lot = &p.lots[0];
    assert_eq!(lot.lot_no, 1);
    let a = calc(lot, 0);
    assert_eq!(
        (
            r(a.seller_amount),
            r(a.seller_comm.amount),
            r(a.buyer_comm.amount),
            r(a.laga.amount),
            r(a.buyer_total)
        ),
        (12_000, 720, 720, 160, 12_880)
    );
    let b = calc(lot, 1);
    assert_eq!(
        (
            r(b.seller_amount),
            r(b.seller_comm.amount),
            r(b.buyer_comm.amount),
            r(b.laga.amount),
            r(b.buyer_total)
        ),
        (17_400, 1_044, 1_044, 240, 18_684)
    );
    let t = lot.totals.as_ref().unwrap();
    assert_eq!(
        (
            r(t.gross),
            r(t.freight),
            r(t.seller_comm),
            r(t.labour),
            r(t.munshiana),
            r(t.cash_paid),
            r(t.phone)
        ),
        (29_400, 3_000, 1_764, 250, 50, 5_000, 20)
    );
    assert_eq!(
        (r(t.deductions), r(t.net), r(t.seller_credit)),
        (10_084, 19_316, 24_316)
    );
    assert_eq!(
        (r(t.buyer_comm), r(t.laga), r(t.buyer_credit)),
        (1_764, 400, 31_564)
    );
    assert_eq!(r(t.shared_income), 3_948);
    // The whole بکری is the one lot.
    assert_eq!(
        (r(p.summary.net), r(p.summary.shared_income)),
        (19_316, 3_948)
    );

    // The live partner split, from plan 11's function: 1,580 / 1,184 / 1,184.
    assert_eq!(split_amounts(&p), vec![1_580, 1_184, 1_184]);
    assert_eq!(
        p.split.iter().map(|s| s.partner_id).collect::<Vec<_>>(),
        w.partners.to_vec()
    );
    assert_eq!(p.split[0].share_bps, 4_000);
    assert_eq!(p.split[0].name_en, "Partner A");
    assert_eq!(p.split[0].working.message_id, "working-partner-share");
    let base = p.split[0]
        .working
        .args
        .iter()
        .find(|arg| arg.name == "base")
        .map(|arg| arg.value);
    assert_eq!(
        base,
        Some(WorkingValue::Money {
            paisa: rupees(3_948)
        })
    );

    // Cash: 50,000 now, 5,000 goes to Rahim, 45,000 after. No confirmation needed.
    assert_eq!(
        (
            r(p.cash.cash_before_paisa),
            r(p.cash.change_paisa),
            r(p.cash.cash_after_paisa)
        ),
        (50_000, -5_000, 45_000)
    );
    assert!(p.cash.lowers_cash && !p.cash.below_zero && !p.cash.needs_confirm);
}

#[test]
fn e2e_26_the_next_row_starts_at_30_with_the_hint_600_and_the_working_of_720() {
    // E2E-26, backend part: after Akram's line (20 of 50) a new row starts at 30 with the rate
    // hint 600, and the commission of line 1 shows "12,000 x 6 / 100 = 720".
    let w = world();
    let draft = draft(vec![with_sales(
        lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![sale("s-1", w.akram, "20", "600"), blank_sale("s-2", "30")],
    )]);
    let p = preview(&w, &draft);
    let rows = &p.lots[0].lines;
    assert_eq!(rows[1].remaining_qty, 30);
    let hint = rows[1].rate_hint.as_ref().expect("a rate hint");
    assert_eq!(
        (r(hint.rate_paisa), hint.source),
        (600, RateHintSource::ThisBikri)
    );
    assert_eq!(hint.date, "2026-10-10");
    let working = &calc(&p.lots[0], 0).seller_comm.working;
    assert_eq!(working.message_id, "working-percent");
    let values: Vec<(String, WorkingValue)> = working
        .args
        .iter()
        .map(|a| (a.name.clone(), a.value))
        .collect();
    assert!(values.contains(&(
        "base".into(),
        WorkingValue::Money {
            paisa: rupees(12_000)
        }
    )));
    assert!(values.contains(&("bps".into(), WorkingValue::Percent { bps: 600 })));
    assert!(values.contains(&("result".into(), WorkingValue::Money { paisa: rupees(720) })));
}

#[test]
fn event_2_one_time_buyer_through_the_pukhta_account() {
    // PRD 4.5.
    let w = world();
    let mut karim = lot(&w, "lot-1", w.karim, w.apple, "10");
    karim.freight = Some("1,000".into());
    karim.cash_paid = None;
    let mut shahid = sale("s-1", w.pukhta_buyer, "10", "1,200");
    shahid.pukhta = Some(PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let p = preview(&w, &draft(vec![with_sales(karim, vec![shahid])]));
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let l = calc(&p.lots[0], 0);
    assert_eq!(
        (r(l.buyer_comm.amount), r(l.laga.amount), r(l.buyer_total)),
        (720, 80, 12_800)
    );
    assert_eq!(l.payment_type, PaymentType::Cash);
    let t = p.lots[0].totals.as_ref().unwrap();
    assert_eq!(
        (r(t.deductions), r(t.net), r(t.shared_income)),
        (1_840, 10_160, 1_540)
    );
    assert_eq!(split_amounts(&p), vec![616, 462, 462]);
    // The cash buyer pays 12,800: cash goes from 50,000 to 62,800.
    assert_eq!(r(p.cash.change_paisa), 12_800);
    assert_eq!(r(p.cash.cash_after_paisa), 62_800);
    assert!(!p.cash.lowers_cash && !p.cash.needs_confirm);
}

#[test]
fn a_partial_sale_leaves_the_rest_in_stock_e2e_16() {
    let w = world();
    let draft = draft(vec![with_sales(
        lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![
            sale("s-1", w.akram, "20", "600"),
            sale("s-2", w.arshad, "25", "580"),
        ],
    )]);
    let p = preview(&w, &draft);
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let t = p.lots[0].totals.as_ref().unwrap();
    assert_eq!(
        (
            r(t.gross),
            r(t.seller_comm),
            r(t.deductions),
            r(t.net),
            r(t.buyer_comm),
            r(t.laga),
            r(t.shared_income)
        ),
        (26_500, 1_590, 9_910, 16_590, 1_590, 360, 3_560)
    );
    assert_eq!(split_amounts(&p), vec![1_424, 1_068, 1_068]);
    assert_eq!(p.lots[0].balance_qty, Some(5));
    assert!(
        p.warnings
            .iter()
            .any(|e| e.code == ErrorCode::ErrQtyLeftUnsold && e.field == "lots[0].qty_received"),
        "{:?}",
        p.warnings
    );
}

#[test]
fn a_negative_net_is_a_warning_and_the_split_is_32_24_24() {
    // Money agent case 5 (E28): one nag at 600, freight 3,000, نقد 5,000.
    let w = world();
    let mut one = lot(&w, "lot-1", w.rahim, w.banana, "1");
    one.labour = None;
    one.munshiana = None;
    one.phone = None;
    let p = preview(
        &w,
        &draft(vec![with_sales(
            one,
            vec![sale("s-1", w.akram, "1", "600")],
        )]),
    );
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let t = p.lots[0].totals.as_ref().unwrap();
    assert_eq!(
        (r(t.net), r(t.seller_credit), r(t.shared_income)),
        (-7_436, -2_436, 80)
    );
    assert!(p.lots[0].net_is_negative);
    assert!(
        p.warnings
            .iter()
            .any(|e| e.code == ErrorCode::ErrLotNetNegative)
    );
    assert_eq!(split_amounts(&p), vec![32, 24, 24]);
}

#[test]
fn a_percent_typed_on_the_line_is_manual_and_the_split_follows() {
    let w = world();
    let mut first = sale("s-1", w.akram, "20", "600");
    first.seller_comm = typed(LineChargeMethod::Percent, "4");
    let draft = draft(vec![with_sales(
        lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![first, sale("s-2", w.arshad, "30", "580")],
    )]);
    let p = preview(&w, &draft);
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let c = &calc(&p.lots[0], 0).seller_comm;
    assert_eq!(
        (r(c.amount), c.method, c.source),
        (480, StoredMethod::Manual, ChargeSource::Typed)
    );
    assert_eq!(c.value_bps, Some(400));
    // 480 + 1,044 + 1,764 + 400 + 20 = 3,708 shared.
    assert_eq!(r(p.summary.shared_income), 3_708);
    // 3,708 at 40/30/30: 1,483 + 1,112 + 1,112 = 3,707, leftover 1 to the largest share.
    assert_eq!(split_amounts(&p), vec![1_484, 1_112, 1_112]);
}

#[test]
fn a_typed_commission_is_refused_when_the_owner_turns_typing_off() {
    let w = world();
    rules::set_overrides(
        &w.env.ctx,
        &w.owner,
        &OverridesSettings {
            allow_line_override: false,
            credit_limit_mode: CreditLimitMode::Warn,
            rate_check_enabled: true,
            rate_check_percent: 30,
        },
    )
    .unwrap();
    let mut first = sale("s-1", w.akram, "20", "600");
    first.seller_comm = typed(LineChargeMethod::Percent, "4");
    let p = preview(
        &w,
        &draft(vec![with_sales(
            lot(&w, "lot-1", w.rahim, w.banana, "50"),
            vec![first],
        )]),
    );
    assert!(!p.allow_line_override);
    assert!(has(
        &p.errors,
        ErrorCode::ErrOverrideNotAllowed,
        "lots[0].sales[0].seller_comm"
    ));
    // The rule decides the number shown meanwhile.
    assert_eq!(r(calc(&p.lots[0], 0).seller_comm.amount), 720);
}

#[test]
fn weight_item_values_the_weight() {
    let w = world();
    let mut grapes = bare_lot(&w, "lot-1", w.rahim, w.weight_fruit, "2");
    grapes.weight_received = Some("12.345".into());
    let mut row = sale("s-1", w.akram, "2", "100");
    row.weight = Some("12.345".into());
    let p = preview(&w, &draft(vec![with_sales(grapes, vec![row])]));
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let l = calc(&p.lots[0], 0);
    assert_eq!(
        (r(l.seller_amount), r(l.laga.amount), r(l.buyer_total)),
        (1_235, 16, 1_325)
    );
}

#[test]
fn a_preview_writes_nothing() {
    let w = world();
    let count = |sql: &str| -> i64 { raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap() };
    let before = (
        count("SELECT COUNT(*) FROM journal_entries"),
        count("SELECT COUNT(*) FROM journal_lines"),
        count("SELECT COUNT(*) FROM bikri_entries"),
        count("SELECT COUNT(*) FROM partner_share_rows"),
        count("SELECT COUNT(*) FROM drafts"),
        audit_total(&w.dir),
    );
    for _ in 0..3 {
        preview(&w, &event1(&w));
    }
    let after = (
        count("SELECT COUNT(*) FROM journal_entries"),
        count("SELECT COUNT(*) FROM journal_lines"),
        count("SELECT COUNT(*) FROM bikri_entries"),
        count("SELECT COUNT(*) FROM partner_share_rows"),
        count("SELECT COUNT(*) FROM drafts"),
        audit_total(&w.dir),
    );
    assert_eq!(before, after);
}

#[test]
fn the_same_draft_gives_the_same_preview() {
    let w = world();
    let draft = event1(&w);
    assert_eq!(preview(&w, &draft), preview(&w, &draft));
}

#[test]
fn a_draft_with_no_lot_says_so_and_returns_zero_totals() {
    let w = world();
    let p = bikri::preview(&w.env.ctx, &draft(vec![])).unwrap();
    assert!(has(&p.errors, ErrorCode::ErrNoLots, "lots"));
    assert_eq!(r(p.summary.gross), 0);
    assert!(p.split.is_empty());
    assert!(!p.cash.needs_confirm);
}

/// The JSON of one sale row as the screen sends it.
fn row_json(key: &str, buyer: Option<i64>, qty: &str, rate: &str) -> serde_json::Value {
    json!({ "key": key, "buyer_account_id": buyer, "pukhta": null, "qty": qty, "weight": null,
        "seller_rate": rate, "rate_differs": false, "buyer_rate": null, "payment_type": null,
        "seller_comm": null, "buyer_comm": null, "laga": null, "note": null })
}

/// `{ "draft": ... }`: what `bikri_preview` takes inside `input`. One lot, Rahim's 50 banana.
fn request_json(w: &World, rows: Vec<serde_json::Value>) -> serde_json::Value {
    json!({
        "draft": {
            "header": {
                "entry_date": "10-10-2026", "bill_date": "", "book": null, "truck_no": "LES-1234",
                "zamindar_account_id": w.rahim, "adda_id": null, "driver_name": null,
                "driver_phone": null, "total_freight": null, "store_id": null, "note": null
            },
            "lots": [{
                "key": "lot-1", "seller_account_id": w.rahim, "pukhta": null,
                "item_id": w.banana, "packing_id": w.box_, "marka_id": null, "store_id": null,
                "qty_received": "50", "weight_received": null, "freight": "3,000",
                "freight_paid_by": "firm",
                "labour": { "method": "per_unit", "value": "5" },
                "munshiana": { "method": "manual", "value": "50" },
                "phone": { "method": "manual", "value": "20" },
                "cash_paid": "5,000", "store_rent": null, "adjust_advance": true,
                "sales": rows
            }]
        }
    })
}

/// Writes a request and the answer to it, the way the frontend copies them. The request is
/// read back into `BikriPreviewInput`, so the file is exactly what the command takes.
fn write_fixture(name: &str, request: &serde_json::Value, w: &World) {
    let input: mandi_services::core::bikri::BikriPreviewInput =
        serde_json::from_value(request.clone()).unwrap();
    let answer = bikri::preview(&w.env.ctx, &input.draft).unwrap();
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(
        folder.join(format!("{name}_request.json")),
        serde_json::to_string_pretty(request).unwrap() + "\n",
    )
    .unwrap();
    std::fs::write(
        folder.join(format!("{name}.json")),
        serde_json::to_string_pretty(&answer).unwrap() + "\n",
    )
    .unwrap();
}

#[test]
fn writes_the_fixtures_the_frontend_copies() {
    // Plan 12 contract. Keys: `lot-1`, `line-1`, `line-2`, `line-3`.
    let w = world();
    // Event 1 after both lines, and a blank third row (the screen always keeps one at the end).
    let event1 = request_json(
        &w,
        vec![
            row_json("line-1", Some(w.akram), "20", "600"),
            row_json("line-2", Some(w.arshad), "30", "580"),
            row_json("line-3", None, "0", ""),
        ],
    );
    write_fixture("bikri_preview_event1", &event1, &w);
    // E2E-26: only the first line; the second row is blank and starts at the remaining 30.
    let e2e_26 = request_json(
        &w,
        vec![
            row_json("line-1", Some(w.akram), "20", "600"),
            row_json("line-2", None, "30", ""),
        ],
    );
    write_fixture("bikri_preview_e2e26", &e2e_26, &w);

    // Spot checks on what was written.
    let input: mandi_services::core::bikri::BikriPreviewInput =
        serde_json::from_value(event1).unwrap();
    let p = bikri::preview(&w.env.ctx, &input.draft).unwrap();
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let remaining: Vec<i64> = p.lots[0].lines.iter().map(|l| l.remaining_qty).collect();
    assert_eq!(remaining, [50, 30, 0]);
    let hints: Vec<Option<i64>> = p.lots[0]
        .lines
        .iter()
        .map(|l| l.rate_hint.as_ref().map(|h| r(h.rate_paisa)))
        .collect();
    assert_eq!(hints, [None, Some(600), Some(580)]);
    // The answer holds numbers (paisa), never text amounts.
    assert!(
        serde_json::to_string(&p)
            .unwrap()
            .contains("\"buyer_total\":1288000")
    );
}

#[test]
fn a_vegetable_has_no_laga_and_says_so() {
    // PRD 4.2 / A2: لاگا is for fruit only; the radish bill carries only the commission.
    let w = world();
    let d = draft(vec![with_sales(
        bare_lot(&w, "lot-1", w.rahim, w.mooli, "10"),
        vec![sale("s-1", w.akram, "10", "100")],
    )]);
    let p = preview(&w, &d);
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    let l = calc(&p.lots[0], 0);
    assert_eq!(
        (r(l.buyer_comm.amount), r(l.laga.amount), r(l.buyer_total)),
        (60, 0, 1_060)
    );
    assert_eq!(l.laga.source, ChargeSource::None);
    assert_eq!(l.laga.working.message_id, "working-none-line");
}

#[test]
fn the_cash_buyer_pays_cash_and_credit_is_refused_for_him() {
    let w = world();
    let d = |payment| {
        let mut row = sale("s-1", w.naqdi, "20", "600");
        row.payment_type = payment;
        draft(vec![with_sales(
            lot(&w, "lot-1", w.rahim, w.banana, "50"),
            vec![row],
        )])
    };
    let p = preview(&w, &d(None));
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    assert_eq!(calc(&p.lots[0], 0).payment_type, PaymentType::Cash);
    // His 12,880 comes in at once: cash change +12,880 - 5,000.
    assert_eq!(r(p.cash.change_paisa), 7_880);
    let p = preview(&w, &d(Some(PaymentType::Credit)));
    assert!(has(
        &p.errors,
        ErrorCode::ErrCreditNotAllowed,
        "lots[0].sales[0].payment_type"
    ));
}
