//! The edit of a trip purchase (PRD FR-7.70, FR-12.24; plan 44 T2a part 2).
//!
//! A purchase can be corrected while nothing of it was sold or written off (`goods_moved`). The edit is
//! the cancel and the save of the same purchase in one transaction, with the purchase keeping its
//! number, its trip, its day and its place in the history:
//!
//! 1. the posting of the shop's parts is reversed (each partner's 0604 account gets his money back),
//! 2. the purchase is checked and worked out again by [`plan::plan`], the same function the save and
//!    the preview use (Code Standards B5), so the new posting is built exactly like a new purchase's,
//! 3. inside the edit window of the purchase and of its payment entry the purchase row, the partners'
//!    parts and the stock move are rewritten, and the new posting is made on the same day,
//! 4. the audit log keeps the old and the new values and the reason.
//!
//! A purchase that carries a part of a trip cost keeps its quantity: the cost was spread by the نگ
//! bought (C24), so a new quantity would need the cost spread again (`carries_cost`).
//!
//! Example (E2E-34): 20 پیٹی at 500 and freight 400 (10,400, parts 4,160 / 3,120 / 3,120) edited to
//! rate 520: value 10,400, paid now 10,800, parts 4,320 / 3,240 / 3,240; each shop part of the old
//! posting goes back to the partner's account and the new one comes out again.

use mandi_db::bikri::edit::{self as edit_repo, NewMove};
use mandi_db::edit_windows::{self, Window};
use mandi_db::ledger as ledger_repo;
use mandi_db::purchases::{self as repo, NewPurchase};
use mandi_db::rusqlite::Connection;
use mandi_db::ventures as venture_repo;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::edit::PurchaseEditInput;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Grams;
use mandi_domain::core::validate;
use serde_json::{Value, json};

use super::plan;
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash::{self, cash_in_hand};
use crate::context::Context;
use crate::edit::entries::{Edited, net_cash_check, not_allowed};
use crate::edit::{date_fixed, reason_for};
use crate::error::ServiceError;
use crate::ledger;
use crate::ventures::{self, Settlement, not_found};
use crate::vouchers::stored_cash_change;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// A purchase as the change list of an edit compares it: what was bought and paid, its split, and
/// the partners' parts.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ServiceError::Db`.
pub(crate) fn row_value(conn: &Connection, id: i64) -> Result<Value, ServiceError> {
    let p = repo::get(conn, id)?.ok_or_else(|| not_found("id"))?;
    let parts = match venture_repo::entry_of_purchase(conn, id)? {
        Some(entry) => venture_repo::parts_of(conn, &[entry.id])?,
        None => Vec::new(),
    };
    let names = |r: &mandi_domain::core::accounts::RecordRef| json!({ "name_ur": r.name_ur, "name_en": r.name_en });
    Ok(json!({
        "supplier": p.supplier_text,
        "item": names(&p.item),
        "packing": names(&p.packing),
        "qty": p.qty,
        "weight_grams": p.weight_grams,
        "rate_paisa": p.rate_paisa,
        "value_paisa": p.value_paisa,
        "freight_paisa": p.freight_paisa,
        "labour_paisa": p.labour_paisa,
        "other_cost_paisa": p.other_cost_paisa,
        "paid_now_paisa": p.paid_now_paisa,
        "trip_cost_paisa": p.trip_cost_paisa,
        "total_cost_paisa": p.total_cost_paisa,
        "other_qty": p.other_qty,
        "shop_name": p.shop_name,
        "sell_date": p.sell_date,
        "note": p.note,
        "parts": parts
            .iter()
            .map(|part| json!({
                "partner": names(&part.partner),
                "share_bps": part.share_bps,
                "source": part.source,
                "amount_paisa": part.amount_paisa,
            }))
            .collect::<Vec<_>>(),
    }))
}

/// The edit inside the transaction of the caller: the save and the rehearsal of the review are this
/// one function (Code Standards B5).
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_PURCHASE_CANCELLED` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_EDIT_DATE_FIXED` on `date`; `ERR_FORMAT_INVALID` on `trip_id`; `ERR_REASON_REQUIRED` on
/// `reason`; `ERR_EDIT_NOT_ALLOWED` on `id` (rule `goods_moved`) or on `qty` (rule `carries_cost`);
/// every error of a new purchase at its field; `ERR_DATE_CLOSED`;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. Nothing is written when any of them is returned.
pub(crate) fn edit_in(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &PurchaseEditInput,
) -> Result<Edited, ServiceError> {
    let old = repo::get(tx, input.id)?.ok_or_else(|| not_found("id"))?;
    if old.status == "cancelled" {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrPurchaseCancelled,
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
        .ok_or_else(|| data("the date of a purchase is not a date"))?;
    // The date and the trip never change (PRD FR-7.70). The date is named before a missing reason.
    let typed = &input.purchase;
    if validate::date("date", &typed.date, window.1).is_ok_and(|d| d != old_date) {
        return Err(date_fixed("date", old_date));
    }
    if typed.trip_id != Some(old.trip_id) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "trip_id",
        ));
    }
    let reason = reason_for(window.1, old_date, &input.reason)?;
    if old.qty_sold != 0 || old.qty_written_off != 0 {
        return Err(not_allowed("id", "goods_moved"));
    }
    let carries_cost = repo::has_cost_part(tx, old.id)?;
    let user = actor.user_id;
    let now = ctx.now_text();

    // The cash question is about the net change of the edit (PRD FR-6.7).
    let cash_before = cash_in_hand(tx)?;
    let old_cash = match old.entry_id {
        Some(entry_id) => {
            let stored = ledger_repo::get_entry(tx, entry_id)?
                .ok_or_else(|| data("the entry of a purchase cannot be read"))?;
            stored_cash_change(tx, &stored)?
        }
        None => mandi_domain::core::money::Paisa::ZERO,
    };
    // 1. The old posting goes back (ERR_DATE_CLOSED if its day is closed). The cash it took from the
    //    shop is back before the new parts are posted.
    let reversal_entry_id = match old.entry_id {
        Some(entry_id) => Some(
            ledger::reverse(ctx, tx, user, entry_id, reason.as_deref().unwrap_or(""))?.entry_id,
        ),
        None => None,
    };
    // 2. The same checking and working out as a new purchase.
    let planned = plan::plan(tx, window, typed)?;
    if !planned.errors.is_empty() {
        return Err(plan::rejected(planned.errors));
    }
    let cash_check = net_cash_check(cash_before, old_cash, planned.cash.change_paisa)?;
    cash::require_confirmed(&cash_check, typed.confirm_cash_below_zero)?;
    let ready = planned
        .ready
        .ok_or_else(|| data("a purchase without errors has no numbers"))?;
    if carries_cost && ready.qty.get() != old.qty {
        return Err(not_allowed("qty", "carries_cost"));
    }
    let old_value = row_value(tx, old.id)?;

    // 3. Rewrite the purchase, its payment entry and its parts inside the windows.
    let entry = venture_repo::entry_of_purchase(tx, old.id)?
        .ok_or_else(|| data("a purchase has no payment entry"))?;
    edit_windows::open(tx, Window::Purchase, old.id, user, &now)?;
    edit_windows::open(tx, Window::VentureEntry, entry.id, user, &now)?;
    let retired = venture_repo::retire_parts(tx, entry.id, user, &now)?;
    venture_repo::update_entry_edited(
        tx,
        entry.id,
        ready.paid_now.as_paisa(),
        None,
        None,
        None,
        0,
        0,
        None,
        user,
        &now,
    )?;
    ventures::write_parts(tx, user, &now, entry.id, &ready.payment.parts)?;
    // Only the parts paid from the shop's cash post (FR-12.30); none at all: no entry.
    let new_entry_id = ventures::post_settlement(
        ctx,
        tx,
        user,
        &now,
        &Settlement {
            venture_entry_id: entry.id,
            date: old_date,
            entry: ready.payment.entry.as_ref(),
            posting_type: mandi_domain::ledger::PostingType::Purchase,
            source_type: mandi_domain::ledger::SourceType::Purchase,
            source_id: old.id,
        },
    )?;
    let sell_iso = ready.sell_date.to_iso();
    repo::update_edited(
        tx,
        old.id,
        &NewPurchase {
            venture_id: old.trip_id,
            entry_date: &old.entry_date,
            sell_date: &sell_iso,
            supplier_text: ready.supplier.as_deref(),
            item_id: ready.item_id,
            packing_id: ready.packing_id,
            qty: ready.qty.get(),
            weight_grams: ready.weight.map(Grams::get),
            rate_paisa: ready.rate.as_paisa(),
            value_paisa: ready.value.as_paisa(),
            freight_paisa: ready.freight.as_paisa(),
            labour_paisa: ready.labour.as_paisa(),
            other_cost_paisa: ready.other_cost.as_paisa(),
            paid_now_paisa: ready.paid_now.as_paisa(),
            note: ready.note.as_deref(),
            other_qty: ready.other_qty,
            shop_name: ready.shop_name.as_deref(),
        },
        new_entry_id,
        user,
        &now,
    )?;
    // The stock move: the old "received" is taken back and the new quantity is received.
    let (move_id, _) =
        repo::received_move(tx, old.id)?.ok_or_else(|| data("a purchase has no stock move"))?;
    let neg_qty = old
        .qty
        .checked_neg()
        .ok_or_else(|| data("the quantity does not fit"))?;
    edit_repo::insert_move(
        tx,
        &NewMove {
            arrival_id: None,
            lot_id: None,
            purchase_id: Some(old.id),
            cost_paisa: 0,
            entry_id: reversal_entry_id,
            move_date: &old.entry_date,
            kind: "reversal",
            qty: neg_qty,
            reason: reason.as_deref(),
            reverses_move_id: Some(move_id),
        },
        user,
        &now,
    )?;
    edit_repo::insert_move(
        tx,
        &NewMove {
            arrival_id: None,
            lot_id: None,
            purchase_id: Some(old.id),
            cost_paisa: 0,
            entry_id: new_entry_id,
            move_date: &old.entry_date,
            kind: "received",
            qty: ready.qty.get(),
            reason: None,
            reverses_move_id: None,
        },
        user,
        &now,
    )?;
    ventures::finish_edit_parts(tx, user, &now, entry.id, &retired, true)?;
    edit_windows::close(tx, Window::VentureEntry, entry.id)?;
    edit_windows::close(tx, Window::Purchase, old.id)?;
    // The consistency rule of PRD C21 holds for the new figures as for any change of a purchase.
    super::ensure_consistent(tx, old.id)?;

    // 4. The audit log keeps the old and the new values.
    let new_value = row_value(tx, old.id)?;
    let mut event = Event::new(Some(user), "edit", "purchase")
        .entity_id(old.id)
        .old(json!({ "number": old.number, "entry_id": old.entry_id, "row": old_value }))
        .new_value(json!({
            "number": old.number,
            "entry_id": new_entry_id,
            "reversal_entry_id": reversal_entry_id,
            "row": new_value,
            "cash_after_paisa": cash_check.cash_after_paisa.as_paisa(),
            "cash_below_zero_confirmed": cash_check.needs_confirm,
        }));
    if let Some(reason) = &reason {
        event = event.reason(reason);
    }
    event = event.sign(crate::day::fix_sign(tx, &old.entry_date)?.as_deref());
    ctx.db.auditor().write(tx, &event.into_entry())?;
    let version = repo::get(tx, old.id)?
        .map(|p| p.version)
        .ok_or_else(|| data("a purchase is missing after its edit"))?;
    Ok(Edited {
        id: old.id,
        number: Some(old.number),
        version,
        entry_id: new_entry_id,
        reversal_entry_id,
    })
}
