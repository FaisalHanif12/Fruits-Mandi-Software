//! Vouchers (PRD 15, plan 10): the shapes the voucher screens send and read back, and the day
//! close. Only shapes and the rules that need no database; the posting is in `services`.
//!
//! Plan 10 builds the framework and the پیشگی voucher (giving an advance, PRD 15.4). The other
//! voucher types come in later plans and use the same table, numbering and list.

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
    /// How the money moves.
    pub mode: VoucherMode,
    /// The bank account, for mode `bank` and `cheque`. Null for cash.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub bank_account_id: Option<i64>,
    /// The cheque number, for mode `cheque`. Empty otherwise.
    pub cheque_no: String,
    /// The cheque date, `DD-MM-YYYY`, for mode `cheque`. Empty otherwise.
    pub cheque_date: String,
    /// The goods or season he is expected to send (PRD 15.4). Empty if none.
    pub expected_goods: String,
    /// A note, up to 500 characters. Empty if none.
    pub narration: String,
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
