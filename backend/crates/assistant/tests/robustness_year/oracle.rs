//! The oracle of the rich year (plan 52 item 12): the true answers, worked out by the app's own
//! reports and analytics (the very functions the screens call), never typed by hand.
//!
//! The result is a flat map `key -> text`. The text is in the form the scorer of
//! `scripts/ai-eval/run.mjs` reads: whole rupees (`1234`, `-50`), a balance with its side
//! (`2880 Dr`), a count (`12`), a share with a percent sign (`72.10%`), a month (`2026-09`), a date
//! or an English name. The question bank refers to a fact as `@key` in its `figures` column.
//!
//! Key shapes (the full list is in `scripts/ai-eval/oracle-keys.txt`):
//!
//! - `a.<period>.<item|total>.<column>`: analytics view A, by item. `<period>` is one of
//!   [`PERIODS`] or a month `m2026_09`.
//! - `b.<period>.<seller|total>.<column>`, `c.<period>.<buyer|total>.<column>`: views B and C.
//! - `f.<period>.<partner>.<column>`: view F. `g.<yyyy_mm>.<column>`: view G, a row per month.
//! - `buyer_items.<buyer>.<period>.<item|total>.<column>`: view A filtered to one buyer.
//! - `buyer_day.<buyer>.<day>.<item|total>.<column>` and `seller_day.<seller>.<day>...`: a party's day.
//! - `r20.<buyer|total>.<column>` and the same for `r24`, `r26`, `r32`, `r38`: balance lists.
//! - `r12.<period>.<line>`, `r30.<period>.<partner|total>.<column>`, `r29.<period>.<head|total>...`
//! - `tiles.<name>`, `partner.<a|b|c>.<column>`, `top.<what>`: leaders worked out from the rows.
//! - `cal.<name>`: the dates a period in words becomes (today, yesterday, month_start, last_month_start ...).
#![allow(dead_code)]

use std::collections::BTreeMap;

use chrono::{Datelike, Duration, NaiveDate};
use mandi_reports::analytics::{
    self, AnalyticsFilters, AnalyticsPage, AnalyticsRunInput, AnalyticsView,
};
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_reports::reports::{
    self, Cell, Column, ColumnKind, PartyFilter, ReportFilters, ReportId, ReportPage,
    ReportRunInput,
};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::{ledger, partners};

use super::seed::{TODAY, Year, date, ddmmyyyy};

/// The days on which a party's purchases or sales are asked (today, yesterday and three dates, on
/// which the party has two items).
pub const PARTY_DAYS: [&str; 5] = [
    "today",
    "yesterday",
    "d2026_10_14",
    "d2026_09_18",
    "d2026_04_25",
];

/// The dates a period in words becomes (`cal.<name>`, `YYYY-MM-DD`), worked out by the app's OWN
/// period helper (`mandi_assistant::tools::period::compute`, the one behind the `period` tool and
/// the understand step), so the bank expects what the app means: "last week" is the calendar week
/// before this one, Monday to Sunday. Only the two ends of "1 se 15 October" are the Owner's own
/// dates, given as a range.
fn calendar(o: &mut Oracle) {
    use mandi_assistant::tools::period::{Input, compute};
    let today = date(TODAY);
    let day = |d: NaiveDate| d.format("%Y-%m-%d").to_string();
    let range_of = |kind: &str, month: Option<i64>| {
        let input: Input = serde_json::from_value(serde_json::json!({
            "kind": kind, "month": month, "day": null, "from": null, "to": null, "n": null
        }))
        .unwrap_or_else(|e| panic!("period input {kind}: {e}"));
        compute(today, &input, &[]).unwrap_or_else(|e| panic!("period {kind}: {e:?}"))
    };
    let first = |kind: &str, month: Option<i64>| range_of(kind, month).from.expect("a first day");
    let last = |kind: &str, month: Option<i64>| range_of(kind, month).to;
    for (name, value) in [
        ("today", last("today", None)),
        ("yesterday", last("yesterday", None)),
        ("month_start", first("this_month", None)),
        ("last_month_start", first("last_month", None)),
        ("last_month_end", last("last_month", None)),
        ("sep_start", first("month", Some(9))),
        ("sep_end", last("month", Some(9))),
        ("last_week_start", first("last_week", None)),
        ("last_week_end", last("last_week", None)),
        (
            "oct_01",
            NaiveDate::from_ymd_opt(today.year(), 10, 1).unwrap_or(today),
        ),
        (
            "oct_15",
            NaiveDate::from_ymd_opt(today.year(), 10, 15).unwrap_or(today),
        ),
    ] {
        o.put(format!("cal.{name}"), day(value));
    }
}

/// The whole rupees a money cell holds.
fn rupees_of(cell: &Cell) -> Option<i64> {
    match cell {
        Cell::Money { paisa } => Some(paisa.rupees()),
        _ => None,
    }
}

/// For each item row of a buyer's view A and for its total: the value, the buyer commission and
/// the laga added (`with_charges`, C5).
fn with_charges(o: &mut Oracle, prefix: &str, page: &AnalyticsPage) {
    let at = |name: &str| page.columns.iter().position(|c| c.key == name);
    let (Some(gross), Some(comm), Some(laga)) = (at("gross"), at("buyer_comm"), at("laga")) else {
        return;
    };
    let sum_of = |cells: &[Cell]| -> Option<i64> {
        Some(
            rupees_of(cells.get(gross)?)?
                + rupees_of(cells.get(comm)?)?
                + rupees_of(cells.get(laga)?)?,
        )
    };
    for row in &page.rows {
        if let (Some((slug, _)), Some(v)) = (row_slug(&row.cells), sum_of(&row.cells)) {
            o.put(format!("{prefix}.{slug}.with_charges"), v.to_string());
        }
    }
    if let Some(v) = sum_of(&page.totals) {
        o.put(format!("{prefix}.total.with_charges"), v.to_string());
    }
}

/// The periods the bank asks about, with the dates the oracle uses (`from`, `to`).
pub const PERIODS: [&str; 9] = [
    "today",
    "yesterday",
    "week",
    "month",
    "last_month",
    "year",
    "last_year",
    "all",
    "up_to_now",
];

#[derive(Default)]
pub struct Oracle {
    pub facts: BTreeMap<String, String>,
    /// The English name behind each slug (`haji_nazeer` is `Haji Nazeer`), for the leaders.
    pub names: BTreeMap<String, String>,
}

impl Oracle {
    fn put(&mut self, key: String, value: String) {
        self.facts.insert(key, value);
    }
    pub fn get(&self, key: &str) -> &str {
        self.facts
            .get(key)
            .unwrap_or_else(|| panic!("no oracle fact {key}"))
    }
}

/// Lower-case ASCII with `_` between words: `Akram Toll Tax` is `akram_toll_tax`.
pub fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut last_sep = true;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_sep = false;
        } else if !last_sep {
            out.push('_');
            last_sep = true;
        }
    }
    out.trim_matches('_').to_owned()
}

/// A cell as the scorer reads it, or none for a cell with no figure.
fn show(cell: &Cell) -> Option<String> {
    match cell {
        Cell::Money { paisa } => Some(paisa.rupees().to_string()),
        Cell::Balance { paisa, side } => Some(match side {
            BalanceSide::Dr => format!("{} Dr", paisa.rupees()),
            BalanceSide::Cr => format!("{} Cr", paisa.rupees()),
            _ => "0".to_owned(),
        }),
        Cell::Number { value } => Some(value.to_string()),
        Cell::Percent { hundredths } => {
            Some(format!("{}.{:02}%", hundredths / 100, hundredths % 100))
        }
        Cell::Date { iso } => Some(iso.clone()),
        Cell::Month { year, month } => Some(format!("{year}-{month:02}")),
        _ => None,
    }
}

/// The English name a cell holds, if it holds a name.
fn name_of(cell: &Cell) -> Option<String> {
    match cell {
        Cell::Party { party } => Some(party.name_en.clone()),
        Cell::Names { names } => Some(names.en.clone()),
        Cell::Text { text } => Some(text.clone()),
        Cell::Month { year, month } => Some(format!("{year}-{month:02}")),
        _ => None,
    }
}

/// One row key: the first name the row holds, with the name itself.
fn row_slug(cells: &[Cell]) -> Option<(String, String)> {
    cells.iter().find_map(name_of).map(|n| (slug(&n), n))
}

/// Writes every figure of a table: `prefix.<row>.<column>` and `prefix.total.<column>`.
fn dump(
    o: &mut Oracle,
    prefix: &str,
    keys: &[(String, ColumnKind)],
    rows: &[&[Cell]],
    totals: &[Cell],
) {
    for cells in rows {
        let Some((row, name)) = row_slug(cells) else {
            continue;
        };
        o.names.entry(row.clone()).or_insert(name);
        for ((key, _), cell) in keys.iter().zip(cells.iter()) {
            if let Some(v) = show(cell) {
                o.put(format!("{prefix}.{row}.{key}"), v);
            }
        }
    }
    for ((key, _), cell) in keys.iter().zip(totals.iter()) {
        if let Some(v) = show(cell) {
            o.put(format!("{prefix}.total.{key}"), v);
        }
    }
}

fn columns_of(columns: &[Column]) -> Vec<(String, ColumnKind)> {
    columns.iter().map(|c| (c.key.clone(), c.kind)).collect()
}

fn dump_report(o: &mut Oracle, prefix: &str, page: &ReportPage) {
    dump_report_rows(o, prefix, page, true);
}

/// As [`dump_report`]; with `rows` false only the totals and the summary lines are kept (a
/// register whose rows are documents, not records with a name of their own).
fn dump_report_rows(o: &mut Oracle, prefix: &str, page: &ReportPage, rows: bool) {
    let keys = columns_of(&page.columns);
    let rows: Vec<&[Cell]> = if rows {
        page.rows.iter().map(|r| r.cells.as_slice()).collect()
    } else {
        Vec::new()
    };
    dump(o, prefix, &keys, &rows, &page.totals);
    for line in &page.summary {
        if let Some(v) = show(&line.cell) {
            o.put(format!("{prefix}.sum.{}", slug(&line.label_id)), v);
        }
    }
    o.put(format!("{prefix}.count"), page.total_count.to_string());
}

fn dump_analytics(o: &mut Oracle, prefix: &str, page: &AnalyticsPage) {
    let keys: Vec<(String, ColumnKind)> = page
        .columns
        .iter()
        .map(|c| (c.key.clone(), c.kind))
        .collect();
    let rows: Vec<&[Cell]> = page.rows.iter().map(|r| r.cells.as_slice()).collect();
    dump(o, prefix, &keys, &rows, &page.totals);
    o.put(format!("{prefix}.count"), page.total_count.to_string());
}

// ---- the periods -------------------------------------------------------------------------------

fn period_dates(name: &str) -> (Option<NaiveDate>, Option<NaiveDate>) {
    let today = date(TODAY);
    let first_of_month = today.with_day(1).unwrap_or(today);
    let last_of_prev = first_of_month - Duration::days(1);
    match name {
        "today" => (Some(today), Some(today)),
        "yesterday" => (
            Some(today - Duration::days(1)),
            Some(today - Duration::days(1)),
        ),
        "week" => {
            let monday = today - Duration::days(i64::from(today.weekday().num_days_from_monday()));
            (Some(monday), Some(today))
        }
        "month" => (Some(first_of_month), Some(today)),
        "last_month" => (last_of_prev.with_day(1), Some(last_of_prev)),
        "year" => (NaiveDate::from_ymd_opt(today.year(), 1, 1), Some(today)),
        "last_year" => (
            NaiveDate::from_ymd_opt(today.year() - 1, 1, 1),
            NaiveDate::from_ymd_opt(today.year() - 1, 12, 31),
        ),
        // The whole book: from the first day to today.
        "all" | "up_to_now" => (Some(date(super::seed::START)), Some(today)),
        other if other.starts_with('d') => {
            // d2026_10_30: that one day.
            let parts: Vec<i32> = other[1..]
                .split('_')
                .filter_map(|p| p.parse().ok())
                .collect();
            let day = match parts.as_slice() {
                [y, m, d] => NaiveDate::from_ymd_opt(
                    *y,
                    u32::try_from(*m).unwrap_or(0),
                    u32::try_from(*d).unwrap_or(0),
                ),
                _ => None,
            };
            (day, day)
        }
        other => {
            // m2026_09: that whole month.
            let digits: Vec<&str> = other.trim_start_matches('m').split('_').collect();
            let (y, m) = (
                digits.first().and_then(|d| d.parse::<i32>().ok()),
                digits.get(1).and_then(|d| d.parse::<u32>().ok()),
            );
            match (y, m) {
                (Some(y), Some(m)) => {
                    let first = NaiveDate::from_ymd_opt(y, m, 1);
                    let next = if m == 12 {
                        NaiveDate::from_ymd_opt(y + 1, 1, 1)
                    } else {
                        NaiveDate::from_ymd_opt(y, m + 1, 1)
                    };
                    (first, next.map(|n| n - Duration::days(1)))
                }
                _ => (None, None),
            }
        }
    }
}

fn afilters(period: &str) -> AnalyticsFilters {
    let (from, to) = period_dates(period);
    AnalyticsFilters {
        from: from.map(ddmmyyyy).unwrap_or_default(),
        to: to.map(ddmmyyyy).unwrap_or_default(),
        ..AnalyticsFilters::default()
    }
}

fn rfilters(period: &str) -> ReportFilters {
    let (from, to) = period_dates(period);
    ReportFilters {
        from: from.map(ddmmyyyy).unwrap_or_default(),
        to: to.map(ddmmyyyy).unwrap_or_default(),
        ..ReportFilters::default()
    }
}

fn run_a(y: &Year, view: AnalyticsView, filters: AnalyticsFilters) -> AnalyticsPage {
    analytics::run(
        y.w.env.ctx.db(),
        &AnalyticsRunInput {
            view,
            filters,
            offset: 0,
            limit: 100,
            sort: None,
        },
    )
    .unwrap_or_else(|e| panic!("analytics {view:?}: {e:?}"))
}

fn run_r(y: &Year, id: ReportId, filters: ReportFilters) -> ReportPage {
    reports::run(
        y.w.env.ctx.db(),
        &ReportRunInput {
            id,
            filters,
            offset: 0,
            limit: 100,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap_or_else(|e| panic!("report {id:?}: {e:?}"))
}

fn month_slugs() -> Vec<String> {
    let mut out = Vec::new();
    let (mut y, mut m) = (2025, 11);
    while (y, m) <= (2026, 11) {
        out.push(format!("m{y}_{m:02}"));
        if m == 12 {
            y += 1;
            m = 1;
        } else {
            m += 1;
        }
    }
    out
}

// ---- the oracle --------------------------------------------------------------------------------

/// Works out every fact. Every number comes from a report or an analytics view.
pub fn compute(y: &Year) -> Oracle {
    let mut o = Oracle::default();
    let periods: Vec<String> = PERIODS
        .iter()
        .map(|p| (*p).to_owned())
        .chain(month_slugs())
        .chain(["d2026_10_30".to_owned(), "d2026_10_15".to_owned()])
        .collect();

    // Views A: by item, every period and every month.
    for p in &periods {
        dump_analytics(
            &mut o,
            &format!("a.{p}"),
            &run_a(y, AnalyticsView::A, afilters(p)),
        );
    }
    // Views B and C for the big periods.
    for p in ["year", "all", "last_month", "month", "last_year"] {
        dump_analytics(
            &mut o,
            &format!("b.{p}"),
            &run_a(y, AnalyticsView::B, afilters(p)),
        );
        dump_analytics(
            &mut o,
            &format!("c.{p}"),
            &run_a(y, AnalyticsView::C, afilters(p)),
        );
    }
    // View F, partners.
    for p in ["year", "all", "last_month", "month", "today"] {
        dump_analytics(
            &mut o,
            &format!("f.{p}"),
            &run_a(y, AnalyticsView::F, afilters(p)),
        );
    }
    // View G: a row for each month of the year.
    let mut g = afilters("all");
    g.group_by = Some(analytics::AnalyticsGroupBy::Month);
    let page = run_a(y, AnalyticsView::G, g);
    let keys: Vec<(String, ColumnKind)> = page
        .columns
        .iter()
        .map(|c| (c.key.clone(), c.kind))
        .collect();
    for row in &page.rows {
        // The first cell is the period (a month).
        if let Some(Cell::Month { year, month }) = row.cells.first() {
            let slug = format!("{year}_{month:02}");
            for ((key, _), cell) in keys.iter().zip(row.cells.iter()) {
                if let Some(v) = show(cell) {
                    o.put(format!("g.{slug}.{key}"), v);
                }
            }
        }
    }
    // The buyers' purchases by item (Umar's "Faisal ne kya kya khareeda").
    for buyer in &y.buyers {
        if ![
            "faisal",
            "akram_toll_tax",
            "akram_baloch",
            "arshad",
            "imran",
            "nadeem",
        ]
        .contains(&buyer.key)
        {
            continue;
        }
        for p in ["all", "year"] {
            let mut f = afilters(p);
            f.buyer = Some(PartyFilter {
                account_id: buyer.id,
                person_key: None,
            });
            dump_analytics(
                &mut o,
                &format!("buyer_items.{}.{p}", buyer.key),
                &run_a(y, AnalyticsView::A, f),
            );
        }
    }
    // Sellers' goods by item (view A with the seller).
    for seller in &y.sellers {
        let mut f = afilters("year");
        f.seller_id = Some(seller.id);
        dump_analytics(
            &mut o,
            &format!("seller_items.{}.year", seller.key),
            &run_a(y, AnalyticsView::A, f),
        );
    }
    // The calendar of the print questions (plan 54 section 6): the exact dates a period in words
    // becomes, from the oracle's own clock (today is fixed), never typed in the bank.
    calendar(&mut o);
    // A party's day (plan 52 section 8.1): what one buyer bought or one seller sold on one day,
    // each item with its qty and amount, and the day's total. Only a day on which the party has
    // lines is written. A buyer's amount is also given with the commission and laga added (the
    // party day sheet R35 reads the amount that way), as `with_charges`.
    for p in PARTY_DAYS {
        for buyer in &y.buyers {
            let mut f = afilters(p);
            f.buyer = Some(PartyFilter {
                account_id: buyer.id,
                person_key: None,
            });
            let page = run_a(y, AnalyticsView::A, f);
            if page.rows.is_empty() {
                continue;
            }
            let prefix = format!("buyer_day.{}.{p}", buyer.key);
            dump_analytics(&mut o, &prefix, &page);
            with_charges(&mut o, &prefix, &page);
        }
        for seller in &y.sellers {
            let mut f = afilters(p);
            f.seller_id = Some(seller.id);
            let page = run_a(y, AnalyticsView::A, f);
            if page.rows.is_empty() {
                continue;
            }
            dump_analytics(&mut o, &format!("seller_day.{}.{p}", seller.key), &page);
        }
    }
    // Item with a period and one seller are in the views above; gardens, trips, fruit:
    dump_analytics(
        &mut o,
        "i.all",
        &run_a(y, AnalyticsView::I, afilters("all")),
    );
    let j = run_a(y, AnalyticsView::J, afilters("all"));
    dump_analytics(&mut o, "j.all", &j);
    // The two trips, in the order the app lists them: `trip1.<column>`, `trip2.<column>`.
    for (i, row) in j.rows.iter().enumerate() {
        for (c, cell) in j.columns.iter().zip(row.cells.iter()) {
            if let Some(v) = show(cell) {
                o.put(format!("trip{}.{}", i + 1, c.key), v);
            }
        }
    }
    for p in ["all", "year", "last_month"] {
        dump_analytics(
            &mut o,
            &format!("k.{p}"),
            &run_a(y, AnalyticsView::K, afilters(p)),
        );
    }

    // Balance lists, as of today.
    let as_of = ReportFilters {
        as_of: ddmmyyyy(date(TODAY)),
        ..ReportFilters::default()
    };
    for (id, name) in [
        (ReportId::R20, "r20"),
        (ReportId::R24, "r24"),
        (ReportId::R26, "r26"),
        (ReportId::R32, "r32"),
    ] {
        dump_report(&mut o, name, &run_r(y, id, as_of.clone()));
    }
    dump_report(
        &mut o,
        "r38",
        &run_r(y, ReportId::R38, ReportFilters::default()),
    );
    // Reports over a period.
    for p in [
        "today",
        "yesterday",
        "week",
        "month",
        "last_month",
        "year",
        "all",
    ] {
        for (id, name) in [
            (ReportId::R12, "r12"),
            (ReportId::R23, "r23"),
            (ReportId::R25, "r25"),
            (ReportId::R29, "r29"),
            (ReportId::R30, "r30"),
            (ReportId::R37, "r37"),
        ] {
            let rows = matches!(id, ReportId::R29 | ReportId::R30 | ReportId::R37);
            dump_report_rows(
                &mut o,
                &format!("{name}.{p}"),
                &run_r(y, id, rfilters(p)),
                rows,
            );
        }
    }
    for (id, name) in [
        (ReportId::R33, "r33"),
        (ReportId::R36, "r36"),
        (ReportId::R10, "r10"),
    ] {
        let rows = id != ReportId::R10;
        dump_report_rows(&mut o, name, &run_r(y, id, rfilters("all")), rows);
    }
    stock_by_item(&mut o, y);

    // The dashboard's tiles and the partners' totals.
    let now = dashboard::tiles_now(y.w.env.ctx.db()).expect("the tiles");
    let t = &now.tiles;
    let money = |p: mandi_services::core::money::Paisa| p.rupees().to_string();
    o.put("tiles.cash".into(), money(t.cash_in_hand_paisa));
    o.put("tiles.sales_today".into(), money(t.sales_gross_paisa));
    o.put("tiles.count_today".into(), t.sales_count.to_string());
    o.put("tiles.commission_today".into(), money(t.commission_paisa));
    o.put("tiles.buyers_dues".into(), money(t.buyers_dues_paisa));
    o.put(
        "tiles.sellers_payable".into(),
        money(t.sellers_payable_paisa),
    );
    o.put("tiles.sellers_owe".into(), money(t.sellers_dues_paisa));
    o.put("tiles.total_dues".into(), money(t.total_dues_paisa));
    o.put("tiles.dead".into(), money(t.dead_amount_paisa));
    o.put("tiles.dead_count".into(), t.dead_count.to_string());
    o.put("tiles.advances_open".into(), money(t.advances_open_paisa));
    o.put("tiles.stock_lots".into(), t.stock_lots.to_string());
    o.put("tiles.stock_qty".into(), t.stock_qty.to_string());
    o.put("tiles.trips_open".into(), t.trips_open.to_string());
    o.put("tiles.trips_qty_left".into(), t.trips_qty_left.to_string());
    o.put(
        "tiles.trips_cost_left".into(),
        money(t.trips_cost_left_paisa),
    );
    o.put("tiles.gardens_open".into(), t.gardens_open.to_string());
    o.put(
        "tiles.gardens_balance".into(),
        money(t.gardens_balance_paisa),
    );
    o.put("tiles.lent_open".into(), money(t.lent_open_paisa));
    o.put("tiles.lent_overdue".into(), money(t.lent_overdue_paisa));
    o.put("tiles.expenses_today".into(), money(t.expenses_today_paisa));
    let full = dashboard::get(
        y.w.env.ctx.db(),
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .expect("the dashboard");
    for p in &full.partners {
        let k = slug(&p.name_en);
        o.put(format!("partner.{k}.total"), money(p.total_paisa));
        o.put(format!("partner.{k}.today_share"), money(p.today_paisa));
        o.put(format!("partner.{k}.month_share"), money(p.month_paisa));
        o.put(format!("partner.{k}.year_share"), money(p.year_paisa));
    }
    let rows = partners::overview(&y.w.env.ctx).expect("the partner overview");
    for r in &rows {
        let k = slug(&r.name_en);
        o.put(
            format!("partner.{k}.capital"),
            money(r.capital_balance_paisa),
        );
        o.put(format!("partner.{k}.share_bps"), r.share_bps.to_string());
        o.put(
            format!("partner.{k}.month_taken"),
            money(r.month_taken_paisa),
        );
    }

    // Party balances, from the app's own ledger balance (the party card's figure).
    for b in &y.buyers {
        let bal = ledger::account_balance(&y.w.env.ctx, b.id).expect("a balance");
        o.put(
            format!("buyer.{}.balance", b.key),
            match bal.side {
                BalanceSide::Dr => format!("{} Dr", bal.amount_paisa.rupees()),
                BalanceSide::Cr => format!("{} Cr", bal.amount_paisa.rupees()),
                _ => "0".to_owned(),
            },
        );
    }
    for s in &y.sellers {
        let bal = ledger::account_balance(&y.w.env.ctx, s.id).expect("a balance");
        o.put(
            format!("seller.{}.balance", s.key),
            match bal.side {
                BalanceSide::Dr => format!("{} Dr", bal.amount_paisa.rupees()),
                BalanceSide::Cr => format!("{} Cr", bal.amount_paisa.rupees()),
                _ => "0".to_owned(),
            },
        );
    }
    leaders(&mut o);
    derived(&mut o);
    estimates(&mut o, y);
    item_estimates(&mut o, y);
    o
}

/// The unsold goods by item, summed from the rows of R10 (the stock report).
fn stock_by_item(o: &mut Oracle, y: &Year) {
    let page = run_r(y, ReportId::R10, rfilters("all"));
    let item_at = page.columns.iter().position(|c| c.key == "item");
    let balance_at = page.columns.iter().position(|c| c.key == "balance");
    let (Some(item_at), Some(balance_at)) = (item_at, balance_at) else {
        return;
    };
    let mut by_item: BTreeMap<String, i64> = BTreeMap::new();
    let mut lots_by_item: BTreeMap<String, i64> = BTreeMap::new();
    for row in &page.rows {
        let (Some(item), Some(Cell::Number { value })) = (
            row.cells.get(item_at).and_then(name_of),
            row.cells.get(balance_at),
        ) else {
            continue;
        };
        *by_item.entry(slug(&item)).or_default() += value;
        *lots_by_item.entry(slug(&item)).or_default() += 1;
    }
    for (item, qty) in by_item {
        o.put(format!("stock.{item}.qty"), qty.to_string());
    }
    for (item, lots) in lots_by_item {
        o.put(format!("stock.{item}.lots"), lots.to_string());
    }
}

/// Numbers of a fact written as `1234` or `1234 Dr`.
fn amount_of(text: &str) -> Option<i64> {
    text.split_whitespace().next()?.parse::<i64>().ok()
}

/// The rows of a table in the facts, as (slug, amount), the biggest first. A slug is the part of
/// the key between `prefix` and `suffix`; `total`, `count` and `sum` are not rows.
fn ranked(o: &Oracle, prefix: &str, suffix: &str) -> Vec<(String, i64)> {
    let mut rows: Vec<(String, i64)> = o
        .facts
        .iter()
        .filter(|(k, _)| {
            k.starts_with(prefix) && k.ends_with(suffix) && k.len() > prefix.len() + suffix.len()
        })
        .filter_map(|(k, v)| {
            let slug = k[prefix.len()..k.len() - suffix.len()].to_owned();
            (!slug.contains('.') && !["total", "count", "sum"].contains(&slug.as_str()))
                .then_some(())?;
            Some((slug, amount_of(v)?))
        })
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows
}

fn shown(o: &Oracle, slug: &str) -> String {
    o.names
        .get(slug)
        .cloned()
        .unwrap_or_else(|| slug.to_owned())
}

/// Leaders worked out from rows the app gave: which item, seller, buyer or month is first, and the
/// order to collect from and to pay. A leader is `top.<what>.name` and `top.<what>.amount`; an
/// order is `top.<what>_first`, `_second` and `_third`, each `Name amount [Dr|Cr] [N days]`.
fn leaders(o: &mut Oracle) {
    let mut puts: Vec<(String, String)> = Vec::new();
    for (what, prefix, suffix) in [
        ("item_commission_year", "a.year.", ".total_comm"),
        ("item_gross_year", "a.year.", ".gross"),
        ("item_qty_year", "a.year.", ".qty"),
        ("item_commission_all", "a.all.", ".total_comm"),
        ("item_commission_last_month", "a.last_month.", ".total_comm"),
        ("seller_commission_year", "b.year.", ".total_comm"),
        ("seller_gross_year", "b.year.", ".gross"),
        ("buyer_purchase_year", "c.year.", ".purchase_value"),
        ("buyer_purchase_all", "c.all.", ".purchase_value"),
        ("buyer_dues", "r20.", ".balance"),
        ("seller_payable", "r24.", ".payable"),
        ("expense_head_year", "r29.year.", ".expense"),
    ] {
        if let Some((slug, n)) = ranked(o, prefix, suffix).first() {
            puts.push((format!("top.{what}.name"), shown(o, slug)));
            puts.push((format!("top.{what}.amount"), n.to_string()));
        }
    }
    // The month with the most commission in G, as `2026-06`.
    if let Some((slug, n)) = ranked(o, "g.", ".commission").first() {
        puts.push(("top.month_commission.name".into(), slug.replace('_', "-")));
        puts.push(("top.month_commission.amount".into(), n.to_string()));
    }
    // To collect from first: the oldest bill first, then the biggest balance.
    let mut collect: Vec<(String, i64, i64)> = ranked(o, "r20.", ".balance")
        .into_iter()
        .filter_map(|(slug, bal)| {
            let days = amount_of(o.facts.get(&format!("r20.{slug}.days"))?)?;
            Some((slug, bal, days))
        })
        .collect();
    collect.sort_by(|a, b| b.2.cmp(&a.2).then(b.1.cmp(&a.1)));
    for (at, word) in ["first", "second", "third"].iter().enumerate() {
        if let Some((slug, bal, days)) = collect.get(at) {
            puts.push((
                format!("top.collect_{word}"),
                format!("{} {bal} Dr {days} days", shown(o, slug)),
            ));
        }
    }
    // The biggest dues.
    for (at, word) in ["first", "second", "third"].iter().enumerate() {
        if let Some((slug, bal)) = ranked(o, "r20.", ".balance").get(at) {
            puts.push((
                format!("top.dues_{word}"),
                format!("{} {bal} Dr", shown(o, slug)),
            ));
        }
    }
    // To pay first: the biggest payable.
    for (at, word) in ["first", "second", "third"].iter().enumerate() {
        if let Some((slug, n)) = ranked(o, "r24.", ".payable").get(at) {
            puts.push((
                format!("top.pay_{word}"),
                format!("{} {n} Cr", shown(o, slug)),
            ));
        }
    }
    for (k, v) in puts {
        o.put(k, v);
    }
}

/// The keys in order, one a line, for `scripts/ai-eval/oracle-keys.txt`.
pub fn manifest(o: &Oracle) -> String {
    let mut out = String::new();
    for key in o.facts.keys() {
        out.push_str(key);
        out.push('\n');
    }
    out
}

// ---- derived facts ----------------------------------------------------------------------------

fn amt(o: &Oracle, key: &str) -> i64 {
    amount_of(o.get(key)).unwrap_or_else(|| panic!("fact {key} is not an amount"))
}

/// `a - b` as the scorer reads a difference: `N up`, `N down` or `0`.
fn difference(a: i64, b: i64) -> String {
    match a.cmp(&b) {
        std::cmp::Ordering::Greater => format!("{} up", a - b),
        std::cmp::Ordering::Less => format!("{} down", b - a),
        std::cmp::Ordering::Equal => "0".to_owned(),
    }
}

/// Sums and differences of figures the app already gave (simple whole-rupee arithmetic).
fn derived(o: &mut Oracle) {
    let facts: Vec<(&str, String)> = vec![
        (
            "d.banana_comm.year_vs_last_year",
            difference(
                amt(o, "a.year.banana.total_comm"),
                amt(o, "a.last_year.banana.total_comm"),
            ),
        ),
        (
            "d.total_comm.year_vs_last_year",
            difference(
                amt(o, "a.year.total.total_comm"),
                amt(o, "a.last_year.total.total_comm"),
            ),
        ),
        (
            "d.gross.year_vs_last_year",
            difference(
                amt(o, "a.year.total.gross"),
                amt(o, "a.last_year.total.gross"),
            ),
        ),
        (
            "d.commission.sep_vs_aug",
            difference(
                amt(o, "a.m2026_09.total.total_comm"),
                amt(o, "a.m2026_08.total.total_comm"),
            ),
        ),
        (
            "d.gross.sep_vs_aug",
            difference(
                amt(o, "a.m2026_09.total.gross"),
                amt(o, "a.m2026_08.total.gross"),
            ),
        ),
        (
            "d.commission.oct_vs_sep",
            difference(
                amt(o, "a.m2026_10.total.total_comm"),
                amt(o, "a.m2026_09.total.total_comm"),
            ),
        ),
        (
            "d.commission.last3_sum",
            (amt(o, "a.m2026_08.total.total_comm")
                + amt(o, "a.m2026_09.total.total_comm")
                + amt(o, "a.m2026_10.total.total_comm"))
            .to_string(),
        ),
        (
            "d.gross.last3_sum",
            (amt(o, "a.m2026_08.total.gross")
                + amt(o, "a.m2026_09.total.gross")
                + amt(o, "a.m2026_10.total.gross"))
            .to_string(),
        ),
        (
            "d.apple_vs_banana_comm_year",
            difference(
                amt(o, "a.year.apple.total_comm"),
                amt(o, "a.year.banana.total_comm"),
            ),
        ),
        (
            "d.akrams_dues_sum",
            format!(
                "{} Dr",
                amt(o, "buyer.akram_toll_tax.balance") + amt(o, "buyer.akram_baloch.balance")
            ),
        ),
        (
            "d.akram_baloch_vs_toll_tax",
            difference(
                amt(o, "buyer.akram_baloch.balance"),
                amt(o, "buyer.akram_toll_tax.balance"),
            ),
        ),
        (
            "d.tariq_vs_zahid_dues",
            difference(
                amt(o, "r20.tariq_mehmood.balance"),
                amt(o, "r20.zahid_mehmood.balance"),
            ),
        ),
    ];
    for (k, v) in facts {
        o.put(k.to_owned(), v);
    }
    // The garden's payments from the shop and from pockets, and the guests' fruit (the total less
    // the partner's own).
    // The average of August, September and October to the nearest rupee, half up.
    let last3 = amt(o, "d.commission.last3_sum");
    o.put(
        "d.commission.last3_avg".into(),
        ((2 * last3 + 3) / 6).to_string(),
    );
    let paid =
        amt(o, "i.all.shahpur_garden.paid_shop") + amt(o, "i.all.shahpur_garden.paid_pocket");
    o.put("d.garden_paid_total".into(), paid.to_string());
    let guests = amt(o, "k.year.total.value") - amt(o, "k.year.partner_a.value");
    o.put("d.fruit_guests_year".into(), guests.to_string());
    // Ten percent more than this year's commission, to the nearest rupee, half up.
    let comm = amt(o, "a.year.total.total_comm");
    o.put(
        "d.comm_year_plus_10pct".into(),
        (comm + (comm * 10 + 50) / 100).to_string(),
    );
    o.put(
        "d.comm_year_10pct".into(),
        ((comm * 10 + 50) / 100).to_string(),
    );
}

// ---- estimates ---------------------------------------------------------------------------------

/// The estimates of the months ahead, from the same history and the same function as the
/// assistant's `estimate` tool (the series of view G up to the last full month, then
/// `mandi_services::forecast::estimate`). Also the year-end total and the pace doubled.
fn estimates(o: &mut Oracle, y: &Year) {
    use mandi_services::core::money::Paisa;
    use mandi_services::domain::forecast::{Method, YearMonth};
    use mandi_services::forecast::{self, Forecast};
    let today = date(TODAY);
    let Some(current) = YearMonth::new(today.year(), today.month()) else {
        return;
    };
    let last_full = today.with_day(1).unwrap_or(today) - Duration::days(1);
    for (measure, column) in [
        ("commission", "commission"),
        ("sales", "gross"),
        ("profit", "profit"),
        ("shop_expenses", "shop_expenses"),
    ] {
        let mut f = afilters("all");
        f.to = ddmmyyyy(last_full);
        f.group_by = Some(analytics::AnalyticsGroupBy::Month);
        let page = run_a(y, AnalyticsView::G, f);
        let at = page.columns.iter().position(|c| c.key == column);
        let Some(at) = at else { continue };
        let history = history_of(&page, at);
        let name = |m: Method| match m {
            Method::SamePace => "same_pace",
            Method::SameGrowth => "same_growth",
            Method::SameSeason => "same_season",
            Method::WhatIfPace => "what_if_pace",
        };
        for ahead in [1u32, 2, 3, 6, 12] {
            let Ok(Forecast::Estimate(e)) = forecast::estimate(&history, current, ahead, None)
            else {
                continue;
            };
            for m in &e.methods {
                o.put(
                    format!("est.{measure}.a{ahead}.{}", name(m.method)),
                    m.total.rupees().to_string(),
                );
                if let Some(per) = m.per_month {
                    o.put(
                        format!("est.{measure}.a{ahead}.{}_per_month", name(m.method)),
                        per.rupees().to_string(),
                    );
                }
                if m.method == Method::SamePace {
                    // The pace doubled: the same-pace month times 2, for each month ahead.
                    if let Some(per) = m.per_month {
                        let doubled = per.rupees() * 2 * i64::from(ahead);
                        o.put(
                            format!("est.{measure}.a{ahead}.double_pace"),
                            doubled.to_string(),
                        );
                        // Tripled: exact in whole rupees, like doubled (half and a percent are not).
                        let tripled = per.rupees() * 3 * i64::from(ahead);
                        o.put(
                            format!("est.{measure}.a{ahead}.triple_pace"),
                            tripled.to_string(),
                        );
                    }
                }
            }
            o.put(
                format!("est.{measure}.a{ahead}.first_month"),
                format!("{}-{:02}", e.first.year(), e.first.month()),
            );
        }
        // The whole year: the months so far this year plus the months ahead to December.
        let months_left = 13 - today.month();
        if let Ok(Forecast::Estimate(e)) = forecast::estimate(&history, current, months_left, None)
        {
            // The books of the year so far, as the estimate itself counts them: the FULL months
            // (the current month is one of the months ahead, so its few days are not added twice).
            let so_far: Option<i64> = Some(
                history
                    .iter()
                    .filter(|h| h.month.year() == current.year())
                    .map(|h| h.value.rupees())
                    .sum(),
            );
            if let Some(full) = so_far {
                o.put(
                    format!("est.{measure}.year_full_months_so_far"),
                    full.to_string(),
                );
            }
            for m in &e.methods {
                o.put(
                    format!("est.{measure}.year_end.{}", name(m.method)),
                    m.total.rupees().to_string(),
                );
                if let Some(so_far) = so_far {
                    o.put(
                        format!("est.{measure}.year_total.{}", name(m.method)),
                        (so_far + m.total.rupees()).to_string(),
                    );
                }
            }
        }
        // When a target is passed (commission only: 30 lakh and 1 crore).
        if measure == "commission" {
            for target in [3_000_000i64, 10_000_000] {
                let Some(t) = Paisa::from_rupees(target).ok() else {
                    continue;
                };
                if let Ok(Forecast::Estimate(e)) =
                    forecast::estimate(&history, current, 12, Some(t))
                {
                    for m in &e.methods {
                        if let Some(tm) = m.target_month {
                            o.put(
                                format!("est.{measure}.target{target}.{}", name(m.method)),
                                format!("{}-{:02}", tm.year(), tm.month()),
                            );
                        }
                    }
                }
            }
        }
    }
}

/// The estimate of one item's commission (view G with the item), next 3 months and the year end.
fn item_estimates(o: &mut Oracle, y: &Year) {
    // The year's commission so far plus the doubled pace to the end of the year.
    if let (Some(so_far), Some(doubled)) = (
        o.facts
            .get("est.commission.year_full_months_so_far")
            .and_then(|v| amount_of(v)),
        o.facts
            .get("est.commission.a2.double_pace")
            .and_then(|v| amount_of(v)),
    ) {
        o.put(
            "d.comm_year_total_double".into(),
            (so_far + doubled).to_string(),
        );
    }
    use mandi_services::domain::forecast::{Method, YearMonth};
    use mandi_services::forecast::{self, Forecast};
    let today = date(TODAY);
    let Some(current) = YearMonth::new(today.year(), today.month()) else {
        return;
    };
    let last_full = today.with_day(1).unwrap_or(today) - Duration::days(1);
    for (name, key) in [("banana", "kela"), ("apple", "saib")] {
        let Some(item) = y.items.iter().find(|i| i.key == key) else {
            continue;
        };
        let mut f = afilters("all");
        f.to = ddmmyyyy(last_full);
        f.group_by = Some(analytics::AnalyticsGroupBy::Month);
        f.item_id = Some(item.id);
        let page = run_a(y, AnalyticsView::G, f);
        let Some(at) = page.columns.iter().position(|c| c.key == "commission") else {
            continue;
        };
        let history = history_of(&page, at);
        for ahead in [3u32, 6] {
            if let Ok(Forecast::Estimate(e)) = forecast::estimate(&history, current, ahead, None) {
                for m in &e.methods {
                    let word = match m.method {
                        Method::SamePace => "same_pace",
                        Method::SameGrowth => "same_growth",
                        Method::SameSeason => "same_season",
                        Method::WhatIfPace => "what_if_pace",
                    };
                    o.put(
                        format!("est_item.{name}.commission.a{ahead}.{word}"),
                        m.total.rupees().to_string(),
                    );
                }
            }
        }
    }
}

/// The few facts of the thin books (no sale, no voucher): the dashboard's tiles. Keys `thin.<tile>`.
pub fn compute_thin(y: &Year) -> Oracle {
    let mut o = Oracle::default();
    let now = dashboard::tiles_now(y.w.env.ctx.db()).expect("the tiles");
    let t = &now.tiles;
    let money = |p: mandi_services::core::money::Paisa| p.rupees().to_string();
    o.put("thin.cash".into(), money(t.cash_in_hand_paisa));
    o.put("thin.sales_today".into(), money(t.sales_gross_paisa));
    o.put("thin.count_today".into(), t.sales_count.to_string());
    o.put("thin.commission_today".into(), money(t.commission_paisa));
    o.put("thin.buyers_dues".into(), money(t.buyers_dues_paisa));
    o.put(
        "thin.sellers_payable".into(),
        money(t.sellers_payable_paisa),
    );
    o.put("thin.stock_qty".into(), t.stock_qty.to_string());
    o.put("thin.advances_open".into(), money(t.advances_open_paisa));
    o
}

/// The monthly series of a column of view G, as the estimator reads it.
fn history_of(
    page: &AnalyticsPage,
    at: usize,
) -> Vec<mandi_services::domain::forecast::MonthValue> {
    use mandi_services::domain::forecast::{MonthValue, YearMonth};
    page.rows
        .iter()
        .filter_map(|row| {
            let (Some(Cell::Month { year, month }), Some(Cell::Money { paisa })) =
                (row.cells.first(), row.cells.get(at))
            else {
                return None;
            };
            let month = YearMonth::new(i32::try_from(*year).ok()?, u32::try_from(*month).ok()?)?;
            Some(MonthValue {
                month,
                value: *paisa,
            })
        })
        .collect()
}
