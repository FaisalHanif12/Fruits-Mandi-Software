//! Gate lane T3 (plan 80), part 2: the app is killed (SIGKILL, no clean-up: a crash or the task
//! manager) in the middle of the BIG saves, and the draft survives a kill (NFR-REL-1, NFR-REL-2,
//! NFR-REL-7, PRD FR-12.8, FR-12.10, E19, E20). Owner: `robustness`.
//!
//! `robustness_fault.rs` already kills the process 40 times in the middle of voucher, account and
//! reversal saves. This file kills it while it runs the saves that write the most tables:
//! a بکری save, a later sale, an edit (reverse and post again), a cancel, a receipt that is matched
//! to bills. After each kill, on the same data file:
//!
//! - the file opens and passes the check, every foreign key holds, the books balance, no entry is
//!   a draft, every entry has all its lines, no edit window is left open;
//! - the numbers agree between tables: each lot's sold quantity is the sum of its live lines, the
//!   analytics facts add up to the live lines, the partner shares add up to income less expenses
//!   (PRD FR-6.5), the بکری numbers have no gap, each saved بکری has a posted entry;
//! - both hash chains verify;
//! - everything the child announced as saved before the kill is there;
//! - the next save works (no stale number or chain head).
//!
//! The draft: a child writes a draft over and over (the screen's autosave) and is killed. After
//! each kill the draft is whole and is not older than the last one that was announced.
//!
//! A kill of the process does not lose what the operating system already has, so this is not a
//! power cut. The real power-cut run is on the Windows list.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use common::*;
use mandi_services::core::bikri::{
    BikriSaveInput, CancelInput, EditOpenInput, EditSaveInput, LaterSaleSaveInput,
};
use mandi_services::core::drafts::{DraftGetInput, DraftSaveInput};
use mandi_services::core::vouchers::ReceiptInput;
use mandi_services::db::audit::{AuditKey, MemoryKeyStore};
use mandi_services::db::clock::ManualClock;
use mandi_services::db::{DataState, Db, DbOptions};
use mandi_services::testing::{CountingRandom, ManualMonotonic, TestEnv, fast_hasher, test_start};
use mandi_services::{Context, Parts, auth, bikri, drafts, ledger, vouchers};
use support::World;
use tempfile::TempDir;

const CHILD_DIR: &str = "MANDI_GATE_T3_CHILD_DIR";
const CHILD_MODE: &str = "MANDI_GATE_T3_CHILD_MODE";

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

fn say(text: &str) {
    let mut out = std::io::stdout().lock();
    writeln!(out, "\n@@{text}").unwrap();
    out.flush().unwrap();
}

fn scalar_in(dir: &Path, sql: &str) -> i64 {
    let conn =
        mandi_services::db::rusqlite::Connection::open(dir.join("data").join("mandi.db")).unwrap();
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

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

/// The worked world with the file closed: the parties, items, partners and 50,000 in cash.
fn prepared() -> TempDir {
    let World { dir, env, .. } = support::world();
    drop(env);
    dir
}

/// A World on an already prepared file, with the ids found by their codes, for the child.
fn world_on(dir: &Path) -> World {
    let env = TestEnv::new(dir).unwrap();
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let id = |code: &str| support::id_of(&env, code);
    let named = |table: &str, name_en: &str| -> i64 {
        env.ctx
            .db()
            .read(|c| {
                c.query_row(
                    &format!("SELECT id FROM {table} WHERE name_en = ?1"),
                    [name_en],
                    |r| r.get::<_, i64>(0),
                )
            })
            .unwrap()
    };
    let partners: Vec<i64> = env
        .ctx
        .db()
        .read(|c| {
            let mut st = c.prepare("SELECT id FROM partners ORDER BY id")?;
            let rows = st.query_map([], |r| r.get::<_, i64>(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .unwrap();
    World {
        dir: tempfile::tempdir().unwrap(),
        owner,
        rahim: id("0010"),
        karim: id("0011"),
        akram: id("0020"),
        arshad: id("0021"),
        naqdi: id("03040000"),
        chukta_khata: id("0030"),
        banana: named("items", "Banana"),
        apple: named("items", "Apple"),
        mooli: named("items", "Radish"),
        weight_fruit: named("items", "Grapes"),
        box_: named("packings", "Box"),
        partners: [partners[0], partners[1], partners[2]],
        env,
    }
}

fn the(w: &World, sql: &str) -> Option<i64> {
    w.env
        .ctx
        .db()
        .read(|c| c.query_row(sql, [], |r| r.get::<_, Option<i64>>(0)))
        .unwrap()
}

// ---------------------------------------------------------------------------------------------
// The child: the "app" that is killed.
// ---------------------------------------------------------------------------------------------

/// Not a test. The parent starts this binary with the folder in the environment and kills it.
#[test]
fn child_process_for_the_gate_kill_tests() {
    let Ok(dir) = std::env::var(CHILD_DIR) else {
        return;
    };
    let mode = std::env::var(CHILD_MODE).unwrap_or_default();
    let w = world_on(Path::new(&dir));
    say("READY");
    if mode == "draft" {
        let mut n = std::env::var("MANDI_GATE_T3_START")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        loop {
            n += 1;
            // The screen's autosave: the whole typing so far, as JSON that can be checked.
            let payload = format!(
                "{{\"seq\":{n},\"typing\":\"{}\",\"end\":true}}",
                "x".repeat(usize::try_from(n % 200).unwrap())
            );
            drafts::save(
                &w.env.ctx,
                &w.owner,
                &DraftSaveInput {
                    screen: "bikri".into(),
                    payload_json: payload,
                },
            )
            .unwrap();
            say(&format!("ACK draft {n}"));
        }
    }
    let mut n = 0_u64;
    loop {
        n += 1;
        let kind = match n % 7 {
            1 | 4 => step_save(&w, n),
            2 => step_receipt(&w, n),
            3 => step_later_sale(&w, n),
            5 => step_edit(&w, n),
            6 => step_big_save(&w, n),
            _ => step_cancel(&w),
        };
        match kind {
            Ok(Some(what)) => say(&format!("ACK {what}")),
            Ok(None) => {}
            Err(text) => say(&format!("ERR {text}")),
        }
    }
}

fn step_save(w: &World, n: u64) -> Result<Option<&'static str>, String> {
    let seller = if n.is_multiple_of(2) {
        w.rahim
    } else {
        w.karim
    };
    let item = if n.is_multiple_of(2) {
        w.banana
    } else {
        w.apple
    };
    let draft = support::draft(vec![support::with_sales(
        support::lot(w, "lot-1", seller, item, "50"),
        vec![
            support::sale("s-1", w.akram, "20", "600"),
            support::sale("s-2", w.arshad, "15", "580"),
        ],
    )]);
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: true,
        },
    )
    .map(|_| Some("bikri"))
    .map_err(|e| format!("save {e:?}"))
}

/// A بکری of 120 lines: a long transaction, so a kill often lands inside it.
fn step_big_save(w: &World, n: u64) -> Result<Option<&'static str>, String> {
    let sales = (0..120)
        .map(|i| {
            let buyer = if i % 2 == 0 { w.akram } else { w.arshad };
            support::sale(&format!("b-{i}"), buyer, "1", &(600 + (i % 5)).to_string())
        })
        .collect();
    let seller = if n.is_multiple_of(2) {
        w.rahim
    } else {
        w.karim
    };
    let item = if n.is_multiple_of(2) {
        w.banana
    } else {
        w.apple
    };
    let draft = support::draft(vec![support::with_sales(
        support::lot(w, "lot-big", seller, item, "130"),
        sales,
    )]);
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: true,
        },
    )
    .map(|_| Some("bikri"))
    .map_err(|e| format!("big save {e:?}"))
}

fn step_receipt(w: &World, n: u64) -> Result<Option<&'static str>, String> {
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: "10-10-2026".into(),
            account_id: w.akram,
            amount: (500 + n).to_string(),
            removed: Default::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .map(|_| Some("receipt"))
    .map_err(|e| format!("receipt {e:?}"))
}

fn step_later_sale(w: &World, n: u64) -> Result<Option<&'static str>, String> {
    let Some(lot_id) = the(
        w,
        "SELECT MAX(l.id) FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id
         WHERE b.status = 'saved' AND l.qty_received - l.qty_sold - l.qty_written_off - l.qty_taken >= 3",
    ) else {
        return Ok(None);
    };
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: "10-10-2026".into(),
            sales: vec![support::sale(&format!("s-{n}"), w.arshad, "3", "590")],
            confirm_cash_below_zero: true,
        },
    )
    .map(|_| Some("later"))
    .map_err(|e| format!("later sale {e:?}"))
}

fn step_edit(w: &World, n: u64) -> Result<Option<&'static str>, String> {
    let Some(entry) = the(
        w,
        "SELECT MAX(e.id) FROM journal_entries e JOIN bikri_entries b ON b.id = e.source_id
         WHERE e.source_type = 'bikri' AND e.posting_type = 'bikri_first' AND b.status = 'saved'
         AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id)",
    ) else {
        return Ok(None);
    };
    let bikri_id = the(
        w,
        &format!("SELECT source_id FROM journal_entries WHERE id = {entry}"),
    )
    .unwrap();
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id,
            entry_id: entry,
        },
    )
    .map_err(|e| format!("open for edit {e:?}"))?;
    let mut draft = opened.draft.clone();
    let rate = 600 + (n % 7);
    if let Some(line) = draft.lots[0].sales.first_mut() {
        line.seller_rate = rate.to_string();
        line.buyer_rate = Some(rate.to_string());
    }
    bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("rate".into()),
            confirm_cash_below_zero: true,
        },
    )
    .map(|_| Some("edit"))
    .map_err(|e| format!("edit {e:?}"))
}

fn step_cancel(w: &World) -> Result<Option<&'static str>, String> {
    let Some(id) = the(
        w,
        "SELECT MIN(id) FROM bikri_entries WHERE status = 'saved'
         AND (SELECT COUNT(*) FROM bikri_entries WHERE status = 'saved') > 3",
    ) else {
        return Ok(None);
    };
    let version = bikri::get(&w.env.ctx, id)
        .map_err(|e| format!("get {e:?}"))?
        .version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id,
            version,
            reason: "gate".into(),
            confirm_cash_below_zero: true,
        },
    )
    .map(|_| Some("cancel"))
    .map_err(|e| format!("cancel {e:?}"))
}

// ---------------------------------------------------------------------------------------------
// The parent.
// ---------------------------------------------------------------------------------------------

struct Child {
    process: std::process::Child,
    lines: mpsc::Receiver<String>,
}

fn spawn_child(dir: &Path, mode: &str, start: u64) -> Child {
    let mut process = Command::new(std::env::current_exe().unwrap())
        .args([
            "child_process_for_the_gate_kill_tests",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_DIR, dir)
        .env(CHILD_MODE, mode)
        .env("MANDI_GATE_T3_START", start.to_string())
        .stdout(Stdio::piped())
        .stderr(if std::env::var("MANDI_GATE_T3_STDERR").is_ok() {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
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
    fn wait_for(&self, text: &str, seconds: u64) -> bool {
        let deadline = std::time::Instant::now() + Duration::from_secs(seconds);
        while std::time::Instant::now() < deadline {
            if let Ok(line) = self.lines.recv_timeout(Duration::from_millis(100))
                && line.starts_with(text)
            {
                return true;
            }
        }
        false
    }

    /// SIGKILL, then every line the child managed to print.
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

/// Checks that the file is sound and the tables agree after a kill.
fn assert_sound(dir: &Path, acked_bikri: i64, acked_receipts: i64, round: usize) {
    let ctx = context_for(dir)
        .unwrap_or_else(|e| panic!("round {round}: the file does not open after a kill: {e:?}"));
    assert_eq!(
        ctx.db().data_state(),
        DataState::Ok,
        "round {round}: the check failed after a kill"
    );
    let c = |sql: &str| scalar_in(dir, sql);
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
            "SELECT COUNT(*) FROM (SELECT e.id FROM journal_entries e LEFT JOIN journal_lines l ON l.entry_id = e.id GROUP BY e.id, e.total_paisa HAVING COUNT(l.id) = 0 OR SUM(l.dr_paisa) <> SUM(l.cr_paisa) OR SUM(l.dr_paisa) <> e.total_paisa)"
        ),
        0,
        "round {round}: an entry is half written"
    );
    assert_eq!(
        c("SELECT COUNT(*) FROM edit_windows"),
        0,
        "round {round}: an edit window was left open"
    );
    // Lots: the sold quantity is the sum of the live lines.
    {
        let conn =
            mandi_services::db::rusqlite::Connection::open(dir.join("data").join("mandi.db"))
                .unwrap();
        let mut st = conn
            .prepare("SELECT l.id, b.status, l.qty_received, l.qty_sold, COALESCE((SELECT SUM(s.qty) FROM bikri_sales s WHERE s.lot_id = l.id AND s.is_active = 1), 0) FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id WHERE b.status = 'saved' AND l.qty_sold <> COALESCE((SELECT SUM(s.qty) FROM bikri_sales s WHERE s.lot_id = l.id AND s.is_active = 1), 0)")
            .unwrap();
        let bad: Vec<(i64, String, i64, i64, i64)> = st
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert!(
            bad.is_empty(),
            "round {round}: (lot, status, received, sold, live lines) {bad:?}"
        );
    }
    assert_eq!(
        c(
            "SELECT COUNT(*) FROM bikri_lots l JOIN bikri_entries b ON b.id = l.bikri_id WHERE b.status = 'saved' AND l.qty_sold <> COALESCE((SELECT SUM(s.qty) FROM bikri_sales s WHERE s.lot_id = l.id AND s.is_active = 1), 0)"
        ),
        0,
        "round {round}: a lot's sold quantity differs from its live lines"
    );
    // Every live line is in a posted entry.
    assert_eq!(
        c(
            "SELECT COUNT(*) FROM bikri_sales s LEFT JOIN journal_entries e ON e.id = s.journal_entry_id WHERE s.is_active = 1 AND (e.id IS NULL OR e.status <> 'posted')"
        ),
        0,
        "round {round}: a live line has no posted entry"
    );
    // Analytics facts add up to the live lines (sign +1 and -1 cancel for reversed lines).
    assert_eq!(
        c(
            "SELECT COALESCE(SUM(sign * buyer_value_paisa), 0) - COALESCE((SELECT SUM(buyer_value_paisa) FROM bikri_sales WHERE is_active = 1), 0) FROM analytics_facts"
        ),
        0,
        "round {round}: analytics facts differ from the live lines"
    );
    // PRD FR-6.5: the partner shares are income less shop expenses.
    assert_eq!(
        c(
            "SELECT COALESCE((SELECT SUM(amount_paisa) FROM partner_share_rows), 0)
             - COALESCE((SELECT SUM(l.cr_paisa - l.dr_paisa) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                         JOIN accounts a ON a.id = l.account_id JOIN account_controls k ON k.id = a.control_id
                         JOIN account_heads h ON h.id = k.head_id WHERE e.status = 'posted' AND h.code IN ('04', '05')), 0)"
        ),
        0,
        "round {round}: the partner shares are not income less expenses"
    );
    // Numbers have no gap, and each saved بکری has a posted entry.
    assert_eq!(
        c("SELECT COALESCE(MAX(number), 0) - COUNT(*) FROM bikri_entries"),
        0,
        "round {round}: بکری numbers have a gap"
    );
    assert_eq!(
        c(
            "SELECT COUNT(*) FROM bikri_entries b WHERE b.status = 'saved' AND NOT EXISTS (SELECT 1 FROM journal_entries e WHERE e.source_type = 'bikri' AND e.source_id = b.id AND e.status = 'posted')"
        ),
        0,
        "round {round}: a saved بکری has no posted entry"
    );
    // A saved بکری has its audit row, a voucher too.
    assert_eq!(
        c(
            "SELECT COUNT(*) FROM bikri_entries b WHERE NOT EXISTS (SELECT 1 FROM audit_log a WHERE a.entity = 'bikri' AND a.entity_id = b.id)"
        ),
        0,
        "round {round}: a بکری has no audit row"
    );
    assert_eq!(
        c(
            "SELECT COUNT(*) FROM vouchers v WHERE NOT EXISTS (SELECT 1 FROM audit_log a WHERE a.entity = 'voucher' AND a.entity_id = v.id)"
        ),
        0,
        "round {round}: a voucher has no audit row"
    );
    // Acknowledged work survived.
    assert!(
        c("SELECT COUNT(*) FROM bikri_entries") >= acked_bikri,
        "round {round}: {acked_bikri} بکری were announced but fewer exist"
    );
    assert!(
        c("SELECT COUNT(*) FROM vouchers WHERE type = 'receipt'") >= acked_receipts,
        "round {round}: {acked_receipts} receipts were announced but fewer exist"
    );
    ledger::verify_journal_chain(&ctx)
        .unwrap_or_else(|e| panic!("round {round}: the journal chain is broken: {e:?}"));
    mandi_services::db::audit::verify_audit_chain(
        &mandi_services::db::rusqlite::Connection::open(dir.join("data").join("mandi.db")).unwrap(),
        &AuditKey::from_bytes([7; 32]),
    )
    .unwrap_or_else(|e| panic!("round {round}: the audit chain is broken: {e:?}"));
}

#[test]
fn killing_the_process_in_the_middle_of_bikri_saves_edits_and_cancels_60_times_never_half_saves() {
    let dir = prepared();
    let mut rng = Rng(0xDA7A_BA5E);
    let (mut acked_bikri, mut acked_receipts, mut total_acks) = (0_i64, 0_i64, 0_usize);
    let mut with_unannounced_work = 0;
    for round in 0..60 {
        let child = spawn_child(dir.path(), "books", 0);
        assert!(
            child.wait_for("READY", 60),
            "round {round}: the child did not start"
        );
        std::thread::sleep(Duration::from_millis(30 + rng.below(350)));
        let lines = child.kill();
        let errors: Vec<&String> = lines.iter().filter(|l| l.starts_with("ERR")).collect();
        assert!(
            errors.is_empty(),
            "round {round}: the child had an error that was not a kill: {errors:?}"
        );
        let before_bikri = acked_bikri;
        for line in &lines {
            match line.as_str() {
                "ACK bikri" => acked_bikri += 1,
                "ACK receipt" => acked_receipts += 1,
                _ => {}
            }
            total_acks += usize::from(line.starts_with("ACK"));
        }
        let _ = before_bikri;
        assert_sound(dir.path(), acked_bikri, acked_receipts, round);
        // Work that was committed but not yet announced when the kill came.
        let in_file = scalar_in(dir.path(), "SELECT COUNT(*) FROM bikri_entries");
        if in_file > acked_bikri {
            with_unannounced_work += 1;
        }
    }
    println!(
        "60 kills; {total_acks} saves announced ({acked_bikri} بکری, {acked_receipts} receipts); in {with_unannounced_work} rounds the file held a save that was committed just before the kill and not announced"
    );
    assert!(
        total_acks >= 60,
        "the child did too little work to mean anything: {total_acks}"
    );
    // The next save works on the file that was killed 60 times.
    let w = world_on(dir.path());
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::event1(&w),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    assert_sound(dir.path(), acked_bikri + 1, acked_receipts, 99);
}

#[test]
fn the_draft_survives_a_kill_whole_and_not_older_than_the_last_one_announced() {
    let dir = prepared();
    let mut rng = Rng(0xD2AF_7001);
    let mut last_acked = 0_u64;
    for round in 0..25 {
        let child = spawn_child(dir.path(), "draft", last_acked);
        assert!(child.wait_for("READY", 60), "round {round}: no start");
        std::thread::sleep(Duration::from_millis(20 + rng.below(300)));
        let lines = child.kill();
        for line in &lines {
            if let Some(n) = line.strip_prefix("ACK draft ") {
                last_acked = last_acked.max(n.parse().unwrap());
            }
        }
        let ctx = context_for(dir.path()).unwrap();
        assert_eq!(ctx.db().data_state(), DataState::Ok, "round {round}");
        login_owner_ctx(&ctx);
        let owner = auth::require(&ctx).unwrap();
        let draft = drafts::get(
            &ctx,
            &owner,
            &DraftGetInput {
                screen: "bikri".into(),
            },
        )
        .unwrap();
        if last_acked == 0 {
            continue;
        }
        let draft =
            draft.unwrap_or_else(|| panic!("round {round}: the draft is gone after a kill"));
        // Whole: it parses and ends as the screen wrote it.
        let value: serde_json::Value = serde_json::from_str(&draft.payload_json)
            .unwrap_or_else(|e| panic!("round {round}: the draft is not whole: {e}"));
        assert_eq!(value["end"], true, "round {round}");
        let seq = value["seq"].as_u64().unwrap();
        assert!(
            seq >= last_acked,
            "round {round}: the draft is older ({seq}) than the last one announced ({last_acked})"
        );
        // Only one row per screen, and no half row.
        assert_eq!(
            scalar_in(
                dir.path(),
                "SELECT COUNT(*) FROM drafts WHERE screen = 'bikri'"
            ),
            1
        );
        last_acked = last_acked.max(seq);
    }
    assert!(
        last_acked >= 25,
        "the child wrote too few drafts: {last_acked}"
    );
}

fn login_owner_ctx(ctx: &Context) {
    auth::login(ctx, "owner", &mandi_services::Secret::new(OWNER_PASSWORD)).unwrap();
}
