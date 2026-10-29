//! What the pickers and the search box send and get back (plan 06, Tech Stack 4.7).
//!
//! The matching itself is in `services::search`. This module holds only the shapes, so the app
//! and the services agree on them.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::masters::MasterKind;

/// Which list a picker searches. Plan 06 has the ten masters; plan 08 adds sellers, buyers and
/// accounts. Serialised like [`MasterKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SearchScope {
    /// شہر.
    City,
    /// قسم مال.
    Item,
    /// پیکنگ.
    Packing,
    /// اڈا.
    Adda,
    /// مارکہ.
    Marka,
    /// سٹور.
    Store,
    /// تفصیل.
    Narration,
    /// خرچہ مد.
    ExpenseHead,
    /// عملہ.
    Staff,
    /// بیوپاری accounts (control 03011). Plan 07 (account search), plan 08 (short codes).
    Seller,
    /// گاہک accounts (control 0304).
    Buyer,
    /// Any active account: sellers, buyers and all others (جنرل, ledgers, the linked account).
    AnyAccount,
    /// The open market trips (plan 34): code `0070`, `0071` ..., their name and market. A short code is
    /// the running number (`0` finds `0070`).
    Trip,
    /// The open gardens (plan 34): code `0080`, `0081` ..., their name. A short code is the running
    /// number (`0` finds `0080`).
    Garden,
    /// Every market trip, closed ones too, for the FILTER field of a report or an analytics view (a
    /// past trip's R33). Same codes and short codes as `trip`. Entry fields use `trip`.
    TripAll,
    /// Every garden, closed ones too, for the FILTER field of a report or an analytics view (a
    /// closed garden's R36). Same codes and short codes as `garden`. Entry fields use `garden`.
    GardenAll,
}

impl SearchScope {
    /// The scope of a master kind.
    pub const fn of(kind: MasterKind) -> SearchScope {
        match kind {
            MasterKind::City => SearchScope::City,
            MasterKind::Item => SearchScope::Item,
            MasterKind::Packing => SearchScope::Packing,
            MasterKind::Adda => SearchScope::Adda,
            MasterKind::Marka => SearchScope::Marka,
            MasterKind::Store => SearchScope::Store,
            MasterKind::Narration => SearchScope::Narration,
            MasterKind::ExpenseHead => SearchScope::ExpenseHead,
            MasterKind::Staff => SearchScope::Staff,
        }
    }

    /// The master kind this scope searches, if it searches a master.
    pub const fn master_kind(self) -> Option<MasterKind> {
        Some(match self {
            SearchScope::Seller
            | SearchScope::Buyer
            | SearchScope::AnyAccount
            | SearchScope::Trip
            | SearchScope::Garden
            | SearchScope::TripAll
            | SearchScope::GardenAll => return None,
            SearchScope::City => MasterKind::City,
            SearchScope::Item => MasterKind::Item,
            SearchScope::Packing => MasterKind::Packing,
            SearchScope::Adda => MasterKind::Adda,
            SearchScope::Marka => MasterKind::Marka,
            SearchScope::Store => MasterKind::Store,
            SearchScope::Narration => MasterKind::Narration,
            SearchScope::ExpenseHead => MasterKind::ExpenseHead,
            SearchScope::Staff => MasterKind::Staff,
        })
    }

    /// The wire form, also the part after `recent.` in the user preference key.
    pub const fn as_str(self) -> &'static str {
        match self {
            SearchScope::Seller => "seller",
            SearchScope::Buyer => "buyer",
            SearchScope::AnyAccount => "any_account",
            SearchScope::Trip => "trip",
            SearchScope::Garden => "garden",
            SearchScope::TripAll => "trip_all",
            SearchScope::GardenAll => "garden_all",
            _ => match self.master_kind() {
                Some(kind) => kind.as_str(),
                None => "unknown",
            },
        }
    }
}

/// One record a picker offers (PRD FR-7.40): its code, both names and a second line that tells
/// two records of the same name apart. Only active records are offered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SearchHit {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// What it is.
    pub kind: SearchScope,
    /// Its code. Empty for lists without a code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The second line in Urdu (for example the city of a truck stand). Empty if there is none.
    pub extra_ur: String,
    /// The second line in English.
    pub extra_en: String,
    /// True for an active record. Always true in search results.
    pub is_active: bool,
}

/// Search one list for the typed text.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SearchMastersInput {
    /// Which list.
    pub scope: SearchScope,
    /// What was typed. Raw: never converted to Urdu (PRD FR-7.22, F41). **Empty text gives the
    /// whole list** (plan 32 R7): the records last chosen by this user first (latest first), then
    /// every other active record by code. A click on a picker opens this list.
    pub text: String,
    /// The most hits to return in this page. At most 50.
    pub limit: u32,
    /// How many hits to skip, for the next page of a long list (the picker loads more when the user
    /// scrolls). 0 for the first page. A page with fewer than `limit` hits is the last one. Missing
    /// means 0.
    #[serde(default)]
    pub offset: u32,
}

/// The last records used in a list, then the rest.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SearchRecentInput {
    /// Which list.
    pub scope: SearchScope,
}

/// Tell the app that a record was chosen, so it comes first next time (PRD FR-7.41).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SearchTouchInput {
    /// Which list.
    pub scope: SearchScope,
    /// The chosen row.
    #[ts(type = "number")]
    pub id: i64,
}
