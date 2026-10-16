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
    /// بینک.
    BankAccount,
    /// بیوپاری accounts (control 03011). Plan 07 (account search), plan 08 (short codes).
    Seller,
    /// گاہک accounts (control 0304).
    Buyer,
    /// Any active account: sellers, buyers and all others (جنرل, ledgers, the linked account).
    AnyAccount,
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
            MasterKind::BankAccount => SearchScope::BankAccount,
        }
    }

    /// The master kind this scope searches, if it searches a master.
    pub const fn master_kind(self) -> Option<MasterKind> {
        Some(match self {
            SearchScope::Seller | SearchScope::Buyer | SearchScope::AnyAccount => return None,
            SearchScope::City => MasterKind::City,
            SearchScope::Item => MasterKind::Item,
            SearchScope::Packing => MasterKind::Packing,
            SearchScope::Adda => MasterKind::Adda,
            SearchScope::Marka => MasterKind::Marka,
            SearchScope::Store => MasterKind::Store,
            SearchScope::Narration => MasterKind::Narration,
            SearchScope::ExpenseHead => MasterKind::ExpenseHead,
            SearchScope::Staff => MasterKind::Staff,
            SearchScope::BankAccount => MasterKind::BankAccount,
        })
    }

    /// The wire form, also the part after `recent.` in the user preference key.
    pub const fn as_str(self) -> &'static str {
        match self {
            SearchScope::Seller => "seller",
            SearchScope::Buyer => "buyer",
            SearchScope::AnyAccount => "any_account",
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
    /// What was typed. Raw: never converted to Urdu (PRD FR-7.22, F41).
    pub text: String,
    /// The most hits to return. At most 50.
    pub limit: u32,
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
