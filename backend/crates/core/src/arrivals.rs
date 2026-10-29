//! Goods that arrived from a garden (مال آیا, PRD FR-12.40, plan 33 v1.1): a stock record of the
//! garden. The fruit of a garden comes to our shop, is written here with its date to sell, and is
//! sold in a بکری as a garden lot linked to the arrival (see `bikri::LotInput.arrival_id`). An
//! arrival posts no money and changes no partner total: it is only goods.
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `gardens_arrival_save` | `ArrivalInput` | `ArrivalSaved` |
//! | `gardens_arrival_cancel` | `ArrivalCancelInput` | `ArrivalCancelled` |
//! | `gardens_arrival_set_sell_date` | `SellDateInput` | `SellDateSet` |
//! | `gardens_arrivals_list` | `ArrivalsListInput` | `ArrivalsList` |
//! | `gardens_arrival_get` | `ArrivalGetInput` | `ArrivalRow` |
//! | `gardens_arrival_write_off` | `ArrivalWriteOffInput` | `ArrivalWriteOffDone` |
//! | `gardens_arrival_write_off_undo` | `ArrivalWriteOffUndoInput` | `ArrivalWriteOffUndone` |
//!
//! There is no preview command: an arrival has nothing to work out. A save with a mistake answers
//! with the error codes, each at its field, and writes nothing.
//!
//! Numbers: an arrival has its own series (مال آیا 1, 2 ...), shown by the screen with a Fluent
//! word in front of the number. It is not a code of the 00x types.
//!
//! Example: the garden شاہ پور باغ (0080) sends 100 پیٹی کیلا on 28-10-2026 to sell on 29-10-2026.
//! A new بکری dated 29-10-2026 or later opens with a ready lot: seller the garden's account 0080,
//! کیلا, پیٹی, 100, linked to the arrival. If only 60 are sold, the lot takes 60 and the arrival
//! keeps 40 for the next new sale. The money is exactly the garden lot of PRD FR-12.36.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::bikri::WriteOffRow;
use crate::error::FieldError;
use crate::ventures::GardenRef;

/// Whether an arrival stands or was cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ArrivalStatus {
    /// In force.
    Saved,
    /// Cancelled with a reason. Its number is kept and never used again.
    Cancelled,
}

impl ArrivalStatus {
    /// The stored form (`garden_arrivals.status`).
    pub const fn as_str(self) -> &'static str {
        match self {
            ArrivalStatus::Saved => "saved",
            ArrivalStatus::Cancelled => "cancelled",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "saved" => Some(ArrivalStatus::Saved),
            "cancelled" => Some(ArrivalStatus::Cancelled),
            _ => None,
        }
    }
}

/// The state filter of `gardens_arrivals_list`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ArrivalStateFilter {
    /// Saved arrivals with goods left (more than 0 نگ).
    InStock,
    /// Saved arrivals with nothing left.
    SoldOut,
    /// Cancelled arrivals.
    Cancelled,
}

/// A garden arrival as typed on the garden screen, type مال آیا. Everything is text; the backend
/// checks every value.
///
/// Field paths in errors: `garden_id`, `date`, `sell_date`, `item_id`, `packing_id`, `qty`,
/// `weight`, `note`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ArrivalInput {
    /// The garden. Required. `ERR_NOT_FOUND`; `ERR_GARDEN_CLOSED` (a closed garden takes no
    /// arrival).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub garden_id: Option<i64>,
    /// The day the goods arrived, `DD-MM-YYYY` or a shortcut. An open day (PRD FR-6.6), not before
    /// the garden's day (`ERR_DATE_BEFORE_GARDEN`, as for any entry of a garden).
    pub date: String,
    /// The day to sell, `DD-MM-YYYY` or a shortcut. Empty means the arrival day. Never before the
    /// arrival day: `ERR_SELL_DATE_BEFORE_ENTRY` on `sell_date`.
    #[serde(default)]
    pub sell_date: Option<String>,
    /// The item. Required.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// The packing. Required.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub packing_id: Option<i64>,
    /// Quantity in نگ. A simple sum such as `20+25` is taken (PRD FR-7.48).
    #[serde(default)]
    pub qty: String,
    /// Weight in kg, up to 3 decimals. Required for an item sold by weight.
    #[serde(default)]
    pub weight: Option<String>,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub note: Option<String>,
}

/// One bikri lot that sold from an arrival.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ArrivalLotRef {
    /// The بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub bikri_number: i64,
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// The lot number inside its بکری, from 1.
    #[ts(type = "number")]
    pub lot_no: i64,
    /// The day of the بکری, `YYYY-MM-DD`.
    pub entry_date: String,
    /// نگ sold by this lot, over the postings in force.
    #[ts(type = "number")]
    pub qty_sold: i64,
}

/// A garden arrival as the lists and the screen read it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ArrivalRow {
    /// The arrival. `gardens_arrival_write_off` and `bikri` lots take this id.
    #[ts(type = "number")]
    pub id: i64,
    /// For the optimistic check of cancel and `gardens_arrival_set_sell_date`.
    #[ts(type = "number")]
    pub version: i64,
    /// Its number in its own series, from 1.
    #[ts(type = "number")]
    pub number: i64,
    /// Its garden, with the garden's account (the seller of the ready lot).
    pub garden: GardenRef,
    /// The day the goods arrived, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The day to sell, `YYYY-MM-DD`. From this day a new بکری opens with a ready lot for the goods
    /// left.
    pub sell_date: String,
    /// Saved or cancelled.
    pub status: ArrivalStatus,
    /// Why it was cancelled.
    pub cancel_reason: Option<String>,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// نگ that arrived.
    #[ts(type = "number")]
    pub qty: i64,
    /// Weight that arrived, in grams, for a weight item.
    #[ts(type = "number | null")]
    pub weight_grams: Option<i64>,
    /// The note.
    pub note: Option<String>,
    /// نگ sold, over the garden lots in force.
    #[ts(type = "number")]
    pub qty_sold: i64,
    /// نگ written off.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// نگ left: arrived less sold less written off.
    #[ts(type = "number")]
    pub qty_left: i64,
    /// Whole days from the day to sell (`sell_date`) to today, while goods are left; 0 otherwise.
    /// Goods meant for a later day are not waiting before it.
    #[ts(type = "number")]
    pub days_waiting: i64,
    /// True while it is saved and nothing of it is sold or written off, on an open day.
    pub can_cancel: bool,
    /// True while it is saved and has goods left: `gardens_arrival_set_sell_date` is allowed.
    pub can_change_sell_date: bool,
    /// The garden lots that sold from it, oldest first.
    pub lots: Vec<ArrivalLotRef>,
    /// The write-offs that can still be undone, oldest first (`WriteOffRow.move_id` goes to
    /// `gardens_arrival_write_off_undo`).
    pub write_offs: Vec<WriteOffRow>,
}

/// What `gardens_arrival_save` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ArrivalSaved {
    /// The arrival as saved.
    pub row: ArrivalRow,
    /// Warnings that did not stop the save, each at its field. Empty today.
    pub warnings: Vec<FieldError>,
}

/// Input of `gardens_arrival_cancel`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ArrivalCancelInput {
    /// The arrival.
    #[ts(type = "number")]
    pub id: i64,
    /// Its version, for the optimistic check.
    #[ts(type = "number")]
    pub version: i64,
    /// Why. Required.
    pub reason: String,
}

/// What `gardens_arrival_cancel` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ArrivalCancelled {
    /// The arrival.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub number: i64,
}

/// Input of `gardens_arrival_get`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ArrivalGetInput {
    /// The arrival.
    #[ts(type = "number")]
    pub id: i64,
}

/// Input of `gardens_arrivals_list`. Dates are `DD-MM-YYYY` or a shortcut; null or empty means no
/// limit. They filter on the arrival day.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct ArrivalsListInput {
    /// Only this garden.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub garden_id: Option<i64>,
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
    /// Only arrivals in this state. Null lists all.
    #[serde(default)]
    pub state: Option<ArrivalStateFilter>,
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives a page of 100 (Code Standards B31).
    #[serde(default)]
    pub limit: u32,
}

/// The totals of `gardens_arrivals_list` over the whole filter (PRD FR-19.11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ArrivalsTotals {
    /// Σ نگ that arrived (cancelled ones left out).
    #[ts(type = "number")]
    pub qty: i64,
    /// Σ نگ sold.
    #[ts(type = "number")]
    pub qty_sold: i64,
    /// Σ نگ written off.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// Σ نگ left.
    #[ts(type = "number")]
    pub qty_left: i64,
}

/// The answer of `gardens_arrivals_list`: oldest arrival day first, then number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ArrivalsList {
    /// One page of rows.
    pub rows: Vec<ArrivalRow>,
    /// How many rows match the filter.
    #[ts(type = "number")]
    pub total_count: i64,
    /// Totals over the whole filter.
    pub totals: ArrivalsTotals,
}

/// Input of `gardens_arrival_write_off`: spoiled goods leave the arrival (nothing is posted).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ArrivalWriteOffInput {
    /// The arrival.
    #[ts(type = "number")]
    pub arrival_id: i64,
    /// How many نگ. Required.
    pub qty: String,
    /// Why. Required.
    pub reason: String,
}

/// What `gardens_arrival_write_off` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ArrivalWriteOffDone {
    /// The arrival.
    #[ts(type = "number")]
    pub arrival_id: i64,
    /// The stock move, to undo it with `gardens_arrival_write_off_undo`.
    #[ts(type = "number")]
    pub move_id: i64,
    /// نگ written off now.
    #[ts(type = "number")]
    pub qty: i64,
    /// نگ written off in all.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// نگ left now.
    #[ts(type = "number")]
    pub qty_left: i64,
}

/// Input of `gardens_arrival_write_off_undo`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ArrivalWriteOffUndoInput {
    /// The stock move of the write-off.
    #[ts(type = "number")]
    pub move_id: i64,
    /// Why. Required.
    pub reason: String,
}

/// What `gardens_arrival_write_off_undo` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ArrivalWriteOffUndone {
    /// The arrival.
    #[ts(type = "number")]
    pub arrival_id: i64,
    /// The reversing stock move.
    #[ts(type = "number")]
    pub reversal_move_id: i64,
    /// نگ written off in all, after the undo.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// نگ left now.
    #[ts(type = "number")]
    pub qty_left: i64,
}

/// Input of `purchases_set_sell_date` and `gardens_arrival_set_sell_date` (PRD FR-12.39, FR-12.40):
/// the date to sell changes while the goods have some left. No money effect.
///
/// Errors: `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_SELL_DATE_BEFORE_ENTRY` on `sell_date` (value `entry_date`); the date codes on
/// `sell_date`; `ERR_PURCHASE_CANCELLED` / `ERR_ARRIVAL_CANCELLED` on `id`;
/// `ERR_NOTHING_TO_SELL` on `id` (nothing is left, so the date has no use).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SellDateInput {
    /// The purchase or the arrival.
    #[ts(type = "number")]
    pub id: i64,
    /// Its version.
    #[ts(type = "number")]
    pub version: i64,
    /// The new day to sell, `DD-MM-YYYY` or a shortcut.
    pub sell_date: String,
}

/// What the two set-sell-date commands give back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SellDateSet {
    /// The purchase or the arrival.
    #[ts(type = "number")]
    pub id: i64,
    /// Its new version.
    #[ts(type = "number")]
    pub version: i64,
    /// The day to sell now, `YYYY-MM-DD`.
    pub sell_date: String,
}
