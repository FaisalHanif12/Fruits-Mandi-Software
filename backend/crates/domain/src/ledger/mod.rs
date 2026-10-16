//! Journal entries as pure functions (PRD FR-6.2, FR-6.3, FR-10.9, FR-12.9, money agent
//! Section 7).
//!
//! Every entry the app writes is built here first. A line has one side and a positive whole-rupee
//! amount; a negative amount is written on the other side. An entry whose debits do not equal its
//! credits is refused before anything is written ([`LedgerError::Unbalanced`]); the database
//! trigger is the second guard, not the first.
//!
//! ```
//! use mandi_domain::core::ids::AccountId;
//! use mandi_domain::core::money::Paisa;
//! use mandi_domain::ledger::{LedgerLine, Side, build_entry};
//! // PRD 4.6: Akram pays 10,000 cash.
//! let p = |rupees| Paisa::from_rupees(rupees).unwrap();
//! let entry = build_entry(vec![
//!     LedgerLine::new(AccountId::new(1), Side::Dr, p(10_000)),
//!     LedgerLine::new(AccountId::new(2), Side::Cr, p(10_000)),
//! ]).unwrap();
//! assert_eq!(entry.total(), p(10_000));
//! ```

use mandi_core::ids::AccountId;
use mandi_core::money::{MoneyError, Paisa};
use thiserror::Error;

pub mod bikri;

/// The most one journal line can hold, in paisa (the CHECK of `journal_lines`, 1e12 rupees).
pub const MAX_LINE_PAISA: i64 = 100_000_000_000_000;

/// What an entry is for (`journal_entries.posting_type`, Architecture 15.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostingType {
    /// The first posting of a بکری.
    BikriFirst,
    /// A later sale of a lot.
    BikriLater,
    /// The mirror of an earlier entry.
    Reversal,
    /// وصولی.
    Receipt,
    /// ادائیگی.
    Payment,
    /// پیشگی.
    Advance,
    /// دکان خرچہ.
    ShopExpense,
    /// ذاتی خرچہ (money a partner takes).
    PersonalExpense,
    /// A partner gives money back.
    PersonalReturn,
    /// ادھار رقم given.
    MoneyLent,
    /// ادھار رقم returned.
    MoneyLentReturn,
    /// جنرل.
    General,
    /// The opening of an account (PRD FR-10.9).
    Opening,
    /// Month close (PRD FR-17.6).
    MonthClose,
}

impl PostingType {
    /// The stored form.
    pub const fn as_str(self) -> &'static str {
        match self {
            PostingType::BikriFirst => "bikri_first",
            PostingType::BikriLater => "bikri_later",
            PostingType::Reversal => "reversal",
            PostingType::Receipt => "receipt",
            PostingType::Payment => "payment",
            PostingType::Advance => "advance",
            PostingType::ShopExpense => "shop_expense",
            PostingType::PersonalExpense => "personal_expense",
            PostingType::PersonalReturn => "personal_return",
            PostingType::MoneyLent => "money_lent",
            PostingType::MoneyLentReturn => "money_lent_return",
            PostingType::General => "general",
            PostingType::Opening => "opening",
            PostingType::MonthClose => "month_close",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<PostingType> {
        [
            PostingType::BikriFirst,
            PostingType::BikriLater,
            PostingType::Reversal,
            PostingType::Receipt,
            PostingType::Payment,
            PostingType::Advance,
            PostingType::ShopExpense,
            PostingType::PersonalExpense,
            PostingType::PersonalReturn,
            PostingType::MoneyLent,
            PostingType::MoneyLentReturn,
            PostingType::General,
            PostingType::Opening,
            PostingType::MonthClose,
        ]
        .into_iter()
        .find(|t| t.as_str() == text)
    }
}

/// What kind of record an entry belongs to (`journal_entries.source_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceType {
    /// A بکری.
    Bikri,
    /// A voucher.
    Voucher,
    /// An account (its opening).
    Account,
    /// A month close.
    MonthClose,
}

impl SourceType {
    /// The stored form.
    pub const fn as_str(self) -> &'static str {
        match self {
            SourceType::Bikri => "bikri",
            SourceType::Voucher => "voucher",
            SourceType::Account => "account",
            SourceType::MonthClose => "month_close",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<SourceType> {
        [
            SourceType::Bikri,
            SourceType::Voucher,
            SourceType::Account,
            SourceType::MonthClose,
        ]
        .into_iter()
        .find(|t| t.as_str() == text)
    }
}

/// Which side of the account a line stands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// بنام.
    Dr,
    /// جمع.
    Cr,
}

impl Side {
    /// The other side.
    pub const fn flip(self) -> Side {
        match self {
            Side::Dr => Side::Cr,
            Side::Cr => Side::Dr,
        }
    }
}

/// One line of an entry: an account (and for a پختہ کھاتہ account, the person), a side and a
/// positive amount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerLine {
    /// The account.
    pub account_id: AccountId,
    /// The person inside a پختہ کھاتہ account (PRD FR-13.5), if any.
    pub person_key: Option<String>,
    /// The side.
    pub side: Side,
    /// The amount, more than zero.
    pub amount: Paisa,
    /// A seller debit that counts as an advance (PRD FR-15.8). Only on a debit.
    pub is_advance: bool,
    /// A note for this line.
    pub narration: Option<String>,
}

impl LedgerLine {
    /// A line with no person, no advance flag and no note.
    pub fn new(account_id: AccountId, side: Side, amount: Paisa) -> Self {
        Self {
            account_id,
            person_key: None,
            side,
            amount,
            is_advance: false,
            narration: None,
        }
    }

    /// The same line for a person of a پختہ کھاتہ account.
    #[must_use]
    pub fn with_person(mut self, person_key: &str) -> Self {
        self.person_key = Some(person_key.to_owned());
        self
    }

    /// The same line, marked as an advance.
    #[must_use]
    pub fn as_advance(mut self) -> Self {
        self.is_advance = true;
        self
    }

    /// The same line with a note.
    #[must_use]
    pub fn with_narration(mut self, narration: &str) -> Self {
        self.narration = Some(narration.to_owned());
        self
    }
}

/// Why an entry could not be built. All of them mean a bug in the caller (an amount that was
/// not checked earlier), except `TooLarge`, which can come from data the user typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum LedgerError {
    /// No lines.
    #[error("an entry needs lines")]
    Empty,
    /// A line of zero.
    #[error("line {line} has no amount")]
    ZeroAmount {
        /// The line, from 1.
        line: usize,
    },
    /// A negative amount. Write it on the other side instead.
    #[error("line {line} is negative")]
    NegativeAmount {
        /// The line, from 1.
        line: usize,
    },
    /// An advance flag on a credit.
    #[error("line {line}: only a debit can be an advance")]
    AdvanceOnCredit {
        /// The line, from 1.
        line: usize,
    },
    /// Debits and credits differ. Nothing is written.
    #[error("entry does not balance: debits {dr:?}, credits {cr:?}")]
    Unbalanced {
        /// The debit total.
        dr: Paisa,
        /// The credit total.
        cr: Paisa,
    },
    /// A line or the total is above what a journal line can hold.
    #[error("amount is too large for a journal line")]
    TooLarge,
    /// A sum that does not fit in 64 bits.
    #[error("money overflow")]
    Overflow,
}

impl From<MoneyError> for LedgerError {
    fn from(_: MoneyError) -> Self {
        LedgerError::Overflow
    }
}

/// An entry that balances: the debits equal the credits, every line is positive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BalancedEntry {
    lines: Vec<LedgerLine>,
    total: Paisa,
}

impl BalancedEntry {
    /// The lines, in order.
    pub fn lines(&self) -> &[LedgerLine] {
        &self.lines
    }

    /// The debit total, which is also the credit total.
    pub const fn total(&self) -> Paisa {
        self.total
    }
}

/// Builds an entry. Checks that there are lines, that each amount is more than zero and within
/// a journal line's limit, that only debits are advances, and that the debits equal the credits.
///
/// # Errors
/// [`LedgerError`], the first problem found. `Unbalanced` carries both totals.
pub fn build_entry(lines: Vec<LedgerLine>) -> Result<BalancedEntry, LedgerError> {
    if lines.is_empty() {
        return Err(LedgerError::Empty);
    }
    let (mut dr, mut cr) = (Paisa::ZERO, Paisa::ZERO);
    for (index, line) in lines.iter().enumerate() {
        let number = index.saturating_add(1);
        if line.amount.is_negative() {
            return Err(LedgerError::NegativeAmount { line: number });
        }
        if line.amount == Paisa::ZERO {
            return Err(LedgerError::ZeroAmount { line: number });
        }
        if line.amount.as_paisa() > MAX_LINE_PAISA {
            return Err(LedgerError::TooLarge);
        }
        if line.is_advance && line.side == Side::Cr {
            return Err(LedgerError::AdvanceOnCredit { line: number });
        }
        match line.side {
            Side::Dr => dr = dr.checked_add(line.amount)?,
            Side::Cr => cr = cr.checked_add(line.amount)?,
        }
    }
    if dr != cr {
        return Err(LedgerError::Unbalanced { dr, cr });
    }
    if dr.as_paisa() > MAX_LINE_PAISA {
        return Err(LedgerError::TooLarge);
    }
    Ok(BalancedEntry { lines, total: dr })
}

/// The mirror of a posted entry: every line with its side swapped (PRD FR-6.3). A reversal is
/// how an entry is corrected; the original is never changed. An advance flag does not carry over
/// (the mirror of an advance debit is a credit).
///
/// Property: the entry plus its reversal is zero on every account.
///
/// ```
/// use mandi_domain::core::ids::AccountId;
/// use mandi_domain::core::money::Paisa;
/// use mandi_domain::ledger::{LedgerLine, Side, build_entry, build_reversal};
/// let p = Paisa::from_rupees(500).unwrap();
/// let entry = build_entry(vec![
///     LedgerLine::new(AccountId::new(1), Side::Dr, p),
///     LedgerLine::new(AccountId::new(2), Side::Cr, p),
/// ]).unwrap();
/// let back = build_reversal(&entry);
/// assert_eq!(back.lines()[0].side, Side::Cr);
/// assert_eq!(back.total(), entry.total());
/// ```
pub fn build_reversal(entry: &BalancedEntry) -> BalancedEntry {
    BalancedEntry {
        lines: entry
            .lines
            .iter()
            .map(|line| LedgerLine {
                account_id: line.account_id,
                person_key: line.person_key.clone(),
                side: line.side.flip(),
                amount: line.amount,
                is_advance: false,
                narration: line.narration.clone(),
            })
            .collect(),
        total: entry.total,
    }
}

/// The opening entry of an account (PRD FR-10.9): the account on its side and ابتدائی فرق on the
/// other, for the same amount. When every opening is typed right, ابتدائی فرق ends at zero.
///
/// Example (PRD 4.3): روکڑ opens with 65,000 بنام, so the entry is روکڑ Dr 65,000 and
/// ابتدائی فرق Cr 65,000.
///
/// # Errors
/// As [`build_entry`]: a zero or too large amount.
pub fn build_opening(
    account: AccountId,
    side: Side,
    amount: Paisa,
    is_advance: bool,
    difference_account: AccountId,
) -> Result<BalancedEntry, LedgerError> {
    let mut mine = LedgerLine::new(account, side, amount);
    mine.is_advance = is_advance;
    let other = LedgerLine::new(difference_account, side.flip(), amount);
    build_entry(vec![mine, other])
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    fn id(n: i64) -> AccountId {
        AccountId::new(n)
    }

    /// PRD 4.4, the sale part of بکری 1: 31,564 = 31,564.
    #[test]
    fn the_sale_part_of_bikri_1_balances_at_31564() {
        let dr = [(10, 12_880), (11, 18_684)];
        let cr = [
            (20, 24_316),
            (21, 1_764),
            (22, 1_764),
            (23, 400),
            (24, 20),
            (25, 250),
            (26, 3_000),
            (27, 50),
        ];
        let mut lines: Vec<LedgerLine> = dr
            .iter()
            .map(|(a, r)| LedgerLine::new(id(*a), Side::Dr, p(*r)))
            .collect();
        lines.extend(
            cr.iter()
                .map(|(a, r)| LedgerLine::new(id(*a), Side::Cr, p(*r))),
        );
        let entry = build_entry(lines).unwrap();
        assert_eq!(entry.total, p(31_564));
    }

    /// PRD 4.4: the whole entry with the cash part, 36,564 = 36,564; بکری 2: 25,600 = 25,600.
    #[test]
    fn the_whole_entries_of_the_prd_day_balance() {
        let mut lines = vec![
            LedgerLine::new(id(10), Side::Dr, p(12_880)),
            LedgerLine::new(id(11), Side::Dr, p(18_684)),
            LedgerLine::new(id(30), Side::Dr, p(5_000)),
        ];
        let credits = [24_316, 1_764, 1_764, 400, 20, 250, 3_000, 50];
        lines.extend(
            credits
                .iter()
                .enumerate()
                .map(|(i, r)| LedgerLine::new(id(40 + i64::try_from(i).unwrap()), Side::Cr, p(*r))),
        );
        lines.push(LedgerLine::new(id(50), Side::Cr, p(5_000)));
        assert_eq!(build_entry(lines).unwrap().total, p(36_564));
        let two = build_entry(vec![
            LedgerLine::new(id(12), Side::Dr, p(12_800)),
            LedgerLine::new(id(41), Side::Cr, p(10_160)),
            LedgerLine::new(id(42), Side::Cr, p(720)),
            LedgerLine::new(id(43), Side::Cr, p(720)),
            LedgerLine::new(id(44), Side::Cr, p(80)),
            LedgerLine::new(id(45), Side::Cr, p(20)),
            LedgerLine::new(id(46), Side::Cr, p(50)),
            LedgerLine::new(id(47), Side::Cr, p(1_000)),
            LedgerLine::new(id(48), Side::Cr, p(50)),
            LedgerLine::new(id(50), Side::Dr, p(12_800)),
            LedgerLine::new(id(12), Side::Cr, p(12_800)),
        ])
        .unwrap();
        assert_eq!(two.total, p(25_600));
    }

    /// The ledger journal that was off by 2 rupees in the old software (PRD 1.1).
    #[test]
    fn debit_100_credit_90_is_refused_with_both_totals() {
        let result = build_entry(vec![
            LedgerLine::new(id(1), Side::Dr, p(100)),
            LedgerLine::new(id(2), Side::Cr, p(90)),
        ]);
        assert_eq!(
            result,
            Err(LedgerError::Unbalanced {
                dr: p(100),
                cr: p(90)
            })
        );
    }

    #[test]
    fn bad_lines_are_refused() {
        assert_eq!(build_entry(Vec::new()), Err(LedgerError::Empty));
        let zero = vec![
            LedgerLine::new(id(1), Side::Dr, Paisa::ZERO),
            LedgerLine::new(id(2), Side::Cr, Paisa::ZERO),
        ];
        assert_eq!(build_entry(zero), Err(LedgerError::ZeroAmount { line: 1 }));
        let negative = vec![LedgerLine::new(id(1), Side::Dr, p(-100))];
        assert_eq!(
            build_entry(negative),
            Err(LedgerError::NegativeAmount { line: 1 })
        );
        let advance_credit = vec![
            LedgerLine::new(id(1), Side::Dr, p(100)),
            LedgerLine::new(id(2), Side::Cr, p(100)).as_advance(),
        ];
        assert_eq!(
            build_entry(advance_credit),
            Err(LedgerError::AdvanceOnCredit { line: 2 })
        );
    }

    #[test]
    fn the_biggest_line_is_allowed_and_one_more_is_too_large() {
        // 1e14 paisa = 1e12 rupees.
        let top = Paisa::try_from_paisa(MAX_LINE_PAISA).unwrap();
        assert!(
            build_entry(vec![
                LedgerLine::new(id(1), Side::Dr, top),
                LedgerLine::new(id(2), Side::Cr, top),
            ])
            .is_ok()
        );
        let over = Paisa::try_from_paisa(MAX_LINE_PAISA + 100).unwrap();
        assert_eq!(
            build_entry(vec![
                LedgerLine::new(id(1), Side::Dr, over),
                LedgerLine::new(id(2), Side::Cr, over),
            ]),
            Err(LedgerError::TooLarge)
        );
        // Two lines under the cap whose total is above it.
        let a = Paisa::try_from_paisa(MAX_LINE_PAISA).unwrap();
        let b = Paisa::try_from_paisa(100).unwrap();
        assert_eq!(
            build_entry(vec![
                LedgerLine::new(id(1), Side::Dr, a),
                LedgerLine::new(id(2), Side::Dr, b),
                LedgerLine::new(id(3), Side::Cr, a.checked_add(b).unwrap()),
            ]),
            Err(LedgerError::TooLarge)
        );
        // Sums beyond i64 are an overflow error, never a wrap.
        let huge = Paisa::try_from_paisa(i64::MAX - i64::MAX % 100).unwrap();
        let lines = vec![
            LedgerLine::new(id(1), Side::Dr, huge),
            LedgerLine::new(id(2), Side::Dr, huge),
        ];
        assert!(matches!(
            build_entry(lines),
            Err(LedgerError::TooLarge | LedgerError::Overflow)
        ));
    }

    #[test]
    fn an_opening_is_the_account_and_the_difference_account_on_opposite_sides() {
        // PRD 4.3: روکڑ 65,000 بنام against ابتدائی فرق.
        let cash = build_opening(id(1), Side::Dr, p(65_000), false, id(9)).unwrap();
        assert_eq!(cash.total, p(65_000));
        assert_eq!(
            (cash.lines[0].side, cash.lines[1].side),
            (Side::Dr, Side::Cr)
        );
        assert_eq!(cash.lines[1].account_id, id(9));
        // ابتدائی سرمایہ 65,000 جمع is the mirror.
        let capital = build_opening(id(2), Side::Cr, p(65_000), false, id(9)).unwrap();
        assert_eq!(
            (capital.lines[0].side, capital.lines[1].side),
            (Side::Cr, Side::Dr)
        );
        // A seller's opening debit as an advance (رحیم 15,000).
        let rahim = build_opening(id(3), Side::Dr, p(15_000), true, id(9)).unwrap();
        assert!(rahim.lines[0].is_advance && !rahim.lines[1].is_advance);
        // An advance on a credit opening cannot be built.
        assert_eq!(
            build_opening(id(3), Side::Cr, p(15_000), true, id(9)),
            Err(LedgerError::AdvanceOnCredit { line: 1 })
        );
        assert_eq!(
            build_opening(id(3), Side::Dr, Paisa::ZERO, false, id(9)),
            Err(LedgerError::ZeroAmount { line: 1 })
        );
    }

    #[test]
    fn a_reversal_swaps_every_side_keeps_people_and_drops_the_advance_flag() {
        let entry = build_entry(vec![
            LedgerLine::new(id(1), Side::Dr, p(15_000)).as_advance(),
            LedgerLine::new(id(2), Side::Cr, p(15_000)).with_person("shahid|0300"),
        ])
        .unwrap();
        let back = build_reversal(&entry);
        assert_eq!(
            (back.lines[0].side, back.lines[1].side),
            (Side::Cr, Side::Dr)
        );
        assert!(back.lines.iter().all(|l| !l.is_advance));
        assert_eq!(back.lines[1].person_key.as_deref(), Some("shahid|0300"));
        assert_eq!(back.total, entry.total);
        // A reversal of a reversal has the original sides again.
        assert_eq!(build_reversal(&back).lines[0].side, Side::Dr);
    }

    fn arb_line() -> impl Strategy<Value = (i64, bool, i64)> {
        (0_i64..6, any::<bool>(), 1_i64..=1_000_000)
    }

    fn net_by_account(entries: &[&BalancedEntry]) -> BTreeMap<(i64, Option<String>), i128> {
        let mut net: BTreeMap<(i64, Option<String>), i128> = BTreeMap::new();
        for entry in entries {
            for line in &entry.lines {
                let amount = i128::from(line.amount.as_paisa());
                let signed = if line.side == Side::Dr {
                    amount
                } else {
                    -amount
                };
                *net.entry((line.account_id.get(), line.person_key.clone()))
                    .or_default() += signed;
            }
        }
        net
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        /// Whatever lines come in, an entry that is built balances; one that does not balance
        /// is refused, never built. It never panics.
        #[test]
        fn built_entries_always_balance(lines in proptest::collection::vec(arb_line(), 0..12)) {
            let ledger: Vec<LedgerLine> = lines
                .iter()
                .map(|(account, dr, rupees)| {
                    LedgerLine::new(id(*account), if *dr { Side::Dr } else { Side::Cr }, p(*rupees))
                })
                .collect();
            match build_entry(ledger.clone()) {
                Ok(entry) => {
                    let (mut dr, mut cr) = (0_i128, 0_i128);
                    for l in &entry.lines {
                        let a = i128::from(l.amount.as_paisa());
                        if l.side == Side::Dr { dr += a } else { cr += a }
                    }
                    prop_assert_eq!(dr, cr);
                    prop_assert_eq!(dr, i128::from(entry.total.as_paisa()));
                }
                Err(LedgerError::Unbalanced { dr, cr }) => prop_assert_ne!(dr, cr),
                Err(LedgerError::Empty) => prop_assert!(ledger.is_empty()),
                Err(other) => prop_assert!(false, "{other:?}"),
            }
        }

        /// An entry plus its reversal is zero on every account (money agent 11.2, property 4).
        #[test]
        fn an_entry_and_its_reversal_make_zero_on_every_account(
            debits in proptest::collection::vec((0_i64..6, 1_i64..=1_000_000), 1..8),
            split in proptest::collection::vec(0_i64..6, 1..8),
        ) {
            // Credits: the same total spread over the chosen accounts (the last takes the rest).
            let total: i64 = debits.iter().map(|(_, r)| *r).sum();
            let each = total / i64::try_from(split.len()).unwrap();
            let mut lines: Vec<LedgerLine> = debits
                .iter()
                .map(|(a, r)| LedgerLine::new(id(*a), Side::Dr, p(*r)))
                .collect();
            let mut left = total;
            for (i, account) in split.iter().enumerate() {
                let amount = if i + 1 == split.len() { left } else { each };
                left -= amount;
                if amount > 0 {
                    lines.push(LedgerLine::new(id(*account), Side::Cr, p(amount)));
                }
            }
            let entry = build_entry(lines).unwrap();
            let back = build_reversal(&entry);
            for (key, net) in net_by_account(&[&entry, &back]) {
                prop_assert_eq!(net, 0, "{:?}", key);
            }
        }

        /// The same with person keys on the lines and advance flags on debits: a reversal
        /// nets to zero per account and per person, and carries no advance flag.
        #[test]
        fn a_reversal_nets_to_zero_per_person_and_drops_the_advance_flag(
            amounts in proptest::collection::vec((0_i64..4, proptest::option::of(0_u8..3), any::<bool>(), 1_i64..=500_000), 1..8),
        ) {
            let mut lines = Vec::new();
            let mut total = 0_i64;
            for (account, person, advance, rupees) in &amounts {
                let mut line = LedgerLine::new(id(*account), Side::Dr, p(*rupees));
                line.person_key = person.map(|n| format!("person{n}"));
                line.is_advance = *advance;
                lines.push(line);
                total += rupees;
            }
            lines.push(LedgerLine::new(id(99), Side::Cr, p(total)));
            let entry = build_entry(lines).unwrap();
            let back = build_reversal(&entry);
            prop_assert!(back.lines().iter().all(|l| !l.is_advance));
            prop_assert_eq!(back.total(), entry.total());
            for (key, net) in net_by_account(&[&entry, &back]) {
                prop_assert_eq!(net, 0, "{:?}", key);
            }
        }

        /// An opening is two lines on opposite sides for the same amount, and the advance flag
        /// is accepted only when the account is on the debit side.
        #[test]
        fn an_opening_is_two_balanced_lines(
            rupees in 1_i64..=1_000_000_000,
            debit in any::<bool>(),
            advance in any::<bool>(),
        ) {
            let side = if debit { Side::Dr } else { Side::Cr };
            match build_opening(id(1), side, p(rupees), advance, id(2)) {
                Ok(entry) => {
                    prop_assert!(!advance || debit);
                    prop_assert_eq!(entry.lines().len(), 2);
                    prop_assert_eq!(entry.total(), p(rupees));
                    prop_assert_eq!(entry.lines()[0].side, side);
                    prop_assert_eq!(entry.lines()[1].side, side.flip());
                    prop_assert_eq!(entry.lines()[1].account_id, id(2));
                }
                Err(LedgerError::AdvanceOnCredit { .. }) => prop_assert!(advance && !debit),
                Err(other) => prop_assert!(false, "{other:?}"),
            }
        }
    }

    /// The edges of a line: zero and negative are refused, the limit itself is allowed and one
    /// rupee more is not (the limit is the CHECK of `journal_lines`).
    #[test]
    fn the_edges_of_a_line_amount() {
        let pair = |amount: Paisa| {
            build_entry(vec![
                LedgerLine::new(id(1), Side::Dr, amount),
                LedgerLine::new(id(2), Side::Cr, amount),
            ])
        };
        assert!(matches!(
            pair(Paisa::ZERO),
            Err(LedgerError::ZeroAmount { line: 1 })
        ));
        let negative = Paisa::try_from_paisa(-100).unwrap();
        assert!(matches!(
            pair(negative),
            Err(LedgerError::NegativeAmount { line: 1 })
        ));
        let top = Paisa::try_from_paisa(MAX_LINE_PAISA).unwrap();
        assert_eq!(pair(top).unwrap().total(), top);
        let over = Paisa::try_from_paisa(MAX_LINE_PAISA + 100).unwrap();
        assert_eq!(pair(over), Err(LedgerError::TooLarge));
    }

    /// An entry whose lines are each within the limit but whose total is not is refused too (the
    /// total is what `journal_entries.total_paisa` holds).
    #[test]
    fn the_total_of_an_entry_has_a_limit_too() {
        let big = Paisa::try_from_paisa(MAX_LINE_PAISA / 4 * 3).unwrap();
        let too_big = build_entry(vec![
            LedgerLine::new(id(1), Side::Dr, big),
            LedgerLine::new(id(2), Side::Dr, big),
            LedgerLine::new(id(3), Side::Cr, big),
            LedgerLine::new(id(4), Side::Cr, big),
        ]);
        assert_eq!(too_big, Err(LedgerError::TooLarge));
        // Right at the limit in total is fine: two halves of it.
        let half = Paisa::try_from_paisa(MAX_LINE_PAISA / 2).unwrap();
        let top = build_entry(vec![
            LedgerLine::new(id(1), Side::Dr, half),
            LedgerLine::new(id(2), Side::Dr, half),
            LedgerLine::new(id(3), Side::Cr, half),
            LedgerLine::new(id(4), Side::Cr, half),
        ])
        .unwrap();
        assert_eq!(top.total().as_paisa(), MAX_LINE_PAISA);
    }

    /// The stored form of every posting type and source type reads back to itself, and a word
    /// that is not stored reads as nothing (`journal_entries.posting_type`, `source_type`).
    #[test]
    fn the_stored_words_of_posting_and_source_types_round_trip() {
        let postings = [
            (PostingType::BikriFirst, "bikri_first"),
            (PostingType::BikriLater, "bikri_later"),
            (PostingType::Reversal, "reversal"),
            (PostingType::Receipt, "receipt"),
            (PostingType::Payment, "payment"),
            (PostingType::Advance, "advance"),
            (PostingType::ShopExpense, "shop_expense"),
            (PostingType::PersonalExpense, "personal_expense"),
            (PostingType::PersonalReturn, "personal_return"),
            (PostingType::MoneyLent, "money_lent"),
            (PostingType::MoneyLentReturn, "money_lent_return"),
            (PostingType::General, "general"),
            (PostingType::Opening, "opening"),
            (PostingType::MonthClose, "month_close"),
        ];
        for (kind, word) in postings {
            assert_eq!(kind.as_str(), word);
            assert_eq!(PostingType::parse(word), Some(kind));
        }
        assert_eq!(PostingType::parse("Receipt"), None);
        assert_eq!(PostingType::parse(""), None);
        let sources = [
            (SourceType::Bikri, "bikri"),
            (SourceType::Voucher, "voucher"),
            (SourceType::Account, "account"),
            (SourceType::MonthClose, "month_close"),
        ];
        for (kind, word) in sources {
            assert_eq!(kind.as_str(), word);
            assert_eq!(SourceType::parse(word), Some(kind));
        }
        assert_eq!(SourceType::parse("sale"), None);
    }

    /// A note and the advance mark are carried on the line, and a money overflow becomes the
    /// ledger's own overflow error.
    #[test]
    fn a_line_can_carry_a_note_and_an_advance_mark_and_an_overflow_has_its_own_error() {
        let line = LedgerLine::new(id(1), Side::Dr, p(15_000))
            .as_advance()
            .with_narration("opening advance");
        assert!(line.is_advance);
        assert_eq!(line.narration.as_deref(), Some("opening advance"));
        let credit = LedgerLine::new(id(2), Side::Cr, p(15_000));
        let entry = build_entry(vec![line, credit]).unwrap();
        assert_eq!(
            entry.lines()[0].narration.as_deref(),
            Some("opening advance")
        );
        assert_eq!(
            LedgerError::from(MoneyError::Overflow),
            LedgerError::Overflow
        );
        // Each line is held to the limit before any sum is made, so a huge line is `TooLarge`;
        // `Overflow` comes only from the checked additions of `MoneyError`.
        let huge = Paisa::try_from_paisa(i64::MAX - i64::MAX % 100).unwrap();
        assert_eq!(
            build_entry(vec![
                LedgerLine::new(id(1), Side::Dr, huge),
                LedgerLine::new(id(2), Side::Cr, huge),
            ]),
            Err(LedgerError::TooLarge)
        );
    }
}
