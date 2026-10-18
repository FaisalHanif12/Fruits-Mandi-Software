//! A typed draft after it has been read into numbers (Code Standards 4.1 layer 2).

use super::input::{FreightPaidBy, LotChargeMethod, PaymentType};
use crate::dates::BusinessDate;
use crate::error::FieldError;
use crate::money::{Grams, Paisa, Qty};
use crate::rules::ChargeValue;

/// A person of the پختہ کھاتہ accounts, trimmed. Checked by the domain, which knows whether the
/// account is a پختہ کھاتہ account.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreparedPukhta {
    /// Urdu name, trimmed.
    pub name_ur: String,
    /// English name, trimmed.
    pub name_en: String,
    /// Phone as typed, trimmed.
    pub phone: String,
}

impl PreparedPukhta {
    /// True if nothing was typed.
    pub fn is_empty(&self) -> bool {
        self.name_ur.is_empty() && self.name_en.is_empty() && self.phone.is_empty()
    }
}

/// A lot charge typed by hand, read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypedLotCharge {
    /// What the value means.
    pub method: LotChargeMethod,
    /// The value in rupees.
    pub value: Paisa,
}

/// The header, read.
#[derive(Debug, Clone, Default)]
pub struct PreparedHeader {
    /// The posting day, if it was valid.
    pub entry_date: Option<BusinessDate>,
    /// The bill day (the posting day when empty).
    pub bill_date: Option<BusinessDate>,
    /// The second number of the بکری (PRD 12.2 "بک"), trimmed. None if empty.
    pub book: Option<String>,
    /// The truck number, trimmed. None if empty.
    pub truck_no: Option<String>,
    /// The driver's name, trimmed. None if empty.
    pub driver_name: Option<String>,
    /// The driver's phone as 11 digits. None if empty.
    pub driver_phone: Option<String>,
    /// A note, trimmed. None if empty.
    pub note: Option<String>,
    /// The زمیندار.
    pub zamindar_account_id: Option<i64>,
    /// The truck stand.
    pub adda_id: Option<i64>,
    /// The default store.
    pub store_id: Option<i64>,
    /// The total freight, if typed.
    pub total_freight: Option<Paisa>,
}

/// A sale line, read. A field that could not be read is `None` and its error is in
/// [`PreparedDraft::issues`]; `has_errors` is then true.
#[derive(Debug, Clone)]
pub struct PreparedSale {
    /// The screen's key.
    pub key: String,
    /// The saved line this row is, in an edit (plan 16). `None` for a new line.
    pub sale_id: Option<i64>,
    /// True for a row with no buyer and no rate (PRD FR-7.52).
    pub is_blank: bool,
    /// True if any value of the row could not be read.
    pub has_errors: bool,
    /// The buyer.
    pub buyer_account_id: Option<i64>,
    /// The one-time buyer.
    pub pukhta: PreparedPukhta,
    /// Quantity in نگ.
    pub qty: Option<Qty>,
    /// Weight sold, if typed.
    pub weight: Option<Grams>,
    /// The seller's rate.
    pub seller_rate: Option<Paisa>,
    /// The buyer's rate, if the rates differ.
    pub buyer_rate: Option<Paisa>,
    /// True if the buyer rate differs.
    pub rate_differs: bool,
    /// How the buyer pays, if chosen.
    pub payment_type: Option<PaymentType>,
    /// A seller commission typed on the line.
    pub seller_comm: Option<ChargeValue>,
    /// A buyer commission typed on the line.
    pub buyer_comm: Option<ChargeValue>,
    /// A لاگا typed on the line.
    pub laga: Option<ChargeValue>,
    /// A note, trimmed. None if empty.
    pub note: Option<String>,
}

/// A lot, read.
#[derive(Debug, Clone)]
pub struct PreparedLot {
    /// The screen's key.
    pub key: String,
    /// The saved lot this row is, in an edit of the first posting (plan 16). `None` for a new lot.
    pub lot_id: Option<i64>,
    /// True if a value of the lot itself could not be read.
    pub has_errors: bool,
    /// The seller.
    pub seller_account_id: Option<i64>,
    /// The one-time seller.
    pub pukhta: PreparedPukhta,
    /// The item.
    pub item_id: Option<i64>,
    /// The packing.
    pub packing_id: Option<i64>,
    /// The مارکہ.
    pub marka_id: Option<i64>,
    /// The store.
    pub store_id: Option<i64>,
    /// Quantity received.
    pub qty_received: Option<Qty>,
    /// Weight received, if typed.
    pub weight_received: Option<Grams>,
    /// Freight, 0 if empty.
    pub freight: Paisa,
    /// Who paid the driver.
    pub freight_paid_by: FreightPaidBy,
    /// مزدوری typed by hand, if any.
    pub labour: Option<TypedLotCharge>,
    /// منشیانہ typed by hand, if any.
    pub munshiana: Option<TypedLotCharge>,
    /// فون typed by hand, if any.
    pub phone: Option<TypedLotCharge>,
    /// نقد as typed. `None` if empty: 0 for an ordinary seller, and the seller credit for a one-time
    /// seller (PRD FR-13.10).
    pub cash_paid: Option<Paisa>,
    /// کرایہ سٹور, 0 if empty.
    pub store_rent: Paisa,
    /// The tick "پیشگی سے کاٹیں".
    pub adjust_advance: bool,
    /// The sale lines.
    pub sales: Vec<PreparedSale>,
}

/// A whole draft, read.
#[derive(Debug, Clone, Default)]
pub struct PreparedDraft {
    /// The header.
    pub header: PreparedHeader,
    /// The lots.
    pub lots: Vec<PreparedLot>,
    /// Every mistake found in the text, each at its field.
    pub issues: Vec<FieldError>,
}
