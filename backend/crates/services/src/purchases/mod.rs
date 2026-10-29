//! Goods bought on market trips (PRD 12.10, FR-12.24 to FR-12.28, plan 16b): purchases, their stock
//! and result, and the write-off of spoiled goods.
//!
//! - [`preview`] and [`save`]: ONE checking and working out of a purchase for both (Code Standards
//!   B5), and ONE transaction for the save (Code Standards B3): the purchase, the venture entry with
//!   the partners' parts (C23), the journal entry for the parts paid from the shop's cash (each
//!   partner's 0604 account بنام, روکڑ جمع), the stock move and the audit row.
//! - [`cancel`]: by reversal, only while nothing was sold or written off.
//! - [`write_off`] and [`write_off_undo`]: spoiled trip goods (PRD FR-12.28). The qty and its cost
//!   (C21, `mandi_domain::bikri::own_cost`) come off the purchase and nothing is posted.
//! - `reads`: the list, one purchase with its lots and write-offs.
//!
//! The sale of trip goods is a lot of a بکری (`services::bikri`) or a sale in another shop
//! (`services::trips`); both move the purchase's `qty_sold` and `cost_released` with [`take_sold`]
//! and [`give_back_entry`].
//!
//! Example (E2E-34): 20 پیٹی at 500 and freight 400, paid from the shop by three partners at
//! 40 / 30 / 30: each partner's 0604 account بنام 4,160 / 3,120 / 3,120, روکڑ جمع 10,400. Selling 15
//! releases cost 7,800; the 5 left cost 2,600; writing them off posts nothing.

mod plan;
mod reads;

pub(crate) use reads::{rows_of_trip, stock_rows};

use mandi_db::bikri::edit::{self as edit_repo, NewMove};
use mandi_db::ledger as ledger_repo;
use mandi_db::purchases::{self as repo, NewPurchase};
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as venture_repo, NewEntry};
use mandi_domain::bikri::own_cost;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::purchases::{
    PurchaseCancelInput, PurchaseCancelled, PurchaseGetInput, PurchaseInput, PurchasePreview,
    PurchaseRow, PurchaseSaved, PurchaseWriteOffDone, PurchaseWriteOffInput,
    PurchaseWriteOffUndoInput, PurchaseWriteOffUndone, PurchasesList, PurchasesListInput,
};
use mandi_domain::core::validate;
use mandi_domain::ledger::{PostingType, SourceType};
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::cash;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};
use crate::ledger;
use crate::ventures::{self, Settlement, TripChange, not_found, require_open_day, required_reason};
use crate::vouchers::stored_cash_change;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// Works out a typed purchase for the live screen (PRD FR-12.24). Nothing is saved.
///
/// Example (E2E-34): 20 at 500 with freight 400 gives value 10,000, paid now 10,400, average cost
/// 520 for each نگ, and the parts 4,160 / 3,120 / 3,120 at 40 / 30 / 30.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix. The mistakes in the
/// purchase come back inside the preview (`errors`).
pub fn preview(ctx: &Context, input: &PurchaseInput) -> Result<PurchasePreview, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db
        .read(|conn| Ok(plan::plan(conn, window, input)?.preview()))
}

/// Saves a purchase in one transaction (PRD FR-12.24). See the module text.
///
/// # Errors
/// Every error of the preview, each at its field and all together, with nothing written;
/// `ERR_TRIP_CLOSED` on `trip_id`; `ERR_PURCHASE_BEFORE_TRIP`, `ERR_DATE_CLOSED` and
/// `ERR_DATE_IN_FUTURE` on `date`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`) when
/// the shop parts take cash below 0 and `confirm_cash_below_zero` is false.
pub fn save(
    ctx: &Context,
    actor: &Actor,
    input: &PurchaseInput,
) -> Result<PurchaseSaved, ServiceError> {
    let window = day::window(ctx)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let planned = plan::plan(tx, window, input)?;
        if !planned.errors.is_empty() {
            return Err(plan::rejected(planned.errors.clone()));
        }
        cash::require_confirmed(&planned.cash, input.confirm_cash_below_zero)?;
        let preview = planned.preview();
        let ready = planned
            .ready
            .ok_or_else(|| data("a purchase without errors has no numbers"))?;
        let user = actor.user_id;
        let date_iso = ready.date.to_iso();
        let number = repo::next_number(tx)?;
        let id = repo::insert(
            tx,
            &NewPurchase {
                venture_id: ready.venture.rec.id,
                entry_date: &date_iso,
                sell_date: &ready.sell_date.to_iso(),
                supplier_text: ready.supplier.as_deref(),
                item_id: ready.item_id,
                packing_id: ready.packing_id,
                qty: ready.qty.get(),
                weight_grams: ready.weight.map(mandi_domain::core::money::Grams::get),
                rate_paisa: ready.rate.as_paisa(),
                value_paisa: ready.value.as_paisa(),
                freight_paisa: ready.freight.as_paisa(),
                labour_paisa: ready.labour.as_paisa(),
                other_cost_paisa: ready.other_cost.as_paisa(),
                paid_now_paisa: ready.paid_now.as_paisa(),
                note: ready.note.as_deref(),
            },
            number,
            user,
            &now,
        )?;
        // The trip's entry for it holds the partners' parts (C23).
        let entry_number = venture_repo::next_entry_number(tx, "purchase")?;
        let venture_entry = venture_repo::insert_entry(
            tx,
            &NewEntry {
                venture_id: ready.venture.rec.id,
                kind: "purchase",
                entry_date: &date_iso,
                amount_paisa: ready.paid_now.as_paisa(),
                reason: None,
                purchase_id: Some(id),
                qty: None,
                shop_name: None,
                cost_paisa: 0,
            },
            entry_number,
            user,
            &now,
        )?;
        ventures::write_parts(tx, user, &now, venture_entry, &ready.payment.parts)?;
        // Only the parts paid from the shop's cash post (FR-12.30); none at all: no entry.
        let entry_id = ventures::post_settlement(
            ctx,
            tx,
            user,
            &now,
            &Settlement {
                venture_entry_id: venture_entry,
                date: ready.date,
                entry: ready.payment.entry.as_ref(),
                posting_type: PostingType::Purchase,
                source_type: SourceType::Purchase,
                source_id: id,
            },
        )?;
        if let Some(entry_id) = entry_id {
            repo::set_entry(tx, id, entry_id, user, &now)?;
        }
        edit_repo::insert_move(
            tx,
            &NewMove {
                arrival_id: None,
                lot_id: None,
                purchase_id: Some(id),
                cost_paisa: 0,
                entry_id,
                move_date: &date_iso,
                kind: "received",
                qty: ready.qty.get(),
                reason: None,
                reverses_move_id: None,
            },
            user,
            &now,
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "create", "purchase")
                .entity_id(id)
                .new_value(json!({
                    "number": number,
                    "trip_id": ready.venture.rec.id,
                    "entry_date": date_iso,
                    "entry_id": entry_id,
                    "item_id": ready.item_id,
                    "qty": ready.qty.get(),
                    "value_paisa": ready.value.as_paisa(),
                    "paid_now_paisa": ready.paid_now.as_paisa(),
                    "parts": ready
                        .payment
                        .parts
                        .iter()
                        .map(|p| json!({
                            "partner_id": p.partner_id.get(),
                            "source": p.source.as_str(),
                            "amount_paisa": p.amount.as_paisa(),
                        }))
                        .collect::<Vec<_>>(),
                    "cash_after_paisa": planned.cash.cash_after_paisa.as_paisa(),
                    "cash_below_zero_confirmed": planned.cash.needs_confirm,
                }))
                .into_entry(),
        )?;
        let version = repo::get(tx, id)?
            .map(|p| p.version)
            .ok_or_else(|| data("a purchase is missing after its save"))?;
        Ok(PurchaseSaved {
            id,
            number,
            version,
            entry_id,
            total_cost: ready.paid_now,
            preview,
        })
    })
}

/// After every change of a purchase's running figures: sold plus written off is never more than
/// bought, the cost released is never more than the cost, and when nothing is left all of the cost
/// has been released (PRD C21, Code Standards 9.3). The table checks the first two; the third is
/// checked here, and a purchase that breaks it rolls the whole save back.
pub(crate) fn ensure_consistent(conn: &Connection, id: i64) -> Result<(), ServiceError> {
    let p = repo::get(conn, id)?.ok_or_else(|| not_found("purchase_id"))?;
    let left = p
        .qty
        .checked_sub(p.qty_sold)
        .and_then(|v| v.checked_sub(p.qty_written_off))
        .ok_or_else(|| data("the quantity of a purchase does not fit"))?;
    if p.status == "saved" && left == 0 && p.cost_released_paisa != p.total_cost_paisa {
        tracing::error!(
            purchase = id,
            released = p.cost_released_paisa,
            total = p.total_cost_paisa,
            "a purchase has no stock left but not all of its cost was released"
        );
        return Err(data("a purchase with no stock left has cost left"));
    }
    Ok(())
}

/// The purchase gives up what an own lot sold: `qty` نگ and `cost` (C21, PRD FR-12.26).
///
/// # Errors
/// `ServiceError::Db` (the checks of `purchases`), `ServiceError::Data` if the figures break the
/// rule of [`ensure_consistent`].
pub(crate) fn take_sold(
    tx: &Connection,
    purchase_id: i64,
    qty: i64,
    cost: Paisa,
    user_id: i64,
    now: &str,
) -> Result<(), ServiceError> {
    repo::add_sold(tx, purchase_id, qty, cost.as_paisa(), user_id, now)?;
    ensure_consistent(tx, purchase_id)
}

/// Gives back to the purchases exactly what the lines in force of a posting took, before the
/// posting is reversed or replaced: the نگ and the stored cost of each line (PRD FR-12.20,
/// FR-12.26).
///
/// # Errors
/// As [`take_sold`].
pub(crate) fn give_back_entry(
    tx: &Connection,
    entry_id: i64,
    user_id: i64,
    now: &str,
) -> Result<(), ServiceError> {
    for taken in edit_repo::own_release_of_entry(tx, entry_id)? {
        let qty = taken
            .qty
            .checked_neg()
            .ok_or_else(|| data("the quantity does not fit"))?;
        let cost = taken
            .cost_paisa
            .checked_neg()
            .ok_or_else(|| data("the cost does not fit"))?;
        repo::add_sold(tx, taken.purchase_id, qty, cost, user_id, now)?;
        ensure_consistent(tx, taken.purchase_id)?;
    }
    Ok(())
}

/// Opens or closes the trips whose goods a posting sold or gave back (PRD FR-12.33): a trip closes by
/// itself when every purchase has nothing left, and opens again when a sale of it is reversed. Call
/// it after the posting and its give-backs are written. `date_iso` is the day of the action.
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn refresh_trips_of_entry(
    tx: &Connection,
    entry_id: i64,
    date_iso: &str,
    user_id: i64,
    now: &str,
) -> Result<(), ServiceError> {
    for trip_id in repo::trips_of_entry(tx, entry_id)? {
        crate::ventures::refresh_trip_status(tx, trip_id, date_iso, user_id, now)?;
    }
    Ok(())
}

/// Cancels a purchase by reversal (PRD FR-12.24): only while nothing of it was sold or written off
/// and it carries no part of a trip cost. Its journal entry (if it had one) is reversed, its stock
/// move is reversed, and the trip is checked again (it may close by itself, PRD FR-12.33).
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_PURCHASE_CANCELLED` on `id`; `ERR_PURCHASE_HAS_SALES` on `id` (values
/// `qty_sold`, `qty_written_off`); `ERR_PURCHASE_CARRIES_TRIP_COST` on `id`; `ERR_VERSION_CHANGED` on
/// `version`; `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `id`;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. Nothing is written when any of them is returned.
pub fn cancel(
    ctx: &Context,
    actor: &Actor,
    input: &PurchaseCancelInput,
) -> Result<PurchaseCancelled, ServiceError> {
    let reason = required_reason(&input.reason)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let purchase = repo::get(tx, input.id)?.ok_or_else(|| not_found("id"))?;
        if purchase.status == "cancelled" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrPurchaseCancelled,
                "id",
            ));
        }
        if purchase.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        if purchase.qty_sold != 0 || purchase.qty_written_off != 0 {
            return Err(ServiceError::Rejected(vec![Rejection {
                code: ErrorCode::ErrPurchaseHasSales,
                field: Some("id".to_owned()),
                values: std::collections::BTreeMap::from([
                    ("qty_sold".to_owned(), ErrorValue::Number(purchase.qty_sold)),
                    (
                        "qty_written_off".to_owned(),
                        ErrorValue::Number(purchase.qty_written_off),
                    ),
                ]),
            }]));
        }
        if repo::carries_trip_cost(tx, purchase.id)? {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrPurchaseCarriesTripCost,
                "id",
            ));
        }
        let date = BusinessDate::parse_iso(&purchase.entry_date)
            .ok_or_else(|| data("the date of a purchase is not a date"))?;
        require_open_day(ctx, date, "id")?;
        let user = actor.user_id;
        // A cancel gives back what the shop parts took from cash (PRD FR-6.7).
        let reversal_entry_id = match purchase.entry_id {
            Some(entry_id) => {
                let stored = ledger_repo::get_entry(tx, entry_id)?
                    .ok_or_else(|| data("the entry of a purchase cannot be read"))?;
                let change = stored_cash_change(tx, &stored)?
                    .checked_neg()
                    .map_err(money_error)?;
                cash::require_confirmed(
                    &cash::cash_after_entry(tx, change)?,
                    input.confirm_cash_below_zero,
                )?;
                Some(ledger::reverse(ctx, tx, user, entry_id, &reason)?.entry_id)
            }
            None => None,
        };
        // The stock move of the purchase is reversed whether or not it had a journal entry.
        let (move_id, _) = repo::received_move(tx, purchase.id)?
            .ok_or_else(|| data("a purchase has no stock move"))?;
        let neg_qty = purchase
            .qty
            .checked_neg()
            .ok_or_else(|| data("the quantity does not fit"))?;
        edit_repo::insert_move(
            tx,
            &NewMove {
                arrival_id: None,
                lot_id: None,
                purchase_id: Some(purchase.id),
                cost_paisa: 0,
                entry_id: reversal_entry_id,
                move_date: &purchase.entry_date,
                kind: "reversal",
                qty: neg_qty,
                reason: Some(&reason),
                reverses_move_id: Some(move_id),
            },
            user,
            &now,
        )?;
        repo::mark_cancelled(tx, purchase.id, &reason, user, &now)?;
        if let Some(entry) = venture_repo::entry_of_purchase(tx, purchase.id)? {
            venture_repo::mark_entry_cancelled(tx, entry.id, &reason, user, &now)?;
        }
        // Cancelling a purchase can leave every other purchase with nothing left (PRD FR-12.33).
        ventures::refresh_trip_status(tx, purchase.trip_id, &purchase.entry_date, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "cancel", "purchase")
                .entity_id(purchase.id)
                .old(json!({
                    "number": purchase.number,
                    "status": "saved",
                    "paid_now_paisa": purchase.paid_now_paisa,
                }))
                .new_value(json!({
                    "status": "cancelled",
                    "reversal_entry_id": reversal_entry_id,
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        Ok(PurchaseCancelled {
            id: purchase.id,
            number: purchase.number,
            reversal_entry_id,
        })
    })
}

/// One page of purchases with the totals over the whole filter. Read only.
///
/// # Errors
/// The date codes on `from` and `to`.
pub fn list(ctx: &Context, input: &PurchasesListInput) -> Result<PurchasesList, ServiceError> {
    reads::list(ctx, input)
}

/// One purchase with its parts, own lots and write-offs. Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn get(ctx: &Context, input: &PurchaseGetInput) -> Result<PurchaseRow, ServiceError> {
    reads::get(ctx, input)
}

/// Writes off spoiled trip goods (PRD FR-12.28): the qty and its cost (C21) come off the purchase.
/// Nothing is posted. The cost is stored on the stock move, so the undo gives back exactly this.
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_PURCHASE_CANCELLED` on `purchase_id`; the quantity codes on `qty`;
/// `ERR_WRITE_OFF_EXCEEDS_BALANCE` on `qty` (value `available`); `ERR_REASON_REQUIRED` on `reason`;
/// `ERR_DATE_CLOSED` on `date`. Nothing is written when any of them is returned.
pub fn write_off(
    ctx: &Context,
    actor: &Actor,
    input: &PurchaseWriteOffInput,
) -> Result<PurchaseWriteOffDone, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = validate::FieldErrors::default();
    let qty = errors.take(validate::qty("qty", &input.qty));
    let reason = errors.take(validate::free_text("reason", &input.reason));
    if reason.as_ref().is_some_and(String::is_empty) {
        errors.push(FieldError::new(ErrorCode::ErrReasonRequired, "reason"));
    }
    errors.finish()?;
    let (Some(qty), Some(reason)) = (qty, reason) else {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrReasonRequired,
            "reason",
        ));
    };
    require_open_day(ctx, today, "date")?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let purchase = repo::get(tx, input.purchase_id)?.ok_or_else(|| not_found("purchase_id"))?;
        if purchase.status == "cancelled" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrPurchaseCancelled,
                "purchase_id",
            ));
        }
        let qty_left = purchase
            .qty
            .checked_sub(purchase.qty_sold)
            .and_then(|v| v.checked_sub(purchase.qty_written_off))
            .ok_or_else(|| data("the quantity of a purchase does not fit"))?;
        if qty.get() > qty_left {
            return Err(ServiceError::Rejected(vec![Rejection {
                code: ErrorCode::ErrWriteOffExceedsBalance,
                field: Some("qty".to_owned()),
                values: std::collections::BTreeMap::from([(
                    "available".to_owned(),
                    ErrorValue::Number(qty_left),
                )]),
            }]));
        }
        let cost_left = Paisa::try_from_paisa(
            purchase
                .total_cost_paisa
                .checked_sub(purchase.cost_released_paisa)
                .ok_or_else(|| data("the cost of a purchase does not fit"))?,
        )
        .map_err(money_error)?;
        let (cost, working) = own_cost(cost_left, qty_left, qty.get())
            .map_err(|e| ServiceError::Data(e.to_string()))?;
        let user = actor.user_id;
        let date_iso = today.to_iso();
        let neg_qty = qty
            .get()
            .checked_neg()
            .ok_or_else(|| data("the quantity does not fit"))?;
        let move_id = edit_repo::insert_move(
            tx,
            &NewMove {
                arrival_id: None,
                lot_id: None,
                purchase_id: Some(purchase.id),
                cost_paisa: cost.as_paisa(),
                entry_id: None,
                move_date: &date_iso,
                kind: "written_off",
                qty: neg_qty,
                reason: Some(&reason),
                reverses_move_id: None,
            },
            user,
            &now,
        )?;
        repo::add_written_off(tx, purchase.id, qty.get(), cost.as_paisa(), user, &now)?;
        ensure_consistent(tx, purchase.id)?;
        let change = ventures::refresh_trip_status(tx, purchase.trip_id, &date_iso, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "write_off", "purchase")
                .entity_id(purchase.id)
                .new_value(json!({
                    "number": purchase.number,
                    "move_id": move_id,
                    "qty": qty.get(),
                    "cost_paisa": cost.as_paisa(),
                    "date": date_iso,
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        let left_after = qty_left
            .checked_sub(qty.get())
            .ok_or_else(|| data("the quantity does not fit"))?;
        Ok(PurchaseWriteOffDone {
            purchase_id: purchase.id,
            move_id,
            qty: qty.get(),
            cost_paisa: cost,
            working,
            qty_written_off: purchase
                .qty_written_off
                .checked_add(qty.get())
                .ok_or_else(|| data("the quantity does not fit"))?,
            qty_left: left_after,
            cost_left_paisa: cost_left.checked_sub(cost).map_err(money_error)?,
            trip_closed: change == TripChange::Closed,
        })
    })
}

/// Undoes a write-off of trip goods (PRD FR-12.28): the qty and the stored cost go back to the
/// purchase, on a day that is still open. The trip opens again if the write-off had closed it.
///
/// # Errors
/// `ERR_NOT_FOUND` on `move_id` (not a write-off of a purchase, or already undone);
/// `ERR_PURCHASE_CANCELLED` on `move_id`; `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on
/// `move_id`.
pub fn write_off_undo(
    ctx: &Context,
    actor: &Actor,
    input: &PurchaseWriteOffUndoInput,
) -> Result<PurchaseWriteOffUndone, ServiceError> {
    let reason = required_reason(&input.reason)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let off = repo::write_off(tx, input.move_id)?.ok_or_else(|| not_found("move_id"))?;
        let purchase = repo::get(tx, off.purchase_id)?
            .ok_or_else(|| data("the purchase of a write-off is missing"))?;
        if purchase.status == "cancelled" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrPurchaseCancelled,
                "move_id",
            ));
        }
        let date = BusinessDate::parse_iso(&off.move_date)
            .ok_or_else(|| data("the date of a stock move is not a date"))?;
        require_open_day(ctx, date, "move_id")?;
        let user = actor.user_id;
        let back_cost = off
            .cost_paisa
            .checked_neg()
            .ok_or_else(|| data("the cost does not fit"))?;
        let reversal_move_id = edit_repo::insert_move(
            tx,
            &NewMove {
                arrival_id: None,
                lot_id: None,
                purchase_id: Some(purchase.id),
                cost_paisa: back_cost,
                entry_id: None,
                move_date: &off.move_date,
                kind: "reversal",
                qty: off.qty,
                reason: Some(&reason),
                reverses_move_id: Some(off.move_id),
            },
            user,
            &now,
        )?;
        let back_qty = off
            .qty
            .checked_neg()
            .ok_or_else(|| data("the quantity does not fit"))?;
        repo::add_written_off(tx, purchase.id, back_qty, back_cost, user, &now)?;
        ensure_consistent(tx, purchase.id)?;
        let change =
            ventures::refresh_trip_status(tx, purchase.trip_id, &off.move_date, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "write_off_undo", "purchase")
                .entity_id(purchase.id)
                .new_value(json!({
                    "number": purchase.number,
                    "move_id": off.move_id,
                    "reversal_move_id": reversal_move_id,
                    "qty": off.qty,
                    "cost_paisa": off.cost_paisa,
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        let after = repo::get(tx, purchase.id)?
            .ok_or_else(|| data("the purchase is missing after the undo"))?;
        let qty_left = after
            .qty
            .checked_sub(after.qty_sold)
            .and_then(|v| v.checked_sub(after.qty_written_off))
            .ok_or_else(|| data("the quantity of a purchase does not fit"))?;
        Ok(PurchaseWriteOffUndone {
            purchase_id: purchase.id,
            reversal_move_id,
            qty_written_off: after.qty_written_off,
            qty_left,
            cost_left_paisa: Paisa::try_from_paisa(
                after
                    .total_cost_paisa
                    .checked_sub(after.cost_released_paisa)
                    .ok_or_else(|| data("the cost of a purchase does not fit"))?,
            )
            .map_err(money_error)?,
            trip_reopened: change == TripChange::Reopened,
        })
    })
}

/// Changes the date to sell of a purchase with goods left (PRD FR-12.39). No money effect: a new
/// بکری from that day opens with a ready lot for the goods left.
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_PURCHASE_CANCELLED`, `ERR_NOTHING_TO_SELL` on `id`; `ERR_VERSION_CHANGED` on
/// `version`; `ERR_SELL_DATE_BEFORE_ENTRY` (value `entry_date`) and the date codes on `sell_date`.
pub fn set_sell_date(
    ctx: &Context,
    actor: &Actor,
    input: &mandi_domain::core::arrivals::SellDateInput,
) -> Result<mandi_domain::core::arrivals::SellDateSet, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let purchase = repo::get(tx, input.id)?.ok_or_else(|| not_found("id"))?;
        if purchase.status == "cancelled" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrPurchaseCancelled,
                "id",
            ));
        }
        if purchase.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        let left = purchase
            .qty
            .checked_sub(purchase.qty_sold)
            .and_then(|v| v.checked_sub(purchase.qty_written_off))
            .ok_or_else(|| data("the quantity of a purchase does not fit"))?;
        if left == 0 {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNothingToSell,
                "id",
            ));
        }
        let entry = BusinessDate::parse_iso(&purchase.entry_date)
            .ok_or_else(|| data("the day of a purchase is not a date"))?;
        let mut errors = validate::FieldErrors::default();
        let sell = ventures::sell_date_of(&mut errors, Some(&input.sell_date), Some(entry), today);
        errors.finish()?;
        let sell = sell.ok_or_else(|| data("a sell date without errors is missing"))?;
        let sell_iso = sell.to_iso();
        let user = actor.user_id;
        repo::set_sell_date(tx, purchase.id, &sell_iso, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "set_sell_date", "purchase")
                .entity_id(purchase.id)
                .old(json!({ "sell_date": purchase.sell_date }))
                .new_value(json!({ "number": purchase.number, "sell_date": sell_iso }))
                .into_entry(),
        )?;
        let after = repo::get(tx, purchase.id)?
            .ok_or_else(|| data("the purchase is missing after the change"))?;
        Ok(mandi_domain::core::arrivals::SellDateSet {
            id: after.id,
            version: after.version,
            sell_date: after.sell_date,
        })
    })
}
