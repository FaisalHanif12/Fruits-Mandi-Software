//! The quality of a sale line (درجہ, PRD FR-12.42, plan 58) on the reports and the prints: R5 and
//! its slips, the ledger with items (R13), the sales register (R2 and R4) and the party day slip
//! (R35). A line with a quality carries it; a report with no quality anywhere reads exactly as it
//! did before (the quality column is not there).
//!
//! The data is Ehsan's 20 پیٹی سیب of plan 58 section 3 on the PRD worked day: اکرم takes 10 اول at
//! 1,200 and 6 دوم at 800, ارشد takes 4 سوم at 500. Every figure is the one of the no-quality case.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::reports::{
    Cell, DetailPart, PartyFilter, PrintCell, PrintDetailPart, ReportFilters, ReportId, ReportPage,
    ReportPrintInput, buyer_bills_print_data,
};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::SaleGrade;

use reports_world::*;

/// 20 apples of رحیم: اکرم 10 and 6, ارشد 4. `grades` gives the quality of each of the three
/// lines (`None` for no quality).
fn apples(
    w: &support::World,
    grades: [Option<&str>; 3],
) -> mandi_services::core::bikri::BikriDraft {
    let mut lines = vec![
        support::sale("s-1", w.akram, "10", "1200"),
        support::sale("s-2", w.akram, "6", "800"),
        support::sale("s-3", w.arshad, "4", "500"),
    ];
    for (line, grade) in lines.iter_mut().zip(grades) {
        line.grade = grade.map(str::to_owned);
    }
    support::draft(vec![support::with_sales(
        support::bare_lot(w, "lot-1", w.rahim, w.apple, "20"),
        lines,
    )])
}

fn graded_world() -> support::World {
    let w = worked_day();
    save(
        &w,
        &apples(&w, [Some("first"), Some("second"), Some("third")]),
    );
    w
}

fn ungraded_world() -> support::World {
    let w = worked_day();
    save(&w, &apples(&w, [None, None, None]));
    w
}

fn keys(page: &ReportPage) -> Vec<&str> {
    page.columns.iter().map(|c| c.key.as_str()).collect()
}

/// `(qty, grade)` of every sale part of a column of detail cells.
fn sale_parts(page: &ReportPage, column: usize) -> Vec<(i64, Option<SaleGrade>)> {
    let mut out = Vec::new();
    for row in &page.rows {
        if let Some(Cell::Detail { parts }) = row.cells.get(column) {
            for part in parts {
                if let DetailPart::Sale { qty, grade, .. } = part {
                    out.push((*qty, *grade));
                }
            }
        }
    }
    out
}

fn sheet(account: Option<i64>) -> ReportFilters {
    ReportFilters {
        as_of: "10-10-2026".into(),
        party: account.map(|account_id| PartyFilter {
            account_id,
            person_key: None,
        }),
        ..ReportFilters::default()
    }
}

#[test]
fn r5_writes_the_quality_of_each_line_and_the_figures_are_the_no_quality_ones() {
    let w = graded_world();
    let page = report(&w, ReportId::R5, ReportFilters::default(), 0, 0);
    let mut parts = sale_parts(&page, 2);
    parts.sort_by_key(|(qty, _)| *qty);
    assert_eq!(
        parts,
        vec![
            (4, Some(SaleGrade::Third)),
            (6, Some(SaleGrade::Second)),
            (10, Some(SaleGrade::First))
        ]
    );
    // The totals: value 18,800 and, after commission and لاگا, 12,800 + 5,136 + 2,152 = 20,088.
    assert_eq!(money(&page.totals[3]), 18_800);
    assert_eq!(money(&page.totals[4]), 20_088);
    // The same report with no quality: no part has one, and the totals are the same.
    let plain = report(
        &ungraded_world(),
        ReportId::R5,
        ReportFilters::default(),
        0,
        0,
    );
    assert!(sale_parts(&plain, 2).iter().all(|(_, g)| g.is_none()));
    assert_eq!(plain.totals, page.totals);
}

#[test]
fn the_buyer_slips_carry_the_quality_to_the_print() {
    let w = graded_world();
    let data = buyer_bills_print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            group_by_month: false,
            sort: None,
            id: ReportId::R5,
            filters: ReportFilters::default(),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    let mut seen: Vec<(String, Option<SaleGrade>)> = Vec::new();
    for slip in &data.slips {
        for row in &slip.rows {
            for cell in &row.cells {
                if let PrintCell::Detail { parts } = cell {
                    for part in parts {
                        if let PrintDetailPart::Sale { qty, grade, .. } = part {
                            seen.push((qty.clone(), *grade));
                        }
                    }
                }
            }
        }
    }
    seen.sort_by_key(|(qty, _)| qty.parse::<i64>().unwrap());
    assert_eq!(
        seen,
        vec![
            ("4".to_owned(), Some(SaleGrade::Third)),
            ("6".to_owned(), Some(SaleGrade::Second)),
            ("10".to_owned(), Some(SaleGrade::First))
        ]
    );
}

#[test]
fn the_ledger_with_items_carries_the_quality_of_the_sale_lines() {
    let w = graded_world();
    let page = report(&w, ReportId::R13, party(w.akram), 0, 0);
    let mut parts = sale_parts(&page, 2);
    parts.sort_by_key(|(qty, _)| *qty);
    assert_eq!(
        parts,
        vec![(6, Some(SaleGrade::Second)), (10, Some(SaleGrade::First))]
    );
    let plain = report(&ungraded_world(), ReportId::R13, party(w.akram), 0, 0);
    assert!(sale_parts(&plain, 2).iter().all(|(_, g)| g.is_none()));
}

#[test]
fn the_sales_register_has_the_quality_column_only_when_a_line_has_one() {
    // With qualities: the column follows the item, and every page has it.
    let w = graded_world();
    for id in [ReportId::R2, ReportId::R4] {
        let page = report(&w, id, ReportFilters::default(), 0, 0);
        let mut expected = vec![
            "date",
            "doc",
            "buyer",
            "city",
            "item",
            "grade",
            "qty",
            "rate",
            "value",
            "buyer_total",
        ];
        // R3 and R4 have the paid on the spot and left columns after the buyer total (PRD FR-12.44).
        if id == ReportId::R4 {
            expected.extend(["paid", "left"]);
        }
        assert_eq!(keys(&page), expected, "{id:?}");
        let words: Vec<&Cell> = page.rows.iter().map(|r| &r.cells[5]).collect();
        assert_eq!(
            words,
            vec![
                &Cell::Label {
                    id: "report-grade-first".into()
                },
                &Cell::Label {
                    id: "report-grade-second".into()
                },
                &Cell::Label {
                    id: "report-grade-third".into()
                },
            ]
        );
        assert_eq!(page.totals.len(), page.columns.len());
        // The money columns moved by one place and still add up: 18,800.
        assert_eq!(money(&page.totals[8]), 18_800);
    }
    // Page by page: a page whose only line has no quality still has the column.
    let w = worked_day();
    save(&w, &apples(&w, [None, Some("second"), None]));
    let first_page = report(&w, ReportId::R2, ReportFilters::default(), 0, 1);
    assert!(keys(&first_page).contains(&"grade"));
    assert_eq!(first_page.rows[0].cells[5], Cell::Empty);
    // With no quality anywhere the register is exactly what it was.
    let plain = report(
        &ungraded_world(),
        ReportId::R2,
        ReportFilters::default(),
        0,
        0,
    );
    assert_eq!(
        keys(&plain),
        [
            "date",
            "doc",
            "buyer",
            "city",
            "item",
            "qty",
            "rate",
            "value",
            "buyer_total"
        ]
    );
    assert_eq!(plain.totals.len(), 9);
}

#[test]
fn the_day_slip_has_the_quality_column_last_and_only_on_a_graded_slip() {
    let w = graded_world();
    // اکرم's slip: two lines of one item (so a total row), with their qualities.
    let page = report(&w, ReportId::R35, sheet(Some(w.akram)), 0, 0);
    assert_eq!(
        keys(&page),
        ["item", "packing", "qty", "rate", "amount", "grade"]
    );
    assert_eq!(page.totals.len(), page.columns.len());
    let labels: Vec<Cell> = page.rows.iter().map(|r| r.cells[5].clone()).collect();
    assert_eq!(
        labels,
        vec![
            Cell::Label {
                id: "report-grade-first".into()
            },
            Cell::Label {
                id: "report-grade-second".into()
            },
            // The item total row has no quality.
            Cell::Empty,
        ]
    );
    // رحیم's slip lists the same three sale lines for the seller.
    let seller = report(&w, ReportId::R35, sheet(Some(w.rahim)), 0, 0);
    assert_eq!(
        keys(&seller),
        ["item", "packing", "qty", "rate", "amount", "grade"]
    );
    // A slip with no quality is as it was: five columns.
    let plain = report(&ungraded_world(), ReportId::R35, sheet(Some(w.akram)), 0, 0);
    assert_eq!(keys(&plain), ["item", "packing", "qty", "rate", "amount"]);
    assert_eq!(plain.totals.len(), 5);
}
