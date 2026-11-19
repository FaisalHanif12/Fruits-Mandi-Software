//! Property tests of a market trip (Code Standards 9.3, PRD C21, C23 to C25, FR-12.27, plan 16b
//! Section 7 item 2). A random trip is made, one purchase is bought, and a random run follows: sales
//! in our shop (an own lot of بکری, also two lots in one بکری and a loss with a typed labour), edits
//! of a sale's quantity, cancels of a sale, sales in another shop and their cancels, write-offs and
//! their undo, and a trip cost spread after sales. After every step:
//!
//! - the parts of the payment add up to the payment, and the shop parts are what left the cash;
//! - the cost released never goes over the total cost, and is the total when nothing is left;
//! - the net credit of the partners' 0604 accounts (credits less debits, reversals included) is the
//!   sales money of the purchase less the parts paid from the shop's cash;
//! - the stock moves of the purchase add up to the stock left, and the partner parts stored with
//!   the lots add up to the 0604 lines of the بکری entries;
//! - the trip is closed exactly when nothing is left, and the books balance.
//!
//! When the trip has closed, the sum over its partners of (money credited - parts paid from the
//! shop - parts paid from pockets) is its result.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_services::core::bikri::{
    BikriSaveInput, CancelInput, EditOpenInput, EditSaveInput, LotInput,
};
use mandi_services::core::purchases::{
    PurchaseGetInput, PurchaseInput, PurchaseWriteOffInput, PurchaseWriteOffUndoInput,
};
use mandi_services::core::trips::{
    OtherSaleInput, TripCostInput, TripEntryCancelInput, TripGetInput, TripInput,
};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput, VentureStatus};
use mandi_services::{bikri, cash, ledger, partners, purchases, trips};
use proptest::prelude::*;
use support::{World, world};

#[derive(Debug, Clone)]
enum Step {
    /// An own lot sells `qty` to اکرم at `rate` on credit. With `labour` above 0 a labour of that
    /// many rupees is typed on the lot (a loss when it is more than the credit). With `twin`, a
    /// second own lot of the same بکری sells one more نگ to ارشد.
    OwnSale {
        qty: i64,
        rate: i64,
        labour: i64,
        twin: bool,
    },
    /// A sale in another shop: `qty` نگ for `amount` rupees.
    OtherSale { qty: i64, amount: i64 },
    /// The goods spoil: `qty` نگ are written off.
    WriteOff { qty: i64 },
    /// An edit of the first lot of an earlier sale to `qty` نگ.
    EditQty { pick: usize, qty: i64 },
    /// A cancel of an earlier sale.
    CancelSale { pick: usize },
    /// The undo of an earlier write-off.
    UndoWriteOff { pick: usize },
    /// A cancel of an earlier sale in another shop.
    CancelOther { pick: usize },
    /// A trip cost of `amount` rupees, spread over the purchase if it has goods left.
    LateCost { amount: i64 },
}

#[derive(Debug, Clone)]
struct Case {
    /// Weights of the partners in the trip (one to three).
    weights: Vec<u32>,
    /// True when a partner pays from his own pocket.
    pockets: Vec<bool>,
    qty: i64,
    rate: i64,
    freight: i64,
    /// A trip cost, or 0 for none.
    trip_cost: i64,
    steps: Vec<Step>,
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => (1_i64..30, 1_i64..3_000, prop_oneof![3 => Just(0_i64), 1 => 1_i64..200_000], any::<bool>())
            .prop_map(|(qty, rate, labour, twin)| Step::OwnSale { qty, rate, labour, twin }),
        2 => (1_i64..30, 1_i64..60_000).prop_map(|(qty, amount)| Step::OtherSale { qty, amount }),
        2 => (1_i64..30).prop_map(|qty| Step::WriteOff { qty }),
        2 => (0_usize..8, 1_i64..40).prop_map(|(pick, qty)| Step::EditQty { pick, qty }),
        1 => (0_usize..8).prop_map(|pick| Step::CancelSale { pick }),
        1 => (0_usize..8).prop_map(|pick| Step::UndoWriteOff { pick }),
        1 => (0_usize..8).prop_map(|pick| Step::CancelOther { pick }),
        1 => (1_i64..3_000).prop_map(|amount| Step::LateCost { amount }),
    ]
}

fn case() -> impl Strategy<Value = Case> {
    (
        proptest::collection::vec(1_u32..1_000, 1..4),
        proptest::collection::vec(any::<bool>(), 3),
        2_i64..40,
        1_i64..2_000,
        0_i64..500,
        0_i64..5_000,
        proptest::collection::vec(step(), 1..8),
    )
        .prop_map(
            |(weights, pockets, qty, rate, freight, trip_cost, steps)| Case {
                weights,
                pockets,
                qty,
                rate,
                freight,
                trip_cost,
                steps,
            },
        )
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

fn orange(w: &World) -> i64 {
    support::add_item(&w.env, &w.owner, "مالٹا", "Orange", "fruit", "unit")
}

fn own_lot(w: &World, purchase_id: i64, qty: i64) -> LotInput {
    LotInput {
        seller_account_id: Some(support::id_of(&w.env, "030119998")),
        purchase_id: Some(purchase_id),
        item_id: None,
        packing_id: None,
        adjust_advance: false,
        ..support::bare_lot(w, "own-1", w.rahim, w.banana, &qty.to_string())
    }
}

/// Σ of a column of the journal lines of the partners' 0604 accounts, in paisa.
fn sum_on_venture_accounts(w: &World, column: &str) -> i64 {
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
    common::raw(&w.dir)
        .query_row(
            &format!(
                "SELECT COALESCE(SUM({column}), 0) FROM journal_lines WHERE account_id IN ({})",
                ids.join(",")
            ),
            [],
            |r| r.get(0),
        )
        .unwrap()
}

/// Credits less debits on the partners' 0604 accounts, in paisa, reversals included.
fn net_on_venture_accounts(w: &World) -> i64 {
    sum_on_venture_accounts(w, "cr_paisa") - sum_on_venture_accounts(w, "dr_paisa")
}

/// Credits less debits of the 0604 lines of the بکری entries (and their reversals), in paisa.
fn net_of_bikri_entries_on_venture_accounts(w: &World) -> i64 {
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
    common::raw(&w.dir)
        .query_row(
            &format!(
                "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0)
                 FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                 WHERE e.source_type = 'bikri' AND l.account_id IN ({})",
                ids.join(",")
            ),
            [],
            |r| r.get(0),
        )
        .unwrap()
}

fn scalar(w: &World, sql: &str) -> i64 {
    common::raw(&w.dir)
        .query_row(sql, [], |r| r.get(0))
        .unwrap()
}

/// An own sale made earlier: the بکری, its posting now, and the quantity of its first lot.
struct Live {
    bikri_id: i64,
    entry_id: i64,
    qty0: i64,
}

fn cash_paisa(w: &World) -> i64 {
    w.env.ctx.db().read(cash::cash_in_hand).unwrap().as_paisa()
}

fn run(case: &Case) {
    let w = world();
    let item = orange(&w);
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
    let trip = trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: None,
            sent_person: None,
            note: None,
            partners: listed,
        },
    )
    .unwrap()
    .id;

    // The purchase (C23): the parts add up to the payment and the shop parts leave the cash.
    let cash_before = cash_paisa(&w);
    let saved = purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(trip),
            date: "10-10-2026".into(),
            supplier: None,
            item_id: Some(item),
            packing_id: Some(w.box_),
            qty: case.qty.to_string(),
            weight: None,
            rate: case.rate.to_string(),
            freight: (case.freight > 0).then(|| case.freight.to_string()),
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let numbers = saved.preview.numbers.clone().unwrap();
    let parts_sum: i64 = numbers
        .parts
        .parts
        .iter()
        .map(|p| p.amount_paisa.as_paisa())
        .sum();
    assert_eq!(parts_sum, numbers.paid_now.as_paisa());
    assert_eq!(
        numbers.parts.shop_total_paisa.as_paisa() + numbers.parts.pocket_total_paisa.as_paisa(),
        numbers.paid_now.as_paisa()
    );
    assert_eq!(
        cash_before - cash_paisa(&w),
        numbers.parts.shop_total_paisa.as_paisa(),
        "the shop parts are what left the cash"
    );
    let mut shop_parts = numbers.parts.shop_total_paisa.as_paisa();
    let mut paid_all = numbers.paid_now.as_paisa();

    if case.trip_cost > 0 {
        let cost = trips::cost_save(
            &w.env.ctx,
            &w.owner,
            &TripCostInput {
                trip_id: Some(trip),
                date: "10-10-2026".into(),
                amount: case.trip_cost.to_string(),
                reason: "truck".into(),
                parts: Vec::new(),
                confirm_cash_below_zero: true,
            },
        )
        .unwrap();
        let numbers = cost.preview.numbers.clone().unwrap();
        let sum: i64 = numbers
            .parts
            .parts
            .iter()
            .map(|p| p.amount_paisa.as_paisa())
            .sum();
        assert_eq!(sum, case.trip_cost * 100);
        shop_parts += numbers.parts.shop_total_paisa.as_paisa();
        paid_all += case.trip_cost * 100;
    }

    set_today(&w.env, 2026, 10, 10);
    let mut live: Vec<Live> = Vec::new();
    let mut offs: Vec<i64> = Vec::new();
    let mut others: Vec<(i64, i64)> = Vec::new();
    for step in &case.steps {
        let row = purchases::get(&w.env.ctx, &PurchaseGetInput { id: saved.id }).unwrap();
        let left = row.qty_left;
        match step {
            Step::OwnSale {
                qty,
                rate,
                labour,
                twin,
            } => {
                if left == 0 {
                    continue;
                }
                let qty = (*qty).min(left);
                let mut first = own_lot(&w, saved.id, qty);
                if *labour > 0 {
                    first.labour = support::manual(&labour.to_string());
                }
                let mut lots = vec![support::with_sales(
                    first,
                    vec![support::sale(
                        "s-1",
                        w.akram,
                        &qty.to_string(),
                        &rate.to_string(),
                    )],
                )];
                if *twin && left > qty {
                    let mut second = own_lot(&w, saved.id, 1);
                    second.key = "own-2".into();
                    lots.push(support::with_sales(
                        second,
                        vec![support::sale("s-2", w.arshad, "1", &rate.to_string())],
                    ));
                }
                let done = bikri::save(
                    &w.env.ctx,
                    &w.owner,
                    &BikriSaveInput {
                        draft: support::draft(lots),
                        confirm_cash_below_zero: true,
                    },
                )
                .unwrap();
                live.push(Live {
                    bikri_id: done.bikri_id,
                    entry_id: done.entry_id,
                    qty0: qty,
                });
            }
            Step::OtherSale { qty, amount } => {
                if left == 0 {
                    continue;
                }
                let done = trips::other_sale_save(
                    &w.env.ctx,
                    &w.owner,
                    &OtherSaleInput {
                        trip_id: Some(trip),
                        date: "10-10-2026".into(),
                        shop: Some("Haji Shop".into()),
                        purchase_id: Some(saved.id),
                        qty: (*qty).min(left).to_string(),
                        amount: amount.to_string(),
                        note: None,
                    },
                )
                .unwrap();
                others.push((done.id, done.version));
            }
            Step::WriteOff { qty } => {
                if left == 0 {
                    continue;
                }
                let done = purchases::write_off(
                    &w.env.ctx,
                    &w.owner,
                    &PurchaseWriteOffInput {
                        purchase_id: saved.id,
                        qty: (*qty).min(left).to_string(),
                        reason: "spoiled".into(),
                    },
                )
                .unwrap();
                offs.push(done.move_id);
            }
            Step::EditQty { pick, qty } => {
                if live.is_empty() {
                    continue;
                }
                let i = pick % live.len();
                let available = left + live[i].qty0;
                let new_qty = (*qty).clamp(1, available);
                let opened = bikri::open_for_edit(
                    &w.env.ctx,
                    &EditOpenInput {
                        bikri_id: live[i].bikri_id,
                        entry_id: live[i].entry_id,
                    },
                )
                .unwrap();
                let mut draft = opened.draft.clone();
                draft.lots[0].qty_received = new_qty.to_string();
                draft.lots[0].sales[0].qty = new_qty.to_string();
                let edited = bikri::edit_save(
                    &w.env.ctx,
                    &w.owner,
                    &EditSaveInput {
                        entry_id: live[i].entry_id,
                        version: opened.version,
                        draft,
                        reason: Some("edit".into()),
                        confirm_cash_below_zero: true,
                    },
                )
                .unwrap();
                live[i].entry_id = edited.entry_id;
                live[i].qty0 = new_qty;
            }
            Step::CancelSale { pick } => {
                if live.is_empty() {
                    continue;
                }
                let gone = live.remove(pick % live.len());
                let detail = bikri::get(&w.env.ctx, gone.bikri_id).unwrap();
                bikri::cancel(
                    &w.env.ctx,
                    &w.owner,
                    &CancelInput {
                        id: gone.bikri_id,
                        version: detail.version,
                        reason: "cancel".into(),
                        confirm_cash_below_zero: true,
                    },
                )
                .unwrap();
            }
            Step::UndoWriteOff { pick } => {
                if offs.is_empty() {
                    continue;
                }
                let move_id = offs.remove(pick % offs.len());
                purchases::write_off_undo(
                    &w.env.ctx,
                    &w.owner,
                    &PurchaseWriteOffUndoInput {
                        move_id,
                        reason: "undo".into(),
                    },
                )
                .unwrap();
            }
            Step::CancelOther { pick } => {
                if others.is_empty() {
                    continue;
                }
                let (id, version) = others.remove(pick % others.len());
                trips::other_sale_cancel(
                    &w.env.ctx,
                    &w.owner,
                    &TripEntryCancelInput {
                        id,
                        version,
                        reason: "cancel".into(),
                        confirm_cash_below_zero: true,
                    },
                )
                .unwrap();
            }
            Step::LateCost { amount } => {
                if left == 0 {
                    continue;
                }
                let cost = trips::cost_save(
                    &w.env.ctx,
                    &w.owner,
                    &TripCostInput {
                        trip_id: Some(trip),
                        date: "10-10-2026".into(),
                        amount: amount.to_string(),
                        reason: "late truck".into(),
                        parts: Vec::new(),
                        confirm_cash_below_zero: true,
                    },
                )
                .unwrap();
                let numbers = cost.preview.numbers.clone().unwrap();
                shop_parts += numbers.parts.shop_total_paisa.as_paisa();
                paid_all += amount * 100;
            }
        }

        // C21: the cost released never goes over the total, and is the total when nothing is left.
        let row = purchases::get(&w.env.ctx, &PurchaseGetInput { id: saved.id }).unwrap();
        let total = row.total_cost_paisa.as_paisa();
        let released = row.cost_released_paisa.as_paisa();
        assert!(released <= total, "released {released} of {total}");
        assert!(released >= 0);
        assert_eq!(released + row.cost_left_paisa.as_paisa(), total);
        if row.qty_left == 0 {
            assert_eq!(released, total);
            assert_eq!(row.cost_left_paisa.as_paisa(), 0);
        }
        assert_eq!(total, paid_all, "the total cost is what was paid in");
        // C25: what the partners were credited, less what they were debited, is the sales money
        // less the parts paid from the shop's cash (reversals of cancels and edits included).
        assert_eq!(
            net_on_venture_accounts(&w),
            row.sales_paisa.as_paisa() + row.other_shop_paisa.as_paisa() - shop_parts
        );
        // The parts stored with the lots are the 0604 lines of the بکری entries.
        assert_eq!(
            scalar(
                &w,
                "SELECT COALESCE(SUM(amount_paisa), 0) FROM lot_credit_parts"
            ),
            net_of_bikri_entries_on_venture_accounts(&w)
        );
        // The stock moves of the purchase add up to the stock left.
        assert_eq!(
            scalar(
                &w,
                &format!(
                    "SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE purchase_id = {}",
                    saved.id
                )
            ),
            row.qty_left
        );
        // A trip with a purchase is closed exactly when nothing is left (FR-12.33).
        let status = trips::get(
            &w.env.ctx,
            &TripGetInput {
                id: trip,
                entries_offset: 0,
                entries_limit: 0,
            },
        )
        .unwrap()
        .row
        .status;
        assert_eq!(status == VentureStatus::Closed, row.qty_left == 0);
        let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
        assert_eq!(trial.total_dr, trial.total_cr);
    }

    // Once closed: Σ over the partners of (credited - shop parts - pocket parts) is the result.
    let detail = trips::get(
        &w.env.ctx,
        &TripGetInput {
            id: trip,
            entries_offset: 0,
            entries_limit: 0,
        },
    )
    .unwrap();
    let net: i64 = detail
        .partner_results
        .iter()
        .map(|p| p.net_paisa.as_paisa())
        .sum();
    let back: i64 = detail
        .partner_results
        .iter()
        .map(|p| p.money_back_paisa.as_paisa())
        .sum();
    let money_in: i64 = detail
        .partner_results
        .iter()
        .map(|p| p.money_in_shop_paisa.as_paisa() + p.money_in_pocket_paisa.as_paisa())
        .sum();
    assert_eq!(money_in, paid_all, "the money in is everything paid");
    assert_eq!(net, back - money_in);
    if detail.row.status == VentureStatus::Closed {
        assert_eq!(net, detail.row.result_paisa.as_paisa());
        // Money credited = the 0604 net + the parts paid from the shop.
        assert_eq!(
            net_on_venture_accounts(&w) + shop_parts - money_in,
            detail.row.result_paisa.as_paisa()
        );
    }
}

proptest! {
    // Each case builds a whole database, so it is a smaller run than the pure maths tests (those run
    // 10,000 cases; C21 itself has 10,000 in the domain crate). Cases are longer now, so 64 of them.
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Plan 16b Section 7 item 2, for any shares, any sources and any run of sales, edits, cancels, write-offs and costs.
    #[test]
    fn a_trip_adds_up_after_every_step(case in case()) {
        run(&case);
    }
}
