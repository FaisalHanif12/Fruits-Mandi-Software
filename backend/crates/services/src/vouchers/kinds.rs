//! The voucher types behind one shape (plan 15, PRD FR-7.67), for the code that does not
//! care which type it has: the preview, the edit and the cancel. Each type keeps its own checks,
//! its own entry and its own matching (`receipt`, `payment`, `advance`); this only asks them.
//!
//! Plan 17 adds ten kinds. Shop and personal expense, personal return, money lent and its return,
//! capital in and out, and khata out and in each carry a `PlannedCash`. A جنرل carries a
//! `PlannedGeneral`, because it has many lines and no single account.

use std::collections::HashMap;

use mandi_db::ledger::StoredEntry;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::NewVoucher;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::FieldError;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::vouchers::{VoucherEntry, VoucherType};
use mandi_domain::ledger::{BalancedEntry, PostingType};

use super::advance::{PlannedAdvance, advance_entry, advance_row, plan_advance};
use super::capital_kinds::plan_capital;
use super::cash_kinds::{
    PlannedCash, plan_money_lent, plan_money_lent_return, plan_personal_expense,
    plan_personal_return, plan_shop_expense,
};
use super::common::{Hint, Role, hint_for};
use super::general_kind::{PlannedGeneral, plan_general};
use super::khata_kinds::{plan_khata_in, plan_khata_out};
use super::payment::{PlannedPayment, payment_entry, payment_row, plan_payment};
use super::posting_type::posting_type_of;
use super::receipt::{PlannedReceipt, plan_receipt, receipt_entry, receipt_row};
use crate::error::ServiceError;

/// The kinds with one account and a plain two-line posting, as a pattern that binds `$p` in every
/// arm: the خرچہ screen, ادھار رقم, سرمایہ and الگ کھاتہ.
macro_rules! cash_kinds {
    ($p:pat) => {
        Planned::ShopExpense($p)
            | Planned::PersonalExpense($p)
            | Planned::PersonalReturn($p)
            | Planned::MoneyLent($p)
            | Planned::MoneyLentReturn($p)
            | Planned::CapitalIn($p)
            | Planned::CapitalOut($p)
            | Planned::KhataOut($p)
            | Planned::KhataIn($p)
    };
}

/// A voucher of any type that went through the checks of its form.
#[derive(Debug, Clone)]
pub(super) enum Planned {
    Receipt(PlannedReceipt),
    Payment(PlannedPayment),
    Advance(PlannedAdvance),
    ShopExpense(PlannedCash),
    PersonalExpense(PlannedCash),
    PersonalReturn(PlannedCash),
    MoneyLent(PlannedCash),
    MoneyLentReturn(PlannedCash),
    CapitalIn(PlannedCash),
    CapitalOut(PlannedCash),
    KhataOut(PlannedCash),
    KhataIn(PlannedCash),
    General(PlannedGeneral),
}

impl Planned {
    /// Checks an entry as typed. `without` is the posting an edit would replace, for the preview
    /// (a return is judged on the books without it, like the save, which posts the reversal
    /// first and so passes none); only the two return kinds read it.
    ///
    /// # Errors
    /// `ServiceError::Db` only; the user's mistakes are in [`Planned::errors`].
    pub fn plan(
        conn: &Connection,
        window: (Option<BusinessDate>, BusinessDate),
        entry: &VoucherEntry,
        without: Option<&StoredEntry>,
    ) -> Result<Planned, ServiceError> {
        Ok(match entry {
            VoucherEntry::Receipt(input) => Planned::Receipt(plan_receipt(conn, window, input)?),
            VoucherEntry::Payment(input) => Planned::Payment(plan_payment(conn, window, input)?),
            VoucherEntry::Advance(input) => {
                Planned::Advance(plan_advance(conn, window, input, &mut HashMap::new())?)
            }
            VoucherEntry::ShopExpense(input) => {
                Planned::ShopExpense(plan_shop_expense(conn, window, input)?)
            }
            VoucherEntry::PersonalExpense(input) => {
                Planned::PersonalExpense(plan_personal_expense(conn, window, input)?)
            }
            VoucherEntry::PersonalReturn(input) => {
                Planned::PersonalReturn(plan_personal_return(conn, window, input, without)?)
            }
            VoucherEntry::MoneyLent(input) => {
                Planned::MoneyLent(plan_money_lent(conn, window, input)?)
            }
            VoucherEntry::MoneyLentReturn(input) => {
                Planned::MoneyLentReturn(plan_money_lent_return(conn, window, input, without)?)
            }
            VoucherEntry::CapitalIn(input) => {
                Planned::CapitalIn(plan_capital(conn, window, input, VoucherType::CapitalIn)?)
            }
            VoucherEntry::CapitalOut(input) => {
                Planned::CapitalOut(plan_capital(conn, window, input, VoucherType::CapitalOut)?)
            }
            VoucherEntry::KhataOut(input) => {
                Planned::KhataOut(plan_khata_out(conn, window, input)?)
            }
            VoucherEntry::KhataIn(input) => Planned::KhataIn(plan_khata_in(conn, window, input)?),
            VoucherEntry::General(input) => Planned::General(plan_general(
                conn,
                window,
                Some(&input.date),
                input.narration.as_deref(),
                &input.lines,
                true,
            )?),
        })
    }

    /// The kind of voucher.
    pub const fn voucher_type(&self) -> VoucherType {
        match self {
            Planned::Receipt(_) => VoucherType::Receipt,
            Planned::Payment(_) => VoucherType::Payment,
            Planned::Advance(_) => VoucherType::Advance,
            cash_kinds!(p) => p.kind,
            Planned::General(_) => VoucherType::General,
        }
    }

    /// What is wrong with the entry, at the fields as sent.
    pub fn errors(&self) -> &[FieldError] {
        match self {
            Planned::Receipt(p) => &p.errors,
            Planned::Payment(p) => &p.errors,
            Planned::Advance(p) => &p.errors,
            cash_kinds!(p) => &p.errors,
            Planned::General(p) => &p.errors,
        }
    }

    /// The posting type of the journal entry.
    pub const fn posting_type(&self) -> PostingType {
        match self {
            Planned::Receipt(_) => PostingType::Receipt,
            Planned::Payment(_) => PostingType::Payment,
            Planned::Advance(_) => PostingType::Advance,
            cash_kinds!(p) => posting_type_of(p.kind),
            Planned::General(_) => PostingType::General,
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
            cash_kinds!(p) => p.entry(),
            Planned::General(p) => p.entry(),
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
            cash_kinds!(p) => p.row(number),
            Planned::General(p) => p.row(number),
        }
    }

    /// The posting date, if it was read.
    pub fn date(&self) -> Option<BusinessDate> {
        match self {
            Planned::Receipt(p) => p.money.date,
            Planned::Payment(p) => p.money.date,
            Planned::Advance(p) => p.date,
            cash_kinds!(p) => p.date,
            Planned::General(p) => p.date,
        }
    }

    /// The amount, if it was read.
    pub fn amount(&self) -> Option<Paisa> {
        match self {
            Planned::Receipt(p) => p.money.amount,
            Planned::Payment(p) => p.money.amount,
            Planned::Advance(p) => (p.amount != Paisa::ZERO).then_some(p.amount),
            cash_kinds!(p) => p.amount,
            // The total of a جنرل, once its entry stands.
            Planned::General(p) => p.entry().ok().map(|e| e.total()),
        }
    }

    /// The account, if it was found.
    pub fn account(&self) -> Option<&RecordRef> {
        match self {
            Planned::Receipt(p) => p.party.as_ref().map(|p| &p.record),
            Planned::Payment(p) => p.party.as_ref().map(|p| &p.record),
            Planned::Advance(p) => p.account.as_ref(),
            cash_kinds!(p) => p.account.as_ref(),
            Planned::General(_) => None,
        }
    }

    /// The one-time person, if any.
    pub fn person_key(&self) -> Option<&str> {
        match self {
            Planned::Receipt(p) => p.party.as_ref().and_then(|p| p.person_key.as_deref()),
            Planned::Payment(p) => p.party.as_ref().and_then(|p| p.person_key.as_deref()),
            Planned::Advance(_) | cash_kinds!(_) | Planned::General(_) => None,
        }
    }

    /// The role of the account and whether it is a پختہ کھاتہ account.
    pub fn role(&self) -> Option<(Role, bool)> {
        match self {
            Planned::Receipt(p) => p.party.as_ref().map(|p| (p.role, p.is_pukhta)),
            Planned::Payment(p) => p.party.as_ref().map(|p| (p.role, p.is_pukhta)),
            Planned::Advance(p) => p.account.as_ref().map(|_| (Role::Seller, false)),
            cash_kinds!(_) | Planned::General(_) => None,
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

impl Planned {
    /// The accounts and people a جنرل touches, to match each buyer again after an edit or a
    /// cancel. Every other kind has one account, handled by [`Planned::account`].
    pub fn general_lines(&self) -> Vec<(i64, Option<String>)> {
        match self {
            Planned::General(p) => p.touched(),
            _ => Vec::new(),
        }
    }

    /// The lines of a جنرل for the audit log, if this is one.
    pub fn general_lines_json(&self) -> Option<serde_json::Value> {
        match self {
            Planned::General(p) => Some(p.lines_json()),
            _ => None,
        }
    }
}
