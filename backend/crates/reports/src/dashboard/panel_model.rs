//! The types of the bell panel (PRD FR-7.44, v1.61, plan 100 section J): three groups, from the
//! books, so an item stays until it is settled.
//!
//! * بیوپاریوں کو ادائیگی: the sellers with the largest net payable (plan 100 section A).
//! * گاہکوں سے وصولی: the buyers who owe the most.
//! * عام یاد دہانیاں: the payables مزدوری, کرایہ, منشیانہ and کرایہ سٹور still to pay, then the
//!   other items of the attention list (drafts, open days, goods waiting, over the credit limit,
//!   old backup, overdue loans and takes, missing names, health check, records differ).
//!
//! "بعد میں" (`assist_snooze` with the item's `item_key`) hides an item until the next بکری save.

use mandi_core::money::Paisa;
use serde::Serialize;
use ts_rs::TS;

use super::model::{AttentionFirst, AttentionKind};
use crate::reports::PartyRef;

/// How many sellers and buyers the panel lists.
pub const PANEL_TOP: usize = 5;

/// One seller or buyer of the panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AttentionPartyRow {
    /// The key `assist_snooze` takes, for example `seller:12`.
    pub item_key: String,
    /// The party.
    pub party: PartyRef,
    /// The seller's net payable, or what the buyer owes.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// Seller: days since the oldest posting still to pay after his advances. Buyer: days of his
    /// oldest open bill. Null when there is none.
    #[ts(type = "number | null")]
    pub days: Option<i64>,
}

/// The sellers or the buyers of the panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AttentionPartyGroup {
    /// The top [`PANEL_TOP`] by amount, without the ones hidden by "بعد میں".
    pub rows: Vec<AttentionPartyRow>,
    /// How many parties have an amount in all (hidden ones too).
    #[ts(type = "number")]
    pub count: i64,
    /// The total over all of them. Sellers: the tile بیوپاریوں کو ادائیگی باقی. Buyers: the tile
    /// گاہکوں کے ذمے.
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
}

/// One payable of head 02 still to pay (مزدوری, کرایہ, کرایہ سٹور, منشیانہ). Its button opens
/// ادائیگی with this account chosen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AttentionPayableRow {
    /// The key `assist_snooze` takes, for example `payable:7`.
    pub item_key: String,
    /// The account.
    #[ts(type = "number")]
    pub account_id: i64,
    /// Its code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Still to pay (its جمع balance).
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
}

/// One other reminder, as a line of the attention list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AttentionPanelItem {
    /// The key `assist_snooze` takes, for example `kind:drafts`.
    pub item_key: String,
    /// What it is.
    pub kind: AttentionKind,
    /// How many there are.
    #[ts(type = "number")]
    pub count: i64,
    /// The money involved, where there is some.
    #[ts(type = "Paisa | null")]
    pub amount_paisa: Option<Paisa>,
    /// The نگ involved, where there are some.
    #[ts(type = "number | null")]
    pub qty: Option<i64>,
    /// The most days past (or old), where that means something.
    #[ts(type = "number | null")]
    pub days: Option<i64>,
    /// The most urgent record.
    pub first: Option<AttentionFirst>,
}

/// What `assist_attention` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AttentionPanel {
    /// بیوپاریوں کو ادائیگی. The button "سب دیکھیں" opens the seller dues page on `shop_owes`.
    pub sellers: AttentionPartyGroup,
    /// گاہکوں سے وصولی. The button opens the buyer dues page.
    pub buyers: AttentionPartyGroup,
    /// The head 02 payables still to pay, without the hidden ones.
    pub payables: Vec<AttentionPayableRow>,
    /// The other reminders, without the hidden ones.
    pub general: Vec<AttentionPanelItem>,
    /// The number on the bell: the rows shown (sellers + buyers + payables + general).
    #[ts(type = "number")]
    pub bell_count: i64,
    /// How many items are hidden until the next بکری save.
    #[ts(type = "number")]
    pub hidden_count: i64,
    /// True while the partner section is locked: the amount of the partner takes item is left out.
    pub partners_locked: bool,
}
