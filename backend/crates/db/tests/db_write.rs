//! `Db::write`, `Db::read`, the start-up check and the disk check (plan 03 Sections 4.2, 4.3,
//! acceptance item 6). NFR-REL-1, NFR-REL-7, NFR-REL-10.
#![cfg(test)]

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use mandi_core::error::ErrorCode;
use mandi_db::disk::{FixedDiskProbe, MIN_FREE_DISK_BYTES};
use mandi_db::{DataState, Db, DbError};
use tempfile::tempdir;

fn add_city(db: &Db, code: &str, fail_after: bool) -> Result<(), DbError> {
    db.write(|tx| {
        tx.execute(
            "INSERT INTO cities (code, name_ur, name_en, search_ur, search_en, search_phonetic) VALUES (?1, 'ملتان', 'Multan', 'ملتان', 'multan', 'mltn')",
            [code],
        )?;
        if fail_after {
            return Err(DbError::Busy);
        }
        Ok(())
    })
}

fn city_count(db: &Db) -> i64 {
    db.read(|conn| -> Result<i64, DbError> {
        Ok(conn.query_row("SELECT COUNT(*) FROM cities", [], |r| r.get(0))?)
    })
    .unwrap()
}

#[test]
fn write_commits_on_ok_and_rolls_back_on_err() {
    let dir = tempdir().unwrap();
    let db = common::open(dir.path());
    add_city(&db, "1", false).unwrap();
    assert_eq!(city_count(&db), 1);
    assert!(add_city(&db, "2", true).is_err());
    assert_eq!(city_count(&db), 1, "the failed action left nothing behind");
}

#[test]
fn write_in_one_transaction_is_all_or_nothing() {
    let dir = tempdir().unwrap();
    let db = common::open(dir.path());
    let result: Result<(), DbError> = db.write(|tx| {
        tx.execute("INSERT INTO cities (code, name_ur, name_en, search_ur, search_en, search_phonetic) VALUES ('1', 'ملتان', 'Multan', 'ملتان', 'multan', 'mltn')", [])?;
        // The second statement breaks a rule of the database: a part of a rupee.
        tx.execute("INSERT INTO vouchers (type, number, entry_date, amount_paisa) VALUES ('receipt', 1, '2026-10-10', 150)", [])?;
        Ok(())
    });
    assert!(matches!(result, Err(DbError::Constraint(_))));
    assert_eq!(city_count(&db), 0);
}

#[test]
fn read_sees_what_write_committed() {
    let dir = tempdir().unwrap();
    let db = common::open(dir.path());
    add_city(&db, "7", false).unwrap();
    let code: String = db
        .read(|conn| -> Result<String, DbError> {
            Ok(conn.query_row("SELECT code FROM cities", [], |r| r.get(0))?)
        })
        .unwrap();
    assert_eq!(code, "7");
    // The read connection cannot write.
    let write_on_reader = db.read(|conn| -> Result<(), DbError> {
        conn.execute("DELETE FROM cities", [])?;
        Ok(())
    });
    assert!(write_on_reader.is_err());
}

#[test]
fn connections_use_the_settings_of_b13() {
    let dir = tempdir().unwrap();
    let db = common::open(dir.path());
    let pragma = |conn: &mandi_db::rusqlite::Connection, name: &str| -> String {
        conn.query_row(&format!("PRAGMA {name}"), [], |r| {
            r.get::<_, rusqlite_value::Text>(0).map(|t| t.0)
        })
        .unwrap()
    };
    let write_side: Vec<String> = db
        .write(|tx| -> Result<Vec<String>, DbError> {
            Ok([
                "journal_mode",
                "foreign_keys",
                "synchronous",
                "recursive_triggers",
            ]
            .iter()
            .map(|n| pragma(tx, n))
            .collect())
        })
        .unwrap();
    // synchronous 2 = FULL.
    assert_eq!(write_side, ["wal", "1", "2", "1"]);
    let read_side: Vec<String> = db
        .read(|conn| -> Result<Vec<String>, DbError> {
            Ok([
                "foreign_keys",
                "synchronous",
                "recursive_triggers",
                "query_only",
            ]
            .iter()
            .map(|n| pragma(conn, n))
            .collect())
        })
        .unwrap();
    assert_eq!(read_side, ["1", "2", "1", "1"]);
}

/// Reads any single column as text.
mod rusqlite_value {
    use mandi_db::rusqlite::types::{FromSql, FromSqlResult, ValueRef};

    pub struct Text(pub String);

    impl FromSql for Text {
        fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
            Ok(Text(match value {
                ValueRef::Integer(i) => i.to_string(),
                ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
                other => format!("{other:?}"),
            }))
        }
    }
}

#[test]
fn a_full_disk_saves_nothing_and_never_runs_the_action() {
    let dir = tempdir().unwrap();
    let mut options = common::options(dir.path());
    options.disk_probe = Arc::new(FixedDiskProbe(MIN_FREE_DISK_BYTES - 1));
    let db = Db::open_checked(&common::db_path(dir.path()), options).unwrap();
    let ran = AtomicBool::new(false);
    let result: Result<(), DbError> = db.write(|tx| {
        ran.store(true, Ordering::SeqCst);
        tx.execute("INSERT INTO cities (code, name_ur, name_en, search_ur, search_en, search_phonetic) VALUES ('1', 'ملتان', 'Multan', 'ملتان', 'multan', 'mltn')", [])?;
        Ok(())
    });
    let error = result.unwrap_err();
    assert!(
        matches!(error, DbError::DiskFull { available_bytes, needed_bytes } if available_bytes == MIN_FREE_DISK_BYTES - 1 && needed_bytes == MIN_FREE_DISK_BYTES)
    );
    assert_eq!(error.code(), ErrorCode::ErrDiskFull);
    assert!(!ran.load(Ordering::SeqCst));
    assert_eq!(city_count(&db), 0);
    // Exactly the limit is enough.
    let mut options = common::options(dir.path());
    options.disk_probe = Arc::new(FixedDiskProbe(MIN_FREE_DISK_BYTES));
    let db = Db::open_checked(&common::db_path(dir.path()), options).unwrap();
    add_city(&db, "1", false).unwrap();
}

#[test]
fn the_limit_is_50_mb() {
    assert_eq!(MIN_FREE_DISK_BYTES, 50 * 1024 * 1024);
}

#[test]
fn saving_waits_for_the_check_and_gives_up_with_db_busy() {
    let dir = tempdir().unwrap();
    // Opened but not checked: the state stays Checking.
    let db = Db::open(&common::db_path(dir.path()), common::options(dir.path())).unwrap();
    assert_eq!(db.data_state(), DataState::Checking);
    let error = add_city(&db, "1", false).unwrap_err();
    assert!(matches!(error, DbError::Busy));
    assert_eq!(error.code(), ErrorCode::ErrDbBusy);
    // Once the check passes, saving works.
    assert_eq!(db.run_quick_check(), DataState::Ok);
    add_city(&db, "1", false).unwrap();
}

#[test]
fn a_waiting_save_goes_on_when_the_background_check_passes() {
    let dir = tempdir().unwrap();
    let mut options = common::options(dir.path());
    options.check_wait = std::time::Duration::from_secs(20);
    let db = Arc::new(Db::open(&common::db_path(dir.path()), options).unwrap());
    let saver = {
        let db = Arc::clone(&db);
        std::thread::spawn(move || add_city(&db, "1", false))
    };
    let handle = db.spawn_quick_check().unwrap();
    assert_eq!(handle.join().unwrap(), DataState::Ok);
    saver.join().unwrap().unwrap();
    assert_eq!(city_count(&db), 1);
}

#[test]
fn a_failed_check_stops_every_save_with_err_data_check_failed() {
    let dir = tempdir().unwrap();
    {
        let db = common::open(dir.path());
        // Fill some pages so the file has table data to damage.
        for i in 0..300 {
            add_city(&db, &i.to_string(), false).unwrap();
        }
    }
    // Damage the file the way a bad disk does: overwrite whole pages in the middle.
    let path = common::db_path(dir.path());
    let mut bytes = std::fs::read(&path).unwrap();
    let page = 4096;
    assert!(
        bytes.len() > page * 40,
        "file is only {} bytes",
        bytes.len()
    );
    for byte in bytes.iter_mut().skip(page * 20).take(page * 10) {
        *byte = 0xFF;
    }
    std::fs::write(&path, &bytes).unwrap();
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));

    match Db::open(&path, common::options(dir.path())) {
        // The damage is found while opening.
        Err(error) => assert_eq!(error.code(), ErrorCode::ErrDataCheckFailed, "{error:?}"),
        // Or by the start-up check, which then stops every save.
        Ok(db) => {
            assert_eq!(db.run_quick_check(), DataState::Failed);
            assert!(db.failure_detail().is_some());
            let error = add_city(&db, "x", false).unwrap_err();
            assert_eq!(error.code(), ErrorCode::ErrDataCheckFailed);
            assert!(matches!(error, DbError::DataCheckFailed(_)));
        }
    }
}

/// Damage in the last pages (rows and their indexes) does not stop the file from opening, so
/// only the start-up check finds it. Then every save returns ERR_DATA_CHECK_FAILED.
#[test]
fn damage_found_only_by_the_start_up_check_stops_every_save() {
    let dir = tempdir().unwrap();
    {
        let db = common::open(dir.path());
        for i in 0..300 {
            add_city(&db, &i.to_string(), false).unwrap();
        }
    }
    let path = common::db_path(dir.path());
    let mut bytes = std::fs::read(&path).unwrap();
    let page = 4096;
    let len = bytes.len();
    for byte in bytes.iter_mut().skip(len - page * 3) {
        *byte = 0xFF;
    }
    std::fs::write(&path, &bytes).unwrap();
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));

    let db = Db::open(&path, common::options(dir.path())).expect("the file still opens");
    assert_eq!(db.run_quick_check(), DataState::Failed);
    assert!(db.failure_detail().is_some());
    let error = add_city(&db, "x", false).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrDataCheckFailed);
    assert!(matches!(error, DbError::DataCheckFailed(_)));
}

#[test]
fn a_file_that_is_not_a_database_gives_err_data_check_failed_and_no_panic() {
    let dir = tempdir().unwrap();
    let path = common::db_path(dir.path());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, vec![0xAB_u8; 8192]).unwrap();
    let error = Db::open(&path, common::options(dir.path())).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrDataCheckFailed, "{error:?}");
    // The garbage file is left as it was, for the restore offer.
    assert_eq!(std::fs::read(&path).unwrap(), vec![0xAB_u8; 8192]);
}

#[test]
fn error_codes_for_the_screen() {
    assert_eq!(DbError::Busy.code(), ErrorCode::ErrDbBusy);
    assert_eq!(
        DbError::MigrationFailed("x".into()).code(),
        ErrorCode::ErrMigrationFailed
    );
    assert_eq!(DbError::JournalUnbalanced.code(), ErrorCode::ErrInternal);
    assert_eq!(DbError::AppendOnly.code(), ErrorCode::ErrInternal);
}
