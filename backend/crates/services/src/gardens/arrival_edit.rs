//! The edit of a garden arrival (PRD FR-7.70, FR-12.40, FR-12.41; plan 44 T2a part 2).
//!
//! An arrival can be corrected while nothing of it was sold or written off (`goods_moved`), and while
//! its garden is open. It posts nothing, so the edit has no reversal: inside the arrival's edit window
//! the row (item, packing, quantity, weight, note, day to sell) is rewritten, and its "received" stock
//! move is taken back and made again with the new quantity. The arrival keeps its split for another
//! shop (that is changed on its own screen), so a quantity below the نگ for another shop is refused
//! (`ERR_SPLIT_ABOVE_QTY`).
//!
//! The checks are the ones of a new arrival ([`arrivals::check_shared`], [`arrivals::check_line`]).
//!
//! Example: 100 پیٹی edited to 120: the stock move of +100 is reversed and +120 is received.

use mandi_db::arrivals::{self as repo, NewArrival};
use mandi_db::bikri::edit::{self as edit_repo, NewMove};
use mandi_db::edit_windows::{self, Window};
use mandi_db::rusqlite::Connection;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::edit::GardenArrivalEditInput;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Grams;
use mandi_domain::core::validate::{self, FieldErrors};
use serde_json::{Value, json};

use super::arrivals::{check_line, check_shared};
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::edit::entries::{Edited, not_allowed};
use crate::edit::{date_fixed, reason_for};
use crate::error::ServiceError;
use crate::ventures::not_found;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// An arrival as the change list of an edit compares it.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ServiceError::Db`.
pub(crate) fn row_value(conn: &Connection, id: i64) -> Result<Value, ServiceError> {
    let a = repo::get(conn, id)?.ok_or_else(|| not_found("id"))?;
    let names = |r: &mandi_domain::core::accounts::RecordRef| json!({ "name_ur": r.name_ur, "name_en": r.name_en });
    Ok(json!({
        "item": names(&a.item),
        "packing": names(&a.packing),
        "qty": a.qty,
        "weight_grams": a.weight_grams,
        "sell_date": a.sell_date,
        "note": a.note,
        "other_qty": a.other_qty,
        "shop_name": a.shop_name,
    }))
}

/// The edit inside the transaction of the caller: the save and the rehearsal of the review are this
/// one function (Code Standards B5).
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_ARRIVAL_CANCELLED` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_EDIT_DATE_FIXED` on `date`; `ERR_FORMAT_INVALID` on `garden_id`; `ERR_REASON_REQUIRED` on
/// `reason`; `ERR_EDIT_NOT_ALLOWED` on `id` (rule `goods_moved`); `ERR_GARDEN_CLOSED` on `garden_id`;
/// every error of a new arrival at its field. Nothing is written when any of them is returned.
pub(crate) fn edit_in(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &GardenArrivalEditInput,
) -> Result<Edited, ServiceError> {
    let old = repo::get(tx, input.id)?.ok_or_else(|| not_found("id"))?;
    if old.status == "cancelled" {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrArrivalCancelled,
            "id",
        ));
    }
    if old.version != input.version {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrVersionChanged,
            "version",
        ));
    }
    let old_date = BusinessDate::parse_iso(&old.entry_date)
        .ok_or_else(|| data("the day of an arrival is not a date"))?;
    let typed = &input.arrival;
    if validate::date("date", &typed.date, window.1).is_ok_and(|d| d != old_date) {
        return Err(date_fixed("date", old_date));
    }
    if typed.garden_id != Some(old.garden_id) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "garden_id",
        ));
    }
    let reason = reason_for(window.1, old_date, &input.reason)?;
    if old.qty_sold != 0 || old.qty_written_off != 0 {
        return Err(not_allowed("id", "goods_moved"));
    }
    // The same checks as a new arrival; the split for another shop is kept as it is.
    let mut errors = FieldErrors::default();
    let shared = check_shared(
        tx,
        window,
        &mut errors,
        typed.garden_id,
        &typed.date,
        typed.sell_date.as_ref(),
    )?;
    let kept_other = old.other_qty.to_string();
    let line = check_line(
        tx,
        &mut errors,
        "",
        typed.item_id,
        typed.packing_id,
        &typed.qty,
        typed.weight.as_ref(),
        typed.note.as_ref(),
        Some(&kept_other),
        old.shop_name.as_ref(),
    )?;
    errors.finish()?;
    let (Some(shared), Some(line)) = (shared, line) else {
        return Err(ServiceError::reject(ErrorCode::ErrInternal));
    };
    let user = actor.user_id;
    let now = ctx.now_text();
    let old_value = row_value(tx, old.id)?;
    let sell_iso = shared.sell_date.to_iso();

    // Rewrite the arrival inside its window; its "received" move is taken back and made again.
    edit_windows::open(tx, Window::GardenArrival, old.id, user, &now)?;
    repo::update_edited(
        tx,
        old.id,
        &NewArrival {
            venture_id: old.garden_id,
            entry_date: &old.entry_date,
            sell_date: &sell_iso,
            item_id: line.item_id,
            packing_id: line.packing_id,
            qty: line.qty,
            weight_grams: line.weight.map(Grams::get),
            note: line.note.as_deref(),
            other_qty: line.other_qty,
            shop_name: line.shop_name.as_deref(),
        },
        user,
        &now,
    )?;
    if let Some(received) = repo::received_move(tx, old.id)? {
        let neg_qty = old
            .qty
            .checked_neg()
            .ok_or_else(|| data("the quantity does not fit"))?;
        edit_repo::insert_move(
            tx,
            &NewMove {
                lot_id: None,
                purchase_id: None,
                arrival_id: Some(old.id),
                cost_paisa: 0,
                entry_id: None,
                move_date: &old.entry_date,
                kind: "reversal",
                qty: neg_qty,
                reason: reason.as_deref(),
                reverses_move_id: Some(received),
            },
            user,
            &now,
        )?;
    }
    edit_repo::insert_move(
        tx,
        &NewMove {
            lot_id: None,
            purchase_id: None,
            arrival_id: Some(old.id),
            cost_paisa: 0,
            entry_id: None,
            move_date: &old.entry_date,
            kind: "received",
            qty: line.qty,
            reason: None,
            reverses_move_id: None,
        },
        user,
        &now,
    )?;
    edit_windows::close(tx, Window::GardenArrival, old.id)?;

    // The audit log keeps the old and the new values.
    let mut event = Event::new(Some(user), "edit", "garden_arrival")
        .entity_id(old.id)
        .old(json!({ "number": old.number, "row": old_value }))
        .new_value(json!({ "number": old.number, "row": row_value(tx, old.id)? }));
    if let Some(reason) = &reason {
        event = event.reason(reason);
    }
    event = event.sign(crate::day::fix_sign(tx, &old.entry_date)?.as_deref());
    ctx.db.auditor().write(tx, &event.into_entry())?;
    let version = repo::get(tx, old.id)?
        .map(|a| a.version)
        .ok_or_else(|| data("an arrival is missing after its edit"))?;
    Ok(Edited {
        id: old.id,
        number: Some(old.number),
        version,
        entry_id: None,
        reversal_entry_id: None,
    })
}
