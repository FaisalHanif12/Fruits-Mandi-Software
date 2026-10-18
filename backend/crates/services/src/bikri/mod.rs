//! The بکری screen (PRD 12, plans 12 and 13): the live preview, the save and the reads.
//!
//! - `shared`: the one working out of a draft (`work_out`): it reads the facts the draft names in
//!   one go (Code Standards B22), calls `mandi_domain::bikri::calc_posting`, builds the journal
//!   entry, splits the income with `partners::split_entry` and asks `cash::cash_after_entry`
//!   what the entry does to cash. The preview and the save both call it (Code Standards B5), so
//!   the screen and the saved numbers cannot differ.
//! - `preview` (here): read only (Code Standards B29).
//! - `save`: one transaction (Code Standards B3).
//! - `read`: a saved بکری, the list, the find and first / previous / next / last.
//! - `pukhta`: the one-time people and the register of the پختہ کھاتہ accounts.
//!
//! The read connection is not re-entrant, so the rules and switches are fetched before the one
//! read of the call (see the services README).

mod cancel;
mod edit;
mod later;
mod load;
mod pukhta;
mod read;
mod rows;
mod save;
mod shared;
mod stock;

pub use cancel::{cancel, preview as cancel_preview, undo};
pub use edit::{edit_preview, edit_save, open_for_edit};
pub use later::{preview as later_sale_preview, save as later_sale_save};
pub use pukhta::{register, search_people};
pub use read::{find, get, list, navigate};
pub use save::save;
pub use stock::{list as stock_list, write_off, write_off_undo};

pub(crate) use shared::work_out;

use mandi_domain::core::bikri::{BikriDraft, BikriPreview, prepare_draft};

use crate::context::Context;
use crate::day;
use crate::error::ServiceError;
use crate::rules;

/// Works out a whole typed بکری for the live summary (PRD 12.5, FR-7.5). Nothing is saved.
///
/// Example (PRD 4.4): Event 1 gives خام 29,400, کل اخراجات 10,084, صافی 19,316, buyer bills
/// 12,880 and 18,684, shared income 3,948 split 1,580 / 1,184 / 1,184, and cash 5,000 lower.
///
/// # Errors
/// The mistakes in the draft come back inside the preview (`errors`), not as an error here.
/// `ServiceError::Db`, `ServiceError::Data` or `ERR_OVERFLOW` for a failure the user cannot fix.
pub fn preview(ctx: &Context, draft: &BikriDraft) -> Result<BikriPreview, ServiceError> {
    let (closed_up_to, today) = day::window(ctx)?;
    let prepared = prepare_draft(draft, today, closed_up_to);
    // Fetched before the read below: the read connection is not re-entrant.
    let rule_set = rules::rule_set(ctx)?;
    let settings = rules::overrides(ctx)?;
    let switches = shared::switches(&settings);
    ctx.db.read(|conn| {
        Ok(work_out(
            conn,
            prepared,
            &rule_set,
            switches,
            &shared::Basis::new_bikri(),
        )?
        .preview())
    })
}
