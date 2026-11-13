//! The report engine (plan 20, PRD 19.1 to 19.5, 19.10, 19.11, NFR-SPD-10, NFR-SPD-11,
//! NFR-SPD-13, NFR-SPD-24, NFR-SEC-13).
//!
//! One engine serves every report, so one screen can show them all:
//!
//! - [`catalog`] lists the reports by group, with the filters each takes;
//! - [`run`] reads and checks the filters, and returns one page of typed rows with the total
//!   number of rows and the totals over the whole filter (never a sum of the page);
//! - [`print_data`] returns the same report with every row, formatted for a print language.
//!
//! A report reads posted records only and writes nothing (PRD FR-6.1, FR-18.8). Every amount in a
//! row is a stored value; every total is a `SUM()` of stored `INTEGER` paisa columns or a
//! checked sum of such sums (money agent Section 9).
//!
//! Built so far (plan 20): the Accounts group R11 روزنامچہ, R13 کھاتہ, R14 فرد حساب, R16 انڈکس,
//! R17 مدوں بیلنس, R18 ٹرائل بیلنس and R19 اوپن ٹرائل; the Sales group R2 مکمل بکری, R3 نقدی, R4
//! ادھاری, R6 بکری تفصیل, R7 قسم مال وار بکری (R8, اڈا رپورٹ, is retired in v1.34); the Collections
//! group R20 وصولی / اگراہی, R21 بحساب شہر, R22 بحساب اگراہی والا, R24 ادائیگی باقی, R26 پیشگی کھاتہ
//! and R27 بیجک سیریز; the Business group R28 آمدن رپورٹ, R29 خرچہ رپورٹ, R30 منافع و تقسیم, R31
//! شراکت دار کھاتہ and R32 ادھار رقم رپورٹ; the Accounts group also has R12 روکڑ. R31 is built from the
//! statement of the services (see [`statement`]). The other reports join the same catalogue in
//! the later plans.

mod accounts;
pub(crate) mod bikri_list;
mod bill_slips;
pub(crate) mod business;
pub(crate) mod collections;
pub(crate) mod common;
pub(crate) mod filters;
pub(crate) mod ids;
mod model;
pub(crate) mod print_data;
mod sales;
pub mod statement;
pub(crate) mod view;
pub(crate) mod vouchers;
#[doc(hidden)]
pub mod workings;

use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::printing::Paper;
use mandi_db::Db;
use mandi_db::rusqlite::Connection;

/// The SQL of the paged reports, for the query plan test (Code Standards B18). Each function
/// gives the query a report runs for a date range, with the values to bind.
#[doc(hidden)]
pub mod plans {
    pub use super::accounts::lines::plans::{journal_page, ledger_page};
    use super::filters::Typed;
    use mandi_db::rusqlite::types::Value;

    fn dates(from: Option<&str>, to: Option<&str>) -> Typed {
        Typed {
            from: from.map(str::to_owned),
            to: to.map(str::to_owned),
            ..Typed::default()
        }
    }

    /// The page query of R2 for a date range.
    pub fn r2_page(from: Option<&str>, to: Option<&str>) -> (String, Vec<Value>) {
        super::sales::register_page_query(&dates(from, to), None, 100, 0)
    }

    /// The totals query of R2 for a date range.
    pub fn r2_totals(from: Option<&str>, to: Option<&str>) -> (String, Vec<Value>) {
        super::sales::register_totals_query(&dates(from, to), None)
    }

    /// The page query of R6 for a date range.
    pub fn r6_page(from: Option<&str>, to: Option<&str>) -> (String, Vec<Value>) {
        super::sales::r6_page_query(&dates(from, to), 100, 0)
    }

    /// The query of R7 for a date range.
    pub fn r7(from: Option<&str>, to: Option<&str>) -> (String, Vec<Value>) {
        super::sales::r7_query(&dates(from, to))
    }

    /// The page query of R28 for a date range.
    pub fn r28_page(from: Option<&str>, to: Option<&str>) -> (String, Vec<Value>) {
        super::business::r28_page_query(&dates(from, to), 100, 0)
    }
}

/// The grouped payable sums of R24, for the parity and query count tests (Code Standards B22).
#[doc(hidden)]
pub mod grouped_sums {
    pub use super::collections::{KeySums, SellerSums, chukta_names, payable_sums};
}

/// Both forms of the day queries of R34, for the equality test of plan 48 P48-S3 only.
#[doc(hidden)]
pub mod day_query_forms {
    pub use super::business::forms::{
        FormRows, buyer_moves, ledger_rows, lot_charges, seller_lines,
    };
}

pub use bikri_list::{
    BikriListPrintData, BikriListPrintDay, BikriListPrintInput, bikri_list_print_data,
};
pub use bill_slips::{BuyerBillSlip, BuyerBillsPrintData};
pub use business::{
    StatementBalance, StatementDay, StatementEntry, StatementItem, StatementKind,
    StatementPrintData, StatementPrintDataInput, StatementSide, entry_lines_sql,
    party_statement_print_data,
};
pub use ids::posting_types;
pub use model::{
    ArgValue, CatalogEntry, Cell, Column, ColumnKind, DateShortcut, DetailPart, FilterKind,
    GardenView, GroupBy, MessageArg, Names, PaidFilter, PartyFilter, PartyRef, ReportFilters,
    ReportGroup, ReportId, ReportPage, ReportPrintInput, ReportRunInput, ReportTable,
    ResolvedFilters, Row, RowKind, RowSource, SourceKind, StockView, SummaryLine, TripView,
};
pub use print_data::{
    PrintArgValue, PrintCell, PrintDetailPart, PrintMessageArg, PrintOptions, PrintReportColumn,
    PrintReportParty, PrintReportRow, PrintReportTable, PrintSummaryLine, ReportPrintData,
    build_print,
};
pub use statement::{StatementRequest, statement_page, statement_paging, statement_request};
pub use view::newest_first_reverses;

pub(crate) use accounts::PageSpec;
use filters::{LineSort, Takes, Typed, resolve};

use crate::error::ReportsError;
use crate::print::load::{load_firm, load_grouping};

/// Today in Pakistan as `YYYY-MM-DD`, from the database clock (the date a PDF name carries).
///
/// # Errors
/// `ReportsError::Data` if the clock is outside the calendar.
pub fn today_iso(db: &Db) -> Result<String, ReportsError> {
    Ok(common::today(db.clock().now_utc())?.to_iso())
}

/// The page size when the screen sends 0 (Code Standards B31).
pub const DEFAULT_LIMIT: i64 = 100;
/// The most rows one page may ask for.
pub const MAX_LIMIT: i64 = 1_000;

/// What the catalogue says about one report.
struct Spec {
    id: ReportId,
    group: ReportGroup,
    filters: &'static [FilterKind],
    party_required: bool,
}

/// The filters of R5: dates, the buyer, the seller, زمیندار, item, the buyer's city and اڈا (PRD
/// FR-19.1), and one بکری (PRD FR-12.12, D8 Feature 8: the button
/// next to the seller-bill print prints ادھاری تفصیل for "this بکری only"). Only R5 takes it.
const BUYER_BILLS_FILTERS: &[FilterKind] = &[
    FilterKind::DateRange,
    FilterKind::Party,
    FilterKind::Seller,
    FilterKind::Zamindar,
    FilterKind::Item,
    FilterKind::City,
    FilterKind::Adda,
    FilterKind::Bikri,
    FilterKind::Lot,
];

/// The filters of R2, R3 and R4: those of the registers, and the item group (PRD v1.29).
const REGISTER_GROUP_FILTERS: &[FilterKind] = &[
    FilterKind::DateRange,
    FilterKind::Party,
    FilterKind::Seller,
    FilterKind::Zamindar,
    FilterKind::Item,
    FilterKind::ItemGroup,
    FilterKind::City,
    FilterKind::Adda,
    FilterKind::Grade,
    FilterKind::Lot,
];

/// The filters of the seller side reports R6 and R27.
const SELLER_SIDE_FILTERS: &[FilterKind] = &[
    FilterKind::DateRange,
    FilterKind::Seller,
    FilterKind::Zamindar,
    FilterKind::Item,
    FilterKind::Adda,
    FilterKind::Lot,
];

/// The filters of R27: those of R6, and the item group (PRD v1.29).
const SELLER_SIDE_GROUP_FILTERS: &[FilterKind] = &[
    FilterKind::DateRange,
    FilterKind::Seller,
    FilterKind::Zamindar,
    FilterKind::Item,
    FilterKind::ItemGroup,
    FilterKind::Adda,
    FilterKind::Lot,
];

/// The filters of the buyer dues reports R20 to R22.
const DUES_FILTERS: &[FilterKind] = &[FilterKind::AsOf, FilterKind::Party, FilterKind::City];

/// The filters of R22: the dues by collector, and one collector (plan 55 §11.1).
const DUES_BY_COLLECTOR_FILTERS: &[FilterKind] = &[
    FilterKind::AsOf,
    FilterKind::Party,
    FilterKind::City,
    FilterKind::Collector,
];

const fn spec(
    id: ReportId,
    group: ReportGroup,
    filters: &'static [FilterKind],
    party_required: bool,
) -> Spec {
    Spec {
        id,
        group,
        filters,
        party_required,
    }
}

/// The reports of the engine, in catalogue order (PRD 19.2): by group, then by number.
const SPECS: &[Spec] = &[
    // The sale book is one بکری, so the بکری must be picked (it is the "required" filter here).
    spec(ReportId::R1, ReportGroup::Sales, &[FilterKind::Bikri], true),
    spec(
        ReportId::R2,
        ReportGroup::Sales,
        REGISTER_GROUP_FILTERS,
        false,
    ),
    spec(
        ReportId::R3,
        ReportGroup::Sales,
        REGISTER_GROUP_FILTERS,
        false,
    ),
    spec(
        ReportId::R4,
        ReportGroup::Sales,
        REGISTER_GROUP_FILTERS,
        false,
    ),
    spec(ReportId::R5, ReportGroup::Sales, BUYER_BILLS_FILTERS, false),
    spec(ReportId::R6, ReportGroup::Sales, SELLER_SIDE_FILTERS, false),
    spec(
        ReportId::R7,
        ReportGroup::Sales,
        &[
            FilterKind::DateRange,
            FilterKind::Party,
            FilterKind::Seller,
            FilterKind::Item,
            FilterKind::ItemGroup,
            FilterKind::City,
            FilterKind::Adda,
        ],
        false,
    ),
    spec(
        ReportId::R10,
        ReportGroup::Sales,
        &[
            FilterKind::DateRange,
            FilterKind::Seller,
            FilterKind::Item,
            FilterKind::ItemGroup,
            FilterKind::StockView,
            FilterKind::Lot,
        ],
        false,
    ),
    spec(
        ReportId::R11,
        ReportGroup::Accounts,
        &[FilterKind::DateRange, FilterKind::Party],
        false,
    ),
    spec(
        ReportId::R12,
        ReportGroup::Accounts,
        &[FilterKind::DateRange],
        false,
    ),
    spec(
        ReportId::R13,
        ReportGroup::Accounts,
        &[FilterKind::DateRange, FilterKind::Party],
        true,
    ),
    spec(
        ReportId::R14,
        ReportGroup::Accounts,
        &[FilterKind::DateRange, FilterKind::Party, FilterKind::Detail],
        true,
    ),
    spec(
        ReportId::R15,
        ReportGroup::Accounts,
        &[
            FilterKind::DateRange,
            FilterKind::Party,
            FilterKind::VoucherType,
        ],
        false,
    ),
    spec(
        ReportId::R16,
        ReportGroup::Accounts,
        &[FilterKind::AsOf, FilterKind::City],
        false,
    ),
    spec(
        ReportId::R17,
        ReportGroup::Accounts,
        &[FilterKind::AsOf],
        false,
    ),
    spec(
        ReportId::R18,
        ReportGroup::Accounts,
        &[FilterKind::AsOf],
        false,
    ),
    spec(ReportId::R19, ReportGroup::Accounts, &[], false),
    spec(ReportId::R20, ReportGroup::Collections, DUES_FILTERS, false),
    spec(ReportId::R21, ReportGroup::Collections, DUES_FILTERS, false),
    spec(
        ReportId::R22,
        ReportGroup::Collections,
        DUES_BY_COLLECTOR_FILTERS,
        false,
    ),
    spec(
        ReportId::R23,
        ReportGroup::Collections,
        &[
            FilterKind::DateRange,
            FilterKind::Party,
            FilterKind::Collector,
        ],
        false,
    ),
    spec(
        ReportId::R24,
        ReportGroup::Collections,
        &[FilterKind::AsOf, FilterKind::Party],
        false,
    ),
    spec(
        ReportId::R25,
        ReportGroup::Collections,
        &[FilterKind::DateRange, FilterKind::Party],
        false,
    ),
    spec(
        ReportId::R26,
        ReportGroup::Collections,
        &[FilterKind::AsOf, FilterKind::Seller],
        false,
    ),
    spec(
        ReportId::R27,
        ReportGroup::Collections,
        SELLER_SIDE_GROUP_FILTERS,
        false,
    ),
    spec(
        ReportId::R28,
        ReportGroup::Business,
        &[
            FilterKind::DateRange,
            FilterKind::Party,
            FilterKind::GroupBy,
        ],
        false,
    ),
    spec(
        ReportId::R29,
        ReportGroup::Business,
        &[FilterKind::DateRange, FilterKind::Partner],
        false,
    ),
    spec(
        ReportId::R30,
        ReportGroup::Business,
        &[FilterKind::DateRange],
        false,
    ),
    // The partner is required: the statement is one partner's (the "party" of this report).
    spec(
        ReportId::R31,
        ReportGroup::Business,
        &[FilterKind::DateRange, FilterKind::Partner],
        true,
    ),
    spec(
        ReportId::R32,
        ReportGroup::Business,
        &[FilterKind::AsOf, FilterKind::Party],
        false,
    ),
    spec(
        ReportId::R33,
        ReportGroup::Business,
        &[
            FilterKind::DateRange,
            FilterKind::Item,
            FilterKind::TripView,
        ],
        false,
    ),
    spec(
        ReportId::R34,
        ReportGroup::Business,
        &[FilterKind::DateRange],
        false,
    ),
    // One day (the "as of" day, today when empty) and, for one party's slip, the party.
    spec(
        ReportId::R35,
        ReportGroup::Business,
        &[FilterKind::AsOf, FilterKind::Party, FilterKind::Collector],
        false,
    ),
    // The gardens are picked by the day made, the main fruit, a partner in them, or one garden.
    spec(
        ReportId::R36,
        ReportGroup::Business,
        &[
            FilterKind::DateRange,
            FilterKind::Item,
            FilterKind::Partner,
            FilterKind::Garden,
            FilterKind::GardenView,
        ],
        false,
    ),
    // The fruit taken: by day, partner (or only the shop's guests), item, or the garden whose lot
    // it came from.
    spec(
        ReportId::R37,
        ReportGroup::Business,
        &[
            FilterKind::DateRange,
            FilterKind::Partner,
            FilterKind::Guests,
            FilterKind::Item,
            FilterKind::Garden,
        ],
        false,
    ),
    // The dead amount: one row for each person moved to it. The status filter (`due`) keeps the
    // people who still owe something; with none, every move (PRD FR-10.10, R38).
    spec(
        ReportId::R38,
        ReportGroup::Business,
        &[FilterKind::Status],
        false,
    ),
];

fn spec_of(id: ReportId) -> Result<&'static Spec, ReportsError> {
    SPECS
        .iter()
        .find(|s| s.id == id)
        .ok_or(ReportsError::NotFound)
}

fn group_title_id(group: ReportGroup) -> &'static str {
    match group {
        ReportGroup::Sales => ids::GROUP_SALES,
        ReportGroup::Accounts => ids::GROUP_ACCOUNTS,
        ReportGroup::Collections => ids::GROUP_COLLECTIONS,
        ReportGroup::Business => ids::GROUP_BUSINESS,
    }
}

/// The columns of a report as its default view shows them (plan 75 T1). Each is built by the very
/// function the report's own run uses, so the catalogue and the report can never disagree. A
/// report whose columns depend on a view takes the view's default.
fn default_columns(spec: &Spec) -> Vec<Column> {
    match spec.id {
        ReportId::R15 => vouchers::columns(vouchers::Layout::All),
        ReportId::R23 => vouchers::columns(vouchers::Layout::Receipts),
        ReportId::R25 => vouchers::columns(vouchers::Layout::Payments),
        // R31 is built from the statement of the services; its columns are fixed there.
        ReportId::R31 => statement::statement_columns(),
        // R27 is listed with the Collections but is built by the Sales code (the bill series).
        ReportId::R27 => sales::default_columns(spec.id),
        _ => match spec.group {
            ReportGroup::Sales => sales::default_columns(spec.id),
            ReportGroup::Accounts => accounts::default_columns(spec.id),
            ReportGroup::Collections => collections::default_columns(spec.id),
            ReportGroup::Business => business::default_columns(spec.id),
        },
    }
}

/// The catalogue: every report with its group, title, filters and column headings (PRD 19.2,
/// FR-19.14).
pub fn catalog() -> Vec<CatalogEntry> {
    SPECS
        .iter()
        .map(|s| CatalogEntry {
            id: s.id,
            group: s.group,
            group_title_id: group_title_id(s.group).to_owned(),
            title_id: ids::title_id(s.id.code()),
            filters: s.filters.to_vec(),
            party_required: s.party_required,
            column_title_ids: default_columns(s)
                .into_iter()
                .map(|column| column.title_id)
                .collect(),
        })
        .collect()
}

/// Checks the page asked for and gives the rows to read (Code Standards 4.3, B31).
pub(crate) fn page_spec(offset: i64, limit: i64) -> Result<PageSpec, ReportsError> {
    let mut problems = Vec::new();
    if offset < 0 {
        problems.push(FieldError::new(ErrorCode::ErrOutOfRange, "offset"));
    }
    if !(0..=MAX_LIMIT).contains(&limit) {
        problems.push(FieldError::new(ErrorCode::ErrOutOfRange, "limit"));
    }
    if !problems.is_empty() {
        return Err(ReportsError::Rejected(problems));
    }
    Ok(PageSpec {
        offset,
        limit: Some(if limit == 0 { DEFAULT_LIMIT } else { limit }),
    })
}

/// Reads and checks the filters, then builds the report, all from one snapshot of the books.
fn build(
    conn: &Connection,
    db: &Db,
    id: ReportId,
    input_filters: &ReportFilters,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    common::in_snapshot(conn, |conn| build_in(conn, db, id, input_filters, spec))
}

fn build_in(
    conn: &Connection,
    db: &Db,
    id: ReportId,
    input_filters: &ReportFilters,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    build_in_with(conn, db, id, input_filters, spec, None)
}

/// The sorts of the ledger and day book reports that are one SQL page, so such a sort never builds
/// the whole report (plan 38 B2): debit and credit (R11, R13, R14), and the date (R11 to R14: oldest
/// first is the natural order, newest first its reverse). The outer `None` is any other sort, which
/// builds the whole report; `Some(None)` is the natural order.
fn fast_sort_of(id: ReportId, sort: &mandi_core::paging::SortInput) -> Option<Option<LineSort>> {
    use mandi_core::paging::SortDirection;
    let desc = sort.direction == SortDirection::Desc;
    match (id, sort.column.as_str()) {
        (ReportId::R11 | ReportId::R12 | ReportId::R13 | ReportId::R14, "date") => {
            Some(desc.then_some(LineSort::Newest))
        }
        // The sales registers: newest first is one SQL page too (D7, plan 59 F1).
        (ReportId::R2 | ReportId::R3 | ReportId::R4, "date") => {
            Some(desc.then_some(LineSort::Newest))
        }
        (ReportId::R11 | ReportId::R13 | ReportId::R14, "debit") => Some(Some(LineSort::Amount {
            credit: false,
            desc,
        })),
        (ReportId::R11 | ReportId::R13 | ReportId::R14, "credit") => {
            Some(Some(LineSort::Amount { credit: true, desc }))
        }
        _ => None,
    }
}

/// Gives a page the heading of each month it touches, from that month's own totals (one totals read
/// for each month), never a pass over the whole filter (plan 38 B2).
fn page_months(
    conn: &Connection,
    db: &Db,
    id: ReportId,
    filters: &ReportFilters,
    page: &mut ReportPage,
) -> Result<(), ReportsError> {
    let plan = view::MonthPlan {
        filters,
        first_day: page.filters.from.clone(),
        last_day: page
            .filters
            .to
            .clone()
            .unwrap_or_else(|| view::OPEN_END_DAY.to_owned()),
    };
    view::add_page_months(page, &plan, |sub| {
        let none = PageSpec {
            offset: 0,
            limit: Some(0),
        };
        build_in(conn, db, id, sub, none)
    })
}

fn build_in_with(
    conn: &Connection,
    db: &Db,
    id: ReportId,
    input_filters: &ReportFilters,
    spec: PageSpec,
    line_sort: Option<LineSort>,
) -> Result<ReportPage, ReportsError> {
    let entry = spec_of(id)?;
    let today = common::today(db.clock().now_utc())?;
    let mut typed: Typed = resolve(
        conn,
        today,
        Takes {
            kinds: entry.filters,
            party_required: entry.party_required,
        },
        input_filters,
    )?;
    typed.line_sort = line_sort;
    match id {
        ReportId::R2 => sales::run_register(conn, &typed, spec, sales::RegisterKind::All),
        ReportId::R3 => sales::run_register(conn, &typed, spec, sales::RegisterKind::Cash),
        ReportId::R4 => sales::run_register(conn, &typed, spec, sales::RegisterKind::Credit),
        ReportId::R1 => sales::run_r1(conn, &typed, spec),
        ReportId::R5 => sales::run_r5(conn, &typed, spec),
        ReportId::R6 => sales::run_r6(conn, &typed, spec),
        ReportId::R7 => sales::run_r7(conn, &typed, spec),
        ReportId::R20 => collections::run_dues(conn, &typed, spec, collections::DuesLayout::Flat),
        ReportId::R21 => collections::run_dues(conn, &typed, spec, collections::DuesLayout::ByCity),
        ReportId::R22 => {
            collections::run_dues(conn, &typed, spec, collections::DuesLayout::ByCollector)
        }
        ReportId::R24 => collections::run_r24(conn, &typed, spec),
        ReportId::R26 => collections::run_r26(conn, &typed, spec),
        ReportId::R27 => sales::run_r27(conn, &typed, spec),
        ReportId::R28 => business::run_r28(conn, &typed, spec),
        ReportId::R29 => business::run_r29(conn, &typed, spec),
        ReportId::R30 => business::run_r30(conn, &typed, spec),
        ReportId::R32 => business::run_r32(conn, &typed, spec),
        ReportId::R33 => business::run_r33(conn, &typed, spec),
        ReportId::R34 => business::run_r34(conn, &typed, spec),
        ReportId::R35 => business::run_r35(conn, &typed, spec),
        ReportId::R36 => business::run_r36(conn, &typed, spec),
        ReportId::R37 => business::run_r37(conn, &typed, spec),
        ReportId::R38 => business::run_r38(conn, &typed, spec),
        ReportId::R10 => sales::run_r10(conn, &typed, spec),
        ReportId::R15 => vouchers::run(conn, &typed, spec, vouchers::Layout::All),
        ReportId::R23 => vouchers::run(conn, &typed, spec, vouchers::Layout::Receipts),
        ReportId::R25 => vouchers::run(conn, &typed, spec, vouchers::Layout::Payments),
        // R31 is the statement of a partner: its lines are put together by the services, so the
        // app builds it with `statement_request` and `statement_page` and never comes here.
        ReportId::R31 => Err(ReportsError::Data(
            "R31 is built from the statement of the services".to_owned(),
        )),
        ReportId::R11 => accounts::run_r11(conn, &typed, spec),
        ReportId::R12 => accounts::run_r12(conn, &typed, spec),
        ReportId::R13 => accounts::run_r13(conn, &typed, spec),
        ReportId::R14 => accounts::run_r14(conn, &typed, spec),
        ReportId::R16 => accounts::run_r16(conn, &typed, spec),
        ReportId::R17 => accounts::run_r17(conn, &typed, spec),
        ReportId::R18 => accounts::run_r18(conn, &typed, spec),
        ReportId::R19 => accounts::run_r19(conn, &typed, spec),
    }
}

/// One page of a report with the totals over the whole filter.
///
/// # Errors
/// `ReportsError::Rejected` with the problems of the filters or the page, each with its field;
/// a database or money error.
pub fn run(db: &Db, input: &ReportRunInput) -> Result<ReportPage, ReportsError> {
    let spec = page_spec(input.offset, input.limit)?;
    let wants_months = input.group_by_month && view::has_months(input.id);
    db.read(|conn| {
        common::in_snapshot(conn, |conn| {
            // A sort needs every row of the filter: the report is built whole, sorted, given its
            // month headings if asked and cut after (plan 38 R4). The totals are the same
            // whole-filter totals as ever.
            if let Some(sort) = &input.sort
                && let Some(fast) = fast_sort_of(input.id, sort)
            {
                // A sort by the date, debit or credit of a ledger or the day book is one SQL page.
                // By an amount the balance is left empty (it shows in date order only).
                let mut page = build_in_with(conn, db, input.id, &input.filters, spec, fast)?;
                let in_date_order = !matches!(fast, Some(LineSort::Amount { .. }));
                if wants_months && in_date_order {
                    page_months(conn, db, input.id, &input.filters, &mut page)?;
                }
                return Ok(page);
            }
            if input.sort.is_some() {
                let all = PageSpec {
                    offset: 0,
                    limit: None,
                };
                let mut page = build_in(conn, db, input.id, &input.filters, all)?;
                view::shape(&mut page, input.id, input.sort.as_ref(), wants_months)?;
                view::cut(&mut page, spec);
                keep_tables_of_the_page(&mut page);
                return Ok(page);
            }
            // Month headings in the order of the report: the page is built alone, and each month
            // it touches gets a heading made from that month's own totals (one totals read per
            // month), never a pass over the whole filter (plan 38 B2).
            if wants_months {
                let mut page = build_in(conn, db, input.id, &input.filters, spec)?;
                page_months(conn, db, input.id, &input.filters, &mut page)?;
                return Ok(page);
            }
            let mut page = build_in(conn, db, input.id, &input.filters, spec)?;
            // A page that starts inside a group needs the heading of that group, with the totals
            // of the whole group (plan 38 B2). Only the reports with group rows pay for it, and
            // only on a later page.
            let starts_with_heading = page.rows.first().is_some_and(|r| r.kind == RowKind::Group);
            if spec.offset > 0 && emits_group_rows(input.id) && !starts_with_heading {
                let all = PageSpec {
                    offset: 0,
                    limit: None,
                };
                let full = build_in(conn, db, input.id, &input.filters, all)?;
                let upto = usize::try_from(spec.offset)
                    .unwrap_or(usize::MAX)
                    .min(full.rows.len());
                page.group_above = full
                    .rows
                    .get(..upto)
                    .and_then(|rows| rows.iter().rev().find(|r| r.kind == RowKind::Group))
                    .cloned();
            }
            Ok(page)
        })
    })
}

/// R20, R21 or R22 with the ages of the open bills left out (plan 48 P48-S1): the same buyers, the
/// same balances and the same total row as [`run`], with the age cells empty. It reads the balances
/// only (0.3 seconds on the NFR-SCL-1 volume against 3.8 for the whole report). It takes the
/// filters of the report and a page; it has no sort.
///
/// # Errors
/// `ReportsError::Rejected` for a filter or page problem; `ReportsError::Data` for a sort, month
/// headings, or a report that is not R20, R21 or R22; a database or money error.
pub fn run_dues_without_ages(db: &Db, input: &ReportRunInput) -> Result<ReportPage, ReportsError> {
    let layout = match input.id {
        ReportId::R20 => collections::DuesLayout::Flat,
        ReportId::R21 => collections::DuesLayout::ByCity,
        ReportId::R22 => collections::DuesLayout::ByCollector,
        _ => {
            return Err(ReportsError::Data(
                "only R20, R21 and R22 can be read without ages".to_owned(),
            ));
        }
    };
    if input.sort.is_some() || input.group_by_month {
        return Err(ReportsError::Data(
            "the dues without ages have no sort and no month headings".to_owned(),
        ));
    }
    let spec = page_spec(input.offset, input.limit)?;
    let entry = spec_of(input.id)?;
    db.read(|conn| {
        common::in_snapshot(conn, |conn| {
            let today = common::today(db.clock().now_utc())?;
            let typed: Typed = resolve(
                conn,
                today,
                Takes {
                    kinds: entry.filters,
                    party_required: entry.party_required,
                },
                &input.filters,
            )?;
            collections::run_dues_without_ages(conn, &typed, spec, layout)
        })
    })
}

/// The most rows [`run_first_rows`] gives back.
pub const MAX_FIRST_ROWS: i64 = 100_000;

/// A report from its first row, with at most `max_rows` rows, built ONCE (plan 48 P48-S2): the
/// page, the columns and the totals over the whole filter are exactly those of [`run`] at offset 0.
/// `run` cuts a page of at most 1,000 rows and works a report whole for every page (R20 takes
/// about 4 seconds on the NFR-SCL-1 volume), so nine pages take 64 seconds. A caller that needs
/// every row of a small report (the assistant's `where` and its two sort keys) reads it here in
/// one build. The rows are all of the filter when `total_count <= max_rows`; ask for one row more
/// than the most you accept, and `total_count` says whether the report is too big. It has no sort
/// and no month headings.
///
/// Example (worked day): R20 with `max_rows` 1,001 has its 2 buyers and `total_count` 2.
///
/// # Errors
/// `ReportsError::Rejected` for a filter problem or a `max_rows` outside 1 to [`MAX_FIRST_ROWS`];
/// `ReportsError::Data` for a sort or month headings; a database or money error.
pub fn run_first_rows(
    db: &Db,
    input: &ReportRunInput,
    max_rows: i64,
) -> Result<ReportPage, ReportsError> {
    if !(1..=MAX_FIRST_ROWS).contains(&max_rows) {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "limit",
        )]));
    }
    if input.sort.is_some() || input.group_by_month {
        return Err(ReportsError::Data(
            "the first rows have no sort and no month headings".to_owned(),
        ));
    }
    let spec = PageSpec {
        offset: 0,
        limit: Some(max_rows),
    };
    db.read(|conn| build(conn, db, input.id, &input.filters, spec))
}

/// R34 gives the tables of every day when it is built whole; a page keeps the tables of the days
/// it has a row of (plan 42).
fn keep_tables_of_the_page(page: &mut ReportPage) {
    if page.tables.is_empty() {
        return;
    }
    let days: Vec<&str> = page
        .rows
        .iter()
        .filter_map(|r| match r.cells.first() {
            Some(Cell::Date { iso }) => Some(iso.as_str()),
            _ => None,
        })
        .collect();
    page.tables.retain(|t| days.contains(&t.day.as_str()));
}

/// The reports that put group rows (a heading with the totals of its group) among their rows.
fn emits_group_rows(id: ReportId) -> bool {
    matches!(
        id,
        ReportId::R17
            | ReportId::R20
            | ReportId::R21
            | ReportId::R22
            | ReportId::R24
            | ReportId::R29
            | ReportId::R33
            | ReportId::R35
    )
}

/// The 80 mm roll is for the day slip of one party (R35 with a party picked, PRD FR-19.12, plan 42):
/// the all-parties table of R35 and the day total sheet R34 have tables that do not fit on it, so
/// they are refused with `ERR_OUT_OF_RANGE` on `paper`, with the papers they take in `allowed`.
/// Every other report keeps the papers it had.
fn check_paper(
    id: ReportId,
    filters: &ReportFilters,
    paper: Option<Paper>,
) -> Result<(), ReportsError> {
    let one_party_slip = id == ReportId::R35 && filters.party.is_some();
    if paper == Some(Paper::Mm80) && matches!(id, ReportId::R34 | ReportId::R35) && !one_party_slip
    {
        return Err(ReportsError::Rejected(vec![
            FieldError::new(ErrorCode::ErrOutOfRange, "paper").with(
                "allowed",
                mandi_core::error::ErrorValue::Text("a4,a5".to_owned()),
            ),
        ]));
    }
    Ok(())
}

/// The print data of a report: every row of the filter, formatted for the print language.
///
/// # Errors
/// As [`run`]; a stored amount that cannot be written in words.
pub fn print_data(db: &Db, input: &ReportPrintInput) -> Result<ReportPrintData, ReportsError> {
    check_paper(input.id, &input.filters, input.paper)?;
    let all = PageSpec {
        offset: 0,
        limit: None,
    };
    // A seller's lots print in the order of the print language (PRD FR-12.12, plan 71 L7).
    let mut filters = input.filters.clone();
    filters.order_language.get_or_insert(input.language);
    let mut page = db.read(|conn| build(conn, db, input.id, &filters, all))?;
    // The print follows the screen: its sort and its month headings, with every row.
    view::shape(
        &mut page,
        input.id,
        input.sort.as_ref(),
        input.group_by_month,
    )?;
    print_data_for_page(db, &page, input)
}

/// The print data of a page that already holds every row, for a report the engine does not read
/// itself (R31, whose lines come from the services). Only the number grouping and the print setup
/// are read; the page is formatted as [`print_data`] formats its own. The header and footer come
/// from `reports_print_header` (PRD FR-19.13), not from here.
///
/// # Errors
/// A database error; a stored amount that cannot be written in words.
pub fn print_data_for_page(
    db: &Db,
    page: &ReportPage,
    input: &ReportPrintInput,
) -> Result<ReportPrintData, ReportsError> {
    build_print(page, &print_options(db, input)?)
}

/// What a print needs besides its rows: the number grouping and the paper.
fn print_options(db: &Db, input: &ReportPrintInput) -> Result<PrintOptions, ReportsError> {
    let grouping = db.read(load_grouping)?;
    Ok(PrintOptions {
        language: input.language,
        grouping,
        paper: input.paper.unwrap_or(Paper::A4),
    })
}

/// The print data of "Print All" for R5: one slip for each credit buyer of the filter, with every
/// line, in the print language (PRD 19.2 R5, FR-19.10, FR-19.11). The filters are those of R5.
/// The slips and the totals are read from one snapshot of the books; the slips add up to the
/// totals of R5 and of R4. Reads only.
///
/// # Errors
/// As [`run`]; `ERR_OUT_OF_RANGE` on `id` if the report is not R5; `ReportsError::Data` if a
/// buyer's lines do not add up to his sums.
pub fn buyer_bills_print_data(
    db: &Db,
    input: &ReportPrintInput,
) -> Result<BuyerBillsPrintData, ReportsError> {
    bill_slips::check_input(input)?;
    let (typed, slips) = db.read(|conn| {
        common::in_snapshot(conn, |conn| {
            let entry = spec_of(ReportId::R5)?;
            let today = common::today(db.clock().now_utc())?;
            let typed: Typed = resolve(
                conn,
                today,
                Takes {
                    kinds: entry.filters,
                    party_required: entry.party_required,
                },
                &input.filters,
            )?;
            let slips = sales::buyer_slips(conn, &typed)?;
            Ok((typed, slips))
        })
    })?;
    bill_slips::build(&slips, &typed.resolved, &print_options(db, input)?)
}

/// Every row of a report with the firm header names and the number grouping, for the Excel
/// export and the readable pack. Reads only; one database read.
///
/// # Errors
/// As [`run`].
pub(crate) fn export_page(
    db: &Db,
    id: ReportId,
    filters: &ReportFilters,
    sort: Option<&mandi_core::paging::SortInput>,
    group_by_month: bool,
) -> Result<
    (
        ReportPage,
        crate::print::load::FirmNames,
        mandi_core::money::Grouping,
    ),
    ReportsError,
> {
    let all = PageSpec {
        offset: 0,
        limit: None,
    };
    db.read(|conn| {
        let mut page = build(conn, db, id, filters, all)?;
        // The export follows the screen: its sort and its month headings, with every row.
        view::shape(&mut page, id, sort, group_by_month)?;
        Ok::<_, ReportsError>((page, load_firm(conn)?, load_grouping(conn)?))
    })
}

/// Every row of a report, read on a connection the caller holds, so several pages can come from
/// one read (the readable pack).
///
/// # Errors
/// As [`run`].
pub(crate) fn page_in(
    conn: &Connection,
    db: &Db,
    id: ReportId,
    filters: &ReportFilters,
) -> Result<ReportPage, ReportsError> {
    build(
        conn,
        db,
        id,
        filters,
        PageSpec {
            offset: 0,
            limit: None,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::l10n::L10n;
    use mandi_core::auth::Language;

    /// Plan 75 T1: every report of the catalogue lists its columns, in the order the report shows
    /// them (PRD FR-19.14).
    #[test]
    fn every_report_lists_its_columns() {
        for entry in catalog() {
            assert!(
                !entry.column_title_ids.is_empty(),
                "{} has no column headings in the catalogue",
                entry.id.code()
            );
        }
    }

    /// Plan 75 T1: every heading ID of every report exists in both languages (the Fluent files are
    /// embedded at build time, so the guide card can always write the heading).
    #[test]
    fn every_column_heading_exists_in_both_languages() {
        let ur = L10n::new(Language::Ur).unwrap();
        let en = L10n::new(Language::En).unwrap();
        let mut missing = Vec::new();
        for entry in catalog() {
            for id in &entry.column_title_ids {
                if !ur.has(id) {
                    missing.push(format!("{} {id}: no Urdu message", entry.id.code()));
                }
                if !en.has(id) {
                    missing.push(format!("{} {id}: no English message", entry.id.code()));
                }
            }
        }
        assert!(missing.is_empty(), "{missing:#?}");
    }

    /// R2, R3 and R4 list the same columns as the register with no optional column; R3 and R4 add
    /// the two columns of money paid on the spot (PRD FR-12.44).
    #[test]
    fn the_registers_list_the_default_shape() {
        let of = |id: ReportId| {
            catalog()
                .into_iter()
                .find(|e| e.id == id)
                .map(|e| e.column_title_ids)
                .unwrap()
        };
        let r2 = of(ReportId::R2);
        let r3 = of(ReportId::R3);
        assert_eq!(r3.get(..r2.len()), Some(r2.as_slice()));
        assert_eq!(
            r3.get(r2.len()..),
            Some(
                [
                    ids::COL_PAID_ON_SPOT.to_owned(),
                    ids::COL_LEFT_ON_ACCOUNT.to_owned()
                ]
                .as_slice()
            )
        );
        assert_eq!(r3, of(ReportId::R4));
    }
}
