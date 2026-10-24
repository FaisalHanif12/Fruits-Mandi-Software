//! The reports keep small copies of domain workings (`mandi_reports::reports::workings`):
//! oldest-first matching, the totals of a seller's payable and the open amounts of money lent
//! (with the days past a return-by day). `mandi-reports` may not depend on
//! `mandi-domain` (Architecture 3), so the copies are pure functions of their own, and this test
//! runs both versions on random inputs and requires the same answer, errors included (money agent
//! Section 9, Code Standards G6). If the domain working ever changes, this fails first.
#![cfg(test)]

use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::money::{MoneyError, Paisa};
use mandi_domain::expenses as domain_expenses;
use mandi_domain::matching as domain_matching;
use mandi_domain::payables as domain_payables;
use mandi_reports::reports::workings as copy;
use proptest::prelude::*;

/// A whole-rupee amount in paisa, from -2 to 5,000 rupees (a negative one must be refused by
/// both).
fn amount() -> impl Strategy<Value = Paisa> {
    (-2_i64..=5_000).prop_map(|rupees| Paisa::from_rupees(rupees).unwrap())
}

/// A non-negative amount.
fn open_amount() -> impl Strategy<Value = Paisa> {
    (0_i64..=5_000).prop_map(|rupees| Paisa::from_rupees(rupees).unwrap())
}

fn lines(amounts: Vec<Paisa>, first_id: i64) -> Vec<(i64, Paisa)> {
    amounts
        .into_iter()
        .zip(first_id..)
        .map(|(a, id)| (id, a))
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn the_copy_of_match_bills_gives_the_domain_answer(
        bills in proptest::collection::vec(open_amount(), 0..8),
        credits in proptest::collection::vec(open_amount(), 0..8),
    ) {
        let bills = lines(bills, 1);
        let credits = lines(credits, 100);
        let domain = domain_matching::match_bills(
            &bills.iter().map(|(id, open)| domain_matching::Open { line_id: *id, open: *open }).collect::<Vec<_>>(),
            &credits.iter().map(|(id, open)| domain_matching::Open { line_id: *id, open: *open }).collect::<Vec<_>>(),
        )
        .unwrap();
        let ours = copy::match_bills(
            &bills.iter().map(|(id, open)| copy::Open { line_id: *id, open: *open }).collect::<Vec<_>>(),
            &credits.iter().map(|(id, open)| copy::Open { line_id: *id, open: *open }).collect::<Vec<_>>(),
        )
        .unwrap();
        let flat = |rows: Vec<(i64, i64, Paisa)>| rows;
        prop_assert_eq!(
            flat(domain.iter().map(|a| (a.bill_line_id, a.credit_line_id, a.amount)).collect()),
            flat(ours.iter().map(|a| (a.bill_line_id, a.credit_line_id, a.amount)).collect())
        );
        // Neither settles more than a credit has, nor more than a bill holds.
        let used: i64 = ours.iter().map(|a| a.amount.rupees()).sum();
        let bills_total: i64 = bills.iter().map(|(_, a)| a.rupees()).sum();
        let credits_total: i64 = credits.iter().map(|(_, a)| a.rupees()).sum();
        prop_assert_eq!(used, bills_total.min(credits_total));
    }

    #[test]
    fn both_refuse_a_negative_open_amount_alike(
        bills in proptest::collection::vec(amount(), 0..5),
        credits in proptest::collection::vec(amount(), 0..5),
    ) {
        // Bills and credits both come from the generator that can give a negative amount, so a
        // negative credit is compared too (finding 5).
        let bills = lines(bills, 1);
        let credits = lines(credits, 100);
        let domain = domain_matching::match_bills(
            &bills.iter().map(|(id, open)| domain_matching::Open { line_id: *id, open: *open }).collect::<Vec<_>>(),
            &credits.iter().map(|(id, open)| domain_matching::Open { line_id: *id, open: *open }).collect::<Vec<_>>(),
        );
        let ours = copy::match_bills(
            &bills.iter().map(|(id, open)| copy::Open { line_id: *id, open: *open }).collect::<Vec<_>>(),
            &credits.iter().map(|(id, open)| copy::Open { line_id: *id, open: *open }).collect::<Vec<_>>(),
        );
        prop_assert_eq!(domain.is_err(), ours.is_err());
        match (domain, ours) {
            (Err(d), Err(o)) => prop_assert_eq!(d, o),
            (Ok(d), Ok(o)) => prop_assert_eq!(
                d.iter().map(|a| (a.bill_line_id, a.credit_line_id, a.amount)).collect::<Vec<_>>(),
                o.iter().map(|a| (a.bill_line_id, a.credit_line_id, a.amount)).collect::<Vec<_>>()
            ),
            (d, o) => prop_assert!(false, "different answers: {:?} and {:?}", d, o),
        }
        let any_negative = bills.iter().chain(credits.iter()).any(|(_, a)| a.is_negative());
        if any_negative {
            // Both must refuse, with the same error.
            let again = copy::match_bills(
                &bills.iter().map(|(id, open)| copy::Open { line_id: *id, open: *open }).collect::<Vec<_>>(),
                &credits.iter().map(|(id, open)| copy::Open { line_id: *id, open: *open }).collect::<Vec<_>>(),
            );
            prop_assert_eq!(again.unwrap_err(), MoneyError::NegativeValue);
        }
    }

    #[test]
    fn the_copy_of_the_payable_totals_gives_the_domain_answer(
        credit in amount(),
        payments in amount(),
        given in amount(),
        on_credits in amount(),
        on_advances in amount(),
    ) {
        let domain = domain_payables::from_totals(credit, payments, given, on_credits, on_advances);
        let ours = copy::payable_totals_from(credit, payments, given, on_credits, on_advances);
        match (domain, ours) {
            (Ok(d), Ok(o)) => {
                prop_assert_eq!(d.open, o.open);
                prop_assert_eq!(d.advances_open, o.advances_open);
                prop_assert_eq!(d.excess_paid, o.excess_paid);
                // Credit left less payments is always P - E (money agent 8.2a): one of the two is 0.
                let credit_left = credit.checked_sub(on_credits).unwrap();
                let net = credit_left.checked_sub(payments).unwrap();
                prop_assert_eq!(net, o.open.checked_sub(o.excess_paid).unwrap());
                prop_assert!(o.open == Paisa::ZERO || o.excess_paid == Paisa::ZERO);
            }
            (Err(d), Err(o)) => prop_assert_eq!(d, o),
            (d, o) => prop_assert!(false, "different answers: {:?} and {:?}", d, o),
        }
    }

    #[test]
    fn the_copy_of_open_amounts_gives_the_domain_answer(
        bills in proptest::collection::vec(amount(), 0..8),
        credits in proptest::collection::vec(amount(), 0..8),
    ) {
        let bills = lines(bills, 1);
        let credits = lines(credits, 100);
        let domain = domain_expenses::open_amounts(&bills, &credits);
        let ours = copy::open_amounts(&bills, &credits);
        match (domain, ours) {
            (Ok(d), Ok(o)) => {
                prop_assert_eq!(d.len(), o.len());
                for (d, o) in d.iter().zip(&o) {
                    prop_assert_eq!((d.line_id, d.returned, d.open), (o.line_id, o.returned, o.open));
                }
                // Σ open is the bills less the credits, never below 0 (money agent 8.2b).
                let open = o.iter().try_fold(Paisa::ZERO, |sum, x| sum.checked_add(x.open)).unwrap();
                let owed = bills.iter().try_fold(Paisa::ZERO, |sum, x| sum.checked_add(x.1)).unwrap();
                let paid = credits.iter().try_fold(Paisa::ZERO, |sum, x| sum.checked_add(x.1)).unwrap();
                let want = owed.checked_sub(paid).unwrap();
                prop_assert_eq!(open, if want.is_negative() { Paisa::ZERO } else { want });
            }
            (Err(d), Err(o)) => prop_assert_eq!(d, o),
            (d, o) => prop_assert!(false, "different answers: {:?} and {:?}", d, o),
        }
    }

    #[test]
    fn the_copy_of_the_days_past_gives_the_domain_answer(
        open in amount(),
        return_by in proptest::option::of(0_u32..60),
        today in 0_u32..60,
    ) {
        let base = BusinessDate::from_ymd(2026, 9, 1).unwrap();
        let day = |n: u32| (0..n).fold(base, |d, _| d.next_day().unwrap());
        let return_by = return_by.map(day);
        let today = day(today);
        prop_assert_eq!(
            domain_expenses::days_past_return_by(open, return_by, today),
            copy::days_past_return_by(open, return_by, today)
        );
    }
}
