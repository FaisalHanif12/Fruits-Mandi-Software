//! All or nothing (PRD FR-12.8, NFR-REL-1, E20, plan 13): a failure forced after each step of the
//! save leaves no row of the بکری, the journal, the matching, the stock, the facts or the audit,
//! keeps the draft, and the next save gets the same number.
//!
//! The failure is a SQLite trigger made on a second connection: it refuses the next INSERT,
//! UPDATE or DELETE on one table, exactly where the save has got to. The same draft has two lots
//! so every step runs: an advance adjusted, a cash sale pinned, share rows, stock, facts.

use mandi_services::bikri;
use mandi_services::core::bikri::BikriDraft;
use mandi_services::core::drafts::DraftSaveInput;
use mandi_services::drafts;

use crate::common::{assert_chain_ok, raw};
use crate::helpers::*;
use crate::support::*;

/// A بکری that touches every table: Event 1 (advance adjusted) and Event 2's lot (a cash sale
/// of a one-time buyer, pinned).
fn big_draft(w: &World) -> BikriDraft {
    let mut line = sale("s-9", w.pukhta_buyer, "10", "1200");
    line.pukhta = person("شاہد", "Shahid", "0300-0000000");
    let mut second = lot(w, "lot-2", w.karim, w.apple, "10");
    second.freight = Some("1,000".into());
    second.cash_paid = None;
    second.sales = vec![line];
    let mut d = event1(w);
    d.lots.push(second);
    d
}

/// Every step of the save that writes: the table and the kind of change.
const STEPS: &[(&str, &str)] = &[
    ("bikri_entries", "INSERT"),
    ("bikri_lots", "INSERT"),
    ("journal_entries", "INSERT"),
    ("journal_lines", "INSERT"),
    ("journal_entries", "UPDATE"),
    // The head of the journal chain is written right after the entry is posted.
    ("settings", "INSERT"),
    ("partner_share_rows", "INSERT"),
    ("bikri_lots", "UPDATE"),
    ("bikri_sales", "INSERT"),
    ("stock_moves", "INSERT"),
    ("analytics_facts", "INSERT"),
    ("bill_allocations", "INSERT"),
    ("advance_allocations", "INSERT"),
    ("audit_log", "INSERT"),
    ("drafts", "DELETE"),
];

#[test]
fn a_failure_forced_after_every_step_leaves_nothing_behind() {
    let w = world();
    give_advance(&w, w.rahim, "15,000");
    drafts::save(
        &w.env.ctx,
        &w.owner,
        &DraftSaveInput {
            screen: "bikri".into(),
            payload_json: "{\"typing\":true}".into(),
        },
    )
    .unwrap();
    let d = big_draft(&w);
    let conn = raw(&w.dir);
    let before = snapshot(&conn);
    let cash_before = balance(&w, support_id(&w, "01010001"));
    for (table, event) in STEPS {
        conn.execute_batch(&format!(
            "CREATE TRIGGER force_fail BEFORE {event} ON {table} BEGIN SELECT RAISE(ABORT, 'forced'); END;"
        ))
        .unwrap();
        let result = save(&w, &d);
        conn.execute_batch("DROP TRIGGER force_fail;").unwrap();
        assert!(
            result.is_err(),
            "the save went through although {event} on {table} was refused"
        );
        assert_eq!(
            snapshot(&conn),
            before,
            "rows were left behind after a failure at {event} on {table}"
        );
        assert_eq!(
            balance(&w, support_id(&w, "01010001")),
            cash_before,
            "{table}"
        );
    }
    // The draft is still there and the books are as before.
    let draft_row = drafts::get(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::drafts::DraftGetInput {
            screen: "bikri".into(),
        },
    )
    .unwrap();
    assert!(draft_row.is_some());
    // The next save works, takes number 1 (no number was used) and removes the draft.
    let saved = save_ok(&w, &d);
    assert_eq!(saved.number, 1);
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM drafts"), 0);
    let trial = mandi_services::ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    mandi_services::ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
    let _ = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
}

fn support_id(w: &World, code: &str) -> i64 {
    id_of(&w.env, code)
}
