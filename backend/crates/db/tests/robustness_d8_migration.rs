//! D8 end gate, attack on migrations 20 and 21 (plan 64, plan 65; PRD FR-13.9, FR-12.44; NFR-REL-1,
//! NFR-REL-13, NFR-SCL-8). Owner: `robustness`.
//!
//! 1. `hostile_old_data_goes_from_19_to_21_in_one_run`: a file of version 19 with posted entries and
//!    their chain, one-time people on 0030 and 0031 (two spellings of one person, a nameless person, a
//!    person with no phone, spaces round a phone, a zero-width character, a 5,000 letter name), old پختہ
//!    party accounts with postings, replaced and cancelled lines, cash and credit lines, seller side
//!    person columns, analytics rows with person keys, dead-amount marks on a person and on an account.
//!    After one upgrade: every chain hash, journal row, balance and number is what it was, the chain
//!    verifies, the integrity and foreign-key checks pass, `chukta_buyers` holds exactly the people with a
//!    name, a cash line is paid in full and a credit line paid 0 (replaced and cancelled lines too), and
//!    running again is a no-op that takes no copy.
//! 2. `a_failure_in_the_last_step_puts_the_whole_file_back_and_a_rerun_then_works`: a stray trigger on
//!    `bikri_sales` makes migration 21 fail after migration 20 ran in the same upgrade. The file is as it
//!    was (same rows, same schema, version 19), the copy is whole, and after the fault is taken away the
//!    upgrade runs and gives the same result as the clean one.
//! 3. `a_failure_in_the_first_step_puts_the_file_back`: a stray table `chukta_buyers` stops migration 20.
//! 4. `killing_the_process_during_the_upgrade_40_times_never_leaves_a_half_upgraded_file`.
//! 5. `a_real_backup_copy_migrates_without_moving_a_number`: only when `D8_REAL_COPY` names a copy of a
//!    real backup. The test works on its own temp copy and never touches the original.
#![cfg(test)]

mod common;

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use chrono::{TimeZone, Utc};
use mandi_db::clock::FixedClock;
use mandi_db::migrate::{
    self, ASSISTANT_QUESTIONS_SQL, ASSISTANT_TIME_LIMIT_SQL, ASSISTANT_TIMING_SQL,
    BALANCE_INDEXES_SQL, BIKRI_RECORDS_ARE_NEVER_DELETED_SQL, DATE_TO_SELL_AND_GARDEN_ARRIVALS_SQL,
    DEAD_MARKS_SQL, EDIT_WINDOWS_SQL, INIT_SQL, ONE_TIME_PARTY_MARK_SQL, PARTS_ARE_SOFT_STATE_SQL,
    SALE_GRADE_SQL, SHOP_SPLIT_SQL, SIMPLE_PARTY_CODES_SQL, SPEED_INDEXES_SQL,
    STABLE_POSTING_ORDER_SQL, SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL, TRIP_AND_GARDEN_CODES_SQL,
    TRIPS_GARDENS_FRUIT_KHATA_SQL,
};
use mandi_db::rusqlite::{Connection, params};
use mandi_db::{DbError, seed};
use rusqlite_migration::{HookError, M, Migrations};
use tempfile::tempdir;

fn clock() -> FixedClock {
    FixedClock(Utc.with_ymd_and_hms(2026, 10, 10, 7, 30, 12).unwrap())
}

/// The file as released with migration 19 (the quality of a sale line).
fn released_version_19() -> Migrations<'static> {
    Migrations::new(vec![
        M::up_with_hook(INIT_SQL, |tx| {
            seed::system_accounts(tx)
                .map(|_| ())
                .map_err(HookError::from)
        }),
        M::up(BALANCE_INDEXES_SQL),
        M::up(SYSTEM_ACCOUNTS_STAY_ACTIVE_SQL),
        M::up(BIKRI_RECORDS_ARE_NEVER_DELETED_SQL),
        M::up(STABLE_POSTING_ORDER_SQL),
        M::up_with_hook(TRIPS_GARDENS_FRUIT_KHATA_SQL, migrate::trips_hook),
        M::up_with_hook(SIMPLE_PARTY_CODES_SQL, migrate::codes_hook),
        M::up_with_hook(TRIP_AND_GARDEN_CODES_SQL, migrate::trip_names_hook),
        M::up(DATE_TO_SELL_AND_GARDEN_ARRIVALS_SQL),
        M::up(SHOP_SPLIT_SQL),
        M::up(DEAD_MARKS_SQL),
        M::up(EDIT_WINDOWS_SQL),
        M::up(ONE_TIME_PARTY_MARK_SQL),
        M::up(PARTS_ARE_SOFT_STATE_SQL),
        M::up(ASSISTANT_QUESTIONS_SQL),
        M::up(SPEED_INDEXES_SQL),
        M::up(ASSISTANT_TIMING_SQL),
        M::up(ASSISTANT_TIME_LIMIT_SQL),
        M::up(SALE_GRADE_SQL),
    ])
}

fn connection(path: &Path) -> Connection {
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

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

fn schema(conn: &Connection) -> Vec<String> {
    conn.prepare(
        "SELECT type || ':' || name || ':' || COALESCE(sql, '') FROM sqlite_master ORDER BY type, name",
    )
    .unwrap()
    .query_map([], |r| r.get(0))
    .unwrap()
    .map(Result::unwrap)
    .collect()
}

/// Every row of every table, as text, by column position, in rowid order.
fn all_rows(conn: &Connection, skip_tables: &[&str]) -> Vec<(String, String)> {
    let names: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    names
        .into_iter()
        .filter(|n| !skip_tables.contains(&n.as_str()))
        .map(|name| {
            let columns: Vec<String> = conn
                .prepare(&format!(
                    "SELECT name FROM pragma_table_info('{name}') ORDER BY cid"
                ))
                .unwrap()
                .query_map([], |r| r.get(0))
                .unwrap()
                .map(Result::unwrap)
                .collect();
            let joined = columns
                .iter()
                .map(|c| format!("quote(\"{c}\")"))
                .collect::<Vec<_>>()
                .join(" || '|' || ");
            let rows: Vec<String> = conn
                .prepare(&format!("SELECT {joined} FROM \"{name}\" ORDER BY rowid"))
                .unwrap()
                .query_map([], |r| r.get(0))
                .unwrap()
                .map(Result::unwrap)
                .collect();
            (name, rows.join("\n"))
        })
        .collect()
}

fn hashes(conn: &Connection) -> Vec<(i64, String)> {
    conn.prepare("SELECT id, chain_hash FROM journal_entries WHERE status = 'posted' ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn balances(conn: &Connection) -> Vec<(i64, i64, i64)> {
    conn.prepare(
        "SELECT account_id, COALESCE(SUM(dr_paisa), 0), COALESCE(SUM(cr_paisa), 0)
         FROM journal_lines GROUP BY account_id ORDER BY account_id",
    )
    .unwrap()
    .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
    .unwrap()
    .map(Result::unwrap)
    .collect()
}

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

fn account(conn: &Connection, code: &str) -> i64 {
    conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
        r.get(0)
    })
    .unwrap()
}

/// Posts a general entry as the app of version 19 did, with the person key on the line of the shared
/// accounts 0030 and 0031 and on any account in `person_on`, in the bytes of the chain (copied from
/// the migration tests, so a hash that verifies after the upgrade proves the format did not move).
fn legacy_post(
    conn: &Connection,
    existing: Option<i64>,
    day: &str,
    from: &str,
    to: &str,
    rupees: i64,
    person: Option<&str>,
) -> i64 {
    let person_of = |code: &str| -> Option<&str> {
        if code == "0030" || code == "0031" {
            person
        } else {
            None
        }
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
    let lines = [(from, rupees * 100, 0_i64), (to, 0, rupees * 100)];
    for (index, (code, dr, cr)) in lines.iter().enumerate() {
        match person_of(code) {
            None => conn.execute(
                "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, ?2, ?3, ?4, ?5)",
                (entry, i64::try_from(index).unwrap() + 1, account(conn, code), dr, cr),
            ),
            Some(key) => conn.execute(
                "INSERT INTO journal_lines (entry_id, line_no, account_id, pukhta_person_key, dr_paisa, cr_paisa) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (entry, i64::try_from(index).unwrap() + 1, account(conn, code), key, dr, cr),
            ),
        }
        .unwrap();
    }
    let (previous, last): (String, i64) = conn
        .query_row(
            "SELECT COALESCE((SELECT chain_hash FROM journal_entries WHERE status = 'posted' ORDER BY id DESC LIMIT 1), ?1),
                    COALESCE((SELECT id FROM journal_entries WHERE status = 'posted' ORDER BY id DESC LIMIT 1), 0)",
            [mandi_db::audit::GENESIS_HASH],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(last < entry);
    let mut bytes = Vec::new();
    put(&mut bytes, Some(&previous));
    put(&mut bytes, Some(&entry.to_string()));
    put(&mut bytes, Some(day));
    put(&mut bytes, Some("general"));
    put(&mut bytes, Some("voucher"));
    put(&mut bytes, Some("1"));
    put(&mut bytes, None);
    put(&mut bytes, None);
    put(&mut bytes, Some(&(rupees * 100).to_string()));
    put(&mut bytes, Some("2"));
    for (index, (code, dr, cr)) in lines.iter().enumerate() {
        put(&mut bytes, Some(&(index + 1).to_string()));
        put(&mut bytes, Some(&account(conn, code).to_string()));
        put(&mut bytes, person_of(code));
        put(&mut bytes, Some(&dr.to_string()));
        put(&mut bytes, Some(&cr.to_string()));
        put(&mut bytes, Some("0"));
        put(&mut bytes, None);
    }
    let hash = blake3::keyed_hash(&[7_u8; 32], &bytes).to_hex().to_string();
    conn.execute(
        "UPDATE journal_entries SET status = 'posted', total_paisa = ?1, chain_hash = ?2 WHERE id = ?3",
        (rupees * 100, &hash, entry),
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

/// One sale line of the version 19 file, with the typed person columns and the stored word.
#[allow(clippy::too_many_arguments)]
fn legacy_line(
    conn: &Connection,
    f: &common::Fixture,
    line_no: i64,
    entry: i64,
    buyer: i64,
    total_rupees: i64,
    payment: &str,
    active: bool,
    person: Option<(&str, &str, Option<&str>, &str)>,
) -> i64 {
    let total = total_rupees * 100;
    let (ur, en, phone, key) = match person {
        Some((ur, en, phone, key)) => (Some(ur), Some(en), phone, Some(key)),
        None => (None, None, None, None),
    };
    conn.execute(
        "INSERT INTO bikri_sales (bikri_id, lot_id, line_no, journal_entry_id, entry_date, buyer_account_id, qty,
            seller_rate_paisa, seller_amount_paisa, seller_comm_method, seller_comm_value_bps, seller_comm_paisa,
            buyer_rate_paisa, buyer_value_paisa, buyer_comm_method, buyer_comm_value_bps, buyer_comm_paisa,
            laga_method, laga_value_paisa, laga_paisa, buyer_total_paisa, payment_type, is_active, ended,
            pukhta_name_ur, pukhta_name_en, pukhta_phone, pukhta_person_key)
         VALUES (?1, ?2, ?3, ?4, '2026-10-10', ?5, 1, ?6, ?6, 'percent', 0, 0, ?6, ?6, 'percent', 0, 0,
            'per_unit', 0, 0, ?6, ?7, ?8, CASE WHEN ?8 = 1 THEN NULL ELSE 'replaced' END, ?9, ?10, ?11, ?12)",
        params![f.bikri, f.lot, line_no, entry, buyer, total, payment, i64::from(active), ur, en, phone, key],
    )
    .unwrap();
    conn.last_insert_rowid()
}

struct Old {
    /// (sale id, buyer total paisa, payment type).
    lines: Vec<(i64, i64, &'static str)>,
    /// The people expected in the new list: (key, name_ur, name_en, phone).
    people: Vec<(String, String, String, Option<String>)>,
}

/// Builds the hostile file of version 19 in `path` and returns what the test must find after.
fn build_v19(path: &Path, backups: &Path) -> Old {
    let mut conn = connection(path);
    migrate::migrate_with(&released_version_19(), &mut conn, backups, &clock()).unwrap();
    assert_eq!(user_version(&conn), 19);
    let f = common::fixture(&conn);
    let akram = account(&conn, "0020");
    let khata = account(&conn, "0030");
    let old_seller = account(&conn, "0031");
    // The fixture's draft entry posts its first line (a credit line of 12,880).
    legacy_post(
        &conn,
        Some(f.entry),
        "2026-10-10",
        "0020",
        "04010001",
        12_880,
        None,
    );
    let mut lines = vec![(f.sale, 1_288_000_i64, "credit")];
    let mut people: Vec<(String, String, String, Option<String>)> = Vec::new();

    // Old پختہ party accounts (decision C9: they stay as plain buyers and sellers) with postings.
    let control_buyer: i64 = conn
        .query_row(
            "SELECT id FROM account_controls WHERE code = '0304'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let control_seller: i64 = conn
        .query_row(
            "SELECT id FROM account_controls WHERE code = '03011'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    for (control, code, serial, party, ur, en) in [
        (
            control_buyer,
            "0032",
            2_i64,
            "buyer",
            "پختہ گاہک",
            "Old One Time Buyer",
        ),
        (
            control_seller,
            "0033",
            3,
            "seller",
            "پختہ بیوپاری",
            "Old One Time Seller",
        ),
    ] {
        conn.execute(
            "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur, search_en,
                 search_phonetic, is_pukhta_party)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?5, lower(?6), lower(?6), 1)",
            params![control, code, serial, party, ur, en],
        )
        .unwrap();
    }
    legacy_post(&conn, None, "2026-10-10", "0032", "04010001", 4_000, None);
    legacy_post(&conn, None, "2026-10-10", "01010001", "0033", 1_500, None);

    // One-time people on 0030 and 0031. (key, ur, en, phone typed, paid kind, rupees, active)
    #[allow(clippy::type_complexity)]
    let cases: Vec<(&str, String, &str, Option<&str>, &str, i64, bool)> = vec![
        (
            "شاہد|03000000000",
            "شاہد".into(),
            "Shahid",
            Some("0300-0000000"),
            "cash",
            12_800,
            true,
        ),
        // The same person typed again later, with another English spelling: the later one wins.
        (
            "شاہد|03000000000",
            "شاہد".into(),
            "Shahid M",
            Some("03000000000"),
            "cash",
            3_000,
            true,
        ),
        (
            "ناصر|03001111111",
            "ناصر".into(),
            "Nasir",
            Some("  03001111111  "),
            "credit",
            2_000,
            true,
        ),
        ("کمال|", "کمال".into(), "Kamal", None, "credit", 1_000, true),
        (
            "کمال|",
            "کمال".into(),
            "Kamal",
            Some("   "),
            "cash",
            500,
            false,
        ),
        ("نامعلوم|", "نامعلوم".into(), "", None, "credit", 700, true),
        (
            "خالی|",
            String::new(),
            "Empty Urdu",
            None,
            "cash",
            600,
            false,
        ),
        (
            "زیرو\u{200b}|0300",
            "زیرو\u{200b}".into(),
            "Zero Width",
            Some("0300"),
            "cash",
            400,
            true,
        ),
        (
            "لمبا|",
            "ل".repeat(5_000),
            "Long",
            None,
            "credit",
            300,
            true,
        ),
    ];
    let mut line_no = 2;
    for (key, ur, en, phone, kind, rupees, active) in cases {
        let entry = legacy_post(
            &conn,
            None,
            "2026-10-10",
            "0030",
            "01010001",
            rupees,
            Some(key),
        );
        let id = legacy_line(
            &conn,
            &f,
            line_no,
            entry,
            khata,
            rupees,
            kind,
            active,
            Some((&ur, en, phone, key)),
        );
        line_no += 1;
        lines.push((
            id,
            rupees * 100,
            if kind == "cash" { "cash" } else { "credit" },
        ));
        // Expected in the list: the last typed names of each key, trimmed; a nameless person is left out.
        if !(ur.trim().is_empty() || en.trim().is_empty()) {
            let phone = phone
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(str::to_owned);
            if let Some(slot) = people.iter_mut().find(|p| p.0 == key) {
                *slot = (
                    key.to_owned(),
                    ur.trim().to_owned(),
                    en.trim().to_owned(),
                    phone,
                );
            } else {
                people.push((
                    key.to_owned(),
                    ur.trim().to_owned(),
                    en.trim().to_owned(),
                    phone,
                ));
            }
        } else if let Some(i) = people.iter().position(|p| p.0 == key) {
            // A later nameless line of a known person does not remove him; his names stay as they were.
            let _ = i;
        }
    }
    // Old lines on the closed seller account 0031, and an ordinary cash line and a cancelled credit line.
    legacy_post(
        &conn,
        None,
        "2026-10-11",
        "01010001",
        "0031",
        3_000,
        Some("بشیر|03002222222"),
    );
    let e = legacy_post(&conn, None, "2026-10-11", "0020", "04010001", 5_000, None);
    let id = legacy_line(&conn, &f, line_no, e, akram, 5_000, "cash", false, None);
    lines.push((id, 500_000, "cash"));
    line_no += 1;
    let e = legacy_post(&conn, None, "2026-10-12", "0020", "04010001", 3_000, None);
    let id = legacy_line(&conn, &f, line_no, e, akram, 3_000, "credit", false, None);
    lines.push((id, 300_000, "credit"));
    // A seller-side person on the lot (old column, kept and renamed), an analytics row with a key.
    conn.execute(
        "UPDATE bikri_lots SET pukhta_name_ur = 'بشیر', pukhta_name_en = 'Bashir', pukhta_phone = '03002222222',
             pukhta_person_key = 'بشیر|03002222222' WHERE id = ?1",
        [f.lot],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO analytics_facts (journal_entry_id, entry_date, year, month, bikri_id, lot_id, sale_id, item_id,
            seller_account_id, buyer_account_id, pukhta_person_key, qty, seller_amount_paisa, seller_comm_paisa,
            buyer_value_paisa, buyer_comm_paisa, laga_paisa, sign)
         VALUES (?1, '2026-10-10', 2026, 10, ?2, ?3, ?4, ?5, ?6, ?7, 'شاہد|03000000000', 1, 100, 0, 100, 0, 0, 1)",
        params![f.entry, f.bikri, f.lot, f.sale, f.item, f.seller, khata],
    )
    .unwrap();
    // Dead-amount marks: one on a person of 0030 and one on an ordinary buyer.
    conn.execute(
        "INSERT INTO dead_marks (account_id, pukhta_person_key, entry_date, reason, owed_paisa, after_entry_id)
         VALUES (?1, 'ناصر|03001111111', '2026-10-12', 'died', 200000, ?2)",
        params![khata, f.entry],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO dead_marks (account_id, entry_date, reason, owed_paisa, after_entry_id)
         VALUES (?1, '2026-10-12', 'left', 128800, ?2)",
        params![akram, f.entry],
    )
    .unwrap();
    let _ = old_seller;
    Old { lines, people }
}

/// What must not move when the upgrade runs: every table but the ones that change on purpose.
const CHANGES_ON_PURPOSE: &[&str] = &["accounts", "dead_marks", "chukta_buyers", "bikri_sales"];

fn assert_upgrade(path: &Path, backups: &Path, old: &Old) {
    let mut conn = connection(path);
    let rows_before = all_rows(&conn, CHANGES_ON_PURPOSE);
    let hashes_before = hashes(&conn);
    let balances_before = balances(&conn);
    let sales_before: Vec<String> = conn
        .prepare("SELECT quote(id) || quote(buyer_total_paisa) || quote(payment_type) || quote(is_active) || quote(version) || quote(updated_at) || quote(ended) FROM bikri_sales ORDER BY id")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let other_accounts = |conn: &Connection| -> Vec<String> {
        conn.prepare(
            "SELECT quote(id) || quote(code) || quote(name_ur) || quote(name_en) || quote(search_ur) || quote(is_active)
                    || quote(opening_dr_paisa) || quote(opening_cr_paisa) || quote(is_system)
             FROM accounts WHERE code NOT IN ('0030', '0031') ORDER BY id",
        )
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
    };
    let accounts_before = other_accounts(&conn);
    let shared_lines_before = count(
        &conn,
        "SELECT COUNT(*) FROM journal_lines l JOIN accounts a ON a.id = l.account_id WHERE a.code IN ('0030', '0031')",
    );
    assert!(
        shared_lines_before >= 10,
        "the fixture has too few lines on 0030 and 0031"
    );
    super_verify_chain(&conn);

    let outcome = migrate::migrate(&mut conn, backups, &clock()).unwrap();
    assert_eq!((outcome.from, outcome.to), (19, 21));
    let copy = outcome.copy.expect("a copy is taken before an upgrade");
    assert!(copy.exists());
    assert_eq!(user_version(&conn), 21);

    // Nothing moved.
    assert_eq!(hashes(&conn), hashes_before, "a chain hash moved");
    super_verify_chain(&conn);
    assert_eq!(
        all_rows(&conn, CHANGES_ON_PURPOSE),
        rows_before,
        "a row moved"
    );
    assert_eq!(balances(&conn), balances_before, "a balance moved");
    assert_eq!(
        other_accounts(&conn),
        accounts_before,
        "another account moved"
    );
    let sales_after: Vec<String> = conn
        .prepare("SELECT quote(id) || quote(buyer_total_paisa) || quote(payment_type) || quote(is_active) || quote(version) || quote(updated_at) || quote(ended) FROM bikri_sales ORDER BY id")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(sales_after, sales_before, "a stored sale line changed");
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    let fk: i64 = count(&conn, "SELECT COUNT(*) FROM pragma_foreign_key_check");
    assert_eq!(fk, 0, "foreign key violations after the upgrade");

    // paid = total for a cash line, 0 for a credit line, whatever the state of the line.
    for (id, total, payment) in &old.lines {
        let paid: i64 = conn
            .query_row(
                "SELECT paid_paisa FROM bikri_sales WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            paid,
            if *payment == "cash" { *total } else { 0 },
            "line {id}"
        );
    }

    // The list of چکتا buyers: exactly the people with both names, as last typed, phone trimmed.
    let list: Vec<(String, String, String, Option<String>)> = conn
        .prepare("SELECT person_key, name_ur, name_en, phone FROM chukta_buyers ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(list.len(), old.people.len(), "{list:?}");
    for person in &old.people {
        assert!(list.contains(person), "missing {person:?} in {list:?}");
    }
    // The old accounts: 0030 renamed and active, 0031 closed, the old party accounts plain and active.
    let name = |code: &str| -> (String, i64, i64) {
        conn.query_row(
            "SELECT name_en, is_active, is_chukta_khata FROM accounts WHERE code = ?1",
            [code],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap()
    };
    assert_eq!(name("0030"), ("Chukta Khata".into(), 1, 1));
    assert_eq!(name("0031"), ("Old Closed Account".into(), 0, 1));
    assert_eq!(name("0032").1, 1);
    assert_eq!(name("0033").1, 1);
    // The old open amounts on 0030 and 0031 are still in the books and in the trial balance.
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM journal_lines l JOIN accounts a ON a.id = l.account_id WHERE a.code IN ('0030', '0031')"
        ),
        shared_lines_before
    );
    let (dr, cr): (i64, i64) = conn
        .query_row(
            "SELECT SUM(dr_paisa), SUM(cr_paisa) FROM journal_lines",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(dr, cr, "the trial balance moved");
    // The mark on a person ends; the mark on a buyer stays.
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM dead_marks WHERE is_active = 1"),
        1
    );
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM dead_marks"), 2);
    // The triggers fire on the new names.
    for sql in [
        "UPDATE bikri_sales SET paid_paisa = 0 WHERE payment_type = 'cash'",
        "UPDATE bikri_sales SET chukta_name_en = 'x' WHERE chukta_name_en IS NOT NULL",
        "UPDATE journal_lines SET chukta_person_key = 'x|' WHERE chukta_person_key IS NOT NULL",
        "UPDATE accounts SET was_one_time_party = 0 WHERE was_one_time_party = 1",
        "DELETE FROM chukta_buyers",
        "DELETE FROM bikri_sales",
    ] {
        assert!(conn.execute(sql, []).is_err(), "{sql} must be refused");
    }

    // Running again is a no-op and takes no copy.
    let again = migrate::migrate(&mut conn, backups, &clock()).unwrap();
    assert_eq!((again.from, again.to), (21, 21));
    assert!(again.copy.is_none());
}

fn super_verify_chain(conn: &Connection) {
    // A file of version 19 names the person column `pukhta_person_key`; the code of today verifies
    // with the new name. So verify only when the new name is there, else compare through a view of
    // the old chain by the migration tests (they do the rename dance). Here: only after the upgrade.
    let has_new: i64 = count(
        conn,
        "SELECT COUNT(*) FROM pragma_table_info('journal_lines') WHERE name = 'chukta_person_key'",
    );
    if has_new == 1 {
        mandi_db::ledger::verify_journal_chain(conn, &common::test_key()).unwrap();
    }
}

#[test]
fn hostile_old_data_goes_from_19_to_21_in_one_run() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mandi.db");
    let backups = dir.path().join("backups");
    let old = build_v19(&path, &backups);
    assert_upgrade(&path, &backups, &old);
}

#[test]
fn a_failure_in_the_last_step_puts_the_whole_file_back_and_a_rerun_then_works() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mandi.db");
    let backups = dir.path().join("backups");
    let old = build_v19(&path, &backups);
    let mut conn = connection(&path);
    // A stray trigger that stops any change of a sale line: migration 21's UPDATE then fails, after
    // migration 20 (the renames, the new table, the hook) ran in the same upgrade.
    conn.execute_batch(
        "CREATE TRIGGER stray_stop BEFORE UPDATE ON bikri_sales BEGIN SELECT RAISE(ABORT, 'ERR_STRAY'); END;",
    )
    .unwrap();
    let rows_before = all_rows(&conn, &[]);
    let schema_before = schema(&conn);
    let hashes_before = hashes(&conn);
    let error = migrate::migrate(&mut conn, &backups, &clock()).unwrap_err();
    assert!(matches!(error, DbError::MigrationFailed(_)), "{error:?}");
    assert_eq!(
        user_version(&conn),
        19,
        "the version moved although the upgrade failed"
    );
    assert_eq!(
        schema(&conn),
        schema_before,
        "the schema is not the old one"
    );
    assert_eq!(
        all_rows(&conn, &[]),
        rows_before,
        "a row changed in a failed upgrade"
    );
    assert_eq!(hashes(&conn), hashes_before);
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM pragma_table_info('bikri_sales') WHERE name = 'pukhta_person_key'"
        ),
        1
    );
    // The copy taken before is whole and at version 19.
    let copies: Vec<PathBuf> = std::fs::read_dir(&backups)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("pre-migration-19-21"))
        })
        .collect();
    assert_eq!(copies.len(), 1, "{copies:?}");
    let copy = Connection::open(&copies[0]).unwrap();
    assert_eq!(user_version(&copy), 19);
    let integrity: String = copy
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    drop(copy);
    // The fault is taken away; the upgrade now runs and everything is as in the clean run.
    conn.execute_batch("DROP TRIGGER stray_stop;").unwrap();
    assert_upgrade_after_failure(&mut conn, &backups, &old);
}

fn assert_upgrade_after_failure(conn: &mut Connection, backups: &Path, old: &Old) {
    let outcome = migrate::migrate(conn, backups, &clock()).unwrap();
    assert_eq!((outcome.from, outcome.to), (19, 21));
    assert_eq!(user_version(conn), 21);
    mandi_db::ledger::verify_journal_chain(conn, &common::test_key()).unwrap();
    for (id, total, payment) in &old.lines {
        let paid: i64 = conn
            .query_row(
                "SELECT paid_paisa FROM bikri_sales WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            paid,
            if *payment == "cash" { *total } else { 0 },
            "line {id}"
        );
    }
    assert_eq!(
        count(conn, "SELECT COUNT(*) FROM chukta_buyers"),
        old.people.len() as i64
    );
}

#[test]
fn a_failure_in_the_first_step_puts_the_file_back() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mandi.db");
    let backups = dir.path().join("backups");
    let old = build_v19(&path, &backups);
    let mut conn = connection(&path);
    conn.execute_batch("CREATE TABLE chukta_buyers (id INTEGER PRIMARY KEY, stray TEXT);")
        .unwrap();
    conn.execute("INSERT INTO chukta_buyers (stray) VALUES ('keep me')", [])
        .unwrap();
    let rows_before = all_rows(&conn, &[]);
    let schema_before = schema(&conn);
    let error = migrate::migrate(&mut conn, &backups, &clock()).unwrap_err();
    assert!(matches!(error, DbError::MigrationFailed(_)), "{error:?}");
    assert_eq!(user_version(&conn), 19);
    assert_eq!(schema(&conn), schema_before);
    assert_eq!(all_rows(&conn, &[]), rows_before);
    // Take the stray table away: the upgrade then works.
    conn.execute_batch("DROP TABLE chukta_buyers;").unwrap();
    assert_upgrade_after_failure(&mut conn, &backups, &old);
}

// ---- Kill during the upgrade ---------------------------------------------------------------------

const CHILD_PATH: &str = "D8_MIGRATION_CHILD_PATH";

/// Not a test. The parent starts this same binary with the variable set and kills it.
#[test]
fn child_process_for_the_migration_kill_test() {
    let Ok(path) = std::env::var(CHILD_PATH) else {
        return;
    };
    let path = PathBuf::from(path);
    let backups = path.parent().unwrap().join("backups");
    let mut out = std::io::stdout().lock();
    writeln!(out, "\n@@START").unwrap();
    out.flush().unwrap();
    drop(out);
    let mut conn = connection(&path);
    let result = migrate::migrate(&mut conn, &backups, &clock());
    let mut out = std::io::stdout().lock();
    match result {
        Ok(_) => writeln!(out, "\n@@DONE").unwrap(),
        Err(e) => writeln!(out, "\n@@FAILED {e}").unwrap(),
    }
    out.flush().unwrap();
}

#[test]
fn killing_the_process_during_the_upgrade_40_times_never_leaves_a_half_upgraded_file() {
    // The reference: one clean upgrade of the same file, to compare and to time.
    let ref_dir = tempdir().unwrap();
    let ref_path = ref_dir.path().join("mandi.db");
    let old = build_v19(&ref_path, &ref_dir.path().join("backups"));
    let template = std::fs::read(&ref_path).unwrap();
    // The file is in WAL mode: copy any wal too, by checkpointing before the copy.
    {
        let conn = connection(&ref_path);
        conn.pragma_update(None, "wal_checkpoint", "TRUNCATE").ok();
    }
    let template = std::fs::read(&ref_path).unwrap_or(template);
    let started = Instant::now();
    {
        let mut conn = connection(&ref_path);
        migrate::migrate(&mut conn, &ref_dir.path().join("backups"), &clock()).unwrap();
    }
    let t_total = started.elapsed();
    let reference_rows = {
        let conn = connection(&ref_path);
        // The tables that carry money and the new paid column, without any time stamp the upgrade makes.
        all_rows(
            &conn,
            &["chukta_buyers", "dead_marks", "audit_log", "settings"],
        )
    };
    println!("a clean upgrade takes {t_total:?}");
    let mut stages = std::collections::BTreeMap::<String, u32>::new();
    let mut state = 0xD8_0021_u64;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        state >> 33
    };
    for round in 1..=40 {
        let dir = tempdir().unwrap();
        let path = dir.path().join("mandi.db");
        std::fs::write(&path, &template).unwrap();
        let mut process = Command::new(std::env::current_exe().unwrap())
            .args([
                "child_process_for_the_migration_kill_test",
                "--exact",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD_PATH, &path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdout = process.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if let Some(line) = line.strip_prefix("@@")
                    && tx.send(line.to_owned()).is_err()
                {
                    break;
                }
            }
        });
        // From the start of the process to a little after the end of a whole upgrade.
        let span = u64::try_from(t_total.as_micros()).unwrap_or(100_000) + 60_000;
        std::thread::sleep(Duration::from_micros(next() % (span * 2)));
        let _ = process.kill();
        let _ = process.wait();
        let mut last = "NOTHING_YET".to_owned();
        while let Ok(line) = lines.recv_timeout(Duration::from_millis(200)) {
            last = line;
        }
        *stages
            .entry(last.split(' ').next().unwrap_or("").to_owned())
            .or_default() += 1;

        // After the kill: the file opens, passes its check, and is at 19 (untouched) or 21 (whole).
        let mut conn = connection(&path);
        let integrity: String = conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok", "round {round}");
        let version = user_version(&conn);
        assert!(
            version == 19 || version == 21,
            "round {round}: version {version}"
        );
        if version == 19 {
            // Not a trace of migration 20 or 21.
            assert_eq!(
                count(
                    &conn,
                    "SELECT COUNT(*) FROM pragma_table_info('bikri_sales') WHERE name IN ('paid_paisa', 'chukta_person_key')"
                ),
                0,
                "round {round}: half of an upgrade is in a file at version 19"
            );
            assert_eq!(
                count(
                    &conn,
                    "SELECT COUNT(*) FROM sqlite_master WHERE name = 'chukta_buyers'"
                ),
                0
            );
        }
        // Run the upgrade again (as the next start of the app does): it ends at 21 with the same result.
        let outcome = migrate::migrate(&mut conn, &dir.path().join("backups2"), &clock()).unwrap();
        assert_eq!(outcome.to, 21);
        assert_eq!(user_version(&conn), 21);
        mandi_db::ledger::verify_journal_chain(&conn, &common::test_key())
            .unwrap_or_else(|e| panic!("round {round}: {e:?}"));
        assert_eq!(
            all_rows(
                &conn,
                &["chukta_buyers", "dead_marks", "audit_log", "settings"]
            ),
            reference_rows,
            "round {round}: the result is not the clean result"
        );
        assert_eq!(
            count(&conn, "SELECT COUNT(*) FROM chukta_buyers"),
            old.people.len() as i64
        );
    }
    println!("kill stages: {stages:?}");
}

// ---- A real backup --------------------------------------------------------------------------------

#[test]
fn a_real_backup_copy_migrates_without_moving_a_number() {
    let Ok(source) = std::env::var("D8_REAL_COPY") else {
        println!("D8_REAL_COPY is not set: the real-copy check is skipped");
        return;
    };
    let dir = tempdir().unwrap();
    let path = dir.path().join("mandi.db");
    std::fs::copy(&source, &path).unwrap();
    let mut conn = connection(&path);
    let before_version = user_version(&conn);
    let rows_before = all_rows(&conn, CHANGES_ON_PURPOSE);
    let hashes_before = hashes(&conn);
    let balances_before = balances(&conn);
    let lines_before: Vec<(i64, i64, String)> = conn
        .prepare("SELECT id, buyer_total_paisa, payment_type FROM bikri_sales ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    println!(
        "real copy: version {before_version}, {} entries, {} sale lines",
        hashes_before.len(),
        lines_before.len()
    );
    let outcome = migrate::migrate(&mut conn, &dir.path().join("backups"), &clock()).unwrap();
    println!("real copy: migrated {} -> {}", outcome.from, outcome.to);
    assert_eq!(user_version(&conn), 21);
    assert_eq!(hashes(&conn), hashes_before, "a chain hash moved");
    assert_eq!(
        all_rows(&conn, CHANGES_ON_PURPOSE),
        rows_before,
        "a row moved"
    );
    assert_eq!(balances(&conn), balances_before, "a balance moved");
    for (id, total, payment) in lines_before {
        let paid: i64 = conn
            .query_row(
                "SELECT paid_paisa FROM bikri_sales WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(paid, if payment == "cash" { total } else { 0 }, "line {id}");
    }
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM pragma_foreign_key_check"),
        0
    );
}
