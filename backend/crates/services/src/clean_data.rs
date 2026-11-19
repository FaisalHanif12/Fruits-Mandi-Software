//! Clean data (plan 32 R9): makes a new data file for the Owner's app with nothing of the business in
//! it, from the live file. Used by the example `clean_data` (`pnpm seed -- --clean <live file>`).
//!
//! The new file has the full schema, the system accounts and the starting rules (6%, 6%, لاگا 8 on
//! fruit) from the migrations, and nothing else of the business: no partners, sellers, buyers, items,
//! packings, staff, cities, entries, drafts or closed days. From the live file it keeps only
//!
//! - the Owner: the same id, user name, display name and password hash, so he signs in with the
//!   password he has today (the wrong-tries counter and the lock start again at zero);
//! - the settings of the firm and the app: every row of `settings` except the journal head and the
//!   last health result, which belong to the books that are left behind. This holds the firm name,
//!   mandi, address, phones, the contact persons, the trade mark, the languages, the number
//!   grouping, the idle lock, the printing and backup setup and the recovery code (its hash);
//! - the language and text size the Owner chose.
//!
//! The live file is never opened, changed or deleted. A copy of it (and of its `-wal` file, if any)
//! is made in a temporary folder next to the new file, read, and removed.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mandi_db::audit::AuditEntry;
use mandi_db::rusqlite::{Connection, params};
use mandi_db::{Db, DbOptions};
use serde_json::json;

use crate::context::Context;
use crate::error::ServiceError;
use crate::health;

/// Settings that belong to the books that are left behind: the head of the journal chain and the
/// last health result. Everything else in `settings` is copied.
pub const LEFT_BEHIND_SETTINGS: [&str; 2] = ["journal_head", "health_last"];
/// The user preferences that are kept: how the Owner likes to see the app.
pub const KEPT_PREFS: [&str; 2] = ["language", "text_size"];

fn fail(text: &str) -> ServiceError {
    ServiceError::Data(text.to_owned())
}

/// Why a clean file was not made, in plain words for the person who runs the tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanError(pub String);

impl std::fmt::Display for CleanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CleanError {}

impl From<ServiceError> for CleanError {
    fn from(error: ServiceError) -> Self {
        match error {
            // Our own messages are already plain words.
            ServiceError::Data(text) => CleanError(text),
            other => CleanError(other.to_string()),
        }
    }
}

/// What the live file held, and what was kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanReport {
    /// The user name of the Owner that was kept.
    pub username: String,
    /// The keys of the settings that were kept.
    pub kept_settings: Vec<String>,
    /// What the live file held and the new file does not: `(what, how many)`.
    pub left_behind: Vec<(&'static str, i64)>,
    /// How many partners the live file has (none is copied).
    pub partners: i64,
    /// System accounts in the new file.
    pub system_accounts: i64,
    /// Starting rules in the new file.
    pub rules: i64,
    /// Sellers, buyers and journal entries in the new file: all 0.
    pub sellers: i64,
    /// Buyers in the new file.
    pub buyers: i64,
    /// Items in the new file.
    pub items: i64,
    /// Journal entries in the new file.
    pub entries: i64,
}

/// What is read from the live file.
struct Live {
    owner_id: i64,
    username: String,
    password_hash: String,
    display_name: String,
    role: String,
    settings: Vec<(String, String)>,
    prefs: Vec<(String, String)>,
    partners: i64,
    counts: Vec<(&'static str, i64)>,
}

/// Copies the live file (and its write-ahead log, if any) into `folder` and returns the copy.
fn copy_live(live: &Path, folder: &Path) -> Result<PathBuf, ServiceError> {
    std::fs::create_dir_all(folder).map_err(|e| fail(&format!("cannot make {folder:?}: {e}")))?;
    let copy = folder.join("mandi.db");
    std::fs::copy(live, &copy).map_err(|e| fail(&format!("cannot copy {live:?}: {e}")))?;
    for suffix in ["-wal", "-shm"] {
        let mut from = live.as_os_str().to_owned();
        from.push(suffix);
        let from = PathBuf::from(from);
        if from.exists() {
            let mut to = copy.as_os_str().to_owned();
            to.push(suffix);
            std::fs::copy(&from, PathBuf::from(to))
                .map_err(|e| fail(&format!("cannot copy {from:?}: {e}")))?;
        }
    }
    Ok(copy)
}

fn count(conn: &Connection, table: &str) -> i64 {
    // `table` is one of the fixed names below, never input. A table the old file does not have
    // counts as 0.
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap_or(0)
}

/// The accounts the Owner added: every account that is not a system account.
fn count_added_accounts(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM accounts WHERE is_system = 0",
        [],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

/// Reads what is kept, from the copy.
fn read_live(copy: &Path) -> Result<Live, ServiceError> {
    let conn = Connection::open(copy)?;
    conn.pragma_update(None, "query_only", "ON")?;
    let (owner_id, username, password_hash, display_name, role) = conn
        .query_row(
            "SELECT id, username, password_hash, display_name, role FROM users
             ORDER BY (role = 'owner') DESC, id LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map_err(|_| fail("the live file has no user: nothing to keep"))?;
    let mut statement = conn.prepare("SELECT key, value_json FROM settings ORDER BY id")?;
    let settings = statement
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(key, _)| !LEFT_BEHIND_SETTINGS.contains(&key.as_str()))
        .collect();
    let mut statement =
        conn.prepare("SELECT key, value_json FROM user_prefs WHERE user_id = ?1")?;
    let prefs = statement
        .query_map([owner_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(key, _)| KEPT_PREFS.contains(&key.as_str()))
        .collect();
    let counts = vec![
        ("partners", count(&conn, "partners")),
        (
            "accounts added (sellers, buyers, people, khatas, gardens ...)",
            count_added_accounts(&conn),
        ),
        ("items", count(&conn, "items")),
        ("packings", count(&conn, "packings")),
        ("staff", count(&conn, "staff")),
        ("cities", count(&conn, "cities")),
        ("sales (بکری)", count(&conn, "bikri_entries")),
        ("posted journal entries", count(&conn, "journal_entries")),
    ];
    let partners = count(&conn, "partners");
    Ok(Live {
        owner_id,
        username,
        password_hash,
        display_name,
        role,
        settings,
        prefs,
        partners,
        counts,
    })
}

/// Writes the Owner and his settings into the new file, in one transaction, with one audit row.
fn write_clean(ctx: &Context, live: &Live, source_name: &str) -> Result<(), ServiceError> {
    let now = ctx
        .db()
        .clock()
        .now_utc()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();
    ctx.db().write(|tx| {
        let existing: i64 = tx.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?;
        if existing > 0 {
            return Err(fail("the new file already has a user"));
        }
        tx.execute(
            "INSERT INTO users (id, created_at, created_by, updated_at, updated_by, username,
                 password_hash, display_name, role)
             VALUES (?1, ?2, ?1, ?2, ?1, ?3, ?4, ?5, ?6)",
            params![
                live.owner_id,
                now,
                live.username,
                live.password_hash,
                live.display_name,
                live.role
            ],
        )?;
        for (key, value) in &live.settings {
            tx.execute(
                "INSERT INTO settings (key, value_json, created_by, updated_by) VALUES (?1, ?2, ?3, ?3)",
                params![key, value, live.owner_id],
            )?;
        }
        for (key, value) in &live.prefs {
            tx.execute(
                "INSERT INTO user_prefs (user_id, key, value_json, created_by, updated_by)
                 VALUES (?1, ?2, ?3, ?1, ?1)",
                params![live.owner_id, key, value],
            )?;
        }
        let keys: Vec<&str> = live.settings.iter().map(|(k, _)| k.as_str()).collect();
        // No password, hash or recovery code goes in the audit row.
        ctx.db().auditor().write(
            tx,
            &AuditEntry {
                user_id: Some(live.owner_id),
                action: "create".to_owned(),
                entity: "clean_data".to_owned(),
                entity_id: None,
                old_json: None,
                new_json: Some(json!({
                    "kept_user": live.username,
                    "kept_settings": keys,
                    "copied_from": source_name,
                })),
                reason: Some("clean data (plan 32)".to_owned()),
            },
        )?;
        Ok::<_, ServiceError>(())
    })
}

/// Makes the clean file `new_path` from the live file `live_path`. `options` say where the copy
/// before a migration goes and which audit key store and clock are used (the real ones in the
/// app's tool, the test ones in a test). Stops when the live file has partners unless
/// `drop_partners` is true.
///
/// # Errors
/// `CleanError` with a plain text when the live file is missing or has no user, when the new file
/// exists, or when the live file has partners and `drop_partners` is false; any database error.
pub fn make_clean_file(
    live_path: &Path,
    new_path: &Path,
    options: DbOptions,
    drop_partners: bool,
) -> Result<CleanReport, CleanError> {
    Ok(make(live_path, new_path, options, drop_partners)?)
}

fn make(
    live_path: &Path,
    new_path: &Path,
    options: DbOptions,
    drop_partners: bool,
) -> Result<CleanReport, ServiceError> {
    if !live_path.is_file() {
        return Err(fail(&format!("the live file {live_path:?} does not exist")));
    }
    if new_path.exists() {
        return Err(fail(&format!(
            "{new_path:?} already exists: remove it first (pnpm seed -- --clean ... --reset)"
        )));
    }
    let same = match (
        live_path.canonicalize(),
        new_path.parent().map(Path::canonicalize),
    ) {
        (Ok(live), Some(Ok(folder))) => new_path
            .file_name()
            .is_some_and(|name| folder.join(name) == live),
        _ => false,
    };
    if same {
        return Err(fail("the new file must not be the live file"));
    }
    let folder = new_path
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    std::fs::create_dir_all(&folder).map_err(|e| fail(&format!("cannot make {folder:?}: {e}")))?;
    let temp = folder.join(format!(".live-copy-{}", std::process::id()));
    let copy = copy_live(live_path, &temp)?;
    let read = read_live(&copy);
    // The copy has done its job whatever the result.
    let _ = std::fs::remove_dir_all(&temp);
    let live = read?;
    if live.partners > 0 && !drop_partners {
        return Err(fail(&format!(
            "the live file has {} partner(s) and a clean file has none. Nothing was made. \
             Run again with --drop-partners if that is what is wanted",
            live.partners
        )));
    }

    let db = Db::open_checked(new_path, options)?;
    let ctx = Context::new(Arc::new(db));
    let source_name = live_path.file_name().map_or_else(
        || "mandi.db".to_owned(),
        |n| n.to_string_lossy().into_owned(),
    );
    write_clean(&ctx, &live, &source_name)?;

    // The new file must be sound: the books are empty and balance, and both chains verify.
    let check = health::check_books(&ctx)?;
    if !check.is_ok() {
        return Err(fail("the new file did not pass the books check"));
    }
    let count = |sql: &str| -> Result<i64, ServiceError> {
        Ok(ctx
            .db()
            .read(|c| c.query_row(sql, [], |r| r.get::<_, i64>(0)))?)
    };
    Ok(CleanReport {
        username: live.username,
        kept_settings: live.settings.into_iter().map(|(key, _)| key).collect(),
        left_behind: live.counts,
        partners: live.partners,
        system_accounts: count("SELECT COUNT(*) FROM accounts")?,
        rules: count("SELECT COUNT(*) FROM charge_rules")?,
        sellers: count(
            "SELECT COUNT(*) FROM accounts a JOIN account_controls k ON k.id = a.control_id
             WHERE k.code = '03011' AND a.is_system = 0",
        )?,
        buyers: count(
            "SELECT COUNT(*) FROM accounts a JOIN account_controls k ON k.id = a.control_id
             WHERE k.code = '0304' AND a.is_system = 0",
        )?,
        items: count("SELECT COUNT(*) FROM items")?,
        entries: count("SELECT COUNT(*) FROM journal_entries")?,
    })
}
