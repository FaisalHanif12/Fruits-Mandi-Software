//! The query plans of the fast reads of plan 48 P48-S3 (Code Standards B18, migration 0016): each
//! statement must read the index it was made for, and never look up an entry or a matching row for
//! every line. A plan test does not need the big file: with no table statistics SQLite chooses the
//! same plan on an empty book as on 3.5 million lines (the plans were first read on the big file).
#![cfg(test)]

mod common;

use common::setup;
use mandi_db::ledger::account_totals_in_controls_text;
use mandi_db::party_reads::{
    BUYER_DEBIT_LINES_SQL, BeforeMethod, HELD_CREDITS_SQL, MATCHED_SUMS_SQL, OpenMethod,
    PartyScope, before_method_with, choose_open_method, left_out_single_key_sql, stray_keys_sql,
};
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

fn plan(conn: &Connection, sql: &str, values: &[Value]) -> String {
    let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
    statement
        .query_map(params_from_iter(values.iter()), |r| r.get::<_, String>(3))
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_balance_before_a_day_reads_the_amounts_index_and_lists_the_entries_to_leave_out() {
    let (_dir, conn) = setup();
    let (sql, values) = left_out_single_key_sql(&["0304", "03011"], "2026-10-10");
    let text = plan(&conn, &sql, &values);
    assert!(
        text.contains("COVERING INDEX idx_journal_lines_account_amounts"),
        "{text}"
    );
    assert!(
        !text.contains("journal_entries USING INTEGER PRIMARY KEY"),
        "an entry is looked up for each line:\n{text}"
    );
    // The entries to leave out are read from the unposted index and the day index.
    assert!(text.contains("idx_journal_entries_unposted"), "{text}");
    assert!(text.contains("idx_journal_entries_entry_date"), "{text}");
}

#[test]
fn the_totals_of_a_control_read_the_amounts_index() {
    let (_dir, conn) = setup();
    for (all_posted, as_of) in [(true, None), (false, None), (false, Some("?1"))] {
        let marks = vec![if as_of.is_some() { "?2" } else { "?1" }.to_owned()];
        let sql = account_totals_in_controls_text(&marks, all_posted, as_of);
        let mut values = Vec::new();
        if as_of.is_some() {
            values.push(Value::Text("2026-10-10".to_owned()));
        }
        values.push(Value::Text("0304".to_owned()));
        let text = plan(&conn, &sql, &values);
        assert!(
            text.contains("COVERING INDEX idx_journal_lines_account_amounts"),
            "{text}"
        );
        assert!(
            !text.contains("journal_entries USING INTEGER PRIMARY KEY"),
            "{text}"
        );
    }
}

#[test]
fn the_held_credits_start_from_the_entries_and_use_the_opening_index() {
    let (_dir, conn) = setup();
    let values = vec![
        Value::Text("03011".to_owned()),
        Value::Text("2026-10-03".to_owned()),
    ];
    let text = plan(&conn, HELD_CREDITS_SQL, &values);
    // The first thing read is the entries, not the lines of the sellers.
    assert!(
        text.lines()
            .next()
            .is_some_and(|l| l.starts_with("SEARCH e USING INTEGER PRIMARY KEY")),
        "{text}"
    );
    assert!(text.contains("idx_journal_entries_entry_date"), "{text}");
    assert!(text.contains("idx_journal_entries_opening"), "{text}");
    // The openings are NOT found by a scan of every entry.
    assert!(
        !text.contains("SCAN journal_entries\n") && !text.ends_with("SCAN journal_entries"),
        "every entry is scanned:\n{text}"
    );
    // The lines of those entries come from the entry id, not from the lines of every seller.
    assert!(
        text.contains("sqlite_autoindex_journal_lines_1 (entry_id=?)"),
        "{text}"
    );
}

#[test]
fn the_matched_sums_read_the_partial_covering_index() {
    let (_dir, conn) = setup();
    let text = plan(&conn, MATCHED_SUMS_SQL, &[]);
    assert!(
        text.contains("COVERING INDEX idx_bill_allocations_open_sum"),
        "{text}"
    );
}

#[test]
fn the_debit_lines_of_the_buyers_read_the_amounts_index() {
    let (_dir, conn) = setup();
    let text = plan(&conn, BUYER_DEBIT_LINES_SQL, &[]);
    assert!(
        text.contains("COVERING INDEX idx_journal_lines_account_amounts"),
        "{text}"
    );
}

#[test]
fn the_check_for_a_stray_person_key_seeks_the_person_index() {
    let (_dir, conn) = setup();
    let (sql, values) = stray_keys_sql(&["0304", "03011"]);
    let text = plan(&conn, &sql, &values);
    assert!(text.contains("idx_journal_lines_account_person"), "{text}");
}

// ---- which method is picked ----

#[test]
fn the_method_before_a_day_is_picked_by_the_entries_before_it_and_the_party_asked_for() {
    let (_dir, conn) = setup();
    let controls = ["0304", "03011"];
    let whole = PartyScope::default();
    // One posted entry on a buyer, dated 10-10-26.
    let entry = common::draft_entry(&conn, 100);
    common::line(&conn, entry, 1, "03040000", 100, 0).unwrap();
    common::line(&conn, entry, 2, "01010001", 0, 100).unwrap();
    common::post(&conn, entry).unwrap();
    let buyer: i64 = conn
        .query_row("SELECT id FROM accounts WHERE code = '03040000'", [], |r| {
            r.get(0)
        })
        .unwrap();

    // No entry is dated before the day: line by line, whatever the size.
    assert_eq!(
        before_method_with(&conn, &controls, "2026-10-10", whole, 0).unwrap(),
        BeforeMethod::PerLine
    );
    // The entry is before the day. With "early" set to 0 it is "many": left out. With 1 it is not.
    assert_eq!(
        before_method_with(&conn, &controls, "2026-10-11", whole, 0).unwrap(),
        BeforeMethod::LeftOut
    );
    assert_eq!(
        before_method_with(&conn, &controls, "2026-10-11", whole, 1).unwrap(),
        BeforeMethod::PerLine
    );
    // A party or a person asked for is always line by line.
    let one = PartyScope {
        account_id: Some(buyer),
        person_key: None,
    };
    let person = PartyScope {
        account_id: None,
        person_key: Some("x"),
    };
    for scope in [one, person] {
        assert_eq!(
            before_method_with(&conn, &controls, "2026-10-11", scope, 0).unwrap(),
            BeforeMethod::PerLine
        );
    }
    // A person key on an ordinary account: line by line again.
    let stray = common::draft_entry(&conn, 100);
    conn.execute(
        "INSERT INTO journal_lines (entry_id, line_no, account_id, chukta_person_key, dr_paisa, cr_paisa)
         VALUES (?1, 1, ?2, 'stray', 100, 0)",
        mandi_db::rusqlite::params![stray, buyer],
    )
    .unwrap();
    common::line(&conn, stray, 2, "01010001", 0, 100).unwrap();
    common::post(&conn, stray).unwrap();
    assert_eq!(
        before_method_with(&conn, &controls, "2026-10-11", whole, 0).unwrap(),
        BeforeMethod::PerLine
    );
}

#[test]
fn the_open_bills_method_needs_enough_buyers_and_lines_and_no_stray_key() {
    let (_dir, conn) = setup();
    let buyer: i64 = conn
        .query_row("SELECT id FROM accounts WHERE code = '03040000'", [], |r| {
            r.get(0)
        })
        .unwrap();
    let ids = [buyer];
    // PerLine is always line by line.
    let (chosen, _, _) = choose_open_method(&conn, &ids, OpenMethod::PerLine, 0, 0).unwrap();
    assert_eq!(chosen, OpenMethod::PerLine);
    // Asked for in bulk, an ordinary buyer is read in bulk.
    let (chosen, bulk, line) = choose_open_method(&conn, &ids, OpenMethod::Bulk, 0, 0).unwrap();
    assert_eq!(
        (chosen, bulk, line),
        (OpenMethod::Bulk, vec![buyer], vec![])
    );
    // Auto with too few accounts, or too few lines, stays line by line.
    let (chosen, _, _) = choose_open_method(&conn, &ids, OpenMethod::Auto, 2, 0).unwrap();
    assert_eq!(chosen, OpenMethod::PerLine, "one buyer is not many buyers");
    let (chosen, _, _) = choose_open_method(&conn, &ids, OpenMethod::Auto, 1, 10).unwrap();
    assert_eq!(
        chosen,
        OpenMethod::PerLine,
        "no debit line is not many lines"
    );
    // No account at all: nothing to read in bulk.
    let (chosen, _, _) = choose_open_method(&conn, &[], OpenMethod::Bulk, 0, 0).unwrap();
    assert_eq!(chosen, OpenMethod::PerLine);
}
