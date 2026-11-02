//! `run_first_rows` (plan 48 P48-S2): a report from its first row, built ONCE. The assistant's
//! `report` tool reads every row of a report to filter it (`where`) or to sort it by two keys.
//! Reading R20 in pages of 1,000 built the whole report nine times (64 seconds on the NFR-SCL-1
//! volume); this read builds it once. It must give the very rows, columns and totals the screen's
//! `run` gives (PRD FR-6.1, FR-19.9), so the expected numbers are the PRD's (4.9, E2E-8, E2E-43):
//! the buyers who owe are اکرم 2,880 and ارشد 18,684, 21,564 in all.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::ReportsError;
use mandi_reports::reports::{
    Cell, MAX_FIRST_ROWS, ReportFilters, ReportId, ReportPage, ReportRunInput, run, run_first_rows,
};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::paging::{SortDirection, SortInput};
use reports_world::*;
use support::World;

fn input(id: ReportId) -> ReportRunInput {
    ReportRunInput {
        id,
        filters: ReportFilters::default(),
        offset: 0,
        limit: 1_000,
        sort: None,
        group_by_month: false,
    }
}

fn first_rows(w: &World, id: ReportId, max_rows: i64) -> Result<ReportPage, ReportsError> {
    run_first_rows(w.env.ctx.db(), &input(id), max_rows)
}

fn keys(page: &ReportPage) -> Vec<String> {
    page.columns.iter().map(|c| c.key.clone()).collect()
}

#[test]
fn the_first_rows_are_the_rows_columns_and_totals_of_the_screens_page() {
    let w = full_day();
    for id in [
        ReportId::R20,
        ReportId::R21,
        ReportId::R22,
        ReportId::R24,
        ReportId::R26,
        ReportId::R34,
    ] {
        let screen = run(w.env.ctx.db(), &input(id)).unwrap();
        let once = first_rows(&w, id, 10_001).unwrap();
        assert_eq!(once.rows, screen.rows, "{id:?} rows");
        assert_eq!(keys(&once), keys(&screen), "{id:?} columns");
        assert_eq!(once.totals, screen.totals, "{id:?} totals");
        assert_eq!(once.total_count, screen.total_count, "{id:?} count");
        assert_eq!(once.summary, screen.summary, "{id:?} summary");
        assert_eq!(once.tables, screen.tables, "{id:?} tables");
    }
}

/// The OLD read of the assistant's `report` tool: pages one after the other, joined. (The tool
/// used pages of 1,000; a page of 1 makes even these small reports take several pages.)
fn paged_all(w: &World, id: ReportId, filters: &ReportFilters, page: i64) -> ReportPage {
    let ask = |offset: i64| {
        run(
            w.env.ctx.db(),
            &ReportRunInput {
                id,
                filters: filters.clone(),
                offset,
                limit: page,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap()
    };
    let mut first = ask(0);
    while i64::try_from(first.rows.len()).unwrap() < first.total_count {
        let next = ask(i64::try_from(first.rows.len()).unwrap());
        if next.rows.is_empty() {
            break;
        }
        first.rows.extend(next.rows);
    }
    first
}

#[test]
fn the_new_read_equals_the_old_paged_read_on_the_worked_day() {
    // Events 1 to 5 (E2E-43's data): the rows, the totals and the count of the one build are those
    // of the pages joined, for the reports the prompt's own questions read (R20, R24) and others.
    let w = full_day();
    let day_sheet = ReportFilters {
        as_of: "10-10-2026".into(),
        ..ReportFilters::default()
    };
    let cash_day = ReportFilters {
        from: "10-10-2026".into(),
        to: "10-10-2026".into(),
        ..ReportFilters::default()
    };
    for (id, filters) in [
        (ReportId::R20, ReportFilters::default()),
        (ReportId::R21, ReportFilters::default()),
        (ReportId::R22, ReportFilters::default()),
        (ReportId::R24, ReportFilters::default()),
        (ReportId::R26, ReportFilters::default()),
        (ReportId::R34, ReportFilters::default()),
        (ReportId::R35, day_sheet),
        // The line reports are SQL pages with a running balance (the money review of P48-S2).
        (ReportId::R12, cash_day),
        (ReportId::R13, party(w.akram)),
    ] {
        let old = paged_all(&w, id, &filters, 1);
        let new = run_first_rows(
            w.env.ctx.db(),
            &ReportRunInput {
                filters: filters.clone(),
                ..input(id)
            },
            10_001,
        )
        .unwrap();
        assert_eq!(new.rows, old.rows, "{id:?} rows");
        assert_eq!(keys(&new), keys(&old), "{id:?} columns");
        assert_eq!(new.totals, old.totals, "{id:?} totals");
        assert_eq!(new.total_count, old.total_count, "{id:?} count");
        assert_eq!(new.summary, old.summary, "{id:?} summary");
        // The closing of a line report is the PRD's: the cash book of 10-10-26 closes at 58,484
        // (E2E-8) and اکرم's ledger at 2,880 بنام (E2E-33).
        let closing = |page: &ReportPage| {
            page.summary
                .iter()
                .find(|l| l.label_id == "report-closing-balance")
                .map(|l| balance(&l.cell))
        };
        match id {
            ReportId::R12 => assert_eq!(closing(&new), Some((BalanceSide::Dr, 58_484))),
            ReportId::R13 => assert_eq!(closing(&new), Some((BalanceSide::Dr, 2_880))),
            _ => {}
        }
    }
}

#[test]
fn the_worked_day_dues_are_the_prd_buyers_and_their_total() {
    // PRD 4.9, E2E-43: اکرم 2,880 and ارشد 18,684 owe, 21,564 in all.
    let w = full_day();
    let page = first_rows(&w, ReportId::R20, 10_001).unwrap();
    assert_eq!(page.total_count, 2);
    assert_eq!(page.rows.len(), 2);
    let balance = page
        .columns
        .iter()
        .position(|c| c.key == "balance")
        .expect("the balance column");
    // R20 lists the buyers by account code: اکرم 03040001, then ارشد 03040002.
    assert_eq!(money(&page.rows[0].cells[balance]), 2_880);
    assert_eq!(money(&page.rows[1].cells[balance]), 18_684);
    assert_eq!(
        page.totals[balance],
        Cell::Money {
            paisa: rupees(21_564)
        }
    );
}

#[test]
fn a_report_bigger_than_the_most_asked_shows_in_the_count_not_in_the_rows() {
    // Asking for one row more than the most accepted tells the caller the report is too big:
    // the rows are cut, the count is of the whole filter.
    let w = full_day();
    let page = first_rows(&w, ReportId::R20, 1).unwrap();
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.total_count, 2);
    assert!(page.total_count > i64::try_from(page.rows.len()).unwrap());
}

#[test]
fn the_most_asked_and_a_sort_are_checked() {
    let w = full_day();
    for bad in [0, -1, MAX_FIRST_ROWS + 1] {
        assert!(
            matches!(
                first_rows(&w, ReportId::R20, bad),
                Err(ReportsError::Rejected(_))
            ),
            "{bad}"
        );
    }
    assert!(first_rows(&w, ReportId::R20, MAX_FIRST_ROWS).is_ok());
    let mut sorted = input(ReportId::R20);
    sorted.sort = Some(SortInput {
        column: "balance".into(),
        direction: SortDirection::Desc,
        language: None,
    });
    assert!(matches!(
        run_first_rows(w.env.ctx.db(), &sorted, 10),
        Err(ReportsError::Data(_))
    ));
    let mut months = input(ReportId::R20);
    months.group_by_month = true;
    assert!(matches!(
        run_first_rows(w.env.ctx.db(), &months, 10),
        Err(ReportsError::Data(_))
    ));
}
