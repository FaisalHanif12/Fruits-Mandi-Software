//! The speed fixes of the plan 80 gate read with indexes (Code Standards B18, NFR-SPD-24): the query
//! plan of each new read is checked, so a change to a query or an index that makes one scan a big
//! table again fails here. The plans are those of the app's own file: no table statistics, so the
//! order of the tables is fixed in the SQL (`CROSS JOIN`, `INDEXED BY`) and the indexes are the ones
//! of the migrations (the sale register's covering index is migration 28).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::analytics::plans as analytics_plans;
use mandi_reports::reports::plans::*;
use mandi_services::db::rusqlite::params_from_iter;
use mandi_services::db::rusqlite::types::Value;
use reports_world::*;

fn plan_of(conn: &mandi_services::db::rusqlite::Connection, query: (String, Vec<Value>)) -> String {
    let mut statement = conn
        .prepare(&format!("EXPLAIN QUERY PLAN {}", query.0))
        .unwrap();
    let rows = statement
        .query_map(params_from_iter(query.1.iter()), |r| r.get::<_, String>(3))
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    rows.join(" | ")
}

#[test]
fn the_sale_registers_add_up_a_long_range_from_the_covering_index() {
    let w = full_day();
    let conn = raw(&w.dir);
    let day = Some("2016-10-11");
    let end = Some("2026-10-10");
    for (name, query) in [
        ("R2", r2_totals_wide(day, end)),
        ("R3", r3_r4_totals_wide(day, end, true)),
        ("R4", r3_r4_totals_wide(day, end, false)),
    ] {
        let plan = plan_of(&conn, query);
        assert!(
            plan.contains("SCAN s USING COVERING INDEX idx_bikri_sales_register"),
            "{name}: {plan}"
        );
        assert!(
            plan.contains("SEARCH je USING INTEGER PRIMARY KEY"),
            "{name}: {plan}"
        );
        assert!(
            plan.contains("SEARCH l USING INTEGER PRIMARY KEY"),
            "{name}: {plan}"
        );
        assert!(
            !plan.contains("SCAN je") && !plan.contains("SCAN l "),
            "{name}: {plan}"
        );
    }
}

#[test]
fn the_income_report_reads_a_long_range_from_the_income_accounts() {
    let w = full_day();
    let conn = raw(&w.dir);
    let long = plan_of(
        &conn,
        r28_groups(Some("2016-10-11"), Some("2026-10-10"), true),
    );
    assert!(
        long.contains("COVERING INDEX idx_journal_lines_account_amounts (account_id=?)"),
        "{long}"
    );
    assert!(
        long.contains("SEARCH e USING INTEGER PRIMARY KEY"),
        "{long}"
    );
    assert!(
        !long.contains("SCAN l") && !long.contains("SCAN e"),
        "{long}"
    );
    // A short range is still found through the date index of the entries.
    let short = plan_of(
        &conn,
        r28_groups(Some("2026-10-10"), Some("2026-10-10"), false),
    );
    assert!(short.contains("idx_journal_entries_entry_date"), "{short}");
    assert!(!short.contains("SCAN "), "{short}");
}

#[test]
fn the_statement_of_a_party_reads_its_entries_by_the_account_index() {
    let w = full_day();
    let conn = raw(&w.dir);
    let account = w.akram;
    let sums = plan_of(&conn, statement_entry_sums(account, None, None));
    assert!(
        sums.contains("COVERING INDEX idx_journal_lines_account_amounts (account_id=?)"),
        "{sums}"
    );
    assert!(
        sums.contains("SEARCH e USING INTEGER PRIMARY KEY"),
        "{sums}"
    );
    assert!(!sums.contains("SCAN "), "{sums}");
    let lines = plan_of(&conn, statement_lines_of_entries(account, &[1, 2, 3]));
    assert!(
        lines.contains("SEARCH e USING INTEGER PRIMARY KEY"),
        "{lines}"
    );
    // Each entry's lines by a point look-up: by the entry's own index or by the account's.
    assert!(
        lines.contains("SEARCH l USING") && lines.contains("entry_id=?"),
        "{lines}"
    );
    assert!(!lines.contains("SCAN "), "{lines}");
}

#[test]
fn the_party_day_sheet_reads_the_lines_of_the_entries_of_the_day() {
    let w = full_day();
    let conn = raw(&w.dir);
    let plan = plan_of(
        &conn,
        (r35_day_lines(), vec![Value::Null, Value::Null, Value::Null]),
    );
    assert!(
        plan.contains("SEARCH e USING INDEX idx_journal_entries_entry_date (entry_date=?)"),
        "{plan}"
    );
    assert!(
        plan.contains("SEARCH l USING INDEX sqlite_autoindex_journal_lines_1 (entry_id=?)"),
        "{plan}"
    );
    assert!(!plan.contains("SCAN "), "{plan}");
}

#[test]
fn the_views_read_the_facts_of_a_range_from_a_covering_index() {
    let w = full_day();
    let conn = raw(&w.dir);
    let plan = plan_of(
        &conn,
        analytics_plans::facts_rows(Some("2016-10-11"), Some("2026-10-10")),
    );
    assert!(
        plan.contains("COVERING INDEX idx_analytics_facts_") && plan.contains("entry_date>?"),
        "{plan}"
    );
    assert!(!plan.contains("SCAN f"), "{plan}");
}
