//! What is payable to a seller, worked out when shown (PRD FR-7.59, FR-15.7, FR-15.9, FR-15.12,
//! plan 15, money agent Section 8.2a). Nothing is stored.
//!
//! The lines of one seller key, oldest first, go in. "Oldest" here is the order of his payable
//! postings, not the order of the account: his opening credit comes first whatever its date
//! (PRD FR-7.59, Umar 15-10-2026), then the others by day, entry and line. The caller puts the
//! lines in this order; the totals do not depend on it, only which credit is open and its age
//! do. For each credit line the working gives:
//! the credit, what his advances took from it, what has been paid against it, and what is still
//! to pay. Payments are his debit lines that are not advances (ادائیگی, نقد of a بکری, جنرل
//! debits), and they settle the oldest credits first, with the same `match_bills` as the buyers'
//! bills (money agent 8.1).
//!
//! Three numbers fall out, and they tie to the ledger (money agent 8.2a):
//!
//! - `open` (P): Σ still to pay.
//! - `advances_open` (A): Σ of his advance lines less what has been adjusted against them.
//! - `excess_paid` (E): payments left over after every credit is paid in full. This is what he
//!   owes the firm beyond his advances ("بیوپاری کے ذمے", PRD FR-15.7, FR-15.9).
//!
//! His balance (credit less debit) is always `P − A − E`. When `P` is above 0 then `E` is 0, so
//! the balance is `P − A`.
//!
//! The پختہ کھاتہ register (R9, `services::bikri::pukhta`) nets each lot's نقد inside its own lot,
//! which is how a one-time seller is paid on the spot (PRD FR-13.10). This working applies every
//! payment to the oldest credit first (PRD FR-7.59), so the open amount and the age of one
//! posting can differ between the two screens for such a seller. Their totals agree, and a test
//! checks it.
//!
//! ```
//! use mandi_domain::core::money::Paisa;
//! use mandi_domain::payables::{Line, work};
//! let p = |rupees| Paisa::from_rupees(rupees).unwrap();
//! let zero = Paisa::ZERO;
//! // PRD 4.7: رحیم's advance 15,000, his بکری credit 24,316 (advance adjusted 15,000),
//! // نقد 5,000 and an ادائیگی of 4,316.
//! let lines = [
//!     Line { line_id: 1, dr: p(15_000), cr: zero, is_advance: true, advance_matched: p(15_000) },
//!     Line { line_id: 2, dr: zero, cr: p(24_316), is_advance: false, advance_matched: p(15_000) },
//!     Line { line_id: 3, dr: p(5_000), cr: zero, is_advance: false, advance_matched: zero },
//!     Line { line_id: 4, dr: p(4_316), cr: zero, is_advance: false, advance_matched: zero },
//! ];
//! let w = work(&lines).unwrap();
//! assert_eq!((w.rows[0].paid, w.rows[0].still_open), (p(9_316), zero));
//! assert_eq!((w.open, w.advances_open, w.excess_paid), (zero, zero, zero));
//! ```

use mandi_core::money::{MoneyError, Paisa};

use crate::matching::{Open, match_bills};

/// One live line of the seller key, with the part of it that advance rows take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    /// The journal line.
    pub line_id: i64,
    /// Debit, 0 on a credit line.
    pub dr: Paisa,
    /// Credit, 0 on a debit line.
    pub cr: Paisa,
    /// True for an advance line (a debit that counts as an advance, PRD FR-15.8).
    pub is_advance: bool,
    /// The sum of the active advance rows that use this line, as the advance or as the credit.
    pub advance_matched: Paisa,
}

/// The working for one credit line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreditRow {
    /// The credit line.
    pub line_id: i64,
    /// The whole credit.
    pub credit: Paisa,
    /// What his advances took from it.
    pub advance_adjusted: Paisa,
    /// What the payments settled of it.
    pub paid: Paisa,
    /// What is still to pay: credit less adjusted less paid. Never below 0.
    pub still_open: Paisa,
}

/// The working for a seller key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayableWork {
    /// One row per credit line, oldest first.
    pub rows: Vec<CreditRow>,
    /// Σ still to pay (P).
    pub open: Paisa,
    /// His open advances (A).
    pub advances_open: Paisa,
    /// Payments beyond every credit (E): what he owes the firm beyond his advances.
    pub excess_paid: Paisa,
}

/// Works out the payable postings of a seller key from its live lines, oldest first.
///
/// # Errors
/// `MoneyError::NegativeValue` if a line has a negative amount or more is matched to it than it
/// holds (a bug in the matching rows); `MoneyError::Overflow` for a sum that does not fit.
pub fn work(lines: &[Line]) -> Result<PayableWork, MoneyError> {
    let mut credits: Vec<Open> = Vec::new();
    let mut adjusted_of: Vec<(i64, Paisa, Paisa)> = Vec::new();
    let mut payments: Vec<Open> = Vec::new();
    let mut advances_open = Paisa::ZERO;
    let mut payments_total = Paisa::ZERO;
    for line in lines {
        if line.dr.is_negative() || line.cr.is_negative() || line.advance_matched.is_negative() {
            return Err(MoneyError::NegativeValue);
        }
        if line.dr > Paisa::ZERO {
            if line.is_advance {
                // The open part of an advance line.
                let open = line.dr.checked_sub(line.advance_matched)?;
                if open.is_negative() {
                    return Err(MoneyError::NegativeValue);
                }
                advances_open = advances_open.checked_add(open)?;
            } else {
                payments.push(Open {
                    line_id: line.line_id,
                    open: line.dr,
                });
                payments_total = payments_total.checked_add(line.dr)?;
            }
        } else if line.cr > Paisa::ZERO {
            let open = line.cr.checked_sub(line.advance_matched)?;
            if open.is_negative() {
                return Err(MoneyError::NegativeValue);
            }
            credits.push(Open {
                line_id: line.line_id,
                open,
            });
            adjusted_of.push((line.line_id, line.cr, line.advance_matched));
        }
    }
    let matched = match_bills(&credits, &payments)?;
    let mut rows = Vec::with_capacity(adjusted_of.len());
    let mut open_total = Paisa::ZERO;
    let mut matched_total = Paisa::ZERO;
    for (line_id, credit, advance_adjusted) in adjusted_of {
        let mut paid = Paisa::ZERO;
        for row in matched.iter().filter(|m| m.bill_line_id == line_id) {
            paid = paid.checked_add(row.amount)?;
        }
        matched_total = matched_total.checked_add(paid)?;
        let still_open = credit.checked_sub(advance_adjusted)?.checked_sub(paid)?;
        open_total = open_total.checked_add(still_open)?;
        rows.push(CreditRow {
            line_id,
            credit,
            advance_adjusted,
            paid,
            still_open,
        });
    }
    Ok(PayableWork {
        rows,
        open: open_total,
        advances_open,
        excess_paid: payments_total.checked_sub(matched_total)?,
    })
}

/// The totals of the working, from sums of the seller key's lines (money agent 8.2a). Used when
/// only the totals are wanted, so a seller with a long history is not read line by line. Gives the
/// same `open`, `advances_open` and `excess_paid` as [`work`] over the same lines.
///
/// ```
/// use mandi_domain::core::money::Paisa;
/// use mandi_domain::payables::from_totals;
/// let p = |rupees| Paisa::from_rupees(rupees).unwrap();
/// // E2E-28 with the tick off: credit 24,316, payments 5,000, advance 15,000 open.
/// let t = from_totals(p(24_316), p(5_000), p(15_000), Paisa::ZERO, Paisa::ZERO).unwrap();
/// assert_eq!((t.open, t.advances_open, t.excess_paid), (p(19_316), p(15_000), Paisa::ZERO));
/// ```
///
/// # Errors
/// `MoneyError::NegativeValue` if more is matched than a side holds (a bug in the matching rows);
/// `MoneyError::Overflow`.
pub fn from_totals(
    credit: Paisa,
    payments: Paisa,
    advances_given: Paisa,
    adjusted_on_credits: Paisa,
    adjusted_on_advances: Paisa,
) -> Result<Totals, MoneyError> {
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
    Ok(Totals {
        open,
        advances_open,
        excess_paid,
    })
}

/// The three totals of a seller key: payable (P), open advances (A) and owed beyond them (E).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Totals {
    /// Σ still to pay (P).
    pub open: Paisa,
    /// His open advances (A).
    pub advances_open: Paisa,
    /// Payments beyond every credit (E).
    pub excess_paid: Paisa,
}

/// A payable posting that still has something to pay, found by walking his credit lines from the
/// newest back (see [`NewestWalk`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenPosting {
    /// The credit line.
    pub line_id: i64,
    /// The whole credit.
    pub credit: Paisa,
    /// What his advances took from it.
    pub advance_adjusted: Paisa,
    /// What the payments settled of it. Only the oldest posting of the walk can have any.
    pub paid: Paisa,
    /// What is still to pay on it. More than 0.
    pub still_open: Paisa,
}

/// Finds the postings that are still open without reading every credit of the seller. Payments
/// settle the OLDEST credits first, so the credits still open are the newest ones: walk back from
/// the newest until the open amounts add up to the payable `open_total` (P). Every posting passed
/// is open in full except the last, which is open for what is left of P.
///
/// ```
/// use mandi_domain::core::money::Paisa;
/// use mandi_domain::payables::NewestWalk;
/// let p = |rupees| Paisa::from_rupees(rupees).unwrap();
/// // Payable 12,000: the newest credit 8,000 is open in full, the one before it, 10,000, for 4,000.
/// let mut walk = NewestWalk::new(p(12_000));
/// assert!(!walk.push(2, p(8_000), Paisa::ZERO).unwrap());
/// assert!(walk.push(1, p(10_000), Paisa::ZERO).unwrap());
/// let found = walk.finish();
/// assert_eq!((found[0].line_id, found[0].paid, found[0].still_open), (1, p(6_000), p(4_000)));
/// assert_eq!((found[1].line_id, found[1].paid, found[1].still_open), (2, Paisa::ZERO, p(8_000)));
/// ```
#[derive(Debug, Clone)]
pub struct NewestWalk {
    remaining: Paisa,
    found: Vec<OpenPosting>,
}

impl NewestWalk {
    /// A walk for a payable of `open_total`.
    pub fn new(open_total: Paisa) -> Self {
        Self {
            remaining: open_total,
            found: Vec::new(),
        }
    }

    /// Takes the next credit line back in time. Returns true when the payable is covered and the
    /// walk can stop. A credit with nothing left after the advances adds nothing.
    ///
    /// # Errors
    /// `MoneyError::NegativeValue` if more is matched than the credit holds;
    /// `MoneyError::Overflow`.
    pub fn push(
        &mut self,
        line_id: i64,
        credit: Paisa,
        advance_adjusted: Paisa,
    ) -> Result<bool, MoneyError> {
        if self.remaining == Paisa::ZERO {
            return Ok(true);
        }
        let left = credit.checked_sub(advance_adjusted)?;
        if left.is_negative() || advance_adjusted.is_negative() {
            return Err(MoneyError::NegativeValue);
        }
        if left == Paisa::ZERO {
            return Ok(false);
        }
        let take = self.remaining.min(left);
        self.found.push(OpenPosting {
            line_id,
            credit,
            advance_adjusted,
            paid: left.checked_sub(take)?,
            still_open: take,
        });
        self.remaining = self.remaining.checked_sub(take)?;
        Ok(self.remaining == Paisa::ZERO)
    }

    /// True if the walk found less than the payable: the lines given were not all the credits.
    pub fn is_short(&self) -> bool {
        self.remaining != Paisa::ZERO
    }

    /// The open postings, OLDEST first.
    pub fn finish(mut self) -> Vec<OpenPosting> {
        self.found.reverse();
        self.found
    }
}

/// The most a seller can return by a وصولی (PRD FR-15.12): his open advances plus what he owes
/// the firm beyond them. A bigger amount is `ERR_RETURN_EXCEEDS_OPEN`.
///
/// ```
/// use mandi_domain::core::money::Paisa;
/// use mandi_domain::payables::return_limit;
/// let p = |rupees| Paisa::from_rupees(rupees).unwrap();
/// // E2E-31: 15,000 of advance open and nothing owed beyond it.
/// assert_eq!(return_limit(p(15_000), Paisa::ZERO).unwrap(), p(15_000));
/// ```
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn return_limit(advances_open: Paisa, excess_paid: Paisa) -> Result<Paisa, MoneyError> {
    advances_open.checked_add(excess_paid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    fn line(id: i64, dr: i64, cr: i64, advance: bool, matched: i64) -> Line {
        Line {
            line_id: id,
            dr: p(dr),
            cr: p(cr),
            is_advance: advance,
            advance_matched: p(matched),
        }
    }

    /// (credit, adjusted, paid, still open) of each row, in rupees.
    fn rows(w: &PayableWork) -> Vec<(i64, i64, i64, i64)> {
        w.rows
            .iter()
            .map(|r| {
                (
                    r.credit.rupees(),
                    r.advance_adjusted.rupees(),
                    r.paid.rupees(),
                    r.still_open.rupees(),
                )
            })
            .collect()
    }

    #[test]
    fn event_4_rahim_is_paid_in_full() {
        // Money agent 8.2a golden: credit 24,316, adjusted 15,000, paid 5,000 + 4,316, open 0.
        let w = work(&[
            line(1, 15_000, 0, true, 15_000),
            line(2, 0, 24_316, false, 15_000),
            line(3, 5_000, 0, false, 0),
            line(4, 4_316, 0, false, 0),
        ])
        .unwrap();
        assert_eq!(rows(&w), [(24_316, 15_000, 9_316, 0)]);
        assert_eq!(
            (w.open, w.advances_open, w.excess_paid),
            (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO)
        );
    }

    #[test]
    fn before_the_payment_rahim_is_owed_4316() {
        // E2E-7: payable 24,316 - 5,000 - 15,000 = 4,316; his ledger is 4,316 credit.
        let w = work(&[
            line(1, 15_000, 0, true, 15_000),
            line(2, 0, 24_316, false, 15_000),
            line(3, 5_000, 0, false, 0),
        ])
        .unwrap();
        assert_eq!(rows(&w), [(24_316, 15_000, 5_000, 4_316)]);
        assert_eq!(w.open, p(4_316));
    }

    #[test]
    fn e2e_27_karim_is_owed_10160() {
        let w = work(&[line(1, 0, 10_160, false, 0)]).unwrap();
        assert_eq!(rows(&w), [(10_160, 0, 0, 10_160)]);
    }

    #[test]
    fn e2e_28_with_the_tick_off_the_advance_stays_open() {
        // Credit 24,316 adjusted 0, نقد 5,000, advance 15,000 open: payable 19,316.
        let lines = [
            line(1, 15_000, 0, true, 0),
            line(2, 0, 24_316, false, 0),
            line(3, 5_000, 0, false, 0),
        ];
        let w = work(&lines).unwrap();
        assert_eq!(rows(&w), [(24_316, 0, 5_000, 19_316)]);
        assert_eq!((w.open, w.advances_open), (p(19_316), p(15_000)));
        // Ledger credit less debit = 24,316 - 20,000 = 4,316 = P - A.
        assert_eq!(w.open.checked_sub(w.advances_open).unwrap(), p(4_316));
        // After paying 19,316: nothing payable, the advance is still 15,000, nothing owed beyond.
        let mut paid = lines.to_vec();
        paid.push(line(4, 19_316, 0, false, 0));
        let w = work(&paid).unwrap();
        assert_eq!(
            (w.open, w.advances_open, w.excess_paid),
            (Paisa::ZERO, p(15_000), Paisa::ZERO)
        );
    }

    #[test]
    fn d4_a29_an_opening_credit_paid_in_part() {
        // کریم opens with 10,160 credit; an ادائیگی of 4,000: paid 4,000, open 6,160.
        let w = work(&[line(1, 0, 10_160, false, 0), line(2, 4_000, 0, false, 0)]).unwrap();
        assert_eq!(rows(&w), [(10_160, 0, 4_000, 6_160)]);
        assert_eq!(w.open, p(6_160));
    }

    #[test]
    fn d4_a29_a_general_credit_is_payable_and_the_advance_stays() {
        // رحیم: 15,000 advance open, a جنرل credit of 1,000: 1,000 payable, advance 15,000.
        let w = work(&[line(1, 15_000, 0, true, 0), line(2, 0, 1_000, false, 0)]).unwrap();
        assert_eq!(rows(&w), [(1_000, 0, 0, 1_000)]);
        assert_eq!((w.open, w.advances_open), (p(1_000), p(15_000)));
        // His balance is 14,000 debit: credit 1,000 less debit 15,000 = P - A.
    }

    #[test]
    fn e2e_31_the_receipt_remainder_and_the_last_advance() {
        // Advance 15,000; he returns 10,000 (adjusts 10,000); بکری 1 credit 24,316 adjusts the
        // last 5,000; نقد 5,000. Receipt remainder 10,000 - 10,000 = 0. Payable 14,316.
        let w = work(&[
            line(1, 15_000, 0, true, 15_000),
            line(2, 0, 10_000, false, 10_000),
            line(3, 0, 24_316, false, 5_000),
            line(4, 5_000, 0, false, 0),
        ])
        .unwrap();
        assert_eq!(
            rows(&w),
            [(10_000, 10_000, 0, 0), (24_316, 5_000, 5_000, 14_316)]
        );
        assert_eq!((w.open, w.advances_open), (p(14_316), Paisa::ZERO));
    }

    #[test]
    fn a_payment_above_the_payable_is_what_he_owes_beyond() {
        // FR-15.7: payable 4,316 and a payment of 6,000: the extra 1,684 is "بیوپاری کے ذمے".
        let w = work(&[
            line(1, 0, 24_316, false, 0),
            line(2, 20_000, 0, false, 0),
            line(3, 6_000, 0, false, 0),
        ])
        .unwrap();
        assert_eq!(rows(&w), [(24_316, 0, 24_316, 0)]);
        assert_eq!((w.open, w.excess_paid), (Paisa::ZERO, p(1_684)));
        assert_eq!(
            return_limit(w.advances_open, w.excess_paid).unwrap(),
            p(1_684)
        );
    }

    #[test]
    fn a_payment_dated_before_the_credit_still_settles_it() {
        // The books net: a debit is applied to the oldest credit whatever its position.
        let w = work(&[line(1, 3_000, 0, false, 0), line(2, 0, 5_000, false, 0)]).unwrap();
        assert_eq!(rows(&w), [(5_000, 0, 3_000, 2_000)]);
        assert_eq!(w.excess_paid, Paisa::ZERO);
    }

    #[test]
    fn more_matched_than_a_line_holds_is_a_bug_signal() {
        assert_eq!(
            work(&[line(1, 0, 1_000, false, 2_000)]).unwrap_err(),
            MoneyError::NegativeValue
        );
        assert_eq!(
            work(&[line(1, 1_000, 0, true, 2_000)]).unwrap_err(),
            MoneyError::NegativeValue
        );
    }

    #[test]
    fn totals_and_the_newest_walk_give_what_the_working_gives() {
        // Money agent 8.2a golden: رحیم after Event 2 (E2E-7): payable 4,316.
        let t = from_totals(p(24_316), p(5_000), p(15_000), p(15_000), p(15_000)).unwrap();
        assert_eq!(
            (t.open, t.advances_open, t.excess_paid),
            (p(4_316), Paisa::ZERO, Paisa::ZERO)
        );
        let mut walk = NewestWalk::new(t.open);
        assert!(walk.push(2, p(24_316), p(15_000)).unwrap());
        let found = walk.finish();
        assert_eq!(
            (
                found[0].credit,
                found[0].advance_adjusted,
                found[0].paid,
                found[0].still_open
            ),
            (p(24_316), p(15_000), p(5_000), p(4_316))
        );
        // More matched than a side holds is a bug signal.
        assert_eq!(
            from_totals(p(100), Paisa::ZERO, Paisa::ZERO, p(200), Paisa::ZERO).unwrap_err(),
            MoneyError::NegativeValue
        );
    }

    /// A random seller history. Kinds: 0 credit, 1 payment, 2 advance, 3 credit that an advance
    /// adjusted in part (filled in below).
    fn history() -> impl Strategy<Value = Vec<(u8, i64)>> {
        proptest::collection::vec((0_u8..3, 1_i64..50), 0..24)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        /// Money agent 8.2a: the balance is P - A - E whatever the history, nothing open is
        /// negative, and the paid amounts add back to the payments less E.
        #[test]
        fn the_ledger_ties_to_the_payable_working(history in history()) {
            // Build lines; advances are adjusted by credits oldest first (as the matching does).
            let mut lines: Vec<Line> = Vec::new();
            let mut adv_open: Vec<(usize, i64)> = Vec::new();
            for (n, (kind, amount)) in history.iter().enumerate() {
                let id = i64::try_from(n).unwrap() + 1;
                let amount = amount * 100;
                match kind {
                    0 => {
                        // A credit that adjusts the open advances before it.
                        let mut left = amount;
                        let mut matched = 0;
                        for slot in adv_open.iter_mut() {
                            let take = left.min(slot.1);
                            slot.1 -= take;
                            left -= take;
                            matched += take;
                            lines[slot.0].advance_matched =
                                Paisa::try_from_paisa(lines[slot.0].advance_matched.as_paisa() + take).unwrap();
                        }
                        lines.push(Line {
                            line_id: id,
                            dr: Paisa::ZERO,
                            cr: Paisa::try_from_paisa(amount).unwrap(),
                            is_advance: false,
                            advance_matched: Paisa::try_from_paisa(matched).unwrap(),
                        });
                    }
                    1 => lines.push(Line {
                        line_id: id,
                        dr: Paisa::try_from_paisa(amount).unwrap(),
                        cr: Paisa::ZERO,
                        is_advance: false,
                        advance_matched: Paisa::ZERO,
                    }),
                    _ => {
                        adv_open.push((lines.len(), amount));
                        lines.push(Line {
                            line_id: id,
                            dr: Paisa::try_from_paisa(amount).unwrap(),
                            cr: Paisa::ZERO,
                            is_advance: true,
                            advance_matched: Paisa::ZERO,
                        });
                    }
                }
            }
            let w = work(&lines).unwrap();
            let dr: i64 = lines.iter().map(|l| l.dr.as_paisa()).sum();
            let cr: i64 = lines.iter().map(|l| l.cr.as_paisa()).sum();
            // P - A - E = credit - debit.
            let lhs = w.open.as_paisa() - w.advances_open.as_paisa() - w.excess_paid.as_paisa();
            prop_assert_eq!(lhs, cr - dr);
            for row in &w.rows {
                prop_assert!(row.still_open >= Paisa::ZERO);
                prop_assert_eq!(
                    row.credit.as_paisa(),
                    row.advance_adjusted.as_paisa() + row.paid.as_paisa() + row.still_open.as_paisa()
                );
            }
            // The totals from sums, and the walk from the newest, give the same answer.
            let sum = |f: &dyn Fn(&Line) -> i64| Paisa::try_from_paisa(lines.iter().map(f).sum()).unwrap();
            let totals = from_totals(
                sum(&|l| l.cr.as_paisa()),
                sum(&|l| if l.is_advance { 0 } else { l.dr.as_paisa() }),
                sum(&|l| if l.is_advance { l.dr.as_paisa() } else { 0 }),
                sum(&|l| if l.cr > Paisa::ZERO { l.advance_matched.as_paisa() } else { 0 }),
                sum(&|l| if l.is_advance { l.advance_matched.as_paisa() } else { 0 }),
            ).unwrap();
            prop_assert_eq!((totals.open, totals.advances_open, totals.excess_paid), (w.open, w.advances_open, w.excess_paid));
            let mut walk = NewestWalk::new(totals.open);
            for line in lines.iter().rev().filter(|l| l.cr > Paisa::ZERO) {
                if walk.push(line.line_id, line.cr, line.advance_matched).unwrap() {
                    break;
                }
            }
            prop_assert!(!walk.is_short());
            let found: Vec<(i64, i64, i64)> = walk.finish().iter().map(|o| (o.line_id, o.paid.as_paisa(), o.still_open.as_paisa())).collect();
            let expected: Vec<(i64, i64, i64)> = w.rows.iter().filter(|r| r.still_open > Paisa::ZERO).map(|r| (r.line_id, r.paid.as_paisa(), r.still_open.as_paisa())).collect();
            prop_assert_eq!(found, expected);
            // Payable and excess never both above zero.
            prop_assert!(w.open == Paisa::ZERO || w.excess_paid == Paisa::ZERO);
            prop_assert!(w.excess_paid >= Paisa::ZERO && w.advances_open >= Paisa::ZERO);
        }
    }
}
