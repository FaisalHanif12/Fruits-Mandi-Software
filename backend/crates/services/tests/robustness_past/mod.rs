//! The 10 years of بکری for the plan 72 speed and attack tests (PRD 12.7, P11; NFR-SCL-1, NFR-SPD-24).
//! Owner: `robustness`.
//!
//! The NFR-SCL-1 volume of `benches/support` has journal lines but no بکری. This builds them on a private
//! copy of that file, kept in `backend/target/perf/nfr_scl1_past` (the `MANDI_PERF_DIR` variable moves it),
//! through plain SQL with the same tables, indexes and triggers as the app: 3,653 days from 11-10-2016 to
//! 10-10-2026, 40 بکری a day, 1 or 2 lots each, about 8 sale lines each, so about 146,000 بکری, 219,000
//! lots and 1.2 million sale lines (NFR-SCL-1). 2% of the بکری are cancelled. Sellers, items and buyers
//! are spread over the real accounts of the volume. Totals are whole rupees and add up exactly (the
//! `expected_*` helpers recompute them from the sale lines, apart from the code under test).
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::{Duration as Days, NaiveDate};
use mandi_services::db::rusqlite::{Connection, params};

/// A change here rebuilds the cached file.
const VERSION: &str = "past-v1";
const LINES_PER_DAY_ENTRIES: u64 = 40;

pub fn first_day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2016, 10, 11).unwrap_or_default()
}

pub fn last_day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 10).unwrap_or_default()
}

fn dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MANDI_PERF_DIR") {
        return PathBuf::from(dir).join("past");
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("perf")
        .join("nfr_scl1_past")
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

/// What was written, so a test knows the busy days and a seller and an item that have بکری.
#[derive(Debug, Clone)]
pub struct PastVolume {
    pub dir: PathBuf,
    pub entries: i64,
    pub lots: i64,
    pub lines: i64,
    /// A seller and an item that sold a lot on many days (for the filters).
    pub seller_id: i64,
    pub item_id: i64,
}

/// The cached data folder of the 10 years, built from the NFR-SCL-1 volume if it is not there yet.
/// `volume` is the folder of that volume (`benches/support::ensure_volume().dir`).
pub fn ensure(volume: &Path) -> PastVolume {
    let dir = dir();
    let marker = dir.join("past.ok");
    let expected = format!("{VERSION} {LINES_PER_DAY_ENTRIES}");
    if std::fs::read_to_string(&marker).unwrap_or_default().trim() != expected {
        println!(
            "past data: building the 10 years of بکری in {}",
            dir.display()
        );
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("data")).expect("data folder");
        std::fs::copy(
            volume.join("data").join("mandi.db"),
            dir.join("data").join("mandi.db"),
        )
        .expect("copy of the volume");
        // The start-up path migrates the file to the version of this build and signs the Owner in.
        {
            let (env, _actor) = open_env(&dir);
            drop(env);
        }
        build(&dir);
        std::fs::write(&marker, &expected).expect("marker");
    }
    let conn = Connection::open(dir.join("data").join("mandi.db")).expect("open");
    let count = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap_or(0) };
    // The seller and item with the most lots: the filters of the speed tests.
    let (seller_id, item_id): (i64, i64) = conn
        .query_row(
            "SELECT seller_account_id, item_id FROM bikri_lots GROUP BY seller_account_id, item_id
             ORDER BY COUNT(*) DESC, seller_account_id LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap_or((0, 0));
    PastVolume {
        dir,
        entries: count("SELECT COUNT(*) FROM bikri_entries"),
        lots: count("SELECT COUNT(*) FROM bikri_lots"),
        lines: count("SELECT COUNT(*) FROM bikri_sales"),
        seller_id,
        item_id,
    }
}

/// The test environment on a folder with a data file (the Owner is made on first use).
pub fn open_env(
    dir: &Path,
) -> (
    mandi_services::testing::TestEnv,
    mandi_services::auth::Actor,
) {
    use mandi_services::{Secret, auth};
    let env = mandi_services::testing::TestEnv::new(dir).expect("open the data");
    let status = auth::status(&env.ctx).expect("status");
    if status.first_run {
        // The volume has its Owner already; a fresh folder does not.
        panic!("the past volume has no Owner: {status:?}");
    }
    auth::login(&env.ctx, "owner", &Secret::new("secret1")).expect("sign in");
    let actor = auth::require(&env.ctx).expect("actor");
    (env, actor)
}

fn build(dir: &Path) {
    let started = Instant::now();
    let conn = Connection::open(dir.join("data").join("mandi.db")).expect("open");
    conn.execute_batch(
        "PRAGMA synchronous = OFF; PRAGMA temp_store = MEMORY; PRAGMA cache_size = -400000;
         PRAGMA foreign_keys = ON;",
    )
    .expect("pragmas");
    let ids = |sql: &str| -> Vec<i64> {
        let mut st = conn.prepare(sql).expect("prepare");
        st.query_map([], |r| r.get(0))
            .expect("query")
            .map(|r| r.expect("row"))
            .collect()
    };
    let sellers = ids(
        "SELECT a.id FROM accounts a JOIN account_controls c ON c.id = a.control_id WHERE c.code = '03011' AND a.is_system = 0 ORDER BY a.id",
    );
    let buyers = ids(
        "SELECT a.id FROM accounts a JOIN account_controls c ON c.id = a.control_id WHERE c.code = '0304' AND a.is_system = 0 ORDER BY a.id",
    );
    let items = ids("SELECT id FROM items ORDER BY id");
    let packings = ids("SELECT id FROM packings ORDER BY id");
    assert!(!sellers.is_empty() && !buyers.is_empty() && !items.is_empty() && !packings.is_empty());
    // One draft journal entry that every sale line points at: the speed tests read, they never post.
    conn.execute(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, status, total_paisa)
         VALUES ('2016-10-11', 'bikri_first', 'bikri', 1, 'draft', 0)",
        [],
    )
    .expect("draft entry");
    let draft_entry = conn.last_insert_rowid();
    let first_number: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(number), 0) + 1 FROM bikri_entries",
            [],
            |r| r.get(0),
        )
        .expect("number");
    let mut rng = Rng(0x0072_0000_5EED_0001);
    let days = (last_day() - first_day()).num_days().max(1);
    let mut number = first_number;
    // Counters per seller and item, so the lot counter trigger is not asked to invent them.
    let mut counters: std::collections::HashMap<(i64, i64), i64> = std::collections::HashMap::new();
    conn.execute_batch("BEGIN").expect("begin");
    {
        let mut entry_st = conn
            .prepare(
                "INSERT INTO bikri_entries (number, entry_date, bill_date, truck_no, status, cancel_reason, created_at, updated_at)
                 VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6, ?6)",
            )
            .expect("entry");
        let mut lot_st = conn
            .prepare(
                "INSERT INTO bikri_lots (bikri_id, lot_no, seller_account_id, item_id, packing_id, qty_received,
                    qty_sold, gross_paisa, net_paisa, lot_counter)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, ?7, ?8)",
            )
            .expect("lot");
        let mut sale_st = conn
            .prepare(
                "INSERT INTO bikri_sales (bikri_id, lot_id, line_no, journal_entry_id, entry_date, buyer_account_id, qty,
                    seller_rate_paisa, seller_amount_paisa, seller_comm_method, seller_comm_value_bps, seller_comm_paisa,
                    buyer_rate_paisa, buyer_value_paisa, buyer_comm_method, buyer_comm_value_bps, buyer_comm_paisa,
                    laga_method, laga_value_paisa, laga_paisa, buyer_total_paisa, rate_diff_paisa, payment_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'percent', 600, ?10, ?8, ?9, 'percent', 600, ?10,
                    'per_unit', 800, ?11, ?12, 0, 'credit')",
            )
            .expect("sale");
        let mut written = 0_u64;
        for d in 0..=days {
            let day = (first_day() + Days::days(d)).format("%Y-%m-%d").to_string();
            for n in 0..LINES_PER_DAY_ENTRIES {
                let cancelled = rng.below(50) == 0;
                let truck = if rng.below(10) < 8 {
                    Some(format!(
                        "LE{}-{}",
                        10 + rng.below(80),
                        1000 + rng.below(9000)
                    ))
                } else {
                    None
                };
                let time = format!("{day}T{:02}:{:02}:00.000Z", 4 + n % 14, rng.below(60));
                entry_st
                    .execute(params![
                        number,
                        day,
                        truck,
                        if cancelled { "cancelled" } else { "saved" },
                        if cancelled { Some("wrong entry") } else { None },
                        time,
                    ])
                    .expect("insert entry");
                let bikri_id = conn.last_insert_rowid();
                let lots = 1 + rng.below(2);
                let mut line_no = 0_i64;
                for lot_no in 1..=lots {
                    let seller = sellers[rng.below(sellers.len() as u64) as usize];
                    let item = items[rng.below(items.len() as u64) as usize];
                    let packing = packings[rng.below(packings.len() as u64) as usize];
                    let lines = 3 + rng.below(10);
                    let mut rows = Vec::new();
                    let (mut qty_total, mut gross) = (0_i64, 0_i64);
                    for _ in 0..lines {
                        let qty = 1 + rng.below(30) as i64;
                        let rate = 300 + rng.below(1_200) as i64;
                        let amount = qty * rate;
                        qty_total += qty;
                        gross += amount;
                        rows.push((
                            buyers[rng.below(buyers.len() as u64) as usize],
                            qty,
                            rate,
                            amount,
                        ));
                    }
                    let counter = counters.entry((seller, item)).or_insert(0);
                    *counter += 1;
                    lot_st
                        .execute(params![
                            bikri_id,
                            lot_no as i64,
                            seller,
                            item,
                            packing,
                            qty_total,
                            gross * 100,
                            *counter
                        ])
                        .expect("insert lot");
                    let lot_id = conn.last_insert_rowid();
                    for (buyer, qty, rate, amount) in rows {
                        line_no += 1;
                        let comm = (amount * 6 + 50) / 100;
                        let laga = qty * 8;
                        sale_st
                            .execute(params![
                                bikri_id,
                                lot_id,
                                line_no,
                                draft_entry,
                                day,
                                buyer,
                                qty,
                                rate * 100,
                                amount * 100,
                                comm * 100,
                                laga * 100,
                                (amount + comm + laga) * 100
                            ])
                            .expect("insert sale");
                    }
                }
                number += 1;
                written += 1;
            }
            if d % 200 == 199 {
                // Commit in chunks, so the log does not grow without end.
                drop(conn.execute_batch("COMMIT; BEGIN"));
                if d % 1_000 == 999 {
                    println!("past data: {written} entries after {:?}", started.elapsed());
                }
            }
        }
    }
    conn.execute_batch("COMMIT").expect("commit");
    let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
    let _ = conn.execute_batch("ANALYZE");
    println!("past data: built in {:?}", started.elapsed());
}

/// The count and the sums of the stored columns over a range of days, from plain SQL on a second
/// connection: what any list of the range must equal (cancelled بکری leave out the sums, as the list
/// does). Returns (count of all بکری, count of saved, gross_paisa, net_paisa, buyer_total_paisa).
pub fn expected(dir: &Path, from: &str, to: &str) -> (i64, i64, i64, i64, i64) {
    let conn = Connection::open(dir.join("data").join("mandi.db")).expect("open");
    let one = |sql: &str| -> i64 {
        conn.query_row(sql, params![from, to], |r| r.get(0))
            .expect("expected")
    };
    (
        one("SELECT COUNT(*) FROM bikri_entries WHERE entry_date BETWEEN ?1 AND ?2"),
        one("SELECT COUNT(*) FROM bikri_entries WHERE entry_date BETWEEN ?1 AND ?2 AND status = 'saved'"),
        one("SELECT COALESCE(SUM(l.gross_paisa), 0) FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id
             WHERE e.entry_date BETWEEN ?1 AND ?2 AND e.status = 'saved'"),
        one("SELECT COALESCE(SUM(l.net_paisa), 0) FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id
             WHERE e.entry_date BETWEEN ?1 AND ?2 AND e.status = 'saved'"),
        one("SELECT COALESCE(SUM(s.buyer_total_paisa), 0) FROM bikri_sales s JOIN bikri_entries e ON e.id = s.bikri_id
             WHERE e.entry_date BETWEEN ?1 AND ?2 AND e.status = 'saved' AND s.is_active = 1"),
    )
}

/// The per-day counts and gross of a range, from plain SQL (all بکری of the day; the gross of the saved ones).
pub fn expected_days(dir: &Path, from: &str, to: &str) -> Vec<(String, i64, i64)> {
    let conn = Connection::open(dir.join("data").join("mandi.db")).expect("open");
    let mut st = conn
        .prepare(
            "SELECT e.entry_date, COUNT(*),
                    COALESCE(SUM(CASE WHEN e.status = 'saved' THEN
                        (SELECT COALESCE(SUM(l.gross_paisa), 0) FROM bikri_lots l WHERE l.bikri_id = e.id) ELSE 0 END), 0)
             FROM bikri_entries e WHERE e.entry_date BETWEEN ?1 AND ?2
             GROUP BY e.entry_date ORDER BY e.entry_date DESC",
        )
        .expect("prepare");
    st.query_map(params![from, to], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .expect("query")
        .map(|r| r.expect("row"))
        .collect()
}
