//! What the partner code asks of a report or an analytics view (PRD FR-17.11, NFR-SEC-14, plan 44).
//!
//! The app asks before it runs, prints or exports a report or a view. The answer says whether the
//! request is refused while the partner section is locked (`ERR_PARTNER_LOCKED`), or runs with the
//! partner parts left out, or needs nothing. Run, print and Excel export all go through the same
//! two functions, so a request cannot get round the lock by the way it is asked.
//!
//! | Report or view | While locked |
//! |---|---|
//! | R30, R31, view F, view K | refused |
//! | R29, R33, R36, R37, views G, I, J with a partner picked | refused |
//! | R33 by partner, R36 by partner | refused |
//! | R29 without a partner | runs, the money partners took and gave back is left out |
//! | R34 | runs, the partner shares, money taken, returned and capital are left out |
//! | views I and J | run, the columns of each partner are left out |
//!
//! Example: R29 for October with no partner picked still shows the shop expenses by head while
//! locked, but not الف's 2,000 taken.

use crate::analytics::{AnalyticsFilters, AnalyticsView};
use crate::reports::{GardenView, ReportFilters, ReportId, TripView};

/// What a request needs from the partner code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartnerNeed {
    /// Nothing: it shows no partner figure.
    None,
    /// It runs with the partner parts left out while the section is locked.
    HideParts,
    /// It is refused while the section is locked.
    Refuse,
}

/// What a report request needs.
#[must_use]
pub fn report_need(id: ReportId, filters: &ReportFilters) -> PartnerNeed {
    let partner_picked = filters.partner_id.is_some();
    match id {
        ReportId::R30 | ReportId::R31 => PartnerNeed::Refuse,
        ReportId::R29 if partner_picked => PartnerNeed::Refuse,
        ReportId::R29 | ReportId::R34 => PartnerNeed::HideParts,
        ReportId::R33 if partner_picked || filters.trip_view == Some(TripView::Partner) => {
            PartnerNeed::Refuse
        }
        ReportId::R36 if partner_picked || filters.garden_view == Some(GardenView::Partner) => {
            PartnerNeed::Refuse
        }
        ReportId::R37 if partner_picked => PartnerNeed::Refuse,
        _ => PartnerNeed::None,
    }
}

/// What an analytics request needs.
#[must_use]
pub fn analytics_need(view: AnalyticsView, filters: &AnalyticsFilters) -> PartnerNeed {
    match view {
        AnalyticsView::F | AnalyticsView::K => PartnerNeed::Refuse,
        _ if filters.partner_id.is_some() => PartnerNeed::Refuse,
        AnalyticsView::I | AnalyticsView::J => PartnerNeed::HideParts,
        _ => PartnerNeed::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_partner_reports_and_views_are_refused_and_the_mixed_ones_hide_their_parts() {
        let plain = ReportFilters::default();
        assert_eq!(report_need(ReportId::R30, &plain), PartnerNeed::Refuse);
        assert_eq!(report_need(ReportId::R31, &plain), PartnerNeed::Refuse);
        assert_eq!(report_need(ReportId::R29, &plain), PartnerNeed::HideParts);
        assert_eq!(report_need(ReportId::R34, &plain), PartnerNeed::HideParts);
        assert_eq!(report_need(ReportId::R33, &plain), PartnerNeed::None);
        assert_eq!(report_need(ReportId::R2, &plain), PartnerNeed::None);
        let by_partner = ReportFilters {
            partner_id: Some(1),
            ..ReportFilters::default()
        };
        for id in [ReportId::R29, ReportId::R33, ReportId::R36, ReportId::R37] {
            assert_eq!(report_need(id, &by_partner), PartnerNeed::Refuse, "{id:?}");
        }
        let trips = ReportFilters {
            trip_view: Some(TripView::Partner),
            ..ReportFilters::default()
        };
        assert_eq!(report_need(ReportId::R33, &trips), PartnerNeed::Refuse);
        let gardens = ReportFilters {
            garden_view: Some(GardenView::Partner),
            ..ReportFilters::default()
        };
        assert_eq!(report_need(ReportId::R36, &gardens), PartnerNeed::Refuse);

        let filters = AnalyticsFilters::default();
        assert_eq!(
            analytics_need(AnalyticsView::F, &filters),
            PartnerNeed::Refuse
        );
        assert_eq!(
            analytics_need(AnalyticsView::K, &filters),
            PartnerNeed::Refuse
        );
        assert_eq!(
            analytics_need(AnalyticsView::I, &filters),
            PartnerNeed::HideParts
        );
        assert_eq!(
            analytics_need(AnalyticsView::J, &filters),
            PartnerNeed::HideParts
        );
        assert_eq!(
            analytics_need(AnalyticsView::A, &filters),
            PartnerNeed::None
        );
        let picked = AnalyticsFilters {
            partner_id: Some(1),
            ..AnalyticsFilters::default()
        };
        assert_eq!(
            analytics_need(AnalyticsView::G, &picked),
            PartnerNeed::Refuse
        );
    }
}
