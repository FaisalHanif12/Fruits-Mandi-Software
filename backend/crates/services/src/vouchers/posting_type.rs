//! The posting type each voucher kind writes (Architecture 15.7).

use mandi_domain::core::vouchers::VoucherType;
use mandi_domain::ledger::PostingType;

/// The posting type a voucher kind writes (Architecture 15.7).
pub(super) const fn posting_type_of(kind: VoucherType) -> PostingType {
    match kind {
        VoucherType::Receipt => PostingType::Receipt,
        VoucherType::Payment => PostingType::Payment,
        VoucherType::Advance => PostingType::Advance,
        VoucherType::ShopExpense => PostingType::ShopExpense,
        VoucherType::PersonalExpense => PostingType::PersonalExpense,
        VoucherType::PersonalReturn => PostingType::PersonalReturn,
        VoucherType::MoneyLent => PostingType::MoneyLent,
        VoucherType::MoneyLentReturn => PostingType::MoneyLentReturn,
        VoucherType::General => PostingType::General,
        VoucherType::CapitalIn => PostingType::CapitalIn,
        VoucherType::CapitalOut => PostingType::CapitalOut,
        VoucherType::KhataOut => PostingType::KhataOut,
        VoucherType::KhataIn => PostingType::KhataIn,
    }
}
