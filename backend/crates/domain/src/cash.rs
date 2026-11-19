//! Cash in hand after an entry (PRD FR-6.7, plan 12): the one rule every cash-out screen shares.
//!
//! The entry asks the user to confirm only when cash ends BELOW 0 AND the entry takes cash out.
//! Cash that is already below 0 does not ask again for an entry that does not lower it, and an
//! entry that raises cash but still leaves it below 0 does not ask either (money agent review of
//! plan 12, point 6).
//!
//! The change comes from the journal entry itself: Σ debit − Σ credit of its lines on the cash
//! account. The preview and the save of every module read it from the same place (Code Standards
//! B5), so no screen has a second formula.
//!
//! ```
//! use mandi_domain::cash::check;
//! use mandi_domain::core::money::Paisa;
//! let p = |rupees| Paisa::from_rupees(rupees).unwrap();
//! // PRD FR-6.7: cash 20,000 and a lot with نقد 25,000 leaves -5,000, so it asks.
//! let asks = check(p(20_000), p(-25_000)).unwrap();
//! assert_eq!(asks.cash_after_paisa, p(-5_000));
//! assert!(asks.needs_confirm);
//! // Cash already at -3,000 and a cash sale of 1,000 leaves -2,000: it does not lower cash.
//! assert!(!check(p(-3_000), p(1_000)).unwrap().needs_confirm);
//! ```

use mandi_core::cash::CashCheck;
use mandi_core::ids::AccountId;
use mandi_core::money::{MoneyError, Paisa};

use crate::ledger::{BalancedEntry, Side};

/// What an entry does to the cash account: Σ debit − Σ credit over its lines on `cash`. A
/// positive result puts cash in, a negative one takes cash out.
///
/// # Errors
/// `MoneyError::Overflow` if a sum does not fit.
pub fn net_change(entry: &BalancedEntry, cash: AccountId) -> Result<Paisa, MoneyError> {
    entry
        .lines()
        .iter()
        .filter(|line| line.account_id == cash)
        .try_fold(Paisa::ZERO, |sum, line| match line.side {
            Side::Dr => sum.checked_add(line.amount),
            Side::Cr => sum.checked_sub(line.amount),
        })
}

/// Cash after an entry, and whether the user must confirm it (PRD FR-6.7). `before` is cash in
/// hand now (debit positive); `change` is what the entry adds to it (negative when it takes cash
/// out).
///
/// # Errors
/// `MoneyError::Overflow` if the sum does not fit.
pub fn check(before: Paisa, change: Paisa) -> Result<CashCheck, MoneyError> {
    let after = before.checked_add(change)?;
    let below_zero = after.is_negative();
    let lowers_cash = change.is_negative();
    Ok(CashCheck {
        cash_before_paisa: before,
        change_paisa: change,
        cash_after_paisa: after,
        below_zero,
        lowers_cash,
        needs_confirm: below_zero && lowers_cash,
    })
}

/// The check of an entry that cannot lower cash (its `change` is 0 or more), without reading cash
/// in hand (PRD FR-6.7). Such an entry never needs the user to confirm, whatever the balance is
/// (`needs_confirm` is `below_zero && lowers_cash`), so the balance is not needed: the answer
/// has `cash_before_paisa` and `cash_after_paisa` of 0 and everything else false. A preview uses
/// it to skip the sum over the cash account, which has hundreds of thousands of lines.
///
/// ```
/// use mandi_domain::core::money::Paisa;
/// use mandi_domain::cash::check_unread;
/// let c = check_unread(Paisa::from_rupees(10_000).unwrap()).unwrap();
/// assert!(!c.needs_confirm && !c.lowers_cash);
/// ```
///
/// # Errors
/// `MoneyError::NegativeValue` if `change` is below 0: an entry that lowers cash needs
/// [`check`] with the real balance.
pub fn check_unread(change: Paisa) -> Result<CashCheck, MoneyError> {
    if change.is_negative() {
        return Err(MoneyError::NegativeValue);
    }
    Ok(CashCheck {
        cash_before_paisa: Paisa::ZERO,
        change_paisa: change,
        cash_after_paisa: Paisa::ZERO,
        below_zero: false,
        lowers_cash: false,
        needs_confirm: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{LedgerLine, build_entry};

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    #[test]
    fn an_entry_that_cannot_lower_cash_never_asks_whatever_cash_is() {
        // check_unread says what check says about the question, without the balance.
        for before in [-50_000, -1, 0, 1, 20_000] {
            for change in [0, 1, 25_000] {
                let read = check(p(before), p(change)).unwrap();
                let unread = check_unread(p(change)).unwrap();
                assert!(!read.needs_confirm);
                assert_eq!(unread.needs_confirm, read.needs_confirm);
                assert_eq!(unread.lowers_cash, read.lowers_cash);
                assert_eq!(unread.change_paisa, read.change_paisa);
            }
        }
        assert_eq!(check_unread(p(-1)), Err(MoneyError::NegativeValue));
    }

    #[test]
    fn plan_12_cash_cases() {
        // PRD FR-6.7: 20,000 and نقد 25,000.
        let c = check(p(20_000), p(-25_000)).unwrap();
        assert_eq!(c.cash_after_paisa, p(-5_000));
        assert!(c.below_zero && c.lowers_cash && c.needs_confirm);
        // Money agent cases: 5,000 - 5,000 = 0 no; 5,000 - 5,001 = -1 yes.
        assert!(!check(p(5_000), p(-5_000)).unwrap().needs_confirm);
        assert!(check(p(5_000), p(-5_001)).unwrap().needs_confirm);
        // Already below 0 and the entry takes more out: asks.
        assert!(check(p(-3_000), p(-100)).unwrap().needs_confirm);
        // Already below 0 and the entry puts cash in: does not ask.
        let c = check(p(-3_000), p(1_000)).unwrap();
        assert_eq!(c.cash_after_paisa, p(-2_000));
        assert!(c.below_zero && !c.lowers_cash && !c.needs_confirm);
        // Event 2 (+12,800) with نقد 5,000: +7,800, no ask.
        assert!(!check(p(0), p(7_800)).unwrap().needs_confirm);
        // An entry that does not touch cash never asks.
        assert!(!check(p(-1), p(0)).unwrap().needs_confirm);
    }

    #[test]
    fn overflow_is_an_error() {
        let max = Paisa::try_from_paisa(i64::MAX - i64::MAX % 100).unwrap();
        assert_eq!(check(max, p(1)).unwrap_err(), MoneyError::Overflow);
    }

    #[test]
    fn the_change_is_read_from_the_cash_lines_of_the_entry() {
        let cash = AccountId::new(1);
        let other = AccountId::new(2);
        let entry = build_entry(vec![
            LedgerLine::new(other, Side::Dr, p(12_880)),
            LedgerLine::new(other, Side::Cr, p(12_880 - 5_000)),
            LedgerLine::new(cash, Side::Cr, p(5_000)),
        ])
        .unwrap();
        assert_eq!(net_change(&entry, cash).unwrap(), p(-5_000));
        assert_eq!(net_change(&entry, AccountId::new(9)).unwrap(), Paisa::ZERO);
    }
}
