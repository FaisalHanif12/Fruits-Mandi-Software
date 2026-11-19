//! Plan 80 gate, the speed fix loop (backend-f): writes the full output of every read that was sped
//! up to files, so the output before and after a change can be compared byte for byte ("no figure
//! changes", Umar). Every test is `#[ignore]`d: it needs the 10-year data (about 3 GB) and a release
//! build.
//!
//! ```text
//! export CARGO_TARGET_DIR="$PWD/backend/target-fix-f" CARGO_INCREMENTAL=0
//! export MANDI_PERF_DIR="$CARGO_TARGET_DIR/perf/nfr_scl1"      # the plain volume, as the gate tests
//! export FIX_F_OUT="$PWD/backend/target-fix-f/compare/before"   # where the files go
//! cargo test --release -p mandi-services --test speed_fix_compare -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The data is `<MANDI_PERF_DIR>/../nfr_scl1_gate`, or the folder in `FIX_F_DATA`. Today is the
//! fixed day of the test clock (10-10-2026), so two runs read the same books. `FIX_F_ONLY=R35,statement`
//! writes only the outputs whose name contains one of the texts. The same file runs on the small
//! golden-sized data of the unit tests through `FIX_F_DATA`.
#![cfg(test)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use mandi_reports::analytics::{
    AnalyticsFilters, AnalyticsGroupBy, AnalyticsRunInput, AnalyticsView,
};
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_reports::reports::{
    self, FilterKind, PartyFilter, ReportFilters, ReportId, ReportRunInput, StatementRowsInput,
    statement_rows,
};
use mandi_services::db::audit::{AuditKey, MemoryKeyStore};
use mandi_services::db::clock::ManualClock;
use mandi_services::db::disk::FixedDiskProbe;
use mandi_services::db::rusqlite::{Connection, OpenFlags};
use mandi_services::db::{Db, DbOptions};
use mandi_services::testing::test_start;

const BUSY_DAY: &str = "10-10-2026";
const ORDINARY_DAY: &str = "15-09-2026";

fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("FIX_F_DATA") {
        return PathBuf::from(dir);
    }
    let base = std::env::var("MANDI_PERF_DIR").expect("set MANDI_PERF_DIR or FIX_F_DATA");
    PathBuf::from(base).with_file_name("nfr_scl1_gate")
}

fn out_dir() -> PathBuf {
    let dir = PathBuf::from(std::env::var("FIX_F_OUT").expect("set FIX_F_OUT"));
    std::fs::create_dir_all(&dir).expect("the output folder");
    dir
}

/// Opens the data without the start-up check (it takes 20 seconds and checks nothing the reads
/// need): the reads work on any state.
fn open() -> Db {
    let dir = data_dir();
    let db = Db::open(
        &dir.join("data").join("mandi.db"),
        DbOptions {
            backups_dir: dir.join("backups"),
            disk_probe: Arc::new(FixedDiskProbe(10 << 30)),
            clock: Arc::new(ManualClock::new(test_start())),
            key_store: Arc::new(MemoryKeyStore::new(AuditKey::from_bytes([7; 32]))),
            busy_timeout: Duration::from_secs(5),
            check_wait: Duration::from_millis(500),
        },
    )
    .expect("the data opens");
    db.run_check_with(|| Ok(()));
    db
}

fn raw() -> Connection {
    Connection::open_with_flags(
        data_dir().join("data").join("mandi.db"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .expect("the data opens read only")
}

fn id_where(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).expect(sql)
}

fn wanted(name: &str) -> bool {
    match std::env::var("FIX_F_ONLY") {
        Ok(list) if !list.trim().is_empty() => list.split(',').any(|p| name.contains(p.trim())),
        _ => true,
    }
}

/// Writes one output: the JSON of the whole result, or of the error.
fn put<T: serde::Serialize, E: std::fmt::Debug>(name: &str, result: Result<T, E>) {
    if !wanted(name) {
        return;
    }
    let text = match result {
        Ok(value) => serde_json::to_string_pretty(&value).expect("json"),
        Err(error) => format!("ERROR {error:?}"),
    };
    std::fs::write(out_dir().join(format!("{name}.json")), &text).expect("write");
    println!("wrote {name} ({} bytes)", text.len());
}

/// Times `call` once and writes its output; the time is only a note.
fn dump<T: serde::Serialize, E: std::fmt::Debug>(name: &str, call: impl FnOnce() -> Result<T, E>) {
    if !wanted(name) {
        return;
    }
    let started = Instant::now();
    let result = call();
    println!("{name}: {:?}", started.elapsed());
    put(name, result);
}

/// As [`dump`] for a result that has no JSON, only `Debug`.
fn dump_debug<T: std::fmt::Debug, E: std::fmt::Debug>(
    name: &str,
    call: impl FnOnce() -> Result<T, E>,
) {
    if !wanted(name) {
        return;
    }
    let started = Instant::now();
    let result = call();
    println!("{name}: {:?}", started.elapsed());
    let text = format!("{result:#?}");
    std::fs::write(out_dir().join(format!("{name}.txt")), &text).expect("write");
}

fn filters_for(
    kinds: &[FilterKind],
    from: &str,
    to: &str,
    party: Option<i64>,
    bikri: i64,
    partner: i64,
) -> ReportFilters {
    let mut f = ReportFilters::default();
    for kind in kinds {
        match kind {
            FilterKind::DateRange => {
                f.from = from.to_owned();
                f.to = to.to_owned();
            }
            FilterKind::AsOf => f.as_of = to.to_owned(),
            FilterKind::Bikri => f.bikri_id = Some(bikri),
            FilterKind::Partner => f.partner_id = Some(partner),
            _ => {}
        }
    }
    if let Some(account_id) = party {
        f.party = Some(PartyFilter {
            account_id,
            person_key: None,
        });
    }
    f
}

/// The reports that were sped up, on the busy day, an ordinary day and ten years, at several pages.
#[test]
#[ignore = "needs the full volume and a release build: see the module doc"]
fn fix_f_00_reports() {
    let db = open();
    let conn = raw();
    let busy_buyer = id_where(&conn, "SELECT id FROM accounts WHERE code = '0020'");
    let busy_bikri = id_where(
        &conn,
        "SELECT MIN(id) FROM bikri_entries WHERE entry_date = '2026-10-10'",
    );
    let partner = id_where(&conn, "SELECT MIN(id) FROM partners");
    let ids = [
        ReportId::R2,
        ReportId::R3,
        ReportId::R4,
        ReportId::R7,
        ReportId::R11,
        ReportId::R16,
        ReportId::R18,
        ReportId::R20,
        ReportId::R21,
        ReportId::R22,
        ReportId::R24,
        ReportId::R28,
        ReportId::R34,
        ReportId::R35,
    ];
    for entry in reports::catalog() {
        if !ids.contains(&entry.id) {
            continue;
        }
        let code = format!("{:?}", entry.id);
        for (name, from, to) in [
            ("busy", BUSY_DAY, BUSY_DAY),
            ("ordinary", ORDINARY_DAY, ORDINARY_DAY),
            ("ten", "11-10-2016", BUSY_DAY),
        ] {
            if matches!(entry.id, ReportId::R34 | ReportId::R35) && name == "ten" {
                continue;
            }
            if entry.filters.contains(&FilterKind::AsOf)
                && !entry.filters.contains(&FilterKind::DateRange)
                && name != "busy"
            {
                continue;
            }
            let party = entry.party_required.then_some(busy_buyer);
            let filters = filters_for(&entry.filters, from, to, party, busy_bikri, partner);
            for offset in [0_i64, 100, 2_500, 100_000] {
                let label = format!("report_{code}_{name}_{offset}");
                if !wanted(&label) {
                    continue;
                }
                dump(&label, || {
                    reports::run(
                        &db,
                        &ReportRunInput {
                            id: entry.id,
                            filters: filters.clone(),
                            offset,
                            limit: 100,
                            sort: None,
                            group_by_month: false,
                        },
                    )
                });
            }
        }
    }
}

/// The statement rows of the busiest buyer and the busiest seller, every shape of the page.
#[test]
#[ignore = "needs the full volume and a release build: see the module doc"]
fn fix_f_01_statement_rows() {
    let db = open();
    let conn = raw();
    let buyer = id_where(&conn, "SELECT id FROM accounts WHERE code = '0020'");
    let seller = id_where(&conn, "SELECT id FROM accounts WHERE code = '0010'");
    let median = id_where(&conn, "SELECT id FROM accounts WHERE code = '0024499'");
    for (who, account) in [("buyer", buyer), ("seller", seller), ("median", median)] {
        for (history, newest) in [(false, false), (false, true), (true, true)] {
            for (from, to) in [
                (None, None),
                (Some("01-01-2024"), Some("31-12-2024")),
                (Some("15-03-2023"), None),
            ] {
                for offset in [0_u32, 100, 5_000] {
                    let label = format!(
                        "statement_{who}_h{}_n{}_{}_{}_{offset}",
                        u8::from(history),
                        u8::from(newest),
                        from.unwrap_or("all"),
                        to.unwrap_or("all")
                    );
                    if !wanted(&label) {
                        continue;
                    }
                    dump(&label, || {
                        statement_rows(
                            &db,
                            &StatementRowsInput {
                                account_id: account,
                                from: from.map(str::to_owned),
                                to: to.map(str::to_owned),
                                include_history: history,
                                newest_first: newest,
                                offset,
                                limit: 100,
                            },
                        )
                    });
                }
            }
        }
    }
}

/// The dashboard, the attention list, the tiles and the lists behind them.
#[test]
#[ignore = "needs the full volume and a release build: see the module doc"]
fn fix_f_02_dashboard() {
    let db = open();
    let backup = || BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    };
    dump("dashboard_get", || dashboard::get(&db, backup()));
    dump("dashboard_attention", || dashboard::attention(&db, backup()));
    dump("dashboard_total_dues", || dashboard::total_dues_parts(&db));
    let today = mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10).expect("a date");
    dump_debug("dashboard_tiles_now", || dashboard::tiles_now(&db));
    dump_debug("dashboard_buyers_dues", || dashboard::buyers_dues(&db, today));
    dump_debug("dashboard_sellers", || dashboard::sellers(&db, today));
}

/// The analytics views that were sped up, over five and ten years.
#[test]
#[ignore = "needs the full volume and a release build: see the module doc"]
fn fix_f_03_analytics() {
    let db = open();
    let range = |from: &str| AnalyticsFilters {
        from: from.into(),
        to: BUSY_DAY.into(),
        ..AnalyticsFilters::default()
    };
    let views = [
        AnalyticsView::A,
        AnalyticsView::B,
        AnalyticsView::C,
        AnalyticsView::D,
        AnalyticsView::E,
        AnalyticsView::G,
        AnalyticsView::H,
        AnalyticsView::L,
    ];
    for view in views {
        for (label, from, group) in [
            ("5y", "11-10-2021", None),
            ("10y", "11-10-2016", None),
            ("5y_month", "11-10-2021", Some(AnalyticsGroupBy::Month)),
            ("10y_month", "11-10-2016", Some(AnalyticsGroupBy::Month)),
            ("10y_year", "11-10-2016", Some(AnalyticsGroupBy::Year)),
        ] {
            let mut filters = range(from);
            filters.group_by = group;
            for offset in [0_i64, 100] {
                let name = format!("analytics_{view:?}_{label}_{offset}");
                if !wanted(&name) {
                    continue;
                }
                dump(&name, || {
                    mandi_reports::analytics::run(
                        &db,
                        &AnalyticsRunInput {
                            view,
                            filters: filters.clone(),
                            offset,
                            limit: 100,
                            sort: None,
                        },
                    )
                });
            }
        }
    }
}

#[allow(dead_code)]
fn unused(_: &Path) {}
