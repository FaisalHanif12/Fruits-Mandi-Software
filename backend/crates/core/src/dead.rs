//! The dead amount (ڈوبی رقم, PRD FR-10.10, plan 40): what the screens send and read back when a
//! person who owes the shop is moved out of the daily lists, and the report R38.
//!
//! A person who owes the shop may die or leave the business. Nothing posts: his balance stays on
//! his account, so the trial balance and every partner's کل رقم stay as they were (PRD A65). The
//! move only sorts him into a group, and the group is what `dead_marks` holds. What he still owes
//! is always his balance read from the journal; the table keeps only the amount on the day of the
//! move, for the history (Architecture 15.4).
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `masters_dead_preview` | [`DeadPreviewInput`] | [`DeadPreview`] |
//! | `masters_dead_mark` | [`DeadMarkInput`] | [`DeadMarked`] |
//! | `masters_dead_undo` | [`DeadUndoInput`] | [`DeadUndone`] |
//! | `reports_dead_amount` | [`DeadAmountInput`] | [`DeadAmountPage`] (R38) |
//!
//! Who can be moved (FR-10.10): a buyer; a seller whose ledger balance is بنام; a پختہ کھاتہ
//! person, by his person key (FR-13.5), where nothing is made inactive and only the key is marked;
//! a person with ادھار رقم (control 0103). Partners, الگ کھاتے, gardens and system accounts are
//! never moved (`ERR_DEAD_NOT_ALLOWED`). The amount is his whole ledger balance: for a seller his
//! open advance and open payables are netted into it (PRD A66).
//!
//! Money words (the money agent, Section 8):
//!
//! - **Owed** = Σ debit − Σ credit of his account (or of his person key). It must be above 0, else
//!   `ERR_NOTHING_OWED`.
//! - **Recovered** = what the credit lines of a وصولی or an ادھار رقم واپسی (and a seller's
//!   وصولی, FR-15.12) put on him since the move, less the reversal of a recovery that was cancelled
//!   or edited: a recovery of 4,000 that is cancelled shows 0, one edited to 3,000 shows 3,000
//!   (PRD FR-10.10).
//! - **Corrections** = the rest of the change since the move: an edit, cancel or undo of an
//!   earlier posting (PRD A67), including a recovery saved before the move. Positive means he owes
//!   more.
//! - **Still owed** = his balance now = owed then − recovered + corrections, never below 0. If the
//!   shop comes to owe him (a credit balance), still owed is 0 and the status is `credit`.
//!
//! Worked numbers (PRD E2E-44, after Events 1 to 5): ارشد owes 18,684 on 11-10-26 and is moved. His
//! son pays 4,000 on 12-10-26. R38 shows owed then 18,684, recovered 4,000, corrections 0, still
//! owed 14,684. The tiles: گاہکوں کے ذمے 2,880, بیوپاریوں کے ذمے 0, ڈوبی رقم 14,684, کل اگراہی
//! 17,564.
//!
//! Field paths in errors: `account_id`, `pukhta_person_key`, `entry_date`, `reason`, `note`;
//! `mark_id` and `reason` for an undo.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::Balance;
use crate::error::ErrorCode;
use crate::money::{MoneyError, Paisa};

/// Why a person was moved to the dead amount (PRD FR-10.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DeadReason {
    /// فوت ہو گئے: he died.
    Died,
    /// کاروبار چھوڑ دیا: he left the business.
    Left,
    /// دیگر: any other reason. A note is required (`ERR_NOTE_REQUIRED` on `note`).
    Other,
}

impl DeadReason {
    /// The wire form, also what `dead_marks.reason` stores.
    pub const fn as_str(self) -> &'static str {
        match self {
            DeadReason::Died => "died",
            DeadReason::Left => "left",
            DeadReason::Other => "other",
        }
    }
}

/// What kind of person a row of R38 is (PRD R38: گاہک, بیوپاری, پختہ کھاتہ, ادھار رقم).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DeadPartyType {
    /// گاہک, an account under 0304.
    Buyer,
    /// بیوپاری, an account under 03011.
    Seller,
    /// پختہ کھاتہ: one one-time person, by his person key. The account itself stays active.
    Pukhta,
    /// ادھار رقم: a person under control 0103.
    MoneyLent,
}

impl DeadPartyType {
    /// The wire form.
    pub const fn as_str(self) -> &'static str {
        match self {
            DeadPartyType::Buyer => "buyer",
            DeadPartyType::Seller => "seller",
            DeadPartyType::Pukhta => "pukhta",
            DeadPartyType::MoneyLent => "money_lent",
        }
    }
}

/// The move of one person, as the party card and the ledger heading show it (PRD FR-7.38,
/// FR-10.10): "ڈوبی رقم" with the date and the reason. Null on a person who is not in the dead
/// amount.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DeadInfo {
    /// The move, to undo it.
    #[ts(type = "number")]
    pub mark_id: i64,
    /// The day of the move, `YYYY-MM-DD`.
    pub entry_date: String,
    /// Why.
    pub reason: DeadReason,
    /// The note typed with the move. Empty if none.
    pub note: String,
}

/// An account linked to the person (PRD E10). It is not moved with him (A68); the dialog shows its
/// balance so the Owner can decide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DeadLinked {
    /// Row id.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The full code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The tag (نشان), or null.
    pub tag: Option<String>,
    /// His balance now, with its side.
    pub balance: Balance,
}

/// Asks what moving a person would do (the "ڈوبی رقم میں ڈالیں" dialog before save). Read only.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DeadPreviewInput {
    /// The account.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The person inside a پختہ کھاتہ account, as stored. Required for a پختہ کھاتہ account and
    /// refused for any other.
    #[serde(default)]
    pub pukhta_person_key: Option<String>,
}

/// What moving a person would do.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DeadPreview {
    /// What he owes now: his whole ledger balance بنام, 0 if he owes nothing (PRD A66).
    #[ts(type = "Paisa")]
    pub owed_paisa: Paisa,
    /// False when he cannot be moved. `reason_not_allowed` says why.
    pub allowed: bool,
    /// The code that says why not: `ERR_NOTHING_OWED`, `ERR_DEAD_NOT_ALLOWED`, `ERR_ALREADY_DEAD`
    /// or `ERR_PERSON_IN_DEAD_AMOUNT`. Null when he can be moved. A code, never text (the plan
    /// listed a string; a code lets the screen show it in the app language).
    pub reason_not_allowed: Option<ErrorCode>,
    /// The day of his last posting, `YYYY-MM-DD`, or null if none. The move may not be dated before
    /// it (`ERR_DATE_BEFORE_LAST_POSTING`).
    pub last_posting_date: Option<String>,
    /// His linked account and its balance, if he has one (PRD E10, A68). Null if none.
    pub linked: Option<DeadLinked>,
}

/// Moves a person to the dead amount (PRD FR-10.10).
///
/// Errors: `ERR_NOT_FOUND` (`account_id`); `ERR_NOTHING_OWED` (`account_id`); `ERR_DEAD_NOT_ALLOWED`
/// (`account_id`: a partner, a الگ کھاتہ, a garden or a system account); `ERR_ALREADY_DEAD`
/// (`account_id`); `ERR_PERSON_IN_DEAD_AMOUNT` (`pukhta_person_key`, when already marked);
/// `ERR_PERSON_REQUIRED` (`pukhta_person_key`); `ERR_NOTE_REQUIRED` (`note`, when the reason is
/// `other`); the open-day codes on `entry_date` (`ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE`) and
/// `ERR_DATE_BEFORE_LAST_POSTING` (value `last_posting_date`).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DeadMarkInput {
    /// The account.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The person inside a پختہ کھاتہ account. Required for a پختہ کھاتہ account.
    #[serde(default)]
    pub pukhta_person_key: Option<String>,
    /// The day of the move, as typed (`DD-MM-YYYY` or a shortcut). An open date.
    pub entry_date: String,
    /// Why.
    pub reason: DeadReason,
    /// A note, as typed. Required when the reason is `other`.
    #[serde(default)]
    pub note: String,
}

/// A person moved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DeadMarked {
    /// The move.
    #[ts(type = "number")]
    pub mark_id: i64,
    /// What he owed on that day, the amount kept in the history.
    #[ts(type = "Paisa")]
    pub owed_paisa: Paisa,
}

/// Makes a person active again ("دوبارہ فعال کریں", PRD FR-10.10): the move ends, the account (or
/// the person key) is active, and his balance goes back into the normal tiles and lists.
///
/// Errors: `ERR_NOT_FOUND` (`mark_id`, also for a move already undone); `ERR_REASON_REQUIRED`
/// (`reason`).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DeadUndoInput {
    /// The move.
    #[ts(type = "number")]
    pub mark_id: i64,
    /// Why he is made active again. Required.
    pub reason: String,
}

/// A move undone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DeadUndone {
    /// Always true; a refusal is an error, not `false`.
    pub ok: bool,
}

/// Which moves R38 lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DeadFilter {
    /// Only the people who still owe something (the default; the tile ڈوبی رقم adds these up).
    #[default]
    Owed,
    /// Every move: also the fully recovered ones and the ones made active again.
    All,
}

/// Where a move stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DeadStatus {
    /// He still owes something and is in the dead amount.
    Owing,
    /// He owes 0 now ("پوری وصول ہو گئی"). The move stays in the history.
    Recovered,
    /// The move was undone: he is active again. Its figures stop on the day of the undo and its
    /// still owed is 0, so the total of the still owed equals the tile.
    Undone,
    /// His balance is now a credit: the shop owes him (for example an earlier sale of his was
    /// cancelled after a part was recovered). Still owed is 0 and he is in no tile.
    Credit,
}

/// Asks for a page of R38 (Code Standards B31, PRD FR-19.11).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DeadAmountInput {
    /// Which moves. Missing means `owed`.
    #[serde(default)]
    pub filter: DeadFilter,
    /// Text to find in the name (either script), the code, the tag or the phone. Empty for all.
    #[serde(default)]
    pub search: String,
    /// Where the page starts. Missing means 0.
    #[serde(default)]
    #[ts(type = "number")]
    pub offset: i64,
    /// How many rows, 1 to 100. Missing means 100.
    #[serde(default = "default_limit")]
    #[ts(type = "number")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    100
}

/// One row of R38: one move.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DeadRow {
    /// The move, to undo it.
    #[ts(type = "number")]
    pub mark_id: i64,
    /// The account. For a پختہ کھاتہ person it is the پختہ account.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The full code.
    pub code: String,
    /// Urdu name of the account.
    pub name_ur: String,
    /// English name of the account.
    pub name_en: String,
    /// The tag (نشان), or null.
    pub tag: Option<String>,
    /// What kind of person.
    pub party_type: DeadPartyType,
    /// The person key, for a پختہ کھاتہ person. Null for any other.
    pub pukhta_person_key: Option<String>,
    /// His Urdu name as saved on his lines, for a پختہ کھاتہ person. Null for any other.
    pub pukhta_name_ur: Option<String>,
    /// His English name, for a پختہ کھاتہ person. Null for any other.
    pub pukhta_name_en: Option<String>,
    /// Why he was moved.
    pub reason: DeadReason,
    /// The note typed with the move.
    pub note: String,
    /// The day of the move, `YYYY-MM-DD`.
    pub entry_date: String,
    /// What he owed on that day.
    #[ts(type = "Paisa")]
    pub owed_then_paisa: Paisa,
    /// What was recovered since: a وصولی or an ادھار رقم واپسی, less the part that was cancelled or
    /// edited away (a cancelled 4,000 counts 0, one edited to 3,000 counts 3,000).
    #[ts(type = "Paisa")]
    pub recovered_paisa: Paisa,
    /// Edits, cancels and undos of earlier postings since the move (PRD A67), also a recovery saved
    /// before the move, signed: above 0 means he owes more, below 0 less. Not a recovery.
    #[ts(type = "number")]
    pub corrections_paisa: i64,
    /// What he owes now: his balance (owed then − recovered + corrections). 0 when fully recovered.
    #[ts(type = "Paisa")]
    pub still_owed_paisa: Paisa,
    /// The day of his last recovery, `YYYY-MM-DD`, or null if none.
    pub last_recovery_date: Option<String>,
    /// Where the move stands.
    pub status: DeadStatus,
}

/// The totals of R38 over the whole filter, never a sum of the page (Code Standards B31).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct DeadTotals {
    /// Σ owed then.
    #[ts(type = "Paisa")]
    pub owed_then_paisa: Paisa,
    /// Σ recovered.
    #[ts(type = "Paisa")]
    pub recovered_paisa: Paisa,
    /// Σ corrections, signed.
    #[ts(type = "number")]
    pub corrections_paisa: i64,
    /// Σ still owed. With the filter `owed` it equals the tile ڈوبی رقم (PRD FR-19.9).
    #[ts(type = "Paisa")]
    pub still_owed_paisa: Paisa,
}

/// A page of R38 with its count and the totals over the whole filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DeadAmountPage {
    /// The rows of this page, oldest move first.
    pub rows: Vec<DeadRow>,
    /// How many rows the whole filter has.
    #[ts(type = "number")]
    pub row_count: i64,
    /// The totals over the whole filter.
    pub totals: DeadTotals,
}

/// The longest reason text an undo may carry (a note is free text, up to 500 characters).
pub const MAX_REASON_CHARS: usize = 500;

/// What a person owes now: his balance `Σ debit − Σ credit`, never below 0 (PRD FR-10.10). A
/// credit balance means the shop owes him, which is not part of the dead amount.
///
/// # Errors
/// `MoneyError::Overflow` if the subtraction does not fit.
///
/// # Example
/// ```
/// use mandi_core::dead::owed_now;
/// use mandi_core::money::Paisa;
/// let rupees = |n| Paisa::from_rupees(n).unwrap();
/// assert_eq!(owed_now(rupees(22_000), rupees(3_316)).unwrap(), rupees(18_684));
/// assert_eq!(owed_now(rupees(1_000), rupees(4_000)).unwrap(), Paisa::ZERO);
/// ```
pub fn owed_now(balance_dr: Paisa, balance_cr: Paisa) -> Result<Paisa, MoneyError> {
    let balance = balance_dr.checked_sub(balance_cr)?;
    Ok(if balance.is_negative() {
        Paisa::ZERO
    } else {
        balance
    })
}

/// The money of one move: what was recovered, the corrections, what is still owed, as R38 and the
/// tile show them (PRD FR-10.10, plan 40). Pure: the services pass in the sums read from the
/// journal, and the identity `still = owed then − recovered + corrections` holds by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeadMoney {
    /// What the recovery entries since the move put on his credit.
    pub recovered: Paisa,
    /// Every other movement since the move, signed: above 0 he owes more.
    pub corrections: Paisa,
    /// What he owes now, never below 0.
    pub still_owed: Paisa,
    /// True when his balance is a credit (the shop owes him).
    pub is_credit: bool,
}

/// Works out the money of one move.
///
/// `owed_then` is what he owed on the day of the move; `balance_dr` and `balance_cr` are the sums
/// of all his posted lines now; `recovery_dr` and `recovery_cr` are the sums of the recovery
/// entries since the move (a `receipt` or `money_lent_return`) and of the reversals of those, so a
/// cancelled recovery nets to 0 (PRD FR-10.10).
///
/// # Errors
/// `MoneyError::Overflow` if a sum does not fit.
///
/// # Example
/// PRD E2E-44: ارشد owed 18,684 and 4,000 was recovered, so 14,684 is owed and nothing was
/// corrected.
/// ```
/// use mandi_core::dead::dead_money;
/// use mandi_core::money::Paisa;
/// let r = |n| Paisa::from_rupees(n).unwrap();
/// let money = dead_money(r(18_684), r(18_684), r(4_000), Paisa::ZERO, r(4_000)).unwrap();
/// assert_eq!(money.recovered, r(4_000));
/// assert_eq!(money.corrections, Paisa::ZERO);
/// assert_eq!(money.still_owed, r(14_684));
/// ```
pub fn dead_money(
    owed_then: Paisa,
    balance_dr: Paisa,
    balance_cr: Paisa,
    recovery_dr: Paisa,
    recovery_cr: Paisa,
) -> Result<DeadMoney, MoneyError> {
    let balance = balance_dr.checked_sub(balance_cr)?;
    let still_owed = owed_now(balance_dr, balance_cr)?;
    let recovered = recovery_cr.checked_sub(recovery_dr)?;
    // still = owed then − recovered + corrections, so the corrections are what is left over. They
    // are the sum of the other lines since the move, until the balance goes below 0 and the still
    // owed is held at 0.
    let corrections = still_owed.checked_sub(owed_then)?.checked_add(recovered)?;
    Ok(DeadMoney {
        recovered,
        corrections,
        still_owed,
        is_credit: balance.is_negative(),
    })
}

/// Where a move stands (PRD FR-10.10): undone, owing, fully recovered, or a credit.
pub fn dead_status(is_active: bool, money: &DeadMoney) -> DeadStatus {
    if !is_active {
        DeadStatus::Undone
    } else if money.is_credit {
        DeadStatus::Credit
    } else if money.still_owed == Paisa::ZERO {
        DeadStatus::Recovered
    } else {
        DeadStatus::Owing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn r(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    #[test]
    fn e2e_44_after_the_move_and_after_the_receipt() {
        // 11-10-26: moved, nothing since.
        let m = dead_money(r(18_684), r(18_684), Paisa::ZERO, Paisa::ZERO, Paisa::ZERO).unwrap();
        assert_eq!(
            (m.recovered, m.corrections, m.still_owed),
            (Paisa::ZERO, Paisa::ZERO, r(18_684))
        );
        assert_eq!(dead_status(true, &m), DeadStatus::Owing);
        // 12-10-26: a receipt of 4,000.
        let m = dead_money(r(18_684), r(18_684), r(4_000), Paisa::ZERO, r(4_000)).unwrap();
        assert_eq!(
            (m.recovered, m.corrections, m.still_owed),
            (r(4_000), Paisa::ZERO, r(14_684))
        );
    }

    #[test]
    fn a_cancelled_recovery_counts_as_nothing_recovered() {
        // The receipt of 4,000 (a credit) and its reversal (a debit) are both recovery lines, so
        // they net to 0: recovered 0, no correction, he owes 18,684 again.
        let m = dead_money(r(18_684), r(22_684), r(4_000), r(4_000), r(4_000)).unwrap();
        assert_eq!(m.recovered, Paisa::ZERO);
        assert_eq!(m.corrections, Paisa::ZERO);
        assert_eq!(m.still_owed, r(18_684));
    }

    #[test]
    fn a_recovery_edited_from_4000_to_3000_counts_3000() {
        // The old receipt (credit 4,000), its reversal (debit 4,000) and the new receipt (credit
        // 3,000): recovery credit 7,000, recovery debit 4,000.
        let m = dead_money(r(18_684), r(22_684), r(7_000), r(4_000), r(7_000)).unwrap();
        assert_eq!(m.recovered, r(3_000));
        assert_eq!(m.corrections, Paisa::ZERO);
        assert_eq!(m.still_owed, r(15_684));
    }

    #[test]
    fn recovering_exactly_all_leaves_zero_and_one_more_rupee_is_a_credit() {
        let m = dead_money(r(18_684), r(18_684), r(18_684), Paisa::ZERO, r(18_684)).unwrap();
        assert_eq!(m.still_owed, Paisa::ZERO);
        assert!(!m.is_credit);
        assert_eq!(dead_status(true, &m), DeadStatus::Recovered);
        let m = dead_money(r(18_684), r(18_684), r(18_685), Paisa::ZERO, r(18_685)).unwrap();
        assert_eq!(m.still_owed, Paisa::ZERO);
        assert!(m.is_credit);
        assert_eq!(dead_status(true, &m), DeadStatus::Credit);
    }

    #[test]
    fn an_earlier_sale_cancelled_after_a_part_was_recovered_goes_to_a_credit() {
        // Owed 18,684; 4,000 recovered; his whole 18,684 sale is cancelled (a credit of 18,684):
        // balance 18,684 − 4,000 − 18,684 = −4,000. Still owed 0, corrections −14,684.
        let m = dead_money(r(18_684), r(18_684), r(22_684), Paisa::ZERO, r(4_000)).unwrap();
        assert_eq!(m.still_owed, Paisa::ZERO);
        assert!(m.is_credit);
        assert_eq!(m.corrections, r(-14_684));
    }

    #[test]
    fn an_undone_move_is_undone_whatever_the_money() {
        let m = dead_money(r(100), r(100), Paisa::ZERO, Paisa::ZERO, Paisa::ZERO).unwrap();
        assert_eq!(dead_status(false, &m), DeadStatus::Undone);
    }

    #[test]
    fn overflow_is_an_error_not_a_wrap() {
        let big = Paisa::try_from_paisa(i64::MAX - i64::MAX % 100).unwrap();
        let low = big.checked_neg().unwrap();
        // 9.2e18 less a credit of -9.2e18 does not fit.
        assert!(dead_money(r(100), big, low, Paisa::ZERO, Paisa::ZERO).is_err());
        assert!(owed_now(big, low).is_err());
    }

    proptest! {
        /// The identity holds for any whole-rupee sums that fit.
        #[test]
        fn still_is_owed_then_minus_recovered_plus_corrections(
            owed in 1_i64..1_000_000,
            bal_dr in 0_i64..2_000_000,
            bal_cr in 0_i64..2_000_000,
            rec_dr in 0_i64..1_000,
            rec_cr in 0_i64..1_000_000,
        ) {
            let m = dead_money(r(owed), r(bal_dr), r(bal_cr), r(rec_dr), r(rec_cr)).unwrap();
            let lhs = r(owed).checked_sub(m.recovered).unwrap().checked_add(m.corrections).unwrap();
            prop_assert_eq!(lhs, m.still_owed);
            prop_assert!(!m.still_owed.is_negative());
            prop_assert_eq!(m.is_credit, bal_cr > bal_dr);
        }
    }
}
