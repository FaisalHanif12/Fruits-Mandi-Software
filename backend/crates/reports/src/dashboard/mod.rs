//! The dashboard and the attention list (plan 19, PRD 9, FR-9.1, FR-7.44 to FR-7.46, NFR-SPD-2,
//! NFR-SPD-22). A read model: every number is read from posted records (PRD FR-6.1), nothing is
//! written (Code Standards B29), and one call reads everything from ONE snapshot of the books
//! with ONE value of today, so two tiles can never come from two different moments.
//!
//! The money tiles use the rows of the reports they must tie to, not copies of their formulas:
//! the buyers' dues are the rows of R20 (PRD FR-19.9), the sellers' payable and the open advances
//! are the workings of R24 (FR-7.59, FR-15.8), the money lent is the working of R32 (FR-16.5).
//! The garden tile of PRD 9.1 comes with the gardens (D4R, round 11).
//!
//! Example (PRD 4.9, after Events 1 to 5): 41,400 in 2 بکری, cash 58,484, dues 21,564, payable
//! 10,160, nothing in stock; the partner strip 996 / 746 / 746; the bell is empty.

mod attention;
mod charts;
mod lent;
mod lists;
mod model;
mod signals;
mod strip;
mod tiles;

use mandi_core::dates::BusinessDate;
use mandi_db::Db;
use mandi_db::rusqlite::Connection;

use crate::error::ReportsError;
use crate::reports::collections::dues::{Buyer, buyers_owing_now};
use crate::reports::collections::payables::{Seller, candidates};
use crate::reports::common::{in_snapshot, today as today_of};
use crate::reports::filters::Typed;
use crate::reports::grouped_sums::SellerSums;

pub use model::{
    AttentionFirst, AttentionItem, AttentionKind, AttentionList, AttentionTarget, BACKUP_OLD_HOURS,
    BackupFacts, CHART_DAYS, DUES_OLD_DAYS, Dashboard, DashboardBikri, DashboardChartDay,
    DashboardChartItem, DashboardDue, DashboardPartner, DashboardTiles, MAX_TODAY_ROWS,
    OLDEST_DUES, PAYABLES_OLD_DAYS, STOCK_WAIT_DAYS, TOP_ITEMS,
};

/// What the tiles and the attention list both read, loaded once.
pub(crate) struct Facts {
    /// Today in Pakistan.
    pub(crate) today: BusinessDate,
    /// Today, `YYYY-MM-DD`.
    pub(crate) today_iso: String,
    /// The buyers who owe, with the ages of their open bills (the rows of R20).
    pub(crate) buyers: Vec<Buyer>,
    /// The seller keys with a balance, an advance or something payable (the rows of R24).
    pub(crate) sellers: Vec<Seller>,
    /// The sums of the seller keys the workings were made from.
    pub(crate) sums: SellerSums,
}

fn load_facts(conn: &Connection, today: BusinessDate) -> Result<Facts, ReportsError> {
    let today_iso = today.to_iso();
    let buyers = buyers_owing_now(conn, today)?;
    let typed = Typed {
        today: Some(today),
        ..Typed::default()
    };
    let (sellers, sums) = candidates(conn, &typed, &today_iso, true)?;
    Ok(Facts {
        today,
        today_iso,
        buyers,
        sellers,
        sums: sums.unwrap_or_default(),
    })
}

/// Everything the dashboard shows (PRD 9). Needs no sign-in check itself: the command does it.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit; `ReportsError::Data` for a day outside the calendar or a missing cash account.
pub fn get(db: &Db, backup: BackupFacts) -> Result<Dashboard, ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let facts = load_facts(conn, today)?;
            let lent = lent::facts(conn, today)?;
            let tiles = tiles::build(conn, &facts, &lent)?;
            let (bikri_today, bikri_today_count) = lists::bikri_today(conn, &facts.today_iso)?;
            Ok(Dashboard {
                today: facts.today_iso.clone(),
                tiles,
                partners: strip::build(conn, today)?,
                bikri_today,
                bikri_today_count,
                oldest_dues: lists::oldest_dues(conn, &facts.buyers)?,
                chart_days: charts::days(conn, today)?,
                chart_items: charts::items(conn, today)?,
                attention: attention::list(conn, &facts, &lent, backup)?,
            })
        })
    })
}

/// The attention list and the number on the bell (PRD FR-7.44).
///
/// # Errors
/// As [`get`].
pub fn attention(db: &Db, backup: BackupFacts) -> Result<AttentionList, ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let facts = load_facts(conn, today)?;
            let lent = lent::facts(conn, today)?;
            attention::list(conn, &facts, &lent, backup)
        })
    })
}
