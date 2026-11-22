//! The figures of a screen that holds several entries (plan 76 point 1, PRD FR-12.17 v1.41).
//!
//! A mixed save writes one entry for each carried lot and one for the new lots. Each entry has
//! its own summary and its own partner split, worked out by the same code as a standalone save.
//! The screen shows one total: these functions add the parts up with checked maths and word the
//! total like a whole بکری. They never round again: every figure is a sum of stored amounts, and
//! each part's split rounded on its own, so the total of a partner is exactly the sum of what the
//! entries will write.
//!
//! Example (the E2E-16 state, 11-10-26): a later sale with shared income 376 (152 / 112 / 112) and
//! a new لاٹ with 1,540 (616 / 462 / 462) give 1,916 and 768 / 574 / 574.

use mandi_core::bikri::{SplitPart, SummaryTotals};
use mandi_core::money::{MoneyError, Paisa};

use super::lot::{Acc, Scope};
use super::working;

/// Adds the summaries of the entries of one screen. `lots` is the number of lots on the screen,
/// which the "sum of N" workings count.
///
/// # Errors
/// `MoneyError::Overflow` for a sum that does not fit.
pub fn merge_summaries(parts: &[&SummaryTotals], lots: i64) -> Result<SummaryTotals, MoneyError> {
    let mut acc = Acc {
        count: lots,
        ..Acc::default()
    };
    for part in parts {
        acc.gross = acc.gross.checked_add(part.gross)?;
        acc.freight = acc.freight.checked_add(part.freight)?;
        acc.seller_comm = acc.seller_comm.checked_add(part.seller_comm)?;
        acc.labour = acc.labour.checked_add(part.labour)?;
        acc.munshiana = acc.munshiana.checked_add(part.munshiana)?;
        acc.cash_paid = acc.cash_paid.checked_add(part.cash_paid)?;
        acc.store_rent = acc.store_rent.checked_add(part.store_rent)?;
        acc.phone = acc.phone.checked_add(part.phone)?;
        acc.advance = acc.advance.checked_add(part.advance)?;
        acc.buyer_comm = acc.buyer_comm.checked_add(part.buyer_comm)?;
        acc.laga = acc.laga.checked_add(part.laga)?;
        acc.rate_diff = acc.rate_diff.checked_add(part.rate_diff)?;
        acc.buyer_credit = acc.buyer_credit.checked_add(part.buyer_credit)?;
        acc.buyer_cash = acc.buyer_cash.checked_add(part.buyer_cash)?;
        acc.buyer_total = acc.buyer_total.checked_add(part.buyer_total)?;
    }
    acc.totals(&Scope::Whole)
}

/// Adds the partner splits of the entries of one screen, partner by partner, in the order the
/// partners first appear. A partner who is in only some of the splits keeps what he has.
///
/// # Errors
/// `MoneyError::Overflow` for a sum that does not fit.
pub fn merge_splits(parts: &[&[SplitPart]]) -> Result<Vec<SplitPart>, MoneyError> {
    let mut merged: Vec<(SplitPart, i64)> = Vec::new();
    for split in parts {
        for part in *split {
            match merged
                .iter_mut()
                .find(|(m, _)| m.partner_id == part.partner_id)
            {
                Some((m, n)) => {
                    m.income_paisa = m.income_paisa.checked_add(part.income_paisa)?;
                    *n = n.checked_add(1).ok_or(MoneyError::Overflow)?;
                }
                None => merged.push((part.clone(), 1)),
            }
        }
    }
    Ok(merged
        .into_iter()
        .map(|(mut m, n)| {
            // One entry keeps its own working (formula with the share); several are a sum.
            if n > 1 {
                m.working = working::sum(n, m.income_paisa);
            }
            m
        })
        .collect())
}

/// The sum of the `change_paisa` of several cash checks: what the entries do to cash together.
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn sum_changes(changes: &[Paisa]) -> Result<Paisa, MoneyError> {
    changes
        .iter()
        .try_fold(Paisa::ZERO, |sum, change| sum.checked_add(*change))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    fn part(id: i64, income: i64) -> SplitPart {
        SplitPart {
            partner_id: id,
            name_ur: String::new(),
            name_en: format!("P{id}"),
            share_bps: 0,
            income_paisa: p(income),
            working: working::sum(1, p(income)),
        }
    }

    #[test]
    fn splits_add_up_partner_by_partner() {
        // The E2E-16 state: a later sale (376) and a new بکری (1,540).
        let a = vec![part(1, 152), part(2, 112), part(3, 112)];
        let b = vec![part(1, 616), part(2, 462), part(3, 462)];
        let merged = merge_splits(&[&a, &b]).unwrap();
        let sums: Vec<i64> = merged
            .iter()
            .map(|m| m.income_paisa.as_paisa() / 100)
            .collect();
        assert_eq!(sums, vec![768, 574, 574]);
        assert_eq!(sums.iter().sum::<i64>(), 1_916);
    }

    #[test]
    fn a_partner_in_one_split_only_keeps_his_part() {
        let a = vec![part(1, 100)];
        let b = vec![part(1, 50), part(2, 30)];
        let merged = merge_splits(&[&a, &b]).unwrap();
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[1].income_paisa, p(30));
    }

    #[test]
    fn cash_changes_add_up() {
        assert_eq!(sum_changes(&[p(0), p(12_800)]).unwrap(), p(12_800));
        assert_eq!(sum_changes(&[]).unwrap(), Paisa::ZERO);
    }
}
