//! Random بکری through the real database (Code Standards 9.3, plan 13): for any draft that the
//! preview accepts, the save writes exactly what the preview showed, the journal balances, the
//! share rows add up to the shared income, the buyer bills add up to the buyer debits, the stock
//! adds up, and every stored amount is a whole rupee.

use mandi_services::bikri;
use mandi_services::core::bikri::{BikriDraft, LotInput, PaymentType, SaleInput};
use mandi_services::ledger;
use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};

use crate::common::raw;
use crate::helpers::*;
use crate::support::*;

#[derive(Debug, Clone)]
struct SaleSpec {
    /// 0 اکرم, 1 ارشد, 2 نقدی منڈی, 3 a one-time buyer.
    buyer: u8,
    /// How much of what is left of the lot this line takes, in percent.
    take_percent: u8,
    rate: u16,
    /// A buyer rate that differs from the seller rate, if any.
    buyer_rate: Option<u16>,
    cash: bool,
}

#[derive(Debug, Clone)]
struct LotSpec {
    seller: u8,
    item: u8,
    qty: u16,
    freight: u16,
    labour_per_unit: u8,
    cash_paid: u16,
    tick: bool,
    sales: Vec<SaleSpec>,
}

fn sale_spec() -> impl Strategy<Value = SaleSpec> {
    (
        0_u8..4,
        1_u8..=100,
        50_u16..3_000,
        prop::option::of(50_u16..3_000),
        any::<bool>(),
    )
        .prop_map(|(buyer, take_percent, rate, buyer_rate, cash)| SaleSpec {
            buyer,
            take_percent,
            rate,
            buyer_rate,
            cash,
        })
}

fn lot_spec() -> impl Strategy<Value = LotSpec> {
    (
        0_u8..2,
        0_u8..3,
        1_u16..80,
        0_u16..3_000,
        0_u8..10,
        0_u16..5_000,
        any::<bool>(),
        prop::collection::vec(sale_spec(), 0..4),
    )
        .prop_map(
            |(seller, item, qty, freight, labour_per_unit, cash_paid, tick, sales)| LotSpec {
                seller,
                item,
                qty,
                freight,
                labour_per_unit,
                cash_paid,
                tick,
                sales,
            },
        )
}

fn build(w: &World, specs: &[LotSpec]) -> BikriDraft {
    let people = [
        ("شاہد", "Shahid", "0300-0000000"),
        ("ناصر", "Nasir", "0300-1111111"),
    ];
    let mut lots: Vec<LotInput> = Vec::new();
    for (index, spec) in specs.iter().enumerate() {
        let seller = [w.rahim, w.karim][usize::from(spec.seller)];
        let item = [w.banana, w.apple, w.mooli][usize::from(spec.item)];
        let mut l = lot(
            w,
            &format!("lot-{index}"),
            seller,
            item,
            &spec.qty.to_string(),
        );
        l.freight = Some(spec.freight.to_string());
        l.labour = per_unit(&spec.labour_per_unit.to_string());
        l.cash_paid = (spec.cash_paid > 0).then(|| spec.cash_paid.to_string());
        l.adjust_advance = spec.tick;
        let mut left = i64::from(spec.qty);
        for (j, s) in spec.sales.iter().enumerate() {
            if left == 0 {
                break;
            }
            let qty = (left * i64::from(s.take_percent) / 100).max(1);
            left -= qty;
            let buyer = [w.akram, w.arshad, w.naqdi, w.pukhta_buyer][usize::from(s.buyer)];
            let mut line: SaleInput = sale(
                &format!("s-{index}-{j}"),
                buyer,
                &qty.to_string(),
                &s.rate.to_string(),
            );
            if s.buyer == 3 {
                line.pukhta = person(people[j % 2].0, people[j % 2].1, people[j % 2].2);
            }
            if s.buyer < 2 && s.cash {
                line.payment_type = Some(PaymentType::Cash);
            }
            if let Some(rate) = s.buyer_rate {
                line.rate_differs = true;
                line.buyer_rate = Some(rate.to_string());
            }
            l.sales.push(line);
        }
        lots.push(l);
    }
    draft(lots)
}

fn sum(w: &World, sql: &str, entry_id: i64) -> i64 {
    raw(&w.dir)
        .query_row(sql, [entry_id], |r| r.get(0))
        .unwrap()
}

#[test]
fn random_bikri_are_saved_exactly_as_previewed_and_the_books_balance() {
    let w = world();
    // An advance for each seller so the tick has something to adjust.
    give_advance(&w, w.rahim, "8,000");
    give_advance(&w, w.karim, "3,000");
    let cash = id_of(&w.env, "01010001");
    let saved_count = std::cell::Cell::new(0_u32);
    let mut runner = TestRunner::new(Config {
        cases: 60,
        failure_persistence: None,
        ..Config::default()
    });
    let strategy = prop::collection::vec(lot_spec(), 1..3);
    runner
        .run(&strategy, |specs| {
            let d = build(&w, &specs);
            let before = bikri::preview(&w.env.ctx, &d).unwrap();
            if !before.errors.is_empty() {
                return Ok(());
            }
            let cash_before = ledger::account_balance(&w.env.ctx, cash).unwrap();
            let saved = save(&w, &d).unwrap();
            saved_count.set(saved_count.get() + 1);
            // The save writes what the preview showed.
            prop_assert_eq!(&saved.preview, &before);
            // The entry balances (the database also checks it) and the sale part's buyer debits are
            // the buyer bills.
            let dr = sum(&w, "SELECT COALESCE(SUM(dr_paisa), 0) FROM journal_lines WHERE entry_id = ?1", saved.entry_id);
            let cr = sum(&w, "SELECT COALESCE(SUM(cr_paisa), 0) FROM journal_lines WHERE entry_id = ?1", saved.entry_id);
            prop_assert_eq!(dr, cr);
            let buyer_debits = sum(
                &w,
                "SELECT COALESCE(SUM(l.dr_paisa), 0) FROM journal_lines l JOIN accounts a ON a.id = l.account_id
                 WHERE l.entry_id = ?1 AND a.party_type = 'buyer'",
                saved.entry_id,
            );
            prop_assert_eq!(buyer_debits, before.summary.buyer_total.as_paisa());
            // The share rows add up to the shared income.
            let shares = sum(&w, "SELECT COALESCE(SUM(amount_paisa), 0) FROM partner_share_rows WHERE journal_entry_id = ?1", saved.entry_id);
            prop_assert_eq!(shares, before.summary.shared_income.as_paisa());
            // The stored lots and lines are the previewed ones.
            let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
            // The stored totals of the whole بکری and of each lot are the previewed ones.
            assert_totals_equal(&detail.totals, &before.summary, "the whole بکری");
            for (stored, shown) in detail.lots.iter().zip(&before.lots) {
                assert_totals_equal(&stored.totals, shown.totals.as_ref().unwrap(), "a lot");
            }
            for (stored, shown) in detail.lots.iter().zip(&before.lots) {
                let totals = shown.totals.as_ref().unwrap();
                prop_assert_eq!(stored.gross, totals.gross);
                prop_assert_eq!(stored.deductions, totals.deductions);
                prop_assert_eq!(stored.net, totals.net);
                let shown_lines: Vec<_> = shown.lines.iter().filter_map(|l| l.calc.as_ref()).collect();
                prop_assert_eq!(stored.sales.len(), shown_lines.len());
                for (s, c) in stored.sales.iter().zip(shown_lines) {
                    prop_assert_eq!(s.buyer_total, c.buyer_total);
                    prop_assert_eq!(s.seller_comm.amount, c.seller_comm.amount);
                    prop_assert_eq!(s.rate_diff, c.rate_diff);
                }
            }
            // Cash moved by exactly what the preview said.
            let cash_after = ledger::account_balance(&w.env.ctx, cash).unwrap();
            let signed = |b: mandi_services::core::accounts::Balance| match b.side {
                mandi_services::core::accounts::BalanceSide::Cr => -b.amount_paisa.as_paisa(),
                _ => b.amount_paisa.as_paisa(),
            };
            prop_assert_eq!(signed(cash_after) - signed(cash_before), before.cash.change_paisa.as_paisa());
            // Stock of each lot of this بکری adds up.
            let conn = raw(&w.dir);
            let bad_stock: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM bikri_lots l WHERE l.bikri_id = ?1 AND
                       (SELECT COALESCE(SUM(qty), 0) FROM stock_moves WHERE lot_id = l.id)
                       <> l.qty_received - l.qty_sold - l.qty_written_off",
                    [saved.bikri_id],
                    |r| r.get(0),
                )
                .unwrap();
            prop_assert_eq!(bad_stock, 0);
            // The books balance and every stored amount is a whole rupee.
            let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
            prop_assert_eq!(trial.total_dr, trial.total_cr);
            let odd: i64 = conn
                .query_row(
                    "SELECT (SELECT COUNT(*) FROM journal_lines WHERE dr_paisa % 100 <> 0 OR cr_paisa % 100 <> 0)
                          + (SELECT COUNT(*) FROM bikri_sales WHERE buyer_total_paisa % 100 <> 0 OR seller_comm_paisa % 100 <> 0)
                          + (SELECT COUNT(*) FROM bikri_lots WHERE net_paisa % 100 <> 0 OR deductions_paisa % 100 <> 0)",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            prop_assert_eq!(odd, 0);
            Ok(())
        })
        .unwrap();
    // Not every random draft is valid, but most are.
    let saved_count = saved_count.get();
    assert!(saved_count >= 15, "only {saved_count} drafts were saved");
    crate::common::assert_chain_ok(&w.dir);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}
