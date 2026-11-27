//! `windows_bounds` (plan 76 point 8, PRD FR-19.11): the first day that has data in a list, for the
//! back button of its 7-day or 1-month window. The sign-in check, then one service call. Read only.
//!
//! The first day of a partner's statement or of his gardens and trips is a partner fact (PRD
//! FR-17.11, NFR-SEC-14): those sources are refused while the partner section is locked, like every
//! other partner read (Code Standards S8). The money a partner took is open without the code from
//! PRD v1.61 (plan 100 E), and so is the first day of its list. Example: with the section locked,
//! `PartnerStatement` answers `ERR_PARTNER_LOCKED`, and `Sales` and `PartnerTakes` still answer.

use mandi_services::core::windows::{WindowSource, WindowsBounds, WindowsBoundsInput};
use mandi_services::{auth, partner_code, windows};

use super::run;
use crate::error::UiError;

/// The first day that has data in a list, and today as the last day a window may show.
#[tauri::command]
pub async fn windows_bounds<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: WindowsBoundsInput,
) -> Result<WindowsBounds, UiError> {
    run(app, "windows_bounds", move |ctx| {
        auth::require(ctx)?;
        if matches!(
            input.source,
            WindowSource::PartnerStatement { .. } | WindowSource::PartnerVentures { .. }
        ) {
            partner_code::require_unlocked(ctx)?;
        }
        windows::bounds(ctx, &input)
    })
    .await
}
