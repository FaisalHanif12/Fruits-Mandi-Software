//! Shares, money sources and the parts of a payment, shared by the market trips (PRD 12.10) and the
//! gardens (PRD 12.11, built in D4R plan 17c). Only shapes and small helpers: the split itself is
//! `core::money::split_by_bps`, and the database work is in `services::ventures`.
//!
//! A venture (a trip or a garden) is owned by some partners, each with a share in bps (the shares of
//! the partners in it total exactly 10,000, PRD FR-12.29). Every payment of the venture is split by
//! those shares (PRD C23). For each partner the entry also says where his part comes from:
//!
//! - `shop` (دکان سے): his 0604 account is debited and روکڑ credited, so it comes off his کل رقم at
//!   once (PRD FR-12.30);
//! - `pocket` (اپنی جیب سے): nothing posts, the venture only records it. A payment whose parts are
//!   all from pockets has no journal entry at all.
//!
//! Example (PRD FR-12.30): a purchase of 96,000, partner الف 50% from the shop and ب 50% from his
//! pocket. The parts are 48,000 and 48,000. الف's 0604 is debited 48,000 and روکڑ credited 48,000;
//! ب's part is recorded on the trip only.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::money::Paisa;
use crate::rules::Working;

/// Where a partner's part of a payment comes from (PRD FR-12.29, A54).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MoneySource {
    /// دکان سے: the shop's cash. Posts to his 0604 account and روکڑ.
    Shop,
    /// اپنی جیب سے: his own pocket. Recorded only; nothing posts and his کل رقم does not change.
    Pocket,
}

impl MoneySource {
    /// The stored form (`venture_partners.source`, `venture_entry_parts.source`).
    pub const fn as_str(self) -> &'static str {
        match self {
            MoneySource::Shop => "shop",
            MoneySource::Pocket => "pocket",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "shop" => Some(MoneySource::Shop),
            "pocket" => Some(MoneySource::Pocket),
            _ => None,
        }
    }
}

/// A trip or a garden (PRD 12.10, 12.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum VentureKind {
    /// A garden rented by some partners for a season (D4R plan 17c).
    Garden,
    /// A market trip (PRD 12.10).
    Trip,
}

impl VentureKind {
    /// The stored form (`ventures.kind`).
    pub const fn as_str(self) -> &'static str {
        match self {
            VentureKind::Garden => "garden",
            VentureKind::Trip => "trip",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "garden" => Some(VentureKind::Garden),
            "trip" => Some(VentureKind::Trip),
            _ => None,
        }
    }
}

/// Whether a venture still takes entries. A trip closes by itself, a garden is closed by the Owner
/// (PRD FR-12.33, FR-12.38).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum VentureStatus {
    /// Takes entries.
    Open,
    /// Takes no new purchase or cost (`ERR_TRIP_CLOSED`). A trip opens again when a sale or a
    /// write-off of it is reversed.
    Closed,
}

impl VentureStatus {
    /// The stored form (`ventures.status`).
    pub const fn as_str(self) -> &'static str {
        match self {
            VentureStatus::Open => "open",
            VentureStatus::Closed => "closed",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "open" => Some(VentureStatus::Open),
            "closed" => Some(VentureStatus::Closed),
            _ => None,
        }
    }
}

/// One partner of a new venture, as typed (PRD FR-12.29). A partner who is not in the venture is
/// left out, or sent with a share of 0, and the backend skips him.
///
/// Field paths in errors: `partners[i].partner_id`, `partners[i].share`, `partners[i].source`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct VenturePartnerInput {
    /// The partner. `ERR_NOT_FOUND` if there is no such partner, `ERR_PARTNER_NOT_LISTED` if he is
    /// sent twice or is not an active partner.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// His share as percent text with up to 2 decimals, 0 to 100, for example `40` or `33.33`.
    /// The shares of the partners in the venture must total exactly 100% (`ERR_SHARES_NOT_100` on
    /// `partners`).
    pub share: String,
    /// Where his money comes from by default. The first entry fills it, and each entry can change it
    /// (PRD A54).
    pub source: MoneySource,
}

/// One partner of a venture as read (PRD FR-12.29).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct VenturePartnerRow {
    /// The partner. `code` is empty: a partner has no code.
    pub partner: RecordRef,
    /// His share in bps (4000 is 40%). 0 only in `trips_defaults`, for a partner who is not in the
    /// last trip.
    pub share_bps: i32,
    /// His default source.
    pub source: MoneySource,
}

/// The source chosen for one partner on one entry (PRD FR-12.29, A54). The entry sends one of these
/// for every partner in the venture; the preview and the save read the shares from the venture.
///
/// Field paths in errors: `parts[i].partner_id`, `parts[i].source`, and `parts` when a partner of
/// the venture is missing.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct EntryPartInput {
    /// The partner. `ERR_PARTNER_NOT_LISTED` on `parts` if he is not in the venture or is sent twice.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// Where his part comes from on this entry.
    pub source: MoneySource,
}

/// One partner's part of a payment (PRD C23), as the preview and the saved view show it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EntryPart {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His share in bps.
    pub share_bps: i32,
    /// Where his part comes from.
    pub source: MoneySource,
    /// His part: `split_by_bps` of the payment, whole rupees. The parts add up to the payment.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// How the part was worked out: the payment, his share and the leftover rule (CalcInfo).
    pub working: Working,
}

/// The parts of one payment and the money that moves (PRD FR-12.30).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct EntryParts {
    /// One part for each partner in the venture, in the order the partners were created.
    pub parts: Vec<EntryPart>,
    /// Σ of the parts from the shop's cash: what the entry takes out of روکڑ.
    #[ts(type = "Paisa")]
    pub shop_total_paisa: Paisa,
    /// Σ of the parts from pockets: recorded, nothing posts.
    #[ts(type = "Paisa")]
    pub pocket_total_paisa: Paisa,
}

/// One partner's part of money that comes to the venture's partners: the seller credit of a trip
/// lot or the money of a sale in another shop (PRD C25), or a garden's closing (C26).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ShareAmount {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His share in bps.
    pub share_bps: i32,
    /// His part: `split_by_bps` of the money. The parts add up to the money.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// How the part was worked out (CalcInfo).
    pub working: Working,
}

/// A trip as other screens point at it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TripRef {
    /// The trip.
    #[ts(type = "number")]
    pub id: i64,
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
}

/// A garden as other screens point at it (plan 33 v1.1): the stock list, the ready goods of a new
/// sale and the arrival rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenRef {
    /// The garden.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number in its own series, from 1.
    #[ts(type = "number")]
    pub number: i64,
    /// Its code (plan 34): `0080` for the first garden. Its account has the same code.
    pub code: String,
    /// Urdu name, as typed.
    pub name_ur: String,
    /// English name, as typed.
    pub name_en: String,
    /// The garden's account (control 0306). It is the seller of a garden lot, so a ready lot can
    /// fill its seller from it.
    pub account: crate::accounts::RecordRef,
}
