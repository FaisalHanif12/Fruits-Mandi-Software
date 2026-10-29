//! Fruit taken in the reports and the analytics (plan d5-late, after D4R 17d; PRD 16.7, FR-16.10 to
//! FR-16.12, FR-17.10, FR-18.2, 18.2 view K, 19.2 R10, R31, R34, R37; E2E-42, E2E-40).
//!
//! The data is made with the real services (`reports_world`, the fruit service). The expected
//! numbers are the PRD's and the money agent's hand-worked cases; the views are also held against
//! each other: the fruit column of view F over all partners is minus the value of view K, the
//! register R37 and view K add up to the same totals, and the lines behind a number add up to it.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::analytics::{
    AnalyticsFilters, AnalyticsGoods, AnalyticsKey, AnalyticsLinesInput, AnalyticsPage,
    AnalyticsPartnerMoney, AnalyticsRunInput, AnalyticsSide, AnalyticsView, lines, run,
};
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_reports::reports::{Cell, ReportFilters, ReportId, ReportPage, RowKind};
use mandi_services::core::accounts::AccountType;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::fruit::{
    FruitCancelInput, FruitEditInput, FruitInput, FruitSaved, FruitSource,
};
use mandi_services::fruit;
use reports_world::*;
use support::World;

// ---- The worlds -------------------------------------------------------------------------------

/// The world after E2E-16: رحیم's lot of 50 کیلا with 45 sold and 5 left.
fn after_e2e_16() -> World {
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    save(&w, &draft);
    w
}

/// حاجی نذیر, another agent, seller 0012.
fn nazeer(w: &World) -> i64 {
    support::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "حاجی نذیر",
        "Haji Nazeer",
        true,
        "",
    )
}

fn home(w: &World, lot: i64, partner: usize, date: &str, qty: &str, rate: &str) -> FruitInput {
    FruitInput {
        for_guests: false,
        partner_id: Some(w.partners[partner]),
        date: date.into(),
        source: FruitSource::Shop,
        lot_id: Some(lot),
        agent_account_id: None,
        item_id: None,
        packing_id: None,
        qty: qty.into(),
        weight: None,
        rate: rate.into(),
        note: None,
        editing_id: None,
    }
}

fn guests(w: &World, agent: i64, date: &str, qty: &str, rate: &str) -> FruitInput {
    FruitInput {
        for_guests: true,
        partner_id: None,
        date: date.into(),
        source: FruitSource::OtherShop,
        lot_id: None,
        agent_account_id: Some(agent),
        item_id: Some(w.apple),
        packing_id: Some(w.box_),
        qty: qty.into(),
        weight: None,
        rate: rate.into(),
        note: None,
        editing_id: None,
    }
}

fn take(w: &World, input: &FruitInput) -> FruitSaved {
    fruit::save(&w.env.ctx, &w.owner, input).unwrap()
}

/// E2E-42: الف takes 2 کیلا at 580 home (FRT-1, our shop), the guests 3 سیب at 1,000 from حاجی
/// نذیر (FRT-2, another shop). Returns the world and the two entries.
fn e2e_42() -> (World, FruitSaved, FruitSaved) {
    let w = after_e2e_16();
    let one = take(&w, &home(&w, 1, 0, "10-10-2026", "2", "580"));
    let agent = nazeer(&w);
    let two = take(&w, &guests(&w, agent, "10-10-2026", "3", "1,000"));
    (w, one, two)
}

// ---- Readers ----------------------------------------------------------------------------------

fn n(cell: &Cell) -> i64 {
    match cell {
        Cell::Number { value } => *value,
        Cell::Money { paisa } => paisa.rupees(),
        Cell::Percent { hundredths } => *hundredths,
        Cell::Empty => 0,
        other => panic!("not a number: {other:?}"),
    }
}

fn names_en(cell: &Cell) -> String {
    match cell {
        Cell::Names { names } => names.en.clone(),
        Cell::Label { id } => id.clone(),
        Cell::Party { party } => party.name_en.clone(),
        Cell::Empty => String::new(),
        other => panic!("not names: {other:?}"),
    }
}

fn day(from: &str, to: &str) -> ReportFilters {
    ReportFilters {
        from: from.into(),
        to: to.into(),
        ..ReportFilters::default()
    }
}

fn r37(w: &World, filters: ReportFilters) -> ReportPage {
    report(w, ReportId::R37, filters, 0, 0)
}

/// (for, item, packing, weight, qty, rate, value, source) of each row of R37.
type Entry = (String, String, String, String, i64, i64, i64, String);

fn entries(page: &ReportPage) -> Vec<Entry> {
    page.rows
        .iter()
        .map(|r| {
            let c = &r.cells;
            let weight = match &c[6] {
                Cell::Text { text } => text.clone(),
                Cell::Empty => String::new(),
                other => panic!("not text: {other:?}"),
            };
            (
                names_en(&c[2]),
                names_en(&c[4]),
                names_en(&c[5]),
                weight,
                n(&c[7]),
                n(&c[8]),
                n(&c[9]),
                names_en(&c[10]),
            )
        })
        .collect()
}

fn entry(who: &str, item: &str, qty: i64, rate: i64, value: i64, source: &str) -> Entry {
    (
        who.into(),
        item.into(),
        "Box".into(),
        String::new(),
        qty,
        rate,
        value,
        source.into(),
    )
}

fn av(w: &World, v: AnalyticsView, f: AnalyticsFilters) -> AnalyticsPage {
    run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            sort: None,
            view: v,
            filters: f,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
}

fn refused(w: &World, v: AnalyticsView, f: AnalyticsFilters) -> Vec<(ErrorCode, String)> {
    match run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            sort: None,
            view: v,
            filters: f,
            offset: 0,
            limit: 0,
        },
    ) {
        Err(mandi_reports::ReportsError::Rejected(items)) => {
            items.into_iter().map(|i| (i.code, i.field)).collect()
        }
        other => panic!("expected a rejection, got {other:?}"),
    }
}

fn on(from: &str, to: &str) -> AnalyticsFilters {
    AnalyticsFilters {
        from: from.into(),
        to: to.into(),
        ..AnalyticsFilters::default()
    }
}

fn column(page: &AnalyticsPage, key: &str) -> usize {
    page.columns
        .iter()
        .position(|c| c.key == key)
        .unwrap_or_else(|| panic!("no column {key}"))
}

fn data_rows(page: &AnalyticsPage) -> Vec<&mandi_reports::analytics::AnalyticsRow> {
    page.rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .collect()
}

/// The rows of view K as (for, item, qty, value, our shop, other shops).
fn k_rows(page: &AnalyticsPage) -> Vec<(String, String, i64, i64, i64, i64)> {
    data_rows(page)
        .iter()
        .map(|r| {
            let c = &r.cells;
            (
                names_en(&c[0]),
                names_en(&c[1]),
                n(&c[2]),
                n(&c[3]),
                n(&c[4]),
                n(&c[5]),
            )
        })
        .collect()
}

fn k_totals(page: &AnalyticsPage) -> (i64, i64, i64, i64) {
    (
        n(&page.totals[2]),
        n(&page.totals[3]),
        n(&page.totals[4]),
        n(&page.totals[5]),
    )
}

/// The fruit column of view F for each partner (creation order) and its total.
fn f_fruit(w: &World, f: AnalyticsFilters) -> (Vec<i64>, i64) {
    let page = av(w, AnalyticsView::F, f);
    let fruit = column(&page, "fruit");
    (
        data_rows(&page)
            .iter()
            .map(|r| n(&r.cells[fruit]))
            .collect(),
        n(&page.totals[fruit]),
    )
}

fn lines_of(
    w: &World,
    view: AnalyticsView,
    filters: AnalyticsFilters,
    row_key: AnalyticsKey,
    column_key: Option<AnalyticsKey>,
) -> mandi_reports::analytics::AnalyticsLinesPage {
    lines(
        w.env.ctx.db(),
        &AnalyticsLinesInput {
            view,
            filters,
            row_key,
            column_key,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
}

/// The (count, amount) of the fruit row of R34 on a day.
fn r34_fruit(w: &World, from: &str, to: &str) -> Vec<(String, i64, i64)> {
    let page = report(w, ReportId::R34, day(from, to), 0, 0);
    page.rows
        .iter()
        .filter(|r| matches!(&r.cells[1], Cell::Label { id } if id == "report-r34-fruit-taken"))
        .map(|r| {
            let date = match &r.cells[0] {
                Cell::Date { iso } => iso.clone(),
                other => panic!("not a date: {other:?}"),
            };
            (date, n(&r.cells[2]), n(&r.cells[3]))
        })
        .collect()
}

/// The lot rows of R10 as (received, sold, written off, taken, balance), and the totals row.
fn r10(w: &World) -> (Vec<[i64; 5]>, [i64; 5]) {
    let page = report(w, ReportId::R10, ReportFilters::default(), 0, 0);
    let pick = |cells: &[Cell]| -> [i64; 5] {
        [
            n(&cells[5]),
            n(&cells[6]),
            n(&cells[7]),
            n(&cells[8]),
            n(&cells[9]),
        ]
    };
    (
        page.rows.iter().map(|r| pick(&r.cells)).collect(),
        pick(&page.totals),
    )
}

// ---- E2E-42 -----------------------------------------------------------------------------------

#[test]
fn e2e_42_the_register_the_day_summary_and_the_stock_show_the_fruit() {
    let (w, _, _) = e2e_42();
    // R37: the two entries of the day, home first, then the guests; totals 5 نگ and 4,160.
    let page = r37(&w, day("10-10-2026", "10-10-2026"));
    assert_eq!(
        entries(&page),
        vec![
            entry("Partner A", "Banana", 2, 580, 1_160, "fruit-source-own"),
            entry(
                "fruit-for-guests",
                "Apple",
                3,
                1_000,
                3_000,
                "fruit-source-other"
            ),
        ]
    );
    assert_eq!(page.total_count, 2);
    assert_eq!((n(&page.totals[7]), n(&page.totals[9])), (5, 4_160));
    // The seller credited: رحیم for the lot, حاجی نذیر for the other shop.
    assert_eq!(names_en(&page.rows[0].cells[3]), "Rahim");
    assert_eq!(names_en(&page.rows[1].cells[3]), "Haji Nazeer");
    // The number series of the entries (FRT-1, FRT-2).
    assert!(matches!(
        &page.rows[0].cells[1],
        Cell::Doc { type_id, number: Some(1) } if type_id == "report-doc-fruit-taken"
    ));
    // Each row opens its fruit entry.
    assert!(page.rows.iter().all(|r| matches!(
        r.source,
        Some(mandi_reports::reports::RowSource {
            kind: mandi_reports::reports::SourceKind::Fruit,
            ..
        })
    )));

    // The filters: a partner, only the guests, an item.
    let mut by_partner = day("10-10-2026", "10-10-2026");
    by_partner.partner_id = Some(w.partners[0]);
    assert_eq!(entries(&r37(&w, by_partner)).len(), 1);
    let mut only_guests = day("10-10-2026", "10-10-2026");
    only_guests.guests_only = Some(true);
    let g = r37(&w, only_guests.clone());
    assert_eq!(
        entries(&g),
        vec![entry(
            "fruit-for-guests",
            "Apple",
            3,
            1_000,
            3_000,
            "fruit-source-other"
        )]
    );
    assert!(g.filters.guests_only == Some(true));
    let mut by_item = day("10-10-2026", "10-10-2026");
    by_item.item_id = Some(w.banana);
    assert_eq!(entries(&r37(&w, by_item)).len(), 1);
    // The guests are no partner: both together are refused at the guests field.
    only_guests.partner_id = Some(w.partners[0]);
    assert_eq!(
        rejected(&w, ReportId::R37, only_guests, 0, 0),
        vec![(ErrorCode::ErrOutOfRange, "filters.guests_only".to_owned())]
    );
    // A day with no fruit has no rows.
    assert!(entries(&r37(&w, day("09-10-2026", "09-10-2026"))).is_empty());

    // R34: one fruit row for the day, 5 نگ and 4,160.
    assert_eq!(
        r34_fruit(&w, "10-10-2026", "10-10-2026"),
        [("2026-10-10".to_owned(), 5, 4_160)]
    );

    // R10: the lot received 50, sold 45, taken 2: the balance is 3 (C16), not 5.
    let (lots, totals) = r10(&w);
    assert_eq!(lots, vec![[50, 45, 0, 2, 3]]);
    assert_eq!(totals, [50, 45, 0, 2, 3]);
}

#[test]
fn e2e_42_view_k_and_the_fruit_column_of_view_f_agree() {
    let (w, _, _) = e2e_42();
    let k = av(&w, AnalyticsView::K, on("10-10-2026", "10-10-2026"));
    assert_eq!(k.title_id, "analytics-view-k-title");
    assert_eq!(
        k_rows(&k),
        vec![
            ("Partner A".into(), "Banana".into(), 2, 1_160, 1_160, 0),
            (
                "fruit-for-guests".into(),
                "Apple".into(),
                3,
                3_000,
                0,
                3_000
            ),
        ]
    );
    assert_eq!(k_totals(&k), (5, 4_160, 1_160, 3_000));
    // The register adds up to the same totals.
    let register = r37(&w, day("10-10-2026", "10-10-2026"));
    assert_eq!(
        (n(&register.totals[7]), n(&register.totals[9])),
        (k_totals(&k).0, k_totals(&k).1)
    );

    // View F: الف −1,160 home −1,000 his part of the guests' = −2,160; ب and ج −1,000.
    let (fruit_col, total) = f_fruit(&w, on("10-10-2026", "10-10-2026"));
    assert_eq!(fruit_col, [-2_160, -1_000, -1_000]);
    // Over all partners the column is minus the value of view K.
    assert_eq!(total, -k_totals(&k).1);
    // The net change of E2E-42: profit share 1,424 / 1,068 / 1,068 plus the fruit.
    let f = av(&w, AnalyticsView::F, on("10-10-2026", "10-10-2026"));
    let net = column(&f, "net_change");
    let other = column(&f, "other");
    assert_eq!(
        data_rows(&f)
            .iter()
            .map(|r| n(&r.cells[net]))
            .collect::<Vec<_>>(),
        [-736, 68, 68]
    );
    // Nothing else is on their account: "other" stays 0.
    assert!(data_rows(&f).iter().all(|r| n(&r.cells[other]) == 0));
}

#[test]
fn e2e_42_the_lines_behind_a_number_add_up_to_it() {
    let (w, _, _) = e2e_42();
    let f = on("10-10-2026", "10-10-2026");
    // The K row of الف opens his one entry.
    let k = av(&w, AnalyticsView::K, f.clone());
    let alif_row = data_rows(&k)[0].key.clone().unwrap();
    let one = lines_of(&w, AnalyticsView::K, f.clone(), alif_row, None);
    assert_eq!((one.total_count, n(&one.totals[9])), (1, 1_160));
    // The guests row opens the guests' entry.
    let guests_row = data_rows(&k)[1].key.clone().unwrap();
    assert_eq!(guests_row.guests, Some(true));
    let two = lines_of(&w, AnalyticsView::K, f.clone(), guests_row, None);
    assert_eq!(
        (two.total_count, n(&two.totals[7]), n(&two.totals[9])),
        (1, 3, 3_000)
    );
    // The whole filter: the two entries.
    let all = lines_of(
        &w,
        AnalyticsView::K,
        f.clone(),
        AnalyticsKey::default(),
        None,
    );
    assert_eq!((all.total_count, n(&all.totals[9])), (2, 4_160));

    // The fruit cell of view F opens his lines on his account: −1,160 and −1,000, in all −2,160.
    let fruit_key = AnalyticsKey {
        money: Some(AnalyticsPartnerMoney::Fruit),
        ..AnalyticsKey::default()
    };
    let alif = AnalyticsKey {
        partner_id: Some(w.partners[0]),
        ..AnalyticsKey::default()
    };
    let cell = lines_of(
        &w,
        AnalyticsView::F,
        f.clone(),
        alif,
        Some(fruit_key.clone()),
    );
    assert_eq!(cell.total_count, 2);
    assert_eq!(
        cell.rows.iter().map(|r| n(&r.cells[3])).collect::<Vec<_>>(),
        [-1_160, -1_000]
    );
    assert_eq!(n(&cell.totals[3]), -2_160);
    // The totals row of the column: every partner's lines, minus 4,160 (4 lines).
    let every = lines_of(
        &w,
        AnalyticsView::F,
        f.clone(),
        AnalyticsKey::default(),
        Some(fruit_key),
    );
    assert_eq!((every.total_count, n(&every.totals[3])), (4, -4_160));
    // The fruit column carries its key, so the screen can send it back.
    let page = av(&w, AnalyticsView::F, f);
    let col = &page.columns[column(&page, "fruit")];
    assert_eq!(
        col.drill.as_ref().and_then(|d| d.money),
        Some(AnalyticsPartnerMoney::Fruit)
    );
}

#[test]
fn e2e_42_the_partner_statement_report_writes_the_fruit_lines() {
    let (w, _, _) = e2e_42();
    let filters = ReportFilters {
        partner_id: Some(w.partners[0]),
        ..ReportFilters::default()
    };
    // R31 is read through the services statement (the app does that); here the page builder is
    // fed the same statement.
    let statement = mandi_services::partners::statement(
        &w.env.ctx,
        &mandi_services::core::statement::PartnersStatementInput {
            group_by_month: false,
            sort: None,
            partner_id: w.partners[0],
            from: None,
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let request = mandi_reports::reports::statement_request(w.env.ctx.db(), &filters).unwrap();
    let page = mandi_reports::reports::statement_page(&request, &statement, 0, 0).unwrap();
    let fruit_rows: Vec<_> = page
        .rows
        .iter()
        .filter(|r| matches!(&r.cells[1], Cell::Label { id } if id == "partners-statement-heading-fruit"))
        .collect();
    // His home fruit and his part of the guests' fruit.
    assert_eq!(fruit_rows.len(), 2);
    let amounts: Vec<i64> = fruit_rows.iter().map(|r| n(&r.cells[3])).collect();
    assert_eq!(amounts, [-1_160, -1_000]);
    for r in &fruit_rows {
        match &r.cells[2] {
            Cell::Message { id, args } => {
                assert_eq!(id, "partners-statement-detail-fruit-taken");
                let names: Vec<&str> = args.iter().map(|a| a.name.as_str()).collect();
                for needed in ["for", "subject", "packing", "qty", "rate", "value"] {
                    assert!(names.contains(&needed), "{needed} in {names:?}");
                }
            }
            other => panic!("not a message: {other:?}"),
        }
    }
}

// ---- Edit and cancel ---------------------------------------------------------------------------

#[test]
fn an_edit_that_moves_a_take_to_another_day_moves_it_in_every_report() {
    let (w, one, _) = e2e_42();
    set_today(&w.env, 2026, 10, 11);
    fruit::edit(
        &w.env.ctx,
        &w.owner,
        &FruitEditInput {
            id: one.id,
            version: one.version,
            entry: home(&w, 1, 0, "11-10-2026", "1", "600"),
            reason: "غلط اندراج".into(),
        },
    )
    .unwrap();
    // R37: 10-10 only the guests; 11-10 الف 1 at 600; both days 4 نگ, 3,600.
    let d10 = r37(&w, day("10-10-2026", "10-10-2026"));
    assert_eq!(entries(&d10).len(), 1);
    assert_eq!((n(&d10.totals[7]), n(&d10.totals[9])), (3, 3_000));
    let d11 = r37(&w, day("11-10-2026", "11-10-2026"));
    assert_eq!(
        entries(&d11),
        vec![entry(
            "Partner A",
            "Banana",
            1,
            600,
            600,
            "fruit-source-own"
        )]
    );
    let both = r37(&w, day("10-10-2026", "11-10-2026"));
    assert_eq!((n(&both.totals[7]), n(&both.totals[9])), (4, 3_600));
    // R34 on each day.
    assert_eq!(
        r34_fruit(&w, "10-10-2026", "11-10-2026"),
        [
            ("2026-10-10".to_owned(), 3, 3_000),
            ("2026-10-11".to_owned(), 1, 600)
        ]
    );
    // View F: each day equals minus view K of the same day.
    for (from, to, expected) in [
        ("10-10-2026", "10-10-2026", vec![-1_000, -1_000, -1_000]),
        ("11-10-2026", "11-10-2026", vec![-600, 0, 0]),
        ("10-10-2026", "11-10-2026", vec![-1_600, -1_000, -1_000]),
    ] {
        let (col, total) = f_fruit(&w, on(from, to));
        assert_eq!(col, expected, "{from}");
        let k = av(&w, AnalyticsView::K, on(from, to));
        assert_eq!(total, -k_totals(&k).1, "{from}");
    }
    // R10: 1 taken, balance 4. The old posting and its reversal are not in any report.
    let (_, totals) = r10(&w);
    assert_eq!(totals, [50, 45, 0, 1, 4]);
    // The lines behind the fruit column of every partner over both days: the guests' three parts
    // of 10-10 and الف's −600 of 11-10. The old −1,160 and its reversal are not there.
    let every = lines_of(
        &w,
        AnalyticsView::F,
        on("10-10-2026", "11-10-2026"),
        AnalyticsKey::default(),
        Some(AnalyticsKey {
            money: Some(AnalyticsPartnerMoney::Fruit),
            ..AnalyticsKey::default()
        }),
    );
    assert_eq!(every.total_count, 4);
    let mut amounts: Vec<i64> = every.rows.iter().map(|r| n(&r.cells[3])).collect();
    amounts.sort_unstable();
    assert_eq!(amounts, [-1_000, -1_000, -1_000, -600]);
    assert_eq!(n(&every.totals[3]), -3_600);
}

#[test]
fn a_cancelled_take_is_in_no_report() {
    let (w, _, two) = e2e_42();
    fruit::cancel(
        &w.env.ctx,
        &w.owner,
        &FruitCancelInput {
            id: two.id,
            version: two.version,
            reason: "غلط اندراج".into(),
        },
    )
    .unwrap();
    let page = r37(&w, ReportFilters::default());
    assert_eq!(entries(&page).len(), 1);
    assert_eq!((n(&page.totals[7]), n(&page.totals[9])), (2, 1_160));
    assert_eq!(
        r34_fruit(&w, "10-10-2026", "10-10-2026"),
        [("2026-10-10".to_owned(), 2, 1_160)]
    );
    let k = av(&w, AnalyticsView::K, on("10-10-2026", "10-10-2026"));
    assert_eq!(k_totals(&k), (2, 1_160, 1_160, 0));
    let (col, total) = f_fruit(&w, on("10-10-2026", "10-10-2026"));
    assert_eq!((col, total), (vec![-1_160, 0, 0], -1_160));
    // The cancelled guest take left the lot alone: taken 2, balance 3.
    assert_eq!(r10(&w).1, [50, 45, 0, 2, 3]);
}

#[test]
fn the_other_readers_of_a_lot_balance_follow_a_take_in_and_out() {
    // The dashboard tile, the lots left on R34 and the day close read the stock moves: a take on
    // the lot writes a `taken` move, and cancelling it writes the reversal on the same day.
    let (w, one, _) = e2e_42();
    let tile = |w: &World| -> (i64, i64) {
        let tiles = dashboard::get(
            w.env.ctx.db(),
            BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        )
        .unwrap()
        .tiles;
        (tiles.stock_lots, tiles.stock_qty)
    };
    let r34_open_qty = |w: &World| -> i64 {
        let page = report(w, ReportId::R34, day("10-10-2026", "10-10-2026"), 0, 0);
        let row = page
            .rows
            .iter()
            .find(|r| matches!(&r.cells[1], Cell::Label { id } if id == "report-r34-open-qty"))
            .unwrap();
        n(&row.cells[2])
    };
    let close_open_qty = |w: &World| -> i64 {
        mandi_services::day::close_preview(
            &w.env.ctx,
            &mandi_services::core::vouchers::DayClosePreviewInput {
                up_to: "10-10-2026".into(),
            },
        )
        .unwrap()
        .summary
        .open_qty
    };
    // 5 left, 2 taken: 3 (C16).
    assert_eq!(tile(&w), (1, 3));
    assert_eq!(r34_open_qty(&w), 3);
    assert_eq!(close_open_qty(&w), 3);
    assert_eq!(r10(&w).1, [50, 45, 0, 2, 3]);
    // Cancelling the home take gives the 2 back everywhere.
    fruit::cancel(
        &w.env.ctx,
        &w.owner,
        &FruitCancelInput {
            id: one.id,
            version: one.version,
            reason: "غلط اندراج".into(),
        },
    )
    .unwrap();
    assert_eq!(tile(&w), (1, 5));
    assert_eq!(r34_open_qty(&w), 5);
    assert_eq!(close_open_qty(&w), 5);
    assert_eq!(r10(&w).1, [50, 45, 0, 0, 5]);
}

#[test]
fn fruit_from_another_shop_is_only_in_view_k_under_all_goods() {
    // E2E-42: the home take is from our lot (consigned), the guests' from another agent's shop.
    let (w, _, _) = e2e_42();
    let goods = |g| AnalyticsFilters {
        goods: g,
        ..on("10-10-2026", "10-10-2026")
    };
    assert_eq!(
        k_totals(&av(&w, AnalyticsView::K, goods(AnalyticsGoods::All))),
        (5, 4_160, 1_160, 3_000)
    );
    let consigned = av(&w, AnalyticsView::K, goods(AnalyticsGoods::Consigned));
    assert_eq!(
        k_rows(&consigned),
        vec![("Partner A".into(), "Banana".into(), 2, 1_160, 1_160, 0)]
    );
    let garden = av(&w, AnalyticsView::K, goods(AnalyticsGoods::Garden));
    assert!(k_rows(&garden).is_empty());
    assert_eq!(k_totals(&garden), (0, 0, 0, 0));
}

// ---- Parts that do not divide, the whole leftover, weight --------------------------------------

#[test]
fn a_guest_take_that_does_not_divide_equally_gives_the_extra_rupee_to_the_first_partner() {
    // 1 سیب at 1,000 for the guests: 334 / 333 / 333 (C27).
    let w = after_e2e_16();
    let agent = nazeer(&w);
    take(&w, &guests(&w, agent, "10-10-2026", "1", "1,000"));
    let (col, total) = f_fruit(&w, on("10-10-2026", "10-10-2026"));
    assert_eq!(col, [-334, -333, -333]);
    assert_eq!(total, -1_000);
    let k = av(&w, AnalyticsView::K, on("10-10-2026", "10-10-2026"));
    assert_eq!(
        k_rows(&k),
        vec![(
            "fruit-for-guests".into(),
            "Apple".into(),
            1,
            1_000,
            0,
            1_000
        )]
    );
}

#[test]
fn taking_all_the_leftover_empties_the_lot_out_of_r10_but_not_out_of_the_register() {
    let w = after_e2e_16();
    take(&w, &home(&w, 1, 0, "10-10-2026", "5", "580"));
    let (lots, totals) = r10(&w);
    assert!(lots.is_empty(), "{lots:?}");
    assert_eq!(totals, [0, 0, 0, 0, 0]);
    let page = r37(&w, ReportFilters::default());
    assert_eq!((n(&page.totals[7]), n(&page.totals[9])), (5, 2_900));
}

#[test]
fn a_weight_item_shows_its_weight_and_its_stored_value_not_qty_times_rate() {
    // 1 نگ of grapes, 12.345 kg at 100 a kilo from another agent's shop: 1,234.5 rounds to 1,235.
    let w = after_e2e_16();
    let agent = nazeer(&w);
    let mut input = guests(&w, agent, "10-10-2026", "1", "100");
    input.item_id = Some(w.weight_fruit);
    input.weight = Some("12.345".into());
    take(&w, &input);
    let page = r37(&w, ReportFilters::default());
    let rows = entries(&page);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        (rows[0].3.as_str(), rows[0].4, rows[0].5, rows[0].6),
        ("12.345", 1, 100, 1_235)
    );
    assert_eq!(n(&page.totals[9]), 1_235);
    let k = av(&w, AnalyticsView::K, AnalyticsFilters::default());
    assert_eq!(k_totals(&k), (1, 1_235, 0, 1_235));
}

// ---- Filters ----------------------------------------------------------------------------------

#[test]
fn the_catalogues_list_r37_and_view_k_with_their_filters() {
    let entry = mandi_reports::reports::catalog()
        .into_iter()
        .find(|e| e.id == ReportId::R37)
        .unwrap();
    assert_eq!(entry.title_id, "report-r37-title");
    let kinds: Vec<String> = entry.filters.iter().map(|f| format!("{f:?}")).collect();
    assert_eq!(kinds, ["DateRange", "Partner", "Guests", "Item", "Garden"]);
    let k = mandi_reports::analytics::catalog()
        .into_iter()
        .find(|e| e.id == AnalyticsView::K)
        .unwrap();
    let kinds: Vec<String> = k.filters.iter().map(|f| format!("{f:?}")).collect();
    assert_eq!(kinds, ["Period", "Item", "Partner", "Garden", "Goods"]);
}

#[test]
fn a_report_that_does_not_take_a_filter_refuses_it_at_its_field() {
    let (w, _, _) = e2e_42();
    // The guests belong to R37 only.
    assert_eq!(
        rejected(
            &w,
            ReportId::R20,
            ReportFilters {
                guests_only: Some(true),
                ..ReportFilters::default()
            },
            0,
            0
        ),
        vec![(ErrorCode::ErrOutOfRange, "filters.guests_only".to_owned())]
    );
    // R37 takes no seller.
    assert_eq!(
        rejected(
            &w,
            ReportId::R37,
            ReportFilters {
                seller_id: Some(w.rahim),
                ..ReportFilters::default()
            },
            0,
            0
        ),
        vec![(ErrorCode::ErrOutOfRange, "filters.seller_id".to_owned())]
    );
    // View K takes no seller, buyer, side, trip goods, comparison or grouping.
    for (filters, field) in [
        (
            AnalyticsFilters {
                seller_id: Some(w.rahim),
                ..AnalyticsFilters::default()
            },
            "filters.seller_id",
        ),
        (
            AnalyticsFilters {
                side: AnalyticsSide::Seller,
                ..AnalyticsFilters::default()
            },
            "filters.side",
        ),
        (
            AnalyticsFilters {
                goods: AnalyticsGoods::Trip,
                ..AnalyticsFilters::default()
            },
            "filters.goods",
        ),
        (
            AnalyticsFilters {
                group_by: Some(mandi_reports::analytics::AnalyticsGroupBy::Day),
                ..AnalyticsFilters::default()
            },
            "filters.group_by",
        ),
    ] {
        assert_eq!(
            refused(&w, AnalyticsView::K, filters),
            vec![(ErrorCode::ErrOutOfRange, field.to_owned())],
            "{field}"
        );
    }
    // The partner belongs to K as to F, G, I and J.
    let mut partner = on("10-10-2026", "10-10-2026");
    partner.partner_id = Some(w.partners[0]);
    let k = av(&w, AnalyticsView::K, partner);
    assert_eq!(
        k_rows(&k),
        vec![("Partner A".into(), "Banana".into(), 2, 1_160, 1_160, 0)]
    );
}

#[test]
fn a_key_field_of_another_view_is_refused_in_the_lines() {
    let (w, _, _) = e2e_42();
    let try_lines = |view, row_key: AnalyticsKey, column_key: Option<AnalyticsKey>| {
        lines(
            w.env.ctx.db(),
            &AnalyticsLinesInput {
                view,
                filters: on("10-10-2026", "10-10-2026"),
                row_key,
                column_key,
                offset: 0,
                limit: 0,
            },
        )
    };
    let fields = |result: Result<_, mandi_reports::ReportsError>| match result {
        Err(mandi_reports::ReportsError::Rejected(items)) => items
            .into_iter()
            .map(|i| (i.code, i.field))
            .collect::<Vec<_>>(),
        Ok(_) => panic!("expected a refusal"),
        Err(other) => panic!("{other:?}"),
    };
    // `money` belongs to view F, `guests` to view K.
    let money = AnalyticsKey {
        money: Some(AnalyticsPartnerMoney::Fruit),
        ..AnalyticsKey::default()
    };
    let guests_row = AnalyticsKey {
        guests: Some(true),
        ..AnalyticsKey::default()
    };
    assert_eq!(
        fields(try_lines(
            AnalyticsView::K,
            AnalyticsKey::default(),
            Some(money)
        )),
        vec![(ErrorCode::ErrOutOfRange, "column_key.money".to_owned())]
    );
    assert_eq!(
        fields(try_lines(AnalyticsView::A, guests_row, None)),
        vec![(ErrorCode::ErrOutOfRange, "row_key.guests".to_owned())]
    );
}

// ---- T12: fruit taken from a garden's lot (E2E-40 with fruit) ----------------------------------

/// E2E-40 up to the day before the closing, with the garden's lot of 102 پیٹی (100 sold at 600), so
/// 2 are left; on 12-10 الف takes 2 at 600 = 1,200 from it, and on the same day another take of 1
/// at 580 from رحیم's own lot of 10 (8 sold). Returns the world and the garden.
fn garden_with_fruit() -> (World, mandi_services::core::gardens::GardenRow) {
    let w = worked_day();
    let garden = garden_made(&w);
    set_today(&w.env, 2026, 10, 11);
    garden_pay(&w, garden.id, false, "11-10-2026", "20,000", "installment");
    garden_pay(&w, garden.id, true, "11-10-2026", "5,000", "labour");
    set_today(&w.env, 2026, 10, 12);
    let mut lot = support::with_sales(
        support::bare_lot(&w, "garden-1", garden.account.id, w.banana, "102"),
        vec![support::sale("s-1", w.arshad, "100", "600")],
    );
    lot.adjust_advance = false;
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = "12-10-2026".into();
    save(&w, &draft);
    garden_other_sale(&w, garden.id, "12-10-2026", "20", "10,000");
    // A consigned lot of رحیم with fruit taken too, to tell the two kinds of goods apart.
    let own = support::with_sales(
        support::bare_lot(&w, "lot-2", w.rahim, w.banana, "10"),
        vec![support::sale("s-2", w.akram, "8", "600")],
    );
    let mut draft = support::draft(vec![own]);
    draft.header.entry_date = "12-10-2026".into();
    save(&w, &draft);
    let lot_of = |seller: i64| -> i64 {
        w.env
            .ctx
            .db()
            .read(|conn| {
                conn.query_row(
                    "SELECT id FROM bikri_lots WHERE seller_account_id = ?1",
                    [seller],
                    |r| r.get(0),
                )
            })
            .unwrap()
    };
    take(
        &w,
        &home(&w, lot_of(garden.account.id), 0, "12-10-2026", "2", "600"),
    );
    take(&w, &home(&w, lot_of(w.rahim), 1, "12-10-2026", "1", "580"));
    (w, garden)
}

#[test]
fn t12_fruit_from_a_garden_lot_is_money_in_for_the_garden_and_closes_with_it() {
    let (w, garden) = garden_with_fruit();
    // R36 before the closing: our shop 56,400, other shops 10,000, fruit (other money in) 1,200,
    // money in 67,600, balance 67,600 - 55,000 = 12,600.
    let page = report(&w, ReportId::R36, ReportFilters::default(), 0, 0);
    let f: Vec<i64> = page.rows[0].cells[3..].iter().map(n).collect();
    // [rent, installments, costs, paid shop, paid pocket, our shop, other shops, other in,
    //  commission, money in, balance, rent due]
    assert_eq!(f[5], 56_400);
    assert_eq!(f[6], 10_000);
    assert_eq!(f[7], 1_200);
    assert_eq!((f[9], f[10]), (67_600, 12_600));
    // View I says the same.
    let i = av(&w, AnalyticsView::I, AnalyticsFilters::default());
    assert_eq!(data_rows(&i).len(), 1);

    // R37 with the garden: only the fruit of its lot; all: both takes.
    let one_garden = ReportFilters {
        garden_id: Some(garden.id),
        ..ReportFilters::default()
    };
    let g = r37(&w, one_garden);
    assert_eq!(
        entries(&g),
        vec![entry(
            "Partner A",
            "Banana",
            2,
            600,
            1_200,
            "fruit-source-own"
        )]
    );
    assert_eq!(entries(&r37(&w, ReportFilters::default())).len(), 2);

    // View K: the goods filter tells the garden's fruit from the consigned.
    let filters = |goods| AnalyticsFilters {
        goods,
        ..AnalyticsFilters::default()
    };
    assert_eq!(
        k_totals(&av(&w, AnalyticsView::K, filters(AnalyticsGoods::All))),
        (3, 1_780, 1_780, 0)
    );
    let gk = av(&w, AnalyticsView::K, filters(AnalyticsGoods::Garden));
    assert_eq!(
        k_rows(&gk),
        vec![("Partner A".into(), "Banana".into(), 2, 1_200, 1_200, 0)]
    );
    let ck = av(&w, AnalyticsView::K, filters(AnalyticsGoods::Consigned));
    assert_eq!(
        k_rows(&ck),
        vec![("Partner B".into(), "Banana".into(), 1, 580, 580, 0)]
    );
    let by_garden = av(
        &w,
        AnalyticsView::K,
        AnalyticsFilters {
            garden_id: Some(garden.id),
            ..AnalyticsFilters::default()
        },
    );
    assert_eq!(k_totals(&by_garden), (2, 1_200, 1_200, 0));
    // The register and the view give the same totals under the same garden filter.
    let reg = r37(
        &w,
        ReportFilters {
            garden_id: Some(garden.id),
            ..ReportFilters::default()
        },
    );
    assert_eq!(
        (n(&reg.totals[7]), n(&reg.totals[9])),
        (k_totals(&by_garden).0, k_totals(&by_garden).1)
    );

    // The closing on 13-10: 67,600 at 60 / 40 = 40,560 / 27,040; results 7,560 / 5,040.
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let by_partner = report(
        &w,
        ReportId::R36,
        ReportFilters {
            garden_view: Some(mandi_reports::reports::GardenView::Partner),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    let partners: Vec<(i64, i64, i64, i64, i64)> = by_partner
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .map(|r| {
            (
                n(&r.cells[2]),
                n(&r.cells[3]),
                n(&r.cells[4]),
                n(&r.cells[5]),
                n(&r.cells[6]),
            )
        })
        .collect();
    assert_eq!(
        partners,
        vec![
            (6_000, 33_000, 0, 40_560, 7_560),
            (4_000, 0, 22_000, 27_040, 5_040)
        ]
    );
}

#[test]
fn the_fruit_of_a_garden_and_the_gardens_column_of_view_f_do_not_overlap() {
    let (w, garden) = garden_with_fruit();
    set_today(&w.env, 2026, 10, 13);
    garden_close(&w, garden.id, "13-10-2026");
    let f = on("10-10-2026", "13-10-2026");
    let page = av(&w, AnalyticsView::F, f.clone());
    let gardens = column(&page, "gardens");
    let fruit = column(&page, "fruit");
    let other = column(&page, "other");
    // Gardens: -33,000 + 40,560 = 7,560 and 27,040 (his money back, nothing paid from his shop
    // cash); fruit: الف -1,200, ب -580; nothing is "other".
    let rows = data_rows(&page);
    assert_eq!(
        rows.iter()
            .map(|r| n(&r.cells[gardens]))
            .collect::<Vec<_>>(),
        [7_560, 27_040, 0]
    );
    assert_eq!(
        rows.iter().map(|r| n(&r.cells[fruit])).collect::<Vec<_>>(),
        [-1_200, -580, 0]
    );
    assert!(rows.iter().all(|r| n(&r.cells[other]) == 0));
    // The fruit column is minus view K, and the lines of الف's cell add up to it.
    let k = av(&w, AnalyticsView::K, f.clone());
    assert_eq!(n(&page.totals[fruit]), -k_totals(&k).1);
    let alif = AnalyticsKey {
        partner_id: Some(w.partners[0]),
        ..AnalyticsKey::default()
    };
    let col_key = AnalyticsKey {
        money: Some(AnalyticsPartnerMoney::Fruit),
        ..AnalyticsKey::default()
    };
    let cell = lines_of(&w, AnalyticsView::F, f, alif, Some(col_key));
    assert_eq!((cell.total_count, n(&cell.totals[3])), (1, -1_200));
}

// ---- Random sequences: the books and the reports agree -------------------------------------------

/// A small deterministic generator, so a failure repeats.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, below: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % below
    }
}

/// A live entry: its id, version and whether it came from the lot.
struct Live {
    id: i64,
    version: i64,
    from_lot: bool,
}

#[test]
fn for_random_takes_edits_and_cancels_the_fruit_column_is_minus_view_k_and_the_register() {
    // Code Standards 9.3: for any sequence, the partners' fruit (the books) is minus the fruit
    // entries (the register, view K and R34), for every period. Two open days, one lot with 5 left.
    for seed in [1_u64, 7, 42, 2026] {
        let w = after_e2e_16();
        set_today(&w.env, 2026, 10, 11);
        let agent = nazeer(&w);
        let mut rng = Lcg(seed);
        let mut live: Vec<Live> = Vec::new();
        let days = ["10-10-2026", "11-10-2026"];
        for step in 0..30 {
            let day_pick = days[rng.next(2) as usize];
            let has_home = live.iter().any(|l| l.from_lot);
            match rng.next(4) {
                // A new guest take from another shop.
                0 | 1 => {
                    let qty = (rng.next(4) + 1).to_string();
                    let rate = format!("{}", (rng.next(15) + 1) * 100);
                    let saved = take(&w, &guests(&w, agent, day_pick, &qty, &rate));
                    live.push(Live {
                        id: saved.id,
                        version: saved.version,
                        from_lot: false,
                    });
                }
                // A new home take from the lot, one at a time (5 are left).
                2 if !has_home => {
                    let qty = (rng.next(5) + 1).to_string();
                    let saved = take(
                        &w,
                        &home(&w, 1, rng.next(3) as usize, day_pick, &qty, "580"),
                    );
                    live.push(Live {
                        id: saved.id,
                        version: saved.version,
                        from_lot: true,
                    });
                }
                // Edit or cancel a live entry.
                _ if !live.is_empty() => {
                    let index = rng.next(live.len() as u64) as usize;
                    let entry = &mut live[index];
                    if rng.next(3) == 0 {
                        fruit::cancel(
                            &w.env.ctx,
                            &w.owner,
                            &FruitCancelInput {
                                id: entry.id,
                                version: entry.version,
                                reason: "غلط اندراج".into(),
                            },
                        )
                        .unwrap();
                        live.remove(index);
                    } else {
                        let qty = (rng.next(4) + 1).to_string();
                        let rate = format!("{}", (rng.next(15) + 1) * 100);
                        let input = if entry.from_lot {
                            home(&w, 1, rng.next(3) as usize, day_pick, &qty, &rate)
                        } else {
                            guests(&w, agent, day_pick, &qty, &rate)
                        };
                        let edited = fruit::edit(
                            &w.env.ctx,
                            &w.owner,
                            &FruitEditInput {
                                id: entry.id,
                                version: entry.version,
                                entry: input,
                                reason: "درستی".into(),
                            },
                        )
                        .unwrap();
                        entry.version = edited.version;
                    }
                }
                _ => {}
            }
            for (from, to) in [
                ("10-10-2026", "10-10-2026"),
                ("11-10-2026", "11-10-2026"),
                ("10-10-2026", "11-10-2026"),
            ] {
                let (parts, total) = f_fruit(&w, on(from, to));
                assert_eq!(parts.iter().sum::<i64>(), total, "seed {seed} step {step}");
                let k = av(&w, AnalyticsView::K, on(from, to));
                let (qty, value, own, other) = k_totals(&k);
                assert_eq!(
                    total, -value,
                    "seed {seed} step {step} {from}-{to}: F against K"
                );
                assert_eq!(own + other, value, "seed {seed} step {step}");
                let register = r37(&w, day(from, to));
                assert_eq!(
                    (n(&register.totals[7]), n(&register.totals[9])),
                    (qty, value),
                    "seed {seed} step {step} {from}-{to}: R37 against K"
                );
                // R34 adds up to the same over the days of the period.
                let r34: (i64, i64) = r34_fruit(&w, from, to)
                    .iter()
                    .fold((0, 0), |a, (_, q, v)| (a.0 + q, a.1 + v));
                assert_eq!(
                    r34,
                    (qty, value),
                    "seed {seed} step {step} {from}-{to}: R34"
                );
                // The lines behind the column add up to the column.
                let every = lines_of(
                    &w,
                    AnalyticsView::F,
                    on(from, to),
                    AnalyticsKey::default(),
                    Some(AnalyticsKey {
                        money: Some(AnalyticsPartnerMoney::Fruit),
                        ..AnalyticsKey::default()
                    }),
                );
                assert_eq!(
                    n(&every.totals[3]),
                    total,
                    "seed {seed} step {step}: F lines"
                );
            }
        }
    }
}

// ---- Request and answer pairs for the frontend ---------------------------------------------------
//
// Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is read
// back into the real input type before it is used) and `<name>.json` is the answer the real engine
// gives for it on the E2E-42 world (or the garden world of T12). The frontend copies the files. The
// test fails if a file differs from the real answer; after a deliberate change, write them again with
// `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test reports_fruit fixtures`.

mod fixtures {
    use std::path::{Path, PathBuf};

    use mandi_reports::analytics::{AnalyticsPrintInput, AnalyticsRunInput};
    use mandi_reports::reports::{ReportPrintInput, ReportRunInput, print_data};
    use serde::Serialize;
    use serde_json::{Value, json};

    use super::*;

    const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";

    fn folder() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }

    fn write_or_check(file: &str, text: &str) {
        let path = folder().join(file);
        if std::env::var_os(WRITE_VARIABLE).is_some() {
            std::fs::create_dir_all(folder()).unwrap();
            std::fs::write(&path, text).unwrap();
            return;
        }
        let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!("{file} is missing: run with {WRITE_VARIABLE}=1 to write the fixtures")
        });
        assert!(
            on_disk == text,
            "{file} is not the real answer any more. If the change is on purpose, run with \
             {WRITE_VARIABLE}=1, read the diff and tell frontend."
        );
    }

    fn pair<A: Serialize>(name: &str, request: &Value, answer: &A) {
        write_or_check(
            &format!("{name}_request.json"),
            &(serde_json::to_string_pretty(request).unwrap() + "\n"),
        );
        write_or_check(
            &format!("{name}.json"),
            &(serde_json::to_string_pretty(answer).unwrap() + "\n"),
        );
    }

    fn typed<T: serde::de::DeserializeOwned>(request: &Value) -> T {
        serde_json::from_value(request.clone()).unwrap()
    }

    /// The report filters with every required key, as the frontend sends them.
    fn rfilters(extra: Value) -> Value {
        let mut base = json!({
            "shortcut": null, "from": "", "to": "", "as_of": "", "party": null,
            "seller_id": null, "zamindar_id": null, "item_id": null, "city_id": null,
            "adda_id": null, "detail": false, "partner_id": null, "garden_id": null,
            "bikri_id": null
        });
        merge(&mut base, &extra);
        base
    }

    /// The analytics filters with every key, as the frontend sends them.
    fn afilters(extra: Value) -> Value {
        let mut base = json!({
            "period": null, "from": "", "to": "", "group_by": null, "item_id": null,
            "seller_id": null, "buyer": null, "city_id": null, "adda_id": null,
            "side": "both", "compare": "off", "partner_id": null, "trip_id": null,
            "goods": "all", "trip_view": null, "garden_id": null
        });
        merge(&mut base, &extra);
        base
    }

    fn merge(base: &mut Value, extra: &Value) {
        if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
            for (k, v) in extra {
                base.insert(k.clone(), v.clone());
            }
        }
    }

    fn run_pair(w: &World, name: &str, request: Value) {
        let input: ReportRunInput = typed(&request);
        let page = mandi_reports::reports::run(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &page);
    }

    fn print_pair(w: &World, name: &str, request: Value) {
        let input: ReportPrintInput = typed(&request);
        let data = print_data(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &data);
    }

    fn a_run_pair(w: &World, name: &str, request: Value) {
        let input: AnalyticsRunInput = typed(&request);
        let page = run(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &page);
    }

    fn a_lines_pair(w: &World, name: &str, request: Value) {
        let input: AnalyticsLinesInput = typed(&request);
        let page = lines(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &page);
    }

    fn a_print_pair(w: &World, name: &str, request: Value) {
        let input: AnalyticsPrintInput = typed(&request);
        let data = mandi_reports::analytics::print_data(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &data);
    }

    #[test]
    fn fixtures_are_the_real_answers() {
        let (w, _, _) = e2e_42();
        let day10 = rfilters(json!({ "from": "10-10-2026", "to": "10-10-2026" }));
        run_pair(
            &w,
            "reports_run_r37_e2e_42",
            json!({ "id": "R37", "filters": day10, "offset": 0, "limit": 0 }),
        );
        run_pair(
            &w,
            "reports_run_r37_guests",
            json!({ "id": "R37",
                "filters": rfilters(json!({ "from": "10-10-2026", "to": "10-10-2026", "guests_only": true })),
                "offset": 0, "limit": 0 }),
        );
        print_pair(
            &w,
            "reports_print_data_r37_en",
            json!({ "id": "R37", "filters": day10, "language": "en", "paper": null }),
        );
        run_pair(
            &w,
            "reports_run_r10_fruit",
            json!({ "id": "R10", "filters": rfilters(json!({})), "offset": 0, "limit": 0 }),
        );
        run_pair(
            &w,
            "reports_run_r34_fruit",
            json!({ "id": "R34", "filters": day10, "offset": 0, "limit": 0 }),
        );
        let a10 = afilters(json!({ "from": "10-10-2026", "to": "10-10-2026" }));
        a_run_pair(
            &w,
            "analytics_run_k_e2e_42",
            json!({ "view": "K", "filters": a10, "offset": 0, "limit": 0 }),
        );
        a_lines_pair(
            &w,
            "analytics_lines_k_guests",
            json!({ "view": "K", "filters": a10, "row_key": { "guests": true, "item_id": w.apple },
                "column_key": null, "offset": 0, "limit": 0 }),
        );
        a_print_pair(
            &w,
            "analytics_print_data_k_en",
            json!({ "view": "K", "filters": a10, "language": "en", "paper": null }),
        );
        a_run_pair(
            &w,
            "analytics_run_f_fruit",
            json!({ "view": "F", "filters": a10, "offset": 0, "limit": 0 }),
        );
        a_lines_pair(
            &w,
            "analytics_lines_f_fruit_alif",
            json!({ "view": "F", "filters": a10,
                "row_key": { "partner_id": w.partners[0] },
                "column_key": { "money": "fruit" }, "offset": 0, "limit": 0 }),
        );
    }
}
