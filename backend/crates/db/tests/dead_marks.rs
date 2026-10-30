//! Migration 0011, the dead amount table (plan 40, PRD FR-10.10): at most one active move for each
//! account or person key, a move is never deleted, and an undo changes nothing but its own columns.
#![cfg(test)]

mod common;

use mandi_db::rusqlite::{Connection, params};
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Connection) {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    (dir, conn)
}

/// The id of a seeded account to hang a move on.
fn account(conn: &Connection) -> i64 {
    conn.query_row("SELECT id FROM accounts WHERE code = '01010001'", [], |r| {
        r.get(0)
    })
    .unwrap()
}

fn insert(
    conn: &Connection,
    account: i64,
    key: Option<&str>,
    reason: &str,
    note: &str,
) -> Result<usize, mandi_db::rusqlite::Error> {
    conn.execute(
        "INSERT INTO dead_marks (account_id, pukhta_person_key, entry_date, reason, note, owed_paisa, after_entry_id)
         VALUES (?1, ?2, '2026-10-11', ?3, ?4, 1868400, 7)",
        params![account, key, reason, note],
    )
}

#[test]
fn only_one_active_move_for_an_account_or_a_person_key() {
    let (_dir, conn) = open();
    let id = account(&conn);
    insert(&conn, id, None, "died", "").unwrap();
    // A second active move of the same plain account is refused (NULL keys are not distinct).
    assert!(insert(&conn, id, None, "left", "").is_err());
    // A person key is a different key, and a different person is another one.
    insert(&conn, id, Some("a|1"), "died", "").unwrap();
    assert!(insert(&conn, id, Some("a|1"), "left", "").is_err());
    insert(&conn, id, Some("b|2"), "died", "").unwrap();
    // After an undo the same key can be moved again.
    conn.execute(
        "UPDATE dead_marks SET is_active = 0, undone_at = '2026-10-12T00:00:00Z', undo_reason = 'x',
                undone_after_entry_id = 9, undone_date = '2026-10-12'
         WHERE account_id = ?1 AND pukhta_person_key IS NULL",
        [id],
    )
    .unwrap();
    insert(&conn, id, None, "left", "").unwrap();
}

#[test]
fn a_move_is_checked_and_never_deleted_or_rewritten() {
    let (_dir, conn) = open();
    let id = account(&conn);
    // A reason of other needs a note; a reason outside the three is refused.
    assert!(insert(&conn, id, None, "other", "  ").is_err());
    assert!(insert(&conn, id, None, "gone", "").is_err());
    insert(&conn, id, None, "other", "ملک سے باہر").unwrap();
    // It owes a whole rupee amount above 0.
    for bad in [0_i64, 150] {
        assert!(
            conn.execute(
                "INSERT INTO dead_marks (account_id, entry_date, reason, owed_paisa, after_entry_id)
                 VALUES (?1, '2026-10-11', 'died', ?2, 0)",
                params![id, bad],
            )
            .is_err(),
            "{bad}"
        );
    }
    // Never deleted.
    assert!(conn.execute("DELETE FROM dead_marks", []).is_err());
    // Its amount, its day and its boundary cannot be rewritten.
    for sql in [
        "UPDATE dead_marks SET owed_paisa = 100",
        "UPDATE dead_marks SET entry_date = '2026-10-12'",
        "UPDATE dead_marks SET after_entry_id = 99",
        "UPDATE dead_marks SET reason = 'died'",
    ] {
        assert!(conn.execute(sql, []).is_err(), "{sql}");
    }
    // An undo needs its reason, the entry id and the day together, and then freezes the row.
    assert!(
        conn.execute("UPDATE dead_marks SET is_active = 0", [])
            .is_err()
    );
    conn.execute(
        "UPDATE dead_marks SET is_active = 0, undone_at = '2026-10-12T00:00:00Z', undo_reason = 'x',
                undone_after_entry_id = 9, undone_date = '2026-10-12'",
        [],
    )
    .unwrap();
    assert!(
        conn.execute("UPDATE dead_marks SET undo_reason = 'y'", [])
            .is_err(),
        "an undone move never changes again"
    );
}
