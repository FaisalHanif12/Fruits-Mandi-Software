//! The Business group of the catalogue (PRD 19.2): the income report R28, the expense report
//! R29, the profit and partner split R30 and the money lent report R32.
//!
//! All read posted records only: the journal lines of the income and expense heads and the
//! partner share rows that every posting wrote (PRD FR-6.5, FR-17.4). They write nothing
//! (PRD FR-18.8).

pub(crate) mod day;
mod expenses;
pub(crate) mod fruit;
mod gardens;
mod income;
pub(crate) mod lent;
mod profit;
mod sheet;
pub(crate) mod trips;

pub(super) use day::run as run_r34;
pub(super) use expenses::run as run_r29;
pub(super) use fruit::run as run_r37;
pub(super) use gardens::run as run_r36;
pub(super) use income::{page_query as r28_page_query, run as run_r28};
pub(super) use lent::run as run_r32;
pub(super) use profit::run as run_r30;
pub(super) use sheet::run as run_r35;
pub(super) use trips::run as run_r33;
