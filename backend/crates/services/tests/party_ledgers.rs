//! Plan 66 T1 and T2 (D8 Feature 6; PRD FR-12.43, FR-19.11, E2E-67, E2E-68, FR-10.10): the party
//! ledgers screen. Level 1 is the balance of every buyer or seller, level 2 is one party's
//! statement with a row for each date. Both are read only.
//!
//! The expected numbers are the PRD's: the worked day after Events 1 to 5 (Section 4), E2E-67
//! (the lists and اکرم's statement), E2E-68 (a line moved to another buyer), and E2E-44 (the dead
//! amount). The parity tests hold the figures to the rest of the app: his balance is the party
//! card and the closing balance of R13; a day's charged and paid are the debit and credit of the
//! R35 all-parties table (the money review of plan 66: they are NOT the slip's "آج ادا", which
//! leaves cash lent and journal entries in "دیگر اندراج").
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::reports::{Cell, PartyFilter, ReportFilters, ReportId, Row};
use mandi_services::core::accounts::{
    AccountFilter, AccountListKind, AccountType, AccountsListInput, BalanceSide,
};
use mandi_services::core::bikri::{BikriSaved, CancelInput, EditOpenInput, EditSaveInput};
use mandi_services::core::dead::DeadMarkInput;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::party_ledgers::{
    LedgerSide, PartyLedgerDay, PartyLedgerStatement, PartyLedgerStatementInput, PartyLedgersList,
    PartyLedgersListInput,
};
use mandi_services::{accounts, assist, bikri, dead, ledger, party_ledgers, vouchers};
use reports_world::*;
use serde_json::json;
use support::World;

fn list(
    w: &World,
    side: LedgerSide,
    text: Option<&str>,
    offset: u32,
    limit: u32,
) -> PartyLedgersList {
    party_ledgers::list(
        &w.env.ctx,
        &PartyLedgersListInput {
            side,
            text: text.map(str::to_owned),
            offset,
            limit,
        },
    )
    .unwrap()
}

fn statement_of(
    w: &World,
    account_id: i64,
    from: Option<&str>,
    to: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<PartyLedgerStatement, mandi_services::ServiceError> {
    party_ledgers::statement(
        &w.env.ctx,
        &PartyLedgerStatementInput {
            account_id,
            from: from.map(str::to_owned),
            to: to.map(str::to_owned),
            offset,
            limit,
        },
    )
}

fn statement(w: &World, account_id: i64) -> PartyLedgerStatement {
    statement_of(w, account_id, None, None, 0, 0).unwrap()
}

/// A row as (code, balance in rupees, side).
fn row_of(list: &PartyLedgersList) -> Vec<(String, i64, BalanceSide)> {
    list.rows
        .iter()
        .map(|r| (r.code.clone(), r.balance_paisa.rupees(), r.side))
        .collect()
}

/// A day as (date, previous, previous side, charged, paid, balance, balance side).
type DayFigures = (String, i64, BalanceSide, i64, i64, i64, BalanceSide);

fn day_of(d: &PartyLedgerDay) -> DayFigures {
    (
        d.date.clone(),
        d.previous_paisa.rupees(),
        d.previous_side,
        d.charged_paisa.rupees(),
        d.paid_paisa.rupees(),
        d.balance_paisa.rupees(),
        d.balance_side,
    )
}

fn days(s: &PartyLedgerStatement) -> Vec<DayFigures> {
    s.rows.iter().map(day_of).collect()
}

fn day_text(
    date: &str,
    previous: (i64, BalanceSide),
    charged: i64,
    paid: i64,
    balance: (i64, BalanceSide),
) -> DayFigures {
    (
        date.to_owned(),
        previous.0,
        previous.1,
        charged,
        paid,
        balance.0,
        balance.1,
    )
}

fn today(w: &World, y: i32, m: u32, d: u32) {
    set_today(&w.env, y, m, d);
}

// ---- E2E-67: the lists and the statements after the worked day --------------------------------

#[test]
fn e2e_67_the_lists_of_buyers_and_sellers() {
    let w = full_day();
    let buyers = list(&w, LedgerSide::Buyer, None, 0, 0);
    // اکرم 2,880 بنام and ارشد 18,684 بنام. The چکتا کھاتہ (شاہد paid in full) and نقدی منڈی have no
    // ledger and are not listed (PRD FR-13.6).
    assert_eq!(
        row_of(&buyers),
        [
            ("0020".to_owned(), 2_880, BalanceSide::Dr),
            ("0021".to_owned(), 18_684, BalanceSide::Dr),
        ]
    );
    assert_eq!(buyers.count, 2);
    assert_eq!(buyers.total_dr_paisa.rupees(), 21_564);
    assert_eq!(buyers.total_cr_paisa.rupees(), 0);
    assert_eq!(buyers.rows[0].account_id, w.akram);
    assert_eq!(buyers.rows[0].name_en, "Akram Toll Tax");
    assert!(buyers.rows.iter().all(|r| r.is_active && !r.is_dead));

    let sellers = list(&w, LedgerSide::Seller, None, 0, 0);
    // رحیم is paid up (0), کریم is owed 10,160 (جمع: the shop owes him). اپنا مال is a system account.
    assert_eq!(
        row_of(&sellers),
        [
            ("0010".to_owned(), 0, BalanceSide::Zero),
            ("0011".to_owned(), 10_160, BalanceSide::Cr),
        ]
    );
    assert_eq!(sellers.total_dr_paisa.rupees(), 0);
    assert_eq!(sellers.total_cr_paisa.rupees(), 10_160);
}

#[test]
fn e2e_67_the_statements_of_akram_rahim_and_karim() {
    let w = full_day();
    // اکرم: 10-10-26 previous 0, charged 12,880, paid 10,000, balance 2,880 بنام.
    let akram = statement(&w, w.akram);
    assert_eq!(akram.head.kind, LedgerSide::Buyer);
    assert_eq!(akram.head.code, "0020");
    assert_eq!(
        days(&akram),
        [day_text(
            "2026-10-10",
            (0, BalanceSide::Zero),
            12_880,
            10_000,
            (2_880, BalanceSide::Dr)
        )]
    );
    assert_eq!(akram.count, 1);
    assert_eq!(
        (
            akram.charged_total_paisa.rupees(),
            akram.paid_total_paisa.rupees()
        ),
        (12_880, 10_000)
    );
    assert_eq!(
        (akram.opening_paisa.rupees(), akram.opening_side),
        (0, BalanceSide::Zero)
    );
    assert_eq!(
        (akram.closing_paisa.rupees(), akram.closing_side),
        (2_880, BalanceSide::Dr)
    );

    // ارشد: 18,684 charged on the day (17,400 + 1,044 + 240), nothing paid.
    assert_eq!(
        days(&statement(&w, w.arshad)),
        [day_text(
            "2026-10-10",
            (0, BalanceSide::Zero),
            18_684,
            0,
            (18_684, BalanceSide::Dr)
        )]
    );

    // رحیم, newest first: on 10-10-26 he was owed 24,316 (the net sale) and was paid 5,000 نقد and
    // 4,316 ادائیگی, so his advance of 15,000 is used up. On 01-09-26 the advance was given: paid
    // 15,000, so his balance is 15,000 بنام (he owes the shop).
    let rahim = statement(&w, w.rahim);
    assert_eq!(rahim.head.kind, LedgerSide::Seller);
    assert_eq!(
        days(&rahim),
        [
            day_text(
                "2026-10-10",
                (15_000, BalanceSide::Dr),
                24_316,
                9_316,
                (0, BalanceSide::Zero)
            ),
            day_text(
                "2026-09-01",
                (0, BalanceSide::Zero),
                0,
                15_000,
                (15_000, BalanceSide::Dr)
            ),
        ]
    );
    assert_eq!(rahim.count, 2);
    assert_eq!(
        (
            rahim.charged_total_paisa.rupees(),
            rahim.paid_total_paisa.rupees()
        ),
        (24_316, 24_316)
    );
    assert_eq!(rahim.closing_side, BalanceSide::Zero);

    // کریم: owed 10,160 from 10-10-26 (جمع).
    assert_eq!(
        days(&statement(&w, w.karim)),
        [day_text(
            "2026-10-10",
            (0, BalanceSide::Zero),
            10_160,
            0,
            (10_160, BalanceSide::Cr)
        )]
    );
}

// ---- paging, the text filter and the footer ------------------------------------------------------

#[test]
fn a_page_is_a_slice_and_the_footer_is_over_the_whole_filter() {
    let w = full_day();
    let second = list(&w, LedgerSide::Buyer, None, 1, 1);
    assert_eq!(
        row_of(&second),
        [("0021".to_owned(), 18_684, BalanceSide::Dr)]
    );
    // The count and the footer never come from the page (Code Standards B31).
    assert_eq!(second.count, 2);
    assert_eq!(second.total_dr_paisa.rupees(), 21_564);
    // A page past the end is empty, and still carries the footer.
    let past = list(&w, LedgerSide::Buyer, None, 5, 10);
    assert!(past.rows.is_empty());
    assert_eq!((past.count, past.total_dr_paisa.rupees()), (2, 21_564));
    // Text: the English name, the Urdu name and the code, as in the account list.
    for (text, code, balance) in [
        ("akram", "0020", 2_880),
        ("ارشد", "0021", 18_684),
        ("0021", "0021", 18_684),
    ] {
        let found = list(&w, LedgerSide::Buyer, Some(text), 0, 0);
        assert_eq!(
            row_of(&found),
            [(code.to_owned(), balance, BalanceSide::Dr)],
            "{text}"
        );
        assert_eq!(found.count, 1);
        assert_eq!(
            found.total_dr_paisa.rupees(),
            balance,
            "the footer of the filter, {text}"
        );
    }
    assert!(
        list(&w, LedgerSide::Buyer, Some("zzzz"), 0, 0)
            .rows
            .is_empty()
    );
}

#[test]
fn the_list_has_the_parties_and_the_order_of_the_account_list() {
    let w = full_day();
    for (side, kind) in [
        (LedgerSide::Buyer, AccountListKind::Buyer),
        (LedgerSide::Seller, AccountListKind::Seller),
    ] {
        for text in ["", "a", "00"] {
            let ours = list(&w, side, Some(text), 0, 0);
            let theirs = accounts::list(
                &w.env.ctx,
                &AccountsListInput {
                    filter: AccountFilter {
                        kind,
                        text: text.to_owned(),
                        include_inactive: true,
                        has_opening: None,
                    },
                    offset: 0,
                    limit: 0,
                    sort: None,
                },
            )
            .unwrap();
            let codes: Vec<&str> = ours.rows.iter().map(|r| r.code.as_str()).collect();
            let expected: Vec<&str> = theirs.rows.iter().map(|r| r.code.as_str()).collect();
            assert_eq!(codes, expected, "{side:?} {text:?}");
            assert_eq!(ours.count, theirs.count, "{side:?} {text:?}");
        }
    }
}

// ---- parity with the party card, R13 and the R35 table -------------------------------------------

/// The closing balance of R13 for a party.
fn r13_closing(w: &World, account: i64) -> (BalanceSide, i64) {
    let page = report(w, ReportId::R13, party(account), 0, 0);
    balance(&page.summary.last().unwrap().cell)
}

/// A row of the R35 all-parties table: (previous, debit, credit, remaining).
type R35Row = ((BalanceSide, i64), i64, i64, (BalanceSide, i64));

/// The row of one party in the R35 all-parties table for a day.
fn r35_row(w: &World, day: &str, code: &str) -> Option<R35Row> {
    let page = report(
        w,
        ReportId::R35,
        ReportFilters {
            as_of: day.into(),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    page.rows
        .iter()
        .find(|row: &&Row| matches!(row.cells.get(1), Some(Cell::Party { party }) if party.code == code))
        .map(|row| {
            (
                balance(&row.cells[2]),
                money(&row.cells[3]),
                money(&row.cells[4]),
                balance(&row.cells[5]),
            )
        })
}

#[test]
fn a_statement_agrees_with_the_party_card_r13_and_the_ledger() {
    let w = full_day();
    for id in [w.akram, w.arshad, w.rahim, w.karim] {
        let s = statement(&w, id);
        let closing = (s.closing_side, s.closing_paisa.rupees());
        let book = ledger::account_balance(&w.env.ctx, id).unwrap();
        assert_eq!(
            closing,
            (book.side, book.amount_paisa.rupees()),
            "ledger {id}"
        );
        let card = assist::party_card(&w.env.ctx, id).unwrap();
        assert_eq!(
            closing,
            (card.balance.side, card.balance.amount_paisa.rupees()),
            "party card {id}"
        );
        assert_eq!(closing, r13_closing(&w, id), "R13 {id}");
        // The newest row's balance is the closing balance, and the list says the same.
        let newest = s.rows.first().unwrap();
        assert_eq!(
            (newest.balance_side, newest.balance_paisa.rupees()),
            closing,
            "newest row {id}"
        );
    }
    // The list is the same figure again.
    for (side, ids) in [
        (LedgerSide::Buyer, [w.akram, w.arshad]),
        (LedgerSide::Seller, [w.rahim, w.karim]),
    ] {
        let listed = list(&w, side, None, 0, 0);
        for (row, id) in listed.rows.iter().zip(ids) {
            let book = ledger::account_balance(&w.env.ctx, id).unwrap();
            assert_eq!(row.account_id, id);
            assert_eq!(
                (row.side, row.balance_paisa),
                (book.side, book.amount_paisa)
            );
        }
    }
}

#[test]
fn a_days_charged_and_paid_are_the_debit_and_credit_of_the_r35_table() {
    let w = full_day();
    // A buyer is charged with the debit column and pays with the credit column; a seller the other
    // way round.
    for (id, code, kind) in [
        (w.akram, "0020", LedgerSide::Buyer),
        (w.arshad, "0021", LedgerSide::Buyer),
        (w.rahim, "0010", LedgerSide::Seller),
        (w.karim, "0011", LedgerSide::Seller),
    ] {
        for day in statement(&w, id).rows {
            let iso = &day.date;
            let ddmmyyyy = format!("{}-{}-{}", &iso[8..10], &iso[5..7], &iso[0..4]);
            let (previous, debit, credit, remaining) = r35_row(&w, &ddmmyyyy, code)
                .unwrap_or_else(|| panic!("R35 has no row for {code} on {iso}"));
            let (charged, paid) = match kind {
                LedgerSide::Buyer => (debit, credit),
                LedgerSide::Seller => (credit, debit),
            };
            assert_eq!(
                (
                    (day.previous_side, day.previous_paisa.rupees()),
                    day.charged_paisa.rupees(),
                    day.paid_paisa.rupees(),
                    (day.balance_side, day.balance_paisa.rupees())
                ),
                (previous, charged, paid, remaining),
                "{code} on {iso}"
            );
        }
    }
}

/// Level 3 is the R35 day slip of the party and the date. Its split of "paid today" is its own, but
/// its previous balance and its new balance are the statement's: the same figures, on the same side.
#[test]
fn the_day_slip_opens_and_closes_on_the_balances_of_the_statement() {
    let w = full_day();
    for id in [w.akram, w.arshad, w.rahim, w.karim] {
        for day in statement(&w, id).rows {
            let iso = &day.date;
            let ddmmyyyy = format!("{}-{}-{}", &iso[8..10], &iso[5..7], &iso[0..4]);
            let slip = report(
                &w,
                ReportId::R35,
                ReportFilters {
                    as_of: ddmmyyyy,
                    party: Some(PartyFilter {
                        account_id: id,
                        person_key: None,
                    }),
                    ..ReportFilters::default()
                },
                0,
                0,
            );
            let line = |label: &str| {
                let cell = slip
                    .summary
                    .iter()
                    .find(|l| l.label_id == label)
                    .unwrap_or_else(|| panic!("the slip of {id} on {iso} has no {label}"))
                    .cell
                    .clone();
                balance(&cell)
            };
            assert_eq!(
                line("report-r35-previous"),
                (day.previous_side, day.previous_paisa.rupees()),
                "previous, {id} on {iso}"
            );
            assert_eq!(
                line("report-r35-new-balance"),
                (day.balance_side, day.balance_paisa.rupees()),
                "new balance, {id} on {iso}"
            );
        }
    }
}

/// The footer reads the balances of the matching parties by their ids up to 500 parties, and from
/// one grouped read of the whole control above that. The two reads give the same totals.
#[test]
fn the_two_ways_of_reading_the_balances_of_a_page_agree() {
    let w = full_day();
    w.env
        .ctx
        .db()
        .read(|conn| {
            for (control, ids) in [
                ("0304", vec![w.akram, w.arshad]),
                ("03011", vec![w.rahim, w.karim]),
            ] {
                let named = mandi_db::ledger::totals_for_accounts(conn, &ids).unwrap();
                let whole: std::collections::HashMap<_, _> =
                    mandi_db::ledger::account_totals_in_controls(conn, &[control], None)
                        .unwrap()
                        .into_iter()
                        .collect();
                for id in ids {
                    let by_name = named.get(&id).map(|t| (t.dr_paisa, t.cr_paisa));
                    let by_control = whole.get(&id).map(|t| (t.dr_paisa, t.cr_paisa));
                    assert_eq!(by_name, by_control, "{control} {id}");
                }
            }
            Ok::<(), mandi_services::ServiceError>(())
        })
        .unwrap();
}

// ---- accounts with no ledger -----------------------------------------------------------------------

#[test]
fn an_account_with_no_ledger_is_not_found() {
    let w = full_day();
    let cash = support::id_of(&w.env, "01010001");
    for id in [w.chukta_khata, w.naqdi, cash, 9_999_999] {
        let error = statement_of(&w, id, None, None, 0, 0).unwrap_err();
        assert_eq!(error.code(), ErrorCode::ErrNotFound, "{id}");
    }
}

// ---- the dates --------------------------------------------------------------------------------------

#[test]
fn a_date_range_and_the_balance_before_it() {
    let w = full_day();
    // رحیم: only the days from 10-10-26. His balance before it is the 15,000 of the advance.
    let from = statement_of(&w, w.rahim, Some("10-10-2026"), None, 0, 0).unwrap();
    assert_eq!(from.count, 1);
    assert_eq!(
        (from.opening_paisa.rupees(), from.opening_side),
        (15_000, BalanceSide::Dr)
    );
    assert_eq!(
        days(&from),
        [day_text(
            "2026-10-10",
            (15_000, BalanceSide::Dr),
            24_316,
            9_316,
            (0, BalanceSide::Zero)
        )]
    );
    assert_eq!(
        (
            from.charged_total_paisa.rupees(),
            from.paid_total_paisa.rupees()
        ),
        (24_316, 9_316)
    );
    assert_eq!(from.closing_side, BalanceSide::Zero);
    // Up to 09-10-26: only the advance, and the closing balance is the balance at the end of that
    // day, not his balance now.
    let to = statement_of(&w, w.rahim, None, Some("09-10-2026"), 0, 0).unwrap();
    assert_eq!(
        days(&to),
        [day_text(
            "2026-09-01",
            (0, BalanceSide::Zero),
            0,
            15_000,
            (15_000, BalanceSide::Dr)
        )]
    );
    assert_eq!(
        (to.closing_paisa.rupees(), to.closing_side),
        (15_000, BalanceSide::Dr)
    );
    // A range with no entry is empty, and its opening and closing are the same balance.
    let empty = statement_of(&w, w.rahim, Some("02-09-2026"), Some("09-10-2026"), 0, 0).unwrap();
    assert!(empty.rows.is_empty());
    assert_eq!(empty.count, 0);
    assert_eq!(
        (
            (empty.opening_paisa, empty.opening_side),
            (empty.closing_paisa, empty.closing_side)
        ),
        (
            (to.closing_paisa, to.closing_side),
            (to.closing_paisa, to.closing_side)
        )
    );
    // `to` before `from` is refused, on `to`.
    let error =
        statement_of(&w, w.rahim, Some("10-10-2026"), Some("01-09-2026"), 0, 0).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrDateRange);
    // A date that is not a date is refused on its own field.
    let error = statement_of(&w, w.rahim, Some("31-02-2026"), None, 0, 0).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ErrDateInvalid);
}

#[test]
fn many_days_page_newest_first_and_each_row_carries_on_from_the_one_before() {
    let w = full_day();
    // اکرم pays again on two later days.
    for (day, text, amount) in [(11, "11-10-2026", "1,000"), (12, "12-10-2026", "500")] {
        today(&w, 2026, 10, day);
        let mut input = receipt(w.akram, amount);
        input.date = text.into();
        vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    }
    let all = statement(&w, w.akram);
    assert_eq!(all.count, 3);
    assert_eq!(
        days(&all),
        [
            day_text(
                "2026-10-12",
                (1_880, BalanceSide::Dr),
                0,
                500,
                (1_380, BalanceSide::Dr)
            ),
            day_text(
                "2026-10-11",
                (2_880, BalanceSide::Dr),
                0,
                1_000,
                (1_880, BalanceSide::Dr)
            ),
            day_text(
                "2026-10-10",
                (0, BalanceSide::Zero),
                12_880,
                10_000,
                (2_880, BalanceSide::Dr)
            ),
        ]
    );
    assert_eq!(
        (
            all.charged_total_paisa.rupees(),
            all.paid_total_paisa.rupees()
        ),
        (12_880, 11_500)
    );
    // One row at a time gives the same rows, and a row's previous balance is the balance after the
    // row below it, so the pages join up.
    let pages: Vec<DayFigures> = (0..3)
        .map(|offset| {
            let page = statement_of(&w, w.akram, None, None, offset, 1).unwrap();
            assert_eq!(page.count, 3, "the count is the whole filter");
            assert_eq!(
                page.charged_total_paisa.rupees(),
                12_880,
                "totals are not the page's"
            );
            assert_eq!(page.rows.len(), 1);
            day_of(&page.rows[0])
        })
        .collect();
    assert_eq!(pages, days(&all));
    for pair in all.rows.windows(2) {
        assert_eq!(
            (pair[0].previous_side, pair[0].previous_paisa),
            (pair[1].balance_side, pair[1].balance_paisa)
        );
    }
    // The newest row is his balance now.
    assert_eq!(
        (all.closing_paisa.rupees(), all.closing_side),
        (1_380, BalanceSide::Dr)
    );
    assert_eq!(r13_closing(&w, w.akram), (BalanceSide::Dr, 1_380));
}

// ---- a same-day edit or cancel leaves no row of its own -----------------------------------------------

fn cancel(w: &World, saved: &BikriSaved) {
    let version = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &CancelInput {
            id: saved.bikri_id,
            version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

#[test]
fn a_cancelled_bikri_leaves_no_row_and_the_books_balance_the_same() {
    // E2E-15: بکری 2 (کریم's سیب) is cancelled on its own day. The entry and its reversal are the
    // two halves of one pair and are left out, as in R35: کریم has nothing to show.
    let w = worked_day();
    save(&w, &support::event1(&w));
    let two = save(&w, &event2(&w));
    assert_eq!(statement(&w, w.karim).rows.len(), 1);
    cancel(&w, &two);
    let karim = statement(&w, w.karim);
    assert!(karim.rows.is_empty());
    assert_eq!(karim.count, 0);
    assert_eq!(karim.closing_side, BalanceSide::Zero);
    assert_eq!(r13_closing(&w, w.karim), (BalanceSide::Zero, 0));
    // The R35 table agrees: no row for him on that day.
    assert!(r35_row(&w, "10-10-2026", "0011").is_none());
    // بکری 1's parties are as before.
    assert_eq!(statement(&w, w.akram).rows.len(), 1);
}

// ---- E2E-68: a line moved to another buyer, and back --------------------------------------------------

#[test]
fn e2e_68_a_moved_line_shows_on_the_new_buyer_and_the_pairs_leave_no_rows() {
    let w = worked_day();
    let one = save(&w, &support::event1(&w));
    save(&w, &event2(&w));
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "4,316")).unwrap();
    event5(&w);
    let tariq = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "طارق",
        "Tariq",
        true,
        "",
    );

    // Each edit posts the بکری again under a new entry: the next edit opens that one.
    let entry = std::cell::Cell::new(one.entry_id);
    let move_line = |to: i64| {
        let opened = bikri::open_for_edit(
            &w.env.ctx,
            &EditOpenInput {
                bikri_id: one.bikri_id,
                entry_id: entry.get(),
            },
        )
        .unwrap();
        let mut draft = opened.draft.clone();
        draft.lots[0].sales[0].buyer_account_id = Some(to);
        let edited = bikri::edit_save(
            &w.env.ctx,
            &w.owner,
            &EditSaveInput {
                entry_id: opened.entry_id,
                version: opened.version,
                draft,
                reason: Some("غلط گاہک".into()),
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
        entry.set(edited.entry_id);
    };

    move_line(tariq);
    // اکرم: no bill left, his 10,000 receipt is credit. One row: the receipt.
    assert_eq!(
        days(&statement(&w, w.akram)),
        [day_text(
            "2026-10-10",
            (0, BalanceSide::Zero),
            0,
            10_000,
            (10_000, BalanceSide::Cr)
        )]
    );
    // طارق: the 12,880 bill. One row, though the posting was reversed and posted again.
    assert_eq!(
        days(&statement(&w, tariq)),
        [day_text(
            "2026-10-10",
            (0, BalanceSide::Zero),
            12_880,
            0,
            (12_880, BalanceSide::Dr)
        )]
    );
    // رحیم and ارشد are as before.
    assert_eq!(statement(&w, w.rahim).rows.len(), 2);
    assert_eq!(
        days(&statement(&w, w.arshad)),
        [day_text(
            "2026-10-10",
            (0, BalanceSide::Zero),
            18_684,
            0,
            (18_684, BalanceSide::Dr)
        )]
    );
    // The list: اکرم 10,000 جمع, ارشد 18,684 بنام, طارق 12,880 بنام. The debit footer is 18,684 +
    // 12,880 = 31,564, the credit footer 10,000.
    let buyers = list(&w, LedgerSide::Buyer, None, 0, 0);
    assert_eq!(
        row_of(&buyers),
        [
            ("0020".to_owned(), 10_000, BalanceSide::Cr),
            ("0021".to_owned(), 18_684, BalanceSide::Dr),
            ("0022".to_owned(), 12_880, BalanceSide::Dr),
        ]
    );
    assert_eq!(
        (
            buyers.total_dr_paisa.rupees(),
            buyers.total_cr_paisa.rupees()
        ),
        (31_564, 10_000)
    );
    // Parity with the R35 table for the new buyers.
    for (id, code) in [(w.akram, "0020"), (tariq, "0022")] {
        let day = &statement(&w, id).rows[0];
        let (previous, debit, credit, remaining) = r35_row(&w, "10-10-2026", code).unwrap();
        assert_eq!(
            (
                (day.previous_side, day.previous_paisa.rupees()),
                day.charged_paisa.rupees(),
                day.paid_paisa.rupees(),
                (day.balance_side, day.balance_paisa.rupees())
            ),
            (previous, debit, credit, remaining),
            "{code}"
        );
    }

    // And back: every figure returns to E2E-67, and طارق has nothing left to show.
    move_line(w.akram);
    assert_eq!(
        days(&statement(&w, w.akram)),
        [day_text(
            "2026-10-10",
            (0, BalanceSide::Zero),
            12_880,
            10_000,
            (2_880, BalanceSide::Dr)
        )]
    );
    let tariq_back = statement(&w, tariq);
    assert!(tariq_back.rows.is_empty());
    assert_eq!(tariq_back.closing_side, BalanceSide::Zero);
    let buyers = list(&w, LedgerSide::Buyer, None, 0, 0);
    assert_eq!(buyers.total_dr_paisa.rupees(), 21_564);
    assert_eq!(buyers.total_cr_paisa.rupees(), 0);
}

// ---- the dead amount (PRD FR-10.10, E2E-44) ---------------------------------------------------------------

#[test]
fn a_buyer_in_the_dead_amount_stays_in_the_list_marked_and_in_the_footer() {
    let w = full_day();
    today(&w, 2026, 10, 11);
    let mark: DeadMarkInput = serde_json::from_value(json!({
        "account_id": w.arshad,
        "entry_date": "11-10-2026", "reason": "died", "note": "فوت ہو گئے"
    }))
    .unwrap();
    dead::mark(&w.env.ctx, &w.owner, &mark).unwrap();
    let buyers = list(&w, LedgerSide::Buyer, None, 0, 0);
    // His account is inactive but he still owes 18,684: he is listed, marked, and in the footer.
    // The footer is R20 (2,880) + his dead amount (18,684), the total of the buyers' controls.
    assert_eq!(
        row_of(&buyers),
        [
            ("0020".to_owned(), 2_880, BalanceSide::Dr),
            ("0021".to_owned(), 18_684, BalanceSide::Dr),
        ]
    );
    assert_eq!(
        buyers
            .rows
            .iter()
            .map(|r| (r.is_active, r.is_dead))
            .collect::<Vec<_>>(),
        [(true, false), (false, true)]
    );
    assert_eq!(buyers.total_dr_paisa.rupees(), 21_564);
    let s = statement(&w, w.arshad);
    assert!(s.head.is_dead && !s.head.is_active);
    assert_eq!(
        (s.closing_paisa.rupees(), s.closing_side),
        (18_684, BalanceSide::Dr)
    );
    // The statement is the same as before the move: nothing posts (PRD FR-10.10).
    assert_eq!(
        days(&s),
        [day_text(
            "2026-10-10",
            (0, BalanceSide::Zero),
            18_684,
            0,
            (18_684, BalanceSide::Dr)
        )]
    );
}

// ---- the commands are read only --------------------------------------------------------------------------------

#[test]
fn reading_a_ledger_writes_nothing() {
    let w = full_day();
    let count = |table: &str| -> i64 {
        raw(&w.dir)
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    };
    let tables = [
        "journal_entries",
        "journal_lines",
        "audit_log",
        "bikri_sales",
        "accounts",
    ];
    let before: Vec<i64> = tables.iter().map(|t| count(t)).collect();
    let first = list(&w, LedgerSide::Buyer, None, 0, 0);
    let second = statement(&w, w.akram);
    // The same read twice is the same answer.
    assert_eq!(first, list(&w, LedgerSide::Buyer, None, 0, 0));
    assert_eq!(second, statement(&w, w.akram));
    let after: Vec<i64> = tables.iter().map(|t| count(t)).collect();
    assert_eq!(before, after);
}
