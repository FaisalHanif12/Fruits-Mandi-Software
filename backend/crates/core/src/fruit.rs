//! Fruit taken (پھل, PRD 16.7, FR-16.10 to FR-16.12, plan 17d): what the پھل screen sends and reads
//! back. A partner takes fruit home, or fruit is served to the shop's guests. It is kept by item and
//! quantity, not only by amount. The value comes off the partner's کل رقم (his 0604 account is
//! debited); for the shop's guests it is split equally between the partners active on the day (C27),
//! never by the firm's shares, and it is never a دکان خرچہ. No cash moves.
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `fruit_preview` | `FruitInput` | `FruitPreview` |
//! | `fruit_save` | `FruitInput` | `FruitSaved` |
//! | `fruit_edit` | `FruitEditInput` | `FruitEdited` |
//! | `fruit_cancel` | `FruitCancelInput` | `FruitCancelled` |
//! | `fruit_get` | `FruitGetInput` | `FruitRow` |
//! | `fruit_list` | `FruitListInput` | `FruitList` |
//! | `fruit_lots` | `FruitLotsInput` | `FruitLots` |
//!
//! The list returns one page with the row count and the totals over the whole filter (Code
//! Standards B31, PRD FR-19.11). An entry's number is its own series, shown as پھل-1 in Urdu and
//! FRT-1 in English.
//!
//! Where the fruit comes from (FR-16.11):
//!
//! - **`shop`**: a lot of our shop with a balance. The backend takes the item and packing from the
//!   lot (what is sent in `item_id` and `packing_id` is ignored, and the preview answers them). The
//!   lot's `qty_taken` goes up, a stock move of kind `taken` is written, and the lot's seller (an
//!   account, a پختہ کھاتہ person, or a garden's account) is credited the value with no commission.
//!   That credit is a payable posting (FR-7.59) and never adjusts an advance. More than the lot's
//!   balance is `ERR_QTY_EXCEEDS_LOT` on `qty`. Trip goods are not offered, and a lot of a closed
//!   garden is refused with `ERR_GARDEN_CLOSED` on `lot_id`.
//! - **`other_shop`**: another commission agent's shop. `agent_account_id` is his seller account; it
//!   is credited the value. The item and packing are typed.
//!
//! Money words (the money agent, Section 5.5):
//!
//! - **Value** (C1) is `qty × rate`, or `round(weight kg × rate per kg)` for an item sold by weight.
//! - **One partner**: his 0604 account is debited the value. Nobody else is affected.
//! - **Guests** (C27): `split_by_qty(value, [1 for each partner active on the date])`, partners in
//!   the order they were created, so the leftover rupee goes to the first one created.
//!   1,000 over 3 partners gives 334 / 333 / 333.
//! - **Source credit**: the lot's seller or the agent is credited the value.
//!
//! Worked numbers (PRD E2E-42, after E2E-16): الف takes 2 پیٹی کیلا at 580 from رحیم's lot: value
//! 1,160; الف's total −1,160; رحیم's payable 1,590 + 1,160 = 2,750; the lot's balance 5 − 2 = 3.
//! Guest fruit: 3 پیٹی سیب at 1,000 from حاجی نذیر's shop: value 3,000, split 1,000 / 1,000 / 1,000;
//! حاجی نذیر's payable 3,000. Cash does not move.
//!
//! Field paths in errors on `FruitInput`: `partner_id`, `date`, `source`, `lot_id`,
//! `agent_account_id`, `item_id`, `packing_id`, `qty`, `weight`, `rate`, `note`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::bikri::{RateHint, SavedPukhta};
use crate::error::FieldError;
use crate::money::Paisa;
use crate::rules::Working;

/// Where the fruit comes from (PRD FR-16.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum FruitSource {
    /// اپنی دکان: a lot of our shop with a balance.
    Shop,
    /// دوسری دکان: another commission agent's shop.
    OtherShop,
}

impl FruitSource {
    /// The stored form (`fruit_takes.source`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FruitSource::Shop => "shop",
            FruitSource::OtherShop => "other_shop",
        }
    }

    /// The form back from `as_str`.
    #[must_use]
    pub fn parse(text: &str) -> Option<FruitSource> {
        match text {
            "shop" => Some(FruitSource::Shop),
            "other_shop" => Some(FruitSource::OtherShop),
            _ => None,
        }
    }
}

/// The state of an entry. A cancelled entry stays in the list and counts for nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum FruitStatus {
    /// In force.
    Saved,
    /// Cancelled by reversal.
    Cancelled,
}

impl FruitStatus {
    /// The stored form (`fruit_takes.status`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FruitStatus::Saved => "saved",
            FruitStatus::Cancelled => "cancelled",
        }
    }

    /// The form back from `as_str`.
    #[must_use]
    pub fn parse(text: &str) -> Option<FruitStatus> {
        match text {
            "saved" => Some(FruitStatus::Saved),
            "cancelled" => Some(FruitStatus::Cancelled),
            _ => None,
        }
    }
}

// ---- One entry ----

/// One fruit entry as the screen types it (PRD 16.7). Everything is text, parsed by the backend.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct FruitInput {
    /// True for fruit served to the shop's guests (مہمان); then `partner_id` must be empty.
    #[serde(default)]
    pub for_guests: bool,
    /// The partner who takes the fruit home. Required when `for_guests` is false:
    /// `ERR_PARTNER_REQUIRED` on `partner_id`; `ERR_NOT_FOUND` or `ERR_PARTNER_NOT_LISTED` if he is
    /// not an active partner. With `for_guests` it must be null: `ERR_FORMAT_INVALID`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
    /// The day, `DD-MM-YYYY` or a shortcut. An open day (PRD FR-6.6).
    pub date: String,
    /// Where the fruit comes from.
    pub source: FruitSource,
    /// The lot, when `source` is `shop`: a lot from `fruit_lots`. `ERR_REQUIRED` on `lot_id` if
    /// missing, `ERR_LOT_NOT_FOUND` if it is not offered (cancelled بکری, trip goods, no balance).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub lot_id: Option<i64>,
    /// The other agent's seller account, when `source` is `other_shop`. `ERR_REQUIRED` on
    /// `agent_account_id` if missing; `ERR_FORMAT_INVALID` if it is not an ordinary seller account.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub agent_account_id: Option<i64>,
    /// The item, when `source` is `other_shop`. With `shop` it is ignored (taken from the lot).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// The packing, as `item_id`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub packing_id: Option<i64>,
    /// نگ taken: a whole number or a simple sum (PRD FR-7.48), more than 0.
    pub qty: String,
    /// Weight in kg, up to 3 decimals, for an item sold by weight. Null or empty otherwise.
    #[serde(default)]
    pub weight: Option<String>,
    /// The rate, whole rupees, more than 0 (`ERR_MUST_BE_MORE_THAN_ZERO` on `rate`). Typed by the
    /// user; the last rate only shows as a hint (PRD FR-7.53).
    pub rate: String,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub note: Option<String>,
}

/// One partner's part of the value, as the preview shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitPart {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His part of the value, whole rupees. His 0604 account is debited this amount, so his کل رقم
    /// goes down by it.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// His کل رقم now (signed).
    #[ts(type = "Paisa")]
    pub total_before_paisa: Paisa,
    /// His کل رقم after this entry (signed).
    #[ts(type = "Paisa")]
    pub total_after_paisa: Paisa,
}

/// The account that is credited the value (PRD FR-16.11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitCredit {
    /// The lot's seller, or the other agent. A garden's account when the lot is a garden's.
    pub account: RecordRef,
    /// The one-time seller of a پختہ کھاتہ lot. Null for any other account.
    pub pukhta: Option<SavedPukhta>,
}

/// What the lot looks like before and after this entry (C16). Null for `other_shop`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitLotBalance {
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// نگ left before this entry (in an edit, with the old quantity given back).
    #[ts(type = "number")]
    pub balance_before: i64,
    /// نگ left after this entry.
    #[ts(type = "number")]
    pub balance_after: i64,
}

/// The numbers of an entry that has no errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitNumbers {
    /// The item (from the lot for `shop`).
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// نگ taken.
    #[ts(type = "number")]
    pub qty: i64,
    /// Weight in grams, if the item is sold by weight.
    #[ts(type = "number | null")]
    pub weight_grams: Option<i64>,
    /// The rate.
    #[ts(type = "Paisa")]
    pub rate_paisa: Paisa,
    /// The value (C1).
    #[ts(type = "Paisa")]
    pub value_paisa: Paisa,
    /// How the value was worked out.
    pub value_working: Working,
    /// One part for the partner, or one for each partner active on the day for the guests, in the
    /// order the partners were created. The parts add up to the value.
    pub parts: Vec<FruitPart>,
    /// How the guests' parts were worked out (C27). Null for one partner.
    pub parts_working: Option<Working>,
    /// Who is credited.
    pub credit: FruitCredit,
    /// The lot's balance before and after. Null for `other_shop`.
    pub lot: Option<FruitLotBalance>,
}

/// An entry checked and worked out, nothing saved (Code Standards B5: the save uses the same
/// function). The mistakes come back here, each at its field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitPreview {
    /// The numbers. Null while there are errors.
    pub numbers: Option<FruitNumbers>,
    /// Mistakes, all together.
    pub errors: Vec<FieldError>,
    /// Warnings that do not stop the save.
    pub warnings: Vec<FieldError>,
    /// The last rate of this item and packing, as a grey hint only (PRD FR-7.53). It is never filled
    /// into `rate`. Null if there is none.
    pub rate_hint: Option<RateHint>,
}

/// What `fruit_save` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitSaved {
    /// The entry.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number in its own series.
    #[ts(type = "number")]
    pub number: i64,
    /// Its version, for an edit or a cancel.
    #[ts(type = "number")]
    pub version: i64,
    /// Its journal entry.
    #[ts(type = "number")]
    pub journal_entry_id: i64,
    /// The preview it was saved from.
    pub preview: FruitPreview,
}

// ---- Edit and cancel by reversal (PRD FR-16.12, FR-15.3) ----

/// Input of `fruit_edit`: the old posting is reversed and the new one is posted, on the same
/// entry, while its day is open.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct FruitEditInput {
    /// The entry. `ERR_NOT_FOUND` on `id`; `ERR_FRUIT_CANCELLED` if it was cancelled.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read. `ERR_VERSION_CHANGED` on `version`.
    #[ts(type = "number")]
    pub version: i64,
    /// The new values, checked like `fruit_save`.
    pub entry: FruitInput,
    /// Why. Required: `ERR_REASON_REQUIRED` on `reason`.
    pub reason: String,
}

/// What `fruit_edit` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitEdited {
    /// The entry.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number (the same).
    #[ts(type = "number")]
    pub number: i64,
    /// Its new version.
    #[ts(type = "number")]
    pub version: i64,
    /// The new posting.
    #[ts(type = "number")]
    pub journal_entry_id: i64,
    /// The reversal of the old posting.
    #[ts(type = "number")]
    pub reversal_entry_id: i64,
    /// The preview of the new values.
    pub preview: FruitPreview,
}

/// Input of `fruit_cancel`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct FruitCancelInput {
    /// The entry. `ERR_NOT_FOUND` on `id`; `ERR_FRUIT_CANCELLED` if it was cancelled already;
    /// `ERR_DATE_CLOSED` on `id` if its day is closed.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read. `ERR_VERSION_CHANGED` on `version`.
    #[ts(type = "number")]
    pub version: i64,
    /// Why. Required: `ERR_REASON_REQUIRED` on `reason`.
    pub reason: String,
}

/// What `fruit_cancel` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitCancelled {
    /// The entry.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub number: i64,
    /// The reversal of its posting.
    #[ts(type = "number")]
    pub reversal_entry_id: i64,
}

// ---- Reading ----

/// The lot an entry took its fruit from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitLotRef {
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// Its number inside the بکری.
    #[ts(type = "number")]
    pub lot_no: i64,
    /// The بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub bikri_number: i64,
}

/// One partner's part of a saved entry, read from its journal lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitPartRow {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His part of the value.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
}

/// One saved entry, in the list and in `fruit_get`. It holds everything the edit form needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitRow {
    /// The entry.
    #[ts(type = "number")]
    pub id: i64,
    /// Its version, for an edit or a cancel.
    #[ts(type = "number")]
    pub version: i64,
    /// Its number in its own series.
    #[ts(type = "number")]
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The partner who took it home. Null for the shop's guests.
    pub partner: Option<RecordRef>,
    /// Where it came from.
    pub source: FruitSource,
    /// The lot, for `shop`.
    pub lot: Option<FruitLotRef>,
    /// Who is credited: the lot's seller or the other agent.
    pub credit_account: RecordRef,
    /// The one-time seller of a پختہ کھاتہ lot, if any.
    pub pukhta: Option<SavedPukhta>,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// نگ taken.
    #[ts(type = "number")]
    pub qty: i64,
    /// Weight in grams, if any.
    #[ts(type = "number | null")]
    pub weight_grams: Option<i64>,
    /// The rate.
    #[ts(type = "Paisa")]
    pub rate_paisa: Paisa,
    /// The value.
    #[ts(type = "Paisa")]
    pub value_paisa: Paisa,
    /// The note, as typed.
    pub note: Option<String>,
    /// Saved or cancelled.
    pub status: FruitStatus,
    /// Why it was cancelled.
    pub cancel_reason: Option<String>,
    /// The parts, read from the journal lines in force: one for a partner, one for each partner
    /// for the guests, in the order the partners were created.
    pub parts: Vec<FruitPartRow>,
    /// The posting in force. Null once cancelled.
    #[ts(type = "number | null")]
    pub journal_entry_id: Option<i64>,
    /// True while it is saved and its day is open, so it can be edited or cancelled.
    pub can_change: bool,
}

/// Input of `fruit_get`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct FruitGetInput {
    /// The entry. `ERR_NOT_FOUND` on `id`.
    #[ts(type = "number")]
    pub id: i64,
}

/// Input of `fruit_list` (PRD FR-16.12, FR-19.10, FR-19.11). Dates are `DD-MM-YYYY` or a shortcut;
/// null or empty means no limit. An empty input lists everything, newest first.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct FruitListInput {
    /// Only this partner's entries. Null for all.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
    /// True lists only the shop's guests' entries (then `partner_id` must be null).
    #[serde(default)]
    pub guests_only: bool,
    /// Only this item. Null for all.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// Only this source. Null for both.
    #[serde(default)]
    pub source: Option<FruitSource>,
    /// First day.
    #[serde(default)]
    pub from: Option<String>,
    /// Last day.
    #[serde(default)]
    pub to: Option<String>,
    /// Only saved or only cancelled entries. Null for both.
    #[serde(default)]
    pub status: Option<FruitStatus>,
    /// Rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// Rows to read. 0 means a page of 100.
    #[serde(default)]
    pub limit: u32,
}

/// Totals over the whole filter, saved entries only (B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitTotals {
    /// Σ of نگ.
    #[ts(type = "number")]
    pub qty: i64,
    /// Σ of the value.
    #[ts(type = "Paisa")]
    pub value_paisa: Paisa,
}

/// One page of entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitList {
    /// The rows of this page, newest first.
    pub rows: Vec<FruitRow>,
    /// How many rows the filter has in all.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over the whole filter.
    pub totals: FruitTotals,
}

// ---- The lot picker ----

/// Input of `fruit_lots`: the lots fruit can be taken from.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct FruitLotsInput {
    /// Text to find in the seller's name, the item or the بکری number. Null for all.
    #[serde(default)]
    pub search: Option<String>,
    /// Rows to read. 0 means 100.
    #[serde(default)]
    pub limit: u32,
}

/// One lot with a balance, as the picker shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitLotRow {
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// Its number inside the بکری.
    #[ts(type = "number")]
    pub lot_no: i64,
    /// The بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub bikri_number: i64,
    /// The day received, `YYYY-MM-DD`.
    pub received_date: String,
    /// The seller account (a garden's account for a garden's lot).
    pub seller: RecordRef,
    /// The one-time seller of a پختہ کھاتہ lot, if any.
    pub pukhta: Option<SavedPukhta>,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// True if the item is sold by weight, so a weight is needed.
    pub sold_by_weight: bool,
    /// نگ left: received less sold, written off and taken (C16). More than 0.
    #[ts(type = "number")]
    pub balance: i64,
}

/// The lots fruit can be taken from: saved بکری, balance more than 0, not trip goods, not a lot of
/// a closed garden (PRD FR-16.11). Oldest received first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct FruitLots {
    /// The rows.
    pub rows: Vec<FruitLotRow>,
    /// How many lots match in all.
    #[ts(type = "number")]
    pub total_count: i64,
}
