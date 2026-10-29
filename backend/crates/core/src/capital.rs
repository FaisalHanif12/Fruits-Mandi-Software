//! Partner capital: money a partner invests and takes back (PRD 17.4, FR-17.7, FR-17.8, plan 17).
//! One screen with a type list (invested, taken back) and the partner's capital history.
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `vouchers_save_capital_in` | `CapitalInput` | `VoucherSaved` |
//! | `vouchers_save_capital_out` | `CapitalInput` | `VoucherSaved` |
//! | `capital_history` | `CapitalHistoryInput` | `CapitalHistory` |
//!
//! Number series (letters from Fluent): invested `س-` / `CI-`, taken back `سو-` / `CO-`. Capital is
//! never income or expense and is never split. Taking back more than the capital balance gives the
//! warning `ERR_CAPITAL_EXCEEDS_BALANCE` (value `balance_paisa`, `over_paisa`; field `amount`) in the
//! preview and still saves (PRD FR-17.8).
//!
//! Worked numbers (PRD E2E-35): on 11-10-26 ب invests 50,000 in cash: cash 108,484, his capital
//! account 50,000 جمع, his profit share still 746, trial balance 135,048 on both sides.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::money::Paisa;
use crate::paging::{MonthSubtotal, SortInput};

/// A سرمایہ جمع or سرمایہ واپسی as typed. Invested: روکڑ بنام, his capital account (0602) جمع. Taken
/// back: the opposite.
///
/// Field paths in errors: `date`, `partner_id`, `amount`, `narration`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CapitalInput {
    /// The day, `DD-MM-YYYY` or a shortcut. An open day, not after today.
    pub date: String,
    /// The partner. Null gives `ERR_PARTNER_REQUIRED`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
    /// Whole rupees, more than 0.
    pub amount: String,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub narration: Option<String>,
    /// True after the user said yes to the cash box (PRD FR-6.7). Needed only for `capital_out`
    /// when it takes روکڑ below 0; ignored by `capital_in`.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// Input of `capital_history`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CapitalHistoryInput {
    /// The partner. `ERR_NOT_FOUND` on `partner_id`.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives the default page of 100 rows (PRD FR-19.11).
    #[serde(default)]
    pub limit: u32,
    /// The column to sort the whole range by: `entry_date`, `kind`, `number`, `amount_paisa`, `narration` or `running_paisa`. Null is the natural order (plan 38
    /// R4). A sorted row still shows its true running figure of the natural order.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// True to give the subtotal of each month in `months`. Used only in date order.
    #[serde(default)]
    pub group_by_month: bool,
}

/// What a row of the capital history is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CapitalRowKind {
    /// His opening capital (PRD FR-17.1, FR-10.9).
    Opening,
    /// سرمایہ جمع.
    Invested,
    /// سرمایہ واپسی.
    TakenBack,
    /// Month close moved his profit into his capital account (PRD FR-17.6). Shown as a row of its
    /// own; his کل رقم does not change by it.
    MonthClose,
}

/// One row of a partner's capital history, oldest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalRow {
    /// What it is.
    pub kind: CapitalRowKind,
    /// The voucher, for `invested` and `taken_back`.
    #[ts(type = "number | null")]
    pub voucher_id: Option<i64>,
    /// The voucher number, for `invested` and `taken_back`.
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// Signed from his side: invested is plus, taken back is minus.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// The note. Empty if none.
    pub narration: String,
    /// His capital balance after this row (signed, credit is plus).
    #[ts(type = "Paisa")]
    pub running_paisa: Paisa,
}

/// What the history adds up to over the whole list (B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalTotals {
    /// Σ invested (opening included).
    #[ts(type = "Paisa")]
    pub invested_paisa: Paisa,
    /// Σ taken back.
    #[ts(type = "Paisa")]
    pub taken_back_paisa: Paisa,
}

/// A partner's capital history, one page (PRD 17.4, FR-19.11). Read only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalHistory {
    /// The partner. `code` is the code of his capital account.
    pub partner: RecordRef,
    /// His capital balance now (signed, credit is plus).
    #[ts(type = "Paisa")]
    pub balance_paisa: Paisa,
    /// The page.
    pub rows: Vec<CapitalRow>,
    /// How many rows there are in all.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over all the rows.
    pub totals: CapitalTotals,
    /// The count and subtotals of each month of the whole range, in the order of the list, when
    /// asked with `group_by_month` and the list is in date order. Figures: `invested_paisa` and `taken_back_paisa`.
    pub months: Vec<MonthSubtotal>,
}
