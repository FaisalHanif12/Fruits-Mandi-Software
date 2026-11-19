//! The two mini charts of the dashboard (PRD 9.4): خام بکری and commission for each of the last
//! 30 days, and this calendar month's commission by item (the top 5). Both read the sales facts
//! (PRD FR-18.6), the same rows as the analytics views and R34, summed with the sign of each
//! row, so a cancelled بکری takes itself out.
//!
//! * Commission is seller commission + buyer commission + لاگا. The three columns are summed
//!   apart in SQL and added here with checked maths.
//! * Days with no sales are listed with 0, so the chart always has 30 points, today last.
//! * An item whose commission is not more than 0 after the signs (a cancelled one nets to 0) does
//!   not take a place. The order is commission, biggest first, then the item.
//!
//! Example (PRD 4.9, October): 10-10-26 gross 41,400 and commission 5,448, the other 29 days 0;
//! کیلا 3,928 and سیب 1,520.

use mandi_core::dates::BusinessDate;
use mandi_core::money::Paisa;
use mandi_db::rusqlite::Connection;

use super::model::{CHART_DAYS, DashboardChartDay, DashboardChartItem, TOP_ITEMS};
use crate::error::ReportsError;
use crate::reports::business::day::sales_by_day;
use crate::reports::common::{add, paisa};

/// The day `n` days before `day`.
pub(crate) fn days_before(day: BusinessDate, n: i64) -> Result<BusinessDate, ReportsError> {
    let mut at = day;
    for _ in 0..n {
        at = at
            .previous_day()
            .ok_or_else(|| ReportsError::Data("a day before the calendar".to_owned()))?;
    }
    Ok(at)
}

/// The last [`CHART_DAYS`] days, oldest first, today last.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit.
pub(crate) fn days(
    conn: &Connection,
    today: BusinessDate,
) -> Result<Vec<DashboardChartDay>, ReportsError> {
    let first = days_before(today, CHART_DAYS.saturating_sub(1))?;
    let sales = sales_by_day(conn, &first.to_iso(), &today.to_iso())?;
    let mut out = Vec::new();
    let mut day = first;
    loop {
        let iso = day.to_iso();
        let s = sales.get(&iso).copied().unwrap_or_default();
        let mut commission = Paisa::ZERO;
        for part in s.commission_parts {
            add(&mut commission, paisa(part)?)?;
        }
        out.push(DashboardChartDay {
            date: iso,
            gross_paisa: paisa(s.gross)?,
            commission_paisa: commission,
        });
        if day == today {
            break;
        }
        day = day
            .next_day()
            .ok_or_else(|| ReportsError::Data("a day after the calendar".to_owned()))?;
    }
    Ok(out)
}

/// This calendar month's commission by item, the biggest [`TOP_ITEMS`].
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit; `ReportsError::Data` for a day outside the calendar.
pub(crate) fn items(
    conn: &Connection,
    today: BusinessDate,
) -> Result<Vec<DashboardChartItem>, ReportsError> {
    let month_start = BusinessDate::from_ymd(today.year(), today.month(), 1)
        .ok_or_else(|| ReportsError::Data("the first day of the month".to_owned()))?
        .to_iso();
    let mut statement = conn.prepare(
        "SELECT f.item_id, i.name_ur, i.name_en,
                COALESCE(SUM(f.sign * f.seller_comm_paisa), 0),
                COALESCE(SUM(f.sign * f.buyer_comm_paisa), 0),
                COALESCE(SUM(f.sign * f.laga_paisa), 0)
         FROM analytics_facts f JOIN items i ON i.id = f.item_id
         WHERE f.entry_date >= ?1 AND f.entry_date <= ?2
         GROUP BY f.item_id, i.name_ur, i.name_en",
    )?;
    let rows = statement
        .query_map([&month_start, &today.to_iso()], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                [
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, i64>(5)?,
                ],
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::new();
    for (item_id, name_ur, name_en, parts) in rows {
        let mut commission = Paisa::ZERO;
        for part in parts {
            add(&mut commission, paisa(part)?)?;
        }
        if commission > Paisa::ZERO {
            out.push(DashboardChartItem {
                item_id,
                name_ur,
                name_en,
                commission_paisa: commission,
            });
        }
    }
    out.sort_by(|a, b| {
        b.commission_paisa
            .cmp(&a.commission_paisa)
            .then(a.item_id.cmp(&b.item_id))
    });
    out.truncate(TOP_ITEMS);
    Ok(out)
}
