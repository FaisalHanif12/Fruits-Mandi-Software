//! What the بکری screen sends, and the closed lists both sides use (plan 12 contract).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::money::Paisa;

/// Who pays the truck freight of a lot (PRD 12.3 "کرایہ ادا"). Freight paid by the seller is
/// shown but not deducted or posted (E6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum FreightPaidBy {
    /// آڑھتی: the firm paid the driver. Deducted from the seller.
    Firm,
    /// بیوپاری: the seller paid the driver himself.
    Seller,
}

impl FreightPaidBy {
    /// The stored form (`bikri_lots.freight_paid_by`).
    pub const fn as_str(self) -> &'static str {
        match self {
            FreightPaidBy::Firm => "firm",
            FreightPaidBy::Seller => "seller",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "firm" => Some(FreightPaidBy::Firm),
            "seller" => Some(FreightPaidBy::Seller),
            _ => None,
        }
    }
}

/// How a buyer pays for a line (PRD 12.4 "ادھاری / نقدی").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    /// ادھاری: the buyer owes it.
    Credit,
    /// نقدی: paid at once.
    Cash,
}

impl PaymentType {
    /// The stored form (`bikri_sales.payment_type`).
    pub const fn as_str(self) -> &'static str {
        match self {
            PaymentType::Credit => "credit",
            PaymentType::Cash => "cash",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "credit" => Some(PaymentType::Credit),
            "cash" => Some(PaymentType::Cash),
            _ => None,
        }
    }
}

/// How a lot charge (مزدوری, منشیانہ, فون) is typed or worked out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum LotChargeMethod {
    /// An amount for each نگ received.
    PerUnit,
    /// An amount once for the lot.
    PerLot,
    /// The whole amount typed by hand.
    Manual,
}

impl LotChargeMethod {
    /// The stored form (`bikri_lots.labour_method` and the like).
    pub const fn as_str(self) -> &'static str {
        match self {
            LotChargeMethod::PerUnit => "per_unit",
            LotChargeMethod::PerLot => "per_lot",
            LotChargeMethod::Manual => "manual",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "per_unit" => Some(LotChargeMethod::PerUnit),
            "per_lot" => Some(LotChargeMethod::PerLot),
            "manual" => Some(LotChargeMethod::Manual),
            _ => None,
        }
    }
}

/// How a commission or لاگا typed on a line is meant (PRD FR-11.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum LineChargeMethod {
    /// A percent, for example `4` or `2.5`.
    Percent,
    /// An amount for each نگ, in whole rupees.
    PerUnit,
}

/// The method kept with a line charge (`bikri_sales.*_method`, Architecture 15.6). A value typed
/// on the line is `manual` (دستی) and keeps the percent or the amount per نگ that was typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum StoredMethod {
    /// فیصد.
    Percent,
    /// فی نگ.
    PerUnit,
    /// فی لاٹ.
    PerLot,
    /// دستی: typed on the line.
    Manual,
}

impl StoredMethod {
    /// The stored form (`bikri_sales.seller_comm_method` and the like).
    pub const fn as_str(self) -> &'static str {
        match self {
            StoredMethod::Percent => "percent",
            StoredMethod::PerUnit => "per_unit",
            StoredMethod::PerLot => "per_lot",
            StoredMethod::Manual => "manual",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "percent" => Some(StoredMethod::Percent),
            "per_unit" => Some(StoredMethod::PerUnit),
            "per_lot" => Some(StoredMethod::PerLot),
            "manual" => Some(StoredMethod::Manual),
            _ => None,
        }
    }
}

/// Where a charge came from (plan 12 contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ChargeSource {
    /// Typed on the line or the lot.
    Typed,
    /// A rule for a party, an item or a group (PRD 11.3 levels 2 to 5).
    Rule,
    /// The global default of the side and charge (level 6).
    Default,
    /// No rule applies and nothing was typed, so the charge is 0.
    None,
}

/// A one-time person of the پختہ کھاتہ accounts (PRD FR-13.2). Both names are required (PRD
/// FR-7.24); the phone is required for a credit buyer and for a seller.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct PukhtaInput {
    /// Urdu name.
    #[serde(default)]
    pub name_ur: String,
    /// English name.
    #[serde(default)]
    pub name_en: String,
    /// Phone, `03XXXXXXXXX` or `03XX-XXXXXXX`.
    #[serde(default)]
    pub phone: String,
}

/// A lot charge typed by hand (PRD 12.3). Empty `value` means nothing was typed, and the rules
/// may suggest the charge.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ChargeInput {
    /// What `value` means.
    pub method: LotChargeMethod,
    /// Whole rupees, as text. For `per_unit`, the amount for each نگ received.
    pub value: String,
}

/// A commission or لاگا typed on a sale line (PRD FR-11.10). Empty `value` means nothing was
/// typed, and the rule decides.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct LineChargeInput {
    /// What `value` means.
    pub method: LineChargeMethod,
    /// A percent (up to 2 decimals) for `percent`, whole rupees for `per_unit`, as text.
    pub value: String,
}

/// The header of a بکری (PRD 12.2). Everything is text; the backend checks every value.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct HeaderInput {
    /// The posting day, `DD-MM-YYYY` or a shortcut. Must be an open day (PRD FR-6.6).
    pub entry_date: String,
    /// The seller's bill day. Empty means the same as `entry_date`.
    #[serde(default)]
    pub bill_date: String,
    /// The optional second number (PRD 12.2 "بک").
    #[serde(default)]
    pub book: Option<String>,
    /// The truck number.
    #[serde(default)]
    pub truck_no: Option<String>,
    /// The زمیندار, the default seller of new lots.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub zamindar_account_id: Option<i64>,
    /// The truck stand (اڈا).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub adda_id: Option<i64>,
    /// The driver's name.
    #[serde(default)]
    pub driver_name: Option<String>,
    /// The driver's phone.
    #[serde(default)]
    pub driver_phone: Option<String>,
    /// The total truck freight in whole rupees. `bikri_split_freight` splits it over the lots.
    #[serde(default)]
    pub total_freight: Option<String>,
    /// The default store.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub store_id: Option<i64>,
    /// A note.
    #[serde(default)]
    pub note: Option<String>,
}

/// One sale line (PRD 12.4).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SaleInput {
    /// The screen's own key for this row. Echoed back so the screen can match its rows.
    pub key: String,
    /// The buyer. The پختہ کھاتہ (خریدار) account for a one-time buyer, the نقدی منڈی account for
    /// a nameless cash buyer.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub buyer_account_id: Option<i64>,
    /// The one-time buyer, for the پختہ کھاتہ (خریدار) account.
    #[serde(default)]
    pub pukhta: Option<PukhtaInput>,
    /// Quantity in نگ. A simple sum such as `20+25` is taken (PRD FR-7.48).
    #[serde(default)]
    pub qty: String,
    /// Weight sold in kg, up to 3 decimals. Required for an item sold by weight.
    #[serde(default)]
    pub weight: Option<String>,
    /// The seller's rate for each نگ, or for each kg for a weight item. Whole rupees.
    #[serde(default)]
    pub seller_rate: String,
    /// True when the buyer rate differs from the seller rate (PRD FR-12.3).
    #[serde(default)]
    pub rate_differs: bool,
    /// The buyer's rate. Used only when `rate_differs` is true.
    #[serde(default)]
    pub buyer_rate: Option<String>,
    /// How the buyer pays. Empty means the default of the buyer (PRD 12.4).
    #[serde(default)]
    pub payment_type: Option<PaymentType>,
    /// A seller commission typed on the line.
    #[serde(default)]
    pub seller_comm: Option<LineChargeInput>,
    /// A buyer commission typed on the line.
    #[serde(default)]
    pub buyer_comm: Option<LineChargeInput>,
    /// A لاگا typed on the line.
    #[serde(default)]
    pub laga: Option<LineChargeInput>,
    /// A note.
    #[serde(default)]
    pub note: Option<String>,
}

/// One lot (PRD 12.3): one seller's goods of one item.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct LotInput {
    /// The screen's own key for this row. Echoed back.
    pub key: String,
    /// The seller. The پختہ کھاتہ (بیوپاری) account for a one-time seller.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub seller_account_id: Option<i64>,
    /// The one-time seller, for the پختہ کھاتہ (بیوپاری) account.
    #[serde(default)]
    pub pukhta: Option<PukhtaInput>,
    /// The item.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// The packing.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub packing_id: Option<i64>,
    /// The مارکہ.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub marka_id: Option<i64>,
    /// The store.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub store_id: Option<i64>,
    /// Quantity received, in نگ. A simple sum is taken.
    #[serde(default)]
    pub qty_received: String,
    /// Weight received in kg. Required for an item sold by weight.
    #[serde(default)]
    pub weight_received: Option<String>,
    /// Freight of this lot, whole rupees.
    #[serde(default)]
    pub freight: Option<String>,
    /// Who paid the driver.
    pub freight_paid_by: FreightPaidBy,
    /// مزدوری. Empty: the rules may suggest it.
    #[serde(default)]
    pub labour: Option<ChargeInput>,
    /// منشیانہ. Empty: the rules may suggest it.
    #[serde(default)]
    pub munshiana: Option<ChargeInput>,
    /// فون. Empty: the rules may suggest it.
    #[serde(default)]
    pub phone: Option<ChargeInput>,
    /// نقد: cash given to the seller in this posting, whole rupees.
    #[serde(default)]
    pub cash_paid: Option<String>,
    /// کرایہ سٹور, whole rupees.
    #[serde(default)]
    pub store_rent: Option<String>,
    /// The tick "پیشگی سے کاٹیں" (PRD FR-12.23).
    pub adjust_advance: bool,
    /// The sale lines.
    #[serde(default)]
    pub sales: Vec<SaleInput>,
}

/// A whole بکری as typed.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct BikriDraft {
    /// The header.
    pub header: HeaderInput,
    /// The lots.
    #[serde(default)]
    pub lots: Vec<LotInput>,
}

/// Input of `bikri_preview`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct BikriPreviewInput {
    /// The draft to work out. Nothing is saved.
    pub draft: BikriDraft,
}

/// One lot for the freight split.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct FreightLotInput {
    /// The screen's key for the lot. Echoed back.
    pub key: String,
    /// Quantity received, in نگ.
    pub qty_received: String,
}

/// Input of `bikri_split_freight` (PRD C17).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SplitFreightInput {
    /// The header's total freight, whole rupees.
    pub total_freight: String,
    /// The lots, in lot order. The lot with the most نگ gets the leftover rupees; on a tie, the
    /// first lot.
    pub lots: Vec<FreightLotInput>,
}

/// One lot's part of the freight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FreightPart {
    /// The key sent with the lot.
    pub key: String,
    /// The lot's freight.
    #[ts(type = "Paisa")]
    pub freight_paisa: Paisa,
}

/// Input of `assist_truck_memory` (PRD FR-7.49, FR-7.50).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct TruckMemoryInput {
    /// The truck number as typed so far.
    pub truck_no: String,
    /// The posting day of the new بکری, `DD-MM-YYYY`. The same-day notice looks at this day.
    pub date: String,
}

/// Input of `assist_seller_memory` (PRD FR-7.51).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SellerMemoryInput {
    /// The seller.
    #[ts(type = "number")]
    pub seller_account_id: i64,
}

/// Input of `assist_last_rate` (PRD FR-7.53).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct LastRateInput {
    /// The item.
    #[ts(type = "number")]
    pub item_id: i64,
    /// The packing.
    #[ts(type = "number")]
    pub packing_id: i64,
    /// The posting day of the new بکری, `DD-MM-YYYY`.
    pub date: String,
}
