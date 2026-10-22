//! A consistent snapshot of the live data file, and the check of any copy (NFR-BAK-4,
//! NFR-BAK-5, NFR-SPD-15, Architecture 14).
//!
//! The snapshot uses the SQLite online backup API in one step on the read connection, so the
//! app keeps working while it runs and the copy is never half of a save. The check opens a copy
//! as a database, runs the full integrity check, counts every table, adds up the books and
//! hashes the file. The check can run in the background after the snapshot is safe.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

use mandi_core::money::Paisa;
use mandi_db::Db;
use mandi_db::disk::DiskProbe;
use mandi_db::rusqlite::backup::{Backup, StepResult};
use mandi_db::rusqlite::{Connection, OpenFlags};

use crate::error::{BackupCode, BackupError};
use crate::io::{hash_file, partial_path, remove_quietly, require_room};
use crate::model::BackupTotals;

/// Free space kept beyond the copy itself (16 MiB).
pub const ROOM_MARGIN: u64 = 16 * 1024 * 1024;

/// The cash account (PRD FR-10.4).
const CASH_CODE: &str = "01010001";

/// What the check of a copy found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inspected {
    /// Size of the file.
    pub size_bytes: u64,
    /// Checksum of the file.
    pub blake3: String,
    /// `PRAGMA user_version`: the number of migrations applied.
    pub schema_version: i64,
    /// Rows of every table.
    pub counts: BTreeMap<String, i64>,
    /// The books.
    pub totals: BackupTotals,
}

fn corrupt(detail: impl std::fmt::Display) -> BackupError {
    BackupError::new(BackupCode::ErrBackupCorrupt, detail)
}

/// Size of the data file and its write-ahead log, which is what a snapshot has to hold.
fn live_size(db: &Db) -> u64 {
    let main = fs::metadata(db.path()).map_or(0, |m| m.len());
    let mut wal = db.path().as_os_str().to_owned();
    wal.push("-wal");
    main.saturating_add(fs::metadata(wal).map_or(0, |m| m.len()))
}

/// Copies the live data to `dest` as one plain SQLite file, in the format of a single file
/// (no `-wal`). Returns its size. Writes through `<dest>.partial` and renames, so `dest` is
/// whole or absent.
///
/// # Errors
/// `ERR_DISK_FULL` when the folder of `dest` has not room for twice the data file;
/// `ERR_INTERNAL` for any other failure (the cause is in the log). Nothing is left behind.
pub fn snapshot_copy(db: &Db, dest: &Path, probe: &dyn DiskProbe) -> Result<u64, BackupError> {
    let needed = live_size(db).saturating_mul(2).saturating_add(ROOM_MARGIN);
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir)?;
        require_room(probe, dir, needed)?;
    }
    let partial = partial_path(dest);
    remove_quietly(&partial);
    let result = copy_live(db, &partial).and_then(|()| {
        fs::rename(&partial, dest)?;
        Ok(fs::metadata(dest)?.len())
    });
    if result.is_err() {
        remove_quietly(&partial);
    }
    result
}

fn copy_live(db: &Db, partial: &Path) -> Result<(), BackupError> {
    let mut to = Connection::open(partial)?;
    db.read(|from| {
        // One step with every page: the copy is a snapshot of one moment, and a save that
        // commits meanwhile goes to the log, not into the copy.
        let backup = Backup::new(from, &mut to)?;
        let mut tries = 0_u32;
        loop {
            match backup.step(-1)? {
                StepResult::Done => return Ok(()),
                StepResult::More => {}
                // A lock: try again a moment later.
                _ => {
                    tries += 1;
                    if tries > 400 {
                        return Err(BackupError::new(
                            BackupCode::ErrDbBusy,
                            "the data file stayed locked",
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
            }
        }
    })?;
    // The copy keeps the log mode of the live file. A plain single file is easier to move,
    // hash and open read-only.
    let mode: String =
        to.pragma_update_and_check(None, "journal_mode", "DELETE", |row| row.get(0))?;
    if !mode.eq_ignore_ascii_case("delete") {
        return Err(BackupError::internal(format!(
            "cannot leave WAL mode, got {mode}"
        )));
    }
    drop(to);
    fs::File::open(partial)?.sync_all()?;
    Ok(())
}

/// Opens a copy as a database and checks it: the full integrity check, the schema version, the
/// rows of every table, the books and the checksum of the file.
///
/// # Errors
/// `ERR_BACKUP_CORRUPT` when the file is not a database, fails its integrity check or cannot be
/// read.
pub fn inspect(path: &Path) -> Result<Inspected, BackupError> {
    let size_bytes = fs::metadata(path)
        .map_err(|e| corrupt(format!("cannot read {}: {e}", path.display())))?
        .len();
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(corrupt)?;
    let lines: Vec<String> = conn
        .prepare("PRAGMA integrity_check")
        .and_then(|mut statement| {
            statement
                .query_map([], |row| row.get::<_, String>(0))
                .and_then(Iterator::collect)
        })
        .map_err(corrupt)?;
    if lines.len() != 1 || lines.first().is_none_or(|line| line != "ok") {
        return Err(corrupt(format!(
            "integrity check: {}",
            lines.into_iter().take(5).collect::<Vec<_>>().join("; ")
        )));
    }
    let schema_version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(corrupt)?;
    let counts = table_counts(&conn).map_err(corrupt)?;
    let totals = read_totals(&conn)?;
    let blake3 = hash_file(path).map_err(|e| corrupt(format!("cannot hash: {e}")))?;
    Ok(Inspected {
        size_bytes,
        blake3,
        schema_version,
        counts,
        totals,
    })
}

/// Rows of every table. The table names come from the file's own schema and are quoted.
fn table_counts(conn: &Connection) -> Result<BTreeMap<String, i64>, mandi_db::rusqlite::Error> {
    let names: Vec<String> = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    let mut counts = BTreeMap::new();
    for name in names {
        let quoted = name.replace('"', "\"\"");
        let count: i64 =
            conn.query_row(&format!("SELECT COUNT(*) FROM \"{quoted}\""), [], |row| {
                row.get(0)
            })?;
        counts.insert(name, count);
    }
    Ok(counts)
}

/// Reads the totals of the books from a data file (PRD FR-6.2: the two sides are equal).
///
/// # Errors
/// `ERR_BACKUP_CORRUPT` when a table is missing or an amount is not whole rupees.
pub fn read_totals(conn: &Connection) -> Result<BackupTotals, BackupError> {
    let one = |sql: &str| -> Result<i64, BackupError> {
        conn.query_row(sql, [], |row| row.get(0)).map_err(corrupt)
    };
    let money = |paisa: i64| Paisa::try_from_paisa(paisa).map_err(corrupt);
    let (debit, credit): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0) FROM journal_lines",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(corrupt)?;
    let (cash_dr, cash_cr): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0) \
             FROM journal_lines l JOIN accounts a ON a.id = l.account_id WHERE a.code = ?1",
            [CASH_CODE],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(corrupt)?;
    let cash = money(cash_dr)?
        .checked_sub(money(cash_cr)?)
        .map_err(corrupt)?;
    let last_entry_date: Option<String> = conn
        .query_row(
            "SELECT MAX(entry_date) FROM journal_entries WHERE status = 'posted'",
            [],
            |row| row.get(0),
        )
        .map_err(corrupt)?;
    Ok(BackupTotals {
        accounts: one("SELECT COUNT(*) FROM accounts")?,
        bikri_entries: one("SELECT COUNT(*) FROM bikri_entries")?,
        vouchers: one("SELECT COUNT(*) FROM vouchers")?,
        journal_entries: one("SELECT COUNT(*) FROM journal_entries WHERE status = 'posted'")?,
        audit_rows: one("SELECT COUNT(*) FROM audit_log")?,
        last_entry_date,
        debit_paisa: money(debit)?,
        credit_paisa: money(credit)?,
        cash_paisa: cash,
    })
}
