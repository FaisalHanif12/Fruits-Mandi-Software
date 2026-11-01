//! Migration 14 (CLAUDE.md rule 4, PRD FR-7.70): the edit of a trip or garden entry RETIRES the old
//! parts (`is_active = 0`) and writes new ones; nothing is deleted. These tests edit entries twice and
//! prove what the money agent asked for: the active parts always add up to the entry, every retired
//! part is linked to its successor, what a reader shows after the edits is exactly what a fresh save
//! of the final values shows (no part is counted twice), and cancelling an edited trip cost gives every
//! purchase its cost back exactly.
//!
//! The numbers are E2E-34 (a trip of three partners at 40 / 30 / 30, 20 at 500 and freight 400) and
//! E2E-40 (a garden of two, 60% from the shop's cash and 40% from a pocket).
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::raw;
use mandi_services::core::edit::{
    GardenEntryEditInput, PurchaseEditInput, TripCostEditInput, TripOtherSaleEditInput,
};
use mandi_services::core::gardens::{
    GardenEntryInput, GardenGetInput, GardenInput, GardenPaymentInput,
};
use mandi_services::core::purchases::{PurchaseGetInput, PurchaseInput};
use mandi_services::core::trips::{
    OtherSaleInput, TripCostInput, TripEntryCancelInput, TripGetInput, TripInput, TripRow,
};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{edit, gardens, purchases, trips};
use serde_json::Value;
use support::{World, world};

fn count(w: &World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

fn partner(partner_id: i64, share: &str, source: MoneySource) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source,
    }
}

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
            partners: vec![
                partner(a, "40", MoneySource::Shop),
                partner(b, "30", MoneySource::Shop),
                partner(c, "30", MoneySource::Shop),
            ],
        },
    )
    .unwrap()
}

fn purchase_input(w: &World, trip: i64, rate: &str) -> PurchaseInput {
    PurchaseInput {
        other_qty: None,
        shop_name: None,
        sell_date: None,
        trip_id: Some(trip),
        date: "10-10-2026".into(),
        supplier: Some("Orange Traders".into()),
        item_id: Some(w.banana),
        packing_id: Some(w.box_),
        qty: "20".into(),
        weight: None,
        rate: rate.into(),
        freight: Some("400".into()),
        labour: None,
        other_cost: None,
        note: None,
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    }
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

/// The parts of one entry: (active count, retired count, active sum in paisa, retired with no link).
fn venture_parts(w: &World, entry_id: i64) -> (i64, i64, i64, i64) {
    let q = |sql: &str| count(w, &sql.replace("{e}", &entry_id.to_string()));
    (
        q("SELECT COUNT(*) FROM venture_entry_parts WHERE entry_id = {e} AND is_active = 1"),
        q("SELECT COUNT(*) FROM venture_entry_parts WHERE entry_id = {e} AND is_active = 0"),
        q(
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM venture_entry_parts WHERE entry_id = {e} AND is_active = 1",
        ),
        q(
            "SELECT COUNT(*) FROM venture_entry_parts WHERE entry_id = {e} AND is_active = 0 AND replaced_by IS NULL",
        ),
    )
}

fn entry_amount(w: &World, entry_id: i64) -> i64 {
    count(
        w,
        &format!("SELECT amount_paisa FROM venture_entries WHERE id = {entry_id}"),
    )
}

fn purchase_total(w: &World, id: i64) -> i64 {
    purchases::get(&w.env.ctx, &PurchaseGetInput { id })
        .unwrap()
        .total_cost_paisa
        .rupees()
}

fn trip_results(w: &World, trip_id: i64) -> Value {
    let detail = trips::get(
        &w.env.ctx,
        &TripGetInput {
            id: trip_id,
            entries_offset: 0,
            entries_limit: 0,
        },
    )
    .unwrap();
    serde_json::to_value(&detail.partner_results).unwrap()
}

#[test]
fn a_purchase_edited_twice_keeps_its_parts_whole_its_history_linked_and_its_readers_true() {
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let saved = purchases::save(ctx, &w.owner, &purchase_input(&w, trip.id, "500")).unwrap();
    let entry: i64 = count(
        &w,
        &format!(
            "SELECT id FROM venture_entries WHERE purchase_id = {}",
            saved.id
        ),
    );
    assert_eq!(venture_parts(&w, entry), (3, 0, 1_040_000, 0));

    let mut version = saved.version;
    for (round, rate) in [(1, "520"), (2, "540")] {
        let done = edit::purchase_save(
            ctx,
            &w.owner,
            &PurchaseEditInput {
                id: saved.id,
                version,
                purchase: purchase_input(&w, trip.id, rate),
                reason: "ریٹ".into(),
            },
        )
        .unwrap();
        version = done.version;
        // One active part for each partner, adding up to the entry; every part an edit retired is
        // linked to the part that replaced it, and none is deleted.
        let (active, retired, sum, unlinked) = venture_parts(&w, entry);
        assert_eq!(
            (active, retired, unlinked),
            (3, 3 * round, 0),
            "round {round}"
        );
        assert_eq!(sum, entry_amount(&w, entry), "round {round}");
    }
    // 20 x 540 + 400 freight = 11,200 at 40 / 30 / 30.
    assert_eq!(entry_amount(&w, entry), 1_120_000);
    // The retired rows form chains: each points at a row that is active or itself retired and linked.
    assert_eq!(
        count(
            &w,
            "SELECT COUNT(*) FROM venture_entry_parts a JOIN venture_entry_parts b ON b.id = a.replaced_by
             WHERE a.is_active = 0 AND b.is_active = 0 AND b.replaced_by IS NULL"
        ),
        0,
        "a link never ends on a retired row that has no successor"
    );
    // The audit row of the last edit holds exactly one part for each partner as the NEW value, and
    // the first edit's new value is what the second edit's old value was.
    let news: Vec<String> = raw(&w.dir)
        .prepare("SELECT new_json FROM audit_log WHERE entity = 'purchase' AND action = 'edit' ORDER BY id")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(news.len(), 2);
    let last: Value = serde_json::from_str(&news[1]).unwrap();
    assert_eq!(
        last["row"]["parts"].as_array().map(Vec::len),
        Some(3),
        "{last}"
    );

    // What the readers show after two edits is what a fresh save of the final values shows.
    let fresh = world();
    let fresh_trip = trip_of_three(&fresh);
    purchases::save(
        &fresh.env.ctx,
        &fresh.owner,
        &purchase_input(&fresh, fresh_trip.id, "540"),
    )
    .unwrap();
    assert_eq!(
        trip_results(&w, trip.id),
        trip_results(&fresh, fresh_trip.id)
    );
}

#[test]
fn a_trip_cost_edited_twice_then_cancelled_gives_every_purchase_its_cost_back() {
    // The path the money agent called the worst: the cancel reads the spread and takes each part off
    // its purchase. If a retired part were read, a second edit and the cancel would take the old
    // spread off twice and the purchase's stored cost would be too low.
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let bought = purchases::save(ctx, &w.owner, &purchase_input(&w, trip.id, "500")).unwrap();
    assert_eq!(purchase_total(&w, bought.id), 10_400);
    let cost = trips::cost_save(ctx, &w.owner, &cost_input(trip.id, "4,000")).unwrap();
    assert_eq!(purchase_total(&w, bought.id), 14_400);
    let entry = cost.id;

    let mut version = cost.version;
    for (round, amount, total) in [(1, "4,400", 14_800), (2, "4,800", 15_200)] {
        let done = edit::trip_cost_save(
            ctx,
            &w.owner,
            &TripCostEditInput {
                id: entry,
                version,
                cost: cost_input(trip.id, amount),
                reason: String::new(),
            },
        )
        .unwrap();
        version = done.version;
        assert_eq!(purchase_total(&w, bought.id), total, "round {round}");
        // The spread adds up to the cost; the partners' parts too.
        let spread_sum = count(
            &w,
            &format!(
                "SELECT COALESCE(SUM(amount_paisa), 0) FROM purchase_cost_parts WHERE entry_id = {entry} AND is_active = 1"
            ),
        );
        assert_eq!(spread_sum, entry_amount(&w, entry), "round {round}");
        let (active, retired, sum, unlinked) = venture_parts(&w, entry);
        assert_eq!(
            (active, retired, unlinked),
            (3, 3 * round, 0),
            "round {round}"
        );
        assert_eq!(sum, entry_amount(&w, entry));
        // The purchase's stored trip cost is the sum of the ACTIVE parts of the saved costs.
        assert_eq!(
            count(
                &w,
                &format!(
                    "SELECT trip_cost_paisa FROM purchases WHERE id = {}",
                    bought.id
                )
            ),
            count(
                &w,
                "SELECT COALESCE(SUM(c.amount_paisa), 0) FROM purchase_cost_parts c
                 JOIN venture_entries x ON x.id = c.entry_id
                 WHERE x.status = 'saved' AND c.is_active = 1"
            )
        );
        assert_eq!(
            count(
                &w,
                &format!(
                    "SELECT COUNT(*) FROM purchase_cost_parts WHERE entry_id = {entry} AND is_active = 0"
                )
            ),
            round,
            "one retired spread row for the purchase in each edit"
        );
    }
    // Cancel the edited cost: the purchase is exactly as it was before the cost.
    trips::cost_cancel(
        ctx,
        &w.owner,
        &TripEntryCancelInput {
            id: entry,
            version,
            reason: "غلط".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    assert_eq!(purchase_total(&w, bought.id), 10_400);
    assert_eq!(
        count(
            &w,
            &format!(
                "SELECT trip_cost_paisa FROM purchases WHERE id = {}",
                bought.id
            )
        ),
        0
    );
    // Nothing was deleted by any of it.
    assert_eq!(count(&w, "SELECT COUNT(*) FROM purchase_cost_parts"), 3);
    assert_eq!(count(&w, "SELECT COUNT(*) FROM edit_windows"), 0);
    // And the result of the trip equals a world that never had the cost.
    let fresh = world();
    let fresh_trip = trip_of_three(&fresh);
    purchases::save(
        &fresh.env.ctx,
        &fresh.owner,
        &purchase_input(&fresh, fresh_trip.id, "500"),
    )
    .unwrap();
    assert_eq!(
        trip_results(&w, trip.id),
        trip_results(&fresh, fresh_trip.id)
    );
}

fn other_sale(trip: i64, purchase: i64, qty: &str, amount: &str) -> OtherSaleInput {
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

#[test]
fn a_sale_in_another_shop_edited_twice_shows_the_same_money_back_as_a_fresh_save() {
    // E2E-34 purchase, 15 of the 20 sold in another shop for 8,460, then edited to 9,000 and to 9,500.
    // The money the partners got back (C25, `partner_money.back_other_paisa`) must be that of one fresh
    // sale of 9,500: 3,800 / 2,850 / 2,850, not those plus the retired parts.
    let w = world();
    let ctx = &w.env.ctx;
    let trip = trip_of_three(&w);
    let bought = purchases::save(ctx, &w.owner, &purchase_input(&w, trip.id, "500")).unwrap();
    let sale = trips::other_sale_save(
        ctx,
        &w.owner,
        &other_sale(trip.id, bought.id, "15", "8,460"),
    )
    .unwrap();
    let entry = sale.id;
    assert_eq!(venture_parts(&w, entry), (3, 0, 846_000, 0));
    let mut version = sale.version;
    for (round, amount) in [(1, "9,000"), (2, "9,500")] {
        let done = edit::trip_other_sale_save(
            ctx,
            &w.owner,
            &TripOtherSaleEditInput {
                id: entry,
                version,
                sale: other_sale(trip.id, bought.id, "15", amount),
                reason: String::new(),
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
        version = done.version;
        let (active, retired, sum, unlinked) = venture_parts(&w, entry);
        assert_eq!(
            (active, retired, unlinked),
            (3, 3 * round, 0),
            "round {round}"
        );
        assert_eq!(sum, entry_amount(&w, entry), "round {round}");
    }
    assert_eq!(entry_amount(&w, entry), 950_000);
    let fresh = world();
    let fresh_trip = trip_of_three(&fresh);
    let fresh_bought = purchases::save(
        &fresh.env.ctx,
        &fresh.owner,
        &purchase_input(&fresh, fresh_trip.id, "500"),
    )
    .unwrap();
    trips::other_sale_save(
        &fresh.env.ctx,
        &fresh.owner,
        &other_sale(fresh_trip.id, fresh_bought.id, "15", "9,500"),
    )
    .unwrap();
    assert_eq!(
        trip_results(&w, trip.id),
        trip_results(&fresh, fresh_trip.id)
    );
}

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
            partners: vec![
                partner(a, "60", MoneySource::Shop),
                partner(b, "40", MoneySource::Pocket),
            ],
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .row
}

fn installment(garden: i64, amount: &str) -> GardenEntryInput {
    GardenEntryInput::Installment(GardenPaymentInput {
        garden_id: Some(garden),
        date: "10-10-2026".into(),
        amount: amount.into(),
        reason: "پہلی قسط".into(),
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    })
}

#[test]
fn a_garden_installment_edited_twice_shows_the_same_paid_and_results_as_a_fresh_save() {
    // E2E-40: 30,000 at 60 / 40 is edited to 32,000 and then to 34,000: parts 20,400 (shop) / 13,600
    // (pocket). The garden's "paid" and each partner's result must be those of one fresh save of
    // 34,000, not the sum of the retired and the active parts.
    let w = world();
    let ctx = &w.env.ctx;
    let garden = garden_of_two(&w);
    let saved = gardens::entry_save(ctx, &w.owner, &installment(garden.id, "30,000")).unwrap();
    let entry = saved.id;
    let mut version = saved.version;
    for (round, amount) in [(1, "32,000"), (2, "34,000")] {
        let done = edit::garden_entry_save(
            ctx,
            &w.owner,
            &GardenEntryEditInput {
                id: entry,
                version,
                entry: installment(garden.id, amount),
                reason: String::new(),
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
        version = done.version;
        let (active, retired, sum, unlinked) = venture_parts(&w, entry);
        assert_eq!(
            (active, retired, unlinked),
            (2, 2 * round, 0),
            "round {round}"
        );
        assert_eq!(sum, entry_amount(&w, entry), "round {round}");
    }
    assert_eq!(entry_amount(&w, entry), 3_400_000);

    let fresh = world();
    let fresh_garden = garden_of_two(&fresh);
    gardens::entry_save(
        &fresh.env.ctx,
        &fresh.owner,
        &installment(fresh_garden.id, "34,000"),
    )
    .unwrap();
    let results = |world: &World, id: i64| {
        let detail = gardens::get(
            &world.env.ctx,
            &GardenGetInput {
                id,
                history_offset: 0,
                history_limit: 0,
            },
        )
        .unwrap();
        serde_json::to_value(&detail.partner_results).unwrap()
    };
    assert_eq!(results(&w, garden.id), results(&fresh, fresh_garden.id));
    let money = |world: &World, id: i64| {
        let detail = gardens::get(
            &world.env.ctx,
            &GardenGetInput {
                id,
                history_offset: 0,
                history_limit: 0,
            },
        )
        .unwrap();
        (detail.row.paid_shop_paisa, detail.row.paid_pocket_paisa)
    };
    assert_eq!(money(&w, garden.id), money(&fresh, fresh_garden.id));
}
