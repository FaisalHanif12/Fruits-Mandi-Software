//! The Fluent message IDs the print data sends to the template (CLAUDE.md rule 7). The
//! frontend defines them in `print.ftl`, in Urdu and in English. Keep this list in step with it.

pub(super) const TITLE_SELLER_BILL: &str = "print-seller-bill-title";
pub(super) const TITLE_BUYER_SLIP: &str = "print-buyer-slip-title";
pub(super) const TITLE_PUKHTA_SLIP: &str = "print-pukhta-slip-title";
pub(super) const FACT_NUMBER: &str = "print-fact-number";
/// The number of a voucher slip. The sale documents use [`FACT_NUMBER`], which reads "Sale No.".
pub(super) const FACT_VOUCHER_NUMBER: &str = "print-fact-voucher-number";
pub(super) const FACT_DATE: &str = "print-fact-date";
pub(super) const FACT_BILL_DATE: &str = "print-fact-bill-date";
pub(super) const FACT_TRUCK: &str = "print-fact-truck";
pub(super) const PARTY_SELLER: &str = "print-party-seller";
pub(super) const PARTY_BUYER: &str = "print-party-buyer";
pub(super) const PARTY_ONETIME_BUYER: &str = "print-party-onetime-buyer";
pub(super) const GROSS_SALE: &str = "print-gross-sale";
pub(super) const FREIGHT: &str = "print-freight";
pub(super) const COMMISSION: &str = "print-commission";
pub(super) const LABOUR: &str = "print-labour";
pub(super) const MUNSHIANA: &str = "print-munshiana";
pub(super) const CASH_PAID: &str = "print-cash-paid";
pub(super) const STORE_RENT: &str = "print-store-rent";
pub(super) const PHONE_CHARGE: &str = "print-phone-charge";
pub(super) const TOTAL_DEDUCTIONS: &str = "print-total-deductions";
pub(super) const NET_SALE: &str = "print-net-sale";
pub(super) const SELLER_OWES: &str = "print-seller-owes";
pub(super) const ITEMS_VALUE: &str = "print-items-value";
pub(super) const BUYER_COMMISSION: &str = "print-buyer-commission";
pub(super) const LAGA: &str = "print-laga";
pub(super) const BUYER_PAYS: &str = "print-buyer-pays";
pub(super) const IN_WORDS: &str = "print-in-words";
pub(super) const PAYMENT_CASH: &str = "print-payment-cash";
pub(super) const PAYMENT_CREDIT: &str = "print-payment-credit";

// The slip of a voucher (plan 15, PRD FR-15.2).
pub(super) const TITLE_RECEIPT_SLIP: &str = "print-receipt-slip-title";
pub(super) const TITLE_PAYMENT_SLIP: &str = "print-payment-slip-title";
pub(super) const TITLE_ADVANCE_SLIP: &str = "print-advance-slip-title";
pub(super) const FACT_REASON: &str = "print-fact-reason";
pub(super) const FACT_COLLECTOR: &str = "print-fact-collector";
pub(super) const FACT_NARRATION: &str = "print-fact-narration";
pub(super) const PARTY_ONETIME_SELLER: &str = "print-party-onetime-seller";
pub(super) const PARTY_ACCOUNT: &str = "print-party-account";
pub(super) const SLIP_AMOUNT_RECEIVED: &str = "print-slip-amount-received";
pub(super) const SLIP_AMOUNT_PAID: &str = "print-slip-amount-paid";
pub(super) const SLIP_AMOUNT_ADVANCE: &str = "print-slip-amount-advance";
