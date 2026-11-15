//! A بکری that holds carried lots and new lots in one save (plan 76 point 1, PRD FR-12.17 v1.41).
//!
//! Goods left in a lot (باقی مال) are sold on a later day as a later posting of the lot's own
//! بکری. New lots are a first posting of a new بکری. One screen can hold both, and one save,
//! one print and one undo cover them.
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `bikri_mixed_preview` | `MixedSaleInput` | `MixedSalePreview` |
//! | `bikri_mixed_save` | `MixedSaleSaveInput` | `MixedSaleSaved` |
//! | `bikri_mixed_undo` | `MixedUndoInput` | `MixedUndoResult` |
//!
//! Error paths: `carried[i].sales[j].<field>`, `carried[i].lot_id`, `new_lots[k].<field>`,
//! `new_lots[k].sales[j].<field>`, `date`. The suffixes are those of `lots[k]` in `bikri_preview`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::input::{BikriDraft, LotInput, SaleInput};
use super::preview::{BikriPreview, SplitPart, SummaryTotals};

/// A carried lot on the screen: the lot and the new lines typed for it.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CarriedLotInput {
    /// The screen's own key for this lot row, echoed back as the `key` of its `LotPreview` and of
    /// its `MixedPart`, like `LotInput::key`. Empty means `lot:<lot_id>`.
    #[serde(default)]
    pub key: String,
    /// The lot, from `bikri_stock_list`.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// The new lines. A carried lot with no line (or only blank rows) writes nothing.
    #[serde(default)]
    pub sales: Vec<SaleInput>,
}

/// Input of `bikri_mixed_preview`: the day, the carried lots and the new lots. Nothing is saved.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MixedSaleInput {
    /// The day of the whole screen, `DD-MM-YYYY` or a shortcut. An open day, not before the goods
    /// of a carried lot were received.
    pub date: String,
    /// The carried lots, in screen order.
    #[serde(default)]
    pub carried: Vec<CarriedLotInput>,
    /// The new lots, in screen order. Empty means no new بکری is made.
    #[serde(default)]
    pub new_lots: Vec<LotInput>,
}

/// Input of `bikri_mixed_save`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MixedSaleSaveInput {
    /// The day of the whole screen.
    pub date: String,
    /// The carried lots.
    #[serde(default)]
    pub carried: Vec<CarriedLotInput>,
    /// The new lots.
    #[serde(default)]
    pub new_lots: Vec<LotInput>,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟" (PRD FR-6.7). One answer for the
    /// whole save: the cash question is asked over all its entries together.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// Which kind of posting a part of the screen is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MixedPartKind {
    /// A later sale of a carried lot, on its old بکری.
    Carried,
    /// The first posting of the new lots, on a new بکری.
    New,
}

/// One entry of the save, with its own totals and split (what the standalone path would show).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MixedPart {
    /// Carried or new.
    pub kind: MixedPartKind,
    /// The key of the carried lot row as the screen sent it (`lot:<lot_id>` when it sent none);
    /// null for the new part. It is the `key` of this lot's `LotPreview` in `preview.lots`.
    pub key: Option<String>,
    /// The carried lot. Null for the new part.
    #[ts(type = "number | null")]
    pub lot_id: Option<i64>,
    /// The old بکری of a carried lot. Null for the new part.
    #[ts(type = "number | null")]
    pub old_bikri_id: Option<i64>,
    /// Its number. Null for the new part.
    #[ts(type = "number | null")]
    pub old_number: Option<i64>,
    /// The lines of PRD 12.5 for this entry alone.
    pub summary: SummaryTotals,
    /// The partner split of this entry alone (it rounds on its own).
    pub split: Vec<SplitPart>,
}

/// What `bikri_mixed_preview` sends back. `preview` has the shapes of `bikri_preview` for the whole
/// screen (lots: the carried lots first, then the new lots; summary, split and cash over all
/// entries). `parts` has each entry's own figures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MixedSalePreview {
    /// The whole screen.
    pub preview: BikriPreview,
    /// The entries the save would write, in the order it writes them.
    pub parts: Vec<MixedPart>,
}

/// A carried lot that was saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CarriedSaved {
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// Its old بکری.
    #[ts(type = "number")]
    pub old_bikri_id: i64,
    /// The old بکری's number (a later sale does not make a new number).
    #[ts(type = "number")]
    pub old_number: i64,
    /// The journal entry of the new posting.
    #[ts(type = "number")]
    pub entry_id: i64,
}

/// What `bikri_mixed_save` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MixedSaleSaved {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The new بکری. Null when the screen had only carried lots.
    #[ts(type = "number | null")]
    pub new_bikri_id: Option<i64>,
    /// Its number. Null when there is no new بکری.
    #[ts(type = "number | null")]
    pub new_bikri_number: Option<i64>,
    /// The carried lots that were saved, in screen order.
    pub carried: Vec<CarriedSaved>,
    /// Every journal entry written, in the order written (the carried ones, then the new one).
    #[ts(type = "Array<number>")]
    pub entry_ids: Vec<i64>,
    /// The بکری this save sold, for the sale print and the buyer parchi (`bikri_ids` of
    /// `DaySlipsInput`, plan 76 point 3, FR-12.48): the new بکری, if any, then the old بکری of each
    /// carried lot, with no duplicates.
    #[ts(type = "Array<number>")]
    pub print_bikri_ids: Vec<i64>,
    /// The whole screen as the save wrote it.
    pub preview: BikriPreview,
}

/// Input of `bikri_mixed_undo`: the ones `bikri_mixed_save` just gave back.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MixedUndoInput {
    /// The journal entries of the carried postings (`MixedSaleSaved::entry_ids` of the carried
    /// lots).
    #[serde(default)]
    #[ts(type = "Array<number>")]
    pub later_entry_ids: Vec<i64>,
    /// The new بکری, if the save made one.
    #[serde(default)]
    #[ts(type = "number | null", optional)]
    pub new_bikri_id: Option<i64>,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟" (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// What `bikri_mixed_undo` gives back: the reversals and a draft with the same values, to open as
/// the screen again (PRD FR-7.47). The draft has no `lot_id` or `sale_id` on its new lots.
#[derive(Debug, Clone, Serialize, TS)]
pub struct MixedUndoResult {
    /// Every reversal entry, in the order made.
    #[ts(type = "Array<number>")]
    pub reversal_entry_ids: Vec<i64>,
    /// The new بکری that was cancelled, with its number (not used again).
    #[ts(type = "number | null")]
    pub cancelled_bikri_id: Option<i64>,
    /// The values to correct and save again.
    pub draft: MixedUndoDraft,
}

/// The screen as it was: the day, the carried lots with their lines, the new lots.
#[derive(Debug, Clone, Serialize, TS)]
pub struct MixedUndoDraft {
    /// The day, `DD-MM-YYYY`.
    pub date: String,
    /// The carried lots with the lines they had.
    pub carried: Vec<MixedUndoCarried>,
    /// The new بکری's draft, or null.
    pub new_draft: Option<BikriDraft>,
}

/// A carried lot and its lines in a [`MixedUndoDraft`]. The lines are drafts of the old lines.
#[derive(Debug, Clone, Serialize, TS)]
pub struct MixedUndoCarried {
    /// The lot.
    #[ts(type = "number")]
    pub lot_id: i64,
    /// The lines it had.
    pub sales: Vec<SaleInput>,
}
