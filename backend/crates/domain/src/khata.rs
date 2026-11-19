//! الگ کھاتہ as pure rules (PRD 16.6, FR-16.6, plan 17): the journal entry of money paid out for a
//! separate khata and money received for it.
//!
//! A khata is never income or expense, never split and never in a partner's total (PRD C22), so
//! these entries touch only the khata's account (control 0305) and cash. Its balance is signed
//! from the khata's side: credit minus debit.
//!
//! ```
//! use mandi_domain::core::money::Paisa;
//! use mandi_domain::khata::signed_balance;
//! let p = |rupees| Paisa::from_rupees(rupees).unwrap();
//! // PRD E2E-38: paid out 10,000, then 4,000 back: -6,000.
//! assert_eq!(signed_balance(p(0), p(10_000)).unwrap(), p(-10_000));
//! assert_eq!(signed_balance(p(4_000), p(10_000)).unwrap(), p(-6_000));
//! ```

use mandi_core::ids::AccountId;
use mandi_core::money::{MoneyError, Paisa};

use crate::expenses::two_lines;
use crate::ledger::{BalancedEntry, LedgerError};

/// رقم دی (PRD FR-16.6): the khata بنام (minus), cash جمع.
///
/// # Errors
/// The errors of `build_entry`.
pub fn khata_out_entry(
    khata: AccountId,
    cash: AccountId,
    amount: Paisa,
) -> Result<BalancedEntry, LedgerError> {
    two_lines(khata, cash, amount)
}

/// رقم آئی (PRD FR-16.6): cash بنام, the khata جمع (plus).
///
/// # Errors
/// The errors of `build_entry`.
pub fn khata_in_entry(
    cash: AccountId,
    khata: AccountId,
    amount: Paisa,
) -> Result<BalancedEntry, LedgerError> {
    two_lines(cash, khata, amount)
}

/// A khata's balance from its side: credit less debit. Plus means the shop holds money for it;
/// minus means it has taken more than it brought (PRD FR-16.6, E54).
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn signed_balance(credit: Paisa, debit: Paisa) -> Result<Paisa, MoneyError> {
    credit.checked_sub(debit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::Side;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    #[test]
    fn paid_out_debits_the_khata_and_credits_cash() {
        let entry = khata_out_entry(AccountId::new(7), AccountId::new(1), p(10_000)).unwrap();
        let lines: Vec<_> = entry
            .lines()
            .iter()
            .map(|l| (l.account_id.get(), l.side))
            .collect();
        assert_eq!(lines, vec![(7, Side::Dr), (1, Side::Cr)]);
    }

    #[test]
    fn received_debits_cash_and_credits_the_khata() {
        let entry = khata_in_entry(AccountId::new(1), AccountId::new(7), p(4_000)).unwrap();
        let lines: Vec<_> = entry
            .lines()
            .iter()
            .map(|l| (l.account_id.get(), l.side))
            .collect();
        assert_eq!(lines, vec![(1, Side::Dr), (7, Side::Cr)]);
    }

    #[test]
    fn the_family_khata_of_e2e_38() {
        // -10,000, then +4,000 back, then the sale of 9,000: +3,000.
        let after_pay = signed_balance(p(0), p(10_000)).unwrap();
        let after_back = signed_balance(p(4_000), p(10_000)).unwrap();
        let after_sale = signed_balance(p(4_000 + 9_000), p(10_000)).unwrap();
        assert_eq!(
            (after_pay, after_back, after_sale),
            (p(-10_000), p(-6_000), p(3_000))
        );
    }
}
