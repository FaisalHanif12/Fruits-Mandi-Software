//! Balance index attack (migration 0002; PRD FR-6.1, FR-6.2, FR-6.3, FR-10.4; NFR-REL-8,
//! NFR-REL-13, NFR-SCL-8, NFR-SPD-3, NFR-SPD-21). Owner: `robustness`.
//!
//! Migration 0002 changed how every balance and the last entry date are read (covering index,
//! partial indexes, a two step read for busy accounts). These tests try to make the new reads
//! give a different number from the plain definition "sum of the posted lines of the account":
//!
//! 1. `last_entry_date_and_every_balance_are_exact_around_the_1000_line_step`: accounts with
//!    998, 999, 1000, 1001 and 1002 lines, entries with one and with two lines on the account,
//!    the latest date placed on the first line, the middle line, the last line and on the lines
//!    just inside and just outside the 1,000 newest lines, and a newest entry that is back-dated.
//!    Every number is compared with the old join (`journal_lines JOIN journal_entries`).
//! 2. `a_draft_inside_a_save_is_never_part_of_any_balance`: a draft entry inside the
//!    transaction that is posting it, read on the same connection and on a second one, on a busy
//!    and on a small account, then posted, then rolled back.
//! 3. `a_draft_left_in_a_damaged_file_is_never_part_of_any_balance`: a committed draft.
//! 4. `an_old_file_migrated_to_0002_keeps_every_number` (with and without a left-over draft):
//!    a populated file is put back to the shape of an old release (schema version 1 or 2: a new
//!    file made from the released migration texts of the db crate, with every row copied in),
//!    opened again through the real start-up path, and every number, every table row and both
//!    hash chains are compared. The copy taken before the upgrade is checked. Migrations 0002 to
//!    0006 all run on that file; what 0006 adds (eight tables, ten columns, two controls) is left
//!    out of the row comparison, and nothing else is.
//! 5. `a_system_account_cannot_be_made_inactive_and_a_refusal_writes_nothing`.
#![cfg(test)]

mod common;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use common::*;
use mandi_services::auth::{self, Actor};
use mandi_services::core::accounts::{
    AccountFilter, AccountInput, AccountType, AccountsListInput, Balance, BalanceSide,
};
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::ids::AccountId;
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::AdvanceInput;
use mandi_services::db::ledger as repo;
use mandi_services::db::migrate::{BALANCE_INDEXES_SQL, INIT_SQL};
use mandi_services::db::rusqlite::{Connection, OpenFlags, params, types::Value};
use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
use mandi_services::ledger::{self, PostingEvent};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, assist};
use tempfile::TempDir;

const INDEXES_OF_0002: [&str; 3] = [
    "idx_journal_lines_account_amounts",
    "idx_journal_entries_unposted",
    "idx_journal_lines_advance",
];
const TRIGGER_OF_0003: &str = "accounts_system_stays_active";
/// The triggers of migration 0004 (plan 13: the بکری tables lose no row, a number is final).
const TRIGGERS_OF_0004: [&str; 7] = [
    "bikri_entries_no_delete",
    "bikri_lots_no_delete",
    "bikri_sales_no_delete",
    "bikri_entries_number_is_final",
    "bikri_entries_no_replace",
    "bikri_lots_no_replace",
    "bikri_sales_no_replace",
];
const LATEST_VERSION: i64 = 10;
/// The tables migration 0006 adds (plan 16b). A file of an old release does not have them, and
/// they are empty in this test, so the row fingerprint leaves them out.
const TABLES_OF_0006: [&str; 8] = [
    "ventures",
    "venture_partners",
    "venture_entries",
    "venture_entry_parts",
    "purchases",
    "purchase_cost_parts",
    "lot_credit_parts",
    "fruit_takes",
];
/// The columns migration 0006 adds to old tables. A row's numbers do not change when they appear,
/// so the row fingerprint leaves them out.
const COLUMNS_OF_0006: [(&str, &str); 10] = [
    ("bikri_lots", "purchase_id"),
    ("bikri_lots", "qty_taken"),
    ("bikri_sales", "cost_paisa"),
    ("stock_moves", "purchase_id"),
    ("stock_moves", "cost_paisa"),
    ("analytics_facts", "purchase_id"),
    ("analytics_facts", "venture_id"),
    ("analytics_facts", "cost_paisa"),
    ("partners", "venture_account_id"),
    ("accounts", "tag"),
];
/// The table migration 0009 adds (plan 33 v1.1). A file of an old release does not have it, and it
/// is empty in this test, so the row fingerprint leaves it out.
const TABLES_OF_0009: [&str; 1] = ["garden_arrivals"];
/// The columns migration 0009 adds to old tables. A row's numbers do not change when they appear,
/// so the row fingerprint leaves them out. (`purchases.sell_date` is in a table of 0006.) The two
/// `venture_entries` columns are migration 0010's (plan 35, the shop split).
const COLUMNS_OF_0009: [(&str, &str); 4] = [
    ("bikri_lots", "arrival_id"),
    ("stock_moves", "arrival_id"),
    ("venture_entries", "arrival_id"),
    ("venture_entries", "moved_qty"),
];
/// The two controls whose party types (separate, garden) the CHECK lists of an old release do not
/// hold. Migration 0006 seeds them, so they are not copied into the old file and are left out of
/// the row fingerprint.
const CONTROLS_OF_0006: [&str; 2] = ["0305", "0306"];
/// The three nullable columns migration 0005 adds (plan 16). A row's numbers do not change when
/// they appear, so the row fingerprint leaves them out.
const COLUMNS_OF_0005: [(&str, &str); 3] = [
    ("journal_entries", "order_key"),
    ("stock_moves", "reverses_move_id"),
    ("bikri_sales", "ended"),
];
const NOW: &str = "2026-10-10T09:00:00Z";

// ---- small helpers ----

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
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

/// The date `offset` days after 2026-08-01, as YYYY-MM-DD. Offset 70 is today (2026-10-10).
fn day_iso(offset: i64) -> String {
    (chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap() + chrono::Duration::days(offset))
        .format("%Y-%m-%d")
        .to_string()
}

const TODAY_OFFSET: i64 = 70;

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

struct World {
    dir: TempDir,
    env: TestEnv,
    owner: Actor,
    cash: i64,
    capital: i64,
}

fn id_of(env: &TestEnv, code: &str) -> i64 {
    env.ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, code))
        .unwrap()
        .unwrap()
}

fn world() -> World {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    set_today(&env, 2026, 10, 10);
    let cash = id_of(&env, "01010001");
    let capital = id_of(&env, "06010001");
    World {
        dir,
        env,
        owner,
        cash,
        capital,
    }
}

fn new_account(w: &World, kind: AccountType, ur: &str, en: &str) -> i64 {
    accounts::save(&w.env.ctx, &w.owner, &account_input(kind, ur, en))
        .unwrap()
        .id
}

/// Posts one entry for each item of `plan`, in one transaction: `(day offset, lines on the
/// account)`. Each line is a debit on `account`; one credit on capital balances the entry.
/// Returns the entry ids and the debit total in rupees.
fn post_plan(w: &World, account: i64, plan: &[(i64, usize)]) -> (Vec<i64>, i64) {
    let mut total_all = 0_i64;
    let ids = w
        .env
        .ctx
        .db()
        .write(|tx| -> Result<Vec<i64>, ServiceError> {
            let mut ids = Vec::new();
            for (i, (offset, n)) in plan.iter().enumerate() {
                let mut lines = Vec::new();
                let mut total = 0_i64;
                for k in 0..*n {
                    let amount = 1 + i64::try_from((i * 7 + k * 3) % 97).unwrap();
                    total += amount;
                    lines.push(LedgerLine::new(
                        AccountId::new(account),
                        Side::Dr,
                        rupees(amount),
                    ));
                }
                lines.push(LedgerLine::new(
                    AccountId::new(w.capital),
                    Side::Cr,
                    rupees(total),
                ));
                total_all += total;
                let posted = ledger::post(
                    &w.env.ctx,
                    tx,
                    w.owner.user_id,
                    &PostingEvent {
                        date: BusinessDate::parse_iso(&day_iso(*offset)).unwrap(),
                        posting_type: PostingType::General,
                        source_type: SourceType::Voucher,
                        source_id: 901,
                        narration: None,
                        reverses: None,
                        entry: build_entry(lines).unwrap(),
                    },
                )?;
                ids.push(posted.entry_id);
            }
            Ok(ids)
        })
        .unwrap();
    (ids, total_all)
}

// ---- the old queries, as the oracle ----

fn old_totals(conn: &Connection, account: i64, as_of: Option<&str>) -> (i64, i64) {
    conn.query_row(
        "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE e.status = 'posted' AND l.account_id = ?1 AND (?2 IS NULL OR e.entry_date <= ?2)",
        params![account, as_of],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .unwrap()
}

fn old_last_date(conn: &Connection, account: i64) -> Option<String> {
    conn.query_row(
        "SELECT MAX(e.entry_date) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND e.status = 'posted'",
        [account],
        |r| r.get(0),
    )
    .unwrap()
}

fn old_all_accounts(conn: &Connection, as_of: Option<&str>) -> Vec<(i64, i64, i64)> {
    let mut statement = conn
        .prepare(
            "SELECT l.account_id, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE e.status = 'posted' AND (?1 IS NULL OR e.entry_date <= ?1)
             GROUP BY l.account_id ORDER BY l.account_id",
        )
        .unwrap();
    statement
        .query_map([as_of], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn side_and_amount(dr: i64, cr: i64) -> (BalanceSide, i64) {
    match dr.cmp(&cr) {
        std::cmp::Ordering::Greater => (BalanceSide::Dr, dr - cr),
        std::cmp::Ordering::Less => (BalanceSide::Cr, cr - dr),
        std::cmp::Ordering::Equal => (BalanceSide::Zero, 0),
    }
}

fn assert_balance_is(balance: &Balance, dr: i64, cr: i64, note: &str) {
    let (side, amount) = side_and_amount(dr, cr);
    assert_eq!(balance.side, side, "{note}: side");
    assert_eq!(balance.amount_paisa.as_paisa(), amount, "{note}: amount");
}

/// All rows of all tables (except those named), hashed.
fn fingerprint(dir: &TempDir, skip: &[&str]) -> u64 {
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
    for table in tables.iter().filter(|t| {
        !skip.contains(&t.as_str())
            && !TABLES_OF_0006.contains(&t.as_str())
            && !TABLES_OF_0009.contains(&t.as_str())
    }) {
        table.hash(&mut hasher);
        let names: Vec<String> = {
            let mut info = conn
                .prepare(&format!("PRAGMA table_info(\"{table}\")"))
                .unwrap();
            info.query_map([], |r| r.get::<_, String>(1))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
                .into_iter()
                .filter(|c| {
                    !COLUMNS_OF_0005.contains(&(table.as_str(), c.as_str()))
                        && !COLUMNS_OF_0006.contains(&(table.as_str(), c.as_str()))
                        && !COLUMNS_OF_0009.contains(&(table.as_str(), c.as_str()))
                })
                .collect()
        };
        let filter = if table == "account_controls" {
            format!(
                "WHERE code NOT IN ({})",
                CONTROLS_OF_0006
                    .iter()
                    .map(|c| format!("'{c}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        };
        let list: Vec<String> = names.iter().map(|c| format!("\"{c}\"")).collect();
        let mut statement = conn
            .prepare(&format!(
                "SELECT {} FROM \"{table}\" {filter} ORDER BY rowid",
                list.join(", ")
            ))
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

// ---- 1. the 1,000 line step ----

#[derive(Clone, Copy, Debug)]
enum Latest {
    /// Dates go up with the entry id (the normal case).
    Ascending,
    /// Dates go up, but the newest entry is back-dated to the second day.
    NewestBackdated,
    /// The entry that owns this line (counting from the oldest, from 0) has the latest date;
    /// every other entry is dated earlier.
    Line(usize),
}

/// The plan for one account: `(day offset, lines)` for each entry, `n_lines` lines in all.
fn plan_for(
    n_lines: usize,
    two_per_entry: bool,
    latest: Latest,
    rng: &mut Rng,
) -> Vec<(i64, usize)> {
    let mut sizes = Vec::new();
    let mut left = n_lines;
    let mut i = 0_usize;
    while left > 0 {
        let want = if two_per_entry && i.is_multiple_of(2) {
            2
        } else {
            1
        };
        let take = want.min(left);
        sizes.push(take);
        left -= take;
        i += 1;
    }
    let entries = sizes.len();
    let mut line_of_first = Vec::new();
    let mut at = 0_usize;
    for size in &sizes {
        line_of_first.push(at);
        at += size;
    }
    sizes
        .iter()
        .enumerate()
        .map(|(index, size)| {
            let ascending = i64::try_from(index * 60 / entries.max(1)).unwrap();
            let day = match latest {
                Latest::Ascending => ascending,
                Latest::NewestBackdated => {
                    if index + 1 == entries {
                        1
                    } else {
                        ascending
                    }
                }
                Latest::Line(k) => {
                    let first = line_of_first[index];
                    if k >= first && k < first + size {
                        TODAY_OFFSET
                    } else {
                        i64::try_from(rng.below(61)).unwrap()
                    }
                }
            };
            (day, *size)
        })
        .collect()
}

fn check_one_account(
    w: &World,
    account: i64,
    plan: &[(i64, usize)],
    total_rupees: i64,
    label: &str,
) {
    let max_day = plan.iter().map(|(d, _)| *d).max().unwrap();
    let wanted = Some(day_iso(max_day));
    let conn = raw(&w.dir);
    assert_eq!(
        old_last_date(&conn, account),
        wanted,
        "{label}: the oracle itself"
    );
    let lines: usize = plan.iter().map(|(_, n)| n).sum();

    // The repository read and the party card.
    let last = w
        .env
        .ctx
        .db()
        .read(|c| repo::last_entry_date(c, account))
        .unwrap();
    assert_eq!(last, wanted, "{label}: last_entry_date ({lines} lines)");
    let card = assist::party_card(&w.env.ctx, account).unwrap();
    assert_eq!(
        card.last_entry_date, wanted,
        "{label}: party card last date"
    );

    // The balance, three ways.
    let paisa = total_rupees * 100;
    assert_balance_is(
        &ledger::account_balance(&w.env.ctx, account).unwrap(),
        paisa,
        0,
        &format!("{label}: account_balance"),
    );
    assert_balance_is(
        &card.balance,
        paisa,
        0,
        &format!("{label}: party card balance"),
    );
    let (dr, cr) = old_totals(&conn, account, None);
    assert_eq!((dr, cr), (paisa, 0), "{label}: the oracle total");

    // The same balance as of days around the dates in the plan (the "leave out later entries"
    // read), and the whole-journal pass with and without a day.
    let mut days: Vec<String> = vec![
        day_iso(-1),
        day_iso(0),
        day_iso(max_day - 1),
        day_iso(max_day),
    ];
    let mut sorted: Vec<i64> = plan.iter().map(|(d, _)| *d).collect();
    sorted.sort_unstable();
    days.push(day_iso(sorted[sorted.len() / 2]));
    days.push(day_iso(TODAY_OFFSET));
    for day in &days {
        let got = w
            .env
            .ctx
            .db()
            .read(|c| repo::totals_of(c, account, None, Some(day)))
            .unwrap();
        let want = old_totals(&conn, account, Some(day));
        assert_eq!(
            (got.dr_paisa, got.cr_paisa),
            want,
            "{label}: totals as of {day}"
        );
    }
}

#[test]
fn last_entry_date_and_every_balance_are_exact_around_the_1000_line_step() {
    let w = world();
    let mut rng = Rng(0x1015_2026);
    let mut cases = 0_u32;
    for &n in &[998_usize, 999, 1000, 1001, 1002] {
        for two_per_entry in [false, true] {
            // The two line variant is run for the three sizes named in the task and for the
            // neighbours only with one line per entry, to keep the run short.
            if two_per_entry && (n == 998 || n == 1002) {
                continue;
            }
            let mut latests = vec![
                Latest::Ascending,
                Latest::NewestBackdated,
                Latest::Line(0),
                Latest::Line(n / 2),
                Latest::Line(n - 1),
            ];
            if n >= 1001 {
                latests.push(Latest::Line(n - 1001));
            }
            if n >= 1000 {
                latests.push(Latest::Line(n - 1000));
            }
            for latest in latests {
                let plan = plan_for(n, two_per_entry, latest, &mut rng);
                let account = new_account(&w, AccountType::Buyer, "ناصر", "Nasir");
                let (_, total) = post_plan(&w, account, &plan);
                let label = format!("n={n} two_per_entry={two_per_entry} {latest:?}");
                check_one_account(&w, account, &plan, total, &label);
                cases += 1;
            }
        }
    }
    assert!(cases >= 40, "only {cases} cases ran");

    // The whole journal: the pass over every account gives the numbers of the old join, for all
    // time and for days, the books balance, and the hash chain still holds.
    let conn = raw(&w.dir);
    for as_of in [
        None,
        Some("2026-08-01"),
        Some("2026-09-15"),
        Some("2026-10-10"),
    ] {
        let got = w
            .env
            .ctx
            .db()
            .read(|c| repo::account_totals(c, as_of))
            .unwrap();
        let got: Vec<(i64, i64, i64)> = got
            .into_iter()
            .map(|(id, t)| (id, t.dr_paisa, t.cr_paisa))
            .collect();
        assert_eq!(
            got,
            old_all_accounts(&conn, as_of),
            "account_totals as of {as_of:?}"
        );
    }
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(
        trial.total_dr, trial.total_cr,
        "the trial balance must balance"
    );
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

// ---- 2. and 3. drafts ----

/// A draft with three lines: the busy account, a small account and capital.
fn insert_draft_with_lines(
    tx: &Connection,
    user_id: i64,
    date: &str,
    busy: i64,
    small: i64,
    capital: i64,
) -> (i64, i64) {
    let id = repo::insert_draft(
        tx,
        &repo::EntryHeader {
            entry_date: date,
            posting_type: "general",
            source_type: "voucher",
            source_id: 902,
            reverses_entry_id: None,
            narration: None,
        },
        user_id,
        NOW,
    )
    .unwrap();
    let line = |no: i64, account: i64, dr: i64, cr: i64| repo::NewLine {
        line_no: no,
        account_id: account,
        person_key: None,
        dr_paisa: dr,
        cr_paisa: cr,
        is_advance: false,
        narration: None,
    };
    repo::insert_line(tx, id, &line(1, busy, 500_000_000, 0), user_id, NOW).unwrap();
    repo::insert_line(tx, id, &line(2, small, 700_000, 0), user_id, NOW).unwrap();
    repo::insert_line(tx, id, &line(3, capital, 0, 500_700_000), user_id, NOW).unwrap();
    (id, 500_700_000)
}

/// Every balance read, on one connection, for the accounts in `watch`.
fn read_everything(conn: &Connection, watch: &[i64]) -> Vec<String> {
    let mut out = Vec::new();
    for &id in watch {
        out.push(format!(
            "totals_of {id} {:?}",
            repo::totals_of(conn, id, None, None).unwrap()
        ));
        out.push(format!(
            "totals_of {id} as of today {:?}",
            repo::totals_of(conn, id, None, Some("2026-10-10")).unwrap()
        ));
        out.push(format!(
            "last date {id} {:?}",
            repo::last_entry_date(conn, id).unwrap()
        ));
    }
    let map = repo::totals_for_accounts(conn, watch).unwrap();
    for &id in watch {
        out.push(format!("for_accounts {id} {:?}", map.get(&id)));
    }
    for as_of in [None, Some("2026-10-10")] {
        let all = repo::account_totals(conn, as_of).unwrap();
        for &id in watch {
            out.push(format!(
                "account_totals {as_of:?} {id} {:?}",
                all.iter().find(|(a, _)| *a == id).map(|(_, t)| *t)
            ));
        }
    }
    out
}

fn busy_and_small(w: &World) -> (i64, i64) {
    let busy = new_account(w, AccountType::Buyer, "ناصر", "Nasir");
    let small = new_account(w, AccountType::Seller, "کریم", "Karim");
    // 1,001 lines on the busy account, none dated today, so a counted draft changes the last
    // date and the balance as of today.
    let plan = plan_for(1001, false, Latest::Ascending, &mut Rng(5));
    post_plan(w, busy, &plan);
    post_plan(w, small, &[(3, 1), (4, 1), (5, 2)]);
    (busy, small)
}

#[test]
fn a_draft_inside_a_save_is_never_part_of_any_balance() {
    let w = world();
    let (busy, small) = busy_and_small(&w);
    let watch = [busy, small, w.cash, w.capital];
    let reader_before = read_everything(&raw(&w.dir), &watch);
    let key = w.env.ctx.db().auditor().key().unwrap();
    let user = w.owner.user_id;
    let (cash, capital) = (w.cash, w.capital);

    let result = w.env.ctx.db().write(|tx| -> Result<(), ServiceError> {
        let (draft, total) = insert_draft_with_lines(tx, user, "2026-10-10", busy, small, capital);
        // The same connection, with the draft in it: not one read counts it.
        assert_eq!(
            read_everything(tx, &watch),
            reader_before,
            "a read inside the save counted the draft"
        );
        // The old definition agrees that the draft is not posted.
        assert_eq!(
            old_totals(tx, busy, None).0,
            old_totals(&raw(&w.dir), busy, None).0
        );
        // A second connection reads while the save is open: it sees the last commit only.
        assert_eq!(read_everything(&raw(&w.dir), &watch), reader_before);
        // Posting it makes it count, everywhere, at once.
        repo::post(tx, &key, draft, total, user, NOW).map_err(ServiceError::Db)?;
        let after = read_everything(tx, &watch);
        assert_ne!(
            after, reader_before,
            "a posted entry must change the numbers"
        );
        let got = repo::totals_of(tx, busy, None, None)?;
        let before = old_totals(&raw(&w.dir), busy, None);
        assert_eq!(got.dr_paisa, before.0 + 500_000_000);
        assert_eq!(
            repo::last_entry_date(tx, busy)?.as_deref(),
            Some("2026-10-10")
        );
        let _ = cash;
        Ok(())
    });
    result.unwrap();

    // After the commit, the services see the posted entry, and the books balance.
    let conn = raw(&w.dir);
    let (dr, cr) = old_totals(&conn, busy, None);
    assert_balance_is(
        &ledger::account_balance(&w.env.ctx, busy).unwrap(),
        dr,
        cr,
        "busy after",
    );
    assert_eq!(
        assist::party_card(&w.env.ctx, busy)
            .unwrap()
            .last_entry_date
            .as_deref(),
        Some("2026-10-10")
    );
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);

    // A save that fails after the draft was written leaves no draft behind and no number moved.
    let before_fail = read_everything(&conn, &watch);
    let failed = w.env.ctx.db().write(|tx| -> Result<(), ServiceError> {
        insert_draft_with_lines(tx, user, "2026-10-10", busy, small, capital);
        assert_eq!(
            read_everything(tx, &watch),
            before_fail,
            "draft counted before the failure"
        );
        Err(ServiceError::reject(ErrorCode::ErrJournalUnbalanced))
    });
    assert!(failed.is_err());
    let conn = raw(&w.dir);
    assert_eq!(read_everything(&conn, &watch), before_fail);
    let unposted: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(unposted, 0, "a rolled back save left a draft");
    let lines_of_nothing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM journal_lines l LEFT JOIN journal_entries e ON e.id = l.entry_id WHERE e.id IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(lines_of_nothing, 0);
}

#[test]
fn a_draft_left_in_a_damaged_file_is_never_part_of_any_balance() {
    let w = world();
    let (busy, small) = busy_and_small(&w);
    let watch = [busy, small, w.cash, w.capital];
    let before = read_everything(&raw(&w.dir), &watch);
    let trial_before = ledger::trial_balance(&w.env.ctx, None).unwrap();
    let card_before = assist::party_card(&w.env.ctx, busy).unwrap();
    let list_input = AccountsListInput {
        sort: None,
        filter: AccountFilter {
            kind: mandi_services::core::accounts::AccountListKind::All,
            text: String::new(),
            include_inactive: true,
            has_opening: None,
        },
        offset: 0,
        limit: 0,
    };
    let list_before = accounts::list(&w.env.ctx, &list_input).unwrap();

    // A draft that was committed (a damaged or hand-edited file, not something the app does).
    let conn = raw(&w.dir);
    insert_draft_with_lines(&conn, w.owner.user_id, "2026-10-10", busy, small, w.capital);
    assert_eq!(
        read_everything(&conn, &watch),
        before,
        "raw reads counted a left-over draft"
    );
    assert_eq!(read_everything(&raw(&w.dir), &watch), before);

    // The services leave it out too.
    assert_eq!(
        ledger::account_balance(&w.env.ctx, busy).unwrap(),
        card_before.balance
    );
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial, trial_before);
    assert_eq!(trial.total_dr, trial.total_cr);
    let card = assist::party_card(&w.env.ctx, busy).unwrap();
    assert_eq!(card, card_before);
    let list = accounts::list(&w.env.ctx, &list_input).unwrap();
    let balances = |l: &mandi_services::core::accounts::AccountsList| -> Vec<String> {
        l.rows
            .iter()
            .map(|r| format!("{} {:?}", r.id, r.balance))
            .collect()
    };
    assert_eq!(balances(&list), balances(&list_before));
}

// ---- 4. an old file migrated to 0002 ----

/// A populated file: accounts, an advance, a reversal, a busy back-dated account, a PRD style
/// worked day on cash.
struct Populated {
    watch: Vec<i64>,
    busy: i64,
}

fn populate(w: &World) -> Populated {
    set_today(&w.env, 2026, 10, 10);
    let rahim = new_account(w, AccountType::Seller, "رحیم", "Rahim");
    let karim = new_account(w, AccountType::Seller, "کریم", "Karim");
    let akram = new_account(w, AccountType::Buyer, "اکرم", "Akram");
    let busy = new_account(w, AccountType::Buyer, "ناصر", "Nasir");
    for (seller, amount, date) in [
        (rahim, "15,000", "01-09-2026"),
        (rahim, "2,500", "20-09-2026"),
        (karim, "700", "05-10-2026"),
    ] {
        vouchers_advance(w, seller, amount, date);
    }
    // 1,001 lines on the busy account with the latest date on its oldest line.
    let plan = plan_for(1001, false, Latest::Line(0), &mut Rng(77));
    let (ids, _) = post_plan(w, busy, &plan);
    // A small account with a back-dated entry and a reversal of an entry.
    let (small_ids, _) = post_plan(w, akram, &[(10, 1), (40, 2), (5, 1)]);
    w.env
        .ctx
        .db()
        .write(|tx| ledger::reverse(&w.env.ctx, tx, w.owner.user_id, small_ids[1], "robustness"))
        .unwrap();
    let _ = ids;
    Populated {
        watch: vec![rahim, karim, akram, busy, w.cash, w.capital],
        busy,
    }
}

fn vouchers_advance(w: &World, seller: i64, amount: &str, date: &str) {
    mandi_services::vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &AdvanceInput {
            confirm_cash_below_zero: true,
            date: date.into(),
            seller_account_id: seller,
            amount: amount.into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .unwrap();
}

/// Every number the screens read, as text, plus a check against the old join. Reads need no
/// sign-in.
fn snapshot(env: &TestEnv, dir: &TempDir, watch: &[i64], busy: i64) -> Vec<String> {
    let conn = raw(dir);
    let mut out = Vec::new();
    let ids: Vec<i64> = {
        let mut statement = conn.prepare("SELECT id FROM accounts ORDER BY id").unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    for id in ids {
        let (dr, cr) = old_totals(&conn, id, None);
        let balance = ledger::account_balance(&env.ctx, id).unwrap();
        assert_balance_is(&balance, dr, cr, &format!("account {id}"));
        out.push(format!("account {id} old {dr}/{cr} service {balance:?}"));
    }
    for &id in watch {
        let card = assist::party_card(&env.ctx, id).unwrap();
        out.push(format!("card {id} {card:?}"));
        let l = ledger::ledger(&env.ctx, id, None, None, None).unwrap();
        out.push(format!(
            "ledger {id} lines {} closing {:?}",
            l.lines.len(),
            l.closing
        ));
        let l = ledger::ledger(&env.ctx, id, None, Some("2026-09-01"), Some("2026-09-30")).unwrap();
        out.push(format!(
            "ledger {id} sept opening {:?} closing {:?}",
            l.opening, l.closing
        ));
    }
    for as_of in [
        None,
        Some("2026-08-15"),
        Some("2026-09-15"),
        Some("2026-10-10"),
    ] {
        let trial = ledger::trial_balance(&env.ctx, as_of).unwrap();
        assert_eq!(
            trial.total_dr, trial.total_cr,
            "trial balance as of {as_of:?}"
        );
        out.push(format!("trial {as_of:?} {trial:?}"));
    }
    let list = accounts::list(
        &env.ctx,
        &AccountsListInput {
            sort: None,
            filter: AccountFilter {
                kind: mandi_services::core::accounts::AccountListKind::All,
                text: String::new(),
                include_inactive: true,
                has_opening: None,
            },
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    for r in &list.rows {
        out.push(format!("list {} {:?}", r.id, r.balance));
    }
    let _ = busy;
    out
}

fn index_names(dir: &TempDir) -> Vec<String> {
    let conn = raw(dir);
    let mut statement = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx_%' ORDER BY name")
        .unwrap();
    statement
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn objects(dir: &TempDir) -> Vec<(String, String)> {
    let conn = raw(dir);
    let mut statement = conn
        .prepare(
            "SELECT type, name FROM sqlite_master WHERE type IN ('table','trigger','view') ORDER BY type, name",
        )
        .unwrap();
    statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

/// The names of the columns of a table of an attached database, in table order.
fn column_names(conn: &Connection, schema: &str, table: &str) -> Vec<String> {
    conn.prepare("SELECT name FROM pragma_table_info(?1, ?2) ORDER BY cid")
        .unwrap()
        .query_map([table, schema], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

/// Puts the populated file of `dir` back to the shape of an old release, at schema version
/// `from_version`: 1 (migrations 0002 to 0006 are still to run) or 2 (0003 to 0006 are).
///
/// The old file is made from the released migration texts of the db crate (`INIT_SQL`, and
/// `BALANCE_INDEXES_SQL` for version 2), so its tables, CHECK lists, triggers and indexes are exactly
/// those of that release. Then every row of every table of the populated file is copied in, for the
/// columns the old table has. The journal entries go in as drafts and are posted after their lines,
/// one by one in id order, the way a save does it (Code Standards B16); every column, also the
/// keyed hash, is copied as it was. The two controls whose party types the old CHECK lists do not
/// hold are not copied: migration 0006 seeds them again.
fn put_back_to_old_release(dir: &TempDir, from_version: i64) {
    assert!(from_version == 1 || from_version == 2);
    let live = dir.path().join("data").join("mandi.db");
    let old = dir.path().join("data").join("old-release.db");
    {
        let conn = Connection::open(&old).unwrap();
        conn.execute_batch(INIT_SQL).unwrap();
        if from_version == 2 {
            conn.execute_batch(BALANCE_INDEXES_SQL).unwrap();
        }
        conn.pragma_update(None, "foreign_keys", "OFF").unwrap();
        conn.execute("ATTACH DATABASE ?1 AS latest", [live.to_str().unwrap()])
            .unwrap();
        let tables: Vec<String> = conn
            .prepare(
                "SELECT name FROM main.sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            )
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        for table in &tables {
            let in_latest = column_names(&conn, "latest", table);
            let common: Vec<String> = column_names(&conn, "main", table)
                .into_iter()
                .filter(|c| in_latest.contains(c))
                .collect();
            let list: Vec<String> = common.iter().map(|c| format!("\"{c}\"")).collect();
            let select: Vec<String> = common
                .iter()
                .map(|c| {
                    if table == "journal_entries" && c == "status" {
                        "'draft'".to_owned()
                    } else {
                        format!("\"{c}\"")
                    }
                })
                .collect();
            if table == "day_state" {
                // The text of migration 0001 itself puts the one row in and the table keeps it (a
                // trigger refuses a delete): its columns are set to the real row's.
                let sets: Vec<String> = common
                    .iter()
                    .filter(|c| c.as_str() != "id")
                    .map(|c| {
                        format!("\"{c}\" = (SELECT \"{c}\" FROM latest.day_state WHERE id = 1)")
                    })
                    .collect();
                conn.execute_batch(&format!("UPDATE main.day_state SET {}", sets.join(", ")))
                    .unwrap();
                continue;
            }
            let filter = if table == "account_controls" {
                "WHERE code NOT IN ('0305', '0306')"
            } else {
                ""
            };
            conn.execute_batch(&format!(
                "INSERT INTO main.\"{table}\" ({}) SELECT {} FROM latest.\"{table}\" {filter} ORDER BY rowid",
                list.join(", "),
                select.join(", ")
            ))
            .unwrap_or_else(|e| panic!("copying {table} into the old release failed: {e}"));
        }
        let posted: Vec<i64> = conn
            .prepare("SELECT id FROM latest.journal_entries WHERE status = 'posted' ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        for id in posted {
            conn.execute(
                "UPDATE main.journal_entries SET status = 'posted' WHERE id = ?1",
                [id],
            )
            .unwrap_or_else(|e| panic!("posting entry {id} in the old release failed: {e}"));
        }
        let broken: i64 = conn
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            broken, 0,
            "the copy into the old release broke a foreign key"
        );
        conn.execute_batch("DETACH DATABASE latest").unwrap();
        conn.pragma_update(None, "user_version", from_version)
            .unwrap();
    }
    for suffix in ["", "-wal", "-shm"] {
        let mut path = live.clone().into_os_string();
        path.push(suffix);
        let _ = std::fs::remove_file(path);
    }
    std::fs::rename(&old, &live).unwrap();
}

/// `from_version` is the schema version of the old file: 1 (migrations 0002 to 0006 run) or 2 (0003
/// to 0006 run).
fn migration_case(with_draft: bool, from_version: i64) {
    assert!(from_version == 1 || from_version == 2);
    let w = world();
    let p = populate(&w);
    if with_draft {
        let conn = raw(&w.dir);
        insert_draft_with_lines(
            &conn,
            w.owner.user_id,
            "2026-10-10",
            p.busy,
            p.watch[0],
            w.capital,
        );
    }
    let before = snapshot(&w.env, &w.dir, &p.watch, p.busy);
    let objects_before = objects(&w.dir);
    let busy_total = old_totals(&raw(&w.dir), p.busy, None);
    let World { dir, env, .. } = w;
    drop(env);

    // Put the file back to the shape of an old release (schema version 1 or 2): a new file is made
    // from the released migration texts, and every row is copied in (see the function).
    put_back_to_old_release(&dir, from_version);
    for name in INDEXES_OF_0002 {
        assert_eq!(
            index_names(&dir).iter().any(|n| n == name),
            from_version == 2,
            "{name}: wrong state in the faked version {from_version} file"
        );
    }
    assert!(
        !objects(&dir).iter().any(|(_, n)| n == TRIGGER_OF_0003),
        "{TRIGGER_OF_0003} must be gone"
    );
    for name in TRIGGERS_OF_0004 {
        assert!(
            !objects(&dir).iter().any(|(_, n)| n == name),
            "{name} must be gone"
        );
    }
    assert_eq!(
        old_totals(&raw(&dir), p.busy, None),
        busy_total,
        "dropping indexes changed data"
    );
    let rows_before = fingerprint(&dir, &["audit_log"]);
    let audit_rows_before = audit_total(&dir);
    let old_file_numbers: Vec<(i64, i64, i64)> = old_all_accounts(&raw(&dir), None);

    // The real start-up path: open the file. It migrates.
    let env2 =
        TestEnv::new(dir.path()).unwrap_or_else(|e| panic!("opening the v1 file failed: {e:?}"));

    // The schema, the data and the copy.
    let version: i64 = raw(&dir)
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, LATEST_VERSION, "schema version after opening");
    for name in INDEXES_OF_0002 {
        assert!(
            index_names(&dir).iter().any(|n| n == name),
            "{name} was not created"
        );
    }
    assert_eq!(
        objects(&dir),
        objects_before,
        "a table, trigger or view differs from the latest file (the triggers of 0003 and 0004 must be back)"
    );
    assert_eq!(
        fingerprint(&dir, &["audit_log"]),
        rows_before,
        "the migration changed a row"
    );
    assert!(
        audit_total(&dir) >= audit_rows_before,
        "audit rows were lost"
    );
    assert_eq!(old_all_accounts(&raw(&dir), None), old_file_numbers);
    let integrity: String = raw(&dir)
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    let fk: i64 = raw(&dir)
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(fk, 0, "foreign key problems after the migration");

    let copies: Vec<std::path::PathBuf> = std::fs::read_dir(dir.path().join("backups"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                n.starts_with(&format!("pre-migration-{from_version}-{LATEST_VERSION}-"))
            })
        })
        .collect();
    assert_eq!(
        copies.len(),
        1,
        "exactly one copy before the upgrade: {copies:?}"
    );
    let copy = Connection::open_with_flags(&copies[0], OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let copy_version: i64 = copy
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(copy_version, from_version, "the copy is the old file");
    let copy_integrity: String = copy
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(copy_integrity, "ok");
    assert_eq!(
        old_all_accounts(&copy, None),
        old_file_numbers,
        "the copy holds the old numbers"
    );
    drop(copy);

    // Every number the screens read is identical, and both chains verify.
    let after = snapshot(&env2, &dir, &p.watch, p.busy);
    assert_eq!(after.len(), before.len());
    for (a, b) in before.iter().zip(after.iter()) {
        assert_eq!(a, b, "a number changed in the migration");
    }
    if with_draft {
        // A draft cannot exist in a healthy file (B16). The chain check must say so.
        assert!(
            ledger::verify_journal_chain(&env2.ctx).is_err(),
            "a left-over draft must be reported by the journal chain check"
        );
    } else {
        ledger::verify_journal_chain(&env2.ctx).unwrap();
    }
    assert_chain_ok(&dir);
    drop(env2);

    // Opening it again changes nothing and takes no second copy.
    let env3 = TestEnv::new(dir.path()).unwrap();
    let again = snapshot(&env3, &dir, &p.watch, p.busy);
    assert_eq!(again, before);
    let copies_now = std::fs::read_dir(dir.path().join("backups"))
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_str()
                .is_some_and(|n| n.starts_with("pre-migration-") && n.ends_with(".db"))
        })
        .count();
    assert_eq!(
        copies_now, 1,
        "a file at the latest version took another copy"
    );
}

#[test]
fn an_old_file_migrated_to_0002_keeps_every_number() {
    migration_case(false, 1);
}

#[test]
fn an_old_file_with_a_left_over_draft_migrates_and_still_leaves_the_draft_out() {
    migration_case(true, 1);
}

#[test]
fn a_version_2_file_migrated_to_0003_keeps_every_number() {
    migration_case(false, 2);
}

// ---- 5. system accounts ----

#[test]
fn a_system_account_cannot_be_made_inactive_and_a_refusal_writes_nothing() {
    let w = world();
    // The accounts list shows only sellers and buyers since plan 32, so the system accounts are
    // read from the table.
    struct SystemAccount {
        id: i64,
        code: String,
    }
    let system: Vec<SystemAccount> = {
        let conn = raw(&w.dir);
        let mut statement = conn
            .prepare("SELECT id, code FROM accounts WHERE is_system = 1 ORDER BY code")
            .unwrap();
        statement
            .query_map([], |r| {
                Ok(SystemAccount {
                    id: r.get(0)?,
                    code: r.get(1)?,
                })
            })
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert!(system.len() >= 15, "only {} system accounts", system.len());

    for row in &system {
        // Through set_active.
        let before = fingerprint(&w.dir, &[]);
        let audit_before = audit_total(&w.dir);
        let error = accounts::set_active(&w.env.ctx, &w.owner, row.id, false).unwrap_err();
        match &error {
            ServiceError::Rejected(items) => {
                assert_eq!(items[0].code, ErrorCode::ErrSystemAccount, "{}", row.code);
                assert_eq!(items[0].field.as_deref(), Some("id"), "{}", row.code);
            }
            other => panic!("{}: not a clean rejection: {other:?}", row.code),
        }
        assert_eq!(
            fingerprint(&w.dir, &[]),
            before,
            "{}: a refusal wrote something",
            row.code
        );
        assert_eq!(
            audit_total(&w.dir),
            audit_before,
            "{}: a refusal wrote an audit row",
            row.code
        );
        assert!(
            accounts::get(&w.env.ctx, row.id).unwrap().is_active,
            "{}",
            row.code
        );

        // Asking for the state it already has changes nothing and writes nothing.
        accounts::set_active(&w.env.ctx, &w.owner, row.id, true).unwrap();
        assert_eq!(
            fingerprint(&w.dir, &[]),
            before,
            "{}: a no-op wrote something",
            row.code
        );
    }

    // Through the edit form: the cash account saved with the active box cleared.
    let cash = accounts::get(&w.env.ctx, w.cash).unwrap();
    let mut form = account_input(AccountType::Other, &cash.name_ur, &cash.name_en);
    form.id = Some(cash.id);
    form.version = Some(cash.version);
    form.control_id = Some(cash.control.id);
    form.code = cash.code.clone();
    form.is_active = false;
    let before = fingerprint(&w.dir, &[]);
    let error = accounts::save(&w.env.ctx, &w.owner, &form).unwrap_err();
    match &error {
        ServiceError::Rejected(items) => {
            assert_eq!(
                items[0].code,
                ErrorCode::ErrSystemAccount,
                "edit form: {items:?}"
            );
        }
        other => panic!("edit form: not a clean rejection: {other:?}"),
    }
    assert_eq!(
        fingerprint(&w.dir, &[]),
        before,
        "the edit form refusal wrote something"
    );
    assert!(accounts::get(&w.env.ctx, w.cash).unwrap().is_active);

    // A normal account is not affected by the rule.
    let rahim = new_account(&w, AccountType::Seller, "رحیم", "Rahim");
    assert!(
        !accounts::set_active(&w.env.ctx, &w.owner, rahim, false)
            .unwrap()
            .is_active
    );
    assert!(
        accounts::set_active(&w.env.ctx, &w.owner, rahim, true)
            .unwrap()
            .is_active
    );

    // The database itself refuses the update (migration 0003, PRD FR-10.4): ERR_SYSTEM_ACCOUNT,
    // the account stays active and nothing at all is written.
    let before = fingerprint(&w.dir, &[]);
    let audit_before = audit_total(&w.dir);
    let raw_conn = raw(&w.dir);
    let refused = raw_conn
        .execute("UPDATE accounts SET is_active = 0 WHERE is_system = 1", [])
        .expect_err("the raw update must be refused by the trigger");
    eprintln!("system account, raw is_active = 0: {refused:?}");
    assert!(
        matches!(
            &refused,
            mandi_services::db::rusqlite::Error::SqliteFailure(f, _)
                if f.code == mandi_services::db::rusqlite::ErrorCode::ConstraintViolation
        ),
        "not a constraint violation: {refused:?}"
    );
    assert!(
        refused.to_string().contains("ERR_SYSTEM_ACCOUNT"),
        "wrong message: {refused}"
    );
    let still_inactive: i64 = raw_conn
        .query_row(
            "SELECT COUNT(*) FROM accounts WHERE is_system = 1 AND is_active = 0",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(still_inactive, 0, "a system account was made inactive");
    drop(raw_conn);
    assert_eq!(
        fingerprint(&w.dir, &[]),
        before,
        "the refused raw update wrote something"
    );
    assert_eq!(audit_total(&w.dir), audit_before);

    // A file damaged outside the app, with the trigger gone and a system account set inactive
    // by hand: the app can put it right, and a posting to it works again.
    let raw_conn = raw(&w.dir);
    raw_conn
        .execute_batch(&format!("DROP TRIGGER {TRIGGER_OF_0003}"))
        .unwrap();
    let changed = raw_conn
        .execute("UPDATE accounts SET is_active = 0 WHERE id = ?1", [w.cash])
        .unwrap();
    assert_eq!(changed, 1);
    let refused = ledger::post(
        &w.env.ctx,
        &raw_conn,
        w.owner.user_id,
        &PostingEvent {
            date: BusinessDate::parse_iso("2026-10-10").unwrap(),
            posting_type: PostingType::General,
            source_type: SourceType::Voucher,
            source_id: 903,
            narration: None,
            reverses: None,
            entry: build_entry(vec![
                LedgerLine::new(AccountId::new(w.cash), Side::Dr, rupees(10)),
                LedgerLine::new(AccountId::new(w.capital), Side::Cr, rupees(10)),
            ])
            .unwrap(),
        },
    );
    assert!(
        refused.is_err(),
        "a new posting on an inactive cash account must be refused"
    );
    drop(raw_conn);
    let healed = accounts::set_active(&w.env.ctx, &w.owner, w.cash, true).unwrap();
    assert!(healed.is_active);
    post_plan(&w, w.cash, &[(TODAY_OFFSET, 1)]);
}
