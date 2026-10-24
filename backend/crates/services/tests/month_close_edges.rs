//! Month close on the months that are not the worked October (plan d5-late round 7, PRD FR-17.6,
//! FR-17.9, FR-6.5, C10, C11, C22): a change of shares in the middle of the month, a partner who
//! left, a month whose profit is exactly 0, an income account with a debit net, and amounts
//! that do not divide evenly between the partners.
//!
//! In every case the same things must hold: the entry balances, the partners' capital credits
//! add up to the profit of the lines exactly, nobody's total (کل رقم) moves, the trial balance
//! stays equal, and the reversal puts every account back. The expected numbers are worked by
//! hand in each test from the PRD rules (C10 for the split, C11 for expenses).
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::expenses::ShopExpenseInput;
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
use mandi_services::core::masters::MasterKind;
use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::month::{
    MonthInput, MonthLineKind, MonthReverseInput, close, preview, reverse,
};
use mandi_services::{day, ledger, masters, partners, vouchers};
use reports_world::*;
use support::World;

fn input() -> MonthInput {
    MonthInput {
        month: "2026-10".into(),
    }
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

/// October is over (today 02-11-26) and all its days are closed.
fn end_of_october(w: &World) {
    set_today(&w.env, 2026, 11, 2);
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "31-10-2026".into(),
        },
    )
    .unwrap();
}

fn electricity(w: &World) -> i64 {
    masters::expense_heads(&w.env.ctx)
        .unwrap()
        .rows
        .iter()
        .find(|r| r.head.name_en == "Electricity")
        .unwrap()
        .id
}

fn shop_expense(w: &World, date: &str, amount: &str) {
    vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &ShopExpenseInput {
            date: date.into(),
            expense_head_id: Some(electricity(w)),
            amount: amount.into(),
            paid_to: None,
            narration: None,
            // An awkward amount may take the cash below zero; the question is not the test.
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
}

fn capital_credits(w: &World) -> Vec<i64> {
    preview(&w.env.ctx, &input())
        .unwrap()
        .lines
        .iter()
        .filter(|l| l.kind == MonthLineKind::PartnerCapital)
        .map(|l| l.cr_paisa.rupees() - l.dr_paisa.rupees())
        .collect()
}

/// Closes October, then checks what must hold in every month, and reverses it.
fn close_and_check(w: &World) {
    let before = (trial(w), totals(w));
    let shown = preview(&w.env.ctx, &input()).unwrap();
    // The capital credits are exactly the profit of the lines.
    let shares: i64 = capital_credits(w).iter().sum();
    assert_eq!(shares, shown.profit_paisa.rupees());
    close(&w.env.ctx, &w.owner, &input()).unwrap();
    let (dr, cr) = trial(w);
    assert_eq!(dr, cr, "the books balance after the close");
    assert_eq!(totals(w), before.1, "nobody's total moves");
    assert_chain_ok(&w.dir);
    reverse(
        &w.env.ctx,
        &w.owner,
        &MonthReverseInput {
            month: "2026-10".into(),
            reason: "check".into(),
        },
    )
    .unwrap();
    assert_eq!(trial(w), before.0, "the reversal puts every account back");
    assert_eq!(totals(w), before.1);
    assert_chain_ok(&w.dir);
}

#[test]
fn a_change_of_shares_in_the_month_gives_each_posting_its_own_shares() {
    // Event 1 to 5 on 10-10-26 at 40 / 30 / 30. From 15-10-26 the shares are 50 / 25 / 25, and a
    // shop expense of 1,000 on 16-10-26 is split 500 / 250 / 250 (C11).
    // Profit shares: الف 996 - 500 = 496; ب 746 - 250 = 496; ج 496. Profit 5,488 - 4,000 = 1,488.
    let w = full_day();
    let [a, b, c] = w.partners;
    set_today(&w.env, 2026, 10, 15);
    partners::set_shares(
        &w.env.ctx,
        &w.owner,
        &PartnersSetSharesInput {
            effective_from: "15-10-2026".into(),
            shares: [(a, "50"), (b, "25"), (c, "25")]
                .iter()
                .map(|&(partner_id, share)| ShareInput {
                    partner_id,
                    share: share.to_owned(),
                })
                .collect(),
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 16);
    shop_expense(&w, "16-10-2026", "1,000");
    end_of_october(&w);
    let shown = preview(&w.env.ctx, &input()).unwrap();
    assert_eq!(shown.profit_paisa.rupees(), 1_488);
    assert_eq!(capital_credits(&w), [496, 496, 496]);
    close_and_check(&w);
}

#[test]
fn a_partner_who_left_still_gets_the_profit_he_earned_in_the_month() {
    // ج leaves after October's postings. His 746 stays his: it is credited to his capital.
    let w = full_day();
    let [a, b, c] = w.partners;
    set_today(&w.env, 2026, 10, 12);
    // A partner with a share cannot be made inactive (ERR_PARTNER_HAS_SHARE): his share goes to 0
    // from 12-10-26 first.
    partners::set_shares(
        &w.env.ctx,
        &w.owner,
        &PartnersSetSharesInput {
            effective_from: "12-10-2026".into(),
            shares: [(a, "60"), (b, "40"), (c, "0")]
                .iter()
                .map(|&(partner_id, share)| ShareInput {
                    partner_id,
                    share: share.to_owned(),
                })
                .collect(),
        },
    )
    .unwrap();
    let detail = partners::get(&w.env.ctx, c).unwrap();
    partners::update(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::partners::PartnerUpdateInput {
            id: c,
            version: detail.row.version,
            name_ur: detail.row.name_ur.clone(),
            name_en: detail.row.name_en.clone(),
            is_active: false,
            confirmed_rules_words: Vec::new(),
        },
    )
    .unwrap();
    end_of_october(&w);
    assert_eq!(capital_credits(&w), [996, 746, 746]);
    close_and_check(&w);
}

#[test]
fn a_month_with_a_profit_of_exactly_nothing_has_no_capital_lines_and_still_closes() {
    // An extra expense of 2,488 on 10-10-26 makes expenses 5,488 = income 5,488. C10 on 2,488 at
    // 40 / 30 / 30: 995 + 746 + 746 = 2,487 and the leftover rupee goes to الف: 996 / 746 / 746.
    // Every partner's share of the month is 996 - 996, 746 - 746, 746 - 746 = 0.
    let w = full_day();
    shop_expense(&w, "10-10-2026", "2,488");
    end_of_october(&w);
    let shown = preview(&w.env.ctx, &input()).unwrap();
    assert_eq!(shown.profit_paisa.rupees(), 0);
    assert!(
        shown
            .lines
            .iter()
            .all(|l| l.kind != MonthLineKind::PartnerCapital),
        "a partner whose month is 0 has no line"
    );
    let before = (trial(&w), totals(&w));
    close(&w.env.ctx, &w.owner, &input()).unwrap();
    let (dr, cr) = trial(&w);
    assert_eq!(dr, cr);
    assert_eq!(totals(&w), before.1);
    // Every income and expense account is 0 now.
    let leftover: i64 = raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM (
                SELECT a.id FROM journal_lines l
                JOIN accounts a ON a.id = l.account_id JOIN journal_entries e ON e.id = l.entry_id
                WHERE e.status = 'posted' AND substr(a.code, 1, 2) IN ('04', '05')
                GROUP BY a.id HAVING SUM(l.dr_paisa) <> SUM(l.cr_paisa))",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(leftover, 0);
}

#[test]
fn an_income_account_with_a_debit_net_is_closed_from_the_credit_side() {
    // A جنرل puts 1,000 on the debit of ریٹ فرق (04010005): a negative rate difference. The
    // account has a debit net of 1,000, so the close credits it 1,000, and the income of the
    // month is 5,488 - 1,000 = 4,488; the profit is 4,488 - 3,000 = 1,488.
    let w = full_day();
    let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
        account_id: Some(account),
        person_key: None,
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    };
    vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &GeneralInput {
            date: "10-10-2026".into(),
            narration: None,
            lines: vec![
                line(support::id_of(&w.env, "04010005"), "1,000", ""),
                line(support::id_of(&w.env, "01010001"), "", "1,000"),
            ],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    end_of_october(&w);
    let shown = preview(&w.env.ctx, &input()).unwrap();
    let rate = shown
        .lines
        .iter()
        .find(|l| l.account.code == "04010005")
        .expect("a line for the account with a debit net");
    assert_eq!((rate.dr_paisa.rupees(), rate.cr_paisa.rupees()), (0, 1_000));
    assert_eq!(rate.kind, MonthLineKind::Income);
    assert_eq!(shown.profit_paisa.rupees(), 1_488);
    close_and_check(&w);
}

#[test]
fn amounts_that_do_not_divide_evenly_still_balance_to_the_rupee() {
    // Shop expenses of awkward sizes, each in its own world: the leftover rupee of C10/C11 goes
    // to the biggest share, and the capital credits always add up to the profit.
    for amount in ["1", "7", "3,384", "99,999"] {
        let w = full_day();
        shop_expense(&w, "10-10-2026", amount);
        end_of_october(&w);
        close_and_check(&w);
    }
}

#[test]
fn a_month_can_be_closed_after_an_expense_head_it_used_was_made_inactive() {
    // بجلی 3,000 is posted on 10-10-26. On 02-11-26 the Owner makes the head inactive, which also
    // makes its 0501 account inactive. The close only moves a balance that is already posted, so
    // it still goes through (PRD FR-17.6). A new posting to the inactive account is still refused.
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
    let head = electricity(&w);
    masters::set_active(&w.env.ctx, &w.owner, MasterKind::ExpenseHead, head, false).unwrap();
    // A normal posting to the inactive head is refused.
    let refused = vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &ShopExpenseInput {
            date: "02-11-2026".into(),
            expense_head_id: Some(head),
            amount: "100".into(),
            paid_to: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    );
    match refused {
        Err(mandi_services::ServiceError::Rejected(items)) => {
            assert_eq!(items[0].code, ErrorCode::ErrAccountInactive);
        }
        other => panic!("a new expense on an inactive head must be refused: {other:?}"),
    }
    let before = (trial(&w), totals(&w));
    close(&w.env.ctx, &w.owner, &input()).unwrap();
    let (dr, cr) = trial(&w);
    assert_eq!(dr, cr);
    assert_eq!(totals(&w), before.1, "nobody's total moves");
    // The shop expense account is 0 after the close.
    let left: i64 = raw(&w.dir)
        .query_row(
            "SELECT COALESCE(SUM(l.dr_paisa), 0) - COALESCE(SUM(l.cr_paisa), 0)
             FROM journal_lines l JOIN accounts a ON a.id = l.account_id
             JOIN journal_entries e ON e.id = l.entry_id
             WHERE e.status = 'posted' AND a.code = '05010001'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(left, 0);
    // Its reversal also goes through, and the month can be closed again.
    reverse(
        &w.env.ctx,
        &w.owner,
        &MonthReverseInput {
            month: "2026-10".into(),
            reason: "check".into(),
        },
    )
    .unwrap();
    assert_eq!(trial(&w), before.0);
}
