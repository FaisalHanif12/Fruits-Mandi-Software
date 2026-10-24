//! The types of the dashboard and the attention list (PRD 9, FR-7.44). Every name is a Rust name
//! with a `Dashboard` or `Attention` prefix, so the generated TypeScript files do not clash with
//! the report types.
//!
//! Money is `Paisa` (whole rupees, exported to TypeScript as `Paisa`), counts and ids are `i64`
//! (exported as `number`). Nothing here is text for the screen: the screen turns each kind into a
//! Fluent message (Code Standards F31).

use mandi_core::money::Paisa;
use serde::Serialize;
use ts_rs::TS;

use crate::reports::{Names, PartyRef};

/// Most rows of today's بکری list (PRD 9.3). The count of all of them is given as well.
pub const MAX_TODAY_ROWS: usize = 100;
/// How many of the oldest buyer dues the dashboard lists (PRD 9.3).
pub const OLDEST_DUES: usize = 10;
/// How many days the first chart shows, today included (PRD 9.4).
pub const CHART_DAYS: i64 = 30;
/// How many items the second chart shows (PRD 9.4).
pub const TOP_ITEMS: usize = 5;
/// A lot waits in باقی مال for more than this many days before it is on the attention list
/// (PRD FR-7.44, assumption A40).
pub const STOCK_WAIT_DAYS: i64 = 2;
/// A buyer's oldest open bill is on the attention list when it is older than this many days.
pub const DUES_OLD_DAYS: i64 = 30;
/// A seller's open payable is on the attention list when part of it is older than this many days.
pub const PAYABLES_OLD_DAYS: i64 = 7;
/// The cloud copy is on the attention list when it is older than this many hours (NFR-BAK-10).
pub const BACKUP_OLD_HOURS: i64 = 24;

/// What the command layer knows about the backups and the database does not: the backup state is
/// kept by the backup service. Passed in, never read from a file here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackupFacts {
    /// True when the cloud copy is switched on. If it is off, an old copy is not a problem.
    pub cloud_on: bool,
    /// Whole hours since the last good cloud copy. `None` when there is none.
    pub cloud_age_hours: Option<i64>,
}

/// The tiles of PRD 9.1. The garden tile comes with the gardens (D4R).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct DashboardTiles {
    /// آج کی بکری: Σ خام بکری of today's postings (reversals take out cancelled ones).
    #[ts(type = "Paisa")]
    pub sales_gross_paisa: Paisa,
    /// How many بکری have sales today. A later sale today of an older بکری counts here, but that
    /// بکری is not in today's list, which holds the بکری dated today. The two can differ on
    /// purpose; do not make one match the other.
    #[ts(type = "number")]
    pub sales_count: i64,
    /// آج کا کمیشن: seller + buyer commission + لاگا today.
    #[ts(type = "Paisa")]
    pub commission_paisa: Paisa,
    /// روکڑ بقایا: the balance of the cash account now.
    #[ts(type = "Paisa")]
    pub cash_in_hand_paisa: Paisa,
    /// وصولی باقی: the shop's total اگراہی, the total row of R20 (FR-19.9).
    #[ts(type = "Paisa")]
    pub buyers_dues_paisa: Paisa,
    /// ادائیگی باقی: Σ of each seller's open payable (FR-7.59), the payable total of R24.
    #[ts(type = "Paisa")]
    pub sellers_payable_paisa: Paisa,
    /// پیشگی باقی: Σ of the advances not yet adjusted (FR-15.8), the open total of R26.
    #[ts(type = "Paisa")]
    pub advances_open_paisa: Paisa,
    /// باقی مال: lots of saved بکری with goods left.
    #[ts(type = "number")]
    pub stock_lots: i64,
    /// باقی مال: their نگ in all.
    #[ts(type = "number")]
    pub stock_qty: i64,
    /// مارکیٹ سفر: open trips.
    #[ts(type = "number")]
    pub trips_open: i64,
    /// مارکیٹ سفر: saved purchases that still have stock.
    #[ts(type = "number")]
    pub trips_purchases_with_stock: i64,
    /// مارکیٹ سفر: نگ left over all of them.
    #[ts(type = "number")]
    pub trips_qty_left: i64,
    /// مارکیٹ سفر: their cost left (C21).
    #[ts(type = "Paisa")]
    pub trips_cost_left_paisa: Paisa,
    /// ادھار رقم باقی: Σ of the open money lent (FR-16.5).
    #[ts(type = "Paisa")]
    pub lent_open_paisa: Paisa,
    /// How much of it is past its return-by day.
    #[ts(type = "Paisa")]
    pub lent_overdue_paisa: Paisa,
    /// آج کا خرچہ: دکان خرچہ today (the 0501 heads only).
    #[ts(type = "Paisa")]
    pub expenses_today_paisa: Paisa,
}

/// One partner of the strip (PRD 9.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DashboardPartner {
    /// The partner.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Today's profit share: income shares less expense shares (C12).
    #[ts(type = "Paisa")]
    pub today_paisa: Paisa,
    /// This calendar month's profit share up to today.
    #[ts(type = "Paisa")]
    pub month_paisa: Paisa,
    /// This calendar year's profit share up to today.
    #[ts(type = "Paisa")]
    pub year_paisa: Paisa,
    /// کل رقم: everything he has in the firm (C22).
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
}

/// One row of today's بکری list (PRD 9.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DashboardBikri {
    /// The بکری, to open it.
    #[ts(type = "number")]
    pub bikri_id: i64,
    /// Its number.
    #[ts(type = "number")]
    pub number: i64,
    /// The truck number as typed. Null for none.
    pub truck_no: Option<String>,
    /// The seller of its first lot (the one-time person's name for a پختہ کھاتہ lot).
    pub seller: Option<Names>,
    /// How many sellers it has. 1 for a truck with one seller.
    #[ts(type = "number")]
    pub seller_count: i64,
    /// خام بکری of all its lots.
    #[ts(type = "Paisa")]
    pub gross_paisa: Paisa,
    /// صافی بکری of all its lots.
    #[ts(type = "Paisa")]
    pub net_paisa: Paisa,
    /// True when it was cancelled (منسوخ).
    pub cancelled: bool,
}

/// One of the oldest buyer dues (PRD 9.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DashboardDue {
    /// The buyer (and the one-time person inside a پختہ کھاتہ account).
    pub party: PartyRef,
    /// His city.
    pub city: Option<Names>,
    /// What he owes.
    #[ts(type = "Paisa")]
    pub balance_paisa: Paisa,
    /// The day of his oldest open bill, `YYYY-MM-DD`.
    pub oldest_date: Option<String>,
    /// How many days old that bill is.
    #[ts(type = "number | null")]
    pub days: Option<i64>,
}

/// One day of the first chart: خام بکری and commission (PRD 9.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DashboardChartDay {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// خام بکری of the day.
    #[ts(type = "Paisa")]
    pub gross_paisa: Paisa,
    /// Seller + buyer commission + لاگا of the day.
    #[ts(type = "Paisa")]
    pub commission_paisa: Paisa,
}

/// One item of the second chart: this month's commission by item (PRD 9.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DashboardChartItem {
    /// The item.
    #[ts(type = "number")]
    pub item_id: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Its commission this month.
    #[ts(type = "Paisa")]
    pub commission_paisa: Paisa,
}

/// What needs attention (PRD FR-7.44), one kind for each line of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AttentionKind {
    /// Unsaved drafts.
    Drafts,
    /// Earlier days that are not closed.
    OpenDays,
    /// Lots waiting in باقی مال for more than 2 days.
    StockWaiting,
    /// Buyers over their credit limit.
    OverCreditLimit,
    /// Buyer dues older than 30 days.
    DuesOld,
    /// Seller payables older than 7 days.
    PayablesOld,
    /// The last cloud backup is older than 24 hours, or there is none.
    BackupOld,
    /// Money lent past its return-by day.
    LoansOverdue,
    /// Money a partner took past its return-by day.
    TakesOverdue,
    /// Records with a missing Urdu or English name.
    NamesMissing,
}

impl AttentionKind {
    /// Every kind, in the order of the list on the screen.
    pub const ALL: [AttentionKind; 10] = [
        AttentionKind::Drafts,
        AttentionKind::OpenDays,
        AttentionKind::StockWaiting,
        AttentionKind::OverCreditLimit,
        AttentionKind::DuesOld,
        AttentionKind::PayablesOld,
        AttentionKind::BackupOld,
        AttentionKind::LoansOverdue,
        AttentionKind::TakesOverdue,
        AttentionKind::NamesMissing,
    ];
}

/// What the screen opens for the first record of an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AttentionTarget {
    /// An account (a buyer or a seller): its ledger. With a person key for a one-time person.
    Account,
    /// A partner.
    Partner,
    /// A بکری.
    Bikri,
    /// A trip purchase.
    Purchase,
}

/// The most urgent record of an item, to open straight from the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AttentionFirst {
    /// What kind of record it is.
    pub target: AttentionTarget,
    /// Its id.
    #[ts(type = "number")]
    pub id: i64,
    /// The person inside a پختہ کھاتہ account.
    pub person_key: Option<String>,
    /// Urdu name.
    pub name_ur: Option<String>,
    /// English name.
    pub name_en: Option<String>,
}

/// One line of the attention list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AttentionItem {
    /// What it is.
    pub kind: AttentionKind,
    /// How many there are. Always more than 0: a kind with nothing to do is not listed.
    #[ts(type = "number")]
    pub count: i64,
    /// The money involved, where there is some.
    #[ts(type = "Paisa | null")]
    pub amount_paisa: Option<Paisa>,
    /// The نگ involved, where there are some.
    #[ts(type = "number | null")]
    pub qty: Option<i64>,
    /// The most days past (or old), where that means something.
    #[ts(type = "number | null")]
    pub days: Option<i64>,
    /// The most urgent record.
    pub first: Option<AttentionFirst>,
}

/// The attention list and its count (PRD FR-7.44). Read only (Code Standards B29).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AttentionList {
    /// The lines, in the order of [`AttentionKind::ALL`], only those with something to do.
    pub items: Vec<AttentionItem>,
    /// The number on the bell: Σ of the counts.
    #[ts(type = "number")]
    pub total_count: i64,
}

/// Everything the dashboard shows (PRD 9). Read only; one signed-in call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Dashboard {
    /// Today, `YYYY-MM-DD`.
    pub today: String,
    /// The tiles.
    pub tiles: DashboardTiles,
    /// The partner strip, in the order the partners were made.
    pub partners: Vec<DashboardPartner>,
    /// Today's بکری, oldest first, at most [`MAX_TODAY_ROWS`].
    pub bikri_today: Vec<DashboardBikri>,
    /// How many بکری there are today in all.
    #[ts(type = "number")]
    pub bikri_today_count: i64,
    /// The oldest buyer dues, at most [`OLDEST_DUES`].
    pub oldest_dues: Vec<DashboardDue>,
    /// The last [`CHART_DAYS`] days, oldest first, today last.
    pub chart_days: Vec<DashboardChartDay>,
    /// This month's commission by item, the biggest first, at most [`TOP_ITEMS`].
    pub chart_items: Vec<DashboardChartItem>,
    /// The attention list (آج کے کام).
    pub attention: AttentionList,
}
