//! The attention items that count things and need no money: unsaved drafts, earlier days that are
//! not closed, lots waiting in باقی مال, the cloud backup and the records with a missing name
//! (PRD FR-7.44). Each function gives its item or `None` when there is nothing to do, and
//! nothing is written (Code Standards B29).
//!
//! Example (PRD 4.9, today 10-10-26, closed up to 09-10-26, nothing waiting): no item at all.
//! The same book on 20-10-26 with nothing closed since: 10 earlier days are not closed.

use mandi_core::dates::BusinessDate;
use mandi_db::bikri::edit::{StockFilter, stock_lots};
use mandi_db::purchases::{ListFilter, StateFilter};
use mandi_db::rusqlite::{Connection, OptionalExtension};

use super::model::{
    AttentionFirst, AttentionItem, AttentionKind, AttentionTarget, BACKUP_OLD_HOURS, BackupFacts,
    STOCK_WAIT_DAYS,
};
use crate::error::ReportsError;

/// An item that only counts.
fn counted(kind: AttentionKind, count: i64) -> AttentionItem {
    AttentionItem {
        kind,
        count,
        amount_paisa: None,
        qty: None,
        days: None,
        first: None,
    }
}

/// Unsaved drafts (PRD FR-12.10).
pub(crate) fn drafts(conn: &Connection) -> Result<Option<AttentionItem>, ReportsError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM drafts", [], |r| r.get(0))?;
    Ok((count > 0).then(|| counted(AttentionKind::Drafts, count)))
}

/// Earlier days that are not closed: the calendar days after the "closed up to" day and before
/// today. If no day was ever closed, they start on the first day with a posting (an opening
/// balance is not a day of work). `days` is the age of the oldest of them.
pub(crate) fn open_days(
    conn: &Connection,
    today: BusinessDate,
) -> Result<Option<AttentionItem>, ReportsError> {
    let closed: Option<String> = conn
        .query_row(
            "SELECT closed_up_to_date FROM day_state WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    let first_open = match closed {
        Some(text) => BusinessDate::parse_iso(&text)
            .ok_or_else(|| ReportsError::Data(format!("closed_up_to_date {text}")))?
            .next_day(),
        None => {
            let first: Option<String> = conn.query_row(
                "SELECT MIN(entry_date) FROM journal_entries
                 WHERE status = 'posted' AND source_type NOT IN ('month_close', 'account')
                   AND posting_type <> 'opening'",
                [],
                |r| r.get(0),
            )?;
            match first {
                Some(text) => BusinessDate::parse_iso(&text),
                None => return Ok(None),
            }
        }
    };
    let Some(first_open) = first_open else {
        return Ok(None);
    };
    let days = today.days_after(first_open);
    if days <= 0 {
        return Ok(None);
    }
    let mut item = counted(AttentionKind::OpenDays, days);
    item.days = Some(days);
    Ok(Some(item))
}

/// Lots waiting in باقی مال for more than [`STOCK_WAIT_DAYS`] days, the lots of بکری and the
/// purchases of trips both, as the باقی مال screen lists them (PRD FR-12.14, FR-12.25).
/// `qty` is their نگ left, `days` the age of the oldest, `first` the oldest one.
pub(crate) fn stock_waiting(
    conn: &Connection,
    today: BusinessDate,
) -> Result<Option<AttentionItem>, ReportsError> {
    // (days waiting, balance, record to open)
    let mut waiting: Vec<(i64, i64, AttentionFirst)> = Vec::new();
    let age = |received: &str| -> i64 {
        BusinessDate::parse_iso(received).map_or(0, |day| today.days_after(day).max(0))
    };
    for lot in stock_lots(conn, &StockFilter::default())? {
        let balance = lot
            .qty_received
            .checked_sub(lot.qty_sold)
            .and_then(|v| v.checked_sub(lot.qty_written_off))
            .ok_or_else(|| ReportsError::Data("the quantity of a lot does not fit".to_owned()))?;
        let days = age(&lot.received_date);
        if balance > 0 && days > STOCK_WAIT_DAYS {
            let (name_ur, name_en) = match &lot.pukhta {
                Some((ur, en, _, _)) => (ur.clone(), en.clone()),
                None => (lot.seller.name_ur.clone(), lot.seller.name_en.clone()),
            };
            waiting.push((
                days,
                balance,
                AttentionFirst {
                    target: AttentionTarget::Bikri,
                    id: lot.bikri_id,
                    person_key: None,
                    name_ur: Some(name_ur),
                    name_en: Some(name_en),
                },
            ));
        }
    }
    let purchases = mandi_db::purchases::list(
        conn,
        &ListFilter {
            trip_id: None,
            from: None,
            to: None,
            item_id: None,
            state: Some(StateFilter::InStock),
        },
        0,
        -1,
    )?;
    for purchase in purchases {
        let balance = purchase
            .qty
            .checked_sub(purchase.qty_sold)
            .and_then(|v| v.checked_sub(purchase.qty_written_off))
            .ok_or_else(|| {
                ReportsError::Data("the quantity of a purchase does not fit".to_owned())
            })?;
        let days = age(&purchase.entry_date);
        if balance > 0 && days > STOCK_WAIT_DAYS {
            waiting.push((
                days,
                balance,
                AttentionFirst {
                    target: AttentionTarget::Purchase,
                    id: purchase.id,
                    person_key: None,
                    name_ur: Some(purchase.item.name_ur.clone()),
                    name_en: Some(purchase.item.name_en.clone()),
                },
            ));
        }
    }
    // The oldest first; on a tie the biggest balance.
    waiting.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
    let mut qty = 0_i64;
    for (_, balance, _) in &waiting {
        qty = qty
            .checked_add(*balance)
            .ok_or_else(|| ReportsError::Data("the waiting stock does not fit".to_owned()))?;
    }
    let Some((days, _, first)) = waiting.first().cloned() else {
        return Ok(None);
    };
    let count = i64::try_from(waiting.len())
        .map_err(|_| ReportsError::Data("the waiting lots do not fit".to_owned()))?;
    let mut item = counted(AttentionKind::StockWaiting, count);
    item.qty = Some(qty);
    item.days = Some(days);
    item.first = Some(first);
    Ok(Some(item))
}

/// The cloud copy is older than [`BACKUP_OLD_HOURS`] hours, or there is none, while the cloud copy
/// is switched on (PRD FR-7.44, NFR-BAK-10). `days` is the age in whole days; none for no copy.
pub(crate) fn backup_old(backup: BackupFacts) -> Option<AttentionItem> {
    if !backup.cloud_on {
        return None;
    }
    match backup.cloud_age_hours {
        Some(hours) if hours <= BACKUP_OLD_HOURS => None,
        Some(hours) => {
            let mut item = counted(AttentionKind::BackupOld, 1);
            item.days = Some(hours / 24);
            Some(item)
        }
        None => Some(counted(AttentionKind::BackupOld, 1)),
    }
}

/// Records with a missing Urdu or English name: the rows whose name is empty after trimming
/// (possible only after the Phase 2 import of old data, PRD FR-7.44, E41).
pub(crate) fn names_missing(conn: &Connection) -> Result<Option<AttentionItem>, ReportsError> {
    const TABLES: [&str; 9] = [
        "accounts",
        "cities",
        "items",
        "packings",
        "addas",
        "stores",
        "expense_heads",
        "staff",
        "partners",
    ];
    let mut count = 0_i64;
    for table in TABLES {
        // The table name is one of the constants above (Code Standards B14).
        let found: i64 = conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM {table} WHERE trim(name_ur) = '' OR trim(name_en) = ''"
            ),
            [],
            |r| r.get(0),
        )?;
        count = count
            .checked_add(found)
            .ok_or_else(|| ReportsError::Data("the records do not fit".to_owned()))?;
    }
    Ok((count > 0).then(|| counted(AttentionKind::NamesMissing, count)))
}
