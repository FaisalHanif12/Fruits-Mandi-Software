//! Plan 53 Section 3 (reworded by the lead on 04-11-2026): the collection list and R20 (FR-19.9)
//! agree while only regular, active buyers owe. R20 counts every person who owes the shop; the
//! collection list is a list of regular, active buyers to collect from (plan 53 Section 4), so it
//! leaves out a چکتا buyer of the shared چکتا کھاتہ and an inactive buyer.
//!
//! The worked day after Events 1 to 5: اکرم 2,880 and ارشد 18,684 owe, together 21,564 (PRD 4.9).
//! Then E2E-21's ناصر buys 2 × 1,000 on credit through the shared چکتا کھاتہ: R20 is 23,564, the
//! list stays 21,564. The difference is exactly his 2,000.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::reports::{ReportFilters, ReportId};
use mandi_services::core::bikri::{ChuktaInput, LineChargeMethod};
use mandi_services::core::vouchers::CollectionListInput;
use mandi_services::vouchers;
use reports_world::*;

/// The total of the collection list over the whole filter, in whole rupees.
fn list_total(w: &support::World) -> i64 {
    vouchers::collection_list(
        &w.env.ctx,
        &CollectionListInput {
            search: None,
            collector_staff_id: None,
            city_id: None,
            offset: 0,
            limit: 100,
        },
    )
    .unwrap()
    .total_paisa
    .rupees()
}

/// The total row of R20, the balance column: the shop's total اگراہی (FR-19.9).
fn r20_total(w: &support::World) -> i64 {
    let page = report(w, ReportId::R20, ReportFilters::default(), 0, 0);
    page.totals.get(3).map_or(0, money)
}

#[test]
fn r20_equals_the_list_total_and_a_chukta_buyer_adds_nothing() {
    let w = full_day();
    // Only regular, active buyers owe: the list and R20 are the same number.
    assert_eq!(list_total(&w), 21_564);
    assert_eq!(r20_total(&w), 21_564);

    // E2E-21 (v1.32): ناصر (a چکتا buyer) takes 2 × 1,000 and pays it in full on the spot, with no
    // commission or لاگا. He owes nothing, so neither the list nor R20 moves (PRD FR-13.6).
    let mut sale = support::sale("s-nasir", w.chukta_khata, "2", "1000");
    sale.chukta = Some(ChuktaInput {
        name_ur: "ناصر".into(),
        name_en: "Nasir".into(),
        phone: "0300-1111111".into(),
    });
    sale.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
    sale.seller_comm = support::typed(LineChargeMethod::Percent, "0");
    sale.laga = support::typed(LineChargeMethod::PerUnit, "0");
    let mut lot = support::bare_lot(&w, "lot-nasir", w.rahim, w.banana, "2");
    lot.sales = vec![sale];
    save(&w, &support::draft(vec![lot]));

    assert_eq!(list_total(&w), 21_564);
    assert_eq!(r20_total(&w), 21_564);
    assert_eq!(r20_total(&w), list_total(&w));
}
