//! The garden figures of the reports against the garden screen (plan d5-late round 11, money agent
//! T15, T16; Code Standards principle 2, G6).
//!
//! `mandi-reports` may not depend on `mandi-services` (Architecture 3), so the reports keep their own
//! copy of the garden derivation (`mandi_db::garden_figures::Money::of`) next to the one the garden
//! screen uses (`services::gardens::read`). Two things hold them together:
//!
//! 1. the copy against the formula of the screen on 10,000 random sets of stored sums, errors
//!    included (the formula is restated here in plain checked maths);
//! 2. random sequences of installments, costs, sales in our shop, sales in other shops, cancels,
//!    closings and undos on a real garden: after every step R36 (both layouts), view I and the
//!    dashboard tile give the numbers of `gardens_get`, `gardens_list` and `gardens_summary`.
//!
//! If the screen's working ever changes, this fails first. Any difference is a defect.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_db::garden_figures::{Money, MoneyDataError};
use mandi_db::gardens::Figures;
use mandi_reports::analytics::{AnalyticsFilters, AnalyticsRunInput, AnalyticsView, run};
use mandi_reports::dashboard::{self, BackupFacts};
use mandi_reports::reports::{Cell, GardenView, ReportFilters, ReportId};
use mandi_services::core::bikri::CancelInput;
use mandi_services::core::gardens::{
    GardenEntryCancelInput, GardenGetInput, GardenHistoryKind, GardenReopenInput, GardensListInput,
};
use mandi_services::{bikri, gardens};
use proptest::prelude::*;
use reports_world::*;

// ---- 1. The copy of the derivation against the formula -------------------------------------------

/// Whole rupees in paisa, from -3 to 9,000 rupees (a sum can be below 0: a debit balance).
fn sum() -> impl Strategy<Value = i64> {
    (-3_i64..=9_000).prop_map(|rupees| rupees * 100)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn the_derivation_is_the_formula_of_the_screen(
        rent in sum(),
        installments in sum(),
        costs in sum(),
        shop_extra in -2_i64..=2,
        stray in 0_u8..=3,
        credits in sum(),
        debits in sum(),
        account_credits in sum(),
        account_debits in sum(),
        other in sum(),
    ) {
        // The parts paid from the shop and from pockets add up to what was paid, unless the test
        // breaks them on purpose (shop_extra != 0). A stray 50 paisa in a stored sum is refused
        // (money is whole rupees).
        let paid = installments + costs;
        let paid_shop = (paid / 200) * 100;
        let stray_paisa = if shop_extra == 0 { 50 } else { 0 };
        let figures = Figures {
            installments_paisa: installments,
            costs_paisa: costs,
            paid_shop_paisa: paid_shop + shop_extra * 100,
            paid_pocket_paisa: paid - paid_shop,
            other_shop_paisa: other + if stray == 1 { stray_paisa } else { 0 },
            credits_paisa: credits + if stray == 2 { stray_paisa } else { 0 },
            debits_paisa: debits + if stray == 3 { stray_paisa } else { 0 },
            account_credits_paisa: account_credits,
            account_debits_paisa: account_debits,
            last_date: Some("2026-10-10".into()),
        };
        let ours = Money::of(rent, &figures);
        if shop_extra != 0 {
            prop_assert_eq!(ours, Err(MoneyDataError::PartsDoNotAdd));
        } else if stray != 0 {
            prop_assert!(matches!(ours, Err(MoneyDataError::Money(_))));
        } else {
            let m = ours.unwrap();
            prop_assert_eq!(m.installments.as_paisa(), installments);
            prop_assert_eq!(m.costs.as_paisa(), costs);
            prop_assert_eq!(m.paid.as_paisa(), paid);
            prop_assert_eq!(m.money_in.as_paisa(), credits - debits);
            prop_assert_eq!(m.balance.as_paisa(), credits - debits - paid);
            prop_assert_eq!(m.rent_due.as_paisa(), rent - installments);
            prop_assert_eq!(m.account_balance.as_paisa(), account_credits - account_debits);
            prop_assert_eq!(m.paid_shop.as_paisa() + m.paid_pocket.as_paisa(), m.paid.as_paisa());
            prop_assert_eq!(m.other_shop.as_paisa(), other);
            prop_assert_eq!(m.last_date.as_deref(), Some("2026-10-10"));
        }
    }
}

// ---- 2. Random sequences on a real garden --------------------------------------------------------------

/// A small deterministic generator, so a failing sequence can be run again from its seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn detail(w: &support::World, id: i64) -> mandi_services::core::gardens::GardenDetail {
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

fn n(cell: &Cell) -> i64 {
    match cell {
        Cell::Money { paisa } => paisa.rupees(),
        Cell::Percent { hundredths } => *hundredths,
        Cell::Number { value } => *value,
        Cell::Empty => 0,
        other => panic!("not a number: {other:?}"),
    }
}

fn off() -> BackupFacts {
    BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    }
}

/// R36, view I and the tile against the screen, for the one garden of the world.
fn assert_same(w: &support::World, id: i64, step: &str) {
    let d = detail(w, id);
    let row = &d.row;
    let page = report(w, ReportId::R36, ReportFilters::default(), 0, 0);
    assert_eq!(page.rows.len(), 1, "{step}");
    let f: Vec<i64> = page.rows[0].cells[3..].iter().map(n).collect();
    // rent, installments, costs, paid shop, paid pocket, our shop, other shops, other in,
    // commission, money in, balance, rent due.
    let screen = [
        row.rent_paisa.rupees(),
        row.installments_paisa.rupees(),
        row.costs_paisa.rupees(),
        row.paid_shop_paisa.rupees(),
        row.paid_pocket_paisa.rupees(),
    ];
    assert_eq!(f[..5], screen, "{step}");
    assert_eq!(f[6], row.other_shop_paisa.rupees(), "{step}");
    assert_eq!(f[9], row.money_in_paisa.rupees(), "{step}");
    assert_eq!(f[10], row.balance_paisa.rupees(), "{step}");
    assert_eq!(f[11], row.rent_due_paisa.rupees(), "{step}");
    // The money in splits into its sources, and what was paid into its two sources.
    assert_eq!(f[5] + f[6] + f[7], f[9], "{step}");
    assert_eq!(f[3] + f[4], f[1] + f[2], "{step}");

    // The list of the screen has the same totals.
    let list = gardens::list(&w.env.ctx, &GardensListInput::default()).unwrap();
    assert_eq!(list.totals.balance_paisa.rupees(), f[10], "{step}");
    assert_eq!(list.totals.money_in_paisa.rupees(), f[9], "{step}");
    assert_eq!(list.totals.rent_due_paisa.rupees(), f[11], "{step}");

    // Each partner: share, paid from the shop and from his pocket, back and result.
    let by_partner = report(
        w,
        ReportId::R36,
        ReportFilters {
            garden_view: Some(GardenView::Partner),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    let mut shown: Vec<(i64, i64, i64, i64, i64)> = by_partner
        .rows
        .iter()
        .map(|r| {
            (
                n(&r.cells[2]),
                n(&r.cells[3]),
                n(&r.cells[4]),
                n(&r.cells[5]),
                n(&r.cells[6]),
            )
        })
        .collect();
    let mut expected: Vec<(i64, i64, i64, i64, i64)> = d
        .partner_results
        .iter()
        .map(|p| {
            (
                i64::from(p.share_bps),
                p.paid_shop_paisa.rupees(),
                p.paid_pocket_paisa.rupees(),
                p.money_back_paisa.rupees(),
                p.net_paisa.rupees(),
            )
        })
        .collect();
    shown.sort_unstable();
    expected.sort_unstable();
    assert_eq!(shown, expected, "{step}");
    // Once closed the results add up to the balance; while open they are minus what was paid.
    let results: i64 = shown.iter().map(|p| p.4).sum();
    if d.row.status == mandi_services::core::ventures::VentureStatus::Closed
        && d.row.account_balance_paisa.rupees() == 0
    {
        assert_eq!(results, f[10], "{step}");
    }

    // View I.
    let i = run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            sort: None,
            view: AnalyticsView::I,
            filters: AnalyticsFilters::default(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let cells = &i.rows[0].cells;
    assert_eq!(n(&cells[2]), f[3], "{step}");
    assert_eq!(n(&cells[3]), f[4], "{step}");
    assert_eq!(n(&cells[4]), f[5], "{step}");
    assert_eq!(n(&cells[5]), f[6], "{step}");
    assert_eq!(n(&cells[6]), f[7], "{step}");
    assert_eq!(n(&cells[7]), f[8], "{step}");
    assert_eq!(n(&cells[8]), f[10], "{step}");

    // View G: the movements of the periods add up to the balance (the closing is not money in).
    let g = run(
        w.env.ctx.db(),
        &AnalyticsRunInput {
            sort: None,
            view: AnalyticsView::G,
            filters: AnalyticsFilters::default(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let at = g
        .columns
        .iter()
        .position(|c| c.key == "gardens_result")
        .unwrap();
    assert_eq!(n(&g.totals[at]), f[10], "{step}: G gardens result");

    // View F: a partner's money from the garden is his stored closing part less his shop parts.
    for p in &d.partner_results {
        let f_view = run(
            w.env.ctx.db(),
            &AnalyticsRunInput {
                sort: None,
                view: AnalyticsView::F,
                filters: AnalyticsFilters {
                    partner_id: Some(p.partner.id),
                    ..AnalyticsFilters::default()
                },
                offset: 0,
                limit: 0,
            },
        )
        .unwrap();
        let at = f_view
            .columns
            .iter()
            .position(|c| c.key == "gardens")
            .unwrap();
        assert_eq!(
            n(&f_view.totals[at]),
            p.money_back_paisa.rupees() - p.paid_shop_paisa.rupees(),
            "{step}: F gardens of partner {}",
            p.partner.id
        );
    }

    // R34 on the day of every step: what was paid into the garden and what came in.
    let day = report(
        w,
        ReportId::R34,
        ReportFilters {
            from: "10-10-2026".into(),
            to: "10-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        0,
    );
    let amount_of = |label: &str| {
        day.rows
            .iter()
            .find(|r| matches!(&r.cells[1], Cell::Label { id } if id == label))
            .map(|r| n(&r.cells[3]))
            .unwrap()
    };
    assert_eq!(
        amount_of("report-r34-gardens-paid"),
        f[1] + f[2],
        "{step}: R34 paid"
    );
    assert_eq!(
        amount_of("report-r34-gardens-in"),
        f[9],
        "{step}: R34 money in"
    );

    // The tile is the summary of the screen.
    let tile = dashboard::get(w.env.ctx.db(), off()).unwrap();
    let summary = gardens::summary(&w.env.ctx).unwrap();
    assert_eq!(tile.tiles.gardens_open, summary.open_gardens, "{step}");
    assert_eq!(
        tile.tiles.gardens_balance_paisa.rupees(),
        summary.balance_total_paisa.rupees(),
        "{step}"
    );
    assert_eq!(tile.gardens.len(), summary.rows.len(), "{step}");
    for (a, b) in tile.gardens.iter().zip(&summary.rows) {
        assert_eq!(
            (a.id, a.balance_paisa, a.rent_due_paisa),
            (b.id, b.balance_paisa, b.rent_due_paisa),
            "{step}"
        );
    }
}

/// Runs one sequence; returns how many steps closed the garden and how many sold in our shop, so
/// the test can check that the sequences really reach those states.
fn one_sequence(seed: u64) -> (usize, usize) {
    let (mut closings, mut sales) = (0_usize, 0_usize);
    let w = worked_day();
    let garden = garden_made(&w);
    let mut rng = Rng(seed | 1);
    assert_same(&w, garden.id, "made");
    for step in 0..14 {
        let d = detail(&w, garden.id);
        let open = d.row.status == mandi_services::core::ventures::VentureStatus::Open;
        let what;
        match rng.below(8) {
            0 | 1 if open => {
                what = "payment";
                let cost = rng.below(2) == 0;
                let amount = format!("{}", (1 + rng.below(120)) * 100);
                // The shop's part may take cash below zero: then the save is refused, which is fine.
                let entry = if cost {
                    mandi_services::core::gardens::GardenEntryInput::Cost
                } else {
                    mandi_services::core::gardens::GardenEntryInput::Installment
                }(mandi_services::core::gardens::GardenPaymentInput {
                    garden_id: Some(garden.id),
                    date: "10-10-2026".into(),
                    amount,
                    reason: "x".into(),
                    parts: Vec::new(),
                    confirm_cash_below_zero: false,
                });
                let _ = gardens::entry_save(&w.env.ctx, &w.owner, &entry);
            }
            2 if open => {
                what = "sale in our shop";
                let qty = 1 + rng.below(40);
                let rate = 50 + rng.below(850);
                let buyer = if rng.below(2) == 0 { w.arshad } else { w.akram };
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
                        buyer,
                        &qty.to_string(),
                        &rate.to_string(),
                    )],
                );
                lot.adjust_advance = false;
                let mut draft = support::draft(vec![lot]);
                draft.header.entry_date = "10-10-2026".into();
                let _ = bikri::save(
                    &w.env.ctx,
                    &w.owner,
                    &mandi_services::core::bikri::BikriSaveInput {
                        draft,
                        confirm_cash_below_zero: true,
                    },
                );
            }
            3 if open => {
                what = "sale in another shop";
                let amount = format!("{}", (1 + rng.below(200)) * 100);
                let _ = gardens::entry_save(
                    &w.env.ctx,
                    &w.owner,
                    &mandi_services::core::gardens::GardenEntryInput::OtherSale(
                        mandi_services::core::gardens::GardenOtherSaleInput {
                            arrival_id: None,
                            garden_id: Some(garden.id),
                            date: "10-10-2026".into(),
                            shop: Some("Haji".into()),
                            qty: "5".into(),
                            amount,
                            note: None,
                        },
                    ),
                );
            }
            4 => {
                what = "cancel an entry";
                let entries: Vec<(i64, i64)> = d
                    .history
                    .iter()
                    .filter(|h| {
                        matches!(
                            h.kind,
                            GardenHistoryKind::Installment
                                | GardenHistoryKind::Cost
                                | GardenHistoryKind::OtherSale
                        )
                    })
                    .filter_map(|h| h.entry_id.zip(h.version))
                    .collect();
                if !entries.is_empty() {
                    let (id, version) =
                        entries[usize::try_from(rng.below(entries.len() as u64)).unwrap_or(0)];
                    let _ = gardens::entry_cancel(
                        &w.env.ctx,
                        &w.owner,
                        &GardenEntryCancelInput {
                            id,
                            version,
                            reason: "x".into(),
                            confirm_cash_below_zero: true,
                        },
                    );
                }
            }
            5 => {
                what = "cancel a sale";
                let sales: Vec<i64> = d
                    .history
                    .iter()
                    .filter(|h| h.kind == GardenHistoryKind::Sale)
                    .filter_map(|h| h.bikri_id)
                    .collect();
                if !sales.is_empty() {
                    let id = sales[usize::try_from(rng.below(sales.len() as u64)).unwrap_or(0)];
                    if let Ok(saved) = bikri::get(&w.env.ctx, id) {
                        let _ = bikri::cancel(
                            &w.env.ctx,
                            &w.owner,
                            &CancelInput {
                                id,
                                version: saved.version,
                                reason: "x".into(),
                                confirm_cash_below_zero: true,
                            },
                        );
                    }
                }
            }
            6 if open => {
                what = "close";
                let _ = gardens::close(
                    &w.env.ctx,
                    &w.owner,
                    &mandi_services::core::gardens::GardenCloseInput {
                        id: garden.id,
                        version: d.row.version,
                        date: "10-10-2026".into(),
                    },
                );
            }
            7 if !open => {
                what = "undo the closing";
                let _ = gardens::reopen(
                    &w.env.ctx,
                    &w.owner,
                    &GardenReopenInput {
                        id: garden.id,
                        version: d.row.version,
                        reason: "x".into(),
                    },
                );
            }
            _ => continue,
        }
        assert_same(&w, garden.id, &format!("seed {seed} step {step}: {what}"));
        let after = detail(&w, garden.id);
        if what == "close"
            && after.row.status == mandi_services::core::ventures::VentureStatus::Closed
        {
            closings += 1;
        }
        if what == "sale in our shop"
            && after
                .history
                .iter()
                .any(|h| h.kind == GardenHistoryKind::Sale)
        {
            sales += 1;
        }
    }
    (closings, sales)
}

#[test]
fn random_sequences_on_a_garden_give_the_numbers_of_the_screen() {
    // Seeds are fixed, so a failure is run again with the same sequence.
    let (mut closings, mut sales) = (0_usize, 0_usize);
    for seed in 0..300_u64 {
        let (c, s) = one_sequence(0x9E37_79B9_7F4A_7C15_u64.wrapping_mul(seed + 1));
        closings += c;
        sales += s;
    }
    // The sequences really close gardens and sell from them.
    assert!(closings >= 50, "{closings} closings");
    assert!(sales >= 100, "{sales} sales");
}

#[test]
fn the_predicate_of_the_closing_is_the_one_of_the_garden_module() {
    let ours = mandi_db::garden_figures::NOT_A_CLOSING;
    // The text of `gardens.rs` (private there): its lines must be the same as ours, word for word.
    let source = include_str!("../../db/src/gardens.rs");
    let squash = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        squash(source).contains(&squash(ours)),
        "NOT_A_CLOSING of garden_figures.rs is not the one of gardens.rs"
    );
}
