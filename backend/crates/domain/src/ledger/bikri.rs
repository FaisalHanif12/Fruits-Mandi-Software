//! The journal entry of a بکری posting (PRD FR-12.9, Architecture 9, money agent Section 7).
//!
//! Sale part:
//!
//! | Account | Side | Amount |
//! |---|---|---|
//! | Buyer of each line | بنام | the line's buyer total |
//! | Seller of each lot | جمع (بنام if negative) | seller credit, C8 |
//! | 04010001 / 04010002 / 04010003 / 04010004 | جمع | Σ seller commission / buyer commission / لاگا / فون |
//! | 04010005 ریٹ فرق | جمع (بنام if negative) | Σ rate difference |
//! | 02010001 / 02010002 / 02010003 / 02010004 | جمع | Σ مزدوری / کرایہ (paid by the firm) / کرایہ سٹور / منشیانہ |
//!
//! Cash part: each lot with نقد: seller بنام, روکڑ جمع. Each نقدی line: روکڑ بنام, the same buyer
//! account جمع.
//!
//! No line is zero and none is negative: a negative amount goes on the other side (money agent
//! Section 7). The entry is checked by [`build_entry`], so one that does not balance is refused
//! before anything is written.
//!
//! Example (PRD 4.4): sale part 12,880 + 18,684 = 31,564 on the debit side and 24,316 + 1,764 +
//! 1,764 + 400 + 20 + 250 + 3,000 + 50 = 31,564 on the credit side; the cash part adds 5,000 to
//! each side, so the whole entry is 36,564.

use mandi_core::bikri::PaymentType;
use mandi_core::ids::AccountId;
use mandi_core::money::{MoneyError, Paisa};

use super::{BalancedEntry, LedgerError, LedgerLine, Side, build_entry};

/// One sale line as the journal needs it. Every amount is the value stored for the line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalSale {
    /// The buyer's account (own account, نقدی منڈی, or a پختہ کھاتہ account).
    pub buyer_account_id: AccountId,
    /// The person of a پختہ کھاتہ account (PRD FR-13.5).
    pub person_key: Option<String>,
    /// What the buyer pays: value + commission + لاگا.
    pub buyer_total: Paisa,
    /// Credit lines stay on the buyer's account; cash lines are settled at once.
    pub payment_type: PaymentType,
    /// The seller commission of the line.
    pub seller_comm: Paisa,
    /// The buyer commission of the line.
    pub buyer_comm: Paisa,
    /// لاگا of the line.
    pub laga: Paisa,
    /// The rate difference (buyer value − seller amount). Can be negative.
    pub rate_diff: Paisa,
}

/// One lot as the journal needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalLot {
    /// The seller's account.
    pub seller_account_id: AccountId,
    /// The person of a پختہ کھاتہ seller account.
    pub person_key: Option<String>,
    /// What the posting credits the seller (C8). Can be negative.
    pub seller_credit: Paisa,
    /// نقد given to the seller in this posting.
    pub cash_paid: Paisa,
    /// Freight paid by the firm (0 when the seller paid it, PRD E6).
    pub freight_deducted: Paisa,
    /// مزدوری.
    pub labour: Paisa,
    /// منشیانہ.
    pub munshiana: Paisa,
    /// کرایہ سٹور.
    pub store_rent: Paisa,
    /// فون.
    pub phone: Paisa,
    /// The lines of this lot.
    pub sales: Vec<JournalSale>,
}

/// Everything the journal of one posting is built from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JournalSource {
    /// The lots, in order.
    pub lots: Vec<JournalLot>,
}

/// The system accounts of PRD FR-10.4 that a بکری posts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemAccounts {
    /// روکڑ 01010001.
    pub cash: AccountId,
    /// کمیشن آمدن (بیوپاری) 04010001.
    pub seller_commission: AccountId,
    /// کمیشن آمدن (گاہک) 04010002.
    pub buyer_commission: AccountId,
    /// لاگا آمدن 04010003.
    pub laga: AccountId,
    /// فون آمدن 04010004.
    pub phone: AccountId,
    /// ریٹ فرق 04010005.
    pub rate_diff: AccountId,
    /// مزدوری کھاتہ 02010001.
    pub labour: AccountId,
    /// کرایہ کھاتہ 02010002.
    pub freight: AccountId,
    /// کرایہ سٹور کھاتہ 02010003.
    pub store_rent: AccountId,
    /// منشیانہ کھاتہ 02010004.
    pub munshiana: AccountId,
}

/// A running total of one income or payable account.
#[derive(Debug, Default, Clone, Copy)]
struct Sums {
    seller_comm: Paisa,
    buyer_comm: Paisa,
    laga: Paisa,
    phone: Paisa,
    rate_diff: Paisa,
    labour: Paisa,
    freight: Paisa,
    store_rent: Paisa,
    munshiana: Paisa,
}

/// Puts `amount` on `side` of `account`, or on the other side if it is negative. A zero amount
/// makes no line.
fn push_signed(
    lines: &mut Vec<LedgerLine>,
    account: AccountId,
    person_key: Option<&str>,
    side: Side,
    amount: Paisa,
) -> Result<(), MoneyError> {
    if amount == Paisa::ZERO {
        return Ok(());
    }
    let (side, amount) = if amount.is_negative() {
        (side.flip(), amount.checked_neg()?)
    } else {
        (side, amount)
    };
    let line = LedgerLine::new(account, side, amount);
    lines.push(match person_key {
        Some(key) => line.with_person(key),
        None => line,
    });
    Ok(())
}

/// Builds the journal entry of a بکری posting: the sale part, then the cash part.
///
/// # Errors
/// [`LedgerError::Unbalanced`] if the debits differ from the credits (a bug signal),
/// [`LedgerError::TooLarge`] if a line or the entry is above what a journal line holds,
/// [`LedgerError::Empty`] if there is nothing to post, [`LedgerError::Overflow`].
pub fn build_bikri_entry(
    source: &JournalSource,
    accounts: &SystemAccounts,
) -> Result<BalancedEntry, LedgerError> {
    let mut lines = Vec::new();
    let mut sums = Sums::default();
    for lot in &source.lots {
        for sale in &lot.sales {
            push_signed(
                &mut lines,
                sale.buyer_account_id,
                sale.person_key.as_deref(),
                Side::Dr,
                sale.buyer_total,
            )?;
            sums.seller_comm = sums.seller_comm.checked_add(sale.seller_comm)?;
            sums.buyer_comm = sums.buyer_comm.checked_add(sale.buyer_comm)?;
            sums.laga = sums.laga.checked_add(sale.laga)?;
            sums.rate_diff = sums.rate_diff.checked_add(sale.rate_diff)?;
        }
        push_signed(
            &mut lines,
            lot.seller_account_id,
            lot.person_key.as_deref(),
            Side::Cr,
            lot.seller_credit,
        )?;
        sums.phone = sums.phone.checked_add(lot.phone)?;
        sums.labour = sums.labour.checked_add(lot.labour)?;
        sums.freight = sums.freight.checked_add(lot.freight_deducted)?;
        sums.store_rent = sums.store_rent.checked_add(lot.store_rent)?;
        sums.munshiana = sums.munshiana.checked_add(lot.munshiana)?;
    }
    for (account, amount) in [
        (accounts.seller_commission, sums.seller_comm),
        (accounts.buyer_commission, sums.buyer_comm),
        (accounts.laga, sums.laga),
        (accounts.phone, sums.phone),
        (accounts.rate_diff, sums.rate_diff),
        (accounts.labour, sums.labour),
        (accounts.freight, sums.freight),
        (accounts.store_rent, sums.store_rent),
        (accounts.munshiana, sums.munshiana),
    ] {
        push_signed(&mut lines, account, None, Side::Cr, amount)?;
    }
    // The cash part (PRD FR-12.9): نقد given to a seller, and the settlement of each cash line.
    for lot in &source.lots {
        if lot.cash_paid > Paisa::ZERO {
            push_signed(
                &mut lines,
                lot.seller_account_id,
                lot.person_key.as_deref(),
                Side::Dr,
                lot.cash_paid,
            )?;
            push_signed(&mut lines, accounts.cash, None, Side::Cr, lot.cash_paid)?;
        }
        for sale in &lot.sales {
            if sale.payment_type == PaymentType::Cash {
                push_signed(&mut lines, accounts.cash, None, Side::Dr, sale.buyer_total)?;
                push_signed(
                    &mut lines,
                    sale.buyer_account_id,
                    sale.person_key.as_deref(),
                    Side::Cr,
                    sale.buyer_total,
                )?;
            }
        }
    }
    build_entry(lines)
}

#[cfg(test)]
mod tests;
