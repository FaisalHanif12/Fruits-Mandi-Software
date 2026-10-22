//! The journal entry of a بکری posting (PRD FR-12.9, Architecture 9, money agent Section 7).
//!
//! Sale part:
//!
//! | Account | Side | Amount |
//! |---|---|---|
//! | Buyer of each line | بنام | the line's buyer total |
//! | Seller of each lot | جمع (بنام if negative) | seller credit, C8 |
//! | Trip lot: each trip partner's 0604 account instead of a seller | جمع (بنام if negative) | his part of the seller credit, C25 |
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
    /// The seller's account. For a trip lot it is the اپنا مال account, which never holds a balance:
    /// the seller credit goes to the trip partners' 0604 accounts instead (see `partner_credits`).
    pub seller_account_id: AccountId,
    /// A trip lot's seller credit split between the trip's partners (PRD FR-12.26, C25): each
    /// partner's 0604 account and his part, in the order the partners were created. Empty for any
    /// other lot. The parts add up to `seller_credit`; the credit is then not posted to
    /// `seller_account_id`. The cost of the goods (C21) is never in the journal.
    pub partner_credits: Vec<(AccountId, Paisa)>,
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
/// makes no line. Returns the position of the line it made.
fn push_signed(
    lines: &mut Vec<LedgerLine>,
    account: AccountId,
    person_key: Option<&str>,
    side: Side,
    amount: Paisa,
) -> Result<Option<usize>, MoneyError> {
    if amount == Paisa::ZERO {
        return Ok(None);
    }
    let (side, amount) = if amount.is_negative() {
        (side.flip(), amount.checked_neg()?)
    } else {
        (side, amount)
    };
    let line = LedgerLine::new(account, side, amount);
    let at = lines.len();
    lines.push(match person_key {
        Some(key) => line.with_person(key),
        None => line,
    });
    Ok(Some(at))
}

/// Where the lines of one sale line are in the built entry (a position in
/// `BalancedEntry::lines()`, from 0).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SaleLayout {
    /// The buyer's debit line: the bill of this line.
    pub buyer_debit: Option<usize>,
    /// The buyer's credit line of the cash part: the cash that settles a نقدی line at once.
    pub cash_credit: Option<usize>,
}

/// Where the lines of one lot are in the built entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LotLayout {
    /// The seller's credit line (C8). `None` when the credit is 0 or negative.
    pub seller_credit: Option<usize>,
    /// The seller's debit line when the credit is negative (the seller owes the firm). It is an
    /// ordinary debit: never an advance, never matched.
    pub seller_debit: Option<usize>,
    /// The seller's debit line of the cash part (نقد).
    pub cash_debit: Option<usize>,
    /// The sale lines of the lot, in order.
    pub sales: Vec<SaleLayout>,
}

/// Where the lines that matter for matching are in the entry. The save uses it to find the
/// stored line ids after posting (a line's position plus one is its `line_no`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntryLayout {
    /// The lots, in the order of the source.
    pub lots: Vec<LotLayout>,
    /// The position of the first line of the cash part: the lines before it are the sale part
    /// and balance on their own, and so do the lines from here on.
    pub cash_part_start: usize,
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
    build_bikri_entry_with_layout(source, accounts).map(|(entry, _)| entry)
}

/// [`build_bikri_entry`] and where the lines that matter for matching are (PRD FR-15.4, FR-15.8):
/// each lot's seller line and each sale line's buyer debit and cash settlement credit. The
/// entry is the same, line for line.
///
/// # Errors
/// As [`build_bikri_entry`].
pub fn build_bikri_entry_with_layout(
    source: &JournalSource,
    accounts: &SystemAccounts,
) -> Result<(BalancedEntry, EntryLayout), LedgerError> {
    let mut lines = Vec::new();
    let mut sums = Sums::default();
    let mut layout = EntryLayout {
        lots: source
            .lots
            .iter()
            .map(|lot| LotLayout {
                sales: vec![SaleLayout::default(); lot.sales.len()],
                ..LotLayout::default()
            })
            .collect(),
        cash_part_start: 0,
    };
    for (lot_index, lot) in source.lots.iter().enumerate() {
        for (sale_index, sale) in lot.sales.iter().enumerate() {
            let at = push_signed(
                &mut lines,
                sale.buyer_account_id,
                sale.person_key.as_deref(),
                Side::Dr,
                sale.buyer_total,
            )?;
            if let Some(slot) = layout
                .lots
                .get_mut(lot_index)
                .and_then(|l| l.sales.get_mut(sale_index))
            {
                slot.buyer_debit = at;
            }
            sums.seller_comm = sums.seller_comm.checked_add(sale.seller_comm)?;
            sums.buyer_comm = sums.buyer_comm.checked_add(sale.buyer_comm)?;
            sums.laga = sums.laga.checked_add(sale.laga)?;
            sums.rate_diff = sums.rate_diff.checked_add(sale.rate_diff)?;
        }
        if lot.partner_credits.is_empty() {
            let at = push_signed(
                &mut lines,
                lot.seller_account_id,
                lot.person_key.as_deref(),
                Side::Cr,
                lot.seller_credit,
            )?;
            if let Some(slot) = layout.lots.get_mut(lot_index) {
                if lot.seller_credit.is_negative() {
                    slot.seller_debit = at;
                } else {
                    slot.seller_credit = at;
                }
            }
        } else {
            // A trip lot: the seller credit goes to the trip's partners (PRD FR-12.26, C25), each on
            // his 0604 account (debit if negative). They add up to the credit, or the entry does not
            // balance and nothing is written. Nothing is matched on these lines.
            for (account, part) in &lot.partner_credits {
                push_signed(&mut lines, *account, None, Side::Cr, *part)?;
            }
        }
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
    layout.cash_part_start = lines.len();
    for (lot_index, lot) in source.lots.iter().enumerate() {
        if lot.cash_paid > Paisa::ZERO {
            let at = push_signed(
                &mut lines,
                lot.seller_account_id,
                lot.person_key.as_deref(),
                Side::Dr,
                lot.cash_paid,
            )?;
            if let Some(slot) = layout.lots.get_mut(lot_index) {
                slot.cash_debit = at;
            }
            push_signed(&mut lines, accounts.cash, None, Side::Cr, lot.cash_paid)?;
        }
        for (sale_index, sale) in lot.sales.iter().enumerate() {
            if sale.payment_type == PaymentType::Cash {
                push_signed(&mut lines, accounts.cash, None, Side::Dr, sale.buyer_total)?;
                let at = push_signed(
                    &mut lines,
                    sale.buyer_account_id,
                    sale.person_key.as_deref(),
                    Side::Cr,
                    sale.buyer_total,
                )?;
                if let Some(slot) = layout
                    .lots
                    .get_mut(lot_index)
                    .and_then(|l| l.sales.get_mut(sale_index))
                {
                    slot.cash_credit = at;
                }
            }
        }
    }
    Ok((build_entry(lines)?, layout))
}

#[cfg(test)]
mod tests;
