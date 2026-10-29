//! The خرچہ screen (PRD 16.2, 16.3, FR-7.67, FR-16.8, FR-16.9, FR-7.62, plan 17): a shop expense, a
//! partner's personal spending and its return, on one screen with a type list. Each type keeps its
//! own input, number series and posting; the preview works the effect out with the same function
//! as the save (Code Standards B5).
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `vouchers_save_shop_expense` | `ShopExpenseInput` | `VoucherSaved` |
//! | `vouchers_save_personal_expense` | `PersonalExpenseInput` | `VoucherSaved` |
//! | `vouchers_save_personal_return` | `PersonalReturnInput` | `VoucherSaved` |
//! | `vouchers_preview_expense` | `ExpensePreviewInput` | `ExpensePreview` |
//! | `partner_takes` | `PartnerTakesInput` | `PartnerTakes` |
//! | `vouchers_repeat_last_month_preview` | `RepeatLastMonthInput` | `RepeatLastMonth` |
//! | `masters_expense_heads` | none | `ExpenseHeads` |
//!
//! There is no mode, bank or cheque on any of them (PRD v1.6): everything posts روکڑ. Number
//! series (letters from Fluent): shop expense `خ-` / `E-`, personal expense `ذ-` / `D-`, personal
//! return `ذو-` / `DR-`.
//!
//! Worked numbers (PRD E2E-39, E2E-20): a shop expense of 3,000 at 40 / 30 / 30 shows الف 1,200,
//! ب 900, ج 900 before the save; a shop expense of 3,384 splits 1,354 / 1,015 / 1,015. Then 2,000
//! as a personal expense of الف shows his کل رقم 996 now and −1,004 after; ب and ج are not affected.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::cash::CashCheck;
use crate::error::FieldError;
use crate::money::Paisa;
use crate::paging::{MonthSubtotal, SortInput};
use crate::rules::Working;
use crate::vouchers::VoucherType;

/// A دکان خرچہ as typed (PRD 16.2): head بنام, روکڑ جمع, split by the shares of the date (E2E-20).
///
/// Field paths in errors: `date`, `expense_head_id`, `amount`, `paid_to`, `narration`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ShopExpenseInput {
    /// The day, `DD-MM-YYYY` or a shortcut. An open day, not after today (PRD FR-6.6).
    pub date: String,
    /// The expense head (the master row of `masters_expense_heads`). Null gives
    /// `ERR_EXPENSE_HEAD_REQUIRED`; a head that does not exist gives `ERR_NOT_FOUND`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub expense_head_id: Option<i64>,
    /// Whole rupees, more than 0.
    pub amount: String,
    /// Who was paid, as typed (PRD FR-7.25). Up to 100 characters.
    #[serde(default)]
    pub paid_to: Option<String>,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub narration: Option<String>,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟" (PRD FR-6.7). Needed only when the
    /// expense takes روکڑ below 0.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// A ذاتی خرچہ as typed (PRD 16.3, FR-16.2): the partner's drawings account بنام, روکڑ جمع. Never
/// split.
///
/// Field paths in errors: `date`, `partner_id`, `amount`, `reason`, `return_by`, `taken_by`,
/// `narration`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PersonalExpenseInput {
    /// The day, `DD-MM-YYYY` or a shortcut. An open day, not after today.
    pub date: String,
    /// The partner. Null gives `ERR_PARTNER_REQUIRED`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
    /// Whole rupees, more than 0.
    pub amount: String,
    /// Why. Required (`ERR_REASON_REQUIRED`).
    pub reason: String,
    /// The day it should be returned, `DD-MM-YYYY`. Not before `date` (`ERR_RETURN_BY_BEFORE_ENTRY`,
    /// PRD FR-16.9); the same day is allowed.
    #[serde(default)]
    pub return_by: Option<String>,
    /// The person who took the cash, if not the partner himself, as typed.
    #[serde(default)]
    pub taken_by: Option<String>,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub narration: Option<String>,
    /// True after the user said yes to the cash box (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// A ذاتی خرچہ واپسی as typed (PRD 16.3): a partner gives money back. روکڑ بنام, drawings جمع.
///
/// Field paths in errors: `date`, `partner_id`, `amount`, `narration`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PersonalReturnInput {
    /// The day, `DD-MM-YYYY` or a shortcut. An open day, not after today.
    pub date: String,
    /// The partner. Null gives `ERR_PARTNER_REQUIRED`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
    /// Whole rupees, more than 0. More than his open amount is `ERR_RETURN_EXCEEDS_OPEN` (value
    /// `open_paisa`, field `amount`).
    pub amount: String,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub narration: Option<String>,
}

/// Any one of the three types of the خرچہ screen, for the preview (PRD FR-7.67). The same shape as
/// `VoucherEntry`: the fields of the type sit beside `type`.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpenseEntry {
    /// دکان خرچہ.
    ShopExpense(ShopExpenseInput),
    /// ذاتی خرچہ.
    PersonalExpense(PersonalExpenseInput),
    /// ذاتی خرچہ واپسی.
    PersonalReturn(PersonalReturnInput),
}

impl ExpenseEntry {
    /// The voucher type of the entry.
    pub const fn voucher_type(&self) -> VoucherType {
        match self {
            ExpenseEntry::ShopExpense(_) => VoucherType::ShopExpense,
            ExpenseEntry::PersonalExpense(_) => VoucherType::PersonalExpense,
            ExpenseEntry::PersonalReturn(_) => VoucherType::PersonalReturn,
        }
    }
}

/// Input of `vouchers_preview_expense` (PRD FR-16.8). Nothing is written.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ExpensePreviewInput {
    /// The entry as typed.
    pub entry: ExpenseEntry,
    /// Set when a saved voucher is being edited: the preview works as if its posting were taken
    /// back first (PRD FR-15.3).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub replacing_voucher_id: Option<i64>,
}

/// One partner's part of a دکان خرچہ (PRD C11), before the save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ExpenseShare {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His share on the date, in bps.
    pub share_bps: i32,
    /// His part of the expense as a positive number (the stored share row is negative). The parts
    /// add up to the expense.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// How the part was worked out (CalcInfo).
    pub working: Working,
}

/// What a personal expense or its return does to one partner's کل رقم (PRD C22, FR-16.8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartnerTotalEffect {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His کل رقم now (signed).
    #[ts(type = "Paisa")]
    pub total_before_paisa: Paisa,
    /// His کل رقم after this entry (signed).
    #[ts(type = "Paisa")]
    pub total_after_paisa: Paisa,
    /// His open money taken now (FR-16.2), before this entry.
    #[ts(type = "Paisa")]
    pub open_before_paisa: Paisa,
    /// His open money taken after this entry.
    #[ts(type = "Paisa")]
    pub open_after_paisa: Paisa,
    /// How the change was worked out.
    pub working: Working,
}

/// An expense entry checked and worked out, nothing saved (PRD FR-16.8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ExpensePreview {
    /// The type of the entry.
    pub voucher_type: VoucherType,
    /// The amount, 0 if it could not be read.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// The expense head account (دکان خرچہ), or null.
    pub head: Option<RecordRef>,
    /// The partner (ذاتی خرچہ and its return), or null.
    pub partner: Option<RecordRef>,
    /// دکان خرچہ: each active partner's part, in the order the partners were created. Empty for the
    /// other two types.
    pub shares: Vec<ExpenseShare>,
    /// ذاتی خرچہ and its return: the chosen partner's total now and after. Null for دکان خرچہ.
    pub partner_effect: Option<PartnerTotalEffect>,
    /// True for ذاتی خرچہ and its return: the other partners are not affected (the screen says so).
    pub others_not_affected: bool,
    /// دکان خرچہ: the partner whose name (either script, compared after normalising) equals
    /// `paid_to`, for the note "یہ شراکت دار کا ذاتی خرچہ تو نہیں؟" (PRD FR-7.68). It only offers; it
    /// never switches the type. Null otherwise.
    pub paid_to_partner: Option<RecordRef>,
    /// Cash in hand after the entry (PRD FR-6.7).
    pub cash: CashCheck,
    /// What stops a save, at the fields as sent.
    pub errors: Vec<FieldError>,
    /// What only warns.
    pub warnings: Vec<FieldError>,
}

// ---- Money taken by a partner (PRD FR-16.2) ----

/// Input of `partner_takes`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartnerTakesInput {
    /// The partner. `ERR_NOT_FOUND` on `partner_id`.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives the default page of 100 rows (PRD FR-19.11).
    #[serde(default)]
    pub limit: u32,
    /// The column to sort the whole range by: `entry_date`, `source`, `number`, `amount_paisa`, `reason`, `taken_by`, `return_by`, `returned_paisa`, `open_paisa` or `days_past_return_by`. Null is the natural order (plan 38
    /// R4). A sorted row still shows its true running figure of the natural order.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// True to give the subtotal of each month in `months`. Used only in date order.
    #[serde(default)]
    pub group_by_month: bool,
}

/// Where a row of money taken came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TakeSource {
    /// A ذاتی خرچہ voucher. `number` is its number.
    Voucher,
    /// An opening debit on his drawings account (PRD FR-10.9): money he had taken before the app.
    Opening,
}

/// One amount a partner took, with what has been returned against it (oldest first, PRD FR-16.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TakeRow {
    /// Where it came from.
    pub source: TakeSource,
    /// The voucher, for source `voucher`.
    #[ts(type = "number | null")]
    pub voucher_id: Option<i64>,
    /// The voucher number, for source `voucher`.
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// What was taken.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// Why. Empty for an opening.
    pub reason: String,
    /// The person who took the cash. Empty if the partner himself.
    pub taken_by: String,
    /// The return-by day, `YYYY-MM-DD`, or null.
    pub return_by: Option<String>,
    /// What oldest-first returns have settled on it.
    #[ts(type = "Paisa")]
    pub returned_paisa: Paisa,
    /// Taken less returned.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
    /// Days past `return_by` today, or null (no date, nothing open, or not yet due).
    #[ts(type = "number | null")]
    pub days_past_return_by: Option<i64>,
}

/// What a partner's money taken adds up to, over the whole list (B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TakesTotals {
    /// Σ taken.
    #[ts(type = "Paisa")]
    pub taken_paisa: Paisa,
    /// Σ returned (all returns, also the part above the rows of this page).
    #[ts(type = "Paisa")]
    pub returned_paisa: Paisa,
    /// Σ still open.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
}

/// A partner's money taken, one page, oldest first (PRD FR-16.2, FR-19.11). Read only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartnerTakes {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// The page.
    pub rows: Vec<TakeRow>,
    /// How many rows there are in all.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over all the rows.
    pub totals: TakesTotals,
    /// The count and subtotals of each month of the whole range, in the order of the list, when
    /// asked with `group_by_month` and the list is in date order. Figures: `taken_paisa`, `returned_paisa` and `open_paisa`.
    pub months: Vec<MonthSubtotal>,
}

// ---- Repeat last month (PRD FR-7.62) ----

/// Input of `vouchers_repeat_last_month_preview`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct RepeatLastMonthInput {
    /// The month to fill, `YYYY-MM`. Not in that form gives `ERR_FORMAT_INVALID` on `month`.
    pub month: String,
}

/// One regular expense proposed from last month, for the user to check and save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RepeatedExpense {
    /// The expense head account.
    pub head: RecordRef,
    /// The master row to send as `ShopExpenseInput::expense_head_id`.
    #[ts(type = "number")]
    pub expense_head_id: i64,
    /// Last month's amount for this head (its latest saved دکان خرچہ).
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// Who was paid last time. Empty if none.
    pub paid_to: String,
    /// The note last time. Empty if none.
    pub narration: String,
    /// The day of that expense, `YYYY-MM-DD`.
    pub last_date: String,
}

/// The regular expenses of last month that are not yet entered in the month asked for. Nothing is
/// saved; the user checks each row and saves it as a normal دکان خرچہ. Read only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RepeatLastMonth {
    /// The month asked for, `YYYY-MM`.
    pub month: String,
    /// Last month, `YYYY-MM`.
    pub last_month: String,
    /// The proposals, most used heads first.
    pub rows: Vec<RepeatedExpense>,
    /// Σ of the proposals.
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
}

// ---- Expense heads, most used first (PRD FR-7.62) ----

/// One expense head of the master list with its use. Named `ExpenseHeadUse` (not `ExpenseHeadRow`)
/// because the generated TypeScript file of `masters::ExpenseHeadRow` would be overwritten by it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ExpenseHeadUse {
    /// The master row (`expense_heads.id`), to send as `expense_head_id`.
    #[ts(type = "number")]
    pub id: i64,
    /// Its code and names.
    pub head: RecordRef,
    /// The account under 0501 that it posts to.
    #[ts(type = "number")]
    pub account_id: i64,
    /// How many saved, not cancelled, دکان خرچہ it has.
    #[ts(type = "number")]
    pub use_count: i64,
    /// The day of its last one, `YYYY-MM-DD`, or null.
    pub last_used_date: Option<String>,
}

/// The active expense heads, most used first (ties: by code). Read only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ExpenseHeads {
    /// The heads.
    pub rows: Vec<ExpenseHeadUse>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vouchers::VoucherEntry;

    /// Every new kind has its stored name, and it reads back (Architecture 15.7).
    #[test]
    fn every_voucher_type_reads_back_from_its_stored_name() {
        let names = [
            ("shop_expense", VoucherType::ShopExpense),
            ("personal_expense", VoucherType::PersonalExpense),
            ("personal_return", VoucherType::PersonalReturn),
            ("money_lent", VoucherType::MoneyLent),
            ("money_lent_return", VoucherType::MoneyLentReturn),
            ("capital_in", VoucherType::CapitalIn),
            ("capital_out", VoucherType::CapitalOut),
            ("khata_out", VoucherType::KhataOut),
            ("khata_in", VoucherType::KhataIn),
            ("general", VoucherType::General),
        ];
        for (name, kind) in names {
            assert_eq!(kind.as_str(), name);
            assert_eq!(VoucherType::parse(name), Some(kind));
        }
    }

    /// The type tag picks the input (PRD FR-7.67), for the expense preview and for the edit.
    #[test]
    fn the_type_tag_picks_the_input() {
        let text = r#"{ "type": "shop_expense", "date": "10-10-2026", "expense_head_id": 1,
            "amount": "3,000" }"#;
        let entry: ExpenseEntry = serde_json::from_str(text).unwrap();
        assert_eq!(entry.voucher_type(), VoucherType::ShopExpense);
        let any: VoucherEntry = serde_json::from_str(text).unwrap();
        assert_eq!(any.voucher_type(), VoucherType::ShopExpense);
        let capital: VoucherEntry = serde_json::from_str(
            r#"{ "type": "capital_out", "date": "20-10-2026", "partner_id": 2, "amount": "10,000" }"#,
        )
        .unwrap();
        assert_eq!(capital.voucher_type(), VoucherType::CapitalOut);
    }

    /// A missing partner, head, person or khata is `null`, so the backend can answer with the
    /// `..._REQUIRED` code at the field instead of the IPC layer refusing the shape.
    #[test]
    fn a_missing_choice_reads_as_null() {
        let input: PersonalExpenseInput =
            serde_json::from_str(r#"{ "date": "10-10-2026", "amount": "2,000", "reason": "x" }"#)
                .unwrap();
        assert_eq!(input.partner_id, None);
        assert!(!input.confirm_cash_below_zero);
    }
}
