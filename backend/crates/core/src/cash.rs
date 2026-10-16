//! Cash in hand after an entry (PRD FR-6.7, plan 12): the one shape every cash-out screen shows
//! in its amber note and every cash-out save checks.
//!
//! The rule itself is `mandi_domain::cash::check`; the balance is read in `mandi_services::cash`.
//! This module only holds the shape, so the preview of a بکری, a voucher and a save all speak
//! the same words.

use serde::Serialize;
use ts_rs::TS;

use crate::money::Paisa;

/// What an entry does to cash in hand (روکڑ, account 01010001).
///
/// Example (PRD FR-6.7): cash is 20,000 and a lot has نقد 25,000. `cash_after` is −5,000, the
/// entry lowers cash, so `below_zero` and `needs_confirm` are true.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct CashCheck {
    /// Cash in hand now, from the posted journal.
    #[ts(type = "Paisa")]
    pub cash_before_paisa: Paisa,
    /// What this entry adds to cash (negative when it takes cash out).
    #[ts(type = "Paisa")]
    pub change_paisa: Paisa,
    /// Cash in hand after the entry.
    #[ts(type = "Paisa")]
    pub cash_after_paisa: Paisa,
    /// True if `cash_after_paisa` is below 0.
    pub below_zero: bool,
    /// True if the entry takes cash out (its change is below 0).
    pub lowers_cash: bool,
    /// True if the user must confirm before the save: cash ends below 0 AND the entry lowers
    /// cash. A cash balance that is already below 0 does not ask again for an entry that does
    /// not lower it.
    pub needs_confirm: bool,
}
