//! `Db::read_snapshot` and the rules of the kept sums (plan 38 B2, found in the money review).
//! PRD FR-6.1: the figures of one screen are read at one moment, whatever is saved meanwhile.
#![cfg(test)]

mod common;

use std::cell::Cell;

use mandi_db::memo::{self, Sums};
use mandi_db::{Db, DbError};
use rusqlite::Connection;
use tempfile::tempdir;

fn add_city(db: &Db, code: &str) -> Result<(), DbError> {
    db.write(|tx| {
        tx.execute(
            "INSERT INTO cities (code, name_ur, name_en, search_ur, search_en, search_phonetic) VALUES (?1, 'ملتان', 'Multan', 'ملتان', 'multan', 'mltn')",
            [code],
        )?;
        Ok(())
    })
}

fn cities(conn: &Connection) -> Result<i64, DbError> {
    Ok(conn.query_row("SELECT COUNT(*) FROM cities", [], |r| r.get(0))?)
}

#[test]
fn a_snapshot_read_does_not_see_a_save_that_commits_in_the_middle() {
    let dir = tempdir().unwrap();
    let db = common::open(dir.path());
    add_city(&db, "1").unwrap();
    let seen = db
        .read_snapshot(|conn| -> Result<(i64, i64), DbError> {
            let first = cities(conn)?;
            add_city(&db, "2")?;
            Ok((first, cities(conn)?))
        })
        .unwrap();
    assert_eq!(seen, (1, 1), "both statements saw the same moment");
    // The next read sees the save.
    let after = db.read(cities).unwrap();
    assert_eq!(after, 2);
    // The plain read has no such promise: it is the reason the other exists.
    let plain = db
        .read(|conn| -> Result<(i64, i64), DbError> {
            let first = cities(conn)?;
            add_city(&db, "3")?;
            Ok((first, cities(conn)?))
        })
        .unwrap();
    assert_eq!(plain, (2, 3));
}

fn counted(calls: &Cell<u32>) -> impl FnOnce() -> rusqlite::Result<Sums> + '_ {
    move || {
        calls.set(calls.get() + 1);
        Ok(Sums {
            count: 1,
            dr_paisa: 100,
            cr_paisa: 0,
        })
    }
}

#[test]
fn only_the_read_connection_keeps_sums() {
    let dir = tempdir().unwrap();
    let db = common::open(dir.path());
    let calls = Cell::new(0);
    db.read(|conn| -> Result<(), DbError> {
        memo::sums(conn, "same", counted(&calls))?;
        memo::sums(conn, "same", counted(&calls))?;
        Ok(())
    })
    .unwrap();
    assert_eq!(calls.get(), 1, "the second answer came from the memory");
    // The write connection sees lines that are not committed yet, so it never keeps an answer.
    let calls = Cell::new(0);
    db.write(|tx| -> Result<(), DbError> {
        memo::sums(tx, "other", counted(&calls))?;
        memo::sums(tx, "other", counted(&calls))?;
        Ok(())
    })
    .unwrap();
    assert_eq!(calls.get(), 2, "both were worked out");
}

#[test]
fn a_connection_without_a_file_keeps_nothing() {
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "query_only", "ON").unwrap();
    let calls = Cell::new(0);
    memo::sums(&conn, "same", counted(&calls)).unwrap();
    memo::sums(&conn, "same", counted(&calls)).unwrap();
    assert_eq!(
        calls.get(),
        2,
        "two in-memory databases cannot be told apart"
    );
}

#[test]
fn a_snapshot_of_the_books_is_one_moment_for_every_read_and_blocks_no_save() {
    // `Db::open_snapshot` (the assistant, NFR-AI-6): every read through it, however many calls and
    // however much later, sees the books as they were when it was made; saves go on meanwhile.
    let dir = tempdir().unwrap();
    let db = common::open(dir.path());
    add_city(&db, "1").unwrap();
    let snapshot = db.open_snapshot().unwrap();
    // A save commits after the snapshot: the writer is not blocked.
    add_city(&db, "2").unwrap();
    assert_eq!(snapshot.read(cities).unwrap(), 1, "the first read");
    add_city(&db, "3").unwrap();
    assert_eq!(snapshot.read(cities).unwrap(), 1, "a later read");
    assert_eq!(
        snapshot.read_snapshot(cities).unwrap(),
        1,
        "a read_snapshot inside it"
    );
    // The books themselves went on.
    assert_eq!(db.read(cities).unwrap(), 3);
    // A snapshot made now is a newer moment.
    assert_eq!(db.open_snapshot().unwrap().read(cities).unwrap(), 3);
    // Nothing can be saved through a snapshot.
    assert!(add_city(&snapshot, "9").is_err());
    assert_eq!(db.read(cities).unwrap(), 3);
}
