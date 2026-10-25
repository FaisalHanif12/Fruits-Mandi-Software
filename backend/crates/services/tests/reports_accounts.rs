//! The Accounts group of the report engine on the PRD worked day (plan 20 step B2, PRD 19.1 to
//! 19.5, 19.8, 19.10, 19.11): R11, R13, R14, R16, R17, R18 and R19.
//!
//! The data is made the way `golden_day.rs` makes it (Events 1 to 5 of PRD Section 4, through the
//! real services). The expected numbers are the PRD's: R18 is the E2E-10 table, R13 for اکرم is
//! E2E-33. A report total is checked against the services trial balance and ledger, so a report
//! can never disagree with the books (money agent Section 9).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::reports::{
    Cell, DetailPart, FilterKind, PartyFilter, PrintCell, PrintDetailPart, ReportFilters, ReportId,
    ReportPrintInput, ReportRunInput, buyer_bills_print_data, catalog, print_data, run,
};
use mandi_services::core::accounts::{BalanceSide, OpeningLineInput, OpeningsInput};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::CancelInput;
use mandi_services::core::error::ErrorCode;
use mandi_services::{accounts, bikri, ledger};

use reports_world::*;

// ---------------------------------------------------------------------------------------------

#[test]
fn the_catalogue_lists_the_accounts_group_with_its_filters() {
    use mandi_reports::reports::ReportGroup;
    let entries: Vec<_> = catalog()
        .into_iter()
        .filter(|e| e.group == ReportGroup::Accounts)
        .collect();
    let ids: Vec<ReportId> = entries.iter().map(|e| e.id).collect();
    assert_eq!(
        ids,
        vec![
            ReportId::R11,
            ReportId::R12,
            ReportId::R13,
            ReportId::R14,
            ReportId::R15,
            ReportId::R16,
            ReportId::R17,
            ReportId::R18,
            ReportId::R19
        ]
    );
    let r13 = entries.iter().find(|e| e.id == ReportId::R13).unwrap();
    assert!(r13.party_required);
    assert_eq!(r13.filters, vec![FilterKind::DateRange, FilterKind::Party]);
    assert_eq!(r13.title_id, "report-r13-title");
    let r18 = entries.iter().find(|e| e.id == ReportId::R18).unwrap();
    assert_eq!(r18.filters, vec![FilterKind::AsOf]);
}

#[test]
fn r18_is_the_e2e_10_trial_balance_and_both_sides_are_85048() {
    let w = full_day();
    let page = report(&w, ReportId::R18, ReportFilters::default(), 0, 0);
    // (code, debit, credit) in rupees, by account code. The zero rows are accounts that have
    // lines whose balance is 0 (PRD E2E-10 lists them with 0).
    let got: Vec<(String, i64, i64)> = page
        .rows
        .iter()
        .map(|r| {
            (
                code_of(r.cells.first().unwrap()),
                money(r.cells.get(1).unwrap()),
                money(r.cells.get(2).unwrap()),
            )
        })
        .collect();
    let expected: Vec<(&str, i64, i64)> = vec![
        ("01010001", 58_484, 0),
        ("02010001", 0, 300),
        ("02010002", 0, 4_000),
        ("02010004", 0, 100),
        ("030110001", 0, 0),
        ("030110002", 0, 10_160),
        ("03040001", 2_880, 0),
        ("03040002", 18_684, 0),
        ("03049999", 0, 0),
        ("04010001", 0, 2_484),
        ("04010002", 0, 2_484),
        ("04010003", 0, 480),
        ("04010004", 0, 40),
        ("05010001", 3_000, 0),
        ("06010001", 0, 65_000),
        ("06019999", 0, 0),
        ("06030001", 2_000, 0),
    ];
    let expected: Vec<(String, i64, i64)> = expected
        .into_iter()
        .map(|(c, d, k)| (c.to_owned(), d, k))
        .collect();
    assert_eq!(got, expected);
    assert_eq!(page.total_count, 17);
    assert_eq!(page.totals.get(1).map(money), Some(85_048));
    assert_eq!(page.totals.get(2).map(money), Some(85_048));
    // The difference line is 0.
    assert_eq!(
        page.summary.first().map(|s| balance(&s.cell)),
        Some((BalanceSide::Zero, 0))
    );
    // The report equals the services trial balance to the rupee.
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(page.totals.get(1).map(money), Some(trial.total_dr.rupees()));
    assert_eq!(page.totals.get(2).map(money), Some(trial.total_cr.rupees()));
    for (row, trial_row) in page.rows.iter().zip(&trial.rows) {
        let (dr, cr) = match trial_row.balance.side {
            BalanceSide::Dr => (trial_row.balance.amount_paisa.rupees(), 0),
            BalanceSide::Cr => (0, trial_row.balance.amount_paisa.rupees()),
            BalanceSide::Zero => (0, 0),
        };
        assert_eq!(
            (
                money(row.cells.get(1).unwrap()),
                money(row.cells.get(2).unwrap())
            ),
            (dr, cr)
        );
    }
}

#[test]
fn the_totals_of_a_page_are_over_the_whole_filter_not_the_page() {
    let w = full_day();
    let first = report(&w, ReportId::R18, ReportFilters::default(), 0, 5);
    let second = report(&w, ReportId::R18, ReportFilters::default(), 5, 5);
    assert_eq!(first.rows.len(), 5);
    assert_eq!(second.rows.len(), 5);
    assert_eq!(first.total_count, 17);
    assert_eq!(first.totals, second.totals);
    assert_eq!(first.totals.get(1).map(money), Some(85_048));
    // The sum of the page rows is not the total.
    let page_dr: i64 = first
        .rows
        .iter()
        .map(|r| money(r.cells.get(1).unwrap()))
        .sum();
    assert_ne!(page_dr, 85_048);
    // A page past the end is empty with the same totals.
    let past = report(&w, ReportId::R18, ReportFilters::default(), 100, 5);
    assert!(past.rows.is_empty());
    assert_eq!(past.totals, first.totals);
}

#[test]
fn r18_as_of_a_day_leaves_out_later_postings() {
    let w = full_day();
    // On 09-10-26 only the openings and the advance exist: 65,000 = 65,000 (E2E-19).
    let filters = ReportFilters {
        as_of: "09-10-2026".into(),
        ..ReportFilters::default()
    };
    let page = report(&w, ReportId::R18, filters, 0, 0);
    assert_eq!(page.totals.get(1).map(money), Some(65_000));
    assert_eq!(page.totals.get(2).map(money), Some(65_000));
    assert_eq!(page.filters.as_of.as_deref(), Some("2026-10-09"));
}

#[test]
fn r13_for_akram_is_e2e_33() {
    let w = full_day();
    let page = report(&w, ReportId::R13, party(w.akram), 0, 0);
    assert_eq!(page.total_count, 2);
    // Opening 0.
    assert_eq!(
        page.summary.first().map(|s| balance(&s.cell)),
        Some((BalanceSide::Zero, 0))
    );
    let sale = page.rows.first().unwrap();
    assert_eq!(
        sale.cells.first(),
        Some(&Cell::Date {
            iso: "2026-10-10".into()
        })
    );
    // بکری 1.
    assert_eq!(
        sale.cells.get(1),
        Some(&Cell::Doc {
            type_id: "report-doc-bikri-first".into(),
            number: Some(1)
        })
    );
    // کیلا 20 پیٹی @ 600 = 12,000، کمیشن 720، لاگا 160, then 12,880 بنام.
    match sale.cells.get(2) {
        Some(Cell::Detail { parts }) => match parts.first() {
            Some(DetailPart::Sale {
                bikri_number,
                item,
                packing,
                qty,
                rate_paisa,
                value_paisa,
                commission_paisa,
                laga_paisa,
            }) => {
                assert_eq!(*bikri_number, 1);
                assert_eq!((item.en.as_str(), packing.en.as_str()), ("Banana", "Box"));
                assert_eq!(*qty, 20);
                assert_eq!(rate_paisa.rupees(), 600);
                assert_eq!(value_paisa.rupees(), 12_000);
                assert_eq!(commission_paisa.rupees(), 720);
                assert_eq!(laga_paisa.rupees(), 160);
            }
            other => panic!("not a sale detail: {other:?}"),
        },
        other => panic!("no detail: {other:?}"),
    }
    assert_eq!(money(sale.cells.get(4).unwrap()), 12_880);
    assert_eq!(
        balance(sale.cells.get(6).unwrap()),
        (BalanceSide::Dr, 12_880)
    );
    assert_eq!(sale.source.map(|s| s.id), Some(1));
    // The receipt: 10,000 جمع, balance 2,880 بنام.
    let receipt = page.rows.get(1).unwrap();
    assert_eq!(money(receipt.cells.get(5).unwrap()), 10_000);
    assert_eq!(
        balance(receipt.cells.get(6).unwrap()),
        (BalanceSide::Dr, 2_880)
    );
    assert_eq!(
        receipt.cells.get(1),
        Some(&Cell::Doc {
            type_id: "report-doc-receipt".into(),
            number: Some(1)
        })
    );
    // Totals row and closing: Σ بنام 12,880, Σ جمع 10,000, closing 2,880 بنام.
    assert_eq!(page.totals.get(4).map(money), Some(12_880));
    assert_eq!(page.totals.get(5).map(money), Some(10_000));
    assert_eq!(
        page.totals.get(6).map(balance),
        Some((BalanceSide::Dr, 2_880))
    );
    assert_eq!(
        page.summary.last().map(|s| balance(&s.cell)),
        Some((BalanceSide::Dr, 2_880))
    );
    // The closing balance is the services ledger's.
    let book = ledger::account_balance(&w.env.ctx, w.akram).unwrap();
    assert_eq!(
        (book.side, book.amount_paisa.rupees()),
        (BalanceSide::Dr, 2_880)
    );
}

#[test]
fn r13_pages_keep_the_running_balance_and_the_totals() {
    let w = full_day();
    let page = report(&w, ReportId::R13, party(w.akram), 1, 1);
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.total_count, 2);
    // The second line starts at line 2 but its balance still counts line 1.
    assert_eq!(
        balance(page.rows.first().unwrap().cells.get(6).unwrap()),
        (BalanceSide::Dr, 2_880)
    );
    assert_eq!(page.totals.get(4).map(money), Some(12_880));
    assert_eq!(page.totals.get(5).map(money), Some(10_000));
}

#[test]
fn r13_with_a_first_day_starts_from_the_balance_before_it() {
    let w = full_day();
    // رحیم's advance of 01-09-26 is before 10-10-26: the opening balance of that day.
    let filters = ReportFilters {
        party: Some(PartyFilter {
            account_id: w.rahim,
            person_key: None,
        }),
        from: "10-10-2026".into(),
        ..ReportFilters::default()
    };
    let page = report(&w, ReportId::R13, filters, 0, 0);
    assert_eq!(
        page.summary.first().map(|s| balance(&s.cell)),
        Some((BalanceSide::Dr, 15_000))
    );
    // 10-10-26: the credit 24,316, the cash 5,000 and the payment 4,316; back to 0.
    let sums: Vec<(i64, i64)> = page
        .rows
        .iter()
        .map(|r| {
            (
                money(r.cells.get(4).unwrap()),
                money(r.cells.get(5).unwrap()),
            )
        })
        .collect();
    assert_eq!(sums, vec![(0, 24_316), (5_000, 0), (4_316, 0)]);
    let balances: Vec<(BalanceSide, i64)> = page
        .rows
        .iter()
        .map(|r| balance(r.cells.get(6).unwrap()))
        .collect();
    assert_eq!(
        balances,
        vec![
            (BalanceSide::Cr, 9_316),
            (BalanceSide::Cr, 4_316),
            (BalanceSide::Zero, 0)
        ]
    );
    // The seller credit shows its lot: کیلا 50 پیٹی, خام 29,400, deductions 29,400 - 24,316.
    match page.rows.first().and_then(|r| r.cells.get(2)) {
        Some(Cell::Detail { parts }) => match parts.first() {
            Some(DetailPart::Lot {
                bikri_number,
                item,
                qty,
                gross_paisa,
                deductions_paisa,
                ..
            }) => {
                assert_eq!((*bikri_number, item.en.as_str(), *qty), (1, "Banana", 50));
                assert_eq!(gross_paisa.rupees(), 29_400);
                assert_eq!(deductions_paisa.rupees(), 5_084);
            }
            other => panic!("not a lot detail: {other:?}"),
        },
        other => panic!("no detail: {other:?}"),
    }
}

#[test]
fn r13_shows_the_second_sale_line_of_the_same_buyer_with_its_own_items() {
    // Two lots of one بکری, each with a line for اکرم: the pairing is by the order of his debit
    // lines, so each line gets its own lot's item and rate.
    let w = worked_day();
    let draft = support::draft(vec![
        support::with_sales(
            support::lot(&w, "lot-1", w.rahim, w.banana, "10"),
            vec![support::sale("s-1", w.akram, "10", "600")],
        ),
        support::with_sales(
            support::lot(&w, "lot-2", w.karim, w.apple, "5"),
            vec![support::sale("s-2", w.akram, "5", "1200")],
        ),
    ]);
    save(&w, &draft);
    let page = report(&w, ReportId::R13, party(w.akram), 0, 0);
    let parts: Vec<(String, i64, i64)> = page
        .rows
        .iter()
        .map(|r| match r.cells.get(2) {
            Some(Cell::Detail { parts }) => match parts.first() {
                Some(DetailPart::Sale {
                    item,
                    qty,
                    rate_paisa,
                    ..
                }) => (item.en.clone(), *qty, rate_paisa.rupees()),
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        parts,
        vec![
            ("Banana".to_owned(), 10, 600),
            ("Apple".to_owned(), 5, 1_200)
        ]
    );
}

#[test]
fn r13_after_a_cancel_shows_the_reversal_with_the_same_items() {
    let w = worked_day();
    let one = save(&w, &support::event1(&w));
    let detail = bikri::get(&w.env.ctx, one.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: one.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let page = report(&w, ReportId::R13, party(w.akram), 0, 0);
    assert_eq!(page.total_count, 2);
    let reversal = page.rows.get(1).unwrap();
    assert_eq!(money(reversal.cells.get(5).unwrap()), 12_880);
    assert_eq!(
        reversal.cells.get(1),
        Some(&Cell::Doc {
            type_id: "report-doc-reversal".into(),
            number: Some(1)
        })
    );
    assert!(matches!(
        reversal.cells.get(2),
        Some(Cell::Detail { parts }) if matches!(parts.first(), Some(DetailPart::Sale { qty: 20, .. }))
    ));
    assert_eq!(
        balance(reversal.cells.get(6).unwrap()),
        (BalanceSide::Zero, 0)
    );
}

#[test]
fn r14_is_the_statement_balance_only_or_with_detail() {
    let w = full_day();
    let page = report(&w, ReportId::R14, party(w.akram), 0, 0);
    // Balance only: no lines, four summary lines: opening, bought, paid, closing.
    assert!(page.rows.is_empty());
    assert_eq!(page.total_count, 0);
    let labels: Vec<&str> = page.summary.iter().map(|s| s.label_id.as_str()).collect();
    assert_eq!(
        labels,
        vec![
            "report-opening-balance",
            "report-statement-bought",
            "report-statement-paid",
            "report-closing-balance"
        ]
    );
    assert_eq!(page.summary.get(1).map(|s| money(&s.cell)), Some(12_880));
    assert_eq!(page.summary.get(2).map(|s| money(&s.cell)), Some(10_000));
    assert_eq!(
        page.summary.get(3).map(|s| balance(&s.cell)),
        Some((BalanceSide::Dr, 2_880))
    );
    // A seller: sold, then paid.
    let rahim = report(&w, ReportId::R14, party(w.rahim), 0, 0);
    let labels: Vec<&str> = rahim.summary.iter().map(|s| s.label_id.as_str()).collect();
    assert_eq!(labels.get(1), Some(&"report-statement-sold"));
    assert_eq!(rahim.summary.get(1).map(|s| money(&s.cell)), Some(24_316));
    // Paid to him: the advance 15,000, the cash 5,000 and the payment 4,316.
    assert_eq!(rahim.summary.get(2).map(|s| money(&s.cell)), Some(24_316));
    assert_eq!(
        rahim.summary.get(3).map(|s| balance(&s.cell)),
        Some((BalanceSide::Zero, 0))
    );
    // With detail it carries the lines of R13.
    let with_detail = report(
        &w,
        ReportId::R14,
        ReportFilters {
            detail: true,
            ..party(w.akram)
        },
        0,
        0,
    );
    assert_eq!(with_detail.rows.len(), 2);
    assert_eq!(with_detail.total_count, 2);
    assert_eq!(with_detail.summary, page.summary);
}

#[test]
fn r11_is_the_daily_journal_and_its_two_sides_are_equal() {
    let w = full_day();
    let filters = ReportFilters {
        from: "10-10-2026".into(),
        to: "10-10-2026".into(),
        ..ReportFilters::default()
    };
    let all = report(&w, ReportId::R11, filters.clone(), 0, 1_000);
    // Entries of the day: بکری 1 (36,564), بکری 2 (25,600), وصولی 10,000, ادائیگی 4,316,
    // دکان خرچہ 3,000, ذاتی خرچہ 2,000 = 81,480 on each side.
    assert_eq!(all.totals.get(4).map(money), Some(81_480));
    assert_eq!(all.totals.get(5).map(money), Some(81_480));
    // Line count: 10-line sale part + 2-line cash part (Event 1 has 2 buyers: 2 + 1 + 6 + 2 ...)
    // is checked against the database rather than counted by hand.
    let in_db: i64 = raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE e.entry_date = '2026-10-10' AND e.status = 'posted'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(all.total_count, in_db);
    assert_eq!(i64::try_from(all.rows.len()).unwrap(), in_db);
    // Pages of 7 add up to the same rows and the same totals.
    let mut paged = Vec::new();
    let mut offset = 0;
    loop {
        let page = report(&w, ReportId::R11, filters.clone(), offset, 7);
        assert_eq!(page.totals, all.totals);
        assert_eq!(page.total_count, in_db);
        if page.rows.is_empty() {
            break;
        }
        offset += 7;
        paged.extend(page.rows);
    }
    assert_eq!(paged, all.rows);
    // Σ of every row equals the totals row.
    let dr: i64 = all
        .rows
        .iter()
        .map(|r| money(r.cells.get(4).unwrap()))
        .sum();
    let cr: i64 = all
        .rows
        .iter()
        .map(|r| money(r.cells.get(5).unwrap()))
        .sum();
    assert_eq!((dr, cr), (81_480, 81_480));
    // The first line is in entry order: بکری 1.
    assert_eq!(
        all.rows.first().and_then(|r| r.cells.get(1)),
        Some(&Cell::Doc {
            type_id: "report-doc-bikri-first".into(),
            number: Some(1)
        })
    );
    // The shortcut "today" is the same day.
    let today = ReportFilters {
        shortcut: Some(mandi_reports::reports::DateShortcut::Today),
        ..ReportFilters::default()
    };
    let by_shortcut = report(&w, ReportId::R11, today, 0, 1_000);
    assert_eq!(by_shortcut.rows, all.rows);
    assert_eq!(by_shortcut.filters.from.as_deref(), Some("2026-10-10"));
    // One party's lines only (FR-19.10): اکرم's two lines of the day.
    let one = report(
        &w,
        ReportId::R11,
        ReportFilters {
            party: party(w.akram).party,
            ..filters
        },
        0,
        0,
    );
    assert_eq!(one.total_count, 2);
    assert_eq!(one.totals.get(4).map(money), Some(12_880));
    assert_eq!(one.totals.get(5).map(money), Some(10_000));
}

#[test]
fn r16_lists_accounts_and_its_sides_equal_the_trial_balance() {
    let w = full_day();
    let page = report(&w, ReportId::R16, ReportFilters::default(), 0, 0);
    assert_eq!(page.totals.get(2).map(money), Some(85_048));
    assert_eq!(page.totals.get(3).map(money), Some(85_048));
    // The accounts come in code order, and one with no line is listed with no balance.
    let codes: Vec<String> = page
        .rows
        .iter()
        .map(|r| code_of(r.cells.first().unwrap()))
        .collect();
    let mut sorted = codes.clone();
    sorted.sort();
    assert_eq!(codes, sorted);
    assert!(
        codes.contains(&"01020001".to_owned()),
        "an account with no lines is listed"
    );
    let akram = page
        .rows
        .iter()
        .find(|r| code_of(r.cells.first().unwrap()) == "03040001")
        .unwrap();
    assert_eq!(money(akram.cells.get(2).unwrap()), 2_880);
    // The short code of a buyer is his serial.
    match akram.cells.first() {
        Some(Cell::Party { party }) => assert_eq!(party.short_code.as_deref(), Some("1")),
        other => panic!("{other:?}"),
    }
    // A city that no account has gives no rows.
    let empty = rejected(
        &w,
        ReportId::R16,
        ReportFilters {
            city_id: Some(9_999),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(
        empty,
        vec![(ErrorCode::ErrNotFound, "filters.city_id".to_owned())]
    );
}

#[test]
fn r17_head_balances_are_net_and_the_two_sides_are_equal() {
    let w = full_day();
    let page = report(&w, ReportId::R17, ReportFilters::default(), 0, 0);
    // Heads (net of all their lines): assets 58,484 بنام; payables 4,400 جمع; parties 21,564
    // بنام less کریم's 10,160 جمع = 11,404 بنام; income 5,488 جمع; expenses 3,000 بنام; capital
    // 65,000 جمع less الف's 2,000 drawings = 63,000 جمع. Σ بنام = Σ جمع = 72,888.
    let heads: Vec<(String, i64, i64)> = page
        .rows
        .iter()
        .filter(|r| r.kind == mandi_reports::reports::RowKind::Group)
        .map(|r| {
            let code = match r.cells.first() {
                Some(Cell::Text { text }) => text.clone(),
                other => panic!("{other:?}"),
            };
            (
                code,
                money(r.cells.get(2).unwrap()),
                money(r.cells.get(3).unwrap()),
            )
        })
        .collect();
    assert_eq!(
        heads,
        vec![
            ("01".to_owned(), 58_484, 0),
            ("02".to_owned(), 0, 4_400),
            ("03".to_owned(), 11_404, 0),
            ("04".to_owned(), 0, 5_488),
            ("05".to_owned(), 3_000, 0),
            ("06".to_owned(), 0, 63_000),
        ]
    );
    assert_eq!(page.totals.get(2).map(money), Some(72_888));
    assert_eq!(page.totals.get(3).map(money), Some(72_888));
    // Controls sit under their head, and the sellers control (03011) nets رحیم 0 and کریم 10,160.
    let controls: Vec<(String, i64, i64)> = page
        .rows
        .iter()
        .filter(|r| r.kind == mandi_reports::reports::RowKind::Normal)
        .map(|r| {
            let code = match r.cells.first() {
                Some(Cell::Text { text }) => text.clone(),
                other => panic!("{other:?}"),
            };
            (
                code,
                money(r.cells.get(2).unwrap()),
                money(r.cells.get(3).unwrap()),
            )
        })
        .collect();
    assert!(controls.contains(&("03011".to_owned(), 0, 10_160)));
    assert!(controls.contains(&("0304".to_owned(), 21_564, 0)));
    assert!(controls.contains(&("0603".to_owned(), 2_000, 0)));
}

#[test]
fn r19_is_the_opening_trial_with_the_opening_difference() {
    let w = full_day();
    let page = report(&w, ReportId::R19, ReportFilters::default(), 0, 0);
    let rows: Vec<(String, i64, i64)> = page
        .rows
        .iter()
        .map(|r| {
            (
                code_of(r.cells.first().unwrap()),
                money(r.cells.get(2).unwrap()),
                money(r.cells.get(3).unwrap()),
            )
        })
        .collect();
    // E2E-19: روکڑ 65,000 بنام, ابتدائی سرمایہ 65,000 جمع, ابتدائی فرق 0.
    assert_eq!(
        rows,
        vec![
            ("01010001".to_owned(), 65_000, 0),
            ("06010001".to_owned(), 0, 65_000),
            ("06019999".to_owned(), 0, 0),
        ]
    );
    assert_eq!(page.totals.get(2).map(money), Some(65_000));
    assert_eq!(page.totals.get(3).map(money), Some(65_000));
    assert_eq!(
        page.rows.first().and_then(|r| r.cells.get(1)),
        Some(&Cell::Date {
            iso: "2026-08-31".into()
        })
    );
}

#[test]
fn r19_shows_a_wrong_opening_as_a_difference() {
    // Only one side typed: روکڑ 65,000 بنام and nothing else. ابتدائی فرق takes the other side.
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = mandi_services::auth::require(&env.ctx).unwrap();
    let cash = support::id_of(&env, "01010001");
    set_today(&env, 2026, 8, 31);
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: cash,
                version: accounts::get(&env.ctx, cash).unwrap().version,
                opening_date: "31-08-2026".into(),
                opening_dr: "65,000".into(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    let page = run(
        env.ctx.db(),
        &ReportRunInput {
            id: ReportId::R19,
            filters: ReportFilters::default(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let diff = page.rows.last().unwrap();
    assert_eq!(code_of(diff.cells.first().unwrap()), "06019999");
    assert_eq!(money(diff.cells.get(3).unwrap()), 65_000);
    // The difference account has the same balance in the trial balance of the books.
    let trial = ledger::trial_balance(&env.ctx, None).unwrap();
    let book = trial.rows.iter().find(|r| r.code == "06019999").unwrap();
    assert_eq!(
        (book.balance.side, book.balance.amount_paisa.rupees()),
        (BalanceSide::Cr, 65_000)
    );
    drop(dir);
}

#[test]
fn filters_are_checked_and_the_problems_come_together() {
    let w = full_day();
    // R13 needs a party.
    assert_eq!(
        rejected(&w, ReportId::R13, ReportFilters::default(), 0, 0),
        vec![(ErrorCode::ErrRequired, "filters.party".to_owned())]
    );
    // R18 takes no date range and no party.
    let got = rejected(
        &w,
        ReportId::R18,
        ReportFilters {
            from: "01-10-2026".into(),
            party: party(w.akram).party,
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(
        got,
        vec![
            (ErrorCode::ErrOutOfRange, "filters.from".to_owned()),
            (ErrorCode::ErrOutOfRange, "filters.party".to_owned())
        ]
    );
    // A date that is not a date, a range the wrong way round, an unknown account, a bad page.
    assert_eq!(
        rejected(
            &w,
            ReportId::R11,
            ReportFilters {
                from: "31-02-2026".into(),
                to: "abc".into(),
                ..ReportFilters::default()
            },
            0,
            0
        ),
        vec![
            (ErrorCode::ErrDateInvalid, "filters.from".to_owned()),
            (ErrorCode::ErrDateInvalid, "filters.to".to_owned())
        ]
    );
    assert_eq!(
        rejected(
            &w,
            ReportId::R11,
            ReportFilters {
                from: "11-10-2026".into(),
                to: "10-10-2026".into(),
                ..ReportFilters::default()
            },
            0,
            0
        ),
        vec![(ErrorCode::ErrDateRange, "filters.to".to_owned())]
    );
    assert_eq!(
        rejected(&w, ReportId::R13, party(9_999), 0, 0),
        vec![(
            ErrorCode::ErrNotFound,
            "filters.party.account_id".to_owned()
        )]
    );
    assert_eq!(
        rejected(
            &w,
            ReportId::R13,
            ReportFilters {
                party: Some(PartyFilter {
                    account_id: w.akram,
                    person_key: Some("x".into())
                }),
                ..ReportFilters::default()
            },
            0,
            0
        ),
        vec![(
            ErrorCode::ErrOutOfRange,
            "filters.party.person_key".to_owned()
        )]
    );
    assert_eq!(
        rejected(&w, ReportId::R18, ReportFilters::default(), -1, 1_001),
        vec![
            (ErrorCode::ErrOutOfRange, "offset".to_owned()),
            (ErrorCode::ErrOutOfRange, "limit".to_owned())
        ]
    );
}

#[test]
fn a_report_writes_nothing() {
    let w = full_day();
    let count = |sql: &str| -> i64 { raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap() };
    let rows = || {
        (
            count("SELECT COUNT(*) FROM journal_entries"),
            count("SELECT COUNT(*) FROM journal_lines"),
            count("SELECT COUNT(*) FROM audit_log"),
            count("SELECT COUNT(*) FROM bill_allocations"),
        )
    };
    let start = rows();
    for id in [
        ReportId::R11,
        ReportId::R16,
        ReportId::R17,
        ReportId::R18,
        ReportId::R19,
    ] {
        report(&w, id, ReportFilters::default(), 0, 0);
    }
    report(&w, ReportId::R13, party(w.akram), 0, 0);
    report(&w, ReportId::R14, party(w.akram), 0, 0);
    assert_eq!(rows(), start);
}

// --- Print data -------------------------------------------------------------------------------

fn print(
    w: &support::World,
    id: ReportId,
    filters: ReportFilters,
    language: Language,
) -> mandi_reports::reports::ReportPrintData {
    print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id,
            filters,
            language,
            paper: None,
        },
    )
    .unwrap()
}

fn text_of(cell: &PrintCell) -> String {
    match cell {
        PrintCell::Money { text } | PrintCell::Number { text } | PrintCell::Date { text } => {
            text.clone()
        }
        PrintCell::Balance { text, .. } => text.clone(),
        PrintCell::Empty => String::new(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn r18_prints_every_row_with_the_totals_in_both_languages() {
    let w = full_day();
    for language in [Language::Ur, Language::En] {
        let data = print(&w, ReportId::R18, ReportFilters::default(), language);
        assert_eq!(data.rows.len(), 17);
        assert_eq!(data.row_count, "17");
        assert_eq!(data.totals.get(1).map(text_of).as_deref(), Some("85,048"));
        assert_eq!(data.totals.get(2).map(text_of).as_deref(), Some("85,048"));
        assert_eq!(data.title_id, "report-r18-title");
        assert_eq!(data.footer_id, "print-page-label");
        assert_eq!(
            data.firm.name,
            match language {
                Language::Ur => "ماہر عدنان اینڈ برادرز",
                Language::En => "Maher Adnan and Brothers",
            }
        );
        assert!(data.words.is_none());
        // A party is shown with its short code and name in the print language.
        match data.rows.iter().find_map(|r| match r.cells.first() {
            Some(PrintCell::Party { party }) if party.code == "03040001" => Some(party.clone()),
            _ => None,
        }) {
            Some(party) => {
                assert_eq!(party.short_code.as_deref(), Some("1"));
                assert_eq!(
                    party.name,
                    match language {
                        Language::Ur => "اکرم ٹول ٹیکس",
                        Language::En => "Akram Toll Tax",
                    }
                );
            }
            None => panic!("no row for 03040001"),
        }
    }
}

#[test]
fn r13_prints_the_items_the_filters_and_the_closing_in_words() {
    let w = full_day();
    let ur = print(&w, ReportId::R13, party(w.akram), Language::Ur);
    let en = print(&w, ReportId::R13, party(w.akram), Language::En);
    assert_eq!(ur.rows.len(), 2);
    // The header lists the party filter (FR-19.10).
    assert_eq!(en.filters.len(), 1);
    assert_eq!(
        en.filters.first().map(|f| f.label_id.as_str()),
        Some("report-filter-party")
    );
    assert_eq!(
        en.filters.first().map(|f| f.value.as_str()),
        Some("1 Akram Toll Tax")
    );
    // The sale line, formatted: Banana, Box, 20, 600, 12,000, 720, 160.
    match en.rows.first().and_then(|r| r.cells.get(2)) {
        Some(PrintCell::Detail { parts }) => match parts.first() {
            Some(PrintDetailPart::Sale {
                item,
                packing,
                qty,
                rate,
                value,
                commission,
                laga,
                ..
            }) => {
                assert_eq!(
                    (item.as_str(), packing.as_str(), qty.as_str()),
                    ("Banana", "Box", "20")
                );
                assert_eq!(
                    (
                        rate.as_str(),
                        value.as_str(),
                        commission.as_str(),
                        laga.as_str()
                    ),
                    ("600", "12,000", "720", "160")
                );
            }
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    }
    match ur.rows.first().and_then(|r| r.cells.get(2)) {
        Some(PrintCell::Detail { parts }) => match parts.first() {
            Some(PrintDetailPart::Sale { item, .. }) => assert_eq!(item, "کیلا"),
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    }
    assert_eq!(en.totals.get(4).map(text_of).as_deref(), Some("12,880"));
    assert_eq!(en.totals.get(5).map(text_of).as_deref(), Some("10,000"));
    assert_eq!(en.totals.get(6).map(text_of).as_deref(), Some("2,880"));
    // The closing balance in words (FR-19.7).
    assert_eq!(
        en.words.as_ref().map(|w| w.text.as_str()),
        Some("Two thousand eight hundred eighty rupees only")
    );
    assert_eq!(
        ur.words.as_ref().map(|w| w.text.clone()),
        Some(mandi_services::core::urdu_words::urdu_words(rupees(2_880)).unwrap())
    );
    assert_eq!(en.dir, mandi_reports::print::TextDir::Ltr);
    assert_eq!(ur.dir, mandi_reports::print::TextDir::Rtl);
    // A print of a balance-only statement says so in its header.
    let statement = print(&w, ReportId::R14, party(w.akram), Language::En);
    assert!(statement.rows.is_empty());
    assert!(
        statement
            .filters
            .iter()
            .any(|f| f.label_id == "report-filter-balance-only")
    );
    assert_eq!(statement.summary.len(), 4);
}

#[test]
fn a_print_with_a_date_range_lists_both_dates_as_dd_mm_yyyy() {
    let w = full_day();
    let data = print(
        &w,
        ReportId::R11,
        ReportFilters {
            from: "10-10-2026".into(),
            to: "2026-10-10".into(),
            ..ReportFilters::default()
        },
        Language::En,
    );
    let facts: Vec<(&str, &str)> = data
        .filters
        .iter()
        .map(|f| (f.label_id.as_str(), f.value.as_str()))
        .collect();
    assert_eq!(
        facts,
        vec![
            ("report-filter-from", "10-10-2026"),
            ("report-filter-to", "10-10-2026")
        ]
    );
    assert_eq!(data.totals.get(4).map(text_of).as_deref(), Some("81,480"));
}

// --- More worked checks ------------------------------------------------------------------------

#[test]
fn r13_after_an_edit_shows_the_old_posting_its_reversal_and_the_new_posting() {
    use mandi_services::core::bikri::{EditOpenInput, EditSaveInput};
    let w = worked_day();
    let saved = save(&w, &support::event1(&w));
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
        },
    )
    .unwrap();
    // E2E-13: اکرم's rate 600 to 610.
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
    let page = report(&w, ReportId::R13, party(w.akram), 0, 0);
    // The old posting (12,880), its reversal (12,880), the new posting (13,092).
    assert_eq!(page.total_count, 3);
    let sale = |row: usize| -> (i64, i64, i64, i64) {
        match page.rows.get(row).and_then(|r| r.cells.get(2)) {
            Some(Cell::Detail { parts }) => match parts.first() {
                Some(DetailPart::Sale {
                    rate_paisa,
                    value_paisa,
                    commission_paisa,
                    laga_paisa,
                    ..
                }) => (
                    rate_paisa.rupees(),
                    value_paisa.rupees(),
                    commission_paisa.rupees(),
                    laga_paisa.rupees(),
                ),
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(sale(0), (600, 12_000, 720, 160));
    assert_eq!(sale(1), (600, 12_000, 720, 160));
    assert_eq!(sale(2), (610, 12_200, 732, 160));
    let balances: Vec<(BalanceSide, i64)> = page
        .rows
        .iter()
        .map(|r| balance(r.cells.get(6).unwrap()))
        .collect();
    assert_eq!(
        balances,
        vec![
            (BalanceSide::Dr, 12_880),
            (BalanceSide::Zero, 0),
            (BalanceSide::Dr, 13_092)
        ]
    );
    assert_eq!(
        page.totals.get(6).map(balance),
        Some((BalanceSide::Dr, 13_092))
    );
}

/// The "bought or sold" and "paid" lines of R14 as `(label, rupees)`.
fn statement_lines(page: &mandi_reports::reports::ReportPage) -> (i64, i64) {
    (
        page.summary.get(1).map(|s| money(&s.cell)).unwrap(),
        page.summary.get(2).map(|s| money(&s.cell)).unwrap(),
    )
}

#[test]
fn r14_after_an_edit_counts_only_the_live_bill_e2e_13() {
    use mandi_services::core::bikri::{EditOpenInput, EditSaveInput};
    let w = worked_day();
    let saved = save(&w, &support::event1(&w));
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: saved.bikri_id,
            entry_id: saved.entry_id,
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
    mandi_services::vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000"))
        .unwrap();

    // اکرم: the old bill 12,880 and its reversal are left out. He bought 13,092 and paid 10,000;
    // 13,092 - 10,000 = 3,092 بنام. (The ledger R13 still lists all three bill lines.)
    let akram = report(&w, ReportId::R14, party(w.akram), 0, 0);
    assert_eq!(statement_lines(&akram), (13_092, 10_000));
    assert_eq!(
        akram.summary.get(3).map(|s| balance(&s.cell)),
        Some((BalanceSide::Dr, 3_092))
    );
    // The same figures with a date range and with detail on.
    let ranged = report(
        &w,
        ReportId::R14,
        ReportFilters {
            from: "10-10-2026".into(),
            to: "10-10-2026".into(),
            detail: true,
            ..party(w.akram)
        },
        0,
        0,
    );
    assert_eq!(statement_lines(&ranged), (13_092, 10_000));
    // R13 keeps its totals: it shows every line, the reversal's credit too. Debit 12,880 + 13,092
    // = 25,972; credit 12,880 (reversal) + 10,000 = 22,880. These are the figures R14 used to
    // show wrongly.
    let ledger_page = report(&w, ReportId::R13, party(w.akram), 0, 0);
    assert_eq!(ledger_page.totals.get(4).map(money), Some(25_972));
    assert_eq!(ledger_page.totals.get(5).map(money), Some(22_880));

    // A seller the same way: the seller credit is now 29,600 - 1,776 - 3,000 - 250 - 50 - 20 =
    // 24,504 (not 24,316 + 24,504). Paid to him: the advance 15,000 and the cash 5,000.
    let rahim = report(&w, ReportId::R14, party(w.rahim), 0, 0);
    assert_eq!(
        rahim
            .summary
            .get(1)
            .map(|s| (s.label_id.as_str(), money(&s.cell))),
        Some(("report-statement-sold", 24_504))
    );
    assert_eq!(rahim.summary.get(2).map(|s| money(&s.cell)), Some(20_000));
    assert_eq!(
        rahim.summary.get(3).map(|s| balance(&s.cell)),
        Some((BalanceSide::Cr, 4_504))
    );
}

#[test]
fn r14_after_a_cancel_leaves_out_the_cancelled_bill_and_keeps_the_closing_balance() {
    let w = worked_day();
    let one = save(&w, &support::event1(&w));
    mandi_services::vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000"))
        .unwrap();
    let before = report(&w, ReportId::R14, party(w.akram), 0, 0);
    assert_eq!(statement_lines(&before), (12_880, 10_000));
    let detail = bikri::get(&w.env.ctx, one.bikri_id).unwrap();
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: one.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    // Nothing is bought now; he still paid 10,000, so he has 10,000 جمع. The pair of the bill and
    // its reversal changes the closing balance by 0 (12,880 - 12,880).
    let after = report(&w, ReportId::R14, party(w.akram), 0, 0);
    assert_eq!(statement_lines(&after), (0, 10_000));
    assert_eq!(
        after.summary.get(3).map(|s| balance(&s.cell)),
        Some((BalanceSide::Cr, 10_000))
    );
    // A seller: his sold line is 0 after the cancel. The cash part went back with the bill (one
    // entry), so only the advance of 15,000 is paid; the closing balance is 15,000 بنام.
    let rahim = report(&w, ReportId::R14, party(w.rahim), 0, 0);
    assert_eq!(statement_lines(&rahim), (0, 15_000));
    assert_eq!(
        rahim.summary.get(3).map(|s| balance(&s.cell)),
        Some((BalanceSide::Dr, 15_000))
    );
}

#[test]
fn r13_of_a_one_time_buyer_is_by_person() {
    let w = full_day();
    let key: String = raw(&w.dir)
        .query_row(
            "SELECT pukhta_person_key FROM journal_lines
             WHERE account_id = ?1 AND pukhta_person_key IS NOT NULL LIMIT 1",
            [w.pukhta_buyer],
            |r| r.get(0),
        )
        .unwrap();
    let person = ReportFilters {
        party: Some(PartyFilter {
            account_id: w.pukhta_buyer,
            person_key: Some(key.clone()),
        }),
        ..ReportFilters::default()
    };
    let page = report(&w, ReportId::R13, person, 0, 0);
    // Shahid bought for 12,800 and paid it on the spot (the cash part): balance 0.
    assert_eq!(page.total_count, 2);
    assert_eq!(page.totals.get(4).map(money), Some(12_800));
    assert_eq!(page.totals.get(5).map(money), Some(12_800));
    assert_eq!(
        page.totals.get(6).map(balance),
        Some((BalanceSide::Zero, 0))
    );
    match page.rows.first().and_then(|r| r.cells.get(2)) {
        Some(Cell::Detail { parts }) => match parts.first() {
            Some(DetailPart::Sale { item, qty, .. }) => {
                assert_eq!((item.en.as_str(), *qty), ("Apple", 10));
            }
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    }
    // The whole account shows the person on each line.
    let whole = report(&w, ReportId::R13, party(w.pukhta_buyer), 0, 0);
    assert_eq!(whole.total_count, 2);
    match whole.rows.first().and_then(|r| r.cells.get(2)) {
        Some(Cell::Detail { parts }) => {
            assert_eq!(parts.first(), Some(&DetailPart::Text { text: key }));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn every_account_ledger_closes_on_the_book_balance() {
    // A report total must equal the matching ledger balance (money agent Section 9): for every
    // account of the worked day, R13 closes where the services balance stands, and its totals
    // are the sums of its lines.
    let w = full_day();
    let ids: Vec<i64> = {
        let conn = raw(&w.dir);
        let mut statement = conn.prepare("SELECT id FROM accounts ORDER BY id").unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    for id in ids {
        let page = report(&w, ReportId::R13, party(id), 0, 0);
        let book = ledger::account_balance(&w.env.ctx, id).unwrap();
        assert_eq!(
            page.totals.get(6).map(balance),
            Some((book.side, book.amount_paisa.rupees())),
            "account {id}"
        );
        let dr: i64 = page
            .rows
            .iter()
            .map(|r| money(r.cells.get(4).unwrap()))
            .sum();
        let cr: i64 = page
            .rows
            .iter()
            .map(|r| money(r.cells.get(5).unwrap()))
            .sum();
        assert_eq!(
            (page.totals.get(4).map(money), page.totals.get(5).map(money)),
            (Some(dr), Some(cr)),
            "account {id}"
        );
        // The last line's balance is the closing balance.
        if let Some(last) = page.rows.last() {
            assert_eq!(
                Some(balance(last.cells.get(6).unwrap())),
                page.totals.get(6).map(balance),
                "account {id}"
            );
        }
    }
}

#[test]
fn the_whole_journal_balances_in_r11_for_all_time() {
    let w = full_day();
    let page = report(&w, ReportId::R11, ReportFilters::default(), 0, 1);
    assert_eq!(page.totals.get(4), page.totals.get(5));
    // 01-09-26 advance 15,000 and the two openings 65,000 each way are in it too.
    assert!(page.totals.get(4).map(money) > Some(81_480));
}

// --- Fixtures for the frontend -------------------------------------------------------------------
//
// Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is read
// back into the real input type before it is used) and `<name>.json` is the answer the real engine
// gives for it on the PRD worked day. A refusal has the shape of the error a command returns
// (`ok`, `errors`, `ref`), where `ref` is a stand-in. The frontend copies the files. The test fails
// if a file differs from the real answer, so a fixture can never go stale; after a deliberate
// change, write them again with
// `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-services --test reports_accounts fixtures`.

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

    /// A page of a report for a request, as a pair.
    fn run_pair(w: &support::World, name: &str, request: Value) {
        let input: ReportRunInput = typed(&request);
        let page = run(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &page);
    }

    fn print_pair(w: &support::World, name: &str, request: Value) {
        let input: ReportPrintInput = typed(&request);
        let data = print_data(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &data);
    }

    /// The slips of R5 "Print All" (round 12).
    fn slips_pair(w: &support::World, name: &str, request: Value) {
        let input: ReportPrintInput = typed(&request);
        let data = buyer_bills_print_data(w.env.ctx.db(), &input).unwrap();
        pair(name, &request, &data);
    }

    /// A refusal as a command returns it.
    fn refusal_pair(w: &support::World, name: &str, request: Value, code: ErrorCode) {
        let input: ReportRunInput = typed(&request);
        let error = run(w.env.ctx.db(), &input).unwrap_err();
        let mandi_reports::ReportsError::Rejected(items) = error else {
            panic!("not a refusal of the user's entry: {error:?}");
        };
        assert_eq!(items.first().map(|i| i.code), Some(code), "{name}");
        let wire = json!({
            "ok": false,
            "errors": items
                .iter()
                .map(|i| json!({ "code": i.code, "field": i.field, "values": i.values }))
                .collect::<Vec<_>>(),
            "ref": "E-20261010-0001"
        });
        pair(name, &request, &wire);
    }

    /// The filters with every key, as the frontend sends them.
    fn filters(extra: Value) -> Value {
        let mut base = json!({
            "shortcut": null, "from": "", "to": "", "as_of": "", "party": null,
            "seller_id": null, "zamindar_id": null, "item_id": null, "city_id": null,
            "adda_id": null, "detail": false
        });
        // Every key of the generated type, so the request is the exact input of the command.
        if let Some(map) = base.as_object_mut() {
            for key in ["partner_id", "garden_id", "bikri_id"] {
                map.entry(key).or_insert(Value::Null);
            }
        }
        if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
            for (k, v) in extra {
                base.insert(k.clone(), v.clone());
            }
        }
        base
    }

    /// R36 in both layouts, its print data and the day summary of the sale day, on the garden of
    /// E2E-40 closed on 13-10-26.
    fn garden_fixtures() {
        let (w, garden) = garden_40();
        set_today(&w.env, 2026, 10, 13);
        garden_close(&w, garden.id, "13-10-2026");
        run_pair(
            &w,
            "reports_run_r36_garden",
            json!({ "id": "R36", "filters": filters(json!({})), "offset": 0, "limit": 0 }),
        );
        run_pair(
            &w,
            "reports_run_r36_partner",
            json!({ "id": "R36", "filters": filters(json!({ "garden_view": "partner" })), "offset": 0, "limit": 0 }),
        );
        print_pair(
            &w,
            "reports_print_data_r36_partner_en",
            json!({ "id": "R36", "filters": filters(json!({ "garden_id": garden.id, "garden_view": "partner" })), "language": "en", "paper": null }),
        );
        run_pair(
            &w,
            "reports_run_r34_garden_day",
            json!({ "id": "R34", "filters": filters(json!({ "from": "12-10-2026", "to": "12-10-2026" })), "offset": 0, "limit": 0 }),
        );
    }

    #[test]
    fn fixtures_are_the_real_answers() {
        let w = full_day();
        let party = |id: i64| json!({ "account_id": id, "person_key": null });

        // The catalogue: the command takes no input.
        write_or_check(
            "reports_catalog.json",
            &(serde_json::to_string_pretty(&catalog()).unwrap() + "\n"),
        );

        run_pair(
            &w,
            "reports_run_r18_page1",
            json!({ "id": "R18", "filters": filters(json!({})), "offset": 0, "limit": 10 }),
        );
        run_pair(
            &w,
            "reports_run_r18_page2",
            json!({ "id": "R18", "filters": filters(json!({})), "offset": 10, "limit": 10 }),
        );
        run_pair(
            &w,
            "reports_run_r18_as_of_09_10",
            json!({ "id": "R18", "filters": filters(json!({ "as_of": "09-10-2026" })), "offset": 0, "limit": 0 }),
        );
        run_pair(
            &w,
            "reports_run_r13_akram",
            json!({ "id": "R13", "filters": filters(json!({ "party": party(w.akram) })), "offset": 0, "limit": 0 }),
        );
        run_pair(
            &w,
            "reports_run_r13_rahim_from_10_10",
            json!({ "id": "R13", "filters": filters(json!({ "party": party(w.rahim), "from": "10-10-2026" })), "offset": 0, "limit": 100 }),
        );
        run_pair(
            &w,
            "reports_run_r14_akram_balance_only",
            json!({ "id": "R14", "filters": filters(json!({ "party": party(w.akram) })), "offset": 0, "limit": 0 }),
        );
        run_pair(
            &w,
            "reports_run_r14_akram_detail",
            json!({ "id": "R14", "filters": filters(json!({ "party": party(w.akram), "detail": true })), "offset": 0, "limit": 0 }),
        );
        run_pair(
            &w,
            "reports_run_r11_today",
            json!({ "id": "R11", "filters": filters(json!({ "shortcut": "today" })), "offset": 0, "limit": 12 }),
        );
        run_pair(
            &w,
            "reports_run_r16_page1",
            json!({ "id": "R16", "filters": filters(json!({})), "offset": 0, "limit": 8 }),
        );
        run_pair(
            &w,
            "reports_run_r17",
            json!({ "id": "R17", "filters": filters(json!({})), "offset": 0, "limit": 0 }),
        );
        run_pair(
            &w,
            "reports_run_r19",
            json!({ "id": "R19", "filters": filters(json!({})), "offset": 0, "limit": 0 }),
        );
        // Round 11: the party day sheet (R35), the table of all parties and two slips.
        run_pair(
            &w,
            "reports_run_r35_table",
            json!({ "id": "R35", "filters": filters(json!({ "as_of": "10-10-2026" })), "offset": 0, "limit": 0 }),
        );
        run_pair(
            &w,
            "reports_run_r35_slip_akram",
            json!({ "id": "R35", "filters": filters(json!({ "as_of": "10-10-2026", "party": party(w.akram) })), "offset": 0, "limit": 0 }),
        );
        run_pair(
            &w,
            "reports_run_r35_slip_rahim",
            json!({ "id": "R35", "filters": filters(json!({ "as_of": "10-10-2026", "party": party(w.rahim) })), "offset": 0, "limit": 0 }),
        );
        print_pair(
            &w,
            "reports_print_data_r35_slip_akram_en",
            json!({ "id": "R35", "filters": filters(json!({ "as_of": "10-10-2026", "party": party(w.akram) })), "language": "en", "paper": null }),
        );
        // Round 12: the sale book of بکری 1 and the buyer bills of the day.
        let first: i64 = w
            .env
            .ctx
            .db()
            .read(|conn| {
                conn.query_row("SELECT id FROM bikri_entries WHERE number = 1", [], |r| {
                    r.get(0)
                })
            })
            .unwrap();
        run_pair(
            &w,
            "reports_run_r1_bikri_1",
            json!({ "id": "R1", "filters": filters(json!({ "bikri_id": first })), "offset": 0, "limit": 0 }),
        );
        run_pair(
            &w,
            "reports_run_r5_credit_buyers",
            json!({ "id": "R5", "filters": filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })), "offset": 0, "limit": 0 }),
        );
        print_pair(
            &w,
            "reports_print_data_r5_en",
            json!({ "id": "R5", "filters": filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })), "language": "en", "paper": null }),
        );
        slips_pair(
            &w,
            "reports_buyer_bills_print_data_en",
            json!({ "id": "R5", "filters": filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })), "language": "en", "paper": null }),
        );
        slips_pair(
            &w,
            "reports_buyer_bills_print_data_ur",
            json!({ "id": "R5", "filters": filters(json!({ "from": "10-10-2026", "to": "10-10-2026" })), "language": "ur", "paper": "a4" }),
        );
        refusal_pair(
            &w,
            "reports_run_r1_needs_a_bikri",
            json!({ "id": "R1", "filters": filters(json!({})), "offset": 0, "limit": 0 }),
            ErrorCode::ErrRequired,
        );
        // Round 11: the gardens report and the day summary on the garden of E2E-40 (closed).
        garden_fixtures();
        refusal_pair(
            &w,
            "reports_run_r13_no_party",
            json!({ "id": "R13", "filters": filters(json!({})), "offset": 0, "limit": 0 }),
            ErrorCode::ErrRequired,
        );
        refusal_pair(
            &w,
            "reports_run_r11_bad_dates",
            json!({ "id": "R11", "filters": filters(json!({ "from": "11-10-2026", "to": "xx" })), "offset": 0, "limit": 0 }),
            ErrorCode::ErrDateInvalid,
        );
        refusal_pair(
            &w,
            "reports_run_r18_filter_not_taken",
            json!({ "id": "R18", "filters": filters(json!({ "from": "01-10-2026" })), "offset": -1, "limit": 0 }),
            ErrorCode::ErrOutOfRange,
        );
        print_pair(
            &w,
            "reports_print_data_r18_en",
            json!({ "id": "R18", "filters": filters(json!({})), "language": "en", "paper": null }),
        );
        print_pair(
            &w,
            "reports_print_data_r13_akram_ur",
            json!({ "id": "R13", "filters": filters(json!({ "party": party(w.akram) })), "language": "ur", "paper": "a4" }),
        );
    }
}

#[test]
fn the_paged_queries_use_indexes_and_never_scan_the_whole_journal() {
    // Code Standards B18: the query plans of the reports are kept in tests. The page query of the
    // journal reads entries by date; the ledger reads the lines of one account by its index.
    use mandi_reports::reports::plans::*;
    let w = full_day();
    let conn = raw(&w.dir);
    let plan = |sql: &str, params: &[&dyn mandi_services::db::rusqlite::ToSql]| -> String {
        let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let rows = statement
            .query_map(params, |r| r.get::<_, String>(3))
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        rows.join(" | ")
    };
    let none: Option<String> = None;
    let account: Option<i64> = Some(w.akram);
    let ledger_plan = plan(
        R13_PAGE_SQL,
        &[&account, &none, &none, &none, &100_i64, &0_i64],
    );
    assert!(
        ledger_plan.contains("USING INDEX") || ledger_plan.contains("USING COVERING INDEX"),
        "{ledger_plan}"
    );
    assert!(!ledger_plan.contains("SCAN l "), "{ledger_plan}");
    let ledger_totals = plan(R13_TOTALS_SQL, &[&account, &none, &none, &none]);
    assert!(!ledger_totals.contains("SCAN l "), "{ledger_totals}");
    let no_account: Option<i64> = None;
    let journal_plan = plan(
        R11_PAGE_SQL,
        &[
            &Some("2026-10-10".to_owned()),
            &Some("2026-10-10".to_owned()),
            &no_account,
            &none,
            &100_i64,
            &0_i64,
        ],
    );
    assert!(
        journal_plan.contains("idx_") || journal_plan.contains("USING INDEX"),
        "{journal_plan}"
    );
}
