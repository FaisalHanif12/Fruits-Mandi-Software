//! The روکڑ بک (PRD 16.1, FR-16.1, FR-19.11, plan 17): the shop's one cash account, every cash in
//! and out from all modules, one page at a time. There is no cash count box (FR-16.1).
//!
//! Command `cashbook_get`: input `CashbookInput`, answer `Cashbook`. Read only.
//!
//! Worked numbers (PRD E2E-8): after Events 1 to 5 the closing cash is 50,000 − 5,000 + 12,800 +
//! 10,000 − 4,316 − 3,000 − 2,000 = 58,484.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::RecordRef;
use crate::money::Paisa;
use crate::paging::{MonthSubtotal, SortInput};
use crate::row_source::RowSource;

/// Input of `cashbook_get`. Dates are `DD-MM-YYYY` or a shortcut; null or empty means no limit
/// (from the first entry, to today).
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct CashbookInput {
    /// First day.
    #[serde(default)]
    pub from: Option<String>,
    /// Last day.
    #[serde(default)]
    pub to: Option<String>,
    /// How many rows to skip. 0 for the first page.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return. 0 gives the default page of 100 rows (PRD FR-19.11).
    #[serde(default)]
    pub limit: u32,
    /// The column to sort the whole range by: `entry_date`, `posting_type`, `number`, `narration`,
    /// `in_paisa` or `out_paisa`. Null is by day and entry, oldest first (plan 38 R4). Only the day
    /// order (either way) has the running cash: for any other column `running_paisa` is null.
    #[serde(default)]
    pub sort: Option<SortInput>,
    /// True to give the subtotal of each month in `months`. Used only in date order.
    #[serde(default)]
    pub group_by_month: bool,
}

/// What a cash row is, for the rows that start the book or move a partner's capital (plan 78 E,
/// PRD FR-16.14). The posting type says what the entry is; this says it in the words the Owner
/// reads: the opening cash, a partner's opening capital, and capital put in or taken back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CashbookKind {
    /// The opening balance typed on the cash account itself (ابتدائی روکڑ).
    OpeningCash,
    /// A partner's opening capital that came into cash (`partner` says whose).
    OpeningCapital,
    /// Capital a partner put in (سرمایہ جمع).
    CapitalIn,
    /// Capital a partner took back (سرمایہ واپسی).
    CapitalOut,
    /// Any other row: a sale, a voucher, an expense and the rest. `posting_type` says which.
    Other,
}

/// One cash movement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CashbookRow {
    /// The journal entry.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The posting type in its stored form: `bikri_first`, `bikri_later`, `reversal`, `receipt`,
    /// `payment`, `advance`, `shop_expense`, `personal_expense`, `personal_return`, `money_lent`,
    /// `money_lent_return`, `capital_in`, `capital_out`, `khata_out`, `khata_in`, `purchase`,
    /// `trip_cost`, `garden_payment`, `venture_receipt`, `garden_close`, `fruit_taken`, `general`,
    /// `opening`, `month_close`. The screen turns it into words with Fluent.
    pub posting_type: String,
    /// The number of the source (a بکری or a voucher), or null.
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The other account of the entry (for example the buyer of a وصولی), or null when there are
    /// several.
    pub account: Option<RecordRef>,
    /// What the row is when it opens the book or moves a partner's capital (plan 78 E). `other` for
    /// every other row.
    pub kind: CashbookKind,
    /// The partner of an opening capital, capital in or capital out row. Null for every other row.
    /// `code` is empty: a partner has no code.
    pub partner: Option<RecordRef>,
    /// The note. Empty if none.
    pub narration: String,
    /// Cash in (0 if the row is cash out).
    #[ts(type = "Paisa")]
    pub in_paisa: Paisa,
    /// Cash out (0 if the row is cash in).
    #[ts(type = "Paisa")]
    pub out_paisa: Paisa,
    /// Cash in hand after this row (signed: cash can go below 0, PRD FR-6.7). Null when the list
    /// is sorted by a column other than the day: the running cash shows in day order only (plan
    /// 38 B2). The screen then says "balance shows in date order only".
    #[ts(type = "Paisa | null")]
    pub running_paisa: Option<Paisa>,
    /// The record the row opens, so a row opens its entry (PRD FR-19.4): a بکری, a voucher, a
    /// trip purchase, a trip or a garden (for its cost, installment, sale in another shop or
    /// closing), fruit taken, a month close or an account (an opening). Null only if the entry has
    /// no source. An entry that belongs to no screen of its own (`kind` `entry`) opens itself.
    pub source: Option<RowSource>,
}

/// What the range adds up to, over the whole range (B31).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CashbookTotals {
    /// Σ cash in.
    #[ts(type = "Paisa")]
    pub in_paisa: Paisa,
    /// Σ cash out.
    #[ts(type = "Paisa")]
    pub out_paisa: Paisa,
}

/// The cash book of a range, one page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Cashbook {
    /// Cash in hand at the start of the range (signed).
    #[ts(type = "Paisa")]
    pub opening_paisa: Paisa,
    /// Cash in hand at the end of the range (signed).
    #[ts(type = "Paisa")]
    pub closing_paisa: Paisa,
    /// The day of the first entry that ever moved cash (not only in this range), `YYYY-MM-DD`, or
    /// null when cash never moved. The window screen goes back to it (plan 78 E, PRD FR-16.14).
    pub first_entry_date: Option<String>,
    /// The page, by day and posting order.
    pub rows: Vec<CashbookRow>,
    /// How many rows there are in the range.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over the whole range.
    pub totals: CashbookTotals,
    /// The count and subtotals of the months THIS PAGE touches (each whole month, not only its rows
    /// on the page), in the order of the list, when asked with `group_by_month` and the list is in
    /// day order. Figures: `in_paisa` and `out_paisa`.
    pub months: Vec<MonthSubtotal>,
}

// ---- Where the capital is (PRD FR-16.14 v1.43, plan 78 E) ----

/// What a line of "where the partners' money is now" is (the left side of the card).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CapitalWhereKind {
    /// روکڑ: cash in hand.
    Cash,
    /// What the buyers owe the shop, the dead amount included (the ledger balances of every buyer).
    BuyersOwe,
    /// Advances with sellers, still open (PRD FR-15.8).
    SellerAdvances,
    /// What sellers owe beyond their open advances (PRD FR-15.9).
    SellersOweBeyond,
    /// Minus what the shop owes sellers (their open payables, negative).
    SellersPayable,
    /// ادھار رقم: money lent and not yet returned.
    MoneyLent,
    /// Minus what is owed to labour, transport, the munshi and stores (head 02, negative).
    OwedForCharges,
    /// الگ کھاتے: separate khatas.
    SeparateKhatas,
    /// Minus the gardens' money held until closing (negative).
    GardensHeld,
    /// Minus linked money already in the partners' totals (the contra account, negative).
    LinkedInPartnerTotals,
    /// Any other account of the left side. Only when not 0.
    Other,
}

/// One line of the left side. Signed: money the shop has or is owed is plus, what it owes is minus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalWhereLine {
    /// What it is.
    pub kind: CapitalWhereKind,
    /// The amount.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
}

/// What a line of "whose it is" (the right side of the card) is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CapitalWhoseKind {
    /// A partner's کل رقم (`partner` says whose).
    PartnerTotal,
    /// ابتدائی سرمایہ (the firm's, account 06010001).
    OpeningCapital,
    /// The partners' parts of the advances to sellers still open (PRD FR-17.13).
    AdvancesOpen,
    /// ابتدائی فرق when not 0: "رقم کا مقام درج نہیں" (where the money is was not recorded).
    UnrecordedWhere,
    /// Any other difference. Only when not 0; the books balance, so it shows a bug.
    Other,
}

/// One line of the right side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalWhoseLine {
    /// What it is.
    pub kind: CapitalWhoseKind,
    /// The partner of a `partner_total` line. `code` is empty.
    pub partner: Option<RecordRef>,
    /// The amount.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
}

/// The card on the روکڑ page: where the partners' money is, and whose it is. Behind the partner code
/// (`ERR_PARTNER_LOCKED`). The two totals are always equal, because the trial balance balances.
///
/// Example (after Events 1 to 5): 58,484 + 21,564 − 10,160 − 4,400 = 65,488 on the left; partners
/// −1,004 + 746 + 746 and ابتدائی سرمایہ 65,000 on the right.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CapitalWhere {
    /// Where it is.
    pub where_lines: Vec<CapitalWhereLine>,
    /// Σ of the left side.
    #[ts(type = "Paisa")]
    pub where_total_paisa: Paisa,
    /// Whose it is.
    pub whose_lines: Vec<CapitalWhoseLine>,
    /// Σ of the right side.
    #[ts(type = "Paisa")]
    pub whose_total_paisa: Paisa,
}
