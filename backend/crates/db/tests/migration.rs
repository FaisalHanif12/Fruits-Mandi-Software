//! Migrations: empty file, re-run, the copy before an upgrade, and a failed upgrade that is
//! put back (plan 03 Section 6, items 1 and 2, NFR-REL-13, NFR-SCL-8).
#![cfg(test)]

mod common;

use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use mandi_core::error::ErrorCode;
use mandi_db::clock::FixedClock;
use mandi_db::migrate::{
    self, BALANCE_INDEXES_SQL, BIKRI_RECORDS_ARE_NEVER_DELETED_SQL, INIT_SQL, MigrationOutcome,
    SIMPLE_PARTY_CODES_SQL, STABLE_POSTING_ORDER_SQL, SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL,
    TRIP_AND_GARDEN_CODES_SQL, TRIPS_GARDENS_FRUIT_KHATA_SQL,
};
use mandi_db::rusqlite::Connection;
use mandi_db::{DbError, seed};
use rusqlite_migration::{HookError, M, Migrations};
use tempfile::tempdir;

/// The schema version of the newest migration (`migrate::migrations()`).
const LATEST: usize = 9;

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

/// The schema as it was released at version 2: migrations 0001 and 0002.
fn released_version_2() -> Migrations<'static> {
    version_2(BALANCE_INDEXES_SQL)
}

/// The schema as it was released at version 3: migrations 0001 to 0003.
fn released_version_3() -> Migrations<'static> {
    Migrations::new(vec![
        M::up_with_hook(INIT_SQL, |tx| {
            seed::system_accounts(tx)
                .map(|_| ())
                .map_err(HookError::from)
        }),
        M::up(BALANCE_INDEXES_SQL),
        M::up(SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL),
    ])
}

/// The schema as it was released at version 4: migrations 0001 to 0004.
fn released_version_4() -> Migrations<'static> {
    Migrations::new(vec![
        M::up_with_hook(INIT_SQL, |tx| {
            seed::system_accounts(tx)
                .map(|_| ())
                .map_err(HookError::from)
        }),
        M::up(BALANCE_INDEXES_SQL),
        M::up(SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL),
        M::up(BIKRI_RECORDS_ARE_NEVER_DELETED_SQL),
    ])
}

/// The schema as it was released at version 5: migrations 0001 to 0005.
fn released_version_5() -> Migrations<'static> {
    Migrations::new(vec![
        M::up_with_hook(INIT_SQL, |tx| {
            seed::system_accounts(tx)
                .map(|_| ())
                .map_err(HookError::from)
        }),
        M::up(BALANCE_INDEXES_SQL),
        M::up(SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL),
        M::up(BIKRI_RECORDS_ARE_NEVER_DELETED_SQL),
        M::up(STABLE_POSTING_ORDER_SQL),
    ])
}

/// The schema as it was released at version 6: migrations 0001 to 0006.
fn released_version_6() -> Migrations<'static> {
    Migrations::new(vec![
        M::up_with_hook(INIT_SQL, |tx| {
            seed::system_accounts(tx)
                .map(|_| ())
                .map_err(HookError::from)
        }),
        M::up(BALANCE_INDEXES_SQL),
        M::up(SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL),
        M::up(BIKRI_RECORDS_ARE_NEVER_DELETED_SQL),
        M::up(STABLE_POSTING_ORDER_SQL),
        M::up_with_hook(TRIPS_GARDENS_FRUIT_KHATA_SQL, migrate::trips_hook),
    ])
}

/// The schema as it was released at version 7: migrations 0001 to 0007.
fn released_version_7() -> Migrations<'static> {
    Migrations::new(vec![
        M::up_with_hook(INIT_SQL, |tx| {
            seed::system_accounts(tx)
                .map(|_| ())
                .map_err(HookError::from)
        }),
        M::up(BALANCE_INDEXES_SQL),
        M::up(SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL),
        M::up(BIKRI_RECORDS_ARE_NEVER_DELETED_SQL),
        M::up(STABLE_POSTING_ORDER_SQL),
        M::up_with_hook(TRIPS_GARDENS_FRUIT_KHATA_SQL, migrate::trips_hook),
        M::up_with_hook(SIMPLE_PARTY_CODES_SQL, migrate::codes_hook),
    ])
}

/// The schema as it was released at version 8: migrations 0001 to 0008.
fn released_version_8() -> Migrations<'static> {
    Migrations::new(vec![
        M::up_with_hook(INIT_SQL, |tx| {
            seed::system_accounts(tx)
                .map(|_| ())
                .map_err(HookError::from)
        }),
        M::up(BALANCE_INDEXES_SQL),
        M::up(SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL),
        M::up(BIKRI_RECORDS_ARE_NEVER_DELETED_SQL),
        M::up(STABLE_POSTING_ORDER_SQL),
        M::up_with_hook(TRIPS_GARDENS_FRUIT_KHATA_SQL, migrate::trips_hook),
        M::up_with_hook(SIMPLE_PARTY_CODES_SQL, migrate::codes_hook),
        M::up_with_hook(TRIP_AND_GARDEN_CODES_SQL, migrate::trip_names_hook),
    ])
}

/// The name and SQL of every object of one kind (`table`, `index`, `trigger`), by name.
fn schema(conn: &Connection, kind: &str) -> Vec<String> {
    conn.prepare(
        "SELECT name || ':' || COALESCE(sql, '') FROM sqlite_master WHERE type = ?1 ORDER BY name",
    )
    .unwrap()
    .query_map([kind], |r| r.get(0))
    .unwrap()
    .map(Result::unwrap)
    .collect()
}

/// Every row of every table, as text, in a fixed order.
fn all_rows(conn: &Connection) -> Vec<(String, String)> {
    all_rows_without(conn, &[])
}

/// [`all_rows`] without some columns, as `(table, column)`: the columns a migration added.
fn all_rows_without(conn: &Connection, skip: &[(&str, &str)]) -> Vec<(String, String)> {
    let names: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    names
        .into_iter()
        .map(|name| {
            let columns: Vec<String> = conn
                .prepare(&format!(
                    "SELECT name FROM pragma_table_info('{name}') ORDER BY cid"
                ))
                .unwrap()
                .query_map([], |r| r.get(0))
                .unwrap()
                .map(Result::unwrap)
                .filter(|c: &String| !skip.iter().any(|(t, s)| *t == name && s == c))
                .collect();
            let joined = columns
                .iter()
                .map(|c| format!("quote(\"{c}\")"))
                .collect::<Vec<_>>()
                .join(" || '|' || ");
            let rows: Vec<String> = conn
                .prepare(&format!("SELECT {joined} FROM \"{name}\" ORDER BY rowid"))
                .unwrap()
                .query_map([], |r| r.get(0))
                .unwrap()
                .map(Result::unwrap)
                .collect();
            (name, rows.join("\n"))
        })
        .collect()
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
            to: LATEST,
            copy: None
        }
    );
    assert_eq!(user_version(&conn), LATEST as i64);
    assert!(!backups.exists(), "nothing to copy from an empty file");
    let counts = seed::counts(&conn).unwrap();
    // The seed of migration 1 (6 heads, 11 controls, 16 accounts) and that of migration 6 (3
    // controls, 1 account: the seller of trip goods).
    assert_eq!(
        (counts.heads, counts.controls, counts.accounts),
        (6, 14, 17)
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
            from: LATEST,
            to: LATEST,
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
            from: LATEST,
            to: LATEST,
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
    conn.pragma_update(None, "user_version", 10).unwrap();
    let error = migrate::migrate(&mut conn, &backups, &clock()).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrMigrationFailed);
    assert_eq!(user_version(&conn), 10);
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
        migrate::migrate(&mut conn, &dir.path().join("none"), &clock()).unwrap();
    }
    // Nothing is pending, so no copy is made.
    let db = common::open(dir.path());
    assert_eq!(db.migration().copy, None);
    assert_eq!(db.migration().to, LATEST);
}

#[test]
fn migration_2_adds_only_indexes_and_keeps_every_number() {
    // Plan: balances read the index only (NFR-SPD-13, NFR-SPD-21). A file made at version 1 with
    // posted entries is upgraded: a copy is taken, three indexes appear, no table or column
    // changes, and every balance is what it was.
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate_with(&version_1(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!(user_version(&conn), 1);
    let cash: i64 = conn
        .query_row("SELECT id FROM accounts WHERE code = '01010001'", [], |r| {
            r.get(0)
        })
        .unwrap();
    let income: i64 = conn
        .query_row("SELECT id FROM accounts WHERE code = '04010001'", [], |r| {
            r.get(0)
        })
        .unwrap();
    for (day, rupees) in [("2026-10-09", 12_880_i64), ("2026-10-10", 10_000)] {
        conn.execute(
            "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, total_paisa)
             VALUES (?1, 'general', 'voucher', 1, ?2)",
            (day, rupees * 100),
        )
        .unwrap();
        let entry = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, 1, ?2, ?3, 0)",
            (entry, cash, rupees * 100),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, 2, ?2, 0, ?3)",
            (entry, income, rupees * 100),
        )
        .unwrap();
        conn.execute(
            "UPDATE journal_entries SET status = 'posted' WHERE id = ?1",
            [entry],
        )
        .unwrap();
    }
    let tables = |conn: &Connection| -> Vec<String> {
        conn.prepare(
            "SELECT name || ':' || sql FROM sqlite_master WHERE type = 'table' ORDER BY name",
        )
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
    };
    let index_names = |conn: &Connection| -> Vec<String> {
        conn.prepare("SELECT name FROM sqlite_master WHERE type = 'index' AND tbl_name IN ('journal_lines', 'journal_entries') ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let before_tables = tables(&conn);
    let before_indexes = index_names(&conn);
    let before = mandi_db::ledger::account_totals(&conn, None).unwrap();
    assert_eq!(before.len(), 2);

    let outcome =
        migrate::migrate_with(&released_version_2(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!((outcome.from, outcome.to), (1, 2));
    assert!(outcome.copy.is_some(), "a copy is taken before an upgrade");
    assert_eq!(user_version(&conn), 2);
    // Only indexes were added.
    assert_eq!(tables(&conn), before_tables);
    let added: Vec<String> = index_names(&conn)
        .into_iter()
        .filter(|n| !before_indexes.contains(n))
        .collect();
    assert_eq!(
        added,
        [
            "idx_journal_entries_unposted",
            "idx_journal_lines_account_amounts",
            "idx_journal_lines_advance"
        ]
    );
    // The numbers are the same.
    assert_eq!(
        mandi_db::ledger::account_totals(&conn, None).unwrap(),
        before
    );
    let totals = mandi_db::ledger::totals_of(&conn, cash, None, None).unwrap();
    assert_eq!((totals.dr_paisa, totals.cr_paisa), (2_288_000, 0));
}

#[test]
fn migration_3_adds_only_a_trigger_and_keeps_every_row_and_balance() {
    // A file made at version 2 with posted entries, a system account and a user account is
    // upgraded: a copy is taken, one trigger appears, no table, column or index changes, no row
    // changes, and from then on a raw update cannot make a system account inactive.
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate_with(&released_version_2(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!(user_version(&conn), 2);
    let id_of = |conn: &Connection, code: &str| -> i64 {
        conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
            r.get(0)
        })
        .unwrap()
    };
    let cash = id_of(&conn, "01010001");
    let income = id_of(&conn, "04010001");
    for (day, rupees) in [("2026-10-09", 12_880_i64), ("2026-10-10", 10_000)] {
        conn.execute(
            "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, total_paisa)
             VALUES (?1, 'general', 'voucher', 1, ?2)",
            (day, rupees * 100),
        )
        .unwrap();
        let entry = conn.last_insert_rowid();
        for (line_no, account, dr, cr) in [(1, cash, rupees * 100, 0), (2, income, 0, rupees * 100)]
        {
            conn.execute(
                "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, ?2, ?3, ?4, ?5)",
                (entry, line_no, account, dr, cr),
            )
            .unwrap();
        }
        conn.execute(
            "UPDATE journal_entries SET status = 'posted' WHERE id = ?1",
            [entry],
        )
        .unwrap();
    }
    // An ordinary account, to make inactive and active again after the upgrade.
    conn.execute(
        "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur, search_en, search_phonetic)
         SELECT id, '0020', 1, 'buyer', 'اکرم', 'Akram', 'اکرم', 'akram', 'krm' FROM account_controls WHERE code = '0304'",
        [],
    )
    .unwrap();
    // The raw update that the database did not stop at version 2.
    conn.execute(
        "UPDATE accounts SET is_active = 0 WHERE code = '01010001'",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE accounts SET is_active = 1 WHERE code = '01010001'",
        [],
    )
    .unwrap();

    let before_tables = schema(&conn, "table");
    let before_indexes = schema(&conn, "index");
    let before_triggers = schema(&conn, "trigger");
    let before_rows = all_rows(&conn);
    let before = mandi_db::ledger::account_totals(&conn, None).unwrap();
    assert_eq!(before.len(), 2);

    let outcome =
        migrate::migrate_with(&released_version_3(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!((outcome.from, outcome.to), (2, 3));
    assert!(outcome.copy.is_some(), "a copy is taken before an upgrade");
    assert_eq!(user_version(&conn), 3);
    // Only one trigger was added.
    assert_eq!(schema(&conn, "table"), before_tables);
    assert_eq!(schema(&conn, "index"), before_indexes);
    let after_triggers = schema(&conn, "trigger");
    let added: Vec<&String> = after_triggers
        .iter()
        .filter(|t| !before_triggers.contains(t))
        .collect();
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(
        added[0].starts_with("accounts_system_stays_active:"),
        "{added:?}"
    );
    assert_eq!(after_triggers.len(), before_triggers.len() + 1);
    // Every row and every balance is what it was.
    assert_eq!(all_rows(&conn), before_rows);
    assert_eq!(
        mandi_db::ledger::account_totals(&conn, None).unwrap(),
        before
    );
    let totals = mandi_db::ledger::totals_of(&conn, cash, None, None).unwrap();
    assert_eq!((totals.dr_paisa, totals.cr_paisa), (2_288_000, 0));

    // The migration also works on the copy taken before it: version 2, still the old behaviour.
    let copy_conn = Connection::open(outcome.copy.unwrap()).unwrap();
    assert_eq!(user_version(&copy_conn), 2);
    copy_conn
        .execute(
            "UPDATE accounts SET is_active = 0 WHERE code = '01010001'",
            [],
        )
        .unwrap();

    // Now: no system account can be made inactive; the failure is classified, not a bare error.
    let system: Vec<String> = conn
        .prepare("SELECT code FROM accounts WHERE is_system = 1 ORDER BY code")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert!(system.len() >= 15, "{} system accounts", system.len());
    for code in &system {
        let error = conn
            .execute("UPDATE accounts SET is_active = 0 WHERE code = ?1", [code])
            .unwrap_err();
        assert!(
            matches!(common::classify(error), DbError::SystemAccount),
            "{code}"
        );
    }
    let all_at_once = conn
        .execute("UPDATE accounts SET is_active = 0 WHERE is_system = 1", [])
        .unwrap_err();
    assert!(matches!(
        common::classify(all_at_once),
        DbError::SystemAccount
    ));
    let still_active: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM accounts WHERE is_system = 1 AND is_active = 0",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(still_active, 0, "a refused update changed a row");
    // Everything else stays allowed: the same value, other columns, a user account made inactive
    // and active again.
    conn.execute("UPDATE accounts SET is_active = 1 WHERE is_system = 1", [])
        .unwrap();
    conn.execute(
        "UPDATE accounts SET note = 'x', is_active = 1 WHERE code = '01010001'",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE accounts SET opening_dr_paisa = 6_500_000, opening_date = '2026-08-31' WHERE code = '01010001'",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE accounts SET note = NULL WHERE code = '01010001'",
        [],
    )
    .unwrap();
    conn.execute("UPDATE accounts SET is_active = 0 WHERE code = '0020'", [])
        .unwrap();
    conn.execute("UPDATE accounts SET is_active = 1 WHERE code = '0020'", [])
        .unwrap();
}

#[test]
fn migration_4_adds_only_triggers_and_keeps_every_row_number_and_balance() {
    // Plan 13: a file made at version 3 with a بکری, its lot and its line is upgraded: a copy is
    // taken, seven triggers appear (no table, column or index changes), no row changes, every
    // بکری keeps its number, and from then on a raw DELETE or a change of a number is refused
    // while the updates of editing and cancelling (plan 16) still work.
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate_with(&released_version_3(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!(user_version(&conn), 3);
    let f = common::fixture(&conn);
    // A second بکری, number 2, cancelled later.
    conn.execute(
        "INSERT INTO bikri_entries (number, entry_date, bill_date, truck_no) VALUES (2, '2026-10-10', '2026-10-10', 'LHR-0077')",
        [],
    )
    .unwrap();
    // What version 3 allowed and the new triggers must not take away: a raw delete of a row that
    // nothing points at.
    conn.execute("DELETE FROM bikri_entries WHERE number = 2", [])
        .unwrap();
    conn.execute(
        "INSERT INTO bikri_entries (number, entry_date, bill_date, truck_no) VALUES (2, '2026-10-10', '2026-10-10', 'LHR-0077')",
        [],
    )
    .unwrap();

    let before_tables = schema(&conn, "table");
    let before_indexes = schema(&conn, "index");
    let before_triggers = schema(&conn, "trigger");
    let before_rows = all_rows(&conn);

    let outcome =
        migrate::migrate_with(&released_version_4(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!((outcome.from, outcome.to), (3, 4));
    assert!(outcome.copy.is_some(), "a copy is taken before an upgrade");
    assert_eq!(user_version(&conn), 4);
    assert_eq!(schema(&conn, "table"), before_tables);
    assert_eq!(schema(&conn, "index"), before_indexes);
    let after_triggers = schema(&conn, "trigger");
    let mut added: Vec<&str> = after_triggers
        .iter()
        .filter(|t| !before_triggers.contains(t))
        .map(|t| t.split(':').next().unwrap_or(""))
        .collect();
    added.sort_unstable();
    assert_eq!(
        added,
        [
            "bikri_entries_no_delete",
            "bikri_entries_no_replace",
            "bikri_entries_number_is_final",
            "bikri_lots_no_delete",
            "bikri_lots_no_replace",
            "bikri_sales_no_delete",
            "bikri_sales_no_replace",
        ]
    );
    // Every row is what it was, and every بکری keeps its number.
    assert_eq!(all_rows(&conn), before_rows);
    let numbers: Vec<i64> = conn
        .prepare("SELECT number FROM bikri_entries ORDER BY id")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(numbers, [1, 2]);

    // The migration also works on the copy: version 3, the old behaviour (a delete still goes).
    let copy_conn = Connection::open(outcome.copy.unwrap()).unwrap();
    assert_eq!(user_version(&copy_conn), 3);
    copy_conn
        .execute("DELETE FROM bikri_entries WHERE number = 2", [])
        .unwrap();

    // Now nothing of a بکری can be deleted, and the number is final.
    let refused = |sql: &str| -> DbError { common::classify(conn.execute(sql, []).unwrap_err()) };
    for sql in [
        "DELETE FROM bikri_sales".to_owned(),
        "DELETE FROM bikri_lots".to_owned(),
        "DELETE FROM bikri_entries".to_owned(),
        "DELETE FROM bikri_entries WHERE number = 2".to_owned(),
        "UPDATE bikri_entries SET number = 9 WHERE number = 2".to_owned(),
        "UPDATE bikri_entries SET number = number + 10".to_owned(),
        // INSERT OR REPLACE would delete the old row first.
        "INSERT OR REPLACE INTO bikri_entries (number, entry_date, bill_date) VALUES (1, '2026-10-10', '2026-10-10')".to_owned(),
        format!("INSERT OR REPLACE INTO bikri_entries (id, number, entry_date, bill_date) VALUES ({}, 77, '2026-10-10', '2026-10-10')", f.bikri),
        format!("INSERT OR REPLACE INTO bikri_lots (id, bikri_id, lot_no, seller_account_id, item_id, packing_id, qty_received) VALUES ({}, {}, 5, {}, {}, 1, 5)", f.lot, f.bikri, f.seller, f.item),
        // A number given twice.
        "INSERT INTO bikri_entries (number, entry_date, bill_date) VALUES (2, '2026-10-10', '2026-10-10')".to_owned(),
    ] {
        assert!(
            matches!(refused(&sql), DbError::AppendOnly),
            "not refused as append-only: {sql}"
        );
    }
    assert_eq!(
        all_rows(&conn),
        before_rows,
        "a refused statement changed a row"
    );

    // What editing and cancelling need still works: the status and its reason, the print count,
    // the running totals and the written-off quantity of a lot, and the switch-off of a line.
    conn.execute(
        "UPDATE bikri_entries SET number = number WHERE number = 2",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE bikri_entries SET status = 'cancelled', cancel_reason = 'غلط اندراج',
             cancelled_at = '2026-10-10T09:00:00Z', print_count = print_count + 1, version = version + 1
         WHERE number = 2",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE bikri_entries SET note = 'x', print_count = 2 WHERE number = 1",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE bikri_lots SET qty_sold = 45, qty_written_off = 5 WHERE id = ?1",
        [f.lot],
    )
    .unwrap();
    conn.execute(
        "UPDATE bikri_lots SET gross_paisa = 1_000_000, freight_paisa = 1_100_000, deductions_paisa = 1_100_000, net_paisa = -100_000, version = version + 1 WHERE id = ?1",
        [f.lot],
    )
    .unwrap();
    conn.execute(
        "UPDATE bikri_sales SET is_active = 0 WHERE id = ?1",
        [f.sale],
    )
    .unwrap();
    // A new بکری still takes the next number.
    conn.execute(
        "INSERT INTO bikri_entries (number, entry_date, bill_date) VALUES (3, '2026-10-10', '2026-10-10')",
        [],
    )
    .unwrap();
}

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

/// Posts a general entry of `rupees` between two accounts on a file of version 4, the way the app
/// of version 4 did: raw rows, then ONE update that sets the status, the total and the keyed hash
/// of the chain, and the head record. The bytes that are hashed are copied from the code of
/// version 4, so a hash that still verifies after the upgrade proves the chain format is the same.
///
/// With `existing`, the lines go into that draft entry (its day and total are already set);
/// otherwise a new entry is made.
fn legacy_post(
    conn: &Connection,
    existing: Option<i64>,
    day: &str,
    from: &str,
    to: &str,
    rupees: i64,
) -> i64 {
    let account = |code: &str| -> i64 {
        conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
            r.get(0)
        })
        .unwrap()
    };
    let entry = match existing {
        Some(entry) => entry,
        None => {
            conn.execute(
                "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, status, total_paisa)
                 VALUES (?1, 'general', 'voucher', 1, 'draft', 0)",
                [day],
            )
            .unwrap();
            conn.last_insert_rowid()
        }
    };
    let lines = [(from, rupees * 100, 0_i64), (to, 0, rupees * 100)];
    for (index, (code, dr, cr)) in lines.iter().enumerate() {
        conn.execute(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, ?2, ?3, ?4, ?5)",
            (entry, i64::try_from(index).unwrap() + 1, account(code), dr, cr),
        )
        .unwrap();
    }
    let (previous, last): (String, i64) = conn
        .query_row(
            "SELECT COALESCE((SELECT chain_hash FROM journal_entries WHERE status = 'posted' ORDER BY id DESC LIMIT 1), ?1),
                    COALESCE((SELECT id FROM journal_entries WHERE status = 'posted' ORDER BY id DESC LIMIT 1), 0)",
            [mandi_db::audit::GENESIS_HASH],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(last < entry);
    let mut bytes = Vec::new();
    put(&mut bytes, Some(&previous));
    put(&mut bytes, Some(&entry.to_string()));
    put(&mut bytes, Some(day));
    put(&mut bytes, Some("general"));
    put(&mut bytes, Some("voucher"));
    put(&mut bytes, Some("1"));
    put(&mut bytes, None);
    put(&mut bytes, None);
    put(&mut bytes, Some(&(rupees * 100).to_string()));
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
    // The test key of `common::test_key()` is 32 bytes of 7.
    let hash = blake3::keyed_hash(&[7_u8; 32], &bytes).to_hex().to_string();
    conn.execute(
        "UPDATE journal_entries SET status = 'posted', total_paisa = ?1, chain_hash = ?2 WHERE id = ?3",
        (rupees * 100, &hash, entry),
    )
    .unwrap();
    conn.execute(
        "INSERT INTO settings (key, value_json) VALUES ('journal_head', ?1)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
        [format!("{{\"id\":{entry},\"hash\":\"{hash}\"}}")],
    )
    .unwrap();
    entry
}

/// Posts a general entry on a file of version 5 through the one function the app uses, with an
/// order key if one is given.
fn post_chained(
    conn: &Connection,
    day: &str,
    from: &str,
    to: &str,
    rupees: i64,
    key: Option<i64>,
) -> i64 {
    let account = |code: &str| -> i64 {
        conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
            r.get(0)
        })
        .unwrap()
    };
    let header = mandi_db::ledger::EntryHeader {
        entry_date: day,
        posting_type: "general",
        source_type: "voucher",
        source_id: 1,
        reverses_entry_id: None,
        narration: None,
    };
    let entry =
        mandi_db::ledger::insert_draft_ordered(conn, &header, key, 1, "2026-10-10T07:30:12.000Z")
            .unwrap();
    for (line_no, (code, dr, cr)) in [(from, rupees * 100, 0), (to, 0, rupees * 100)]
        .into_iter()
        .enumerate()
    {
        mandi_db::ledger::insert_line(
            conn,
            entry,
            &mandi_db::ledger::NewLine {
                line_no: i64::try_from(line_no).unwrap() + 1,
                account_id: account(code),
                person_key: None,
                dr_paisa: dr,
                cr_paisa: cr,
                is_advance: false,
                narration: None,
            },
            1,
            "2026-10-10T07:30:12.000Z",
        )
        .unwrap();
    }
    mandi_db::ledger::post(
        conn,
        &common::test_key(),
        entry,
        rupees * 100,
        1,
        "2026-10-10T07:30:12.000Z",
    )
    .unwrap();
    entry
}

#[test]
fn migration_5_adds_three_columns_and_keeps_every_row_number_and_hash() {
    // Plan 16: a populated file made at version 4 (posted entries with their hash chain, a بکری
    // with a lot, a sale line and stock moves) is upgraded: a copy is taken, `order_key`,
    // `reverses_move_id` and `ended` appear as empty columns, one index and one trigger are added, no row
    // changes, every balance and every chain hash is what it was, and the chain still verifies.
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate_with(&released_version_4(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!(user_version(&conn), 4);
    let f = common::fixture(&conn);
    let cash = common::account_id(&conn, "01010001");
    // The fixture's draft entry is the posting of its sale line: give it its body and post it first.
    let first = legacy_post(
        &conn,
        Some(f.entry),
        "2026-10-10",
        "01010001",
        "04010001",
        12_880,
    );
    let second = legacy_post(&conn, None, "2026-10-10", "01010001", "04010002", 10_000);
    let third = legacy_post(&conn, None, "2026-10-10", "01010001", "04010003", 160);
    for (kind, qty, entry) in [("received", 50_i64, first), ("sold", -20, second)] {
        conn.execute(
            "INSERT INTO stock_moves (lot_id, journal_entry_id, move_date, kind, qty) VALUES (?1, ?2, '2026-10-10', ?3, ?4)",
            (f.lot, entry, kind, qty),
        )
        .unwrap();
    }
    let hashes = |conn: &Connection| -> Vec<(i64, String)> {
        conn.prepare(
            "SELECT id, chain_hash FROM journal_entries WHERE status = 'posted' ORDER BY id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
    };
    let before_hashes = hashes(&conn);
    assert_eq!(before_hashes.len(), 3);
    let before_tables = schema(&conn, "table");
    let before_indexes = schema(&conn, "index");
    let before_triggers = schema(&conn, "trigger");
    let before_rows = all_rows(&conn);
    let before_totals = mandi_db::ledger::account_totals(&conn, None).unwrap();

    let outcome =
        migrate::migrate_with(&released_version_5(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!((outcome.from, outcome.to), (4, 5));
    assert!(outcome.copy.is_some(), "a copy is taken before an upgrade");
    assert_eq!(user_version(&conn), 5);

    // Only these two tables changed, and only by one column each.
    let after_tables = schema(&conn, "table");
    let changed: Vec<&str> = after_tables
        .iter()
        .filter(|t| !before_tables.contains(t))
        .map(|t| t.split(':').next().unwrap_or(""))
        .collect();
    assert_eq!(changed, ["bikri_sales", "journal_entries", "stock_moves"]);
    let columns = |table: &str| -> Vec<String> {
        conn.prepare(&format!(
            "SELECT name FROM pragma_table_info('{table}') ORDER BY cid"
        ))
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
    };
    assert_eq!(
        columns("journal_entries").last().map(String::as_str),
        Some("order_key")
    );
    assert_eq!(
        columns("stock_moves").last().map(String::as_str),
        Some("reverses_move_id")
    );
    assert_eq!(
        columns("bikri_sales").last().map(String::as_str),
        Some("ended")
    );
    let added_indexes: Vec<String> = schema(&conn, "index")
        .into_iter()
        .filter(|i| !before_indexes.contains(i))
        .map(|i| i.split(':').next().unwrap_or("").to_owned())
        .collect();
    assert_eq!(added_indexes, ["idx_stock_moves_reverses"]);
    let added_triggers: Vec<String> = schema(&conn, "trigger")
        .into_iter()
        .filter(|t| !before_triggers.contains(t))
        .map(|t| t.split(':').next().unwrap_or("").to_owned())
        .collect();
    assert_eq!(added_triggers, ["stock_moves_reversal_mirrors"]);

    // Every row is what it was; the new columns are empty; every hash and balance is the same.
    assert_eq!(
        all_rows_without(
            &conn,
            &[
                ("journal_entries", "order_key"),
                ("stock_moves", "reverses_move_id"),
                ("bikri_sales", "ended")
            ]
        ),
        before_rows
    );
    let set: i64 = conn
        .query_row(
            "SELECT (SELECT COUNT(*) FROM journal_entries WHERE order_key IS NOT NULL)
                  + (SELECT COUNT(*) FROM stock_moves WHERE reverses_move_id IS NOT NULL)
                  + (SELECT COUNT(*) FROM bikri_sales WHERE ended IS NOT NULL)",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(set, 0);
    assert_eq!(hashes(&conn), before_hashes);
    mandi_db::ledger::verify_journal_chain(&conn, &common::test_key()).unwrap();
    assert_eq!(
        mandi_db::ledger::account_totals(&conn, None).unwrap(),
        before_totals
    );
    let totals = mandi_db::ledger::totals_of(&conn, cash, None, None).unwrap();
    assert_eq!((totals.dr_paisa, totals.cr_paisa), (2_304_000, 0));

    // The copy is the version-4 file as it was.
    let copy_conn = Connection::open(outcome.copy.unwrap()).unwrap();
    assert_eq!(user_version(&copy_conn), 4);
    assert!(
        copy_conn
            .prepare("SELECT order_key FROM journal_entries")
            .is_err(),
        "the copy has no order key"
    );

    // The key is placed when the entry is a draft, is covered by the hash, and is final once
    // posted. An entry without a key hashes as before.
    let replaced = post_chained(
        &conn,
        "2026-10-10",
        "01010001",
        "04010004",
        20,
        Some(second),
    );
    mandi_db::ledger::verify_journal_chain(&conn, &common::test_key()).unwrap();
    assert_eq!(
        mandi_db::ledger::order_key_of(&conn, replaced).unwrap(),
        Some(second)
    );
    assert_eq!(
        mandi_db::ledger::order_key_of(&conn, third).unwrap(),
        Some(third)
    );
    let refused = |sql: &str| -> DbError { common::classify(conn.execute(sql, []).unwrap_err()) };
    assert!(matches!(
        refused(&format!(
            "UPDATE journal_entries SET order_key = 1 WHERE id = {replaced}"
        )),
        DbError::JournalPosted
    ));
    assert!(
        conn.execute(
            "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, order_key)
             VALUES ('2026-10-10', 'general', 'voucher', 1, 0)",
            [],
        )
        .is_err(),
        "a key must be more than 0"
    );
    // A changed key breaks the chain (someone edited the file outside the app).
    conn.execute_batch("DROP TRIGGER journal_entries_posted_is_final;")
        .unwrap();
    conn.execute(
        "UPDATE journal_entries SET order_key = ?1 WHERE id = ?2",
        (first, replaced),
    )
    .unwrap();
    assert!(mandi_db::ledger::verify_journal_chain(&conn, &common::test_key()).is_err());

    // A stock move reversal names the move it reverses, once, as its exact opposite.
    let sold: i64 = conn
        .query_row("SELECT id FROM stock_moves WHERE kind = 'sold'", [], |r| {
            r.get(0)
        })
        .unwrap();
    let reversal = |qty: i64, kind: &str, of: i64| {
        conn.execute(
            "INSERT INTO stock_moves (lot_id, journal_entry_id, move_date, kind, qty, reverses_move_id)
             VALUES (?1, NULL, '2026-10-10', ?2, ?3, ?4)",
            (f.lot, kind, qty, of),
        )
    };
    assert!(
        reversal(19, "reversal", sold).is_err(),
        "not the opposite quantity"
    );
    assert!(reversal(20, "sold", sold).is_err(), "not a reversal");
    reversal(20, "reversal", sold).unwrap();
    assert!(
        reversal(20, "reversal", sold).is_err(),
        "a move is reversed once"
    );
    let reversed_reversal: i64 = conn.last_insert_rowid();
    assert!(
        reversal(-20, "reversal", reversed_reversal).is_err(),
        "a reversal is not reversed again"
    );
}

/// The rows of the two account tables of a file, without the columns migration 6 added or changed
/// and without the cash account, whose English name migration 6 changes (PRD FR-10.4 v1.6).
fn old_account_rows(conn: &Connection) -> Vec<String> {
    conn.prepare(
        "SELECT quote(id) || '|' || quote(control_id) || '|' || quote(code) || '|' || quote(serial) || '|' ||
                quote(party_type) || '|' || quote(name_ur) || '|' || quote(search_ur) || '|' ||
                quote(opening_dr_paisa) || '|' || quote(opening_cr_paisa) || '|' || quote(is_system)
         FROM accounts WHERE code <> '01010001' ORDER BY id",
    )
    .unwrap()
    .query_map([], |r| r.get(0))
    .unwrap()
    .map(Result::unwrap)
    .collect()
}

#[test]
fn migration_6_keeps_every_row_number_and_hash_and_widens_the_checks() {
    // Plan 16b (PRD v1.7): a populated file made at version 5 (posted entries with their hash chain,
    // a بکری with a lot, a sale line, stock moves and two partners) is upgraded: a copy is taken, the
    // tables of trips, gardens and fruit appear, the new columns are added, the system accounts and
    // controls of v1.6 are seeded, each partner gets his 0604 account, and the CHECK lists of six
    // tables take the new words. No existing row changes, every balance and every chain hash is what
    // it was, and the chain still verifies on this very connection (D4-A32).
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate_with(&released_version_5(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!(user_version(&conn), 5);
    let f = common::fixture(&conn);
    let cash = common::account_id(&conn, "01010001");
    let first = legacy_post(
        &conn,
        Some(f.entry),
        "2026-10-10",
        "01010001",
        "04010001",
        12_880,
    );
    let second = legacy_post(&conn, None, "2026-10-10", "01010001", "04010002", 10_000);
    for (kind, qty, entry) in [("received", 50_i64, first), ("sold", -20, second)] {
        conn.execute(
            "INSERT INTO stock_moves (lot_id, journal_entry_id, move_date, kind, qty) VALUES (?1, ?2, '2026-10-10', ?3, ?4)",
            (f.lot, entry, kind, qty),
        )
        .unwrap();
    }
    // Two partners, each with a capital and a drawings account, as plan 11 made them.
    for (n, (ur, en)) in [("الف", "Alif"), ("ب", "Bay")].into_iter().enumerate() {
        let n = i64::try_from(n).unwrap() + 1;
        let mut ids = Vec::new();
        for control in ["0602", "0603"] {
            conn.execute(
                "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur,
                     search_en, search_phonetic)
                 SELECT id, ?2, ?3, 'other', ?4, ?5, ?4, lower(?5), lower(?5)
                 FROM account_controls WHERE code = ?1",
                (control, format!("{control}{n:04}"), n, ur, en),
            )
            .unwrap();
            ids.push(conn.last_insert_rowid());
        }
        conn.execute(
            "INSERT INTO partners (name_ur, name_en, capital_account_id, drawings_account_id)
             VALUES (?1, ?2, ?3, ?4)",
            (ur, en, ids[0], ids[1]),
        )
        .unwrap();
    }
    let hashes = |conn: &Connection| -> Vec<(i64, String)> {
        conn.prepare(
            "SELECT id, chain_hash FROM journal_entries WHERE status = 'posted' ORDER BY id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
    };
    let before_hashes = hashes(&conn);
    assert_eq!(before_hashes.len(), 2);
    let before_tables = schema(&conn, "table");
    let before_indexes = schema(&conn, "index");
    let before_triggers = schema(&conn, "trigger");
    let before_rows = all_rows(&conn);
    let before_accounts = old_account_rows(&conn);
    let before_totals = mandi_db::ledger::account_totals(&conn, None).unwrap();
    let accounts_before = seed::counts(&conn).unwrap().accounts;

    let outcome =
        migrate::migrate_with(&released_version_6(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!((outcome.from, outcome.to), (5, 6));
    assert!(outcome.copy.is_some(), "a copy is taken before an upgrade");
    assert_eq!(user_version(&conn), 6);

    // The tables whose text changed or that are new, by name.
    let name_of = |text: &String| text.split(':').next().unwrap_or("").to_owned();
    let changed: Vec<String> = schema(&conn, "table")
        .iter()
        .filter(|t| !before_tables.contains(t))
        .map(name_of)
        .collect();
    assert_eq!(
        changed,
        [
            "account_controls",
            "accounts",
            "analytics_facts",
            "bikri_lots",
            "bikri_sales",
            "fruit_takes",
            "journal_entries",
            "lot_credit_parts",
            "partners",
            "purchase_cost_parts",
            "purchases",
            "stock_moves",
            "venture_entries",
            "venture_entry_parts",
            "venture_partners",
            "ventures",
            "vouchers"
        ]
    );
    // No index is lost, and the new ones are there.
    let index_names: Vec<String> = schema(&conn, "index").iter().map(name_of).collect();
    for old in before_indexes.iter().map(name_of) {
        assert!(index_names.contains(&old), "index {old} was lost");
    }
    for new in [
        "idx_analytics_facts_purchase",
        "idx_analytics_facts_venture",
        "idx_bikri_lots_purchase",
        "idx_fruit_takes_date",
        "idx_lot_credit_parts_lot",
        "idx_partners_venture_account",
        "idx_purchase_cost_parts_purchase",
        "idx_purchases_venture",
        "idx_purchases_date",
        "idx_stock_moves_purchase",
        "idx_venture_entries_venture",
        "idx_venture_entry_parts_partner",
        "idx_venture_partners_partner",
        "idx_ventures_kind_status",
        "idx_ventures_search_en",
    ] {
        assert!(
            index_names.contains(&new.to_owned()),
            "index {new} is missing"
        );
    }
    // No trigger is lost; two were made again, and the new ones are there.
    let trigger_names: Vec<String> = schema(&conn, "trigger").iter().map(name_of).collect();
    for old in before_triggers.iter().map(name_of) {
        assert!(trigger_names.contains(&old), "trigger {old} was lost");
    }
    let changed_triggers: Vec<String> = schema(&conn, "trigger")
        .iter()
        .filter(|t| !before_triggers.contains(t))
        .map(name_of)
        .collect();
    assert_eq!(
        changed_triggers,
        [
            "bikri_sales_snapshot_is_final",
            "purchase_cost_parts_no_delete",
            "purchase_cost_parts_no_update",
            "purchases_no_delete",
            "purchases_record_is_final",
            "stock_moves_has_an_owner",
            "stock_moves_reversal_mirrors",
            "venture_entries_no_delete",
            "venture_entries_record_is_final",
            "venture_entry_parts_no_delete",
            "venture_entry_parts_no_update",
            "venture_partners_shares_locked_delete",
            "venture_partners_shares_locked_insert",
            "venture_partners_shares_locked_update",
            "ventures_identity_is_final",
            "ventures_no_delete",
            "fruit_takes_no_delete",
            "lot_credit_parts_no_delete",
            "lot_credit_parts_no_update"
        ]
        .map(str::to_owned)
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
    );

    // Every old row is what it was. The new columns are left out of the comparison, the new tables
    // are not in the old file, and the accounts are compared by `old_account_rows`.
    let skip = [
        ("analytics_facts", "purchase_id"),
        ("analytics_facts", "venture_id"),
        ("analytics_facts", "cost_paisa"),
        ("bikri_lots", "purchase_id"),
        ("bikri_lots", "qty_taken"),
        ("bikri_sales", "cost_paisa"),
        ("stock_moves", "purchase_id"),
        ("stock_moves", "cost_paisa"),
        ("partners", "venture_account_id"),
        ("accounts", "tag"),
    ];
    let new_tables = [
        "fruit_takes",
        "lot_credit_parts",
        "purchase_cost_parts",
        "purchases",
        "venture_entries",
        "venture_entry_parts",
        "venture_partners",
        "ventures",
    ];
    let after_rows: Vec<(String, String)> = all_rows_without(&conn, &skip)
        .into_iter()
        .filter(|(table, _)| !new_tables.contains(&table.as_str()))
        .collect();
    assert_eq!(before_rows.len(), after_rows.len());
    for ((table, before), (after_table, after)) in before_rows.iter().zip(&after_rows) {
        assert_eq!(table, after_table);
        if table == "accounts" {
            // The cash account's English name changes and new accounts follow the old ones.
            assert!(after.lines().count() > before.lines().count());
        } else if table == "account_controls" {
            assert!(
                after.starts_with(before.as_str()),
                "{table}: old rows unchanged and new rows after them"
            );
        } else {
            assert_eq!(before, after, "{table}");
        }
    }
    assert_eq!(
        old_account_rows(&conn)[..before_accounts.len()],
        before_accounts[..],
        "every old account is what it was, apart from the English name of the cash account"
    );
    assert_eq!(hashes(&conn), before_hashes);
    mandi_db::ledger::verify_journal_chain(&conn, &common::test_key()).unwrap();
    assert_eq!(
        mandi_db::ledger::account_totals(&conn, None).unwrap(),
        before_totals
    );
    let totals = mandi_db::ledger::totals_of(&conn, cash, None, None).unwrap();
    assert_eq!((totals.dr_paisa, totals.cr_paisa), (2_288_000, 0));
    let bad_keys: i64 = conn
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(bad_keys, 0);
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");

    // The seed: the seller of trip goods, the cash name, the controls, a 0604 account per partner.
    let one = |sql: &str| -> String { conn.query_row(sql, [], |r| r.get::<_, String>(0)).unwrap() };
    let count = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap() };
    assert_eq!(
        one(
            "SELECT party_type || '|' || is_system || '|' || name_en || '|' || search_en FROM accounts WHERE code = '030119998'"
        ),
        "seller|1|Own Goods (Trips)|own goods (trips)"
    );
    assert_eq!(
        one(
            "SELECT name_en || '|' || search_en || '|' || search_phonetic FROM accounts WHERE code = '01010001'"
        ),
        format!(
            "Cash|{}|{}",
            mandi_core::text::search_en("Cash"),
            mandi_core::text::search_phonetic("Cash")
        )
    );
    assert_eq!(
        count("SELECT COUNT(*) FROM accounts WHERE code IN ('01040001', '04010006', '05020001')"),
        0,
        "the accounts of the firm's own goods are not made"
    );
    assert_eq!(
        count("SELECT COUNT(*) FROM account_controls WHERE code IN ('0104', '0502')"),
        0
    );
    for (code, party) in [("0305", "separate"), ("0306", "garden"), ("0604", "other")] {
        assert_eq!(
            one(&format!(
                "SELECT party_type FROM account_controls WHERE code = '{code}'"
            )),
            party
        );
    }
    assert_eq!(
        one(
            "SELECT h.code FROM account_controls c JOIN account_heads h ON h.id = c.head_id WHERE c.code = '0604'"
        ),
        "06"
    );
    let ventures_of_partners: Vec<(String, String)> = conn
        .prepare(
            "SELECT a.code, a.name_en FROM partners p JOIN accounts a ON a.id = p.venture_account_id ORDER BY p.id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(
        ventures_of_partners,
        [
            (
                "06040001".to_owned(),
                "Gardens Trips and Fruit - Alif".to_owned()
            ),
            (
                "06040002".to_owned(),
                "Gardens Trips and Fruit - Bay".to_owned()
            ),
        ]
    );
    assert_eq!(
        seed::counts(&conn).unwrap().accounts,
        accounts_before + 3,
        "the seller of trip goods and one 0604 account for each of the two partners"
    );
    // The tag exists and is empty on every migrated account (D4-A32).
    assert_eq!(
        count("SELECT COUNT(*) FROM pragma_table_info('accounts') WHERE name = 'tag'"),
        1
    );
    assert_eq!(
        count("SELECT COUNT(*) FROM accounts WHERE tag IS NOT NULL"),
        0
    );
    let tag = |value: &str| {
        conn.execute(
            "UPDATE accounts SET tag = ?1 WHERE id = ?2",
            (value, f.buyer),
        )
    };
    tag("K").unwrap();
    tag("D.P").unwrap();
    tag("ABCD").unwrap();
    for bad in ["k", "ABCDE", ".", "1", "A B", "ا"] {
        assert!(tag(bad).is_err(), "tag {bad:?} must be refused");
    }
    conn.execute("UPDATE accounts SET tag = NULL WHERE id = ?1", [f.buyer])
        .unwrap();

    // The widened lists accept the new words on this same connection, and still refuse others.
    let entry_with = |posting: &str, source: &str| {
        conn.execute(
            "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id)
             VALUES ('2026-10-10', ?1, ?2, 1)",
            (posting, source),
        )
    };
    for (posting, source) in [
        ("purchase", "purchase"),
        ("trip_cost", "venture"),
        ("garden_payment", "venture"),
        ("venture_receipt", "venture"),
        ("garden_close", "venture"),
        ("fruit_taken", "fruit"),
        ("capital_in", "voucher"),
        ("capital_out", "voucher"),
        ("khata_out", "voucher"),
        ("khata_in", "voucher"),
    ] {
        entry_with(posting, source).unwrap();
    }
    assert!(
        entry_with("own_write_off", "purchase").is_err(),
        "own_write_off is gone"
    );
    assert!(entry_with("sale", "purchase").is_err());
    assert!(entry_with("purchase", "elsewhere").is_err());
    for kind in ["capital_in", "capital_out", "khata_out", "khata_in"] {
        conn.execute(
            "INSERT INTO vouchers (type, number, entry_date, amount_paisa, partner_id)
             VALUES (?1, 1, '2026-10-10', 100, NULL)",
            [kind],
        )
        .unwrap();
    }
    assert!(
        conn.execute(
            "INSERT INTO vouchers (type, number, entry_date, amount_paisa) VALUES ('nonsense', 1, '2026-10-10', 100)",
            [],
        )
        .is_err()
    );
    let control = |party: &str| {
        conn.execute(
            "INSERT INTO account_controls (head_id, code, name_ur, name_en, party_type)
             VALUES (1, ?1, 'x', 'x', ?2)",
            (format!("t{party}"), party),
        )
    };
    control("separate").unwrap();
    control("garden").unwrap();
    assert!(control("stranger").is_err());

    // A trip with two partners, its first purchase and its first entry (PRD FR-12.29).
    let partner_ids: Vec<i64> = conn
        .prepare("SELECT id FROM partners ORDER BY id")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    conn.execute(
        "INSERT INTO ventures (kind, number, entry_date, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES ('trip', 1, '2026-10-10', 'سفر-1', 'TRP-1', 'سفر-1', 'trp-1', 'trp')",
        [],
    )
    .unwrap();
    let trip = conn.last_insert_rowid();
    assert!(
        conn.execute(
            "INSERT INTO ventures (kind, number, entry_date, name_ur, name_en, search_ur, search_en, search_phonetic)
             VALUES ('trip', 1, '2026-10-10', 'x', 'x', 'x', 'x', 'x')",
            [],
        )
        .is_err(),
        "a trip number is used once"
    );
    assert!(
        conn.execute(
            "INSERT INTO ventures (kind, number, entry_date, name_ur, name_en, search_ur, search_en, search_phonetic, rent_paisa)
             VALUES ('trip', 2, '2026-10-10', 'x', 'x', 'x', 'x', 'x', 100)",
            [],
        )
        .is_err(),
        "a trip has no rent"
    );
    assert!(
        conn.execute(
            "INSERT INTO ventures (kind, number, entry_date, name_ur, name_en, search_ur, search_en, search_phonetic)
             VALUES ('garden', 1, '2026-10-10', 'x', 'x', 'x', 'x', 'x')",
            [],
        )
        .is_err(),
        "a garden has its account"
    );
    for (partner, share, source) in [
        (partner_ids[0], 6_000, "shop"),
        (partner_ids[1], 4_000, "pocket"),
    ] {
        conn.execute(
            "INSERT INTO venture_partners (venture_id, partner_id, share_bps, source) VALUES (?1, ?2, ?3, ?4)",
            (trip, partner, share, source),
        )
        .unwrap();
    }
    assert!(
        conn.execute(
            "INSERT INTO venture_partners (venture_id, partner_id, share_bps, source) VALUES (?1, ?2, 0, 'shop')",
            (trip, partner_ids[0]),
        )
        .is_err(),
        "a partner is in a venture once, with a share above 0"
    );
    // The purchase: 20 at 500 = 10,000 and freight 400, paid now 10,400 (E2E-34).
    conn.execute(
        "INSERT INTO purchases (number, venture_id, entry_date, item_id, packing_id, qty, rate_paisa, value_paisa,
             freight_paisa, paid_now_paisa, total_cost_paisa)
         SELECT 1, ?1, '2026-10-10', item_id, packing_id, 20, 50_000, 1_000_000, 40_000, 1_040_000, 1_040_000
         FROM bikri_lots LIMIT 1",
        [trip],
    )
    .unwrap();
    let purchase_id = conn.last_insert_rowid();
    let bad_purchase = |paid: i64, total: i64, extra: &str| {
        conn.execute(
            &format!(
                "INSERT INTO purchases (number, venture_id, entry_date, item_id, packing_id, qty, rate_paisa,
                     value_paisa, freight_paisa, paid_now_paisa, total_cost_paisa {extra})
                 SELECT 9, ?1, '2026-10-10', item_id, packing_id, 20, 50_000, 1_000_000, 40_000, ?2, ?3
                 FROM bikri_lots LIMIT 1"
            ),
            (trip, paid, total),
        )
    };
    assert!(
        bad_purchase(1_000_000, 1_000_000, "").is_err(),
        "paid now is the value plus freight, labour and other cost"
    );
    assert!(
        bad_purchase(1_040_000, 1_000_000, "").is_err(),
        "the total cost is paid now plus the trip costs"
    );
    // The first entry of the trip: its payment, with the partners' parts (C23: 6,240 and 4,160).
    conn.execute(
        "INSERT INTO venture_entries (venture_id, kind, number, entry_date, amount_paisa, purchase_id)
         VALUES (?1, 'purchase', 1, '2026-10-10', 1_040_000, ?2)",
        (trip, purchase_id),
    )
    .unwrap();
    let entry = conn.last_insert_rowid();
    for (partner, share, source, amount) in [
        (partner_ids[0], 6_000, "shop", 624_000),
        (partner_ids[1], 4_000, "pocket", 416_000),
    ] {
        conn.execute(
            "INSERT INTO venture_entry_parts (entry_id, partner_id, share_bps, source, amount_paisa)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (entry, partner, share, source, amount),
        )
        .unwrap();
    }
    assert!(
        conn.execute(
            "UPDATE venture_entry_parts SET amount_paisa = 100 WHERE entry_id = ?1",
            [entry]
        )
        .is_err(),
        "the parts of an entry are final"
    );
    assert!(
        conn.execute(
            "INSERT INTO venture_entries (venture_id, kind, number, entry_date, amount_paisa)
             VALUES (?1, 'cost', 1, '2026-10-11', 100)",
            [trip],
        )
        .is_err(),
        "a trip cost says why"
    );
    // Shares are locked after the first entry (PRD FR-12.29).
    assert!(
        conn.execute(
            "UPDATE venture_partners SET share_bps = 5_000 WHERE venture_id = ?1 AND partner_id = ?2",
            (trip, partner_ids[0]),
        )
        .is_err()
    );
    assert!(
        conn.execute("DELETE FROM venture_partners WHERE venture_id = ?1", [trip])
            .is_err()
    );
    conn.execute(
        "UPDATE venture_partners SET source = 'shop' WHERE venture_id = ?1 AND partner_id = ?2",
        (trip, partner_ids[1]),
    )
    .unwrap();
    // A trip cost spread over the purchase (C24); the purchase's total moves with its part.
    conn.execute(
        "INSERT INTO venture_entries (venture_id, kind, number, entry_date, amount_paisa, reason)
         VALUES (?1, 'cost', 1, '2026-10-11', 40_000, 'truck')",
        [trip],
    )
    .unwrap();
    let cost_entry = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO purchase_cost_parts (entry_id, purchase_id, amount_paisa) VALUES (?1, ?2, 40_000)",
        (cost_entry, purchase_id),
    )
    .unwrap();
    conn.execute(
        "UPDATE purchases SET trip_cost_paisa = 40_000, total_cost_paisa = 1_080_000 WHERE id = ?1",
        [purchase_id],
    )
    .unwrap();
    assert!(
        conn.execute(
            "UPDATE purchases SET trip_cost_paisa = 0 WHERE id = ?1",
            [purchase_id]
        )
        .is_err(),
        "the total cost is paid now plus the trip costs"
    );
    assert!(
        conn.execute("DELETE FROM purchase_cost_parts", []).is_err(),
        "the spread of a cost is final"
    );
    // The partner of an open trip is found (PRD A63), and a venture is never deleted.
    assert_eq!(
        count(&format!(
            "SELECT EXISTS (SELECT 1 FROM venture_partners vp JOIN ventures v ON v.id = vp.venture_id
                            WHERE vp.partner_id = {} AND v.status = 'open')",
            partner_ids[0]
        )),
        1
    );
    assert!(matches!(
        common::classify(
            conn.execute("DELETE FROM ventures WHERE id = ?1", [trip])
                .unwrap_err()
        ),
        DbError::AppendOnly
    ));
    conn.execute(
        "UPDATE ventures SET status = 'closed', closed_date = '2026-10-12' WHERE id = ?1",
        [trip],
    )
    .unwrap();
    assert!(
        conn.execute("UPDATE ventures SET number = 9 WHERE id = ?1", [trip])
            .is_err(),
        "the identity of a venture is final"
    );

    // A purchase move has no lot; a move with neither a lot nor a purchase is refused. A reversal
    // names the same lot, purchase and cost as the move it reverses, with the opposite sign.
    let move_of = |lot: Option<i64>, purchase: Option<i64>, kind: &str, qty: i64, cost: i64| {
        conn.execute(
            "INSERT INTO stock_moves (lot_id, purchase_id, journal_entry_id, move_date, kind, qty, cost_paisa)
             VALUES (?1, ?2, NULL, '2026-10-10', ?3, ?4, ?5)",
            (lot, purchase, kind, qty, cost),
        )
    };
    move_of(None, None, "received", 20, 0).unwrap_err();
    move_of(None, Some(purchase_id), "received", 20, 0).unwrap();
    move_of(Some(f.lot), Some(purchase_id), "sold", -15, 0).unwrap();
    move_of(None, Some(purchase_id), "written_off", -5, 260_000).unwrap();
    let off = conn.last_insert_rowid();
    move_of(Some(f.lot), None, "taken", -1, 0).unwrap();
    let reversal = |cost: i64, qty: i64| {
        conn.execute(
            "INSERT INTO stock_moves (lot_id, purchase_id, move_date, kind, qty, cost_paisa, reverses_move_id)
             VALUES (NULL, ?1, '2026-10-10', 'reversal', ?2, ?3, ?4)",
            (purchase_id, qty, cost, off),
        )
    };
    assert!(
        reversal(260_000, 5).is_err(),
        "the cost of a reversal is the opposite of the move's"
    );
    assert!(
        reversal(-260_000, 4).is_err(),
        "the qty of a reversal is the opposite of the move's"
    );
    reversal(-260_000, 5).unwrap();
    assert!(
        conn.execute(
            "INSERT INTO stock_moves (lot_id, purchase_id, move_date, kind, qty, cost_paisa, reverses_move_id)
             VALUES (?1, ?2, '2026-10-10', 'reversal', 5, -260000, ?3)",
            (f.lot, purchase_id, off),
        )
        .is_err(),
        "a reversal names the same lot and purchase as the move it reverses"
    );
    // A lot cannot give more than it received, counting the fruit taken from it.
    conn.execute(
        "UPDATE bikri_lots SET qty_taken = 30, qty_sold = 20 WHERE id = ?1",
        [f.lot],
    )
    .unwrap();
    assert!(
        conn.execute(
            "UPDATE bikri_lots SET qty_taken = 31 WHERE id = ?1",
            [f.lot]
        )
        .is_err()
    );

    // A purchase is never deleted; its record is final; the running figures can move.
    assert!(matches!(
        common::classify(
            conn.execute("DELETE FROM purchases WHERE id = ?1", [purchase_id])
                .unwrap_err()
        ),
        DbError::AppendOnly
    ));
    assert!(
        conn.execute("UPDATE purchases SET qty = 21 WHERE id = ?1", [purchase_id])
            .is_err()
    );
    conn.execute(
        "UPDATE purchases SET qty_sold = 15, cost_released_paisa = 810_000 WHERE id = ?1",
        [purchase_id],
    )
    .unwrap();
    assert!(
        conn.execute(
            "UPDATE purchases SET qty_sold = 21 WHERE id = ?1",
            [purchase_id]
        )
        .is_err(),
        "sold is never more than bought"
    );
    assert!(
        conn.execute(
            "UPDATE purchases SET cost_released_paisa = 1_080_100 WHERE id = ?1",
            [purchase_id],
        )
        .is_err(),
        "the cost released is never more than the cost"
    );

    // The partners' parts of a trip lot's seller credit (C25): E2E-34's 8,460 at 60 / 40 is 5,076 and
    // 3,384 here, one row for each (posting, lot, partner); a row is never changed.
    for (partner, share, amount) in [
        (partner_ids[0], 6_000, 507_600),
        (partner_ids[1], 4_000, 338_400),
    ] {
        conn.execute(
            "INSERT INTO lot_credit_parts (lot_id, journal_entry_id, partner_id, share_bps, amount_paisa)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (f.lot, f.entry, partner, share, amount),
        )
        .unwrap();
    }
    assert!(
        conn.execute(
            "INSERT INTO lot_credit_parts (lot_id, journal_entry_id, partner_id, share_bps, amount_paisa)
             VALUES (?1, ?2, ?3, 6000, 100)",
            (f.lot, f.entry, partner_ids[0]),
        )
        .is_err(),
        "one row for each posting, lot and partner"
    );
    assert!(
        conn.execute("UPDATE lot_credit_parts SET amount_paisa = 100", [])
            .is_err()
    );
    assert!(conn.execute("DELETE FROM lot_credit_parts", []).is_err());

    // Fruit taken: the table accepts a take for one partner and for the guests; a take from our shop
    // names its lot.
    let take = |partner: Option<i64>, source: &str, lot: Option<i64>, number: i64| {
        conn.execute(
            "INSERT INTO fruit_takes (number, entry_date, partner_id, source, lot_id, credit_account_id, item_id,
                 packing_id, qty, rate_paisa, value_paisa)
             SELECT ?1, '2026-10-10', ?2, ?3, ?4, ?5, item_id, packing_id, 2, 58_000, 116_000
             FROM bikri_lots LIMIT 1",
            (number, partner, source, lot, f.seller),
        )
    };
    take(Some(partner_ids[0]), "shop", Some(f.lot), 1).unwrap();
    take(None, "other_shop", None, 2).unwrap();
    take(None, "shop", None, 3).unwrap_err();

    // The copy is the version-5 file as it was.
    let copy_conn = Connection::open(outcome.copy.unwrap()).unwrap();
    assert_eq!(user_version(&copy_conn), 5);
    assert!(!table_exists(&copy_conn, "purchases"));
    assert!(!table_exists(&copy_conn, "ventures"));
    assert_eq!(seed::counts(&copy_conn).unwrap().accounts, accounts_before);
}

/// Adds an account with an old-style code straight into a file of version 6.
fn add_old_account(
    conn: &Connection,
    control: &str,
    code: &str,
    serial: i64,
    party: &str,
    name: &str,
) -> i64 {
    conn.execute(
        "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur,
             search_en, search_phonetic)
         SELECT id, ?2, ?3, ?4, ?5, ?5, ?5, lower(?5), lower(?5) FROM account_controls WHERE code = ?1",
        (control, code, serial, party, name),
    )
    .unwrap();
    conn.last_insert_rowid()
}

#[test]
fn migration_7_renumbers_the_codes_and_changes_nothing_else() {
    // Plan 32 (PRD 1.8): a populated file made at version 6 (old codes, posted entries with their
    // hash chain, items, packings and staff) is upgraded: a copy is taken, sellers, buyers, the two
    // پختہ کھاتہ accounts, items, packings and staff get the new codes in the order they have, staff
    // get a code column, the upper limit of `accounts.serial` is gone. Every other code, every id,
    // every amount, every balance and every chain hash is what it was.
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate_with(&released_version_6(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!(user_version(&conn), 6);

    // Sellers with a gap in their serials, buyers, a khata and a person who borrows.
    let ids: Vec<i64> = [
        ("03011", "030110001", 1, "seller", "Rahim"),
        ("03011", "030110002", 2, "seller", "Karim"),
        ("03011", "030110005", 5, "seller", "Nazir"),
        ("0304", "03040001", 1, "buyer", "Akram"),
        ("0304", "03040002", 2, "buyer", "Arshad"),
        ("0304", "03040010", 10, "buyer", "Tariq"),
        ("0305", "03050001", 1, "separate", "Family"),
        ("0103", "01030001", 1, "other", "Aslam Lent"),
    ]
    .into_iter()
    .map(|(control, code, serial, party, name)| {
        add_old_account(&conn, control, code, serial, party, name)
    })
    .collect();
    // Items and packings with codes typed by hand, out of order.
    for (code, name) in [("2", "Mango"), ("1", "Banana"), ("12", "Apple")] {
        conn.execute(
            "INSERT INTO items (code, name_ur, name_en, search_ur, search_en, search_phonetic, item_group)
             VALUES (?1, ?2, ?2, ?2, lower(?2), lower(?2), 'fruit')",
            (code, name),
        )
        .unwrap();
    }
    for (code, name) in [("3", "Box"), ("1", "Bag")] {
        conn.execute(
            "INSERT INTO packings (code, name_ur, name_en, search_ur, search_en, search_phonetic)
             VALUES (?1, ?2, ?2, ?2, lower(?2), lower(?2))",
            (code, name),
        )
        .unwrap();
    }
    for name in ["Aslam", "Bilal"] {
        conn.execute(
            "INSERT INTO staff (name_ur, name_en, search_ur, search_en, search_phonetic, role)
             VALUES (?1, ?1, ?1, lower(?1), lower(?1), 'munshi')",
            [name],
        )
        .unwrap();
    }
    post_chained(&conn, "2026-10-10", "03040001", "04010001", 12_880, None);
    post_chained(&conn, "2026-10-10", "01010001", "03040001", 10_000, None);
    post_chained(&conn, "2026-10-10", "030110002", "01010001", 4_316, None);

    let hashes = |conn: &Connection| -> Vec<(i64, String)> {
        conn.prepare(
            "SELECT id, chain_hash FROM journal_entries WHERE status = 'posted' ORDER BY id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
    };
    let before_hashes = hashes(&conn);
    assert_eq!(before_hashes.len(), 3);
    let skip_codes = [
        ("accounts", "code"),
        ("accounts", "serial"),
        ("items", "code"),
        ("packings", "code"),
        ("staff", "code"),
    ];
    let before_rows = all_rows_without(&conn, &skip_codes);
    let before_totals = mandi_db::ledger::account_totals(&conn, None).unwrap();
    let before_triggers: Vec<String> = schema(&conn, "trigger");

    let outcome =
        migrate::migrate_with(&released_version_7(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!((outcome.from, outcome.to), (6, 7));
    assert!(outcome.copy.is_some(), "a copy is taken before an upgrade");
    assert_eq!(user_version(&conn), 7);

    // The new codes, in the order the records have.
    let code_of = |id: i64| -> (String, i64) {
        conn.query_row(
            "SELECT code, serial FROM accounts WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
    };
    let got: Vec<(String, i64)> = ids.iter().map(|id| code_of(*id)).collect();
    let want = [
        ("0010", 0),
        ("0011", 1),
        ("0012", 2),
        ("0020", 0),
        ("0021", 1),
        ("0022", 2),
        ("03050001", 1),
        ("01030001", 1),
    ];
    assert_eq!(
        got,
        want.map(|(c, n)| (c.to_owned(), n)).to_vec(),
        "sellers and buyers are numbered from 0 in their old order; the khata and the lent person keep theirs"
    );
    let one = |sql: &str| -> (String, i64) {
        conn.query_row(sql, [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
    };
    assert_eq!(
        one("SELECT code, serial FROM accounts WHERE is_pukhta_khata = 1 AND party_type = 'buyer'"),
        ("0030".to_owned(), 0)
    );
    assert_eq!(
        one(
            "SELECT code, serial FROM accounts WHERE is_pukhta_khata = 1 AND party_type = 'seller'"
        ),
        ("0031".to_owned(), 1)
    );
    for kept in [
        "01010001",
        "01020001",
        "02010001",
        "03040000",
        "030119998",
        "04010001",
        "06010001",
        "06019999",
    ] {
        let found: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM accounts WHERE code = ?1",
                [kept],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(found, 1, "{kept} keeps its code");
    }
    let list = |sql: &str| -> Vec<String> {
        conn.prepare(sql)
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert_eq!(
        list("SELECT code || ' ' || name_en FROM items ORDER BY code"),
        ["0040 Banana", "0041 Mango", "0042 Apple"],
        "items follow the order of their old codes as numbers: 1, 2, 12"
    );
    assert_eq!(
        list("SELECT code || ' ' || name_en FROM packings ORDER BY code"),
        ["0050 Bag", "0051 Box"]
    );
    assert_eq!(
        list("SELECT code || ' ' || name_en FROM staff ORDER BY code"),
        ["0060 Aslam", "0061 Bilal"]
    );

    // The staff code is unique; the serial of an account has no upper limit any more, but is not
    // negative; a system account still cannot change its code.
    assert!(
        conn.execute(
            "INSERT INTO staff (name_ur, name_en, search_ur, search_en, search_phonetic, role, code)
             VALUES ('x', 'x', 'x', 'x', 'x', 'munshi', '0060')",
            [],
        )
        .is_err()
    );
    add_old_account(&conn, "03011", "0019999999", 9_999_999, "seller", "Last");
    assert_eq!(
        one("SELECT code, serial FROM accounts WHERE name_en = 'Last'"),
        ("0019999999".to_owned(), 9_999_999)
    );
    assert!(
        conn.execute(
            "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur,
                 search_en, search_phonetic)
             SELECT id, '0019', -1, 'seller', 'n', 'n', 'n', 'n', 'n' FROM account_controls WHERE code = '03011'",
            [],
        )
        .is_err(),
        "a negative serial is still refused"
    );
    let changed = conn.execute("UPDATE accounts SET code = '9' WHERE code = '0030'", []);
    assert!(changed.is_err(), "a system account keeps its code");
    assert_eq!(
        schema(&conn, "trigger"),
        before_triggers,
        "no trigger is lost or changed"
    );
    let temp: i64 = conn
        .query_row("SELECT COUNT(*) FROM sqlite_temp_master", [], |r| r.get(0))
        .unwrap();
    assert_eq!(temp, 0, "the working table of the migration is gone");

    // Nothing else moved: every other column of every row, every hash, every balance.
    // The account added above is left out of the comparison.
    conn.execute("DELETE FROM accounts WHERE name_en = 'Last'", [])
        .unwrap();
    assert_eq!(all_rows_without(&conn, &skip_codes), before_rows);
    assert_eq!(hashes(&conn), before_hashes);
    mandi_db::ledger::verify_journal_chain(&conn, &common::test_key()).unwrap();
    assert_eq!(
        mandi_db::ledger::account_totals(&conn, None).unwrap(),
        before_totals
    );
    let bad_keys: i64 = conn
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(bad_keys, 0);
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
}

#[test]
fn migration_8_gives_trips_and_gardens_their_codes_and_changes_nothing_else() {
    // Plan 34: a populated file made at version 7 (three trips named the old way, two gardens with
    // their accounts under control 0306, posted entries with their hash chain) is upgraded: a copy
    // is taken, each garden's account gets the code of its garden, each trip gets the name "0070
    // 10-10-2026" in both scripts with its search keys made again. Nothing else moves.
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate_with(&released_version_7(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!(user_version(&conn), 7);

    let garden_one = add_old_account(&conn, "0306", "03060001", 1, "garden", "Shahpur");
    let garden_two = add_old_account(&conn, "0306", "03060002", 2, "garden", "Multan Bagh");
    for (kind, number, day, name_ur, name_en, account) in [
        (
            "trip",
            1,
            "2026-10-10",
            "سفر-1 10-10-2026",
            "TRP-1 10-10-2026",
            None,
        ),
        (
            "trip",
            2,
            "2026-10-11",
            "سفر-2 11-10-2026",
            "TRP-2 11-10-2026",
            None,
        ),
        (
            "garden",
            1,
            "2026-10-10",
            "شاہ پور",
            "Shahpur",
            Some(garden_one),
        ),
        (
            "garden",
            2,
            "2026-10-12",
            "ملتان باغ",
            "Multan Bagh",
            Some(garden_two),
        ),
        (
            "trip",
            3,
            "2026-10-12",
            "سفر-3 12-10-2026",
            "TRP-3 12-10-2026",
            None,
        ),
    ] {
        conn.execute(
            "INSERT INTO ventures (kind, number, entry_date, name_ur, name_en, search_ur, search_en,
                 search_phonetic, account_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?4, lower(?5), lower(?5), ?6)",
            (kind, number, day, name_ur, name_en, account),
        )
        .unwrap();
    }
    post_chained(&conn, "2026-10-10", "01010001", "03060001", 5_000, None);
    post_chained(&conn, "2026-10-12", "03060002", "01010001", 1_000, None);

    let hashes = |conn: &Connection| -> Vec<(i64, String)> {
        conn.prepare(
            "SELECT id, chain_hash FROM journal_entries WHERE status = 'posted' ORDER BY id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
    };
    let before_hashes = hashes(&conn);
    assert_eq!(before_hashes.len(), 2);
    let skip = [
        ("accounts", "code"),
        ("accounts", "serial"),
        ("ventures", "name_ur"),
        ("ventures", "name_en"),
        ("ventures", "search_ur"),
        ("ventures", "search_en"),
        ("ventures", "search_phonetic"),
    ];
    let before_rows = all_rows_without(&conn, &skip);
    let before_totals = mandi_db::ledger::account_totals(&conn, None).unwrap();

    let outcome =
        migrate::migrate_with(&released_version_8(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!((outcome.from, outcome.to), (7, 8));
    assert!(outcome.copy.is_some(), "a copy is taken before an upgrade");
    assert_eq!(user_version(&conn), 8);

    // The gardens' accounts have the code of their garden, and the serial is the running number.
    let account_of = |id: i64| -> (String, i64) {
        conn.query_row(
            "SELECT code, serial FROM accounts WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
    };
    assert_eq!(account_of(garden_one), ("0080".to_owned(), 0));
    assert_eq!(account_of(garden_two), ("0081".to_owned(), 1));
    // The trips are named by their code and day, the same in both scripts, with new search keys.
    let trips: Vec<(i64, String, String, String)> = conn
        .prepare("SELECT number, name_ur, name_en, search_en FROM ventures WHERE kind = 'trip' ORDER BY number")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let want = [
        (1, "0070 10-10-2026"),
        (2, "0071 11-10-2026"),
        (3, "0072 12-10-2026"),
    ];
    for ((number, ur, en, key), (want_number, want_name)) in trips.iter().zip(want) {
        assert_eq!(*number, want_number);
        assert_eq!((ur.as_str(), en.as_str()), (want_name, want_name));
        assert_eq!(key, &mandi_core::text::search_en(want_name));
    }
    // The gardens keep the names they were typed with.
    let garden_names: Vec<String> = conn
        .prepare("SELECT name_en FROM ventures WHERE kind = 'garden' ORDER BY number")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(garden_names, ["Shahpur", "Multan Bagh"]);

    // Nothing else moved: every other column of every row, every hash, every balance.
    assert_eq!(all_rows_without(&conn, &skip), before_rows);
    assert_eq!(hashes(&conn), before_hashes);
    mandi_db::ledger::verify_journal_chain(&conn, &common::test_key()).unwrap();
    assert_eq!(
        mandi_db::ledger::account_totals(&conn, None).unwrap(),
        before_totals
    );
    let bad_keys: i64 = conn
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(bad_keys, 0);
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
}

#[test]
fn migration_9_adds_the_date_to_sell_and_the_arrivals_and_changes_nothing_else() {
    // Plan 33 v1.1: a file at version 8 with a trip and two purchases is upgraded. Each purchase gets
    // its own entry date as its date to sell; the table of garden arrivals, the two link columns and
    // the triggers exist; no other row or number changes.
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let mut conn = connection(&dir.path().join("mandi.db"));
    migrate::migrate_with(&released_version_8(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!(user_version(&conn), 8);
    conn.execute(
        "INSERT INTO items (code, name_ur, name_en, search_ur, search_en, search_phonetic, item_group)
         VALUES ('0040', 'x', 'x', 'x', 'x', 'x', 'fruit')",
        [],
    )
    .unwrap();
    let item = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO packings (code, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES ('0050', 'x', 'x', 'x', 'x', 'x')",
        [],
    )
    .unwrap();
    let packing = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO ventures (kind, number, entry_date, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES ('trip', 1, '2026-10-10', '0070 10-10-2026', '0070 10-10-2026', 'x', 'x', 'x')",
        [],
    )
    .unwrap();
    let trip = conn.last_insert_rowid();
    for (number, day) in [(1, "2026-10-10"), (2, "2026-10-12")] {
        conn.execute(
            "INSERT INTO purchases (number, venture_id, entry_date, item_id, packing_id, qty, rate_paisa,
                 value_paisa, freight_paisa, paid_now_paisa, total_cost_paisa)
             VALUES (?1, ?2, ?3, ?4, ?5, 20, 50_000, 1_000_000, 40_000, 1_040_000, 1_040_000)",
            (number, trip, day, item, packing),
        )
        .unwrap();
    }
    let skip = [("purchases", "sell_date")];
    let before_rows = all_rows_without(&conn, &skip);

    let outcome = migrate::migrate(&mut conn, &backups, &clock()).unwrap();
    assert_eq!((outcome.from, outcome.to), (8, 9));
    assert!(outcome.copy.is_some(), "a copy is taken before an upgrade");
    assert_eq!(user_version(&conn), 9);

    // Nothing else moved: every other column of every other table reads as before.
    let mut after_rows = all_rows_without(&conn, &skip);
    after_rows.retain(|(table, _)| table != "garden_arrivals");
    let mut before_rows = before_rows;
    before_rows.retain(|(table, _)| table != "garden_arrivals");
    assert_eq!(after_rows, before_rows);

    // Each purchase starts with its own entry date as its date to sell.
    let sell: Vec<(String, String)> = conn
        .prepare("SELECT entry_date, sell_date FROM purchases ORDER BY number")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(
        sell,
        [
            ("2026-10-10".into(), "2026-10-10".into()),
            ("2026-10-12".into(), "2026-10-12".into())
        ]
    );
    // A date to sell later than the purchase day is allowed; an earlier one is refused.
    conn.execute(
        "UPDATE purchases SET sell_date = '2026-10-15' WHERE number = 1",
        [],
    )
    .unwrap();
    let early = conn.execute(
        "UPDATE purchases SET sell_date = '2026-10-09' WHERE number = 1",
        [],
    );
    assert!(early.is_err(), "a date to sell before the purchase day");
    let new_early = conn.execute(
        "INSERT INTO purchases (number, venture_id, entry_date, sell_date, item_id, packing_id, qty,
             rate_paisa, value_paisa, freight_paisa, paid_now_paisa, total_cost_paisa)
         VALUES (9, ?1, '2026-10-12', '2026-10-11', ?2, ?3, 20, 50_000, 1_000_000, 0, 1_000_000, 1_000_000)",
        (trip, item, packing),
    );
    assert!(
        new_early.is_err(),
        "a new purchase with an early date to sell"
    );

    // The arrivals table: empty, with its checks and triggers.
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM garden_arrivals"),
        0,
        "no arrival yet"
    );
    let account: i64 = conn
        .query_row("SELECT id FROM accounts LIMIT 1", [], |r| r.get(0))
        .unwrap();
    conn.execute(
        "INSERT INTO ventures (kind, number, entry_date, name_ur, name_en, search_ur, search_en, search_phonetic, account_id)
         VALUES ('garden', 1, '2026-10-10', 'g', 'g', 'g', 'g', 'g', ?1)",
        [account],
    )
    .unwrap();
    let garden = conn.last_insert_rowid();
    let arrival = |number: i64, day: &str, sell: &str, qty: i64| {
        conn.execute(
            "INSERT INTO garden_arrivals (number, venture_id, entry_date, sell_date, item_id, packing_id, qty)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (number, garden, day, sell, item, packing, qty),
        )
    };
    arrival(1, "2026-10-10", "2026-10-11", 100).unwrap();
    assert!(
        arrival(2, "2026-10-10", "2026-10-09", 100).is_err(),
        "sell date before arrival"
    );
    assert!(
        arrival(1, "2026-10-10", "2026-10-10", 100).is_err(),
        "number used once"
    );
    assert!(
        arrival(3, "2026-10-10", "2026-10-10", 0).is_err(),
        "qty above 0"
    );
    let arrival_id: i64 = conn
        .query_row("SELECT id FROM garden_arrivals WHERE number = 1", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(
        conn.execute(
            "UPDATE garden_arrivals SET qty_sold = 101 WHERE id = ?1",
            [arrival_id]
        )
        .is_err(),
        "sold plus written off is never more than arrived"
    );
    assert!(
        conn.execute(
            "UPDATE garden_arrivals SET qty = 50 WHERE id = ?1",
            [arrival_id]
        )
        .is_err(),
        "what arrived never changes"
    );
    conn.execute(
        "UPDATE garden_arrivals SET qty_sold = 60, sell_date = '2026-10-12' WHERE id = ?1",
        [arrival_id],
    )
    .unwrap();
    assert!(
        conn.execute("DELETE FROM garden_arrivals WHERE id = ?1", [arrival_id])
            .is_err(),
        "an arrival is never deleted"
    );
    assert!(
        conn.execute(
            "UPDATE garden_arrivals SET status = 'cancelled' WHERE id = ?1",
            [arrival_id]
        )
        .is_err(),
        "a cancel needs a reason"
    );
    // A stock move may belong to an arrival, and a reversal must mirror it.
    conn.execute(
        "INSERT INTO stock_moves (arrival_id, move_date, kind, qty) VALUES (?1, '2026-10-11', 'written_off', -5)",
        [arrival_id],
    )
    .unwrap();
    let move_id = conn.last_insert_rowid();
    assert!(
        conn.execute(
            "INSERT INTO stock_moves (move_date, kind, qty) VALUES ('2026-10-11', 'written_off', -5)",
            [],
        )
        .is_err(),
        "a move has an owner"
    );
    assert!(
        conn.execute(
            "INSERT INTO stock_moves (arrival_id, move_date, kind, qty, reverses_move_id)
             VALUES (?1, '2026-10-11', 'reversal', 4, ?2)",
            (arrival_id, move_id),
        )
        .is_err(),
        "a reversal is the exact opposite"
    );
    conn.execute(
        "INSERT INTO stock_moves (arrival_id, move_date, kind, qty, reverses_move_id)
         VALUES (?1, '2026-10-11', 'reversal', 5, ?2)",
        (arrival_id, move_id),
    )
    .unwrap();
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}
