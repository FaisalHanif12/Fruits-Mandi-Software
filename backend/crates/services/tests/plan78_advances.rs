//! Advances to sellers in the partners' totals (plan 78 B4, PRD FR-17.13, C28, FR-17.9, FR-17.12).
//!
//! رحیم takes 15,000 on 01-09-26. The partners at 40 / 30 / 30 have 6,000 / 4,500 / 4,500 of it out,
//! so their کل رقم is lower by those parts until it is settled. A settlement comes back by the parts
//! still open of THAT advance (E2E-31: 10,000 comes back 4,000 / 3,000 / 3,000).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_services::cashbook;
use mandi_services::core::cashbook::{CapitalWhereKind, CapitalWhoseKind};
use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
use mandi_services::core::statement::{
    PartnersStatementInput, StatementDetail, StatementHeading, StatementLine,
};
use mandi_services::{partners, vouchers};
use proptest::prelude::*;
use reports_world::*;
use support::World;

fn advance(w: &World, date: &str, amount: &str) {
    vouchers::save_advance(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::AdvanceInput {
            confirm_cash_below_zero: false,
            date: date.into(),
            seller_account_id: w.rahim,
            amount: amount.into(),
            removed: Default::default(),
            expected_goods: String::new(),
            narration: String::new(),
            reason: None,
        },
    )
    .unwrap();
}

fn statement(w: &World, partner: i64) -> mandi_services::core::statement::PartnerStatement {
    partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            partner_id: partner,
            from: None,
            to: None,
            offset: 0,
            limit: 0,
            sort: None,
            group_by_month: false,
        },
    )
    .unwrap()
}

fn totals(w: &World) -> Vec<i64> {
    w.partners
        .iter()
        .map(|p| statement(w, *p).closing_total_paisa.rupees())
        .collect()
}

/// (day, detail, amount) of the advance lines of a partner's statement.
fn advance_lines(w: &World, partner: i64) -> Vec<(String, StatementDetail, i64)> {
    statement(w, partner)
        .lines
        .iter()
        .filter(|l: &&StatementLine| l.heading == StatementHeading::Advance)
        .map(|l| {
            (
                l.entry_date.clone(),
                l.detail,
                l.amount_paisa.unwrap().rupees(),
            )
        })
        .collect()
}

#[test]
fn an_advance_takes_each_partners_part_off_his_total_and_a_settlement_gives_it_back() {
    // E2E-31 numbers.
    let w = support::world();
    advance(&w, "01-09-2026", "15,000");
    assert_eq!(totals(&w), vec![-6_000, -4_500, -4_500]);
    // رحیم returns 10,000 in cash on 10-10-26: 4,000 / 3,000 / 3,000 come back.
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.rahim, "10,000")).unwrap();
    assert_eq!(totals(&w), vec![-2_000, -1_500, -1_500]);
    let b = w.partners[1];
    assert_eq!(
        advance_lines(&w, b),
        vec![
            ("2026-09-01".to_owned(), StatementDetail::AdvanceOut, -4_500),
            ("2026-10-10".to_owned(), StatementDetail::AdvanceBack, 3_000),
        ]
    );
    // The rest, 5,000, comes back 2,000 / 1,500 / 1,500 and nothing is out.
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.rahim, "5,000")).unwrap();
    assert_eq!(totals(&w), vec![0, 0, 0]);
    assert_eq!(advance_lines(&w, b).len(), 3);
}

#[test]
fn e2e_37_the_advance_lines_of_partner_b_and_his_total() {
    // After Events 1 to 5: رحیم's advance of 01-09-26 is settled by بکری 1 on 10-10-26. ب has
    // -4,500 on 01-09 and +4,500 on 10-10, and his کل رقم stays 746 (PRD E2E-37).
    let w = full_day();
    let b = w.partners[1];
    assert_eq!(
        advance_lines(&w, b),
        vec![
            ("2026-09-01".to_owned(), StatementDetail::AdvanceOut, -4_500),
            ("2026-10-10".to_owned(), StatementDetail::AdvanceBack, 4_500),
        ]
    );
    assert_eq!(totals(&w), vec![-1_004, 746, 746]);
    // The seller is named on the line and the share is shown.
    let line = statement(&w, b)
        .lines
        .into_iter()
        .find(|l| l.heading == StatementHeading::Advance)
        .unwrap();
    assert_eq!(line.subject.as_ref().map(|s| s.id), Some(w.rahim));
    assert_eq!(
        line.values.get("share_bps"),
        Some(&mandi_services::core::error::ErrorValue::Number(3_000))
    );
}

#[test]
fn a_later_share_change_does_not_change_what_comes_back() {
    let w = support::world();
    advance(&w, "01-09-2026", "15,000");
    assert_eq!(totals(&w), vec![-6_000, -4_500, -4_500]);
    // The shares become 50 / 25 / 25 from 11-10-26. The advance keeps its own parts.
    partners::set_shares(
        &w.env.ctx,
        &w.owner,
        &PartnersSetSharesInput {
            effective_from: "11-10-2026".into(),
            shares: w
                .partners
                .iter()
                .zip(["50", "25", "25"])
                .map(|(partner_id, share)| ShareInput {
                    partner_id: *partner_id,
                    share: share.into(),
                })
                .collect(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 11, 12);
    let mut paid = receipt(w.rahim, "15,000");
    paid.date = "12-11-2026".into();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &paid).unwrap();
    assert_eq!(totals(&w), vec![0, 0, 0]);
    let a = w.partners[0];
    assert_eq!(advance_lines(&w, a).last().unwrap().2, 6_000);
}

#[test]
fn cancelling_the_receipt_puts_the_advance_back_out() {
    let w = support::world();
    advance(&w, "01-09-2026", "15,000");
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.rahim, "10,000")).unwrap();
    assert_eq!(totals(&w), vec![-2_000, -1_500, -1_500]);
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::VoucherCancelInput {
            id: saved.voucher_id,
            version: vouchers::get(&w.env.ctx, saved.voucher_id).unwrap().version,
            reason: "wrong".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    assert_eq!(totals(&w), vec![-6_000, -4_500, -4_500]);
    let b = w.partners[1];
    assert_eq!(
        advance_lines(&w, b).len(),
        1,
        "the back line goes with the cancelled receipt"
    );
}

/// What the advances to sellers still have out, in rupees, read with plain SQL.
fn open_advances_sql(w: &World) -> i64 {
    w.env
        .ctx
        .db()
        .read(|conn| {
            conn.query_row(
                "SELECT COALESCE(SUM(l.dr_paisa - COALESCE((SELECT SUM(a.amount_paisa) FROM advance_allocations a
                                                            WHERE a.advance_line_id = l.id AND a.is_active = 1), 0)), 0) / 100
                 FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                 WHERE l.is_advance = 1 AND l.dr_paisa > 0 AND e.status = 'posted' AND e.reverses_entry_id IS NULL
                   AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id AND r.status = 'posted')",
                [],
                |r| r.get::<_, i64>(0),
            )
        })
        .unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 8, ..ProptestConfig::default() })]

    /// PRD FR-17.13, FR-17.9: after every step of a random run of advances and receipts from the seller,
    /// the partners' open parts equal the advances still out, no part is below 0, and the card
    /// "where the capital is" balances with no `other` line (Σ کل رقم + opening capital + open advances =
    /// the equity of the books).
    #[test]
    fn the_partners_open_parts_always_equal_the_advances_still_out(
        steps in prop::collection::vec((any::<bool>(), 1i64..=30), 1..=6),
    ) {
        let w = support::world();
        for (is_advance, thousands) in steps {
            let open = open_advances_sql(&w);
            if is_advance {
                let input = mandi_services::core::vouchers::AdvanceInput {
                    confirm_cash_below_zero: true,
                    date: "10-10-2026".into(),
                    seller_account_id: w.rahim,
                    amount: format!("{}", thousands * 1_000),
                    removed: Default::default(),
                    expected_goods: String::new(),
                    narration: String::new(),
                    reason: None,
                };
                vouchers::save_advance(&w.env.ctx, &w.owner, &input).unwrap();
            } else if open > 0 {
                // A receipt that returns part or all of what is open.
                let amount = (thousands * 1_000).min(open);
                vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.rahim, &amount.to_string())).unwrap();
            }
            let card = cashbook::capital_where(&w.env.ctx).unwrap();
            prop_assert_eq!(card.net_paisa, card.whose_total_paisa);
            prop_assert!(card.have_lines.iter().chain(&card.owe_lines).all(|l| l.kind != CapitalWhereKind::Other));
            prop_assert!(card.whose_lines.iter().all(|l| l.kind != CapitalWhoseKind::Other));
            let parts: i64 = card
                .whose_lines
                .iter()
                .filter(|l| l.kind == CapitalWhoseKind::AdvancesOpen)
                .map(|l| l.amount_paisa.rupees())
                .sum();
            prop_assert_eq!(parts, open_advances_sql(&w));
        }
    }
}
