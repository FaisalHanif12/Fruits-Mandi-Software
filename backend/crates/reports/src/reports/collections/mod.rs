//! The Collections group of the catalogue (PRD 19.2): buyer dues R20, by city R21 and by
//! collector R22, seller payables R24 and advances R26.
//!
//! The dues, payables and advances are the read-time workings of PRD FR-15.4, FR-7.59 and
//! FR-15.8 over the posted lines and the matching rows. Nothing is stored and nothing is written
//! (PRD FR-18.8). The total row of R20 is the shop's total اگراہی and equals the dashboard tile
//! (PRD FR-19.9); the total of the payable column of R24 equals the tile ادائیگی باقی.

use super::model::{Column, ReportId};

mod advances;
pub(crate) mod dues;
mod grouped;
pub(crate) mod payables;

pub(super) use advances::run as run_r26;
pub(super) use dues::run_without_ages as run_dues_without_ages;
pub(super) use dues::{Layout as DuesLayout, run as run_dues};
pub use grouped::{KeySums, SellerSums, chukta_names, payable_sums};
pub(super) use payables::run as run_r24;

/// The columns of a report of this group as its default view shows them. Empty for a report of
/// another group. Each comes from the same function the report's run uses (plan 75 T1).
pub(super) fn default_columns(id: ReportId) -> Vec<Column> {
    match id {
        ReportId::R20 => dues::columns(DuesLayout::Flat),
        ReportId::R21 => dues::columns(DuesLayout::ByCity),
        ReportId::R22 => dues::columns(DuesLayout::ByCollector),
        ReportId::R24 => payables::columns(),
        ReportId::R26 => advances::columns(),
        _ => Vec::new(),
    }
}
