//! The reads of trip purchases: the list, one purchase, and the stock rows of باقی مال (PRD
//! FR-12.25, FR-12.27, plan 16b). Every figure is read from the stored rows; nothing is worked out
//! again except sums and differences, with checked maths (Code Standards B20).

use std::collections::HashMap;

use mandi_db::purchases::{self as repo, ListFilter, PurchaseRec, SoldFigures, StateFilter};
use mandi_db::rusqlite::Connection;
use mandi_db::ventures as venture_repo;
use mandi_domain::core::bikri::{OwnStockRow, WriteOffRow};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::money::{Paisa, Qty, avg_rate};
use mandi_domain::core::purchases::{
    PurchaseGetInput, PurchaseLotRef, PurchaseRow, PurchaseStateFilter, PurchaseStatus,
    PurchasesList, PurchasesListInput, PurchasesTotals,
};
use mandi_domain::core::rules::{Working, WorkingArg, WorkingValue};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::ventures::{EntryPart, MoneySource, TripRef};
use mandi_domain::ventures::payment_working;

use super::{data, not_found};
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};

/// The default page of a list (PRD FR-19.11).
const DEFAULT_PAGE: u32 = 100;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

fn money_arg(name: &str, value: Paisa) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value: WorkingValue::Money { paisa: value },
    }
}

/// An optional date of a filter: empty or missing is no limit.
fn optional_date(
    errors: &mut FieldErrors,
    field: &str,
    text: Option<&String>,
    today: BusinessDate,
) -> Option<String> {
    let text = text.map(|t| t.trim()).filter(|t| !t.is_empty())?;
    errors
        .take(validate::date(field, text, today))
        .map(BusinessDate::to_iso)
}

/// One purchase as the screens show it, with its figures. `parts` is filled by the caller that
/// reads the partners' parts (`purchases_get`).
fn row_of(
    rec: &PurchaseRec,
    figures: SoldFigures,
    today: BusinessDate,
) -> Result<PurchaseRow, ServiceError> {
    let cancelled = rec.status == "cancelled";
    let qty_left = if cancelled {
        0
    } else {
        rec.qty
            .checked_sub(rec.qty_sold)
            .and_then(|v| v.checked_sub(rec.qty_written_off))
            .ok_or_else(|| data("the quantity of a purchase does not fit"))?
    };
    let released = paisa(rec.cost_released_paisa)?;
    let total = paisa(rec.total_cost_paisa)?;
    let cost_left = if cancelled {
        Paisa::ZERO
    } else {
        total.checked_sub(released).map_err(money_error)?
    };
    let (sales, other_shop, cost_written_off) = if cancelled {
        (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO)
    } else {
        (
            paisa(figures.sales_paisa)?,
            paisa(figures.other_shop_paisa)?,
            paisa(figures.cost_written_off_paisa)?,
        )
    };
    let released = if cancelled { Paisa::ZERO } else { released };
    // What was released and not written off was sold, in our shop or in another.
    let cost_sold = released
        .checked_sub(cost_written_off)
        .map_err(money_error)?;
    let result = sales
        .checked_add(other_shop)
        .and_then(|v| v.checked_sub(released))
        .map_err(money_error)?;
    let days_waiting = if qty_left > 0 {
        BusinessDate::parse_iso(&rec.entry_date).map_or(0, |d| today.days_after(d).max(0))
    } else {
        0
    };
    let avg = Qty::new(rec.qty)
        .map_err(money_error)
        .and_then(|q| avg_rate(total, q).map_err(money_error))?;
    let carries_cost = rec.trip_cost_paisa > 0;
    Ok(PurchaseRow {
        id: rec.id,
        version: rec.version,
        number: rec.number,
        trip: TripRef {
            id: rec.trip_id,
            number: rec.trip_number,
            code: mandi_domain::core::codes::number_code(
                mandi_domain::core::codes::CodeType::Trip,
                rec.trip_number,
            ),
            name_ur: rec.trip_name_ur.clone(),
            name_en: rec.trip_name_en.clone(),
        },
        entry_date: rec.entry_date.clone(),
        status: PurchaseStatus::parse(&rec.status)
            .ok_or_else(|| data("a purchase has a status that is not saved or cancelled"))?,
        cancel_reason: rec.cancel_reason.clone(),
        supplier: rec.supplier_text.clone(),
        item: rec.item.clone(),
        packing: rec.packing.clone(),
        qty: rec.qty,
        weight_grams: rec.weight_grams,
        rate_paisa: paisa(rec.rate_paisa)?,
        value_paisa: paisa(rec.value_paisa)?,
        freight_paisa: paisa(rec.freight_paisa)?,
        labour_paisa: paisa(rec.labour_paisa)?,
        other_cost_paisa: paisa(rec.other_cost_paisa)?,
        paid_now_paisa: paisa(rec.paid_now_paisa)?,
        trip_cost_paisa: paisa(rec.trip_cost_paisa)?,
        total_cost_paisa: total,
        avg_cost_paisa: avg,
        note: rec.note.clone(),
        entry_id: rec.entry_id,
        parts: Vec::new(),
        qty_sold: rec.qty_sold,
        qty_written_off: rec.qty_written_off,
        qty_left,
        cost_released_paisa: released,
        cost_left_paisa: cost_left,
        sales_paisa: sales,
        other_shop_paisa: other_shop,
        cost_sold_paisa: cost_sold,
        cost_written_off_paisa: cost_written_off,
        result_paisa: result,
        result_working: Working {
            message_id: "working-purchase-result".to_owned(),
            args: vec![
                money_arg("sales", sales),
                money_arg("other_shop", other_shop),
                money_arg("cost_sold", cost_sold),
                money_arg("cost_written_off", cost_written_off),
                money_arg("result", result),
            ],
            rule_id: None,
        },
        days_waiting,
        can_cancel: !cancelled && rec.qty_sold == 0 && rec.qty_written_off == 0 && !carries_cost,
        lots: Vec::new(),
        write_offs: Vec::new(),
    })
}

fn write_off_rows(off: Vec<repo::PurchaseWriteOff>) -> HashMap<i64, Vec<WriteOffRow>> {
    let mut by_purchase: HashMap<i64, Vec<WriteOffRow>> = HashMap::new();
    for w in off {
        by_purchase
            .entry(w.purchase_id)
            .or_default()
            .push(WriteOffRow {
                move_id: w.move_id,
                move_date: w.move_date,
                qty: w.qty,
                reason: w.reason,
            });
    }
    by_purchase
}

/// The rows of purchases, with their figures from three grouped queries (Code Standards B22).
pub(super) fn rows_of(
    conn: &Connection,
    recs: &[PurchaseRec],
    today: BusinessDate,
) -> Result<Vec<PurchaseRow>, ServiceError> {
    let ids: Vec<i64> = recs.iter().map(|r| r.id).collect();
    let figures = repo::sold_figures(conn, &ids)?;
    recs.iter()
        .map(|rec| {
            row_of(
                rec,
                figures.get(&rec.id).copied().unwrap_or_default(),
                today,
            )
        })
        .collect()
}

/// The partners' parts of the payments of these purchases, by purchase id (PRD FR-12.30).
fn parts_of(
    conn: &Connection,
    purchase_ids: &[i64],
) -> Result<HashMap<i64, Vec<EntryPart>>, ServiceError> {
    let mut out: HashMap<i64, Vec<EntryPart>> = HashMap::new();
    for id in purchase_ids {
        let Some(entry) = venture_repo::entry_of_purchase(conn, *id)? else {
            continue;
        };
        let amount = paisa(entry.amount_paisa)?;
        for part in venture_repo::parts_of(conn, &[entry.id])? {
            let share = i32::try_from(part.share_bps)
                .map_err(|_| data("a stored share is out of range"))?;
            let part_amount = paisa(part.amount_paisa)?;
            out.entry(*id).or_default().push(EntryPart {
                partner: part.partner,
                share_bps: share,
                source: MoneySource::parse(&part.source)
                    .ok_or_else(|| data("a stored money source is not shop or pocket"))?,
                amount_paisa: part_amount,
                working: payment_working(amount, share, part_amount),
            });
        }
    }
    Ok(out)
}

/// Every purchase of a trip, oldest first, with its partners' parts (for `trips_get`).
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data`.
pub(crate) fn rows_of_trip(
    conn: &Connection,
    trip_id: i64,
    today: BusinessDate,
) -> Result<Vec<PurchaseRow>, ServiceError> {
    let filter = ListFilter {
        trip_id: Some(trip_id),
        ..ListFilter::default()
    };
    let mut recs = repo::list(conn, &filter, 0, i64::from(i32::MAX))?;
    recs.sort_by_key(|r| r.id);
    let mut rows = rows_of(conn, &recs, today)?;
    let ids: Vec<i64> = recs.iter().map(|r| r.id).collect();
    let mut parts = parts_of(conn, &ids)?;
    for row in &mut rows {
        row.parts = parts.remove(&row.id).unwrap_or_default();
    }
    Ok(rows)
}

/// One page of the purchases that match, with the totals over the whole filter.
pub(super) fn list(
    ctx: &Context,
    input: &PurchasesListInput,
) -> Result<PurchasesList, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let from = optional_date(&mut errors, "from", input.from.as_ref(), today);
    let to = optional_date(&mut errors, "to", input.to.as_ref(), today);
    errors.finish()?;
    let filter = ListFilter {
        trip_id: input.trip_id,
        from,
        to,
        item_id: input.item_id,
        state: input.state.map(|s| match s {
            PurchaseStateFilter::InStock => StateFilter::InStock,
            PurchaseStateFilter::SoldOut => StateFilter::SoldOut,
            PurchaseStateFilter::Cancelled => StateFilter::Cancelled,
        }),
    };
    let limit = if input.limit == 0 {
        DEFAULT_PAGE
    } else {
        input.limit
    };
    ctx.db.read(|conn| {
        let recs = repo::list(conn, &filter, i64::from(input.offset), i64::from(limit))?;
        let rows = rows_of(conn, &recs, today)?;
        let sums = repo::totals(conn, &filter)?;
        let released = paisa(sums.cost_released_paisa)?;
        let result = paisa(sums.sales_paisa)?
            .checked_add(paisa(sums.other_shop_paisa)?)
            .and_then(|v| v.checked_sub(released))
            .map_err(money_error)?;
        Ok(PurchasesList {
            rows,
            total_count: u32::try_from(sums.count)
                .map_err(|_| data("the number of purchases does not fit"))?,
            totals: PurchasesTotals {
                total_cost_paisa: paisa(sums.total_cost_paisa)?,
                cost_left_paisa: paisa(sums.cost_left_paisa)?,
                qty_left: sums.qty_left,
                result_paisa: result,
            },
        })
    })
}

/// One purchase with its own lots and write-offs.
pub(super) fn get(ctx: &Context, input: &PurchaseGetInput) -> Result<PurchaseRow, ServiceError> {
    let (_, today) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let rec = repo::get(conn, input.id)?.ok_or_else(|| not_found("id"))?;
        let mut row = rows_of(conn, std::slice::from_ref(&rec), today)?
            .pop()
            .ok_or_else(|| data("a purchase has no row"))?;
        row.lots = repo::lots_of(conn, rec.id)?
            .into_iter()
            .map(|l| {
                Ok(PurchaseLotRef {
                    bikri_id: l.bikri_id,
                    bikri_number: l.bikri_number,
                    lot_id: l.lot_id,
                    lot_no: l.lot_no,
                    entry_date: l.entry_date,
                    qty_sold: l.qty_sold,
                    cost_paisa: paisa(l.cost_paisa)?,
                    sales_paisa: paisa(l.sales_paisa)?,
                })
            })
            .collect::<Result<Vec<_>, ServiceError>>()?;
        row.write_offs = write_off_rows(repo::write_offs_of(conn, &[rec.id])?)
            .remove(&rec.id)
            .unwrap_or_default();
        row.parts = parts_of(conn, &[rec.id])?
            .remove(&rec.id)
            .unwrap_or_default();
        Ok(row)
    })
}

/// The trip goods with stock left, for the باقی مال list (PRD FR-12.25).
pub(crate) fn stock_rows(
    conn: &Connection,
    filter: &ListFilter,
    today: BusinessDate,
) -> Result<Vec<OwnStockRow>, ServiceError> {
    let in_stock = ListFilter {
        state: Some(StateFilter::InStock),
        ..filter.clone()
    };
    let totals = repo::totals(conn, &in_stock)?;
    let recs = repo::list(conn, &in_stock, 0, totals.count.max(1))?;
    let rows = rows_of(conn, &recs, today)?;
    let ids: Vec<i64> = recs.iter().map(|r| r.id).collect();
    let mut offs = write_off_rows(repo::write_offs_of(conn, &ids)?);
    Ok(rows
        .into_iter()
        .map(|row| OwnStockRow {
            purchase_id: row.id,
            number: row.number,
            received_date: row.entry_date,
            trip: row.trip,
            supplier: row.supplier,
            item: row.item,
            packing: row.packing,
            qty_bought: row.qty,
            qty_sold: row.qty_sold,
            qty_written_off: row.qty_written_off,
            balance: row.qty_left,
            total_cost_paisa: row.total_cost_paisa,
            cost_left_paisa: row.cost_left_paisa,
            days_waiting: row.days_waiting,
            write_offs: offs.remove(&row.id).unwrap_or_default(),
        })
        .collect())
}
