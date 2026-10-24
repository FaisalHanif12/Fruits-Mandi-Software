//! Today's بکری list of the dashboard (PRD 9.3). One row for each بکری dated today, oldest number
//! first, with the figures its lots keep (خام بکری and صافی بکری are stored at save and never
//! worked out again, Code Standards B20). A cancelled بکری is listed with its status.
//!
//! Example (PRD 4.9, after Events 1 and 2 on 10-10-26): بکری 1, LES-1234, رحیم, 29,400, 19,316;
//! بکری 2, LHR-0077, کریم, 12,000, 10,160.

use std::collections::HashMap;

use mandi_db::rusqlite::{Connection, params, params_from_iter};

use super::model::{DashboardBikri, MAX_TODAY_ROWS};
use crate::error::ReportsError;
use crate::reports::Names;
use crate::reports::common::{IN_CHUNK, paisa, placeholders};

/// The first seller of a بکری, by its first lot.
type FirstSellers = HashMap<i64, Names>;

/// The seller of the first lot of each of these بکری; a one-time seller shows his own name.
fn first_sellers(conn: &Connection, ids: &[i64]) -> Result<FirstSellers, ReportsError> {
    let mut out = HashMap::new();
    for chunk in ids.chunks(IN_CHUNK) {
        let sql = format!(
            "SELECT l.bikri_id, COALESCE(l.pukhta_name_ur, a.name_ur), COALESCE(l.pukhta_name_en, a.name_en)
             FROM bikri_lots l JOIN accounts a ON a.id = l.seller_account_id
             WHERE l.bikri_id IN ({}) AND l.lot_no = (SELECT MIN(m.lot_no) FROM bikri_lots m
                                                      WHERE m.bikri_id = l.bikri_id)",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                Names {
                    ur: r.get(1)?,
                    en: r.get(2)?,
                },
            ))
        })?;
        for row in rows {
            let (id, names) = row?;
            out.insert(id, names);
        }
    }
    Ok(out)
}

/// Today's بکری: the rows (at most [`MAX_TODAY_ROWS`]) and how many there are in all.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees.
pub(crate) fn bikri_today(
    conn: &Connection,
    today: &str,
) -> Result<(Vec<DashboardBikri>, i64), ReportsError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bikri_entries WHERE entry_date = ?1",
        [today],
        |r| r.get(0),
    )?;
    let limit = i64::try_from(MAX_TODAY_ROWS).unwrap_or(i64::MAX);
    let mut statement = conn.prepare(
        "SELECT b.id, b.number, b.truck_no, b.status = 'cancelled',
                COALESCE(SUM(l.gross_paisa), 0), COALESCE(SUM(l.net_paisa), 0),
                COUNT(DISTINCT l.seller_account_id || '|' || COALESCE(l.pukhta_person_key, ''))
         FROM bikri_entries b LEFT JOIN bikri_lots l ON l.bikri_id = b.id
         WHERE b.entry_date = ?1
         GROUP BY b.id ORDER BY b.number, b.id LIMIT ?2",
    )?;
    let stored = statement
        .query_map(params![today, limit], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, bool>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let ids: Vec<i64> = stored.iter().map(|row| row.0).collect();
    let sellers = first_sellers(conn, &ids)?;
    let mut rows = Vec::with_capacity(stored.len());
    for (bikri_id, number, truck_no, cancelled, gross, net, seller_count) in stored {
        rows.push(DashboardBikri {
            bikri_id,
            number,
            truck_no: truck_no.filter(|t| !t.trim().is_empty()),
            seller: sellers.get(&bikri_id).cloned(),
            seller_count,
            gross_paisa: paisa(gross)?,
            net_paisa: paisa(net)?,
            cancelled,
        });
    }
    Ok((rows, count))
}
