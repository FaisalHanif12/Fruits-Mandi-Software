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
//! The Accounts group is built: R11 روزنامچہ, R13 کھاتہ, R14 فرد حساب, R16 انڈکس, R17 مدوں
//! بیلنس, R18 ٹرائل بیلنس and R19 اوپن ٹرائل. The other groups join the same catalogue in the
//! next steps.

mod accounts;
mod common;
mod filters;
mod ids;
mod model;
mod print_data;

use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::printing::Paper;
use mandi_db::Db;
use mandi_db::rusqlite::Connection;

/// The SQL of the paged reports, for the query plan test (Code Standards B18).
#[doc(hidden)]
pub mod plans {
    pub use super::accounts::{R11_PAGE_SQL, R11_TOTALS_SQL, R13_PAGE_SQL, R13_TOTALS_SQL};
}

pub use ids::posting_types;
pub use model::{
    CatalogEntry, Cell, Column, ColumnKind, DateShortcut, DetailPart, FilterKind, Names,
    PartyFilter, PartyRef, ReportFilters, ReportGroup, ReportId, ReportPage, ReportPrintInput,
    ReportRunInput, ResolvedFilters, Row, RowKind, RowSource, SourceKind, SummaryLine,
};
pub use print_data::{
    PrintCell, PrintDetailPart, PrintOptions, PrintReportColumn, PrintReportParty, PrintReportRow,
    PrintSummaryLine, ReportPrintData, build_print,
};

use accounts::PageSpec;
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

/// The reports of the engine, in catalogue order (PRD 19.2).
const SPECS: &[Spec] = &[
    Spec {
        id: ReportId::R11,
        group: ReportGroup::Accounts,
        filters: &[FilterKind::DateRange, FilterKind::Party],
        party_required: false,
    },
    Spec {
        id: ReportId::R13,
        group: ReportGroup::Accounts,
        filters: &[FilterKind::DateRange, FilterKind::Party],
        party_required: true,
    },
    Spec {
        id: ReportId::R14,
        group: ReportGroup::Accounts,
        filters: &[FilterKind::DateRange, FilterKind::Party, FilterKind::Detail],
        party_required: true,
    },
    Spec {
        id: ReportId::R16,
        group: ReportGroup::Accounts,
        filters: &[FilterKind::AsOf, FilterKind::City],
        party_required: false,
    },
    Spec {
        id: ReportId::R17,
        group: ReportGroup::Accounts,
        filters: &[FilterKind::AsOf],
        party_required: false,
    },
    Spec {
        id: ReportId::R18,
        group: ReportGroup::Accounts,
        filters: &[FilterKind::AsOf],
        party_required: false,
    },
    Spec {
        id: ReportId::R19,
        group: ReportGroup::Accounts,
        filters: &[],
        party_required: false,
    },
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
fn page_spec(offset: i64, limit: i64) -> Result<PageSpec, ReportsError> {
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
