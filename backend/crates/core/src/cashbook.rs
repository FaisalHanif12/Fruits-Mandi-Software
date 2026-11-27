//! The روکڑ بک (PRD 16.1, FR-16.1, FR-16.14 v1.60, FR-19.11, plan 17, plan 99): the shop's one cash
//! account, every cash in and out from all modules, one page at a time. There is no cash count box
//! (FR-16.1). Every row is named for what it is ([`CashKind`]), with the records it is about
//! ([`CashRefs`]).
//!
//! Command `cashbook_get`: input `CashbookInput`, answer `Cashbook`. Read only.
//!
//! Rows:
//! - A journal entry that moves cash is one row, except a بکری posting, which gives one row for the
//!   money the buyers paid on the spot (موقع پر جمع, in) and one for the نقد given to the sellers
//!   (نقد ادائیگی, out). The پیشگی typed on a lot is its own entry and its own row.
//! - By default an edited or cancelled entry shows once, in the form in force: the old posting and
//!   its الٹا اندراج are left out. With `show_reversed` they show too. A reversal is posted on the
//!   day of the entry it reverses, so leaving the pair out changes no day-end balance.
//! - The cards and the totals are over the entries in force, so they are the same either way.
//!
//! Worked numbers (PRD E2E-8, FR-16.14): after Events 1 to 5 the rows of 10-10-26 are بکری 1 نقد
//! ادائیگی −5,000; بکری 2 موقع پر جمع +12,800; وصولی 1 +10,000; ادائیگی 1 −4,316; دکان خرچہ 1
//! −3,000; ذاتی خرچہ 1 −2,000; closing 58,484.

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
    /// "پرانے اور منسوخ بھی دکھائیں" (PRD FR-16.14 v1.60): true shows the reversed postings and their
    /// الٹا اندراج too. False, the default, shows each edited entry once, in the form in force, and a
    /// cancelled entry not at all. The cards and the totals are the same either way.
    #[serde(default)]
    pub show_reversed: bool,
    /// The type filter (PRD FR-16.14 v1.60): only rows of these kinds. Null or empty is every kind
    /// (سب). The screen takes the kinds of a group from `Cashbook.summary`. With a filter the rows
    /// have no running cash (`running_paisa` null), and `totals` are of the filtered rows.
    #[serde(default)]
    #[ts(optional)]
    pub kinds: Option<Vec<CashKind>>,
}

/// What one row of the روکڑ بک is (PRD FR-16.14 v1.60). The screen turns it into words with Fluent,
/// with the numbers and names of the row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CashKind {
    /// The opening typed on the cash account itself (ابتدائی روکڑ).
    OpeningCash,
    /// A partner's opening capital that came into cash (`refs.partner`).
    OpeningCapital,
    /// The ابتدائی بقایا of a party: a seller, a buyer, a person of money lent or a الگ کھاتہ
    /// (`refs.parties`), posted against روکڑ (PRD FR-10.9 v1.60).
    OpeningBalance,
    /// وصولی (`number`, `refs.parties`).
    Receipt,
    /// ادائیگی to a seller or a payable account (`number`, `refs.parties`).
    Payment,
    /// پیشگی to a seller (`number`, `refs.parties`).
    Advance,
    /// پیشگی to a buyer, saved as cash lent (PRD FR-15.11 v1.60): `number` is its number in the
    /// ادائیگی series.
    BuyerAdvance,
    /// نقد ادھار: cash lent to a buyer from the ادائیگی tab (`number`, `refs.parties`).
    CashLent,
    /// ادھار رقم given (`number`, `refs.parties`).
    MoneyLent,
    /// ادھار رقم returned in cash (`number`, `refs.parties`).
    MoneyLentReturn,
    /// دکان خرچہ (`number`, `refs.expense_head`).
    ShopExpense,
    /// ذاتی خرچہ (`number`, `refs.partner`).
    PersonalExpense,
    /// ذاتی خرچہ واپسی (`number`, `refs.partner`).
    PersonalReturn,
    /// موقع پر جمع of a بکری: what its buyers paid on the spot (`number` the بکری, `refs.parties` the
    /// buyers).
    BikriPaid,
    /// نقد ادائیگی of a بکری: the نقد given to its sellers (`number` the بکری, `refs.parties` the
    /// sellers).
    BikriSellerCash,
    /// پیشگی typed on a lot of a بکری (`number` the بکری, `refs.parties` the sellers).
    BikriAdvance,
    /// باغ کی قسط (`number` the instalment, `refs.garden`).
    GardenInstallment,
    /// باغ خرچ (`number` the cost, `refs.garden`).
    GardenCost,
    /// دوسری دکان میں فروخت of a garden (`number`, `refs.garden`).
    GardenOtherSale,
    /// رعایت واپسی from a garden's owner (PRD FR-12.51; `number`, `refs.garden`).
    GardenRefund,
    /// خریداری of a market trip (`number` the purchase خ-1, `refs.trip`).
    Purchase,
    /// سفر خرچ (`number`, `refs.trip`).
    TripCost,
    /// دوسری دکان میں فروخت of a market trip (`number`, `refs.trip`).
    TripOtherSale,
    /// سرمایہ جمع (`number`, `refs.partner`).
    CapitalIn,
    /// سرمایہ واپسی (`number`, `refs.partner`).
    CapitalOut,
    /// الگ کھاتہ: رقم دی (`number`, `refs.parties`).
    KhataOut,
    /// الگ کھاتہ: رقم آئی (`number`, `refs.parties`).
    KhataIn,
    /// جنرل with a cash line (`number`, `refs.parties` the other accounts).
    General,
    /// پھل (no cash moves on the پھل screen today; kept so a cash line of it would still be named).
    Fruit,
    /// Anything else.
    Other,
}

/// The choices of the type filter of the روکڑ بک (PRD FR-16.14 v1.60): سب is no choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CashGroup {
    /// وصولی.
    Receipt,
    /// ادائیگی.
    Payment,
    /// پیشگی (sellers and buyers).
    Advance,
    /// ادھار (ادھار رقم and نقد ادھار).
    Lent,
    /// خرچہ (shop and personal).
    Expense,
    /// بکری (موقع پر جمع, نقد ادائیگی, پیشگی of a lot).
    Bikri,
    /// باغ.
    Garden,
    /// سفر.
    Trip,
    /// سرمایہ (capital in and out, and a partner's opening capital).
    Capital,
    /// الگ کھاتہ.
    Khata,
    /// ابتدائی بقایا (ابتدائی روکڑ and the openings of parties).
    Opening,
    /// دیگر.
    Other,
}

impl CashGroup {
    /// Every group, in the order of the filter.
    pub const ALL: [CashGroup; 12] = [
        CashGroup::Receipt,
        CashGroup::Payment,
        CashGroup::Advance,
        CashGroup::Lent,
        CashGroup::Expense,
        CashGroup::Bikri,
        CashGroup::Garden,
        CashGroup::Trip,
        CashGroup::Capital,
        CashGroup::Khata,
        CashGroup::Opening,
        CashGroup::Other,
    ];
}

impl CashKind {
    /// Every kind, in the order of the filter groups.
    pub const ALL: [CashKind; 30] = [
        CashKind::Receipt,
        CashKind::Payment,
        CashKind::Advance,
        CashKind::BuyerAdvance,
        CashKind::CashLent,
        CashKind::MoneyLent,
        CashKind::MoneyLentReturn,
        CashKind::ShopExpense,
        CashKind::PersonalExpense,
        CashKind::PersonalReturn,
        CashKind::BikriPaid,
        CashKind::BikriSellerCash,
        CashKind::BikriAdvance,
        CashKind::GardenInstallment,
        CashKind::GardenCost,
        CashKind::GardenOtherSale,
        CashKind::GardenRefund,
        CashKind::Purchase,
        CashKind::TripCost,
        CashKind::TripOtherSale,
        CashKind::CapitalIn,
        CashKind::CapitalOut,
        CashKind::OpeningCapital,
        CashKind::KhataOut,
        CashKind::KhataIn,
        CashKind::OpeningCash,
        CashKind::OpeningBalance,
        CashKind::General,
        CashKind::Fruit,
        CashKind::Other,
    ];

    /// The group of the type filter this kind is in.
    ///
    /// ```
    /// use mandi_core::cashbook::{CashGroup, CashKind};
    /// assert_eq!(CashKind::BuyerAdvance.group(), CashGroup::Advance);
    /// assert_eq!(CashKind::CashLent.group(), CashGroup::Lent);
    /// ```
    pub const fn group(self) -> CashGroup {
        match self {
            CashKind::Receipt => CashGroup::Receipt,
            CashKind::Payment => CashGroup::Payment,
            CashKind::Advance | CashKind::BuyerAdvance => CashGroup::Advance,
            CashKind::CashLent | CashKind::MoneyLent | CashKind::MoneyLentReturn => CashGroup::Lent,
            CashKind::ShopExpense | CashKind::PersonalExpense | CashKind::PersonalReturn => {
                CashGroup::Expense
            }
            CashKind::BikriPaid | CashKind::BikriSellerCash | CashKind::BikriAdvance => {
                CashGroup::Bikri
            }
            CashKind::GardenInstallment
            | CashKind::GardenCost
            | CashKind::GardenOtherSale
            | CashKind::GardenRefund => CashGroup::Garden,
            CashKind::Purchase | CashKind::TripCost | CashKind::TripOtherSale => CashGroup::Trip,
            CashKind::CapitalIn | CashKind::CapitalOut | CashKind::OpeningCapital => {
                CashGroup::Capital
            }
            CashKind::KhataOut | CashKind::KhataIn => CashGroup::Khata,
            CashKind::OpeningCash | CashKind::OpeningBalance => CashGroup::Opening,
            CashKind::General | CashKind::Fruit | CashKind::Other => CashGroup::Other,
        }
    }

    /// The storage-free form used by the queries, the same as the serialised name.
    pub const fn as_str(self) -> &'static str {
        match self {
            CashKind::OpeningCash => "opening_cash",
            CashKind::OpeningCapital => "opening_capital",
            CashKind::OpeningBalance => "opening_balance",
            CashKind::Receipt => "receipt",
            CashKind::Payment => "payment",
            CashKind::Advance => "advance",
            CashKind::BuyerAdvance => "buyer_advance",
            CashKind::CashLent => "cash_lent",
            CashKind::MoneyLent => "money_lent",
            CashKind::MoneyLentReturn => "money_lent_return",
            CashKind::ShopExpense => "shop_expense",
            CashKind::PersonalExpense => "personal_expense",
            CashKind::PersonalReturn => "personal_return",
            CashKind::BikriPaid => "bikri_paid",
            CashKind::BikriSellerCash => "bikri_seller_cash",
            CashKind::BikriAdvance => "bikri_advance",
            CashKind::GardenInstallment => "garden_installment",
            CashKind::GardenCost => "garden_cost",
            CashKind::GardenOtherSale => "garden_other_sale",
            CashKind::GardenRefund => "garden_refund",
            CashKind::Purchase => "purchase",
            CashKind::TripCost => "trip_cost",
            CashKind::TripOtherSale => "trip_other_sale",
            CashKind::CapitalIn => "capital_in",
            CashKind::CapitalOut => "capital_out",
            CashKind::KhataOut => "khata_out",
            CashKind::KhataIn => "khata_in",
            CashKind::General => "general",
            CashKind::Fruit => "fruit",
            CashKind::Other => "other",
        }
    }

    /// Reads [`CashKind::as_str`] back.
    pub fn parse(text: &str) -> Option<CashKind> {
        CashKind::ALL.into_iter().find(|k| k.as_str() == text)
    }
}

/// Whether the posting of a row is in force (PRD FR-6.3, FR-16.14). Rows that are not in force show
/// only with `show_reversed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CashRowStatus {
    /// In force.
    Live,
    /// An old posting that an edit or a cancel reversed.
    Reversed,
    /// The الٹا اندراج itself.
    Reversal,
}

/// The most names a row gives in `refs.parties`; `refs.party_count` says how many there are.
pub const MAX_ROW_PARTIES: usize = 3;

/// The records a row is about, so the screen can name it (PRD FR-16.14 v1.60). Empty or null when
/// the kind has none.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, TS)]
pub struct CashRefs {
    /// The parties: the account of a voucher or an opening, the buyers or sellers of a بکری row, the
    /// other accounts of a جنرل. At most [`MAX_ROW_PARTIES`], in line order. A چکتا buyer shows by
    /// his own names (with the account 0030).
    pub parties: Vec<RecordRef>,
    /// How many parties the row has, all of them.
    #[ts(type = "number")]
    pub party_count: i64,
    /// The partner of a capital, personal or opening capital row. `code` is empty.
    pub partner: Option<RecordRef>,
    /// The garden of a garden row (`id` is `ventures.id`, `code` its code `0080`...).
    pub garden: Option<RecordRef>,
    /// The market trip of a trip row (`id` is `ventures.id`, `code` its code `0070`...).
    pub trip: Option<RecordRef>,
    /// The expense head of a دکان خرچہ.
    pub expense_head: Option<RecordRef>,
}

/// One row of cash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CashbookRow {
    /// The journal entry. A بکری posting can give two rows with the same entry: `entry_id` with
    /// `kind` is unique.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// The posting type in its stored form: `bikri_first`, `bikri_later`, `bikri_advance` (the
    /// پیشگی of a lot), `reversal`, `receipt`, `payment`, `advance`, `shop_expense`,
    /// `personal_expense`, `personal_return`, `money_lent`, `money_lent_return`, `capital_in`,
    /// `capital_out`, `khata_out`, `khata_in`, `purchase`, `trip_cost`, `garden_payment`,
    /// `venture_receipt`, `fruit_taken`, `general`, `opening`.
    pub posting_type: String,
    /// What the row is (a reversal has the kind of the posting it reverses).
    pub kind: CashKind,
    /// The filter group of `kind`.
    pub group: CashGroup,
    /// In force, reversed, or the reversal.
    pub status: CashRowStatus,
    /// The number the row is named with: a voucher's, a بکری's, a purchase's (خ-1) or a garden or
    /// trip entry's. Null when the row has none (an opening).
    #[ts(type = "number | null")]
    pub number: Option<i64>,
    /// The records the row is about.
    pub refs: CashRefs,
    /// The note. Empty if none.
    pub narration: String,
    /// Cash in (0 if the row is cash out).
    #[ts(type = "Paisa")]
    pub in_paisa: Paisa,
    /// Cash out (0 if the row is cash in).
    #[ts(type = "Paisa")]
    pub out_paisa: Paisa,
    /// Cash in hand after this row (signed: cash can go below 0, PRD FR-6.7). Null when the list
    /// is sorted by a column other than the day, or filtered by kind: the running cash shows in
    /// day order over every row only (plan 38 B2). With `show_reversed` it counts the rows shown.
    #[ts(type = "Paisa | null")]
    pub running_paisa: Option<Paisa>,
    /// The record the row opens, so a row opens its entry (PRD FR-19.4): a بکری, a voucher, a
    /// trip purchase, a trip or a garden (for its cost, installment, sale in another shop or
    /// closing), fruit taken, a month close or an account (an opening). Null only if the entry has
    /// no source. An entry that belongs to no screen of its own (`kind` `entry`) opens itself.
    pub source: Option<RowSource>,
}

/// What the rows add up to, over the whole range and filter (B31), the postings in force only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CashbookTotals {
    /// Σ cash in.
    #[ts(type = "Paisa")]
    pub in_paisa: Paisa,
    /// Σ cash out.
    #[ts(type = "Paisa")]
    pub out_paisa: Paisa,
}

/// In and out of one type over the whole date range, the postings in force only. It does not narrow
/// with the `kinds` filter, so the filter can show every type with its figures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CashGroupSummary {
    /// The type.
    pub group: CashGroup,
    /// Its kinds: send these as `kinds` to filter by this type.
    pub kinds: Vec<CashKind>,
    /// How many rows in force it has in the range.
    #[ts(type = "number")]
    pub count: i64,
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
    /// Card کل سرمایہ (PRD FR-16.14 v1.60): capital put in less capital taken back, opening capital
    /// included, in every entry dated up to the window's end (`to`, or today). Signed.
    #[ts(type = "Paisa")]
    pub total_capital_paisa: Paisa,
    /// Card شروع میں روکڑ: cash in hand at the end of the day before `from` (0 with no `from`).
    #[ts(type = "Paisa")]
    pub opening_paisa: Paisa,
    /// Card آخر میں روکڑ: cash in hand at the end of the range. With no `to` it is the tile
    /// موجود روکڑ.
    #[ts(type = "Paisa")]
    pub closing_paisa: Paisa,
    /// The day of the first entry that ever moved cash (not only in this range), `YYYY-MM-DD`, or
    /// null when cash never moved. The window screen goes back to it (plan 78 E, PRD FR-16.14).
    pub first_entry_date: Option<String>,
    /// The page, by day and posting order (a بکری posting: موقع پر جمع then نقد ادائیگی).
    pub rows: Vec<CashbookRow>,
    /// How many rows there are in the range and filter.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The totals over the whole range and filter.
    pub totals: CashbookTotals,
    /// In and out by type over the whole range: every group, in filter order, also one with
    /// nothing.
    pub summary: Vec<CashGroupSummary>,
    /// The count and subtotals of the months THIS PAGE touches (each whole month, not only its rows
    /// on the page), in the order of the list, when asked with `group_by_month` and the list is in
    /// day order. Figures: `in_paisa` and `out_paisa`.
    pub months: Vec<MonthSubtotal>,
}
