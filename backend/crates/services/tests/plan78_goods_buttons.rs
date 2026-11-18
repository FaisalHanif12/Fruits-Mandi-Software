//! The three load buttons of the بکری screen (plan 78 C1, PRD FR-12.50): each shows only when it has
//! goods. `bikri_goods_sources` counts them and `bikri_goods_to_load` lists the cards of trip goods
//! and garden goods on ANY day to sell (the old ready lots wanted the day to sell to be due).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::bikri;
use mandi_services::core::bikri::GoodsToLoadInput;
use mandi_services::core::ventures::VentureKind;
use reports_world::*;

fn sources(w: &support::World) -> (i64, i64, i64) {
    let s = bikri::goods_sources(&w.env.ctx).unwrap();
    (s.stock_lots, s.trip_rows, s.garden_rows)
}

#[test]
fn a_button_has_a_count_only_when_its_kind_of_goods_is_left() {
    // The worked day's opening position: no lot, no trip, no garden.
    let w = worked_day();
    assert_eq!(sources(&w), (0, 0, 0));
    for kind in [VentureKind::Trip, VentureKind::Garden] {
        let card = bikri::goods_to_load(&w.env.ctx, &GoodsToLoadInput { kind }).unwrap();
        assert!(card.rows.is_empty(), "{kind:?}");
    }

    // A lot of رحیم with 20 of 50 sold: 30 left in باقی مال. Nothing else.
    let lot = support::with_sales(
        support::bare_lot(&w, "l-1", w.rahim, w.banana, "50"),
        vec![support::sale("s-1", w.akram, "20", "600")],
    );
    save(&w, &support::draft(vec![lot]));
    assert_eq!(sources(&w), (1, 0, 0));

    // A garden arrival of 100 پیٹی with a day to sell far in the future: the garden button shows now,
    // and its card lists it (the old ready lots did not list it before its day).
    let (_account, arrival) = garden_with_arrival(&w, "100", "31-10-2026");
    assert_eq!(sources(&w), (1, 0, 1));
    let garden = bikri::goods_to_load(
        &w.env.ctx,
        &GoodsToLoadInput {
            kind: VentureKind::Garden,
        },
    )
    .unwrap();
    assert_eq!(garden.rows.len(), 1);
    assert_eq!(garden.rows[0].id, arrival.id);
    assert_eq!(garden.rows[0].qty_left, 100);
    assert_eq!(garden.rows[0].sell_date, "2026-10-31");
    assert!(garden.rows[0].garden.is_some());
    // The ready lots of the old button still list nothing for that day (the date is not due).
    let ready = bikri::ready_goods(
        &w.env.ctx,
        &mandi_services::core::bikri::ReadyGoodsInput {
            date: "10-10-2026".into(),
        },
    )
    .unwrap();
    assert!(ready.rows.is_empty());
    // The trip card is empty: no trip has goods.
    let trip = bikri::goods_to_load(
        &w.env.ctx,
        &GoodsToLoadInput {
            kind: VentureKind::Trip,
        },
    )
    .unwrap();
    assert!(trip.rows.is_empty());
}

#[test]
fn the_trip_card_lists_a_purchase_with_goods_left() {
    // E2E-34: 20 پیٹی مالٹا bought, 15 sold in a بکری: 5 are left for a later sale.
    let (w, _trip, purchase, _sale) = trip_34_world();
    assert_eq!(sources(&w), (0, 1, 0));
    let card = bikri::goods_to_load(
        &w.env.ctx,
        &GoodsToLoadInput {
            kind: VentureKind::Trip,
        },
    )
    .unwrap();
    assert_eq!(card.rows.len(), 1);
    assert_eq!(card.rows[0].id, purchase);
    assert_eq!(card.rows[0].qty, 20);
    assert_eq!(card.rows[0].qty_left, 5);
    assert!(card.rows[0].trip.is_some());
    // An own lot never has a balance in باقی مال: the count of lots stays 0.
    assert_eq!(sources(&w).0, 0);
}
