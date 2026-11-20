//! Plan 80, lane T2: where the time of `dashboard_get` and the tile pages goes, on the full data
//! (finding T2-1). Each tile has its own narrow read in `mandi_reports::dashboard`; this times
//! them one by one, so the slow part of the 1 s target (NFR-SPD-2) is named. Owner: `robustness`.
//!
//! `#[ignore]`d like the rest of lane T2: run `node scripts/perf/gate-t2-run.mjs --only
//! t2_08_dashboard_parts` (the runner knows the file by the test name).
#![cfg(test)]

#[path = "../benches/bikri_draft/mod.rs"]
mod bikri_draft;
#[allow(dead_code)]
#[path = "robustness_gate_t2/harness.rs"]
mod harness;
#[path = "../benches/support/mod.rs"]
mod support;
#[allow(dead_code)]
#[path = "robustness_gate_t2/volume.rs"]
mod volume;

use harness::{Row, finish, measure};
use mandi_reports::dashboard;

#[test]
#[ignore = "needs the full volume and a release build: see robustness_gate_t2.rs"]
fn t2_08_dashboard_parts() {
    let gate = volume::gate_volume();
    let (env, _actor) = support::open(&gate.dir);
    let db = env.ctx.db();
    let today = mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10).expect("a date");
    let mut rows: Vec<Row> = Vec::new();
    let nfr = "NFR-SPD-2 (a part of dashboard_get)";
    rows.push(measure(
        "dashparts.cash_in_hand",
        nfr,
        "tile موجود روکڑ",
        300.0,
        || dashboard::cash_in_hand(db).expect("cash"),
    ));
    rows.push(measure(
        "dashparts.sales_today",
        nfr,
        "tiles آج کی بکری and آج کا کمیشن",
        300.0,
        || dashboard::sales_today(db, today).expect("sales"),
    ));
    rows.push(measure(
        "dashparts.stock",
        nfr,
        "tile باقی مال",
        300.0,
        || dashboard::stock(db, today).expect("stock"),
    ));
    rows.push(measure(
        "dashparts.dead",
        nfr,
        "tile ڈوبی رقم",
        300.0,
        || dashboard::dead(db).expect("dead"),
    ));
    rows.push(measure(
        "dashparts.buyers_dues",
        nfr,
        "tile گاہکوں کے ذمے (balances of every buyer)",
        300.0,
        || dashboard::buyers_dues(db, today).expect("buyers"),
    ));
    rows.push(measure(
        "dashparts.sellers",
        nfr,
        "tiles of the sellers (payable, advances, owed)",
        300.0,
        || dashboard::sellers(db, today).expect("sellers"),
    ));
    rows.push(measure(
        "dashparts.total_dues",
        nfr,
        "tile کل اگراہی (the four parts)",
        300.0,
        || dashboard::total_dues(db, today).expect("total dues"),
    ));
    rows.push(measure(
        "dashparts.tiles_now",
        nfr,
        "every tile without the lists (narrow reads)",
        1_000.0,
        || dashboard::tiles_now(db).expect("tiles"),
    ));
    finish(&rows);
}

/// Finding R1-4 (lane R1): `partners::money::advance_net` reads and replays every advance in the
/// books for each call. The partner pages call it for each day or each partner, so they are timed
/// on the full data: the overview, the month statement (PRD FR-17.12), the day inside it, the
/// whole statement (R31), and the "where the capital is" card of روکڑ (PRD FR-16.14). The file
/// holds 3,000 seller advances over the 10 years (NFR-SCL-1 is 10 times the shop's real volume).
#[test]
#[ignore = "needs the full volume and a release build: see robustness_gate_t2.rs"]
fn t2_09_partner_pages() {
    use mandi_services::core::statement::{
        PartnersStatementDayInput, PartnersStatementInput, PartnersStatementMonthInput,
    };
    use mandi_services::{cashbook, partners};

    let gate = volume::gate_volume();
    let (env, _actor) = support::open(&gate.dir);
    let ctx = &*env.ctx;
    let conn = mandi_services::db::rusqlite::Connection::open_with_flags(
        gate.dir.join("data").join("mandi.db"),
        mandi_services::db::rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .expect("the data");
    let partner: i64 = conn
        .query_row("SELECT MIN(id) FROM partners", [], |r| r.get(0))
        .expect("a partner");
    let advances: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM journal_lines WHERE is_advance = 1",
            [],
            |r| r.get(0),
        )
        .expect("a count");
    harness::fact(
        "partner_pages.advances",
        &format!("{advances} advance lines in the books"),
    );
    let mut rows: Vec<Row> = Vec::new();
    let nfr = "NFR-SPD-24 (finding R1-4)";
    rows.push(measure(
        "partner.overview",
        nfr,
        "partners_overview (3 partners, every column)",
        1_000.0,
        || partners::overview(ctx).expect("the overview"),
    ));
    for (name, month) in [
        ("this_month", None),
        ("a_year_back", Some("2025-10")),
        ("five_years_back", Some("2021-10")),
    ] {
        rows.push(measure(
            &format!("partner.statement_month.{name}"),
            nfr,
            &format!("partners_statement_month, {name} (a row a day with movement)"),
            1_000.0,
            || {
                partners::statement_month(
                    ctx,
                    &PartnersStatementMonthInput {
                        partner_id: partner,
                        month: month.map(str::to_owned),
                    },
                )
                .expect("the month")
            },
        ));
    }
    rows.push(measure(
        "partner.statement_day",
        nfr,
        "partners_statement_day, the busy day (500 بکری)",
        1_000.0,
        || {
            partners::statement_day(
                ctx,
                &PartnersStatementDayInput {
                    partner_id: partner,
                    date: "10-10-2026".into(),
                },
            )
            .expect("the day")
        },
    ));
    let r31 = |offset: u32| {
        partners::statement(
            ctx,
            &PartnersStatementInput {
                partner_id: partner,
                from: None,
                to: None,
                offset,
                limit: 100,
                group_by_month: false,
                sort: None,
            },
        )
        .expect("the statement")
    };
    rows.push(measure(
        "partner.statement.r31.first",
        nfr,
        "partners_statement (R31), every date, first page of 100",
        1_000.0,
        || r31(0),
    ));
    rows.push(harness::measure_next(
        "partner.statement.r31.next",
        nfr,
        "partners_statement (R31), every date, next page",
        300.0,
        || r31(0),
        || r31(100),
    ));
    // R1 (the sale book, one بکری): the first بکری of the busy day, 11 lines.
    {
        use mandi_reports::reports::{self, ReportFilters, ReportId, ReportRunInput};
        let busy_bikri: i64 = conn
            .query_row(
                "SELECT MIN(id) FROM bikri_entries WHERE entry_date = '2026-10-10'",
                [],
                |r| r.get(0),
            )
            .expect("a بکری");
        rows.push(measure(
            "report.R1.one_bikri",
            "NFR-SPD-10",
            "R1 sale book, one بکری of the busy day",
            1_000.0,
            || {
                reports::run(
                    env.ctx.db(),
                    &ReportRunInput {
                        id: ReportId::R1,
                        filters: ReportFilters {
                            bikri_id: Some(busy_bikri),
                            ..ReportFilters::default()
                        },
                        offset: 0,
                        limit: 100,
                        sort: None,
                        group_by_month: false,
                    },
                )
                .expect("the sale book")
            },
        ));
    }
    rows.push(measure(
        "partner.capital_where",
        nfr,
        "cashbook_capital_where (the card on روکڑ)",
        1_000.0,
        || cashbook::capital_where(ctx).expect("the card"),
    ));
    finish(&rows);
}

/// NFR-SPD-6 (picker search over 20,000 accounts, with spelling mistakes, both names) and NFR-SPD-20
/// (English to Urdu suggestions), on the full data. The search index is built from the 20,030
/// accounts of the file.
#[test]
#[ignore = "needs the full volume and a release build: see robustness_gate_t2.rs"]
fn t2_10_search_and_translit() {
    use mandi_services::core::accounts::{AccountScope, SearchAccountsInput};
    use mandi_services::core::translit::Direction;
    use mandi_services::{search, translit};
    use std::cell::Cell;

    let gate = volume::gate_volume();
    let (env, actor) = support::open(&gate.dir);
    let ctx = &*env.ctx;
    let mut rng = support::Rng(77);
    // 64 texts of each kind, taken from the names the file was made with.
    let texts = |kind: &str, rng: &mut support::Rng| -> Vec<String> {
        (0..64)
            .map(|_| {
                let (ur, en) =
                    support::account_names(u32::try_from(rng.below(20_000)).unwrap_or(0));
                match kind {
                    "en_start" => en.chars().take(6).collect(),
                    "en_typo" => {
                        let mut c: Vec<char> = en.chars().take(8).collect();
                        if c.len() > 3 {
                            c.remove(3);
                        }
                        c.into_iter().collect()
                    }
                    "ur_start" => ur.chars().take(5).collect(),
                    "ur_typo" => {
                        let mut c: Vec<char> = ur.chars().take(7).collect();
                        if c.len() > 3 {
                            c.remove(3);
                        }
                        c.into_iter().collect()
                    }
                    "one_letter" => en.chars().take(1).collect(),
                    _ => "zzqxj".to_owned(),
                }
            })
            .collect()
    };
    let mut rows: Vec<Row> = Vec::new();
    for kind in [
        "en_start",
        "en_typo",
        "ur_start",
        "ur_typo",
        "one_letter",
        "no_match",
    ] {
        let list = texts(kind, &mut rng);
        let next = Cell::new(0_usize);
        rows.push(measure(
            &format!("search.{kind}"),
            "NFR-SPD-6",
            &format!("picker search over 20,030 accounts, {kind}"),
            50.0,
            || {
                let i = next.get();
                next.set(i + 1);
                let text = list.get(i % list.len()).cloned().unwrap_or_default();
                search::search_accounts(
                    ctx,
                    &actor,
                    &SearchAccountsInput {
                        include_dead: None,
                        scope: AccountScope::Any,
                        control_id: None,
                        text,
                        limit: 20,
                        offset: 0,
                        with_chukta: None,
                    },
                )
                .expect("a search")
            },
        ));
    }
    let words = [
        "arshad",
        "arshad multani",
        "akram toll tax",
        "muhammad",
        "shahid",
        "ch",
        "sheikh",
    ];
    let next = Cell::new(0_usize);
    rows.push(measure(
        "translit.to_urdu",
        "NFR-SPD-20",
        "translit_suggest, English letters to Urdu, one key press",
        50.0,
        || {
            let i = next.get();
            next.set(i + 1);
            translit::suggest(
                ctx,
                words.get(i % words.len()).copied().unwrap_or("ali"),
                Direction::ToUrdu,
            )
            .expect("suggestions")
        },
    ));
    finish(&rows);
}
