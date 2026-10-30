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
