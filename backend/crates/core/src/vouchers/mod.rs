//! Vouchers (PRD 15, plan 10): the shapes the voucher screens send and read back, and the day
//! close. Only shapes and the rules that need no database; the posting is in `services`.
//!
//! Plan 10 builds the framework and the پیشگی voucher (giving an advance, PRD 15.4). Plan 15 adds
//! the وصولی and the ادائیگی ([`entry`]); the other voucher types come in later plans and use the
//! same table, numbering and list.

mod entry;

pub use entry::{
    AdvanceEffect, AdvancesReport, AdvancesReportInput, AdvancesReportRow, BillEffect,
    CollectionBuyer, CollectionGroup, CollectionList, CollectionListInput, LineSource,
    MAX_OPEN_ROWS, MadeRegular, MakeRegularInput, OpenBill, OpenBills, OpenBillsInput,
    OpenPayables, OpenPayablesInput, PayablePosting, PaymentInput, ReceiptInput, ReceiptRowResult,
    ReceiptsInput, ReceiptsSaved, VoucherCancelInput, VoucherCancelled, VoucherEditInput,
    VoucherEntry, VoucherGetInput, VoucherHit, VoucherPreview, VoucherPreviewInput,
};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::{Balance, LineError, RecordRef};
use crate::money::Paisa;

/// The kind of a voucher (`vouchers.type`, Architecture 15.7). Each kind has its own number
/// series, shown as و-1, ا-1, پ-1 in Urdu and R-1, P-1, A-1 in English (letters from Fluent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum VoucherType {
    /// وصولی: money from a buyer.
    Receipt,
    /// ادائیگی: money out.
    Payment,
    /// پیشگی: an advance to a seller.
    Advance,
    /// دکان خرچہ.
    ShopExpense,
    /// ذاتی خرچہ: money a partner takes.
    PersonalExpense,
    /// A partner gives money back.
    PersonalReturn,
    /// ادھار رقم given.
    MoneyLent,
    /// ادھار رقم returned.
    MoneyLentReturn,
    /// جنرل.
    General,
    /// سرمایہ جمع: a partner invests (PRD FR-17.7, plan 17).
    CapitalIn,
    /// سرمایہ واپسی: a partner takes his investment back.
    CapitalOut,
    /// الگ کھاتہ رقم دی: money paid out for a separate khata (PRD FR-16.6).
    KhataOut,
    /// الگ کھاتہ رقم آئی: money received for a separate khata.
    KhataIn,
}

impl VoucherType {
    /// The stored form.
    pub const fn as_str(self) -> &'static str {
        match self {
            VoucherType::Receipt => "receipt",
            VoucherType::Payment => "payment",
            VoucherType::Advance => "advance",
            VoucherType::ShopExpense => "shop_expense",
            VoucherType::PersonalExpense => "personal_expense",
            VoucherType::PersonalReturn => "personal_return",
            VoucherType::MoneyLent => "money_lent",
            VoucherType::MoneyLentReturn => "money_lent_return",
            VoucherType::General => "general",
            VoucherType::CapitalIn => "capital_in",
            VoucherType::CapitalOut => "capital_out",
            VoucherType::KhataOut => "khata_out",
            VoucherType::KhataIn => "khata_in",
        }
    }

    /// Reads the stored form.
    pub fn parse(text: &str) -> Option<VoucherType> {
        [
            VoucherType::Receipt,
            VoucherType::Payment,
            VoucherType::Advance,
            VoucherType::ShopExpense,
            VoucherType::PersonalExpense,
            VoucherType::PersonalReturn,
            VoucherType::MoneyLent,
            VoucherType::MoneyLentReturn,
            VoucherType::General,
            VoucherType::CapitalIn,
            VoucherType::CapitalOut,
            VoucherType::KhataOut,
            VoucherType::KhataIn,
        ]
        .into_iter()
        .find(|t| t.as_str() == text)
    }
}

/// How the money moves (PRD 15.1 "ذریعہ").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum VoucherMode {
    /// نقد: cash in hand (روکڑ).
    Cash,
    /// بینک: a bank account.
    Bank,
    /// چیک: a cheque on a bank account, posted to the bank at once (PRD A30).
    Cheque,
}

impl VoucherMode {
    /// The stored form.
    pub const fn as_str(self) -> &'static str {
        match self {
            VoucherMode::Cash => "cash",
            VoucherMode::Bank => "bank",
            VoucherMode::Cheque => "cheque",
        }
    }

    /// Reads the stored form (`salary` is not a mode of an advance).
    pub fn parse(text: &str) -> Option<VoucherMode> {
        match text {
            "cash" => Some(VoucherMode::Cash),
            "bank" => Some(VoucherMode::Bank),
            "cheque" => Some(VoucherMode::Cheque),
            _ => None,
        }
    }
}

/// The payment fields that no longer exist (PRD 15.1, v1.6: all money is روکڑ, one cash account,
/// no bank and no cheque). A voucher input still has a place for them only so an old client that
/// sends one is told, with a field error at the name it sent, instead of being posted to the bank:
/// `ERR_FIELD_NOT_ALLOWED`. They are not in the generated TypeScript.
///
/// ```
/// use mandi_core::vouchers::RemovedPaymentFields;
/// let sent: RemovedPaymentFields =
///     serde_json::from_str(r#"{"mode":"cash","cheque_no":"","bank_account_id":null}"#).unwrap();
/// assert_eq!(sent.names_sent(), vec!["mode"]);
/// ```
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RemovedPaymentFields {
    /// The old mode (`cash`, `bank` or `cheque`). Any value is refused.
    #[serde(default)]
    pub mode: Option<serde_json::Value>,
    /// The old bank account. Any value but null is refused.
    #[serde(default)]
    pub bank_account_id: Option<serde_json::Value>,
    /// The old cheque number. Refused unless it is null or empty text.
    #[serde(default)]
    pub cheque_no: Option<serde_json::Value>,
    /// The old cheque date. Refused unless it is null or empty text.
    #[serde(default)]
    pub cheque_date: Option<serde_json::Value>,
}

impl RemovedPaymentFields {
    /// The names of the removed fields that carry a value, in the order `mode`, `bank_account_id`,
    /// `cheque_no`, `cheque_date`. An empty text for a cheque number or date names no cheque
    /// (the old screens always sent them), so it is not counted.
    pub fn names_sent(&self) -> Vec<&'static str> {
        let blank = |v: &Option<serde_json::Value>| match v {
            None | Some(serde_json::Value::Null) => true,
            Some(serde_json::Value::String(text)) => text.trim().is_empty(),
            Some(_) => false,
        };
        let mut names = Vec::new();
        if !matches!(self.mode, None | Some(serde_json::Value::Null)) {
            names.push("mode");
        }
        if !matches!(self.bank_account_id, None | Some(serde_json::Value::Null)) {
            names.push("bank_account_id");
        }
        if !blank(&self.cheque_no) {
            names.push("cheque_no");
        }
        if !blank(&self.cheque_date) {
            names.push("cheque_date");
        }
        names
    }
}

/// A پیشگی voucher as typed (PRD 15.1, 15.4): money given to a seller before his goods are sold.
/// All values are text; the backend checks every one.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AdvanceInput {
    /// The day, `DD-MM-YYYY` or a shortcut. Must be an open day, not after today (PRD FR-6.6).
    pub date: String,
    /// The seller.
    #[ts(type = "number")]
    pub seller_account_id: i64,
    /// Whole rupees, more than 0.
    pub amount: String,
    /// The payment mode, bank and cheque fields were taken out in v1.6: all money is روکڑ
    /// (PRD 15.1). An old client that still sends one gets `ERR_FIELD_NOT_ALLOWED` at its name
    /// and nothing is posted. Not in the generated TypeScript.
    #[serde(flatten)]
    #[ts(skip)]
    pub removed: RemovedPaymentFields,
    /// The goods or season he is expected to send (PRD 15.4). Empty if none.
    pub expected_goods: String,
    /// A note, up to 500 characters. Empty if none.
    pub narration: String,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟". Needed only when the advance
    /// takes روکڑ below 0 (PRD FR-6.7); without it the save is refused with
    /// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` and nothing is written.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// A voucher saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct VoucherSaved {
    /// Row id.
    #[ts(type = "number")]
    pub voucher_id: i64,
    /// The kind.
    pub voucher_type: VoucherType,
    /// The number in its series, given at save and never used again.
    #[ts(type = "number")]
    pub number: i64,
    /// The journal entry of the posting.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// The seller's balance after this voucher.
    pub new_balance: Balance,
}

/// Several پیشگی vouchers in the running list (PRD FR-15.1).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AdvancesInput {
    /// The lines, in the order of the list.
    pub lines: Vec<AdvanceInput>,
}

/// One line of the running list, checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AdvancePreviewRow {
    /// The seller, or null if the id is wrong.
    pub account: Option<RecordRef>,
    /// What is wrong with the line (field paths `lines[i].<field>`). Empty if nothing.
    pub errors: Vec<LineError>,
    /// The amount, 0 if it could not be read.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// The seller's balance before this line, counting the lines above it for the same seller.
    pub current_balance: Balance,
    /// The seller's balance after this line.
    pub new_balance: Balance,
}

/// The running list checked, nothing saved (the same function as the save, Code Standards B5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AdvancesPreview {
    /// One row per line.
    pub rows: Vec<AdvancePreviewRow>,
    /// The total of the lines that have no error.
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
}

/// The result of one line when the list is saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AdvanceRowResult {
    /// The voucher, if the line was saved.
    pub saved: Option<VoucherSaved>,
    /// Why it was not saved (field paths `lines[i].<field>`). Empty if it was.
    pub errors: Vec<LineError>,
}

/// The list saved. Each voucher is its own posting event and its own transaction (PRD FR-15.1),
/// so a line that fails does not stop the others.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AdvancesSaved {
    /// One result per line, in the order sent.
    pub rows: Vec<AdvanceRowResult>,
    /// The total of the lines that were saved.
    #[ts(type = "Paisa")]
    pub saved_paisa: Paisa,
}

/// The vouchers of a kind in a date range.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct VouchersListInput {
    /// The kind, or null for all.
    #[serde(default)]
    pub voucher_type: Option<VoucherType>,
    /// From this day, `DD-MM-YYYY`, or empty for the start.
    pub from: String,
    /// To this day, `DD-MM-YYYY`, or empty for no end.
    pub to: String,
    /// Only this account (for a پختہ کھاتہ account, all its people unless `person_key` is set).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub account_id: Option<i64>,
    /// Only this one-time person.
    #[serde(default)]
    pub person_key: Option<String>,
    /// How many rows to skip. 0 for the first page (PRD FR-19.11).
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives a page of 100 (Code Standards B31).
    #[serde(default)]
    pub limit: u32,
}

/// One page of vouchers with the totals over the whole filter (PRD FR-15.1, FR-19.11, B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct VouchersList {
    /// This page, by day and number. Cancelled ones are in the list, marked by `status`.
    pub rows: Vec<VoucherRow>,
    /// How many vouchers match the filter, cancelled ones too.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over every matching voucher, never over the page.
    pub totals: VouchersTotals,
}

/// The totals of the voucher list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct VouchersTotals {
    /// How many of the matching vouchers are cancelled.
    #[ts(type = "number")]
    pub cancelled_count: i64,
    /// The total of the saved vouchers, cancelled ones left out. Null when the list is of all
    /// types: receipts bring money in and payments and advances send it out, so one sum of them
    /// means nothing.
    #[ts(type = "Paisa | null")]
    pub total_paisa: Option<Paisa>,
}

/// A voucher in the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct VoucherRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The kind.
    pub voucher_type: VoucherType,
    /// The number in its series.
    #[ts(type = "number")]
    pub number: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The account, if any.
    pub account: Option<RecordRef>,
    /// The amount.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// How the money moved.
    pub mode: VoucherMode,
    /// The bank account, if any (its bank name and number are in the masters).
    #[ts(type = "number | null")]
    pub bank_account_id: Option<i64>,
    /// The cheque number. Empty if none.
    pub cheque_no: String,
    /// The cheque date, `YYYY-MM-DD`, or null.
    pub cheque_date: Option<String>,
    /// The expected goods of an advance. Empty if none.
    pub expected_goods: String,
    /// The note. Empty if none.
    pub narration: String,
    /// `saved` or `cancelled`.
    pub status: String,
    /// The `version` of the row, to send back with an edit or a cancel.
    #[ts(type = "number")]
    pub version: i64,
    /// The one-time person, for a voucher on a پختہ کھاتہ account.
    pub person: Option<crate::bikri::SavedPukhta>,
    /// The collector of a وصولی (a staff member), if any.
    #[ts(type = "number | null")]
    pub collector_staff_id: Option<i64>,
    /// Why, for a payment to a buyer. Empty if none.
    pub reason: String,
    /// Why it was cancelled. Empty if it was not.
    pub cancel_reason: String,
    /// The journal entry of the live posting. For a cancelled voucher, its last posting.
    #[ts(type = "number | null")]
    pub entry_id: Option<i64>,
    /// True if the voucher is saved and its day is still open, so it can be edited or cancelled
    /// (PRD FR-15.3, FR-6.6).
    pub editable: bool,
}

/// Ask for a seller's advances (PRD FR-7.60).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AssistAdvancesInput {
    /// The seller.
    #[ts(type = "number")]
    pub seller_id: i64,
}

/// One advance that is not yet fully adjusted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OpenAdvance {
    /// `voucher` for a پیشگی voucher, `opening` for an opening debit marked as an advance.
    pub source: String,
    /// The voucher number, for source `voucher`.
    #[ts(type = "number | null")]
    pub voucher_number: Option<i64>,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// What was given.
    #[ts(type = "Paisa")]
    pub given_paisa: Paisa,
    /// What has been adjusted against it so far.
    #[ts(type = "Paisa")]
    pub adjusted_paisa: Paisa,
    /// Given less adjusted. More than 0.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
}

/// The last advance given to a seller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LastAdvance {
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The amount.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// The voucher number.
    #[ts(type = "number")]
    pub voucher_number: i64,
}

/// A seller's advances for the card in the پیشگی screen (PRD FR-7.60). Read only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AdvancesInfo {
    /// The open advances, oldest first.
    pub open: Vec<OpenAdvance>,
    /// Their total: given less adjusted.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
    /// The last پیشگی voucher, or null if there is none.
    pub last: Option<LastAdvance>,
}

// ---- Day close and reopen (PRD FR-21.1, FR-21.2) ----

/// What closing the days up to a day would do.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DayClosePreviewInput {
    /// The last day to close, `DD-MM-YYYY`, or empty for the oldest open day.
    pub up_to: String,
}

/// The summary shown before a day is closed (PRD FR-21.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DayClosePreview {
    /// The first day that would be closed, `YYYY-MM-DD`.
    pub from_date: String,
    /// The last day that would be closed, `YYYY-MM-DD`.
    pub up_to: String,
    /// How many days that is.
    pub days: u32,
    /// The debit total of the trial balance up to the last day.
    #[ts(type = "Paisa")]
    pub trial_dr_paisa: Paisa,
    /// The credit total of the trial balance up to the last day.
    #[ts(type = "Paisa")]
    pub trial_cr_paisa: Paisa,
    /// Debit less credit. It must be zero for the day to close.
    pub trial_difference: Balance,
    /// Cash in hand (روکڑ) at the end of the last day.
    pub cash_in_hand: Balance,
    /// How many journal entries are dated in these days.
    #[ts(type = "number")]
    pub entries: i64,
    /// The full summary of the days that would be closed (PRD FR-21.1, 4.9).
    pub summary: DayCloseSummary,
}

/// One partner's share of the days that would be closed (PRD FR-21.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DayClosePartner {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His income share (commission, لاگا, فون, ریٹ فرق).
    #[ts(type = "Paisa")]
    pub income_paisa: Paisa,
    /// His expense share as a positive number.
    #[ts(type = "Paisa")]
    pub expense_paisa: Paisa,
    /// Income less expense (PRD C12).
    #[ts(type = "Paisa")]
    pub profit_paisa: Paisa,
}

/// What the days to close came to (PRD FR-21.1, the values of PRD 4.9). Every number comes from
/// stored records. For the worked day 10-10-26: 2 بکری, خام 41,400, commission 5,448, shared
/// income 5,488, shares 996 / 746 / 746, no lots left.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DayCloseSummary {
    /// How many بکری were saved in these days (cancelled ones left out).
    #[ts(type = "number")]
    pub bikri_count: i64,
    /// Σ خام بکری.
    #[ts(type = "Paisa")]
    pub gross_paisa: Paisa,
    /// Seller commission + buyer commission + لاگا.
    #[ts(type = "Paisa")]
    pub commission_paisa: Paisa,
    /// Shared income: commission + لاگا + فون + ریٹ فرق.
    #[ts(type = "Paisa")]
    pub shared_income_paisa: Paisa,
    /// Each active partner's share, in the order the partners were created.
    pub partners: Vec<DayClosePartner>,
    /// Lots with a balance left in باقی مال at the end of the last day.
    #[ts(type = "number")]
    pub open_lots: i64,
    /// Their نگ.
    #[ts(type = "number")]
    pub open_qty: i64,
}

impl DayCloseSummary {
    /// The summary of no days at all: every number 0 and no partners.
    pub fn empty() -> DayCloseSummary {
        DayCloseSummary {
            bikri_count: 0,
            gross_paisa: Paisa::ZERO,
            commission_paisa: Paisa::ZERO,
            shared_income_paisa: Paisa::ZERO,
            partners: Vec::new(),
            open_lots: 0,
            open_qty: 0,
        }
    }
}

/// Close the days up to a day.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DayCloseInput {
    /// The last day to close, `DD-MM-YYYY`, or empty for the oldest open day.
    pub up_to: String,
}

/// Reopen a day and every later day (PRD FR-21.2).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DayReopenInput {
    /// The first day to reopen, `DD-MM-YYYY`. It and every later day become open.
    pub date: String,
    /// Why. Required.
    pub reason: String,
}
