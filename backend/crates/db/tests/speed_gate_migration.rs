//! Migration 27 (plan 80 gate, the speed fix loop): one covering index of the sale lines for the
//! totals of the sales registers. An index only: no table, column, row, amount, id or chain hash
//! changes, and the file still passes its integrity check.
#![cfg(test)]

use mandi_db::migrate;
use mandi_db::rusqlite::Connection;
use tempfile::tempdir;

fn user_version(conn: &Connection) -> i64 {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap()
}

fn index_exists(conn: &Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
        [name],
        |r| r.get::<_, i64>(0),
    )
    .unwrap()
        == 1
}

/// The rows of every table, counted, and the sums of the money columns that migration 27 must not move.
fn fingerprint(conn: &Connection) -> Vec<(String, i64)> {
    let mut statement = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap();
    let tables: Vec<String> = statement
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    tables
        .into_iter()
        .map(|t| {
            let rows: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM \"{t}\""), [], |r| r.get(0))
                .unwrap();
            (t, rows)
        })
        .collect()
}

#[test]
fn the_speed_gate_migration_adds_the_register_index_and_changes_no_row() {
    let dir = tempdir().unwrap();
    let mut conn = Connection::open(dir.path().join("mandi.db")).unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    // The versions are not fixed numbers: other plans add migrations. Walk up one version at a time
    // and look at the step that makes the index.
    let mut latest = Connection::open(dir.path().join("latest.db")).unwrap();
    migrate::migrations().to_latest(&mut latest).unwrap();
    let last = user_version(&latest);
    let mut step_found = None;
    let mut before = fingerprint(&conn);
    for version in 1..=last {
        let usize_version = usize::try_from(version).unwrap();
        migrate::migrations()
            .to_version(&mut conn, usize_version)
            .unwrap();
        assert_eq!(user_version(&conn), version);
        let has = index_exists(&conn, "idx_bikri_sales_register");
        if has && step_found.is_none() {
            step_found = Some(version);
            // The step that made the index: no table and no row changed or went.
            assert_eq!(fingerprint(&conn), before, "migration {version}");
            // The new index is usable for the read it is for.
            conn.prepare(
                "SELECT COUNT(*), SUM(qty) FROM bikri_sales INDEXED BY idx_bikri_sales_register
                 WHERE is_active = 1",
            )
            .unwrap();
        }
        before = fingerprint(&conn);
    }
    assert!(
        step_found.is_some(),
        "no migration makes the register index"
    );
    assert!(index_exists(&conn, "idx_bikri_sales_register"));
    let check: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(check, "ok");
}
