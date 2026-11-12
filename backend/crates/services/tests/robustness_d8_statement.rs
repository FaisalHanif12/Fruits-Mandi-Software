//! Plan 74 T4, speed of the party statement on 10 years of data (PRD FR-12.47; NFR-SPD-14, NFR-SPD-24;
//! plan 74 S1). Owner: `robustness`.
//!
//! Only with `MANDI_PERF_VOLUME=1`; it reads the NFR-SCL-1 file of `pnpm perf` (never rebuilt here when
//! the marker is there). For the busiest buyer (about 55,000 lines) and the busiest seller (about
//! 49,000 lines), p95 over 15 runs, in a release build:
//!
//! | # | What | Budget |
//! |---|---|---|
//! | 1 | open the statement page: the first 30-day window (today minus 29 to today) with its totals | 1 s |
//! | 2 | a window move: the 30 days before it | 300 ms |
//! | 3 | go to a date 5 years back (five different days around it) | 1 s |
//! | 4 | open one day: the R35 day read of the party for a day he has an entry | 300 ms |
//! | 5 | the 6-month statement print data: the whole statement of the period and the print rows | 2 s |
//!
//! It also checks that every answer is the right one (the window has 30 days of rows at most, the closing
//! of the first window is the balance of the party, the day read has rows, the print data has days). The
//! p95 is asserted only in a release build (`MANDI_PERF_VOLUME=1 cargo test --release -p mandi-services
//! --test robustness_d8_statement -- --nocapture`).
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[path = "../benches/support/mod.rs"]
mod volume;

use std::time::{Duration, Instant};

use mandi_reports::reports::{
    PartyFilter, ReportFilters, ReportId, ReportRunInput, StatementPrintDataInput,
    party_statement_print_data, run as run_report,
};
use mandi_services::core::auth::Language;
use mandi_services::core::party_ledgers::{
    PartyLedgerGoToInput, PartyLedgerStatement, PartyLedgerStatementInput,
};
use mandi_services::{Context, ledger, party_ledgers};

fn p95(mut v: Vec<Duration>) -> Duration {
    v.sort();
    let i = ((v.len() as f64) * 0.95).ceil() as usize;
    v[i.saturating_sub(1).min(v.len() - 1)]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1_000.0
}

/// `YYYY-MM-DD` as `DD-MM-YYYY`.
fn typed(iso: &str) -> String {
    let mut parts = iso.split('-');
    let (y, m, d) = (
        parts.next().unwrap_or(""),
        parts.next().unwrap_or(""),
        parts.next().unwrap_or(""),
    );
    format!("{d}-{m}-{y}")
}

fn window_of(ctx: &Context, account_id: i64, from: &str, to: &str) -> PartyLedgerStatement {
    party_ledgers::statement(
        ctx,
        &PartyLedgerStatementInput {
            account_id,
            from: Some(typed(from)),
            to: Some(typed(to)),
            offset: 0,
            limit: 100,
        },
    )
    .unwrap_or_else(|e| panic!("statement {from} to {to}: {e:?}"))
}

/// Times `f` 15 times after one warm-up, and returns the p95.
fn measure(f: &dyn Fn()) -> Duration {
    f();
    p95((0..15)
        .map(|_| {
            let started = Instant::now();
            f();
            started.elapsed()
        })
        .collect())
}

#[test]
fn the_party_statement_on_ten_years_against_plan_74_s1() {
    if std::env::var("MANDI_PERF_VOLUME").is_err() {
        println!(
            "MANDI_PERF_VOLUME is not set: the volume check of the party statement is skipped"
        );
        return;
    }
    let v = volume::ensure_volume();
    volume::describe(&v);
    let (env, _actor) = volume::open(&v.dir);
    let ctx = &*env.ctx;
    let today = volume::last_day();
    let iso = |back: i64| {
        (today - chrono::Duration::days(back))
            .format("%Y-%m-%d")
            .to_string()
    };
    let mut table: Vec<(String, Duration, Duration)> = Vec::new();
    for (account_id, who) in [
        (v.busy_buyer, "busiest buyer"),
        (v.busy_seller, "busiest seller"),
    ] {
        let first_from = iso(29);
        let first_to = iso(0);
        // The first window is right: at most 30 dates, and its closing is the balance of the party.
        let first = window_of(ctx, account_id, &first_from, &first_to);
        assert!(
            first.rows.len() <= 30,
            "{who}: {} rows in 30 days",
            first.rows.len()
        );
        let balance = ledger::account_balance(ctx, account_id).unwrap();
        assert_eq!(
            (first.closing_paisa, first.closing_side),
            (balance.amount_paisa, balance.side),
            "{who}: the closing of the newest window is his balance"
        );
        // 1. Open the page.
        table.push((
            format!("{who}: open, first 30-day window with totals"),
            measure(&|| {
                window_of(ctx, account_id, &first_from, &first_to);
            }),
            Duration::from_millis(1_000),
        ));
        // 2. A window move: the 30 days before.
        let (prev_from, prev_to) = (iso(59), iso(30));
        let moved = window_of(ctx, account_id, &prev_from, &prev_to);
        assert!(moved.rows.len() <= 30, "{who}: a moved window");
        table.push((
            format!("{who}: window move (previous 30 days)"),
            measure(&|| {
                window_of(ctx, account_id, &prev_from, &prev_to);
            }),
            Duration::from_millis(300),
        ));
        // 3. Go to a date 5 years back: five days around it, each run asks another one.
        let five_years = 365 * 5;
        let days: Vec<String> = (0..5).map(|n| typed(&iso(five_years + 7 * n))).collect();
        let at = std::cell::Cell::new(0_usize);
        let went = party_ledgers::go_to_date(
            ctx,
            &PartyLedgerGoToInput {
                account_id,
                date: days[0].clone(),
            },
        )
        .unwrap();
        assert!(went.statement.rows.len() <= 30, "{who}: go to date window");
        table.push((
            format!("{who}: go to a date 5 years back"),
            measure(&|| {
                let i = at.get();
                at.set(i + 1);
                party_ledgers::go_to_date(
                    ctx,
                    &PartyLedgerGoToInput {
                        account_id,
                        date: days[i % days.len()].clone(),
                    },
                )
                .unwrap();
            }),
            Duration::from_millis(1_000),
        ));
        // 4. Open one day: the R35 day read of this party, for days he has an entry (the newest ones of
        // the first window, and the ones of the window 5 years back).
        let mut entry_days: Vec<String> = first.rows.iter().map(|r| r.date.clone()).collect();
        entry_days.extend(went.statement.rows.iter().map(|r| r.date.clone()));
        entry_days.truncate(8);
        assert!(
            !entry_days.is_empty(),
            "{who}: no day with an entry to open"
        );
        let read_day = |day: &str| {
            let page = run_report(
                ctx.db(),
                &ReportRunInput {
                    id: ReportId::R35,
                    filters: ReportFilters {
                        as_of: typed(day),
                        party: Some(PartyFilter {
                            account_id,
                            person_key: None,
                        }),
                        ..ReportFilters::default()
                    },
                    offset: 0,
                    limit: 0,
                    sort: None,
                    group_by_month: false,
                },
            )
            .unwrap_or_else(|e| panic!("{who}: day {day}: {e:?}"));
            // A day with only a receipt has no goods table, but it has the lines of the slip.
            assert!(
                !page.rows.is_empty() || !page.summary.is_empty(),
                "{who}: the day slip of {day} is empty"
            );
        };
        for day in &entry_days {
            read_day(day);
        }
        let at = std::cell::Cell::new(0_usize);
        table.push((
            format!("{who}: open one day (R35 day read)"),
            measure(&|| {
                let i = at.get();
                at.set(i + 1);
                read_day(&entry_days[i % entry_days.len()]);
            }),
            Duration::from_millis(300),
        ));
        // 5. The 6-month statement print data (the whole statement of the period, then the print rows).
        let (six_from, six_to) = (typed(&iso(182)), typed(&iso(0)));
        let build = || {
            let whole = party_ledgers::statement_whole(ctx, account_id, &six_from, &six_to)
                .unwrap_or_else(|e| panic!("{who}: whole statement: {e:?}"));
            let data = party_statement_print_data(
                ctx.db(),
                &whole,
                &StatementPrintDataInput {
                    account_id,
                    from: six_from.clone(),
                    to: six_to.clone(),
                    language: Language::En,
                    paper: None,
                },
            )
            .unwrap_or_else(|e| panic!("{who}: print data: {e:?}"));
            assert!(!data.days.is_empty(), "{who}: 6 months and no day");
            data.days.len()
        };
        let days_in_print = build();
        println!("VOLUME {who}: the 6-month print has {days_in_print} days");
        table.push((
            format!("{who}: 6-month statement print data ({days_in_print} days)"),
            measure(&|| {
                build();
            }),
            Duration::from_millis(2_000),
        ));
    }
    let mut over = Vec::new();
    for (name, measured, budget) in &table {
        let ok = measured <= budget;
        println!(
            "PERF statement: {name}: p95 {:.1} ms vs {:.0} ms ({})",
            ms(*measured),
            ms(*budget),
            if ok { "OK" } else { "OVER" }
        );
        if !ok {
            over.push(name.clone());
        }
    }
    if !cfg!(debug_assertions) {
        assert!(over.is_empty(), "over the plan 74 S1 budget: {over:?}");
    }
}
