//! Journal attack (D2 attack list item 4, NFR-REL-8, PRD FR-6.2, FR-6.3, FR-6.6, FR-10.9).
//! Owner: `robustness`.
//!
//! Random sequences of openings, advances, direct postings, reversals (good and bad), day closes
//! and reopens, with hostile dates and amounts mixed in. After EVERY step:
//!
//! - the books balance in the raw tables (not only through the service): sum of debits equals
//!   sum of credits, every posted entry balances on its own and matches its total, no draft is
//!   left, every reversal nets its original to zero on each account and person;
//! - a step the rules refuse leaves the whole database byte for byte as it was (a fingerprint of
//!   every table before and after), and a step the rules allow is not refused;
//! - the open-day window is exactly "after the closed-up-to date and not after today".
//!
//! At the end the balances read three ways (raw SQL, the ledger service, the party card) agree,
//! both hash chains verify, and every day close row equals a fresh recomputation of its day.
//!
//! A failing run prints its seed; run again with `ROBUST_SEED=<seed>` to replay it.
#![cfg(test)]

mod common;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use common::*;
use mandi_services::core::accounts::{
    AccountInput, AccountType, BalanceSide, OpeningLineInput, OpeningsInput,
};
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::ids::AccountId;
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::{AdvanceInput, DayCloseInput, DayReopenInput};
use mandi_services::db::rusqlite::{Connection, types::Value};
use mandi_services::domain::ledger::{
    LedgerError, LedgerLine, PostingType, Side, SourceType, build_entry,
};
use mandi_services::ledger::{self, PostingEvent};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, assist, auth, day, vouchers};
use tempfile::TempDir;

/// Small deterministic random numbers (splitmix64).
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

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[usize::try_from(self.below(items.len() as u64)).unwrap()]
    }
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

/// Every table, row by row, hashed. Equal before and after means nothing was written.
fn fingerprint(dir: &TempDir) -> u64 {
    let conn = raw(dir);
    let tables: Vec<String> = {
        let mut statement = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            )
            .unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    let mut hasher = DefaultHasher::new();
    for table in tables {
        table.hash(&mut hasher);
        let mut statement = conn
            .prepare(&format!("SELECT * FROM \"{table}\" ORDER BY rowid"))
            .unwrap();
        let columns = statement.column_count();
        let mut rows = statement.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            for i in 0..columns {
                let value: Value = row.get(i).unwrap();
                format!("{value:?}").hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

fn one(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

/// The invariants of the books, read from the raw tables.
fn assert_books_balance(dir: &TempDir, note: &str) {
    let conn = raw(dir);
    let (dr, cr): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id WHERE e.status = 'posted'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(dr, cr, "debits and credits differ after {note}");
    assert_eq!(
        one(
            &conn,
            "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'"
        ),
        0,
        "a draft entry is left after {note}"
    );
    assert_eq!(
        one(
            &conn,
            "SELECT COUNT(*) FROM (
                SELECT e.id FROM journal_entries e LEFT JOIN journal_lines l ON l.entry_id = e.id
                WHERE e.status = 'posted'
                GROUP BY e.id, e.total_paisa
                HAVING COUNT(l.id) = 0 OR SUM(l.dr_paisa) <> SUM(l.cr_paisa) OR SUM(l.dr_paisa) <> e.total_paisa)"
        ),
        0,
        "an entry does not balance or does not match its total after {note}"
    );
    assert_eq!(
        one(
            &conn,
            "SELECT COUNT(*) FROM (
                SELECT r.id FROM journal_entries r
                JOIN journal_lines l ON l.entry_id IN (r.id, r.reverses_entry_id)
                WHERE r.reverses_entry_id IS NOT NULL
                GROUP BY r.id, l.account_id, COALESCE(l.pukhta_person_key, '')
                HAVING SUM(l.dr_paisa) <> SUM(l.cr_paisa))"
        ),
        0,
        "a reversal does not net its original to zero after {note}"
    );
    assert_eq!(
        one(
            &conn,
            "SELECT COUNT(*) FROM journal_lines WHERE dr_paisa % 100 <> 0 OR cr_paisa % 100 <> 0 OR dr_paisa < 0 OR cr_paisa < 0"
        ),
        0,
        "a line is not whole rupees after {note}"
    );
}

struct World {
    dir: TempDir,
    env: TestEnv,
    owner: auth::Actor,
    sellers: Vec<i64>,
    buyer: i64,
    cash: i64,
    capital: i64,
    rng: Rng,
    seed: u64,
}

fn account_input(kind: AccountType, ur: &str, en: &str) -> AccountInput {
    AccountInput {
        id: None,
        version: None,
        account_type: kind,
        control_id: None,
        code: String::new(),
        name_ur: ur.into(),
        name_en: en.into(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    }
}

fn id_of(env: &TestEnv, code: &str) -> i64 {
    env.ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, code))
        .unwrap()
        .unwrap()
}

fn world(seed: u64) -> World {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    set_today(&env, 2026, 8, 31);
    let rahim = accounts::save(
        &env.ctx,
        &owner,
        &account_input(AccountType::Seller, "رحیم", "Rahim"),
    )
    .unwrap()
    .id;
    let karim = accounts::save(
        &env.ctx,
        &owner,
        &account_input(AccountType::Seller, "کریم", "Karim"),
    )
    .unwrap()
    .id;
    let akram = accounts::save(
        &env.ctx,
        &owner,
        &account_input(AccountType::Buyer, "اکرم", "Akram"),
    )
    .unwrap()
    .id;
    let cash = id_of(&env, "01010001");
    let capital = id_of(&env, "06010001");
    let line = |account: i64, dr: &str, cr: &str| OpeningLineInput {
        account_id: account,
        version: accounts::get(&env.ctx, account).unwrap().version,
        opening_date: "31-08-2026".into(),
        opening_dr: dr.into(),
        opening_cr: cr.into(),
        opening_is_advance: false,
    };
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(cash, "65,000", ""), line(capital, "", "65,000")],
        },
    )
    .unwrap();
    set_today(&env, 2026, 10, 10);
    World {
        dir,
        env,
        owner,
        sellers: vec![rahim, karim],
        buyer: akram,
        cash,
        capital,
        rng: Rng(seed),
        seed,
    }
}

/// The date `n` days after 2026-08-31, as DD-MM-YYYY and as YYYY-MM-DD.
fn date_after(n: i64) -> (String, String) {
    let day = chrono::NaiveDate::from_ymd_opt(2026, 8, 31).unwrap() + chrono::Duration::days(n);
    (
        day.format("%d-%m-%Y").to_string(),
        day.format("%Y-%m-%d").to_string(),
    )
}

const TODAY_OFFSET: i64 = 40; // 2026-10-10 is 40 days after 2026-08-31

/// What a step must give.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expect {
    Ok,
    Refused,
    /// Either; if it works, the check after it must still pass.
    Either,
}

/// A refusal must be a clear rejection with a code the user can read. An internal failure
/// (`Data`, `Db`, a panic) on hostile input is a defect.
fn assert_clean_refusal(error: &ServiceError, step: &str) {
    assert!(
        matches!(error, ServiceError::Rejected(_)),
        "{step}: refused with an internal error, not a rejection: {error:?}"
    );
}

/// How many steps of each kind worked and how many were refused, over all sequences, so the
/// test can show that it really reached both.
static STATS: std::sync::Mutex<std::collections::BTreeMap<String, (u32, u32)>> =
    std::sync::Mutex::new(std::collections::BTreeMap::new());

fn record(step: &str, worked: bool) {
    let kind = step.split_whitespace().next().unwrap_or("?").to_owned();
    let mut stats = STATS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let slot = stats.entry(kind).or_insert((0, 0));
    if worked {
        slot.0 += 1;
    } else {
        slot.1 += 1;
    }
}

fn check(w: &World, step: &str, expect: Expect, before: u64, result: &Result<(), ServiceError>) {
    record(step, result.is_ok());
    match (result, expect) {
        (Ok(()), Expect::Refused) => {
            panic!("seed {}: {step}: was accepted but must be refused", w.seed)
        }
        (Err(error), Expect::Ok) => panic!(
            "seed {}: {step}: was refused but must work: {error:?}",
            w.seed
        ),
        (Err(error), _) => {
            assert_clean_refusal(error, &format!("seed {} {step}", w.seed));
            assert_eq!(
                fingerprint(&w.dir),
                before,
                "seed {}: {step}: a refused step wrote something",
                w.seed
            );
        }
        (Ok(()), _) => {}
    }
}

fn closed_up_to(w: &World) -> Option<String> {
    day::status(&w.env.ctx).unwrap().closed_up_to
}

/// True if `iso` is inside the open window.
fn is_open(w: &World, iso: &str) -> bool {
    closed_up_to(w).is_none_or(|c| iso > c.as_str()) && iso <= "2026-10-10"
}

const BAD_DATES: [&str; 11] = [
    "31-02-2026",
    "",
    "   ",
    "00-00-0000",
    "32-01-2026",
    "10-13-2026",
    "ten",
    "29-02-2027",
    "10-10-",
    "-10-2026",
    "99999999999999999999-10-2026",
];

/// Forms of 10 October 2026 a clerk might type. The rules do not say which are allowed
/// (PRD FR-7.19 names DD-MM-YYYY and shortcuts), so each may be accepted or refused. If accepted,
/// it must mean 10 October 2026.
const LENIENT_DATES: [&str; 6] = [
    "10/10/2026",
    "10.10.2026",
    "10-10-26",
    " 10-10-2026 ",
    "2026-10-10",
    "۱۰-۱۰-۲۰۲۶",
];

const BAD_AMOUNTS: [&str; 14] = [
    "0",
    "-5",
    "1.5",
    "1,5.0",
    "",
    "   ",
    "abc",
    "1e5",
    "10,00,00,00,001",
    "٣٠٠٠٠٠٠٠٠٠٠٠٠",
    "+5",
    "5-",
    "1,,000",
    "０",
];

const GOOD_AMOUNTS: [&str; 8] = [
    "1",
    "500",
    "1,200",
    "12,00,000",
    "۶۰۰",
    "20+25",
    "10,00,00,00,000",
    "99",
];

fn advance_step(w: &mut World) {
    let seller = *w.rng.pick(&w.sellers.clone());
    let bad_date = w.rng.below(100) < 12;
    let bad_amount = w.rng.below(100) < 15;
    let offset = i64::try_from(w.rng.below(45)).unwrap();
    let (ddmmyyyy, iso) = date_after(offset - 2);
    let lenient = !bad_date && w.rng.below(100) < 12;
    let date = if bad_date {
        (*w.rng.pick(&BAD_DATES)).to_owned()
    } else if lenient {
        (*w.rng.pick(&LENIENT_DATES)).to_owned()
    } else {
        ddmmyyyy
    };
    let iso = if lenient {
        "2026-10-10".to_owned()
    } else {
        iso
    };
    let amount = if bad_amount {
        (*w.rng.pick(&BAD_AMOUNTS)).to_owned()
    } else {
        (*w.rng.pick(&GOOD_AMOUNTS)).to_owned()
    };
    let mut expect = if bad_date || bad_amount || !is_open(w, &iso) {
        Expect::Refused
    } else {
        Expect::Ok
    };
    // Digits of another script are a judgement call of the parser: if it takes them, fine.
    if amount == "٣٠٠٠٠٠٠٠٠٠٠٠٠" || amount == "０" {
        expect = if bad_date || !is_open(w, &iso) {
            Expect::Refused
        } else {
            Expect::Either
        };
    }
    if lenient && expect == Expect::Ok {
        expect = Expect::Either;
    }
    let before = fingerprint(&w.dir);
    let result = vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &AdvanceInput {
            confirm_cash_below_zero: true,
            date: date.clone(),
            seller_account_id: seller,
            amount: amount.clone(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    );
    if let Ok(saved) = &result {
        // Whatever the text said, the entry sits on the day it meant.
        let stored: String = raw(&w.dir)
            .query_row(
                "SELECT entry_date FROM journal_entries WHERE id = ?1",
                [saved.entry_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            stored, iso,
            "seed {}: date text '{date}' was saved on the wrong day",
            w.seed
        );
        let paisa: i64 = raw(&w.dir)
            .query_row(
                "SELECT total_paisa FROM journal_entries WHERE id = ?1",
                [saved.entry_id],
                |r| r.get(0),
            )
            .unwrap();
        let wanted: Option<i64> = match amount.as_str() {
            "1" => Some(1),
            "500" => Some(500),
            "1,200" => Some(1200),
            "12,00,000" => Some(1_200_000),
            "۶۰۰" => Some(600),
            "20+25" => Some(45),
            "10,00,00,00,000" => Some(10_000_000_000),
            "99" => Some(99),
            _ => None,
        };
        if let Some(rupees) = wanted {
            assert_eq!(
                paisa,
                rupees * 100,
                "seed {}: amount text '{amount}' was saved wrong",
                w.seed
            );
        }
    }
    check(
        w,
        &format!("advance date '{date}' amount '{amount}'"),
        expect,
        before,
        &result.map(|_| ()),
    );
}

fn reverse_step(w: &mut World) {
    let conn = raw(&w.dir);
    // Any posted entry that is not an account opening (an opening is reversed through its
    // account, never directly), or a number that is no entry at all.
    let candidates: Vec<(i64, String, bool, bool)> = {
        let mut statement = conn
            .prepare(
                "SELECT e.id, e.entry_date, e.reverses_entry_id IS NOT NULL,
                        EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id)
                 FROM journal_entries e WHERE e.status = 'posted' AND e.posting_type <> 'opening'
                 ORDER BY e.id",
            )
            .unwrap();
        statement
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    let (target, expect, label) = if candidates.is_empty() || w.rng.below(100) < 10 {
        let wrong = *w.rng.pick(&[0_i64, -1, 999_999, i64::MAX]);
        (wrong, Expect::Refused, format!("entry {wrong}"))
    } else {
        let (id, date, is_reversal, is_reversed) = w.rng.pick(&candidates).clone();
        let expect = if is_reversal || is_reversed || !is_open(w, &date) {
            Expect::Refused
        } else {
            Expect::Ok
        };
        (
            id,
            expect,
            format!("entry {id} dated {date} reversal={is_reversal} reversed={is_reversed}"),
        )
    };
    let before = fingerprint(&w.dir);
    let result = w
        .env
        .ctx
        .db()
        .write(|tx| ledger::reverse(&w.env.ctx, tx, w.owner.user_id, target, "robustness"));
    // A caller that reverses a reversal, or an entry twice, is told so with an internal error:
    // that is a bug signal for the caller, and it must still write nothing. A date or a missing
    // entry is a normal rejection.
    match (&result, expect) {
        (Err(ServiceError::Data(_)), Expect::Refused) => {
            assert_eq!(
                fingerprint(&w.dir),
                before,
                "seed {}: {label}: wrote on a bug signal",
                w.seed
            );
        }
        _ => check(
            w,
            &format!("reverse {label}"),
            expect,
            before,
            &result.map(|_| ()),
        ),
    }
}

fn direct_post_step(w: &mut World) {
    let accounts = [w.cash, w.capital, w.buyer, w.sellers[0], w.sellers[1]];
    let n_lines = 2 + usize::try_from(w.rng.below(3)).unwrap();
    let mut lines = Vec::new();
    let mut total = 0_i64;
    for i in 0..n_lines {
        let amount = 1 + i64::try_from(w.rng.below(500_000)).unwrap();
        total += amount;
        let account = *w.rng.pick(&accounts);
        lines.push(LedgerLine::new(
            AccountId::new(account),
            Side::Dr,
            rupees(amount),
        ));
        let _ = i;
    }
    // Credits that add to the same total, in 1 to 3 lines.
    let mut left = total;
    while left > 0 {
        let part = if left == 1 {
            1
        } else {
            1 + i64::try_from(w.rng.below(u64::try_from(left).unwrap())).unwrap()
        };
        lines.push(LedgerLine::new(
            AccountId::new(*w.rng.pick(&accounts)),
            Side::Cr,
            rupees(part),
        ));
        left -= part;
    }
    let entry = build_entry(lines).unwrap();
    let offset = i64::try_from(w.rng.below(45)).unwrap();
    let (_, iso) = date_after(offset - 2);
    let expect = if is_open(w, &iso) {
        Expect::Ok
    } else {
        Expect::Refused
    };
    let before = fingerprint(&w.dir);
    let result = w.env.ctx.db().write(|tx| {
        ledger::post(
            &w.env.ctx,
            tx,
            w.owner.user_id,
            &PostingEvent {
                date: BusinessDate::parse_iso(&iso).unwrap(),
                posting_type: PostingType::General,
                source_type: SourceType::Voucher,
                source_id: 900,
                narration: Some("robustness".into()),
                reverses: None,
                entry,
            },
        )
    });
    check(
        w,
        &format!("post on {iso}"),
        expect,
        before,
        &result.map(|_| ()),
    );
}

fn opening_step(w: &mut World) {
    // A new account with an opening, on a date that may be closed.
    let offset = i64::try_from(w.rng.below(45)).unwrap();
    let (ddmmyyyy, iso) = date_after(offset - 2);
    let seller = w.rng.below(2) == 0;
    let mut form = account_input(
        if seller {
            AccountType::Seller
        } else {
            AccountType::Buyer
        },
        "ناصر",
        "Nasir",
    );
    form.opening_date = ddmmyyyy;
    let side_debit = w.rng.below(2) == 0;
    let amount = format!("{}", 1 + w.rng.below(900_000));
    if side_debit {
        form.opening_dr = amount;
        form.opening_is_advance = seller && w.rng.below(2) == 0;
    } else {
        form.opening_cr = amount;
    }
    let expect = if is_open(w, &iso) {
        Expect::Ok
    } else {
        Expect::Refused
    };
    let before = fingerprint(&w.dir);
    let result = accounts::save(&w.env.ctx, &w.owner, &form);
    check(
        w,
        &format!("new account with an opening on {iso}"),
        expect,
        before,
        &result.map(|_| ()),
    );
}

fn close_step(w: &mut World) {
    let offset = i64::try_from(w.rng.below(50)).unwrap();
    let (ddmmyyyy, iso) = date_after(offset - 3);
    let closed = closed_up_to(w);
    // An empty text closes the oldest open day.
    let empty = w.rng.below(100) < 15;
    let up_to = if empty { String::new() } else { ddmmyyyy };
    let expect = if empty {
        match closed.as_deref() {
            Some(c) if c >= "2026-10-10" => Expect::Refused,
            _ => Expect::Ok,
        }
    } else if iso.as_str() <= "2026-10-10" && closed.as_deref().is_none_or(|c| iso.as_str() > c) {
        Expect::Ok
    } else {
        Expect::Refused
    };
    let before = fingerprint(&w.dir);
    let result = day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: up_to.clone(),
        },
    );
    let was = closed.clone();
    check(
        w,
        &format!("close '{up_to}' (closed up to {was:?})"),
        expect,
        before,
        &result.as_ref().map(|_| ()).map_err(clone_error),
    );
    if result.is_ok() {
        verify_close_rows(w, was.as_deref());
    }
}

fn clone_error(error: &ServiceError) -> ServiceError {
    match error {
        ServiceError::Rejected(items) => ServiceError::Rejected(items.clone()),
        other => ServiceError::Data(format!("{other:?}")),
    }
}

fn reopen_step(w: &mut World) {
    let offset = i64::try_from(w.rng.below(50)).unwrap();
    let (ddmmyyyy, iso) = date_after(offset - 3);
    let closed = closed_up_to(w);
    let reason = (*w.rng.pick(&["checking", "", "   ", "x"])).to_owned();
    let expect = if reason.trim().is_empty()
        || iso.as_str() > "2026-10-10"
        || !closed.as_deref().is_some_and(|c| iso.as_str() <= c)
    {
        Expect::Refused
    } else {
        Expect::Ok
    };
    let before = fingerprint(&w.dir);
    let result = day::reopen(
        &w.env.ctx,
        &w.owner,
        &DayReopenInput {
            date: ddmmyyyy,
            reason: reason.clone(),
        },
    );
    check(
        w,
        &format!("reopen {iso} reason '{reason}' (closed up to {closed:?})"),
        expect,
        before,
        &result.map(|_| ()),
    );
    if expect == Expect::Ok {
        let audit_rows = audit_count(&w.dir, "day_reopen");
        assert!(audit_rows >= 1, "a reopen wrote no audit row");
    }
}

/// Each day just closed has one history row, equal to a fresh recomputation of that day.
fn verify_close_rows(w: &World, was: Option<&str>) {
    let now = closed_up_to(w).unwrap();
    let conn = raw(&w.dir);
    let first = match was {
        Some(day) => {
            chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").unwrap() + chrono::Duration::days(1)
        }
        None => {
            // The oldest posting, or the closed day itself.
            let oldest: Option<String> = conn
                .query_row("SELECT MIN(entry_date) FROM journal_entries", [], |r| {
                    r.get(0)
                })
                .unwrap();
            let last = chrono::NaiveDate::parse_from_str(&now, "%Y-%m-%d").unwrap();
            oldest
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
                .map_or(last, |d| d.min(last))
        }
    };
    let last = chrono::NaiveDate::parse_from_str(&now, "%Y-%m-%d").unwrap();
    let mut day = first;
    while day <= last {
        let iso = day.format("%Y-%m-%d").to_string();
        // The newest 'close' row of that day.
        let json: String = conn
            .query_row(
                "SELECT summary_json FROM day_closes WHERE entry_date = ?1 AND action = 'close' ORDER BY id DESC LIMIT 1",
                [&iso],
                |r| r.get(0),
            )
            .unwrap_or_else(|e| panic!("seed {}: no close row for {iso}: {e}", w.seed));
        let row: serde_json::Value = serde_json::from_str(&json).unwrap();
        let trial = ledger::trial_balance(&w.env.ctx, Some(&iso)).unwrap();
        assert_eq!(
            row["trial_dr_paisa"].as_i64().unwrap(),
            trial.total_dr.as_paisa(),
            "seed {}: day {iso} debit total differs from a fresh recomputation",
            w.seed
        );
        assert_eq!(
            row["trial_cr_paisa"].as_i64().unwrap(),
            trial.total_cr.as_paisa(),
            "seed {}: day {iso}",
            w.seed
        );
        let cash = ledger::ledger(&w.env.ctx, w.cash, None, None, Some(&iso))
            .unwrap()
            .closing;
        let side = match cash.side {
            BalanceSide::Dr => "dr",
            BalanceSide::Cr => "cr",
            BalanceSide::Zero => "zero",
        };
        assert_eq!(
            row["cash_in_hand_side"].as_str().unwrap(),
            side,
            "seed {}: day {iso} cash side",
            w.seed
        );
        assert_eq!(
            row["cash_in_hand_paisa"].as_i64().unwrap(),
            cash.amount_paisa.as_paisa(),
            "seed {}: day {iso} cash",
            w.seed
        );
        let entries: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM journal_entries WHERE status = 'posted' AND entry_date = ?1",
                [&iso],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            row["entries"].as_i64().unwrap(),
            entries,
            "seed {}: day {iso} entry count",
            w.seed
        );
        day += chrono::Duration::days(1);
    }
}

/// The balances of every account read three ways agree.
fn assert_balances_agree(w: &World) {
    let conn = raw(&w.dir);
    let mut statement = conn.prepare("SELECT id FROM accounts ORDER BY id").unwrap();
    let ids: Vec<i64> = statement
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    for id in ids {
        let (dr, cr): (i64, i64) = conn
            .query_row(
                "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
                 FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                 WHERE l.account_id = ?1 AND e.status = 'posted'",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        let (side, amount) = match dr.cmp(&cr) {
            std::cmp::Ordering::Greater => (BalanceSide::Dr, dr - cr),
            std::cmp::Ordering::Less => (BalanceSide::Cr, cr - dr),
            std::cmp::Ordering::Equal => (BalanceSide::Zero, 0),
        };
        let service = ledger::account_balance(&w.env.ctx, id).unwrap();
        assert_eq!(
            (service.side, service.amount_paisa.as_paisa()),
            (side, amount),
            "seed {}: account {id} by ledger",
            w.seed
        );
        let card = assist::party_card(&w.env.ctx, id).unwrap().balance;
        assert_eq!(
            (card.side, card.amount_paisa.as_paisa()),
            (side, amount),
            "seed {}: account {id} by party card",
            w.seed
        );
        let listed = accounts::get(&w.env.ctx, id).unwrap().balance;
        assert_eq!(
            (listed.side, listed.amount_paisa.as_paisa()),
            (side, amount),
            "seed {}: account {id} by accounts::get",
            w.seed
        );
    }
}

fn run_sequence(seed: u64, steps: usize) {
    let mut w = world(seed);
    for step in 0..steps {
        let kind = w.rng.below(100);
        match kind {
            0..=39 => advance_step(&mut w),
            40..=54 => reverse_step(&mut w),
            55..=64 => direct_post_step(&mut w),
            65..=72 => opening_step(&mut w),
            73..=86 => close_step(&mut w),
            _ => reopen_step(&mut w),
        }
        assert_books_balance(&w.dir, &format!("seed {seed} step {step}"));
        let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
        assert_eq!(trial.total_dr, trial.total_cr, "seed {seed} step {step}");
        let status = day::status(&w.env.ctx).unwrap();
        assert!(
            status
                .closed_up_to
                .as_deref()
                .is_none_or(|c| c <= status.today.as_str()),
            "seed {seed} step {step}: closed up to a day after today"
        );
    }
    assert_balances_agree(&w);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
    let _ = TODAY_OFFSET;
}

fn seeds() -> Vec<u64> {
    match std::env::var("ROBUST_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(one) => vec![one],
        None => (1..=24).map(|n| 0x00D2_0000 + n).collect(),
    }
}

#[test]
fn random_sequences_keep_the_books_balanced_and_refusals_write_nothing() {
    for seed in seeds() {
        run_sequence(seed, 70);
    }
    let stats = STATS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    println!("steps (worked, refused) by kind: {stats:?}");
    if std::env::var("ROBUST_SEED").is_err() {
        // The test is only worth something if it reached both outcomes of every kind.
        for (kind, least_worked, least_refused) in [
            ("advance", 100, 50),
            ("reverse", 20, 10),
            ("post", 20, 10),
            ("new", 10, 5),
            ("close", 15, 15),
            ("reopen", 8, 15),
        ] {
            let (worked, refused) = stats.get(kind).copied().unwrap_or((0, 0));
            assert!(worked >= least_worked, "{kind}: only {worked} steps worked");
            assert!(
                refused >= least_refused,
                "{kind}: only {refused} steps were refused"
            );
        }
    }
}

#[test]
fn an_unbalanced_entry_is_refused_before_anything_is_written() {
    // Layer 1, the domain: the builder refuses it, so there is nothing to post.
    let one_sided = build_entry(vec![
        LedgerLine::new(AccountId::new(1), Side::Dr, rupees(100)),
        LedgerLine::new(AccountId::new(2), Side::Cr, rupees(99)),
    ]);
    assert!(matches!(one_sided, Err(LedgerError::Unbalanced { .. })));
    assert!(build_entry(vec![]).is_err());
    assert!(
        build_entry(vec![LedgerLine::new(
            AccountId::new(1),
            Side::Dr,
            rupees(100)
        )])
        .is_err()
    );
    // Many lines that nearly balance: off by one rupee at the end of a long entry.
    let mut lines: Vec<LedgerLine> = (0..199)
        .map(|i| LedgerLine::new(AccountId::new(1 + i % 3), Side::Dr, rupees(10)))
        .collect();
    lines.push(LedgerLine::new(AccountId::new(9), Side::Cr, rupees(1_989)));
    assert!(matches!(
        build_entry(lines),
        Err(LedgerError::Unbalanced { .. })
    ));
}

#[test]
fn the_database_refuses_an_unbalanced_or_changed_entry_even_when_the_code_is_bypassed() {
    // Layer 2, the triggers: raw SQL, as a bug (or a tool outside the app) would do it.
    let w = world(1);
    let before = fingerprint(&w.dir);
    let conn = raw(&w.dir);
    let cash = w.cash;
    let capital = w.capital;
    let try_sql = |label: &str, statements: &[&str]| {
        let result: Result<(), _> = (|| {
            conn.execute_batch("BEGIN")?;
            for statement in statements {
                if let Err(error) = conn.execute_batch(statement) {
                    let _ = conn.execute_batch("ROLLBACK");
                    return Err(error);
                }
            }
            let _ = conn.execute_batch("ROLLBACK");
            Ok(())
        })();
        assert!(result.is_err(), "{label}: the database accepted it");
        let text = result.unwrap_err().to_string();
        assert!(
            text.contains("ERR_") || text.contains("CHECK") || text.contains("constraint"),
            "{label}: {text}"
        );
    };
    let draft = "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id) VALUES ('2026-10-10', 'general', 'voucher', 1)";
    let line = |no: i64, account: i64, dr: i64, cr: i64| {
        format!(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (last_insert_rowid(), {no}, {account}, {dr}, {cr})"
        )
    };
    let post = "UPDATE journal_entries SET status = 'posted', total_paisa = 10000, chain_hash = '0000000000000000000000000000000000000000000000000000000000000000' WHERE id = (SELECT MAX(id) FROM journal_entries)";
    try_sql(
        "born posted",
        &[
            "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, status, total_paisa) VALUES ('2026-10-10', 'general', 'voucher', 1, 'posted', 100)",
        ],
    );
    try_sql("no lines", &[draft, post]);
    try_sql(
        "debit 100, credit 99",
        &[
            draft,
            &line(1, cash, 10_000, 0),
            &line(2, capital, 0, 9_900),
            post,
        ],
    );
    try_sql(
        "total says more than the lines",
        &[
            draft,
            &line(1, cash, 10_000, 0),
            &line(2, capital, 0, 10_000),
            "UPDATE journal_entries SET status = 'posted', total_paisa = 20000, chain_hash = '0000000000000000000000000000000000000000000000000000000000000000' WHERE id = (SELECT MAX(id) FROM journal_entries)",
        ],
    );
    try_sql(
        "a line with debit and credit",
        &[draft, &line(1, cash, 10_000, 10_000)],
    );
    try_sql("a line with neither", &[draft, &line(1, cash, 0, 0)]);
    try_sql("a negative line", &[draft, &line(1, cash, -10_000, 0)]);
    try_sql(
        "paisa that are not whole rupees",
        &[draft, &line(1, cash, 10_050, 0)],
    );
    try_sql(
        "a line on no account",
        &[draft, &line(1, 987_654, 10_000, 0)],
    );
    try_sql(
        "a line above the maximum",
        &[draft, &line(1, cash, 100_000_000_000_100, 0)],
    );
    try_sql(
        "a repeated line number",
        &[
            draft,
            &line(1, cash, 10_000, 0),
            &line(1, capital, 0, 10_000),
        ],
    );
    // A posted entry cannot be changed, deleted or added to.
    let posted: i64 = one(
        &conn,
        "SELECT MIN(id) FROM journal_entries WHERE status = 'posted'",
    );
    try_sql(
        "change a posted date",
        &[&format!(
            "UPDATE journal_entries SET entry_date = '2020-01-01' WHERE id = {posted}"
        )],
    );
    try_sql(
        "back to draft",
        &[&format!(
            "UPDATE journal_entries SET status = 'draft' WHERE id = {posted}"
        )],
    );
    try_sql(
        "delete a posted entry",
        &[&format!("DELETE FROM journal_entries WHERE id = {posted}")],
    );
    try_sql(
        "change a posted line",
        &[&format!(
            "UPDATE journal_lines SET dr_paisa = dr_paisa + 100, cr_paisa = cr_paisa WHERE entry_id = {posted} AND dr_paisa > 0"
        )],
    );
    try_sql(
        "delete a posted line",
        &[&format!(
            "DELETE FROM journal_lines WHERE entry_id = {posted}"
        )],
    );
    try_sql(
        "add a line to a posted entry",
        &[&format!(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES ({posted}, 99, {cash}, 100, 0)"
        )],
    );
    try_sql(
        "replace a posted entry",
        &[&format!(
            "INSERT OR REPLACE INTO journal_entries (id, entry_date, posting_type, source_type, source_id) VALUES ({posted}, '2026-10-10', 'general', 'voucher', 1)"
        )],
    );
    try_sql(
        "replace a posted line",
        &[&format!(
            "INSERT OR REPLACE INTO journal_lines (id, entry_id, line_no, account_id, dr_paisa, cr_paisa) SELECT id, entry_id, line_no, account_id, dr_paisa + 100, cr_paisa FROM journal_lines WHERE entry_id = {posted} AND dr_paisa > 0"
        )],
    );
    // The other append-only tables.
    // (The other append-only tables are empty until D3; db/tests/triggers.rs covers them.)
    {
        let table = "audit_log";
        try_sql(
            &format!("delete from {table}"),
            &[&format!("DELETE FROM {table}")],
        );
        try_sql(
            &format!("update {table}"),
            &[&format!("UPDATE {table} SET id = id + 1000000")],
        );
    }
    assert_eq!(
        fingerprint(&w.dir),
        before,
        "an attack that was refused left a trace"
    );
    assert_books_balance(&w.dir, "the trigger attack");
}

#[test]
fn every_posting_path_refuses_a_closed_date_and_the_day_after_today() {
    let mut w = world(2);
    close_through(&w, "05-09-2026");
    let closed_dates = ["05-09-2026", "04-09-2026", "31-08-2026"];
    for date in closed_dates {
        let before = fingerprint(&w.dir);
        let seller = w.sellers[0];
        let advance = vouchers::save_advance(
            &w.env.ctx,
            &w.owner,
            &AdvanceInput {
                confirm_cash_below_zero: true,
                date: date.into(),
                seller_account_id: seller,
                amount: "100".into(),
                removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
                expected_goods: String::new(),
                narration: String::new(),
            },
        );
        assert_eq!(
            code_of(advance.unwrap_err()),
            ErrorCode::ErrDateClosed,
            "{date}"
        );
        let mut form = account_input(AccountType::Buyer, "ناصر", "Nasir");
        form.opening_date = date.into();
        form.opening_dr = "100".into();
        assert_eq!(
            code_of(accounts::save(&w.env.ctx, &w.owner, &form).unwrap_err()),
            ErrorCode::ErrDateClosed,
            "{date}"
        );
        let accounts_line = OpeningLineInput {
            account_id: w.buyer,
            version: accounts::get(&w.env.ctx, w.buyer).unwrap().version,
            opening_date: date.into(),
            opening_dr: "100".into(),
            opening_cr: String::new(),
            opening_is_advance: false,
        };
        let opened = accounts::openings_save(
            &w.env.ctx,
            &w.owner,
            &OpeningsInput {
                lines: vec![accounts_line],
            },
        );
        assert_eq!(
            code_of(opened.unwrap_err()),
            ErrorCode::ErrDateClosed,
            "{date}"
        );
        assert_eq!(
            fingerprint(&w.dir),
            before,
            "{date}: a refusal wrote something"
        );
    }
    // The day after today is refused too, by every path.
    let before = fingerprint(&w.dir);
    let tomorrow = AdvanceInput {
        confirm_cash_below_zero: true,
        date: "11-10-2026".into(),
        seller_account_id: w.sellers[0],
        amount: "100".into(),
        removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
        expected_goods: String::new(),
        narration: String::new(),
    };
    assert_eq!(
        code_of(vouchers::save_advance(&w.env.ctx, &w.owner, &tomorrow).unwrap_err()),
        ErrorCode::ErrDateInFuture
    );
    assert_eq!(fingerprint(&w.dir), before);
    // Reversing an entry on a closed day is refused; on an open day it works.
    let open_one = vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &AdvanceInput {
            confirm_cash_below_zero: true,
            date: "06-09-2026".into(),
            ..tomorrow.clone()
        },
    )
    .unwrap();
    close_through(&w, "06-09-2026");
    let before = fingerprint(&w.dir);
    let reversed =
        w.env.ctx.db().write(|tx| {
            ledger::reverse(&w.env.ctx, tx, w.owner.user_id, open_one.entry_id, "late")
        });
    assert_eq!(code_of(reversed.unwrap_err()), ErrorCode::ErrDateClosed);
    assert_eq!(fingerprint(&w.dir), before);
    // The window follows the clock: a day later, the day after today becomes open.
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_advance(&w.env.ctx, &w.owner, &tomorrow).unwrap();
    w.rng = Rng(0);
    assert_books_balance(&w.dir, "the closed date attack");
}

fn close_through(w: &World, date: &str) {
    day::close(&w.env.ctx, &w.owner, &DayCloseInput { up_to: date.into() }).unwrap();
}

fn code_of(error: ServiceError) -> ErrorCode {
    match error {
        ServiceError::Rejected(items) => items[0].code,
        other => panic!("not a rejection: {other:?}"),
    }
}

#[test]
fn the_clock_moved_back_never_opens_a_closed_day() {
    // NFR-REL-11: entry dates come from the user; the closed-up-to date does not follow the
    // system clock. Close up to 09-10, then set the clock a year back and a year forward.
    let w = world(3);
    close_through(&w, "09-10-2026");
    for (y, m, d) in [(2025, 1, 1), (2026, 9, 1), (2027, 1, 1)] {
        set_today(&w.env, y, m, d);
        let status = day::status(&w.env.ctx).unwrap();
        assert_eq!(status.closed_up_to.as_deref(), Some("2026-10-09"));
        let before = fingerprint(&w.dir);
        let result = vouchers::save_advance(
            &w.env.ctx,
            &w.owner,
            &AdvanceInput {
                confirm_cash_below_zero: true,
                date: "09-10-2026".into(),
                seller_account_id: w.sellers[0],
                amount: "10".into(),
                removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
                expected_goods: String::new(),
                narration: String::new(),
            },
        );
        assert!(
            result.is_err(),
            "a closed day opened when the clock said {y}-{m}-{d}"
        );
        assert_eq!(fingerprint(&w.dir), before);
    }
}
