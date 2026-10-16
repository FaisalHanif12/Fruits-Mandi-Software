//! Random drafts through the real database (Code Standards 9.3): the preview is repeatable, its
//! buyer bills add up, and its split adds back to the shared income.

use proptest::prelude::*;

use crate::support::*;

fn arb_rows() -> impl Strategy<Value = Vec<(u32, u32, bool)>> {
    prop::collection::vec((1u32..=60, 1u32..=3_000, any::<bool>()), 1..=5)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn the_preview_is_repeatable_and_adds_up(rows in arb_rows(), cash in prop::option::of(0u32..=30_000)) {
        let w = world();
        let sales = rows
            .iter()
            .enumerate()
            .map(|(i, (qty, rate, second))| {
                sale(&format!("s-{i}"), if *second { w.arshad } else { w.akram }, &qty.to_string(), &rate.to_string())
            })
            .collect();
        let mut one = lot(&w, "lot-1", w.rahim, w.banana, "300");
        one.cash_paid = cash.map(|c| c.to_string());
        let d = draft(vec![with_sales(one, sales)]);
        let first = preview(&w, &d);
        prop_assert_eq!(&first, &preview(&w, &d));
        prop_assert!(first.errors.is_empty(), "{:?}", first.errors);
        let lines: Vec<_> = first.lots[0].lines.iter().filter_map(|l| l.calc.as_ref()).collect();
        // The buyer bills add up to the summary.
        let bills = lines.iter().fold(0_i64, |s, l| s + l.buyer_total.as_paisa());
        prop_assert_eq!(bills, first.summary.buyer_total.as_paisa());
        // The lot commission is the sum of the stored line commissions.
        let comm = lines.iter().fold(0_i64, |s, l| s + l.seller_comm.amount.as_paisa());
        prop_assert_eq!(comm, first.summary.seller_comm.as_paisa());
        // The split adds back to the shared income, whatever the leftover.
        let split = first.split.iter().fold(0_i64, |s, p| s + p.income_paisa.as_paisa());
        prop_assert_eq!(split, first.summary.shared_income.as_paisa());
        // Cash after = before + credit-free cash lines - نقد.
        let change = first.summary.buyer_cash.as_paisa() - first.summary.cash_paid.as_paisa();
        prop_assert_eq!(first.cash.change_paisa.as_paisa(), change);
        prop_assert_eq!(first.cash.cash_after_paisa.as_paisa(), first.cash.cash_before_paisa.as_paisa() + change);
    }
}
