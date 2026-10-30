//! The types of the analytics commands (plan 21, PRD 18.1 to 18.3, FR-18.2, FR-18.4, FR-18.5).
//!
//! The shape follows the report engine on purpose: typed columns and rows of the same cells, a
//! total row over the whole filter, and Fluent message IDs for every word, so the screen lays
//! out what it receives and works out nothing (CLAUDE.md rule 1). Money travels as paisa.

use mandi_core::auth::Language;
use mandi_core::printing::Paper;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::print::{PrintFact, TextDir};
use crate::reports::{
    Cell, Column, ColumnKind, Names, PartyFilter, PartyRef, PrintCell, Row, RowKind,
};

/// The views of PRD 18.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub enum AnalyticsView {
    /// By item, قسم مال وار.
    #[serde(rename = "A")]
    A,
    /// By seller, بیوپاری وار.
    #[serde(rename = "B")]
    B,
    /// By buyer, گاہک وار.
    #[serde(rename = "C")]
    C,
    /// Item by month, قسم مال × مہینہ.
    #[serde(rename = "D")]
    D,
    /// Seller by item, بیوپاری × قسم مال.
    #[serde(rename = "E")]
    E,
    /// Partners, شراکت دار وار.
    #[serde(rename = "F")]
    F,
    /// Trend, رجحان.
    #[serde(rename = "G")]
    G,
    /// Top 10, سرفہرست.
    #[serde(rename = "H")]
    H,
    /// Gardens, باغات.
    #[serde(rename = "I")]
    I,
    /// Trips, مارکیٹ سفر.
    #[serde(rename = "J")]
    J,
    /// Fruit taken, پھل.
    #[serde(rename = "K")]
    K,
}

impl AnalyticsView {
    /// The letter of PRD 18.2.
    pub const fn code(self) -> &'static str {
        match self {
            AnalyticsView::A => "A",
            AnalyticsView::B => "B",
            AnalyticsView::C => "C",
            AnalyticsView::D => "D",
            AnalyticsView::E => "E",
            AnalyticsView::F => "F",
            AnalyticsView::G => "G",
            AnalyticsView::H => "H",
            AnalyticsView::I => "I",
            AnalyticsView::J => "J",
            AnalyticsView::K => "K",
        }
    }
}

/// The period shortcuts of PRD 18.1. They win over typed dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsPeriod {
    /// آج.
    Today,
    /// یہ ہفتہ: Monday to today.
    ThisWeek,
    /// یہ مہینہ: the 1st to today.
    ThisMonth,
    /// پچھلا مہینہ.
    LastMonth,
    /// یہ سال: 1 January to today.
    ThisYear,
    /// پچھلا سال: the whole of it.
    LastYear,
}

/// How view D breaks its columns up (PRD 18.1 "group by").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsGroupBy {
    /// One column for each day.
    Day,
    /// One column for each month (the default).
    Month,
    /// One column for each year.
    Year,
}

/// Which commission is counted (PRD 18.1 "side").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsSide {
    /// Seller commission, buyer commission and لاگا.
    #[default]
    Both,
    /// Seller commission only.
    Seller,
    /// Buyer commission and لاگا only.
    Buyer,
}

/// The comparison of PRD 18.1.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsCompare {
    /// No comparison.
    #[default]
    Off,
    /// The period of the same length that ends the day before this one starts.
    Previous,
    /// The same dates one year earlier.
    LastYear,
}

/// Which goods are counted (PRD 18.1 "goods").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsGoods {
    /// All goods.
    #[default]
    All,
    /// Consigned goods: sold for a seller who sent them.
    Consigned,
    /// The goods of market trips.
    Trip,
    /// The goods of gardens.
    Garden,
}

/// How view J lays out its rows (PRD 18.2 "trip, purchase or item, day").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsTripView {
    /// One row for each trip (the default).
    #[default]
    Trip,
    /// One row for each purchase.
    Purchase,
    /// One row for each item.
    Item,
    /// One row for each day the goods were bought.
    Day,
}

/// Which money of a partner's gardens, trips and fruit account a column of view F stands for
/// (PRD 18.2 F, FR-18.2): the lines behind the column are his lines on that account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsPartnerMoney {
    /// His lines for market trips.
    Trips,
    /// His lines for gardens.
    Gardens,
    /// His lines for fruit taken: what he took home and his part of the guests' fruit.
    Fruit,
}

/// Which filter boxes a view shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsFilterKind {
    /// The period shortcuts and the from and to dates.
    Period,
    /// An item.
    Item,
    /// A seller.
    Seller,
    /// A buyer, and for a پختہ کھاتہ account one person.
    Buyer,
    /// The buyer's city.
    City,
    /// An اڈا.
    Adda,
    /// Which commission is counted.
    Side,
    /// The comparison.
    Compare,
    /// Day, month or year columns (views D and G).
    GroupBy,
    /// A partner (views F, G, I, J and K).
    Partner,
    /// A market trip (views A to E, G, H and J).
    Trip,
    /// Which goods (views A to E, G, H and J).
    Goods,
    /// How view J lays out its rows.
    TripView,
    /// A garden (views A to E, G, H and I).
    Garden,
}

/// One view in the catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsCatalogEntry {
    /// The letter of PRD 18.2.
    pub id: AnalyticsView,
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The filters the view takes.
    pub filters: Vec<AnalyticsFilterKind>,
}

/// The filters of a run, as the screen sends them (PRD 18.1). Dates are text as typed
/// (`10-10-2026`, `10`, `10-9`) or `YYYY-MM-DD`; empty means open. A key may be left out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct AnalyticsFilters {
    /// A period shortcut. It wins over `from` and `to`.
    pub period: Option<AnalyticsPeriod>,
    /// First day.
    pub from: String,
    /// Last day.
    pub to: String,
    /// The columns of views D and G. Month when left out; refused for the other views.
    pub group_by: Option<AnalyticsGroupBy>,
    /// An item.
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// A seller.
    #[ts(type = "number | null")]
    pub seller_id: Option<i64>,
    /// A buyer, and for a پختہ کھاتہ account the person.
    pub buyer: Option<PartyFilter>,
    /// The buyer's city.
    #[ts(type = "number | null")]
    pub city_id: Option<i64>,
    /// An اڈا.
    #[ts(type = "number | null")]
    pub adda_id: Option<i64>,
    /// Which commission is counted.
    pub side: AnalyticsSide,
    /// The comparison. It needs a from date and a to date.
    pub compare: AnalyticsCompare,
    /// A partner (views F, G and J).
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
    /// A market trip.
    #[ts(type = "number | null")]
    pub trip_id: Option<i64>,
    /// A garden.
    #[ts(type = "number | null")]
    pub garden_id: Option<i64>,
    /// Which goods are counted.
    pub goods: AnalyticsGoods,
    /// How view J lays out its rows. Trip when left out; refused for the other views.
    pub trip_view: Option<AnalyticsTripView>,
}

impl Default for AnalyticsFilters {
    fn default() -> Self {
        Self {
            period: None,
            from: String::new(),
            to: String::new(),
            group_by: None,
            item_id: None,
            seller_id: None,
            buyer: None,
            city_id: None,
            adda_id: None,
            side: AnalyticsSide::Both,
            compare: AnalyticsCompare::Off,
            partner_id: None,
            trip_id: None,
            garden_id: None,
            goods: AnalyticsGoods::All,
            trip_view: None,
        }
    }
}

/// The filters as the backend read them, echoed back to the screen and into the print header.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsResolvedFilters {
    /// The shortcut, if one was used.
    pub period: Option<AnalyticsPeriod>,
    /// First day, `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`.
    pub to: Option<String>,
    /// The columns of view D.
    pub group_by: Option<AnalyticsGroupBy>,
    /// The item.
    pub item: Option<Names>,
    /// The seller.
    pub seller: Option<PartyRef>,
    /// The buyer.
    pub buyer: Option<PartyRef>,
    /// The buyer's city.
    pub city: Option<Names>,
    /// The اڈا.
    pub adda: Option<Names>,
    /// Which commission is counted.
    pub side: AnalyticsSide,
    /// The comparison.
    pub compare: AnalyticsCompare,
    /// First day of the compared period, `YYYY-MM-DD`.
    pub compare_from: Option<String>,
    /// Last day of the compared period, `YYYY-MM-DD`.
    pub compare_to: Option<String>,
    /// The partner.
    pub partner: Option<Names>,
    /// The trip.
    pub trip: Option<Names>,
    /// The garden.
    pub garden: Option<Names>,
    /// Which goods are counted.
    pub goods: AnalyticsGoods,
    /// How view J lays out its rows.
    pub trip_view: Option<AnalyticsTripView>,
}

/// Which lines a row or a column stands for. It is what the screen sends back to open the lines
/// behind a number (FR-18.2): the `key` of the row, and for views D and E the `drill` of the
/// column. Every field that is set narrows the lines.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct AnalyticsKey {
    /// One item.
    #[ts(type = "number | null")]
    pub item_id: Option<i64>,
    /// One seller.
    #[ts(type = "number | null")]
    pub seller_id: Option<i64>,
    /// One buyer account.
    #[ts(type = "number | null")]
    pub buyer_id: Option<i64>,
    /// One person of a پختہ کھاتہ account.
    pub person_key: Option<String>,
    /// First day, `YYYY-MM-DD`. The lines are also inside the period of the filters.
    pub from: Option<String>,
    /// Last day, `YYYY-MM-DD`.
    pub to: Option<String>,
    /// Every item except these (the "other items" column of view E).
    #[ts(type = "Array<number>")]
    pub not_item_ids: Vec<i64>,
    /// One partner (the share rows behind a number of view F).
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
    /// The trip purchases whose sales are meant (the lines behind a row of view J).
    #[ts(type = "Array<number>")]
    pub purchase_ids: Vec<i64>,
    /// True for the row of the shop's guests in view K (the fruit entries with no partner).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "boolean | null", optional)]
    pub guests: Option<bool>,
    /// Which of a partner's venture money a column of view F stands for. Set by the column, not by
    /// the row.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub money: Option<AnalyticsPartnerMoney>,
}

/// What a dynamic column is called. The screen names a month from Fluent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnalyticsHeading {
    /// A day, `YYYY-MM-DD`.
    Day {
        /// The day.
        iso: String,
    },
    /// A month of a year.
    Month {
        /// The year.
        #[ts(type = "number")]
        year: i64,
        /// The month, 1 to 12.
        #[ts(type = "number")]
        month: i64,
    },
    /// A year.
    Year {
        /// The year.
        #[ts(type = "number")]
        year: i64,
    },
    /// An item.
    Item {
        /// The names.
        names: Names,
    },
    /// All the items that have no column of their own.
    OtherItems,
    /// A partner (the columns of view J that follow each partner's share).
    Partner {
        /// The names.
        names: Names,
    },
}

/// One column of a view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsColumn {
    /// A stable key, for example `total_comm`, or `m_2026_10` for a month.
    pub key: String,
    /// Fluent message ID of the heading. For a column with a `heading` it is the ID of the
    /// column group and the heading names the column.
    pub title_id: String,
    /// What it holds.
    pub kind: ColumnKind,
    /// Names a dynamic column (a month or an item).
    pub heading: Option<AnalyticsHeading>,
    /// What this column adds to the key of a row to open the lines behind a cell (views D, E).
    pub drill: Option<AnalyticsKey>,
}

/// One row of a view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsRow {
    /// One cell for each column.
    pub cells: Vec<Cell>,
    /// Opens the lines behind the row. Null for a heading row.
    pub key: Option<AnalyticsKey>,
    /// How it is drawn.
    pub kind: RowKind,
}

/// The input of `analytics_run`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AnalyticsRunInput {
    /// The view.
    pub view: AnalyticsView,
    /// The filters.
    pub filters: AnalyticsFilters,
    /// The first row, 0 or more.
    #[ts(type = "number")]
    pub offset: i64,
    /// The rows to return. 0 means 100; at most 1,000.
    #[ts(type = "number")]
    pub limit: i64,
    /// The column to sort the whole filter by: the `key` of a column of the answer. Null is the
    /// order of the view. The rows between two heading rows are sorted among themselves. For view
    /// C (by buyer) only `buyer`, `qty`, `purchase_value`, `buyer_comm` and `laga` can be sorted:
    /// the dues and the days to pay are worked out for the page only. An unknown key is
    /// `ERR_OUT_OF_RANGE` on `sort.column` (value `allowed`). The charts never change with a sort
    /// (plan 38 R4).
    #[serde(default)]
    pub sort: Option<mandi_core::paging::SortInput>,
}

/// How a chart is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsChartKind {
    /// The biggest rows by one measure (a bar chart), with one point "others" for the rest.
    Ranked,
    /// One point for every period (a line chart).
    Trend,
}

/// What a value of a chart is: the column it comes from and its kind (money or a count).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsChartSeries {
    /// The `key` of the column of the table the values come from.
    pub key: String,
    /// `money` (the value is paisa, whole rupees) or `number`.
    pub kind: crate::reports::ColumnKind,
}

/// One point of a chart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsChartPoint {
    /// What the point is called, a cell like the table's: an item, a party, a month, a day. Empty
    /// for the point `others`.
    pub label: Cell,
    /// Opens the lines behind the point, as the row of the table does. Null for `others`.
    pub key: Option<AnalyticsKey>,
    /// One value for each series, in the order of `series`: a money or number cell.
    pub values: Vec<Cell>,
    /// True for the point that adds up the rows beyond the biggest ten.
    pub is_others: bool,
}

/// A chart of a view, over the WHOLE filter. Its numbers are the sums of the table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsChart {
    /// Ranked (bars) or trend (lines).
    pub kind: AnalyticsChartKind,
    /// For a view with sections (H): the heading of the section the chart belongs to. Null
    /// otherwise.
    pub title: Option<Cell>,
    /// The measures. One for a ranked chart; several for a trend (view G has five).
    pub series: Vec<AnalyticsChartSeries>,
    /// The points. A ranked chart has at most 11 (ten and `others`); a trend has one for every
    /// period, at most 120 (see `by_month`).
    pub points: Vec<AnalyticsChartPoint>,
    /// How many rows the point `others` adds up. 0 when there is none.
    #[ts(type = "number")]
    pub others_count: i64,
    /// True when the periods were days and there were more than 120, so the points are months.
    pub by_month: bool,
}

/// One page of a view with the totals over the whole filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsPage {
    /// The view.
    pub view: AnalyticsView,
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The filters as read.
    pub filters: AnalyticsResolvedFilters,
    /// The columns.
    pub columns: Vec<AnalyticsColumn>,
    /// The rows of this page.
    pub rows: Vec<AnalyticsRow>,
    /// The first row of this page.
    #[ts(type = "number")]
    pub offset: i64,
    /// The page size asked for (100 when 0 was sent).
    #[ts(type = "number")]
    pub limit: i64,
    /// All the rows of the filter, not this page's.
    #[ts(type = "number")]
    pub total_count: i64,
    /// Fluent message ID of the totals row label.
    pub totals_label_id: String,
    /// The totals row over the whole filter, one cell for each column.
    pub totals: Vec<Cell>,
    /// The charts of the view over the whole filter: none, one, or one for each section (view H).
    /// A sort and the page never change them (plan 38 R4).
    pub charts: Vec<AnalyticsChart>,
}

/// The input of `analytics_lines`: the lines behind a row or a cell.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AnalyticsLinesInput {
    /// The view the number was in.
    pub view: AnalyticsView,
    /// The filters of that run.
    pub filters: AnalyticsFilters,
    /// The `key` of the row. An empty key stands for the whole filter (the totals row).
    pub row_key: AnalyticsKey,
    /// The `drill` of the column, for a cell of view D or E.
    pub column_key: Option<AnalyticsKey>,
    /// The first line, 0 or more.
    #[ts(type = "number")]
    pub offset: i64,
    /// The lines to return. 0 means 100; at most 1,000.
    #[ts(type = "number")]
    pub limit: i64,
}

/// The lines behind a number: one row for each sale line fact, reversal rows negative, so the
/// lines add up to the number that was opened. Each row opens its بکری through `source`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsLinesPage {
    /// The view the number was in.
    pub view: AnalyticsView,
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The filters as read.
    pub filters: AnalyticsResolvedFilters,
    /// The columns.
    pub columns: Vec<Column>,
    /// The lines of this page.
    pub rows: Vec<Row>,
    /// The first line of this page.
    #[ts(type = "number")]
    pub offset: i64,
    /// The page size asked for.
    #[ts(type = "number")]
    pub limit: i64,
    /// All the lines behind the number.
    #[ts(type = "number")]
    pub total_count: i64,
    /// Fluent message ID of the totals row label.
    pub totals_label_id: String,
    /// The totals over all the lines, one cell for each column.
    pub totals: Vec<Cell>,
}

/// The input of `analytics_print_data`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AnalyticsPrintInput {
    /// The view.
    pub view: AnalyticsView,
    /// The filters.
    pub filters: AnalyticsFilters,
    /// The print language.
    pub language: Language,
    /// The paper class. A4 when left out.
    pub paper: Option<Paper>,
    /// The sort of the screen: the print has the same order, with every row (plan 38 R4).
    #[serde(default)]
    pub sort: Option<mandi_core::paging::SortInput>,
}

/// A dynamic column heading, formatted for print.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnalyticsPrintHeading {
    /// A day as printed.
    Day {
        /// The day as printed.
        text: String,
    },
    /// A month of a year. The template names the month from Fluent.
    Month {
        /// The year as printed.
        year: String,
        /// The month, 1 to 12.
        #[ts(type = "number")]
        month: i64,
    },
    /// A year as printed.
    Year {
        /// The year as printed.
        text: String,
    },
    /// An item in the print language.
    Item {
        /// The name.
        text: String,
    },
    /// All the items that have no column of their own.
    OtherItems,
    /// A partner in the print language.
    Partner {
        /// The name.
        text: String,
    },
}

/// One printed column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsPrintColumn {
    /// A stable key.
    pub key: String,
    /// Fluent message ID of the heading.
    pub title_id: String,
    /// What it holds. The template aligns numbers and money to the right edge.
    pub kind: ColumnKind,
    /// Names a dynamic column.
    pub heading: Option<AnalyticsPrintHeading>,
}

/// One printed row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsPrintRow {
    /// One cell for each column.
    pub cells: Vec<PrintCell>,
    /// How it is drawn.
    pub kind: RowKind,
}

/// Everything a template needs to print one view with every row (PRD FR-18.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AnalyticsPrintData {
    /// The view.
    pub view: AnalyticsView,
    /// The print language.
    pub language: Language,
    /// The writing direction.
    pub dir: TextDir,
    /// The paper class.
    pub paper: Paper,
    /// True when Urdu digits are used on this print.
    pub urdu_digits: bool,
    /// Fluent message ID of the title.
    pub title_id: String,
    /// The filters of the run, the facts under the title. They go to `reports_print_header` with
    /// the title.
    pub filters: Vec<PrintFact>,
    /// The columns.
    pub columns: Vec<AnalyticsPrintColumn>,
    /// Every row of the filter.
    pub rows: Vec<AnalyticsPrintRow>,
    /// How many rows there are, as printed.
    pub row_count: String,
    /// Fluent message ID of the totals row label.
    pub totals_label_id: String,
    /// The totals row, one cell for each column.
    pub totals: Vec<PrintCell>,
}

/// One saved view of PRD FR-18.5: a name and a set of filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct AnalyticsSavedView {
    /// The name the Owner gave it, 1 to 60 letters.
    pub name: String,
    /// The view it opens.
    pub view: AnalyticsView,
    /// The filters it applies.
    pub filters: AnalyticsFilters,
}

/// The input of `analytics_views_save`: the whole list of saved views of the signed-in user.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct AnalyticsViewsSaveInput {
    /// The views, at most 50.
    pub views: Vec<AnalyticsSavedView>,
}
