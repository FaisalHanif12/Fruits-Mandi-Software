//! الگ کھاتہ: a separate khata kept apart from the partners, the commission and the profit (PRD
//! 16.6, FR-16.6, FR-16.7, E54, plan 17). One screen with a type list (paid out, received) and the
//! khata's statement.
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `khata_create` | `KhataCreateInput` | `KhataAccount` |
//! | `vouchers_save_khata_out` | `KhataOutInput` | `VoucherSaved` |
//! | `vouchers_save_khata_in` | `KhataInInput` | `VoucherSaved` |
//! | `khata_statement` | `KhataStatementInput` | `KhataStatement` |
//!
//! Number series (letters from Fluent): paid out `کد-` / `KO-`, received `کا-` / `KI-`. A khata is
//! never income or expense, never split, and never in any partner's total (PRD C22). Its balance
//! is signed from the khata's side: plus means the shop holds money for it, minus means it has
//! taken more than it brought. A khata can be the seller of a lot in بکری (PRD FR-16.7): its seller
//! credit goes to the khata, and every seller-side default is 0 for such a lot.
//!
//! Worked numbers (PRD E2E-38, خاندانی کھاتہ): 10-10-26 pays out 10,000 (−10,000, cash 40,000);
//! 11-10-26 gets 4,000 back (−6,000, cash 44,000); then a lot sells 10 پیٹی کیلا at 900 to ارشد:
//! 9,000 to the khata, which shows +3,000.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::money::Paisa;
use crate::paging::{MonthSubtotal, SortInput};

/// A new separate khata (the `F4` of the screen, PRD 16.6): an account under control 0305.
///
/// Field paths in errors: `name_ur`, `name_en`, `note`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct KhataCreateInput {
    /// Urdu name (PRD FR-7.28), for example خاندانی کھاتہ.
    pub name_ur: String,
    /// English name, for example Family Khata.
    pub name_en: String,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub note: Option<String>,
    /// Words the user confirmed in the spelling check (PRD FR-7.66).
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
}

/// A separate khata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct KhataAccount {
    /// The account (control 0305).
    pub account: RecordRef,
    /// The note. Empty if none.
    pub note: String,
    /// Its balance now, signed from the khata's side: plus is credit, minus is debit.
    #[ts(type = "Paisa")]
    pub balance_paisa: Paisa,
}

/// رقم دی as typed (PRD 16.6): the khata بنام (minus), روکڑ جمع.
///
/// Field paths in errors: `date`, `khata_account_id`, `amount`, `reason`, `narration`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct KhataOutInput {
    /// The day, `DD-MM-YYYY` or a shortcut. An open day, not after today.
    pub date: String,
    /// The khata (control 0305). Null gives `ERR_KHATA_REQUIRED`; an account of another kind gives
    /// `ERR_FORMAT_INVALID`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub khata_account_id: Option<i64>,
    /// Whole rupees, more than 0.
    pub amount: String,
    /// Why. Required (`ERR_REASON_REQUIRED`).
    pub reason: String,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub narration: Option<String>,
    /// True after the user said yes to the cash box (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// رقم آئی as typed (PRD 16.6): روکڑ بنام, the khata جمع (plus).
///
/// Field paths in errors: `date`, `khata_account_id`, `amount`, `reason`, `narration`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct KhataInInput {
    /// The day, `DD-MM-YYYY` or a shortcut. An open day, not after today.
    pub date: String,
    /// The khata. Null gives `ERR_KHATA_REQUIRED`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub khata_account_id: Option<i64>,
    /// Whole rupees, more than 0.
    pub amount: String,
    /// Why. Required (`ERR_REASON_REQUIRED`).
    pub reason: String,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub narration: Option<String>,
    /// Sent by the screen with every cash entry; ignored here, because money coming in cannot take
    /// cash below 0. An edit that lowers cash uses the one on `VoucherEditInput`.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// Input of `khata_statement` (PRD FR-16.6, FR-19.10, FR-19.11).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct KhataStatementInput {
    /// The khata. `ERR_NOT_FOUND` on `khata_account_id`.
    #[ts(type = "number")]
    pub khata_account_id: i64,
    /// First day, `DD-MM-YYYY` or a shortcut. Null or empty means the start.
    #[serde(default)]
    pub from: Option<String>,
    /// Last day. Null or empty means today.
    #[serde(default)]
    pub to: Option<String>,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives the default page of 100 rows.
    #[serde(default)]
    pub limit: u32,
    /// The column to sort the whole range by: `entry_date`, `kind`, `number`, `reason`,
    /// `narration`, `amount_paisa` or `running_paisa`. Null is by day and entry (plan 38 R4). A
    /// sorted row still shows its true running balance of the natural order.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// True to give the subtotal of each month in `months`. Used only in date order.
    #[serde(default)]
    pub group_by_month: bool,
}

/// What a row of a khata statement is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum KhataRowKind {
    /// رقم دی (minus).
    PaidOut,
    /// رقم آئی (plus).
    Received,
    /// The seller credit of a lot of the khata's goods sold in بکری (plus, PRD FR-16.7). `number` is
    /// the بکری number.
    Sale,
    /// A جنرل entry or an opening on the khata.
    Other,
}

/// One entry of a khata statement, in date order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct KhataEntryRow {
    /// What it is.
    pub kind: KhataRowKind,
    /// The journal entry.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// The voucher, for `paid_out` and `received`.
    #[ts(type = "number | null")]
    pub voucher_id: Option<i64>,
    /// The voucher number (series کد- or کا-) or the بکری number for `sale`.
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// Why. Empty if none.
    pub reason: String,
    /// The note. Empty if none.
    pub narration: String,
    /// Signed from the khata's side: paid out is minus, received and sales are plus.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// The khata's balance after this entry.
    #[ts(type = "Paisa")]
    pub running_paisa: Paisa,
}

/// What a statement adds up to, over the whole range (B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct KhataTotals {
    /// Σ paid out (positive).
    #[ts(type = "Paisa")]
    pub paid_out_paisa: Paisa,
    /// Σ received and sales (positive).
    #[ts(type = "Paisa")]
    pub received_paisa: Paisa,
}

/// A khata's statement, one page (PRD 16.6, FR-19.11). Read only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct KhataStatement {
    /// The khata.
    pub khata: KhataAccount,
    /// Its balance at the start of the range (signed, credit is plus).
    #[ts(type = "Paisa")]
    pub opening_paisa: Paisa,
    /// Its balance at the end of the range.
    #[ts(type = "Paisa")]
    pub closing_paisa: Paisa,
    /// The page.
    pub rows: Vec<KhataEntryRow>,
    /// How many entries there are in the range.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over the whole range.
    pub totals: KhataTotals,
    /// The count and subtotals of each month of the whole range, in the order of the list, when
    /// asked with `group_by_month` and the list is in date order. Figures: `paid_out_paisa` and
    /// `received_paisa`.
    pub months: Vec<MonthSubtotal>,
}
