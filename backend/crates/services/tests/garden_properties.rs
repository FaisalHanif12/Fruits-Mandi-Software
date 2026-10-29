//! Property tests of a garden (Code Standards 9.3, PRD C23, C26, FR-12.34 to FR-12.38, plan 17c).
//! A random garden is made (one to three partners, any shares, any money sources), and a random run
//! follows: installments and costs, sales in another shop, sales in our shop (a lot of بکری whose
//! seller is the garden), and cancels of any of them. After every step:
//!
//! - the parts of every live payment add up to the payment (C23);
//! - what left the cash is the shop parts less the money that came in from other shops;
//! - what the garden shows as paid, as money from other shops and as rent still due is what the run
//!   put in, and the running balance is the money in less everything paid (FR-12.37);
//! - while the garden is open its account holds exactly its money in, and the books balance.
//!
//! Then the garden is closed (and sometimes undone and closed again). After the closing:
//!
//! - the garden's account is 0 (C26);
//! - the money credited to the partners is the balance the account held (a loss is a debit);
//! - Σ over the partners of (money back - parts paid from the shop - parts paid from pockets) is the
//!   result, which is the money in less everything paid;
//! - the net on the partners' 0604 accounts is the money back less the parts paid from the shop.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::bikri::{BikriSaveInput, CancelInput};
use mandi_services::core::gardens::{
    GardenCloseInput, GardenDetail, GardenEntryCancelInput, GardenEntryInput, GardenGetInput,
    GardenInput, GardenOtherSaleInput, GardenPaymentInput, GardenReopenInput,
};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput, VentureStatus};
use mandi_services::{bikri, cash, gardens, ledger, partners};
use proptest::prelude::*;
use support::{World, world};

#[derive(Debug, Clone)]
enum Step {
    /// An installment of `amount` rupees.
    Installment { amount: i64 },
    /// A running cost of `amount` rupees.
    Cost { amount: i64 },
    /// A sale in another shop: `qty` نگ for `amount` rupees.
    OtherSale { qty: i64, amount: i64 },
    /// A lot of بکری whose seller is the garden sells `qty` to اکرم at `rate`. With `labour` above 0
    /// a labour of that many rupees is typed on the lot (a loss when it is more than the credit).
    OwnSale { qty: i64, rate: i64, labour: i64 },
    /// A cancel of an earlier payment or sale in another shop.
    CancelEntry { pick: usize },
    /// A cancel of an earlier sale in our shop.
    CancelSale { pick: usize },
}

#[derive(Debug, Clone)]
struct Case {
    /// Weights of the partners in the garden (one to three).
    weights: Vec<u32>,
    /// True when a partner pays from his own pocket.
    pockets: Vec<bool>,
    rent: i64,
    steps: Vec<Step>,
    /// Close the garden at the end.
    close: bool,
    /// After closing, undo the closing and close again.
    reopen: bool,
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => (1_i64..200_000).prop_map(|amount| Step::Installment { amount }),
        2 => (1_i64..50_000).prop_map(|amount| Step::Cost { amount }),
        2 => (1_i64..50, 1_i64..300_000).prop_map(|(qty, amount)| Step::OtherSale { qty, amount }),
        3 => (1_i64..50, 1_i64..3_000, prop_oneof![3 => Just(0_i64), 1 => 1_i64..400_000])
            .prop_map(|(qty, rate, labour)| Step::OwnSale { qty, rate, labour }),
        1 => (0_usize..8).prop_map(|pick| Step::CancelEntry { pick }),
        1 => (0_usize..8).prop_map(|pick| Step::CancelSale { pick }),
    ]
}

fn case() -> impl Strategy<Value = Case> {
    (
        proptest::collection::vec(1_u32..1_000, 1..4),
        proptest::collection::vec(any::<bool>(), 3),
        1_i64..500_000,
        proptest::collection::vec(step(), 1..9),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(weights, pockets, rent, steps, close, reopen)| Case {
            weights,
            pockets,
            rent,
            steps,
            close,
            reopen,
        })
}

/// Shares in bps that total exactly 10,000, from weights.
fn shares_of(weights: &[u32]) -> Vec<i32> {
    let total: u32 = weights.iter().sum();
    let mut out: Vec<i32> = weights
        .iter()
        .map(|w| i32::try_from(u64::from(*w) * 10_000 / u64::from(total)).unwrap())
        .collect();
    let used: i32 = out.iter().sum();
    out[0] += 10_000 - used;
    out
}

fn percent(bps: i32) -> String {
    format!("{}.{:02}", bps / 100, bps % 100)
}

fn scalar(w: &World, sql: &str) -> i64 {
    common::raw(&w.dir)
        .query_row(sql, [], |r| r.get(0))
        .unwrap()
}

fn cash_paisa(w: &World) -> i64 {
    w.env.ctx.db().read(cash::cash_in_hand).unwrap().as_paisa()
}

/// Credits less debits of an account, in paisa (jama is plus).
fn credit_balance(w: &World, account: i64) -> i64 {
    let b = ledger::account_balance(&w.env.ctx, account).unwrap();
    match b.side {
        BalanceSide::Cr => b.amount_paisa.as_paisa(),
        BalanceSide::Dr => -b.amount_paisa.as_paisa(),
        BalanceSide::Zero => 0,
    }
}

/// Credits less debits on the partners' 0604 accounts, in paisa, reversals included.
fn net_on_venture_accounts(w: &World) -> i64 {
    let ids: Vec<String> = w
        .partners
        .iter()
        .map(|p| {
            partners::get(&w.env.ctx, *p)
                .unwrap()
                .row
                .venture_account_id
                .to_string()
        })
        .collect();
    scalar(
        w,
        &format!(
            "SELECT COALESCE(SUM(cr_paisa - dr_paisa), 0) FROM journal_lines WHERE account_id IN ({})",
            ids.join(",")
        ),
    )
}

fn detail(w: &World, id: i64) -> GardenDetail {
    gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap()
}

fn payment(garden: i64, amount: i64, reason: &str) -> GardenPaymentInput {
    GardenPaymentInput {
        garden_id: Some(garden),
        date: "10-10-2026".into(),
        amount: amount.to_string(),
        reason: reason.into(),
        parts: Vec::new(),
        confirm_cash_below_zero: true,
    }
}

/// What the run has put in so far, worked out here and not by the garden code.
#[derive(Default)]
struct Run {
    /// Live payments: (entry id, entry version, is an installment, amount in paisa).
    payments: Vec<(i64, i64, bool, i64)>,
    /// Live sales in another shop: (entry id, entry version, amount in paisa).
    others: Vec<(i64, i64, i64)>,
    /// Live sales in our shop: the بکری ids.
    sales: Vec<i64>,
}

impl Run {
    fn paid(&self) -> i64 {
        self.payments.iter().map(|p| p.3).sum()
    }

    fn installments(&self) -> i64 {
        self.payments.iter().filter(|p| p.2).map(|p| p.3).sum()
    }

    fn other(&self) -> i64 {
        self.others.iter().map(|o| o.2).sum()
    }
}

/// The checks that hold after every step while the garden is open.
fn check_open(w: &World, garden: i64, cash0: i64, run: &Run) {
    let d = detail(w, garden);
    let row = &d.row;
    assert_eq!(row.status, VentureStatus::Open);
    // C23: every live payment's parts add up to the payment.
    assert_eq!(
        scalar(
            w,
            "SELECT COUNT(*) FROM venture_entries e
             WHERE e.kind IN ('installment', 'cost') AND e.status = 'saved'
               AND e.amount_paisa <> (SELECT COALESCE(SUM(p.amount_paisa), 0)
                                      FROM venture_entry_parts p WHERE p.entry_id = e.id)"
        ),
        0,
        "the parts of a payment add up to the payment"
    );
    let (paid, other) = (run.paid(), run.other());
    let shop_parts = scalar(
        w,
        "SELECT COALESCE(SUM(p.amount_paisa), 0)
         FROM venture_entry_parts p JOIN venture_entries e ON e.id = p.entry_id
         WHERE p.source = 'shop' AND e.status = 'saved' AND e.kind IN ('installment', 'cost')",
    );
    let pocket_parts = scalar(
        w,
        "SELECT COALESCE(SUM(p.amount_paisa), 0)
         FROM venture_entry_parts p JOIN venture_entries e ON e.id = p.entry_id
         WHERE p.source = 'pocket' AND e.status = 'saved' AND e.kind IN ('installment', 'cost')",
    );
    assert_eq!(shop_parts + pocket_parts, paid);
    assert_eq!(row.paid_paisa.as_paisa(), paid, "paid is what the run paid");
    assert_eq!(row.paid_shop_paisa.as_paisa(), shop_parts);
    assert_eq!(row.paid_pocket_paisa.as_paisa(), pocket_parts);
    assert_eq!(row.other_shop_paisa.as_paisa(), other);
    assert_eq!(
        row.rent_due_paisa.as_paisa(),
        row.rent_paisa.as_paisa() - run.installments(),
        "rent still due is the rent less the live installments"
    );
    // The cash: shop parts left, the money of other shops came in; a garden lot gives no cash.
    assert_eq!(cash0 - cash_paisa(w), shop_parts - other);
    // FR-12.37: the running balance is money in less everything paid, and the account holds
    // exactly the money in.
    assert_eq!(
        row.balance_paisa.as_paisa(),
        row.money_in_paisa.as_paisa() - paid
    );
    assert_eq!(
        row.account_balance_paisa.as_paisa(),
        row.money_in_paisa.as_paisa()
    );
    assert_eq!(
        credit_balance(w, row.account.id),
        row.money_in_paisa.as_paisa()
    );
    // Before the closing nothing was credited to a partner for the garden.
    assert_eq!(net_on_venture_accounts(w), -shop_parts);
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr, "the books balance");
}

/// The checks after the closing.
fn check_closed(w: &World, garden: i64, before: &GardenDetail) {
    let d = detail(w, garden);
    let row = &d.row;
    assert_eq!(row.status, VentureStatus::Closed);
    assert_eq!(
        credit_balance(w, row.account.id),
        0,
        "C26: the account is 0"
    );
    assert_eq!(row.account_balance_paisa.as_paisa(), 0);
    let balance = before.row.money_in_paisa.as_paisa();
    let back: i64 = d
        .partner_results
        .iter()
        .map(|p| p.money_back_paisa.as_paisa())
        .sum();
    assert_eq!(
        back, balance,
        "what was credited to the partners is the balance"
    );
    let net: i64 = d
        .partner_results
        .iter()
        .map(|p| p.net_paisa.as_paisa())
        .sum();
    let paid = row.paid_paisa.as_paisa();
    assert_eq!(net, balance - paid, "the nets add up to the result");
    assert_eq!(net, row.balance_paisa.as_paisa());
    for p in &d.partner_results {
        assert_eq!(
            p.net_paisa.as_paisa(),
            p.money_back_paisa.as_paisa()
                - p.paid_shop_paisa.as_paisa()
                - p.paid_pocket_paisa.as_paisa()
        );
        // A part has the sign of the balance and is never more than it.
        assert!(p.money_back_paisa.as_paisa() * balance.signum() >= 0);
        assert!(p.money_back_paisa.as_paisa().abs() <= balance.abs());
    }
    assert_eq!(
        net_on_venture_accounts(w),
        back - row.paid_shop_paisa.as_paisa(),
        "the 0604 net is the money back less the shop parts"
    );
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr, "the books balance");
}

fn run(case: &Case) {
    let w = world();
    let shares = shares_of(&case.weights);
    let listed: Vec<VenturePartnerInput> = w
        .partners
        .iter()
        .enumerate()
        .map(|(i, id)| VenturePartnerInput {
            partner_id: *id,
            share: shares.get(i).map_or("0".to_owned(), |bps| percent(*bps)),
            source: if case.pockets[i] {
                MoneySource::Pocket
            } else {
                MoneySource::Shop
            },
        })
        .collect();
    let cash0 = cash_paisa(&w);
    let garden = gardens::create(
        &w.env.ctx,
        &w.owner,
        &GardenInput {
            name_ur: "شاہ پور باغ".into(),
            name_en: "Shahpur Garden".into(),
            confirmed_rules_words: vec![
                "شاہ".into(),
                "پور".into(),
                "باغ".into(),
                "Shahpur".into(),
                "Garden".into(),
            ],
            date: "10-10-2026".into(),
            item_id: Some(w.banana),
            rent: case.rent.to_string(),
            season_end: None,
            note: None,
            partners: listed,
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
    .row;
    set_today(&w.env, 2026, 10, 10);

    let mut run = Run::default();
    for step in &case.steps {
        match step {
            Step::Installment { amount } | Step::Cost { amount } => {
                let is_installment = matches!(step, Step::Installment { .. });
                let input = payment(garden.id, *amount, "payment");
                let saved = gardens::entry_save(
                    &w.env.ctx,
                    &w.owner,
                    &if is_installment {
                        GardenEntryInput::Installment(input)
                    } else {
                        GardenEntryInput::Cost(input)
                    },
                )
                .unwrap();
                run.payments
                    .push((saved.id, saved.version, is_installment, amount * 100));
            }
            Step::OtherSale { qty, amount } => {
                let saved = gardens::entry_save(
                    &w.env.ctx,
                    &w.owner,
                    &GardenEntryInput::OtherSale(GardenOtherSaleInput {
                        arrival_id: None,
                        garden_id: Some(garden.id),
                        date: "10-10-2026".into(),
                        shop: Some("Haji Traders".into()),
                        qty: qty.to_string(),
                        amount: amount.to_string(),
                        note: None,
                    }),
                )
                .unwrap();
                run.others.push((saved.id, saved.version, amount * 100));
            }
            Step::OwnSale { qty, rate, labour } => {
                let mut lot = support::with_sales(
                    support::bare_lot(
                        &w,
                        "garden-1",
                        garden.account.id,
                        w.banana,
                        &qty.to_string(),
                    ),
                    vec![support::sale(
                        "s-1",
                        w.akram,
                        &qty.to_string(),
                        &rate.to_string(),
                    )],
                );
                lot.adjust_advance = false;
                if *labour > 0 {
                    lot.labour = support::manual(&labour.to_string());
                }
                let done = bikri::save(
                    &w.env.ctx,
                    &w.owner,
                    &BikriSaveInput {
                        draft: support::draft(vec![lot]),
                        confirm_cash_below_zero: true,
                    },
                )
                .unwrap();
                run.sales.push(done.bikri_id);
            }
            Step::CancelEntry { pick } => {
                let total = run.payments.len() + run.others.len();
                if total == 0 {
                    continue;
                }
                let i = pick % total;
                let (id, version) = if i < run.payments.len() {
                    let gone = run.payments.remove(i);
                    (gone.0, gone.1)
                } else {
                    let gone = run.others.remove(i - run.payments.len());
                    (gone.0, gone.1)
                };
                gardens::entry_cancel(
                    &w.env.ctx,
                    &w.owner,
                    &GardenEntryCancelInput {
                        id,
                        version,
                        reason: "cancel".into(),
                        confirm_cash_below_zero: true,
                    },
                )
                .unwrap();
            }
            Step::CancelSale { pick } => {
                if run.sales.is_empty() {
                    continue;
                }
                let id = run.sales.remove(pick % run.sales.len());
                let sale = bikri::get(&w.env.ctx, id).unwrap();
                bikri::cancel(
                    &w.env.ctx,
                    &w.owner,
                    &CancelInput {
                        id,
                        version: sale.version,
                        reason: "cancel".into(),
                        confirm_cash_below_zero: true,
                    },
                )
                .unwrap();
            }
        }
        check_open(&w, garden.id, cash0, &run);
    }

    if case.close {
        let before = detail(&w, garden.id);
        let closed = gardens::close(
            &w.env.ctx,
            &w.owner,
            &GardenCloseInput {
                id: garden.id,
                version: before.row.version,
                date: "10-10-2026".into(),
            },
        )
        .unwrap();
        assert_eq!(closed.row.status, VentureStatus::Closed);
        check_closed(&w, garden.id, &before);
        if case.reopen {
            let undone = gardens::reopen(
                &w.env.ctx,
                &w.owner,
                &GardenReopenInput {
                    id: garden.id,
                    version: closed.row.version,
                    reason: "undo".into(),
                },
            )
            .unwrap();
            assert_eq!(undone.row.status, VentureStatus::Open);
            check_open(&w, garden.id, cash0, &run);
            let again = detail(&w, garden.id);
            assert_eq!(
                again.row.money_in_paisa, before.row.money_in_paisa,
                "the undo gives the account its money back"
            );
            let closed_again = gardens::close(
                &w.env.ctx,
                &w.owner,
                &GardenCloseInput {
                    id: garden.id,
                    version: again.row.version,
                    date: "10-10-2026".into(),
                },
            )
            .unwrap();
            assert_eq!(closed_again.row.status, VentureStatus::Closed);
            check_closed(&w, garden.id, &before);
        }
    }
}

proptest! {
    // Each case builds a whole database, so it is a smaller run than the pure maths tests (the
    // closing split has its own 10,000-case property test in the domain crate).
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Plan 17c: for any shares, any sources and any run of payments, sales and cancels, a garden
    /// adds up after every step and closes to 0.
    #[test]
    fn a_garden_adds_up_after_every_step_and_closes_to_zero(case in case()) {
        run(&case);
    }
}
