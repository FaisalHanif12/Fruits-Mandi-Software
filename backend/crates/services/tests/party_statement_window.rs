//! Plan 74 T1 (PRD FR-12.47, FR-12.43, FR-19.11; Code Standards B31): one party's statement, 30 days at
//! a time. The window is `from` and `to` of the statement read (its opening, charged, paid and
//! closing are the totals of the filter); "go to a date" answers the 30-day window that ENDS on
//! min(date + 15 days, today) with the date's own row (plan 74 W5a).
//!
//! The data is the worked day (PRD Section 4) and later receipts of اکرم on 11-10, 12-10, 15-11 and
//! 02-12-2026 (today is 05-12-2026): his balance is 2,880 after the first day, 1,880, 1,380, 1,000 and
//! at last 0.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::reports::ReportId;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::party_ledgers::{
    PartyLedgerGoTo, PartyLedgerGoToInput, PartyLedgerStatement, PartyLedgerStatementInput,
};
use mandi_services::{assist, ledger, party_ledgers, vouchers};
use reports_world::*;
use support::World;

/// The worked day, and اکرم paying 1,000, 500, 380 and 1,000 on four later days. Today is 05-12-2026.
fn world() -> World {
    let w = full_day();
    for (today, date, amount) in [
        ((2026, 10, 11), "11-10-2026", "1,000"),
        ((2026, 10, 12), "12-10-2026", "500"),
        ((2026, 11, 15), "15-11-2026", "380"),
        ((2026, 12, 2), "02-12-2026", "1,000"),
    ] {
        set_today(&w.env, today.0, today.1, today.2);
        let mut input = receipt(w.akram, amount);
        input.date = date.into();
        vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    }
    set_today(&w.env, 2026, 12, 5);
    w
}

fn go_to(
    w: &World,
    account: i64,
    date: &str,
) -> Result<PartyLedgerGoTo, mandi_services::ServiceError> {
    party_ledgers::go_to_date(
        &w.env.ctx,
        &PartyLedgerGoToInput {
            account_id: account,
            date: date.to_owned(),
        },
    )
}

fn statement(
    w: &World,
    account: i64,
    from: Option<&str>,
    to: Option<&str>,
) -> PartyLedgerStatement {
    party_ledgers::statement(
        &w.env.ctx,
        &PartyLedgerStatementInput {
            account_id: account,
            from: from.map(str::to_owned),
            to: to.map(str::to_owned),
            offset: 0,
            limit: 100,
        },
    )
    .unwrap()
}

fn dates(s: &PartyLedgerStatement) -> Vec<&str> {
    s.rows.iter().map(|r| r.date.as_str()).collect()
}

fn figures(s: &PartyLedgerStatement) -> ((i64, BalanceSide), i64, i64, (i64, BalanceSide)) {
    (
        (s.opening_paisa.rupees(), s.opening_side),
        s.charged_total_paisa.rupees(),
        s.paid_total_paisa.rupees(),
        (s.closing_paisa.rupees(), s.closing_side),
    )
}

fn span(g: &PartyLedgerGoTo) -> i64 {
    let to = BusinessDate::parse_iso(&g.to).unwrap();
    let from = BusinessDate::parse_iso(&g.from).unwrap();
    to.days_after(from)
}

#[test]
fn a_window_is_the_totals_of_its_dates_and_carries_on_from_the_balance_before_it() {
    let w = world();
    // The window 27-09 to 26-10-2026: the three October days.
    let s = statement(&w, w.akram, Some("27-09-2026"), Some("26-10-2026"));
    assert_eq!(dates(&s), ["2026-10-12", "2026-10-11", "2026-10-10"]);
    assert_eq!(s.count, 3);
    assert_eq!(
        figures(&s),
        (
            (0, BalanceSide::Zero),
            12_880,
            11_500,
            (1_380, BalanceSide::Dr)
        )
    );
    // The next window, 27-10 to 25-11: one date, and it opens where the last one closed.
    let next = statement(&w, w.akram, Some("27-10-2026"), Some("25-11-2026"));
    assert_eq!(dates(&next), ["2026-11-15"]);
    assert_eq!(
        figures(&next),
        ((1_380, BalanceSide::Dr), 0, 380, (1_000, BalanceSide::Dr))
    );
    // A window with no date is empty, and its opening and closing are one balance.
    let none = statement(&w, w.akram, Some("20-11-2026"), Some("30-11-2026"));
    assert!(none.rows.is_empty());
    assert_eq!(none.count, 0);
    assert_eq!(
        figures(&none),
        ((1_000, BalanceSide::Dr), 0, 0, (1_000, BalanceSide::Dr))
    );
}

#[test]
fn the_newest_window_closes_on_the_party_card_and_r13() {
    let w = world();
    // The window ending today: 06-11 to 05-12-2026.
    let s = statement(&w, w.akram, Some("06-11-2026"), Some("05-12-2026"));
    assert_eq!(dates(&s), ["2026-12-02", "2026-11-15"]);
    let closing = (s.closing_side, s.closing_paisa.rupees());
    let book = ledger::account_balance(&w.env.ctx, w.akram).unwrap();
    assert_eq!(closing, (book.side, book.amount_paisa.rupees()));
    let card = assist::party_card(&w.env.ctx, w.akram).unwrap();
    assert_eq!(
        closing,
        (card.balance.side, card.balance.amount_paisa.rupees())
    );
    let r13 = report(&w, ReportId::R13, party(w.akram), 0, 0);
    assert_eq!(closing, balance(&r13.summary.last().unwrap().cell));
    assert_eq!(closing, (BalanceSide::Zero, 0));
}

#[test]
fn go_to_a_date_with_an_entry_opens_its_window_with_the_row_in_it() {
    let w = world();
    // 11-10-2026: the window ends 15 days after it, 26-10, and begins 29 days before that.
    let g = go_to(&w, w.akram, "11-10-2026").unwrap();
    assert_eq!(
        (g.from.as_str(), g.to.as_str()),
        ("2026-09-27", "2026-10-26")
    );
    assert_eq!(span(&g), 29, "30 days, both ends included");
    assert!(g.has_entry);
    // Newest first: 12-10, 11-10, 10-10. The row to show is the second.
    assert_eq!(
        dates(&g.statement),
        ["2026-10-12", "2026-10-11", "2026-10-10"]
    );
    assert_eq!(g.row, Some(1));
    // The statement is the window's page with its totals, the same figures as the read.
    assert_eq!(
        figures(&g.statement),
        figures(&statement(
            &w,
            w.akram,
            Some("27-09-2026"),
            Some("26-10-2026")
        ))
    );
    assert_eq!(g.statement.count, 3);
}

#[test]
fn go_to_a_date_with_no_entry_gives_the_window_around_it_and_no_row() {
    let w = world();
    let g = go_to(&w, w.akram, "20-10-2026").unwrap();
    assert_eq!(
        (g.from.as_str(), g.to.as_str()),
        ("2026-10-06", "2026-11-04")
    );
    assert!(!g.has_entry);
    assert_eq!(g.row, None);
    // The nearest entries are in view on the older side; none lies after it in this window.
    assert_eq!(
        dates(&g.statement),
        ["2026-10-12", "2026-10-11", "2026-10-10"]
    );
    // A date typed in any form the date fields take: 20-10 is the same day of this year.
    let again = go_to(&w, w.akram, "20-10").unwrap();
    assert_eq!((again.from, again.to), (g.from, g.to));
}

#[test]
fn a_window_never_ends_after_today() {
    let w = world();
    // 02-12-2026 + 15 days is after today (05-12-2026), so the window ends today.
    let g = go_to(&w, w.akram, "02-12-2026").unwrap();
    assert_eq!(
        (g.from.as_str(), g.to.as_str()),
        ("2026-11-06", "2026-12-05")
    );
    assert_eq!(span(&g), 29);
    assert!(g.has_entry);
    assert_eq!(g.row, Some(0));
    // The newest window closes on his balance now.
    assert_eq!(
        (g.statement.closing_side, g.statement.closing_paisa.rupees()),
        (BalanceSide::Zero, 0)
    );
    // Today itself is the same window.
    let today = go_to(&w, w.akram, "05-12-2026").unwrap();
    assert_eq!(
        (today.from.as_str(), today.to.as_str()),
        ("2026-11-06", "2026-12-05")
    );
    assert!(!today.has_entry);
}

#[test]
fn a_date_after_today_a_bad_date_and_an_account_with_no_ledger_are_refused() {
    let w = world();
    let error = go_to(&w, w.akram, "06-12-2026").unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrDateInFuture);
    let error = go_to(&w, w.akram, "31-02-2026").unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrDateInvalid);
    let error = go_to(&w, w.akram, "").unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrRequired);
    // The shared چکتا account has no ledger (PRD FR-13.6).
    let error = go_to(&w, w.chukta_khata, "10-10-2026").unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrNotFound);
}

#[test]
fn a_seller_has_his_window_too() {
    let w = world();
    // رحیم: 01-09 (advance), 10-10 (his sale, cash and payment). A date in the middle of them.
    let g = go_to(&w, w.rahim, "10-10-2026").unwrap();
    assert!(g.has_entry);
    assert_eq!(dates(&g.statement), ["2026-10-10"]);
    assert_eq!(
        (g.statement.closing_side, g.statement.closing_paisa.rupees()),
        (BalanceSide::Zero, 0)
    );
}
