//! The read-time workings the Collections, Sales and Business reports need: oldest-first matching
//! (PRD FR-15.4, money agent 8.1), the totals of a seller's payable (PRD FR-7.59, money agent
//! 8.2a) and the open amounts of money lent (PRD FR-16.5, money agent 8.2b).
//!
//! These are copies of `mandi_domain::matching::match_bills`,
//! `mandi_domain::payables::from_totals`, `mandi_domain::expenses::open_amounts` and
//! `mandi_domain::expenses::days_past_return_by`, kept here because `mandi-reports` may depend only on
//! `mandi-db` and `mandi-core` (Architecture 3, `workspace_rules.rs`). They are pure, they write
//! nothing, and they are never used to post anything: the services keep the working that decides
//! what is saved. A parity test (`services/tests/reports_parity.rs`) runs both versions on
//! thousands of random inputs and requires the same answer, so the copy cannot drift.

use mandi_core::dates::BusinessDate;
use mandi_core::money::{MoneyError, Paisa};

/// A line and the part of it that is still open, in whole rupees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Open {
    /// The journal line.
    pub line_id: i64,
    /// What is still open on it. Never negative; a line with nothing open is skipped.
    pub open: Paisa,
}

/// One matching row: `amount` of `credit_line_id` settles `bill_line_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Allocation {
    /// The bill that is settled.
    pub bill_line_id: i64,
    /// The credit that settles it.
    pub credit_line_id: i64,
    /// The amount, more than 0.
    pub amount: Paisa,
}

/// Matches credits to bills, oldest first (PRD FR-15.4).
///
/// For each credit in order, the oldest bills are taken first, `min(credit left, bill open)` at a
/// time. A credit never settles more than it has, a bill never more than its open amount, and
/// what a credit has left after all bills are full stays unmatched.
///
/// # Errors
/// `MoneyError::NegativeValue` for an open amount below 0; `MoneyError::Overflow`.
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

/// The three totals of a seller key: payable (P), open advances (A) and owed beyond them (E).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayableTotals {
    /// Σ still to pay (P).
    pub open: Paisa,
    /// His open advances (A).
    pub advances_open: Paisa,
    /// Payments beyond every credit (E): what he owes the firm beyond his advances.
    pub excess_paid: Paisa,
}

/// The totals of the working, from sums of the seller key's lines (money agent 8.2a).
///
/// ```
/// use mandi_core::money::Paisa;
/// use mandi_reports::reports::workings::payable_totals_from;
/// let p = |rupees| Paisa::from_rupees(rupees).unwrap();
/// // E2E-28 with the tick off: credit 24,316, payments 5,000, advance 15,000 open.
/// let t = payable_totals_from(p(24_316), p(5_000), p(15_000), Paisa::ZERO, Paisa::ZERO).unwrap();
/// assert_eq!((t.open, t.advances_open, t.excess_paid), (p(19_316), p(15_000), Paisa::ZERO));
/// ```
///
/// # Errors
/// `MoneyError::NegativeValue` if more is matched than a side holds (a bug in the matching rows);
/// `MoneyError::Overflow`.
pub fn payable_totals_from(
    credit: Paisa,
    payments: Paisa,
    advances_given: Paisa,
    adjusted_on_credits: Paisa,
    adjusted_on_advances: Paisa,
) -> Result<PayableTotals, MoneyError> {
    let credit_left = credit.checked_sub(adjusted_on_credits)?;
    let advances_open = advances_given.checked_sub(adjusted_on_advances)?;
    if credit_left.is_negative() || advances_open.is_negative() || payments.is_negative() {
        return Err(MoneyError::NegativeValue);
    }
    let difference = credit_left.checked_sub(payments)?;
    let (open, excess_paid) = if difference.is_negative() {
        (Paisa::ZERO, difference.checked_neg()?)
    } else {
        (difference, Paisa::ZERO)
    };
    Ok(PayableTotals {
        open,
        advances_open,
        excess_paid,
    })
}

/// What is still open on one bill (money lent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenAmount {
    /// The bill's journal line.
    pub line_id: i64,
    /// What returns have settled of it, oldest first.
    pub returned: Paisa,
    /// What is still open.
    pub open: Paisa,
}

/// The open amount of each bill when the credits settle the oldest bills first (PRD FR-16.5,
/// money agent 8.2b). `bills` and `credits` are `(line id, amount)`, oldest first. Done at read
/// time; nothing is stored. Σ open is `max(0, Σ bills - Σ credits)`.
///
/// ```
/// use mandi_core::money::Paisa;
/// use mandi_reports::reports::workings::open_amounts;
/// let p = |rupees| Paisa::from_rupees(rupees).unwrap();
/// // E2E-30: 5,000 lent, 3,000 returned in cash and 2,000 from salary.
/// let open = open_amounts(&[(1, p(5_000))], &[(2, p(3_000)), (3, p(2_000))]).unwrap();
/// assert_eq!((open[0].returned, open[0].open), (p(5_000), p(0)));
/// ```
///
/// # Errors
/// `MoneyError::NegativeValue` for an amount below 0, `MoneyError::Overflow`.
pub fn open_amounts(
    bills: &[(i64, Paisa)],
    credits: &[(i64, Paisa)],
) -> Result<Vec<OpenAmount>, MoneyError> {
    let as_open = |lines: &[(i64, Paisa)]| -> Vec<Open> {
        lines
            .iter()
            .map(|&(line_id, open)| Open { line_id, open })
            .collect()
    };
    let rows = match_bills(&as_open(bills), &as_open(credits))?;
    bills
        .iter()
        .map(|&(line_id, amount)| {
            let mut returned = Paisa::ZERO;
            for row in rows.iter().filter(|r| r.bill_line_id == line_id) {
                returned = returned.checked_add(row.amount)?;
            }
            Ok(OpenAmount {
                line_id,
                returned,
                open: amount.checked_sub(returned)?,
            })
        })
        .collect()
}

/// How many days `today` is after the return-by day, if the amount is still open and the day has
/// passed (PRD FR-7.44, E2E-30b: return by 15-10-26 and today 16-10-26 is 1). `None` when nothing
/// is open, there is no return-by day, or it has not passed.
pub fn days_past_return_by(
    open: Paisa,
    return_by: Option<BusinessDate>,
    today: BusinessDate,
) -> Option<i64> {
    let day = return_by?;
    if open <= Paisa::ZERO || today <= day {
        return None;
    }
    Some(today.days_after(day))
}
