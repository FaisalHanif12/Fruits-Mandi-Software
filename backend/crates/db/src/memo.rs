//! A small memory of the sums of one account over a range (plan 38 B2, NFR-SPD-24).
//!
//! The totals of a ledger or of the cash book are the same for every page of it, and on 10 years of
//! data they cost a pass over hundreds of thousands of journal lines. This keeps the last answers,
//! so the first page pays for the pass and each next page does not.
//!
//! It is correct because the journal only grows: a line or an entry is never changed after it is
//! posted and never deleted (PRD FR-6.3, Code Standards B17). The sums of any range can change
//! only when a new line is written, and every new line has a higher id than every older one. So an
//! answer is kept together with the highest line id at the time, and used only while that id is
//! still the highest. The memory is emptied every time a data file is opened (a restore opens it
//! again), and it holds at most [`CAPACITY`] answers.
//!
//! Example: page 1 of the cash book reads the 514,000 cash lines once (0.4 s); page 2 reads the
//! highest line id (instant) and takes the answer from here.
//!
//! The same memory keeps the totals of every account of some controls up to a day
//! ([`account_rows`]): the balance of every buyer at the start of a day, which every page of the
//! collection sheet needs and which does not change between its pages (plan 76, NFR-SPD-24).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use rusqlite::Connection;

/// The most answers kept. The oldest are dropped first.
const CAPACITY: usize = 64;

/// The most per-account answers kept (each holds a row for every account asked, a few hundred
/// kilobytes at 20,000 accounts).
const ROWS_CAPACITY: usize = 8;

/// For each account: its id, Σ debits and Σ credits, paisa.
pub type AccountRows = Arc<Vec<(i64, i64, i64)>>;

/// Count and two sums of a range of journal lines: the shape every ledger total has.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Sums {
    /// How many lines, or entries, the range holds.
    pub count: i64,
    /// Σ debits, paisa.
    pub dr_paisa: i64,
    /// Σ credits, paisa.
    pub cr_paisa: i64,
}

type Key = (String, String);

struct Store {
    /// (data file, what was asked) -> (highest line id at the time, answer)
    answers: HashMap<Key, (i64, Sums)>,
    /// The order the keys were added, oldest first.
    order: Vec<Key>,
    /// The same for the answers that hold a row for each account.
    rows: HashMap<Key, (i64, AccountRows)>,
    /// The order the keys of `rows` were added, oldest first.
    rows_order: Vec<Key>,
}

fn store() -> MutexGuard<'static, Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE
        .get_or_init(|| {
            Mutex::new(Store {
                answers: HashMap::new(),
                order: Vec::new(),
                rows: HashMap::new(),
                rows_order: Vec::new(),
            })
        })
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// Changes every time the memory is cleared: every time a data file is opened or its content is
/// replaced (a restore). A result kept elsewhere (`mandi_reports::result_cache`) uses it as part of
/// its key, so a restored file can never be given an answer made for the file before it.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// The generation of the memory (see [`GENERATION`]).
pub fn generation() -> u64 {
    GENERATION.load(Ordering::Acquire)
}

/// Forgets every answer. Called when a data file is opened.
pub fn clear() {
    // The generation moves first, so a result made for the old file is never matched afterwards.
    GENERATION.fetch_add(1, Ordering::AcqRel);
    let mut store = store();
    store.answers.clear();
    store.order.clear();
    store.rows.clear();
    store.rows_order.clear();
}

/// True for a connection opened read only (`PRAGMA query_only`), the way the read connection of
/// [`crate::Db`] is.
fn is_read_connection(conn: &Connection) -> rusqlite::Result<bool> {
    let on: i64 = conn.pragma_query_value(None, "query_only", |row| row.get(0))?;
    Ok(on == 1)
}

/// The highest id among all journal lines: the version of the journal.
fn version(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COALESCE(MAX(id), 0) FROM journal_lines", [], |r| {
        r.get(0)
    })
}

/// Runs `f` in one read transaction when the connection is in autocommit, so the version, the
/// answer and the check after it all see the same moment (the same as `mandi_reports::result_cache`
/// does for its fingerprint). In a transaction already, `f` just runs.
fn in_one_view<T>(
    conn: &Connection,
    f: impl FnOnce() -> rusqlite::Result<T>,
) -> rusqlite::Result<T> {
    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        let result = f();
        // Nothing was written, so ending the transaction is a rollback.
        drop(tx);
        result
    } else {
        f()
    }
}

/// True while an entry waits as a draft. Posting a draft writes no journal line, so the version
/// below would not move: nothing is kept while a draft exists (it never does outside a save, B16).
fn draft_exists(conn: &Connection) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM journal_entries WHERE status <> 'posted')",
        [],
        |r| r.get(0),
    )
}

/// The answer for `what` on this data file if the journal has not grown since it was made;
/// otherwise `compute()` is run and its answer kept. The version, the answer and the check after it
/// are one read transaction, and nothing is kept while a draft entry exists.
///
/// # Errors
/// Any SQLite error from reading the version or from `compute`.
pub fn sums(
    conn: &Connection,
    what: &str,
    compute: impl FnOnce() -> rusqlite::Result<Sums>,
) -> rusqlite::Result<Sums> {
    // Only the read connection may use the memory. The write connection sees lines of a save
    // that is not committed yet (drafts that are posted later, or a rollback that gives the same
    // ids to other lines), so its answers must never be kept. A file without a path (an
    // in-memory database) has no key that tells two of them apart.
    let path = conn.path().unwrap_or("");
    if path.is_empty() || !is_read_connection(conn)? {
        return compute();
    }
    let key: Key = (path.to_owned(), what.to_owned());
    in_one_view(conn, || {
        if draft_exists(conn)? {
            return compute();
        }
        let now = version(conn)?;
        if let Some((then, answer)) = store().answers.get(&key)
            && *then == now
        {
            return Ok(*answer);
        }
        let answer = compute()?;
        // A save that commits while the answer is worked out is not inside this view, but the check
        // is kept: an answer is stored only for the version it was made at.
        if version(conn)? != now {
            return Ok(answer);
        }
        let mut store = store();
        if !store.answers.contains_key(&key) {
            if store.order.len() >= CAPACITY && !store.order.is_empty() {
                let oldest = store.order.remove(0);
                store.answers.remove(&oldest);
            }
            store.order.push(key.clone());
        }
        store.answers.insert(key.clone(), (now, answer));
        Ok(answer)
    })
}

/// As [`sums`], for an answer that has a row for each account (see [`AccountRows`]). The same
/// rule keeps it correct: it is used only while no journal line has been written since, only on the
/// read connection and never while a draft entry exists. The rows are shared, not copied.
///
/// # Errors
/// Any SQLite error from reading the version or from `compute`.
pub fn account_rows(
    conn: &Connection,
    what: &str,
    compute: impl FnOnce() -> rusqlite::Result<Vec<(i64, i64, i64)>>,
) -> rusqlite::Result<AccountRows> {
    let path = conn.path().unwrap_or("");
    if path.is_empty() || !is_read_connection(conn)? {
        return compute().map(Arc::new);
    }
    let key: Key = (path.to_owned(), what.to_owned());
    in_one_view(conn, || {
        if draft_exists(conn)? {
            return compute().map(Arc::new);
        }
        let now = version(conn)?;
        if let Some((then, answer)) = store().rows.get(&key)
            && *then == now
        {
            return Ok(Arc::clone(answer));
        }
        let answer: AccountRows = Arc::new(compute()?);
        // A save that commits while the rows are worked out may be inside them. Then the answer is
        // given but not kept, so it can never be taken for the answer of an older moment.
        if version(conn)? != now {
            return Ok(answer);
        }
        let mut store = store();
        if !store.rows.contains_key(&key) {
            if store.rows_order.len() >= ROWS_CAPACITY && !store.rows_order.is_empty() {
                let oldest = store.rows_order.remove(0);
                store.rows.remove(&oldest);
            }
            store.rows_order.push(key.clone());
        }
        store.rows.insert(key.clone(), (now, Arc::clone(&answer)));
        Ok(answer)
    })
}
