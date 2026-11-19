//! Partner capital as pure rules (PRD 17.4, FR-17.7, FR-17.8, plan 17): the journal entry of
//! money a partner puts in and takes back, and the warning for taking back more than his balance.
//!
//! The preview and the save call the same functions (Code Standards B5). Capital is never income
//! or expense and is never split, so nothing here rounds: every amount is added or taken away
//! with checked maths.
//!
//! ```
//! use mandi_domain::capital::capital_excess;
//! use mandi_domain::core::money::Paisa;
//! let p = |rupees| Paisa::from_rupees(rupees).unwrap();
//! // PRD E2E-35 and FR-17.8: ب has 50,000 of capital and takes back 60,000: 10,000 is over.
//! assert_eq!(capital_excess(p(50_000), p(60_000)).unwrap(), Some(p(10_000)));
//! assert_eq!(capital_excess(p(50_000), p(10_000)).unwrap(), None);
//! ```

use mandi_core::ids::AccountId;
use mandi_core::money::{MoneyError, Paisa};

use crate::expenses::two_lines;
use crate::ledger::{BalancedEntry, LedgerError};

/// Capital invested (PRD FR-17.7): cash بنام, the partner's capital account (0602) جمع.
///
/// # Errors
/// The errors of `build_entry`.
pub fn capital_in_entry(
    cash: AccountId,
    capital: AccountId,
    amount: Paisa,
) -> Result<BalancedEntry, LedgerError> {
    two_lines(cash, capital, amount)
}

/// Capital taken back (PRD FR-17.7): the capital account بنام, cash جمع.
///
/// # Errors
/// The errors of `build_entry`.
pub fn capital_out_entry(
    capital: AccountId,
    cash: AccountId,
    amount: Paisa,
) -> Result<BalancedEntry, LedgerError> {
    two_lines(capital, cash, amount)
}

/// The part of a take-back that is above the capital balance, if any (PRD FR-17.8). `None` when
/// the amount fits. `balance` is the credit balance of the capital account (below 0 when the
/// account stands on the debit side, which counts as nothing to take back).
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn capital_excess(balance: Paisa, amount: Paisa) -> Result<Option<Paisa>, MoneyError> {
    let available = balance.max(Paisa::ZERO);
    if amount > available {
        Ok(Some(amount.checked_sub(available)?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::Side;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    #[test]
    fn invested_puts_cash_in_and_credits_capital() {
        // PRD E2E-35: ب invests 50,000.
        let entry = capital_in_entry(AccountId::new(1), AccountId::new(2), p(50_000)).unwrap();
        let lines: Vec<_> = entry
            .lines()
            .iter()
            .map(|l| (l.account_id.get(), l.side, l.amount))
            .collect();
        assert_eq!(
            lines,
            vec![(1, Side::Dr, p(50_000)), (2, Side::Cr, p(50_000))]
        );
        assert_eq!(entry.total(), p(50_000));
    }

    #[test]
    fn taken_back_is_the_opposite() {
        let entry = capital_out_entry(AccountId::new(2), AccountId::new(1), p(10_000)).unwrap();
        let lines: Vec<_> = entry
            .lines()
            .iter()
            .map(|l| (l.account_id.get(), l.side))
            .collect();
        assert_eq!(lines, vec![(2, Side::Dr), (1, Side::Cr)]);
    }

    #[test]
    fn a_zero_amount_is_not_an_entry() {
        assert!(capital_in_entry(AccountId::new(1), AccountId::new(2), Paisa::ZERO).is_err());
    }

    #[test]
    fn taking_back_more_than_the_balance_is_over_by_the_difference() {
        assert_eq!(
            capital_excess(p(40_000), p(40_000)).unwrap(),
            None,
            "exactly the balance fits"
        );
        assert_eq!(capital_excess(p(40_000), p(40_100)).unwrap(), Some(p(100)));
        // A debit balance leaves nothing to take back.
        assert_eq!(capital_excess(p(-500), p(300)).unwrap(), Some(p(300)));
        assert_eq!(capital_excess(Paisa::ZERO, p(1)).unwrap(), Some(p(1)));
    }
}
