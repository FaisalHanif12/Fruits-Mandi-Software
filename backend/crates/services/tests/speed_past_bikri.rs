//! Plan 72 T1 (PRD 12.7 پچھلی بکری, FR-19.11, NFR-SPD-24, Code Standards B25, B31): the screen
//! پچھلی بکری opens in under 1 second, and each move (the next window, a jump 5 years back, the
//! switch "show everything", a search) takes under 300 ms, on ten years of data.
//!
//! The data is the NFR-SCL-1 volume of `robustness` (`benches/support`, built once into
//! `backend/target/perf/nfr_scl1`: 20,000 accounts, 1.03 million posted entries, 3.5 million
//! journal lines). The volume has no بکری, so this test copies it and adds ten years of بکری by
//! plain SQL: 11 بکری on each of 3,650 days (the PRD's pace is far less), 2 lots of 14 sale lines
//! in each, and 500 بکری on today (NFR-SCL-2: the busiest day), about 1.1 million sale lines in all.
//! Today is 12-11-2026.
//!
//! The test is `#[ignore]`d: it needs the big file and a release build, because bundled SQLite
//! built without optimisation is several times slower than the real app. Run it with:
//!
//! ```text
//! cargo test --release -p mandi-services --test speed_past_bikri -- --ignored --nocapture
//! ```
//!
//! It prints one line for each step and fails if a budget is missed. `MANDI_SPEED_REPORT_ONLY=1`
//! prints without failing, to read the numbers on a slow machine. The reference hardware is slower
//! than a developer's machine, so a pass here is a necessary check; the final check is on the
//! reference computer before go-live (NFR Section 2).
#![cfg(test)]

mod common;
#[path = "../benches/support/mod.rs"]
mod volume;

use std::time::{Duration, Instant};

use mandi_services::bikri;
use mandi_services::core::bikri::{BikriList, BikriListInput};
use mandi_services::db::rusqlite::Connection;

/// Opening the screen: today's section and the first window.
const OPEN: Duration = Duration::from_millis(1_000);
/// Each move: the next window, a date jump, the switch, a search.
const MOVE: Duration = Duration::from_millis(300);
/// The rows of a page.
const PAGE: u32 = 25;
/// The days with 11 بکری each, before today.
const DAYS: i64 = 3_650;
/// بکری on the busiest day, today (NFR-SCL-2).
const TODAY_ENTRIES: i64 = 500;

/// Adds `DAYS` days of بکری and the busy day of today to the copy, by plain SQL (the copy is
/// migrated first by opening it as a test environment). Returns a seller who has lots in it.
fn add_bikri(path: &std::path::Path) -> i64 {
    let conn = Connection::open(path).unwrap_or_else(|e| volume::fail(&format!("no data: {e}")));
    conn.execute_batch("PRAGMA synchronous = OFF; PRAGMA temp_store = MEMORY;")
        .unwrap_or_else(|e| volume::fail(&format!("pragma: {e}")));
    let count = |sql: &str| -> i64 {
        conn.query_row(sql, [], |r| r.get(0))
            .unwrap_or_else(|e| volume::fail(&format!("{sql}: {e}")))
    };
    let sellers =
        count("SELECT COUNT(*) FROM accounts WHERE party_type = 'seller' AND is_system = 0");
    let buyers =
        count("SELECT COUNT(*) FROM accounts WHERE party_type = 'buyer' AND is_system = 0");
    let items = count("SELECT COUNT(*) FROM items");
    let packs = count("SELECT COUNT(*) FROM packings");
    let started = Instant::now();
    let sql = format!(
        "
        CREATE TEMP TABLE t_sellers (k INTEGER PRIMARY KEY, id INTEGER NOT NULL);
        INSERT INTO t_sellers (id) SELECT id FROM accounts WHERE party_type = 'seller' AND is_system = 0 ORDER BY id;
        CREATE TEMP TABLE t_buyers (k INTEGER PRIMARY KEY, id INTEGER NOT NULL);
        INSERT INTO t_buyers (id) SELECT id FROM accounts WHERE party_type = 'buyer' AND is_system = 0 ORDER BY id;
        CREATE TEMP TABLE t_items (k INTEGER PRIMARY KEY, id INTEGER NOT NULL);
        INSERT INTO t_items (id) SELECT id FROM items ORDER BY id;
        CREATE TEMP TABLE t_packs (k INTEGER PRIMARY KEY, id INTEGER NOT NULL);
        INSERT INTO t_packs (id) SELECT id FROM packings ORDER BY id;

        -- بکری: 11 on each day, 500 on the last day (today, 2026-11-12). Old ones have a truck; every
        -- 40th is cancelled.
        WITH RECURSIVE d(day) AS (SELECT 0 UNION ALL SELECT day + 1 FROM d WHERE day < {days}),
             k(slot) AS (SELECT 0 UNION ALL SELECT slot + 1 FROM k WHERE slot < {today_entries} - 1)
        INSERT INTO bikri_entries (number, entry_date, bill_date, truck_no, status, cancel_reason)
        SELECT row_number() OVER (ORDER BY day, slot),
               date('2026-11-12', '-' || ({days} - day) || ' days'),
               date('2026-11-12', '-' || ({days} - day) || ' days'),
               CASE WHEN day < 3000 THEN 'TRK-' || (day * 7 + slot) ELSE NULL END,
               CASE WHEN (day + slot) % 40 = 7 THEN 'cancelled' ELSE 'saved' END,
               CASE WHEN (day + slot) % 40 = 7 THEN 'wrong entry' ELSE NULL END
        FROM d JOIN k ON slot < CASE WHEN day = {days} THEN {today_entries} ELSE 11 END;

        -- Two lots in each.
        WITH l(lot_no) AS (SELECT 1 UNION ALL SELECT 2)
        INSERT INTO bikri_lots (bikri_id, lot_no, seller_account_id, item_id, packing_id, qty_received)
        SELECT e.id, l.lot_no,
               (SELECT id FROM t_sellers WHERE k = 1 + ((e.id * 31 + l.lot_no * 17) % {sellers})),
               (SELECT id FROM t_items WHERE k = 1 + ((e.id + l.lot_no * 5) % {items})),
               (SELECT id FROM t_packs WHERE k = 1 + (e.id % {packs})),
               1000
        FROM bikri_entries e JOIN l;

        -- 14 sale lines in each lot: qty 1 to 8, rate 200 to 899 rupees, commission 6%, لاگا 8 a نگ.
        WITH RECURSIVE j(n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM j WHERE n < 14),
             b AS (SELECT l.id AS lot_id, l.bikri_id, e.entry_date, j.n,
                          1 + ((l.id + j.n) % 8) AS qty,
                          200 + ((l.id * 13 + j.n * 7) % 700) AS rate
                   FROM bikri_lots l JOIN bikri_entries e ON e.id = l.bikri_id JOIN j)
        INSERT INTO bikri_sales (bikri_id, lot_id, line_no, journal_entry_id, entry_date,
               buyer_account_id, qty, seller_rate_paisa, seller_amount_paisa, seller_comm_method,
               seller_comm_value_bps, seller_comm_paisa, buyer_rate_paisa, buyer_value_paisa,
               buyer_comm_method, buyer_comm_value_bps, buyer_comm_paisa, laga_method,
               laga_value_paisa, laga_paisa, buyer_total_paisa, rate_diff_paisa, payment_type,
               is_active, paid_paisa)
        SELECT bikri_id, lot_id, n, 1 + (lot_id % 1000000), entry_date,
               (SELECT id FROM t_buyers WHERE k = 1 + ((lot_id * 7 + n * 13) % {buyers})),
               qty, rate * 100, qty * rate * 100, 'percent',
               600, (qty * rate * 6 + 50) / 100 * 100, rate * 100, qty * rate * 100,
               'percent', 600, (qty * rate * 6 + 50) / 100 * 100, 'per_unit',
               800, qty * 800, qty * rate * 100 + (qty * rate * 6 + 50) / 100 * 100 + qty * 800, 0,
               'credit', 1, 0
        FROM b;

        -- A line of a cancelled بکری is not in force.
        UPDATE bikri_sales SET is_active = 0, ended = 'cancelled'
        WHERE bikri_id IN (SELECT id FROM bikri_entries WHERE status = 'cancelled');

        -- The lot sums are the sums of its lines.
        UPDATE bikri_lots SET qty_received = a.q, qty_sold = a.q, gross_paisa = a.g,
               seller_comm_paisa = a.c, deductions_paisa = a.c, net_paisa = a.g - a.c
        FROM (SELECT lot_id, SUM(qty) AS q, SUM(seller_amount_paisa) AS g, SUM(seller_comm_paisa) AS c
              FROM bikri_sales GROUP BY lot_id) a
        WHERE bikri_lots.id = a.lot_id;
        ",
        days = DAYS,
        today_entries = TODAY_ENTRIES,
        sellers = sellers,
        buyers = buyers,
        items = items,
        packs = packs,
    );
    conn.execute_batch(&sql)
        .unwrap_or_else(|e| volume::fail(&format!("cannot add the بکری: {e}")));
    let entries = count("SELECT COUNT(*) FROM bikri_entries");
    let lots = count("SELECT COUNT(*) FROM bikri_lots");
    let sales = count("SELECT COUNT(*) FROM bikri_sales");
    volume::note(&format!(
        "past bikri: {entries} بکری, {lots} lots, {sales} sale lines added in {:.1} s",
        started.elapsed().as_secs_f64()
    ));
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").ok();
    count("SELECT seller_account_id FROM bikri_lots WHERE bikri_id = 2000 AND lot_no = 1")
}

/// One measured step: its name, the first call and the best and the median of the next five.
struct Line {
    name: String,
    budget: Duration,
    cold: Duration,
    best: Duration,
    median: Duration,
}

fn ms(duration: Duration) -> String {
    format!("{:.1}", duration.as_secs_f64() * 1_000.0)
}

/// Times a step: the first call cold, then five warm ones.
fn measure(name: &str, budget: Duration, mut call: impl FnMut() -> BikriList) -> Line {
    mandi_services::db::memo::clear();
    let started = Instant::now();
    drop(call());
    let cold = started.elapsed();
    let mut warm: Vec<Duration> = (0..5)
        .map(|_| {
            let started = Instant::now();
            drop(call());
            started.elapsed()
        })
        .collect();
    warm.sort();
    Line {
        name: name.to_owned(),
        budget,
        cold,
        best: warm.first().copied().unwrap_or_default(),
        median: warm.get(2).copied().unwrap_or_default(),
    }
}

#[test]
#[ignore = "needs the 10-year volume and a release build: see the module doc"]
fn the_past_sales_screen_is_fast_on_ten_years_of_data() {
    let big = volume::ensure_volume();
    volume::describe(&big);
    let copy = volume::copy_of_volume(&big);
    // Opening the copy migrates it to the newest schema; the بکری are added after that.
    let (env, _actor) = volume::open(copy.path());
    common::set_today(&env, 2026, 11, 12);
    let seller = add_bikri(&copy.path().join("data").join("mandi.db"));
    let ctx = &env.ctx;

    let list = |input: BikriListInput| bikri::list(ctx, &input).expect("the list");
    let window = |from: &str, to: &str| BikriListInput {
        from: Some(from.into()),
        to: Some(to.into()),
        group_by_day: true,
        limit: PAGE,
        ..BikriListInput::default()
    };
    let mut lines: Vec<Line> = Vec::new();
    // Opening: today's section (500 بکری, 25 shown) and the first window.
    lines.push(measure("open: today's section", OPEN, || {
        list(window("12-11-2026", "12-11-2026"))
    }));
    lines.push(measure("open: first window 05-11 to 11-11", OPEN, || {
        list(window("05-11-2026", "11-11-2026"))
    }));
    // Moves.
    lines.push(measure("move: one step back 29-10 to 04-11", MOVE, || {
        list(window("29-10-2026", "04-11-2026"))
    }));
    lines.push(measure("move: a jump 5 years back", MOVE, || {
        list(window("08-11-2021", "14-11-2021"))
    }));
    lines.push(measure(
        "move: next page of the window (offset 25)",
        MOVE,
        || {
            list(BikriListInput {
                offset: PAGE,
                ..window("05-11-2026", "11-11-2026")
            })
        },
    ));
    lines.push(measure(
        "move: the switch 'show everything', window",
        MOVE,
        || {
            list(BikriListInput {
                all: true,
                ..window("05-11-2026", "11-11-2026")
            })
        },
    ));
    lines.push(measure(
        "move: the switch 'show everything', today",
        MOVE,
        || {
            list(BikriListInput {
                all: true,
                ..window("12-11-2026", "12-11-2026")
            })
        },
    ));
    lines.push(measure(
        "move: today's last page (offset 475)",
        MOVE,
        || {
            list(BikriListInput {
                offset: 475,
                ..window("12-11-2026", "12-11-2026")
            })
        },
    ));
    // Filters over a window.
    lines.push(measure(
        "filter: a seller over a window of 7 days",
        MOVE,
        || {
            list(BikriListInput {
                seller_id: Some(seller),
                ..window("05-11-2026", "11-11-2026")
            })
        },
    ));
    lines.push(measure(
        "filter: a number search over a window",
        MOVE,
        || {
            list(BikriListInput {
                search: Some("40000".into()),
                ..window("05-11-2026", "11-11-2026")
            })
        },
    ));
    lines.push(measure(
        "filter: a sort by gross over today (500)",
        MOVE,
        || {
            list(BikriListInput {
                sort: Some(mandi_services::core::paging::SortInput {
                    language: None,
                    column: "gross".into(),
                    direction: mandi_services::core::paging::SortDirection::Desc,
                }),
                ..window("12-11-2026", "12-11-2026")
            })
        },
    ));

    // The steps above read real rows: today has 500 بکری (13 of them cancelled), a window of 7 days
    // has 11 a day, and "show everything" counts the cancelled ones too.
    let today = list(window("12-11-2026", "12-11-2026"));
    assert_eq!(today.rows.len(), PAGE as usize);
    assert_eq!(today.total_count, TODAY_ENTRIES - 13);
    assert_eq!(
        today.days.first().map(|d| d.count),
        Some(TODAY_ENTRIES - 13)
    );
    let everything = list(BikriListInput {
        all: true,
        ..window("12-11-2026", "12-11-2026")
    });
    assert_eq!(everything.total_count, TODAY_ENTRIES);
    let week = list(BikriListInput {
        all: true,
        ..window("05-11-2026", "11-11-2026")
    });
    assert_eq!((week.total_count, week.days.len()), (77, 7));
    assert!(week.days.iter().all(|d| d.count == 11));
    let years_back = list(window("08-11-2021", "14-11-2021"));
    assert_eq!(years_back.days.len(), 7);
    assert!(years_back.total_count > 60);

    volume::note("past bikri speed (ms): cold, best of 5 warm, median of 5 warm, budget");
    let mut missed: Vec<String> = Vec::new();
    for line in &lines {
        let worst = line.cold.max(line.median);
        let verdict = if worst <= line.budget { "ok" } else { "OVER" };
        volume::note(&format!(
            "  {:<52} cold {:>7} best {:>7} median {:>7} budget {:>6} {verdict}",
            line.name,
            ms(line.cold),
            ms(line.best),
            ms(line.median),
            ms(line.budget)
        ));
        if worst > line.budget {
            missed.push(line.name.clone());
        }
    }
    let open_time = lines
        .iter()
        .take(2)
        .map(|l| l.cold)
        .fold(Duration::ZERO, |a, b| a + b);
    volume::note(&format!(
        "  open (today's section + first window, cold): {} ms, budget {} ms",
        ms(open_time),
        ms(OPEN)
    ));
    if std::env::var("MANDI_SPEED_REPORT_ONLY").is_err() {
        assert!(missed.is_empty(), "over budget: {missed:?}");
        assert!(open_time <= OPEN, "opening took {}", ms(open_time));
    }
}
