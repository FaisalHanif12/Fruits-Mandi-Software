//! The reports of plan d5-late, round 7 (PRD 19.2, 16.1, 16.5, 17.9, FR-19.10, FR-19.11):
//! R12 روکڑ, R29 خرچہ رپورٹ, R31 شراکت دار کھاتہ and R32 ادھار رقم رپورٹ, on the PRD worked day.
//!
//! The expected numbers are the PRD's: the cash of 10-10-26 (E2E-8, E2E-36), the expense and the
//! money Alif took (E2E-9, E2E-29), اسلم's loan (E2E-30, E2E-30b) and the statement of Bay
//! (E2E-37). Each report is also held against the screen that shows the same figures (the cash
//! book, the loans, the partner statement) and against R30, so a report can never disagree with
//! the books (money agent Section 9).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
mod xlsx_support;

use common::*;
use mandi_reports::export::{ReportExportInput, export_report_page};
use mandi_reports::reports::{
    ArgValue, Cell, FilterKind, PrintArgValue, PrintCell, ReportFilters, ReportGroup, ReportId,
    ReportPrintInput, catalog, print_data_for_page, statement_page, statement_request,
};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::auth::Language;
use mandi_services::core::capital::CapitalInput;
use mandi_services::core::cashbook::CashbookInput;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::expenses::{PersonalExpenseInput, PersonalReturnInput, ShopExpenseInput};
use mandi_services::core::lent::{
    LentPersonCreateInput, LentReportInput, LentReturnMode, MoneyLentInput, MoneyLentReturnInput,
};
use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
use mandi_services::core::statement::PartnersStatementInput;
use mandi_services::{ServiceError, cashbook, lent, masters, partners, vouchers};
use reports_world::*;
use support::World;

fn dates(from: &str, to: &str) -> ReportFilters {
    ReportFilters {
        from: from.into(),
        to: to.into(),
        ..ReportFilters::default()
    }
}

fn partner(id: i64) -> ReportFilters {
    ReportFilters {
        partner_id: Some(id),
        ..ReportFilters::default()
    }
}

/// Repeats a save that asks to confirm rule-made spellings, confirming what it asks.
fn confirming<T>(mut attempt: impl FnMut(&[String]) -> Result<T, ServiceError>) -> T {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        match attempt(&words) {
            Ok(done) => return done,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

fn text_of(cell: &Cell) -> String {
    match cell {
        Cell::Text { text } => text.clone(),
        Cell::Empty => String::new(),
        other => panic!("not text: {other:?}"),
    }
}

fn label_of(cell: &Cell) -> String {
    match cell {
        Cell::Label { id } => id.clone(),
        other => panic!("not a label: {other:?}"),
    }
}

fn names_en(cell: &Cell) -> String {
    match cell {
        Cell::Names { names } => names.en.clone(),
        other => panic!("not names: {other:?}"),
    }
}

/// The worked day of PRD 4 (Events 1 to 5) with Event 5 saved through the real خرچہ vouchers, so
/// the cash book has a voucher behind every voucher entry (the shared `full_day` posts Event 5 as
/// plain journal entries).
fn real_day() -> World {
    let w = worked_day();
    save(&w, &support::event1(&w));
    save(&w, &event2(&w));
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "4,316")).unwrap();
    let head = confirming(|words| {
        masters::save(
            &w.env.ctx,
            &w.owner,
            &MasterInput::ExpenseHead(ExpenseHeadInput {
                id: None,
                version: None,
                code: String::new(),
                name_ur: "بجلی".into(),
                name_en: "Electricity".into(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    });
    drop(head);
    let head_id = masters::expense_heads(&w.env.ctx)
        .unwrap()
        .rows
        .iter()
        .find(|r| r.head.name_en == "Electricity")
        .unwrap()
        .id;
    vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &ShopExpenseInput {
            date: "10-10-2026".into(),
            expense_head_id: Some(head_id),
            amount: "3,000".into(),
            paid_to: Some("WAPDA".into()),
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    vouchers::save_personal_expense(
        &w.env.ctx,
        &w.owner,
        &PersonalExpenseInput {
            date: "10-10-2026".into(),
            partner_id: Some(w.partners[0]),
            amount: "2,000".into(),
            reason: "گھر کا خرچ".into(),
            return_by: None,
            taken_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    w
}

// ---- The catalogue ---------------------------------------------------------------------------

#[test]
fn the_four_reports_are_in_the_catalogue_with_their_filters() {
    let entries = catalog();
    let find = |id| entries.iter().find(|e| e.id == id).unwrap();
    assert_eq!(find(ReportId::R12).group, ReportGroup::Accounts);
    assert_eq!(find(ReportId::R12).filters, vec![FilterKind::DateRange]);
    assert_eq!(
        find(ReportId::R29).filters,
        vec![FilterKind::DateRange, FilterKind::Partner]
    );
    assert!(!find(ReportId::R29).party_required);
    assert_eq!(
        find(ReportId::R31).filters,
        vec![FilterKind::DateRange, FilterKind::Partner]
    );
    assert!(
        find(ReportId::R31).party_required,
        "the statement is one partner's"
    );
    assert_eq!(
        find(ReportId::R32).filters,
        vec![FilterKind::AsOf, FilterKind::Party]
    );
    assert_eq!(find(ReportId::R32).group, ReportGroup::Business);
}

// ---- R12 -------------------------------------------------------------------------------------

#[test]
fn r12_for_the_day_is_the_cash_of_e2e_8_and_e2e_36() {
    let w = real_day();
    let page = report(&w, ReportId::R12, dates("10-10-2026", "10-10-2026"), 0, 0);
    // Event 1's 5,000 to رحیم, Event 2's 12,800, the receipt, the payment and the two expenses.
    let moves: Vec<(i64, i64)> = page
        .rows
        .iter()
        .map(|r| (money(&r.cells[4]), money(&r.cells[5])))
        .collect();
    assert_eq!(
        moves,
        vec![
            (0, 5_000),
            (12_800, 0),
            (10_000, 0),
            (0, 4_316),
            (0, 3_000),
            (0, 2_000)
        ]
    );
    let running: Vec<i64> = page.rows.iter().map(|r| balance(&r.cells[6]).1).collect();
    assert_eq!(
        running,
        vec![45_000, 57_800, 67_800, 63_484, 60_484, 58_484]
    );
    assert!(
        page.rows
            .iter()
            .all(|r| balance(&r.cells[6]).0 == BalanceSide::Dr)
    );
    // Opening 50,000, in 22,800, out 14,316, closing 58,484 (E2E-36).
    assert_eq!(
        page.summary
            .iter()
            .map(|s| (s.label_id.as_str(), balance(&s.cell)))
            .collect::<Vec<_>>(),
        vec![
            ("report-opening-balance", (BalanceSide::Dr, 50_000)),
            ("report-closing-balance", (BalanceSide::Dr, 58_484))
        ]
    );
    assert_eq!(
        (money(&page.totals[4]), money(&page.totals[5])),
        (22_800, 14_316)
    );
    assert_eq!(balance(&page.totals[6]), (BalanceSide::Dr, 58_484));
    assert_eq!(page.total_count, 6);
    assert_eq!(page.title_id, "report-r12-title");
}

#[test]
fn r12_names_the_other_side_only_when_it_is_one_account() {
    let w = real_day();
    let page = report(&w, ReportId::R12, dates("10-10-2026", "10-10-2026"), 0, 0);
    let account = |i: usize| match &page.rows[i].cells[2] {
        Cell::Party { party } => party.code.clone(),
        Cell::Empty => String::new(),
        other => panic!("{other:?}"),
    };
    // The entry of a بکری has many other accounts (buyers, the seller, income), so its cash row
    // names none. A receipt, a payment and an expense have one other side each.
    assert_eq!(account(0), "");
    assert_eq!(account(1), "");
    assert_eq!(account(2), "03040001");
    assert_eq!(account(3), "030110001");
    assert_eq!(account(4), "05010001");
    assert_eq!(account(5), "06030001");
}

#[test]
fn r12_is_the_cash_book_of_the_screen_row_for_row() {
    let w = real_day();
    let screen = cashbook::get(
        &w.env.ctx,
        &CashbookInput {
            from: Some("10-10-2026".into()),
            to: Some("10-10-2026".into()),
            offset: 0,
            limit: 1_000,
        },
    )
    .unwrap();
    let page = report(&w, ReportId::R12, dates("10-10-2026", "10-10-2026"), 0, 0);
    assert_eq!(page.total_count, screen.total_count);
    assert_eq!(page.rows.len(), screen.rows.len());
    for (row, shown) in page.rows.iter().zip(&screen.rows) {
        assert_eq!(money(&row.cells[4]), shown.in_paisa.rupees());
        assert_eq!(money(&row.cells[5]), shown.out_paisa.rupees());
        assert_eq!(balance(&row.cells[6]).1, shown.running_paisa.rupees());
        assert!(matches!(&row.cells[0], Cell::Date { iso } if *iso == shown.entry_date));
        assert!(
            matches!(&row.cells[1], Cell::Doc { number, .. } if *number == shown.number),
            "the document number is the screen's"
        );
        assert_eq!(text_of(&row.cells[3]), shown.narration);
    }
    assert_eq!(
        balance(&page.summary[0].cell).1,
        screen.opening_paisa.rupees()
    );
    assert_eq!(
        balance(&page.summary[1].cell).1,
        screen.closing_paisa.rupees()
    );
}

#[test]
fn r12_a_page_far_down_has_the_running_cash_of_the_whole_range() {
    let w = real_day();
    let page = report(&w, ReportId::R12, dates("10-10-2026", "10-10-2026"), 2, 2);
    assert_eq!(page.rows.len(), 2);
    assert_eq!(
        page.rows
            .iter()
            .map(|r| balance(&r.cells[6]).1)
            .collect::<Vec<_>>(),
        vec![67_800, 63_484]
    );
    assert_eq!(page.total_count, 6);
    // The totals are over the whole range, not the page.
    assert_eq!(
        (money(&page.totals[4]), money(&page.totals[5])),
        (22_800, 14_316)
    );
}

#[test]
fn r12_a_later_range_opens_with_the_cash_before_it_and_an_empty_one_has_no_rows() {
    let w = real_day();
    set_today(&w.env, 2026, 10, 12);
    let page = report(&w, ReportId::R12, dates("11-10-2026", "12-10-2026"), 0, 0);
    assert!(page.rows.is_empty());
    assert_eq!(page.total_count, 0);
    assert_eq!(balance(&page.summary[0].cell), (BalanceSide::Dr, 58_484));
    assert_eq!(balance(&page.summary[1].cell), (BalanceSide::Dr, 58_484));
    assert_eq!(money(&page.totals[4]), 0);
    // No dates at all: the book starts at the first entry, with nothing before it.
    let all = report(&w, ReportId::R12, ReportFilters::default(), 0, 0);
    assert_eq!(balance(&all.summary[0].cell).1, 0);
    assert_eq!(balance(&all.summary[1].cell), (BalanceSide::Dr, 58_484));
}

#[test]
fn r12_shows_cash_below_zero_on_the_credit_side_and_agrees_with_the_cash_book() {
    // A shop expense of 70,000 (confirmed) takes the cash of 58,484 to -11,516.
    let w = real_day();
    let head = masters::expense_heads(&w.env.ctx)
        .unwrap()
        .rows
        .iter()
        .find(|r| r.head.name_en == "Electricity")
        .unwrap()
        .id;
    vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &ShopExpenseInput {
            date: "10-10-2026".into(),
            expense_head_id: Some(head),
            amount: "70,000".into(),
            paid_to: None,
            narration: None,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let page = report(&w, ReportId::R12, dates("10-10-2026", "10-10-2026"), 0, 0);
    let last = page.rows.last().unwrap();
    assert_eq!(balance(&last.cells[6]), (BalanceSide::Cr, 11_516));
    assert_eq!(balance(&page.summary[1].cell), (BalanceSide::Cr, 11_516));
    assert_eq!(balance(&page.totals[6]), (BalanceSide::Cr, 11_516));
    assert_eq!(
        (money(&page.totals[4]), money(&page.totals[5])),
        (22_800, 84_316)
    );
    let screen = cashbook::get(
        &w.env.ctx,
        &CashbookInput {
            from: Some("10-10-2026".into()),
            to: Some("10-10-2026".into()),
            offset: 0,
            limit: 1_000,
        },
    )
    .unwrap();
    assert_eq!(screen.closing_paisa.rupees(), -11_516);
}

// ---- R29 -------------------------------------------------------------------------------------

#[test]
fn r29_for_the_day_has_the_shop_expense_and_what_alif_took() {
    let w = full_day();
    let page = report(&w, ReportId::R29, dates("10-10-2026", "10-10-2026"), 0, 0);
    // Shop expenses: a heading and the Electricity row; partner money: a heading and Alif.
    assert_eq!(page.rows.len(), 4);
    assert_eq!(
        label_of(&page.rows[0].cells[0]),
        "report-section-shop-expenses"
    );
    assert_eq!(money(&page.rows[0].cells[2]), 3_000);
    assert_eq!(names_en(&page.rows[1].cells[1]), "Electricity");
    assert_eq!(money(&page.rows[1].cells[2]), 3_000);
    assert_eq!(
        label_of(&page.rows[2].cells[0]),
        "report-section-partner-money"
    );
    assert_eq!(
        (money(&page.rows[2].cells[3]), money(&page.rows[2].cells[4])),
        (2_000, 0)
    );
    assert_eq!(names_en(&page.rows[3].cells[1]), "Partner A");
    assert_eq!(
        (money(&page.rows[3].cells[3]), money(&page.rows[3].cells[4])),
        (2_000, 0)
    );
    assert_eq!(
        page.totals
            .iter()
            .map(|c| match c {
                Cell::Money { paisa } => paisa.rupees(),
                _ => 0,
            })
            .collect::<Vec<_>>(),
        vec![0, 0, 3_000, 2_000, 0]
    );
    assert_eq!(page.title_id, "report-r29-title");
}

#[test]
fn r29_ties_to_r30_the_expense_and_what_the_partners_took() {
    let w = full_day();
    let r29 = report(&w, ReportId::R29, ReportFilters::default(), 0, 0);
    let r30 = report(&w, ReportId::R30, ReportFilters::default(), 0, 0);
    let expenses = r30
        .summary
        .iter()
        .find(|s| s.label_id == "report-r30-expenses")
        .unwrap();
    assert_eq!(money(&r29.totals[2]), money(&expenses.cell));
    // R30's "taken" and "returned" columns add up to R29's.
    assert_eq!(money(&r29.totals[3]), money(&r30.totals[4]));
    assert_eq!(money(&r29.totals[4]), money(&r30.totals[5]));
}

#[test]
fn r29_with_a_partner_shows_only_his_money_and_a_partner_who_took_nothing_has_no_row() {
    let w = full_day();
    let [a, b, _] = w.partners;
    let alif = report(&w, ReportId::R29, partner(a), 0, 0);
    assert_eq!(
        alif.rows.len(),
        2,
        "a heading and his row, no shop expenses"
    );
    assert_eq!(
        label_of(&alif.rows[0].cells[0]),
        "report-section-partner-money"
    );
    assert_eq!(money(&alif.totals[3]), 2_000);
    assert_eq!(money(&alif.totals[2]), 0);
    assert_eq!(
        alif.filters.partner.as_ref().map(|n| n.en.as_str()),
        Some("Partner A")
    );
    let bay = report(&w, ReportId::R29, partner(b), 0, 0);
    assert!(bay.rows.is_empty());
    assert_eq!(money(&bay.totals[3]), 0);
}

#[test]
fn r29_shows_what_came_back_when_alif_returns_money() {
    let w = full_day();
    let a = w.partners[0];
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_personal_return(
        &w.env.ctx,
        &w.owner,
        &PersonalReturnInput {
            date: "11-10-2026".into(),
            partner_id: Some(a),
            amount: "2,000".into(),
            narration: None,
        },
    )
    .unwrap();
    let page = report(&w, ReportId::R29, dates("10-10-2026", "11-10-2026"), 0, 0);
    assert_eq!(
        (money(&page.totals[3]), money(&page.totals[4])),
        (2_000, 2_000)
    );
    // R30 agrees: خالص is back to his profit share 996 (E2E-29).
    let r30 = report(&w, ReportId::R30, dates("10-10-2026", "11-10-2026"), 0, 0);
    assert_eq!(money(&r30.rows[0].cells[6]), 996);
    // A day of its own sees only the return.
    let day = report(&w, ReportId::R29, dates("11-10-2026", "11-10-2026"), 0, 0);
    assert_eq!(
        (
            money(&day.totals[2]),
            money(&day.totals[3]),
            money(&day.totals[4])
        ),
        (0, 0, 2_000)
    );
}

#[test]
fn r29_asks_for_a_real_partner_and_the_other_reports_refuse_a_partner() {
    let w = full_day();
    assert_eq!(
        rejected(&w, ReportId::R29, partner(9_999), 0, 0),
        vec![(ErrorCode::ErrNotFound, "filters.partner_id".to_owned())]
    );
    assert_eq!(
        rejected(&w, ReportId::R30, partner(w.partners[0]), 0, 0),
        vec![(ErrorCode::ErrOutOfRange, "filters.partner_id".to_owned())]
    );
    assert_eq!(
        rejected(&w, ReportId::R31, ReportFilters::default(), 0, 0),
        vec![(ErrorCode::ErrRequired, "filters.partner_id".to_owned())],
        "R31 needs a partner"
    );
}

// ---- R32 -------------------------------------------------------------------------------------

/// Last month's world with اسلم lent 5,000 on 10-10-26 (return by 15-10-26), as E2E-30.
fn lent_world() -> (World, i64, i64) {
    let w = full_day();
    let aslam = confirming(|words| {
        lent::person_create(
            &w.env.ctx,
            &w.owner,
            &LentPersonCreateInput {
                name_ur: "اسلم".into(),
                name_en: "Aslam".into(),
                phone: Some("0300-1234567".into()),
                staff_id: None,
                confirmed_rules_words: words.to_vec(),
            },
        )
    })
    .account
    .id;
    let salaries = {
        confirming(|words| {
            masters::save(
                &w.env.ctx,
                &w.owner,
                &MasterInput::ExpenseHead(ExpenseHeadInput {
                    id: None,
                    version: None,
                    code: String::new(),
                    name_ur: "تنخواہ".into(),
                    name_en: "Salaries".into(),
                    confirmed_rules_words: words.to_vec(),
                }),
            )
        });
        masters::expense_heads(&w.env.ctx)
            .unwrap()
            .rows
            .iter()
            .find(|r| r.head.name_en == "Salaries")
            .unwrap()
            .id
    };
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &MoneyLentInput {
            date: "10-10-2026".into(),
            person_account_id: Some(aslam),
            amount: "5,000".into(),
            reason: "گھریلو ضرورت".into(),
            return_by: Some("15-10-2026".into()),
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    (w, aslam, salaries)
}

fn as_of(day: &str) -> ReportFilters {
    ReportFilters {
        as_of: day.into(),
        ..ReportFilters::default()
    }
}

#[test]
fn r32_e2e_30b_shows_the_loan_open_and_one_day_past_its_return_by_day() {
    let (w, _aslam, _) = lent_world();
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &MoneyLentReturnInput {
            date: "11-10-2026".into(),
            person_account_id: Some(_aslam),
            amount: "3,000".into(),
            mode: LentReturnMode::Cash,
            salary_head_id: None,
            narration: None,
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 16);
    let page = report(&w, ReportId::R32, ReportFilters::default(), 0, 0);
    assert_eq!(page.rows.len(), 1);
    let row = &page.rows[0].cells;
    assert_eq!(code_of(&row[0]).len(), 8, "his ادھار رقم account code");
    // lent 5,000, returned 3,000 cash and 0 salary, open 2,000.
    assert_eq!(
        (
            money(&row[2]),
            money(&row[3]),
            money(&row[4]),
            money(&row[5])
        ),
        (5_000, 3_000, 0, 2_000)
    );
    assert!(matches!(&row[6], Cell::Date { iso } if iso == "2026-10-10"));
    assert!(matches!(&row[7], Cell::Date { iso } if iso == "2026-10-15"));
    assert!(matches!(&row[8], Cell::Number { value: 1 }));
    // The totals, and the overdue line.
    assert_eq!(
        page.totals
            .iter()
            .map(|c| match c {
                Cell::Money { paisa } => paisa.rupees(),
                _ => 0,
            })
            .collect::<Vec<_>>(),
        vec![0, 0, 5_000, 3_000, 0, 2_000, 0, 0, 0]
    );
    assert_eq!(page.summary[0].label_id, "report-r32-overdue");
    assert_eq!(money(&page.summary[0].cell), 2_000);
    // It is as of today: the print header says which day.
    assert_eq!(page.filters.as_of.as_deref(), Some("2026-10-16"));
}

#[test]
fn r32_e2e_30_the_loan_is_settled_by_cash_and_salary_and_the_figures_are_the_screens() {
    let (w, aslam, salaries) = lent_world();
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &MoneyLentReturnInput {
            date: "11-10-2026".into(),
            person_account_id: Some(aslam),
            amount: "3,000".into(),
            mode: LentReturnMode::Cash,
            salary_head_id: None,
            narration: None,
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 12);
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &MoneyLentReturnInput {
            date: "12-10-2026".into(),
            person_account_id: Some(aslam),
            amount: "2,000".into(),
            mode: LentReturnMode::Salary,
            salary_head_id: Some(salaries),
            narration: None,
        },
    )
    .unwrap();
    let page = report(&w, ReportId::R32, ReportFilters::default(), 0, 0);
    let row = &page.rows[0].cells;
    assert_eq!(
        (
            money(&row[2]),
            money(&row[3]),
            money(&row[4]),
            money(&row[5])
        ),
        (5_000, 3_000, 2_000, 0)
    );
    assert!(
        matches!(&row[6], Cell::Empty),
        "nothing open, no oldest date"
    );
    assert!(matches!(&row[8], Cell::Empty));
    // The screen's report gives the same row and the same totals.
    let screen = lent::report(
        &w.env.ctx,
        &LentReportInput {
            as_of: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(screen.rows.len(), page.rows.len());
    let shown = &screen.rows[0];
    assert_eq!(money(&row[2]), shown.lent_paisa.rupees());
    assert_eq!(money(&row[3]), shown.returned_cash_paisa.rupees());
    assert_eq!(money(&row[4]), shown.returned_salary_paisa.rupees());
    assert_eq!(money(&row[5]), shown.open_paisa.rupees());
    assert_eq!(money(&page.totals[2]), screen.totals.lent_paisa.rupees());
    assert_eq!(money(&page.totals[5]), screen.totals.open_paisa.rupees());
    assert_eq!(
        money(&page.summary[0].cell),
        screen.totals.overdue_paisa.rupees()
    );
    // Only the Owner's lent people count: Alif's drawings are not here.
    assert_eq!(page.total_count, 1);
}

#[test]
fn r32_as_of_an_earlier_day_leaves_out_what_came_after_and_the_open_ones_come_first() {
    let (w, aslam, _) = lent_world();
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &MoneyLentReturnInput {
            date: "11-10-2026".into(),
            person_account_id: Some(aslam),
            amount: "5,000".into(),
            mode: LentReturnMode::Cash,
            salary_head_id: None,
            narration: None,
        },
    )
    .unwrap();
    // As of 10-10 the loan is still open; as of 11-10 it is settled.
    let early = report(&w, ReportId::R32, as_of("10-10-2026"), 0, 0);
    assert_eq!(money(&early.rows[0].cells[5]), 5_000);
    assert_eq!(early.filters.as_of.as_deref(), Some("2026-10-10"));
    let late = report(&w, ReportId::R32, as_of("11-10-2026"), 0, 0);
    assert_eq!(money(&late.rows[0].cells[5]), 0);
    assert_eq!(money(&late.rows[0].cells[3]), 5_000);
}

#[test]
fn r32_a_picked_account_that_is_not_a_lent_person_gives_an_empty_report() {
    let (w, _, _) = lent_world();
    let page = report(&w, ReportId::R32, party(w.akram), 0, 0);
    assert!(page.rows.is_empty());
    assert_eq!(page.total_count, 0);
    assert!(page.filters.as_of.is_some());
}

// ---- R31 -------------------------------------------------------------------------------------

/// The page the app builds: the filters are read, the services give the statement, the engine
/// makes the page.
fn r31(
    w: &World,
    filters: &ReportFilters,
    offset: i64,
    limit: i64,
) -> mandi_reports::reports::ReportPage {
    let request = statement_request(w.env.ctx.db(), filters).unwrap();
    let day = |iso: &Option<String>| {
        iso.as_deref()
            .and_then(mandi_services::core::dates::BusinessDate::parse_iso)
            .map(mandi_services::core::dates::format_ddmmyyyy)
    };
    let statement = partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            partner_id: request.partner_id,
            from: day(&request.from),
            to: day(&request.to),
            offset: u32::try_from(offset).unwrap(),
            limit: u32::try_from(limit).unwrap(),
        },
    )
    .unwrap();
    statement_page(&request, &statement, offset, limit).unwrap()
}

fn bay_invests(w: &World) {
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_capital_in(
        &w.env.ctx,
        &w.owner,
        &CapitalInput {
            date: "11-10-2026".into(),
            partner_id: Some(w.partners[1]),
            amount: "50,000".into(),
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

fn message_args(cell: &Cell) -> (String, Vec<(String, ArgValue)>) {
    match cell {
        Cell::Message { id, args } => (
            id.clone(),
            args.iter()
                .map(|a| (a.name.clone(), a.value.clone()))
                .collect(),
        ),
        other => panic!("not a message: {other:?}"),
    }
}

fn arg<'a>(args: &'a [(String, ArgValue)], name: &str) -> &'a ArgValue {
    &args
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("no arg {name}"))
        .1
}

#[test]
fn r31_e2e_37_the_statement_of_bay_is_a_report_with_the_running_total() {
    let w = real_day();
    bay_invests(&w);
    let page = r31(&w, &partner(w.partners[1]), 0, 100);
    assert_eq!(page.id, ReportId::R31);
    assert_eq!(page.title_id, "report-r31-title");
    // date, amount, running: +1,184, +462, -900, +50,000 (E2E-37).
    let figures: Vec<(String, i64, i64)> = page
        .rows
        .iter()
        .map(|r| {
            let date = match &r.cells[0] {
                Cell::Date { iso } => iso.clone(),
                other => panic!("{other:?}"),
            };
            (date, money(&r.cells[3]), money(&r.cells[4]))
        })
        .collect();
    assert_eq!(
        figures,
        vec![
            ("2026-10-10".to_owned(), 1_184, 1_184),
            ("2026-10-10".to_owned(), 462, 1_646),
            ("2026-10-10".to_owned(), -900, 746),
            ("2026-10-11".to_owned(), 50_000, 50_746),
        ]
    );
    // The groups and the sentences are Fluent messages with their arguments.
    let headings: Vec<String> = page.rows.iter().map(|r| label_of(&r.cells[1])).collect();
    assert_eq!(
        headings,
        vec![
            "partners-statement-heading-bikri",
            "partners-statement-heading-bikri",
            "partners-statement-heading-expense",
            "partners-statement-heading-capital"
        ]
    );
    let (id, args) = message_args(&page.rows[0].cells[2]);
    assert_eq!(id, "partners-statement-detail-income-share");
    assert!(matches!(arg(&args, "number"), ArgValue::Text { text } if text == "1"));
    let (id, args) = message_args(&page.rows[2].cells[2]);
    assert_eq!(id, "partners-statement-detail-expense-share");
    // The voucher is a number in its own series, and the subject is the expense head.
    assert!(matches!(
        arg(&args, "voucher"),
        ArgValue::Message { id, .. } if id == "vouchers-number-shop-expense"
    ));
    assert!(matches!(
        arg(&args, "subject"),
        ArgValue::Names { names } if names.en == "Electricity"
    ));
    let (id, args) = message_args(&page.rows[3].cells[2]);
    assert_eq!(id, "partners-statement-detail-capital-in");
    assert!(matches!(
        arg(&args, "voucher"),
        ArgValue::Message { id, .. } if id == "vouchers-number-capital-in"
    ));
    // Opening 0, added 51,646, taken away 900, closing 50,746. The totals row is the net.
    assert_eq!(
        page.summary
            .iter()
            .map(|s| (s.label_id.as_str(), money(&s.cell)))
            .collect::<Vec<_>>(),
        vec![
            ("partners-statement-opening", 0),
            ("partners-statement-plus", 51_646),
            ("partners-statement-minus", 900),
            ("partners-statement-closing", 50_746)
        ]
    );
    assert_eq!(
        (money(&page.totals[3]), money(&page.totals[4])),
        (50_746, 50_746)
    );
    assert_eq!(page.total_count, 4);
    assert_eq!(
        page.filters.partner.as_ref().map(|n| n.en.as_str()),
        Some("Partner B")
    );
}

#[test]
fn r31_a_range_opens_with_the_total_before_it_and_a_page_keeps_the_running_total() {
    let w = real_day();
    bay_invests(&w);
    let filters = ReportFilters {
        partner_id: Some(w.partners[1]),
        from: "11-10-2026".into(),
        to: "11-10-2026".into(),
        ..ReportFilters::default()
    };
    let page = r31(&w, &filters, 0, 100);
    assert_eq!(page.rows.len(), 1);
    assert_eq!(
        money(&page.summary[0].cell),
        746,
        "the total at the start of 11-10"
    );
    assert_eq!(money(&page.rows[0].cells[4]), 50_746);
    assert_eq!(money(&page.totals[3]), 50_000);
    assert_eq!(page.filters.from.as_deref(), Some("2026-10-11"));
    // A second page of the whole statement keeps the running total.
    let second = r31(&w, &partner(w.partners[1]), 2, 2);
    assert_eq!(
        second
            .rows
            .iter()
            .map(|r| money(&r.cells[4]))
            .collect::<Vec<_>>(),
        vec![746, 50_746]
    );
    assert_eq!((second.offset, second.limit, second.total_count), (2, 2, 4));
}

#[test]
fn r31_the_closing_total_is_the_partner_total_of_the_overview_for_every_partner() {
    let w = real_day();
    bay_invests(&w);
    let overview = partners::overview(&w.env.ctx).unwrap();
    for id in w.partners {
        let page = r31(&w, &partner(id), 0, 100);
        let total = overview
            .iter()
            .find(|r| r.partner_id == id)
            .unwrap()
            .total_paisa;
        assert_eq!(money(&page.summary[3].cell), total.rupees());
        // opening + net = closing, the check the page builder makes for itself.
        assert_eq!(
            money(&page.summary[0].cell) + money(&page.totals[3]),
            total.rupees()
        );
    }
}

#[test]
fn r31_needs_a_partner_and_refuses_an_unknown_one_at_its_field() {
    let w = full_day();
    let fields = |filters: &ReportFilters| match statement_request(w.env.ctx.db(), filters) {
        Err(mandi_reports::ReportsError::Rejected(items)) => items
            .into_iter()
            .map(|e| (e.code, e.field))
            .collect::<Vec<_>>(),
        other => panic!("{other:?}"),
    };
    assert_eq!(
        fields(&ReportFilters::default()),
        vec![(ErrorCode::ErrRequired, "filters.partner_id".to_owned())]
    );
    assert_eq!(
        fields(&partner(9_999)),
        vec![(ErrorCode::ErrNotFound, "filters.partner_id".to_owned())]
    );
    assert_eq!(
        fields(&ReportFilters {
            partner_id: Some(w.partners[0]),
            from: "31-02-2026".into(),
            ..ReportFilters::default()
        })[0]
            .1,
        "filters.from"
    );
}

#[test]
fn the_engine_alone_does_not_build_r31() {
    // The engine alone cannot make R31: its lines come from the services.
    let w = full_day();
    let error = mandi_reports::reports::run(
        w.env.ctx.db(),
        &mandi_reports::reports::ReportRunInput {
            id: ReportId::R31,
            filters: partner(w.partners[0]),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap_err();
    assert!(
        matches!(error, mandi_reports::ReportsError::Data(_)),
        "{error:?}"
    );
}

#[test]
fn r31_prints_its_sentences_in_the_print_language_with_the_numbers_already_written() {
    let w = real_day();
    bay_invests(&w);
    let page = r31(&w, &partner(w.partners[1]), 0, 0);
    let print = |language| {
        print_data_for_page(
            w.env.ctx.db(),
            &page,
            &ReportPrintInput {
                id: ReportId::R31,
                filters: partner(w.partners[1]),
                language,
                paper: None,
            },
        )
        .unwrap()
    };
    let text_of_arg = |args: &[mandi_reports::reports::PrintMessageArg], name: &str| match &args
        .iter()
        .find(|a| a.name == name)
        .unwrap()
        .value
    {
        PrintArgValue::Text { text } => text.clone(),
        other => panic!("{other:?}"),
    };
    for (language, subject) in [(Language::En, "Electricity"), (Language::Ur, "بجلی")] {
        let data = print(language);
        assert_eq!(data.rows.len(), 4);
        let PrintCell::Message { id, args } = &data.rows[2].cells[2] else {
            panic!("{:?}", data.rows[2].cells[2]);
        };
        assert_eq!(id, "partners-statement-detail-expense-share");
        assert_eq!(text_of_arg(args, "subject"), subject);
        let voucher = &args.iter().find(|a| a.name == "voucher").unwrap().value;
        assert!(matches!(
            voucher,
            PrintArgValue::Message { id, args } if id == "vouchers-number-shop-expense"
                && text_of_arg(args, "number") == "1"
        ));
        // The amounts are text in the print's number style, the running total included.
        let PrintCell::Money { text } = &data.rows[3].cells[4] else {
            panic!("{:?}", data.rows[3].cells[4]);
        };
        assert_eq!(text, "50,746");
        // The partner is one of the facts in the header.
        assert!(
            data.filters
                .iter()
                .any(|f| f.label_id == "report-filter-partner")
        );
    }
}

#[test]
fn r31_exports_to_a_sheet_with_the_sentences_and_the_numbers_in_both_languages() {
    let w = real_day();
    bay_invests(&w);
    let page = r31(&w, &partner(w.partners[1]), 0, 0);
    let dir = tempfile::tempdir().unwrap();
    // The number series of a voucher is written in the sheet language: E- and CI- in English,
    // خ- and س- in Urdu.
    for (language, subject, expense, capital, right_to_left) in [
        (Language::En, "Electricity", "E-1", "CI-1", false),
        (Language::Ur, "بجلی", "خ-1", "س-1", true),
    ] {
        let path = dir.path().join(format!("r31-{language:?}.xlsx"));
        let done = export_report_page(
            w.env.ctx.db(),
            &page,
            &ReportExportInput {
                id: ReportId::R31,
                filters: ReportFilters::default(),
                language,
                path: path.display().to_string(),
            },
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(done.rows, 4);
        let sheet = xlsx_support::read(&path);
        assert_eq!(sheet.right_to_left, right_to_left);
        let texts = sheet.all_texts();
        assert!(
            texts
                .iter()
                .any(|t| t.contains(expense) && t.contains(subject)),
            "the expense sentence in {language:?}: {texts:?}"
        );
        assert!(texts.iter().any(|t| t.contains(capital)), "{texts:?}");
        // The income share sentence names the sale: بکری 1.
        assert!(texts.iter().any(|t| t.contains(" 1:")), "{texts:?}");
        // Amounts and running totals are numbers, a minus is a minus.
        let numbers: Vec<String> = (0..=sheet.last_row())
            .flat_map(|r| (0..6).map(move |c| (r, c)))
            .filter_map(|(r, c)| match sheet.get(r, c) {
                xlsx_support::Value::Num(n) => Some(n.clone()),
                _ => None,
            })
            .collect();
        for want in ["1184", "462", "-900", "50000", "1646", "746", "50746"] {
            assert!(numbers.contains(&want.to_owned()), "{want} in {numbers:?}");
        }
        // Nothing fell back to its message ID, except the titles the frontend writes later.
        let l10n = mandi_reports::export::l10n::L10n::new(language).unwrap();
        for text in &texts {
            let pending = text.starts_with("report-r31-") && !l10n.has(text);
            assert!(
                pending || !text.starts_with("partners-") && !text.starts_with("vouchers-"),
                "{text}"
            );
        }
    }
}

#[test]
fn r32_lists_those_with_something_open_first_like_the_screen_and_leaves_out_people_with_no_loan() {
    let (w, aslam, _) = lent_world();
    // Bilal borrows 1,000 on 10-10 and pays it back at once; Carim is a person with no loan.
    let make = |ur: &str, en: &str| {
        confirming(|words| {
            lent::person_create(
                &w.env.ctx,
                &w.owner,
                &LentPersonCreateInput {
                    name_ur: ur.into(),
                    name_en: en.into(),
                    phone: None,
                    staff_id: None,
                    confirmed_rules_words: words.to_vec(),
                },
            )
        })
        .account
        .id
    };
    let bilal = make("بلال", "Bilal");
    let _carim = make("کریم احمد", "Karim Ahmad");
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &MoneyLentInput {
            date: "10-10-2026".into(),
            person_account_id: Some(bilal),
            amount: "1,000".into(),
            reason: "ضرورت".into(),
            return_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    vouchers::save_money_lent_return(
        &w.env.ctx,
        &w.owner,
        &MoneyLentReturnInput {
            date: "10-10-2026".into(),
            person_account_id: Some(bilal),
            amount: "1,000".into(),
            mode: LentReturnMode::Cash,
            salary_head_id: None,
            narration: None,
        },
    )
    .unwrap();
    let page = report(&w, ReportId::R32, ReportFilters::default(), 0, 0);
    let screen = lent::report(
        &w.env.ctx,
        &LentReportInput {
            as_of: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(page.rows.len(), 2, "Aslam and Bilal, not Karim Ahmad");
    assert_eq!(screen.rows.len(), 2);
    // The same people in the same order, with the same figures.
    for (row, shown) in page.rows.iter().zip(&screen.rows) {
        assert_eq!(code_of(&row.cells[0]), shown.person.code);
        assert_eq!(money(&row.cells[2]), shown.lent_paisa.rupees());
        assert_eq!(money(&row.cells[5]), shown.open_paisa.rupees());
    }
    // Aslam (open 5,000) first, Bilal (settled) after.
    assert_eq!(money(&page.rows[0].cells[5]), 5_000);
    assert_eq!(money(&page.rows[1].cells[5]), 0);
    assert_eq!(money(&page.rows[1].cells[3]), 1_000);
    let _ = aslam;
    // A page of one row keeps the count and the totals of everybody.
    let first = report(&w, ReportId::R32, ReportFilters::default(), 0, 1);
    assert_eq!((first.rows.len(), first.total_count), (1, 2));
    assert_eq!(money(&first.totals[2]), 6_000);
}

// ---- Fixtures for the frontend ----------------------------------------------------------------

/// Real answers of the round 7 reports as JSON files for the frontend: each pair is the request
/// (read back into the real input type, so it is exactly what the command takes) and the answer
/// the real code gives. The test compares the answers with the files on disk and FAILS if any
/// differ, so a fixture can never go stale. After a deliberate change, write the files again
/// with `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test reports_round7 fixtures`,
/// read the diff and tell `frontend`.
mod fixtures {
    use std::path::{Path, PathBuf};

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

    /// The filters with every key, as the frontend sends them.
    fn filters(extra: Value) -> Value {
        let mut base = json!({
            "shortcut": null, "from": "", "to": "", "as_of": "", "party": null,
            "seller_id": null, "zamindar_id": null, "item_id": null, "city_id": null,
            "adda_id": null, "partner_id": null, "detail": false
        });
        if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
            for (k, v) in extra {
                base.insert(k.clone(), v.clone());
            }
        }
        base
    }

    fn run_pair(w: &World, name: &str, request: Value) {
        let input: mandi_reports::reports::ReportRunInput = typed(&request);
        let page = if input.id == ReportId::R31 {
            r31(w, &input.filters, input.offset, input.limit)
        } else {
            mandi_reports::reports::run(w.env.ctx.db(), &input).unwrap()
        };
        pair(name, &request, &page);
    }

    #[test]
    fn fixtures_are_the_real_answers() {
        let w = real_day();
        let bay = w.partners[1];
        let alif = w.partners[0];
        run_pair(
            &w,
            "reports_run_r12_day",
            json!({ "id": "R12", "offset": 0, "limit": 100,
                "filters": filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })) }),
        );
        run_pair(
            &w,
            "reports_run_r29_day",
            json!({ "id": "R29", "offset": 0, "limit": 100,
                "filters": filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })) }),
        );
        run_pair(
            &w,
            "reports_run_r29_alif",
            json!({ "id": "R29", "offset": 0, "limit": 100,
                "filters": filters(json!({ "partner_id": alif })) }),
        );
        bay_invests(&w);
        // R31 reads the statement of the services (the app joins the two).
        run_pair(
            &w,
            "reports_run_r31_bay",
            json!({ "id": "R31", "offset": 0, "limit": 100,
                "filters": filters(json!({ "partner_id": bay })) }),
        );
        let print_request = json!({ "id": "R31", "language": "en", "paper": null,
            "filters": filters(json!({ "partner_id": bay })) });
        let input: ReportPrintInput = typed(&print_request);
        let page = r31(&w, &input.filters, 0, 0);
        pair(
            "reports_print_data_r31_bay_en",
            &print_request,
            &print_data_for_page(w.env.ctx.db(), &page, &input).unwrap(),
        );
        // A refusal as a command returns it: R31 without a partner.
        let missing = filters(json!({}));
        let error = statement_request(w.env.ctx.db(), &typed(&missing)).unwrap_err();
        let mandi_reports::ReportsError::Rejected(items) = error else {
            panic!("{error:?}");
        };
        assert_eq!(items[0].code, ErrorCode::ErrRequired);
        pair(
            "reports_run_r31_partner_required",
            &json!({ "id": "R31", "offset": 0, "limit": 100, "filters": missing }),
            &json!({
                "ok": false,
                "errors": items.iter()
                    .map(|i| json!({ "code": i.code, "field": i.field, "values": i.values }))
                    .collect::<Vec<_>>(),
                "ref": "E-20261010-0001"
            }),
        );
    }

    #[test]
    fn the_money_lent_fixture_is_the_real_answer() {
        let (w, aslam, _) = lent_world();
        set_today(&w.env, 2026, 10, 11);
        vouchers::save_money_lent_return(
            &w.env.ctx,
            &w.owner,
            &MoneyLentReturnInput {
                date: "11-10-2026".into(),
                person_account_id: Some(aslam),
                amount: "3,000".into(),
                mode: LentReturnMode::Cash,
                salary_head_id: None,
                narration: None,
            },
        )
        .unwrap();
        set_today(&w.env, 2026, 10, 16);
        run_pair(
            &w,
            "reports_run_r32_e2e_30b",
            json!({ "id": "R32", "offset": 0, "limit": 100,
                "filters": filters(json!({ "as_of": "16-10-2026" })) }),
        );
    }
}
