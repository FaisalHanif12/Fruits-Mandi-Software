//! The inputs and answers of the voucher screen's three types (plan 15, PRD 15.1 to 15.4):
//! وصولی (receipt), ادائیگی (payment) and, from plan 10, پیشگی (advance).
//!
//! All inputs are text, as typed. The backend checks every value (Code Standards V0). Each type
//! keeps its own input, its own number series and its own posting (PRD FR-7.67); the preview, the
//! edit and the cancel take any of the three through [`VoucherEntry`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{AdvanceInput, OpenAdvance, VoucherMode, VoucherSaved, VoucherType};
use crate::accounts::{Balance, LineError, RecordRef};
use crate::capital::CapitalInput;
use crate::cash::CashCheck;
use crate::error::FieldError;
use crate::expenses::{PersonalExpenseInput, PersonalReturnInput, ShopExpenseInput};
use crate::general::GeneralInput;
use crate::khata::{KhataInInput, KhataOutInput};
use crate::lent::{MoneyLentInput, MoneyLentReturnInput};
use crate::money::Paisa;

/// A وصولی as typed (PRD 15.2, FR-15.12): money from a buyer, or a seller returning advance
/// money.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ReceiptInput {
    /// The day, `DD-MM-YYYY` or a shortcut. Must be an open day, not after today (PRD FR-6.6).
    pub date: String,
    /// The buyer or the seller.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The one-time person, for a پختہ کھاتہ account (PRD FR-13.5). Null for any other account.
    #[serde(default)]
    pub person_key: Option<String>,
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
    /// The collector (اگراہی والا), a staff member. Null if none.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub collector_staff_id: Option<i64>,
    /// A note, up to 500 characters. Empty if none.
    pub narration: String,
}

/// An ادائیگی as typed (PRD 15.3): money to a seller, to a buyer (cash lent, PRD FR-15.11) or to
/// a payable account such as the transporters (PRD FR-14.2).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PaymentInput {
    /// The day, `DD-MM-YYYY` or a shortcut. Must be an open day, not after today (PRD FR-6.6).
    pub date: String,
    /// The account paid.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The one-time person, for a پختہ کھاتہ account (PRD FR-13.5). Null for any other account.
    #[serde(default)]
    pub person_key: Option<String>,
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
    /// Why. Required when the account is a buyer (cash lent, PRD FR-15.11). Empty otherwise.
    pub reason: String,
    /// A note, up to 500 characters. Empty if none.
    pub narration: String,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟". Needed only when the payment
    /// takes روکڑ below 0 (PRD FR-6.7); without it the save is refused with
    /// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` and nothing is written.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// Any one voucher type, for the preview and the edit (PRD FR-7.67). Plan 17 adds the nine other
/// kinds; each is also saved by its own command.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VoucherEntry {
    /// وصولی.
    Receipt(ReceiptInput),
    /// ادائیگی.
    Payment(PaymentInput),
    /// پیشگی.
    Advance(AdvanceInput),
    /// دکان خرچہ (plan 17).
    ShopExpense(ShopExpenseInput),
    /// ذاتی خرچہ.
    PersonalExpense(PersonalExpenseInput),
    /// ذاتی خرچہ واپسی.
    PersonalReturn(PersonalReturnInput),
    /// ادھار رقم دی.
    MoneyLent(MoneyLentInput),
    /// ادھار رقم واپسی.
    MoneyLentReturn(MoneyLentReturnInput),
    /// سرمایہ جمع.
    CapitalIn(CapitalInput),
    /// سرمایہ واپسی.
    CapitalOut(CapitalInput),
    /// الگ کھاتہ رقم دی.
    KhataOut(KhataOutInput),
    /// الگ کھاتہ رقم آئی.
    KhataIn(KhataInInput),
    /// جنرل.
    General(GeneralInput),
}

impl VoucherEntry {
    /// The kind of voucher this entry is.
    pub const fn voucher_type(&self) -> VoucherType {
        match self {
            VoucherEntry::Receipt(_) => VoucherType::Receipt,
            VoucherEntry::Payment(_) => VoucherType::Payment,
            VoucherEntry::Advance(_) => VoucherType::Advance,
            VoucherEntry::ShopExpense(_) => VoucherType::ShopExpense,
            VoucherEntry::PersonalExpense(_) => VoucherType::PersonalExpense,
            VoucherEntry::PersonalReturn(_) => VoucherType::PersonalReturn,
            VoucherEntry::MoneyLent(_) => VoucherType::MoneyLent,
            VoucherEntry::MoneyLentReturn(_) => VoucherType::MoneyLentReturn,
            VoucherEntry::CapitalIn(_) => VoucherType::CapitalIn,
            VoucherEntry::CapitalOut(_) => VoucherType::CapitalOut,
            VoucherEntry::KhataOut(_) => VoucherType::KhataOut,
            VoucherEntry::KhataIn(_) => VoucherType::KhataIn,
            VoucherEntry::General(_) => VoucherType::General,
        }
    }
}

/// Input of `vouchers_preview`: the entry as typed, worked out with the same function as the
/// save (Code Standards B5). Nothing is written.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct VoucherPreviewInput {
    /// The entry as typed.
    pub entry: VoucherEntry,
    /// Set when an existing voucher is being edited: the preview then works as if that voucher's
    /// posting were taken back first, so the balances, the bills it settles and the cash note are
    /// those of the edit (PRD FR-15.3).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub replacing_voucher_id: Option<i64>,
}

/// Where a bill, an advance or a payable posting came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum LineSource {
    /// A بکری. `number` is the بکری number.
    Bikri,
    /// An opening balance (PRD FR-10.9).
    Opening,
    /// An ادائیگی to a buyer, cash lent to him (PRD FR-15.11). `number` is the voucher number.
    CashLent,
    /// A وصولی from a seller. `number` is the voucher number.
    Receipt,
    /// A جنرل entry.
    General,
    /// The firm's own purchase on credit (PRD FR-12.24).
    Purchase,
    /// Anything else.
    Other,
}

/// One open bill of a buyer (PRD FR-7.58): a debit line not yet fully settled by receipts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OpenBill {
    /// The journal line.
    #[ts(type = "number")]
    pub line_id: i64,
    /// The day of the bill, `YYYY-MM-DD`.
    pub entry_date: String,
    /// Where it came from.
    pub source: LineSource,
    /// The بکری number (source `bikri`) or the voucher number (source `cash_lent`).
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The whole bill.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// What is still open on it.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
    /// Its age: today less the day of the bill (PRD C14).
    #[ts(type = "number")]
    pub days: i64,
}

/// What a receipt does to one bill (PRD FR-7.58: "the list marks which bills it will settle").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BillEffect {
    /// The bill, with what is open on it now.
    pub bill: OpenBill,
    /// What this voucher settles on it. Normally 0 or more. It is below 0 only when a receipt
    /// dated before later receipts moves one of their payments to a later bill.
    #[ts(type = "Paisa")]
    pub settled_paisa: Paisa,
    /// What is open on it after this voucher.
    #[ts(type = "Paisa")]
    pub still_open_paisa: Paisa,
}

/// What a seller's receipt does to one open advance (PRD FR-15.12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AdvanceEffect {
    /// The advance, with what is open on it now.
    pub advance: OpenAdvance,
    /// What this receipt adjusts on it.
    #[ts(type = "Paisa")]
    pub adjusted_paisa: Paisa,
    /// What is open on it after this receipt.
    #[ts(type = "Paisa")]
    pub still_open_paisa: Paisa,
}

/// The entry checked and worked out, nothing saved (PRD FR-7.58, FR-7.59, FR-6.7, FR-7.68).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct VoucherPreview {
    /// The account, or null if the id is wrong.
    pub account: Option<RecordRef>,
    /// The amount, 0 if it could not be read.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// The account's balance (or the person's, for a پختہ کھاتہ) before this voucher.
    pub current_balance: Balance,
    /// The same balance after it.
    pub new_balance: Balance,
    /// Receipt from a buyer: the bills this amount settles, oldest first.
    pub bill_effects: Vec<BillEffect>,
    /// Receipt from a buyer: the part of the amount that is more than all his open bills. It
    /// stays as his credit and settles his next bill (PRD FR-15.5).
    #[ts(type = "Paisa")]
    pub extra_credit_paisa: Paisa,
    /// Receipt from a seller: the advances this amount adjusts, oldest first.
    pub advance_effects: Vec<AdvanceEffect>,
    /// Receipt from a seller: the part of the amount that settles what he owes the firm beyond
    /// his advances.
    #[ts(type = "Paisa")]
    pub owes_settled_paisa: Paisa,
    /// Receipt from a seller: the most he can return (open advances plus what he owes beyond
    /// them). Null for any other voucher.
    #[ts(type = "Paisa | null")]
    pub return_limit_paisa: Option<Paisa>,
    /// Payment to a seller: what is payable to him now (PRD FR-7.59). Null for any other voucher.
    #[ts(type = "Paisa | null")]
    pub payable_open_paisa: Option<Paisa>,
    /// True for a payment to a seller who has nothing payable: the screen offers to make it a
    /// پیشگی (PRD FR-7.68). Never switches by itself.
    pub payable_is_zero_for_seller: bool,
    /// Cash in hand after this voucher (PRD FR-6.7). `needs_confirm` is true when the save will
    /// need `confirm_cash_below_zero`. For a voucher that cannot lower cash (a receipt, in cash
    /// or by bank) the balance is not read, which is the slowest part of a preview: then
    /// `cash_before_paisa` and `cash_after_paisa` are 0 and `lowers_cash`, `below_zero` and
    /// `needs_confirm` are false. The screen shows the amounts only when `needs_confirm` is true.
    pub cash: CashCheck,
    /// What stops a save. The screen shows each at its field.
    pub errors: Vec<FieldError>,
    /// What only warns. A save is allowed. For example `ERR_PAYMENT_ABOVE_PAYABLE` (PRD FR-15.7).
    pub warnings: Vec<FieldError>,
}

/// Input of `vouchers_get`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct VoucherGetInput {
    /// The voucher.
    #[ts(type = "number")]
    pub id: i64,
}

/// Input of `vouchers_edit` (PRD FR-15.3): the voucher is taken back by a reversal and the new
/// version is posted, in one transaction. The type of a voucher cannot change.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct VoucherEditInput {
    /// The voucher.
    #[ts(type = "number")]
    pub id: i64,
    /// The `version` read with the voucher. A different one is `ERR_VERSION_CHANGED`.
    #[ts(type = "number")]
    pub version: i64,
    /// The new values. Its type must be the voucher's type.
    pub entry: VoucherEntry,
    /// Why, for the audit log. Empty if none.
    pub reason: String,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟" (PRD FR-6.7). Needed when the edit
    /// takes cash in hand below 0, for example a smaller receipt.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// Input of `vouchers_cancel` (PRD FR-15.3): the voucher is taken back by a reversal and marked
/// منسوخ. It keeps its number.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct VoucherCancelInput {
    /// The voucher.
    #[ts(type = "number")]
    pub id: i64,
    /// The `version` read with the voucher. A different one is `ERR_VERSION_CHANGED`.
    #[ts(type = "number")]
    pub version: i64,
    /// Why. Required (`ERR_REASON_REQUIRED`).
    pub reason: String,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟" (PRD FR-6.7). Needed when taking
    /// the voucher back lowers cash below 0, for example cancelling a receipt.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// A voucher cancelled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct VoucherCancelled {
    /// Row id.
    #[ts(type = "number")]
    pub voucher_id: i64,
    /// The kind.
    pub voucher_type: VoucherType,
    /// Its number, which is never used again.
    #[ts(type = "number")]
    pub number: i64,
    /// The reversal entry.
    #[ts(type = "number")]
    pub reversal_entry_id: i64,
    /// The account's balance (or the person's) after the cancel.
    pub new_balance: Balance,
}

// ---- The open bills of a buyer (PRD FR-7.58) ----

/// Input of `assist_open_bills`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct OpenBillsInput {
    /// The buyer.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The one-time person, for the پختہ کھاتہ buyer account.
    #[serde(default)]
    pub person_key: Option<String>,
}

/// The most open bills or payable postings a read gives (PRD FR-7.58, FR-7.59). A party can have
/// tens of thousands of them; the screen shows the oldest, which payments settle first, and says
/// how many there are in all.
pub const MAX_OPEN_ROWS: usize = 200;

/// A buyer's open bills, oldest first (PRD FR-7.58). Read only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OpenBills {
    /// The buyer.
    pub account: RecordRef,
    /// The oldest open bills, oldest first, at most `MAX_OPEN_ROWS` (200).
    pub bills: Vec<OpenBill>,
    /// How many open bills he has in all. The screen says "first 200 of N" when `truncated`.
    #[ts(type = "number")]
    pub total_count: i64,
    /// True if `bills` holds only the oldest of them (`total_count` is more than its length).
    pub truncated: bool,
    /// Σ of what is open on ALL his open bills, not only on `bills`. The "پورا بقایا" button fills
    /// this amount. It equals his balance when he has no credit waiting for a bill.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
    /// Credit that no bill has used yet (a receipt more than his bills, PRD FR-15.5).
    #[ts(type = "Paisa")]
    pub unmatched_credit_paisa: Paisa,
    /// His balance now.
    pub balance: Balance,
    /// The age in days of the oldest open bill, or null if there is none.
    #[ts(type = "number | null")]
    pub oldest_days: Option<i64>,
}

// ---- What is payable to a seller (PRD FR-7.59) ----

/// Input of `assist_open_payables`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct OpenPayablesInput {
    /// The seller.
    #[ts(type = "number")]
    pub seller_id: i64,
    /// The one-time person, for the پختہ کھاتہ بیوپاری account.
    #[serde(default)]
    pub person_key: Option<String>,
    /// True lists the postings with nothing left to pay too (for a report). Missing means false.
    #[serde(default)]
    pub include_settled: bool,
}

/// One payable posting of a seller, worked out when shown (PRD FR-7.59). Nothing is stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PayablePosting {
    /// The day of the posting, `YYYY-MM-DD`.
    pub entry_date: String,
    /// Where it came from: a بکری, his opening credit, a جنرل credit, a purchase on credit.
    pub source: LineSource,
    /// The بکری number, for source `bikri`.
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The credit of the posting (his صافی بکری, C8).
    #[ts(type = "Paisa")]
    pub credit_paisa: Paisa,
    /// What his advances took from it (PRD FR-15.8).
    #[ts(type = "Paisa")]
    pub advance_adjusted_paisa: Paisa,
    /// What has been paid against it: ادائیگی, نقد and جنرل debits, oldest first.
    #[ts(type = "Paisa")]
    pub paid_paisa: Paisa,
    /// What is still to pay on it.
    #[ts(type = "Paisa")]
    pub still_open_paisa: Paisa,
    /// Its age: today less the day of the posting.
    #[ts(type = "number")]
    pub days: i64,
}

/// What is payable to a seller (PRD FR-7.59). Read only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OpenPayables {
    /// The seller.
    pub account: RecordRef,
    /// The payable postings, oldest first. Only those with something to pay, unless
    /// `include_settled` was set. Without `include_settled` at most `MAX_OPEN_ROWS` (200), the
    /// oldest; with it, all of them.
    pub postings: Vec<PayablePosting>,
    /// How many postings have something to pay, in all (with `include_settled`, how many there
    /// are in all). The screen says "first 200 of N" when `truncated`.
    #[ts(type = "number")]
    pub total_count: i64,
    /// True if `postings` holds only the oldest of them.
    pub truncated: bool,
    /// Σ still to pay on ALL his postings, not only on `postings`. The "پورا بقایا" button fills
    /// this amount.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
    /// His open advances (PRD FR-15.8). They are not taken off `open_paisa`.
    #[ts(type = "Paisa")]
    pub advances_open_paisa: Paisa,
    /// What he owes the firm beyond his advances: payments above his payable (PRD FR-15.7,
    /// FR-15.9, "بیوپاری کے ذمے").
    #[ts(type = "Paisa")]
    pub owes_firm_paisa: Paisa,
    /// His balance now. When `open_paisa` is above 0 it equals `open_paisa` less
    /// `advances_open_paisa`.
    pub balance: Balance,
}

// ---- The advances report (PRD R26 data) ----

/// Input of `advances_report`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AdvancesReportInput {
    /// One seller, or null for all sellers who were ever given an advance.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub seller_id: Option<i64>,
    /// The "as of" day, `DD-MM-YYYY` or a shortcut. Empty means today.
    pub as_of: String,
}

/// One seller's advances (PRD R26): given, returned, adjusted and open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AdvancesReportRow {
    /// The seller.
    pub account: RecordRef,
    /// Σ of his advances: پیشگی vouchers and opening debits marked as advances.
    #[ts(type = "Paisa")]
    pub given_paisa: Paisa,
    /// What he gave back by a وصولی on his account (PRD FR-15.12).
    #[ts(type = "Paisa")]
    pub returned_paisa: Paisa,
    /// What his sales adjusted (PRD FR-15.8).
    #[ts(type = "Paisa")]
    pub adjusted_paisa: Paisa,
    /// Given less returned less adjusted.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
    /// The day of his last advance, `YYYY-MM-DD`.
    pub last_date: Option<String>,
}

/// The advances report (PRD R26 data): one row per seller and the totals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AdvancesReport {
    /// The day the report is as of, `YYYY-MM-DD`.
    pub as_of: String,
    /// By seller code.
    pub rows: Vec<AdvancesReportRow>,
    /// Σ given.
    #[ts(type = "Paisa")]
    pub given_paisa: Paisa,
    /// Σ returned.
    #[ts(type = "Paisa")]
    pub returned_paisa: Paisa,
    /// Σ adjusted.
    #[ts(type = "Paisa")]
    pub adjusted_paisa: Paisa,
    /// Σ open.
    #[ts(type = "Paisa")]
    pub open_paisa: Paisa,
}

// ---- The collection list (PRD FR-15.6, FR-7.61) ----

/// Input of `assist_collection_list`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CollectionListInput {
    /// The day, `DD-MM-YYYY` or a shortcut. Buyers whose collection day is this weekday are
    /// planned.
    pub date: String,
    /// Only this collector.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub collector_staff_id: Option<i64>,
    /// Only this city.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub city_id: Option<i64>,
}

/// A buyer on the collection list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionBuyer {
    /// The buyer.
    pub account: RecordRef,
    /// His phone as typed. Empty if none.
    pub phone: String,
    /// What he owes (his debit balance).
    #[ts(type = "Paisa")]
    pub dues_paisa: Paisa,
    /// The day of his oldest open bill, `YYYY-MM-DD`.
    pub oldest_bill_date: Option<String>,
    /// Its age in days.
    #[ts(type = "number | null")]
    pub oldest_days: Option<i64>,
}

/// Buyers of one collector in one city.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionGroup {
    /// The collector (a staff member). Null for buyers with no collector.
    pub collector: Option<RecordRef>,
    /// The city. Null for buyers with no city.
    pub city: Option<RecordRef>,
    /// The buyers, by code.
    pub buyers: Vec<CollectionBuyer>,
    /// Σ of their dues.
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
}

/// Today's planned collections (PRD FR-15.6). Read only; printable by the screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionList {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// Its weekday, 0 (Monday) to 6 (Sunday), as stored on the account.
    #[ts(type = "number")]
    pub weekday: u32,
    /// By collector, then city.
    pub groups: Vec<CollectionGroup>,
    /// Σ of all dues on the list.
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
}

/// Input of `vouchers_save_receipts_batch` (PRD FR-7.61): the receipts of the collection round.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ReceiptsInput {
    /// The receipts, in the order of the list. A row with amount 0 or empty is skipped.
    pub rows: Vec<ReceiptInput>,
}

/// The result of one row of the collection round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ReceiptRowResult {
    /// The voucher, if the row was saved.
    pub saved: Option<VoucherSaved>,
    /// True if the row was left out because its amount is 0 or empty.
    pub skipped: bool,
    /// Why it was not saved (field paths `rows[i].<field>`). Empty if it was or was skipped.
    pub errors: Vec<LineError>,
}

/// The collection round saved. Each receipt is its own posting event and its own transaction
/// (PRD FR-15.1), so a row that fails does not stop the others.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ReceiptsSaved {
    /// One result per row, in the order sent.
    pub rows: Vec<ReceiptRowResult>,
    /// The total of the rows that were saved.
    #[ts(type = "Paisa")]
    pub saved_paisa: Paisa,
}

// ---- Make a one-time person a regular account (PRD FR-13.8) ----

/// Input of `pukhta_make_regular`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MakeRegularInput {
    /// The one-time person.
    pub person_key: String,
    /// His side: a one-time buyer or seller. The account must be of the same side.
    pub side: crate::bikri::PukhtaSide,
    /// The new account, filled as in the account form (a quick add with `F4`, PRD FR-10.8). The
    /// id and version must be null.
    pub account_input: crate::accounts::AccountInput,
}

/// A one-time person made regular.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MadeRegular {
    /// The new account.
    pub account: RecordRef,
    /// The balance that was moved to it with a جنرل entry. Zero if he owed nothing and was owed
    /// nothing.
    pub moved: Balance,
    /// The جنرل entry. Null if nothing was moved.
    #[ts(type = "number | null")]
    pub entry_id: Option<i64>,
}

// ---- Global search (PRD FR-5.4) ----

/// A voucher found by `search_global` by its number. Opening it opens the voucher screen on this
/// id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct VoucherHit {
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
    /// `saved` or `cancelled`.
    pub status: String,
}
