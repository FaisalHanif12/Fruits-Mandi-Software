//! Gate lane T3 (plan 80), part 5: ALL the migrations in one run, from the oldest shape with
//! entries (NFR-SCL-8, NFR-REL-1, NFR-REL-13, NFR-SEC-6, Code Standards B19). Owner: `robustness`.
//!
//! `migration.rs` checks every migration on its own, from the version before it, and
//! `robustness_d8_migration.rs` checks the hostile file of version 19 up to the latest one. Here a
//! file of version 4 holds posted entries with their keyed hash chain, a بکری with its lot, its
//! sale line and stock moves, and goes up through migrations 5 to 25 in ONE run. After it:
//!
//! - every chain hash is the one that was written, and the journal chain still verifies with the
//!   code of today (so nothing in the hashed bytes moved through 21 changes of the design);
//! - every journal line, every balance, the sale line's money, the lot's quantities, the account
//!   names and codes are what they were;
//! - the checks of SQLite pass, and a second run is a no-op that takes no copy;
//! - a tamper after the upgrade is still found by the chain.
#![cfg(test)]

mod common;

use chrono::{TimeZone, Utc};
use mandi_db::clock::FixedClock;
use mandi_db::migrate::{
    self, BALANCE_INDEXES_SQL, BIKRI_RECORDS_ARE_NEVER_DELETED_SQL, INIT_SQL,
    SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL,
};
use mandi_db::rusqlite::Connection;
use mandi_db::seed;
use rusqlite_migration::{HookError, M, Migrations};
use tempfile::tempdir;

const LATEST: i64 = 25;

fn clock() -> FixedClock {
    FixedClock(Utc.with_ymd_and_hms(2026, 10, 10, 7, 30, 12).unwrap())
}

/// The schema as it was released at version 4: migrations 0001 to 0004.
fn released_version_4() -> Migrations<'static> {
    Migrations::new(vec![
        M::up_with_hook(INIT_SQL, |tx| {
            seed::system_accounts(tx)
                .map(|_| ())
                .map_err(HookError::from)
        }),
        M::up(BALANCE_INDEXES_SQL),
        M::up(SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL),
        M::up(BIKRI_RECORDS_ARE_NEVER_DELETED_SQL),
    ])
}

fn connection(path: &std::path::Path) -> Connection {
    let conn = Connection::open(path).unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    conn.pragma_update(None, "recursive_triggers", "ON")
        .unwrap();
    conn
}

fn user_version(conn: &Connection) -> i64 {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap()
}

/// Appends bytes the way the hash chain writes a text or a number (type byte, length, bytes).
fn put(out: &mut Vec<u8>, text: Option<&str>) {
    match text {
        None => out.push(0),
        Some(text) => {
            out.push(1);
            out.extend_from_slice(&(text.len() as u64).to_le_bytes());
            out.extend_from_slice(text.as_bytes());
        }
    }
}

/// Posts a general entry the way the app of version 4 did: raw rows, then one update that sets the
/// status, the total and the keyed hash of the chain, and the head record. The hashed bytes are
/// copied from the code of version 4, so a hash that still verifies after the upgrade proves that
/// the chain format has not changed.
fn legacy_post(
    conn: &Connection,
    existing: Option<i64>,
    day: &str,
    from: &str,
    to: &str,
    paisa: i64,
) -> i64 {
    let account = |code: &str| -> i64 {
        conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
            r.get(0)
        })
        .unwrap()
    };
    let entry = match existing {
        Some(entry) => entry,
        None => {
            conn.execute(
                "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, status, total_paisa)
                 VALUES (?1, 'general', 'voucher', 1, 'draft', 0)",
                [day],
            )
            .unwrap();
            conn.last_insert_rowid()
        }
    };
    let lines = [(from, paisa, 0_i64), (to, 0, paisa)];
    for (index, (code, dr, cr)) in lines.iter().enumerate() {
        conn.execute(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, ?2, ?3, ?4, ?5)",
            (entry, i64::try_from(index).unwrap() + 1, account(code), dr, cr),
        )
        .unwrap();
    }
    let previous: String = conn
        .query_row(
            "SELECT COALESCE((SELECT chain_hash FROM journal_entries WHERE status = 'posted' ORDER BY id DESC LIMIT 1), ?1)",
            [mandi_db::audit::GENESIS_HASH],
            |r| r.get(0),
        )
        .unwrap();
    let mut bytes = Vec::new();
    put(&mut bytes, Some(&previous));
    put(&mut bytes, Some(&entry.to_string()));
    put(&mut bytes, Some(day));
    put(&mut bytes, Some("general"));
    put(&mut bytes, Some("voucher"));
    put(&mut bytes, Some("1"));
    put(&mut bytes, None);
    put(&mut bytes, None);
    put(&mut bytes, Some(&paisa.to_string()));
    put(&mut bytes, Some("2"));
    for (index, (code, dr, cr)) in lines.iter().enumerate() {
        put(&mut bytes, Some(&(index + 1).to_string()));
        put(&mut bytes, Some(&account(code).to_string()));
        put(&mut bytes, None);
        put(&mut bytes, Some(&dr.to_string()));
        put(&mut bytes, Some(&cr.to_string()));
        put(&mut bytes, Some("0"));
        put(&mut bytes, None);
    }
    // The test key of `common::test_key()` is 32 bytes of 7.
    let hash = blake3::keyed_hash(&[7_u8; 32], &bytes).to_hex().to_string();
    conn.execute(
        "UPDATE journal_entries SET status = 'posted', total_paisa = ?1, chain_hash = ?2 WHERE id = ?3",
        (paisa, &hash, entry),
    )
    .unwrap();
    conn.execute(
        "INSERT INTO settings (key, value_json) VALUES ('journal_head', ?1)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
        [format!("{{\"id\":{entry},\"hash\":\"{hash}\"}}")],
    )
    .unwrap();
    entry
}

fn rows(conn: &Connection, sql: &str) -> Vec<String> {
    let columns = conn.prepare(sql).unwrap().column_count();
    conn.prepare(sql)
        .unwrap()
        .query_map([], |r| {
            let mut text = String::new();
            for i in 0..columns {
                let value: mandi_db::rusqlite::types::Value = r.get(i)?;
                text.push_str(&format!("{value:?}|"));
            }
            Ok(text)
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// What an upgrade must not move, read with columns that exist at version 4 and still at 25.
fn untouchable(conn: &Connection) -> Vec<(&'static str, Vec<String>)> {
    vec![
        (
            "chain hashes",
            rows(
                conn,
                "SELECT id, entry_date, status, total_paisa, chain_hash FROM journal_entries ORDER BY id",
            ),
        ),
        (
            "journal lines",
            rows(
                conn,
                "SELECT id, entry_id, line_no, account_id, dr_paisa, cr_paisa, is_advance FROM journal_lines ORDER BY id",
            ),
        ),
        (
            "balances",
            rows(
                conn,
                // Only accounts with lines: the upgrades rename, add and remove empty system accounts.
                "SELECT a.code, SUM(l.dr_paisa), SUM(l.cr_paisa)
                 FROM accounts a JOIN journal_lines l ON l.account_id = a.id GROUP BY a.id ORDER BY a.code",
            ),
        ),
        (
            "the sale line",
            rows(
                conn,
                "SELECT id, lot_id, qty, seller_rate_paisa, seller_amount_paisa, seller_comm_paisa,
                        buyer_value_paisa, buyer_comm_paisa, laga_paisa, buyer_total_paisa
                 FROM bikri_sales ORDER BY id",
            ),
        ),
        (
            "the lot",
            rows(
                conn,
                "SELECT id, bikri_id, lot_no, seller_account_id, item_id, packing_id, qty_received FROM bikri_lots ORDER BY id",
            ),
        ),
        (
            "stock moves",
            rows(
                conn,
                "SELECT id, lot_id, journal_entry_id, move_date, kind, qty FROM stock_moves ORDER BY id",
            ),
        ),
        (
            "the settings of the chain head",
            rows(conn, "SELECT key, value_json FROM settings WHERE key = 'journal_head'"),
        ),
    ]
}

#[test]
fn a_file_of_version_4_with_posted_entries_goes_through_every_migration_with_every_hash_unchanged()
{
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let path = dir.path().join("mandi.db");
    let mut conn = connection(&path);
    migrate::migrate_with(&released_version_4(), &mut conn, &backups, &clock()).unwrap();
    assert_eq!(user_version(&conn), 4);
    let f = common::fixture(&conn);

    // Forty posted entries between the cash, income, seller and buyer accounts, a draft entry of
    // the sale line first (as the fixture makes it), and stock moves.
    let accounts = [
        ("01010001", "04010001"),
        ("01010001", "04010002"),
        ("01010001", "04010003"),
        ("0020", "04010001"),
        ("0010", "01010001"),
        ("01010001", "0020"),
        ("0020", "0010"),
        ("02010001", "01010001"),
    ];
    let first = legacy_post(
        &conn,
        Some(f.entry),
        "2026-10-10",
        "01010001",
        "04010001",
        1_288_000,
    );
    let mut entries = vec![first];
    for n in 0..39_usize {
        let (from, to) = accounts[n % accounts.len()];
        let day = format!("2026-10-{:02}", 1 + (n % 9));
        let paisa = 100 * i64::try_from(37 + n * 13).unwrap();
        entries.push(legacy_post(&conn, None, &day, from, to, paisa));
    }
    for (kind, qty, entry) in [("received", 50_i64, entries[0]), ("sold", -20, entries[1])] {
        conn.execute(
            "INSERT INTO stock_moves (lot_id, journal_entry_id, move_date, kind, qty) VALUES (?1, ?2, '2026-10-10', ?3, ?4)",
            (f.lot, entry, kind, qty),
        )
        .unwrap();
    }
    // A name and code the upgrade must keep.
    let names_before = rows(
        &conn,
        "SELECT id, code, name_ur, name_en, opening_dr_paisa, opening_cr_paisa FROM accounts WHERE is_system = 0 ORDER BY id",
    );
    assert!(names_before.len() >= 2);
    let before = untouchable(&conn);
    let posted_before = before[0].1.len();
    assert_eq!(posted_before, 40);

    let outcome = migrate::migrate(&mut conn, &backups, &clock()).unwrap();
    assert_eq!(
        (outcome.from, outcome.to),
        (4, usize::try_from(LATEST).unwrap())
    );
    assert!(outcome.copy.is_some(), "a copy is taken before an upgrade");
    assert_eq!(user_version(&conn), LATEST);

    let after = untouchable(&conn);
    for ((label, was), (_, now)) in before.iter().zip(after.iter()) {
        assert_eq!(was, now, "{label} moved in the upgrade from 4 to {LATEST}");
    }
    assert_eq!(
        rows(
            &conn,
            "SELECT id, code, name_ur, name_en, opening_dr_paisa, opening_cr_paisa FROM accounts WHERE is_system = 0 ORDER BY id",
        ),
        names_before,
        "an account changed"
    );
    // The chain still verifies with the code of today, entry by entry.
    mandi_db::ledger::verify_journal_chain(&conn, &common::test_key()).unwrap();
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    let broken_keys: i64 = conn
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(broken_keys, 0);
    // New columns hold the right values for old rows: the lot has its counter, the sale line was
    // a credit line so it is paid nothing, no account is linked.
    let (counter, paid): (i64, i64) = conn
        .query_row(
            "SELECT (SELECT lot_counter FROM bikri_lots WHERE id = ?1), (SELECT paid_paisa FROM bikri_sales WHERE id = ?2)",
            (f.lot, f.sale),
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((counter, paid), (1, 0));
    // The books balance.
    let imbalance: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(dr_paisa), 0) - COALESCE(SUM(cr_paisa), 0) FROM journal_lines",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(imbalance, 0);
    // A second run does nothing and takes no copy.
    let again = migrate::migrate(&mut conn, &backups, &clock()).unwrap();
    assert_eq!(
        (again.from, again.to),
        (
            usize::try_from(LATEST).unwrap(),
            usize::try_from(LATEST).unwrap()
        )
    );
    assert!(again.copy.is_none());

    // Someone changes an amount after the upgrade, outside the app: the chain finds it.
    let triggers: Vec<String> = rows(
        &conn,
        "SELECT name FROM sqlite_master WHERE type = 'trigger'",
    );
    for t in triggers {
        let name = t
            .trim_start_matches("Text(\"")
            .trim_end_matches("\")|")
            .to_owned();
        conn.execute_batch(&format!("DROP TRIGGER IF EXISTS \"{name}\""))
            .unwrap();
    }
    conn.pragma_update(None, "foreign_keys", "OFF").unwrap();
    conn.execute_batch(
        "UPDATE journal_lines SET dr_paisa = dr_paisa + 100 WHERE id = (SELECT id FROM journal_lines WHERE dr_paisa > 0 ORDER BY id LIMIT 1 OFFSET 7);
         UPDATE journal_lines SET cr_paisa = cr_paisa + 100 WHERE id = (SELECT l2.id FROM journal_lines l2 WHERE l2.entry_id = (SELECT entry_id FROM journal_lines WHERE dr_paisa > 0 ORDER BY id LIMIT 1 OFFSET 7) AND l2.cr_paisa > 0);",
    )
    .unwrap();
    assert!(
        mandi_db::ledger::verify_journal_chain(&conn, &common::test_key()).is_err(),
        "the chain still verifies after an amount was changed outside the app"
    );
}

#[test]
fn an_empty_file_of_version_1_goes_to_the_latest_and_a_file_from_the_future_is_refused_untouched() {
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let path = dir.path().join("mandi.db");
    let mut conn = connection(&path);
    let one = Migrations::new(vec![M::up_with_hook(INIT_SQL, |tx| {
        seed::system_accounts(tx)
            .map(|_| ())
            .map_err(HookError::from)
    })]);
    migrate::migrate_with(&one, &mut conn, &backups, &clock()).unwrap();
    assert_eq!(user_version(&conn), 1);
    let outcome = migrate::migrate(&mut conn, &backups, &clock()).unwrap();
    assert_eq!(
        (outcome.from, outcome.to),
        (1, usize::try_from(LATEST).unwrap())
    );
    mandi_db::ledger::verify_journal_chain(&conn, &common::test_key()).unwrap();
    // A file made by a newer app: refused, and not a byte of it changes.
    conn.pragma_update(None, "user_version", LATEST + 1)
        .unwrap();
    drop(conn);
    let before = std::fs::read(&path).unwrap();
    let mut conn = connection(&path);
    let refused = migrate::migrate(&mut conn, &backups, &clock());
    assert!(refused.is_err(), "a file from the future was accepted");
    drop(conn);
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "the file was touched"
    );
}
