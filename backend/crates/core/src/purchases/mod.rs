//! Goods bought on a market trip (PRD 12.10, FR-12.24 to FR-12.28, plan 16b): what the خریداری
//! screen of a trip sends and reads back. The goods belong to the partners in the trip, not to the
//! firm. The shop sells them in a بکری (the lot's seller is account 030119998 with a `purchase_id`,
//! see `bikri::LotInput`) or the partners sell them in another shop (`trips_other_sale_*`), and every
//! purchase has a result. Spoiled goods are written off. The cost of the goods sold or written off
//! is the purchase's cost left over the نگ left (PRD C21, A45). The cost is shown, never posted.
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `purchases_preview` | `PurchaseInput` | `PurchasePreview` |
//! | `purchases_save` | `PurchaseInput` | `PurchaseSaved` |
//! | `purchases_cancel` | `PurchaseCancelInput` | `PurchaseCancelled` |
//! | `purchases_list` | `PurchasesListInput` | `PurchasesList` |
//! | `purchases_get` | `PurchaseGetInput` | `PurchaseRow` |
//! | `purchases_write_off` | `PurchaseWriteOffInput` | `PurchaseWriteOffDone` |
//! | `purchases_write_off_undo` | `PurchaseWriteOffUndoInput` | `PurchaseWriteOffUndone` |
//! | `purchases_set_sell_date` | `SellDateInput` (in `arrivals`) | `SellDateSet` (in `arrivals`) |
//!
//! Money is paisa and arrives as text; names are `RecordRef` with both scripts (Code Standards
//! F38). Purchase numbers are their own series, shown as خ-1 in Urdu and PUR-1 in English (letters
//! from Fluent).
//!
//! Worked numbers (PRD E2E-34): 20 پیٹی مالٹا at 500 is a value of 10,000; freight 400 paid at once
//! makes a total cost of 10,400 (520 per نگ). Paid from the shop by three partners at 40 / 30 / 30 it
//! is 4,160 / 3,120 / 3,120 (C23). Selling 15 of them takes cost 10,400 × 15 ÷ 20 = 7,800 (C21); the
//! last 5 written off take the 2,600 that is left.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::bikri::WriteOffRow;
use crate::cash::CashCheck;
use crate::error::FieldError;
use crate::money::Paisa;
use crate::rules::Working;
use crate::ventures::{EntryPart, EntryPartInput, EntryParts, TripRef};

/// Whether a purchase stands or was cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseStatus {
    /// In force.
    Saved,
    /// Cancelled by a reversal (PRD FR-12.24). Its number is kept and never used again.
    Cancelled,
}

impl PurchaseStatus {
    /// The stored form (`purchases.status`).
    pub const fn as_str(self) -> &'static str {
        match self {
            PurchaseStatus::Saved => "saved",
            PurchaseStatus::Cancelled => "cancelled",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "saved" => Some(PurchaseStatus::Saved),
            "cancelled" => Some(PurchaseStatus::Cancelled),
            _ => None,
        }
    }
}

/// The state filter of `purchases_list`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseStateFilter {
    /// Saved purchases with stock left (more than 0 نگ). The purchase picker of an own lot uses it.
    InStock,
    /// Saved purchases with nothing left.
    SoldOut,
    /// Cancelled purchases.
    Cancelled,
}

/// A purchase as typed on the خریداری screen of a trip (PRD 12.10). Everything is text; the backend
/// checks every value. The same input is used for the preview and the save.
///
/// Field paths in errors and warnings: `trip_id`, `date`, `supplier`, `item_id`, `packing_id`,
/// `qty`, `weight`, `rate`, `freight`, `labour`, `other_cost`, `note`, `parts`, `parts[i].partner_id`,
/// `parts[i].source`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PurchaseInput {
    /// The trip. Required. `ERR_NOT_FOUND`; `ERR_TRIP_CLOSED` (a closed trip takes no purchase).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub trip_id: Option<i64>,
    /// The day, `DD-MM-YYYY` or a shortcut. Must be an open day (PRD FR-6.6) and not before the
    /// trip's day (`ERR_PURCHASE_BEFORE_TRIP`).
    pub date: String,
    /// Who the goods were bought from, as typed (PRD FR-7.25). Nothing is owed to him: a purchase is
    /// paid at once (PRD A53).
    #[serde(default)]
    pub supplier: Option<String>,
    /// The item.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// The packing.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub packing_id: Option<i64>,
    /// Quantity in نگ. A simple sum such as `20+25` is taken (PRD FR-7.48).
    #[serde(default)]
    pub qty: String,
    /// Weight in kg, up to 3 decimals. Required for an item sold by weight.
    #[serde(default)]
    pub weight: Option<String>,
    /// The day to sell, `DD-MM-YYYY` or a shortcut (PRD FR-12.39). Empty means the purchase day.
    /// Never before the purchase day: `ERR_SELL_DATE_BEFORE_ENTRY` on `sell_date`. A new بکری from
    /// this day opens with a ready lot for the goods left.
    #[serde(default)]
    pub sell_date: Option<String>,
    /// The rate for each نگ, or for each kg for a weight item. Whole rupees.
    #[serde(default)]
    pub rate: String,
    /// Freight, paid at once and added to the cost (PRD FR-12.24).
    #[serde(default)]
    pub freight: Option<String>,
    /// Labour, paid at once and added to the cost.
    #[serde(default)]
    pub labour: Option<String>,
    /// Other cost, paid at once and added to the cost.
    #[serde(default)]
    pub other_cost: Option<String>,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub note: Option<String>,
    /// The source of each partner's part on this entry, one row for every partner in the trip. Filled
    /// from `trips_defaults` or the trip's last entry, and changeable here (PRD A54). An empty list
    /// takes the trip's default sources.
    #[serde(default)]
    pub parts: Vec<EntryPartInput>,
    /// The Owner's answer to the cash question (PRD FR-6.7). Used by the save only. Asked when the
    /// shop parts take روکڑ below 0.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// The working of each amount of a purchase (PRD FR-7.43, Code Standards B30).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PurchaseWorkings {
    /// C1: `qty × rate`, or the kilograms times the rate for a weight item.
    pub value: Working,
    /// Value plus freight, labour and other cost.
    pub paid_now: Working,
    /// Paid now plus the part of the trip costs. At the time of the purchase there is none, so it is
    /// paid now (PRD FR-12.31).
    pub total_cost: Working,
    /// The average cost for each نگ (A45), for display only.
    pub avg_cost: Working,
}

/// The numbers of a purchase that is complete enough to work out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PurchaseNumbers {
    /// قیمت: C1.
    #[ts(type = "Paisa")]
    pub value: Paisa,
    /// کرایہ, 0 if empty.
    #[ts(type = "Paisa")]
    pub freight: Paisa,
    /// مزدوری, 0 if empty.
    #[ts(type = "Paisa")]
    pub labour: Paisa,
    /// دیگر خرچ, 0 if empty.
    #[ts(type = "Paisa")]
    pub other_cost: Paisa,
    /// ادا شدہ: value plus freight, labour and other cost. It is split by the trip's % (C23).
    #[ts(type = "Paisa")]
    pub paid_now: Paisa,
    /// کل لاگت: paid now plus its part of the trip costs (0 when it is bought). This is the cost of
    /// the stock.
    #[ts(type = "Paisa")]
    pub total_cost: Paisa,
    /// The average cost for each نگ, rounded to the rupee (display only, PRD A45).
    #[ts(type = "Paisa")]
    pub avg_cost: Paisa,
    /// Each partner's part of what is paid now, where it comes from, and what leaves روکڑ (the shop
    /// parts). Pocket parts post nothing (PRD FR-12.30).
    pub parts: EntryParts,
    /// The working of the amounts above.
    pub workings: PurchaseWorkings,
}

/// What the preview of a purchase sends back. Nothing is saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PurchasePreview {
    /// The numbers, or null while the purchase cannot work them out (a field is missing or wrong).
    pub numbers: Option<PurchaseNumbers>,
    /// What the posting does to cash in hand (PRD FR-6.7): the shop parts count, a purchase with
    /// only pocket parts leaves cash as it is.
    pub cash: CashCheck,
    /// What stops a save, each at its field.
    pub errors: Vec<FieldError>,
    /// What only warns.
    pub warnings: Vec<FieldError>,
}

/// What `purchases_save` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PurchaseSaved {
    /// The purchase.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number, from 1 in its own series.
    #[ts(type = "number")]
    pub number: i64,
    /// Its version, for a cancel.
    #[ts(type = "number")]
    pub version: i64,
    /// The journal entry of the posting. Null when every part is from a pocket (PRD FR-12.30).
    #[ts(type = "number | null")]
    pub entry_id: Option<i64>,
    /// The total cost: the cost of the stock.
    #[ts(type = "Paisa")]
    pub total_cost: Paisa,
    /// The preview of the saved numbers.
    pub preview: PurchasePreview,
}

/// Input of `purchases_cancel` (PRD FR-12.24): by reversal, with a reason, only while nothing of the
/// purchase was sold or written off.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PurchaseCancelInput {
    /// The purchase. `ERR_NOT_FOUND` on `id`; `ERR_PURCHASE_CANCELLED` if already cancelled;
    /// `ERR_PURCHASE_HAS_SALES` (values `qty_sold`, `qty_written_off`) if any of it was sold or
    /// written off; `ERR_PURCHASE_CARRIES_TRIP_COST` if it carries a part of a trip cost.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read. `ERR_VERSION_CHANGED` on `version` if it moved.
    #[ts(type = "number")]
    pub version: i64,
    /// Why. Required: `ERR_REASON_REQUIRED` on `reason`.
    pub reason: String,
    /// The answer to the cash question: a cancel gives cash back, so this is asked only if cash
    /// would go below 0 for another reason (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// What `purchases_cancel` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PurchaseCancelled {
    /// The purchase.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number (kept, never used again).
    #[ts(type = "number")]
    pub number: i64,
    /// The reversal entry. Null when the purchase had no journal entry (all parts from pockets).
    #[ts(type = "number | null")]
    pub reversal_entry_id: Option<i64>,
}

/// Input of `purchases_get`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PurchaseGetInput {
    /// The purchase. `ERR_NOT_FOUND` on `id`.
    #[ts(type = "number")]
    pub id: i64,
}

/// Input of `purchases_list` (PRD FR-12.25, FR-19.10, FR-19.11, B31). Dates are `DD-MM-YYYY` or a
/// shortcut; null or empty means no limit. They filter on the day bought.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct PurchasesListInput {
    /// Only the purchases of this trip.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub trip_id: Option<i64>,
    /// First day.
    #[serde(default)]
    pub from: Option<String>,
    /// Last day.
    #[serde(default)]
    pub to: Option<String>,
    /// Only this item.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// Only this state. Null lists every purchase.
    #[serde(default)]
    pub state: Option<PurchaseStateFilter>,
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives the default page of 100 rows (PRD FR-19.11).
    #[serde(default)]
    pub limit: u32,
}

/// An own lot that took goods from a purchase, for the purchase's view (PRD FR-12.27).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PurchaseLotRef {
    /// The بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub bikri_number: i64,
    /// The lot inside it.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// The lot number, from 1.
    #[ts(type = "number")]
    pub lot_no: i64,
    /// The day of the sale, `YYYY-MM-DD`.
    pub entry_date: String,
    /// نگ sold on the lines in force.
    #[ts(type = "number")]
    pub qty_sold: i64,
    /// C21: the cost of the goods sold on those lines.
    #[ts(type = "Paisa")]
    pub cost_paisa: Paisa,
    /// Sales money of the lot: its seller credit (C8), which goes to the trip's partners (C25).
    #[ts(type = "Paisa")]
    pub sales_paisa: Paisa,
}

/// One purchase with its running figures and its result (PRD FR-12.25, FR-12.27). It is the data of
/// report R33 (D5). In `purchases_list` the lists `lots` and `write_offs` are empty;
/// `purchases_get` fills them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PurchaseRow {
    /// The purchase.
    #[ts(type = "number")]
    pub id: i64,
    /// Its version.
    #[ts(type = "number")]
    pub version: i64,
    /// Its number, from 1 (خ-1, PUR-1).
    #[ts(type = "number")]
    pub number: i64,
    /// Its trip.
    pub trip: TripRef,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The day to sell, `YYYY-MM-DD` (PRD FR-12.39). A new بکری from this day opens with a ready
    /// lot for the goods left. Starts as `entry_date`.
    pub sell_date: String,
    /// Saved or cancelled.
    pub status: PurchaseStatus,
    /// Why it was cancelled.
    pub cancel_reason: Option<String>,
    /// Who it was bought from, as typed.
    pub supplier: Option<String>,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// نگ bought.
    #[ts(type = "number")]
    pub qty: i64,
    /// Weight bought, in grams, for a weight item.
    #[ts(type = "number | null")]
    pub weight_grams: Option<i64>,
    /// The rate.
    #[ts(type = "Paisa")]
    pub rate_paisa: Paisa,
    /// C1.
    #[ts(type = "Paisa")]
    pub value_paisa: Paisa,
    /// Freight.
    #[ts(type = "Paisa")]
    pub freight_paisa: Paisa,
    /// Labour.
    #[ts(type = "Paisa")]
    pub labour_paisa: Paisa,
    /// Other cost.
    #[ts(type = "Paisa")]
    pub other_cost_paisa: Paisa,
    /// Value plus freight, labour and other cost: what was paid at once.
    #[ts(type = "Paisa")]
    pub paid_now_paisa: Paisa,
    /// Its part of the trip costs (C24).
    #[ts(type = "Paisa")]
    pub trip_cost_paisa: Paisa,
    /// Paid now plus its part of the trip costs.
    #[ts(type = "Paisa")]
    pub total_cost_paisa: Paisa,
    /// The average cost for each نگ (display only, A45).
    #[ts(type = "Paisa")]
    pub avg_cost_paisa: Paisa,
    /// A note.
    pub note: Option<String>,
    /// The journal entry of the purchase. Null when every part was from a pocket.
    #[ts(type = "number | null")]
    pub entry_id: Option<i64>,
    /// Each partner's part of what was paid now and where it came from (only in `purchases_get`;
    /// empty in the list).
    pub parts: Vec<EntryPart>,
    /// نگ sold, over the own lots in force and the sales in other shops.
    #[ts(type = "number")]
    pub qty_sold: i64,
    /// نگ written off.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// نگ left: bought less sold less written off. 0 for a cancelled purchase.
    #[ts(type = "number")]
    pub qty_left: i64,
    /// The cost already released: sold plus written off.
    #[ts(type = "Paisa")]
    pub cost_released_paisa: Paisa,
    /// The cost of the نگ left: total cost less the cost released.
    #[ts(type = "Paisa")]
    pub cost_left_paisa: Paisa,
    /// Sales money in our shop: the seller credits of its own lots (PRD FR-12.27).
    #[ts(type = "Paisa")]
    pub sales_paisa: Paisa,
    /// Money from sales in other shops (PRD FR-12.32).
    #[ts(type = "Paisa")]
    pub other_shop_paisa: Paisa,
    /// C21: the cost of the goods sold, in our shop and in other shops.
    #[ts(type = "Paisa")]
    pub cost_sold_paisa: Paisa,
    /// The cost of the goods written off (a loss).
    #[ts(type = "Paisa")]
    pub cost_written_off_paisa: Paisa,
    /// The result: sales money plus other-shop money less the cost sold and the cost written off
    /// (PRD FR-12.27). Negative is a loss, shown in red. E2E-34b: 8,460 − 7,800 − 2,600 = −1,940.
    #[ts(type = "Paisa")]
    pub result_paisa: Paisa,
    /// How the result was worked out.
    pub result_working: Working,
    /// Whole days from the day to sell (`sell_date`) to today, while stock is left; 0 otherwise. Goods
    /// meant for a later day are not waiting before it.
    #[ts(type = "number")]
    pub days_waiting: i64,
    /// True while nothing was sold or written off, it carries no trip cost and it is not cancelled:
    /// `purchases_cancel` will accept it (if its day is open).
    pub can_cancel: bool,
    /// True while it is saved and has goods left: `purchases_set_sell_date` is allowed.
    pub can_change_sell_date: bool,
    /// The own lots that took goods from it (only in `purchases_get`).
    pub lots: Vec<PurchaseLotRef>,
    /// The write-offs that can still be undone, oldest first (only in `purchases_get`).
    pub write_offs: Vec<WriteOffRow>,
}

/// What the lists of purchases add up to, over the whole filter and not only the page (B31).
/// Cancelled purchases are not counted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PurchasesTotals {
    /// Σ of the total cost.
    #[ts(type = "Paisa")]
    pub total_cost_paisa: Paisa,
    /// Σ of the cost left.
    #[ts(type = "Paisa")]
    pub cost_left_paisa: Paisa,
    /// Σ of the نگ left.
    #[ts(type = "number")]
    pub qty_left: i64,
    /// Σ of the result.
    #[ts(type = "Paisa")]
    pub result_paisa: Paisa,
}

/// The answer of `purchases_list`: one page and the totals over the whole filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PurchasesList {
    /// This page: newest day first, then by number.
    pub rows: Vec<PurchaseRow>,
    /// How many purchases match the filter.
    pub total_count: u32,
    /// The totals over all of them.
    pub totals: PurchasesTotals,
}

/// Input of `purchases_write_off` (PRD FR-12.28): spoiled trip goods. It is dated today, which must
/// be an open day (`ERR_DATE_CLOSED` on `date`). It posts nothing: the partners paid for the goods
/// already, and no money comes back.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PurchaseWriteOffInput {
    /// The purchase. `ERR_NOT_FOUND` on `purchase_id`; `ERR_PURCHASE_CANCELLED`.
    #[ts(type = "number")]
    pub purchase_id: i64,
    /// نگ to write off. A simple sum is taken. `ERR_WRITE_OFF_EXCEEDS_BALANCE` (value `available`)
    /// on `qty` if it is more than the stock left.
    pub qty: String,
    /// Why (خراب ...). Required: `ERR_REASON_REQUIRED` on `reason`.
    pub reason: String,
}

/// What `purchases_write_off` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PurchaseWriteOffDone {
    /// The purchase.
    #[ts(type = "number")]
    pub purchase_id: i64,
    /// The stock move, to undo it with `purchases_write_off_undo`.
    #[ts(type = "number")]
    pub move_id: i64,
    /// How many نگ were written off now.
    #[ts(type = "number")]
    pub qty: i64,
    /// C21: the cost written off, a loss that shows in the trip's result. Nothing is posted.
    #[ts(type = "Paisa")]
    pub cost_paisa: Paisa,
    /// How the cost was worked out.
    pub working: Working,
    /// All the نگ written off from the purchase.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// نگ left.
    #[ts(type = "number")]
    pub qty_left: i64,
    /// The cost of the نگ left.
    #[ts(type = "Paisa")]
    pub cost_left_paisa: Paisa,
    /// True if this write-off closed the trip (PRD FR-12.33).
    pub trip_closed: bool,
}

/// Input of `purchases_write_off_undo`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PurchaseWriteOffUndoInput {
    /// The write-off move, from `WriteOffRow::move_id` of `OwnStockRow` or `PurchaseRow`.
    /// `ERR_NOT_FOUND` on `move_id` if it is not a write-off of a purchase, or was undone already;
    /// `ERR_DATE_CLOSED` on `move_id` when its day is closed.
    #[ts(type = "number")]
    pub move_id: i64,
    /// Why. Required: `ERR_REASON_REQUIRED` on `reason`.
    pub reason: String,
}

/// What `purchases_write_off_undo` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PurchaseWriteOffUndone {
    /// The purchase.
    #[ts(type = "number")]
    pub purchase_id: i64,
    /// The opposite stock move.
    #[ts(type = "number")]
    pub reversal_move_id: i64,
    /// All the نگ written off after the undo.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// نگ left.
    #[ts(type = "number")]
    pub qty_left: i64,
    /// The cost of the نگ left.
    #[ts(type = "Paisa")]
    pub cost_left_paisa: Paisa,
    /// True if this undo opened the trip again (PRD FR-12.33).
    pub trip_reopened: bool,
}
