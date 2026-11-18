//! The types of the commission page (plan 76 point 7, plan 77, PRD FR-9.3).
//!
//! One page reads a period (one day, a month, a year or a range of days) and, when asked, the
//! commission and sale of one party in it. The shapes follow the screen: a row for each day (a
//! row for each month in a year), a totals row over the whole period, and the figures the screen
//! shows, never worked out again on the screen (CLAUDE.md rule 1).

use mandi_core::auth::Language;
use mandi_core::money::Paisa;
use mandi_core::printing::Paper;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::analytics::AnalyticsGoods;
use crate::reports::PartyRef;

/// What a period is (PRD FR-9.3, plan 77): one date, a month, a year or a range of dates.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CommissionPeriodKind {
    /// One date: its one row, with its detail under it.
    Day,
    /// One calendar month: a row for each day that has a figure (the default).
    #[default]
    Month,
    /// One calendar year: a row for each month that has a figure.
    Year,
    /// A range of dates: a row for each day that has a figure, one page at a time (Code Standards
    /// B31).
    Range,
}

/// The period as the screen sends it. Each kind reads its own fields and refuses a missing one.
///
/// Example: `{ kind: "year", year: 2026 }` or `{ kind: "range", from: "01-10-2026", to: "10-10-2026" }`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct CommissionPeriod {
    /// What the period is. Month when left out.
    pub kind: CommissionPeriodKind,
    /// The date of a day: `YYYY-MM-DD`, `DD-MM-YYYY` or the shortcuts of PRD FR-7.19.
    #[ts(optional)]
    pub date: Option<String>,
    /// The month of a month, `YYYY-MM`. Null is the current month.
    #[ts(optional)]
    pub month: Option<String>,
    /// The year of a year, for example 2026.
    #[ts(type = "number | null", optional)]
    pub year: Option<i32>,
    /// The first date of a range.
    #[ts(optional)]
    pub from: Option<String>,
    /// The last date of a range.
    #[ts(optional)]
    pub to: Option<String>,
}

/// Which side of the sale a party is on (PRD FR-9.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CommissionSide {
    /// A buyer, گاہک.
    Buyer,
    /// A seller, بیوپاری.
    Seller,
}

/// One party to read the commission of, with his side (PRD FR-9.3, plan 77).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CommissionParty {
    /// Buyer or seller. The account must be of that side (`ERR_WRONG_PARTY_TYPE`).
    pub side: CommissionSide,
    /// The account.
    #[ts(type = "number")]
    pub account_id: i64,
    /// One person of the چکتا کھاتہ account (a buyer only). Null means the whole account.
    #[serde(default)]
    #[ts(optional)]
    pub chukta_person_key: Option<String>,
}

/// The party the page was read for, with his names (to head the page and its print).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CommissionPartyShown {
    /// Buyer or seller.
    pub side: CommissionSide,
    /// The account, with his code, names and tag; for a چکتا buyer his person key and names.
    pub party: PartyRef,
}

/// The period the page was read for, as dates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CommissionPeriodShown {
    /// What it is.
    pub kind: CommissionPeriodKind,
    /// The first day, `YYYY-MM-DD`.
    pub from: String,
    /// The last day, `YYYY-MM-DD`.
    pub to: String,
}

/// Input of `dashboard_commission_month`.
///
/// `month` alone (today's input) is a month. `period` wins when both are sent.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct CommissionMonthInput {
    /// The month, `YYYY-MM`. Null or empty is the current month. A month after the current one is
    /// refused (`ERR_DATE_IN_FUTURE` on `month`).
    #[serde(default)]
    pub month: Option<String>,
    /// The period: a day, a month, a year or a range (plan 77). Null means the month above.
    #[serde(default)]
    #[ts(optional)]
    pub period: Option<CommissionPeriod>,
    /// One party to read the commission of. Null is the whole shop (plan 76).
    #[serde(default)]
    #[ts(optional)]
    pub party: Option<CommissionParty>,
    /// Which goods are counted (plan 78 D4, PRD FR-9.3): all (the default), consigned goods, trip
    /// goods or garden goods, as the goods filter of analytics (PRD 18.1). Under a goods filter the
    /// shared income and the partner columns are not shown.
    #[serde(default)]
    #[ts(optional)]
    pub goods: Option<AnalyticsGoods>,
    /// Rows to skip: only a range is paged.
    #[serde(default)]
    #[ts(type = "number | null", optional)]
    pub offset: Option<u32>,
    /// Rows to return: only a range is paged. Null or 0 gives the default page of 100.
    #[serde(default)]
    #[ts(type = "number | null", optional)]
    pub limit: Option<u32>,
}

/// A partner of the page: always sent, because names are not behind the partner code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CommissionPartner {
    /// The partner.
    #[ts(type = "number")]
    pub id: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
}

/// One partner's income share of a day or of the month.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartnerShare {
    /// The partner.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// His income share, whole rupees.
    #[ts(type = "Paisa")]
    pub share_paisa: Paisa,
}

/// The figures of one day, of one month of a year, or of the whole period (the totals row, `date`
/// empty).
///
/// Whole shop: seller, buyer, لاگا, total, shared income and the partner shares (plan 76).
/// Under a seller: `sale_paisa` is his خام بکری, `seller_comm_paisa` the commission charged to
/// him, `buyer_comm_paisa` and `laga_paisa` what the buyers of his lots were charged, and
/// `total_comm_paisa` = `goods_comm_paisa` the commission on his goods (all three added).
/// Under a buyer: `sale_paisa` is the value he bought, `buyer_comm_paisa` and `laga_paisa` what
/// he was charged, `total_comm_paisa` the two added, and `seller_comm_paisa` is 0. Under either,
/// the shared income is 0 and there are no partner shares (they belong to whole postings).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CommissionDayRow {
    /// The day, `YYYY-MM-DD`; the month, `YYYY-MM`, in a year. Empty on the totals row.
    pub date: String,
    /// Seller commission.
    #[ts(type = "Paisa")]
    pub seller_comm_paisa: Paisa,
    /// Buyer commission.
    #[ts(type = "Paisa")]
    pub buyer_comm_paisa: Paisa,
    /// لاگا.
    #[ts(type = "Paisa")]
    pub laga_paisa: Paisa,
    /// Seller commission + buyer commission + لاگا (under a buyer, buyer commission + لاگا).
    #[ts(type = "Paisa")]
    pub total_comm_paisa: Paisa,
    /// Shared income: every income share row of the day.
    #[ts(type = "Paisa")]
    pub shared_income_paisa: Paisa,
    /// Each partner's income share, in the order of `partners`. Null while the partner section is
    /// locked (PRD FR-17.11), and under a party filter.
    pub partner_shares: Option<Vec<PartnerShare>>,
    /// Under a party: his sale (a seller) or the value he bought (a buyer). Null for the shop.
    #[ts(type = "Paisa | null")]
    pub sale_paisa: Option<Paisa>,
    /// Under a seller: the commission on his goods (seller + buyer commission + لاگا of his
    /// lots), the same as `total_comm_paisa`. Null otherwise.
    #[ts(type = "Paisa | null")]
    pub goods_comm_paisa: Option<Paisa>,
}

/// What `dashboard_commission_month` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CommissionMonth {
    /// The month of the first day of the period, `YYYY-MM` (the month asked for, for a month).
    pub month: String,
    /// The year of the first day of the period.
    pub year: i32,
    /// The period the rows are for.
    pub period: CommissionPeriodShown,
    /// The party the rows are for; null for the whole shop.
    pub party: Option<CommissionPartyShown>,
    /// The goods the rows are for (plan 78 D4). Under a goods filter `partners` is empty and every
    /// row has no partner shares and no shared income.
    pub goods: AnalyticsGoods,
    /// The first month with a commission fact, or null when there is none. The back button stops
    /// here.
    pub first_month: Option<String>,
    /// The current month, where the next button stops.
    pub current_month: String,
    /// The partners, always sent. Empty under a party filter.
    pub partners: Vec<CommissionPartner>,
    /// The rows of this page, oldest first: one for each day (a day, a month, a range) or each
    /// month (a year) that has a figure.
    pub rows: Vec<CommissionDayRow>,
    /// How many rows the whole period has.
    #[ts(type = "number")]
    pub total_count: i64,
    /// Rows skipped: the first row of this page is row `offset` of the period.
    #[ts(type = "number")]
    pub offset: u32,
    /// The page size used (a range is paged; the other kinds send every row).
    #[ts(type = "number")]
    pub limit: u32,
    /// The totals row over the whole period, never a sum of the page.
    pub totals: CommissionDayRow,
    /// True when the partner columns are hidden because the partner section is locked.
    pub partners_locked: bool,
}

impl CommissionMonth {
    /// Hides every partner share, while the partner section is locked (PRD FR-17.11). The shared
    /// income stays: it is the shop's, not a partner's. A party page has no partner columns.
    pub fn hide_partners(&mut self) {
        if self.party.is_some() || self.goods != AnalyticsGoods::All {
            return;
        }
        for row in &mut self.rows {
            row.partner_shares = None;
        }
        self.totals.partner_shares = None;
        self.partners_locked = true;
    }
}

/// Input of `dashboard_commission_day`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CommissionDayInput {
    /// The day, `YYYY-MM-DD` (as the rows give it) or `DD-MM-YYYY`.
    pub date: String,
    /// How many entries to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many entries to return. 0 gives the default page of 100.
    #[serde(default)]
    pub limit: u32,
    /// Only this party's lines (plan 77). Null is the whole day.
    #[serde(default)]
    #[ts(optional)]
    pub party: Option<CommissionParty>,
    /// Only these goods (plan 78 D4). Null is all goods.
    #[serde(default)]
    #[ts(optional)]
    pub goods: Option<AnalyticsGoods>,
}

/// One posting behind a day's commission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CommissionEntry {
    /// The journal entry.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// Its posting type, for example `bikri_first`, `bikri_later` or `reversal`.
    pub posting_type: String,
    /// What it belongs to: `bikri`, `voucher` and so on.
    pub source_type: String,
    /// The row of that source.
    #[ts(type = "number")]
    pub source_id: i64,
    /// The بکری, when the posting is of one.
    #[ts(type = "number | null")]
    pub bikri_id: Option<i64>,
    /// Its number.
    #[ts(type = "number | null")]
    pub bikri_number: Option<i64>,
    /// The sellers of its lines (under a party, only the lines of that party).
    pub sellers: Vec<PartyRef>,
    /// The buyers of its lines (under a party, only the lines of that party).
    pub buyers: Vec<PartyRef>,
    /// The figures of this posting (`date` is the day).
    pub figures: CommissionDayRow,
}

/// What `dashboard_commission_day` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CommissionDay {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The party the postings are for; null for the whole day.
    pub party: Option<CommissionPartyShown>,
    /// The goods the postings are for (plan 78 D4).
    pub goods: AnalyticsGoods,
    /// The partners, always sent. Empty under a party filter or a goods filter.
    pub partners: Vec<CommissionPartner>,
    /// One page of the postings of the day, oldest first.
    pub entries: Vec<CommissionEntry>,
    /// How many postings the day has.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The day's figures: the row of the month page for this day.
    pub totals: CommissionDayRow,
    /// True when the partner columns are hidden.
    pub partners_locked: bool,
}

impl CommissionDay {
    /// Hides every partner share, while the partner section is locked. A party page has none.
    pub fn hide_partners(&mut self) {
        if self.party.is_some() || self.goods != AnalyticsGoods::All {
            return;
        }
        for entry in &mut self.entries {
            entry.figures.partner_shares = None;
        }
        self.totals.partner_shares = None;
        self.partners_locked = true;
    }
}

/// Input of `dashboard_commission_print_data` and of the sheet of kind `commission_month`: the
/// period, the party, the print language.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct CommissionPrintInput {
    /// The month, `YYYY-MM`; null is the current month. `period` wins when both are sent.
    #[serde(default)]
    pub month: Option<String>,
    /// The period (plan 77).
    #[serde(default)]
    #[ts(optional)]
    pub period: Option<CommissionPeriod>,
    /// One party (plan 77); the print header names the side and the party.
    #[serde(default)]
    #[ts(optional)]
    pub party: Option<CommissionParty>,
    /// Which goods (plan 78 D4); null is all goods. The header names a goods filter.
    #[serde(default)]
    #[ts(optional)]
    pub goods: Option<AnalyticsGoods>,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    pub paper: Option<Paper>,
}
