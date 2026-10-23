//! جنرل: a journal entry of free lines (PRD 16.4, FR-6.2, FR-6.5, FR-15.4, plan 17). It is used
//! for corrections, transfers and goods returns. It saves only when Σ بنام = Σ جمع.
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `general_preview` | `GeneralPreviewInput` | `GeneralPreview` |
//! | `general_save` | `GeneralInput` | `VoucherSaved` (`new_balance` is zero: a journal has no one account) |
//!
//! Number series (letters from Fluent): `ج-` / `G-`. Lines on income or shop expense accounts are
//! split between the partners (PRD FR-6.5); lines on buyer accounts take part in bill matching
//! (FR-15.4); a line on a seller account never adjusts his advances (a debit settles his payables,
//! a credit is a payable posting, PRD 16.4); a line on اپنا مال is refused with
//! `ERR_OWN_GOODS_ACCOUNT_RESTRICTED`; a cash credit uses the cash box (PRD FR-6.7).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::{Balance, RecordRef};
use crate::cash::CashCheck;
use crate::error::FieldError;
use crate::expenses::ExpenseShare;
use crate::money::Paisa;

/// One line of a جنرل entry as typed.
///
/// Field paths in errors: `lines[i].account_id`, `lines[i].person_key`, `lines[i].dr`,
/// `lines[i].cr`, `lines[i].narration`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GeneralLineInput {
    /// The account. Null gives `ERR_REQUIRED`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub account_id: Option<i64>,
    /// The one-time person, for a پختہ کھاتہ account (PRD FR-13.5). Null for any other account.
    #[serde(default)]
    pub person_key: Option<String>,
    /// بنام, whole rupees. Empty for none.
    #[serde(default)]
    pub dr: String,
    /// جمع, whole rupees. Empty for none. A line has one side only (`ERR_FORMAT_INVALID` on
    /// `lines[i].cr` if both are more than 0, `ERR_REQUIRED` on `lines[i].dr` if neither is).
    #[serde(default)]
    pub cr: String,
    /// A note for the line, up to 500 characters.
    #[serde(default)]
    pub narration: Option<String>,
}

/// A جنرل entry to save (PRD 16.4).
///
/// Field paths in errors: `date`, `narration`, `lines`, and the paths of the lines above.
/// `ERR_GENERAL_TOO_FEW_LINES` on `lines` (fewer than two); `ERR_GENERAL_NOT_BALANCED` on `lines`
/// (values `dr_paisa`, `cr_paisa`); nothing is written.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GeneralInput {
    /// The day, `DD-MM-YYYY` or a shortcut. An open day, not after today.
    pub date: String,
    /// A note for the whole entry, up to 500 characters.
    #[serde(default)]
    pub narration: Option<String>,
    /// The lines, at least two.
    pub lines: Vec<GeneralLineInput>,
    /// True after the user said yes to the cash box (PRD FR-6.7). Needed when the entry credits
    /// روکڑ and takes it below 0.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// Input of `general_preview`: the lines as typed. The date is optional; with it the partner split
/// of income and expense lines is shown (the shares of that day).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GeneralPreviewInput {
    /// The day, `DD-MM-YYYY` or a shortcut. Null or empty to skip the split and the date check.
    #[serde(default)]
    pub date: Option<String>,
    /// The lines as typed.
    pub lines: Vec<GeneralLineInput>,
}

/// One line of the preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GeneralLinePreview {
    /// The account, or null if the id is missing or wrong.
    pub account: Option<RecordRef>,
    /// بنام, 0 if empty or unreadable.
    #[ts(type = "Paisa")]
    pub dr_paisa: Paisa,
    /// جمع, 0 if empty or unreadable.
    #[ts(type = "Paisa")]
    pub cr_paisa: Paisa,
    /// The account's balance after this entry, or null if the line has errors.
    pub balance_after: Option<Balance>,
    /// What is wrong with the line (paths `lines[i].<field>`). Empty if nothing.
    pub errors: Vec<FieldError>,
}

/// A جنرل entry checked, nothing saved. The totals are worked out by the backend; the screen
/// never adds (CLAUDE.md rule 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GeneralPreview {
    /// One row per line, in the order sent.
    pub lines: Vec<GeneralLinePreview>,
    /// Σ بنام of the lines that could be read.
    #[ts(type = "Paisa")]
    pub total_dr_paisa: Paisa,
    /// Σ جمع of the lines that could be read.
    #[ts(type = "Paisa")]
    pub total_cr_paisa: Paisa,
    /// بنام less جمع. 0 when balanced.
    #[ts(type = "Paisa")]
    pub difference_paisa: Paisa,
    /// True when the totals are equal and more than 0.
    pub is_balanced: bool,
    /// True when the entry would save: balanced, at least two lines and no error. The screen keeps
    /// Save disabled until this is true.
    pub can_save: bool,
    /// The partners' parts of the income and shop expense lines (PRD FR-6.5), net of the lines, when
    /// a date was sent. Empty if there is no such line.
    pub split: Vec<ExpenseShare>,
    /// Cash in hand after the entry (PRD FR-6.7).
    pub cash: CashCheck,
    /// What stops a save and belongs to the whole entry (`lines`, `date`).
    pub errors: Vec<FieldError>,
    /// What only warns.
    pub warnings: Vec<FieldError>,
}
