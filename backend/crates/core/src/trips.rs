//! Market trips (مارکیٹ سفر, PRD 12.10, FR-12.29 to FR-12.33, plan 16b): what the trip screens send
//! and read back. A trip is made with its partners and their shares first, then each partner's
//! money source, then the details (PRD FR-12.29). The goods bought on it are in `purchases`.
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `trips_defaults` | none | `TripDefaults` |
//! | `trips_create` | `TripInput` | `TripRow` |
//! | `trips_set_partners` | `TripSetPartnersInput` | `TripRow` |
//! | `trips_list` | `TripsListInput` | `TripsList` |
//! | `trips_get` | `TripGetInput` | `TripDetail` |
//! | `trips_summary` | none | `TripsSummary` |
//! | `trips_cost_preview` / `trips_cost_save` | `TripCostInput` | `TripCostPreview` / `TripCostSaved` |
//! | `trips_cost_cancel` | `TripEntryCancelInput` | `TripEntryCancelled` |
//! | `trips_other_sale_preview` / `trips_other_sale_save` | `OtherSaleInput` | `OtherSalePreview` / `OtherSaleSaved` |
//! | `trips_other_sale_cancel` | `TripEntryCancelInput` | `TripEntryCancelled` |
//!
//! Lists return one page with the row count and the totals over the whole filter (Code Standards
//! B31, PRD FR-19.11). A trip's number is its own series (1, 2, 3 ...) and its code, the number a screen
//! shows, is `0070`, `0071` ... (type 007, plan 34), the same in both languages; trip costs and sales in another shop have their own series too (letters from Fluent:
//! trip cost `TC-1`, sale in another shop `OS-1`, purchase `PUR-1`).
//!
//! Worked numbers (PRD E2E-41): a trip with الف 50% from the shop and ب 50% from his pocket buys 80
//! پیٹی مالٹا at 1,200 (96,000: parts 48,000 / 48,000), then a truck cost of 4,000 is spread over its
//! one purchase (C24: 4,000), parts 2,000 / 2,000. The cost is 100,000, 1,250 for each نگ.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::cash::CashCheck;
use crate::error::FieldError;
use crate::money::Paisa;
use crate::purchases::PurchaseRow;
use crate::rules::Working;
use crate::ventures::{
    EntryPart, EntryPartInput, EntryParts, ShareAmount, VenturePartnerInput, VenturePartnerRow,
    VentureStatus,
};

// ---- The trip itself ----

/// A new trip (PRD FR-12.29, 12.10). Shares and sources first, then the details.
///
/// Field paths in errors: `date`, `market`, `sent_person`, `note`, `partners`,
/// `partners[i].partner_id`, `partners[i].share`, `partners[i].source`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct TripInput {
    /// The day, `DD-MM-YYYY` or a shortcut. An open day (PRD FR-6.6).
    pub date: String,
    /// The market the goods are bought in, as typed (PRD FR-7.25).
    #[serde(default)]
    pub market: Option<String>,
    /// Who was sent, as typed.
    #[serde(default)]
    pub sent_person: Option<String>,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub note: Option<String>,
    /// The partners in the trip with their shares and default sources. The shares total exactly 100%
    /// (`ERR_SHARES_NOT_100` on `partners`, value `total_bps`). Rows with a share of 0 are skipped.
    pub partners: Vec<VenturePartnerInput>,
}

/// New shares for a trip that has no entry yet (PRD FR-12.29). After its first entry the shares are
/// fixed: `ERR_VENTURE_SHARES_LOCKED` on `partners`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct TripSetPartnersInput {
    /// The trip. `ERR_NOT_FOUND` on `id`.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read. `ERR_VERSION_CHANGED` on `version`.
    #[ts(type = "number")]
    pub version: i64,
    /// The partners, as in `TripInput`.
    pub partners: Vec<VenturePartnerInput>,
}

/// The form of a new trip, filled from the last trip (PRD A56): the partners with the % and the
/// sources of the last trip. For the first trip, the active partners with the firm's shares of today
/// and the source `shop`. A partner who is not in the last trip is listed with a share of 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripDefaults {
    /// Every active partner, in the order the partners were created.
    pub partners: Vec<VenturePartnerRow>,
    /// True when the shares and sources come from the last trip, false when they are the firm's.
    pub from_last_trip: bool,
}

/// What the lists of trips add up to, over the whole filter (B31). Money in is what the partners
/// paid; money back is the seller credits and other-shop money they received (PRD FR-12.27).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripsTotals {
    /// Σ نگ left over the trips.
    #[ts(type = "number")]
    pub qty_left: i64,
    /// Σ cost left.
    #[ts(type = "Paisa")]
    pub cost_left_paisa: Paisa,
    /// Σ total cost of the purchases.
    #[ts(type = "Paisa")]
    pub cost_paisa: Paisa,
    /// Σ sales money (seller credits and other-shop money).
    #[ts(type = "Paisa")]
    pub sales_paisa: Paisa,
    /// Σ result.
    #[ts(type = "Paisa")]
    pub result_paisa: Paisa,
}

/// Input of `trips_list` (PRD FR-19.10, FR-19.11). Dates are `DD-MM-YYYY` or a shortcut; null or
/// empty means no limit. They filter on the day of the trip.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct TripsListInput {
    /// First day.
    #[serde(default)]
    pub from: Option<String>,
    /// Last day.
    #[serde(default)]
    pub to: Option<String>,
    /// Only open or only closed trips.
    #[serde(default)]
    pub status: Option<VentureStatus>,
    /// Only trips this partner is in.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
    /// Text to find in the trip's name, market or sent person.
    #[serde(default)]
    pub search: Option<String>,
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives the default page of 100 rows (PRD FR-19.11).
    #[serde(default)]
    pub limit: u32,
}

/// One trip with its figures (PRD FR-12.25, FR-12.27). All figures are read from stored rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripRow {
    /// The trip.
    #[ts(type = "number")]
    pub id: i64,
    /// Its version, for `trips_set_partners`.
    #[ts(type = "number")]
    pub version: i64,
    /// Its number in its own series, from 1.
    #[ts(type = "number")]
    pub number: i64,
    /// Its code, the number a screen shows (plan 34): `0070` for the first trip, `0071` for the
    /// second. Type 007, the number less one, no padding.
    pub code: String,
    /// Urdu name, made from the code and the date.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The market, as typed.
    pub market: Option<String>,
    /// Who was sent, as typed.
    pub sent_person: Option<String>,
    /// A note.
    pub note: Option<String>,
    /// Open or closed. A trip closes by itself once it has a purchase and nothing is left, and opens
    /// again if a sale or a write-off of it is reversed (PRD FR-12.33).
    pub status: VentureStatus,
    /// The day it closed, `YYYY-MM-DD`.
    pub closed_date: Option<String>,
    /// True once the trip has an entry: its shares can no longer change (PRD FR-12.29).
    pub shares_locked: bool,
    /// The partners in the trip with their shares and default sources.
    pub partners: Vec<VenturePartnerRow>,
    /// How many saved purchases it has.
    #[ts(type = "number")]
    pub purchase_count: i64,
    /// نگ bought.
    #[ts(type = "number")]
    pub qty_bought: i64,
    /// نگ sold (our shop and other shops).
    #[ts(type = "number")]
    pub qty_sold: i64,
    /// نگ written off.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// نگ left.
    #[ts(type = "number")]
    pub qty_left: i64,
    /// Total cost of the saved purchases (paid now plus the trip costs spread over them).
    #[ts(type = "Paisa")]
    pub cost_paisa: Paisa,
    /// The cost of the نگ left.
    #[ts(type = "Paisa")]
    pub cost_left_paisa: Paisa,
    /// Sales money in our shop: the seller credits of its own lots.
    #[ts(type = "Paisa")]
    pub sales_paisa: Paisa,
    /// Money from sales in other shops.
    #[ts(type = "Paisa")]
    pub other_shop_paisa: Paisa,
    /// C21: the cost of the goods sold.
    #[ts(type = "Paisa")]
    pub cost_sold_paisa: Paisa,
    /// The cost of the goods written off.
    #[ts(type = "Paisa")]
    pub cost_written_off_paisa: Paisa,
    /// The result: sales money plus other-shop money less the cost sold and written off (PRD
    /// FR-12.27). Negative is a loss.
    #[ts(type = "Paisa")]
    pub result_paisa: Paisa,
    /// How the result was worked out.
    pub result_working: Working,
}

/// The answer of `trips_list`: one page and the totals over the whole filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripsList {
    /// This page: newest day first, then by number.
    pub rows: Vec<TripRow>,
    /// How many trips match the filter.
    pub total_count: u32,
    /// The totals over all of them.
    pub totals: TripsTotals,
}

/// Input of `trips_get`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct TripGetInput {
    /// The trip. `ERR_NOT_FOUND` on `id`.
    #[ts(type = "number")]
    pub id: i64,
    /// How many entries to skip in `entries` (the trip's history, newest first). 0 for the first
    /// page.
    #[serde(default)]
    pub entries_offset: u32,
    /// How many entries to return. 0 gives the default page of 100.
    #[serde(default)]
    pub entries_limit: u32,
}

/// The kind of an entry on a trip's history (PRD FR-7.67: the type list of the trip screen).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TripEntryKind {
    /// خریداری: shown by its own purchase row.
    Purchase,
    /// سفر خرچ: a cost of the whole trip.
    Cost,
    /// دوسری دکان میں فروخت: a sale of trip goods in another shop.
    OtherSale,
}

impl TripEntryKind {
    /// The stored form (`venture_entries.kind`): `purchase`, `cost` or `other_sale`.
    pub const fn as_str(self) -> &'static str {
        match self {
            TripEntryKind::Purchase => "purchase",
            TripEntryKind::Cost => "cost",
            TripEntryKind::OtherSale => "other_sale",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "purchase" => Some(TripEntryKind::Purchase),
            "cost" => Some(TripEntryKind::Cost),
            "other_sale" => Some(TripEntryKind::OtherSale),
            _ => None,
        }
    }
}

/// How a trip cost was spread over a purchase (PRD C24).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CostSpreadPart {
    /// The purchase.
    #[ts(type = "number")]
    pub purchase_id: i64,
    /// Its number (خ-1, PUR-1).
    #[ts(type = "number")]
    pub purchase_number: i64,
    /// The item.
    pub item: RecordRef,
    /// نگ bought: the weight of this purchase in the spread.
    #[ts(type = "number")]
    pub qty: i64,
    /// Its part of the cost. The parts add up to the cost.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// How the part was worked out (CalcInfo).
    pub working: Working,
}

/// One entry on a trip's history, newest first (PRD FR-7.67). A purchase appears here too, with its
/// parts; its full figures are in `TripDetail::purchases`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripEntryRow {
    /// The entry (`venture_entries.id`). `trips_cost_cancel` and `trips_other_sale_cancel` take it.
    #[ts(type = "number")]
    pub id: i64,
    /// Its version, for a cancel.
    #[ts(type = "number")]
    pub version: i64,
    /// What it is.
    pub kind: TripEntryKind,
    /// Its number in the series of its kind.
    #[ts(type = "number")]
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The payment, or the money received for a sale in another shop.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// Why (a cost).
    pub reason: Option<String>,
    /// The shop, as typed (a sale in another shop).
    pub shop_name: Option<String>,
    /// نگ sold (a sale in another shop).
    #[ts(type = "number | null")]
    pub qty: Option<i64>,
    /// The purchase it belongs to (a purchase, or a sale in another shop): id and number.
    #[ts(type = "number | null")]
    pub purchase_id: Option<i64>,
    /// Its number.
    #[ts(type = "number | null")]
    pub purchase_number: Option<i64>,
    /// C21: the cost of the goods sold in another shop. 0 for the other kinds.
    #[ts(type = "Paisa")]
    pub cost_paisa: Paisa,
    /// The journal entry. Null when every part was from a pocket (PRD FR-12.30).
    #[ts(type = "number | null")]
    pub journal_entry_id: Option<i64>,
    /// Saved or cancelled.
    pub cancelled: bool,
    /// Why it was cancelled.
    pub cancel_reason: Option<String>,
    /// The parts of a payment (C23), with their sources. Empty for a sale in another shop.
    pub parts: Vec<EntryPart>,
    /// The credits of a sale in another shop (C25). Empty for a payment.
    pub credits: Vec<ShareAmount>,
    /// How a trip cost was spread over the purchases (C24). Empty for the other kinds.
    pub spread: Vec<CostSpreadPart>,
    /// True if `trips_cost_cancel` or `trips_other_sale_cancel` (or `purchases_cancel` for a
    /// purchase) would accept it now, apart from a closed day.
    pub can_cancel: bool,
}

/// One partner's money in a trip and what he got back (PRD FR-12.27, E2E-34b).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripPartnerResult {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His share in bps.
    pub share_bps: i32,
    /// What he paid from the shop's cash: it came off his کل رقم when it was paid.
    #[ts(type = "Paisa")]
    pub money_in_shop_paisa: Paisa,
    /// What he paid from his own pocket: recorded only.
    #[ts(type = "Paisa")]
    pub money_in_pocket_paisa: Paisa,
    /// What he got back: his parts of the seller credits (C25) and of the money of sales in other
    /// shops.
    #[ts(type = "Paisa")]
    pub money_back_paisa: Paisa,
    /// His part of the result so far: money back less money in (shop and pocket). E2E-34b:
    /// 3,384 − 4,160 = −776.
    #[ts(type = "Paisa")]
    pub net_paisa: Paisa,
    /// How the net was worked out (CalcInfo).
    pub working: Working,
}

/// One trip with its purchases, history and each partner's money (PRD FR-7.67, FR-12.25, FR-12.27).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripDetail {
    /// The trip and its figures.
    pub row: TripRow,
    /// Its purchases, oldest first, with stock, cost left and result.
    pub purchases: Vec<PurchaseRow>,
    /// One page of its history: purchases, trip costs and sales in other shops, newest first.
    pub entries: Vec<TripEntryRow>,
    /// How many entries it has in all.
    #[ts(type = "number")]
    pub entries_count: i64,
    /// Each partner's money in and money back.
    pub partner_results: Vec<TripPartnerResult>,
}

/// The data of the dashboard tile مارکیٹ سفر (PRD 9.1, FR-12.25): the open trips, the نگ of their
/// goods not yet sold and the cost left. D5 shows the tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct TripsSummary {
    /// How many trips are open.
    #[ts(type = "number")]
    pub open_trips: i64,
    /// How many saved purchases still have stock.
    #[ts(type = "number")]
    pub purchases_with_stock: i64,
    /// نگ left over all of them.
    #[ts(type = "number")]
    pub qty_left: i64,
    /// Their cost left (C21).
    #[ts(type = "Paisa")]
    pub cost_left_paisa: Paisa,
}

// ---- A trip cost (PRD FR-12.31) ----

/// A cost of the whole trip (truck, food), as typed. It is paid in as in FR-12.30 and spread over the
/// trip's purchases that still have goods left (C24).
///
/// Field paths in errors: `trip_id`, `date`, `amount`, `reason`, `parts`, `parts[i].partner_id`,
/// `parts[i].source`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct TripCostInput {
    /// The trip. `ERR_NOT_FOUND`; `ERR_TRIP_CLOSED`; `ERR_TRIP_NO_STOCK` (no purchase has goods left).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub trip_id: Option<i64>,
    /// The day. An open day, not before the trip's day.
    pub date: String,
    /// The cost in whole rupees. A simple sum is taken.
    pub amount: String,
    /// Why (the truck, the food). Required: `ERR_REASON_REQUIRED` on `reason`.
    pub reason: String,
    /// The source of each partner's part on this entry (PRD A54). Empty takes the trip's defaults.
    #[serde(default)]
    pub parts: Vec<EntryPartInput>,
    /// The Owner's answer to the cash question (PRD FR-6.7). Used by the save only.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// The numbers of a trip cost that is complete enough to work out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripCostNumbers {
    /// The cost.
    #[ts(type = "Paisa")]
    pub amount: Paisa,
    /// Each partner's part (C23), its source, and what leaves روکڑ.
    pub parts: EntryParts,
    /// How the cost is spread over the purchases with goods left (C24). Each purchase's cost left
    /// goes up by its part.
    pub spread: Vec<CostSpreadPart>,
}

/// What the preview of a trip cost sends back. Nothing is saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripCostPreview {
    /// The numbers, or null while the cost cannot work them out.
    pub numbers: Option<TripCostNumbers>,
    /// What the posting does to cash in hand (PRD FR-6.7).
    pub cash: CashCheck,
    /// What stops a save, each at its field.
    pub errors: Vec<FieldError>,
    /// What only warns.
    pub warnings: Vec<FieldError>,
}

/// What `trips_cost_save` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripCostSaved {
    /// The entry (`venture_entries.id`), to cancel it.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number, from 1 in the series of trip costs.
    #[ts(type = "number")]
    pub number: i64,
    /// Its version, for a cancel.
    #[ts(type = "number")]
    pub version: i64,
    /// The journal entry. Null when every part is from a pocket.
    #[ts(type = "number | null")]
    pub entry_id: Option<i64>,
    /// The preview of the saved numbers.
    pub preview: TripCostPreview,
}

// ---- A sale in another shop (PRD FR-12.32) ----

/// A sale of trip goods in another shop, as typed: date, shop, purchase, qty and the money received.
/// The qty and its cost (C21) come off the purchase; the money comes into روکڑ and goes to the trip's
/// partners by their % (C25).
///
/// Field paths in errors: `trip_id`, `date`, `shop`, `purchase_id`, `qty`, `amount`, `note`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct OtherSaleInput {
    /// The trip. `ERR_NOT_FOUND`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub trip_id: Option<i64>,
    /// The day. An open day, not before the purchase's day.
    pub date: String,
    /// The shop, as typed (PRD FR-7.25).
    #[serde(default)]
    pub shop: Option<String>,
    /// The purchase the goods come from. It must be a purchase of this trip with stock left
    /// (`ERR_QTY_EXCEEDS_LOT`, value `available`, when it has fewer نگ).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub purchase_id: Option<i64>,
    /// نگ sold. A simple sum is taken.
    pub qty: String,
    /// The money received, whole rupees (PRD FR-12.32: it comes into روکڑ).
    pub amount: String,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub note: Option<String>,
}

/// The numbers of a sale in another shop that is complete enough to work out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OtherSaleNumbers {
    /// The money received.
    #[ts(type = "Paisa")]
    pub amount: Paisa,
    /// نگ sold.
    #[ts(type = "number")]
    pub qty: i64,
    /// C21: the cost of the goods sold. Stored, never posted.
    #[ts(type = "Paisa")]
    pub cost_paisa: Paisa,
    /// How the cost was worked out.
    pub cost_working: Working,
    /// The result of this sale: the money less the cost. Negative is a loss.
    #[ts(type = "Paisa")]
    pub result_paisa: Paisa,
    /// How the result was worked out.
    pub result_working: Working,
    /// Each trip partner's part of the money (C25).
    pub credits: Vec<ShareAmount>,
    /// نگ left in the purchase after this sale.
    #[ts(type = "number")]
    pub qty_left_after: i64,
    /// نگ left for another shop after this sale (PRD FR-12.41).
    #[ts(type = "number")]
    pub qty_other_left_after: i64,
    /// نگ left for our shop after this sale (our part less what the sale moves from it).
    #[ts(type = "number")]
    pub qty_our_left_after: i64,
    /// نگ this sale moves from our shop's part to the part for another shop (plan 35 D1): the sale
    /// takes its qty from the part for another shop first, and what that part does not have is
    /// moved from our shop's part in the same save. 0 when the part for another shop has enough.
    /// More than the purchase has left in all is `ERR_QTY_EXCEEDS_LOT` with `available` = the
    /// whole qty left. A cancel moves them back.
    #[ts(type = "number")]
    pub qty_from_our_shop: i64,
    /// The cost of the نگ left after this sale.
    #[ts(type = "Paisa")]
    pub cost_left_after_paisa: Paisa,
}

/// What the preview of a sale in another shop sends back. Nothing is saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OtherSalePreview {
    /// The numbers, or null while the sale cannot work them out.
    pub numbers: Option<OtherSaleNumbers>,
    /// What stops a save, each at its field.
    pub errors: Vec<FieldError>,
    /// What only warns.
    pub warnings: Vec<FieldError>,
}

/// What `trips_other_sale_save` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OtherSaleSaved {
    /// The entry (`venture_entries.id`), to cancel it.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number, from 1 in the series of sales in other shops.
    #[ts(type = "number")]
    pub number: i64,
    /// Its version, for a cancel.
    #[ts(type = "number")]
    pub version: i64,
    /// The journal entry: روکڑ بنام, each trip partner's 0604 جمع his part.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// True if this sale closed the trip (PRD FR-12.33).
    pub trip_closed: bool,
    /// The preview of the saved numbers.
    pub preview: OtherSalePreview,
}

// ---- Cancelling an entry of a trip ----

/// Cancels a trip cost or a sale in another shop by reversal (PRD FR-12.31, FR-12.32). A purchase is
/// cancelled with `purchases_cancel`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct TripEntryCancelInput {
    /// The entry (`TripEntryRow::id`). `ERR_NOT_FOUND` on `id`; `ERR_TRIP_ENTRY_CANCELLED` if it is
    /// cancelled already. A trip cost: `ERR_TRIP_COST_HAS_SALES` (values `qty_sold`,
    /// `qty_written_off`) if any purchase it was spread over has been sold or written off.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read. `ERR_VERSION_CHANGED` on `version` if it moved.
    #[ts(type = "number")]
    pub version: i64,
    /// Why. Required: `ERR_REASON_REQUIRED` on `reason`.
    pub reason: String,
    /// The answer to the cash question: a cancel of a sale in another shop takes the money out of
    /// روکڑ again (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// What a cancel of a trip entry gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripEntryCancelled {
    /// The entry.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number (kept, never used again).
    #[ts(type = "number")]
    pub number: i64,
    /// The reversal entry. Null when the entry had no journal entry.
    #[ts(type = "number | null")]
    pub reversal_entry_id: Option<i64>,
    /// True if this cancel opened the trip again (PRD FR-12.33).
    pub trip_reopened: bool,
}
