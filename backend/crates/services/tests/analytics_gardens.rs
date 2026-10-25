//! Gardens in the analytics, the day summary and the dashboard of plan d5-late, round 11 (PRD 18.2
//! view I, the gardens columns of views F and G, 19.2 R34, 9.1 the garden tile; E2E-40, E2E-40b).
//!
//! The expected numbers are the PRD's and the money agent's worked cases. Every garden figure is
//! also held against the garden screen (`services::gardens::get` and `summary`), so a report, the
//! analytics, the tile and the screen cannot disagree.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::analytics::{
    AnalyticsFilters, AnalyticsGoods, AnalyticsGroupBy, AnalyticsLinesInput, AnalyticsPage,
    AnalyticsRunInput, AnalyticsSide, AnalyticsView, run,
};
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_reports::reports::{Cell, ReportFilters, ReportId, RowKind};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::GardenGetInput;
use mandi_services::gardens;
use reports_world::*;
use support::World;

fn filters() -> AnalyticsFilters {
    AnalyticsFilters::default()
}

fn view(w: &World, v: AnalyticsView, f: AnalyticsFilters) -> AnalyticsPage {
    run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: v,
            filters: f,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
}

fn refused(w: &World, v: AnalyticsView, f: AnalyticsFilters) -> Vec<(ErrorCode, String)> {
    match run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            view: v,
            filters: f,
            offset: 0,
            limit: 0,
        },
    ) {
        Err(mandi_reports::ReportsError::Rejected(items)) => {
            items.into_iter().map(|i| (i.code, i.field)).collect()
        }
        other => panic!("expected a rejection, got {other:?}"),
    }
}

fn n(cell: &Cell) -> i64 {
    match cell {
        Cell::Number { value } => *value,
        Cell::Money { paisa } => paisa.rupees(),
        Cell::Percent { hundredths } => *hundredths,
        Cell::Empty => 0,
        other => panic!("not a number: {other:?}"),
    }
}

fn data_rows(page: &AnalyticsPage) -> Vec<&mandi_reports::analytics::AnalyticsRow> {
    page.rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .collect()
}

fn column(page: &AnalyticsPage, key: &str) -> usize {
    page.columns
        .iter()
        .position(|c| c.key == key)
        .unwrap_or_else(|| panic!("no column {key}"))
}

fn on(from: &str, to: &str) -> AnalyticsFilters {
    AnalyticsFilters {
        from: from.into(),
        to: to.into(),
        ..filters()
    }
}

fn daily() -> AnalyticsFilters {
    AnalyticsFilters {
        group_by: Some(AnalyticsGroupBy::Day),
        ..filters()
    }
}

/// The money columns of the first row of view I, from paid_shop to result.
fn i_figures(page: &AnalyticsPage) -> Vec<i64> {
    data_rows(page)[0].cells[2..9].iter().map(n).collect()
}

/// (share, result) of each partner column pair of the first row of view I.
fn i_partners(page: &AnalyticsPage) -> Vec<(i64, i64)> {
    data_rows(page)[0].cells[9..]
        .chunks(2)
        .map(|pair| (n(&pair[0]), n(&pair[1])))
        .collect()
}

fn g_gardens(page: &AnalyticsPage) -> Vec<i64> {
    let at = column(page, "gardens_result");
    data_rows(page).iter().map(|r| n(&r.cells[at])).collect()
}

fn f_gardens_of(w: &World, partner: i64, from: &str, to: &str) -> i64 {
    let page = view(
        w,
        AnalyticsView::F,
        AnalyticsFilters {
            partner_id: Some(partner),
            ..on(from, to)
        },
    );
    n(&data_rows(&page)[0].cells[column(&page, "gardens")])
}

fn off() -> BackupFacts {
    BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    }
}

#[test]
fn view_i_e2e_40_before_the_closing_is_the_running_balance_and_the_partners_lose_what_they_paid() {
    let (w, _) = garden_40();
    let page = view(&w, AnalyticsView::I, filters());
    assert_eq!(data_rows(&page).len(), 1);
    // paid shop, paid pocket, our shop, other shops, other in, commission, result.
    assert_eq!(
        i_figures(&page),
        [33_000, 22_000, 56_400, 10_000, 0, 8_000, 11_400]
    );
    assert_eq!(i_partners(&page), [(6_000, -33_000), (4_000, -22_000)]);
    // The commission follows the side filter, and only that.
    let seller = view(
        &w,
        AnalyticsView::I,
        AnalyticsFilters {
            side: AnalyticsSide::Seller,
            ..filters()
        },
    );
    assert_eq!(i_figures(&seller)[5], 3_600);
}

#[test]
fn view_i_e2e_40_after_the_closing_the_results_add_up_to_the_result() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let page = view(&w, AnalyticsView::I, filters());
    assert_eq!(
        i_figures(&page),
        [33_000, 22_000, 56_400, 10_000, 0, 8_000, 11_400]
    );
    assert_eq!(i_partners(&page), [(6_000, 6_840), (4_000, 4_560)]);
    // The totals row, and the results of the partners add up to the result.
    assert_eq!(
        page.totals[2..9].iter().map(n).collect::<Vec<_>>(),
        [33_000, 22_000, 56_400, 10_000, 0, 8_000, 11_400]
    );
    assert_eq!(n(&page.totals[10]) + n(&page.totals[12]), 11_400);
    // The row opens the sale lines of the garden's goods, whatever the period.
    let key = data_rows(&page)[0].key.clone().unwrap();
    let behind = mandi_reports::analytics::lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view: AnalyticsView::I,
            filters: on("10-10-2026", "10-10-2026"),
            row_key: key,
            column_key: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(behind.total_count, 1);
    assert_eq!(n(behind.totals.last().unwrap()), 8_000);
}

#[test]
fn view_i_e2e_40b_the_loss_and_the_results() {
    let (w, garden) = garden_40b();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let page = view(&w, AnalyticsView::I, filters());
    assert_eq!(i_figures(&page), [33_000, 22_000, 0, 10_000, 0, 0, -45_000]);
    assert_eq!(i_partners(&page), [(6_000, -27_000), (4_000, -18_000)]);
}

#[test]
fn view_i_filters_pick_gardens_and_refuse_what_it_does_not_take() {
    let (w, garden) = garden_40();
    let [_, _, c] = w.partners;
    assert_eq!(
        data_rows(&view(
            &w,
            AnalyticsView::I,
            AnalyticsFilters {
                garden_id: Some(garden.id),
                ..filters()
            }
        ))
        .len(),
        1
    );
    // ج is in no garden; the apple is not the garden's main fruit.
    assert_eq!(
        data_rows(&view(
            &w,
            AnalyticsView::I,
            AnalyticsFilters {
                partner_id: Some(c),
                ..filters()
            }
        ))
        .len(),
        0
    );
    assert_eq!(
        data_rows(&view(
            &w,
            AnalyticsView::I,
            AnalyticsFilters {
                item_id: Some(w.apple),
                ..filters()
            }
        ))
        .len(),
        0
    );
    // Trip or consigned goods have no gardens.
    for goods in [AnalyticsGoods::Trip, AnalyticsGoods::Consigned] {
        assert_eq!(
            refused(
                &w,
                AnalyticsView::I,
                AnalyticsFilters { goods, ..filters() }
            ),
            vec![(ErrorCode::ErrOutOfRange, "filters.goods".to_owned())]
        );
    }
    assert_eq!(
        refused(
            &w,
            AnalyticsView::I,
            AnalyticsFilters {
                seller_id: Some(w.rahim),
                ..filters()
            }
        ),
        vec![(ErrorCode::ErrOutOfRange, "filters.seller_id".to_owned())]
    );
    // A garden filter is not taken by F and J (they count no sales, or trips).
    for v in [AnalyticsView::F, AnalyticsView::J] {
        assert_eq!(
            refused(
                &w,
                v,
                AnalyticsFilters {
                    garden_id: Some(garden.id),
                    ..filters()
                }
            ),
            vec![(ErrorCode::ErrOutOfRange, "filters.garden_id".to_owned())],
            "{v:?}"
        );
    }
    assert_eq!(
        refused(
            &w,
            AnalyticsView::A,
            AnalyticsFilters {
                garden_id: Some(9_999),
                ..filters()
            }
        ),
        vec![(ErrorCode::ErrNotFound, "filters.garden_id".to_owned())]
    );
}

#[test]
fn view_i_agrees_with_the_garden_screen() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let page = view(&w, AnalyticsView::I, filters());
    let d = gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id: garden.id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap();
    assert_eq!(i_figures(&page)[0], d.row.paid_shop_paisa.rupees());
    assert_eq!(i_figures(&page)[1], d.row.paid_pocket_paisa.rupees());
    assert_eq!(i_figures(&page)[3], d.row.other_shop_paisa.rupees());
    assert_eq!(i_figures(&page)[6], d.row.balance_paisa.rupees());
    let results: Vec<i64> = d
        .partner_results
        .iter()
        .map(|r| r.net_paisa.rupees())
        .collect();
    let shown: Vec<i64> = i_partners(&page).iter().map(|p| p.1).collect();
    assert_eq!(shown, results);
}

#[test]
fn view_f_gardens_column_e2e_40_by_day() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let [a, b, c] = w.partners;
    // الف: the shop's parts -18,000, then -12,000 and -3,000, nothing on the sale days, then the
    // closing +39,840; lifetime +6,840.
    assert_eq!(f_gardens_of(&w, a, "10-10-2026", "10-10-2026"), -18_000);
    assert_eq!(f_gardens_of(&w, a, "11-10-2026", "11-10-2026"), -15_000);
    assert_eq!(f_gardens_of(&w, a, "12-10-2026", "12-10-2026"), 0);
    assert_eq!(f_gardens_of(&w, a, "13-10-2026", "13-10-2026"), 39_840);
    assert_eq!(f_gardens_of(&w, a, "01-10-2026", "31-10-2026"), 6_840);
    // ب paid from his pocket, so only the closing is in the books; ج is in no garden.
    assert_eq!(f_gardens_of(&w, b, "01-10-2026", "12-10-2026"), 0);
    assert_eq!(f_gardens_of(&w, b, "13-10-2026", "13-10-2026"), 26_560);
    assert_eq!(f_gardens_of(&w, c, "01-10-2026", "31-10-2026"), 0);
    // The whole table: income 3,200 / 2,400 / 2,400 and the gardens money, net 10,040 / 28,960 /
    // 2,400 (the partner totals of E2E-40), and nothing is left over in "other".
    let page = view(&w, AnalyticsView::F, filters());
    let net = column(&page, "net_change");
    let other = column(&page, "other");
    let rows = data_rows(&page);
    assert_eq!(
        rows.iter().map(|r| n(&r.cells[net])).collect::<Vec<_>>(),
        [10_040, 28_960, 2_400]
    );
    assert!(rows.iter().all(|r| n(&r.cells[other]) == 0));
}

#[test]
fn view_f_gardens_column_in_a_loss_closing_is_a_debit() {
    // E2E-40b: الف -33,000 + 6,000 = -27,000; ب only the 4,000 back (his 22,000 was his pocket).
    let (w, garden) = garden_40b();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let [a, b, _] = w.partners;
    assert_eq!(f_gardens_of(&w, a, "01-10-2026", "31-10-2026"), -27_000);
    assert_eq!(f_gardens_of(&w, b, "01-10-2026", "31-10-2026"), 4_000);
}

#[test]
fn view_g_gardens_result_e2e_40_by_day_adds_up_to_the_balance() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let page = view(&w, AnalyticsView::G, daily());
    // 10-10 -30,000; 11-10 -25,000; 12-10 +66,400 (the sale 56,400 and the other shop 10,000); the
    // closing is not money in, so 13-10 has no row. They add up to the balance 11,400.
    assert_eq!(g_gardens(&page), [-30_000, -25_000, 66_400]);
    let at = column(&page, "gardens_result");
    assert_eq!(n(&page.totals[at]), 11_400);
    // With one garden picked, the same; with a trip or consigned goods, none.
    let one = view(
        &w,
        AnalyticsView::G,
        AnalyticsFilters {
            garden_id: Some(garden.id),
            ..daily()
        },
    );
    assert_eq!(g_gardens(&one), [-30_000, -25_000, 66_400]);
    let trip_goods = view(
        &w,
        AnalyticsView::G,
        AnalyticsFilters {
            goods: AnalyticsGoods::Trip,
            ..daily()
        },
    );
    assert!(
        data_rows(&trip_goods)
            .iter()
            .all(|r| matches!(r.cells[at], Cell::Empty))
    );
}

#[test]
fn view_g_gardens_result_e2e_40b() {
    let (w, garden) = garden_40b();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let page = view(&w, AnalyticsView::G, daily());
    assert_eq!(g_gardens(&page), [-30_000, -25_000, 10_000]);
    let at = column(&page, "gardens_result");
    assert_eq!(n(&page.totals[at]), -45_000);
}

#[test]
fn view_g_with_a_partner_has_his_garden_money_as_view_f() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let [a, _, _] = w.partners;
    let page = view(
        &w,
        AnalyticsView::G,
        AnalyticsFilters {
            partner_id: Some(a),
            ..daily()
        },
    );
    let at = column(&page, "gardens_result");
    // Named as the gardens column of view F when a partner is picked.
    assert_eq!(page.columns[at].title_id, "analytics-col-gardens");
    assert_eq!(n(&page.totals[at]), 6_840);
    assert_eq!(
        n(&page.totals[at]),
        f_gardens_of(&w, a, "01-10-2026", "31-10-2026")
    );
}

#[test]
fn the_garden_filter_and_garden_goods_narrow_views_a_to_e_and_h() {
    let (w, garden) = garden_40();
    // A consigned sale as well.
    save(&w, &support::event1(&w));
    let gross = |p: &AnalyticsPage| n(&p.totals[column(p, "gross")]);
    let run_a = |f: AnalyticsFilters| view(&w, AnalyticsView::A, f);
    // The garden's 100 پیٹی at 600 = 60,000; the consigned 29,400.
    assert_eq!(gross(&run_a(filters())), 89_400);
    assert_eq!(
        gross(&run_a(AnalyticsFilters {
            goods: AnalyticsGoods::Garden,
            ..filters()
        })),
        60_000
    );
    assert_eq!(
        gross(&run_a(AnalyticsFilters {
            garden_id: Some(garden.id),
            ..filters()
        })),
        60_000
    );
    assert_eq!(
        gross(&run_a(AnalyticsFilters {
            goods: AnalyticsGoods::Consigned,
            ..filters()
        })),
        29_400
    );
}

// ---- R34, the day summary --------------------------------------------------------------------------

/// (count, amount) of a labelled row of R34 on one day.
fn r34(w: &World, day: &str, label: &str) -> (i64, i64) {
    let page = report(
        w,
        ReportId::R34,
        ReportFilters {
            from: day.into(),
            to: day.into(),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    let row = page
        .rows
        .iter()
        .find(|r| matches!(&r.cells[1], Cell::Label { id } if id == label))
        .unwrap_or_else(|| panic!("no row {label} on {day}"));
    (n(&row.cells[2]), n(&row.cells[3]))
}

#[test]
fn r34_shows_what_was_paid_into_the_garden_and_what_came_in_each_day() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let paid = "report-r34-gardens-paid";
    let money_in = "report-r34-gardens-in";
    // Paid: 30,000 (one payment) on 10-10, 25,000 (two) on 11-10. In: the sale and the other shop
    // on 12-10. The closing day shows neither.
    assert_eq!(r34(&w, "10-10-2026", paid), (1, 30_000));
    assert_eq!(r34(&w, "11-10-2026", paid), (2, 25_000));
    assert_eq!(r34(&w, "12-10-2026", paid), (0, 0));
    assert_eq!(r34(&w, "10-10-2026", money_in).1, 0);
    assert_eq!(r34(&w, "12-10-2026", money_in).1, 66_400);
    assert_eq!(r34(&w, "13-10-2026", paid), (0, 0));
    assert_eq!(r34(&w, "13-10-2026", money_in).1, 0);
    // The garden rows are for information: the cash lines still come from the cash account.
    let cash_out_10 = r34(&w, "10-10-2026", "report-r34-cash-out").1;
    assert!(cash_out_10 >= 18_000);
}

// ---- The dashboard tile ----------------------------------------------------------------------------

#[test]
fn the_garden_tile_is_the_garden_summary_of_the_screen() {
    let w = worked_day();
    // No garden: nothing on the tile.
    let none = dashboard::get(w.env.ctx.db(), off()).unwrap();
    assert_eq!((none.tiles.gardens_open, none.gardens.len()), (0, 0));
    assert_eq!(none.tiles.gardens_balance_paisa.rupees(), 0);

    // After the first installment only: balance -30,000, rent due 20,000.
    let garden = garden_made(&w);
    let d = dashboard::get(w.env.ctx.db(), off()).unwrap();
    assert_eq!(d.tiles.gardens_open, 1);
    assert_eq!(d.tiles.gardens_balance_paisa.rupees(), -30_000);
    assert_eq!(d.gardens[0].balance_paisa.rupees(), -30_000);
    assert_eq!(d.gardens[0].rent_due_paisa.rupees(), 20_000);

    // The rest of E2E-40: balance 11,400, rent due 0.
    set_today(&w.env, 2026, 10, 11);
    garden_pay(&w, garden.id, false, "11-10-2026", "20,000", "installment");
    garden_pay(&w, garden.id, true, "11-10-2026", "5,000", "labour");
    set_today(&w.env, 2026, 10, 12);
    garden_lot_sale(&w, garden.account.id, "12-10-2026", "100", w.arshad, "600");
    garden_other_sale(&w, garden.id, "12-10-2026", "20", "10,000");
    let d = dashboard::get(w.env.ctx.db(), off()).unwrap();
    let summary = gardens::summary(&w.env.ctx).unwrap();
    assert_eq!(d.tiles.gardens_open, summary.open_gardens);
    assert_eq!(
        d.tiles.gardens_balance_paisa.rupees(),
        summary.balance_total_paisa.rupees()
    );
    assert_eq!(d.tiles.gardens_balance_paisa.rupees(), 11_400);
    assert_eq!(d.gardens.len(), summary.rows.len());
    for (tile, screen) in d.gardens.iter().zip(&summary.rows) {
        assert_eq!(tile.id, screen.id);
        assert_eq!(tile.number, screen.number);
        assert_eq!(tile.balance_paisa, screen.balance_paisa);
        assert_eq!(tile.rent_due_paisa, screen.rent_due_paisa);
    }

    // Closed: it leaves the tile.
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let d = dashboard::get(w.env.ctx.db(), off()).unwrap();
    assert_eq!((d.tiles.gardens_open, d.gardens.len()), (0, 0));
    assert_eq!(d.tiles.gardens_balance_paisa.rupees(), 0);
}

// ---- Money agent review: more cases ------------------------------------------------------------------

#[test]
fn r34_counts_a_garden_other_shop_sale_once_and_not_as_a_trip_sale() {
    let (w, _) = garden_40();
    // The 10,000 of the other shop is garden money in (66,400 with the sale in our shop), and no
    // trip sold anything.
    assert_eq!(r34(&w, "12-10-2026", "report-r34-gardens-in").1, 66_400);
    assert_eq!(r34(&w, "12-10-2026", "report-r34-trips-sold").1, 0);
}

#[test]
fn view_g_with_a_garden_picked_shows_only_the_garden() {
    // The world also has بکری 1 (consigned, 3,948 of shared income) and the garden of E2E-40.
    let (w, garden) = garden_40();
    save(&w, &support::event1(&w));
    let page = view(
        &w,
        AnalyticsView::G,
        AnalyticsFilters {
            garden_id: Some(garden.id),
            ..daily()
        },
    );
    for key in ["shared_income", "shop_expenses", "profit", "trips_result"] {
        let at = column(&page, key);
        assert!(
            data_rows(&page)
                .iter()
                .all(|r| matches!(r.cells[at], Cell::Empty)),
            "{key} follows no garden"
        );
    }
    assert_eq!(g_gardens(&page), [-30_000, -25_000, 66_400]);
    // Gross and commission are the garden's only: 60,000 and 8,000 on 12-10.
    let gross = column(&page, "gross");
    assert_eq!(n(&data_rows(&page).last().unwrap().cells[gross]), 60_000);
}

#[test]
fn view_i_keeps_the_whole_life_when_the_period_ends_before_the_sale() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let page = view(&w, AnalyticsView::I, on("01-10-2026", "10-10-2026"));
    assert_eq!(
        i_figures(&page),
        [33_000, 22_000, 56_400, 10_000, 0, 8_000, 11_400]
    );
}

#[test]
fn an_undone_and_repeated_closing_counts_once_in_views_f_g_and_r36() {
    use mandi_services::core::gardens::GardenReopenInput;
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let version = gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id: garden.id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap()
    .row
    .version;
    gardens::reopen(
        &w.env.ctx,
        &w.owner,
        &GardenReopenInput {
            id: garden.id,
            version,
            reason: "x".into(),
        },
    )
    .unwrap();
    garden_close(&w, garden.id, "13-10-2026");
    let [a, _, _] = w.partners;
    assert_eq!(f_gardens_of(&w, a, "13-10-2026", "13-10-2026"), 39_840);
    assert_eq!(f_gardens_of(&w, a, "01-10-2026", "31-10-2026"), 6_840);
    // G: the closing is not money in, so there is no 13-10 row.
    assert_eq!(
        g_gardens(&view(&w, AnalyticsView::G, daily())),
        [-30_000, -25_000, 66_400]
    );
    assert_eq!(
        i_partners(&view(&w, AnalyticsView::I, filters())),
        [(6_000, 6_840), (4_000, 4_560)]
    );
}

#[test]
fn two_gardens_with_sales_and_closings_are_summed_not_mixed() {
    // A second garden: rent 10,000, installment 4,000 (2,400 from the shop, 1,600 from the
    // pocket), a sale of 10 at 300 on 12-10: value 3,000, seller comm 180, buyer comm 180, لاگا 80,
    // garden account credit 2,820, commission 440. Balance 2,820 - 4,000 = -1,180. Closing 2,820
    // -> 1,692 / 1,128.
    let (w, first) = garden_40();
    let second = garden_made_with(&w, "10,000", "4,000");
    garden_lot_sale(&w, second.account.id, "12-10-2026", "10", w.akram, "300");
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, first.id, "13-10-2026");
    garden_close(&w, second.id, "13-10-2026");

    let page = view(&w, AnalyticsView::I, filters());
    let rows = data_rows(&page);
    assert_eq!(rows.len(), 2);
    let fig = |i: usize| rows[i].cells[2..9].iter().map(n).collect::<Vec<_>>();
    assert_eq!(fig(0), [33_000, 22_000, 56_400, 10_000, 0, 8_000, 11_400]);
    assert_eq!(fig(1), [2_400, 1_600, 2_820, 0, 0, 440, -1_180]);
    // The totals add the two; the partners' results add up to the results.
    assert_eq!(
        page.totals[2..9].iter().map(n).collect::<Vec<_>>(),
        [35_400, 23_600, 59_220, 10_000, 0, 8_440, 10_220]
    );
    let partners = |i: usize| -> Vec<(i64, i64)> {
        rows[i].cells[9..]
            .chunks(2)
            .map(|p| (n(&p[0]), n(&p[1])))
            .collect()
    };
    assert_eq!(partners(0), [(6_000, 6_840), (4_000, 4_560)]);
    // الف: back 1,692 less 2,400 from the shop; ب: back 1,128 less 1,600 from his pocket.
    assert_eq!(partners(1), [(6_000, -708), (4_000, -472)]);
    assert_eq!(n(&page.totals[10]), 6_840 - 708);
    assert_eq!(n(&page.totals[12]), 4_560 - 472);

    // F: الف's money from gardens over both: 6,840 - 708.
    let [a, b, _] = w.partners;
    assert_eq!(f_gardens_of(&w, a, "01-10-2026", "31-10-2026"), 6_132);
    assert_eq!(
        f_gardens_of(&w, b, "01-10-2026", "31-10-2026"),
        26_560 + 1_128
    );
    // G: the movement of both gardens; over their lives it is the sum of their balances.
    let g = view(&w, AnalyticsView::G, daily());
    let at = column(&g, "gardens_result");
    assert_eq!(n(&g.totals[at]), 11_400 - 1_180);
    // R36 totals.
    let r = report(&w, ReportId::R36, ReportFilters::default(), 0, 0);
    assert_eq!(r.rows.len(), 2);
    assert_eq!(money(&r.totals[13]), 10_220);
}

#[test]
fn a_partner_made_inactive_after_the_closing_keeps_his_rows() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let [_, b, _] = w.partners;
    w.env
        .ctx
        .db()
        .write(|tx| {
            tx.execute("UPDATE partners SET is_active = 0 WHERE id = ?1", [b])
                .map_err(mandi_services::db::DbError::from)
        })
        .unwrap();
    let rows = report(
        &w,
        ReportId::R36,
        ReportFilters {
            garden_view: Some(mandi_reports::reports::GardenView::Partner),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(rows.rows.len(), 2);
    assert_eq!(money(&rows.totals[6]), 11_400);
    assert_eq!(i_partners(&view(&w, AnalyticsView::I, filters())).len(), 2);
}

#[test]
fn r36_by_partner_with_a_partner_picked_shows_only_his_rows() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let [a, _, _] = w.partners;
    let page = report(
        &w,
        ReportId::R36,
        ReportFilters {
            garden_view: Some(mandi_reports::reports::GardenView::Partner),
            partner_id: Some(a),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(page.rows.len(), 1);
    assert_eq!(money(&page.rows[0].cells[6]), 6_840);
    assert_eq!(money(&page.totals[6]), 6_840);
}
