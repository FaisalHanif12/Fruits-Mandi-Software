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
    SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL,
};
use mandi_db::rusqlite::Connection;
use mandi_db::{DbError, seed};
use rusqlite_migration::{HookError, M, Migrations};
use tempfile::tempdir;

/// The schema version of the newest migration (`migrate::migrations()`).
const LATEST: usize = 5;

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
         SELECT id, '03040001', 1, 'buyer', 'اکرم', 'Akram', 'اکرم', 'akram', 'krm' FROM account_controls WHERE code = '0304'",
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
    conn.execute(
        "UPDATE accounts SET is_active = 0 WHERE code = '03040001'",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE accounts SET is_active = 1 WHERE code = '03040001'",
        [],
    )
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

    let outcome = migrate::migrate(&mut conn, &backups, &clock()).unwrap();
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
