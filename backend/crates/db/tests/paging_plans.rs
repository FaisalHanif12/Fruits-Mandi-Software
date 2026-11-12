//! The query plans of the paged lists (plan 17b, PRD FR-19.11, NFR-SPD-24, Code Standards B18,
//! B31): a page and its totals search indexed columns, so the first page of a list over ten
//! years of data does not read every row of the table. `EXPLAIN QUERY PLAN` is read with the
//! same text the code runs, and the values bound are the ones a screen sends.
#![cfg(test)]

mod common;

use common::setup;
use mandi_db::bikri::saved::{
    ListFilter as BikriFilter, list_days_queries, list_query, list_totals_queries,
};
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

/// True if the plan reads a whole journal table: a `SCAN` of one of the aliases the party ledger
/// queries give them, with no index. (`SCAN days` is the party's own daily sums, kept in memory for the one
/// reading.)
fn scans_a_journal_table(text: &str) -> bool {
    text.lines().any(|line| {
        ["SCAN l", "SCAN e", "SCAN o", "SCAN r", "SCAN journal"]
            .iter()
            .any(|start| line.starts_with(start) && !line.contains("USING"))
    })
}

fn voucher_filter<'a>(
    voucher_type: Option<mandi_core::vouchers::VoucherType>,
    day: Option<&'a str>,
    account: Option<i64>,
) -> ListFilter<'a> {
    ListFilter {
        hide_cancelled: false,
        order_by: None,
        search: None,
        voucher_type,
        from: day,
        to: day,
        account_id: account,
    }
}

fn bikri_filter(day: Option<&str>) -> BikriFilter {
    BikriFilter {
        order_by: None,
        search: None,
        from: day.map(str::to_owned),
        to: day.map(str::to_owned),
        seller_id: None,
        item_id: None,
        status: None,
        number: None,
        truck_prefix: None,
        limit: 100,
        offset: 0,
        // The buyer filters of the find (plan 67) are not paged lists' business.
        ..BikriFilter::default()
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

/// Plan 72 P11 (PRD 12.7, Code Standards B18, NFR-SPD-24): the window of past بکری. Its page, its three
/// totals and its two day queries search the date index with a range and then the lots and the sale
/// lines of those بکری only. None reads a whole table: not `bikri_entries` (40,000 rows in ten years),
/// not `bikri_lots`, not `bikri_sales` (1,200,000 rows, NFR-SCL-1). No new index was needed.
#[test]
fn a_window_of_bikri_its_totals_and_its_days_search_the_date_index_with_a_range() {
    let (_dir, conn) = setup();
    let window = || BikriFilter {
        from: Some("2026-11-05".into()),
        to: Some("2026-11-11".into()),
        status: Some(mandi_core::bikri::BikriStatus::Saved),
        limit: 25,
        offset: 0,
        ..BikriFilter::default()
    };
    let range = "SEARCH e USING INDEX idx_bikri_entries_date (entry_date>? AND entry_date<?)";
    // Every filter the screen sets over a window: none, a seller, an item, the status set aside
    // (the switch "show everything"), and a number or truck search.
    let cases = [
        ("the window", window()),
        (
            "a seller",
            BikriFilter {
                seller_id: Some(7),
                ..window()
            },
        ),
        (
            "an item",
            BikriFilter {
                item_id: Some(9),
                ..window()
            },
        ),
        (
            "everything shown",
            BikriFilter {
                status: None,
                ..window()
            },
        ),
        (
            "a search",
            BikriFilter {
                search: Some("4021".into()),
                ..window()
            },
        ),
    ];
    for (name, filter) in cases {
        let (sql, binds) = list_query(&filter);
        let page = plan(&conn, &sql, &binds);
        assert!(
            !page.lines().any(|l| l.starts_with("SCAN ")),
            "{name} page:\n{page}"
        );
        // A search lets SQLite choose between its indexes; every other case uses the range.
        if name != "a search" {
            assert!(page.contains(range), "{name} page:\n{page}");
            assert!(
                page.contains("idx_bikri_lots_bikri") && page.contains("idx_bikri_sales_bikri"),
                "{name} page:\n{page}"
            );
        }
        for (which, (sql, binds)) in list_totals_queries(&filter)
            .into_iter()
            .chain(list_days_queries(&filter))
            .enumerate()
        {
            let text = plan(&conn, &sql, &binds);
            assert!(
                !text.lines().any(|l| l.starts_with("SCAN ")),
                "{name} query {which}:\n{text}"
            );
            if name != "a search" {
                assert!(
                    text.contains("USING INDEX idx_bikri_entries_date (entry_date>? AND entry_date<?)")
                        || text.contains(
                            "USING COVERING INDEX idx_bikri_entries_date (entry_date>? AND entry_date<?)"
                        ),
                    "{name} query {which}:\n{text}"
                );
            }
        }
    }
    // The two day queries group on the date index in its own order: neither sorts the rows, and the
    // gross reads the lots of those بکری through their own index.
    let [count, gross] = list_days_queries(&window());
    let count = plan(&conn, &count.0, &count.1);
    assert!(count.contains(range), "{count}");
    assert!(!count.contains("TEMP B-TREE"), "{count}");
    let gross = plan(&conn, &gross.0, &gross.1);
    assert!(
        gross.contains("SEARCH l USING INDEX idx_bikri_lots_bikri (bikri_id=?)"),
        "{gross}"
    );
    assert!(!gross.contains("TEMP B-TREE"), "{gross}");
}

#[test]
fn the_party_ledgers_queries_search_an_index() {
    use mandi_db::party_ledgers::{list_page_sql, statement_page_sql};
    let (_dir, conn) = setup();
    let none = Value::Null;
    // A statement reads the party's own lines through the covering index of the balances, never
    // every journal line (3,500,000 of them, NFR-SCL-1). The entry of each line is found by its key.
    // One query gives the page and the totals, so the lines are read once (D8 robustness MINOR 2).
    let page = plan(
        &conn,
        &statement_page_sql(),
        &[
            Value::Integer(7),
            none.clone(),
            none.clone(),
            Value::Integer(100),
            Value::Integer(0),
        ],
    );
    assert!(
        page.contains("USING COVERING INDEX idx_journal_lines_account_amounts (account_id=?)"),
        "{page}"
    );
    // The same-day test looks entries up by their key and by the reversal index, never by a scan.
    assert!(!scans_a_journal_table(&page), "{page}");
    // The party's lines are read ONCE: one search of the line index, however many sums read them.
    assert_eq!(
        page.matches("idx_journal_lines_account_amounts").count(),
        1,
        "{page}"
    );
    // The list of parties reads the accounts of one control and nothing else.
    let list = plan(
        &conn,
        &list_page_sql(),
        &[
            Value::Text("0304".into()),
            Value::Text(String::new()),
            Value::Text(String::new()),
            Value::Integer(100),
            Value::Integer(0),
        ],
    );
    assert!(!list.contains("journal_lines"), "{list}");
}
