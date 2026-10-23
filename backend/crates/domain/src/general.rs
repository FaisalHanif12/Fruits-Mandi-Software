//! جنرل as pure rules (PRD 16.4, FR-6.2, plan 17): the lines of a free journal entry, their
//! totals and the entry they make. It is saved only when Σ بنام = Σ جمع.
//!
//! The preview and the save read the lines with the same service function and build the entry
//! here (Code Standards B5). Nothing here rounds; the totals are checked sums.
//!
//! ```
//! use mandi_domain::core::money::Paisa;
//! use mandi_domain::general::{GeneralLine, totals};
//! use mandi_domain::ledger::Side;
//! let p = |rupees| Paisa::from_rupees(rupees).unwrap();
//! let lines = vec![
//!     GeneralLine { account_id: 1, person_key: None, side: Side::Dr, amount: p(1_000), narration: None },
//!     GeneralLine { account_id: 2, person_key: None, side: Side::Cr, amount: p(900), narration: None },
//! ];
//! let (dr, cr) = totals(&lines).unwrap();
//! assert_eq!((dr, cr), (p(1_000), p(900)));
//! ```

use mandi_core::ids::AccountId;
use mandi_core::money::{MoneyError, Paisa};

use crate::ledger::{BalancedEntry, LedgerError, LedgerLine, Side, build_entry};

/// The fewest lines a جنرل entry can have (PRD 16.4).
pub const MIN_LINES: usize = 2;

/// One line of a جنرل entry that was read and checked: an account, a side and a positive amount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralLine {
    /// The account.
    pub account_id: i64,
    /// The one-time person of a پختہ کھاتہ account (PRD FR-13.5).
    pub person_key: Option<String>,
    /// The side.
    pub side: Side,
    /// More than 0.
    pub amount: Paisa,
    /// A note for the line.
    pub narration: Option<String>,
}

/// Σ بنام and Σ جمع of the lines.
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn totals(lines: &[GeneralLine]) -> Result<(Paisa, Paisa), MoneyError> {
    let (mut dr, mut cr) = (Paisa::ZERO, Paisa::ZERO);
    for line in lines {
        match line.side {
            Side::Dr => dr = dr.checked_add(line.amount)?,
            Side::Cr => cr = cr.checked_add(line.amount)?,
        }
    }
    Ok((dr, cr))
}

/// The journal entry of the lines, in the order typed. It is refused unless the totals are equal
/// (`LedgerError::Unbalanced`), so an unbalanced جنرل can never reach the journal (PRD FR-6.2).
///
/// # Errors
/// The errors of `build_entry`.
pub fn general_entry(lines: &[GeneralLine]) -> Result<BalancedEntry, LedgerError> {
    build_entry(
        lines
            .iter()
            .map(|line| {
                let mut built =
                    LedgerLine::new(AccountId::new(line.account_id), line.side, line.amount);
                built.person_key.clone_from(&line.person_key);
                built.narration.clone_from(&line.narration);
                built
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    fn line(account: i64, side: Side, amount: i64) -> GeneralLine {
        GeneralLine {
            account_id: account,
            person_key: None,
            side,
            amount: p(amount),
            narration: None,
        }
    }

    #[test]
    fn balanced_lines_make_an_entry_in_the_order_typed() {
        let lines = vec![
            line(1, Side::Dr, 700),
            line(2, Side::Dr, 300),
            line(3, Side::Cr, 1_000),
        ];
        assert_eq!(totals(&lines).unwrap(), (p(1_000), p(1_000)));
        let entry = general_entry(&lines).unwrap();
        let accounts: Vec<i64> = entry.lines().iter().map(|l| l.account_id.get()).collect();
        assert_eq!(accounts, vec![1, 2, 3]);
        assert_eq!(entry.total(), p(1_000));
    }

    #[test]
    fn unbalanced_lines_are_refused_with_both_totals() {
        let lines = vec![line(1, Side::Dr, 1_000), line(2, Side::Cr, 900)];
        let error = general_entry(&lines).unwrap_err();
        assert_eq!(
            error,
            LedgerError::Unbalanced {
                dr: p(1_000),
                cr: p(900)
            }
        );
    }

    #[test]
    fn a_person_key_and_a_note_travel_with_the_line() {
        let mut one = line(5, Side::Dr, 100);
        one.person_key = Some("shahid|0300".to_owned());
        one.narration = Some("correction".to_owned());
        let entry = general_entry(&[one, line(6, Side::Cr, 100)]).unwrap();
        assert_eq!(entry.lines()[0].person_key.as_deref(), Some("shahid|0300"));
        assert_eq!(entry.lines()[0].narration.as_deref(), Some("correction"));
    }
}
