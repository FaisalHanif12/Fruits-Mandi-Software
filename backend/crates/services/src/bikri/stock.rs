//! The goods left to sell (باقی مال, PRD 12.8, FR-12.14 to FR-12.16, plan 16): the list, the
//! write-off of spoiled or returned goods, and the undo of a write-off.
//!
//! A write-off is a stock move only: no journal entry for consigned goods (PRD FR-12.16). It needs
//! a reason. It is dated today, which must be an open day, and so must the day of a write-off that
//! is undone (the undo is the opposite move, on the same day as the first, like any reversal).
//! Nothing is deleted: an undo adds an opposite move that names the one it reverses.
//!
//! Example: 5 نگ of کیلا are spoiled: the write-off takes 5 off the lot (received 50, sold 45,
//! written off 5, balance 0) and the lot leaves the list; its undo puts the 5 back.

use mandi_db::bikri::edit::{self as edit_repo, NewMove, StockFilter};
use mandi_domain::core::bikri::{
    StockList, StockListInput, StockRow, WriteOffDone, WriteOffInput, WriteOffRow,
    WriteOffUndoInput, WriteOffUndone,
};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::validate::{self, FieldErrors};
use serde_json::json;

use super::read::optional_date;
use super::shared::data;
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError};

/// The lots with goods left (PRD FR-12.14): date received, بکری number, seller, item, packing,
/// received, sold, written off, balance and days waiting. Only saved (not cancelled) بکری. Read
/// only.
///
/// # Errors
/// The date codes on `date`, `from` and `to`.
pub fn list(ctx: &Context, input: &StockListInput) -> Result<StockList, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let date = optional_date(&mut errors, "date", input.date.as_ref(), today);
    let from = optional_date(&mut errors, "from", input.from.as_ref(), today);
    let to = optional_date(&mut errors, "to", input.to.as_ref(), today);
    errors.finish()?;
    let (from, to) = match date {
        Some(day) => (Some(day.clone()), Some(day)),
        None => (from, to),
    };
    let own_filter = mandi_db::purchases::ListFilter {
        trip_id: None,
        from: from.clone(),
        to: to.clone(),
        item_id: input.item_id,
        state: None,
    };
    let filter = StockFilter {
        from,
        to,
        seller_id: input.seller_id,
        item_id: input.item_id,
        lot_id: input.lot_id,
        include_written_off: input.include_written_off,
    };
    ctx.db.read(|conn| {
        let lots = edit_repo::stock_lots(conn, &filter)?;
        let ids: Vec<i64> = lots.iter().map(|l| l.lot_id).collect();
        let write_offs = edit_repo::write_offs_of_lots(conn, &ids)?;
        let mut rows = Vec::with_capacity(lots.len());
        let mut balance_total = 0_i64;
        for lot in lots {
            let balance = lot
                .qty_received
                .checked_sub(lot.qty_sold)
                .and_then(|v| v.checked_sub(lot.qty_written_off))
                .and_then(|v| v.checked_sub(lot.qty_taken))
                .ok_or_else(|| data("the quantity of a lot does not fit"))?;
            balance_total = balance_total
                .checked_add(balance)
                .ok_or_else(|| data("the quantity of the stock does not fit"))?;
            let days_waiting = BusinessDate::parse_iso(&lot.received_date)
                .map_or(0, |received| today.days_after(received).max(0));
            rows.push(StockRow {
                lot_id: lot.lot_id,
                lot_no: lot.lot_no,
                bikri_id: lot.bikri_id,
                bikri_number: lot.bikri_number,
                received_date: lot.received_date,
                seller: lot.seller,
                pukhta: lot.pukhta.map(|(name_ur, name_en, phone, person_key)| {
                    mandi_domain::core::bikri::SavedPukhta {
                        name_ur,
                        name_en,
                        phone,
                        person_key,
                    }
                }),
                item: lot.item,
                packing: lot.packing,
                qty_received: lot.qty_received,
                qty_sold: lot.qty_sold,
                qty_written_off: lot.qty_written_off,
                qty_taken: lot.qty_taken,
                balance,
                days_waiting,
                write_offs: write_offs
                    .iter()
                    .filter(|w| w.lot_id == lot.lot_id)
                    .map(|w| WriteOffRow {
                        move_id: w.move_id,
                        move_date: w.move_date.clone(),
                        qty: w.qty,
                        reason: w.reason.clone(),
                    })
                    .collect(),
            });
        }
        // The trip goods with stock left, marked اپنا مال with their trip (PRD FR-12.25). A seller
        // filter leaves them out unless it is the اپنا مال account.
        let own_account = mandi_db::accounts::id_by_code(conn, super::shared::OWN_GOODS_CODE)?;
        // One lot asked for: the trip goods are not lots, so none of them is listed.
        let own_rows = if input.lot_id.is_some() {
            Vec::new()
        } else if input.seller_id.is_none() || input.seller_id == own_account {
            crate::purchases::stock_rows(conn, &own_filter, today)?
        } else {
            Vec::new()
        };
        let mut own_balance_total = 0_i64;
        let mut own_cost_left = mandi_domain::core::money::Paisa::ZERO;
        for row in &own_rows {
            own_balance_total = own_balance_total
                .checked_add(row.balance)
                .ok_or_else(|| data("the quantity of the stock does not fit"))?;
            own_cost_left = own_cost_left
                .checked_add(row.cost_left_paisa)
                .map_err(crate::error::money_error)?;
        }
        // One page over the lots first, then the trip goods; the totals above are over every
        // matching row, never the page (PRD FR-19.11, B31).
        let lots = i64::try_from(rows.len()).map_err(|_| data("too many rows"))?;
        let own = i64::try_from(own_rows.len()).map_err(|_| data("too many rows"))?;
        let total_count = lots.checked_add(own).ok_or_else(|| data("too many rows"))?;
        let (rows, own_rows) = crate::page::slice_two(rows, own_rows, input.offset, input.limit);
        Ok(StockList {
            total_count,
            rows,
            balance_total,
            own_rows,
            own_balance_total,
            own_cost_left_paisa: own_cost_left,
        })
    })
}

/// A reason is required (PRD FR-12.16).
fn required_reason(text: &str) -> Result<String, FieldError> {
    let reason = validate::free_text("reason", text)?;
    if reason.is_empty() {
        return Err(FieldError::new(ErrorCode::ErrReasonRequired, "reason"));
    }
    Ok(reason)
}

/// The day of a stock move must be an open day (PRD FR-6.6).
fn check_open(ctx: &Context, date: BusinessDate, field: &str) -> Result<(), ServiceError> {
    let (closed_up_to, _) = day::window(ctx)?;
    match closed_up_to {
        Some(closed) if date <= closed => Err(ServiceError::Rejected(vec![Rejection {
            code: ErrorCode::ErrDateClosed,
            field: Some(field.to_owned()),
            values: std::collections::BTreeMap::from([(
                "closed_up_to".to_owned(),
                ErrorValue::Text(closed.to_iso()),
            )]),
        }])),
        _ => Ok(()),
    }
}

/// Writes off goods left in a lot (PRD FR-12.16).
///
/// # Errors
/// `ERR_LOT_NOT_FOUND` and `ERR_BIKRI_CANCELLED` on `lot_id`; the quantity codes on `qty`;
/// `ERR_WRITE_OFF_EXCEEDS_BALANCE` on `qty` (value `available`); `ERR_REASON_REQUIRED` on
/// `reason`; `ERR_DATE_CLOSED` on `date`. Nothing is written when any of them is returned.
pub fn write_off(
    ctx: &Context,
    actor: &Actor,
    input: &WriteOffInput,
) -> Result<WriteOffDone, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let qty = errors.take(validate::qty("qty", &input.qty));
    let reason = errors.take(required_reason(&input.reason));
    errors.finish()?;
    let (Some(qty), Some(reason)) = (qty, reason) else {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrReasonRequired,
            "reason",
        ));
    };
    check_open(ctx, today, "date")?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let state = edit_repo::lot_state(tx, input.lot_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrLotNotFound, "lot_id"))?;
        if state.bikri_status != "saved" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrBikriCancelled,
                "lot_id",
            ));
        }
        let available = state
            .qty_received
            .checked_sub(state.qty_sold)
            .and_then(|v| v.checked_sub(state.qty_written_off))
            .and_then(|v| v.checked_sub(state.qty_taken))
            .ok_or_else(|| data("the quantity of a lot does not fit"))?;
        if qty.get() > available {
            return Err(ServiceError::Rejected(vec![Rejection {
                code: ErrorCode::ErrWriteOffExceedsBalance,
                field: Some("qty".to_owned()),
                values: std::collections::BTreeMap::from([(
                    "available".to_owned(),
                    ErrorValue::Number(available),
                )]),
            }]));
        }
        let user = actor.user_id;
        let move_id = edit_repo::insert_move(
            tx,
            &NewMove {
                lot_id: Some(state.id),
                purchase_id: None,
                cost_paisa: 0,
                entry_id: None,
                move_date: &today.to_iso(),
                kind: "written_off",
                qty: qty
                    .get()
                    .checked_neg()
                    .ok_or_else(|| data("the quantity does not fit"))?,
                reason: Some(&reason),
                reverses_move_id: None,
            },
            user,
            &now,
        )?;
        edit_repo::add_written_off(tx, state.id, qty.get(), user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "write_off", "bikri_lot")
                .entity_id(state.id)
                .new_value(json!({
                    "bikri_number": state.bikri_number,
                    "lot_no": state.lot_no,
                    "move_id": move_id,
                    "qty": qty.get(),
                    "date": today.to_iso(),
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        let written_off = state
            .qty_written_off
            .checked_add(qty.get())
            .ok_or_else(|| data("the quantity of a lot does not fit"))?;
        Ok(WriteOffDone {
            lot_id: state.id,
            move_id,
            qty: qty.get(),
            qty_written_off: written_off,
            balance: available
                .checked_sub(qty.get())
                .ok_or_else(|| data("the quantity of a lot does not fit"))?,
        })
    })
}

/// Undoes a write-off (PRD FR-12.16): the opposite stock move, on the same day as the write-off,
/// which must still be open.
///
/// # Errors
/// `ERR_NOT_FOUND` on `move_id` (not a write-off, or already undone); `ERR_BIKRI_CANCELLED` on
/// `move_id`; `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `move_id`.
pub fn write_off_undo(
    ctx: &Context,
    actor: &Actor,
    input: &WriteOffUndoInput,
) -> Result<WriteOffUndone, ServiceError> {
    let reason = required_reason(&input.reason)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let write_off = edit_repo::write_off(tx, input.move_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "move_id"))?;
        let state = edit_repo::lot_state(tx, write_off.lot_id)?
            .ok_or_else(|| data("the lot of a write-off is missing"))?;
        if state.bikri_status != "saved" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrBikriCancelled,
                "move_id",
            ));
        }
        let date = BusinessDate::parse_iso(&write_off.move_date)
            .ok_or_else(|| data("the date of a stock move is not a date"))?;
        check_open(ctx, date, "move_id")?;
        let user = actor.user_id;
        let reversal_move_id = edit_repo::insert_move(
            tx,
            &NewMove {
                lot_id: Some(state.id),
                purchase_id: None,
                cost_paisa: 0,
                entry_id: None,
                move_date: &write_off.move_date,
                kind: "reversal",
                qty: write_off.qty,
                reason: Some(&reason),
                reverses_move_id: Some(write_off.move_id),
            },
            user,
            &now,
        )?;
        let put_back = write_off
            .qty
            .checked_neg()
            .ok_or_else(|| data("the quantity of a write-off does not fit"))?;
        edit_repo::add_written_off(tx, state.id, put_back, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "write_off_undo", "bikri_lot")
                .entity_id(state.id)
                .new_value(json!({
                    "bikri_number": state.bikri_number,
                    "lot_no": state.lot_no,
                    "move_id": write_off.move_id,
                    "reversal_move_id": reversal_move_id,
                    "qty": write_off.qty,
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        let written_off = state
            .qty_written_off
            .checked_sub(write_off.qty)
            .ok_or_else(|| data("the quantity of a lot does not fit"))?;
        let balance = state
            .qty_received
            .checked_sub(state.qty_sold)
            .and_then(|v| v.checked_sub(written_off))
            .and_then(|v| v.checked_sub(state.qty_taken))
            .ok_or_else(|| data("the quantity of a lot does not fit"))?;
        Ok(WriteOffUndone {
            lot_id: state.id,
            reversal_move_id,
            qty_written_off: written_off,
            balance,
        })
    })
}
