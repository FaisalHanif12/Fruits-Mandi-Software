//! A small memory of whole results, so the next page of a long list does not read ten years again
//! (robustness finding 2 and 3, NFR-SPD-24; decision of tech-lead, 15-11-2026).
//!
//! A page of an analytics view or of the commission page of a range needs a pass over every fact of
//! the filter before it can cut its rows, and on ten years that pass costs seconds. The same filter
//! gives the same result for every page of it, so the first page pays for the pass and the next
//! pages take the result from here.
//!
//! **Why an answer here is never stale.** An answer is kept with a fingerprint of the books and used
//! only while the fingerprint is exactly the same:
//!
//! * the highest id of `analytics_facts`, of `journal_entries` and of `partner_share_rows`. These
//!   tables only grow (PRD FR-6.3, Code Standards B17): a save, an edit, a cancel or a reversal
//!   writes a new row with a higher id, so the highest id changes;
//! * `PRAGMA data_version` of the connection that asks. It changes whenever any other connection
//!   commits (every save of the app, a change of a name or a master list included), so a rename of an
//!   account, which writes none of the tables above, also makes the answer miss. It is a number of
//!   one connection only: two connections can show the same value for different books (every new
//!   connection starts at 2). So the fingerprint also holds a token of the connection that is never
//!   used for another (below), and an answer is used only by the connection that made it;
//! * the generation of [`mandi_db::memo`]. It changes every time a data file is opened or its content
//!   is replaced (a restore), so a restored file can never be given an answer made for the file that
//!   was there before, even if its ids and its `data_version` happen to be the same;
//! * the path of the data file, and the question itself: the command, the full filter with its sort
//!   and the day of today (the shortcuts of a period are worked out from it). The page (offset and
//!   limit) is not part of it, which is the point.
//!
//! The fingerprint is read before the answer is worked out and again after it. An answer is kept
//! only when the two are the same, so one that was made while a save committed is never kept.
//!
//! Only a read connection (opened `query_only`) uses it: the write connection sees lines of a save
//! that is not committed yet. A snapshot of the assistant (`Db::open_snapshot`) is a new connection
//! for each question, so it gets its own token, and what it makes is never used by the screens' read
//! connection, nor the reverse. The memory holds at most
//! [`CAPACITY`] results, newest last, lives only in memory and is never written to the data file or
//! to a backup. It holds no total of the books (PRD FR-6.1): it is a copy of what a read has just
//! worked out, thrown away at the next change.
//!
//! Example: page 1 of view B over ten years groups the 1.2 million facts once (seconds); page 2 reads
//! the four numbers of the fingerprint (microseconds), finds the same fingerprint, and cuts its rows
//! from the kept result.

use std::any::Any;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use mandi_db::rusqlite::Connection;
use mandi_db::rusqlite::functions::FunctionFlags;

use crate::error::ReportsError;

/// The environment variable that turns the memory off (for the speed tests only).
const NO_CACHE_VARIABLE: &str = "MANDI_NO_RESULT_CACHE";

/// The most results kept. The oldest is dropped first.
const CAPACITY: usize = 8;

/// How many times a kept result was used, and how many times one was worked out, since the process
/// started. For tests that check a next page is served from memory.
static HITS: AtomicU64 = AtomicU64::new(0);
static MISSES: AtomicU64 = AtomicU64::new(0);

/// `(hits, misses)` since the process started.
#[doc(hidden)]
pub fn counts() -> (u64, u64) {
    (HITS.load(Ordering::Relaxed), MISSES.load(Ordering::Relaxed))
}

/// The name of the SQL function that holds the token of a connection.
const TOKEN_FUNCTION: &str = "mandi_cache_connection";

/// The next token. A token is given once and never again, so two connections never share one.
static NEXT_TOKEN: AtomicU64 = AtomicU64::new(1);

/// The token of this connection: a number that no other connection has had in this process. It is
/// kept in the connection itself, as the result of a SQL function it is given the first time it asks.
fn connection_token(conn: &Connection) -> Result<i64, ReportsError> {
    if let Ok(token) = conn.query_row(&format!("SELECT {TOKEN_FUNCTION}()"), [], |r| {
        r.get::<_, i64>(0)
    }) {
        return Ok(token);
    }
    let token = i64::try_from(NEXT_TOKEN.fetch_add(1, Ordering::Relaxed)).unwrap_or(i64::MAX);
    conn.create_scalar_function(TOKEN_FUNCTION, 0, FunctionFlags::SQLITE_UTF8, move |_| {
        Ok(token)
    })?;
    Ok(token)
}

/// What the books looked like when an answer was worked out.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Fingerprint {
    /// The connection that made it, and that alone may use it.
    connection: i64,
    /// Changes when a data file is opened or restored.
    generation: u64,
    /// Changes when another connection commits.
    data_version: i64,
    /// The highest id of each table that only grows.
    facts: i64,
    journal: i64,
    shares: i64,
}

struct Kept {
    path: String,
    what: String,
    fingerprint: Fingerprint,
    value: Arc<dyn Any + Send + Sync>,
}

fn store() -> MutexGuard<'static, VecDeque<Kept>> {
    static STORE: OnceLock<Mutex<VecDeque<Kept>>> = OnceLock::new();
    STORE
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// True for a connection opened read only (`PRAGMA query_only`), the way the read connection of
/// [`mandi_db::Db`] is.
fn is_read_connection(conn: &Connection) -> Result<bool, ReportsError> {
    let on: i64 = conn.pragma_query_value(None, "query_only", |row| row.get(0))?;
    Ok(on == 1)
}

fn highest_id(conn: &Connection, table: &str) -> Result<i64, ReportsError> {
    // `table` is one of the constants of this file, never user input.
    Ok(conn.query_row(
        &format!("SELECT COALESCE(MAX(id), 0) FROM {table}"),
        [],
        |r| r.get(0),
    )?)
}

/// The generation of the memory. Tests of this file fix it, because other tests of the same process
/// open data files and so change it.
fn generation() -> u64 {
    #[cfg(test)]
    if let Some(fixed) = tests::FIXED_GENERATION.with(std::cell::Cell::get) {
        return fixed;
    }
    mandi_db::memo::generation()
}

fn fingerprint(conn: &Connection) -> Result<Fingerprint, ReportsError> {
    // The five numbers are read at one moment: inside the transaction of the caller, or in one of
    // their own.
    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        let found = fingerprint_in(&tx);
        // Nothing was written, so ending the transaction is a rollback.
        drop(tx);
        found
    } else {
        fingerprint_in(conn)
    }
}

fn fingerprint_in(conn: &Connection) -> Result<Fingerprint, ReportsError> {
    // The generation is read first: a restore that starts after it changes `data_version` too.
    let generation = generation();
    let data_version: i64 = conn.pragma_query_value(None, "data_version", |row| row.get(0))?;
    Ok(Fingerprint {
        connection: connection_token(conn)?,
        generation,
        data_version,
        facts: highest_id(conn, "analytics_facts")?,
        journal: highest_id(conn, "journal_entries")?,
        shares: highest_id(conn, "partner_share_rows")?,
    })
}

/// The kept result for `what` on this data file, if the books are as they were when it was made.
fn lookup<T: Send + Sync + 'static>(path: &str, what: &str, now: &Fingerprint) -> Option<Arc<T>> {
    let mut store = store();
    let at = store.iter().position(|k| {
        k.path == path && k.what == what && k.fingerprint.connection == now.connection
    })?;
    let kept = store.get(at)?;
    if &kept.fingerprint != now {
        // Out of date: drop it now, it can never be used again.
        store.remove(at);
        return None;
    }
    let value = Arc::clone(&kept.value).downcast::<T>().ok()?;
    // Used: it becomes the newest.
    if let Some(entry) = store.remove(at) {
        store.push_back(entry);
    }
    Some(value)
}

fn keep(path: &str, what: &str, fingerprint: Fingerprint, value: Arc<dyn Any + Send + Sync>) {
    let mut store = store();
    store.retain(|k| {
        !(k.path == path && k.what == what && k.fingerprint.connection == fingerprint.connection)
    });
    store.push_back(Kept {
        path: path.to_owned(),
        what: what.to_owned(),
        fingerprint,
        value,
    });
    while store.len() > CAPACITY {
        store.pop_front();
    }
}

/// Forgets everything kept. For tests that count how often a result is worked out.
#[doc(hidden)]
pub fn forget_all() {
    store().clear();
}

/// The result of `compute` for `what`, from memory when the books are as they were when it was last
/// worked out, else worked out now and kept.
///
/// `what` is the whole question except the page: the command, the full filter with its sort and
/// the day of today. A connection that is not the read connection, or a file without a path (an
/// in-memory database), never uses the memory.
///
/// # Errors
/// What `compute` returns; a database error reading the fingerprint.
pub(crate) fn remembered<T: Send + Sync + 'static>(
    conn: &Connection,
    what: &str,
    compute: impl FnOnce() -> Result<T, ReportsError>,
) -> Result<Arc<T>, ReportsError> {
    let path = conn.path().unwrap_or("");
    // `MANDI_NO_RESULT_CACHE=1` turns the memory off, so a speed test can measure the first page
    // (the whole read) on every run instead of the next page (a copy from memory).
    if path.is_empty()
        || !is_read_connection(conn)?
        || std::env::var_os(NO_CACHE_VARIABLE).is_some()
    {
        return compute().map(Arc::new);
    }
    let before = fingerprint(conn)?;
    if let Some(hit) = lookup::<T>(path, what, &before) {
        HITS.fetch_add(1, Ordering::Relaxed);
        return Ok(hit);
    }
    MISSES.fetch_add(1, Ordering::Relaxed);
    let value = Arc::new(compute()?);
    // A save that committed while the result was worked out may be in it or not: keep it only
    // when the books did not move at all.
    if fingerprint(conn)? == before {
        keep(
            path,
            what,
            before,
            Arc::clone(&value) as Arc<dyn Any + Send + Sync>,
        );
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    thread_local! {
        /// A generation a test fixes for its own thread.
        pub(super) static FIXED_GENERATION: Cell<Option<u64>> = const { Cell::new(None) };
    }

    /// A data file with the three tables the fingerprint reads, a read connection to it (the way
    /// the read connection of `Db` is) and a second connection that writes.
    struct Files {
        /// The tests share one memory, so they run one after the other.
        _turn: std::sync::MutexGuard<'static, ()>,
        _dir: tempfile::TempDir,
        read: Connection,
        write: Connection,
    }

    fn files() -> Files {
        static TURN: Mutex<()> = Mutex::new(());
        let turn = TURN.lock().unwrap_or_else(PoisonError::into_inner);
        forget_all();
        FIXED_GENERATION.with(|g| g.set(Some(1)));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mandi.db");
        let write = Connection::open(&path).unwrap();
        write
            .execute_batch(
                "PRAGMA journal_mode = WAL;
                 CREATE TABLE analytics_facts (id INTEGER PRIMARY KEY, v INTEGER);
                 CREATE TABLE journal_entries (id INTEGER PRIMARY KEY, v INTEGER);
                 CREATE TABLE partner_share_rows (id INTEGER PRIMARY KEY, v INTEGER);
                 CREATE TABLE accounts (id INTEGER PRIMARY KEY, name TEXT);
                 INSERT INTO accounts (id, name) VALUES (1, 'a');",
            )
            .unwrap();
        let read = Connection::open(&path).unwrap();
        read.pragma_update(None, "query_only", true).unwrap();
        Files {
            _turn: turn,
            _dir: dir,
            read,
            write,
        }
    }

    /// Works `what` out, counting how often `compute` really runs.
    fn ask(f: &Files, what: &str, runs: &Cell<u32>) -> i64 {
        *remembered(&f.read, what, || {
            runs.set(runs.get() + 1);
            Ok(i64::from(runs.get()))
        })
        .unwrap()
    }

    #[test]
    fn the_same_question_on_the_same_books_is_worked_out_once() {
        let f = files();
        let runs = Cell::new(0);
        assert_eq!(ask(&f, "q1", &runs), 1);
        assert_eq!(ask(&f, "q1", &runs), 1);
        assert_eq!(ask(&f, "q1", &runs), 1);
        assert_eq!(runs.get(), 1);
        // Another question is another result.
        assert_eq!(ask(&f, "q2", &runs), 2);
        assert_eq!(runs.get(), 2);
    }

    #[test]
    fn a_new_row_in_any_table_the_fingerprint_reads_makes_the_next_ask_work_again() {
        for table in ["analytics_facts", "journal_entries", "partner_share_rows"] {
            let f = files();
            let runs = Cell::new(0);
            assert_eq!(ask(&f, "q", &runs), 1);
            f.write
                .execute(&format!("INSERT INTO {table} (v) VALUES (1)"), [])
                .unwrap();
            assert_eq!(ask(&f, "q", &runs), 2, "{table}");
            // And the new answer is kept again.
            assert_eq!(ask(&f, "q", &runs), 2, "{table}");
        }
    }

    #[test]
    fn a_change_that_writes_none_of_those_tables_still_makes_it_work_again() {
        // A rename writes no journal row, no fact and no share row; the version of the data does move.
        let f = files();
        let runs = Cell::new(0);
        assert_eq!(ask(&f, "q", &runs), 1);
        f.write
            .execute("UPDATE accounts SET name = 'b' WHERE id = 1", [])
            .unwrap();
        assert_eq!(ask(&f, "q", &runs), 2);
    }

    #[test]
    fn a_restore_or_an_open_gives_a_new_generation_and_nothing_is_kept_across_it() {
        let f = files();
        let runs = Cell::new(0);
        assert_eq!(ask(&f, "q", &runs), 1);
        assert_eq!(ask(&f, "q", &runs), 1);
        // What `Db::open` and a restore do: a new generation. The ids and the data version of the file
        // are the same, and still nothing made before is used.
        FIXED_GENERATION.with(|g| g.set(Some(2)));
        assert_eq!(ask(&f, "q", &runs), 2);
        assert_eq!(ask(&f, "q", &runs), 2);
    }

    #[test]
    fn clearing_the_memory_of_the_data_layer_moves_the_generation() {
        // The real generation, which a restore and an open move through `memo::clear`.
        FIXED_GENERATION.with(|g| g.set(None));
        let before = mandi_db::memo::generation();
        mandi_db::memo::clear();
        assert!(mandi_db::memo::generation() > before);
    }

    #[test]
    fn a_result_worked_out_while_the_books_moved_is_not_kept() {
        let f = files();
        let runs = Cell::new(0);
        // The first ask is overtaken by a save while it works.
        let value = *remembered(&f.read, "q", || {
            runs.set(runs.get() + 1);
            f.write
                .execute("INSERT INTO analytics_facts (v) VALUES (1)", [])
                .unwrap();
            Ok(10_i64)
        })
        .unwrap();
        assert_eq!(value, 10);
        // It was not kept: the next ask works again, and that one is kept.
        assert_eq!(ask(&f, "q", &runs), 2);
        assert_eq!(ask(&f, "q", &runs), 2);
        assert_eq!(runs.get(), 2);
    }

    #[test]
    fn two_connections_to_the_same_file_never_share_a_result() {
        // Every new connection starts its data version at the same number, so equal numbers say
        // nothing across connections: a result made through one is never used through another, even
        // when the ids and the versions are all equal (a snapshot of the assistant is a new
        // connection for each question).
        let f = files();
        let other = Connection::open(f.read.path().unwrap()).unwrap();
        other.pragma_update(None, "query_only", true).unwrap();
        let runs = Cell::new(0);
        assert_eq!(ask(&f, "q", &runs), 1);
        let through_other = *remembered(&other, "q", || {
            runs.set(runs.get() + 1);
            Ok(i64::from(runs.get()))
        })
        .unwrap();
        assert_eq!(through_other, 2);
        // Each keeps its own.
        assert_eq!(ask(&f, "q", &runs), 1);
        let again = *remembered(&other, "q", || {
            runs.set(runs.get() + 1);
            Ok(i64::from(runs.get()))
        })
        .unwrap();
        assert_eq!(again, 2);
        assert_eq!(runs.get(), 2);
    }

    #[test]
    fn the_write_connection_never_uses_the_memory() {
        let f = files();
        let runs = Cell::new(0);
        for _ in 0..3 {
            let value = *remembered(&f.write, "q", || {
                runs.set(runs.get() + 1);
                Ok(1_i64)
            })
            .unwrap();
            assert_eq!(value, 1);
        }
        assert_eq!(runs.get(), 3);
    }

    #[test]
    fn at_most_eight_results_are_kept_and_the_oldest_goes_first() {
        let f = files();
        let runs = Cell::new(0);
        for n in 0..CAPACITY {
            ask(&f, &format!("q{n}"), &runs);
        }
        // All eight are still there.
        let before = runs.get();
        for n in 0..CAPACITY {
            ask(&f, &format!("q{n}"), &runs);
        }
        assert_eq!(runs.get(), before);
        // A ninth pushes the oldest out.
        ask(&f, "q-new", &runs);
        let before = runs.get();
        ask(&f, "q0", &runs);
        assert_eq!(runs.get(), before + 1);
    }
}
