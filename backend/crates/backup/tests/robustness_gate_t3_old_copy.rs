//! Gate lane T3 (plan 80), part 7: a backup made by an OLDER app, with posted entries and their
//! keyed hash chain, is restored by the current app (NFR-REL-4, NFR-REL-13, NFR-INS-4, NFR-SEC-6,
//! PRD FR-20.9, FR-20.11). Owner: `robustness`.
//!
//! This is the case of a shop whose cloud copies were made before an update: the computer is lost
//! after the update, and the Owner restores the last copy on a new computer with the new app.
//! `restore_new_computer.rs` upgrades an EMPTY old copy. Here the old copy (schema version 4)
//! holds 40 posted entries. After the restore: the app is at the latest schema, every chain hash,
//! every journal line and every balance is what the old app wrote, both chains verify with the key
//! that travelled in the copy, and the books balance.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
mod harness;
mod new_computer;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod support;

use mandi_backup::api::{FirstRunRestoreInput, RestoreSource};
use mandi_services::db::audit::verify_audit_chain;
use mandi_services::db::rusqlite::Connection;
use mandi_services::health::{self, ChainState};
use mandi_services::ledger;
use new_computer::*;

/// Appends bytes the way the hash chain writes a text or a number (type byte, length, bytes).
fn put(out: &mut Vec<u8>, text: Option<&str>) {
    match text {
        None => out.push(0),
        Some(text) => {
            out.push(1);
            out.extend_from_slice(&(text.len() as u64).to_le_bytes());
            out.extend_from_slice(text.as_bytes());
        }
    }
}

/// Posts a general entry the way the app of version 4 did (see `robustness_gate_t3_migrations.rs`
/// in the db crate): the hashed bytes are those of version 4, with the key of 32 bytes of 7.
fn legacy_post(conn: &Connection, day: &str, from: &str, to: &str, paisa: i64) {
    let account = |code: &str| -> i64 {
        conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
            r.get(0)
        })
        .unwrap()
    };
    conn.execute(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, status, total_paisa)
         VALUES (?1, 'general', 'voucher', 1, 'draft', 0)",
        [day],
    )
    .unwrap();
    let entry = conn.last_insert_rowid();
    let lines = [(from, paisa, 0_i64), (to, 0, paisa)];
    for (index, (code, dr, cr)) in lines.iter().enumerate() {
        conn.execute(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, ?2, ?3, ?4, ?5)",
            (entry, i64::try_from(index).unwrap() + 1, account(code), dr, cr),
        )
        .unwrap();
    }
    let previous: String = conn
        .query_row(
            "SELECT COALESCE((SELECT chain_hash FROM journal_entries WHERE status = 'posted' ORDER BY id DESC LIMIT 1), ?1)",
            [mandi_db::audit::GENESIS_HASH],
            |r| r.get(0),
        )
        .unwrap();
    let mut bytes = Vec::new();
    put(&mut bytes, Some(&previous));
    put(&mut bytes, Some(&entry.to_string()));
    put(&mut bytes, Some(day));
    put(&mut bytes, Some("general"));
    put(&mut bytes, Some("voucher"));
    put(&mut bytes, Some("1"));
    put(&mut bytes, None);
    put(&mut bytes, None);
    put(&mut bytes, Some(&paisa.to_string()));
    put(&mut bytes, Some("2"));
    for (index, (code, dr, cr)) in lines.iter().enumerate() {
        put(&mut bytes, Some(&(index + 1).to_string()));
        put(&mut bytes, Some(&account(code).to_string()));
        put(&mut bytes, None);
        put(&mut bytes, Some(&dr.to_string()));
        put(&mut bytes, Some(&cr.to_string()));
        put(&mut bytes, Some("0"));
        put(&mut bytes, None);
    }
    let hash = blake3::keyed_hash(&[7_u8; 32], &bytes).to_hex().to_string();
    conn.execute(
        "UPDATE journal_entries SET status = 'posted', total_paisa = ?1, chain_hash = ?2 WHERE id = ?3",
        (paisa, &hash, entry),
    )
    .unwrap();
    conn.execute(
        "INSERT INTO settings (key, value_json) VALUES ('journal_head', ?1)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
        [format!("{{\"id\":{entry},\"hash\":\"{hash}\"}}")],
    )
    .unwrap();
}

fn rows(conn: &Connection, sql: &str) -> Vec<String> {
    let columns = conn.prepare(sql).unwrap().column_count();
    conn.prepare(sql)
        .unwrap()
        .query_map([], |r| {
            let mut text = String::new();
            for i in 0..columns {
                let value: mandi_services::db::rusqlite::types::Value = r.get(i)?;
                text.push_str(&format!("{value:?}|"));
            }
            Ok(text)
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn books(conn: &Connection) -> Vec<(&'static str, Vec<String>)> {
    vec![
        (
            "the entries and their chain hashes",
            rows(
                conn,
                "SELECT id, entry_date, status, total_paisa, chain_hash FROM journal_entries ORDER BY id",
            ),
        ),
        (
            "the journal lines",
            rows(
                conn,
                "SELECT id, entry_id, line_no, account_id, dr_paisa, cr_paisa FROM journal_lines ORDER BY id",
            ),
        ),
        (
            "the balances",
            rows(
                conn,
                "SELECT a.code, SUM(l.dr_paisa), SUM(l.cr_paisa) FROM accounts a JOIN journal_lines l ON l.account_id = a.id GROUP BY a.id ORDER BY a.code",
            ),
        ),
    ]
}

#[test]
fn a_copy_of_an_older_app_with_forty_posted_entries_is_restored_and_upgraded_with_every_hash_unchanged()
 {
    let old = old_computer();
    let new = new_computer(old.cloud.clone());
    let app = i64::try_from(new.ctx.db().migration().to).unwrap();
    let older = 4_i64;
    let dir = tempfile::tempdir().unwrap();
    let plain = dir.path().join("older.db");
    let before = {
        let mut conn = Connection::open(&plain).unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        mandi_db::migrate::migrations()
            .to_version(&mut conn, usize::try_from(older).unwrap())
            .unwrap();
        conn.execute(
            "INSERT INTO users (username, password_hash, display_name, role) \
             VALUES ('owner', 'not-a-real-hash', 'Owner', 'owner')",
            [],
        )
        .unwrap();
        // No line is on an income or shop expense account: every posting that touches one writes
        // partner share rows (PRD FR-6.5), and these entries are written by hand with none, so
        // the books check of the money tables (NFR-SEC-6 v1.21) would rightly find the split missing.
        let pairs = [
            ("01010001", "02010001"),
            ("06010001", "01010001"),
            ("02010001", "06010001"),
            ("02010001", "01010001"),
            ("01010001", "06010001"),
            ("06010001", "02010001"),
        ];
        for n in 0..40_usize {
            let (from, to) = pairs[n % pairs.len()];
            // A system account that may not exist in the seed of version 4 is skipped by the
            // choice of pairs above; the expense head 05010001 is made if it is missing.
            if from == "05010001" {
                let exists: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM accounts WHERE code = '05010001'",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap();
                if exists == 0 {
                    legacy_post(
                        &conn,
                        &format!("2026-10-{:02}", 1 + (n % 9)),
                        "01010001",
                        "04010001",
                        100,
                    );
                    continue;
                }
            }
            legacy_post(
                &conn,
                &format!("2026-10-{:02}", 1 + (n % 9)),
                from,
                to,
                100 * i64::try_from(37 + n * 13).unwrap(),
            );
        }
        let state = books(&conn);
        let _: String = conn
            .pragma_update_and_check(None, "journal_mode", "DELETE", |row| row.get(0))
            .unwrap();
        state
    };
    assert_eq!(before[0].1.len(), 40);
    let key = mandi_backup::key::BackupKey::parse(&old.printed_key).unwrap();
    let name = put_crafted_copy(
        &old.cloud,
        &key,
        &plain,
        "20261011T090000000Z",
        Some(&old_chain_key()),
        true,
    );
    let source: RestoreSource = cloud_source(&old);
    let done = new
        .service
        .first_run_restore(
            &FirstRunRestoreInput {
                source,
                copy: name,
                key: old.printed_key.clone(),
            },
            &|_| {},
        )
        .unwrap();
    assert_eq!(done.upgraded_from, Some(older));
    let version: i64 = new
        .ctx
        .db()
        .read(|c| {
            c.query_row("PRAGMA user_version", [], |r| r.get(0))
                .map_err(|e| e.to_string())
        })
        .unwrap();
    assert_eq!(version, app);
    // Every entry, line, balance and chain hash is what the old app wrote.
    let after = new
        .ctx
        .db()
        .read(|c| -> Result<_, String> { Ok(books(c)) })
        .unwrap();
    for ((label, was), (_, now)) in before.iter().zip(after.iter()) {
        assert_eq!(
            was, now,
            "{label} moved in the restore and upgrade from {older} to {app}"
        );
    }
    // Both chains verify with the key that travelled in the copy, and the books balance.
    ledger::verify_journal_chain(&new.ctx).expect("the journal chain verifies");
    let chain_key = new
        .keys
        .current()
        .expect("the new computer has the chain key");
    new.ctx
        .db()
        .read(|conn| verify_audit_chain(conn, &chain_key).map_err(|e| e.to_string()))
        .expect("the audit chain verifies");
    let check = health::check_books(&new.ctx).unwrap();
    assert_eq!(check.journal_chain, ChainState::Ok);
    assert_eq!(check.audit_chain, ChainState::Ok);
    assert!(check.is_ok(), "{check:?}");
    assert_eq!(check.difference, mandi_services::core::money::Paisa::ZERO);
}
