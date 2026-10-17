//! The three voucher types behind one shape (plan 15, PRD FR-7.67), for the code that does not
//! care which type it has: the preview, the edit and the cancel. Each type keeps its own checks,
//! its own entry and its own matching (`receipt`, `payment`, `advance`); this only asks them.

use std::collections::HashMap;

use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::NewVoucher;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::FieldError;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::vouchers::{VoucherEntry, VoucherType};
use mandi_domain::ledger::{BalancedEntry, PostingType};

use super::advance::{PlannedAdvance, advance_entry, advance_match, advance_row, plan_advance};
use super::common::{Hint, Role, hint_for};
use super::payment::{PlannedPayment, payment_entry, payment_match, payment_row, plan_payment};
use super::receipt::{PlannedReceipt, plan_receipt, receipt_entry, receipt_match, receipt_row};
use crate::error::ServiceError;

/// A voucher of any type that went through the checks of its form.
#[derive(Debug, Clone)]
pub(super) enum Planned {
    Receipt(PlannedReceipt),
    Payment(PlannedPayment),
    Advance(PlannedAdvance),
}

impl Planned {
    /// Checks an entry as typed.
    ///
    /// # Errors
    /// `ServiceError::Db` only; the user's mistakes are in [`Planned::errors`].
    pub fn plan(
        conn: &Connection,
        window: (Option<BusinessDate>, BusinessDate),
        entry: &VoucherEntry,
    ) -> Result<Planned, ServiceError> {
        Ok(match entry {
            VoucherEntry::Receipt(input) => Planned::Receipt(plan_receipt(conn, window, input)?),
            VoucherEntry::Payment(input) => Planned::Payment(plan_payment(conn, window, input)?),
            VoucherEntry::Advance(input) => {
                Planned::Advance(plan_advance(conn, window, input, &mut HashMap::new())?)
            }
        })
    }

    /// The kind of voucher.
    pub const fn voucher_type(&self) -> VoucherType {
        match self {
            Planned::Receipt(_) => VoucherType::Receipt,
            Planned::Payment(_) => VoucherType::Payment,
            Planned::Advance(_) => VoucherType::Advance,
        }
    }

    /// What is wrong with the entry, at the fields as sent.
    pub fn errors(&self) -> &[FieldError] {
        match self {
            Planned::Receipt(p) => &p.errors,
            Planned::Payment(p) => &p.errors,
            Planned::Advance(p) => &p.errors,
        }
    }

    /// The posting type of the journal entry.
    pub const fn posting_type(&self) -> PostingType {
        match self {
            Planned::Receipt(_) => PostingType::Receipt,
            Planned::Payment(_) => PostingType::Payment,
            Planned::Advance(_) => PostingType::Advance,
        }
    }

    /// The journal entry.
    ///
    /// # Errors
    /// `ServiceError::Data` if the entry has a mistake.
    pub fn entry(&self) -> Result<BalancedEntry, ServiceError> {
        match self {
            Planned::Receipt(p) => receipt_entry(p),
            Planned::Payment(p) => payment_entry(p),
            Planned::Advance(p) => advance_entry(p),
        }
    }

    /// The voucher row, with the number it keeps.
    ///
    /// # Errors
    /// `ServiceError::Data` if the entry has a mistake.
    pub fn row(&self, number: i64) -> Result<NewVoucher, ServiceError> {
        match self {
            Planned::Receipt(p) => receipt_row(p, number),
            Planned::Payment(p) => payment_row(p, number),
            Planned::Advance(p) => advance_row(p, number),
        }
    }

    /// The posting date, if it was read.
    pub fn date(&self) -> Option<BusinessDate> {
        match self {
            Planned::Receipt(p) => p.money.date,
            Planned::Payment(p) => p.money.date,
            Planned::Advance(p) => p.date,
        }
    }

    /// The amount, if it was read.
    pub fn amount(&self) -> Option<Paisa> {
        match self {
            Planned::Receipt(p) => p.money.amount,
            Planned::Payment(p) => p.money.amount,
            Planned::Advance(p) => (p.amount != Paisa::ZERO).then_some(p.amount),
        }
    }

    /// The account, if it was found.
    pub fn account(&self) -> Option<&RecordRef> {
        match self {
            Planned::Receipt(p) => p.party.as_ref().map(|p| &p.record),
            Planned::Payment(p) => p.party.as_ref().map(|p| &p.record),
            Planned::Advance(p) => p.account.as_ref(),
        }
    }

    /// The one-time person, if any.
    pub fn person_key(&self) -> Option<&str> {
        match self {
            Planned::Receipt(p) => p.party.as_ref().and_then(|p| p.person_key.as_deref()),
            Planned::Payment(p) => p.party.as_ref().and_then(|p| p.person_key.as_deref()),
            Planned::Advance(_) => None,
        }
    }

    /// The role of the account and whether it is a پختہ کھاتہ account.
    pub fn role(&self) -> Option<(Role, bool)> {
        match self {
            Planned::Receipt(p) => p.party.as_ref().map(|p| (p.role, p.is_pukhta)),
            Planned::Payment(p) => p.party.as_ref().map(|p| (p.role, p.is_pukhta)),
            Planned::Advance(p) => p.account.as_ref().map(|_| (Role::Seller, false)),
        }
    }

    /// Matches the posted entry (fast path: the rows of the party are canonical).
    ///
    /// # Errors
    /// The errors of the matching.
    pub fn match_posted(
        &self,
        tx: &Connection,
        user_id: i64,
        now: &str,
        entry_id: i64,
        hint: &Hint,
    ) -> Result<(), ServiceError> {
        match self {
            Planned::Receipt(p) => receipt_match(tx, user_id, now, p, entry_id, hint),
            Planned::Payment(p) => payment_match(tx, user_id, now, p, entry_id, hint),
            Planned::Advance(p) => advance_match(tx, user_id, now, p),
        }
    }

    /// What the buyer's rows add up to now, to be read just before the posting (see [`Hint`]).
    ///
    /// # Errors
    /// `ServiceError::Db`.
    pub fn hint(&self, conn: &Connection) -> Result<Hint, ServiceError> {
        match (self.account(), self.role()) {
            (Some(account), Some((role, _))) => hint_for(conn, role, account.id, self.person_key()),
            _ => Ok(Hint::default()),
        }
    }
}
