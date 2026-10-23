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
//! ادھاری, R6 بکری تفصیل, R7 قسم مال وار بکری, R8 اڈا رپورٹ and R9 پختہ کھاتہ رپورٹ; the Collections
//! group R20 وصولی / اگراہی, R21 بحساب شہر, R22 بحساب اگراہی والا, R24 ادائیگی باقی, R26 پیشگی کھاتہ
//! and R27 بیجک سیریز; the Business group R28 آمدن رپورٹ and R30 منافع و تقسیم. The other reports
//! join the same catalogue in the later plans.

mod accounts;
mod business;
mod collections;
pub(crate) mod common;
mod filters;
pub(crate) mod ids;
mod model;
pub(crate) mod print_data;
mod sales;
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
    pub use super::accounts::{R11_PAGE_SQL, R11_TOTALS_SQL, R13_PAGE_SQL, R13_TOTALS_SQL};
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
    pub use super::collections::{KeySums, SellerSums, payable_sums, pukhta_names};
}

pub use ids::posting_types;
pub use model::{
    CatalogEntry, Cell, Column, ColumnKind, DateShortcut, DetailPart, FilterKind, GroupBy, Names,
    PaidFilter, PartyFilter, PartyRef, ReportFilters, ReportGroup, ReportId, ReportPage,
    ReportPrintInput, ReportRunInput, ResolvedFilters, Row, RowKind, RowSource, SourceKind,
    SummaryLine,
};
pub use print_data::{
    PrintCell, PrintDetailPart, PrintOptions, PrintReportColumn, PrintReportParty, PrintReportRow,
    PrintSummaryLine, ReportPrintData, build_print,
};

pub(crate) use accounts::PageSpec;
use filters::{Takes, Typed, resolve};

use crate::error::ReportsError;
use crate::print::load::{load_firm, load_grouping, load_setup};

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

/// The filters of the three sales registers: dates, the buyer, and the seller, زمیندار, item,
/// the buyer's city and اڈا (PRD FR-19.1).
const REGISTER_FILTERS: &[FilterKind] = &[
    FilterKind::DateRange,
    FilterKind::Party,
    FilterKind::Seller,
    FilterKind::Zamindar,
    FilterKind::Item,
    FilterKind::City,
    FilterKind::Adda,
];

/// The filters of the seller side reports R6 and R27.
const SELLER_SIDE_FILTERS: &[FilterKind] = &[
    FilterKind::DateRange,
    FilterKind::Seller,
    FilterKind::Zamindar,
    FilterKind::Item,
    FilterKind::Adda,
];

/// The filters of the buyer dues reports R20 to R22.
const DUES_FILTERS: &[FilterKind] = &[FilterKind::AsOf, FilterKind::Party, FilterKind::City];

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
    spec(ReportId::R2, ReportGroup::Sales, REGISTER_FILTERS, false),
    spec(ReportId::R3, ReportGroup::Sales, REGISTER_FILTERS, false),
    spec(ReportId::R4, ReportGroup::Sales, REGISTER_FILTERS, false),
    spec(ReportId::R6, ReportGroup::Sales, SELLER_SIDE_FILTERS, false),
    spec(
        ReportId::R7,
        ReportGroup::Sales,
        &[
            FilterKind::DateRange,
            FilterKind::Party,
            FilterKind::Seller,
            FilterKind::Item,
            FilterKind::City,
            FilterKind::Adda,
        ],
        false,
    ),
    spec(
        ReportId::R8,
        ReportGroup::Sales,
        &[FilterKind::DateRange, FilterKind::Adda],
        false,
    ),
    spec(
        ReportId::R9,
        ReportGroup::Sales,
        &[FilterKind::DateRange, FilterKind::Party, FilterKind::Status],
        false,
    ),
    spec(
        ReportId::R11,
        ReportGroup::Accounts,
        &[FilterKind::DateRange, FilterKind::Party],
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
    spec(ReportId::R22, ReportGroup::Collections, DUES_FILTERS, false),
    spec(
        ReportId::R24,
        ReportGroup::Collections,
        &[FilterKind::AsOf, FilterKind::Party],
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
        SELLER_SIDE_FILTERS,
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
        ReportId::R30,
        ReportGroup::Business,
        &[FilterKind::DateRange],
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

/// The catalogue: every report with its group, title and filters (PRD 19.2).
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

/// Reads and checks the filters, then builds the report.
fn build(
    conn: &Connection,
    db: &Db,
    id: ReportId,
    input_filters: &ReportFilters,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let entry = spec_of(id)?;
    let today = common::today(db.clock().now_utc())?;
    let typed: Typed = resolve(
        conn,
        today,
        Takes {
            kinds: entry.filters,
            party_required: entry.party_required,
        },
        input_filters,
    )?;
    match id {
        ReportId::R2 => sales::run_register(conn, &typed, spec, sales::RegisterKind::All),
        ReportId::R3 => sales::run_register(conn, &typed, spec, sales::RegisterKind::Cash),
        ReportId::R4 => sales::run_register(conn, &typed, spec, sales::RegisterKind::Credit),
        ReportId::R6 => sales::run_r6(conn, &typed, spec),
        ReportId::R7 => sales::run_r7(conn, &typed, spec),
        ReportId::R8 => sales::run_r8(conn, &typed, spec),
        ReportId::R9 => sales::run_r9(conn, &typed, spec),
        ReportId::R20 => collections::run_dues(conn, &typed, spec, collections::DuesLayout::Flat),
        ReportId::R21 => collections::run_dues(conn, &typed, spec, collections::DuesLayout::ByCity),
        ReportId::R22 => {
            collections::run_dues(conn, &typed, spec, collections::DuesLayout::ByCollector)
        }
        ReportId::R24 => collections::run_r24(conn, &typed, spec),
        ReportId::R26 => collections::run_r26(conn, &typed, spec),
        ReportId::R27 => sales::run_r27(conn, &typed, spec),
        ReportId::R28 => business::run_r28(conn, &typed, spec),
        ReportId::R30 => business::run_r30(conn, &typed, spec),
        ReportId::R11 => accounts::run_r11(conn, &typed, spec),
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
    db.read(|conn| build(conn, db, input.id, &input.filters, spec))
}

/// The print data of a report: every row of the filter, formatted for the print language.
///
/// # Errors
/// As [`run`]; a stored amount that cannot be written in words.
pub fn print_data(db: &Db, input: &ReportPrintInput) -> Result<ReportPrintData, ReportsError> {
    let all = PageSpec {
        offset: 0,
        limit: None,
    };
    let (page, firm, grouping, setup) = db.read(|conn| {
        let page = build(conn, db, input.id, &input.filters, all)?;
        Ok::<_, ReportsError>((
            page,
            load_firm(conn)?,
            load_grouping(conn)?,
            load_setup(conn)?,
        ))
    })?;
    build_print(
        &page,
        &PrintOptions {
            language: input.language,
            grouping,
            urdu_digits_setting: setup.urdu_digits,
            paper: input.paper.unwrap_or(Paper::A4),
            firm,
            now_utc: db.clock().now_utc(),
        },
    )
}
