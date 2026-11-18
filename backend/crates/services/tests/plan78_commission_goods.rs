//! The goods filter of the commission page (plan 78 D4, PRD FR-9.3 v1.43): all, consigned goods,
//! trip goods and garden goods, read from the same facts and the same SQL text as the goods filter of
//! analytics (PRD 18.1), so the page and analytics view A can never disagree.
//!
//! The world is the worked day's opening position with three kinds of goods on 10-10-26 and 11-10-26:
//! * consigned, بکری 1 of the worked day: seller 1,764, buyer 1,764, لاگا 400 = 3,928;
//! * trip goods, E2E-34: 15 پیٹی مالٹا at 600: seller 540, buyer 540, لاگا 120 = 1,200;
//! * garden goods, 10 پیٹی کیلا at 600 from a garden's lot: seller 360, buyer 360, لاگا 80 = 800.
//!
//! The three kinds add to all goods: seller 2,664, buyer 2,664, لاگا 600, total 5,928.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_reports::analytics::{
    AnalyticsFilters, AnalyticsGoods, AnalyticsRunInput, AnalyticsView, run,
};
use mandi_reports::dashboard::{
    self, CommissionDayInput, CommissionMonthInput, CommissionPrintInput,
};
use mandi_services::core::auth::Language;
use reports_world::*;
use support::World;

const ALL_KINDS: [AnalyticsGoods; 3] = [
    AnalyticsGoods::Consigned,
    AnalyticsGoods::Trip,
    AnalyticsGoods::Garden,
];

/// The world with the three kinds of goods, today 11-10-26.
fn world_of_three() -> World {
    let (w, _trip, _purchase, _sale) = trip_34_world();
    save(&w, &support::event1(&w));
    let garden = garden_made(&w);
    set_today(&w.env, 2026, 10, 11);
    garden_lot_sale(&w, garden.account.id, "11-10-2026", "10", w.arshad, "600");
    w
}

fn month(w: &World, goods: Option<AnalyticsGoods>) -> dashboard::CommissionMonth {
    dashboard::commission_month(
        w.env.ctx.db(),
        &CommissionMonthInput {
            month: None,
            period: None,
            party: None,
            goods,
            offset: None,
            limit: None,
        },
    )
    .unwrap()
}

/// (seller, buyer, laga, total) of a totals row, in whole rupees.
fn figures(row: &dashboard::CommissionDayRow) -> (i64, i64, i64, i64) {
    (
        row.seller_comm_paisa.rupees(),
        row.buyer_comm_paisa.rupees(),
        row.laga_paisa.rupees(),
        row.total_comm_paisa.rupees(),
    )
}

#[test]
fn the_three_kinds_of_goods_add_up_to_all_goods_with_the_worked_numbers() {
    let w = world_of_three();
    let all = month(&w, None);
    assert_eq!(figures(&all.totals), (2_664, 2_664, 600, 5_928));
    assert_eq!(month(&w, Some(AnalyticsGoods::All)).totals, all.totals);
    assert_eq!(
        figures(&month(&w, Some(AnalyticsGoods::Consigned)).totals),
        (1_764, 1_764, 400, 3_928)
    );
    assert_eq!(
        figures(&month(&w, Some(AnalyticsGoods::Trip)).totals),
        (540, 540, 120, 1_200)
    );
    assert_eq!(
        figures(&month(&w, Some(AnalyticsGoods::Garden)).totals),
        (360, 360, 80, 800)
    );
    // Day by day, too: the same partition on every date.
    for row in &all.rows {
        let mut sum = (0, 0, 0, 0);
        for goods in ALL_KINDS {
            let page = month(&w, Some(goods));
            if let Some(r) = page.rows.iter().find(|r| r.date == row.date) {
                let f = figures(r);
                sum = (sum.0 + f.0, sum.1 + f.1, sum.2 + f.2, sum.3 + f.3);
            }
        }
        assert_eq!(sum, figures(row), "{}", row.date);
    }
}

#[test]
fn a_goods_filter_has_no_partner_columns_and_no_shared_income() {
    let w = world_of_three();
    let all = month(&w, None);
    assert_eq!(all.partners.len(), 3);
    assert!(all.totals.shared_income_paisa.rupees() > 0);
    for goods in ALL_KINDS {
        let page = month(&w, Some(goods));
        assert_eq!(page.goods, goods);
        assert!(page.partners.is_empty(), "{goods:?}");
        assert!(!page.partners_locked);
        for row in page.rows.iter().chain([&page.totals]) {
            assert_eq!(row.shared_income_paisa.rupees(), 0, "{goods:?}");
            assert!(row.partner_shares.as_ref().is_none_or(Vec::is_empty));
        }
    }
}

#[test]
fn the_page_agrees_with_analytics_view_a_under_every_goods_filter() {
    let w = world_of_three();
    for goods in ALL_KINDS.into_iter().chain([AnalyticsGoods::All]) {
        let page = month(&w, Some(goods));
        let view = run(
            w.env.ctx.db(),
            &AnalyticsRunInput {
                sort: None,
                view: AnalyticsView::A,
                filters: AnalyticsFilters {
                    goods,
                    ..AnalyticsFilters::default()
                },
                offset: 0,
                limit: 0,
            },
        )
        .unwrap();
        let at = view
            .columns
            .iter()
            .position(|c| c.key == "total_comm")
            .unwrap();
        let total = match &view.totals[at] {
            mandi_reports::reports::Cell::Money { paisa } => paisa.rupees(),
            other => panic!("{other:?}"),
        };
        assert_eq!(page.totals.total_comm_paisa.rupees(), total, "{goods:?}");
    }
}

#[test]
fn the_day_detail_and_the_print_follow_the_goods_filter() {
    let w = world_of_three();
    let day = |date: &str, goods: AnalyticsGoods| {
        dashboard::commission_day(
            w.env.ctx.db(),
            &CommissionDayInput {
                date: date.into(),
                offset: 0,
                limit: 0,
                party: None,
                goods: Some(goods),
            },
        )
        .unwrap()
    };
    // 10-10-26 holds the consigned and the trip postings; the garden sale is of 11-10-26.
    assert_eq!(
        figures(&day("2026-10-10", AnalyticsGoods::Trip).totals).3,
        1_200
    );
    assert_eq!(day("2026-10-10", AnalyticsGoods::Garden).total_count, 0);
    let garden = day("2026-10-11", AnalyticsGoods::Garden);
    assert_eq!(garden.total_count, 1);
    assert_eq!(garden.goods, AnalyticsGoods::Garden);
    assert!(garden.partners.is_empty());
    assert_eq!(figures(&garden.totals).3, 800);

    // The print: no shared income column under a goods filter, and the header names the goods.
    let spec = |goods: Option<AnalyticsGoods>| {
        dashboard::commission_month_spec(
            w.env.ctx.db(),
            &CommissionPrintInput {
                month: None,
                period: None,
                party: None,
                goods,
                language: Language::En,
                paper: None,
            },
            true,
        )
        .unwrap()
    };
    let all = spec(None);
    assert!(all.columns.iter().any(|c| c.key == "shared_income"));
    assert!(all.facts.is_empty() || all.facts.iter().all(|f| !f.label_id.contains("goods")));
    let trip = spec(Some(AnalyticsGoods::Trip));
    assert!(trip.columns.iter().all(|c| c.key != "shared_income"));
    assert!(trip.columns.iter().all(|c| !c.key.starts_with("partner_")));
    assert!(
        trip.facts
            .iter()
            .any(|f| f.label_id == "analytics-filter-goods-trip")
    );
}
