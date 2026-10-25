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
/// The control of separate khatas (PRD FR-10.4, 16.6). A khata is never income or expense and is
/// never in a partner's total; it can be the seller of a lot (PRD FR-16.7).
pub const KHATA_CONTROL: &str = "0305";
/// The control of people who borrow money (PRD FR-10.4 v1.2).
pub const MONEY_LENT_CONTROL: &str = "0103";
/// The system account that takes the other side of every opening (PRD FR-10.9).
pub const OPENING_DIFFERENCE_CODE: &str = "06019999";
/// The head of income accounts (PRD FR-17.3). Every posting on them writes partner share rows.
pub const INCOME_HEAD: &str = "04";
/// The head of shop expense accounts (PRD FR-17.3). Every posting on them is split.
pub const EXPENSE_HEAD: &str = "05";
/// نقدی منڈی, the cash buyer with no name (PRD FR-10.4): he pays at once, so he never holds a
/// balance.
pub const NAQDI_MANDI_CODE: &str = "03040000";
/// اپنا مال, the seller of a trip's goods (PRD FR-10.4, FR-12.26). It never holds a balance.
pub const OWN_GOODS_CODE: &str = "030119998";
/// The control of a partner's gardens, trips and fruit account (PRD FR-10.4 v1.6). A code starts with
/// it. These accounts are made with the partner and never take an opening balance (PRD FR-10.9).
pub const VENTURE_CONTROL_CODE: &str = "0604";
/// The control of a garden's account (PRD FR-10.4 v1.6). The account is made with the garden and
/// never takes an opening balance (PRD FR-10.9).
pub const GARDEN_CONTROL_CODE: &str = "0306";
/// بینک: not used from v1.6, because all money is روکڑ (PRD FR-10.4). Nothing opens on it.
pub const BANK_ACCOUNT_CODE: &str = "01020001";
/// Whether an account can take an opening balance (PRD FR-10.9). This is the one named place for
/// the list, used by the save (`ERR_OPENING_NOT_ALLOWED`) and by `AccountRow::can_take_opening`
/// for the openings grid. The documents do not say which accounts are left out; the list is the
/// team default and an open question for Umar:
///
/// - income and shop expense accounts (heads 04 and 05), because their postings need partner
///   share rows (PRD FR-6.5);
/// - ابتدائی فرق itself, which is the other side of every opening;
/// - نقدی منڈی, who pays at once and never holds a balance (PRD FR-12.4);
/// - the پختہ کھاتہ accounts, which hold many people, each with his own balance (PRD FR-13.5).
///
/// ```
/// use mandi_core::accounts::can_take_opening;
/// assert!(can_take_opening("01", "01010001", false)); // روکڑ
/// assert!(can_take_opening("03", "030110001", false)); // a seller
/// assert!(!can_take_opening("04", "04010001", false)); // income
/// assert!(!can_take_opening("06", "06019999", false)); // ابتدائی فرق
/// assert!(!can_take_opening("03", "03040000", false)); // نقدی منڈی
/// assert!(!can_take_opening("03", "03049999", true)); // پختہ کھاتہ
/// ```
pub fn can_take_opening(head_code: &str, code: &str, is_pukhta_khata: bool) -> bool {
    head_code != INCOME_HEAD
        && head_code != EXPENSE_HEAD
        && code != OPENING_DIFFERENCE_CODE
        && code != NAQDI_MANDI_CODE
        // اپنا مال never holds a balance (PRD FR-12.26). The 0604 and 0306 accounts hold only the
        // money of gardens, trips and fruit, which comes from their own entries, and بینک is not
        // used (PRD FR-10.9 v1.6, D4-A31, D4-A32).
        && code != OWN_GOODS_CODE
        && code != BANK_ACCOUNT_CODE
        && !code.starts_with(VENTURE_CONTROL_CODE)
        && !code.starts_with(GARDEN_CONTROL_CODE)
        && !is_pukhta_khata
}

/// Which controls an account of type `other` may go under.
pub struct ControlRule;

impl ControlRule {
    /// True for a control with party type `other` that is not one of the controls that have
    /// their own way in: 0103 is the type ادھار رقم, 0501 is filled by expense heads and 0102 by
    /// bank accounts (plan 06). 0604 (gardens, trips and fruit) is made with each partner (plan 11).
    pub fn allows_other(code: &str, party_type: &str) -> bool {
        party_type == "other" && !matches!(code, "0103" | "0501" | "0102" | "0604")
    }
}

/// The short code of a seller or buyer account (PRD FR-7.39): the serial of its code without
/// leading zeros. `None` for every other account (other accounts have no short code), and for a
/// system serial (0 and 9000 to 9999, PRD FR-10.3), which is never a short code.
///
/// ```
/// use mandi_core::accounts::short_code_of;
/// assert_eq!(short_code_of("03040001").as_deref(), Some("1"));
/// assert_eq!(short_code_of("030110012").as_deref(), Some("12"));
/// assert_eq!(short_code_of("03049999"), None); // پختہ کھاتہ (system serial)
/// assert_eq!(short_code_of("01010001"), None); // cash: not a seller or a buyer
/// ```
pub fn short_code_of(code: &str) -> Option<String> {
    let serial = code
        .strip_prefix(SELLER_CONTROL)
        .or_else(|| code.strip_prefix(BUYER_CONTROL))?;
    if serial.len() != 4 || !serial.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let number: u32 = serial.parse().ok()?;
    (FIRST_SERIAL..=LAST_SERIAL)
        .contains(&number)
        .then(|| number.to_string())
}

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
    /// The tag (نشان, PRD 10.2): empty, or 1 to 4 English capital letters and dots with at least
    /// one letter, for example `K`, `BA`, `D.P`. A lower-case letter is made capital (`k` is
    /// saved as `K`). Never converted to Urdu (PRD FR-7.25). Missing means no tag. Errors:
    /// `ERR_TAG_INVALID` on `tag`.
    #[serde(default)]
    pub tag: String,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
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
    /// The tag (نشان) of an account, shown after its name through `displayName` (PRD 10.2,
    /// Code Standards F38). Null when the account has none, and always null for a record that is
    /// not an account (a city, an item, a packing, a staff member).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag: Option<String>,
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

impl Balance {
    /// No balance.
    pub const ZERO: Balance = Balance {
        amount_paisa: Paisa::ZERO,
        side: BalanceSide::Zero,
    };

    /// The balance from the debit and credit totals of an account: the difference with the side
    /// it stands on (PRD C13). Checked arithmetic; the amount is never negative.
    ///
    /// ```
    /// use mandi_core::accounts::{Balance, BalanceSide};
    /// use mandi_core::money::Paisa;
    /// let dr = Paisa::from_rupees(12_880).unwrap();
    /// let cr = Paisa::from_rupees(10_000).unwrap();
    /// let b = Balance::from_totals(dr, cr).unwrap();
    /// assert_eq!((b.amount_paisa, b.side), (Paisa::from_rupees(2_880).unwrap(), BalanceSide::Dr));
    /// assert_eq!(Balance::from_totals(cr, dr).unwrap().side, BalanceSide::Cr);
    /// assert_eq!(Balance::from_totals(dr, dr).unwrap(), Balance::ZERO);
    /// ```
    pub fn from_totals(dr: Paisa, cr: Paisa) -> Result<Balance, crate::money::MoneyError> {
        let net = dr.checked_sub(cr)?;
        Balance::from_net(net)
    }

    /// The balance from a signed net (debit minus credit).
    pub fn from_net(net: Paisa) -> Result<Balance, crate::money::MoneyError> {
        if net == Paisa::ZERO {
            Ok(Balance::ZERO)
        } else if net.is_negative() {
            Ok(Balance {
                amount_paisa: net.checked_neg()?,
                side: BalanceSide::Cr,
            })
        } else {
            Ok(Balance {
                amount_paisa: net,
                side: BalanceSide::Dr,
            })
        }
    }
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
    /// The tag (نشان), or absent if none (PRD 10.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag: Option<String>,
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
    /// False for the accounts that cannot take an opening balance (see `can_take_opening`), so
    /// the openings grid can leave them out.
    pub can_take_opening: bool,
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
    /// How many rows to return, at most 50,000 (2.5 times the 20,000 accounts of NFR-SCL-1). 0
    /// gives the most, so a list with no limit returns every matching account.
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
    /// What was typed in the tag (نشان). Empty if none. With the names and the phone it decides
    /// which similar account is a duplicate (PRD FR-10.7, E38). Missing means none.
    #[serde(default)]
    pub tag: String,
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
    /// The tag (نشان), or absent if none (PRD 10.2, E38).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag: Option<String>,
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
    /// True when its name, its tag and its phone all equal what was typed: the duplicate warning
    /// of PRD FR-10.7. The same name and phone with another tag is not a duplicate (PRD E38).
    pub is_duplicate: bool,
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
    /// Values for the message, as in any error (Code Standards 4.3): for example
    /// `{"closed_up_to": "09-10-2026"}` on `ERR_DATE_CLOSED`.
    #[ts(type = "Record<string, string | number>")]
    pub values: std::collections::BTreeMap<String, crate::error::ErrorValue>,
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
    /// The tag (نشان), or absent if none (PRD 10.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag: Option<String>,
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
