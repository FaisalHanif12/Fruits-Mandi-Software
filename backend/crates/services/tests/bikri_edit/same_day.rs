//! Two entries of one day keep their order when one of them is edited (migration 0005, PRD
//! FR-15.3, FR-15.4, FR-15.8, FR-15.12, plan 16): the new posting of an edit takes the order key of
//! the posting it replaces, so matching still sees it where it was.
//!
//! Before plan 16 an edited voucher was placed after every entry of its day, which could refuse a
//! return that was fine or leave an advance unadjusted.

use mandi_services::core::vouchers::{
    AdvanceInput, ReceiptInput, VoucherEditInput, VoucherEntry, VoucherMode,
};
use mandi_services::vouchers;

use crate::helpers::*;
use crate::support::*;
use crate::tools::{self, matched, open_advance_of};

fn voucher_version(w: &World, id: i64) -> i64 {
    vouchers::get(&w.env.ctx, id).unwrap().version
}

fn seller_return(date: &str, seller: i64, amount: &str) -> ReceiptInput {
    ReceiptInput {
        date: date.into(),
        account_id: seller,
        person_key: None,
        amount: amount.into(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        collector_staff_id: None,
        narration: String::new(),
    }
}

fn edit_voucher(
    w: &World,
    id: i64,
    entry: VoucherEntry,
) -> Result<mandi_services::core::vouchers::VoucherSaved, mandi_services::ServiceError> {
    vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id,
            version: voucher_version(w, id),
            entry,
            reason: "درستی".into(),
            confirm_cash_below_zero: true,
        },
    )
}

#[test]
fn a_same_day_seller_return_edited_after_a_ticked_bikri_stays_allowed() {
    let w = world();
    give_advance(&w, w.rahim, "15,000");
    // 10,000 of the advance comes back in cash, then Event 1 (ticked) adjusts the other 5,000.
    let back = vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &seller_return("10-10-2026", w.rahim, "10,000"),
    )
    .unwrap();
    assert_eq!(open_advance_of(&w, w.rahim), 5_000);
    save_ok(&w, &event1(&w));
    assert_eq!(open_advance_of(&w, w.rahim), 0);
    assert_eq!(matched(&w, "advance_allocations"), 15_000);
    // The return is corrected to 8,000 AFTER the بکری. In the order of the day it still comes
    // before it, so there is advance to return; the بکری then adjusts the other 7,000.
    edit_voucher(
        &w,
        back.voucher_id,
        VoucherEntry::Receipt(seller_return("10-10-2026", w.rahim, "8,000")),
    )
    .unwrap();
    assert_eq!(open_advance_of(&w, w.rahim), 0);
    assert_eq!(matched(&w, "advance_allocations"), 15_000);
    // The return's own credit line holds 8,000 of it.
    assert_eq!(
        scalar(
            &w,
            "SELECT COALESCE(SUM(a.amount_paisa), 0) FROM advance_allocations a
             JOIN journal_lines l ON l.id = a.credit_line_id
             JOIN journal_entries e ON e.id = l.entry_id
             WHERE a.is_active = 1 AND e.posting_type = 'receipt'"
        ),
        800_000
    );
    // The بکری now holds 7,000 of the advance. A return may be at most what is free plus what the
    // old version held (8,000), so 10,000 is refused with that limit, and 7,000 is fine.
    let version = voucher_version(&w, back.voucher_id);
    let too_much = vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: back.voucher_id,
            version,
            entry: VoucherEntry::Receipt(seller_return("10-10-2026", w.rahim, "10,000")),
            reason: String::new(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap_err();
    assert!(tools::refused(
        &too_much,
        mandi_services::core::error::ErrorCode::ErrReturnExceedsOpen,
        "amount"
    ));
    assert_eq!(
        tools::refusal_number(
            &too_much,
            mandi_services::core::error::ErrorCode::ErrReturnExceedsOpen,
            "open_paisa"
        ),
        Some(800_000)
    );
    edit_voucher(
        &w,
        back.voucher_id,
        VoucherEntry::Receipt(seller_return("10-10-2026", w.rahim, "7,000")),
    )
    .unwrap();
    assert_eq!(matched(&w, "advance_allocations"), 15_000);
    assert_eq!(tools::trial_difference(&w), 0);
}

#[test]
fn an_edited_advance_keeps_the_adjustment_the_bikri_made() {
    let w = world();
    give_advance(&w, w.rahim, "15,000");
    save_ok(&w, &event1(&w));
    assert_eq!(open_advance_of(&w, w.rahim), 0);
    let advance_id = scalar(&w, "SELECT id FROM vouchers WHERE type = 'advance'");
    let advance = |amount: &str| {
        VoucherEntry::Advance(AdvanceInput {
            confirm_cash_below_zero: true,
            date: "10-10-2026".into(),
            seller_account_id: w.rahim,
            amount: amount.into(),
            mode: VoucherMode::Cash,
            bank_account_id: None,
            cheque_no: String::new(),
            cheque_date: String::new(),
            expected_goods: String::new(),
            narration: String::new(),
        })
    };
    // The advance is corrected to 30,000 after the بکری took 15,000 of it. It is still the older
    // entry of the day, so the بکری's credit of 24,316 adjusts as much as it can: 24,316 of the
    // 30,000, and 5,684 is open. (Placed after the بکری, all 30,000 would be open.)
    edit_voucher(&w, advance_id, advance("30,000")).unwrap();
    assert_eq!(matched(&w, "advance_allocations"), 24_316);
    assert_eq!(open_advance_of(&w, w.rahim), 5_684);
    // Corrected down to 12,000: the بکری can adjust only 12,000.
    edit_voucher(&w, advance_id, advance("12,000")).unwrap();
    assert_eq!(matched(&w, "advance_allocations"), 12_000);
    assert_eq!(open_advance_of(&w, w.rahim), 0);
    assert_eq!(tools::trial_difference(&w), 0);
}

#[test]
fn a_buyers_oldest_bill_stays_oldest_after_the_bikri_is_edited() {
    // اکرم has two bills on one day: بکری 1 (12,880) and بکری 2 (10,680). He pays 12,880, which
    // settles the first. Editing بکری 1 posts a new first bill with a higher id; by its place in the
    // day it is still the oldest, so the 12,880 still goes to it first.
    let w = world();
    let one = save_ok(&w, &event1(&w));
    let mut second = draft(vec![with_sales(
        bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
        vec![sale("s-1", w.akram, "10", "1000")],
    )]);
    second.lots[0].cash_paid = None;
    let two = save_ok(&w, &second);
    tools::receipt(&w, w.akram, "12,880");
    let bill_two_matched = |w: &World| {
        scalar(
            w,
            &format!(
                "SELECT COALESCE(SUM(a.amount_paisa), 0) FROM bill_allocations a
                 JOIN journal_lines l ON l.id = a.bill_line_id
                 WHERE a.is_active = 1 AND l.entry_id = {}",
                two.entry_id
            ),
        )
    };
    assert_eq!(bill_two_matched(&w), 0);
    let opened = tools::open(&w, one.bikri_id, one.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "610".into();
    tools::edit(&w, &opened, &d);
    // Bill 1 is now 13,092: 12,880 of it is paid and بکری 2 is untouched.
    assert_eq!(bill_two_matched(&w), 0);
    assert_eq!(tools::open_bills(&w, w.akram), (212 + 10_680, 0));
    assert_eq!(matched(&w, "bill_allocations"), 12_880);
}
