//! The خرچہ screen and ادھار رقم as pure rules (PRD 16.2, 16.3, 16.5, FR-16.2, FR-16.4, FR-16.5,
//! FR-16.8, FR-16.9, plan 17): the journal entry of each type, the open amount of money taken and
//! money lent, and what a personal entry does to a partner.
//!
//! The preview and the save call the same functions (Code Standards B5). Nothing here rounds: the
//! split of a shop expense between the partners is done by the hook of plan 11 inside the posting
//! (`split_by_bps` in `partners`), and every amount here is added or taken away with checked
//! maths.
//!
//! ```
//! use mandi_domain::core::money::Paisa;
//! use mandi_domain::expenses::{open_amounts, personal_effect, PersonalKind};
//! let p = |rupees| Paisa::from_rupees(rupees).unwrap();
//! // PRD E2E-30: 5,000 lent; 3,000 returned in cash and 2,000 from salary leave nothing open.
//! let rows = open_amounts(&[(1, p(5_000))], &[(2, p(3_000)), (3, p(2_000))]).unwrap();
//! assert_eq!((rows[0].returned, rows[0].open), (p(5_000), p(0)));
//! // PRD E2E-39: Partner A has 996; taking 2,000 leaves -1,004 and 2,000 open.
//! let effect = personal_effect(PersonalKind::Take, p(996), p(0), p(2_000)).unwrap();
//! assert_eq!(effect.total_after, p(-1_004));
//! assert_eq!(effect.open_after, p(2_000));
//! ```

use mandi_core::dates::BusinessDate;
use mandi_core::ids::AccountId;
use mandi_core::money::{MoneyError, Paisa};

use crate::ledger::{BalancedEntry, LedgerError, LedgerLine, Side, build_entry};
use crate::matching::{Open, match_bills};

/// An entry of two lines: `debit` بنام and `credit` جمع, both for `amount`.
pub(crate) fn two_lines(
    debit: AccountId,
    credit: AccountId,
    amount: Paisa,
) -> Result<BalancedEntry, LedgerError> {
    build_entry(vec![
        LedgerLine::new(debit, Side::Dr, amount),
        LedgerLine::new(credit, Side::Cr, amount),
    ])
}

/// A دکان خرچہ (PRD 16.2): the head's account بنام, cash جمع. The hook of plan 11 splits it
/// between the partners (PRD FR-6.5, E2E-20).
///
/// # Errors
/// The errors of [`build_entry`].
pub fn shop_expense_entry(
    head_account: AccountId,
    cash: AccountId,
    amount: Paisa,
) -> Result<BalancedEntry, LedgerError> {
    two_lines(head_account, cash, amount)
}

/// A ذاتی خرچہ (PRD FR-16.2): the partner's drawings account بنام, cash جمع. Never split.
///
/// # Errors
/// The errors of [`build_entry`].
pub fn personal_expense_entry(
    drawings: AccountId,
    cash: AccountId,
    amount: Paisa,
) -> Result<BalancedEntry, LedgerError> {
    two_lines(drawings, cash, amount)
}

/// A ذاتی خرچہ واپسی (PRD FR-16.2): cash بنام, the drawings account جمع.
///
/// # Errors
/// The errors of [`build_entry`].
pub fn personal_return_entry(
    drawings: AccountId,
    cash: AccountId,
    amount: Paisa,
) -> Result<BalancedEntry, LedgerError> {
    two_lines(cash, drawings, amount)
}

/// Money lent (PRD FR-16.4): the person's account بنام, cash جمع.
///
/// # Errors
/// The errors of [`build_entry`].
pub fn money_lent_entry(
    person: AccountId,
    cash: AccountId,
    amount: Paisa,
) -> Result<BalancedEntry, LedgerError> {
    two_lines(person, cash, amount)
}

/// Money lent, returned in cash (PRD FR-16.4): cash بنام, the person جمع.
///
/// # Errors
/// The errors of [`build_entry`].
pub fn money_lent_return_cash_entry(
    person: AccountId,
    cash: AccountId,
    amount: Paisa,
) -> Result<BalancedEntry, LedgerError> {
    two_lines(cash, person, amount)
}

/// Money lent, returned from salary (PRD FR-16.4): the salary head بنام, the person جمع. No cash
/// moves. The salary head is a shop expense, so the hook splits it between the partners (E2E-30:
/// 2,000 gives -800 / -600 / -600).
///
/// # Errors
/// The errors of [`build_entry`].
pub fn money_lent_return_salary_entry(
    person: AccountId,
    salary_head: AccountId,
    amount: Paisa,
) -> Result<BalancedEntry, LedgerError> {
    two_lines(salary_head, person, amount)
}

/// What is still open on one bill (money taken, money lent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenAmount {
    /// The bill's journal line.
    pub line_id: i64,
    /// What returns have settled of it, oldest first.
    pub returned: Paisa,
    /// What is still open.
    pub open: Paisa,
}

/// The open amount of each bill when the credits settle the oldest bills first (money agent 8.2b,
/// PRD FR-16.2, FR-16.5). `bills` and `credits` are `(line id, amount)`, oldest first. It is the
/// same working as the matching of a buyer's bills (`match_bills`), done at read time; nothing is
/// stored. Σ open is `max(0, Σ bills - Σ credits)`, which is the account's debit balance.
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

/// A personal entry of the خرچہ screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalKind {
    /// ذاتی خرچہ: the partner takes money.
    Take,
    /// ذاتی خرچہ واپسی: the partner gives money back.
    Return,
}

/// What a personal entry does to one partner (PRD FR-16.8, C20, C22).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonalEffect {
    /// His کل رقم after the entry.
    pub total_after: Paisa,
    /// The money he took that is still open, after the entry.
    pub open_after: Paisa,
}

/// His کل رقم and his open amount after a personal entry (PRD FR-16.8): a take lowers the total
/// and raises the open amount; a return does the opposite. Only the chosen partner changes; the
/// others are not affected. The open amount never goes below 0: a return above it is refused
/// (`ERR_RETURN_EXCEEDS_OPEN`) before it comes here.
///
/// # Errors
/// `MoneyError::Overflow`, `MoneyError::NegativeValue` for an open amount below 0.
pub fn personal_effect(
    kind: PersonalKind,
    total_before: Paisa,
    open_before: Paisa,
    amount: Paisa,
) -> Result<PersonalEffect, MoneyError> {
    let (total_after, open_after) = match kind {
        PersonalKind::Take => (
            total_before.checked_sub(amount)?,
            open_before.checked_add(amount)?,
        ),
        PersonalKind::Return => (
            total_before.checked_add(amount)?,
            open_before.checked_sub(amount)?,
        ),
    };
    if open_after.is_negative() {
        return Err(MoneyError::NegativeValue);
    }
    Ok(PersonalEffect {
        total_after,
        open_after,
    })
}

/// The part of a return that is above what is open, if any (PRD FR-16.2, FR-16.5): `None` when
/// the return fits. The open amount is the account's debit balance, never below 0.
///
/// # Errors
/// `MoneyError::Overflow`.
pub fn return_excess(open: Paisa, amount: Paisa) -> Result<Option<Paisa>, MoneyError> {
    if amount > open {
        Ok(Some(amount.checked_sub(open)?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    fn date(y: i32, m: u32, d: u32) -> BusinessDate {
        BusinessDate::from_ymd(y, m, d).unwrap()
    }

    fn sides(entry: &BalancedEntry) -> Vec<(i64, Side, Paisa)> {
        entry
            .lines()
            .iter()
            .map(|l| (l.account_id.get(), l.side, l.amount))
            .collect()
    }

    #[test]
    fn each_type_posts_the_sides_of_the_prd() {
        let (head, cash, person) = (AccountId::new(10), AccountId::new(1), AccountId::new(20));
        assert_eq!(
            sides(&shop_expense_entry(head, cash, p(3_000)).unwrap()),
            vec![(10, Side::Dr, p(3_000)), (1, Side::Cr, p(3_000))]
        );
        assert_eq!(
            sides(&personal_expense_entry(head, cash, p(2_000)).unwrap()),
            vec![(10, Side::Dr, p(2_000)), (1, Side::Cr, p(2_000))]
        );
        assert_eq!(
            sides(&personal_return_entry(head, cash, p(2_000)).unwrap()),
            vec![(1, Side::Dr, p(2_000)), (10, Side::Cr, p(2_000))]
        );
        assert_eq!(
            sides(&money_lent_entry(person, cash, p(5_000)).unwrap()),
            vec![(20, Side::Dr, p(5_000)), (1, Side::Cr, p(5_000))]
        );
        assert_eq!(
            sides(&money_lent_return_cash_entry(person, cash, p(3_000)).unwrap()),
            vec![(1, Side::Dr, p(3_000)), (20, Side::Cr, p(3_000))]
        );
        // The salary deduction touches no cash at all.
        let salary = money_lent_return_salary_entry(person, head, p(2_000)).unwrap();
        assert_eq!(
            sides(&salary),
            vec![(10, Side::Dr, p(2_000)), (20, Side::Cr, p(2_000))]
        );
        assert!(salary.lines().iter().all(|l| l.account_id != cash));
    }

    #[test]
    fn a_zero_amount_is_not_an_entry() {
        let (a, b) = (AccountId::new(1), AccountId::new(2));
        assert!(shop_expense_entry(a, b, Paisa::ZERO).is_err());
    }

    #[test]
    fn e2e_30_the_loan_is_settled_oldest_first() {
        // 5,000 lent on 10-10; 3,000 cash back on 11-10; 2,000 from salary on 12-10.
        let after_cash = open_amounts(&[(1, p(5_000))], &[(2, p(3_000))]).unwrap();
        assert_eq!(
            (after_cash[0].returned, after_cash[0].open),
            (p(3_000), p(2_000))
        );
        let after_salary = open_amounts(&[(1, p(5_000))], &[(2, p(3_000)), (3, p(2_000))]).unwrap();
        assert_eq!(
            (after_salary[0].returned, after_salary[0].open),
            (p(5_000), Paisa::ZERO)
        );
    }

    #[test]
    fn a_return_settles_the_oldest_bill_first_and_may_split_over_two() {
        // Bills 4,000 and 3,000; a return of 5,000 settles 4,000 and 1,000.
        let rows = open_amounts(&[(1, p(4_000)), (2, p(3_000))], &[(9, p(5_000))]).unwrap();
        assert_eq!((rows[0].returned, rows[0].open), (p(4_000), Paisa::ZERO));
        assert_eq!((rows[1].returned, rows[1].open), (p(1_000), p(2_000)));
        // The open amounts add up to the debit balance 7,000 - 5,000.
        assert_eq!(rows[0].open.checked_add(rows[1].open).unwrap(), p(2_000));
    }

    #[test]
    fn credits_above_the_bills_leave_nothing_open_never_a_negative() {
        let rows = open_amounts(&[(1, p(1_000))], &[(9, p(5_000))]).unwrap();
        assert_eq!(rows[0].open, Paisa::ZERO);
        assert!(open_amounts(&[], &[(9, p(5_000))]).unwrap().is_empty());
    }

    #[test]
    fn days_past_counts_only_open_amounts_after_the_day() {
        let by = Some(date(2026, 10, 15));
        // E2E-30b: today 16-10-26 is 1 day past 15-10-26.
        assert_eq!(
            days_past_return_by(p(2_000), by, date(2026, 10, 16)),
            Some(1)
        );
        // The same day is not past.
        assert_eq!(days_past_return_by(p(2_000), by, date(2026, 10, 15)), None);
        // Nothing open, or no return-by day.
        assert_eq!(
            days_past_return_by(Paisa::ZERO, by, date(2026, 11, 1)),
            None
        );
        assert_eq!(days_past_return_by(p(1), None, date(2026, 11, 1)), None);
    }

    #[test]
    fn e2e_39_a_take_and_its_return_move_only_the_chosen_partner() {
        // A's total is 2,196 - 1,200 = 996 before the take.
        let take = personal_effect(PersonalKind::Take, p(996), Paisa::ZERO, p(2_000)).unwrap();
        assert_eq!(take.total_after, p(-1_004));
        assert_eq!(take.open_after, p(2_000));
        // E2E-29: the return of 2,000 brings the total back to 996 and nothing is open.
        let back = personal_effect(PersonalKind::Return, p(-1_004), p(2_000), p(2_000)).unwrap();
        assert_eq!(back.total_after, p(996));
        assert_eq!(back.open_after, Paisa::ZERO);
    }

    #[test]
    fn a_return_above_the_open_amount_is_not_an_effect() {
        assert_eq!(
            personal_effect(PersonalKind::Return, p(0), p(1_000), p(1_001)),
            Err(MoneyError::NegativeValue)
        );
        assert_eq!(return_excess(p(1_000), p(1_001)).unwrap(), Some(p(1)));
        assert_eq!(return_excess(p(1_000), p(1_000)).unwrap(), None);
    }

    mod properties {
        use super::*;
        use proptest::prelude::*;

        fn amounts() -> impl Strategy<Value = Vec<i64>> {
            proptest::collection::vec(1_i64..50_000, 0..12)
        }

        fn lines(rupees: &[i64], first_id: i64) -> Vec<(i64, Paisa)> {
            rupees
                .iter()
                .zip(first_id..)
                .map(|(r, id)| (id, Paisa::from_rupees(*r).unwrap()))
                .collect()
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(10_000))]

            /// Money agent 8.2b: for any bills and credits, no open amount is negative or above
            /// its bill, returned plus open is the bill, and Σ open is the debit balance
            /// max(0, Σ bills - Σ credits). The open amounts never depend on the order of the
            /// credits, only on their total.
            #[test]
            fn open_amounts_are_never_negative_and_add_up_to_the_balance(
                bills in amounts(),
                credits in amounts(),
            ) {
                let bill_lines = lines(&bills, 1);
                let credit_lines = lines(&credits, 1_000);
                let rows = open_amounts(&bill_lines, &credit_lines).unwrap();
                prop_assert_eq!(rows.len(), bills.len());
                let mut open_total = 0_i64;
                for (row, (_, amount)) in rows.iter().zip(&bill_lines) {
                    prop_assert!(!row.open.is_negative());
                    prop_assert!(row.open <= *amount);
                    prop_assert_eq!(row.returned.checked_add(row.open).unwrap(), *amount);
                    open_total += row.open.rupees();
                }
                let balance = bills.iter().sum::<i64>() - credits.iter().sum::<i64>();
                prop_assert_eq!(open_total, balance.max(0));
                // The oldest bills are settled first: every bill before the first open one is
                // settled in full.
                if let Some(at) = rows.iter().position(|r| r.open > Paisa::ZERO) {
                    prop_assert!(rows.iter().take(at).all(|r| r.open == Paisa::ZERO));
                }
                let mut reversed = credit_lines.clone();
                reversed.reverse();
                let other: i64 = open_amounts(&bill_lines, &reversed)
                    .unwrap()
                    .iter()
                    .map(|r| r.open.rupees())
                    .sum();
                prop_assert_eq!(other, open_total);
            }

            /// A personal take and then its return gives back the same total and open amount.
            #[test]
            fn a_take_and_its_return_cancel_out(total in -100_000_i64..100_000, amount in 1_i64..50_000) {
                let (total, amount) = (Paisa::from_rupees(total).unwrap(), Paisa::from_rupees(amount).unwrap());
                let took = personal_effect(PersonalKind::Take, total, Paisa::ZERO, amount).unwrap();
                let back = personal_effect(PersonalKind::Return, took.total_after, took.open_after, amount).unwrap();
                prop_assert_eq!(back.total_after, total);
                prop_assert_eq!(back.open_after, Paisa::ZERO);
            }
        }
    }
}
