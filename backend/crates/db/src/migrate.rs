//! Numbered migrations with a full copy first (plan 03 Section 4.1, NFR-REL-13, NFR-SCL-8).
//!
//! `0001_init.sql` holds the whole design of Architecture Section 15. A released migration is
//! never edited: a change is a new numbered file added to [`migrations`] (Code Standards B19).
//!
//! Before an existing data file with pending migrations is changed, [`migrate_with`] copies
//! it with the SQLite online backup API to
//! `<backups_dir>/pre-migration-<from>-<to>-<utc>.db` and checks the copy. If the upgrade
//! fails, or the file fails its checks afterwards, the copy is put back and the error is
//! `ERR_MIGRATION_FAILED`. A new, empty file needs no copy.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::backup::Backup;
use rusqlite::{Connection, OpenFlags};
use rusqlite_migration::{HookError, M, Migrations, SchemaVersion};

use crate::clock::Clock;
use crate::error::DbError;

/// The first migration: every table, index and trigger of Architecture Section 15.
pub const INIT_SQL: &str = include_str!("../migrations/0001_init.sql");

/// Pages copied in one step of the backup. The pause is zero: the copy is short.
const BACKUP_PAGES_PER_STEP: i32 = 512;

/// All migrations, in order. Migration 1 also seeds the system accounts, in its own
/// transaction (plan 03 Section 4.4).
pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up_with_hook(INIT_SQL, |tx| {
        crate::seed::system_accounts(tx)
            .map(|_| ())
            .map_err(HookError::from)
    })])
}

/// What a call to [`migrate`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationOutcome {
    /// Schema version before (0 for an empty file).
    pub from: usize,
    /// Schema version after.
    pub to: usize,
    /// The copy taken before the upgrade. `None` for an empty file or when nothing was pending.
    pub copy: Option<PathBuf>,
}

/// Brings the data file to the latest schema. See the module comment.
///
/// # Errors
/// `DbError::MigrationFailed` when the file is newer than this app, when the copy cannot be
/// made or fails its check, or when the upgrade fails (after the copy was put back).
pub fn migrate(
    conn: &mut Connection,
    backups_dir: &Path,
    clock: &dyn Clock,
) -> Result<MigrationOutcome, DbError> {
    migrate_with(&migrations(), conn, backups_dir, clock)
}

/// Same as [`migrate`] with a given list of migrations. Tests use it to inject a bad one.
///
/// # Errors
/// See [`migrate`].
pub fn migrate_with(
    migrations: &Migrations<'_>,
    conn: &mut Connection,
    backups_dir: &Path,
    clock: &dyn Clock,
) -> Result<MigrationOutcome, DbError> {
    let failed = |what: &str, error: &dyn std::fmt::Display| {
        DbError::MigrationFailed(format!("{what}: {error}"))
    };
    let from = match migrations
        .current_version(conn)
        .map_err(|e| failed("cannot read the schema version", &e))?
    {
        SchemaVersion::NoneSet => 0,
        SchemaVersion::Inside(v) => v.get(),
        SchemaVersion::Outside(v) => {
            return Err(DbError::MigrationFailed(format!(
                "the data file has schema version {v}, newer than this app"
            )));
        }
    };
    let pending = migrations
        .pending_migrations(conn)
        .map_err(|e| failed("cannot count pending migrations", &e))?;
    let to = from.saturating_add(usize::try_from(pending).unwrap_or(0));
    if to == from {
        return Ok(MigrationOutcome {
            from,
            to,
            copy: None,
        });
    }

    // An empty file has nothing to lose. An existing one is copied first.
    let copy = if from > 0 {
        let stamp = clock.now_utc().format("%Y%m%dT%H%M%SZ");
        let path = backups_dir.join(format!("pre-migration-{from}-{to}-{stamp}.db"));
        make_checked_copy(conn, &path, from)?;
        tracing::info!(from, to, copy = %path.display(), "copied the data file before migrating");
        Some(path)
    } else {
        None
    };

    let result = migrations
        .to_latest(conn)
        .map_err(|e| failed("the upgrade failed", &e))
        .and_then(|()| check_after_upgrade(conn));
    match result {
        Ok(()) => Ok(MigrationOutcome { from, to, copy }),
        Err(error) => {
            if let Some(copy_path) = &copy
                && let Err(restore_error) = restore_copy(copy_path, conn, from)
            {
                return Err(DbError::MigrationFailed(format!(
                    "{error}; putting back the copy also failed: {restore_error}"
                )));
            }
            tracing::warn!(from, to, "migration failed, data put back: {error}");
            Err(error)
        }
    }
}

/// Foreign keys and the page structure must be sound after an upgrade.
fn check_after_upgrade(conn: &Connection) -> Result<(), DbError> {
    let bad_keys: i64 =
        conn.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })?;
    if bad_keys > 0 {
        return Err(DbError::MigrationFailed(format!(
            "{bad_keys} foreign key problems after the upgrade"
        )));
    }
    let quick: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if quick != "ok" {
        return Err(DbError::MigrationFailed(format!(
            "quick check after the upgrade: {quick}"
        )));
    }
    Ok(())
}

/// Copies the whole database with the online backup API, then opens the copy and checks it.
fn make_checked_copy(
    conn: &Connection,
    path: &Path,
    expected_version: usize,
) -> Result<(), DbError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut copy = Connection::open(path)?;
    {
        let backup = Backup::new(conn, &mut copy)?;
        backup.run_to_completion(BACKUP_PAGES_PER_STEP, Duration::ZERO, None)?;
    }
    check_copy(&copy, expected_version)
        .map_err(|e| DbError::MigrationFailed(format!("the copy failed its check: {e}")))
}

/// A copy is good when SQLite finds no damage and it holds the version we copied.
fn check_copy(copy: &Connection, expected_version: usize) -> Result<(), String> {
    let integrity: String = copy
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if integrity != "ok" {
        return Err(format!("integrity check says: {integrity}"));
    }
    let version: i64 = copy
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if usize::try_from(version).ok() != Some(expected_version) {
        return Err(format!(
            "schema version {version}, expected {expected_version}"
        ));
    }
    Ok(())
}

/// Puts a copy back over the live database, page by page, and checks the result.
///
/// # Errors
/// `DbError::MigrationFailed` when the copy cannot be read or the restored file fails the check.
pub fn restore_copy(
    copy_path: &Path,
    conn: &mut Connection,
    expected_version: usize,
) -> Result<(), DbError> {
    let source = Connection::open_with_flags(copy_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    {
        let backup = Backup::new(&source, conn)?;
        backup.run_to_completion(BACKUP_PAGES_PER_STEP, Duration::ZERO, None)?;
    }
    check_copy(conn, expected_version)
        .map_err(|e| DbError::MigrationFailed(format!("the restored file failed its check: {e}")))
}
