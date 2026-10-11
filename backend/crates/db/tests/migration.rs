//! Migrations: empty file, re-run, the copy before an upgrade, and a failed upgrade that is
//! put back (plan 03 Section 6, items 1 and 2, NFR-REL-13, NFR-SCL-8).
#![cfg(test)]

mod common;

use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use mandi_core::error::ErrorCode;
use mandi_db::clock::FixedClock;
use mandi_db::migrate::{self, INIT_SQL, MigrationOutcome};
use mandi_db::rusqlite::Connection;
use mandi_db::{DbError, seed};
use rusqlite_migration::{HookError, M, Migrations};
use tempfile::tempdir;

fn clock() -> FixedClock {
    FixedClock(Utc.with_ymd_and_hms(2026, 10, 10, 7, 30, 12).unwrap())
}

fn version_1() -> Migrations<'static> {
    Migrations::new(vec![M::up_with_hook(INIT_SQL, |tx| {
        seed::system_accounts(tx)
            .map(|_| ())
            .map_err(HookError::from)
    })])
}

fn version_2(second: &'static str) -> Migrations<'static> {
    Migrations::new(vec![
        M::up_with_hook(INIT_SQL, |tx| {
            seed::system_accounts(tx)
                .map(|_| ())
                .map_err(HookError::from)
        }),
        M::up(second),
    ])
}

fn connection(path: &std::path::Path) -> Connection {
    let conn = Connection::open(path).unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    conn.pragma_update(None, "recursive_triggers", "ON")
        .unwrap();
    conn
}

fn user_version(conn: &Connection) -> i64 {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap()
}

fn table_exists(conn: &Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [name],
        |r| r.get::<_, i64>(0),
    )
    .unwrap()
        == 1
}

#[test]
fn an_empty_file_gets_the_whole_design_and_the_seed_without_a_copy() {
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    let outcome = migrate::migrate(&mut conn, &backups, &clock()).unwrap();
    assert_eq!(
        outcome,
        MigrationOutcome {
            from: 0,
            to: 1,
            copy: None
        }
    );
    assert_eq!(user_version(&conn), 1);
    assert!(!backups.exists(), "nothing to copy from an empty file");
    let counts = seed::counts(&conn).unwrap();
    assert_eq!(
        (counts.heads, counts.controls, counts.accounts),
        (6, 11, 16)
    );
}

#[test]
fn running_again_is_a_no_op_and_takes_no_copy() {
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate(&mut conn, &backups, &clock()).unwrap();
    let changes = conn.total_changes();
    let again = migrate::migrate(&mut conn, &backups, &clock()).unwrap();
    assert_eq!(
        again,
        MigrationOutcome {
            from: 1,
            to: 1,
            copy: None
        }
    );
    assert_eq!(conn.total_changes(), changes);
    assert!(!backups.exists());
    // The same through the Db handle: open, close, open again.
    let other = tempdir().unwrap();
    drop(common::open(other.path()));
    let db = common::open(other.path());
    assert_eq!(
        db.migration(),
        &MigrationOutcome {
            from: 1,
            to: 1,
            copy: None
        }
    );
}

#[test]
fn an_existing_file_is_copied_and_checked_before_the_upgrade() {
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate_with(&version_1(), &mut conn, &backups, &clock()).unwrap();
    // The client's data: Akram's 12,880 owed, as an opening debit.
    conn.execute(
        "UPDATE accounts SET opening_dr_paisa = 1_288_000 WHERE code = '01010001'",
        [],
    )
    .unwrap();

    let outcome = migrate::migrate_with(
        &version_2("CREATE TABLE extra (x INTEGER) STRICT;"),
        &mut conn,
        &backups,
        &clock(),
    )
    .unwrap();
    assert_eq!(outcome.from, 1);
    assert_eq!(outcome.to, 2);
    let copy: PathBuf = outcome.copy.expect("a copy was taken");
    assert_eq!(copy, backups.join("pre-migration-1-2-20261010T073012Z.db"));
    assert_eq!(user_version(&conn), 2);
    assert!(table_exists(&conn, "extra"));

    // The copy is the data as it was: version 1, no new table, the same numbers.
    let copy_conn = Connection::open(&copy).unwrap();
    assert_eq!(user_version(&copy_conn), 1);
    assert!(!table_exists(&copy_conn, "extra"));
    let opening: i64 = copy_conn
        .query_row(
            "SELECT opening_dr_paisa FROM accounts WHERE code = '01010001'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(opening, 1_288_000);
    let integrity: String = copy_conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    assert_eq!(seed::counts(&copy_conn).unwrap().accounts, 16);
}

#[test]
fn a_failed_upgrade_leaves_the_data_as_it_was_and_returns_err_migration_failed() {
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate_with(&version_1(), &mut conn, &backups, &clock()).unwrap();
    conn.execute(
        "UPDATE accounts SET opening_dr_paisa = 6_500_000 WHERE code = '01010001'",
        [],
    )
    .unwrap();
    let before = seed::counts(&conn).unwrap();

    // The second statement fails after the first one created a table.
    let bad = version_2(
        "CREATE TABLE half_done (x INTEGER) STRICT; INSERT INTO no_such_table VALUES (1);",
    );
    let error = migrate::migrate_with(&bad, &mut conn, &backups, &clock()).unwrap_err();
    assert!(matches!(error, DbError::MigrationFailed(_)), "{error:?}");
    assert_eq!(error.code(), ErrorCode::ErrMigrationFailed);

    assert_eq!(user_version(&conn), 1);
    assert!(!table_exists(&conn, "half_done"));
    assert_eq!(seed::counts(&conn).unwrap(), before);
    let opening: i64 = conn
        .query_row(
            "SELECT opening_dr_paisa FROM accounts WHERE code = '01010001'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(opening, 6_500_000);
    // The copy stays on disk for the support team.
    assert!(
        backups
            .join("pre-migration-1-2-20261010T073012Z.db")
            .is_file()
    );
    // And the file works: the good upgrade still runs afterwards.
    migrate::migrate_with(
        &version_2("CREATE TABLE extra (x INTEGER) STRICT;"),
        &mut conn,
        &backups,
        &clock(),
    )
    .unwrap();
    assert_eq!(user_version(&conn), 2);
}

/// The restore step itself: damage the live file after a copy, put the copy back.
#[test]
fn the_copy_can_be_put_back_over_a_damaged_file() {
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate_with(&version_1(), &mut conn, &backups, &clock()).unwrap();
    conn.execute(
        "UPDATE accounts SET opening_dr_paisa = 6_500_000 WHERE code = '01010001'",
        [],
    )
    .unwrap();
    let outcome = migrate::migrate_with(
        &version_2("CREATE TABLE extra (x INTEGER) STRICT;"),
        &mut conn,
        &backups,
        &clock(),
    )
    .unwrap();
    let copy = outcome.copy.unwrap();

    conn.execute_batch("DROP TABLE extra; UPDATE accounts SET opening_dr_paisa = 0, opening_date = NULL WHERE code = '01010001'; PRAGMA user_version = 1;").unwrap();
    migrate::restore_copy(&copy, &mut conn, 1).unwrap();
    assert_eq!(user_version(&conn), 1);
    let opening: i64 = conn
        .query_row(
            "SELECT opening_dr_paisa FROM accounts WHERE code = '01010001'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(opening, 6_500_000);
    // A copy with the wrong version is refused.
    let wrong = migrate::restore_copy(&copy, &mut conn, 2).unwrap_err();
    assert!(matches!(wrong, DbError::MigrationFailed(_)));
}

#[test]
fn a_file_from_a_newer_app_is_refused_untouched() {
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate(&mut conn, &backups, &clock()).unwrap();
    conn.pragma_update(None, "user_version", 7).unwrap();
    let error = migrate::migrate(&mut conn, &backups, &clock()).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrMigrationFailed);
    assert_eq!(user_version(&conn), 7);
    assert!(!backups.exists());
}

#[test]
fn db_open_on_a_file_at_the_latest_version_takes_no_copy() {
    let dir = tempdir().unwrap();
    let path = common::db_path(dir.path());
    // A file already at the latest version, made directly.
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    {
        let mut conn = connection(&path);
        migrate::migrate_with(&version_1(), &mut conn, &dir.path().join("none"), &clock()).unwrap();
    }
    // Nothing is pending, so no copy is made.
    let db = common::open(dir.path());
    assert_eq!(db.migration().copy, None);
    assert_eq!(db.migration().to, 1);
}
