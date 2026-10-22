//! The Sales group of the report engine on the PRD worked day (plan 20 step B3, PRD 19.2,
//! FR-13.6, FR-19.10, FR-19.11): R2, R3, R4, R6, R7, R8, R9 and R27.
//!
//! The data is made the way `golden_day.rs` makes it (Events 1 to 5 of PRD Section 4, through the
//! real services; `reports_world`). The expected numbers are the PRD's: the three registers add
//! up to 41,400 and 44,364, R6 is the seller summary of E2E-1, R7 is view A of PRD 18.4, R9 is
//! the register of the one-time screen. A report total is also checked against the stored lots
//! and facts, so a report can never disagree with the bills (money agent Section 9).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::reports::{
    Cell, DateShortcut, FilterKind, PaidFilter, PartyFilter, PrintCell, ReportFilters, ReportGroup,
    ReportId, ReportPage, ReportPrintInput, catalog, print_data,
};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{CancelInput, PukhtaRegisterInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::{bikri, vouchers};

use reports_world::*;

fn number(cell: &Cell) -> i64 {
    match cell {
        Cell::Number { value } => *value,
        Cell::Empty => 0,
        other => panic!("not a number: {other:?}"),
    }
}

fn names_en(cell: &Cell) -> String {
    match cell {
        Cell::Names { names } => names.en.clone(),
        Cell::Empty => String::new(),
        other => panic!("not names: {other:?}"),
    }
}

fn label(cell: &Cell) -> String {
    match cell {
        Cell::Label { id } => id.clone(),
        other => panic!("not a label: {other:?}"),
    }
}

fn party_name(cell: &Cell) -> (String, String, Option<String>) {
    match cell {
        Cell::Party { party } => (
            party.code.clone(),
            party.name_en.clone(),
            party.person_key.clone(),
        ),
        other => panic!("not a party: {other:?}"),
    }
}

/// The cells of a row of money, by column index.
fn moneys(page: &ReportPage, row: usize, columns: &[usize]) -> Vec<i64> {
    let cells = &page.rows.get(row).unwrap().cells;
    columns
        .iter()
        .map(|c| money(cells.get(*c).unwrap()))
        .collect()
}

// ---------------------------------------------------------------------------------------------

#[test]
fn the_catalogue_lists_the_sales_and_the_other_groups_with_their_filters() {
    let entries = catalog();
    let of = |group: ReportGroup| -> Vec<ReportId> {
        entries
            .iter()
            .filter(|e| e.group == group)
            .map(|e| e.id)
            .collect()
    };
    assert_eq!(
        of(ReportGroup::Sales),
        vec![
            ReportId::R2,
            ReportId::R3,
            ReportId::R4,
            ReportId::R6,
            ReportId::R7,
            ReportId::R8,
            ReportId::R9
        ]
    );
    assert_eq!(
        of(ReportGroup::Collections),
        vec![
            ReportId::R20,
            ReportId::R21,
            ReportId::R22,
            ReportId::R24,
            ReportId::R26,
            ReportId::R27
        ]
    );
    assert_eq!(
        of(ReportGroup::Business),
        vec![ReportId::R28, ReportId::R30]
    );
    let r2 = entries.iter().find(|e| e.id == ReportId::R2).unwrap();
    assert_eq!(
        r2.filters,
        vec![
            FilterKind::DateRange,
            FilterKind::Party,
            FilterKind::Seller,
            FilterKind::Zamindar,
            FilterKind::Item,
            FilterKind::City,
            FilterKind::Adda
        ]
    );
    assert_eq!(r2.group_title_id, "report-group-sales");
    assert_eq!(r2.title_id, "report-r2-title");
    assert!(!r2.party_required);
    let r9 = entries.iter().find(|e| e.id == ReportId::R9).unwrap();
    assert_eq!(
        r9.filters,
        vec![FilterKind::DateRange, FilterKind::Party, FilterKind::Status]
    );
}

// --- R2, R3, R4 ----------------------------------------------------------------------------------

#[test]
fn r2_lists_every_sale_line_and_its_totals_are_the_bills_of_the_day() {
    let w = full_day();
    let page = report(&w, ReportId::R2, ReportFilters::default(), 0, 0);
    assert_eq!(page.total_count, 3);
    // Columns: date, doc, buyer, city, item, qty, rate, value, buyer_total.
    assert_eq!(
        page.columns
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>(),
        vec![
            "date",
            "doc",
            "buyer",
            "city",
            "item",
            "qty",
            "rate",
            "value",
            "buyer_total"
        ]
    );
    // Event 1: اکرم 20 at 600 = 12,000 and he pays 12,880 (E2E-2); ارشد 30 at 580 = 17,400, 18,684.
    let first = &page.rows.first().unwrap().cells;
    assert_eq!(code_of(first.get(2).unwrap()), "03040001");
    assert_eq!(names_en(first.get(4).unwrap()), "Banana");
    assert_eq!(first.get(3), Some(&Cell::Empty));
    assert_eq!(
        (number(first.get(5).unwrap()), money(first.get(6).unwrap())),
        (20, 600)
    );
    assert_eq!(moneys(&page, 0, &[7, 8]), vec![12_000, 12_880]);
    assert_eq!(moneys(&page, 1, &[6, 7, 8]), vec![580, 17_400, 18_684]);
    // Event 2: a one-time buyer, shown by his own name and person key under the پختہ account.
    let (code, name, key) = party_name(page.rows.get(2).unwrap().cells.get(2).unwrap());
    assert_eq!((code.as_str(), name.as_str()), ("03049999", "Shahid"));
    assert!(key.is_some_and(|k| k.contains("03000000000")));
    assert_eq!(moneys(&page, 2, &[7, 8]), vec![12_000, 12_800]);
    // The totals: 60 نگ, value 41,400 (= خام بکری of the day), buyer totals 44,364.
    assert_eq!(page.totals.get(5).map(number), Some(60));
    assert_eq!(page.totals.get(7).map(money), Some(41_400));
    assert_eq!(page.totals.get(8).map(money), Some(44_364));
    // Each row opens its بکری.
    assert!(
        page.rows
            .iter()
            .all(|r| r.source.is_some_and(|s| format!("{:?}", s.kind) == "Bikri"))
    );
}

#[test]
fn r3_and_r4_split_r2_into_cash_and_credit_lines() {
    let w = full_day();
    let all = report(&w, ReportId::R2, ReportFilters::default(), 0, 0);
    let cash = report(&w, ReportId::R3, ReportFilters::default(), 0, 0);
    let credit = report(&w, ReportId::R4, ReportFilters::default(), 0, 0);
    // شاہد paid on the spot (نقدی); اکرم and ارشد are on credit (ادھاری).
    assert_eq!((cash.total_count, credit.total_count), (1, 2));
    assert_eq!(cash.totals.get(8).map(money), Some(12_800));
    assert_eq!(credit.totals.get(8).map(money), Some(31_564));
    // The two reports add up to the full register, column by column.
    for column in [5_usize, 7, 8] {
        let sum = |p: &ReportPage| {
            p.totals
                .get(column)
                .map(|c| match c {
                    Cell::Number { value } => *value,
                    other => money(other),
                })
                .unwrap()
        };
        assert_eq!(sum(&cash) + sum(&credit), sum(&all), "column {column}");
    }
    assert_eq!(cash.title_id, "report-r3-title");
    assert_eq!(credit.title_id, "report-r4-title");
}

#[test]
fn the_registers_filter_by_date_party_seller_item_and_page_by_page() {
    let w = full_day();
    let run = |filters: ReportFilters| report(&w, ReportId::R2, filters, 0, 0);
    // A day with no sale, and the shortcut for today.
    let empty = run(ReportFilters {
        from: "11-10-2026".into(),
        ..ReportFilters::default()
    });
    assert_eq!((empty.total_count, empty.rows.len()), (0, 0));
    assert_eq!(empty.totals.get(8).map(money), Some(0));
    let today = run(ReportFilters {
        shortcut: Some(DateShortcut::Today),
        ..ReportFilters::default()
    });
    assert_eq!(today.total_count, 3);
    // One buyer; one seller; one item; the buyer's city (no buyer has one).
    assert_eq!(run(party(w.akram)).total_count, 1);
    assert_eq!(
        run(ReportFilters {
            seller_id: Some(w.karim),
            ..ReportFilters::default()
        })
        .total_count,
        1
    );
    assert_eq!(
        run(ReportFilters {
            item_id: Some(w.banana),
            ..ReportFilters::default()
        })
        .total_count,
        2
    );
    assert_eq!(
        run(ReportFilters {
            item_id: Some(w.banana),
            seller_id: Some(w.karim),
            ..ReportFilters::default()
        })
        .total_count,
        0,
        "the filters combine"
    );
    // One-time buyer by his person key.
    let key: String = raw(&w.dir)
        .query_row(
            "SELECT pukhta_person_key FROM bikri_sales WHERE pukhta_person_key IS NOT NULL LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let one = run(ReportFilters {
        party: Some(PartyFilter {
            account_id: w.pukhta_buyer,
            person_key: Some(key),
        }),
        ..ReportFilters::default()
    });
    assert_eq!(one.total_count, 1);
    // Pages: the totals stay those of the whole filter, the rows are cut.
    let first = report(&w, ReportId::R2, ReportFilters::default(), 0, 2);
    let second = report(&w, ReportId::R2, ReportFilters::default(), 2, 2);
    assert_eq!((first.rows.len(), second.rows.len()), (2, 1));
    assert_eq!(first.totals, second.totals);
    assert_eq!(first.total_count, 3);
    assert_eq!(first.totals.get(8).map(money), Some(44_364));
}

#[test]
fn a_cancelled_bikri_leaves_the_registers_and_an_edit_shows_only_the_new_line() {
    let w = worked_day();
    save(&w, &support::event1(&w));
    let two = save(&w, &event2(&w));
    // All three lines in force.
    assert_eq!(
        report(&w, ReportId::R2, ReportFilters::default(), 0, 0).total_count,
        3
    );
    // E2E-15: cancel بکری 2.
    let detail = bikri::get(&w.env.ctx, two.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: two.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let page = report(&w, ReportId::R2, ReportFilters::default(), 0, 0);
    assert_eq!(page.total_count, 2);
    assert_eq!(page.totals.get(7).map(money), Some(29_400));
    // R6 and R7 leave the cancelled lot and item too.
    let r6 = report(&w, ReportId::R6, ReportFilters::default(), 0, 0);
    assert_eq!(r6.total_count, 1);
    let r7 = report(&w, ReportId::R7, ReportFilters::default(), 0, 0);
    assert_eq!(
        r7.total_count, 1,
        "سیب adds nothing after the cancel (FR-18.7)"
    );
    assert_eq!(
        names_en(r7.rows.first().unwrap().cells.first().unwrap()),
        "Banana"
    );
    assert_eq!(r7.totals.get(7).map(money), Some(3_928));
    // R9 still lists the cancelled one-time line, marked cancelled and left out of the totals.
    let r9 = report(&w, ReportId::R9, ReportFilters::default(), 0, 0);
    assert_eq!(r9.total_count, 1);
    assert_eq!(
        label(r9.rows.first().unwrap().cells.get(9).unwrap()),
        "report-status-cancelled"
    );
    assert_eq!(r9.totals.get(7).map(money), Some(0));
}

// --- R6 and R27 ----------------------------------------------------------------------------------

#[test]
fn r6_is_the_seller_summary_of_each_lot_and_the_deductions_add_up() {
    let w = full_day();
    let page = report(&w, ReportId::R6, ReportFilters::default(), 0, 0);
    assert_eq!(page.total_count, 2);
    let keys: Vec<&str> = page.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(
        keys,
        vec![
            "date",
            "doc",
            "seller",
            "item",
            "qty",
            "gross",
            "freight",
            "commission",
            "labour",
            "munshiana",
            "cash_paid",
            "store_rent",
            "phone_charge",
            "deductions",
            "net"
        ]
    );
    // E2E-1, رحیم's lot: خام 29,400; کرایہ 3,000; کمیشن 1,764; مزدوری 250; منشیانہ 50; نقد 5,000;
    // فون 20; کل اخراجات 10,084; صافی 19,316.
    let all: Vec<usize> = (5..15).collect();
    assert_eq!(
        moneys(&page, 0, &all),
        vec![29_400, 3_000, 1_764, 250, 50, 5_000, 0, 20, 10_084, 19_316]
    );
    // PRD 4.5, کریم's lot: خام 12,000; کرایہ 1,000; کمیشن 720; مزدوری 50; منشیانہ 50; فون 20;
    // کل اخراجات 1,840; صافی 10,160.
    assert_eq!(
        moneys(&page, 1, &all),
        vec![12_000, 1_000, 720, 50, 50, 0, 0, 20, 1_840, 10_160]
    );
    assert_eq!(
        code_of(page.rows.first().unwrap().cells.get(2).unwrap()),
        "030110001"
    );
    assert_eq!(number(page.rows.first().unwrap().cells.get(4).unwrap()), 50);
    // The total row, and the identity: خام بکری − کل اخراجات = صافی بکری; the seven columns add
    // up to کل اخراجات.
    let totals: Vec<i64> = all
        .iter()
        .map(|c| money(page.totals.get(*c).unwrap()))
        .collect();
    assert_eq!(
        totals,
        vec![41_400, 4_000, 2_484, 300, 100, 5_000, 0, 40, 11_924, 29_476]
    );
    assert_eq!(totals[0] - totals[8], totals[9]);
    assert_eq!(totals[1..8].iter().sum::<i64>(), totals[8]);
    // Filters: one seller; one item that has no lot.
    let karim = report(
        &w,
        ReportId::R6,
        ReportFilters {
            seller_id: Some(w.karim),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(karim.total_count, 1);
    assert_eq!(karim.totals.get(14).map(money), Some(10_160));
    let none = report(
        &w,
        ReportId::R6,
        ReportFilters {
            item_id: Some(w.mooli),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(none.total_count, 0);
}

#[test]
fn r6_shows_a_lot_with_all_its_postings_after_a_later_sale() {
    // E2E-16 and E2E-16b: 45 of 50 sold, then the last 5 sold on 11-10-26. The lot shows the
    // totals of both postings: خام 29,300, کل اخراجات 10,078, صافی 19,222.
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].sales = vec![
        support::sale("s-1", w.akram, "20", "600"),
        support::sale("s-2", w.arshad, "25", "580"),
    ];
    let saved = save(&w, &draft);
    let first = report(&w, ReportId::R6, ReportFilters::default(), 0, 0);
    assert_eq!(moneys(&first, 0, &[5, 13, 14]), vec![26_500, 9_910, 16_590]);
    let lot_id: i64 = raw(&w.dir)
        .query_row(
            "SELECT id FROM bikri_lots WHERE bikri_id = ?1",
            [saved.bikri_id],
            |r| r.get(0),
        )
        .unwrap();
    set_today(&w.env, 2026, 10, 11);
    let mut line = support::sale("s-3", w.arshad, "5", "560");
    line.sale_id = None;
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::LaterSaleSaveInput {
            lot_id,
            date: "11-10-2026".into(),
            sales: vec![line],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let second = report(&w, ReportId::R6, ReportFilters::default(), 0, 0);
    assert_eq!(second.total_count, 1, "still one lot");
    assert_eq!(
        moneys(&second, 0, &[5, 13, 14]),
        vec![29_300, 10_078, 19_222]
    );
    assert_eq!(
        number(second.rows.first().unwrap().cells.get(4).unwrap()),
        50
    );
    // The register shows the later sale's line under its own date.
    let r2 = report(&w, ReportId::R2, ReportFilters::default(), 0, 0);
    assert_eq!(r2.total_count, 3);
    match r2.rows.last().unwrap().cells.first() {
        Some(Cell::Date { iso }) => assert_eq!(iso, "2026-10-11"),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        r2.rows.last().unwrap().cells.get(1),
        Some(&Cell::Doc {
            type_id: "report-doc-bikri-later".into(),
            number: Some(1)
        })
    );
}

#[test]
fn r27_is_the_bill_series_in_number_order() {
    let w = full_day();
    let page = report(&w, ReportId::R27, ReportFilters::default(), 0, 0);
    assert_eq!(page.total_count, 2);
    // doc, date, bill_date, seller, gross, deductions, net.
    assert_eq!(moneys(&page, 0, &[4, 5, 6]), vec![29_400, 10_084, 19_316]);
    assert_eq!(moneys(&page, 1, &[4, 5, 6]), vec![12_000, 1_840, 10_160]);
    assert_eq!(
        page.rows.first().unwrap().cells.first(),
        Some(&Cell::Doc {
            type_id: "report-doc-bikri-first".into(),
            number: Some(1)
        })
    );
    assert_eq!(
        page.rows.get(1).unwrap().cells.first(),
        Some(&Cell::Doc {
            type_id: "report-doc-bikri-first".into(),
            number: Some(2)
        })
    );
    assert_eq!(
        moneys_of_totals(&page, &[4, 5, 6]),
        vec![41_400, 11_924, 29_476]
    );
    // It is R6 with the lots of one بکری and seller together: the same three totals.
    let r6 = report(&w, ReportId::R6, ReportFilters::default(), 0, 0);
    assert_eq!(
        moneys_of_totals(&r6, &[5, 13, 14]),
        moneys_of_totals(&page, &[4, 5, 6])
    );
    // A page of one row keeps the totals.
    let one = report(&w, ReportId::R27, ReportFilters::default(), 1, 1);
    assert_eq!((one.rows.len(), one.total_count), (1, 2));
    assert_eq!(one.totals, page.totals);
}

fn moneys_of_totals(page: &ReportPage, columns: &[usize]) -> Vec<i64> {
    columns
        .iter()
        .map(|c| money(page.totals.get(*c).unwrap()))
        .collect()
}

// --- R7 ------------------------------------------------------------------------------------------

#[test]
fn r7_is_view_a_of_prd_18_4_with_the_average_rate_and_the_commission_columns() {
    let w = full_day();
    let page = report(&w, ReportId::R7, ReportFilters::default(), 0, 0);
    assert_eq!(page.total_count, 2);
    // item, qty, gross, avg_rate, seller_comm, buyer_comm, laga, total_comm.
    let banana = &page.rows.first().unwrap().cells;
    assert_eq!(names_en(banana.first().unwrap()), "Banana");
    assert_eq!(number(banana.get(1).unwrap()), 50);
    assert_eq!(
        banana[2..].iter().map(money).collect::<Vec<_>>(),
        vec![29_400, 588, 1_764, 1_764, 400, 3_928]
    );
    let apple = &page.rows.get(1).unwrap().cells;
    assert_eq!(names_en(apple.first().unwrap()), "Apple");
    assert_eq!(number(apple.get(1).unwrap()), 10);
    assert_eq!(
        apple[2..].iter().map(money).collect::<Vec<_>>(),
        vec![12_000, 1_200, 720, 720, 80, 1_520]
    );
    // The total row: 60 نگ, 41,400, no average, 2,484, 2,484, 480, 5,448 (= today's commission).
    assert_eq!(number(page.totals.get(1).unwrap()), 60);
    assert_eq!(page.totals.get(3), Some(&Cell::Empty));
    assert_eq!(
        [2_usize, 4, 5, 6, 7]
            .iter()
            .map(|c| money(page.totals.get(*c).unwrap()))
            .collect::<Vec<_>>(),
        vec![41_400, 2_484, 2_484, 480, 5_448]
    );
    // It agrees with the journal: the commission income accounts' net credit for the day.
    let seller_comm = net_credit(&w, "04010001");
    let buyer_comm = net_credit(&w, "04010002");
    let laga = net_credit(&w, "04010003");
    assert_eq!(
        (seller_comm, buyer_comm, laga),
        (2_484, 2_484, 480),
        "the report totals equal the journal"
    );
    // One item, one seller.
    let one = report(
        &w,
        ReportId::R7,
        ReportFilters {
            item_id: Some(w.apple),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(one.total_count, 1);
    assert_eq!(one.totals.get(7).map(money), Some(1_520));
    let by_seller = report(
        &w,
        ReportId::R7,
        ReportFilters {
            seller_id: Some(w.rahim),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(by_seller.totals.get(7).map(money), Some(3_928));
}

/// The net credit of an account in whole rupees, from the services ledger.
fn net_credit(w: &support::World, code: &str) -> i64 {
    let id = support::id_of(&w.env, code);
    let b = mandi_services::ledger::account_balance(&w.env.ctx, id).unwrap();
    b.amount_paisa.rupees()
}

// --- R8 ------------------------------------------------------------------------------------------

#[test]
fn r8_counts_the_trucks_lots_and_goods_of_each_adda() {
    let w = full_day();
    // The worked day has no اڈا on its headers: one row without a name, last.
    let none = report(&w, ReportId::R8, ReportFilters::default(), 0, 0);
    assert_eq!(none.total_count, 1);
    assert_eq!(none.rows.first().unwrap().cells.first(), Some(&Cell::Empty));
    // Give بکری 1 an اڈا (a plain update of the test data) and look again.
    let conn = raw(&w.dir);
    conn.execute(
        "INSERT INTO addas (code, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES ('1', 'سوات اڈا', 'Swat Adda', 'swat', 'swat adda', 'swt')",
        [],
    )
    .unwrap();
    let adda = conn.last_insert_rowid();
    conn.execute(
        "UPDATE bikri_entries SET adda_id = ?1 WHERE number = 1",
        [adda],
    )
    .unwrap();
    drop(conn);
    let page = report(&w, ReportId::R8, ReportFilters::default(), 0, 0);
    assert_eq!(page.total_count, 2);
    // adda, trucks, lots, qty, gross.
    let swat = &page.rows.first().unwrap().cells;
    assert_eq!(names_en(swat.first().unwrap()), "Swat Adda");
    assert_eq!(
        (
            number(&swat[1]),
            number(&swat[2]),
            number(&swat[3]),
            money(&swat[4])
        ),
        (1, 1, 50, 29_400)
    );
    let rest = &page.rows.get(1).unwrap().cells;
    assert_eq!(rest.first(), Some(&Cell::Empty));
    assert_eq!(
        (
            number(&rest[1]),
            number(&rest[2]),
            number(&rest[3]),
            money(&rest[4])
        ),
        (1, 1, 10, 12_000)
    );
    assert_eq!(
        (
            number(page.totals.get(1).unwrap()),
            number(page.totals.get(2).unwrap()),
            number(page.totals.get(3).unwrap()),
            money(page.totals.get(4).unwrap())
        ),
        (2, 2, 60, 41_400)
    );
    // The filter keeps one اڈا.
    let only = report(
        &w,
        ReportId::R8,
        ReportFilters {
            adda_id: Some(adda),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(only.total_count, 1);
    assert_eq!(only.totals.get(4).map(money), Some(29_400));
}

// --- R9 ------------------------------------------------------------------------------------------

#[test]
fn r9_lists_the_one_time_line_as_paid_with_the_numbers_of_prd_4_5() {
    let w = full_day();
    let page = report(&w, ReportId::R9, ReportFilters::default(), 0, 0);
    assert_eq!(page.total_count, 1);
    let row = &page.rows.first().unwrap().cells;
    // date, doc, side, person, phone, item, qty, amount, commission, status, open.
    assert_eq!(label(&row[2]), "report-side-buyer");
    let (code, name, key) = party_name(&row[3]);
    assert_eq!((code.as_str(), name.as_str()), ("03049999", "Shahid"));
    assert!(key.is_some());
    assert_eq!(
        row[4],
        Cell::Text {
            text: "03000000000".into()
        }
    );
    assert_eq!(names_en(&row[5]), "Apple");
    assert_eq!(number(&row[6]), 10);
    // He pays 12,800; his commission is 720; paid, nothing open (PRD 4.5, E2E-5).
    assert_eq!((money(&row[7]), money(&row[8])), (12_800, 720));
    assert_eq!(label(&row[9]), "report-status-paid");
    assert_eq!(row[10], Cell::Empty);
    assert_eq!(
        [6_usize, 7, 8, 10]
            .iter()
            .map(|c| match page.totals.get(*c).unwrap() {
                Cell::Number { value } => *value,
                other => money(other),
            })
            .collect::<Vec<_>>(),
        vec![10, 12_800, 720, 0]
    );
    // Paid and due filters.
    let paid = report(
        &w,
        ReportId::R9,
        ReportFilters {
            status: Some(PaidFilter::Paid),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    let due = report(
        &w,
        ReportId::R9,
        ReportFilters {
            status: Some(PaidFilter::Due),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!((paid.total_count, due.total_count), (1, 0));
    // Only a پختہ کھاتہ account can be the party.
    assert_eq!(
        rejected(&w, ReportId::R9, party(w.akram), 0, 0),
        vec![(ErrorCode::ErrOutOfRange, "filters.party".to_owned())]
    );
}

/// The one-time register agrees with the register of the one-time screen, line by line: paid or
/// open, amount, commission and qty (the report works the open amount out the same way).
fn assert_r9_equals_the_screen(w: &support::World) {
    let screen = bikri::register(&w.env.ctx, &PukhtaRegisterInput::default()).unwrap();
    let page = report(w, ReportId::R9, ReportFilters::default(), 0, 0);
    assert_eq!(page.rows.len(), screen.len());
    for (row, line) in page.rows.iter().zip(&screen) {
        let cells = &row.cells;
        assert_eq!(number(&cells[6]), line.qty);
        assert_eq!(money(&cells[7]), line.amount.rupees());
        assert_eq!(money(&cells[8]), line.commission.rupees());
        assert_eq!(money(&cells[10]), line.open.rupees());
        let state = label(&cells[9]);
        let expected = if line.cancelled {
            "report-status-cancelled"
        } else if line.paid {
            "report-status-paid"
        } else {
            "report-status-due"
        };
        assert_eq!(state, expected);
        let (_, _, key) = party_name(&cells[3]);
        assert_eq!(key.as_deref(), Some(line.person_key.as_str()));
    }
}

#[test]
fn r9_works_per_person_on_credit_and_agrees_with_the_one_time_screen() {
    use mandi_services::core::bikri::{LineChargeMethod, PaymentType, PukhtaInput};
    // E2E-21: شاہد 1 × 1,000 and ناصر 2 × 1,000 on credit, then a receipt of 1,000 for شاہد.
    let w = worked_day();
    let line = |key: &str, qty: &str, ur: &str, en: &str, phone: &str| {
        let mut l = support::sale(key, w.pukhta_buyer, qty, "1000");
        l.pukhta = Some(PukhtaInput {
            name_ur: ur.into(),
            name_en: en.into(),
            phone: phone.into(),
        });
        l.payment_type = Some(PaymentType::Credit);
        l.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
        l.seller_comm = support::typed(LineChargeMethod::Percent, "0");
        l.laga = support::typed(LineChargeMethod::PerUnit, "0");
        l
    };
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "3");
    lot.sales = vec![
        line("s-1", "1", "شاہد", "Shahid", "0300-0000000"),
        line("s-2", "2", "ناصر", "Nasir", "0300-1111111"),
    ];
    save(&w, &support::draft(vec![lot]));
    let shahid = mandi_services::core::text::person_key("شاہد", "0300-0000000");
    let nasir = mandi_services::core::text::person_key("ناصر", "0300-1111111");

    let before = report(&w, ReportId::R9, ReportFilters::default(), 0, 0);
    assert_eq!(before.total_count, 2);
    assert_eq!(before.totals.get(10).map(money), Some(3_000));
    assert_r9_equals_the_screen(&w);

    let mut input = receipt(w.pukhta_buyer, "1,000");
    input.person_key = Some(shahid.clone());
    vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();

    let page = report(&w, ReportId::R9, ReportFilters::default(), 0, 0);
    assert_eq!(page.total_count, 2);
    let by_person = |key: &str| -> (String, i64) {
        let row = page
            .rows
            .iter()
            .find(|r| party_name(&r.cells[3]).2.as_deref() == Some(key))
            .unwrap();
        (label(&row.cells[9]), money(&row.cells[10]))
    };
    // شاہد 0 and ناصر 2,000 still due: the payment never settles the other person's bill.
    assert_eq!(by_person(&shahid), ("report-status-paid".to_owned(), 0));
    assert_eq!(by_person(&nasir), ("report-status-due".to_owned(), 2_000));
    assert_eq!(page.totals.get(10).map(money), Some(2_000));
    assert_r9_equals_the_screen(&w);

    // The filters: due only; one person.
    let due = report(
        &w,
        ReportId::R9,
        ReportFilters {
            status: Some(PaidFilter::Due),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(due.total_count, 1);
    let one = report(
        &w,
        ReportId::R9,
        ReportFilters {
            party: Some(PartyFilter {
                account_id: w.pukhta_buyer,
                person_key: Some(nasir),
            }),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(one.total_count, 1);
    assert_eq!(one.totals.get(10).map(money), Some(2_000));
}

#[test]
fn r9_lists_a_one_time_seller_with_what_is_still_to_pay_him() {
    use mandi_services::core::bikri::PukhtaInput;
    // A one-time seller's lot is paid on the spot when the net is paid as نقد (PRD FR-13.10); a
    // part left unpaid is open until a payment settles it.
    let w = worked_day();
    let mut lot = support::bare_lot(&w, "lot-1", w.pukhta_seller, w.banana, "10");
    lot.pukhta = Some(PukhtaInput {
        name_ur: "نذیر".into(),
        name_en: "Nazir".into(),
        phone: "0300-2222222".into(),
    });
    lot.cash_paid = Some("4,000".into());
    lot.sales = vec![support::sale("s-1", w.akram, "10", "1000")];
    save(&w, &support::draft(vec![lot]));
    let page = report(&w, ReportId::R9, ReportFilters::default(), 0, 0);
    // Lines: the lot of the one-time seller. Gross 10,000; the seller's commission is 6%.
    let seller_rows: Vec<_> = page
        .rows
        .iter()
        .filter(|r| label(&r.cells[2]) == "report-side-seller")
        .collect();
    assert_eq!(seller_rows.len(), 1);
    let cells = &seller_rows[0].cells;
    assert_eq!(money(&cells[7]), 10_000);
    assert_eq!(money(&cells[8]), 600);
    // Net after 600 commission, 4,000 نقد and the lot's other charges is still to pay.
    assert_eq!(label(&cells[9]), "report-status-due");
    assert!(money(&cells[10]) > 0);
    assert_r9_equals_the_screen(&w);
}

// --- Page, query plans and writes ----------------------------------------------------------------

#[test]
fn the_sales_queries_use_the_date_index_and_never_scan_the_sale_lines() {
    // Code Standards B18: with a date range, the register is found through the date index of the
    // journal entries, the lots through the date index of the بکری, the facts through their own
    // date index and the income by the indexes of the journal lines. Without the range a scan
    // is the only way, so only the dated queries are checked.
    use mandi_reports::reports::plans::*;
    let w = full_day();
    let conn = raw(&w.dir);
    let plan = |query: (String, Vec<mandi_services::db::rusqlite::types::Value>)| -> String {
        let mut statement = conn
            .prepare(&format!("EXPLAIN QUERY PLAN {}", query.0))
            .unwrap();
        let rows = statement
            .query_map(
                mandi_services::db::rusqlite::params_from_iter(query.1.iter()),
                |r| r.get::<_, String>(3),
            )
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        rows.join(" | ")
    };
    let day = Some("2026-10-10");
    let register = plan(r2_page(day, day));
    assert!(
        register.contains("idx_journal_entries_entry_date"),
        "{register}"
    );
    assert!(!register.contains("SCAN s "), "{register}");
    let totals = plan(r2_totals(day, day));
    assert!(
        totals.contains("idx_journal_entries_entry_date"),
        "{totals}"
    );
    let lots = plan(r6_page(day, day));
    assert!(lots.contains("idx_bikri_entries_date"), "{lots}");
    let facts = plan(r7(day, day));
    assert!(facts.contains("idx_analytics_facts_date"), "{facts}");
    let income = plan(r28_page(day, day));
    assert!(!income.contains("SCAN "), "{income}");
    // No filter at all is still a valid query.
    assert!(!plan(r2_page(None, None)).is_empty());
}

#[test]
fn a_sales_report_writes_nothing() {
    let w = full_day();
    let count = |sql: &str| -> i64 { raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap() };
    let rows = || {
        (
            count("SELECT COUNT(*) FROM journal_entries"),
            count("SELECT COUNT(*) FROM journal_lines"),
            count("SELECT COUNT(*) FROM audit_log"),
            count("SELECT COUNT(*) FROM bill_allocations"),
            count("SELECT COUNT(*) FROM bikri_sales"),
            count("SELECT COUNT(*) FROM analytics_facts"),
        )
    };
    let start = rows();
    for id in [
        ReportId::R2,
        ReportId::R3,
        ReportId::R4,
        ReportId::R6,
        ReportId::R7,
        ReportId::R8,
        ReportId::R9,
        ReportId::R27,
    ] {
        report(&w, id, ReportFilters::default(), 0, 0);
    }
    assert_eq!(rows(), start);
}

// --- Print data ----------------------------------------------------------------------------------

#[test]
fn every_sales_report_prints_every_row_in_both_languages_with_the_same_totals() {
    let w = full_day();
    for id in [
        ReportId::R2,
        ReportId::R3,
        ReportId::R4,
        ReportId::R6,
        ReportId::R7,
        ReportId::R8,
        ReportId::R9,
        ReportId::R27,
    ] {
        let page = report(&w, id, ReportFilters::default(), 0, 0);
        for language in [Language::Ur, Language::En] {
            let data = print_data(
                w.env.ctx.db(),
                &ReportPrintInput {
                    id,
                    filters: ReportFilters::default(),
                    language,
                    paper: None,
                },
            )
            .unwrap();
            assert_eq!(data.rows.len(), page.rows.len(), "{id:?}");
            assert_eq!(data.columns.len(), page.columns.len(), "{id:?}");
            assert_eq!(data.totals.len(), page.columns.len(), "{id:?}");
            assert_eq!(data.title_id, page.title_id);
            for row in &data.rows {
                assert_eq!(row.cells.len(), page.columns.len(), "{id:?}");
            }
        }
    }
    // The English R2 prints the buyer with his short code and name, and the totals as text.
    let en = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id: ReportId::R2,
            filters: ReportFilters::default(),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    match en.rows.first().unwrap().cells.get(2).unwrap() {
        PrintCell::Party { party } => {
            assert_eq!(
                (party.short_code.as_deref(), party.name.as_str()),
                (Some("1"), "Akram Toll Tax")
            );
        }
        other => panic!("{other:?}"),
    }
    let text = |cell: &PrintCell| match cell {
        PrintCell::Money { text } | PrintCell::Number { text } => text.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!(text(en.totals.get(5).unwrap()), "60");
    assert_eq!(text(en.totals.get(7).unwrap()), "41,400");
    assert_eq!(text(en.totals.get(8).unwrap()), "44,364");
    // The header of the print lists the filters it was given, dates as DD-MM-YYYY.
    let filtered = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id: ReportId::R4,
            filters: ReportFilters {
                from: "10-10-2026".into(),
                to: "10-10-2026".into(),
                status: None,
                ..ReportFilters::default()
            },
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    let facts: Vec<(String, String)> = filtered
        .filters
        .iter()
        .map(|f| (f.label_id.clone(), f.value.clone()))
        .collect();
    assert_eq!(
        facts,
        vec![
            ("report-filter-from".to_owned(), "10-10-2026".to_owned()),
            ("report-filter-to".to_owned(), "10-10-2026".to_owned())
        ]
    );
    // R9 says which state it was asked for.
    let due = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id: ReportId::R9,
            filters: ReportFilters {
                status: Some(PaidFilter::Due),
                ..ReportFilters::default()
            },
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert!(
        due.filters
            .iter()
            .any(|f| f.label_id == "report-filter-due" && f.value.is_empty())
    );
}

#[test]
fn the_filters_a_sales_report_does_not_take_are_refused_together() {
    let w = full_day();
    // R8 takes dates and اڈا only.
    let got = rejected(
        &w,
        ReportId::R8,
        ReportFilters {
            seller_id: Some(w.rahim),
            item_id: Some(w.banana),
            status: Some(PaidFilter::Paid),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(
        got,
        vec![
            (ErrorCode::ErrOutOfRange, "filters.seller_id".to_owned()),
            (ErrorCode::ErrOutOfRange, "filters.item_id".to_owned()),
            (ErrorCode::ErrOutOfRange, "filters.status".to_owned()),
        ]
    );
    // An unknown seller and an unknown item are not found.
    let got = rejected(
        &w,
        ReportId::R2,
        ReportFilters {
            seller_id: Some(999_999),
            item_id: Some(999_999),
            adda_id: Some(999_999),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(
        got,
        vec![
            (ErrorCode::ErrNotFound, "filters.seller_id".to_owned()),
            (ErrorCode::ErrNotFound, "filters.item_id".to_owned()),
            (ErrorCode::ErrNotFound, "filters.adda_id".to_owned()),
        ]
    );
}
