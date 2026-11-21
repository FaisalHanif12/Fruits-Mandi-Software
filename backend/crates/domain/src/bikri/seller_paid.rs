//! A seller's money paid on the spot (موقع پر ادائیگی, PRD FR-12.23 v1.53, plan 91).
//!
//! No new posting and no new money rule. The money a seller takes in the same بکری is the نقد of his
//! lots (seller بنام, روکڑ جمع, PRD FR-12.9). This file does two things with it, both with checked
//! `Paisa` arithmetic:
//!
//! * [`spread_seller_paid`] puts one amount over the نقد of a seller's lots, in lot order, each up to
//!   its credit, and all the rest on his last lot;
//! * [`seller_lines`] works out the lines of a seller's summary (previous balance, today's credit,
//!   paid, what was cut or added, new balance) from the نقد his lots have. The preview and the save
//!   both read them from [`calc_posting`](super::calc_posting), so they cannot differ (Code
//!   Standards B5).
//!
//! Worked numbers (PRD plan 91): previous 30,000 بنام, credit 20,000, paid 5,000: cut 15,000, new
//! 15,000 بنام. Two lots with credits 12,000 and 8,000 and 15,000 paid: 12,000 and 3,000.

use mandi_core::accounts::BalanceSide;
use mandi_core::bikri::{LotPreview, PreparedDraft, SellerPaidLines};
use mandi_core::money::{MoneyError, Paisa};
use mandi_core::rules::{Working, WorkingArg, WorkingValue};

use super::lookups::{Lookups, PartyKind};

/// A lot of the draft as the spread sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpreadLot {
    /// The place of the lot in the draft.
    pub index: usize,
    /// The seller of the lot.
    pub seller_id: i64,
    /// What the lot credits the seller (C8), before نقد. Can be negative.
    pub credit: Paisa,
    /// False for a lot that takes no cash: trip goods, a garden's goods, a later sale.
    pub takes_cash: bool,
}

/// Puts `amount` over the نقد of the lots of `seller_id` that take cash, in the order given. Every lot
/// but the last gets the amount that is left, up to its own credit (a credit of 0 or less takes
/// nothing); the last lot gets all that is left, even above its credit. So the parts always add up
/// to `amount` (PRD FR-12.23 v1.53).
///
/// Returns the place of each lot with its نقد, or `None` when no lot of the seller takes cash.
///
/// Example: credits 12,000 and 8,000, `amount` 15,000: `[(0, 12,000), (1, 3,000)]`. Credits 12,000
/// and 8,000, `amount` 30,000: `[(0, 12,000), (1, 18,000)]`.
///
/// # Errors
/// [`MoneyError::NegativeValue`] for a negative `amount`; [`MoneyError::Overflow`].
pub fn spread_seller_paid(
    lots: &[SpreadLot],
    seller_id: i64,
    amount: Paisa,
) -> Result<Option<Vec<(usize, Paisa)>>, MoneyError> {
    if amount.is_negative() {
        return Err(MoneyError::NegativeValue);
    }
    let mine: Vec<&SpreadLot> = lots
        .iter()
        .filter(|lot| lot.seller_id == seller_id && lot.takes_cash)
        .collect();
    let Some((last, others)) = mine.split_last() else {
        return Ok(None);
    };
    let mut left = amount;
    let mut parts = Vec::with_capacity(mine.len());
    for lot in others {
        // A lot that credits nothing (or is negative) takes nothing; the rest goes on.
        let room = lot.credit.max(Paisa::ZERO);
        let give = left.min(room);
        left = left.checked_sub(give)?;
        parts.push((lot.index, give));
    }
    parts.push((last.index, left));
    Ok(Some(parts))
}

/// The side of a signed balance (debit positive).
fn side_of(signed: Paisa) -> BalanceSide {
    if signed == Paisa::ZERO {
        BalanceSide::Zero
    } else if signed.is_negative() {
        BalanceSide::Cr
    } else {
        BalanceSide::Dr
    }
}

/// A signed balance as the screen shows it: never negative, with its side.
fn shown(signed: Paisa) -> Result<(Paisa, BalanceSide), MoneyError> {
    let side = side_of(signed);
    let amount = if signed.is_negative() {
        signed.checked_neg()?
    } else {
        signed
    };
    Ok((amount, side))
}

fn money(name: &str, paisa: Paisa) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value: WorkingValue::Money { paisa },
    }
}

/// How the new balance was worked out: `previous + paid - credit`, all signed with debit positive.
fn new_balance_working(previous: Paisa, paid: Paisa, credit: Paisa, new: Paisa) -> Working {
    Working {
        message_id: "working-seller-new-balance".to_owned(),
        args: vec![
            money("previous", previous),
            money("paid", paid),
            money("credit", credit),
            money("result", new),
        ],
        rule_id: None,
    }
}

/// The lines of one seller's summary (PRD FR-12.23 v1.53).
///
/// * `previous` is his ledger balance before this بکری, debit positive (a جمع balance is negative);
/// * `credit` is the sum of the seller credits (C8) of his lots in this بکری, before نقد;
/// * `paid` is the sum of the نقد of those lots, 0 or more.
///
/// `new = previous + paid - credit`. Today's credit counts as 0 when it is negative (the charges
/// of the lots are more than their sales): that part adds to what he owes (`increase_owed`). What
/// is not paid in cash is `carry = credit - paid` (credit never below 0), the rest of today's sale
/// that goes into his account (Umar: "بکری کی باقی رقم، کھاتے میں جمع"). When it is 0 or more it
/// first takes off what he owed (`cut`, never more than what he owed) and the rest adds to what the
/// shop owes him (`added_to_credit`); `carry = cut + added_to_credit`. When it would be negative he
/// took more than today's credit (`over_paid`) and `carry` is 0. `limit` is today's credit plus what the shop owed him (never below 0); a `paid`
/// above it needs the Owner's confirmation.
///
/// Examples: previous 30,000, credit 20,000, paid 5,000 gives cut 15,000, new 15,000 بنام, limit
/// 20,000. Previous -10,000 (جمع), credit 20,000, paid 28,000 gives over_paid 8,000, new 2,000 جمع,
/// limit 30,000, no confirmation.
///
/// # Errors
/// [`MoneyError::NegativeValue`] for a negative `paid`; [`MoneyError::Overflow`].
pub fn seller_lines(
    seller_account_id: i64,
    previous: Paisa,
    credit: Paisa,
    paid: Paisa,
) -> Result<SellerPaidLines, MoneyError> {
    if paid.is_negative() {
        return Err(MoneyError::NegativeValue);
    }
    let owes = previous.max(Paisa::ZERO);
    let owed_to_him = if previous.is_negative() {
        previous.checked_neg()?
    } else {
        Paisa::ZERO
    };
    // A lot whose charges are more than its sales credits him nothing and adds to what he owes.
    let credit_up = credit.max(Paisa::ZERO);
    let increase_owed = if credit.is_negative() {
        credit.checked_neg()?
    } else {
        Paisa::ZERO
    };
    let carry = credit_up.checked_sub(paid)?;
    let (carry_up, cut, added_to_credit, over_paid) = if carry.is_negative() {
        (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO, carry.checked_neg()?)
    } else {
        let cut = carry.min(owes);
        (carry, cut, carry.checked_sub(cut)?, Paisa::ZERO)
    };
    let new = previous.checked_add(paid)?.checked_sub(credit)?;
    // What he can take without a question: today's credit plus what the shop owed him. A negative
    // credit (charges above sales) takes off what he was owed, so he is never moved from being owed
    // to owing without being asked.
    let limit = credit.checked_add(owed_to_him)?.max(Paisa::ZERO);
    let (previous_paisa, previous_side) = shown(previous)?;
    let (new_paisa, new_side) = shown(new)?;
    Ok(SellerPaidLines {
        seller_account_id,
        previous_paisa,
        previous_side,
        credit_paisa: credit,
        paid_paisa: paid,
        carry_paisa: carry_up,
        cut_paisa: cut,
        added_to_credit_paisa: added_to_credit,
        over_paid_paisa: over_paid,
        increase_owed_paisa: increase_owed,
        new_paisa,
        new_side,
        limit_paisa: limit,
        needs_confirm: paid > limit,
        working: new_balance_working(previous, paid, credit, new),
    })
}

/// One summary for each seller of a complete lot, in the order of his first lot (PRD FR-12.23 v1.53).
/// A lot of trip goods or of a garden has none (it takes no نقد), and neither has a lot that could
/// not be worked out. The credit and the نقد are the sums over his lots as `calc_posting` worked them
/// out, so they are the numbers the journal is built from.
///
/// # Errors
/// [`MoneyError::Overflow`].
pub(super) fn sellers_of(
    draft: &PreparedDraft,
    lookups: &Lookups,
    lots: &[LotPreview],
) -> Result<Vec<SellerPaidLines>, MoneyError> {
    // (seller, credit, paid), in the order of the first lot of each seller.
    let mut sums: Vec<(i64, Paisa, Paisa)> = Vec::new();
    for (lot, preview) in draft.lots.iter().zip(lots) {
        let (Some(seller_id), Some(totals)) = (lot.seller_account_id, preview.totals.as_ref())
        else {
            continue;
        };
        let Some(facts) = lookups.accounts.get(&seller_id) else {
            continue;
        };
        if facts.is_own_goods || !matches!(facts.kind, PartyKind::Seller | PartyKind::Khata) {
            continue;
        }
        match sums.iter_mut().find(|(id, _, _)| *id == seller_id) {
            Some((_, credit, paid)) => {
                *credit = credit.checked_add(totals.seller_credit)?;
                *paid = paid.checked_add(totals.cash_paid)?;
            }
            None => sums.push((seller_id, totals.seller_credit, totals.cash_paid)),
        }
    }
    sums.into_iter()
        .map(|(seller_id, credit, paid)| {
            let previous = lookups
                .previous
                .get(&seller_id)
                .copied()
                .unwrap_or(Paisa::ZERO);
            seller_lines(seller_id, previous, credit, paid)
        })
        .collect()
}

/// The previous balance of a summary as a signed amount, debit positive.
fn previous_signed(lines: &SellerPaidLines) -> Result<Paisa, MoneyError> {
    match lines.previous_side {
        BalanceSide::Cr => lines.previous_paisa.checked_neg(),
        BalanceSide::Dr | BalanceSide::Zero => Ok(lines.previous_paisa),
    }
}

/// Puts together the summaries of the parts of a mixed screen (PRD FR-12.17, FR-12.48): a seller who
/// has a carried lot and a new lot in one save has one summary. His previous balance is the one all
/// the parts were worked out against (nothing of the save is written yet), his credit and his نقد
/// are the sums over the parts, so `new` is what the ledger holds after the whole save. The order is
/// that of the first part he is in.
///
/// # Errors
/// [`MoneyError::Overflow`].
pub fn merge_sellers(parts: &[&[SellerPaidLines]]) -> Result<Vec<SellerPaidLines>, MoneyError> {
    // (seller, previous signed, credit, paid)
    let mut sums: Vec<(i64, Paisa, Paisa, Paisa)> = Vec::new();
    for lines in parts.iter().flat_map(|part| part.iter()) {
        match sums
            .iter_mut()
            .find(|(id, _, _, _)| *id == lines.seller_account_id)
        {
            Some((_, _, credit, paid)) => {
                *credit = credit.checked_add(lines.credit_paisa)?;
                *paid = paid.checked_add(lines.paid_paisa)?;
            }
            None => sums.push((
                lines.seller_account_id,
                previous_signed(lines)?,
                lines.credit_paisa,
                lines.paid_paisa,
            )),
        }
    }
    sums.into_iter()
        .map(|(seller_id, previous, credit, paid)| seller_lines(seller_id, previous, credit, paid))
        .collect()
}
