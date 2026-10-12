//! Balances and openings as money (PRD C13, FR-10.9, Code Standards 9.3): a balance is never
//! negative and its side follows the sign of debit minus credit, for any totals.
#![cfg(test)]

use mandi_core::accounts::{Balance, BalanceSide, opening_request};
use mandi_core::dates::BusinessDate;
use mandi_core::error::ErrorCode;
use mandi_core::money::Paisa;
use proptest::prelude::*;

fn paisa(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

#[test]
fn the_worked_balances_of_the_prd() {
    // PRD C13: 12,880 debit and 10,000 credit leave 2,880 on the debit side.
    let b = Balance::from_totals(paisa(12_880), paisa(10_000)).unwrap();
    assert_eq!((b.side, b.amount_paisa), (BalanceSide::Dr, paisa(2_880)));
    // Paying more than owed flips the side with no signed value shown.
    let b = Balance::from_totals(paisa(12_880), paisa(15_000)).unwrap();
    assert_eq!((b.side, b.amount_paisa), (BalanceSide::Cr, paisa(2_120)));
    assert_eq!(
        Balance::from_totals(paisa(5), paisa(5)).unwrap(),
        Balance::ZERO
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// For any totals in the allowed range the amount is never negative, the side matches the
    /// sign, and the amount is the exact difference (no rounding anywhere).
    #[test]
    fn a_balance_is_the_exact_difference_with_its_side(
        dr in 0_i64..=1_000_000_000_000_i64,
        cr in 0_i64..=1_000_000_000_000_i64,
    ) {
        let b = Balance::from_totals(paisa(dr), paisa(cr)).unwrap();
        prop_assert!(!b.amount_paisa.is_negative());
        prop_assert_eq!(b.amount_paisa.as_paisa(), (dr - cr).abs() * 100);
        let want = match dr.cmp(&cr) {
            std::cmp::Ordering::Greater => BalanceSide::Dr,
            std::cmp::Ordering::Less => BalanceSide::Cr,
            std::cmp::Ordering::Equal => BalanceSide::Zero,
        };
        prop_assert_eq!(b.side, want);
        // The same balance from the signed net.
        prop_assert_eq!(Balance::from_net(paisa(dr - cr)).unwrap(), b);
    }

    /// An opening request never panics, is never both sides, and a seller-only advance flag is
    /// refused for anyone else.
    #[test]
    fn opening_requests_are_one_sided_and_whole_rupees(
        dr in "[0-9,.]{0,12}",
        cr in "[0-9,.]{0,12}",
        is_advance in any::<bool>(),
        is_seller in any::<bool>(),
    ) {
        let today = BusinessDate::parse_iso("2026-10-10").unwrap();
        match opening_request("", "31-08-2026", &dr, &cr, is_advance, is_seller, None, today) {
            Ok(found) => {
                prop_assert!(found.dr == Paisa::ZERO || found.cr == Paisa::ZERO);
                prop_assert_eq!(found.dr.as_paisa() % 100, 0);
                prop_assert_eq!(found.cr.as_paisa() % 100, 0);
                prop_assert!(!found.dr.is_negative() && !found.cr.is_negative());
                if found.is_advance {
                    prop_assert!(is_seller && found.dr != Paisa::ZERO);
                }
            }
            Err(errors) => {
                prop_assert!(!errors.is_empty());
                prop_assert!(errors.iter().all(|e| e.code != ErrorCode::ErrInternal));
            }
        }
    }
}
