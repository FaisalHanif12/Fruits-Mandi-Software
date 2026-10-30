//! ادھار رقم: money lent to staff and others (PRD 16.5, FR-16.4, FR-16.5, FR-16.9, plan 17). One
//! screen with a type list (lent, returned), a person list and a report.
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `vouchers_save_money_lent` | `MoneyLentInput` | `VoucherSaved` |
//! | `vouchers_save_money_lent_return` | `MoneyLentReturnInput` | `VoucherSaved` |
//! | `lent_person_create` | `LentPersonCreateInput` | `LentPerson` |
//! | `lent_list` | `LentListInput` | `LentList` |
//! | `lent_report` | `LentReportInput` | `LentReport` |
//! | `assist_attention_overdue` | none | `AttentionOverdue` |
//!
//! Number series (letters from Fluent): lent `ر-` / `L-`, returned `رو-` / `LR-`. A return settles
//! the person's oldest open loans first, worked out when shown; nothing is stored (PRD FR-16.5).
//!
//! Worked numbers (PRD E2E-30, اسلم): 10-10-26 lent 5,000 (cash 53,484, open 5,000); 11-10-26
//! returns 3,000 in cash (open 2,000, cash 56,484); 12-10-26 2,000 from salary (open 0, cash still
//! 56,484; تنخواہ 2,000 split −800 / −600 / −600).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::{Balance, RecordRef};
use crate::money::Paisa;
use crate::paging::{MonthSubtotal, SortInput};
use crate::vouchers::MAX_OPEN_ROWS;

/// ادھار رقم given, as typed (PRD 16.5): the person's account بنام, روکڑ جمع.
///
/// Field paths in errors: `date`, `person_account_id`, `amount`, `reason`, `return_by`, `narration`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MoneyLentInput {
    /// The day, `DD-MM-YYYY` or a shortcut. An open day, not after today.
    pub date: String,
    /// The person's ادھار رقم account (control 0103). Null gives `ERR_LENT_PERSON_REQUIRED`; an
    /// account of another kind gives `ERR_FORMAT_INVALID`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub person_account_id: Option<i64>,
    /// Whole rupees, more than 0.
    pub amount: String,
    /// Why. Required (`ERR_REASON_REQUIRED`).
    pub reason: String,
    /// The day it should come back, `DD-MM-YYYY`. Not before `date` (`ERR_RETURN_BY_BEFORE_ENTRY`,
    /// PRD FR-16.9). Example: lent on 15-10-26 with return by 10-10-26 is refused.
    #[serde(default)]
    pub return_by: Option<String>,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub narration: Option<String>,
    /// True after the user said yes to the cash box (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// How a loan is returned (PRD FR-16.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum LentReturnMode {
    /// نقد: cash comes into روکڑ.
    Cash,
    /// تنخواہ سے کٹوتی: deducted from salary. No cash moves; the salary head is a shop expense and
    /// is split between the partners.
    Salary,
}

/// ادھار رقم واپسی as typed (PRD 16.5, FR-16.4).
///
/// Field paths in errors: `date`, `person_account_id`, `amount`, `mode`, `salary_head_id`,
/// `narration`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MoneyLentReturnInput {
    /// The day, `DD-MM-YYYY` or a shortcut. An open day, not after today.
    pub date: String,
    /// The person. Null gives `ERR_LENT_PERSON_REQUIRED`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub person_account_id: Option<i64>,
    /// Whole rupees, more than 0. More than the open amount is `ERR_RETURN_EXCEEDS_OPEN` (value
    /// `open_paisa`, field `amount`).
    pub amount: String,
    /// Cash or salary.
    pub mode: LentReturnMode,
    /// The expense head (`masters_expense_heads`) for mode `salary`. Null means تنخواہ. Ignored for
    /// mode `cash`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub salary_head_id: Option<i64>,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub narration: Option<String>,
}

/// A new person who can be lent money (the `F4` of the screen, PRD 16.5): an account under control
/// 0103, named in both scripts.
///
/// Field paths in errors: `name_ur`, `name_en`, `phone`, `staff_id`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct LentPersonCreateInput {
    /// Urdu name (PRD FR-7.28).
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Phone, `03XXXXXXXXX` or `03XX-XXXXXXX`. Empty or null for none.
    #[serde(default)]
    pub phone: Option<String>,
    /// The staff record, if he is staff. `ERR_NOT_FOUND` on `staff_id`. The account is linked to it.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub staff_id: Option<i64>,
    /// Words the user confirmed in the spelling check (PRD FR-7.66).
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
}

/// A person who can be lent money.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LentPerson {
    /// His ادھار رقم account.
    pub account: RecordRef,
    /// His staff record, if he is staff.
    pub staff: Option<RecordRef>,
    /// His phone. Empty if none.
    pub phone: String,
}

/// Input of `lent_list`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct LentListInput {
    /// The person's account. `ERR_NOT_FOUND` on `person_id`.
    #[ts(type = "number")]
    pub person_id: i64,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives the default page of 100 rows (PRD FR-19.11).
    #[serde(default)]
    pub limit: u32,
    /// The column to sort the whole range by: `entry_date`, `number`, `amount_paisa`, `reason`, `return_by`, `returned_paisa`, `open_paisa` or `days_past_return_by`. Null is the natural order (plan 38
    /// R4). A sorted row still shows its true running figure of the natural order.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// True to give the subtotal of each month in `months`. Used only in date order.
    #[serde(default)]
    pub group_by_month: bool,
    /// True to list the loans that are fully returned too. False (the default) leaves them out
    /// of the rows, the count, the totals and the months: a fully returned one is finished (plan
    /// 38 R5). The totals are always over the rows listed.
    #[serde(default)]
    pub show_finished: bool,
}

/// One loan with what has been returned against it, oldest first (PRD FR-16.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LentRow {
    /// The voucher.
    #[ts(type = "number")]
    pub voucher_id: i64,
    /// Its number in the series ر-.
    #[ts(type = "number")]
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// What was lent.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// Why.
    pub reason: String,
    /// The return-by day, `YYYY-MM-DD`, or null.
    pub return_by: Option<String>,
    /// What oldest-first returns have settled on it.
    #[ts(type = "Paisa")]
    pub returned_paisa: Paisa,
    /// Lent less returned.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
    /// Days past `return_by` today, or null (no date, nothing open, or not yet due).
    #[ts(type = "number | null")]
    pub days_past_return_by: Option<i64>,
}

/// What a person's loans add up to, over the whole list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LentTotals {
    /// Σ lent.
    #[ts(type = "Paisa")]
    pub lent_paisa: Paisa,
    /// Σ returned, in cash and by salary.
    #[ts(type = "Paisa")]
    pub returned_paisa: Paisa,
    /// Σ still open.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
}

/// A person's loans, one page (PRD FR-16.5, FR-19.11). Read only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LentList {
    /// The person.
    pub person: LentPerson,
    /// His account balance now. It equals the open total.
    pub balance: Balance,
    /// The page.
    pub rows: Vec<LentRow>,
    /// How many loans he has in all.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over all his loans.
    pub totals: LentTotals,
    /// The count and subtotals of each month of the whole range, in the order of the list, when
    /// asked with `group_by_month` and the list is in date order. Figures: `lent_paisa`, `returned_paisa` and `open_paisa`.
    pub months: Vec<MonthSubtotal>,
}

/// Input of `lent_report` (PRD R32 data). A balance list takes an "as of" day (PRD FR-19.10).
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct LentReportInput {
    /// The day the balances are as of, `DD-MM-YYYY` or a shortcut. Null or empty means today.
    #[serde(default)]
    pub as_of: Option<String>,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives the default page of 100 rows.
    #[serde(default)]
    pub limit: u32,
    /// Only people whose code or name (either script) holds this text. The totals are over these
    /// people.
    #[serde(default)]
    pub search: Option<String>,
    /// The column to sort the whole report by: `person` (English name), `is_staff`, `lent_paisa`,
    /// `returned_cash_paisa`, `returned_salary_paisa`, `open_paisa`, `oldest_open_date`,
    /// `return_by` or `days_past_return_by`. Null puts people with something open first, the
    /// oldest open loan first (plan 38 R4).
    #[serde(default)]
    pub sort: Option<SortInput>,
}

/// One person on the report R32.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LentReportRow {
    /// The person's account.
    pub person: RecordRef,
    /// True if he has a staff record.
    pub is_staff: bool,
    /// Σ lent up to the day.
    #[ts(type = "Paisa")]
    pub lent_paisa: Paisa,
    /// Σ returned in cash.
    #[ts(type = "Paisa")]
    pub returned_cash_paisa: Paisa,
    /// Σ returned by salary deduction.
    #[ts(type = "Paisa")]
    pub returned_salary_paisa: Paisa,
    /// Still open.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
    /// The day of his oldest open loan, `YYYY-MM-DD`, or null.
    pub oldest_open_date: Option<String>,
    /// The return-by day of his oldest open loan that has one, `YYYY-MM-DD`, or null.
    pub return_by: Option<String>,
    /// Days past that return-by day as of the day, or null.
    #[ts(type = "number | null")]
    pub days_past_return_by: Option<i64>,
}

/// The totals of R32 over the whole filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LentReportTotals {
    /// Σ lent.
    #[ts(type = "Paisa")]
    pub lent_paisa: Paisa,
    /// Σ returned in cash.
    #[ts(type = "Paisa")]
    pub returned_cash_paisa: Paisa,
    /// Σ returned by salary.
    #[ts(type = "Paisa")]
    pub returned_salary_paisa: Paisa,
    /// Σ open.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
    /// Σ open of the people past their return-by day.
    #[ts(type = "Paisa")]
    pub overdue_paisa: Paisa,
}

/// R32: everyone who was lent money, one page (PRD 16.5, FR-19.11). Read only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LentReport {
    /// The day it is as of, `YYYY-MM-DD`.
    pub as_of: String,
    /// The page, most open first.
    pub rows: Vec<LentReportRow>,
    /// How many people there are in all.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over all the people.
    pub totals: LentReportTotals,
}

// ---- Attention: overdue loans and partner takes (PRD FR-7.44) ----

/// What is overdue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum OverdueKind {
    /// ادھار رقم past its return-by day.
    Loan,
    /// Money a partner took (ذاتی خرچہ) past its return-by day.
    Take,
}

/// One overdue item for the attention list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OverdueItem {
    /// Loan or take.
    pub kind: OverdueKind,
    /// The person (a loan) or the partner (a take).
    pub who: RecordRef,
    /// The voucher.
    #[ts(type = "number")]
    pub voucher_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub number: i64,
    /// The day it was given, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The return-by day, `YYYY-MM-DD`.
    pub return_by: String,
    /// Still open.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
    /// Days past the return-by day today. 1 or more.
    #[ts(type = "number")]
    pub days_past: i64,
}

/// The overdue loans and partner takes (the data of PRD FR-7.44 for these two; D5 builds the bell).
/// Read only; nothing is written (Code Standards B29). At most `MAX_OPEN_ROWS` (200) items, the
/// most overdue first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AttentionOverdue {
    /// The items, most days past first.
    pub items: Vec<OverdueItem>,
    /// How many items there are in all.
    #[ts(type = "number")]
    pub total_count: i64,
    /// True if `items` holds only the first `MAX_OPEN_ROWS` of them.
    pub truncated: bool,
}

/// The most items `assist_attention_overdue` gives.
pub const MAX_OVERDUE_ITEMS: usize = MAX_OPEN_ROWS;
