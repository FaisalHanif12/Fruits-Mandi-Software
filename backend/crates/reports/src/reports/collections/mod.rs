//! The Collections group of the catalogue (PRD 19.2): buyer dues R20, by city R21 and by
//! collector R22, seller payables R24 and advances R26.
//!
//! The dues, payables and advances are the read-time workings of PRD FR-15.4, FR-7.59 and
//! FR-15.8 over the posted lines and the matching rows. Nothing is stored and nothing is written
//! (PRD FR-18.8). The total row of R20 is the shop's total اگراہی and equals the dashboard tile
//! (PRD FR-19.9); the total of the payable column of R24 equals the tile ادائیگی باقی.

mod advances;
pub(crate) mod dues;
mod grouped;
pub(crate) mod payables;

pub(super) use advances::run as run_r26;
pub(super) use dues::{Layout as DuesLayout, run as run_dues};
pub use grouped::{KeySums, SellerSums, payable_sums, pukhta_names};
pub(super) use payables::run as run_r24;
