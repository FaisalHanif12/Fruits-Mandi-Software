//! The edit of a trip cost (PRD FR-7.70, FR-12.31; plan 44 T2a part 2).
//!
//! A cost can be corrected while nothing of the purchases it was spread over was sold or written off
//! (`goods_moved`). The edit takes the cost back like a cancel (the posting is reversed, each purchase
//! gives up its part), then checks and works out the cost again with the same function the save uses
//! ([`cost::plan_with`], Code Standards B5) over the SAME purchases, and writes the new parts, the new
//! spread and the new posting on the same day inside the cost's edit window.
//!
//! Example (E2E-41): a truck of 4,000 on a trip whose only purchase is 80 نگ is edited to 4,400: the
//! purchase's total cost goes from 100,000 to 100,400 and the parts are 2,200 / 2,200.

use mandi_db::edit_windows::{self, Window};
use mandi_db::ledger as ledger_repo;
use mandi_db::purchases as purchase_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::ventures as repo;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::edit::TripCostEditInput;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::validate;
use mandi_domain::ledger::{PostingType, SourceType};
use serde_json::{Value, json};

use super::cost::plan_with;
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash::{self, cash_in_hand};
use crate::context::Context;
use crate::edit::entries::{Edited, net_cash_check, not_allowed};
use crate::edit::{date_fixed, reason_for_at};
use crate::error::ServiceError;
use crate::ledger;
use crate::purchases;
use crate::ventures::{self, Settlement, not_found};
use crate::vouchers::stored_cash_change;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// A trip cost as the change list of an edit compares it: the amount, why, the partners' parts and
/// the spread over the purchases.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ServiceError::Db`.
pub(crate) fn row_value(conn: &Connection, id: i64) -> Result<Value, ServiceError> {
    let entry = repo::get_entry(conn, id)?
        .filter(|e| e.kind == "cost")
        .ok_or_else(|| not_found("id"))?;
    let parts = repo::parts_of(conn, &[id])?;
    let spread = purchase_repo::cost_parts_detail(conn, &[id])?;
    let names = |r: &mandi_domain::core::accounts::RecordRef| json!({ "name_ur": r.name_ur, "name_en": r.name_en });
    Ok(json!({
        "amount_paisa": entry.amount_paisa,
        "reason": entry.reason,
        "parts": parts
            .iter()
            .map(|p| json!({
                "partner": names(&p.partner),
                "share_bps": p.share_bps,
                "source": p.source,
                "amount_paisa": p.amount_paisa,
            }))
            .collect::<Vec<_>>(),
        "spread": spread
            .iter()
            .map(|s| json!({
                "item": names(&s.item),
                "purchase_number": s.purchase_number,
                "amount_paisa": s.amount_paisa,
            }))
            .collect::<Vec<_>>(),
    }))
}

/// The edit inside the transaction of the caller: the save and the rehearsal of the review are this
/// one function (Code Standards B5).
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_TRIP_ENTRY_CANCELLED` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_EDIT_DATE_FIXED` on `date`; `ERR_FORMAT_INVALID` on `trip_id`; `ERR_REASON_REQUIRED` on
/// `reason`; `ERR_EDIT_NOT_ALLOWED` on `id` (rule `goods_moved`); every error of a new cost at its
/// field; `ERR_DATE_CLOSED`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. Nothing is written when any of them
/// is returned.
pub(crate) fn edit_in(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &TripCostEditInput,
) -> Result<Edited, ServiceError> {
    let entry = repo::get_entry(tx, input.id)?
        .filter(|e| e.kind == "cost")
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
        .ok_or_else(|| data("the date of a trip cost is not a date"))?;
    let typed = &input.cost;
    if validate::date("date", &typed.date, window.1).is_ok_and(|d| d != old_date) {
        return Err(date_fixed("date", old_date));
    }
    if typed.trip_id != Some(entry.venture_id) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "trip_id",
        ));
    }
    // The cost has a reason of its own (why it was paid), so the reason of the edit is `edit_reason`.
    let reason = reason_for_at("edit_reason", window.1, old_date, &input.reason)?;
    // Nothing of the purchases it touched may have moved (the rule of a cancel).
    let old_spread = purchase_repo::cost_parts_of_entry(tx, entry.id)?;
    let touched: Vec<i64> = old_spread.iter().map(|p| p.purchase_id).collect();
    for rec in purchase_repo::get_many(tx, &touched)? {
        if rec.qty_sold != 0 || rec.qty_written_off != 0 {
            return Err(not_allowed("id", "goods_moved"));
        }
    }
    let user = actor.user_id;
    let now = ctx.now_text();
    let old_value = row_value(tx, entry.id)?;
    // The cash question is about the net change of the edit (PRD FR-6.7).
    let cash_before = cash_in_hand(tx)?;
    let old_cash = match entry.journal_entry_id {
        Some(journal_id) => {
            let stored = ledger_repo::get_entry(tx, journal_id)?
                .ok_or_else(|| data("the entry of a trip cost cannot be read"))?;
            stored_cash_change(tx, &stored)?
        }
        None => mandi_domain::core::money::Paisa::ZERO,
    };

    // 1. Take the cost back: the posting is reversed, each purchase gives up its part.
    let reversal_entry_id = match entry.journal_entry_id {
        Some(journal_id) => Some(
            ledger::reverse(ctx, tx, user, journal_id, reason.as_deref().unwrap_or(""))?.entry_id,
        ),
        None => None,
    };
    for part in &old_spread {
        let back = part
            .amount_paisa
            .checked_neg()
            .ok_or_else(|| data("the cost does not fit"))?;
        purchase_repo::add_trip_cost(tx, part.purchase_id, back, user, &now)?;
        purchases::ensure_consistent(tx, part.purchase_id)?;
    }
    // 2. The same checking and working out as a new cost, over the same purchases.
    let planned = plan_with(tx, window, typed, Some(&touched))?;
    if !planned.errors.is_empty() {
        return Err(ServiceError::from(planned.errors.clone()));
    }
    let cash_check = net_cash_check(cash_before, old_cash, planned.cash.change_paisa)?;
    cash::require_confirmed(&cash_check, typed.confirm_cash_below_zero)?;
    let ready = planned
        .ready
        .ok_or_else(|| data("a cost without errors has no numbers"))?;

    // 3. Rewrite the entry, its parts, its spread and its posting inside the cost's window.
    edit_windows::open(tx, Window::VentureEntry, entry.id, user, &now)?;
    let retired = repo::retire_parts(tx, entry.id, user, &now)?;
    let retired_spread = purchase_repo::retire_cost_parts(tx, entry.id, user, &now)?;
    repo::update_entry_edited(
        tx,
        entry.id,
        ready.amount.as_paisa(),
        Some(&ready.reason),
        None,
        None,
        0,
        0,
        None,
        user,
        &now,
    )?;
    ventures::write_parts(tx, user, &now, entry.id, &ready.payment.parts)?;
    let new_entry_id = ventures::post_settlement(
        ctx,
        tx,
        user,
        &now,
        &Settlement {
            venture_entry_id: entry.id,
            date: old_date,
            entry: ready.payment.entry.as_ref(),
            posting_type: PostingType::TripCost,
            source_type: SourceType::Venture,
            source_id: entry.id,
        },
    )?;
    // C24: each purchase carries its new part.
    for part in &ready.spread {
        purchase_repo::insert_cost_part(
            tx,
            entry.id,
            part.purchase_id,
            part.amount.as_paisa(),
            user,
            &now,
        )?;
        purchase_repo::add_trip_cost(tx, part.purchase_id, part.amount.as_paisa(), user, &now)?;
        purchases::ensure_consistent(tx, part.purchase_id)?;
    }
    purchase_repo::link_retired_cost_parts(tx, entry.id, &retired_spread, user, &now)?;
    if !purchase_repo::cost_parts_add_up(tx, entry.id)? {
        return Err(data(
            "the spread of a trip cost does not add up after an edit",
        ));
    }
    ventures::finish_edit_parts(tx, user, &now, entry.id, &retired, true)?;
    edit_windows::close(tx, Window::VentureEntry, entry.id)?;

    // 4. The audit log keeps the old and the new values.
    let mut event = Event::new(Some(user), "edit", "trip_cost")
        .entity_id(entry.id)
        .old(
            json!({ "number": entry.number, "entry_id": entry.journal_entry_id, "row": old_value }),
        )
        .new_value(json!({
            "number": entry.number,
            "entry_id": new_entry_id,
            "reversal_entry_id": reversal_entry_id,
            "row": row_value(tx, entry.id)?,
            "cash_after_paisa": cash_check.cash_after_paisa.as_paisa(),
            "cash_below_zero_confirmed": cash_check.needs_confirm,
        }));
    if let Some(reason) = &reason {
        event = event.reason(reason);
    }
    ctx.db.auditor().write(tx, &event.into_entry())?;
    let version = repo::get_entry(tx, entry.id)?
        .map(|e| e.version)
        .ok_or_else(|| data("a trip cost is missing after its edit"))?;
    Ok(Edited {
        id: entry.id,
        number: Some(entry.number),
        version,
        entry_id: new_entry_id,
        reversal_entry_id,
    })
}
