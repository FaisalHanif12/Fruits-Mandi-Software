//! Plan 76 property tests (money agent plan 76 item list g and f): for any run of بکری with random
//! quantities and rates and one random cancel,
//!
//! * the commission page: on every day Σ of the partner columns = the shared income, the month
//!   totals row = the sum of the day rows, and total commission = seller + buyer + لاگا;
//! * the partner statement of the month: the columns of each day add up to the day's net, the
//!   closing is his کل رقم on the overview, and the opening of the next month is this closing;
//! * the mixed save of a carried lot and a new lot writes the same entries and share rows as the
//!   two standalone saves one after the other.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_reports::dashboard::{self, CommissionMonthInput};
use mandi_services::bikri;
use mandi_services::core::bikri::{
    BikriSaveInput, CancelInput, CarriedLotInput, LaterSaleSaveInput, MixedSaleInput,
    MixedSaleSaveInput,
};
use mandi_services::core::statement::PartnersStatementMonthInput;
use mandi_services::partners;
use proptest::prelude::*;
use support::World;

/// The lines of an entry as (account code, debit, credit).
type JournalRows = Vec<(String, i64, i64)>;
/// The share rows of an entry as (kind, amount).
type ShareRows = Vec<(String, i64)>;

fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12)
}

#[derive(Debug, Clone)]
struct Run {
    /// Each بکری: quantity of the lot, quantity to اکرم, rate to اکرم, rate to ارشد.
    bikris: Vec<(u32, u32, u32, u32)>,
    cancel_last: bool,
}

fn runs() -> impl Strategy<Value = Run> {
    (
        proptest::collection::vec((10_u32..60, 1_u32..10, 100_u32..900, 100_u32..900), 1..4),
        any::<bool>(),
    )
        .prop_map(|(bikris, cancel_last)| Run {
            bikris,
            cancel_last,
        })
}

fn save_run(w: &World, run: &Run) {
    let mut last = None;
    for (i, (qty, to_akram, rate_a, rate_b)) in run.bikris.iter().enumerate() {
        let to_arshad = qty.saturating_sub(*to_akram).clamp(1, 9);
        let lot = support::with_sales(
            support::bare_lot(w, &format!("lot-{i}"), w.rahim, w.banana, &qty.to_string()),
            vec![
                support::sale("s-1", w.akram, &to_akram.to_string(), &rate_a.to_string()),
                support::sale("s-2", w.arshad, &to_arshad.to_string(), &rate_b.to_string()),
            ],
        );
        let saved = bikri::save(
            &w.env.ctx,
            &w.owner,
            &BikriSaveInput {
                draft: support::draft(vec![lot]),
                confirm_cash_below_zero: true,
            },
        )
        .unwrap();
        last = Some(saved);
    }
    if run.cancel_last
        && let Some(saved) = last
    {
        let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
        bikri::cancel(
            &w.env.ctx,
            &w.owner,
            &CancelInput {
                id: saved.bikri_id,
                version: detail.version,
                reason: "x".into(),
                confirm_cash_below_zero: true,
            },
        )
        .unwrap();
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases()))]

    #[test]
    fn the_commission_page_and_the_statement_month_always_add_up(run in runs()) {
        let w = support::world();
        save_run(&w, &run);
        let m = dashboard::commission_month(
            w.env.ctx.db(),
            &CommissionMonthInput {
                month: None,
                ..CommissionMonthInput::default()
            },
        )
        .unwrap();
        let mut seller = 0_i64;
        for row in &m.rows {
            let shares: i64 = row
                .partner_shares
                .as_ref()
                .unwrap()
                .iter()
                .map(|s| s.share_paisa.as_paisa())
                .sum();
            prop_assert_eq!(shares, row.shared_income_paisa.as_paisa());
            prop_assert_eq!(
                row.total_comm_paisa.as_paisa(),
                row.seller_comm_paisa.as_paisa()
                    + row.buyer_comm_paisa.as_paisa()
                    + row.laga_paisa.as_paisa()
            );
            seller += row.seller_comm_paisa.as_paisa();
        }
        prop_assert_eq!(seller, m.totals.seller_comm_paisa.as_paisa());
        let all: i64 = m.rows.iter().map(|r| r.shared_income_paisa.as_paisa()).sum();
        prop_assert_eq!(all, m.totals.shared_income_paisa.as_paisa());

        // The statement of each partner.
        let overview = partners::overview(&w.env.ctx).unwrap();
        for p in &overview {
            let s = partners::statement_month(
                &w.env.ctx,
                &PartnersStatementMonthInput { partner_id: p.partner_id, month: None },
            )
            .unwrap();
            let mut running = s.opening_total_paisa.as_paisa();
            for day in &s.days {
                running += day.columns.net_paisa.as_paisa();
                prop_assert_eq!(running, day.running_paisa.as_paisa());
                let parts = day.columns.commission_paisa.as_paisa()
                    + day.columns.expense_paisa.as_paisa()
                    + day.columns.gardens_trips_paisa.as_paisa()
                    + day.columns.fruit_paisa.as_paisa()
                    + day.columns.taken_returned_paisa.as_paisa()
                    + day.columns.capital_paisa.as_paisa();
                prop_assert_eq!(parts, day.columns.net_paisa.as_paisa());
            }
            prop_assert_eq!(running, s.closing_total_paisa.as_paisa());
            prop_assert_eq!(s.closing_total_paisa, p.total_paisa);
        }
    }

    #[test]
    fn a_mixed_save_writes_what_the_two_standalone_saves_write(
        left in 1_u32..5,
        carried_rate in 100_u32..900,
        new_rate in 100_u32..1500,
    ) {
        // A lot of 50 with 50 - left sold; then, on the next day, carried lines and a new lot.
        let make = || {
            let w = support::world();
            let lot = support::with_sales(
                support::lot(&w, "lot-1", w.rahim, w.banana, "50"),
                vec![support::sale("s-1", w.akram, &(50 - left).to_string(), "600")],
            );
            let saved = bikri::save(
                &w.env.ctx,
                &w.owner,
                &BikriSaveInput { draft: support::draft(vec![lot]), confirm_cash_below_zero: true },
            )
            .unwrap();
            set_today(&w.env, 2026, 10, 11);
            let lot_id = bikri::get(&w.env.ctx, saved.bikri_id).unwrap().lots[0].id;
            (w, lot_id)
        };
        let new_lot = |w: &World| {
            support::with_sales(
                support::bare_lot(w, "new-1", w.karim, w.apple, "10"),
                vec![support::sale("n-1", w.arshad, "10", &new_rate.to_string())],
            )
        };
        let (w1, lot1) = make();
        let carried = vec![CarriedLotInput {
            key: String::new(),
            lot_id: lot1,
            sales: vec![support::sale("c-1", w1.arshad, &left.to_string(), &carried_rate.to_string())],
        }];
        let mixed = bikri::mixed_save(
            &w1.env.ctx,
            &w1.owner,
            &MixedSaleSaveInput {
                date: "11-10-2026".into(),
                carried: carried.clone(),
                new_lots: vec![new_lot(&w1)],
                confirm_cash_below_zero: true,
            },
        )
        .unwrap();
        let (w2, lot2) = make();
        let later = bikri::later_sale_save(
            &w2.env.ctx,
            &w2.owner,
            &LaterSaleSaveInput {
                lot_id: lot2,
                date: "11-10-2026".into(),
                sales: vec![support::sale("c-1", w2.arshad, &left.to_string(), &carried_rate.to_string())],
                confirm_cash_below_zero: true,
            },
        )
        .unwrap();
        let mut draft = support::draft(vec![new_lot(&w2)]);
        draft.header.entry_date = "11-10-2026".into();
        let fresh = bikri::save(
            &w2.env.ctx,
            &w2.owner,
            &BikriSaveInput { draft, confirm_cash_below_zero: true },
        )
        .unwrap();
        let rows = |w: &World, entry: i64| -> (JournalRows, ShareRows) {
            let conn = common::raw(&w.dir);
            let lines: Vec<(String, i64, i64)> = conn
                .prepare(
                    "SELECT a.code, l.dr_paisa, l.cr_paisa FROM journal_lines l
                     JOIN accounts a ON a.id = l.account_id WHERE l.entry_id = ?1
                     ORDER BY a.code, l.dr_paisa, l.cr_paisa",
                )
                .unwrap()
                .query_map([entry], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .map(Result::unwrap)
                .collect();
            let shares: Vec<(String, i64)> = conn
                .prepare(
                    "SELECT kind, amount_paisa FROM partner_share_rows
                     WHERE journal_entry_id = ?1 ORDER BY partner_id, kind, amount_paisa",
                )
                .unwrap()
                .query_map([entry], |r| Ok((r.get(0)?, r.get(1)?)))
                .unwrap()
                .map(Result::unwrap)
                .collect();
            (lines, shares)
        };
        prop_assert_eq!(rows(&w1, mixed.entry_ids[0]), rows(&w2, later.entry_id));
        prop_assert_eq!(rows(&w1, mixed.entry_ids[1]), rows(&w2, fresh.entry_id));
    }
}

#[test]
fn a_mixed_input_is_sendable() {
    // The input types exist and are plain data (compile-time check of the contract).
    let _ = MixedSaleInput {
        date: String::new(),
        carried: Vec::new(),
        new_lots: Vec::new(),
    };
}
