//! The edit of a sale of trip goods in another shop (PRD FR-7.70, FR-12.32, C21, C25; plan 44 T2a
//! part 2).
//!
//! A sale can be corrected while it is the only thing that moved goods of its purchase (`goods_moved`):
//! the cost of the goods (C21) is worked out from what is left of the purchase, so a later sale or
//! write-off would be worked out against a different cost left. The edit takes the sale back like a
//! cancel (the posting and its stock move are reversed, the purchase gets its نگ and cost back), then
//! checks and works out the sale again with the same function the save uses ([`sale::plan`], Code
//! Standards B5), and writes the new credits, the new posting and the new stock move on the same day
//! inside the entry's edit window.
//!
//! Example (E2E-34): 8,460 received for 15 of 20 نگ at a cost of 10,400, edited to 9,000: the cost of
//! the goods is still 7,800 and the result goes from 660 to 1,200; the trip's partners are credited
//! 3,600 / 2,700 / 2,700 instead of 3,384 / 2,538 / 2,538.

use mandi_db::bikri::edit::{self as edit_repo, NewMove};
use mandi_db::edit_windows::{self, Window};
use mandi_db::ledger as ledger_repo;
use mandi_db::purchases as purchase_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::ventures as repo;
use mandi_domain::cash;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::edit::TripOtherSaleEditInput;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::ledger::{PostingType, SourceType};
use serde_json::{Value, json};

use super::sale::plan;
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash::cash_in_hand;
use crate::context::Context;
use crate::edit::entries::{Edited, not_allowed};
use crate::edit::{date_fixed, reason_for};
use crate::error::{ServiceError, money_error};
use crate::ledger;
use crate::purchases;
use crate::ventures::{self, Settlement, not_found};
use crate::vouchers::stored_cash_change;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// A sale in another shop as the change list of an edit compares it: the shop, the quantity, the
/// money, the cost of the goods and the credits of the trip's partners.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ServiceError::Db`.
pub(crate) fn row_value(conn: &Connection, id: i64) -> Result<Value, ServiceError> {
    let entry = repo::get_entry(conn, id)?
        .filter(|e| e.kind == "other_sale")
        .ok_or_else(|| not_found("id"))?;
    let parts = repo::parts_of(conn, &[id])?;
    let names = |r: &mandi_domain::core::accounts::RecordRef| json!({ "name_ur": r.name_ur, "name_en": r.name_en });
    Ok(json!({
        "shop": entry.shop_name,
        "qty": entry.qty,
        "amount_paisa": entry.amount_paisa,
        "cost_paisa": entry.cost_paisa,
        "note": entry.reason,
        "credits": parts
            .iter()
            .map(|p| json!({
                "partner": names(&p.partner),
                "share_bps": p.share_bps,
                "amount_paisa": p.amount_paisa,
            }))
            .collect::<Vec<_>>(),
    }))
}

/// The edit inside the transaction of the caller: the save and the rehearsal of the review are this
/// one function (Code Standards B5).
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_TRIP_ENTRY_CANCELLED` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_EDIT_DATE_FIXED` on `date`; `ERR_FORMAT_INVALID` on `trip_id` and `purchase_id`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_EDIT_NOT_ALLOWED` on `id` (rule `goods_moved`); every error
/// of a new sale at its field; `ERR_DATE_CLOSED`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. Nothing is
/// written when any of them is returned.
pub(crate) fn edit_in(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &TripOtherSaleEditInput,
) -> Result<Edited, ServiceError> {
    let entry = repo::get_entry(tx, input.id)?
        .filter(|e| e.kind == "other_sale")
        .ok_or_else(|| not_found("id"))?;
    if entry.status == "cancelled" {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrTripEntryCancelled,
            "id",
        ));
    }
    if entry.version != input.version {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrVersionChanged,
            "version",
        ));
    }
    let old_date = BusinessDate::parse_iso(&entry.entry_date)
        .ok_or_else(|| data("the date of a sale is not a date"))?;
    let typed = &input.sale;
    if validate::date("date", &typed.date, window.1).is_ok_and(|d| d != old_date) {
        return Err(date_fixed("date", old_date));
    }
    if typed.trip_id != Some(entry.venture_id) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "trip_id",
        ));
    }
    let purchase_id = entry
        .purchase_id
        .ok_or_else(|| data("a sale in another shop has no purchase"))?;
    if typed.purchase_id != Some(purchase_id) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "purchase_id",
        ));
    }
    let reason = reason_for(window.1, old_date, &input.reason)?;
    // This sale must be the only thing that moved goods of its purchase (C21).
    let sold_here = entry.qty.unwrap_or(0);
    let purchase = purchase_repo::get(tx, purchase_id)?.ok_or_else(|| not_found("purchase_id"))?;
    let moved_total = purchase
        .qty_sold
        .checked_add(purchase.qty_written_off)
        .ok_or_else(|| data("the quantity of a purchase does not fit"))?;
    if moved_total != sold_here {
        return Err(not_allowed("id", "goods_moved"));
    }
    let journal_id = entry
        .journal_entry_id
        .ok_or_else(|| data("a sale in another shop has no journal entry"))?;
    let user = actor.user_id;
    let now = ctx.now_text();
    let old_value = row_value(tx, entry.id)?;
    // What the sale did to cash, and cash before the edit: the question is about the net change.
    let stored = ledger_repo::get_entry(tx, journal_id)?
        .ok_or_else(|| data("the entry of a sale cannot be read"))?;
    let old_cash = stored_cash_change(tx, &stored)?;
    let cash_before = cash_in_hand(tx)?;

    // 1. Take the sale back: the posting and its stock move are reversed, the purchase gets its نگ and
    //    its cost back.
    let reversal = ledger::reverse(ctx, tx, user, journal_id, reason.as_deref().unwrap_or(""))?;
    edit_repo::reverse_moves(tx, journal_id, reversal.entry_id, user, &now)?;
    let qty_back = sold_here
        .checked_neg()
        .ok_or_else(|| data("the quantity does not fit"))?;
    let cost_back = entry
        .cost_paisa
        .checked_neg()
        .ok_or_else(|| data("the cost of a sale does not fit"))?;
    let moved_back = entry
        .moved_qty
        .checked_neg()
        .ok_or_else(|| data("the quantity of a sale does not fit"))?;
    purchase_repo::add_sold_in_other_shop(
        tx,
        purchase_id,
        qty_back,
        moved_back,
        cost_back,
        user,
        &now,
    )?;
    purchases::ensure_consistent(tx, purchase_id)?;
    ventures::refresh_trip_status(tx, entry.venture_id, &entry.entry_date, user, &now)?;

    // 2. The same checking and working out as a new sale.
    let planned = plan(tx, window, typed)?;
    if !planned.errors.is_empty() {
        return Err(ServiceError::from(planned.errors.clone()));
    }
    let ready = planned
        .ready
        .ok_or_else(|| data("a sale without errors has no numbers"))?;
    // The cost of the goods (C21) is kept when the quantity is not changed (the way an unchanged line of
    // a بکری keeps its stored cost, PRD FR-12.20): a trip cost saved after the sale belongs to the goods
    // that were left, not to this sale. A new quantity is worked out again, and is refused when a trip
    // cost was saved after the sale (`carries_cost`), since the cost left is no longer the one it had.
    let cost = if ready.qty.get() == sold_here {
        Paisa::try_from_paisa(entry.cost_paisa).map_err(money_error)?
    } else if purchase_repo::has_cost_part_after(tx, purchase_id, entry.id)? {
        return Err(not_allowed("qty", "carries_cost"));
    } else {
        ready.cost
    };
    // The cash question is about the net change of the edit (PRD FR-6.7).
    let net = ready.amount.checked_sub(old_cash).map_err(money_error)?;
    let check = cash::check(cash_before, net).map_err(money_error)?;
    crate::cash::require_confirmed(&check, input.confirm_cash_below_zero)?;

    // 3. Rewrite the entry, its credits, its posting and its stock move inside the entry's window.
    edit_windows::open(tx, Window::VentureEntry, entry.id, user, &now)?;
    repo::delete_parts(tx, entry.id)?;
    repo::update_entry_edited(
        tx,
        entry.id,
        ready.amount.as_paisa(),
        ready.note.as_deref(),
        Some(ready.qty.get()),
        ready.shop.as_deref(),
        cost.as_paisa(),
        ready.moved,
        None,
        user,
        &now,
    )?;
    ventures::write_credits(tx, user, &now, entry.id, &ready.credits)?;
    let new_entry_id = ventures::post_settlement(
        ctx,
        tx,
        user,
        &now,
        &Settlement {
            venture_entry_id: entry.id,
            date: old_date,
            entry: Some(&ready.entry),
            posting_type: PostingType::VentureReceipt,
            source_type: SourceType::Venture,
            source_id: entry.id,
        },
    )?
    .ok_or_else(|| data("a sale in another shop has no journal entry"))?;
    let qty_out = ready
        .qty
        .get()
        .checked_neg()
        .ok_or_else(|| data("the quantity does not fit"))?;
    edit_repo::insert_move(
        tx,
        &NewMove {
            arrival_id: None,
            lot_id: None,
            purchase_id: Some(purchase_id),
            cost_paisa: cost.as_paisa(),
            entry_id: Some(new_entry_id),
            move_date: &entry.entry_date,
            kind: "sold",
            qty: qty_out,
            reason: None,
            reverses_move_id: None,
        },
        user,
        &now,
    )?;
    purchases::take_sold_other_shop(
        tx,
        purchase_id,
        ready.qty.get(),
        ready.moved,
        cost,
        user,
        &now,
    )?;
    ventures::refresh_trip_status(tx, entry.venture_id, &entry.entry_date, user, &now)?;
    edit_windows::close(tx, Window::VentureEntry, entry.id)?;

    // 4. The audit log keeps the old and the new values.
    let mut event = Event::new(Some(user), "edit", "trip_other_sale")
        .entity_id(entry.id)
        .old(json!({ "number": entry.number, "entry_id": journal_id, "row": old_value }))
        .new_value(json!({
            "number": entry.number,
            "entry_id": new_entry_id,
            "reversal_entry_id": reversal.entry_id,
            "row": row_value(tx, entry.id)?,
            "cash_after_paisa": check.cash_after_paisa.as_paisa(),
            "cash_below_zero_confirmed": check.needs_confirm,
        }));
    if let Some(reason) = &reason {
        event = event.reason(reason);
    }
    ctx.db.auditor().write(tx, &event.into_entry())?;
    let version = repo::get_entry(tx, entry.id)?
        .map(|e| e.version)
        .ok_or_else(|| data("a sale is missing after its edit"))?;
    Ok(Edited {
        id: entry.id,
        number: Some(entry.number),
        version,
        entry_id: Some(new_entry_id),
        reversal_entry_id: Some(reversal.entry_id),
    })
}
