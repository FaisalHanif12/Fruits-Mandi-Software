//! The one-time fix "پرانے ابتدائی بقایا روکڑ میں درج کریں" (PRD FR-10.9 v1.60, plan 99): the openings
//! of sellers, buyers, persons of money lent and الگ کھاتے that were saved against ابتدائی فرق are
//! reversed and posted again on the same date against روکڑ.
//!
//! Commands `accounts_openings_to_cash_preview` (no input) and `accounts_openings_to_cash` (input
//! [`OpeningsToCashInput`]). The save needs a reason and the Owner's sign (PRD FR-21.4); it reopens
//! the closed days from the first opening and closes them again in the same transaction.
//!
//! No party balance and no partner total changes. Worked numbers (plan 99 Section 4): openings
//! بنام 125,000 and جمع 40,000; cash 1,057,840 becomes 1,057,840 − 125,000 + 40,000 = 972,840, and
//! ابتدائی فرق 85,000 جمع becomes 0.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Balance, RecordRef};
use crate::money::Paisa;

/// Whose opening it is: the four kinds whose openings go through روکڑ (PRD FR-10.9 v1.60).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum OpeningPartyKind {
    /// بیوپاری (control 03011).
    Seller,
    /// گاہک (control 0304).
    Buyer,
    /// A person of ادھار رقم (control 0103).
    MoneyLent,
    /// A الگ کھاتہ (control 0305).
    SeparateKhata,
}

/// One opening the fix moves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OpeningToCashRow {
    /// The account.
    pub account: RecordRef,
    /// Its kind.
    pub kind: OpeningPartyKind,
    /// The day of the opening, `YYYY-MM-DD` (unchanged).
    pub date: String,
    /// The opening بنام (unchanged), 0 when it is جمع: this much goes out of cash.
    #[ts(type = "Paisa")]
    pub dr_paisa: Paisa,
    /// The opening جمع (unchanged), 0 when it is بنام: this much comes into cash.
    #[ts(type = "Paisa")]
    pub cr_paisa: Paisa,
    /// What it does to cash in hand: `cr − dr` (signed).
    #[ts(type = "Paisa")]
    pub cash_change_paisa: Paisa,
    /// True when its day is closed: the save reopens it and closes it again.
    pub day_closed: bool,
}

/// What the fix would do. Nothing is written by the preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OpeningsToCashPreview {
    /// The openings to move, by day then account code. Empty when there are none: the screen then
    /// hides the button (the save answers `ERR_OPENINGS_ALREADY_IN_CASH`).
    pub rows: Vec<OpeningToCashRow>,
    /// Σ بنام: out of cash.
    #[ts(type = "Paisa")]
    pub total_dr_paisa: Paisa,
    /// Σ جمع: into cash.
    #[ts(type = "Paisa")]
    pub total_cr_paisa: Paisa,
    /// Σ جمع − Σ بنام (signed).
    #[ts(type = "Paisa")]
    pub cash_change_paisa: Paisa,
    /// Cash in hand now (signed).
    #[ts(type = "Paisa")]
    pub cash_before_paisa: Paisa,
    /// Cash in hand after the fix (signed).
    #[ts(type = "Paisa")]
    pub cash_after_paisa: Paisa,
    /// True when cash would end below zero while the change takes cash out: the save then needs
    /// `confirm_cash_below_zero` (PRD FR-6.7).
    pub needs_cash_confirm: bool,
    /// ابتدائی فرق now.
    pub opening_difference_before: Balance,
    /// ابتدائی فرق after the fix.
    pub opening_difference_after: Balance,
    /// The first closed day that the save reopens, `YYYY-MM-DD`, or null when every opening is on
    /// an open day.
    pub reopens_from: Option<String>,
}

/// Input of `accounts_openings_to_cash`.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct OpeningsToCashInput {
    /// Why. Required (as FR-21.2).
    #[serde(default)]
    pub reason: String,
    /// The Owner's sign (دستخط, PRD FR-21.4). Required.
    #[serde(default)]
    pub sign: String,
    /// The answer "ہاں، محفوظ کریں" of the cash box (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// The fix done.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OpeningsToCashDone {
    /// The openings moved, in the order of the preview.
    pub rows: Vec<OpeningToCashRow>,
    /// Σ بنام moved.
    #[ts(type = "Paisa")]
    pub total_dr_paisa: Paisa,
    /// Σ جمع moved.
    #[ts(type = "Paisa")]
    pub total_cr_paisa: Paisa,
    /// Cash in hand after the fix (signed).
    #[ts(type = "Paisa")]
    pub cash_after_paisa: Paisa,
    /// ابتدائی فرق after the fix.
    pub opening_difference_after: Balance,
    /// The first closed day that was reopened and closed again, `YYYY-MM-DD`, or null.
    pub reopened_from: Option<String>,
    /// The "closed up to" day after the fix (the same as before), `YYYY-MM-DD`, or null.
    pub closed_up_to: Option<String>,
}
