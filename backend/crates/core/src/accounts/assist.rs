//! What a code brings (plan 08, PRD FR-7.38 to FR-7.42, FR-5.4): picker rows for accounts, the
//! answer to a typed code, the party card and the global search. Only shapes: the matching is in
//! `services::search` and the card is worked out in `services::assist`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{AccountType, Balance};
use crate::money::Paisa;

/// Which accounts an account field takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AccountScope {
    /// A seller field (control 03011). A short code is the running number of a seller's code
    /// (`0` finds `0010`). The اپنا مال account is not offered here (PRD FR-12.26).
    Seller,
    /// The seller field of a lot: the same as `seller`, and also the اپنا مال account (030119998),
    /// which is offered only here (PRD FR-12.26, FR-10.4). Picking it asks for a purchase.
    LotSeller,
    /// A buyer field (control 0304), including نقدی منڈی and پختہ کھاتہ. A short code is the running
    /// number of a buyer's code (`1` finds `0021`).
    Buyer,
    /// A field that takes any account. A short code lists the seller and the buyer with that
    /// running number, each with its type (PRD 1.8 R5): `1` lists `0011` and `0021`.
    Any,
    /// The accounts of one control (`control_id` names it). Short codes do not apply: other
    /// accounts are found by full code or name (PRD FR-7.39).
    Control,
}

/// Search the accounts of a field.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SearchAccountsInput {
    /// Which accounts the field takes.
    pub scope: AccountScope,
    /// The control, for scope `control`. Ignored for the other scopes.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub control_id: Option<i64>,
    /// What was typed. Raw: never converted to Urdu (PRD FR-7.22). **Empty text gives the whole
    /// list of the field** (plan 32 R7): the accounts this user chose last first (latest first), then
    /// every other active account by code. A click on the picker opens this list.
    pub text: String,
    /// The most hits in this page, at most 50.
    pub limit: u32,
    /// How many hits to skip, for the next page of a long list. 0 for the first page. A page with
    /// fewer than `limit` hits is the last one. Missing means 0.
    #[serde(default)]
    pub offset: u32,
}

/// The last accounts used in a field, then the rest (`Alt+Down`, PRD FR-7.41).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AccountsRecentInput {
    /// Which accounts the field takes.
    pub scope: AccountScope,
    /// The control, for scope `control`. Each control has its own recent list.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub control_id: Option<i64>,
}

/// Tell the app that an account was chosen in a field, so it comes first next time (PRD
/// FR-7.41). Writes the user's own preference and nothing else (Code Standards B29).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AccountsTouchInput {
    /// Which accounts the field takes.
    pub scope: AccountScope,
    /// The control, for scope `control`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub control_id: Option<i64>,
    /// The account chosen.
    #[ts(type = "number")]
    pub id: i64,
}

/// One account a picker offers (PRD FR-7.40). The balance is not here: it is read from the
/// journal on demand with `accounts_balances`, so the search stays fast.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AccountHit {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The full code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The tag (نشان), or absent if none (PRD 10.2, FR-7.40).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag: Option<String>,
    /// Second line in Urdu: father or firm and city.
    pub extra_ur: String,
    /// Second line in English.
    pub extra_en: String,
    /// The kind of account, so a list of seller and buyer shows "بیوپاری" or "گاہک".
    pub account_type: AccountType,
    /// The control it is under.
    #[ts(type = "number")]
    pub control_id: i64,
    /// False for an account hidden from pickers.
    pub is_active: bool,
}

/// A code typed in an account field, to be taken on `Enter` or on leaving the field (never
/// while typing, because `1` may be the start of `12`).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ResolveCodeInput {
    /// Which accounts the field takes.
    pub scope: AccountScope,
    /// The control, for scope `control`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub control_id: Option<i64>,
    /// What was typed. A running number (`0`, `11`: digits with no leading zero, at most 7) is a
    /// short code; a full code (`0010`, `00111`, or any other account's code) is looked up as it
    /// is (plan 32 R5).
    pub text: String,
}

/// How a typed code came out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ResolveStatus {
    /// Exactly one active account. It is in `account`.
    Found,
    /// A field that takes any account, and both a seller and a buyer have this running number.
    /// They are in `candidates`.
    Several,
    /// No account has this code. `suggested_code` is the full code an `F4` add would use, if
    /// the field has one series.
    Unknown,
    /// Only for the field of one control (other accounts keep the control code plus a 4-digit
    /// serial): the serial belongs to the system (0000 and 9000 to 9999), so such a code cannot
    /// be added (PRD FR-10.3). Never returned for sellers and buyers from plan 32: every running
    /// number from 0 is a normal short code, and the two پختہ کھاتہ accounts have no short code.
    Reserved,
    /// The account exists but is hidden from pickers; a new posting cannot use it. It is in
    /// `account`.
    Inactive,
}

/// The answer to a typed code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ResolvedCode {
    /// How it came out.
    pub status: ResolveStatus,
    /// The account, for `found` and `inactive`.
    pub account: Option<AccountHit>,
    /// The seller and the buyer, for `several`. Empty otherwise.
    pub candidates: Vec<AccountHit>,
    /// The full code an add would use, for `unknown`.
    pub suggested_code: Option<String>,
}

/// The balances of some accounts, read from the journal.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AccountsBalancesInput {
    /// The accounts, at most 500.
    #[ts(type = "Array<number>")]
    pub ids: Vec<i64>,
}

/// One account's balance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AccountBalance {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The balance now.
    pub balance: Balance,
}

/// Ask for the party card of an account.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartyCardInput {
    /// The account.
    #[ts(type = "number")]
    pub account_id: i64,
}

/// Why a card shows a warning. The screen shows a word as well as a colour (NFR-USE-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CardWarningKind {
    /// The account is hidden from pickers.
    Inactive,
    /// A buyer owes more than his credit limit. `amount_paisa` is how much more.
    OverCreditLimit,
    /// A seller owes the firm more than his open advances cover. `amount_paisa` is how much
    /// (PRD FR-15.9).
    OwesFirm,
}

/// One warning on a card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CardWarning {
    /// What it is.
    pub kind: CardWarningKind,
    /// The amount it is about, if it has one.
    #[ts(type = "Paisa | null")]
    pub amount_paisa: Option<Paisa>,
}

/// Everything the card next to an account field shows (PRD FR-7.38). Read only. Balances come
/// from the posted journal, never from the search index (Code Standards B27).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartyCard {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The full code.
    pub code: String,
    /// The kind of account.
    pub account_type: AccountType,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The tag (نشان), or absent if none (PRD 10.2, FR-7.38).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag: Option<String>,
    /// Father or firm in Urdu. Empty if none.
    pub father_or_firm_ur: String,
    /// Father or firm in English. Empty if none.
    pub father_or_firm_en: String,
    /// City in Urdu. Empty if none.
    pub city_ur: String,
    /// City in English. Empty if none.
    pub city_en: String,
    /// Phone as 11 digits. Empty if none.
    pub phone: String,
    /// False when hidden from pickers.
    pub is_active: bool,
    /// The balance now, with its side (بنام or جمع).
    pub balance: Balance,
    /// The date of the last posted entry on this account, `YYYY-MM-DD`, or null if none.
    pub last_entry_date: Option<String>,
    /// A buyer's credit limit, if he has one.
    #[ts(type = "Paisa | null")]
    pub credit_limit: Option<Paisa>,
    /// How much a buyer owes now (his debit balance, 0 if none). Null for other accounts.
    #[ts(type = "Paisa | null")]
    pub credit_used: Option<Paisa>,
    /// A seller's open advance: advances given less what was adjusted (PRD FR-15.8). Null for
    /// other accounts.
    #[ts(type = "Paisa | null")]
    pub advance_open: Option<Paisa>,
    /// Warnings, each shown with a word and a colour.
    pub warnings: Vec<CardWarning>,
}

/// Global search (`Ctrl+K`, PRD FR-5.4).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SearchGlobalInput {
    /// What was typed. Raw, never converted.
    pub text: String,
    /// The most hits, at most 50.
    pub limit: u32,
}

/// One result of the global search. Each result has a `kind`: accounts, and (plan 13) a بکری
/// found by its number or by the start of its truck number. Later deliverables add voucher
/// numbers and screens as more kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GlobalHit {
    /// An account.
    Account {
        /// The account.
        hit: AccountHit,
    },
    /// A بکری (PRD FR-5.4). The accounts come first, then the بکری with this number, then the
    /// newest بکری whose truck number starts with the text.
    Bikri {
        /// The بکری.
        hit: crate::bikri::BikriHit,
    },
    /// A voucher found by its number (PRD FR-5.4, plan 15): the digits alone find the voucher
    /// with that number in each series, `R-12`, `P-12` and `A-12` (or `و-12`, `ا-12`, `پ-12`)
    /// find one series. They come after the بکری.
    Voucher {
        /// The voucher.
        hit: crate::vouchers::VoucherHit,
    },
}
