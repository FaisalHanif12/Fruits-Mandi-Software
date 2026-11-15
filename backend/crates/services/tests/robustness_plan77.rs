//! Plan 77 end gate: the analytics views with day, month and year grouping and a range (A to M), view L (buyer by
//! item), view M (shop and partner kharch, which is also the kharch page), the drill lines behind them, and
//! the commission page with a party and a period (PRD FR-9.3, FR-9.5, FR-18.1, FR-18.2, FR-18.9, FR-17.11;
//! NFR-SPD-24; Code Standards 4.1 V0, B31). Owner: `robustness`.
//!
//! Part A (every run, small data): every view with every grouping and filter set either answers or refuses
//! with a code; bad dates, groupings, parts, parties and page numbers are refused and never end in a data or
//! database error; the lines behind view M cannot reach a partner's kharch under the shop part.
//!
//! Part B (only with `MANDI_PERF_VOLUME=1`, release build): the same reads on the 10-year file that
//! `robustness_plan76` keeps in `backend/target/perf/nfr_scl1_plan76`, plus 20 expense heads and 24,000 shop
//! expense entries made here by plain SQL. Oracles: the view M totals equal plain SQL on the journal, the
//! partner part equals the drawings of the partner, and the lines behind a cell add up to the cell.
//!
//! `MANDI_PERF_VOLUME=1 cargo test --release -p mandi-services --test robustness_plan77 -- --nocapture`
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
// The file already carries `#![allow(dead_code)]` at its top.
#[path = "../benches/support/mod.rs"]
mod volume;

use std::time::{Duration, Instant};

use common::set_today;
use mandi_reports::ReportsError;
use mandi_reports::analytics::{
    AnalyticsExpensePart, AnalyticsFilters, AnalyticsGroupBy, AnalyticsKey, AnalyticsLinesInput,
    AnalyticsPartnerMoney, AnalyticsRunInput, AnalyticsView, lines, run,
};
use mandi_reports::dashboard::{
    CommissionMonthInput, CommissionPeriod, CommissionPeriodKind, commission_month,
};
use mandi_reports::reports::{Cell, PartyFilter};
use mandi_services::bikri;
use mandi_services::core::bikri::{BikriSaveInput, CancelInput};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::masters::{ExpenseHeadInput, MasterInput, MasterRow};
use mandi_services::core::paging::{SortDirection, SortInput};
use mandi_services::db::rusqlite::Connection;
use mandi_services::{ServiceError, masters};
use support::World;

const ALL_VIEWS: [AnalyticsView; 13] = [
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
];

fn refusal(error: &ReportsError) -> bool {
    matches!(error, ReportsError::Rejected(_) | ReportsError::NotFound)
}

fn filters(from: &str, to: &str, group_by: Option<AnalyticsGroupBy>) -> AnalyticsFilters {
    AnalyticsFilters {
        from: from.to_owned(),
        to: to.to_owned(),
        group_by,
        ..AnalyticsFilters::default()
    }
}

fn run_input(
    view: AnalyticsView,
    filters: AnalyticsFilters,
    offset: i64,
    limit: i64,
) -> AnalyticsRunInput {
    AnalyticsRunInput {
        view,
        filters,
        offset,
        limit,
        sort: None,
    }
}

fn lines_input(
    view: AnalyticsView,
    filters: AnalyticsFilters,
    row_key: AnalyticsKey,
) -> AnalyticsLinesInput {
    AnalyticsLinesInput {
        view,
        filters,
        row_key,
        column_key: None,
        offset: 0,
        limit: 100,
    }
}

/// The last cell of a totals row as paisa.
fn last_money(cells: &[Cell]) -> Option<i64> {
    match cells.last() {
        Some(Cell::Money { paisa }) => Some(paisa.as_paisa()),
        _ => None,
    }
}

/// Every cell of a totals row, as text, to tell two totals rows apart.
fn totals_text(cells: &[Cell]) -> String {
    format!("{cells:?}")
}

/// E2E-16 (45 of 50 sold), today 11-10-2026: some sale facts and share rows for the views to read.
fn world_with_sales() -> World {
    let w = support::world();
    let draft = support::draft(vec![support::with_sales(
        support::lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![
            support::sale("s-1", w.akram, "20", "600"),
            support::sale("s-2", w.arshad, "25", "580"),
        ],
    )]);
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    w
}

// ---- part A ---------------------------------------------------------------------------------------------

#[test]
fn every_view_takes_every_grouping_and_filter_set_or_refuses_it_with_a_code() {
    let w = world_with_sales();
    let db = w.env.ctx.db();
    let groupings = [
        None,
        Some(AnalyticsGroupBy::Day),
        Some(AnalyticsGroupBy::Month),
        Some(AnalyticsGroupBy::Year),
    ];
    let mut answered = 0;
    let mut refused = Vec::new();
    for view in ALL_VIEWS {
        for group_by in groupings {
            for set in 0..6 {
                let mut f = filters("01-10-2026", "11-10-2026", group_by);
                match set {
                    1 => f.item_id = Some(w.banana),
                    2 => f.seller_id = Some(w.rahim),
                    3 => {
                        f.buyer = Some(PartyFilter {
                            account_id: w.akram,
                            person_key: None,
                        });
                    }
                    4 => {
                        f.item_id = Some(w.banana);
                        f.buyer = Some(PartyFilter {
                            account_id: w.akram,
                            person_key: None,
                        });
                    }
                    5 => {
                        f.item_id = Some(w.banana);
                        f.seller_id = Some(w.rahim);
                    }
                    _ => {}
                }
                match run(db, &run_input(view, f, 0, 100)) {
                    Ok(page) => {
                        answered += 1;
                        assert!(
                            i64::try_from(page.rows.len()).unwrap() <= 100,
                            "{view:?} {group_by:?} set {set}: {} rows for a limit of 100",
                            page.rows.len()
                        );
                    }
                    Err(error) => {
                        assert!(
                            refusal(&error),
                            "{view:?} {group_by:?} set {set}: not a refusal but {error:?}"
                        );
                        refused.push(format!("{view:?} {group_by:?} set {set}"));
                    }
                }
            }
        }
    }
    println!(
        "SWEEP {answered} answers, {} refusals: {:?}",
        refused.len(),
        refused
    );
    assert!(answered > 100, "only {answered} combinations answered");
}

#[test]
fn bad_periods_groupings_parts_parties_and_page_numbers_are_refused_and_never_a_data_error() {
    let w = world_with_sales();
    let db = w.env.ctx.db();
    let long = "9".repeat(10_000);
    // Dates: each of these is refused on view A and on view M.
    for view in [AnalyticsView::A, AnalyticsView::M] {
        for (from, to) in [
            ("31-02-2026", "11-10-2026"),
            ("01-10-2026", "31-02-2026"),
            ("abc", "11-10-2026"),
            ("01-10-2026", "abc"),
            ("11-10-2026", "01-10-2026"),
            ("'; DROP TABLE analytics_facts; --", "11-10-2026"),
            (long.as_str(), long.as_str()),
            ("99-99-9999", "99-99-9999"),
        ] {
            let f = filters(from, to, Some(AnalyticsGroupBy::Month));
            let result = run(db, &run_input(view, f, 0, 100));
            match result {
                Ok(_) => panic!("{view:?}: the period {from:.20?} to {to:.20?} was accepted"),
                Err(error) => assert!(
                    refusal(&error),
                    "{view:?}: period {from:.20?} to {to:.20?}: {error:?}"
                ),
            }
        }
    }
    // Page numbers.
    for (offset, limit) in [(-1, 100), (0, -1), (0, 1_001), (0, i64::MAX), (i64::MIN, 0)] {
        let result = run(
            db,
            &run_input(
                AnalyticsView::A,
                filters("01-10-2026", "11-10-2026", None),
                offset,
                limit,
            ),
        );
        match result {
            Ok(_) => panic!("offset {offset} limit {limit} was accepted"),
            Err(error) => assert!(refusal(&error), "offset {offset} limit {limit}: {error:?}"),
        }
    }
    let far = run(
        db,
        &run_input(
            AnalyticsView::A,
            filters("01-10-2026", "11-10-2026", None),
            i64::MAX / 2,
            100,
        ),
    );
    match far {
        Ok(page) => assert!(page.rows.is_empty(), "an offset past the end gave rows"),
        Err(error) => assert!(refusal(&error), "{error:?}"),
    }
    // A column the view does not have, as the sort: refused with a code, nothing run.
    for column in [
        "evil; DROP TABLE analytics_facts",
        "",
        "\u{0}",
        long.as_str(),
    ] {
        for view in [
            AnalyticsView::A,
            AnalyticsView::C,
            AnalyticsView::L,
            AnalyticsView::M,
        ] {
            let mut input = run_input(view, filters("01-10-2026", "11-10-2026", None), 0, 100);
            input.sort = Some(SortInput {
                column: column.to_owned(),
                direction: SortDirection::Desc,
                language: None,
            });
            match run(db, &input) {
                Ok(_) => panic!("{view:?}: the sort column {column:.20?} was accepted"),
                Err(error) => assert!(
                    refusal(&error),
                    "{view:?}: sort column {column:.20?}: {error:?}"
                ),
            }
        }
    }
    // A matrix view counts the periods that have data and refuses more than 36 of them. This book has data on
    // one day, so day grouping over 10 years answers here; the ten-year test checks the refusal on real data.
    for view in [AnalyticsView::D, AnalyticsView::M] {
        let f = filters("11-10-2016", "11-10-2026", Some(AnalyticsGroupBy::Day));
        if let Err(error) = run(db, &run_input(view, f, 0, 100)) {
            assert!(refusal(&error), "{view:?}: {error:?}");
        }
    }
    // The part of view M on a view that has none, and a partner that does not exist.
    let mut on_a = filters("01-10-2026", "11-10-2026", None);
    on_a.expense_part = Some(AnalyticsExpensePart::Partner);
    match run(db, &run_input(AnalyticsView::A, on_a, 0, 100)) {
        Ok(_) => panic!("expense_part was accepted on view A"),
        Err(error) => assert!(refusal(&error), "{error:?}"),
    }
    for partner_id in [0, -1, i64::MAX, 999_999] {
        let mut f = filters("01-10-2026", "11-10-2026", None);
        f.expense_part = Some(AnalyticsExpensePart::Partner);
        f.partner_id = Some(partner_id);
        match run(db, &run_input(AnalyticsView::M, f, 0, 100)) {
            Ok(page) => {
                println!(
                    "LENIENT: view M partner part with partner {partner_id} answered {} rows",
                    page.rows.len()
                );
                assert!(page.rows.is_empty(), "partner {partner_id} gave rows");
            }
            Err(error) => assert!(refusal(&error), "partner {partner_id}: {error:?}"),
        }
    }
    // Parties of the wrong kind or no kind, on the views that take a buyer. (نقدی منڈی is a real buyer account,
    // so it is not in this list.)
    let odd_buyers = [w.rahim, 0, -1, i64::MAX, support::id_of(&w.env, "01010001")];
    for account_id in odd_buyers {
        let mut f = filters("01-10-2026", "11-10-2026", None);
        f.buyer = Some(PartyFilter {
            account_id,
            person_key: None,
        });
        match run(db, &run_input(AnalyticsView::C, f, 0, 100)) {
            Ok(page) => {
                println!(
                    "LENIENT: view C with buyer {account_id} (not a buyer) answered {} rows",
                    page.rows.len()
                );
                assert!(page.rows.is_empty(), "a non-buyer {account_id} gave rows");
            }
            Err(error) => assert!(refusal(&error), "buyer {account_id}: {error:?}"),
        }
    }
    let mut keyed = filters("01-10-2026", "11-10-2026", None);
    keyed.buyer = Some(PartyFilter {
        account_id: w.akram,
        person_key: Some("x'; DROP TABLE analytics_facts; --".into()),
    });
    match run(db, &run_input(AnalyticsView::C, keyed, 0, 100)) {
        Ok(page) => assert!(
            page.rows.is_empty(),
            "a person key on an ordinary buyer gave rows"
        ),
        Err(error) => assert!(refusal(&error), "{error:?}"),
    }
    for item_id in [0, -1, i64::MAX] {
        let mut f = filters("01-10-2026", "11-10-2026", None);
        f.item_id = Some(item_id);
        if let Err(error) = run(db, &run_input(AnalyticsView::L, f, 0, 100)) {
            assert!(refusal(&error), "item {item_id}: {error:?}");
        }
    }
}

#[test]
fn the_lines_behind_view_m_cannot_reach_a_partners_kharch_under_the_shop_part() {
    let w = world_with_sales();
    let db = w.env.ctx.db();
    let shop = filters("01-10-2026", "11-10-2026", Some(AnalyticsGroupBy::Month));
    // A key that asks for the partner's own money under the shop part is refused.
    for money in [
        AnalyticsPartnerMoney::Taken,
        AnalyticsPartnerMoney::Trips,
        AnalyticsPartnerMoney::Gardens,
        AnalyticsPartnerMoney::Fruit,
    ] {
        let key = AnalyticsKey {
            money: Some(money),
            partner_id: Some(w.partners[0]),
            ..AnalyticsKey::default()
        };
        match lines(db, &lines_input(AnalyticsView::M, shop.clone(), key)) {
            Ok(page) => panic!(
                "the shop part opened a partner's {money:?} lines: {} rows",
                page.rows.len()
            ),
            Err(error) => assert!(refusal(&error), "{money:?}: {error:?}"),
        }
    }
    // The money field on other views is refused too.
    let key = AnalyticsKey {
        money: Some(AnalyticsPartnerMoney::Taken),
        ..AnalyticsKey::default()
    };
    match lines(db, &lines_input(AnalyticsView::A, shop.clone(), key)) {
        Ok(_) => panic!("row_key.money was accepted on view A"),
        Err(error) => assert!(refusal(&error), "{error:?}"),
    }
    let guests = AnalyticsKey {
        guests: Some(true),
        ..AnalyticsKey::default()
    };
    match lines(db, &lines_input(AnalyticsView::M, shop.clone(), guests)) {
        Ok(_) => panic!("row_key.guests was accepted on view M"),
        Err(error) => assert!(refusal(&error), "{error:?}"),
    }
    // Unknown heads and hostile limits.
    for head_id in [0, -1, i64::MAX] {
        let key = AnalyticsKey {
            head_id: Some(head_id),
            ..AnalyticsKey::default()
        };
        if let Err(error) = lines(db, &lines_input(AnalyticsView::M, shop.clone(), key)) {
            assert!(refusal(&error), "head {head_id}: {error:?}");
        }
    }
    for (offset, limit) in [(-1, 100), (0, -1), (0, 1_001), (0, i64::MAX)] {
        let mut input = lines_input(AnalyticsView::M, shop.clone(), AnalyticsKey::default());
        input.offset = offset;
        input.limit = limit;
        match lines(db, &input) {
            Ok(_) => panic!("lines: offset {offset} limit {limit} was accepted"),
            Err(error) => assert!(refusal(&error), "offset {offset} limit {limit}: {error:?}"),
        }
    }
}

#[test]
fn a_save_or_a_cancel_between_page_1_and_page_2_changes_the_answer() {
    // The next page of the same filter comes from a kept result (result_cache). It must never be older than
    // the books: a save, then a cancel, between the two pages gives pages that agree with the new books.
    let w = world_with_sales();
    let db = w.env.ctx.db();
    let f = filters("01-10-2026", "11-10-2026", None);
    let page = |offset: i64| run(db, &run_input(AnalyticsView::A, f.clone(), offset, 1)).unwrap();
    let range = CommissionPeriod {
        kind: CommissionPeriodKind::Range,
        from: Some("01-10-2026".into()),
        to: Some("11-10-2026".into()),
        ..CommissionPeriod::default()
    };
    let commission_page = |offset: u32| {
        commission_month(
            db,
            &CommissionMonthInput {
                month: None,
                period: Some(range.clone()),
                party: None,
                offset: Some(offset),
                limit: Some(1),
            },
        )
        .unwrap()
    };
    let first = page(0);
    let first_total = totals_text(&first.totals);
    let first_items = first.total_count;
    let first_commission = commission_page(0);
    assert_eq!(first_items, 1, "one item sold so far");
    // A save: کریم's 10 سیب to اکرم at 1,200.
    let draft = support::draft(vec![support::with_sales(
        support::lot(&w, "lot-2", w.karim, w.apple, "10"),
        vec![support::sale("s-3", w.akram, "10", "1200")],
    )]);
    let saved = bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // Page 2 of the SAME filter, asked after the save, is the new books, not the old result.
    let second = page(1);
    assert_eq!(second.total_count, 2, "page 2 still counts the old books");
    assert_ne!(
        totals_text(&second.totals),
        first_total,
        "page 2 still has the old totals row"
    );
    assert_eq!(second.rows.len(), 1, "page 2 has the second item");
    // And page 1 again agrees with page 2 (one set of books).
    let again = page(0);
    assert_eq!(again.total_count, second.total_count);
    assert_eq!(totals_text(&again.totals), totals_text(&second.totals));
    let second_commission = commission_page(1);
    assert_ne!(
        second_commission.totals.total_comm_paisa, first_commission.totals.total_comm_paisa,
        "the commission range still has the old totals"
    );
    // Both sales are dated 10-10-2026, so the range still has one day: page 2 is empty but has the totals.
    assert_eq!(second_commission.total_count, 1);
    assert!(second_commission.rows.is_empty());
    // A cancel: the totals go back to the first books, on page 1 and on page 2 alike.
    let version = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: saved.bikri_id,
            version,
            reason: "entered twice".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let after_cancel_two = page(1);
    let after_cancel_one = page(0);
    assert_eq!(
        totals_text(&after_cancel_one.totals),
        first_total,
        "the cancel is not in page 1"
    );
    assert_eq!(
        totals_text(&after_cancel_two.totals),
        first_total,
        "the cancel is not in page 2"
    );
    assert_eq!(
        commission_page(1).totals.total_comm_paisa,
        first_commission.totals.total_comm_paisa,
        "the cancel is not in the commission range"
    );
}

// ---- part B: ten years -----------------------------------------------------------------------------------

fn p95(mut v: Vec<Duration>) -> Duration {
    v.sort();
    let i = ((v.len() as f64) * 0.95).ceil() as usize;
    v[i.saturating_sub(1).min(v.len() - 1)]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1_000.0
}

/// `MANDI_PERF_ONLY=analytics,kharch` runs only the rows whose name has one of the words.
fn selected(name: &str) -> bool {
    match std::env::var("MANDI_PERF_ONLY") {
        Ok(list) if !list.trim().is_empty() => list
            .split(',')
            .any(|part| name.to_lowercase().contains(&part.trim().to_lowercase())),
        _ => true,
    }
}

/// The one-minute load average and the CPU speed now, for the log (the Mac is shared).
fn machine_now() -> String {
    let load = std::process::Command::new("uptime")
        .output()
        .map(|o| {
            let text = String::from_utf8_lossy(&o.stdout).to_string();
            text.split("load averages:")
                .nth(1)
                .unwrap_or("?")
                .trim()
                .to_owned()
        })
        .unwrap_or_else(|_| "?".to_owned());
    // The fixed money-shaped workload of core/benches/calibration.rs: 139 us when the Mac is idle.
    let mut state: u64 = 0x5EED_0000_0001;
    let mut items: Vec<(i128, i128)> = Vec::with_capacity(10_000);
    for _ in 0..10_000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let rupees = i128::from((state >> 16) % 1_000_000_000);
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        items.push((rupees * 100, i128::from((state >> 16) % 10_001)));
    }
    let mut best = f64::MAX;
    for _ in 0..20 {
        let started = Instant::now();
        let mut parts: Vec<i128> = Vec::with_capacity(items.len());
        let mut total: i128 = 0;
        for &(paisa, bps) in &items {
            let rounded = (paisa * bps + 5_000) / 10_000;
            total += rounded;
            parts.push(rounded);
        }
        parts.sort_unstable();
        std::hint::black_box((total, parts.get(5_000).copied()));
        best = best.min(started.elapsed().as_secs_f64() * 1_000_000.0);
    }
    format!("load {load}, cpu calibration {best:.0} us (idle 139 us)")
}

type Table = Vec<(String, Duration, Duration)>;

/// One warm-up, then 9 timed runs (`MANDI_PERF_RUNS` changes it); the p95. Skipped by `MANDI_PERF_ONLY`.
fn timed(table: &mut Table, name: String, budget: Duration, f: &mut dyn FnMut()) {
    if !selected(&name) {
        return;
    }
    f();
    let runs = std::env::var("MANDI_PERF_RUNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(9)
        .max(1);
    let took = p95((0..runs)
        .map(|_| {
            let started = Instant::now();
            f();
            started.elapsed()
        })
        .collect());
    println!("RAW {name}: p95 {:.1} ms vs {:.0} ms", ms(took), ms(budget));
    println!("MACHINE while measuring {name}: {}", machine_now());
    table.push((name, took, budget));
}

fn scalar(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

/// 20 expense heads (account control 0501) made through the service, then 24,000 shop expense entries over
/// 10 years by plain SQL. Returns the ids of the head accounts.
fn make_expense_data(
    env: &mandi_services::testing::TestEnv,
    owner: &mandi_services::auth::Actor,
    path: &std::path::Path,
) {
    let names = [
        ("بجلی", "Electricity"),
        ("چائے", "Tea"),
        ("کرایہ دکان", "Shop Rent"),
        ("تنخواہ", "Salaries"),
        ("پانی", "Water"),
        ("گیس", "Gas"),
        ("ٹیلیفون", "Telephone"),
        ("مرمت", "Repair"),
        ("صفائی", "Cleaning"),
        ("ٹرانسپورٹ", "Transport"),
        ("ٹیکس", "Tax"),
        ("ایندھن", "Fuel"),
        ("چندہ", "Donation"),
        ("سجاوٹ", "Decoration"),
        ("بیمہ", "Insurance"),
        ("قانونی", "Legal"),
        ("بینک", "Bank"),
        ("پرنٹنگ", "Printing"),
        ("پانی ٹینکی", "Water Tank"),
        ("متفرق", "Misc"),
    ];
    let mut accounts: Vec<i64> = Vec::new();
    for (ur, en) in names {
        let mut words: Vec<String> = Vec::new();
        'attempt: for _ in 0..5 {
            let saved = masters::save(
                &env.ctx,
                owner,
                &MasterInput::ExpenseHead(ExpenseHeadInput {
                    id: None,
                    version: None,
                    code: String::new(),
                    name_ur: ur.to_owned(),
                    name_en: en.to_owned(),
                    confirmed_rules_words: words.clone(),
                }),
            );
            match saved {
                Ok(MasterRow::ExpenseHead(row)) => {
                    accounts.push(row.account_id);
                    break 'attempt;
                }
                Ok(_) => break 'attempt,
                Err(ServiceError::Rejected(items)) => {
                    for item in items {
                        if let Some(ErrorValue::Text(found)) = item.values.get("words") {
                            words.extend(found.split(' ').map(str::to_owned));
                        } else if item.code != ErrorCode::ErrSpellingUnconfirmed {
                            println!("NOTE expense head {en} refused: {:?}", item.code);
                            break 'attempt;
                        }
                    }
                }
                Err(other) => panic!("expense head {en}: {other:?}"),
            }
        }
    }
    assert!(
        accounts.len() >= 10,
        "only {} expense heads were made",
        accounts.len()
    );
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(
        "PRAGMA foreign_keys = OFF; PRAGMA synchronous = OFF; PRAGMA busy_timeout = 120000;
         CREATE TEMP TABLE perf_h (idx INTEGER PRIMARY KEY, account_id INTEGER, head_id INTEGER);",
    )
    .unwrap();
    for (idx, account) in accounts.iter().enumerate() {
        conn.execute(
            "INSERT INTO perf_h (idx, account_id, head_id) VALUES (?1, ?2, (SELECT id FROM expense_heads WHERE account_id = ?2))",
            (idx as i64, account),
        )
        .unwrap();
    }
    let heads = i64::try_from(accounts.len()).unwrap();
    let cash = scalar(&conn, "SELECT id FROM accounts WHERE code = '01010001'");
    let base = scalar(&conn, "SELECT COALESCE(MAX(id), 0) FROM journal_entries");
    let started = Instant::now();
    conn.execute_batch(&format!(
        "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 24000)
         INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, created_by, updated_by)
         SELECT date('2016-10-12', '+' || ((i * 3600) / 24000) || ' days'), 'shop_expense', 'voucher', 0, 1, 1 FROM n;
         INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, created_by, updated_by)
         SELECT e.id, 1, h.account_id, (500 + ((e.id - {base}) * 37) % 9500) * 100, 0, 1, 1
         FROM journal_entries e JOIN perf_h h ON h.idx = (e.id - {base}) % {heads}
         WHERE e.id > {base} AND e.id <= {base} + 24000
         UNION ALL
         SELECT e.id, 2, {cash}, 0, (500 + ((e.id - {base}) * 37) % 9500) * 100, 1, 1
         FROM journal_entries e WHERE e.id > {base} AND e.id <= {base} + 24000;
         INSERT INTO vouchers (type, number, entry_date, amount_paisa, mode, expense_head_id, current_entry_id, status)
         SELECT 'shop_expense', e.id - {base}, e.entry_date, (500 + ((e.id - {base}) * 37) % 9500) * 100, 'cash',
                h.head_id, e.id, 'saved'
         FROM journal_entries e JOIN perf_h h ON h.idx = (e.id - {base}) % {heads}
         WHERE e.id > {base} AND e.id <= {base} + 24000;
         UPDATE journal_entries SET status = 'posted', chain_hash = printf('%064x', id),
                total_paisa = (SELECT SUM(dr_paisa) FROM journal_lines l WHERE l.entry_id = journal_entries.id)
         WHERE id > {base} AND id <= {base} + 24000;"
    ))
    .unwrap();
    let broken = scalar(&conn, "SELECT COUNT(*) FROM pragma_foreign_key_check");
    assert_eq!(broken, 0, "the expense data made foreign key problems");
    println!(
        "GEN 24,000 shop expense entries on {} heads: {:.1} s",
        accounts.len(),
        started.elapsed().as_secs_f64()
    );
}

#[test]
fn the_views_and_the_kharch_page_on_ten_years_against_nfr_spd_24() {
    if std::env::var("MANDI_PERF_VOLUME").is_err() {
        println!("MANDI_PERF_VOLUME is not set: the ten-year check of plan 77 is skipped");
        return;
    }
    let v = volume::ensure_volume();
    let kept = v.dir.with_file_name("nfr_scl1_plan76");
    let made = std::fs::read_to_string(kept.join("plan76.ok")).unwrap_or_default();
    if made.trim() != "plan76 v2" {
        println!(
            "NOTE the file of robustness_plan76 is not there: run its ten-year test first (it makes the file)"
        );
        return;
    }
    let started = Instant::now();
    let probe = mandi_services::testing::TestEnv::new(&kept).unwrap();
    println!(
        "OPEN the 10-year file, TestEnv::new only (no sign-in): {:.1} s",
        started.elapsed().as_secs_f64()
    );
    drop(probe);
    let started = Instant::now();
    let (env, actor) = volume::open(&kept);
    println!(
        "OPEN the 10-year file with sign-in: {:.1} s",
        started.elapsed().as_secs_f64()
    );
    let path = kept.join("data").join("mandi.db");
    if !kept.join("plan77.ok").exists() {
        make_expense_data(&env, &actor, &path);
        std::fs::write(kept.join("plan77.ok"), "plan77 v1").unwrap();
    }
    let db = env.ctx.db();
    let oracle = Connection::open(&path).unwrap();
    let partner_a = scalar(&oracle, "SELECT MIN(id) FROM partners");
    let first = Duration::from_millis(1_000);
    // NFR v1.19, NFR-SPD-24 (Umar, 15-11-2026): the first page of views A to E, H and L over a period longer
    // than one year may take up to 4 seconds at the NFR-SCL-1 volume; each next page stays under 300 ms.
    let first_long = Duration::from_millis(4_000);
    let next = Duration::from_millis(300);
    let mut table: Table = Vec::new();

    // -- the oracles for view M.
    let shop_oracle = |from: &str, to: &str| -> i64 {
        oracle
            .query_row(
                "SELECT COALESCE(SUM(l.dr_paisa - l.cr_paisa), 0) FROM journal_lines l
                 JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted' AND e.source_type <> 'month_close'
                 JOIN accounts a ON a.id = l.account_id JOIN account_controls c ON c.id = a.control_id
                 JOIN account_heads h ON h.id = c.head_id
                 WHERE h.code = '05' AND e.entry_date >= ?1 AND e.entry_date <= ?2",
                [from, to],
                |r| r.get(0),
            )
            .unwrap()
    };
    let drawings_oracle = |from: &str, to: &str| -> i64 {
        oracle
            .query_row(
                &format!(
                    "SELECT COALESCE(SUM(CASE WHEN e.reverses_entry_id IS NULL THEN l.dr_paisa ELSE -l.cr_paisa END), 0)
                     FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id AND e.status = 'posted'
                     WHERE l.account_id = (SELECT drawings_account_id FROM partners WHERE id = {partner_a})
                       AND e.entry_date >= ?1 AND e.entry_date <= ?2"
                ),
                [from, to],
                |r| r.get(0),
            )
            .unwrap()
    };
    let shop = |from: &str, to: &str, group_by: Option<AnalyticsGroupBy>| {
        run_input(AnalyticsView::M, filters(from, to, group_by), 0, 100)
    };
    let partner = |from: &str, to: &str, group_by: Option<AnalyticsGroupBy>| {
        let mut f = filters(from, to, group_by);
        f.expense_part = Some(AnalyticsExpensePart::Partner);
        run_input(AnalyticsView::M, f, 0, 100)
    };
    // Month 5 years back, in the two date forms the screen sends.
    for (name, input, from, to, is_partner) in [
        (
            "shop kharch, 10 years by year",
            shop("11-10-2016", "10-10-2026", Some(AnalyticsGroupBy::Year)),
            "2016-10-11",
            "2026-10-10",
            false,
        ),
        (
            "shop kharch, 2026 by month",
            shop("01-01-2026", "10-10-2026", Some(AnalyticsGroupBy::Month)),
            "2026-01-01",
            "2026-10-10",
            false,
        ),
        (
            "shop kharch, October by day",
            shop("01-10-2026", "10-10-2026", Some(AnalyticsGroupBy::Day)),
            "2026-10-01",
            "2026-10-10",
            false,
        ),
        (
            "partner kharch, 10 years by year",
            partner("11-10-2016", "10-10-2026", Some(AnalyticsGroupBy::Year)),
            "2016-10-11",
            "2026-10-10",
            true,
        ),
        (
            "partner kharch, 2026 by month",
            partner("01-01-2026", "10-10-2026", Some(AnalyticsGroupBy::Month)),
            "2026-01-01",
            "2026-10-10",
            true,
        ),
    ] {
        let page = run(db, &input).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let total = last_money(&page.totals);
        let expected = if is_partner {
            drawings_oracle(from, to)
        } else {
            shop_oracle(from, to)
        };
        println!(
            "SHAPE {name}: {} rows, total {total:?} (oracle {expected})",
            page.rows.len()
        );
        assert_eq!(
            total,
            Some(expected),
            "{name}: the totals row is not the journal"
        );
    }
    for (name, input, budget) in [
        (
            "kharch shop 10 years by year",
            shop("11-10-2016", "10-10-2026", Some(AnalyticsGroupBy::Year)),
            first,
        ),
        (
            "kharch shop current month by day",
            shop("01-10-2026", "10-10-2026", Some(AnalyticsGroupBy::Day)),
            first,
        ),
        (
            "kharch shop month 5 years back by day",
            shop("01-10-2021", "31-10-2021", Some(AnalyticsGroupBy::Day)),
            next,
        ),
        (
            "kharch shop year 5 years back by month",
            shop("01-01-2021", "31-12-2021", Some(AnalyticsGroupBy::Month)),
            next,
        ),
        (
            "kharch partner current month by day",
            partner("01-10-2026", "10-10-2026", Some(AnalyticsGroupBy::Day)),
            first,
        ),
        (
            "kharch partner month 5 years back by day",
            partner("01-10-2021", "31-10-2021", Some(AnalyticsGroupBy::Day)),
            next,
        ),
        (
            "kharch partner 10 years by year",
            partner("11-10-2016", "10-10-2026", Some(AnalyticsGroupBy::Year)),
            first,
        ),
    ] {
        timed(&mut table, name.into(), budget, &mut || {
            run(db, &input).unwrap();
        });
    }

    // -- the lines behind a cell of view M add up to the cell.
    let by_year = run(
        db,
        &shop("11-10-2016", "10-10-2026", Some(AnalyticsGroupBy::Year)),
    )
    .unwrap();
    let row = by_year.rows.first().expect("a head row");
    let key = row.key.clone().expect("a key on the row");
    let cell_total = last_money(&row.cells);
    let lines_input_m = lines_input(
        AnalyticsView::M,
        filters("11-10-2016", "10-10-2026", Some(AnalyticsGroupBy::Year)),
        key,
    );
    let drilled = lines(db, &lines_input_m).unwrap();
    println!(
        "SHAPE lines of the first head: {} lines, total {:?}, row total {cell_total:?}",
        drilled.total_count,
        last_money(&drilled.totals)
    );
    assert_eq!(
        last_money(&drilled.totals),
        cell_total,
        "the lines do not add up to the row"
    );
    timed(
        &mut table,
        "kharch lines of one head, 10 years, page 1".into(),
        first,
        &mut || {
            lines(db, &lines_input_m).unwrap();
        },
    );
    let mut second = lines_input_m.clone();
    second.offset = 100;
    timed(
        &mut table,
        "kharch lines of one head, 10 years, page 2".into(),
        next,
        &mut || {
            lines(db, &second).unwrap();
        },
    );
    let partner_lines = lines_input(
        AnalyticsView::M,
        {
            let mut f = filters("11-10-2016", "10-10-2026", None);
            f.expense_part = Some(AnalyticsExpensePart::Partner);
            f
        },
        AnalyticsKey::default(),
    );
    let drawn = lines(db, &partner_lines).unwrap();
    println!("SHAPE partner kharch lines: {} entries", drawn.total_count);
    assert_eq!(
        last_money(&drawn.totals),
        Some(drawings_oracle("2016-10-11", "2026-10-10")),
        "the partner entries do not add up to his drawings"
    );
    timed(
        &mut table,
        "kharch lines of the partner part, 10 years".into(),
        first,
        &mut || {
            lines(db, &partner_lines).unwrap();
        },
    );

    // -- the matrix views with more periods than columns they take (36): refused, and how fast.
    for (view, label, group_by) in [
        (AnalyticsView::D, "by day 10 years", AnalyticsGroupBy::Day),
        (AnalyticsView::M, "by day 10 years", AnalyticsGroupBy::Day),
        (
            AnalyticsView::D,
            "by month 10 years",
            AnalyticsGroupBy::Month,
        ),
        (AnalyticsView::D, "by year 10 years", AnalyticsGroupBy::Year),
    ] {
        let input = run_input(
            view,
            filters("11-10-2016", "10-10-2026", Some(group_by)),
            0,
            100,
        );
        let name = format!("analytics view {view:?} {label} (36 column rule)");
        if !selected(&name) {
            continue;
        }
        match run(db, &input) {
            Ok(page) => println!(
                "SHAPE {name}: answered, {} rows, {} columns",
                page.rows.len(),
                page.columns.len()
            ),
            Err(error) => {
                assert!(refusal(&error), "{name}: not a refusal but {error:?}");
                println!("SHAPE {name}: refused, {error:?}");
            }
        }
        timed(&mut table, name, first_long, &mut || {
            let _ = run(db, &input);
        });
    }
    // -- views A, B, C, E, L: plain, grouped by month and year over 10 years, by day over one year.
    let ten = ("11-10-2016", "10-10-2026");
    let one = ("11-10-2025", "10-10-2026");
    for view in [
        AnalyticsView::A,
        AnalyticsView::B,
        AnalyticsView::C,
        AnalyticsView::E,
        AnalyticsView::L,
    ] {
        for (label, range, group_by) in [
            ("plain 10 years", ten, None),
            ("by year 10 years", ten, Some(AnalyticsGroupBy::Year)),
            ("by month 10 years", ten, Some(AnalyticsGroupBy::Month)),
            ("by day 1 year", one, Some(AnalyticsGroupBy::Day)),
        ] {
            let input = run_input(view, filters(range.0, range.1, group_by), 0, 100);
            // A row the filter leaves out is not even run once (a grouped view can take minutes).
            if !selected(&format!("analytics view {view:?} {label}, page 1"))
                && !selected(&format!("analytics view {view:?} {label}, page 2"))
            {
                continue;
            }
            match run(db, &input) {
                Ok(page) => {
                    println!(
                        "SHAPE view {view:?} {label}: {} rows of {}",
                        page.rows.len(),
                        page.total_count
                    );
                }
                Err(error) => {
                    println!("NOTE view {view:?} {label} refused: {error:?}");
                    continue;
                }
            }
            timed(
                &mut table,
                format!("analytics view {view:?} {label}, page 1"),
                if range == ten { first_long } else { first },
                &mut || {
                    run(db, &input).unwrap();
                },
            );
            let mut page_two = input.clone();
            page_two.offset = 100;
            timed(
                &mut table,
                format!("analytics view {view:?} {label}, page 2"),
                next,
                &mut || {
                    run(db, &page_two).unwrap();
                },
            );
        }
    }
    // -- party and item together on the sale views.
    let mut combined = filters(ten.0, ten.1, Some(AnalyticsGroupBy::Year));
    combined.buyer = Some(PartyFilter {
        account_id: v.busy_buyer,
        person_key: None,
    });
    combined.item_id = Some(scalar(&oracle, "SELECT MIN(id) FROM items"));
    for view in [AnalyticsView::A, AnalyticsView::C, AnalyticsView::L] {
        let input = run_input(view, combined.clone(), 0, 100);
        if run(db, &input).is_ok() {
            timed(
                &mut table,
                format!("analytics view {view:?} busiest buyer and one item, by year"),
                first_long,
                &mut || {
                    run(db, &input).unwrap();
                },
            );
        }
    }

    let mut over = Vec::new();
    for (name, measured, budget) in &table {
        let ok = measured <= budget;
        println!(
            "PERF plan77: {name}: p95 {:.1} ms vs {:.0} ms ({})",
            ms(*measured),
            ms(*budget),
            if ok { "OK" } else { "OVER" }
        );
        if !ok {
            over.push(name.clone());
        }
    }
    if !cfg!(debug_assertions) {
        assert!(over.is_empty(), "over the NFR-SPD-24 budget: {over:?}");
    }
}
