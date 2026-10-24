//! The dashboard and the attention list (plan 19, PRD 9, FR-9.1, FR-7.44 to FR-7.46, NFR-SPD-2,
//! NFR-SPD-22). A read model: every number is read from posted records (PRD FR-6.1), nothing is
//! written (Code Standards B29), and one call reads everything from one snapshot of the books, so
//! the tiles can never come from two different moments.

mod lists;
mod model;
mod signals;

pub use model::{
    AttentionFirst, AttentionItem, AttentionKind, AttentionList, AttentionTarget, BACKUP_OLD_HOURS,
    BackupFacts, CHART_DAYS, DUES_OLD_DAYS, Dashboard, DashboardBikri, DashboardChartDay,
    DashboardChartItem, DashboardDue, DashboardPartner, DashboardTiles, MAX_TODAY_ROWS,
    OLDEST_DUES, PAYABLES_OLD_DAYS, STOCK_WAIT_DAYS, TOP_ITEMS,
};
