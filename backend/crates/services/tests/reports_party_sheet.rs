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
use mandi_services::core::bikri::{
    CancelInput, EditOpenInput, EditSaveInput, FreightPaidBy, LaterSaleSaveInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
use mandi_services::core::printing::Paper;
use mandi_services::core::vouchers::AdvanceInput;
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

/// The ids of the summary lines of a page, in order.
fn summary_ids(page: &ReportPage) -> Vec<&str> {
    page.summary.iter().map(|l| l.label_id.as_str()).collect()
}

/// The English names of the item and the packing of a goods row.
fn goods_names(row: &Row) -> (String, String) {
    let pick = |cell: &Cell| match cell {
        Cell::Names { names } => names.en.clone(),
        other => panic!("not names: {other:?}"),
    };
    (pick(&row.cells[0]), pick(&row.cells[1]))
}

/// qty, rate and amount of a goods row, in whole rupees.
fn goods_figures(row: &Row) -> (i64, i64, i64) {
    let qty = match &row.cells[2] {
        Cell::Number { value } => *value,
        other => panic!("not a number: {other:?}"),
    };
    (qty, money(&row.cells[3]), money(&row.cells[4]))
}

fn summary_of(page: &ReportPage, label: &str) -> Option<Cell> {
    page.summary
        .iter()
        .find(|l| l.label_id == label)
        .map(|l| l.cell.clone())
}

/// The key of a چکتا buyer, by his English name.
fn person_key(w: &World, name_en: &str) -> String {
    w.env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row(
                "SELECT chukta_person_key FROM bikri_sales WHERE chukta_name_en = ?1 LIMIT 1",
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
            mandi_reports::reports::FilterKind::Party,
            // One collector (plan 55 §11.1).
            mandi_reports::reports::FilterKind::Collector
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
    // The sellers' heading. The page has no pinned totals row: what is owed to the shop (the
    // buyers' heading row and the collection line) and what the shop owes (the sellers' lines)
    // are never added together (E2E-43).
    assert_eq!(figures(&page.rows[4]), (15_000, 9_316, 34_476, -10_160));
    assert_eq!(page.totals.len(), page.columns.len());
    assert!(page.totals.iter().all(|c| matches!(c, Cell::Empty)));
    // The blank column for the collector's note, and the short code of each party.
    assert!(page.rows.iter().all(|r| matches!(&r.cells[6], Cell::Empty)));
    assert_eq!(text(&page.rows[1].cells[0]), "1");
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
    // One line: کیلا, پیٹی, 20 at 600 = 12,000 (the goods value; the commission is below).
    assert_eq!(page.rows.len(), 1);
    assert_eq!(
        goods_names(&page.rows[0]),
        ("Banana".to_owned(), "Box".to_owned())
    );
    assert_eq!(goods_figures(&page.rows[0]), (20, 600, 12_000));
    assert_eq!(page.rows[0].kind, RowKind::Normal);
    // The slip has no totals row (FR-19.12).
    assert!(page.totals.iter().all(|c| matches!(c, Cell::Empty)));
    // The lines, in the PRD order, with no other-entries line when it is 0.
    assert_eq!(
        summary_ids(&page),
        vec![
            "report-r35-goods",
            "report-r35-commission",
            "report-r35-laga",
            "report-r35-today-total",
            "report-r35-paid-today",
            "report-r35-today-left",
            "report-r35-previous",
            "report-r35-new-balance",
            "report-r35-total-qty",
        ]
    );
    let get = |id: &str| line_of(&page, id);
    assert_eq!(get("report-r35-goods"), 12_000);
    assert_eq!(get("report-r35-commission"), 720);
    assert_eq!(get("report-r35-laga"), 160);
    assert_eq!(get("report-r35-today-total"), 12_880);
    assert_eq!(get("report-r35-paid-today"), 10_000);
    assert_eq!(get("report-r35-today-left"), 2_880);
    assert_eq!(get("report-r35-previous"), 0);
    assert_eq!(get("report-r35-new-balance"), 2_880);
    // The balances carry their side: 2,880 بنام.
    assert!(matches!(
        summary_of(&page, "report-r35-new-balance"),
        Some(Cell::Balance {
            side: BalanceSide::Dr,
            ..
        })
    ));
    assert!(matches!(
        summary_of(&page, "report-r35-total-qty"),
        Some(Cell::Number { value: 20 })
    ));
    assert_eq!(page.filters.party.as_ref().unwrap().code, "0020");
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
                account_id: w.chukta_khata,
                person_key: Some(key),
            }),
        ),
        0,
        0,
    );
    assert_eq!(page.rows.len(), 1);
    assert_eq!(goods_figures(&page.rows[0]), (10, 1_200, 12_000));
    assert_eq!(line_of(&page, "report-r35-goods"), 12_000);
    assert_eq!(line_of(&page, "report-r35-commission"), 720);
    assert_eq!(line_of(&page, "report-r35-laga"), 80);
    assert_eq!(line_of(&page, "report-r35-today-total"), 12_800);
    // Paid 12,800 on the spot (the cash part of a نقدی line) is paid today.
    assert_eq!(line_of(&page, "report-r35-paid-today"), 12_800);
    assert_eq!(line_of(&page, "report-r35-today-left"), 0);
    assert_eq!(line_of(&page, "report-r35-new-balance"), 0);
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
    // His two lines of کیلا, پیٹی and their item total: 50 نگ, 29,400.
    assert_eq!(rahim.rows.len(), 3);
    assert_eq!(goods_figures(&rahim.rows[0]), (20, 600, 12_000));
    assert_eq!(goods_figures(&rahim.rows[1]), (30, 580, 17_400));
    assert_eq!(rahim.rows[2].kind, RowKind::ItemTotal);
    assert!(
        matches!(&rahim.rows[2].cells[0], Cell::Message { id, .. } if id == "report-r35-item-total")
    );
    assert!(matches!(
        &rahim.rows[2].cells[2],
        Cell::Number { value: 50 }
    ));
    assert_eq!(money(&rahim.rows[2].cells[4]), 29_400);
    assert_eq!(
        summary_ids(&rahim),
        vec![
            "report-r35-gross",
            "report-r35-commission",
            "report-r35-freight",
            "report-r35-labour",
            "report-r35-munshiana",
            "report-r35-phone",
            "report-r35-net-sale",
            "report-r35-paid-today",
            "report-r35-today-left",
            "report-r35-previous",
            "report-r35-new-balance",
        ]
    );
    let get = |id: &str| line_of(&rahim, id);
    assert_eq!(get("report-r35-gross"), 29_400);
    assert_eq!(get("report-r35-commission"), 1_764);
    assert_eq!(get("report-r35-freight"), 3_000);
    assert_eq!(get("report-r35-labour"), 250);
    assert_eq!(get("report-r35-munshiana"), 50);
    assert_eq!(get("report-r35-phone"), 20);
    // The net sale is a credit: 24,316 جمع (a balance, because it can be negative).
    assert_eq!(get("report-r35-net-sale"), -24_316);
    // نقد 5,000 + payment 4,316; left 15,000 جمع; previous 15,000 بنام; new 0.
    assert_eq!(get("report-r35-paid-today"), 9_316);
    assert_eq!(get("report-r35-today-left"), -15_000);
    assert_eq!(get("report-r35-previous"), 15_000);
    assert_eq!(get("report-r35-new-balance"), 0);

    let karim = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.karim)),
        0,
        0,
    );
    assert_eq!(line_of(&karim, "report-r35-previous"), 0);
    assert_eq!(line_of(&karim, "report-r35-gross"), 12_000);
    assert_eq!(line_of(&karim, "report-r35-net-sale"), -10_160);
    // Nothing was paid: the collector's line is left blank.
    assert!(matches!(
        summary_of(&karim, "report-r35-paid-today"),
        Some(Cell::Empty)
    ));
    // 10,160 جمع.
    assert_eq!(line_of(&karim, "report-r35-new-balance"), -10_160);
    // A seller's slip has no total نگ line (PRD FR-19.12).
    assert!(summary_of(&karim, "report-r35-total-qty").is_none());
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
            sheet("10-10-2026", account(w.chukta_khata)),
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
                group_by_month: false,
                sort: None,
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
            group_by_month: false,
            sort: None,
            id: ReportId::R35,
            filters: sheet("10-10-2026", account(w.akram)),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(slip.rows.len(), 1);
    assert_eq!(slip.summary.len(), 9);
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
    // The sellers' table has its own total in the footer: credit 34,476 (رحیم 24,316 and کریم
    // 10,160), debit 9,316 (the نقد and the payment), remaining 10,160 جمع.
    assert_eq!(
        money(&summary_of(&page, "report-r35-sellers-credit").unwrap()),
        34_476
    );
    assert_eq!(
        money(&summary_of(&page, "report-r35-sellers-debit").unwrap()),
        9_316
    );
    assert_eq!(
        signed(&summary_of(&page, "report-r35-sellers-remaining").unwrap()),
        -10_160
    );
}

#[test]
fn a_table_with_no_sellers_has_no_sellers_total() {
    // Before the first posting nobody is listed: no sellers' section, so no sellers' total, and
    // the collection line is 0.
    let w = worked_day();
    let page = report(&w, ReportId::R35, sheet("01-08-2026", None), 0, 0);
    assert!(page.rows.is_empty());
    assert!(summary_of(&page, "report-r35-sellers-credit").is_none());
    assert_eq!(
        money(&summary_of(&page, "report-r35-collection").unwrap()),
        0
    );
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
    assert_eq!(goods_figures(&slip.rows[0]), (20, 610, 12_200));
    assert_eq!(line_of(&slip, "report-r35-commission"), 732);
    assert_eq!(line_of(&slip, "report-r35-today-total"), 13_092);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), 3_092);
    // رحیم's side of the same edit: the posting and its reversal are both left out of the day
    // (money agent edge test 1): gross 29,600, commission 1,776, charges 3,320, net 24,504, نقد
    // 5,000, left 19,504 جمع, previous 15,000 بنام, new 4,504 جمع (no payment in this story).
    let rahim = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.rahim)),
        0,
        0,
    );
    assert_eq!(line_of(&rahim, "report-r35-gross"), 29_600);
    assert_eq!(line_of(&rahim, "report-r35-commission"), 1_776);
    assert_eq!(line_of(&rahim, "report-r35-net-sale"), -24_504);
    assert_eq!(line_of(&rahim, "report-r35-paid-today"), 5_000);
    assert_eq!(line_of(&rahim, "report-r35-today-left"), -19_504);
    assert_eq!(line_of(&rahim, "report-r35-new-balance"), -4_504);
    assert_eq!(line_of(&rahim, "report-r35-new-balance"), book(&w, w.rahim));
}

#[test]
fn e2e_32_cash_lent_to_a_buyer_is_another_debit_of_his_day() {
    // A payment of 3,000 to اکرم (reason نقد ادھار) is one of his bills: the slip shows it as
    // "other entries", the day debit is 12,880 + 3,000, and he owes 5,880.
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
    assert_eq!(line_of(&slip, "report-r35-today-total"), 12_880);
    assert_eq!(line_of(&slip, "report-r35-other"), 3_000);
    assert_eq!(line_of(&slip, "report-r35-paid-today"), 10_000);
    assert_eq!(line_of(&slip, "report-r35-today-left"), 5_880);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), 5_880);
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
    // The net sale is −2,180 (a debit: بنام), نقد 500 is paid today: left 2,680 بنام.
    assert_eq!(line_of(&slip, "report-r35-net-sale"), 2_180);
    assert!(matches!(
        summary_of(&slip, "report-r35-net-sale"),
        Some(Cell::Balance {
            side: BalanceSide::Dr,
            ..
        })
    ));
    assert_eq!(line_of(&slip, "report-r35-paid-today"), 500);
    assert_eq!(line_of(&slip, "report-r35-today-left"), 2_680);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), book(&w, w.karim));
    assert_eq!(line_of(&slip, "report-r35-new-balance"), 2_680);
    // The goods row is the sale (1,000); the deductions on the lines below make the net.
    assert_eq!(slip.rows.len(), 1);
    assert_eq!(goods_figures(&slip.rows[0]), (10, 100, 1_000));
    let deductions: i64 = slip
        .summary
        .iter()
        .filter(|l| {
            matches!(
                l.label_id.as_str(),
                "report-r35-commission"
                    | "report-r35-freight"
                    | "report-r35-labour"
                    | "report-r35-munshiana"
                    | "report-r35-phone"
                    | "report-r35-store-rent"
            )
        })
        .map(|l| money(&l.cell))
        .sum();
    assert_eq!(1_000 - deductions, -2_180);
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
    // Left and new are credits: 500 جمع.
    assert_eq!(line_of(&slip, "report-r35-today-left"), -500);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), -500);
    assert!(matches!(
        summary_of(&slip, "report-r35-new-balance"),
        Some(Cell::Balance {
            side: BalanceSide::Cr,
            ..
        })
    ));
}

#[test]
fn an_opening_set_and_changed_on_the_sheet_day_is_one_other_debit() {
    // An opening debit of 5,000 on اکرم dated the sheet day, then changed to 7,000: the first
    // opening and its reversal are left out of the day, the new one is his other entry.
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
    assert_eq!(line_of(&slip, "report-r35-other"), 7_000);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), 7_000);
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
    // A جنرل credit of 500 on اکرم: "paid today" stays the 10,000 receipt, and the 500 is an
    // other entry (a credit); the new balance is still the ledger (2,880 − 500).
    let w = full_day();
    let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
        account_id: Some(account),
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
    assert_eq!(line_of(&slip, "report-r35-paid-today"), 10_000);
    assert_eq!(line_of(&slip, "report-r35-other"), -500);
    assert_eq!(line_of(&slip, "report-r35-today-left"), 2_380);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), 2_380);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), book(&w, w.akram));
    let page = report(&w, ReportId::R35, sheet("10-10-2026", None), 0, 0);
    assert_eq!(
        figures(row_of(&page, "Akram Toll Tax")),
        (0, 12_880, 10_500, 2_380)
    );
}

#[test]
fn an_advance_given_to_a_seller_is_not_a_payment() {
    // رحیم is given a پیشگی of 2,000 on the day: "paid today" stays 5,000 نقد + 4,316 payment,
    // the 2,000 is an other entry (a debit), and he is 2,000 بنام.
    let w = full_day();
    vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &AdvanceInput {
            confirm_cash_below_zero: false,
            date: "10-10-2026".into(),
            seller_account_id: w.rahim,
            amount: "2,000".into(),
            removed: Default::default(),
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
    assert_eq!(line_of(&slip, "report-r35-paid-today"), 9_316);
    assert_eq!(line_of(&slip, "report-r35-other"), 2_000);
    // 15,000 Dr + 5,000 + 4,316 + 2,000 − 24,316 = 2,000 Dr.
    assert_eq!(line_of(&slip, "report-r35-today-left"), -13_000);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), 2_000);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), book(&w, w.rahim));
}

#[test]
fn a_posting_with_a_negative_lot_and_a_good_lot_lists_both_items() {
    // Two lots of کریم in one بکری: سیب 10 at 600 (credit 2,520) and کیلا 10 at 100 whose net is
    // negative (−2,180, a debit). The goods table lists both lines, one for each item.
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
    assert_eq!(goods_names(&slip.rows[0]).0, "Apple");
    assert_eq!(goods_figures(&slip.rows[0]), (10, 600, 6_000));
    assert_eq!(goods_names(&slip.rows[1]).0, "Banana");
    assert_eq!(goods_figures(&slip.rows[1]), (10, 100, 1_000));
    assert_eq!(line_of(&slip, "report-r35-gross"), 7_000);
    // 340 جمع.
    assert_eq!(line_of(&slip, "report-r35-net-sale"), -340);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), -340);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), book(&w, w.karim));
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

#[test]
fn a_lot_with_nothing_sold_in_its_first_posting_still_charges_the_seller() {
    // Money agent edge test 11: کریم's lot of 10 سیب has no sale line, but its charges (freight
    // 3,000, labour 5 × 10, munshiana 50, phone 20) and the 5,000 نقد are posted on him. The
    // slip has no goods rows, a net sale of −3,120 (a debit) and left 8,120 بنام.
    let w = worked_day();
    let karim_lot = support::lot(&w, "lot-k", w.karim, w.apple, "10");
    let rahim_lot = support::with_sales(
        support::lot(&w, "lot-r", w.rahim, w.banana, "10"),
        vec![support::sale("s-1", w.akram, "10", "600")],
    );
    save(&w, &support::draft(vec![rahim_lot, karim_lot]));
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.karim)),
        0,
        0,
    );
    assert!(slip.rows.is_empty());
    assert_eq!(line_of(&slip, "report-r35-gross"), 0);
    assert_eq!(line_of(&slip, "report-r35-freight"), 3_000);
    assert_eq!(line_of(&slip, "report-r35-labour"), 50);
    assert_eq!(line_of(&slip, "report-r35-munshiana"), 50);
    assert_eq!(line_of(&slip, "report-r35-phone"), 20);
    assert_eq!(line_of(&slip, "report-r35-net-sale"), 3_120);
    assert_eq!(line_of(&slip, "report-r35-paid-today"), 5_000);
    assert_eq!(line_of(&slip, "report-r35-today-left"), 8_120);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), 8_120);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), book(&w, w.karim));
}

#[test]
fn a_seller_who_returns_advance_money_has_it_as_an_other_credit() {
    // E2E-31: رحیم returns 10,000 of his 15,000 advance in cash on 10-10-26 (a وصولی on his
    // account), then Event 1. Net sale 24,316, other 10,000 جمع, نقد 5,000: left 29,316 جمع,
    // previous 15,000 بنام, new 14,316 جمع.
    let w = worked_day();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.rahim, "10,000")).unwrap();
    save(&w, &support::event1(&w));
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.rahim)),
        0,
        0,
    );
    assert_eq!(line_of(&slip, "report-r35-net-sale"), -24_316);
    assert_eq!(line_of(&slip, "report-r35-other"), -10_000);
    assert_eq!(line_of(&slip, "report-r35-paid-today"), 5_000);
    assert_eq!(line_of(&slip, "report-r35-today-left"), -29_316);
    assert_eq!(line_of(&slip, "report-r35-previous"), 15_000);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), -14_316);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), book(&w, w.rahim));
}

#[test]
fn the_own_goods_account_has_no_slip() {
    // اپنا مال never holds a balance (its lots credit the trip's partners): a slip for it is
    // refused, not printed empty.
    let w = full_day();
    let own = support::id_of(&w.env, "030119998");
    assert_eq!(
        rejected(&w, ReportId::R35, sheet("10-10-2026", account(own)), 0, 0),
        vec![(
            ErrorCode::ErrOutOfRange,
            "filters.party.account_id".to_owned()
        )]
    );
}

#[test]
fn a_party_with_nothing_on_the_day_has_an_empty_slip_with_his_balance() {
    // On a quiet day the slip has no goods and the previous balance is the new one.
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    let slip = report(
        &w,
        ReportId::R35,
        sheet("11-10-2026", account(w.akram)),
        0,
        0,
    );
    assert!(slip.rows.is_empty());
    assert_eq!(line_of(&slip, "report-r35-today-total"), 0);
    assert_eq!(line_of(&slip, "report-r35-today-left"), 0);
    assert_eq!(line_of(&slip, "report-r35-previous"), 2_880);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), 2_880);
}

#[test]
fn the_80_mm_roll_is_for_the_slip_of_one_party_only() {
    // Plan 42 / lead: R35 with a party takes `mm80` and echoes it; the all-parties table of R35
    // and R34 are refused with ERR_OUT_OF_RANGE on `paper`. A4 and A5 stay allowed for all.
    let w = full_day();
    let print = |id: ReportId, filters: ReportFilters, paper: Paper| {
        print_data(
            w.env.ctx.db(),
            &ReportPrintInput {
                group_by_month: false,
                sort: None,
                id,
                filters,
                language: Language::En,
                paper: Some(paper),
            },
        )
    };
    for who in [w.akram, w.rahim] {
        let data = print(
            ReportId::R35,
            sheet("10-10-2026", account(who)),
            Paper::Mm80,
        )
        .unwrap();
        assert_eq!(data.paper, Paper::Mm80);
    }
    let refused = |result: Result<mandi_reports::reports::ReportPrintData, _>| match result {
        Err(mandi_reports::error::ReportsError::Rejected(items)) => items
            .iter()
            .map(|i| (i.code, i.field.clone()))
            .collect::<Vec<_>>(),
        other => panic!("not refused: {other:?}"),
    };
    let want = vec![(ErrorCode::ErrOutOfRange, "paper".to_owned())];
    assert_eq!(
        refused(print(ReportId::R35, sheet("10-10-2026", None), Paper::Mm80)),
        want
    );
    let day = ReportFilters {
        from: "10-10-2026".into(),
        to: "10-10-2026".into(),
        ..ReportFilters::default()
    };
    assert_eq!(
        refused(print(ReportId::R34, day.clone(), Paper::Mm80)),
        want
    );
    // The papers a table does take.
    for paper in [Paper::A4, Paper::A5] {
        assert_eq!(
            print(ReportId::R34, day.clone(), paper).unwrap().paper,
            paper
        );
        assert_eq!(
            print(ReportId::R35, sheet("10-10-2026", None), paper)
                .unwrap()
                .paper,
            paper
        );
    }
}

fn on_r34(w: &World, day: &str) -> ReportPage {
    report(
        w,
        ReportId::R34,
        ReportFilters {
            from: day.into(),
            to: day.into(),
            ..ReportFilters::default()
        },
        0,
        0,
    )
}

fn r34_line(page: &ReportPage, id: &str) -> i64 {
    let row = page
        .rows
        .iter()
        .find(|r| matches!(&r.cells[1], Cell::Label { id: x } if x == id))
        .unwrap_or_else(|| panic!("no row {id}"));
    match &row.cells[3] {
        Cell::Balance { .. } => signed(&row.cells[3]),
        other => money(other),
    }
}

/// The count column of a row of R34 (the نگ of the day for `report-r34-qty`).
fn r34_count(page: &ReportPage, id: &str) -> i64 {
    let row = page
        .rows
        .iter()
        .find(|r| matches!(&r.cells[1], Cell::Label { id: x } if x == id))
        .unwrap_or_else(|| panic!("no row {id}"));
    match &row.cells[2] {
        Cell::Number { value } => *value,
        other => panic!("not a number: {other:?}"),
    }
}

#[test]
fn a_same_day_cancel_leaves_the_cancelled_bill_out_of_r34_and_the_slips() {
    // E2E-15 shape: بکری 2 is cancelled the day it was made. شاہد and کریم have no row in the
    // tables and empty slips; the commission lines and the buyers' and sellers' lines count
    // only بکری 1.
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
    let page = on_r34(&w, "2026-10-10");
    assert_eq!(r34_count(&page, "report-r34-qty"), 50);
    assert_eq!(r34_line(&page, "report-r34-commission-seller"), 1_764);
    assert_eq!(r34_line(&page, "report-r34-commission-buyer"), 1_764);
    assert_eq!(r34_line(&page, "report-r34-commission-laga"), 400);
    assert_eq!(r34_line(&page, "report-r34-buyers-billed"), 31_564);
    assert_eq!(r34_line(&page, "report-r34-buyers-received"), 0);
    assert_eq!(r34_line(&page, "report-r34-buyers-left"), 31_564);
    assert_eq!(r34_line(&page, "report-r34-sellers-net"), -24_316);
    assert_eq!(r34_line(&page, "report-r34-sellers-paid"), 5_000);
    let buyers = page.tables.iter().find(|t| t.key == "buyers").unwrap();
    let names: Vec<String> = buyers
        .rows
        .iter()
        .map(|r| match &r.cells[1] {
            Cell::Party { party } => party.name_en.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(names, vec!["Arshad Multani", "Akram Toll Tax"]);
    let sellers = page.tables.iter().find(|t| t.key == "sellers").unwrap();
    assert_eq!(sellers.rows.len(), 1);
    // The slips of the cancelled bill's people are empty, with nothing before and nothing after.
    let key = person_key(&w, "Shahid");
    let shahid = report(
        &w,
        ReportId::R35,
        sheet(
            "10-10-2026",
            Some(PartyFilter {
                account_id: w.chukta_khata,
                person_key: Some(key),
            }),
        ),
        0,
        0,
    );
    assert!(shahid.rows.is_empty());
    assert_eq!(line_of(&shahid, "report-r35-today-total"), 0);
    assert_eq!(line_of(&shahid, "report-r35-new-balance"), 0);
    let karim = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.karim)),
        0,
        0,
    );
    assert!(karim.rows.is_empty());
    assert_eq!(line_of(&karim, "report-r35-gross"), 0);
    assert_eq!(line_of(&karim, "report-r35-new-balance"), 0);
}

#[test]
fn freight_the_seller_paid_himself_is_not_a_deduction_on_his_slip() {
    // Money agent edge test: Event 1 with the freight paid by the seller. No freight line;
    // net 29,400 − 1,764 − 250 − 50 − 20 = 27,316 جمع; نقد 5,000 paid; left 22,316 جمع;
    // previous 15,000 بنام; new 7,316 جمع.
    let w = worked_day();
    let mut d = support::event1(&w);
    d.lots[0].freight_paid_by = FreightPaidBy::Seller;
    save(&w, &d);
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.rahim)),
        0,
        0,
    );
    assert!(summary_of(&slip, "report-r35-freight").is_none());
    assert_eq!(line_of(&slip, "report-r35-net-sale"), -27_316);
    assert_eq!(line_of(&slip, "report-r35-paid-today"), 5_000);
    assert_eq!(line_of(&slip, "report-r35-today-left"), -22_316);
    assert_eq!(line_of(&slip, "report-r35-previous"), 15_000);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), -7_316);
    assert_eq!(line_of(&slip, "report-r35-new-balance"), book(&w, w.rahim));
}

#[test]
fn e2e_16_and_16b_a_later_sale_has_no_lot_charges_on_its_slip() {
    // E2E-16: 45 of 50 sold on 10-10-26. E2E-16b: the last 5 sold to ارشد at 560 on 11-10-26.
    let w = worked_day();
    let mut d = support::event1(&w);
    d.lots[0].sales[1].qty = "25".into();
    let saved = save(&w, &d);
    let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
    set_today(&w.env, 2026, 10, 11);
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: "11-10-2026".into(),
            sales: vec![support::sale("l", w.arshad, "5", "560")],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // 10-10: رحیم gross 26,500, commission 1,590, charges 3,320, net 21,590 جمع, نقد 5,000,
    // left 16,590 جمع, previous 15,000 بنام, new 1,590 جمع. ارشد: 14,500 + 870 + 200 = 15,570.
    let rahim = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.rahim)),
        0,
        0,
    );
    assert_eq!(line_of(&rahim, "report-r35-gross"), 26_500);
    assert_eq!(line_of(&rahim, "report-r35-commission"), 1_590);
    assert_eq!(line_of(&rahim, "report-r35-net-sale"), -21_590);
    assert_eq!(line_of(&rahim, "report-r35-today-left"), -16_590);
    assert_eq!(line_of(&rahim, "report-r35-new-balance"), -1_590);
    let arshad = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.arshad)),
        0,
        0,
    );
    assert_eq!(line_of(&arshad, "report-r35-today-total"), 15_570);
    // 11-10: رحیم gross 2,800, only the commission 168 below it (no lot charge again), net 2,632
    // جمع, nothing paid (blank), previous 1,590 جمع, new 4,222 جمع. ارشد 2,800 + 168 + 40 = 3,008.
    let rahim = report(
        &w,
        ReportId::R35,
        sheet("11-10-2026", account(w.rahim)),
        0,
        0,
    );
    assert_eq!(
        summary_ids(&rahim),
        vec![
            "report-r35-gross",
            "report-r35-commission",
            "report-r35-net-sale",
            "report-r35-paid-today",
            "report-r35-today-left",
            "report-r35-previous",
            "report-r35-new-balance",
        ]
    );
    assert_eq!(line_of(&rahim, "report-r35-gross"), 2_800);
    assert_eq!(line_of(&rahim, "report-r35-commission"), 168);
    assert_eq!(line_of(&rahim, "report-r35-net-sale"), -2_632);
    assert!(matches!(
        summary_of(&rahim, "report-r35-paid-today"),
        Some(Cell::Empty)
    ));
    assert_eq!(line_of(&rahim, "report-r35-previous"), -1_590);
    assert_eq!(line_of(&rahim, "report-r35-new-balance"), -4_222);
    let arshad = report(
        &w,
        ReportId::R35,
        sheet("11-10-2026", account(w.arshad)),
        0,
        0,
    );
    assert_eq!(line_of(&arshad, "report-r35-goods"), 2_800);
    assert_eq!(line_of(&arshad, "report-r35-commission"), 168);
    assert_eq!(line_of(&arshad, "report-r35-laga"), 40);
    assert_eq!(line_of(&arshad, "report-r35-today-total"), 3_008);
    assert_eq!(line_of(&arshad, "report-r35-previous"), 15_570);
    assert_eq!(line_of(&arshad, "report-r35-new-balance"), 18_578);
    // R34 for 11-10: 5 نگ; 168 / 168 / 40; billed 3,008; sellers' net 2,632 جمع; paid 0.
    let page = on_r34(&w, "2026-10-11");
    assert_eq!(r34_count(&page, "report-r34-qty"), 5);
    assert_eq!(r34_line(&page, "report-r34-commission-seller"), 168);
    assert_eq!(r34_line(&page, "report-r34-commission-buyer"), 168);
    assert_eq!(r34_line(&page, "report-r34-commission-laga"), 40);
    assert_eq!(r34_line(&page, "report-r34-buyers-billed"), 3_008);
    assert_eq!(r34_line(&page, "report-r34-sellers-net"), -2_632);
    assert_eq!(r34_line(&page, "report-r34-sellers-paid"), 0);
}

/// A lot of grapes (sold by weight) of رحیم: 2 نگ weighing 12.345 kg, sold to اکرم at 100 a kilo.
/// The line is worth 12.345 × 100 = 1,234.5 → 1,235 (C1); commission 6% of 1,235 = 74.1 → 74;
/// لاگا 8 for each of the 2 نگ = 16; اکرم pays 1,235 + 74 + 16 = 1,325.
fn weighed_world() -> World {
    let w = worked_day();
    let mut lot = support::bare_lot(&w, "lot-g", w.rahim, w.weight_fruit, "2");
    lot.weight_received = Some("12.345".into());
    let mut sale = support::sale("s-1", w.akram, "2", "100");
    sale.weight = Some("12.345".into());
    sale.payment_type = Some(mandi_services::core::bikri::PaymentType::Credit);
    lot.sales = vec![sale];
    save(&w, &support::draft(vec![lot]));
    w
}

#[test]
fn a_weighed_line_shows_its_weight_beside_the_rate_per_kilo_and_the_stored_amount() {
    // Plan 42, PRD A73: "12.345 kg @ 100 = 1,235". The row has the weight cell last; the rate is
    // the rate of a kilo and the amount is the stored value, never qty × rate (2 × 100).
    let w = weighed_world();
    for who in [w.akram, w.rahim] {
        let slip = report(&w, ReportId::R35, sheet("10-10-2026", account(who)), 0, 0);
        let keys: Vec<&str> = slip.columns.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(
            keys,
            vec!["item", "packing", "qty", "rate", "amount", "weight"]
        );
        assert_eq!(slip.columns[5].title_id, "report-col-weight");
        assert_eq!(slip.rows.len(), 1);
        assert_eq!(goods_figures(&slip.rows[0]), (2, 100, 1_235));
        assert!(matches!(
            &slip.rows[0].cells[5],
            Cell::Weight { grams: 12_345 }
        ));
        assert_eq!(slip.totals.len(), 6);
    }
    let buyer = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.akram)),
        0,
        0,
    );
    assert_eq!(line_of(&buyer, "report-r35-goods"), 1_235);
    assert_eq!(line_of(&buyer, "report-r35-commission"), 74);
    assert_eq!(line_of(&buyer, "report-r35-laga"), 16);
    assert_eq!(line_of(&buyer, "report-r35-today-total"), 1_325);
    // The print holds the weight as text in kilos, with the digits of the print.
    let print = |language: Language| {
        print_data(
            w.env.ctx.db(),
            &ReportPrintInput {
                group_by_month: false,
                sort: None,
                id: ReportId::R35,
                filters: sheet("10-10-2026", account(w.akram)),
                language,
                paper: None,
            },
        )
        .unwrap()
    };
    for language in [Language::En, Language::Ur] {
        let data = print(language);
        assert!(matches!(
            &data.rows[0].cells[5],
            mandi_reports::reports::PrintCell::Weight { text } if text == "12.345"
        ));
    }
    // A slip with no weighed line has no weight column (the worked day's slips are unchanged).
    let plain = full_day();
    let slip = report(
        &plain,
        ReportId::R35,
        sheet("10-10-2026", account(plain.akram)),
        0,
        0,
    );
    assert_eq!(slip.columns.len(), 5);
}

#[test]
fn the_item_total_of_weighed_lines_adds_their_weights() {
    // Two weighed lines of grapes (12.345 kg and 2.5 kg) and one line of banana: the weight column
    // is there for all rows, the banana line has none, and the grapes total row holds 14.845 kg.
    let w = worked_day();
    let mut lot = support::bare_lot(&w, "lot-g", w.rahim, w.weight_fruit, "3");
    lot.weight_received = Some("14.845".into());
    let mut first = support::sale("s-1", w.akram, "2", "100");
    first.weight = Some("12.345".into());
    let mut second = support::sale("s-2", w.arshad, "1", "100");
    second.weight = Some("2.5".into());
    lot.sales = vec![first, second];
    let banana = support::with_sales(
        support::bare_lot(&w, "lot-b", w.rahim, w.banana, "10"),
        vec![support::sale("s-3", w.akram, "10", "600")],
    );
    save(&w, &support::draft(vec![lot, banana]));
    let slip = report(
        &w,
        ReportId::R35,
        sheet("10-10-2026", account(w.rahim)),
        0,
        0,
    );
    // Grapes: two lines and their total; then banana alone.
    let kinds: Vec<RowKind> = slip.rows.iter().map(|r| r.kind).collect();
    assert_eq!(
        kinds,
        vec![
            RowKind::Normal,
            RowKind::Normal,
            RowKind::ItemTotal,
            RowKind::Normal
        ]
    );
    assert!(matches!(
        &slip.rows[0].cells[5],
        Cell::Weight { grams: 12_345 }
    ));
    assert!(matches!(
        &slip.rows[1].cells[5],
        Cell::Weight { grams: 2_500 }
    ));
    assert!(matches!(
        &slip.rows[2].cells[5],
        Cell::Weight { grams: 14_845 }
    ));
    assert!(matches!(&slip.rows[3].cells[5], Cell::Empty));
}
