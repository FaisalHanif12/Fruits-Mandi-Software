//! Gardens (باغات, PRD 12.11, FR-12.34 to FR-12.38, FR-12.29, FR-12.30, plan 17c): what the garden
//! screens send and read back. A garden is rented by some partners for a season. It is made with its
//! partners and their shares first, then each partner's money source, then the details (PRD
//! FR-12.29). Its fruit is sold in بکری (the garden is the seller of a lot) or in another shop, and
//! when the goods stop coming the Owner closes it and its money goes to its partners by their %
//! (C26).
//!
//! The commands and what each one answers:
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `gardens_defaults` | none | `GardenDefaults` |
//! | `gardens_create_preview` | `GardenInput` | `GardenCreatePreview` |
//! | `gardens_create` | `GardenInput` | `GardenCreated` |
//! | `gardens_set_partners` | `GardenSetPartnersInput` | `GardenRow` |
//! | `gardens_list` | `GardensListInput` | `GardensList` |
//! | `gardens_get` | `GardenGetInput` | `GardenDetail` |
//! | `gardens_summary` | none | `GardensSummary` |
//! | `gardens_entry_preview` / `gardens_entry_save` | `GardenEntryInput` | `GardenEntryPreview` / `GardenEntrySaved` |
//! | `gardens_entry_cancel` | `GardenEntryCancelInput` | `GardenEntryCancelled` |
//! | `gardens_close_preview` / `gardens_close` | `GardenCloseInput` | `GardenClosePreview` / `GardenClosed` |
//! | `gardens_reopen` | `GardenReopenInput` | `GardenReopened` |
//!
//! Lists return one page with the row count and the totals over the whole filter (Code Standards
//! B31, PRD FR-19.11). A garden's number is its own series (1, 2, 3 ...) and its code, the number a
//! screen shows, is `0080`, `0081` ... (type 008, plan 34), the same in both languages, and the code of
//! its account; the screen lists its entries by date, with no entry number (plan 17c, Q3).
//!
//! Money words (the money agent, Section 5.5):
//!
//! - **Running balance** (FR-12.37) is the money in less every live payment, shop and pocket parts
//!   together. Money in is every line on the garden's account (seller credits of its lots, the money
//!   of sales in other shops, a negative credit lowering it), except the closing and its undo.
//! - **Rent due** is the rent less the live installments (costs do not count), signed.
//! - **Closing** (C26): the garden account's balance B is split by the garden's shares with
//!   `split_by_bps`. A negative B is split as a positive amount and every part is made negative
//!   (C11). The garden's account ends at 0. B = 0 posts nothing and the garden still closes (E64).
//! - **A partner's part of the result** is always his money back less what he paid (shop and pocket):
//!   a derived figure, never `split_by_bps` of the result, so the parts add up to the result exactly.
//!
//! Worked numbers (PRD E2E-40): rent 50,000; الف 60% from the shop, ب 40% from his pocket. Installments
//! 30,000 (parts 18,000 / 12,000), then 20,000 (12,000 / 8,000) and labour 5,000 (3,000 / 2,000).
//! Money in 56,400 + 10,000 = 66,400, paid 55,000, running balance 11,400. Closing: 66,400 gives
//! 39,840 / 26,560. الف: 39,840 − 33,000 = 6,840; ب: 26,560 − 22,000 = 4,560.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::cash::CashCheck;
use crate::error::FieldError;
use crate::money::Paisa;
use crate::paging::{MonthSubtotal, SortInput};
use crate::rules::Working;
use crate::ventures::{
    EntryPart, EntryPartInput, EntryParts, ShareAmount, VenturePartnerInput, VenturePartnerRow,
    VentureStatus,
};

// ---- The garden itself ----

/// A new garden (PRD FR-12.29, 12.11). Shares and sources first, then the details. Saving makes its
/// own account under control 0306 with both names (FR-12.34), and the first installment if one is
/// typed.
///
/// Field paths in errors: `name_ur`, `name_en`, `date`, `item_id`, `rent`, `season_end`, `note`,
/// `partners`, `partners[i].partner_id`, `partners[i].share`, `partners[i].source`, `paid_now`,
/// `paid_now_reason`, `parts`, `parts[i].partner_id`, `parts[i].source`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardenInput {
    /// Urdu name. Urdu letters only (PRD FR-7.28).
    pub name_ur: String,
    /// English name. English letters only.
    pub name_en: String,
    /// Words the user confirmed in the spelling check (PRD FR-7.66). `ERR_SPELLING_UNCONFIRMED` on a
    /// name if one is missing.
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
    /// The day, `DD-MM-YYYY` or a shortcut. An open day (PRD FR-6.6).
    pub date: String,
    /// The main fruit. Null if none.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// The full rent (ٹھیکہ) for the season, whole rupees, more than 0.
    pub rent: String,
    /// The last day of the season, `DD-MM-YYYY`. Not before `date`. Null or empty if none.
    #[serde(default)]
    pub season_end: Option<String>,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub note: Option<String>,
    /// The partners in the garden with their shares and default sources. The shares total exactly
    /// 100% (`ERR_SHARES_NOT_100` on `partners`, value `total_bps`). Rows with a share of 0 are
    /// skipped.
    pub partners: Vec<VenturePartnerInput>,
    /// The first installment (پہلی قسط), whole rupees, saved with the garden as an installment
    /// (FR-12.35). Null or empty if none.
    #[serde(default)]
    pub paid_now: Option<String>,
    /// Why the first installment was paid. Required when `paid_now` is typed
    /// (`ERR_REASON_REQUIRED` on `paid_now_reason`). The screen fills it with the word "first
    /// installment" like a typed value (Code Standards F43), and the Owner can change it.
    #[serde(default)]
    pub paid_now_reason: Option<String>,
    /// The source of each partner for the first installment. Empty takes the default sources.
    #[serde(default)]
    pub parts: Vec<EntryPartInput>,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟" when the shop parts of the first
    /// installment take cash below 0 (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// New shares for a garden that has no entry yet (PRD FR-12.29). After its first entry the shares
/// are fixed: `ERR_VENTURE_SHARES_LOCKED` on `partners`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardenSetPartnersInput {
    /// The garden. `ERR_NOT_FOUND` on `id`.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read. `ERR_VERSION_CHANGED` on `version`.
    #[ts(type = "number")]
    pub version: i64,
    /// The partners, as in `GardenInput`.
    pub partners: Vec<VenturePartnerInput>,
}

/// The form of a new garden: every active partner with the firm's share of today and the source
/// `shop` (PRD A56 for gardens, plan 17c Q4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenDefaults {
    /// Every active partner, in the order the partners were created.
    pub partners: Vec<VenturePartnerRow>,
}

/// What the garden screen shows of a new garden before it is saved: the parts of its first
/// installment and the cash question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenCreatePreview {
    /// The parts of the first installment (C23). Null if there is none or it has a mistake.
    pub first_installment: Option<EntryParts>,
    /// What the shop parts do to cash (PRD FR-6.7).
    pub cash: CashCheck,
    /// Mistakes, each at its field. Nothing can be saved while there are any.
    pub errors: Vec<FieldError>,
    /// Notes that do not stop the save.
    pub warnings: Vec<FieldError>,
}

/// The garden just made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenCreated {
    /// The garden with its figures.
    pub row: GardenRow,
    /// The first installment's entry (`venture_entries.id`). Null if none was typed.
    #[ts(type = "number | null")]
    pub first_entry_id: Option<i64>,
    /// Its journal entry. Null if none or if every part was from a pocket (PRD FR-12.30).
    #[ts(type = "number | null")]
    pub journal_entry_id: Option<i64>,
    /// The preview it was saved from.
    pub preview: GardenCreatePreview,
}

/// What the lists of gardens add up to, over the whole filter (B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardensTotals {
    /// Σ rent.
    #[ts(type = "Paisa")]
    pub rent_paisa: Paisa,
    /// Σ live payments (installments and costs, shop and pocket).
    #[ts(type = "Paisa")]
    pub paid_paisa: Paisa,
    /// Σ money in.
    #[ts(type = "Paisa")]
    pub money_in_paisa: Paisa,
    /// Σ running balances (money in less paid).
    #[ts(type = "Paisa")]
    pub balance_paisa: Paisa,
    /// Σ rent due (signed).
    #[ts(type = "Paisa")]
    pub rent_due_paisa: Paisa,
    /// Σ commission on the goods sold in our shop (plan 36), the same definition as the rows.
    #[ts(type = "Paisa")]
    pub commission_paisa: Paisa,
}

/// Input of `gardens_list` (PRD FR-19.10, FR-19.11). Dates are `DD-MM-YYYY` or a shortcut; null or
/// empty means no limit. They filter on the day of the garden.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct GardensListInput {
    /// First day.
    #[serde(default)]
    pub from: Option<String>,
    /// Last day.
    #[serde(default)]
    pub to: Option<String>,
    /// Only open or only closed gardens.
    #[serde(default)]
    pub status: Option<VentureStatus>,
    /// Only gardens this partner is in.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
    /// Text to find in the garden's name (either script).
    #[serde(default)]
    pub search: Option<String>,
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives the default page of 100 rows (PRD FR-19.11).
    #[serde(default)]
    pub limit: u32,
    /// The column to sort the whole filter by: `number`, `code`, `entry_date`, `name_ur`, `name_en`,
    /// `rent_paisa`, `season_end_date`, `status`, `closed_date`, `paid_paisa`, `money_in_paisa`,
    /// `balance_paisa`, `rent_due_paisa` or `commission_paisa`. Null is newest day first (plan 38
    /// R4).
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// True to give the subtotal of each month in `months`. Used only in date order.
    #[serde(default)]
    pub group_by_month: bool,
}

/// One garden with its figures (PRD FR-12.34, FR-12.37). All figures are read from stored rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenRow {
    /// The garden.
    #[ts(type = "number")]
    pub id: i64,
    /// Its version, for `gardens_set_partners`, `gardens_close` and `gardens_reopen`.
    #[ts(type = "number")]
    pub version: i64,
    /// Its number in its own series, from 1.
    #[ts(type = "number")]
    pub number: i64,
    /// Its code, the number a screen shows (plan 34): `0080` for the first garden, `0081` for the
    /// second. Type 008, the number less one, no padding. The garden's account has the same code.
    pub code: String,
    /// Urdu name, as typed.
    pub name_ur: String,
    /// English name, as typed.
    pub name_en: String,
    /// Its own account under control 0306. `code` is the account code.
    pub account: RecordRef,
    /// The day it was made, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The main fruit.
    pub item: Option<RecordRef>,
    /// The full rent for the season.
    #[ts(type = "Paisa")]
    pub rent_paisa: Paisa,
    /// The last day of the season, `YYYY-MM-DD`.
    pub season_end_date: Option<String>,
    /// A note.
    pub note: Option<String>,
    /// Open, or closed by the Owner (PRD FR-12.38).
    pub status: VentureStatus,
    /// The day it was closed, `YYYY-MM-DD`.
    pub closed_date: Option<String>,
    /// True once the garden has an entry: its shares can no longer change (PRD FR-12.29).
    pub shares_locked: bool,
    /// The partners in the garden with their shares and default sources.
    pub partners: Vec<VenturePartnerRow>,
    /// Σ live installments.
    #[ts(type = "Paisa")]
    pub installments_paisa: Paisa,
    /// Σ live running costs (labour, spray, transport).
    #[ts(type = "Paisa")]
    pub costs_paisa: Paisa,
    /// Σ live payments: installments and costs, shop and pocket parts together.
    #[ts(type = "Paisa")]
    pub paid_paisa: Paisa,
    /// Of that, the parts paid from the shop's cash.
    #[ts(type = "Paisa")]
    pub paid_shop_paisa: Paisa,
    /// Of that, the parts paid from pockets.
    #[ts(type = "Paisa")]
    pub paid_pocket_paisa: Paisa,
    /// The rent less the live installments (FR-12.34). Signed: a negative number means more was paid
    /// than the rent.
    #[ts(type = "Paisa")]
    pub rent_due_paisa: Paisa,
    /// Money in: the seller credits of its lots, the money of sales in other shops and the fruit
    /// taken from its lots, less any negative credit. The closing and its undo are left out.
    #[ts(type = "Paisa")]
    pub money_in_paisa: Paisa,
    /// Of that, the money of sales in other shops.
    #[ts(type = "Paisa")]
    pub other_shop_paisa: Paisa,
    /// The running balance (FR-12.37): money in less paid. Starts in minus. After the closing it is
    /// the result (a profit, or a loss when negative).
    #[ts(type = "Paisa")]
    pub balance_paisa: Paisa,
    /// How the balance was worked out (CalcInfo).
    pub balance_working: Working,
    /// The garden account's own balance, credit positive. 0 once the garden is closed.
    #[ts(type = "Paisa")]
    pub account_balance_paisa: Paisa,
    /// The day of the latest entry or lot posting of the garden, `YYYY-MM-DD`: the closing cannot be
    /// dated before it. Null if there is none.
    pub last_date: Option<String>,
    /// The shop's commission on the goods sold in OUR shop (plan 36, PRD FR-12.27, FR-12.37):
    /// seller commission + buyer commission + لاگا of the stored sale lines, with reversals taken
    /// out. A sale in another shop adds nothing. The same number as analytics views J and I and
    /// R36 (not the seller-only column of R33). Display only: nothing posts.
    #[ts(type = "Paisa")]
    pub commission_paisa: Paisa,
    /// How the commission was worked out: the three parts and their sum (CalcInfo).
    pub commission_working: Working,
}

/// The answer of `gardens_list`: one page and the totals over the whole filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardensList {
    /// This page: newest day first, then by number.
    pub rows: Vec<GardenRow>,
    /// How many gardens match the filter.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over all of them.
    pub totals: GardensTotals,
    /// The count and subtotals of each month of the whole filter (the day), in the order of the
    /// list, when asked with `group_by_month` and the list is in date order. Figures: `rent_paisa`.
    pub months: Vec<MonthSubtotal>,
}

/// Input of `gardens_get`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardenGetInput {
    /// The garden. `ERR_NOT_FOUND` on `id`.
    #[ts(type = "number")]
    pub id: i64,
    /// How many rows of `history` to skip. 0 for the first page.
    #[serde(default)]
    pub history_offset: u32,
    /// How many rows to return. 0 gives the default page of 100.
    #[serde(default)]
    pub history_limit: u32,
}

/// What a row of a garden's history is (PRD FR-7.67: the type list of the garden screen, FR-12.37).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum GardenHistoryKind {
    /// قسط: a part of the rent, paid on a date.
    Installment,
    /// باغ خرچ: a running cost (labour, spray, transport).
    Cost,
    /// دوسری دکان میں فروخت: goods sold in another shop.
    OtherSale,
    /// بکری: the seller credit of a lot of the garden sold in our shop (negative if the lot owes).
    Sale,
    /// پھل: fruit taken from a lot of the garden (plan 17d).
    Fruit,
    /// The closing (FR-12.38). It moves the balance out to the partners and is not money in.
    Close,
}

/// One row of a garden's history, oldest first by day, with the running balance after it (PRD
/// FR-12.37).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenHistoryRow {
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// What it is.
    pub kind: GardenHistoryKind,
    /// The garden entry (`venture_entries.id`): for an installment, a cost, a sale in another shop
    /// and the closing. `gardens_entry_cancel` takes it. Null for a sale or fruit row.
    #[ts(type = "number | null")]
    pub entry_id: Option<i64>,
    /// Its version, for a cancel.
    #[ts(type = "number | null")]
    pub version: Option<i64>,
    /// The بکری of a sale row: id and number.
    #[ts(type = "number | null")]
    pub bikri_id: Option<i64>,
    /// Its number.
    #[ts(type = "number | null")]
    pub bikri_number: Option<i64>,
    /// The entry's own amount: the payment, or the money received, or the closing balance without
    /// its sign. For a sale row, the signed seller credit.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
    /// What the row does to the running balance: minus for a payment, plus for money in, 0 for the
    /// closing.
    #[ts(type = "Paisa")]
    pub effect_paisa: Paisa,
    /// The running balance after this row (FR-12.37).
    #[ts(type = "Paisa")]
    pub running_balance_paisa: Paisa,
    /// Why (an installment or a cost).
    pub reason: Option<String>,
    /// The shop, as typed (a sale in another shop).
    pub shop_name: Option<String>,
    /// نگ sold (a sale in another shop).
    #[ts(type = "number | null")]
    pub qty: Option<i64>,
    /// The journal entry. Null when every part of a payment was from a pocket, and for a closing
    /// with nothing to move (PRD FR-12.30, E64).
    #[ts(type = "number | null")]
    pub journal_entry_id: Option<i64>,
    /// True once it was cancelled; it stays in the list and counts for nothing.
    pub cancelled: bool,
    /// Why it was cancelled.
    pub cancel_reason: Option<String>,
    /// The parts of a payment (C23), with their sources. Empty for the other kinds.
    pub parts: Vec<EntryPart>,
    /// The parts of the closing (C26), signed. Empty for the other kinds.
    pub credits: Vec<ShareAmount>,
    /// True if `gardens_entry_cancel` would accept it now, apart from a closed day. False while the
    /// garden is closed and for sale and fruit rows.
    pub can_cancel: bool,
}

/// One partner's money in a garden and what he got back (PRD FR-12.38, FR-17.10, E2E-40).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenPartnerResult {
    /// The partner. `code` is empty.
    pub partner: RecordRef,
    /// His share in bps.
    pub share_bps: i32,
    /// What he paid from the shop's cash: it came off his کل رقم when it was paid.
    #[ts(type = "Paisa")]
    pub paid_shop_paisa: Paisa,
    /// What he paid from his own pocket: recorded only.
    #[ts(type = "Paisa")]
    pub paid_pocket_paisa: Paisa,
    /// His part of the closing (C26), signed. 0 while the garden is open.
    #[ts(type = "Paisa")]
    pub money_back_paisa: Paisa,
    /// His part of the result: money back less what he paid, shop and pocket. While the garden is
    /// open it is minus what he paid. E2E-40: 39,840 − 33,000 = 6,840.
    #[ts(type = "Paisa")]
    pub net_paisa: Paisa,
    /// How the net was worked out (CalcInfo).
    pub working: Working,
}

/// One garden with its history and each partner's money (PRD FR-7.67, FR-12.37, FR-12.38).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenDetail {
    /// The garden and its figures.
    pub row: GardenRow,
    /// One page of its history, oldest first, each row with the running balance after it.
    pub history: Vec<GardenHistoryRow>,
    /// How many rows the history has in all.
    #[ts(type = "number")]
    pub history_count: i64,
    /// Each partner's money in and money back.
    pub partner_results: Vec<GardenPartnerResult>,
}

/// One open garden on the dashboard tile باغات (PRD 9.1, FR-12.37).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenSummaryRow {
    /// The garden.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number in its own series, from 1.
    #[ts(type = "number")]
    pub number: i64,
    /// Its code (plan 34): `0080` for the first garden.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Its running balance.
    #[ts(type = "Paisa")]
    pub balance_paisa: Paisa,
    /// The rent still due (signed).
    #[ts(type = "Paisa")]
    pub rent_due_paisa: Paisa,
    /// Its commission on the goods sold in our shop (plan 36).
    #[ts(type = "Paisa")]
    pub commission_paisa: Paisa,
}

/// The data of the dashboard tile باغات (PRD 9.1): the open gardens and each one's running balance.
/// D5 shows the tile; the shape is kept stable for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardensSummary {
    /// How many gardens are open.
    #[ts(type = "number")]
    pub open_gardens: i64,
    /// Σ running balances of the open gardens.
    #[ts(type = "Paisa")]
    pub balance_total_paisa: Paisa,
    /// Σ commission on the goods sold in our shop, over the open gardens (plan 36).
    #[ts(type = "Paisa")]
    pub commission_paisa: Paisa,
    /// The open gardens, by number.
    pub rows: Vec<GardenSummaryRow>,
}

// ---- Installments, running costs and sales in another shop (PRD FR-12.35, FR-12.36) ----

/// An installment or a running cost of a garden, as typed. It is paid in as in FR-12.30: split by the
/// garden's % (C23), each partner's part from the shop's cash or from his pocket.
///
/// Field paths in errors: `garden_id`, `date`, `amount`, `reason`, `parts`, `parts[i].partner_id`,
/// `parts[i].source`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardenPaymentInput {
    /// The garden. `ERR_NOT_FOUND`; `ERR_GARDEN_CLOSED`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub garden_id: Option<i64>,
    /// The day, `DD-MM-YYYY`. An open day, not before the garden's day (`ERR_DATE_BEFORE_GARDEN`).
    pub date: String,
    /// Whole rupees, more than 0.
    pub amount: String,
    /// Why. Required (`ERR_REASON_REQUIRED`).
    pub reason: String,
    /// The source of each partner for this entry. Empty takes the garden's default sources.
    #[serde(default)]
    pub parts: Vec<EntryPartInput>,
    /// True after the user said yes to "کیا یہ اندراج درست ہے؟" when the shop parts take cash below
    /// 0 (PRD FR-6.7).
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// A sale of the garden's goods in another shop, as typed (PRD FR-12.36). The money comes into
/// روکڑ and goes to the garden's account; nothing is split before the closing.
///
/// Field paths in errors: `garden_id`, `date`, `shop`, `qty`, `amount`, `note`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardenOtherSaleInput {
    /// The garden. `ERR_NOT_FOUND`; `ERR_GARDEN_CLOSED`.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub garden_id: Option<i64>,
    /// The day, `DD-MM-YYYY`. An open day, not before the garden's day.
    pub date: String,
    /// The shop, as typed (PRD FR-7.25).
    #[serde(default)]
    pub shop: Option<String>,
    /// The garden arrival (مال آیا) the goods come from (PRD FR-12.41). The qty comes off that
    /// arrival's part for another shop (`ERR_QTY_EXCEEDS_LOT` on `qty`, value `available`). Required
    /// when the garden has any arrival (`ERR_REQUIRED` on `arrival_id`); for a garden with no arrival
    /// it stays empty and the sale is typed by hand as before. The arrival must belong to this garden
    /// (`ERR_FORMAT_INVALID`) and stand (`ERR_ARRIVAL_CANCELLED`).
    #[serde(default)]
    #[ts(type = "number | null", optional)]
    pub arrival_id: Option<i64>,
    /// نگ sold, a whole number more than 0.
    pub qty: String,
    /// The money received, whole rupees, more than 0.
    pub amount: String,
    /// A note, up to 500 characters.
    #[serde(default)]
    pub note: Option<String>,
}

/// One entry of the type list of the garden screen (PRD FR-7.67): قسط, باغ خرچ or
/// دوسری دکان میں فروخت.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GardenEntryInput {
    /// قسط: a part of the rent.
    Installment(GardenPaymentInput),
    /// باغ خرچ: a running cost.
    Cost(GardenPaymentInput),
    /// دوسری دکان میں فروخت.
    OtherSale(GardenOtherSaleInput),
}

/// What an entry does, worked out by the same function as the save (Code Standards B5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenEntryNumbers {
    /// The amount: the payment, or the money received.
    #[ts(type = "Paisa")]
    pub amount: Paisa,
    /// The parts of a payment (C23). Null for a sale in another shop.
    pub parts: Option<EntryParts>,
    /// The running balance before this entry.
    #[ts(type = "Paisa")]
    pub balance_before_paisa: Paisa,
    /// The running balance after it.
    #[ts(type = "Paisa")]
    pub balance_after_paisa: Paisa,
    /// The rent due after it (an installment lowers it; a cost or a sale does not).
    #[ts(type = "Paisa")]
    pub rent_due_after_paisa: Paisa,
    /// How the balance after was worked out (CalcInfo).
    pub balance_working: Working,
    /// Only for a sale in another shop linked to an arrival (PRD FR-12.41, plan 35 D1), else null:
    /// the نگ the sale moves from our shop's part to the part for another shop. 0 when the part for
    /// another shop has enough.
    #[ts(type = "number | null")]
    pub qty_from_our_shop: Option<i64>,
    /// Same case: نگ left of the arrival in all after this sale.
    #[ts(type = "number | null")]
    pub qty_left_after: Option<i64>,
    /// Same case: نگ left for another shop after this sale.
    #[ts(type = "number | null")]
    pub qty_other_left_after: Option<i64>,
    /// Same case: نگ left for our shop after this sale.
    #[ts(type = "number | null")]
    pub qty_our_left_after: Option<i64>,
}

/// The answer of `gardens_entry_preview`: the numbers of the entry, nothing saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenEntryPreview {
    /// What kind of entry this is.
    pub kind: GardenHistoryKind,
    /// The numbers. Null if there is a mistake.
    pub numbers: Option<GardenEntryNumbers>,
    /// What the entry does to cash (PRD FR-6.7): the shop parts of a payment take cash out, the
    /// money of a sale in another shop brings it in.
    pub cash: CashCheck,
    /// Mistakes, each at its field. Nothing can be saved while there are any.
    pub errors: Vec<FieldError>,
    /// Notes that do not stop the save.
    pub warnings: Vec<FieldError>,
}

/// An entry saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenEntrySaved {
    /// The entry (`venture_entries.id`).
    #[ts(type = "number")]
    pub id: i64,
    /// Its number in the series of its kind.
    #[ts(type = "number")]
    pub number: i64,
    /// Its version, for a cancel.
    #[ts(type = "number")]
    pub version: i64,
    /// Its journal entry. Null when every part of a payment was from a pocket (PRD FR-12.30).
    #[ts(type = "number | null")]
    pub journal_entry_id: Option<i64>,
    /// The preview it was saved from.
    pub preview: GardenEntryPreview,
}

/// Cancels an entry of a garden by reversal while its day is open (PRD FR-15.3 style). Refused while
/// the garden is closed (`ERR_GARDEN_CLOSED`): undo the closing first.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardenEntryCancelInput {
    /// The entry (`GardenHistoryRow::entry_id`). `ERR_NOT_FOUND` on `id`.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read. `ERR_VERSION_CHANGED` on `version`.
    #[ts(type = "number")]
    pub version: i64,
    /// Why. Required (`ERR_REASON_REQUIRED`).
    pub reason: String,
    /// True after the user said yes when taking the money of a sale back lowers cash below 0.
    #[serde(default)]
    pub confirm_cash_below_zero: bool,
}

/// An entry cancelled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenEntryCancelled {
    /// The entry.
    #[ts(type = "number")]
    pub id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub number: i64,
    /// The reversal's journal entry. Null if the entry had none.
    #[ts(type = "number | null")]
    pub reversal_entry_id: Option<i64>,
}

// ---- Closing (PRD FR-12.38) ----

/// Closes a garden (PRD FR-12.38), after the confirm box of FR-7.8. Used by the preview and the save.
///
/// Field paths in errors: `id`, `version`, `date`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardenCloseInput {
    /// The garden. `ERR_NOT_FOUND` on `id`; `ERR_GARDEN_CLOSED` if it is closed already.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read. `ERR_VERSION_CHANGED` on `version`.
    #[ts(type = "number")]
    pub version: i64,
    /// The closing day, `DD-MM-YYYY`. An open day, not before the latest entry or lot posting of the
    /// garden (`ERR_CLOSE_BEFORE_LAST_ENTRY`, value `last_date`).
    pub date: String,
}

/// What a closing does, worked out by the same function as the save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenCloseNumbers {
    /// The garden account's balance B, credit positive (C26). The garden's money in, less any
    /// negative credit.
    #[ts(type = "Paisa")]
    pub balance_paisa: Paisa,
    /// Σ live payments of the garden: the total cost.
    #[ts(type = "Paisa")]
    pub total_cost_paisa: Paisa,
    /// The money in (FR-12.37).
    #[ts(type = "Paisa")]
    pub money_in_paisa: Paisa,
    /// The result: money in less the total cost. Negative is a loss.
    #[ts(type = "Paisa")]
    pub result_paisa: Paisa,
    /// Each partner's part of the closing (C26), signed, in the order the partners were created. The
    /// parts add up to `balance_paisa`.
    pub credits: Vec<ShareAmount>,
    /// Each partner's money in and money back with this closing.
    pub partner_results: Vec<GardenPartnerResult>,
    /// False when `balance_paisa` is 0: nothing is posted, the garden still closes (E64).
    pub posts_entry: bool,
}

/// The answer of `gardens_close_preview`: the numbers of the closing, nothing saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenClosePreview {
    /// The numbers. Null if there is a mistake.
    pub numbers: Option<GardenCloseNumbers>,
    /// The earliest day the garden can be closed on, `YYYY-MM-DD`: its latest entry or lot posting.
    /// Null if there is none.
    pub earliest_date: Option<String>,
    /// Mistakes, each at its field. Nothing can be saved while there are any.
    pub errors: Vec<FieldError>,
    /// Notes that do not stop the save.
    pub warnings: Vec<FieldError>,
}

/// A garden closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenClosed {
    /// The garden with its figures after the closing.
    pub row: GardenRow,
    /// The closing entry (`venture_entries.id`).
    #[ts(type = "number")]
    pub close_entry_id: i64,
    /// Its journal entry. Null when the balance was 0.
    #[ts(type = "number | null")]
    pub journal_entry_id: Option<i64>,
    /// The preview it was saved from.
    pub preview: GardenClosePreview,
}

/// Undoes the closing of a garden by reversal while the closing day is open (PRD FR-12.38).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct GardenReopenInput {
    /// The garden. `ERR_NOT_FOUND` on `id`; `ERR_GARDEN_NOT_CLOSED` if it is open.
    #[ts(type = "number")]
    pub id: i64,
    /// The version read. `ERR_VERSION_CHANGED` on `version`.
    #[ts(type = "number")]
    pub version: i64,
    /// Why. Required (`ERR_REASON_REQUIRED`).
    pub reason: String,
}

/// A closing undone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct GardenReopened {
    /// The garden with its figures: open again, its account holding its money again.
    pub row: GardenRow,
    /// The reversal's journal entry. Null if the closing had none.
    #[ts(type = "number | null")]
    pub reversal_entry_id: Option<i64>,
}
