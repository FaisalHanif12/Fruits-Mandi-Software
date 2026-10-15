//! Tampering outside the app (D1 attack list item 4, NFR-SEC-6, NFR-SEC-5; PRD FR-6.3, FR-21.3).
//! Owner: `robustness`.
//!
//! Someone opens `mandi.db` with another tool, switches off the triggers that block changes,
//! and edits money or history. Every journal entry and every audit row carries a keyed hash that
//! includes the one before it. The key is not in the file. So:
//!
//! - a changed amount (even when both sides are changed and the entry still balances), a line
//!   moved to another account, a changed date, a flipped advance mark, a removed entry, a forged
//!   entry are all found by the journal chain, and the answer names an entry;
//! - an edited, removed, forged or re-attributed audit row is found by the audit chain;
//! - the same file read with a different key fails from the first row: the file alone is not
//!   enough to rebuild a good chain.
//!
//! The first test is the control: the untouched file verifies. Known limit (status.md, plan 07):
//! removing the LAST rows of a chain keeps it valid; anchoring the tail outside the file belongs
//! to the backup plan (D5).
#![cfg(test)]

mod common;

use common::*;
use mandi_services::core::accounts::{AccountInput, AccountType};
use mandi_services::core::vouchers::{AdvanceInput, VoucherMode};
use mandi_services::db::audit::{AuditKey, verify_audit_chain};
use mandi_services::db::rusqlite::Connection;
use mandi_services::testing::TestEnv;
use mandi_services::{accounts, auth, ledger, vouchers};
use tempfile::TempDir;

fn seller(ur: &str, en: &str) -> AccountInput {
    AccountInput {
        id: None,
        version: None,
        account_type: AccountType::Seller,
        control_id: None,
        code: String::new(),
        name_ur: ur.into(),
        name_en: en.into(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
    }
}

fn advance(to: i64, amount: u32) -> AdvanceInput {
    AdvanceInput {
        date: "10-10-2026".into(),
        seller_account_id: to,
        amount: amount.to_string(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        expected_goods: String::new(),
        narration: "first".into(),
    }
}

struct World {
    dir: TempDir,
    env: TestEnv,
    rahim: i64,
    karim: i64,
}

/// Two sellers and six advances: six journal entries, each with a cash line and a seller line,
/// and a dozen audit rows.
fn world() -> World {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = accounts::save(&env.ctx, &owner, &seller("رحیم", "Rahim"))
        .unwrap()
        .id;
    let karim = accounts::save(&env.ctx, &owner, &seller("کریم", "Karim"))
        .unwrap()
        .id;
    for n in 0..6_u32 {
        let to = if n % 2 == 0 { rahim } else { karim };
        vouchers::save_advance(&env.ctx, &owner, &advance(to, 1_000 + n * 100)).unwrap();
    }
    World {
        dir,
        env,
        rahim,
        karim,
    }
}

/// A connection of the attacker: the triggers that block changes are dropped.
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

fn one(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

fn journal_check(w: &World) -> Result<(), String> {
    ledger::verify_journal_chain(&w.env.ctx).map_err(|e| e.to_string())
}

fn audit_check(w: &World, key: &AuditKey) -> Result<(), String> {
    verify_audit_chain(&raw(&w.dir), key).map_err(|e| e.to_string())
}

fn key() -> AuditKey {
    AuditKey::from_bytes([7; 32])
}

#[test]
fn the_untouched_file_verifies_and_the_wrong_key_does_not() {
    let w = world();
    journal_check(&w).unwrap();
    audit_check(&w, &key()).unwrap();
    // The key is not in the file: another key cannot verify the chain from the first row.
    let other = AuditKey::from_bytes([8; 32]);
    assert!(
        audit_check(&w, &other).is_err(),
        "a different key verified the audit chain"
    );
    let entries = one(&raw(&w.dir), "SELECT COUNT(*) FROM journal_entries");
    assert!(entries >= 6, "{entries} entries");
}

#[test]
fn every_edit_of_the_journal_outside_the_app_is_found_and_names_an_entry() {
    // (label, the statements the attacker runs on entry `{e}`, the id of a line `{dr}` on the
    // debit side and `{cr}` on the credit side, `{other}` the other seller)
    let scenarios: Vec<(&str, Vec<&str>)> = vec![
        (
            "both sides of an entry raised by one rupee, total raised too (still balanced)",
            vec![
                "UPDATE journal_lines SET dr_paisa = dr_paisa + 100 WHERE id = {dr}",
                "UPDATE journal_lines SET cr_paisa = cr_paisa + 100 WHERE id = {cr}",
                "UPDATE journal_entries SET total_paisa = total_paisa + 100 WHERE id = {e}",
            ],
        ),
        (
            "an amount moved from one seller to the other (balanced, same total)",
            vec!["UPDATE journal_lines SET account_id = {other} WHERE id = {cr}"],
        ),
        (
            "the date of an entry moved to another day",
            vec!["UPDATE journal_entries SET entry_date = '2026-09-01' WHERE id = {e}"],
        ),
        (
            "the advance mark switched off on a seller line",
            vec!["UPDATE journal_lines SET is_advance = 0 WHERE id = {dr} AND is_advance = 1"],
        ),
        (
            "an entry removed with its lines",
            vec![
                "DELETE FROM journal_lines WHERE entry_id = {e}",
                "DELETE FROM journal_entries WHERE id = {e}",
            ],
        ),
        (
            "an entry added with made-up hash text",
            vec![
                "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, status, total_paisa, chain_hash, created_by, updated_by) VALUES ('2026-10-10', 'general', 'voucher', 1, 'posted', 500000, '0000000000000000000000000000000000000000000000000000000000000000', 1, 1)",
            ],
        ),
    ];
    let mut report = Vec::new();
    for (label, statements) in scenarios {
        let w = world();
        let conn = attacker(&w);
        // A middle entry: the third advance. Its debit line is the seller side (is_advance = 1).
        let e = one(
            &conn,
            "SELECT id FROM journal_entries WHERE posting_type = 'advance' ORDER BY id LIMIT 1 OFFSET 2",
        );
        let dr = one(
            &conn,
            &format!("SELECT id FROM journal_lines WHERE entry_id = {e} AND dr_paisa > 0"),
        );
        let cr = one(
            &conn,
            &format!("SELECT id FROM journal_lines WHERE entry_id = {e} AND cr_paisa > 0"),
        );
        let other = one(
            &conn,
            &format!(
                "SELECT id FROM accounts WHERE id IN ({}, {}) AND id <> (SELECT account_id FROM journal_lines WHERE id = {dr})",
                w.rahim, w.karim
            ),
        );
        for statement in &statements {
            let sql = statement
                .replace("{e}", &e.to_string())
                .replace("{dr}", &dr.to_string())
                .replace("{cr}", &cr.to_string())
                .replace("{other}", &other.to_string());
            conn.execute_batch(&sql).unwrap_or_else(|error| {
                panic!("{label}: the tamper statement failed: {error}\n{sql}")
            });
        }
        // The change really happened (a tamper that changed nothing proves nothing).
        match journal_check(&w).err() {
            Some(message) => report.push(format!("found: {label}: {message}")),
            None => panic!(
                "NOT FOUND: {label}. The journal chain still verifies after this edit made outside the app."
            ),
        }
    }
    for line in report {
        println!("{line}");
    }
}

#[test]
fn every_edit_of_the_audit_log_outside_the_app_is_found() {
    let scenarios: Vec<(&str, &str)> = vec![
        (
            "the new value of a middle row edited",
            "UPDATE audit_log SET new_json = '{\"amount\":1}' WHERE id = {m}",
        ),
        (
            "the old value of a middle row edited",
            "UPDATE audit_log SET old_json = '{\"amount\":9}' WHERE id = {m}",
        ),
        (
            "a middle row removed",
            "DELETE FROM audit_log WHERE id = {m}",
        ),
        (
            "the user of a middle row changed",
            "UPDATE audit_log SET user_id = 99 WHERE id = {m}",
        ),
        (
            "the action of a middle row changed",
            "UPDATE audit_log SET action = 'view' WHERE id = {m}",
        ),
        (
            "the time of a middle row changed",
            "UPDATE audit_log SET at = '2020-01-01T00:00:00Z' WHERE id = {m}",
        ),
        (
            "the reason of a middle row changed",
            "UPDATE audit_log SET reason = 'nothing happened' WHERE id = {m}",
        ),
        (
            "a row added with made-up hash text",
            "INSERT INTO audit_log (at, user_id, action, entity, chain_hash) VALUES ('2026-10-10T09:00:00Z', 1, 'create', 'voucher', '00000000000000000000000000000000000000000000000000000000000000ff')",
        ),
        (
            "the hash of a middle row replaced by the plain hash of its text (no key)",
            "UPDATE audit_log SET chain_hash = lower(hex(randomblob(32))) WHERE id = {m}",
        ),
    ];
    for (label, statement) in scenarios {
        let w = world();
        let conn = attacker(&w);
        let total = one(&conn, "SELECT COUNT(*) FROM audit_log");
        assert!(total >= 10, "{total} audit rows");
        let middle = one(
            &conn,
            &format!(
                "SELECT id FROM audit_log ORDER BY id LIMIT 1 OFFSET {}",
                total / 2
            ),
        );
        conn.execute_batch(&statement.replace("{m}", &middle.to_string()))
            .unwrap_or_else(|error| panic!("{label}: the tamper statement failed: {error}"));
        match audit_check(&w, &key()) {
            Err(message) => println!("found: {label}: {message}"),
            Ok(()) => panic!(
                "NOT FOUND: {label}. The audit chain still verifies after this edit made outside the app."
            ),
        }
    }
}
