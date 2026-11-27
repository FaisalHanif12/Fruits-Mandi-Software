//! A garden's "فروخت کی رقم کہاں ہے" (Where the sale money is, PRD FR-12.53 v1.60, plan 99): one row
//! for each sale line of the garden's lots in our shop, one row for a lot's charges, and one row for
//! each sale in another shop.
//!
//! Command `gardens_sale_money`: input [`GardensSaleMoneyInput`], answer [`GardensSaleMoney`]. Read
//! only.
//!
//! The garden's credit of a sale line is its stored value less the seller commission on it. The
//! lot's other charges (freight paid by آڑھتی, مزدوری, منشیانہ, فون, کرایہ سٹور) are one row of their
//! own with a negative credit, so the rows of a lot add up to its seller credit (PRD C8) and nothing
//! is split by guess. The totals of the garden's credit over the whole garden are its money in from
//! our shop and other shops (R36); a relief refund and fruit taken are no sale and are not here.
//!
//! Example: 20 پیٹی آم sold to مانی at 1,000: value 20,000, his bill 21,360, paid on the spot 0,
//! still on his account 21,360, the garden's credit 20,000 − 1,200 = 18,800.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::money::Paisa;
use crate::rules::Working;

/// Input of `gardens_sale_money`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardensSaleMoneyInput {
    /// The garden (`ventures.id`).
    #[ts(type = "number")]
    pub garden_id: i64,
    /// First day, `DD-MM-YYYY` or a shortcut. Null or empty is no limit.
    #[serde(default)]
    pub from: Option<String>,
    /// Last day. Null or empty is no limit.
    #[serde(default)]
    pub to: Option<String>,
    /// Only the sale lines of the lots of this goods-arrived entry, and the sales in another shop
    /// that name it (a click on an arrival row). Null is every row.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub arrival_id: Option<i64>,
    /// Only rows whose buyer (name or code), other shop, item or بکری number holds this text.
    #[serde(default)]
    pub search: Option<String>,
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives a page of 100 (Code Standards B31).
    #[serde(default)]
    pub limit: u32,
}

/// What a row is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SaleMoneyKind {
    /// A sale line of a garden lot in our shop.
    Bikri,
    /// The charges of a garden lot of that بکری other than the seller commission, taken off the
    /// garden's credit.
    LotCharges,
    /// A sale in another shop: the money came into روکڑ.
    OtherShop,
}

/// The lot of a row of our shop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SaleMoneyLot {
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// Its number in its بکری.
    #[ts(type = "number")]
    pub lot_no: i64,
    /// The counter of its name (PRD FR-12.45).
    #[ts(type = "number")]
    pub lot_counter: i64,
    /// Its item.
    pub item: RecordRef,
    /// Its packing.
    pub packing: RecordRef,
}

/// The charges of a `lot_charges` row, each 0 or more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct SaleMoneyCharges {
    /// کرایہ, when آڑھتی paid it.
    #[ts(type = "Paisa")]
    pub freight_paisa: Paisa,
    /// مزدوری.
    #[ts(type = "Paisa")]
    pub labour_paisa: Paisa,
    /// منشیانہ.
    #[ts(type = "Paisa")]
    pub munshiana_paisa: Paisa,
    /// فون.
    #[ts(type = "Paisa")]
    pub phone_paisa: Paisa,
    /// کرایہ سٹور.
    #[ts(type = "Paisa")]
    pub store_rent_paisa: Paisa,
}

/// One row. A field that its kind does not have is null.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SaleMoneyRow {
    /// What it is.
    pub kind: SaleMoneyKind,
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The بکری (`bikri`, `lot_charges`).
    #[ts(type = "number | null")]
    pub bikri_id: Option<i64>,
    /// Its number.
    #[ts(type = "number | null")]
    pub bikri_number: Option<i64>,
    /// The lot (`bikri`, `lot_charges`).
    pub lot: Option<SaleMoneyLot>,
    /// The item of a sale in another shop that names an arrival.
    pub item: Option<RecordRef>,
    /// The buyer (`bikri`); a چکتا buyer by his own names.
    pub buyer: Option<RecordRef>,
    /// The other shop, as typed (`other_shop`); empty if none was typed.
    pub shop_name: Option<String>,
    /// The garden entry of a sale in another shop.
    #[ts(type = "number | null")]
    pub entry_id: Option<i64>,
    /// نگ sold (`bikri`, `other_shop`).
    #[ts(type = "number | null")]
    pub qty: Option<i64>,
    /// The weight of a line of an item sold by weight, in grams.
    #[ts(type = "number | null")]
    pub weight_grams: Option<i64>,
    /// The seller rate (`bikri`), per نگ or per kg.
    #[ts(type = "Paisa | null")]
    pub rate_paisa: Option<Paisa>,
    /// The value: qty × rate (`bikri`), or the money received (`other_shop`).
    #[ts(type = "Paisa | null")]
    pub value_paisa: Option<Paisa>,
    /// The buyer's bill of the line (`bikri`).
    #[ts(type = "Paisa | null")]
    pub buyer_total_paisa: Option<Paisa>,
    /// Paid on the spot (`bikri`).
    #[ts(type = "Paisa | null")]
    pub paid_paisa: Option<Paisa>,
    /// Settled since by his وصولی and other credits, from the bill matching (`bikri`).
    #[ts(type = "Paisa | null")]
    pub collected_paisa: Option<Paisa>,
    /// Still on the buyer's account (`bikri`): bill − paid − collected.
    #[ts(type = "Paisa | null")]
    pub still_due_paisa: Option<Paisa>,
    /// The garden's credit: value − seller commission (`bikri`), − the lot's charges
    /// (`lot_charges`, below 0), the money received (`other_shop`).
    #[ts(type = "Paisa")]
    pub garden_credit_paisa: Paisa,
    /// The charges (`lot_charges`).
    pub charges: Option<SaleMoneyCharges>,
    /// How the garden's credit was worked out (PRD FR-7.43): `working-sale-money-credit`,
    /// `working-sale-money-lot-charges` or `working-sale-money-other-shop`.
    pub working: Working,
}

/// The totals over the whole filter (never only the page).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SaleMoneyTotals {
    /// Σ نگ.
    #[ts(type = "number")]
    pub qty: i64,
    /// Σ value (our shop and other shops).
    #[ts(type = "Paisa")]
    pub value_paisa: Paisa,
    /// Σ buyers' bills.
    #[ts(type = "Paisa")]
    pub buyer_total_paisa: Paisa,
    /// Σ paid on the spot.
    #[ts(type = "Paisa")]
    pub paid_paisa: Paisa,
    /// Σ settled since.
    #[ts(type = "Paisa")]
    pub collected_paisa: Paisa,
    /// Σ still on the buyers' accounts.
    #[ts(type = "Paisa")]
    pub still_due_paisa: Paisa,
    /// Σ lot charges, 0 or more (already taken off `garden_credit_paisa`).
    #[ts(type = "Paisa")]
    pub lot_charges_paisa: Paisa,
    /// Σ the garden's credit. Over the whole garden it is its money in from our shop and from other
    /// shops (R36).
    #[ts(type = "Paisa")]
    pub garden_credit_paisa: Paisa,
}

/// The answer of `gardens_sale_money`: oldest day first, then بکری, lot and line; a lot's charges
/// after its lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardensSaleMoney {
    /// One page of rows.
    pub rows: Vec<SaleMoneyRow>,
    /// How many rows match the filter.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over the whole filter.
    pub totals: SaleMoneyTotals,
}
