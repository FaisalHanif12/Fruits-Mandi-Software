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
        let card = bikri::goods_to_load(
            &w.env.ctx,
            &GoodsToLoadInput {
                kind,
                offset: 0,
                limit: 0,
            },
        )
        .unwrap();
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
            offset: 0,
            limit: 0,
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
            offset: 0,
            limit: 0,
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
            offset: 0,
            limit: 0,
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

/// Plan 80 R2-5 (PRD FR-12.50, FR-19.11, Code Standards B31): a card of goods to load comes a page at a
/// time with the count of the whole card, so every row can be reached (it stopped at 100 rows before).
/// Three arrivals of one garden, oldest day to sell first: page 1 of 2 rows, page 2 of 1 row, a page
/// past the end is empty, and the count is 3 in all of them.
#[test]
fn a_card_of_goods_is_paged_with_the_count_of_the_whole_card() {
    use mandi_services::core::arrivals::ArrivalInput;
    let w = worked_day();
    let (_account, first) = garden_with_arrival(&w, "10", "20-10-2026");
    for (qty, sell_date) in [("20", "15-10-2026"), ("30", "25-10-2026")] {
        mandi_services::gardens::arrivals::save(
            &w.env.ctx,
            &w.owner,
            &ArrivalInput {
                garden_id: Some(first.garden.id),
                date: "10-10-2026".into(),
                sell_date: Some(sell_date.into()),
                item_id: Some(w.banana),
                packing_id: Some(w.box_),
                qty: qty.into(),
                weight: None,
                note: None,
            },
        )
        .unwrap();
    }
    let page = |offset: u32, limit: u32| {
        let card = bikri::goods_to_load(
            &w.env.ctx,
            &GoodsToLoadInput {
                kind: VentureKind::Garden,
                offset,
                limit,
            },
        )
        .unwrap();
        (
            card.rows
                .iter()
                .map(|r| (r.sell_date.clone(), r.qty_left))
                .collect::<Vec<_>>(),
            card.total_count,
        )
    };
    let all = page(0, 0);
    assert_eq!(
        all,
        (
            vec![
                ("2026-10-15".to_owned(), 20),
                ("2026-10-20".to_owned(), 10),
                ("2026-10-25".to_owned(), 30)
            ],
            3
        )
    );
    assert_eq!(
        page(0, 2),
        (
            vec![("2026-10-15".to_owned(), 20), ("2026-10-20".to_owned(), 10)],
            3
        )
    );
    assert_eq!(page(2, 2), (vec![("2026-10-25".to_owned(), 30)], 3));
    assert_eq!(page(3, 2), (Vec::new(), 3));
}
