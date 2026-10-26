//! The ten master lists of PRD 10.3 (plan 06): city, item, packing, truck stand (اڈا), mark,
//! store, standard narration, expense head and staff.
//!
//! This module holds what is shared by the screens, the services and the database layer and has
//! no business rules of its own: the kinds, the input a screen sends (all text), the row a
//! screen reads back, and the checks of every field ([`prepare`]). It never touches the database.
//!
//! - A master is added, edited and made inactive. It is never deleted (PRD E21, D3).
//! - Every named master has `name_ur` and `name_en` (PRD FR-7.24, FR-7.28), checked for their
//!   script here and for their spelling in `services::spelling` (PRD FR-7.66).
//! - The three search columns are built here, only by `core::text` (Code Standards B26).

mod prepare;
mod rows;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use prepare::{
    AccountToMake, ColValue, MAX_CODE_DIGITS, NameTexts, Prepared, Reference, prepare,
};
pub use rows::{
    AddaRow, CityRow, ExpenseHeadRow, ItemRow, MarkaRow, MasterRow, NarrationRow, PackingRow,
    StaffRow, StoreRow, format_kg,
};

/// The kind of a master list. Serialised as `city`, `item`, `packing`, `adda`, `marka`, `store`,
/// `narration`, `expense_head`, `staff` (also the audit entity names).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MasterKind {
    /// شہر.
    City,
    /// قسم مال.
    Item,
    /// پیکنگ.
    Packing,
    /// اڈا (truck stand).
    Adda,
    /// مارکہ.
    Marka,
    /// سٹور.
    Store,
    /// تفصیل (standard narration).
    Narration,
    /// خرچہ مد (expense head); makes an account under control 0501.
    ExpenseHead,
    /// عملہ.
    Staff,
}

impl MasterKind {
    /// Every kind, in the order of PRD 10.3.
    pub const ALL: [MasterKind; 9] = [
        MasterKind::City,
        MasterKind::Item,
        MasterKind::Packing,
        MasterKind::Adda,
        MasterKind::Marka,
        MasterKind::Store,
        MasterKind::Narration,
        MasterKind::ExpenseHead,
        MasterKind::Staff,
    ];

    /// The storage and wire form, for example `expense_head`.
    pub const fn as_str(self) -> &'static str {
        match self {
            MasterKind::City => "city",
            MasterKind::Item => "item",
            MasterKind::Packing => "packing",
            MasterKind::Adda => "adda",
            MasterKind::Marka => "marka",
            MasterKind::Store => "store",
            MasterKind::Narration => "narration",
            MasterKind::ExpenseHead => "expense_head",
            MasterKind::Staff => "staff",
        }
    }

    /// Reads the wire form.
    pub fn parse(text: &str) -> Option<MasterKind> {
        MasterKind::ALL.into_iter().find(|k| k.as_str() == text)
    }

    /// The table of this kind (Architecture 15.4). A fixed list, never built from input.
    pub const fn table(self) -> &'static str {
        match self {
            MasterKind::City => "cities",
            MasterKind::Item => "items",
            MasterKind::Packing => "packings",
            MasterKind::Adda => "addas",
            MasterKind::Marka => "markas",
            MasterKind::Store => "stores",
            MasterKind::Narration => "narrations",
            MasterKind::ExpenseHead => "expense_heads",
            MasterKind::Staff => "staff",
        }
    }

    /// True if the table has a `code` column (narrations do not; staff have one from plan 32).
    pub const fn has_code(self) -> bool {
        !matches!(self, MasterKind::Narration)
    }

    /// The code type of an item, a packing or a staff member: their codes are the type number plus
    /// a running number (`0040`, `0050`, `0060`, plan 32 R2). `None` for every other kind, which
    /// keeps a plain number (PRD 1.8 R4).
    pub const fn code_type(self) -> Option<crate::codes::CodeType> {
        match self {
            MasterKind::Item => Some(crate::codes::CodeType::Item),
            MasterKind::Packing => Some(crate::codes::CodeType::Packing),
            MasterKind::Staff => Some(crate::codes::CodeType::Staff),
            _ => None,
        }
    }
}

/// Everything a screen sends to save a master: a new one (`id` null) or an edit (`id` and the
/// `version` it read). All values are text; the backend checks every one (Code Standards V0).
/// `confirmed_rules_words` are the words the user confirmed in the spelling check (PRD FR-7.66).
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MasterInput {
    /// شہر.
    City(CityInput),
    /// قسم مال.
    Item(ItemInput),
    /// پیکنگ.
    Packing(PackingInput),
    /// اڈا.
    Adda(AddaInput),
    /// مارکہ.
    Marka(MarkaInput),
    /// سٹور.
    Store(StoreInput),
    /// تفصیل.
    Narration(NarrationInput),
    /// خرچہ مد.
    ExpenseHead(ExpenseHeadInput),
    /// عملہ.
    Staff(StaffInput),
}

impl MasterInput {
    /// The kind of this input.
    pub fn kind(&self) -> MasterKind {
        match self {
            MasterInput::City(_) => MasterKind::City,
            MasterInput::Item(_) => MasterKind::Item,
            MasterInput::Packing(_) => MasterKind::Packing,
            MasterInput::Adda(_) => MasterKind::Adda,
            MasterInput::Marka(_) => MasterKind::Marka,
            MasterInput::Store(_) => MasterKind::Store,
            MasterInput::Narration(_) => MasterKind::Narration,
            MasterInput::ExpenseHead(_) => MasterKind::ExpenseHead,
            MasterInput::Staff(_) => MasterKind::Staff,
        }
    }
}

/// A city (شہر).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CityInput {
    /// Null to add, the row id to edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// The `version` read with the row, for an edit. Null when adding.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// Unique code, 1 to 10 digits. Empty gives the next free code.
    pub code: String,
    /// Urdu name (PRD FR-7.28).
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Words the user confirmed in the spelling check (PRD FR-7.66).
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
}

/// An item (قسم مال).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ItemInput {
    /// Null to add, the row id to edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// The `version` read with the row, for an edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// Unique code. Empty gives the next free code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// `fruit` (پھل) or `vegetable` (سبزی).
    pub item_group: String,
    /// The packing it usually comes in, if any.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub default_packing_id: Option<i64>,
    /// `unit` (نگ) or `weight` (وزن).
    pub sold_by: String,
    /// Words the user confirmed in the spelling check.
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
}

/// A packing (پیکنگ).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PackingInput {
    /// Null to add, the row id to edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// The `version` read with the row, for an edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// Unique code. Empty gives the next free code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Approximate weight in kg, up to 3 decimals, for example `12.5`. Empty if not known.
    pub approx_kg: String,
    /// Words the user confirmed in the spelling check.
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
}

/// A truck stand (اڈا).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AddaInput {
    /// Null to add, the row id to edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// The `version` read with the row, for an edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// Unique code. Empty gives the next free code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The city it is in, if any.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub city_id: Option<i64>,
    /// Words the user confirmed in the spelling check.
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
}

/// A mark on crates (مارکہ). The mark is kept as typed, in one script (PRD FR-7.25), so it has
/// no spelling check.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MarkaInput {
    /// Null to add, the row id to edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// The `version` read with the row, for an edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// Unique code. Empty gives the next free code.
    pub code: String,
    /// The mark as typed, for example `AR`.
    pub mark_text: String,
    /// The seller it belongs to, if any.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub owner_account_id: Option<i64>,
}

/// A store (سٹور).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct StoreInput {
    /// Null to add, the row id to edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// The `version` read with the row, for an edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// Unique code. Empty gives the next free code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Words the user confirmed in the spelling check.
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
}

/// A standard narration (تفصیل). Free text, so no spelling check (PRD FR-7.66).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct NarrationInput {
    /// Null to add, the row id to edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// The `version` read with the row, for an edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// The Urdu text, up to 500 characters.
    pub text_ur: String,
    /// The English text, up to 500 characters.
    pub text_en: String,
    /// The voucher types it is offered in: any of `receipt`, `payment`, `advance`,
    /// `shop_expense`, `personal_expense`, `personal_return`, `money_lent`,
    /// `money_lent_return`, `general`. Empty means every type.
    #[serde(default)]
    pub used_in: Vec<String>,
}

/// An expense head (خرچہ مد). Saving it also makes its account under control 0501.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ExpenseHeadInput {
    /// Null to add, the row id to edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// The `version` read with the row, for an edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// Unique code. Empty gives the next free code.
    pub code: String,
    /// Urdu name (also the name of its account).
    pub name_ur: String,
    /// English name (also the name of its account).
    pub name_en: String,
    /// Words the user confirmed in the spelling check.
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
}

/// A staff member (عملہ).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct StaffInput {
    /// Null to add, the row id to edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// The `version` read with the row, for an edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Unique code `006` plus a running number (`0060`, `0061` ...). Empty gives the next free code
    /// (plan 32 R2).
    #[serde(default)]
    pub code: String,
    /// The job: `munshi`, `collector` or `labour`. Not a login role.
    pub job: String,
    /// Phone, `03XXXXXXXXX` or `03XX-XXXXXXX`. Empty if not known.
    pub phone: String,
    /// Words the user confirmed in the spelling check.
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
}

/// What a screen asks for to list one master.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MastersListInput {
    /// Which master.
    pub kind: MasterKind,
    /// True to include the inactive ones too.
    pub include_inactive: bool,
    /// Only this record, for a form that edits one. The answer has the same shape: its count and
    /// totals are over this one record (none if it is inactive and `include_inactive` is false).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// How many rows to skip. 0 for the first page (PRD FR-19.11).
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives a page of 100 (Code Standards B31).
    #[serde(default)]
    pub limit: u32,
}

/// One page of a master list with the totals over the whole list (PRD FR-19.11, B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MastersList {
    /// This page, by code (by name for the kinds without a code).
    pub rows: Vec<MasterRow>,
    /// How many records match: active ones, and the inactive ones when they were asked for.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The counts over every matching record.
    pub totals: MastersTotals,
}

/// The totals of a master list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MastersTotals {
    /// How many of the matching records are active.
    #[ts(type = "number")]
    pub active_count: i64,
}

/// What a screen sends to make a master active or inactive.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MastersSetActiveInput {
    /// Which master.
    pub kind: MasterKind,
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The new state. The `version` is not needed: the change is idempotent.
    pub is_active: bool,
}

/// What a screen sends to get the next free code.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MastersNextCodeInput {
    /// Which master. A narration has no code and gives an empty text.
    pub kind: MasterKind,
}
