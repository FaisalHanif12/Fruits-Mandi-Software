//! Property tests of fruit taken (Code Standards 9.3, PRD C16, C27, FR-16.10 to FR-16.12, plan 17d).
//! A lot is sold in part, then a random run follows: fruit taken from the lot or from another agent's
//! shop, for a partner or for the guests, edits of the quantity, and cancels. After every step:
//!
//! - the lot's `qty_taken` is the quantity of the live entries from the lot, and its balance (C16) is
//!   never below 0;
//! - the parts of every live entry add up to its value (C27), and no part is negative;
//! - the books balance;
//! - the partners' totals together are what they were, less the value of the live entries (fruit only
//!   moves money from the partners to the lot's seller or the agent);
//! - the lot's seller is credited the value of its live entries, and the agent the value of his.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_services::core::accounts::{AccountType, BalanceSide};
use mandi_services::core::bikri::BikriSaveInput;
use mandi_services::core::fruit::{
    FruitCancelInput, FruitEditInput, FruitGetInput, FruitInput, FruitSource,
};
use mandi_services::{bikri, fruit, ledger, partners};
use proptest::prelude::*;
use support::{World, world};

#[derive(Debug, Clone)]
enum Step {
    /// Fruit taken: `shop` from the lot, else from the agent's shop; `guests` for the shop's guests.
    Take {
        shop: bool,
        guests: bool,
        qty: i64,
        rate: i64,
    },
    /// An edit of an earlier entry to `qty` نگ.
    Edit { pick: usize, qty: i64 },
    /// A cancel of an earlier entry.
    Cancel { pick: usize },
}

#[derive(Debug, Clone)]
struct Case {
    lot_qty: i64,
    sold: i64,
    steps: Vec<Step>,
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        4 => (any::<bool>(), any::<bool>(), 1_i64..9, 1_i64..2_000)
            .prop_map(|(shop, guests, qty, rate)| Step::Take { shop, guests, qty, rate }),
        2 => (0_usize..8, 1_i64..9).prop_map(|(pick, qty)| Step::Edit { pick, qty }),
        1 => (0_usize..8).prop_map(|pick| Step::Cancel { pick }),
    ]
}

fn case() -> impl Strategy<Value = Case> {
    (10_i64..60, proptest::collection::vec(step(), 1..10)).prop_flat_map(|(lot_qty, steps)| {
        (1_i64..=lot_qty).prop_map(move |sold| Case {
            lot_qty,
            sold,
            steps: steps.clone(),
        })
    })
}

/// A live entry the run keeps track of.
struct Live {
    id: i64,
    version: i64,
    shop: bool,
    guests: bool,
    qty: i64,
    rate: i64,
}

fn input_of(w: &World, agent: i64, live: &Live, editing: Option<i64>) -> FruitInput {
    FruitInput {
        for_guests: live.guests,
        partner_id: (!live.guests).then_some(w.partners[0]),
        date: "10-10-2026".into(),
        source: if live.shop {
            FruitSource::Shop
        } else {
            FruitSource::OtherShop
        },
        lot_id: live.shop.then_some(1),
        agent_account_id: (!live.shop).then_some(agent),
        item_id: (!live.shop).then_some(w.apple),
        packing_id: (!live.shop).then_some(w.box_),
        qty: live.qty.to_string(),
        weight: None,
        rate: live.rate.to_string(),
        note: None,
        editing_id: editing,
    }
}

fn scalar(w: &World, sql: &str) -> i64 {
    common::raw(&w.dir)
        .query_row(sql, [], |r| r.get(0))
        .unwrap()
}

fn partner_totals(w: &World) -> i64 {
    partners::overview(&w.env.ctx)
        .unwrap()
        .iter()
        .map(|r| r.total_paisa.as_paisa())
        .sum()
}

/// Credits less debits of an account, in paisa.
fn credit_of(w: &World, account: i64) -> i64 {
    let b = ledger::account_balance(&w.env.ctx, account).unwrap();
    match b.side {
        BalanceSide::Cr => b.amount_paisa.as_paisa(),
        BalanceSide::Dr => -b.amount_paisa.as_paisa(),
        BalanceSide::Zero => 0,
    }
}

fn run(case: &Case) {
    let w = world();
    set_today(&w.env, 2026, 10, 10);
    let lot = support::with_sales(
        support::bare_lot(&w, "l-1", w.rahim, w.banana, &case.lot_qty.to_string()),
        vec![support::sale("s-1", w.akram, &case.sold.to_string(), "600")],
    );
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::draft(vec![lot]),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let agent = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "حاجی",
        "Haji",
        true,
        "",
    );
    let base_totals = partner_totals(&w);
    let base_rahim = credit_of(&w, w.rahim);
    let mut live: Vec<Live> = Vec::new();

    for step in &case.steps {
        let taken_from_lot =
            |live: &[Live]| -> i64 { live.iter().filter(|l| l.shop).map(|l| l.qty).sum() };
        match step {
            Step::Take {
                shop,
                guests,
                qty,
                rate,
            } => {
                let available = case.lot_qty - case.sold - taken_from_lot(&live);
                if *shop && *qty > available {
                    continue;
                }
                let entry = Live {
                    id: 0,
                    version: 0,
                    shop: *shop,
                    guests: *guests,
                    qty: *qty,
                    rate: *rate,
                };
                let saved =
                    fruit::save(&w.env.ctx, &w.owner, &input_of(&w, agent, &entry, None)).unwrap();
                live.push(Live {
                    id: saved.id,
                    version: saved.version,
                    ..entry
                });
            }
            Step::Edit { pick, qty } => {
                if live.is_empty() {
                    continue;
                }
                let i = pick % live.len();
                let others = taken_from_lot(&live) - if live[i].shop { live[i].qty } else { 0 };
                let available = case.lot_qty - case.sold - others;
                if live[i].shop && *qty > available {
                    continue;
                }
                let changed = Live {
                    qty: *qty,
                    ..Live {
                        id: live[i].id,
                        version: live[i].version,
                        shop: live[i].shop,
                        guests: live[i].guests,
                        qty: live[i].qty,
                        rate: live[i].rate,
                    }
                };
                let done = fruit::edit(
                    &w.env.ctx,
                    &w.owner,
                    &FruitEditInput {
                        id: live[i].id,
                        version: live[i].version,
                        entry: input_of(&w, agent, &changed, Some(live[i].id)),
                        reason: "edit".into(),
                    },
                )
                .unwrap();
                live[i].qty = *qty;
                live[i].version = done.version;
            }
            Step::Cancel { pick } => {
                if live.is_empty() {
                    continue;
                }
                let gone = live.remove(pick % live.len());
                fruit::cancel(
                    &w.env.ctx,
                    &w.owner,
                    &FruitCancelInput {
                        id: gone.id,
                        version: gone.version,
                        reason: "cancel".into(),
                    },
                )
                .unwrap();
            }
        }

        // C16: qty_taken is the live quantity of the lot and the balance never goes below 0.
        let from_lot = taken_from_lot(&live);
        assert_eq!(
            scalar(&w, "SELECT qty_taken FROM bikri_lots WHERE id = 1"),
            from_lot
        );
        assert!(case.lot_qty - case.sold - from_lot >= 0);
        // C27: the parts of every live entry add up to its value, and none is negative.
        let mut live_values = 0_i64;
        let (mut lot_values, mut agent_values) = (0_i64, 0_i64);
        for entry in &live {
            let row = fruit::get(&w.env.ctx, &FruitGetInput { id: entry.id }).unwrap();
            let value = entry.qty * entry.rate * 100;
            assert_eq!(row.value_paisa.as_paisa(), value);
            let parts: i64 = row.parts.iter().map(|p| p.amount_paisa.as_paisa()).sum();
            assert_eq!(parts, value, "the parts add up to the value");
            assert!(row.parts.iter().all(|p| p.amount_paisa.as_paisa() > 0));
            live_values += value;
            if entry.shop {
                lot_values += value;
            } else {
                agent_values += value;
            }
        }
        let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
        assert_eq!(trial.total_dr, trial.total_cr, "the books balance");
        assert_eq!(partner_totals(&w), base_totals - live_values);
        assert_eq!(credit_of(&w, w.rahim), base_rahim + lot_values);
        assert_eq!(credit_of(&w, agent), agent_values);
    }
}

proptest! {
    // Each case builds a whole database, so it is a smaller run than the pure maths tests (the
    // guest split itself has 10,000 cases in the domain crate).
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Plan 17d: for any run of fruit taken, edited and cancelled, the lot, the parts and the books
    /// add up after every step.
    #[test]
    fn fruit_taken_adds_up_after_every_step(case in case()) {
        run(&case);
    }
}
