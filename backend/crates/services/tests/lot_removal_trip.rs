//! Plan 80 Q-R2-1 (PRD FR-7.70, FR-12.26, C21, C25, E2E-34): removing a lot of trip goods in an edit gives
//! the goods and their cost back to the purchase, takes the trip partners' credits back, and leaves the
//! partners' payments from the shop as they were.
//!
//! E2E-34: 20 پیٹی مالٹا bought at 500, freight 400 (cost 10,400, paid from the shop 4,160 / 3,120 /
//! 3,120), and an own lot sold 15 of them to اکرم at 600 (cost released 7,800, credits 3,384 / 2,538 /
//! 2,538). The بکری gets a second lot (رحیم's 10 کیلا to ارشد at 580), then the own lot is removed.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;

use common::{assert_chain_ok, raw};
use mandi_services::{bikri, ledger};
use reports_world::*;

fn numbers(w: &support::World, sql: &str) -> Vec<i64> {
    let conn = raw(&w.dir);
    let mut statement = conn.prepare(sql).unwrap();
    statement
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn removing_the_own_lot_gives_the_goods_the_cost_and_the_credits_back_e2e_34() {
    let (w, _trip, purchase, bikri_id) = trip_34_world();
    let partner_totals = || {
        numbers(
            &w,
            "SELECT SUM(l.dr_paisa) - SUM(l.cr_paisa) FROM journal_lines l
             JOIN accounts a ON a.id = l.account_id
             JOIN account_controls k ON k.id = a.control_id
             WHERE k.code = '0604' GROUP BY a.id ORDER BY a.id",
        )
    };
    let purchase_row = || {
        numbers(
            &w,
            &format!(
                "SELECT qty_sold FROM purchases WHERE id = {purchase}
                 UNION ALL SELECT cost_released_paisa FROM purchases WHERE id = {purchase}"
            ),
        )
    };
    // Before: 15 sold, cost released 7,800; each partner paid his part and has his credit (E2E-34):
    // -4,160 + 3,384, -3,120 + 2,538, -3,120 + 2,538 in rupees, as debit minus credit in paisa.
    assert_eq!(purchase_row(), [15, 780_000]);
    assert_eq!(
        partner_totals(),
        [
            -(338_400 - 416_000),
            -(253_800 - 312_000),
            -(253_800 - 312_000)
        ]
    );

    // Edit 1: the بکری gets a second lot, رحیم's 10 کیلا to ارشد at 580, with no charges.
    let entry = numbers(
        &w,
        &format!(
            "SELECT id FROM journal_entries WHERE posting_type = 'bikri_first' AND source_id = {bikri_id}"
        ),
    )[0];
    let opened = tools::open(&w, bikri_id, entry);
    let mut draft = opened.draft.clone();
    assert_eq!(draft.lots.len(), 1);
    draft.lots.push(support::with_sales(
        support::bare_lot(&w, "lot-2", w.rahim, w.banana, "10"),
        vec![support::sale("s-2", w.arshad, "10", "580")],
    ));
    let added = tools::edit(&w, &opened, &draft);
    assert_eq!(
        purchase_row(),
        [15, 780_000],
        "adding a lot changes nothing of the trip"
    );

    // Edit 2: the own lot (the first) is removed.
    let opened = tools::open(&w, bikri_id, added.entry_id);
    assert_eq!(opened.draft.lots.len(), 2);
    let mut draft = opened.draft.clone();
    draft.lots.remove(0);
    tools::edit(&w, &opened, &draft);
    // The purchase has all 20 again, and none of its cost is released.
    assert_eq!(purchase_row(), [0, 0]);
    // Each partner is back to what he paid from the shop (C23: 4,160 / 3,120 / 3,120): the credits of
    // the removed lot are taken back (C25), the payments stay.
    assert_eq!(partner_totals(), [416_000, 312_000, 312_000]);
    // The own lot is a row, marked removed; the other lot is in force; the books balance.
    assert_eq!(
        numbers(
            &w,
            "SELECT removed_at IS NOT NULL FROM bikri_lots ORDER BY id"
        ),
        [1, 0]
    );
    assert_eq!(bikri::get(&w.env.ctx, bikri_id).unwrap().lots.len(), 1);
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}
