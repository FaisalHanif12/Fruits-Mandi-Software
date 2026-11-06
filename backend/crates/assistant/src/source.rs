//! The filters of a source, with whole records (lead decision, plan 48 T1).
//!
//! A button under an answer opens a report or an analytics view with the filters the model read it
//! with. The screens' pickers hold a whole value (id, code, both names, tag), so every filter
//! that names a record carries the whole record, not a bare id. The panel puts the record into
//! the address as a picker value; the screen shows the name at once, with no extra lookup.
//!
//! [`SourceReportFilters`] and [`SourceAnalyticsFilters`] have the same fields as the report
//! engine's [`ReportFilters`] and the analytics [`AnalyticsFilters`], except that each `*_id`
//! field is a record. `to_filters` turns them back into the engine's own filters (the ids), so
//! the tools and the buttons use one description of a source.
//!
//! Example (worked day): R13 of اکرم ٹول ٹیکس is
//! `party: { record: { id: 20, code: "0020", name_ur: "اکرم ٹول ٹیکس", name_en: "Akram Toll Tax",
//! tag: null }, person_key: null }`; `to_filters()` gives `party: { account_id: 20 }`.

use mandi_reports::analytics::{
    AnalyticsCompare, AnalyticsFilters, AnalyticsGoods, AnalyticsGroupBy, AnalyticsPeriod,
    AnalyticsSide, AnalyticsTripView,
};
use mandi_reports::reports::{
    DateShortcut, GardenView, GroupBy, PaidFilter, PartyFilter, ReportFilters, TripView,
};
use mandi_services::core::rules::ItemGroup;
use mandi_services::core::vouchers::VoucherType;
use serde::Serialize;
use ts_rs::TS;

/// A whole record, as a picker holds it: id, code, both names and the tag. The screen draws the
/// name through `displayName`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnswerRecord {
    /// The record's id (an account, an item, a city, a garden ...).
    #[ts(type = "number")]
    pub id: i64,
    /// Its code, for example `0020`. Null for a record with no code.
    pub code: Option<String>,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The tag after the name (PRD 10.2). Null for none.
    pub tag: Option<String>,
}

/// A party filter: the account as a whole record and, for a پختہ کھاتہ account, the person
/// (PRD FR-13.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SourceParty {
    /// The account.
    pub record: AnswerRecord,
    /// The person key inside a پختہ کھاتہ account. Null for the whole account.
    pub person_key: Option<String>,
}

impl SourceParty {
    fn to_filter(&self) -> PartyFilter {
        PartyFilter {
            account_id: self.record.id,
            person_key: self.person_key.clone(),
        }
    }
}

/// The one بکری of the sale book report (R1): its id and the number the Owner knows. The screen
/// never shows the id in place of the number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct SourceBikri {
    /// The row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The بکری number.
    #[ts(type = "number")]
    pub number: i64,
}

/// The filters of a report source, the same as [`ReportFilters`] with whole records.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, TS)]
pub struct SourceReportFilters {
    /// A date shortcut. When set, `from` and `to` are ignored.
    pub shortcut: Option<DateShortcut>,
    /// First day, `YYYY-MM-DD`. Empty means from the first day of data.
    pub from: String,
    /// Last day, `YYYY-MM-DD`. Empty means up to the last day of data.
    pub to: String,
    /// The day of a balance list (R20, R24, R26, R32). Empty means up to now.
    pub as_of: String,
    /// The party.
    pub party: Option<SourceParty>,
    /// A seller account.
    pub seller: Option<AnswerRecord>,
    /// A زمیندار account.
    pub zamindar: Option<AnswerRecord>,
    /// An item.
    pub item: Option<AnswerRecord>,
    /// All the items of a group, پھل or سبزی (R2, R3, R4, R7, R10, R27). Left out of the wire form
    /// when none.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub item_group: Option<ItemGroup>,
    /// A city.
    pub city: Option<AnswerRecord>,
    /// An اڈا.
    pub adda: Option<AnswerRecord>,
    /// The statement with its detail lines (R14).
    pub detail: bool,
    /// The income report by day or month (R28).
    pub group_by: Option<GroupBy>,
    /// Paid or due lines only (R9).
    pub status: Option<PaidFilter>,
    /// The partner of R31 or R29.
    pub partner: Option<AnswerRecord>,
    /// One kind of voucher (R15).
    pub voucher_type: Option<VoucherType>,
    /// The layout of the trip report (R33).
    pub trip_view: Option<TripView>,
    /// One garden (R36).
    pub garden: Option<AnswerRecord>,
    /// The layout of the gardens report (R36).
    pub garden_view: Option<GardenView>,
    /// One بکری (R1).
    pub bikri: Option<SourceBikri>,
    /// Only the fruit taken for the shop's guests (R37).
    pub guests_only: bool,
    /// One collector, a staff member (R22, R23, R35). Left out of the wire form when none.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub collector: Option<AnswerRecord>,
}

fn id_of(record: &Option<AnswerRecord>) -> Option<i64> {
    record.as_ref().map(|r| r.id)
}

impl SourceReportFilters {
    /// The report engine's own filters: every record as its id.
    pub fn to_filters(&self) -> ReportFilters {
        ReportFilters {
            shortcut: self.shortcut,
            from: self.from.clone(),
            to: self.to.clone(),
            as_of: self.as_of.clone(),
            party: self.party.as_ref().map(SourceParty::to_filter),
            seller_id: id_of(&self.seller),
            zamindar_id: id_of(&self.zamindar),
            item_id: id_of(&self.item),
            item_group: self.item_group,
            city_id: id_of(&self.city),
            adda_id: id_of(&self.adda),
            detail: self.detail,
            group_by: self.group_by,
            status: self.status,
            partner_id: id_of(&self.partner),
            voucher_type: self.voucher_type,
            trip_view: self.trip_view,
            garden_id: id_of(&self.garden),
            garden_view: self.garden_view,
            bikri_id: self.bikri.map(|b| b.id),
            guests_only: self.guests_only.then_some(true),
            collector_id: id_of(&self.collector),
            ..ReportFilters::default()
        }
    }
}

/// The filters of an analytics source, the same as [`AnalyticsFilters`] with whole records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SourceAnalyticsFilters {
    /// A period shortcut. It wins over `from` and `to`.
    pub period: Option<AnalyticsPeriod>,
    /// First day, `YYYY-MM-DD`. Empty means open.
    pub from: String,
    /// Last day, `YYYY-MM-DD`. Empty means open.
    pub to: String,
    /// The columns of views D and G.
    pub group_by: Option<AnalyticsGroupBy>,
    /// An item.
    pub item: Option<AnswerRecord>,
    /// All the items of a group, پھل or سبزی (views A, B, C, D, G and H). Left out of the wire form when
    /// none.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub item_group: Option<ItemGroup>,
    /// A seller.
    pub seller: Option<AnswerRecord>,
    /// A buyer, and for a پختہ کھاتہ account the person.
    pub buyer: Option<SourceParty>,
    /// The buyer's city.
    pub city: Option<AnswerRecord>,
    /// An اڈا.
    pub adda: Option<AnswerRecord>,
    /// Which commission is counted.
    pub side: AnalyticsSide,
    /// The comparison. It needs a from date and a to date.
    pub compare: AnalyticsCompare,
    /// A partner (views F, G and J).
    pub partner: Option<AnswerRecord>,
    /// A market trip.
    pub trip: Option<AnswerRecord>,
    /// A garden.
    pub garden: Option<AnswerRecord>,
    /// Which goods are counted.
    pub goods: AnalyticsGoods,
    /// How view J lays out its rows.
    pub trip_view: Option<AnalyticsTripView>,
}

impl Default for SourceAnalyticsFilters {
    fn default() -> Self {
        Self {
            period: None,
            from: String::new(),
            to: String::new(),
            group_by: None,
            item: None,
            item_group: None,
            seller: None,
            buyer: None,
            city: None,
            adda: None,
            side: AnalyticsSide::Both,
            compare: AnalyticsCompare::Off,
            partner: None,
            trip: None,
            garden: None,
            goods: AnalyticsGoods::All,
            trip_view: None,
        }
    }
}

impl SourceAnalyticsFilters {
    /// The analytics view's own filters: every record as its id.
    pub fn to_filters(&self) -> AnalyticsFilters {
        AnalyticsFilters {
            period: self.period,
            from: self.from.clone(),
            to: self.to.clone(),
            group_by: self.group_by,
            item_id: id_of(&self.item),
            item_group: self.item_group,
            seller_id: id_of(&self.seller),
            buyer: self.buyer.as_ref().map(SourceParty::to_filter),
            city_id: id_of(&self.city),
            adda_id: id_of(&self.adda),
            side: self.side,
            compare: self.compare,
            partner_id: id_of(&self.partner),
            trip_id: id_of(&self.trip),
            garden_id: id_of(&self.garden),
            goods: self.goods,
            trip_view: self.trip_view,
            hide_partners: false,
        }
    }
}
