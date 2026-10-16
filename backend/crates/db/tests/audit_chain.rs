//! The audit writer and its hash chain (plan 03 Section 4.5, acceptance item 5, NFR-SEC-5/6).
#![cfg(test)]

mod common;

use mandi_db::DbError;
use mandi_db::audit::{
    AuditEntry, AuditKey, ChainBreak, GENESIS_HASH, VerifyError, verify_audit_chain,
};
use mandi_db::rusqlite::Connection;
use serde_json::json;
use tempfile::{TempDir, tempdir};

fn entry(action: &str, entity_id: i64) -> AuditEntry {
    AuditEntry {
        user_id: None,
        action: action.to_owned(),
        entity: "account".to_owned(),
        entity_id: Some(entity_id),
        old_json: Some(json!({ "credit_limit_paisa": 5_000_000 })),
        new_json: Some(json!({ "credit_limit_paisa": 7_500_000 })),
        reason: Some("حد بڑھائی".to_owned()),
    }
}

fn write_rows(dir: &TempDir, count: i64) -> mandi_db::Db {
    let db = common::open(dir.path());
    for i in 1..=count {
        db.write(|tx| -> Result<(), DbError> {
            db.auditor().write(tx, &entry("edit", i))?;
            Ok(())
        })
        .unwrap();
    }
    db
}

fn verify(conn: &Connection) -> Result<(), VerifyError> {
    verify_audit_chain(conn, &common::test_key())
}

/// Tampering outside the app: someone opens the file with another tool. The triggers are
/// dropped first, as they could be by anyone with file access. The chain still catches it.
fn drop_audit_triggers(conn: &Connection) {
    conn.execute_batch("DROP TRIGGER audit_log_no_update; DROP TRIGGER audit_log_no_delete;")
        .unwrap();
}

#[test]
fn rows_are_written_with_time_user_and_a_chain() {
    let dir = tempdir().unwrap();
    let db = common::open(dir.path());
    let id = db
        .write(|tx| -> Result<i64, DbError> {
            db.auditor().write(
                tx,
                &AuditEntry {
                    user_id: None,
                    ..entry("create", 5)
                },
            )
        })
        .unwrap();
    assert_eq!(id, 1);
    let conn = common::raw(dir.path());
    let (at, action, entity, entity_id, old, new, reason, hash): (String, String, String, i64, String, String, String, String) = conn
        .query_row("SELECT at, action, entity, entity_id, old_json, new_json, reason, chain_hash FROM audit_log", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
        })
        .unwrap();
    assert_eq!(
        at, "2026-10-10T07:30:12.000Z",
        "UTC time from the injected clock"
    );
    assert_eq!(
        (action.as_str(), entity.as_str(), entity_id),
        ("create", "account", 5)
    );
    assert_eq!(old, r#"{"credit_limit_paisa":5000000}"#);
    assert_eq!(new, r#"{"credit_limit_paisa":7500000}"#);
    assert_eq!(reason, "حد بڑھائی");
    assert_eq!(hash.len(), 64);
    assert_ne!(hash, GENESIS_HASH);
    verify(&conn).unwrap();
}

#[test]
fn the_hash_is_deterministic_and_depends_on_the_key_and_the_row() {
    let first = tempdir().unwrap();
    let second = tempdir().unwrap();
    drop(write_rows(&first, 3));
    drop(write_rows(&second, 3));
    let hashes = |dir: &TempDir| -> Vec<String> {
        let conn = common::raw(dir.path());
        let mut statement = conn
            .prepare("SELECT chain_hash FROM audit_log ORDER BY id")
            .unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    // Same key, same clock, same rows: same chain.
    assert_eq!(hashes(&first), hashes(&second));
    let all = hashes(&first);
    assert_eq!(all.len(), 3);
    assert!(all[0] != all[1] && all[1] != all[2]);
    // Another key does not verify.
    let conn = common::raw(first.path());
    let other = AuditKey::from_bytes([8; 32]);
    assert!(matches!(
        verify_audit_chain(&conn, &other),
        Err(VerifyError::Broken(ChainBreak { at_id: 1 }))
    ));
}

#[test]
fn an_empty_log_is_good() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    verify(&common::raw(dir.path())).unwrap();
}

#[test]
fn an_edited_row_is_found_at_its_own_id() {
    let dir = tempdir().unwrap();
    drop(write_rows(&dir, 4));
    let conn = common::raw(dir.path());
    verify(&conn).unwrap();
    drop_audit_triggers(&conn);
    conn.execute(
        "UPDATE audit_log SET new_json = '{\"credit_limit_paisa\":99900000}' WHERE id = 3",
        [],
    )
    .unwrap();
    let error = verify(&conn).unwrap_err();
    assert!(
        matches!(error, VerifyError::Broken(ChainBreak { at_id: 3 })),
        "{error:?}"
    );
}

#[test]
fn every_field_is_covered_by_the_hash() {
    for (column, value) in [
        ("at", "'2026-10-10T07:30:13.000Z'"),
        ("user_id", "NULL"),
        ("action", "'delete'"),
        ("entity", "'voucher'"),
        ("entity_id", "99"),
        ("old_json", "NULL"),
        ("reason", "'x'"),
    ] {
        let dir = tempdir().unwrap();
        drop(write_rows(&dir, 2));
        let conn = common::raw(dir.path());
        drop_audit_triggers(&conn);
        // user_id has a foreign key: the first row has NULL there already, so use a value that differs.
        let value = if column == "user_id" { "1" } else { value };
        if column == "user_id" {
            conn.execute("INSERT INTO users (id, username, password_hash, display_name, role) VALUES (1, 'o', 'x', 'O', 'owner')", []).unwrap();
        }
        conn.execute(
            &format!("UPDATE audit_log SET {column} = {value} WHERE id = 1"),
            [],
        )
        .unwrap();
        let error = verify(&conn).unwrap_err();
        assert!(
            matches!(error, VerifyError::Broken(ChainBreak { at_id: 1 })),
            "{column}: {error:?}"
        );
    }
}

#[test]
fn a_removed_row_is_found_at_the_row_after_the_gap() {
    let dir = tempdir().unwrap();
    drop(write_rows(&dir, 4));
    let conn = common::raw(dir.path());
    drop_audit_triggers(&conn);
    conn.execute("DELETE FROM audit_log WHERE id = 2", [])
        .unwrap();
    let error = verify(&conn).unwrap_err();
    assert!(
        matches!(error, VerifyError::Broken(ChainBreak { at_id: 3 })),
        "{error:?}"
    );
}

#[test]
fn a_row_made_without_the_key_does_not_pass() {
    let dir = tempdir().unwrap();
    drop(write_rows(&dir, 2));
    let conn = common::raw(dir.path());
    // A forger who does not have the key can only guess a hash.
    conn.execute(
        "INSERT INTO audit_log (at, action, entity, chain_hash) VALUES ('2026-10-10T08:00:00.000Z', 'edit', 'account', ?1)",
        ["b".repeat(64)],
    )
    .unwrap();
    assert!(matches!(
        verify(&conn),
        Err(VerifyError::Broken(ChainBreak { at_id: 3 }))
    ));
}

#[test]
fn the_audit_row_rolls_back_with_the_action() {
    let dir = tempdir().unwrap();
    let db = common::open(dir.path());
    let result: Result<(), DbError> = db.write(|tx| {
        db.auditor().write(tx, &entry("edit", 1))?;
        Err(DbError::Busy)
    });
    assert!(result.is_err());
    let count: i64 = common::raw(dir.path())
        .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        count, 0,
        "an audit row without its action would be a lie (Code Standards S7)"
    );
}

#[test]
fn the_key_never_shows_in_debug_output() {
    let text = format!("{:?}", common::test_key());
    assert_eq!(text, "AuditKey([hidden])");
    assert!(!text.contains('7'));
}

#[test]
fn a_random_key_is_32_bytes_and_different_each_time() {
    let a = AuditKey::random().unwrap();
    let b = AuditKey::random().unwrap();
    assert_ne!(a, b);
}
