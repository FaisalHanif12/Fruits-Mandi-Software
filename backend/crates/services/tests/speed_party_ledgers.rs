//! NFR-SPD-24 (PRD FR-19.11, FR-12.43, Code Standards B31), D8 robustness MINOR 2: a later page of
//! the party ledgers statement of the busiest buyer cost 256 of its 300 ms at the NFR-SCL-1 volume.
//!
//! The busiest buyer of NFR-SCL-1 has about 55,000 journal lines over 10 years. This test gives one
//! buyer that many lines by plain SQL (every entry is first a draft, then one `UPDATE` posts it, so
//! the balance trigger checks it), with about one entry in a hundred reversed on its own day (the
//! same-day pairs the statement leaves out). It then reads a statement three ways:
//!
//! - the first page, a later page (offset 3,000) and the last page, each with its totals;
//! - the statement walked page by page: every row starts where the older one ended, and the closing
//!   balance is the sum of every line of the buyer (the figure of his party card, PRD FR-6.1);
//! - the p95 time of the first and of the later page, printed as `PERF party ledgers statement`.
//!
//! The test also keeps the two queries of the statement as they were before this fix, as a reference:
//! the new one-reading query must give the same rows and the same totals on every case (including the
//! same-day pairs), and the p95 of both is printed side by side (`PERF old` and `PERF new`).
//!
//! The p95 is asserted only in a release build (`cargo test --release -p mandi-services --test
//! speed_party_ledgers -- --nocapture`), where SQLite is built with optimisation like the app;
//! unoptimised SQLite is several times slower, so a debug run prints the numbers and checks the
//! figures only. `MANDI_SPEED_REPORT_ONLY=1` prints without failing.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[path = "../benches/support/mod.rs"]
mod volume;

use std::time::{Duration, Instant};

use mandi_services::core::party_ledgers::{PartyLedgerStatement, PartyLedgerStatementInput};
use mandi_services::db::rusqlite::{Connection, params};
use mandi_services::party_ledgers;

/// Journal lines of the busiest buyer of NFR-SCL-1 (the NFR-SCL-1 file builder says about 55,000).
const LINES: u64 = 55_000;
/// The first day of the 10 years.
const FIRST_DAY: &str = "2016-10-11";
/// Days in the 10 years, 11-10-2016 to 10-10-2026.
const DAYS: i64 = 3_652;
/// The page of the statement.
const PAGE: u32 = 100;
/// A later page begins here (the robustness test of NFR-SPD-24 reads the same one).
const LATER_OFFSET: u32 = 3_000;
/// Each next page (NFR-SPD-24).
const NEXT_PAGE: Duration = Duration::from_millis(300);
/// The first page with its totals (NFR-SPD-24).
const FIRST_PAGE: Duration = Duration::from_millis(1_000);
/// Timed runs of each page.
const RUNS: usize = 15;
/// A 64-character hash. The speed test never verifies the chain.
const FAKE_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// One line of the report (the test prints, as `speed_pages` does).
fn note(text: &str) {
    println!("{text}");
}

fn id_of(conn: &Connection, code: &str) -> i64 {
    conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
        r.get(0)
    })
    .unwrap()
}

/// The date `n` days after the first day.
fn day(n: i64) -> String {
    let first = chrono::NaiveDate::parse_from_str(FIRST_DAY, "%Y-%m-%d").unwrap();
    (first + chrono::Duration::days(n))
        .format("%Y-%m-%d")
        .to_string()
}

/// One posted entry of two lines: the buyer and the cash account. Returns its row id.
fn post(
    conn: &Connection,
    date: &str,
    kind: &str,
    reverses: Option<i64>,
    lines: [(i64, i64, i64); 2],
) -> i64 {
    conn.execute(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, reverses_entry_id, created_by, updated_by)
         VALUES (?1, ?2, 'voucher', 1, ?3, 1, 1)",
        params![date, kind, reverses],
    )
    .unwrap();
    let id = conn.last_insert_rowid();
    let mut total = 0;
    for (n, (account, dr, cr)) in lines.iter().enumerate() {
        conn.execute(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, created_by, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 1)",
            params![id, i64::try_from(n).unwrap() + 1, account, dr, cr],
        )
        .unwrap();
        total += dr;
    }
    conn.execute(
        "UPDATE journal_entries SET status = 'posted', total_paisa = ?1, chain_hash = ?2 WHERE id = ?3",
        params![total, FAKE_HASH, id],
    )
    .unwrap();
    id
}

/// Gives the buyer `LINES` posted lines: sales and receipts in a fixed pattern, and every hundredth
/// entry reversed on its own day. Returns his signed balance (debits less credits) in paisa and how
/// many dates have a live line.
fn busy_buyer(w: &support::World) -> (i64, i64) {
    let conn = common::raw(&w.dir);
    let cash = id_of(&conn, "01010001");
    conn.execute_batch("BEGIN").unwrap();
    let (mut lines, mut n, mut net) = (0_u64, 0_i64, 0_i64);
    let mut live_days = std::collections::BTreeSet::new();
    while lines < LINES {
        let date = day(n * DAYS / i64::try_from(LINES).unwrap());
        // A sale (debit), or on every fourth entry a receipt (credit); amounts are whole rupees.
        let amount = 100 * (1 + n % 97);
        let sale = n % 4 != 3;
        let (dr, cr) = if sale { (amount, 0) } else { (0, amount) };
        let original = post(
            &conn,
            &date,
            if sale { "general" } else { "receipt" },
            None,
            [(w.akram, dr, cr), (cash, cr, dr)],
        );
        lines += 1;
        if n % 100 == 50 {
            // The same-day reversal: the two halves cancel on his account and leave no row.
            post(
                &conn,
                &date,
                "reversal",
                Some(original),
                [(w.akram, cr, dr), (cash, dr, cr)],
            );
            lines += 1;
        } else {
            net += dr - cr;
            live_days.insert(date);
        }
        n += 1;
    }
    conn.execute_batch("COMMIT").unwrap();
    (net, i64::try_from(live_days.len()).unwrap())
}

fn statement(
    w: &support::World,
    offset: u32,
    from: Option<&str>,
    to: Option<&str>,
) -> PartyLedgerStatement {
    party_ledgers::statement(
        &w.env.ctx,
        &PartyLedgerStatementInput {
            account_id: w.akram,
            from: from.map(str::to_owned),
            to: to.map(str::to_owned),
            offset,
            limit: PAGE,
        },
    )
    .unwrap()
}

/// The same-day test as it was (and still is, once for each line, in `party_ledgers.rs` of `mandi-db`).
const SAME_DAY_PAIR: &str = "(e.reverses_entry_id IS NOT NULL AND EXISTS (
        SELECT 1 FROM journal_entries o
        WHERE o.id = e.reverses_entry_id AND o.entry_date = e.entry_date))
    OR EXISTS (SELECT 1 FROM journal_entries r
        WHERE r.reverses_entry_id = e.id AND r.status = 'posted' AND r.entry_date = e.entry_date)";

/// The statement as it was before D8 robustness MINOR 2: the same-day test three times for each line
/// in the CTE, and the CTE read twice, once for the page and once for the totals. The reference
/// the new query is held equal to.
fn old_statement(
    conn: &Connection,
    account: i64,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    offset: i64,
) -> (Vec<mandi_services::db::party_ledgers::DayRow>, [i64; 7]) {
    let cte = format!(
        "WITH days AS (
           SELECT e.entry_date AS d,
                  SUM(l.dr_paisa) AS all_dr, SUM(l.cr_paisa) AS all_cr,
                  SUM(CASE WHEN {SAME_DAY_PAIR} THEN 0 ELSE l.dr_paisa END) AS live_dr,
                  SUM(CASE WHEN {SAME_DAY_PAIR} THEN 0 ELSE l.cr_paisa END) AS live_cr,
                  SUM(CASE WHEN {SAME_DAY_PAIR} THEN 0 ELSE 1 END) AS live_n
           FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
           WHERE l.account_id = ?1 AND e.status = 'posted'
           GROUP BY e.entry_date)"
    );
    let page = format!(
        "{cte}
         SELECT d, all_dr, all_cr, live_dr, live_cr, prev_dr, prev_cr FROM (
           SELECT d, all_dr, all_cr, live_dr, live_cr, live_n,
                  COALESCE(SUM(all_dr) OVER w, 0) AS prev_dr,
                  COALESCE(SUM(all_cr) OVER w, 0) AS prev_cr
           FROM days
           WINDOW w AS (ORDER BY d ROWS BETWEEN UNBOUNDED PRECEDING AND 1 PRECEDING))
         WHERE live_n > 0 AND (?2 IS NULL OR d >= ?2) AND (?3 IS NULL OR d <= ?3)
         ORDER BY d DESC LIMIT ?4 OFFSET ?5"
    );
    let totals = format!(
        "{cte}
         SELECT COUNT(CASE WHEN live_n > 0 AND (?2 IS NULL OR d >= ?2) AND (?3 IS NULL OR d <= ?3)
                           THEN 1 END),
                COALESCE(SUM(CASE WHEN live_n > 0 AND (?2 IS NULL OR d >= ?2) AND (?3 IS NULL OR d <= ?3)
                                  THEN live_dr END), 0),
                COALESCE(SUM(CASE WHEN live_n > 0 AND (?2 IS NULL OR d >= ?2) AND (?3 IS NULL OR d <= ?3)
                                  THEN live_cr END), 0),
                COALESCE(SUM(CASE WHEN ?2 IS NOT NULL AND d < ?2 THEN all_dr END), 0),
                COALESCE(SUM(CASE WHEN ?2 IS NOT NULL AND d < ?2 THEN all_cr END), 0),
                COALESCE(SUM(CASE WHEN ?3 IS NULL OR d <= ?3 THEN all_dr END), 0),
                COALESCE(SUM(CASE WHEN ?3 IS NULL OR d <= ?3 THEN all_cr END), 0)
         FROM days"
    );
    let days = conn
        .prepare_cached(&page)
        .unwrap()
        .query_map(params![account, from, to, limit, offset], |r| {
            Ok(mandi_services::db::party_ledgers::DayRow {
                date: r.get(0)?,
                all_dr: r.get(1)?,
                all_cr: r.get(2)?,
                live_dr: r.get(3)?,
                live_cr: r.get(4)?,
                prev_dr: r.get(5)?,
                prev_cr: r.get(6)?,
            })
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let totals = conn
        .prepare_cached(&totals)
        .unwrap()
        .query_row(params![account, from, to], |r| {
            Ok([
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ])
        })
        .unwrap();
    (days, totals)
}

/// The new one-reading query, as the same pair.
fn new_statement(
    conn: &Connection,
    account: i64,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    offset: i64,
) -> (Vec<mandi_services::db::party_ledgers::DayRow>, [i64; 7]) {
    let (days, t) =
        mandi_services::db::party_ledgers::statement_page(conn, account, from, to, limit, offset)
            .unwrap();
    (
        days,
        [
            t.count,
            t.live_dr,
            t.live_cr,
            t.opening_dr,
            t.opening_cr,
            t.closing_dr,
            t.closing_cr,
        ],
    )
}

fn p95(mut times: Vec<Duration>) -> Duration {
    times.sort();
    let rank = (times.len() * 95).div_ceil(100).max(1);
    times[rank - 1]
}

fn timed(run: &dyn Fn()) -> Duration {
    run(); // warm-up: the page cache and the prepared statement
    p95((0..RUNS)
        .map(|_| {
            let started = Instant::now();
            run();
            started.elapsed()
        })
        .collect())
}

fn ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

#[test]
fn a_later_page_of_the_busiest_buyers_statement_adds_up_and_is_fast() {
    let w = support::world();
    let started = Instant::now();
    let (net, live_days) = busy_buyer(&w);
    note(&format!(
        "data: {LINES} lines for one buyer, {live_days} dates with a live line, built in {:.1} s",
        started.elapsed().as_secs_f64()
    ));

    // The figures first. The closing balance is the sum of every line of the buyer, the figure of
    // his party card, and the walk of every page makes the count and the totals of the filter.
    let first = statement(&w, 0, None, None);
    assert_eq!(
        i64::from(first.count),
        live_days,
        "one row for each live date"
    );
    let signed = |paisa: i64, side: mandi_services::core::accounts::BalanceSide| match side {
        mandi_services::core::accounts::BalanceSide::Dr => paisa,
        mandi_services::core::accounts::BalanceSide::Cr => -paisa,
        mandi_services::core::accounts::BalanceSide::Zero => 0,
    };
    assert_eq!(
        signed(first.closing_paisa.as_paisa(), first.closing_side)
            - signed(first.opening_paisa.as_paisa(), first.opening_side),
        net,
        "closing less opening is the net of the live lines (the pairs cancel)"
    );
    // The newest date is first; each row starts where the older one ended.
    let mut offset = 0;
    let mut newer_start: Option<(i64, String)> = None;
    let mut rows = 0_u32;
    loop {
        let page = statement(&w, offset, None, None);
        if page.rows.is_empty() {
            break;
        }
        for row in &page.rows {
            let balance = signed(row.balance_paisa.as_paisa(), row.balance_side);
            if let Some((newer_previous, newer_date)) = &newer_start {
                assert!(
                    row.date < *newer_date,
                    "newest first: {} then {newer_date}",
                    row.date
                );
                assert_eq!(
                    balance, *newer_previous,
                    "{}: ends where {newer_date} starts",
                    row.date
                );
            }
            newer_start = Some((
                signed(row.previous_paisa.as_paisa(), row.previous_side),
                row.date.clone(),
            ));
            rows += 1;
        }
        offset += PAGE;
    }
    assert_eq!(i64::from(rows), live_days);
    assert_eq!(
        newer_start.map(|(previous, _)| previous),
        Some(signed(first.opening_paisa.as_paisa(), first.opening_side)),
        "the oldest row starts at the opening balance"
    );

    // The new query is the old one, row for row and total for total, on every kind of page.
    let conn = common::raw(&w.dir);
    for (from, to, offset) in [
        (None, None, 0_i64),
        (None, None, i64::from(LATER_OFFSET)),
        (None, None, 3_600),
        (None, None, 9_999),
        (Some("2020-02-29"), None, 0),
        (None, Some("2021-06-30"), 100),
        (Some("2024-01-01"), Some("2025-12-31"), 0),
        (Some("2030-01-01"), None, 0),
    ] {
        assert_eq!(
            new_statement(&conn, w.akram, from, to, i64::from(PAGE), offset),
            old_statement(&conn, w.akram, from, to, i64::from(PAGE), offset),
            "{from:?} {to:?} offset {offset}"
        );
    }
    // The raw queries side by side: the same lines, read the old way and the new way.
    for (name, offset) in [
        ("first page", 0_i64),
        ("later page", i64::from(LATER_OFFSET)),
    ] {
        let old = timed(&|| {
            old_statement(&conn, w.akram, None, None, i64::from(PAGE), offset);
        });
        let new = timed(&|| {
            new_statement(&conn, w.akram, None, None, i64::from(PAGE), offset);
        });
        note(&format!(
            "PERF old {name}: p95 {:.1} ms; PERF new {name}: p95 {:.1} ms ({:.1} times faster)",
            ms(old),
            ms(new),
            ms(old) / ms(new)
        ));
    }

    // The speed.
    let first_page = timed(&|| {
        statement(&w, 0, None, None);
    });
    let later_page = timed(&|| {
        statement(&w, LATER_OFFSET, None, None);
    });
    let last_page = timed(&|| {
        statement(&w, u32::try_from(live_days).unwrap() - PAGE, None, None);
    });
    // A page of a date range reads the same lines (the opening balance is the lines before it).
    let ranged = timed(&|| {
        statement(&w, 0, Some("01-01-2024"), Some("31-12-2025"));
    });
    for (name, measured, budget) in [
        ("first page", first_page, FIRST_PAGE),
        ("later page (offset 3000)", later_page, NEXT_PAGE),
        ("last page", last_page, NEXT_PAGE),
        ("a date range, first page", ranged, FIRST_PAGE),
    ] {
        note(&format!(
            "PERF party ledgers statement: {name}: p95 {:.1} ms vs {:.0} ms ({})",
            ms(measured),
            ms(budget),
            if measured <= budget { "OK" } else { "OVER" }
        ));
        if !cfg!(debug_assertions) && std::env::var_os("MANDI_SPEED_REPORT_ONLY").is_none() {
            assert!(
                measured <= budget,
                "{name}: {:.1} ms over {:.0} ms",
                ms(measured),
                ms(budget)
            );
        }
    }
}

/// The NFR-SCL-1 file of `pnpm perf` (3.5 million lines, `MANDI_PERF_VOLUME=1`): the statement of the
/// busiest buyer, the busiest seller and an ordinary buyer, read the old way and the new way on the
/// same data in the same run, row for row, with the p95 of each printed side by side. The file is read
/// only. `MANDI_PERF_DIR` points at the file when it is not in `backend/target/perf/nfr_scl1`.
#[test]
fn at_the_nfr_scl_1_volume_the_new_statement_equals_the_old_and_is_faster() {
    if std::env::var("MANDI_PERF_VOLUME").map_or(true, |v| v != "1") {
        note("MANDI_PERF_VOLUME is not set: the volume check of the statement is skipped");
        return;
    }
    let v = volume::ensure_volume();
    let conn = Connection::open_with_flags(
        v.dir.join("data").join("mandi.db"),
        mandi_services::db::rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let mut rows: Vec<(String, Duration, Duration)> = Vec::new();
    for (who, account) in [
        ("busiest buyer", v.busy_buyer),
        ("busiest seller", v.busy_seller),
        ("ordinary buyer", v.median_buyer),
    ] {
        for (name, from, to, offset) in [
            ("first page", None, None, 0_i64),
            ("later page (offset 3000)", None, None, 3_000),
            ("a date range", Some("2023-01-01"), Some("2024-12-31"), 0),
        ] {
            let page = i64::from(PAGE);
            assert_eq!(
                new_statement(&conn, account, from, to, page, offset),
                old_statement(&conn, account, from, to, page, offset),
                "{who} {name}"
            );
            let old = timed(&|| {
                old_statement(&conn, account, from, to, page, offset);
            });
            let new = timed(&|| {
                new_statement(&conn, account, from, to, page, offset);
            });
            rows.push((format!("{who}, {name}"), old, new));
        }
    }
    for (name, old, new) in &rows {
        note(&format!(
            "PERF volume {name}: old p95 {:.1} ms, new p95 {:.1} ms ({:.1} times faster)",
            ms(*old),
            ms(*new),
            ms(*old) / ms(*new)
        ));
    }
}
