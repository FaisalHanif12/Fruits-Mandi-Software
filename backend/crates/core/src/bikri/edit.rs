//! Goods left to sell, later sales, edit, cancel, undo and write-off (plan 16 contract; PRD 12.8,
//! 12.9, FR-7.47). Money is paisa. Names are `RecordRef` with both scripts, so the screen shows
//! them through `displayName` (Code Standards F38).
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `bikri_stock_list` | `StockListInput` | `StockList` |
//! | `bikri_later_sale_preview` | `LaterSaleInput` | `BikriPreview` |
//! | `bikri_later_sale_save` | `LaterSaleSaveInput` | `LaterSaleSaved` |
//! | `bikri_open_for_edit` | `EditOpenInput` | `EditableBikri` |
//! | `bikri_edit_preview` | `EditPreviewInput` | `BikriPreview` |
//! | `bikri_edit_save` | `EditSaveInput` | `BikriEdited` |
//! | `bikri_cancel_preview` | `CancelPreviewInput` | `CancelPreview` |
//! | `bikri_cancel` | `CancelInput` | `BikriCancelled` |
//! | `bikri_undo` | `UndoInput` | `UndoResult` |
//! | `bikri_write_off` | `WriteOffInput` | `WriteOffDone` |
//! | `bikri_write_off_undo` | `WriteOffUndoInput` | `WriteOffUndone` |
//!
//! Field paths in the errors of the later sale and of an edit are the ones of the book screen
//! (`lots[0].sales[2].qty`). In a later sale there is one lot, so the path starts `lots[0]`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::input::{BikriDraft, SaleInput};
use super::preview::BikriPreview;
use super::saved::{BikriStatus, SavedLot, SavedPosting, SavedPukhta};
use crate::accounts::RecordRef;
use crate::cash::CashCheck;

/// The reason an undo (PRD FR-7.47) writes to the audit log and to `cancel_reason`. It is a Fluent
/// message ID, not text: the screen turns it into "محفوظ کرنے کے فوراً بعد واپس" or "Undone straight
/// after saving" (CLAUDE.md rule 7).
pub const UNDO_REASON_ID: &str = "bikri-undo-reason";

// ---- Goods left to sell (باقی مال, PRD 12.8) ----

/// Input of `bikri_stock_list` (PRD FR-12.14, FR-19.10). Dates are `DD-MM-YYYY` or a shortcut;
/// null or empty means no limit. `date` is one day and wins over `from` and `to`. They filter on
/// the day the goods were received.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct StockListInput {
    /// One day.
    #[serde(default)]
    pub date: Option<String>,
    /// First day.
    #[serde(default)]
    pub from: Option<String>,
    /// Last day.
    #[serde(default)]
    pub to: Option<String>,
    /// Only lots of this seller (the party filter of PRD FR-19.10).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub seller_id: Option<i64>,
    /// Only lots of this item.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// Also list a lot that has a write-off you can undo even when nothing is left of it, so the
    /// undo can be found. Its `balance` is then 0.
    #[serde(default)]
    pub include_written_off: bool,
}

/// A write-off of a lot that has not been undone (PRD FR-12.16).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct WriteOffRow {
    /// The stock move. `bikri_write_off_undo` takes this id.
    #[ts(type = "number")]
    pub move_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub move_date: String,
    /// How many نگ were written off (more than 0).
    #[ts(type = "number")]
    pub qty: i64,
    /// The reason given. Free text.
    pub reason: Option<String>,
}

/// One lot with goods left (PRD FR-12.14): date received, بکری number, seller, item, packing,
/// received, sold, written off, balance, days waiting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StockRow {
    /// The lot. `bikri_later_sale_*` and `bikri_write_off` take this id.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// The lot number inside its بکری, from 1.
    #[ts(type = "number")]
    pub lot_no: i64,
    /// Its بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// The بکری number.
    #[ts(type = "number")]
    pub bikri_number: i64,
    /// The day the goods were received (the day of the first posting), `YYYY-MM-DD`.
    pub received_date: String,
    /// The seller account.
    pub seller: RecordRef,
    /// The one-time seller, for the پختہ کھاتہ account.
    pub pukhta: Option<SavedPukhta>,
    /// The item.
    pub item: RecordRef,
    /// The packing.
    pub packing: RecordRef,
    /// نگ received.
    #[ts(type = "number")]
    pub qty_received: i64,
    /// نگ sold, over all postings in force.
    #[ts(type = "number")]
    pub qty_sold: i64,
    /// نگ written off.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// نگ left: received less sold less written off.
    #[ts(type = "number")]
    pub balance: i64,
    /// Whole days from `received_date` to today.
    #[ts(type = "number")]
    pub days_waiting: i64,
    /// The write-offs of this lot that can still be undone, oldest first.
    pub write_offs: Vec<WriteOffRow>,
}

/// The answer of `bikri_stock_list`. Own goods rows join in plan 16b.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct StockList {
    /// Oldest received first, then by بکری number and lot number.
    pub rows: Vec<StockRow>,
    /// Σ of the `balance` of the rows (نگ).
    #[ts(type = "number")]
    pub balance_total: i64,
}

// ---- Later sale (PRD FR-12.17) ----

/// Input of `bikri_later_sale_preview`: new sale lines for the goods left in one lot, dated
/// `date`. Nothing is saved. Each line carries the screen's key, so the answer can be matched.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct LaterSaleInput {
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// The day of the sale, `DD-MM-YYYY` or a shortcut. An open day (PRD FR-6.6), not before the
    /// goods were received.
    pub date: String,
    /// The new lines. A line sent with `sale_id` null is a new line.
    #[serde(default)]
    pub sales: Vec<SaleInput>,
}

/// Input of `bikri_later_sale_save` (PRD FR-17, FR-6.7).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct LaterSaleSaveInput {
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// The day of the sale.
    pub date: String,
    /// The new lines.
    #[serde(default)]
    pub sales: Vec<SaleInput>,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟" (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// What `bikri_later_sale_save` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LaterSaleSaved {
    /// The بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number (the same as before: a later sale does not make a new number).
    #[ts(type = "number")]
    pub number: i64,
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// The journal entry of the new posting.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// The numbers the save wrote; the same as `bikri_later_sale_preview` gave just before.
    pub preview: BikriPreview,
}

// ---- Edit (PRD FR-12.18 to FR-12.21) ----

/// Input of `bikri_open_for_edit`: one posting of a بکری, from `BikriDetail::postings`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct EditOpenInput {
    /// The بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// The posting (journal entry) to edit: a `bikri_first` or `bikri_later` one that is not
    /// reversed.
    #[ts(type = "number")]
    pub entry_id: i64,
}

/// A posting loaded for editing (PRD FR-12.18). The draft is in the shape of the book screen and
/// is built from the STORED values: a charge the user typed comes back as a typed charge, a
/// charge a rule decided comes back empty (and `stored` holds its rule and amount).
///
/// - For the first posting the draft holds the header, every lot of the بکری (each with `lot_id`)
///   and the lines of that posting (each with `sale_id`). The screen may add lines and lots and
///   remove lines; it may not remove a lot.
/// - For a later posting the draft has one lot and the lines of that posting. Only the lines
///   (and the date it keeps) can be changed there.
///
/// Keys: a lot has the key `lot:<id>` and a line `sale:<id>`.
#[derive(Debug, Clone, Serialize, TS)]
pub struct EditableBikri {
    /// The بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub number: i64,
    /// Its version, to send back with `bikri_edit_save`.
    #[ts(type = "number")]
    pub version: i64,
    /// The posting being edited.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// `bikri_first` or `bikri_later`.
    pub posting_type: String,
    /// The day of the posting, `YYYY-MM-DD`. An edit keeps it (PRD FR-12.19).
    pub entry_date: String,
    /// False when the day is closed: the Owner must reopen it first (PRD FR-12.18). The draft is
    /// still returned so the screen can show it.
    pub editable: bool,
    /// The draft to show and change.
    pub draft: BikriDraft,
    /// The stored values behind the draft: the lots of the posting, each with only the lines of
    /// this posting in force. The screen shows the rule tags and the amounts from here and
    /// compares typed text with the loaded text; it never adds money.
    pub stored: Vec<SavedLot>,
    /// Every posting of the بکری, oldest first, so the screen can offer the others.
    pub postings: Vec<SavedPosting>,
}

/// Input of `bikri_edit_preview` (Code Standards B5: the same working as the save).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct EditPreviewInput {
    /// The posting being edited.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// The edited draft, with the `lot_id` and `sale_id` it was loaded with.
    pub draft: BikriDraft,
}

/// Input of `bikri_edit_save` (PRD FR-12.19): a reversal of the posting and a new posting on the
/// same day, in one transaction.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct EditSaveInput {
    /// The posting being edited.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// The version read with `bikri_open_for_edit`; `ERR_VERSION_CHANGED` (field `version`) if the
    /// بکری changed since.
    #[ts(type = "number")]
    pub version: i64,
    /// The edited draft.
    pub draft: BikriDraft,
    /// Why. Free text, written to the audit log. Optional: the screen asks for it when the day is
    /// not today.
    #[serde(default)]
    pub reason: Option<String>,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟" (PRD FR-6.7). Needed only when the
    /// edit lowers cash and leaves it below 0.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// What `bikri_edit_save` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BikriEdited {
    /// The بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub number: i64,
    /// Its version after the edit.
    #[ts(type = "number")]
    pub version: i64,
    /// The posting that was taken back.
    #[ts(type = "number")]
    pub old_entry_id: i64,
    /// The reversal of it.
    #[ts(type = "number")]
    pub reversal_entry_id: i64,
    /// The new posting, to open next with `bikri_open_for_edit`.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// The numbers the save wrote; the same as `bikri_edit_preview` gave just before.
    pub preview: BikriPreview,
}

// ---- Cancel and undo (PRD FR-12.22, FR-7.47) ----

/// Input of `bikri_cancel_preview`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CancelPreviewInput {
    /// The بکری.
    #[ts(type = "number")]
    pub id: i64,
}

/// What a cancel would do, for the confirm box (PRD FR-7.8, FR-6.7). Read only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CancelPreview {
    /// The بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub number: i64,
    /// Its version, to send back with `bikri_cancel`.
    #[ts(type = "number")]
    pub version: i64,
    /// Saved or already cancelled.
    pub status: BikriStatus,
    /// The postings in force that the cancel would reverse, oldest first.
    pub postings: Vec<SavedPosting>,
    /// What the cancel does to cash in hand (a نقدی sale cancelled takes its cash back).
    pub cash: CashCheck,
    /// The days of those postings that are closed (`YYYY-MM-DD`), oldest first. If there are
    /// any, the cancel is refused with `ERR_DATE_CLOSED` until the Owner reopens the day.
    pub closed_dates: Vec<String>,
    /// True if the cancel can be done now: the بکری is saved and every posting day is open.
    pub can_cancel: bool,
}

/// Input of `bikri_cancel` (PRD FR-12.22): every posting of the بکری is reversed.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CancelInput {
    /// The بکری.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read with `bikri_get` or `bikri_cancel_preview`.
    #[ts(type = "number")]
    pub version: i64,
    /// Why. Required (PRD FR-12.11): `ERR_REASON_REQUIRED` on `reason`.
    pub reason: String,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟" (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// What `bikri_cancel` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct BikriCancelled {
    /// The بکری. Its number is kept and never used again.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub number: i64,
    /// The reversal entries, one for each posting that was in force.
    #[ts(type = "Array<number>")]
    pub reversal_entry_ids: Vec<i64>,
}

/// Input of `bikri_undo` (PRD FR-7.47): a cancel straight after the save, with the reason
/// `UNDO_REASON_ID`. The 10 seconds are the screen's; the backend only needs the day to be open.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct UndoInput {
    /// The بکری just saved.
    #[ts(type = "number")]
    pub id: i64,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟" (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// What `bikri_undo` gives back: the cancelled بکری and a draft with the same values, to open as
/// a new unsaved entry (PRD FR-7.47). The draft has no `lot_id` or `sale_id`.
#[derive(Debug, Clone, Serialize, TS)]
pub struct UndoResult {
    /// The cancelled بکری.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number, which is not used again.
    #[ts(type = "number")]
    pub number: i64,
    /// The reversal entries.
    #[ts(type = "Array<number>")]
    pub reversal_entry_ids: Vec<i64>,
    /// The values to correct and save again.
    pub draft: BikriDraft,
}

// ---- Write-off (PRD FR-12.16) ----

/// Input of `bikri_write_off`: goods that spoiled or came back are taken off the lot. A stock
/// move only; no journal entry for consigned goods.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct WriteOffInput {
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// نگ to write off. A simple sum is taken (PRD FR-7.48).
    pub qty: String,
    /// Why (خراب، واپس ...). Required: `ERR_REASON_REQUIRED` on `reason`.
    pub reason: String,
}

/// What `bikri_write_off` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct WriteOffDone {
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// The stock move, to undo it with `bikri_write_off_undo`.
    #[ts(type = "number")]
    pub move_id: i64,
    /// How many نگ were written off now.
    #[ts(type = "number")]
    pub qty: i64,
    /// All the نگ written off from the lot.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// نگ left.
    #[ts(type = "number")]
    pub balance: i64,
}

/// Input of `bikri_write_off_undo`: records the opposite stock move.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct WriteOffUndoInput {
    /// The write-off move, from `WriteOffRow::move_id`. `ERR_NOT_FOUND` on `move_id` if it is not a
    /// write-off, or has been undone already.
    #[ts(type = "number")]
    pub move_id: i64,
    /// Why. Required: `ERR_REASON_REQUIRED` on `reason`.
    pub reason: String,
}

/// What `bikri_write_off_undo` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct WriteOffUndone {
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// The opposite stock move that was recorded.
    #[ts(type = "number")]
    pub reversal_move_id: i64,
    /// All the نگ written off from the lot after the undo.
    #[ts(type = "number")]
    pub qty_written_off: i64,
    /// نگ left.
    #[ts(type = "number")]
    pub balance: i64,
}
