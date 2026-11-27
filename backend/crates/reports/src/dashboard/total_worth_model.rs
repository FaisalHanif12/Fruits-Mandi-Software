//! The types of the tile and the page کل مالیت (Total Worth, PRD FR-9.10, v1.61, plan 100).

use mandi_core::auth::Language;
use mandi_core::money::Paisa;
use mandi_core::printing::Paper;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::reports::PartyRef;

/// The tile کل مالیت: the money to have, the money to give and the net of them (have less give).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, TS)]
pub struct TotalWorthTile {
    /// Money to have.
    #[ts(type = "Paisa")]
    pub have_paisa: Paisa,
    /// Money to give.
    #[ts(type = "Paisa")]
    pub give_paisa: Paisa,
    /// Have less give. Can be below 0.
    #[ts(type = "Paisa")]
    pub net_paisa: Paisa,
}

/// The two sections of the page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TotalWorthSide {
    /// Money to have.
    #[default]
    Have,
    /// Money to give.
    Give,
}

/// Where a row of the page comes from. The first nine are money to have, the rest money to give,
/// in the order of the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TotalWorthSource {
    /// روکڑ: one row (it can be below 0, PRD FR-6.7).
    Cash,
    /// گاہکوں کے ذمے: the rows of R20.
    BuyersDues,
    /// بیوپاریوں کے ذمے: what sellers owe beyond their open advances.
    SellersDues,
    /// ڈوبی رقم: the rows of R38 still owed.
    DeadAmount,
    /// ادھار رقم باقی: the open money lent, people in the dead amount left out.
    MoneyLent,
    /// پیشگی باقی: the part of a seller's open advances that his payable does not cover.
    OpenAdvances,
    /// الگ کھاتے with a minus balance (they owe the shop).
    KhataMinus,
    /// Gardens whose account is بنام (their charges are more than their sales).
    GardenMinus,
    /// Any other بنام balance of the books not in a list above (rare, for example an old amount on
    /// an old account). It keeps the net equal to the books.
    OtherHave,
    /// بیوپاریوں کو ادائیگی باقی: each seller's net payable.
    SellersPayable,
    /// مزدوری, کرایہ, کرایہ سٹور and منشیانہ still to pay.
    Payables,
    /// Buyers with a جمع balance.
    BuyersCredit,
    /// الگ کھاتے with a plus balance (the shop holds money for them).
    KhataPlus,
    /// Gardens whose account holds money not yet given to their partners (paid at closing).
    GardenPlus,
    /// Any other جمع balance of the books not in a list above (rare).
    OtherGive,
}

impl TotalWorthSource {
    /// Every source, in the order of the page: money to have first.
    pub const ALL: [TotalWorthSource; 15] = [
        TotalWorthSource::Cash,
        TotalWorthSource::BuyersDues,
        TotalWorthSource::SellersDues,
        TotalWorthSource::DeadAmount,
        TotalWorthSource::MoneyLent,
        TotalWorthSource::OpenAdvances,
        TotalWorthSource::KhataMinus,
        TotalWorthSource::GardenMinus,
        TotalWorthSource::OtherHave,
        TotalWorthSource::SellersPayable,
        TotalWorthSource::Payables,
        TotalWorthSource::BuyersCredit,
        TotalWorthSource::KhataPlus,
        TotalWorthSource::GardenPlus,
        TotalWorthSource::OtherGive,
    ];

    /// The side the source belongs to.
    #[must_use]
    pub const fn side(self) -> TotalWorthSide {
        match self {
            Self::Cash
            | Self::BuyersDues
            | Self::SellersDues
            | Self::DeadAmount
            | Self::MoneyLent
            | Self::OpenAdvances
            | Self::KhataMinus
            | Self::GardenMinus
            | Self::OtherHave => TotalWorthSide::Have,
            Self::SellersPayable
            | Self::Payables
            | Self::BuyersCredit
            | Self::KhataPlus
            | Self::GardenPlus
            | Self::OtherGive => TotalWorthSide::Give,
        }
    }
}

/// One party (or the cash) of the page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TotalWorthRow {
    /// Its source.
    pub source: TotalWorthSource,
    /// The account, with its code, names and tag, to open its history. For `cash` it is روکڑ.
    pub party: PartyRef,
    /// The amount. Above 0, except the cash row, which can be below 0.
    #[ts(type = "Paisa")]
    pub amount_paisa: Paisa,
}

/// The total of one source over all its parties (not only the page).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct TotalWorthPart {
    /// The source.
    pub source: TotalWorthSource,
    /// Its side.
    pub side: TotalWorthSide,
    /// Its total. Equals its own tile or report (buyers' dues = the R20 total, and so on).
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
    /// How many rows it has.
    #[ts(type = "number")]
    pub count: i64,
}

/// Input of `dashboard_total_worth`: one page of the rows of one side.
#[derive(Debug, Clone, Default, Deserialize, TS)]
pub struct TotalWorthInput {
    /// The section.
    #[serde(default)]
    pub side: TotalWorthSide,
    /// How many rows to skip.
    #[serde(default)]
    pub offset: u32,
    /// How many rows to return, at most 100. 0 gives 100.
    #[serde(default)]
    pub limit: u32,
}

/// What `dashboard_total_worth` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TotalWorthPage {
    /// Today, `YYYY-MM-DD`.
    pub today: String,
    /// The side of `rows`.
    pub side: TotalWorthSide,
    /// The page of rows of that side, by source in the order of [`TotalWorthSource::ALL`], then
    /// by Urdu name.
    pub rows: Vec<TotalWorthRow>,
    /// How many rows that side has in all.
    #[ts(type = "number")]
    pub total_count: i64,
    /// The offset asked.
    pub offset: u32,
    /// The page size used.
    pub limit: u32,
    /// Every source of both sides that has at least one row, in the order of the page, with its
    /// total.
    pub parts: Vec<TotalWorthPart>,
    /// The two section totals and the net: the same figures as the tile.
    pub totals: TotalWorthTile,
}

/// Input of `dashboard_total_worth_print_data` and of the Excel sheet: every row of both sides.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct TotalWorthPrintInput {
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    pub paper: Option<Paper>,
}
