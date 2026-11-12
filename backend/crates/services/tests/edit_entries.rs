//! Plan 44 T2a part 2: the edit of the trip and garden entries (PRD FR-7.70, FR-12.24 to FR-12.38).
//! An edit reverses the old posting, rewrites the entry inside its edit window (migration 12) and posts
//! the new one on the same day, in one transaction. The expected numbers are the PRD's (E2E-34, E2E-41,
//! E2E-40) and are never changed to suit the code. The edit window must be empty after every save,
//! after every refusal and after a rollback.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::{raw, set_today};
use mandi_services::core::arrivals::ArrivalInput;
use mandi_services::core::edit::{DayFixCloseInput, DayFixOpenInput};
use mandi_services::core::edit::{
    EditEntity, GardenArrivalEditInput, GardenEntryEditInput, GardenHeaderEditInput,
    PurchaseEditInput, TripCostEditInput, TripHeaderEditInput, TripOtherSaleEditInput,
};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::gardens::{
    GardenCloseInput, GardenEntryInput, GardenInput, GardenOtherSaleInput, GardenPaymentInput,
};
use mandi_services::core::purchases::{PurchaseGetInput, PurchaseInput, PurchaseWriteOffInput};
use mandi_services::core::trips::{OtherSaleInput, TripCostInput, TripInput, TripRow};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::{ServiceError, cash, day, edit, gardens, partners, purchases, trips};
use serde_json::{Value, json};
use support::{World, world};

// ---- The real answers as files for the frontend ----

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn write_or_check(file: &str, text: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(file);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::write(&path, text).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{file} is missing: run with {WRITE_VARIABLE}=1 to write the fixtures")
    });
    assert!(
        on_disk == text,
        "{file} is not the real answer any more. If the change is on purpose, run with \
         {WRITE_VARIABLE}=1, read the diff and tell frontend."
    );
}

/// One pair of files: the request and the real answer.
fn pair<A: serde::Serialize>(name: &str, request: &Value, answer: &A) {
    write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

/// A request as the screen sends it, read into its input type (so a fixture is always a valid one).
fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
    serde_json::from_value(request.clone()).unwrap()
}

fn shop(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    }
}

/// A trip of the three partners at 40 / 30 / 30, all paying from the shop's cash.
fn trip_of_three(w: &World) -> TripRow {
    let [a, b, c] = w.partners;
    trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: vec![shop(a, "40"), shop(b, "30"), shop(c, "30")],
        },
    )
    .unwrap()
}

fn purchase_input(
    w: &World,
    trip: i64,
    qty: &str,
    rate: &str,
    freight: Option<&str>,
) -> PurchaseInput {
    PurchaseInput {
        other_qty: None,
        shop_name: None,
        sell_date: None,
        trip_id: Some(trip),
        date: "10-10-2026".into(),
        supplier: Some("Orange Traders".into()),
        item_id: Some(w.banana),
        packing_id: Some(w.box_),
        qty: qty.into(),
        weight: None,
        rate: rate.into(),
        freight: freight.map(str::to_owned),
        labour: None,
        other_cost: None,
        note: None,
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    }
}

fn cash_now(w: &World) -> i64 {
    w.env.ctx.db().read(cash::cash_in_hand).unwrap().rupees()
}

/// Σ credit - Σ debit of an account, in rupees.
fn credit_balance(w: &World, account_id: i64) -> i64 {
    let (dr, cr): (i64, i64) = raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0) FROM journal_lines WHERE account_id = ?1",
            [account_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    (cr - dr) / 100
}

fn venture_account(w: &World, partner_id: i64) -> i64 {
    partners::get(&w.env.ctx, partner_id)
        .unwrap()
        .row
        .venture_account_id
}

fn three_balances(w: &World) -> [i64; 3] {
    w.partners.map(|p| credit_balance(w, venture_account(w, p)))
}

fn count(w: &World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

/// The trial balance difference: 0 whenever the books balance.
fn books_balance(w: &World) -> bool {
    count(
        w,
        "SELECT COALESCE(SUM(dr_paisa), 0) - COALESCE(SUM(cr_paisa), 0) FROM journal_lines",
    ) == 0
}

/// The edit window must be empty between two saves.
fn windows_empty(w: &World) -> bool {
    count(w, "SELECT COUNT(*) FROM edit_windows") == 0
}

fn code_of(error: &ServiceError) -> (ErrorCode, Option<String>) {
    let ServiceError::Rejected(items) = error else {
        panic!("not a refusal: {error:?}");
    };
    (items[0].code, items[0].field.clone())
}

fn edit_input(id: i64, version: i64, purchase: PurchaseInput, reason: &str) -> PurchaseEditInput {
    PurchaseEditInput {
        id,
        version,
        purchase,
        reason: reason.into(),
    }
}

#[test]
fn a_purchase_is_edited_by_reversal_and_a_new_posting_e2e_34() {
    // 20 at 500 with freight 400: paid 10,400, parts 4,160 / 3,120 / 3,120, cash 50,000 -> 39,600.
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let saved = purchases::save(
        ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "500", Some("400")),
    )
    .unwrap();
    assert_eq!(cash_now(&w), 39_600);
    assert_eq!(three_balances(&w), [-4_160, -3_120, -3_120]);

    // The same entry sent back changes nothing.
    let same = edit_input(
        saved.id,
        saved.version,
        purchase_input(&w, trip.id, "20", "500", Some("400")),
        "",
    );
    let preview = edit::purchase_preview(ctx, &same).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(
        preview.review.changes.is_empty(),
        "{:?}",
        preview.review.changes
    );
    assert!(windows_empty(&w), "a review leaves no window open");

    // The rate becomes 520: value 10,400, paid now 10,800, parts 4,320 / 3,240 / 3,240, cash 39,200.
    let purchase_json = |rate: &str, date: &str| {
        json!({
            "trip_id": trip.id, "date": date, "supplier": "Orange Traders", "item_id": w.banana,
            "packing_id": w.box_, "qty": "20", "weight": null, "rate": rate, "freight": "400",
            "labour": null, "other_cost": null, "note": null, "parts": [],
            "confirm_cash_below_zero": false
        })
    };
    let request = json!({
        "id": saved.id, "version": saved.version, "reason": "ریٹ غلط تھا",
        "purchase": purchase_json("520", "10-10-2026")
    });
    let bigger: PurchaseEditInput = typed(&request);
    let entries_before = count(&w, "SELECT COUNT(*) FROM journal_entries");
    let preview = edit::purchase_preview(ctx, &bigger).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    pair("purchases_edit_preview_rate_520", &request, &preview);
    // A mistake comes back inside the preview, at its field: the date never changes.
    let moved_request = json!({
        "id": saved.id, "version": saved.version, "reason": "",
        "purchase": purchase_json("520", "09-10-2026")
    });
    let moved = edit::purchase_preview(ctx, &typed(&moved_request)).unwrap();
    assert_eq!(moved.errors[0].code, ErrorCode::ErrEditDateFixed);
    pair("purchases_edit_preview_date_fixed", &moved_request, &moved);
    let cash_effect = preview.review.effect.cash.as_ref().expect("cash moves");
    assert_eq!(
        (
            cash_effect.before_paisa.rupees(),
            cash_effect.after_paisa.rupees()
        ),
        (39_600, 39_200)
    );
    assert!(!cash_effect.needs_confirm);
    let fields: Vec<&str> = preview
        .review
        .changes
        .iter()
        .map(|c| c.field.as_str())
        .collect();
    for want in ["rate", "value", "paid_now", "total_cost", "amount"] {
        assert!(fields.contains(&want), "{want} in {fields:?}");
    }
    assert_eq!(
        count(&w, "SELECT COUNT(*) FROM journal_entries"),
        entries_before,
        "a review writes nothing"
    );
    assert!(windows_empty(&w));

    let done = edit::purchase_save(ctx, &w.owner, &bigger).unwrap();
    pair("purchases_edit_save_rate_520", &request, &done);
    assert_eq!(done.id, saved.id);
    assert_eq!(done.number, Some(saved.number));
    assert!(done.entry_id.is_some() && done.reversal_entry_id.is_some());
    assert_eq!(done.edited.count, 1);
    assert_eq!(cash_now(&w), 39_200);
    assert_eq!(three_balances(&w), [-4_320, -3_240, -3_240]);
    assert!(books_balance(&w), "the books balance");
    assert!(windows_empty(&w), "the window is closed after the save");
    let row = purchases::get(ctx, &PurchaseGetInput { id: saved.id }).unwrap();
    assert_eq!(row.paid_now_paisa.rupees(), 10_800);
    assert_eq!(row.total_cost_paisa.rupees(), 10_800);
    assert_eq!(row.parts.len(), 3);
    assert!(row.flags.edited.is_some());
    // The stock is still 20, received once.
    assert_eq!(row.qty, 20);
    assert_eq!(
        count(
            &w,
            "SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE purchase_id = 1 AND lot_id IS NULL"
        ),
        20
    );
    // The audit log keeps old and new.
    assert_eq!(
        count(
            &w,
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'purchase' AND action = 'edit'"
        ),
        1
    );
    // The stamp and the history come from the audit log.
    let history = mandi_reports::audit::edit_history(
        ctx.db(),
        &mandi_reports::audit::EditHistoryInput {
            entity: EditEntity::Purchase,
            id: saved.id,
        },
    )
    .unwrap();
    assert_eq!(history.versions.len(), 1);
}

#[test]
fn a_purchase_edit_refuses_what_it_must() {
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let saved = purchases::save(
        ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "500", Some("400")),
    )
    .unwrap();
    let books = count(&w, "SELECT COUNT(*) FROM journal_entries");
    let try_edit =
        |input: &PurchaseEditInput| edit::purchase_save(ctx, &w.owner, input).unwrap_err();

    // A stale version.
    let stale = edit_input(
        saved.id,
        saved.version + 5,
        purchase_input(&w, trip.id, "20", "520", Some("400")),
        "",
    );
    assert_eq!(
        code_of(&try_edit(&stale)),
        (ErrorCode::ErrVersionChanged, Some("version".into()))
    );
    // The date never changes.
    let mut moved = purchase_input(&w, trip.id, "20", "520", Some("400"));
    moved.date = "09-10-2026".into();
    let err = try_edit(&edit_input(saved.id, saved.version, moved, ""));
    assert_eq!(
        code_of(&err),
        (ErrorCode::ErrEditDateFixed, Some("date".into()))
    );
    // A mistake is named at its field; nothing is written.
    let err = try_edit(&edit_input(
        saved.id,
        saved.version,
        purchase_input(&w, trip.id, "0", "520", Some("400")),
        "",
    ));
    assert_eq!(code_of(&err).1.as_deref(), Some("qty"));
    // Cash below zero asks (the shop has 39,600 and a purchase of 99,999,000 would take far more).
    let big = purchase_input(&w, trip.id, "20", "5000000", None);
    let err = try_edit(&edit_input(saved.id, saved.version, big, ""));
    assert_eq!(code_of(&err).0, ErrorCode::ErrCashBelowZeroNotConfirmed);
    assert_eq!(count(&w, "SELECT COUNT(*) FROM journal_entries"), books);
    assert_eq!(cash_now(&w), 39_600);
    assert_eq!(three_balances(&w), [-4_160, -3_120, -3_120]);
    assert!(windows_empty(&w), "a refusal rolls the window back too");

    // Tomorrow the purchase is not of today: a reason is needed.
    set_today(&w.env, 2026, 10, 11);
    let asked = edit_input(
        saved.id,
        saved.version,
        purchase_input(&w, trip.id, "20", "520", Some("400")),
        "",
    );
    assert_eq!(
        code_of(&try_edit(&asked)),
        (ErrorCode::ErrReasonRequired, Some("reason".into()))
    );
    let preview = edit::purchase_preview(ctx, &asked).unwrap();
    assert!(preview.review.reason_required && preview.errors.is_empty());
    let with_reason = edit_input(
        saved.id,
        saved.version,
        purchase_input(&w, trip.id, "20", "520", Some("400")),
        "ریٹ غلط تھا",
    );
    edit::purchase_save(ctx, &w.owner, &with_reason).unwrap();
    assert_eq!(three_balances(&w), [-4_320, -3_240, -3_240]);
    assert!(windows_empty(&w));
}

#[test]
fn a_purchase_with_sold_goods_or_a_trip_cost_keeps_what_the_cost_rests_on() {
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let saved = purchases::save(
        ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "500", Some("400")),
    )
    .unwrap();
    let [a, b, c] = w.partners;
    // A trip cost of 4,000 is spread over the purchase: it carries a cost now.
    trips::cost_save(
        ctx,
        &w.owner,
        &TripCostInput {
            trip_id: Some(trip.id),
            date: "10-10-2026".into(),
            amount: "4,000".into(),
            reason: "truck".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let version = purchases::get(ctx, &PurchaseGetInput { id: saved.id })
        .unwrap()
        .version;
    // Its quantity is kept (the cost was spread by the نگ bought) ...
    let err = edit::purchase_save(
        ctx,
        &w.owner,
        &edit_input(
            saved.id,
            version,
            purchase_input(&w, trip.id, "25", "500", Some("400")),
            "",
        ),
    )
    .unwrap_err();
    let ServiceError::Rejected(items) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrEditNotAllowed, Some("qty"))
    );
    assert_eq!(
        items[0].values.get("rule"),
        Some(&ErrorValue::Text("carries_cost".into()))
    );
    // ... but the rate can change: the total cost keeps its part of the trip cost.
    edit::purchase_save(
        ctx,
        &w.owner,
        &edit_input(
            saved.id,
            version,
            purchase_input(&w, trip.id, "20", "520", Some("400")),
            "",
        ),
    )
    .unwrap();
    let row = purchases::get(ctx, &PurchaseGetInput { id: saved.id }).unwrap();
    assert_eq!(row.paid_now_paisa.rupees(), 10_800);
    assert_eq!(row.total_cost_paisa.rupees(), 10_800 + 4_000);
    assert!(books_balance(&w) && windows_empty(&w));
    let _ = (a, b, c);

    // Once goods are written off the purchase is final.
    let version = purchases::get(ctx, &PurchaseGetInput { id: saved.id })
        .unwrap()
        .version;
    purchases::write_off(
        ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: saved.id,
            qty: "2".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
    let moved_goods = edit_input(
        saved.id,
        version + 1,
        purchase_input(&w, trip.id, "20", "530", Some("400")),
        "",
    );
    let refused = edit::purchase_preview(ctx, &moved_goods).unwrap();
    assert_eq!(refused.errors[0].code, ErrorCode::ErrEditNotAllowed);
    pair(
        "purchases_edit_preview_goods_moved",
        &json!({
            "id": saved.id, "version": version + 1, "reason": "",
            "purchase": {
                "trip_id": trip.id, "date": "10-10-2026", "supplier": "Orange Traders",
                "item_id": w.banana, "packing_id": w.box_, "qty": "20", "weight": null,
                "rate": "530", "freight": "400", "labour": null, "other_cost": null,
                "note": null, "parts": [], "confirm_cash_below_zero": false
            }
        }),
        &refused,
    );
    let err = edit::purchase_save(ctx, &w.owner, &moved_goods).unwrap_err();
    let ServiceError::Rejected(items) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrEditNotAllowed, Some("id"))
    );
    assert_eq!(
        items[0].values.get("rule"),
        Some(&ErrorValue::Text("goods_moved".into()))
    );
    assert!(windows_empty(&w));
}

fn cost_input(trip: i64, amount: &str) -> TripCostInput {
    TripCostInput {
        trip_id: Some(trip),
        date: "10-10-2026".into(),
        amount: amount.into(),
        reason: "truck".into(),
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    }
}

fn trip_cost_of(w: &World, purchase_id: i64) -> i64 {
    purchases::get(&w.env.ctx, &PurchaseGetInput { id: purchase_id })
        .unwrap()
        .total_cost_paisa
        .rupees()
}

#[test]
fn a_trip_cost_is_edited_and_spread_again_over_the_same_purchases() {
    // E2E-34 purchase: 20 at 500 + freight 400 = 10,400. A truck of 4,000 is spread over it (parts
    // 1,600 / 1,200 / 1,200); the purchase's total cost is 14,400.
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let bought = purchases::save(
        ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "500", Some("400")),
    )
    .unwrap();
    let cost = trips::cost_save(ctx, &w.owner, &cost_input(trip.id, "4,000")).unwrap();
    assert_eq!(trip_cost_of(&w, bought.id), 14_400);
    // A second purchase made AFTER the cost is not part of its spread.
    let later = purchases::save(
        ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "10", "400", None),
    )
    .unwrap();
    assert_eq!(trip_cost_of(&w, later.id), 4_000);
    let cash_before = cash_now(&w);
    let balances_before = three_balances(&w);
    assert_eq!(
        balances_before,
        [
            -4_160 - 1_600 - 1_600,
            -3_120 - 1_200 - 1_200,
            -3_120 - 1_200 - 1_200
        ]
    );

    // The same cost sent back changes nothing.
    let same = TripCostEditInput {
        id: cost.id,
        version: cost.version,
        cost: cost_input(trip.id, "4,000"),
        reason: String::new(),
    };
    let preview = edit::trip_cost_preview(ctx, &same).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(
        preview.review.changes.is_empty(),
        "{:?}",
        preview.review.changes
    );

    // 4,400: parts 1,760 / 1,320 / 1,320 and the purchase carries all of it.
    let request = json!({
        "id": cost.id, "version": cost.version, "reason": "",
        "cost": {
            "trip_id": trip.id, "date": "10-10-2026", "amount": "4,400", "reason": "truck",
            "parts": [], "confirm_cash_below_zero": false
        }
    });
    let bigger: TripCostEditInput = typed(&request);
    let preview = edit::trip_cost_preview(ctx, &bigger).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    pair("trips_cost_edit_preview_4400", &request, &preview);
    let c = preview.review.effect.cash.as_ref().expect("cash moves");
    assert_eq!(
        (c.before_paisa.rupees(), c.after_paisa.rupees()),
        (cash_before, cash_before - 400)
    );
    assert_eq!(
        trip_cost_of(&w, bought.id),
        14_400,
        "a review writes nothing"
    );
    let done = edit::trip_cost_save(ctx, &w.owner, &bigger).unwrap();
    pair("trips_cost_edit_save_4400", &request, &done);
    assert_eq!(done.number, Some(cost.number));
    assert_eq!(done.edited.count, 1);
    assert_eq!(trip_cost_of(&w, bought.id), 14_800);
    assert_eq!(
        trip_cost_of(&w, later.id),
        4_000,
        "the later purchase carries none"
    );
    assert_eq!(cash_now(&w), cash_before - 400);
    assert_eq!(
        three_balances(&w),
        [
            balances_before[0] - 160,
            balances_before[1] - 120,
            balances_before[2] - 120
        ]
    );
    assert!(books_balance(&w) && windows_empty(&w));

    // Nothing of the purchase may have moved.
    let version = done.version;
    purchases::write_off(
        ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: bought.id,
            qty: "1".into(),
            reason: "خراب".into(),
        },
    )
    .unwrap();
    let err = edit::trip_cost_save(
        ctx,
        &w.owner,
        &TripCostEditInput {
            id: cost.id,
            version,
            cost: cost_input(trip.id, "4,500"),
            reason: String::new(),
        },
    )
    .unwrap_err();
    let ServiceError::Rejected(items) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrEditNotAllowed, Some("id"))
    );
    assert_eq!(
        items[0].values.get("rule"),
        Some(&ErrorValue::Text("goods_moved".into()))
    );
    assert!(windows_empty(&w));
}

fn sale_input(trip: i64, purchase: i64, qty: &str, amount: &str) -> OtherSaleInput {
    OtherSaleInput {
        trip_id: Some(trip),
        date: "10-10-2026".into(),
        shop: Some("حاجی کی دکان".into()),
        purchase_id: Some(purchase),
        qty: qty.into(),
        amount: amount.into(),
        note: None,
    }
}

fn purchase_figures(w: &World, id: i64) -> (i64, i64, i64) {
    let row = purchases::get(&w.env.ctx, &PurchaseGetInput { id }).unwrap();
    (row.qty_sold, row.cost_released_paisa.rupees(), row.qty_left)
}

#[test]
fn a_sale_in_another_shop_is_edited_and_the_cost_follows_c21() {
    // E2E-34: 15 of 20 نگ sold in another shop for 8,460: cost 7,800, credits 3,384 / 2,538 / 2,538.
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let bought = purchases::save(
        ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "500", Some("400")),
    )
    .unwrap();
    let sale = trips::other_sale_save(
        ctx,
        &w.owner,
        &sale_input(trip.id, bought.id, "15", "8,460"),
    )
    .unwrap();
    assert_eq!(purchase_figures(&w, bought.id), (15, 7_800, 5));
    let cash_before = cash_now(&w);
    let balances_before = three_balances(&w);
    assert_eq!(
        balances_before,
        [-4_160 + 3_384, -3_120 + 2_538, -3_120 + 2_538]
    );

    let same = TripOtherSaleEditInput {
        id: sale.id,
        version: sale.version,
        sale: sale_input(trip.id, bought.id, "15", "8,460"),
        reason: String::new(),
        confirm_cash_below_zero: false,
    };
    let preview = edit::trip_other_sale_preview(ctx, &same).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(
        preview.review.changes.is_empty(),
        "{:?}",
        preview.review.changes
    );

    // 9,000 for the same 15: credits 3,600 / 2,700 / 2,700 and cash up by 540.
    let request = json!({
        "id": sale.id, "version": sale.version, "reason": "",
        "confirm_cash_below_zero": false,
        "sale": {
            "trip_id": trip.id, "date": "10-10-2026", "shop": "حاجی کی دکان",
            "purchase_id": bought.id, "qty": "15", "amount": "9,000", "note": null
        }
    });
    let better: TripOtherSaleEditInput = typed(&request);
    let preview = edit::trip_other_sale_preview(ctx, &better).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    pair("trips_other_sale_edit_preview_9000", &request, &preview);
    assert_eq!(
        purchase_figures(&w, bought.id),
        (15, 7_800, 5),
        "a review writes nothing"
    );
    let done = edit::trip_other_sale_save(ctx, &w.owner, &better).unwrap();
    assert_eq!(done.number, Some(sale.number));
    assert_eq!(cash_now(&w), cash_before + 540);
    assert_eq!(
        three_balances(&w),
        [
            balances_before[0] + 216,
            balances_before[1] + 162,
            balances_before[2] + 162
        ]
    );
    assert_eq!(purchase_figures(&w, bought.id), (15, 7_800, 5));
    assert!(books_balance(&w) && windows_empty(&w));

    // 10 نگ instead of 15: the cost is 10,400 x 10 / 20 = 5,200 and 10 are left.
    let fewer = TripOtherSaleEditInput {
        version: done.version,
        sale: sale_input(trip.id, bought.id, "10", "6,000"),
        ..same.clone()
    };
    edit::trip_other_sale_save(ctx, &w.owner, &fewer).unwrap();
    assert_eq!(purchase_figures(&w, bought.id), (10, 5_200, 10));
    assert!(books_balance(&w) && windows_empty(&w));

    // Not the only mover any more: another sale of the same purchase makes it final.
    let second =
        trips::other_sale_save(ctx, &w.owner, &sale_input(trip.id, bought.id, "2", "1,000"))
            .unwrap();
    let version = purchases::get(ctx, &PurchaseGetInput { id: bought.id })
        .unwrap()
        .version;
    let _ = (second, version);
    let first_version = mandi_services::trips::get(
        ctx,
        &mandi_services::core::trips::TripGetInput {
            id: trip.id,
            entries_limit: 100,
            entries_offset: 0,
        },
    )
    .unwrap()
    .entries
    .iter()
    .find(|e| e.id == sale.id)
    .unwrap()
    .version;
    let err = edit::trip_other_sale_save(
        ctx,
        &w.owner,
        &TripOtherSaleEditInput {
            id: sale.id,
            version: first_version,
            sale: sale_input(trip.id, bought.id, "10", "6,100"),
            reason: String::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    let ServiceError::Rejected(items) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrEditNotAllowed, Some("id"))
    );
    assert_eq!(
        items[0].values.get("rule"),
        Some(&ErrorValue::Text("goods_moved".into()))
    );
    assert!(windows_empty(&w));
}

fn pocket(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Pocket,
    }
}

/// E2E-40: شاہ پور باغ, الف 60% from the shop's cash and ب 40% from his pocket, rent 50,000.
fn garden_of_two(w: &World) -> mandi_services::core::gardens::GardenRow {
    let [a, b, _] = w.partners;
    gardens::create(
        &w.env.ctx,
        &w.owner,
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
            rent: "50,000".into(),
            season_end: None,
            note: None,
            partners: vec![shop(a, "60"), pocket(b, "40")],
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .row
}

fn payment(garden: i64, amount: &str, reason: &str) -> GardenPaymentInput {
    GardenPaymentInput {
        garden_id: Some(garden),
        date: "10-10-2026".into(),
        amount: amount.into(),
        reason: reason.into(),
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    }
}

#[test]
fn a_garden_installment_is_edited_and_only_the_shop_part_posts() {
    // E2E-40: an installment of 30,000 at 60 / 40: parts 18,000 (shop) / 12,000 (pocket).
    let w = world();
    let ctx = &w.env.ctx;
    let garden = garden_of_two(&w);
    let [a, b, _] = w.partners;
    let saved = gardens::entry_save(
        ctx,
        &w.owner,
        &GardenEntryInput::Installment(payment(garden.id, "30,000", "پہلی قسط")),
    )
    .unwrap();
    assert_eq!(cash_now(&w), 50_000 - 18_000);
    assert_eq!(credit_balance(&w, venture_account(&w, a)), -18_000);
    assert_eq!(credit_balance(&w, venture_account(&w, b)), 0);

    let same = GardenEntryEditInput {
        id: saved.id,
        version: saved.version,
        entry: GardenEntryInput::Installment(payment(garden.id, "30,000", "پہلی قسط")),
        reason: String::new(),
        confirm_cash_below_zero: false,
    };
    let preview = edit::garden_entry_preview(ctx, &same).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(
        preview.review.changes.is_empty(),
        "{:?}",
        preview.review.changes
    );

    // 32,000: parts 19,200 / 12,800. Only 19,200 posts; cash 50,000 - 19,200.
    let request = json!({
        "id": saved.id, "version": saved.version, "reason": "", "confirm_cash_below_zero": false,
        "entry": {
            "type": "installment", "garden_id": garden.id, "date": "10-10-2026",
            "amount": "32,000", "reason": "پہلی قسط", "parts": [], "confirm_cash_below_zero": false
        }
    });
    let bigger: GardenEntryEditInput = typed(&request);
    let preview = edit::garden_entry_preview(ctx, &bigger).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    pair("gardens_entry_edit_preview_32000", &request, &preview);
    assert_eq!(cash_now(&w), 32_000, "a review writes nothing");
    let done = edit::garden_entry_save(ctx, &w.owner, &bigger).unwrap();
    pair("gardens_entry_edit_save_32000", &request, &done);
    assert_eq!(done.number, Some(saved.number));
    assert_eq!(cash_now(&w), 50_000 - 19_200);
    assert_eq!(credit_balance(&w, venture_account(&w, a)), -19_200);
    assert_eq!(credit_balance(&w, venture_account(&w, b)), 0);
    assert!(books_balance(&w) && windows_empty(&w));

    // The reason of the edit is `edit_reason`: tomorrow the entry is not of today.
    set_today(&w.env, 2026, 10, 11);
    let again = GardenEntryEditInput {
        version: done.version,
        entry: GardenEntryInput::Installment(payment(garden.id, "31,000", "پہلی قسط")),
        ..same.clone()
    };
    let err = edit::garden_entry_save(ctx, &w.owner, &again).unwrap_err();
    assert_eq!(
        code_of(&err),
        (ErrorCode::ErrReasonRequired, Some("edit_reason".into()))
    );
    // The kind never changes.
    let wrong_kind = GardenEntryEditInput {
        entry: GardenEntryInput::Cost(payment(garden.id, "31,000", "پہلی قسط")),
        reason: "غلط".into(),
        ..again.clone()
    };
    assert_eq!(
        code_of(&edit::garden_entry_save(ctx, &w.owner, &wrong_kind).unwrap_err()),
        (ErrorCode::ErrFormatInvalid, Some("type".into()))
    );
    let fixed = GardenEntryEditInput {
        reason: "رقم غلط".into(),
        ..again
    };
    edit::garden_entry_save(ctx, &w.owner, &fixed).unwrap();
    assert_eq!(cash_now(&w), 50_000 - 18_600);
    assert!(books_balance(&w) && windows_empty(&w));

    // A closed garden takes no edit.
    gardens::close(
        ctx,
        &w.owner,
        &GardenCloseInput {
            id: garden.id,
            version: gardens::get(
                ctx,
                &mandi_services::core::gardens::GardenGetInput {
                    id: garden.id,
                    history_offset: 0,
                    history_limit: 0,
                },
            )
            .unwrap()
            .row
            .version,
            date: "11-10-2026".into(),
        },
    )
    .unwrap();
    let version = mandi_services::gardens::get(
        ctx,
        &mandi_services::core::gardens::GardenGetInput {
            id: garden.id,
            history_offset: 0,
            history_limit: 100,
        },
    )
    .unwrap()
    .history
    .iter()
    .find(|h| h.entry_id == Some(saved.id))
    .and_then(|h| h.version)
    .unwrap();
    let err = edit::garden_entry_save(
        ctx,
        &w.owner,
        &GardenEntryEditInput {
            id: saved.id,
            version,
            entry: GardenEntryInput::Installment(payment(garden.id, "31,500", "پہلی قسط")),
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(
        code_of(&err),
        (ErrorCode::ErrGardenClosed, Some("id".into()))
    );
    assert!(windows_empty(&w));
}

#[test]
fn a_garden_sale_in_another_shop_is_edited_and_the_gardens_account_follows() {
    let w = world();
    let ctx = &w.env.ctx;
    let garden = garden_of_two(&w);
    let sale = |qty: &str, amount: &str| {
        GardenEntryInput::OtherSale(GardenOtherSaleInput {
            garden_id: Some(garden.id),
            date: "10-10-2026".into(),
            shop: Some("حاجی کی دکان".into()),
            arrival_id: None,
            qty: qty.into(),
            amount: amount.into(),
            note: None,
        })
    };
    let saved = gardens::entry_save(ctx, &w.owner, &sale("20", "10,000")).unwrap();
    assert_eq!(cash_now(&w), 60_000);
    assert_eq!(credit_balance(&w, garden.account.id), 10_000);
    let edited = GardenEntryEditInput {
        id: saved.id,
        version: saved.version,
        entry: sale("20", "12,000"),
        reason: String::new(),
        confirm_cash_below_zero: false,
    };
    let preview = edit::garden_entry_preview(ctx, &edited).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let c = preview.review.effect.cash.as_ref().expect("cash moves");
    assert_eq!(
        (c.before_paisa.rupees(), c.after_paisa.rupees()),
        (60_000, 62_000)
    );
    edit::garden_entry_save(ctx, &w.owner, &edited).unwrap();
    assert_eq!(cash_now(&w), 62_000);
    assert_eq!(credit_balance(&w, garden.account.id), 12_000);
    assert!(books_balance(&w) && windows_empty(&w));
}

fn arrival_input(w: &World, garden: i64, qty: &str, note: Option<&str>) -> ArrivalInput {
    ArrivalInput {
        garden_id: Some(garden),
        date: "10-10-2026".into(),
        sell_date: None,
        item_id: Some(w.banana),
        packing_id: Some(w.box_),
        qty: qty.into(),
        weight: None,
        note: note.map(str::to_owned),
    }
}

fn arrival_stock(w: &World, id: i64) -> i64 {
    count(
        w,
        &format!(
            "SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE arrival_id = {id} AND lot_id IS NULL"
        ),
    )
}

#[test]
fn a_garden_arrival_is_edited_and_its_stock_follows() {
    let w = world();
    let ctx = &w.env.ctx;
    let garden = garden_of_two(&w);
    let saved = mandi_services::gardens::arrivals::save(
        ctx,
        &w.owner,
        &arrival_input(&w, garden.id, "100", None),
    )
    .unwrap()
    .row;
    assert_eq!(arrival_stock(&w, saved.id), 100);

    let same = GardenArrivalEditInput {
        id: saved.id,
        version: saved.version,
        arrival: arrival_input(&w, garden.id, "100", None),
        reason: String::new(),
    };
    let preview = edit::garden_arrival_preview(ctx, &same).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(
        preview.review.changes.is_empty(),
        "{:?}",
        preview.review.changes
    );

    let request = json!({
        "id": saved.id, "version": saved.version, "reason": "",
        "arrival": {
            "garden_id": garden.id, "date": "10-10-2026", "sell_date": null, "item_id": w.banana,
            "packing_id": w.box_, "qty": "120", "weight": null, "note": "بارش میں آئے"
        }
    });
    let bigger: GardenArrivalEditInput = typed(&request);
    let preview = edit::garden_arrival_preview(ctx, &bigger).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    pair("gardens_arrival_edit_preview_120", &request, &preview);
    let fields: Vec<&str> = preview
        .review
        .changes
        .iter()
        .map(|c| c.field.as_str())
        .collect();
    assert!(
        fields.contains(&"qty") && fields.contains(&"note"),
        "{fields:?}"
    );
    assert_eq!(arrival_stock(&w, saved.id), 100, "a review writes nothing");
    let done = edit::garden_arrival_save(ctx, &w.owner, &bigger).unwrap();
    assert_eq!(done.number, Some(saved.number));
    assert!(
        done.entry_id.is_none() && done.reversal_entry_id.is_none(),
        "nothing posts"
    );
    assert_eq!(arrival_stock(&w, saved.id), 120);
    assert!(windows_empty(&w));

    // The date never changes; a quantity below the split for another shop is refused by the split.
    let mut moved = arrival_input(&w, garden.id, "120", None);
    moved.date = "09-10-2026".into();
    let err = edit::garden_arrival_save(
        ctx,
        &w.owner,
        &GardenArrivalEditInput {
            version: done.version,
            arrival: moved,
            ..same.clone()
        },
    )
    .unwrap_err();
    assert_eq!(
        code_of(&err),
        (ErrorCode::ErrEditDateFixed, Some("date".into()))
    );
    assert!(windows_empty(&w));
}

fn trip_header(w: &World, trip: &TripRow, market: &str) -> TripInput {
    let [a, b, c] = w.partners;
    let _ = trip;
    TripInput {
        date: "10-10-2026".into(),
        market: Some(market.into()),
        sent_person: None,
        note: Some("نوٹ".into()),
        partners: vec![shop(a, "40"), shop(b, "30"), shop(c, "30")],
    }
}

#[test]
fn a_trip_header_is_edited_and_its_shares_stay() {
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let input = |market: &str| TripHeaderEditInput {
        id: trip.id,
        version: trip.version,
        trip: trip_header(&w, &trip, market),
        reason: String::new(),
    };
    let [pa, pb, pc] = w.partners;
    let request = json!({
        "id": trip.id, "version": trip.version, "reason": "منڈی کا نام بدلا",
        "trip": {
            "date": "10-10-2026", "market": "Multan", "sent_person": null, "note": "نوٹ",
            "partners": [
                { "partner_id": pa, "share": "40", "source": "shop" },
                { "partner_id": pb, "share": "30", "source": "shop" },
                { "partner_id": pc, "share": "30", "source": "shop" }
            ]
        }
    });
    let preview = edit::trip_header_preview(ctx, &typed(&request)).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    pair("trips_header_edit_preview_market", &request, &preview);
    let fields: Vec<&str> = preview
        .review
        .changes
        .iter()
        .map(|c| c.field.as_str())
        .collect();
    assert_eq!(fields, ["market", "note"]);
    let done = edit::trip_header_save(ctx, &w.owner, &input("Multan")).unwrap();
    assert_eq!(done.number, Some(trip.number));
    assert_eq!(done.edited.count, 1);
    let row = trips::get(
        ctx,
        &mandi_services::core::trips::TripGetInput {
            id: trip.id,
            entries_limit: 10,
            entries_offset: 0,
        },
    )
    .unwrap()
    .row;
    assert_eq!(row.market.as_deref(), Some("Multan"));
    // Other shares are refused.
    let [a, b, c] = w.partners;
    let mut other = input("Multan");
    other.version = done.version;
    other.trip.partners = vec![shop(a, "50"), shop(b, "25"), shop(c, "25")];
    let err = edit::trip_header_save(ctx, &w.owner, &other).unwrap_err();
    assert_eq!(
        code_of(&err),
        (ErrorCode::ErrVentureSharesLocked, Some("partners".into()))
    );
    assert!(windows_empty(&w) && books_balance(&w));
}

#[test]
fn a_garden_header_is_edited_and_its_account_and_rent_follow() {
    let w = world();
    let ctx = &w.env.ctx;
    let garden = garden_of_two(&w);
    let [a, b, _] = w.partners;
    let input = |rent: &str, name_en: &str, version: i64| GardenHeaderEditInput {
        id: garden.id,
        version,
        garden: GardenInput {
            name_ur: "شاہ پور باغ".into(),
            name_en: name_en.into(),
            confirmed_rules_words: vec![
                "شاہ".into(),
                "پور".into(),
                "باغ".into(),
                "Shahpur".into(),
                "Garden".into(),
                "Bagh".into(),
            ],
            date: "10-10-2026".into(),
            item_id: Some(w.banana),
            rent: rent.into(),
            season_end: None,
            note: Some("نیا نوٹ".into()),
            partners: vec![shop(a, "60"), pocket(b, "40")],
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
        reason: String::new(),
    };
    let preview =
        edit::garden_header_preview(ctx, &input("60,000", "Shahpur Bagh", garden.version)).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    pair(
        "gardens_header_edit_preview_rent_60000",
        &json!({
            "id": garden.id, "version": garden.version, "reason": "",
            "garden": {
                "name_ur": "شاہ پور باغ", "name_en": "Shahpur Bagh",
                "confirmed_rules_words": ["شاہ", "پور", "باغ", "Shahpur", "Garden", "Bagh"],
                "date": "10-10-2026", "item_id": w.banana, "rent": "60,000", "season_end": null,
                "note": "نیا نوٹ",
                "partners": [
                    { "partner_id": a, "share": "60", "source": "shop" },
                    { "partner_id": b, "share": "40", "source": "pocket" }
                ],
                "paid_now": null, "paid_now_reason": null, "parts": [],
                "confirm_cash_below_zero": false
            }
        }),
        &preview,
    );
    let fields: Vec<&str> = preview
        .review
        .changes
        .iter()
        .map(|c| c.field.as_str())
        .collect();
    for want in ["name", "rent", "note"] {
        assert!(fields.contains(&want), "{want} in {fields:?}");
    }
    assert!(windows_empty(&w), "a review leaves no window");
    let cash_before = cash_now(&w);
    let done = edit::garden_header_save(
        ctx,
        &w.owner,
        &input("60,000", "Shahpur Bagh", garden.version),
    )
    .unwrap();
    assert_eq!(done.edited.count, 1);
    assert_eq!(cash_now(&w), cash_before, "the rent posts nothing");
    let detail = gardens::get(
        ctx,
        &mandi_services::core::gardens::GardenGetInput {
            id: garden.id,
            history_offset: 0,
            history_limit: 10,
        },
    )
    .unwrap();
    assert_eq!(detail.row.rent_paisa.rupees(), 60_000);
    assert_eq!(detail.row.name_en, "Shahpur Bagh");
    // The garden's account carries the new name.
    let account_name: String = raw(&w.dir)
        .query_row(
            "SELECT name_en FROM accounts WHERE id = ?1",
            [garden.account.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(account_name, "Shahpur Bagh");
    assert!(windows_empty(&w) && books_balance(&w));
}

#[test]
fn a_purchase_of_a_closed_day_is_fixed_through_reopen_and_fix() {
    // Plan 44 FR-21.4 for a trip purchase: the day is closed, the row still offers Edit through a
    // reopen, the review says so, and after `day_fix_open` the same edit saves.
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let saved = purchases::save(
        ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "500", Some("400")),
    )
    .unwrap();
    day::close(
        ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);

    let row = purchases::get(ctx, &PurchaseGetInput { id: saved.id }).unwrap();
    assert!(row.flags.can_edit && row.flags.needs_reopen && row.flags.locked_reason.is_none());
    let input = edit_input(
        saved.id,
        saved.version,
        purchase_input(&w, trip.id, "20", "520", Some("400")),
        "ریٹ غلط تھا",
    );
    let preview = edit::purchase_preview(ctx, &input).unwrap();
    assert_eq!(preview.errors[0].code, ErrorCode::ErrDateClosed);
    assert!(preview.review.needs_reopen && preview.review.reason_required);
    let err = edit::purchase_save(ctx, &w.owner, &input).unwrap_err();
    assert_eq!(code_of(&err).0, ErrorCode::ErrDateClosed);
    assert!(windows_empty(&w));

    let fix = day::fix_open(
        ctx,
        &w.owner,
        &DayFixOpenInput {
            date: "10-10-2026".into(),
            reason: "ریٹ غلط تھا".into(),
            sign: "adnan".into(),
        },
    )
    .unwrap();
    let preview = edit::purchase_preview(ctx, &input).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(!preview.review.needs_reopen);
    edit::purchase_save(ctx, &w.owner, &input).unwrap();
    assert_eq!(three_balances(&w), [-4_320, -3_240, -3_240]);
    let closed = day::fix_close(ctx, &w.owner, &DayFixCloseInput { token: fix.token }).unwrap();
    assert_eq!(closed.closed, ["2026-10-10"]);
    assert!(books_balance(&w) && windows_empty(&w));
}

#[test]
fn a_sale_edit_keeps_its_cost_when_a_trip_cost_came_after_it() {
    // 15 of 20 sold for 8,460, cost 7,800 (C21). A truck of 1,000 is saved later: it belongs to the 5
    // left, not to the sale. Editing only the money keeps the cost 7,800; a new quantity is refused.
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let bought = purchases::save(
        ctx,
        &w.owner,
        &purchase_input(&w, trip.id, "20", "500", Some("400")),
    )
    .unwrap();
    let sale = trips::other_sale_save(
        ctx,
        &w.owner,
        &sale_input(trip.id, bought.id, "15", "8,460"),
    )
    .unwrap();
    trips::cost_save(ctx, &w.owner, &cost_input(trip.id, "1,000")).unwrap();
    assert_eq!(trip_cost_of(&w, bought.id), 11_400);
    assert_eq!(purchase_figures(&w, bought.id), (15, 7_800, 5));

    let edit_of = |qty: &str, amount: &str| TripOtherSaleEditInput {
        id: sale.id,
        version: sale.version,
        sale: sale_input(trip.id, bought.id, qty, amount),
        reason: String::new(),
        confirm_cash_below_zero: false,
    };
    edit::trip_other_sale_save(ctx, &w.owner, &edit_of("15", "9,000")).unwrap();
    assert_eq!(
        purchase_figures(&w, bought.id),
        (15, 7_800, 5),
        "the cost of the goods sold is not re-worked on the new total cost"
    );
    let version = trips::get(
        ctx,
        &mandi_services::core::trips::TripGetInput {
            id: trip.id,
            entries_limit: 100,
            entries_offset: 0,
        },
    )
    .unwrap()
    .entries
    .iter()
    .find(|e| e.id == sale.id)
    .unwrap()
    .version;
    let err = edit::trip_other_sale_save(
        ctx,
        &w.owner,
        &TripOtherSaleEditInput {
            version,
            ..edit_of("10", "6,000")
        },
    )
    .unwrap_err();
    let ServiceError::Rejected(items) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrEditNotAllowed, Some("qty"))
    );
    assert_eq!(
        items[0].values.get("rule"),
        Some(&ErrorValue::Text("carries_cost".into()))
    );
    assert!(books_balance(&w) && windows_empty(&w));
}

#[test]
fn the_cash_question_of_a_payment_edit_is_about_its_net_change() {
    // Cash 50,000. A purchase of 20 at 3,000 (60,000) is confirmed and takes cash to -10,000. Cutting it
    // to 2,500 (50,000) raises cash to 0: no question, in the review and in the save. Raising it to
    // 3,500 would take cash from -10,000 down to -20,000: it asks.
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let mut first = purchase_input(&w, trip.id, "20", "3,000", None);
    first.confirm_cash_below_zero = true;
    let saved = purchases::save(ctx, &w.owner, &first).unwrap();
    assert_eq!(cash_now(&w), -10_000);

    let cut = edit_input(
        saved.id,
        saved.version,
        purchase_input(&w, trip.id, "20", "2,500", None),
        "",
    );
    let preview = edit::purchase_preview(ctx, &cut).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let c = preview.review.effect.cash.as_ref().expect("cash moves");
    assert_eq!(
        (c.before_paisa.rupees(), c.after_paisa.rupees()),
        (-10_000, 0)
    );
    assert!(!c.needs_confirm);
    let done = edit::purchase_save(ctx, &w.owner, &cut).unwrap();
    assert_eq!(cash_now(&w), 0);

    let more = edit_input(
        saved.id,
        done.version,
        purchase_input(&w, trip.id, "20", "3,500", None),
        "",
    );
    let preview = edit::purchase_preview(ctx, &more).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(preview.review.effect.cash.as_ref().unwrap().needs_confirm);
    let err = edit::purchase_save(ctx, &w.owner, &more).unwrap_err();
    assert_eq!(code_of(&err).0, ErrorCode::ErrCashBelowZeroNotConfirmed);
    assert_eq!(cash_now(&w), 0);
    assert!(windows_empty(&w));
}

#[test]
fn a_garden_sale_with_an_arrival_gives_its_goods_back_and_is_final_once_others_moved() {
    let w = world();
    let ctx = &w.env.ctx;
    let garden = garden_of_two(&w);
    let arrival = mandi_services::gardens::arrivals::save(
        ctx,
        &w.owner,
        &arrival_input(&w, garden.id, "100", None),
    )
    .unwrap()
    .row;
    let sale = |qty: &str, amount: &str| {
        GardenEntryInput::OtherSale(GardenOtherSaleInput {
            garden_id: Some(garden.id),
            date: "10-10-2026".into(),
            shop: Some("حاجی کی دکان".into()),
            arrival_id: Some(arrival.id),
            qty: qty.into(),
            amount: amount.into(),
            note: Some("نوٹ".into()),
        })
    };
    let arrival_row = |w: &World| {
        mandi_services::gardens::arrivals::get(
            &w.env.ctx,
            &mandi_services::core::arrivals::ArrivalGetInput { id: arrival.id },
        )
        .unwrap()
    };
    let saved = gardens::entry_save(ctx, &w.owner, &sale("20", "10,000")).unwrap();
    assert_eq!(
        (arrival_row(&w).qty_sold, arrival_row(&w).qty_left),
        (20, 80)
    );
    assert_eq!(arrival_stock(&w, arrival.id), 80);
    // The history row of the sale gives what the edit form needs to open it again: its own number,
    // the arrival and the note (PRD FR-7.70).
    let history = |w: &World| {
        mandi_services::gardens::get(
            &w.env.ctx,
            &mandi_services::core::gardens::GardenGetInput {
                id: garden.id,
                history_offset: 0,
                history_limit: 0,
            },
        )
        .unwrap()
        .history
    };
    let rows = history(&w);
    let row = rows
        .iter()
        .find(|r| r.entry_id == Some(saved.id))
        .expect("the sale is in the history");
    assert_eq!(
        (row.number, row.arrival_id, row.note.as_deref()),
        (Some(1), Some(arrival.id), Some("نوٹ"))
    );
    // Every entry of the garden has its own number; only a sale in another shop has an arrival
    // and a note.
    for other in rows.iter().filter(|r| r.entry_id.is_some()) {
        assert!(other.number.is_some());
        if other.entry_id != Some(saved.id) {
            assert_eq!((other.arrival_id, other.note.as_deref()), (None, None));
        }
    }

    // 30 for 12,000: the arrival gets its 20 back and gives 30.
    let edited = GardenEntryEditInput {
        id: saved.id,
        version: saved.version,
        entry: sale("30", "12,000"),
        reason: String::new(),
        confirm_cash_below_zero: false,
    };
    let done = edit::garden_entry_save(ctx, &w.owner, &edited).unwrap();
    assert_eq!(
        (arrival_row(&w).qty_sold, arrival_row(&w).qty_left),
        (30, 70)
    );
    assert_eq!(arrival_stock(&w, arrival.id), 70);
    assert_eq!(credit_balance(&w, garden.account.id), 12_000);
    assert!(books_balance(&w) && windows_empty(&w));

    // Another sale of the same arrival makes the first one final (but it can still be cancelled).
    gardens::entry_save(ctx, &w.owner, &sale("5", "2,000")).unwrap();
    let err = edit::garden_entry_save(
        ctx,
        &w.owner,
        &GardenEntryEditInput {
            version: done.version,
            entry: sale("30", "12,500"),
            ..edited
        },
    )
    .unwrap_err();
    let ServiceError::Rejected(items) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(
        (items[0].code, items[0].field.as_deref()),
        (ErrorCode::ErrEditNotAllowed, Some("id"))
    );
    assert_eq!(
        items[0].values.get("rule"),
        Some(&ErrorValue::Text("goods_moved".into()))
    );
    assert!(windows_empty(&w));
}
