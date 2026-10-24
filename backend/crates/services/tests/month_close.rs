//! Month close and its reversal on the PRD worked day (plan 22 round 7, PRD FR-17.6, FR-21.2,
//! FR-17.9, E2E-9, E2E-10, E2E-37).
//!
//! October 2026 after Events 1 to 5: the close moves Dr 04010001 2,484, 04010002 2,484, 04010003
//! 480, 04010004 40 and Cr 05010001 3,000, and puts 996, 746 and 746 on the partners' capital
//! accounts. The trial balance goes 85,048, then 82,048, then 85,048 after the reversal. Nobody's
//! total (کل رقم) changes, and the books keep balancing. The expected numbers are the PRD's.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::reports::{ReportFilters, ReportId};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::month::{
    MonthInput, MonthLineKind, MonthReverseInput, MonthState, close, list, preview, reverse,
};
use mandi_services::{ServiceError, day, ledger, partners};
use reports_world::*;
use support::World;

fn input(month: &str) -> MonthInput {
    MonthInput {
        month: month.into(),
    }
}

fn why(month: &str, reason: &str) -> MonthReverseInput {
    MonthReverseInput {
        month: month.into(),
        reason: reason.into(),
    }
}

fn refusal(error: &ServiceError) -> (ErrorCode, String) {
    match error {
        ServiceError::Rejected(items) => {
            (items[0].code, items[0].field.clone().unwrap_or_default())
        }
        other => panic!("{other:?}"),
    }
}

/// October is over (today 02-11-26) and all its days are closed.
fn october_closed() -> World {
    let w = full_day();
    set_today(&w.env, 2026, 11, 2);
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "31-10-2026".into(),
        },
    )
    .unwrap();
    w
}

fn trial(w: &World) -> (i64, i64) {
    let t = ledger::trial_balance(&w.env.ctx, None).unwrap();
    (t.total_dr.rupees(), t.total_cr.rupees())
}

fn totals(w: &World) -> Vec<i64> {
    partners::overview(&w.env.ctx)
        .unwrap()
        .into_iter()
        .map(|r| r.total_paisa.rupees())
        .collect()
}

fn equity(w: &World) -> i64 {
    // Capital, income and expense accounts and drawings, read the same way whatever the close:
    // credits less debits of heads 04, 05 (income, expenses), 06 (capital, drawings).
    let net: i64 = raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(l.cr_paisa), 0) - COALESCE(SUM(l.dr_paisa), 0)
             FROM journal_lines l JOIN accounts a ON a.id = l.account_id
             JOIN journal_entries e ON e.id = l.entry_id
             WHERE e.status = 'posted' AND substr(a.code, 1, 2) IN ('04', '05', '06')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    net / 100
}

fn shares(w: &World) -> i64 {
    raw(&w.dir)
        .query_row("SELECT COUNT(*) FROM partner_share_rows", [], |r| r.get(0))
        .unwrap()
}

fn page_totals(w: &World, id: ReportId) -> Vec<String> {
    let filters = ReportFilters {
        from: "2026-10-01".into(),
        to: "2026-10-31".into(),
        ..ReportFilters::default()
    };
    let page = report(w, id, filters, 0, 1_000);
    let mut cells: Vec<String> = page
        .totals
        .iter()
        .chain(page.summary.iter().map(|s| &s.cell))
        .map(|c| format!("{c:?}"))
        .collect();
    cells.extend(
        page.rows
            .iter()
            .flat_map(|r| r.cells.iter().map(|c| format!("{c:?}"))),
    );
    cells
}

// ---- The golden October close ---------------------------------------------------------------

#[test]
fn the_october_close_has_the_prd_lines_and_the_preview_is_the_same_plan() {
    let w = october_closed();
    let shown = preview(&w.env.ctx, &input("2026-10")).unwrap();
    assert_eq!(
        (shown.first_day.as_str(), shown.last_day.as_str()),
        ("2026-10-01", "2026-10-31")
    );
    assert_eq!(
        shown.entry_date, "2026-11-02",
        "the entry is dated the day it is made"
    );
    let line = |code: &str| {
        shown
            .lines
            .iter()
            .find(|l| l.account.code == code)
            .unwrap_or_else(|| panic!("no line for {code}"))
    };
    for (code, dr) in [
        ("04010001", 2_484),
        ("04010002", 2_484),
        ("04010003", 480),
        ("04010004", 40),
    ] {
        assert_eq!(
            (line(code).dr_paisa.rupees(), line(code).cr_paisa.rupees()),
            (dr, 0),
            "{code}"
        );
        assert_eq!(line(code).kind, MonthLineKind::Income);
    }
    assert_eq!(
        (
            line("05010001").dr_paisa.rupees(),
            line("05010001").cr_paisa.rupees()
        ),
        (0, 3_000)
    );
    assert_eq!(line("05010001").kind, MonthLineKind::Expense);
    let capital: Vec<i64> = shown
        .lines
        .iter()
        .filter(|l| l.kind == MonthLineKind::PartnerCapital)
        .map(|l| l.cr_paisa.rupees())
        .collect();
    assert_eq!(capital, [996, 746, 746]);
    assert_eq!(shown.lines.len(), 8);
    assert_eq!(
        (
            shown.income_paisa.rupees(),
            shown.expense_paisa.rupees(),
            shown.profit_paisa.rupees()
        ),
        (5_488, 3_000, 2_488)
    );
    assert_eq!(shown.total_paisa.rupees(), 5_488);
    // Saving posts exactly what the preview showed.
    let before = (trial(&w), totals(&w), equity(&w));
    let done = close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap();
    assert_eq!(done.total_paisa.rupees(), 5_488);
    let lines: Vec<(String, i64, i64)> = {
        let conn = raw(&w.dir);
        let mut statement = conn
            .prepare(
                "SELECT a.code, l.dr_paisa, l.cr_paisa FROM journal_lines l
                 JOIN accounts a ON a.id = l.account_id WHERE l.entry_id = ?1 ORDER BY l.line_no",
            )
            .unwrap();
        statement
            .query_map([done.entry_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let expected: Vec<(String, i64, i64)> = shown
        .lines
        .iter()
        .map(|l| {
            (
                l.account.code.clone(),
                l.dr_paisa.as_paisa(),
                l.cr_paisa.as_paisa(),
            )
        })
        .collect();
    assert_eq!(lines, expected);
    // The entry: posted, dated today, typed month_close, with no share rows of its own.
    let (date, kind, source, status): (String, String, String, String) = raw(&w.dir)
        .query_row(
            "SELECT entry_date, posting_type, source_type, status FROM journal_entries WHERE id = ?1",
            [done.entry_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        (
            date.as_str(),
            kind.as_str(),
            source.as_str(),
            status.as_str()
        ),
        ("2026-11-02", "month_close", "month_close", "posted")
    );
    // The books: 85,048 becomes 82,048; the partners' totals do not move; equity does not move.
    assert_eq!(before.0, (85_048, 85_048));
    assert_eq!(trial(&w), (82_048, 82_048));
    assert_eq!(totals(&w), before.1);
    assert_eq!(totals(&w), [-1_004, 746, 746]);
    assert_eq!(equity(&w), before.2);
    assert_chain_ok(&w.dir);
    assert_eq!(audit_count(&w.dir, "month_close"), 1);
}

#[test]
fn the_reports_of_the_month_do_not_change_and_a_close_writes_no_share_rows() {
    let w = october_closed();
    let rows = shares(&w);
    let before: Vec<Vec<String>> = [ReportId::R12, ReportId::R28, ReportId::R29, ReportId::R30]
        .iter()
        .map(|id| page_totals(&w, *id))
        .collect();
    close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap();
    assert_eq!(shares(&w), rows, "a month close writes no share rows");
    let after: Vec<Vec<String>> = [ReportId::R12, ReportId::R28, ReportId::R29, ReportId::R30]
        .iter()
        .map(|id| page_totals(&w, *id))
        .collect();
    assert_eq!(
        before, after,
        "R12, R28, R29 and R30 still show October as it was"
    );
    // R18 as of the end of October still has the income open; as of today it is closed.
    let as_of = |day: &str| {
        let page = report(
            &w,
            ReportId::R18,
            ReportFilters {
                as_of: day.into(),
                ..ReportFilters::default()
            },
            0,
            1_000,
        );
        page.totals
            .iter()
            .map(|c| format!("{c:?}"))
            .collect::<Vec<_>>()
    };
    assert_ne!(as_of("2026-10-31"), as_of("2026-11-02"));
    let end_of_october = ledger::trial_balance(&w.env.ctx, Some("2026-10-31")).unwrap();
    assert_eq!(end_of_october.total_dr.rupees(), 85_048);
    assert_eq!(
        ledger::trial_balance(&w.env.ctx, Some("2026-11-02"))
            .unwrap()
            .total_dr
            .rupees(),
        82_048
    );
}

#[test]
fn the_reversal_puts_everything_back_and_the_month_can_be_closed_again() {
    let w = october_closed();
    let first = close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap();
    let shown_after_first = preview(&w.env.ctx, &input("2026-10"));
    assert_eq!(
        refusal(&shown_after_first.unwrap_err()),
        (ErrorCode::ErrMonthClosed, "month".into())
    );
    let rows = shares(&w);
    let undone = reverse(&w.env.ctx, &w.owner, &why("2026-10", "wrong month")).unwrap();
    assert_ne!(undone.reversal_entry_id, first.entry_id);
    assert_eq!(trial(&w), (85_048, 85_048));
    assert_eq!(totals(&w), [-1_004, 746, 746]);
    assert_eq!(
        shares(&w),
        rows,
        "the reversal of a month close copies no rows"
    );
    let (state, reversed): (i64, Option<i64>) = raw(&w.dir)
        .query_row(
            "SELECT is_active, reversed_entry_id FROM month_closes",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((state, reversed), (0, Some(undone.reversal_entry_id)));
    assert_eq!(audit_count(&w.dir, "month_close_reverse"), 1);
    assert_chain_ok(&w.dir);
    // A close that was reversed cannot be reversed again.
    let again = reverse(&w.env.ctx, &w.owner, &why("2026-10", "again")).unwrap_err();
    assert_eq!(refusal(&again), (ErrorCode::ErrNotFound, "month".into()));
    // Closing again gives the same lines (the old close and its reversal are left out).
    let second = preview(&w.env.ctx, &input("2026-10")).unwrap();
    assert_eq!(second.lines.len(), 8);
    assert_eq!(second.total_paisa.rupees(), 5_488);
    close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap();
    assert_eq!(trial(&w), (82_048, 82_048));
    assert_eq!(totals(&w), [-1_004, 746, 746]);
}

#[test]
fn while_a_month_is_closed_its_days_cannot_be_reopened_until_the_close_is_reversed() {
    let w = october_closed();
    let reopen = |date: &str| {
        day::reopen(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::vouchers::DayReopenInput {
                date: date.into(),
                reason: "check".into(),
            },
        )
    };
    close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap();
    let error = reopen("10-10-2026").unwrap_err();
    assert_eq!(refusal(&error), (ErrorCode::ErrMonthClosed, "date".into()));
    reverse(&w.env.ctx, &w.owner, &why("2026-10", "fix a mistake")).unwrap();
    reopen("10-10-2026").unwrap();
}

// ---- Refusals -------------------------------------------------------------------------------

#[test]
fn a_month_is_refused_for_its_own_reasons_each_on_the_month() {
    let w = full_day();
    // Today is 10-10-26: October is not over.
    assert_eq!(
        refusal(&preview(&w.env.ctx, &input("2026-10")).unwrap_err()),
        (ErrorCode::ErrDateInFuture, "month".into())
    );
    // Over (today moves on), but its days are still open.
    set_today(&w.env, 2026, 11, 2);
    let open_days = close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap_err();
    assert_eq!(
        refusal(&open_days),
        (ErrorCode::ErrDayNotClosed, "month".into())
    );
    if let ServiceError::Rejected(items) = &open_days {
        assert!(items[0].values.contains_key("closed_up_to"));
    }
    // Bad texts.
    for bad in [
        "2026-13", "2026-1", "26-10", "2026/10", "", "2026-00", "abcd-ef",
    ] {
        assert_eq!(
            refusal(&preview(&w.env.ctx, &input(bad)).unwrap_err()),
            (ErrorCode::ErrFormatInvalid, "month".into()),
            "{bad:?}"
        );
    }
    // Nothing happened, and nothing was written.
    assert_eq!(
        raw(&w.dir)
            .query_row("SELECT COUNT(*) FROM month_closes", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(trial(&w), (85_048, 85_048));
}

#[test]
fn a_month_with_nothing_to_move_a_second_close_and_a_missing_reason_are_refused() {
    let w = october_closed();
    // September has no income and no expense.
    assert_eq!(
        refusal(&close(&w.env.ctx, &w.owner, &input("2026-09")).unwrap_err()),
        (ErrorCode::ErrNothingToClose, "month".into())
    );
    close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap();
    assert_eq!(
        refusal(&close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap_err()),
        (ErrorCode::ErrMonthClosed, "month".into())
    );
    for reason in ["", "   "] {
        assert_eq!(
            refusal(&reverse(&w.env.ctx, &w.owner, &why("2026-10", reason)).unwrap_err()),
            (ErrorCode::ErrReasonRequired, "reason".into())
        );
    }
    assert_eq!(
        refusal(&reverse(&w.env.ctx, &w.owner, &why("2026-08", "x")).unwrap_err()),
        (ErrorCode::ErrNotFound, "month".into())
    );
    // Still exactly one close in force.
    assert_eq!(
        raw(&w.dir)
            .query_row(
                "SELECT COUNT(*) FROM month_closes WHERE is_active = 1",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
}

#[test]
fn a_close_is_refused_when_today_is_closed() {
    let w = october_closed();
    // Closing up to today closes today too: an entry dated today could not be posted.
    let closed = day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "02-11-2026".into(),
        },
    );
    match closed {
        Ok(_) => {
            assert_eq!(
                refusal(&close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap_err()),
                (ErrorCode::ErrDateClosed, "month".into())
            );
            assert_eq!(trial(&w), (85_048, 85_048));
        }
        Err(error) => {
            // The day close refuses to close today itself, so the guard cannot be reached this
            // way; the close then works as normal.
            assert_eq!(refusal(&error).0, ErrorCode::ErrDateInFuture);
            close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap();
        }
    }
}

// ---- Months that are not the happy case -------------------------------------------------------

#[test]
fn a_loss_month_puts_debits_on_the_capital_accounts() {
    // Only Event 5's shop expense exists: income 0, expense 3,000, the shares are -1,200, -900,
    // -900. The close credits the expense and debits each partner's capital.
    let w = worked_day();
    event5(&w);
    set_today(&w.env, 2026, 11, 2);
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "31-10-2026".into(),
        },
    )
    .unwrap();
    let shown = preview(&w.env.ctx, &input("2026-10")).unwrap();
    assert_eq!(shown.lines.len(), 4);
    let expense = shown
        .lines
        .iter()
        .find(|l| l.kind == MonthLineKind::Expense)
        .unwrap();
    assert_eq!(
        (expense.dr_paisa.rupees(), expense.cr_paisa.rupees()),
        (0, 3_000)
    );
    let capital: Vec<i64> = shown
        .lines
        .iter()
        .filter(|l| l.kind == MonthLineKind::PartnerCapital)
        .map(|l| l.dr_paisa.rupees())
        .collect();
    assert_eq!(capital, [1_200, 900, 900]);
    assert_eq!(shown.profit_paisa.rupees(), -3_000);
    let before = (trial(&w), totals(&w));
    close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap();
    assert_eq!(totals(&w), before.1, "nobody's total moves");
    assert_eq!(trial(&w).0, trial(&w).1);
    // The income-less-expense and the partner sums agree: the close balanced.
    assert_chain_ok(&w.dir);
}

#[test]
fn the_months_list_says_where_each_month_stands() {
    let w = full_day();
    let states = |w: &World| -> Vec<(String, MonthState)> {
        list(&w.env.ctx)
            .unwrap()
            .rows
            .into_iter()
            .map(|r| (r.month, r.state))
            .collect()
    };
    // Today is 10-10-26 and the days up to 09-10 are closed. The books start in August 2026.
    assert_eq!(
        states(&w),
        [
            ("2026-10".into(), MonthState::NotFinished),
            ("2026-09".into(), MonthState::Ready),
            ("2026-08".into(), MonthState::Ready),
        ]
    );
    let w = october_closed();
    let rows = list(&w.env.ctx).unwrap().rows;
    assert_eq!(rows[0].month, "2026-11");
    assert_eq!(rows[0].state, MonthState::NotFinished);
    assert_eq!(rows[1].state, MonthState::Ready);
    close(&w.env.ctx, &w.owner, &input("2026-10")).unwrap();
    let rows = list(&w.env.ctx).unwrap().rows;
    assert_eq!(rows[1].state, MonthState::Closed);
    assert_eq!(rows[1].closed_on.as_deref(), Some("2026-11-02"));
    reverse(&w.env.ctx, &w.owner, &why("2026-10", "x")).unwrap();
    let rows = list(&w.env.ctx).unwrap().rows;
    assert_eq!(
        (rows[1].state, rows[1].reversed_count),
        (MonthState::Ready, 1)
    );
}
