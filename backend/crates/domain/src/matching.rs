//! Oldest-first matching of credits to bills (PRD FR-15.4, FR-15.5, FR-15.8, money agent
//! Section 8). One pure function does all of it:
//!
//! - Bills and receipts: a buyer's bills are his debit lines; a receipt (or any credit line)
//!   settles the oldest open bills first. A credit that is more than all open bills stays
//!   unmatched and settles his next bills (PRD FR-15.5).
//! - Advances: a seller's open advances are the "bills" and the seller credit line of a بکری
//!   posting is the "credit" (PRD FR-15.8).
//! - What a seller is still owed (PRD FR-7.59) is the same working with his credit lines as the
//!   "bills" and his debit lines that are not advances as the "credits".
//!
//! The caller lists the open amounts, oldest first by `(entry_date, entry id, line_no)`. Nothing
//! here touches the database; the services read the lines and write the rows.
//!
//! ```
//! use mandi_domain::core::money::Paisa;
//! use mandi_domain::matching::{Open, match_bills};
//! let p = |rupees| Paisa::from_rupees(rupees).unwrap();
//! // Money agent 8.1: bills 12,880 and 5,000; a receipt of 15,000 settles 12,880 and 2,120.
//! let bills = [Open { line_id: 1, open: p(12_880) }, Open { line_id: 2, open: p(5_000) }];
//! let credits = [Open { line_id: 9, open: p(15_000) }];
//! let rows = match_bills(&bills, &credits).unwrap();
//! assert_eq!((rows[0].bill_line_id, rows[0].amount), (1, p(12_880)));
//! assert_eq!((rows[1].bill_line_id, rows[1].amount), (2, p(2_120)));
//! ```

use mandi_core::money::{MoneyError, Paisa};

/// A line and the part of it that is still open, in whole rupees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Open {
    /// The journal line.
    pub line_id: i64,
    /// What is still open on it. Never negative; a line with nothing open is skipped.
    pub open: Paisa,
}

/// One matching row to write: `amount` of `credit_line_id` settles `bill_line_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Allocation {
    /// The bill (or advance) that is settled.
    pub bill_line_id: i64,
    /// The credit that settles it.
    pub credit_line_id: i64,
    /// The amount, more than 0.
    pub amount: Paisa,
}

/// Matches credits to bills, oldest first (PRD FR-15.4).
///
/// For each credit in order, the oldest bills are taken first, `min(credit left, bill open)` at
/// a time. A credit never settles more than it has, a bill never more than its open amount, and
/// what a credit has left after all bills are full stays unmatched. The total matched is
/// `min(Σ open bills, Σ open credits)`.
///
/// # Errors
/// `MoneyError::NegativeValue` for an open amount below 0 (a bug in the caller);
/// `MoneyError::Overflow` for a sum that does not fit.
pub fn match_bills(bills: &[Open], credits: &[Open]) -> Result<Vec<Allocation>, MoneyError> {
    if bills.iter().chain(credits).any(|l| l.open.is_negative()) {
        return Err(MoneyError::NegativeValue);
    }
    let mut left: Vec<(i64, Paisa)> = bills.iter().map(|b| (b.line_id, b.open)).collect();
    // `first` never goes back: every bill before it is full.
    let mut first = 0_usize;
    let mut rows = Vec::new();
    for credit in credits {
        let mut remaining = credit.open;
        while remaining > Paisa::ZERO {
            let Some(bill) = left.get_mut(first) else {
                break;
            };
            if bill.1 == Paisa::ZERO {
                first = first.saturating_add(1);
                continue;
            }
            let take = remaining.min(bill.1);
            rows.push(Allocation {
                bill_line_id: bill.0,
                credit_line_id: credit.line_id,
                amount: take,
            });
            bill.1 = bill.1.checked_sub(take)?;
            remaining = remaining.checked_sub(take)?;
        }
    }
    Ok(rows)
}

/// A line and the place it has in the order of its account: `(entry_date, entry id, line_no)`
/// turned into one number, the position in the list of all lines of the account, oldest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ranked {
    /// The line and what is open on it.
    pub line: Open,
    /// Its position in the order of the account's lines. A smaller number is older.
    pub rank: usize,
}

/// Matches seller credits to advances, oldest first, where a credit adjusts only the advances
/// that come BEFORE it in the order of the account (PRD FR-15.8: "each later seller credit"). An
/// advance given after a credit is never used by it, so the same books give the same answer
/// whether the credit was saved before or after, and whether the matching is made again.
///
/// Both lists are oldest first. A credit takes, in order, the open advances with a smaller rank
/// than its own.
///
/// ```
/// use mandi_domain::core::money::Paisa;
/// use mandi_domain::matching::{Open, Ranked, match_before};
/// let p = |rupees| Paisa::from_rupees(rupees).unwrap();
/// let ranked = |line_id, rupees, rank| Ranked { line: Open { line_id, open: p(rupees) }, rank };
/// // An advance of 15,000 at place 0, a credit of 24,316 at place 1: 15,000 is adjusted.
/// let rows = match_before(&[ranked(1, 15_000, 0)], &[ranked(9, 24_316, 1)]).unwrap();
/// assert_eq!(rows[0].amount, p(15_000));
/// // The credit came first (place 0) and the advance after it (place 1): nothing is adjusted.
/// assert!(match_before(&[ranked(1, 15_000, 1)], &[ranked(9, 24_316, 0)]).unwrap().is_empty());
/// ```
///
/// # Errors
/// As [`match_bills`].
pub fn match_before(bills: &[Ranked], credits: &[Ranked]) -> Result<Vec<Allocation>, MoneyError> {
    if bills
        .iter()
        .chain(credits)
        .any(|l| l.line.open.is_negative())
    {
        return Err(MoneyError::NegativeValue);
    }
    let mut left: Vec<(i64, Paisa, usize)> = bills
        .iter()
        .map(|b| (b.line.line_id, b.line.open, b.rank))
        .collect();
    // Every bill before `first` is full, so a credit never needs to look at it again.
    let mut first = 0_usize;
    let mut rows = Vec::new();
    for credit in credits {
        let mut remaining = credit.line.open;
        let mut at = first;
        while remaining > Paisa::ZERO {
            let Some(bill) = left.get_mut(at) else {
                break;
            };
            if bill.2 >= credit.rank {
                break;
            }
            if bill.1 == Paisa::ZERO {
                if at == first {
                    first = first.saturating_add(1);
                }
                at = at.saturating_add(1);
                continue;
            }
            let take = remaining.min(bill.1);
            rows.push(Allocation {
                bill_line_id: bill.0,
                credit_line_id: credit.line.line_id,
                amount: take,
            });
            bill.1 = bill.1.checked_sub(take)?;
            remaining = remaining.checked_sub(take)?;
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests;
