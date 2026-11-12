//! Plan 72 (پچھلی بکری, PRD 12.7, P4 to P8, P11; NFR-SPD-24; Code Standards B31): speed on 10 years of بکری
//! and attacks on the windows, the filters and the totals. Owner: `robustness`.
//!
//! The screen asks the backend for `bikri::list` three ways (plan 72 Section 3 as built): today's section
//! (`date` = today), the 7-day window (`from` and `to`, `group_by_day`), and the window after a jump (`to` =
//! the day typed). Each is one page plus the totals over the whole filter plus the day headings.
//!
//! Attacks (small data, always run):
//! 1. `windows_at_month_and_year_ends_equal_the_stored_rows`: windows that end on and cross 28 and 29 February,
//!    30 and 31 of a month, 31 December and 1 January, today and yesterday. Count, totals and every day
//!    heading equal plain SQL over the same rows; a day with no بکری is a heading with 0; today is not in the
//!    window that ends yesterday and is in the one that ends today.
//! 2. `a_date_after_today_and_other_bad_dates_are_refused`.
//! 3. `hostile_filter_text`: SQL, wildcards, control and bidi text, a 100,000 character text, numbers past i64;
//!    never a panic, never everybody for text that matches nothing.
//! 4. `limits_offsets_and_the_day_range_at_their_edges`.
//! 5. `totals_are_the_sum_of_every_page_and_of_the_days`: for the default, `all` and filtered lists.
//!
//! Volume (release, `MANDI_PERF_VOLUME=1`; builds about 1.6 million sale lines once, in `target/perf`):
//! 6. `at_ten_years_the_windows_equal_plain_sql_and_pages_add_up`.
//! 7. `at_ten_years_the_95th_percentile_against_nfr_spd_24`: open, one arrow step, go to a date 5 years
//!    back, with `all`, with filters. Held on CPU time scaled to the reference machine (scripts/perf/README.md
//!    Section 2: 3,752 / 2,268); the wall p95 is printed.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_save/helpers.rs"]
mod helpers;
mod robustness_past;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;
#[path = "../benches/support/mod.rs"]
mod volume;

use std::time::{Duration, Instant};

use chrono::{Duration as Days, NaiveDate};
use common::raw;
use mandi_services::core::bikri::{BikriList, BikriListInput, BikriStatus};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::paging::{SortDirection, SortInput};
use mandi_services::{Context, ServiceError, bikri};
use support::*;

// ---- helpers ------------------------------------------------------------------------------------------

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 10).unwrap()
}

fn ddmmyyyy(d: NaiveDate) -> String {
    d.format("%d-%m-%Y").to_string()
}

fn iso(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn input() -> BikriListInput {
    BikriListInput::default()
}

/// The seven days that end on `end` (inclusive), as the screen asks for them.
fn window(end: NaiveDate, limit: u32, offset: u32) -> BikriListInput {
    BikriListInput {
        from: Some(ddmmyyyy(end - Days::days(6))),
        to: Some(ddmmyyyy(end)),
        group_by_day: true,
        limit,
        offset,
        ..input()
    }
}

fn day_of(d: NaiveDate, limit: u32) -> BikriListInput {
    BikriListInput {
        date: Some(ddmmyyyy(d)),
        limit,
        ..input()
    }
}

fn list(ctx: &Context, i: &BikriListInput) -> Result<BikriList, ServiceError> {
    bikri::list(ctx, i)
}

fn ok(ctx: &Context, i: &BikriListInput) -> BikriList {
    list(ctx, i).unwrap_or_else(|e| panic!("{i:?}: {e:?}"))
}

/// Put one بکری of one lot and one line on a day, with stored sums, by plain SQL.
/// Returns its id. `gross` is in rupees; the buyer total is the same (no commission or لاگا).
fn put(
    w: &World,
    number: i64,
    day: NaiveDate,
    status: &str,
    gross: i64,
    truck: Option<&str>,
) -> i64 {
    put_on(&raw(&w.dir), w, number, day, status, gross, truck)
}

/// [`put`] on a connection the caller keeps (many rows quickly).
fn put_on(
    conn: &mandi_services::db::rusqlite::Connection,
    w: &World,
    number: i64,
    day: NaiveDate,
    status: &str,
    gross: i64,
    truck: Option<&str>,
) -> i64 {
    let day = iso(day);
    let draft: i64 = match conn.query_row(
        "SELECT id FROM journal_entries WHERE status = 'draft' AND posting_type = 'bikri_first' LIMIT 1",
        [],
        |r| r.get(0),
    ) {
        Ok(id) => id,
        Err(_) => {
            conn.execute(
                "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, status, total_paisa)
                 VALUES (?1, 'bikri_first', 'bikri', 1, 'draft', 0)",
                [&day],
            )
            .unwrap();
            conn.last_insert_rowid()
        }
    };
    conn.execute(
        "INSERT INTO bikri_entries (number, entry_date, bill_date, truck_no, status, cancel_reason)
         VALUES (?1, ?2, ?2, ?3, ?4, CASE WHEN ?4 = 'cancelled' THEN 'wrong' END)",
        mandi_services::db::rusqlite::params![number, day, truck, status],
    )
    .unwrap();
    let id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO bikri_lots (bikri_id, lot_no, seller_account_id, item_id, packing_id, qty_received, qty_sold,
            gross_paisa, net_paisa, lot_counter)
         VALUES (?1, 1, ?2, ?3, ?4, 10, 10, ?5, ?5, (SELECT COALESCE(MAX(lot_counter), 0) + 1 FROM bikri_lots
                 WHERE seller_account_id = ?2 AND item_id = ?3))",
        mandi_services::db::rusqlite::params![id, w.rahim, w.banana, w.box_, gross * 100],
    )
    .unwrap();
    let lot = conn.last_insert_rowid();
    // No commission and no لاگا on these test lines: the buyer total is the value (a table CHECK).
    let total = gross;
    conn.execute(
        "INSERT INTO bikri_sales (bikri_id, lot_id, line_no, journal_entry_id, entry_date, buyer_account_id, qty,
            seller_rate_paisa, seller_amount_paisa, seller_comm_method, seller_comm_value_bps, seller_comm_paisa,
            buyer_rate_paisa, buyer_value_paisa, buyer_comm_method, buyer_comm_value_bps, buyer_comm_paisa,
            laga_method, laga_value_paisa, laga_paisa, buyer_total_paisa, rate_diff_paisa, payment_type)
         VALUES (?1, ?2, 1, ?3, ?4, ?5, 10, ?8, ?6, 'percent', 0, 0, ?8, ?6, 'percent', 0, 0, 'per_unit', 0, 0,
            ?7, 0, 'credit')",
        mandi_services::db::rusqlite::params![id, lot, draft, day, w.akram, gross * 100, total * 100, gross * 10],
    )
    .unwrap();
    id
}

/// What plain SQL says about a range: the default filter (saved only) or `all`.
struct Sql {
    ids_newest_first: Vec<i64>,
    count: i64,
    gross: i64,
    net: i64,
    buyer_total: i64,
    days: Vec<(String, i64, i64)>,
}

fn sql_of(w: &World, from: NaiveDate, to: NaiveDate, all: bool) -> Sql {
    let conn = raw(&w.dir);
    let (f, t) = (iso(from), iso(to));
    let cond = if all { "" } else { " AND e.status = 'saved'" };
    let ids: Vec<i64> = conn
        .prepare(&format!(
            "SELECT e.id FROM bikri_entries e WHERE e.entry_date BETWEEN ?1 AND ?2{cond} ORDER BY e.entry_date DESC, e.number DESC"
        ))
        .unwrap()
        .query_map([&f, &t], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let one = |sql: &str| -> i64 { conn.query_row(sql, [&f, &t], |r| r.get(0)).unwrap() };
    let saved = " e.entry_date BETWEEN ?1 AND ?2 AND e.status = 'saved'";
    let gross = one(&format!(
        "SELECT COALESCE(SUM(l.gross_paisa), 0) FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id WHERE{saved}"
    ));
    let net = one(&format!(
        "SELECT COALESCE(SUM(l.net_paisa), 0) FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id WHERE{saved}"
    ));
    let buyer_total = one(&format!(
        "SELECT COALESCE(SUM(s.buyer_total_paisa), 0) FROM bikri_sales s JOIN bikri_entries e ON e.id = s.bikri_id WHERE{saved} AND s.is_active = 1"
    ));
    let mut days = Vec::new();
    let mut d = to;
    while d >= from {
        let day = iso(d);
        let count: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM bikri_entries e WHERE e.entry_date = ?1{cond}"),
                [&day],
                |r| r.get(0),
            )
            .unwrap();
        let g: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(l.gross_paisa), 0) FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id
                 WHERE e.entry_date = ?1 AND e.status = 'saved'",
                [&day],
                |r| r.get(0),
            )
            .unwrap();
        days.push((day, count, g));
        d -= Days::days(1);
    }
    Sql {
        count: ids.len() as i64,
        ids_newest_first: ids,
        gross,
        net,
        buyer_total,
        days,
    }
}

/// Walks every page of an input and returns all ids, the sum of the gross of the saved rows and the first page.
fn walk(ctx: &Context, base: &BikriListInput, page: u32) -> (Vec<i64>, i64, BikriList) {
    let first = ok(
        ctx,
        &BikriListInput {
            limit: page,
            offset: 0,
            ..base.clone()
        },
    );
    let mut ids = Vec::new();
    let mut gross = 0_i64;
    let mut offset = 0_u32;
    loop {
        let p = ok(
            ctx,
            &BikriListInput {
                limit: page,
                offset,
                ..base.clone()
            },
        );
        assert_eq!(
            p.total_count, first.total_count,
            "the count moves with the page"
        );
        assert_eq!(p.totals, first.totals, "the totals move with the page");
        assert_eq!(p.days, first.days, "the days move with the page");
        if p.rows.is_empty() {
            break;
        }
        offset += p.rows.len() as u32;
        for r in &p.rows {
            ids.push(r.id);
            if r.status == BikriStatus::Saved {
                gross += r.gross.as_paisa();
            }
        }
    }
    (ids, gross, first)
}

fn check_against_sql(
    w: &World,
    ctx: &Context,
    from: NaiveDate,
    to: NaiveDate,
    all: bool,
    what: &str,
) {
    let sql = sql_of(w, from, to, all);
    let base = BikriListInput {
        from: Some(ddmmyyyy(from)),
        to: Some(ddmmyyyy(to)),
        group_by_day: true,
        all,
        ..input()
    };
    let (ids, gross_rows, first) = walk(ctx, &base, 25);
    assert_eq!(ids, sql.ids_newest_first, "{what}: rows");
    assert_eq!(first.total_count, sql.count, "{what}: count");
    assert_eq!(first.totals.gross.as_paisa(), sql.gross, "{what}: gross");
    assert_eq!(first.totals.net.as_paisa(), sql.net, "{what}: net");
    assert_eq!(
        first.totals.buyer_total.as_paisa(),
        sql.buyer_total,
        "{what}: buyer total"
    );
    assert_eq!(
        gross_rows, sql.gross,
        "{what}: the gross of the rows of every page"
    );
    // Day headings: one for each day of the range, newest first, empty days included.
    let days: Vec<(String, i64, i64)> = first
        .days
        .iter()
        .map(|d| (d.date.clone(), d.count, d.gross_paisa.as_paisa()))
        .collect();
    assert_eq!(days, sql.days, "{what}: days");
    assert_eq!(
        first.days.iter().map(|d| d.count).sum::<i64>(),
        first.total_count,
        "{what}: day counts add up"
    );
    assert_eq!(
        first
            .days
            .iter()
            .map(|d| d.gross_paisa.as_paisa())
            .sum::<i64>(),
        first.totals.gross.as_paisa(),
        "{what}: day gross adds up"
    );
}

fn rejected(e: &ServiceError) -> bool {
    matches!(e, ServiceError::Rejected(_))
}

// ---- Attacks on small data -----------------------------------------------------------------------------

#[test]
fn windows_at_month_and_year_ends_equal_the_stored_rows() {
    let w = world();
    let d = |y, m, dd| NaiveDate::from_ymd_opt(y, m, dd).unwrap();
    // Days around the ends, some with two بکری, one cancelled, one with a truck, none on others.
    let days = [
        d(2024, 2, 28),
        d(2024, 2, 29),
        d(2024, 3, 1),
        d(2025, 2, 28),
        d(2025, 3, 1),
        d(2025, 12, 30),
        d(2025, 12, 31),
        d(2026, 1, 1),
        d(2026, 1, 2),
        d(2026, 3, 31),
        d(2026, 4, 1),
        d(2026, 4, 30),
        d(2026, 5, 1),
        d(2026, 9, 30),
        d(2026, 10, 1),
        d(2026, 10, 9),
        d(2026, 10, 10),
    ];
    let mut number = 1;
    for (i, day) in days.iter().enumerate() {
        put(
            &w,
            number,
            *day,
            "saved",
            1_000 + i as i64 * 100,
            Some("LES-1"),
        );
        number += 1;
        if i % 3 == 0 {
            put(&w, number, *day, "saved", 500, None);
            number += 1;
        }
        if i % 4 == 1 {
            put(&w, number, *day, "cancelled", 9_900, Some("CAN-1"));
            number += 1;
        }
    }
    let ctx = &*w.env.ctx;
    // Windows that end on a day before, on and after an end; each 7 days and each 7 days back from it.
    for end in [
        d(2024, 2, 28),
        d(2024, 2, 29),
        d(2024, 3, 1),
        d(2024, 3, 6),
        d(2025, 2, 28),
        d(2025, 3, 6),
        d(2025, 12, 31),
        d(2026, 1, 1),
        d(2026, 1, 7),
        d(2026, 4, 30),
        d(2026, 5, 6),
        d(2026, 3, 31),
        d(2026, 10, 9),
        d(2026, 10, 10),
    ] {
        for step in [0_i64, 7, 14] {
            let to = end - Days::days(step);
            let from = to - Days::days(6);
            for all in [false, true] {
                check_against_sql(
                    &w,
                    ctx,
                    from,
                    to,
                    all,
                    &format!("window {from} to {to}, all {all}"),
                );
            }
        }
    }
    // Today's section: one day, date = today; and the window that ends yesterday does not hold today.
    let today_rows = ok(ctx, &day_of(today(), 100));
    assert_eq!(
        today_rows.total_count,
        sql_of(&w, today(), today(), false).count
    );
    let yesterday = today() - Days::days(1);
    let win = ok(ctx, &window(yesterday, 100, 0));
    assert!(
        win.rows.iter().all(|r| r.entry_date != iso(today())),
        "today is in the window that ends yesterday"
    );
    assert_eq!(
        win.days.first().map(|d| d.date.clone()),
        Some(iso(yesterday))
    );
    assert_eq!(win.days.len(), 7);
    let with_today = ok(ctx, &window(today(), 100, 0));
    assert!(with_today.rows.iter().any(|r| r.entry_date == iso(today())));
    // An empty window: a heading for each day with 0.
    let empty = ok(ctx, &window(d(2023, 6, 30), 100, 0));
    assert_eq!(empty.total_count, 0);
    assert_eq!(empty.days.len(), 7);
    assert!(
        empty
            .days
            .iter()
            .all(|d| d.count == 0 && d.gross_paisa.as_paisa() == 0)
    );
    assert_eq!(empty.days.first().unwrap().date, "2023-06-30");
    assert_eq!(empty.days.last().unwrap().date, "2023-06-24");
    // A one-day range equals that day's heading.
    for day in [d(2024, 2, 29), d(2026, 1, 1), d(2025, 12, 31)] {
        let one = ok(
            ctx,
            &BikriListInput {
                from: Some(ddmmyyyy(day)),
                to: Some(ddmmyyyy(day)),
                group_by_day: true,
                limit: 100,
                ..input()
            },
        );
        assert_eq!(one.days.len(), 1);
        assert_eq!(one.days[0].count, one.total_count);
    }
}

#[test]
fn a_date_after_today_and_other_bad_dates_are_refused() {
    let w = world();
    put(&w, 1, today(), "saved", 1_000, None);
    let ctx = &*w.env.ctx;
    let count = |w: &World| -> i64 {
        raw(&w.dir)
            .query_row("SELECT COUNT(*) FROM bikri_entries", [], |r| r.get(0))
            .unwrap()
    };
    let before = count(&w);
    let tomorrow = ddmmyyyy(today() + Days::days(1));
    let far = ddmmyyyy(today() + Days::days(4_000));
    let past = ddmmyyyy(today() - Days::days(6));
    let gd = |from: Option<String>, to: Option<String>| BikriListInput {
        from,
        to,
        group_by_day: true,
        ..input()
    };
    // Each of these must be refused cleanly.
    let must_refuse: Vec<(&str, BikriListInput)> = vec![
        (
            "to tomorrow",
            gd(Some(past.clone()), Some(tomorrow.clone())),
        ),
        ("to far future", gd(Some(past.clone()), Some(far.clone()))),
        (
            "from tomorrow",
            gd(Some(tomorrow.clone()), Some(far.clone())),
        ),
        (
            "date tomorrow",
            BikriListInput {
                date: Some(tomorrow.clone()),
                ..input()
            },
        ),
        (
            "date far",
            BikriListInput {
                date: Some(far.clone()),
                group_by_day: true,
                ..input()
            },
        ),
        (
            "31 February",
            gd(Some("31-02-2026".into()), Some("07-03-2026".into())),
        ),
        (
            "29 February 2026",
            BikriListInput {
                date: Some("29-02-2026".into()),
                ..input()
            },
        ),
        (
            "zero date",
            BikriListInput {
                date: Some("00-00-0000".into()),
                ..input()
            },
        ),
        (
            "text date",
            BikriListInput {
                date: Some("yesterday".into()),
                ..input()
            },
        ),
        (
            "sql date",
            BikriListInput {
                date: Some("'; DROP TABLE bikri_entries; --".into()),
                ..input()
            },
        ),
        (
            "nul date",
            gd(Some("10-10-2026\u{0}".into()), Some("10-10-2026".into())),
        ),
        ("reversed", gd(Some(ddmmyyyy(today())), Some(past.clone()))),
    ];
    for (what, i) in must_refuse {
        match list(ctx, &i) {
            Err(e) => assert!(rejected(&e), "{what}: not a clean refusal: {e:?}"),
            Ok(l) => panic!(
                "{what}: answered with {} rows and {} days",
                l.rows.len(),
                l.days.len()
            ),
        }
    }
    // A date in the future is refused with the code of the future, on the right field.
    let e = list(ctx, &gd(Some(past), Some(tomorrow))).unwrap_err();
    match e {
        ServiceError::Rejected(items) => assert!(
            items
                .iter()
                .any(|i| i.code == ErrorCode::ErrDateInFuture && i.field.as_deref() == Some("to")),
            "{items:?}"
        ),
        other => panic!("{other:?}"),
    }
    assert_eq!(before, count(&w), "a refusal wrote something");
}

/// A window typed in a very old year (the date box takes `01-01-0100`, and a year 99 is a real year for
/// a date but not for some date libraries): the answer is an empty window of the right days, or a clean
/// refusal. It is never an internal error, a wrong year (1999 for 0099) or rows of another day.
#[test]
fn windows_in_very_old_years_are_answered_empty_or_refused_cleanly() {
    let w = world();
    put(&w, 1, today(), "saved", 1_000, None);
    let ctx = &*w.env.ctx;
    let windows = [
        ("26-12-0099", "01-01-0100"),
        ("01-01-0100", "07-01-0100"),
        ("01-01-0001", "07-01-0001"),
        ("26-12-0000", "01-01-0001"),
        ("01-01-0050", "07-01-0050"),
        ("01-01-1900", "07-01-1900"),
        ("26-12-1969", "01-01-1970"),
        ("01-01-2000", "07-01-2000"),
        ("23-02-2024", "29-02-2024"),
        ("01-01-10000", "07-01-10000"),
        ("01-01-9999", "07-01-9999"),
    ];
    for (from, to) in windows {
        let result = list(
            ctx,
            &BikriListInput {
                from: Some(from.into()),
                to: Some(to.into()),
                group_by_day: true,
                limit: 100,
                ..input()
            },
        );
        match result {
            Err(e) => assert!(rejected(&e), "{from} to {to}: not a clean refusal: {e:?}"),
            Ok(l) => {
                assert!(l.rows.is_empty(), "{from} to {to}: rows of another day");
                assert_eq!(l.total_count, 0, "{from} to {to}");
                assert_eq!(l.days.len(), 7, "{from} to {to}: the seven days");
                assert!(
                    l.days
                        .iter()
                        .all(|d| d.count == 0 && d.gross_paisa.as_paisa() == 0),
                    "{from} to {to}: a day with a count"
                );
                // The days are the days asked for, in the year asked for.
                let year = to.rsplit('-').next().unwrap_or("");
                let want = year.trim_start_matches('0');
                assert!(
                    l.days.first().is_some_and(|d| d.date.starts_with(year)
                        || d.date.trim_start_matches('0').starts_with(want)),
                    "{from} to {to}: the first day is {:?}",
                    l.days.first().map(|d| d.date.clone())
                );
            }
        }
    }
    // The same days one at a time.
    for day in ["01-01-0100", "01-01-0001", "01-01-1900", "31-12-9999"] {
        match list(
            ctx,
            &BikriListInput {
                date: Some(day.into()),
                ..input()
            },
        ) {
            Err(e) => assert!(rejected(&e), "{day}: not a clean refusal: {e:?}"),
            Ok(l) => assert!(l.rows.is_empty() && l.total_count == 0, "{day}: rows"),
        }
    }
}

#[test]
fn hostile_filter_text() {
    let w = world();
    for n in 0..30 {
        put(
            &w,
            n + 1,
            today() - Days::days(n % 7),
            "saved",
            1_000 + n * 10,
            Some(if n % 2 == 0 { "LES-1234" } else { "100-ABC" }),
        );
    }
    let ctx = &*w.env.ctx;
    let base = window(today(), 100, 0);
    let all_rows = ok(ctx, &base).total_count;
    assert_eq!(all_rows, 30);
    let nonsense = [
        "%",
        "_",
        "\\",
        "%%",
        "_%",
        "'",
        "\"",
        "' OR 1=1 --",
        "'; DROP TABLE bikri_entries; --",
        "%' OR '1'='1",
        "<script>",
        "..\\..\\",
        "\u{0}",
        "a\u{0}b",
        "\u{202e}",
        "\u{200b}",
        "😀",
        "ZZZZ",
        "99999999999999999999",
        "-1",
        "-0",
        "1e5",
        "0x1F",
        "٣",
        "لل",
    ];
    for text in nonsense {
        let i = BikriListInput {
            search: Some(text.to_owned()),
            ..base.clone()
        };
        match list(ctx, &i) {
            Ok(l) => {
                assert!(l.rows.len() <= 100, "{text:?}");
                assert!(
                    l.total_count < all_rows || text.trim().is_empty() || text == "\u{200b}",
                    "text {text:?} matched all {} بکری",
                    l.total_count
                );
                // Totals and days follow the same filter.
                assert_eq!(
                    l.days.iter().map(|d| d.count).sum::<i64>(),
                    l.total_count,
                    "{text:?}"
                );
            }
            Err(e) => assert!(rejected(&e), "{text:?}: {e:?}"),
        }
    }
    // Wildcards are text, not patterns: none of them finds a truck.
    for text in ["%", "_", "\\", "%%"] {
        let l = ok(
            ctx,
            &BikriListInput {
                search: Some(text.to_owned()),
                ..base.clone()
            },
        );
        assert_eq!(l.total_count, 0, "{text:?} acted as a pattern");
    }
    // A number finds that بکری or the trucks that start with those digits; a truck prefix finds its trucks.
    let l = ok(
        ctx,
        &BikriListInput {
            search: Some("100".to_owned()),
            ..base.clone()
        },
    );
    assert!(
        l.total_count >= 15,
        "the trucks that start with 100: {}",
        l.total_count
    );
    let l = ok(
        ctx,
        &BikriListInput {
            search: Some("les".to_owned()),
            ..base.clone()
        },
    );
    assert_eq!(l.total_count, 15);
    // Ids that are nobody's.
    for id in [0_i64, -1, i64::MIN, i64::MAX, 987_654_321] {
        let l = ok(
            ctx,
            &BikriListInput {
                seller_id: Some(id),
                ..base.clone()
            },
        );
        assert_eq!(l.total_count, 0, "seller {id}");
        let l = ok(
            ctx,
            &BikriListInput {
                item_id: Some(id),
                ..base.clone()
            },
        );
        assert_eq!(l.total_count, 0, "item {id}");
    }
    // A sort by an unknown column is refused; every known column works with the days.
    let e = list(
        ctx,
        &BikriListInput {
            sort: Some(SortInput {
                column: "e.id; DROP TABLE bikri_entries".into(),
                direction: SortDirection::Desc,
                language: None,
            }),
            ..base.clone()
        },
    )
    .unwrap_err();
    assert!(rejected(&e), "{e:?}");
    for column in [
        "number",
        "entry_date",
        "truck_no",
        "status",
        "qty_received",
        "qty_sold",
        "gross",
        "net",
        "buyer_total",
    ] {
        for direction in [SortDirection::Asc, SortDirection::Desc] {
            let l = ok(
                ctx,
                &BikriListInput {
                    sort: Some(SortInput {
                        column: column.into(),
                        direction,
                        language: None,
                    }),
                    ..base.clone()
                },
            );
            assert_eq!(l.total_count, 30, "{column}");
            assert_eq!(
                l.days.iter().map(|d| d.count).sum::<i64>(),
                30,
                "{column}: days"
            );
        }
    }
    let rows: i64 = raw(&w.dir)
        .query_row("SELECT COUNT(*) FROM bikri_entries", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 30, "the table is not as it was");
}

/// A search text longer than SQLite's pattern limit (50,000 bytes) must be refused with a code, like the
/// چکتا search does (`ERR_TEXT_TOO_LONG`), not end as an internal database error.
#[test]
fn a_very_long_search_text_is_refused_cleanly_not_as_a_database_error() {
    let w = world();
    put(&w, 1, today(), "saved", 1_000, Some("LES-1"));
    let ctx = &*w.env.ctx;
    for len in [
        1_000_usize,
        10_000,
        49_000,
        49_999,
        50_000,
        100_000,
        1_000_000,
    ] {
        let started = Instant::now();
        let result = list(
            ctx,
            &BikriListInput {
                date: Some(ddmmyyyy(today())),
                search: Some("L".repeat(len)),
                ..input()
            },
        );
        match result {
            Ok(l) => assert_eq!(l.total_count, 0, "{len} letters"),
            Err(e) => assert!(
                rejected(&e),
                "a search of {len} letters ended as {e:?}, not a refusal with a code"
            ),
        }
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "a search of {len} letters took {:?}",
            started.elapsed()
        );
    }
}

#[test]
fn limits_offsets_and_the_day_range_at_their_edges() {
    let w = world();
    for n in 0..60 {
        put(
            &w,
            n + 1,
            today() - Days::days(n % 10),
            if n % 9 == 0 { "cancelled" } else { "saved" },
            1_000 + n * 7 * 100,
            None,
        );
    }
    let ctx = &*w.env.ctx;
    let base = BikriListInput {
        from: Some(ddmmyyyy(today() - Days::days(6))),
        to: Some(ddmmyyyy(today())),
        group_by_day: true,
        ..input()
    };
    let whole = ok(
        ctx,
        &BikriListInput {
            limit: 100,
            ..base.clone()
        },
    );
    assert!(whole.total_count > 25);
    let n = whole.total_count as u32;
    for (offset, limit) in [
        (0_u32, 0_u32),
        (0, 1),
        (0, 24),
        (0, 25),
        (0, 26),
        (0, 50),
        (0, 99),
        (0, 100),
        (0, 101),
        (0, 1_000_000),
        (0, u32::MAX),
        (1, 100),
        (n - 1, 100),
        (n, 100),
        (n + 1, 100),
        (1_000_000, 100),
        (u32::MAX, 100),
        (u32::MAX, u32::MAX),
    ] {
        let l = ok(
            ctx,
            &BikriListInput {
                offset,
                limit,
                ..base.clone()
            },
        );
        assert!(
            l.rows.len() <= 100,
            "offset {offset} limit {limit}: {} rows",
            l.rows.len()
        );
        assert_eq!(
            l.total_count, whole.total_count,
            "offset {offset} limit {limit}: count"
        );
        assert_eq!(
            l.totals, whole.totals,
            "offset {offset} limit {limit}: totals follow the page"
        );
        assert_eq!(
            l.days, whole.days,
            "offset {offset} limit {limit}: days follow the page"
        );
        if limit != 0 {
            let want = (n as usize)
                .saturating_sub(offset as usize)
                .min(limit.min(100) as usize);
            assert_eq!(l.rows.len(), want, "offset {offset} limit {limit}");
        }
        // The rows are the whole's rows from the offset on.
        let skip = (offset as usize).min(whole.rows.len());
        let expect: Vec<i64> = whole
            .rows
            .iter()
            .skip(skip)
            .take(l.rows.len())
            .map(|r| r.id)
            .collect();
        assert_eq!(
            l.rows.iter().map(|r| r.id).collect::<Vec<_>>(),
            expect,
            "offset {offset} limit {limit}"
        );
    }
    // Pages of 3, 25, 26, 50 and 100 put together are the whole, in order, none twice.
    for page in [3_u32, 25, 26, 50, 100] {
        let (ids, _gross, _first) = walk(ctx, &base, page);
        assert_eq!(
            ids,
            whole.rows.iter().map(|r| r.id).collect::<Vec<_>>(),
            "page size {page}"
        );
    }
    // The day range: 366 days work with one heading each; 367, reversed and missing ends are refused.
    let end = today();
    let year = ok(
        ctx,
        &BikriListInput {
            from: Some(ddmmyyyy(end - Days::days(365))),
            to: Some(ddmmyyyy(end)),
            group_by_day: true,
            limit: 25,
            ..input()
        },
    );
    assert_eq!(year.days.len(), 366);
    for i in [
        BikriListInput {
            from: Some(ddmmyyyy(end - Days::days(366))),
            to: Some(ddmmyyyy(end)),
            group_by_day: true,
            ..input()
        },
        BikriListInput {
            from: Some(ddmmyyyy(end)),
            to: Some(ddmmyyyy(end - Days::days(1))),
            group_by_day: true,
            ..input()
        },
        BikriListInput {
            from: Some(ddmmyyyy(end)),
            group_by_day: true,
            ..input()
        },
        BikriListInput {
            to: Some(ddmmyyyy(end)),
            group_by_day: true,
            ..input()
        },
        BikriListInput {
            group_by_day: true,
            ..input()
        },
    ] {
        match list(ctx, &i) {
            Err(e) => assert!(rejected(&e), "{i:?}: {e:?}"),
            Ok(l) => panic!("{i:?} was answered with {} days", l.days.len()),
        }
    }
    // Without group_by_day no days come.
    let plain = ok(
        ctx,
        &BikriListInput {
            group_by_day: false,
            ..base.clone()
        },
    );
    assert!(plain.days.is_empty());
}

#[test]
fn totals_are_the_sum_of_every_page_and_of_the_days() {
    let w = world();
    for n in 0..80 {
        let status = if n % 7 == 0 { "cancelled" } else { "saved" };
        put(
            &w,
            n + 1,
            today() - Days::days(n % 12),
            status,
            1_000 + n * 37 * 100,
            Some(if n % 3 == 0 { "LES-7" } else { "AB-1" }),
        );
    }
    let ctx = &*w.env.ctx;
    let to = today() - Days::days(1);
    let from = to - Days::days(6);
    let make = |f: &dyn Fn(BikriListInput) -> BikriListInput| {
        f(BikriListInput {
            from: Some(ddmmyyyy(from)),
            to: Some(ddmmyyyy(to)),
            group_by_day: true,
            ..input()
        })
    };
    let variants: Vec<(&str, BikriListInput)> = vec![
        ("default", make(&|i| i)),
        ("all", make(&|i| BikriListInput { all: true, ..i })),
        (
            "show finished",
            make(&|i| BikriListInput {
                show_finished: true,
                ..i
            }),
        ),
        (
            "seller",
            make(&|i| BikriListInput {
                seller_id: Some(w.rahim),
                ..i
            }),
        ),
        (
            "item",
            make(&|i| BikriListInput {
                item_id: Some(w.banana),
                ..i
            }),
        ),
        (
            "truck text",
            make(&|i| BikriListInput {
                search: Some("les".into()),
                ..i
            }),
        ),
        (
            "all with filters",
            make(&|i| BikriListInput {
                all: true,
                search: Some("zzz".into()),
                seller_id: Some(-1),
                ..i
            }),
        ),
        (
            "cancelled only",
            make(&|i| BikriListInput {
                status: Some(BikriStatus::Cancelled),
                ..i
            }),
        ),
    ];
    let default_all = ok(ctx, &variants[1].1);
    for (what, i) in &variants {
        for page in [1_u32, 7, 25, 100] {
            let (ids, gross_rows, first) = walk(ctx, i, page);
            assert_eq!(
                ids.len() as i64,
                first.total_count,
                "{what} page {page}: rows walked"
            );
            assert_eq!(
                first.days.iter().map(|d| d.count).sum::<i64>(),
                first.total_count,
                "{what}: day counts"
            );
            assert_eq!(
                first
                    .days
                    .iter()
                    .map(|d| d.gross_paisa.as_paisa())
                    .sum::<i64>(),
                first.totals.gross.as_paisa(),
                "{what}: day gross"
            );
            assert_eq!(
                gross_rows,
                first.totals.gross.as_paisa(),
                "{what} page {page}: gross of the rows"
            );
        }
    }
    // `all` sets every filter aside: the same window with nonsense filters gives the same answer.
    let all_filtered = ok(ctx, &variants[6].1);
    assert_eq!(all_filtered.total_count, default_all.total_count);
    assert_eq!(all_filtered.totals, default_all.totals);
    assert_eq!(all_filtered.days, default_all.days);
    // Cancelled بکری are in `all`, never in the sums.
    let sql_all = sql_of(&w, from, to, true);
    assert_eq!(default_all.total_count, sql_all.count);
    assert_eq!(default_all.totals.gross.as_paisa(), sql_all.gross);
    let default = ok(ctx, &variants[0].1);
    assert!(
        default.total_count < default_all.total_count,
        "the cancelled ones are left out of the default"
    );
    assert_eq!(
        default.totals, default_all.totals,
        "a cancelled بکری is in no sum"
    );
}

/// The busiest window NFR-SCL-2 allows: 500 بکری a day for 7 days. Exact against plain SQL, and one page
/// of 25 with the totals and the day headings is quick (in a release build, on CPU time scaled to the
/// reference machine, held to the 300 ms of an arrow move).
#[test]
fn a_window_of_seven_busy_days_is_exact_and_quick() {
    let w = world();
    let mut number = 1;
    let conn = raw(&w.dir);
    conn.execute_batch("BEGIN").unwrap();
    for back in 0..7_i64 {
        for k in 0..500_i64 {
            let status = if k % 50 == 0 { "cancelled" } else { "saved" };
            put_on(
                &conn,
                &w,
                number,
                today() - Days::days(back + 1),
                status,
                1_000 + k * 10,
                Some("BUSY-1"),
            );
            number += 1;
        }
    }
    conn.execute_batch("COMMIT").unwrap();
    drop(conn);
    let ctx = &*w.env.ctx;
    let to = today() - Days::days(1);
    let from = to - Days::days(6);
    for all in [false, true] {
        check_against_sql(&w, ctx, from, to, all, &format!("busy window, all {all}"));
    }
    let f = || {
        ok(ctx, &window(to, 25, 0));
    };
    f();
    let started = cpu_seconds();
    for _ in 0..10 {
        f();
    }
    let cpu = (cpu_seconds() - started) * 100.0;
    let wall = {
        let s = Instant::now();
        f();
        s.elapsed()
    };
    println!(
        "PERF past sales: a busy window (3,500 بکری): CPU {cpu:.0} ms (about {:.0} ms on the reference machine), one call {:.0} ms wall, build {}",
        cpu * 3_752.0 / 2_268.0,
        ms(wall),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    if !cfg!(debug_assertions) {
        assert!(
            cpu * 3_752.0 / 2_268.0 <= 300.0,
            "a busy window costs {cpu:.0} ms of CPU"
        );
    }
}

/// After a real save, edit and cancel the list shows the stored numbers at once (plan 72 P10): the row, the
/// day heading, the totals. The screen keeps no figure; this is what it reads again.
#[test]
fn after_a_real_edit_and_cancel_the_list_shows_the_new_numbers() {
    let w = world();
    let saved = helpers::save_ok(&w, &event1(&w));
    let ctx = &*w.env.ctx;
    let today_window = || ok(ctx, &window(today(), 25, 0));
    let before = today_window();
    assert_eq!(before.total_count, 1);
    assert_eq!(before.totals.gross.rupees(), 29_400);
    assert_eq!(before.days[0].gross_paisa.rupees(), 29_400);
    assert_eq!(before.rows[0].gross.rupees(), 29_400);
    // E2E-13: اکرم's rate 600 to 610: gross 29,600 in the row, the day heading and the totals.
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "610".into();
    tools::edit(&w, &opened, &d);
    let after = today_window();
    assert_eq!(after.total_count, 1);
    assert_eq!(after.rows[0].id, saved.bikri_id);
    assert_eq!(after.rows[0].gross.rupees(), 29_600, "the row");
    assert_eq!(
        after.days[0].gross_paisa.rupees(),
        29_600,
        "the day heading"
    );
    assert_eq!(after.totals.gross.rupees(), 29_600, "the window total");
    assert_eq!(
        after.rows[0].gross,
        bikri::get(ctx, saved.bikri_id).unwrap().lots[0].gross,
        "the row is the stored value of the بکری"
    );
    assert!(before.rows[0].flags.edited.is_none(), "never edited yet");
    assert!(
        after.rows[0].flags.edited.is_some(),
        "the edited mark (FR-7.71)"
    );
    // Cancel: out of the default list, in `all` with its status, out of every sum.
    tools::cancel(&w, saved.bikri_id, true).unwrap();
    let default = today_window();
    assert_eq!(default.total_count, 0);
    assert_eq!(default.totals.gross.rupees(), 0);
    assert_eq!(default.days[0].count, 0);
    let all = ok(
        ctx,
        &BikriListInput {
            all: true,
            ..window(today(), 25, 0)
        },
    );
    assert_eq!(all.total_count, 1);
    assert_eq!(all.rows[0].status, BikriStatus::Cancelled);
    assert_eq!(
        all.totals.gross.rupees(),
        0,
        "a cancelled بکری is in no sum"
    );
    assert_eq!(all.days[0].count, 1);
    assert_eq!(all.days[0].gross_paisa.rupees(), 0);
}

// ---- Volume --------------------------------------------------------------------------------------------

fn on() -> bool {
    if std::env::var("MANDI_PERF_VOLUME").is_err() {
        println!("MANDI_PERF_VOLUME is not set: the volume checks of the past sales are skipped");
        return false;
    }
    true
}

fn volume_env() -> (
    mandi_services::testing::TestEnv,
    robustness_past::PastVolume,
) {
    let v = volume::ensure_volume();
    let past = robustness_past::ensure(&v.dir);
    let (env, _actor) = robustness_past::open_env(&past.dir);
    (env, past)
}

#[allow(clippy::type_complexity)]
fn sql_range(
    dir: &std::path::Path,
    from: NaiveDate,
    to: NaiveDate,
) -> (Vec<(String, i64, i64)>, i64, i64, i64, i64, i64) {
    let (all, saved, gross, net, buyer) = robustness_past::expected(dir, &iso(from), &iso(to));
    let mut days = robustness_past::expected_days(dir, &iso(from), &iso(to));
    // Fill the days with no بکری (those before the first day of the data).
    let mut d = to;
    let mut full = Vec::new();
    while d >= from {
        match days.iter().position(|x| x.0 == iso(d)) {
            Some(i) => full.push(days.remove(i)),
            None => full.push((iso(d), 0, 0)),
        }
        d -= Days::days(1);
    }
    (full, all, saved, gross, net, buyer)
}

#[test]
fn at_ten_years_the_windows_equal_plain_sql_and_pages_add_up() {
    if !on() {
        return;
    }
    let (env, past) = volume_env();
    let ctx = &*env.ctx;
    println!(
        "PAST DATA: {} بکری, {} lots, {} sale lines",
        past.entries, past.lots, past.lines
    );
    let d = |y, m, dd| NaiveDate::from_ymd_opt(y, m, dd).unwrap();
    let ends = [
        today() - Days::days(1),
        today() - Days::days(8),
        d(2021, 10, 10),
        d(2024, 2, 29),
        d(2024, 3, 6),
        d(2025, 12, 31),
        d(2026, 1, 6),
        d(2019, 4, 30),
        d(2016, 10, 17),
        d(2016, 10, 11),
        d(2016, 10, 8),
    ];
    for end in ends {
        let start = end - Days::days(6);
        let (days, all, saved, gross, net, buyer) = sql_range(&past.dir, start, end);
        for show_all in [false, true] {
            let what = format!("window {start} to {end}, all {show_all}");
            let base = BikriListInput {
                from: Some(ddmmyyyy(start)),
                to: Some(ddmmyyyy(end)),
                group_by_day: true,
                all: show_all,
                ..input()
            };
            let (ids, gross_rows, first) = walk(ctx, &base, 100);
            assert_eq!(
                first.total_count,
                if show_all { all } else { saved },
                "{what}: count"
            );
            assert_eq!(ids.len() as i64, first.total_count, "{what}: rows walked");
            assert_eq!(first.totals.gross.as_paisa(), gross, "{what}: gross");
            assert_eq!(first.totals.net.as_paisa(), net, "{what}: net");
            assert_eq!(
                first.totals.buyer_total.as_paisa(),
                buyer,
                "{what}: buyer total"
            );
            assert_eq!(gross_rows, gross, "{what}: gross of the rows");
            assert_eq!(first.days.len(), 7, "{what}: days");
            for (got, want) in first.days.iter().zip(days.iter()) {
                assert_eq!(got.date, want.0, "{what}");
                if show_all {
                    assert_eq!(got.count, want.1, "{what}: day {}", got.date);
                }
                assert_eq!(
                    got.gross_paisa.as_paisa(),
                    want.2,
                    "{what}: day gross {}",
                    got.date
                );
            }
        }
    }
    // Today's section: the 40 بکری of the last day, 2% of them cancelled.
    let section = ok(ctx, &day_of(today(), 100));
    assert!(
        (30..=40).contains(&section.total_count),
        "{}",
        section.total_count
    );
    // Filters: a seller and an item that have بکری; count and totals are of those.
    let filtered = BikriListInput {
        from: Some("11-10-2016".into()),
        to: Some(ddmmyyyy(today())),
        seller_id: Some(past.seller_id),
        item_id: Some(past.item_id),
        ..input()
    };
    let (ids, gross_rows, first) = walk(ctx, &filtered, 100);
    assert!(first.total_count > 0);
    assert_eq!(ids.len() as i64, first.total_count);
    assert_eq!(gross_rows, first.totals.gross.as_paisa());
}

fn p95(mut v: Vec<Duration>) -> Duration {
    v.sort();
    let i = ((v.len() as f64) * 0.95).ceil() as usize;
    v[i.saturating_sub(1).min(v.len() - 1)]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1_000.0
}

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

#[test]
fn at_ten_years_the_95th_percentile_against_nfr_spd_24() {
    if !on() {
        return;
    }
    let (env, past) = volume_env();
    let ctx = &*env.ctx;
    let ratio = 3_752.0 / 2_268.0;
    let yesterday = today() - Days::days(1);
    let page = 25;
    let section = |extra: &BikriListInput| BikriListInput {
        date: Some(ddmmyyyy(today())),
        limit: page,
        ..extra.clone()
    };
    let win = |end: NaiveDate, extra: &BikriListInput| BikriListInput {
        from: Some(ddmmyyyy(end - Days::days(6))),
        to: Some(ddmmyyyy(end)),
        group_by_day: true,
        limit: page,
        ..extra.clone()
    };
    let plain = input();
    let all = BikriListInput {
        all: true,
        ..input()
    };
    let seller = BikriListInput {
        seller_id: Some(past.seller_id),
        ..input()
    };
    let item = BikriListInput {
        item_id: Some(past.item_id),
        ..input()
    };
    let both = BikriListInput {
        seller_id: Some(past.seller_id),
        item_id: Some(past.item_id),
        search: Some("LE1".into()),
        show_finished: true,
        ..input()
    };
    let five_years_back = NaiveDate::from_ymd_opt(2021, 10, 10).unwrap();
    let first_week = NaiveDate::from_ymd_opt(2016, 10, 17).unwrap();
    // One screen action is the calls it needs: today's section and the window with its days.
    type Action<'a> = Box<dyn Fn() + 'a>;
    let actions: Vec<(&str, f64, Action<'_>)> = vec![
        (
            "open (today's section + the window ending yesterday)",
            1_000.0,
            Box::new(|| {
                ok(ctx, &section(&plain));
                ok(ctx, &win(yesterday, &plain));
            }),
        ),
        (
            "open with `all`",
            1_000.0,
            Box::new(|| {
                ok(ctx, &section(&all));
                ok(ctx, &win(yesterday, &all));
            }),
        ),
        (
            "open with a seller filter",
            1_000.0,
            Box::new(|| {
                ok(ctx, &section(&seller));
                ok(ctx, &win(yesterday, &seller));
            }),
        ),
        (
            "open with an item filter",
            1_000.0,
            Box::new(|| {
                ok(ctx, &section(&item));
                ok(ctx, &win(yesterday, &item));
            }),
        ),
        (
            "open with seller, item, truck text and finished",
            1_000.0,
            Box::new(|| {
                ok(ctx, &section(&both));
                ok(ctx, &win(yesterday, &both));
            }),
        ),
        (
            "arrow: the next window back",
            300.0,
            Box::new(|| {
                ok(ctx, &win(yesterday - Days::days(7), &plain));
            }),
        ),
        (
            "arrow: three windows back",
            300.0,
            Box::new(|| {
                ok(ctx, &win(yesterday - Days::days(21), &plain));
            }),
        ),
        (
            "arrow with `all`",
            300.0,
            Box::new(|| {
                ok(ctx, &win(yesterday - Days::days(7), &all));
            }),
        ),
        (
            "arrow with a seller filter",
            300.0,
            Box::new(|| {
                ok(ctx, &win(yesterday - Days::days(7), &seller));
            }),
        ),
        (
            "arrow with an item filter",
            300.0,
            Box::new(|| {
                ok(ctx, &win(yesterday - Days::days(7), &item));
            }),
        ),
        (
            "arrow with seller, item, truck text and finished",
            300.0,
            Box::new(|| {
                ok(ctx, &win(yesterday - Days::days(7), &both));
            }),
        ),
        (
            "go to a date 5 years back",
            1_000.0,
            Box::new(|| {
                ok(ctx, &win(five_years_back, &plain));
            }),
        ),
        (
            "go to a date 5 years back with `all`",
            1_000.0,
            Box::new(|| {
                ok(ctx, &win(five_years_back, &all));
            }),
        ),
        (
            "go to a date 5 years back with a seller filter",
            1_000.0,
            Box::new(|| {
                ok(ctx, &win(five_years_back, &seller));
            }),
        ),
        (
            "go to a date 5 years back with seller, item and text",
            1_000.0,
            Box::new(|| {
                ok(ctx, &win(five_years_back, &both));
            }),
        ),
        (
            "go to the first week of the data",
            1_000.0,
            Box::new(|| {
                ok(ctx, &win(first_week, &plain));
            }),
        ),
        (
            "next page of 25 inside a window",
            300.0,
            Box::new(|| {
                ok(
                    ctx,
                    &BikriListInput {
                        offset: 25,
                        ..win(yesterday, &plain)
                    },
                );
            }),
        ),
    ];
    let mut over = Vec::new();
    for (name, target, f) in &actions {
        f(); // warm-up
        let wall = p95((0..15)
            .map(|_| {
                let s = Instant::now();
                f();
                s.elapsed()
            })
            .collect());
        let cpu = (0..3)
            .map(|_| cpu_ms_of(5, f.as_ref()))
            .fold(f64::MAX, f64::min);
        let verdict = if cpu * ratio <= *target { "OK" } else { "OVER" };
        println!(
            "PERF past sales: {name}: wall p95 {:.0} ms, CPU {cpu:.0} ms (about {:.0} ms on the reference machine) vs {target:.0} ms - {verdict}",
            ms(wall),
            cpu * ratio
        );
        if cpu * ratio > *target {
            over.push((*name).to_owned());
        }
    }
    if !cfg!(debug_assertions) {
        assert!(
            over.is_empty(),
            "over the NFR-SPD-24 and plan 72 targets on the reference machine: {over:?}"
        );
    }
}
