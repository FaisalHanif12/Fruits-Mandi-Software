//! Plan 38 B1 (PRD FR-19.11, Code Standards B31, NFR-SPD-24): every report of the engine and every
//! analytics view loads one page at a time, with the row count and the totals over the WHOLE
//! filter. For each of them, on the PRD worked day:
//!
//! - a page of 25, 50 or 100 rows never holds more than that, and the count is the same on every
//!   page;
//! - the pages of ONE row, read one after the other, hold every row once, in the order of the
//!   whole list (this proves the slicing whatever the size of the data);
//! - the totals row is the same on every page and on the whole list;
//! - an offset past the end gives no rows and the same count and totals.
//!
//! The advances report (R26 data) and the learned lists of fruit take the same page.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::analytics::{AnalyticsFilters, AnalyticsPage, AnalyticsRunInput, AnalyticsView};
use mandi_reports::reports::{FilterKind, ReportFilters, ReportId, ReportPage, RowKind};
use mandi_services::core::vouchers::AdvancesReportInput;
use mandi_services::vouchers;
use reports_world::*;
use serde::Serialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const PAGE_SIZES: [i64; 3] = [25, 50, 100];
const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Writes the file (when asked to) or checks that it holds exactly this text.
fn write_or_check(file: &str, text: &str) {
    let path = folder().join(file);
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

/// One pair of files: the request (exactly the `input` of the command) and the real answer.
fn pair<A: Serialize>(name: &str, request: &Value, answer: &A) {
    write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
    );
}

fn day_filters(w: &support::World, id: ReportId) -> ReportFilters {
    let entry = mandi_reports::reports::catalog()
        .into_iter()
        .find(|e| e.id == id)
        .unwrap();
    let mut filters = if id == ReportId::R1 {
        // The sale book is one بکری: the first of the worked day.
        let first: i64 = w
            .env
            .ctx
            .db()
            .read(|conn| {
                conn.query_row("SELECT id FROM bikri_entries WHERE number = 1", [], |r| {
                    r.get(0)
                })
            })
            .unwrap();
        ReportFilters {
            bikri_id: Some(first),
            ..ReportFilters::default()
        }
    } else if entry.party_required {
        party(w.akram)
    } else {
        ReportFilters::default()
    };
    if entry.filters.contains(&FilterKind::DateRange) {
        filters.from = "2026-10-10".into();
        filters.to = "2026-10-10".into();
    }
    if entry.filters.contains(&FilterKind::AsOf) {
        filters.as_of = "2026-10-10".into();
    }
    filters
}

fn rows_of(page: &ReportPage) -> Vec<Value> {
    page.rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect()
}

#[test]
fn every_report_pages_with_the_count_and_the_totals_of_the_whole_filter() {
    let w = full_day();
    let ids: Vec<ReportId> = mandi_reports::reports::catalog()
        .iter()
        .map(|e| e.id)
        // The partner statement is built by the services, with its own paging test.
        .filter(|id| *id != ReportId::R31)
        .collect();
    assert!(ids.len() >= 22, "{} reports", ids.len());
    let mut grouped_pages = 0;
    for id in ids {
        let filters = day_filters(&w, id);
        let whole = report(&w, id, filters.clone(), 0, 1_000);
        let all = rows_of(&whole);
        let what = id.code();
        assert_eq!(
            usize::try_from(whole.total_count).unwrap(),
            all.len(),
            "{what}: the count is the number of rows"
        );
        // A page of 25, 50 or 100 holds at most that many rows and the same count and totals.
        for size in PAGE_SIZES {
            let page = report(&w, id, filters.clone(), 0, size);
            assert!(
                i64::try_from(page.rows.len()).unwrap() <= size,
                "{what} {size}"
            );
            assert_eq!(page.total_count, whole.total_count, "{what} {size}");
            assert_eq!(
                serde_json::to_value(&page.totals).unwrap(),
                serde_json::to_value(&whole.totals).unwrap(),
                "{what} {size}: totals over the whole filter"
            );
        }
        // Pages of one row hold every row once, in order, with the same totals each time.
        let mut paged = Vec::new();
        for offset in 0..whole.total_count {
            let page = report(&w, id, filters.clone(), offset, 1);
            assert_eq!(page.rows.len(), 1, "{what} offset {offset}");
            assert_eq!(page.total_count, whole.total_count, "{what}");
            assert_eq!(
                serde_json::to_value(&page.totals).unwrap(),
                serde_json::to_value(&whole.totals).unwrap(),
                "{what}: totals on page {offset}"
            );
            // A page that starts inside a group says which group, with the whole group's heading
            // row; a page that starts with a heading, or any page of a report without group rows,
            // has none (plan 38 B2).
            let before = usize::try_from(offset).unwrap();
            let starts_with_heading = whole.rows[before].kind == RowKind::Group;
            let expected = if offset == 0 || starts_with_heading {
                None
            } else {
                whole.rows[..before]
                    .iter()
                    .rev()
                    .find(|r| r.kind == RowKind::Group)
                    .map(|r| serde_json::to_value(r).unwrap())
            };
            assert_eq!(
                page.group_above
                    .as_ref()
                    .map(|r| serde_json::to_value(r).unwrap()),
                expected,
                "{what}: the group a page starts in, page {offset}"
            );
            if expected.is_some() {
                grouped_pages += 1;
            }
            paged.extend(rows_of(&page));
        }
        assert_eq!(paged, all, "{what}: the pages are the whole list");
        // Past the end: no rows, the same count and totals.
        let past = report(&w, id, filters, whole.total_count + 50, 25);
        assert!(past.rows.is_empty(), "{what}");
        assert_eq!(past.total_count, whole.total_count, "{what}");
        assert_eq!(
            serde_json::to_value(&past.totals).unwrap(),
            serde_json::to_value(&whole.totals).unwrap(),
            "{what}"
        );
    }
    // The worked day has grouped reports with more than one row in a group.
    assert!(grouped_pages > 0, "no page started inside a group");
}

fn view(w: &support::World, v: AnalyticsView, offset: i64, limit: i64) -> AnalyticsPage {
    mandi_reports::analytics::run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            sort: None,
            view: v,
            filters: AnalyticsFilters::default(),
            offset,
            limit,
        },
    )
    .unwrap()
}

#[test]
fn every_analytics_view_pages_with_the_count_and_the_totals_of_the_whole_filter() {
    let w = full_day();
    for v in [
        AnalyticsView::A,
        AnalyticsView::B,
        AnalyticsView::C,
        AnalyticsView::D,
        AnalyticsView::E,
        AnalyticsView::F,
        AnalyticsView::G,
        AnalyticsView::H,
        AnalyticsView::I,
        AnalyticsView::J,
        AnalyticsView::K,
    ] {
        let whole = view(&w, v, 0, 1_000);
        let all: Vec<Value> = whole
            .rows
            .iter()
            .map(|r| serde_json::to_value(r).unwrap())
            .collect();
        let what = format!("{v:?}");
        assert_eq!(
            usize::try_from(whole.total_count).unwrap(),
            all.len(),
            "{what}"
        );
        for size in PAGE_SIZES {
            let page = view(&w, v, 0, size);
            assert!(
                i64::try_from(page.rows.len()).unwrap() <= size,
                "{what} {size}"
            );
            assert_eq!(page.total_count, whole.total_count, "{what} {size}");
            assert_eq!(
                serde_json::to_value(&page.totals).unwrap(),
                serde_json::to_value(&whole.totals).unwrap(),
                "{what} {size}: totals over the whole filter"
            );
        }
        let mut paged = Vec::new();
        for offset in 0..whole.total_count {
            let page = view(&w, v, offset, 1);
            assert_eq!(page.rows.len(), 1, "{what} offset {offset}");
            assert_eq!(
                serde_json::to_value(&page.totals).unwrap(),
                serde_json::to_value(&whole.totals).unwrap(),
                "{what}: totals on page {offset}"
            );
            paged.extend(page.rows.iter().map(|r| serde_json::to_value(r).unwrap()));
        }
        assert_eq!(paged, all, "{what}: the pages are the whole list");
        let past = view(&w, v, whole.total_count + 50, 25);
        assert!(past.rows.is_empty(), "{what}");
        assert_eq!(past.total_count, whole.total_count, "{what}");
    }
}

#[test]
fn the_advances_report_pages_with_the_totals_over_every_seller() {
    let w = full_day();
    let page = |offset: u32, limit: u32| {
        vouchers::advances_report(
            &w.env.ctx,
            &AdvancesReportInput {
                seller_id: None,
                as_of: String::new(),
                offset,
                limit,
            },
        )
        .unwrap()
    };
    let whole = page(0, 0);
    // The worked day: رحیم was given 15,000 in advance.
    assert_eq!(
        (whole.total_count, whole.rows.len(), whole.limit),
        (1, 1, 100)
    );
    assert_eq!(whole.given_paisa.rupees(), 15_000);
    for limit in [25, 50, 100] {
        let p = page(0, limit);
        assert_eq!((p.total_count, p.rows.len(), p.limit), (1, 1, limit));
        assert_eq!(p.given_paisa, whole.given_paisa);
    }
    // Past the end: no rows, the same count and totals.
    let past = page(5, 25);
    assert!(past.rows.is_empty());
    assert_eq!((past.total_count, past.given_paisa), (1, whole.given_paisa));
}

#[test]
fn the_fruit_lots_page_by_offset_with_the_count_of_all_of_them() {
    let w = full_day();
    let lots = |offset: u32, limit: u32| {
        mandi_services::fruit::lots(
            &w.env.ctx,
            &mandi_services::core::fruit::FruitLotsInput {
                search: None,
                offset,
                limit,
            },
        )
        .unwrap()
    };
    let whole = lots(0, 0);
    assert_eq!(whole.limit, 100);
    let mut paged = Vec::new();
    for offset in 0..whole.total_count {
        let p = lots(u32::try_from(offset).unwrap(), 1);
        assert_eq!(p.total_count, whole.total_count);
        paged.extend(p.rows);
    }
    assert_eq!(paged, whole.rows);
    let past = lots(500, 25);
    assert!(past.rows.is_empty());
    assert_eq!(past.total_count, whole.total_count);
    let _ = json!(null);
}

#[test]
fn the_chart_sends_controls_with_a_count_and_the_accounts_a_page_at_a_time_fixtures() {
    // Plan 38 B1: `coa_tree` has no accounts; `coa_control_accounts` gives a control's page.
    let w = full_day();
    let tree = mandi_services::coa::tree(&w.env.ctx).unwrap();
    let buyers = tree
        .heads
        .iter()
        .flat_map(|h| &h.controls)
        .find(|c| c.code == "0304")
        .unwrap();
    assert_eq!(buyers.account_count, 4);
    pair("coa_tree", &json!(null), &tree);
    let request = json!({ "control_id": buyers.id, "offset": 0, "limit": 25 });
    let input: mandi_services::core::accounts::CoaControlAccountsInput =
        serde_json::from_value(request.clone()).unwrap();
    let page = mandi_services::coa::control_accounts(&w.env.ctx, &input).unwrap();
    assert_eq!((page.rows.len(), page.total_count), (4, 4));
    pair("coa_control_accounts_buyers", &request, &page);
    let second = json!({ "control_id": buyers.id, "offset": 2, "limit": 2 });
    let input: mandi_services::core::accounts::CoaControlAccountsInput =
        serde_json::from_value(second.clone()).unwrap();
    let page = mandi_services::coa::control_accounts(&w.env.ctx, &input).unwrap();
    assert_eq!((page.rows.len(), page.total_count, page.offset), (2, 4, 2));
    pair("coa_control_accounts_buyers_page_2", &second, &page);
}
