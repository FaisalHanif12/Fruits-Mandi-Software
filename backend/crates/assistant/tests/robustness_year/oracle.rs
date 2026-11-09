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
        ("week_start", first("this_week", None)),
        ("year_start", first("this_year", None)),
        ("last_year_start", first("last_year", None)),
        ("last_year_end", last("last_year", None)),
        ("first_day", date(super::seed::START)),
        ("aug_start", first("month", Some(8))),
        ("aug_end", last("month", Some(8))),
        (
            "nov_03",
            NaiveDate::from_ymd_opt(today.year(), 11, 3).unwrap_or(today),
        ),
        (
            "oct_14",
            NaiveDate::from_ymd_opt(today.year(), 10, 14).unwrap_or(today),
        ),
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
        "last_week" | "last_week2" => {
            let monday = today - Duration::days(i64::from(today.weekday().num_days_from_monday()));
            let back = if name == "last_week" { 7 } else { 14 };
            (
                Some(monday - Duration::days(back)),
                Some(monday - Duration::days(back - 6)),
            )
        }
        "last_year2" => (
            NaiveDate::from_ymd_opt(today.year() - 2, 1, 1),
            NaiveDate::from_ymd_opt(today.year() - 2, 12, 31),
        ),
        other if other.starts_with('r') && other[1..].split('_').count() == 6 => {
            // r2026_10_01_2026_10_15: that range of days.
            let n: Vec<i32> = other[1..]
                .split('_')
                .filter_map(|p| p.parse().ok())
                .collect();
            match n.as_slice() {
                [y1, m1, d1, y2, m2, d2] => (
                    NaiveDate::from_ymd_opt(
                        *y1,
                        u32::try_from(*m1).unwrap_or(0),
                        u32::try_from(*d1).unwrap_or(0),
                    ),
                    NaiveDate::from_ymd_opt(
                        *y2,
                        u32::try_from(*m2).unwrap_or(0),
                        u32::try_from(*d2).unwrap_or(0),
                    ),
                ),
                _ => (None, None),
            }
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
    capability_reports(&mut o, y);

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
    screen_series_estimates(&mut o, y);
    balance_estimates(&mut o, y);
    state_and_list_facts(&mut o, y);
    vegetable_facts(&mut o, y);
    item_axis_facts(&mut o, y);
    quality_facts(&mut o, y);
    money_axis_facts(&mut o, y);
    d7_facts(&mut o, y);
    o
}

/// A report that may be refused for these filters: its facts are written only when it runs.
fn try_r(y: &Year, id: ReportId, filters: ReportFilters) -> Option<ReportPage> {
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
    .ok()
}

/// The reports the capability matrix reads (plan 55 Section 11, `scripts/ai-eval/capability.tsv`)
/// that the oracle did not dump before: the registers by period (totals, the summary lines and the
/// count; their rows are documents), the balance lists as of today, and one buyer's and one seller's
/// own reports. Keys: `<report>.<period>.total.<column>`, `<report>.<period>.count`, `r13.<buyer>...`.
fn capability_reports(o: &mut Oracle, y: &Year) {
    let periods = [
        "today",
        "yesterday",
        "week",
        "month",
        "last_month",
        "year",
        "all",
    ];
    for p in periods {
        for (id, name) in [
            (ReportId::R2, "r2"),
            (ReportId::R3, "r3"),
            (ReportId::R4, "r4"),
            (ReportId::R5, "r5"),
            (ReportId::R6, "r6"),
            (ReportId::R7, "r7"),
            (ReportId::R8, "r8"),
            (ReportId::R11, "r11"),
            (ReportId::R15, "r15"),
            (ReportId::R27, "r27"),
            (ReportId::R28, "r28"),
            (ReportId::R34, "r34"),
        ] {
            // The day book and the day summary list a line or a day for each day: not for all time.
            if matches!(id, ReportId::R11 | ReportId::R34) && p == "all" {
                continue;
            }
            if let Some(page) = try_r(y, id, rfilters(p)) {
                dump_report_rows(o, &format!("{name}.{p}"), &page, false);
            }
        }
    }
    let as_of = ReportFilters {
        as_of: ddmmyyyy(date(TODAY)),
        ..ReportFilters::default()
    };
    for (id, name, rows) in [
        (ReportId::R16, "r16", false),
        (ReportId::R17, "r17", false),
        (ReportId::R18, "r18", false),
        (ReportId::R21, "r21", true),
        (ReportId::R22, "r22", true),
    ] {
        if let Some(page) = try_r(y, id, as_of.clone()) {
            dump_report_rows(o, name, &page, rows);
        }
    }
    if let Some(page) = try_r(y, ReportId::R19, ReportFilters::default()) {
        dump_report_rows(o, "r19", &page, false);
    }
    // One buyer: his ledger, his statement, his bills and his day sheets.
    for key in ["faisal", "akram_toll_tax"] {
        let Some(b) = y.buyers.iter().find(|b| b.key == key) else {
            continue;
        };
        let party = Some(PartyFilter {
            account_id: b.id,
            person_key: None,
        });
        let with = |base: ReportFilters| ReportFilters {
            party: party.clone(),
            ..base
        };
        for (id, name) in [(ReportId::R13, "r13"), (ReportId::R14, "r14")] {
            if let Some(page) = try_r(y, id, with(rfilters("all"))) {
                dump_report_rows(o, &format!("{name}.{key}"), &page, false);
            }
        }
        if let Some(page) = try_r(y, ReportId::R5, with(rfilters("year"))) {
            dump_report_rows(o, &format!("r5.{key}.year"), &page, false);
        }
        let sheet = ReportFilters {
            as_of: ddmmyyyy(date(TODAY)),
            party: party.clone(),
            ..ReportFilters::default()
        };
        if let Some(page) = try_r(y, ReportId::R35, sheet) {
            dump_report_rows(o, &format!("r35.{key}.today"), &page, false);
        }
    }
    // One seller: his bills of the year.
    for key in ["rahim", "karim"] {
        let Some(s) = y.sellers.iter().find(|s| s.key == key) else {
            continue;
        };
        let filters = ReportFilters {
            seller_id: Some(s.id),
            ..rfilters("year")
        };
        if let Some(page) = try_r(y, ReportId::R6, filters) {
            dump_report_rows(o, &format!("r6.{key}.year"), &page, false);
        }
    }
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

/// The row with the smallest amount above zero (ties: the name that sorts first), as (slug, amount),
/// leaving out the rows named in `skip`.
fn lowest(o: &Oracle, prefix: &str, suffix: &str, skip: &[&str]) -> Option<(String, i64)> {
    ranked(o, prefix, suffix)
        .into_iter()
        .filter(|(slug, n)| *n > 0 && !skip.contains(&slug.as_str()))
        .min_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)))
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
    // The LOWEST ones (lead, 05-11-2026): the item, seller or buyer with the least commission or
    // sale in a period, and the month with the least commission, each with its name and its amount,
    // from the same rows. A row with no amount is not "the lowest" (it did not sell). Keys are
    // `low.<what>.name` and `low.<what>.amount`.
    // A seller row of view B can be the own goods of the trips or a garden, and a buyer row of view C
    // the cash buyer with no name: they are not "a seller" or "a buyer" to the Owner. The plain key
    // leaves them out; the key ending `_any` is the lowest row of the view as it stands.
    let not_sellers = ["own_goods_trips", "shahpur_garden"];
    let not_buyers = ["naqdi_mandi_cash_buyer"];
    for period in ["year", "all", "last_month", "month"] {
        for (what, view, suffix, skip) in [
            ("item_commission", "a", ".total_comm", &[][..]),
            ("item_gross", "a", ".gross", &[][..]),
            ("seller_commission", "b", ".total_comm", &not_sellers[..]),
            ("seller_gross", "b", ".gross", &not_sellers[..]),
            ("buyer_purchase", "c", ".purchase_value", &not_buyers[..]),
            ("buyer_commission", "c", ".buyer_comm", &not_buyers[..]),
        ] {
            let prefix = format!("{view}.{period}.");
            if let Some((slug, n)) = lowest(o, &prefix, suffix, skip) {
                puts.push((format!("low.{what}_{period}.name"), shown(o, &slug)));
                puts.push((format!("low.{what}_{period}.amount"), n.to_string()));
            }
            if !skip.is_empty()
                && let Some((slug, n)) = lowest(o, &prefix, suffix, &[])
            {
                puts.push((format!("low.{what}_{period}_any.name"), shown(o, &slug)));
                puts.push((format!("low.{what}_{period}_any.amount"), n.to_string()));
            }
        }
    }
    // The month with the least commission: among the finished months, and among all months with
    // the running one (which has only a few days), because both readings are honest.
    let running = format!("{:04}_{:02}", TODAY.0, TODAY.1);
    let months = ranked(o, "g.", ".commission");
    let least = |skip_running: bool| {
        months
            .iter()
            .filter(|(slug, n)| *n > 0 && !(skip_running && *slug == running))
            .min_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)))
            .cloned()
    };
    if let Some((slug, n)) = least(true) {
        puts.push(("low.month_commission.name".into(), slug.replace('_', "-")));
        puts.push(("low.month_commission.amount".into(), n.to_string()));
    }
    if let Some((slug, n)) = least(false) {
        puts.push((
            "low.month_commission_any.name".into(),
            slug.replace('_', "-"),
        ));
        puts.push(("low.month_commission_any.amount".into(), n.to_string()));
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
        let history = history_of(&page, at, current);
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
    use mandi_services::domain::forecast::YearMonth;
    let today = date(TODAY);
    let Some(current) = YearMonth::new(today.year(), today.month()) else {
        return;
    };
    let last_full = today.with_day(1).unwrap_or(today) - Duration::days(1);
    // Every item (plan 55 Section 11: not banana and apple only), the commission and the sales: the
    // next 1, 3, 6 and 12 months by each method, the pace doubled and tripled, and the year end.
    // The keys of banana and apple commission for 3 and 6 months are the same as before.
    for item in &y.items {
        let name = item_name(item.key);
        for (measure, column) in [("commission", "commission"), ("sales", "gross")] {
            let mut f = afilters("all");
            f.to = ddmmyyyy(last_full);
            f.group_by = Some(analytics::AnalyticsGroupBy::Month);
            f.item_id = Some(item.id);
            let page = run_a(y, AnalyticsView::G, f);
            if let Some(at) = page.columns.iter().position(|c| c.key == column) {
                put_series_estimates(
                    o,
                    &format!("est_item.{name}.{measure}"),
                    &history_of(&page, at, current),
                    current,
                );
            }
        }
    }
    // A seller's goods and a buyer's purchases (the sales measure with a seller or a buyer), for
    // the mixed questions "Rahim ka maal is saal ... aur agle 3 mahine ka andaza".
    for key in ["rahim", "karim"] {
        let Some(s) = y.sellers.iter().find(|s| s.key == key) else {
            continue;
        };
        let mut f = afilters("all");
        f.to = ddmmyyyy(last_full);
        f.group_by = Some(analytics::AnalyticsGroupBy::Month);
        f.seller_id = Some(s.id);
        let page = run_a(y, AnalyticsView::G, f);
        if let Some(at) = page.columns.iter().position(|c| c.key == "gross") {
            put_series_estimates(
                o,
                &format!("est_party.{key}.sales"),
                &history_of(&page, at, current),
                current,
            );
        }
    }
    for key in ["faisal", "tariq"] {
        let Some(b) = y.buyers.iter().find(|b| b.key == key) else {
            continue;
        };
        let mut f = afilters("all");
        f.to = ddmmyyyy(last_full);
        f.group_by = Some(analytics::AnalyticsGroupBy::Month);
        f.buyer = Some(PartyFilter {
            account_id: b.id,
            person_key: None,
        });
        let page = run_a(y, AnalyticsView::G, f);
        if let Some(at) = page.columns.iter().position(|c| c.key == "gross") {
            put_series_estimates(
                o,
                &format!("est_party.{key}.sales"),
                &history_of(&page, at, current),
                current,
            );
        }
    }
}

/// The estimates of one monthly series, as keys `<prefix>.a<N>.<method>` for 1, 3, 6 and 12 months
/// ahead, the pace doubled and tripled, and the year end (`year_end`, and `year_total` with the
/// months of the year so far). The same function as the assistant's `estimate` tool.
fn put_series_estimates(
    o: &mut Oracle,
    prefix: &str,
    history: &[mandi_services::domain::forecast::MonthValue],
    current: mandi_services::domain::forecast::YearMonth,
) {
    use mandi_services::domain::forecast::Method;
    use mandi_services::forecast::{self, Forecast};
    for ahead in [1u32, 3, 6, 12] {
        let Ok(Forecast::Estimate(e)) = forecast::estimate(history, current, ahead, None) else {
            continue;
        };
        for m in &e.methods {
            let word = method_word(m.method);
            o.put(
                format!("{prefix}.a{ahead}.{word}"),
                m.total.rupees().to_string(),
            );
            if m.method == Method::SamePace
                && let Some(per) = m.per_month
            {
                let ahead_n = i64::from(ahead);
                o.put(
                    format!("{prefix}.a{ahead}.double_pace"),
                    (per.rupees() * 2 * ahead_n).to_string(),
                );
                o.put(
                    format!("{prefix}.a{ahead}.triple_pace"),
                    (per.rupees() * 3 * ahead_n).to_string(),
                );
            }
        }
    }
    let months_left = 13 - date(TODAY).month();
    if let Ok(Forecast::Estimate(e)) = forecast::estimate(history, current, months_left, None) {
        let so_far: i64 = history
            .iter()
            .filter(|h| h.month.year() == current.year())
            .map(|h| h.value.rupees())
            .sum();
        for m in &e.methods {
            let word = method_word(m.method);
            o.put(
                format!("{prefix}.year_end.{word}"),
                m.total.rupees().to_string(),
            );
            o.put(
                format!("{prefix}.year_total.{word}"),
                (so_far + m.total.rupees()).to_string(),
            );
        }
    }
}

/// The English word the oracle uses for an item key of the seed (`kela` is `banana`).
fn item_name(key: &str) -> &'static str {
    match key {
        "kela" => "banana",
        "saib" => "apple",
        "mausami" | "mosambi" => "mosambi",
        "aam" | "mango" => "mango",
        "angoor" | "grapes" => "grapes",
        "aloo" | "potato" => "potato",
        "mooli" => "radish",
        "tamatar" => "tomato",
        "piyaz" => "onion",
        "saag" => "saag",
        _ => "other",
    }
}

fn method_word(m: mandi_services::domain::forecast::Method) -> &'static str {
    use mandi_services::domain::forecast::Method;
    match m {
        Method::SamePace => "same_pace",
        Method::SameGrowth => "same_growth",
        Method::SameSeason => "same_season",
        Method::WhatIfPace => "what_if_pace",
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

/// The monthly series of a column of view G, as the estimator reads it. View G has no row for a
/// month with no sales, so the series is filled the way the tool's `fill_history` does it: from the
/// first month with a row to the month before `current`, a quiet month is 0 (F6: a seasonal item, a
/// seller or a buyer with gaps still has an estimate).
fn history_of(
    page: &AnalyticsPage,
    at: usize,
    current: mandi_services::domain::forecast::YearMonth,
) -> Vec<mandi_services::domain::forecast::MonthValue> {
    use mandi_services::core::money::Paisa;
    use mandi_services::domain::forecast::{MonthValue, YearMonth};
    let mut rows: Vec<MonthValue> = page
        .rows
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
        .collect();
    rows.sort_by_key(|r| r.month);
    let Some(first) = rows.first().map(|r| r.month) else {
        return rows;
    };
    let mut filled = Vec::new();
    let mut month = first;
    while month < current {
        let value = rows
            .iter()
            .find(|r| r.month == month)
            .map_or(Paisa::ZERO, |r| r.value);
        filled.push(MonthValue { month, value });
        match month.plus_months(1) {
            Some(next) => month = next,
            None => break,
        }
    }
    filled
}

/// The full months before `current`, from January 2025: the month, its first day and its last day.
fn months_before(
    current: mandi_services::domain::forecast::YearMonth,
) -> Vec<(
    mandi_services::domain::forecast::YearMonth,
    NaiveDate,
    NaiveDate,
)> {
    use mandi_services::domain::forecast::YearMonth;
    let mut out = Vec::new();
    let Some(mut month) = YearMonth::new(2025, 1) else {
        return out;
    };
    while month < current {
        let Some(next) = month.plus_months(1) else {
            break;
        };
        let (Some(first), Some(next_first)) = (
            NaiveDate::from_ymd_opt(month.year(), month.month(), 1),
            NaiveDate::from_ymd_opt(next.year(), next.month(), 1),
        ) else {
            break;
        };
        out.push((month, first, next_first - Duration::days(1)));
        month = next;
    }
    out
}

/// A series read month by month from a screen: the months before the first month with a figure are
/// left out and a quiet month after it is 0 (the tool's `fill_history`).
fn trimmed(
    values: &[(
        mandi_services::domain::forecast::YearMonth,
        mandi_services::core::money::Paisa,
    )],
) -> Vec<mandi_services::domain::forecast::MonthValue> {
    use mandi_services::core::money::Paisa;
    use mandi_services::domain::forecast::MonthValue;
    let Some(at) = values.iter().position(|(_, v)| *v != Paisa::ZERO) else {
        return Vec::new();
    };
    values
        .iter()
        .skip(at)
        .map(|(month, value)| MonthValue {
            month: *month,
            value: *value,
        })
        .collect()
}

/// The balance estimates (plan 55 Section 11.1, F4 and F5; PRD FR-7.86, approved by Umar): the balance
/// now plus the average net change of a month times the months ahead. Worked out here from the books
/// by the definition in the notes of the estimate tool, never by its code:
///
/// - a party (`faisal`, `tariq`, `rahim`) and the cash (`cash`): the change of a month is the sum of
///   (debit - credit) of the posted lines of the account dated in the month, entries of source type
///   `account` (openings) left out; the balance now counts every posted line.
/// - the buyers' dues (`dues`): the change of a month is R20's total as of the last day of the month
///   less R20's total as of the last day of the month before.
/// - the months run from the first month with a line (not an opening) to the month before today's; a
///   quiet month is 0. The pace uses the LAST 3 of them; fewer than 3 is too short: `.n` only.
/// - `per_month` is the sum over the window divided by its months, rounded once to a whole rupee, half
///   away from zero.
///
/// Keys: `est_balance.<subject>.now`, `.n` (months in the window), `.per_month` (its size),
/// `.direction` (`up` or `down`), `.after<j>` for j = 1, 2, 3, 6 and 12: the size of the balance after
/// j months from today (now + j x per_month). Sizes are whole rupees without a sign. The dues are
/// never below 0: a month that would take them below 0 has no fact.
fn balance_estimates(o: &mut Oracle, y: &Year) {
    use mandi_services::db::rusqlite::{self, params};
    use mandi_services::domain::forecast::YearMonth;
    let today = date(TODAY);
    let Some(current) = YearMonth::new(today.year(), today.month()) else {
        return;
    };
    let months = months_before(current);
    let db = y.w.env.ctx.db();
    let iso = |d: NaiveDate| d.format("%Y-%m-%d").to_string();
    let cash_id: i64 = db
        .read(|c| {
            c.query_row("SELECT id FROM accounts WHERE code = '01010001'", [], |r| {
                r.get(0)
            })
        })
        .unwrap_or_else(|e: rusqlite::Error| panic!("the cash account: {e}"));
    // One account: (count of lines, net debit less credit in rupees) of each month, openings left out;
    // and the balance now.
    // For the cash the money partners put in or take out (capital_in, capital_out, and their reversals) is not a
    // change of the pace (backend3's note, PRD FR-7.86): it is left out of the months.
    const NOT_CAPITAL: &str = "AND e.posting_type NOT IN ('capital_in', 'capital_out') \
         AND NOT (e.posting_type = 'reversal' AND e.reverses_entry_id IN \
                  (SELECT id FROM journal_entries WHERE posting_type IN ('capital_in', 'capital_out')))";
    let account_series = |id: i64, skip_capital: bool| -> (Vec<(i64, i64)>, i64) {
        let rows = months
            .iter()
            .map(|(_, first, last)| {
                db.read(|c| {
                    c.query_row(
                        &format!(
                            "SELECT COUNT(*), COALESCE(SUM(l.dr_paisa - l.cr_paisa), 0) \
                             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id \
                             WHERE l.account_id = ?1 AND e.status = 'posted' AND e.source_type <> 'account' \
                               AND e.entry_date >= ?2 AND e.entry_date <= ?3 {}",
                            if skip_capital { NOT_CAPITAL } else { "" }
                        ),
                        params![id, iso(*first), iso(*last)],
                        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)? / 100)),
                    )
                })
                .unwrap_or_else(|e: rusqlite::Error| panic!("a month of the account: {e}"))
            })
            .collect();
        let now = db
            .read(|c| {
                c.query_row(
                    "SELECT COALESCE(SUM(l.dr_paisa - l.cr_paisa), 0) \
                     FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id \
                     WHERE l.account_id = ?1 AND e.status = 'posted'",
                    params![id],
                    |r| r.get::<_, i64>(0),
                )
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("the balance now: {e}"))
            / 100;
        (rows, now)
    };
    let put = |o: &mut Oracle, name: &str, rows: &[(i64, i64)], now: i64, floor_zero: bool| {
        let Some(first) = rows.iter().position(|(count, _)| *count > 0) else {
            return;
        };
        let window: Vec<i64> = rows[first..].iter().map(|(_, net)| *net).collect();
        // The pace is the average of the LAST 3 full months, as for the other estimates (backend3,
        // `domain::forecast::BALANCE_MONTHS_MAX`); fewer than 3 months in the books is too short.
        let available = i64::try_from(window.len()).unwrap_or(0);
        o.put(
            format!("est_balance.{name}.n"),
            available.min(3).to_string(),
        );
        o.put(format!("est_balance.{name}.now"), now.abs().to_string());
        if available < 3 {
            return;
        }
        let window = &window[window.len() - 3..];
        let n = 3i64;
        let total: i64 = window.iter().sum();
        // Half away from zero, once.
        let per_month = total.signum() * ((total.abs() * 2 + n) / (n * 2));
        o.put(
            format!("est_balance.{name}.per_month"),
            per_month.abs().to_string(),
        );
        o.put(
            format!("est_balance.{name}.direction"),
            if per_month >= 0 { "up" } else { "down" }.into(),
        );
        for j in [1i64, 2, 3, 6, 12] {
            let after = now + j * per_month;
            if floor_zero && after < 0 {
                continue;
            }
            o.put(
                format!("est_balance.{name}.after{j}"),
                after.abs().to_string(),
            );
        }
    };
    for (name, id) in y
        .buyers
        .iter()
        .filter(|b| b.key == "faisal" || b.key == "tariq")
        .map(|b| (b.key, b.id))
        .chain(
            y.sellers
                .iter()
                .filter(|s| s.key == "rahim")
                .map(|s| (s.key, s.id)),
        )
    {
        let (rows, now) = account_series(id, false);
        put(o, name, &rows, now, false);
    }
    let (rows, now) = account_series(cash_id, true);
    put(o, "cash", &rows, now, false);
    // The buyers' dues (backend3, PRD FR-7.86): the sum over the owners (each buyer account, each چکتا کھاتہ
    // person) of the balance when it is a debit, at the end of each month, with the opening lines applied first
    // (an opening and a reversal of one count from the start) and a dead mark ignored. The change of a month is
    // that sum at its end less the sum at the end of the month before. Worked here from the journal lines.
    let dues_at = |day: NaiveDate| -> i64 {
        db.read(|c| {
            c.query_row(
                "SELECT COALESCE(SUM(CASE WHEN bal > 0 THEN bal ELSE 0 END), 0) FROM ( \
                   SELECT SUM(l.dr_paisa - l.cr_paisa) AS bal \
                   FROM journal_lines l \
                   JOIN journal_entries e ON e.id = l.entry_id \
                   JOIN accounts a ON a.id = l.account_id \
                   JOIN account_controls k ON k.id = a.control_id \
                   WHERE k.code = '0304' AND e.status = 'posted' \
                     AND (e.entry_date <= ?1 OR e.posting_type = 'opening' \
                          OR (e.posting_type = 'reversal' AND e.reverses_entry_id IN \
                              (SELECT id FROM journal_entries WHERE posting_type = 'opening'))) \
                   GROUP BY l.account_id, COALESCE(l.chukta_person_key, ''))",
                params![iso(day)],
                |r| r.get::<_, i64>(0),
            )
        })
        .unwrap_or_else(|e: rusqlite::Error| panic!("the dues at {day}: {e}"))
            / 100
    };
    let dues_now = |day: NaiveDate| -> Option<i64> {
        let page = try_r(
            y,
            ReportId::R20,
            ReportFilters {
                as_of: ddmmyyyy(day),
                ..ReportFilters::default()
            },
        )?;
        let at = page.columns.iter().position(|c| c.key == "balance")?;
        match page.totals.get(at)? {
            Cell::Balance { paisa, .. } | Cell::Money { paisa } => Some(paisa.rupees()),
            _ => None,
        }
    };
    let mut rows = Vec::new();
    let mut before = months
        .first()
        .map_or(0, |(_, first, _)| dues_at(*first - Duration::days(1)));
    for (_, first, last) in &months {
        let count: i64 = db
            .read(|c| {
                c.query_row(
                    "SELECT COUNT(*) FROM journal_lines l \
                     JOIN journal_entries e ON e.id = l.entry_id \
                     JOIN accounts a ON a.id = l.account_id \
                     JOIN account_controls k ON k.id = a.control_id \
                     WHERE k.code = '0304' AND e.status = 'posted' AND e.source_type <> 'account' \
                       AND e.entry_date >= ?1 AND e.entry_date <= ?2",
                    params![iso(*first), iso(*last)],
                    |r| r.get(0),
                )
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("lines on buyer accounts: {e}"));
        let end = dues_at(*last);
        rows.push((count, end - before));
        before = end;
    }
    // The balance now is the tile's, R20's total.
    let now = dues_now(today).unwrap_or(0);
    put(o, "dues", &rows, now, true);
}

/// The facts of the app's own state and short lists (plan 55 Section 11.1, R1 to R6; the capability
/// matrix gaps Q1 to Q6 and P2), worked out here from the screens and the books, never from the
/// assistant's tools:
///
/// - `r15_type.<receipt|payment|advance>.*`: R15 (the voucher list) of one voucher type, all time.
/// - `staff.<munshi|collector|labour|all>` and `heads.count`: the active staff by job and the active
///   expense heads (SQL on the masters).
/// - `truck.<truck slug>.{count,last_number,last_gross,last_net}`: for the four trucks with the most
///   بکری, the number of saved بکری, the newest one's number and its gross and net (sums of its lots).
/// - `attention.total` and `attention.<kind>.{count,amount,qty,days}`: the bell and the dashboard list.
/// - `days.{closed_up_to,months_to_close,last_month_close}`: the status bar and the day close screen.
/// - `audit.bikri<N>.count`: the audit viewer's rows of one بکری, for the first three.
/// - `rules.<item>.<side>_<charge>`: the rule that wins today for an item (and its group) with no
///   party: the percent as `6.00%`, an amount in whole rupees, or `none`. PRD 11.3, worked here from
///   the rows of `charge_rules` (the newer rule wins a tie, FR-11.5).
fn state_and_list_facts(o: &mut Oracle, y: &Year) {
    use mandi_services::core::vouchers::VoucherType;
    use mandi_services::db::rusqlite::{self, params};
    let db = y.w.env.ctx.db();
    let today = date(TODAY);
    // R15 by voucher type.
    for (word, kind) in [
        ("receipt", VoucherType::Receipt),
        ("payment", VoucherType::Payment),
        ("advance", VoucherType::Advance),
    ] {
        let filters = ReportFilters {
            voucher_type: Some(kind),
            ..rfilters("all")
        };
        if let Some(page) = try_r(y, ReportId::R15, filters) {
            dump_report_rows(o, &format!("r15_type.{word}"), &page, false);
        }
    }
    // The staff and the expense heads.
    let jobs: Vec<(String, i64)> = db
        .read(|c| {
            let mut st = c.prepare(
                "SELECT role, COUNT(*) FROM staff WHERE is_active = 1 GROUP BY role ORDER BY role",
            )?;
            let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
            rows.collect::<Result<Vec<_>, rusqlite::Error>>()
        })
        .unwrap_or_default();
    let mut all = 0;
    for job in ["munshi", "collector", "labour"] {
        let n = jobs.iter().find(|(r, _)| r == job).map_or(0, |(_, n)| *n);
        all += n;
        o.put(format!("staff.{job}"), n.to_string());
    }
    o.put("staff.all".into(), all.to_string());
    let heads: i64 = db
        .read(|c| {
            c.query_row(
                "SELECT COUNT(*) FROM expense_heads WHERE is_active = 1",
                [],
                |r| r.get(0),
            )
        })
        .unwrap_or_else(|e: rusqlite::Error| panic!("expense heads: {e}"));
    o.put("heads.count".into(), heads.to_string());
    // The short lists of the masters (plan 55 R5): packings, stores, marks and standard narrations. For each
    // the active rows and the first by code; a mark is given by its owner (the mark text is never given to the
    // model, FR-7.83) and a narration only by its count (its text is never given either).
    let count_of = |sql: &str| -> i64 {
        db.read(|c| c.query_row(sql, [], |r| r.get(0)))
            .unwrap_or_else(|e: rusqlite::Error| panic!("count {sql}: {e}"))
    };
    let first_of =
        |sql: &str| -> Option<String> { db.read(|c| c.query_row(sql, [], |r| r.get(0))).ok() };
    o.put(
        "packings.count".into(),
        count_of("SELECT COUNT(*) FROM packings WHERE is_active = 1").to_string(),
    );
    if let Some(name) =
        first_of("SELECT name_en FROM packings WHERE is_active = 1 ORDER BY code LIMIT 1")
    {
        o.put("packings.first".into(), name);
    }
    o.put(
        "stores.count".into(),
        count_of("SELECT COUNT(*) FROM stores WHERE is_active = 1").to_string(),
    );
    if let Some(name) =
        first_of("SELECT name_en FROM stores WHERE is_active = 1 ORDER BY code LIMIT 1")
    {
        o.put("stores.first".into(), name);
    }
    o.put(
        "markas.count".into(),
        count_of("SELECT COUNT(*) FROM markas WHERE is_active = 1").to_string(),
    );
    if let Some(name) = first_of(
        "SELECT a.name_en FROM markas m JOIN accounts a ON a.id = m.owner_account_id \
         WHERE m.is_active = 1 ORDER BY m.code LIMIT 1",
    ) {
        o.put("markas.first_owner".into(), name);
    }
    o.put(
        "narrations.count".into(),
        count_of("SELECT COUNT(*) FROM narrations WHERE is_active = 1").to_string(),
    );
    // The trucks with the most بکری.
    let trucks: Vec<(String, i64)> = db
        .read(|c| {
            let mut st = c.prepare(
                "SELECT truck_no, COUNT(*) AS n FROM bikri_entries \
                 WHERE status = 'saved' AND truck_no IS NOT NULL AND truck_no <> '' \
                 GROUP BY truck_no ORDER BY n DESC, truck_no LIMIT 4",
            )?;
            let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
            rows.collect::<Result<Vec<_>, rusqlite::Error>>()
        })
        .unwrap_or_default();
    for (truck, n) in &trucks {
        let k = slug(truck);
        let newest: Option<(i64, String, i64, i64)> = db
            .read(|c| {
                c.query_row(
                    "SELECT e.number, e.entry_date, COALESCE(SUM(l.gross_paisa), 0), COALESCE(SUM(l.net_paisa), 0) \
                     FROM bikri_entries e JOIN bikri_lots l ON l.bikri_id = e.id \
                     WHERE e.truck_no = ?1 AND e.status = 'saved' \
                       AND e.id = (SELECT id FROM bikri_entries WHERE truck_no = ?1 AND status = 'saved' \
                                   ORDER BY entry_date DESC, number DESC LIMIT 1) \
                     GROUP BY e.id",
                    params![truck],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
            })
            .ok();
        o.put(format!("truck.{k}.count"), n.to_string());
        if let Some((number, day, gross, net)) = newest {
            o.put(format!("truck.{k}.last_number"), number.to_string());
            o.put(format!("truck.{k}.last_date"), day);
            o.put(format!("truck.{k}.last_gross"), (gross / 100).to_string());
            o.put(format!("truck.{k}.last_net"), (net / 100).to_string());
        }
    }
    // The attention list.
    if let Ok(list) = dashboard::attention(
        db,
        BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    ) {
        use mandi_reports::dashboard::AttentionKind;
        o.put("attention.total".into(), list.total_count.to_string());
        for item in &list.items {
            let k = match item.kind {
                AttentionKind::Drafts => "drafts",
                AttentionKind::OpenDays => "open_days",
                AttentionKind::StockWaiting => "stock_waiting",
                AttentionKind::OverCreditLimit => "over_credit_limit",
                AttentionKind::DuesOld => "dues_old",
                AttentionKind::PayablesOld => "payables_old",
                AttentionKind::BackupOld => "backup_old",
                AttentionKind::LoansOverdue => "loans_overdue",
                AttentionKind::TakesOverdue => "takes_overdue",
                AttentionKind::NamesMissing => "names_missing",
            };
            o.put(format!("attention.{k}.count"), item.count.to_string());
            if let Some(amount) = item.amount_paisa {
                o.put(format!("attention.{k}.amount"), amount.rupees().to_string());
            }
            if let Some(qty) = item.qty {
                o.put(format!("attention.{k}.qty"), qty.to_string());
            }
            if let Some(days) = item.days {
                o.put(format!("attention.{k}.days"), days.to_string());
            }
        }
    }
    // The day state.
    if let Ok(status) = mandi_services::day::status(&y.w.env.ctx) {
        if let Some(closed) = status.closed_up_to {
            o.put("days.closed_up_to".into(), closed);
        }
        o.put(
            "days.today_open".into(),
            if status.today_open { "yes" } else { "no" }.into(),
        );
    }
    if let Ok(months) = mandi_services::month::list(&y.w.env.ctx) {
        use mandi_services::month::MonthState;
        let to_close = months
            .rows
            .iter()
            .filter(|r| matches!(r.state, MonthState::Ready | MonthState::DaysOpen))
            .count();
        o.put("days.months_to_close".into(), to_close.to_string());
        if let Some(last) = months.rows.iter().find(|r| r.state == MonthState::Closed) {
            o.put("days.last_month_close".into(), last.month.clone());
        }
    }
    // The audit rows of the first three بکری.
    for number in 1..=3i64 {
        let id: Option<i64> = db
            .read(|c| {
                c.query_row(
                    "SELECT id FROM bikri_entries WHERE number = ?1",
                    params![number],
                    |r| r.get(0),
                )
            })
            .ok();
        let Some(id) = id else { continue };
        let page = mandi_reports::audit::list(
            db,
            &mandi_reports::audit::AuditListInput {
                filters: mandi_reports::audit::AuditFilters {
                    entity: "bikri".to_owned(),
                    entity_id: Some(id),
                    ..mandi_reports::audit::AuditFilters::default()
                },
                offset: 0,
                limit: 100,
                sort: None,
                group_by_month: false,
            },
        );
        if let Ok(page) = page {
            o.put(
                format!("audit.bikri{number}.count"),
                page.total_count.to_string(),
            );
        }
    }
    // The rule that wins today for an item with no party (PRD 11.3): the item level, then the item
    // group, then the default; the newer rule wins a tie.
    struct Rule {
        side: String,
        charge: String,
        account: Option<i64>,
        item: Option<i64>,
        group: Option<String>,
        bps: Option<i64>,
        paisa: Option<i64>,
    }
    let rules: Vec<Rule> = db
        .read(|c| {
            let mut st = c.prepare(
                "SELECT side, charge_type, account_id, item_id, item_group, value_bps, value_paisa \
                 FROM charge_rules \
                 WHERE is_active = 1 AND from_date <= ?1 AND (to_date IS NULL OR to_date >= ?1) \
                 ORDER BY from_date DESC, id DESC",
            )?;
            let rows = st.query_map(params![iso_date(today)], |r| {
                Ok(Rule {
                    side: r.get(0)?,
                    charge: r.get(1)?,
                    account: r.get(2)?,
                    item: r.get(3)?,
                    group: r.get(4)?,
                    bps: r.get(5)?,
                    paisa: r.get(6)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, rusqlite::Error>>()
        })
        .unwrap_or_default();
    // The seller parties asked (a seller's own rules apply to the seller side only), and none.
    let sellers: Vec<(Option<(&'static str, i64)>, String)> =
        std::iter::once((None, String::new()))
            .chain(
                y.sellers
                    .iter()
                    .filter(|s| s.key == "rahim" || s.key == "karim")
                    .map(|s| (Some((s.key, s.id)), format!("{}.", s.key))),
            )
            .collect();
    for item in &y.items {
        let group: Option<String> = db
            .read(|c| {
                c.query_row(
                    "SELECT item_group FROM items WHERE id = ?1",
                    params![item.id],
                    |r| r.get(0),
                )
            })
            .ok();
        let name = item_name(item.key);
        for (party, prefix) in &sellers {
            for (side, charge) in [
                ("seller", "commission"),
                ("seller", "labour"),
                ("seller", "munshiana"),
                ("seller", "phone"),
                ("buyer", "commission"),
                ("buyer", "laga"),
            ] {
                let of = || {
                    rules
                        .iter()
                        .filter(|r| r.side == side && r.charge == charge)
                };
                // PRD 11.3: party and item, party, item, item group, default; newest first, so the
                // newer rule wins a tie (FR-11.5).
                let party_id = if side == "seller" {
                    party.map(|(_, id)| id)
                } else {
                    None
                };
                let winner = party_id
                    .and_then(|p| {
                        of().find(|r| {
                            r.account == Some(p) && r.item == Some(item.id) && r.group.is_none()
                        })
                    })
                    .or_else(|| {
                        party_id.and_then(|p| {
                            of().find(|r| {
                                r.account == Some(p) && r.item.is_none() && r.group.is_none()
                            })
                        })
                    })
                    .or_else(|| {
                        of().find(|r| {
                            r.account.is_none() && r.item == Some(item.id) && r.group.is_none()
                        })
                    })
                    .or_else(|| {
                        of().find(|r| {
                            r.account.is_none()
                                && r.item.is_none()
                                && r.group.is_some()
                                && r.group == group
                        })
                    })
                    .or_else(|| {
                        of().find(|r| r.account.is_none() && r.item.is_none() && r.group.is_none())
                    });
                let text = match winner {
                    Some(Rule { bps: Some(bps), .. }) => {
                        format!("{}.{:02}%", bps / 100, bps % 100)
                    }
                    Some(Rule {
                        paisa: Some(paisa), ..
                    }) => (paisa / 100).to_string(),
                    _ => "none".to_owned(),
                };
                o.put(format!("rules.{prefix}{name}.{side}_{charge}"), text);
            }
        }
    }
}

fn iso_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// The vegetables and the fruit, by item and as a group (Umar, 06-11-2026: the assistant must handle vegetables as
/// well as fruit). All from the books and the screens:
///
/// - `sale_lines.<item>` and `sale_months.<item>`: the active sale lines of an item and the months they fall in (SQL).
/// - `group.<fruit|vegetable>.<period>.{qty,gross,total_comm,laga}`: the sum of view A's item rows of the group,
///   for `today`, `month`, `last_month`, `year` and `all`. A period with no row for an item counts 0.
/// - `veg.laga_lines`: the sale lines of a vegetable that carry a لاگا. It must be 0 (PRD 4.2: the لاگا rule is for
///   the fruit group), so no answer can honestly give a لاگا on a vegetable line.
fn vegetable_facts(o: &mut Oracle, y: &Year) {
    use mandi_services::db::rusqlite::{self, params};
    let db = y.w.env.ctx.db();
    let group_of = |id: i64| -> String {
        db.read(|c| {
            c.query_row(
                "SELECT item_group FROM items WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
        })
        .unwrap_or_else(|e: rusqlite::Error| panic!("the group of item {id}: {e}"))
    };
    for item in &y.items {
        let name = item_name(item.key);
        let (lines, months): (i64, i64) = db
            .read(|c| {
                c.query_row(
                    "SELECT COUNT(*), COUNT(DISTINCT substr(s.entry_date, 1, 7)) \
                     FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
                     WHERE s.is_active = 1 AND l.item_id = ?1",
                    params![item.id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
            })
            .unwrap_or((0, 0));
        o.put(format!("sale_lines.{name}"), lines.to_string());
        o.put(format!("sale_months.{name}"), months.to_string());
    }
    for group in ["fruit", "vegetable"] {
        let names: Vec<&str> = y
            .items
            .iter()
            .filter(|item| group_of(item.id) == group)
            .map(|item| item_name(item.key))
            .collect();
        for period in ["today", "month", "last_month", "year", "all"] {
            for column in ["qty", "gross", "total_comm", "laga"] {
                let sum: i64 = names
                    .iter()
                    .filter_map(|n| o.facts.get(&format!("a.{period}.{n}.{column}")))
                    .filter_map(|v| amount_of(v))
                    .sum();
                o.put(format!("group.{group}.{period}.{column}"), sum.to_string());
            }
        }
    }
    // The vegetable with the most commission in the year and in all time (its English name).
    for period in ["year", "all"] {
        let mut best: Option<(i64, i64)> = None;
        for item in y
            .items
            .iter()
            .filter(|item| group_of(item.id) == "vegetable")
        {
            let name = item_name(item.key);
            let comm = o
                .facts
                .get(&format!("a.{period}.{name}.total_comm"))
                .and_then(|v| amount_of(v))
                .unwrap_or(0);
            if best.is_none_or(|(top, _)| comm > top) {
                best = Some((comm, item.id));
            }
        }
        if let Some((_, id)) = best {
            let english: Option<String> = db
                .read(|c| {
                    c.query_row(
                        "SELECT name_en FROM items WHERE id = ?1",
                        params![id],
                        |r| r.get(0),
                    )
                })
                .ok();
            if let Some(english) = english {
                o.put(format!("group_top.vegetable.{period}"), english);
            }
        }
    }
    // More about each vegetable (Umar, 06-11-2026): what is left, the month against the last month with the zeros
    // written out, the last rate, the buyer and the seller who took the most, and the best seller of the group.
    let mut stock_total = 0i64;
    let mut best_gross: Option<(i64, String)> = None;
    let mut best_qty: Option<(i64, String)> = None;
    for item in y
        .items
        .iter()
        .filter(|item| group_of(item.id) == "vegetable")
    {
        let name = item_name(item.key);
        let fact =
            |key: String| -> i64 { o.facts.get(&key).and_then(|v| amount_of(v)).unwrap_or(0) };
        let stock = fact(format!("stock.{name}.qty"));
        stock_total += stock;
        let (month, last) = (
            fact(format!("a.month.{name}.gross")),
            fact(format!("a.last_month.{name}.gross")),
        );
        let (year_gross, year_qty) = (
            fact(format!("a.year.{name}.gross")),
            fact(format!("a.year.{name}.qty")),
        );
        o.put(format!("veg.stock.{name}"), stock.to_string());
        o.put(format!("veg.month.{name}.gross"), month.to_string());
        o.put(format!("veg.last_month.{name}.gross"), last.to_string());
        o.put(format!("veg.month_vs_last.{name}"), difference(month, last));
        if best_gross.as_ref().is_none_or(|(g, _)| year_gross > *g) {
            best_gross = Some((year_gross, item.id.to_string()));
        }
        if best_qty.as_ref().is_none_or(|(q, _)| year_qty > *q) {
            best_qty = Some((year_qty, item.id.to_string()));
        }
        // The last sale line of the item: its rate and its day.
        let last_line: Option<(i64, String)> = db
            .read(|c| {
                c.query_row(
                    "SELECT s.seller_rate_paisa, s.entry_date FROM bikri_sales s \
                     JOIN bikri_lots l ON l.id = s.lot_id JOIN bikri_entries e ON e.id = s.bikri_id \
                     WHERE s.is_active = 1 AND l.item_id = ?1 \
                     ORDER BY s.entry_date DESC, e.number DESC, s.lot_id DESC, s.line_no DESC, s.id DESC LIMIT 1",
                    params![item.id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
            })
            .ok();
        if let Some((rate, day)) = last_line {
            o.put(format!("veg.last_rate.{name}"), (rate / 100).to_string());
            o.put(format!("veg.last_rate_day.{name}"), day);
        }
        // The buyer and the seller with the most money on this item, in all the books (a system account such
        // as the cash buyer is not a person).
        let top = |sql: &str| -> Option<String> {
            db.read(|c| c.query_row(sql, params![item.id], |r| r.get(0)))
                .ok()
        };
        if let Some(buyer) = top(
            "SELECT a.name_en FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
             JOIN accounts a ON a.id = s.buyer_account_id \
             WHERE s.is_active = 1 AND l.item_id = ?1 AND a.is_system = 0 \
             GROUP BY a.id ORDER BY SUM(s.buyer_value_paisa) DESC, a.id LIMIT 1",
        ) {
            o.put(format!("veg.top_buyer.{name}"), buyer);
        }
        if let Some(seller) = top(
            "SELECT a.name_en FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
             JOIN accounts a ON a.id = l.seller_account_id \
             WHERE s.is_active = 1 AND l.item_id = ?1 AND a.is_system = 0 \
             GROUP BY a.id ORDER BY SUM(s.seller_amount_paisa) DESC, a.id LIMIT 1",
        ) {
            o.put(format!("veg.top_seller.{name}"), seller);
        }
    }
    o.put("veg.stock.total".into(), stock_total.to_string());
    for (what, best) in [("gross", best_gross), ("qty", best_qty)] {
        if let Some((_, id)) = best {
            let english: Option<String> = db
                .read(|c| {
                    c.query_row(
                        "SELECT name_en FROM items WHERE id = ?1",
                        params![id.parse::<i64>().unwrap_or(0)],
                        |r| r.get(0),
                    )
                })
                .ok();
            if let Some(english) = english {
                o.put(format!("group_best.vegetable.year.{what}"), english);
            }
        }
    }
    let laga_lines: i64 = db
        .read(|c| {
            c.query_row(
                "SELECT COUNT(*) FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
                 JOIN items i ON i.id = l.item_id \
                 WHERE s.is_active = 1 AND i.item_group = 'vegetable' AND s.laga_paisa <> 0",
                [],
                |r| r.get(0),
            )
        })
        .unwrap_or(-1);
    o.put("veg.laga_lines".into(), laga_lines.to_string());
}

/// The periods of the item axis of the capability matrix (`scripts/ai-eval/capability-items.tsv`): the ones a
/// question names (today, yesterday, a named day, this and last week, this and last month, a named month, this
/// and last year, a range, "so far"), and the period before each, for "against the last ...".
const IX_PERIODS: [&str; 18] = [
    "today",
    "yesterday",
    "d2026_11_03",
    "d2026_10_14",
    "d2026_10_13",
    "week",
    "last_week",
    "last_week2",
    "month",
    "last_month",
    "m2026_09",
    "m2026_08",
    "year",
    "last_year",
    "last_year2",
    "r2026_10_01_2026_10_15",
    "r2026_09_16_2026_09_30",
    "up_to_now",
];

/// The period before one of the item axis.
fn ix_prev(period: &str) -> Option<&'static str> {
    Some(match period {
        "today" => "yesterday",
        "yesterday" => "d2026_11_03",
        "d2026_10_14" => "d2026_10_13",
        "week" => "last_week",
        "last_week" => "last_week2",
        "month" => "last_month",
        "last_month" => "m2026_09",
        "m2026_09" => "m2026_08",
        "year" => "last_year",
        "last_year" => "last_year2",
        "r2026_10_01_2026_10_15" => "r2026_09_16_2026_09_30",
        _ => return None,
    })
}

/// The money and quantities of view A for one item (or one group, or all) in one period.
#[derive(Clone, Copy, Default)]
struct IxRow {
    qty: i64,
    gross: i64,
    seller: i64,
    buyer: i64,
    laga: i64,
    total: i64,
}

/// The item axis (Umar, 06-11-2026): for every item, the two groups and all items, in every period of
/// [`IX_PERIODS`], from the screens (view A, B and C) and the books (SQL), never from the assistant's tools:
/// `ix.<period>.<unit>.{gross,qty,comm_seller,comm_buyer,laga,comm_total,avg_rate,vs_gross,vs_qty}`, and for an
/// item `{rank,last_rate,buyers_n,sellers_n,top_buyer,top_seller}`; for Banana and Potato with Faisal as the buyer
/// `ix.<period>.<item>.faisal.*`; `ix.<period>.<unit>.{months,best_month,best_month_gross}` for a year, last year
/// and so far; `ix.stock.<unit>`. A unit is an item (its English name in lower case), `fruit`, `vegetable` or
/// `all`. An item with no sale in a period has the zero facts. The zero facts of `a.<period>.<item>` (gross, qty
/// and total_comm) are written for every item too.
fn item_axis_facts(o: &mut Oracle, y: &Year) {
    use mandi_services::db::rusqlite::{self, params};
    let db = y.w.env.ctx.db();
    let items: Vec<(&'static str, i64, bool)> = y
        .items
        .iter()
        .map(|it| {
            let group: String = db
                .read(|c| {
                    c.query_row(
                        "SELECT item_group FROM items WHERE id = ?1",
                        params![it.id],
                        |r| r.get(0),
                    )
                })
                .unwrap_or_else(|e: rusqlite::Error| panic!("group of {}: {e}", it.key));
            (item_name(it.key), it.id, group == "vegetable")
        })
        .collect();
    let faisal = y.buyers.iter().find(|b| b.key == "faisal").map(|b| b.id);
    let money = |cells: &[Cell], at: Option<usize>| -> i64 {
        at.and_then(|i| cells.get(i))
            .and_then(rupees_of)
            .unwrap_or(0)
    };
    let number = |cells: &[Cell], at: Option<usize>| -> i64 {
        match at.and_then(|i| cells.get(i)) {
            Some(Cell::Number { value }) => *value,
            _ => 0,
        }
    };
    // View A for a period (with a buyer if given): the row of each item, and the total.
    let view_a = |period: &str, buyer: Option<i64>| -> (BTreeMap<String, IxRow>, IxRow) {
        let mut f = afilters(period);
        f.buyer = buyer.map(|b| PartyFilter {
            account_id: b,
            person_key: None,
        });
        let page = run_a(y, AnalyticsView::A, f);
        let at = |key: &str| page.columns.iter().position(|c| c.key == key);
        let (qty, gross, seller, buyer_c, laga, total) = (
            at("qty"),
            at("gross"),
            at("seller_comm"),
            at("buyer_comm"),
            at("laga"),
            at("total_comm"),
        );
        let row_of = |cells: &[Cell]| IxRow {
            qty: number(cells, qty),
            gross: money(cells, gross),
            seller: money(cells, seller),
            buyer: money(cells, buyer_c),
            laga: money(cells, laga),
            total: money(cells, total),
        };
        let rows = page
            .rows
            .iter()
            .filter_map(|r| row_slug(&r.cells).map(|(slug, _)| (slug, row_of(&r.cells))))
            .collect();
        (rows, row_of(&page.totals))
    };
    let avg = |r: &IxRow| -> i64 {
        if r.qty <= 0 {
            0
        } else {
            (r.gross * 2 + r.qty) / (r.qty * 2)
        }
    };
    let put_row = |o: &mut Oracle, prefix: &str, r: &IxRow| {
        o.put(format!("{prefix}.gross"), r.gross.to_string());
        o.put(format!("{prefix}.qty"), r.qty.to_string());
        o.put(format!("{prefix}.comm_seller"), r.seller.to_string());
        o.put(format!("{prefix}.comm_buyer"), r.buyer.to_string());
        o.put(format!("{prefix}.laga"), r.laga.to_string());
        o.put(format!("{prefix}.comm_total"), r.total.to_string());
        o.put(format!("{prefix}.avg_rate"), avg(r).to_string());
    };
    // Every period once: the rows of each unit.
    let mut table: BTreeMap<&str, BTreeMap<String, IxRow>> = BTreeMap::new();
    for p in IX_PERIODS {
        let (rows, total) = view_a(p, None);
        let mut units: BTreeMap<String, IxRow> = BTreeMap::new();
        for (name, _, veg) in &items {
            let r = rows.get(*name).copied().unwrap_or_default();
            units.insert((*name).to_owned(), r);
            let group = units
                .entry(if *veg { "vegetable" } else { "fruit" }.to_owned())
                .or_default();
            group.qty += r.qty;
            group.gross += r.gross;
            group.seller += r.seller;
            group.buyer += r.buyer;
            group.laga += r.laga;
            group.total += r.total;
        }
        units.insert("all".to_owned(), total);
        table.insert(p, units);
    }
    for p in IX_PERIODS {
        let units = &table[p];
        // The rank of each item by its gross: 0 when it sold nothing.
        let mut order: Vec<(&str, i64)> = items
            .iter()
            .map(|(n, _, _)| (*n, units[*n].gross))
            .filter(|(_, g)| *g > 0)
            .collect();
        order.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        let (from, to) = period_dates(p);
        for (unit, row) in units {
            put_row(o, &format!("ix.{p}.{unit}"), row);
            if let Some(prev) = ix_prev(p) {
                let before = table[prev][unit];
                o.put(
                    format!("ix.{p}.{unit}.vs_gross"),
                    difference(row.gross, before.gross),
                );
                o.put(
                    format!("ix.{p}.{unit}.vs_qty"),
                    difference(row.qty, before.qty),
                );
            }
        }
        // The item with the most sales of a group, and of all the items (by gross; the first by name on a tie).
        let capital = |name: &str| -> String {
            let mut chars = name.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        };
        for (unit, only) in [
            ("fruit", Some(false)),
            ("vegetable", Some(true)),
            ("all", None),
        ] {
            let top = order.iter().find(|(n, _)| {
                only.is_none_or(|veg| {
                    items
                        .iter()
                        .any(|(name, _, is_veg)| name == n && *is_veg == veg)
                })
            });
            if let Some((name, _)) = top {
                o.put(format!("ix.{p}.{unit}.top_item"), capital(name));
            }
        }
        for (name, id, _) in &items {
            let rank = order
                .iter()
                .position(|(n, _)| n == name)
                .map_or(0, |i| i + 1);
            o.put(format!("ix.{p}.{name}.rank"), rank.to_string());
            // The last line of the item in the period: its rate.
            let last: Option<i64> = db
                .read(|c| {
                    c.query_row(
                        "SELECT s.seller_rate_paisa FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
                         JOIN bikri_entries e ON e.id = s.bikri_id \
                         WHERE s.is_active = 1 AND l.item_id = ?1 AND s.entry_date >= ?2 AND s.entry_date <= ?3 \
                         ORDER BY s.entry_date DESC, e.number DESC, s.lot_id DESC, s.line_no DESC, s.id DESC LIMIT 1",
                        params![
                            id,
                            from.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
                            to.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default()
                        ],
                        |r| r.get(0),
                    )
                })
                .ok();
            o.put(
                format!("ix.{p}.{name}.last_rate"),
                (last.unwrap_or(0) / 100).to_string(),
            );
            // The buyers and the sellers of the item in the period (views C and B with the item).
            let with_item = |view: AnalyticsView| {
                let mut f = afilters(p);
                f.item_id = Some(*id);
                run_a(y, view, f)
            };
            for (view, key, count, value_col, top) in [
                (
                    AnalyticsView::C,
                    "buyer",
                    "buyers_n",
                    "purchase_value",
                    "top_buyer",
                ),
                (
                    AnalyticsView::B,
                    "seller",
                    "sellers_n",
                    "gross",
                    "top_seller",
                ),
            ] {
                let page = with_item(view);
                let value_at = page.columns.iter().position(|c| c.key == value_col);
                let _ = key;
                let mut best: Option<(i64, String)> = None;
                for r in &page.rows {
                    let v = money(&r.cells, value_at);
                    if let Some((_, name)) = r.cells.iter().find_map(name_of).map(|n| (v, n))
                        && best.as_ref().is_none_or(|(b, _)| v > *b)
                    {
                        best = Some((v, name));
                    }
                }
                o.put(
                    format!("ix.{p}.{name}.{count}"),
                    page.total_count.to_string(),
                );
                if let Some((_, who)) = best {
                    o.put(format!("ix.{p}.{name}.{top}"), who);
                }
            }
        }
        // The buyers and the sellers of a group and of all the items, by SQL on the lines (a buyer is an account,
        // or one person of a چکتا کھاتہ account, as the views count them).
        let day = |d: Option<NaiveDate>| {
            d.map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default()
        };
        for (unit, filter) in [
            ("fruit", "AND i.item_group = 'fruit'"),
            ("vegetable", "AND i.item_group = 'vegetable'"),
            ("all", ""),
        ] {
            for (count_key, top_key, who, value) in [
                (
                    "buyers_n",
                    "top_buyer",
                    "COALESCE(NULLIF(s.chukta_name_en, ''), a.name_en)",
                    "s.buyer_value_paisa",
                ),
                (
                    "sellers_n",
                    "top_seller",
                    "COALESCE(NULLIF(l.chukta_name_en, ''), a.name_en)",
                    "s.seller_amount_paisa",
                ),
            ] {
                let party = if count_key == "buyers_n" {
                    "s.buyer_account_id"
                } else {
                    "l.seller_account_id"
                };
                let person = if count_key == "buyers_n" {
                    "s.chukta_person_key"
                } else {
                    "l.chukta_person_key"
                };
                let sql = format!(
                    "SELECT {who}, SUM({value}) AS v FROM bikri_sales s \
                     JOIN bikri_lots l ON l.id = s.lot_id JOIN items i ON i.id = l.item_id \
                     JOIN accounts a ON a.id = {party} \
                     WHERE s.is_active = 1 AND s.entry_date >= ?1 AND s.entry_date <= ?2 {filter} \
                     GROUP BY {party}, COALESCE({person}, '') ORDER BY v DESC, a.id"
                );
                let rows: Vec<(String, i64)> = db
                    .read(|c| {
                        let mut st = c.prepare(&sql)?;
                        let rows = st.query_map(params![day(from), day(to)], |r| {
                            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
                        })?;
                        rows.collect::<Result<Vec<_>, rusqlite::Error>>()
                    })
                    .unwrap_or_default();
                o.put(format!("ix.{p}.{unit}.{count_key}"), rows.len().to_string());
                if let Some((name, _)) = rows.first() {
                    o.put(format!("ix.{p}.{unit}.{top_key}"), name.clone());
                }
            }
        }
        // Banana and Potato for one buyer (Faisal).
        if let Some(f) = faisal {
            let (rows, _) = view_a(p, Some(f));
            let before = ix_prev(p).map(|prev| view_a(prev, Some(f)).0);
            for name in ["banana", "potato"] {
                let r = rows.get(name).copied().unwrap_or_default();
                put_row(o, &format!("ix.{p}.{name}.faisal"), &r);
                // The value with the buyer commission and the laga added (C5), the other true figure of
                // "how much did he buy".
                o.put(
                    format!("ix.{p}.{name}.faisal.with_charges"),
                    (r.gross + r.buyer + r.laga).to_string(),
                );
                // The sellers of that item to him (view B with the item and the buyer).
                if let Some((_, id, _)) = items.iter().find(|(n, _, _)| n == &name) {
                    let mut filters = afilters(p);
                    filters.item_id = Some(*id);
                    filters.buyer = Some(PartyFilter {
                        account_id: f,
                        person_key: None,
                    });
                    let page = run_a(y, AnalyticsView::B, filters);
                    let value_at = page.columns.iter().position(|c| c.key == "gross");
                    o.put(
                        format!("ix.{p}.{name}.faisal.sellers_n"),
                        page.total_count.to_string(),
                    );
                    let mut best: Option<(i64, String)> = None;
                    for row in &page.rows {
                        let v = money(&row.cells, value_at);
                        if let Some(who) = row.cells.iter().find_map(name_of)
                            && best.as_ref().is_none_or(|(b, _)| v > *b)
                        {
                            best = Some((v, who));
                        }
                    }
                    if let Some((_, who)) = best {
                        o.put(format!("ix.{p}.{name}.faisal.top_seller"), who);
                    }
                    // The last rate he paid for the item in the period (R13 with the item, "akram ko kela aakhri kis
                    // rate par diya").
                    let last: Option<i64> = db
                        .read(|c| {
                            c.query_row(
                                "SELECT s.seller_rate_paisa FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
                                 JOIN bikri_entries e ON e.id = s.bikri_id \
                                 WHERE s.is_active = 1 AND s.buyer_account_id = ?1 AND l.item_id = ?2 \
                                 AND s.entry_date >= ?3 AND s.entry_date <= ?4 \
                                 ORDER BY s.entry_date DESC, e.number DESC, s.lot_id DESC, s.line_no DESC, s.id DESC LIMIT 1",
                                params![f, id, day(from), day(to)],
                                |r| r.get(0),
                            )
                        })
                        .ok();
                    if let Some(rate) = last {
                        o.put(
                            format!("ix.{p}.{name}.faisal.last_rate"),
                            (rate / 100).to_string(),
                        );
                    }
                }
                if let Some(before) = &before {
                    let b = before.get(name).copied().unwrap_or_default();
                    o.put(
                        format!("ix.{p}.{name}.faisal.vs_gross"),
                        difference(r.gross, b.gross),
                    );
                }
            }
        }
    }
    // Months with sales and the best month, for a year, last year and so far, by SQL on the lines.
    for p in ["year", "last_year", "up_to_now"] {
        let (from, to) = period_dates(p);
        let (from, to) = (
            from.map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            to.map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
        );
        let mut units: Vec<(String, String)> = items
            .iter()
            .map(|(n, id, _)| ((*n).to_owned(), format!("AND l.item_id = {id}")))
            .collect();
        units.push(("fruit".into(), "AND i.item_group = 'fruit'".into()));
        units.push(("vegetable".into(), "AND i.item_group = 'vegetable'".into()));
        units.push(("all".into(), String::new()));
        for (unit, filter) in units {
            let sql = format!(
                "SELECT substr(s.entry_date, 1, 7) AS m, SUM(s.seller_amount_paisa) \
                 FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id JOIN items i ON i.id = l.item_id \
                 WHERE s.is_active = 1 AND s.entry_date >= ?1 AND s.entry_date <= ?2 {filter} \
                 GROUP BY m ORDER BY SUM(s.seller_amount_paisa) DESC, m LIMIT 1"
            );
            let best: Option<(String, i64)> = db
                .read(|c| c.query_row(&sql, params![from, to], |r| Ok((r.get(0)?, r.get(1)?))))
                .ok();
            let count_sql = format!(
                "SELECT COUNT(DISTINCT substr(s.entry_date, 1, 7)) \
                 FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id JOIN items i ON i.id = l.item_id \
                 WHERE s.is_active = 1 AND s.entry_date >= ?1 AND s.entry_date <= ?2 {filter}"
            );
            let months: i64 = db
                .read(|c| c.query_row(&count_sql, params![from, to], |r| r.get(0)))
                .unwrap_or(0);
            o.put(format!("ix.{p}.{unit}.months"), months.to_string());
            if let Some((month, paisa)) = best {
                o.put(format!("ix.{p}.{unit}.best_month"), month);
                o.put(
                    format!("ix.{p}.{unit}.best_month_gross"),
                    (paisa / 100).to_string(),
                );
            }
        }
    }
    // What is left: R10 as the facts of `stock.*` hold it.
    let (mut fruit, mut veg) = (0i64, 0i64);
    for (name, _, is_veg) in &items {
        let qty = o
            .facts
            .get(&format!("stock.{name}.qty"))
            .and_then(|v| amount_of(v))
            .unwrap_or(0);
        o.put(format!("ix.stock.{name}"), qty.to_string());
        if *is_veg {
            veg += qty;
        } else {
            fruit += qty;
        }
    }
    o.put("ix.stock.fruit".into(), fruit.to_string());
    o.put("ix.stock.vegetable".into(), veg.to_string());
    o.put("ix.stock.all".into(), (fruit + veg).to_string());
    // The zero facts of view A: an item with no sale in a period has gross, qty and commission 0 (so that
    // "0 sold" can be checked).
    for p in PERIODS {
        for (name, _, _) in &items {
            for column in ["gross", "qty", "total_comm"] {
                o.facts
                    .entry(format!("a.{p}.{name}.{column}"))
                    .or_insert_with(|| "0".to_owned());
            }
        }
    }
}

/// The sale lines by quality (plan 58 T4, PRD FR-12.42): for every period of [`IX_PERIODS`], every item and all the
/// items, and each quality (`first`, `second`, `third`), by SQL on the lines of the books (a quality is on the line,
/// and no screen groups by it), never from the assistant's tools:
/// `ixq.<period>.<unit>.<quality>.{lines,qty,gross,comm_seller,comm_buyer,laga,comm_total}` always (0 when no line),
/// and `{avg_rate,last_rate,buyers_n,top_buyer,sellers_n,top_seller}` when there is a line. A unit is an item (its
/// English name in lower case) or `all`. The quality changes no figure: every other fact of the oracle is the
/// same with or without it (the offline test `a_quality_changes_no_amount`).
fn quality_facts(o: &mut Oracle, y: &Year) {
    use mandi_services::db::rusqlite::{self, params};
    let db = y.w.env.ctx.db();
    let day = |d: Option<NaiveDate>| {
        d.map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default()
    };
    let units: Vec<(String, Option<i64>)> = y
        .items
        .iter()
        .map(|it| (item_name(it.key).to_owned(), Some(it.id)))
        .chain(std::iter::once(("all".to_owned(), None)))
        .collect();
    let rupees = |paisa: i64| (paisa / 100).to_string();
    for p in IX_PERIODS {
        let (from, to) = period_dates(p);
        let (from_text, to_text) = (day(from), day(to));
        for (unit, item) in &units {
            for quality in ["first", "second", "third"] {
                let prefix = format!("ixq.{p}.{unit}.{quality}");
                let args = params![quality, from_text, to_text, item];
                let (lines, qty, gross, seller, buyer, laga): (i64, i64, i64, i64, i64, i64) = db
                    .read(|c| {
                        c.query_row(
                            "SELECT COUNT(*), COALESCE(SUM(s.qty), 0), COALESCE(SUM(s.seller_amount_paisa), 0), \
                             COALESCE(SUM(s.seller_comm_paisa), 0), COALESCE(SUM(s.buyer_comm_paisa), 0), \
                             COALESCE(SUM(s.laga_paisa), 0) \
                             FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
                             WHERE s.is_active = 1 AND s.grade = ?1 AND s.entry_date >= ?2 AND s.entry_date <= ?3 \
                             AND (?4 IS NULL OR l.item_id = ?4)",
                            args,
                            |r| {
                                Ok((
                                    r.get(0)?,
                                    r.get(1)?,
                                    r.get(2)?,
                                    r.get(3)?,
                                    r.get(4)?,
                                    r.get(5)?,
                                ))
                            },
                        )
                    })
                    .unwrap_or_else(|e: rusqlite::Error| panic!("quality sums {prefix}: {e}"));
                o.put(format!("{prefix}.lines"), lines.to_string());
                o.put(format!("{prefix}.qty"), qty.to_string());
                o.put(format!("{prefix}.gross"), rupees(gross));
                o.put(format!("{prefix}.comm_seller"), rupees(seller));
                o.put(format!("{prefix}.comm_buyer"), rupees(buyer));
                o.put(format!("{prefix}.laga"), rupees(laga));
                o.put(
                    format!("{prefix}.comm_total"),
                    rupees(seller + buyer + laga),
                );
                if lines == 0 {
                    continue;
                }
                if qty > 0 {
                    let rate = (gross / 100 * 2 + qty) / (qty * 2);
                    o.put(format!("{prefix}.avg_rate"), rate.to_string());
                }
                let last: Option<i64> = db
                    .read(|c| {
                        c.query_row(
                            "SELECT s.seller_rate_paisa FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
                             JOIN bikri_entries e ON e.id = s.bikri_id \
                             WHERE s.is_active = 1 AND s.grade = ?1 AND s.entry_date >= ?2 AND s.entry_date <= ?3 \
                             AND (?4 IS NULL OR l.item_id = ?4) \
                             ORDER BY s.entry_date DESC, e.number DESC, s.lot_id DESC, s.line_no DESC, s.id DESC LIMIT 1",
                            params![quality, from_text, to_text, item],
                            |r| r.get(0),
                        )
                    })
                    .ok();
                if let Some(rate) = last {
                    o.put(format!("{prefix}.last_rate"), (rate / 100).to_string());
                }
                // The buyers and the sellers of those lines (a buyer is an account, or one person of a چکتا کھاتہ
                // account, as the views count them).
                for (count_key, top_key, who, value, party, person) in [
                    (
                        "buyers_n",
                        "top_buyer",
                        "COALESCE(NULLIF(s.chukta_name_en, ''), a.name_en)",
                        "s.buyer_value_paisa",
                        "s.buyer_account_id",
                        "s.chukta_person_key",
                    ),
                    (
                        "sellers_n",
                        "top_seller",
                        "COALESCE(NULLIF(l.chukta_name_en, ''), a.name_en)",
                        "s.seller_amount_paisa",
                        "l.seller_account_id",
                        "l.chukta_person_key",
                    ),
                ] {
                    let sql = format!(
                        "SELECT {who}, SUM({value}) AS v FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
                         JOIN accounts a ON a.id = {party} \
                         WHERE s.is_active = 1 AND s.grade = ?1 AND s.entry_date >= ?2 AND s.entry_date <= ?3 \
                         AND (?4 IS NULL OR l.item_id = ?4) \
                         GROUP BY {party}, COALESCE({person}, '') ORDER BY v DESC, a.id"
                    );
                    let rows: Vec<(String, i64)> = db
                        .read(|c| {
                            let mut st = c.prepare(&sql)?;
                            let rows = st
                                .query_map(params![quality, from_text, to_text, item], |r| {
                                    Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
                                })?;
                            rows.collect::<Result<Vec<_>, rusqlite::Error>>()
                        })
                        .unwrap_or_default();
                    o.put(format!("{prefix}.{count_key}"), rows.len().to_string());
                    if let Some((name, _)) = rows.first() {
                        o.put(format!("{prefix}.{top_key}"), name.clone());
                    }
                }
            }
        }
    }
}

/// The print data of R5 "Print All" (every buyer's slip of a day), and the seller bill (R1) and the buyer slips (R5)
/// of one بکری, asked of the app's own print code in both print languages (plan 58 T4, PRD FR-12.42): a line with a
/// quality names it after the item, and a day or a بکری with no quality prints exactly as before. Returns what is
/// wrong, empty when all is right.
pub fn quality_print_problems(y: &Year) -> Vec<String> {
    use mandi_reports::reports::ReportPrintInput;
    use mandi_services::core::auth::Language;
    use mandi_services::db::rusqlite::params;
    let db = y.w.env.ctx.db();
    let today = ddmmyyyy(date(TODAY));
    let mut problems = Vec::new();
    // A بکری of today with qualities, and an old one with none.
    let one = |sql: &str| -> Option<i64> {
        db.read(|c| c.query_row(sql, params![], |r| r.get::<_, i64>(0)))
            .ok()
    };
    let graded = one(
        "SELECT bikri_id FROM bikri_sales WHERE is_active = 1 AND grade IS NOT NULL \
         ORDER BY entry_date DESC, bikri_id DESC LIMIT 1",
    );
    let plain = one(
        "SELECT bikri_id FROM bikri_sales WHERE is_active = 1 GROUP BY bikri_id \
         HAVING MAX(grade IS NOT NULL) = 0 ORDER BY bikri_id LIMIT 1",
    );
    let (Some(graded), Some(plain)) = (graded, plain) else {
        return vec!["the books have no بکری with a quality, or none without".to_owned()];
    };
    for language in [Language::En, Language::Ur] {
        // The print data carries the quality as its key (`"grade":"first"`) or as the Fluent id of its word
        // (`report-grade-first`); the template writes the word after the item in the print language.
        let has_word = |text: &str| {
            ["first", "second", "third"]
                .iter()
                .any(|g| text.contains(&format!("\"grade\":\"{g}\"")))
                || text.contains("report-grade-")
        };
        let print = |id: ReportId, filters: ReportFilters| ReportPrintInput {
            id,
            filters,
            language,
            paper: None,
            sort: None,
            group_by_month: false,
        };
        let tag = format!("{language:?}");
        // R5 on a day with qualities: some slip line names one. On a day with none: no slip does.
        let day_of = |from: &str| ReportFilters {
            from: from.to_owned(),
            to: from.to_owned(),
            ..ReportFilters::default()
        };
        match reports::buyer_bills_print_data(db, &print(ReportId::R5, day_of(&today))) {
            Ok(data) => {
                let text = serde_json::to_string(&data).unwrap_or_default();
                if data.slips.is_empty() {
                    problems.push(format!("R5 {tag} today: no slip, so nothing to check"));
                } else if !has_word(&text) {
                    problems.push(format!("R5 {tag} today: no slip line names a quality"));
                }
            }
            Err(e) => problems.push(format!("R5 {tag} today: {e:?}")),
        }
        let old_day = ddmmyyyy(date((2026, 10, 14)));
        match reports::buyer_bills_print_data(db, &print(ReportId::R5, day_of(&old_day))) {
            Ok(data) => {
                let text = serde_json::to_string(&data).unwrap_or_default();
                if has_word(&text) {
                    problems.push(format!(
                        "R5 {tag} 14-10-2026: a quality word on a day with none"
                    ));
                }
            }
            Err(e) => problems.push(format!("R5 {tag} 14-10-2026: {e:?}")),
        }
        // The seller bill (R1) and the buyer slips (R5) of one بکری, from the app's print code: a بکری with a
        // quality names it in the print language (the key `grade` and its word in `detail`); one with none has
        // neither.
        let words: [&str; 3] = if language == Language::En {
            ["First", "Second", "Third"]
        } else {
            ["اول", "دوم", "سوم"]
        };
        for (bikri, wants_word) in [(graded, true), (plain, false)] {
            for (doc, name) in [
                (
                    mandi_services::core::printing::PrintDoc::SellerBill,
                    "R1 seller bill",
                ),
                (
                    mandi_services::core::printing::PrintDoc::BuyerSlips,
                    "R5 buyer slips",
                ),
            ] {
                let input = mandi_reports::print::BikriPrintDataInput {
                    bikri_id: bikri,
                    doc,
                    language,
                    only_buyer_id: None,
                };
                match mandi_reports::print::bikri_print_data(db, &input) {
                    Ok(data) => {
                        let text = serde_json::to_string(&data).unwrap_or_default();
                        let key = text.contains("\"grade\":\"");
                        let word = words.iter().any(|w| text.contains(w));
                        if wants_word && !(key && word) {
                            // What the first graded line is written as, for whoever fixes it.
                            let sample = serde_json::to_value(&data)
                                .ok()
                                .and_then(|v| {
                                    v["pages"].as_array().and_then(|pages| {
                                        pages.iter().find_map(|page| {
                                            page["lines"].as_array().and_then(|lines| {
                                                lines.iter().find_map(|line| {
                                                    line.get("grade")?;
                                                    line["detail"].as_str().map(str::to_owned)
                                                })
                                            })
                                        })
                                    })
                                })
                                .unwrap_or_default();
                            problems.push(format!(
                                "{name} {tag} bikri {bikri}: the quality is not on the print (key {key}, word {word}), a graded line reads {sample:?}"
                            ));
                        }
                        if !wants_word && (key || word) {
                            problems.push(format!(
                                "{name} {tag} bikri {bikri}: a quality on a بکری with none (key {key}, word {word})"
                            ));
                        }
                    }
                    Err(e) => problems.push(format!("{name} {tag} bikri {bikri}: {e:?}")),
                }
            }
        }
    }
    problems
}

/// The period axis of the money and party subjects (lead, 06-11-2026): receipts (R23), payments (R25), the cash book
/// (R12), the shop and personal expenses by head (R29), the partners (R30, view F), fruit taken (R37, view K), gardens
/// and trips (R36, R33, views I and J), one buyer, one seller and one collector, capital in and out, and the balance
/// lists as of the last day of the period (R20, R24, R26, R32), for every period of [`IX_PERIODS`], each from the
/// screen the app has for it, never the assistant's tools:
///
/// - `mx.<period>.<r12|r23|r25|r37|r33|r36>.*` and `mx.<period>.<r29|r30>.<row>.*`: the totals, the count and the rows;
/// - `mx.<period>.<f|i|j|k>.<row>.*`: the analytics views;
/// - `mx.<period>.r23.<buyer>.*`, `mx.<period>.c.<buyer>.*` (what a buyer paid and bought), `mx.<period>.r25.<seller>.*`,
///   `mx.<period>.b.<seller>.*` (what a seller was paid and sold), `mx.<period>.r23.rasheed.*` (a collector's receipts);
/// - `mx.<period>.r23.top_party`, `.top_amount` (the biggest receipt), the same for `r25`, `mx.<period>.r29.top_head`,
///   `mx.<period>.r30.top_partner`;
/// - `mx.<period>.capital.{in,out}` and `mx.<period>.capital.b.{in,out}`: partner capital invested and taken back (SQL on
///   the vouchers);
/// - `mx.asof.<period>.<r20|r24|r26|r32>.*` and `.top_party`: the balance lists as of the last day of the period;
/// - `mx.<period>.vs.<name>`: a figure against the same figure of the period before (`1200 up`, `300 down`, `0`).
fn money_axis_facts(o: &mut Oracle, y: &Year) {
    use mandi_services::db::rusqlite::params;
    let db = y.w.env.ctx.db();
    let collector: Option<i64> = db
        .read(|c| {
            c.query_row("SELECT id FROM staff WHERE name_en = 'Rasheed'", [], |r| {
                r.get(0)
            })
        })
        .ok();
    let party_of = |id: i64| {
        Some(PartyFilter {
            account_id: id,
            person_key: None,
        })
    };
    let buyers: Vec<(String, i64)> = ["akram_toll_tax", "faisal"]
        .iter()
        .filter_map(|k| {
            y.buyers
                .iter()
                .find(|b| b.key == *k)
                .map(|b| ((*k).to_owned(), b.id))
        })
        .collect();
    let sellers: Vec<(String, i64)> = ["rahim", "karim"]
        .iter()
        .filter_map(|k| {
            y.sellers
                .iter()
                .find(|s| s.key == *k)
                .map(|s| ((*k).to_owned(), s.id))
        })
        .collect();
    let big = |id: ReportId, filters: ReportFilters| -> Option<ReportPage> {
        reports::run(
            db,
            &ReportRunInput {
                id,
                filters,
                offset: 0,
                limit: 1_000,
                sort: None,
                group_by_month: false,
            },
        )
        .ok()
    };
    let top_of = |page: &ReportPage, col: &str| -> Option<(String, i64)> {
        let at = page.columns.iter().position(|c| c.key == col)?;
        page.rows
            .iter()
            .filter_map(|r| {
                let name = r.cells.iter().find_map(name_of)?;
                let v = rupees_of(r.cells.get(at)?)?;
                Some((name, v))
            })
            .filter(|(_, v)| *v > 0)
            .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
    };
    let day = |d: NaiveDate| d.format("%Y-%m-%d").to_string();
    for p in IX_PERIODS {
        let (from, to) = period_dates(p);
        // The registers and lists of the period.
        for (id, name, rows) in [
            (ReportId::R12, "r12", false),
            (ReportId::R23, "r23", false),
            (ReportId::R25, "r25", false),
            (ReportId::R29, "r29", true),
            (ReportId::R30, "r30", true),
            (ReportId::R37, "r37", false),
            (ReportId::R33, "r33", false),
            (ReportId::R36, "r36", false),
        ] {
            let Some(page) = big(id, rfilters(p)) else {
                continue;
            };
            dump_report_rows(o, &format!("mx.{p}.{name}"), &page, rows);
            let top = match id {
                ReportId::R23 | ReportId::R25 => top_of(&page, "amount"),
                ReportId::R29 => top_of(&page, "expense"),
                ReportId::R30 => top_of(&page, "profit_share"),
                _ => None,
            };
            if let Some((who, amount)) = top {
                match id {
                    ReportId::R29 => o.put(format!("mx.{p}.r29.top_head"), who),
                    ReportId::R30 => o.put(format!("mx.{p}.r30.top_partner"), who),
                    _ => {
                        o.put(format!("mx.{p}.{name}.top_party"), who);
                        o.put(format!("mx.{p}.{name}.top_amount"), amount.to_string());
                    }
                }
            }
        }
        for (view, name) in [
            (AnalyticsView::F, "f"),
            (AnalyticsView::I, "i"),
            (AnalyticsView::J, "j"),
            (AnalyticsView::K, "k"),
        ] {
            dump_analytics(o, &format!("mx.{p}.{name}"), &run_a(y, view, afilters(p)));
        }
        // One buyer: what he paid (R23) and what he bought (view C).
        for (key, id) in &buyers {
            let paid = ReportFilters {
                party: party_of(*id),
                ..rfilters(p)
            };
            if let Some(page) = big(ReportId::R23, paid) {
                dump_report_rows(o, &format!("mx.{p}.r23.{key}"), &page, false);
            }
            let mut f = afilters(p);
            f.buyer = party_of(*id);
            let page = run_a(y, AnalyticsView::C, f);
            dump_analytics(o, &format!("mx.{p}.c.{key}"), &page);
            let at = |col: &str| page.columns.iter().position(|c| c.key == col);
            let sum = |col: &str| {
                at(col)
                    .and_then(|i| page.totals.get(i))
                    .and_then(rupees_of)
                    .unwrap_or(0)
            };
            o.put(
                format!("mx.{p}.c.{key}.bought_with_charges"),
                (sum("purchase_value") + sum("buyer_comm") + sum("laga")).to_string(),
            );
        }
        // One seller: what was paid to him (R25) and what he sold (view B).
        for (key, id) in &sellers {
            let paid = ReportFilters {
                party: party_of(*id),
                ..rfilters(p)
            };
            if let Some(page) = big(ReportId::R25, paid) {
                dump_report_rows(o, &format!("mx.{p}.r25.{key}"), &page, false);
            }
            let mut f = afilters(p);
            f.seller_id = Some(*id);
            dump_analytics(
                o,
                &format!("mx.{p}.b.{key}"),
                &run_a(y, AnalyticsView::B, f),
            );
        }
        // One collector's receipts.
        if let Some(id) = collector {
            let f = ReportFilters {
                collector_id: Some(id),
                ..rfilters(p)
            };
            if let Some(page) = big(ReportId::R23, f) {
                dump_report_rows(o, &format!("mx.{p}.r23.rasheed"), &page, false);
            }
        }
        // Partner capital invested and taken back, in all and for partner B.
        let (from_text, to_text) = (
            from.map(day).unwrap_or_default(),
            to.map(day).unwrap_or_default(),
        );
        let partner_b = y.partner_ids[1];
        for (kind, label) in [("capital_in", "in"), ("capital_out", "out")] {
            for (suffix, partner) in [("", None), (".b", Some(partner_b))] {
                let total: i64 = db
                    .read(|c| {
                        c.query_row(
                            "SELECT COALESCE(SUM(amount_paisa), 0) FROM vouchers \
                             WHERE status = 'saved' AND type = ?1 AND entry_date >= ?2 AND entry_date <= ?3 \
                             AND (?4 IS NULL OR partner_id = ?4)",
                            params![kind, from_text, to_text, partner],
                            |r| r.get(0),
                        )
                    })
                    .unwrap_or(0);
                o.put(
                    format!("mx.{p}.capital{suffix}.{label}"),
                    (total / 100).to_string(),
                );
            }
        }
        // The balance lists as of the last day of the period.
        let end = to.unwrap_or_else(|| date(TODAY));
        let as_of = ReportFilters {
            as_of: ddmmyyyy(end),
            ..ReportFilters::default()
        };
        for (id, name, col) in [
            (ReportId::R20, "r20", "balance"),
            (ReportId::R24, "r24", "payable"),
            (ReportId::R26, "r26", "open"),
            (ReportId::R32, "r32", "open"),
        ] {
            let Some(page) = big(id, as_of.clone()) else {
                continue;
            };
            dump_report_rows(o, &format!("mx.asof.{p}.{name}"), &page, true);
            if let Some((who, _)) = top_of(&page, col) {
                o.put(format!("mx.asof.{p}.{name}.top_party"), who);
            }
        }
    }
    // Each figure against the same figure of the period before.
    let mut figures: Vec<(String, String)> = vec![
        ("receipts".into(), "r23.total.amount".into()),
        ("payments".into(), "r25.total.amount".into()),
        ("cash_in".into(), "r12.total.cash_in".into()),
        ("cash_out".into(), "r12.total.cash_out".into()),
        ("expense".into(), "r29.total.expense".into()),
        ("taken".into(), "r29.total.taken".into()),
        ("returned".into(), "r29.total.returned".into()),
        ("fruit".into(), "r37.total.value".into()),
        ("collected".into(), "r23.rasheed.total.amount".into()),
        ("capital_in".into(), "capital.in".into()),
        ("capital_out".into(), "capital.out".into()),
        ("capital_b_in".into(), "capital.b.in".into()),
        ("electricity".into(), "r29.electricity.expense".into()),
        ("shop_rent".into(), "r29.shop_rent.expense".into()),
        ("tea".into(), "r29.tea.expense".into()),
        ("salaries".into(), "r29.salaries.expense".into()),
        ("profit_a".into(), "r30.partner_a.profit_share".into()),
        ("profit_b".into(), "r30.partner_b.profit_share".into()),
        ("taken_a".into(), "r30.partner_a.taken".into()),
    ];
    for (key, _) in &buyers {
        figures.push((format!("paid_{key}"), format!("r23.{key}.total.amount")));
        figures.push((
            format!("bought_{key}"),
            format!("c.{key}.total.purchase_value"),
        ));
    }
    for (key, _) in &sellers {
        figures.push((format!("paid_to_{key}"), format!("r25.{key}.total.amount")));
        figures.push((format!("sold_{key}"), format!("b.{key}.total.gross")));
    }
    for p in IX_PERIODS {
        let Some(prev) = ix_prev(p) else { continue };
        for (name, key) in &figures {
            let now = o
                .facts
                .get(&format!("mx.{p}.{key}"))
                .and_then(|v| amount_of(v));
            let before = o
                .facts
                .get(&format!("mx.{prev}.{key}"))
                .and_then(|v| amount_of(v));
            if let (Some(a), Some(b)) = (now, before) {
                o.put(format!("mx.{p}.vs.{name}"), difference(a, b));
            }
        }
    }
}

/// The keys of a page's columns as text slices.
fn as_strs(keys: &[String]) -> Vec<&str> {
    keys.iter().map(String::as_str).collect()
}

/// A money column of a total row, by the column's key (0 when the screen has no such column).
fn money_total(columns: &[&str], totals: &[Cell], key: &str) -> mandi_services::core::money::Paisa {
    columns
        .iter()
        .position(|c| *c == key)
        .and_then(|at| totals.get(at))
        .and_then(|cell| match cell {
            Cell::Money { paisa } => Some(*paisa),
            _ => None,
        })
        .unwrap_or(mandi_services::core::money::Paisa::ZERO)
}

/// The series the estimate tool reads beyond view G's four columns (plan 55 Section 11.1, F1 to F3),
/// each worked out from the screen it adds up to, one run of the screen for each calendar month, never
/// from the tool's own queries:
///
/// - `est.commission_buyer`, `est.laga`: view A, its total row (buyer_comm, laga).
/// - `est.receipts`: R23, its total row (amount).
/// - `est.garden_sales`: view I, sales_our_shop + sales_other_shops.
/// - `est.trip_sales`: view J, sales_money.
/// - `est_head.<head>`: R29, the head's row (expense).
/// - `est_partner.<a|b|c>.profit`: view G with the partner, the profit column.
///
/// The keys under each prefix are those of [`put_series_estimates`].
fn screen_series_estimates(o: &mut Oracle, y: &Year) {
    use mandi_services::core::money::Paisa;
    use mandi_services::domain::forecast::YearMonth;
    let today = date(TODAY);
    let Some(current) = YearMonth::new(today.year(), today.month()) else {
        return;
    };
    let months = months_before(current);
    let mut buyer_comm = Vec::new();
    let mut laga = Vec::new();
    let mut receipts = Vec::new();
    let mut garden = Vec::new();
    let mut trip = Vec::new();
    let mut heads: BTreeMap<String, BTreeMap<YearMonth, Paisa>> = BTreeMap::new();
    for (month, from, to) in &months {
        let (from, to) = (ddmmyyyy(*from), ddmmyyyy(*to));
        let a = |view| {
            let page = run_a(
                y,
                view,
                AnalyticsFilters {
                    from: from.clone(),
                    to: to.clone(),
                    ..AnalyticsFilters::default()
                },
            );
            let keys: Vec<String> = page.columns.iter().map(|c| c.key.clone()).collect();
            (keys, page.totals)
        };
        let (keys, totals) = a(AnalyticsView::A);
        buyer_comm.push((*month, money_total(&as_strs(&keys), &totals, "buyer_comm")));
        laga.push((*month, money_total(&as_strs(&keys), &totals, "laga")));
        let (keys, totals) = a(AnalyticsView::I);
        let keys = as_strs(&keys);
        let ours = money_total(&keys, &totals, "sales_our_shop");
        let other = money_total(&keys, &totals, "sales_other_shops");
        garden.push((*month, ours.checked_add(other).unwrap_or(ours)));
        let (keys, totals) = a(AnalyticsView::J);
        trip.push((*month, money_total(&as_strs(&keys), &totals, "sales_money")));
        let r = |id| {
            try_r(
                y,
                id,
                ReportFilters {
                    from: from.clone(),
                    to: to.clone(),
                    ..ReportFilters::default()
                },
            )
        };
        if let Some(page) = r(ReportId::R23) {
            let keys: Vec<&str> = page.columns.iter().map(|c| c.key.as_str()).collect();
            receipts.push((*month, money_total(&keys, &page.totals, "amount")));
        }
        if let Some(page) = r(ReportId::R29) {
            let at = page.columns.iter().position(|c| c.key == "expense");
            for row in &page.rows {
                let (Some((slug, _)), Some(at)) = (row_slug(&row.cells), at) else {
                    continue;
                };
                if let Some(Cell::Money { paisa }) = row.cells.get(at) {
                    heads.entry(slug).or_default().insert(*month, *paisa);
                }
            }
        }
    }
    for (prefix, series) in [
        ("est.commission_buyer", &buyer_comm),
        ("est.laga", &laga),
        ("est.receipts", &receipts),
        ("est.garden_sales", &garden),
        ("est.trip_sales", &trip),
    ] {
        put_series_estimates(o, prefix, &trimmed(series), current);
    }
    for (head, by_month) in &heads {
        let series: Vec<(YearMonth, Paisa)> = months
            .iter()
            .map(|(m, _, _)| (*m, by_month.get(m).copied().unwrap_or(Paisa::ZERO)))
            .collect();
        put_series_estimates(o, &format!("est_head.{head}"), &trimmed(&series), current);
    }
    let last_full = today.with_day(1).unwrap_or(today) - Duration::days(1);
    for (name, id) in ["a", "b", "c"].iter().zip(y.partner_ids) {
        let mut f = afilters("all");
        f.to = ddmmyyyy(last_full);
        f.group_by = Some(analytics::AnalyticsGroupBy::Month);
        f.partner_id = Some(id);
        let page = run_a(y, AnalyticsView::G, f);
        if let Some(at) = page.columns.iter().position(|c| c.key == "profit") {
            put_series_estimates(
                o,
                &format!("est_partner.{name}.profit"),
                &history_of(&page, at, current),
                current,
            );
        }
    }
}

// ---- D7: the picky real-time bank (plan 59, B1) ----------------------------------------------------------------
//
// Every fact below is SQL on the books (`bikri_sales`, `bikri_lots`, `vouchers`, `journal_lines`), never the
// assistant's tools and never typed. A buyer's day is cross-checked against the app's own day sheet (R35) when the
// oracle is built, so a mistake in this SQL stops the build instead of reaching the bank.

/// One combination of filters for the D7 sale facts.
struct D7Combo {
    name: &'static str,
    buyer: Option<&'static str>,
    seller: Option<&'static str>,
    item: Option<&'static str>,
    group: Option<&'static str>,
    pay: Option<&'static str>,
    grade: Option<&'static str>,
    period: &'static str,
}

/// A combination with no filter at all; each combination below names the filters it sets.
const D7_NONE: D7Combo = D7Combo {
    name: "",
    buyer: None,
    seller: None,
    item: None,
    group: None,
    pay: None,
    grade: None,
    period: "year",
};

/// The combinations the D7 bank asks (several filters at once; some are empty on purpose).
const D7_COMBOS: [D7Combo; 33] = [
    D7Combo {
        name: "faisal_rahim_last_month",
        buyer: Some("faisal"),
        seller: Some("rahim"),
        period: "last_month",
        ..D7_NONE
    },
    D7Combo {
        name: "faisal_karim_year",
        buyer: Some("faisal"),
        seller: Some("karim"),
        ..D7_NONE
    },
    D7Combo {
        name: "rizwan_banana_year",
        buyer: Some("rizwan"),
        item: Some("kela"),
        ..D7_NONE
    },
    D7Combo {
        name: "rahim_banana_year",
        seller: Some("rahim"),
        item: Some("kela"),
        ..D7_NONE
    },
    D7Combo {
        name: "karim_apple_last_month",
        seller: Some("karim"),
        item: Some("saib"),
        period: "last_month",
        ..D7_NONE
    },
    D7Combo {
        name: "banana_cash_year",
        item: Some("kela"),
        pay: Some("cash"),
        ..D7_NONE
    },
    D7Combo {
        name: "fruit_credit_last_month",
        group: Some("fruit"),
        pay: Some("credit"),
        period: "last_month",
        ..D7_NONE
    },
    D7Combo {
        name: "usman_apple_first_year",
        buyer: Some("usman"),
        item: Some("saib"),
        grade: Some("first"),
        ..D7_NONE
    },
    D7Combo {
        name: "usman_apple_first_last_month",
        buyer: Some("usman"),
        item: Some("saib"),
        grade: Some("first"),
        period: "last_month",
        ..D7_NONE
    },
    D7Combo {
        name: "vegetable_cash_last_month",
        group: Some("vegetable"),
        pay: Some("cash"),
        period: "last_month",
        ..D7_NONE
    },
    D7Combo {
        name: "cash_today",
        pay: Some("cash"),
        period: "today",
        ..D7_NONE
    },
    D7Combo {
        name: "credit_today",
        pay: Some("credit"),
        period: "today",
        ..D7_NONE
    },
    D7Combo {
        name: "akram_baloch_mango_today",
        buyer: Some("akram_baloch"),
        item: Some("aam"),
        period: "today",
        ..D7_NONE
    },
    D7Combo {
        name: "nadeem_potato_week",
        buyer: Some("nadeem"),
        item: Some("aloo"),
        period: "week",
        ..D7_NONE
    },
    D7Combo {
        name: "sarfraz_grapes_year",
        seller: Some("sarfraz"),
        item: Some("angoor"),
        ..D7_NONE
    },
    D7Combo {
        name: "akram_baloch_banana_second_last_month",
        buyer: Some("akram_baloch"),
        item: Some("kela"),
        grade: Some("second"),
        period: "last_month",
        ..D7_NONE
    },
    D7Combo {
        name: "shahid_banana_sep",
        buyer: Some("shahid"),
        item: Some("kela"),
        period: "m2026_09",
        ..D7_NONE
    },
    D7Combo {
        name: "tahir_grapes_last_month",
        seller: Some("tahir"),
        item: Some("angoor"),
        period: "last_month",
        ..D7_NONE
    },
    D7Combo {
        name: "mango_credit_month",
        item: Some("aam"),
        pay: Some("credit"),
        period: "month",
        ..D7_NONE
    },
    D7Combo {
        name: "rizwan_all_last_month",
        buyer: Some("rizwan"),
        period: "last_month",
        ..D7_NONE
    },
    D7Combo {
        name: "apple_week",
        item: Some("saib"),
        period: "week",
        ..D7_NONE
    },
    // The newest-line cases of `seed::d7_last_rate_cases` (grapes, 03 to 05-11-2026).
    D7Combo {
        name: "grapes_today",
        item: Some("angoor"),
        period: "today",
        ..D7_NONE
    },
    D7Combo {
        name: "grapes_week",
        item: Some("angoor"),
        period: "week",
        ..D7_NONE
    },
    D7Combo {
        name: "shahid_grapes_today",
        buyer: Some("shahid"),
        item: Some("angoor"),
        period: "today",
        ..D7_NONE
    },
    D7Combo {
        name: "karim_grapes_today",
        seller: Some("karim"),
        item: Some("angoor"),
        period: "today",
        ..D7_NONE
    },
    // A seller and an item with the commission asked (R10), and uncommon period words (R9: may, septmber).
    D7Combo {
        name: "rahim_banana_last_month",
        seller: Some("rahim"),
        item: Some("kela"),
        period: "last_month",
        ..D7_NONE
    },
    D7Combo {
        name: "akram_toll_tax_may",
        buyer: Some("akram_toll_tax"),
        period: "m2026_05",
        ..D7_NONE
    },
    D7Combo {
        name: "faisal_sep",
        buyer: Some("faisal"),
        period: "m2026_09",
        ..D7_NONE
    },
    D7Combo {
        name: "potato_week",
        item: Some("aloo"),
        period: "week",
        ..D7_NONE
    },
    // The cases of `seed::d7_richer_cases`: a ریٹ مختلف line that is the newest apple line of its day, and a later sale
    // of an older بکری on a day that also has a newer بکری of the same item.
    D7Combo {
        name: "apple_oct22",
        item: Some("saib"),
        period: "d2026_10_22",
        ..D7_NONE
    },
    D7Combo {
        name: "mosambi_oct19",
        item: Some("mausami"),
        period: "d2026_10_19",
        ..D7_NONE
    },
    D7Combo {
        name: "naveed_oct14",
        buyer: Some("naveed"),
        period: "d2026_10_14",
        ..D7_NONE
    },
    D7Combo {
        name: "naveed_cash_oct14",
        buyer: Some("naveed"),
        pay: Some("cash"),
        period: "d2026_10_14",
        ..D7_NONE
    },
];

/// True for a fact that depends on the quality of a sale line: the `ixq.*` facts, and the D7 combinations that filter
/// on a quality. Building the books with no quality changes exactly these facts and no other.
pub fn is_quality_fact(key: &str) -> bool {
    key.starts_with("ixq.")
        || D7_COMBOS
            .iter()
            .any(|c| c.grade.is_some() && key.starts_with(&format!("d7.sale.{}.", c.name)))
}

/// The facts of the D7 bank: `d7.bd.*` (a buyer's day), `d7.sale.*` (filter combinations), `d7.recv.*` (goods that
/// came in), `d7.stock.*` (stock by group), `d7.bikri.*` (counts), `d7.partner.*`, `d7.last_bill.*`, `d7.cmp.*`.
fn d7_facts(o: &mut Oracle, y: &Year) {
    use mandi_services::db::rusqlite::{self, params};
    let db = y.w.env.ctx.db();
    let iso = |d: Option<NaiveDate>| {
        d.map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default()
    };
    let rupees = |paisa: i64| (paisa / 100).to_string();
    let buyer_id = |key: &str| -> i64 {
        y.buyers
            .iter()
            .find(|b| b.key == key)
            .map(|b| b.id)
            .unwrap_or_else(|| panic!("no buyer {key}"))
    };
    let seller_id = |key: &str| -> i64 {
        y.sellers
            .iter()
            .find(|s| s.key == key)
            .map(|s| s.id)
            .unwrap_or_else(|| panic!("no seller {key}"))
    };
    let item_id = |key: &str| -> i64 {
        y.items
            .iter()
            .find(|i| i.key == key)
            .map(|i| i.id)
            .unwrap_or_else(|| panic!("no item {key}"))
    };
    let side = |paisa: i64| -> String {
        match paisa.cmp(&0) {
            std::cmp::Ordering::Greater => format!("{} Dr", paisa / 100),
            std::cmp::Ordering::Less => format!("{} Cr", -paisa / 100),
            std::cmp::Ordering::Equal => "0".to_owned(),
        }
    };
    let half_up = |gross: i64, qty: i64| -> i64 {
        if qty <= 0 {
            0
        } else {
            (gross * 2 + qty) / (qty * 2)
        }
    };

    // 1. A buyer's day: each item with its qty and amount, the cash on the spot, the credit left, the balance.
    for (buyer, day) in [
        ("akram_toll_tax", "d2026_10_14"),
        ("rizwan", "d2026_10_14"),
        ("nadeem", "d2026_10_14"),
        ("usman", "d2026_10_14"),
        ("faisal", "today"),
        ("faisal", "yesterday"),
        ("naveed", "d2026_10_14"),
        ("rafique", "d2026_10_14"),
    ] {
        let id = buyer_id(buyer);
        let (from, _) = period_dates(day);
        let date_text = iso(from);
        let prefix = format!("d7.bd.{buyer}.{day}");
        let items: Vec<(String, i64, i64, i64, i64)> = db
            .read(|c| {
                let mut st = c.prepare(
                    "SELECT i.name_en, SUM(s.qty), SUM(s.buyer_value_paisa), COUNT(*), MAX(s.buyer_rate_paisa) \
                     FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id JOIN items i ON i.id = l.item_id \
                     WHERE s.is_active = 1 AND s.buyer_account_id = ?1 AND s.entry_date = ?2 \
                     GROUP BY l.item_id ORDER BY i.name_en",
                )?;
                let rows = st.query_map(params![id, date_text], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
                })?;
                rows.collect::<Result<Vec<_>, rusqlite::Error>>()
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 buyer day {prefix}: {e}"));
        for (name, qty, value, lines, top_rate) in &items {
            let slug = slug(name);
            o.put(format!("{prefix}.{slug}.qty"), qty.to_string());
            o.put(format!("{prefix}.{slug}.amount"), rupees(*value));
            // One line: its own rate; more lines: the average rate of the item that day.
            let rate = if *lines == 1 {
                top_rate / 100
            } else {
                half_up(value / 100, *qty)
            };
            o.put(format!("{prefix}.{slug}.rate"), rate.to_string());
        }
        let (n, value, comm, laga, total, cash, credit): (i64, i64, i64, i64, i64, i64, i64) = db
            .read(|c| {
                c.query_row(
                    "SELECT COUNT(*), COALESCE(SUM(buyer_value_paisa),0), COALESCE(SUM(buyer_comm_paisa),0), \
                     COALESCE(SUM(laga_paisa),0), COALESCE(SUM(buyer_total_paisa),0), \
                     COALESCE(SUM(paid_paisa),0), \
                     COALESCE(SUM(buyer_total_paisa - paid_paisa),0) \
                     FROM bikri_sales WHERE is_active = 1 AND buyer_account_id = ?1 AND entry_date = ?2",
                    params![id, date_text],
                    |r| {
                        Ok((
                            r.get(0)?,
                            r.get(1)?,
                            r.get(2)?,
                            r.get(3)?,
                            r.get(4)?,
                            r.get(5)?,
                            r.get(6)?,
                        ))
                    },
                )
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 buyer day totals {prefix}: {e}"));
        let paid: i64 = db
            .read(|c| {
                c.query_row(
                    "SELECT COALESCE(SUM(amount_paisa),0) FROM vouchers WHERE type = 'receipt' AND status = 'saved' \
                     AND account_id = ?1 AND entry_date = ?2",
                    params![id, date_text],
                    |r| r.get(0),
                )
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 receipts {prefix}: {e}"));
        let balance_end: i64 = db
            .read(|c| {
                c.query_row(
                    "SELECT COALESCE(SUM(jl.dr_paisa),0) - COALESCE(SUM(jl.cr_paisa),0) FROM journal_lines jl \
                     JOIN journal_entries je ON je.id = jl.entry_id \
                     WHERE je.status = 'posted' AND jl.account_id = ?1 AND je.entry_date <= ?2",
                    params![id, date_text],
                    |r| r.get(0),
                )
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 balance {prefix}: {e}"));
        let balance_before: i64 = db
            .read(|c| {
                c.query_row(
                    "SELECT COALESCE(SUM(jl.dr_paisa),0) - COALESCE(SUM(jl.cr_paisa),0) FROM journal_lines jl \
                     JOIN journal_entries je ON je.id = jl.entry_id \
                     WHERE je.status = 'posted' AND jl.account_id = ?1 AND je.entry_date < ?2",
                    params![id, date_text],
                    |r| r.get(0),
                )
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 balance before {prefix}: {e}"));
        o.put(format!("{prefix}.lines"), n.to_string());
        o.put(format!("{prefix}.value"), rupees(value));
        o.put(format!("{prefix}.comm_buyer"), rupees(comm));
        o.put(format!("{prefix}.laga"), rupees(laga));
        o.put(format!("{prefix}.charges"), rupees(total));
        o.put(format!("{prefix}.cash"), rupees(cash));
        o.put(format!("{prefix}.credit"), rupees(credit));
        o.put(format!("{prefix}.paid"), rupees(paid));
        // What stayed unpaid of the day's bill: the day's bill, less the cash on the spot and the receipts of the day.
        let left = total - cash - paid;
        o.put(format!("{prefix}.left"), rupees(left));
        o.put(format!("{prefix}.balance_before"), side(balance_before));
        o.put(format!("{prefix}.balance_end"), side(balance_end));
        // The app's own day sheet (R35) must say the same: it is the check of the SQL above.
        let sheet = ReportFilters {
            as_of: ddmmyyyy(from.unwrap_or_else(|| date(TODAY))),
            party: Some(PartyFilter {
                account_id: id,
                person_key: None,
            }),
            ..ReportFilters::default()
        };
        if let Some(page) = try_r(y, ReportId::R35, sheet) {
            dump_report_rows(o, &format!("d7.r35.{buyer}.{day}"), &page, false);
            let sheet_left =
                amount_of(o.get(&format!("d7.r35.{buyer}.{day}.sum.report_r35_today_left")))
                    .unwrap_or(0);
            let sheet_previous =
                amount_of(o.get(&format!("d7.r35.{buyer}.{day}.sum.report_r35_previous")))
                    .unwrap_or(0);
            assert_eq!(
                sheet_left,
                left / 100,
                "{prefix}: the day sheet R35 says {sheet_left} is left, the books say {}",
                left / 100
            );
            assert_eq!(
                sheet_previous.abs(),
                balance_before.abs() / 100,
                "{prefix}: the day sheet R35 says the balance before was {sheet_previous}, the books say {}",
                balance_before / 100
            );
        }
        o.put(
            format!("{prefix}.balance_now"),
            o.get(&format!("buyer.{buyer}.balance")).to_owned(),
        );
    }

    // 2. Filter combinations: several filters at once (every one must hold).
    for c in &D7_COMBOS {
        let (from, to) = period_dates(c.period);
        let (from, to) = (iso(from), iso(to));
        let buyer = c.buyer.map(buyer_id);
        let seller = c.seller.map(seller_id);
        let item = c.item.map(item_id);
        let (group, pay, grade) = (c.group, c.pay, c.grade);
        let args = params![from, to, buyer, seller, item, group, pay, grade];
        let filter = "s.is_active = 1 AND s.entry_date >= ?1 AND s.entry_date <= ?2 \
                      AND (?3 IS NULL OR s.buyer_account_id = ?3) AND (?4 IS NULL OR l.seller_account_id = ?4) \
                      AND (?5 IS NULL OR l.item_id = ?5) AND (?6 IS NULL OR i.item_group = ?6) \
                      AND (?7 IS NULL OR (?7 = 'cash' AND s.paid_paisa > 0) \
                           OR (?7 = 'credit' AND s.paid_paisa < s.buyer_total_paisa)) AND (?8 IS NULL OR s.grade = ?8)";
        let sql = format!(
            "SELECT COUNT(*), COALESCE(SUM(s.qty),0), COALESCE(SUM(s.buyer_value_paisa),0), \
             COALESCE(SUM(s.seller_amount_paisa),0), COALESCE(SUM(s.seller_comm_paisa),0), \
             COALESCE(SUM(s.buyer_comm_paisa),0), COALESCE(SUM(s.laga_paisa),0), COALESCE(SUM(s.buyer_total_paisa),0), \
             COALESCE(MAX(s.seller_rate_paisa),0), COALESCE(MIN(s.seller_rate_paisa),0), \
             COUNT(DISTINCT s.buyer_account_id), COUNT(DISTINCT l.seller_account_id) \
             FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id JOIN items i ON i.id = l.item_id WHERE {filter}"
        );
        let row: (i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64) = db
            .read(|conn| {
                conn.query_row(&sql, args, |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                        r.get(7)?,
                        r.get(8)?,
                        r.get(9)?,
                        r.get(10)?,
                        r.get(11)?,
                    ))
                })
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 combo {}: {e}", c.name));
        let (
            lines,
            qty,
            gross,
            seller_amount,
            comm_seller,
            comm_buyer,
            laga,
            charges,
            max_rate,
            min_rate,
            buyers_n,
            sellers_n,
        ) = row;
        let prefix = format!("d7.sale.{}", c.name);
        o.put(format!("{prefix}.lines"), lines.to_string());
        o.put(format!("{prefix}.qty"), qty.to_string());
        o.put(format!("{prefix}.gross"), rupees(gross));
        o.put(format!("{prefix}.seller_amount"), rupees(seller_amount));
        o.put(format!("{prefix}.comm_seller"), rupees(comm_seller));
        o.put(format!("{prefix}.comm_buyer"), rupees(comm_buyer));
        o.put(format!("{prefix}.laga"), rupees(laga));
        o.put(
            format!("{prefix}.comm_total"),
            rupees(comm_seller + comm_buyer + laga),
        );
        o.put(format!("{prefix}.charges"), rupees(charges));
        o.put(format!("{prefix}.buyers_n"), buyers_n.to_string());
        o.put(format!("{prefix}.sellers_n"), sellers_n.to_string());
        if lines > 0 {
            o.put(
                format!("{prefix}.avg_rate"),
                half_up(gross / 100, qty).to_string(),
            );
            o.put(format!("{prefix}.max_rate"), (max_rate / 100).to_string());
            o.put(format!("{prefix}.min_rate"), (min_rate / 100).to_string());
            let last_sql = format!(
                "SELECT s.seller_rate_paisa FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
                 JOIN items i ON i.id = l.item_id JOIN bikri_entries e ON e.id = s.bikri_id WHERE {filter} \
                 ORDER BY s.entry_date DESC, e.number DESC, s.lot_id DESC, s.line_no DESC, s.id DESC LIMIT 1"
            );
            let last: i64 = db
                .read(|conn| conn.query_row(&last_sql, args, |r| r.get(0)))
                .unwrap_or_else(|e: rusqlite::Error| panic!("d7 combo last rate {}: {e}", c.name));
            // The buyer's rate of that same newest line (it differs only on a ریٹ مختلف line).
            let last_buyer_sql = last_sql.replace("s.seller_rate_paisa", "s.buyer_rate_paisa");
            let last_buyer: i64 = db
                .read(|conn| conn.query_row(&last_buyer_sql, args, |r| r.get(0)))
                .unwrap_or_else(|e: rusqlite::Error| {
                    panic!("d7 combo last buyer rate {}: {e}", c.name)
                });
            o.put(
                format!("{prefix}.last_buyer_rate"),
                (last_buyer / 100).to_string(),
            );
            o.put(format!("{prefix}.last_rate"), (last / 100).to_string());
            let first_sql = format!(
                "SELECT s.entry_date FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
                 JOIN items i ON i.id = l.item_id JOIN bikri_entries e ON e.id = s.bikri_id WHERE {filter} \
                 ORDER BY s.entry_date ASC, e.number ASC, s.lot_id ASC, s.line_no ASC, s.id ASC LIMIT 1"
            );
            let first: String = db
                .read(|conn| conn.query_row(&first_sql, args, |r| r.get(0)))
                .unwrap_or_else(|e: rusqlite::Error| panic!("d7 combo first date {}: {e}", c.name));
            o.put(format!("{prefix}.first_date"), first);
            let last_date_sql = format!(
                "SELECT s.entry_date FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
                 JOIN items i ON i.id = l.item_id JOIN bikri_entries e ON e.id = s.bikri_id WHERE {filter} \
                 ORDER BY s.entry_date DESC, e.number DESC, s.lot_id DESC, s.line_no DESC, s.id DESC LIMIT 1"
            );
            let last_date: String = db
                .read(|conn| conn.query_row(&last_date_sql, args, |r| r.get(0)))
                .unwrap_or_else(|e: rusqlite::Error| panic!("d7 combo last date {}: {e}", c.name));
            o.put(format!("{prefix}.last_date"), last_date);
        }
    }

    // The newest-line rule (plan 59 Section 11): the line's own date, then the بکری number, then its line number,
    // never the row id. The seed's cases are built so that the row id gives ANOTHER rate; both are checked here,
    // so the cases keep telling the two rules apart.
    let by_row_id = |extra: &str, args: &[&dyn mandi_services::db::rusqlite::ToSql]| -> i64 {
        let sql = format!(
            "SELECT s.seller_rate_paisa FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id \
             WHERE s.is_active = 1 AND l.item_id = ?1 AND s.entry_date = ?2 {extra} \
             ORDER BY s.entry_date DESC, s.id DESC LIMIT 1"
        );
        db.read(|conn| conn.query_row(&sql, args, |r| r.get(0)))
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 rate by row id: {e}"))
    };
    let grapes_id = item_id("angoor");
    let today_text = iso(Some(date(TODAY)));
    let shahid = buyer_id("shahid");
    for (name, want, wrong, by_id) in [
        (
            "grapes_today",
            "5000",
            "5200",
            by_row_id("", params![grapes_id, today_text]),
        ),
        (
            "shahid_grapes_today",
            "4900",
            "5050",
            by_row_id(
                "AND s.buyer_account_id = ?3",
                params![grapes_id, today_text, shahid],
            ),
        ),
    ] {
        assert_eq!(
            o.get(&format!("d7.sale.{name}.last_rate")),
            want,
            "d7 facts: the newest {name} line must follow the rule (date, بکری number, line number)"
        );
        assert_eq!(
            (by_id / 100).to_string(),
            wrong,
            "d7 facts: the case {name} must give another rate when sorted by the row id"
        );
        o.put(
            format!("d7.sale.{name}.last_rate_by_row_id"),
            wrong.to_owned(),
        );
    }

    // 2b. A بکری by its number (the whole bill: lines, goods, the commission of both sides and laga).
    for number in [100_i64, 466] {
        let (lines, qty, gross, comm, day): (i64, i64, i64, i64, String) = db
            .read(|c| {
                c.query_row(
                    "SELECT COUNT(*), COALESCE(SUM(s.qty),0), COALESCE(SUM(s.buyer_value_paisa),0), \
                     COALESCE(SUM(s.seller_comm_paisa + s.buyer_comm_paisa + s.laga_paisa),0), MIN(e.entry_date) \
                     FROM bikri_sales s JOIN bikri_entries e ON e.id = s.bikri_id \
                     WHERE s.is_active = 1 AND e.number = ?1",
                    params![number],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
                )
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 bikri number {number}: {e}"));
        o.put(format!("d7.bikri_no.{number}.lines"), lines.to_string());
        o.put(format!("d7.bikri_no.{number}.qty"), qty.to_string());
        o.put(format!("d7.bikri_no.{number}.gross"), rupees(gross));
        o.put(format!("d7.bikri_no.{number}.comm_total"), rupees(comm));
        o.put(format!("d7.bikri_no.{number}.date"), day);
    }

    // 2c. A voucher by its number (the receipt, the payment and the shop expense of that number).
    for (kind, number) in [("receipt", 100_i64), ("payment", 50), ("shop_expense", 20)] {
        let row: (i64, String, String) = db
            .read(|c| {
                c.query_row(
                    "SELECT v.amount_paisa, v.entry_date, COALESCE(a.name_en, '') FROM vouchers v \
                     LEFT JOIN accounts a ON a.id = v.account_id \
                     WHERE v.type = ?1 AND v.number = ?2 AND v.status = 'saved'",
                    params![kind, number],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 voucher {kind} {number}: {e}"));
        o.put(format!("d7.voucher.{kind}_{number}.amount"), rupees(row.0));
        o.put(format!("d7.voucher.{kind}_{number}.date"), row.1);
        if !row.2.is_empty() {
            o.put(format!("d7.voucher.{kind}_{number}.party"), row.2);
        }
    }

    // 2d. An item in one packing (R6): potatoes come in bags only, so potatoes in boxes are 0.
    for (item, packing, period) in [
        ("aloo", "Box", "last_month"),
        ("aloo", "Bag", "last_month"),
        ("kela", "Box", "last_month"),
    ] {
        let (from, to) = period_dates(period);
        let (from, to) = (iso(from), iso(to));
        let item_id = item_id(item);
        let (qty, gross): (i64, i64) = db
            .read(|c| {
                c.query_row(
                    "SELECT COALESCE(SUM(s.qty),0), COALESCE(SUM(s.buyer_value_paisa),0) FROM bikri_sales s \
                     JOIN bikri_lots l ON l.id = s.lot_id JOIN packings p ON p.id = l.packing_id \
                     WHERE s.is_active = 1 AND l.item_id = ?1 AND p.name_en = ?2 \
                     AND s.entry_date >= ?3 AND s.entry_date <= ?4",
                    params![item_id, packing, from, to],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 packing {item} {packing}: {e}"));
        let name = format!(
            "d7.pack.{}_{}_{period}",
            item_name(item),
            packing.to_lowercase()
        );
        o.put(format!("{name}.qty"), qty.to_string());
        o.put(format!("{name}.gross"), rupees(gross));
    }

    // The cases of `seed::d7_richer_cases`, checked the same way (the rule against the row id).
    let mosambi_id = item_id("mausami");
    let by_id_mosambi = by_row_id("", params![mosambi_id, iso(period_dates("d2026_10_19").0)]);
    assert_eq!(
        o.get("d7.sale.mosambi_oct19.last_rate"),
        "2550",
        "d7 facts: the newest mosambi line of 19-10-2026 is the line of the newer بکری"
    );
    assert_eq!(
        (by_id_mosambi / 100).to_string(),
        "2400",
        "d7 facts: the case mosambi_oct19 must give the later sale when sorted by the row id"
    );
    o.put(
        "d7.sale.mosambi_oct19.last_rate_by_row_id".into(),
        "2400".into(),
    );
    assert_eq!(
        (
            o.get("d7.sale.apple_oct22.last_rate"),
            o.get("d7.sale.apple_oct22.last_buyer_rate")
        ),
        ("4300", "4350"),
        "d7 facts: the newest apple line of 22-10-2026 is the ریٹ مختلف line: seller 4,300, buyer 4,350"
    );

    // 2e. The ریٹ مختلف line (buyer rate other than the seller rate): every such line of the books.
    let (rd_n, rd_seller, rd_buyer, rd_qty, rd_value, rd_amount, rd_diff): (
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
    ) = db
        .read(|c| {
            c.query_row(
                "SELECT COUNT(*), COALESCE(MAX(seller_rate_paisa),0), COALESCE(MAX(buyer_rate_paisa),0), \
                 COALESCE(SUM(qty),0), COALESCE(SUM(buyer_value_paisa),0), COALESCE(SUM(seller_amount_paisa),0), \
                 COALESCE(SUM(rate_diff_paisa),0) FROM bikri_sales WHERE is_active = 1 AND rate_diff_paisa <> 0",
                [],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                    ))
                },
            )
        })
        .unwrap_or_else(|e: rusqlite::Error| panic!("d7 rate difference: {e}"));
    assert_eq!(
        rd_n, 1,
        "d7 facts: the books hold exactly one ریٹ مختلف line"
    );
    o.put("d7.ratediff.lines".into(), rd_n.to_string());
    o.put(
        "d7.ratediff.seller_rate".into(),
        (rd_seller / 100).to_string(),
    );
    o.put(
        "d7.ratediff.buyer_rate".into(),
        (rd_buyer / 100).to_string(),
    );
    o.put("d7.ratediff.qty".into(), rd_qty.to_string());
    o.put("d7.ratediff.buyer_value".into(), rupees(rd_value));
    o.put("d7.ratediff.seller_amount".into(), rupees(rd_amount));
    o.put("d7.ratediff.diff".into(), rupees(rd_diff));

    // 2f. A چکتا buyer of the shared چکتا کھاتہ (Aamir): his lines. He paid in full on the spot, so he owes nothing.
    let (pk_lines, pk_qty, pk_value, pk_total): (i64, i64, i64, i64) = db
        .read(|c| {
            c.query_row(
                "SELECT COUNT(*), COALESCE(SUM(qty),0), COALESCE(SUM(buyer_value_paisa),0), \
                 COALESCE(SUM(buyer_total_paisa),0) FROM bikri_sales \
                 WHERE is_active = 1 AND chukta_name_en = 'Aamir'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
        })
        .unwrap_or_else(|e: rusqlite::Error| panic!("d7 chukta person: {e}"));
    o.put("d7.pk.aamir.lines".into(), pk_lines.to_string());
    o.put("d7.pk.aamir.qty".into(), pk_qty.to_string());
    o.put("d7.pk.aamir.amount".into(), rupees(pk_value));
    o.put("d7.pk.aamir.charges".into(), rupees(pk_total));
    o.put("d7.pk.aamir.balance".into(), rupees(0));

    // 2g. A separate khata (الگ کھاتہ): paid out, received, the balance (minus: it took more than it brought).
    let (out_n, out_sum, in_n, in_sum): (i64, i64, i64, i64) = db
        .read(|c| {
            c.query_row(
                "SELECT COALESCE(SUM(type = 'khata_out'),0), \
                 COALESCE(SUM(CASE WHEN type = 'khata_out' THEN amount_paisa ELSE 0 END),0), \
                 COALESCE(SUM(type = 'khata_in'),0), \
                 COALESCE(SUM(CASE WHEN type = 'khata_in' THEN amount_paisa ELSE 0 END),0) \
                 FROM vouchers WHERE type IN ('khata_out','khata_in') AND status = 'saved'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
        })
        .unwrap_or_else(|e: rusqlite::Error| panic!("d7 khata: {e}"));
    o.put("d7.khata.family.paid_out".into(), rupees(out_sum));
    o.put("d7.khata.family.received".into(), rupees(in_sum));
    o.put("d7.khata.family.entries".into(), (out_n + in_n).to_string());
    o.put(
        "d7.khata.family.balance".into(),
        rupees((out_sum - in_sum).abs()),
    );
    o.put(
        "d7.khata.family.balance_signed".into(),
        (-(out_sum - in_sum) / 100).to_string(),
    );

    // 2h. A city: what the buyers of Kasur owe (R21 groups the dues by the buyer's city), and how many owe.
    let (city_dues, city_buyers): (i64, i64) = db
        .read(|c| {
            c.query_row(
                "SELECT COALESCE(SUM(b.bal),0), COALESCE(SUM(b.bal > 0),0) FROM ( \
                   SELECT a.id, (SELECT COALESCE(SUM(jl.dr_paisa),0) - COALESCE(SUM(jl.cr_paisa),0) \
                      FROM journal_lines jl JOIN journal_entries je ON je.id = jl.entry_id \
                      WHERE je.status = 'posted' AND jl.account_id = a.id) AS bal \
                   FROM accounts a JOIN cities ci ON ci.id = a.city_id WHERE ci.name_en = 'Kasur') b",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
        })
        .unwrap_or_else(|e: rusqlite::Error| panic!("d7 city: {e}"));
    o.put("d7.city.kasur.dues".into(), rupees(city_dues));
    o.put("d7.city.kasur.buyers".into(), city_buyers.to_string());
    // The collection list by city (R21) must say the same.
    assert_eq!(
        amount_of(o.get("r21.kasur.balance")),
        Some(city_dues / 100),
        "d7 facts: the buyers of Kasur owe {} in the books and {} in R21",
        city_dues / 100,
        o.get("r21.kasur.balance")
    );

    // 3. Goods that came in (the lots of the bikri entries of a period), by item and for all.
    for (name, item, period) in [
        ("banana_last_month", Some("kela"), "last_month"),
        ("mango_year", Some("aam"), "year"),
        ("all_today", None, "today"),
        ("apple_month", Some("saib"), "month"),
    ] {
        let (from, to) = period_dates(period);
        let (from, to) = (iso(from), iso(to));
        let item = item.map(item_id);
        let (lots, qty): (i64, i64) = db
            .read(|c| {
                c.query_row(
                    "SELECT COUNT(*), COALESCE(SUM(l.qty_received),0) FROM bikri_lots l \
                     JOIN bikri_entries e ON e.id = l.bikri_id \
                     WHERE e.status = 'saved' AND e.entry_date >= ?1 AND e.entry_date <= ?2 \
                     AND (?3 IS NULL OR l.item_id = ?3)",
                    params![from, to, item],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 received {name}: {e}"));
        o.put(format!("d7.recv.{name}.lots"), lots.to_string());
        o.put(format!("d7.recv.{name}.qty"), qty.to_string());
    }

    // 4. Stock by group, from the stock of each item.
    let groups: Vec<(String, String)> = db
        .read(|c| {
            let mut st = c.prepare("SELECT name_en, item_group FROM items")?;
            let rows = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            rows.collect::<Result<Vec<_>, rusqlite::Error>>()
        })
        .unwrap_or_else(|e: rusqlite::Error| panic!("d7 item groups: {e}"));
    for group in ["fruit", "vegetable"] {
        let (mut qty, mut lots) = (0i64, 0i64);
        for (name, g) in &groups {
            if g == group {
                let slug = slug(name);
                qty += o
                    .facts
                    .get(&format!("stock.{slug}.qty"))
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(0);
                lots += o
                    .facts
                    .get(&format!("stock.{slug}.lots"))
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(0);
            }
        }
        o.put(format!("d7.stock.{group}.qty"), qty.to_string());
        o.put(format!("d7.stock.{group}.lots"), lots.to_string());
    }

    // 5. How many bikri were made in a period (saved entries), and by one seller last month is view B's count.
    for period in ["today", "yesterday", "week", "month", "last_month", "year"] {
        let (from, to) = period_dates(period);
        let (from, to) = (iso(from), iso(to));
        let n: i64 = db
            .read(|c| {
                c.query_row(
                    "SELECT COUNT(*) FROM bikri_entries WHERE status = 'saved' AND entry_date >= ?1 AND entry_date <= ?2",
                    params![from, to],
                    |r| r.get(0),
                )
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 bikri count {period}: {e}"));
        o.put(format!("d7.bikri.{period}.count"), n.to_string());
    }

    // 6. A partner's share in percent (the app shows 40%, the books keep 4,000 basis points), and who took the most.
    for k in ["partner_a", "partner_b", "usman"] {
        let bps: i64 = o
            .facts
            .get(&format!("partner.{k}.share_bps"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        o.put(
            format!("d7.partner.{k}.share_pct"),
            format!("{}.{:02}%", bps / 100, bps % 100),
        );
    }
    let taken = |k: &str| amount_of(o.get(&format!("r29.all.{k}.taken"))).unwrap_or(0);
    let mut best: Vec<(&str, &str, i64)> = vec![
        ("partner_a", "Partner A", taken("partner_a")),
        ("partner_b", "Partner B", taken("partner_b")),
        ("usman", "Usman", taken("usman")),
    ];
    best.sort_by_key(|entry| std::cmp::Reverse(entry.2));
    o.put("d7.top.drawings.name".into(), best[0].1.to_owned());
    o.put("d7.top.drawings.amount".into(), best[0].2.to_string());
    o.put("d7.top.drawings.least_amount".into(), best[2].2.to_string());

    // 7. A seller's newest bill (date, number, gross and net of the bill).
    for seller in ["karim", "rahim", "bashir"] {
        let id = seller_id(seller);
        let row: Option<(i64, String, i64, i64)> = db
            .read(|c| {
                c.query_row(
                    "SELECT e.number, e.entry_date, SUM(l.gross_paisa), SUM(l.net_paisa) FROM bikri_lots l \
                     JOIN bikri_entries e ON e.id = l.bikri_id WHERE l.seller_account_id = ?1 AND e.status = 'saved' \
                     GROUP BY e.id ORDER BY e.entry_date DESC, e.id DESC LIMIT 1",
                    params![id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .map(Some)
                .or_else(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Ok(None),
                    other => Err(other),
                })
            })
            .unwrap_or_else(|e: rusqlite::Error| panic!("d7 last bill {seller}: {e}"));
        if let Some((number, day, gross, net)) = row {
            o.put(format!("d7.last_bill.{seller}.number"), number.to_string());
            o.put(format!("d7.last_bill.{seller}.date"), day);
            o.put(format!("d7.last_bill.{seller}.gross"), rupees(gross));
            o.put(format!("d7.last_bill.{seller}.net"), rupees(net));
        }
    }

    // 8. Two items against each other in a period: who sold more (by gross), and by how much.
    for (a, b, period) in [
        ("banana", "apple", "month"),
        ("mango", "grapes", "year"),
        ("potato", "banana", "last_month"),
    ] {
        let ga = amount_of(o.get(&format!("ix.{period}.{a}.gross"))).unwrap_or(0);
        let gb = amount_of(o.get(&format!("ix.{period}.{b}.gross"))).unwrap_or(0);
        let cap = |s: &str| {
            let mut chars = s.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        };
        let (more, less) = if ga >= gb { (a, b) } else { (b, a) };
        o.put(format!("d7.cmp.{a}_{b}.{period}.more"), cap(more));
        o.put(format!("d7.cmp.{a}_{b}.{period}.less"), cap(less));
        o.put(
            format!("d7.cmp.{a}_{b}.{period}.diff"),
            (ga - gb).abs().to_string(),
        );
    }

    // The SQL above must agree with the app's own analytics where they overlap (view A with a seller, the item
    // axis): a mistake here stops the build instead of reaching the bank.
    for (what, want, got) in [
        (
            "Rahim's banana gross",
            "d7.sale.rahim_banana_year.gross",
            "seller_items.rahim.year.banana.gross",
        ),
        (
            "Rahim's banana qty",
            "d7.sale.rahim_banana_year.qty",
            "seller_items.rahim.year.banana.qty",
        ),
        (
            "Rahim's banana seller commission",
            "d7.sale.rahim_banana_year.comm_seller",
            "seller_items.rahim.year.banana.seller_comm",
        ),
    ] {
        assert_eq!(
            amount_of(o.get(want)),
            amount_of(o.get(got)),
            "d7 fact {what}: the books say {}, the app's view says {}",
            o.get(want),
            o.get(got)
        );
    }
    assert_eq!(
        amount_of(o.get("d7.sale.apple_week.last_rate")),
        amount_of(o.get("ix.week.apple.last_rate")),
        "d7 facts: the newest apple rate of this week must be the one of the item axis"
    );
    let sold_today = amount_of(o.get("d7.sale.cash_today.gross")).unwrap_or(0)
        + amount_of(o.get("d7.sale.credit_today.gross")).unwrap_or(0);
    assert_eq!(
        sold_today,
        amount_of(o.get("ix.today.all.gross")).unwrap_or(-1),
        "d7 facts: cash and credit sales of today must add up to the sales of today in view A"
    );

    // 9. Two expense heads together, from the expense report (R29) of all time.
    let rent = amount_of(o.get("r29.all.shop_rent.expense")).unwrap_or(0);
    let electricity = amount_of(o.get("r29.all.electricity.expense")).unwrap_or(0);
    o.put(
        "d7.exp.rent_plus_electricity".into(),
        (rent + electricity).to_string(),
    );
}
