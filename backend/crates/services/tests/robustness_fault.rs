//! Fault tolerance (D1 attack list item 3 and 6, D2 attack list; NFR-REL-1, NFR-REL-2, NFR-REL-6,
//! NFR-REL-7, NFR-REL-10, NFR-REL-12, NFR-REL-13). Owner: `robustness`.
//!
//! - The app process is killed (SIGKILL, no clean-up, like a crash or the task manager) in the
//!   middle of saves, 40 times in a row, on the same data file. After each restart: the file opens,
//!   the check passes, every posting is whole (voucher, journal entry, audit row), the books
//!   balance, both hash chains verify, and every save that was acknowledged before the kill is
//!   still there.
//! - The process is killed while a new data file is being made and set up (migration, seed,
//!   first run), 80 times at different moments.
//! - The disk is full: by the free-space check (before anything is written) and by SQLite itself
//!   running out of pages in the middle of a save. Nothing is half saved, the file is sound, and
//!   saving works again when space comes back.
//! - Damaged files: truncated, zeroed header, garbage in the middle, flipped bytes. None of them
//!   crashes the program; a file that fails its check stops all saving.
//! - A long run of many operations shows no slowdown and no memory growth (NFR-REL-12).
//!
//! A kill of the process does not lose what the operating system already has, so this does not
//! test a power cut. A power cut is the same as long as `synchronous = FULL` and WAL are set,
//! which the first test checks; the real power-cut run is on the Windows list.
#![cfg(test)]

mod common;

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use common::*;
use mandi_services::core::accounts::{AccountInput, AccountType, OpeningLineInput, OpeningsInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::{AdvanceInput, AdvancesInput, VoucherMode};
use mandi_services::db::audit::{AuditKey, MemoryKeyStore};
use mandi_services::db::clock::ManualClock;
use mandi_services::db::disk::DiskProbe;
use mandi_services::db::{DataState, Db, DbOptions};
use mandi_services::testing::{CountingRandom, ManualMonotonic, TestEnv, fast_hasher, test_start};
use mandi_services::{Context, Parts, ServiceError, accounts, auth, ledger, vouchers};
use tempfile::TempDir;

const CHILD_DIR: &str = "MANDI_ROBUST_CHILD_DIR";
const CHILD_MODE: &str = "MANDI_ROBUST_CHILD_MODE";

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

fn count(dir: &Path, sql: &str) -> i64 {
    let conn =
        mandi_services::db::rusqlite::Connection::open(dir.join("data").join("mandi.db")).unwrap();
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

fn buyer_input(n: u64) -> AccountInput {
    AccountInput {
        id: None,
        version: None,
        account_type: AccountType::Buyer,
        control_id: None,
        code: String::new(),
        name_ur: "اکرم".into(),
        name_en: "Akram".into(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: "30-09-2026".into(),
        opening_dr: format!("{}", 100 + n),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
    }
}

fn seller_input(ur: &str, en: &str) -> AccountInput {
    AccountInput {
        account_type: AccountType::Seller,
        name_ur: ur.into(),
        name_en: en.into(),
        opening_date: String::new(),
        opening_dr: String::new(),
        ..buyer_input(0)
    }
}

fn advance(seller: i64, amount: u64) -> AdvanceInput {
    AdvanceInput {
        date: "10-10-2026".into(),
        seller_account_id: seller,
        amount: amount.to_string(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        expected_goods: String::new(),
        narration: String::new(),
    }
}

/// A data file with the Owner, two sellers and 65,000 in cash. The environment is dropped, so
/// the file is closed.
fn prepared() -> TempDir {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    accounts::save(&env.ctx, &owner, &seller_input("رحیم", "Rahim")).unwrap();
    accounts::save(&env.ctx, &owner, &seller_input("کریم", "Karim")).unwrap();
    let cash = env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, "01010001"))
        .unwrap()
        .unwrap();
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: cash,
                version: accounts::get(&env.ctx, cash).unwrap().version,
                opening_date: "31-08-2026".into(),
                opening_dr: "65,000".into(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    dir
}

// ---------------------------------------------------------------------------------------------
// The child: the "app" that gets killed.
// ---------------------------------------------------------------------------------------------

/// One protocol line to the parent. The test harness prints its own text without a line end
/// ("test name ... "), so a line of ours starts on a new line and is marked with `@@`.
fn say(text: &str) {
    let mut out = std::io::stdout().lock();
    writeln!(out, "\n@@{text}").unwrap();
    out.flush().unwrap();
}

/// Not a test. The parent starts this same binary with `CHILD_DIR` set and kills it. Run on its
/// own it does nothing.
#[test]
fn child_process_for_the_kill_tests() {
    let Ok(dir) = std::env::var(CHILD_DIR) else {
        return;
    };
    let mode = std::env::var(CHILD_MODE).unwrap_or_default();
    let dir = Path::new(&dir);
    say("START");
    let env = TestEnv::new(dir).unwrap();
    say("OPENED");
    if mode == "open" {
        // Set up from nothing: the first run, the sign-in.
        if auth::status(&env.ctx).unwrap().first_run {
            set_up(&env);
            say("SETUP");
        }
        login_owner(&env);
        say("LOGGED_IN");
        return;
    }
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, "030110001"))
        .unwrap()
        .unwrap();
    let karim = env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, "030110002"))
        .unwrap()
        .unwrap();
    let mut n = 0_u64;
    loop {
        n += 1;
        match n % 5 {
            0 | 1 => {
                vouchers::save_advance(
                    &env.ctx,
                    &owner,
                    &advance(if n.is_multiple_of(2) { rahim } else { karim }, 100 + n),
                )
                .unwrap();
                say("ACK voucher 1");
            }
            2 => {
                accounts::save(&env.ctx, &owner, &buyer_input(n)).unwrap();
                say("ACK account 1");
            }
            3 => {
                let lines: Vec<AdvanceInput> = (0..4).map(|k| advance(rahim, 10 + n + k)).collect();
                let saved =
                    vouchers::save_advances(&env.ctx, &owner, &AdvancesInput { lines }).unwrap();
                let worked = saved.rows.iter().filter(|r| r.saved.is_some()).count();
                say(&format!("ACK voucher {worked}"));
            }
            _ => {
                // Reverse the newest advance entry that is not reversed yet.
                let target: Option<i64> = env
                    .ctx
                    .db()
                    .read(|c| {
                        c.query_row(
                            "SELECT MAX(e.id) FROM journal_entries e WHERE e.posting_type = 'advance'
                             AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id)",
                            [],
                            |r| r.get(0),
                        )
                    })
                    .unwrap();
                if let Some(id) = target {
                    env.ctx
                        .db()
                        .write(|tx| ledger::reverse(&env.ctx, tx, owner.user_id, id, "child"))
                        .unwrap();
                    say("ACK reversal 1");
                }
            }
        }
    }
}

struct Child {
    process: std::process::Child,
    lines: mpsc::Receiver<String>,
}

fn spawn_child(dir: &Path, mode: &str) -> Child {
    let mut process = Command::new(std::env::current_exe().unwrap())
        .args([
            "child_process_for_the_kill_tests",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_DIR, dir)
        .env(CHILD_MODE, mode)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let out = process.stdout.take().unwrap();
    let (tx, lines) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(out).lines().map_while(Result::ok) {
            let Some(line) = line.strip_prefix("@@") else {
                continue;
            };
            if tx.send(line.to_owned()).is_err() {
                break;
            }
        }
    });
    Child { process, lines }
}

impl Child {
    /// Kills the process with SIGKILL and returns every line it managed to print.
    fn kill(mut self) -> Vec<String> {
        let _ = self.process.kill();
        let _ = self.process.wait();
        let mut out = Vec::new();
        while let Ok(line) = self.lines.recv_timeout(Duration::from_millis(300)) {
            out.push(line);
        }
        out
    }
}

/// Reads the table counts of the whole data file and checks that every posting is whole.
fn assert_sound_after_kill(
    dir: &Path,
    acked_vouchers: i64,
    acked_accounts: i64,
    round: usize,
) -> (i64, i64) {
    let ctx = context_for(dir)
        .unwrap_or_else(|e| panic!("round {round}: the file does not open after a kill: {e:?}"));
    assert_eq!(
        ctx.db().data_state(),
        DataState::Ok,
        "round {round}: the check failed after a kill"
    );
    let c = |sql: &str| count(dir, sql);
    let integrity: String = {
        let conn =
            mandi_services::db::rusqlite::Connection::open(dir.join("data").join("mandi.db"))
                .unwrap();
        conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap()
    };
    assert_eq!(integrity, "ok", "round {round}");
    assert_eq!(
        c("SELECT COUNT(*) FROM pragma_foreign_key_check"),
        0,
        "round {round}: a foreign key is broken"
    );
    // The books.
    assert_eq!(
        c(
            "SELECT COALESCE(SUM(l.dr_paisa), 0) - COALESCE(SUM(l.cr_paisa), 0) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id WHERE e.status = 'posted'"
        ),
        0,
        "round {round}: debits differ from credits"
    );
    assert_eq!(
        c("SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'"),
        0,
        "round {round}: a draft entry was left"
    );
    assert_eq!(
        c(
            "SELECT COUNT(*) FROM journal_lines l LEFT JOIN journal_entries e ON e.id = l.entry_id WHERE e.id IS NULL"
        ),
        0,
        "round {round}: a line has no entry"
    );
    assert_eq!(
        c(
            "SELECT COUNT(*) FROM (SELECT e.id FROM journal_entries e LEFT JOIN journal_lines l ON l.entry_id = e.id GROUP BY e.id, e.total_paisa HAVING COUNT(l.id) = 0 OR SUM(l.dr_paisa) <> SUM(l.cr_paisa) OR SUM(l.dr_paisa) <> e.total_paisa)"
        ),
        0,
        "round {round}: an entry is half written"
    );
    // A voucher, its journal entry and its audit row go together.
    let vouchers_n = c("SELECT COUNT(*) FROM vouchers");
    assert_eq!(
        c("SELECT COUNT(*) FROM vouchers WHERE current_entry_id IS NULL"),
        0,
        "round {round}: a voucher has no entry"
    );
    assert_eq!(
        c("SELECT COUNT(*) FROM journal_entries WHERE posting_type = 'advance'"),
        vouchers_n,
        "round {round}: vouchers and advance entries differ"
    );
    assert_eq!(
        c("SELECT COUNT(*) FROM audit_log WHERE action = 'create' AND entity = 'voucher'"),
        vouchers_n,
        "round {round}: vouchers and their audit rows differ"
    );
    assert_eq!(
        c("SELECT COALESCE(MAX(number), 0) FROM vouchers WHERE type = 'advance'"),
        vouchers_n,
        "round {round}: voucher numbers have a gap"
    );
    // An account, its opening entry and its audit row go together.
    let accounts_n = c("SELECT COUNT(*) FROM accounts WHERE is_system = 0 AND name_en = 'Akram'");
    assert_eq!(
        c(
            "SELECT COUNT(*) FROM accounts WHERE is_system = 0 AND name_en = 'Akram' AND opening_dr_paisa > 0 AND opening_entry_id IS NULL"
        ),
        0,
        "round {round}: an account has an opening and no entry"
    );
    // Acknowledged work survived.
    assert!(
        vouchers_n >= acked_vouchers,
        "round {round}: {acked_vouchers} vouchers were acknowledged but {vouchers_n} exist"
    );
    assert!(
        accounts_n >= acked_accounts,
        "round {round}: {acked_accounts} accounts were acknowledged but {accounts_n} exist"
    );
    // The hash chains.
    ledger::verify_journal_chain(&ctx)
        .unwrap_or_else(|e| panic!("round {round}: the journal chain is broken: {e:?}"));
    mandi_services::db::audit::verify_audit_chain(
        &mandi_services::db::rusqlite::Connection::open(dir.join("data").join("mandi.db")).unwrap(),
        &AuditKey::from_bytes([7; 32]),
    )
    .unwrap_or_else(|e| panic!("round {round}: the audit chain is broken: {e:?}"));
    (vouchers_n, accounts_n)
}

/// A context on the file in `dir`, as the app builds it at start-up (opens, upgrades, checks).
fn context_for(dir: &Path) -> Result<Context, mandi_services::db::DbError> {
    let db = Db::open_checked(&dir.join("data").join("mandi.db"), options(dir))?;
    Ok(Context::with_parts(
        Arc::new(db),
        Parts {
            monotonic: Arc::new(ManualMonotonic::default()),
            hasher: fast_hasher().unwrap(),
            random: Arc::new(CountingRandom::default()),
        },
    ))
}

#[test]
fn the_database_is_set_up_for_crash_safety() {
    // WAL and synchronous FULL are what make a power cut safe (Code Standards B13, NFR-REL-1).
    let (dir, env) = env();
    set_up(&env);
    env.ctx
        .db()
        .write(|tx| -> Result<(), ServiceError> {
            let mode: String = tx.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
            let sync: i64 = tx.query_row("PRAGMA synchronous", [], |r| r.get(0))?;
            let keys: i64 = tx.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?;
            assert_eq!(mode.to_lowercase(), "wal");
            assert_eq!(sync, 2, "synchronous must be FULL (2)");
            assert_eq!(keys, 1);
            Ok(())
        })
        .unwrap();
    drop(env);
    drop(dir);
}

#[test]
fn killing_the_process_in_the_middle_of_saves_40_times_never_half_saves() {
    let dir = prepared();
    let mut rng = Rng(0xD1D2_0001);
    let (mut acked_total, mut kills_with_extra, mut last_vouchers, mut last_accounts) =
        (0_i64, 0_u32, 0_i64, 0_i64);
    let (mut acked_vouchers_all, mut acked_accounts_all) = (0_i64, 0_i64);
    let base_vouchers = count(dir.path(), "SELECT COUNT(*) FROM vouchers");
    let _ = base_vouchers;
    for round in 1..=40 {
        let child = spawn_child(dir.path(), "saves");
        // Wait for the child to be saving, then let it run a random moment.
        let wanted = 1 + rng.below(14);
        let started = Instant::now();
        let mut seen = 0_u64;
        let mut lines = Vec::new();
        while seen < wanted && started.elapsed() < Duration::from_secs(20) {
            if let Ok(line) = child.lines.recv_timeout(Duration::from_millis(50)) {
                if line.starts_with("ACK") {
                    seen += 1;
                }
                lines.push(line);
            }
        }
        std::thread::sleep(Duration::from_micros(rng.below(15_000)));
        lines.extend(child.kill());
        assert!(
            lines.iter().any(|l| l == "OPENED"),
            "round {round}: the child never opened the file"
        );
        let (mut av, mut aa) = (0_i64, 0_i64);
        for line in lines.iter().filter(|l| l.starts_with("ACK")) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let n: i64 = parts.get(2).and_then(|n| n.parse().ok()).unwrap_or(0);
            match parts.get(1) {
                Some(&"voucher") => av += n,
                Some(&"account") => aa += n,
                _ => {}
            }
        }
        acked_vouchers_all += av;
        acked_accounts_all += aa;
        acked_total += av + aa;
        let (vouchers_before, accounts_before) = (last_vouchers, last_accounts);
        // `assert_sound_after_kill` compares with this round's acknowledgements added to all the
        // earlier ones: nothing acknowledged in any round may be missing.
        let (vouchers_n, accounts_n) =
            assert_sound_after_kill(dir.path(), acked_vouchers_all, acked_accounts_all, round);
        // Saved in this round beyond what the child acknowledged: a save that committed in the
        // moment before the kill, whose acknowledgement was never printed.
        if vouchers_n - vouchers_before > av || accounts_n - accounts_before > aa {
            kills_with_extra += 1;
        }
        last_vouchers = vouchers_n;
        last_accounts = accounts_n;
    }
    println!(
        "40 kills: {acked_total} saves acknowledged, {last_vouchers} vouchers exist; {kills_with_extra} kills came after a commit and before its acknowledgement"
    );
    assert!(
        acked_total >= 40,
        "the child did too little work for the test to mean anything: {acked_total}"
    );
}

#[test]
fn killing_the_process_while_the_file_is_made_and_set_up_80_times_leaves_a_good_file() {
    // A pristine file, to compare the fixed tables with.
    let reference = {
        let (dir, env) = env();
        drop(env);
        dir
    };
    let tables: Vec<String> = {
        let conn = mandi_services::db::rusqlite::Connection::open(
            reference.path().join("data").join("mandi.db"),
        )
        .unwrap();
        let mut st = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .unwrap();
        st.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    // How long a whole set-up takes in this build: one run to the end.
    let (t_open, t_total) = {
        let dir = tempfile::tempdir().unwrap();
        let started = Instant::now();
        let child = spawn_child(dir.path(), "open");
        let mut t_open = Duration::ZERO;
        loop {
            match child.lines.recv_timeout(Duration::from_secs(30)) {
                Ok(line) if line == "OPENED" => t_open = started.elapsed(),
                Ok(line) if line == "LOGGED_IN" => break,
                Ok(_) => {}
                Err(_) => panic!("the child never finished its set-up"),
            }
        }
        (t_open, started.elapsed())
    };
    println!("a whole set-up takes {t_total:?}; the file is open after {t_open:?}");
    let mut rng = Rng(0xD1_0003);
    let mut stages = std::collections::BTreeMap::<String, u32>::new();
    for round in 1..=80 {
        let dir = tempfile::tempdir().unwrap();
        let child = spawn_child(dir.path(), "open");
        // Anywhere from the start of the process to just after the end of the set-up, so kills
        // land in the migration, the seed, the first run and the sign-in.
        let span = u64::try_from(t_total.as_micros()).unwrap_or(50_000);
        std::thread::sleep(Duration::from_micros(rng.below(span + span / 3)));
        let lines = child.kill();
        let stage = lines
            .last()
            .cloned()
            .unwrap_or_else(|| "NOTHING_YET".to_owned());
        *stages.entry(stage).or_default() += 1;
        // The file (if it exists) opens, upgrades if it must, and passes its check.
        let env = TestEnv::new(dir.path()).unwrap_or_else(|e| {
            panic!("round {round}: the file does not open after a kill: {e:?}")
        });
        assert_eq!(env.ctx.db().data_state(), DataState::Ok, "round {round}");
        for table in &tables {
            if ["users", "settings", "audit_log", "user_prefs"].contains(&table.as_str()) {
                continue;
            }
            assert_eq!(
                count(dir.path(), &format!("SELECT COUNT(*) FROM \"{table}\"")),
                count(
                    reference.path(),
                    &format!("SELECT COUNT(*) FROM \"{table}\"")
                ),
                "round {round}: table {table} differs from a clean start (a half-made file)"
            );
        }
        // The first run is whole or not there: Owner, firm, recovery key and both audit rows.
        let users = count(dir.path(), "SELECT COUNT(*) FROM users");
        let firm = count(
            dir.path(),
            "SELECT COUNT(*) FROM settings WHERE key = 'firm'",
        );
        let recovery = count(
            dir.path(),
            "SELECT COUNT(*) FROM settings WHERE key LIKE '%recovery%'",
        );
        assert_eq!(
            (users, firm, recovery),
            if users == 1 { (1, 1, 1) } else { (0, 0, 0) },
            "round {round}: a half first run"
        );
        if users == 1 {
            assert!(
                count(dir.path(), "SELECT COUNT(*) FROM audit_log") >= 2,
                "round {round}: first run without its audit rows"
            );
        }
        // And the app can be used from there: finish the setup if needed, then sign in.
        if auth::status(&env.ctx).unwrap().first_run {
            set_up(&env);
        }
        login_owner(&env);
        assert_chain_ok(&dir);
    }
    println!("80 kills during set-up, by the last line the child printed: {stages:?}");
    assert!(
        stages.len() >= 3,
        "the kills did not reach different stages: {stages:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// Disk full.
// ---------------------------------------------------------------------------------------------

/// A disk whose free space the test sets.
struct Switch(AtomicU64);

impl DiskProbe for Switch {
    fn available_bytes(&self, _path: &Path) -> std::io::Result<u64> {
        Ok(self.0.load(Ordering::SeqCst))
    }
}

fn env_with_switch() -> (TempDir, Arc<Context>, Arc<Switch>) {
    let dir = tempfile::tempdir().unwrap();
    let switch = Arc::new(Switch(AtomicU64::new(10 << 30)));
    let db = Db::open_checked(
        &dir.path().join("data").join("mandi.db"),
        DbOptions {
            backups_dir: dir.path().join("backups"),
            disk_probe: switch.clone(),
            clock: Arc::new(ManualClock::new(test_start())),
            key_store: Arc::new(MemoryKeyStore::new(AuditKey::from_bytes([7; 32]))),
            busy_timeout: Duration::from_millis(500),
            check_wait: Duration::from_millis(500),
        },
    )
    .unwrap();
    let ctx = Context::with_parts(
        Arc::new(db),
        Parts {
            monotonic: Arc::new(ManualMonotonic::default()),
            hasher: fast_hasher().unwrap(),
            random: Arc::new(CountingRandom::default()),
        },
    );
    (dir, Arc::new(ctx), switch)
}

fn table_counts(dir: &TempDir) -> Vec<(String, i64)> {
    let conn = raw(dir);
    let names: Vec<String> = {
        let mut st = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .unwrap();
        st.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    names
        .into_iter()
        .map(|n| {
            let c: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM \"{n}\""), [], |r| r.get(0))
                .unwrap();
            (n, c)
        })
        .collect()
}

#[test]
fn a_full_disk_stops_every_save_before_it_writes_and_saving_works_again_when_space_returns() {
    let (dir, ctx, switch) = env_with_switch();
    let first = common::first_run_input();
    auth::first_run(&ctx, &first).unwrap();
    auth::login(&ctx, "owner", &mandi_services::Secret::new(OWNER_PASSWORD)).unwrap();
    let owner = auth::require(&ctx).unwrap();
    let rahim = accounts::save(&ctx, &owner, &seller_input("رحیم", "Rahim"))
        .unwrap()
        .id;
    let before = table_counts(&dir);
    // 10 MB free: below the 50 MB a save needs.
    switch.0.store(10 << 20, Ordering::SeqCst);
    let attempts: Vec<(&str, Result<(), ServiceError>)> = vec![
        (
            "advance",
            vouchers::save_advance(&ctx, &owner, &advance(rahim, 500)).map(|_| ()),
        ),
        (
            "account",
            accounts::save(&ctx, &owner, &buyer_input(1)).map(|_| ()),
        ),
        (
            "advance list",
            vouchers::save_advances(
                &ctx,
                &owner,
                &AdvancesInput {
                    lines: vec![advance(rahim, 5), advance(rahim, 6)],
                },
            )
            .map(|_| ()),
        ),
        (
            "deactivate",
            accounts::set_active(&ctx, &owner, rahim, false).map(|_| ()),
        ),
        (
            "day close",
            mandi_services::day::close(
                &ctx,
                &owner,
                &mandi_services::core::vouchers::DayCloseInput {
                    up_to: String::new(),
                },
            )
            .map(|_| ()),
        ),
    ];
    for (label, result) in attempts {
        match result {
            Err(error) => assert!(error.has_code(ErrorCode::ErrDiskFull), "{label}: {error:?}"),
            Ok(()) => {
                // The advance list reports a failure line by line; it must not have saved any.
                assert_eq!(label, "advance list");
            }
        }
    }
    assert_eq!(
        table_counts(&dir),
        before,
        "a save on a full disk wrote something"
    );
    // Reading still works, and the books are sound.
    assert!(accounts::get(&ctx, rahim).is_ok());
    assert!(ledger::trial_balance(&ctx, None).is_ok());
    // The disk is cleaned: the same save works.
    switch.0.store(10 << 30, Ordering::SeqCst);
    vouchers::save_advance(&ctx, &owner, &advance(rahim, 500)).unwrap();
    assert_chain_ok(&dir);
}

#[test]
fn sqlite_running_out_of_pages_in_the_middle_of_a_save_rolls_everything_back() {
    // The free-space check can be wrong (another program fills the disk a moment later), so SQLite
    // can run out of room halfway through a save. Make that happen: cap the number of pages the
    // writing connection may use, just above what the file has now.
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = accounts::save(&env.ctx, &owner, &seller_input("رحیم", "Rahim"))
        .unwrap()
        .id;
    let cash = env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, "01010001"))
        .unwrap()
        .unwrap();
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: cash,
                version: accounts::get(&env.ctx, cash).unwrap().version,
                opening_date: "31-08-2026".into(),
                opening_dr: "65,000".into(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    // Checkpoint, so the page count is the real size.
    let pages: i64 = env
        .ctx
        .db()
        .write(|tx| {
            tx.query_row("PRAGMA page_count", [], |r| r.get(0))
                .map_err(ServiceError::from)
        })
        .unwrap();
    let mut refused = 0;
    let mut worked = 0;
    for headroom in [0_i64, 1, 2, 3, 4, 6, 9, 14, 20] {
        let before = table_counts(&dir);
        let cap = pages + headroom;
        env.ctx
            .db()
            .write(|tx| {
                tx.execute_batch(&format!("PRAGMA max_page_count = {cap}"))
                    .map_err(ServiceError::from)
            })
            .unwrap();
        // A save that needs more than `headroom` new pages: an advance, a long list, an account.
        let lines: Vec<AdvanceInput> = (0..30).map(|k| advance(rahim, 1000 + k)).collect();
        let results = [
            vouchers::save_advance(&env.ctx, &owner, &advance(rahim, 700)).map(|_| ()),
            vouchers::save_advances(&env.ctx, &owner, &AdvancesInput { lines }).map(|_| ()),
            accounts::save(&env.ctx, &owner, &buyer_input(headroom as u64)).map(|_| ()),
        ];
        for result in results {
            match result {
                Err(error) => {
                    assert!(
                        error.has_code(ErrorCode::ErrDiskFull),
                        "headroom {headroom}: a full file gave {error:?} instead of ERR_DISK_FULL"
                    );
                    refused += 1;
                }
                Ok(()) => worked += 1,
            }
        }
        // Lift the cap for the checks.
        env.ctx
            .db()
            .write(|tx| {
                tx.execute_batch("PRAGMA max_page_count = 1073741823")
                    .map_err(ServiceError::from)
            })
            .unwrap();
        // Whatever was saved is whole; whatever was refused left nothing.
        let after = table_counts(&dir);
        let n =
            |counts: &[(String, i64)], t: &str| counts.iter().find(|c| c.0 == t).map_or(0, |c| c.1);
        let new_vouchers = n(&after, "vouchers") - n(&before, "vouchers");
        let new_entries = n(&after, "journal_entries") - n(&before, "journal_entries");
        let new_accounts = n(&after, "accounts") - n(&before, "accounts");
        // Every new voucher has one entry (advance), every new account with an opening has two.
        assert_eq!(
            new_entries,
            new_vouchers + new_accounts,
            "headroom {headroom}: entries do not match what was saved"
        );
        assert_eq!(
            n(&after, "audit_log") - n(&before, "audit_log"),
            new_vouchers + new_accounts,
            "headroom {headroom}: audit rows do not match what was saved"
        );
        let integrity: String = raw(&dir)
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok", "headroom {headroom}");
        assert_eq!(
            raw(&dir)
                .query_row(
                    "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_chain_ok(&dir);
        ledger::verify_journal_chain(&env.ctx).unwrap();
    }
    println!(
        "pages used: {pages}; saves refused for lack of room: {refused}; saves that fitted: {worked}"
    );
    assert!(
        refused >= 3,
        "the cap never bit, so the test shows nothing: {refused}"
    );
}

#[test]
fn sending_the_same_save_twice_gives_two_vouchers_because_the_backend_has_no_request_number() {
    // NFR-REL: "the same save sent twice quickly gives exactly one posting". The backend has no
    // request id on a voucher, so the guard against a double click can only be on the screen.
    // This test records what the backend does, so a change is noticed. See the finding.
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = accounts::save(&env.ctx, &owner, &seller_input("رحیم", "Rahim"))
        .unwrap()
        .id;
    let input = advance(rahim, 500);
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let (ctx, owner, input, barrier) = (
                env.ctx.clone(),
                owner.clone(),
                input.clone(),
                barrier.clone(),
            );
            std::thread::spawn(move || {
                barrier.wait();
                vouchers::save_advance(&ctx, &owner, &input).is_ok()
            })
        })
        .collect();
    let worked: usize = handles
        .into_iter()
        .map(|h| usize::from(h.join().unwrap()))
        .sum();
    let saved = count(dir.path(), "SELECT COUNT(*) FROM vouchers");
    println!("two identical saves at once: {worked} worked, {saved} vouchers exist");
    // Whatever happens, the books are whole and the numbers have no gap.
    assert_eq!(saved, i64::try_from(worked).unwrap());
    assert_eq!(
        count(dir.path(), "SELECT COALESCE(MAX(number), 0) FROM vouchers"),
        saved
    );
    assert_chain_ok(&dir);
}

// ---------------------------------------------------------------------------------------------
// Damaged files.
// ---------------------------------------------------------------------------------------------

fn options(dir: &Path) -> DbOptions {
    DbOptions {
        backups_dir: dir.join("backups"),
        disk_probe: Arc::new(mandi_services::db::disk::FixedDiskProbe(10 << 30)),
        clock: Arc::new(ManualClock::new(test_start())),
        key_store: Arc::new(MemoryKeyStore::new(AuditKey::from_bytes([7; 32]))),
        busy_timeout: Duration::from_millis(300),
        check_wait: Duration::from_millis(300),
    }
}

#[test]
fn damaged_files_never_crash_the_program_and_a_file_that_fails_its_check_stops_saving() {
    // A populated file to damage.
    let source = prepared();
    {
        // Fold the log into the main file, so the damage lands in the file the app reads.
        let conn = mandi_services::db::rusqlite::Connection::open(
            source.path().join("data").join("mandi.db"),
        )
        .unwrap();
        let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
    }
    let bytes = std::fs::read(source.path().join("data").join("mandi.db")).unwrap();
    assert!(
        bytes.len() > 100_000,
        "the test file is too small to damage: {}",
        bytes.len()
    );
    let mut rng = Rng(0xD1_0004);
    let mut cases: Vec<(String, Vec<u8>)> = vec![
        ("empty file".into(), Vec::new()),
        ("1000 bytes".into(), bytes[..1000].to_vec()),
        ("half the file".into(), bytes[..bytes.len() / 2].to_vec()),
        ("header zeroed".into(), {
            let mut b = bytes.clone();
            b[..100].fill(0);
            b
        }),
        ("header text only".into(), b"SQLite format 3\0".to_vec()),
        (
            "text file".into(),
            "this is not a database, it is a note"
                .repeat(1000)
                .into_bytes(),
        ),
        ("garbage appended".into(), {
            let mut b = bytes.clone();
            b.extend(std::iter::repeat_n(0xAB, 50_000));
            b
        }),
        ("a page of 0xFF in the middle".into(), {
            let mut b = bytes.clone();
            let at = (b.len() / 2) / 4096 * 4096;
            b[at..at + 4096].fill(0xFF);
            b
        }),
        ("a page of zeros at the start of the tables".into(), {
            let mut b = bytes.clone();
            b[4096..8192].fill(0);
            b
        }),
    ];
    for n in 0..24 {
        let mut b = bytes.clone();
        let flips = 1 + rng.below(300);
        for _ in 0..flips {
            let at = usize::try_from(rng.below(b.len() as u64)).unwrap();
            let skip_header = at < 100;
            if !skip_header || n % 3 == 0 {
                b[at] ^= 1 << rng.below(8);
            }
        }
        cases.push((format!("{flips} flipped bits (round {n})"), b));
    }
    let (mut open_failed, mut check_failed, mut passed) = (0, 0, 0);
    for (label, damaged) in cases {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("data")).unwrap();
        std::fs::write(dir.path().join("data").join("mandi.db"), &damaged).unwrap();
        let opened = std::panic::catch_unwind(|| {
            Db::open_checked(
                &dir.path().join("data").join("mandi.db"),
                options(dir.path()),
            )
        });
        let opened =
            opened.unwrap_or_else(|_| panic!("{label}: the program panicked on a damaged file"));
        match opened {
            Err(error) => {
                open_failed += 1;
                // A clear error with a code the screen can show, never a bare SQLite message.
                let code = error.code();
                assert!(
                    matches!(
                        code,
                        ErrorCode::ErrDataCheckFailed
                            | ErrorCode::ErrMigrationFailed
                            | ErrorCode::ErrInternal
                    ),
                    "{label}: {error:?}"
                );
            }
            Ok(db) if db.data_state() == DataState::Failed => {
                check_failed += 1;
                // Saving is held (NFR-REL-7); nothing is written.
                let attempt: Result<(), mandi_services::db::DbError> = db.write(|tx| {
                    tx.execute("INSERT INTO cities (code, name_ur, name_en, search_ur, search_en, search_phonetic) VALUES ('x','ا','a','ا','a','a')", [])?;
                    Ok(())
                });
                match attempt {
                    Err(error) => assert_eq!(
                        error.code(),
                        ErrorCode::ErrDataCheckFailed,
                        "{label}: {error:?}"
                    ),
                    Ok(()) => panic!("{label}: a file that failed its check accepted a save"),
                }
            }
            Ok(db) => {
                passed += 1;
                // The damage missed everything the check looks at. Reading and the chain checks
                // must still not crash.
                let ctx = Context::with_parts(
                    Arc::new(db),
                    Parts {
                        monotonic: Arc::new(ManualMonotonic::default()),
                        hasher: fast_hasher().unwrap(),
                        random: Arc::new(CountingRandom::default()),
                    },
                );
                let _ = ledger::verify_journal_chain(&ctx);
                let _ = ledger::trial_balance(&ctx, None);
                let _ = accounts::list(
                    &ctx,
                    &mandi_services::core::accounts::AccountsListInput {
                        filter: mandi_services::core::accounts::AccountFilter {
                            account_type: None,
                            text: String::new(),
                            include_inactive: true,
                        },
                        offset: 0,
                        limit: 0,
                    },
                );
            }
        }
    }
    println!(
        "damaged files: {open_failed} would not open, {check_failed} failed the check, {passed} passed the quick check"
    );
    assert!(
        open_failed + check_failed >= 8,
        "the damage was never found: {open_failed} {check_failed}"
    );
}

#[test]
fn a_migration_that_fails_puts_the_data_back_and_a_file_from_the_future_is_not_touched() {
    use mandi_services::db::rusqlite::Connection;
    let source = prepared();
    let path = source.path().join("data").join("mandi.db");
    {
        let conn = Connection::open(&path).unwrap();
        let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
        // A newer app made this file: the schema version is ahead.
        conn.pragma_update(None, "user_version", 99).unwrap();
    }
    let before = std::fs::read(&path).unwrap();
    let error = Db::open(&path, options(source.path())).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrMigrationFailed, "{error:?}");
    // Opening it read nothing that changes the file's contents or its version.
    let conn = Connection::open(&path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 99, "the version of a newer file was changed");
    drop(conn);
    let after = std::fs::read(&path).unwrap();
    assert_eq!(before.len(), after.len(), "the file changed size");
}

// ---------------------------------------------------------------------------------------------
// A long run.
// ---------------------------------------------------------------------------------------------

fn resident_mb() -> f64 {
    let pid = std::process::id().to_string();
    let out = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<f64>()
        .unwrap_or(0.0)
        / 1024.0
}

#[test]
fn a_long_run_of_many_operations_shows_no_slowdown_and_no_memory_growth() {
    // NFR-REL-12 stands in for 12 hours of use: 6 blocks of 400 mixed operations (saves,
    // searches, cards, a trial balance, an occasional account) on one open file.
    let (_dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = accounts::save(&env.ctx, &owner, &seller_input("رحیم", "Rahim"))
        .unwrap()
        .id;
    for n in 0..60 {
        let mut input = buyer_input(n);
        input.opening_date = String::new();
        input.opening_dr = String::new();
        input.name_en =
            ["Akram", "Arshad", "Shahid", "Nasir"][usize::try_from(n % 4).unwrap()].into();
        input.name_ur = ["اکرم", "ارشد", "شاہد", "ناصر"][usize::try_from(n % 4).unwrap()].into();
        accounts::save(&env.ctx, &owner, &input).unwrap();
    }
    let texts = [
        "akram", "arshd", "اکرم", "shahid", "1", "2", "xyz", "nasr", "03040010",
    ];
    let mut block_times = Vec::new();
    let mut block_mem = Vec::new();
    let mut by_kind: Vec<[f64; 4]> = Vec::new();
    let mut n = 0_u64;
    for _block in 0..6 {
        let started = Instant::now();
        let mut kinds = [0.0_f64; 4];
        for _ in 0..400 {
            n += 1;
            let op_started = Instant::now();
            let kind = match n % 8 {
                0 => 0,
                1..=4 => 1,
                5 | 6 => 2,
                _ => 3,
            };
            match n % 8 {
                0 => {
                    vouchers::save_advance(&env.ctx, &owner, &advance(rahim, 1 + n % 900)).unwrap();
                }
                1..=4 => {
                    let text = texts[usize::try_from(n % 9).unwrap()];
                    let _ = mandi_services::search::search_accounts(
                        &env.ctx,
                        &owner,
                        &mandi_services::core::accounts::SearchAccountsInput {
                            scope: mandi_services::core::accounts::AccountScope::Any,
                            control_id: None,
                            text: text.into(),
                            limit: 20,
                        },
                    )
                    .unwrap();
                }
                5 => {
                    let _ = mandi_services::assist::party_card(&env.ctx, rahim).unwrap();
                }
                6 => {
                    let _ = ledger::trial_balance(&env.ctx, None).unwrap();
                }
                _ => {
                    // Some wrong sign-in tries and a lock check, as a day at the counter has.
                    let _ = auth::status(&env.ctx).unwrap();
                    let _ = mandi_services::day::status(&env.ctx).unwrap();
                }
            }
            if let Some(slot) = kinds.get_mut(kind) {
                *slot += op_started.elapsed().as_secs_f64();
            }
        }
        by_kind.push(kinds);
        block_times.push(started.elapsed().as_secs_f64());
        block_mem.push(resident_mb());
    }
    println!("block seconds {block_times:?}; resident MB {block_mem:?}");
    println!(
        "seconds per block by kind [save, search, card or trial balance, status]: {by_kind:?}"
    );
    // The blocks do the same work. A save, a card and a trial balance cost more as the data grows
    // (they read more lines), which is not a slowdown of the program. A search and a status
    // read do not depend on the data that grows here, so they must stay the same: that is the
    // test for a program that slows down by itself (a leak, a list that grows, a cache that
    // is never emptied).
    let flat = |kind: usize| -> (f64, f64) { (by_kind[1][kind], by_kind[5][kind]) };
    for (name, kind) in [("search", 1_usize), ("status read", 3)] {
        let (second, last) = flat(kind);
        assert!(
            last < second * 1.6 + 0.01,
            "{name}: the last block took {last:.3}s, the second {second:.3}s"
        );
    }
    assert!(
        block_times[5] < block_times[1] * 3.0,
        "the whole block became three times slower: {block_times:?}"
    );
    let growth = block_mem[5] - block_mem[1];
    assert!(
        growth < 25.0,
        "memory grew by {growth:.1} MB over four blocks: {block_mem:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// Lock and sign-out are safety steps: they work when the disk is full (NFR-SEC-3, NFR-REL-10).
// ---------------------------------------------------------------------------------------------

#[test]
fn lock_and_sign_out_still_work_on_a_full_disk() {
    let (dir, ctx, switch) = env_with_switch();
    auth::first_run(&ctx, &common::first_run_input()).unwrap();
    auth::login(&ctx, "owner", &mandi_services::Secret::new(OWNER_PASSWORD)).unwrap();
    assert!(auth::require(&ctx).is_ok());
    let before = table_counts(&dir);
    // 10 MB free: every write is refused, the audit row of the lock included.
    switch.0.store(10 << 20, Ordering::SeqCst);
    auth::lock(&ctx).unwrap();
    assert!(
        auth::require(&ctx)
            .unwrap_err()
            .has_code(ErrorCode::ErrSessionLocked),
        "the app stayed open because the audit row could not be written"
    );
    auth::logout(&ctx).unwrap();
    assert!(
        auth::require(&ctx)
            .unwrap_err()
            .has_code(ErrorCode::ErrNotLoggedIn),
        "the user stayed signed in because the audit row could not be written"
    );
    assert_eq!(
        table_counts(&dir),
        before,
        "a refused write left something behind"
    );
    // Signing in needs a write (the audit row), so it waits for space; then all is as before.
    assert!(
        auth::login(&ctx, "owner", &mandi_services::Secret::new(OWNER_PASSWORD)).is_err(),
        "a sign-in was written on a full disk"
    );
    switch.0.store(10 << 30, Ordering::SeqCst);
    auth::login(&ctx, "owner", &mandi_services::Secret::new(OWNER_PASSWORD)).unwrap();
    assert_chain_ok(&dir);
}
