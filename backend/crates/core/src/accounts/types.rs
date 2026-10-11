//! The wire types of plan 07 (all exported to `frontend/src/bindings`).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::ErrorCode;
use crate::money::Paisa;

/// The four kinds of account a user can make (PRD 10.2 "قسم کھاتہ").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// بیوپاری, under control 03011.
    Seller,
    /// گاہک, under control 0304.
    Buyer,
    /// ادھار رقم, a person who borrows from the shop, under control 0103.
    MoneyLent,
    /// دیگر: any other account, under the control `control_id` names.
    Other,
}

impl AccountType {
    /// The wire form.
    pub const fn as_str(self) -> &'static str {
        match self {
            AccountType::Seller => "seller",
            AccountType::Buyer => "buyer",
            AccountType::MoneyLent => "money_lent",
            AccountType::Other => "other",
        }
    }

    /// The control a type always goes under, or `None` for `Other` (the user chooses).
    pub const fn fixed_control(self) -> Option<&'static str> {
        match self {
            AccountType::Seller => Some(SELLER_CONTROL),
            AccountType::Buyer => Some(BUYER_CONTROL),
            AccountType::MoneyLent => Some(MONEY_LENT_CONTROL),
            AccountType::Other => None,
        }
    }
}

/// The control of sellers (PRD FR-10.2).
pub const SELLER_CONTROL: &str = "03011";
/// The control of buyers (PRD FR-10.2).
pub const BUYER_CONTROL: &str = "0304";
/// The control of people who borrow money (PRD FR-10.4 v1.2).
pub const MONEY_LENT_CONTROL: &str = "0103";
/// The system account that takes the other side of every opening (PRD FR-10.9).
pub const OPENING_DIFFERENCE_CODE: &str = "06019999";
/// The first normal serial (PRD FR-10.3).
pub const FIRST_SERIAL: u32 = 1;
/// The last normal serial. 9000 to 9999 and 0 are for system accounts (PRD FR-10.3).
pub const LAST_SERIAL: u32 = 8999;

/// Everything a screen sends to save an account: a new one (`id` null) or an edit (`id` and the
/// `version` it read). All values are text; the backend checks every one (Code Standards V0).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AccountInput {
    /// Null to add, the row id to edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub id: Option<i64>,
    /// The `version` read with the row, for an edit.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// The kind of account.
    pub account_type: AccountType,
    /// The control, for type `other`. Ignored for the other types.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub control_id: Option<i64>,
    /// The full code (control code plus 4 digits). Empty gives the next free code.
    pub code: String,
    /// Urdu name (PRD FR-7.28).
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Words the user confirmed in the spelling check (PRD FR-7.66). The father or firm name is
    /// checked with the names.
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
    /// Father or firm name in Urdu. Empty if none.
    pub father_or_firm_ur: String,
    /// Father or firm name in English. Empty if none.
    pub father_or_firm_en: String,
    /// The city, if any.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub city_id: Option<i64>,
    /// The truck stand, if any.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub adda_id: Option<i64>,
    /// Phone, `03XXXXXXXXX` or `03XX-XXXXXXX`. Empty if none.
    pub phone1: String,
    /// Second phone. Empty if none.
    pub phone2: String,
    /// Address. Empty if none.
    pub address: String,
    /// The same person's other account (as seller and as buyer), if any.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub linked_account_id: Option<i64>,
    /// `DD-MM-YYYY` or a shortcut. Required when an opening amount is typed. Must be an open day.
    pub opening_date: String,
    /// Opening debit (بنام) in whole rupees, as text. Empty if none.
    pub opening_dr: String,
    /// Opening credit (جمع) in whole rupees, as text. Empty if none. Not together with a debit.
    pub opening_cr: String,
    /// The opening debit of a seller counts as an advance (PRD FR-15.8).
    pub opening_is_advance: bool,
    /// Credit sales allowed (PRD FR-12.4). Ticked by default in the form.
    pub credit_allowed: bool,
    /// Credit limit in whole rupees, as text. Empty if none.
    pub credit_limit: String,
    /// A seller's usual مارکہ.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub default_marka_id: Option<i64>,
    /// The collector of a buyer's dues.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub collector_staff_id: Option<i64>,
    /// The collection day, 0 (Monday) to 6 (Sunday). Null if none.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub collection_weekday: Option<i64>,
    /// False hides the account from pickers; reports keep it.
    pub is_active: bool,
    /// A note, up to 500 characters.
    pub note: String,
}

/// A small pointer to another record, for the lists and the form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RecordRef {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Its code. Empty if it has none (a staff member).
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
}

/// Which side a balance stands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum BalanceSide {
    /// بنام: the account is owed money, or holds an asset.
    Dr,
    /// جمع: the account owes money, or holds income or capital.
    Cr,
    /// Nothing either way.
    Zero,
}

/// A balance as the screens show it: an amount that is never negative and its side
/// ("2,880 بنام"). Worked out in Rust from the posted journal, never by the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct Balance {
    /// The amount, 0 or more.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// The side.
    pub side: BalanceSide,
}

/// One account as the list and the form show it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AccountRow {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Version, to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
    /// The full code.
    pub code: String,
    /// The 4-digit serial as a number.
    pub serial: u32,
    /// The kind of account (an account under another control is `other`).
    pub account_type: AccountType,
    /// The control it is under.
    pub control: RecordRef,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Father or firm name in Urdu. Empty if none.
    pub father_or_firm_ur: String,
    /// Father or firm name in English. Empty if none.
    pub father_or_firm_en: String,
    /// The city, if any.
    pub city: Option<RecordRef>,
    /// The truck stand, if any.
    pub adda: Option<RecordRef>,
    /// Phone as 11 digits. Empty if none.
    pub phone1: String,
    /// Second phone. Empty if none.
    pub phone2: String,
    /// Address. Empty if none.
    pub address: String,
    /// The linked account, if any.
    pub linked_account: Option<RecordRef>,
    /// The opening date, `YYYY-MM-DD`, or null.
    pub opening_date: Option<String>,
    /// The opening debit.
    #[ts(type = "Paisa")]
    pub opening_dr_paisa: Paisa,
    /// The opening credit.
    #[ts(type = "Paisa")]
    pub opening_cr_paisa: Paisa,
    /// The opening debit counts as an advance.
    pub opening_is_advance: bool,
    /// Credit sales allowed.
    pub credit_allowed: bool,
    /// The credit limit, if any.
    #[ts(type = "Paisa | null")]
    pub credit_limit_paisa: Option<Paisa>,
    /// A seller's usual مارکہ (its code and text).
    pub default_marka: Option<RecordRef>,
    /// The collector, if any.
    pub collector: Option<RecordRef>,
    /// The collection day, 0 (Monday) to 6 (Sunday), or null.
    #[ts(type = "number | null")]
    pub collection_weekday: Option<i64>,
    /// A system account of PRD FR-10.4: its code cannot change and it cannot be deleted.
    pub is_system: bool,
    /// The پختہ کھاتہ account of one-time people (PRD Section 13).
    pub is_pukhta_khata: bool,
    /// False when hidden from pickers.
    pub is_active: bool,
    /// The note. Empty if none.
    pub note: String,
    /// The balance now, from the posted journal.
    pub balance: Balance,
}

/// What the account list is filtered by.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AccountFilter {
    /// Only this kind of account, or null for all.
    #[serde(default)]
    pub account_type: Option<AccountType>,
    /// Part of the code, or part of a name in either script. Empty for all.
    pub text: String,
    /// True to list the inactive accounts too.
    pub include_inactive: bool,
}

/// The account list with its footer (PRD FR-10.5).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AccountsListInput {
    /// The filter.
    pub filter: AccountFilter,
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return, at most 2,000. 0 gives the most.
    #[serde(default)]
    pub limit: u32,
}

/// The rows of the list and the totals of all rows that match (not only this page).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AccountsList {
    /// This page, by code.
    pub rows: Vec<AccountRow>,
    /// How many accounts match the filter.
    pub count: u32,
    /// The sum of the opening debits of all matching accounts.
    #[ts(type = "Paisa")]
    pub total_dr_paisa: Paisa,
    /// The sum of the opening credits of all matching accounts.
    #[ts(type = "Paisa")]
    pub total_cr_paisa: Paisa,
}

/// One account for the edit form.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AccountsGetInput {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
}

/// Make an account active or inactive.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AccountsSetActiveInput {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The new state.
    pub is_active: bool,
}

/// The next free code for a kind of account.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AccountsNextCodeInput {
    /// The kind.
    pub account_type: AccountType,
    /// The control, for type `other`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub control_id: Option<i64>,
}

/// What has been typed so far, to find accounts that look the same (PRD FR-10.7, FR-7.63).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AccountsSimilarInput {
    /// What was typed in the Urdu name. Empty if nothing yet.
    pub name_ur: String,
    /// What was typed in the English name. Empty if nothing yet.
    pub name_en: String,
    /// What was typed in the phone. Empty if nothing yet.
    pub phone: String,
    /// The account being edited, which is left out of the answer.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub exclude_id: Option<i64>,
}

/// An account that looks like the one being typed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SimilarAccount {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Father or firm name in Urdu.
    pub father_or_firm_ur: String,
    /// Father or firm name in English.
    pub father_or_firm_en: String,
    /// City name in Urdu. Empty if none.
    pub city_ur: String,
    /// City name in English. Empty if none.
    pub city_en: String,
    /// Phone. Empty if none.
    pub phone1: String,
    /// The kind of account.
    pub account_type: AccountType,
}

/// One line of the openings grid.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct OpeningLineInput {
    /// The account.
    #[ts(type = "number")]
    pub account_id: i64,
    /// The `version` read with the account.
    #[ts(type = "number")]
    pub version: i64,
    /// The opening date, `DD-MM-YYYY`.
    pub opening_date: String,
    /// Opening debit as text. Empty if none.
    pub opening_dr: String,
    /// Opening credit as text. Empty if none.
    pub opening_cr: String,
    /// The debit counts as an advance (sellers only).
    pub opening_is_advance: bool,
}

/// The whole grid.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct OpeningsInput {
    /// The lines, in the order of the grid.
    pub lines: Vec<OpeningLineInput>,
}

/// A problem on one line of the grid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct LineError {
    /// The error code.
    pub code: ErrorCode,
    /// The field, for example `opening_dr`.
    pub field: String,
}

/// One line of the preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OpeningPreviewRow {
    /// The account, or null if the id is wrong.
    pub account: Option<RecordRef>,
    /// What is wrong with the line. Empty if nothing.
    pub errors: Vec<LineError>,
    /// The opening debit that will be posted, 0 if none or if the line has errors.
    #[ts(type = "Paisa")]
    pub opening_dr_paisa: Paisa,
    /// The opening credit that will be posted.
    #[ts(type = "Paisa")]
    pub opening_cr_paisa: Paisa,
}

/// The grid checked, nothing saved (the same function as the save, Code Standards B5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OpeningsPreview {
    /// One row per line.
    pub rows: Vec<OpeningPreviewRow>,
    /// The debit total of the lines.
    #[ts(type = "Paisa")]
    pub total_dr_paisa: Paisa,
    /// The credit total of the lines.
    #[ts(type = "Paisa")]
    pub total_cr_paisa: Paisa,
    /// The balance of ابتدائی فرق if the grid were saved. Zero when all openings balance.
    pub opening_difference_after: Balance,
}

/// The grid saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct OpeningsSaved {
    /// The accounts, as saved (also the lines that did not change).
    pub rows: Vec<AccountRow>,
    /// The balance of ابتدائی فرق now.
    pub opening_difference: Balance,
}

/// The chart of accounts in three levels with live balances (PRD FR-10.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CoaTree {
    /// The six heads.
    pub heads: Vec<HeadNode>,
}

/// A head (مد) with its controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct HeadNode {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Code, for example `03`.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// `asset`, `payable`, `party`, `income`, `expense` or `capital`.
    pub nature: String,
    /// The balance of everything under it.
    pub balance: Balance,
    /// Its controls.
    pub controls: Vec<ControlNode>,
}

/// A control (کنٹرول) with its accounts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ControlNode {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Code, for example `0304`.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// `seller`, `buyer` or `other`.
    pub party_type: String,
    /// The balance of everything under it.
    pub balance: Balance,
    /// Its accounts, by code.
    pub accounts: Vec<AccountNode>,
}

/// An account in the tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AccountNode {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The balance now.
    pub balance: Balance,
    /// False when hidden from pickers.
    pub is_active: bool,
    /// A system account.
    pub is_system: bool,
}

/// A head in the plain list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CoaHead {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// `asset`, `payable`, `party`, `income`, `expense` or `capital`.
    pub nature: String,
}

/// A control in the plain list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CoaControl {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The head it is under.
    #[ts(type = "number")]
    pub head_id: i64,
    /// Code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// `seller`, `buyer` or `other`.
    pub party_type: String,
    /// True if an account of type `other` may be made under it (not sellers, buyers or
    /// ادھار رقم, which have their own types).
    pub allowed_for_other: bool,
}
