//! The edit of a garden installment, running cost or sale in another shop (PRD FR-7.70, FR-12.35,
//! FR-12.36; plan 44 T2a part 2).
//!
//! The garden must be open. A payment can be corrected at any time while the garden is open (it
//! touches no goods); a sale in another shop must be the only thing that moved goods of its arrival
//! (`goods_moved`). The edit takes the entry back like a cancel (the posting is reversed; a sale gives
//! its نگ back to the arrival), then checks and works out the entry again with the same function the
//! save uses ([`entry::plan`], Code Standards B5), and writes the new parts, the new posting and the new
//! stock move on the same day inside the entry's edit window.
//!
//! Because a payment has a reason of its own (why it was paid), the reason of the edit is
//! `edit_reason`.
//!
//! Example (E2E-40): the installment of 30,000 at 60 / 40 (الف from the shop, ب from his pocket) is
//! edited to 32,000: the parts are 19,200 / 12,800 and only 19,200 posts.

use mandi_db::arrivals as arrival_repo;
use mandi_db::bikri::edit::{self as edit_repo, NewMove};
use mandi_db::edit_windows::{self, Window};
use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as repo};
use mandi_domain::cash;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::edit::GardenEntryEditInput;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::gardens::GardenEntryInput;
use mandi_domain::core::validate;
use mandi_domain::ledger::{PostingType, SourceType};
use serde_json::{Value, json};

use super::data;
use super::entry::{Body, plan};
use super::read::load;
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash::cash_in_hand;
use crate::context::Context;
use crate::edit::entries::{Edited, net_cash_check, not_allowed};
use crate::edit::{date_fixed, reason_for_at};
use crate::error::{ServiceError, money_error};
use crate::ledger;
use crate::ventures::{self, Settlement, not_found};
use crate::vouchers::stored_cash_change;

/// A garden entry as the change list of an edit compares it: the money, why or the note, the shop
/// and quantity of a sale, and the partners' parts of a payment.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ServiceError::Db`.
pub(crate) fn row_value(conn: &Connection, id: i64) -> Result<Value, ServiceError> {
    let entry = repo::get_entry(conn, id)?
        .filter(|e| matches!(e.kind.as_str(), "installment" | "cost" | "other_sale"))
        .ok_or_else(|| not_found("id"))?;
    let parts = repo::parts_of(conn, &[id])?;
    let names = |r: &mandi_domain::core::accounts::RecordRef| json!({ "name_ur": r.name_ur, "name_en": r.name_en });
    Ok(json!({
        "amount_paisa": entry.amount_paisa,
        "reason": entry.reason,
        "shop": entry.shop_name,
        "qty": entry.qty,
        "parts": parts
            .iter()
            .map(|p| json!({
                "partner": names(&p.partner),
                "share_bps": p.share_bps,
                "source": p.source,
                "amount_paisa": p.amount_paisa,
            }))
            .collect::<Vec<_>>(),
    }))
}

/// The kind of entry an input is.
fn kind_text(entry: &GardenEntryInput) -> &'static str {
    match entry {
        GardenEntryInput::Installment(_) => "installment",
        GardenEntryInput::Cost(_) => "cost",
        GardenEntryInput::OtherSale(_) => "other_sale",
    }
}

/// The edit inside the transaction of the caller: the save and the rehearsal of the review are this
/// one function (Code Standards B5).
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_GARDEN_ENTRY_CANCELLED` and `ERR_GARDEN_CLOSED` on `id`;
/// `ERR_VERSION_CHANGED` on `version`; `ERR_EDIT_DATE_FIXED` on `date`; `ERR_FORMAT_INVALID` on
/// `type`, `garden_id` and `arrival_id`; `ERR_REASON_REQUIRED` on `edit_reason`;
/// `ERR_EDIT_NOT_ALLOWED` on `id` (rule `goods_moved`); every error of a new entry at its field;
/// `ERR_DATE_CLOSED`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. Nothing is written when any of them is
/// returned.
#[allow(clippy::too_many_lines)]
pub(crate) fn edit_in(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &GardenEntryEditInput,
) -> Result<Edited, ServiceError> {
    let entry = repo::get_entry(tx, input.id)?
        .filter(|e| matches!(e.kind.as_str(), "installment" | "cost" | "other_sale"))
        .ok_or_else(|| not_found("id"))?;
    let garden = load(tx, entry.venture_id)?.ok_or_else(|| not_found("id"))?;
    if entry.status == "cancelled" {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrGardenEntryCancelled,
            "id",
        ));
    }
    if !garden.is_open() {
        return Err(ServiceError::reject_field(ErrorCode::ErrGardenClosed, "id"));
    }
    if entry.version != input.version {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrVersionChanged,
            "version",
        ));
    }
    let old_date = BusinessDate::parse_iso(&entry.entry_date)
        .ok_or_else(|| data("the date of a garden entry is not a date"))?;
    // The kind, the garden, the day and (for a sale) the arrival never change.
    if kind_text(&input.entry) != entry.kind {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "type",
        ));
    }
    let (typed_garden, typed_date, typed_arrival) = match &input.entry {
        GardenEntryInput::Installment(p) | GardenEntryInput::Cost(p) => {
            (p.garden_id, p.date.as_str(), entry.arrival_id)
        }
        GardenEntryInput::OtherSale(s) => (s.garden_id, s.date.as_str(), s.arrival_id),
    };
    if validate::date("date", typed_date, window.1).is_ok_and(|d| d != old_date) {
        return Err(date_fixed("date", old_date));
    }
    if typed_garden != Some(entry.venture_id) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "garden_id",
        ));
    }
    if typed_arrival != entry.arrival_id {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "arrival_id",
        ));
    }
    let reason = reason_for_at("edit_reason", window.1, old_date, &input.reason)?;
    // A sale in another shop must be the only thing that moved goods of its arrival.
    if let (Some(arrival_id), Some(qty)) = (entry.arrival_id, entry.qty) {
        let arrival = arrival_repo::get(tx, arrival_id)?.ok_or_else(|| not_found("arrival_id"))?;
        let moved_total = arrival
            .qty_sold
            .checked_add(arrival.qty_written_off)
            .ok_or_else(|| data("the quantity of an arrival does not fit"))?;
        if moved_total != qty {
            return Err(not_allowed("id", "goods_moved"));
        }
    }
    let user = actor.user_id;
    let now = ctx.now_text();
    let old_value = row_value(tx, entry.id)?;
    let cash_before = cash_in_hand(tx)?;
    // A sale in another shop always posts its money; a payment with every part from a pocket does not.
    if entry.kind == "other_sale" && entry.journal_entry_id.is_none() {
        return Err(data("a sale in another shop has no journal entry"));
    }
    let old_cash = match entry.journal_entry_id {
        Some(journal_id) => {
            let stored = ledger_repo::get_entry(tx, journal_id)?
                .ok_or_else(|| data("the entry of a garden entry cannot be read"))?;
            stored_cash_change(tx, &stored)?
        }
        None => mandi_domain::core::money::Paisa::ZERO,
    };

    // 1. Take the entry back: the posting is reversed; a sale gives its نگ back to the arrival.
    let reversal_entry_id = match entry.journal_entry_id {
        Some(journal_id) => {
            let reversal =
                ledger::reverse(ctx, tx, user, journal_id, reason.as_deref().unwrap_or(""))?;
            if let (Some(arrival_id), Some(qty)) = (entry.arrival_id, entry.qty) {
                edit_repo::reverse_moves(tx, journal_id, reversal.entry_id, user, &now)?;
                let qty_back = qty
                    .checked_neg()
                    .ok_or_else(|| data("the quantity does not fit"))?;
                let moved_back = entry
                    .moved_qty
                    .checked_neg()
                    .ok_or_else(|| data("the quantity does not fit"))?;
                arrival_repo::add_sold_in_other_shop(
                    tx, arrival_id, qty_back, moved_back, user, &now,
                )?;
            }
            Some(reversal.entry_id)
        }
        None => None,
    };
    // 2. The same checking and working out as a new entry.
    let planned = plan(tx, window, &input.entry)?;
    if !planned.errors.is_empty() {
        return Err(ServiceError::from(planned.errors.clone()));
    }
    let ready = planned
        .ready
        .ok_or_else(|| data("an entry without errors has no numbers"))?;
    // The cash question. A payment asks inside the entry (its shop parts, after the reversal gave the
    // old ones back); a sale asks about the net change of the money received.
    let (cash_check, confirmed) = match &input.entry {
        GardenEntryInput::Installment(p) | GardenEntryInput::Cost(p) => (
            net_cash_check(cash_before, old_cash, planned.cash.change_paisa)?,
            p.confirm_cash_below_zero,
        ),
        GardenEntryInput::OtherSale(_) => {
            let net = ready.amount.checked_sub(old_cash).map_err(money_error)?;
            (
                cash::check(cash_before, net).map_err(money_error)?,
                input.confirm_cash_below_zero,
            )
        }
    };
    crate::cash::require_confirmed(&cash_check, confirmed)?;

    // 3. Rewrite the entry, its parts, its posting and its stock move inside the entry's window.
    // Only a payment (an installment or a cost) has partner parts; a sale in another shop credits the
    // garden's account instead.
    let expect_parts = matches!(&ready.body, Body::Payment { .. });
    let (qty, moved) = match &ready.body {
        Body::OtherSale { qty, moved, .. } => (Some(*qty), *moved),
        Body::Payment { .. } => (None, 0),
    };
    edit_windows::open(tx, Window::VentureEntry, entry.id, user, &now)?;
    let retired = repo::retire_parts(tx, entry.id, user, &now)?;
    repo::update_entry_edited(
        tx,
        entry.id,
        ready.amount.as_paisa(),
        ready.reason.as_deref(),
        qty,
        ready.shop.as_deref(),
        0,
        moved,
        None,
        user,
        &now,
    )?;
    let (settled, posting_type) = match &ready.body {
        Body::Payment { payment } => {
            ventures::write_parts(tx, user, &now, entry.id, &payment.parts)?;
            (payment.entry.as_ref(), PostingType::GardenPayment)
        }
        Body::OtherSale { entry: posted, .. } => (Some(posted), PostingType::VentureReceipt),
    };
    let new_entry_id = ventures::post_settlement(
        ctx,
        tx,
        user,
        &now,
        &Settlement {
            venture_entry_id: entry.id,
            date: old_date,
            entry: settled,
            posting_type,
            source_type: SourceType::Venture,
            source_id: entry.id,
        },
    )?;
    if let (Some(arrival_id), Some(qty)) = (entry.arrival_id, qty) {
        let qty_out = qty
            .checked_neg()
            .ok_or_else(|| data("the quantity does not fit"))?;
        edit_repo::insert_move(
            tx,
            &NewMove {
                arrival_id: Some(arrival_id),
                lot_id: None,
                purchase_id: None,
                cost_paisa: 0,
                entry_id: new_entry_id,
                move_date: &entry.entry_date,
                kind: "sold",
                qty: qty_out,
                reason: None,
                reverses_move_id: None,
            },
            user,
            &now,
        )?;
        arrival_repo::add_sold_in_other_shop(tx, arrival_id, qty, moved, user, &now)?;
    }
    ventures::finish_edit_parts(tx, user, &now, entry.id, &retired, expect_parts)?;
    edit_windows::close(tx, Window::VentureEntry, entry.id)?;

    // 4. The audit log keeps the old and the new values.
    let mut event = Event::new(Some(user), "edit", "garden_entry")
        .entity_id(entry.id)
        .old(json!({
            "number": entry.number,
            "kind": entry.kind,
            "entry_id": entry.journal_entry_id,
            "row": old_value,
        }))
        .new_value(json!({
            "number": entry.number,
            "kind": entry.kind,
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
        .ok_or_else(|| data("a garden entry is missing after its edit"))?;
    Ok(Edited {
        id: entry.id,
        number: Some(entry.number),
        version,
        entry_id: new_entry_id,
        reversal_entry_id,
    })
}
