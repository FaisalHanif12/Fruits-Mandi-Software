//! Plan 80, lane T1: property tests of the whole books, on random sequences of EVERY posting type
//! through the real service functions, on a temporary database.
//!
//! The existing property tests each drive one module (vouchers, a garden, a trip, fruit, khatas and
//! loans, the partners). This one mixes them all in one world, with advances to sellers, new
//! accounts with openings, shop and partner links, a share change in the middle, the dead amount,
//! edits, cancels, undo, carried lots, rent relief and refund, and checks the invariants of the
//! whole books after every step (`checks.rs`):
//!
//! * every journal entry balances, the trial balance is 0, nothing is left as a draft;
//! * every split adds back exactly (partner share rows, C23 to C28);
//! * Σ partner کل رقم + ابتدائی سرمایہ + open advances to sellers = equity (FR-17.9, FR-17.13);
//! * Σ partner share rows = income less head-05 expenses (FR-6.5), entry by entry;
//! * an entry plus its reversal is zero on every account;
//! * the stored matching rows are the canonical ones;
//! * a trip's cost released never goes over its total and equals it when nothing is left (C21);
//! * a closed garden's account is 0 (C26);
//! * a refused step leaves no row behind (NFR-REL-1).
//!
//! The run is deterministic (a fixed seed), so it is the same on every machine. For new ground set
//! `T1_RANDOM=1`; `PROPTEST_CASES=n` sets the number of runs (default 30; each is about 16 steps).
//! A failure is shrunk and printed with the steps that cause it.
//!
//! Run it with `cargo test -p mandi-services --test robustness_gate_t1 -- --nocapture`.
#![cfg(test)]

#[path = "../common/mod.rs"]
mod common;

mod checks;
mod steps;
#[allow(dead_code)]
#[path = "../bikri_preview/support.rs"]
mod support;
mod universe;

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use mandi_services::{ledger, partners};
use proptest::prelude::*;
use proptest::strategy::Union;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
use steps::{Line, Out, Step};
use universe::Setup;

/// A whole run: its world and its steps.
#[derive(Debug, Clone)]
struct Case {
    setup: Setup,
    steps: Vec<Step>,
}

fn line() -> impl Strategy<Value = Line> {
    (0_u8..8, 1_u8..8, 100_u16..1_000, any::<bool>()).prop_map(|(buyer, qty, rate, paid)| Line {
        buyer,
        qty,
        rate,
        paid,
    })
}

fn day() -> impl Strategy<Value = u8> {
    0_u8..3
}

/// The steps with their weights. Sales and the money around them come most often.
fn step() -> BoxedStrategy<Step> {
    let amount = || 1_u16..6_000;
    let items: Vec<(u32, BoxedStrategy<Step>)> = vec![
        (
            8,
            (
                0_u8..6,
                0_u8..3,
                proptest::collection::vec(line(), 1..4),
                0_u8..4,
                prop_oneof![3 => Just(0_u16), 1 => 1_u16..2_000],
                prop_oneof![3 => Just(0_u16), 1 => 1_u16..2_000],
                any::<bool>(),
                day(),
            )
                .prop_map(
                    |(seller, item, lines, extra, cash, freight, adjust, day)| Step::Sale {
                        seller,
                        item,
                        lines,
                        extra,
                        cash,
                        freight,
                        adjust,
                        day,
                    },
                )
                .boxed(),
        ),
        (
            4,
            (any::<u8>(), 0_u8..9, 100_u16..1_000, 0_u8..8, day())
                .prop_map(|(pick, qty, rate, buyer, day)| Step::Later {
                    pick,
                    qty,
                    rate,
                    buyer,
                    day,
                })
                .boxed(),
        ),
        (
            2,
            (
                any::<u8>(),
                0_u8..9,
                100_u16..1_000,
                0_u8..9,
                100_u16..1_000,
                day(),
            )
                .prop_map(|(pick, qty, rate, new_qty, new_rate, day)| Step::Mixed {
                    pick,
                    qty,
                    rate,
                    new_qty,
                    new_rate,
                    day,
                })
                .boxed(),
        ),
        (
            4,
            (any::<u8>(), any::<u8>(), any::<u16>())
                .prop_map(|(pick, change, value)| Step::EditSale {
                    pick,
                    change,
                    value,
                })
                .boxed(),
        ),
        (
            2,
            any::<u8>()
                .prop_map(|pick| Step::CancelSale { pick })
                .boxed(),
        ),
        (
            2,
            any::<u8>().prop_map(|pick| Step::UndoSale { pick }).boxed(),
        ),
        (
            3,
            any::<u8>().prop_map(|pick| Step::WriteOff { pick }).boxed(),
        ),
        (
            4,
            (any::<u8>(), amount(), day())
                .prop_map(|(who, amount, day)| Step::Receipt { who, amount, day })
                .boxed(),
        ),
        (
            2,
            (any::<u8>(), amount(), day())
                .prop_map(|(who, amount, day)| Step::Payment { who, amount, day })
                .boxed(),
        ),
        (
            4,
            (any::<u8>(), amount(), day())
                .prop_map(|(who, amount, day)| Step::Advance { who, amount, day })
                .boxed(),
        ),
        (
            2,
            (amount(), day())
                .prop_map(|(amount, day)| Step::Expense { amount, day })
                .boxed(),
        ),
        (
            2,
            (0_u8..3, amount(), day())
                .prop_map(|(partner, amount, day)| Step::Take {
                    partner,
                    amount,
                    day,
                })
                .boxed(),
        ),
        (
            3,
            (0_u8..3, amount(), day())
                .prop_map(|(partner, amount, day)| Step::Return {
                    partner,
                    amount,
                    day,
                })
                .boxed(),
        ),
        (
            2,
            (0_u8..3, 1_u16..40_000, day())
                .prop_map(|(partner, amount, day)| Step::CapitalIn {
                    partner,
                    amount,
                    day,
                })
                .boxed(),
        ),
        (
            1,
            (0_u8..3, 1_u16..40_000, day())
                .prop_map(|(partner, amount, day)| Step::CapitalOut {
                    partner,
                    amount,
                    day,
                })
                .boxed(),
        ),
        (
            1,
            (amount(), day())
                .prop_map(|(amount, day)| Step::General { amount, day })
                .boxed(),
        ),
        (
            2,
            (0_u8..2, amount(), day())
                .prop_map(|(person, amount, day)| Step::Lend {
                    person,
                    amount,
                    day,
                })
                .boxed(),
        ),
        (
            2,
            (0_u8..2, amount(), any::<bool>(), day())
                .prop_map(|(person, amount, salary, day)| Step::LendBack {
                    person,
                    amount,
                    salary,
                    day,
                })
                .boxed(),
        ),
        (
            2,
            (0_u8..3, amount(), day())
                .prop_map(|(khata, amount, day)| Step::KhataOut { khata, amount, day })
                .boxed(),
        ),
        (
            2,
            (0_u8..3, amount(), day())
                .prop_map(|(khata, amount, day)| Step::KhataIn { khata, amount, day })
                .boxed(),
        ),
        (
            3,
            (any::<u8>(), amount())
                .prop_map(|(pick, amount)| Step::VoucherEdit { pick, amount })
                .boxed(),
        ),
        (
            2,
            any::<u8>()
                .prop_map(|pick| Step::VoucherCancel { pick })
                .boxed(),
        ),
        (
            2,
            (2_u8..30, 100_u16..2_000, day())
                .prop_map(|(qty, rate, day)| Step::TripBuy { qty, rate, day })
                .boxed(),
        ),
        (
            2,
            (amount(), day())
                .prop_map(|(amount, day)| Step::TripCost { amount, day })
                .boxed(),
        ),
        (
            4,
            (any::<u8>(), any::<u8>(), 100_u16..3_000, day())
                .prop_map(|(pick, qty, rate, day)| Step::OwnSale {
                    pick,
                    qty,
                    rate,
                    day,
                })
                .boxed(),
        ),
        (
            4,
            (any::<u8>(), any::<u8>(), 100_u16..30_000, day())
                .prop_map(|(pick, qty, amount, day)| Step::TripOther {
                    pick,
                    qty,
                    amount,
                    day,
                })
                .boxed(),
        ),
        (
            2,
            (any::<u8>(), any::<u8>())
                .prop_map(|(pick, qty)| Step::TripWriteOff { pick, qty })
                .boxed(),
        ),
        (
            3,
            any::<u8>()
                .prop_map(|pick| Step::TripWriteOffUndo { pick })
                .boxed(),
        ),
        (
            4,
            any::<u8>()
                .prop_map(|pick| Step::TripCancel { pick })
                .boxed(),
        ),
        (
            3,
            (1_u16..60_000, day())
                .prop_map(|(amount, day)| Step::GInstall { amount, day })
                .boxed(),
        ),
        (
            2,
            (amount(), day())
                .prop_map(|(amount, day)| Step::GCost { amount, day })
                .boxed(),
        ),
        (
            2,
            (any::<u8>(), 100_u16..30_000, day())
                .prop_map(|(qty, amount, day)| Step::GOther { qty, amount, day })
                .boxed(),
        ),
        (
            3,
            (any::<u8>(), 100_u16..3_000, day())
                .prop_map(|(qty, rate, day)| Step::GOwnSale {
                    qty: qty % 40 + 1,
                    rate,
                    day,
                })
                .boxed(),
        ),
        (
            3,
            (1_u16..30_000)
                .prop_map(|cut| Step::GRelief { cut })
                .boxed(),
        ),
        (
            3,
            (amount(), day())
                .prop_map(|(amount, day)| Step::GRefund { amount, day })
                .boxed(),
        ),
        (
            1,
            any::<u8>().prop_map(|pick| Step::GCancel { pick }).boxed(),
        ),
        (3, day().prop_map(|day| Step::GClose { day }).boxed()),
        (3, Just(Step::GReopen).boxed()),
        (
            5,
            (0_u8..16, any::<u8>(), any::<u8>(), 100_u16..2_000, day())
                .prop_map(|(kind, pick, qty, rate, day)| Step::Fruit {
                    kind,
                    pick,
                    qty,
                    rate,
                    day,
                })
                .boxed(),
        ),
        (
            4,
            (any::<u8>(), any::<u8>())
                .prop_map(|(pick, qty)| Step::FruitEdit { pick, qty })
                .boxed(),
        ),
        (
            3,
            any::<u8>()
                .prop_map(|pick| Step::FruitCancel { pick })
                .boxed(),
        ),
        (
            2,
            (any::<bool>(), any::<bool>(), 1_u16..30_000, any::<bool>())
                .prop_map(|(seller, debit, amount, advance)| Step::NewAccount {
                    seller,
                    debit,
                    amount,
                    advance,
                })
                .boxed(),
        ),
        (4, any::<u8>().prop_map(|pick| Step::Dead { pick }).boxed()),
        (
            4,
            any::<u8>().prop_map(|pick| Step::DeadUndo { pick }).boxed(),
        ),
        (
            1,
            (1_u16..1_000, 1_u16..1_000, 1_u16..1_000)
                .prop_map(|(a, b, c)| Step::Shares { weights: [a, b, c] })
                .boxed(),
        ),
        // The steps of the plan 80 gate fixes: days that close and reopen (with the Owner's sign),
        // the undo, the garden's refund cap and its shares lock.
        (2, (0_u8..2).prop_map(|day| Step::DayClose { day }).boxed()),
        (
            3,
            (0_u8..2, 0_u8..5)
                .prop_map(|(day, sign)| Step::Reopen { day, sign })
                .boxed(),
        ),
        (
            3,
            (0_u8..2, any::<u8>(), any::<u8>(), any::<u16>(), 0_u8..5)
                .prop_map(|(day, pick, change, value, sign)| Step::FixCycle {
                    day,
                    pick,
                    change,
                    value,
                    sign,
                })
                .boxed(),
        ),
        (
            3,
            (
                0_u8..6,
                0_u8..3,
                any::<u8>(),
                100_u16..1_000,
                day(),
                any::<bool>(),
            )
                .prop_map(|(seller, item, qty, rate, day, edit)| Step::SaleUndo {
                    seller,
                    item,
                    qty,
                    rate,
                    day,
                    edit,
                })
                .boxed(),
        ),
        (3, any::<u16>().prop_map(|by| Step::GRaise { by }).boxed()),
        (
            2,
            (any::<u8>(), 1_u16..60_000)
                .prop_map(|(pick, amount)| Step::GInstallEdit { pick, amount })
                .boxed(),
        ),
        (
            1,
            (1_u16..1_000, 1_u16..1_000, 1_u16..1_000)
                .prop_map(|(a, b, c)| Step::GArrivalLock { weights: [a, b, c] })
                .boxed(),
        ),
    ];
    Union::new_weighted(items).boxed()
}

fn setup() -> impl Strategy<Value = Setup> {
    (
        [1_u32..1_000, 1_u32..1_000, 1_u32..1_000],
        [any::<bool>(), any::<bool>(), any::<bool>()],
        [1_u32..1_000, 1_u32..1_000, 1_u32..1_000],
        [any::<bool>(), any::<bool>(), any::<bool>()],
        (2_i64..40, 100_i64..2_000),
        1_i64..300_000,
    )
        .prop_map(
            |(trip_weights, trip_pockets, garden_weights, garden_pockets, purchase, rent)| Setup {
                trip_weights,
                trip_pockets,
                garden_weights,
                garden_pockets,
                purchase,
                rent,
            },
        )
}

/// The first steps of every run: a بکری with stock left, an advance, a partner take, two loans, a
/// garden installment, so that the steps that act on earlier entries (returns, edits, cancels,
/// fruit from a lot, dead amount) have something to act on from the start.
fn warm_up() -> impl Strategy<Value = Vec<Step>> {
    (
        (100_u16..1_000, 100_u16..1_000, 1_u8..8, 1_u8..8),
        1_000_u16..9_000,
        1_000_u16..9_000,
        1_000_u16..9_000,
        1_000_u16..9_000,
        1_000_u16..30_000,
    )
        .prop_map(
            |((rate_a, rate_b, qty_a, qty_b), advance, take, lend_a, lend_b, install)| {
                vec![
                    Step::Sale {
                        seller: 0,
                        item: 0,
                        lines: vec![
                            Line {
                                buyer: 0,
                                qty: qty_a,
                                rate: rate_a,
                                paid: false,
                            },
                            Line {
                                buyer: 1,
                                qty: qty_b,
                                rate: rate_b,
                                paid: true,
                            },
                        ],
                        extra: 6,
                        cash: 0,
                        freight: 0,
                        adjust: true,
                        day: 0,
                    },
                    Step::Advance {
                        who: 2,
                        amount: advance,
                        day: 0,
                    },
                    Step::Take {
                        partner: 0,
                        amount: take,
                        day: 0,
                    },
                    Step::Lend {
                        person: 0,
                        amount: lend_a,
                        day: 0,
                    },
                    Step::Lend {
                        person: 1,
                        amount: lend_b,
                        day: 0,
                    },
                    Step::GInstall {
                        amount: install,
                        day: 0,
                    },
                ]
            },
        )
}

fn case() -> impl Strategy<Value = Case> {
    (setup(), warm_up(), proptest::collection::vec(step(), 8..24)).prop_map(
        |(setup, mut steps, more)| {
            steps.extend(more);
            Case { setup, steps }
        },
    )
}

/// What each kind of step did over the whole run: (saved, nothing to act on, refused).
static TALLY: Mutex<BTreeMap<&'static str, (u32, u32, u32)>> = Mutex::new(BTreeMap::new());
static EXEC_MS: AtomicU64 = AtomicU64::new(0);
static CHECK_MS: AtomicU64 = AtomicU64::new(0);
static BUILD_MS: AtomicU64 = AtomicU64::new(0);
/// The refusals by step and code.
static REFUSALS: Mutex<BTreeMap<String, u32>> = Mutex::new(BTreeMap::new());

fn record(step: &Step, out: &Out) {
    let mut tally = TALLY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let slot = tally.entry(step.name()).or_default();
    match out {
        Out::Done => slot.0 += 1,
        Out::Skipped => slot.1 += 1,
        Out::Refused(code) => {
            slot.2 += 1;
            *REFUSALS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .entry(format!("{} {code:?}", step.name()))
                .or_default() += 1;
        }
    }
}

/// Runs a case and says how each step ended (D saved, S nothing to act on, R refused).
fn run_case(case: &Case) -> String {
    let t = Instant::now();
    let mut shape = String::new();
    let mut u = universe::build(&case.setup);
    BUILD_MS.fetch_add(t.elapsed().as_millis() as u64, Ordering::Relaxed);
    checks::check_all(&u, "the world before any step");
    for (i, step) in case.steps.iter().enumerate() {
        let before = checks::fingerprint(&u);
        let audits = checks::audit_rows(&u);
        let t = Instant::now();
        let out = steps::exec(&mut u, step);
        EXEC_MS.fetch_add(t.elapsed().as_millis() as u64, Ordering::Relaxed);
        let at = format!("step {i} {step:?} -> {out:?}");
        if out != Out::Done {
            assert_eq!(
                before,
                checks::fingerprint(&u),
                "{at}: a refused step left rows behind (NFR-REL-1)"
            );
        }
        // S7, NFR-SEC-5: a step that saved wrote an audit row in its own save; one that did not wrote none.
        let audits_now = checks::audit_rows(&u);
        if out == Out::Done {
            assert!(
                audits_now > audits,
                "{at}: S7: a saved step wrote no audit row"
            );
        } else {
            assert_eq!(
                audits_now, audits,
                "{at}: S7: a refused step left an audit row"
            );
        }
        // The closed-up-to day is the one the steps made it (the harness keeps its own idea of it), and
        // a closed day changes only through a reopen (PRD FR-6.6, FR-6.3, FR-10.11).
        let closed = checks::closed_up_to(&u);
        assert_eq!(
            checks::level_of(closed.as_deref()),
            u.closed_level,
            "{at}: the closed-up-to day is {closed:?}, not what the steps did to it"
        );
        if matches!(
            step,
            Step::DayClose { .. } | Step::Reopen { .. } | Step::FixCycle { .. }
        ) {
            u.closed_ids = checks::closed_entries(&u);
        } else {
            let now = checks::closed_entries(&u);
            assert_eq!(
                now,
                u.closed_ids,
                "{at}: O: a posting changed a closed day (up to {closed:?}), new or gone: {:?}",
                now.symmetric_difference(&u.closed_ids).collect::<Vec<_>>()
            );
        }
        shape.push(match out {
            Out::Done => 'D',
            Out::Skipped => 'S',
            Out::Refused(_) => 'R',
        });
        record(step, &out);
        let t = Instant::now();
        checks::check_all(&u, &at);
        CHECK_MS.fetch_add(t.elapsed().as_millis() as u64, Ordering::Relaxed);
    }
    // At the end: the chains verify, and every partner's statement ends at his کل رقم (FR-17.10).
    ledger::verify_journal_chain(&u.w.env.ctx).expect("the journal chain verifies");
    common::assert_chain_ok(&u.w.dir);
    let overview = partners::overview(&u.w.env.ctx).expect("the overview reads");
    for p in u.w.partners {
        let total = overview
            .iter()
            .find(|r| r.partner_id == p)
            .map_or(0, |r| r.total_paisa.as_paisa());
        let s = partners::statement(
            &u.w.env.ctx,
            &mandi_services::core::statement::PartnersStatementInput {
                group_by_month: false,
                sort: None,
                partner_id: p,
                from: None,
                to: None,
                offset: 0,
                limit: 1,
            },
        )
        .expect("the statement reads");
        assert_eq!(
            s.closing_total_paisa.as_paisa(),
            total,
            "the statement of partner {p} does not end at his کل رقم"
        );
    }
    shape
}

fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(30)
}

#[test]
fn every_posting_type_keeps_the_whole_books_exact() {
    let config = Config {
        cases: cases(),
        max_shrink_iters: 120,
        failure_persistence: None,
        ..Config::default()
    };
    let mut runner = if std::env::var("T1_RANDOM").is_ok() {
        TestRunner::new(config)
    } else {
        TestRunner::new_with_rng(config, TestRng::deterministic_rng(RngAlgorithm::ChaCha))
    };
    // A cancel that saves needs an entry to exist and none of its goods to have sold, which a random
    // order seldom gives. This fixed run saves it (a cost, then a sale in another shop, each
    // cancelled) with every check of a normal run, so the kind is never a hole in the tally below.
    let prelude = run_case(&fixed(vec![
        Step::TripCost {
            amount: 4_001,
            day: 0,
        },
        Step::TripCancel { pick: 0 },
        Step::TripOther {
            pick: 0,
            qty: 2,
            amount: 5_000,
            day: 1,
        },
        Step::TripCancel { pick: 0 },
    ]));
    assert_eq!(prelude, "DDDD", "the fixed run of the cancels");
    let result = runner.run(&case(), |case| {
        let _ = run_case(&case);
        Ok(())
    });
    let tally = TALLY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    eprintln!(
        "\nms: build {} steps {} checks {}",
        BUILD_MS.load(Ordering::Relaxed),
        EXEC_MS.load(Ordering::Relaxed),
        CHECK_MS.load(Ordering::Relaxed)
    );
    eprintln!("step kind                  saved  nothing  refused");
    for (name, (done, skipped, refused)) in tally.iter() {
        eprintln!("{name:<26} {done:>5} {skipped:>8} {refused:>8}");
    }
    eprintln!("refusals:");
    for (what, n) in REFUSALS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
    {
        eprintln!("  {what}: {n}");
    }
    if let Err(error) = result {
        panic!("{error}");
    }
    // Every kind of posting must really have been saved in the run, or the run proves nothing
    // about it. (A kind that never saves is a hole in this test, not a pass.)
    let missing: Vec<&&str> = tally
        .iter()
        .filter(|(_, (done, _, _))| *done == 0)
        .map(|(name, _)| name)
        .collect();
    if cases() >= 30 && std::env::var("T1_RANDOM").is_err() {
        assert!(
            missing.is_empty(),
            "these steps never saved in the whole run: {missing:?}"
        );
    }
}

/// A run from a fixed list of steps (the shrunk cases of the findings).
#[allow(dead_code)]
fn fixed(steps: Vec<Step>) -> Case {
    Case {
        setup: Setup {
            trip_weights: [1, 1, 1],
            trip_pockets: [false; 3],
            garden_weights: [1, 1, 1],
            garden_pockets: [false; 3],
            purchase: (2, 100),
            rent: 1,
        },
        steps,
    }
}

#[allow(dead_code)]
fn sale_step(day: u8, lines: Vec<Line>) -> Step {
    Step::Sale {
        seller: 0,
        item: 0,
        lines,
        extra: 0,
        cash: 0,
        freight: 0,
        adjust: false,
        day,
    }
}

// ---- Fixed runs: the hard corners of the advance rules (PRD FR-17.13, C28), by hand ----
//
// The random run reaches these by chance. Each one below is a run of exactly the steps that make the
// corner, and the same checks run after every step.

fn bought(buyer: u8, qty: u8, rate: u16) -> Line {
    Line {
        buyer,
        qty,
        rate,
        paid: false,
    }
}

#[test]
fn an_advance_is_settled_after_the_shares_changed() {
    // رحیم takes 15,000 on 10-10 at 40 / 30 / 30. On 12-10 the shares become a third each. His بکری
    // of that day settles part of the advance, and so does his own وصولی. The partners get the
    // settlement back by the parts of THAT advance, not by the new shares (C28).
    let shape = run_case(&fixed(vec![
        Step::Advance {
            who: 0,
            amount: 15_000,
            day: 0,
        },
        Step::Shares { weights: [1, 1, 1] },
        Step::Sale {
            seller: 0,
            item: 0,
            lines: vec![bought(0, 20, 600)],
            extra: 0,
            cash: 0,
            freight: 0,
            adjust: true,
            day: 2,
        },
        Step::Receipt {
            who: 2,
            amount: 3_001,
            day: 2,
        },
        Step::Advance {
            who: 0,
            amount: 7_777,
            day: 2,
        },
        Step::Receipt {
            who: 2,
            amount: 4_999,
            day: 2,
        },
    ]));
    // Every step saved: a test that skips its steps proves nothing.
    assert_eq!(shape, "DDDDDD");
}

#[test]
fn a_seller_in_the_dead_amount_with_an_open_advance() {
    // The advance stays out of the partners' totals while he is dead (nothing posts, FR-10.10),
    // and comes back to them when his recovery settles it.
    let shape = run_case(&fixed(vec![
        Step::Advance {
            who: 0,
            amount: 9_999,
            day: 0,
        },
        Step::Dead { pick: 2 },
        Step::Receipt {
            who: 2,
            amount: 4_000,
            day: 1,
        },
        Step::DeadUndo { pick: 0 },
        Step::Receipt {
            who: 2,
            amount: 5_999,
            day: 2,
        },
    ]));
    // Every step saved: a test that skips its steps proves nothing.
    assert_eq!(shape, "DDDDD");
}

#[test]
fn a_sale_that_settled_an_advance_is_edited_and_cancelled() {
    // E14 and E15 with advances: the settlement follows the edit, then the cancel gives the advance
    // back to the seller and the parts back to the partners.
    let shape = run_case(&fixed(vec![
        Step::Advance {
            who: 0,
            amount: 12_345,
            day: 0,
        },
        Step::Sale {
            seller: 0,
            item: 0,
            lines: vec![bought(0, 10, 500), bought(1, 7, 450)],
            extra: 5,
            cash: 0,
            freight: 100,
            adjust: true,
            day: 1,
        },
        Step::EditSale {
            pick: 0,
            change: 0,
            value: 130,
        },
        Step::EditSale {
            pick: 0,
            change: 3,
            value: 77,
        },
        Step::Later {
            pick: 0,
            qty: 1,
            rate: 300,
            buyer: 1,
            day: 2,
        },
        Step::CancelSale { pick: 0 },
    ]));
    // Every step saved: a test that skips its steps proves nothing.
    assert_eq!(shape, "DDDDDD");
}

#[test]
fn an_opening_advance_with_the_shares_of_its_day() {
    // A new seller made with an opening advance (FR-10.9, FR-17.13), then a share change, then a
    // بکری of his that settles it.
    let shape = run_case(&fixed(vec![
        Step::NewAccount {
            seller: true,
            debit: true,
            amount: 20_000,
            advance: true,
        },
        Step::Shares { weights: [5, 3, 2] },
        Step::Sale {
            seller: 3,
            item: 1,
            lines: vec![bought(0, 30, 700)],
            extra: 0,
            cash: 500,
            freight: 0,
            adjust: true,
            day: 2,
        },
        Step::Payment {
            who: 5,
            amount: 1_000,
            day: 2,
        },
    ]));
    // Every step saved: a test that skips its steps proves nothing.
    assert_eq!(shape, "DDDD");
}

#[test]
fn a_garden_closed_after_relief_and_a_refund() {
    // PRD FR-12.51 and C26: installments over a lowered rent, the owner's refund, sales, the closing,
    // the undo of the closing and the closing again.
    let shape = run_case(&fixed(vec![
        Step::GInstall {
            amount: 40_000,
            day: 0,
        },
        Step::GRelief { cut: 29_999 },
        Step::GRefund {
            amount: 5_000,
            day: 2,
        },
        Step::GOwnSale {
            qty: 5,
            rate: 900,
            day: 1,
        },
        Step::GOther {
            qty: 3,
            amount: 12_000,
            day: 1,
        },
        Step::GClose { day: 2 },
        Step::GReopen,
        Step::GClose { day: 2 },
    ]));
    // Every step saved: a test that skips its steps proves nothing.
    assert_eq!(shape, "DDDDDDDD");
}

#[test]
fn a_trip_costed_sold_written_off_and_cancelled() {
    // C21 and C24: a cost spread over two purchases, sales in our shop and in another shop, a
    // write-off and its undo, a cancel of the cost (refused once goods have sold).
    let shape = run_case(&fixed(vec![
        Step::TripBuy {
            qty: 9,
            rate: 700,
            day: 0,
        },
        Step::TripCost {
            amount: 4_001,
            day: 0,
        },
        Step::OwnSale {
            pick: 0,
            qty: 0,
            rate: 900,
            day: 1,
        },
        Step::TripOther {
            pick: 1,
            qty: 3,
            amount: 6_000,
            day: 1,
        },
        Step::TripWriteOff { pick: 0, qty: 0 },
        Step::TripWriteOffUndo { pick: 0 },
        Step::TripCancel { pick: 0 },
        Step::OwnSale {
            pick: 1,
            qty: 200,
            rate: 800,
            day: 2,
        },
    ]));
    // Every step saved, but one (the cancel of the cost is refused: goods of its purchases have sold, PRD FR-12.31): a test that skips its steps proves nothing.
    assert_eq!(shape, "DDDDDDRD");
}

// ---- Fixed runs: the fixes of the plan 80 gate (R2-2, Q-R2-2, Q-R4-1, Q-R4-2, R4-3) ----

#[test]
fn an_undo_works_straight_after_a_save_and_not_after_a_later_sale_or_an_edit() {
    // PRD FR-7.47, plan 80 R2-2. Each undo is judged against the harness's own list of the بکری that
    // had a later sale, a carried posting or an edit; the refusal is excused only there.
    let shape = run_case(&fixed(vec![
        // بکری 1 keeps 4 نگ, so a later sale can touch it.
        Step::Sale {
            seller: 0,
            item: 0,
            lines: vec![bought(0, 5, 500)],
            extra: 4,
            cash: 0,
            freight: 0,
            adjust: false,
            day: 1,
        },
        Step::Later {
            pick: 0,
            qty: 1,
            rate: 300,
            buyer: 1,
            day: 2,
        },
        // It had a later sale: the undo is refused (ERR_VERSION_CHANGED).
        Step::UndoSale { pick: 0 },
        // A new بکری and the undo straight after it: works.
        Step::SaleUndo {
            seller: 1,
            item: 1,
            qty: 3,
            rate: 640,
            day: 1,
            edit: false,
        },
        // A new بکری, edited, then the undo: refused, and the step is done (the sale and the edit saved).
        Step::SaleUndo {
            seller: 1,
            item: 2,
            qty: 3,
            rate: 640,
            day: 1,
            edit: true,
        },
        // Saved بکری now: 1 (later sale), 3 (edited), 4 (fresh). The undo of the fresh one works.
        sale_step(2, vec![bought(1, 2, 700)]),
        Step::UndoSale { pick: 2 },
    ]));
    assert_eq!(shape, "DDRDDDD");
}

#[test]
fn the_plain_reopen_and_the_fix_ask_the_owners_sign_and_close_the_days_again() {
    // PRD FR-21.2, FR-21.4, FR-7.71, plan 80 Q-R2-2: a wrong sign opens nothing and is audited once
    // with no typed text; a blank sign is refused first; the right sign opens, and the fix closes the
    // same days again with the sign on the edit's audit row.
    let shape = run_case(&fixed(vec![
        sale_step(0, vec![bought(0, 10, 500)]),
        Step::DayClose { day: 1 },
        Step::Reopen { day: 0, sign: 3 },
        Step::Reopen { day: 0, sign: 4 },
        Step::FixCycle {
            day: 0,
            pick: 0,
            change: 0,
            value: 25,
            sign: 3,
        },
        Step::FixCycle {
            day: 0,
            pick: 0,
            change: 0,
            value: 25,
            sign: 4,
        },
        Step::FixCycle {
            day: 0,
            pick: 0,
            change: 0,
            value: 25,
            sign: 2,
        },
        Step::FixCycle {
            day: 1,
            pick: 0,
            change: 0,
            value: 26,
            sign: 1,
        },
        Step::Reopen { day: 0, sign: 0 },
        // The 11th is open now: nothing to reopen.
        Step::Reopen { day: 1, sign: 0 },
        Step::DayClose { day: 0 },
        // Already closed.
        Step::DayClose { day: 0 },
        // A posting on a closed day is refused.
        sale_step(0, vec![bought(0, 1, 500)]),
    ]));
    assert_eq!(shape, "DDDRDRDDDRDRR");
}

#[test]
fn goods_arrived_locks_the_shares_of_a_garden() {
    // PRD FR-12.29, plan 80 Q-R4-1: the shares change while the garden has no entry; once goods
    // arrived they are locked by the service and by the three triggers of migration 26, and a
    // cancelled arrival still counts.
    let shape = run_case(&fixed(vec![Step::GArrivalLock { weights: [1, 2, 3] }]));
    assert_eq!(shape, "D");
}

#[test]
fn the_refunds_of_a_garden_owner_stay_covered() {
    // PRD FR-12.51, plan 80 R4-3 and Q-R4-2: rent 1, installments 40,000, a refund of 5,001. A rent
    // raised over installments less refunds is refused, one rupee under it is allowed; the installment
    // cancelled or edited down is refused, edited up is allowed; the closing asks (the owner still owes)
    // and goes through with the answer yes.
    let shape = run_case(&fixed(vec![
        Step::GInstall {
            amount: 40_000,
            day: 0,
        },
        Step::GRefund {
            amount: 5_000,
            day: 2,
        },
        Step::GRaise { by: 2 },
        Step::GRaise { by: 0 },
        Step::GRaise { by: 1 },
        Step::GCancel { pick: 0 },
        Step::GInstallEdit {
            pick: 0,
            amount: 30_000,
        },
        Step::GInstallEdit {
            pick: 0,
            amount: 50_000,
        },
        Step::GClose { day: 2 },
    ]));
    assert_eq!(shape, "DDRDRRRDD");
}
