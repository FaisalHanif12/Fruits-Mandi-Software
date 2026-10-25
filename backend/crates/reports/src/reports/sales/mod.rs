//! The Sales group of the catalogue (PRD 19.2): the full sales register R2, cash sales R3, credit
//! sales R4, the seller summary R6, item-wise sales R7, the truck stand report R8, the register
//! of one-time parties R9 and the bill series R27.
//!
//! Every figure is a value stored when the بکری was saved (Code Standards B20): the sale lines
//! (`bikri_sales`), the lots (`bikri_lots`) and the facts (`analytics_facts`). Nothing is worked
//! out again from a rule, so a report can never disagree with the bills that were printed.

mod adda;
mod buyer_bills;
mod items;
mod lots;
mod pukhta;
mod register;
mod stock;

pub(super) use adda::run as run_r8;
pub(super) use buyer_bills::{Slips, run as run_r5, slips as buyer_slips};
pub(super) use items::{query as r7_query, run as run_r7};
pub(super) use lots::{r6_page_query, run_r1, run_r6, run_r27};
pub(super) use pukhta::run as run_r9;
pub(super) use register::{
    Kind as RegisterKind, page_query as register_page_query, run as run_register,
    totals_query as register_totals_query,
};
pub(super) use stock::run as run_r10;
