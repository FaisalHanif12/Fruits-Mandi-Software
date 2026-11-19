//! Gate lane T4: the جنرل entry under bad input, and E5 (a buyer returns goods: a جنرل entry for
//! now) (plan 80; Code Standards 4.2, 4.3, M10, V0; PRD 16.4, FR-6.2, FR-7.48, E5, E24, E31).
//! Owner: `robustness`.
//!
//! The جنرل screen is the one place where the user writes the journal lines himself, so it is the
//! entry with the most ways to go wrong: lines that do not balance, both sides on one line, zero,
//! negative, decimals, the largest amount, Urdu digits, dates that cannot be. Every bad entry is
//! refused with a code and writes nothing (not one journal row, voucher row or audit row); a good
//! entry in other spellings gives the same books.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::raw;
use mandi_services::bikri;
use mandi_services::core::bikri::BikriSaveInput;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
use mandi_services::{ServiceError, ledger, vouchers};
use support::*;

fn line(account: i64, dr: &str, cr: &str) -> GeneralLineInput {
    GeneralLineInput {
        account_id: Some(account),
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    }
}

fn entry(date: &str, lines: Vec<GeneralLineInput>) -> GeneralInput {
    GeneralInput {
        date: date.into(),
        narration: None,
        lines,
        confirm_cash_below_zero: true,
    }
}

fn counts(w: &World) -> Vec<i64> {
    let conn = raw(&w.dir);
    [
        "journal_entries",
        "journal_lines",
        "vouchers",
        "partner_share_rows",
        "bill_allocations",
        "advance_allocations",
        "audit_log",
    ]
    .iter()
    .map(|t| {
        conn.query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |r| r.get(0))
            .unwrap()
    })
    .collect()
}

fn codes(error: &ServiceError) -> Vec<ErrorCode> {
    match error {
        ServiceError::Rejected(items) => items.iter().map(|i| i.code).collect(),
        other => vec![other.code()],
    }
}

/// A world with Event 1 saved: Akram owes 12,880 and Arshad 18,684.
fn world_with_event1() -> World {
    let w = world();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: event1(&w),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    w
}

fn balance(w: &World, id: i64) -> (String, i64) {
    let b = ledger::account_balance(&w.env.ctx, id).unwrap();
    (format!("{:?}", b.side), b.amount_paisa.rupees())
}

#[test]
fn a_bad_journal_entry_is_refused_with_a_code_and_writes_nothing() {
    use ErrorCode::*;
    let w = world_with_event1();
    let (cash, akram, rahim) = (id_of(&w.env, "01010001"), w.akram, w.rahim);
    let before = counts(&w);
    let cases: Vec<(&str, GeneralInput, Vec<ErrorCode>)> = vec![
        (
            "does not balance",
            entry(
                "10-10-2026",
                vec![line(cash, "1,000", ""), line(akram, "", "900")],
            ),
            vec![ErrGeneralNotBalanced],
        ),
        (
            "one line",
            entry("10-10-2026", vec![line(cash, "1,000", "")]),
            vec![ErrGeneralTooFewLines, ErrGeneralNotBalanced],
        ),
        (
            "no lines",
            entry("10-10-2026", vec![]),
            vec![ErrGeneralTooFewLines],
        ),
        (
            "both sides on one line",
            entry(
                "10-10-2026",
                vec![line(cash, "1,000", "1,000"), line(akram, "", "0")],
            ),
            vec![
                ErrOutOfRange,
                ErrFormatInvalid,
                ErrGeneralNotBalanced,
                ErrMustBeMoreThanZero,
                ErrFieldNotAllowed,
                ErrGeneralTooFewLines,
            ],
        ),
        (
            "zero on both lines",
            entry(
                "10-10-2026",
                vec![line(cash, "0", ""), line(akram, "", "0")],
            ),
            vec![
                ErrGeneralTooFewLines,
                ErrMustBeMoreThanZero,
                ErrRequired,
                ErrOutOfRange,
            ],
        ),
        (
            "decimals",
            entry(
                "10-10-2026",
                vec![line(cash, "100.50", ""), line(akram, "", "100.50")],
            ),
            vec![ErrDecimalsNotAllowed],
        ),
        (
            "negative",
            entry(
                "10-10-2026",
                vec![line(cash, "-5", ""), line(akram, "", "-5")],
            ),
            vec![ErrNegativeNotAllowed, ErrNotANumber],
        ),
        (
            "over the largest amount",
            entry(
                "10-10-2026",
                vec![
                    line(cash, "10,00,00,00,001", ""),
                    line(akram, "", "10,00,00,00,001"),
                ],
            ),
            vec![ErrAmountTooLarge],
        ),
        (
            "text for an amount",
            entry(
                "10-10-2026",
                vec![line(cash, "abc", ""), line(akram, "", "abc")],
            ),
            vec![ErrNotANumber],
        ),
        (
            "a date that is not a date",
            entry(
                "31-02-2026",
                vec![line(cash, "1,000", ""), line(akram, "", "1,000")],
            ),
            vec![ErrDateInvalid],
        ),
        (
            "00-00-0000",
            entry(
                "00-00-0000",
                vec![line(cash, "1,000", ""), line(akram, "", "1,000")],
            ),
            vec![ErrDateInvalid],
        ),
        (
            "a date in the future",
            entry(
                "10-10-2099",
                vec![line(cash, "1,000", ""), line(akram, "", "1,000")],
            ),
            vec![ErrDateInFuture],
        ),
        (
            "an account that is not there",
            entry(
                "10-10-2026",
                vec![line(cash, "1,000", ""), line(9_999_999, "", "1,000")],
            ),
            vec![ErrNotFound],
        ),
        (
            "no account on a line",
            GeneralInput {
                date: "10-10-2026".into(),
                narration: None,
                lines: vec![
                    line(cash, "1,000", ""),
                    GeneralLineInput {
                        account_id: None,
                        dr: String::new(),
                        cr: "1,000".into(),
                        narration: None,
                    },
                ],
                confirm_cash_below_zero: true,
            },
            vec![ErrRequired, ErrNotFound],
        ),
        (
            "a narration with a control character",
            GeneralInput {
                narration: Some("a\u{0}b".into()),
                ..entry(
                    "10-10-2026",
                    vec![line(cash, "1,000", ""), line(akram, "", "1,000")],
                )
            },
            vec![ErrControlChars],
        ),
        (
            "a narration over 500 characters",
            GeneralInput {
                narration: Some("ا".repeat(501)),
                ..entry(
                    "10-10-2026",
                    vec![line(cash, "1,000", ""), line(akram, "", "1,000")],
                )
            },
            vec![ErrTextTooLong],
        ),
    ];
    let mut failures = Vec::new();
    for (label, input, allowed) in cases {
        match vouchers::general_save(&w.env.ctx, &w.owner, &input) {
            Ok(_) => failures.push(format!("{label}: was SAVED")),
            Err(error) => {
                let got = codes(&error);
                if !got.iter().any(|c| allowed.contains(c)) {
                    failures.push(format!("{label}: wanted one of {allowed:?}, got {got:?}"));
                }
                if let ServiceError::Rejected(items) = &error
                    && items
                        .iter()
                        .any(|i| i.field.is_none() && FIELD_BOUND.contains(&i.code))
                {
                    failures.push(format!("{label}: an input error with no field: {items:?}"));
                }
            }
        }
        if counts(&w) != before {
            failures.push(format!(
                "{label}: refused, but the file changed (half-save)"
            ));
        }
    }
    let _ = rahim;
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// Codes that say a typed value is wrong: they must name a field.
const FIELD_BOUND: &[ErrorCode] = &[
    ErrorCode::ErrDecimalsNotAllowed,
    ErrorCode::ErrNegativeNotAllowed,
    ErrorCode::ErrNotANumber,
    ErrorCode::ErrAmountTooLarge,
    ErrorCode::ErrDateInvalid,
    ErrorCode::ErrDateInFuture,
    ErrorCode::ErrControlChars,
    ErrorCode::ErrTextTooLong,
];

/// E5: the buyer returns goods. There is no return screen yet, so the Owner makes a جنرل entry:
/// the buyer is credited, the seller (who takes the goods back) is debited. The books balance, the
/// buyer owes less, the entry is one voucher, and the same entry in Urdu digits and with a
/// grouped amount gives the same books.
#[test]
fn a_buyer_returns_goods_with_a_journal_entry_in_any_spelling_of_the_amount() {
    let w = world_with_event1();
    assert_eq!(balance(&w, w.akram).1, 12_880);
    let spellings = [
        "2000",
        "2,000",
        "۲۰۰۰",
        "٢٠٠٠",
        "1,000+1,000",
        "2\u{200b}000",
    ];
    let mut owed = 12_880;
    for text in spellings {
        let before = counts(&w);
        let saved = vouchers::general_save(
            &w.env.ctx,
            &w.owner,
            &entry(
                "10-10-2026",
                vec![line(w.rahim, text, ""), line(w.akram, "", text)],
            ),
        );
        match saved {
            Ok(_) => owed -= 2_000,
            Err(error) => panic!("{text:?} was refused: {:?}", codes(&error)),
        }
        let after = counts(&w);
        assert_eq!(after[0], before[0] + 1, "{text}: one journal entry");
        assert_eq!(after[2], before[2] + 1, "{text}: one voucher");
        assert_eq!(balance(&w, w.akram).1, owed, "{text}: Akram owes {owed}");
    }
    let conn = raw(&w.dir);
    let unbalanced: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM (SELECT entry_id FROM journal_lines GROUP BY entry_id HAVING SUM(dr_paisa) <> SUM(cr_paisa))",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(unbalanced, 0);
    common::assert_chain_ok(&w.dir);
}

/// E16 and PRD FR-6.6: a date on or before the "closed up to" day, a date in the future and a date
/// that cannot be are refused at the date field with a code, for the جنرل entry and the بکری
/// header alike, and nothing is written. The day after the closed day is still open.
#[test]
fn closed_future_and_impossible_dates_are_refused_at_their_field_and_write_nothing() {
    use mandi_services::core::vouchers::DayCloseInput;
    let w = world_with_event1();
    // Close up to 09-10-2026 (the books of the opening day balance, so the close is allowed).
    mandi_services::day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "09-10-2026".into(),
        },
    )
    .expect("close the day");
    let (cash, akram) = (id_of(&w.env, "01010001"), w.akram);
    let before = counts(&w);
    for (date, want) in [
        ("09-10-2026", ErrorCode::ErrDateClosed),
        ("08-10-2026", ErrorCode::ErrDateClosed),
        ("01-01-2020", ErrorCode::ErrDateClosed),
        ("11-10-2026", ErrorCode::ErrDateInFuture),
        ("10-10-2099", ErrorCode::ErrDateInFuture),
        ("31-02-2026", ErrorCode::ErrDateInvalid),
        ("00-00-0000", ErrorCode::ErrDateInvalid),
        ("29-02-2027", ErrorCode::ErrDateInvalid),
        ("2026-10-10", ErrorCode::ErrDateInvalid),
    ] {
        let general = entry(
            date,
            vec![line(cash, "1,000", ""), line(akram, "", "1,000")],
        );
        match vouchers::general_save(&w.env.ctx, &w.owner, &general) {
            Err(ServiceError::Rejected(items)) => assert!(
                items
                    .iter()
                    .any(|i| i.code == want && i.field.as_deref() == Some("date")),
                "general {date}: wanted {want:?} at date, got {items:?}"
            ),
            other => panic!("general {date}: {other:?}"),
        }
        let mut d = event1(&w);
        d.header.entry_date = date.into();
        let p = bikri::preview(&w.env.ctx, &d).unwrap();
        assert!(
            p.errors
                .iter()
                .any(|e| e.code == want && e.field == "header.entry_date"),
            "bikri {date}: wanted {want:?} at header.entry_date, got {:?}",
            p.errors
                .iter()
                .map(|e| (e.code, e.field.clone()))
                .collect::<Vec<_>>()
        );
        let saved = bikri::save(
            &w.env.ctx,
            &w.owner,
            &BikriSaveInput {
                draft: d,
                confirm_cash_below_zero: true,
            },
        );
        assert!(
            matches!(saved, Err(ServiceError::Rejected(_))),
            "bikri {date} was saved"
        );
        assert_eq!(
            counts(&w),
            before,
            "{date}: a refused entry wrote something"
        );
    }
    // The open day itself still saves.
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &entry(
            "10-10-2026",
            vec![line(cash, "1,000", ""), line(akram, "", "1,000")],
        ),
    )
    .expect("today is open");
}
