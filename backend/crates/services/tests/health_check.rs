//! The daily books check (plan 22 early, NFR-REL-8, NFR-OBS-5, NFR-SEC-6): the trial balance
//! difference, the journal and audit chains, the stored latest result, and the number grouping
//! setting that goes with the same plan.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_services::core::money::{Grouping, Paisa};
use mandi_services::health::{self, ChainState};
use mandi_services::{settings, settings_ext};
use reports_world::*;
use serde_json::json;

fn rupees_of(p: Paisa) -> i64 {
    p.rupees()
}

#[test]
fn the_books_of_the_worked_day_are_right() {
    // E2E-10: 85,048 on each side after Events 1 to 5.
    let w = full_day();
    let books = health::check_books(&w.env.ctx).unwrap();
    assert_eq!(rupees_of(books.debit), 85_048);
    assert_eq!(rupees_of(books.credit), 85_048);
    assert_eq!(books.difference, Paisa::ZERO);
    assert_eq!(books.journal_chain, ChainState::Ok);
    assert_eq!(books.audit_chain, ChainState::Ok);
    assert!(books.is_ok());
    let entries: i64 = raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM journal_entries WHERE status = 'posted'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(books.journal_entries, entries);
    let rows: i64 = raw(&w.dir)
        .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
        .unwrap();
    assert_eq!(books.audit_rows, rows);
    // The stored summary holds the difference and the chains, and no account balance.
    let summary = books.summary_json().to_string();
    assert!(summary.contains("\"balanced\":true") && summary.contains("\"difference_paisa\":0"));
    assert!(
        !summary.contains("85048") && !summary.contains("8504800"),
        "{summary}"
    );
}

#[test]
fn a_clean_run_is_stored_and_writes_no_audit_row_but_a_bad_one_does() {
    let w = full_day();
    let ctx = &w.env.ctx;
    assert!(health::last(ctx).unwrap().is_none());
    let books = health::check_books(ctx).unwrap();
    let report = json!({ "checked_at": "2026-10-10T09:00:00.000Z", "ok": true });
    let audit_before = audit_count(&w.dir, "check_failed");
    health::store_last(ctx, &report, &books, Some(w.owner.user_id)).unwrap();
    assert_eq!(health::last(ctx).unwrap(), Some(report));
    assert_eq!(audit_count(&w.dir, "check_failed"), audit_before);

    // Someone changes an audit row outside the app: the chain breaks and the run says so.
    let conn = raw(&w.dir);
    conn.execute_batch("DROP TRIGGER audit_log_no_update;")
        .unwrap();
    conn.execute("UPDATE audit_log SET reason = 'changed' WHERE id = 1", [])
        .unwrap();
    let bad = health::check_books(ctx).unwrap();
    assert!(!bad.is_ok());
    assert!(matches!(&bad.audit_chain, ChainState::Broken(text) if text.contains('1')));
    assert_eq!(bad.journal_chain, ChainState::Ok);
    assert_eq!(bad.difference, Paisa::ZERO);
    let report = json!({ "checked_at": "2026-10-11T09:00:00.000Z", "ok": false });
    health::store_last(ctx, &report, &bad, None).unwrap();
    assert_eq!(health::last(ctx).unwrap(), Some(report));
    assert_eq!(audit_count(&w.dir, "check_failed"), audit_before + 1);
}

#[test]
fn a_changed_journal_entry_or_line_is_found() {
    let w = full_day();
    let ctx = &w.env.ctx;
    let conn = raw(&w.dir);
    // A changed total breaks the journal chain at that entry.
    conn.execute_batch("DROP TRIGGER journal_entries_posted_is_final;")
        .unwrap();
    let last: i64 = conn
        .query_row("SELECT MAX(id) FROM journal_entries", [], |r| r.get(0))
        .unwrap();
    conn.execute(
        "UPDATE journal_entries SET total_paisa = total_paisa + 100 WHERE id = ?1",
        [last],
    )
    .unwrap();
    let books = health::check_books(ctx).unwrap();
    assert!(
        matches!(&books.journal_chain, ChainState::Broken(text) if text.contains(&last.to_string()))
    );
    assert!(!books.is_ok());
    // A line made bigger outside the app: the books no longer balance by that amount.
    conn.execute_batch("DROP TRIGGER journal_lines_no_update;")
        .unwrap();
    conn.execute(
        "UPDATE journal_lines SET dr_paisa = dr_paisa + 100
         WHERE id = (SELECT MAX(id) FROM journal_lines WHERE dr_paisa > 0)",
        [],
    )
    .unwrap();
    let books = health::check_books(ctx).unwrap();
    assert_eq!(rupees_of(books.difference), 1);
    assert!(!books.is_ok());
}

#[test]
fn the_number_grouping_is_saved_with_an_audit_row_and_a_repeat_writes_nothing() {
    let w = worked_day();
    let ctx = &w.env.ctx;
    assert_eq!(
        settings::get(ctx).unwrap().number_grouping,
        Grouping::International
    );
    assert_eq!(
        settings_ext::number_grouping(ctx).unwrap(),
        Grouping::International
    );
    let before = audit_count(&w.dir, "edit");
    assert_eq!(
        settings_ext::set_number_grouping(ctx, &w.owner, Grouping::Lakh).unwrap(),
        Grouping::Lakh
    );
    assert_eq!(settings::get(ctx).unwrap().number_grouping, Grouping::Lakh);
    assert_eq!(audit_count(&w.dir, "edit"), before + 1);
    // The same value again: nothing written.
    settings_ext::set_number_grouping(ctx, &w.owner, Grouping::Lakh).unwrap();
    assert_eq!(audit_count(&w.dir, "edit"), before + 1);
    // The audit row holds the old and the new value.
    let (old, new): (String, String) = raw(&w.dir)
        .query_row(
            "SELECT old_json, new_json FROM audit_log WHERE action = 'edit' AND entity = 'settings'
             ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(old, r#"{"number_grouping":"international"}"#);
    assert_eq!(new, r#"{"number_grouping":"lakh"}"#);
    assert_chain_ok(&w.dir);
}

#[test]
fn a_line_of_an_account_with_no_row_is_still_counted() {
    // Money review: lines of an account that is not in the tree must not drop out of the check.
    let w = full_day();
    let ctx = &w.env.ctx;
    let conn = raw(&w.dir);
    conn.pragma_update(None, "foreign_keys", "OFF").unwrap();
    conn.execute_batch("DROP TRIGGER journal_lines_no_update;")
        .unwrap();
    // One debit line moves to an account that does not exist: the trial balance loses it.
    let (debit_line, credit_line): (i64, i64) = conn
        .query_row(
            "SELECT d.id, c.id FROM journal_lines d JOIN journal_lines c
               ON d.dr_paisa = c.cr_paisa AND d.dr_paisa > 0
             ORDER BY d.dr_paisa DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    conn.execute(
        "UPDATE journal_lines SET account_id = 999999 WHERE id = ?1",
        [debit_line],
    )
    .unwrap();
    let books = health::check_books(ctx).unwrap();
    assert_ne!(books.difference, Paisa::ZERO);
    assert!(!books.is_ok());
    // A debit and a credit of the same size both move: the trial balance still agrees with
    // itself, but the lines no longer do once the lost debit grows by 1 rupee.
    conn.execute(
        "UPDATE journal_lines SET account_id = 999999 WHERE id = ?1",
        [credit_line],
    )
    .unwrap();
    let books = health::check_books(ctx).unwrap();
    assert_eq!(books.difference, Paisa::ZERO, "a balanced pair moved away");
    conn.execute(
        "UPDATE journal_lines SET dr_paisa = dr_paisa + 100 WHERE id = ?1",
        [debit_line],
    )
    .unwrap();
    let books = health::check_books(ctx).unwrap();
    assert_eq!(rupees_of(books.difference), 1);
    assert!(!books.is_ok());
}

#[test]
fn a_check_that_cannot_run_replaces_an_old_ok_and_writes_an_audit_row() {
    let w = full_day();
    let ctx = &w.env.ctx;
    let books = health::check_books(ctx).unwrap();
    let good = json!({ "checked_at": "2026-10-10T09:00:00.000Z", "ok": true });
    health::store_last(ctx, &good, &books, None).unwrap();
    // A line that is not a whole rupee (written outside the app): the check cannot run.
    let conn = raw(&w.dir);
    conn.pragma_update(None, "ignore_check_constraints", "ON")
        .unwrap();
    conn.execute_batch("DROP TRIGGER journal_lines_no_update;")
        .unwrap();
    conn.execute(
        "UPDATE journal_lines SET dr_paisa = dr_paisa + 50
         WHERE id = (SELECT MAX(id) FROM journal_lines WHERE dr_paisa > 0)",
        [],
    )
    .unwrap();
    let error = health::check_books(ctx).unwrap_err();
    assert!(!error.to_string().contains("85048"), "{error}");
    let before = audit_count(&w.dir, "check_failed");
    let failed = json!({ "checked_at": "2026-10-11T09:00:00.000Z", "ok": false });
    health::store_failed(ctx, &failed, None).unwrap();
    assert_eq!(health::last(ctx).unwrap(), Some(failed));
    assert_eq!(audit_count(&w.dir, "check_failed"), before + 1);
    let new_json: String = raw(&w.dir)
        .query_row(
            "SELECT new_json FROM audit_log WHERE action = 'check_failed' ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(new_json, r#"{"ran":false}"#);
}
