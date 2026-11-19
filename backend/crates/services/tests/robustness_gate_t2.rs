//! Plan 80, lane T2 (volume and speed): the NFR-SCL-1 dataset with its 1.2 million sale lines, and
//! a p95 for every NFR-SPD target that a Mac can measure. Owner: `robustness`.
//!
//! Every test here is `#[ignore]`d: it needs a big data file (about 3 GB) and a release build,
//! because bundled SQLite built without optimisation is several times slower than the real app.
//! Run through `node scripts/perf/gate-t2-run.mjs`, which sets the folders, runs the tests one at
//! a time and prints the table. By hand:
//!
//! ```text
//! export CARGO_TARGET_DIR="$PWD/backend/target-gate-t2" CARGO_INCREMENTAL=0
//! export MANDI_PERF_DIR="$CARGO_TARGET_DIR/perf/nfr_scl1"
//! cargo test --release -p mandi-services --test robustness_gate_t2 -- --ignored --nocapture --test-threads=1
//! ```
//!
//! `MANDI_SPEED_REPORT_ONLY=1` prints and records every row without failing on a row over target.
//! `GATE_T2_BIKRI=400` builds a tiny sale layer in `nfr_scl1_gate_dev`, to try the generator.
#![cfg(test)]

#[path = "../benches/bikri_draft/mod.rs"]
#[allow(dead_code)]
mod bikri_draft;
#[allow(dead_code)]
#[path = "robustness_gate_t2/harness.rs"]
mod harness;
#[path = "../benches/support/mod.rs"]
mod support;
#[allow(dead_code)]
#[path = "robustness_gate_t2/volume.rs"]
mod volume;

use mandi_services::auth;
use mandi_services::db::rusqlite::{Connection, OpenFlags};
use mandi_services::testing::TestEnv;

/// What every timing test starts with: the data, opened, with the Owner signed in.
struct World {
    gate: volume::Gate,
    env: TestEnv,
    actor: auth::Actor,
}

fn world() -> World {
    let gate = volume::gate_volume();
    let (env, actor) = support::open(&gate.dir);
    World { gate, env, actor }
}

fn raw(gate: &volume::Gate) -> Connection {
    Connection::open_with_flags(
        gate.dir.join("data").join("mandi.db"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap_or_else(|e| support::fail(&format!("cannot open the data read only: {e}")))
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap_or(-1)
}

#[test]
#[ignore = "builds the 670 MB base volume of NFR-SCL-1 and the sale layer: see the module doc"]
fn t2_00_build_volume() {
    let gate = volume::gate_volume();
    let conn = raw(&gate);
    let full = std::env::var("GATE_T2_BIKRI").is_err();
    let sales = count(&conn, "SELECT COUNT(*) FROM bikri_sales");
    let facts = count(&conn, "SELECT COUNT(*) FROM analytics_facts");
    let lines = count(&conn, "SELECT COUNT(*) FROM journal_lines");
    let entries = count(
        &conn,
        "SELECT COUNT(*) FROM journal_entries WHERE status = 'posted'",
    );
    let accounts = count(&conn, "SELECT COUNT(*) FROM accounts");
    let drafts = count(
        &conn,
        "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'",
    );
    let busy_bikri = count(
        &conn,
        "SELECT COUNT(*) FROM bikri_entries WHERE entry_date = '2026-10-10'",
    );
    let busy_lines = count(
        &conn,
        "SELECT COUNT(*) FROM bikri_sales WHERE entry_date = '2026-10-10'",
    );
    let (dr, cr): (i64, i64) = conn
        .query_row(
            "SELECT SUM(dr_paisa), SUM(cr_paisa) FROM journal_lines",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap_or((0, -1));
    harness::fact(
        "volume",
        &format!(
            "{accounts} accounts, {entries} posted entries, {lines} journal lines, {sales} sale lines, {facts} analytics facts, busiest day {busy_bikri} بکری and {busy_lines} lines, debits {dr} = credits {cr}"
        ),
    );
    assert_eq!(drafts, 0, "no entry is left as a draft");
    assert_eq!(dr, cr, "the books balance over the whole file");
    assert_eq!(sales, facts, "one fact for every sale line");
    if full {
        assert_eq!(sales, 1_200_000, "NFR-SCL-1: 1,200,000 sale lines");
        assert!(
            lines >= 3_500_000,
            "NFR-SCL-1: 3,500,000 journal lines, found {lines}"
        );
        assert!(
            accounts >= 20_000,
            "NFR-SCL-1: 20,000 accounts, found {accounts}"
        );
        assert_eq!(busy_bikri, 500, "NFR-SCL-2: 500 بکری on the busiest day");
        assert!(
            busy_lines >= 5_000,
            "NFR-SCL-2: 5,000 lines on the busiest day"
        );
    }
    let _ = world;
}

// ---- Helpers for the measuring tests ----------------------------------------------------------

use harness::{Row, finish, measure, measure_next, measure_with};

/// NFR-SPD-24: the first page of any list with its totals, and each next page.
const FIRST_PAGE_MS: f64 = 1_000.0;
const NEXT_PAGE_MS: f64 = 300.0;
/// The rows of a page, as the screens ask for them.
const PAGE: u32 = 100;
/// The busiest day of the data (NFR-SCL-2) and an ordinary one.
const BUSY_DAY: &str = "10-10-2026";
const ORDINARY_DAY: &str = "15-09-2026";

/// The first page and the next page of one list, each timed cold.
fn pages<T>(
    rows: &mut Vec<Row>,
    id: &str,
    nfr: &str,
    action: &str,
    mut call: impl FnMut(u32) -> T,
) {
    rows.push(measure(
        &format!("{id}.first"),
        nfr,
        &format!("{action}, first page with totals"),
        FIRST_PAGE_MS,
        || call(0),
    ));
    // The next page as a user meets it: the first page has just been read.
    let mut again = |offset: u32| call(offset);
    let prime = std::cell::RefCell::new(&mut again);
    rows.push(measure_next(
        &format!("{id}.next"),
        nfr,
        &format!("{action}, next page"),
        NEXT_PAGE_MS,
        || drop((prime.borrow_mut())(0)),
        || drop((prime.borrow_mut())(PAGE)),
    ));
}

fn id_where(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0))
        .unwrap_or_else(|e| support::fail(&format!("{sql}: {e}")))
}

// ---- NFR-SPD-2, 22, 24: the dashboard and every tile page (plan 79) ---------------------------

#[test]
#[ignore = "needs the full volume and a release build: see the module doc"]
fn t2_02_dashboard_and_tile_pages() {
    use mandi_reports::dashboard::{
        self, BackupFacts, BuyerDuesInput, CashSalesDayInput, CashSalesMonthInput,
        CommissionDayInput, CommissionMonthInput, CommissionParty, CommissionPeriod,
        CommissionPeriodKind, CommissionSide, LentOpenInput, SellerDuesInput, SellerDuesSide,
    };
    use mandi_services::core::paging::{SortDirection, SortInput};
    use mandi_services::core::vouchers::AdvancesReportInput;
    use mandi_services::vouchers;

    let w = world();
    let db = w.env.ctx.db();
    let ctx = &*w.env.ctx;
    let conn = raw(&w.gate);
    let busy_buyer = w.gate.busy_buyer;
    let busy_seller = w.gate.busy_seller;
    let backup = || BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    };
    let sort = |column: &str, direction: SortDirection| {
        Some(SortInput {
            language: None,
            column: column.into(),
            direction,
        })
    };
    let mut rows: Vec<Row> = Vec::new();

    // The tiles, and the attention list.
    rows.push(measure(
        "dash.get",
        "NFR-SPD-2",
        "dashboard_get: every tile, the lists and the charts",
        1_000.0,
        || dashboard::get(db, backup()).expect("the dashboard"),
    ));
    rows.push(measure(
        "dash.attention",
        "NFR-SPD-22",
        "assist_attention: the list and the count on the bell",
        1_000.0,
        || dashboard::attention(db, backup()).expect("the attention list"),
    ));
    rows.push(measure(
        "dash.total_dues",
        "NFR-SPD-24",
        "dashboard_total_dues: the four parts and the grand total (tile کل اگراہی page)",
        FIRST_PAGE_MS,
        || dashboard::total_dues_parts(db).expect("total dues"),
    ));

    // The commission page.
    let month = |period: Option<CommissionPeriod>, party: Option<CommissionParty>, offset: u32| {
        dashboard::commission_month(
            db,
            &CommissionMonthInput {
                month: None,
                period,
                party,
                goods: None,
                offset: Some(offset),
                limit: Some(PAGE),
            },
        )
        .expect("the commission page")
    };
    rows.push(measure(
        "dash.commission.month",
        "NFR-SPD-24",
        "dashboard_commission_month: this month",
        FIRST_PAGE_MS,
        || month(None, None, 0),
    ));
    let year = || CommissionPeriod {
        kind: CommissionPeriodKind::Year,
        year: Some(2025),
        ..CommissionPeriod::default()
    };
    rows.push(measure(
        "dash.commission.year",
        "NFR-SPD-24",
        "dashboard_commission_month: one year, a row a month",
        FIRST_PAGE_MS,
        || month(Some(year()), None, 0),
    ));
    let range = || CommissionPeriod {
        kind: CommissionPeriodKind::Range,
        from: Some("11-10-2016".into()),
        to: Some("10-10-2026".into()),
        ..CommissionPeriod::default()
    };
    pages(
        &mut rows,
        "dash.commission.range10y",
        "NFR-SPD-24",
        "commission page, a 10 year range (a row a day)",
        |o| month(Some(range()), None, o),
    );
    rows.push(measure(
        "dash.commission.seller",
        "NFR-SPD-24",
        "commission page, the busiest seller, this year",
        FIRST_PAGE_MS,
        || {
            month(
                Some(CommissionPeriod {
                    kind: CommissionPeriodKind::Year,
                    year: Some(2026),
                    ..CommissionPeriod::default()
                }),
                Some(CommissionParty {
                    side: CommissionSide::Seller,
                    account_id: busy_seller,
                    chukta_person_key: None,
                }),
                0,
            )
        },
    ));
    rows.push(measure(
        "dash.commission.buyer10y",
        "NFR-SPD-24",
        "commission page, the busiest buyer, 10 year range",
        FIRST_PAGE_MS,
        || {
            month(
                Some(range()),
                Some(CommissionParty {
                    side: CommissionSide::Buyer,
                    account_id: busy_buyer,
                    chukta_person_key: None,
                }),
                0,
            )
        },
    ));
    pages(
        &mut rows,
        "dash.commission.day",
        "NFR-SPD-24",
        "commission, the busy day's detail (500 بکری)",
        |o| {
            dashboard::commission_day(
                db,
                &CommissionDayInput {
                    date: BUSY_DAY.into(),
                    offset: o,
                    limit: PAGE,
                    party: None,
                    goods: None,
                },
            )
            .expect("the commission day")
        },
    );

    // The buyer dues page.
    let buyers = |party: Option<i64>, search: Option<&str>, sort_by: Option<SortInput>, o: u32| {
        dashboard::buyer_dues(
            db,
            &BuyerDuesInput {
                search: search.map(str::to_owned),
                party_id: party,
                sort: sort_by,
                offset: o,
                limit: PAGE,
            },
        )
        .expect("buyer dues")
    };
    pages(
        &mut rows,
        "dash.buyer_dues",
        "NFR-SPD-24",
        "dashboard_buyer_dues, all buyers",
        |o| buyers(None, None, None, o),
    );
    pages(
        &mut rows,
        "dash.buyer_dues.sort",
        "NFR-SPD-24",
        "dashboard_buyer_dues, sorted by balance",
        |o| buyers(None, None, sort("balance", SortDirection::Desc), o),
    );
    rows.push(measure(
        "dash.buyer_dues.party",
        "NFR-SPD-24",
        "dashboard_buyer_dues with party_id (the busiest buyer)",
        FIRST_PAGE_MS,
        || buyers(Some(busy_buyer), None, None, 0),
    ));
    rows.push(measure(
        "dash.buyer_dues.search",
        "NFR-SPD-24",
        "dashboard_buyer_dues, a text search",
        FIRST_PAGE_MS,
        || buyers(None, Some("akram"), None, 0),
    ));

    // The seller dues page, both tiles.
    let sellers = |side: SellerDuesSide, party: Option<i64>, sort_by: Option<SortInput>, o: u32| {
        dashboard::seller_dues(
            db,
            &SellerDuesInput {
                side,
                party_id: party,
                search: None,
                sort: sort_by,
                offset: o,
                limit: PAGE,
            },
        )
        .expect("seller dues")
    };
    for (name, side) in [
        ("all", SellerDuesSide::All),
        ("shop_owes", SellerDuesSide::ShopOwes),
        ("seller_owes", SellerDuesSide::SellerOwes),
    ] {
        pages(
            &mut rows,
            &format!("dash.seller_dues.{name}"),
            "NFR-SPD-24",
            &format!("dashboard_seller_dues, side {name}"),
            |o| sellers(side, None, None, o),
        );
    }
    rows.push(measure(
        "dash.seller_dues.party",
        "NFR-SPD-24",
        "dashboard_seller_dues with party_id (the busiest seller)",
        FIRST_PAGE_MS,
        || sellers(SellerDuesSide::All, Some(busy_seller), None, 0),
    ));
    rows.push(measure(
        "dash.seller_dues.sort",
        "NFR-SPD-24",
        "dashboard_seller_dues, sorted by balance",
        FIRST_PAGE_MS,
        || {
            sellers(
                SellerDuesSide::All,
                None,
                sort("balance", SortDirection::Desc),
                0,
            )
        },
    ));

    // The people who borrowed.
    pages(
        &mut rows,
        "dash.lent_open",
        "NFR-SPD-24",
        "dashboard_lent_open",
        |o| {
            dashboard::lent_open(
                db,
                &LentOpenInput {
                    search: None,
                    sort: None,
                    offset: o,
                    limit: PAGE,
                },
            )
            .expect("lent open")
        },
    );

    // The cash sales page: a month, a day with the busy day's chukta buyers.
    rows.push(measure(
        "dash.cash_sales.month",
        "NFR-SPD-24",
        "dashboard_cash_sales_month: this month",
        FIRST_PAGE_MS,
        || {
            dashboard::cash_sales_month(db, &CashSalesMonthInput { month: None })
                .expect("cash sales month")
        },
    ));
    rows.push(measure(
        "dash.cash_sales.month_old",
        "NFR-SPD-24",
        "dashboard_cash_sales_month: a month 4 years back",
        FIRST_PAGE_MS,
        || {
            dashboard::cash_sales_month(
                db,
                &CashSalesMonthInput {
                    month: Some("2022-05".into()),
                },
            )
            .expect("cash sales month")
        },
    ));
    pages(
        &mut rows,
        "dash.cash_sales.day",
        "NFR-SPD-24",
        "dashboard_cash_sales_day, the busy day",
        |o| {
            dashboard::cash_sales_day(
                db,
                &CashSalesDayInput {
                    date: BUSY_DAY.into(),
                    offset: o,
                    limit: PAGE,
                },
            )
            .expect("cash sales day")
        },
    );

    // The advances page (the tile پیشگی باقی): only the open ones, and all.
    let advances = |only_open: bool, seller: Option<i64>, o: u32| {
        vouchers::advances_report(
            ctx,
            &AdvancesReportInput {
                seller_id: seller,
                as_of: String::new(),
                offset: o,
                limit: PAGE,
                search: None,
                only_open,
            },
        )
        .expect("advances")
    };
    pages(
        &mut rows,
        "dash.advances.open",
        "NFR-SPD-24",
        "advances_report only_open (tile پیشگی باقی page)",
        |o| advances(true, None, o),
    );
    pages(
        &mut rows,
        "dash.advances.all",
        "NFR-SPD-24",
        "advances_report, every seller ever given an advance (R26)",
        |o| advances(false, None, o),
    );
    rows.push(measure(
        "dash.advances.seller",
        "NFR-SPD-24",
        "advances_report for one seller",
        FIRST_PAGE_MS,
        || advances(false, Some(busy_seller), 0),
    ));

    // The print data of the pages: every row. NFR-SPD-14 is a 15 page report; these are far
    // longer, so the row is information (target 0).
    {
        use mandi_services::core::auth::Language;
        let _ = Language::Ur;
    }
    let _ = &conn;
    finish(&rows);
}

// ---- NFR-SPD-10, 11, 13, 24: every report R1 to R38 on the full data ---------------------------

/// Every report of the catalogue, for the busy day (500 بکری, 5,000 lines: NFR-SCL-2), for an
/// ordinary day, and over the whole 10 years. The first page with its totals, and the next page
/// when the report has one.
#[test]
#[ignore = "needs the full volume and a release build: see the module doc"]
fn t2_03_every_report() {
    use mandi_reports::reports::{
        self, FilterKind, PartyFilter, ReportFilters, ReportId, ReportRunInput,
    };

    let w = world();
    let db = w.env.ctx.db();
    let conn = raw(&w.gate);
    let busy_buyer = w.gate.busy_buyer;
    let busy_bikri = id_where(
        &conn,
        "SELECT MIN(id) FROM bikri_entries WHERE entry_date = '2026-10-10'",
    );
    let partner = id_where(&conn, "SELECT MIN(id) FROM partners");
    let mut rows: Vec<Row> = Vec::new();
    let mut refused: Vec<String> = Vec::new();

    let filters_for = |kinds: &[FilterKind], from: &str, to: &str, party_required: bool| {
        let mut f = ReportFilters::default();
        for kind in kinds {
            match kind {
                FilterKind::DateRange => {
                    f.from = from.to_owned();
                    f.to = to.to_owned();
                }
                FilterKind::AsOf => f.as_of = to.to_owned(),
                FilterKind::Bikri => f.bikri_id = Some(busy_bikri),
                FilterKind::Partner => f.partner_id = Some(partner),
                _ => {}
            }
        }
        if party_required {
            f.party = Some(PartyFilter {
                account_id: busy_buyer,
                person_key: None,
            });
        }
        f
    };

    for entry in reports::catalog() {
        let code = format!("{:?}", entry.id);
        // R31 is built by the partner statement of the services (timed in t2_09).
        if entry.id == ReportId::R31 {
            continue;
        }
        // The variants: the busy day, an ordinary day, ten years.
        let variants = [
            ("busy_day", BUSY_DAY, BUSY_DAY),
            ("ordinary_day", ORDINARY_DAY, ORDINARY_DAY),
            ("ten_years", "11-10-2016", BUSY_DAY),
        ];
        for (name, from, to) in variants {
            // R34 and R35 make a sheet for each day: ten years is 3,650 sheets, not an NFR action.
            if matches!(entry.id, ReportId::R34 | ReportId::R35) && name == "ten_years" {
                continue;
            }
            // The party ledger and statement are asked for the whole 10 years only.
            if entry.party_required && name != "ten_years" {
                continue;
            }
            // A balance list has no range: one variant.
            if entry.filters.contains(&FilterKind::AsOf)
                && !entry.filters.contains(&FilterKind::DateRange)
                && name != "busy_day"
            {
                continue;
            }
            let filters = filters_for(&entry.filters, from, to, entry.party_required);
            let run = |offset: i64| {
                reports::run(
                    db,
                    &ReportRunInput {
                        id: entry.id,
                        filters: filters.clone(),
                        offset,
                        limit: i64::from(PAGE),
                        sort: None,
                        group_by_month: false,
                    },
                )
            };
            // A report that refuses its filters is a finding of its own, not a time.
            let first = match run(0) {
                Ok(page) => page,
                Err(error) => {
                    refused.push(format!("{code} {name}: {error}"));
                    continue;
                }
            };
            let (nfr, target) = match entry.id {
                ReportId::R18 => ("NFR-SPD-13", 2_000.0),
                _ if entry.party_required => ("NFR-SPD-11/24", FIRST_PAGE_MS),
                _ if name == "ten_years" => ("NFR-SPD-24", FIRST_PAGE_MS),
                _ => ("NFR-SPD-10", FIRST_PAGE_MS),
            };
            rows.push(measure(
                &format!("report.{code}.{name}.first"),
                nfr,
                &format!("{code} {name}, first page ({} rows)", first.total_count),
                target,
                || run(0).expect("the report"),
            ));
            if first.total_count > i64::from(PAGE) {
                rows.push(measure_next(
                    &format!("report.{code}.{name}.next"),
                    "NFR-SPD-24",
                    &format!("{code} {name}, next page"),
                    NEXT_PAGE_MS,
                    || run(0).expect("the report"),
                    || run(i64::from(PAGE)).expect("the report"),
                ));
            }
        }
    }
    for text in &refused {
        harness::fact("report_refused", text);
    }
    finish(&rows);
}

// ---- NFR-SPD-12, 24: every analytics view over 5 years ------------------------------------------

/// The views A to M over 5 years, and the views with a grouping by day, month and year. The first
/// page of A to E, H and L over more than a year may take 4 s (NFR-SPD-24 v1.19); NFR-SPD-12 still
/// says 1 s, so that figure is kept in the row.
#[test]
#[ignore = "needs the full volume and a release build: see the module doc"]
fn t2_04_every_analytics_view() {
    use mandi_reports::analytics::{
        AnalyticsFilters, AnalyticsGroupBy, AnalyticsRunInput, AnalyticsView,
    };

    let w = world();
    let db = w.env.ctx.db();
    let mut rows: Vec<Row> = Vec::new();
    let five = || AnalyticsFilters {
        from: "11-10-2021".into(),
        to: BUSY_DAY.into(),
        ..AnalyticsFilters::default()
    };
    let ten = || AnalyticsFilters {
        from: "11-10-2016".into(),
        to: BUSY_DAY.into(),
        ..AnalyticsFilters::default()
    };
    let run = |view: AnalyticsView, filters: AnalyticsFilters, offset: i64| {
        mandi_reports::analytics::run(
            db,
            &AnalyticsRunInput {
                view,
                filters,
                offset,
                limit: i64::from(PAGE),
                sort: None,
            },
        )
    };
    let exception = |view: AnalyticsView| {
        matches!(
            view,
            AnalyticsView::A
                | AnalyticsView::B
                | AnalyticsView::C
                | AnalyticsView::D
                | AnalyticsView::E
                | AnalyticsView::H
                | AnalyticsView::L
        )
    };
    let mut refused: Vec<String> = Vec::new();
    let mut cases: Vec<(String, AnalyticsView, AnalyticsFilters)> = Vec::new();
    for view in [
        AnalyticsView::A,
        AnalyticsView::B,
        AnalyticsView::C,
        AnalyticsView::D,
        AnalyticsView::E,
        AnalyticsView::F,
        AnalyticsView::G,
        AnalyticsView::H,
        AnalyticsView::I,
        AnalyticsView::J,
        AnalyticsView::K,
        AnalyticsView::L,
        AnalyticsView::M,
    ] {
        cases.push((format!("{view:?} 5 years"), view, five()));
        cases.push((format!("{view:?} 10 years"), view, ten()));
    }
    for view in [AnalyticsView::D, AnalyticsView::G] {
        for (name, group) in [
            ("by day", AnalyticsGroupBy::Day),
            ("by month", AnalyticsGroupBy::Month),
            ("by year", AnalyticsGroupBy::Year),
        ] {
            cases.push((
                format!("{view:?} 5 years {name}"),
                view,
                AnalyticsFilters {
                    group_by: Some(group),
                    ..five()
                },
            ));
        }
    }
    for (name, view, filters) in cases {
        let first = match run(view, filters.clone(), 0) {
            Ok(page) => page,
            Err(error) => {
                refused.push(format!("{name}: {error}"));
                continue;
            }
        };
        let long = name.contains("10 years") || name.contains("5 years");
        let (nfr, target) = if exception(view) && long {
            ("NFR-SPD-24 exception (SPD-12: 1 s)", 4_000.0)
        } else {
            ("NFR-SPD-12/24", FIRST_PAGE_MS)
        };
        let also = if exception(view) && long {
            FIRST_PAGE_MS
        } else {
            0.0
        };
        rows.push(measure_with(
            &format!("analytics.{}.first", name.replace(' ', "_")),
            nfr,
            &format!(
                "analytics view {name}, first page ({} rows)",
                first.total_count
            ),
            target,
            also,
            || run(view, filters.clone(), 0).expect("the view"),
        ));
        if first.total_count > i64::from(PAGE) {
            rows.push(measure_next(
                &format!("analytics.{}.next", name.replace(' ', "_")),
                "NFR-SPD-24",
                &format!("analytics view {name}, next page"),
                NEXT_PAGE_MS,
                || run(view, filters.clone(), 0).expect("the view"),
                || run(view, filters.clone(), i64::from(PAGE)).expect("the view"),
            ));
        }
    }
    for text in &refused {
        harness::fact("analytics_refused", text);
    }
    finish(&rows);
}

// ---- NFR-SPD-3, 10, 11, 21, 23, 24: lists, windows, ledgers and statements ----------------------

#[test]
#[ignore = "needs the full volume and a release build: see the module doc"]
fn t2_05_lists_windows_and_statements() {
    use mandi_reports::reports::{StatementRowsInput, statement_rows};
    use mandi_services::core::bikri::{BikriListInput, StockListInput};
    use mandi_services::core::cashbook::CashbookInput;
    use mandi_services::core::paging::{SortDirection, SortInput};
    use mandi_services::core::party_ledgers::{
        LedgerSide, PartyLedgerGoToInput, PartyLedgerStatementInput, PartyLedgersListInput,
    };
    use mandi_services::core::statement::PartnersStatementMonthInput;
    use mandi_services::{assist, bikri, cashbook, partners, party_ledgers};

    let w = world();
    let ctx = &*w.env.ctx;
    let db = ctx.db();
    let conn = raw(&w.gate);
    let busy_buyer = w.gate.busy_buyer;
    let busy_seller = w.gate.busy_seller;
    let partner = id_where(&conn, "SELECT MIN(id) FROM partners");
    let seller_with_lots = id_where(
        &conn,
        "SELECT seller_account_id FROM bikri_lots WHERE bikri_id = (SELECT MIN(id) FROM bikri_entries WHERE entry_date = '2026-10-10') LIMIT 1",
    );
    let mut rows: Vec<Row> = Vec::new();

    // پچھلی بکری: today's section (500 بکری), the window of 7 days, the moves (PRD FR-12.46).
    let list = |input: BikriListInput| bikri::list(ctx, &input).expect("the بکری list");
    let window = |from: &str, to: &str| BikriListInput {
        from: Some(from.into()),
        to: Some(to.into()),
        group_by_day: true,
        limit: 25,
        ..BikriListInput::default()
    };
    rows.push(measure(
        "past.today",
        "NFR-SPD-24",
        "پچھلی بکری opens: today's section (500 بکری)",
        FIRST_PAGE_MS,
        || list(window(BUSY_DAY, BUSY_DAY)),
    ));
    rows.push(measure(
        "past.window",
        "NFR-SPD-24",
        "پچھلی بکری: the 7 day window before today",
        FIRST_PAGE_MS,
        || list(window("03-10-2026", "09-10-2026")),
    ));
    rows.push(measure(
        "past.step_back",
        "NFR-SPD-24",
        "پچھلی بکری: one step back (7 days)",
        NEXT_PAGE_MS,
        || list(window("26-09-2026", "02-10-2026")),
    ));
    rows.push(measure(
        "past.jump_5y",
        "NFR-SPD-24",
        "پچھلی بکری: a jump 5 years back",
        NEXT_PAGE_MS,
        || list(window("08-10-2021", "14-10-2021")),
    ));
    rows.push(measure(
        "past.show_all_today",
        "NFR-SPD-24",
        "پچھلی بکری: show everything, today",
        NEXT_PAGE_MS,
        || {
            list(BikriListInput {
                all: true,
                ..window(BUSY_DAY, BUSY_DAY)
            })
        },
    ));
    rows.push(measure(
        "past.today_last_page",
        "NFR-SPD-24",
        "پچھلی بکری: today's last page (offset 475)",
        NEXT_PAGE_MS,
        || {
            list(BikriListInput {
                offset: 475,
                ..window(BUSY_DAY, BUSY_DAY)
            })
        },
    ));
    rows.push(measure(
        "past.seller",
        "NFR-SPD-24",
        "پچھلی بکری: a seller over the window",
        NEXT_PAGE_MS,
        || {
            list(BikriListInput {
                seller_id: Some(seller_with_lots),
                ..window("03-10-2026", "09-10-2026")
            })
        },
    ));
    rows.push(measure(
        "past.sort_gross",
        "NFR-SPD-24",
        "پچھلی بکری: sort by gross over today (500)",
        NEXT_PAGE_MS,
        || {
            list(BikriListInput {
                sort: Some(SortInput {
                    language: None,
                    column: "gross".into(),
                    direction: SortDirection::Desc,
                }),
                ..window(BUSY_DAY, BUSY_DAY)
            })
        },
    ));
    rows.push(measure(
        "past.ten_years_range",
        "NFR-SPD-24",
        "بکری list over 10 years, first page of 25",
        FIRST_PAGE_MS,
        || {
            list(BikriListInput {
                from: Some("11-10-2016".into()),
                to: Some(BUSY_DAY.into()),
                limit: 25,
                ..BikriListInput::default()
            })
        },
    ));

    // باقی مال (stock).
    pages(
        &mut rows,
        "stock.list",
        "NFR-SPD-24",
        "باقی مال list",
        |o| {
            bikri::stock_list(
                ctx,
                &StockListInput {
                    offset: o,
                    limit: PAGE,
                    ..StockListInput::default()
                },
            )
            .expect("the stock list")
        },
    );

    // Party ledgers (PRD FR-12.43, FR-12.47).
    for (name, side) in [
        ("buyers", LedgerSide::Buyer),
        ("sellers", LedgerSide::Seller),
    ] {
        pages(
            &mut rows,
            &format!("ledgers.list.{name}"),
            "NFR-SPD-24",
            &format!("party ledgers level 1, {name}"),
            |o| {
                party_ledgers::list(
                    ctx,
                    &PartyLedgersListInput {
                        side,
                        text: None,
                        offset: o,
                        limit: PAGE,
                    },
                )
                .expect("the party ledgers")
            },
        );
    }
    let statement = |account: i64, from: Option<&str>, to: Option<&str>, o: u32| {
        party_ledgers::statement(
            ctx,
            &PartyLedgerStatementInput {
                account_id: account,
                from: from.map(str::to_owned),
                to: to.map(str::to_owned),
                offset: o,
                limit: 30,
            },
        )
        .expect("the statement")
    };
    rows.push(measure(
        "ledgers.statement.busy_buyer.30d",
        "NFR-SPD-24",
        "party statement, the busiest buyer, newest 30 days window",
        FIRST_PAGE_MS,
        || statement(busy_buyer, Some("11-09-2026"), Some(BUSY_DAY), 0),
    ));
    rows.push(measure(
        "ledgers.statement.busy_buyer.all",
        "NFR-SPD-24",
        "party statement, the busiest buyer, every date, first 30",
        FIRST_PAGE_MS,
        || statement(busy_buyer, None, None, 0),
    ));
    rows.push(measure_next(
        "ledgers.statement.busy_buyer.next",
        "NFR-SPD-24",
        "party statement, the busiest buyer, next 30 dates",
        NEXT_PAGE_MS,
        || statement(busy_buyer, None, None, 0),
        || statement(busy_buyer, None, None, 30),
    ));
    rows.push(measure(
        "ledgers.statement.busy_seller.30d",
        "NFR-SPD-24",
        "party statement, the busiest seller, 30 days window",
        FIRST_PAGE_MS,
        || statement(busy_seller, Some("11-09-2026"), Some(BUSY_DAY), 0),
    ));
    rows.push(measure(
        "ledgers.statement.5y_back",
        "NFR-SPD-24",
        "party statement, the busiest buyer, a window 5 years back",
        NEXT_PAGE_MS,
        || statement(busy_buyer, Some("01-09-2021"), Some("30-09-2021"), 0),
    ));
    rows.push(measure(
        "ledgers.go_to_date",
        "NFR-SPD-24",
        "party statement go to a date (busiest buyer, 3 years back)",
        FIRST_PAGE_MS,
        || {
            party_ledgers::go_to_date(
                ctx,
                &PartyLedgerGoToInput {
                    account_id: busy_buyer,
                    date: "15-03-2023".into(),
                },
            )
            .expect("go to date")
        },
    ));
    let rows_of = |account: i64, history: bool, newest: bool, o: u32| {
        statement_rows(
            db,
            &StatementRowsInput {
                account_id: account,
                from: None,
                to: None,
                include_history: history,
                newest_first: newest,
                offset: o,
                limit: PAGE,
            },
        )
        .expect("the statement rows")
    };
    pages(
        &mut rows,
        "ledgers.rows.busy_buyer",
        "NFR-SPD-24",
        "statement rows (one row an entry), busiest buyer, newest first",
        |o| rows_of(busy_buyer, false, true, o),
    );
    pages(
        &mut rows,
        "ledgers.rows.busy_buyer_hist",
        "NFR-SPD-24",
        "statement rows with the edit history, busiest buyer",
        |o| rows_of(busy_buyer, true, true, o),
    );
    pages(
        &mut rows,
        "ledgers.rows.busy_seller",
        "NFR-SPD-24",
        "statement rows, busiest seller, oldest first",
        |o| rows_of(busy_seller, false, false, o),
    );
    rows.push(measure(
        "ledgers.whole.30d",
        "NFR-SPD-14",
        "party statement for print, 30 days (the busiest buyer)",
        2_000.0,
        || {
            party_ledgers::statement_whole(ctx, busy_buyer, "11-09-2026", BUSY_DAY)
                .expect("statement")
        },
    ));
    rows.push(measure(
        "ledgers.whole.6m",
        "NFR-SPD-14",
        "party statement for print, 6 months (the busiest buyer)",
        2_000.0,
        || {
            party_ledgers::statement_whole(ctx, busy_buyer, "10-04-2026", BUSY_DAY)
                .expect("statement")
        },
    ));

    // روکڑ بک: a window of 7 days, and the first page of 10 years (PRD FR-16.14).
    let book = |from: Option<&str>, to: Option<&str>, o: u32| {
        cashbook::get(
            ctx,
            &CashbookInput {
                from: from.map(str::to_owned),
                to: to.map(str::to_owned),
                offset: o,
                limit: PAGE,
                sort: None,
                group_by_month: false,
            },
        )
        .expect("the cash book")
    };
    pages(
        &mut rows,
        "cashbook.busy_day",
        "NFR-SPD-24",
        "روکڑ بک, the busy day",
        |o| book(Some(BUSY_DAY), Some(BUSY_DAY), o),
    );
    pages(
        &mut rows,
        "cashbook.window7",
        "NFR-SPD-24",
        "روکڑ بک, a window of 7 days",
        |o| book(Some("03-10-2026"), Some("09-10-2026"), o),
    );
    pages(
        &mut rows,
        "cashbook.ten_years",
        "NFR-SPD-24",
        "روکڑ بک, 10 years",
        |o| book(None, None, o),
    );

    // Partners: the overview, a month of one partner's statement (PRD FR-17.12).
    rows.push(measure(
        "partners.overview",
        "NFR-SPD-3",
        "partners overview",
        300.0,
        || partners::overview(ctx).expect("the overview"),
    ));
    rows.push(measure(
        "partners.statement_month",
        "NFR-SPD-24",
        "partner statement, this month, a row a day",
        FIRST_PAGE_MS,
        || {
            partners::statement_month(
                ctx,
                &PartnersStatementMonthInput {
                    partner_id: partner,
                    month: None,
                },
            )
            .expect("the partner statement")
        },
    ));

    // The hints of the entry screens (NFR-SPD-21, NFR-SPD-23).
    rows.push(measure(
        "assist.seller_memory",
        "NFR-SPD-23",
        "assist_seller_memory, the busiest seller",
        100.0,
        || assist::seller_memory(ctx, busy_seller).expect("seller memory"),
    ));
    rows.push(measure(
        "assist.party_card.busy_buyer",
        "NFR-SPD-21",
        "assist_party_card, the busiest buyer (with 1.2 million sale lines in the file)",
        100.0,
        || assist::party_card(ctx, busy_buyer).expect("the card"),
    ));
    rows.push(measure(
        "assist.party_card.busy_seller",
        "NFR-SPD-21",
        "assist_party_card, the busiest seller",
        100.0,
        || assist::party_card(ctx, busy_seller).expect("the card"),
    ));
    finish(&rows);
}

// ---- NFR-SPD-1, 7, 8, NFR-SCL-3: start-up, the live summary and the save of a بکری ---------------

use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, FreightPaidBy, HeaderInput, LotInput, SaleInput,
};

/// A private copy of the data in the perf folder, removed when it goes out of scope.
struct Copy {
    dir: std::path::PathBuf,
}

impl Copy {
    fn of(gate: &volume::Gate) -> Self {
        let dir = gate
            .dir
            .with_file_name(format!("gate_t2_copy_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let to = dir.join("data");
        std::fs::create_dir_all(&to).unwrap_or_else(|e| support::fail(&format!("no folder: {e}")));
        std::fs::copy(gate.dir.join("data").join("mandi.db"), to.join("mandi.db"))
            .unwrap_or_else(|e| support::fail(&format!("cannot copy the data: {e}")));
        Self { dir }
    }
}

impl Drop for Copy {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn account_by_code(env: &TestEnv, code: &str) -> i64 {
    bikri_draft::id_by_code(env, code)
}

/// `lots` lots of `lines` lines each. The seller of lot `n` is `001n`, the buyers are `002k` in
/// turn over 500 buyers, every line is 10 نگ at 600 (the shape of `benches/bikri_draft`, with the
/// account codes of today: the bench helper still asks for the codes of before plan 32).
fn draft(env: &TestEnv, lots: usize, lines: usize) -> BikriDraft {
    let packing = bikri_draft::first_id(env, "packings", 0);
    // Active buyers only: the layer moved 100 of them to the dead amount (PRD FR-10.10).
    let buyers: Vec<i64> = env
        .ctx
        .db()
        .read(|conn| {
            let mut statement = conn.prepare(
                "SELECT id FROM accounts WHERE code GLOB '002[0-9]*' AND is_system = 0 AND is_active = 1
                 ORDER BY serial LIMIT 500",
            )?;
            let rows = statement.query_map([], |r| r.get::<_, i64>(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_else(|e| support::fail(&format!("no buyers: {e}")));
    let lot = |n: usize| LotInput {
        arrival_id: None,
        key: format!("lot-{n}"),
        lot_id: None,
        seller_account_id: Some(account_by_code(env, &format!("001{}", 1 + n))),
        purchase_id: None,
        item_id: Some(bikri_draft::first_id(
            env,
            "items",
            i64::try_from(n % 3).unwrap_or(0),
        )),
        packing_id: Some(packing),
        marka_id: None,
        store_id: None,
        qty_received: (10 * lines).to_string(),
        weight_received: None,
        freight: Some("3,000".to_owned()),
        freight_paid_by: FreightPaidBy::Firm,
        labour: None,
        munshiana: None,
        phone: None,
        cash_paid: Some("5,000".to_owned()),
        store_rent: None,
        adjust_advance: true,
        sales: (0..lines)
            .map(|j| SaleInput {
                key: format!("s-{n}-{j}"),
                sale_id: None,
                buyer_account_id: buyers.get((n * lines + j) % 500).copied(),
                chukta: None,
                qty: "10".to_owned(),
                weight: None,
                seller_rate: "600".to_owned(),
                rate_differs: false,
                buyer_rate: None,
                paid: None,
                seller_comm: None,
                buyer_comm: None,
                laga: None,
                grade: None,
                note: None,
            })
            .collect(),
    };
    BikriDraft {
        header: HeaderInput {
            entry_date: BUSY_DAY.to_owned(),
            bill_date: String::new(),
            book: None,
            truck_no: None,
            zamindar_account_id: None,
            adda_id: None,
            driver_name: None,
            driver_phone: None,
            total_freight: None,
            store_id: None,
            note: None,
        },
        lots: (0..lots).map(lot).collect(),
    }
}

#[test]
#[ignore = "needs the full volume and a release build: see the module doc"]
fn t2_06_start_up_preview_and_save() {
    use mandi_services::bikri;

    let gate = volume::gate_volume();
    let mut rows: Vec<Row> = Vec::new();

    // The backend part of NFR-SPD-1: open the file, run the start-up path, sign in. (The cold
    // start also loads the window and the screen: that part is `pnpm e2e` on Windows.)
    rows.push(measure(
        "start.open_and_sign_in",
        "NFR-SPD-1 (backend part)",
        "open the 2.3 GB data file with the start-up check on this thread (the app runs it in the background), sign in (fast test hasher): information",
        0.0,
        || support::open(&gate.dir),
    ));
    {
        let (env, _actor) = support::open(&gate.dir);
        // Saving waits for this check, at most CHECK_WAIT = 30 s, then a save is refused as busy
        // (`Db::wait_until_writable`, NFR-REL-7).
        rows.push(measure(
            "start.quick_check",
            "NFR-REL-7 (CHECK_WAIT 30 s)",
            "PRAGMA quick_check of the whole file, the time saving is held after start-up",
            30_000.0,
            || env.ctx.db().run_quick_check(),
        ));
    }

    let (env, _actor) = support::open(&gate.dir);
    let thirty = draft(&env, 3, 10);
    let five_hundred = draft(&env, 50, 10);
    let thousand = draft(&env, 50, 20);
    for (name, d, lines) in [
        ("30", &thirty, 30_usize),
        ("500", &five_hundred, 500),
        ("1,000", &thousand, 1_000),
    ] {
        let p = bikri::preview(&env.ctx, d).expect("a preview");
        let worked = p
            .lots
            .iter()
            .flat_map(|l| l.lines.iter())
            .filter(|l| l.calc.is_some())
            .count();
        assert_eq!(
            worked, lines,
            "the {name}-line draft must be worked out, errors {:?}",
            p.errors
        );
    }
    // The live summary: 150 ms in all, of which the screen's own pause is 50 ms.
    rows.push(measure(
        "preview.30",
        "NFR-SPD-7",
        "bikri_preview, 30 lines",
        100.0,
        || bikri::preview(&env.ctx, &thirty).expect("a preview"),
    ));
    rows.push(measure_with(
        "preview.500",
        "NFR-SCL-3",
        "bikri_preview, 500 lines",
        100.0,
        150.0,
        || bikri::preview(&env.ctx, &five_hundred).expect("a preview"),
    ));
    rows.push(measure_with(
        "preview.1000",
        "NFR-SCL-3",
        "bikri_preview, 1,000 lines (the NFR-SCL-3 maximum)",
        100.0,
        150.0,
        || bikri::preview(&env.ctx, &thousand).expect("a preview"),
    ));
    drop(env);

    // The saves, on a private copy: each one adds a بکری.
    let copy = Copy::of(&gate);
    let (env, actor) = support::open(&copy.dir);
    let save_30 = BikriSaveInput {
        draft: draft(&env, 3, 10),
        confirm_cash_below_zero: true,
    };
    let saved = bikri::save(&env.ctx, &actor, &save_30).expect("the first save");
    assert_eq!(
        saved
            .preview
            .lots
            .iter()
            .map(|l| l.lines.len())
            .sum::<usize>(),
        30
    );
    rows.push(measure(
        "save.30",
        "NFR-SPD-8",
        "bikri_save, 30 lines (3 lots)",
        300.0,
        || bikri::save(&env.ctx, &actor, &save_30).expect("a save"),
    ));
    let save_1000 = BikriSaveInput {
        draft: draft(&env, 50, 20),
        confirm_cash_below_zero: true,
    };
    rows.push(measure(
        "save.1000",
        "NFR-SCL-3",
        "bikri_save, 1,000 lines (information: NFR-SPD-8 is for 30)",
        0.0,
        || bikri::save(&env.ctx, &actor, &save_1000).expect("a save"),
    ));
    // A voucher on the busiest buyer and seller, and the preview of one.
    {
        use mandi_services::core::vouchers::{AdvanceInput, ReceiptInput};
        use mandi_services::vouchers;
        let receipt = ReceiptInput {
            date: BUSY_DAY.into(),
            account_id: gate.busy_buyer,
            amount: "1,000".into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            collector_staff_id: None,
            narration: String::new(),
        };
        rows.push(measure(
            "save.receipt",
            "NFR-SPD-9",
            "vouchers_save_receipt, the busiest buyer",
            150.0,
            || vouchers::save_receipt(&env.ctx, &actor, &receipt).expect("a receipt"),
        ));
        let advance = AdvanceInput {
            confirm_cash_below_zero: false,
            date: BUSY_DAY.into(),
            seller_account_id: gate.busy_seller,
            amount: "1,000".into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: String::new(),
        };
        rows.push(measure(
            "save.advance",
            "NFR-SPD-9",
            "vouchers_save_advance, the busiest seller",
            150.0,
            || vouchers::save_advance(&env.ctx, &actor, &advance).expect("an advance"),
        ));
    }
    finish(&rows);
}

// ---- Code Standards B18: the plans of the plan 79 queries on the full data ----------------------

/// The plan of one statement as text, one line a step.
fn plan_of(
    conn: &Connection,
    sql: &str,
    args: Vec<mandi_services::db::rusqlite::types::Value>,
) -> Vec<String> {
    let mut statement = conn
        .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
        .unwrap_or_else(|e| support::fail(&format!("cannot explain: {e}\n{sql}")));
    statement
        .query_map(
            mandi_services::db::rusqlite::params_from_iter(args.iter()),
            |r| r.get::<_, String>(3),
        )
        .unwrap_or_else(|e| support::fail(&format!("cannot explain: {e}")))
        .filter_map(Result::ok)
        .collect()
}

/// A step that reads a whole big table: `SCAN` of `journal_lines`, `analytics_facts` or
/// `bikri_sales`, with or without an index (a covering index still walks every entry).
fn big_scan(step: &str) -> Option<&'static str> {
    for table in [
        "journal_lines",
        "analytics_facts",
        "bikri_sales",
        "partner_share_rows",
    ] {
        // The alias form `SCAN l` is matched through the table name that follows in the plan only
        // when the planner prints it; the plans here always print the table or the alias.
        if step.starts_with("SCAN")
            && (step.contains(table)
                || step.starts_with("SCAN l ")
                || step.starts_with("SCAN s ")
                || step.starts_with("SCAN f "))
        {
            return Some(table);
        }
    }
    None
}

#[test]
#[ignore = "needs the full volume and a release build: see the module doc"]
fn t2_07_query_plans() {
    use mandi_reports::dashboard::{cash_sales_plans, commission_plans};
    use mandi_services::db::rusqlite::types::Value;

    let gate = volume::gate_volume();
    let conn = raw(&gate);
    let text = |s: &str| Value::Text(s.to_owned());
    let mut cases: Vec<(&str, String, Vec<Value>)> = Vec::new();

    // The cash sales page (plan 79 item 2c).
    cases.push((
        "cash_sales.days.10y",
        cash_sales_plans::days().to_owned(),
        vec![text("2016-10-11"), text("2026-10-10")],
    ));
    cases.push((
        "cash_sales.days.month",
        cash_sales_plans::days().to_owned(),
        vec![text("2026-10-01"), text("2026-10-31")],
    ));
    cases.push((
        "cash_sales.day_buyers",
        cash_sales_plans::day_buyers().to_owned(),
        vec![text("2026-10-10")],
    ));
    // The commission page (plan 76, 77).
    for (name, from, to) in [
        ("month", "2026-10-01", "2026-10-31"),
        ("10y", "2016-10-11", "2026-10-10"),
    ] {
        let (sql, args) = commission_plans::facts_by_day(from, to);
        cases.push((
            if name == "month" {
                "commission.facts.month"
            } else {
                "commission.facts.10y"
            },
            sql,
            args,
        ));
        let (sql, args) = commission_plans::shares_by_day(from, to);
        cases.push((
            if name == "month" {
                "commission.shares.month"
            } else {
                "commission.shares.10y"
            },
            sql,
            args,
        ));
    }
    // The last change of every buyer and seller (the sort by last change and the print of the
    // dues pages): one grouped read of the whole books of those accounts.
    for (name, control) in [
        ("buyer_dues.last_changes", "0304"),
        ("seller_dues.last_changes", "03011"),
    ] {
        cases.push((
            name,
            format!(
                "SELECT l.account_id, MAX(e.entry_date) FROM journal_lines l
                 JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
                 JOIN accounts a ON a.id = l.account_id
                 JOIN account_controls k ON k.id = a.control_id
                 WHERE k.code = '{control}' GROUP BY l.account_id"
            ),
            vec![],
        ));
    }
    // The sums of every seller's lines (seller dues, total dues, the tile): the first of the six
    // grouped reads of `payable_sums`.
    cases.push((
        "seller_dues.payable_sums.lines",
        "SELECT account_id, COALESCE(SUM(cr_paisa), 0), COALESCE(SUM(dr_paisa), 0)
         FROM journal_lines WHERE account_id IN (SELECT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id WHERE k.code = '03011')
         GROUP BY account_id"
            .to_owned(),
        vec![],
    ));
    cases.push((
        "seller_dues.payable_sums.advances",
        "SELECT account_id, COALESCE(SUM(dr_paisa), 0) FROM journal_lines
         WHERE is_advance = 1 AND account_id IN (SELECT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id WHERE k.code = '03011')
         GROUP BY account_id"
            .to_owned(),
        vec![],
    ));
    // The register and the summary of one busy day.
    for (name, (sql, args)) in [
        (
            "r2.page.day",
            mandi_reports::reports::plans::r2_page(Some("2026-10-10"), Some("2026-10-10")),
        ),
        (
            "r2.totals.day",
            mandi_reports::reports::plans::r2_totals(Some("2026-10-10"), Some("2026-10-10")),
        ),
        (
            "r6.page.day",
            mandi_reports::reports::plans::r6_page(Some("2026-10-10"), Some("2026-10-10")),
        ),
        (
            "r7.day",
            mandi_reports::reports::plans::r7(Some("2026-10-10"), Some("2026-10-10")),
        ),
        (
            "r7.10y",
            mandi_reports::reports::plans::r7(Some("2016-10-11"), Some("2026-10-10")),
        ),
        (
            "r28.page.month",
            mandi_reports::reports::plans::r28_page(Some("2026-10-01"), Some("2026-10-31")),
        ),
    ] {
        cases.push((name, sql, args));
    }

    let mut scans: Vec<String> = Vec::new();
    for (name, sql, args) in cases {
        let plan = plan_of(&conn, &sql, args);
        harness::fact(&format!("plan.{name}"), &plan.join(" | "));
        for step in &plan {
            if let Some(table) = big_scan(step) {
                scans.push(format!("{name}: {step} ({table})"));
            }
        }
    }
    for scan in &scans {
        harness::fact("plan.scan_of_a_big_table", scan);
    }
    // The page and day queries must not scan a big table; the whole-books groupings above are
    // expected to, and are reported, not failed.
    let paged: Vec<&String> = scans
        .iter()
        .filter(|s| {
            s.starts_with("cash_sales")
                || s.starts_with("commission.facts.month")
                || s.starts_with("commission.shares.month")
                || s.starts_with("r2.")
                || s.starts_with("r6.")
        })
        .collect();
    assert!(
        paged.is_empty(),
        "a page query scans a big table: {paged:?}"
    );
}
