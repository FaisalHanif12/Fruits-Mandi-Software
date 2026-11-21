//! The reads of the sales page (plan 83, PRD FR-9.9): the rows of a period, with or without a party,
//! an item or some goods, and the بکری behind one day. Read only.

use std::collections::{BTreeMap, HashMap};

use mandi_core::accounts::RecordRef;
use mandi_core::bikri::LotName;
use mandi_core::dates::BusinessDate;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::months::Month;
use mandi_db::Db;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, OptionalExtension, params_from_iter};

use super::model::{SalesDay, SalesDayInput, SalesDayRow, SalesPeriod, SalesPeriodInput};
use super::sums::{DAY_LEN, Figures, bikri_in_buckets, buckets_in, narrow_sql};
use crate::analytics::AnalyticsGoods;
use crate::dashboard::commission::{
    CommissionParty, CommissionPeriodKind, CommissionPeriodShown, Resolved, narrowed,
    resolve_party, resolve_period,
};
use crate::error::ReportsError;
use crate::reports::common::{in_snapshot, parse_day, today as today_of};
use crate::reports::{PageSpec, page_spec};

fn reject(code: ErrorCode, field: &str) -> ReportsError {
    ReportsError::Rejected(vec![FieldError::new(code, field)])
}

/// Checks the item and loads it for the heading of the page.
///
/// # Errors
/// `ERR_NOT_FOUND` on `item_id`; a database error.
pub(super) fn resolve_item(
    conn: &Connection,
    item_id: Option<i64>,
) -> Result<Option<RecordRef>, ReportsError> {
    let Some(id) = item_id else {
        return Ok(None);
    };
    let found = conn
        .query_row(
            "SELECT id, code, name_ur, name_en FROM items WHERE id = ?1",
            [id],
            |r| {
                Ok(RecordRef {
                    id: r.get(0)?,
                    code: r.get(1)?,
                    name_ur: r.get(2)?,
                    name_en: r.get(3)?,
                    tag: None,
                })
            },
        )
        .optional()?;
    found
        .map(Some)
        .ok_or_else(|| reject(ErrorCode::ErrNotFound, "item_id"))
}

/// Reads the page of a period. `paging` cuts the rows of a range; `None` returns every row.
///
/// # Errors
/// As [`sales_period`].
pub(super) fn read_period(
    conn: &Connection,
    today: BusinessDate,
    input: &SalesPeriodInput,
    paging: Option<PageSpec>,
) -> Result<SalesPeriod, ReportsError> {
    let resolved: Resolved = resolve_period(today, None, input.period.as_ref())?;
    let party_shown = resolve_party(conn, input.party.as_ref())?;
    let party = narrowed(input.party.as_ref());
    let item = resolve_item(conn, input.item_id)?;
    let goods = input.goods.unwrap_or_default();
    let (from, to) = (resolved.from.to_iso(), resolved.to.to_iso());
    let (narrow, narrow_args) = narrow_sql(party.as_ref(), input.item_id, goods, 3);
    // The sums of every day of the range are the same for every page of it, and on ten years they
    // cost a pass over every fact: the first page works them out, the next pages take them from
    // memory while the books are as they were (`result_cache`).
    let what = format!(
        "sales|{from}|{to}|{}|{party:?}|{:?}|{goods:?}",
        resolved.bucket_len(),
        input.item_id
    );
    let buckets = crate::result_cache::remembered(conn, &what, || {
        buckets_in(
            conn,
            &from,
            &to,
            resolved.bucket_len(),
            &narrow,
            &narrow_args,
        )
    })?;
    let mut sum = Figures::default();
    // Only a range is paged; a day, a month and a year are short. The figures of every bucket are
    // added into the totals, but only the rows of the page are built.
    let paged = match (resolved.kind, paging) {
        (CommissionPeriodKind::Range, Some(spec)) => Some(spec),
        _ => None,
    };
    let window = paged.map(|spec| {
        let start = usize::try_from(spec.offset).unwrap_or(usize::MAX);
        let end = spec.limit.map_or(usize::MAX, |l| {
            start.saturating_add(usize::try_from(l).unwrap_or(usize::MAX))
        });
        (start, end)
    });
    let mut rows = Vec::new();
    let mut count: usize = 0;
    for (key, figures) in buckets.iter() {
        if figures.is_zero() {
            continue;
        }
        sum.merge(figures)?;
        if window.is_none_or(|(start, end)| count >= start && count < end) {
            rows.push(figures.row(key)?);
        }
        count = count.saturating_add(1);
    }
    let total_count = i64::try_from(count).unwrap_or(i64::MAX);
    let (offset, limit) = match paged {
        Some(spec) => (
            u32::try_from(spec.offset).unwrap_or(u32::MAX),
            u32::try_from(spec.reported_limit()).unwrap_or(0),
        ),
        None => (0, 0),
    };
    let first_month: Option<String> =
        conn.query_row("SELECT MIN(entry_date) FROM analytics_facts", [], |r| {
            r.get::<_, Option<String>>(0)
        })?;
    Ok(SalesPeriod {
        month: Month::of(resolved.from).key(),
        year: resolved.from.year(),
        period: CommissionPeriodShown {
            kind: resolved.kind,
            from,
            to,
        },
        party: party_shown,
        item,
        goods,
        first_month: first_month.and_then(|d| d.get(..7).map(str::to_owned)),
        current_month: Month::of(today).key(),
        today: today.to_iso(),
        totals: sum.row("")?,
        rows,
        total_count,
        offset,
        limit,
    })
}

/// The sales of a period (plan 83, PRD FR-9.9): the rows, one page of them for a range, and the
/// totals over the whole period. Read only.
///
/// # Errors
/// `ERR_REQUIRED`, `ERR_FORMAT_INVALID`, `ERR_DATE_INVALID`, `ERR_DATE_IN_FUTURE` and
/// `ERR_DATE_RANGE` on the period; `ERR_NOT_FOUND`, `ERR_WRONG_PARTY_TYPE` and `ERR_OUT_OF_RANGE`
/// on the party; `ERR_NOT_FOUND` on `item_id`; `ERR_OUT_OF_RANGE` on `offset` and `limit`; a
/// database or money error.
pub fn sales_period(db: &Db, input: &SalesPeriodInput) -> Result<SalesPeriod, ReportsError> {
    let spec = page_spec(
        i64::from(input.offset.unwrap_or(0)),
        i64::from(input.limit.unwrap_or(0)),
    )?;
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            read_period(conn, today, input, Some(spec))
        })
    })
}

/// Every row of a period, for a print or a sheet (never paged). Read only.
///
/// # Errors
/// As [`sales_period`].
pub(super) fn sales_every_row(
    db: &Db,
    input: &SalesPeriodInput,
) -> Result<SalesPeriod, ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            read_period(conn, today, input, None)
        })
    })
}

/// The seller and lot names of the بکری of one day, for the rows of a page: each lot that has sales
/// on the day under the same narrowing as the sums, in lot order.
fn lots_of(
    conn: &Connection,
    iso: &str,
    bikri_ids: &[i64],
    party: Option<&CommissionParty>,
    item_id: Option<i64>,
    goods: AnalyticsGoods,
) -> Result<HashMap<i64, Vec<LotName>>, ReportsError> {
    let mut out: HashMap<i64, Vec<LotName>> = HashMap::new();
    if bikri_ids.is_empty() {
        return Ok(out);
    }
    // `?1` is the day, `?2` on are the بکری; the narrowing values follow them.
    let (narrow_here, narrow_args) =
        narrow_sql(party, item_id, goods, bikri_ids.len().saturating_add(2));
    let marks = (0..bikri_ids.len())
        .map(|i| format!("?{}", i.saturating_add(2)))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT f.bikri_id, l.id, l.lot_counter, sa.id, sa.code, sa.name_ur, sa.name_en, sa.tag,
                it.id, it.code, it.name_ur, it.name_en
         FROM analytics_facts f
         JOIN bikri_lots l ON l.id = f.lot_id
         JOIN accounts sa ON sa.id = l.seller_account_id
         JOIN items it ON it.id = l.item_id
         WHERE f.entry_date = ?1 AND f.bikri_id IN ({marks}){narrow_here}
         GROUP BY f.bikri_id, l.id HAVING SUM(f.sign) > 0
         ORDER BY f.bikri_id, l.lot_no"
    );
    let mut args = vec![Value::Text(iso.to_owned())];
    args.extend(bikri_ids.iter().map(|id| Value::Integer(*id)));
    args.extend(narrow_args);
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params_from_iter(args.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                LotName {
                    lot_id: r.get(1)?,
                    counter: r.get(2)?,
                    seller: RecordRef {
                        id: r.get(3)?,
                        code: r.get(4)?,
                        name_ur: r.get(5)?,
                        name_en: r.get(6)?,
                        tag: r.get(7)?,
                    },
                    item: RecordRef {
                        id: r.get(8)?,
                        code: r.get(9)?,
                        name_ur: r.get(10)?,
                        name_en: r.get(11)?,
                        tag: None,
                    },
                },
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (bikri_id, lot) in rows {
        out.entry(bikri_id).or_default().push(lot);
    }
    Ok(out)
}

/// The بکری of one day with their figures of that day (plan 83, PRD FR-9.9). Read only.
///
/// # Errors
/// `ERR_REQUIRED`, `ERR_DATE_INVALID` and `ERR_DATE_IN_FUTURE` on `date`; the party and item errors
/// of [`sales_period`]; `ERR_OUT_OF_RANGE` on `offset` and `limit`; a database or money error.
pub fn sales_day(db: &Db, input: &SalesDayInput) -> Result<SalesDay, ReportsError> {
    let spec = page_spec(i64::from(input.offset), i64::from(input.limit))?;
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let text = input.date.trim();
            if text.is_empty() {
                return Err(reject(ErrorCode::ErrRequired, "date"));
            }
            let date: BusinessDate = parse_day(text, today).map_err(|code| reject(code, "date"))?;
            if date > today {
                return Err(reject(ErrorCode::ErrDateInFuture, "date"));
            }
            let iso = date.to_iso();
            let party_shown = resolve_party(conn, input.party.as_ref())?;
            let party = narrowed(input.party.as_ref());
            let item = resolve_item(conn, input.item_id)?;
            let goods = input.goods.unwrap_or_default();
            let (narrow, narrow_args) = narrow_sql(party.as_ref(), input.item_id, goods, 3);

            // The same query as the period page, on this one day: so the rows add up to the day's
            // row of the period page.
            let mut figures: BTreeMap<i64, Figures> = BTreeMap::new();
            for (_, one) in bikri_in_buckets(conn, &iso, &iso, DAY_LEN, &narrow, &narrow_args)? {
                if one.figures.count > 0 {
                    figures
                        .entry(one.bikri_id)
                        .or_default()
                        .merge(&one.figures)?;
                }
            }
            let ids: Vec<i64> = figures.keys().copied().collect();
            let numbers = numbers_of(conn, &ids)?;
            let mut totals = Figures::default();
            let mut ordered: Vec<(i64, i64, Figures)> = Vec::with_capacity(ids.len());
            for (id, one) in &figures {
                totals.merge(one)?;
                ordered.push((numbers.get(id).copied().unwrap_or_default(), *id, *one));
            }
            ordered.sort_unstable_by_key(|(number, id, _)| (*number, *id));
            let total_count = i64::try_from(ordered.len()).unwrap_or(i64::MAX);
            let page = spec.slice(&ordered);
            let page_ids: Vec<i64> = page.iter().map(|(_, id, _)| *id).collect();
            let mut lots = lots_of(conn, &iso, &page_ids, party.as_ref(), input.item_id, goods)?;
            let mut rows = Vec::with_capacity(page.len());
            for (number, id, one) in page {
                let lots_here = lots.remove(&id).unwrap_or_default();
                let mut sellers: Vec<RecordRef> = Vec::new();
                for lot in &lots_here {
                    if !sellers.iter().any(|s| s.id == lot.seller.id) {
                        sellers.push(lot.seller.clone());
                    }
                }
                let line = one.row(&iso)?;
                rows.push(SalesDayRow {
                    bikri_id: id,
                    bikri_number: number,
                    sellers,
                    lots: lots_here,
                    qty: line.qty,
                    gross_paisa: line.gross_paisa,
                    buyer_total_paisa: line.buyer_total_paisa,
                    cash_paisa: line.cash_paisa,
                    credit_paisa: line.credit_paisa,
                });
            }
            Ok(SalesDay {
                date: iso.clone(),
                party: party_shown,
                item,
                goods,
                totals: totals.row(&iso)?,
                rows,
                total_count,
            })
        })
    })
}

/// The numbers of some بکری, in one statement.
fn numbers_of(conn: &Connection, ids: &[i64]) -> Result<HashMap<i64, i64>, ReportsError> {
    let mut out = HashMap::with_capacity(ids.len());
    if ids.is_empty() {
        return Ok(out);
    }
    let marks = vec!["?"; ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT id, number FROM bikri_entries WHERE id IN ({marks})"
    ))?;
    let rows = statement
        .query_map(params_from_iter(ids.iter()), |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    out.extend(rows);
    Ok(out)
}
