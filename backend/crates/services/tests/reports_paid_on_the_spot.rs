//! Plan 65 T2 (D8 Feature 4, PRD FR-12.44, FR-19.8, FR-19.12, R3, R4, R5, R13, R35): the reports that
//! used the ادھاری / نقدی word follow the money paid on the spot.
//!
//! One بکری on 10-10-26: رحیم's 50 کیلا, اکرم 20 at 600 (12,000 + 720 + 160 = 12,880) who pays 3,000 on
//! the spot, and ارشد 30 at 580 (17,400 + 1,044 + 240 = 18,684) who pays all of it. The numbers are
//! worked by hand from the PRD and never changed to suit the code:
//!
//! | Buyer | Value | Buyer total | Paid | Left |
//! |---|---|---|---|---|
//! | اکرم | 12,000 | 12,880 | 3,000 | 9,880 |
//! | ارشد | 17,400 | 18,684 | 18,684 | 0 |
//! | Total | 29,400 | 31,564 | 21,684 | 9,880 |
//!
//! R2 has both lines. R3 (paid above 0) has both, R4 (something left) has اکرم's only. R5 has both
//! buyers, in the Urdu order of the names (ارشد first), and its totals are those of R2.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::reports::{
    Cell, DetailPart, ReportFilters, ReportId, ReportPage, ReportPrintInput, buyer_bills_print_data,
};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::auth::Language;
use mandi_services::core::error::ErrorCode;
use reports_world::*;
use support::World;

const DAY: &str = "10-10-2026";

/// The one بکری: اکرم pays 3,000 of 12,880, ارشد pays all 18,684.
fn day_with_part_payment() -> World {
    let w = worked_day();
    let mut akram = support::sale("s-1", w.akram, "20", "600");
    akram.paid = Some("3,000".into());
    let mut arshad = support::sale("s-2", w.arshad, "30", "580");
    arshad.paid = Some("18,684".into());
    let lot = support::with_sales(
        support::bare_lot(&w, "lot-1", w.rahim, w.banana, "50"),
        vec![akram, arshad],
    );
    save(&w, &support::draft(vec![lot]));
    w
}

fn day_filters() -> ReportFilters {
    ReportFilters {
        from: DAY.into(),
        to: DAY.into(),
        ..ReportFilters::default()
    }
}

/// The position of a column by its key.
fn at(page: &ReportPage, key: &str) -> usize {
    page.columns
        .iter()
        .position(|c| c.key == key)
        .unwrap_or_else(|| panic!("no column {key}"))
}

/// (buyer_total, paid, left) of a row of R3 or R4, in rupees.
fn totals_of_row(page: &ReportPage, row: usize) -> (i64, i64, i64) {
    let cells = &page.rows[row].cells;
    (
        money(&cells[at(page, "buyer_total")]),
        money(&cells[at(page, "paid")]),
        money(&cells[at(page, "left")]),
    )
}

#[test]
fn r3_has_every_line_with_money_paid_and_r4_every_line_with_something_left() {
    let w = day_with_part_payment();
    let r2 = report(&w, ReportId::R2, day_filters(), 0, 0);
    let r3 = report(&w, ReportId::R3, day_filters(), 0, 0);
    let r4 = report(&w, ReportId::R4, day_filters(), 0, 0);

    // R2 is as it was: no paid and left columns, both lines, the sums of the PRD (E2E-36 style).
    assert_eq!(r2.rows.len(), 2);
    assert!(
        r2.columns
            .iter()
            .all(|c| c.key != "paid" && c.key != "left")
    );
    assert_eq!(money(&r2.totals[at(&r2, "value")]), 29_400);
    assert_eq!(money(&r2.totals[at(&r2, "buyer_total")]), 31_564);

    // R3: both lines were paid something. Its old columns and its new last two columns.
    assert_eq!(r3.rows.len(), 2);
    let keys: Vec<&str> = r3.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(
        &keys[keys.len() - 2..],
        ["paid", "left"],
        "paid and left are the last two"
    );
    assert_eq!(totals_of_row(&r3, 0), (12_880, 3_000, 9_880));
    assert_eq!(totals_of_row(&r3, 1), (18_684, 18_684, 0));
    assert_eq!(money(&r3.totals[at(&r3, "buyer_total")]), 31_564);
    assert_eq!(money(&r3.totals[at(&r3, "paid")]), 21_684);
    assert_eq!(money(&r3.totals[at(&r3, "left")]), 9_880);

    // R4: only اکرم's line has something left. A part-paid line is on R3 and on R4 (plan 60 P12).
    assert_eq!(r4.rows.len(), 1);
    assert_eq!(totals_of_row(&r4, 0), (12_880, 3_000, 9_880));
    assert_eq!(money(&r4.totals[at(&r4, "paid")]), 3_000);
    assert_eq!(money(&r4.totals[at(&r4, "left")]), 9_880);
    // The totals row is one query over the whole filter, and the rows of a page add up to it.
    assert_eq!(r3.total_count, 2);
    assert_eq!(r4.total_count, 1);
}

#[test]
fn r5_lists_every_buyer_of_the_day_and_its_totals_are_those_of_r2() {
    let w = day_with_part_payment();
    let r2 = report(&w, ReportId::R2, day_filters(), 0, 0);
    let r5 = report(&w, ReportId::R5, day_filters(), 0, 0);
    let keys: Vec<&str> = r5.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "serial",
            "buyer",
            "detail",
            "value",
            "buyer_total",
            "paid",
            "left"
        ]
    );
    // Both buyers, ارشد first (the Urdu order); ارشد paid in full and is a row with 0 left (D8 Q1).
    assert_eq!(r5.rows.len(), 2);
    let row = |i: usize| -> (i64, i64, i64, i64) {
        let c = &r5.rows[i].cells;
        (
            money(&c[at(&r5, "value")]),
            money(&c[at(&r5, "buyer_total")]),
            money(&c[at(&r5, "paid")]),
            money(&c[at(&r5, "left")]),
        )
    };
    assert_eq!(code_of(&r5.rows[0].cells[1]), "0021");
    assert_eq!(row(0), (17_400, 18_684, 18_684, 0));
    assert_eq!(code_of(&r5.rows[1].cells[1]), "0020");
    assert_eq!(row(1), (12_000, 12_880, 3_000, 9_880));
    // The totals: value and buyer total are R2's; paid and left are the sums of the rows.
    assert_eq!(
        money(&r5.totals[at(&r5, "value")]),
        money(&r2.totals[at(&r2, "value")])
    );
    assert_eq!(
        money(&r5.totals[at(&r5, "buyer_total")]),
        money(&r2.totals[at(&r2, "buyer_total")])
    );
    assert_eq!(money(&r5.totals[at(&r5, "paid")]), 21_684);
    assert_eq!(money(&r5.totals[at(&r5, "left")]), 9_880);
    // paid + left = buyer total, in every row and in the totals.
    for i in 0..2 {
        let (_, total, paid, left) = row(i);
        assert_eq!(paid + left, total);
    }
    assert_eq!(21_684 + 9_880, 31_564);
}

#[test]
fn a_buyers_ledger_shows_the_money_paid_on_the_spot_after_its_sale_line() {
    // PRD FR-19.8: the credit line that follows the sale line is marked موقع پر جمع (the detail part
    // `paid_on_spot` with the بکری number). اکرم's ledger: 12,880 بنام, then 3,000 جمع, balance 9,880.
    let w = day_with_part_payment();
    let r13 = report(&w, ReportId::R13, party(w.akram), 0, 0);
    let debit = at(&r13, "debit");
    let credit = at(&r13, "credit");
    let detail = at(&r13, "detail");
    let balance_at = at(&r13, "balance");
    let lines: Vec<&mandi_reports::reports::Row> = r13
        .rows
        .iter()
        .filter(|r| matches!(&r.cells[detail], Cell::Detail { .. }))
        .collect();
    assert_eq!(lines.len(), 2, "the sale line and its paid credit");
    // The sale line.
    assert_eq!(money(&lines[0].cells[debit]), 12_880);
    let Cell::Detail { parts } = &lines[0].cells[detail] else {
        panic!("not a detail");
    };
    assert!(matches!(&parts[0], DetailPart::Sale { .. }));
    // The money paid on the spot, marked, with the بکری number.
    assert_eq!(money(&lines[1].cells[credit]), 3_000);
    let Cell::Detail { parts } = &lines[1].cells[detail] else {
        panic!("not a detail");
    };
    assert_eq!(parts.len(), 1);
    assert!(
        matches!(&parts[0], DetailPart::PaidOnSpot { bikri_number: 1 }),
        "{parts:?}"
    );
    assert_eq!(
        balance(&lines[1].cells[balance_at]),
        (BalanceSide::Dr, 9_880)
    );
    // ارشد paid it all: the same two lines, and a balance of 0 after the credit.
    let r13 = report(&w, ReportId::R13, party(w.arshad), 0, 0);
    let detail = at(&r13, "detail");
    let paid_lines = r13
        .rows
        .iter()
        .filter(|r| {
            matches!(&r.cells[detail], Cell::Detail { parts }
                if matches!(&parts[0], DetailPart::PaidOnSpot { .. }))
        })
        .count();
    assert_eq!(paid_lines, 1);
}

#[test]
fn the_party_day_table_counts_the_money_paid_on_the_spot_as_paid_today() {
    // PRD FR-19.12: previous, bought that day (debit), paid that day (credit), remaining. اکرم:
    // 0, 12,880, 3,000, 9,880; ارشد: 0, 18,684, 18,684, 0. The buyers' heading: 0, 31,564, 21,684,
    // 9,880, which is the total of R20.
    let w = day_with_part_payment();
    let page = report(
        &w,
        ReportId::R35,
        ReportFilters {
            as_of: DAY.into(),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    let signed = |cell: &Cell| -> i64 {
        match cell {
            Cell::Balance { paisa, side } => match side {
                BalanceSide::Cr => -paisa.rupees(),
                _ => paisa.rupees(),
            },
            other => panic!("not a balance: {other:?}"),
        }
    };
    let figures = |i: usize| -> (i64, i64, i64, i64) {
        let c = &page.rows[i].cells;
        (signed(&c[2]), money(&c[3]), money(&c[4]), signed(&c[5]))
    };
    // The buyers' heading, then ارشد and اکرم in the Urdu order.
    assert_eq!(figures(0), (0, 31_564, 21_684, 9_880));
    assert_eq!(figures(1), (0, 18_684, 18_684, 0));
    assert_eq!(figures(2), (0, 12_880, 3_000, 9_880));
    let r20 = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(money(&r20.totals[3]), 9_880);
}

#[test]
fn r5_takes_one_bikri_and_no_other_register_does() {
    // D8 Feature 8 (FR-12.12): the R5 button next to the seller-bill print prints "this بکری only".
    // R5 takes the filter `bikri_id`; its slips, totals and print header follow; R2 to R4 refuse it.
    let w = day_with_part_payment();
    let first = first_bikri_id(&w);
    // A second بکری of the same day: کریم's 10 سیب to اکرم at 1,200 (12,000 + 720 + 80 = 12,800), of
    // which اکرم pays 800 on the spot.
    let mut line = support::sale("s-3", w.akram, "10", "1200");
    line.paid = Some("800".into());
    let lot = support::with_sales(
        support::bare_lot(&w, "lot-2", w.karim, w.apple, "10"),
        vec![line],
    );
    let second = save(&w, &support::draft(vec![lot])).bikri_id;
    let only = |id: i64| ReportFilters {
        bikri_id: Some(id),
        ..day_filters()
    };
    let figures = |page: &ReportPage| -> Vec<(i64, i64, i64, i64)> {
        page.rows
            .iter()
            .map(|r| {
                (
                    money(&r.cells[at(page, "value")]),
                    money(&r.cells[at(page, "buyer_total")]),
                    money(&r.cells[at(page, "paid")]),
                    money(&r.cells[at(page, "left")]),
                )
            })
            .collect()
    };
    // The first بکری: ارشد (paid in full) and اکرم (3,000 of 12,880).
    let one = report(&w, ReportId::R5, only(first), 0, 0);
    assert_eq!(
        figures(&one),
        [(17_400, 18_684, 18_684, 0), (12_000, 12_880, 3_000, 9_880)]
    );
    assert_eq!(money(&one.totals[at(&one, "buyer_total")]), 31_564);
    // The second: اکرم only, 800 of 12,800.
    let two = report(&w, ReportId::R5, only(second), 0, 0);
    assert_eq!(figures(&two), [(12_000, 12_800, 800, 12_000)]);
    // The whole day: اکرم has both of his lines in one row, 12,880 + 12,800 = 25,680 with 3,800
    // paid and 21,880 left.
    let day = report(&w, ReportId::R5, day_filters(), 0, 0);
    assert_eq!(
        figures(&day),
        [(17_400, 18_684, 18_684, 0), (24_000, 25_680, 3_800, 21_880)]
    );
    // The slips and the print header follow the filter, and the header names the بکری by its number.
    let print = |filters: ReportFilters| {
        buyer_bills_print_data(
            w.env.ctx.db(),
            &ReportPrintInput {
                group_by_month: false,
                sort: None,
                id: ReportId::R5,
                filters,
                language: Language::En,
                paper: None,
            },
        )
        .unwrap()
    };
    let data = print(only(second));
    assert_eq!(data.slips.len(), 1);
    assert_eq!(
        (data.totals[4].clone(), data.totals[5].clone()),
        (
            mandi_reports::reports::PrintCell::Money { text: "800".into() },
            mandi_reports::reports::PrintCell::Money {
                text: "12,000".into()
            }
        )
    );
    let facts: Vec<(&str, &str)> = data
        .filters
        .iter()
        .map(|f| (f.label_id.as_str(), f.value.as_str()))
        .collect();
    assert!(facts.contains(&("report-filter-bikri", "2")), "{facts:?}");
    // No other register takes it: the filter is refused at its field.
    for id in [ReportId::R2, ReportId::R3, ReportId::R4] {
        assert_eq!(
            rejected(&w, id, only(first), 0, 0),
            vec![(ErrorCode::ErrOutOfRange, "filters.bikri_id".to_owned())],
            "{id:?}"
        );
    }
}

/// The id of the first بکری (number 1).
fn first_bikri_id(w: &World) -> i64 {
    w.env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row("SELECT id FROM bikri_entries WHERE number = 1", [], |r| {
                r.get(0)
            })
        })
        .unwrap()
}
