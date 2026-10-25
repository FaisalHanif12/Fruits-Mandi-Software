//! R36 باغات رپورٹ (the gardens report) of plan d5-late, round 11 (PRD 19.2 R36, 12.11, FR-12.37,
//! FR-12.38, E2E-40, E2E-40b) and the ties of the garden figures to the garden screen.
//!
//! The expected numbers are the PRD's (`worked_day.py`) and the money agent's worked cases:
//! E2E-40: paid 55,000 (shop 33,000, pocket 22,000), our shop 56,400, other shops 10,000, money
//! in 66,400, commission 8,000, balance 11,400; closing 39,840 / 26,560, results 6,840 / 4,560.
//! E2E-40b: money in 10,000, balance -45,000, closing 6,000 / 4,000, results -27,000 / -18,000.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
mod xlsx_support;

use common::*;
use mandi_reports::reports::{Cell, GardenView, ReportFilters, ReportId, ReportPage, RowKind};
use mandi_services::core::bikri::{CancelInput, EditOpenInput, EditSaveInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{
    GardenEntryCancelInput, GardenGetInput, GardenHistoryKind, GardenReopenInput,
};
use mandi_services::{bikri, gardens};
use reports_world::*;

fn r36(w: &support::World, filters: ReportFilters) -> ReportPage {
    report(w, ReportId::R36, filters, 0, 0)
}

fn by_partner() -> ReportFilters {
    ReportFilters {
        garden_view: Some(GardenView::Partner),
        ..ReportFilters::default()
    }
}

/// The twelve money cells of a garden row, from the rent to the rent due.
fn figures(page: &ReportPage, row: usize) -> Vec<i64> {
    page.rows[row].cells[3..].iter().map(money).collect()
}

fn pct(cell: &Cell) -> i64 {
    match cell {
        Cell::Percent { hundredths } => *hundredths,
        other => panic!("not a percent: {other:?}"),
    }
}

/// The partner rows as (share in hundredths, paid shop, paid pocket, back, result).
fn partner_rows(page: &ReportPage) -> Vec<(i64, i64, i64, i64, i64)> {
    page.rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .map(|r| {
            (
                pct(&r.cells[2]),
                money(&r.cells[3]),
                money(&r.cells[4]),
                money(&r.cells[5]),
                money(&r.cells[6]),
            )
        })
        .collect()
}

fn label(cell: &Cell) -> String {
    match cell {
        Cell::Label { id } => id.clone(),
        other => panic!("not a label: {other:?}"),
    }
}

/// The money in splits by source, what was paid by source of money, and for a closed garden the
/// results of the partners add up to the balance (money agent T16).
fn assert_foots(page: &ReportPage) {
    for row in &page.rows {
        let f: Vec<i64> = row.cells[3..].iter().map(money).collect();
        // [rent, installments, costs, paid shop, paid pocket, our shop, other shops, other in,
        //  commission, money in, balance, rent due]
        assert_eq!(
            f[5] + f[6] + f[7],
            f[9],
            "our shop + other shops + other in = money in"
        );
        assert_eq!(
            f[3] + f[4],
            f[1] + f[2],
            "shop + pocket = installments + costs"
        );
        assert_eq!(f[9] - (f[1] + f[2]), f[10], "balance = money in - paid");
        assert_eq!(f[0] - f[1], f[11], "rent due = rent - installments");
    }
}

fn detail(w: &support::World, id: i64) -> mandi_services::core::gardens::GardenDetail {
    gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap()
}

#[test]
fn the_catalogue_has_r36_with_its_filters() {
    let entry = mandi_reports::reports::catalog()
        .into_iter()
        .find(|e| e.id == ReportId::R36)
        .unwrap();
    assert_eq!(entry.title_id, "report-r36-title");
    assert!(!entry.party_required);
    let kinds: Vec<String> = entry.filters.iter().map(|f| format!("{f:?}")).collect();
    assert_eq!(
        kinds,
        ["DateRange", "Item", "Partner", "Garden", "GardenView"]
    );
}

#[test]
fn e2e_40_before_the_closing_the_balance_is_11400_and_the_results_are_what_was_paid() {
    let (w, garden) = garden_40();
    let page = r36(&w, ReportFilters::default());
    assert_eq!(page.rows.len(), 1);
    assert_eq!(label(&page.rows[0].cells[2]), "report-garden-open");
    // rent, installments, costs, paid shop, paid pocket, our shop, other shops, other in,
    // commission, money in, balance, rent due.
    assert_eq!(
        figures(&page, 0),
        [
            50_000, 50_000, 5_000, 33_000, 22_000, 56_400, 10_000, 0, 8_000, 66_400, 11_400, 0
        ]
    );
    assert_foots(&page);
    // The totals row is the row (one garden).
    assert_eq!(
        page.totals[3..].iter().map(money).collect::<Vec<_>>(),
        figures(&page, 0)
    );
    // Nothing has come back while it is open: each partner's result is minus what he paid.
    let partners = r36(&w, by_partner());
    assert_eq!(
        partner_rows(&partners),
        [
            (6_000, 33_000, 0, 0, -33_000),
            (4_000, 0, 22_000, 0, -22_000)
        ]
    );
    assert_eq!(money(&partners.totals[6]), -55_000);
    // The same numbers as the garden screen.
    let d = detail(&w, garden.id);
    assert_eq!(d.row.balance_paisa.rupees(), 11_400);
    assert_eq!(d.row.account_balance_paisa.rupees(), 66_400);
}

#[test]
fn e2e_40_after_the_closing_the_balance_stays_the_result_and_the_results_add_up_to_it() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let page = r36(&w, ReportFilters::default());
    assert_eq!(label(&page.rows[0].cells[2]), "report-garden-closed");
    // The closing is not money in: the balance is still the result 11,400, as on the screen.
    assert_eq!(
        figures(&page, 0),
        [
            50_000, 50_000, 5_000, 33_000, 22_000, 56_400, 10_000, 0, 8_000, 66_400, 11_400, 0
        ]
    );
    assert_foots(&page);
    let partners = r36(&w, by_partner());
    assert_eq!(
        partner_rows(&partners),
        [
            (6_000, 33_000, 0, 39_840, 6_840),
            (4_000, 0, 22_000, 26_560, 4_560)
        ]
    );
    // The totals: paid shop 33,000, pocket 22,000, back 66,400, results 11,400 = the balance.
    assert_eq!(
        [3, 4, 5, 6].map(|i| money(&partners.totals[i])),
        [33_000, 22_000, 66_400, 11_400]
    );
    let d = detail(&w, garden.id);
    assert_eq!(d.row.balance_paisa.rupees(), 11_400);
    assert_eq!(d.row.account_balance_paisa.rupees(), 0);
    let screen: Vec<i64> = d
        .partner_results
        .iter()
        .map(|r| r.net_paisa.rupees())
        .collect();
    assert_eq!(screen, [6_840, 4_560]);
}

#[test]
fn e2e_40b_a_loss_of_45000_and_the_results_add_up_to_it() {
    let (w, garden) = garden_40b();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let page = r36(&w, ReportFilters::default());
    assert_eq!(
        figures(&page, 0),
        [
            50_000, 50_000, 5_000, 33_000, 22_000, 0, 10_000, 0, 0, 10_000, -45_000, 0
        ]
    );
    assert_foots(&page);
    let partners = r36(&w, by_partner());
    assert_eq!(
        partner_rows(&partners),
        [
            (6_000, 33_000, 0, 6_000, -27_000),
            (4_000, 0, 22_000, 4_000, -18_000)
        ]
    );
    assert_eq!(money(&partners.totals[6]), -45_000);
}

#[test]
fn a_closing_that_is_undone_and_made_again_counts_once() {
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let version = detail(&w, garden.id).row.version;
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
    // After the undo: open, nothing back, the balance did not change.
    let page = r36(&w, ReportFilters::default());
    assert_eq!(label(&page.rows[0].cells[2]), "report-garden-open");
    assert_eq!(figures(&page, 0)[10], 11_400);
    let partners = r36(&w, by_partner());
    assert_eq!(
        partner_rows(&partners),
        [
            (6_000, 33_000, 0, 0, -33_000),
            (4_000, 0, 22_000, 0, -22_000)
        ]
    );
    // Closed again: 39,840 / 26,560 once, not twice.
    garden_close(&w, garden.id, "13-10-2026");
    let partners = r36(&w, by_partner());
    assert_eq!(
        partner_rows(&partners),
        [
            (6_000, 33_000, 0, 39_840, 6_840),
            (4_000, 0, 22_000, 26_560, 4_560)
        ]
    );
    assert_foots(&r36(&w, ReportFilters::default()));
}

#[test]
fn an_edit_of_the_sale_moves_the_commission_the_money_in_and_the_closing() {
    // Rate 600 to 610: value 61,000, seller comm 3,660, buyer comm 3,660, لاگا 800, credit 57,340,
    // commission 8,120, money in 67,340, balance 12,340; closing 40,404 / 26,936.
    let w = worked_day();
    let garden = garden_made(&w);
    set_today(&w.env, 2026, 10, 11);
    garden_pay(&w, garden.id, false, "11-10-2026", "20,000", "installment");
    garden_pay(&w, garden.id, true, "11-10-2026", "5,000", "labour");
    set_today(&w.env, 2026, 10, 12);
    let sale = garden_lot_sale(&w, garden.account.id, "12-10-2026", "100", w.arshad, "600");
    garden_other_sale(&w, garden.id, "12-10-2026", "20", "10,000");
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: sale.bikri_id,
            entry_id: sale.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("rate".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let page = r36(&w, ReportFilters::default());
    assert_eq!(figures(&page, 0)[5], 57_340);
    assert_eq!(figures(&page, 0)[8], 8_120);
    assert_eq!(figures(&page, 0)[9], 67_340);
    assert_eq!(figures(&page, 0)[10], 12_340);
    assert_foots(&page);
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let partners = r36(&w, by_partner());
    assert_eq!(
        partner_rows(&partners),
        [
            (6_000, 33_000, 0, 40_404, 7_404),
            (4_000, 0, 22_000, 26_936, 4_936)
        ]
    );
    assert_eq!(money(&partners.totals[6]), 12_340);
}

#[test]
fn a_cancelled_sale_leaves_the_figures_of_e2e_40b() {
    let (w, garden) = garden_40();
    // The sale of 12-10 is the second bikri of the world; find it through the garden's history.
    let sale_id = detail(&w, garden.id)
        .history
        .iter()
        .find_map(|h| (h.kind == GardenHistoryKind::Sale).then_some(h.bikri_id))
        .flatten()
        .unwrap();
    let version = bikri::get(&w.env.ctx, sale_id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: sale_id,
            version,
            reason: "x".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let page = r36(&w, ReportFilters::default());
    // Our shop 0, commission 0 (the sale and its reversal net), money in 10,000, balance -45,000.
    assert_eq!(
        figures(&page, 0),
        [
            50_000, 50_000, 5_000, 33_000, 22_000, 0, 10_000, 0, 0, 10_000, -45_000, 0
        ]
    );
    assert_foots(&page);
}

#[test]
fn a_cancelled_installment_leaves_the_rent_due_and_the_parts() {
    let (w, garden) = garden_40();
    // Cancel the 20,000 installment of 11-10.
    let (entry_id, version) = detail(&w, garden.id)
        .history
        .iter()
        .find_map(|h| {
            (h.kind == GardenHistoryKind::Installment && h.entry_date == "2026-10-11")
                .then(|| (h.entry_id.unwrap(), h.version.unwrap()))
        })
        .unwrap();
    gardens::entry_cancel(
        &w.env.ctx,
        &w.owner,
        &GardenEntryCancelInput {
            id: entry_id,
            version,
            reason: "x".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let page = r36(&w, ReportFilters::default());
    // Installments 30,000, paid 35,000 (shop 21,000, pocket 14,000), rent due 20,000.
    assert_eq!(figures(&page, 0)[1], 30_000);
    assert_eq!(figures(&page, 0)[3], 21_000);
    assert_eq!(figures(&page, 0)[4], 14_000);
    assert_eq!(figures(&page, 0)[11], 20_000);
    assert_foots(&page);
}

#[test]
fn a_closing_of_a_garden_with_no_money_in_posts_nothing_and_the_results_are_the_losses() {
    // E64: three payments, no sale; closed. Results -33,000 / -22,000 = the balance -55,000.
    let w = worked_day();
    let garden = garden_made(&w);
    set_today(&w.env, 2026, 10, 11);
    garden_pay(&w, garden.id, false, "11-10-2026", "20,000", "installment");
    garden_pay(&w, garden.id, true, "11-10-2026", "5,000", "labour");
    garden_close(&w, garden.id, "11-10-2026");
    let partners = r36(&w, by_partner());
    assert_eq!(
        partner_rows(&partners),
        [
            (6_000, 33_000, 0, 0, -33_000),
            (4_000, 0, 22_000, 0, -22_000)
        ]
    );
    let page = r36(&w, ReportFilters::default());
    assert_eq!(figures(&page, 0)[10], -55_000);
    assert_eq!(money(&partners.totals[6]), -55_000);
    assert_foots(&page);
}

#[test]
fn a_loss_closing_with_a_negative_seller_credit_is_split_on_the_other_side() {
    // Rent 1,000 with an installment of 1,000 (600 / 400); a lot of 1 at 500 with labour 1,000:
    // value 500, seller comm 30, buyer comm 30, لاگا 8, buyer pays 538, the garden account is
    // debited 530; commission 68; money in -530; balance -1,530; closing B = -530 -> -318 / -212.
    let w = worked_day();
    let garden = garden_made_with(&w, "1,000", "1,000");
    let mut lot = support::with_sales(
        support::bare_lot(&w, "garden-1", garden.account.id, w.banana, "1"),
        vec![support::sale("s-1", w.arshad, "1", "500")],
    );
    lot.labour = support::manual("1,000");
    lot.adjust_advance = false;
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = "10-10-2026".into();
    save(&w, &draft);
    let page = r36(&w, ReportFilters::default());
    // rent 1,000, installments 1,000, costs 0, paid 600 / 400, our shop -530, commission 68.
    assert_eq!(
        figures(&page, 0),
        [1_000, 1_000, 0, 600, 400, -530, 0, 0, 68, -530, -1_530, 0]
    );
    assert_foots(&page);
    garden_close(&w, garden.id, "10-10-2026");
    let partners = r36(&w, by_partner());
    assert_eq!(
        partner_rows(&partners),
        [(6_000, 600, 0, -318, -918), (4_000, 0, 400, -212, -612)]
    );
    assert_eq!(money(&partners.totals[6]), -1_530);
    let d = detail(&w, garden.id);
    assert_eq!(d.row.balance_paisa.rupees(), -1_530);
    assert_eq!(d.row.account_balance_paisa.rupees(), 0);
}

#[test]
fn the_filters_pick_the_gardens_and_leave_the_commission_over_the_whole_life() {
    let (w, garden) = garden_40();
    let [a, b, c] = w.partners;
    // A period that covers only the day made: the garden is picked, its commission is lifetime.
    let only_day_made = ReportFilters {
        from: "10-10-2026".into(),
        to: "10-10-2026".into(),
        ..ReportFilters::default()
    };
    assert_eq!(figures(&r36(&w, only_day_made), 0)[8], 8_000);
    // A period before the garden: no garden.
    set_today(&w.env, 2026, 10, 13);
    let before = ReportFilters {
        from: "01-10-2026".into(),
        to: "09-10-2026".into(),
        ..ReportFilters::default()
    };
    assert_eq!(r36(&w, before).rows.len(), 0);
    // The item picks gardens by their main fruit, not the items of their lots.
    let item = |id: i64| ReportFilters {
        item_id: Some(id),
        ..ReportFilters::default()
    };
    assert_eq!(r36(&w, item(w.banana)).rows.len(), 1);
    assert_eq!(r36(&w, item(w.apple)).rows.len(), 0);
    // The partner picks the gardens he is in; ج is in none.
    let partner = |id: i64| ReportFilters {
        partner_id: Some(id),
        ..ReportFilters::default()
    };
    assert_eq!(r36(&w, partner(a)).rows.len(), 1);
    assert_eq!(r36(&w, partner(b)).rows.len(), 1);
    assert_eq!(r36(&w, partner(c)).rows.len(), 0);
    // One garden, and an unknown one.
    let one = ReportFilters {
        garden_id: Some(garden.id),
        ..ReportFilters::default()
    };
    assert_eq!(r36(&w, one).rows.len(), 1);
    assert_eq!(
        rejected(
            &w,
            ReportId::R36,
            ReportFilters {
                garden_id: Some(9_999),
                ..ReportFilters::default()
            },
            0,
            0
        ),
        vec![(ErrorCode::ErrNotFound, "filters.garden_id".to_owned())]
    );
}

#[test]
fn the_garden_filters_are_refused_by_the_reports_that_do_not_take_them() {
    let w = worked_day();
    assert_eq!(
        rejected(
            &w,
            ReportId::R33,
            ReportFilters {
                garden_view: Some(GardenView::Partner),
                ..ReportFilters::default()
            },
            0,
            0
        ),
        vec![(ErrorCode::ErrOutOfRange, "filters.garden_view".to_owned())]
    );
    assert_eq!(
        rejected(
            &w,
            ReportId::R18,
            ReportFilters {
                garden_id: Some(1),
                ..ReportFilters::default()
            },
            0,
            0
        ),
        vec![(ErrorCode::ErrOutOfRange, "filters.garden_id".to_owned())]
    );
}

#[test]
fn several_gardens_are_totalled_over_the_whole_filter_and_page() {
    let (w, first) = garden_40();
    // A second garden: rent 10,000, an installment of 4,000 from the shop and the pocket.
    let second = garden_made_with(&w, "10,000", "4,000");
    let all = r36(&w, ReportFilters::default());
    assert_eq!(all.rows.len(), 2);
    assert_foots(&all);
    let sum = |i: usize| figures(&all, 0)[i] + figures(&all, 1)[i];
    for i in 0..12 {
        assert_eq!(money(&all.totals[3 + i]), sum(i), "column {i}");
    }
    // A page keeps the count and the totals of the whole filter.
    let page2 = report(&w, ReportId::R36, ReportFilters::default(), 1, 1);
    assert_eq!(page2.rows.len(), 1);
    assert_eq!(page2.total_count, 2);
    assert_eq!(page2.totals, all.totals);
    let _ = (first, second);
}

// ---- Excel -------------------------------------------------------------------------------------------

/// Every number of a sheet, as the text Excel stores.
fn numbers_of(sheet: &xlsx_support::Sheet) -> Vec<String> {
    (0..=sheet.last_row())
        .flat_map(|r| (0..20).map(move |c| (r, c)))
        .filter_map(|(r, c)| match sheet.get(r, c) {
            xlsx_support::Value::Num(n) => Some(n.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn r36_exports_to_excel_with_the_numbers_of_the_page_in_both_layouts() {
    use mandi_reports::export::{ReportExportInput, export_report};
    use mandi_services::core::auth::Language;
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let dir = tempfile::tempdir().unwrap();
    for (name, view, wants) in [
        (
            "garden",
            GardenView::Garden,
            vec![
                "50000", "5000", "33000", "22000", "56400", "10000", "8000", "66400", "11400",
            ],
        ),
        (
            "partner",
            GardenView::Partner,
            vec!["33000", "22000", "39840", "26560", "6840", "4560", "11400"],
        ),
    ] {
        let path = dir.path().join(format!("r36-{name}.xlsx"));
        export_report(
            w.env.ctx.db(),
            &ReportExportInput {
                id: ReportId::R36,
                filters: ReportFilters {
                    garden_view: Some(view),
                    ..ReportFilters::default()
                },
                language: Language::En,
                path: path.display().to_string(),
            },
            &mut |_| {},
        )
        .unwrap();
        let numbers = numbers_of(&xlsx_support::read(&path));
        for want in wants {
            assert!(
                numbers.contains(&want.to_owned()),
                "{name}: {want} in {numbers:?}"
            );
        }
    }
}

#[test]
fn view_i_exports_to_excel_with_the_numbers_of_the_page() {
    use mandi_reports::analytics::AnalyticsFilters;
    use mandi_reports::export::{AnalyticsExportInput, export_analytics};
    use mandi_services::core::auth::Language;
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("view-i.xlsx");
    export_analytics(
        w.env.ctx.db(),
        &AnalyticsExportInput {
            view: mandi_reports::analytics::AnalyticsView::I,
            filters: AnalyticsFilters::default(),
            language: Language::En,
            path: path.display().to_string(),
        },
        &mut |_| {},
    )
    .unwrap();
    let numbers = numbers_of(&xlsx_support::read(&path));
    for want in [
        "33000", "22000", "56400", "10000", "8000", "11400", "6840", "4560",
    ] {
        assert!(numbers.contains(&want.to_owned()), "{want} in {numbers:?}");
    }
}

#[test]
fn r36_prints_in_both_languages_with_the_filters_in_the_header() {
    use mandi_reports::reports::{ReportPrintInput, print_data};
    use mandi_services::core::auth::Language;
    let (w, garden) = garden_40();
    for language in [Language::Ur, Language::En] {
        let data = print_data(
            w.env.ctx.db(),
            &ReportPrintInput {
                id: ReportId::R36,
                filters: ReportFilters {
                    garden_id: Some(garden.id),
                    garden_view: Some(GardenView::Partner),
                    ..ReportFilters::default()
                },
                language,
                paper: None,
            },
        )
        .unwrap();
        assert_eq!(data.rows.len(), 2, "{language:?}");
        // The garden and the layout are named in the header.
        assert!(data.filters.len() >= 2, "{language:?}");
    }
}

// ---- R31, the statement of a partner, with the garden lines (D4R-A8) ---------------------------------

#[test]
fn r31_of_alif_after_e2e_40_has_the_garden_lines_and_ends_at_his_total_10040() {
    use mandi_reports::reports::{statement_page, statement_request};
    use mandi_services::core::statement::PartnersStatementInput;
    let (w, garden) = garden_40();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let [a, _, _] = w.partners;
    let filters = ReportFilters {
        partner_id: Some(a),
        ..ReportFilters::default()
    };
    let request = statement_request(w.env.ctx.db(), &filters).unwrap();
    let statement = mandi_services::partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            partner_id: request.partner_id,
            from: None,
            to: None,
            offset: 0,
            limit: 100,
        },
    )
    .unwrap();
    let page = statement_page(&request, &statement, 0, 100).unwrap();
    // The lines with an amount: -18,000 (the installment of 10-10), -12,000 and -3,000 (11-10), his
    // share of the commission +3,200 (12-10) and the closing +39,840 (13-10); the running total
    // ends at his کل رقم 10,040 (C22).
    let lines: Vec<(i64, i64)> = page
        .rows
        .iter()
        .filter(|r| !matches!(&r.cells[3], Cell::Empty))
        .map(|r| (money(&r.cells[3]), money(&r.cells[4])))
        .collect();
    assert_eq!(
        lines,
        [
            (-18_000, -18_000),
            (-12_000, -30_000),
            (-3_000, -33_000),
            (3_200, -29_800),
            (39_840, 10_040)
        ]
    );
}
