//! What the live preview of a بکری sends back (plan 12 contract).

use serde::Serialize;
use ts_rs::TS;

use super::input::{ChargeSource, FreightPaidBy, LotChargeMethod, PaymentType, StoredMethod};
use crate::cash::CashCheck;
use crate::error::FieldError;
use crate::money::Paisa;
use crate::rules::{RuleLevel, Working};

/// Where a rate hint came from (PRD FR-7.53).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RateHintSource {
    /// An earlier line of this بکری (saved or not).
    ThisBikri,
    /// A saved sale of the same item and packing on the posting day.
    Today,
    /// A saved sale of the same item and packing on the last day it sold.
    LastDay,
}

/// The last rate of an item and packing, shown as grey placeholder text only (PRD FR-7.53, Code
/// Standards B29). It has no `Deserialize`, so it can never be sent back as an input rate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RateHint {
    /// The rate.
    #[ts(type = "Paisa")]
    pub rate_paisa: Paisa,
    /// Where it came from.
    pub source: RateHintSource,
    /// The day of the sale it came from, `YYYY-MM-DD`.
    pub date: String,
}

/// The rule that decided a charge, as a tag the screen can build its words from (the backend
/// writes no text). With the level, the seller or buyer and the item of the line, the screen
/// shows "رحیم + کیلا 5%" (PRD FR-11.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct RuleLabel {
    /// The level of PRD 11.3.
    pub level: RuleLevel,
    /// The rule, or null for a value typed on the line.
    #[ts(type = "number | null")]
    pub rule_id: Option<i64>,
}

/// A commission or لاگا of one sale line, with how it was worked out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LineCharge {
    /// The method kept with the line. `manual` for a value typed on the line.
    pub method: StoredMethod,
    /// The percent in basis points, for `percent`, and for `manual` typed as a percent.
    #[ts(type = "number | null")]
    pub value_bps: Option<i32>,
    /// The amount per نگ or per lot, for `per_unit` and `per_lot`, and for `manual` typed per
    /// نگ.
    #[ts(type = "Paisa | null")]
    pub value_paisa: Option<Paisa>,
    /// The amount, whole rupees.
    #[ts(type = "Paisa")]
    pub amount: Paisa,
    /// The rule that decided, or null.
    #[ts(type = "number | null")]
    pub rule_id: Option<i64>,
    /// The tag of that rule.
    pub rule_label: Option<RuleLabel>,
    /// Where it came from.
    pub source: ChargeSource,
    /// How the amount was worked out (PRD FR-7.43).
    pub working: Working,
}

/// A lot charge (مزدوری, منشیانہ, فون), effective as the screen must show it in the field
/// (NFR-USE-27).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LotCharge {
    /// The method.
    pub method: LotChargeMethod,
    /// What the field shows: the amount for each نگ for `per_unit`, the amount for `per_lot`,
    /// and the amount itself for `manual`.
    #[ts(type = "Paisa")]
    pub value_paisa: Paisa,
    /// The amount charged for the lot, whole rupees.
    #[ts(type = "Paisa")]
    pub amount: Paisa,
    /// The rule that suggested it, or null.
    #[ts(type = "number | null")]
    pub rule_id: Option<i64>,
    /// The tag of that rule.
    pub rule_label: Option<RuleLabel>,
    /// Where it came from.
    pub source: ChargeSource,
    /// How the amount was worked out.
    pub working: Working,
}

/// The `working` of each calculated amount of a sale line (PRD FR-7.43, Code Standards B30).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LineWorkings {
    /// C1 for the seller rate.
    pub seller_amount: Working,
    /// C1 for the buyer rate.
    pub buyer_value: Working,
    /// C5.
    pub buyer_total: Working,
    /// C19.
    pub rate_diff: Working,
}

/// The numbers of one sale line that has everything it needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LineCalc {
    /// Quantity in نگ.
    #[ts(type = "number")]
    pub qty: i64,
    /// Weight sold, in grams, for a weight item.
    #[ts(type = "number | null")]
    pub weight_grams: Option<i64>,
    /// The seller's rate.
    #[ts(type = "Paisa")]
    pub seller_rate: Paisa,
    /// The buyer's rate (the seller's when the rates do not differ).
    #[ts(type = "Paisa")]
    pub buyer_rate: Paisa,
    /// C1 with the seller rate.
    #[ts(type = "Paisa")]
    pub seller_amount: Paisa,
    /// C1 with the buyer rate.
    #[ts(type = "Paisa")]
    pub buyer_value: Paisa,
    /// The seller's commission.
    pub seller_comm: LineCharge,
    /// The buyer's commission.
    pub buyer_comm: LineCharge,
    /// لاگا.
    pub laga: LineCharge,
    /// C5: the buyer value plus his commission plus لاگا.
    #[ts(type = "Paisa")]
    pub buyer_total: Paisa,
    /// C19: the buyer value less the seller amount. Can be negative.
    #[ts(type = "Paisa")]
    pub rate_diff: Paisa,
    /// How the buyer pays.
    pub payment_type: PaymentType,
    /// The working of the four amounts above.
    pub workings: LineWorkings,
}

/// One row of the sale lines, in the order it was sent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LinePreview {
    /// The key sent with the row.
    pub key: String,
    /// The position in the lot, from 1.
    #[ts(type = "number")]
    pub line_no: i64,
    /// True for a row with no buyer and no rate. The totals and the save ignore it.
    pub is_blank: bool,
    /// The quantity a row at this position starts with: the lot's quantity received less the
    /// quantity of the rows before it (PRD FR-7.52).
    #[ts(type = "number")]
    pub remaining_qty: i64,
    /// The last rate of this item and packing, as grey text only (PRD FR-7.53).
    pub rate_hint: Option<RateHint>,
    /// The numbers, or null while the row is not complete.
    pub calc: Option<LineCalc>,
    /// Warnings of this row (also in the list of the whole preview).
    pub warnings: Vec<FieldError>,
}

/// The `working` of each total (PRD FR-7.43, Code Standards B30).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SummaryWorkings {
    /// خام بکری.
    pub gross: Working,
    /// کرایہ.
    pub freight: Working,
    /// کمیشن.
    pub seller_comm: Working,
    /// مزدوری.
    pub labour: Working,
    /// منشیانہ.
    pub munshiana: Working,
    /// نقد.
    pub cash_paid: Working,
    /// کرایہ سٹور.
    pub store_rent: Working,
    /// فون.
    pub phone: Working,
    /// C6.
    pub deductions: Working,
    /// C7.
    pub net: Working,
    /// C8.
    pub seller_credit: Working,
    /// گاہک کمیشن.
    pub buyer_comm: Working,
    /// لاگا.
    pub laga: Working,
    /// ریٹ فرق.
    pub rate_diff: Working,
    /// ادھاری.
    pub buyer_credit: Working,
    /// نقدی.
    pub buyer_cash: Working,
    /// Buyer bills together.
    pub buyer_total: Working,
    /// C9.
    pub shared_income: Working,
}

/// The lines of PRD 12.5 for one lot or for the whole بکری. Only rows that are complete are
/// added.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SummaryTotals {
    /// خام بکری: the sum of the seller amounts.
    #[ts(type = "Paisa")]
    pub gross: Paisa,
    /// کرایہ: the freight paid by the firm.
    #[ts(type = "Paisa")]
    pub freight: Paisa,
    /// کمیشن: the sum of the seller commissions of the lines.
    #[ts(type = "Paisa")]
    pub seller_comm: Paisa,
    /// مزدوری.
    #[ts(type = "Paisa")]
    pub labour: Paisa,
    /// منشیانہ.
    #[ts(type = "Paisa")]
    pub munshiana: Paisa,
    /// نقد.
    #[ts(type = "Paisa")]
    pub cash_paid: Paisa,
    /// کرایہ سٹور.
    #[ts(type = "Paisa")]
    pub store_rent: Paisa,
    /// فون.
    #[ts(type = "Paisa")]
    pub phone: Paisa,
    /// کل اخراجات (C6).
    #[ts(type = "Paisa")]
    pub deductions: Paisa,
    /// صافی بکری (C7). Can be negative.
    #[ts(type = "Paisa")]
    pub net: Paisa,
    /// What the posting credits the seller (C8): the net before نقد. Can be negative.
    #[ts(type = "Paisa")]
    pub seller_credit: Paisa,
    /// گاہک کمیشن.
    #[ts(type = "Paisa")]
    pub buyer_comm: Paisa,
    /// لاگا.
    #[ts(type = "Paisa")]
    pub laga: Paisa,
    /// ریٹ فرق. Can be negative.
    #[ts(type = "Paisa")]
    pub rate_diff: Paisa,
    /// ادھاری: buyer bills of credit lines.
    #[ts(type = "Paisa")]
    pub buyer_credit: Paisa,
    /// نقدی: buyer bills of cash lines.
    #[ts(type = "Paisa")]
    pub buyer_cash: Paisa,
    /// All buyer bills together.
    #[ts(type = "Paisa")]
    pub buyer_total: Paisa,
    /// The income shared between the partners (C9).
    #[ts(type = "Paisa")]
    pub shared_income: Paisa,
    /// The working of each amount.
    pub workings: SummaryWorkings,
}

/// One lot of the preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LotPreview {
    /// The key sent with the lot.
    pub key: String,
    /// The lot number, from 1, in the order sent.
    #[ts(type = "number")]
    pub lot_no: i64,
    /// The freight typed on the lot, even if the seller paid it.
    #[ts(type = "Paisa")]
    pub freight: Paisa,
    /// Who paid the driver.
    pub freight_paid_by: FreightPaidBy,
    /// True once the picked item is sold by weight: the lot and its rows then take a weight
    /// (PRD E23). False if no item is picked yet or it is sold by نگ.
    pub sold_by_weight: bool,
    /// مزدوری as it will be charged, or null while the lot cannot work it out.
    pub labour: Option<LotCharge>,
    /// منشیانہ as it will be charged.
    pub munshiana: Option<LotCharge>,
    /// فون as it will be charged.
    pub phone: Option<LotCharge>,
    /// Quantity of the rows that have a valid quantity (blank rows left out). The same sum as the
    /// `remaining_qty` of the rows and the warning `ERR_QTY_LEFT_UNSOLD` use.
    #[ts(type = "number")]
    pub sold_qty: i64,
    /// Quantity received less quantity sold, 0 if more was sold (that row has
    /// `ERR_QTY_EXCEEDS_LOT`). Null while the quantity received is not valid.
    #[ts(type = "number | null")]
    pub balance_qty: Option<i64>,
    /// True if the net is below 0: the seller owes the firm (PRD FR-12.7).
    pub net_is_negative: bool,
    /// The lot's lines of PRD 12.5, or null while the lot cannot work them out.
    pub totals: Option<SummaryTotals>,
    /// The rows, in the order sent. Blank rows are included.
    pub lines: Vec<LinePreview>,
}

/// One partner's part of the income of this بکری (PRD FR-17.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SplitPart {
    /// The partner.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// His share, in basis points.
    pub share_bps: i32,
    /// His part of the shared income.
    #[ts(type = "Paisa")]
    pub income_paisa: Paisa,
    /// How it was worked out.
    pub working: Working,
}

/// Everything the live preview sends back. Nothing is saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BikriPreview {
    /// The lots, in the order sent.
    pub lots: Vec<LotPreview>,
    /// The lines of PRD 12.5 for the whole بکری ("پوری بکری").
    pub summary: SummaryTotals,
    /// The live partner split of the shared income.
    pub split: Vec<SplitPart>,
    /// Cash in hand after this بکری (PRD FR-6.7).
    pub cash: CashCheck,
    /// True if a commission or لاگا may be typed on a line (PRD FR-11.10). The rate tag cells
    /// are editable only then.
    pub allow_line_override: bool,
    /// What stops a save. The screen shows each at its field.
    pub errors: Vec<FieldError>,
    /// What only warns. A save is allowed.
    pub warnings: Vec<FieldError>,
}

/// The truck's earlier بکری (PRD FR-7.49).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TruckLast {
    /// Its بکری number.
    #[ts(type = "number")]
    pub bikri_number: i64,
    /// Its day, `YYYY-MM-DD`.
    pub date: String,
    /// The truck stand.
    #[ts(type = "number | null")]
    pub adda_id: Option<i64>,
    /// The زمیندار.
    #[ts(type = "number | null")]
    pub zamindar_account_id: Option<i64>,
    /// The driver's name.
    pub driver_name: Option<String>,
    /// The driver's phone.
    pub driver_phone: Option<String>,
}

/// A بکری of this truck on the posting day (PRD FR-7.50).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TruckSameDay {
    /// Its بکری number.
    #[ts(type = "number")]
    pub bikri_number: i64,
}

/// A truck number used before, for the list under the field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TruckSuggestion {
    /// The truck number as saved.
    pub truck_no: String,
    /// The day of its last بکری, `YYYY-MM-DD`.
    pub date: String,
}

/// What the app remembers about a truck (PRD FR-7.49, FR-7.50). Read from saved بکری.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TruckMemory {
    /// The last بکری of exactly this truck number (ignoring letter case), if any.
    pub last: Option<TruckLast>,
    /// A saved, not cancelled بکری of this truck on the posting day, if any.
    pub same_day: Option<TruckSameDay>,
    /// Truck numbers that start with the typed text, newest first, at most 8.
    pub suggestions: Vec<TruckSuggestion>,
}

/// What the app remembers about a seller (PRD FR-7.51). Every value is put into the visible
/// field like a typed value (NFR-USE-27).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SellerMemory {
    /// The item of his last lot.
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// The packing of his last lot.
    #[ts(type = "number | null")]
    pub packing_id: Option<i64>,
    /// His default مارکہ, else the مارکہ of his last lot.
    #[ts(type = "number | null")]
    pub marka_id: Option<i64>,
    /// True if the item and packing came from a lot he sent before.
    pub from_last_lot: bool,
}
