//! R35 پارٹی کی روزانہ پرچی (the party day sheet) of plan d5-late, round 11 (PRD 19.2, FR-19.12,
//! FR-19.10, FR-19.11, E2E-43), on the PRD worked day.
//!
//! The expected numbers are the PRD's (E2E-43, 10-10-26): اکرم 0, 12,880, 0, 10,000, 2,880; ارشد
//! 0, 18,684, 0, 0, 18,684; شاہد 0, 12,800, 12,800, 0, 0; the buyers' remaining total 21,564 =
//! R20; رحیم 15,000 بنام, 24,316, 5,000, 4,316, 0; کریم 0, 10,160, 0, 0, 10,160 جمع.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
mod xlsx_support;

use common::*;
use mandi_reports::reports::{
    Cell, PartyFilter, ReportFilters, ReportId, ReportPage, ReportPrintInput, Row, RowKind,
    print_data,
};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{CancelInput, EditOpenInput, EditSaveInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
use mandi_services::core::vouchers::{AdvanceInput, VoucherMode};
use mandi_services::{accounts, bikri, ledger, vouchers};
use reports_world::*;
use support::World;

fn sheet(day: &str, party: Option<PartyFilter>) -> ReportFilters {
    ReportFilters {
        as_of: day.into(),
        party,
        ..ReportFilters::default()
    }
}

fn account(id: i64) -> Option<PartyFilter> {
    Some(PartyFilter {
        account_id: id,
        person_key: None,
    })
}

/// A balance cell as a signed number: a debit balance is positive.
fn signed(cell: &Cell) -> i64 {
    match cell {
        Cell::Balance { paisa, side } => match side {
            BalanceSide::Cr => -paisa.rupees(),
            _ => paisa.rupees(),
        },
        other => panic!("not a balance: {other:?}"),
    }
}

fn text(cell: &Cell) -> String {
    match cell {
        Cell::Text { text } => text.clone(),
        Cell::Empty => String::new(),
        other => panic!("not text: {other:?}"),
    }
}

/// previous, debit, credit, remaining of a table row (a party row or a heading).
fn figures(row: &Row) -> (i64, i64, i64, i64) {
    (
        signed(&row.cells[2]),
        money(&row.cells[3]),
        money(&row.cells[4]),
        signed(&row.cells[5]),
    )
}

/// The English name of the party of a row.
fn name_en(row: &Row) -> String {
    match &row.cells[1] {
        Cell::Party { party } => party.name_en.clone(),
        other => panic!("not a party: {other:?}"),
    }
}

fn summary_of(page: &ReportPage, label: &str) -> Option<Cell> {
    page.summary
        .iter()
        .find(|l| l.label_id == label)
        .map(|l| l.cell.clone())
}

/// The key of a one-time person, by his English name.
fn person_key(w: &World, name_en: &str) -> String {
    w.env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row(
                "SELECT pukhta_person_key FROM bikri_sales WHERE pukhta_name_en = ?1 LIMIT 1",
                [name_en],
                |r| r.get(0),
            )
        })
        .unwrap()
}

#[test]
fn the_catalogue_has_r35_with_the_day_and_the_party() {
    let entry = mandi_reports::reports::catalog()
        .into_iter()
        .find(|e| e.id == ReportId::R35)
        .unwrap();
    assert_eq!(
        entry.filters,
        vec![
            mandi_reports::reports::FilterKind::AsOf,
            mandi_reports::reports::FilterKind::Party
        ]
    );
    assert_eq!(entry.title_id, "report-r35-title");
    assert!(!entry.party_required);
}

#[test]
fn e2e_43_the_table_of_all_parties_for_the_worked_day() {
    let w = full_day();
    let page = report(&w, ReportId::R35, sheet("10-10-2026", None), 0, 0);
    // Buyers first, then sellers; each with a heading row that carries its totals.
    let kinds: Vec<RowKind> = page.rows.iter().map(|r| r.kind).collect();
    assert_eq!(
        kinds,
        vec![
            RowKind::Group,
            RowKind::Normal,
            RowKind::Normal,
            RowKind::Normal,
            RowKind::Group,
            RowKind::Normal,
            RowKind::Normal
        ]
    );
    // Sorted by Urdu name: ارشد، اکرم، شاہد; then رحیم، کریم.
    let names: Vec<String> = page
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .map(name_en)
        .collect();
    assert_eq!(
        names,
        vec![
            "Arshad Multani",
            "Akram Toll Tax",
            "Shahid",
            "Rahim",
            "Karim"
        ]
    );
    let rows: Vec<(i64, i64, i64, i64)> = page
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .map(figures)
        .collect();
    assert_eq!(
        rows,
        vec![
            (0, 18_684, 0, 18_684),
            (0, 12_880, 10_000, 2_880),
            (0, 12_800, 12_800, 0),
            // رحیم: 15,000 بنام; نقد 5,000 and the payment 4,316 are debits; the credit is 24,316.
            (15_000, 9_316, 24_316, 0),
            // کریم: 10,160 جمع.
            (0, 0, 10_160, -10_160),
        ]
    );
    // Every row holds previous + debit − credit = remaining.
    for row in page.rows.iter() {
        let (previous, debit, credit, remaining) = figures(row);
        assert_eq!(previous + debit - credit, remaining);
    }
    // The buyers' heading: 0 + 44,364 − 22,800 = 21,564, the total of R20 (FR-19.9).
    assert_eq!(figures(&page.rows[0]), (0, 44_364, 22_800, 21_564));
    let r20 = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(money(&r20.totals[3]), 21_564);
    // The sellers' heading. The totals row is the buyers' table, not buyers plus sellers: what is
    // owed to the shop and what the shop owes are not added together (E2E-43).
    assert_eq!(figures(&page.rows[4]), (15_000, 9_316, 34_476, -10_160));
    assert_eq!(
        (
            signed(&page.totals[2]),
            money(&page.totals[3]),
            money(&page.totals[4]),
            signed(&page.totals[5])
        ),
        (0, 44_364, 22_800, 21_564)
    );
    // The blank column for the collector's note, and the short code of each party.
    assert!(page.rows.iter().all(|r| matches!(&r.cells[6], Cell::Empty)));
    assert_eq!(text(&page.rows[1].cells[0]), "2");
    assert_eq!(page.filters.as_of.as_deref(), Some("2026-10-10"));
}

#[test]
fn e2e_43_the_buyers_slip_of_akram() {
    let w = full_day();
    let page = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.akram)),
        0,
        0,
    );
    // One line: 20 پیٹی کیلا at 600, 12,880 with commission and لاگا.
    assert_eq!(page.rows.len(), 1);
    let cells = &page.rows[0].cells;
    assert!(matches!(&cells[0], Cell::Number { value: 20 }));
    match &cells[1] {
        Cell::Names { names } => {
            assert_eq!(names.ur, "پیٹی کیلا");
            assert_eq!(names.en, "Box Banana");
        }
        other => panic!("not names: {other:?}"),
    }
    assert_eq!((money(&cells[2]), money(&cells[3])), (600, 12_880));
    assert!(matches!(&page.totals[0], Cell::Number { value: 20 }));
    assert_eq!(money(&page.totals[3]), 12_880);
    // The footer: previous 0, bought 12,880, paid 10,000, remaining 2,880 بنام.
    assert!(matches!(
        summary_of(&page, "report-r35-total-qty"),
        Some(Cell::Number { value: 20 })
    ));
    assert_eq!(
        signed(&summary_of(&page, "report-r35-previous").unwrap()),
        0
    );
    assert_eq!(
        money(&summary_of(&page, "report-r35-bought").unwrap()),
        12_880
    );
    assert_eq!(
        money(&summary_of(&page, "report-r35-paid").unwrap()),
        10_000
    );
    assert_eq!(
        signed(&summary_of(&page, "report-r35-remaining").unwrap()),
        2_880
    );
    // No cash-on-the-spot line and no other-debits line when they are 0.
    assert!(summary_of(&page, "report-r35-on-spot").is_none());
    assert!(summary_of(&page, "report-r35-other-debits").is_none());
    assert_eq!(page.filters.party.as_ref().unwrap().code, "03040001");
}

#[test]
fn a_one_time_buyer_who_paid_on_the_spot_has_nothing_to_collect() {
    let w = full_day();
    let key = person_key(&w, "Shahid");
    let page = report(
        &w,
        ReportId::R35,
        sheet(
            "10-10-2026",
            Some(PartyFilter {
                account_id: w.pukhta_buyer,
                person_key: Some(key),
            }),
        ),
        0,
        0,
    );
    assert_eq!(page.rows.len(), 1);
    assert_eq!(
        (money(&page.rows[0].cells[2]), money(&page.rows[0].cells[3])),
        (1_200, 12_800)
    );
    assert_eq!(
        money(&summary_of(&page, "report-r35-bought").unwrap()),
        12_800
    );
    // Paid 12,800 on the spot (the cash part of a نقدی line); "paid that day" is left blank.
    assert_eq!(
        money(&summary_of(&page, "report-r35-on-spot").unwrap()),
        12_800
    );
    assert!(matches!(
        summary_of(&page, "report-r35-paid"),
        Some(Cell::Empty)
    ));
    assert_eq!(
        signed(&summary_of(&page, "report-r35-remaining").unwrap()),
        0
    );
}

#[test]
fn e2e_43_the_sellers_slips_of_rahim_and_karim() {
    let w = full_day();
    let rahim = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.rahim)),
        0,
        0,
    );
    // His lot: 50 پیٹی کیلا, credit 24,316.
    assert_eq!(rahim.rows.len(), 1);
    let cells = &rahim.rows[0].cells;
    assert!(matches!(&cells[0], Cell::Number { value: 50 }));
    assert_eq!(money(&cells[3]), 24_316);
    let line = |page: &ReportPage, label: &str| {
        summary_of(page, label).map_or(0, |c| match c {
            Cell::Balance { .. } => signed(&c),
            Cell::Empty => 0,
            other => money(&other),
        })
    };
    // Previous 15,000 بنام, sold 24,316, نقد 5,000, paid that day 4,316, remaining 0.
    assert_eq!(line(&rahim, "report-r35-previous"), 15_000);
    assert_eq!(line(&rahim, "report-r35-sold"), 24_316);
    assert_eq!(line(&rahim, "report-r35-cash-given"), 5_000);
    assert_eq!(line(&rahim, "report-r35-paid"), 4_316);
    assert_eq!(line(&rahim, "report-r35-remaining"), 0);

    let karim = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.karim)),
        0,
        0,
    );
    assert_eq!(line(&karim, "report-r35-previous"), 0);
    assert_eq!(line(&karim, "report-r35-sold"), 10_160);
    assert!(summary_of(&karim, "report-r35-cash-given").is_none());
    assert!(matches!(
        summary_of(&karim, "report-r35-paid"),
        Some(Cell::Empty)
    ));
    // 10,160 جمع.
    assert_eq!(line(&karim, "report-r35-remaining"), -10_160);
}

#[test]
fn another_day_has_its_own_previous_balance_and_a_quiet_party_has_no_row() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    // On 11-10-26 nothing happened: the parties who still owe or are owed show with their
    // balance, those at 0 do not, and nobody has a debit or a credit.
    let page = report(&w, ReportId::R35, sheet("11-10-2026", None), 0, 0);
    let names: Vec<String> = page
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .map(name_en)
        .collect();
    assert_eq!(names, vec!["Arshad Multani", "Akram Toll Tax", "Karim"]);
    for row in page.rows.iter().filter(|r| r.kind == RowKind::Normal) {
        let (previous, debit, credit, remaining) = figures(row);
        assert_eq!((debit, credit), (0, 0));
        assert_eq!(previous, remaining);
    }
    // Before the first posting nothing is listed.
    let early = report(&w, ReportId::R35, sheet("01-08-2026", None), 0, 0);
    assert!(early.rows.is_empty());
    assert!(
        early
            .totals
            .iter()
            .all(|c| matches!(c, Cell::Empty | Cell::Balance { .. } | Cell::Money { .. }))
    );
}

#[test]
fn a_same_day_cancel_leaves_the_bill_and_its_reversal_out_of_the_day() {
    // بکری 2 is cancelled on the day it was made: شاہد bought nothing, so he has no row and his
    // slip is empty; the balances of the others do not change.
    let w = worked_day();
    save(&w, &support::event1(&w));
    let second = save(&w, &event2(&w));
    let detail = bikri::get(&w.env.ctx, second.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: second.bikri_id,
            version: detail.version,
            reason: "غلط".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let page = report(&w, ReportId::R35, sheet("10-10-2026", None), 0, 0);
    let names: Vec<String> = page
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .map(name_en)
        .collect();
    assert_eq!(names, vec!["Arshad Multani", "Akram Toll Tax", "Rahim"]);
    // Buyers: 12,880 + 18,684 charged, nothing paid, 31,564 remaining.
    assert_eq!(figures(&page.rows[0]), (0, 31_564, 0, 31_564));
    for row in page.rows.iter() {
        let (previous, debit, credit, remaining) = figures(row);
        assert_eq!(previous + debit - credit, remaining);
    }
    // R20 agrees.
    let r20 = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(money(&r20.totals[3]), 31_564);
}

#[test]
fn a_party_that_is_not_a_buyer_or_a_seller_and_a_one_time_account_without_his_person_are_refused() {
    let w = full_day();
    let cash = support::id_of(&w.env, "01010001");
    assert_eq!(
        rejected(&w, ReportId::R35, sheet("10-10-2026", account(cash)), 0, 0),
        vec![(
            ErrorCode::ErrOutOfRange,
            "filters.party.account_id".to_owned()
        )]
    );
    assert_eq!(
        rejected(
            &w,
            ReportId::R35,
            sheet("10-10-2026", account(w.pukhta_buyer)),
            0,
            0
        ),
        vec![(
            ErrorCode::ErrRequired,
            "filters.party.person_key".to_owned()
        )]
    );
    // The date filter is read like any other.
    assert_eq!(
        rejected(&w, ReportId::R35, sheet("31-02-2026", None), 0, 0),
        vec![(ErrorCode::ErrDateInvalid, "filters.as_of".to_owned())]
    );
}

#[test]
fn the_table_pages_with_totals_over_the_whole_filter_and_prints_in_both_languages() {
    let w = full_day();
    let all = report(&w, ReportId::R35, sheet("10-10-2026", None), 0, 0);
    let second = report(&w, ReportId::R35, sheet("10-10-2026", None), 2, 2);
    assert_eq!(second.rows.len(), 2);
    assert_eq!(second.total_count, all.total_count);
    assert_eq!(second.totals, all.totals);
    for language in [Language::Ur, Language::En] {
        let data = print_data(
            w.env.ctx.db(),
            &ReportPrintInput {
                id: ReportId::R35,
                filters: sheet("10-10-2026", None),
                language,
                paper: None,
            },
        )
        .unwrap();
        assert_eq!(data.rows.len(), 7, "{language:?}");
        assert_eq!(data.report, ReportId::R35);
    }
    // The slip prints its footer lines too.
    let slip = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id: ReportId::R35,
            filters: sheet("10-10-2026", account(w.akram)),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(slip.rows.len(), 1);
    assert_eq!(slip.summary.len(), 5);
}

/// The balance of an account as the services ledger reads it, signed (a debit is positive).
fn book(w: &World, account: i64) -> i64 {
    let book = ledger::account_balance(&w.env.ctx, account).unwrap();
    match book.side {
        BalanceSide::Cr => -book.amount_paisa.rupees(),
        _ => book.amount_paisa.rupees(),
    }
}

/// One row of the table, by the English name of its party.
fn row_of<'a>(page: &'a ReportPage, name: &str) -> &'a Row {
    page.rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .find(|r| name_en(r) == name)
        .unwrap_or_else(|| panic!("no row for {name}"))
}

#[test]
fn the_collection_line_of_the_table_is_the_total_of_r20() {
    let w = full_day();
    let page = report(&w, ReportId::R35, sheet("10-10-2026", None), 0, 0);
    let r20 = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(
        money(&summary_of(&page, "report-r35-collection").unwrap()),
        21_564
    );
    assert_eq!(money(&r20.totals[3]), 21_564);
}

#[test]
fn e2e_13_a_bill_edited_on_its_own_day_counts_once_and_the_remaining_is_the_ledger() {
    // اکرم's rate 600 to 610 on the day of the sale: he is charged 13,092 once (the first
    // posting and its reversal are both left out of the day), pays 10,000, owes 3,092.
    let w = worked_day();
    let one = save(&w, &support::event1(&w));
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: one.bikri_id,
            entry_id: one.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("ریٹ کی درستی".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    let page = report(&w, ReportId::R35, sheet("10-10-2026", None), 0, 0);
    assert_eq!(
        figures(row_of(&page, "Akram Toll Tax")),
        (0, 13_092, 10_000, 3_092)
    );
    assert_eq!(
        figures(row_of(&page, "Arshad Multani")),
        (0, 18_684, 0, 18_684)
    );
    assert_eq!(
        signed(&row_of(&page, "Akram Toll Tax").cells[5]),
        book(&w, w.akram)
    );
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.akram)),
        0,
        0,
    );
    assert_eq!(slip.rows.len(), 1);
    assert_eq!(
        (money(&slip.rows[0].cells[2]), money(&slip.rows[0].cells[3])),
        (610, 13_092)
    );
    assert_eq!(
        money(&summary_of(&slip, "report-r35-bought").unwrap()),
        13_092
    );
    assert_eq!(
        signed(&summary_of(&slip, "report-r35-remaining").unwrap()),
        3_092
    );
}

#[test]
fn e2e_32_cash_lent_to_a_buyer_is_another_debit_of_his_day() {
    // A payment of 3,000 to اکرم (reason نقد ادھار) is one of his bills: the slip shows it as
    // "other debits", the day debit is 12,880 + 3,000, and he owes 5,880.
    let w = full_day();
    let mut lent = payment(w.akram, "3,000");
    lent.reason = "نقد ادھار".into();
    vouchers::save_payment(&w.env.ctx, &w.owner, &lent).unwrap();
    let page = report(&w, ReportId::R35, sheet("10-10-2026", None), 0, 0);
    assert_eq!(
        figures(row_of(&page, "Akram Toll Tax")),
        (0, 15_880, 10_000, 5_880)
    );
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.akram)),
        0,
        0,
    );
    assert_eq!(
        money(&summary_of(&slip, "report-r35-bought").unwrap()),
        12_880
    );
    assert_eq!(
        money(&summary_of(&slip, "report-r35-other-debits").unwrap()),
        3_000
    );
    assert_eq!(
        signed(&summary_of(&slip, "report-r35-remaining").unwrap()),
        5_880
    );
    // The buyers' total is the total of R20 and the dashboard tile: 24,564.
    assert_eq!(
        money(&summary_of(&page, "report-r35-collection").unwrap()),
        24_564
    );
    let r20 = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(money(&r20.totals[3]), 24_564);
}

/// A lot of کریم's whose deductions are more than its sales (PRD FR-12.7): 10 پیٹی سیب at 100 to
/// اکرم, freight 3,000, 500 cash given, so the net is −2,180 and the seller owes 2,680.
fn negative_net_world() -> World {
    let w = worked_day();
    let mut lot = support::lot(&w, "lot-n", w.karim, w.apple, "10");
    lot.cash_paid = Some("500".into());
    let lot = support::with_sales(lot, vec![support::sale("s-1", w.akram, "10", "100")]);
    save(&w, &support::draft(vec![lot]));
    w
}

#[test]
fn a_lot_with_a_negative_net_is_a_negative_sale_and_not_cash_the_seller_got() {
    let w = negative_net_world();
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.karim)),
        0,
        0,
    );
    let line = |label: &str| {
        summary_of(&slip, label).map_or(0, |c| match c {
            Cell::Balance { .. } => signed(&c),
            Cell::Empty => 0,
            other => money(&other),
        })
    };
    // Sold −2,180 (the lot's net), cash given 500, nothing paid later: 2,680 بنام.
    assert_eq!(line("report-r35-sold"), -2_180);
    assert_eq!(line("report-r35-cash-given"), 500);
    assert_eq!(line("report-r35-paid"), 0);
    assert_eq!(line("report-r35-remaining"), -(-2_680));
    assert_eq!(line("report-r35-remaining"), book(&w, w.karim));
    // The rows add up to what was sold.
    let rows_total: i64 = slip.rows.iter().map(|r| money(&r.cells[3])).sum();
    assert_eq!(rows_total, -2_180);
    assert_eq!(money(&slip.totals[3]), -2_180);
    // In the table: previous 0, debit 2,680, credit 0, remaining 2,680 بنام.
    let page = report(&w, ReportId::R35, sheet("10-10-2026", None), 0, 0);
    assert_eq!(figures(row_of(&page, "Karim")), (0, 2_680, 0, 2_680));
}

#[test]
fn a_buyer_who_pays_more_than_he_owes_is_in_credit_and_is_not_taken_off_the_collection() {
    // اکرم buys 1,140 (1,000 + 60 + 80) and pays 1,640: 500 جمع. The sheet shows him in credit;
    // the collection line stays what the buyers owe, as R20 does.
    let w = negative_net_world();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "1,640")).unwrap();
    let page = report(&w, ReportId::R35, sheet("10-10-2026", None), 0, 0);
    assert_eq!(
        figures(row_of(&page, "Akram Toll Tax")),
        (0, 1_140, 1_640, -500)
    );
    assert_eq!(book(&w, w.akram), -500);
    let r20 = report(&w, ReportId::R20, ReportFilters::default(), 0, 0);
    assert_eq!(
        money(&summary_of(&page, "report-r35-collection").unwrap()),
        money(&r20.totals[3])
    );
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.akram)),
        0,
        0,
    );
    assert_eq!(
        signed(&summary_of(&slip, "report-r35-remaining").unwrap()),
        -500
    );
}

#[test]
fn an_opening_set_and_changed_on_the_sheet_day_is_one_other_debit() {
    // An opening debit of 5,000 on اکرم dated the sheet day, then changed to 7,000: the first
    // opening and its reversal are left out of the day, the new one is his other debit.
    let w = worked_day();
    let set_opening = |dr: &str| {
        let version = accounts::get(&w.env.ctx, w.akram).unwrap().version;
        accounts::openings_save(
            &w.env.ctx,
            &w.owner,
            &OpeningsInput {
                lines: vec![OpeningLineInput {
                    account_id: w.akram,
                    version,
                    opening_date: "10-10-2026".into(),
                    opening_dr: dr.into(),
                    opening_cr: String::new(),
                    opening_is_advance: false,
                }],
            },
        )
        .unwrap();
    };
    set_opening("5,000");
    set_opening("7,000");
    let page = report(&w, ReportId::R35, sheet("10-10-2026", None), 0, 0);
    assert_eq!(
        figures(row_of(&page, "Akram Toll Tax")),
        (0, 7_000, 0, 7_000)
    );
    assert_eq!(book(&w, w.akram), 7_000);
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.akram)),
        0,
        0,
    );
    assert_eq!(
        money(&summary_of(&slip, "report-r35-other-debits").unwrap()),
        7_000
    );
    assert_eq!(
        signed(&summary_of(&slip, "report-r35-remaining").unwrap()),
        7_000
    );
}

fn line_of(page: &ReportPage, label: &str) -> i64 {
    summary_of(page, label).map_or(0, |c| match c {
        Cell::Balance { .. } => signed(&c),
        Cell::Empty => 0,
        other => money(&other),
    })
}

#[test]
fn a_credit_that_is_not_a_receipt_is_not_paid_that_day() {
    // A جنرل credit of 500 on اکرم: the collector's "paid" stays the 10,000 receipt, and the 500
    // is its own line; the remaining is still the ledger (2,880 - 500).
    let w = full_day();
    let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
        account_id: Some(account),
        person_key: None,
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    };
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &GeneralInput {
            date: "10-10-2026".into(),
            narration: Some("credit".into()),
            lines: vec![
                line(support::id_of(&w.env, "01010001"), "500", ""),
                line(w.akram, "", "500"),
            ],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.akram)),
        0,
        0,
    );
    assert_eq!(line_of(&slip, "report-r35-paid"), 10_000);
    assert_eq!(line_of(&slip, "report-r35-other-credits"), 500);
    assert_eq!(line_of(&slip, "report-r35-remaining"), 2_380);
    assert_eq!(line_of(&slip, "report-r35-remaining"), book(&w, w.akram));
    let page = report(&w, ReportId::R35, sheet("10-10-2026", None), 0, 0);
    assert_eq!(
        figures(row_of(&page, "Akram Toll Tax")),
        (0, 12_880, 10_500, 2_380)
    );
}

#[test]
fn an_advance_given_to_a_seller_is_not_a_payment() {
    // رحیم is given a پیشگی of 2,000 on the day: "paid" stays the 4,316 payment, the 2,000 is
    // another debit, and he is 2,000 بنام.
    let w = full_day();
    vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &AdvanceInput {
            confirm_cash_below_zero: false,
            date: "10-10-2026".into(),
            seller_account_id: w.rahim,
            amount: "2,000".into(),
            mode: VoucherMode::Cash,
            bank_account_id: None,
            cheque_no: String::new(),
            cheque_date: String::new(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .unwrap();
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.rahim)),
        0,
        0,
    );
    assert_eq!(line_of(&slip, "report-r35-paid"), 4_316);
    assert_eq!(line_of(&slip, "report-r35-other-debits"), 2_000);
    assert_eq!(line_of(&slip, "report-r35-cash-given"), 5_000);
    // 15,000 Dr + 5,000 + 4,316 + 2,000 - 24,316 = 2,000 Dr.
    assert_eq!(line_of(&slip, "report-r35-remaining"), 2_000);
    assert_eq!(line_of(&slip, "report-r35-remaining"), book(&w, w.rahim));
}

#[test]
fn a_posting_with_a_negative_lot_and_a_good_lot_keeps_the_item_of_the_good_one() {
    // Two lots of کریم in one بکری: سیب 10 at 600 (credit 2,520) and کیلا 10 at 100 whose net is
    // negative (-2,180, a debit). The credit line belongs to the سیب lot, so its item and نگ show.
    let w = worked_day();
    let mut apple = support::lot(&w, "lot-a", w.karim, w.apple, "10");
    apple.cash_paid = None;
    let apple = support::with_sales(apple, vec![support::sale("s-1", w.akram, "10", "600")]);
    let mut banana = support::lot(&w, "lot-b", w.karim, w.banana, "10");
    banana.cash_paid = None;
    let banana = support::with_sales(banana, vec![support::sale("s-2", w.akram, "10", "100")]);
    save(&w, &support::draft(vec![apple, banana]));
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.karim)),
        0,
        0,
    );
    assert_eq!(slip.rows.len(), 2);
    assert!(matches!(&slip.rows[0].cells[0], Cell::Number { value: 10 }));
    match &slip.rows[0].cells[1] {
        Cell::Names { names } => assert_eq!(names.en, "Box Apple"),
        other => panic!("not names: {other:?}"),
    }
    assert_eq!(money(&slip.rows[0].cells[3]), 2_520);
    assert_eq!(money(&slip.rows[1].cells[3]), -2_180);
    assert!(matches!(&slip.totals[0], Cell::Number { value: 10 }));
    assert_eq!(money(&slip.totals[3]), 340);
    assert_eq!(line_of(&slip, "report-r35-sold"), 340);
    // 340 جمع.
    assert_eq!(line_of(&slip, "report-r35-remaining"), -340);
    assert_eq!(line_of(&slip, "report-r35-remaining"), book(&w, w.karim));
}

#[test]
fn a_person_on_an_account_that_has_no_persons_is_refused() {
    let w = full_day();
    let bad = ReportFilters {
        as_of: "10-10-2026".into(),
        party: Some(PartyFilter {
            account_id: w.akram,
            person_key: Some("x".into()),
        }),
        ..ReportFilters::default()
    };
    assert_eq!(
        rejected(&w, ReportId::R35, bad, 0, 0),
        vec![(
            ErrorCode::ErrOutOfRange,
            "filters.party.person_key".to_owned()
        )]
    );
}
