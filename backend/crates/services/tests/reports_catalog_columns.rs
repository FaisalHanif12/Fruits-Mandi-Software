//! Plan 75 T1 (PRD FR-19.14): the column headings the catalogue lists for a report are the
//! headings the report itself shows when it runs. The catalogue and the run take their columns
//! from one definition, so this test proves it on a real database for every report the engine can
//! run: the default view of each report, with no optional column.
//!
//! The data is the D8 lot of E2E-65 (one بکری, one packing, one rate, no quality), so no register
//! has an optional column. R31, the partner statement, is built by the services from its own
//! lines and is checked with the statement tests; its columns come from `statement_columns`.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::reports::{PartyFilter, ReportFilters, ReportId, ReportRunInput, catalog, run};
use reports_world::*;

/// The filters a report needs to run at all: a party for the ledgers, the بکری for the sale book.
fn filters_for(id: ReportId, party: i64, bikri: i64) -> ReportFilters {
    let mut filters = ReportFilters::default();
    match id {
        ReportId::R1 => filters.bikri_id = Some(bikri),
        ReportId::R13 | ReportId::R14 => {
            filters.party = Some(PartyFilter {
                account_id: party,
                person_key: None,
            });
        }
        _ => {}
    }
    filters
}

#[test]
fn the_catalogue_columns_are_the_columns_the_report_shows() {
    let (w, bikri) = d8_paid_world();
    let mut checked = Vec::new();
    for entry in catalog() {
        // R31 is not run by the engine (its lines come from the services).
        if entry.id == ReportId::R31 {
            continue;
        }
        let page = run(
            w.env.ctx.db(),
            &ReportRunInput {
                id: entry.id,
                filters: filters_for(entry.id, w.rahim, bikri),
                offset: 0,
                limit: 0,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap_or_else(|e| panic!("{} did not run: {e:?}", entry.id.code()));
        let shown: Vec<String> = page.columns.iter().map(|c| c.title_id.clone()).collect();
        assert_eq!(
            entry.column_title_ids,
            shown,
            "{}: the catalogue and the report show different columns",
            entry.id.code()
        );
        checked.push(entry.id.code());
    }
    // Every report of the catalogue but R31 was run: none was skipped by accident.
    assert_eq!(checked.len() + 1, catalog().len(), "{checked:?}");
}
