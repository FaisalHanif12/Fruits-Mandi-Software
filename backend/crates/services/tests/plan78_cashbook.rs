//! The cash book names the rows that start it and the rows that move a partner's capital (plan 78 E,
//! PRD FR-16.14): the opening cash, capital in and capital out, each with its partner, and the day of
//! the first entry that ever moved cash.
//!
//! The world is the worked day (E2E-8: cash 58,484 after Events 1 to 5) and E2E-35: on 11-10-26 ب
//! puts in 50,000 (108,484), and on 12-10-26 takes 10,000 back (98,484).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_services::core::capital::CapitalInput;
use mandi_services::core::cashbook::{CashKind, CashbookInput};
use mandi_services::{cashbook, vouchers};
use reports_world::*;

fn capital(date: &str, partner: i64, amount: &str) -> CapitalInput {
    CapitalInput {
        date: date.into(),
        partner_id: Some(partner),
        amount: amount.into(),
        narration: None,
        confirm_cash_below_zero: false,
    }
}

#[test]
fn the_opening_cash_and_the_capital_rows_are_named_with_their_partner() {
    let w = full_day();
    let b = w.partners[1];
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_capital_in(&w.env.ctx, &w.owner, &capital("11-10-2026", b, "50,000")).unwrap();
    set_today(&w.env, 2026, 10, 12);
    vouchers::save_capital_out(&w.env.ctx, &w.owner, &capital("12-10-2026", b, "10,000")).unwrap();

    let book = cashbook::get(&w.env.ctx, &CashbookInput::default()).unwrap();
    assert_eq!(book.first_entry_date.as_deref(), Some("2026-08-31"));
    assert_eq!(book.closing_paisa.rupees(), 98_484);
    // The first row is the opening cash of the cash account, 65,000 on 31-08-26.
    let first = book.rows.first().unwrap();
    assert_eq!(first.kind, CashKind::OpeningCash);
    assert_eq!(first.entry_date, "2026-08-31");
    assert_eq!(first.in_paisa.rupees(), 65_000);
    assert!(first.refs.partner.is_none());

    let find = |kind: CashKind| -> Vec<&mandi_services::core::cashbook::CashbookRow> {
        book.rows.iter().filter(|r| r.kind == kind).collect()
    };
    let capital_in = find(CashKind::CapitalIn);
    assert_eq!(capital_in.len(), 1);
    assert_eq!(capital_in[0].in_paisa.rupees(), 50_000);
    assert_eq!(capital_in[0].refs.partner.as_ref().map(|p| p.id), Some(b));
    let capital_out = find(CashKind::CapitalOut);
    assert_eq!(capital_out.len(), 1);
    assert_eq!(capital_out[0].out_paisa.rupees(), 10_000);
    assert_eq!(capital_out[0].refs.partner.as_ref().map(|p| p.id), Some(b));
    // Every other row is named for what it is; only a partner's own money names a partner.
    for row in book.rows.iter().filter(|r| {
        !matches!(
            r.kind,
            CashKind::OpeningCash | CashKind::CapitalIn | CashKind::CapitalOut
        )
    }) {
        assert_ne!(row.kind, CashKind::Other, "{}", row.posting_type);
        assert_eq!(
            row.refs.partner.is_some(),
            matches!(
                row.kind,
                CashKind::PersonalExpense | CashKind::PersonalReturn
            ),
            "{}",
            row.posting_type
        );
    }
    // A window that does not reach the first day still knows the first day of the book.
    let later = cashbook::get(
        &w.env.ctx,
        &CashbookInput {
            from: Some("11-10-2026".into()),
            ..CashbookInput::default()
        },
    )
    .unwrap();
    assert_eq!(later.first_entry_date.as_deref(), Some("2026-08-31"));
    assert_eq!(later.opening_paisa.rupees(), 58_484);
    assert_eq!(later.rows.len(), 2);
}

#[test]
fn a_book_with_no_cash_entry_has_no_first_day() {
    let w = support::world_with(0, false);
    let book = cashbook::get(&w.env.ctx, &CashbookInput::default()).unwrap();
    assert_eq!(book.first_entry_date, None);
    assert!(book.rows.is_empty());
}
