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
        language: None,
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
    // A first page is cold: the kept sums of an earlier line must not help it.
    mandi_services::db::memo::clear();
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
    check_running_sums(&big, ctx);

    let account_list = |sort_input: Option<SortInput>, offset: u32| {
        accounts::list(
            ctx,
            &AccountsListInput {
                filter: AccountFilter {
                    kind: AccountListKind::default(),
                    text: String::new(),
                    include_inactive: false,
                    has_opening: None,
                },
                offset,
                limit: PAGE,
                sort: sort_input,
            },
        )
        .expect("the account list")
    };
    lines.push(measure("accounts_list (natural)", |o| {
        account_list(None, o)
    }));
    lines.push(measure("accounts_list sort name_en", |o| {
        account_list(sort("name_en", SortDirection::Asc), o)
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
    lines.push(measure("reports R13 busiest buyer sort credit", |o| {
        ledger(sort("credit", SortDirection::Asc), false, o)
    }));
    // The middle of the ledger: the running balance is the furthest from both ends.
    lines.push(measure("reports R13 busiest buyer middle page", |o| {
        ledger(None, false, 27_000 + o)
    }));
    lines.push(measure(
        "reports R13 busiest buyer middle with months",
        |o| ledger(None, true, 27_000 + o),
    ));
    lines.push(measure("reports R13 busiest buyer newest first", |o| {
        ledger(sort("date", SortDirection::Desc), false, o)
    }));
    lines.push(measure(
        "reports R13 busiest buyer newest first, middle",
        |o| ledger(sort("date", SortDirection::Desc), true, 27_000 + o),
    ));
    lines.push(measure("reports R11 ten years newest first", |o| {
        run_report(
            ctx.db(),
            &ReportRunInput {
                id: ReportId::R11,
                filters: ReportFilters {
                    from: "11-10-2016".into(),
                    to: "10-10-2026".into(),
                    ..ReportFilters::default()
                },
                offset: i64::from(o),
                limit: i64::from(PAGE),
                sort: sort("date", SortDirection::Desc),
                group_by_month: false,
            },
        )
        .expect("the day book")
    }));
    lines.push(measure("reports R11 ten years sort debit", |o| {
        run_report(
            ctx.db(),
            &ReportRunInput {
                id: ReportId::R11,
                filters: ReportFilters {
                    from: "11-10-2016".into(),
                    to: "10-10-2026".into(),
                    ..ReportFilters::default()
                },
                offset: i64::from(o),
                limit: i64::from(PAGE),
                sort: sort("debit", SortDirection::Desc),
                group_by_month: false,
            },
        )
        .expect("the day book")
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
        voucher_list(
            None,
            sort("amount_paisa", SortDirection::Desc),
            None,
            false,
            o,
        )
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

/// The running sums of the big paths are right, whichever end of the account they are read from:
/// a few rows at the start, near the middle and at the end of the busiest buyer's ledger and of the
/// cash book are compared with the window sums the old queries gave (read from the same file by a
/// second, plain connection). Not timed.
fn check_running_sums(big: &volume::Volume, ctx: &mandi_services::Context) {
    use mandi_reports::reports::Cell;
    let raw = mandi_services::db::rusqlite::Connection::open_with_flags(
        big.dir.join("data").join("mandi.db"),
        mandi_services::db::rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .expect("the volume file");
    // ---- The party ledger R13 of the busiest buyer ----
    // Whole account, a wide range with no end, a 61-day range (walked by the day index) and a
    // range with no start. The balance is the balance of the whole account after the line, so the
    // truth is the old window sum over every line, cut to the days afterwards. Offsets near both
    // ends and at 1/4, 1/2 and 3/4 reach every plan of `lines.rs`: the walk from the near end and
    // the account-first plan far from both ends.
    let dmy = |iso: &str| format!("{}-{}-{}", &iso[8..10], &iso[5..7], &iso[..4]);
    let scenarios: [(Option<&str>, Option<&str>); 4] = [
        (None, None),
        (Some("2022-01-01"), None),
        (Some("2023-03-01"), Some("2023-04-30")),
        (None, Some("2019-06-30")),
    ];
    for (from_iso, to_iso) in scenarios {
        let in_filter: i64 = raw
            .query_row(
                "SELECT COUNT(*) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                 WHERE l.account_id = ?1 AND e.status = 'posted'
                   AND (?2 IS NULL OR e.entry_date >= ?2) AND (?3 IS NULL OR e.entry_date <= ?3)",
                mandi_services::db::rusqlite::params![big.busy_buyer, from_iso, to_iso],
                |r| r.get(0),
            )
            .expect("a count");
        assert!(in_filter > 10, "{from_iso:?} {to_iso:?}: {in_filter} lines");
        let mut offsets = vec![
            0,
            100.min(in_filter - 3),
            in_filter / 4,
            in_filter / 2 - 2,
            3 * in_filter / 4,
            in_filter - 4,
        ];
        offsets.dedup();
        for (offset, newest) in offsets
            .iter()
            .flat_map(|o| [(*o, false), (*o, true)])
            .collect::<Vec<_>>()
        {
            let way = if newest { "DESC" } else { "ASC" };
            let truth: Vec<i64> = raw
                .prepare(&format!(
                    "SELECT rd - rc FROM (
                         SELECT SUM(dr_paisa) OVER w AS rd, SUM(cr_paisa) OVER w AS rc, entry_date, id, line_no
                         FROM (SELECT l.dr_paisa, l.cr_paisa, e.entry_date, e.id, l.line_no
                               FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                               WHERE l.account_id = ?1 AND e.status = 'posted')
                         WINDOW w AS (ORDER BY entry_date, id, line_no
                                      ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW))
                     WHERE (?3 IS NULL OR entry_date >= ?3) AND (?4 IS NULL OR entry_date <= ?4)
                     ORDER BY entry_date {way}, id {way}, line_no {way} LIMIT 3 OFFSET ?2",
                ))
                .expect("a statement")
                .query_map(
                    mandi_services::db::rusqlite::params![big.busy_buyer, offset, from_iso, to_iso],
                    |r| r.get(0),
                )
                .expect("rows")
                .collect::<Result<_, _>>()
                .expect("values");
            let page = run_report(
                ctx.db(),
                &ReportRunInput {
                    id: ReportId::R13,
                    filters: ReportFilters {
                        party: Some(PartyFilter {
                            account_id: big.busy_buyer,
                            person_key: None,
                        }),
                        from: from_iso.map(dmy).unwrap_or_default(),
                        to: to_iso.map(dmy).unwrap_or_default(),
                        ..ReportFilters::default()
                    },
                    offset,
                    limit: 3,
                    sort: if newest {
                        sort("date", SortDirection::Desc)
                    } else {
                        None
                    },
                    group_by_month: false,
                },
            )
            .expect("the ledger page");
            assert_eq!(page.total_count, in_filter, "{from_iso:?} {to_iso:?}");
            let shown: Vec<i64> = page
                .rows
                .iter()
                .map(|row| match row.cells.last() {
                    Some(Cell::Balance { paisa, side }) => match side {
                        mandi_services::core::accounts::BalanceSide::Dr => paisa.as_paisa(),
                        mandi_services::core::accounts::BalanceSide::Cr => -paisa.as_paisa(),
                        mandi_services::core::accounts::BalanceSide::Zero => 0,
                    },
                    other => panic!("not a balance: {other:?}"),
                })
                .collect();
            assert_eq!(
                shown, truth,
                "R13 running balance at offset {offset}, days {from_iso:?} to {to_iso:?}, newest first {newest}"
            );
        }
    }
    // ---- The cash book ----
    let entries: i64 = raw
        .query_row(
            "SELECT COUNT(DISTINCT entry_id) FROM journal_lines WHERE account_id = 1",
            [],
            |r| r.get(0),
        )
        .expect("a count");
    for offset in [0, 100, entries / 2 - 2, entries - 4] {
        let truth: Vec<i64> = raw
            .prepare(
                "WITH p AS (SELECT e.id AS entry_id, e.entry_date AS d, SUM(l.dr_paisa) AS i, SUM(l.cr_paisa) AS o
                            FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                            WHERE l.account_id = 1 AND e.status = 'posted' GROUP BY e.id)
                 SELECT ri - ro FROM (SELECT d, entry_id, SUM(i) OVER w AS ri, SUM(o) OVER w AS ro FROM p
                                      WINDOW w AS (ORDER BY d, entry_id))
                 ORDER BY d, entry_id LIMIT 3 OFFSET ?1",
            )
            .expect("a statement")
            .query_map([offset], |r| r.get(0))
            .expect("rows")
            .collect::<Result<_, _>>()
            .expect("values");
        let book = cashbook::get(
            ctx,
            &CashbookInput {
                from: None,
                to: None,
                offset: u32::try_from(offset).expect("an offset"),
                limit: 3,
                sort: None,
                group_by_month: false,
            },
        )
        .expect("the cash book page");
        let shown: Vec<i64> = book
            .rows
            .iter()
            .map(|r| r.running_paisa.expect("a running cash").as_paisa())
            .collect();
        assert_eq!(shown, truth, "cash book running cash at offset {offset}");
        // The same rows from the other end of the book, newest first.
        let newest = u32::try_from(entries - offset - 3).expect("an offset");
        let back = cashbook::get(
            ctx,
            &CashbookInput {
                from: None,
                to: None,
                offset: newest,
                limit: 3,
                sort: sort("entry_date", SortDirection::Desc),
                group_by_month: false,
            },
        )
        .expect("the newest-first page");
        let mut backwards: Vec<i64> = back
            .rows
            .iter()
            .map(|r| r.running_paisa.expect("a running cash").as_paisa())
            .collect();
        backwards.reverse();
        assert_eq!(backwards, truth, "newest-first running cash at {offset}");
    }
}
