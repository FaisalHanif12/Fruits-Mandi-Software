//! The partner code check for reports and analytics (PRD FR-17.11, NFR-SEC-14, Code Standards S8).
//!
//! Every way to get a report or a view (run, print data, the print job, Excel export, the lines
//! behind a number) asks here first, after the sign-in check. What a request needs is decided by
//! `mandi_reports::partner_guard`; this module only reads the lock and applies the answer.

use mandi_reports::analytics::{AnalyticsFilters, AnalyticsView};
use mandi_reports::partner_guard::{PartnerNeed, analytics_need, report_need};
use mandi_reports::reports::{ReportFilters, ReportId};
use mandi_services::{Context, ServiceError, partner_code};

/// The check for a report. A partner report is refused with `ERR_PARTNER_LOCKED` while the
/// section is locked; a mixed one gets `hide_partners` set, so its partner parts are left out.
///
/// # Errors
/// `ERR_PARTNER_LOCKED`; `ServiceError::Db`.
pub(crate) fn report(
    ctx: &Context,
    id: ReportId,
    filters: &mut ReportFilters,
) -> Result<(), ServiceError> {
    match report_need(id, filters) {
        PartnerNeed::None => {}
        PartnerNeed::HideParts => filters.hide_partners = partner_code::is_locked(ctx)?,
        PartnerNeed::Refuse => partner_code::require_unlocked(ctx)?,
    }
    Ok(())
}

/// The check for an analytics view, as [`report`].
///
/// # Errors
/// `ERR_PARTNER_LOCKED`; `ServiceError::Db`.
pub(crate) fn analytics(
    ctx: &Context,
    view: AnalyticsView,
    filters: &mut AnalyticsFilters,
) -> Result<(), ServiceError> {
    match analytics_need(view, filters) {
        PartnerNeed::None => {}
        PartnerNeed::HideParts => filters.hide_partners = partner_code::is_locked(ctx)?,
        PartnerNeed::Refuse => partner_code::require_unlocked(ctx)?,
    }
    Ok(())
}
