//! The goods a new بکری opens with (PRD FR-12.39, FR-12.40, plan 33 v1.1): every stock record with
//! goods left whose date to sell is on or before the بکری's date. A record is a purchase of an open
//! trip, or an arrival of an open garden. The screen turns each row into a ready lot:
//!
//! - a trip row: seller اپنا مال (030119998), `purchase_id` = `id`;
//! - a garden row: seller = `garden.account`, `arrival_id` = `id`.
//!
//! In both the item and the packing are the row's, and the lot's `qty_received` is `qty_left`.
//! The goods not sold stay in the record and come again into the next new بکری.
//!
//! Example: on 29-10-2026 a trip purchase خ-1 (80 پیٹی مالٹا, sell date 28-10-2026) and a garden
//! arrival (100 پیٹی کیلا, sell date 29-10-2026) are listed, oldest sell date first. An arrival with
//! sell date 30-10-2026 is not.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::ventures::{GardenRef, TripRef, VentureKind};

/// Input of `bikri_ready_goods`.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct ReadyGoodsInput {
    /// The date of the new بکری, `DD-MM-YYYY` or a shortcut. Empty means today. Records whose date
    /// to sell is on or before it are listed.
    #[serde(default)]
    pub date: String,
}

/// One stock record that can be a ready lot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ReadyGoodsRow {
    /// `trip` for a purchase of a trip, `garden` for a garden arrival.
    pub kind: VentureKind,
    /// The purchase id (kind trip) or the arrival id (kind garden).
    #[ts(type = "number")]
    pub id: i64,
    /// The purchase number or the arrival number, in its own series.
    #[ts(type = "number")]
    pub number: i64,
    /// The day to sell, `YYYY-MM-DD`.
    pub sell_date: String,
    /// The day it was bought or arrived, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// نگ bought or arrived.
    #[ts(type = "number")]
    pub qty: i64,
    /// نگ left to sell in OUR shop (more than 0): the qty of the ready lot. The part for another
    /// shop never comes into a sale (PRD FR-12.41); rows with nothing left for our shop are not
    /// listed.
    #[ts(type = "number")]
    pub qty_left: i64,
    /// The whole weight in grams of what was bought or arrived, for a weight item. Use it as the
    /// lot's weight only when `qty_left` equals `qty`; otherwise the user types the weight.
    #[ts(type = "number | null")]
    pub weight_grams: Option<i64>,
    /// The trip, for a trip row.
    pub trip: Option<TripRef>,
    /// The garden with its account, for a garden row.
    pub garden: Option<GardenRef>,
}

/// The answer of `bikri_ready_goods`: oldest date to sell first, then number (trip purchases and
/// garden arrivals together, in one list). At most 100 rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ReadyGoods {
    /// The rows.
    pub rows: Vec<ReadyGoodsRow>,
}
