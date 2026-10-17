//! Property tests of plan 15 (Code Standards 9.3, money agent Section 11.2), on a real database:
//! random runs of receipts, cash lent, payments, advances, بکری, edits and cancels keep the books
//! exact, and the preview of every receipt is what its save does.
//!
//! - A buyer's open bills less his unmatched credit equal his balance, after any run.
//! - A seller's payable working ties to his ledger: open − advances open − owed beyond = credit
//!   less debit; his advance open equals given − returned − adjusted and is never below 0.
//! - After any run the stored matching rows are the canonical ones: working them out again
//!   changes nothing.
//! - The preview of a receipt shows the bills it settles and the credit that stays; after the
//!   save the bills are as shown.
//! - A receipt from a seller never makes more payable to him than before it.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::BikriSaveInput;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::{
    AdvanceInput, AdvancesReportInput, OpenBillsInput, OpenPayablesInput, PaymentInput,
    ReceiptInput, VoucherCancelInput, VoucherEditInput, VoucherEntry, VoucherMode,
    VoucherPreviewInput, VoucherType,
};
use mandi_services::{ServiceError, bikri, ledger, vouchers};
use proptest::prelude::*;
use support::World;

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn day(offset: u8) -> String {
    format!("{:02}-10-2026", 6 + offset % 5)
}

fn receipt(account: i64, date: &str, amount: i64) -> ReceiptInput {
    ReceiptInput {
        date: date.into(),
        account_id: account,
        person_key: None,
        amount: amount.to_string(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        collector_staff_id: None,
        narration: String::new(),
    }
}

fn payment(account: i64, date: &str, amount: i64, reason: &str) -> PaymentInput {
    PaymentInput {
        date: date.into(),
        account_id: account,
        person_key: None,
        amount: amount.to_string(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        reason: reason.into(),
        narration: String::new(),
        confirm_cash_below_zero: true,
    }
}

fn advance(seller: i64, date: &str, amount: i64) -> AdvanceInput {
    AdvanceInput {
        date: date.into(),
        seller_account_id: seller,
        amount: amount.to_string(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        expected_goods: String::new(),
        narration: String::new(),
        confirm_cash_below_zero: true,
    }
}

/// A world with plenty of cash so the cash question never decides a case.
fn rich_world() -> World {
    support::world_with(100_000_000, true)
}

fn matching_is_canonical(w: &World, buyer: i64, seller: i64) {
    let now = w.env.ctx.now_text();
    let (off, added) = w
        .env
        .ctx
        .db()
        .write(|tx| {
            let b = mandi_services::matching::rematch(tx, w.owner.user_id, &now, buyer, None)?;
            let s = mandi_services::matching::rematch_advances(tx, w.owner.user_id, &now, seller)?;
            Ok::<_, ServiceError>((b.switched_off + s.switched_off, b.added + s.added))
        })
        .unwrap();
    assert_eq!(
        (off, added),
        (0, 0),
        "the stored rows are not the canonical ones"
    );
}

fn signed(balance: mandi_services::core::accounts::Balance) -> i64 {
    match balance.side {
        BalanceSide::Cr => -balance.amount_paisa.as_paisa(),
        _ => balance.amount_paisa.as_paisa(),
    }
}

fn live_vouchers(w: &World, kinds: &[VoucherType]) -> Vec<(i64, VoucherType)> {
    vouchers::list(
        &w.env.ctx,
        &mandi_services::core::vouchers::VouchersListInput {
            voucher_type: None,
            from: String::new(),
            to: String::new(),
            account_id: None,
            person_key: None,
        },
    )
    .unwrap()
    .rows
    .into_iter()
    .filter(|v| v.status == "saved" && kinds.contains(&v.voucher_type))
    .map(|v| (v.id, v.voucher_type))
    .collect()
}

/// The invariants of a buyer after any step.
fn check_buyer(w: &World) {
    let bills = vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: w.akram,
            person_key: None,
        },
    )
    .unwrap();
    // Σ open bills − unmatched credit = his balance (debit positive).
    assert_eq!(
        bills.open_paisa.as_paisa() - bills.unmatched_credit_paisa.as_paisa(),
        signed(bills.balance),
        "open bills and credit do not tie to the balance"
    );
    // A bill is never open for more than it is, nor negative; they are oldest first.
    let mut last: Option<&str> = None;
    for bill in &bills.bills {
        assert!(bill.open_paisa > Paisa::ZERO && bill.open_paisa <= bill.amount_paisa);
        if let Some(previous) = last {
            assert!(previous <= bill.entry_date.as_str());
        }
        last = Some(bill.entry_date.as_str());
    }
    // Credit waiting and a bill open at the same time cannot both be: the credit would settle it.
    assert!(
        bills.unmatched_credit_paisa == Paisa::ZERO || bills.bills.is_empty(),
        "credit waits while a bill is open"
    );
}

/// The invariants of a seller after any step.
fn check_seller(w: &World) {
    let p = vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: w.rahim,
            person_key: None,
            include_settled: true,
        },
    )
    .unwrap();
    // open − advances open − owed beyond = credit less debit.
    assert_eq!(
        p.open_paisa.as_paisa() - p.advances_open_paisa.as_paisa() - p.owes_firm_paisa.as_paisa(),
        -signed(p.balance),
        "the payable working does not tie to the ledger"
    );
    assert!(p.open_paisa == Paisa::ZERO || p.owes_firm_paisa == Paisa::ZERO);
    // The short read (sums and a walk back from the newest) shows the postings the full working
    // shows, and the same totals.
    let short = vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: w.rahim,
            person_key: None,
            include_settled: false,
        },
    )
    .unwrap();
    let still_open: Vec<_> = p
        .postings
        .iter()
        .filter(|posting| posting.still_open_paisa > Paisa::ZERO)
        .cloned()
        .collect();
    assert_eq!(short.postings, still_open);
    assert_eq!(
        (
            short.open_paisa,
            short.advances_open_paisa,
            short.owes_firm_paisa,
            short.balance
        ),
        (
            p.open_paisa,
            p.advances_open_paisa,
            p.owes_firm_paisa,
            p.balance
        )
    );
    for posting in &p.postings {
        assert!(posting.still_open_paisa >= Paisa::ZERO);
        assert_eq!(
            posting.credit_paisa.as_paisa(),
            posting.advance_adjusted_paisa.as_paisa()
                + posting.paid_paisa.as_paisa()
                + posting.still_open_paisa.as_paisa()
        );
    }
    // The advances report: open = given - returned - adjusted >= 0, and equals the card.
    let report = vouchers::advances_report(
        &w.env.ctx,
        &AdvancesReportInput {
            seller_id: Some(w.rahim),
            as_of: String::new(),
        },
    )
    .unwrap();
    let open = report.rows.first().map_or(Paisa::ZERO, |r| r.open_paisa);
    if let Some(row) = report.rows.first() {
        assert_eq!(
            row.open_paisa.as_paisa(),
            row.given_paisa.as_paisa()
                - row.returned_paisa.as_paisa()
                - row.adjusted_paisa.as_paisa()
        );
        assert!(row.open_paisa >= Paisa::ZERO);
    }
    assert_eq!(open, p.advances_open_paisa);
    assert_eq!(
        open,
        mandi_services::assist::advances(&w.env.ctx, w.rahim)
            .unwrap()
            .open_paisa
    );
    // The books balance.
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[derive(Debug, Clone)]
struct Step {
    op: u8,
    day: u8,
    amount: u16,
    pick: u8,
}

fn steps() -> impl Strategy<Value = Vec<Step>> {
    proptest::collection::vec(
        (0_u8..9, 0_u8..5, 1_u16..60, any::<u8>()).prop_map(|(op, day, amount, pick)| Step {
            op,
            day,
            amount,
            pick,
        }),
        1..14,
    )
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, failure_persistence: None, ..ProptestConfig::default() })]

    /// A buyer: bills (cash lent), receipts with a preview each, edits and cancels.
    #[test]
    fn a_buyer_stays_exact_and_the_preview_is_the_save(run in steps()) {
        let w = rich_world();
        for step in &run {
            let date = day(step.day);
            let amount = i64::from(step.amount) * 100;
            match step.op % 4 {
                // A bill: cash lent to the buyer.
                0 => {
                    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.akram, &date, amount, "x")).unwrap();
                }
                // A receipt, previewed first.
                1 | 2 => {
                    let input = receipt(w.akram, &date, amount);
                    let preview = vouchers::preview(&w.env.ctx, &VoucherPreviewInput {
                        entry: VoucherEntry::Receipt(input.clone()),
                        replacing_voucher_id: None,
                    }).unwrap();
                    prop_assert!(preview.errors.is_empty());
                    let before = vouchers::open_bills(&w.env.ctx, &OpenBillsInput { account_id: w.akram, person_key: None }).unwrap();
                    vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
                    let after = vouchers::open_bills(&w.env.ctx, &OpenBillsInput { account_id: w.akram, person_key: None }).unwrap();
                    // Every bill the preview named is as shown; every other bill is unchanged.
                    for bill in &before.bills {
                        let now = after.bills.iter().find(|b| b.line_id == bill.line_id).map_or(Paisa::ZERO, |b| b.open_paisa);
                        match preview.bill_effects.iter().find(|e| e.bill.line_id == bill.line_id) {
                            Some(effect) => {
                                prop_assert_eq!(effect.still_open_paisa, now);
                                prop_assert_eq!(effect.bill.open_paisa, bill.open_paisa);
                                prop_assert_eq!(effect.settled_paisa.as_paisa(), bill.open_paisa.as_paisa() - now.as_paisa());
                            }
                            None => prop_assert_eq!(bill.open_paisa, now),
                        }
                    }
                    prop_assert_eq!(
                        preview.extra_credit_paisa.as_paisa(),
                        after.unmatched_credit_paisa.as_paisa() - before.unmatched_credit_paisa.as_paisa()
                    );
                    prop_assert_eq!(signed(preview.new_balance), signed(after.balance));
                }
                // Edit or cancel an earlier receipt or loan.
                _ => {
                    let live = live_vouchers(&w, &[VoucherType::Receipt, VoucherType::Payment]);
                    if live.is_empty() {
                        continue;
                    }
                    let (id, kind) = live[usize::from(step.pick) % live.len()];
                    let row = vouchers::get(&w.env.ctx, id).unwrap();
                    if step.op % 2 == 0 {
                        vouchers::cancel(&w.env.ctx, &w.owner, &VoucherCancelInput {
                            id, version: row.version, reason: "x".into(), confirm_cash_below_zero: true,
                        }).unwrap();
                    } else {
                        let entry = match kind {
                            VoucherType::Receipt => VoucherEntry::Receipt(receipt(w.akram, &date, amount)),
                            _ => VoucherEntry::Payment(payment(w.akram, &date, amount, "y")),
                        };
                        // The preview of the edit says the same as the edit.
                        let preview = vouchers::preview(&w.env.ctx, &VoucherPreviewInput {
                            entry: entry.clone(),
                            replacing_voucher_id: Some(id),
                        }).unwrap();
                        prop_assert!(preview.errors.is_empty());
                        let saved = vouchers::edit(&w.env.ctx, &w.owner, &VoucherEditInput {
                            id, version: row.version, entry, reason: String::new(), confirm_cash_below_zero: true,
                        }).unwrap();
                        prop_assert_eq!(signed(preview.new_balance), signed(saved.new_balance));
                    }
                }
            }
            check_buyer(&w);
            matching_is_canonical(&w, w.akram, w.rahim);
        }
    }

    /// A seller: advances, بکری credits (ticked or not), payments, returns, edits and cancels.
    #[test]
    fn a_seller_stays_exact_and_a_return_never_raises_the_payable(run in steps()) {
        let w = rich_world();
        for step in &run {
            let date = day(step.day);
            let amount = i64::from(step.amount) * 100;
            match step.op % 6 {
                0 => {
                    vouchers::save_advance(&w.env.ctx, &w.owner, &advance(w.rahim, &date, amount)).unwrap();
                }
                1 => {
                    // A بکری of one line: qty 1 at `amount`, the tick follows the pick.
                    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "1");
                    lot.adjust_advance = step.pick % 2 == 0;
                    lot.sales = vec![support::sale("s-1", w.akram, "1", &amount.to_string())];
                    // The partner shares start on 10-10-26 and cannot be back-dated (PRD FR-17.2),
                    // so a بکری is always today's.
                    let draft = support::draft(vec![lot]);
                    bikri::save(&w.env.ctx, &w.owner, &BikriSaveInput {
                        draft,
                        confirm_cash_below_zero: true,
                    }).unwrap();
                }
                2 => {
                    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, &date, amount, "")).unwrap();
                }
                3 => {
                    // A return: the preview and the save agree about whether it is allowed.
                    let input = receipt(w.rahim, &date, amount);
                    let preview = vouchers::preview(&w.env.ctx, &VoucherPreviewInput {
                        entry: VoucherEntry::Receipt(input.clone()),
                        replacing_voucher_id: None,
                    }).unwrap();
                    let payable = |w: &World| vouchers::open_payables(&w.env.ctx, &OpenPayablesInput {
                        seller_id: w.rahim, person_key: None, include_settled: false,
                    }).unwrap().open_paisa;
                    let before = payable(&w);
                    match vouchers::save_receipt(&w.env.ctx, &w.owner, &input) {
                        Ok(_) => {
                            prop_assert!(preview.errors.is_empty(), "{:?}", preview.errors);
                            prop_assert!(payable(&w) <= before, "a return raised the payable");
                            // Each advance the preview named is open for what it said after the
                            // save (a back-dated return can also move what later credits took,
                            // which the preview shows in the same figure).
                            let info = mandi_services::assist::advances(&w.env.ctx, w.rahim).unwrap();
                            for effect in &preview.advance_effects {
                                let now = info
                                    .open
                                    .iter()
                                    .find(|o| {
                                        o.voucher_number == effect.advance.voucher_number
                                            && o.entry_date == effect.advance.entry_date
                                    })
                                    .map_or(Paisa::ZERO, |o| o.open_paisa);
                                prop_assert_eq!(effect.still_open_paisa, now);
                            }
                        }
                        Err(error) => {
                            prop_assert_eq!(error.code(), ErrorCode::ErrReturnExceedsOpen);
                            prop_assert_eq!(preview.errors.first().map(|e| e.code), Some(ErrorCode::ErrReturnExceedsOpen));
                            prop_assert_eq!(payable(&w), before);
                        }
                    }
                }
                _ => {
                    let live = live_vouchers(&w, &[VoucherType::Receipt, VoucherType::Payment, VoucherType::Advance]);
                    let sellers: Vec<(i64, VoucherType)> = live
                        .into_iter()
                        .filter(|(id, _)| {
                            vouchers::get(&w.env.ctx, *id)
                                .map(|r| r.account.map(|a| a.id) == Some(w.rahim))
                                .unwrap_or(false)
                        })
                        .collect();
                    if sellers.is_empty() {
                        continue;
                    }
                    let (id, kind) = sellers[usize::from(step.pick) % sellers.len()];
                    let row = vouchers::get(&w.env.ctx, id).unwrap();
                    if step.op % 2 == 0 {
                        // A cancel always works: nothing is raised by taking a voucher back,
                        // except a return, which the books accept as they are.
                        vouchers::cancel(&w.env.ctx, &w.owner, &VoucherCancelInput {
                            id, version: row.version, reason: "x".into(), confirm_cash_below_zero: true,
                        }).unwrap();
                    } else {
                        let entry = match kind {
                            VoucherType::Receipt => VoucherEntry::Receipt(receipt(w.rahim, &date, amount)),
                            VoucherType::Payment => VoucherEntry::Payment(payment(w.rahim, &date, amount, "")),
                            _ => VoucherEntry::Advance(advance(w.rahim, &date, amount)),
                        };
                        let preview = vouchers::preview(&w.env.ctx, &VoucherPreviewInput {
                            entry: entry.clone(),
                            replacing_voucher_id: Some(id),
                        }).unwrap();
                        let edited = vouchers::edit(&w.env.ctx, &w.owner, &VoucherEditInput {
                            id, version: row.version, entry, reason: String::new(), confirm_cash_below_zero: true,
                        });
                        match edited {
                            Ok(saved) => {
                                prop_assert!(preview.errors.is_empty(), "{:?}", preview.errors);
                                prop_assert_eq!(signed(preview.new_balance), signed(saved.new_balance));
                            }
                            // An edit may be refused (a return too big for the books without
                            // the old one); nothing is changed then.
                            Err(error) => {
                                prop_assert_eq!(error.code(), ErrorCode::ErrReturnExceedsOpen);
                                prop_assert_eq!(preview.errors.first().map(|e| e.code), Some(ErrorCode::ErrReturnExceedsOpen));
                            }
                        }
                    }
                }
            }
            check_seller(&w);
            matching_is_canonical(&w, w.akram, w.rahim);
        }
    }
}

#[test]
fn the_amounts_of_a_receipt_run_stay_whole_rupees() {
    // A small fixed run, so a failure of the random ones can be told from a broken world.
    let w = rich_world();
    vouchers::save_payment(
        &w.env.ctx,
        &w.owner,
        &payment(w.akram, "10-10-2026", 3_000, "x"),
    )
    .unwrap();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10-10-2026", 1_000)).unwrap();
    check_buyer(&w);
    let bills = vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: w.akram,
            person_key: None,
        },
    )
    .unwrap();
    assert_eq!(bills.open_paisa, rupees(2_000));
}
