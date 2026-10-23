//! The database handle: one write connection, one read connection, one transaction per
//! user action (Code Standards B3, B13, B21, plan 03 Sections 4.1 to 4.3).
//!
//! - Every connection: WAL, `foreign_keys = ON`, `synchronous = FULL`, `recursive_triggers = ON`
//!   (so `INSERT OR REPLACE` cannot delete a row without the append-only triggers firing),
//!   and a busy timeout.
//! - [`Db::write`] checks free disk space, waits for the start-up check, begins an
//!   `IMMEDIATE` transaction and commits only if the closure returns `Ok`.
//! - [`Db::read`] uses the second connection, so reads never wait for a save.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};

use crate::audit::{Auditor, KeyStore, KeyringKeyStore};
use crate::clock::{Clock, SystemClock};
use crate::disk::{DiskProbe, MIN_FREE_DISK_BYTES, RealDiskProbe};
use crate::error::DbError;
use crate::migrate::{self, MigrationOutcome};

/// How long SQLite waits for another process that holds the file (B13).
pub const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
/// How long a save waits for the start-up check before it gives up with `ERR_DB_BUSY`.
pub const CHECK_WAIT: Duration = Duration::from_secs(30);

/// Where the start-up check of the data file stands (NFR-REL-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataState {
    /// The check is still running. Saving waits.
    Checking,
    /// The check passed.
    Ok,
    /// The check found damage. Every save returns `ERR_DATA_CHECK_FAILED`.
    Failed,
}

/// The parts of a [`Db`] that tests replace.
#[derive(Clone)]
pub struct DbOptions {
    /// Where the copy taken before a migration goes.
    pub backups_dir: PathBuf,
    /// Free disk space check.
    pub disk_probe: Arc<dyn DiskProbe>,
    /// Time for audit rows and copy names.
    pub clock: Arc<dyn Clock>,
    /// Where the audit chain key lives.
    pub key_store: Arc<dyn KeyStore>,
    /// SQLite busy timeout.
    pub busy_timeout: Duration,
    /// How long a save waits for the start-up check.
    pub check_wait: Duration,
}

impl DbOptions {
    /// The real clock, disk probe and key store, with copies in `backups_dir`.
    pub fn new(backups_dir: impl Into<PathBuf>) -> Self {
        Self {
            backups_dir: backups_dir.into(),
            disk_probe: Arc::new(RealDiskProbe),
            clock: Arc::new(SystemClock),
            key_store: Arc::new(KeyringKeyStore),
            busy_timeout: BUSY_TIMEOUT,
            check_wait: CHECK_WAIT,
        }
    }
}

struct Health {
    state: Mutex<(DataState, Option<String>)>,
    changed: Condvar,
}

/// The open data file.
pub struct Db {
    path: PathBuf,
    writer: Mutex<Connection>,
    reader: Mutex<Connection>,
    health: Health,
    disk_probe: Arc<dyn DiskProbe>,
    auditor: Auditor,
    clock: Arc<dyn Clock>,
    migration: MigrationOutcome,
    busy_timeout: Duration,
    check_wait: Duration,
}

impl std::fmt::Debug for Db {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Db")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // A panic while holding the lock leaves the connection usable: an open transaction
    // rolls back when it is dropped.
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Opens a connection and sets the pragmas of B13. The first statement that touches the
/// file fails with `DataCheckFailed` if the file is not a database.
fn open_connection(
    path: &Path,
    busy_timeout: Duration,
    read_only: bool,
) -> Result<Connection, DbError> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    conn.busy_timeout(busy_timeout)?;
    if !read_only {
        let mode: String =
            conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get(0))?;
        if !mode.eq_ignore_ascii_case("wal") {
            return Err(DbError::DataCheckFailed(format!(
                "cannot use WAL mode, got {mode}"
            )));
        }
    }
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "recursive_triggers", "ON")?;
    conn.pragma_update(None, "synchronous", "FULL")?;
    if read_only {
        conn.pragma_update(None, "query_only", "ON")?;
    }
    Ok(conn)
}

impl Db {
    /// Opens the data file, creates it if needed, and brings it to the latest schema
    /// (with a copy first, see `migrate`). The start-up check has NOT run yet: the state is
    /// `Checking` and saves wait. Call [`Db::spawn_quick_check`] or [`Db::run_quick_check`].
    ///
    /// # Errors
    /// `DataCheckFailed` when the file is not a database; `MigrationFailed`; or an I/O error.
    pub fn open(path: &Path, options: DbOptions) -> Result<Db, DbError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut writer = open_connection(path, options.busy_timeout, false)?;
        let migration =
            migrate::migrate(&mut writer, &options.backups_dir, options.clock.as_ref())?;
        let reader = open_connection(path, options.busy_timeout, true)?;
        Ok(Db {
            path: path.to_owned(),
            writer: Mutex::new(writer),
            reader: Mutex::new(reader),
            health: Health {
                state: Mutex::new((DataState::Checking, None)),
                changed: Condvar::new(),
            },
            disk_probe: options.disk_probe,
            auditor: Auditor::new(options.key_store, Arc::clone(&options.clock)),
            clock: options.clock,
            migration,
            busy_timeout: options.busy_timeout,
            check_wait: options.check_wait,
        })
    }

    /// Opens a data file for READING ONLY: a snapshot taken by the backup, for the readable pack
    /// (PRD FR-20.5). Nothing is created, upgraded or written: both connections are read-only,
    /// every save fails, and the journal mode is left as it is. The file must already have this
    /// app's schema (a snapshot made by this version does).
    ///
    /// # Errors
    /// `DataCheckFailed` when the file is not a database; `MigrationFailed` when its schema is
    /// not this app's; an I/O error.
    pub fn open_read_only(path: &Path, options: DbOptions) -> Result<Db, DbError> {
        let open = || -> Result<Connection, DbError> {
            let conn = Connection::open_with_flags(
                path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )?;
            conn.busy_timeout(options.busy_timeout)?;
            conn.pragma_update(None, "foreign_keys", "ON")?;
            conn.pragma_update(None, "query_only", "ON")?;
            Ok(conn)
        };
        let reader = open()?;
        let version = migrate::ensure_current(&reader)?;
        let writer = open()?;
        Ok(Db {
            path: path.to_owned(),
            writer: Mutex::new(writer),
            reader: Mutex::new(reader),
            health: Health {
                state: Mutex::new((DataState::Ok, None)),
                changed: Condvar::new(),
            },
            disk_probe: options.disk_probe,
            auditor: Auditor::new(options.key_store, Arc::clone(&options.clock)),
            clock: options.clock,
            migration: MigrationOutcome {
                from: version,
                to: version,
                copy: None,
            },
            busy_timeout: options.busy_timeout,
            check_wait: options.check_wait,
        })
    }

    /// [`Db::open`] followed by the start-up check on this thread. For tools and tests.
    ///
    /// # Errors
    /// Same as [`Db::open`].
    pub fn open_checked(path: &Path, options: DbOptions) -> Result<Db, DbError> {
        let db = Db::open(path, options)?;
        db.run_quick_check();
        Ok(db)
    }

    /// The path of the data file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// What the migration did when the file was opened.
    pub fn migration(&self) -> &MigrationOutcome {
        &self.migration
    }

    /// The clock of this database: audit rows and the services use the same one.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// The audit writer (plan 03 Section 4.5).
    pub fn auditor(&self) -> &Auditor {
        &self.auditor
    }

    /// Where the start-up check stands.
    pub fn data_state(&self) -> DataState {
        lock(&self.health.state).0
    }

    /// What the failed check found, for the log. `None` unless the state is `Failed`.
    pub fn failure_detail(&self) -> Option<String> {
        lock(&self.health.state).1.clone()
    }

    fn set_state(&self, state: DataState, detail: Option<String>) {
        *lock(&self.health.state) = (state, detail);
        self.health.changed.notify_all();
    }

    /// Runs `PRAGMA quick_check` on its own connection (reads and the check do not block each
    /// other), sets the state and returns it (NFR-REL-7).
    pub fn run_quick_check(&self) -> DataState {
        match self.quick_check() {
            Ok(()) => {
                self.set_state(DataState::Ok, None);
                DataState::Ok
            }
            Err(detail) => {
                tracing::error!(detail = %detail, "start-up data check failed");
                self.set_state(DataState::Failed, Some(detail));
                DataState::Failed
            }
        }
    }

    fn quick_check(&self) -> Result<(), String> {
        let conn =
            open_connection(&self.path, self.busy_timeout, true).map_err(|e| e.to_string())?;
        let mut statement = conn
            .prepare("PRAGMA quick_check")
            .map_err(|e| e.to_string())?;
        let lines: Vec<String> = statement
            .query_map([], |row| row.get::<_, String>(0))
            .and_then(|rows| rows.collect())
            .map_err(|e| e.to_string())?;
        if lines.len() == 1 && lines.first().is_some_and(|line| line == "ok") {
            Ok(())
        } else {
            Err(lines.into_iter().take(5).collect::<Vec<_>>().join("; "))
        }
    }

    /// Runs the start-up check on a background thread, so the login screen is not delayed
    /// (NFR-REL-7). If no thread can be started it runs on the caller's thread.
    pub fn spawn_quick_check(self: &Arc<Self>) -> Option<JoinHandle<DataState>> {
        let db = Arc::clone(self);
        let spawned = std::thread::Builder::new()
            .name("mandi-quick-check".to_owned())
            .spawn(move || db.run_quick_check());
        match spawned {
            Ok(handle) => Some(handle),
            Err(error) => {
                tracing::warn!("cannot start the check thread, running it now: {error}");
                self.run_quick_check();
                None
            }
        }
    }

    /// Waits for the start-up check, then says whether saving is allowed.
    fn wait_until_writable(&self) -> Result<(), DbError> {
        let guard = lock(&self.health.state);
        let (guard, _timeout) = self
            .health
            .changed
            .wait_timeout_while(guard, self.check_wait, |(state, _)| {
                *state == DataState::Checking
            })
            .unwrap_or_else(PoisonError::into_inner);
        match guard.0 {
            DataState::Ok => Ok(()),
            DataState::Failed => Err(DbError::DataCheckFailed(
                guard.1.clone().unwrap_or_default(),
            )),
            DataState::Checking => Err(DbError::Busy),
        }
    }

    /// Refuses a save when the disk is nearly full (NFR-REL-10). If the probe itself fails,
    /// the save goes on: SQLite reports a full disk too and rolls back.
    fn check_disk(&self) -> Result<(), DbError> {
        let dir = self.path.parent().unwrap_or(Path::new("."));
        match self.disk_probe.available_bytes(dir) {
            Ok(available) if available < MIN_FREE_DISK_BYTES => Err(DbError::DiskFull {
                available_bytes: available,
                needed_bytes: MIN_FREE_DISK_BYTES,
            }),
            Ok(_) => Ok(()),
            Err(error) => {
                tracing::warn!("cannot read free disk space: {error}");
                Ok(())
            }
        }
    }

    /// One user action = one call of this function = one transaction (CLAUDE.md rule 3).
    /// Checks the start-up state and the free disk space, begins an `IMMEDIATE` transaction,
    /// runs `action`, and commits only if it returns `Ok`. On `Err` everything is rolled back.
    ///
    /// # Errors
    /// `ERR_DATA_CHECK_FAILED`, `ERR_DISK_FULL`, `ERR_DB_BUSY` before anything is written,
    /// or whatever `action` returns.
    pub fn write<T, E>(&self, action: impl FnOnce(&Transaction<'_>) -> Result<T, E>) -> Result<T, E>
    where
        E: From<DbError>,
    {
        self.wait_until_writable()?;
        self.check_disk()?;
        let mut conn = lock(&self.writer);
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(DbError::from)?;
        let value = action(&tx)?;
        tx.commit().map_err(DbError::from)?;
        Ok(value)
    }

    /// Runs a read on the second connection. It sees every committed save.
    ///
    /// # Errors
    /// Whatever `action` returns.
    pub fn read<T, E>(&self, action: impl FnOnce(&Connection) -> Result<T, E>) -> Result<T, E> {
        let conn = lock(&self.reader);
        action(&conn)
    }

    /// Replaces the whole content of the data file with the content of `source`, a complete and
    /// already checked SQLite file, while the app keeps its connections (PRD FR-20.9, plan 18).
    ///
    /// The copy is the SQLite online backup API run in one step into the write connection, so
    /// it is all or nothing: if it fails, the data file is as it was. Both connections are held
    /// for the time of the copy, so no save and no read sees half of it. The data file stays in
    /// WAL mode. The caller checks and upgrades `source` first, and throws away what the
    /// services keep in memory afterwards (`Context::invalidate_caches`).
    ///
    /// # Errors
    /// `DataCheckFailed` when `source` is not a database or the file fails its check after the
    /// copy; `Busy`; an I/O error; any other SQLite error.
    pub fn replace_contents_from(&self, source: &Path) -> Result<(), DbError> {
        let from = Connection::open_with_flags(
            source,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        {
            let mut writer = lock(&self.writer);
            let _reader = lock(&self.reader);
            let copy = rusqlite::backup::Backup::new(&from, &mut writer)?;
            // One step with every page (-1): the data file changes in one transaction. A step
            // that finds a lock is tried again a moment later.
            let mut tries = 0_u32;
            loop {
                match copy.step(-1)? {
                    rusqlite::backup::StepResult::Done => break,
                    rusqlite::backup::StepResult::More => {}
                    _ => {
                        tries += 1;
                        if tries > 400 {
                            return Err(DbError::Busy);
                        }
                        std::thread::sleep(Duration::from_millis(25));
                    }
                }
            }
        }
        match self.run_quick_check() {
            DataState::Ok => Ok(()),
            _ => Err(DbError::DataCheckFailed(
                self.failure_detail().unwrap_or_default(),
            )),
        }
    }

    /// The data version of the file as the read connection sees it. It changes whenever the
    /// write connection (or another process) commits, so a scheduler can tell that the data
    /// changed without reading any table.
    ///
    /// # Errors
    /// Any SQLite error.
    pub fn data_version(&self) -> Result<i64, DbError> {
        let conn = lock(&self.reader);
        Ok(conn.query_row("PRAGMA data_version", [], |row| row.get(0))?)
    }
}
