//! Bills and advances after an edit, a cancel and a later sale (PRD FR-12.22, FR-15.4, FR-15.8,
//! E14, E15, Architecture 12). Every reversal is followed by a full re-match of the parties it
//! touches, oldest first.

use mandi_services::bikri;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{BikriDraft, LaterSaleSaveInput};

use crate::helpers::*;
use crate::support::*;
use crate::tools::{self, matched, open_advance_of};

/// Event 1 with only 45 of the 50 sold (اکرم 20 at 600, ارشد 25 at 580), as in E2E-16.
fn part_sold(w: &World) -> BikriDraft {
    let mut d = event1(w);
    d.lots[0].sales[1].qty = "25".into();
    d
}

fn later_five(w: &World, lot_id: i64, date: &str) -> mandi_services::core::bikri::LaterSaleSaved {
    bikri::later_sale_save(
        &w.env.ctx,
        &w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: date.into(),
            sales: vec![sale("l-1", w.arshad, "5", "560")],
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
}

#[test]
fn an_advance_is_adjusted_by_the_first_posting_and_by_a_later_sale_and_an_edit_keeps_it_right() {
    let w = world();
    give_advance(&w, w.rahim, "25,000");
    let saved = save_ok(&w, &part_sold(&w));
    // The seller credit of the first posting: 26,500 − 1,590 − 3,320 = 21,590.
    assert_eq!(matched(&w, "advance_allocations"), 21_590);
    assert_eq!(open_advance_of(&w, w.rahim), 3_410);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let lot_id = opened.draft.lots[0].lot_id.unwrap();
    // A later sale on the same day: 5 × 560 = 2,800 − 168 = 2,632 more goes against the advance.
    let later = later_five(&w, lot_id, "10-10-2026");
    assert_eq!(matched(&w, "advance_allocations"), 21_590 + 2_632);
    assert_eq!(open_advance_of(&w, w.rahim), 778);
    // Edit اکرم's rate in the FIRST posting to 610: its credit becomes 21,778 and the later
    // credit is still adjusted, so the open advance is 25,000 − 21,778 − 2,632 = 590.
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "610".into();
    tools::edit(&w, &opened, &d);
    assert_eq!(open_advance_of(&w, w.rahim), 590);
    assert_eq!(matched(&w, "advance_allocations"), 21_778 + 2_632);
    // Edit the later sale: 5 at 570 → 2,850 − 171 = 2,679.
    let later_open = tools::open(&w, saved.bikri_id, later.entry_id);
    assert!(later_open.editable && later_open.posting_type == "bikri_later");
    let mut d = later_open.draft.clone();
    d.lots[0].sales[0].seller_rate = "570".into();
    tools::edit(&w, &later_open, &d);
    assert_eq!(matched(&w, "advance_allocations"), 21_778 + 2_679);
    assert_eq!(open_advance_of(&w, w.rahim), 543);
    // The seller owes nothing beyond the advance: the ledger is credit 24,457 − 5,000 − 25,000 + ...
    // so only the totals are checked: the books balance and every credit line is matched at most
    // once.
    assert_eq!(tools::trial_difference(&w), 0);
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM (SELECT credit_line_id FROM advance_allocations WHERE is_active = 1
               GROUP BY credit_line_id HAVING SUM(amount_paisa) >
                 (SELECT cr_paisa FROM journal_lines WHERE id = credit_line_id))"
        ),
        0
    );
}

#[test]
fn an_edit_after_a_receipt_matches_the_bill_again_and_back() {
    let w = world();
    let saved = save_ok(&w, &event1(&w));
    tools::receipt(&w, w.akram, "10,000");
    assert_eq!(tools::open_bills(&w, w.akram), (2_880, 0));
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "610".into();
    let edited = tools::edit(&w, &opened, &d);
    // His bill is now 13,092: 10,000 paid, 3,092 open.
    assert_eq!(tools::open_bills(&w, w.akram), (3_092, 0));
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 3_092));
    assert_eq!(matched(&w, "bill_allocations"), 10_000);
    let again = tools::open(&w, saved.bikri_id, edited.entry_id);
    let mut d = again.draft.clone();
    d.lots[0].sales[0].seller_rate = "600".into();
    tools::edit(&w, &again, &d);
    assert_eq!(tools::open_bills(&w, w.akram), (2_880, 0));
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 2_880));
    // ارشد's bill was never touched by any of this.
    assert_eq!(tools::open_bills(&w, w.arshad), (18_684, 0));
}

#[test]
fn an_edit_that_makes_a_bill_smaller_than_the_receipt_leaves_the_extra_as_credit() {
    // اکرم paid 10,000. The edit sells him only 5 at 600: 3,000 + 180 + 40 = 3,220. His receipt is
    // then more than his bill: 6,780 stays as his credit (PRD FR-15.5, E11).
    let w = world();
    let saved = save_ok(&w, &event1(&w));
    tools::receipt(&w, w.akram, "10,000");
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].qty = "5".into();
    tools::edit(&w, &opened, &d);
    assert_eq!(tools::open_bills(&w, w.akram), (0, 6_780));
    assert_eq!(balance(&w, w.akram), (BalanceSide::Cr, 6_780));
    assert_eq!(matched(&w, "bill_allocations"), 3_220);
}

#[test]
fn a_cancel_turns_matched_receipts_into_credit_that_the_next_bill_takes() {
    let w = world();
    let saved = save_ok(&w, &event1(&w));
    tools::receipt(&w, w.akram, "10,000");
    tools::cancel(&w, saved.bikri_id, true).unwrap();
    // No bill left: his 10,000 is credit, nothing is matched.
    assert_eq!(tools::open_bills(&w, w.akram), (0, 10_000));
    assert_eq!(balance(&w, w.akram), (BalanceSide::Cr, 10_000));
    assert_eq!(matched(&w, "bill_allocations"), 0);
    // A new بکری for him is settled by that credit, oldest first.
    save_ok(&w, &event1(&w));
    assert_eq!(tools::open_bills(&w, w.akram), (2_880, 0));
    assert_eq!(matched(&w, "bill_allocations"), 10_000);
}
