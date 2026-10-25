//! Save, edit and cancel of a fruit entry (PRD FR-16.10 to FR-16.12, plan 17d). Each is one
//! transaction. The numbers come from [`plan`], the same function the preview uses (Code Standards
//! B5). No cash moves, so there is no cash question.

use mandi_db::bikri::edit as edit_repo;
use mandi_db::fruit::{self as frepo, FruitRec, NewFruit};
use mandi_db::gardens as garden_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::fruit::{
    FruitCancelInput, FruitCancelled, FruitEditInput, FruitEdited, FruitInput, FruitSaved,
};
use mandi_domain::ledger::{PostingType, SourceType};
use serde_json::{Value, json};

use super::plan::{Ready, plan};
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::ServiceError;
use crate::ledger::{self, PostingEvent};
use crate::ventures::{not_found, require_open_day, required_reason};

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

fn columns<'a>(ready: &'a Ready, date_iso: &'a str) -> NewFruit<'a> {
    NewFruit {
        entry_date: date_iso,
        partner_id: ready.partner_id,
        source: ready.source.as_str(),
        lot_id: ready.lot_id,
        credit_account_id: ready.credit_account_id,
        pukhta_person_key: ready.person_key.as_deref(),
        item_id: ready.item_id,
        packing_id: ready.packing_id,
        qty: ready.qty,
        weight_grams: ready.weight_grams,
        rate_paisa: ready.rate.as_paisa(),
        value_paisa: ready.value.as_paisa(),
        note: ready.note.as_deref(),
    }
}

/// Posts the journal entry of an entry. `order_key` keeps the place of the posting an edit replaces.
fn post_entry(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    ready: &Ready,
    source_id: i64,
    order_key: Option<i64>,
) -> Result<i64, ServiceError> {
    let posted = ledger::post_ordered(
        ctx,
        tx,
        user_id,
        &PostingEvent {
            date: ready.date,
            posting_type: PostingType::FruitTaken,
            source_type: SourceType::Fruit,
            source_id,
            narration: None,
            reverses: None,
            entry: ready.entry.clone(),
        },
        order_key,
    )?;
    Ok(posted.entry_id)
}

/// The lot's `qty_taken` goes up and a stock move of kind `taken` is written (C16).
fn take_from_lot(
    tx: &Connection,
    ready: &Ready,
    date_iso: &str,
    entry_id: i64,
    user_id: i64,
    now: &str,
) -> Result<(), ServiceError> {
    let Some(lot_id) = ready.lot_id else {
        return Ok(());
    };
    frepo::add_taken(tx, lot_id, ready.qty, user_id, now)?;
    let negative = ready
        .qty
        .checked_neg()
        .ok_or_else(|| data("the quantity of a fruit entry does not fit"))?;
    edit_repo::insert_move(
        tx,
        &edit_repo::NewMove {
            lot_id: Some(lot_id),
            purchase_id: None,
            cost_paisa: 0,
            entry_id: Some(entry_id),
            move_date: date_iso,
            kind: "taken",
            qty: negative,
            reason: None,
            reverses_move_id: None,
        },
        user_id,
        now,
    )?;
    Ok(())
}

fn audit_values(rec: &FruitRec) -> Value {
    json!({
        "number": rec.number,
        "entry_date": rec.entry_date,
        "partner_id": rec.partner.as_ref().map(|p| p.id),
        "source": rec.source,
        "lot_id": rec.lot.map(|l| l.0),
        "credit_account_id": rec.credit.id,
        "item_id": rec.item.id,
        "packing_id": rec.packing.id,
        "qty": rec.qty,
        "weight_grams": rec.weight_grams,
        "rate_paisa": rec.rate_paisa,
        "value_paisa": rec.value_paisa,
        "status": rec.status,
    })
}

/// The entry to change: found, in force, the version read, its day open, not a closed garden's.
fn load_open(
    ctx: &Context,
    tx: &Connection,
    id: i64,
    version: i64,
) -> Result<FruitRec, ServiceError> {
    let old = frepo::get(tx, id)?.ok_or_else(|| not_found("id"))?;
    if old.status == "cancelled" {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFruitCancelled,
            "id",
        ));
    }
    if old.version != version {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrVersionChanged,
            "version",
        ));
    }
    let date = BusinessDate::parse_iso(&old.entry_date)
        .ok_or_else(|| data("the date of a fruit entry is not a date"))?;
    require_open_day(ctx, date, "id")?;
    // The money of a closed garden was paid out to its partners: undo the closing first.
    if lot_of_closed_garden(tx, &old)? {
        return Err(ServiceError::reject_field(ErrorCode::ErrGardenClosed, "id"));
    }
    Ok(old)
}

pub(super) fn lot_of_closed_garden(
    conn: &Connection,
    rec: &FruitRec,
) -> Result<bool, ServiceError> {
    let Some((lot_id, ..)) = rec.lot else {
        return Ok(false);
    };
    match edit_repo::lot_state(conn, lot_id)? {
        Some(state) => Ok(garden_repo::account_is_closed(
            conn,
            state.seller_account_id,
        )?),
        None => Ok(false),
    }
}

/// The preview of an entry, with the entry being edited given back.
pub(super) fn preview_of(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &FruitInput,
) -> Result<mandi_domain::core::fruit::FruitPreview, ServiceError> {
    let editing = input
        .editing_id
        .map(|id| frepo::get(conn, id))
        .transpose()?
        .flatten()
        .filter(|rec| rec.status == "saved");
    Ok(plan(conn, window, input, editing.as_ref())?.preview())
}

/// Saves a fruit entry (PRD FR-16.10, FR-16.11).
///
/// # Errors
/// Every error of the preview, each at its field and all together, with nothing written.
pub(super) fn save(
    ctx: &Context,
    actor: &Actor,
    input: &FruitInput,
) -> Result<FruitSaved, ServiceError> {
    let window = day::window(ctx)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let planned = plan(tx, window, input, None)?;
        let preview = planned.preview();
        if !planned.errors.is_empty() {
            return Err(ServiceError::from(planned.errors));
        }
        let ready = planned
            .ready
            .ok_or_else(|| data("an entry without errors has no numbers"))?;
        let user = actor.user_id;
        let date_iso = ready.date.to_iso();
        let number = frepo::next_number(tx)?;
        let id = frepo::insert(tx, &columns(&ready, &date_iso), number, user, &now)?;
        let entry_id = post_entry(ctx, tx, user, &ready, id, None)?;
        frepo::set_entries(tx, id, Some(entry_id), entry_id, user, &now)?;
        take_from_lot(tx, &ready, &date_iso, entry_id, user, &now)?;
        let saved =
            frepo::get(tx, id)?.ok_or_else(|| data("a fruit entry is missing after its save"))?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "create", "fruit_take")
                .entity_id(id)
                .new_value(audit_values(&saved))
                .into_entry(),
        )?;
        Ok(FruitSaved {
            id,
            number,
            version: saved.version,
            journal_entry_id: entry_id,
            preview,
        })
    })
}

/// Edits an entry by reversal while its day is open: the old posting and its stock move are taken
/// back, the new ones are written, on the same entry and number (PRD FR-16.12).
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_FRUIT_CANCELLED`, `ERR_DATE_CLOSED` and `ERR_GARDEN_CLOSED` on `id`;
/// `ERR_VERSION_CHANGED`; `ERR_REASON_REQUIRED`; every error of the preview. Nothing is written when
/// one is returned.
pub(super) fn edit(
    ctx: &Context,
    actor: &Actor,
    input: &FruitEditInput,
) -> Result<FruitEdited, ServiceError> {
    let reason = required_reason(&input.reason)?;
    let window = day::window(ctx)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let old = load_open(ctx, tx, input.id, input.version)?;
        let planned = plan(tx, window, &input.entry, Some(&old))?;
        let preview = planned.preview();
        if !planned.errors.is_empty() {
            return Err(ServiceError::from(planned.errors));
        }
        let ready = planned
            .ready
            .ok_or_else(|| data("an entry without errors has no numbers"))?;
        let user = actor.user_id;
        let date_iso = ready.date.to_iso();
        let old_entry = old
            .current_entry_id
            .ok_or_else(|| data("a saved fruit entry has no posting"))?;
        // The old posting is taken back on its own day. The new one keeps its place among the day's
        // entries unless the edit moves it to another day.
        let order_key = if old.entry_date == date_iso {
            Some(ledger::order_key_of(tx, old_entry)?)
        } else {
            None
        };
        let reversal = ledger::reverse(ctx, tx, user, old_entry, &reason)?;
        edit_repo::reverse_moves(tx, old_entry, reversal.entry_id, user, &now)?;
        if let Some((lot_id, ..)) = old.lot {
            frepo::add_taken(tx, lot_id, -old.qty, user, &now)?;
        }
        let entry_id = post_entry(ctx, tx, user, &ready, old.id, order_key)?;
        frepo::update(tx, old.id, &columns(&ready, &date_iso), user, &now)?;
        frepo::set_entries(tx, old.id, None, entry_id, user, &now)?;
        take_from_lot(tx, &ready, &date_iso, entry_id, user, &now)?;
        let saved = frepo::get(tx, old.id)?
            .ok_or_else(|| data("a fruit entry is missing after its edit"))?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "edit", "fruit_take")
                .entity_id(old.id)
                .old(audit_values(&old))
                .new_value(audit_values(&saved))
                .reason(&reason)
                .into_entry(),
        )?;
        Ok(FruitEdited {
            id: old.id,
            number: old.number,
            version: saved.version,
            journal_entry_id: entry_id,
            reversal_entry_id: reversal.entry_id,
            preview,
        })
    })
}

/// Cancels an entry by reversal while its day is open (PRD FR-16.12).
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_FRUIT_CANCELLED`, `ERR_DATE_CLOSED` and `ERR_GARDEN_CLOSED` on `id`;
/// `ERR_VERSION_CHANGED` on `version`; `ERR_REASON_REQUIRED` on `reason`.
pub(super) fn cancel(
    ctx: &Context,
    actor: &Actor,
    input: &FruitCancelInput,
) -> Result<FruitCancelled, ServiceError> {
    let reason = required_reason(&input.reason)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let old = load_open(ctx, tx, input.id, input.version)?;
        let user = actor.user_id;
        let old_entry = old
            .current_entry_id
            .ok_or_else(|| data("a saved fruit entry has no posting"))?;
        let reversal = ledger::reverse(ctx, tx, user, old_entry, &reason)?;
        edit_repo::reverse_moves(tx, old_entry, reversal.entry_id, user, &now)?;
        if let Some((lot_id, ..)) = old.lot {
            frepo::add_taken(tx, lot_id, -old.qty, user, &now)?;
        }
        frepo::mark_cancelled(tx, old.id, &reason, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "cancel", "fruit_take")
                .entity_id(old.id)
                .old(audit_values(&old))
                .new_value(json!({
                    "status": "cancelled",
                    "reversal_entry_id": reversal.entry_id,
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        Ok(FruitCancelled {
            id: old.id,
            number: old.number,
            reversal_entry_id: reversal.entry_id,
        })
    })
}
