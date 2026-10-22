//! The Business group of the catalogue (PRD 19.2): the income report R28 and the profit and
//! partner split R30.
//!
//! Both read posted records only: the journal lines of the income and expense heads and the
//! partner share rows that every posting wrote (PRD FR-6.5, FR-17.4). They write nothing
//! (PRD FR-18.8).

mod income;
mod profit;

pub(super) use income::{page_query as r28_page_query, run as run_r28};
pub(super) use profit::run as run_r30;
