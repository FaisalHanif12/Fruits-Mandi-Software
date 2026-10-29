//! NFR-SPD-24 (PRD FR-19.11, Code Standards B31, plan 38 B2 slice c): a page of any list, ledger,
//! register, report or analytics table, with 10 years of data, shows its first page WITH the
//! totals in under 1 second, and each next page in under 300 ms.
//!
//! The data is the NFR-SCL-1 volume of `robustness` (`benches/support`, built once into
//! `backend/target/perf/nfr_scl1`): 20,000 accounts, 3.5 million journal lines, 1.03 million
//! posted entries, the busiest buyer with about 55,000 lines and the cash account with about
//! 514,000. The voucher list is measured on its own 110,000 vouchers, made here by plain SQL on a
//! fresh data file (the volume has no `vouchers` rows).
//!
//! This test is `#[ignore]`d: it needs the big file and a release build, because bundled SQLite
//! built without optimisation is several times slower than the real app. Run it with:
//!
//! ```text
//! cargo test --release -p mandi-services --test speed_pages -- --ignored --nocapture
//! ```
//!
//! It prints one line for each list (first page, next page, budget) and fails if a budget is
//! missed. `MANDI_SPEED_REPORT_ONLY=1` prints without failing, to read the numbers on a slow
//! machine. The budgets are the NFR's own; the reference hardware is slower than a developer's
//! machine, so a pass here is a necessary check, and the final check is on the reference
//! computer before go-live (NFR Section 2).
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "../benches/support/mod.rs"]
mod volume;

use std::time::{Duration, Instant};

use mandi_reports::analytics::{AnalyticsFilters, AnalyticsRunInput, AnalyticsView};
use mandi_reports::reports::{
    PartyFilter, ReportFilters, ReportId, ReportRunInput, run as run_report,
};
use mandi_services::core::accounts::{AccountFilter, AccountListKind, AccountsListInput};
use mandi_services::core::cashbook::CashbookInput;
use mandi_services::core::masters::{MasterKind, MastersListInput};
use mandi_services::core::paging::{SortDirection, SortInput};
use mandi_services::core::vouchers::{VoucherType, VouchersListInput};
use mandi_services::{accounts, cashbook, masters, vouchers};

/// The first page with its totals.
const FIRST_PAGE: Duration = Duration::from_millis(1_000);
/// Each next page.
const NEXT_PAGE: Duration = Duration::from_millis(300);
/// The rows of a page.
const PAGE: u32 = 100;
/// How many vouchers the voucher list is measured on (10 years at about 30 a day).
const VOUCHERS: i64 = 110_000;

fn sort(column: &str, direction: SortDirection) -> Option<SortInput> {
    Some(SortInput {
        column: column.into(),
        direction,
    })
}

/// One measured list: its name, the time of the first page, the time of the next page.
struct Line {
    name: String,
    first: Duration,
    next: Duration,
}

/// Times a list: the first page (offset 0) and the next one (offset `PAGE`), each a fresh call.
fn measure<T>(name: &str, mut page: impl FnMut(u32) -> T) -> Line {
    let started = Instant::now();
    let first_page = page(0);
    let first = started.elapsed();
    drop(first_page);
    let started = Instant::now();
    let next_page = page(PAGE);
    let next = started.elapsed();
    drop(next_page);
    Line {
        name: name.to_owned(),
        first,
        next,
    }
}

fn ms(duration: Duration) -> u128 {
    duration.as_millis()
}

/// Makes `VOUCHERS` vouchers by plain SQL on a fresh data file, over the 10 years of the volume,
/// and returns the world they belong to.
fn world_with_vouchers() -> support::World {
    let w = support::world();
    let conn = common::raw(&w.dir);
    let buyers = [w.akram, w.arshad];
    let sellers = [w.rahim, w.karim];
    let tx = conn.unchecked_transaction().expect("a transaction");
    {
        let mut statement = tx
            .prepare(
                "INSERT INTO vouchers (type, number, entry_date, account_id, amount_paisa, mode,
                                       narration, status, cancel_reason)
                 VALUES (?1, ?2, date('2016-10-11', '+' || ?3 || ' days'), ?4, ?5, 'cash', ?6, ?7, ?8)",
            )
            .expect("an insert");
        for i in 0..VOUCHERS {
            let receipt = i % 2 == 0;
            let number = i / 2 + 1;
            let account = if receipt {
                buyers[usize::try_from(i % 4 / 2).unwrap_or(0)]
            } else {
                sellers[usize::try_from(i % 4 / 2).unwrap_or(0)]
            };
            let cancelled = i % 50 == 7;
            statement
                .execute(mandi_services::db::rusqlite::params![
                    if receipt { "receipt" } else { "payment" },
                    number,
                    i % 3_650,
                    account,
                    (1 + i % 900) * 100,
                    format!("voucher {i}"),
                    if cancelled { "cancelled" } else { "saved" },
                    if cancelled { Some("wrong") } else { None },
                ])
                .expect("a voucher");
        }
    }
    tx.commit().expect("a commit");
    drop(conn);
    w
}

#[test]
#[ignore = "needs the 10-year volume and a release build: see the module doc"]
fn a_page_of_every_list_is_fast_on_ten_years_of_data() {
    let mut lines: Vec<Line> = Vec::new();

    // ---- The volume: accounts, masters, the cash book, ledgers ----
    let big = volume::ensure_volume();
    volume::describe(&big);
    let (env, _actor) = volume::open(&big.dir);
    let ctx = &env.ctx;

    let account_list = |sort_input: Option<SortInput>, offset: u32| {
        accounts::list(
            ctx,
            &AccountsListInput {
                filter: AccountFilter {
                    kind: AccountListKind::default(),
                    text: String::new(),
                    include_inactive: false,
                },
                offset,
                limit: PAGE,
                sort: sort_input,
            },
        )
        .expect("the account list")
    };
    lines.push(measure("accounts_list (natural)", |o| account_list(None, o)));
    lines.push(measure("accounts_list sort name_en", |o| {
        account_list(sort("name_en", SortDirection::Asc), o)
    }));
    lines.push(measure("accounts_list sort balance", |o| {
        account_list(sort("balance", SortDirection::Desc), o)
    }));
    lines.push(measure("masters_list items (natural)", |o| {
        masters::list_page(
            ctx,
            &MastersListInput {
                kind: MasterKind::Item,
                include_inactive: false,
                id: None,
                offset: o,
                limit: PAGE,
                search: None,
                sort: None,
            },
        )
        .expect("the item list")
    }));
    lines.push(measure("masters_list items sort name_en + search", |o| {
        masters::list_page(
            ctx,
            &MastersListInput {
                kind: MasterKind::Item,
                include_inactive: false,
                id: None,
                offset: o,
                limit: PAGE,
                search: Some("a".into()),
                sort: sort("name_en", SortDirection::Asc),
            },
        )
        .expect("the item list")
    }));

    let book = |sort_input: Option<SortInput>, group_by_month: bool, offset: u32| {
        cashbook::get(
            ctx,
            &CashbookInput {
                from: None,
                to: None,
                offset,
                limit: PAGE,
                sort: sort_input,
                group_by_month,
            },
        )
        .expect("the cash book")
    };
    lines.push(measure("cashbook_get (natural, 514k lines)", |o| {
        book(None, false, o)
    }));
    lines.push(measure("cashbook_get sort out_paisa", |o| {
        book(sort("out_paisa", SortDirection::Desc), false, o)
    }));
    lines.push(measure("cashbook_get with month subtotals", |o| {
        book(None, true, o)
    }));

    // The party ledger R13 of the busiest buyer (about 55,000 lines).
    let ledger = |sort_input: Option<SortInput>, months: bool, offset: u32| {
        run_report(
            ctx.db(),
            &ReportRunInput {
                id: ReportId::R13,
                filters: ReportFilters {
                    party: Some(PartyFilter {
                        account_id: big.busy_buyer,
                        person_key: None,
                    }),
                    ..ReportFilters::default()
                },
                offset: i64::from(offset),
                limit: i64::from(PAGE),
                sort: sort_input,
                group_by_month: months,
            },
        )
        .expect("the ledger")
    };
    lines.push(measure("reports R13 busiest buyer (natural)", |o| {
        ledger(None, false, o)
    }));
    lines.push(measure("reports R13 busiest buyer with months", |o| {
        ledger(None, true, o)
    }));
    lines.push(measure("reports R13 busiest buyer sort debit", |o| {
        ledger(sort("debit", SortDirection::Desc), false, o)
    }));
    lines.push(measure("reports R11 day book (10 years)", |o| {
        run_report(
            ctx.db(),
            &ReportRunInput {
                id: ReportId::R11,
                filters: ReportFilters {
                    from: "01-10-2026".into(),
                    to: "10-10-2026".into(),
                    ..ReportFilters::default()
                },
                offset: i64::from(o),
                limit: i64::from(PAGE),
                sort: None,
                group_by_month: false,
            },
        )
        .expect("the day book")
    }));
    lines.push(measure("analytics view B by seller", |o| {
        mandi_reports::analytics::run(
            ctx.db(),
            &AnalyticsRunInput {
                view: AnalyticsView::B,
                filters: AnalyticsFilters::default(),
                offset: i64::from(o),
                limit: i64::from(PAGE),
                sort: None,
            },
        )
        .expect("the analytics view")
    }));

    // ---- Vouchers, on their own data file ----
    let w = world_with_vouchers();
    let voucher_list = |kind: Option<VoucherType>,
                        sort_input: Option<SortInput>,
                        search: Option<&str>,
                        months: bool,
                        offset: u32| {
        vouchers::list(
            &w.env.ctx,
            &VouchersListInput {
                voucher_type: kind,
                from: String::new(),
                to: String::new(),
                account_id: None,
                person_key: None,
                offset,
                limit: PAGE,
                show_finished: false,
                search: search.map(str::to_owned),
                sort: sort_input,
                group_by_month: months,
            },
        )
        .expect("the voucher list")
    };
    lines.push(measure("vouchers_list (110k, natural)", |o| {
        voucher_list(None, None, None, false, o)
    }));
    lines.push(measure("vouchers_list one type", |o| {
        voucher_list(Some(VoucherType::Receipt), None, None, false, o)
    }));
    lines.push(measure("vouchers_list sort amount_paisa", |o| {
        voucher_list(None, sort("amount_paisa", SortDirection::Desc), None, false, o)
    }));
    lines.push(measure("vouchers_list sort account", |o| {
        voucher_list(None, sort("account", SortDirection::Asc), None, false, o)
    }));
    lines.push(measure("vouchers_list search text", |o| {
        voucher_list(None, None, Some("voucher 1234"), false, o)
    }));
    lines.push(measure("vouchers_list with month subtotals", |o| {
        voucher_list(Some(VoucherType::Payment), None, None, true, o)
    }));

    // ---- The report ----
    volume::note("NFR-SPD-24: first page with totals < 1000 ms, next page < 300 ms");
    let mut missed = Vec::new();
    for line in &lines {
        let verdict = if line.first <= FIRST_PAGE && line.next <= NEXT_PAGE {
            "ok"
        } else {
            "SLOW"
        };
        volume::note(&format!(
            "{verdict:>4}  first {:>6} ms  next {:>6} ms  {}",
            ms(line.first),
            ms(line.next),
            line.name
        ));
        if verdict == "SLOW" {
            missed.push(line.name.clone());
        }
    }
    if std::env::var("MANDI_SPEED_REPORT_ONLY").is_err() {
        assert!(missed.is_empty(), "too slow: {missed:?}");
    }
}
