//! The query plans of the paged lists (plan 17b, PRD FR-19.11, NFR-SPD-24, Code Standards B18,
//! B31): a page and its totals search indexed columns, so the first page of a list over ten
//! years of data does not read every row of the table. `EXPLAIN QUERY PLAN` is read with the
//! same text the code runs, and the values bound are the ones a screen sends.
#![cfg(test)]

mod common;

use common::setup;
use mandi_db::bikri::saved::{ListFilter as BikriFilter, list_query, list_totals_queries};
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};
use mandi_db::vouchers::{ListFilter, list_page_query, list_totals_query};

/// The plan of a statement as one text.
fn plan(conn: &Connection, sql: &str, binds: &[Value]) -> String {
    let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
    statement
        .query_map(params_from_iter(binds.iter()), |r| r.get::<_, String>(3))
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>()
        .join("\n")
}

/// True if the plan reads a whole table: a `SCAN` that names no index.
fn scans_a_whole_table(text: &str) -> bool {
    text.lines()
        .any(|line| line.starts_with("SCAN ") && !line.contains("USING"))
}

fn voucher_filter<'a>(
    voucher_type: Option<mandi_core::vouchers::VoucherType>,
    day: Option<&'a str>,
    account: Option<i64>,
) -> ListFilter<'a> {
    ListFilter {
        voucher_type,
        from: day,
        to: day,
        account_id: account,
        person_key: None,
    }
}

fn bikri_filter(day: Option<&str>) -> BikriFilter {
    BikriFilter {
        from: day.map(str::to_owned),
        to: day.map(str::to_owned),
        seller_id: None,
        item_id: None,
        status: None,
        number: None,
        truck_prefix: None,
        limit: 100,
        offset: 0,
    }
}

#[test]
fn a_page_of_vouchers_and_its_totals_search_an_index() {
    let (_dir, conn) = setup();
    let receipt = Some(mandi_core::vouchers::VoucherType::Receipt);
    let day = Some("2026-10-10");
    // (the filter, the index its page and its totals must search)
    for (name, filter, index) in [
        (
            "one day",
            voucher_filter(None, day, None),
            "idx_vouchers_date",
        ),
        (
            "a type and a day",
            voucher_filter(receipt, day, None),
            "sqlite_autoindex_vouchers_1 (type=?)",
        ),
        (
            "one account",
            voucher_filter(None, None, Some(7)),
            "idx_vouchers_account",
        ),
    ] {
        let (sql, binds) = list_page_query(&filter, 0, 100);
        let page = plan(&conn, &sql, &binds);
        assert!(
            page.contains(&format!("SEARCH v USING INDEX {index}")),
            "{name}:\n{page}"
        );
        let (sql, binds) = list_totals_query(&filter);
        let totals = plan(&conn, &sql, &binds);
        assert!(
            totals.contains(&format!("SEARCH v USING INDEX {index}")),
            "{name}:\n{totals}"
        );
        assert!(
            !scans_a_whole_table(&page) && !scans_a_whole_table(&totals),
            "{name}"
        );
    }
    // A type alone searches the unique index of the type and its number.
    let (sql, binds) = list_page_query(&voucher_filter(receipt, None, None), 0, 100);
    let page = plan(&conn, &sql, &binds);
    assert!(
        page.contains("SEARCH v USING INDEX sqlite_autoindex_vouchers_1 (type=?)"),
        "{page}"
    );
    // No filter: the page reads the date index in order and stops after the page, with no sort of
    // the whole table (only the vouchers of one day are sorted by number).
    let (sql, binds) = list_page_query(&voucher_filter(None, None, None), 0, 100);
    let page = plan(&conn, &sql, &binds);
    assert!(
        page.contains("SCAN v USING INDEX idx_vouchers_date"),
        "{page}"
    );
    assert!(!page.contains("USE TEMP B-TREE FOR ORDER BY"), "{page}");
}

#[test]
fn a_page_of_bikri_and_its_totals_search_an_index() {
    let (_dir, conn) = setup();
    // One day: the page and the three totals search the date index and then the lots and the
    // sale lines of those بکری only, never every sale line (1,200,000 of them, NFR-SCL-1).
    let filter = bikri_filter(Some("2026-10-10"));
    let (sql, binds) = list_query(&filter);
    let page = plan(&conn, &sql, &binds);
    assert!(
        page.contains("SEARCH e USING INDEX idx_bikri_entries_date"),
        "{page}"
    );
    assert!(
        page.contains("idx_bikri_lots_bikri") && page.contains("idx_bikri_sales_bikri"),
        "{page}"
    );
    assert!(!scans_a_whole_table(&page), "{page}");
    let [count, lots, sales] = list_totals_queries(&filter);
    for (name, (sql, binds), inner) in [
        ("count", count, None),
        ("lots", lots, Some("idx_bikri_lots_bikri")),
        ("sales", sales, Some("idx_bikri_sales_bikri")),
    ] {
        let text = plan(&conn, &sql, &binds);
        assert!(
            text.contains("USING INDEX idx_bikri_entries_date")
                || text.contains("USING COVERING INDEX idx_bikri_entries_date"),
            "{name}:\n{text}"
        );
        assert!(!scans_a_whole_table(&text), "{name}:\n{text}");
        if let Some(index) = inner {
            assert!(text.contains(index), "{name}:\n{text}");
        }
    }
    // No filter: the page reads the date index backwards and stops after the page.
    let (sql, binds) = list_query(&bikri_filter(None));
    let page = plan(&conn, &sql, &binds);
    assert!(
        page.contains("SCAN e USING INDEX idx_bikri_entries_date"),
        "{page}"
    );
}
