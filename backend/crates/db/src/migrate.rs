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
use rusqlite::{Connection, OpenFlags, Transaction};
use rusqlite_migration::{HookError, M, Migrations, SchemaVersion};

use crate::clock::Clock;
use crate::error::DbError;

/// The first migration: every table, index and trigger of Architecture Section 15.
pub const INIT_SQL: &str = include_str!("../migrations/0001_init.sql");

/// The second migration: indexes so that balances read the index only. No table, no stored
/// total (PRD FR-6.1).
pub const BALANCE_INDEXES_SQL: &str = include_str!("../migrations/0002_balance_indexes.sql");

/// The third migration: a trigger that stops a system account being made inactive (PRD FR-10.4).
/// No table, column or index changes and no row is touched.
pub const SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL: &str =
    include_str!("../migrations/0003_system_accounts_stay_active.sql");

/// The fourth migration: triggers that stop a بکری, a lot or a sale line being deleted and a
/// بکری changing its number (plan 13). No table, column or index changes and no row is touched.
pub const BIKRI_RECORDS_ARE_NEVER_DELETED_SQL: &str =
    include_str!("../migrations/0004_bikri_records_are_never_deleted.sql");

/// The fifth migration: a stable order key on journal entries, so an edited posting keeps its
/// place in its day for matching, and a link from a reversal stock move to the move it reverses
/// (plan 16). Two nullable columns and one index: no row is touched.
pub const STABLE_POSTING_ORDER_SQL: &str =
    include_str!("../migrations/0005_stable_posting_order.sql");

/// The sixth migration (plan 16b, PRD v1.7): the tables of the market trips, the gardens and the
/// fruit taken, the trip goods columns, the 0604 account of each partner, the tag column, and the
/// wider checks and system accounts that trips, partner capital and the separate khata need. See
/// the header of the file and [`trips_hook`]. No row is changed and no number moves.
pub const TRIPS_GARDENS_FRUIT_KHATA_SQL: &str =
    include_str!("../migrations/0006_trips_gardens_fruit_khata.sql");

/// The seventh migration (plan 32, PRD 1.8): simple codes. Sellers, buyers, the two پختہ کھاتہ accounts,
/// items, packings and staff are renumbered to the type number plus a running number from 0
/// (`0010`, `0020`, `0030`, `0040`, `0050`, `0060`), staff get a `code` column, and the upper limit of
/// `accounts.serial` is lifted by [`codes_hook`]. No amount, journal line or id changes.
pub const SIMPLE_PARTY_CODES_SQL: &str = include_str!("../migrations/0007_simple_party_codes.sql");

/// The eighth migration (plan 34, PRD 1.8): easy codes for market trips (type 007) and gardens
/// (type 008). The account of each garden gets the code of its garden, and the Rust hook
/// [`trip_names_hook`] gives each trip its new name. No amount, id or chain hash changes.
pub const TRIP_AND_GARDEN_CODES_SQL: &str =
    include_str!("../migrations/0008_trip_and_garden_codes.sql");

/// Pages copied in one step of the backup. The pause is zero: the copy is short.
const BACKUP_PAGES_PER_STEP: i32 = 512;

/// All migrations, in order. Migration 1 also seeds the system accounts, in its own
/// transaction (plan 03 Section 4.4).
pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up_with_hook(INIT_SQL, |tx| {
            crate::seed::system_accounts(tx).map_err(HookError::from)?;
            crate::seed::default_rules(tx)
                .map(|_| ())
                .map_err(HookError::from)
        }),
        M::up(BALANCE_INDEXES_SQL),
        M::up(SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL),
        M::up(BIKRI_RECORDS_ARE_NEVER_DELETED_SQL),
        M::up(STABLE_POSTING_ORDER_SQL),
        M::up_with_hook(TRIPS_GARDENS_FRUIT_KHATA_SQL, trips_hook),
        M::up_with_hook(SIMPLE_PARTY_CODES_SQL, codes_hook),
        M::up_with_hook(TRIP_AND_GARDEN_CODES_SQL, trip_names_hook),
    ])
}

/// One edit of a table's text in `sqlite_schema`: `from` must occur exactly once in it.
struct Widen {
    table: &'static str,
    from: &'static str,
    to: &'static str,
}

/// The CHECK lists that migration 6 widens (Architecture 15.4, 15.6 and 15.7 v1.7). Each `from` is
/// the text of migrations 1 and 5 as released.
const WIDEN: [Widen; 7] = [
    // Posting types of purchases, trip costs, garden payments, sales in another shop, a garden's
    // closing, fruit taken, partner capital and the separate khata.
    Widen {
        table: "journal_entries",
        from: "'general', 'opening', 'month_close')",
        to: "'general', 'opening', 'month_close', 'purchase', 'trip_cost', 'garden_payment', 'venture_receipt', 'garden_close', 'fruit_taken', 'capital_in', 'capital_out', 'khata_out', 'khata_in')",
    },
    Widen {
        table: "journal_entries",
        from: "source_type IN ('bikri', 'voucher', 'account', 'month_close')",
        to: "source_type IN ('bikri', 'voucher', 'account', 'month_close', 'purchase', 'venture', 'fruit')",
    },
    // The voucher types of plan 17.
    Widen {
        table: "vouchers",
        from: "'money_lent_return', 'general')",
        to: "'money_lent_return', 'general', 'capital_in', 'capital_out', 'khata_out', 'khata_in')",
    },
    // The party types `separate` (a separate khata) and `garden` (Architecture 15.4).
    Widen {
        table: "account_controls",
        from: "party_type IN ('seller', 'buyer', 'other')",
        to: "party_type IN ('seller', 'buyer', 'separate', 'garden', 'other')",
    },
    Widen {
        table: "accounts",
        from: "party_type IN ('seller', 'buyer', 'other')",
        to: "party_type IN ('seller', 'buyer', 'separate', 'garden', 'other')",
    },
    // Fruit taken from a lot (PRD FR-16.11).
    Widen {
        table: "stock_moves",
        from: "kind IN ('received', 'sold', 'written_off', 'reversal')",
        to: "kind IN ('received', 'sold', 'written_off', 'taken', 'reversal')",
    },
    // A purchase move has no lot.
    Widen {
        table: "stock_moves",
        from: "lot_id INTEGER NOT NULL REFERENCES",
        to: "lot_id INTEGER REFERENCES",
    },
];

/// Widens the CHECK lists in place (see the header of migration 6): edits the text of each table
/// in `sqlite_schema` and raises the schema version, so this connection and every later one read
/// the new text. Each edit must find its text exactly once, or the migration fails and the copy
/// taken before it is put back.
fn widen_checks(tx: &Connection) -> Result<(), HookError> {
    edit_table_text(tx, &WIDEN, "migration 6")
}

/// Edits the text of tables in `sqlite_schema`, as [`widen_checks`] does. `label` names the migration
/// in the error text.
fn edit_table_text(tx: &Connection, edits: &[Widen], label: &str) -> Result<(), HookError> {
    let version: i64 = tx.query_row("PRAGMA schema_version", [], |row| row.get(0))?;
    tx.execute_batch("PRAGMA writable_schema = ON")?;
    for edit in edits {
        let sql: String = tx.query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [edit.table],
            |row| row.get(0),
        )?;
        if sql.matches(edit.from).count() != 1 {
            return Err(HookError::Hook(format!(
                "{label}: the text to widen was not found exactly once in {}",
                edit.table
            )));
        }
        tx.execute(
            "UPDATE sqlite_master SET sql = ?1 WHERE type = 'table' AND name = ?2",
            rusqlite::params![sql.replacen(edit.from, edit.to, 1), edit.table],
        )?;
    }
    let next = version
        .checked_add(1)
        .ok_or_else(|| HookError::Hook(format!("{label}: the schema version is too big")))?;
    tx.execute_batch(&format!("PRAGMA schema_version = {next}"))?;
    tx.execute_batch("PRAGMA writable_schema = OFF")?;
    Ok(())
}

/// After the SQL of migration 6: widen the checks, then seed the new controls and the system
/// account (the `separate` and `garden` controls need the wider party type check), rename the cash
/// account in English, and make the 0604 account of every partner that exists. Public so the tests
/// of migration 7 can build the file as it was released at version 6.
pub fn trips_hook(tx: &Transaction) -> Result<(), HookError> {
    widen_checks(tx)?;
    crate::seed::trip_accounts(tx)?;
    crate::seed::partner_venture_accounts(tx)?;
    Ok(())
}

/// The one CHECK that migration 7 widens: `accounts.serial` was limited to 0 to 9999 for the old
/// 4-digit serial. A running number has no upper limit (plan 32 R2), so only the lower limit stays.
const WIDEN_SERIAL: [Widen; 1] = [Widen {
    table: "accounts",
    from: "serial BETWEEN 0 AND 9999",
    to: "serial >= 0",
}];

/// After the SQL of migration 7: lift the upper limit of `accounts.serial` (see [`WIDEN_SERIAL`]).
/// Public so the tests of migration 8 can build the file as it was released at version 7.
pub fn codes_hook(tx: &Transaction) -> Result<(), HookError> {
    edit_table_text(tx, &WIDEN_SERIAL, "migration 7")
}

/// After the SQL of migration 8: every trip is named by its code and its day, in both scripts
/// (`0070 10-10-2026`), and its three search columns are made again from the new names by
/// `mandi_core::text`, as every save of a named record does (Code Standards B26). Public so the tests
/// of migration 8 can build the file as it was released at version 7.
pub fn trip_names_hook(tx: &Transaction) -> Result<(), HookError> {
    use mandi_core::codes::{CodeType, number_code};
    use mandi_core::dates::{BusinessDate, format_ddmmyyyy};
    let trips: Vec<(i64, i64, String)> = {
        let mut statement = tx.prepare(
            "SELECT id, number, entry_date FROM ventures WHERE kind = 'trip' ORDER BY number",
        )?;
        statement
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<Result<Vec<_>, _>>()?
    };
    for (id, number, entry_date) in trips {
        let day = BusinessDate::parse_iso(&entry_date)
            .map(format_ddmmyyyy)
            .ok_or_else(|| {
                HookError::Hook(format!("migration 8: trip {id} has the date {entry_date}"))
            })?;
        let name = format!("{} {day}", number_code(CodeType::Trip, number));
        tx.execute(
            "UPDATE ventures SET name_ur = ?1, name_en = ?1, search_ur = ?2, search_en = ?3,
                 search_phonetic = ?4 WHERE id = ?5",
            rusqlite::params![
                name,
                mandi_core::text::search_ur(&name),
                mandi_core::text::search_en(&name),
                mandi_core::text::search_phonetic(&name),
                id
            ],
        )?;
    }
    Ok(())
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

/// Checks that a file has exactly this app's schema, and changes nothing. A snapshot opened for
/// reading only (the readable pack) must not be upgraded, and cannot be read right if it is
/// older.
///
/// # Errors
/// `DbError::MigrationFailed` when the file has no schema, an older one (migrations are
/// pending) or a newer one than this app.
pub fn ensure_current(conn: &Connection) -> Result<usize, DbError> {
    let failed = |what: &str, error: &dyn std::fmt::Display| {
        DbError::MigrationFailed(format!("{what}: {error}"))
    };
    let migrations = migrations();
    let version = match migrations
        .current_version(conn)
        .map_err(|e| failed("cannot read the schema version", &e))?
    {
        SchemaVersion::NoneSet => 0,
        SchemaVersion::Inside(v) => v.get(),
        SchemaVersion::Outside(v) => {
            return Err(DbError::MigrationFailed(format!(
                "the file has schema version {v}, newer than this app"
            )));
        }
    };
    let pending = migrations
        .pending_migrations(conn)
        .map_err(|e| failed("cannot count pending migrations", &e))?;
    if version == 0 || pending > 0 {
        return Err(DbError::MigrationFailed(format!(
            "the file has schema version {version}, older than this app"
        )));
    }
    Ok(version)
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
