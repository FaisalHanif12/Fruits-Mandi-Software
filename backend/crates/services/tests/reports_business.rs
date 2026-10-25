//! The Business group of the report engine on the PRD worked day (plan 20 step B3, PRD 19.2,
//! FR-6.5, FR-16.3, FR-17.3, C12, C20): R28 آمدن رپورٹ and R30 منافع و تقسیم.
//!
//! The expected numbers are the PRD's (Section 4.9, E2E-9): the day's income is 5,488, the shop
//! expense 3,000, the profit 2,488 = 996 + 746 + 746, and الف took 2,000 so his خالص is −1,004.
//! The report totals are also checked against the journal and the partner share rows, so a report
//! can never disagree with the books (money agent Section 9).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::reports::{
    Cell, FilterKind, GroupBy, PrintCell, ReportFilters, ReportGroup, ReportId, ReportPrintInput,
    catalog, print_data,
};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::auth::Language;
use mandi_services::core::error::ErrorCode;

use reports_world::*;

fn names_en(cell: &Cell) -> String {
    match cell {
        Cell::Names { names } => names.en.clone(),
        other => panic!("not names: {other:?}"),
    }
}

/// The money cells of a row from a column on.
fn amounts(cells: &[Cell], from: usize) -> Vec<i64> {
    cells[from..].iter().map(money).collect()
}

fn by_day() -> ReportFilters {
    ReportFilters {
        group_by: Some(GroupBy::Day),
        ..ReportFilters::default()
    }
}

#[test]
fn the_business_group_is_in_the_catalogue_with_its_filters() {
    let entries: Vec<_> = catalog()
        .into_iter()
        .filter(|e| e.group == ReportGroup::Business)
        .collect();
    assert_eq!(
        entries.iter().map(|e| e.id).collect::<Vec<_>>(),
        vec![
            ReportId::R28,
            ReportId::R29,
            ReportId::R30,
            ReportId::R31,
            ReportId::R32,
            ReportId::R33,
            ReportId::R34,
            ReportId::R35,
            ReportId::R36
        ]
    );
    assert_eq!(
        entries[0].filters,
        vec![
            FilterKind::DateRange,
            FilterKind::Party,
            FilterKind::GroupBy
        ]
    );
    // R29 takes the dates and an optional partner; R30 only the dates.
    assert_eq!(
        entries[1].filters,
        vec![FilterKind::DateRange, FilterKind::Partner]
    );
    assert_eq!(entries[2].filters, vec![FilterKind::DateRange]);
    assert_eq!(entries[0].group_title_id, "report-group-business");
}

// --- R28 -----------------------------------------------------------------------------------------

#[test]
fn r28_by_day_lists_each_income_account_and_the_day_is_5488_jama() {
    let w = full_day();
    let page = report(&w, ReportId::R28, by_day(), 0, 0);
    assert_eq!(
        page.columns
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>(),
        vec!["period", "account", "amount"]
    );
    assert_eq!(format!("{:?}", page.columns[0].kind), "Date");
    // PRD 4.9: seller commission 2,484, buyer commission 2,484, لاگا 480, فون 40.
    assert_eq!(page.total_count, 4);
    let rows: Vec<(String, String, (BalanceSide, i64))> = page
        .rows
        .iter()
        .map(|r| {
            let date = match &r.cells[0] {
                Cell::Date { iso } => iso.clone(),
                other => panic!("{other:?}"),
            };
            (date, code_of(&r.cells[1]), balance(&r.cells[2]))
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (
                "2026-10-10".into(),
                "04010001".into(),
                (BalanceSide::Cr, 2_484)
            ),
            (
                "2026-10-10".into(),
                "04010002".into(),
                (BalanceSide::Cr, 2_484)
            ),
            (
                "2026-10-10".into(),
                "04010003".into(),
                (BalanceSide::Cr, 480)
            ),
            (
                "2026-10-10".into(),
                "04010004".into(),
                (BalanceSide::Cr, 40)
            ),
        ]
    );
    // The total is the shared income of the day, 5,488 جمع (PRD 4.9).
    assert_eq!(
        balance(page.totals.get(2).unwrap()),
        (BalanceSide::Cr, 5_488)
    );
    assert_eq!(page.resolved_group(), Some(GroupBy::Day));
}

trait ResolvedGroup {
    fn resolved_group(&self) -> Option<GroupBy>;
}

impl ResolvedGroup for mandi_reports::reports::ReportPage {
    fn resolved_group(&self) -> Option<GroupBy> {
        self.filters.group_by
    }
}

#[test]
fn r28_by_month_is_the_default_and_names_the_month_by_numbers() {
    let w = full_day();
    let page = report(&w, ReportId::R28, ReportFilters::default(), 0, 0);
    assert_eq!(format!("{:?}", page.columns[0].kind), "Month");
    assert_eq!(page.total_count, 4);
    assert_eq!(
        page.rows[0].cells[0],
        Cell::Month {
            year: 2026,
            month: 10
        }
    );
    assert_eq!(
        balance(page.totals.get(2).unwrap()),
        (BalanceSide::Cr, 5_488)
    );
    assert_eq!(page.filters.group_by, Some(GroupBy::Month));
    // A range that holds no income, and one account.
    let none = report(
        &w,
        ReportId::R28,
        ReportFilters {
            from: "11-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(none.total_count, 0);
    assert_eq!(balance(none.totals.get(2).unwrap()), (BalanceSide::Zero, 0));
    let account = support::id_of(&w.env, "04010003");
    let one = report(
        &w,
        ReportId::R28,
        ReportFilters {
            party: party(account).party,
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(one.total_count, 1);
    assert_eq!(balance(one.totals.get(2).unwrap()), (BalanceSide::Cr, 480));
}

#[test]
fn r28_equals_the_net_credit_of_the_income_accounts_in_the_books() {
    // Money agent Section 9: a report total equals the matching ledger balance.
    let w = full_day();
    let page = report(&w, ReportId::R28, by_day(), 0, 0);
    for row in &page.rows {
        let code = code_of(&row.cells[1]);
        let id = support::id_of(&w.env, &code);
        let book = mandi_services::ledger::account_balance(&w.env.ctx, id).unwrap();
        assert_eq!(
            balance(&row.cells[2]),
            (book.side, book.amount_paisa.rupees()),
            "{code}"
        );
    }
    // Paged by one row, the pages join to the same rows and the totals stay.
    let mut joined = Vec::new();
    for offset in 0..4 {
        let part = report(&w, ReportId::R28, by_day(), offset, 1);
        assert_eq!(part.total_count, 4);
        assert_eq!(part.totals, page.totals);
        joined.extend(part.rows);
    }
    assert_eq!(joined, page.rows);
}

#[test]
fn r28_leaves_out_a_month_close_and_a_cancelled_posting_cancels_itself() {
    let w = worked_day();
    let one = save(&w, &support::event1(&w));
    let before = report(&w, ReportId::R28, by_day(), 0, 0);
    let before_r30 = report(&w, ReportId::R30, ReportFilters::default(), 0, 0);
    assert_eq!(
        balance(before.totals.get(2).unwrap()),
        (BalanceSide::Cr, 3_948)
    );
    // A month close debits the income and credits the partners' capital; it is not income earned.
    // It is posted as plain data to see that the report ignores it.
    let month_close_id = {
        use mandi_services::core::ids::AccountId;
        use mandi_services::domain::ledger::{
            LedgerLine, PostingType, Side, SourceType, build_entry,
        };
        use mandi_services::ledger::PostingEvent;
        let income = support::id_of(&w.env, "04010001");
        let capital = support::id_of(&w.env, "06010001");
        let entry = build_entry(vec![
            LedgerLine::new(AccountId::new(income), Side::Dr, rupees(1_764)),
            LedgerLine::new(AccountId::new(capital), Side::Cr, rupees(1_764)),
        ])
        .unwrap();
        w.env
            .ctx
            .db()
            .write(|tx| {
                mandi_services::ledger::post(
                    &w.env.ctx,
                    tx,
                    w.owner.user_id,
                    &PostingEvent {
                        date: mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10)
                            .unwrap(),
                        posting_type: PostingType::MonthClose,
                        source_type: SourceType::MonthClose,
                        source_id: 1,
                        narration: None,
                        reverses: None,
                        entry,
                    },
                )
            })
            .unwrap()
            .entry_id
    };
    let after = report(&w, ReportId::R28, by_day(), 0, 0);
    assert_eq!(
        after.totals, before.totals,
        "a month close is not income earned"
    );
    assert_eq!(after.rows, before.rows);
    assert_eq!(
        report(&w, ReportId::R30, ReportFilters::default(), 0, 0),
        before_r30,
        "R30 leaves out a month close"
    );
    // The reversal of a month close keeps source type month_close but has posting type reversal.
    // It is left out too, so R28 and R30 still read the same (finding 2).
    w.env
        .ctx
        .db()
        .write(|tx| {
            mandi_services::ledger::reverse(&w.env.ctx, tx, w.owner.user_id, month_close_id, "test")
        })
        .unwrap();
    let reversed = report(&w, ReportId::R28, by_day(), 0, 0);
    assert_eq!(reversed.totals, before.totals);
    assert_eq!(reversed.rows, before.rows);
    assert_eq!(
        report(&w, ReportId::R30, ReportFilters::default(), 0, 0),
        before_r30,
        "R30 leaves out the reversal of a month close"
    );
    // Cancelling the بکری: the reversal cancels the income of the day.
    let detail = mandi_services::bikri::get(&w.env.ctx, one.bikri_id).unwrap();
    mandi_services::bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::CancelInput {
            id: one.bikri_id,
            version: detail.version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let cancelled = report(&w, ReportId::R28, by_day(), 0, 0);
    assert_eq!(
        balance(cancelled.totals.get(2).unwrap()),
        (BalanceSide::Zero, 0)
    );
}

// --- R30 -----------------------------------------------------------------------------------------

#[test]
fn r30_is_the_prd_4_9_partner_table() {
    let w = full_day();
    let page = report(&w, ReportId::R30, ReportFilters::default(), 0, 0);
    assert_eq!(
        page.columns
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>(),
        vec![
            "partner",
            "income_share",
            "expense_share",
            "profit_share",
            "taken",
            "returned",
            "net"
        ]
    );
    assert_eq!(page.total_count, 3);
    // income share, expense share, profit share, taken, returned, خالص (C20).
    // الف 2,196 / 1,200 / 996, took 2,000: 996 − 2,000 = −1,004. ب and ج 1,646 / 900 / 746.
    let a = &page.rows[0].cells;
    assert_eq!(amounts(a, 1), vec![2_196, 1_200, 996, 2_000, 0, -1_004]);
    for row in &page.rows[1..] {
        assert_eq!(amounts(&row.cells, 1), vec![1_646, 900, 746, 0, 0, 746]);
    }
    assert_eq!(names_en(&a[0]), "Partner A");
    // The total row: 5,488 / 3,000 / 2,488 / 2,000 / 0 / 2,488 − 2,000 = 488.
    assert_eq!(
        amounts(&page.totals, 1),
        vec![5_488, 3_000, 2_488, 2_000, 0, 488]
    );
    // The lines above: income 5,488 − expenses 3,000 = profit 2,488 = 996 + 746 + 746.
    let summary: Vec<(String, i64)> = page
        .summary
        .iter()
        .map(|s| (s.label_id.clone(), money(&s.cell)))
        .collect();
    assert_eq!(
        summary,
        vec![
            ("report-r30-income".to_owned(), 5_488),
            ("report-r30-expenses".to_owned(), 3_000),
            ("report-r30-profit".to_owned(), 2_488),
        ]
    );
    // The partners' shares add up to the journal exactly (PRD FR-6.5).
    assert_eq!(money(page.totals.get(1).unwrap()), summary[0].1);
    assert_eq!(money(page.totals.get(2).unwrap()), summary[1].1);
    assert_eq!(money(page.totals.get(3).unwrap()), summary[2].1);
}

#[test]
fn r30_takes_the_date_range_and_shows_what_a_partner_returned_e2e_29() {
    let w = full_day();
    // On 11-10-26 الف returns 2,000: for 10-10 to 11-10 his net taken is 0 and خالص is 996.
    let day = |d: u32| set_today(&w.env, 2026, 10, d);
    day(11);
    post_return(&w, 2_000);
    let both = report(
        &w,
        ReportId::R30,
        ReportFilters {
            from: "10-10-2026".into(),
            to: "11-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(
        amounts(&both.rows[0].cells, 1),
        vec![2_196, 1_200, 996, 2_000, 2_000, 996]
    );
    // The first day alone: nothing was returned yet.
    let first = report(
        &w,
        ReportId::R30,
        ReportFilters {
            from: "10-10-2026".into(),
            to: "10-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(
        amounts(&first.rows[0].cells, 1),
        vec![2_196, 1_200, 996, 2_000, 0, -1_004]
    );
    // The second day alone: no income, no expense, 2,000 returned: خالص +2,000.
    let second = report(
        &w,
        ReportId::R30,
        ReportFilters {
            from: "11-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    assert_eq!(
        amounts(&second.rows[0].cells, 1),
        vec![0, 0, 0, 0, 2_000, 2_000]
    );
    assert_eq!(amounts(&second.totals, 1), vec![0, 0, 0, 0, 2_000, 2_000]);
}

/// Posts الف's return of 2,000 (a ذاتی خرچہ واپسی) as plain journal postings, as `reports_world`
/// posts Event 5.
fn post_return(w: &support::World, rupees_back: i64) {
    use mandi_services::core::ids::AccountId;
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::PostingEvent;
    let cash = support::id_of(&w.env, "01010001");
    let drawings = support::id_of(&w.env, "06030001");
    let entry = build_entry(vec![
        LedgerLine::new(AccountId::new(cash), Side::Dr, rupees(rupees_back)),
        LedgerLine::new(AccountId::new(drawings), Side::Cr, rupees(rupees_back)),
    ])
    .unwrap();
    w.env
        .ctx
        .db()
        .write(|tx| {
            mandi_services::ledger::post(
                &w.env.ctx,
                tx,
                w.owner.user_id,
                &PostingEvent {
                    date: mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 11)
                        .unwrap(),
                    posting_type: PostingType::PersonalReturn,
                    source_type: SourceType::Voucher,
                    source_id: 9_003,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap();
}

#[test]
fn r30_shows_a_loss_as_a_minus_amount() {
    // A day with only an expense: no income, a shop expense of 3,000, profit −3,000, split
    // −1,200 / −900 / −900 (E2E-20 style): the profit shares are below 0 and the table says so.
    let w = worked_day();
    event5(&w);
    let page = report(&w, ReportId::R30, ReportFilters::default(), 0, 0);
    assert_eq!(
        amounts(&page.rows[0].cells, 1),
        vec![0, 1_200, -1_200, 2_000, 0, -3_200]
    );
    assert_eq!(
        page.summary
            .iter()
            .map(|s| money(&s.cell))
            .collect::<Vec<_>>(),
        vec![0, 3_000, -3_000]
    );
    assert_eq!(money(page.totals.get(3).unwrap()), -3_000);
}

#[test]
fn r30_and_r28_write_nothing_and_print_in_both_languages() {
    let w = full_day();
    let count = |sql: &str| -> i64 { raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap() };
    let rows = || {
        (
            count("SELECT COUNT(*) FROM journal_entries"),
            count("SELECT COUNT(*) FROM journal_lines"),
            count("SELECT COUNT(*) FROM partner_share_rows"),
            count("SELECT COUNT(*) FROM audit_log"),
        )
    };
    let start = rows();
    for id in [ReportId::R28, ReportId::R30] {
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
            assert_eq!(data.rows.len(), page.rows.len());
            assert_eq!(data.totals.len(), page.columns.len());
        }
    }
    assert_eq!(rows(), start);
    // The English R30 prints the minus sign of a negative amount; R28 by month prints the month
    // as numbers for the template to name; the grouping is in the header.
    let en = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id: ReportId::R30,
            filters: ReportFilters::default(),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    match &en.rows[0].cells[6] {
        PrintCell::Money { text } => assert_eq!(text, "-1,004"),
        other => panic!("{other:?}"),
    }
    let month = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id: ReportId::R28,
            filters: ReportFilters::default(),
            language: Language::En,
            paper: None,
        },
    )
    .unwrap();
    assert_eq!(
        month.rows[0].cells[0],
        PrintCell::Month {
            year: "2026".into(),
            month: 10
        }
    );
    assert!(
        month
            .filters
            .iter()
            .any(|f| f.label_id == "report-filter-by-month" && f.value.is_empty())
    );
    let day = print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            id: ReportId::R28,
            filters: by_day(),
            language: Language::Ur,
            paper: None,
        },
    )
    .unwrap();
    match &day.rows[0].cells[0] {
        PrintCell::Date { text } => assert_eq!(text, "10-10-2026"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_business_filters_not_taken_are_refused() {
    let w = full_day();
    // R30 takes dates only; R28 takes a grouping, R30 does not.
    assert_eq!(
        rejected(
            &w,
            ReportId::R30,
            ReportFilters {
                group_by: Some(GroupBy::Day),
                ..party(w.akram)
            },
            0,
            0
        ),
        vec![
            (ErrorCode::ErrOutOfRange, "filters.party".to_owned()),
            (ErrorCode::ErrOutOfRange, "filters.group_by".to_owned())
        ]
    );
}
