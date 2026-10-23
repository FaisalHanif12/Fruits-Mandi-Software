//! A snapshot opened for reading only (plan 22 round 10, PRD FR-20.5): reads work, nothing can
//! be written, nothing is upgraded, and a file with another schema is refused.
#![cfg(test)]

mod common;

use common::*;
use mandi_db::rusqlite::Connection;
use mandi_db::{Db, DbError};

#[test]
fn a_snapshot_opens_read_only_reads_and_refuses_every_write() {
    let dir = tempfile::tempdir().unwrap();
    let path = db_path(dir.path());
    // The live database: opened, checked, then closed (its WAL is folded into the file).
    drop(Db::open_checked(&path, options(dir.path())).unwrap());
    let before = std::fs::read(&path).unwrap();
    let db = Db::open_read_only(&path, options(dir.path())).unwrap();
    // Reads see the seeded books (the system accounts).
    let accounts: i64 = db
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0)))
        .unwrap();
    assert!(accounts > 0);
    // A save is refused by SQLite itself, whatever the code asks for.
    let write = db.write(|tx| {
        tx.execute("UPDATE accounts SET note = 'x'", [])
            .map_err(DbError::from)
    });
    assert!(write.is_err(), "a read-only snapshot took a write");
    let sneaky = db.read(|conn| {
        conn.execute("DELETE FROM accounts", [])
            .map_err(DbError::from)
    });
    assert!(sneaky.is_err(), "a read took a delete");
    drop(db);
    // The file is exactly as it was: no upgrade, no journal mode change.
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn a_file_with_an_older_schema_or_no_schema_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("old.db");
    {
        let conn = Connection::open(&old).unwrap();
        conn.execute_batch("CREATE TABLE t (n INTEGER); PRAGMA user_version = 1;")
            .unwrap();
    }
    let result = Db::open_read_only(&old, options(dir.path()));
    assert!(
        matches!(result, Err(DbError::MigrationFailed(ref text)) if text.contains("older")),
        "{:?}",
        result.err()
    );
    let empty = dir.path().join("empty.db");
    Connection::open(&empty)
        .unwrap()
        .execute_batch("CREATE TABLE t (n INTEGER);")
        .unwrap();
    assert!(matches!(
        Db::open_read_only(&empty, options(dir.path())),
        Err(DbError::MigrationFailed(_))
    ));
    // A file that is not a database at all is refused too.
    let junk = dir.path().join("junk.db");
    std::fs::write(&junk, b"this is not a database at all, just text").unwrap();
    assert!(Db::open_read_only(&junk, options(dir.path())).is_err());
}
