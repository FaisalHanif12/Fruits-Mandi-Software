//! Gate lane T3 (plan 80), part 8: tampering with the tables that are NOT in a hash chain
//! (NFR-SEC-6, NFR-SEC-5; PRD FR-6.1, FR-6.3). Owner: `robustness`.
//!
//! The journal and the audit log are protected by keyed hash chains, and `robustness_tamper.rs`
//! proves that every edit of them is found. NFR-SEC-6 adds: "Other tables are protected by the
//! journal: every money table can be rebuilt and compared from the journal." The books check
//! (`health::check_books`) only looks at the trial balance difference and the two chains. This file
//! records what it sees when someone changes a money table outside the app and leaves the journal
//! alone.
//!
//! The first test is the control: an edit of a journal line is found. The second test prints, for
//! each other table, whether the books check found the edit. The third test is ignored: it asks
//! for the strict reading of NFR-SEC-6 and is the QUESTION T3-Q3 in the report.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_services::core::bikri::BikriSaveInput;
use mandi_services::db::rusqlite::Connection;
use mandi_services::{bikri, health};
use support::World;

fn world() -> World {
    let w = support::world();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::event1(&w),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    w
}

fn attacker(w: &World) -> Connection {
    let conn = raw(&w.dir);
    let names: Vec<String> = {
        let mut st = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger'")
            .unwrap();
        st.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    for name in names {
        conn.execute_batch(&format!("DROP TRIGGER IF EXISTS \"{name}\""))
            .unwrap();
    }
    conn.pragma_update(None, "foreign_keys", "OFF").unwrap();
    conn
}

const OUTSIDE_THE_CHAINS: &[(&str, &str)] = &[
    (
        "a sale line: the buyer rate raised by one rupee, with its value and total",
        "UPDATE bikri_sales SET buyer_rate_paisa = buyer_rate_paisa + 100, buyer_value_paisa = buyer_value_paisa + qty * 100, buyer_total_paisa = buyer_total_paisa + qty * 100, rate_diff_paisa = rate_diff_paisa + qty * 100 WHERE id = (SELECT MIN(id) FROM bikri_sales)",
    ),
    (
        "a sale line: the seller rate raised by one rupee, with its amount",
        "UPDATE bikri_sales SET seller_rate_paisa = seller_rate_paisa + 100, seller_amount_paisa = seller_amount_paisa + qty * 100, rate_diff_paisa = rate_diff_paisa - qty * 100 WHERE id = (SELECT MIN(id) FROM bikri_sales)",
    ),
    (
        "a partner share row raised by one rupee",
        "UPDATE partner_share_rows SET amount_paisa = amount_paisa + 100 WHERE id = (SELECT MIN(id) FROM partner_share_rows)",
    ),
    (
        "an analytics fact raised by one rupee",
        "UPDATE analytics_facts SET seller_amount_paisa = seller_amount_paisa + 100 WHERE id = (SELECT MIN(id) FROM analytics_facts)",
    ),
    (
        "a stock move changed by one",
        "UPDATE stock_moves SET qty = qty + 1 WHERE id = (SELECT MIN(id) FROM stock_moves)",
    ),
    (
        "a lot's kept gross and net raised by one rupee",
        "UPDATE bikri_lots SET gross_paisa = gross_paisa + 100, net_paisa = net_paisa + 100 WHERE id = (SELECT MIN(id) FROM bikri_lots)",
    ),
];

#[test]
fn control_an_edit_of_a_journal_line_is_found_by_the_books_check() {
    let w = world();
    assert!(health::check_books(&w.env.ctx).unwrap().is_ok());
    let conn = attacker(&w);
    conn.execute_batch(
        "UPDATE journal_lines SET dr_paisa = dr_paisa + 100 WHERE id = (SELECT MIN(id) FROM journal_lines WHERE dr_paisa > 0);
         UPDATE journal_lines SET cr_paisa = cr_paisa + 100 WHERE id = (SELECT MIN(id) FROM journal_lines WHERE cr_paisa > 0 AND entry_id = (SELECT entry_id FROM journal_lines WHERE dr_paisa > 0 ORDER BY id LIMIT 1));",
    )
    .unwrap();
    assert!(!health::check_books(&w.env.ctx).unwrap().is_ok());
}

#[test]
fn what_the_books_check_says_about_edits_of_the_tables_outside_the_chains() {
    let mut not_found = Vec::new();
    for (label, sql) in OUTSIDE_THE_CHAINS {
        let w = world();
        let conn = attacker(&w);
        conn.execute_batch(sql).unwrap();
        let changed = conn.changes();
        assert_eq!(changed, 1, "{label}: the edit changed {changed} rows");
        let found = !health::check_books(&w.env.ctx).unwrap().is_ok();
        println!("{}: {label}", if found { "found    " } else { "NOT FOUND" });
        if !found {
            not_found.push(*label);
        }
    }
    println!(
        "{} of {} edits outside the chains were not found",
        not_found.len(),
        OUTSIDE_THE_CHAINS.len()
    );
}

#[test]
fn every_edit_of_a_money_table_outside_the_app_is_found() {
    let mut not_found = Vec::new();
    for (label, sql) in OUTSIDE_THE_CHAINS {
        let w = world();
        let conn = attacker(&w);
        conn.execute_batch(sql).unwrap();
        if health::check_books(&w.env.ctx).unwrap().is_ok() {
            not_found.push(*label);
        }
    }
    assert!(
        not_found.is_empty(),
        "edits that no check found: {not_found:#?}"
    );
}
