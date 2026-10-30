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

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use rusqlite::Connection;

/// The most answers kept. The oldest are dropped first.
const CAPACITY: usize = 64;

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
}

fn store() -> MutexGuard<'static, Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE
        .get_or_init(|| {
            Mutex::new(Store {
                answers: HashMap::new(),
                order: Vec::new(),
            })
        })
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// Forgets every answer. Called when a data file is opened.
pub fn clear() {
    let mut store = store();
    store.answers.clear();
    store.order.clear();
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

/// The answer for `what` on this data file if the journal has not grown since it was made;
/// otherwise `compute()` is run and its answer kept.
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
    let now = version(conn)?;
    if let Some((then, answer)) = store().answers.get(&key)
        && *then == now
    {
        return Ok(*answer);
    }
    let answer = compute()?;
    let mut store = store();
    if !store.answers.contains_key(&key) {
        if store.order.len() >= CAPACITY && !store.order.is_empty() {
            let oldest = store.order.remove(0);
            store.answers.remove(&oldest);
        }
        store.order.push(key.clone());
    }
    store.answers.insert(key, (now, answer));
    Ok(answer)
}
