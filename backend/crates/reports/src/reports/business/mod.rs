//! The Business group of the catalogue (PRD 19.2): the income report R28, the expense report
//! R29, the profit and partner split R30 and the money lent report R32.
//!
//! All read posted records only: the journal lines of the income and expense heads and the
//! partner share rows that every posting wrote (PRD FR-6.5, FR-17.4). They write nothing
//! (PRD FR-18.8).

use super::model::{Column, GardenView, GroupBy, ReportId, TripView};

pub(crate) mod day;
mod dead;
pub(crate) mod expenses;
pub(crate) mod forms;
pub(crate) mod fruit;
mod gardens;
mod income;
pub(crate) mod lent;
mod profit;
mod sheet;
pub(crate) mod trips;

pub(super) use day::run as run_r34;
pub(super) use dead::run as run_r38;
pub(super) use expenses::run as run_r29;
pub(super) use fruit::run as run_r37;
pub(super) use gardens::run as run_r36;
pub(super) use income::{page_query as r28_page_query, run as run_r28};
pub(super) use lent::run as run_r32;
pub(super) use profit::run as run_r30;
pub(super) use sheet::run as run_r35;
pub use sheet::{
    StatementBalance, StatementDay, StatementEntry, StatementItem, StatementKind,
    StatementPrintData, StatementPrintDataInput, StatementSide, entry_lines_sql,
    party_statement_print_data,
};
pub(super) use trips::run as run_r33;

/// The columns of a report of this group as its default view shows them: R33 by trip, R36 by
/// garden, R28 by month, R35 as the table of all parties. Empty for a report of another group.
/// Each comes from the same function the report's run uses (plan 75 T1).
pub(super) fn default_columns(id: ReportId) -> Vec<Column> {
    match id {
        ReportId::R28 => income::columns(GroupBy::default()),
        ReportId::R29 => expenses::columns(),
        ReportId::R30 => profit::columns(),
        ReportId::R32 => lent::columns(),
        ReportId::R33 => trips::columns(TripView::default()),
        ReportId::R34 => day::columns(),
        ReportId::R35 => sheet::table_columns(),
        ReportId::R36 => gardens::columns(GardenView::default()),
        ReportId::R37 => fruit::columns(),
        ReportId::R38 => dead::columns(),
        _ => Vec::new(),
    }
}
