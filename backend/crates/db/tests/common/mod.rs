//! Shared helpers for the db integration tests.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use mandi_db::DbError;
use mandi_db::audit::{AuditKey, MemoryKeyStore};
use mandi_db::clock::FixedClock;
use mandi_db::disk::FixedDiskProbe;
use mandi_db::rusqlite::{Connection, Error, params};
use mandi_db::{Db, DbOptions};
use tempfile::{TempDir, tempdir};

/// 10 GB free: far above the 50 MB limit.
pub const PLENTY_OF_DISK: u64 = 10 * 1024 * 1024 * 1024;

/// The test key. Real keys are random.
pub fn test_key() -> AuditKey {
    AuditKey::from_bytes([7; 32])
}

/// Options for a test database in `dir`: fixed clock, memory key store, plenty of disk.
pub fn options(dir: &Path) -> DbOptions {
    DbOptions {
        backups_dir: dir.join("backups"),
        disk_probe: Arc::new(FixedDiskProbe(PLENTY_OF_DISK)),
        clock: Arc::new(FixedClock(
            Utc.with_ymd_and_hms(2026, 10, 10, 7, 30, 12).unwrap(),
        )),
        key_store: Arc::new(MemoryKeyStore::new(test_key())),
        busy_timeout: Duration::from_millis(500),
        check_wait: Duration::from_millis(300),
    }
}

/// The path of the data file inside `dir`.
pub fn db_path(dir: &Path) -> PathBuf {
    dir.join("data").join("mandi.db")
}

/// A new, checked database in `dir`.
pub fn open(dir: &Path) -> Db {
    Db::open_checked(&db_path(dir), options(dir)).unwrap()
}

/// A plain connection to the same file, with foreign keys and recursive triggers on, like ours.
pub fn raw(dir: &Path) -> Connection {
    let conn = Connection::open(db_path(dir)).unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    conn.pragma_update(None, "recursive_triggers", "ON")
        .unwrap();
    conn
}

/// The id of a seeded account.
pub fn account_id(conn: &Connection, code: &str) -> i64 {
    conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |row| {
        row.get(0)
    })
    .unwrap()
}

// ---- Ledger and sale fixtures shared by trigger_*.rs ----

pub fn setup() -> (TempDir, Connection) {
    let dir = tempdir().unwrap();
    drop(open(dir.path()));
    let conn = raw(dir.path());
    (dir, conn)
}

pub fn classify(error: Error) -> DbError {
    DbError::from(error)
}

pub fn draft_entry(conn: &Connection, total_paisa: i64) -> i64 {
    conn.execute(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, total_paisa)
         VALUES ('2026-10-10', 'general', 'voucher', 1, ?1)",
        [total_paisa],
    )
    .unwrap();
    conn.last_insert_rowid()
}

pub fn line(
    conn: &Connection,
    entry: i64,
    no: i64,
    account: &str,
    dr: i64,
    cr: i64,
) -> Result<i64, Error> {
    let account = account_id(conn, account);
    conn.execute(
        "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![entry, no, account, dr, cr],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn post(conn: &Connection, entry: i64) -> Result<usize, Error> {
    conn.execute(
        "UPDATE journal_entries SET status = 'posted' WHERE id = ?1",
        [entry],
    )
}

/// Cash receives 12,880 and the commission and laga income accounts give it (PRD 4.4 numbers
/// simplified to one entry): 1_288_000 on each side.
pub fn balanced_entry(conn: &Connection) -> i64 {
    let entry = draft_entry(conn, 1_288_000);
    line(conn, entry, 1, "01010001", 1_288_000, 0).unwrap();
    line(conn, entry, 2, "04010001", 0, 72_000).unwrap();
    line(conn, entry, 3, "04010002", 0, 72_000).unwrap();
    line(conn, entry, 4, "04010003", 0, 16_000).unwrap();
    line(conn, entry, 5, "02010001", 0, 1_128_000).unwrap();
    entry
}

pub struct Fixture {
    pub entry: i64,
    pub bikri: i64,
    pub lot: i64,
    pub sale: i64,
    pub item: i64,
    pub seller: i64,
    pub buyer: i64,
}

/// A saved lot and sale with the PRD 4.4 numbers for Akram: 20 x 600 = 12,000, commission 720
/// each side, laga 160, he pays 12,880.
pub fn fixture(conn: &Connection) -> Fixture {
    let s = |sql: &str| conn.execute_batch(sql).unwrap();
    s(
        "INSERT INTO packings (code, name_ur, name_en, search_ur, search_en, search_phonetic) VALUES ('1', 'پیٹی', 'Box', 'پیٹی', 'box', 'bx')",
    );
    s(
        "INSERT INTO items (code, name_ur, name_en, search_ur, search_en, search_phonetic, item_group) VALUES ('1', 'کیلا', 'Banana', 'کیلا', 'banana', 'bnn', 'fruit')",
    );
    let control = |code: &str| -> i64 {
        conn.query_row(
            "SELECT id FROM account_controls WHERE code = ?1",
            [code],
            |r| r.get(0),
        )
        .unwrap()
    };
    let account = |control_code: &str,
                   code: &str,
                   serial: i64,
                   party: &str,
                   ur: &str,
                   en: &str|
     -> i64 {
        conn.execute(
            "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur, search_en, search_phonetic) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?5, lower(?6), lower(?6))",
            params![control(control_code), code, serial, party, ur, en],
        )
        .unwrap();
        conn.last_insert_rowid()
    };
    let seller = account("03011", "030110001", 1, "seller", "رحیم", "Rahim");
    let buyer = account("0304", "03040001", 1, "buyer", "اکرم", "Akram");
    let one = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap() };
    let item = one("SELECT id FROM items LIMIT 1");
    let packing = one("SELECT id FROM packings LIMIT 1");
    conn.execute("INSERT INTO bikri_entries (number, entry_date, bill_date, truck_no) VALUES (1, '2026-10-10', '2026-10-10', 'LES-1234')", []).unwrap();
    let bikri = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO bikri_lots (bikri_id, lot_no, seller_account_id, item_id, packing_id, qty_received) VALUES (?1, 1, ?2, ?3, ?4, 50)",
        params![bikri, seller, item, packing],
    )
    .unwrap();
    let lot = conn.last_insert_rowid();
    let entry = draft_entry(conn, 1_288_000);
    conn.execute(
        "INSERT INTO bikri_sales (bikri_id, lot_id, line_no, journal_entry_id, entry_date, buyer_account_id, qty,
            seller_rate_paisa, seller_amount_paisa, seller_comm_method, seller_comm_value_bps, seller_comm_paisa,
            buyer_rate_paisa, buyer_value_paisa, buyer_comm_method, buyer_comm_value_bps, buyer_comm_paisa,
            laga_method, laga_value_paisa, laga_paisa, buyer_total_paisa, payment_type)
         VALUES (?1, ?2, 1, ?3, '2026-10-10', ?4, 20, 60000, 1200000, 'percent', 600, 72000,
            60000, 1200000, 'percent', 600, 72000, 'per_unit', 800, 16000, 1288000, 'credit')",
        params![bikri, lot, entry, buyer],
    )
    .unwrap();
    let sale = conn.last_insert_rowid();
    Fixture {
        entry,
        bikri,
        lot,
        sale,
        item,
        seller,
        buyer,
    }
}

/// A bill and a payment line of Akram (PRD 4.6): he owes 12,880 and pays 10,000.
pub fn bill_and_payment(conn: &Connection) -> (i64, i64, i64, i64) {
    let f = fixture(conn);
    let bill_entry = draft_entry(conn, 1_288_000);
    let bill = line(conn, bill_entry, 1, "03040001", 1_288_000, 0).unwrap();
    let pay_entry = draft_entry(conn, 1_000_000);
    let payment = line(conn, pay_entry, 1, "03040001", 0, 1_000_000).unwrap();
    (bill, payment, f.buyer, f.seller)
}
