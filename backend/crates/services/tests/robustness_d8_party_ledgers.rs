//! D8 end gate, attack on the party ledgers screen (PRD FR-12.43, FR-19.11, plan 66; NFR-SPD-24; Code
//! Standards B31). Owner: `robustness`.
//!
//! 1. `hostile_filters_and_paging`: SQL, wildcard, control and bidi text, a 100,000 character text, offsets and
//!    limits at and past their ends, bad and reversed dates, wrong accounts. Never a panic or an internal
//!    error; at most 100 rows; the count and the totals do not change with the page; pages put together are the
//!    big page; a text that matches nothing matches nothing (never everybody).
//! 2. `after_random_sales_edits_cancels_and_receipts_every_party_adds_up`: a seeded run of 60 operations on
//!    paid-on-the-spot sales, edits, cancels and receipts. After each: every buyer and seller has a statement
//!    that adds up (`previous` and the day make `balance`, each row starts where the older one ended, the
//!    page totals are the walk totals), its closing is the balance of the account, the party card and the row
//!    of the list; a چکتا buyer is in no list.
//! 3. `at_the_nfr_scl_1_volume_*` (only with `MANDI_PERF_VOLUME=1`, it uses the 2 GB file of `pnpm perf`):
//!    parity of the statement of the busiest buyer and seller with the balance, the card and the list, a walk
//!    of every page, and the 95th percentile of the list and of the first and a later page of a statement.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_save/helpers.rs"]
mod helpers;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;
#[path = "../benches/support/mod.rs"]
mod volume;

use std::time::{Duration, Instant};

use common::assert_chain_ok;
use helpers::*;
use mandi_services::core::accounts::{AccountType, BalanceSide};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::party_ledgers::{
    LedgerSide, PartyLedgerDay, PartyLedgerStatement, PartyLedgerStatementInput, PartyLedgersList,
    PartyLedgersListInput,
};
use mandi_services::{ServiceError, assist, ledger, party_ledgers};
use support::*;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

fn list(
    ctx: &mandi_services::Context,
    side: LedgerSide,
    text: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<PartyLedgersList, ServiceError> {
    party_ledgers::list(
        ctx,
        &PartyLedgersListInput {
            side,
            text: text.map(str::to_owned),
            offset,
            limit,
        },
    )
}

fn statement(
    ctx: &mandi_services::Context,
    account_id: i64,
    from: Option<&str>,
    to: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<PartyLedgerStatement, ServiceError> {
    party_ledgers::statement(
        ctx,
        &PartyLedgerStatementInput {
            account_id,
            from: from.map(str::to_owned),
            to: to.map(str::to_owned),
            offset,
            limit,
        },
    )
}

/// A signed balance, a debit balance positive.
fn signed(paisa: i64, side: BalanceSide) -> i64 {
    match side {
        BalanceSide::Dr => paisa,
        BalanceSide::Cr => -paisa,
        _ => 0,
    }
}

fn clean_refusal(e: &ServiceError) -> bool {
    matches!(e, ServiceError::Rejected(_))
}

#[test]
fn hostile_filters_and_paging() {
    let w = world();
    // Some parties: the worked day's buyers and sellers, a چکتا sale, a few more buyers.
    save_ok(&w, &event1(&w));
    let mut s = sale("s-1", w.chukta_khata, "10", "1200");
    s.chukta = person("شاہد", "Shahid", "0300-0000000");
    save_ok(
        &w,
        &draft(vec![with_sales(
            bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
            vec![s],
        )]),
    );
    for n in 0..12 {
        add_account(
            &w.env,
            &w.owner,
            AccountType::Buyer,
            &format!(
                "خریدار {}",
                ["ا", "ب", "پ", "ت", "ٹ", "ث", "ج", "چ", "ح", "خ", "د", "ڈ"][n]
            ),
            &format!("Customer {n}"),
            true,
            "",
        );
    }
    let ctx = &*w.env.ctx;
    let all_buyers = list(ctx, LedgerSide::Buyer, None, 0, 100).unwrap();
    assert_eq!(
        all_buyers.count, 14,
        "2 buyers + 12; naqdi, 0030 and the people are not in it"
    );
    let total = (all_buyers.total_dr_paisa, all_buyers.total_cr_paisa);

    // Texts: each must be answered cleanly; a text that is a nonsense matches nothing.
    let nonsense = [
        "zzzz",
        "!!!",
        "%",
        "_",
        "\\",
        "'",
        "\"",
        "' OR 1=1 --",
        "'; DROP TABLE accounts; --",
        "%' OR '1'='1",
        "<script>",
        "..\\..\\",
        "\u{0}",
        "a\u{0}b",
        "\u{202e}",
        "\u{200b}",
        "\u{200b}\u{200b}",
        "😀",
        "9999999999999",
    ];
    for text in nonsense {
        let what = format!("text {text:?}");
        match list(ctx, LedgerSide::Buyer, Some(text), 0, 100) {
            Ok(l) => {
                assert!(l.rows.len() <= 100, "{what}");
                // A text that names no party must not list everybody.
                assert!(
                    l.count < all_buyers.count
                        || text.trim().is_empty()
                        || ["\u{200b}", "\u{200b}\u{200b}", "\u{202e}", "\u{0}"].contains(&text),
                    "{what}: matched all {} buyers",
                    l.count
                );
                if l.rows.is_empty() {
                    assert_eq!(l.count, 0, "{what}");
                    assert_eq!(
                        (l.total_dr_paisa.as_paisa(), l.total_cr_paisa.as_paisa()),
                        (0, 0),
                        "{what}: totals of nothing"
                    );
                }
            }
            Err(e) => assert!(clean_refusal(&e), "{what}: {e:?}"),
        }
    }
    // The nonsense that must give nothing at all.
    for text in [
        "zzzz",
        "!!!",
        "%",
        "_",
        "\\",
        "' OR 1=1 --",
        "<script>",
        "9999999999999",
        "😀",
    ] {
        let l = list(ctx, LedgerSide::Buyer, Some(text), 0, 100).unwrap();
        assert_eq!(l.count, 0, "text {text:?} matched {} buyers", l.count);
    }
    // A very long text.
    let long = "ا".repeat(100_000);
    let started = Instant::now();
    match list(ctx, LedgerSide::Buyer, Some(&long), 0, 100) {
        Ok(l) => assert_eq!(l.count, 0),
        Err(e) => assert!(clean_refusal(&e), "{e:?}"),
    }
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "a 100,000 letter text took {:?}",
        started.elapsed()
    );
    // Tables still there, books still balance.
    assert_eq!(tools::trial_difference(&w), 0);

    // Offsets and limits.
    for (offset, limit) in [
        (0, 0),
        (0, 1),
        (0, 3),
        (0, 99),
        (0, 100),
        (0, 101),
        (0, 1_000_000),
        (0, u32::MAX),
        (13, 100),
        (14, 100),
        (15, 100),
        (1_000_000, 100),
        (u32::MAX, 100),
        (u32::MAX, u32::MAX),
    ] {
        let l = list(ctx, LedgerSide::Buyer, None, offset, limit)
            .unwrap_or_else(|e| panic!("offset {offset} limit {limit}: {e:?}"));
        assert!(
            l.rows.len() <= 100,
            "offset {offset} limit {limit}: {} rows",
            l.rows.len()
        );
        assert_eq!(l.count, 14, "offset {offset} limit {limit}");
        assert_eq!(
            (l.total_dr_paisa, l.total_cr_paisa),
            total,
            "offset {offset} limit {limit}: totals follow the page"
        );
        let want = 14_usize.saturating_sub(offset as usize).min(if limit == 0 {
            25
        } else {
            limit.min(100) as usize
        });
        // A limit of 0 gives the default page; the default is at most 100.
        if limit != 0 {
            assert_eq!(l.rows.len(), want, "offset {offset} limit {limit}");
        }
    }
    // Pages of 3 put together are the whole list, in order, with no row twice.
    let whole = list(ctx, LedgerSide::Buyer, None, 0, 100).unwrap().rows;
    let mut joined = Vec::new();
    let mut offset = 0;
    loop {
        let page = list(ctx, LedgerSide::Buyer, None, offset, 3).unwrap();
        if page.rows.is_empty() {
            break;
        }
        offset += u32::try_from(page.rows.len()).unwrap();
        joined.extend(page.rows);
        assert!(joined.len() <= 100);
    }
    assert_eq!(joined, whole);
    // The sellers list; and a seller is not in the buyers' list and the reverse.
    let sellers = list(ctx, LedgerSide::Seller, None, 0, 100).unwrap();
    assert!(sellers.rows.iter().all(|r| r.account_id != w.akram));
    assert!(whole.iter().all(|r| r.account_id != w.rahim
        && r.account_id != w.chukta_khata
        && r.account_id != w.naqdi));

    // Statements: wrong accounts are refused cleanly with nothing read wrongly.
    for id in [
        0_i64,
        -1,
        i64::MIN,
        i64::MAX,
        987_654_321,
        w.chukta_khata,
        w.naqdi,
        id_of(&w.env, "01010001"),
        id_of(&w.env, "04010001"),
        id_of(&w.env, "0031"),
    ] {
        let e = statement(ctx, id, None, None, 0, 100).expect_err(&format!("account {id}"));
        assert!(clean_refusal(&e), "account {id}: {e:?}");
    }
    // Dates.
    let ok = statement(ctx, w.akram, None, None, 0, 100).unwrap();
    assert_eq!(ok.count, 1);
    for (from, to) in [
        (Some("31-02-2026"), None),
        (Some("00-00-0000"), None),
        (Some("abc"), None),
        (Some("' OR 1=1"), None),
        (Some("10-10-2026"), Some("09-10-2026")),
        (None, Some("99-99-9999")),
        (Some("\u{0}"), None),
        (Some("10-10-2026\u{0}"), None),
        (Some("10-10-2027"), None),
        (None, Some("10-10-2099")),
    ] {
        match statement(ctx, w.akram, from, to, 0, 100) {
            Ok(s) => assert!(s.count <= 1, "{from:?} {to:?}"),
            Err(e) => assert!(clean_refusal(&e), "{from:?} {to:?}: {e:?}"),
        }
    }
    // The valid shortcuts and ranges work, and a range with no entry is empty with the same opening and
    // closing (a day before everything: 0; a day after: the balance).
    let before = statement(ctx, w.akram, Some("01-01-2000"), Some("01-01-2001"), 0, 100).unwrap();
    assert_eq!((before.count, before.closing_paisa.as_paisa()), (0, 0));
    let same = statement(ctx, w.akram, Some("10-10-2026"), Some("10-10-2026"), 0, 100).unwrap();
    assert_eq!(same.count, 1);
    assert_eq!(same.closing_paisa, ok.closing_paisa);
    for (offset, limit) in [(0, 0), (0, 1), (5, 100), (u32::MAX, 100), (0, u32::MAX)] {
        let s = statement(ctx, w.akram, None, None, offset, limit).unwrap();
        assert!(s.rows.len() <= 100);
        assert_eq!(s.count, 1);
        assert_eq!(s.charged_total_paisa, ok.charged_total_paisa);
        assert_eq!(s.closing_paisa, ok.closing_paisa);
    }
    assert_chain_ok(&w.dir);
}

/// One statement walked page by page: the rows add up and chain, the totals are the walk's.
fn walk(
    ctx: &mandi_services::Context,
    account_id: i64,
    kind: LedgerSide,
    page: u32,
) -> (PartyLedgerStatement, Vec<PartyLedgerDay>) {
    let first = statement(ctx, account_id, None, None, 0, page).unwrap();
    let mut rows: Vec<PartyLedgerDay> = Vec::new();
    let mut offset = 0;
    loop {
        let s = statement(ctx, account_id, None, None, offset, page).unwrap();
        assert_eq!(s.count, first.count, "the count moves with the page");
        assert_eq!(
            s.charged_total_paisa, first.charged_total_paisa,
            "charged total moves with the page"
        );
        assert_eq!(
            s.paid_total_paisa, first.paid_total_paisa,
            "paid total moves with the page"
        );
        assert_eq!(s.closing_paisa, first.closing_paisa);
        if s.rows.is_empty() {
            break;
        }
        offset += u32::try_from(s.rows.len()).unwrap();
        rows.extend(s.rows);
    }
    assert_eq!(
        rows.len() as u32,
        first.count,
        "the walk found another number of dates than the count"
    );
    let (mut charged, mut paid) = (0_i64, 0_i64);
    for (i, row) in rows.iter().enumerate() {
        let (prev, bal) = (
            signed(row.previous_paisa.as_paisa(), row.previous_side),
            signed(row.balance_paisa.as_paisa(), row.balance_side),
        );
        let (c, p) = (row.charged_paisa.as_paisa(), row.paid_paisa.as_paisa());
        // A buyer's charged is a debit; a seller's charged is a credit.
        let delta = match kind {
            LedgerSide::Buyer => c - p,
            LedgerSide::Seller => p - c,
        };
        assert_eq!(prev + delta, bal, "row {i} ({}) does not add up", row.date);
        if let Some(older) = rows.get(i + 1) {
            assert_eq!(
                signed(older.balance_paisa.as_paisa(), older.balance_side),
                prev,
                "row {i} does not start where the older one ended"
            );
            assert!(older.date < row.date, "dates are not newest first");
        }
        charged += c;
        paid += p;
    }
    assert_eq!(
        charged,
        first.charged_total_paisa.as_paisa(),
        "charged total is not the walk's"
    );
    assert_eq!(
        paid,
        first.paid_total_paisa.as_paisa(),
        "paid total is not the walk's"
    );
    let delta = match kind {
        LedgerSide::Buyer => charged - paid,
        LedgerSide::Seller => paid - charged,
    };
    assert_eq!(
        signed(first.opening_paisa.as_paisa(), first.opening_side) + delta,
        signed(first.closing_paisa.as_paisa(), first.closing_side),
        "opening + the days is not the closing"
    );
    (first, rows)
}

fn assert_parties(w: &World, buyers: &[i64], sellers: &[i64], what: &str) {
    let ctx = &*w.env.ctx;
    for (ids, kind) in [(buyers, LedgerSide::Buyer), (sellers, LedgerSide::Seller)] {
        let everyone = list(ctx, kind, None, 0, 100).unwrap();
        assert_eq!(
            everyone.count as usize,
            ids.len(),
            "{what}: the list of {kind:?}"
        );
        let (mut dr, mut cr) = (0_i64, 0_i64);
        for row in &everyone.rows {
            match row.side {
                BalanceSide::Dr => dr += row.balance_paisa.as_paisa(),
                BalanceSide::Cr => cr += row.balance_paisa.as_paisa(),
                _ => {}
            }
        }
        assert_eq!(
            dr,
            everyone.total_dr_paisa.as_paisa(),
            "{what}: footer debit"
        );
        assert_eq!(
            cr,
            everyone.total_cr_paisa.as_paisa(),
            "{what}: footer credit"
        );
        for &id in ids {
            let (first, _rows) = walk(ctx, id, kind, 3);
            let b = ledger::account_balance(ctx, id).unwrap();
            assert_eq!(
                (first.closing_paisa, first.closing_side),
                (b.amount_paisa, b.side),
                "{what}: statement closing of {id} is not his balance"
            );
            let card = assist::party_card(ctx, id).unwrap();
            assert_eq!(
                (card.balance.amount_paisa, card.balance.side),
                (b.amount_paisa, b.side),
                "{what}: card of {id}"
            );
            let row = everyone
                .rows
                .iter()
                .find(|r| r.account_id == id)
                .unwrap_or_else(|| panic!("{what}: {id} not in the list"));
            assert_eq!(
                (row.balance_paisa, row.side),
                (b.amount_paisa, b.side),
                "{what}: list row of {id}"
            );
        }
    }
}

#[test]
fn after_random_sales_edits_cancels_and_receipts_every_party_adds_up() {
    let w = world_with(0, true);
    let tariq = add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "طارق محمود",
        "Tariq Mehmood",
        true,
        "",
    );
    let buyers = [w.akram, w.arshad, tariq];
    let sellers = [w.rahim, w.karim];
    let mut rng = Rng(0xD8_0066);
    let mut live: Vec<(i64, i64)> = Vec::new();
    let mut counts = [0_u32; 4]; // saved, edited, cancelled, receipts
    for step in 0..60 {
        let what = format!("step {step}");
        let kind = rng.below(10);
        if kind < 5 || live.is_empty() {
            let mut s_lines = Vec::new();
            let n = 1 + rng.below(4) as usize;
            let qtys: Vec<i64> = (0..n).map(|_| 1 + rng.below(15) as i64).collect();
            let received: i64 = qtys.iter().sum();
            for (i, q) in qtys.iter().enumerate() {
                let rate = 100 + rng.below(1_000);
                let mut s = sale(
                    &format!("s-{i}"),
                    buyers[rng.below(3) as usize],
                    &q.to_string(),
                    &rate.to_string(),
                );
                let total = (q * rate as i64 * 106 / 100 + q * 8 - 2).max(1);
                s.paid = match rng.below(4) {
                    0 => None,
                    1 => Some(total.to_string()),
                    _ => Some(rng.below(total as u64 + 1).to_string()),
                };
                s_lines.push(s);
            }
            let seller = sellers[rng.below(2) as usize];
            let mut lot = with_sales(
                bare_lot(&w, "lot-1", seller, w.banana, &received.to_string()),
                s_lines,
            );
            if rng.below(2) == 0 {
                lot.cash_paid = Some(rng.below(500).to_string());
            }
            if let Ok(saved) = save(&w, &draft(vec![lot])) {
                live.push((saved.bikri_id, saved.entry_id));
                counts[0] += 1;
            }
        } else if kind < 7 {
            let index = rng.below(live.len() as u64) as usize;
            let (bikri_id, entry_id) = live[index];
            let opened = tools::open(&w, bikri_id, entry_id);
            let mut d = opened.draft.clone();
            let line = rng.below(d.lots[0].sales.len() as u64) as usize;
            // Change the buyer, or the paid amount, or the rate.
            match rng.below(3) {
                0 => d.lots[0].sales[line].buyer_account_id = Some(buyers[rng.below(3) as usize]),
                1 => d.lots[0].sales[line].paid = None,
                _ => d.lots[0].sales[line].seller_rate = (100 + rng.below(1_000)).to_string(),
            }
            if let Ok(done) = tools::try_edit(&w, &opened, &d, true) {
                live[index] = (bikri_id, done.entry_id);
                counts[1] += 1;
            }
        } else if kind < 8 && live.len() > 1 {
            let index = rng.below(live.len() as u64) as usize;
            let (bikri_id, _) = live.remove(index);
            tools::cancel(&w, bikri_id, true).unwrap();
            counts[2] += 1;
        } else {
            let b = buyers[rng.below(3) as usize];
            let (open, _) = tools::open_bills(&w, b);
            if open > 0 {
                tools::receipt(&w, b, &(1 + rng.below(open as u64)).to_string());
                counts[3] += 1;
            }
        }
        if step % 6 == 5 || step == 59 {
            assert_parties(&w, &buyers, &sellers, &what);
        }
    }
    eprintln!(
        "random run: saved {} / edited {} / cancelled {} / receipts {}",
        counts[0], counts[1], counts[2], counts[3]
    );
    assert!(
        counts[0] >= 12 && counts[1] >= 3 && counts[2] >= 2 && counts[3] >= 2,
        "too little happened: {counts:?}"
    );
    assert_eq!(tools::trial_difference(&w), 0);
}

// ---- The NFR-SCL-1 volume --------------------------------------------------------------------------

fn volume_on() -> bool {
    if std::env::var("MANDI_PERF_VOLUME").is_err() {
        println!(
            "MANDI_PERF_VOLUME is not set: the volume checks of the party ledgers are skipped"
        );
        return false;
    }
    true
}

fn p95(mut v: Vec<Duration>) -> Duration {
    v.sort();
    let i = ((v.len() as f64) * 0.95).ceil() as usize;
    v[i.saturating_sub(1).min(v.len() - 1)]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1_000.0
}

#[test]
fn at_the_nfr_scl_1_volume_the_statements_add_up_and_agree_with_the_balance_the_card_and_the_list()
{
    if !volume_on() {
        return;
    }
    let v = volume::ensure_volume();
    volume::describe(&v);
    let (env, _actor) = volume::open(&v.dir);
    let ctx = &*env.ctx;
    for (id, kind, name) in [
        (v.busy_buyer, LedgerSide::Buyer, "busiest buyer"),
        (v.busy_seller, LedgerSide::Seller, "busiest seller"),
        (v.median_buyer, LedgerSide::Buyer, "ordinary buyer"),
    ] {
        let started = Instant::now();
        let (first, rows) = walk(ctx, id, kind, 100);
        println!(
            "VOLUME {name}: {} dates walked in {:?}",
            rows.len(),
            started.elapsed()
        );
        let b = ledger::account_balance(ctx, id).unwrap();
        assert_eq!(
            (first.closing_paisa, first.closing_side),
            (b.amount_paisa, b.side),
            "{name}"
        );
        let card = assist::party_card(ctx, id).unwrap();
        assert_eq!(
            (card.balance.amount_paisa, card.balance.side),
            (b.amount_paisa, b.side),
            "{name}: card"
        );
        let code: String = ctx
            .db()
            .read(|c| {
                c.query_row("SELECT code FROM accounts WHERE id = ?1", [id], |r| {
                    r.get(0)
                })
                .map_err(mandi_services::ServiceError::from)
            })
            .unwrap();
        let listed = list(ctx, kind, Some(&code), 0, 100).unwrap();
        let row = listed
            .rows
            .iter()
            .find(|r| r.account_id == id)
            .unwrap_or_else(|| panic!("{name} not in the list"));
        assert_eq!(
            (row.balance_paisa, row.side),
            (b.amount_paisa, b.side),
            "{name}: list row"
        );
    }
    // The list's footer over the whole control is the sum of every balance, and not of the page.
    let page = list(ctx, LedgerSide::Buyer, None, 0, 100).unwrap();
    let page2 = list(ctx, LedgerSide::Buyer, None, 100, 100).unwrap();
    assert_eq!(
        (page.count, page.total_dr_paisa, page.total_cr_paisa),
        (page2.count, page2.total_dr_paisa, page2.total_cr_paisa)
    );
    assert!(page.count > 8_000);
}

#[test]
fn at_the_nfr_scl_1_volume_the_95th_percentile_against_nfr_spd_24() {
    if !volume_on() {
        return;
    }
    let v = volume::ensure_volume();
    let (env, _actor) = volume::open(&v.dir);
    let ctx = &*env.ctx;
    let runs = 20;
    let mut rows: Vec<(&str, Duration, Duration)> = Vec::new();
    let mut measure = |name: &'static str, target: Duration, f: &dyn Fn()| {
        f(); // warm-up
        let times: Vec<Duration> = (0..runs)
            .map(|_| {
                let s = Instant::now();
                f();
                s.elapsed()
            })
            .collect();
        rows.push((name, p95(times), target));
    };
    let first = Duration::from_millis(1_000);
    let next = Duration::from_millis(300);
    measure("list buyers, first page of 100 with totals", first, &|| {
        list(ctx, LedgerSide::Buyer, None, 0, 100).unwrap();
    });
    measure("list buyers, next page", next, &|| {
        list(ctx, LedgerSide::Buyer, None, 100, 100).unwrap();
    });
    measure("list buyers, a text filter", first, &|| {
        list(ctx, LedgerSide::Buyer, Some("ارشد"), 0, 100).unwrap();
    });
    measure("list sellers, first page", first, &|| {
        list(ctx, LedgerSide::Seller, None, 0, 100).unwrap();
    });
    measure("statement of the busiest buyer, first page", first, &|| {
        statement(ctx, v.busy_buyer, None, None, 0, 100).unwrap();
    });
    measure(
        "statement of the busiest buyer, a later page (offset 3000)",
        next,
        &|| {
            statement(ctx, v.busy_buyer, None, None, 3_000, 100).unwrap();
        },
    );
    measure(
        "statement of the busiest seller, first page",
        first,
        &|| {
            statement(ctx, v.busy_seller, None, None, 0, 100).unwrap();
        },
    );
    measure("statement of an ordinary buyer, first page", first, &|| {
        statement(ctx, v.median_buyer, None, None, 0, 100).unwrap();
    });
    let mut over = Vec::new();
    for (name, measured, target) in &rows {
        println!(
            "PERF party ledgers: {name}: p95 {:.1} ms vs {:.0} ms ({})",
            ms(*measured),
            ms(*target),
            if measured <= target { "OK" } else { "OVER" }
        );
        if measured > target {
            over.push(*name);
        }
    }
    if !cfg!(debug_assertions) {
        assert!(over.is_empty(), "over the NFR-SPD-24 target: {over:?}");
    }
}

#[allow(dead_code)]
fn unused(_: ErrorCode) {}

/// CPU seconds this process has used, read from `ps` (the workspace forbids `unsafe`, also in tests).
fn cpu_seconds() -> f64 {
    let out = std::process::Command::new("ps")
        .args(["-o", "cputime=", "-p", &std::process::id().to_string()])
        .output()
        .expect("ps");
    let text = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let mut seconds = 0.0_f64;
    for part in text.split(':') {
        seconds = seconds * 60.0 + part.parse::<f64>().unwrap_or(0.0);
    }
    seconds
}

fn cpu_ms_of(repeat: usize, f: &dyn Fn()) -> f64 {
    let start = cpu_seconds();
    for _ in 0..repeat {
        f();
    }
    (cpu_seconds() - start) * 1_000.0 / repeat as f64
}

/// The same pages in CPU time, scaled to the reference machine of NFR 0.2 with the ratio of
/// scripts/perf/README.md Section 2 (3,752 / 2,268): the wall clock of a busy machine lies.
#[test]
fn at_the_nfr_scl_1_volume_the_cpu_time_of_the_pages_on_the_reference_machine() {
    if !volume_on() {
        return;
    }
    let ratio = 3_752.0 / 2_268.0;
    let v = volume::ensure_volume();
    let (env, _actor) = volume::open(&v.dir);
    let ctx = &*env.ctx;
    let mut over = Vec::new();
    let mut check = |name: &str, target_ms: f64, f: &dyn Fn()| {
        f();
        let cpu = (0..3).map(|_| cpu_ms_of(10, f)).fold(f64::MAX, f64::min);
        println!(
            "CPU party ledgers: {name}: {cpu:.1} ms on this Mac, about {:.0} ms on the reference machine vs {target_ms:.0} ms",
            cpu * ratio
        );
        if cpu * ratio > target_ms {
            over.push(name.to_owned());
        }
    };
    check("list buyers, first page", 1_000.0, &|| {
        list(ctx, LedgerSide::Buyer, None, 0, 100).unwrap();
    });
    check("list buyers, next page", 300.0, &|| {
        list(ctx, LedgerSide::Buyer, None, 100, 100).unwrap();
    });
    check(
        "statement of the busiest buyer, first page",
        1_000.0,
        &|| {
            statement(ctx, v.busy_buyer, None, None, 0, 100).unwrap();
        },
    );
    check(
        "statement of the busiest buyer, a later page",
        300.0,
        &|| {
            statement(ctx, v.busy_buyer, None, None, 3_000, 100).unwrap();
        },
    );
    check(
        "statement of the busiest seller, first page",
        1_000.0,
        &|| {
            statement(ctx, v.busy_seller, None, None, 0, 100).unwrap();
        },
    );
    if !cfg!(debug_assertions) {
        assert!(
            over.is_empty(),
            "over the NFR-SPD-24 target on the reference machine: {over:?}"
        );
    }
}
