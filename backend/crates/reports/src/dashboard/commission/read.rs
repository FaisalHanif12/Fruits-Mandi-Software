//! The reads of the commission page (plan 76 point 7, plan 77, PRD FR-9.3): the rows of a period,
//! with or without one party, and the postings behind one day. Read only.

use std::collections::{BTreeMap, HashMap};

use mandi_core::dates::BusinessDate;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::months::Month;
use mandi_db::Db;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, OptionalExtension, params, params_from_iter};

use super::model::{
    CommissionDay, CommissionDayInput, CommissionDayRow, CommissionEntry, CommissionMonth,
    CommissionMonthInput, CommissionParty, CommissionPartyShown, CommissionPeriodKind,
    CommissionPeriodShown, CommissionSide,
};
use super::period::{self, Resolved};
use super::sums::{Day, Shape, buckets_in, partners_in, party_sql};
use crate::error::ReportsError;
use crate::reports::common::{
    chukta_party, in_snapshot, load_parties, load_party, paisa, parse_day, today as today_of,
};
use crate::reports::grouped_sums::chukta_names;
use crate::reports::{PageSpec, page_spec};

/// The most entries a page of one day's detail holds when no limit is sent.
const DETAIL_PAGE: usize = 100;

fn reject(code: ErrorCode, field: &str) -> ReportsError {
    ReportsError::Rejected(vec![FieldError::new(code, field)])
}

/// Checks the party and loads him for the heading of the page: the account must exist and be of
/// the side asked for, and a person key belongs to the چکتا کھاتہ account only.
///
/// # Errors
/// `ERR_NOT_FOUND` (`party.account_id`), `ERR_WRONG_PARTY_TYPE` (`party.account_id`),
/// `ERR_OUT_OF_RANGE` (`party.chukta_person_key`); a database error.
pub(super) fn resolve_party(
    conn: &Connection,
    party: Option<&CommissionParty>,
) -> Result<Option<CommissionPartyShown>, ReportsError> {
    let Some(party) = party else {
        return Ok(None);
    };
    let found: Option<(String, i64)> = conn
        .query_row(
            "SELECT party_type, is_chukta_khata FROM accounts WHERE id = ?1",
            [party.account_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((kind, chukta)) = found else {
        return Err(reject(ErrorCode::ErrNotFound, "party.account_id"));
    };
    // The old closed account 0031 is a seller only by name: it is the چکتا کھاتہ family.
    let right_side = match party.side {
        CommissionSide::Seller => kind == "seller" && chukta == 0,
        CommissionSide::Buyer => kind == "buyer",
    };
    if !right_side {
        return Err(reject(ErrorCode::ErrWrongPartyType, "party.account_id"));
    }
    let key = party
        .chukta_person_key
        .as_deref()
        .map(str::trim)
        .filter(|k| !k.is_empty());
    if key.is_some() && chukta == 0 {
        return Err(reject(ErrorCode::ErrOutOfRange, "party.chukta_person_key"));
    }
    let base = load_party(conn, party.account_id)?
        .ok_or_else(|| reject(ErrorCode::ErrNotFound, "party.account_id"))?;
    let shown = match key {
        Some(key) => {
            let names = chukta_names(conn, &[party.account_id])?;
            let saved = names.get(key);
            chukta_party(
                base,
                Some(key.to_owned()),
                saved.map(|p| p.name_ur.as_str()),
                saved.map(|p| p.name_en.as_str()),
            )
        }
        None => base,
    };
    Ok(Some(CommissionPartyShown {
        side: party.side,
        party: shown,
    }))
}

/// The party with the person key trimmed, as the facts query reads it.
fn narrowed(party: Option<&CommissionParty>) -> Option<CommissionParty> {
    party.map(|p| CommissionParty {
        side: p.side,
        account_id: p.account_id,
        chukta_person_key: p
            .chukta_person_key
            .as_deref()
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .map(str::to_owned),
    })
}

/// Reads the page of a period. `paging` cuts the rows of a range; `None` returns every row.
///
/// # Errors
/// As [`commission_month`].
pub(super) fn read_period(
    conn: &Connection,
    today: BusinessDate,
    input: &CommissionMonthInput,
    paging: Option<PageSpec>,
) -> Result<CommissionMonth, ReportsError> {
    let resolved: Resolved = period::resolve(today, input.month.as_deref(), input.period.as_ref())?;
    let party_shown = resolve_party(conn, input.party.as_ref())?;
    let party = narrowed(input.party.as_ref());
    let shape = Shape::of(party.as_ref());
    let (from, to) = (resolved.from.to_iso(), resolved.to.to_iso());
    // The sums of every day of the range are the same for every page of it, and on ten years they
    // cost a pass over every fact: the first page works them out, the next pages take them from
    // memory while the books are as they were (`result_cache`, robustness finding 3).
    let what = format!("commission|{from}|{to}|{}|{party:?}", resolved.bucket_len());
    let read = crate::result_cache::remembered(conn, &what, || {
        let partners = if shape == Shape::Shop {
            partners_in(conn, &from, &to)?
        } else {
            Vec::new()
        };
        let buckets = buckets_in(conn, &from, &to, resolved.bucket_len(), party.as_ref())?;
        Ok((partners, buckets))
    })?;
    let (partners, buckets) = (&read.0, &read.1);
    let mut sum = Day::default();
    // Only a range is paged; a day, a month and a year are short. The sums of every day are added
    // into the totals, but only the rows of the page are built (robustness finding 3).
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
    let mut rows: Vec<CommissionDayRow> = Vec::new();
    let mut count: usize = 0;
    for (key, day) in buckets {
        if day.is_zero()? {
            continue;
        }
        sum.merge(day)?;
        if window.is_none_or(|(start, end)| count >= start && count < end) {
            rows.push(day.row(key, partners, shape)?);
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
    Ok(CommissionMonth {
        month: Month::of(resolved.from).key(),
        year: resolved.from.year(),
        period: CommissionPeriodShown {
            kind: resolved.kind,
            from,
            to,
        },
        party: party_shown,
        first_month: first_month.and_then(|d| d.get(..7).map(str::to_owned)),
        current_month: Month::of(today).key(),
        totals: sum.row("", partners, shape)?,
        partners: partners.clone(),
        rows,
        total_count,
        offset,
        limit,
        partners_locked: false,
    })
}

/// The commission of a period (plan 76 point 7, plan 77): the rows, one page of them for a range,
/// and the totals over the whole period. Read only.
///
/// # Errors
/// `ERR_REQUIRED`, `ERR_FORMAT_INVALID`, `ERR_DATE_INVALID`, `ERR_DATE_IN_FUTURE` and
/// `ERR_DATE_RANGE` on the period; `ERR_NOT_FOUND`, `ERR_WRONG_PARTY_TYPE` and `ERR_OUT_OF_RANGE`
/// on the party; a database or money error.
pub fn commission_month(
    db: &Db,
    input: &CommissionMonthInput,
) -> Result<CommissionMonth, ReportsError> {
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
/// As [`commission_month`].
pub(super) fn commission_every_row(
    db: &Db,
    input: &CommissionMonthInput,
) -> Result<CommissionMonth, ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            read_period(conn, today, input, None)
        })
    })
}

/// The postings behind one day's commission (plan 76 point 7), of one party when asked
/// (plan 77). Read only.
///
/// # Errors
/// `ERR_DATE_INVALID` and `ERR_DATE_IN_FUTURE` on `date`; the party errors of
/// [`commission_month`]; a database or money error.
pub fn commission_day(db: &Db, input: &CommissionDayInput) -> Result<CommissionDay, ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let date = parse_day(input.date.trim(), today)
                .map_err(|_| reject(ErrorCode::ErrDateInvalid, "date"))?;
            // A day after today has no postings; the month read refuses it the same way
            // (robustness finding 8).
            if date > today {
                return Err(reject(ErrorCode::ErrDateInFuture, "date"));
            }
            let iso = date.to_iso();
            let party_shown = resolve_party(conn, input.party.as_ref())?;
            let party = narrowed(input.party.as_ref());
            let shape = Shape::of(party.as_ref());
            let partners = if shape == Shape::Shop {
                partners_in(conn, &iso, &iso)?
            } else {
                Vec::new()
            };

            // The postings with facts, then (for the whole shop) the income shares of every
            // posting of the day.
            let mut by_entry: BTreeMap<i64, Day> = BTreeMap::new();
            let mut bikri_of: HashMap<i64, i64> = HashMap::new();
            // `?1` is the day; the party values follow from `?2`.
            let (narrow, narrow_args) = party_sql(party.as_ref(), 2);
            let sale = match shape {
                Shape::Buyer => "buyer_value_paisa",
                Shape::Seller => "seller_amount_paisa",
                Shape::Shop => "0",
            };
            let mut args = vec![Value::Text(iso.clone())];
            args.extend(narrow_args.clone());
            let mut facts = conn.prepare(&format!(
                "SELECT journal_entry_id, MIN(bikri_id), SUM(sign * seller_comm_paisa),
                        SUM(sign * buyer_comm_paisa), SUM(sign * laga_paisa), SUM(sign * {sale})
                 FROM analytics_facts WHERE entry_date = ?1{narrow} GROUP BY journal_entry_id"
            ))?;
            for row in facts.query_map(params_from_iter(args.iter()), |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, Option<i64>>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, i64>(5)?,
                ))
            })? {
                let (entry, bikri, seller, buyer, laga, sale) = row?;
                let day = by_entry.entry(entry).or_default();
                day.seller = paisa(seller)?;
                day.buyer = paisa(buyer)?;
                day.laga = paisa(laga)?;
                day.sale = paisa(sale)?;
                if let Some(b) = bikri {
                    bikri_of.insert(entry, b);
                }
            }
            if shape == Shape::Shop {
                let mut shares = conn.prepare(
                    "SELECT journal_entry_id, partner_id, SUM(amount_paisa) FROM partner_share_rows
                     WHERE kind = 'income' AND entry_date = ?1
                     GROUP BY journal_entry_id, partner_id",
                )?;
                for row in shares.query_map([&iso], |r| {
                    Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))
                })? {
                    let (entry, partner, amount) = row?;
                    by_entry
                        .entry(entry)
                        .or_default()
                        .shares
                        .insert(partner, paisa(amount)?);
                }
            }
            let mut empty = Vec::new();
            for (id, day) in &by_entry {
                if day.is_zero()? {
                    empty.push(*id);
                }
            }
            for id in empty {
                by_entry.remove(&id);
            }

            let mut totals = Day::default();
            for day in by_entry.values() {
                totals.merge(day)?;
            }
            let ids: Vec<i64> = by_entry.keys().copied().collect();
            let total_count = i64::try_from(ids.len()).unwrap_or(i64::MAX);
            let limit = if input.limit == 0 {
                DETAIL_PAGE
            } else {
                input.limit as usize
            };
            let page: Vec<i64> = ids
                .iter()
                .skip(input.offset as usize)
                .take(limit)
                .copied()
                .collect();

            // What the page's postings are, who sold and who bought (of the party's lines only).
            let mut info: HashMap<i64, (String, String, i64)> = HashMap::new();
            let mut sellers: HashMap<i64, Vec<i64>> = HashMap::new();
            let mut buyers: HashMap<i64, Vec<i64>> = HashMap::new();
            let mut numbers: HashMap<i64, i64> = HashMap::new();
            {
                let mut entry = conn.prepare(
                    "SELECT posting_type, source_type, source_id FROM journal_entries WHERE id = ?1",
                )?;
                // `?1` is the posting, `?2` the day; the party values follow from `?3`.
                let (who_narrow, who_args) = party_sql(party.as_ref(), 3);
                let mut who = conn.prepare(&format!(
                    "SELECT DISTINCT seller_account_id, buyer_account_id FROM analytics_facts
                     WHERE journal_entry_id = ?1 AND entry_date = ?2{who_narrow}"
                ))?;
                let mut number = conn.prepare("SELECT number FROM bikri_entries WHERE id = ?1")?;
                for id in &page {
                    let (posting, source, source_id) = entry.query_row([id], |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, i64>(2)?,
                        ))
                    })?;
                    info.insert(*id, (posting, source, source_id));
                    let mut who_values = vec![Value::Integer(*id), Value::Text(iso.clone())];
                    who_values.extend(who_args.clone());
                    for row in who.query_map(params_from_iter(who_values.iter()), |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
                    })? {
                        let (seller, buyer) = row?;
                        let s = sellers.entry(*id).or_default();
                        if !s.contains(&seller) {
                            s.push(seller);
                        }
                        let b = buyers.entry(*id).or_default();
                        if !b.contains(&buyer) {
                            b.push(buyer);
                        }
                    }
                    if let Some(b) = bikri_of.get(id) {
                        let n: Option<i64> = number
                            .query_row(params![b], |r| r.get(0))
                            .optional()?;
                        if let Some(n) = n {
                            numbers.insert(*b, n);
                        }
                    }
                }
            }
            let mut account_ids: Vec<i64> = sellers
                .values()
                .chain(buyers.values())
                .flatten()
                .copied()
                .collect();
            account_ids.sort_unstable();
            account_ids.dedup();
            let parties = load_parties(conn, &account_ids)?;
            let party_of = |id: &i64| parties.get(id).cloned();
            let mut entries = Vec::with_capacity(page.len());
            for id in &page {
                let day = by_entry.get(id).cloned().unwrap_or_default();
                let (posting_type, source_type, source_id) =
                    info.get(id).cloned().ok_or_else(|| {
                        ReportsError::Data("a posting of the day is missing".to_owned())
                    })?;
                let bikri_id = bikri_of.get(id).copied();
                entries.push(CommissionEntry {
                    entry_id: *id,
                    posting_type,
                    source_type,
                    source_id,
                    bikri_id,
                    bikri_number: bikri_id.and_then(|b| numbers.get(&b).copied()),
                    sellers: sellers
                        .get(id)
                        .map(|v| v.iter().filter_map(party_of).collect())
                        .unwrap_or_default(),
                    buyers: buyers
                        .get(id)
                        .map(|v| v.iter().filter_map(party_of).collect())
                        .unwrap_or_default(),
                    figures: day.row(&iso, &partners, shape)?,
                });
            }
            Ok(CommissionDay {
                date: iso.clone(),
                party: party_shown,
                totals: totals.row(&iso, &partners, shape)?,
                partners,
                entries,
                total_count,
                partners_locked: false,
            })
        })
    })
}
