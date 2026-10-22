//! The partner split of a posting (PRD FR-6.5, FR-17.3, FR-17.4, C9 to C12, plan 11), as pure
//! functions. No database, no clock.
//!
//! One posting is one journal entry. Its lines on income accounts (head 04) make ONE income base
//! and its lines on shop expense accounts (head 05, the دکان خرچہ heads) make
//! ONE expense base. Each base is split on its own between the partners by their shares on the
//! posting date, with `split_by_bps` and nothing else (money agent Section 5):
//!
//! - income base = Σ credit − Σ debit over the head 04 lines (signed; a negative ریٹ فرق is a
//!   debit and lowers it);
//! - expense base = Σ debit − Σ credit over the head 05 lines (signed; a normal expense is
//!   positive, a refund credited to an expense account makes it negative);
//! - income row amount = the partner's part of the income base (same sign as the base);
//! - expense row amount = minus the partner's part of the expense base.
//!
//! The base stored on a row is the signed base that was split, so for one entry and one kind:
//! `Σ amount = base` for income and `Σ amount = −base` for expense. Over the whole entry,
//! `Σ all rows = income base − expense base`, exactly.
//!
//! The preview and the save both call [`share_bases`] and [`split_posting`] (Code Standards B5).
//!
//! ```
//! use mandi_domain::core::ids::PartnerId;
//! use mandi_domain::core::money::{Bps, Paisa};
//! use mandi_domain::partners::{ShareBases, split_posting};
//! // PRD 4.4: shared income 3,948 at 40 / 30 / 30 gives 1,580 / 1,184 / 1,184.
//! let shares = [
//!     (PartnerId::new(1), Bps::new(4_000).unwrap()),
//!     (PartnerId::new(2), Bps::new(3_000).unwrap()),
//!     (PartnerId::new(3), Bps::new(3_000).unwrap()),
//! ];
//! let bases = ShareBases { income: Paisa::from_rupees(3_948).unwrap(), expense: Paisa::ZERO };
//! let rows = split_posting(&shares, bases).unwrap();
//! let amounts: Vec<i64> = rows.iter().map(|r| r.amount.rupees()).collect();
//! assert_eq!(amounts, vec![1_580, 1_184, 1_184]);
//! ```

use mandi_core::ids::{AccountId, PartnerId};
use mandi_core::money::{BPS_FULL, Bps, MoneyError, Paisa, split_by_bps};
use thiserror::Error;

use crate::ledger::{BalancedEntry, Side};

/// What an account is for the partner split, taken from its head (`04` income, `05` shop
/// expense). Every other head is [`Nature::Other`]: its lines never make share rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nature {
    /// Head 04: commission, لاگا, فون, ریٹ فرق.
    Income,
    /// Head 05: the دکان خرچہ heads.
    Expense,
    /// Anything else.
    Other,
}

impl Nature {
    /// The nature of an account from the code of its head (`04`, `05`).
    pub fn from_head_code(head_code: &str) -> Nature {
        match head_code {
            "04" => Nature::Income,
            "05" => Nature::Expense,
            _ => Nature::Other,
        }
    }
}

/// Which base a share row came from (`partner_share_rows.kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShareKind {
    /// A part of the income base.
    Income,
    /// A part of the expense base.
    Expense,
}

impl ShareKind {
    /// The stored form.
    pub const fn as_str(self) -> &'static str {
        match self {
            ShareKind::Income => "income",
            ShareKind::Expense => "expense",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<ShareKind> {
        match text {
            "income" => Some(ShareKind::Income),
            "expense" => Some(ShareKind::Expense),
            _ => None,
        }
    }
}

/// The two signed bases of one entry (see the module text for the signs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShareBases {
    /// Σ credit − Σ debit over the income lines.
    pub income: Paisa,
    /// Σ debit − Σ credit over the shop expense lines.
    pub expense: Paisa,
}

impl ShareBases {
    /// True when the entry makes no share rows at all.
    pub fn is_zero(&self) -> bool {
        self.income == Paisa::ZERO && self.expense == Paisa::ZERO
    }
}

/// Works out the two bases of an entry from its lines. `nature_of` tells what each account is
/// (from its head). This is the one formula of C9 for the preview and the save (money agent,
/// design review of plan 11).
///
/// Example: the lines of PRD 4.4, sale part, give an income base of 3,948 (commissions 1,764 +
/// 1,764, لاگا 400, فون 20).
///
/// # Errors
/// `MoneyError::Overflow` if a sum does not fit.
pub fn share_bases(
    entry: &BalancedEntry,
    nature_of: impl Fn(AccountId) -> Nature,
) -> Result<ShareBases, MoneyError> {
    let (mut income, mut expense) = (Paisa::ZERO, Paisa::ZERO);
    for line in entry.lines() {
        match (nature_of(line.account_id), line.side) {
            (Nature::Income, Side::Cr) => income = income.checked_add(line.amount)?,
            (Nature::Income, Side::Dr) => income = income.checked_sub(line.amount)?,
            (Nature::Expense, Side::Dr) => expense = expense.checked_add(line.amount)?,
            (Nature::Expense, Side::Cr) => expense = expense.checked_sub(line.amount)?,
            (Nature::Other, _) => {}
        }
    }
    Ok(ShareBases { income, expense })
}

/// One share row to write: a partner's part of one base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShareRowDraft {
    /// The partner.
    pub partner_id: PartnerId,
    /// Which base.
    pub kind: ShareKind,
    /// The signed base that was split.
    pub base: Paisa,
    /// The partner's share on the posting date.
    pub share_bps: Bps,
    /// The partner's signed part: positive income is positive, a normal expense is negative.
    pub amount: Paisa,
}

impl ShareRowDraft {
    /// The row of a reversal: the same row with base and amount negated (PRD FR-17.4). A
    /// reversal copies the saved rows and never splits again, so it cancels the original
    /// exactly even if the shares have changed since.
    ///
    /// # Errors
    /// `MoneyError::Overflow` for a value that cannot be negated.
    pub fn mirrored(&self) -> Result<ShareRowDraft, MoneyError> {
        Ok(ShareRowDraft {
            base: self.base.checked_neg()?,
            amount: self.amount.checked_neg()?,
            ..*self
        })
    }
}

/// Why a split was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PartnersError {
    /// The shares on the posting date do not total exactly 10,000 bps (PRD FR-17.1, E9).
    #[error("shares total {total_bps} bps, not 10,000")]
    SharesNot100 {
        /// The total found.
        total_bps: i32,
    },
    /// A money failure (an overflow, or the same partner twice, which is a bug).
    #[error("money: {0}")]
    Money(MoneyError),
    /// The rows did not add back to their base. A bug signal; nothing may be saved.
    #[error("the share rows do not add up to the amount that was split")]
    Inconsistent,
}

impl From<MoneyError> for PartnersError {
    fn from(error: MoneyError) -> Self {
        PartnersError::Money(error)
    }
}

/// The total of the shares in bps, saturating into `i32` (only used to report).
fn total_of(shares: &[(PartnerId, Bps)]) -> i32 {
    let total = shares.iter().fold(0_i64, |acc, (_, bps)| {
        acc.saturating_add(i64::from(bps.get()))
    });
    i32::try_from(total).unwrap_or(i32::MAX)
}

/// Splits the bases of one posting between the partners (PRD C10, C11).
///
/// `shares` are the shares of ALL partners on the posting date, in partner id order (an inactive
/// partner who still had a share on that date is included). Returns the rows: the income rows
/// first, then the expense rows, in the order of `shares`. A base of zero makes no rows, and a
/// partner with 0 bps makes none (he can never take the leftover, because the largest share
/// wins and the shares total 10,000).
///
/// The total of the shares is checked only when at least one base is not zero, so an entry that
/// nets to zero (for example a جنرل moving money between two income accounts) posts even when
/// the shares are wrong, and writes no rows.
///
/// # Errors
/// `SharesNot100` (with `total_bps`) when a base is not zero and the shares do not total
/// 10,000; `Money(Overflow)`; `Inconsistent` if the rows do not add back to a base.
pub fn split_posting(
    shares: &[(PartnerId, Bps)],
    bases: ShareBases,
) -> Result<Vec<ShareRowDraft>, PartnersError> {
    if bases.is_zero() {
        return Ok(Vec::new());
    }
    let total_bps = total_of(shares);
    if total_bps != BPS_FULL {
        return Err(PartnersError::SharesNot100 { total_bps });
    }
    let active: Vec<(PartnerId, Bps)> = shares
        .iter()
        .copied()
        .filter(|&(_, bps)| bps.get() > 0)
        .collect();
    let mut rows = Vec::with_capacity(active.len().saturating_mul(2));
    if bases.income != Paisa::ZERO {
        for (partner_id, part) in split_by_bps(bases.income, &active)? {
            rows.push(ShareRowDraft {
                partner_id,
                kind: ShareKind::Income,
                base: bases.income,
                share_bps: share_of(&active, partner_id)?,
                amount: part,
            });
        }
    }
    if bases.expense != Paisa::ZERO {
        for (partner_id, part) in split_by_bps(bases.expense, &active)? {
            rows.push(ShareRowDraft {
                partner_id,
                kind: ShareKind::Expense,
                base: bases.expense,
                share_bps: share_of(&active, partner_id)?,
                amount: part.checked_neg()?,
            });
        }
    }
    check_rows(&rows, bases)?;
    Ok(rows)
}

/// The share of a partner in the list that was split.
fn share_of(shares: &[(PartnerId, Bps)], partner_id: PartnerId) -> Result<Bps, PartnersError> {
    shares
        .iter()
        .find(|&&(id, _)| id == partner_id)
        .map(|&(_, bps)| bps)
        .ok_or(PartnersError::Inconsistent)
}

/// The rows of each kind must add back to their base (PRD FR-6.5): income rows to the income
/// base, expense rows to minus the expense base.
fn check_rows(rows: &[ShareRowDraft], bases: ShareBases) -> Result<(), PartnersError> {
    let sum_of = |kind: ShareKind| -> Result<Paisa, MoneyError> {
        rows.iter()
            .filter(|r| r.kind == kind)
            .try_fold(Paisa::ZERO, |acc, r| acc.checked_add(r.amount))
    };
    if sum_of(ShareKind::Income)? != bases.income
        || sum_of(ShareKind::Expense)? != bases.expense.checked_neg()?
    {
        return Err(PartnersError::Inconsistent);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{LedgerLine, build_entry};
    use proptest::prelude::*;

    fn rs(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    fn shares(rows: &[(i64, i32)]) -> Vec<(PartnerId, Bps)> {
        rows.iter()
            .map(|&(id, bps)| (PartnerId::new(id), Bps::new(bps).unwrap()))
            .collect()
    }

    fn forty_thirty_thirty() -> Vec<(PartnerId, Bps)> {
        shares(&[(1, 4_000), (2, 3_000), (3, 3_000)])
    }

    fn bases(income: i64, expense: i64) -> ShareBases {
        ShareBases {
            income: rs(income),
            expense: rs(expense),
        }
    }

    /// The amounts in rupees, income rows then expense rows.
    fn amounts(rows: &[ShareRowDraft]) -> Vec<i64> {
        rows.iter().map(|r| r.amount.rupees()).collect()
    }

    fn split(shares: &[(PartnerId, Bps)], income: i64, expense: i64) -> Vec<ShareRowDraft> {
        split_posting(shares, bases(income, expense)).unwrap()
    }

    // ---- the worked numbers of plan 11 Section 3 ----

    #[test]
    fn income_3948_is_1580_1184_1184() {
        // PRD 4.4, E2E-4: 1,579 + 1,184 + 1,184 = 3,947, the leftover rupee goes to الف.
        let rows = split(&forty_thirty_thirty(), 3_948, 0);
        assert_eq!(amounts(&rows), vec![1_580, 1_184, 1_184]);
        assert!(rows.iter().all(|r| r.kind == ShareKind::Income));
        assert!(rows.iter().all(|r| r.base == rs(3_948)));
        let bps: Vec<i32> = rows.iter().map(|r| r.share_bps.get()).collect();
        assert_eq!(bps, vec![4_000, 3_000, 3_000]);
    }

    #[test]
    fn income_1540_has_no_leftover() {
        // PRD 4.5, E2E-5.
        let rows = split(&forty_thirty_thirty(), 1_540, 0);
        assert_eq!(amounts(&rows), vec![616, 462, 462]);
    }

    #[test]
    fn expense_3000_is_minus_1200_900_900() {
        let rows = split(&forty_thirty_thirty(), 0, 3_000);
        assert_eq!(amounts(&rows), vec![-1_200, -900, -900]);
        assert!(rows.iter().all(|r| r.kind == ShareKind::Expense));
        assert!(rows.iter().all(|r| r.base == rs(3_000)));
    }

    #[test]
    fn expense_3384_is_minus_1354_1015_1015() {
        // E2E-20: 1,353 + 1,015 + 1,015 = 3,383, the leftover rupee goes to الف, then negated.
        let rows = split(&forty_thirty_thirty(), 0, 3_384);
        assert_eq!(amounts(&rows), vec![-1_354, -1_015, -1_015]);
        assert_eq!(amounts(&rows).iter().sum::<i64>(), -3_384);
    }

    #[test]
    fn negative_income_300_is_minus_120_90_90() {
        // C11: the absolute amount is split, then negated. floor(-120.0) style mistakes would
        // give different numbers on other amounts, so also try -3,948.
        let rows = split(&forty_thirty_thirty(), -300, 0);
        assert_eq!(amounts(&rows), vec![-120, -90, -90]);
        assert!(rows.iter().all(|r| r.kind == ShareKind::Income));
        let rows = split(&forty_thirty_thirty(), -3_948, 0);
        assert_eq!(amounts(&rows), vec![-1_580, -1_184, -1_184]);
    }

    #[test]
    fn income_and_cost_in_one_entry_are_split_separately() {
        // E2E-34: income 9,660 and cost of goods sold 7,800.
        let rows = split(&forty_thirty_thirty(), 9_660, 7_800);
        assert_eq!(rows.len(), 6);
        let income: Vec<i64> = rows
            .iter()
            .filter(|r| r.kind == ShareKind::Income)
            .map(|r| r.amount.rupees())
            .collect();
        let expense: Vec<i64> = rows
            .iter()
            .filter(|r| r.kind == ShareKind::Expense)
            .map(|r| r.amount.rupees())
            .collect();
        assert_eq!(income, vec![3_864, 2_898, 2_898]);
        assert_eq!(expense, vec![-3_120, -2_340, -2_340]);
        // Net per partner 744 / 558 / 558, together 1,860 = 9,660 - 7,800.
        assert_eq!(amounts(&rows).iter().sum::<i64>(), 1_860);
    }

    #[test]
    fn income_and_expense_are_not_split_as_a_net() {
        // Income 10 gives 4 / 3 / 3. Expense 1 gives -1 / 0 / 0. Net 3 / 3 / 3. A split of the
        // net 9 would give 5 / 2 / 2. The PRD splits each base on its own (E2E-34).
        let rows = split(&forty_thirty_thirty(), 10, 1);
        let income: Vec<i64> = rows
            .iter()
            .filter(|r| r.kind == ShareKind::Income)
            .map(|r| r.amount.rupees())
            .collect();
        let expense: Vec<i64> = rows
            .iter()
            .filter(|r| r.kind == ShareKind::Expense)
            .map(|r| r.amount.rupees())
            .collect();
        assert_eq!(income, vec![4, 3, 3]);
        assert_eq!(expense, vec![-1, 0, 0]);
    }

    #[test]
    fn a_refund_on_an_expense_account_gives_positive_expense_rows() {
        let rows = split(&forty_thirty_thirty(), 0, -300);
        assert_eq!(amounts(&rows), vec![120, 90, 90]);
        assert!(rows.iter().all(|r| r.kind == ShareKind::Expense));
        assert!(rows.iter().all(|r| r.base == rs(-300)));
    }

    #[test]
    fn shares_not_100_are_refused_with_the_total() {
        let nine = shares(&[(1, 4_000), (2, 3_000), (3, 2_000)]);
        assert_eq!(
            split_posting(&nine, bases(3_948, 0)),
            Err(PartnersError::SharesNot100 { total_bps: 9_000 })
        );
        let over = shares(&[(1, 6_000), (2, 5_000)]);
        assert_eq!(
            split_posting(&over, bases(0, 100)),
            Err(PartnersError::SharesNot100 { total_bps: 11_000 })
        );
        assert_eq!(
            split_posting(&[], bases(100, 0)),
            Err(PartnersError::SharesNot100 { total_bps: 0 })
        );
    }

    #[test]
    fn a_zero_base_needs_no_shares_and_writes_no_rows() {
        let wrong = shares(&[(1, 4_000)]);
        assert_eq!(split_posting(&wrong, bases(0, 0)), Ok(Vec::new()));
        assert_eq!(split_posting(&[], bases(0, 0)), Ok(Vec::new()));
    }

    #[test]
    fn the_leftover_goes_to_the_largest_share_then_the_lowest_id() {
        // Equal shares: 1,001 at 50 / 50 gives 501 / 500, the lower id first.
        let rows = split(&shares(&[(1, 5_000), (2, 5_000)]), 1_001, 0);
        assert_eq!(amounts(&rows), vec![501, 500]);
        // Thirds: the 3,334 partner takes the leftover.
        let thirds = shares(&[(1, 3_333), (2, 3_333), (3, 3_334)]);
        assert_eq!(amounts(&split(&thirds, 1_000, 0)), vec![333, 333, 334]);
    }

    #[test]
    fn one_partner_takes_everything() {
        let one = shares(&[(7, 10_000)]);
        assert_eq!(amounts(&split(&one, 777, 0)), vec![777]);
        assert_eq!(amounts(&split(&one, 0, 777)), vec![-777]);
    }

    #[test]
    fn a_partner_with_no_share_gets_no_row_and_no_leftover() {
        // 0 / 5000 / 5000 on 1,001: only partners 2 and 3 get rows.
        let rows = split(&shares(&[(1, 0), (2, 5_000), (3, 5_000)]), 1_001, 0);
        let who: Vec<i64> = rows.iter().map(|r| r.partner_id.get()).collect();
        assert_eq!(who, vec![2, 3]);
        assert_eq!(amounts(&rows), vec![501, 500]);
    }

    #[test]
    fn ten_partners_add_back() {
        let ten = shares(&[
            (1, 1_000),
            (2, 1_000),
            (3, 1_000),
            (4, 1_000),
            (5, 1_000),
            (6, 1_000),
            (7, 1_000),
            (8, 1_000),
            (9, 1_000),
            (10, 1_000),
        ]);
        let rows = split(&ten, 12_345, 6_789);
        assert_eq!(rows.len(), 20);
        assert_eq!(amounts(&rows).iter().sum::<i64>(), 12_345 - 6_789);
    }

    #[test]
    fn the_same_partner_twice_is_a_bug() {
        let twice = shares(&[(1, 5_000), (1, 5_000)]);
        assert_eq!(
            split_posting(&twice, bases(100, 0)),
            Err(PartnersError::Money(MoneyError::DuplicateId))
        );
    }

    #[test]
    fn a_reversed_row_cancels_the_original() {
        let rows = split(&forty_thirty_thirty(), 3_948, 3_384);
        for row in &rows {
            let back = row.mirrored().unwrap();
            assert_eq!(back.partner_id, row.partner_id);
            assert_eq!(back.kind, row.kind);
            assert_eq!(back.share_bps, row.share_bps);
            assert_eq!(back.base.as_paisa(), -row.base.as_paisa());
            assert_eq!(back.amount.as_paisa(), -row.amount.as_paisa());
            assert_eq!(back.mirrored().unwrap(), *row);
        }
    }

    // ---- the bases of an entry ----

    const CASH: i64 = 1;
    const BUYER: i64 = 2;
    const COMM_SELLER: i64 = 10; // head 04
    const COMM_BUYER: i64 = 11; // head 04
    const LAGA: i64 = 12; // head 04
    const PHONE: i64 = 13; // head 04
    const RATE_DIFF: i64 = 14; // head 04
    const ELECTRICITY: i64 = 20; // head 05, 0501
    const COST_OWN: i64 = 21; // head 05, 0502
    const CAPITAL: i64 = 30; // head 06
    const PAYABLE: i64 = 40; // head 02

    fn nature(id: AccountId) -> Nature {
        match id.get() {
            10..=14 => Nature::Income,
            20 | 21 => Nature::Expense,
            _ => Nature::Other,
        }
    }

    fn line(account: i64, side: Side, rupees: i64) -> LedgerLine {
        LedgerLine::new(AccountId::new(account), side, rs(rupees))
    }

    #[test]
    fn the_sale_part_of_bikri_1_has_an_income_base_of_3948() {
        // PRD 4.4 sale part (the seller and the payables are not income).
        let entry = build_entry(vec![
            line(BUYER, Side::Dr, 12_880),
            line(BUYER, Side::Dr, 18_684),
            line(PAYABLE, Side::Cr, 24_316),
            line(COMM_SELLER, Side::Cr, 1_764),
            line(COMM_BUYER, Side::Cr, 1_764),
            line(LAGA, Side::Cr, 400),
            line(PHONE, Side::Cr, 20),
            line(PAYABLE, Side::Cr, 3_300),
        ])
        .unwrap();
        let got = share_bases(&entry, nature).unwrap();
        assert_eq!(got.income, rs(3_948));
        assert_eq!(got.expense, Paisa::ZERO);
    }

    #[test]
    fn a_negative_rate_difference_lowers_the_income_base() {
        // The same lines with a rate difference of -100 written as a debit.
        let entry = build_entry(vec![
            line(BUYER, Side::Dr, 31_464),
            line(RATE_DIFF, Side::Dr, 100),
            line(PAYABLE, Side::Cr, 27_616),
            line(COMM_SELLER, Side::Cr, 1_764),
            line(COMM_BUYER, Side::Cr, 1_764),
            line(LAGA, Side::Cr, 400),
            line(PHONE, Side::Cr, 20),
        ])
        .unwrap();
        let got = share_bases(&entry, nature).unwrap();
        assert_eq!(got.income, rs(3_848), "3,948 - 100");
    }

    #[test]
    fn expense_lines_net_and_own_goods_cost_counts() {
        // A shop expense of 3,000, partly refunded 500, and the cost of own goods 7,800.
        let entry = build_entry(vec![
            line(ELECTRICITY, Side::Dr, 3_000),
            line(ELECTRICITY, Side::Cr, 500),
            line(COST_OWN, Side::Dr, 7_800),
            line(CASH, Side::Cr, 10_300),
        ])
        .unwrap();
        let got = share_bases(&entry, nature).unwrap();
        assert_eq!(got.expense, rs(10_300));
        assert_eq!(got.income, Paisa::ZERO);
    }

    #[test]
    fn other_heads_make_no_base() {
        let entry = build_entry(vec![
            line(CASH, Side::Dr, 65_000),
            line(CAPITAL, Side::Cr, 65_000),
        ])
        .unwrap();
        let got = share_bases(&entry, nature).unwrap();
        assert!(got.is_zero());
    }

    #[test]
    fn a_credit_and_a_debit_on_income_accounts_can_net_to_zero() {
        let entry = build_entry(vec![
            line(COMM_SELLER, Side::Dr, 500),
            line(COMM_BUYER, Side::Cr, 500),
        ])
        .unwrap();
        let got = share_bases(&entry, nature).unwrap();
        assert!(got.is_zero());
        // So it needs no shares and writes no rows.
        assert_eq!(split_posting(&[], got), Ok(Vec::new()));
    }

    #[test]
    fn an_income_line_and_an_expense_line_of_the_same_amount_cancel_per_partner() {
        let entry = build_entry(vec![
            line(ELECTRICITY, Side::Dr, 1_001),
            line(COMM_SELLER, Side::Cr, 1_001),
        ])
        .unwrap();
        let got = share_bases(&entry, nature).unwrap();
        let rows = split_posting(&forty_thirty_thirty(), got).unwrap();
        for partner in 1..=3 {
            let net: i64 = rows
                .iter()
                .filter(|r| r.partner_id.get() == partner)
                .map(|r| r.amount.as_paisa())
                .sum();
            assert_eq!(net, 0, "partner {partner}");
        }
    }

    #[test]
    fn the_largest_entry_a_journal_can_hold_splits_exactly() {
        use crate::ledger::MAX_LINE_PAISA;
        // One line at the limit on each side: the base is the limit, and the rows add back.
        let big = Paisa::try_from_paisa(MAX_LINE_PAISA).unwrap();
        let entry = build_entry(vec![
            LedgerLine::new(AccountId::new(CASH), Side::Dr, big),
            LedgerLine::new(AccountId::new(COMM_SELLER), Side::Cr, big),
        ])
        .unwrap();
        let got = share_bases(&entry, nature).unwrap();
        assert_eq!(got.income, big);
        let rows = split_posting(&forty_thirty_thirty(), got).unwrap();
        let sum: i64 = rows.iter().map(|r| r.amount.as_paisa()).sum();
        assert_eq!(sum, MAX_LINE_PAISA);
        // Past 64 bits, the checked maths returns an error and never wraps.
        let huge = Paisa::try_from_paisa(i64::MAX - i64::MAX % 100).unwrap();
        assert_eq!(huge.checked_add(huge), Err(MoneyError::Overflow));
    }

    // ---- property tests ----

    /// 1 to 10 partners with random shares totalling exactly 10,000.
    fn arb_shares() -> impl Strategy<Value = Vec<(PartnerId, Bps)>> {
        (1usize..=10).prop_flat_map(|n| {
            proptest::collection::vec(0u32..1_000, n).prop_map(move |weights| {
                let sum: u32 = weights.iter().sum::<u32>().max(1);
                let mut bps: Vec<i32> = weights
                    .iter()
                    .map(|&w| i32::try_from(w * 10_000 / sum).unwrap())
                    .collect();
                let given: i32 = bps.iter().sum();
                bps[0] += 10_000 - given;
                bps.iter()
                    .enumerate()
                    .map(|(i, &b)| {
                        (
                            PartnerId::new(i64::try_from(i).unwrap() + 1),
                            Bps::new(b).unwrap(),
                        )
                    })
                    .collect()
            })
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        #[test]
        fn rows_add_back_to_each_base(
            shares in arb_shares(),
            income in -50_000_000i64..50_000_000,
            expense in -50_000_000i64..50_000_000,
        ) {
            let b = bases(income, expense);
            let rows = split_posting(&shares, b).unwrap();
            let sum = |kind: ShareKind| -> i64 {
                rows.iter().filter(|r| r.kind == kind).map(|r| r.amount.as_paisa()).sum()
            };
            prop_assert_eq!(sum(ShareKind::Income), b.income.as_paisa());
            prop_assert_eq!(sum(ShareKind::Expense), -b.expense.as_paisa());
            // Whole entry: income base minus expense base.
            let all: i64 = rows.iter().map(|r| r.amount.as_paisa()).sum();
            prop_assert_eq!(all, b.income.as_paisa() - b.expense.as_paisa());
            for row in &rows {
                prop_assert_eq!(row.amount.as_paisa() % 100, 0);
                prop_assert_eq!(row.base.as_paisa() % 100, 0);
                prop_assert!(row.share_bps.get() > 0);
                // The DB CHECK: a part is never more than the whole.
                prop_assert!(row.amount.as_paisa().abs() <= row.base.as_paisa().abs());
                // Income parts have the sign of the base; expense parts the opposite.
                match row.kind {
                    ShareKind::Income => {
                        if b.income.as_paisa() >= 0 { prop_assert!(row.amount.as_paisa() >= 0) }
                        else { prop_assert!(row.amount.as_paisa() <= 0) }
                    }
                    ShareKind::Expense => {
                        if b.expense.as_paisa() >= 0 { prop_assert!(row.amount.as_paisa() <= 0) }
                        else { prop_assert!(row.amount.as_paisa() >= 0) }
                    }
                }
            }
        }

        #[test]
        fn rows_and_their_reversal_make_zero_per_partner(
            shares in arb_shares(),
            income in -50_000_000i64..50_000_000,
            expense in -50_000_000i64..50_000_000,
        ) {
            let rows = split_posting(&shares, bases(income, expense)).unwrap();
            let back: Vec<ShareRowDraft> =
                rows.iter().map(|r| r.mirrored().unwrap()).collect();
            for (partner, _) in &shares {
                for kind in [ShareKind::Income, ShareKind::Expense] {
                    let net: i64 = rows
                        .iter()
                        .chain(back.iter())
                        .filter(|r| r.partner_id == *partner && r.kind == kind)
                        .map(|r| r.amount.as_paisa())
                        .sum();
                    prop_assert_eq!(net, 0);
                    let base_net: i64 = rows
                        .iter()
                        .chain(back.iter())
                        .filter(|r| r.partner_id == *partner && r.kind == kind)
                        .map(|r| r.base.as_paisa())
                        .sum();
                    prop_assert_eq!(base_net, 0);
                }
            }
        }

        #[test]
        fn each_partner_gets_the_split_of_the_income_minus_the_split_of_the_expense(
            shares in arb_shares(),
            income in 0i64..50_000_000,
            expense in 0i64..50_000_000,
        ) {
            let rows = split_posting(&shares, bases(income, expense)).unwrap();
            let income_parts = split_by_bps(rs(income), &shares).unwrap();
            let expense_parts = split_by_bps(rs(expense), &shares).unwrap();
            for (index, (partner, _)) in shares.iter().enumerate() {
                let mine: i64 = rows
                    .iter()
                    .filter(|r| r.partner_id == *partner)
                    .map(|r| r.amount.as_paisa())
                    .sum();
                let want = income_parts[index].1.as_paisa() - expense_parts[index].1.as_paisa();
                prop_assert_eq!(mine, want);
            }
        }
    }

    // ---- a property over a whole sequence of postings (Code Standards 9.3) ----

    /// One random line: `(account, is_debit, rupees)`. Accounts 10 to 14 are income, 20 and 21
    /// shop expense, the rest other heads (see `nature`). Cash (account 1) balances the entry.
    type RawLine = (i64, bool, i64);

    fn arb_raw_entry() -> impl Strategy<Value = Vec<RawLine>> {
        let account = prop_oneof![
            Just(2_i64),
            10_i64..=14,
            20_i64..=21,
            Just(30_i64),
            Just(40_i64)
        ];
        proptest::collection::vec((account, any::<bool>(), 1_i64..200_000), 1..7)
    }

    /// The entry with a cash line that makes it balance.
    fn balanced(lines: &[RawLine]) -> BalancedEntry {
        let mut built: Vec<LedgerLine> = lines
            .iter()
            .map(|&(account, is_debit, rupees)| {
                line(account, if is_debit { Side::Dr } else { Side::Cr }, rupees)
            })
            .collect();
        let net: i64 = lines
            .iter()
            .map(|&(_, is_debit, rupees)| if is_debit { rupees } else { -rupees })
            .sum();
        if net > 0 {
            built.push(line(CASH, Side::Cr, net));
        } else if net < 0 {
            built.push(line(CASH, Side::Dr, -net));
        }
        build_entry(built).unwrap()
    }

    /// Income minus expense of the raw lines, worked out apart from `share_bases`.
    fn income_minus_expense(lines: &[RawLine]) -> i64 {
        lines
            .iter()
            .map(|&(account, is_debit, rupees)| match account {
                10..=14 if is_debit => -rupees,
                10..=14 => rupees,
                20..=21 if is_debit => -rupees,
                20..=21 => rupees,
                _ => 0,
            })
            .sum()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        #[test]
        fn a_sequence_with_some_postings_reversed_adds_up_to_income_minus_expense_of_the_rest(
            entries in proptest::collection::vec(
                (arb_raw_entry(), 0usize..3, any::<bool>()),
                1..8,
            ),
            share_sets in proptest::collection::vec(arb_shares(), 3),
        ) {
            // `all_rows` is what the book holds: the rows of every posting, and the mirrored
            // rows of the postings that were reversed. `kept_rows` are the rows of the postings
            // that were not reversed.
            let mut all_rows: Vec<ShareRowDraft> = Vec::new();
            let mut kept_rows: Vec<ShareRowDraft> = Vec::new();
            let mut expected = 0_i64;
            for (raw, which, reversed) in &entries {
                // The shares can differ from one posting to the next (a share change).
                let shares = &share_sets[*which];
                let entry = balanced(raw);
                let bases = share_bases(&entry, nature).unwrap();
                let rows = split_posting(shares, bases).unwrap();
                let total: i64 = rows.iter().map(|r| r.amount.as_paisa()).sum();
                prop_assert_eq!(
                    total,
                    income_minus_expense(raw) * 100,
                    "rows of one posting = income - expense"
                );
                all_rows.extend(rows.iter().copied());
                if *reversed {
                    // The reversal copies the rows with the opposite sign, whatever the shares
                    // are by now.
                    let back: Vec<ShareRowDraft> =
                        rows.iter().map(|r| r.mirrored().unwrap()).collect();
                    let net: i64 = rows
                        .iter()
                        .chain(back.iter())
                        .map(|r| r.amount.as_paisa())
                        .sum();
                    prop_assert_eq!(net, 0);
                    all_rows.extend(back);
                } else {
                    kept_rows.extend(rows.iter().copied());
                    expected += income_minus_expense(raw) * 100;
                }
            }
            // The book: shares = income - shop expenses of the postings that were not reversed.
            let sum_all: i64 = all_rows.iter().map(|r| r.amount.as_paisa()).sum();
            prop_assert_eq!(sum_all, expected);
            // And partner by partner: a reversed posting leaves nothing behind for anybody.
            for partner in (1..=10).map(PartnerId::new) {
                let book: i64 = all_rows
                    .iter()
                    .filter(|r| r.partner_id == partner)
                    .map(|r| r.amount.as_paisa())
                    .sum();
                let kept: i64 = kept_rows
                    .iter()
                    .filter(|r| r.partner_id == partner)
                    .map(|r| r.amount.as_paisa())
                    .sum();
                prop_assert_eq!(book, kept);
            }
        }
    }
}
